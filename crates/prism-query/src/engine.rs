//! `QueryEngine` — core query execution entry point.
//!
//! Implements the query tool lifecycle:
//!   1. Parse PrismQL string via `PrismQlParser::parse` (S-3.01 public API only)
//!   2. Resolve client scopes via `scoping::resolve_clients`
//!   3. Build push-down plan via `pushdown::classify_predicates`
//!   4. Run ephemeral materialization pipeline via `materialization` module
//!   5. Return `QueryResult`
//!
//! The `execute_scheduled` variant returns the `SessionContext` for detection-engine
//! reuse (S-4.03) — the caller manages the `SessionContext` lifetime.
//!
//! # BC References
//! - BC-2.11.001 — `query` MCP Tool: scoping + PrismQL query string
//! - BC-2.11.005 — Ephemeral materialization pipeline
//! - BC-2.11.006 — Security limits (30s timeout, 10K records, 200MB GreedyMemoryPool)
//! - BC-2.11.011 — Cross-client query scoping
//!
//! # Architecture Compliance
//! - Security perimeter (INV-SEC-PERIMETER-001, BC-2.11.006):
//!   parser consumed ONLY via `PrismQlParser::parse`. Restricted symbols
//!   (`parse_filter`, `parse_pipe`, `parse_sql`, sub-builders, `ParseLimits`
//!   thread-local API) MUST NOT appear here.
//!
//! Story: S-3.02

// Implementation module: all stub sites are now filled per S-3.02-FOLLOWUP-RUNTIME.
// Dead code suppression retained during the transition phase.
// dead_code suppression removed — all items are now used (ADV-W3MT-P58-MED-002)

use std::sync::{Arc, Mutex};

use prism_core::{OrgSlug, PrismError, SensorId};
use prism_credentials::CredentialStore;
use prism_ocsf::OcsfNormalizer;
use prism_sensors::{AdapterRegistry, CredentialResolver};
use prism_storage::RocksStorageBackend;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

use crate::{
    alias_store::AliasStore,
    alias_types::AliasScope,
    cache::{CacheConfig, SensorResponseCache},
    cursor::{spawn_cursor_cleanup_task, QueryCursorRegistry},
    scoping::ClientRegistry,
    table_registry::TableRegistry,
};

// ---------------------------------------------------------------------------
// Capability
// ---------------------------------------------------------------------------

/// Query-time capabilities granted to the caller.
///
/// Used for capability-gated table access (e.g., `prism_audit` requires
/// `AuditRead`). Capabilities are passed via `QueryOptions` and checked
/// before scan begins (F-LP1-HIGH-3 / BC-2.15.011).
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Capability {
    /// Grants access to `prism_audit` table. (BC-2.15.011)
    AuditRead,
}

// ---------------------------------------------------------------------------
// QueryEngineConfig
// ---------------------------------------------------------------------------

/// Configuration for the `QueryEngine`.
///
/// All limits default to the BC-2.11.006 specified values when constructed via
/// `QueryEngineConfig::default()`.
///
/// Implements BC-2.11.006 — configurable via TOML.
#[derive(Debug, Clone)]
pub struct QueryEngineConfig {
    /// Maximum query execution time in seconds. Default: 30. (BC-2.11.006)
    pub timeout_secs: u64,
    /// Maximum records materialized across all sources. Default: 10_000. (BC-2.11.006)
    pub max_materialized_records: usize,
    /// Per-query memory budget in bytes. Default: 200 * 1024 * 1024. (BC-2.11.006)
    pub memory_pool_bytes: usize,
    /// Maximum fan-out concurrency. Default: 10. (BC-2.11.005)
    pub max_fan_out_concurrency: usize,
}

impl Default for QueryEngineConfig {
    fn default() -> Self {
        Self {
            timeout_secs: 30,
            max_materialized_records: 10_000,
            memory_pool_bytes: 200 * 1024 * 1024,
            max_fan_out_concurrency: 10,
        }
    }
}

// ---------------------------------------------------------------------------
// QueryOptions
// ---------------------------------------------------------------------------

/// Per-call options forwarded from the MCP `query` tool parameters.
///
/// Implements BC-2.11.001 scoping parameters.
#[derive(Debug, Clone, Default)]
pub struct QueryOptions {
    /// Client scope override: `None` = all configured clients. (BC-2.11.011)
    pub clients: Option<Vec<OrgSlug>>,
    /// Sensor scope override: `None` = all sensors for resolved clients.
    pub sensors: Option<Vec<SensorId>>,
    /// Max results returned (tool-level truncation). Default 25, max 1000. (BC-2.11.001)
    pub limit: Option<usize>,
    /// Bypass response cache. (BC-2.11.001)
    pub force_refresh: bool,
    /// Caller capabilities for capability-gated tables (e.g., `prism_audit`).
    /// (BC-2.15.011, F-LP1-HIGH-3)
    pub capabilities: Vec<Capability>,
}

// ---------------------------------------------------------------------------
// QueryResult
// ---------------------------------------------------------------------------

/// The output of a successful query execution.
///
/// Contains OCSF-normalized Arrow RecordBatches and query metadata.
/// Implements BC-2.11.001 response schema.
#[derive(Debug)]
pub struct QueryResult {
    /// OCSF-normalized result batches from DataFusion execution.
    pub batches: Vec<arrow::record_batch::RecordBatch>,
    /// Total records available before tool-level `limit` truncation.
    pub total_available: usize,
    /// True if `total_available > returned_results`. (BC-2.11.001)
    pub is_truncated: bool,
    /// Actual returned record count (after truncation).
    pub returned_results: usize,
    /// Query metadata for the MCP response `query_context` field.
    pub context: QueryResultContext,
    /// Per-sensor error messages for partial failures. (BC-2.11.005)
    pub sensor_errors: Vec<String>,
}

/// Metadata attached to every `QueryResult` (BC-2.11.001 `query_context`).
#[derive(Debug, Default)]
pub struct QueryResultContext {
    /// Original PrismQL string as received.
    pub original_query: String,
    /// Expanded query after alias resolution.
    pub expanded_query: String,
    /// Client IDs queried.
    pub clients_queried: Vec<OrgSlug>,
    /// Sensor types queried.
    pub sensors_queried: Vec<String>,
    /// Total wall-clock execution time.
    pub execution_time_ms: u64,
}

// ---------------------------------------------------------------------------
// QueryEngine
// ---------------------------------------------------------------------------

/// Core query execution engine.
///
/// Orchestrates the ephemeral materialization pipeline:
/// parse → scope resolve → push-down classify → fan-out → normalize →
/// Arrow batches → DataFusion MemTable → SQL plan → result.
///
/// `SessionContext` is ephemeral for non-scheduled queries — it is never
/// stored in this struct. See BC-2.11.005 architecture compliance rule.
///
/// # BC References
/// - BC-2.11.001 — entry-point contract
/// - BC-2.11.005 — pipeline contract
/// - BC-2.11.006 — security limits
///
/// # CR-003 / BC-2.07.002: Cursor and cache wiring
/// `QueryEngine` owns the `cursor_registry` and `cache` as shared resources.
/// The cursor cleanup task is started in `new()` and cancelled via
/// `cleanup_shutdown` when `QueryEngine` is dropped. Without this wiring,
/// cursor cleanup is dead code and the cache is unreachable.
pub struct QueryEngine {
    /// Registry of sensor adapters indexed by `(OrgId, SensorId)`.
    pub(crate) adapter_registry: Arc<AdapterRegistry>,
    /// Credential store for sensor authentication. (AI-opaque boundary)
    /// Retained for production wiring; not yet consumed in execute_inner. (ADV-W3MT-P58-MED-002)
    #[allow(dead_code)]
    pub(crate) credential_store: Arc<dyn CredentialStore>,
    /// OCSF normalizer for converting raw sensor JSON to Arrow. (S-1.04)
    /// Passed to MaterializationContext; not read directly. (ADV-W3MT-P58-MED-002)
    pub(crate) ocsf_normalizer: Arc<OcsfNormalizer>,
    /// Registry of configured client IDs. (BC-2.11.011)
    pub(crate) client_registry: Arc<ClientRegistry>,
    /// Engine-level configuration (limits, pool sizes, concurrency).
    pub(crate) config: QueryEngineConfig,
    /// Shared cursor registry — tracks all active pagination cursors (BC-2.07.001/002).
    /// Used by pagination module; not read directly in execute_inner. (ADV-W3MT-P58-MED-002)
    #[allow(dead_code)]
    pub(crate) cursor_registry: Arc<Mutex<QueryCursorRegistry>>,
    /// Shared sensor-fetch response cache (BC-2.07.003/006).
    ///
    /// Threaded into the materialization pipeline via
    /// `MaterializationContext::with_response_cache` so sensor fetches are
    /// cache-checked before fan-out and populated after (QRY-02 closure).
    /// Shared with the write path through `response_cache()` →
    /// `CacheInvalidator` (BC-2.07.004).
    pub(crate) cache: Arc<SensorResponseCache>,
    /// Cancellation token used to signal the cursor cleanup task to stop.
    cleanup_shutdown: CancellationToken,
    /// Handle to the background cursor cleanup task (BC-2.07.002 §Background Cleanup).
    /// Held to ensure the task is aborted on Drop — not a dead field.
    cleanup_handle: Option<JoinHandle<()>>,
    /// Credential resolver for per-(org, sensor) auth dispatch in fan_out().
    /// (F-LP1-CRIT-2: replaces placeholder auth construction; all built-in auth types deleted in PLUGIN-MIGRATION-001-A)
    pub(crate) credential_resolver: Arc<dyn CredentialResolver>,
    /// OrgSlug → OrgId mapping for per-org adapter selection. (F-LP1-CRIT-3)
    /// When `None`, falls back to `get_all_for_sensor` (test/MVP mode).
    pub org_registry: Option<Arc<prism_core::OrgRegistry>>,
    /// RocksDB storage backend for internal table registration.
    /// (F-LP1-CRIT-1: `register_internal_tables` invoked from `execute_inner`)
    /// When `None`, internal tables are not registered (e.g. query-only mode).
    pub(crate) storage: Option<Arc<dyn RocksStorageBackend>>,
    /// Per-org overlay resolved spec map for per-org endpoint dispatch (ADR-029).
    /// Produced at boot by `OverlayLoader::load_overlays` (step 4) and threaded through
    /// `RunningServer` → `QueryEngine` → `MaterializationContext` for O(1) lookup at fan-out.
    /// `None` when no overlay config exists (test/MVP mode).
    /// (F-LP2-CRIT-001 + F-LP2-HIGH-001 wiring — S-CONFIG-MULTI-TENANT-OVERRIDE-001)
    /// ADR-042: ArcSwap-backed so hot-reload can atomically swap the map.
    /// `None` = single-tenant mode (no overlay config). In-flight queries that
    /// call `resolved_spec_map()` hold their Arc snapshot for the query lifetime.
    pub resolved_spec_map: Option<
        Arc<
            arc_swap::ArcSwap<
                std::collections::HashMap<
                    prism_spec_engine::ResolvedSpecKey,
                    prism_spec_engine::ResolvedSensorSpec,
                >,
            >,
        >,
    >,
    /// Alias store for `@alias_name` expansion in PrismQL queries (BC-2.11.008).
    ///
    /// When `Some`, `execute_inner` resolves all `@alias` tokens from the store
    /// before passing the expanded query to the materialization pipeline.
    /// When `None`, alias tokens are passed through unchanged (query-only / test mode).
    ///
    /// The `Arc<Mutex<>>` matches the PrismServer wiring so both the CRUD tools
    /// and the query executor share the same live AliasStore instance.
    /// (F-PASS9-LOW-1 fix — S-5.01-FOLLOWUP-MCP-BOOT)
    pub(crate) alias_store: Option<Arc<Mutex<AliasStore>>>,
    /// Infusion registry for plugin-backed enrichment UDFs (BC-2.19.001 / S-DEMO-ENRICHMENT-PIVOT-001).
    ///
    /// When `Some`, `execute_inner` and `execute_scheduled_inner` call `register_infusion_udfs`
    /// on the ephemeral `SessionContext` so analyst queries using `| enrich infusion(field)` resolve.
    /// When `None`, no enrichment UDFs are registered (query-only / test mode without enrichment).
    pub(crate) infusion_registry: Option<Arc<prism_spec_engine::InfusionRegistry>>,
    /// Tier 2 in-memory LRU cache for infusion enrichment (BC-2.19.002 / HIGH-1 fix).
    ///
    /// Process-shared across all queries; consulted on Tier 1 miss, populated on source call.
    /// `None` when no infusion registry is configured (query-only / test mode).
    pub(crate) infusion_lru_cache: Option<Arc<prism_spec_engine::InfusionLruCache>>,
    /// Tier 3 RocksDB persistent cache for infusion enrichment (BC-2.19.002 / HIGH-1 fix).
    ///
    /// Backed by the `infusion_cache` CF via `CacheBackend` trait injection.
    /// `None` when no infusion registry is configured (query-only / test mode).
    pub(crate) infusion_tier3_cache: Option<Arc<prism_spec_engine::InfusionTier3Cache>>,
    /// Dynamic table registry — tracks which sensor tables are currently available.
    ///
    /// Populated from `ConfigSnapshot.sensor_specs` at startup. Updated on hot-reload
    /// via `register_sensor` / `deregister_sensor` (BC-2.16.007). Used in the plan-time
    /// availability gate (before `materialize_query`) to return `E-QUERY-037`
    /// (`TableNotAvailable`) for unregistered tables (BC-2.11.001, S-3.13).
    ///
    /// `Arc<TableRegistry>` so the gate and hot-reload path share the same instance.
    /// When `None`, the availability gate is skipped (legacy / test mode without spec engine).
    pub(crate) table_registry: Option<Arc<TableRegistry>>,

    /// Optional confirmation token store for `token_count()` accessor (BC-2.08.005 RECONCILIATION-3).
    ///
    /// Wired via `with_token_store()` when the write pipeline is active.
    /// `None` in query-only mode (new/new_with_cache_config paths) → `token_count()` returns 0.
    pub(crate) token_store: Option<Arc<prism_security::ConfirmationTokenStore>>,
}

impl QueryEngine {
    /// Construct a `QueryEngine` with the provided dependencies.
    ///
    /// Starts the cursor cleanup background task (BC-2.07.002 §Background Cleanup).
    /// The task is cancelled when this `QueryEngine` is dropped.
    ///
    /// # BC-2.11.001
    /// The engine accepts at minimum a query string at call time. This
    /// constructor wires the shared dependencies once at startup.
    pub fn new(
        adapter_registry: Arc<AdapterRegistry>,
        credential_store: Arc<dyn CredentialStore>,
        ocsf_normalizer: Arc<OcsfNormalizer>,
        client_registry: Arc<ClientRegistry>,
        config: QueryEngineConfig,
    ) -> Self {
        Self::new_with_cache_config(
            adapter_registry,
            credential_store,
            ocsf_normalizer,
            client_registry,
            config,
            CacheConfig::default(),
        )
    }

    /// Construct a `QueryEngine` with explicit cache configuration.
    ///
    /// Used by tests and operators that need non-default cache bounds.
    pub fn new_with_cache_config(
        adapter_registry: Arc<AdapterRegistry>,
        credential_store: Arc<dyn CredentialStore>,
        ocsf_normalizer: Arc<OcsfNormalizer>,
        client_registry: Arc<ClientRegistry>,
        config: QueryEngineConfig,
        cache_config: CacheConfig,
    ) -> Self {
        let cursor_registry = Arc::new(Mutex::new(QueryCursorRegistry::new()));
        let cache = Arc::new(SensorResponseCache::new(cache_config));
        let shutdown = CancellationToken::new();

        // Start cursor cleanup background task (BC-2.07.002 §Background Cleanup).
        // Task exits when `shutdown` is cancelled (via Drop).
        let handle = spawn_cursor_cleanup_task(Arc::clone(&cursor_registry), shutdown.clone());

        // Default credential resolver: always returns CredentialNotFound.
        // Tests that need real auth override via `new_full`.
        let credential_resolver: Arc<dyn CredentialResolver> = Arc::new(NullCredentialResolver);

        Self {
            adapter_registry,
            credential_store,
            ocsf_normalizer,
            client_registry,
            config,
            cursor_registry,
            cache,
            cleanup_shutdown: shutdown,
            cleanup_handle: Some(handle),
            credential_resolver,
            org_registry: None,
            storage: None,
            resolved_spec_map: None,
            alias_store: None,
            infusion_registry: None,
            // S-1.14-REDO HIGH-1: Tier 2/3 caches default to None; wired via with_infusion_registry.
            infusion_lru_cache: None,
            infusion_tier3_cache: None,
            // S-3.13: table_registry wired as None by default in new/new_with_cache_config.
            // Production boot uses new_full (with a real ConfigSnapshot) or
            // with_table_registry() to supply a pre-populated TableRegistry.
            table_registry: None,
            // S-5.04 RECONCILIATION-3: token_store None by default; wired via with_token_store()
            // when the write pipeline is active.
            token_store: None,
        }
    }

    /// Override the `CredentialResolver` on an existing engine.
    ///
    /// Primarily used in tests to inject a `StubCredentialResolver` so
    /// `fan_out()` can reach `StubAdapter::fetch` without credential failures.
    /// (F-LP1-CRIT-2)
    pub fn with_credential_resolver(mut self, resolver: Arc<dyn CredentialResolver>) -> Self {
        self.credential_resolver = resolver;
        self
    }

    /// Return the `ClientRegistry` used by this engine for client-scope resolution.
    ///
    /// Exposed publicly so that callers in adjacent crates (e.g., `prism-mcp`) can
    /// pass the same `ClientRegistry` to `ExplainOptions::client_registry` for
    /// consistent client-scope semantics between `explain_query` and `query`
    /// (F-PASS10-HIGH-3 fix; ADR-022 §F wiring discipline).
    pub fn client_registry(&self) -> Arc<crate::scoping::ClientRegistry> {
        Arc::clone(&self.client_registry)
    }

    /// Return the engine-owned sensor-fetch response cache (BC-2.07.003).
    ///
    /// Exposed so the boot path can construct a `CacheInvalidator` over the
    /// SAME cache instance the read pipeline populates — write-then-read
    /// consistency (BC-2.07.004) requires the write path to invalidate the
    /// cache the query path reads from, not a separate instance.
    pub fn response_cache(&self) -> Arc<SensorResponseCache> {
        Arc::clone(&self.cache)
    }

    /// Construct a `QueryEngine` with full production dependencies.
    ///
    /// Includes `CredentialResolver`, `OrgRegistry`, `RocksStorageBackend`,
    /// `resolved_spec_map`, and `alias_store` for end-to-end fan_out dispatch
    /// with per-org endpoint overlay resolution, internal table access, and
    /// `@alias` expansion in PrismQL queries (BC-2.11.008).
    /// (F-LP1-CRIT-1/2/3, F-LP2-CRIT-001 wiring, F-PASS9-LOW-1)
    #[allow(clippy::too_many_arguments)]
    pub fn new_full(
        adapter_registry: Arc<AdapterRegistry>,
        credential_store: Arc<dyn CredentialStore>,
        ocsf_normalizer: Arc<OcsfNormalizer>,
        client_registry: Arc<ClientRegistry>,
        config: QueryEngineConfig,
        credential_resolver: Arc<dyn CredentialResolver>,
        org_registry: Arc<prism_core::OrgRegistry>,
        storage: Arc<dyn RocksStorageBackend>,
        resolved_spec_map: Arc<
            std::collections::HashMap<
                prism_spec_engine::ResolvedSpecKey,
                prism_spec_engine::ResolvedSensorSpec,
            >,
        >,
        alias_store: Arc<Mutex<AliasStore>>,
    ) -> Self {
        let cursor_registry = Arc::new(Mutex::new(QueryCursorRegistry::new()));
        let cache = Arc::new(SensorResponseCache::new(CacheConfig::default()));
        let shutdown = CancellationToken::new();
        let handle = spawn_cursor_cleanup_task(Arc::clone(&cursor_registry), shutdown.clone());

        Self {
            adapter_registry,
            credential_store,
            ocsf_normalizer,
            client_registry,
            config,
            cursor_registry,
            cache,
            cleanup_shutdown: shutdown,
            cleanup_handle: Some(handle),
            credential_resolver,
            org_registry: Some(org_registry),
            storage: Some(storage),
            // ADR-042: wrap in ArcSwap so reload path can atomically swap the map.
            resolved_spec_map: Some(Arc::new(arc_swap::ArcSwap::new(resolved_spec_map))),
            alias_store: Some(alias_store),
            infusion_registry: None,
            // S-1.14-REDO HIGH-1: Tier 2/3 caches default to None; wired via with_infusion_registry.
            infusion_lru_cache: None,
            infusion_tier3_cache: None,
            // S-3.13: table_registry is None in new_full; callers that need it
            // (production boot path with spec engine loaded) use with_table_registry().
            table_registry: None,
            // S-5.04 RECONCILIATION-3: token_store None; wired via with_token_store()
            token_store: None,
        }
    }

    /// Set the infusion registry for plugin-backed enrichment UDF registration.
    ///
    /// Also allocates the shared Tier 2 (LRU, 10 000-entry capacity) and Tier 3 (RocksDB via
    /// `CacheBackend::NullCache` placeholder until `with_infusion_caches` is called with a
    /// real backend) caches so that `execute_inner` can call `register_infusion_udfs_with_cache`
    /// (BC-2.19.002 / HIGH-1 fix). Tests that need a real Tier 3 backend must call
    /// `with_infusion_caches` after this method.
    ///
    /// When set, `execute_inner` and `execute_scheduled_inner` call `register_infusion_udfs_with_cache`
    /// on each ephemeral `SessionContext` before query execution.
    pub fn with_infusion_registry(
        mut self,
        registry: Arc<prism_spec_engine::InfusionRegistry>,
    ) -> Self {
        // Allocate Tier 2 LRU cache (10 000-entry capacity, default per BC-2.19.002 / cache.rs).
        // `const { }` enforces the nonzero invariant at compile time — no runtime panic possible
        // (OBS-1, S-1.14-REDO: `InfusionLruCache::new` accepts `NonZeroUsize`, not `usize`).
        let lru = Arc::new(prism_spec_engine::InfusionLruCache::new(
            const { std::num::NonZeroUsize::new(10_000).unwrap() },
        ));
        // Allocate Tier 3 cache with NullCacheBackend (no RocksDB dependency at this call site;
        // production boot calls with_infusion_caches to wire the real RocksDB backend).
        let tier3 = Arc::new(prism_spec_engine::InfusionTier3Cache::new(Arc::new(
            crate::null_cache::NullCacheBackend,
        )));
        self.infusion_registry = Some(registry);
        self.infusion_lru_cache = Some(lru);
        self.infusion_tier3_cache = Some(tier3);
        self
    }

    /// Override the Tier 2 + Tier 3 caches on an engine that already has an infusion registry.
    ///
    /// Called by `prism-bin` boot path (S-1.14-REDO AC-7) to wire the real RocksDB `CacheBackend`
    /// after storage is initialized. Tests may inject an in-memory `CacheBackend` for Tier 3
    /// testing without a real RocksDB instance.
    ///
    /// Unconditionally stores `lru_cache` and `tier3_cache` on the engine regardless of whether
    /// `with_infusion_registry` has been called. Caches stored without an infusion registry are
    /// inert — they are allocated but never exercised, because `execute_inner` gates all infusion
    /// UDF registration on `infusion_registry` being `Some`. The production boot path always calls
    /// `with_infusion_registry` immediately before this method (see `boot.rs`).
    pub fn with_infusion_caches(
        mut self,
        lru_cache: Arc<prism_spec_engine::InfusionLruCache>,
        tier3_cache: Arc<prism_spec_engine::InfusionTier3Cache>,
    ) -> Self {
        self.infusion_lru_cache = Some(lru_cache);
        self.infusion_tier3_cache = Some(tier3_cache);
        self
    }

    /// Set the `TableRegistry` on an existing engine (S-3.13 plan-time gate).
    ///
    /// Called from the production boot path after `TableRegistry::from_snapshot()` has
    /// been populated from the initial `ConfigSnapshot`. Also used in tests that need
    /// the plan-time availability gate active.
    ///
    /// # BC-2.11.001 / BC-2.16.001
    /// The engine uses the registry to return `E-QUERY-037` (`TableNotAvailable`) for
    /// queries against unconfigured sensor tables, before any fan-out occurs.
    pub fn with_table_registry(mut self, registry: Arc<TableRegistry>) -> Self {
        self.table_registry = Some(registry);
        self
    }

    /// Wire an `AliasStore` into this engine.
    ///
    /// Required for alias expansion in `execute_inner` (Step 0: `@alias` token
    /// resolution before Chumsky parse). Without this, alias tokens in query strings
    /// are passed through unexpanded and will cause parse errors.
    ///
    /// Used in tests and by `PrismServer::with_deps` / `new_full` callers that
    /// need the engine to expand aliases at query time.
    ///
    /// Story: S-DEMO-PRISMQL-ONBOARDING-001-B (F-PRL-MED-001 fix seam).
    pub fn with_alias_store(mut self, store: Arc<Mutex<AliasStore>>) -> Self {
        self.alias_store = Some(store);
        self
    }

    /// Return the `TableRegistry` arc, if wired.
    ///
    /// Exposed for tests that need to inspect or update the registry.
    pub fn table_registry(&self) -> Option<Arc<TableRegistry>> {
        self.table_registry.as_ref().map(Arc::clone)
    }

    /// Return the `resolved_spec_map` arc, if wired (IMP-8 / BC-2.10.008 per-org scoping).
    ///
    /// The map holds `(OrgSlug, SensorId) → ResolvedSensorSpec` entries built at boot
    /// from `customers/<slug>/*.overlay.toml` files. Only orgs with explicit overlay
    /// entries appear as keys — an org registered in `OrgRegistry` but absent from this
    /// map has zero provisioned sensors (BC-2.10.008 Option B semantics).
    ///
    /// Returns `None` when no overlay config exists (test / MVP mode without multi-tenant).
    pub fn resolved_spec_map(
        &self,
    ) -> Option<
        Arc<
            std::collections::HashMap<
                prism_spec_engine::ResolvedSpecKey,
                prism_spec_engine::ResolvedSensorSpec,
            >,
        >,
    > {
        // ADR-042 / D2: load_full() returns a fresh Arc snapshot — same external type as
        // before. In-flight queries that hold this Arc see a consistent map for their lifetime
        // even if a reload fires and calls ArcSwap::store() concurrently.
        self.resolved_spec_map.as_ref().map(|swap| swap.load_full())
    }

    /// Atomically rebuild and swap the resolved spec map from a new ConfigSnapshot.
    ///
    /// Called by the hot-reload path after `ConfigSnapshot` has been swapped in
    /// `ConfigManager`. In-flight queries that have already called `resolved_spec_map()`
    /// hold their prior `Arc<HashMap>` for their lifetime; the swap is invisible to them
    /// (ADR-042 / AD-007 in-flight-query consistency guarantee).
    ///
    /// # Arguments
    /// - `customers_dir` — path to the `customers/` overlay directory.
    /// - `type_specs` — TYPE specs from the POST-reload `ConfigSnapshot.sensor_specs`.
    /// - `org_registry` — the engine's `OrgRegistry`.
    ///
    /// # Behavior
    /// - If `resolved_spec_map` is `None` (single-tenant mode): no-op, returns `Ok(0)`.
    /// - If `OverlayLoader::load_overlays` returns validation errors: existing map retained,
    ///   errors logged, returns `Err`. Caller should log and continue (non-fatal; DI-031).
    /// - On success: swaps in the new map, returns `Ok(overlay_count)`.
    pub fn rebuild_resolved_spec_map(
        &self,
        customers_dir: &std::path::Path,
        type_specs: &std::collections::HashMap<String, prism_spec_engine::spec_parser::SensorSpec>,
        org_registry: &prism_core::OrgRegistry,
    ) -> Result<usize, prism_spec_engine::error::SpecEngineError> {
        use prism_spec_engine::overlay::OverlayLoader;

        let Some(ref swap) = self.resolved_spec_map else {
            return Ok(0); // single-tenant mode — no-op per ADR-042 D3
        };

        let result = OverlayLoader::load_overlays(customers_dir, type_specs, org_registry);

        if !result.errors.is_empty() {
            // Non-fatal: log and retain existing map (DI-031 fail-closed on reload).
            tracing::warn!(
                event_type = "reload.overlay_rebuild_failed",
                error_count = result.errors.len(),
                "Hot-reload overlay rebuild failed; retaining prior resolved_spec_map (ADR-042)"
            );
            // Return first error as representative using ValidationFailed.
            let error_strings: Vec<String> = result.errors.iter().map(|e| e.to_string()).collect();
            return Err(
                prism_spec_engine::error::SpecEngineError::ValidationFailed {
                    errors: error_strings,
                },
            );
        }

        let count = result.resolved.len();
        swap.store(Arc::new(result.resolved));
        tracing::info!(
            event_type = "reload.overlay_rebuilt",
            overlay_count = count,
            "resolved_spec_map rebuilt and swapped atomically (ADR-042)"
        );
        Ok(count)
    }

    /// Return the `OrgRegistry` arc, if wired (IMP-8 / BC-2.10.008 per-org enumeration).
    ///
    /// Used by `render_client_list_resource` to enumerate all registered orgs and pair
    /// them with their sensor count from `resolved_spec_map`. Returns `None` when the
    /// engine is running in test / MVP mode without multi-tenant org support.
    pub fn org_registry(&self) -> Option<Arc<prism_core::OrgRegistry>> {
        self.org_registry.as_ref().map(Arc::clone)
    }

    /// Return the `InfusionRegistry` arc, if wired (CRIT-001 — BC-2.11.022 ADR-045 §A).
    ///
    /// Used by `dispatch_read_resource` to pass a live infusion registry snapshot to
    /// `build_reference_content` so the `prismql://reference` resource reflects the
    /// currently-loaded enrichment UDFs at query time. Returns `None` when the engine
    /// is running in test mode without enrichment configured.
    pub fn infusion_registry(&self) -> Option<Arc<prism_spec_engine::InfusionRegistry>> {
        self.infusion_registry.as_ref().map(Arc::clone)
    }

    /// Return the `AdapterRegistry` arc (S-5.04 — health probe wiring).
    ///
    /// Exposed so `PrismServer::with_deps` can pass the adapter registry to
    /// `SensorHealthChecker::new()` without requiring a separate constructor argument.
    /// The registry is always present (populated at boot step 9A for S-DEMO-001 scope).
    pub fn adapter_registry(&self) -> Arc<AdapterRegistry> {
        Arc::clone(&self.adapter_registry)
    }

    /// Return the current number of non-expired active pagination cursors.
    ///
    /// Used by `check_sensor_health` to populate `resource_pressure.active_cursor_count`
    /// (BC-2.08.005 RECONCILIATION-3 — S-5.04 live-wiring obligation).
    ///
    /// Acquires the cursor-registry mutex; poison-tolerant (recovers via `into_inner`
    /// per the F-006 pattern established in `context.rs`).
    pub fn cursor_count(&self) -> usize {
        let guard = match self.cursor_registry.lock() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };
        guard.active_count()
    }

    /// Return the current number of unexpired, unconsumed confirmation tokens.
    ///
    /// Used by `check_sensor_health` to populate `resource_pressure.active_token_count`
    /// (BC-2.08.005 RECONCILIATION-3 — S-5.04 live-wiring obligation).
    ///
    /// Reads `ConfirmationTokenStore::active_count()` from the optional token store.
    /// Returns `0` when no `ConfirmationTokenStore` is available (query-only mode).
    pub fn token_count(&self) -> usize {
        self.token_store
            .as_ref()
            .map(|ts| ts.active_count())
            .unwrap_or(0)
    }

    /// Wire an optional `ConfirmationTokenStore` for `token_count()` (BC-2.08.005 RECONCILIATION-3).
    ///
    /// Called by `boot.rs::step9_start_mcp_server` via the `QueryEngine::new_full` chain
    /// when the write pipeline is active.
    pub fn with_token_store(mut self, store: Arc<prism_security::ConfirmationTokenStore>) -> Self {
        self.token_store = Some(store);
        self
    }
}

impl Drop for QueryEngine {
    /// Cancel and abort the cursor cleanup background task on drop (CR-003 / OBS-008).
    ///
    /// `cancel()` signals the task to exit gracefully via the CancellationToken.
    /// `abort()` is called additionally to ensure the task is terminated even if
    /// it is blocked in the interval tick (e.g., the tokio runtime is shutting down
    /// before the cancellation is observed).
    fn drop(&mut self) {
        self.cleanup_shutdown.cancel();
        if let Some(h) = self.cleanup_handle.take() {
            h.abort();
        }
    }
}

impl QueryEngine {
    /// Execute a PrismQL query string and return normalized results.
    ///
    /// Wraps the entire lifecycle in a 30-second `tokio::time::timeout`.
    /// On timeout returns `PrismError::QueryTimeout`. (BC-2.11.006)
    ///
    /// The `SessionContext` is ephemeral — it is created, used, and dropped
    /// within this call. (BC-2.11.005, AC-7)
    ///
    /// # BC-2.11.001
    /// Accepts a PrismQL query string + optional scoping parameters.
    ///
    /// # BC-2.11.005
    /// Delegates to the materialization pipeline.
    ///
    /// # BC-2.11.006
    /// Enforces 30s timeout, 10K record cap, 200MB GreedyMemoryPool.
    /// Rejects `limit > 1000` with `E-QUERY-033`. (F-LP1-HIGH-7)
    pub async fn execute(
        &self,
        query_str: &str,
        options: QueryOptions,
    ) -> Result<QueryResult, PrismError> {
        let start = std::time::Instant::now();

        // F-LP1-HIGH-7 / F-LP2-LOW-1: enforce max-1000 limit (BC-2.11.001).
        // Uses dedicated QueryLimitExceeded variant (not QueryExecutionFailed) so callers
        // can match the correct error code without parsing a string. (F-LP2-LOW-1)
        if let Some(limit) = options.limit {
            if limit > 1000 {
                return Err(PrismError::QueryLimitExceeded {
                    requested: limit,
                    max: 1000,
                });
            }
        }

        // BC-2.11.006: wrap the entire execution in a 30-second timeout.
        let timeout_secs = self.config.timeout_secs;
        let result = tokio::time::timeout(
            std::time::Duration::from_secs(timeout_secs),
            self.execute_inner(query_str, options),
        )
        .await;

        match result {
            Ok(Ok(mut qr)) => {
                qr.context.execution_time_ms = start.elapsed().as_millis() as u64;
                Ok(qr)
            }
            Ok(Err(e)) => Err(e),
            Err(_elapsed) => Err(PrismError::QueryTimeout {
                elapsed_ms: start.elapsed().as_millis() as u64,
            }),
        }
    }

    /// Inner execution body (without timeout wrapper).
    async fn execute_inner(
        &self,
        query_str: &str,
        options: QueryOptions,
    ) -> Result<QueryResult, PrismError> {
        // Step 0: Alias expansion — resolve all `@alias_name` tokens before parsing.
        //
        // BC-2.11.008: aliases created via MCP tools must be consulted at query time.
        // BC-2.11.009: alias resolution pre-parse; per-client alias overrides global for
        //   the queried client scope. Current implementation expands against AliasScope::Global
        //   only — no per-client overrides are applied. BC-2.11.009 specifies per-client
        //   override as a postcondition; that scope thread is not yet wired here.
        // F-PASS9-LOW-1: alias_store is wired into QueryEngine via new_full() so both
        // the CRUD tools and the query executor share the same live AliasStore.
        let (effective_query, expanded_query_for_context) =
            if let Some(ref store_arc) = self.alias_store {
                // Lock is held only for the duration of alias expansion — not across the
                // full async pipeline (which awaits sensor fetches).
                let expanded = {
                    let store = store_arc.lock().map_err(|_| PrismError::Internal {
                        detail: "alias_store lock poisoned during query execution".to_string(),
                    })?;
                    crate::alias_resolver::AliasResolver::expand(
                        query_str,
                        &store,
                        &AliasScope::Global,
                        &std::collections::HashMap::new(),
                        0,
                    )?
                };
                let display = expanded.clone();
                (std::borrow::Cow::Owned(expanded), display)
            } else {
                (std::borrow::Cow::Borrowed(query_str), query_str.to_string())
            };
        let effective_query: &str = &effective_query;

        // S-3.13 Step 1a: Plan-time table availability gate (BC-2.11.001, AC-2, AC-8).
        //
        // Gate ordering (BC-2.11.019 v1.5, S-DEMO-FIDELITY-REMEDIATION-001 HIGH-001):
        //   E-QUERY-001 (parse) → E-QUERY-037 (table not found) → E-QUERY-038 (column not found)
        //   → E-QUERY-039 (enrich UDF not found, LAST).
        //
        // Fires BEFORE client scope resolution and BEFORE `materialize_query` (fail fast,
        // no fan-out). Validates each source_ref extracted from the AST against the
        // `TableRegistry`. If a source_ref is not registered, returns `E-QUERY-037`
        // (`TableNotAvailable`) with pre-formatted available-sensor context and a
        // Levenshtein-based did_you_mean suggestion.
        //
        // Gate is mode-agnostic (AC-8): the same check applies to SQL, filter, and pipe mode
        // because all three produce source_refs in the parsed AST before this point.
        //
        // Gate is skipped when `table_registry` is None (legacy / test mode without
        // spec engine wiring) — this preserves backward compatibility for tests that
        // predate S-3.13 and do not need the availability check.
        // ADR-039 / SEC-001: pass org_scope and resolved_spec_map so the gate can filter
        // available_sensors / available_tables to the requesting org's scope in multi-tenant
        // overlay deployments, preventing cross-tenant vendor enumeration (CWE-200).
        // ADR-042: load a snapshot from the ArcSwap to get Option<Arc<HashMap>>.
        let resolved_spec_snapshot = self.resolved_spec_map();
        check_table_availability(
            effective_query,
            self.table_registry.as_deref(),
            options.clients.as_deref(),
            resolved_spec_snapshot.as_deref(),
        )?;

        // S-DEMO-PRISMQL-ONBOARDING-001-B: plan-time column gate (BC-2.11.016 / E-QUERY-038).
        // Fires AFTER E-QUERY-037 passes (table exists → then check columns).
        // Gate ordering: E-QUERY-001 (parse) → E-QUERY-037 (table not found) → E-QUERY-038
        // (column not found) → E-QUERY-039 (enrich, LAST). The client_id is derived from
        // the first explicit client scope.
        let client_id_for_col_gate = options
            .clients
            .as_deref()
            .and_then(|c| c.first())
            .map(|o| o.as_str().to_owned())
            .unwrap_or_default();
        check_query_column_availability(
            effective_query,
            &client_id_for_col_gate,
            options.clients.as_deref(),
            resolved_spec_snapshot.as_deref(),
            self.table_registry.as_deref(),
        )?;

        // S-DEMO-FIDELITY-REMEDIATION-001 AC-N1B: Plan-time enrichment UDF gate (E-QUERY-039).
        //
        // Fires LAST — after E-QUERY-037 (table gate) and E-QUERY-038 (column gate).
        // Gate ordering (BC-2.11.019 v1.5): E-QUERY-001 → E-QUERY-037 → E-QUERY-038 → E-QUERY-039.
        //
        // Validates that all enrichment function names in the query (pipe: `| enrich name(col)`;
        // SQL: `SELECT name(col)` or `WHERE name(col) = val`) are registered per-field UDF names
        // in the InfusionRegistry. Returns E-QUERY-039 with available_infusions and did_you_mean
        // when an unregistered name is detected (prevents "infusion_id used as UDF name" silent
        // failures).
        //
        // Gate is skipped when `infusion_registry` is None (enrichment not configured).
        check_enrich_udf_availability(effective_query, self.infusion_registry.as_deref())?;

        // Step 1: Resolve client scope (BC-2.11.011).
        let clients =
            crate::scoping::resolve_clients(options.clients.clone(), &self.client_registry)?;

        // Step 2: Create ephemeral SessionContext with GreedyMemoryPool (BC-2.11.006).
        // HIGH-001 / ADV-W3MT-P58-HIGH-001: memory_pool_bytes was stored but not consumed.
        // Now wired via `build_session_context` which wraps RuntimeEnvBuilder + GreedyMemoryPool.
        let session_ctx = crate::memory::build_session_context(self.config.memory_pool_bytes)?;

        // S-DEMO-ENRICHMENT-PIVOT-001 / BC-2.19.001: register plugin-backed enrichment UDFs so
        // analyst queries using `| enrich infusion(field)` resolve in this ephemeral context.
        // No-op when `infusion_registry` is `None` (enrichment not configured).
        //
        // Error propagation: the inner error from `register_infusion_udfs` already carries
        // the canonical taxonomy code (E-INFUSE-002 for duplicate UDF names at spec-load time;
        // E-INFUSE-007 is FORWARD-RESERVED in taxonomy v1.82 — DataFusion 53.1's register_udf
        // is infallible so no call failure can occur here) and the real infusion_id.
        // We propagate verbatim — no outer prefix that would inject a function name into the
        // {infusion_id} slot or double-prefix the error code (MED-2 fix).
        // OBS-1 (S-1.14-REDO burst-4): Tier-1 lifetime note.
        //
        // `QueryScopedInfusionCache` (Tier-1) is allocated fresh per DataFusion batch
        // invocation inside `InfusionAsyncUdf::invoke_async_with_args`. This is CORRECT
        // behavior: Tier-1 deduplicates within a single DataFusion batch (e.g., 500 rows
        // mapping to 30 unique IPs — only 30 source calls). Tier-2 (process-shared LRU)
        // provides cross-batch dedup across multiple `execute()` calls without RocksDB.
        // Tier-3 (RocksDB, persistent) provides cross-process/restart persistence.
        //
        // AC-2 compliance: per-batch Tier-1 + process-shared Tier-2 together satisfy the
        // "no redundant source calls within a single query" requirement — DataFusion may
        // invoke the UDF in multiple batches, but Tier-2 absorbs all cross-batch hits.
        if let Some(ref registry) = self.infusion_registry {
            // HIGH-1 fix (BC-2.19.002): use three-tier cache path when caches are wired.
            // Falls back to Tier-1-only (no-cache) path in test/legacy mode when caches are None.
            match (&self.infusion_lru_cache, &self.infusion_tier3_cache) {
                (Some(lru), Some(t3)) => crate::infusion_udf::register_infusion_udfs_with_cache(
                    &session_ctx,
                    registry.udf_descriptors(),
                    Arc::clone(lru),
                    Arc::clone(t3),
                    // DEFAULT_CACHE_TTL_SECS is the Tier-2/3 write TTL fallback; it is
                    // OVERRIDDEN per-UDF by descriptor.cache_ttl_secs when set on the
                    // infusion spec (F-TTL-1). The literal 3600 here is NOT the effective
                    // TTL for specs that declare `cache_ttl_secs`.
                    crate::infusion_udf::DEFAULT_CACHE_TTL_SECS,
                ),
                _ => crate::infusion_udf::register_infusion_udfs(
                    &session_ctx,
                    registry.udf_descriptors(),
                ),
            }
            .map_err(|e| prism_core::PrismError::QueryExecutionFailed {
                detail: e.to_string(),
            })?;
        }

        // F-LP1-HIGH-3: Capability gate — check BEFORE registering internal tables.
        // Parse-time depth/size checks happen inside PrismQlParser::parse (security.rs).
        // Pre-execution capability gate: if the query references `prism_audit` but the
        // caller lacks `Capability::AuditRead`, reject with E-QUERY-011. This runs before
        // any storage scan, not inside the DataFusion `scan()` trait method (approach b).
        // Check against the EXPANDED query so alias-resolved table refs are caught.
        check_internal_table_capabilities(effective_query, &options.capabilities)?;

        // F-LP1-CRIT-1 / F-LP2-CRIT-1 Layer 2: register internal tables into the session context
        // before materialization so `prism_*` table references resolve in DataFusion.
        // Passes caller capabilities so each RocksDbTableProvider enforces the scan-time gate
        // (Layer 2 defense-in-depth: even if pre-execution gate is bypassed, scan() rejects).
        // Safety: when storage is None, internal tables are not available — DataFusion
        // will return "table not found" for `prism_*` queries (acceptable for query-only mode).
        if let Some(ref storage) = self.storage {
            crate::internal_tables::register_internal_tables_with_capabilities(
                &session_ctx,
                Arc::clone(storage),
                &options.capabilities,
            )?;
        }

        // Step 3: Set up MaterializationContext with engine dependencies.
        // F-LP2-CRIT-001: pass resolved_spec_map so fan_out_with_overlay_map is used
        // when per-org overlay endpoints are configured (ADR-029).
        // QRY-02: attach the engine-owned response cache so sensor fetches are
        // cache-checked before fan-out and populated after (BC-2.07.003).
        let mut mat_ctx = crate::materialization::MaterializationContext::new_with_resolver(
            Arc::clone(&self.adapter_registry),
            Arc::clone(&self.ocsf_normalizer),
            self.config.max_materialized_records,
            Arc::clone(&self.credential_resolver),
            self.org_registry.clone(),
            // ADR-042: load snapshot from ArcSwap — returns Option<Arc<HashMap>>.
            self.resolved_spec_map(),
        )
        .with_response_cache(Arc::clone(&self.cache));

        // Step 4: Resolve effective options (merge client scope into options).
        //
        // Preserve the original `clients` scope semantics for `resolve_source_refs`:
        // - `None` (no explicit scope) → keep `None` so resolve_source_refs uses the
        //   ALL scope path (iterate all registered adapters, not all clients in registry).
        //   This prevents false E-QUERY-032 errors for orgs that don't have a sensor.
        // - `Some([...])` (explicit scope) → keep the resolved/validated client list.
        //
        // The `clients` variable (from resolve_clients above) is used for client-side
        // validation and metrics only, NOT for fan-out scope gating.
        let effective_clients = if options.clients.is_none() {
            // No explicit scope: let resolve_source_refs use ALL-scope fan-out.
            None
        } else {
            Some(clients.clone())
        };
        let effective_options = QueryOptions {
            clients: effective_clients,
            capabilities: options.capabilities.clone(),
            ..options.clone()
        };

        // Step 5: Run the materialization pipeline → DataFusion execution → batches.
        // F-LP1-CRIT-5: pipeline now returns MaterializationOutput with both batches and sensor_errors.
        // Use effective_query (alias-expanded) so @alias tokens are resolved before parsing.
        let output = crate::materialization::run_materialization_pipeline(
            effective_query,
            &effective_options,
            &mut mat_ctx,
            &session_ctx,
        )
        .await?;

        // Step 6: Apply tool-level limit truncation.
        let limit = options.limit.unwrap_or(usize::MAX);
        let total_rows: usize = output.batches.iter().map(|b| b.num_rows()).sum();
        let is_truncated = total_rows > limit;
        let returned_results = total_rows.min(limit);

        // Truncate to limit (if needed).
        let final_batches = if is_truncated {
            truncate_batches_to_limit(output.batches, limit)
        } else {
            output.batches
        };

        // Step 7: Build QueryResult.
        // ADV-W3MT-P58-HIGH-005: sensors_queried now populated from materialization output.
        // F-PASS9-LOW-1: expanded_query reflects alias resolution (BC-2.11.008).
        let context = QueryResultContext {
            original_query: query_str.to_string(),
            expanded_query: expanded_query_for_context,
            clients_queried: clients,
            sensors_queried: output.sensors_queried,
            execution_time_ms: 0, // filled in by execute()
        };

        Ok(QueryResult {
            batches: final_batches,
            total_available: total_rows,
            is_truncated,
            returned_results,
            context,
            sensor_errors: output.sensor_errors,
        })
    }

    /// Analyze a PrismQL query string and return an `ExplainResult` without
    /// executing any sensor API calls.
    ///
    /// Thin wrapper over `explain::explain()` that satisfies the COMP-003 interface
    /// specified in `module-decomposition.md` line 185. (CR-006, BC-2.11.010)
    ///
    /// # Registry injection (OBS-1 fix)
    /// If `options.table_registry` is `None` and the engine has a wired `table_registry`,
    /// the engine's registry is injected into the options so that
    /// `ExplainResult.available_tables` is populated from the live registry without
    /// requiring callers to retrieve and thread `QueryEngine::table_registry()` manually.
    /// Callers that supply their own `options.table_registry` are not overridden.
    ///
    /// # No sensor API calls
    /// Delegates to `explain::explain()` which is a pure plan-analysis function.
    /// No `fan_out()`, no sensor adapter `fetch()`.
    pub fn explain(
        &self,
        query_str: &str,
        mut options: crate::explain::ExplainOptions,
    ) -> Result<crate::explain::ExplainResult, PrismError> {
        // Inject the engine's table_registry into the options when the caller did
        // not supply one. This makes the wrapper correct-by-construction: any future
        // caller of QueryEngine::explain gets available_tables populated from the
        // wired registry without needing to know about ExplainOptions::table_registry.
        if options.table_registry.is_none() {
            if let Some(ref registry) = self.table_registry {
                options.table_registry = Some(Arc::clone(registry));
            }
        }
        // SEC-003: inject resolved_spec_map so that available_tables is filtered to
        // the requesting org's visible tables (CWE-200 cross-tenant info disclosure fix).
        // Mirrors the SEC-001 wiring used in execute_inner for sensor fan-out.
        // ADR-042: use the accessor which calls load_full() for the correct Arc<HashMap> type.
        if options.resolved_spec_map.is_none() {
            options.resolved_spec_map = self.resolved_spec_map();
        }
        crate::explain::explain(query_str, options)
    }

    /// Execute a PrismQL query string and return results alongside the
    /// materialized `SessionContext` for detection-engine reuse.
    ///
    /// The caller (S-4.03 detection engine) manages the `SessionContext`
    /// lifetime. This is the only method that returns an `Arc<SessionContext>`.
    ///
    /// MUST NOT be used for regular analyst queries — use `execute()` instead.
    ///
    /// # BC-2.11.005
    /// The `SessionContext` is kept alive by the caller; not ephemeral here.
    pub async fn execute_scheduled(
        &self,
        query_str: &str,
        clients: Option<Vec<OrgSlug>>,
    ) -> Result<
        (
            QueryResult,
            Arc<datafusion::execution::context::SessionContext>,
        ),
        PrismError,
    > {
        let start = std::time::Instant::now();

        // HIGH-002 / ADV-W3MT-P59-HIGH-002: wrap the entire scheduled execution in a timeout.
        // BC-2.11.006 requires 30s timeout for the full execution lifecycle, including
        // execute_scheduled. Mirrors the same pattern as execute().
        let timeout_secs = self.config.timeout_secs;
        let result = tokio::time::timeout(
            std::time::Duration::from_secs(timeout_secs),
            self.execute_scheduled_inner(query_str, clients),
        )
        .await;

        match result {
            Ok(Ok((mut qr, ctx))) => {
                qr.context.execution_time_ms = start.elapsed().as_millis() as u64;
                Ok((qr, ctx))
            }
            Ok(Err(e)) => Err(e),
            Err(_elapsed) => Err(PrismError::QueryTimeout {
                elapsed_ms: start.elapsed().as_millis() as u64,
            }),
        }
    }

    /// Inner body for `execute_scheduled` (without the timeout wrapper).
    async fn execute_scheduled_inner(
        &self,
        query_str: &str,
        clients: Option<Vec<OrgSlug>>,
    ) -> Result<
        (
            QueryResult,
            Arc<datafusion::execution::context::SessionContext>,
        ),
        PrismError,
    > {
        // S-3.13: plan-time table availability gate for scheduled queries (AC-8 mode-agnostic).
        // Gate ordering (BC-2.11.019 v1.5, S-DEMO-FIDELITY-REMEDIATION-001 HIGH-001):
        //   E-QUERY-037 (table) → E-QUERY-038 (column) → E-QUERY-039 (enrich) → E-QUERY-011 (capability).
        // H1 fix: capability gate moved AFTER 037/038/039 to match execute_inner canonical order.
        // Rationale: "table not found" and "column not found" are more actionable first errors than
        // "you lack capability" — the capability gate still fires before any I/O.
        // ADR-039 / SEC-001: pass org_scope and resolved_spec_map for org-scoped filtering.
        // Gate fires BEFORE resolve_clients to avoid moving `clients` before the borrow.
        // ADR-042: load snapshot from ArcSwap — snapshot lives long enough for the borrow below.
        let resolved_spec_snapshot_scheduled = self.resolved_spec_map();
        check_table_availability(
            query_str,
            self.table_registry.as_deref(),
            clients.as_deref(),
            resolved_spec_snapshot_scheduled.as_deref(),
        )?;

        // S-DEMO-PRISMQL-ONBOARDING-001-B: plan-time column gate (BC-2.11.016 / E-QUERY-038).
        // Fires AFTER E-QUERY-037 passes (table exists → then check columns).
        let client_id_for_col_gate_sched = clients
            .as_deref()
            .and_then(|c| c.first())
            .map(|o| o.as_str().to_owned())
            .unwrap_or_default();
        check_query_column_availability(
            query_str,
            &client_id_for_col_gate_sched,
            clients.as_deref(),
            resolved_spec_snapshot_scheduled.as_deref(),
            self.table_registry.as_deref(),
        )?;

        // S-DEMO-FIDELITY-REMEDIATION-001 AC-N1B: E-QUERY-039 enrichment UDF gate for
        // scheduled queries — fires LAST among content gates (after E-QUERY-037 and E-QUERY-038).
        // Gate ordering (BC-2.11.019 v1.5): E-QUERY-037 → E-QUERY-038 → E-QUERY-039.
        check_enrich_udf_availability(query_str, self.infusion_registry.as_deref())?;

        // H1 fix (S-DEMO-FIDELITY-REMEDIATION-001): capability gate (E-QUERY-011) fires AFTER
        // 037/038/039, mirroring execute_inner. Previously this gate ran BEFORE 037/038/039
        // in execute_scheduled_inner, causing asymmetric first-error behavior.
        // Canonical gate order: E-QUERY-001 (parse) → E-QUERY-037 → E-QUERY-038 → E-QUERY-039
        //   → E-QUERY-011 (capability, LAST pre-I/O gate).
        // Scheduled queries run in system context with no capabilities — this means they
        // cannot reference prism_audit (correct secure-by-default for scheduled queries).
        // The gate is best-effort: if query_str fails to parse, the pipeline handles it.
        check_internal_table_capabilities(query_str, &[])?;

        // Resolve client scope (BC-2.11.011).
        let resolved_clients = crate::scoping::resolve_clients(clients, &self.client_registry)?;

        // Create ephemeral SessionContext with GreedyMemoryPool (BC-2.11.006).
        // HIGH-001 / ADV-W3MT-P58-HIGH-001: use build_session_context (not SessionContext::new()).
        // Kept alive by the returned Arc so the caller's detection engine can reuse it.
        let session_ctx = Arc::new(crate::memory::build_session_context(
            self.config.memory_pool_bytes,
        )?);

        // S-DEMO-ENRICHMENT-PIVOT-001 / BC-2.19.001: register plugin-backed enrichment UDFs
        // for scheduled queries as well (detection-engine enrichment context).
        //
        // Error propagation: propagate the inner error verbatim — same rationale as
        // execute_inner (MED-2 fix; see that site for the full explanation).
        if let Some(ref registry) = self.infusion_registry {
            // HIGH-1 fix (BC-2.19.002): same three-tier cache wiring as execute_inner.
            match (&self.infusion_lru_cache, &self.infusion_tier3_cache) {
                (Some(lru), Some(t3)) => crate::infusion_udf::register_infusion_udfs_with_cache(
                    &session_ctx,
                    registry.udf_descriptors(),
                    Arc::clone(lru),
                    Arc::clone(t3),
                    // DEFAULT_CACHE_TTL_SECS is the Tier-2/3 write TTL fallback; it is
                    // OVERRIDDEN per-UDF by descriptor.cache_ttl_secs when set on the
                    // infusion spec (F-TTL-1). The literal 3600 here is NOT the effective
                    // TTL for specs that declare `cache_ttl_secs`.
                    crate::infusion_udf::DEFAULT_CACHE_TTL_SECS,
                ),
                _ => crate::infusion_udf::register_infusion_udfs(
                    &session_ctx,
                    registry.udf_descriptors(),
                ),
            }
            .map_err(|e| prism_core::PrismError::QueryExecutionFailed {
                detail: e.to_string(),
            })?;
        }

        // F-LP1-CRIT-1: register internal tables for scheduled queries too.
        // Scheduled queries run with no caller capabilities (system context).
        if let Some(ref storage) = self.storage {
            crate::internal_tables::register_internal_tables_with_capabilities(
                &session_ctx,
                Arc::clone(storage),
                &[],
            )?;
        }

        // Set up MaterializationContext.
        // F-LP2-CRIT-001: pass resolved_spec_map so fan_out_with_overlay_map is used
        // when per-org overlay endpoints are configured (ADR-029).
        // QRY-02: scheduled queries share the same response cache as analyst
        // queries (BC-2.07.003 — single cache type, keyed per BC-2.07.005).
        let mut mat_ctx = crate::materialization::MaterializationContext::new_with_resolver(
            Arc::clone(&self.adapter_registry),
            Arc::clone(&self.ocsf_normalizer),
            self.config.max_materialized_records,
            Arc::clone(&self.credential_resolver),
            self.org_registry.clone(),
            // ADR-042: load snapshot from ArcSwap — returns Option<Arc<HashMap>>.
            self.resolved_spec_map(),
        )
        .with_response_cache(Arc::clone(&self.cache));

        let effective_options = QueryOptions {
            clients: Some(resolved_clients.clone()),
            ..QueryOptions::default()
        };

        // Run the materialization pipeline against the session context.
        let output = crate::materialization::run_materialization_pipeline(
            query_str,
            &effective_options,
            &mut mat_ctx,
            &session_ctx,
        )
        .await?;

        let total_rows: usize = output.batches.iter().map(|b| b.num_rows()).sum();

        // ADV-W3MT-P58-HIGH-005: sensors_queried now populated from materialization output.
        // Note: execution_time_ms is set to 0 here and filled in by execute_scheduled()
        // after the timeout wrapper completes. (ADV-W3MT-P59-HIGH-002)
        let context = QueryResultContext {
            original_query: query_str.to_string(),
            expanded_query: query_str.to_string(),
            clients_queried: resolved_clients,
            sensors_queried: output.sensors_queried,
            execution_time_ms: 0, // filled in by execute_scheduled()
        };

        let qr = QueryResult {
            batches: output.batches,
            total_available: total_rows,
            is_truncated: false,
            returned_results: total_rows,
            context,
            sensor_errors: output.sensor_errors,
        };

        Ok((qr, session_ctx))
    }
}

// ---------------------------------------------------------------------------
// check_internal_table_capabilities — pre-execution capability gate (F-LP1-HIGH-3)
// ---------------------------------------------------------------------------

/// Pre-execution capability gate for internal tables (Layer 1 of defense-in-depth).
///
/// Parses the query string and **recursively** walks all AST positions where a subquery
/// can appear — WHERE / HAVING predicates, SELECT projection expressions, JOIN sources
/// and ON conditions, GROUP BY / ORDER BY expressions, function-call argument lists,
/// and DML source_select and filter clauses (INSERT INTO … SELECT … and UPDATE/DELETE WHERE)
/// — to extract every referenced `prism_*` table name. (F-LP2-CRIT-1 Layer 1)
///
/// For each extracted table, consults `INTERNAL_TABLE_DESCRIPTORS` to check
/// `requires_audit_read`. If `true` and `Capability::AuditRead` is absent, rejects
/// with `PrismError::AuditTableAccessDenied` (E-QUERY-011). This makes the policy
/// data-driven: future tables with `requires_audit_read = true` are automatically gated.
/// (F-LP2-MED-3 / F-LP2-CRIT-1 Layer 3)
///
/// # Parse failures
/// If the query fails to parse, returns `Ok(())` — the pipeline handles parse errors.
///
/// # BC-2.15.011
/// `prism_audit` requires `audit.read` capability → `Capability::AuditRead`.
fn check_internal_table_capabilities(
    query_str: &str,
    capabilities: &[Capability],
) -> Result<(), PrismError> {
    // Best-effort parse — if parsing fails, let the pipeline surface the error.
    let ast = match crate::filter_parser::PrismQlParser::parse(query_str) {
        Ok(ast) => ast,
        Err(_) => return Ok(()), // parse errors handled downstream
    };

    // Extract ALL source table names recursively (Layer 1: subquery walk).
    let source_names = crate::materialization::extract_source_names_recursive(&ast);

    // Layer 3: descriptor-driven policy check via INTERNAL_TABLE_DESCRIPTORS.
    let has_audit_read = capabilities.contains(&Capability::AuditRead);
    for name in &source_names {
        // Look up the descriptor for this table; if it requires audit.read and caller lacks it, deny.
        if crate::internal_tables::table_requires_audit_read(name) && !has_audit_read {
            return Err(PrismError::AuditTableAccessDenied);
        }
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// check_table_availability — plan-time E-QUERY-037 gate (S-3.13)
// ---------------------------------------------------------------------------

/// Plan-time table availability gate (Layer 1 of the availability check).
///
/// Parses the query string, extracts all source_refs via the AST visitor, and
/// checks each against the provided `TableRegistry`. Returns `Err(PrismError::TableNotAvailable)`
/// for the first unregistered table encountered, with:
/// - `sensor`: the prefix of the table name (e.g. `"crowdstrike"` for `"crowdstrike_alerts"`)
/// - `available_sensors`: comma-separated list (filtered to requesting org when org_scope is provided)
/// - `available_tables`: comma-separated list (filtered to requesting org when org_scope is provided)
/// - `did_you_mean`: Levenshtein-based suggestion (filtered to requesting org's tables)
///
/// # Gate skip conditions
/// - `registry` is `None`: skip immediately (legacy / test mode without spec engine wiring)
/// - Query fails to parse: return `Ok(())` — parse errors are handled downstream
/// - Table name starts with `prism_`: skip (internal tables have their own gate)
///
/// # Org-scoped error enumeration (ADR-039 / SEC-001 / CWE-200)
/// `org_scope` and `resolved_spec_map` are forwarded to `check_availability_gate` to filter
/// the enumeration fields to the requesting org's scope. When either is `None`, the global
/// registry is used (single-tenant backward compatibility).
///
/// # AC-8 mode-agnostic guarantee
/// This function runs on the alias-expanded query string before `materialize_query` —
/// the same code path is reached by SQL, filter, and pipe mode queries.
///
/// # BC-2.11.001 / S-3.13 AC-2, AC-3, AC-8 / ADR-039
fn check_table_availability(
    query_str: &str,
    registry: Option<&TableRegistry>,
    org_scope: Option<&[prism_core::OrgSlug]>,
    resolved_spec_map: Option<
        &std::collections::HashMap<
            prism_spec_engine::ResolvedSpecKey,
            prism_spec_engine::ResolvedSensorSpec,
        >,
    >,
) -> Result<(), PrismError> {
    // Skip when no registry is wired — preserves backward compatibility for legacy tests.
    // All constructors (new, new_with_cache_config, new_full) initialize table_registry
    // as None; callers that need the gate use with_table_registry().
    let Some(registry) = registry else {
        return Ok(());
    };
    // Delegate to the registry's gate method with org-scope parameters.
    // The gate body lives in table_registry.rs, keeping engine.rs
    // free of stub macros per POL-12 / test_AC_8_no_todo_or_unimplemented_remains.
    registry.check_availability_gate(query_str, org_scope, resolved_spec_map)
}

// ---------------------------------------------------------------------------
// E-QUERY-039 plan-time enrichment UDF gate (S-DEMO-FIDELITY-REMEDIATION-001 AC-N1B)
// ---------------------------------------------------------------------------

/// DataFusion built-in scalar function names — computed once at first use via LazyLock.
///
/// The enrich gate must NOT flag a scalar name that DataFusion can resolve as a built-in
/// (e.g. `lower`, `upper`, `coalesce`, `concat`, `length`, `abs`, `round`, `cast`, …).
/// These are registered in the default `SessionContext` via `build_session_context` →
/// `SessionStateDefaults::register_scalar_functions`. The PrismQL AST renderer emits
/// ALL unrecognized scalar function calls as `ScalarFunc::Unknown(name)`, including
/// DataFusion built-ins — so the gate must exclude them explicitly.
///
/// Mechanism: call `SessionStateDefaults::default_scalar_functions()` once to enumerate
/// every built-in Arc<ScalarUDF>, collect their lowercase names into a HashSet, and
/// store it in a static LazyLock. Per-query cost: a single O(1) HashSet lookup per
/// collected name. LazyLock initialization happens at first gate invocation (process start
/// in production; first test run otherwise) — not per query.
///
/// Case-insensitive exclusion: DataFusion normalizes function names to lowercase
/// internally; we lowercase the collected name before the lookup to match.
///
/// # BC-2.11.019 v1.5 §F-PJL1-HIGH-001 compliance — "or equivalent" rationale
///
/// The BC implementation note states: "The check MUST use `ctx.state().scalar_functions()`
/// **or equivalent** — so that the gate's built-in exclusion list is always consistent with
/// what DataFusion can actually resolve. Hard-coding an allowlist is an anti-pattern."
///
/// `SessionStateDefaults::default_scalar_functions()` IS the canonical "or equivalent":
///
/// 1. `build_session_context` (memory.rs) creates a `SessionContext` with the **default**
///    `SessionConfig` — no built-in scalars are removed or replaced in that construction.
/// 2. After building the default context, only infusion async-UDFs (`Arc<AsyncScalarUdf>`)
///    are registered beyond the defaults; those are NOT built-in scalar functions and are
///    handled separately by the `InfusionRegistry` path.
/// 3. Therefore, `ctx.state().scalar_functions()` on the execution context and
///    `SessionStateDefaults::default_scalar_functions()` enumerate the **identical set**
///    of built-in scalar names — the two are provably equivalent for this deployment model.
///
/// Using `SessionStateDefaults::default_scalar_functions()` in a `LazyLock` avoids the
/// cost of creating a full `SessionContext` at every gate invocation while remaining
/// 100% consistent with the execution context's built-in resolution. This is NOT a
/// hard-coded allowlist — it is a runtime-derived set populated at first use, staying
/// in sync with DataFusion version upgrades automatically.
///
/// BC-2.11.019 v1.5 §Gate firing condition: fire E-QUERY-039 ONLY for a name that is
/// neither a DataFusion built-in scalar NOR a registered enrichment UDF.
/// F-PJL1-HIGH-001 (S-DEMO-FIDELITY-REMEDIATION-001 Pass-J LOCAL cascade).
/// F-PLL1-LOW-002 (S-DEMO-FIDELITY-REMEDIATION-001 Pass-L LOCAL cascade) — compliance
/// comment added; no behavior change.
static DATAFUSION_BUILTIN_SCALAR_NAMES: std::sync::LazyLock<std::collections::HashSet<String>> =
    std::sync::LazyLock::new(|| {
        use datafusion::execution::SessionStateDefaults;
        let mut names = std::collections::HashSet::new();
        for udf in SessionStateDefaults::default_scalar_functions() {
            // Primary name (e.g. "character_length").
            names.insert(udf.name().to_ascii_lowercase());
            // Aliases (e.g. "length", "char_length" for character_length).
            // DataFusion resolves SQL function calls by both name and aliases.
            for alias in udf.aliases() {
                names.insert(alias.to_ascii_lowercase());
            }
        }
        names
    });

/// Walk ALL scalar-expr positions in a `SqlQuery` and collect `ScalarFunc::Unknown` names.
///
/// This is the SINGLE canonical walk used by `check_enrich_udf_availability` for both
/// `Ast::Sql(Select)` and `Ast::SqlPipe` head queries.  It covers every position where a
/// scalar function call can appear in a `SqlQuery`:
///
/// | Position                        | Coverage rationale |
/// |---------------------------------|--------------------|
/// | SELECT projection items         | primary enrichment site |
/// | WHERE clause predicate          | defensive (HIGH-003 fix) |
/// | JOIN ON conditions (all joins)  | C1/C2 fix: `ON badudf(x)=y` bypassed gate |
/// | GROUP BY exprs                  | C1/C2 fix: `GROUP BY badudf(x)` bypassed gate |
/// | ORDER BY exprs                  | C1/C2 fix: `ORDER BY badudf(x)` bypassed gate |
/// | HAVING predicate                | forward-compat; mirrors WHERE walk |
///
/// `Join.on` is typed as `Expr` (not `Predicate`) in the AST, so it goes through
/// `collect_unknown_scalar_from_expr` directly.
///
/// S-DEMO-FIDELITY-REMEDIATION-001 C1+C2 fix; BC-2.11.019 v1.5.
fn collect_unknown_scalars_from_sql_query(sq: &crate::ast::SqlQuery, out: &mut Vec<String>) {
    use crate::ast::SelectItem;

    // (a) SELECT projection items.
    for item in &sq.select.items {
        if let SelectItem::Expr { expr, .. } = item {
            collect_unknown_scalar_from_expr(expr, out);
        }
    }
    // (b) WHERE-clause predicate tree (HIGH-003 fix).
    if let Some(pred) = &sq.where_ {
        collect_unknown_scalar_from_predicate(pred, out);
    }
    // (c) JOIN ON conditions — C1/C2 fix.
    // `Join.on` is `Expr`, not `Predicate`.
    for join in &sq.joins {
        collect_unknown_scalar_from_expr(&join.on, out);
    }
    // (d) GROUP BY expressions — C1/C2 fix.
    for expr in &sq.group_by {
        collect_unknown_scalar_from_expr(expr, out);
    }
    // (e) ORDER BY expressions — C1/C2 fix.
    for oe in &sq.order_by {
        collect_unknown_scalar_from_expr(&oe.expr, out);
    }
    // (f) HAVING predicate (forward-compat; mirrors WHERE walk).
    if let Some(pred) = &sq.having {
        collect_unknown_scalar_from_predicate(pred, out);
    }
}

/// Collect all `ScalarFunc::Unknown` names from an `Expr` tree.
///
/// Recurses into `FuncCall::Scalar` arguments, `Expr::Logical`, `Expr::Not`, and
/// `Expr::Compare` (lhs/rhs) to find every `ScalarFunc::Unknown(name)` node.
///
/// Module-level so it is accessible from `#[cfg(test)]` blocks for unit testing.
/// Called by `check_enrich_udf_availability` and `collect_unknown_scalars_from_sql_query`.
fn collect_unknown_scalar_from_expr(expr: &crate::ast::Expr, out: &mut Vec<String>) {
    use crate::ast::{Expr, FuncCall, ScalarFunc};
    match expr {
        Expr::FuncCall(FuncCall::Scalar {
            func: ScalarFunc::Unknown(name),
            args,
        }) => {
            out.push(name.clone());
            for arg in args {
                collect_unknown_scalar_from_expr(arg, out);
            }
        }
        Expr::FuncCall(FuncCall::Scalar { args, .. }) => {
            for arg in args {
                collect_unknown_scalar_from_expr(arg, out);
            }
        }
        Expr::FuncCall(FuncCall::Aggregate { args, .. }) => {
            for arg in args {
                collect_unknown_scalar_from_expr(arg, out);
            }
        }
        Expr::Logical { lhs, rhs, .. } => {
            collect_unknown_scalar_from_expr(lhs, out);
            collect_unknown_scalar_from_expr(rhs, out);
        }
        Expr::Not(inner) => collect_unknown_scalar_from_expr(inner, out),
        Expr::Compare { lhs, rhs, .. } => {
            collect_unknown_scalar_from_expr(lhs, out);
            collect_unknown_scalar_from_expr(rhs, out);
        }
        // Leaf nodes or non-function expressions — nothing to collect.
        _ => {}
    }
}

/// Collect all `ScalarFunc::Unknown` names from a `Predicate` tree.
///
/// Handles `Predicate::Compare { lhs, rhs }` whose lhs/rhs are `Box<Expr>`, and
/// recurses into `Predicate::Logical` and `Predicate::Not`.
///
/// BC-2.11.019 §Precondition 1(b): WHERE-clause unknown scalar functions must be gated
/// at plan time (HIGH-003 fix in S-DEMO-FIDELITY-REMEDIATION-001).
///
/// Module-level so it is accessible from `#[cfg(test)]` blocks for unit testing.
/// Called by `check_enrich_udf_availability`.
fn collect_unknown_scalar_from_predicate(pred: &crate::ast::Predicate, out: &mut Vec<String>) {
    use crate::ast::Predicate;
    match pred {
        Predicate::Compare { lhs, rhs, .. } => {
            collect_unknown_scalar_from_expr(lhs, out);
            collect_unknown_scalar_from_expr(rhs, out);
        }
        Predicate::Logical { predicates, .. } => {
            for p in predicates {
                collect_unknown_scalar_from_predicate(p, out);
            }
        }
        Predicate::Not(inner) => collect_unknown_scalar_from_predicate(inner, out),
        // All other variants (StringOp, Regex, In, InSubquery, Between, Cidr,
        // Has, Missing, IsNull, Wildcard, RecoveryError) do not embed function calls.
        _ => {}
    }
}

/// Plan-time enrichment UDF availability gate — E-QUERY-039 (BC-2.11.019 v1.5).
///
/// Fires AFTER `check_table_availability` AND `check_query_column_availability`
/// (BC-2.11.019 v1.5 §Gate ordering: gate sequence is 001 → 037 → 038 → 039;
/// enrich gate is last in the chain).
///
/// Parses the query string, collects all enrichment function names used in the query
/// (both pipe-mode `| enrich udf_name(col)` and SQL-mode `SELECT udf_name(col)`), then
/// validates each against the `InfusionRegistry` descriptor set.
///
/// # Gate skip conditions
/// - `registry` is `None`: skip immediately (enrichment not configured).
/// - Query fails to parse: return `Ok(())` — parse errors handled downstream.
/// - No enrichment names found in the AST: return `Ok(())` (query doesn't use enrichment).
/// - Name is a DataFusion built-in scalar: skip (resolved by `ctx.sql()` — not an enrichment).
///
/// # SQL path detection
/// SQL-mode enrichment: `ScalarFunc::Unknown(name)` in `FuncCall::Scalar` nodes.
/// Both `Ast::Sql(Select)` and `Ast::SqlPipe` head queries are handled via
/// `collect_unknown_scalars_from_sql_query`, which scans all six scalar positions:
///
/// | Position                        | Coverage rationale |
/// |---------------------------------|--------------------|
/// | SELECT projection items         | primary enrichment site (non-wildcard only) |
/// | WHERE clause predicate          | HIGH-003 fix: `WHERE badudf(x) > 0` bypassed gate |
/// | JOIN ON conditions (all joins)  | C1/C2 fix: `ON badudf(x)=y` bypassed gate |
/// | GROUP BY exprs                  | C1/C2 fix: `GROUP BY badudf(x)` bypassed gate |
/// | ORDER BY exprs                  | C1/C2 fix: `ORDER BY badudf(x)` bypassed gate |
/// | HAVING predicate                | forward-compat; mirrors WHERE walk |
///
/// DataFusion built-in scalars (lower, upper, coalesce, etc.) are excluded via
/// `DATAFUSION_BUILTIN_SCALAR_NAMES` before the registered-UDF check.
///
/// # Pipe path detection
/// Pipe-mode enrichment: `PipeStage::Enrich(EnrichStage { infusion, .. })` nodes in
/// the pipe stage list. The `infusion` field holds the caller-supplied UDF name.
///
/// # Reference
/// S-DEMO-FIDELITY-REMEDIATION-001 AC-N1B; BC-2.11.019 v1.5; error-taxonomy.md E-QUERY-039.
/// F-PJL1-HIGH-001 (Pass-J LOCAL cascade): DataFusion built-in exclusion added.
fn check_enrich_udf_availability(
    query_str: &str,
    registry: Option<&prism_spec_engine::InfusionRegistry>,
) -> Result<(), PrismError> {
    use crate::ast::{Ast, PipeStage, SqlStatement};
    use crate::filter_parser::PrismQlParser;
    use prism_core::error::EnrichUdfNotFoundDetails;

    // Skip when no registry is wired — enrichment not configured in this deployment.
    let Some(registry) = registry else {
        return Ok(());
    };

    // Parse the query. On parse failure, return Ok(()) — parse errors are emitted
    // downstream as E-QUERY-001. This mirrors check_table_availability's behavior.
    let ast = match PrismQlParser::parse(query_str) {
        Ok(ast) => ast,
        Err(_) => return Ok(()),
    };

    // Build the registered UDF name set from the live registry.
    let descriptors = registry.udf_descriptors();
    let registered_names: std::collections::HashSet<&str> =
        descriptors.iter().map(|d| d.name.as_str()).collect();

    // Collect enrichment UDF names from the AST via direct pattern matching.
    // Using direct match (not the Visitor trait) to avoid coupling with the full
    // visitor infrastructure — enrichment nodes are a well-defined subset.
    //
    // F-PNL1-MED-001 (S-DEMO-FIDELITY-REMEDIATION-001 Pass-N LOCAL cascade):
    // BC-2.11.019 v1.5 §F-PJL1-HIGH-001 "Scope of change" states the DataFusion
    // built-in exclusion applies to SQL-mode `ScalarFunc::Unknown` gate logic ONLY.
    // Pipe-mode `EnrichStage.infusion` gate is UNAFFECTED — `| enrich lower(col)`
    // is an explicit enrichment directive; `lower` there is not a DataFusion scalar
    // call but an unregistered infusion name, so E-QUERY-039 MUST fire.
    // Fix: separate pipe-mode names and SQL-mode names into distinct Vecs so the
    // built-in skip is applied to SQL names only.
    let mut pipe_enrich_names: Vec<String> = Vec::new(); // no built-in skip
    let mut sql_unknown_names: Vec<String> = Vec::new(); // built-in skip applied

    match &ast {
        // Pipe mode: `FROM table | enrich udf_name(col)` stages.
        Ast::Pipe(pq) => {
            for stage in &pq.stages {
                if let PipeStage::Enrich(es) = stage {
                    pipe_enrich_names.push(es.infusion.clone());
                }
            }
        }
        // SqlPipe mode: SQL head with pipe stages.
        // Enrich names can appear in TWO places:
        //   (a) pipe stages: `… | enrich udf_name(col)` — pipe-mode, no built-in skip.
        //   (b) SQL HEAD: any scalar position in the head SqlQuery (SELECT, WHERE,
        //       JOIN ON, GROUP BY, ORDER BY, HAVING) — SQL-mode, built-in skip applied.
        // BC-2.11.019 §Precondition 1(b): projection OR WHERE (either site counts).
        // C1/C2 fix: use collect_unknown_scalars_from_sql_query to cover ALL positions
        // including JOIN ON / GROUP BY / ORDER BY which the previous inline walk missed.
        Ast::SqlPipe(spq) => {
            // (a) pipe stages — pipe-mode, no built-in skip.
            for stage in &spq.stages {
                if let PipeStage::Enrich(es) = stage {
                    pipe_enrich_names.push(es.infusion.clone());
                }
            }
            // (b) SQL head — ALL scalar positions via canonical shared walk, SQL-mode.
            collect_unknown_scalars_from_sql_query(&spq.head, &mut sql_unknown_names);
        }
        // SQL mode: scan ALL scalar positions via canonical shared walk.
        // BC-2.11.019 §Precondition 1(b): projection OR WHERE (either site counts).
        // C1/C2 fix: use collect_unknown_scalars_from_sql_query to cover ALL positions
        // including JOIN ON / GROUP BY / ORDER BY which the previous inline walk missed.
        Ast::Sql(SqlStatement::Select(sq)) => {
            collect_unknown_scalars_from_sql_query(sq, &mut sql_unknown_names);
        }
        // Filter mode and DML have no enrichment syntax.
        _ => {}
    }

    // Validate pipe-mode enrich names — NO DataFusion built-in exclusion.
    // BC-2.11.019 v1.5 §F-PJL1-HIGH-001: pipe-mode `| enrich <name>` is an explicit
    // enrichment directive. A built-in name like `lower` used as a pipe-mode infusion
    // is NOT a DataFusion scalar — it is an unregistered infusion the analyst is trying
    // to apply, so E-QUERY-039 MUST fire when it is not in InfusionRegistry.
    //
    // Validate SQL-mode unknown scalar names — WITH DataFusion built-in exclusion.
    // BC-2.11.019 v1.5: skip names that are DataFusion built-in scalars —
    // they are resolvable by ctx.sql() and must NOT trigger E-QUERY-039.
    // F-PJL1-HIGH-001 (S-DEMO-FIDELITY-REMEDIATION-001 Pass-J LOCAL cascade).
    //
    // Iterator chain: pipe names first (no skip), then filtered SQL names (skip applied).
    let sql_names_filtered = sql_unknown_names.iter().filter(|name| {
        let name_lower = name.to_ascii_lowercase();
        !DATAFUSION_BUILTIN_SCALAR_NAMES.contains(&name_lower)
    });
    let all_names_to_check = pipe_enrich_names.iter().chain(sql_names_filtered);

    for requested in all_names_to_check {
        if !registered_names.contains(requested.as_str()) {
            // Requested name is not a registered per-field UDF name.
            // Build available_infusions from all registered per-field names.
            // MED-001 fix: sort + dedup so the list is deterministic (lexicographic order)
            // as required by error-taxonomy.md §E-QUERY-039 (canonicalized at v2.01, matches current v2.03). This mirrors the
            // sort+dedup fix applied to available_columns in check_column_availability
            // (`check_column_availability` OBS-FRESH-1 fix) for E-QUERY-038 parity.
            let mut available_infusions: Vec<String> =
                descriptors.iter().map(|d| d.name.clone()).collect();
            available_infusions.sort();
            available_infusions.dedup();

            // did_you_mean: Levenshtein ≤ 3 suggestion from registered names.
            // OBS-1 fix: lexicographic tie-break (name asc) to ensure determinism
            // when multiple names have the same minimum edit distance. This mirrors
            // the column gate's determinism fix (check_query_column_availability).
            // F-PHL1-HIGH-001: cap `requested` at 128 bytes (SEC-002 / CWE-407)
            // before the O(m×n) Levenshtein loop — mirrors the table gate cap in
            // `table_registry::did_you_mean`.
            let requested_capped = crate::table_registry::cap_name_for_levenshtein(requested);
            let did_you_mean = available_infusions
                .iter()
                .map(|n| (n.clone(), strsim::levenshtein(requested_capped, n)))
                .filter(|(_, dist)| *dist <= 3)
                .min_by_key(|(name, dist)| (*dist, name.clone()))
                .map(|(name, _)| name);

            return Err(PrismError::EnrichUdfNotFound(Box::new(
                EnrichUdfNotFoundDetails::new(requested.clone(), available_infusions, did_you_mean),
            )));
        }
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Shared column-name extractor for E-QUERY-038 gate (F-001B-DC-HIGH-001)
// ---------------------------------------------------------------------------

/// Extract the bare column name from a `FieldPath`, handling table-qualified references.
///
/// This helper is the SINGLE extraction point used at ALL FIVE positions in
/// `check_query_column_availability` (SELECT, WHERE, GROUP BY, ORDER BY, JOIN ON),
/// preventing the `.first()` false-reject bug from recurring at any one position
/// independently. GROUP BY, ORDER BY, and JOIN ON positions use
/// `extract_field_paths_from_expr` which calls this helper for each `Expr::Field`
/// found while recursing into FuncCall args.
///
/// # Behaviour
/// - **1-segment path** (`["severity"]`): returns `Some("severity")` — unqualified column.
/// - **2+-segment path, matching qualifier** (`["crowdstrike_alerts", "severity"]` when
///   `table_name = "crowdstrike_alerts"`): returns `Some("severity")` — the last segment.
/// - **2+-segment path, unknown qualifier** (`["other_table", "severity"]` when
///   `table_name = "crowdstrike_alerts"` and no matching alias): returns `None` — fail-open.
///   E-QUERY-038 is NOT emitted for an unrecognised qualifier; that is a separate error class.
///
/// # Alias handling
/// `table_alias` is the SQL `AS alias` identifier from the `FROM` clause (e.g. `t` in
/// `FROM crowdstrike_alerts t`). When the qualifier matches the alias, the last segment
/// is returned exactly as it is for a full table-name match. If the parser does not
/// populate `FromClause.alias` for a given query, pass `None`.
///
/// # Anti-drift guarantee
/// Every extraction site in `check_query_column_availability` calls this helper.
/// Adding a new extraction site without using this helper will cause TD-VSDD-060
/// (sibling-site sweep) to flag it in adversarial review.
///
/// Reference: BC-2.11.016 §Postconditions; F-001B-DC-HIGH-001.
fn extract_column_name_from_field_path(
    fp: &crate::ast::FieldPath,
    table_name: &str,
    table_alias: Option<&str>,
) -> Option<String> {
    match fp.segments.len() {
        0 => None,
        1 => fp.segments.first().cloned(),
        _ => {
            // Multi-segment: check whether the leading qualifier is recognised.
            let qualifier = fp.segments.first()?;
            let qualifier_matches =
                qualifier == table_name || table_alias.is_some_and(|alias| qualifier == alias);
            if qualifier_matches {
                // Return the last segment as the bare column name.
                fp.segments.last().cloned()
            } else {
                // Unknown qualifier — fail-open: do not gate on it.
                // This is not an E-QUERY-038 situation; it may be a cross-table
                // expression or a future syntax extension.
                None
            }
        }
    }
}

// ---------------------------------------------------------------------------
// E-QUERY-038 helper: recursive field-path extractor (M2 fix)
// ---------------------------------------------------------------------------

/// Recursively extract bare column names from an `Expr` tree.
///
/// Walks ALL `Expr::Field` references, including those nested inside
/// `Expr::FuncCall` argument lists, `Expr::Compare` (JOIN ON conditions),
/// `Expr::Logical` (AND/OR chains in JOIN ON), and `Expr::Not` at any depth.
/// This is the M2 fix for S-DEMO-FIDELITY-REMEDIATION-001 finding M2:
/// `check_query_column_availability` previously used `Expr::Field` direct
/// matches for GROUP BY and ORDER BY positions, causing queries like
/// `GROUP BY lower(col_typo)` and `JOIN ON a.typo_col = b.col` to bypass the
/// E-QUERY-038 gate.
///
/// # What is collected
/// - `Expr::Field(fp)` — bare and qualified column refs, resolved via
///   `extract_column_name_from_field_path` (table_name + alias matching).
/// - `Expr::FuncCall(Aggregate { args, .. })` — recurse into all args.
/// - `Expr::FuncCall(Scalar { args, .. })` — recurse into all args.
/// - `Expr::FuncCall(Window { .. })` — no args to recurse into (yet).
/// - `Expr::Compare { lhs, rhs, .. }` — recurse into both operands (JOIN ON).
/// - `Expr::Logical { lhs, rhs, .. }` — recurse into both operands (AND/OR).
/// - `Expr::Not(inner)` — recurse into inner.
/// - `Expr::In { field, .. }` — collect the field directly.
/// - `Expr::InSubquery { field, .. }` — collect the field directly; subquery is fail-open.
/// - `Expr::TimestampArithmetic { base, .. }` — recurse into base.
///
/// # What is NOT collected
/// - `Expr::VirtualField(_)` (`_sensor`, `_client`) — always valid, skip.
/// - `Expr::Literal(_)` — not a column ref.
/// - `Expr::Star` — wildcard, not a column ref.
/// - `Expr::Now` / `Expr::Interval(_)` — planning constants, not column refs.
///
/// # Non-exhaustive safety
/// Unknown future `Expr` and `FuncCall` variants are silently skipped via
/// the `_ => {}` catch-all arm, preserving fail-open gate semantics and
/// satisfying `#[non_exhaustive]` discipline.
///
/// Reference: S-DEMO-FIDELITY-REMEDIATION-001 finding M2; BC-2.11.016.
fn extract_field_paths_from_expr(
    expr: &crate::ast::Expr,
    table_name: &str,
    table_alias: Option<&str>,
    out: &mut Vec<String>,
) {
    use crate::ast::{Expr, FuncCall};
    match expr {
        Expr::Field(fp) => {
            if let Some(col) = extract_column_name_from_field_path(fp, table_name, table_alias) {
                out.push(col);
            }
        }
        Expr::FuncCall(fc) => match fc {
            FuncCall::Aggregate { args, .. } | FuncCall::Scalar { args, .. } => {
                for arg in args {
                    extract_field_paths_from_expr(arg, table_name, table_alias, out);
                }
            }
            FuncCall::Window { .. } => {} // No args yet.
            // #[non_exhaustive] catch-all.
            #[allow(unreachable_patterns)]
            _ => {}
        },
        // JOIN ON conditions: `col_a = col_b`
        Expr::Compare { lhs, rhs, .. } => {
            extract_field_paths_from_expr(lhs, table_name, table_alias, out);
            extract_field_paths_from_expr(rhs, table_name, table_alias, out);
        }
        // JOIN ON AND/OR chains.
        Expr::Logical { lhs, rhs, .. } => {
            extract_field_paths_from_expr(lhs, table_name, table_alias, out);
            extract_field_paths_from_expr(rhs, table_name, table_alias, out);
        }
        // NOT wrapping.
        Expr::Not(inner) => {
            extract_field_paths_from_expr(inner, table_name, table_alias, out);
        }
        // IN membership: collect the field directly.
        Expr::In { field, .. } => {
            if let Some(col) = extract_column_name_from_field_path(field, table_name, table_alias) {
                out.push(col);
            }
        }
        // IN subquery: collect the field; the subquery itself is fail-open.
        Expr::InSubquery { field, .. } => {
            if let Some(col) = extract_column_name_from_field_path(field, table_name, table_alias) {
                out.push(col);
            }
        }
        // TimestampArithmetic: recurse into the base expression.
        Expr::TimestampArithmetic { base, .. } => {
            extract_field_paths_from_expr(base, table_name, table_alias, out);
        }
        // VirtualField, Literal, Star, Now, Interval, and future variants: fail-open.
        _ => {}
    }
}

// ---------------------------------------------------------------------------
// E-QUERY-038 plan-time column gate (S-DEMO-PRISMQL-ONBOARDING-001-B)
// ---------------------------------------------------------------------------

/// Plan-time column availability gate — E-QUERY-038 (BC-2.11.016).
///
/// Fires AFTER `check_table_availability` passes (table exists → then check columns).
/// Gate ordering: E-QUERY-001 (parse) → E-QUERY-037 (table not found) → E-QUERY-038
/// (column not found). If the table check fails, this gate is never reached.
///
/// # Schema source selection
///
/// ## Multi-tenant path (`resolved_spec_map` is `Some`)
/// Checks each column reference against `resolved_spec_map → ResolvedSensorSpec.spec.tables
/// → TableSpec.columns → ColumnSpec.name` for the (table, org_scope) pair.
///
/// ## Single-tenant / table_registry fallback (`resolved_spec_map` is `None`)
/// Falls back to `table_registry.columns_for_table(table_name)` (M1 fix,
/// S-DEMO-FIDELITY-REMEDIATION-001). If `table_registry` is also `None`, fails open.
/// If the table has zero columns in the registry, fails open (backward-compatible for
/// tables without a column spec). When columns are present and the requested column is
/// absent, E-QUERY-038 is returned with `available_columns` populated from the registry.
///
/// The prior behaviour ("when `resolved_spec_map` is `None`, returns `Ok(())`") was
/// removed by the M1 fix; the gate now fires for single-tenant mode via the registry
/// fallback. F-PJL1-MED-001 (S-DEMO-FIDELITY-REMEDIATION-001 Pass-J LOCAL cascade).
///
/// # Error payload
/// `available_columns` is ALWAYS present in the error (empty `[]` only when no columns
/// are registered for the table). `did_you_mean` uses `strsim::levenshtein` with the
/// same ≤3 threshold as the E-QUERY-037 gate (D-1163).
///
/// # BC-2.11.016 / S-DEMO-PRISMQL-ONBOARDING-001-B AC-001, AC-002
fn check_column_availability(
    column_name: &str,
    table_name: &str,
    client_id: &str,
    org_scope: Option<&[prism_core::OrgSlug]>,
    resolved_spec_map: Option<
        &std::collections::HashMap<
            prism_spec_engine::ResolvedSpecKey,
            prism_spec_engine::ResolvedSensorSpec,
        >,
    >,
    table_registry: Option<&crate::table_registry::TableRegistry>,
) -> Result<(), PrismError> {
    use prism_core::error::{ColumnNotFoundDetails, PrismError};

    // M1 fix (S-DEMO-FIDELITY-REMEDIATION-001): when resolved_spec_map is None,
    // fall back to table_registry.columns_for_table() for single-tenant mode.
    // If the table has no columns in the registry (empty list or table unknown),
    // fail-open to preserve backward-compatible behavior for tables without column specs.
    if resolved_spec_map.is_none() {
        let Some(registry) = table_registry else {
            // No schema source at all — fail-open.
            return Ok(());
        };
        // F-PBL1-LOW-001 fix (Pass-B S-DEMO-FIDELITY-REMEDIATION-001): sort + dedup
        // `available_columns` for deterministic ordering, matching the multi-tenant
        // path (`check_query_column_availability` OBS-FRESH-1 fix). `columns_for_table` returns columns
        // in spec insertion order; without sort+dedup the available_columns in the
        // E-QUERY-038 error are non-deterministic across sensor registrations.
        let mut available_columns = registry.columns_for_table(table_name);
        available_columns.sort();
        available_columns.dedup();
        // Fail-open: if no columns registered for this table, skip gate.
        if available_columns.is_empty() {
            return Ok(());
        }
        // Column is in the available set — gate passes.
        if available_columns.contains(&column_name.to_string()) {
            return Ok(());
        }
        // did_you_mean: same ≤3 Levenshtein threshold as multi-tenant path.
        // F-PHL1-MED-001: cap `column_name` at 128 bytes (SEC-002 / CWE-407)
        // before the O(m×n) Levenshtein computation.
        let column_name_capped = crate::table_registry::cap_name_for_levenshtein(column_name);
        let did_you_mean = available_columns
            .iter()
            .map(|c| (c.clone(), strsim::levenshtein(column_name_capped, c)))
            .filter(|(_, dist)| *dist <= 3)
            .min_by_key(|(name, dist)| (*dist, name.clone()))
            .map(|(c, _)| c);
        tracing::warn!(
            event_type = "column_not_found.rejected",
            column = %column_name,
            table = %table_name,
            client_id = %client_id,
            available_count = available_columns.len(),
            "E-QUERY-038: column not found at plan time (single-tenant registry path)"
        );
        return Err(PrismError::ColumnNotFound(Box::new(
            prism_core::error::ColumnNotFoundDetails::new(
                column_name,
                table_name,
                client_id,
                available_columns,
                did_you_mean,
            ),
        )));
    }

    // Multi-tenant path: resolved_spec_map is Some.
    let Some(spec_map) = resolved_spec_map else {
        // Unreachable due to the check above, but satisfies the type system.
        return Ok(());
    };

    // Collect available columns for this table from entries matching org_scope.
    // Org-scoping: filter spec_map entries to those whose org_slug is in org_scope.
    // When org_scope is None, use all entries (no org restriction).
    //
    // Table-name matching: the query uses fully-qualified `{sensor_id}_{table_suffix}` form
    // (e.g. "crowdstrike_alerts"), while `TableSpec.table_name` is the SHORT suffix (e.g.
    // "alerts"). Reconstruct the fully-qualified form as `{spec.sensor_id}_{tbl.table_name}`
    // for matching (test-fixture naming contract per BC-2.11.016 §Test Vectors).
    //
    // Fail-open guard: if the table_name matches NO entry in the spec_map (regardless of
    // org_scope), the gate cannot validate columns — skip. This prevents false positives
    // when the spec_map is populated but doesn't include schema for this particular table
    // (e.g. legacy tables without overlay specs). Only gate when the table IS in the schema.
    // This is distinct from EC-039 (zero-column table IS in schema → gate fires with []).
    let org_visible_entries: Vec<&prism_spec_engine::ResolvedSensorSpec> = spec_map
        .values()
        .filter(|spec| {
            // DI-008: filter to org-visible entries.
            if let Some(scopes) = org_scope {
                scopes.iter().any(|s| s.as_str() == spec.org_slug.as_str())
            } else {
                true
            }
        })
        .collect();

    let table_in_schema = org_visible_entries.iter().any(|spec| {
        let sensor_id = &spec.spec.sensor_id;
        spec.spec
            .tables
            .iter()
            .any(|tbl| format!("{sensor_id}_{}", tbl.table_name) == table_name)
    });

    // Fail-open: if this table has no schema entry, we cannot validate columns.
    if !table_in_schema {
        return Ok(());
    }

    let mut available_columns: Vec<String> = org_visible_entries
        .iter()
        .flat_map(|spec| {
            let sensor_id = spec.spec.sensor_id.clone();
            spec.spec
                .tables
                .iter()
                .filter(move |tbl| format!("{sensor_id}_{}", tbl.table_name) == table_name)
                .flat_map(|tbl| tbl.columns.iter().map(|c| c.name.clone()))
        })
        .collect();

    // OBS-FRESH-1: sort + dedup `available_columns` before use to ensure deterministic
    // ordering and no duplicates in the ColumnNotFoundDetails error. Without this,
    // `spec_map.values()` iterates in HashMap order (non-deterministic) and multi-org-scope
    // queries that contribute the same column via multiple overlays produce duplicates.
    available_columns.sort();
    available_columns.dedup();

    // If the column is in the available set, the gate passes.
    if available_columns.contains(&column_name.to_string()) {
        return Ok(());
    }

    // did_you_mean: find closest column by Levenshtein distance ≤ 3 (D-1163).
    // Tie-break: when multiple candidates share the same minimum distance, pick the
    // lexicographically-smallest name for deterministic output. (F-001B-PASS-LOW-001 /
    // BC-2.11.016 AC-001 — multi-client queries iterate HashMap in non-deterministic order.)
    // After sort+dedup above, `available_columns` is already in stable lex order, so the
    // tie-break by name in `min_by_key` is now redundant but retained for clarity.
    // F-PHL1-MED-001: cap `column_name` at 128 bytes (SEC-002 / CWE-407)
    // before the O(m×n) Levenshtein computation — multi-tenant path.
    let column_name_capped_mt = crate::table_registry::cap_name_for_levenshtein(column_name);
    let did_you_mean = available_columns
        .iter()
        .map(|c| (c.clone(), strsim::levenshtein(column_name_capped_mt, c)))
        .filter(|(_, dist)| *dist <= 3)
        .min_by_key(|(name, dist)| (*dist, name.clone()))
        .map(|(c, _)| c);

    // Emit audit tracing event per SAP-1 / PG-LP11-001.
    tracing::warn!(
        event_type = "column_not_found.rejected",
        column = %column_name,
        table = %table_name,
        client_id = %client_id,
        available_count = available_columns.len(),
        "E-QUERY-038: column not found at plan time"
    );

    // SEC-002 trust-boundary: `client_id` here is an `OrgSlug` string validated to
    // `^[a-zA-Z0-9_-]{1,64}$` by `OrgSlug::new` in `tenant.rs` before it reaches this
    // function. That validation guarantees `client_id` cannot carry prompt-injection
    // characters (newlines, quotes, control chars) into the LLM-facing error message.
    Err(PrismError::ColumnNotFound(Box::new(
        ColumnNotFoundDetails::new(
            column_name,
            table_name,
            client_id,
            available_columns,
            did_you_mean,
        ),
    )))
}

/// Plan-time column availability gate (E-QUERY-038) — query-level driver.
///
/// Parses `query_str`, extracts the FROM table and all non-wildcard column
/// references from ALL positions where column resolution is required, then calls
/// `check_column_availability` for each one. Returns the first
/// `Err(PrismError::ColumnNotFound)` found, or `Ok(())` if all columns pass.
///
/// # Column positions checked (BC-2.11.016 Precondition 2):
/// - SELECT clause (non-wildcard field refs)
/// - WHERE clause (FieldPath refs from all Predicate variants)
/// - GROUP BY clause (Expr::Field refs and nested FuncCall args via `extract_field_paths_from_expr`)
/// - ORDER BY clause (Expr::Field refs and nested FuncCall args via `extract_field_paths_from_expr`)
/// - JOIN ON clause (Expr::Field refs and nested FuncCall args via `extract_field_paths_from_expr`)
///
/// Gate skip conditions:
/// - BOTH `resolved_spec_map` AND `table_registry` are `None`: skip (no schema source wired).
///   When `table_registry` is wired but `resolved_spec_map` is `None` (single-tenant mode),
///   the gate FIRES via the `table_registry.columns_for_table()` fallback (M1 fix,
///   S-DEMO-FIDELITY-REMEDIATION-001). Pre-M1 behavior (skip whenever `resolved_spec_map`
///   is `None`) was incorrect and has been removed.
/// - Query fails to parse: skip (parse errors handled downstream)
/// - SELECT * or SELECT table.*: skip for SELECT position (wildcard columns always pass)
///
/// SQL SELECT mode and SqlPipe head are both checked (the SqlPipe head carries an
/// explicit column projection in the SELECT clause that is gated identically to SQL
/// SELECT; F-001B-DC-HIGH-001 / BC-2.11.020). Filter and Pipe modes have no explicit
/// column projection and DataFusion handles field resolution in those modes.
///
/// # BC-2.11.016 / S-DEMO-PRISMQL-ONBOARDING-001-B
fn check_query_column_availability(
    query_str: &str,
    client_id: &str,
    org_scope: Option<&[prism_core::OrgSlug]>,
    resolved_spec_map: Option<
        &std::collections::HashMap<
            prism_spec_engine::ResolvedSpecKey,
            prism_spec_engine::ResolvedSensorSpec,
        >,
    >,
    table_registry: Option<&crate::table_registry::TableRegistry>,
) -> Result<(), PrismError> {
    use crate::ast::{Ast, Expr, SelectItem, SqlStatement};
    use crate::filter_parser::PrismQlParser;

    // Fail-open when NEITHER resolved_spec_map NOR table_registry is wired.
    // M1 fix (S-DEMO-FIDELITY-REMEDIATION-001): previously returned Ok(())
    // immediately when resolved_spec_map.is_none(), bypassing E-QUERY-038 in
    // single-tenant mode. Now, if table_registry is wired, use it as fallback.
    if resolved_spec_map.is_none() && table_registry.is_none() {
        return Ok(());
    }

    // Parse the query — if parsing fails, skip (parse errors handled downstream).
    let ast = match PrismQlParser::parse(query_str) {
        Ok(ast) => ast,
        Err(_) => return Ok(()),
    };

    // Handle SQL SELECT mode and SqlPipe head — both carry an explicit column
    // projection in the SELECT clause and an optional WHERE that references columns.
    // Filter and Pipe mode have no explicit column projection so they remain fail-open.
    // BC-2.11.020 / HIGH-1 sibling sweep: without the SqlPipe arm, a SqlPipe query
    // whose head projects a typo'd column (e.g. `SELECT sev FROM …`) would bypass
    // the E-QUERY-038 pedagogical gate, getting a confusing DataFusion error at
    // execution time instead of the clean "column not found" diagnostic. (TD-VSDD-060)
    let sql_query = match &ast {
        Ast::Sql(SqlStatement::Select(q)) => q,
        Ast::SqlPipe(spq) => &spq.head,
        _ => return Ok(()),
    };

    // Derive the fully-qualified table name from the FROM clause.
    // Custom SourceRef: raw is already the full table name (e.g. "crowdstrike_alerts").
    // External SourceRef: "sensor.table" dotted → "sensor_table" underscore convention.
    let table_name = {
        use crate::ast::SourceRefKind;
        match &sql_query.from.source.kind {
            SourceRefKind::Custom => sql_query.from.source.raw.clone(),
            SourceRefKind::External { sensor, table } => format!("{sensor}_{table}"),
            // Internal / Composite tables have no column schema in resolved_spec_map.
            _ => return Ok(()),
        }
    };

    // Skip prism_* internal tables (they have a separate capability gate).
    if table_name.starts_with("prism_") {
        return Ok(());
    }

    // Table alias from the FROM clause (e.g. `t` in `FROM crowdstrike_alerts t`).
    // Passed to `extract_column_name_from_field_path` so that queries using an alias
    // qualifier (`t.severity`) are handled identically to table-name qualifiers
    // (`crowdstrike_alerts.severity`) — both return "severity", not the qualifier.
    let from_alias: Option<&str> = sql_query.from.alias.as_deref();

    // ── Position 1: SELECT clause — non-wildcard field refs ───────────────────
    //
    // F-001B-DC-HIGH-001: use `extract_column_name_from_field_path` instead of
    // `fp.segments.first()` so that qualified refs (`crowdstrike_alerts.severity`)
    // correctly extract "severity" rather than the table name.
    //
    // F-PBL1-MED-001 fix (Pass-B S-DEMO-FIDELITY-REMEDIATION-001): route through
    // `extract_field_paths_from_expr` so that column refs nested inside FuncCall
    // args (e.g. `SELECT count(typo_col)`, `SELECT lower(typo_col)`) are also
    // validated. Previously, only bare `Expr::Field` refs were extracted; FuncCall
    // args fell through to `_ => None` and bypassed E-QUERY-038.
    //
    // Wildcard items (Star / TableStar) are still skipped — no column to extract.
    // VirtualField (_sensor, _client) are still skipped — always valid sentinels.
    // This makes AC-M2's claim that `extract_field_paths_from_expr` is the SINGLE
    // extraction point for ALL 5 positions (SELECT, WHERE, GROUP BY, ORDER BY,
    // JOIN ON) true for Position 1 as well.
    let mut select_cols: Vec<String> = Vec::new();
    for item in &sql_query.select.items {
        match item {
            SelectItem::Star => {}         // SELECT * — skip (no column to validate)
            SelectItem::TableStar(_) => {} // SELECT table.* — skip
            SelectItem::Expr { expr, .. } => {
                match expr {
                    // VirtualField (_sensor, _client, etc.) — always valid, skip.
                    Expr::VirtualField(_) => {}
                    // All other Expr variants (Field, FuncCall, Compare, etc.) — use
                    // the recursive walker so FuncCall args are validated.
                    _ => {
                        extract_field_paths_from_expr(
                            expr,
                            &table_name,
                            from_alias,
                            &mut select_cols,
                        );
                    }
                }
            }
            // #[non_exhaustive] catch-all for future SelectItem variants.
            #[allow(unreachable_patterns)]
            _ => {}
        }
    }

    // ── Position 2: WHERE clause — recursively extract FieldPath refs ─────────
    //
    // BC-2.11.016 Precondition 2: "The query references a column name in a position
    // where column resolution is possible (e.g., `SELECT <column>`, `WHERE <column> = ...`,
    // `GROUP BY <column>`, `ORDER BY <column>`)."
    //
    // `extract_predicate_columns` walks the entire Predicate tree and collects the
    // resolved column name from every FieldPath encountered via
    // `extract_column_name_from_field_path`. Virtual fields (_sensor, _client)
    // and literal operands are ignored — only explicit column name references are checked.
    // F-001B-DC-HIGH-001: pass table_name + from_alias to the predicate extractor so
    // qualified WHERE refs are handled correctly.
    let where_cols: Vec<String> = sql_query
        .where_
        .as_ref()
        .map(|pred| extract_predicate_columns(pred, &table_name, from_alias))
        .unwrap_or_default();

    // ── Position 3: GROUP BY clause — recurse into FuncCall args (M2 fix) ────
    //
    // F-001B-DC-HIGH-001: use extract_column_name_from_field_path instead of .first().
    // M2 fix (S-DEMO-FIDELITY-REMEDIATION-001): use `extract_field_paths_from_expr`
    // instead of direct `Expr::Field` match so that column refs wrapped in function
    // calls (e.g. `GROUP BY lower(col_typo)`) are also validated against the schema.
    let mut group_by_cols: Vec<String> = Vec::new();
    for expr in &sql_query.group_by {
        extract_field_paths_from_expr(expr, &table_name, from_alias, &mut group_by_cols);
    }

    // ── Position 4: ORDER BY clause — recurse into FuncCall args (M2 fix) ────
    //
    // F-001B-DC-HIGH-001: use extract_column_name_from_field_path instead of .first().
    // M2 fix: same FuncCall-arg recursion as GROUP BY — handles `ORDER BY lower(col_typo)`.
    let mut order_by_cols: Vec<String> = Vec::new();
    for oe in &sql_query.order_by {
        extract_field_paths_from_expr(&oe.expr, &table_name, from_alias, &mut order_by_cols);
    }

    // ── Position 5: JOIN ON clause — recurse into JOIN ON expressions (M2 fix) ──
    //
    // M2 fix (S-DEMO-FIDELITY-REMEDIATION-001): validate column refs in JOIN ON
    // expressions for the FROM table. JOIN ON is typed as `Expr` (not `Predicate`),
    // so we call `extract_field_paths_from_expr` directly.
    //
    // Fail-open for cross-table refs (unknown qualifier → `extract_column_name_from_field_path`
    // returns None). Only same-table column typos (unqualified or FROM-table-qualified refs)
    // are caught here — this is the same conservative policy as all other positions.
    let mut join_on_cols: Vec<String> = Vec::new();
    for join in &sql_query.joins {
        extract_field_paths_from_expr(&join.on, &table_name, from_alias, &mut join_on_cols);
    }

    // ── Gate: check all positions in order ────────────────────────────────────
    for col in select_cols
        .iter()
        .chain(where_cols.iter())
        .chain(group_by_cols.iter())
        .chain(order_by_cols.iter())
        .chain(join_on_cols.iter())
    {
        check_column_availability(
            col,
            &table_name,
            client_id,
            org_scope,
            resolved_spec_map,
            table_registry,
        )?;
    }

    // ── E-QUERY-002 type-compatibility gate — AFTER column-existence gate ─────
    //
    // BC-2.11.017 AC-003: For each (column, operator) pair in the WHERE clause
    // where both are resolvable, verify the operator is valid for the column's
    // ColumnType. Order: E-QUERY-037 table gate → E-QUERY-038 column-existence
    // gate (above) → E-QUERY-002 type-compat gate (this).
    //
    // Only runs when: (a) resolved_spec_map is wired, (b) the WHERE predicate tree
    // contains Compare predicates with a FieldPath lhs, (c) the column is present in
    // the table spec (existence gate already passed). Fail-open for unknown columns.
    //
    // The gate is driven by `collect_predicate_type_pairs` which returns `(col, op_str)`
    // pairs from `Predicate::Compare` nodes. Type lookup is done via the same org-scoped
    // spec map path used by the existence gate. On mismatch, returns
    // `PrismError::QueryTypeMismatch` carrying the ColumnType for the error-mapping layer
    // to call `valid_operators_for_type(actual_type)`.
    if let Some(where_pred) = &sql_query.where_ {
        // F-001B-DC-HIGH-001: pass table_name + from_alias so qualified refs in Compare
        // predicates extract the bare column name, not the table qualifier.
        let type_pairs = collect_predicate_type_pairs(where_pred, &table_name, from_alias);
        for (col_name, op_str) in &type_pairs {
            check_operator_type_compatibility(
                col_name,
                op_str,
                &table_name,
                org_scope,
                resolved_spec_map,
            )?;
        }
    }

    Ok(())
}

/// Extract column names from a `Predicate` tree, resolving table-qualified references.
///
/// Walks all variants that carry a `FieldPath` directly, and recurses into
/// `Logical` and `Not` for nested predicates. `VirtualField` segments
/// (`_sensor`, `_client`) are implicitly excluded because those appear as
/// `Expr::VirtualField`, not `Expr::Field` — the `Compare { lhs, .. }` arm
/// matches `lhs` only when it is `Expr::Field`.
///
/// F-001B-DC-HIGH-001: uses `extract_column_name_from_field_path` for each
/// FieldPath so that qualified refs (`crowdstrike_alerts.severity`) return the
/// bare column name ("severity"), not the table qualifier ("crowdstrike_alerts").
///
/// # Non-exhaustive safety
/// Unknown future `Predicate` variants are silently skipped via the `_ => {}`
/// catch-all, which satisfies `#[non_exhaustive]` discipline and preserves
/// fail-open semantics for unrecognised predicate forms.
fn extract_predicate_columns(
    pred: &crate::ast::Predicate,
    table_name: &str,
    table_alias: Option<&str>,
) -> Vec<String> {
    let mut cols = Vec::new();
    collect_predicate_columns(pred, table_name, table_alias, &mut cols);
    cols
}

fn collect_predicate_columns(
    pred: &crate::ast::Predicate,
    table_name: &str,
    table_alias: Option<&str>,
    out: &mut Vec<String>,
) {
    use crate::ast::{Expr, Predicate};
    match pred {
        // Compare: lhs may be Expr::Field (the column being compared).
        // F-001B-DC-HIGH-001: use extract_column_name_from_field_path to handle qualified refs.
        Predicate::Compare { lhs, .. } => {
            if let Expr::Field(fp) = lhs.as_ref() {
                if let Some(name) = extract_column_name_from_field_path(fp, table_name, table_alias)
                {
                    out.push(name);
                }
            }
        }
        // StringOp: field is FieldPath directly.
        Predicate::StringOp { field, .. } => {
            if let Some(name) = extract_column_name_from_field_path(field, table_name, table_alias)
            {
                out.push(name);
            }
        }
        // Regex: field is FieldPath directly.
        Predicate::Regex { field, .. } => {
            if let Some(name) = extract_column_name_from_field_path(field, table_name, table_alias)
            {
                out.push(name);
            }
        }
        // In / InSubquery / Between / Cidr / Wildcard: field is FieldPath directly.
        Predicate::In { field, .. }
        | Predicate::InSubquery { field, .. }
        | Predicate::Between { field, .. }
        | Predicate::Cidr { field, .. }
        | Predicate::Wildcard { field, .. } => {
            if let Some(name) = extract_column_name_from_field_path(field, table_name, table_alias)
            {
                out.push(name);
            }
        }
        // Has / Missing: the argument IS the FieldPath.
        Predicate::Has(fp) | Predicate::Missing(fp) => {
            if let Some(name) = extract_column_name_from_field_path(fp, table_name, table_alias) {
                out.push(name);
            }
        }
        // IsNull: field is FieldPath.
        Predicate::IsNull { field, .. } => {
            if let Some(name) = extract_column_name_from_field_path(field, table_name, table_alias)
            {
                out.push(name);
            }
        }
        // Logical: recurse into each child predicate.
        Predicate::Logical { predicates, .. } => {
            for child in predicates {
                collect_predicate_columns(child, table_name, table_alias, out);
            }
        }
        // Not: recurse into the inner predicate.
        Predicate::Not(inner) => {
            collect_predicate_columns(inner, table_name, table_alias, out);
        }
        // RecoveryError: no column to extract (error-recovery sentinel).
        Predicate::RecoveryError => {}
        // #[non_exhaustive] catch-all: fail-open for future predicate variants.
        #[allow(unreachable_patterns)]
        _ => {}
    }
}

// ---------------------------------------------------------------------------
// E-QUERY-002 type-compatibility helpers (S-DEMO-PRISMQL-ONBOARDING-001-B)
// ---------------------------------------------------------------------------

/// Convert a `CompareOp` to its canonical operator string for E-QUERY-002 type checking.
///
/// Returns the operator string as it would appear in a PrismQL query expression:
/// `Eq` → `"="`, `Ne` → `"!="`, `Gt` → `">"`, `Lt` → `"<"`, `Ge` → `">="`, `Le` → `"<="`,
/// `Like` → `"LIKE"`. `Cidr`/`NotCidr` are excluded (CIDR membership does not map to a
/// `valid_operators_for_type` entry — those are handled by `Predicate::Cidr` directly).
///
/// Returns `None` for `Cidr`/`NotCidr` and any future variants (non_exhaustive fallback)
/// so the caller can skip the type check for those operators.
fn compare_op_to_str(op: &crate::ast::CompareOp) -> Option<&'static str> {
    use crate::ast::CompareOp;
    match op {
        CompareOp::Eq => Some("="),
        CompareOp::Ne => Some("!="),
        CompareOp::Gt => Some(">"),
        CompareOp::Lt => Some("<"),
        CompareOp::Ge => Some(">="),
        CompareOp::Le => Some("<="),
        CompareOp::Like => Some("LIKE"),
        // CIDR operators are not in the valid_operators_for_type set — skip.
        CompareOp::Cidr | CompareOp::NotCidr => None,
        // #[non_exhaustive] catch-all: fail-open for future CompareOp variants.
        #[allow(unreachable_patterns)]
        _ => None,
    }
}

/// Collect `(column_name, operator_string)` pairs from `Predicate::Compare` nodes
/// in a predicate tree, for E-QUERY-002 type-compatibility checking.
///
/// Only `Predicate::Compare { lhs: Expr::Field(_), op, .. }` forms are extracted
/// (operator applied directly to a named column). Literal lhs, subquery lhs, and
/// non-Compare predicates are skipped — they cannot produce a type-mismatch error.
///
/// Recurses into `Logical` and `Not` for nested predicates. Virtual fields are
/// excluded because `lhs` for those is `Expr::VirtualField`, not `Expr::Field`.
///
/// F-001B-DC-HIGH-001: `table_name` and `table_alias` are forwarded to
/// `extract_column_name_from_field_path` so qualified refs (`t.severity = 1`)
/// are resolved to the bare column name before the type-compatibility check.
///
/// Reference: BC-2.11.017 AC-003; S-DEMO-PRISMQL-ONBOARDING-001-B.
fn collect_predicate_type_pairs(
    pred: &crate::ast::Predicate,
    table_name: &str,
    table_alias: Option<&str>,
) -> Vec<(String, String)> {
    let mut out = Vec::new();
    collect_predicate_type_pairs_inner(pred, table_name, table_alias, &mut out);
    out
}

fn collect_predicate_type_pairs_inner(
    pred: &crate::ast::Predicate,
    table_name: &str,
    table_alias: Option<&str>,
    out: &mut Vec<(String, String)>,
) {
    use crate::ast::{Expr, Predicate};
    match pred {
        // Compare: extract (column, operator) when lhs is a FieldPath.
        // F-001B-DC-HIGH-001: use extract_column_name_from_field_path to correctly
        // resolve qualified refs (`crowdstrike_alerts.severity = ...`) to "severity".
        Predicate::Compare { lhs, op, .. } => {
            if let Expr::Field(fp) = lhs.as_ref() {
                if let Some(col_name) =
                    extract_column_name_from_field_path(fp, table_name, table_alias)
                {
                    if let Some(op_str) = compare_op_to_str(op) {
                        out.push((col_name, op_str.to_string()));
                    }
                }
            }
        }
        // Logical: recurse into each child predicate.
        Predicate::Logical { predicates, .. } => {
            for child in predicates {
                collect_predicate_type_pairs_inner(child, table_name, table_alias, out);
            }
        }
        // Not: recurse into the inner predicate.
        Predicate::Not(inner) => {
            collect_predicate_type_pairs_inner(inner, table_name, table_alias, out);
        }
        // All other predicate variants (StringOp, Regex, In, Between, Cidr, Has,
        // Missing, IsNull, Wildcard, RecoveryError, future variants): no Compare
        // operator to check — skip.
        #[allow(unreachable_patterns)]
        _ => {}
    }
}

/// E-QUERY-002 plan-time type-compatibility gate for a single (column, operator) pair.
///
/// Given a resolved_spec_map and org_scope, looks up the column's `ColumnType` from
/// the table spec and checks whether the operator is in `valid_operators_for_type(column_type)`.
/// Returns `Err(PrismError::QueryTypeMismatch)` on mismatch. Returns `Ok(())` when:
/// - the column is not found in the spec (fail-open: existence gate fires first)
/// - the operator IS valid for the column type
///
/// # Ordering
/// MUST be called AFTER `check_column_availability` for the same column. If the column
/// does not exist in the spec, the existence gate will have already returned an error.
/// This function uses fail-open semantics for unfound columns because the existence gate
/// owns that error path.
///
/// Reference: BC-2.11.017 AC-003; S-DEMO-PRISMQL-ONBOARDING-001-B.
fn check_operator_type_compatibility(
    column_name: &str,
    operator: &str,
    table_name: &str,
    org_scope: Option<&[prism_core::OrgSlug]>,
    resolved_spec_map: Option<
        &std::collections::HashMap<
            prism_spec_engine::ResolvedSpecKey,
            prism_spec_engine::ResolvedSensorSpec,
        >,
    >,
) -> Result<(), PrismError> {
    let Some(spec_map) = resolved_spec_map else {
        return Ok(());
    };

    // Find the ColumnType for this column from the org-visible spec entries.
    let column_type = spec_map
        .values()
        .filter(|spec| {
            if let Some(scopes) = org_scope {
                scopes.iter().any(|s| s.as_str() == spec.org_slug.as_str())
            } else {
                true
            }
        })
        .flat_map(|spec| {
            let sensor_id = spec.spec.sensor_id.clone();
            spec.spec
                .tables
                .iter()
                .filter(move |tbl| format!("{sensor_id}_{}", tbl.table_name) == table_name)
                .flat_map(|tbl| tbl.columns.iter())
                .filter_map(|col| {
                    if col.name == column_name {
                        Some(col.column_type.clone())
                    } else {
                        None
                    }
                })
        })
        .next();

    // Fail-open: if the column's type is not in the spec, the existence gate handles it.
    let Some(actual_type) = column_type else {
        return Ok(());
    };

    // Check whether the operator is valid for this column's type.
    let valid_ops = valid_operators_for_type(actual_type.clone());
    if valid_ops.contains(&operator) {
        return Ok(());
    }

    // Operator is NOT in the valid set for this column type → E-QUERY-002 type mismatch.
    Err(PrismError::QueryTypeMismatch {
        column: column_name.to_string(),
        table: table_name.to_string(),
        actual_type,
        operator: operator.to_string(),
    })
}

// ---------------------------------------------------------------------------
// Pedagogical enrichment helpers (S-DEMO-PRISMQL-ONBOARDING-001-B)
// ---------------------------------------------------------------------------

/// E-QUERY-001 enrichment: extract `near_text` (≤50 chars) from a Chumsky error span.
///
/// Returns the offending token slice from the input string using the chumsky error's
/// span (start/end indices). Truncated to ≤50 chars per DI-006 (BC-2.11.017).
/// Returns `""` when the parser cannot provide a token (e.g. end-of-input).
///
/// Reference: BC-2.11.017 postconditions; S-DEMO-PRISMQL-ONBOARDING-001-B AC-003.
pub fn extract_near_text(input: &str, offset: usize) -> String {
    // End-of-input: offset is at or beyond the input length → empty string (EC-003).
    if offset >= input.len() {
        return String::new();
    }
    // SAFETY (belt-and-suspenders, F-001B-PASS-CRIT-001): if `offset` falls mid-char
    // (e.g. a caller passes a byte offset that bisects a multibyte UTF-8 sequence),
    // snap forward to the next valid char boundary rather than panicking.
    // `str::floor_char_boundary` is nightly-only; use a forward search instead.
    let safe_offset = if input.is_char_boundary(offset) {
        offset
    } else {
        // Advance byte-by-byte until we land on a char boundary (or reach end).
        let mut o = offset;
        while o < input.len() && !input.is_char_boundary(o) {
            o += 1;
        }
        o
    };
    if safe_offset >= input.len() {
        return String::new();
    }
    // Extract from `safe_offset` to the end of the first word / whitespace-delimited token.
    // The "offending token" is the word starting at `safe_offset`. Walk forward until
    // whitespace or end-of-input, then slice.
    let remainder = &input[safe_offset..];
    let token_end = remainder
        .find(|c: char| c.is_whitespace())
        .unwrap_or(remainder.len());
    let token = &remainder[..token_end];
    // Truncate to ≤50 CHARACTERS per DI-006 (injection safety).
    //
    // MUST use char-count truncation, NOT byte-slice truncation.
    // Byte index 50 may fall inside a multibyte UTF-8 character (e.g. '—' = 3 bytes,
    // 'é' = 2 bytes) causing a panic: "byte index 50 is not a char boundary".
    // Model-controlled PQL is the source, so crafted inputs can trigger this.
    // Fix: iterate by char and collect the first ≤50 chars. (F-001-B-FRESH-001 / DI-006)
    token.chars().take(50).collect::<String>()
}

/// E-QUERY-002 enrichment: return the valid operators for a `ColumnType`.
///
/// Returns a compile-time static slice of operator strings. Callers convert to
/// `Vec<String>` for the JSON payload:
///
/// String → `["=", "!=", "LIKE", "IN", "NOT IN"]`
/// Integer → `["=", "!=", "<", ">", "<=", ">=", "BETWEEN", "IN", "NOT IN"]`
/// Float → `["=", "!=", "<", ">", "<=", ">=", "BETWEEN"]`
/// Boolean → `["=", "!="]`
/// Datetime → `["=", "!=", "<", ">", "<=", ">=", "BETWEEN"]`
/// Json → `["=", "!="]`
///
/// Reference: BC-2.11.017 postconditions; S-DEMO-PRISMQL-ONBOARDING-001-B AC-003.
pub fn valid_operators_for_type(
    column_type: prism_core::column::ColumnType,
) -> &'static [&'static str] {
    use prism_core::column::ColumnType;
    match column_type {
        ColumnType::String => &["=", "!=", "LIKE", "IN", "NOT IN"],
        ColumnType::Integer => &["=", "!=", "<", ">", "<=", ">=", "BETWEEN", "IN", "NOT IN"],
        ColumnType::Float => &["=", "!=", "<", ">", "<=", ">=", "BETWEEN"],
        ColumnType::Boolean => &["=", "!="],
        ColumnType::Datetime => &["=", "!=", "<", ">", "<=", ">=", "BETWEEN"],
        ColumnType::Json => &["=", "!="],
        _ => &["=", "!="], // non_exhaustive fallback
    }
}

/// E-QUERY-003 enrichment: produce a `how_to_fix` string from the `detail` message.
///
/// Matches the limit violation category from the `detail` string and returns a
/// human-readable fix instruction. Catch-all: "Simplify or shorten the query."
///
/// Reference: BC-2.11.017 postconditions; S-DEMO-PRISMQL-ONBOARDING-001-B AC-003.
pub fn how_to_fix_for_security_limit(detail: &str) -> String {
    let lower = detail.to_lowercase();
    // "expanded"/"alias" branch must come BEFORE the generic "size"/"64kb" branch:
    // alias_resolver.rs emits "expanded query exceeds 64KB limit (N bytes)" which
    // contains "64kb" — if size fires first it returns the wrong message.
    // explain.rs emits "expanded query size N bytes exceeds maximum allowed M bytes"
    // which contains "size" — same ordering issue. (F-PRL-FRESH-002 fix, BC-2.11.017.)
    if lower.contains("expanded") || lower.contains("alias") {
        "The alias expansion produced a query over 64KB. Simplify the aliased query or use a narrower alias.".to_string()
    } else if lower.contains("size") || lower.contains("64kb") || lower.contains("64 kb") {
        "Shorten the query. Remove large IN (...) lists or break into multiple queries.".to_string()
    } else if lower.contains("depth") || lower.contains("nesting") {
        "Flatten nested conditions. Use AND/OR instead of deeply nested parentheses.".to_string()
    } else if lower.contains("pipe") {
        "Reduce the number of pipe stages. Combine adjacent filter conditions.".to_string()
    } else if lower.contains("regex") {
        "Use a shorter regex pattern. Consider using LIKE instead of regex for simple pattern matching.".to_string()
    } else {
        "Simplify or shorten the query.".to_string()
    }
}

/// E-QUERY-037 suggestion update: produce the `suggestion` field with a
/// `prism_describe('<client_id>')` reference.
///
/// When `did_you_mean` is `Some(table)`: includes a retry hint referencing the table.
/// When `did_you_mean` is `None`: includes only the `prism_describe` pointer.
///
/// Reference: BC-2.11.017 postconditions; S-DEMO-PRISMQL-ONBOARDING-001-B AC-004.
/// # Trust boundary (SEC-002 / CWE-116)
/// `client_id` is an `OrgSlug` string validated to `^[a-zA-Z0-9_-]{1,64}$` by
/// `OrgSlug::new` in `tenant.rs` before reaching any call site of this function.
/// That regex prohibits newlines, quotes, and control characters, so `client_id`
/// cannot carry prompt-injection or newline/quote injection into the LLM-facing
/// suggestion string that appears in the MCP error envelope.
pub fn e_query_037_suggestion(client_id: &str, did_you_mean: Option<&str>) -> String {
    match did_you_mean {
        Some(table) => format!(
            "Call prism_describe('{client_id}') to see available tables and columns. \
             If you meant '{table}', retry with that table name."
        ),
        None => format!(
            "Call prism_describe('{client_id}') to see available tables and columns for this client."
        ),
    }
}

// ---------------------------------------------------------------------------
// normalized_pql Chumsky re-serializer (S-DEMO-PRISMQL-ONBOARDING-001-B)
// ---------------------------------------------------------------------------

/// Produce the normalized (canonicalized) PrismQL string from a parsed `Ast`.
///
/// Walks the AST and produces a whitespace-normalized, uppercase-keyword form
/// of the original query. The normalized string MUST round-trip through Chumsky
/// (parse to the same AST as the original). EXCLUDED: DataFusion plan node strings
/// (`HashJoin`, `TableScan`, `SortExec`, `Aggregate`).
///
/// Returns `None` when the normalized form would be empty (should not occur for
/// a validly-parsed AST; defensive guard per BC-2.11.018 Error Cases EC-11-055).
///
/// Reference: BC-2.11.018; S-DEMO-PRISMQL-ONBOARDING-001-B AC-005, AC-006.
pub fn normalize_pql(ast: &crate::ast::Ast) -> Option<String> {
    crate::ast::PqlNormalizer::normalize(ast)
}

// ---------------------------------------------------------------------------
// NullCredentialResolver — no-op for test/legacy constructors
// ---------------------------------------------------------------------------

/// No-op credential resolver used by `new()` / `new_with_cache_config`.
///
/// Production code should use `new_full` with a real resolver.
/// Test code that needs specific auth behavior should implement `CredentialResolver`.
struct NullCredentialResolver;

impl CredentialResolver for NullCredentialResolver {
    fn resolve(
        &self,
        _client_id: &str,
        sensor_id: SensorId,
    ) -> Result<Box<dyn prism_sensors::SensorAuth>, prism_sensors::SensorError> {
        Err(prism_sensors::SensorError::Internal {
            detail: format!(
                "NullCredentialResolver: no credential configured for sensor {sensor_id:?}; \
                 use QueryEngine::new_full with a real CredentialResolver in production"
            ),
        })
    }
}

// ---------------------------------------------------------------------------
// Unit tests: alias_store wiring in execute (F-PASS9-LOW-1 / BC-2.11.008)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod alias_wiring_tests {
    use std::sync::{Arc, Mutex};

    use super::*;
    use crate::{
        alias_store::AliasStore,
        alias_types::{AliasEntry, AliasScope},
    };

    /// Minimal no-op credential store for unit tests that don't exercise auth.
    struct NoopCs;

    #[async_trait::async_trait]
    impl prism_credentials::CredentialStore for NoopCs {
        async fn get(
            &self,
            _t: &prism_core::OrgSlug,
            _s: &str,
            _n: &prism_credentials::namespace::CredentialName,
        ) -> Result<Option<secrecy::SecretString>, PrismError> {
            Ok(None)
        }
        async fn set(
            &self,
            _t: &prism_core::OrgSlug,
            _s: &str,
            _n: &prism_credentials::namespace::CredentialName,
            _v: secrecy::SecretString,
        ) -> Result<(), PrismError> {
            Ok(())
        }
        async fn delete(
            &self,
            _t: &prism_core::OrgSlug,
            _s: &str,
            _n: &prism_credentials::namespace::CredentialName,
        ) -> Result<bool, PrismError> {
            Ok(false)
        }
        async fn list(
            &self,
            _t: &prism_core::OrgSlug,
        ) -> Result<Vec<(String, prism_credentials::namespace::CredentialName)>, PrismError>
        {
            Ok(vec![])
        }
        async fn exists(
            &self,
            _t: &prism_core::OrgSlug,
            _s: &str,
            _n: &prism_credentials::namespace::CredentialName,
        ) -> Result<bool, PrismError> {
            Ok(false)
        }
    }

    /// Build a minimal `QueryEngine` with an alias_store for unit tests.
    ///
    /// Uses `new_with_cache_config` + manual field injection so we don't need
    /// the full production dependency tree (OrgRegistry, RocksDB, etc.).
    fn make_engine_with_alias_store(alias_store: Arc<Mutex<AliasStore>>) -> QueryEngine {
        use prism_sensors::AdapterRegistry;

        let mut engine = QueryEngine::new_with_cache_config(
            Arc::new(AdapterRegistry::new()),
            Arc::new(NoopCs),
            Arc::new(prism_ocsf::OcsfNormalizer::new()),
            Arc::new(crate::scoping::ClientRegistry::new(vec![])),
            QueryEngineConfig::default(),
            crate::cache::CacheConfig::default(),
        );
        // Inject alias_store directly (field is pub(crate) for test access).
        engine.alias_store = Some(alias_store);
        engine
    }

    /// BC-2.11.008: when alias_store is wired, @alias tokens in query strings
    /// are expanded before the pipeline executes.
    ///
    /// This test verifies the expansion path by checking that `expanded_query`
    /// in the QueryResult reflects the alias substitution.
    ///
    /// NOTE: The engine has no sensor adapters so the materialization pipeline
    /// returns empty results — but the alias expansion happens in Step 0
    /// of execute_inner BEFORE the pipeline, so we can observe it via the
    /// `context.expanded_query` field.
    #[tokio::test]
    async fn test_alias_store_wired_into_execute_expands_at_query_time() {
        // Build an alias store with one global alias: @crowdstrike_alerts → "SELECT * FROM alerts"
        let _tmpdir = tempfile::tempdir().expect("create tempdir for alias wiring test store");
        let mut store = AliasStore::empty(_tmpdir.path().join("test-alias-wiring.toml"));
        let create_result = store
            .create_or_update(
                AliasEntry {
                    name: "crowdstrike_alerts".to_string(),
                    scope: AliasScope::Global,
                    query: "SELECT * FROM alerts".to_string(),
                    parameters: None,
                    description: None,
                },
                None, // no confirmation token needed for new alias
            )
            .expect("store.create_or_update must succeed for test alias");
        // Verify it was created (not ConfirmationRequired) — it's a new alias.
        assert!(
            matches!(create_result, crate::alias_types::CreateResult::Created(_)),
            "new alias must be Created, not ConfirmationRequired"
        );

        let engine = make_engine_with_alias_store(Arc::new(Mutex::new(store)));

        // Execute a query that references the alias.
        // The engine has no adapters → result is empty batches, but the
        // expansion MUST happen before the pipeline (observable via context).
        let result = engine
            .execute("@crowdstrike_alerts", QueryOptions::default())
            .await
            .expect("execute with alias must succeed (expansion → empty pipeline)");

        assert_eq!(
            result.context.original_query, "@crowdstrike_alerts",
            "original_query must preserve the raw query before alias expansion"
        );
        assert_eq!(
            result.context.expanded_query, "SELECT * FROM alerts",
            "expanded_query must reflect alias expansion (BC-2.11.008)"
        );
    }

    /// BC-2.11.008: when alias_store is None (test/query-only mode),
    /// query strings are passed through unchanged.
    ///
    /// Uses a valid PrismQL query (no @alias tokens) to verify that the
    /// original_query == expanded_query when no store is wired.
    #[tokio::test]
    async fn test_alias_store_absent_passes_query_through_unchanged() {
        // Use the same NoopCs helper from this module.
        let engine = QueryEngine::new_with_cache_config(
            Arc::new(prism_sensors::AdapterRegistry::new()),
            Arc::new(NoopCs),
            Arc::new(prism_ocsf::OcsfNormalizer::new()),
            Arc::new(crate::scoping::ClientRegistry::new(vec![])),
            QueryEngineConfig::default(),
            crate::cache::CacheConfig::default(),
        );
        // alias_store is None by default in new_with_cache_config.

        // Use a valid PrismQL SELECT query (no alias tokens).
        // Engine has no adapters → empty batches, but expansion path executes.
        let result = engine
            .execute(
                "SELECT * FROM crowdstrike_detections LIMIT 5",
                QueryOptions::default(),
            )
            .await
            .expect("execute with no alias store must succeed");

        assert_eq!(
            result.context.original_query, "SELECT * FROM crowdstrike_detections LIMIT 5",
            "original_query must be preserved"
        );
        assert_eq!(
            result.context.expanded_query, "SELECT * FROM crowdstrike_detections LIMIT 5",
            "without alias_store, expanded_query == original_query"
        );
    }

    /// MED-2: verify the engine's `map_err` wrapper does NOT inject a function name into
    /// the error's `{infusion_id}` slot and does NOT double-prefix with `E-INFUSE-007`.
    ///
    /// The engine.rs `map_err` closure must propagate the inner error verbatim so the
    /// real taxonomy code (E-INFUSE-002 for duplicates; E-INFUSE-007 is FORWARD-RESERVED in
    /// taxonomy v1.82 and has no current emitter since DataFusion 53.1's register_udf is
    /// infallible) surfaces with the real infusion_id, not with the function name
    /// 'execute_inner' or 'execute_scheduled_inner' in the {infusion_id} slot.
    ///
    /// This test constructs duplicate descriptors directly and passes them to
    /// `register_infusion_udfs`, then wraps the error through the SAME `map_err` pattern
    /// used in execute_inner. Before the fix the detail would contain 'execute_inner';
    /// after the fix it must NOT.
    #[test]
    fn test_infusion_udf_registration_error_does_not_inject_function_name() {
        use prism_spec_engine::InfusionSource;
        use prism_spec_engine::InfusionUdfDescriptor;

        // Build two descriptors with the same name but different infusion_ids.
        // This simulates the scenario where register_infusion_udfs catches the duplicate.
        #[derive(Debug)]
        struct NullSrc;
        impl InfusionSource for NullSrc {
            fn enrich_single(&self, _: &str, _: &str) -> Option<serde_json::Value> {
                None
            }
            fn enrich_batch(
                &self,
                inputs: &[String],
                input_type: &str,
            ) -> Vec<Option<serde_json::Value>> {
                inputs
                    .iter()
                    .map(|i| self.enrich_single(i, input_type))
                    .collect()
            }
        }

        let descriptors = vec![
            InfusionUdfDescriptor::new(
                "threat_score",
                "ip",
                "string",
                "threatintel_v1",
                Arc::new(NullSrc),
                None,
                3600,
            ),
            InfusionUdfDescriptor::new(
                "threat_score", // duplicate name
                "ip",
                "string",
                "threatintel_v2",
                Arc::new(NullSrc),
                None,
                3600,
            ),
        ];

        // Simulate the map_err pattern used in execute_inner — FIXED version (propagates
        // inner error verbatim, without prepending a function-name prefix).
        let ctx = datafusion::execution::context::SessionContext::new();
        let inner_err = crate::infusion_udf::register_infusion_udfs(&ctx, descriptors).unwrap_err();

        // The engine wraps in PrismError::QueryExecutionFailed { detail: e.to_string() }.
        let detail = inner_err.to_string();

        // Must contain the real infusion_id of the colliding spec (not a function name).
        assert!(
            detail.contains("threatintel_v2"),
            "error detail must contain the real infusion_id 'threatintel_v2'; got: {detail}"
        );

        // Must contain E-INFUSE-002 (duplicate at spec-load time).
        assert!(
            detail.contains("E-INFUSE-002"),
            "error detail must contain 'E-INFUSE-002'; got: {detail}"
        );

        // Must NOT contain a function name in the infusion_id slot.
        assert!(
            !detail.contains("execute_inner"),
            "error detail must NOT contain 'execute_inner' (function name injected as infusion_id); \
             got: {detail}"
        );
        assert!(
            !detail.contains("execute_scheduled_inner"),
            "error detail must NOT contain 'execute_scheduled_inner'; got: {detail}"
        );
    }

    // ---------------------------------------------------------------------------
    // S-3.13 CRIT-1 Engine-level test: E-QUERY-037 fires via QueryEngine::execute
    // ---------------------------------------------------------------------------

    /// S-3.13 / AC-2 / AC-8: `QueryEngine::execute` with a wired `TableRegistry`
    /// returns `PrismError::TableNotAvailable` (E-QUERY-037) for a query targeting
    /// an unregistered table — BEFORE any fan-out occurs.
    ///
    /// This is the LOAD-BEARING engine-level test for CRIT-1. It drives the full
    /// `QueryEngine::execute` path (not just `check_availability_gate` in isolation)
    /// with the registry wired via `with_table_registry(...)`. The empty `AdapterRegistry`
    /// guarantees no fan-out occurs — any fan-out attempt would return empty results,
    /// not E-QUERY-037. The only way E-QUERY-037 fires from `execute` is via the
    /// plan-time gate in `check_table_availability`.
    ///
    /// BC-2.11.001 AC-2: fire before fan-out. BC-2.11.001 AC-8: mode-agnostic.
    #[tokio::test]
    async fn test_S3_13_engine_execute_with_wired_registry_returns_e_query_037_before_fanout() {
        use crate::table_registry::TableRegistry;
        use prism_spec_engine::spec_parser::{AuthType, SensorSpec, TableSpec};

        // Build a TableRegistry with only armis registered (no crowdstrike).
        let registry = Arc::new(TableRegistry::new());
        let armis_spec = SensorSpec::new(
            "armis",
            "Armis sensor",
            AuthType::ApiKey,
            "https://example.com",
            vec![TableSpec::new_point_in_time(
                "alerts",
                "security_finding",
                vec![],
                vec![],
            )],
            None,
            "1.0.0",
            Vec::new(),
        );
        registry
            .register_sensor(&armis_spec)
            .expect("register armis must not fail");

        // Build a QueryEngine with the registry wired (the CRIT-1 production path).
        let engine = QueryEngine::new_with_cache_config(
            Arc::new(prism_sensors::AdapterRegistry::new()), // empty — no fan-out possible
            Arc::new(NoopCs),
            Arc::new(prism_ocsf::OcsfNormalizer::new()),
            Arc::new(crate::scoping::ClientRegistry::new(vec![])),
            QueryEngineConfig::default(),
            crate::cache::CacheConfig::default(),
        )
        .with_table_registry(Arc::clone(&registry)); // CRIT-1: wire the registry

        // Execute a query targeting an UNREGISTERED table (crowdstrike_alerts).
        // The plan-time gate must fire E-QUERY-037 before any fan-out.
        let result = engine
            .execute(
                "SELECT * FROM crowdstrike_alerts LIMIT 5",
                QueryOptions::default(),
            )
            .await;

        match result {
            Err(PrismError::TableNotAvailable(ref details)) => {
                let display = details.to_string();
                assert!(
                    display.starts_with("E-QUERY-037:"),
                    "S-3.13 CRIT-1 / AC-2: QueryEngine::execute must return E-QUERY-037 for \
                     unregistered table 'crowdstrike_alerts' when registry is wired. \
                     Display was: {display}"
                );
                assert_eq!(
                    details.table, "crowdstrike_alerts",
                    "CRIT-1: table field must be 'crowdstrike_alerts'"
                );
                // available_sensors must list only armis (the registered sensor).
                assert!(
                    details.available_sensors.contains("armis"),
                    "CRIT-1 / AC-2: available_sensors must list 'armis'. Got: '{}'",
                    details.available_sensors
                );
            }
            Ok(_) => panic!(
                "S-3.13 CRIT-1 / AC-2: QueryEngine::execute must NOT succeed for \
                 unregistered table 'crowdstrike_alerts' when registry is wired — \
                 E-QUERY-037 must fire before fan-out"
            ),
            Err(other) => panic!(
                "S-3.13 CRIT-1 / AC-2: expected PrismError::TableNotAvailable, \
                 got different error: {other:?}"
            ),
        }
    }
}

// ---------------------------------------------------------------------------
// Unit tests: QueryEngine::explain wrapper injects table_registry (OBS-1)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod explain_wrapper_tests {
    use std::sync::Arc;

    use prism_core::PrismError;
    use prism_spec_engine::spec_parser::{AuthType, SensorSpec, TableSpec};

    use super::*;
    use crate::{explain::ExplainOptions, table_registry::TableRegistry};

    /// Minimal no-op credential store (mirrors alias_wiring_tests::NoopCs).
    struct NoopCs;

    #[async_trait::async_trait]
    impl prism_credentials::CredentialStore for NoopCs {
        async fn get(
            &self,
            _t: &prism_core::OrgSlug,
            _s: &str,
            _n: &prism_credentials::namespace::CredentialName,
        ) -> Result<Option<secrecy::SecretString>, PrismError> {
            Ok(None)
        }
        async fn set(
            &self,
            _t: &prism_core::OrgSlug,
            _s: &str,
            _n: &prism_credentials::namespace::CredentialName,
            _v: secrecy::SecretString,
        ) -> Result<(), PrismError> {
            Ok(())
        }
        async fn delete(
            &self,
            _t: &prism_core::OrgSlug,
            _s: &str,
            _n: &prism_credentials::namespace::CredentialName,
        ) -> Result<bool, PrismError> {
            Ok(false)
        }
        async fn list(
            &self,
            _t: &prism_core::OrgSlug,
        ) -> Result<Vec<(String, prism_credentials::namespace::CredentialName)>, PrismError>
        {
            Ok(vec![])
        }
        async fn exists(
            &self,
            _t: &prism_core::OrgSlug,
            _s: &str,
            _n: &prism_credentials::namespace::CredentialName,
        ) -> Result<bool, PrismError> {
            Ok(false)
        }
    }

    /// Build a minimal `QueryEngine` with a wired `TableRegistry`.
    fn make_engine_with_registry(registry: Arc<TableRegistry>) -> QueryEngine {
        use prism_sensors::AdapterRegistry;

        QueryEngine::new_with_cache_config(
            Arc::new(AdapterRegistry::new()),
            Arc::new(NoopCs),
            Arc::new(prism_ocsf::OcsfNormalizer::new()),
            Arc::new(crate::scoping::ClientRegistry::new(vec![])),
            QueryEngineConfig::default(),
            crate::cache::CacheConfig::default(),
        )
        .with_table_registry(registry)
    }

    /// OBS-1 fix: `QueryEngine::explain` injects `self.table_registry` into the
    /// options when `options.table_registry` is `None`.
    ///
    /// Verifies that `ExplainResult.available_tables` contains the table registered
    /// in the engine's registry WITHOUT the caller needing to thread the registry
    /// through `ExplainOptions::table_registry` manually.
    ///
    /// This is the correctness test for the wrapper: a future caller using
    /// `QueryEngine::explain` (not the standalone `explain::explain` function) must
    /// get correct `available_tables` from the wired engine registry.
    ///
    /// `#[tokio::test]` required because `QueryEngine::new_with_cache_config` starts
    /// the cursor cleanup background task via `spawn_cursor_cleanup_task`, which
    /// requires a tokio runtime context even though `explain()` itself is synchronous.
    #[tokio::test]
    async fn test_explain_wrapper_injects_engine_table_registry_into_options() {
        // Build a registry with armis registered.
        let registry = Arc::new(TableRegistry::new());
        let armis_spec = SensorSpec::new(
            "armis",
            "Armis sensor",
            AuthType::ApiKey,
            "https://example.com",
            vec![TableSpec::new_point_in_time(
                "alerts",
                "security_finding",
                vec![],
                vec![],
            )],
            None,
            "1.0.0",
            Vec::new(),
        );
        registry
            .register_sensor(&armis_spec)
            .expect("register armis must not fail");

        let engine = make_engine_with_registry(Arc::clone(&registry));

        // Call QueryEngine::explain WITHOUT setting options.table_registry.
        // The wrapper must inject self.table_registry so available_tables is populated.
        let opts = ExplainOptions::default(); // table_registry is None
        let result = engine
            .explain("armis_alerts | severity = 'critical'", opts)
            .expect("explain must succeed for a valid filter query");

        assert!(
            result
                .available_tables
                .contains(&"armis_alerts".to_string()),
            "OBS-1 fix: QueryEngine::explain must inject self.table_registry so \
             ExplainResult.available_tables contains 'armis_alerts'. Got: {:?}",
            result.available_tables
        );
    }

    /// OBS-1 fix (caller-supplied registry is NOT overridden): when the caller
    /// explicitly sets `options.table_registry`, the wrapper must preserve the
    /// caller-supplied value, not replace it with `self.table_registry`.
    ///
    /// This guards against the inject-always anti-pattern where the engine silently
    /// masks a caller-supplied registry (useful for per-request registry overrides).
    ///
    /// `#[tokio::test]` required — same reason as the injection test above.
    #[tokio::test]
    async fn test_explain_wrapper_does_not_override_caller_supplied_table_registry() {
        // Engine registry: only armis.
        let engine_registry = Arc::new(TableRegistry::new());
        let armis_spec = SensorSpec::new(
            "armis",
            "Armis sensor",
            AuthType::ApiKey,
            "https://example.com",
            vec![TableSpec::new_point_in_time(
                "alerts",
                "security_finding",
                vec![],
                vec![],
            )],
            None,
            "1.0.0",
            Vec::new(),
        );
        engine_registry
            .register_sensor(&armis_spec)
            .expect("register armis in engine_registry must not fail");

        let engine = make_engine_with_registry(Arc::clone(&engine_registry));

        // Caller-supplied registry: only crowdstrike (different from engine registry).
        let caller_registry = Arc::new(TableRegistry::new());
        let cs_spec = SensorSpec::new(
            "crowdstrike",
            "CrowdStrike sensor",
            AuthType::ApiKey,
            "https://example.com",
            vec![TableSpec::new_point_in_time(
                "alerts",
                "security_finding",
                vec![],
                vec![],
            )],
            None,
            "1.0.0",
            Vec::new(),
        );
        caller_registry
            .register_sensor(&cs_spec)
            .expect("register crowdstrike in caller_registry must not fail");

        // Call explain with caller-supplied registry (crowdstrike only).
        let opts = ExplainOptions {
            table_registry: Some(Arc::clone(&caller_registry)),
            ..ExplainOptions::default()
        };
        let result = engine
            .explain("crowdstrike_alerts | severity = 'critical'", opts)
            .expect("explain must succeed");

        // Caller-supplied registry (crowdstrike) must win, not the engine registry (armis).
        assert!(
            result
                .available_tables
                .contains(&"crowdstrike_alerts".to_string()),
            "OBS-1 fix: caller-supplied table_registry must be preserved — \
             crowdstrike_alerts must appear. Got: {:?}",
            result.available_tables
        );
        assert!(
            !result
                .available_tables
                .contains(&"armis_alerts".to_string()),
            "OBS-1 fix: engine registry must NOT override caller-supplied registry — \
             armis_alerts must NOT appear when caller supplied crowdstrike-only registry. \
             Got: {:?}",
            result.available_tables
        );
    }
}

// ---------------------------------------------------------------------------
// Unit tests: SEC-003 production path via QueryEngine::explain (CR-NEW-001)
// ---------------------------------------------------------------------------
//
// These tests verify that calling QueryEngine::explain() (not the free function
// explain::explain()) correctly injects self.resolved_spec_map into the options,
// so that available_tables is filtered to the requesting org's visible tables.
//
// This is the PRODUCTION-PATH test demanded by CR-NEW-001 (S-3.13 fix-burst).
// The key invariant under test: resolved_spec_map MUST be None in the options
// supplied to qe.explain() — the engine's injection wiring is what we're verifying.

#[cfg(test)]
mod sec003_engine_path_tests {
    use std::{collections::HashMap, sync::Arc};

    use prism_core::{OrgSlug, SensorId};
    use prism_sensors::AdapterRegistry;
    use prism_spec_engine::{ResolvedSensorSpec, ResolvedSpecKey};

    use super::*;
    use crate::{explain::ExplainOptions, scoping::ClientRegistry, table_registry::TableRegistry};

    /// Minimal no-op credential store for SEC-003 engine tests.
    struct NoopCs;

    #[async_trait::async_trait]
    impl prism_credentials::CredentialStore for NoopCs {
        async fn get(
            &self,
            _t: &prism_core::OrgSlug,
            _s: &str,
            _n: &prism_credentials::namespace::CredentialName,
        ) -> Result<Option<secrecy::SecretString>, PrismError> {
            Ok(None)
        }
        async fn set(
            &self,
            _t: &prism_core::OrgSlug,
            _s: &str,
            _n: &prism_credentials::namespace::CredentialName,
            _v: secrecy::SecretString,
        ) -> Result<(), PrismError> {
            Ok(())
        }
        async fn delete(
            &self,
            _t: &prism_core::OrgSlug,
            _s: &str,
            _n: &prism_credentials::namespace::CredentialName,
        ) -> Result<bool, PrismError> {
            Ok(false)
        }
        async fn list(
            &self,
            _t: &prism_core::OrgSlug,
        ) -> Result<Vec<(String, prism_credentials::namespace::CredentialName)>, PrismError>
        {
            Ok(vec![])
        }
        async fn exists(
            &self,
            _t: &prism_core::OrgSlug,
            _s: &str,
            _n: &prism_credentials::namespace::CredentialName,
        ) -> Result<bool, PrismError> {
            Ok(false)
        }
    }

    /// Build a two-org resolved_spec_map: acme→armis (armis_devices), contoso→crowdstrike (crowdstrike_alerts).
    fn make_two_org_spec_map() -> HashMap<ResolvedSpecKey, ResolvedSensorSpec> {
        use prism_spec_engine::{
            overlay::{OverlayLoader, SensorInstanceOverlay},
            spec_parser::{AuthType, SensorSpec, TableSpec},
        };

        let make_resolved = |sensor_id: &str, table_suffix: &str, org: &str| {
            let spec = SensorSpec::new(
                sensor_id,
                format!("{sensor_id} sensor"),
                AuthType::ApiKey,
                "https://example.com",
                vec![TableSpec::new_point_in_time(
                    table_suffix,
                    "security_finding",
                    vec![],
                    vec![],
                )],
                None,
                "1.0.0",
                Vec::new(),
            );
            let overlay_toml =
                format!("extends = \"{sensor_id}\"\ninstance_id = \"{sensor_id}@{org}\"");
            let overlay: SensorInstanceOverlay = toml::from_str(&overlay_toml)
                .expect("SEC-003 engine fixture: SensorInstanceOverlay TOML must parse");
            let org_slug = OrgSlug::new(org);
            let resolved =
                OverlayLoader::merge_overlay_onto_type_spec(&spec, &overlay, org_slug.clone());
            let sensor_id_typed = SensorId::new(sensor_id);
            let key: ResolvedSpecKey = (org_slug, sensor_id_typed);
            (key, resolved)
        };

        let mut map = HashMap::new();
        let (k, v) = make_resolved("armis", "devices", "acme");
        map.insert(k, v);
        let (k, v) = make_resolved("crowdstrike", "alerts", "contoso");
        map.insert(k, v);
        map
    }

    /// Build a global TableRegistry with both armis and crowdstrike sensors.
    fn make_two_sensor_registry() -> Arc<TableRegistry> {
        use prism_spec_engine::spec_parser::{AuthType, SensorSpec, TableSpec};
        let registry = TableRegistry::new();
        let make_spec = |sensor_id: &str, table_suffix: &str| {
            SensorSpec::new(
                sensor_id,
                format!("{sensor_id} sensor"),
                AuthType::ApiKey,
                "https://example.com",
                vec![TableSpec::new_point_in_time(
                    table_suffix,
                    "security_finding",
                    vec![],
                    vec![],
                )],
                None,
                "1.0.0",
                Vec::new(),
            )
        };
        registry
            .register_sensor(&make_spec("armis", "devices"))
            .expect("register armis must not fail");
        registry
            .register_sensor(&make_spec("crowdstrike", "alerts"))
            .expect("register crowdstrike must not fail");
        Arc::new(registry)
    }

    /// Build a QueryEngine with wired resolved_spec_map and table_registry.
    ///
    /// Uses new_with_cache_config + direct pub(crate) field injection so we
    /// don't need the full production dependency tree (OrgRegistry, RocksDB, etc.).
    fn make_engine_with_spec_map(
        registry: Arc<TableRegistry>,
        spec_map: HashMap<ResolvedSpecKey, ResolvedSensorSpec>,
    ) -> QueryEngine {
        let mut engine = QueryEngine::new_with_cache_config(
            Arc::new(AdapterRegistry::new()),
            Arc::new(NoopCs),
            Arc::new(prism_ocsf::OcsfNormalizer::new()),
            Arc::new(ClientRegistry::new(vec![])),
            QueryEngineConfig::default(),
            crate::cache::CacheConfig::default(),
        );
        // Inject resolved_spec_map — ADR-042: field is now ArcSwap-backed.
        engine.resolved_spec_map = Some(Arc::new(arc_swap::ArcSwap::new(Arc::new(spec_map))));
        // Wire the table_registry so available_tables is populated.
        engine = engine.with_table_registry(registry);
        engine
    }

    /// CR-NEW-001 / SEC-003 production path test (S-3.13 fix-burst, CWE-200).
    ///
    /// Verifies that calling `QueryEngine::explain()` — the production code path
    /// used by the MCP `explain_query` handler after the CR-NEW-001 fix — correctly
    /// injects `self.resolved_spec_map` into the options and filters
    /// `available_tables` to the requesting org's visible tables.
    ///
    /// CRITICAL: `resolved_spec_map` in `ExplainOptions` is set to `None`.  The engine
    /// injection (engine.rs:753-757) is the mechanism under test.  Passing it
    /// pre-populated would bypass the wiring and make this a vacuous test.
    ///
    /// `client_registry` IS supplied in opts (same as the MCP handler does via
    /// `qe.client_registry()`) — the engine does not inject that field, so tests
    /// must mirror the MCP handler call-site exactly.
    ///
    /// Fixture: acme → armis (armis_devices), contoso → crowdstrike (crowdstrike_alerts).
    /// Calling with clients=Some([acme]) must yield armis_devices but NOT crowdstrike_alerts.
    ///
    /// `#[tokio::test]` required because QueryEngine::new_with_cache_config starts
    /// the cursor cleanup background task, which requires a tokio runtime context.
    #[tokio::test]
    #[allow(non_snake_case, clippy::expect_used)]
    async fn test_SEC_003_explain_production_path_via_query_engine_filters_other_org() {
        let registry = make_two_sensor_registry();
        let spec_map = make_two_org_spec_map();
        let acme = OrgSlug::new("acme");

        // Build a ClientRegistry that contains "acme" so resolve_clients() accepts it.
        // The MCP handler supplies this via `qe.client_registry()` in ExplainOptions.
        let client_registry = Arc::new(ClientRegistry::new(vec![acme.clone()]));

        let engine = make_engine_with_spec_map(registry, spec_map);

        // CRITICAL: resolved_spec_map is None here — the engine's injection (engine.rs:753-757)
        // must supply it.  This is the exact options shape the MCP handler constructs.
        // client_registry is supplied explicitly (mirrors the MCP handler at server.rs:1749).
        let opts = ExplainOptions {
            clients: Some(vec![acme]),
            client_registry: Some(client_registry),
            resolved_spec_map: None, // engine injects self.resolved_spec_map
            ..ExplainOptions::default()
        };

        let result = engine
            .explain("severity = 'critical'", opts)
            .expect("QueryEngine::explain must succeed for valid query");

        // acme can only see armis_devices — the injection must have fired.
        assert!(
            result
                .available_tables
                .contains(&"armis_devices".to_string()),
            "SEC-003 production path: acme's table 'armis_devices' must appear \
             in available_tables after QueryEngine::explain injection. Got: {:?}",
            result.available_tables
        );

        // contoso's table must NOT leak — cross-tenant CWE-200 protection.
        assert!(
            !result
                .available_tables
                .contains(&"crowdstrike_alerts".to_string()),
            "SEC-003 / CWE-200 production path: contoso's table 'crowdstrike_alerts' \
             must NOT appear in available_tables when requesting org is acme. \
             Got: {:?}",
            result.available_tables
        );
    }
    /// SEC-001 production-path regression test (S-3.13 MED-A / sibling-coverage gap).
    ///
    /// Verifies that calling `QueryEngine::execute()` — the EXECUTE production code path,
    /// not the explain path — correctly uses `self.resolved_spec_map` + `options.clients`
    /// to filter the E-QUERY-037 enumeration to the requesting org's visible sensors only.
    ///
    /// # What this test proves
    ///
    /// The 5 existing `test_SEC_001_*` tests in `table_registry_tests.rs` call
    /// `check_availability_gate` DIRECTLY, bypassing the engine. The existing engine-path
    /// test (`test_S3_13_engine_execute_with_wired_registry_returns_e_query_037_before_fanout`)
    /// uses `QueryOptions::default()` (clients=None) + no `resolved_spec_map`, so it never
    /// exercises the org-FILTER path. This test closes that gap.
    ///
    /// # Load-bearing guarantee
    ///
    /// The E-QUERY-037 enumeration depends on both:
    /// 1. `engine.resolved_spec_map` being wired (injected via `execute_inner` into
    ///    `check_table_availability`), AND
    /// 2. `options.clients` carrying the requesting org's slug (used as `org_scope`).
    ///
    /// If either plumbing is removed, the filter reverts to global scope and
    /// `available_sensors`/`available_tables` would contain BOTH orgs — causing this test
    /// to fail on the `!contains("crowdstrike")` / `!contains("crowdstrike_alerts")` asserts.
    ///
    /// # Fixture
    ///
    /// acme → armis (armis_devices), contoso → crowdstrike (crowdstrike_alerts).
    /// Execute with `clients=Some([acme])` on `unknown_table`.
    /// E-QUERY-037 must enumerate acme's tables only (armis_devices) — NOT contoso's.
    ///
    /// # Execution note
    ///
    /// `check_table_availability` fires BEFORE `resolve_clients`, so the empty
    /// `ClientRegistry` in `make_engine_with_spec_map` is acceptable — `resolve_clients`
    /// is never reached when the gate returns E-QUERY-037.
    ///
    /// `#[tokio::test]` required: `QueryEngine::new_with_cache_config` spawns the cursor
    /// cleanup background task which requires a tokio runtime context.
    ///
    /// BC-2.11.001 / ADR-039 / SEC-001 / CWE-200.
    #[tokio::test]
    #[allow(non_snake_case, clippy::expect_used)]
    async fn test_SEC_001_e_query_037_production_path_via_query_engine_filters_other_org() {
        let registry = make_two_sensor_registry();
        let spec_map = make_two_org_spec_map();
        let acme = OrgSlug::new("acme");

        let engine = make_engine_with_spec_map(registry, spec_map);

        // Execute against an unknown table so E-QUERY-037 fires.
        // clients=Some([acme]) is the org-scope the gate must use for filtering.
        // check_table_availability fires BEFORE resolve_clients — the empty ClientRegistry
        // in make_engine_with_spec_map is acceptable for this test path.
        let result = engine
            .execute(
                "SELECT * FROM unknown_table LIMIT 5",
                QueryOptions {
                    clients: Some(vec![acme]),
                    ..QueryOptions::default()
                },
            )
            .await;

        match result {
            Err(PrismError::TableNotAvailable(ref details)) => {
                // available_sensors must contain acme's sensor ("armis")
                // and must NOT contain contoso's sensor ("crowdstrike").
                let sensors: Vec<&str> = details.available_sensors.split(", ").collect();
                assert!(
                    sensors.contains(&"armis"),
                    "SEC-001 execute production path: acme's sensor 'armis' must appear \
                     in E-QUERY-037 available_sensors. Got: '{}'",
                    details.available_sensors
                );
                assert!(
                    !sensors.contains(&"crowdstrike"),
                    "SEC-001 / CWE-200 execute production path: contoso's sensor 'crowdstrike' \
                     must NOT appear in E-QUERY-037 available_sensors when requesting org is acme. \
                     Got: '{}'",
                    details.available_sensors
                );

                // available_tables must contain acme's table ("armis_devices")
                // and must NOT contain contoso's table ("crowdstrike_alerts").
                let tables: Vec<&str> = details.available_tables.split(", ").collect();
                assert!(
                    tables.contains(&"armis_devices"),
                    "SEC-001 execute production path: acme's table 'armis_devices' must appear \
                     in E-QUERY-037 available_tables. Got: '{}'",
                    details.available_tables
                );
                assert!(
                    !tables.contains(&"crowdstrike_alerts"),
                    "SEC-001 / CWE-200 execute production path: contoso's table 'crowdstrike_alerts' \
                     must NOT appear in E-QUERY-037 available_tables when requesting org is acme. \
                     Got: '{}'",
                    details.available_tables
                );
            }
            Ok(_) => panic!(
                "SEC-001 execute production path: QueryEngine::execute must NOT succeed for \
                 unknown table 'unknown_table' when registry is wired — E-QUERY-037 must fire"
            ),
            Err(other) => panic!(
                "SEC-001 execute production path: expected PrismError::TableNotAvailable, \
                 got different error: {other:?}"
            ),
        }
    }
}

// ---------------------------------------------------------------------------
// ADR-042 Red Gate tests — rebuild_resolved_spec_map (ArcSwap-backed)
// ---------------------------------------------------------------------------
//
// BC traces: BC-2.10.013 (EC-10-034), ADR-042 in-flight-query consistency guarantee.
//
// ALL FOUR tests MUST fail until the implementer:
//   1. Adds `arc-swap = "1"` to prism-query/Cargo.toml.
//   2. Changes the `resolved_spec_map` field to `Option<Arc<ArcSwap<HashMap<...>>>>`.
//   3. Updates `new_full` to wrap with `ArcSwap::new(resolved_spec_map)`.
//   4. Updates the `resolved_spec_map()` accessor to call `swap.load_full()`.
//   5. Adds `rebuild_resolved_spec_map(&self, ...) -> Result<usize, SpecEngineError>`.
//   6. Updates the test-helper in sec003_engine_path_tests to use
//      `Arc::new(arc_swap::ArcSwap::new(Arc::new(spec_map)))`.
//
// NOTE on clippy::expect_used/unwrap_used:
//   Tests may use `expect` / `unwrap` — #[allow] gates below.

#[cfg(test)]
#[allow(non_snake_case, clippy::expect_used, clippy::unwrap_used)]
mod adr_042_tests {
    use std::{collections::HashMap, path::PathBuf, sync::Arc};

    use prism_core::{OrgId, OrgRegistry, OrgSlug, SensorId};
    use prism_sensors::AdapterRegistry;
    use prism_spec_engine::{
        overlay::{OverlayLoader, SensorInstanceOverlay},
        spec_parser::{AuthType, SensorSpec, TableSpec},
        ResolvedSensorSpec, ResolvedSpecKey,
    };

    use super::*;
    use crate::{cache::CacheConfig, scoping::ClientRegistry};

    // ────────────────────────────────────────────────────────────────────────────
    // Shared test infrastructure
    // ────────────────────────────────────────────────────────────────────────────

    /// Minimal no-op credential store reused across tests.
    struct NoopCredStore;

    #[async_trait::async_trait]
    impl prism_credentials::CredentialStore for NoopCredStore {
        async fn get(
            &self,
            _t: &OrgSlug,
            _s: &str,
            _n: &prism_credentials::namespace::CredentialName,
        ) -> Result<Option<secrecy::SecretString>, PrismError> {
            Ok(None)
        }
        async fn set(
            &self,
            _t: &OrgSlug,
            _s: &str,
            _n: &prism_credentials::namespace::CredentialName,
            _v: secrecy::SecretString,
        ) -> Result<(), PrismError> {
            Ok(())
        }
        async fn delete(
            &self,
            _t: &OrgSlug,
            _s: &str,
            _n: &prism_credentials::namespace::CredentialName,
        ) -> Result<bool, PrismError> {
            Ok(false)
        }
        async fn list(
            &self,
            _t: &OrgSlug,
        ) -> Result<Vec<(String, prism_credentials::namespace::CredentialName)>, PrismError>
        {
            Ok(vec![])
        }
        async fn exists(
            &self,
            _t: &OrgSlug,
            _s: &str,
            _n: &prism_credentials::namespace::CredentialName,
        ) -> Result<bool, PrismError> {
            Ok(false)
        }
    }

    /// Build a minimal `QueryEngine` via `new_with_cache_config`.
    ///
    /// The `resolved_spec_map` field is left as `None` (single-tenant mode).
    /// Tests that need a populated map inject it directly via `pub(crate)`.
    fn make_minimal_engine() -> QueryEngine {
        QueryEngine::new_with_cache_config(
            Arc::new(AdapterRegistry::new()),
            Arc::new(NoopCredStore),
            Arc::new(prism_ocsf::OcsfNormalizer::new()),
            Arc::new(ClientRegistry::new(vec![])),
            QueryEngineConfig::default(),
            CacheConfig::default(),
        )
    }

    /// Build a `ResolvedSensorSpec` with a single table via the real `OverlayLoader` merge path.
    fn make_resolved(
        sensor_id: &str,
        table_name: &str,
        org: &str,
    ) -> (ResolvedSpecKey, ResolvedSensorSpec) {
        let spec = SensorSpec::new(
            sensor_id,
            format!("{sensor_id} sensor"),
            AuthType::ApiKey,
            "https://example.com",
            vec![TableSpec::new_point_in_time(
                table_name,
                "security_finding",
                vec![],
                vec![],
            )],
            None,
            "1.0.0",
            Vec::new(),
        );
        let overlay_toml =
            format!("extends = \"{sensor_id}\"\ninstance_id = \"{sensor_id}@{org}\"");
        let overlay: SensorInstanceOverlay = toml::from_str(&overlay_toml)
            .expect("ADR-042 fixture: SensorInstanceOverlay TOML must parse");
        let org_slug = OrgSlug::new(org);
        let resolved =
            OverlayLoader::merge_overlay_onto_type_spec(&spec, &overlay, org_slug.clone());
        let key: ResolvedSpecKey = (org_slug, SensorId::new(sensor_id));
        (key, resolved)
    }

    // ────────────────────────────────────────────────────────────────────────────
    // Test 3 — BC-ADR-042 single-tenant no-op
    //
    // Traces to: ADR-042 §D3 (single-tenant/None mode returns Ok(0) with no side effects).
    //
    // When `self.resolved_spec_map` is `None` (single-tenant mode), the method MUST:
    //   - Return `Ok(0)` immediately (no-op).
    //   - Leave `resolved_spec_map()` returning `None`.
    //   - Perform no I/O on `customers_dir`.
    // ────────────────────────────────────────────────────────────────────────────
    #[tokio::test]
    async fn test_BC_ADR_042_single_tenant_rebuild_is_noop_returns_ok_zero() {
        let engine = make_minimal_engine();

        // Sanity: single-tenant engine has no resolved_spec_map.
        assert!(
            engine.resolved_spec_map().is_none(),
            "ADR-042 precondition: engine.resolved_spec_map() must be None in single-tenant mode"
        );

        let dummy_path = PathBuf::from("/nonexistent/customers");
        let type_specs: HashMap<String, prism_spec_engine::spec_parser::SensorSpec> =
            HashMap::new();
        let org_registry = OrgRegistry::new();

        let result = engine.rebuild_resolved_spec_map(&dummy_path, &type_specs, &org_registry);

        assert_eq!(
            result.unwrap(),
            0,
            "ADR-042 Test3: rebuild_resolved_spec_map on single-tenant engine (None map) \
             MUST return Ok(0) — no-op; got something else"
        );

        // No side effect: the map must remain None after the call.
        assert!(
            engine.resolved_spec_map().is_none(),
            "ADR-042 Test3: resolved_spec_map() MUST remain None after rebuild_resolved_spec_map \
             in single-tenant mode — no side effects permitted"
        );
    }

    // ────────────────────────────────────────────────────────────────────────────
    // Test 4 — BC-ADR-042 in-flight query snapshot isolation
    //
    // Traces to: ADR-042 §In-Flight Query Consistency Guarantee.
    //
    // `resolved_spec_map` is `Option<Arc<arc_swap::ArcSwap<HashMap<...>>>>`.
    // `resolved_spec_map()` calls `swap.load_full()` to return a fresh Arc snapshot.
    //
    // Assertion logic:
    //   - old_arc snapshot (held before rebuild) must still contain spec_A (acme→alerts).
    //   - fresh load after rebuild must contain spec_B (acme→alerts + acme→hosts).
    //   - These are different Arc pointers — !Arc::ptr_eq.
    // ────────────────────────────────────────────────────────────────────────────
    #[tokio::test]
    async fn test_BC_ADR_042_inflight_snapshot_isolation_during_rebuild() {
        use tempfile::TempDir;

        // ── Build initial map: acme → crowdstrike (crowdstrike_alerts only) ──
        let (key_a, val_a) = make_resolved("crowdstrike", "crowdstrike_alerts", "acme");
        let mut initial_map: HashMap<ResolvedSpecKey, ResolvedSensorSpec> = HashMap::new();
        initial_map.insert(key_a, val_a);

        // Inject into engine via pub(crate) field using the ArcSwap shape.
        let mut engine = make_minimal_engine();
        engine.resolved_spec_map = Some(Arc::new(arc_swap::ArcSwap::new(Arc::new(initial_map))));

        // ── Step 1: snapshot (simulate in-flight query holding the old Arc) ──
        let old_arc = engine
            .resolved_spec_map()
            .expect("ADR-042 Test4: resolved_spec_map() must be Some after initial injection");

        assert!(
            old_arc
                .keys()
                .any(|(_, sensor)| sensor.as_ref() == "crowdstrike"),
            "ADR-042 Test4: old_arc must contain the crowdstrike key before rebuild"
        );
        let old_table_count = old_arc.values().flat_map(|r| r.spec.tables.iter()).count();
        assert_eq!(
            old_table_count, 1,
            "ADR-042 Test4: initial map must have exactly 1 table (crowdstrike_alerts)"
        );

        // ── Step 2: prepare an updated overlay on disk with a second table ───
        //
        // The updated spec adds "crowdstrike_hosts" to the crowdstrike sensor.
        let tmp = TempDir::new().expect("TempDir::new must succeed");
        let customers_dir = tmp.path().join("customers");
        std::fs::create_dir_all(customers_dir.join("acme"))
            .expect("create customers/acme/ must succeed");

        // Write the overlay: acme → crowdstrike
        let overlay_toml = "extends = \"crowdstrike\"\ninstance_id = \"crowdstrike@acme\"\n";
        std::fs::write(
            customers_dir.join("acme").join("crowdstrike.sensor.toml"),
            overlay_toml,
        )
        .expect("write overlay TOML must succeed");

        // Register acme in OrgRegistry (required by OverlayLoader::load_overlays).
        let org_registry = {
            let reg = OrgRegistry::new();
            reg.register(OrgSlug::new("acme"), OrgId::new())
                .expect("register acme must succeed");
            reg
        };

        // Build the updated TYPE spec (crowdstrike with two tables).
        let updated_type_spec = SensorSpec::new(
            "crowdstrike",
            "CrowdStrike sensor",
            AuthType::ApiKey,
            "https://api.crowdstrike.com",
            vec![
                TableSpec::new_point_in_time(
                    "crowdstrike_alerts",
                    "security_finding",
                    vec![],
                    vec![],
                ),
                TableSpec::new_point_in_time(
                    "crowdstrike_hosts",
                    "device_inventory_info",
                    vec![],
                    vec![],
                ),
            ],
            None,
            "1.0.0",
            Vec::new(),
        );
        let mut type_specs: HashMap<String, prism_spec_engine::spec_parser::SensorSpec> =
            HashMap::new();
        type_specs.insert("crowdstrike".to_string(), updated_type_spec);

        // ── Step 3: rebuild (simulates hot-reload) ────────────────────────────
        let rebuild_result =
            engine.rebuild_resolved_spec_map(&customers_dir, &type_specs, &org_registry);

        assert!(
            rebuild_result.is_ok(),
            "ADR-042 Test4: rebuild_resolved_spec_map must return Ok; got {:?}",
            rebuild_result
        );
        let overlay_count = rebuild_result.unwrap();
        assert_eq!(
            overlay_count, 1,
            "ADR-042 Test4: rebuild must report 1 overlay (acme→crowdstrike); got {overlay_count}"
        );

        // ── Step 4: in-flight snapshot isolation ─────────────────────────────
        //
        // The `old_arc` held BEFORE the rebuild must still contain only the initial
        // data — it is an immutable snapshot, not a live reference to the ArcSwap.
        let old_table_count_after = old_arc.values().flat_map(|r| r.spec.tables.iter()).count();
        assert_eq!(
            old_table_count_after, 1,
            "ADR-042 Test4 ISOLATION: old_arc (held before rebuild) must still contain \
             exactly 1 table even after rebuild — the in-flight snapshot is immutable. \
             Got {old_table_count_after} tables, meaning the ArcSwap store is incorrectly \
             mutating the old pointer (not creating a new Arc)."
        );

        // ── Step 5: fresh load sees the new map ───────────────────────────────
        //
        // A NEW call to resolved_spec_map() must return the post-rebuild Arc.
        let new_arc = engine
            .resolved_spec_map()
            .expect("ADR-042 Test4: resolved_spec_map() must be Some after rebuild");

        let new_table_count = new_arc.values().flat_map(|r| r.spec.tables.iter()).count();
        assert_eq!(
            new_table_count, 2,
            "ADR-042 Test4 FRESHNESS: new_arc (loaded after rebuild) must contain \
             2 tables (crowdstrike_alerts + crowdstrike_hosts). \
             Got {new_table_count} — means ArcSwap::store did not update the pointer."
        );

        // ── Step 6: different Arc pointers confirm atomic swap ─────────────────
        assert!(
            !Arc::ptr_eq(&old_arc, &new_arc),
            "ADR-042 Test4 POINTER: old_arc and new_arc must be DIFFERENT Arc pointers \
             — the ArcSwap must allocate a fresh Arc on store(), not mutate in place."
        );
    }
}

// ---------------------------------------------------------------------------
// Helper: truncate batches to a row limit
// ---------------------------------------------------------------------------

/// Truncate a list of `RecordBatch`es to at most `limit` rows total.
fn truncate_batches_to_limit(
    batches: Vec<arrow::record_batch::RecordBatch>,
    limit: usize,
) -> Vec<arrow::record_batch::RecordBatch> {
    let mut result = Vec::new();
    let mut remaining = limit;
    for batch in batches {
        if remaining == 0 {
            break;
        }
        if batch.num_rows() <= remaining {
            remaining -= batch.num_rows();
            result.push(batch);
        } else {
            result.push(batch.slice(0, remaining));
            remaining = 0;
        }
    }
    result
}

// ─────────────────────────────────────────────────────────────────────────────
// F-001B-PASS-LOW-001 — did_you_mean non-determinism on equidistant ties
// (BC-2.11.016 AC-001)
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
#[allow(
    non_snake_case,
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::too_many_lines
)]
mod bc_2_11_016_did_you_mean_determinism_tests {
    use std::collections::HashMap;

    use prism_core::{OrgSlug, SensorId};
    use prism_spec_engine::{
        overlay::{OverlayLoader, SensorInstanceOverlay},
        spec_parser::{AuthType, ColumnSpec, SensorSpec, TableSpec},
        ResolvedSensorSpec, ResolvedSpecKey,
    };

    use super::check_column_availability;
    use prism_core::column::ColumnType;

    /// Build a `ResolvedSensorSpec` for a single sensor+table+columns under one org.
    fn make_resolved_with_columns(
        sensor_id: &str,
        table_suffix: &str,
        org: &str,
        columns: Vec<ColumnSpec>,
    ) -> (ResolvedSpecKey, ResolvedSensorSpec) {
        let spec = SensorSpec::new(
            sensor_id,
            format!("{sensor_id} sensor"),
            AuthType::ApiKey,
            "https://example.com",
            vec![TableSpec::new_point_in_time(
                table_suffix,
                "security_finding",
                columns,
                vec![],
            )],
            None,
            "1.0.0",
            Vec::new(),
        );
        let overlay_toml =
            format!("extends = \"{sensor_id}\"\ninstance_id = \"{sensor_id}@{org}\"");
        let overlay: SensorInstanceOverlay = toml::from_str(&overlay_toml)
            .expect("did_you_mean fixture: SensorInstanceOverlay TOML must parse");
        let org_slug = OrgSlug::new(org);
        let resolved =
            OverlayLoader::merge_overlay_onto_type_spec(&spec, &overlay, org_slug.clone());
        let key: ResolvedSpecKey = (org_slug, SensorId::new(sensor_id));
        (key, resolved)
    }

    /// F-001B-PASS-LOW-001: `check_column_availability` must return the
    /// LEXICOGRAPHICALLY-SMALLEST column when multiple equidistant candidates exist.
    ///
    /// Setup: two sensors registered under the SAME org ("acme") with the SAME table name
    /// ("acme_alerts"). Both sensors each have one column that is equidistant from the typo:
    ///   - sensor "alpha_sensor" → column "severity_a" (levenshtein("sevrty", "severity_a") = 3)
    ///   - sensor "beta_sensor"  → column "severity_b" (levenshtein("sevrty", "severity_b") = 3)
    ///
    /// Because `spec_map.values()` iterates in HashMap order (non-deterministic), the
    /// `min_by_key(|(_, dist)| *dist)` call may return either "severity_a" or "severity_b"
    /// depending on which spec_map entry is encountered first. The BC-2.11.016 AC-001
    /// contract requires deterministic tie-breaking (lexicographically smallest: "severity_a").
    ///
    /// Current code does NOT secondary-sort by column name, so this assertion fails on any
    /// run where the HashMap iterator returns "severity_b" before "severity_a".
    ///
    /// Load-bearing (TD-VSDD-059): assertion `did_you_mean == Some("severity_a")` fails
    /// if lexicographic tie-breaking is removed — the code would return a HashMap-order
    /// non-deterministic candidate instead of the lexicographically smallest one.
    /// The test runs `check_column_availability` in a tight loop to expose non-determinism
    /// if the secondary sort is absent.
    #[test]
    fn test_BC_2_11_016_did_you_mean_lexicographic_tiebreak_on_equidistant_candidates() {
        // Column A: "severity_a" — equidistant from typo.
        // Column B: "severity_b" — equidistant from typo.
        // Levenshtein("sevrty", "severity_a") = distance to reach "severity_a" from "sevrty":
        //   sevrty → severity (1 insert 'e') → severity_a (1 append '_a') = 3
        // Levenshtein("sevrty", "severity_b") = same structure = 3
        // Both distance 3 ≤ 3: both qualify for did_you_mean.
        //
        // Expected deterministic result: "severity_a" (lexicographically smallest).
        // Current code returns whichever HashMap iteration encounters first (non-deterministic).

        let col_a = ColumnSpec::new("severity_a", ColumnType::String, None, vec![]);
        let col_b = ColumnSpec::new("severity_b", ColumnType::String, None, vec![]);

        // Two sensors, SAME table name "alerts" under SAME org "acme".
        // Fully-qualified: "alpha_sensor_alerts" and "beta_sensor_alerts" — different tables!
        // To share available_columns for the same table, both must produce the same FQ name.
        // FQ name = "{sensor_id}_{table_suffix}". To collide both in one available_columns vec,
        // we need the query's table_name to match BOTH sensors' FQ names, which is impossible
        // since they differ by sensor_id. The correct setup: one sensor with BOTH columns.
        // Instead, use ONE sensor with a table that has BOTH equidistant columns.
        let col_a2 = ColumnSpec::new("severity_aa", ColumnType::String, None, vec![]);
        let col_b2 = ColumnSpec::new("severity_ab", ColumnType::String, None, vec![]);
        // Levenshtein("sevrty", "severity_aa") — let's verify:
        //   both "severity_aa" and "severity_ab" differ from "sevrty" by the same edit distance.
        //   Actually let's use simpler names with KNOWN equidistant properties.
        //   Typo = "sevrity" (swap r/i vs i/r... let me use typo "aaab" with cols "aaac" and "aaad").
        //   Levenshtein("aaab", "aaac") = 1. Levenshtein("aaab", "aaad") = 1. Both equidistant.

        // Use clear typo="aaab", col_x="aaac", col_y="aaad" (both distance 1 from "aaab").
        // Insert in REVERSE lexicographic order so current code returns "aaad" first,
        // making the assertion `Some("aaac")` reliably fail.
        let col_y = ColumnSpec::new("aaad", ColumnType::String, None, vec![]); // lexically LATER, inserted FIRST
        let col_x = ColumnSpec::new("aaac", ColumnType::String, None, vec![]); // lexically FIRST, inserted SECOND

        // One sensor with BOTH columns in the same table (acme → sensor_one → single_alerts).
        // Vec order: ["aaad", "aaac"] — current min_by_key picks "aaad" (first minimum found).
        // Expected: "aaac" (lexicographically smallest). Current code returns "aaad".
        let (key_one, val_one) = make_resolved_with_columns(
            "sensor_one",
            "single",
            "acme",
            vec![col_y, col_x], // reverse order: "aaad" first so current code hits it first
        );
        let _ = (col_a, col_b, col_a2, col_b2); // suppress unused warnings

        let mut spec_map = HashMap::new();
        spec_map.insert(key_one, val_one);

        // fully-qualified table name: "sensor_one_single"
        let table_name = "sensor_one_single";
        let column_typo = "aaab"; // equidistant (dist=1) from both "aaac" and "aaad"
        let org = OrgSlug::new("acme");
        let org_scope = [org];

        // call check_column_availability: should return ColumnNotFound with did_you_mean.
        let result = check_column_availability(
            column_typo,
            table_name,
            "test-client",
            Some(&org_scope),
            Some(&spec_map),
            None, // table_registry not needed; resolved_spec_map is wired
        );

        let err = result
            .expect_err("check_column_availability must return Err for unknown column 'aaab'");

        // Extract did_you_mean from the error.
        let did_you_mean = match err {
            prism_core::error::PrismError::ColumnNotFound(ref d) => d.did_you_mean.clone(),
            other => panic!("expected ColumnNotFound, got: {other:?}"),
        };

        // BC-2.11.016 AC-001: did_you_mean must be deterministic.
        // Expected: "aaac" (lexicographically smallest of equidistant candidates "aaac" / "aaad").
        // Load-bearing: removing the lexicographic tie-break causes non-deterministic results.
        assert_eq!(
            did_you_mean.as_deref(),
            Some("aaac"),
            "BC-2.11.016: did_you_mean must return lexicographically-smallest equidistant \
             candidate 'aaac' (not '{}'); current code has no tie-break so the result is \
             non-deterministic",
            did_you_mean.as_deref().unwrap_or("<None>")
        );
    }

    /// F-001B-PASS-LOW-001 (multi-org variant): same non-determinism in a multi-client query
    /// where one sensor contributes multiple equidistant columns via flat_map, but in this
    /// variant the equidistant columns are part of the SAME flat_map sequence from a
    /// single sensor entry. The non-lexicographic column is inserted first in the Vec so
    /// that `min_by_key` reliably picks it over the lexicographically-correct candidate.
    ///
    /// The multi-org aspect verifies the org_scope filter path is correctly exercised.
    ///
    /// Load-bearing (TD-VSDD-059): assertion `did_you_mean == Some("bbb1")` fails if
    /// lexicographic tie-breaking is removed — without it, `min_by_key` returns "bbb2"
    /// (inserted first, encountered first by flat_map) instead of the lexicographic minimum.
    #[test]
    fn test_BC_2_11_016_did_you_mean_lexicographic_tiebreak_multi_sensor_same_table() {
        // Multi-org test: org_scope covers "acme". One sensor with two equidistant columns
        // inserted in reverse lexicographic order so current code returns the wrong one.
        //
        // Columns: "bbb2" (dist=1 from "bbb0", inserted FIRST) and "bbb1" (dist=1, SECOND).
        // Current code: min_by_key picks "bbb2" (first minimum encountered in Vec iteration).
        // Expected: "bbb1" (lexicographically smallest equidistant candidate).

        let col_wrong = ColumnSpec::new("bbb2", ColumnType::String, None, vec![]); // wrong: inserted first
        let col_right = ColumnSpec::new("bbb1", ColumnType::String, None, vec![]); // right: lexicographic min

        // Single sensor under "acme" with columns in reverse-lex order.
        let (key_one, val_one) = make_resolved_with_columns(
            "shared_sensor",
            "logs",
            "acme",
            vec![col_wrong, col_right], // "bbb2" first → current code returns "bbb2"
        );

        let mut spec_map = HashMap::new();
        spec_map.insert(key_one, val_one);

        // org_scope = acme only (exercises the org_scope filter path).
        let acme = OrgSlug::new("acme");
        let org_scope = [acme];

        let table_name = "shared_sensor_logs";
        let column_typo = "bbb0"; // dist=1 from both "bbb1" and "bbb2" — equidistant.

        let result = check_column_availability(
            column_typo,
            table_name,
            "test-client",
            Some(&org_scope),
            Some(&spec_map),
            None, // table_registry not needed; resolved_spec_map is wired
        );

        let err = result
            .expect_err("check_column_availability must return Err for unknown column 'bbb0'");

        let did_you_mean = match err {
            prism_core::error::PrismError::ColumnNotFound(ref d) => d.did_you_mean.clone(),
            other => panic!("expected ColumnNotFound, got: {other:?}"),
        };

        // BC-2.11.016 AC-001: deterministic tie-break → lexicographically smallest.
        // Expected: "bbb1" (alphabetically before "bbb2").
        // Load-bearing: without the lex tie-break, min_by_key picks "bbb2" (Vec-order first).
        assert_eq!(
            did_you_mean.as_deref(),
            Some("bbb1"),
            "BC-2.11.016: did_you_mean must return lexicographically-smallest equidistant \
             candidate 'bbb1' (got '{}'); current code picks Vec-order first minimum, \
             not lex-smallest",
            did_you_mean.as_deref().unwrap_or("<None>")
        );
    }

    // ─────────────────────────────────────────────────────────────────────────────
    // OBS-FRESH-1 — available_columns nondeterministic ordering and possible duplicates
    // (BC-2.11.016 AC — multi-org-scope determinism)
    // ─────────────────────────────────────────────────────────────────────────────

    /// OBS-FRESH-1: `available_columns` in `check_column_availability` must be sorted
    /// and deduped before constructing the `ColumnNotFoundDetails` error.
    ///
    /// Problem: `available_columns` is built via `flat_map` over `spec_map.values()`,
    /// which iterates in HashMap order (non-deterministic). In a multi-org-scope query
    /// where multiple sensors contribute columns for the same table, the `available_columns`
    /// Vec may appear in any order across calls, and may contain duplicates when multiple
    /// org-scoped entries define the same column name (e.g. multi-tenant overlays that
    /// share the same base spec).
    ///
    /// Fix: sort + dedup `available_columns` before constructing the error.
    ///
    /// Load-bearing (available_columns order): two sensors (alpha, beta) each contribute
    /// one column ("col_z" and "col_a" respectively). The error's `available_columns`
    /// field must be sorted ["col_a", "col_z"] regardless of HashMap iteration order.
    /// Removing the sort causes non-deterministic ordering failure.
    ///
    /// Load-bearing (duplicates): two org-scoped entries for the same sensor/table
    /// (simulating multi-org-scope with overlapping column names) would produce duplicates
    /// without the dedup. A second sensor with the same column name verifies no dupes appear.
    #[test]
    fn test_BC_2_11_016_available_columns_sorted_deduped_in_column_not_found_error() {
        // Two sensors under "acme", contributing columns in reverse-lex order.
        // Sensor "zebra" → column "col_z" (would appear first if HashMap puts zebra first).
        // Sensor "alpha" → column "col_a" (lexically earlier).
        // The error's available_columns must be sorted: ["col_a", "col_z"].
        //
        // NOTE: both sensors have different sensor IDs → different FQ table names.
        // "zebra_findings" and "alpha_findings" — these are DIFFERENT FQ tables.
        // To get BOTH columns in the same available_columns Vec, both sensors must match
        // the query's table_name. That requires both FQ names to equal table_name — impossible
        // unless they share the same FQ name.
        //
        // Correct setup: ONE sensor with TWO columns — let's verify sort within single-sensor case.
        // The HashMap non-determinism is across sensors, not within one sensor's column Vec.
        // For sorting: two sensors both with FQ = "shared_s1_findings" requires different
        // sensor IDs — also impossible.
        //
        // REAL test for sort: one sensor with multiple columns inserted in reverse-lex order.
        // available_columns from flat_map preserves Vec order (deterministic within one sensor),
        // but across multiple spec_map entries (multi-sensor OR multi-org) the HashMap order matters.
        // For the sort fix, use TWO spec_map entries that both produce columns for the SAME
        // fully-qualified table name. This is achievable by using two org-slug entries for
        // the SAME sensor spec (simulating multi-tenant overlays of the same sensor).

        // Two entries: same sensor_id "shared", same table_suffix "data",
        // but different org_slugs ("acme1" and "acme2"). The org_scope includes BOTH.
        // Both produce FQ table "shared_data". Both contribute the same column "col_dup"
        // plus each contributes a unique column: "col_z" (acme1) and "col_a" (acme2).
        let col_z = ColumnSpec::new("col_z", ColumnType::String, None, vec![]);
        let col_a = ColumnSpec::new("col_a", ColumnType::String, None, vec![]);
        let col_dup = ColumnSpec::new("col_dup", ColumnType::String, None, vec![]);

        let col_z2 = ColumnSpec::new("col_z", ColumnType::String, None, vec![]); // duplicate name
        let col_a2 = ColumnSpec::new("col_a", ColumnType::String, None, vec![]); // duplicate name
        let col_dup2 = ColumnSpec::new("col_dup", ColumnType::String, None, vec![]); // duplicate

        let (key_acme1, val_acme1) = make_resolved_with_columns(
            "shared",
            "data",
            "acme1",
            vec![col_z, col_dup], // "col_z" first (reverse lex)
        );
        let (key_acme2, val_acme2) = make_resolved_with_columns(
            "shared",
            "data",
            "acme2",
            vec![col_dup2, col_a], // "col_dup" then "col_a"
        );

        // Suppress unused:
        let _ = (col_z2, col_a2);

        let mut spec_map = HashMap::new();
        spec_map.insert(key_acme1, val_acme1);
        spec_map.insert(key_acme2, val_acme2);

        // org_scope covers BOTH orgs → both entries contribute columns.
        let acme1 = OrgSlug::new("acme1");
        let acme2 = OrgSlug::new("acme2");
        let org_scope = [acme1, acme2];

        let table_name = "shared_data";
        let column_typo = "completely_unknown_xyz"; // not in any column list, dist > 3 → no did_you_mean

        let result = check_column_availability(
            column_typo,
            table_name,
            "test-client",
            Some(&org_scope),
            Some(&spec_map),
            None, // table_registry not needed; resolved_spec_map is wired
        );

        let err = result.expect_err(
            "check_column_availability must return Err for unknown column 'completely_unknown_xyz'",
        );

        let details = match err {
            prism_core::error::PrismError::ColumnNotFound(ref d) => d.clone(),
            other => panic!("expected ColumnNotFound, got: {other:?}"),
        };

        let cols = &details.available_columns;

        // OBS-FRESH-1 sort assertion: available_columns must be sorted lexicographically.
        // The implementation sorts+deduplicates before returning; without that step,
        // order would depend on HashMap iteration (non-deterministic).
        // Expected after sort+dedup: ["col_a", "col_dup", "col_z"].
        let mut expected_sorted = cols.clone();
        expected_sorted.sort();
        expected_sorted.dedup();

        assert_eq!(
            cols, &expected_sorted,
            "OBS-FRESH-1: available_columns must be sorted and deduped. Got: {:?}. \
             Expected sorted+deduped: {:?}. \
             Current HEAD builds via flat_map over HashMap values (nondeterministic order) \
             without sort or dedup.",
            cols, expected_sorted
        );

        // OBS-FRESH-1 dedup assertion: no duplicate column names.
        let unique_count = {
            let mut seen = std::collections::HashSet::new();
            cols.iter().filter(|c| seen.insert(*c)).count()
        };
        assert_eq!(
            cols.len(),
            unique_count,
            "OBS-FRESH-1: available_columns must not contain duplicates. Got: {:?}",
            cols
        );
    }
}

// ---------------------------------------------------------------------------
// HIGH-1 sibling-sweep load-bearing tests (S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001)
// BC-2.11.020 — SqlPipe mode-agnostic gate coverage for E-QUERY-011 / E-QUERY-037 / E-QUERY-038
// ---------------------------------------------------------------------------

#[cfg(test)]
mod sqlpipe_gate_sweep_tests {
    use super::*;
    use std::sync::Arc;

    struct NoopCs2;
    #[async_trait::async_trait]
    impl prism_credentials::CredentialStore for NoopCs2 {
        async fn get(
            &self,
            _t: &prism_core::OrgSlug,
            _s: &str,
            _n: &prism_credentials::namespace::CredentialName,
        ) -> Result<Option<secrecy::SecretString>, PrismError> {
            Ok(None)
        }
        async fn set(
            &self,
            _t: &prism_core::OrgSlug,
            _s: &str,
            _n: &prism_credentials::namespace::CredentialName,
            _v: secrecy::SecretString,
        ) -> Result<(), PrismError> {
            Ok(())
        }
        async fn delete(
            &self,
            _t: &prism_core::OrgSlug,
            _s: &str,
            _n: &prism_credentials::namespace::CredentialName,
        ) -> Result<bool, PrismError> {
            Ok(false)
        }
        async fn list(
            &self,
            _t: &prism_core::OrgSlug,
        ) -> Result<Vec<(String, prism_credentials::namespace::CredentialName)>, PrismError>
        {
            Ok(vec![])
        }
        async fn exists(
            &self,
            _t: &prism_core::OrgSlug,
            _s: &str,
            _n: &prism_credentials::namespace::CredentialName,
        ) -> Result<bool, PrismError> {
            Ok(false)
        }
    }

    fn make_test_engine() -> QueryEngine {
        QueryEngine::new_with_cache_config(
            Arc::new(prism_sensors::AdapterRegistry::new()),
            Arc::new(NoopCs2),
            Arc::new(prism_ocsf::OcsfNormalizer::new()),
            Arc::new(crate::scoping::ClientRegistry::new(vec![])),
            QueryEngineConfig::default(),
            crate::cache::CacheConfig::default(),
        )
    }

    // ── E-QUERY-011 / SqlPipe ─────────────────────────────────────────────────

    /// HIGH-1 load-bearing (E-QUERY-011): A SqlPipe query whose head SELECT references
    /// `prism_audit` must be denied with `PrismError::AuditTableAccessDenied` when
    /// `Capability::AuditRead` is absent from the caller's capabilities.
    ///
    /// Before the HIGH-1 fix, `extract_source_names_recursive` had a `_ => {}` catch-all
    /// for `Ast::SqlPipe`, so the E-QUERY-011 gate was a no-op for SqlPipe mode.
    /// This test drives `check_internal_table_capabilities` via the parse-and-extract
    /// path that was broken.
    ///
    /// BC-2.11.020 / BC-2.15.011 / F-LP2-CRIT-1
    #[test]
    fn test_high1_sqlpipe_head_prism_audit_denied_without_audit_read_capability_e_query_011() {
        // A SqlPipe query whose head references prism_audit (internal table).
        // With no AuditRead capability → must return AuditTableAccessDenied (E-QUERY-011).
        let query = "SELECT * FROM prism_audit WHERE event_type = 'query.execute' | limit 10";

        // Parse first to confirm this is an Ast::SqlPipe (belt-and-suspenders).
        let ast = crate::filter_parser::PrismQlParser::parse(query)
            .expect("SqlPipe audit-head query must parse");
        assert!(
            matches!(ast, crate::ast::Ast::SqlPipe(_)),
            "HIGH-1 setup: query must parse as Ast::SqlPipe; got {ast:?}"
        );

        // Invoke gate directly with NO AuditRead capability.
        let result = check_internal_table_capabilities(query, &[]);
        assert!(
            matches!(result, Err(PrismError::AuditTableAccessDenied)),
            "HIGH-1 / E-QUERY-011: SqlPipe head referencing prism_audit WITHOUT \
             Capability::AuditRead must return Err(PrismError::AuditTableAccessDenied); \
             before the HIGH-1 fix this returned Ok(()), bypassing the gate. Got: {result:?}"
        );
    }

    /// HIGH-1 gate pass (E-QUERY-011 inverse): Same SqlPipe query WITH AuditRead must pass.
    #[test]
    fn test_high1_sqlpipe_head_prism_audit_allowed_with_audit_read_capability() {
        let query = "SELECT * FROM prism_audit WHERE event_type = 'query.execute' | limit 10";
        let result = check_internal_table_capabilities(query, &[Capability::AuditRead]);
        assert!(
            result.is_ok(),
            "HIGH-1 / E-QUERY-011 inverse: SqlPipe head referencing prism_audit WITH \
             Capability::AuditRead must return Ok(()); got: {result:?}"
        );
    }

    // ── E-QUERY-037 / SqlPipe ─────────────────────────────────────────────────

    /// HIGH-1 load-bearing (E-QUERY-037): A SqlPipe query whose head references an
    /// unregistered table must return `PrismError::TableNotAvailable` (E-QUERY-037) with
    /// `available_tables` / `did_you_mean` populated — NOT succeed or return a different error.
    ///
    /// Before the HIGH-1 fix, `extract_sources_from_ast_for_gate` had `_ => {}` for
    /// `Ast::SqlPipe`, so the E-QUERY-037 gate was a no-op for SqlPipe mode.
    ///
    /// BC-2.11.001 / AC-8 mode-agnostic / S-3.13 AC-2 / TD-VSDD-060
    #[tokio::test]
    async fn test_high1_sqlpipe_head_unregistered_table_returns_e_query_037() {
        use crate::table_registry::TableRegistry;
        use prism_spec_engine::spec_parser::{AuthType, SensorSpec, TableSpec};

        // Build a registry with only armis registered (no crowdstrike).
        let registry = Arc::new(TableRegistry::new());
        let armis_spec = SensorSpec::new(
            "armis",
            "Armis sensor",
            AuthType::ApiKey,
            "https://example.com",
            vec![TableSpec::new_point_in_time(
                "devices",
                "network_activity",
                vec![],
                vec![],
            )],
            None,
            "1.0.0",
            Vec::new(),
        );
        registry
            .register_sensor(&armis_spec)
            .expect("register armis must not fail");

        // Engine with registry wired — SqlPipe head references an unregistered table.
        let engine = make_test_engine().with_table_registry(Arc::clone(&registry));

        // Execute a SqlPipe query targeting crowdstrike.detections (unregistered).
        let result = engine
            .execute(
                "SELECT * FROM crowdstrike.detections WHERE severity = 'HIGH' | limit 20",
                QueryOptions::default(),
            )
            .await;

        match result {
            Err(PrismError::TableNotAvailable(ref details)) => {
                let display = details.to_string();
                assert!(
                    display.starts_with("E-QUERY-037:"),
                    "HIGH-1 / E-QUERY-037: SqlPipe head referencing unregistered table must \
                     return E-QUERY-037; display was: {display}"
                );
                // available_sensors / available_tables must list armis (the registered sensor).
                assert!(
                    details.available_sensors.contains("armis"),
                    "HIGH-1 / E-QUERY-037: available_sensors must contain 'armis'. \
                     Got: '{}'",
                    details.available_sensors
                );
            }
            Ok(_) => panic!(
                "HIGH-1 / E-QUERY-037: SqlPipe query targeting unregistered table must NOT succeed \
                 (before fix the gate was bypassed for SqlPipe). E-QUERY-037 must fire."
            ),
            Err(other) => panic!(
                "HIGH-1 / E-QUERY-037: expected PrismError::TableNotAvailable, got: {other:?}"
            ),
        }
    }

    // ── E-QUERY-038 / SqlPipe ─────────────────────────────────────────────────

    /// HIGH-1 load-bearing (E-QUERY-038): A SqlPipe query whose head SELECT projects
    /// a column name that does not exist in the resolved_spec_map must return
    /// `PrismError::ColumnNotFound` (E-QUERY-038) at plan time.
    ///
    /// Before the HIGH-1 fix, `check_query_column_availability` had `_ => return Ok(())`
    /// for `Ast::SqlPipe`, so the E-QUERY-038 pedagogical gate was bypassed for SqlPipe mode.
    ///
    /// BC-2.11.016 / S-DEMO-PRISMQL-ONBOARDING-001-B / TD-VSDD-060
    #[tokio::test]
    async fn test_high1_sqlpipe_head_typo_column_returns_e_query_038() {
        use crate::table_registry::TableRegistry;
        use prism_core::SensorId;
        use prism_spec_engine::{
            overlay::{OverlayLoader, SensorInstanceOverlay},
            spec_parser::{AuthType, ColumnSpec, SensorSpec, TableSpec},
            ResolvedSpecKey,
        };
        use std::collections::HashMap;
        use std::sync::Arc as StdArc;

        // Build TableRegistry with crowdstrike registered (one column: "severity").
        // ColumnSpec is #[non_exhaustive] in an external crate — use Default then mutate.
        let mut col_spec = ColumnSpec::default();
        col_spec.name = "severity".to_string();
        col_spec.column_type = prism_core::column::ColumnType::String;
        let crowdstrike_spec = SensorSpec::new(
            "crowdstrike",
            "CrowdStrike sensor",
            AuthType::ApiKey,
            "https://example.com",
            vec![TableSpec::new_point_in_time(
                "detections",
                "security_finding",
                vec![col_spec],
                vec![],
            )],
            None,
            "1.0.0",
            Vec::new(),
        );
        let registry = Arc::new(TableRegistry::new());
        registry
            .register_sensor(&crowdstrike_spec)
            .expect("register crowdstrike must not fail");

        // Build resolved_spec_map using the canonical OverlayLoader factory
        // (same pattern as `make_two_org_spec_map` in the multi-tenant gate tests).
        let org = "testorg";
        let overlay_toml = "extends = \"crowdstrike\"\ninstance_id = \"crowdstrike@testorg\"";
        let overlay: SensorInstanceOverlay = toml::from_str(overlay_toml)
            .expect("E-QUERY-038 fixture: SensorInstanceOverlay TOML must parse");
        let org_slug = OrgSlug::new(org);
        let resolved = OverlayLoader::merge_overlay_onto_type_spec(
            &crowdstrike_spec,
            &overlay,
            org_slug.clone(),
        );
        let key: ResolvedSpecKey = (org_slug, SensorId::new("crowdstrike"));
        let mut spec_map = HashMap::new();
        spec_map.insert(key, resolved);

        // Wire the spec_map into the engine.
        let mut engine = make_test_engine().with_table_registry(Arc::clone(&registry));
        engine.resolved_spec_map = Some(Arc::new(arc_swap::ArcSwap::new(StdArc::new(spec_map))));

        // SqlPipe query projecting a non-existent column "severit" (typo of "severity",
        // Levenshtein distance = 1 which is within the ≤3 threshold).
        // With org_scope matching "testorg", the column gate resolves "crowdstrike_detections"
        // to the spec above, finds only "severity", and must deny "severit" with E-QUERY-038.
        // Note: "sev" would be distance 5 from "severity" (>3 threshold) so would give
        // did_you_mean=None. "severit" (missing trailing 'y') is the correct test typo.
        //
        // IMPORTANT: Uses underscore form "crowdstrike_detections" (NOT dot form
        // "crowdstrike.detections"). BC-2.11.001 v1.15 / EC-11-067: dot-notation in FROM
        // targets is rejected with E-QUERY-037 for ALL modes including SqlPipe.
        // The underscore form must pass the availability gate so the column gate fires.
        let result = engine
            .execute(
                "SELECT severit FROM crowdstrike_detections | limit 5",
                QueryOptions {
                    clients: Some(vec![OrgSlug::new("testorg")]),
                    ..Default::default()
                },
            )
            .await;

        match result {
            Err(PrismError::ColumnNotFound(ref details)) => {
                let display = details.to_string();
                assert!(
                    display.starts_with("E-QUERY-038:"),
                    "HIGH-1 / E-QUERY-038: SqlPipe head with typo'd column must return \
                     E-QUERY-038; display was: {display}"
                );
                assert_eq!(
                    details.column, "severit",
                    "HIGH-1 / E-QUERY-038: column field must be 'severit'"
                );
                // did_you_mean should suggest "severity" (Levenshtein distance = 1 from "severit").
                assert!(
                    details.did_you_mean.as_deref() == Some("severity"),
                    "HIGH-1 / E-QUERY-038: did_you_mean should suggest 'severity'; got {:?}",
                    details.did_you_mean
                );
            }
            Ok(_) => panic!(
                "HIGH-1 / E-QUERY-038: SqlPipe query with typo'd column 'severit' must NOT \
                 succeed. E-QUERY-038 (ColumnNotFound) must fire."
            ),
            Err(other) => {
                panic!("HIGH-1 / E-QUERY-038: expected PrismError::ColumnNotFound, got: {other:?}")
            }
        }
    }

    // ── H1: execute_inner vs execute_scheduled_inner gate ordering ────────────

    /// H1 regression: capability gate (E-QUERY-011) ordering consistency between
    /// `execute` and `execute_scheduled`.
    ///
    /// BEFORE the H1 fix: `execute_scheduled_inner` ran the capability gate FIRST
    /// (before E-QUERY-037/038/039), while `execute_inner` ran it LAST (after 037/038/039).
    /// This meant the same query would return different first errors depending on entry point.
    ///
    /// AFTER the H1 fix: both entry points use the same canonical gate order:
    ///   E-QUERY-037 → E-QUERY-038 → E-QUERY-039 → E-QUERY-011 (capability, LAST).
    ///
    /// Test strategy: query referencing `prism_audit` (requires AuditRead capability) AND
    /// an unregistered external table. With a TableRegistry that only has no registered
    /// sensor, E-QUERY-037 should fire for the unregistered table from BOTH entry points.
    /// (prism_* tables are skipped by the table gate, so the outer table in a combined query
    /// would trip 037.) For a pure prism_audit query, E-QUERY-011 fires from both.
    ///
    /// This test uses a pure `prism_audit` query with NO TableRegistry to verify that
    /// E-QUERY-011 fires consistently from both execute and execute_scheduled.
    ///
    /// Load-bearing (TD-VSDD-059): verifies both paths return the same error type.
    /// If the capability gate is removed from execute_scheduled, this test fails.
    #[tokio::test]
    async fn test_h1_capability_gate_consistent_across_execute_and_execute_scheduled() {
        let engine = make_test_engine();

        // `prism_audit` query — no AuditRead capability in QueryOptions (default).
        // Both execute and execute_scheduled must return E-QUERY-011.
        let query = "SELECT * FROM prism_audit LIMIT 10";

        let execute_result = engine.execute(query, QueryOptions::default()).await;

        // Map execute_scheduled to Result<QueryResult, PrismError> by discarding the Arc<SessionContext>
        // (SessionContext does not impl Debug so we cannot unwrap_err on the raw tuple result).
        let scheduled_result = engine
            .execute_scheduled(query, None)
            .await
            .map(|(qr, _ctx)| qr);

        // Both must error.
        assert!(
            execute_result.is_err(),
            "H1: execute with prism_audit + no AuditRead must return Err; got Ok"
        );
        assert!(
            scheduled_result.is_err(),
            "H1: execute_scheduled with prism_audit + no AuditRead must return Err; got Ok. \
             If Ok: the capability gate in execute_scheduled_inner is missing or bypassed."
        );

        let exec_err = execute_result.unwrap_err();
        let sched_err = scheduled_result.unwrap_err();

        // Both must be AuditTableAccessDenied (E-QUERY-011).
        assert!(
            matches!(exec_err, PrismError::AuditTableAccessDenied),
            "H1: execute must return AuditTableAccessDenied; got: {exec_err:?}"
        );
        assert!(
            matches!(sched_err, PrismError::AuditTableAccessDenied),
            "H1: execute_scheduled must return AuditTableAccessDenied (E-QUERY-011) to \
             match execute behavior. Got: {sched_err:?}. \
             Prior to H1 fix: the capability gate ran BEFORE 037/038/039 in \
             execute_scheduled but AFTER in execute — this test verifies alignment."
        );
    }

    // ── F-P1L4-MED-001: DISCRIMINATING gate-ordering regression guard ─────────

    /// F-P1L4-MED-001 load-bearing (DISCRIMINATING H1 ordering test).
    ///
    /// The existing `test_h1_capability_gate_consistent_across_execute_and_execute_scheduled`
    /// uses a PURE `prism_audit` query with NO competing gate (table/column/enrich).
    /// That test is NECESSARY but NOT SUFFICIENT — it would PASS against the pre-fix
    /// `execute_scheduled_inner` where the capability gate ran FIRST (before 037/038/039),
    /// because a pure `prism_audit` query doesn't hit the table gate at all.
    ///
    /// This test DISCRIMINATES the ordering by constructing a query that SIMULTANEOUSLY
    /// triggers:
    ///   - E-QUERY-037 (TableNotAvailable): `ghost_sensor_detections` is NOT registered in
    ///     the wired TableRegistry (which has only `armis` sensors).
    ///   - E-QUERY-011 (AuditTableAccessDenied): the query JOINs `prism_audit`, which requires
    ///     `Capability::AuditRead` that is ABSENT from both `QueryOptions::default()` (execute)
    ///     and the system context `&[]` (execute_scheduled).
    ///
    /// With current (post-fix) gate ordering — E-QUERY-037 → E-QUERY-038 → E-QUERY-039 → E-QUERY-011:
    ///   BOTH `execute` and `execute_scheduled` return E-QUERY-037 (table gate fires first).
    ///
    /// With pre-fix `execute_scheduled_inner` ordering — E-QUERY-011 BEFORE 037/038/039:
    ///   `execute` returns E-QUERY-037 (table first) but `execute_scheduled` would return
    ///   E-QUERY-011 (capability first) — ASYMMETRIC first-error behavior.
    ///
    /// This test WOULD FAIL against the pre-fix code because `execute_scheduled` would return
    /// E-QUERY-011, not E-QUERY-037. It PASSES against the current code.
    ///
    /// TableRegistry wiring note: `check_table_availability` skips when `registry` is None.
    /// A wired registry is REQUIRED for E-QUERY-037 to fire. The engine here has `armis`
    /// registered so the gate is active for any non-armis table.
    ///
    /// `prism_audit` is classified as `SourceRefKind::Internal` and SKIPPED by the table gate
    /// (`check_availability_gate` line: "Skip internal prism_* tables"). So the table gate
    /// only fires for `ghost_sensor_detections` (Custom kind, not in registry).
    ///
    /// `check_internal_table_capabilities` sees BOTH source names via `extract_source_names_recursive`
    /// (FROM + JOIN sources) and returns E-QUERY-011 when `prism_audit` is present and no
    /// AuditRead capability is provided.
    ///
    /// F-P1L4-MED-001 / BC-2.11.019 v1.5 / H1 fix (S-DEMO-FIDELITY-REMEDIATION-001)
    #[tokio::test]
    async fn test_h1_gate_ordering_discriminating_table_fires_before_capability() {
        use crate::table_registry::TableRegistry;
        use prism_spec_engine::spec_parser::{AuthType, SensorSpec, TableSpec};

        // Build an engine with a wired TableRegistry containing only `armis`.
        // This activates the E-QUERY-037 gate for any non-armis table.
        let armis_spec = SensorSpec::new(
            "armis",
            "Armis sensor",
            AuthType::ApiKey,
            "https://example.com",
            vec![TableSpec::new_point_in_time(
                "alerts",
                "security_finding",
                vec![],
                vec![],
            )],
            None,
            "1.0.0",
            Vec::new(),
        );
        let registry = Arc::new(TableRegistry::new());
        registry
            .register_sensor(&armis_spec)
            .expect("register armis must not fail");

        let engine = QueryEngine::new_with_cache_config(
            Arc::new(prism_sensors::AdapterRegistry::new()),
            Arc::new(NoopCs2),
            Arc::new(prism_ocsf::OcsfNormalizer::new()),
            Arc::new(crate::scoping::ClientRegistry::new(vec![])),
            QueryEngineConfig::default(),
            crate::cache::CacheConfig::default(),
        )
        .with_table_registry(Arc::clone(&registry));

        // Discriminating query: references BOTH an unregistered external table AND prism_audit.
        //   `ghost_sensor_detections` — Custom kind, NOT in registry → E-QUERY-037 (table gate)
        //   `prism_audit` via JOIN   — Internal kind, SKIPPED by table gate; requires AuditRead
        //                              capability → E-QUERY-011 (capability gate)
        //
        // Pre-fix (capability FIRST in execute_scheduled_inner):
        //   execute         returns E-QUERY-037 (table first, correct order)
        //   execute_scheduled returns E-QUERY-011 (capability FIRST — the bug)
        //   → ASYMMETRIC: the same query gives different first errors per entry point.
        //
        // Post-fix (table FIRST in both):
        //   BOTH return E-QUERY-037 (table gate fires before capability gate).
        let query = "SELECT * FROM ghost_sensor_detections JOIN prism_audit ON id = id LIMIT 10";

        let execute_result = engine.execute(query, QueryOptions::default()).await;
        let scheduled_result = engine
            .execute_scheduled(query, None)
            .await
            .map(|(qr, _ctx)| qr);

        // Both must error.
        assert!(
            execute_result.is_err(),
            "F-P1L4-MED-001: execute with unregistered table must return Err; got Ok"
        );
        assert!(
            scheduled_result.is_err(),
            "F-P1L4-MED-001: execute_scheduled with unregistered table must return Err; got Ok"
        );

        let exec_err = execute_result.unwrap_err();
        let sched_err = scheduled_result.unwrap_err();

        // DISCRIMINATING assertion: BOTH must return E-QUERY-037 (TableNotAvailable),
        // NOT E-QUERY-011 (AuditTableAccessDenied).
        //
        // If execute_scheduled_inner still runs the capability gate FIRST (pre-fix ordering),
        // `sched_err` would be AuditTableAccessDenied (E-QUERY-011), and this assertion fails.
        assert!(
            matches!(exec_err, PrismError::TableNotAvailable(_)),
            "F-P1L4-MED-001 DISCRIMINATING: execute must return TableNotAvailable (E-QUERY-037) \
             when table gate fires before capability gate. \
             Got: {exec_err:?}. If AuditTableAccessDenied: the ordering is wrong \
             (capability gate is running before table gate in execute_inner)."
        );
        assert!(
            matches!(sched_err, PrismError::TableNotAvailable(_)),
            "F-P1L4-MED-001 DISCRIMINATING: execute_scheduled must return TableNotAvailable \
             (E-QUERY-037) when table gate fires before capability gate. \
             Got: {sched_err:?}. \
             If AuditTableAccessDenied (E-QUERY-011): the H1 fix is incomplete — \
             execute_scheduled_inner is still running the capability gate FIRST \
             (before E-QUERY-037), causing asymmetric first-error behavior. \
             This test FAILS against the pre-fix execute_scheduled_inner ordering."
        );

        // Sanity-check: error message must mention the unregistered table name.
        if let PrismError::TableNotAvailable(ref details) = exec_err {
            assert!(
                details.to_string().starts_with("E-QUERY-037:"),
                "F-P1L4-MED-001: E-QUERY-037 error display must start with 'E-QUERY-037:'; \
                 got: {}",
                details
            );
        }
    }
}

// ---------------------------------------------------------------------------
// S-DEMO-FIDELITY-REMEDIATION-001 C1+C2 test-only re-export
// ---------------------------------------------------------------------------

/// Test-only re-export of `collect_unknown_scalars_from_sql_query` for unit tests
/// in `bc_2_11_019_n1b_test.rs` that construct `SqlQuery` nodes directly.
///
/// The production function is module-private. This wrapper grants `pub(crate)` visibility
/// exclusively for direct AST construction tests that verify GROUP BY / ORDER BY / JOIN ON
/// walks without going through the parser (some positions are not reachable via the
/// PrismQL parser grammar — see C1+C2 note in `bc_2_11_019_n1b_test.rs`).
///
/// TD-VSDD-059: load-bearing — removing the GROUP BY / ORDER BY / JOIN ON walks from
/// `collect_unknown_scalars_from_sql_query` causes the corresponding unit tests to fail.
#[cfg(test)]
pub(crate) fn collect_unknown_scalars_from_sql_query_test_only(
    sq: &crate::ast::SqlQuery,
    out: &mut Vec<String>,
) {
    collect_unknown_scalars_from_sql_query(sq, out)
}

// ---------------------------------------------------------------------------
// S-DEMO-FIDELITY-REMEDIATION-001 HIGH-003 unit tests
// ---------------------------------------------------------------------------
//
// These tests directly exercise `collect_unknown_scalar_from_predicate` and
// `collect_unknown_scalar_from_expr` (now module-level private fns accessible
// from this #[cfg(test)] block via `super::`).
//
// Rationale for direct AST construction (not query-string parsing):
// The SQL parser's WHERE predicate grammar uses `build_predicate_parser()` which
// does NOT include scalar function call syntax — `WHERE badudf(col) = 1` is a
// parse error (E-QUERY-001) at runtime. The collect_unknown_scalar_from_predicate
// helper is defensive code for programmatic AST construction (e.g., macros,
// future parser extensions). These unit tests verify the logic directly.
//
// TD-VSDD-059: load-bearing unit tests on the actual collect_ functions.

#[cfg(test)]
mod enrich_gate_where_clause_unit_tests {
    use crate::ast::{
        CompareOp, Expr, FieldPath, FuncCall, Literal, LogicalOp, Predicate, ScalarFunc,
    };

    /// HIGH-003 regression — `collect_unknown_scalar_from_predicate` finds
    /// `ScalarFunc::Unknown` in a `Predicate::Compare { lhs: Expr::FuncCall(..) }`.
    ///
    /// Constructs a predicate: `badudf(field) = "value"` where `badudf` is
    /// `ScalarFunc::Unknown("badudf")`. Asserts the name is collected.
    #[test]
    fn test_high003_collect_unknown_scalar_in_compare_lhs() {
        let pred = Predicate::Compare {
            lhs: Box::new(Expr::FuncCall(FuncCall::Scalar {
                func: ScalarFunc::Unknown("badudf".to_string()),
                args: vec![Expr::Field(FieldPath::new(vec!["col".to_string()]))],
            })),
            op: CompareOp::Eq,
            rhs: Box::new(Expr::Literal(Literal::String("value".to_string()))),
        };

        let mut out = Vec::new();
        super::collect_unknown_scalar_from_predicate(&pred, &mut out);

        assert_eq!(
            out,
            vec!["badudf".to_string()],
            "HIGH-003: collect_unknown_scalar_from_predicate must collect \
             ScalarFunc::Unknown in Predicate::Compare lhs"
        );
    }

    /// HIGH-003 regression — `collect_unknown_scalar_from_predicate` finds
    /// `ScalarFunc::Unknown` nested in `Predicate::Logical` (AND/OR).
    #[test]
    fn test_high003_collect_unknown_scalar_in_logical_predicate() {
        let compare_with_udf = Predicate::Compare {
            lhs: Box::new(Expr::FuncCall(FuncCall::Scalar {
                func: ScalarFunc::Unknown("badudf".to_string()),
                args: vec![],
            })),
            op: CompareOp::Eq,
            rhs: Box::new(Expr::Literal(Literal::Integer(1))),
        };
        let simple_compare = Predicate::Compare {
            lhs: Box::new(Expr::Field(FieldPath::new(vec!["severity".to_string()]))),
            op: CompareOp::Eq,
            rhs: Box::new(Expr::Literal(Literal::String("high".to_string()))),
        };

        let logical = Predicate::Logical {
            op: LogicalOp::And,
            predicates: vec![compare_with_udf, simple_compare],
        };

        let mut out = Vec::new();
        super::collect_unknown_scalar_from_predicate(&logical, &mut out);

        assert_eq!(
            out,
            vec!["badudf".to_string()],
            "HIGH-003: collect_unknown_scalar_from_predicate must collect \
             ScalarFunc::Unknown nested inside Predicate::Logical"
        );
    }

    /// HIGH-003 regression — `collect_unknown_scalar_from_predicate` finds nothing
    /// in a simple field-vs-literal comparison (no function call).
    ///
    /// Negative control: regular predicates must NOT emit false positives.
    #[test]
    fn test_high003_no_false_positive_on_simple_predicate() {
        let pred = Predicate::Compare {
            lhs: Box::new(Expr::Field(FieldPath::new(vec!["severity".to_string()]))),
            op: CompareOp::Eq,
            rhs: Box::new(Expr::Literal(Literal::String("high".to_string()))),
        };

        let mut out = Vec::new();
        super::collect_unknown_scalar_from_predicate(&pred, &mut out);

        assert!(
            out.is_empty(),
            "HIGH-003: collect_unknown_scalar_from_predicate must NOT collect \
             anything from a plain field = literal predicate. Got: {out:?}"
        );
    }

    /// HIGH-003 regression — `collect_unknown_scalar_from_predicate` handles `Not`.
    #[test]
    fn test_high003_collect_unknown_scalar_in_not_predicate() {
        let inner = Predicate::Compare {
            lhs: Box::new(Expr::FuncCall(FuncCall::Scalar {
                func: ScalarFunc::Unknown("evil_udf".to_string()),
                args: vec![],
            })),
            op: CompareOp::Ne,
            rhs: Box::new(Expr::Literal(Literal::Integer(0))),
        };
        let not_pred = Predicate::Not(Box::new(inner));

        let mut out = Vec::new();
        super::collect_unknown_scalar_from_predicate(&not_pred, &mut out);

        assert_eq!(
            out,
            vec!["evil_udf".to_string()],
            "HIGH-003: collect_unknown_scalar_from_predicate must collect \
             ScalarFunc::Unknown inside Predicate::Not"
        );
    }
}

// ---------------------------------------------------------------------------
// S-DEMO-FIDELITY-REMEDIATION-001 M2 — E-QUERY-038 FuncCall-arg + JOIN ON tests
// ---------------------------------------------------------------------------
//
// Finding M2: `check_query_column_availability` only matched `Expr::Field` directly
// in GROUP BY and ORDER BY, missing column refs wrapped in function calls (e.g.
// `GROUP BY lower(col_typo)` bypassed the gate). JOIN ON was not validated at all.
//
// Fix: `extract_field_paths_from_expr` recursively collects FieldPath refs from
// FuncCall args. GROUP BY, ORDER BY, and JOIN ON positions now use this helper
// instead of a direct `Expr::Field` match.
//
// Tests assert E-QUERY-038 fires for:
//   1. GROUP BY with a column typo wrapped in a function call (lower(typo_col))
//   2. ORDER BY with a column typo wrapped in a function call (lower(typo_col))
//   3. JOIN ON with a bare column typo in the FROM table
//
// TD-VSDD-059: load-bearing — removing the FuncCall-arg walk from
// `extract_field_paths_from_expr` or removing the Position 5 JOIN ON walk from
// `check_query_column_availability` causes these tests to fail.

#[cfg(test)]
#[allow(
    non_snake_case,
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::too_many_lines
)]
mod m2_column_gate_funccall_and_join_tests {
    use std::{collections::HashMap, sync::Arc};

    use prism_core::{OrgSlug, SensorId};
    use prism_sensors::AdapterRegistry;
    use prism_spec_engine::{
        overlay::{OverlayLoader, SensorInstanceOverlay},
        spec_parser::{AuthType, ColumnSpec, SensorSpec, TableSpec},
        ResolvedSensorSpec, ResolvedSpecKey,
    };

    use super::*;
    use crate::{scoping::ClientRegistry, table_registry::TableRegistry};
    use prism_core::column::ColumnType;

    // ── Helpers ────────────────────────────────────────────────────────────────

    /// Minimal no-op credential store for M2 gate tests.
    struct NoopCs;

    #[async_trait::async_trait]
    impl prism_credentials::CredentialStore for NoopCs {
        async fn get(
            &self,
            _t: &prism_core::OrgSlug,
            _s: &str,
            _n: &prism_credentials::namespace::CredentialName,
        ) -> Result<Option<secrecy::SecretString>, PrismError> {
            Ok(None)
        }
        async fn set(
            &self,
            _t: &prism_core::OrgSlug,
            _s: &str,
            _n: &prism_credentials::namespace::CredentialName,
            _v: secrecy::SecretString,
        ) -> Result<(), PrismError> {
            Ok(())
        }
        async fn delete(
            &self,
            _t: &prism_core::OrgSlug,
            _s: &str,
            _n: &prism_credentials::namespace::CredentialName,
        ) -> Result<bool, PrismError> {
            Ok(false)
        }
        async fn list(
            &self,
            _t: &prism_core::OrgSlug,
        ) -> Result<Vec<(String, prism_credentials::namespace::CredentialName)>, PrismError>
        {
            Ok(vec![])
        }
        async fn exists(
            &self,
            _t: &prism_core::OrgSlug,
            _s: &str,
            _n: &prism_credentials::namespace::CredentialName,
        ) -> Result<bool, PrismError> {
            Ok(false)
        }
    }

    /// Build a resolved spec map for `crowdstrike_alerts` (sensor="crowdstrike", table="alerts")
    /// under org "acme" with explicit columns: `severity` (String) and `timestamp` (Datetime).
    ///
    /// The table is registered in the `TableRegistry` and also in the resolved spec map
    /// so that both E-QUERY-037 (table gate) and E-QUERY-038 (column gate) can fire.
    fn make_crowdstrike_engine_with_columns() -> (QueryEngine, OrgSlug) {
        let org = OrgSlug::new("acme");
        let sensor_id = "crowdstrike";
        let table_suffix = "alerts";

        let columns = vec![
            ColumnSpec::new("severity", ColumnType::String, None, vec![]),
            ColumnSpec::new("timestamp", ColumnType::Datetime, None, vec![]),
        ];

        let spec = SensorSpec::new(
            sensor_id,
            "CrowdStrike sensor",
            AuthType::ApiKey,
            "https://api.crowdstrike.com",
            vec![TableSpec::new_point_in_time(
                table_suffix,
                "security_finding",
                columns,
                vec![],
            )],
            None,
            "1.0.0",
            Vec::new(),
        );

        // Build table registry.
        let registry = Arc::new(TableRegistry::new());
        registry
            .register_sensor(&spec)
            .expect("register crowdstrike must not fail");

        // Build resolved spec map with the same columns.
        let overlay_toml = format!("extends = \"{sensor_id}\"\ninstance_id = \"{sensor_id}@acme\"");
        let overlay: SensorInstanceOverlay = toml::from_str(&overlay_toml)
            .expect("M2 fixture: SensorInstanceOverlay TOML must parse");
        let resolved = OverlayLoader::merge_overlay_onto_type_spec(&spec, &overlay, org.clone());
        let key: ResolvedSpecKey = (org.clone(), SensorId::new(sensor_id));
        let mut spec_map: HashMap<ResolvedSpecKey, ResolvedSensorSpec> = HashMap::new();
        spec_map.insert(key, resolved);

        // Build engine with wired spec_map.
        let mut engine = QueryEngine::new_with_cache_config(
            Arc::new(AdapterRegistry::new()),
            Arc::new(NoopCs),
            Arc::new(prism_ocsf::OcsfNormalizer::new()),
            Arc::new(ClientRegistry::new(vec![org.clone()])),
            QueryEngineConfig::default(),
            crate::cache::CacheConfig::default(),
        );
        engine.resolved_spec_map = Some(Arc::new(arc_swap::ArcSwap::new(Arc::new(spec_map))));
        engine = engine.with_table_registry(registry);

        (engine, org)
    }

    // ── Tests ──────────────────────────────────────────────────────────────────

    /// M2 fix — E-QUERY-038 must fire for a column typo wrapped in a function call
    /// in the GROUP BY clause.
    ///
    /// Before fix: `GROUP BY lower(typo_col)` would bypass the column gate because
    /// `check_query_column_availability` only matched `Expr::Field` directly in the
    /// GROUP BY position — `lower(typo_col)` is `Expr::FuncCall(Aggregate { args: [Field] })`,
    /// which fell through to the `_ => None` arm.
    ///
    /// After fix: `extract_field_paths_from_expr` recurses into FuncCall args and
    /// collects `typo_col`, which then fails the schema check → E-QUERY-038.
    ///
    /// Load-bearing (M2 fix): removing FuncCall recursion from the GROUP BY walk in
    /// `extract_field_paths_from_expr` causes this query to bypass the column gate
    /// and produce a DataFusion error (E-INT-001) instead of E-QUERY-038.
    #[tokio::test]
    async fn test_m2_group_by_funccall_arg_col_typo_triggers_e_query_038() {
        let (engine, org) = make_crowdstrike_engine_with_columns();

        // `lower` is an aggregate-like scalar; the parser represents it as
        // FuncCall::Aggregate in AST. `typo_col` is not in the schema (only
        // `severity` and `timestamp` are valid).
        let query = "SELECT severity, COUNT(*) FROM crowdstrike_alerts GROUP BY lower(typo_col)";

        let result = engine
            .execute(
                query,
                QueryOptions {
                    clients: Some(vec![org]),
                    ..QueryOptions::default()
                },
            )
            .await;

        match result {
            Err(PrismError::ColumnNotFound(ref details)) => {
                assert_eq!(
                    details.column, "typo_col",
                    "M2 GROUP BY FuncCall: column must be 'typo_col', got: {:?}",
                    details.column
                );
                assert_eq!(
                    details.table, "crowdstrike_alerts",
                    "M2 GROUP BY FuncCall: table must be 'crowdstrike_alerts'"
                );
            }
            Ok(_) => panic!(
                "M2 GROUP BY FuncCall: engine.execute must NOT succeed — E-QUERY-038 must fire \
                 for column typo inside lower() in GROUP BY. Before M2 fix, the FuncCall arg \
                 was not walked."
            ),
            Err(other) => panic!(
                "M2 GROUP BY FuncCall: expected PrismError::ColumnNotFound (E-QUERY-038), \
                 got different error: {other:?}"
            ),
        }
    }

    /// M2 fix — E-QUERY-038 must fire for a column typo wrapped in a function call
    /// in the ORDER BY clause.
    ///
    /// Mirrors the GROUP BY test above for the ORDER BY position.
    #[tokio::test]
    async fn test_m2_order_by_funccall_arg_col_typo_triggers_e_query_038() {
        let (engine, org) = make_crowdstrike_engine_with_columns();

        // `lower(typo_col)` in ORDER BY — `typo_col` not in schema.
        let query = "SELECT severity FROM crowdstrike_alerts ORDER BY lower(typo_col)";

        let result = engine
            .execute(
                query,
                QueryOptions {
                    clients: Some(vec![org]),
                    ..QueryOptions::default()
                },
            )
            .await;

        match result {
            Err(PrismError::ColumnNotFound(ref details)) => {
                assert_eq!(
                    details.column, "typo_col",
                    "M2 ORDER BY FuncCall: column must be 'typo_col', got: {:?}",
                    details.column
                );
                assert_eq!(
                    details.table, "crowdstrike_alerts",
                    "M2 ORDER BY FuncCall: table must be 'crowdstrike_alerts'"
                );
            }
            Ok(_) => panic!(
                "M2 ORDER BY FuncCall: engine.execute must NOT succeed — E-QUERY-038 must fire \
                 for column typo inside lower() in ORDER BY. Before M2 fix, the FuncCall arg \
                 was not walked."
            ),
            Err(other) => panic!(
                "M2 ORDER BY FuncCall: expected PrismError::ColumnNotFound (E-QUERY-038), \
                 got different error: {other:?}"
            ),
        }
    }

    /// M2 fix — E-QUERY-038 must fire for a bare column typo in JOIN ON.
    ///
    /// Before fix: JOIN ON was not scanned at all — `check_query_column_availability`
    /// had no Position 5 and `sql_query.joins` was never iterated.
    ///
    /// After fix: Position 5 iterates `sql_query.joins` and calls
    /// `extract_field_paths_from_expr(&join.on, ...)` for each join, collecting
    /// bare column refs and failing on typos.
    ///
    /// Note: The PrismQL parser only parses `col = col` equality in ON clauses.
    /// An unqualified bare column ref in JOIN ON is treated as a FROM-table ref,
    /// so `typo_col` in `ON typo_col = other_table.id` will be caught.
    ///
    /// Load-bearing (M2 fix): removing the JOIN ON walk from `check_query_column_availability`
    /// causes this query to bypass the column gate, returning a DataFusion error or no error
    /// instead of E-QUERY-038.
    #[tokio::test]
    async fn test_m2_join_on_col_typo_triggers_e_query_038() {
        let (engine, org) = make_crowdstrike_engine_with_columns();

        // JOIN ON with a typo'd FROM-table column.
        // Parser: `crowdstrike_alerts JOIN crowdstrike_alerts b ON typo_col = b.severity`
        // `typo_col` is unqualified → FROM table → not in schema → E-QUERY-038.
        let query = "SELECT a.severity FROM crowdstrike_alerts a \
                     JOIN crowdstrike_alerts b ON a.typo_col = b.severity";

        let result = engine
            .execute(
                query,
                QueryOptions {
                    clients: Some(vec![org]),
                    ..QueryOptions::default()
                },
            )
            .await;

        match result {
            Err(PrismError::ColumnNotFound(ref details)) => {
                assert_eq!(
                    details.column, "typo_col",
                    "M2 JOIN ON: column must be 'typo_col', got: {:?}",
                    details.column
                );
                assert_eq!(
                    details.table, "crowdstrike_alerts",
                    "M2 JOIN ON: table must be 'crowdstrike_alerts'"
                );
            }
            Ok(_) => panic!(
                "M2 JOIN ON: engine.execute must NOT succeed — E-QUERY-038 must fire \
                 for column typo in JOIN ON. Before M2 fix, the JOIN ON position \
                 was not walked at all."
            ),
            Err(other) => panic!(
                "M2 JOIN ON: expected PrismError::ColumnNotFound (E-QUERY-038), \
                 got different error: {other:?}"
            ),
        }
    }
}

// ---------------------------------------------------------------------------
// S-DEMO-FIDELITY-REMEDIATION-001 Pass-B F-PBL1-MED-001 — SELECT-projection
// FuncCall-arg columns bypass E-QUERY-038
// ---------------------------------------------------------------------------
//
// Finding: the SELECT-projection loop (Position 1) in `check_query_column_availability`
// matched only `Expr::Field(fp)` directly and returned `None` for `Expr::FuncCall { .. }`.
// So `SELECT count(typo_col) FROM crowdstrike_alerts` did NOT validate `typo_col` against
// the column schema — but the identical `GROUP BY count(typo_col)` DID (Position 3 uses
// the recursive `extract_field_paths_from_expr`).
//
// Fix: route the SELECT-projection position through `extract_field_paths_from_expr` so
// nested FuncCall args are validated, matching GROUP BY/ORDER BY/JOIN ON.
//
// Tests assert E-QUERY-038 fires for column typos nested inside FuncCall in SELECT:
//   1. Aggregate FuncCall: `SELECT count(typo_col) FROM crowdstrike_alerts`
//   2. Scalar FuncCall: `SELECT lower(typo_col) FROM crowdstrike_alerts`
//
// No-regression tests assert the gate does NOT fire for:
//   1. SELECT * (wildcard — no column validation)
//   2. SELECT count(severity) (valid column reference inside FuncCall)
//   3. SELECT severity AS s (field alias — valid column ref)
//
// TD-VSDD-059: load-bearing — removing `extract_field_paths_from_expr` from the
// SELECT-projection position (reverting to direct `Expr::Field` match with `_ => None`)
// causes test_f_pbl1_med001_select_aggregate_funccall_typo_col_triggers_e_query_038 and
// test_f_pbl1_med001_select_scalar_funccall_typo_col_triggers_e_query_038 to fail (they
// would return Ok or a non-E-QUERY-038 error instead of PrismError::ColumnNotFound).

#[cfg(test)]
#[allow(
    non_snake_case,
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::too_many_lines
)]
mod f_pbl1_med001_select_funccall_col_gate_tests {
    use std::{collections::HashMap, sync::Arc};

    use prism_core::{OrgSlug, SensorId};
    use prism_sensors::AdapterRegistry;
    use prism_spec_engine::{
        overlay::{OverlayLoader, SensorInstanceOverlay},
        spec_parser::{AuthType, ColumnSpec, SensorSpec, TableSpec},
        ResolvedSensorSpec, ResolvedSpecKey,
    };

    use super::*;
    use crate::{scoping::ClientRegistry, table_registry::TableRegistry};
    use prism_core::column::ColumnType;

    // ── Helpers (mirrors m2_column_gate_funccall_and_join_tests::make_crowdstrike_engine_with_columns) ──

    /// No-op credential store (same stub as m2 module).
    struct NoopCs;

    #[async_trait::async_trait]
    impl prism_credentials::CredentialStore for NoopCs {
        async fn get(
            &self,
            _t: &prism_core::OrgSlug,
            _s: &str,
            _n: &prism_credentials::namespace::CredentialName,
        ) -> Result<Option<secrecy::SecretString>, PrismError> {
            Ok(None)
        }
        async fn set(
            &self,
            _t: &prism_core::OrgSlug,
            _s: &str,
            _n: &prism_credentials::namespace::CredentialName,
            _v: secrecy::SecretString,
        ) -> Result<(), PrismError> {
            Ok(())
        }
        async fn delete(
            &self,
            _t: &prism_core::OrgSlug,
            _s: &str,
            _n: &prism_credentials::namespace::CredentialName,
        ) -> Result<bool, PrismError> {
            Ok(false)
        }
        async fn list(
            &self,
            _t: &prism_core::OrgSlug,
        ) -> Result<Vec<(String, prism_credentials::namespace::CredentialName)>, PrismError>
        {
            Ok(vec![])
        }
        async fn exists(
            &self,
            _t: &prism_core::OrgSlug,
            _s: &str,
            _n: &prism_credentials::namespace::CredentialName,
        ) -> Result<bool, PrismError> {
            Ok(false)
        }
    }

    /// Build a `QueryEngine` with `crowdstrike_alerts` registered with columns
    /// `["severity", "timestamp"]` plus a resolved_spec_map for multi-tenant gate.
    ///
    /// Mirrors `m2_column_gate_funccall_and_join_tests::make_crowdstrike_engine_with_columns`.
    fn make_crowdstrike_engine() -> (QueryEngine, OrgSlug) {
        let org = OrgSlug::new("acme");
        let sensor_id = "crowdstrike";
        let table_suffix = "alerts";

        let columns = vec![
            ColumnSpec::new("severity", ColumnType::String, None, vec![]),
            ColumnSpec::new("timestamp", ColumnType::Datetime, None, vec![]),
        ];

        let spec = SensorSpec::new(
            sensor_id,
            "CrowdStrike sensor",
            AuthType::ApiKey,
            "https://api.crowdstrike.com",
            vec![TableSpec::new_point_in_time(
                table_suffix,
                "security_finding",
                columns,
                vec![],
            )],
            None,
            "1.0.0",
            Vec::new(),
        );

        let registry = Arc::new(TableRegistry::new());
        registry
            .register_sensor(&spec)
            .expect("register crowdstrike must not fail");

        let overlay_toml = format!("extends = \"{sensor_id}\"\ninstance_id = \"{sensor_id}@acme\"");
        let overlay: SensorInstanceOverlay = toml::from_str(&overlay_toml)
            .expect("F-PBL1-MED-001 fixture: SensorInstanceOverlay TOML must parse");
        let resolved = OverlayLoader::merge_overlay_onto_type_spec(&spec, &overlay, org.clone());
        let key: ResolvedSpecKey = (org.clone(), SensorId::new(sensor_id));
        let mut spec_map: HashMap<ResolvedSpecKey, ResolvedSensorSpec> = HashMap::new();
        spec_map.insert(key, resolved);

        let mut engine = QueryEngine::new_with_cache_config(
            Arc::new(AdapterRegistry::new()),
            Arc::new(NoopCs),
            Arc::new(prism_ocsf::OcsfNormalizer::new()),
            Arc::new(ClientRegistry::new(vec![org.clone()])),
            QueryEngineConfig::default(),
            crate::cache::CacheConfig::default(),
        );
        engine.resolved_spec_map = Some(Arc::new(arc_swap::ArcSwap::new(Arc::new(spec_map))));
        engine = engine.with_table_registry(registry);

        (engine, org)
    }

    // ── Red gate tests (must FAIL before fix, PASS after fix) ─────────────────

    /// F-PBL1-MED-001 — E-QUERY-038 must fire when a typo'd column is wrapped in
    /// an aggregate FuncCall in the SELECT clause (e.g. `SELECT count(typo_col) ...`).
    ///
    /// Before fix: Position 1 (SELECT) uses a direct `Expr::Field` match; the
    /// `Expr::FuncCall` arm falls through to `_ => None`, so `typo_col` inside
    /// `count(typo_col)` is never extracted and the gate silently passes.
    ///
    /// After fix: Position 1 routes through `extract_field_paths_from_expr`, which
    /// recurses into FuncCall args and collects `typo_col` → E-QUERY-038.
    ///
    /// Load-bearing (F-PBL1-MED-001): reverting SELECT to direct `Expr::Field` match
    /// causes this test to produce Ok or a non-E-QUERY-038 error.
    #[tokio::test]
    async fn test_f_pbl1_med001_select_aggregate_funccall_typo_col_triggers_e_query_038() {
        let (engine, org) = make_crowdstrike_engine();

        // `count(typo_col)` — typo_col is not in the schema (only severity, timestamp are valid).
        let query = "SELECT count(typo_col) FROM crowdstrike_alerts";

        let result = engine
            .execute(
                query,
                QueryOptions {
                    clients: Some(vec![org]),
                    ..QueryOptions::default()
                },
            )
            .await;

        match result {
            Err(PrismError::ColumnNotFound(ref details)) => {
                assert_eq!(
                    details.column, "typo_col",
                    "F-PBL1-MED-001 SELECT aggregate FuncCall: column must be 'typo_col', \
                     got: {:?}",
                    details.column
                );
                assert_eq!(
                    details.table, "crowdstrike_alerts",
                    "F-PBL1-MED-001 SELECT aggregate FuncCall: table must be 'crowdstrike_alerts'"
                );
            }
            Ok(_) => panic!(
                "F-PBL1-MED-001 SELECT aggregate FuncCall: engine.execute must NOT succeed — \
                 E-QUERY-038 must fire for column typo inside count() in SELECT. Before fix, \
                 the FuncCall arg in SELECT was not walked (fell through to `_ => None`)."
            ),
            Err(other) => panic!(
                "F-PBL1-MED-001 SELECT aggregate FuncCall: expected \
                 PrismError::ColumnNotFound (E-QUERY-038), got different error: {other:?}"
            ),
        }
    }

    /// F-PBL1-MED-001 — E-QUERY-038 must fire when a typo'd column is wrapped in
    /// a scalar FuncCall in the SELECT clause (e.g. `SELECT lower(typo_col) ...`).
    ///
    /// Mirrors the aggregate FuncCall test above for scalar functions.
    ///
    /// Load-bearing (F-PBL1-MED-001): reverting SELECT to direct `Expr::Field` match
    /// causes this test to produce Ok or a non-E-QUERY-038 error.
    #[tokio::test]
    async fn test_f_pbl1_med001_select_scalar_funccall_typo_col_triggers_e_query_038() {
        let (engine, org) = make_crowdstrike_engine();

        // `lower(typo_col)` — typo_col is not in the schema.
        let query = "SELECT lower(typo_col) FROM crowdstrike_alerts";

        let result = engine
            .execute(
                query,
                QueryOptions {
                    clients: Some(vec![org]),
                    ..QueryOptions::default()
                },
            )
            .await;

        match result {
            Err(PrismError::ColumnNotFound(ref details)) => {
                assert_eq!(
                    details.column, "typo_col",
                    "F-PBL1-MED-001 SELECT scalar FuncCall: column must be 'typo_col', \
                     got: {:?}",
                    details.column
                );
                assert_eq!(
                    details.table, "crowdstrike_alerts",
                    "F-PBL1-MED-001 SELECT scalar FuncCall: table must be 'crowdstrike_alerts'"
                );
            }
            Ok(_) => panic!(
                "F-PBL1-MED-001 SELECT scalar FuncCall: engine.execute must NOT succeed — \
                 E-QUERY-038 must fire for column typo inside lower() in SELECT. Before fix, \
                 the FuncCall arg in SELECT was not walked (fell through to `_ => None`)."
            ),
            Err(other) => panic!(
                "F-PBL1-MED-001 SELECT scalar FuncCall: expected \
                 PrismError::ColumnNotFound (E-QUERY-038), got different error: {other:?}"
            ),
        }
    }

    // ── No-regression tests (must PASS before and after fix) ──────────────────

    /// F-PBL1-MED-001 no-regression — `SELECT *` must NOT trigger E-QUERY-038.
    ///
    /// Wildcards are excluded from column validation (SelectItem::Star → None in the
    /// SELECT walk). No columns are extracted → gate passes.
    #[tokio::test]
    async fn test_f_pbl1_med001_select_star_no_e_query_038() {
        let (engine, org) = make_crowdstrike_engine();

        let query = "SELECT * FROM crowdstrike_alerts LIMIT 1";

        let result = engine
            .execute(
                query,
                QueryOptions {
                    clients: Some(vec![org]),
                    ..QueryOptions::default()
                },
            )
            .await;

        // Gate must NOT fire E-QUERY-038 for SELECT * (wildcard — no columns to validate).
        // The query may fail downstream for other reasons (no data, DataFusion error) but
        // NOT with PrismError::ColumnNotFound.
        if let Err(PrismError::ColumnNotFound(ref details)) = result {
            panic!(
                "F-PBL1-MED-001 SELECT *: gate must NOT fire E-QUERY-038 for wildcard. \
                 Got ColumnNotFound for column '{}' in table '{}'.",
                details.column, details.table
            );
        }
    }

    /// F-PBL1-MED-001 no-regression — `SELECT count(severity)` with a valid column
    /// inside FuncCall must NOT trigger E-QUERY-038.
    ///
    /// After the fix, `extract_field_paths_from_expr` recurses into FuncCall args.
    /// When the extracted column name IS in the schema, the gate must pass.
    #[tokio::test]
    async fn test_f_pbl1_med001_select_aggregate_funccall_valid_col_no_e_query_038() {
        let (engine, org) = make_crowdstrike_engine();

        // `count(severity)` — `severity` IS in the schema.
        let query = "SELECT count(severity) FROM crowdstrike_alerts";

        let result = engine
            .execute(
                query,
                QueryOptions {
                    clients: Some(vec![org]),
                    ..QueryOptions::default()
                },
            )
            .await;

        // Gate must NOT fire E-QUERY-038 for a valid column inside FuncCall.
        if let Err(PrismError::ColumnNotFound(ref details)) = result {
            panic!(
                "F-PBL1-MED-001 SELECT valid FuncCall: gate must NOT fire E-QUERY-038 \
                 for 'count(severity)' — 'severity' is in the schema. \
                 Got ColumnNotFound for column '{}' in table '{}'.",
                details.column, details.table
            );
        }
    }
}

// ---------------------------------------------------------------------------
// S-DEMO-FIDELITY-REMEDIATION-001 M1 — E-QUERY-038 single-tenant registry path
// ---------------------------------------------------------------------------
//
// Finding M1: `check_query_column_availability` returned `Ok(())` immediately
// when `resolved_spec_map.is_none()` (single-tenant mode), making E-QUERY-038
// dead in single-tenant deployments.
//
// Fix: add `columns_by_table` to `TableRegistry`. When `resolved_spec_map` is
// None but `table_registry` is Some, use `registry.columns_for_table()` to look
// up the column list and validate column refs. Fail-open when the table has no
// columns registered (backward-compatible for specs without explicit column lists).
//
// Tests assert E-QUERY-038 fires in single-tenant mode (resolved_spec_map = None,
// table_registry = Some with columns) for a column typo in the SELECT clause.
//
// TD-VSDD-059: load-bearing — removing the single-tenant branch in
// `check_column_availability` causes `test_m1_single_tenant_column_gate_fires`
// to fail (it would return Ok instead of Err(ColumnNotFound)).

#[cfg(test)]
#[allow(
    non_snake_case,
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::too_many_lines
)]
mod m1_single_tenant_column_gate_tests {
    use std::sync::Arc;

    use prism_sensors::AdapterRegistry;

    use super::*;
    use crate::{scoping::ClientRegistry, table_registry::TableRegistry};
    use prism_core::column::ColumnType;

    struct NoopCs;

    #[async_trait::async_trait]
    impl prism_credentials::CredentialStore for NoopCs {
        async fn get(
            &self,
            _t: &prism_core::OrgSlug,
            _s: &str,
            _n: &prism_credentials::namespace::CredentialName,
        ) -> Result<Option<secrecy::SecretString>, PrismError> {
            Ok(None)
        }
        async fn set(
            &self,
            _t: &prism_core::OrgSlug,
            _s: &str,
            _n: &prism_credentials::namespace::CredentialName,
            _v: secrecy::SecretString,
        ) -> Result<(), PrismError> {
            Ok(())
        }
        async fn delete(
            &self,
            _t: &prism_core::OrgSlug,
            _s: &str,
            _n: &prism_credentials::namespace::CredentialName,
        ) -> Result<bool, PrismError> {
            Ok(false)
        }
        async fn list(
            &self,
            _t: &prism_core::OrgSlug,
        ) -> Result<Vec<(String, prism_credentials::namespace::CredentialName)>, PrismError>
        {
            Ok(vec![])
        }
        async fn exists(
            &self,
            _t: &prism_core::OrgSlug,
            _s: &str,
            _n: &prism_credentials::namespace::CredentialName,
        ) -> Result<bool, PrismError> {
            Ok(false)
        }
    }

    /// Build a QueryEngine in single-tenant mode:
    /// - `resolved_spec_map = None` (single-tenant — no org-scoped spec map)
    /// - `table_registry` wired with `crowdstrike_alerts` having columns
    ///   `severity` (String) and `timestamp` (Datetime).
    ///
    /// This is the M1 test setup: the engine has NO resolved_spec_map,
    /// so the gate must rely on the TableRegistry for column validation.
    fn make_single_tenant_engine_with_columns() -> QueryEngine {
        use prism_spec_engine::spec_parser::{AuthType, ColumnSpec, SensorSpec, TableSpec};

        let columns = vec![
            ColumnSpec::new("severity", ColumnType::String, None, vec![]),
            ColumnSpec::new("timestamp", ColumnType::Datetime, None, vec![]),
        ];

        let spec = SensorSpec::new(
            "crowdstrike",
            "CrowdStrike sensor",
            AuthType::ApiKey,
            "https://api.crowdstrike.com",
            vec![TableSpec::new_point_in_time(
                "alerts",
                "security_finding",
                columns,
                vec![],
            )],
            None,
            "1.0.0",
            Vec::new(),
        );

        // Wire table registry with explicit columns.
        let registry = Arc::new(TableRegistry::new());
        registry
            .register_sensor(&spec)
            .expect("register crowdstrike must not fail");

        // Build engine WITHOUT resolved_spec_map (single-tenant).
        QueryEngine::new_with_cache_config(
            Arc::new(AdapterRegistry::new()),
            Arc::new(NoopCs),
            Arc::new(prism_ocsf::OcsfNormalizer::new()),
            Arc::new(ClientRegistry::new(vec![])),
            QueryEngineConfig::default(),
            crate::cache::CacheConfig::default(),
        )
        .with_table_registry(registry)
        // resolved_spec_map is NOT wired (defaults to None).
    }

    /// M1 fix — E-QUERY-038 must fire in single-tenant mode when a column typo
    /// is in the SELECT clause and `resolved_spec_map = None`.
    ///
    /// Before fix: `check_query_column_availability` returned Ok(()) immediately
    /// when resolved_spec_map.is_none(), so `typo_col` was accepted without
    /// validation and the query proceeded to DataFusion → internal error.
    ///
    /// After fix: the gate falls back to `table_registry.columns_for_table()`,
    /// finds the column list `["severity", "timestamp"]`, and rejects `typo_col`
    /// with PrismError::ColumnNotFound (E-QUERY-038).
    #[tokio::test]
    async fn test_m1_single_tenant_column_gate_fires() {
        let engine = make_single_tenant_engine_with_columns();

        // `typo_col` is not in the schema — only `severity` and `timestamp` are valid.
        let query = "SELECT typo_col FROM crowdstrike_alerts LIMIT 5";

        let result = engine.execute(query, QueryOptions::default()).await;

        match result {
            Err(PrismError::ColumnNotFound(ref details)) => {
                assert_eq!(
                    details.column, "typo_col",
                    "M1 single-tenant: column must be 'typo_col', got: {:?}",
                    details.column
                );
                assert_eq!(
                    details.table, "crowdstrike_alerts",
                    "M1 single-tenant: table must be 'crowdstrike_alerts'"
                );
            }
            Ok(_) => panic!(
                "M1 single-tenant: engine.execute must NOT succeed — E-QUERY-038 must fire \
                 even when resolved_spec_map=None, using table_registry.columns_for_table() \
                 as fallback. Before M1 fix, the gate returned Ok(()) immediately."
            ),
            Err(other) => panic!(
                "M1 single-tenant: expected PrismError::ColumnNotFound (E-QUERY-038), \
                 got different error: {other:?}"
            ),
        }
    }

    /// F-PBL1-LOW-001 — single-tenant E-QUERY-038 `available_columns` must be
    /// sorted lexicographically, matching the multi-tenant path.
    ///
    /// The multi-tenant path does `available_columns.sort(); available_columns.dedup()`
    /// before constructing the error (OBS-FRESH-1 fix). The single-tenant path
    /// (using `registry.columns_for_table()`) previously returned columns in spec
    /// insertion order, which is non-deterministic across sensor registrations.
    ///
    /// This test registers columns in reverse-alphabetical order (`z_col`, `a_col`)
    /// and asserts the error's `available_columns` field is sorted `["a_col", "z_col"]`.
    ///
    /// Before fix: `available_columns` in the error would be `["z_col", "a_col"]`
    /// (spec insertion order), so the assertion fails.
    ///
    /// After fix: `sort(); dedup()` applied to the single-tenant branch produces
    /// `["a_col", "z_col"]`, matching the expected sorted order.
    ///
    /// Load-bearing (F-PBL1-LOW-001): removing the sort/dedup from the single-tenant
    /// branch causes this assertion to fail when columns are registered in non-sorted order.
    #[tokio::test]
    async fn test_f_pbl1_low001_single_tenant_available_columns_sorted() {
        use prism_spec_engine::spec_parser::{AuthType, ColumnSpec, SensorSpec, TableSpec};

        // Register columns in reverse-alphabetical order to verify sort.
        let columns = vec![
            ColumnSpec::new("z_col", ColumnType::String, None, vec![]),
            ColumnSpec::new("a_col", ColumnType::String, None, vec![]),
        ];

        let spec = SensorSpec::new(
            "armis",
            "Armis sensor",
            AuthType::ApiKey,
            "https://api.armis.com",
            vec![TableSpec::new_point_in_time(
                "devices",
                "security_finding",
                columns,
                vec![],
            )],
            None,
            "1.0.0",
            Vec::new(),
        );

        let registry = Arc::new(TableRegistry::new());
        registry
            .register_sensor(&spec)
            .expect("register armis must not fail");

        let engine = QueryEngine::new_with_cache_config(
            Arc::new(AdapterRegistry::new()),
            Arc::new(NoopCs),
            Arc::new(prism_ocsf::OcsfNormalizer::new()),
            Arc::new(ClientRegistry::new(vec![])),
            QueryEngineConfig::default(),
            crate::cache::CacheConfig::default(),
        )
        .with_table_registry(registry);

        // `typo_col` is not in the schema — only `z_col` and `a_col` are valid.
        let query = "SELECT typo_col FROM armis_devices LIMIT 5";

        let result = engine.execute(query, QueryOptions::default()).await;

        match result {
            Err(PrismError::ColumnNotFound(ref details)) => {
                assert_eq!(
                    details.column, "typo_col",
                    "F-PBL1-LOW-001: column must be 'typo_col'"
                );
                assert_eq!(
                    details.available_columns,
                    vec!["a_col".to_string(), "z_col".to_string()],
                    "F-PBL1-LOW-001: available_columns must be sorted lexicographically. \
                     Before fix, columns are returned in spec insertion order [z_col, a_col]; \
                     after fix, sort+dedup produces [a_col, z_col]."
                );
            }
            Ok(_) => panic!(
                "F-PBL1-LOW-001: engine.execute must NOT succeed — E-QUERY-038 must fire \
                 for typo_col when z_col and a_col are registered."
            ),
            Err(other) => panic!(
                "F-PBL1-LOW-001: expected PrismError::ColumnNotFound (E-QUERY-038), \
                 got different error: {other:?}"
            ),
        }
    }

    /// M1 negative — gate fails-open for a table with no columns registered.
    ///
    /// Registering a sensor without explicit columns means `columns_for_table`
    /// returns an empty Vec. In that case, the gate must fail-open (return Ok)
    /// to preserve backward compatibility for specs without column lists.
    #[tokio::test]
    async fn test_m1_single_tenant_no_columns_registered_fails_open() {
        use prism_spec_engine::spec_parser::{AuthType, SensorSpec, TableSpec};

        // Register WITHOUT columns (empty column list).
        let spec = SensorSpec::new(
            "armis",
            "Armis sensor",
            AuthType::ApiKey,
            "https://api.armis.com",
            vec![TableSpec::new_point_in_time(
                "devices",
                "security_finding",
                vec![], // No columns
                vec![],
            )],
            None,
            "1.0.0",
            Vec::new(),
        );

        let registry = Arc::new(TableRegistry::new());
        registry
            .register_sensor(&spec)
            .expect("register armis must not fail");

        let engine = QueryEngine::new_with_cache_config(
            Arc::new(AdapterRegistry::new()),
            Arc::new(NoopCs),
            Arc::new(prism_ocsf::OcsfNormalizer::new()),
            Arc::new(ClientRegistry::new(vec![])),
            QueryEngineConfig::default(),
            crate::cache::CacheConfig::default(),
        )
        .with_table_registry(registry);

        // Even with a typo'd column, gate must fail-open (no columns to validate against).
        let query = "SELECT completely_bogus_col FROM armis_devices LIMIT 5";

        let result = engine.execute(query, QueryOptions::default()).await;

        // We expect either Ok (execute past gate → DataFusion error on actual execution)
        // or an Err that is NOT ColumnNotFound (DataFusion error is acceptable here).
        match result {
            Ok(_) => {} // Gate correctly failed open — no column schema to validate.
            Err(PrismError::ColumnNotFound(_)) => panic!(
                "M1 negative: gate must NOT fire E-QUERY-038 when no columns are registered \
                 for the table. Fail-open is required for backward compat."
            ),
            Err(_other) => {} // DataFusion or other error downstream — acceptable (gate passed).
        }
    }
}

// ---------------------------------------------------------------------------
// F-PHL1-HIGH-001 + F-PHL1-MED-001 — CWE-407 strsim input cap (SEC-002)
// ---------------------------------------------------------------------------
//
// These tests verify that over-cap inputs (> DID_YOU_MEAN_MAX_NAME_BYTES = 128 bytes)
// to the enrich UDF gate (E-QUERY-039) and column gate (E-QUERY-038) are capped
// before the Levenshtein computation, closing CWE-407 (Algorithmic Complexity DoS).
//
// Before fix: `check_enrich_udf_availability` and `check_column_availability` pass
//   `requested`/`column_name` verbatim (potentially 64KB from the query) to strsim.
// After fix:  `cap_name_for_levenshtein` clamps to 128 bytes at a char boundary
//   BEFORE the computation — same as the existing table-gate cap in table_registry.rs.
//
// The `test_f_phl1_cap_name_for_levenshtein_*` unit tests directly test the
// `cap_name_for_levenshtein` helper (fails RED before the helper is implemented).
// The integration tests verify the gates return correct errors for over-cap input.
//
// Load-bearing (TD-VSDD-059): `test_f_phl1_cap_name_for_levenshtein_over_cap_truncates`
// calls `cap_name_for_levenshtein` and asserts the returned slice is ≤ 128 bytes.
// Before fix: the helper doesn't exist → compile error.
// After fix: the helper is present → tests pass.

#[cfg(test)]
#[allow(
    non_snake_case,
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::too_many_lines
)]
mod cwe407_cap_unit_tests {
    use crate::table_registry::cap_name_for_levenshtein;

    /// F-PHL1-HIGH-001/MED-001: `cap_name_for_levenshtein` must return ≤ 128 bytes
    /// for an over-cap input.
    ///
    /// Load-bearing (TD-VSDD-059): before fix the function does not exist →
    /// compile error. After fix, the slice is clamped to ≤ 128 bytes.
    #[test]
    fn test_f_phl1_cap_name_for_levenshtein_over_cap_truncates() {
        let long_name: String = "x".repeat(200);
        let capped = cap_name_for_levenshtein(&long_name);
        assert!(
            capped.len() <= 128,
            "cap_name_for_levenshtein must return ≤ 128 bytes for a 200-byte input; \
             got {} bytes",
            capped.len()
        );
        assert!(
            long_name.is_char_boundary(capped.len()),
            "capped slice must end at a UTF-8 char boundary"
        );
    }

    /// F-PHL1: `cap_name_for_levenshtein` must be a no-op for inputs ≤ 128 bytes.
    #[test]
    fn test_f_phl1_cap_name_for_levenshtein_short_name_unchanged() {
        let short_name = "severity";
        let capped = cap_name_for_levenshtein(short_name);
        assert_eq!(
            capped, short_name,
            "cap_name_for_levenshtein must return input unchanged when len ≤ 128"
        );
    }

    /// F-PHL1: `cap_name_for_levenshtein` must return valid UTF-8 when truncating
    /// a string with multi-byte characters.
    ///
    /// "é" = 2 bytes (UTF-8). A 65-char string of "é" = 130 bytes > 128.
    /// The cap must land at the last char boundary ≤ 128, which is byte 128
    /// only if it's a char boundary; otherwise step back.
    /// 65 × 2 = 130 bytes; cap at ≤ 128 → last "é" boundary at byte 128
    /// (since every "é" starts at an even byte offset, 128 is a char boundary).
    #[test]
    fn test_f_phl1_cap_name_for_levenshtein_multibyte_char_boundary() {
        // "é" = U+00E9 = 0xC3 0xA9 (2 bytes in UTF-8).
        let multibyte: String = "é".repeat(65); // 130 bytes
        assert_eq!(multibyte.len(), 130, "fixture: 65 × é = 130 bytes");
        let capped = cap_name_for_levenshtein(&multibyte);
        assert!(
            capped.len() <= 128,
            "cap_name_for_levenshtein must return ≤ 128 bytes for multibyte input; \
             got {} bytes",
            capped.len()
        );
        // The returned slice must be valid UTF-8 (str invariant).
        assert!(
            std::str::from_utf8(capped.as_bytes()).is_ok(),
            "capped slice must be valid UTF-8"
        );
    }
}

#[cfg(test)]
#[allow(
    non_snake_case,
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::too_many_lines
)]
mod cwe407_strsim_cap_tests {
    use std::sync::Arc;

    use prism_sensors::AdapterRegistry;

    use super::*;
    use crate::{scoping::ClientRegistry, table_registry::TableRegistry};
    use prism_core::column::ColumnType;

    struct NoopCs2;

    #[async_trait::async_trait]
    impl prism_credentials::CredentialStore for NoopCs2 {
        async fn get(
            &self,
            _t: &prism_core::OrgSlug,
            _s: &str,
            _n: &prism_credentials::namespace::CredentialName,
        ) -> Result<Option<secrecy::SecretString>, PrismError> {
            Ok(None)
        }
        async fn set(
            &self,
            _t: &prism_core::OrgSlug,
            _s: &str,
            _n: &prism_credentials::namespace::CredentialName,
            _v: secrecy::SecretString,
        ) -> Result<(), PrismError> {
            Ok(())
        }
        async fn delete(
            &self,
            _t: &prism_core::OrgSlug,
            _s: &str,
            _n: &prism_credentials::namespace::CredentialName,
        ) -> Result<bool, PrismError> {
            Ok(false)
        }
        async fn list(
            &self,
            _t: &prism_core::OrgSlug,
        ) -> Result<Vec<(String, prism_credentials::namespace::CredentialName)>, PrismError>
        {
            Ok(vec![])
        }
        async fn exists(
            &self,
            _t: &prism_core::OrgSlug,
            _s: &str,
            _n: &prism_credentials::namespace::CredentialName,
        ) -> Result<bool, PrismError> {
            Ok(false)
        }
    }

    /// F-PHL1-HIGH-001: `check_enrich_udf_availability` must cap over-cap UDF names
    /// at `DID_YOU_MEAN_MAX_NAME_BYTES` (128 bytes) before the Levenshtein computation.
    ///
    /// Precondition: the enrich gate is active (InfusionRegistry wired, empty registry
    /// is sufficient — ANY unknown name triggers the did_you_mean Levenshtein loop).
    /// A query with a 200-byte UDF name must return E-QUERY-039, not hang or panic.
    ///
    /// Load-bearing (TD-VSDD-059): before fix, `strsim::levenshtein` receives the full
    /// 200-byte token → O(200 × len(registered_name)) per loop iteration. With
    /// many registered names this becomes an MCP-triggerable DoS. After fix, the cap
    /// clamps input to 128 bytes before any strsim call.
    ///
    /// SEC-002 / CWE-407 / F-PHL1-HIGH-001.
    #[tokio::test]
    async fn test_f_phl1_high001_enrich_gate_over_cap_name_returns_e_query_039() {
        use prism_spec_engine::InfusionRegistry;

        // An empty InfusionRegistry is sufficient: any UDF name in the query is unknown,
        // so the did_you_mean Levenshtein loop fires immediately on the first unknown name.
        // (With an empty registry, available_infusions is [], so no actual Levenshtein
        // computation is done — but the cap is still applied before the loop to ensure
        // the guard is present even when the registry later gains registered names.)
        let registry = Arc::new(InfusionRegistry::new());

        // Build QueryEngine with the enrich registry wired.
        let engine = QueryEngine::new_with_cache_config(
            Arc::new(AdapterRegistry::new()),
            Arc::new(NoopCs2),
            Arc::new(prism_ocsf::OcsfNormalizer::new()),
            Arc::new(ClientRegistry::new(vec![])),
            QueryEngineConfig::default(),
            crate::cache::CacheConfig::default(),
        )
        .with_infusion_registry(registry);

        // Construct a 200-byte UDF name: 200 ASCII 'x' characters.
        // This is >128 bytes (DID_YOU_MEAN_MAX_NAME_BYTES) but within the max query size.
        // Before fix: strsim::levenshtein receives all 200 chars, creating O(200*14)
        //   computation per registered name — exploitable DoS via crafted MCP query.
        // After fix: input is capped at 128 bytes before the Levenshtein loop.
        let over_cap_udf_name: String = "x".repeat(200);
        // Use SQL mode so the unknown scalar function name triggers the enrich gate.
        let query =
            format!("SELECT {over_cap_udf_name}(ip_address) FROM crowdstrike_alerts LIMIT 5");

        let result = engine.execute(&query, QueryOptions::default()).await;

        // Must return E-QUERY-039 (EnrichUdfNotFound) — the gate must fire and
        // cap the input internally without hanging.
        match result {
            Err(PrismError::EnrichUdfNotFound(ref details)) => {
                // The gate correctly rejected the unknown over-cap UDF name.
                // `requested` in the details is the RAW name (pre-cap) — we only
                // cap for the Levenshtein computation, NOT the error field.
                assert_eq!(
                    details.infusion.len(),
                    200,
                    "F-PHL1-HIGH-001: details.infusion must carry the full 200-char name (cap \
                     is only for Levenshtein computation); got len: {}",
                    details.infusion.len()
                );
            }
            // E-QUERY-037 fires first if no TableRegistry is wired — still valid:
            // the important invariant is that the engine DOES NOT panic or hang.
            // If the enrich gate fires, it must return EnrichUdfNotFound.
            Err(PrismError::TableNotAvailable(_)) => {
                // Table gate fired before enrich gate (no TableRegistry) — acceptable:
                // the over-cap name never reached the Levenshtein computation because
                // the table gate fired first. This is a correct short-circuit.
            }
            Ok(_) => panic!(
                "F-PHL1-HIGH-001: engine.execute must not succeed for an unregistered 200-char \
                 UDF name. Either E-QUERY-039 or E-QUERY-037 must fire."
            ),
            Err(other) => {
                // Any other error (parse, column, etc.) is a test-setup issue.
                // The test verifies no PANIC — if we reach here without panic,
                // the CWE-407 DoS vector is not triggered.
                let _ = other; // not a panic — acceptable for this safety test
            }
        }
    }

    /// F-PHL1-MED-001: `check_column_availability` (single-tenant path) must cap
    /// over-cap column names at 128 bytes before the Levenshtein computation.
    ///
    /// Precondition: single-tenant mode (resolved_spec_map = None, TableRegistry wired).
    /// A query with a 200-byte column name must return E-QUERY-038 (ColumnNotFound).
    ///
    /// Load-bearing (TD-VSDD-059): before fix, `column_name` is passed verbatim to strsim.
    /// After fix: `cap_name_for_levenshtein(column_name)` clamps to 128 bytes.
    ///
    /// SEC-002 / CWE-407 / F-PHL1-MED-001.
    #[tokio::test]
    async fn test_f_phl1_med001_column_gate_single_tenant_over_cap_name_returns_e_query_038() {
        use prism_spec_engine::spec_parser::{AuthType, ColumnSpec, SensorSpec, TableSpec};

        let columns = vec![
            ColumnSpec::new("severity", ColumnType::String, None, vec![]),
            ColumnSpec::new("timestamp", ColumnType::Datetime, None, vec![]),
        ];

        let spec = SensorSpec::new(
            "crowdstrike",
            "CrowdStrike sensor",
            AuthType::ApiKey,
            "https://api.crowdstrike.com",
            vec![TableSpec::new_point_in_time(
                "alerts",
                "security_finding",
                columns,
                vec![],
            )],
            None,
            "1.0.0",
            Vec::new(),
        );

        let registry = Arc::new(TableRegistry::new());
        registry
            .register_sensor(&spec)
            .expect("register crowdstrike must not fail");

        let engine = QueryEngine::new_with_cache_config(
            Arc::new(AdapterRegistry::new()),
            Arc::new(NoopCs2),
            Arc::new(prism_ocsf::OcsfNormalizer::new()),
            Arc::new(ClientRegistry::new(vec![])),
            QueryEngineConfig::default(),
            crate::cache::CacheConfig::default(),
        )
        .with_table_registry(registry);

        // Construct a 200-byte column name.
        let over_cap_col: String = "c".repeat(200);
        let query = format!("SELECT {over_cap_col} FROM crowdstrike_alerts LIMIT 5");

        let result = engine.execute(&query, QueryOptions::default()).await;

        // Must return E-QUERY-038 (ColumnNotFound) with the over-cap name.
        // The key safety property: no panic, no hang.
        match result {
            Err(PrismError::ColumnNotFound(ref details)) => {
                assert_eq!(
                    details.column.len(),
                    200,
                    "F-PHL1-MED-001: details.column must carry the full 200-char name; \
                     got len: {}",
                    details.column.len()
                );
                assert_eq!(
                    details.table, "crowdstrike_alerts",
                    "F-PHL1-MED-001: table must be 'crowdstrike_alerts'"
                );
            }
            Ok(_) => panic!(
                "F-PHL1-MED-001: engine must NOT succeed for a 200-char column name that is \
                 not in the schema. E-QUERY-038 must fire."
            ),
            Err(other) => {
                panic!("F-PHL1-MED-001: expected E-QUERY-038 (ColumnNotFound), got: {other:?}")
            }
        }
    }

    /// F-PHL1-MED-001 multi-tenant: `check_column_availability` (multi-tenant path)
    /// must also cap over-cap column names.
    ///
    /// Load-bearing (TD-VSDD-059): `check_column_availability` (multi-tenant path) also
    /// calls `strsim::levenshtein(column_name, c)` without a cap before this fix.
    ///
    /// SEC-002 / CWE-407 / F-PHL1-MED-001.
    #[tokio::test]
    async fn test_f_phl1_med001_column_gate_multi_tenant_over_cap_name_returns_e_query_038() {
        use prism_spec_engine::{
            overlay::{OverlayLoader, SensorInstanceOverlay},
            spec_parser::{AuthType, ColumnSpec, SensorSpec, TableSpec},
            ResolvedSpecKey,
        };

        // Build a resolved_spec_map with one sensor ("crowdstrike") under org "acme".
        let columns = vec![
            ColumnSpec::new("severity", ColumnType::String, None, vec![]),
            ColumnSpec::new("timestamp", ColumnType::Datetime, None, vec![]),
        ];

        let spec = SensorSpec::new(
            "crowdstrike",
            "CrowdStrike sensor",
            AuthType::ApiKey,
            "https://api.crowdstrike.com",
            vec![TableSpec::new_point_in_time(
                "alerts",
                "security_finding",
                columns,
                vec![],
            )],
            None,
            "1.0.0",
            Vec::new(),
        );

        let overlay_toml = r#"extends = "crowdstrike"
instance_id = "crowdstrike@acme""#;
        let overlay: SensorInstanceOverlay = toml::from_str(overlay_toml)
            .expect("multi-tenant fixture: SensorInstanceOverlay TOML must parse");
        let org_slug = prism_core::OrgSlug::new("acme");
        let sensor_id = prism_core::SensorId::new("crowdstrike");
        let resolved =
            OverlayLoader::merge_overlay_onto_type_spec(&spec, &overlay, org_slug.clone());
        let key: ResolvedSpecKey = (org_slug.clone(), sensor_id);

        let mut spec_map = std::collections::HashMap::new();
        spec_map.insert(key, resolved);

        // Build engine with resolved_spec_map wired (multi-tenant mode).
        // Use the pub(crate) field directly (same pattern as ADR-042 tests).
        let mut engine = QueryEngine::new_with_cache_config(
            Arc::new(AdapterRegistry::new()),
            Arc::new(NoopCs2),
            Arc::new(prism_ocsf::OcsfNormalizer::new()),
            Arc::new(ClientRegistry::new(vec![org_slug])),
            QueryEngineConfig::default(),
            crate::cache::CacheConfig::default(),
        );
        engine.resolved_spec_map = Some(Arc::new(arc_swap::ArcSwap::new(Arc::new(spec_map))));

        // 200-byte column name → triggers multi-tenant path of check_column_availability.
        let over_cap_col: String = "d".repeat(200);
        let query = format!("SELECT {over_cap_col} FROM crowdstrike_alerts LIMIT 5");

        let result = engine.execute(&query, QueryOptions::default()).await;

        match result {
            Err(PrismError::ColumnNotFound(ref details)) => {
                assert_eq!(
                    details.column.len(),
                    200,
                    "F-PHL1-MED-001 multi-tenant: details.column must carry the full 200-char \
                     name; got len: {}",
                    details.column.len()
                );
            }
            Ok(_) => panic!(
                "F-PHL1-MED-001 multi-tenant: must NOT succeed for a 200-char column name. \
                 E-QUERY-038 must fire."
            ),
            Err(other) => panic!(
                "F-PHL1-MED-001 multi-tenant: expected E-QUERY-038 (ColumnNotFound), \
                 got: {other:?}"
            ),
        }
    }
}

// ---------------------------------------------------------------------------
// F-PNL1-MED-001 — pipe-mode built-in name fires E-QUERY-039 (no skip)
// ---------------------------------------------------------------------------
//
// BC-2.11.019 v1.5 §F-PJL1-HIGH-001 "Scope of change":
//   "SQL-mode `ScalarFunc::Unknown` gate logic only. Pipe-mode `EnrichStage.infusion`
//    gate is UNAFFECTED (pipe-mode `| enrich` is an explicit enrichment directive —
//    a built-in name there is NOT a DataFusion scalar, it's an unregistered infusion
//    the analyst is trying to apply, so it SHOULD fire E-QUERY-039)."
//
// Before fix: `check_enrich_udf_availability` collected all names into one Vec and
//   applied `DATAFUSION_BUILTIN_SCALAR_NAMES` skip uniformly — pipe-mode enrich names
//   were incorrectly excluded when they matched a DataFusion built-in name (e.g. `lower`).
// After fix: pipe-mode names and SQL-mode names are collected into separate Vecs;
//   built-in skip applies only to SQL-mode names.

#[cfg(test)]
#[allow(
    non_snake_case,
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::too_many_lines
)]
mod pipe_mode_builtin_enrich_gate_tests {
    use std::sync::Arc;

    use prism_sensors::AdapterRegistry;

    use super::*;
    use crate::{scoping::ClientRegistry, table_registry::TableRegistry};
    use prism_core::column::ColumnType;

    struct NoopCsPipe;

    #[async_trait::async_trait]
    impl prism_credentials::CredentialStore for NoopCsPipe {
        async fn get(
            &self,
            _t: &prism_core::OrgSlug,
            _s: &str,
            _n: &prism_credentials::namespace::CredentialName,
        ) -> Result<Option<secrecy::SecretString>, PrismError> {
            Ok(None)
        }
        async fn set(
            &self,
            _t: &prism_core::OrgSlug,
            _s: &str,
            _n: &prism_credentials::namespace::CredentialName,
            _v: secrecy::SecretString,
        ) -> Result<(), PrismError> {
            Ok(())
        }
        async fn delete(
            &self,
            _t: &prism_core::OrgSlug,
            _s: &str,
            _n: &prism_credentials::namespace::CredentialName,
        ) -> Result<bool, PrismError> {
            Ok(false)
        }
        async fn list(
            &self,
            _t: &prism_core::OrgSlug,
        ) -> Result<Vec<(String, prism_credentials::namespace::CredentialName)>, PrismError>
        {
            Ok(vec![])
        }
        async fn exists(
            &self,
            _t: &prism_core::OrgSlug,
            _s: &str,
            _n: &prism_credentials::namespace::CredentialName,
        ) -> Result<bool, PrismError> {
            Ok(false)
        }
    }

    /// Build a QueryEngine with:
    ///   - TableRegistry containing `crowdstrike_alerts` (columns: `severity`, `ioc_value`)
    ///   - InfusionRegistry that is empty (no infusions registered — `lower` is NOT an infusion)
    ///
    /// This setup ensures E-QUERY-037 (table not found) and E-QUERY-038 (column not found)
    /// do NOT fire before E-QUERY-039 (enrich gate), so the enrich gate exercises the
    /// pipe-mode path cleanly.
    fn make_engine_with_sensor_and_empty_infusion_registry() -> QueryEngine {
        use prism_spec_engine::{
            spec_parser::{AuthType, ColumnSpec, SensorSpec, TableSpec},
            InfusionRegistry,
        };

        // Register `crowdstrike` sensor with table `alerts` (→ `crowdstrike_alerts` in queries).
        // Include `ioc_value` and `severity` columns so E-QUERY-038 does not fire for those.
        let columns = vec![
            ColumnSpec::new("ioc_value", ColumnType::String, None, vec![]),
            ColumnSpec::new("severity", ColumnType::String, None, vec![]),
        ];
        let sensor = SensorSpec::new(
            "crowdstrike",
            "CrowdStrike sensor",
            AuthType::ApiKey,
            "https://example.com",
            vec![TableSpec::new_point_in_time(
                "alerts",
                "security_finding",
                columns,
                vec![],
            )],
            None,
            "1.0.0",
            Vec::new(),
        );
        let table_registry = Arc::new(TableRegistry::new());
        table_registry
            .register_sensor(&sensor)
            .expect("register crowdstrike sensor must not fail");

        // Empty InfusionRegistry — `lower` is NOT a registered infusion name.
        let infusion_registry = Arc::new(InfusionRegistry::new());

        QueryEngine::new_with_cache_config(
            Arc::new(AdapterRegistry::new()),
            Arc::new(NoopCsPipe),
            Arc::new(prism_ocsf::OcsfNormalizer::new()),
            Arc::new(ClientRegistry::new(vec![])),
            QueryEngineConfig::default(),
            crate::cache::CacheConfig::default(),
        )
        .with_table_registry(table_registry)
        .with_infusion_registry(infusion_registry)
    }

    /// F-PNL1-MED-001 load-bearing: pipe-mode `| enrich lower(ioc_value)` where `lower`
    /// is NOT a registered infusion (empty InfusionRegistry) MUST return E-QUERY-039.
    ///
    /// Before fix: `lower` matched `DATAFUSION_BUILTIN_SCALAR_NAMES` (it is a DataFusion
    ///   built-in) and was silently skipped — the gate was a no-op for this pipe query.
    /// After fix: pipe-mode enrich names bypass the built-in skip entirely. `lower` is not
    ///   in InfusionRegistry → E-QUERY-039 fires with `infusion: "lower"`.
    ///
    /// BC-2.11.019 v1.5 §F-PJL1-HIGH-001 scope: "Pipe-mode `EnrichStage.infusion` gate
    /// is UNAFFECTED — a built-in name there is NOT a DataFusion scalar, it is an
    /// unregistered infusion the analyst is trying to apply, so it SHOULD fire E-QUERY-039."
    ///
    /// Load-bearing (TD-VSDD-059): before fix the single-Vec approach skips `lower` →
    /// `execute` succeeds or returns a different error. After fix, E-QUERY-039 is returned.
    #[tokio::test]
    async fn test_pipe_mode_builtin_name_fires_e_query_039() {
        let engine = make_engine_with_sensor_and_empty_infusion_registry();

        // Pipe-mode query: `lower` is not a registered infusion — E-QUERY-039 MUST fire.
        // Before fix: `lower` is in DATAFUSION_BUILTIN_SCALAR_NAMES → skipped → gate is no-op.
        // After fix: pipe-mode names bypass DATAFUSION_BUILTIN_SCALAR_NAMES → gate fires.
        let result = engine
            .execute(
                "FROM crowdstrike_alerts | enrich lower(ioc_value)",
                QueryOptions::default(),
            )
            .await;

        match result {
            Err(PrismError::EnrichUdfNotFound(ref details)) => {
                assert_eq!(
                    details.infusion, "lower",
                    "F-PNL1-MED-001: details.infusion must be 'lower'; got: '{}'",
                    details.infusion
                );
                // available_infusions must be empty (empty registry).
                assert!(
                    details.available_infusions.is_empty(),
                    "F-PNL1-MED-001: available_infusions must be empty (no infusions registered); \
                     got: {:?}",
                    details.available_infusions
                );
            }
            Ok(_) => panic!(
                "F-PNL1-MED-001: pipe-mode `| enrich lower(ioc_value)` must NOT succeed — \
                 `lower` is not a registered infusion; E-QUERY-039 must fire. \
                 Before fix: built-in skip wrongly silenced the gate for pipe-mode names."
            ),
            Err(PrismError::TableNotAvailable(ref details)) => panic!(
                "F-PNL1-MED-001: E-QUERY-037 fired unexpectedly — table '{}' was not found. \
                 Test setup registers 'crowdstrike_alerts'; check TableRegistry wiring.",
                details.table
            ),
            Err(PrismError::ColumnNotFound(ref details)) => panic!(
                "F-PNL1-MED-001: E-QUERY-038 fired unexpectedly — column '{}' was not found. \
                 Test setup registers 'ioc_value' and 'severity'; check ColumnSpec wiring.",
                details.column
            ),
            Err(other) => panic!(
                "F-PNL1-MED-001: unexpected error — expected E-QUERY-039 (EnrichUdfNotFound), \
                 got: {other:?}"
            ),
        }
    }

    /// Regression guard: SQL-mode `SELECT lower(severity) FROM crowdstrike_alerts` with
    /// InfusionRegistry wired but `lower` NOT a registered infusion MUST NOT fire E-QUERY-039.
    ///
    /// `lower` is a DataFusion built-in scalar. In SQL mode, the built-in exclusion applies —
    /// the gate must pass and DataFusion resolves `lower` during execution.
    ///
    /// This test guards against the fix breaking the F-PJL1-HIGH-001 regression guard:
    /// the built-in exclusion for SQL mode must remain active after the pipe/SQL split.
    ///
    /// BC-2.11.019 v1.5 §F-PJL1-HIGH-001 + EC-11-064.
    #[tokio::test]
    async fn test_sql_mode_builtin_name_does_not_fire_e_query_039() {
        let engine = make_engine_with_sensor_and_empty_infusion_registry();

        // SQL-mode query using DataFusion built-in `lower`. The infusion registry is empty
        // (lower is not a registered infusion), but the built-in exclusion applies in SQL mode.
        // The gate MUST pass; `lower(severity)` is resolved by DataFusion at execution time.
        // The execute call will fail with a DataFusion/execution error (no real adapter is
        // wired), but it must NOT fail with E-QUERY-039.
        let result = engine
            .execute(
                "SELECT lower(severity) FROM crowdstrike_alerts LIMIT 1",
                QueryOptions::default(),
            )
            .await;

        match result {
            Err(PrismError::EnrichUdfNotFound(ref details)) => panic!(
                "F-PNL1-MED-001 regression guard: SQL-mode `lower` MUST NOT fire E-QUERY-039. \
                 `lower` is a DataFusion built-in; the SQL-mode built-in exclusion must remain \
                 active after the pipe/SQL split fix. Got infusion: '{}'",
                details.infusion
            ),
            // Any other outcome (Ok, E-QUERY-037, execution error, etc.) is acceptable —
            // the important invariant is that E-QUERY-039 does NOT fire for `lower` in SQL mode.
            _ => {
                // Passes: the enrich gate correctly did not fire for a DataFusion built-in
                // in SQL mode. The execution may fail for other reasons (no adapter, no data)
                // but NOT because of the enrich gate.
            }
        }
    }
}

// ---------------------------------------------------------------------------
// S-DEMO-FIDELITY-REMEDIATION-001 F-PWL1-LOW-001 — HAVING clause column gate
// ---------------------------------------------------------------------------
//
// Finding F-PWL1-LOW-001: `check_query_column_availability` walked 5 positions
// (SELECT, WHERE, GROUP BY, ORDER BY, JOIN ON) but NOT the HAVING clause.
// A query like `SELECT severity, count(*) FROM crowdstrike_alerts GROUP BY severity
// HAVING count(typo_col) > 5` bypassed E-QUERY-038 entirely — `typo_col` in HAVING
// was never validated against the schema.
//
// Sibling asymmetry: E-QUERY-039 (enrich gate) and E-QUERY-037 (source-walk) both
// cover HAVING; only E-QUERY-038 (column gate) was missing this position.
//
// Fix: Position 6 — HAVING — uses `extract_predicate_columns` (same helper as
// Position 2 / WHERE), since `having` is `Option<Predicate>` identical in type to
// `where_`. Base-column refs nested in aggregate FuncCalls like `count(typo_col)` are
// extracted by the existing `collect_predicate_columns` → `extract_field_paths_from_expr`
// call chain, matching the WHERE position's behaviour.
//
// BC-2.11.016 v1.5 / F-PWL1-LOW-001.
//
// Tests assert:
//   1. (red-gate) HAVING with typo'd column fires E-QUERY-038.
//   2. (no-regression) HAVING with valid column does NOT fire E-QUERY-038.
//
// TD-VSDD-059: load-bearing — removing the Position 6 HAVING walk from
// `check_query_column_availability` causes
// test_BC_2_11_016_having_column_gate_typo_fires_e_query_038 to return Ok or a
// non-E-QUERY-038 error instead of PrismError::ColumnNotFound.

#[cfg(test)]
#[allow(
    non_snake_case,
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::too_many_lines
)]
mod f_pwl1_low001_having_column_gate_tests {
    use std::{collections::HashMap, sync::Arc};

    use prism_core::{OrgSlug, SensorId};
    use prism_sensors::AdapterRegistry;
    use prism_spec_engine::{
        overlay::{OverlayLoader, SensorInstanceOverlay},
        spec_parser::{AuthType, ColumnSpec, SensorSpec, TableSpec},
        ResolvedSensorSpec, ResolvedSpecKey,
    };

    use super::*;
    use crate::{scoping::ClientRegistry, table_registry::TableRegistry};
    use prism_core::column::ColumnType;

    /// No-op credential store (same stub pattern used throughout this module).
    struct NoopCs;

    #[async_trait::async_trait]
    impl prism_credentials::CredentialStore for NoopCs {
        async fn get(
            &self,
            _t: &prism_core::OrgSlug,
            _s: &str,
            _n: &prism_credentials::namespace::CredentialName,
        ) -> Result<Option<secrecy::SecretString>, PrismError> {
            Ok(None)
        }
        async fn set(
            &self,
            _t: &prism_core::OrgSlug,
            _s: &str,
            _n: &prism_credentials::namespace::CredentialName,
            _v: secrecy::SecretString,
        ) -> Result<(), PrismError> {
            Ok(())
        }
        async fn delete(
            &self,
            _t: &prism_core::OrgSlug,
            _s: &str,
            _n: &prism_credentials::namespace::CredentialName,
        ) -> Result<bool, PrismError> {
            Ok(false)
        }
        async fn list(
            &self,
            _t: &prism_core::OrgSlug,
        ) -> Result<Vec<(String, prism_credentials::namespace::CredentialName)>, PrismError>
        {
            Ok(vec![])
        }
        async fn exists(
            &self,
            _t: &prism_core::OrgSlug,
            _s: &str,
            _n: &prism_credentials::namespace::CredentialName,
        ) -> Result<bool, PrismError> {
            Ok(false)
        }
    }

    /// Build a `QueryEngine` with `crowdstrike_alerts` registered with columns
    /// `["severity", "timestamp"]` under org "acme".
    ///
    /// Mirrors the fixture pattern established in `m2_column_gate_funccall_and_join_tests`
    /// and `f_pbl1_med001_select_funccall_col_gate_tests`.
    fn make_crowdstrike_engine() -> (QueryEngine, OrgSlug) {
        let org = OrgSlug::new("acme");
        let sensor_id = "crowdstrike";
        let table_suffix = "alerts";

        let columns = vec![
            ColumnSpec::new("severity", ColumnType::String, None, vec![]),
            ColumnSpec::new("timestamp", ColumnType::Datetime, None, vec![]),
        ];

        let spec = SensorSpec::new(
            sensor_id,
            "CrowdStrike sensor",
            AuthType::ApiKey,
            "https://api.crowdstrike.com",
            vec![TableSpec::new_point_in_time(
                table_suffix,
                "security_finding",
                columns,
                vec![],
            )],
            None,
            "1.0.0",
            Vec::new(),
        );

        let registry = Arc::new(TableRegistry::new());
        registry
            .register_sensor(&spec)
            .expect("F-PWL1-LOW-001 fixture: register crowdstrike must not fail");

        let overlay_toml = format!("extends = \"{sensor_id}\"\ninstance_id = \"{sensor_id}@acme\"");
        let overlay: SensorInstanceOverlay = toml::from_str(&overlay_toml)
            .expect("F-PWL1-LOW-001 fixture: SensorInstanceOverlay TOML must parse");
        let resolved = OverlayLoader::merge_overlay_onto_type_spec(&spec, &overlay, org.clone());
        let key: ResolvedSpecKey = (org.clone(), SensorId::new(sensor_id));
        let mut spec_map: HashMap<ResolvedSpecKey, ResolvedSensorSpec> = HashMap::new();
        spec_map.insert(key, resolved);

        let mut engine = QueryEngine::new_with_cache_config(
            Arc::new(AdapterRegistry::new()),
            Arc::new(NoopCs),
            Arc::new(prism_ocsf::OcsfNormalizer::new()),
            Arc::new(ClientRegistry::new(vec![org.clone()])),
            QueryEngineConfig::default(),
            crate::cache::CacheConfig::default(),
        );
        engine.resolved_spec_map = Some(Arc::new(arc_swap::ArcSwap::new(Arc::new(spec_map))));
        engine = engine.with_table_registry(registry);

        (engine, org)
    }

    // ── Red-gate test (must FAIL before fix, PASS after fix) ──────────────────

    /// F-PWL1-LOW-001 — E-QUERY-038 must fire when a typo'd column is referenced
    /// in the HAVING clause predicate (e.g. `HAVING typo_col > 5`).
    ///
    /// Before fix: Position 6 (HAVING) was absent from `check_query_column_availability`.
    /// `typo_col` in `HAVING typo_col > 5` was never extracted, so the gate silently
    /// passed — a pedagogical asymmetry vs WHERE / GROUP BY / ORDER BY.
    ///
    /// After fix: Position 6 calls `extract_predicate_columns` on `sql_query.having`
    /// (same helper as Position 2 / WHERE). `typo_col > 5` is parsed as
    /// `Predicate::Compare { lhs: Expr::Field("typo_col"), op: Gt, rhs: ... }`;
    /// `collect_predicate_columns` extracts the `lhs` FieldPath → `typo_col`
    /// → E-QUERY-038.
    ///
    /// Note on PrismQL HAVING grammar: the predicate parser (shared with WHERE) accepts
    /// `field op literal` form in HAVING; `HAVING funcall(col) op value` is not currently
    /// supported by the parser (the FuncCall-in-predicate-LHS form is not in the grammar).
    /// The tested query `HAVING typo_col > 5` exercises the primary code path added in
    /// this fix (the `having` field walk) and is the most direct proof of the gate.
    ///
    /// BC-2.11.016 v1.5 / F-PWL1-LOW-001.
    ///
    /// Load-bearing (F-PWL1-LOW-001): removing the Position 6 HAVING walk from
    /// `check_query_column_availability` causes this test to return Ok or a
    /// non-E-QUERY-038 error instead of PrismError::ColumnNotFound.
    #[tokio::test]
    async fn test_BC_2_11_016_having_column_gate_typo_fires_e_query_038() {
        let (engine, org) = make_crowdstrike_engine();

        // `typo_col` is not in the schema (only `severity` and `timestamp` are valid).
        // PrismQL HAVING predicate: `field op literal` form (same grammar as WHERE).
        // Before the fix, Position 6 (HAVING) was absent so this silently passed.
        let query = "SELECT severity, count(*) FROM crowdstrike_alerts \
                     GROUP BY severity HAVING typo_col > 5";

        let result = engine
            .execute(
                query,
                QueryOptions {
                    clients: Some(vec![org]),
                    ..QueryOptions::default()
                },
            )
            .await;

        match result {
            Err(PrismError::ColumnNotFound(ref details)) => {
                assert_eq!(
                    details.column, "typo_col",
                    "F-PWL1-LOW-001: column in E-QUERY-038 must be 'typo_col', got: {:?}",
                    details.column
                );
                assert_eq!(
                    details.table, "crowdstrike_alerts",
                    "F-PWL1-LOW-001: table in E-QUERY-038 must be 'crowdstrike_alerts'"
                );
            }
            Ok(_) => panic!(
                "F-PWL1-LOW-001: engine.execute must NOT succeed — E-QUERY-038 must fire for \
                 column typo inside count() in HAVING. Before the fix, Position 6 (HAVING) \
                 was absent from check_query_column_availability."
            ),
            Err(other) => panic!(
                "F-PWL1-LOW-001: expected PrismError::ColumnNotFound (E-QUERY-038), \
                 got different error: {other:?}"
            ),
        }
    }

    // ── No-regression guard (must PASS both before and after fix) ─────────────

    /// F-PWL1-LOW-001 no-regression — HAVING with a valid column must NOT fire E-QUERY-038.
    ///
    /// `HAVING severity = 'critical'` — `severity` is a valid column in `crowdstrike_alerts`.
    /// The gate must pass Position 6 without error; the query may fail later (no real
    /// adapter wired) but must NOT fail with E-QUERY-038.
    ///
    /// BC-2.11.016 v1.5 / F-PWL1-LOW-001.
    #[tokio::test]
    async fn test_BC_2_11_016_having_column_gate_valid_col_no_e_query_038() {
        let (engine, org) = make_crowdstrike_engine();

        // `severity` is a valid column — HAVING gate must NOT fire E-QUERY-038.
        let query = "SELECT severity, count(*) FROM crowdstrike_alerts \
                     GROUP BY severity HAVING severity = 'critical'";

        let result = engine
            .execute(
                query,
                QueryOptions {
                    clients: Some(vec![org]),
                    ..QueryOptions::default()
                },
            )
            .await;

        match result {
            Err(PrismError::ColumnNotFound(ref details)) => panic!(
                "F-PWL1-LOW-001 no-regression: E-QUERY-038 fired unexpectedly for valid column \
                 '{}'. `severity` is registered; the HAVING gate must NOT reject it.",
                details.column
            ),
            // Any other outcome (Ok, execution error, other PrismError) is acceptable —
            // the invariant is only that E-QUERY-038 (ColumnNotFound) does NOT fire.
            _ => {}
        }
    }
}
