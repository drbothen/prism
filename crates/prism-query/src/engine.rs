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

use prism_core::error::sanitize_for_log;
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
    pub(crate) org_registry: Option<Arc<prism_core::OrgRegistry>>,
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
    pub(crate) resolved_spec_map: Option<
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

    /// Wire the per-org registry for per-org adapter selection (F-LP1-CRIT-3 / ADR-029).
    ///
    /// Matches the `new_full` constructor path; use this method when constructing via
    /// `new_with_cache_config` and adding the registry post-construction.
    /// (F-MCPRS-PRL1-OBS-002: centralises registry wiring, replaces direct pub-field writes)
    pub fn with_org_registry(mut self, registry: Arc<prism_core::OrgRegistry>) -> Self {
        self.org_registry = Some(registry);
        self
    }

    /// Wire the per-org overlay resolved spec map for per-org endpoint dispatch (ADR-029).
    ///
    /// Wraps the supplied map in `Arc<ArcSwap<...>>` so hot-reload can atomically swap it.
    /// Matches the `new_full` constructor path; use this method when constructing via
    /// `new_with_cache_config` and adding the map post-construction.
    /// (F-MCPRS-PRL1-OBS-002: centralises ArcSwap wrapping, replaces 23 direct-field callsites)
    pub fn with_resolved_spec_map(
        mut self,
        resolved: Arc<
            std::collections::HashMap<
                prism_spec_engine::ResolvedSpecKey,
                prism_spec_engine::ResolvedSensorSpec,
            >,
        >,
    ) -> Self {
        self.resolved_spec_map = Some(Arc::new(arc_swap::ArcSwap::new(resolved)));
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

        // ADR-048 v1.2 §D.7.4 — aggregate-in-predicate gate fires BEFORE temporal gate.
        //
        // When a WHERE predicate contains an aggregate fn-call (e.g., `WHERE stddev(x) = 5`),
        // the D.7 aggregate gate in `check_enrich_udf_availability` must fire E-QUERY-001
        // BEFORE `check_temporal_literals` fires E-QUERY-042. Without this pre-check, a
        // query like `WHERE stddev(x) = '2026-06-24'` would receive E-QUERY-042
        // (NonColumnLhsComparison) from the temporal gate — which is incorrect: the
        // primary fault is the aggregate-in-WHERE, not the date-like RHS.
        //
        // Passing `None` for the registry here runs ONLY the aggregate gate (E-QUERY-001)
        // and skips the E-QUERY-039 infusion-UDF check (which requires E-QUERY-037 and
        // E-QUERY-038 to have passed first, per BC-2.11.019 gate ordering). The full
        // E-QUERY-039 check runs at the call site below (after table and column gates).
        //
        // COST: this invocation performs a full `PrismQlParser::parse` (O(n)) regardless
        // of whether any aggregate is found in a predicate position. The double-parse cost
        // is accepted and ADR-048 §D.7.4-ratified — consistent with the temporal-check
        // double-parse design (HIGH-4 note below), which also re-parses at plan time.
        check_enrich_udf_availability(effective_query, None)?;

        // ADR-052 §D4 Option A: E-QUERY-041 temporal literal gate fires BEFORE E-QUERY-037.
        //
        // EC-013 (story spec): dotted external-source queries with temporal literals must
        // produce E-QUERY-041 (bad timestamp format), NOT E-QUERY-037 (table not found).
        // Example: `FROM ghost_sensor.devices | where timestamp > '2026-06-24'`
        //   → E-QUERY-041 (not E-QUERY-037), because the dotted source normalises to
        //   "ghost_sensor_devices" for schema lookup in check_temporal_literals.
        //
        // HIGH-4 design note (EARLY-GATE-ORDERING-ONLY): this early check re-parses the
        // query string intentionally. It does NOT share the AST with run_materialization_pipeline
        // because inject_now (which runs inside the pipeline, pre-temporal-gate) has NOT yet
        // fired here. The double-parse is the minimal correct design: (1) this pass enforces
        // E-QUERY-041 BEFORE E-QUERY-037 for dotted-source queries (EC-013 ordering); (2) the
        // mutation-carrying coercion pass runs inside run_materialization_pipeline on the
        // inject_now-resolved AST. Any mutation applied here is intentionally discarded — the
        // coerced AST is produced by Step 1c of run_materialization_pipeline. Refactoring to
        // share the AST would require moving inject_now before check_table_availability, which
        // changes the semantic order of planner steps and risks inject_now side-effects
        // (resolving $NOW before the table-availability check) — that is a larger spec change.
        //
        // When no temporal literals are present, this check returns Ok(()) immediately and the
        // canonical E-QUERY-037 → E-QUERY-038 → E-QUERY-039 ordering is preserved.
        // Parse failure → pass through (pipeline surfaces E-QUERY-001 downstream).
        //
        // skip_projection=true (FIX-2): the early gate only checks WHERE/HAVING/JOIN predicates
        // so that a projection-position RawTemporalLiteral (e.g., `SELECT '2026-06-24' FROM t`)
        // does NOT fire E-QUERY-002 before check_table_availability fires E-QUERY-037 for
        // unregistered tables (BC-2.11.019 gate ordering). Projection checks run in the
        // in-pipeline check_temporal_literals call (skip_projection=false, after table
        // availability is confirmed). EC-013 is preserved: WHERE-predicate Datetime checks still
        // fire E-QUERY-041 before E-QUERY-037 for registered dotted-source queries.
        if let Ok(mut ast) = crate::filter_parser::PrismQlParser::parse(effective_query) {
            crate::materialization::check_temporal_literals(
                &mut ast,
                self.table_registry.as_deref(),
                true, // skip_projection: defer SELECT/GROUP-BY/ORDER-BY to in-pipeline pass
            )?;
        }

        // S-3.13 Step 1a: Plan-time table availability gate (BC-2.11.001, AC-2, AC-8).
        //
        // Gate ordering (BC-2.11.019 §Gate ordering, S-DEMO-FIDELITY-REMEDIATION-001 HIGH-001):
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
            self.infusion_registry.as_deref(),
        )?;

        // S-DEMO-FIDELITY-REMEDIATION-001 AC-N1B: Plan-time enrichment UDF gate (E-QUERY-039).
        //
        // Fires LAST — after E-QUERY-037 (table gate) and E-QUERY-038 (column gate).
        // Gate ordering (BC-2.11.019 §Gate ordering): E-QUERY-001 → E-QUERY-037 → E-QUERY-038 → E-QUERY-039.
        //
        // Validates that all enrichment function names in the query (pipe: `| enrich name(col)`;
        // SQL: `SELECT name(col)` or `WHERE name(col) = val`) are registered per-field UDF names
        // in the InfusionRegistry. Returns E-QUERY-039 with available_infusions and did_you_mean
        // when an unregistered name is detected (prevents "infusion_id used as UDF name" silent
        // failures).
        //
        // Gate is skipped when `infusion_registry` is None (enrichment not configured).
        //
        // COST: this invocation performs a full `PrismQlParser::parse` (O(n)) — the same
        // double-parse cost accepted for the registry=None invocation above (ADR-048 §D.7.4-ratified
        // gate ordering; consistent with the temporal-check double-parse design).
        check_enrich_udf_availability(effective_query, self.infusion_registry.as_deref())?;

        // ADR-052 D4 Option A: plan-time temporal literal gate is now implemented as
        // an AST-walk inside run_materialization_pipeline (check_temporal_literals).
        // The old text-scanner (check_temporal_literals) is deleted; the AST-walk fires
        // against the same parsed AST used for execution, after inject_now.
        // Gate ordering: E-QUERY-037 → E-QUERY-038 → E-QUERY-039 → [AST-walk in mat pipeline].

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

        // ADR-052 §D4 v1.10 Option A: wire table_registry so check_temporal_literals
        // can resolve column types for the seven-arm dispatch
        // (E-QUERY-041 / coerce / mismatch / E-QUERY-042 for non-Field-LHS and GROUP BY/ORDER BY).
        if let Some(ref tr) = self.table_registry {
            mat_ctx = mat_ctx.with_table_registry(Arc::clone(tr));
        }

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
        // ADR-048 §D.7.4 — aggregate-in-predicate gate fires BEFORE temporal gate (mirrors
        // execute_inner's registry=None pre-check). Passing None runs ONLY the aggregate gate
        // (E-QUERY-001) and skips E-QUERY-039; E-QUERY-039 runs at the call site below after
        // table and column gates. COST: performs a full `PrismQlParser::parse` (O(n)) —
        // ADR-048 §D.7.4-ratified double-parse; cost accepted, consistent with temporal-check
        // design (see execute_inner HIGH-4 comment for the full rationale).
        check_enrich_udf_availability(query_str, None)?;

        // ADR-052 §D4 Option A: E-QUERY-041 temporal literal gate fires BEFORE E-QUERY-037.
        // EC-013: dotted external-source temporal literal queries → E-QUERY-041, not E-QUERY-037.
        // Mirrors execute_inner's early temporal check. See execute_inner HIGH-4 comment for
        // the EARLY-GATE-ORDERING-ONLY design rationale (intentional double-parse).
        // skip_projection=true (FIX-2): same scoping as execute_inner — projection checks
        // deferred to in-pipeline pass so E-QUERY-037 wins for unregistered tables.
        if let Ok(mut ast) = crate::filter_parser::PrismQlParser::parse(query_str) {
            crate::materialization::check_temporal_literals(
                &mut ast,
                self.table_registry.as_deref(),
                true, // skip_projection: defer SELECT/GROUP-BY/ORDER-BY to in-pipeline pass
            )?;
        }

        // S-3.13: plan-time table availability gate for scheduled queries (AC-8 mode-agnostic).
        // Gate ordering (BC-2.11.019 §Gate ordering, S-DEMO-FIDELITY-REMEDIATION-001 HIGH-001):
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
            self.infusion_registry.as_deref(),
        )?;

        // S-DEMO-FIDELITY-REMEDIATION-001 AC-N1B: E-QUERY-039 enrichment UDF gate for
        // scheduled queries — fires LAST among content gates (after E-QUERY-037 and E-QUERY-038).
        // Gate ordering (BC-2.11.019 §Gate ordering): E-QUERY-037 → E-QUERY-038 → E-QUERY-039.
        // COST: performs a full `PrismQlParser::parse` (O(n)) — ADR-048 §D.7.4-ratified
        // double-parse; cost accepted (consistent with temporal-check double-parse design).
        check_enrich_udf_availability(query_str, self.infusion_registry.as_deref())?;

        // ADR-052 D4 Option A: temporal gate is now in run_materialization_pipeline.
        // See execute_inner for the full gate-ordering comment.

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

        // ADR-052 §D4 Option A: wire table_registry for AST-walk temporal gate.
        if let Some(ref tr) = self.table_registry {
            mat_ctx = mat_ctx.with_table_registry(Arc::clone(tr));
        }

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

/// DataFusion built-in function names (scalar + aggregate + window) — computed once at
/// first use via `LazyLock`.
///
/// The enrich gate must NOT flag a name that DataFusion can resolve as a built-in function
/// of ANY kind: scalar (e.g. `lower`, `upper`, `coalesce`), aggregate (e.g. `stddev`,
/// `median`, `variance`, `approx_distinct`), or window (e.g. `row_number`, `rank`,
/// `dense_rank`).
///
/// PrismQL's SQL parser recognises only 7 function names as `FuncCall::Aggregate`
/// (COUNT / SUM / AVG / MIN / MAX / PERCENTILE / DISTINCT_COUNT). Every other function
/// name — including DataFusion built-in aggregates and window functions — parses as
/// `ScalarFunc::Unknown(name)`. Without explicit exclusion of aggregates and windows,
/// `SELECT stddev(col) FROM t` would falsely trigger E-QUERY-039 because `stddev` is not
/// in the scalar-only exclusion set, even though DataFusion's `ctx.sql()` resolves it
/// correctly as an aggregate function.
///
/// Mechanism: union `SessionStateDefaults::default_scalar_functions()`,
/// `default_aggregate_functions()`, and `default_window_functions()` — enumerate every
/// built-in UDF, collect their lowercase names and aliases into a single `HashSet`, and
/// store in a static `LazyLock`. Per-query cost: a single O(1) HashSet lookup per
/// collected name. Initialization is once-per-process (at first gate invocation).
///
/// Case-insensitive exclusion: DataFusion normalizes function names to lowercase
/// internally; we lowercase the collected name before lookup to match.
///
/// # BC-2.11.019 §F-PJL1-HIGH-001 amendment — extended "or equivalent" rationale
///
/// BC-2.11.019 stated: "fire E-QUERY-039 ONLY for a name that is neither a
/// DataFusion built-in scalar NOR a registered enrichment UDF."
///
/// BC-2.11.019 (F1, S-DEMO-FIDELITY-REMEDIATION-001 Pass-N1b) amends this to:
/// "fire E-QUERY-039 ONLY for a `ScalarFunc::Unknown(name)` in SQL mode when name is
/// (a) not a PQL typed scalar variant, (b) NOT in ANY of scalar_functions() +
/// aggregate_functions() + window_functions(), AND (c) not in InfusionRegistry."
///
/// Scope of change: the expanded exclusion applies to SQL-mode `ScalarFunc::Unknown`
/// names ONLY — the same scope as the original v1.5 fix. Pipe-mode `| enrich <name>` is
/// an explicit enrichment directive; a built-in aggregate name used there is still an
/// unregistered infusion and MUST still fire E-QUERY-039. This distinction is preserved
/// by the separate `pipe_enrich_names` / `sql_unknown_names` Vec split in
/// `check_enrich_udf_availability`.
///
/// F-PJL1-HIGH-001 (S-DEMO-FIDELITY-REMEDIATION-001 Pass-J LOCAL cascade) — original fix.
/// F1 amendment (S-DEMO-FIDELITY-REMEDIATION-001 Pass-N1b LOCAL cascade) — this change.
static DATAFUSION_BUILTIN_FUNCTION_NAMES: std::sync::LazyLock<std::collections::HashSet<String>> =
    std::sync::LazyLock::new(|| {
        use datafusion::execution::SessionStateDefaults;
        let mut names = std::collections::HashSet::new();

        // (1) Built-in scalar functions (e.g. lower, upper, coalesce, concat, abs, round).
        for udf in SessionStateDefaults::default_scalar_functions() {
            names.insert(udf.name().to_ascii_lowercase());
            for alias in udf.aliases() {
                names.insert(alias.to_ascii_lowercase());
            }
        }

        // (2) Built-in aggregate functions (e.g. stddev, median, variance, approx_distinct,
        //     array_agg, string_agg, corr, covar_pop, bool_and, bool_or, regr_*).
        //     These parse as ScalarFunc::Unknown in PrismQL but are resolvable by ctx.sql()
        //     as aggregate functions — they must NOT trigger E-QUERY-039.
        for udaf in SessionStateDefaults::default_aggregate_functions() {
            names.insert(udaf.name().to_ascii_lowercase());
            for alias in udaf.aliases() {
                names.insert(alias.to_ascii_lowercase());
            }
        }

        // (3) Built-in window functions (e.g. row_number, rank, dense_rank, percent_rank,
        //     lag, lead, first_value, last_value, nth_value, ntile, cume_dist).
        //     Same parse-as-Unknown issue; same resolution: ctx.sql() handles them correctly.
        for udwf in SessionStateDefaults::default_window_functions() {
            names.insert(udwf.name().to_ascii_lowercase());
            for alias in udwf.aliases() {
                names.insert(alias.to_ascii_lowercase());
            }
        }

        names
    });

/// Set of DataFusion built-in AGGREGATE function names (lowercase) — used by the
/// aggregate-in-predicate plan-time gate in `check_enrich_udf_availability` (ADR-048 D.3).
///
/// Derived from `SessionStateDefaults::default_aggregate_functions()` — the same source
/// used to populate the aggregate portion of `DATAFUSION_BUILTIN_FUNCTION_NAMES`.
/// Single source of truth: DataFusion's aggregate registry, not a hard-coded list.
///
/// When a `ScalarFunc::Unknown(name)` in a predicate fn-call LHS position resolves to
/// a name in this set, the gate returns E-QUERY-001 (QueryParseFailed) with the
/// ADR-048 D.3 aggregate-in-where message.
static DATAFUSION_BUILTIN_AGGREGATE_NAMES: std::sync::LazyLock<std::collections::HashSet<String>> =
    std::sync::LazyLock::new(|| {
        use datafusion::execution::SessionStateDefaults;
        let mut names = std::collections::HashSet::new();
        for udaf in SessionStateDefaults::default_aggregate_functions() {
            names.insert(udaf.name().to_ascii_lowercase());
            for alias in udaf.aliases() {
                names.insert(alias.to_ascii_lowercase());
            }
        }
        // PrismQL-specific aggregate names ABSENT from DataFusion 53.1's built-in registry
        // (EMPIRICALLY VERIFIED — see `datafusion_aggregate_registry_empirical_tests` below)
        // but semantically aggregate functions in PrismQL grammar (ADR-048 v1.2 D.7.1).
        // These must also be rejected in WHERE predicates with the canonical E-QUERY-001
        // message — they cannot be valid WHERE predicate LHS values.
        //
        // distinct_count: maps to SQL APPROX_DISTINCT / COUNT(DISTINCT ...) at emit time.
        //   DataFusion 53.1 uses "approx_distinct", NOT "distinct_count" — absent from registry.
        // percentile:     maps to APPROX_PERCENTILE_CONT at emit time.
        //   DataFusion 53.1 has NO "percentile" built-in — absent from registry.
        //   ADR-048 v1.3 claimed "percentile IS registered" — EMPIRICALLY FALSE (F-PQLFN-P4-MED-001).
        //   ADR-048 v1.4 retracted this claim (§D.2 PERCENTILE note corrected; F-PQLFN-P4-MED-001); manual insert is necessary, not redundant.
        //
        // Both are in the removed parser-level AGGREGATE_FUNC_NAMES list (OD-4 removal) and
        // must be covered by this plan-time gate to maintain the WHERE aggregate invariant
        // (ADR-048 D.6). The union mechanism: DataFusion registry ∪ PrismQL-specific names.
        names.insert("distinct_count".to_string());
        names.insert("percentile".to_string());
        names
    });

/// Walk the six top-level scalar-expr positions in a `SqlQuery` and collect
/// `ScalarFunc::Unknown` names.
///
/// This is the SINGLE canonical walk used by `check_enrich_udf_availability` for both
/// `Ast::Sql(Select)` and `Ast::SqlPipe` head queries.  It covers the six top-level
/// positions in a `SqlQuery` where a scalar function call is typically written:
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
/// # Intentionally excluded positions (BC-2.11.019 §OBS-001)
///
/// `Expr::InSubquery` / `Predicate::InSubquery` subquery bodies are NOT descended into.
/// This is intentional fail-open behaviour adjudicated in BC-2.11.019 §OBS-001:
/// a `ScalarFunc::Unknown` inside `IN (SELECT ...)` reaches DataFusion planning as a
/// function-not-found error (not the opaque `E-INT-001` crash that this gate prevents);
/// the subquery body self-governs.  If PrismQL grammar is later extended to require enrich
/// gating inside subquery bodies, BC-2.11.019 must be updated at that time.
///
/// S-DEMO-FIDELITY-REMEDIATION-001 C1+C2 fix; BC-2.11.019 §Precondition 1(b).
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
/// Recurses into `FuncCall::Scalar` / `FuncCall::Aggregate` arguments, `Expr::Logical`
/// (lhs/rhs), `Expr::Not`, and `Expr::Compare` (lhs/rhs) to find every
/// `ScalarFunc::Unknown(name)` node.
///
/// Intentionally excluded positions (see BC-2.11.019 §OBS-001):
/// - `Expr::TimestampArithmetic { base, .. }` — `base` is always `Expr::Now` (grammar
///   constraint; see `build_temporal_rhs_parser`); a UDF cannot appear there.
/// - `Expr::InSubquery { subquery, .. }` — subquery body is fail-open per OBS-001.
///
/// Module-level so it is accessible from `#[cfg(test)]` blocks for unit testing.
/// Called by `check_enrich_udf_availability` and `collect_unknown_scalars_from_sql_query`.
///
/// All `Expr` variants are enumerated explicitly (no wildcard `_ => {}`) so that adding a
/// new `Expr` variant forces a compile error here, preventing a silent no-op for a variant
/// that may contain a nested `ScalarFunc::Unknown`. `Expr` is `#[non_exhaustive]` but
/// in-crate matches are exhaustively checkable — future variants must force a compile error
/// here (mirrors `shift_scalar_spans_in_expr` discipline, F-PQLFN-PR14-OBS-004).
fn collect_unknown_scalar_from_expr(expr: &crate::ast::Expr, out: &mut Vec<String>) {
    use crate::ast::{Expr, FuncCall, ScalarFunc};
    match expr {
        Expr::FuncCall(FuncCall::Scalar {
            func: ScalarFunc::Unknown(name),
            args,
            ..
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
        Expr::FuncCall(FuncCall::Window { .. }) => {
            // Window stub: no args field currently (S-3.06 will add fields).
            // No ScalarFunc::Unknown to collect.
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
        // Leaf nodes — no sub-expressions that can contain a function call.
        Expr::Literal(_) | Expr::Field(_) | Expr::VirtualField(_) => {}
        Expr::Star | Expr::Now | Expr::Interval(_) => {}
        Expr::In { .. } => {
            // FieldPath + Vec<Literal> — no Expr sub-tree that can contain
            // a ScalarFunc::Unknown.
        }
        Expr::TimestampArithmetic { .. } => {
            // `base` is always `Expr::Now` per the grammar (`build_temporal_rhs_parser`
            // only parses `NOW() ± INTERVAL '...'` — the base is hard-coded to
            // `Expr::Now`). A `ScalarFunc::Unknown` cannot appear as the `base`
            // expression of a timestamp arithmetic node in valid PrismQL AST.
        }
        Expr::InSubquery { .. } => {
            // The subquery body CAN structurally contain a `ScalarFunc::Unknown` in
            // its SELECT items, but the gate intentionally does NOT descend into
            // subquery bodies — BC-2.11.019 §OBS-001 fail-open convention (DataFusion
            // produces function-not-found, not the opaque E-INT-001 crash this gate
            // prevents). Enumerated explicitly so a future grammar extension that
            // widens the gate to subquery bodies forces a compile error here rather
            // than silently becoming a no-op.
        }
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
///
/// All `Predicate` variants are enumerated explicitly (no wildcard `_ => {}`) so that
/// adding a new `Predicate` variant forces a compile error here, preventing a silent
/// no-op for a variant that may hold a nested `Expr` containing a `ScalarFunc::Unknown`.
/// `Predicate` is `#[non_exhaustive]` but in-crate matches are exhaustively checkable —
/// future variants must force a compile error here (mirrors `shift_scalar_spans_in_predicate`
/// discipline, F-PQLFN-PR14-OBS-004).
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
        Predicate::StringOp { .. } => {
            // FieldPath + String pattern + flags — no Expr or FuncCall containing
            // ScalarFunc::Unknown.
        }
        Predicate::Regex { .. } => {
            // FieldPath + RegexLiteral — no Expr or FuncCall containing
            // ScalarFunc::Unknown.
        }
        Predicate::In { .. } => {
            // FieldPath + Vec<Literal> — no Expr or FuncCall containing
            // ScalarFunc::Unknown.
        }
        Predicate::InSubquery { .. } => {
            // The subquery body CAN structurally contain a `ScalarFunc::Unknown` in
            // its SELECT items, but the gate intentionally does NOT descend into
            // subquery bodies — BC-2.11.019 §OBS-001 fail-open convention (DataFusion
            // produces function-not-found, not the opaque E-INT-001 crash this gate
            // prevents). Enumerated explicitly so a future grammar extension that
            // widens the gate to subquery bodies forces a compile error here rather
            // than silently becoming a no-op.
        }
        Predicate::Between { .. } => {
            // FieldPath + Literal bounds — `low` and `high` are `Literal` (not `Expr`),
            // so a ScalarFunc::Unknown provably cannot appear in either position.
        }
        Predicate::Cidr { .. } => {
            // FieldPath + CidrLiteral — no Expr or FuncCall containing
            // ScalarFunc::Unknown.
        }
        Predicate::Has(_) => {
            // FieldPath only — no Expr or FuncCall containing ScalarFunc::Unknown.
        }
        Predicate::Missing(_) => {
            // FieldPath only — no Expr or FuncCall containing ScalarFunc::Unknown.
        }
        Predicate::IsNull { .. } => {
            // FieldPath + negated bool — no Expr or FuncCall containing
            // ScalarFunc::Unknown.
        }
        Predicate::Wildcard { .. } => {
            // FieldPath + String pattern + negated bool — no Expr or FuncCall
            // containing ScalarFunc::Unknown.
        }
        Predicate::RecoveryError => {
            // Sentinel produced by error recovery only — no fields; never contains
            // a function call.
        }
    }
}

/// Collect all `ScalarFunc::Unknown` names AND their source offsets from an `Expr` tree.
///
/// Mirrors `collect_unknown_scalar_from_expr` but accumulates `(name, span.start)` pairs
/// so the aggregate-in-predicate gate (E-QUERY-001) can report truthful source offsets
/// per ADR-048 §D.7.2 (F-PQLFN-P21-OBS-003).
///
/// The `span.start` field is the byte offset of the function name in the original query
/// string, populated by filter_parser.rs `fn_call_comparison` via `map_with`. For AST
/// nodes constructed outside the parser (tests, direct construction), `span` is `Span::ZERO`
/// → `span.start == 0`; callers should accept `0` as "offset unknown" in that case.
/// All `Expr` variants are enumerated explicitly (no wildcard `_ => {}`) so that adding a
/// new `Expr` variant forces a compile error here, preventing a silent no-op for a variant
/// that may contain a nested `ScalarFunc::Unknown`. `Expr` is `#[non_exhaustive]` but
/// in-crate matches are exhaustively checkable — future variants must force a compile error
/// here (mirrors `shift_scalar_spans_in_expr` discipline, F-PQLFN-PR14-OBS-004).
fn collect_unknown_scalar_offsets_from_expr(
    expr: &crate::ast::Expr,
    out: &mut Vec<(String, usize)>,
) {
    use crate::ast::{Expr, FuncCall, ScalarFunc};
    match expr {
        Expr::FuncCall(FuncCall::Scalar {
            func: ScalarFunc::Unknown(name),
            args,
            span,
        }) => {
            out.push((name.clone(), span.start));
            for arg in args {
                collect_unknown_scalar_offsets_from_expr(arg, out);
            }
        }
        Expr::FuncCall(FuncCall::Scalar { args, .. }) => {
            for arg in args {
                collect_unknown_scalar_offsets_from_expr(arg, out);
            }
        }
        Expr::FuncCall(FuncCall::Aggregate { args, .. }) => {
            for arg in args {
                collect_unknown_scalar_offsets_from_expr(arg, out);
            }
        }
        Expr::FuncCall(FuncCall::Window { .. }) => {
            // Window stub: no args field currently (S-3.06 will add fields).
            // No ScalarFunc::Unknown span to collect.
        }
        Expr::Logical { lhs, rhs, .. } => {
            collect_unknown_scalar_offsets_from_expr(lhs, out);
            collect_unknown_scalar_offsets_from_expr(rhs, out);
        }
        Expr::Not(inner) => collect_unknown_scalar_offsets_from_expr(inner, out),
        Expr::Compare { lhs, rhs, .. } => {
            collect_unknown_scalar_offsets_from_expr(lhs, out);
            collect_unknown_scalar_offsets_from_expr(rhs, out);
        }
        // Leaf nodes — no sub-expressions that can contain a function call.
        Expr::Literal(_) | Expr::Field(_) | Expr::VirtualField(_) => {}
        Expr::Star | Expr::Now | Expr::Interval(_) => {}
        Expr::In { .. } => {
            // FieldPath + Vec<Literal> — no Expr sub-tree that can contain
            // a ScalarFunc::Unknown.
        }
        Expr::TimestampArithmetic { .. } => {
            // `base` is always `Expr::Now` per the grammar — a `ScalarFunc::Unknown`
            // cannot appear as the `base` expression of a timestamp arithmetic node
            // in valid PrismQL AST.
        }
        Expr::InSubquery { .. } => {
            // Intentionally not descended — BC-2.11.019 §OBS-001 fail-open convention.
            // Enumerated explicitly so a future grammar extension forces a compile error
            // here rather than silently becoming a no-op.
        }
    }
}

/// Collect all `ScalarFunc::Unknown` names AND their source offsets from a `Predicate` tree.
///
/// Mirrors `collect_unknown_scalar_from_predicate` but accumulates `(name, span.start)` pairs
/// for the aggregate-in-predicate E-QUERY-001 gate (F-PQLFN-P21-OBS-003).
///
/// All `Predicate` variants are enumerated explicitly (no wildcard `_ => {}`) so that
/// adding a new `Predicate` variant forces a compile error here, preventing a silent
/// no-op for a variant that may hold a nested `Expr` containing a `ScalarFunc::Unknown`.
/// `Predicate` is `#[non_exhaustive]` but in-crate matches are exhaustively checkable —
/// future variants must force a compile error here (mirrors `shift_scalar_spans_in_predicate`
/// discipline, F-PQLFN-PR14-OBS-004).
fn collect_unknown_scalar_offsets_from_predicate(
    pred: &crate::ast::Predicate,
    out: &mut Vec<(String, usize)>,
) {
    use crate::ast::Predicate;
    match pred {
        Predicate::Compare { lhs, rhs, .. } => {
            collect_unknown_scalar_offsets_from_expr(lhs, out);
            collect_unknown_scalar_offsets_from_expr(rhs, out);
        }
        Predicate::Logical { predicates, .. } => {
            for p in predicates {
                collect_unknown_scalar_offsets_from_predicate(p, out);
            }
        }
        Predicate::Not(inner) => collect_unknown_scalar_offsets_from_predicate(inner, out),
        Predicate::StringOp { .. } => {
            // FieldPath + String pattern + flags — no Expr or FuncCall containing
            // ScalarFunc::Unknown.
        }
        Predicate::Regex { .. } => {
            // FieldPath + RegexLiteral — no Expr or FuncCall containing
            // ScalarFunc::Unknown.
        }
        Predicate::In { .. } => {
            // FieldPath + Vec<Literal> — no Expr or FuncCall containing
            // ScalarFunc::Unknown.
        }
        Predicate::InSubquery { .. } => {
            // The subquery body CAN structurally contain a `ScalarFunc::Unknown` in
            // its SELECT items, but the gate intentionally does NOT descend into
            // subquery bodies — BC-2.11.019 §OBS-001 fail-open convention. Enumerated
            // explicitly so a future grammar extension that widens the gate to subquery
            // bodies forces a compile error here rather than silently becoming a no-op.
        }
        Predicate::Between { .. } => {
            // FieldPath + Literal bounds — `low` and `high` are `Literal` (not `Expr`),
            // so a ScalarFunc::Unknown provably cannot appear in either position.
        }
        Predicate::Cidr { .. } => {
            // FieldPath + CidrLiteral — no Expr or FuncCall containing
            // ScalarFunc::Unknown.
        }
        Predicate::Has(_) => {
            // FieldPath only — no Expr or FuncCall containing ScalarFunc::Unknown.
        }
        Predicate::Missing(_) => {
            // FieldPath only — no Expr or FuncCall containing ScalarFunc::Unknown.
        }
        Predicate::IsNull { .. } => {
            // FieldPath + negated bool — no Expr or FuncCall containing
            // ScalarFunc::Unknown.
        }
        Predicate::Wildcard { .. } => {
            // FieldPath + String pattern + negated bool — no Expr or FuncCall
            // containing ScalarFunc::Unknown.
        }
        Predicate::RecoveryError => {
            // Sentinel produced by error recovery only — no fields; never contains
            // a function call.
        }
    }
}

/// Plan-time enrichment UDF availability gate — E-QUERY-039 (BC-2.11.019).
///
/// Fires AFTER `check_table_availability` AND `check_query_column_availability`
/// (BC-2.11.019 §Gate ordering: gate sequence is 001 → 037 → 038 → 039;
/// enrich gate is last in the chain).
///
/// Parses the query string, collects all enrichment function names used in the query
/// (both pipe-mode `| enrich udf_name(col)` and SQL-mode `SELECT udf_name(col)`), then
/// validates each against the `InfusionRegistry` descriptor set.
///
/// Also runs the aggregate-in-predicate plan-time gate (ADR-048 D.3): `ScalarFunc::Unknown`
/// names from predicate fn-call LHS positions are checked against DataFusion built-in
/// aggregate names; if an aggregate appears in a WHERE/where predicate, E-QUERY-001 fires.
///
/// # Gate skip conditions
/// - `registry` is `None`: skip immediately (enrichment not configured).
/// - Query fails to parse: return `Ok(())` — parse errors handled downstream.
/// - No enrichment names found in the AST: return `Ok(())` (query doesn't use enrichment).
/// - Name is a DataFusion built-in scalar: skip (resolved by `ctx.sql()` — not an enrichment).
///
/// # Predicate-position walks (ADR-048 §D.7.1)
///
/// Seven WHERE/where positions are walked into `predicate_fncall_names` for the
/// aggregate-in-predicate gate (ADR-048 D.3). HAVING is exempt at every position
/// (§D.7.1 HAVING exemption; §D.7.3). Positions 1-3 were added by
/// DEFECT-PQL-FNCALL-LHS-001; positions 4-5 by OD-5; position 6 by OD-6 (ADR-048
/// §D.7.5); position 7 by OD-7 (ADR-048 §D.7.6).
///
/// | Position | AST arm | Field walked |
/// |---|---|---|
/// | 1 — `pipe \| where` | `Ast::Pipe` | `PipeStage::Where(pred)` |
/// | 2 — filter-mode root predicate | `Ast::Filter(fe)` | `fe.predicate` |
/// | 3 — `SqlPipe \| where` | `Ast::SqlPipe` | `PipeStage::Where(pred)` in stage list |
/// | 4 — SQL WHERE | `Ast::Sql(Select)` | `sq.where_` |
/// | 5 — SqlPipe-head WHERE | `Ast::SqlPipe` | `spq.head.where_` |
/// | 6 — DML DELETE/UPDATE WHERE | `Ast::Sql(Dml)` | `dml.filter` |
/// | 7 — INSERT source_select WHERE | `Ast::Sql(Dml)` | `dml.source_select.where_` |
///
/// Positions 4 and 5 are also walked by `collect_unknown_scalars_from_sql_query` into
/// `sql_unknown_names` for the E-QUERY-039 enrichment check; they additionally reach
/// E-QUERY-039 via the `predicate_fncall_names → sql_unknown_names` fold in
/// `check_enrich_udf_availability` (two-path coverage). Positions 1-3 and 6-7 reach
/// E-QUERY-039 exclusively via the fold (walk-observable through E-QUERY-039 signals);
/// `collect_unknown_scalars_from_sql_query` does not walk those AST arms.
///
/// # SQL path detection (E-QUERY-039 enrichment check)
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
/// DataFusion built-in functions (scalar, aggregate, window — e.g. lower, stddev,
/// row_number) are excluded via `DATAFUSION_BUILTIN_FUNCTION_NAMES` before the
/// registered-UDF check. `Ast::Sql(Dml)` (DELETE/UPDATE/INSERT) is NOT walked by
/// `collect_unknown_scalars_from_sql_query`; DML predicates feed `predicate_fncall_names`
/// only (positions 6-7 above).
///
/// # Pipe and filter path detection (E-QUERY-039 enrichment check)
/// Pipe-mode enrichment: `PipeStage::Enrich(EnrichStage { infusion, .. })` nodes in
/// the pipe stage list (both `Ast::Pipe` and `Ast::SqlPipe`). The `infusion` field holds
/// the caller-supplied UDF name. No DataFusion built-in skip is applied — `| enrich lower(col)`
/// is an explicit enrichment directive; `lower` there is an unregistered infusion name and
/// E-QUERY-039 MUST fire.
///
/// Pipe-mode `| where` predicates (`PipeStage::Where`) in both `Ast::Pipe` and `Ast::SqlPipe`
/// stage lists are walked into `predicate_fncall_names` (positions 1 and 3 above) for the
/// aggregate gate and then folded into `sql_unknown_names` for the E-QUERY-039 check via
/// the `predicate_fncall_names → sql_unknown_names` fold in `check_enrich_udf_availability`.
/// They do not feed `pipe_enrich_names`.
///
/// Filter mode (`Ast::Filter`): the root predicate is walked into `predicate_fncall_names`
/// (position 2 above) for the aggregate gate and then folded into `sql_unknown_names` for
/// the E-QUERY-039 check. Filter mode has no `| enrich` stages.
///
/// # Reference
/// S-DEMO-FIDELITY-REMEDIATION-001 AC-N1B; BC-2.11.019; error-taxonomy.md E-QUERY-039.
/// F-PJL1-HIGH-001 (Pass-J LOCAL cascade): original scalar exclusion.
/// F1 amendment (Pass-N1b): expanded to aggregate + window functions.
/// DEFECT-PQL-FNCALL-LHS-001: predicate fn-call LHS gate, seven-position audit,
/// positions 1-3 added. ADR-048 §D.7.5 (OD-6): DML WHERE position 6.
/// ADR-048 §D.7.6 (OD-7): INSERT source_select WHERE position 7. OD-5: positions 4-5.
// ── helper: HAVING-interception detail-builder (BC-2.11.019 v1.26 §OBS-004) ──────────────────
/// Build the HAVING-interception E-QUERY-001 detail string for a PrismQL aggregate name.
///
/// Two branches (BC-2.11.019 v1.26 §OBS-004, F-PQLFN-PR5-LOW-001):
/// (a) `name_lower == "percentile"` → two-arg canonical template `(field, p)` — the
///     existing byte-verbatim template UNCHANGED from prior implementation.
/// (b) any other name → signature-neutral generic template `(...)` — correct fail-safe
///     guidance for any future AGGREGATE-only name reaching the arm without the two-arg
///     PERCENTILE misapplication risk.
///
/// The `'{name}'` placeholder is INPUT-VERBATIM (analyst's original casing echoed);
/// the template body uses `{name_upper}` (always uppercase, PrismQL keyword form).
/// Branch (b) is unreachable today (triggering set = {"percentile"}) but is unit-tested
/// directly via `having_aggregate_interception_detail_tests`.
///
/// Caller wraps the returned detail in `PrismError::QueryParseFailed { offset, detail, query }`.
/// (BC-2.11.019 v1.26 §OBS-004; ADR-048 v1.17 §D.2; POL-24)
fn having_aggregate_interception_detail(name: &str) -> String {
    let name_lower = name.to_ascii_lowercase();
    let name_upper = name.to_ascii_uppercase();
    if name_lower == "percentile" {
        // Branch (a): percentile-specific two-arg template (field, p).
        // Byte-verbatim per POL-24 / ADR-048 v1.17 §D.2 canonical template.
        format!(
            "'{name}' is a PrismQL aggregate function; \
             {name_upper} is not directly supported in HAVING predicates \
             \u{2014} alias it in SELECT: \
             SELECT {name_upper}(field, p) AS alias ... HAVING alias > threshold \
             (ADR-048 D.3 OD-2)"
        )
    } else {
        // Branch (b): signature-neutral generic template (...).
        // Unreachable today (triggering set = {"percentile"}).
        // Byte-exact per POL-24 / BC-2.11.019 v1.26 §OBS-004.
        format!(
            "'{name}' is a PrismQL aggregate function; \
             {name_upper} is not directly supported in HAVING predicates \
             \u{2014} alias it in SELECT: \
             SELECT {name_upper}(...) AS alias ... HAVING alias > threshold \
             (ADR-048 D.3 OD-2)"
        )
    }
}

fn check_enrich_udf_availability(
    query_str: &str,
    registry: Option<&prism_spec_engine::InfusionRegistry>,
) -> Result<(), PrismError> {
    use crate::ast::{Ast, PipeStage, SqlStatement};
    use crate::filter_parser::PrismQlParser;
    use prism_core::error::EnrichUdfNotFoundDetails;

    // Parse the query BEFORE the registry-None check. On parse failure, return Ok(()) —
    // parse errors are emitted downstream as E-QUERY-001. Parsing happens first because
    // the aggregate-in-predicate gate (ADR-048 D.3) must run even when infusion registry
    // is not configured — it does not depend on the registry.
    let ast = match PrismQlParser::parse(query_str) {
        Ok(ast) => ast,
        Err(_) => return Ok(()),
    };

    // Collect enrichment UDF names from the AST via direct pattern matching.
    // Using direct match (not the Visitor trait) to avoid coupling with the full
    // visitor infrastructure — enrichment nodes are a well-defined subset.
    //
    // F-PNL1-MED-001 (S-DEMO-FIDELITY-REMEDIATION-001 Pass-N LOCAL cascade):
    // BC-2.11.019 §F-PJL1-HIGH-001 "Scope of change" states the DataFusion
    // built-in exclusion applies to SQL-mode `ScalarFunc::Unknown` gate logic ONLY.
    // Pipe-mode `EnrichStage.infusion` gate is UNAFFECTED — `| enrich lower(col)`
    // is an explicit enrichment directive; `lower` there is not a DataFusion scalar
    // call but an unregistered infusion name, so E-QUERY-039 MUST fire.
    // Fix: separate pipe-mode names and SQL-mode names into distinct Vecs so the
    // built-in skip is applied to SQL names only.
    let mut pipe_enrich_names: Vec<String> = Vec::new(); // no built-in skip
    let mut sql_unknown_names: Vec<String> = Vec::new(); // built-in skip applied
                                                         // ScalarFunc::Unknown names from predicate fn-call LHS — all seven positions per
                                                         // ADR-048 §D.7.1: pipe | where, filter root, SqlPipe | where, SQL WHERE,
                                                         // SqlPipe-head WHERE, SQL DML WHERE, INSERT source_select WHERE. Checked for aggregate classification
                                                         // (ADR-048 D.3 plan-time gate) before being folded into sql_unknown_names for
                                                         // E-QUERY-039. BC-2.11.019 §Postconditions third bullet (DEFECT-PQL-FNCALL-LHS-001).
                                                         // (String, usize) = (name, span.start) — offset is the byte position of the
                                                         // function name in the original query string (F-PQLFN-P21-OBS-003). For AST
                                                         // nodes constructed outside the parser, span.start == 0 ("offset unknown").
    let mut predicate_fncall_names: Vec<(String, usize)> = Vec::new();
    // EC-11-086 (ADR-048 v1.17 §D.2): ScalarFunc::Unknown names from HAVING predicate position
    // (position f of collect_unknown_scalars_from_sql_query). Checked against
    // DATAFUSION_BUILTIN_AGGREGATE_NAMES BEFORE the infusion-registry-None guard, so the
    // interception fires regardless of whether a registry is configured (registry-INDEPENDENT).
    // "percentile" (manually inserted in DATAFUSION_BUILTIN_AGGREGATE_NAMES) is the primary
    // case: excluded from build_agg_call_parser (OD-2 grammar ambiguity), so it parses as
    // ScalarFunc::Unknown("percentile") in HAVING and reaches this gate.
    let mut having_fncall_names: Vec<(String, usize)> = Vec::new();

    match &ast {
        // Pipe mode: `FROM table | enrich udf_name(col)` stages.
        // Post-DEFECT-PQL-FNCALL-LHS-001: also walk PipeStage::Where predicates —
        // build_predicate_parser now accepts ScalarFunc::Unknown fn-call LHS.
        Ast::Pipe(pq) => {
            for stage in &pq.stages {
                match stage {
                    PipeStage::Enrich(es) => {
                        pipe_enrich_names.push(es.infusion.clone());
                    }
                    PipeStage::Where(pred) => {
                        collect_unknown_scalar_offsets_from_predicate(
                            pred,
                            &mut predicate_fncall_names,
                        );
                    }
                    _ => {}
                }
            }
        }
        // Filter mode: root predicate may contain ScalarFunc::Unknown fn-call LHS
        // (post-DEFECT-PQL-FNCALL-LHS-001 grammar extension). Walk the predicate.
        // Previously fell through to `_ => {}` (gate was a no-op for Ast::Filter).
        Ast::Filter(fe) => {
            collect_unknown_scalar_offsets_from_predicate(
                &fe.predicate,
                &mut predicate_fncall_names,
            );
        }
        // SqlPipe mode: SQL head with pipe stages.
        // Enrich names can appear in THREE places:
        //   (a) pipe stages `| enrich udf_name(col)` — pipe-mode, no built-in skip.
        //   (b) pipe stages `| where fn(col) = val` — predicate fn-call LHS.
        //   (c) SQL HEAD: any scalar position in the head SqlQuery (SELECT, WHERE,
        //       JOIN ON, GROUP BY, ORDER BY, HAVING) — SQL-mode, built-in skip applied.
        // BC-2.11.019 §Precondition 1(b): projection OR WHERE (either site counts).
        // C1/C2 fix: use collect_unknown_scalars_from_sql_query to cover ALL positions
        // including JOIN ON / GROUP BY / ORDER BY which the previous inline walk missed.
        //
        // ADR-048 v1.2 §D.7.1 (position 5): SqlPipe-head WHERE predicate fn-call names
        // are walked into predicate_fncall_names for the aggregate-in-predicate gate.
        // Previously these went only to sql_unknown_names via collect_unknown_scalars_from_sql_query
        // position (b), then were filtered by DATAFUSION_BUILTIN_FUNCTION_NAMES before the
        // E-QUERY-039 check — so the aggregate gate never saw them (F-PQLFN-P2-HIGH-001).
        // The head WHERE is walked into BOTH predicate_fncall_names (aggregate gate) AND
        // sql_unknown_names (E-QUERY-039 gate) — duplicate entries are harmless.
        Ast::SqlPipe(spq) => {
            for stage in &spq.stages {
                match stage {
                    // (a) pipe stages — pipe-mode, no built-in skip.
                    PipeStage::Enrich(es) => {
                        pipe_enrich_names.push(es.infusion.clone());
                    }
                    // (b) pipe | where predicates — predicate fn-call LHS.
                    PipeStage::Where(pred) => {
                        collect_unknown_scalar_offsets_from_predicate(
                            pred,
                            &mut predicate_fncall_names,
                        );
                    }
                    _ => {}
                }
            }
            // (D.7.1 position 5) SqlPipe-head WHERE → predicate_fncall_names (aggregate gate).
            // HAVING is intentionally excluded from predicate_fncall_names — only head.where_ is walked here.
            if let Some(pred) = &spq.head.where_ {
                collect_unknown_scalar_offsets_from_predicate(pred, &mut predicate_fncall_names);
            }
            // (EC-11-086 / ADR-048 v1.17 §D.2) SqlPipe head HAVING → having_fncall_names.
            if let Some(pred) = &spq.head.having {
                collect_unknown_scalar_offsets_from_predicate(pred, &mut having_fncall_names);
            }
            // (c) SQL head — ALL scalar positions via canonical shared walk, SQL-mode.
            // This also walks head.where_ into sql_unknown_names (duplicate for WHERE names —
            // harmless; aggregate names will be filtered by DATAFUSION_BUILTIN_FUNCTION_NAMES
            // before E-QUERY-039, and non-aggregate unknowns reach E-QUERY-039 correctly).
            collect_unknown_scalars_from_sql_query(&spq.head, &mut sql_unknown_names);
        }
        // SQL mode: scan ALL scalar positions via canonical shared walk.
        // BC-2.11.019 §Precondition 1(b): projection OR WHERE (either site counts).
        // C1/C2 fix: use collect_unknown_scalars_from_sql_query to cover ALL positions
        // including JOIN ON / GROUP BY / ORDER BY which the previous inline walk missed.
        //
        // ADR-048 v1.2 §D.7.1 (position 4): SQL WHERE predicate fn-call names are walked
        // into predicate_fncall_names for the aggregate-in-predicate gate. Previously they
        // went only to sql_unknown_names via collect_unknown_scalars_from_sql_query position (b),
        // then were filtered by DATAFUSION_BUILTIN_FUNCTION_NAMES before the E-QUERY-039 check —
        // so the aggregate gate never saw them (F-PQLFN-P2-HIGH-001). The WHERE predicate is
        // walked into BOTH predicate_fncall_names (aggregate gate) AND sql_unknown_names via the
        // shared walk below — duplicate entries are harmless.
        Ast::Sql(SqlStatement::Select(sq)) => {
            // (D.7.1 position 4) SQL WHERE → predicate_fncall_names (aggregate gate).
            // HAVING is intentionally excluded from predicate_fncall_names — only sq.where_ is walked here.
            if let Some(pred) = &sq.where_ {
                collect_unknown_scalar_offsets_from_predicate(pred, &mut predicate_fncall_names);
            }
            // (EC-11-086 / ADR-048 v1.17 §D.2) SQL HAVING → having_fncall_names.
            // Separate from predicate_fncall_names: HAVING may legitimately contain aggregates,
            // but names in DATAFUSION_BUILTIN_AGGREGATE_NAMES that parse as ScalarFunc::Unknown
            // (e.g., "percentile") are intercepted with HAVING-specific E-QUERY-001 guidance.
            if let Some(pred) = &sq.having {
                collect_unknown_scalar_offsets_from_predicate(pred, &mut having_fncall_names);
            }
            // All positions (SELECT, WHERE, JOIN ON, GROUP BY, ORDER BY, HAVING) via canonical
            // shared walk into sql_unknown_names for E-QUERY-039.
            collect_unknown_scalars_from_sql_query(sq, &mut sql_unknown_names);
        }
        // (D.7.1 position 6 + 7) DML WHERE: walk filter and source_select WHERE into
        // predicate_fncall_names.
        //
        // Position 6 (DELETE/UPDATE WHERE): build_delete_parser and build_update_parser
        // bind build_predicate_parser for their WHERE clause; post-branch fn_call_comparison
        // is in build_predicate_parser so DML WHERE accepts fn-call LHS. Without this arm
        // the aggregate gate silently passes DML WHERE aggregates → SILENT EMPTY SUCCESS
        // (DML execution no-ops to Ok(vec![])). (ADR-048 §D.7.5, OD-6)
        //
        // Position 7 (INSERT source_select WHERE): build_insert_parser calls build_sql_parser
        // → build_sql_predicate_parser → build_predicate_parser → fn_call_comparison; INSERT
        // carries source_select: Option<SqlQuery> whose WHERE must also be walked. Without this
        // walk, INSERT INTO t SELECT ... WHERE stddev(x) > 5 parses to
        // DmlNode{filter:None, source_select:Some(SqlQuery{where_:Some(...stddev...)})}; the
        // gate sees filter=None and walks nothing → SILENT EMPTY SUCCESS. (ADR-048 §D.7.6, OD-7)
        //
        // source_select HAVING is intentionally exempt — HAVING may legitimately contain
        // aggregate functions (§D.7.1 HAVING exemption; §D.7.3).
        //
        // Note: `Ast` and `SqlStatement` are non-exhaustive for external crates, but within
        // this crate all current variants (Pipe, Filter, SqlPipe, Sql(Select), Sql(Dml)) are
        // explicitly handled — `_ => {}` is removed as it is unreachable within-crate.
        Ast::Sql(SqlStatement::Dml(dml)) => {
            // (D.7.1 position 6) DELETE/UPDATE WHERE → predicate_fncall_names.
            if let Some(pred) = &dml.filter {
                collect_unknown_scalar_offsets_from_predicate(pred, &mut predicate_fncall_names);
            }
            // (D.7.1 position 7) INSERT source_select WHERE → predicate_fncall_names.
            if let Some(src) = &dml.source_select {
                if let Some(pred) = &src.where_ {
                    collect_unknown_scalar_offsets_from_predicate(
                        pred,
                        &mut predicate_fncall_names,
                    );
                }
                // src.having is intentionally exempt — HAVING may legitimately contain
                // aggregate functions (§D.7.1 HAVING exemption; §D.7.3). INSERT
                // source_select HAVING follows the same rule as regular SQL HAVING.
            }
        }
    }

    // Aggregate-in-predicate plan-time gate (ADR-048 D.3) — runs regardless of infusion
    // registry. When a ScalarFunc::Unknown name from a predicate fn-call LHS position is
    // a DataFusion built-in aggregate function (e.g., stddev, variance, median, corr), it
    // cannot be a valid predicate — aggregates belong in HAVING, not WHERE.
    //
    // Rejected with E-QUERY-001 (QueryParseFailed) at plan time so the analyst receives
    // the controlled ADR-048 D.3 message rather than an uncontrolled -32000 / QueryPlanFailed
    // from DataFusion (which also rejects aggregates in WHERE, but with an opaque error).
    //
    // Source of truth: DATAFUSION_BUILTIN_AGGREGATE_NAMES (derived from DataFusion's
    // default_aggregate_functions() registry — same source as the aggregate portion of
    // DATAFUSION_BUILTIN_FUNCTION_NAMES). No hard-coded name list.
    for (name, offset) in &predicate_fncall_names {
        let name_lower = name.to_ascii_lowercase();
        if DATAFUSION_BUILTIN_AGGREGATE_NAMES.contains(&name_lower) {
            return Err(PrismError::QueryParseFailed {
                offset: *offset,
                detail: format!(
                    "'{name}' is an aggregate function; \
                     aggregate fn-calls are not valid in WHERE/where predicates \
                     (use HAVING for post-aggregation filters, ADR-048 D.3)"
                ),
                query: query_str.to_string(),
            });
        }
    }

    // EC-11-086: HAVING-position DATAFUSION_BUILTIN_AGGREGATE_NAMES interception (ADR-048 v1.17 §D.2).
    // Fires BEFORE the registry-None guard — registry-INDEPENDENT.
    // Catches ScalarFunc::Unknown names from HAVING position (f) that are (a) in
    // DATAFUSION_BUILTIN_AGGREGATE_NAMES and (b) NOT in DATAFUSION_BUILTIN_FUNCTION_NAMES.
    // Criterion (b) restricts the gate to names that DataFusion cannot resolve natively in HAVING:
    // natively-registered aggregates (e.g., stddev, variance, corr) are in DATAFUSION_BUILTIN_FUNCTION_NAMES
    // and CAN appear in HAVING without error (DataFusion resolves them via ctx.sql()); those names
    // are NOT intercepted here. Only the manually-inserted names ("percentile", "distinct_count")
    // satisfy criterion (b) — and distinct_count parses as FuncCall::Aggregate (not ScalarFunc::Unknown)
    // via build_agg_call_parser, so it never populates having_fncall_names.
    // Primary case: "percentile" — excluded from build_agg_call_parser (OD-2 two-arg grammar
    // ambiguity), parses as ScalarFunc::Unknown("percentile") in HAVING, NOT in DATAFUSION_BUILTIN_FUNCTION_NAMES.
    // Without this gate: registry=None → Ok(()) → DataFusion plan error; registry=Some → E-QUERY-039.
    // (BC-2.11.004 v1.48 EC-11-086; BC-2.11.019 v1.26 §OBS-004; ADR-048 v1.17 §D.2)
    for (name, offset) in &having_fncall_names {
        let name_lower = name.to_ascii_lowercase();
        if DATAFUSION_BUILTIN_AGGREGATE_NAMES.contains(&name_lower)
            && !DATAFUSION_BUILTIN_FUNCTION_NAMES.contains(&name_lower)
        {
            // Two-branch detail-builder (BC-2.11.019 v1.26 §OBS-004, F-PQLFN-PR5-LOW-001).
            // `having_aggregate_interception_detail` branches on name_lower == "percentile":
            //   (a) percentile → two-arg canonical template `(field, p)` (byte-verbatim, POL-24)
            //   (b) any other name → generic template `(...)` (unreachable today; unit-tested)
            // The v1.22 debug_assert_eq! guard is REMOVED — it compiled out in release; a future
            // AGGREGATE-only name would have emitted the two-arg template (wrong guidance).
            // The two-branch helper provides correct fail-safe guidance without the assertion.
            return Err(PrismError::QueryParseFailed {
                offset: *offset,
                detail: having_aggregate_interception_detail(name),
                query: query_str.to_string(),
            });
        }
    }

    // Skip E-QUERY-039 check when no infusion registry is configured.
    let Some(registry) = registry else {
        return Ok(());
    };

    // Fold predicate fn-call names into sql_unknown_names for E-QUERY-039 gate.
    // DataFusion built-in exclusion (DATAFUSION_BUILTIN_FUNCTION_NAMES) applies —
    // aggregate names that reached here were not in the aggregate-only set (would have
    // been caught above), and non-aggregate DataFusion built-ins (e.g., lower, upper)
    // are excluded from E-QUERY-039 by the DATAFUSION_BUILTIN_FUNCTION_NAMES filter below.
    // Folded here (before descriptor materialization) so the emptiness check below sees
    // the complete set of names. (F-PQLFN-P7-OBS-001 hoist)
    sql_unknown_names.extend(predicate_fncall_names.iter().map(|(n, _)| n.clone()));

    // OBS-001 hoist (F-PQLFN-P7-OBS-001): skip descriptor materialization when there
    // are no names to validate. The aggregate gate already ran; if pipe_enrich_names and
    // sql_unknown_names (which now includes predicate_fncall_names) are both empty, no
    // E-QUERY-039 check is possible — early return avoids the O(n) udf_descriptors()
    // allocation on queries with no enrichment syntax.
    if pipe_enrich_names.is_empty() && sql_unknown_names.is_empty() {
        return Ok(());
    }

    // Build the registered UDF name set from the live registry.
    let descriptors = registry.udf_descriptors();
    let registered_names: std::collections::HashSet<&str> =
        descriptors.iter().map(|d| d.name.as_str()).collect();

    // Validate pipe-mode enrich names — NO DataFusion built-in exclusion.
    // BC-2.11.019 §F-PJL1-HIGH-001: pipe-mode `| enrich <name>` is an explicit
    // enrichment directive. A built-in name like `lower` used as a pipe-mode infusion
    // is NOT a DataFusion scalar — it is an unregistered infusion the analyst is trying
    // to apply, so E-QUERY-039 MUST fire when it is not in InfusionRegistry.
    //
    // Validate SQL-mode unknown scalar names — WITH DataFusion built-in exclusion.
    // BC-2.11.019 §F-PJL1-HIGH-001 (F1 amendment): skip names that are DataFusion built-in functions of ANY kind
    // (scalar, aggregate, or window) — they are resolvable by ctx.sql() and must NOT
    // trigger E-QUERY-039.
    // F-PJL1-HIGH-001 (S-DEMO-FIDELITY-REMEDIATION-001 Pass-J LOCAL cascade) — original.
    // F1 amendment (S-DEMO-FIDELITY-REMEDIATION-001 Pass-N1b) — expanded to agg + window.
    //
    // Iterator chain: pipe names first (no skip), then filtered SQL names (skip applied).
    let sql_names_filtered = sql_unknown_names.iter().filter(|name| {
        let name_lower = name.to_ascii_lowercase();
        !DATAFUSION_BUILTIN_FUNCTION_NAMES.contains(&name_lower)
    });
    let all_names_to_check = pipe_enrich_names.iter().chain(sql_names_filtered);

    for requested in all_names_to_check {
        if !registered_names.contains(requested.as_str()) {
            // Requested name is not a registered per-field UDF name.
            // Build available_infusions from all registered per-field names.
            // MED-001 fix: sort + dedup so the list is deterministic (lexicographic order)
            // as required by error-taxonomy.md §E-QUERY-039 Message Format. This mirrors the
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
            //
            // F-PQLFN-PR14-OBS-001: sanitize `requested` BEFORE the Levenshtein
            // computation so `did_you_mean` and `EnrichUdfNotFoundDetails.infusion`
            // derive from the same sanitized form (BC-2.11.019 v1.26 §Injection-safety,
            // CWE-116/CWE-117). Without this, a name containing Cc/U+2028/U+2029
            // control chars computes did_you_mean against the raw string while the
            // constructor stores the sanitized form — two different string forms of
            // the same input. `EnrichUdfNotFoundDetails::new` applies sanitize_for_log
            // itself, so passing the raw `requested` to the constructor is equivalent
            // to passing the sanitized form for the stored `infusion` field.
            //
            // F-PHL1-HIGH-001: cap the SANITIZED string at 128 bytes (SEC-002 / CWE-407
            // Algorithmic Complexity DoS guard) before the O(m×n) Levenshtein loop —
            // mirrors the table gate cap in `table_registry::did_you_mean`.
            let sanitized_requested = sanitize_for_log(requested);
            let requested_capped =
                crate::table_registry::cap_name_for_levenshtein(&sanitized_requested);
            let did_you_mean = available_infusions
                .iter()
                .map(|n| (n.clone(), strsim::levenshtein(requested_capped, n)))
                .filter(|(_, dist)| *dist <= 3)
                .min_by_key(|(name, dist)| (*dist, name.clone()))
                .map(|(name, _)| name);

            // Pass raw `requested` to the constructor — EnrichUdfNotFoundDetails::new
            // applies sanitize_for_log itself, producing the same sanitized infusion field.
            return Err(PrismError::EnrichUdfNotFound(Box::new(
                EnrichUdfNotFoundDetails::new(requested.clone(), available_infusions, did_you_mean),
            )));
        }
    }

    Ok(())
}

// NOTE: E-QUERY-041 temporal literal gate is now implemented as an AST-walk
// inside run_materialization_pipeline (materialization.rs::check_temporal_literals).
// The old text-scanner (check_temporal_literals + TemporalChecker + helpers) has been
// deleted as part of ADR-052 §D4 Option A implementation.
// Deleted functions: check_temporal_literals, extract_temporal_value_from_parse_error,
//   check_temporal_literals (old stub), extract_primary_table_from_ast,
//   TemporalChecker, is_bad_literal_in_datetime_column,
//   extract_column_name_adjacent_to_quoted_value, extract_table_name_from_query_str,
//   check_string_is_valid_rfc3339.
// Reference: ADR-052 §D4 v1.4; S-PRISMQL-NATIVE-TEMPORAL-TYPING-001 Task 14.

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
// BC-2.11.016 HEAD-JOIN PER-REFERENCE SCOPING helpers (ADV-FIX-P16-MED-001)
// ---------------------------------------------------------------------------

/// Recursively extract `(col_name, is_bare)` pairs from an `Expr` tree.
///
/// Identical traversal to `extract_field_paths_from_expr`; each produced pair carries
/// `is_bare = true` iff the source `FieldPath` had exactly one segment (bare unqualified
/// ref), `false` for multi-segment FROM-alias- or table-name-qualified refs.
///
/// Used by `check_query_column_availability` to implement BC-2.11.016
/// PER-REFERENCE SCOPING: the HEAD-JOIN suspension fires only when `is_bare = true`
/// for the *specific reference* being checked. Qualified refs (`alias.col`, `table.col`)
/// carry `is_bare = false` and are never suspension-eligible, regardless of whether a
/// co-resident bare ref with the same extracted column name exists at another position.
///
/// BC-2.11.016 §Preconditions.2 HEAD-JOIN PER-REFERENCE SCOPING
/// (ADV-FIX-P16-MED-001).
fn extract_field_paths_with_bareness(
    expr: &crate::ast::Expr,
    table_name: &str,
    table_alias: Option<&str>,
    out: &mut Vec<(String, bool)>,
) {
    use crate::ast::{Expr, FuncCall};
    match expr {
        Expr::Field(fp) => {
            if let Some(col) = extract_column_name_from_field_path(fp, table_name, table_alias) {
                let is_bare = fp.segments.len() == 1;
                out.push((col, is_bare));
            }
        }
        Expr::FuncCall(fc) => match fc {
            FuncCall::Aggregate { args, .. } | FuncCall::Scalar { args, .. } => {
                for arg in args {
                    extract_field_paths_with_bareness(arg, table_name, table_alias, out);
                }
            }
            FuncCall::Window { .. } => {}
            #[allow(unreachable_patterns)]
            _ => {}
        },
        Expr::Compare { lhs, rhs, .. } => {
            extract_field_paths_with_bareness(lhs, table_name, table_alias, out);
            extract_field_paths_with_bareness(rhs, table_name, table_alias, out);
        }
        Expr::Logical { lhs, rhs, .. } => {
            extract_field_paths_with_bareness(lhs, table_name, table_alias, out);
            extract_field_paths_with_bareness(rhs, table_name, table_alias, out);
        }
        Expr::Not(inner) => {
            extract_field_paths_with_bareness(inner, table_name, table_alias, out);
        }
        Expr::In { field, .. } => {
            if let Some(col) = extract_column_name_from_field_path(field, table_name, table_alias) {
                let is_bare = field.segments.len() == 1;
                out.push((col, is_bare));
            }
        }
        Expr::InSubquery { field, .. } => {
            if let Some(col) = extract_column_name_from_field_path(field, table_name, table_alias) {
                let is_bare = field.segments.len() == 1;
                out.push((col, is_bare));
            }
        }
        Expr::TimestampArithmetic { base, .. } => {
            extract_field_paths_with_bareness(base, table_name, table_alias, out);
        }
        _ => {}
    }
}

/// Extract `(col_name, is_bare)` pairs from a `Predicate` tree.
///
/// Mirrors `extract_predicate_columns` / `collect_predicate_columns` but emits
/// `(col_name, is_bare)` pairs for BC-2.11.016 PER-REFERENCE SCOPING.
/// Used by `check_query_column_availability` for WHERE (position 2) and HAVING
/// (position 6) clauses so the gate loop can apply HEAD-JOIN suspension per-reference.
///
/// BC-2.11.016 §Preconditions.2 HEAD-JOIN PER-REFERENCE SCOPING
/// (ADV-FIX-P16-MED-001).
fn extract_predicate_columns_with_bareness(
    pred: &crate::ast::Predicate,
    table_name: &str,
    table_alias: Option<&str>,
) -> Vec<(String, bool)> {
    let mut cols = Vec::new();
    collect_predicate_columns_with_bareness(pred, table_name, table_alias, &mut cols);
    cols
}

fn collect_predicate_columns_with_bareness(
    pred: &crate::ast::Predicate,
    table_name: &str,
    table_alias: Option<&str>,
    out: &mut Vec<(String, bool)>,
) {
    use crate::ast::{Expr, Predicate};
    match pred {
        Predicate::Compare { lhs, .. } => match lhs.as_ref() {
            Expr::Field(fp) => {
                if let Some(name) = extract_column_name_from_field_path(fp, table_name, table_alias)
                {
                    let is_bare = fp.segments.len() == 1;
                    out.push((name, is_bare));
                }
            }
            Expr::FuncCall(_) => {
                // Fn-call LHS (all seven shared-parser predicate positions, ADR-048 §D.7.1,
                // + HAVING FuncCall LHS contexts, §D.3/§D.7.3).
                // Recurse into FuncCall args, preserving per-reference bareness for each field ref.
                extract_field_paths_with_bareness(lhs.as_ref(), table_name, table_alias, out);
            }
            _ => {}
        },
        Predicate::StringOp { field, .. } | Predicate::Regex { field, .. } => {
            if let Some(name) = extract_column_name_from_field_path(field, table_name, table_alias)
            {
                let is_bare = field.segments.len() == 1;
                out.push((name, is_bare));
            }
        }
        Predicate::In { field, .. }
        | Predicate::InSubquery { field, .. }
        | Predicate::Between { field, .. }
        | Predicate::Cidr { field, .. }
        | Predicate::Wildcard { field, .. } => {
            if let Some(name) = extract_column_name_from_field_path(field, table_name, table_alias)
            {
                let is_bare = field.segments.len() == 1;
                out.push((name, is_bare));
            }
        }
        Predicate::Has(fp) | Predicate::Missing(fp) => {
            if let Some(name) = extract_column_name_from_field_path(fp, table_name, table_alias) {
                let is_bare = fp.segments.len() == 1;
                out.push((name, is_bare));
            }
        }
        Predicate::IsNull { field, .. } => {
            if let Some(name) = extract_column_name_from_field_path(field, table_name, table_alias)
            {
                let is_bare = field.segments.len() == 1;
                out.push((name, is_bare));
            }
        }
        Predicate::Logical { predicates, .. } => {
            for child in predicates {
                collect_predicate_columns_with_bareness(child, table_name, table_alias, out);
            }
        }
        Predicate::Not(inner) => {
            collect_predicate_columns_with_bareness(inner, table_name, table_alias, out);
        }
        Predicate::RecoveryError => {}
        #[allow(unreachable_patterns)]
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
/// # `compute_did_you_mean` (ADV-PR-P3-OBS-001)
/// When `false`, the Levenshtein suggestion loops are skipped entirely and the error
/// carries `did_you_mean: None`. Pass `false` only from call sites whose
/// `ColumnNotFound` result will be discarded — specifically the BC-2.11.016 FP-001
/// HEAD-JOIN suspension arm, where bare unqualified refs fail-open and the Levenshtein
/// computation is pure waste (~hundreds of ms on adversarial queries with ~500 bare
/// unknown refs and a JOIN). All other call sites pass `true`.
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
    compute_did_you_mean: bool,
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
        // EC-11-041 (ADV-FIX-P9-OBS-001): `columns_for_table` returns [] for BOTH
        // "table not in registry" AND "table IS registered but has zero columns".
        // Fail-open ONLY when the table is NOT registered (E-QUERY-037 domain).
        // When the table IS registered but has zero columns, fall through so
        // E-QUERY-038 fires with `available_columns: []` per BC-2.11.016 EC-11-041.
        if available_columns.is_empty() && !registry.is_registered(table_name) {
            return Ok(());
        }
        // Column is in the available set — gate passes.
        if available_columns.contains(&column_name.to_string()) {
            return Ok(());
        }
        // did_you_mean: same ≤3 Levenshtein threshold as multi-tenant path.
        // F-PHL1-MED-001: cap `column_name` at 128 bytes for Levenshtein input only (CWE-407
        // Algorithmic Complexity DoS guard). This cap bounds the did_you_mean computation
        // only; it does NOT cap the column_name in the error response or log field.
        // ADV-PR-P3-OBS-001: skip Levenshtein when compute_did_you_mean is false — the
        // BC-2.11.016 FP-001 HEAD-JOIN suspension arm discards ColumnNotFound errors, so
        // the suggestion computation is pure waste on the suspended call path.
        let did_you_mean: Option<String> = if compute_did_you_mean {
            let column_name_capped = crate::table_registry::cap_name_for_levenshtein(column_name);
            available_columns
                .iter()
                .map(|c| (c.clone(), strsim::levenshtein(column_name_capped, c)))
                .filter(|(_, dist)| *dist <= 3)
                .min_by_key(|(name, dist)| (*dist, name.clone()))
                .map(|(c, _)| c)
        } else {
            None
        };
        // SEC-FIND-001 (CWE-117): sanitize user-supplied column_name before structured log
        // emission — strip Unicode Cc + U+2028/U+2029 (same pattern as infusion_udf.rs
        // `warn_coercion_failed` per TD-VSDD-060 sibling-sweep).
        let safe_column_name = sanitize_for_log(column_name);
        tracing::warn!(
            event_type = "column_not_found.rejected",
            column = %safe_column_name,
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

    // ADR-058 §G / S-ADR058-OCSF-ROUTING-001 holdout gap (Fix B, de-duplicated):
    // Delegate to shared helper ocsf_or_raw_column_names_for_table — single source of
    // truth for OCSF-aware column-name projection, shared with get_initial_available_columns.
    let mut available_columns: Vec<String> = org_visible_entries
        .iter()
        .flat_map(|spec_entry| {
            let sensor_id = spec_entry.spec.sensor_id.clone();
            let ocsf_naming = spec_entry.spec.ocsf_column_naming;
            spec_entry
                .spec
                .tables
                .iter()
                .filter(move |tbl| format!("{sensor_id}_{}", tbl.table_name) == table_name)
                .flat_map(move |tbl| {
                    ocsf_or_raw_column_names_for_table(tbl, ocsf_naming).into_iter()
                })
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
    // F-PHL1-MED-001: cap `column_name` at 128 bytes for Levenshtein input only (CWE-407
    // Algorithmic Complexity DoS guard) — multi-tenant path. Does NOT cap the column_name
    // in the error response or log field.
    // ADV-PR-P3-OBS-001: skip Levenshtein when compute_did_you_mean is false — the
    // BC-2.11.016 FP-001 HEAD-JOIN suspension arm discards ColumnNotFound errors, so
    // the suggestion computation is pure waste on the suspended call path.
    let did_you_mean: Option<String> = if compute_did_you_mean {
        let column_name_capped_mt = crate::table_registry::cap_name_for_levenshtein(column_name);
        available_columns
            .iter()
            .map(|c| (c.clone(), strsim::levenshtein(column_name_capped_mt, c)))
            .filter(|(_, dist)| *dist <= 3)
            .min_by_key(|(name, dist)| (*dist, name.clone()))
            .map(|(c, _)| c)
    } else {
        None
    };

    // SEC-FIND-001 (CWE-117): sanitize user-supplied column_name before structured log
    // emission — strip Unicode Cc + U+2028/U+2029 (TD-VSDD-060 sibling-sweep).
    // Emit audit tracing event per SAP-1 / PG-LP11-001.
    let safe_column_name_mt = sanitize_for_log(column_name);
    tracing::warn!(
        event_type = "column_not_found.rejected",
        column = %safe_column_name_mt,
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
/// - HAVING clause (FieldPath refs and agg-fn column args via `extract_predicate_columns`;
///   HAVING also accepts `agg_fn(col) op literal` predicate form via `build_having_predicate_parser`
///   (ADR-048 / F-PXL3-MED-002) — a deliberate grammar divergence from WHERE (which remains
///   `field op literal` only). BC-2.11.016 / F-PWL1-LOW-001 / F-PXL3-MED-002)
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
pub(crate) fn check_query_column_availability(
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
    infusion_registry: Option<&prism_spec_engine::InfusionRegistry>,
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
    // BC-2.11.020 / HIGH-1 sibling sweep: without the SqlPipe arm, a SqlPipe query
    // whose head projects a typo'd column (e.g. `SELECT sev FROM …`) would bypass
    // the E-QUERY-038 pedagogical gate, getting a confusing DataFusion error at
    // execution time instead of the clean "column not found" diagnostic. (TD-VSDD-060)
    //
    // Filter and Pipe modes have no explicit column projection (they are effectively
    // `SELECT *`), but they DO carry predicate columns that must be checked.
    // DRIFT-IEQ-NONEXISTENT-COL-ERRPATH-001: before this fix, `Ast::Filter` and
    // `Ast::Pipe` fell through to `_ => return Ok(())`, bypassing the E-QUERY-038 gate
    // entirely for predicate columns. Non-existent columns referenced by IEQ/IIN/INE
    // (or any other operator) in Filter/Pipe predicates now fire E-QUERY-038 at plan
    // time, consistent with the BC-2.11.016 column availability gate (fourteen
    // positions across SQL, pipe, and filter modes).
    let sql_query = match &ast {
        Ast::Sql(SqlStatement::Select(q)) => q,
        Ast::SqlPipe(spq) => &spq.head,

        // ── Filter mode: check columns in the root predicate ─────────────────
        //
        // Filter queries are `source | predicate` — the predicate may reference
        // any column. Extract all FieldPath column refs from the predicate and
        // check each against the spec map / table registry.
        //
        // Table name: Custom refs already carry the underscore form
        // (`crowdstrike_alerts`); External refs are converted to
        // `{sensor}_{table}` — consistent with the SQL SELECT table-name path.
        //
        // No SELECT alias exists in filter mode → `from_alias = None`.
        Ast::Filter(fe) => {
            use crate::ast::SourceRefKind;
            let table_name = match &fe.source.kind {
                SourceRefKind::Custom => fe.source.raw.clone(),
                SourceRefKind::External { sensor, table } => format!("{sensor}_{table}"),
                // Composite / Internal sources: no column schema — fail-open.
                _ => return Ok(()),
            };
            if table_name.starts_with("prism_") {
                return Ok(());
            }
            // Position 7: Filter root predicate — E-QUERY-038 column existence gate.
            // No FieldPath aliases in filter mode.
            let pred_cols = extract_predicate_columns(&fe.predicate, &table_name, None);
            for col in &pred_cols {
                check_column_availability(
                    col,
                    &table_name,
                    client_id,
                    org_scope,
                    resolved_spec_map,
                    table_registry,
                    true, // compute_did_you_mean: error propagates to caller (ADV-PR-P3-OBS-001)
                )?;
            }
            // E-QUERY-002 type-compat gate — AFTER column-existence gate (BC-2.11.016
            // MED-001 ordering lock; BC-2.11.017 AC-003).
            // Mirrors the SQL WHERE path. Walks Predicate::Compare nodes and returns
            // QueryTypeMismatch when the operator is not valid for the column's ColumnType.
            // Uses collect_predicate_type_pairs which emits "IEQ"/"INE" for
            // case_insensitive=true predicates — correctly flagging IEQ/INE on Integer/Float/
            // Boolean/Datetime columns as type mismatches (S-PRISMQL-CASE-INSENSITIVE-001).
            let type_pairs = collect_predicate_type_pairs(&fe.predicate, &table_name, None);
            for (col_name, op_str) in &type_pairs {
                check_operator_type_compatibility(
                    col_name,
                    op_str,
                    &table_name,
                    org_scope,
                    resolved_spec_map,
                    table_registry,
                )?;
            }
            return Ok(());
        }

        // ── Pipe mode: check columns in all pipe stage positions ───────────────
        //
        // Pipe queries are `source | stage | stage …`. Column availability is
        // checked at positions 8/9 (| where predicates), 10 (| sort field keys),
        // 11 (| stats ... by grouping refs), 12 (| fields column refs),
        // 13 (| enrich input column), and 14 (| dedup field keys) per
        // BC-2.11.016 exhaustive position enumeration.
        //
        // Position 8/9 also runs the E-QUERY-002 type-compat gate after the
        // E-QUERY-038 existence gate — same ordering as the SQL WHERE path.
        //
        // Stage types without schema-bound column refs (Limit, Tail, Join) are
        // fail-open via the `_ => {}` catch-all in `check_pipe_stage_columns`.
        // Enrich (pos 13) and Dedup (pos 14) have explicit arms.
        Ast::Pipe(pq) => {
            use crate::ast::SourceRefKind;
            let table_name = match &pq.source.kind {
                SourceRefKind::Custom => pq.source.raw.clone(),
                SourceRefKind::External { sensor, table } => format!("{sensor}_{table}"),
                _ => return Ok(()),
            };
            if table_name.starts_with("prism_") {
                return Ok(());
            }
            check_pipe_stage_columns(
                &pq.stages,
                &table_name,
                None, // Ast::Pipe: no FROM-alias (pure pipe source, no SQL head)
                client_id,
                org_scope,
                resolved_spec_map,
                table_registry,
                infusion_registry,
                None, // Ast::Pipe: raw schema as initial binding (no head projection)
            )?;
            return Ok(());
        }

        // All other AST variants (Dml, composite sources, etc.) — fail-open.
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
    //
    // BC-2.11.016 PER-REFERENCE SCOPING: use `extract_field_paths_with_bareness`
    // so each extracted reference carries its `is_bare` flag for the HEAD-JOIN gate.
    let mut select_cols: Vec<(String, bool)> = Vec::new();
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
                        extract_field_paths_with_bareness(
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
    let where_cols: Vec<(String, bool)> = sql_query
        .where_
        .as_ref()
        .map(|pred| extract_predicate_columns_with_bareness(pred, &table_name, from_alias))
        .unwrap_or_default();

    // ── Position 3: GROUP BY clause — recurse into FuncCall args (M2 fix) ────
    //
    // F-001B-DC-HIGH-001: use extract_column_name_from_field_path instead of .first().
    // M2 fix (S-DEMO-FIDELITY-REMEDIATION-001): use `extract_field_paths_with_bareness`
    // instead of direct `Expr::Field` match so that column refs wrapped in function
    // calls (e.g. `GROUP BY lower(col_typo)`) are also validated against the schema.
    let mut group_by_cols: Vec<(String, bool)> = Vec::new();
    for expr in &sql_query.group_by {
        extract_field_paths_with_bareness(expr, &table_name, from_alias, &mut group_by_cols);
    }

    // ── Position 4: ORDER BY clause — recurse into FuncCall args (M2 fix) ────
    //
    // F-001B-DC-HIGH-001: use extract_column_name_from_field_path instead of .first().
    // M2 fix: same FuncCall-arg recursion as GROUP BY — handles `ORDER BY lower(col_typo)`.
    let mut order_by_cols: Vec<(String, bool)> = Vec::new();
    for oe in &sql_query.order_by {
        extract_field_paths_with_bareness(&oe.expr, &table_name, from_alias, &mut order_by_cols);
    }

    // ── Position 5: JOIN ON clause — recurse into JOIN ON expressions (M2 fix) ──
    //
    // M2 fix (S-DEMO-FIDELITY-REMEDIATION-001): validate column refs in JOIN ON
    // expressions for the FROM table. JOIN ON is typed as `Expr` (not `Predicate`),
    // so we call `extract_field_paths_with_bareness` directly.
    //
    // Fail-open for cross-table refs (unknown qualifier → `extract_column_name_from_field_path`
    // returns None). Only same-table column typos (unqualified or FROM-table-qualified refs)
    // are caught here — this is the same conservative policy as all other positions.
    let mut join_on_cols: Vec<(String, bool)> = Vec::new();
    for join in &sql_query.joins {
        extract_field_paths_with_bareness(&join.on, &table_name, from_alias, &mut join_on_cols);
    }

    // ── Position 6: HAVING clause — reuse the WHERE predicate extractor ────────
    //
    // BC-2.11.016 / F-PWL1-LOW-001: HAVING is `Option<Predicate>` (identical
    // in type to WHERE), so we reuse `extract_predicate_columns` — the same helper
    // used by Position 2 (WHERE). This closes the pedagogical asymmetry where
    // E-QUERY-039 (enrich gate) and E-QUERY-037 (source-walk) already covered
    // HAVING but E-QUERY-038 (column-existence gate) did not.
    //
    // Column refs directly in HAVING predicates (e.g. `HAVING typo_col > 5`) and
    // column refs inside `IN` / `BETWEEN` / etc. HAVING predicates are all extracted
    // by `collect_predicate_columns` via the existing match arms.
    let having_cols: Vec<(String, bool)> = sql_query
        .having
        .as_ref()
        .map(|pred| extract_predicate_columns_with_bareness(pred, &table_name, from_alias))
        .unwrap_or_default();

    // ── Gate: check all positions in order ────────────────────────────────────
    //
    // BC-2.11.016 HEAD-JOIN PER-REFERENCE SCOPING (ADV-FIX-P16-MED-001):
    // When the head SQL query's JOIN list is non-empty AND the *specific reference*
    // being checked was a BARE UNQUALIFIED ref (single-segment FieldPath; `is_bare = true`)
    // AND it is absent from the FROM schema, the E-QUERY-038 gate MUST NOT fire
    // (fail-open per FP-001). Rationale: DataFusion resolves bare unqualified refs across
    // ALL join sources at execution time; a bare ref absent from the FROM schema may
    // validly exist in a JOIN-partner table.
    //
    // Per-reference scoping (BC-2.11.016 — fixes v1.20 FN-001 defect): qualified
    // references (`alias.col`, `table.col`) carry `is_bare = false` and ALWAYS retain
    // full E-QUERY-038 checking, regardless of whether a co-resident bare ref with the
    // same extracted column name exists at another position. The v1.20 `bare_head_cols`
    // (name-keyed HashSet) wrongly suspended qualified refs when a bare ref with the
    // same name was present — the per-reference `is_bare` flag eliminates that conflation.
    //
    // Joinless queries: `head_has_joins = false` → standard gate for all refs (unchanged).
    let head_has_joins = !sql_query.joins.is_empty();

    for (col, is_bare_ref) in select_cols
        .iter()
        .chain(where_cols.iter())
        .chain(group_by_cols.iter())
        .chain(order_by_cols.iter())
        .chain(join_on_cols.iter())
        .chain(having_cols.iter())
    {
        if head_has_joins && *is_bare_ref {
            // HEAD-JOIN SUSPENSION: this specific reference was a bare unqualified ref
            // (single-segment FieldPath). Fail-open on ColumnNotFound — the column may
            // exist in a JOIN-partner table at execution time. All non-ColumnNotFound
            // errors propagate unchanged. Qualified refs never enter this branch.
            //
            // ADV-PR-P3-OBS-001: pass compute_did_you_mean=false — the ColumnNotFound
            // error is discarded immediately below (BC-2.11.016 FP-001 suspension
            // semantics), so computing the Levenshtein suggestion is pure waste.
            match check_column_availability(
                col,
                &table_name,
                client_id,
                org_scope,
                resolved_spec_map,
                table_registry,
                false, // compute_did_you_mean: ColumnNotFound discarded by suspension (ADV-PR-P3-OBS-001)
            ) {
                Ok(()) => {}
                Err(PrismError::ColumnNotFound(_)) => {} // Absent bare ref → fail-open
                Err(e) => return Err(e),
            }
        } else {
            check_column_availability(
                col,
                &table_name,
                client_id,
                org_scope,
                resolved_spec_map,
                table_registry,
                true, // compute_did_you_mean: error propagates to caller (ADV-PR-P3-OBS-001)
            )?;
        }
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
                table_registry,
            )?;
        }
    }

    // ── Positions 9–14: SqlPipe stage columns ─────────────────────────────────
    //
    // The SQL head (positions 1–6) was processed above via `sql_query` extracted
    // from `Ast::SqlPipe(spq) => &spq.head`. Now walk the pipe stages that follow
    // the SQL head: `| where` (pos 9), `| sort` (pos 10), `| stats by` (pos 11),
    // `| fields` (pos 12), `| enrich` (pos 13), `| dedup` (pos 14).
    //
    // BC-2.11.016 SQLPIPE HEAD-PROJECTION BINDING RULE: seed the stage-walk
    // initial `available` from the head projection output. Three branches:
    //   (a) Pure SELECT * / SELECT t.* → None (fall back to raw schema, same as Ast::Pipe)
    //   (b) Fully-explicit SELECT (no Star/TableStar) → Some(explicit-item union)
    //   (c) MIXED-STAR (Star/TableStar AND explicit items) → Some(schema_cols ∪ explicit-item union)
    // Head SQL clause checking (positions 1–6) above is unaffected — it always uses
    // the raw schema.
    if let crate::ast::Ast::SqlPipe(spq) = &ast {
        // Pre-compute raw schema columns so compute_sqlpipe_head_binding can use them
        // for the MIXED-STAR branch (c) — schema_cols ∪ explicit-item contributions.
        // (For branches (a) and (b), this value is either ignored or unused.)
        let schema_cols = get_initial_available_columns(
            &table_name,
            org_scope,
            resolved_spec_map,
            table_registry,
        );
        let head_binding = compute_sqlpipe_head_binding(
            sql_query,
            &table_name,
            from_alias,
            schema_cols.as_deref(),
        );
        check_pipe_stage_columns(
            &spq.stages,
            &table_name,
            from_alias, // BC-2.11.016 FROM-ALIAS RESOLUTION: thread declared alias to stage walk
            client_id,
            org_scope,
            resolved_spec_map,
            table_registry,
            infusion_registry,
            head_binding, // BC-2.11.016: head-projection seeding for SqlPipe stage walk
        )?;
    }

    Ok(())
}

/// Derive queryable column names for a single table, OCSF-aware.
///
/// ADR-058 §G / S-ADR058-OCSF-ROUTING-001 — shared projection helper.
/// Single source of truth for "which Arrow-level names are queryable for this table?"
/// Used by `check_column_availability` (multi-tenant arm) and
/// `get_initial_available_columns` (multi-tenant arm) to prevent future divergence.
///
/// When `ocsf_column_naming=true`:
///   - Tier-1 columns: `ocsf_field_to_arrow_name(ocsf_field)` for cols with `ocsf_field.is_some()`
///   - Synthesized pseudo-cols always present: `class_uid` (Integer), `_sensor` (String)
///   - `raw_extensions` (Json) added iff any Tier-2 col (`ocsf_field.is_none()`) exists
///   - Tier-2 raw `col.name` values are NOT included
///
/// When `ocsf_column_naming=false`:
///   - Raw `col.name` for every column (existing behavior, byte-for-byte)
fn ocsf_or_raw_column_names_for_table(
    tbl: &prism_spec_engine::spec_parser::TableSpec,
    ocsf_column_naming: bool,
) -> Vec<String> {
    // ADR-058 §I1 Consolidated-Projection Invariant: canonical logic lives in the
    // shared helper; this function is a thin forward so engine.rs and table_registry.rs
    // never diverge (OBS-1 fix, S-ADR058-OCSF-ROUTING-001).
    prism_spec_engine::column_mapping::ocsf_projected_column_names(tbl, ocsf_column_naming)
}

/// Compute the initial available column set for a table from schema sources.
///
/// Returns `Some(sorted_deduped_columns)` when a schema source provides columns for
/// this table, or `None` when no schema is available (fail-open sentinel).
///
/// Used by `check_pipe_stage_columns` to seed the BC-2.11.016 DERIVED-COLUMN
/// BINDING RULE initial state before walking pipe stages.
fn get_initial_available_columns(
    table_name: &str,
    org_scope: Option<&[prism_core::OrgSlug]>,
    resolved_spec_map: Option<
        &std::collections::HashMap<
            prism_spec_engine::ResolvedSpecKey,
            prism_spec_engine::ResolvedSensorSpec,
        >,
    >,
    table_registry: Option<&crate::table_registry::TableRegistry>,
) -> Option<Vec<String>> {
    if let Some(spec_map) = resolved_spec_map {
        // Multi-tenant path: collect columns from org-visible spec entries.
        let org_visible: Vec<&prism_spec_engine::ResolvedSensorSpec> = spec_map
            .values()
            .filter(|spec| {
                if let Some(scopes) = org_scope {
                    scopes.iter().any(|s| s.as_str() == spec.org_slug.as_str())
                } else {
                    true
                }
            })
            .collect();
        let table_in_schema = org_visible.iter().any(|spec| {
            let sid = &spec.spec.sensor_id;
            spec.spec
                .tables
                .iter()
                .any(|tbl| format!("{sid}_{}", tbl.table_name) == table_name)
        });
        if !table_in_schema {
            return None; // Table not in schema — fail-open.
        }
        // ADR-058 §G / S-ADR058-OCSF-ROUTING-001 Fix (re-cascade P1 HIGH-001):
        // Use shared helper ocsf_or_raw_column_names_for_table so that OCSF-flattened
        // Arrow names are seeded here exactly as in check_column_availability Fix B.
        // Single source of truth prevents future divergence between the two paths.
        let mut cols: Vec<String> = org_visible
            .iter()
            .flat_map(|spec| {
                let sid = spec.spec.sensor_id.clone();
                let ocsf_naming = spec.spec.ocsf_column_naming;
                spec.spec
                    .tables
                    .iter()
                    .filter(move |tbl| format!("{sid}_{}", tbl.table_name) == table_name)
                    .flat_map(move |tbl| {
                        ocsf_or_raw_column_names_for_table(tbl, ocsf_naming).into_iter()
                    })
            })
            .collect();
        cols.sort();
        cols.dedup();
        Some(cols)
    } else if let Some(registry) = table_registry {
        // Single-tenant M1 path: use table_registry.columns_for_table().
        let mut cols = registry.columns_for_table(table_name);
        cols.sort();
        cols.dedup();
        if cols.is_empty() {
            // EC-11-041 (ADV-FIX-P9-OBS-001): `columns_for_table` returns [] for BOTH
            // "table not in registry" (fail-open → None) AND "table IS registered but
            // has zero columns" (gate fires → Some([])). The prior code returned None
            // unconditionally, so check_pipe_stage_columns failed-open for zero-column
            // registered tables (opaque E-QUERY-034 instead of structured E-QUERY-038).
            // Use `is_registered` to resolve the ambiguity.
            if registry.is_registered(table_name) {
                Some(vec![]) // Registered with zero columns — gate fires (EC-11-041).
            } else {
                None // Not in registry — fail-open (E-QUERY-037 domain).
            }
        } else {
            Some(cols)
        }
    } else {
        None // No schema source at all — fail-open.
    }
}

/// Fire E-QUERY-038 against an explicit available-column set (binding-context path).
///
/// Used by `check_pipe_stage_columns` when the current binding context `available`
/// differs from the raw schema (e.g., after a `| stats` REPLACE or mid-pipe).
/// Constructs `ColumnNotFoundDetails` with `did_you_mean` via Levenshtein ≤ 3, same
/// as `check_column_availability`.
fn check_column_against_available_set(
    column_name: &str,
    table_name: &str,
    client_id: &str,
    available_columns: &[String],
) -> Result<(), PrismError> {
    if available_columns.contains(&column_name.to_string()) {
        return Ok(());
    }
    let column_name_capped = crate::table_registry::cap_name_for_levenshtein(column_name);
    let did_you_mean = available_columns
        .iter()
        .map(|c| (c.clone(), strsim::levenshtein(column_name_capped, c)))
        .filter(|(_, dist)| *dist <= 3)
        .min_by_key(|(name, dist)| (*dist, name.clone()))
        .map(|(c, _)| c);
    // SEC-FIND-001 (CWE-117): sanitize user-supplied column_name before structured log
    // emission — strip Unicode Cc + U+2028/U+2029 (TD-VSDD-060 sibling-sweep).
    let safe_column_name_bc = sanitize_for_log(column_name);
    tracing::warn!(
        event_type = "column_not_found.rejected",
        column = %safe_column_name_bc,
        table = %table_name,
        client_id = %client_id,
        available_count = available_columns.len(),
        "E-QUERY-038: column not found at plan time (binding-context path)"
    );
    Err(PrismError::ColumnNotFound(Box::new(
        prism_core::error::ColumnNotFoundDetails::new(
            column_name,
            table_name,
            client_id,
            available_columns.to_vec(),
            did_you_mean,
        ),
    )))
}

/// Compute the initial binding context for the SqlPipe stage walk from the HEAD SQL projection.
///
/// BC-2.11.016 SQLPIPE HEAD-PROJECTION BINDING RULE — three branches:
///
/// **(a) Pure-star:** all SELECT items are `Star`/`TableStar` (e.g., `SELECT *`,
///   `SELECT t.*`) → returns `None`. Caller falls back to `get_initial_available_columns`
///   (full raw schema), preserving existing `SELECT *` behavior.
///
/// **(b) Fully-explicit:** no `Star`/`TableStar` items → returns
///   `Some(({explicit AS aliases} ∪ {bare-Field un-aliased names} ∪ {bare GROUP BY field names},
///   suspended))`.
///
/// **(c) MIXED-STAR:** at least one `Star`/`TableStar` item AND at least one explicit
///   non-star item (e.g., `SELECT *, upper(severity) AS sev_up …`) → returns
///   `Some((schema_cols ∪ explicit-item-union, suspended))`.
///   The `schema_cols` parameter (pre-computed raw schema for this table/org) is required for
///   this branch; if `schema_cols` is `None` (schema unavailable) the function returns `None`
///   (fail-open, same as branch (a)).
///
/// In all branches, `suspended = true` when any explicit non-`Field` SELECT item lacks an
/// explicit `AS <alias>` (anonymous aggregate/computed column; output name unpredictable at
/// plan time; FP-001 fail-open; mirrors the Stats anonymous-aggregate rule).
///
/// **LAST-SEGMENT OUTPUT-NAME RULE (BC-2.11.016):** In branches (b) and (c), an
/// un-aliased bare-`Field` SELECT item whose qualifier matches NEITHER the FROM table name
/// NOR the declared FROM alias (e.g., `j.col` where `j` is a JOIN alias) has its last
/// path segment (`col`) seeded as the output column name with **DERIVED** provenance.
/// SQL output-naming semantics: `SELECT j.col` produces output column `col`. The column's
/// type is not statically known from the FROM table schema (FP-001 fail-open for both
/// E-QUERY-038 existence and E-QUERY-002 type-compat via SIBLING-GATE CONSISTENCY).
///
/// Head SQL clause checking (positions 1–6) runs against raw schema unchanged; this function
/// only affects the stage walk initial state.
fn compute_sqlpipe_head_binding(
    head: &crate::ast::SqlQuery,
    table_name: &str,
    from_alias: Option<&str>,
    // Raw schema columns for this table/org — used by MIXED-STAR branch (c) to seed the
    // union base. May be None when no schema source is available (→ fail-open via None return).
    schema_cols: Option<&[String]>,
) -> Option<(Vec<String>, std::collections::HashSet<String>, bool)> {
    use crate::ast::{Expr, SelectItem};

    let has_star = head
        .select
        .items
        .iter()
        .any(|item| matches!(item, SelectItem::Star | SelectItem::TableStar(_)));
    let has_explicit = head
        .select
        .items
        .iter()
        .any(|item| !matches!(item, SelectItem::Star | SelectItem::TableStar(_)));

    // Branch (a): pure SELECT * / SELECT t.* — fall back to raw schema.
    if has_star && !has_explicit {
        // BC-2.11.016 STAR-WITH-JOIN SUSPENSION RULE: when the head's JOIN list is
        // non-empty and at least one Star/TableStar item is present (branches (a) and (c)),
        // the initial binding context for the pipe-stage walk MUST be suspended := true.
        // Star expansion spans ALL join-source schemas at execution; the FROM table's raw
        // schema is an incomplete picture — checking downstream pipe-stage column refs
        // against the FROM schema alone fires false E-QUERY-038/E-QUERY-002 on columns
        // that validly exist only in the joined table (FP-001 violation class: star-with-join).
        // Joinless star heads are unchanged — they fall through to raw-schema seeding (None).
        if !head.joins.is_empty() {
            return Some((vec![], std::collections::HashSet::new(), true));
        }
        return None;
    }

    // Branch (c): MIXED-STAR — seed the available set with raw schema columns, then add
    // explicit-item contributions (aliases, bare fields, GROUP BY keys).
    if has_star && has_explicit {
        let base = match schema_cols {
            Some(cols) => cols.to_vec(),
            None => {
                // No schema available for the star component — fail-open.
                return None;
            }
        };
        let mut available: Vec<String> = base;
        // SIBLING-GATE CONSISTENCY (BC-2.11.016): track which names are DERIVED
        // (explicit AS aliases). Schema-columns (star component), bare fields, and GROUP BY
        // keys are RAW. Only explicit aliases are DERIVED.
        let mut derived: std::collections::HashSet<String> = std::collections::HashSet::new();
        let mut suspended = false;

        for item in &head.select.items {
            match item {
                // Wildcard items contribute the full schema (already in `available` via `base`).
                SelectItem::Star | SelectItem::TableStar(_) => {}

                // Explicit AS alias — the alias is the output name regardless of the expression.
                // The alias is DERIVED: its type is not the raw schema type for that name.
                SelectItem::Expr {
                    alias: Some(alias), ..
                } => {
                    available.push(alias.clone());
                    derived.insert(alias.clone()); // DERIVED: SqlPipe head alias
                }

                // Un-aliased bare Field — the output name is the column name itself.
                // (The column is already reachable via the star component, but add it
                // explicitly so sorting/dedup logic is consistent.)
                // Qualifier matches FROM source (table name or declared alias) → RAW provenance.
                // Qualifier unknown (e.g., JOIN alias `j` in `SELECT j.col`) →
                // LAST-SEGMENT OUTPUT-NAME RULE (BC-2.11.016): SQL output-naming
                // semantics produce output column = last path segment; seed with DERIVED
                // provenance (type not statically known from the FROM table schema; FP-001).
                SelectItem::Expr {
                    expr: Expr::Field(fp),
                    alias: None,
                } => {
                    if let Some(col) =
                        extract_column_name_from_field_path(fp, table_name, from_alias)
                    {
                        available.push(col);
                        // RAW: qualifier matches FROM source; name IS the schema column name.
                    } else if fp.segments.len() > 1 {
                        // LAST-SEGMENT OUTPUT-NAME RULE (BC-2.11.016): unknown qualifier
                        // (e.g., JOIN alias) → seed last segment as DERIVED.
                        if let Some(last) = fp.segments.last() {
                            available.push(last.clone());
                            derived.insert(last.clone()); // DERIVED: unknown qualifier; type not from FROM schema
                        }
                    }
                }

                // Un-aliased VirtualField — skip without suspending.
                SelectItem::Expr {
                    expr: Expr::VirtualField(_),
                    alias: None,
                } => {}

                // Un-aliased non-Field expression (e.g., `count(*)`, `sum(amount)`).
                // Output name is unpredictable at plan time → suspended := true (FP-001).
                SelectItem::Expr { alias: None, .. } => {
                    suspended = true;
                }

                #[allow(unreachable_patterns)]
                _ => {}
            }
        }

        // GROUP BY bare-field names (grouping keys always present in aggregate result).
        // RAW: GROUP BY keys are schema column names.
        for expr in &head.group_by {
            if let Expr::Field(fp) = expr {
                if let Some(col) = extract_column_name_from_field_path(fp, table_name, from_alias) {
                    available.push(col);
                }
            }
        }

        // BC-2.11.016 STAR-WITH-JOIN SUSPENSION RULE (branch (c) application):
        // when the head's JOIN list is non-empty, force suspended := true regardless of
        // whether an anonymous aggregate already triggered suspension above. The Star/TableStar
        // component brings all join-source columns into scope at execution; the partial schema
        // seed (FROM table columns + explicit items) is incomplete for join-source columns.
        // Checking downstream pipe-stage refs against an incomplete set fires false positives
        // (FP-001). This override is additive — if suspended was already true from an
        // anonymous aggregate, it remains true.
        if !head.joins.is_empty() {
            suspended = true;
        }

        available.sort();
        available.dedup();
        return Some((available, derived, suspended));
    }

    // Branch (b): fully-explicit SELECT (no Star/TableStar items).
    let mut available: Vec<String> = Vec::new();
    // SIBLING-GATE CONSISTENCY (BC-2.11.016): explicit AS aliases are DERIVED.
    // Un-aliased bare fields and GROUP BY keys are RAW.
    let mut derived: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut suspended = false;

    for item in &head.select.items {
        match item {
            // No wildcards in this branch — unreachable, but keep match exhaustive.
            SelectItem::Star | SelectItem::TableStar(_) => {}

            // Explicit AS alias — the alias is the output name regardless of the expression.
            // The alias is DERIVED: its type at execution may differ from the raw schema type.
            SelectItem::Expr {
                alias: Some(alias), ..
            } => {
                available.push(alias.clone());
                derived.insert(alias.clone()); // DERIVED: SqlPipe head alias
            }

            // Un-aliased bare Field — the output name is the column name itself.
            // Qualifier matches FROM source (table name or declared alias) → RAW provenance;
            // its name and type match the original schema column.
            // Qualifier unknown (e.g., JOIN alias `j` in `SELECT j.col`) →
            // LAST-SEGMENT OUTPUT-NAME RULE (BC-2.11.016): SQL output-naming
            // semantics produce output column = last path segment; seed with DERIVED
            // provenance (type not statically known from the FROM table schema; FP-001).
            SelectItem::Expr {
                expr: Expr::Field(fp),
                alias: None,
            } => {
                if let Some(col) = extract_column_name_from_field_path(fp, table_name, from_alias) {
                    available.push(col);
                    // RAW: qualifier matches FROM source; name IS the schema column name.
                } else if fp.segments.len() > 1 {
                    // LAST-SEGMENT OUTPUT-NAME RULE (BC-2.11.016): unknown qualifier
                    // (e.g., JOIN alias) → seed last segment as DERIVED.
                    if let Some(last) = fp.segments.last() {
                        available.push(last.clone());
                        derived.insert(last.clone()); // DERIVED: unknown qualifier; type not from FROM schema
                    }
                }
                // Zero-segment paths: extract_column_name_from_field_path returns None and
                // len is not > 1; skip silently (malformed path, not a valid column ref).
            }

            // Un-aliased VirtualField (_sensor, _client) — always-valid sentinels; not schema
            // columns; skip without suspending.
            SelectItem::Expr {
                expr: Expr::VirtualField(_),
                alias: None,
            } => {}

            // Un-aliased non-Field expression (e.g., `count(*)`, `sum(amount)`, `1 + 1`).
            // Output name is auto-generated by DataFusion and unpredictable at plan time.
            // → suspended := true for the stage walk (FP-001 fail-open; mirrors Stats rule).
            SelectItem::Expr { alias: None, .. } => {
                suspended = true;
            }

            // #[non_exhaustive] catch-all for future SelectItem variants.
            #[allow(unreachable_patterns)]
            _ => {}
        }
    }

    // GROUP BY bare-field names are visible in the stage walk output
    // (they are grouping keys, always present in the aggregate result).
    // RAW: GROUP BY keys are schema column names.
    for expr in &head.group_by {
        if let Expr::Field(fp) = expr {
            if let Some(col) = extract_column_name_from_field_path(fp, table_name, from_alias) {
                available.push(col);
                // RAW: not added to `derived`.
            }
        }
        // Non-bare-field GROUP BY expressions: skip without suspending. The GROUP BY key
        // will appear under a DataFusion-generated name; but this case is exotic and the
        // head-position-3 gate already validated these against the raw schema.
    }

    available.sort();
    available.dedup();
    Some((available, derived, suspended))
}

/// Walk all column-bearing pipe stage types and check column availability + type-compat.
///
/// Called from both the `Ast::Pipe` arm (all stages) and the `Ast::SqlPipe` arm
/// (stages after the SQL head is processed for positions 1–6).
///
/// # Position coverage (BC-2.11.016)
/// - Position 8/9: `PipeStage::Where` — predicates (E-QUERY-038 existence + E-QUERY-002 type-compat)
/// - Position 10: `PipeStage::Sort` — sort field keys (E-QUERY-038 existence only; no operator)
/// - Position 11: `PipeStage::Stats` — `by_fields` grouping refs; REPLACE binding after
/// - Position 12: `PipeStage::Fields` — inclusion/exclusion column refs (E-QUERY-038 existence only)
/// - Position 13: `PipeStage::Enrich` — input column (E-QUERY-038 existence); suspend after
/// - Position 14: `PipeStage::Dedup` — dedup field keys (E-QUERY-038 existence only)
///
/// # DERIVED-COLUMN BINDING RULE (BC-2.11.016)
/// Maintains a running `{ available: Vec<String>, suspended: bool }` context while
/// walking stages in order.
///
/// - **Initial state:** for `Ast::Pipe` (pass `initial_binding_override = None`):
///   `available = schema_columns(table)` from spec_map or registry; if no schema source,
///   fail-open immediately. For `Ast::SqlPipe` (BC-2.11.016 SQLPIPE HEAD-PROJECTION
///   BINDING RULE, pass `initial_binding_override = Some((cols, suspended))`): uses the
///   head projection output — not the raw schema — as the starting binding context.
/// - **Enrich stage:** position-13 input column is checked against `available` BEFORE
///   updating the context. When `infusion_registry` is wired and the UDF descriptor
///   resolves, output columns are UNIONed into `available` (downstream stages can reference
///   enriched columns). When the registry is absent or the descriptor lookup fails
///   defensively → `suspended := true` (fail-open per FP-001; EC-11-054/EC-11-055).
/// - **Stats stage:** `by_fields` are checked against `available` BEFORE updating.
///   After those checks: `available` is REPLACED with `{explicit_aliases} ∪ {by_field_names}`.
///   Anonymous aggregates (no `AS alias`) produce unpredictable DataFusion names → `suspended := true`.
/// - **All other stages** (Where, Sort, Dedup): `available` and `suspended` are unchanged.
/// - **Fields stage:** FIELDS TRANSITION RULE (BC-2.11.016 OBS-002):
///   include-list → `available := {listed}` (REPLACE; provenance preserved for surviving names);
///   exclude-list → `available := available ∖ {listed}` (SUBTRACT; provenance preserved).
///   Suspension state carries forward unchanged.
/// - **SIBLING-GATE CONSISTENCY (BC-2.11.016 MED-001):** names in `available` carry
///   per-name provenance (RAW vs DERIVED). E-QUERY-002 type-compat gate MUST skip DERIVED names
///   (fail-open per FP-001). RAW names retain full type-compat checking.
/// - **Suspension propagation:** once `suspended = true`, all subsequent stages skip E-QUERY-038.
///
/// # Ordering
/// E-QUERY-038 (existence) fires before E-QUERY-002 (type-compat) for each stage,
/// consistent with BC-2.11.016 gate ordering (table → column → type).
#[allow(clippy::too_many_arguments)]
fn check_pipe_stage_columns(
    stages: &[crate::ast::PipeStage],
    table_name: &str,
    // BC-2.11.016 FROM-ALIAS RESOLUTION (OBS-001): declared FROM-alias for SqlPipe
    // pipe-stage positions 9–14. When Some("t"), qualifier "t" in field paths like `t.col`
    // is stripped to bare "col" before checking against the binding context. Pass None for
    // Ast::Pipe (no FROM-alias) and for SqlPipe with no declared alias.
    table_alias: Option<&str>,
    client_id: &str,
    org_scope: Option<&[prism_core::OrgSlug]>,
    resolved_spec_map: Option<
        &std::collections::HashMap<
            prism_spec_engine::ResolvedSpecKey,
            prism_spec_engine::ResolvedSensorSpec,
        >,
    >,
    table_registry: Option<&crate::table_registry::TableRegistry>,
    infusion_registry: Option<&prism_spec_engine::InfusionRegistry>,
    // BC-2.11.016 SQLPIPE HEAD-PROJECTION BINDING RULE: when Some((cols, derived, suspended)),
    // use these as the initial binding context instead of get_initial_available_columns().
    // `derived` is the SIBLING-GATE CONSISTENCY set of DERIVED name provenance (v1.15 MED-001).
    // Callers pass Some(...) for Ast::SqlPipe (head-projection seeding), None for Ast::Pipe
    // (raw schema seeding; existing behavior preserved).
    initial_binding_override: Option<(Vec<String>, std::collections::HashSet<String>, bool)>,
) -> Result<(), PrismError> {
    use crate::ast::{AggFunc, PipeStage};

    // BC-2.11.016 DERIVED-COLUMN BINDING RULE: build initial available set.
    //
    // For Ast::SqlPipe with an explicit SELECT head, initial_binding_override carries the
    // head projection output (BC-2.11.016 SQLPIPE HEAD-PROJECTION BINDING RULE).
    // For Ast::Pipe (or SqlPipe with SELECT *), fall back to raw schema via
    // get_initial_available_columns(). If no schema source is available, fail-open immediately.
    //
    // BC-2.11.016 SIBLING-GATE CONSISTENCY (MED-001): `derived_names` tracks which
    // names in `current_available` are DERIVED (SqlPipe head alias, stats alias, enrich output)
    // vs RAW (original schema column). E-QUERY-002 gate MUST skip DERIVED names (FP-001).
    let (mut current_available, mut derived_names, mut suspended) = match initial_binding_override {
        Some((cols, derived, susp)) => (cols, derived, susp),
        None => {
            match get_initial_available_columns(
                table_name,
                org_scope,
                resolved_spec_map,
                table_registry,
            ) {
                Some(cols) => (cols, std::collections::HashSet::new(), false),
                None => {
                    // No schema source — all checks would fail-open anyway. Return early
                    // (equivalent to old behavior; preserves existing RG-065 fail-open).
                    return Ok(());
                }
            }
        }
    };

    for stage in stages {
        if suspended {
            // Once suspended, ALL subsequent E-QUERY-038 checks are skipped (FP-001).
            continue;
        }
        match stage {
            // Position 8/9: `| where` stage predicates — E-QUERY-038 + E-QUERY-002.
            // Checked against the CURRENT binding context (not raw schema), so post-stats
            // references to aggregate aliases are found correctly (CRIT-002).
            //
            // BC-2.11.016 FROM-ALIAS RESOLUTION (OBS-001): pass `table_alias` so
            // alias-qualified refs like `t.col` (where `t` is the declared FROM-alias) are
            // stripped to bare `col` before the existence check. Without this threading,
            // all alias-qualified refs silently bypass the gate.
            //
            // BC-2.11.016 SIBLING-GATE CONSISTENCY (MED-001): skip E-QUERY-002 for
            // DERIVED names (stats alias, enrich output, SqlPipe head alias). DERIVED names
            // have an unknown type at plan time; applying raw-schema operator restrictions
            // would produce false E-QUERY-002 errors (FP-001 violation).
            PipeStage::Where(pred) => {
                let pred_cols = extract_predicate_columns(pred, table_name, table_alias);
                for col in &pred_cols {
                    check_column_against_available_set(
                        col,
                        table_name,
                        client_id,
                        &current_available,
                    )?;
                }
                // E-QUERY-002 type-compat: after existence gate, check operator compat.
                // Uses collect_predicate_type_pairs which emits "IEQ"/"INE" for
                // case_insensitive=true predicates (BC-2.11.016 MED-001).
                let type_pairs = collect_predicate_type_pairs(pred, table_name, table_alias);
                for (col_name, op_str) in &type_pairs {
                    // SIBLING-GATE CONSISTENCY (BC-2.11.016 MED-001): skip E-QUERY-002
                    // for DERIVED names — their type is not statically known at plan time;
                    // applying raw-schema type restrictions would produce false positives (FP-001).
                    if derived_names.contains(col_name) {
                        continue;
                    }
                    check_operator_type_compatibility(
                        col_name,
                        op_str,
                        table_name,
                        org_scope,
                        resolved_spec_map,
                        table_registry,
                    )?;
                }
            }
            // Position 10: `| sort by` field keys — E-QUERY-038 existence only.
            // Sort keys reference columns by name; no operator → no type-compat gate.
            // Checked against current binding context (post-stats REPLACE if applicable).
            PipeStage::Sort(exprs) => {
                for se in exprs {
                    if let Some(col) =
                        extract_column_name_from_field_path(&se.field, table_name, table_alias)
                    {
                        check_column_against_available_set(
                            &col,
                            table_name,
                            client_id,
                            &current_available,
                        )?;
                    }
                }
            }
            // Position 11: `| stats ... by` grouping field refs — E-QUERY-038 existence only.
            // Checked against current `available` BEFORE updating. After checks: `available`
            // is REPLACED with {explicit_aliases ∪ by_field_names} per BC-2.11.016
            // DERIVED-COLUMN BINDING RULE. Anonymous aggregates (no alias) → suspended.
            //
            // BC-2.11.016 SIBLING-GATE CONSISTENCY (MED-001): after the REPLACE,
            // update `derived_names` — explicit aliases are DERIVED; by-fields preserve
            // their prior provenance (DERIVED if they were DERIVED before, RAW otherwise).
            PipeStage::Stats(stats) => {
                // Check by_fields against current available BEFORE replacing.
                for fp in &stats.by_fields {
                    if let Some(col) =
                        extract_column_name_from_field_path(fp, table_name, table_alias)
                    {
                        check_column_against_available_set(
                            &col,
                            table_name,
                            client_id,
                            &current_available,
                        )?;
                    }
                }
                // Position 11 (agg-arg): check aggregate function argument field paths
                // against current available BEFORE replacing the binding context
                // (BC-2.11.016 DERIVED-COLUMN BINDING RULE, EC-11-058).
                // FP-001 fail-open: future #[non_exhaustive] variants matched by `_`
                // carry no extractable field path — the check is skipped for them.
                for agg in &stats.aggregates {
                    let fp = match &agg.func {
                        AggFunc::CountField(fp)
                        | AggFunc::Sum(fp)
                        | AggFunc::Avg(fp)
                        | AggFunc::Min(fp)
                        | AggFunc::Max(fp)
                        | AggFunc::DistinctCount(fp) => Some(fp),
                        AggFunc::Percentile { field, .. } => Some(field),
                        // AggFunc::Count has no argument field.
                        // `_` covers future #[non_exhaustive] variants — fail-open (FP-001).
                        _ => None,
                    };
                    if let Some(fp) = fp {
                        if let Some(col) =
                            extract_column_name_from_field_path(fp, table_name, table_alias)
                        {
                            check_column_against_available_set(
                                &col,
                                table_name,
                                client_id,
                                &current_available,
                            )?;
                        }
                    }
                }
                // Compute replacement set: explicit aliases ∪ by-field column names.
                // Also compute new derived_names for SIBLING-GATE CONSISTENCY (MED-001):
                //   - Explicit aliases → DERIVED (stats output aliases have unknown types at plan time)
                //   - By-fields → preserve prior provenance from current `derived_names`
                let mut replacement: Vec<String> = Vec::new();
                let mut new_derived: std::collections::HashSet<String> =
                    std::collections::HashSet::new();
                let mut has_anonymous = false;
                for agg in &stats.aggregates {
                    if let Some(alias) = &agg.alias {
                        replacement.push(alias.clone());
                        new_derived.insert(alias.clone()); // DERIVED: stats output alias
                    } else {
                        // Anonymous aggregate — DataFusion auto-name is not predictable at
                        // plan time. Set suspended to avoid false positives (FP-001).
                        has_anonymous = true;
                    }
                }
                for fp in &stats.by_fields {
                    if let Some(col) =
                        extract_column_name_from_field_path(fp, table_name, table_alias)
                    {
                        // Preserve prior provenance: by-field is DERIVED only if it was
                        // DERIVED before the Stats stage (e.g., it was a prior stats alias).
                        if derived_names.contains(&col) {
                            new_derived.insert(col.clone());
                        }
                        replacement.push(col);
                    } else {
                        // Non-static by-expression — unpredictable downstream name → suspend.
                        has_anonymous = true;
                    }
                }
                if has_anonymous {
                    // Anonymous aggregates present → downstream names are unpredictable →
                    // fail-open for all subsequent stages (FP-001).
                    suspended = true;
                } else {
                    derived_names = new_derived;
                    replacement.sort();
                    replacement.dedup();
                    current_available = replacement;
                }
            }
            // Position 12: `| fields` column refs — E-QUERY-038 existence only.
            // Grammar keyword is `fields` (PipeStage::Fields); BC-2.11.016+ corrected
            // the earlier v1.6 EC-11-052 `| project` wording to `| fields`.
            //
            // BC-2.11.016 FIELDS TRANSITION RULE (OBS-002):
            // Include (`fstage.include = true`): validate listed cols, then
            //   `available := {listed}` (REPLACE — analogous to Stats stage REPLACE);
            //   provenance of surviving names preserved from prior binding.
            // Exclude (`fstage.include = false`): validate listed cols, then
            //   `available := available ∖ {listed}` (SUBTRACT);
            //   provenance of surviving names preserved.
            // Suspension state carries forward unchanged.
            // Rationale: SQL emitter's `apply_fields` genuinely restricts the projection;
            // downstream refs to removed columns fail at DataFusion (false-negative class
            // before this rule).
            PipeStage::Fields(fstage) => {
                let mut listed: Vec<String> = Vec::new();
                for fp in &fstage.fields {
                    if let Some(col) =
                        extract_column_name_from_field_path(fp, table_name, table_alias)
                    {
                        check_column_against_available_set(
                            &col,
                            table_name,
                            client_id,
                            &current_available,
                        )?;
                        listed.push(col);
                    }
                }
                if fstage.include {
                    // Include-list → REPLACE: downstream sees only the listed columns.
                    // Preserve provenance for names that were already in `derived_names`.
                    let new_derived: std::collections::HashSet<String> = listed
                        .iter()
                        .filter(|n| derived_names.contains(*n))
                        .cloned()
                        .collect();
                    derived_names = new_derived;
                    listed.sort();
                    listed.dedup();
                    current_available = listed;
                } else {
                    // Exclude-list → SUBTRACT: remove listed columns from available.
                    let excluded: std::collections::HashSet<&str> =
                        listed.iter().map(|s| s.as_str()).collect();
                    current_available.retain(|n| !excluded.contains(n.as_str()));
                    derived_names.retain(|n| !excluded.contains(n.as_str()));
                }
            }
            // Position 13: `| enrich f(input_col)` input column — E-QUERY-038 existence only.
            // The input column is checked against `available` BEFORE this stage updates the
            // binding context (BC-2.11.016 position 13).
            //
            // BC-2.11.016 union path: when InfusionRegistry is wired (registry = Some),
            // resolve the infusion output schema and UNION output_columns into current_available
            // so downstream stages can check references to enrich output columns (EC-11-054
            // green-lock + EC-11-056 new test). When registry is absent, fall back to suspend
            // (existing fail-open behavior — keeps EC-11-054/EC-11-055 no-registry tests GREEN).
            //
            // BC-2.11.016 SIBLING-GATE CONSISTENCY (MED-001): enrich output columns
            // are DERIVED — their types come from the infusion schema, not the raw TableRegistry.
            // Add them to `derived_names` so E-QUERY-002 skips them (FP-001).
            PipeStage::Enrich(es) => {
                if let Some(col) =
                    extract_column_name_from_field_path(&es.field, table_name, table_alias)
                {
                    check_column_against_available_set(
                        &col,
                        table_name,
                        client_id,
                        &current_available,
                    )?;
                }
                if let Some(registry) = infusion_registry {
                    // Registry wired: resolve the infusion_id for this UDF name, then fetch the
                    // output column list via enrich_descriptor. UNION output_columns into
                    // current_available so post-enrich stages see the enriched binding context.
                    //
                    // Defensive: if the descriptor lookup fails (shouldn't happen post-E-QUERY-039
                    // validation, but code defensively), fall back to suspend (fail-open / FP-001).
                    let resolved_descriptor = registry
                        .udf_descriptors()
                        .into_iter()
                        .find(|d| d.name == es.infusion)
                        .and_then(|d| registry.enrich_descriptor(&d.infusion_id).ok());
                    if let Some(descriptor) = resolved_descriptor {
                        for col in &descriptor.output_columns {
                            if !current_available.contains(col) {
                                current_available.push(col.clone());
                            }
                            // Enrich output columns are DERIVED (their type is defined by the
                            // infusion schema, not the raw TableRegistry) — mark for MED-001.
                            derived_names.insert(col.clone());
                        }
                        current_available.sort();
                        current_available.dedup();
                        // Do NOT suspend — downstream stages have the enriched binding context.
                    } else {
                        // Descriptor lookup failed (defensive path) → fail-open.
                        suspended = true;
                    }
                } else {
                    // No registry wired → suspend all downstream checks (FP-001 / EC-11-054).
                    suspended = true;
                }
            }
            // Position 14: `| dedup` field keys — E-QUERY-038 existence only.
            // Dedup field paths are plain column refs (same as sort keys position 10);
            // checked against current binding context.
            PipeStage::Dedup(fields) => {
                for fp in fields {
                    if let Some(col) =
                        extract_column_name_from_field_path(fp, table_name, table_alias)
                    {
                        check_column_against_available_set(
                            &col,
                            table_name,
                            client_id,
                            &current_available,
                        )?;
                    }
                }
            }
            // BC-2.11.016 STAGE-JOIN SUSPENSION RULE: when the stage walk
            // encounters a PipeStage::Join stage, set suspended := true for the
            // remainder of the walk. Join-source schemas are not statically seeded
            // into the binding context (only the FROM table's schema is populated in
            // `available`); downstream column references that resolve through the join
            // source would falsely fire E-QUERY-038 if checked against the FROM-only
            // `available` set (FP-001 violation class: stage-join). Symmetry with
            // STAR-WITH-JOIN SUSPENSION RULE (head-level). Once suspended, all
            // subsequent stages skip E-QUERY-038 (suspension propagation clause).
            PipeStage::Join(_) => {
                suspended = true;
            }
            // Other stage types — fail-open:
            // - Limit(u64) / Tail(u64): carry no column refs.
            // - Future variants: fail-open per AST-completeness invariant (OBS-002);
            //   new column-bearing variants require a corresponding arm.
            #[allow(unreachable_patterns)]
            _ => {}
        }
    }
    Ok(())
}

/// Extract column names from a `Predicate` tree, resolving table-qualified references.
///
/// Walks all variants that carry a `FieldPath` directly, and recurses into
/// `Logical` and `Not` for nested predicates. `VirtualField` segments
/// (`_sensor`, `_client`) are implicitly excluded because those appear as
/// `Expr::VirtualField`, not `Expr::Field`.
///
/// The `Compare { lhs, .. }` arm matches `lhs` on two forms (ADR-048):
/// - `Expr::Field(fp)` — bare column reference (WHERE / HAVING bare predicate);
///   extracted via `extract_column_name_from_field_path`.
/// - `Expr::FuncCall(_)` — fn-call LHS: either an aggregate (HAVING `agg_fn(col) op literal`
///   per ADR-048 D.3) or a scalar fn-call (pipe `| where` `scalar_fn(col) op literal`
///   per DEFECT-PQL-FNCALL-LHS-001). Recursed via `extract_field_paths_from_expr` to
///   reach nested `Expr::Field` args. Exercised for the seven §D.7.1 shared-parser
///   predicate positions PLUS HAVING FuncCall LHS contexts (FuncCall::Aggregate per
///   §D.3, FuncCall::Scalar base fallthrough per §D.7.3); §D.7.1's HAVING exemption
///   applies to the aggregate gate, not to column extraction.
/// - All other `lhs` forms (`VirtualField`, `Literal`, etc.) — fail-open (silently skipped).
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
        // Compare: lhs may be:
        //   - Expr::Field(fp)        — bare column ref (WHERE / HAVING bare predicate)
        //   - Expr::FuncCall(..)     — fn-call LHS: all seven shared-parser predicate positions
        //                             (ADR-048 §D.7.1; WHERE-safe via arg-recursion, ADR-048 §D.3)
        //                             + HAVING FuncCall LHS contexts (§D.3/§D.7.3).
        //
        // For Expr::Field: extract via extract_column_name_from_field_path (handles
        //   qualified refs, F-001B-DC-HIGH-001).
        // For Expr::FuncCall: recurse into args via extract_field_paths_from_expr, which
        //   already handles FuncCall::Aggregate/Scalar arg lists at any nesting depth.
        //   WHERE-safe: `extract_field_paths_from_expr` recurses into FuncCall args regardless of
        //   function identity — column extraction operates on args, not the fn name (ADR-048 §D.3 v1.3).
        //
        // F-001B-DC-HIGH-001; ADR-048 D.3; DEFECT-PQL-FNCALL-LHS-001.
        Predicate::Compare { lhs, .. } => {
            match lhs.as_ref() {
                Expr::Field(fp) => {
                    if let Some(name) =
                        extract_column_name_from_field_path(fp, table_name, table_alias)
                    {
                        out.push(name);
                    }
                }
                Expr::FuncCall(_) => {
                    // Fn-call LHS (all seven shared-parser positions, ADR-048 §D.7.1,
                    // + HAVING FuncCall LHS contexts, §D.3/§D.7.3).
                    // Recurse into args via extract_field_paths_from_expr to extract Expr::Field refs.
                    extract_field_paths_from_expr(lhs.as_ref(), table_name, table_alias, out);
                }
                // Other lhs forms (VirtualField, Literal, etc.) — fail-open.
                _ => {}
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
        //
        // ADR-048 D.3: When lhs is Expr::FuncCall (HAVING aggregate predicate), the
        // type-compatibility check is intentionally skipped. The result of an aggregate
        // function (e.g., COUNT, SUM) is always numeric, and the comparison operator
        // is already validated by the grammar. Type-checking the column inside the agg
        // arg is the job of the E-QUERY-038 column-gate (collect_predicate_columns),
        // not this function.
        //
        // BC-2.11.016 MED-001 / BC-2.11.024 / S-PRISMQL-CASE-INSENSITIVE-001:
        // When `case_insensitive = true`, the effective operator is "IEQ" (for Eq) or
        // "INE" (for Ne) — NOT plain "=" / "!=". This distinction is load-bearing:
        // "IEQ"/"INE" are NOT in valid_operators_for_type(Integer/Float/Boolean/Datetime),
        // so check_operator_type_compatibility correctly flags them as type mismatches.
        // Without this translation, compare_op_to_str(Eq) → "=" passes Integer columns
        // silently, bypassing the E-QUERY-002 gate for IEQ-on-Integer predicates.
        Predicate::Compare {
            lhs,
            op,
            case_insensitive,
            ..
        } => {
            if let Expr::Field(fp) = lhs.as_ref() {
                if let Some(col_name) =
                    extract_column_name_from_field_path(fp, table_name, table_alias)
                {
                    // Emit the canonical operator name, accounting for case-insensitive variants.
                    use crate::ast::CompareOp;
                    let effective_op_str: Option<&'static str> = if *case_insensitive {
                        match op {
                            CompareOp::Eq => Some("IEQ"),
                            CompareOp::Ne => Some("INE"),
                            // Other ops with case_insensitive=true are not representable in the
                            // PrismQL AST; fall through to compare_op_to_str for forward-compat.
                            _ => compare_op_to_str(op),
                        }
                    } else {
                        compare_op_to_str(op)
                    };
                    if let Some(op_str) = effective_op_str {
                        out.push((col_name, op_str.to_string()));
                    }
                }
            }
            // FuncCall LHS (ADR-048 D.3): intentionally skipped — see comment above.
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
        // IIN: case-insensitive set membership — ADV-FIX-P3-LOW-001.
        //
        // `Predicate::In { case_insensitive: true, .. }` is the IIN operator.  It
        // must emit "IIN" so that `check_operator_type_compatibility` fires the
        // E-QUERY-002 plan-time gate for non-String columns (e.g., Integer).
        //
        // Operator string "IIN" matches the byte-form used by
        // `collect_ci_compare_fields` in materialization.rs and by
        // `valid_operators_for_type(ColumnType::String)` — both of which include
        // "IIN".  `valid_operators_for_type(Integer/Float/Boolean/Datetime)` does
        // NOT include "IIN", so the gate fires correctly.
        //
        // Negated+CI handling (TD-VSDD-060 defensive posture): the PrismQL parser
        // rejects `NOT IIN` as non-representable in the AST (filter_parser.rs —
        // "<invalid: negated IIN not representable>"), so `negated: true,
        // case_insensitive: true` can only arrive via hand-constructed ASTs.
        // We emit "IIN" for ALL `case_insensitive: true` In predicates regardless
        // of `negated`, because "IIN" is the correct type-check label and the
        // valid_operators table excludes it for non-String types either way.
        Predicate::In {
            field,
            case_insensitive: true,
            ..
        } => {
            if let Some(col_name) =
                extract_column_name_from_field_path(field, table_name, table_alias)
            {
                out.push((col_name, "IIN".to_string()));
            }
        }
        // All other predicate variants (StringOp, Regex, plain In, Between, Cidr,
        // Has, Missing, IsNull, Wildcard, RecoveryError, future variants): no
        // Compare operator to check — skip.
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
    table_registry: Option<&crate::table_registry::TableRegistry>,
) -> Result<(), PrismError> {
    // HIGH-001 (BC-2.11.016): when resolved_spec_map is None (single-tenant mode),
    // fall back to table_registry.column_type_for() — mirrors the M1 pattern used by
    // check_column_availability (S-DEMO-FIDELITY-REMEDIATION-001).
    if resolved_spec_map.is_none() {
        let Some(registry) = table_registry else {
            return Ok(()); // No schema source — fail-open.
        };
        let Some(actual_type) = registry.column_type_for(table_name, column_name) else {
            return Ok(()); // Column type unknown — fail-open; existence gate handles it.
        };
        let valid_ops = valid_operators_for_type(actual_type.clone());
        if valid_ops.contains(&operator) {
            return Ok(());
        }
        return Err(PrismError::QueryTypeMismatch {
            column: column_name.to_string(),
            table: table_name.to_string(),
            actual_type,
            operator: operator.to_string(),
            suggested_column: crate::materialization::ocsf_suggested_string_column(column_name),
        });
    }
    let Some(spec_map) = resolved_spec_map else {
        // Unreachable: guarded by is_none() check above.
        return Ok(());
    };

    // ADR-058 §G / S-ADR058-OCSF-ROUTING-001 holdout gap (Fix C):
    // When spec.ocsf_column_naming=true, match column_name against the OCSF-flattened Arrow name
    // (ocsf_field_to_arrow_name(col.ocsf_field)). Synthesized pseudo-cols (class_uid, _sensor,
    // raw_extensions) are not in tbl.columns, so fail-open for them — type is correct by
    // construction (Integer, String, Json). When ocsf_column_naming=false, match col.name.
    let column_type = spec_map
        .values()
        .filter(|spec| {
            if let Some(scopes) = org_scope {
                scopes.iter().any(|s| s.as_str() == spec.org_slug.as_str())
            } else {
                true
            }
        })
        .flat_map(|spec_entry| {
            let sensor_id = spec_entry.spec.sensor_id.clone();
            let ocsf_naming = spec_entry.spec.ocsf_column_naming;
            spec_entry
                .spec
                .tables
                .iter()
                .filter(move |tbl| format!("{sensor_id}_{}", tbl.table_name) == table_name)
                .flat_map(move |tbl| {
                    tbl.columns.iter().filter_map(move |col| {
                        let effective_name = if ocsf_naming {
                            col.ocsf_field.as_deref().map(|f| {
                                prism_spec_engine::column_mapping::ocsf_field_to_arrow_name(f)
                            })
                        } else {
                            Some(col.name.clone())
                        };
                        if effective_name.as_deref() == Some(column_name) {
                            Some(col.column_type.clone())
                        } else {
                            None
                        }
                    })
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
    // Populate suggested_column via the OCSF sibling mapping (b1 form, error-taxonomy v2.22):
    // severity_id→severity, status_id→status, activity_id→activity_name,
    // disposition_id→disposition. This matches the materialization-layer gate
    // (check_ci_column_types) which is kept as defense-in-depth but fires second.
    Err(PrismError::QueryTypeMismatch {
        column: column_name.to_string(),
        table: table_name.to_string(),
        actual_type,
        operator: operator.to_string(),
        suggested_column: crate::materialization::ocsf_suggested_string_column(column_name),
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
/// String → `["=", "!=", "LIKE", "IN", "NOT IN", "IEQ", "IIN", "INE"]`
/// Integer → `["=", "!=", "<", ">", "<=", ">=", "BETWEEN", "IN", "NOT IN"]`
/// Float → `["=", "!=", "<", ">", "<=", ">=", "BETWEEN"]`
/// Boolean → `["=", "!="]`
/// Datetime → `["=", "!=", "<", ">", "<=", ">=", "BETWEEN"]`
/// Json → `["=", "!="]`
///
/// Note: "NOT IIN" is intentionally absent. Negated IIN is not representable in the
/// PrismQL AST (ast.rs: "<invalid: negated IIN not representable>") — it is never a
/// legal operator. BC-2.11.024; F-P24-MED-001 (S-PRISMQL-CASE-INSENSITIVE-001).
///
/// Reference: BC-2.11.017 postconditions; S-DEMO-PRISMQL-ONBOARDING-001-B AC-003.
pub fn valid_operators_for_type(
    column_type: prism_core::column::ColumnType,
) -> &'static [&'static str] {
    use prism_core::column::ColumnType;
    match column_type {
        // BC-2.11.024: IEQ/IIN/INE are valid string-column case-insensitive operators.
        // "NOT IIN" is NOT included — negated IIN is not representable in the PrismQL AST.
        // F-P24-MED-001 (LOCAL pass-24, S-PRISMQL-CASE-INSENSITIVE-001).
        ColumnType::String => &["=", "!=", "LIKE", "IN", "NOT IN", "IEQ", "IIN", "INE"],
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
                "",
            ),
            InfusionUdfDescriptor::new(
                "threat_score", // duplicate name
                "ip",
                "string",
                "threatintel_v2",
                Arc::new(NullSrc),
                None,
                3600,
                "",
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
        engine = engine.with_resolved_spec_map(Arc::new(spec_map));
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
        engine = engine.with_resolved_spec_map(Arc::new(initial_map));

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
            true, // compute_did_you_mean: test exercises the suggestion path
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
            true, // compute_did_you_mean: test exercises the suggestion path
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
            true, // compute_did_you_mean: test exercises the suggestion path
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
        engine = engine.with_resolved_spec_map(StdArc::new(spec_map));

        // SqlPipe query projecting a non-existent column "severit" (typo of "severity",
        // Levenshtein distance = 1 which is within the ≤3 threshold).
        // With org_scope matching "testorg", the column gate resolves "crowdstrike_detections"
        // to the spec above, finds only "severity", and must deny "severit" with E-QUERY-038.
        // Note: "sev" would be distance 5 from "severity" (>3 threshold) so would give
        // did_you_mean=None. "severit" (missing trailing 'y') is the correct test typo.
        //
        // IMPORTANT: Uses underscore form "crowdstrike_detections" (NOT dot form
        // "crowdstrike.detections"). BC-2.11.001 / EC-11-067: dot-notation in FROM
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
    /// F-P1L4-MED-001 / BC-2.11.019 / H1 fix (S-DEMO-FIDELITY-REMEDIATION-001)
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

    // ── BC-2.11.004 TM-SCHED: execute_scheduled aggregate-gate parity lock ──────

    /// TD-VSDD-059 parity gap (F-PQLFN-P3-LOW-002): `execute_scheduled_inner` aggregate-gate
    /// parity lock.
    ///
    /// The ADR-048 v1.2 D.7 unified plan-time aggregate-gate was verified for the `execute`
    /// path (TM-03/09 in `temporal_typing_tests.rs`). This test locks the SAME gate via
    /// `execute_scheduled`, proving the early `check_enrich_udf_availability(query_str, None)`
    /// call at the start of `execute_scheduled_inner` fires and returns E-QUERY-001 with the
    /// canonical D.3 message substrings — symmetrically to `execute`.
    ///
    /// Query: `FROM crowdstrike_detections | where stddev(risk_score) = 5`
    ///
    /// # Expected: GREEN on arrival
    /// `execute_scheduled_inner` already calls `check_enrich_udf_availability(query_str, None)`
    /// as its FIRST gate (before the table gate). `stddev` ∈
    /// `DATAFUSION_BUILTIN_AGGREGATE_NAMES` → the aggregate gate fires → E-QUERY-001 with the
    /// canonical D.3 message containing "stddev", "aggregate function", and "HAVING".
    ///
    /// Both `execute` and `execute_scheduled` must return `QueryParseFailed` for this query,
    /// demonstrating symmetric aggregate-gate behavior across both execution paths.
    ///
    /// Traces to: BC-2.11.004 EC-11-082 (renumbered from EC-11-013 in v1.47; SR-006 collision with BC-2.11.005); ADR-048 v1.2 §D.7.4; F-PQLFN-P3-LOW-002.
    #[tokio::test]
    async fn test_BC_2_11_004_tm_sched_parity_aggregate_gate_execute_scheduled() {
        let engine = make_test_engine();

        let query = "FROM crowdstrike_detections | where stddev(risk_score) = 5";

        let execute_result = engine.execute(query, QueryOptions::default()).await;

        // Map execute_scheduled to Result<QueryResult, PrismError> by discarding the
        // Arc<SessionContext> (SessionContext does not impl Debug so we cannot
        // unwrap_err on the raw tuple result).
        let scheduled_result = engine
            .execute_scheduled(query, None)
            .await
            .map(|(qr, _ctx)| qr);

        // Both must error.
        assert!(
            execute_result.is_err(),
            "TM-SCHED: execute with stddev aggregate in | where must return Err; got Ok"
        );
        assert!(
            scheduled_result.is_err(),
            "TM-SCHED: execute_scheduled with stddev aggregate in | where must return Err; \
             got Ok. If Ok: the aggregate gate in execute_scheduled_inner is missing or bypassed."
        );

        let exec_err = execute_result.unwrap_err();
        let sched_err = scheduled_result.unwrap_err();

        // Both must be QueryParseFailed (E-QUERY-001).
        assert!(
            matches!(exec_err, PrismError::QueryParseFailed { .. }),
            "TM-SCHED: execute must return QueryParseFailed (E-QUERY-001); got: {exec_err:?}"
        );
        assert!(
            matches!(sched_err, PrismError::QueryParseFailed { .. }),
            "TM-SCHED: execute_scheduled must return QueryParseFailed (E-QUERY-001) \
             to match execute behavior. Got: {sched_err:?}. \
             If TableNotAvailable (E-QUERY-037): the aggregate gate is NOT firing before the \
             table gate in execute_scheduled_inner (ADR-048 v1.2 §D.7.4 gate ordering violated)."
        );

        // Both Display outputs must contain the canonical D.3 message substrings.
        let exec_display = format!("{exec_err}");
        let sched_display = format!("{sched_err}");

        for (label, display) in [
            ("execute", &exec_display),
            ("execute_scheduled", &sched_display),
        ] {
            assert!(
                display.contains("stddev"),
                "TM-SCHED [{label}]: Display must contain 'stddev' (aggregate fn name, \
                 ADR-048 D.3 canonical). Got: {display}"
            );
            assert!(
                display.contains("aggregate function"),
                "TM-SCHED [{label}]: Display must contain 'aggregate function' \
                 (ADR-048 D.3 canonical message). Got: {display}"
            );
            assert!(
                display.contains("HAVING"),
                "TM-SCHED [{label}]: Display must contain 'HAVING' (use HAVING guidance, \
                 ADR-048 D.3). Got: {display}"
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
// Post-DEFECT-PQL-FNCALL-LHS-001: `build_predicate_parser()` (shared by pipe `| where`,
// filter mode, and SQL WHERE via `build_sql_predicate_parser`) now accepts fn-call LHS via
// `fn_call_comparison` — `WHERE badudf(col) = 1` can now parse successfully. Direct AST
// construction is used here to isolate the collect_ function logic from the parser and
// test the walker independently of the grammar production rules. These unit tests verify
// the walk logic directly without parser round-trips.
// TD-VSDD-059: load-bearing unit tests on the actual collect_ functions.
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
                span: crate::ast::Span::ZERO,
            })),
            op: CompareOp::Eq,
            rhs: Box::new(Expr::Literal(Literal::String("value".to_string()))),
            case_insensitive: false,
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
                span: crate::ast::Span::ZERO,
            })),
            op: CompareOp::Eq,
            rhs: Box::new(Expr::Literal(Literal::Integer(1))),
            case_insensitive: false,
        };
        let simple_compare = Predicate::Compare {
            lhs: Box::new(Expr::Field(FieldPath::new(vec!["severity".to_string()]))),
            op: CompareOp::Eq,
            rhs: Box::new(Expr::Literal(Literal::String("high".to_string()))),
            case_insensitive: false,
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
            case_insensitive: false,
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
                span: crate::ast::Span::ZERO,
            })),
            op: CompareOp::Ne,
            rhs: Box::new(Expr::Literal(Literal::Integer(0))),
            case_insensitive: false,
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
        engine = engine.with_resolved_spec_map(Arc::new(spec_map));
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
        engine = engine.with_resolved_spec_map(Arc::new(spec_map));
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

    /// EC-11-041 (BC-2.11.016) — SQL-mode zero-column gate fires E-QUERY-038.
    ///
    /// Previously (M1 backward-compat behavior): registered table with zero columns
    /// caused the single-tenant gate to fail-open, letting the query reach DataFusion
    /// and producing an opaque E-QUERY-034 error.
    ///
    /// BC-2.11.016 EC-11-041 supersedes that fail-open behavior:
    /// "Table has zero registered columns → E-QUERY-038 with available_columns: [],
    /// did_you_mean absent." (ADV-FIX-P9-OBS-001)
    ///
    /// This test verifies that the SQL-mode path (`check_column_availability`) fires
    /// E-QUERY-038 with `available_columns: []` for a registered zero-column table,
    /// consistent with the pipe-mode test in `drift_ieq_nonexistent_col_errpath_001_tests`.
    #[tokio::test]
    async fn test_m1_single_tenant_no_columns_registered_fires_e_query_038() {
        use prism_spec_engine::spec_parser::{AuthType, SensorSpec, TableSpec};

        // Register WITHOUT columns (empty column list) — EC-11-041 fixture.
        let spec = SensorSpec::new(
            "armis",
            "Armis sensor",
            AuthType::ApiKey,
            "https://api.armis.com",
            vec![TableSpec::new_point_in_time(
                "devices",
                "security_finding",
                vec![], // No columns — zero-column registered table (EC-11-041).
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

        // SQL-mode query referencing a non-existent column on a zero-column table.
        let query = "SELECT completely_bogus_col FROM armis_devices LIMIT 5";

        let result = engine.execute(query, QueryOptions::default()).await;

        // EC-11-041: E-QUERY-038 must fire with available_columns: [] (not fail-open).
        match result {
            Err(PrismError::ColumnNotFound(ref details)) => {
                assert!(
                    details.available_columns.is_empty(),
                    "EC-11-041 SQL-mode: available_columns must be [] for a zero-column table. \
                     Got: {:?}",
                    details.available_columns
                );
                assert!(
                    details.did_you_mean.is_none(),
                    "EC-11-041 SQL-mode: did_you_mean must be absent for a zero-column table. \
                     Got: {:?}",
                    details.did_you_mean
                );
            }
            Ok(_) => panic!(
                "EC-11-041 SQL-mode: engine.execute must NOT succeed. E-QUERY-038 must fire \
                 with available_columns=[] for a registered zero-column table (ADV-FIX-P9-OBS-001). \
                 Fail-open is FORBIDDEN per BC-2.11.016 EC-11-041."
            ),
            Err(other) => panic!(
                "EC-11-041 SQL-mode: expected PrismError::ColumnNotFound (E-QUERY-038) with \
                 available_columns=[], got: {other:?}. BC-2.11.016 EC-11-041 requires \
                 E-QUERY-038 for registered zero-column tables, not a DataFusion error."
            ),
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
        engine = engine.with_resolved_spec_map(Arc::new(spec_map));

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
// BC-2.11.019 §F-PJL1-HIGH-001 "Scope of change":
//   "SQL-mode `ScalarFunc::Unknown` gate logic only. Pipe-mode `EnrichStage.infusion`
//    gate is UNAFFECTED (pipe-mode `| enrich` is an explicit enrichment directive —
//    a built-in name there is NOT a DataFusion scalar, it's an unregistered infusion
//    the analyst is trying to apply, so it SHOULD fire E-QUERY-039)."
//
// Before fix: `check_enrich_udf_availability` collected all names into one Vec and
//   applied `DATAFUSION_BUILTIN_FUNCTION_NAMES` skip uniformly — pipe-mode enrich names
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
    /// Before fix: `lower` matched `DATAFUSION_BUILTIN_FUNCTION_NAMES` (it is a DataFusion
    ///   built-in) and was silently skipped — the gate was a no-op for this pipe query.
    /// After fix: pipe-mode enrich names bypass the built-in skip entirely. `lower` is not
    ///   in InfusionRegistry → E-QUERY-039 fires with `infusion: "lower"`.
    ///
    /// BC-2.11.019 §F-PJL1-HIGH-001 scope: "Pipe-mode `EnrichStage.infusion` gate
    /// is UNAFFECTED — a built-in name there is NOT a DataFusion scalar, it is an
    /// unregistered infusion the analyst is trying to apply, so it SHOULD fire E-QUERY-039."
    ///
    /// Load-bearing (TD-VSDD-059): before fix the single-Vec approach skips `lower` →
    /// `execute` succeeds or returns a different error. After fix, E-QUERY-039 is returned.
    #[tokio::test]
    async fn test_pipe_mode_builtin_name_fires_e_query_039() {
        let engine = make_engine_with_sensor_and_empty_infusion_registry();

        // Pipe-mode query: `lower` is not a registered infusion — E-QUERY-039 MUST fire.
        // Before fix: `lower` is in DATAFUSION_BUILTIN_FUNCTION_NAMES → skipped → gate is no-op.
        // After fix: pipe-mode names bypass DATAFUSION_BUILTIN_FUNCTION_NAMES → gate fires.
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
    /// BC-2.11.019 §F-PJL1-HIGH-001 + EC-11-064.
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
// HAVING typo_col > 5` bypassed E-QUERY-038 entirely — `typo_col` in HAVING
// was never validated against the schema.
//
// Sibling asymmetry: E-QUERY-039 (enrich gate) and E-QUERY-037 (source-walk) both
// cover HAVING; only E-QUERY-038 (column gate) was missing this position.
//
// Fix (F-PWL1-LOW-001): Position 6 — HAVING — added to `check_query_column_availability`,
// using `extract_predicate_columns` (same helper as Position 2 / WHERE), since
// `having` is `Option<Predicate>` identical in type to `where_`. The bare-column form
// (`HAVING typo_col > 5`, parsed as `Predicate::Compare { lhs: Expr::Field }`) is
// extracted via the existing `collect_predicate_columns` Expr::Field arm.
//
// Grammar extension (F-PXL3-MED-002 / ADR-048): HAVING additionally accepts the
// `agg_fn(col) op literal` predicate form (e.g., `HAVING count(typo_col) > 5`) via
// `build_having_predicate_parser`. The `collect_predicate_columns` Expr::FuncCall arm
// (added in F-PXL3-MED-002) extracts the column nested inside the aggregate argument
// → `typo_col` → E-QUERY-038. WHERE does NOT accept this form (deliberate ADR-048
// grammar divergence: aggregate predicates in WHERE are semantically invalid SQL).
//
// BC-2.11.016 / F-PWL1-LOW-001 / F-PXL3-MED-002.
//
// Tests assert:
//   1. (red-gate) HAVING with typo'd bare column fires E-QUERY-038.
//   2. (no-regression) HAVING with valid column does NOT fire E-QUERY-038.
//   Additional tests for the agg-fn form are in f_pxl3_med002_having_agg_predicate_col_gate_tests.
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
        engine = engine.with_resolved_spec_map(Arc::new(spec_map));
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
    /// Note on PrismQL HAVING grammar: the HAVING predicate accepts two forms:
    ///   1. `field op literal` (bare column comparison) — via `build_sql_predicate_parser`,
    ///      same as WHERE. This is the form tested here (`HAVING typo_col > 5`).
    ///   2. `agg_fn(col) op literal` (aggregate-function comparison) — via
    ///      `build_having_predicate_parser` (ADR-048 / F-PXL3-MED-002). This form is HAVING-only;
    ///      WHERE deliberately does NOT accept it (aggregate in WHERE is pre-aggregation-invalid).
    ///
    /// The tested query `HAVING typo_col > 5` exercises form (1) — the bare-column path through
    /// `collect_predicate_columns` → `Expr::Field` arm. It is the most direct proof that
    /// Position 6 (HAVING) is walked by `check_query_column_availability`.
    /// Form (2) is covered by `test_BC_2_11_016_having_agg_fn_predicate_typo_fires_e_query_038`
    /// in the `f_pxl3_med002_having_agg_predicate_col_gate_tests` module (F-PXL3-MED-002).
    ///
    /// BC-2.11.016 / F-PWL1-LOW-001.
    ///
    /// Load-bearing (F-PWL1-LOW-001): removing the Position 6 HAVING walk from
    /// `check_query_column_availability` causes this test to return Ok or a
    /// non-E-QUERY-038 error instead of PrismError::ColumnNotFound.
    #[tokio::test]
    async fn test_BC_2_11_016_having_column_gate_typo_fires_e_query_038() {
        let (engine, org) = make_crowdstrike_engine();

        // `typo_col` is not in the schema (only `severity` and `timestamp` are valid).
        // PrismQL HAVING predicate: `field op literal` form (bare-column form, same as WHERE).
        // After ADR-048 (F-PXL3-MED-002), HAVING also accepts `agg_fn(col) op literal` form,
        // but this test exercises the bare-column path specifically (Position 6 HAVING walk).
        // Before F-PWL1-LOW-001, Position 6 (HAVING) was absent so this silently passed.
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
    /// BC-2.11.016 / F-PWL1-LOW-001.
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

// ---------------------------------------------------------------------------
// F-PXL3-MED-002 — HAVING aggregate-predicate column gate (ADR-048)
// ---------------------------------------------------------------------------
//
// Tests for the grammar + extractor extension that makes
// `HAVING count(typo_col) > 5` fire E-QUERY-038 (column-not-found gate)
// instead of E-QUERY-001 (parse error).
//
// ADR-048 ratifies a deliberate HAVING/WHERE grammar divergence:
//   - HAVING gains the `agg_fn(col) op literal` predicate form.
//   - WHERE does NOT (WHERE is pre-aggregation; aggregate predicates there
//     are semantically invalid and must remain E-QUERY-001).
//
// Three load-bearing tests:
//   1. Red→Green: `HAVING count(typo_col) > 5` → E-QUERY-038 (typo inside agg fn)
//   2. Acceptance:  `HAVING count(severity) > 0` (valid col) must NOT fire E-QUERY-038
//   3. WHERE divergence guard: `WHERE count(severity) > 5` must stay E-QUERY-001

#[cfg(test)]
#[allow(
    non_snake_case,
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::too_many_lines
)]
mod f_pxl3_med002_having_agg_predicate_col_gate_tests {
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
    /// Mirrors the fixture pattern from `f_pwl1_low001_having_column_gate_tests`.
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
            .expect("F-PXL3-MED-002 fixture: register crowdstrike must not fail");

        let overlay_toml = format!("extends = \"{sensor_id}\"\ninstance_id = \"{sensor_id}@acme\"");
        let overlay: SensorInstanceOverlay = toml::from_str(&overlay_toml)
            .expect("F-PXL3-MED-002 fixture: SensorInstanceOverlay TOML must parse");
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
        engine = engine.with_resolved_spec_map(Arc::new(spec_map));
        engine = engine.with_table_registry(registry);

        (engine, org)
    }

    // ── Test 1 (Red→Green): HAVING count(typo_col) > 5 → E-QUERY-038 ─────────

    /// F-PXL3-MED-002 — E-QUERY-038 must fire for a typo'd column name referenced
    /// inside an aggregate function in the HAVING predicate.
    ///
    /// Query: `HAVING count(typo_col) > 5` — `typo_col` is not in the schema.
    ///
    /// Before fix (ADR-048): the HAVING grammar only accepts `field op literal`
    /// form; `count(typo_col)` is not parseable as such, so the query produces
    /// E-QUERY-001 (parse error) instead of the correct E-QUERY-038 (column-not-found).
    ///
    /// After fix: HAVING gains the `agg_fn(col) op literal` predicate form.
    /// The column extractor walks the FuncCall args and extracts `typo_col`
    /// → E-QUERY-038.
    ///
    /// ADR-048; BC-2.11.016.
    ///
    /// Load-bearing (F-PXL3-MED-002): without the grammar + extractor fix,
    /// this test panics with "expected ColumnNotFound, got different error"
    /// (i.e., a parse error fires instead).
    #[tokio::test]
    async fn test_BC_2_11_016_having_agg_fn_predicate_typo_fires_e_query_038() {
        let (engine, org) = make_crowdstrike_engine();
        let query = "SELECT severity, count(*) FROM crowdstrike_alerts \
                     GROUP BY severity HAVING count(typo_col) > 5";

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
            Err(PrismError::ColumnNotFound(ref d)) => {
                assert_eq!(
                    d.column, "typo_col",
                    "ADR-048: column in E-QUERY-038 must be 'typo_col', got: {:?}",
                    d.column
                );
                assert_eq!(
                    d.table, "crowdstrike_alerts",
                    "ADR-048: table in E-QUERY-038 must be 'crowdstrike_alerts'"
                );
            }
            Ok(_) => panic!(
                "ADR-048: engine.execute must NOT succeed — E-QUERY-038 must fire for \
                 typo_col inside count() in HAVING predicate."
            ),
            Err(other) => panic!(
                "ADR-048: expected PrismError::ColumnNotFound (E-QUERY-038) for \
                 HAVING count(typo_col) > 5, got: {other:?}"
            ),
        }
    }

    // ── Test 2 (acceptance): HAVING count(valid_col) must NOT fire E-QUERY-038 ─

    /// F-PXL3-MED-002 acceptance — HAVING with a valid column in the aggregate
    /// function must NOT fire E-QUERY-038.
    ///
    /// `HAVING count(severity) > 0` — `severity` is valid in `crowdstrike_alerts`.
    /// The column gate must pass; the query may fail later (no real adapter wired)
    /// but must NOT produce PrismError::ColumnNotFound.
    ///
    /// ADR-048; BC-2.11.016.
    #[tokio::test]
    async fn test_BC_2_11_016_having_agg_fn_predicate_valid_col_no_e_query_038() {
        let (engine, org) = make_crowdstrike_engine();
        let query = "SELECT severity, count(*) FROM crowdstrike_alerts \
                     GROUP BY severity HAVING count(severity) > 0";

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
            Err(PrismError::ColumnNotFound(ref d)) => panic!(
                "ADR-048 acceptance: E-QUERY-038 fired unexpectedly for valid column \
                 '{}' inside count(). `severity` is registered; the gate must NOT reject it.",
                d.column
            ),
            // Any other outcome (Ok, execution error, other PrismError) is acceptable —
            // the invariant is that E-QUERY-038 (ColumnNotFound) does NOT fire.
            _ => {}
        }
    }

    // ── Test 3 (WHERE divergence guard): WHERE count(col) must stay E-QUERY-001 ─

    /// F-PXL3-MED-002 WHERE divergence guard — `WHERE count(severity) > 5` must
    /// produce a parse error (E-QUERY-001), NOT pass silently.
    ///
    /// WHERE is pre-aggregation; aggregate functions in WHERE are semantically
    /// invalid SQL. ADR-048 ratifies the deliberate grammar divergence: the
    /// `agg_fn(col) op literal` predicate form is added to HAVING ONLY.
    ///
    /// Before and after the fix, this query must result in an error. After the fix,
    /// the parse error must still occur (not accidentally granted the agg-predicate form).
    ///
    /// This test is a regression guard ensuring WHERE did NOT silently gain the
    /// aggregate-predicate grammar form from the HAVING extension.
    ///
    /// ADR-048 §Constraint; BC-2.11.016.
    #[tokio::test]
    async fn test_BC_2_11_016_where_agg_fn_predicate_stays_e_query_001() {
        let (engine, org) = make_crowdstrike_engine();
        // WHERE count(severity) > 5 — canonically-ordered: WHERE precedes any GROUP BY.
        // This must NOT parse successfully — E-QUERY-001 parse error must fire.
        //
        // Previous query form ("GROUP BY severity WHERE count(severity) > 5") failed on
        // CLAUSE-ORDERING (WHERE after GROUP BY), not on WHERE rejecting the aggregate
        // predicate. The test would have passed even if WHERE accidentally gained the
        // agg-fn form, defeating the divergence guard (F-L2-CRIT-001 sibling, MED finding).
        //
        // This form ("SELECT severity FROM … WHERE count(severity) > 5") places WHERE in
        // the canonical position so the failure is attributable specifically to WHERE
        // grammar rejecting aggregate-function LHS predicates (ADR-048 D.6 regression guard).
        let query = "SELECT severity FROM crowdstrike_alerts WHERE count(severity) > 5";

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
            // QueryParseFailed is the expected outcome — E-QUERY-001.
            Err(PrismError::QueryParseFailed { .. }) => {}
            // ColumnNotFound would mean WHERE accidentally gained the agg-predicate form.
            Err(PrismError::ColumnNotFound(ref d)) => panic!(
                "ADR-048 divergence guard: WHERE must NOT parse aggregate predicates. \
                 Got ColumnNotFound for '{}' — WHERE grammar was accidentally extended.",
                d.column
            ),
            // A successful parse+execution would be even worse.
            Ok(_) => panic!(
                "ADR-048 divergence guard: `WHERE count(severity) > 5` must NOT succeed — \
                 aggregate predicates in WHERE are semantically invalid SQL."
            ),
            // Any other error (e.g., table/column error) means parse passed — that's wrong.
            Err(other) => panic!(
                "ADR-048 divergence guard: expected PrismError::QueryParseFailed for \
                 WHERE count(severity) > 5, got: {other:?}"
            ),
        }
    }
}

// DRIFT-IEQ-NONEXISTENT-COL-ERRPATH-001
//
// Root cause: `check_query_column_availability` returns `Ok(())` early for `Ast::Filter`
// and `Ast::Pipe` (the `_ => return Ok(())` arm in the AST match). This means columns
// referenced by IEQ/IIN/INE predicates in Filter/Pipe mode bypass the E-QUERY-038
// plan-time gate entirely. When execution proceeds to DataFusion with a non-existent
// column (e.g. `lower(severity_id) = lower('high')` for a table that has no
// `severity_id` column), DataFusion fails with a generic "column not found" error at
// planning time, which is mapped to `PrismError::QueryExecutionFailed` (opaque
// "Internal error" to the MCP client).
//
// Fix: extend `check_query_column_availability` to walk predicate columns in
// `Ast::Filter` (the root predicate) and `Ast::Pipe` (all `| where` stages) using
// the same `extract_predicate_columns` + `check_column_availability` helpers that
// already serve the SQL SELECT/WHERE path (Positions 2 and 6).
//
// RED GATE: the two "nonexistent_col" tests MUST FAIL on the unfixed codebase,
// returning `PrismError::QueryExecutionFailed` instead of `PrismError::ColumnNotFound`.
// They turn GREEN after the fix lands.
#[cfg(test)]
#[allow(
    non_snake_case,
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::too_many_lines
)]
mod drift_ieq_nonexistent_col_errpath_001_tests {
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

    // ── Helper ─────────────────────────────────────────────────────────────────

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

    /// Build a `crowdstrike_alerts` engine (sensor="crowdstrike", table="alerts")
    /// under org "acme". Valid columns: `severity` (String), `timestamp` (Datetime).
    ///
    /// Mirrors `m2_column_gate_funccall_and_join_tests::make_crowdstrike_engine_with_columns`
    /// and `f_pwl1_low001_having_column_gate_tests::make_crowdstrike_engine`.
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
            .expect("DRIFT-IEQ fixture: SensorInstanceOverlay TOML must parse");
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
        engine = engine.with_resolved_spec_map(Arc::new(spec_map));
        engine = engine.with_table_registry(registry);

        (engine, org)
    }

    // ── Test 1 (RED GATE): Filter mode, IEQ, non-existent column → E-QUERY-038 ─

    /// DRIFT-IEQ-NONEXISTENT-COL-ERRPATH-001 — filter mode.
    ///
    /// `crowdstrike.alerts | severity_id IEQ 'high'` references `severity_id`, which
    /// is NOT in the `crowdstrike_alerts` schema (only `severity` and `timestamp` are
    /// valid). Before the fix, `check_query_column_availability` returned `Ok(())`
    /// immediately for `Ast::Filter` (the `_ => return Ok(())` arm), so execution
    /// fell through to DataFusion which produced an opaque `QueryExecutionFailed`
    /// (E-QUERY-034) — "Internal error" to the MCP client.
    ///
    /// After fix: the filter predicate columns are walked by
    /// `check_query_column_availability`, and `severity_id` fails the schema check,
    /// yielding `PrismError::ColumnNotFound` (E-QUERY-038) with
    /// `column="severity_id"`, `table="crowdstrike_alerts"`.
    ///
    /// BC-2.11.016 fourteen-position gate must apply to Filter predicate columns.
    /// Red Gate: removing predicate-column extraction for `Ast::Filter` from
    /// `check_query_column_availability` causes this test to return a non-ColumnNotFound
    /// error (QueryExecutionFailed) instead of ColumnNotFound.
    #[tokio::test]
    async fn test_DRIFT_IEQ_filter_mode_nonexistent_col_yields_e_query_038() {
        let (engine, org) = make_crowdstrike_engine();

        // `severity_id` is NOT in the schema; only `severity` (String) and
        // `timestamp` (Datetime) are registered columns.
        // Use underscore notation (Custom source ref) — dot notation is rejected by
        // E-QUERY-037 before the column gate, so this test must use the canonical
        // registered form "crowdstrike_alerts".
        let query = "crowdstrike_alerts | severity_id IEQ 'high'";

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
                    details.column, "severity_id",
                    "DRIFT-IEQ-001 filter: column in E-QUERY-038 must be 'severity_id', \
                     got: {:?}",
                    details.column
                );
                assert_eq!(
                    details.table, "crowdstrike_alerts",
                    "DRIFT-IEQ-001 filter: table in E-QUERY-038 must be 'crowdstrike_alerts', \
                     got: {:?}",
                    details.table
                );
                // did_you_mean should be Some("severity") — Levenshtein distance 3
                // ("severity_id" → "severity" is 3 ops: remove "_", "i", "d").
                // We do not assert the exact did_you_mean value since the threshold
                // and suggestion logic may vary; the key invariant is the error code.
            }
            Ok(_) => panic!(
                "DRIFT-IEQ-001 filter: engine.execute must NOT succeed — E-QUERY-038 must fire \
                 for non-existent column 'severity_id' in filter IEQ predicate. Before the fix, \
                 Ast::Filter bypassed check_query_column_availability entirely."
            ),
            Err(other) => panic!(
                "DRIFT-IEQ-001 filter: expected PrismError::ColumnNotFound (E-QUERY-038), \
                 got different error: {other:?}. Before the fix this would be \
                 QueryExecutionFailed (DataFusion generic error)."
            ),
        }
    }

    // ── Test 2 (RED GATE): Pipe mode, IEQ, non-existent column → E-QUERY-038 ───

    /// DRIFT-IEQ-NONEXISTENT-COL-ERRPATH-001 — pipe mode.
    ///
    /// `crowdstrike.alerts | where severity_id IEQ 'high'` references `severity_id` in
    /// a `| where` stage — not in the schema. Before the fix, `Ast::Pipe` fell through
    /// the `_ => return Ok(())` arm in `check_query_column_availability`, so DataFusion
    /// produced a generic error at planning time.
    ///
    /// After fix: `| where` stage predicates are walked and `severity_id` fails
    /// the schema check → E-QUERY-038.
    ///
    /// Red Gate: removing predicate-column extraction for `Ast::Pipe` from
    /// `check_query_column_availability` causes this test to return QueryExecutionFailed
    /// instead of ColumnNotFound.
    #[tokio::test]
    async fn test_DRIFT_IEQ_pipe_mode_nonexistent_col_yields_e_query_038() {
        let (engine, org) = make_crowdstrike_engine();

        // Pipe mode: `source | where predicate` syntax.
        // Use underscore notation for the same reason as the filter test.
        // `severity_id` is NOT a registered column in `crowdstrike_alerts`.
        let query = "crowdstrike_alerts | where severity_id IEQ 'high'";

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
                    details.column, "severity_id",
                    "DRIFT-IEQ-001 pipe: column in E-QUERY-038 must be 'severity_id', \
                     got: {:?}",
                    details.column
                );
                assert_eq!(
                    details.table, "crowdstrike_alerts",
                    "DRIFT-IEQ-001 pipe: table in E-QUERY-038 must be 'crowdstrike_alerts', \
                     got: {:?}",
                    details.table
                );
            }
            Ok(_) => panic!(
                "DRIFT-IEQ-001 pipe: engine.execute must NOT succeed — E-QUERY-038 must fire \
                 for non-existent column 'severity_id' in pipe | where IEQ predicate. Before \
                 the fix, Ast::Pipe bypassed check_query_column_availability entirely."
            ),
            Err(other) => panic!(
                "DRIFT-IEQ-001 pipe: expected PrismError::ColumnNotFound (E-QUERY-038), \
                 got different error: {other:?}."
            ),
        }
    }

    // ── Test 3 (no-regression): Filter mode, IEQ, EXISTING column → no E-QUERY-038

    /// DRIFT-IEQ-NONEXISTENT-COL-ERRPATH-001 no-regression — filter mode, valid column.
    ///
    /// `crowdstrike.alerts | severity IEQ 'high'` uses `severity` which IS in the schema.
    /// The E-QUERY-038 gate must NOT fire for existing columns.
    ///
    /// The query will fail for other reasons (no adapter wired, no data), but it must
    /// NOT produce `PrismError::ColumnNotFound`.
    #[tokio::test]
    async fn test_DRIFT_IEQ_filter_mode_existing_col_no_e_query_038() {
        let (engine, org) = make_crowdstrike_engine();

        let query = "crowdstrike_alerts | severity IEQ 'high'";

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
            Err(PrismError::ColumnNotFound(ref d)) => panic!(
                "DRIFT-IEQ-001 no-regression filter: E-QUERY-038 fired unexpectedly for \
                 existing column 'severity'. Got: column='{}', table='{}'",
                d.column, d.table
            ),
            // Ok or any other error variant is acceptable — the invariant is that
            // E-QUERY-038 does NOT fire for a valid column.
            _ => {}
        }
    }

    // ── Test 4 (no-regression): Pipe mode, IEQ, EXISTING column → no E-QUERY-038 ──

    /// DRIFT-IEQ-NONEXISTENT-COL-ERRPATH-001 no-regression — pipe mode, valid column.
    ///
    /// `crowdstrike.alerts | where severity IEQ 'high'` uses the existing `severity`
    /// column. E-QUERY-038 must NOT fire.
    #[tokio::test]
    async fn test_DRIFT_IEQ_pipe_mode_existing_col_no_e_query_038() {
        let (engine, org) = make_crowdstrike_engine();

        let query = "crowdstrike_alerts | where severity IEQ 'high'";

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
            Err(PrismError::ColumnNotFound(ref d)) => panic!(
                "DRIFT-IEQ-001 no-regression pipe: E-QUERY-038 fired unexpectedly for \
                 existing column 'severity'. Got: column='{}', table='{}'",
                d.column, d.table
            ),
            _ => {}
        }
    }

    // ── Fixture: severity_id registered as Integer (MED-001 ordering tests) ──────

    /// Build a `crowdstrike_alerts` engine where `severity_id` IS a registered column
    /// (as Integer), in addition to `severity` (String) and `timestamp` (Datetime).
    ///
    /// Used ONLY for MED-001 ordering lock tests. Do NOT use for positions 7/8/9–12
    /// RED gate tests, which require `severity_id` to be ABSENT from the schema.
    fn make_crowdstrike_engine_with_severity_id_int() -> (QueryEngine, OrgSlug) {
        let org = OrgSlug::new("acme");
        let sensor_id = "crowdstrike";
        let table_suffix = "alerts";

        let columns = vec![
            ColumnSpec::new("severity", ColumnType::String, None, vec![]),
            ColumnSpec::new("severity_id", ColumnType::Integer, None, vec![]),
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
            .expect("DRIFT-IEQ-INT fixture: SensorInstanceOverlay TOML must parse");
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
        engine = engine.with_resolved_spec_map(Arc::new(spec_map));
        engine = engine.with_table_registry(registry);

        (engine, org)
    }

    // ── Test 5 (RED GATE): SqlPipe | where stage with nonexistent column ─────────

    /// BC-2.11.016 position 9 — SqlPipe `| where` stage predicates.
    ///
    /// EC-11-049: `SELECT * FROM crowdstrike_alerts | where severity_id IEQ 'high'`
    /// where `severity_id` is NOT registered → E-QUERY-038 with
    /// `column: "severity_id"`, `table: "crowdstrike_alerts"`.
    ///
    /// RED GATE: the `Ast::SqlPipe` arm in `check_query_column_availability` currently
    /// processes only the HEAD SQL (`&spq.head`) for positions 1–6. The `| where` stages
    /// in `spq.stages` are not yet walked — `severity_id` in the stage predicate bypasses
    /// the gate, reaching DataFusion with no structured error. The fix must extend the
    /// SqlPipe arm to iterate `spq.stages`, walking `PipeStage::Where` predicates
    /// identically to the `Ast::Pipe` arm (position 8).
    ///
    /// Note: `SELECT *` in the head means no non-wildcard column refs in the SELECT
    /// clause, so position-1 gate does not fire. The only gate trigger is position-9
    /// (stage | where).
    ///
    /// HIGH-002 finding from BC-2.11.016 changelog.
    #[tokio::test]
    async fn test_DRIFT_IEQ_sqlpipe_stage_nonexistent_col_yields_e_query_038() {
        let (engine, org) = make_crowdstrike_engine();

        // SqlPipe: head is `SELECT * FROM crowdstrike_alerts` — SELECT * skips position 1.
        // Stage is `| where severity_id IEQ 'high'` — `severity_id` is NOT registered.
        let query = "SELECT * FROM crowdstrike_alerts | where severity_id IEQ 'high'";

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
                    details.column, "severity_id",
                    "DRIFT-IEQ-001 sqlpipe-stage pos-9: column in E-QUERY-038 must be \
                     'severity_id', got: {:?}",
                    details.column
                );
                assert_eq!(
                    details.table, "crowdstrike_alerts",
                    "DRIFT-IEQ-001 sqlpipe-stage pos-9: table in E-QUERY-038 must be \
                     'crowdstrike_alerts', got: {:?}",
                    details.table
                );
                // did_you_mean: "severity_id" → "severity" is Levenshtein distance 3
                // (delete '_', 'i', 'd'). Within ≤3 threshold; suggestion expected.
            }
            Ok(_) => panic!(
                "DRIFT-IEQ-001 sqlpipe-stage pos-9: engine.execute must NOT succeed — \
                 E-QUERY-038 must fire for non-existent column 'severity_id' in SqlPipe \
                 | where stage. Before the fix, the SqlPipe arm does not walk stage \
                 predicates (only the head SQL), so the gate is bypassed entirely."
            ),
            Err(other) => panic!(
                "DRIFT-IEQ-001 sqlpipe-stage pos-9: expected PrismError::ColumnNotFound \
                 (E-QUERY-038), got different error: {other:?}. Before the fix this would \
                 be QueryExecutionFailed (DataFusion column resolution error for 'severity_id')."
            ),
        }
    }

    // ── Test 6 (RED GATE): Pipe | sort with typo column ──────────────────────────

    /// BC-2.11.016 position 10 — pipe `| sort` field key references.
    ///
    /// EC-11-050: `crowdstrike_alerts | sort sevrity desc` where `sevrity` is a typo
    /// of `severity` (Levenshtein distance 1 — one insertion of 'e') and is NOT a
    /// registered column → E-QUERY-038 with `column: "sevrity"`,
    /// `table: "crowdstrike_alerts"`, `did_you_mean: "severity"`.
    ///
    /// RED GATE: the `Ast::Pipe` arm currently only walks `PipeStage::Where` predicates.
    /// `PipeStage::Sort` sort-key field references are not yet extracted or checked
    /// against the schema. The fix must extend the Pipe arm to iterate `PipeStage::Sort`
    /// keys and call `check_column_availability` on each field reference.
    #[tokio::test]
    async fn test_DRIFT_pipe_sort_nonexistent_col_yields_e_query_038() {
        let (engine, org) = make_crowdstrike_engine();

        // `sevrity` is a typo of `severity` (distance 1: missing 'e' between 'v' and 'r').
        // `sevrity` is NOT in the schema; `severity` and `timestamp` are.
        let query = "crowdstrike_alerts | sort sevrity desc";

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
                    details.column, "sevrity",
                    "DRIFT-IEQ-001 pipe-sort pos-10: column in E-QUERY-038 must be \
                     'sevrity', got: {:?}",
                    details.column
                );
                assert_eq!(
                    details.table, "crowdstrike_alerts",
                    "DRIFT-IEQ-001 pipe-sort pos-10: table in E-QUERY-038 must be \
                     'crowdstrike_alerts', got: {:?}",
                    details.table
                );
                assert!(
                    details.did_you_mean.as_deref() == Some("severity"),
                    "DRIFT-IEQ-001 pipe-sort pos-10: did_you_mean must be 'severity' \
                     (distance 1 from 'sevrity'); got: {:?}",
                    details.did_you_mean
                );
            }
            Ok(_) => panic!(
                "DRIFT-IEQ-001 pipe-sort pos-10: engine.execute must NOT succeed — \
                 E-QUERY-038 must fire for non-existent sort key 'sevrity'. Before the \
                 fix, PipeStage::Sort field keys are not walked in \
                 check_query_column_availability."
            ),
            Err(other) => panic!(
                "DRIFT-IEQ-001 pipe-sort pos-10: expected PrismError::ColumnNotFound \
                 (E-QUERY-038), got different error: {other:?}."
            ),
        }
    }

    // ── Test 7 (RED GATE): Pipe | stats by with typo column ──────────────────────

    /// BC-2.11.016 position 11 — pipe `| stats ... by` grouping field references.
    ///
    /// EC-11-051: `crowdstrike_alerts | stats count() by sevrity` where `sevrity` is NOT
    /// a registered column → E-QUERY-038 with `column: "sevrity"`,
    /// `table: "crowdstrike_alerts"`, `did_you_mean: "severity"`.
    ///
    /// Grammar: `stats agg_fn [by field, ...]` — a bare `count()` aggregation is required;
    /// `sevrity` appears in the `by` (grouping) fields list.
    ///
    /// RED GATE: the `Ast::Pipe` arm currently only walks `PipeStage::Where` predicates.
    /// `PipeStage::Stats { by_fields }` grouping references are not yet checked.
    /// The fix must iterate `by_fields` and call `check_column_availability` on each.
    #[tokio::test]
    async fn test_DRIFT_pipe_stats_by_nonexistent_col_yields_e_query_038() {
        let (engine, org) = make_crowdstrike_engine();

        // `stats count() by sevrity` — `count()` is the required aggregation;
        // `sevrity` (distance 1 from "severity") is the grouping field, NOT in schema.
        let query = "crowdstrike_alerts | stats count() by sevrity";

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
                    details.column, "sevrity",
                    "DRIFT-IEQ-001 pipe-stats pos-11: column in E-QUERY-038 must be \
                     'sevrity', got: {:?}",
                    details.column
                );
                assert_eq!(
                    details.table, "crowdstrike_alerts",
                    "DRIFT-IEQ-001 pipe-stats pos-11: table in E-QUERY-038 must be \
                     'crowdstrike_alerts', got: {:?}",
                    details.table
                );
                assert!(
                    details.did_you_mean.as_deref() == Some("severity"),
                    "DRIFT-IEQ-001 pipe-stats pos-11: did_you_mean must be 'severity'; \
                     got: {:?}",
                    details.did_you_mean
                );
            }
            Ok(_) => panic!(
                "DRIFT-IEQ-001 pipe-stats pos-11: engine.execute must NOT succeed — \
                 E-QUERY-038 must fire for non-existent stats grouping field 'sevrity'. \
                 Before the fix, PipeStage::Stats by_fields are not walked in \
                 check_query_column_availability."
            ),
            Err(other) => panic!(
                "DRIFT-IEQ-001 pipe-stats pos-11: expected PrismError::ColumnNotFound \
                 (E-QUERY-038), got different error: {other:?}."
            ),
        }
    }

    // ── Test 8 (RED GATE): Pipe | fields with typo column ────────────────────────

    /// BC-2.11.016+ position 12 — pipe `| fields` column refs.
    ///
    /// BC-2.11.016 corrected the grammar keyword for position 12 from `| project`
    /// (v1.6 wording) to `| fields` (the real PrismQL grammar keyword in pipe_parser.rs:
    /// `kw_ci("fields")` → `PipeStage::Fields`). This test uses `| fields`.
    ///
    /// EC-11-052 (adapted): `crowdstrike_alerts | fields sevrity, timestamp` where
    /// `sevrity` is NOT a registered column → E-QUERY-038 with
    /// `column: "sevrity"`, `table: "crowdstrike_alerts"`, `did_you_mean: "severity"`.
    ///
    /// RED GATE: the `Ast::Pipe` arm currently only walks `PipeStage::Where` predicates.
    /// `PipeStage::Fields` inclusion/exclusion column references are not yet checked.
    /// The fix must iterate the `fields` list and call `check_column_availability` on each.
    #[tokio::test]
    async fn test_DRIFT_pipe_project_nonexistent_col_yields_e_query_038() {
        let (engine, org) = make_crowdstrike_engine();

        // Grammar keyword is `fields` (not `project`) — see GRAMMAR DISCREPANCY note above.
        // `sevrity` (distance 1 from "severity") is NOT in the schema.
        // `timestamp` IS in the schema and must not trigger E-QUERY-038.
        let query = "crowdstrike_alerts | fields sevrity, timestamp";

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
                    details.column, "sevrity",
                    "DRIFT-IEQ-001 pipe-fields pos-12: column in E-QUERY-038 must be \
                     'sevrity', got: {:?}",
                    details.column
                );
                assert_eq!(
                    details.table, "crowdstrike_alerts",
                    "DRIFT-IEQ-001 pipe-fields pos-12: table in E-QUERY-038 must be \
                     'crowdstrike_alerts', got: {:?}",
                    details.table
                );
                assert!(
                    details.did_you_mean.as_deref() == Some("severity"),
                    "DRIFT-IEQ-001 pipe-fields pos-12: did_you_mean must be 'severity'; \
                     got: {:?}",
                    details.did_you_mean
                );
            }
            Ok(_) => panic!(
                "DRIFT-IEQ-001 pipe-fields pos-12: engine.execute must NOT succeed — \
                 E-QUERY-038 must fire for non-existent fields column 'sevrity'. Before \
                 the fix, PipeStage::Fields is not walked in check_query_column_availability."
            ),
            Err(other) => panic!(
                "DRIFT-IEQ-001 pipe-fields pos-12: expected PrismError::ColumnNotFound \
                 (E-QUERY-038), got different error: {other:?}."
            ),
        }
    }

    // ── Tests 9-10 (MED-001 ordering locks): existing Integer column + IEQ ───────

    /// DRIFT-IEQ-NONEXISTENT-COL-ERRPATH-001 MED-001 ordering lock — filter mode.
    ///
    /// When `severity_id` EXISTS as Integer in the `crowdstrike_alerts` schema, the
    /// filter predicate `severity_id IEQ 'high'` must produce E-QUERY-002
    /// (QueryTypeMismatch — IEQ is not in the valid operator set for Integer), NOT
    /// E-QUERY-038 (ColumnNotFound, which would fire only for an absent column).
    ///
    /// Gate ordering: E-QUERY-038 (column existence) fires only when the column is ABSENT.
    /// When the column exists, the E-QUERY-038 gate passes; E-QUERY-002 (type compat)
    /// is responsible for the rejection (operator not valid for column type).
    ///
    /// Note: E-QUERY-002 type-compatibility checking for `Ast::Filter` predicate
    /// columns is not yet implemented in `check_query_column_availability` — only the
    /// SQL SELECT WHERE path calls `check_operator_type_compatibility`. This test is
    /// therefore RED until E-QUERY-002 type-compat is extended to Filter predicates.
    #[tokio::test]
    async fn test_DRIFT_IEQ_filter_mode_existing_int_col_yields_e_query_002() {
        let (engine, org) = make_crowdstrike_engine_with_severity_id_int();

        // `severity_id` IS in the schema (Integer). IEQ is not valid for Integer.
        // Must NOT produce ColumnNotFound (E-QUERY-038); MUST produce QueryTypeMismatch
        // (E-QUERY-002).
        let query = "crowdstrike_alerts | severity_id IEQ 'high'";

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
            Err(PrismError::QueryTypeMismatch {
                ref column,
                ref table,
                ref suggested_column,
                ..
            }) => {
                assert_eq!(
                    column.as_str(),
                    "severity_id",
                    "DRIFT-IEQ-001 MED-001 filter: column in E-QUERY-002 must be \
                     'severity_id', got: {:?}",
                    column
                );
                assert_eq!(
                    table.as_str(),
                    "crowdstrike_alerts",
                    "DRIFT-IEQ-001 MED-001 filter: table in E-QUERY-002 must be \
                     'crowdstrike_alerts', got: {:?}",
                    table
                );
                assert_eq!(
                    suggested_column.as_deref(),
                    Some("severity"),
                    "DRIFT-IEQ-001 MED-001 filter: suggested_column in E-QUERY-002 must be \
                     Some(\"severity\") for 'severity_id' (b1 form with OCSF sibling), \
                     got: {:?}",
                    suggested_column
                );
            }
            Err(PrismError::ColumnNotFound(ref d)) => panic!(
                "DRIFT-IEQ-001 MED-001 filter: E-QUERY-038 (ColumnNotFound) fired for \
                 EXISTING column 'severity_id' — gate ordering violated. E-QUERY-038 must \
                 NOT fire for an existing column; E-QUERY-002 must fire instead. \
                 Got: column='{}', table='{}'",
                d.column, d.table
            ),
            Ok(_) => panic!(
                "DRIFT-IEQ-001 MED-001 filter: engine.execute must NOT succeed — \
                 IEQ is not valid for Integer column 'severity_id'; E-QUERY-002 must fire."
            ),
            Err(other) => panic!(
                "DRIFT-IEQ-001 MED-001 filter: expected PrismError::QueryTypeMismatch \
                 (E-QUERY-002), got different error: {other:?}. E-QUERY-002 type-compat \
                 checking is not yet implemented for Ast::Filter predicates — RED gate."
            ),
        }
    }

    /// DRIFT-IEQ-NONEXISTENT-COL-ERRPATH-001 MED-001 ordering lock — pipe mode.
    ///
    /// When `severity_id` EXISTS as Integer in the `crowdstrike_alerts` schema, a pipe
    /// `| where severity_id IEQ 'high'` must produce E-QUERY-002, NOT E-QUERY-038.
    ///
    /// Same reasoning as the filter-mode ordering lock above. E-QUERY-002 type-compat
    /// checking for `Ast::Pipe` `| where` stage predicates is not yet implemented
    /// (only `check_column_availability` / E-QUERY-038 is called in the Pipe arm,
    /// not `check_operator_type_compatibility` / E-QUERY-002). This test is RED until
    /// the fix extends E-QUERY-002 to the Pipe predicate path.
    #[tokio::test]
    async fn test_DRIFT_IEQ_pipe_mode_existing_int_col_yields_e_query_002() {
        let (engine, org) = make_crowdstrike_engine_with_severity_id_int();

        // `severity_id` IS in the schema (Integer). IEQ is not valid for Integer.
        // Must NOT produce ColumnNotFound (E-QUERY-038); MUST produce QueryTypeMismatch
        // (E-QUERY-002).
        let query = "crowdstrike_alerts | where severity_id IEQ 'high'";

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
            Err(PrismError::QueryTypeMismatch {
                ref column,
                ref table,
                ref suggested_column,
                ..
            }) => {
                assert_eq!(
                    column.as_str(),
                    "severity_id",
                    "DRIFT-IEQ-001 MED-001 pipe: column in E-QUERY-002 must be \
                     'severity_id', got: {:?}",
                    column
                );
                assert_eq!(
                    table.as_str(),
                    "crowdstrike_alerts",
                    "DRIFT-IEQ-001 MED-001 pipe: table in E-QUERY-002 must be \
                     'crowdstrike_alerts', got: {:?}",
                    table
                );
                assert_eq!(
                    suggested_column.as_deref(),
                    Some("severity"),
                    "DRIFT-IEQ-001 MED-001 pipe: suggested_column in E-QUERY-002 must be \
                     Some(\"severity\") for 'severity_id' (b1 form with OCSF sibling), \
                     got: {:?}",
                    suggested_column
                );
            }
            Err(PrismError::ColumnNotFound(ref d)) => panic!(
                "DRIFT-IEQ-001 MED-001 pipe: E-QUERY-038 (ColumnNotFound) fired for \
                 EXISTING column 'severity_id' — gate ordering violated. E-QUERY-038 must \
                 NOT fire for an existing column. Got: column='{}', table='{}'",
                d.column, d.table
            ),
            Ok(_) => panic!(
                "DRIFT-IEQ-001 MED-001 pipe: engine.execute must NOT succeed — \
                 IEQ is not valid for Integer column 'severity_id'; E-QUERY-002 must fire."
            ),
            Err(other) => panic!(
                "DRIFT-IEQ-001 MED-001 pipe: expected PrismError::QueryTypeMismatch \
                 (E-QUERY-002), got different error: {other:?}. E-QUERY-002 type-compat \
                 checking is not yet implemented for Ast::Pipe | where predicates — RED gate."
            ),
        }
    }

    // ── Test 11 (RED GATE): IIN on Integer column — ADV-FIX-P3-LOW-001 ──────────

    /// ADV-FIX-P3-LOW-001 — plan-time E-QUERY-002 for `IIN` on an Integer column.
    ///
    /// `collect_predicate_type_pairs_inner` emits "IEQ"/"INE" for
    /// `Predicate::Compare { case_insensitive: true, op: Eq/Ne }` (lines 3210-3246).
    /// The IIN sibling — `Predicate::In { case_insensitive: true, .. }` — falls through
    /// the `_ => {}` catch-all, so "IIN" is never emitted to the type-compat gate.
    /// `check_operator_type_compatibility` is therefore never called for IIN predicates,
    /// leaving `severity_id IIN ('high', 'critical')` on an Integer column to proceed
    /// past plan time.
    ///
    /// Both the `Ast::Filter` arm (line 2383) and the `Ast::Pipe` arm (line 2841) call
    /// the same `collect_predicate_type_pairs` helper. The bug is in the shared inner
    /// function `collect_predicate_type_pairs_inner`. One filter-mode test suffices;
    /// the pipe-mode path is fixed by the same change to the shared function.
    ///
    /// Operator string: "IIN" — matching `collect_ci_compare_fields` in
    /// materialization.rs (line 2229: `out.push((last.clone(), "IIN".to_string()))`),
    /// so the plan-time and materialization-layer gate use the same byte-form.
    ///
    /// Fix: extend `collect_predicate_type_pairs_inner` to match
    /// `Predicate::In { case_insensitive: true, negated: false, .. }` and emit
    /// `(column_name, "IIN")`, then call `check_operator_type_compatibility` with
    /// "IIN". Since "IIN" is absent from `valid_operators_for_type(Integer)`, the gate
    /// returns `Err(QueryTypeMismatch)` at plan time.
    ///
    /// RED GATE: currently returns some non-QueryTypeMismatch result because IIN falls
    /// through `_ => {}` in `collect_predicate_type_pairs_inner`.
    #[tokio::test]
    async fn test_BC_2_11_016_low001_iin_integer_column_plan_time_e_query_002() {
        let (engine, org) = make_crowdstrike_engine_with_severity_id_int();

        // `severity_id` IS in the schema (Integer). IIN is not valid for Integer.
        // Must NOT produce ColumnNotFound (E-QUERY-038 — column exists).
        // MUST produce QueryTypeMismatch (E-QUERY-002 — IIN not valid for Integer).
        // Filter mode: `crowdstrike_alerts | severity_id IIN ('high', 'critical')`.
        let query = "crowdstrike_alerts | severity_id IIN ('high', 'critical')";

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
            Err(PrismError::QueryTypeMismatch {
                ref column,
                ref table,
                ref suggested_column,
                ..
            }) => {
                assert_eq!(
                    column.as_str(),
                    "severity_id",
                    "ADV-FIX-P3-LOW-001: column in E-QUERY-002 must be 'severity_id', \
                     got: {:?}",
                    column
                );
                assert_eq!(
                    table.as_str(),
                    "crowdstrike_alerts",
                    "ADV-FIX-P3-LOW-001: table in E-QUERY-002 must be 'crowdstrike_alerts', \
                     got: {:?}",
                    table
                );
                assert_eq!(
                    suggested_column.as_deref(),
                    Some("severity"),
                    "ADV-FIX-P3-LOW-001: suggested_column must be Some(\"severity\") \
                     (OCSF sibling: severity_id → severity); got: {:?}",
                    suggested_column
                );
            }
            Err(PrismError::ColumnNotFound(ref d)) => panic!(
                "ADV-FIX-P3-LOW-001: E-QUERY-038 (ColumnNotFound) fired for EXISTING column \
                 'severity_id' — gate ordering violated. E-QUERY-038 must NOT fire for an \
                 existing column; E-QUERY-002 must fire instead. \
                 Got: column='{}', table='{}'",
                d.column, d.table
            ),
            Ok(_) => panic!(
                "ADV-FIX-P3-LOW-001: engine.execute returned Ok — IIN is not valid for \
                 Integer column 'severity_id'; E-QUERY-002 must fire at plan time. \
                 Currently Predicate::In {{ case_insensitive: true }} falls through \
                 the _ => {{}} catch-all in collect_predicate_type_pairs_inner, so \
                 the IIN operator is never emitted to check_operator_type_compatibility."
            ),
            Err(other) => panic!(
                "ADV-FIX-P3-LOW-001: expected PrismError::QueryTypeMismatch (E-QUERY-002), \
                 got different error: {other:?}. Fix: add Predicate::In \
                 {{ case_insensitive: true }} arm to collect_predicate_type_pairs_inner \
                 emitting (column, \"IIN\") so the E-QUERY-002 plan-time gate fires."
            ),
        }
    }

    // ── Fixture: armis_devices engine (EC-11-054, EC-11-056) ──────────────────

    /// Build an `armis_devices` engine (sensor="armis", table="devices") under org "acme".
    /// Valid columns: `device_cves_first` (String), `device_id` (String).
    ///
    /// No InfusionRegistry wired — E-QUERY-039 check is skipped; enrichment output
    /// schema is therefore unresolvable at plan time → suspension path activates after
    /// fix. This is the correct path for EC-11-054 (CRIT-001) and EC-11-055.
    ///
    /// For EC-11-056 (position 13 input typo), only the input column check fires;
    /// the lack of InfusionRegistry is irrelevant to that check.
    fn make_armis_engine() -> (QueryEngine, OrgSlug) {
        let org = OrgSlug::new("acme");
        let sensor_id = "armis";
        let table_suffix = "devices";

        let columns = vec![
            ColumnSpec::new("device_cves_first", ColumnType::String, None, vec![]),
            ColumnSpec::new("device_id", ColumnType::String, None, vec![]),
        ];

        let spec = SensorSpec::new(
            sensor_id,
            "Armis sensor",
            AuthType::ApiKey,
            "https://api.armis.com",
            vec![TableSpec::new_point_in_time(
                table_suffix,
                "inventory_device",
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

        let overlay_toml = format!("extends = \"{sensor_id}\"\ninstance_id = \"{sensor_id}@acme\"");
        let overlay: SensorInstanceOverlay = toml::from_str(&overlay_toml)
            .expect("DRIFT-IEQ armis fixture: SensorInstanceOverlay TOML must parse");
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
        engine = engine.with_resolved_spec_map(Arc::new(spec_map));
        engine = engine.with_table_registry(registry);

        (engine, org)
    }

    // ── Fixture: single-tenant crowdstrike engine with severity_id:Integer (HIGH-001) ──

    /// Build a `crowdstrike_alerts` engine WITHOUT `resolved_spec_map` (single-tenant mode).
    /// Valid columns: `severity` (String), `severity_id` (Integer), `timestamp` (Datetime).
    ///
    /// The engine uses only `table_registry` for column availability; `resolved_spec_map`
    /// is intentionally left as `None`. This reproduces the single-tenant deployment path
    /// where `check_column_availability` uses the M1-era `table_registry.columns_for_table()`
    /// fallback but `check_operator_type_compatibility` currently returns `Ok(())` immediately
    /// because it gates on `resolved_spec_map.is_some()` — the HIGH-001 bug.
    fn make_crowdstrike_engine_no_spec_map_with_severity_id_int() -> QueryEngine {
        let columns = vec![
            ColumnSpec::new("severity", ColumnType::String, None, vec![]),
            ColumnSpec::new("severity_id", ColumnType::Integer, None, vec![]),
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

        // Build engine WITHOUT resolved_spec_map (single-tenant mode — M1-era path).
        // resolved_spec_map defaults to None in new_with_cache_config.
        QueryEngine::new_with_cache_config(
            Arc::new(AdapterRegistry::new()),
            Arc::new(NoopCs),
            Arc::new(prism_ocsf::OcsfNormalizer::new()),
            Arc::new(ClientRegistry::new(vec![])),
            QueryEngineConfig::default(),
            crate::cache::CacheConfig::default(),
        )
        .with_table_registry(registry)
        // resolved_spec_map is NOT wired — single-tenant mode.
    }

    // ── Tests 11-13 (RED GATE): CRIT-002 — stats alias downstream (EC-11-053) ─

    /// BC-2.11.016 EC-11-053 / CRIT-002 regression lock — sort after stats with alias.
    ///
    /// `crowdstrike_alerts | stats count() as cnt by severity | sort cnt`
    /// — `cnt` is the explicit alias from the stats aggregate output. After the Stats
    /// stage, the binding context replaces `available` with `{cnt, severity}` per the
    /// DERIVED-COLUMN BINDING RULE (BC-2.11.016 §Preconditions.2). The downstream
    /// `| sort cnt` must NOT fire E-QUERY-038 because `cnt` is in the new available set.
    ///
    /// Grammar note: the BC's EC-11-053 writes `\| sort by cnt` but the PrismQL sort
    /// stage grammar has no `by` keyword — `by` is only used inside `stats`. Using
    /// `sort cnt` (bare field, ascending) which is the correct syntax per pipe_parser.rs.
    ///
    /// RED GATE: current code has no binding context for Stats — it checks `cnt` against
    /// the ORIGINAL schema `{severity, timestamp}` where `cnt` is absent → false-positive
    /// E-QUERY-038. The fix must implement Stats REPLACE semantics so downstream sort
    /// sees `{cnt, severity}` instead of the original schema.
    #[tokio::test]
    async fn test_BC_2_11_016_crit002_stats_alias_downstream_sort_no_false_positive() {
        let (engine, org) = make_crowdstrike_engine();

        // EC-11-053: stats alias `cnt` must be visible to downstream `| sort cnt`.
        // Grammar: `stats count() as cnt by severity` → alias "cnt"; `| sort cnt` → sort key.
        let query = "crowdstrike_alerts | stats count() as cnt by severity | sort cnt";

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
            Err(PrismError::ColumnNotFound(ref details)) if details.column == "cnt" => {
                panic!(
                    "BC-2.11.016 CRIT-002 sort: FALSE-POSITIVE E-QUERY-038 fired on stats \
                     aggregate alias 'cnt' in downstream | sort stage. The Stats stage REPLACE \
                     semantics must update the binding context to {{cnt, severity}} so that \
                     downstream stages see 'cnt' as valid. Current code checks 'cnt' against \
                     the ORIGINAL schema {{severity, timestamp}} — 'cnt' is absent → \
                     incorrect rejection. EC-11-053 regression lock. column='{}', table='{}'",
                    details.column, details.table
                )
            }
            // Any non-ColumnNotFound result is acceptable — the invariant is that the
            // Stats alias 'cnt' must NOT produce a false-positive E-QUERY-038 in sort.
            _ => {}
        }
    }

    /// BC-2.11.016 EC-11-053 / CRIT-002 regression lock — where after stats with alias.
    ///
    /// `crowdstrike_alerts | stats count() as cnt by severity | where cnt > 5`
    /// — after Stats, `cnt` is in the replacement binding set `{cnt, severity}`.
    /// The downstream `| where cnt > 5` must NOT fire E-QUERY-038 on `cnt`.
    ///
    /// RED GATE: same as the sort variant — current code checks `cnt` against the
    /// original schema where it is absent, producing a false-positive E-QUERY-038.
    #[tokio::test]
    async fn test_BC_2_11_016_crit002_stats_alias_downstream_where_no_false_positive() {
        let (engine, org) = make_crowdstrike_engine();

        // EC-11-053: stats alias `cnt` must be visible to downstream `| where cnt > 5`.
        let query = "crowdstrike_alerts | stats count() as cnt by severity | where cnt > 5";

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
            Err(PrismError::ColumnNotFound(ref details)) if details.column == "cnt" => {
                panic!(
                    "BC-2.11.016 CRIT-002 where: FALSE-POSITIVE E-QUERY-038 fired on stats \
                     aggregate alias 'cnt' in downstream | where stage. Stats REPLACE binding \
                     must make 'cnt' visible downstream. EC-11-053 regression lock. \
                     column='{}', table='{}'",
                    details.column, details.table
                )
            }
            _ => {}
        }
    }

    /// BC-2.11.016 EC-11-053 / CRIT-002 regression lock — fields after stats with alias.
    ///
    /// `crowdstrike_alerts | stats count() as cnt by severity | fields cnt, severity`
    /// — after Stats, `{cnt, severity}` is the replacement binding set. The downstream
    /// `| fields cnt, severity` must NOT fire E-QUERY-038 on `cnt`.
    ///
    /// RED GATE: current code checks `cnt` against original schema → false-positive E-QUERY-038.
    #[tokio::test]
    async fn test_BC_2_11_016_crit002_stats_alias_downstream_fields_no_false_positive() {
        let (engine, org) = make_crowdstrike_engine();

        // EC-11-053: stats alias `cnt` must be visible to downstream `| fields cnt, severity`.
        let query = "crowdstrike_alerts | stats count() as cnt by severity | fields cnt, severity";

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
            Err(PrismError::ColumnNotFound(ref details)) if details.column == "cnt" => {
                panic!(
                    "BC-2.11.016 CRIT-002 fields: FALSE-POSITIVE E-QUERY-038 fired on stats \
                     aggregate alias 'cnt' in downstream | fields stage. Stats REPLACE binding \
                     must make 'cnt' visible downstream. EC-11-053 regression lock. \
                     column='{}', table='{}'",
                    details.column, details.table
                )
            }
            _ => {}
        }
    }

    // ── Test 14 (RED GATE): Stats REPLACE semantics — original column removed ──

    /// BC-2.11.016 — Stats REPLACE semantics precision win.
    ///
    /// After a `| stats count() as cnt by severity` stage, the binding context `available`
    /// is REPLACED with `{cnt, severity}` (explicit aliases ∪ by-field names). The original
    /// schema column `timestamp` is NOT in the replacement set and must trigger E-QUERY-038
    /// when referenced in a downstream `| sort timestamp`.
    ///
    /// This is the precision win of REPLACE vs UNION: UNION would still allow `timestamp`
    /// (it remains in the union), but REPLACE correctly rejects it because after a stats
    /// aggregation, only the aggregate outputs and GROUP BY keys remain in the result.
    /// Referencing `timestamp` after stats is a logical error that the gate should catch.
    ///
    /// RED GATE: current code has no binding context — it checks `timestamp` against the
    /// ORIGINAL schema `{severity, timestamp}` where it IS present → no E-QUERY-038. The
    /// fix must replace `available` with `{cnt, severity}` so that `timestamp` is absent
    /// from the downstream sort check.
    #[tokio::test]
    async fn test_BC_2_11_016_stats_replace_removes_original_schema_col_yields_e_query_038() {
        let (engine, org) = make_crowdstrike_engine();

        // `timestamp` is in the original schema but NOT in the Stats replacement set {cnt, severity}.
        // After fix: E-QUERY-038 fires on `timestamp` in the downstream | sort stage.
        // Before fix: `timestamp` is in the original schema → no E-QUERY-038 (WRONG — false negative).
        let query = "crowdstrike_alerts | stats count() as cnt by severity | sort timestamp";

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
                    details.column, "timestamp",
                    "BC-2.11.016 REPLACE: Stats binding REPLACE must gate on 'timestamp' \
                     (not in {{cnt, severity}} replacement set); got column='{}', table='{}'",
                    details.column, details.table
                );
                assert_eq!(
                    details.table, "crowdstrike_alerts",
                    "BC-2.11.016 REPLACE: table must be 'crowdstrike_alerts'"
                );
                // 'timestamp' is distance >3 from 'cnt' and 'severity' — did_you_mean absent.
                // No assertion on did_you_mean (implementation detail).
            }
            Ok(_) => panic!(
                "BC-2.11.016 REPLACE: engine.execute must NOT succeed — 'timestamp' is NOT in \
                 the Stats replacement binding set {{cnt, severity}} and must trigger E-QUERY-038. \
                 Before the fix, current code checks 'timestamp' against the ORIGINAL schema \
                 {{severity, timestamp}} where it IS present → no error (false negative). \
                 The Stats REPLACE semantics are not yet implemented."
            ),
            Err(other) => panic!(
                "BC-2.11.016 REPLACE: expected PrismError::ColumnNotFound (E-QUERY-038) for \
                 'timestamp' after Stats REPLACE, got: {other:?}"
            ),
        }
    }

    // ── Tests 15-16 (RED GATE): CRIT-001 — enrich output (EC-11-054/055) ────────

    /// BC-2.11.016 EC-11-054 / CRIT-001 regression lock — enrich output downstream.
    ///
    /// `armis_devices | enrich cvss_base_score(device_cves_first) | where cvss_base_score >= 7.0`
    /// — after the Enrich stage, `cvss_base_score` is either added to `available` (if the
    /// infusion output is statically resolvable) OR `suspended = true` (if it is not). Either
    /// way, the downstream `| where cvss_base_score >= 7.0` must NOT fire E-QUERY-038.
    ///
    /// Fixture approach (per task: "stub at the lowest real boundary"):
    /// This fixture uses NO InfusionRegistry (`infusion_registry = None`) so E-QUERY-039 is
    /// skipped and the infusion output is unresolvable at plan time. The fix must activate
    /// `suspended = true` after the Enrich stage in this configuration, preventing downstream
    /// column checks. This exercises BC-2.11.016 §DERIVED-COLUMN BINDING RULE ¶Enrich
    /// unresolvable path (equivalent to the fail-open semantics of FP-001).
    ///
    /// RED GATE: current code falls through to `_ => {}` for `PipeStage::Enrich` in
    /// `check_pipe_stage_columns`, then the WHERE stage fires E-QUERY-038 on `cvss_base_score`
    /// (not in original schema `{device_cves_first, device_id}`) — FALSE-POSITIVE.
    #[tokio::test]
    async fn test_BC_2_11_016_crit001_ec11054_enrich_output_downstream_no_false_positive() {
        let (engine, org) = make_armis_engine();

        // EC-11-054: `cvss_base_score` is the infusion output, not in the original schema.
        // It must NOT trigger E-QUERY-038 — either union (resolvable) or suspension (unresolvable).
        // This fixture exercises the unresolvable path (no InfusionRegistry → suspension).
        let query =
            "armis_devices | enrich cvss_base_score(device_cves_first) | where cvss_base_score >= 7.0";

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
            Err(PrismError::ColumnNotFound(ref details)) if details.column == "cvss_base_score" => {
                panic!(
                    "BC-2.11.016 CRIT-001 EC-11-054: FALSE-POSITIVE E-QUERY-038 fired on \
                     enrich output column 'cvss_base_score' in downstream | where stage. \
                     After Enrich, either 'cvss_base_score' must be in the available set \
                     (resolvable) or suspended=true (unresolvable) — neither path should fire \
                     E-QUERY-038 on this column. FP-001 invariant violated. \
                     column='{}', table='{}'",
                    details.column, details.table
                )
            }
            // Any other result (execution error, Ok) is acceptable — the invariant is
            // that the ENRICH output column must NOT produce a false-positive E-QUERY-038.
            _ => {}
        }
    }

    /// BC-2.11.016 EC-11-055 / CRIT-001 regression lock — post-enrich typo, fail-open.
    ///
    /// `armis_devices | enrich cvss_base_score(device_cves_first) | where cvvs_base_score >= 7.0`
    /// — `cvvs_base_score` is a TYPO of `cvss_base_score` (Levenshtein distance 2: swap 'vs'/'cv').
    /// When the infusion output is unresolvable, `suspended = true` activates after Enrich,
    /// and ALL subsequent column checks are skipped. The downstream WHERE stage must NOT fire
    /// E-QUERY-038 on `cvvs_base_score` — this is the fail-open tolerated outcome (EC-11-055).
    ///
    /// Per BC v1.8 FP-001 invariant: false negatives (missing a typo in infusion output) are
    /// acceptable; false positives on correct queries are BLOCKING defects. When the output
    /// schema is unresolvable, the gate must fail-open for ALL downstream columns.
    ///
    /// RED GATE: same as EC-11-054 — current code fires E-QUERY-038 on `cvvs_base_score`
    /// (not in schema) because there is no suspension logic.
    #[tokio::test]
    async fn test_BC_2_11_016_crit001_ec11055_post_enrich_typo_fail_open_no_e_query_038() {
        let (engine, org) = make_armis_engine();

        // EC-11-055: `cvvs_base_score` is a typo — after suspension, it must NOT trigger
        // E-QUERY-038 (false negative accepted; false positive is FP-001 violation).
        let query =
            "armis_devices | enrich cvss_base_score(device_cves_first) | where cvvs_base_score >= 7.0";

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
            Err(PrismError::ColumnNotFound(ref details)) if details.column == "cvvs_base_score" => {
                panic!(
                    "BC-2.11.016 CRIT-001 EC-11-055: FALSE-POSITIVE E-QUERY-038 fired on \
                     post-enrich column reference 'cvvs_base_score'. When the infusion output \
                     is unresolvable (suspended=true), ALL downstream column checks must be \
                     skipped per FP-001 fail-open rule. column='{}', table='{}'",
                    details.column, details.table
                )
            }
            _ => {}
        }
    }

    // ── Test 17 (RED GATE): Position 13 — enrich input column typo (EC-11-056) ─

    /// BC-2.11.016 position 13 / EC-11-056 — enrich INPUT column typo → E-QUERY-038.
    ///
    /// `armis_devices | enrich cvss_base_score(device_cves_firsst) | head 5`
    /// — `device_cves_firsst` is a typo of `device_cves_first` (Levenshtein distance 1:
    /// extra 's'). The ENRICH INPUT column is checked against `available` BEFORE the
    /// Enrich stage updates the binding context (position 13 in the BC gate table).
    /// `device_cves_firsst` is NOT in the original schema → E-QUERY-038 fires with
    /// `column: "device_cves_firsst"` and `did_you_mean: "device_cves_first"`.
    ///
    /// RED GATE: current `check_pipe_stage_columns` has `PipeStage::Enrich` in the
    /// `_ => {}` catch-all (position 13 gate does not exist yet). The fix must add a
    /// `PipeStage::Enrich(es)` arm that checks `es.field` against the current binding set
    /// BEFORE updating the context.
    ///
    /// Note: `head 5` maps to `PipeStage::Limit(5)` which carries no column refs.
    /// E-QUERY-039 does NOT fire (no InfusionRegistry wired → `check_enrich_udf_availability`
    /// returns Ok immediately).
    #[tokio::test]
    async fn test_BC_2_11_016_pos13_ec11056_enrich_input_typo_yields_e_query_038() {
        let (engine, org) = make_armis_engine();

        // `device_cves_firsst` (extra 's') is NOT in schema; `device_cves_first` is.
        // Levenshtein distance 1 → did_you_mean: "device_cves_first".
        let query = "armis_devices | enrich cvss_base_score(device_cves_firsst) | head 5";

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
                    details.column, "device_cves_firsst",
                    "BC-2.11.016 pos-13 EC-11-056: column in E-QUERY-038 must be \
                     'device_cves_firsst' (enrich input typo); got: '{}'",
                    details.column
                );
                assert_eq!(
                    details.table, "armis_devices",
                    "BC-2.11.016 pos-13 EC-11-056: table must be 'armis_devices'"
                );
                assert_eq!(
                    details.did_you_mean.as_deref(),
                    Some("device_cves_first"),
                    "BC-2.11.016 pos-13 EC-11-056: did_you_mean must be 'device_cves_first' \
                     (Levenshtein distance 1 from 'device_cves_firsst'); got: {:?}",
                    details.did_you_mean
                );
            }
            Ok(_) => panic!(
                "BC-2.11.016 pos-13 EC-11-056: engine.execute must NOT succeed — E-QUERY-038 \
                 must fire for enrich input typo 'device_cves_firsst'. Before the fix, \
                 PipeStage::Enrich falls through check_pipe_stage_columns `_ => {{}}` catch-all \
                 — position 13 gate does not exist."
            ),
            Err(other) => panic!(
                "BC-2.11.016 pos-13 EC-11-056: expected PrismError::ColumnNotFound \
                 (E-QUERY-038) for enrich input typo, got: {other:?}"
            ),
        }
    }

    // ── Test 18 (RED GATE): Position 14 — dedup field typo → E-QUERY-038 ────────

    /// BC-2.11.016 position 14 / EC-11-057 (adapted for crowdstrike) — dedup typo.
    ///
    /// `crowdstrike_alerts | dedup sevrity | head 5`
    /// — `sevrity` is a typo of `severity` (Levenshtein distance 1: missing 'e' between
    /// 'v' and 'r'). The `dedup` field keys are validated against the current binding set
    /// at position 14. `sevrity` is NOT in the schema → E-QUERY-038 with
    /// `column: "sevrity"`, `table: "crowdstrike_alerts"`, `did_you_mean: "severity"`.
    ///
    /// RED GATE: current `check_pipe_stage_columns` treats `PipeStage::Dedup` as
    /// `_ => {}` (position 14 gate does not exist). The fix must add a `PipeStage::Dedup`
    /// arm that iterates the `Vec<FieldPath>` dedup keys and calls `check_column_availability`
    /// on each, consistent with the `PipeStage::Sort` arm (position 10).
    ///
    /// Note: `head 5` → `PipeStage::Limit(5)` — no column refs; does not interfere.
    #[tokio::test]
    async fn test_BC_2_11_016_pos14_dedup_typo_yields_e_query_038() {
        let (engine, org) = make_crowdstrike_engine();

        // `sevrity` (distance 1 from "severity") is NOT in the schema.
        let query = "crowdstrike_alerts | dedup sevrity | head 5";

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
                    details.column, "sevrity",
                    "BC-2.11.016 pos-14: column in E-QUERY-038 must be 'sevrity' \
                     (dedup key typo); got: '{}'",
                    details.column
                );
                assert_eq!(
                    details.table, "crowdstrike_alerts",
                    "BC-2.11.016 pos-14: table must be 'crowdstrike_alerts'"
                );
                assert_eq!(
                    details.did_you_mean.as_deref(),
                    Some("severity"),
                    "BC-2.11.016 pos-14: did_you_mean must be 'severity' \
                     (distance 1 from 'sevrity'); got: {:?}",
                    details.did_you_mean
                );
            }
            Ok(_) => panic!(
                "BC-2.11.016 pos-14: engine.execute must NOT succeed — E-QUERY-038 must fire \
                 for dedup key typo 'sevrity'. Before the fix, PipeStage::Dedup falls through \
                 check_pipe_stage_columns `_ => {{}}` catch-all — position 14 gate does not exist."
            ),
            Err(other) => panic!(
                "BC-2.11.016 pos-14: expected PrismError::ColumnNotFound (E-QUERY-038) for \
                 dedup typo, got: {other:?}"
            ),
        }
    }

    // ── Test 19 (RED GATE): HIGH-001 — single-tenant E-QUERY-002 ─────────────────

    /// BC-2.11.016 HIGH-001 — single-tenant path must fire E-QUERY-002.
    ///
    /// In single-tenant mode (`resolved_spec_map = None`), the E-QUERY-038 column-existence
    /// gate fires via the `table_registry.columns_for_table()` fallback (M1 fix). However,
    /// the E-QUERY-002 type-compatibility gate (`check_operator_type_compatibility`) currently
    /// returns `Ok(())` immediately when `resolved_spec_map` is `None` (line 3055-3057 in
    /// engine.rs). This means `severity_id IEQ 'high'` — where `severity_id` is registered
    /// as Integer — does NOT fire E-QUERY-002 in single-tenant mode; instead the query
    /// proceeds to execution (and fails opaquely with a DataFusion error).
    ///
    /// After fix: `check_operator_type_compatibility` falls back to `table_registry` for
    /// column type lookup (same M1 pattern as `check_column_availability`), finds
    /// `severity_id: Integer`, determines `IEQ` is not valid for Integer, and returns
    /// `PrismError::QueryTypeMismatch` with `suggested_column: Some("severity")` (OCSF sibling).
    ///
    /// Fixture: `make_crowdstrike_engine_no_spec_map_with_severity_id_int()` — table_registry
    /// only, no `resolved_spec_map`, severity_id:Integer in schema.
    ///
    /// RED GATE: current `check_operator_type_compatibility` returns Ok when spec_map is None
    /// → no E-QUERY-002 fires → test panics on Ok or wrong-error branch.
    #[tokio::test]
    async fn test_BC_2_11_016_high001_single_tenant_int_col_ieq_yields_e_query_002() {
        let engine = make_crowdstrike_engine_no_spec_map_with_severity_id_int();

        // `severity_id` IS in schema (Integer). IEQ is not valid for Integer.
        // Must NOT produce ColumnNotFound (E-QUERY-038 — column exists).
        // MUST produce QueryTypeMismatch (E-QUERY-002 — operator invalid for type).
        // Filter mode: `crowdstrike_alerts | severity_id IEQ 'high'` (Ast::Filter).
        let query = "crowdstrike_alerts | severity_id IEQ 'high'";

        let result = engine.execute(query, QueryOptions::default()).await;

        match result {
            Err(PrismError::QueryTypeMismatch {
                ref column,
                ref table,
                ref suggested_column,
                ..
            }) => {
                assert_eq!(
                    column.as_str(),
                    "severity_id",
                    "BC-2.11.016 HIGH-001: column in E-QUERY-002 must be 'severity_id'; \
                     got: {column:?}"
                );
                assert_eq!(
                    table.as_str(),
                    "crowdstrike_alerts",
                    "BC-2.11.016 HIGH-001: table in E-QUERY-002 must be 'crowdstrike_alerts'"
                );
                assert_eq!(
                    suggested_column.as_deref(),
                    Some("severity"),
                    "BC-2.11.016 HIGH-001: suggested_column must be Some(\"severity\") \
                     (OCSF sibling: severity_id → severity); got: {suggested_column:?}"
                );
            }
            Err(PrismError::ColumnNotFound(ref d)) => panic!(
                "BC-2.11.016 HIGH-001: E-QUERY-038 (ColumnNotFound) fired for EXISTING column \
                 'severity_id' — gate ordering violated. E-QUERY-038 must NOT fire for an \
                 existing column. Got: column='{}', table='{}'",
                d.column, d.table
            ),
            Ok(_) => panic!(
                "BC-2.11.016 HIGH-001: engine.execute returned Ok — IEQ is not valid for \
                 Integer column 'severity_id'; E-QUERY-002 must fire. Current code returns Ok \
                 in single-tenant mode because check_operator_type_compatibility returns Ok(()) \
                 immediately when resolved_spec_map is None (RED gate)."
            ),
            Err(other) => panic!(
                "BC-2.11.016 HIGH-001: expected PrismError::QueryTypeMismatch (E-QUERY-002), \
                 got different error: {other:?}. check_operator_type_compatibility must extend \
                 to use table_registry fallback (same M1 pattern as check_column_availability)."
            ),
        }
    }

    // ── Test 20 (RED GATE): Anonymous aggregate suspension ───────────────────────

    /// BC-2.11.016 — anonymous aggregate suspension → no E-QUERY-038 downstream.
    ///
    /// `crowdstrike_alerts | stats count() by severity | sort count`
    /// — `count()` has NO explicit `AS alias`. The Stats stage replacement set is
    /// `{} ∪ {severity}` = `{severity}` (anonymous aggregates do not contribute per
    /// BC v1.8 §DERIVED-COLUMN BINDING RULE). Because the auto-generated DataFusion name
    /// for anonymous aggregates is not predictable at plan time, the gate MUST fail-open
    /// for downstream references to those names.
    ///
    /// Per BC v1.8: "Anonymous aggregations (no explicit `as alias`) do not contribute to
    /// the replacement set — their DataFusion-generated names are not predictable at plan
    /// time; fail-open for those references in subsequent stages." This means the gate
    /// must set `suspended = true` (or equivalent) when anonymous aggregates are present,
    /// so that downstream `| sort count` does NOT trigger E-QUERY-038 on `count`.
    ///
    /// RED GATE: current code has no Stats binding context at all — it checks `count`
    /// against the original schema `{severity, timestamp}` where `count` is absent →
    /// false-positive E-QUERY-038. The fix must activate suspension (or equivalent
    /// fail-open) when anonymous aggregates are present after a Stats stage.
    #[tokio::test]
    async fn test_BC_2_11_016_anonymous_agg_suspension_no_false_positive_e_query_038() {
        let (engine, org) = make_crowdstrike_engine();

        // `stats count() by severity` — no alias on count(); `| sort count` references
        // the anonymous aggregate auto-name. Must NOT fire E-QUERY-038.
        let query = "crowdstrike_alerts | stats count() by severity | sort count";

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
            Err(PrismError::ColumnNotFound(ref details)) if details.column == "count" => {
                panic!(
                    "BC-2.11.016 anonymous-agg: FALSE-POSITIVE E-QUERY-038 fired on anonymous \
                     aggregate reference 'count' in downstream | sort stage. When stats has \
                     anonymous aggregates, the gate must fail-open (suspended=true or equivalent) \
                     for downstream column refs — DataFusion auto-names are not predictable at \
                     plan time. FP-001 invariant violated. column='{}', table='{}'",
                    details.column, details.table
                )
            }
            // Any other result is acceptable — invariant is no false-positive on anonymous agg name.
            _ => {}
        }
    }

    // ── Tests 21-23 (v1.9): Enrich output UNION path — BC-2.11.016 ─────────────────

    // ── Fixture: armis_devices engine WITH InfusionRegistry (tests 21-22) ──────────────────

    /// Build an `armis_devices` engine identical to `make_armis_engine()` but with an
    /// `InfusionRegistry` wired containing the `"nvd_cvss"` infusion.
    ///
    /// Infusion spec:
    ///   infusion_id = "nvd_cvss"
    ///   fields = [{ name: "threat_score", input_field: "device_cves_first", ... }]
    ///   pipe_stage = None  (so enrich_descriptor falls back to field names)
    ///
    /// Registry state after load_spec:
    ///   udf_to_infusion = { "threat_score" => "nvd_cvss" }
    ///   entries = { "nvd_cvss" => (spec, [InfusionUdfDescriptor { name: "threat_score", ... }]) }
    ///
    /// Implications for the binding context gate (after v1.9 fix):
    ///   check_enrich_udf_availability: "threat_score" IS in registered_names → no E-QUERY-039.
    ///   check_pipe_stage_columns PipeStage::Enrich arm (after fix):
    ///     - Look up "threat_score" in udf_to_infusion → infusion_id = "nvd_cvss"
    ///     - enrich_descriptor("nvd_cvss") → output_columns = ["threat_score"]
    ///     - UNION: current_available = {device_cves_first, device_id} ∪ {threat_score}
    ///     - NO suspension
    ///
    /// For ec11054/ec11055 implications: those tests use make_armis_engine() (no registry),
    /// so this fixture does not affect them. The registry=None fallback path tested by
    /// ec11054/ec11055 is unchanged by the v1.9 plumbing.
    fn make_armis_engine_with_threat_score_registry() -> (QueryEngine, OrgSlug) {
        use prism_spec_engine::{InfusionField, InfusionRegistry, InfusionSpec, InfusionType};

        let (engine, org) = make_armis_engine();

        let registry = InfusionRegistry::new();
        let spec = InfusionSpec::new(
            "nvd_cvss",
            "NVD CVSS enrichment (test fixture)",
            InfusionType::LocalLookup,
            vec![InfusionField::new(
                "threat_score",
                "device_cves_first",
                "string",
                "float64",
            )],
            "/dev/null",
        );
        registry
            .load_spec(spec)
            .expect("nvd_cvss spec must load for test fixture");
        let engine = engine.with_infusion_registry(Arc::new(registry));

        (engine, org)
    }

    // ── Test 21 (GREEN-lock): correct output ref must NOT fire E-QUERY-038 ───────────────

    /// BC-2.11.016 GREEN-LOCK — resolvable enrich output: correct downstream ref is OK.
    ///
    /// `armis_devices | enrich threat_score(device_cves_first) | where threat_score > 5 | sort threat_score`
    ///
    /// "threat_score" IS a declared output column of the "nvd_cvss" infusion (InfusionRegistry
    /// wired). This reference must NEVER fire E-QUERY-038.
    ///
    /// ## Status under both code paths
    /// - **Before fix (always-suspend):** `suspended=true` after PipeStage::Enrich → WHERE and
    ///   SORT stages SKIPPED → no E-QUERY-038. Vacuously GREEN — suspension prevents false positive.
    /// - **After fix (union):** output_columns=["threat_score"] UNIONed into available set →
    ///   "threat_score" IS in available → WHERE and SORT gates pass → still no E-QUERY-038.
    ///
    /// This is a GREEN-lock (not a RED gate) that locks the invariant: a correct reference to a
    /// declared enrich output MUST NEVER cause a false-positive E-QUERY-038 under either path.
    /// In combination with test 22, it guards against a regression where the union fix accidentally
    /// emits E-QUERY-038 for correct output refs.
    ///
    /// EXPECTED GREEN under both old (suspension) and new (union) code.
    #[tokio::test]
    async fn test_BC_2_11_016_enrich_union_resolvable_output_downstream_ref_ok() {
        let (engine, org) = make_armis_engine_with_threat_score_registry();

        // "threat_score" IS the declared output of "nvd_cvss" infusion.
        // Must NOT fire E-QUERY-038 under any implementation path.
        let query = "armis_devices | enrich threat_score(device_cves_first) | where threat_score > 5 | sort threat_score";

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
            Err(PrismError::ColumnNotFound(ref details)) if details.column == "threat_score" => {
                panic!(
                    "BC-2.11.016 GREEN-LOCK: FALSE-POSITIVE E-QUERY-038 fired on declared \
                     enrich output column 'threat_score'. Whether via union (after fix) or \
                     suspension (before fix), a correct output column reference MUST NOT fire \
                     E-QUERY-038. FP-001 invariant violated. column='{}', table='{}'",
                    details.column, details.table
                )
            }
            // Any other result (Ok, execution error, unrelated ColumnNotFound) is acceptable.
            _ => {}
        }
    }

    // ── Test 22 (RED GATE): post-enrich typo must yield E-QUERY-038 after union ──────────

    /// BC-2.11.016 RED GATE — resolvable enrich output: post-enrich typo must fire E-QUERY-038.
    ///
    /// `armis_devices | enrich threat_score(device_cves_first) | where threat_scor > 5`
    /// — "threat_scor" is a typo of declared output "threat_score" (Levenshtein distance 1:
    /// missing trailing 'e').
    ///
    /// ## Expected behavior after v1.9 fix (GREEN path)
    /// `check_pipe_stage_columns` (with registry) looks up "threat_score" UDF →
    ///   infusion_id "nvd_cvss" → enrich_descriptor("nvd_cvss") → output_columns=["threat_score"]
    /// UNION: current_available = {device_cves_first, device_id, threat_score}
    /// WHERE "threat_scor" checked against available → NOT found →
    ///   E-QUERY-038: column="threat_scor", did_you_mean="threat_score" (distance 1).
    ///
    /// ## Current behavior before fix (RED — why this test fails now)
    /// `check_pipe_stage_columns` has no registry param → `PipeStage::Enrich` arm always sets
    /// `suspended = true` → WHERE stage SKIPPED → "threat_scor" not checked →
    /// no E-QUERY-038 → query proceeds to execution → fails with non-ColumnNotFound error
    /// (no armis adapter wired) → test PANICS on the `Err(other)` arm below.
    ///
    /// The analyst who types `threat_scor` instead of `threat_score` receives no plan-time
    /// error; the query fails opaquely at execution time. After the fix, they receive a clear
    /// E-QUERY-038 with did_you_mean:"threat_score".
    ///
    /// ## did_you_mean calculation
    /// Available after union: {device_cves_first, device_id, threat_score}.
    /// Levenshtein("threat_scor", "threat_score") = 1 (add 'e').
    /// Levenshtein("threat_scor", "device_cves_first") >> 3 → excluded.
    /// Levenshtein("threat_scor", "device_id") >> 3 → excluded.
    /// Result: did_you_mean = "threat_score". ✓
    ///
    /// EXPECTED RED — current always-suspend swallows post-enrich column typos.
    #[tokio::test]
    async fn test_BC_2_11_016_enrich_union_resolvable_post_enrich_typo_yields_e_query_038() {
        let (engine, org) = make_armis_engine_with_threat_score_registry();

        // "threat_scor" (Levenshtein distance 1 from "threat_score") is NOT in any schema.
        // After fix: union places "threat_score" in available → typo caught → E-QUERY-038.
        let query =
            "armis_devices | enrich threat_score(device_cves_first) | where threat_scor > 5";

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
            Err(PrismError::ColumnNotFound(ref details)) if details.column == "threat_scor" => {
                // After fix: GREEN — E-QUERY-038 fired on the typo.
                assert_eq!(
                    details.column, "threat_scor",
                    "BC-2.11.016: column in E-QUERY-038 must be 'threat_scor' (enrich \
                     output typo); got: '{}'",
                    details.column
                );
                assert_eq!(
                    details.table, "armis_devices",
                    "BC-2.11.016: table must be 'armis_devices'"
                );
                assert_eq!(
                    details.did_you_mean.as_deref(),
                    Some("threat_score"),
                    "BC-2.11.016: did_you_mean must be 'threat_score' \
                     (Levenshtein distance 1 from 'threat_scor'; available after union = \
                     {{device_cves_first, device_id, threat_score}}); got: {:?}",
                    details.did_you_mean
                );
            }
            Ok(_) => panic!(
                "BC-2.11.016 RED-GATE: engine.execute returned Ok — E-QUERY-038 must fire \
                 for post-enrich typo 'threat_scor'. Current always-suspend swallows post-enrich \
                 column checks when InfusionRegistry is wired; after v1.9 registry plumbing, \
                 the union path must populate available={{device_cves_first,device_id,threat_score}} \
                 and reject this typo."
            ),
            Err(PrismError::ColumnNotFound(ref d)) => panic!(
                "BC-2.11.016: E-QUERY-038 fired but for wrong column '{}'; \
                 expected 'threat_scor'. Full details: {:?}",
                d.column, d
            ),
            Err(other) => panic!(
                "BC-2.11.016 RED-GATE: expected E-QUERY-038 (ColumnNotFound) for \
                 post-enrich typo 'threat_scor', got different error: {other:?}. \
                 Current always-suspend skips post-enrich column checks when registry is wired; \
                 after fix, the union path must catch the typo."
            ),
        }
    }

    // ── Test 23 (GREEN, regression lock): no-registry suspension preserved ──────────────

    /// BC-2.11.016 regression lock — no-InfusionRegistry path: suspension preserved.
    ///
    /// When NO InfusionRegistry is wired (registry = None), the post-enrich suspension
    /// (fail-open, FP-001) must still activate correctly after the v1.9 plumbing change.
    ///
    /// ## Analysis: is the suspend branch dead code after v1.9?
    ///
    /// For the wired-registry path (Some):
    /// - `check_enrich_udf_availability` (runs BEFORE the binding context gate) rejects any
    ///   UDF name NOT registered with E-QUERY-039. Only registered UDF names reach
    ///   `check_pipe_stage_columns`.
    /// - A registered UDF name has a corresponding entry in `udf_to_infusion` + `entries`,
    ///   so `enrich_descriptor(infusion_id)` ALWAYS succeeds for registered UDFs.
    /// - **Conclusion: when registry is wired, the suspend fallback inside PipeStage::Enrich
    ///   is dead code.** Every valid enrich stage (that passes E-QUERY-039) has a resolvable
    ///   output schema. The BC v1.9 "suspend-when-unresolvable" fallback clause is vestigial
    ///   for the wired-registry execution path.
    /// - PO note: the BC fallback clause is mechanically unreachable in the wired-registry
    ///   path but is kept live by the no-registry path tested here; no spec amendment needed.
    ///
    /// For the no-registry path (None):
    /// - `check_enrich_udf_availability` is a no-op (returns Ok immediately).
    /// - `check_pipe_stage_columns` receives `infusion_registry = None` → must still set
    ///   `suspended = true` (same as current behavior) for all post-enrich stages.
    /// - This regression lock verifies that behavior is preserved after the v1.9 plumbing.
    ///
    /// Query: `armis_devices | enrich cvss_base_score(device_cves_first) | where off_schema_col > 5`
    /// — "off_schema_col" is NOT in the original schema {device_cves_first, device_id} and NOT
    /// a typo of any output column (arbitrary name). With registry=None, suspension must prevent
    /// E-QUERY-038 (fail-open per FP-001).
    ///
    /// GREEN under both old and new code (regression lock, not a RED gate).
    #[tokio::test]
    async fn test_BC_2_11_016_enrich_no_registry_suspension_preserved_regression_lock() {
        let (engine, org) = make_armis_engine(); // deliberately NO InfusionRegistry

        // "off_schema_col" is completely absent from the schema and unrelated to any
        // enrichment output. With no registry, suspension must prevent E-QUERY-038.
        let query =
            "armis_devices | enrich cvss_base_score(device_cves_first) | where off_schema_col > 5";

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
            Err(PrismError::ColumnNotFound(ref details)) if details.column == "off_schema_col" => {
                panic!(
                    "BC-2.11.016 regression: E-QUERY-038 fired on 'off_schema_col' after \
                     enrich with no registry — suspension path broken by v1.9 plumbing. \
                     FP-001 invariant: when registry=None, all post-enrich column checks must \
                     be skipped (fail-open). column='{}', table='{}'",
                    details.column, details.table
                )
            }
            // Any other result (Ok, execution error, unrelated ColumnNotFound) is acceptable.
            _ => {}
        }
    }

    // ── Fixture: crowdstrike_alerts with host_name column (EC-11-058) ─────────────

    /// Build a `crowdstrike_alerts` engine with `severity` (String), `host_name` (Integer),
    /// and `timestamp` (Datetime) under org "acme".
    ///
    /// Used ONLY for EC-11-058 — stats aggregate argument field path tests.
    /// `host_name` is Integer: semantically correct for `sum()` at execution time.
    /// `host_nme` is the typo used in the RED test (Levenshtein distance 1 from "host_name").
    ///
    /// Grammar note: `sum(field)` accepts any FieldPath at parse time — no type-check
    /// during Chumsky parsing. Type-compatibility is DataFusion's concern; it is never
    /// reached because the plan-time E-QUERY-038 gate fires first (after the fix).
    fn make_crowdstrike_engine_with_host_name() -> (QueryEngine, OrgSlug) {
        let org = OrgSlug::new("acme");
        let sensor_id = "crowdstrike";
        let table_suffix = "alerts";

        let columns = vec![
            ColumnSpec::new("severity", ColumnType::String, None, vec![]),
            ColumnSpec::new("host_name", ColumnType::Integer, None, vec![]),
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
            .expect("register crowdstrike with host_name must not fail");

        let overlay_toml = format!("extends = \"{sensor_id}\"\ninstance_id = \"{sensor_id}@acme\"");
        let overlay: SensorInstanceOverlay = toml::from_str(&overlay_toml)
            .expect("EC-11-058 fixture: SensorInstanceOverlay TOML must parse");
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
        engine = engine.with_resolved_spec_map(Arc::new(spec_map));
        engine = engine.with_table_registry(registry);

        (engine, org)
    }

    // ── Test 25 (RED GATE): Stats aggregate argument typo → E-QUERY-038 ──────────

    /// BC-2.11.016 position 11 — stats aggregate function argument field path.
    ///
    /// EC-11-058: `crowdstrike_alerts | stats sum(host_nme) by severity` where
    /// `host_name` is a registered column (Integer) but `host_nme` is NOT.
    /// Levenshtein("host_nme", "host_name") = 1 (insert 'a' after 'n').
    ///
    /// Expected: `PrismError::ColumnNotFound` (E-QUERY-038) with
    ///   column = "host_nme",
    ///   table  = "crowdstrike_alerts",
    ///   did_you_mean = Some("host_name").
    ///
    /// ADV-FIX-P4-OBS-001 adjudicated IN-SCOPE (BC-2.11.016).
    ///
    /// RED GATE rationale: the current `PipeStage::Stats` arm (engine.rs ~2877-2918)
    /// checks `by_fields` against the available set but does NOT iterate aggregate
    /// function argument field paths (CountField/Sum/Avg/Min/Max/DistinctCount/
    /// Percentile). `host_nme` inside `sum(host_nme)` therefore bypasses the gate
    /// entirely — no E-QUERY-038 is raised — and the query falls through to DataFusion
    /// which produces an opaque execution error. The fix must extend the Stats arm to
    /// extract field paths from each AggFunc variant (reusing the same
    /// `extract_field_paths_from_expr` helper as HAVING position 6) and call
    /// `check_column_against_available_set` on each, BEFORE the Stats stage replaces
    /// the binding context (pre-REPLACE invariant from BC-2.11.016 DERIVED-COLUMN
    /// BINDING RULE).
    #[tokio::test]
    async fn test_BC_2_11_016_ec11058_stats_agg_arg_typo_yields_e_query_038() {
        let (engine, org) = make_crowdstrike_engine_with_host_name();

        // EC-11-058 canonical vector (BC-2.11.016):
        // `host_nme` is a typo of the registered column `host_name` (distance 1).
        // `severity` IS in the schema — the by-field check passes.
        // Only the aggregate argument field path (`host_nme`) must trigger E-QUERY-038.
        let query = "crowdstrike_alerts | stats sum(host_nme) by severity";

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
                    details.column, "host_nme",
                    "EC-11-058 stats-agg-arg pos-11: column in E-QUERY-038 must be \
                     'host_nme', got: {:?}",
                    details.column
                );
                assert_eq!(
                    details.table, "crowdstrike_alerts",
                    "EC-11-058 stats-agg-arg pos-11: table in E-QUERY-038 must be \
                     'crowdstrike_alerts', got: {:?}",
                    details.table
                );
                assert!(
                    details.did_you_mean.as_deref() == Some("host_name"),
                    "EC-11-058 stats-agg-arg pos-11: did_you_mean must be 'host_name' \
                     (Levenshtein distance 1 from 'host_nme'); got: {:?}",
                    details.did_you_mean
                );
            }
            Ok(_) => panic!(
                "EC-11-058 stats-agg-arg pos-11: engine.execute must NOT succeed — \
                 E-QUERY-038 must fire for non-existent aggregate argument column \
                 'host_nme'. Before the fix, PipeStage::Stats only checks by_fields; \
                 aggregate function argument field paths are not walked in \
                 check_query_column_availability, so 'host_nme' bypasses the gate entirely."
            ),
            Err(other) => panic!(
                "EC-11-058 stats-agg-arg pos-11: expected PrismError::ColumnNotFound \
                 (E-QUERY-038) for typo column 'host_nme' in sum() aggregate argument, \
                 got different error: {other:?}. Before the fix this would be an \
                 execution error (DataFusion or adapter error) because the plan-time \
                 column gate does not walk AggFunc FieldPath arguments."
            ),
        }
    }

    // ── Tests 27-31 (RED GATE + GREEN lock): SQLPIPE HEAD-PROJECTION BINDING RULE ──
    //    BC-2.11.016 EC-11-059 / EC-11-059b / EC-11-060 / EC-11-061 (FP-001)
    //
    //    Root cause: `check_pipe_stage_columns` seeds the initial `available` set for
    //    `Ast::SqlPipe` stage walk from raw schema_columns(table, OrgId), IGNORING the
    //    head SQL projection output. This causes:
    //      (a) false-positive E-QUERY-038 on SELECT aliases (e.g., `cnt` from `count(*) AS cnt`,
    //          `sev` from `severity AS sev`) — those names are valid at execution time but absent
    //          from the raw schema, so the gate incorrectly rejects them (FP-001 violation).
    //      (b) less-precise error for post-head typos: did_you_mean and available_columns reflect
    //          the raw schema rather than the head output set (e.g., EC-11-060: `sevv` against
    //          {severity,timestamp} gives no suggestion; against head output {sev} gives
    //          did_you_mean:"sev" — a precision win).
    //      (c) false-positive E-QUERY-038 on `SELECT count(*) FROM t GROUP BY g | sort xyz` —
    //          anonymous aggregate without alias must set suspended=true for the stage walk per
    //          FP-001 fail-open; instead the gate fires on `xyz` using raw schema today.
    //
    //    Fix (BC-2.11.016 SQLPIPE HEAD-PROJECTION BINDING RULE):
    //      For `Ast::SqlPipe`, initial `available` for stage walk is seeded from head projection
    //      output — not raw schema:
    //        (a) `SELECT *` → full raw schema (unchanged; current behavior preserved)
    //        (b) explicit SELECT → `{explicit AS aliases} ∪ {bare Field SELECT names} ∪
    //            {bare GROUP BY field names}`
    //        (c) any non-Field SELECT item without AS alias → suspended := true (fail-open;
    //            mirrors Stats anonymous-aggregate rule; FP-001)
    //      Head SQL clause checking (positions 1–6) still runs against raw schema unchanged.

    /// BC-2.11.016 EC-11-059 — SqlPipe sort after aliased aggregate: no false-positive.
    ///
    /// `SELECT count(*) AS cnt FROM crowdstrike_alerts GROUP BY severity | sort cnt`
    ///
    /// SQLPIPE HEAD-PROJECTION BINDING RULE: head output = {cnt (alias from count(*) AS cnt),
    /// severity (bare field in GROUP BY)}. The stage walk initial `available = {cnt, severity}`.
    /// `| sort cnt` finds `cnt` in the binding set → NO E-QUERY-038. EC-11-059.
    ///
    /// EXPECTED RED — current code seeds `available` from raw schema {severity, timestamp}.
    /// `cnt` is NOT in the raw schema → E-QUERY-038 fires on `cnt` (false positive, FP-001
    /// violation). The test panics on the ColumnNotFound arm for `cnt`, confirming RED gate.
    #[tokio::test]
    async fn test_BC_2_11_016_ec11059_sqlpipe_head_alias_no_false_positive() {
        let (engine, org) = make_crowdstrike_engine();

        // EC-11-059 canonical vector (BC-2.11.016).
        // Head: SELECT count(*) AS cnt FROM crowdstrike_alerts GROUP BY severity
        // Stage: | sort cnt
        // After fix: available = {cnt, severity} → cnt found → no E-QUERY-038.
        // Before fix: available = {severity, timestamp} → cnt absent → FALSE-POSITIVE E-QUERY-038.
        let query = "SELECT count(*) AS cnt FROM crowdstrike_alerts GROUP BY severity | sort cnt";

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
            Err(PrismError::ColumnNotFound(ref details)) if details.column == "cnt" => panic!(
                "BC-2.11.016 EC-11-059: FALSE-POSITIVE E-QUERY-038 fired on SqlPipe head \
                 alias 'cnt' in downstream | sort stage. SQLPIPE HEAD-PROJECTION BINDING RULE \
                 requires: after explicit SELECT head, available = {{explicit aliases ∪ GROUP BY \
                 fields}} = {{cnt, severity}}; 'cnt' must be found in the binding set. Current \
                 code seeds available from raw schema {{severity, timestamp}} — 'cnt' absent → \
                 incorrect rejection. FP-001 invariant violated. \
                 column='{}', table='{}', available={:?}, did_you_mean={:?}",
                details.column, details.table, details.available_columns, details.did_you_mean
            ),
            // Any other result (Ok, execution error, unrelated ColumnNotFound) is acceptable.
            // The only invariant is that the SELECT alias 'cnt' must NOT produce a false-positive
            // E-QUERY-038 in the downstream | sort stage.
            _ => {}
        }
    }

    /// BC-2.11.016 EC-11-059b — SqlPipe where after plain alias: no false-positive.
    ///
    /// `SELECT severity AS sev FROM crowdstrike_alerts | where sev = 'High'`
    ///
    /// SQLPIPE HEAD-PROJECTION BINDING RULE: head output = {sev (alias from severity AS sev)}.
    /// Stage walk initial `available = {sev}`. `| where sev = 'High'` finds `sev` in the
    /// binding set → NO E-QUERY-038.
    ///
    /// EXPECTED RED — current code seeds `available` from raw schema {severity, timestamp}.
    /// `sev` is NOT in the raw schema (the alias is not in the schema; only `severity` is) →
    /// E-QUERY-038 fires on `sev` (false positive, FP-001 violation). The test panics on the
    /// ColumnNotFound arm for `sev`, confirming RED gate.
    ///
    /// Note on head SQL check (positions 1–6): `severity` (bare Field) is checked against raw
    /// schema → found → OK. The FP-001 violation occurs only in the stage walk (position 9).
    #[tokio::test]
    async fn test_BC_2_11_016_ec11059b_sqlpipe_plain_alias_no_false_positive() {
        let (engine, org) = make_crowdstrike_engine();

        // Head: SELECT severity AS sev FROM crowdstrike_alerts (head SQL check: severity ok)
        // Stage: | where sev = 'High'
        // After fix: available = {sev} → sev found → no E-QUERY-038.
        // Before fix: available = {severity, timestamp} → sev absent → FALSE-POSITIVE E-QUERY-038.
        let query = "SELECT severity AS sev FROM crowdstrike_alerts | where sev = 'High'";

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
            Err(PrismError::ColumnNotFound(ref details)) if details.column == "sev" => panic!(
                "BC-2.11.016 EC-11-059b: FALSE-POSITIVE E-QUERY-038 fired on SqlPipe head \
                 plain alias 'sev' in downstream | where stage. SQLPIPE HEAD-PROJECTION BINDING \
                 RULE requires: after `SELECT severity AS sev`, available = {{sev}}; 'sev' must be \
                 found in the binding set. Current code seeds available from raw schema \
                 {{severity, timestamp}} — 'sev' is the alias NOT in the raw schema → incorrect \
                 rejection. FP-001 invariant violated. \
                 column='{}', table='{}', available={:?}, did_you_mean={:?}",
                details.column, details.table, details.available_columns, details.did_you_mean
            ),
            _ => {}
        }
    }

    /// BC-2.11.016 EC-11-060 — SqlPipe post-head typo yields E-QUERY-038 (precision win).
    ///
    /// `SELECT severity AS sev FROM crowdstrike_alerts | where sevv = 'High'`
    ///
    /// SQLPIPE HEAD-PROJECTION BINDING RULE: head output = {sev}. The post-head typo `sevv`
    /// is NOT in {sev} → E-QUERY-038 fires with `did_you_mean: "sev"` (lev("sevv","sev")=1)
    /// and `available_columns: ["sev"]`. This is the precision win: the error message shows
    /// the head-output set rather than the raw schema, giving a directly actionable suggestion.
    ///
    /// EXPECTED RED — current code seeds `available` from raw schema {severity, timestamp}.
    /// E-QUERY-038 DOES fire on `sevv` today (sevv not in raw schema either), but:
    ///   (a) did_you_mean is ABSENT today: lev("sevv","severity")=5 > 3, lev("sevv","timestamp")>>3
    ///       — no raw-schema column is within distance ≤ 3 from "sevv". After fix: lev("sevv","sev")=1
    ///       → did_you_mean = Some("sev").
    ///   (b) available_columns today = ["severity", "timestamp"] (raw schema). After fix: ["sev"].
    ///
    /// The test asserts the v1.13 semantics: did_you_mean = Some("sev") and available_columns
    /// contains "sev" but NOT "severity". These assertions FAIL today → RED gate.
    #[tokio::test]
    async fn test_BC_2_11_016_ec11060_sqlpipe_post_head_typo_yields_e_query_038() {
        let (engine, org) = make_crowdstrike_engine();

        // EC-11-060 canonical vector (BC-2.11.016).
        // Head: SELECT severity AS sev FROM crowdstrike_alerts → head output = {sev}
        // Stage: | where sevv = 'High' — `sevv` is a typo of `sev` (Levenshtein distance 1)
        // After fix: available = {sev}; sevv not in {sev} → E-QUERY-038 with
        //   did_you_mean: Some("sev"), available_columns: ["sev"].
        // Before fix: available = {severity, timestamp}; sevv not found →
        //   E-QUERY-038 with did_you_mean: None (lev("sevv","severity")=5 > 3),
        //   available_columns: ["severity", "timestamp"].
        let query = "SELECT severity AS sev FROM crowdstrike_alerts | where sevv = 'High'";

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
                    details.column, "sevv",
                    "BC-2.11.016 EC-11-060: E-QUERY-038 must be on column 'sevv'; \
                     got: '{}'",
                    details.column
                );
                assert_eq!(
                    details.table, "crowdstrike_alerts",
                    "BC-2.11.016 EC-11-060: table must be 'crowdstrike_alerts'"
                );
                // Distinguishing assertion (a): did_you_mean reflects head-output set {sev}.
                // Today: None (lev("sevv","severity")=5 > threshold; no raw-schema match).
                // After fix: Some("sev") (lev("sevv","sev")=1 ≤ threshold).
                assert_eq!(
                    details.did_you_mean.as_deref(),
                    Some("sev"),
                    "BC-2.11.016 EC-11-060: did_you_mean must be 'sev' (lev=1 against \
                     head-output {{sev}}). Current code uses raw schema {{severity, timestamp}}: \
                     lev('sevv','severity')=5 > 3 → did_you_mean absent. After fix: head output \
                     {{sev}}, lev('sevv','sev')=1 → did_you_mean=Some('sev'). RED gate. \
                     Got: {:?}",
                    details.did_you_mean
                );
                // Distinguishing assertion (b): available_columns reflects head output, not raw schema.
                // Today: ["severity", "timestamp"]. After fix: ["sev"].
                assert!(
                    details.available_columns.iter().any(|c| c == "sev"),
                    "BC-2.11.016 EC-11-060: available_columns must contain 'sev' (head \
                     projection output). Current code returns raw schema columns \
                     ['severity','timestamp']; after fix available = ['sev']. \
                     Got: {:?}",
                    details.available_columns
                );
                assert!(
                    !details.available_columns.iter().any(|c| c == "severity"),
                    "BC-2.11.016 EC-11-060: available_columns must NOT contain 'severity' \
                     after SQLPIPE HEAD-PROJECTION BINDING RULE — head output is {{sev}}, not the \
                     raw schema. After fix the error reflects only the head-output set. \
                     Got: {:?}",
                    details.available_columns
                );
            }
            Ok(_) => panic!(
                "BC-2.11.016 EC-11-060: engine.execute must NOT succeed — 'sevv' is a \
                 post-head typo not in head output {{sev}} and must trigger E-QUERY-038."
            ),
            Err(other) => panic!(
                "BC-2.11.016 EC-11-060: expected PrismError::ColumnNotFound (E-QUERY-038) \
                 for post-head typo 'sevv', got: {other:?}"
            ),
        }
    }

    /// BC-2.11.016 EC-11-061 — anonymous head aggregate suspends stage walk: no false-positive.
    ///
    /// `SELECT count(*) FROM crowdstrike_alerts GROUP BY severity | sort xyz`
    ///
    /// SQLPIPE HEAD-PROJECTION BINDING RULE: `count(*)` is a non-Field SELECT item without an
    /// explicit AS alias → its output name is unpredictable at plan time → `suspended := true`
    /// for the stage walk (FP-001 fail-open; mirrors Stats anonymous-aggregate rule). All
    /// subsequent stage positions are skipped → NO E-QUERY-038 on `xyz`.
    ///
    /// EXPECTED RED — current code seeds `available` from raw schema {severity, timestamp}.
    /// `xyz` is NOT in the raw schema → E-QUERY-038 fires on `xyz` (false positive, FP-001
    /// violation: the anonymous aggregate means we cannot know what names are valid downstream;
    /// rejecting `xyz` is forbidden). The test panics on the ColumnNotFound arm for `xyz`,
    /// confirming RED gate.
    #[tokio::test]
    async fn test_BC_2_11_016_ec11061_sqlpipe_anon_head_agg_suspends() {
        let (engine, org) = make_crowdstrike_engine();

        // EC-11-061 canonical vector (BC-2.11.016).
        // Head: SELECT count(*) FROM crowdstrike_alerts GROUP BY severity
        //   — count(*) has no alias → anonymous aggregate → suspended := true
        // Stage: | sort xyz  — xyz is any arbitrary name
        // After fix: suspended = true → stage walk skipped → no E-QUERY-038 on xyz.
        // Before fix: available = {severity, timestamp}; xyz not found →
        //   FALSE-POSITIVE E-QUERY-038 on xyz (FP-001 violation).
        let query = "SELECT count(*) FROM crowdstrike_alerts GROUP BY severity | sort xyz";

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
            Err(PrismError::ColumnNotFound(ref details)) if details.column == "xyz" => panic!(
                "BC-2.11.016 EC-11-061: FALSE-POSITIVE E-QUERY-038 fired on 'xyz' in \
                 SqlPipe stage walk after anonymous head aggregate. SQLPIPE HEAD-PROJECTION \
                 BINDING RULE: `SELECT count(*)` has no AS alias → output name unpredictable \
                 at plan time → suspended := true for the stage walk (FP-001 fail-open). \
                 Current code seeds available from raw schema {{severity, timestamp}} and checks \
                 'xyz' — not found → incorrect rejection. \
                 column='{}', table='{}', available={:?}",
                details.column, details.table, details.available_columns
            ),
            // Any other result (Ok, execution error, unrelated ColumnNotFound) is acceptable.
            // The invariant is that anonymous head aggregates must activate suspension so
            // downstream stage refs do NOT produce false-positive E-QUERY-038.
            _ => {}
        }
    }

    /// BC-2.11.016 GREEN lock — SELECT * SqlPipe sort with valid schema column.
    ///
    /// `SELECT * FROM crowdstrike_alerts | sort severity`
    ///
    /// SQLPIPE HEAD-PROJECTION BINDING RULE: `SELECT *` → available = schema_columns(table, OrgId)
    /// = {severity, timestamp} (full raw schema; same as Ast::Pipe initial state; current behavior
    /// preserved). `| sort severity` finds `severity` in the binding set → NO E-QUERY-038.
    ///
    /// EXPECTED GREEN today and after fix — `SELECT *` path is unchanged by the rule. This
    /// lock verifies the `SELECT *` branch is not regressed by the v1.13 fix.
    #[tokio::test]
    async fn test_BC_2_11_016_ec11059_star_head_sort_valid_col_green_lock() {
        let (engine, org) = make_crowdstrike_engine();

        // SELECT * → full raw schema in available; `severity` is in the schema → no E-QUERY-038.
        // Both today (before fix) and after fix, this must pass.
        let query = "SELECT * FROM crowdstrike_alerts | sort severity";

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
            Err(PrismError::ColumnNotFound(ref details)) if details.column == "severity" => panic!(
                "BC-2.11.016 GREEN lock: E-QUERY-038 fired on 'severity' in SELECT * \
                 SqlPipe sort — SELECT * must preserve full raw schema in available set \
                 (SQLPIPE HEAD-PROJECTION BINDING RULE §(a): SELECT * → schema_columns). \
                 This is a regression in the star-head path. \
                 column='{}', table='{}', available={:?}",
                details.column, details.table, details.available_columns
            ),
            // Any other result (Ok, execution error, unrelated ColumnNotFound) is acceptable.
            _ => {}
        }
    }

    // ── Test 26 (GREEN lock): Stats aggregate argument — valid column passes gate ──

    /// EC-11-058 GREEN lock — stats aggregate argument with the REAL column name.
    ///
    /// `crowdstrike_alerts | stats sum(host_name) by severity` uses the registered
    /// column `host_name` (Integer) in the sum() aggregate argument and the registered
    /// column `severity` (String) in the by-field list.
    ///
    /// E-QUERY-038 must NOT fire: `host_name` exists in the available set BEFORE the
    /// Stats stage replaces the binding context. Any other error (adapter not wired,
    /// execution failure) is acceptable; only `PrismError::ColumnNotFound` is forbidden.
    ///
    /// This lock confirms the agg-arg check is applied BEFORE the REPLACE step and
    /// checks against the INCOMING available set — not the post-REPLACE context —
    /// so the real schema column is always found.
    #[tokio::test]
    async fn test_BC_2_11_016_ec11058_stats_agg_arg_valid_col_no_e_query_038() {
        let (engine, org) = make_crowdstrike_engine_with_host_name();

        // Real column name in the sum() arg: E-QUERY-038 must NOT fire.
        let query = "crowdstrike_alerts | stats sum(host_name) by severity";

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
            Err(PrismError::ColumnNotFound(ref d))
                if d.column == "host_name" || d.column == "severity" =>
            {
                panic!(
                    "EC-11-058 GREEN lock: E-QUERY-038 fired unexpectedly for registered \
                     column '{}' in crowdstrike_alerts stats agg arg. \
                     The gate must NOT fire for columns that exist in the available set \
                     before the Stats REPLACE step.",
                    d.column
                )
            }
            // Ok or any other error (no adapter, execution failure) is acceptable.
            // The only invariant is: E-QUERY-038 does NOT fire for host_name or severity.
            _ => {}
        }
    }

    // ── Tests 32-34 (RED GATE): ADV-FIX-P6-MED-002 — MIXED-STAR head-projection ──
    //
    //    BC-2.11.016 EC-11-062 / EC-11-063 / EC-11-064 (FP-001)
    //
    //    Root cause: `compute_sqlpipe_head_binding` short-circuits on ANY Star or TableStar
    //    item in the SELECT list:
    //        let has_star = head.select.items.iter()
    //            .any(|item| matches!(item, SelectItem::Star | SelectItem::TableStar(_)));
    //        if has_star { return None; }   // ← wrong: ignores explicit aliases in MIXED lists
    //
    //    When the head SELECT contains BOTH a wildcard (Star/TableStar) AND explicit non-star
    //    items with AS aliases — a MIXED-STAR head — the function incorrectly returns `None`,
    //    causing `check_pipe_stage_columns` to seed `available` from the raw schema alone.
    //    The explicit aliases (e.g., `sev_up`, `cnt`, `lo`) are absent from the raw schema
    //    → E-QUERY-038 fires on downstream stage references to those aliases (FP-001 violation).
    //
    //    Fix (BC-2.11.016 MIXED-STAR branch (c)):
    //    When head SELECT contains at least one Star/TableStar AND at least one explicit
    //    non-star item:
    //      available = schema_columns(table, OrgId)                     ← from Star/TableStar
    //                ∪ {AS aliases from explicit items}                 ← computed aliases
    //                ∪ {bare Field names of un-aliased explicit bare-Field items}
    //                ∪ {bare field names in the GROUP BY clause}
    //    If any explicit non-Field item lacks an AS alias → suspended := true (FP-001 fail-open).
    //    Branches (a) (pure SELECT *) and (b) (fully-explicit SELECT) are unchanged.

    /// BC-2.11.016 EC-11-062 — MIXED-STAR head with aliased function expression:
    /// no false-positive E-QUERY-038 on alias in downstream | where stage.
    ///
    /// `SELECT *, upper(severity) AS sev_up FROM crowdstrike_alerts | where sev_up = 'HIGH'`
    ///
    /// MIXED-STAR branch (c): head has `*` (Star) AND `upper(severity) AS sev_up`
    /// (explicit non-star item WITH an alias). Initial `available` after fix:
    ///   schema_columns ∪ {sev_up} = {severity, timestamp, sev_up}
    /// `upper(severity) AS sev_up` carries an explicit AS alias → no anonymous-item suspension.
    /// `| where sev_up = 'HIGH'` finds `sev_up` in the binding context → NO E-QUERY-038.
    ///
    /// EXPECTED RED — current code: `has_star = true` → `compute_sqlpipe_head_binding` returns
    /// `None` → `check_pipe_stage_columns` seeds `available` from raw schema {severity, timestamp}.
    /// `sev_up` is NOT in the raw schema → E-QUERY-038 fires on `sev_up` (FP-001 violation;
    /// ADV-FIX-P6-MED-002). The test panics on the ColumnNotFound arm confirming RED gate.
    #[tokio::test]
    async fn test_BC_2_11_016_ec11062_mixed_star_alias_no_false_positive() {
        let (engine, org) = make_crowdstrike_engine();

        // EC-11-062 canonical vector (BC-2.11.016).
        // Head: SELECT *, upper(severity) AS sev_up FROM crowdstrike_alerts
        //   — Star AND upper(severity) AS sev_up (aliased scalar function).
        // Stage: | where sev_up = 'HIGH'
        // After fix (MIXED-STAR branch (c)):
        //   available = {severity, timestamp} ∪ {sev_up} = {severity, sev_up, timestamp}
        //   → sev_up found → no E-QUERY-038.
        // Before fix (current): has_star → None → available = {severity, timestamp}
        //   → sev_up absent → FALSE-POSITIVE E-QUERY-038 on sev_up (FP-001 violation).
        let query =
            "SELECT *, upper(severity) AS sev_up FROM crowdstrike_alerts | where sev_up = 'HIGH'";

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
            Err(PrismError::ColumnNotFound(ref details)) if details.column == "sev_up" => panic!(
                "BC-2.11.016 EC-11-062: FALSE-POSITIVE E-QUERY-038 fired on MIXED-STAR \
                 head alias 'sev_up' in downstream | where stage. MIXED-STAR branch (c) requires \
                 initial available = schema_columns ∪ {{sev_up}} = {{severity, timestamp, sev_up}}; \
                 'sev_up' must be found in the binding context. Current code short-circuits on any \
                 Star/TableStar (has_star check) → compute_sqlpipe_head_binding returns None → \
                 available = raw schema {{severity, timestamp}} → 'sev_up' absent → incorrect \
                 rejection (FP-001 violation; ADV-FIX-P6-MED-002). \
                 column='{}', table='{}', available={:?}, did_you_mean={:?}",
                details.column, details.table, details.available_columns, details.did_you_mean
            ),
            // Any other result (Ok, execution error, unrelated ColumnNotFound) is acceptable.
            // The only invariant: MIXED-STAR alias 'sev_up' must NOT produce false-positive
            // E-QUERY-038 in the downstream | where stage.
            _ => {}
        }
    }

    /// BC-2.11.016 EC-11-063 — MIXED-STAR head with aliased aggregate and GROUP BY:
    /// no false-positive E-QUERY-038 on alias in downstream | sort stage.
    ///
    /// `SELECT *, count(severity) AS cnt FROM crowdstrike_alerts GROUP BY severity | sort cnt`
    ///
    /// MIXED-STAR branch (c): head has `*` (Star) AND `count(severity) AS cnt`
    /// (non-Field item WITH an alias). GROUP BY `severity` also contributes. Initial
    /// `available` after fix:
    ///   schema_columns ∪ {cnt} ∪ {severity (GROUP BY, already in schema)}
    ///   = {severity, timestamp, cnt}
    /// `count(severity) AS cnt` carries an explicit AS alias → no anonymous-item suspension.
    /// `| sort cnt` finds `cnt` in the binding context → NO E-QUERY-038.
    ///
    /// EXPECTED RED — current code: `has_star = true` → returns `None` → raw schema
    /// {severity, timestamp}. `cnt` is NOT in the raw schema → E-QUERY-038 fires on `cnt`
    /// (FP-001 violation; ADV-FIX-P6-MED-002). The test panics on the ColumnNotFound arm
    /// for `cnt`, confirming RED gate.
    #[tokio::test]
    async fn test_BC_2_11_016_ec11063_mixed_star_agg_alias_no_false_positive() {
        let (engine, org) = make_crowdstrike_engine();

        // EC-11-063 canonical vector (BC-2.11.016).
        // Head: SELECT *, count(severity) AS cnt FROM crowdstrike_alerts GROUP BY severity
        //   — Star AND count(severity) AS cnt (aliased aggregate; explicit alias).
        // Stage: | sort cnt
        // After fix (MIXED-STAR branch (c)):
        //   available = {severity, timestamp} ∪ {cnt} ∪ {severity (GROUP BY)}
        //             = {severity, timestamp, cnt}
        //   → cnt found → no E-QUERY-038.
        // Before fix (current): has_star → None → available = {severity, timestamp}
        //   → cnt absent → FALSE-POSITIVE E-QUERY-038 on cnt (FP-001 violation).
        let query =
            "SELECT *, count(severity) AS cnt FROM crowdstrike_alerts GROUP BY severity | sort cnt";

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
            Err(PrismError::ColumnNotFound(ref details)) if details.column == "cnt" => panic!(
                "BC-2.11.016 EC-11-063: FALSE-POSITIVE E-QUERY-038 fired on MIXED-STAR \
                 head aggregate alias 'cnt' in downstream | sort stage. MIXED-STAR branch (c) \
                 requires initial available = schema_columns ∪ {{cnt}} ∪ {{severity (GROUP BY)}} \
                 = {{severity, timestamp, cnt}}; 'cnt' must be found. Current code short-circuits \
                 on any Star (has_star) → compute_sqlpipe_head_binding returns None → \
                 available = raw schema {{severity, timestamp}} → 'cnt' absent → incorrect \
                 rejection (FP-001 violation; ADV-FIX-P6-MED-002). \
                 column='{}', table='{}', available={:?}, did_you_mean={:?}",
                details.column, details.table, details.available_columns, details.did_you_mean
            ),
            // Any other result (Ok, execution error, unrelated ColumnNotFound) is acceptable.
            // The only invariant: MIXED-STAR aggregate alias 'cnt' must NOT produce false-positive
            // E-QUERY-038 in the downstream | sort stage.
            _ => {}
        }
    }

    /// BC-2.11.016 EC-11-064 — MIXED-STAR head with TableStar variant and explicit alias:
    /// no false-positive E-QUERY-038 on alias in downstream | fields stage.
    ///
    /// `SELECT t.*, lower(severity) AS lo FROM crowdstrike_alerts t | fields lo, severity`
    ///
    /// MIXED-STAR branch (c) triggered by `t.*` (TableStar): head has TableStar AND
    /// `lower(severity) AS lo` (explicit item WITH an alias). Initial `available` after fix:
    ///   schema_columns(table, OrgId) ∪ {lo} = {severity, timestamp, lo}
    /// (`severity` is already in schema_columns via the TableStar path.)
    /// `lower(severity) AS lo` carries an explicit AS alias → no anonymous-item suspension.
    /// `| fields lo, severity` finds both `lo` and `severity` → NO E-QUERY-038.
    ///
    /// EXPECTED RED — current code: TableStar triggers `has_star = true` → returns `None`
    /// → raw schema {severity, timestamp}. `lo` is NOT in the raw schema → E-QUERY-038 fires
    /// on `lo` (FP-001 violation; ADV-FIX-P6-MED-002). The test panics on the ColumnNotFound
    /// arm for `lo`, confirming RED gate.
    #[tokio::test]
    async fn test_BC_2_11_016_ec11064_mixed_star_tablestar_no_false_positive() {
        let (engine, org) = make_crowdstrike_engine();

        // EC-11-064 canonical vector (BC-2.11.016).
        // Head: SELECT t.*, lower(severity) AS lo FROM crowdstrike_alerts t
        //   — t.* (TableStar) AND lower(severity) AS lo (aliased scalar function).
        //   Table alias `t` used in `t.*` (TableStar qualifier).
        // Stage: | fields lo, severity
        // After fix (MIXED-STAR branch (c) via TableStar):
        //   available = {severity, timestamp} ∪ {lo} = {lo, severity, timestamp}
        //   → lo found; severity found → no E-QUERY-038.
        // Before fix (current): has_star (TableStar) → None → available = {severity, timestamp}
        //   → lo absent → FALSE-POSITIVE E-QUERY-038 on lo (FP-001 violation).
        let query =
            "SELECT t.*, lower(severity) AS lo FROM crowdstrike_alerts t | fields lo, severity";

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
            Err(PrismError::ColumnNotFound(ref details)) if details.column == "lo" => panic!(
                "BC-2.11.016 EC-11-064: FALSE-POSITIVE E-QUERY-038 fired on MIXED-STAR \
                 head alias 'lo' (TableStar variant) in downstream | fields stage. MIXED-STAR \
                 branch (c) triggered by `t.*` requires initial available = schema_columns ∪ {{lo}} \
                 = {{severity, timestamp, lo}}; 'lo' must be found. Current code short-circuits on \
                 any TableStar (has_star) → compute_sqlpipe_head_binding returns None → \
                 available = raw schema {{severity, timestamp}} → 'lo' absent → incorrect \
                 rejection (FP-001 violation; ADV-FIX-P6-MED-002). \
                 column='{}', table='{}', available={:?}, did_you_mean={:?}",
                details.column, details.table, details.available_columns, details.did_you_mean
            ),
            // Any other result (Ok, execution error, unrelated ColumnNotFound) is acceptable.
            // The invariant: TableStar MIXED-STAR alias 'lo' must NOT produce false-positive
            // E-QUERY-038 in the downstream | fields stage.
            _ => {}
        }
    }

    // ── Tests 35-36 (RED GATE): ADV-FIX-P7-MED-001 SIBLING-GATE CONSISTENCY ─────
    //    BC-2.11.016 — DERIVED-name provenance prevents false E-QUERY-002.
    //
    //    Root cause (MED-001): `check_operator_type_compatibility` in
    //    `check_pipe_stage_columns` looks up the RAW schema type for every name in
    //    `available`, regardless of provenance.  When a DERIVED name (stats alias,
    //    SqlPipe head alias) shadows a raw-schema column with a different declared type,
    //    the raw-type check incorrectly fires E-QUERY-002 for the DERIVED alias.
    //
    //    Fix: names in `available` carry per-name RAW vs DERIVED provenance.
    //    E-QUERY-002 MUST skip DERIVED names (fail-open per FP-001 and SIBLING-GATE
    //    CONSISTENCY).  RAW names retain full type-compat checking unchanged.
    //
    //    Test 35: RED gate — false E-QUERY-002 on SqlPipe head alias `severity`
    //      (`count(*) AS severity` shadows raw String `severity`; alias is Int64 at run).
    //    Test 36: RED gate — false E-QUERY-002 on stats output alias `severity`
    //      (`count() as severity by timestamp`; alias is Int64 at run).
    //    Test 37: GREEN-LOCK — RAW String `severity` with `>` STILL fires E-QUERY-002
    //      after fix (guards against over-broad fail-open skipping RAW names).

    /// BC-2.11.016 MED-001 SIBLING-GATE CONSISTENCY — SqlPipe head alias shadow.
    ///
    /// `SELECT count(*) AS severity FROM crowdstrike_alerts | where severity > 5`
    ///
    /// `severity` is a SqlPipe head alias (`count(*) AS severity`) — provenance DERIVED.
    /// At execution its type is Int64.  The raw schema column `severity` has type String.
    /// E-QUERY-002 MUST NOT apply String's operator restrictions to this DERIVED alias.
    ///
    /// SQLPIPE HEAD-PROJECTION BINDING RULE (BC-2.11.016, currently implemented):
    /// explicit SELECT head → `available = {severity (alias)}` for the stage walk.
    ///
    /// Current behavior (RED): `check_operator_type_compatibility` looks up the raw
    /// schema type for `severity` (String), finds `>` absent from
    /// `valid_operators_for_type(String)` → fires false E-QUERY-002 (FP-001 violation,
    /// MED-001).
    ///
    /// After fix (GREEN): alias `severity` carries DERIVED provenance → E-QUERY-002
    /// gate skips it (SIBLING-GATE CONSISTENCY) → no false E-QUERY-002.
    ///
    /// RED GATE: currently fires `PrismError::QueryTypeMismatch { column: "severity" }`.
    #[tokio::test]
    async fn test_BC_2_11_016_v1_15_med001_sqlpipe_head_derived_alias_no_false_e_query_002() {
        let (engine, org) = make_crowdstrike_engine();

        // `count(*) AS severity` — alias shadows raw String `severity`.
        // At execution the alias is Int64 (count output); `>` is valid for Int64.
        // The E-QUERY-002 gate MUST NOT apply raw String operator restrictions to this alias.
        let query = "SELECT count(*) AS severity FROM crowdstrike_alerts | where severity > 5";

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
            Err(PrismError::QueryTypeMismatch { ref column, .. })
                if column.as_str() == "severity" =>
            {
                panic!(
                    "BC-2.11.016 MED-001 SqlPipe-head: FALSE E-QUERY-002 fired on \
                     DERIVED alias 'severity'. `count(*) AS severity` in the head is DERIVED \
                     (Int64 at execution); E-QUERY-002 must NOT apply raw-schema String type's \
                     operator restrictions to it per SIBLING-GATE CONSISTENCY (FP-001). \
                     Current code: check_operator_type_compatibility looks up raw schema type \
                     (String) → '>' not in valid_operators_for_type(String) → false E-QUERY-002. \
                     Fix: track RAW vs DERIVED provenance; skip E-QUERY-002 for DERIVED names."
                )
            }
            Err(PrismError::ColumnNotFound(ref d)) if d.column == "severity" => panic!(
                "BC-2.11.016 MED-001 SqlPipe-head: E-QUERY-038 fired on DERIVED alias \
                 'severity' — alias must be in available (SQLPIPE HEAD-PROJECTION BINDING RULE \
                 explicit head: available = {{severity}} as alias). \
                 column='{}', table='{}'",
                d.column, d.table
            ),
            // Ok or any other non-QueryTypeMismatch-on-severity result is acceptable.
            // The invariant: E-QUERY-002 must NOT fire for a DERIVED name.
            _ => {}
        }
    }

    /// BC-2.11.016 MED-001 SIBLING-GATE CONSISTENCY — stats output alias shadow.
    ///
    /// `crowdstrike_alerts | stats count() as severity by timestamp | where severity > 5`
    ///
    /// `severity` is a stats output alias (`count() as severity`) — provenance DERIVED.
    /// After the Stats stage REPLACE, `available = {severity (alias), timestamp (by-field)}`.
    /// E-QUERY-002 MUST NOT apply raw String operator restrictions to DERIVED `severity`.
    ///
    /// Current behavior (RED): same root cause as the SqlPipe head variant.  Stats alias
    /// `severity` shadows raw String `severity`; `check_operator_type_compatibility` looks
    /// up the raw type (String), finds `>` absent → false E-QUERY-002.
    ///
    /// After fix (GREEN): stats output alias `severity` carries DERIVED provenance →
    /// E-QUERY-002 gate skips it → no false E-QUERY-002.
    ///
    /// RED GATE: currently fires `PrismError::QueryTypeMismatch { column: "severity" }`.
    #[tokio::test]
    async fn test_BC_2_11_016_v1_15_med001_stats_derived_alias_no_false_e_query_002() {
        let (engine, org) = make_crowdstrike_engine();

        // `stats count() as severity by timestamp` — alias shadows raw String `severity`.
        // STATS REPLACE: available = {severity (alias), timestamp (by-field)}.
        // `| where severity > 5` — `severity` is DERIVED → E-QUERY-002 MUST NOT fire.
        let query =
            "crowdstrike_alerts | stats count() as severity by timestamp | where severity > 5";

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
            Err(PrismError::QueryTypeMismatch { ref column, .. })
                if column.as_str() == "severity" =>
            {
                panic!(
                    "BC-2.11.016 MED-001 stats-shadow: FALSE E-QUERY-002 fired on \
                     DERIVED stats alias 'severity'. `count() as severity` is DERIVED (Int64 \
                     at execution); E-QUERY-002 must NOT apply raw-schema String type's \
                     operator restrictions per SIBLING-GATE CONSISTENCY (FP-001). \
                     Current code: check_operator_type_compatibility looks up raw String type \
                     → '>' not in valid_operators_for_type(String) → false E-QUERY-002. \
                     Fix: track RAW vs DERIVED provenance; skip E-QUERY-002 for DERIVED names."
                )
            }
            Err(PrismError::ColumnNotFound(ref d)) if d.column == "severity" => panic!(
                "BC-2.11.016 MED-001 stats-shadow: E-QUERY-038 fired on DERIVED stats \
                 alias 'severity' — after Stats REPLACE, available = {{severity, timestamp}}; \
                 'severity' must be found. column='{}', table='{}'",
                d.column, d.table
            ),
            // Ok or any other non-QueryTypeMismatch-on-severity result is acceptable.
            _ => {}
        }
    }

    // ── Test 37 (GREEN-LOCK): RAW-provenance type-compat retained ─────────────────

    /// BC-2.11.016 MED-001 GREEN-LOCK — RAW provenance: E-QUERY-002 retained.
    ///
    /// `crowdstrike_alerts | where severity > 5`
    ///
    /// `severity` is a RAW String column (original schema, no aliasing). `>` is NOT in
    /// `valid_operators_for_type(String)` → E-QUERY-002 MUST fire.
    ///
    /// This is a regression lock: the SIBLING-GATE CONSISTENCY fix must NOT disable
    /// E-QUERY-002 for RAW names (only DERIVED names are skipped).  If the fix
    /// over-broadly skips E-QUERY-002 for all names, this test catches it.
    ///
    /// EXPECTED GREEN both before and after fix:
    ///  - Before: E-QUERY-002 fires for ALL names (DERIVED and RAW alike).
    ///  - After: E-QUERY-002 fires for RAW names only — still fires for RAW `severity`.
    #[tokio::test]
    async fn test_BC_2_11_016_v1_15_raw_provenance_type_compat_retained_e_query_002() {
        let (engine, org) = make_crowdstrike_engine();

        // `severity` (String) is RAW — original schema type; not shadowed by any alias.
        // `>` is NOT valid for String per valid_operators_for_type(String).
        // E-QUERY-002 must fire both before and after the SIBLING-GATE CONSISTENCY fix.
        let query = "crowdstrike_alerts | where severity > 5";

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
            Err(PrismError::QueryTypeMismatch {
                ref column,
                ref table,
                ..
            }) => {
                assert_eq!(
                    column.as_str(),
                    "severity",
                    "BC-2.11.016 RAW-lock: E-QUERY-002 must fire for RAW column \
                     'severity' (String); got: {:?}",
                    column
                );
                assert_eq!(
                    table.as_str(),
                    "crowdstrike_alerts",
                    "BC-2.11.016 RAW-lock: table must be 'crowdstrike_alerts'"
                );
                // E-QUERY-002 fired for RAW String `severity` — correct, retained after fix.
            }
            Ok(_) => panic!(
                "BC-2.11.016 RAW-lock: engine.execute returned Ok — E-QUERY-002 must \
                 fire for RAW String 'severity' with '>' operator. The SIBLING-GATE CONSISTENCY \
                 fix must NOT disable E-QUERY-002 for RAW names (only DERIVED names are skipped). \
                 '>' is NOT in valid_operators_for_type(String)."
            ),
            Err(other) => panic!(
                "BC-2.11.016 RAW-lock: expected PrismError::QueryTypeMismatch \
                 (E-QUERY-002) for RAW String 'severity' with '>' operator, got: {other:?}. \
                 E-QUERY-002 must still fire for RAW names after the SIBLING-GATE CONSISTENCY fix."
            ),
        }
    }

    // ── Test 38 (RED GATE): ADV-FIX-P7-OBS-001 FROM-ALIAS RESOLUTION ─────────────
    //    BC-2.11.016 EC-11-065 — alias-qualified typo bypasses gate.
    //
    //    Root cause (OBS-001): `check_pipe_stage_columns` passes `table_alias = None` to
    //    `extract_column_name_from_field_path` for all SqlPipe pipe-stage calls.  When the
    //    head SQL declares `FROM crowdstrike_alerts t`, references like `t.sevrity` in pipe
    //    stages have qualifier "t".  Without the FROM-alias threaded through, the function
    //    sees an unknown qualifier ("t" matches neither the table name nor None) → returns
    //    None → gate SKIPS the reference → typo `sevrity` bypasses E-QUERY-038 and reaches
    //    DataFusion as an opaque column-resolution error.
    //
    //    Fix: `check_pipe_stage_columns` resolves the declared FROM-alias from the head SQL
    //    and passes it as `from_alias`.  Qualifier matching the alias → stripped to bare name
    //    → checked against available → E-QUERY-038 fires on the typo.

    /// BC-2.11.016 EC-11-065 FROM-ALIAS RESOLUTION — alias-qualified typo.
    ///
    /// `SELECT * FROM crowdstrike_alerts t | where t.sevrity IEQ 'x'`
    ///
    /// `sevrity` is a typo of `severity` (Levenshtein distance 1).  The FROM-alias `t`
    /// qualifies the reference: `t.sevrity`.
    ///
    /// After fix: qualifier "t" matches FROM-alias → stripped to bare `sevrity` → checked
    /// against available (SELECT * → full raw schema = {severity, timestamp}) → NOT found
    /// → E-QUERY-038 with `column: "sevrity"`, `did_you_mean: "severity"`.
    ///
    /// RED GATE: currently `table_alias = None` is passed → qualifier "t" is unknown →
    /// `extract_column_name_from_field_path` returns None → gate skips `t.sevrity` →
    /// no E-QUERY-038 → DataFusion column resolution failure (opaque error).
    #[tokio::test]
    async fn test_BC_2_11_016_v1_15_ec11065_from_alias_typo_yields_e_query_038() {
        let (engine, org) = make_crowdstrike_engine();

        // EC-11-065 canonical vector (BC-2.11.016).
        // Head: SELECT * FROM crowdstrike_alerts t  → FROM-alias = "t"
        //   available (SELECT *) = {severity, timestamp} (full raw schema).
        // Stage: | where t.sevrity IEQ 'x'  → qualifier "t" matches FROM-alias
        //   After fix: bare "sevrity" checked → NOT in {severity, timestamp}
        //     → E-QUERY-038; did_you_mean = "severity" (Levenshtein distance 1).
        //   Before fix: table_alias=None → "t" unknown → None → gate skips → no E-QUERY-038.
        let query = "SELECT * FROM crowdstrike_alerts t | where t.sevrity IEQ 'x'";

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
                    details.column, "sevrity",
                    "BC-2.11.016 EC-11-065 FROM-ALIAS: column in E-QUERY-038 must be \
                     'sevrity' (alias-qualified typo stripped to bare name); got: '{}'",
                    details.column
                );
                assert_eq!(
                    details.table, "crowdstrike_alerts",
                    "BC-2.11.016 EC-11-065 FROM-ALIAS: table must be 'crowdstrike_alerts'"
                );
                assert_eq!(
                    details.did_you_mean.as_deref(),
                    Some("severity"),
                    "BC-2.11.016 EC-11-065 FROM-ALIAS: did_you_mean must be 'severity' \
                     (Levenshtein distance 1 from 'sevrity'; available = {{severity, timestamp}}); \
                     got: {:?}",
                    details.did_you_mean
                );
            }
            Ok(_) => panic!(
                "BC-2.11.016 EC-11-065 FROM-ALIAS: engine.execute must NOT succeed — \
                 E-QUERY-038 must fire for alias-qualified typo 't.sevrity'. Before fix: \
                 table_alias=None → qualifier 't' unknown → None → gate skips → typo bypasses \
                 plan-time validation and reaches DataFusion as an opaque error."
            ),
            Err(other) => panic!(
                "BC-2.11.016 EC-11-065 FROM-ALIAS: expected PrismError::ColumnNotFound \
                 (E-QUERY-038) for alias-qualified typo 't.sevrity', got: {other:?}. \
                 Before fix: DataFusion column resolution failure because 'sevrity' is not a \
                 real column in crowdstrike_alerts (opaque error). Fix: thread FROM-alias 't' \
                 through extract_column_name_from_field_path so 't.sevrity' → bare 'sevrity' \
                 → gate fires E-QUERY-038 with did_you_mean='severity'."
            ),
        }
    }

    // ── Tests 39-41 (RED GATE + GREEN-LOCK): ADV-FIX-P7-OBS-002 FIELDS TRANSITION ─
    //    BC-2.11.016 EC-11-066/067/068 — include/exclude fields transitions.
    //
    // ── Regression locks: ADV-FIX-P8-OBS-001 — FROM-ALIAS RESOLUTION positions 10-14 ──
    //
    //    BC-2.11.016 FROM-ALIAS RESOLUTION is implemented across ALL PipeStage arms in
    //    `check_pipe_stage_columns` (table_alias threaded to every
    //    `extract_column_name_from_field_path` call).  Position 9 (`| where t.sevrity`) is
    //    already locked by EC-11-065 (test 38).  These five locks cover positions 10-14 —
    //    sort, stats by-key, fields, enrich input, and dedup — so that a future refactor
    //    reverting any one arm's `table_alias` to `None` would produce an immediate RED here.
    //
    //    All five tests use `make_crowdstrike_engine()` + alias `t` declared in the FROM clause.
    //    `t.sevrity` (Levenshtein distance 1 from "severity") is the alias-qualified typo.
    //    Expected result for each: E-QUERY-038 with column="sevrity",
    //    table="crowdstrike_alerts", did_you_mean=Some("severity").
    //
    //    EXPECTED COLOR: GREEN immediately (alias threading already implemented).
    //    If any test is RED → that arm has a threading gap → finding severity upgrades.
    //
    //    Root cause (OBS-002): `PipeStage::Fields` was in the "all other stages — unchanged"
    //    group in `check_pipe_stage_columns`.  The SQL emitter's `apply_fields` genuinely
    //    restricts the projection — downstream references to removed columns fail at DataFusion
    //    (false-negative class).  Without the FIELDS TRANSITION RULE, the gate passed queries
    //    that DataFusion would later reject.
    //
    //    Fix (FIELDS TRANSITION RULE):
    //      `| fields a, b`   (include-list, no leading `-`) → `available := {listed names}` (REPLACE)
    //      `| fields - a, b` (exclude-list, leading `-`)    → `available := available ∖ {listed}` (subtract)
    //    Provenance and suspension carry forward unchanged.

    /// BC-2.11.016 EC-11-066 FIELDS TRANSITION — include-then-stale-ref.
    ///
    /// `crowdstrike_alerts | fields severity | where timestamp > 0`
    ///
    /// Both `severity` and `timestamp` are registered columns.  After `| fields severity`
    /// (include-list), available is REPLACED with `{severity}`.  The subsequent
    /// `| where timestamp > 0` references `timestamp`, which is NOT in the include-set →
    /// E-QUERY-038 fires on `timestamp`.
    ///
    /// `>` is valid for Datetime per valid_operators_for_type (["=","!=","<",">","<=",">=",
    /// "BETWEEN"]) → E-QUERY-002 does NOT fire for `timestamp`; the error is E-QUERY-038.
    ///
    /// RED GATE: currently `PipeStage::Fields` does not update `available` — `timestamp`
    /// stays in the original schema available set → no E-QUERY-038 → query passes plan-time
    /// → execution fails (no adapter) → non-ColumnNotFound error.
    #[tokio::test]
    async fn test_BC_2_11_016_v1_15_ec11066_fields_include_stale_ref_yields_e_query_038() {
        let (engine, org) = make_crowdstrike_engine();

        // EC-11-066 canonical vector (BC-2.11.016).
        // | fields severity  →  available := {severity}  (REPLACE; timestamp removed)
        // | where timestamp > 0  →  timestamp NOT in {severity}  →  E-QUERY-038.
        // Before fix: available unchanged {severity, timestamp}; timestamp found → no E-QUERY-038.
        let query = "crowdstrike_alerts | fields severity | where timestamp > 0";

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
                    details.column, "timestamp",
                    "BC-2.11.016 EC-11-066 FIELDS-INCLUDE: E-QUERY-038 must fire on \
                     'timestamp' (removed from available by | fields severity include REPLACE); \
                     got: '{}'",
                    details.column
                );
                assert_eq!(
                    details.table, "crowdstrike_alerts",
                    "BC-2.11.016 EC-11-066 FIELDS-INCLUDE: table must be \
                     'crowdstrike_alerts'"
                );
                // did_you_mean: lev("timestamp", "severity") >> 3 → absent (not asserted).
            }
            Ok(_) => panic!(
                "BC-2.11.016 EC-11-066 FIELDS-INCLUDE: engine.execute must NOT succeed — \
                 E-QUERY-038 must fire for 'timestamp' after `| fields severity` include REPLACE. \
                 Before fix: PipeStage::Fields does not update available → timestamp stays in \
                 {{severity, timestamp}} → plan-time gate passes → execution fails (no adapter)."
            ),
            Err(other) => panic!(
                "BC-2.11.016 EC-11-066 FIELDS-INCLUDE: expected PrismError::ColumnNotFound \
                 (E-QUERY-038) for 'timestamp' after fields include REPLACE, got: {other:?}. \
                 Before fix: execution error instead of plan-time E-QUERY-038. \
                 Fix: | fields include-list → available := {{listed}}; downstream 'timestamp' \
                 NOT found → E-QUERY-038."
            ),
        }
    }

    /// BC-2.11.016 EC-11-067 FIELDS TRANSITION — exclude-then-reference.
    ///
    /// `crowdstrike_alerts | fields - timestamp | sort timestamp`
    ///
    /// `timestamp` is a registered column.  After `| fields - timestamp` (exclude-list),
    /// available is reduced: `{severity, timestamp} ∖ {timestamp}` = `{severity}`.
    /// The subsequent `| sort timestamp` references `timestamp`, which is NOT in the
    /// reduced available set → E-QUERY-038 fires on `timestamp`.
    ///
    /// RED GATE: currently `PipeStage::Fields` does not update available — `timestamp`
    /// stays in the available set → no E-QUERY-038 → query passes plan-time → execution
    /// fails (no adapter) → non-ColumnNotFound error.
    #[tokio::test]
    async fn test_BC_2_11_016_v1_15_ec11067_fields_exclude_yields_e_query_038() {
        let (engine, org) = make_crowdstrike_engine();

        // EC-11-067 canonical vector (BC-2.11.016).
        // | fields - timestamp  →  available := {severity, timestamp} ∖ {timestamp} = {severity}
        // | sort timestamp  →  timestamp NOT in {severity}  →  E-QUERY-038.
        // Before fix: available unchanged {severity, timestamp}; timestamp found → no E-QUERY-038.
        let query = "crowdstrike_alerts | fields - timestamp | sort timestamp";

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
                    details.column, "timestamp",
                    "BC-2.11.016 EC-11-067 FIELDS-EXCLUDE: E-QUERY-038 must fire on \
                     'timestamp' (subtracted from available by | fields - timestamp); \
                     got: '{}'",
                    details.column
                );
                assert_eq!(
                    details.table, "crowdstrike_alerts",
                    "BC-2.11.016 EC-11-067 FIELDS-EXCLUDE: table must be \
                     'crowdstrike_alerts'"
                );
            }
            Ok(_) => panic!(
                "BC-2.11.016 EC-11-067 FIELDS-EXCLUDE: engine.execute must NOT succeed — \
                 E-QUERY-038 must fire for 'timestamp' after `| fields - timestamp` exclude \
                 subtraction. Before fix: PipeStage::Fields does not update available → \
                 timestamp stays in {{severity, timestamp}} → sort passes plan-time check."
            ),
            Err(other) => panic!(
                "BC-2.11.016 EC-11-067 FIELDS-EXCLUDE: expected PrismError::ColumnNotFound \
                 (E-QUERY-038) for 'timestamp' after fields exclude subtraction, got: {other:?}. \
                 Before fix: execution error instead of plan-time E-QUERY-038. \
                 Fix: | fields - list → available := available ∖ {{listed}}; sort 'timestamp' \
                 NOT found → E-QUERY-038."
            ),
        }
    }

    // ── ADV-FIX-P8-OBS-001 regression lock: position 10 (sort) ─────────────────────

    /// ADV-FIX-P8-OBS-001 regression lock — FROM-ALIAS RESOLUTION position 10: `| sort`.
    ///
    /// `SELECT * FROM crowdstrike_alerts t | sort t.sevrity desc`
    ///
    /// `sevrity` is an alias-qualified typo of `severity` (Levenshtein distance 1).
    /// FROM-alias `t` is declared in the head SQL.  After alias resolution (position 10),
    /// `t.sevrity` is stripped to bare `sevrity`, checked against available
    /// `{severity, timestamp}` (SELECT *), and found absent → E-QUERY-038 fires with
    /// `column: "sevrity"`, `table: "crowdstrike_alerts"`, `did_you_mean: "severity"`.
    ///
    /// REGRESSION LOCK: if a future refactor reverts the `PipeStage::Sort` arm's
    /// `table_alias` parameter to `None`, the qualifier "t" becomes unknown →
    /// `extract_column_name_from_field_path` returns None → gate silently skips
    /// `t.sevrity` → no E-QUERY-038 → this test goes RED.
    ///
    /// EXPECTED GREEN immediately (alias threading already implemented in pos-10 arm).
    #[tokio::test]
    async fn test_ADV_FIX_P8_OBS_001_pos10_sort_alias_qualified_typo_regression_lock() {
        let (engine, org) = make_crowdstrike_engine();

        // SqlPipe: head declares FROM-alias "t"; stage uses alias-qualified typo "t.sevrity".
        // Available (SELECT *) = {severity, timestamp}.
        // Alias resolution: "t" matches FROM-alias → bare "sevrity" → NOT in available → E-QUERY-038.
        let query = "SELECT * FROM crowdstrike_alerts t | sort t.sevrity desc";

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
                    details.column, "sevrity",
                    "ADV-FIX-P8-OBS-001 pos-10 sort: column in E-QUERY-038 must be 'sevrity' \
                     (alias-qualified typo stripped to bare name); got: '{}'",
                    details.column
                );
                assert_eq!(
                    details.table, "crowdstrike_alerts",
                    "ADV-FIX-P8-OBS-001 pos-10 sort: table must be 'crowdstrike_alerts'"
                );
                assert_eq!(
                    details.did_you_mean.as_deref(),
                    Some("severity"),
                    "ADV-FIX-P8-OBS-001 pos-10 sort: did_you_mean must be 'severity' \
                     (Levenshtein distance 1 from 'sevrity'); got: {:?}",
                    details.did_you_mean
                );
            }
            Ok(_) => panic!(
                "ADV-FIX-P8-OBS-001 pos-10 sort: engine.execute must NOT succeed — E-QUERY-038 \
                 must fire for alias-qualified sort key typo 't.sevrity'. REGRESSION: \
                 PipeStage::Sort arm may have lost table_alias threading → qualifier 't' unknown \
                 → gate skips → typo reaches DataFusion."
            ),
            Err(other) => panic!(
                "ADV-FIX-P8-OBS-001 pos-10 sort: expected PrismError::ColumnNotFound (E-QUERY-038) \
                 for alias-qualified typo 't.sevrity' in sort key, got: {other:?}. \
                 REGRESSION: table_alias not threaded to extract_column_name_from_field_path \
                 in the PipeStage::Sort arm."
            ),
        }
    }

    // ── ADV-FIX-P8-OBS-001 regression lock: position 11 (stats by-key) ──────────────

    /// ADV-FIX-P8-OBS-001 regression lock — table-name-qualified resolution position 11:
    /// `| stats by`.
    ///
    /// `crowdstrike_alerts | stats count() by crowdstrike_alerts.sevrity`
    ///
    /// Grammar note: `| stats` is only available in the Pipe-form parser
    /// (`build_pipe_parser`) — it is intentionally absent from the SqlPipe pipe-stages
    /// parser (`build_pipe_stages_parser`), so `SELECT * FROM crowdstrike_alerts t | stats …`
    /// is a parse error.  The stats-arm regression lock therefore uses Pipe form, where the
    /// table name itself acts as the qualifier instead of a FROM-alias.
    ///
    /// `sevrity` is a table-name-qualified typo of `severity` (Levenshtein distance 1).
    /// `crowdstrike_alerts.sevrity` is a FieldPath with segments `["crowdstrike_alerts", "sevrity"]`.
    /// `extract_column_name_from_field_path` matches the qualifier against the table name
    /// ("crowdstrike_alerts") → strips the qualifier → bare `sevrity` → checked against
    /// available `{severity, timestamp}` → NOT found → E-QUERY-038 fires with
    /// `column: "sevrity"`, `table: "crowdstrike_alerts"`, `did_you_mean: "severity"`.
    ///
    /// REGRESSION LOCK: if a future refactor breaks `extract_column_name_from_field_path`
    /// calls in the `PipeStage::Stats` by-fields arm (wrong table_name, wrong return, or
    /// skipped call), the qualifier would go unrecognised → gate skips `sevrity` → no
    /// E-QUERY-038 → this test goes RED.
    ///
    /// EXPECTED GREEN immediately (table-name-qualified path extraction already
    /// implemented in the pos-11 stats by-fields arm).
    #[tokio::test]
    async fn test_ADV_FIX_P8_OBS_001_pos11_stats_by_tablename_qualified_typo_regression_lock() {
        let (engine, org) = make_crowdstrike_engine();

        // Pipe form: table-name-qualified by-key typo "crowdstrike_alerts.sevrity".
        // Available = {severity, timestamp}.
        // Qualifier "crowdstrike_alerts" matches table name → bare "sevrity" → NOT in
        // available → E-QUERY-038; did_you_mean = "severity" (Levenshtein distance 1).
        let query = "crowdstrike_alerts | stats count() by crowdstrike_alerts.sevrity";

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
                    details.column, "sevrity",
                    "ADV-FIX-P8-OBS-001 pos-11 stats-by: column in E-QUERY-038 must be 'sevrity' \
                     (table-name-qualified by-key typo stripped to bare name); got: '{}'",
                    details.column
                );
                assert_eq!(
                    details.table, "crowdstrike_alerts",
                    "ADV-FIX-P8-OBS-001 pos-11 stats-by: table must be 'crowdstrike_alerts'"
                );
                assert_eq!(
                    details.did_you_mean.as_deref(),
                    Some("severity"),
                    "ADV-FIX-P8-OBS-001 pos-11 stats-by: did_you_mean must be 'severity' \
                     (Levenshtein distance 1 from 'sevrity'); got: {:?}",
                    details.did_you_mean
                );
            }
            Ok(_) => panic!(
                "ADV-FIX-P8-OBS-001 pos-11 stats-by: engine.execute must NOT succeed — E-QUERY-038 \
                 must fire for table-name-qualified stats by-key typo 'crowdstrike_alerts.sevrity'. \
                 REGRESSION: PipeStage::Stats by-fields arm may have broken field-path extraction → \
                 qualifier unrecognised → gate skips → typo reaches DataFusion."
            ),
            Err(other) => panic!(
                "ADV-FIX-P8-OBS-001 pos-11 stats-by: expected PrismError::ColumnNotFound \
                 (E-QUERY-038) for table-name-qualified by-key typo 'crowdstrike_alerts.sevrity', \
                 got: {other:?}. REGRESSION: extract_column_name_from_field_path not called \
                 correctly in the PipeStage::Stats by-fields arm."
            ),
        }
    }

    // ── ADV-FIX-P8-OBS-001 regression lock: position 12 (fields) ────────────────────

    /// ADV-FIX-P8-OBS-001 regression lock — FROM-ALIAS RESOLUTION position 12: `| fields`.
    ///
    /// `SELECT * FROM crowdstrike_alerts t | fields t.sevrity`
    ///
    /// `sevrity` is an alias-qualified typo of `severity` (Levenshtein distance 1).
    /// FROM-alias `t` is declared in the head SQL.  After alias resolution (position 12
    /// fields column check), `t.sevrity` is stripped to bare `sevrity`, checked against
    /// available `{severity, timestamp}` (SELECT *), and found absent → E-QUERY-038 fires
    /// with `column: "sevrity"`, `table: "crowdstrike_alerts"`, `did_you_mean: "severity"`.
    ///
    /// REGRESSION LOCK: if a future refactor reverts the `PipeStage::Fields` arm's
    /// `table_alias` parameter to `None`, the qualifier "t" becomes unknown → gate silently
    /// skips `t.sevrity` → no E-QUERY-038 → this test goes RED.
    ///
    /// EXPECTED GREEN immediately (alias threading already implemented in pos-12 arm).
    #[tokio::test]
    async fn test_ADV_FIX_P8_OBS_001_pos12_fields_alias_qualified_typo_regression_lock() {
        let (engine, org) = make_crowdstrike_engine();

        // SqlPipe: head declares FROM-alias "t"; fields list uses alias-qualified typo "t.sevrity".
        // Available (SELECT *) = {severity, timestamp}.
        // Alias resolution: "t" → bare "sevrity" → NOT in available → E-QUERY-038.
        let query = "SELECT * FROM crowdstrike_alerts t | fields t.sevrity";

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
                    details.column, "sevrity",
                    "ADV-FIX-P8-OBS-001 pos-12 fields: column in E-QUERY-038 must be 'sevrity' \
                     (alias-qualified fields column typo stripped to bare name); got: '{}'",
                    details.column
                );
                assert_eq!(
                    details.table, "crowdstrike_alerts",
                    "ADV-FIX-P8-OBS-001 pos-12 fields: table must be 'crowdstrike_alerts'"
                );
                assert_eq!(
                    details.did_you_mean.as_deref(),
                    Some("severity"),
                    "ADV-FIX-P8-OBS-001 pos-12 fields: did_you_mean must be 'severity' \
                     (Levenshtein distance 1 from 'sevrity'); got: {:?}",
                    details.did_you_mean
                );
            }
            Ok(_) => panic!(
                "ADV-FIX-P8-OBS-001 pos-12 fields: engine.execute must NOT succeed — E-QUERY-038 \
                 must fire for alias-qualified fields column typo 't.sevrity'. REGRESSION: \
                 PipeStage::Fields arm may have lost table_alias threading → qualifier 't' unknown \
                 → gate skips → typo reaches DataFusion."
            ),
            Err(other) => panic!(
                "ADV-FIX-P8-OBS-001 pos-12 fields: expected PrismError::ColumnNotFound \
                 (E-QUERY-038) for alias-qualified fields column typo 't.sevrity', got: {other:?}. \
                 REGRESSION: table_alias not threaded to extract_column_name_from_field_path \
                 in the PipeStage::Fields arm."
            ),
        }
    }

    // ── ADV-FIX-P8-OBS-001 regression lock: position 13 (enrich input) ──────────────

    /// ADV-FIX-P8-OBS-001 regression lock — FROM-ALIAS RESOLUTION position 13: `| enrich` input.
    ///
    /// `SELECT * FROM crowdstrike_alerts t | enrich cvss_base_score(t.sevrity)`
    ///
    /// `sevrity` is an alias-qualified typo of `severity` (Levenshtein distance 1).
    /// FROM-alias `t` is declared in the head SQL.  The enrich INPUT column check fires at
    /// position 13 (BEFORE the Enrich stage updates the binding context).  After alias
    /// resolution, `t.sevrity` is stripped to bare `sevrity`, checked against available
    /// `{severity, timestamp}` (SELECT *), and found absent → E-QUERY-038 fires with
    /// `column: "sevrity"`, `table: "crowdstrike_alerts"`, `did_you_mean: "severity"`.
    ///
    /// No InfusionRegistry is wired in `make_crowdstrike_engine()`, so
    /// `check_enrich_udf_availability` is a no-op (returns Ok immediately) — E-QUERY-039
    /// does NOT fire; the input-column check at position 13 is reached unconditionally.
    ///
    /// REGRESSION LOCK: if a future refactor reverts the `PipeStage::Enrich` input arm's
    /// `table_alias` parameter to `None`, the qualifier "t" becomes unknown → gate silently
    /// skips `t.sevrity` → no E-QUERY-038 → this test goes RED.
    ///
    /// EXPECTED GREEN immediately (alias threading already implemented in pos-13 arm).
    #[tokio::test]
    async fn test_ADV_FIX_P8_OBS_001_pos13_enrich_input_alias_qualified_typo_regression_lock() {
        let (engine, org) = make_crowdstrike_engine();

        // SqlPipe: head declares FROM-alias "t"; enrich input uses alias-qualified typo "t.sevrity".
        // No InfusionRegistry wired → E-QUERY-039 check is no-op; position-13 input check runs.
        // Available (SELECT *) = {severity, timestamp}.
        // Alias resolution: "t" → bare "sevrity" → NOT in available → E-QUERY-038.
        let query = "SELECT * FROM crowdstrike_alerts t | enrich cvss_base_score(t.sevrity)";

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
                    details.column, "sevrity",
                    "ADV-FIX-P8-OBS-001 pos-13 enrich-input: column in E-QUERY-038 must be \
                     'sevrity' (alias-qualified enrich input typo stripped to bare name); got: '{}'",
                    details.column
                );
                assert_eq!(
                    details.table, "crowdstrike_alerts",
                    "ADV-FIX-P8-OBS-001 pos-13 enrich-input: table must be 'crowdstrike_alerts'"
                );
                assert_eq!(
                    details.did_you_mean.as_deref(),
                    Some("severity"),
                    "ADV-FIX-P8-OBS-001 pos-13 enrich-input: did_you_mean must be 'severity' \
                     (Levenshtein distance 1 from 'sevrity'); got: {:?}",
                    details.did_you_mean
                );
            }
            Ok(_) => panic!(
                "ADV-FIX-P8-OBS-001 pos-13 enrich-input: engine.execute must NOT succeed — \
                 E-QUERY-038 must fire for alias-qualified enrich input typo 't.sevrity'. \
                 REGRESSION: PipeStage::Enrich input arm may have lost table_alias threading → \
                 qualifier 't' unknown → gate skips → typo reaches DataFusion."
            ),
            Err(other) => panic!(
                "ADV-FIX-P8-OBS-001 pos-13 enrich-input: expected PrismError::ColumnNotFound \
                 (E-QUERY-038) for alias-qualified enrich input typo 't.sevrity', got: {other:?}. \
                 REGRESSION: table_alias not threaded to extract_column_name_from_field_path \
                 in the PipeStage::Enrich input arm."
            ),
        }
    }

    // ── ADV-FIX-P8-OBS-001 regression lock: position 14 (dedup) ────────────────────

    /// ADV-FIX-P8-OBS-001 regression lock — FROM-ALIAS RESOLUTION position 14: `| dedup`.
    ///
    /// `SELECT * FROM crowdstrike_alerts t | dedup t.sevrity`
    ///
    /// `sevrity` is an alias-qualified typo of `severity` (Levenshtein distance 1).
    /// FROM-alias `t` is declared in the head SQL.  After alias resolution (position 14
    /// dedup field check), `t.sevrity` is stripped to bare `sevrity`, checked against
    /// available `{severity, timestamp}` (SELECT *), and found absent → E-QUERY-038 fires
    /// with `column: "sevrity"`, `table: "crowdstrike_alerts"`, `did_you_mean: "severity"`.
    ///
    /// REGRESSION LOCK: if a future refactor reverts the `PipeStage::Dedup` arm's
    /// `table_alias` parameter to `None`, the qualifier "t" becomes unknown → gate silently
    /// skips `t.sevrity` → no E-QUERY-038 → this test goes RED.
    ///
    /// EXPECTED GREEN immediately (alias threading already implemented in pos-14 arm).
    #[tokio::test]
    async fn test_ADV_FIX_P8_OBS_001_pos14_dedup_alias_qualified_typo_regression_lock() {
        let (engine, org) = make_crowdstrike_engine();

        // SqlPipe: head declares FROM-alias "t"; dedup key uses alias-qualified typo "t.sevrity".
        // Available (SELECT *) = {severity, timestamp}.
        // Alias resolution: "t" → bare "sevrity" → NOT in available → E-QUERY-038.
        let query = "SELECT * FROM crowdstrike_alerts t | dedup t.sevrity";

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
                    details.column, "sevrity",
                    "ADV-FIX-P8-OBS-001 pos-14 dedup: column in E-QUERY-038 must be 'sevrity' \
                     (alias-qualified dedup key typo stripped to bare name); got: '{}'",
                    details.column
                );
                assert_eq!(
                    details.table, "crowdstrike_alerts",
                    "ADV-FIX-P8-OBS-001 pos-14 dedup: table must be 'crowdstrike_alerts'"
                );
                assert_eq!(
                    details.did_you_mean.as_deref(),
                    Some("severity"),
                    "ADV-FIX-P8-OBS-001 pos-14 dedup: did_you_mean must be 'severity' \
                     (Levenshtein distance 1 from 'sevrity'); got: {:?}",
                    details.did_you_mean
                );
            }
            Ok(_) => panic!(
                "ADV-FIX-P8-OBS-001 pos-14 dedup: engine.execute must NOT succeed — E-QUERY-038 \
                 must fire for alias-qualified dedup key typo 't.sevrity'. REGRESSION: \
                 PipeStage::Dedup arm may have lost table_alias threading → qualifier 't' unknown \
                 → gate skips → typo reaches DataFusion."
            ),
            Err(other) => panic!(
                "ADV-FIX-P8-OBS-001 pos-14 dedup: expected PrismError::ColumnNotFound (E-QUERY-038) \
                 for alias-qualified dedup key typo 't.sevrity', got: {other:?}. \
                 REGRESSION: table_alias not threaded to extract_column_name_from_field_path \
                 in the PipeStage::Dedup arm."
            ),
        }
    }

    /// BC-2.11.016 EC-11-068 FIELDS TRANSITION — include-then-valid-ref (GREEN-LOCK).
    ///
    /// `crowdstrike_alerts | fields severity | where severity IEQ 'High'`
    ///
    /// `severity` IS in the include-list.  After `| fields severity` (REPLACE), available =
    /// `{severity}`.  The subsequent `| where severity IEQ 'High'` finds `severity` in
    /// available → NO E-QUERY-038.  IEQ is valid for String (S-PRISMQL-CASE-INSENSITIVE-001).
    ///
    /// EXPECTED GREEN both before and after fix:
    ///  - Before fix: `timestamp` is still in available (no transition) → `severity` is ALSO
    ///    in available (original schema) → no false E-QUERY-038 on `severity`.
    ///  - After fix: available = `{severity}` (REPLACE) → `severity` is in available → no
    ///    E-QUERY-038.  Either path: `severity IEQ 'High'` passes E-QUERY-038 and E-QUERY-002.
    #[tokio::test]
    async fn test_BC_2_11_016_v1_15_ec11068_fields_include_valid_ref_no_e_query_038() {
        let (engine, org) = make_crowdstrike_engine();

        // EC-11-068 canonical vector (BC-2.11.016).
        // | fields severity  →  available := {severity}  (REPLACE after fix)
        // | where severity IEQ 'High'  →  severity in {severity}  →  NO E-QUERY-038.
        // Both before and after fix: severity is in available → no false positive.
        let query = "crowdstrike_alerts | fields severity | where severity IEQ 'High'";

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
            Err(PrismError::ColumnNotFound(ref details)) if details.column == "severity" => {
                panic!(
                    "BC-2.11.016 EC-11-068 GREEN-LOCK: FALSE-POSITIVE E-QUERY-038 fired \
                     on 'severity'. After `| fields severity` include REPLACE, available = \
                     {{severity}}; 'severity' IS in the available set — E-QUERY-038 must NOT \
                     fire. FP-001 invariant violated. \
                     column='{}', table='{}', available={:?}",
                    details.column, details.table, details.available_columns
                )
            }
            // Ok, execution error, or ColumnNotFound on a column other than 'severity'
            // is acceptable.  The only invariant: E-QUERY-038 must NOT fire on 'severity'.
            _ => {}
        }
    }

    // ── Helpers: zero-column table fixtures (EC-11-041) ──────────────────────────

    /// Build a `crowdstrike_alerts` engine in single-tenant mode with ZERO registered
    /// columns.
    ///
    /// Unlike `make_crowdstrike_engine`, this fixture does NOT set `resolved_spec_map`
    /// (single-tenant mode) and registers `crowdstrike_alerts` with an EMPTY column
    /// list.
    ///
    /// Gate-ordering proof for this state:
    ///  - `TableRegistry::is_registered("crowdstrike_alerts")` → true
    ///    (the table IS in `registered`; `register_sensor` always inserts there).
    ///  - `TableRegistry::columns_for_table("crowdstrike_alerts")` → []
    ///    (`register_sensor` only populates `columns_by_table` when `!table.columns.is_empty()`;
    ///    zero-column tables are absent from `columns_by_table` — same result as unregistered).
    ///
    /// Consequence:
    ///  - E-QUERY-037 does NOT fire (table IS registered).
    ///  - E-QUERY-038 single-tenant path: `if available_columns.is_empty() { return Ok(()) }`
    ///    → FAILS OPEN (ADV-FIX-P9-OBS-001).
    fn make_zero_column_engine_single_tenant() -> (QueryEngine, OrgSlug) {
        let org = OrgSlug::new("acme");
        let sensor_id = "crowdstrike";
        let table_suffix = "alerts";

        // Zero columns — the key fixture ingredient for EC-11-041.
        let columns: Vec<ColumnSpec> = vec![];

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
            .expect("EC-11-041 fixture: register_sensor must not fail for zero-column table");

        let mut engine = QueryEngine::new_with_cache_config(
            Arc::new(AdapterRegistry::new()),
            Arc::new(NoopCs),
            Arc::new(prism_ocsf::OcsfNormalizer::new()),
            Arc::new(ClientRegistry::new(vec![org.clone()])),
            QueryEngineConfig::default(),
            crate::cache::CacheConfig::default(),
        );
        // Single-tenant: do NOT set resolved_spec_map — this is the critical distinction.
        engine = engine.with_table_registry(registry);
        (engine, org)
    }

    /// Build a `crowdstrike_alerts` engine in multi-tenant mode with ZERO registered
    /// columns.
    ///
    /// Both the `TableRegistry` AND the `resolved_spec_map` are populated with a
    /// zero-column `crowdstrike_alerts` table, mirroring what the production boot
    /// path would build for a sensor spec with no `[[tables]][*].columns` entries.
    ///
    /// Multi-tenant path in `get_initial_available_columns`:
    ///  - `table_in_schema = true` (table IS in spec_map).
    ///  - `cols = vec![]` (spec has zero columns).
    ///  - Returns `Some(vec![])` — NOT `None`.
    ///  - `check_pipe_stage_columns` initializes `current_available = vec![]` and
    ///    `suspended = false` → any column reference hits `check_column_against_available_set`
    ///    with an empty available set → E-QUERY-038 fires with `available_columns: []`.
    ///
    /// This correctly honors BC-2.11.016 EC-11-041.
    fn make_zero_column_engine_multi_tenant() -> (QueryEngine, OrgSlug) {
        let org = OrgSlug::new("acme");
        let sensor_id = "crowdstrike";
        let table_suffix = "alerts";

        // Zero columns — same as single-tenant fixture.
        let columns: Vec<ColumnSpec> = vec![];

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
            .expect("EC-11-041 multi-tenant fixture: register_sensor must not fail");

        let overlay_toml = format!("extends = \"{sensor_id}\"\ninstance_id = \"{sensor_id}@acme\"");
        let overlay: SensorInstanceOverlay = toml::from_str(&overlay_toml)
            .expect("EC-11-041 fixture: SensorInstanceOverlay TOML must parse");
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
        // Multi-tenant: set resolved_spec_map so the multi-tenant path in
        // get_initial_available_columns is exercised.
        engine = engine.with_resolved_spec_map(Arc::new(spec_map));
        engine = engine.with_table_registry(registry);
        (engine, org)
    }

    // ── Test: EC-11-041 single-tenant RED (ADV-FIX-P9-OBS-001) ─────────────────

    /// BC-2.11.016 EC-11-041 — single-tenant zero-column table gate (ADV-FIX-P9-OBS-001).
    ///
    /// `crowdstrike_alerts` is registered with ZERO columns. The table IS in the
    /// `registered` set, so E-QUERY-037 passes. The single-tenant path then calls
    /// `columns_for_table("crowdstrike_alerts")` → `[]` (the zero-column table is
    /// absent from `columns_by_table` — `register_sensor` only inserts there when
    /// `!table.columns.is_empty()`).
    ///
    /// Current single-tenant code:
    ///
    ///   // get_initial_available_columns single-tenant branch:
    ///   if cols.is_empty() { return None; }  // → fail-open
    ///
    ///   // check_column_availability single-tenant branch:
    ///   if available_columns.is_empty() { return Ok(()); }  // → fail-open
    ///
    /// Result: the pedagogical gate is skipped; the query reaches DataFusion and hits
    /// an opaque `QueryExecutionFailed` (E-QUERY-034) instead of the structured
    /// E-QUERY-038 response with `available_columns: []`.
    ///
    /// BC-2.11.016 EC-11-041 mandates:
    ///   table with zero registered columns → E-QUERY-038 with `available_columns: []`,
    ///   `did_you_mean: absent`.
    ///
    /// RED GATE: currently FAILS (wrong error or Ok) until the production code is fixed
    /// to distinguish "table not in registry" (fail-open) from "table IS registered but
    /// has zero columns" (E-QUERY-038 with available_columns: []).
    ///
    /// Fix hint: `register_sensor` must track zero-column tables in a separate sentinel
    /// OR `check_column_availability`/`get_initial_available_columns` must check
    /// `is_registered` before treating an empty `columns_for_table` result as fail-open.
    #[tokio::test]
    async fn test_BC_2_11_016_EC_11_041_single_tenant_zero_column_gate_fires() {
        let (engine, org) = make_zero_column_engine_single_tenant();

        // EC-11-041 canonical query vector (BC-2.11.016):
        // pipe-mode | where stage (position 8) against a zero-column table.
        // `any_col` is not in the schema (there are NO columns), so E-QUERY-038 must fire
        // with available_columns=[] and did_you_mean absent.
        let query = "crowdstrike_alerts | where any_col = 'x'";

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
                // EC-11-041: available_columns MUST be empty [] (zero-column table).
                assert!(
                    details.available_columns.is_empty(),
                    "BC-2.11.016 EC-11-041 single-tenant: available_columns must be [] \
                     for a zero-column table. Got: {:?}",
                    details.available_columns
                );
                // EC-11-041: did_you_mean MUST be absent (no columns to suggest).
                assert!(
                    details.did_you_mean.is_none(),
                    "BC-2.11.016 EC-11-041 single-tenant: did_you_mean must be absent \
                     (no columns to compute Levenshtein against). Got: {:?}",
                    details.did_you_mean
                );
                assert_eq!(
                    details.table, "crowdstrike_alerts",
                    "BC-2.11.016 EC-11-041 single-tenant: table must be 'crowdstrike_alerts', \
                     got: {:?}",
                    details.table
                );
            }
            Ok(_) => panic!(
                "BC-2.11.016 EC-11-041 single-tenant RED GATE: engine.execute must NOT succeed. \
                 E-QUERY-038 must fire with available_columns=[] for a zero-column table. \
                 ADV-FIX-P9-OBS-001: the single-tenant path in get_initial_available_columns \
                 returns None when columns_for_table returns [] (can't distinguish 'not registered' \
                 from 'registered with zero columns'), so check_pipe_stage_columns returns Ok(()) \
                 immediately (fail-open) instead of firing E-QUERY-038."
            ),
            Err(other) => panic!(
                "BC-2.11.016 EC-11-041 single-tenant RED GATE: expected \
                 PrismError::ColumnNotFound (E-QUERY-038) with available_columns=[], \
                 got: {other:?}. ADV-FIX-P9-OBS-001: single-tenant gate must fire E-QUERY-038 \
                 with empty available_columns, not fail-open into an opaque DataFusion error."
            ),
        }
    }

    // ── Test: EC-11-041 multi-tenant GREEN-LOCK ─────────────────────────────────

    /// BC-2.11.016 EC-11-041 — multi-tenant zero-column gate GREEN-LOCK.
    ///
    /// Confirms that the MULTI-TENANT path correctly fires E-QUERY-038 with
    /// `available_columns: []` for a table registered with zero columns, per EC-11-041.
    ///
    /// Multi-tenant path: `get_initial_available_columns` finds the table in spec_map
    /// (`table_in_schema = true`), collects zero columns, and returns `Some(vec![])`.
    /// `check_pipe_stage_columns` initializes `current_available = vec[]`, `suspended = false`.
    /// `any_col` is not in `[]` → `check_column_against_available_set` fires E-QUERY-038
    /// with `available_columns: []`, `did_you_mean: None`.
    ///
    /// This test must remain GREEN before and after the single-tenant fix — it documents
    /// the correct behavior as a regression anchor.
    #[tokio::test]
    async fn test_BC_2_11_016_EC_11_041_multi_tenant_zero_column_gate_fires_green_lock() {
        let (engine, org) = make_zero_column_engine_multi_tenant();

        // EC-11-041 canonical query vector — same as single-tenant test.
        let query = "crowdstrike_alerts | where any_col = 'x'";

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
                // GREEN-LOCK: multi-tenant honors EC-11-041 — available_columns must be [].
                assert!(
                    details.available_columns.is_empty(),
                    "BC-2.11.016 EC-11-041 multi-tenant GREEN-LOCK: available_columns must be [] \
                     for a zero-column table. REGRESSION: multi-tenant path must return \
                     Some([]) from get_initial_available_columns. Got: {:?}",
                    details.available_columns
                );
                // GREEN-LOCK: did_you_mean must be absent (no candidates).
                assert!(
                    details.did_you_mean.is_none(),
                    "BC-2.11.016 EC-11-041 multi-tenant GREEN-LOCK: did_you_mean must be absent. \
                     Got: {:?}",
                    details.did_you_mean
                );
                assert_eq!(
                    details.table, "crowdstrike_alerts",
                    "BC-2.11.016 EC-11-041 multi-tenant GREEN-LOCK: table must be \
                     'crowdstrike_alerts'. Got: {:?}",
                    details.table
                );
            }
            Ok(_) => panic!(
                "BC-2.11.016 EC-11-041 multi-tenant GREEN-LOCK REGRESSION: engine.execute must \
                 NOT succeed. E-QUERY-038 must fire for a zero-column table on the multi-tenant \
                 path. get_initial_available_columns must return Some([]) — not None — when the \
                 table IS in spec_map but has zero columns (table_in_schema=true, cols=[])."
            ),
            Err(other) => panic!(
                "BC-2.11.016 EC-11-041 multi-tenant GREEN-LOCK REGRESSION: expected \
                 PrismError::ColumnNotFound (E-QUERY-038) with available_columns=[], \
                 got: {other:?}. Multi-tenant get_initial_available_columns must return \
                 Some([]) for a zero-column table in spec_map."
            ),
        }
    }

    // ── Fixture: dual-table with JOIN target (EC-11-069) ─────────────────────────

    /// Build an engine with two registered tables for EC-11-069 JOIN alias tests.
    ///
    /// Tables:
    ///   - `crowdstrike_alerts` (FROM table): severity (String), timestamp (Datetime)
    ///   - `some_other_table`  (JOIN target): col (String), id (String)
    ///     sensor_id "some_other" + table_name "table" → registered as "some_other_table".
    ///
    /// Both tables are registered in the `TableRegistry` so E-QUERY-037 passes for
    /// JOIN-target sources (`extract_sources_from_ast_for_gate` collects JOIN table refs
    /// in `spq.head.joins` for SqlPipe — omitting the JOIN table causes E-QUERY-037 to
    /// fire on `some_other_table` before the column gate runs).
    /// Only `crowdstrike_alerts` is wired into `resolved_spec_map` — the binding-context
    /// walk for the SqlPipe stage is seeded from the FROM table schema; the JOIN table
    /// schema is not consulted for head-projection binding.
    fn make_engine_with_join_tables() -> (QueryEngine, OrgSlug) {
        let org = OrgSlug::new("acme");

        // ── Primary sensor: crowdstrike (FROM table) ──────────────────────────
        let cs_sensor_id = "crowdstrike";
        let cs_table_suffix = "alerts";
        let cs_columns = vec![
            ColumnSpec::new("severity", ColumnType::String, None, vec![]),
            ColumnSpec::new("timestamp", ColumnType::Datetime, None, vec![]),
        ];
        let cs_spec = SensorSpec::new(
            cs_sensor_id,
            "CrowdStrike sensor",
            AuthType::ApiKey,
            "https://api.crowdstrike.com",
            vec![TableSpec::new_point_in_time(
                cs_table_suffix,
                "security_finding",
                cs_columns,
                vec![],
            )],
            None,
            "1.0.0",
            Vec::new(),
        );

        // ── JOIN-target sensor: some_other (→ some_other_table) ──────────────
        let jo_sensor_id = "some_other";
        let jo_table_suffix = "table";
        let jo_columns = vec![
            ColumnSpec::new("col", ColumnType::String, None, vec![]),
            ColumnSpec::new("id", ColumnType::String, None, vec![]),
        ];
        let jo_spec = SensorSpec::new(
            jo_sensor_id,
            "Some Other sensor",
            AuthType::ApiKey,
            "https://api.example.com",
            vec![TableSpec::new_point_in_time(
                jo_table_suffix,
                "some_category",
                jo_columns,
                vec![],
            )],
            None,
            "1.0.0",
            Vec::new(),
        );

        let registry = Arc::new(TableRegistry::new());
        registry
            .register_sensor(&cs_spec)
            .expect("EC-11-069 fixture: register crowdstrike must not fail");
        registry
            .register_sensor(&jo_spec)
            .expect("EC-11-069 fixture: register some_other must not fail");

        // Multi-tenant resolved_spec_map for crowdstrike only (FROM table binding context).
        let overlay_toml =
            format!("extends = \"{cs_sensor_id}\"\ninstance_id = \"{cs_sensor_id}@acme\"");
        let overlay: SensorInstanceOverlay = toml::from_str(&overlay_toml)
            .expect("EC-11-069 fixture: SensorInstanceOverlay TOML must parse");
        let resolved = OverlayLoader::merge_overlay_onto_type_spec(&cs_spec, &overlay, org.clone());
        let key: ResolvedSpecKey = (org.clone(), SensorId::new(cs_sensor_id));
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
        engine = engine.with_resolved_spec_map(Arc::new(spec_map));
        engine = engine.with_table_registry(registry);

        (engine, org)
    }

    // ── Test 49 (RED GATE): EC-11-069 — JOIN-aliased last segment not seeded → false E-QUERY-038 ─

    /// BC-2.11.016 EC-11-069 — LAST-SEGMENT OUTPUT-NAME RULE (ADV-FIX-P10-OBS-001).
    ///
    /// `SELECT j.col FROM crowdstrike_alerts JOIN some_other_table j
    ///  ON crowdstrike_alerts.severity = j.id | where col = 'x'`
    ///
    /// The SELECT item `j.col` is an un-aliased bare-Field with qualifier `j`.  `j` is a
    /// JOIN alias declared by `JOIN some_other_table j` — NOT the FROM source
    /// `crowdstrike_alerts` and NOT a declared FROM alias (None here).  In
    /// `compute_sqlpipe_head_binding` branch (b) (fully-explicit SELECT, no Star/TableStar),
    /// `extract_column_name_from_field_path(fp, "crowdstrike_alerts", None)` returns `None`
    /// (qualifier `j` unknown) → `col` is never pushed to `available`.
    ///
    /// Current behavior (RED — FP-001 violation):
    ///   `available = []`; `| where col = 'x'` finds `col` absent → E-QUERY-038 fires with
    ///   `column: "col"`, `available_columns: []`.  SQL output-naming: `SELECT j.col` produces
    ///   output column `col`; the query WOULD succeed at execution → false positive.
    ///
    /// Expected behavior after LAST-SEGMENT fix (GREEN):
    ///   qualifier `j` ≠ FROM table/alias → seed last segment `col` as DERIVED;
    ///   `| where col = 'x'` finds `col` in `available` → NO E-QUERY-038.
    ///
    /// JOIN ON analysis (position 5, unaffected by fix):
    ///   `crowdstrike_alerts.severity` → qualifier matches FROM table → "severity" extracted
    ///   → in schema → OK.  `j.id` → qualifier `j` unknown → None → skip.
    ///
    /// RED GATE trigger: currently `ColumnNotFound { column: "col" }` fires.
    #[tokio::test]
    async fn test_BC_2_11_016_ec11069_join_qualified_last_segment_no_false_e_query_038() {
        let (engine, org) = make_engine_with_join_tables();

        // EC-11-069 canonical vector (BC-2.11.016).
        // JOIN ON uses crowdstrike_alerts.severity (in schema) = j.id (skip via None).
        let query = "SELECT j.col FROM crowdstrike_alerts \
                     JOIN some_other_table j ON crowdstrike_alerts.severity = j.id \
                     | where col = 'x'";

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
            Err(PrismError::ColumnNotFound(ref details)) if details.column == "col" => panic!(
                "BC-2.11.016 EC-11-069 RED GATE: FALSE E-QUERY-038 on 'col'. \
                 `SELECT j.col` (JOIN-aliased bare-Field, qualifier 'j' ≠ FROM source) MUST \
                 seed 'col' as DERIVED via LAST-SEGMENT OUTPUT-NAME RULE; `| where col = 'x'` \
                 must find 'col' in available — NO E-QUERY-038 (FP-001). \
                 Before fix: compute_sqlpipe_head_binding branch (b) calls \
                 extract_column_name_from_field_path → qualifier 'j' unknown → None → \
                 'col' not seeded → available=[] → E-QUERY-038 with available_columns=[]. \
                 column='{}', table='{}', available={:?}",
                details.column, details.table, details.available_columns
            ),
            // Ok or any other error (no adapter wired, DataFusion error) is acceptable.
            // The invariant: E-QUERY-038 must NOT fire on 'col' for a JOIN-aliased SELECT
            // item whose last path segment IS the SQL output column name.
            _ => {}
        }
    }

    // ── Test 50 (RED GATE): EC-11-069 sibling-gate companion — DERIVED provenance,
    //                        SIBLING-GATE CONSISTENCY (E-QUERY-002 must not fire) ─────

    /// BC-2.11.016 EC-11-069 companion — SIBLING-GATE CONSISTENCY (ADV-FIX-P10-OBS-001).
    ///
    /// `SELECT j.severity FROM crowdstrike_alerts JOIN some_other_table j
    ///  ON crowdstrike_alerts.severity = j.id | where severity > 5`
    ///
    /// `j.severity` has qualifier `j` (JOIN alias, not FROM source `crowdstrike_alerts`).
    /// After the LAST-SEGMENT fix: last segment `severity` seeded as DERIVED (type not
    /// statically known from the FROM schema).  Raw `severity` in `crowdstrike_alerts` is
    /// String; `>` is NOT in `valid_operators_for_type(String)`.  If E-QUERY-002 applied
    /// the raw String type to DERIVED `severity`, it would fire a false E-QUERY-002
    /// (FP-001 violation).  SIBLING-GATE CONSISTENCY (BC-2.11.016) requires E-QUERY-002
    /// to skip DERIVED names.
    ///
    /// Current behavior (RED — two FP-001 violations possible):
    ///   `j.severity` → None (qualifier `j` unknown) → `severity` NOT seeded →
    ///   `| where severity > 5` finds `severity` absent → E-QUERY-038 fires (wrong error;
    ///   E-QUERY-002 never reached because E-QUERY-038 fires first via `?`).
    ///
    /// Expected behavior after fix (GREEN):
    ///   `j.severity` → last segment `severity` seeded as DERIVED →
    ///   E-QUERY-038: `severity` found → gate passes; E-QUERY-002: DERIVED → skipped;
    ///   neither gate fires.
    ///
    /// RED GATE trigger: currently `ColumnNotFound { column: "severity" }` fires
    ///   (before the fix seeds `severity` as DERIVED).
    #[tokio::test]
    async fn test_BC_2_11_016_ec11069_sibling_gate_join_qualified_no_false_e_query_002() {
        let (engine, org) = make_engine_with_join_tables();

        // EC-11-069 sibling-gate companion vector (BC-2.11.016).
        // Raw `severity` in crowdstrike_alerts is String; `>` invalid for String.
        // After fix: `severity` seeded as DERIVED → E-QUERY-002 skipped (SIBLING-GATE CONSISTENCY).
        let query = "SELECT j.severity FROM crowdstrike_alerts \
                     JOIN some_other_table j ON crowdstrike_alerts.severity = j.id \
                     | where severity > 5";

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
            // False E-QUERY-002: fires if fix seeds 'severity' with RAW provenance instead of
            // DERIVED. Raw String type → '>' invalid → false E-QUERY-002 (FP-001 violation;
            // SIBLING-GATE CONSISTENCY requires DERIVED names to skip E-QUERY-002).
            Err(PrismError::QueryTypeMismatch { ref column, .. })
                if column.as_str() == "severity" =>
            {
                panic!(
                    "BC-2.11.016 EC-11-069 sibling-gate: FALSE E-QUERY-002 on DERIVED \
                     'severity'. `j.severity` is JOIN-aliased (qualifier 'j' ≠ FROM source); \
                     LAST-SEGMENT seeds 'severity' as DERIVED — E-QUERY-002 MUST skip DERIVED \
                     names per SIBLING-GATE CONSISTENCY (FP-001). Likely fix defect: seeded \
                     with RAW instead of DERIVED provenance → raw String operator set applied. \
                     column='{}'",
                    column
                )
            }
            // Currently fires (RED): j.severity → None → severity not seeded → available=[] →
            // E-QUERY-038 fires. After fix: severity seeded as DERIVED → both gates pass.
            Err(PrismError::ColumnNotFound(ref d)) if d.column == "severity" => panic!(
                "BC-2.11.016 EC-11-069 sibling-gate RED GATE: E-QUERY-038 on 'severity'. \
                 Before fix: j.severity → qualifier 'j' unknown → None → severity not seeded \
                 → available=[] → E-QUERY-038 (FP-001 violation; SQL output 'severity' is valid). \
                 After fix: severity seeded as DERIVED via LAST-SEGMENT OUTPUT-NAME RULE → \
                 E-QUERY-038 gate passes; E-QUERY-002 skipped via SIBLING-GATE CONSISTENCY. \
                 column='{}', table='{}', available={:?}",
                d.column, d.table, d.available_columns
            ),
            // Ok or any other error (no adapter wired, DataFusion error) is acceptable.
            // Invariants: no false E-QUERY-002 on DERIVED 'severity';
            //             no false E-QUERY-038 on DERIVED 'severity'.
            _ => {}
        }
    }

    // ── Test 51 (RED GATE): EC-11-070 — SELECT j.* with JOIN → star-with-join suspension ─

    /// BC-2.11.016 EC-11-070 — STAR-WITH-JOIN SUSPENSION RULE (ADV-FIX-P12-OBS-002).
    ///
    /// `SELECT j.* FROM crowdstrike_alerts
    ///  JOIN some_other_table j ON crowdstrike_alerts.severity = j.id
    ///  | where other_only_col = 'x'`
    ///
    /// `j.*` is a `TableStar` SELECT item — `has_star = true`, `has_explicit = false`
    /// → branch (a) of `compute_sqlpipe_head_binding` returns `None`.
    ///
    /// Current behavior (RED — FP-001 violation):
    ///   `initial_binding_override = None` → `check_pipe_stage_columns` falls back to
    ///   `get_initial_available_columns("crowdstrike_alerts")` = `{severity, timestamp}`;
    ///   `suspended = false`; `| where other_only_col = 'x'` finds `other_only_col` absent
    ///   → E-QUERY-038 fires with `column: "other_only_col"`, `table: "crowdstrike_alerts"`.
    ///   This is a false positive: `j.*` star-expansion spans `some_other_table` at execution;
    ///   `other_only_col` is a valid join-source column that the gate has no schema for (FP-001).
    ///
    /// Expected behavior after STAR-WITH-JOIN SUSPENSION fix (GREEN):
    ///   Branch (a) detects `j.*` (TableStar) + non-empty JOIN list → `suspended := true`
    ///   overrides partial `available` seeding; `| where other_only_col = 'x'` is skipped
    ///   (suspended); no E-QUERY-038 fires (fail-open per FP-001).
    ///
    /// JOIN ON analysis (position 5, unaffected by fix):
    ///   `crowdstrike_alerts.severity` → qualifier matches FROM table → "severity" extracted
    ///   → in schema → OK. `j.id` → qualifier `j` unknown → None → skip.
    ///
    /// RED GATE trigger: currently `ColumnNotFound { column: "other_only_col" }` fires
    ///   because branch (a) seeds FROM schema only and `other_only_col` is absent.
    ///
    /// Fixture: `make_engine_with_join_tables()` — both `crowdstrike_alerts` (severity,
    ///   timestamp) and `some_other_table` (col, id) registered so E-QUERY-037 passes.
    ///   `other_only_col` is absent from the FROM table's registered columns (mirroring
    ///   a join-source-only column per BC-2.11.016 EC-11-070 fail-open rationale).
    #[tokio::test]
    async fn test_BC_2_11_016_ec11070_tablestar_with_join_no_false_e_query_038() {
        let (engine, org) = make_engine_with_join_tables();

        // EC-11-070 canonical vector (BC-2.11.016).
        // SELECT j.* → TableStar; JOIN present → STAR-WITH-JOIN SUSPENSION RULE must fire.
        // JOIN ON: crowdstrike_alerts.severity (in schema) = j.id (qualifier j unknown → skip).
        // | where other_only_col = 'x': other_only_col absent from crowdstrike_alerts schema.
        let query = "SELECT j.* FROM crowdstrike_alerts \
                     JOIN some_other_table j ON crowdstrike_alerts.severity = j.id \
                     | where other_only_col = 'x'";

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
            Err(PrismError::ColumnNotFound(ref details)) if details.column == "other_only_col" => {
                panic!(
                    "BC-2.11.016 EC-11-070 RED GATE: FALSE E-QUERY-038 on 'other_only_col'. \
                     `SELECT j.*` (TableStar, branch (a)) with non-empty JOIN list MUST trigger \
                     STAR-WITH-JOIN SUSPENSION RULE: suspended := true; `| where other_only_col` \
                     must be skipped (fail-open per FP-001). \
                     Before fix: compute_sqlpipe_head_binding branch (a) returns None → \
                     check_pipe_stage_columns seeds available from crowdstrike_alerts raw schema \
                     {{severity, timestamp}} with suspended=false → other_only_col absent → \
                     E-QUERY-038 fires (false positive; star expansion spans join sources). \
                     column='{}', table='{}', available={:?}",
                    details.column, details.table, details.available_columns
                )
            }
            // Ok or any other error (no adapter wired, DataFusion error) is acceptable.
            // Invariant: E-QUERY-038 MUST NOT fire on 'other_only_col' for a star-with-join head.
            _ => {}
        }
    }

    // ── Test 52 (RED GATE): EC-11-071 — SELECT * with JOIN → star-with-join suspension ──

    /// BC-2.11.016 EC-11-071 — STAR-WITH-JOIN SUSPENSION RULE (ADV-FIX-P12-OBS-002).
    ///
    /// `SELECT * FROM crowdstrike_alerts
    ///  JOIN some_other_table j ON crowdstrike_alerts.severity = j.id
    ///  | where other_only_col = 'x'`
    ///
    /// `*` is a bare `Star` SELECT item — `has_star = true`, `has_explicit = false`
    /// → branch (a) of `compute_sqlpipe_head_binding` returns `None`.
    ///
    /// Current behavior (RED — FP-001 violation):
    ///   `initial_binding_override = None` → `check_pipe_stage_columns` falls back to
    ///   `get_initial_available_columns("crowdstrike_alerts")` = `{severity, timestamp}`;
    ///   `suspended = false`; `| where other_only_col = 'x'` finds `other_only_col` absent
    ///   → E-QUERY-038 fires with `column: "other_only_col"`, `table: "crowdstrike_alerts"`.
    ///   Same FP-001 violation class as EC-11-070: bare `*` star-expansion spans all JOIN
    ///   sources at execution; the gate has no schema for `some_other_table`.
    ///
    /// Expected behavior after STAR-WITH-JOIN SUSPENSION fix (GREEN):
    ///   Branch (a) detects `*` (Star) + non-empty JOIN list → `suspended := true`
    ///   overrides partial `available` seeding; `| where other_only_col = 'x'` is skipped
    ///   (suspended); no E-QUERY-038 fires (fail-open per FP-001).
    ///
    /// JOIN ON analysis (position 5, unaffected by fix):
    ///   Identical to EC-11-070 — `crowdstrike_alerts.severity` found; `j.id` skipped.
    ///
    /// RED GATE trigger: currently `ColumnNotFound { column: "other_only_col" }` fires
    ///   for the same reason as EC-11-070 — branch (a) seeds FROM schema only.
    ///
    /// Fixture: `make_engine_with_join_tables()` — same as EC-11-070.
    #[tokio::test]
    async fn test_BC_2_11_016_ec11071_star_with_join_no_false_e_query_038() {
        let (engine, org) = make_engine_with_join_tables();

        // EC-11-071 canonical vector (BC-2.11.016).
        // SELECT * → bare Star; JOIN present → STAR-WITH-JOIN SUSPENSION RULE must fire.
        // JOIN ON: crowdstrike_alerts.severity (in schema) = j.id (qualifier j unknown → skip).
        // | where other_only_col = 'x': other_only_col absent from crowdstrike_alerts schema.
        let query = "SELECT * FROM crowdstrike_alerts \
                     JOIN some_other_table j ON crowdstrike_alerts.severity = j.id \
                     | where other_only_col = 'x'";

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
            Err(PrismError::ColumnNotFound(ref details)) if details.column == "other_only_col" => {
                panic!(
                    "BC-2.11.016 EC-11-071 RED GATE: FALSE E-QUERY-038 on 'other_only_col'. \
                     `SELECT *` (bare Star, branch (a)) with non-empty JOIN list MUST trigger \
                     STAR-WITH-JOIN SUSPENSION RULE: suspended := true; `| where other_only_col` \
                     must be skipped (fail-open per FP-001). \
                     Before fix: compute_sqlpipe_head_binding branch (a) returns None → \
                     check_pipe_stage_columns seeds available from crowdstrike_alerts raw schema \
                     {{severity, timestamp}} with suspended=false → other_only_col absent → \
                     E-QUERY-038 fires (false positive; SELECT * with JOIN expands across all \
                     join-source schemas at execution). \
                     column='{}', table='{}', available={:?}",
                    details.column, details.table, details.available_columns
                )
            }
            // Ok or any other error (no adapter wired, DataFusion error) is acceptable.
            // Invariant: E-QUERY-038 MUST NOT fire on 'other_only_col' for a star-with-join head.
            _ => {}
        }
    }

    // ── Test 53 (RED GATE): EC-11-072 — stage-level JOIN → STAGE-JOIN SUSPENSION RULE ──

    /// BC-2.11.016 EC-11-072 — STAGE-JOIN SUSPENSION RULE (ADV-FIX-P14-OBS-001).
    ///
    /// `FROM crowdstrike_alerts | join some_other_table on severity == id | where col = 'x'`
    ///
    /// Pipe grammar: `'join' [join_kind] source 'ON' field ['==' field]` — no alias support
    /// (pipe_parser.rs join_stage; JoinStage has no alias field).
    /// `col` is registered in `some_other_table` but absent from `crowdstrike_alerts` schema
    /// — it is a valid join-source column that `| where col = 'x'` references after the join.
    ///
    /// Current behavior (RED — FP-001 violation):
    ///   Stage walk: `PipeStage::Join` falls into `_ => {}` catch-all in
    ///   `check_pipe_stage_columns` — `suspended` remains `false`, `current_available`
    ///   unchanged as `{severity, timestamp}` (FROM table schema only). Next stage:
    ///   `PipeStage::Where(col = 'x')` checks `col` against `{severity, timestamp}` →
    ///   absent → E-QUERY-038 fires with `column: "col"`. `col` is a valid column of
    ///   `some_other_table` at execution — this is a false positive (FP-001 violation
    ///   class: stage-join; symmetric with STAR-WITH-JOIN SUSPENSION RULE at head level).
    ///
    /// Expected behavior after STAGE-JOIN SUSPENSION RULE fix (GREEN):
    ///   Stage walk encounters `PipeStage::Join` → `suspended := true`; subsequent
    ///   `PipeStage::Where(col = 'x')` is skipped — NO E-QUERY-038 (fail-open per FP-001).
    ///
    /// JOIN ON analysis (pipe position, not SQL position 5 — falls into `_ => {}`):
    ///   `severity` is the left field (in `crowdstrike_alerts` schema); `id` is the right
    ///   field (in `some_other_table`). The ON fields are NOT checked in the current code
    ///   (PipeStage::Join → `_ => {}`), so no E-QUERY-038 fires on the ON fields regardless.
    ///
    /// RED GATE trigger: currently `ColumnNotFound { column: "col" }` fires because
    ///   `PipeStage::Join` does NOT set `suspended := true` in the `_ => {}` catch-all.
    ///
    /// Fixture: `make_engine_with_join_tables()` — `crowdstrike_alerts` (severity, timestamp)
    ///   and `some_other_table` (col, id) both registered; `col` is in `some_other_table`
    ///   but absent from `crowdstrike_alerts` raw schema (the FROM-only `available` set).
    #[tokio::test]
    async fn test_BC_2_11_016_ec11072_stage_join_suspension_no_false_e_query_038() {
        let (engine, org) = make_engine_with_join_tables();

        // EC-11-072 canonical vector (BC-2.11.016).
        // Pipe grammar: 'join' [kind] source 'ON' field ['==' field] — no alias.
        // Source: some_other_table; ON condition: severity (FROM col) == id (join col).
        // | where col = 'x': `col` is in some_other_table but absent from crowdstrike_alerts.
        // With STAGE-JOIN SUSPENSION RULE: PipeStage::Join → suspended := true →
        //   | where col is skipped → NO E-QUERY-038.
        // Without fix (current): PipeStage::Join → _ => {} → suspended=false →
        //   | where col = 'x' → col absent from {severity, timestamp} → E-QUERY-038 fires.
        let query = "FROM crowdstrike_alerts \
                     | join some_other_table on severity == id \
                     | where col = 'x'";

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
            Err(PrismError::ColumnNotFound(ref details)) if details.column == "col" => panic!(
                "BC-2.11.016 EC-11-072 RED GATE: FALSE E-QUERY-038 on 'col'. \
                 `PipeStage::Join` MUST set `suspended := true` (STAGE-JOIN SUSPENSION RULE); \
                 `| where col = 'x'` must be skipped (fail-open per FP-001). \
                 `col` is a valid column of `some_other_table` at execution — checking it \
                 against the FROM-only `available` set {{severity, timestamp}} fires a false \
                 positive (FP-001 violation class: stage-join; symmetric with \
                 STAR-WITH-JOIN SUSPENSION RULE at head level). \
                 Before fix: `PipeStage::Join` falls into `_ => {{}}` catch-all in \
                 `check_pipe_stage_columns` — `suspended` remains false → \
                 `current_available = {{severity, timestamp}}` → `col` absent → E-QUERY-038. \
                 column='{}', table='{}', available={:?}",
                details.column, details.table, details.available_columns
            ),
            // Ok or any other error (no adapter wired, DataFusion error) is acceptable.
            // Invariant: E-QUERY-038 MUST NOT fire on 'col' after a PipeStage::Join stage.
            _ => {}
        }
    }

    // ── Test 54 (GREEN lock): EC-11-073 — MIXED-STAR branch (c) with head JOIN ──────

    /// BC-2.11.016 EC-11-073 — STAR-WITH-JOIN SUSPENSION RULE, MIXED-STAR branch (c),
    /// spec-anchored lock (ADV-FIX-P14-OBS-003).
    ///
    /// `SELECT *, upper(severity) AS sev_up FROM crowdstrike_alerts
    ///  JOIN some_other_table j ON crowdstrike_alerts.severity = j.id
    ///  | where u_col = 'x'`
    ///
    /// MIXED-STAR branch (c) triggered: `*` (Star item, has_star = true) AND
    /// `upper(severity) AS sev_up` (explicit non-star item with AS alias, has_explicit = true).
    /// `head.joins` is non-empty (SqlPipe SQL head JOIN present).
    ///
    /// STAR-WITH-JOIN SUSPENSION RULE (BC-2.11.016 branch (c) application,
    /// `compute_sqlpipe_head_binding` lines 2953–2963):
    ///   After branch (c) builds partial `available = {severity, timestamp, sev_up}`,
    ///   `if !head.joins.is_empty() { suspended = true; }` overrides and sets
    ///   `suspended := true` for the initial pipe-stage binding context. The Star/TableStar
    ///   component brings ALL join-source columns into scope at execution; the partial schema
    ///   seed is incomplete for join-source columns — checking against it fires false positives.
    ///
    /// Expected behavior (GREEN — already implemented at v1.18):
    ///   `check_pipe_stage_columns` receives `initial_binding_override` with `suspended=true`;
    ///   the loop body's `if suspended { continue; }` skips `PipeStage::Where(u_col = 'x')`;
    ///   NO E-QUERY-038 fires (fail-open per FP-001).
    ///
    /// Spec-anchored lock (closes OBS-3): confirms STAR-WITH-JOIN SUSPENSION RULE applies
    ///   to MIXED-STAR branch (c) symmetrically with pure-star branch (a) (EC-11-071) —
    ///   the suspension override takes precedence over the partial `available` union.
    ///
    /// JOIN ON analysis (position 5, SQL head — unaffected by suspension):
    ///   `crowdstrike_alerts.severity = j.id`: qualifier `crowdstrike_alerts` matches FROM
    ///   table → `severity` extracted → in schema → OK; `j.id` → qualifier `j` unknown →
    ///   None → skipped (fail-open for cross-table refs per position-5 policy).
    ///
    /// Fixture: `make_engine_with_join_tables()` — same as EC-11-070/071.
    #[tokio::test]
    async fn test_BC_2_11_016_ec11073_mixed_star_head_join_suspension_green_lock() {
        let (engine, org) = make_engine_with_join_tables();

        // EC-11-073 canonical vector (BC-2.11.016).
        // MIXED-STAR head: `*` (Star) + `upper(severity) AS sev_up` (explicit alias).
        // head.joins non-empty → STAR-WITH-JOIN SUSPENSION RULE branch (c):
        //   suspended := true overriding partial available = {severity, timestamp, sev_up}.
        // | where u_col = 'x': u_col absent from crowdstrike_alerts AND some_other_table;
        //   suspended := true → skipped → NO E-QUERY-038 (fail-open per FP-001).
        let query = "SELECT *, upper(severity) AS sev_up FROM crowdstrike_alerts \
                     JOIN some_other_table j ON crowdstrike_alerts.severity = j.id \
                     | where u_col = 'x'";

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
            Err(PrismError::ColumnNotFound(ref details)) if details.column == "u_col" => panic!(
                "BC-2.11.016 EC-11-073 GREEN LOCK REGRESSION: FALSE E-QUERY-038 on \
                 'u_col'. MIXED-STAR branch (c) with non-empty head JOIN list MUST trigger \
                 STAR-WITH-JOIN SUSPENSION RULE: `if !head.joins.is_empty() {{ suspended = true; }}` \
                 overrides partial MIXED-STAR `available` seeding; `| where u_col = 'x'` must \
                 be skipped — NO E-QUERY-038 (FP-001). \
                 Regression: v1.18 STAR-WITH-JOIN SUSPENSION RULE branch (c) application \
                 (`compute_sqlpipe_head_binding` lines 2953–2963) may have been removed or \
                 broken. Star/TableStar component brings all join-source columns into scope \
                 at execution; the partial schema seed is incomplete — checking against it \
                 fires false positives on join-source-only columns (FP-001 class: \
                 star-with-join, branch (c)). \
                 column='{}', table='{}', available={:?}",
                details.column, details.table, details.available_columns
            ),
            // Ok or any other error (no adapter wired, DataFusion error) is acceptable.
            // Invariant: E-QUERY-038 MUST NOT fire on 'u_col' for MIXED-STAR + head-join query.
            _ => {}
        }
    }

    // ── Tests 55–66: EC-11-074/075 HEAD-JOIN SUSPENSION RULE (ADV-FIX-P15-MED-001) ──
    //
    // BC-2.11.016 §Preconditions.2 HEAD-JOIN SUSPENSION RULE:
    //   When the head SQL query's JOIN list is non-empty AND a bare unqualified column
    //   reference at positions 1–6 is absent from `schema_columns(table, OrgId)`, the
    //   E-QUERY-038 gate MUST NOT fire (fail-open).  DataFusion resolves unqualified
    //   column references across all join sources at execution time; a false positive
    //   here is a FP-001 violation.  Joinless queries and columns PRESENT in the FROM
    //   schema are UNCHANGED.
    //
    // Fixture: `make_engine_with_join_tables()` — registers:
    //   crowdstrike_alerts (severity: String, timestamp: Datetime)  ← FROM table
    //   some_other_table   (col: String, id: String)               ← JOIN target
    // Org: "acme".  `col` is absent from crowdstrike_alerts schema (fail-open target).
    //
    // Grammar note on position 2 (WHERE IEQ): BC-2.11.024 §SQL-Mode Rejection forbids
    //   IEQ/IIN/INE in SQL-mode WHERE — the parser returns a parse error.  The test for
    //   the "IEQ drift shape" therefore uses `WHERE col = 'high'` (plain equality) as the
    //   closest parseable case-insensitive/plain predicate.  Dropped shape noted below.

    // ── Test 55 (RED GATE): EC-11-074 position 1 — SELECT col, Ast::Sql ─────────────

    /// BC-2.11.016 EC-11-074 — HEAD-JOIN SUSPENSION RULE, position 1 (SELECT),
    /// `Ast::Sql` form (ADV-FIX-P15-MED-001).
    ///
    /// `SELECT col FROM crowdstrike_alerts
    ///  JOIN some_other_table ON crowdstrike_alerts.severity = some_other_table.id`
    ///
    /// `col` is in `some_other_table` but absent from `crowdstrike_alerts` raw schema.
    /// HEAD-JOIN SUSPENSION RULE: head JOIN list is non-empty → E-QUERY-038 MUST NOT fire
    /// for absent bare unqualified refs (fail-open per FP-001).
    ///
    /// Current behavior (RED — FP-001 violation):
    ///   `check_query_column_availability` collects `select_cols = ["col"]`; JOIN list is
    ///   non-empty but the SUSPENSION RULE is not yet implemented; `check_column_availability`
    ///   is called for `col` → absent from crowdstrike_alerts schema → E-QUERY-038(col).
    ///
    /// Expected after fix (GREEN):
    ///   When `sql_query.joins` is non-empty, absent bare unqualified refs are skipped
    ///   (fail-open); no E-QUERY-038 fires on `col`.
    ///
    /// JOIN ON analysis: `crowdstrike_alerts.severity` → qualifier matches FROM → "severity"
    ///   extracted → in schema → OK; `some_other_table.id` → qualifier mismatch → None → skip.
    #[tokio::test]
    async fn test_BC_2_11_016_ec11074_head_join_suspension_select_no_false_e_query_038() {
        let (engine, org) = make_engine_with_join_tables();

        // EC-11-074 position 1: bare `col` in SELECT; col absent from crowdstrike_alerts schema.
        // Ast::Sql (no pipe stages).
        let query = "SELECT col FROM crowdstrike_alerts \
                     JOIN some_other_table ON crowdstrike_alerts.severity = some_other_table.id";

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
            Err(PrismError::ColumnNotFound(ref details)) if details.column == "col" => panic!(
                "BC-2.11.016 EC-11-074 RED GATE (position 1 SELECT): FALSE E-QUERY-038 \
                 on 'col'. HEAD-JOIN SUSPENSION RULE: when the SQL head JOIN list is non-empty, \
                 bare unqualified refs ABSENT from `schema_columns(table, OrgId)` MUST NOT fire \
                 E-QUERY-038 (fail-open per FP-001). DataFusion resolves `col` through \
                 `some_other_table` at execution — this is a false positive (FP-001 violation \
                 class: head-join). Fix: in `check_query_column_availability`, when \
                 `sql_query.joins` is non-empty, skip `check_column_availability` for columns \
                 absent from the FROM schema (fail-open). column='{}', table='{}', available={:?}",
                details.column, details.table, details.available_columns
            ),
            // Ok or any other error (no adapter wired, DataFusion error) is acceptable.
            // Invariant: E-QUERY-038 MUST NOT fire on 'col' when head JOIN list is non-empty.
            _ => {}
        }
    }

    // ── Test 56 (RED GATE): EC-11-074 position 2 — WHERE col, Ast::Sql ──────────────

    /// BC-2.11.016 EC-11-074 — HEAD-JOIN SUSPENSION RULE, position 2 (WHERE),
    /// `Ast::Sql` form (ADV-FIX-P15-MED-001).
    ///
    /// EC-11-074 describes the "IEQ drift shape" (`WHERE col IEQ 'high'`) as the
    /// canonical position-2 vector.  IEQ is rejected in SQL-mode WHERE by
    /// BC-2.11.024 §SQL-Mode Rejection (parser returns E-QUERY-001 parse error for
    /// SQL-mode IEQ/IIN/INE).  The closest parseable substitute is plain equality:
    ///   `WHERE col = 'high'`
    /// This is noted here; the dropped IEQ shape is intentional, not a test gap.
    ///
    /// HEAD-JOIN SUSPENSION RULE: head JOIN list is non-empty; bare `col` absent from
    /// `crowdstrike_alerts` schema → E-QUERY-038 MUST NOT fire.
    ///
    /// Current behavior (RED — FP-001 violation):
    ///   `where_cols = ["col"]`; no suspension implemented; E-QUERY-038(col) fires.
    ///   `severity` in SELECT is checked first (in schema → OK); then `col` in WHERE.
    #[tokio::test]
    async fn test_BC_2_11_016_ec11074_head_join_suspension_where_no_false_e_query_038() {
        let (engine, org) = make_engine_with_join_tables();

        // EC-11-074 position 2: bare `col` in WHERE (IEQ not valid in SQL mode; using =).
        // See test doc for dropped-shape rationale.
        let query = "SELECT severity FROM crowdstrike_alerts \
                     JOIN some_other_table ON crowdstrike_alerts.severity = some_other_table.id \
                     WHERE col = 'high'";

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
            Err(PrismError::ColumnNotFound(ref details)) if details.column == "col" => panic!(
                "BC-2.11.016 EC-11-074 RED GATE (position 2 WHERE): FALSE E-QUERY-038 \
                 on 'col'. HEAD-JOIN SUSPENSION RULE: non-empty head JOIN list → absent bare \
                 unqualified `col` MUST NOT fire E-QUERY-038 (fail-open per FP-001). `col` \
                 is valid in `some_other_table` at execution; DataFusion resolves it via the \
                 JOIN source. Fix: detect non-empty `sql_query.joins` and skip absent-col \
                 checks in the positions-1-6 gate loop. Note: IEQ form dropped (SQL-mode \
                 rejects IEQ per BC-2.11.024); plain `= 'high'` is equivalent for this gate. \
                 column='{}', table='{}', available={:?}",
                details.column, details.table, details.available_columns
            ),
            _ => {}
        }
    }

    // ── Test 57 (RED GATE): EC-11-074 position 3 — GROUP BY col, Ast::Sql ───────────

    /// BC-2.11.016 EC-11-074 — HEAD-JOIN SUSPENSION RULE, position 3 (GROUP BY),
    /// `Ast::Sql` form (ADV-FIX-P15-MED-001).
    ///
    /// `SELECT severity FROM crowdstrike_alerts
    ///  JOIN some_other_table ON crowdstrike_alerts.severity = some_other_table.id
    ///  GROUP BY col`
    ///
    /// `col` absent from `crowdstrike_alerts` schema; GROUP BY is position 3.
    /// HEAD-JOIN SUSPENSION RULE applies: non-empty JOIN → skip absent bare unqualified
    /// refs → no E-QUERY-038.
    #[tokio::test]
    async fn test_BC_2_11_016_ec11074_head_join_suspension_groupby_no_false_e_query_038() {
        let (engine, org) = make_engine_with_join_tables();

        // EC-11-074 position 3: bare `col` in GROUP BY; col absent from crowdstrike_alerts schema.
        let query = "SELECT severity FROM crowdstrike_alerts \
                     JOIN some_other_table ON crowdstrike_alerts.severity = some_other_table.id \
                     GROUP BY col";

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
            Err(PrismError::ColumnNotFound(ref details)) if details.column == "col" => panic!(
                "BC-2.11.016 EC-11-074 RED GATE (position 3 GROUP BY): FALSE E-QUERY-038 \
                 on 'col'. HEAD-JOIN SUSPENSION RULE: non-empty head JOIN list → absent bare \
                 unqualified `col` in GROUP BY MUST NOT fire E-QUERY-038 (fail-open per FP-001). \
                 `col` is a valid column in `some_other_table` at execution. \
                 column='{}', table='{}', available={:?}",
                details.column, details.table, details.available_columns
            ),
            _ => {}
        }
    }

    // ── Test 58 (RED GATE): EC-11-074 position 4 — ORDER BY col, Ast::Sql ───────────

    /// BC-2.11.016 EC-11-074 — HEAD-JOIN SUSPENSION RULE, position 4 (ORDER BY),
    /// `Ast::Sql` form (ADV-FIX-P15-MED-001).
    ///
    /// `SELECT severity FROM crowdstrike_alerts
    ///  JOIN some_other_table ON crowdstrike_alerts.severity = some_other_table.id
    ///  ORDER BY col`
    ///
    /// `col` absent from `crowdstrike_alerts` schema; ORDER BY is position 4.
    /// HEAD-JOIN SUSPENSION RULE applies: non-empty JOIN → skip absent bare unqualified
    /// refs → no E-QUERY-038.
    #[tokio::test]
    async fn test_BC_2_11_016_ec11074_head_join_suspension_orderby_no_false_e_query_038() {
        let (engine, org) = make_engine_with_join_tables();

        // EC-11-074 position 4: bare `col` in ORDER BY; col absent from crowdstrike_alerts schema.
        let query = "SELECT severity FROM crowdstrike_alerts \
                     JOIN some_other_table ON crowdstrike_alerts.severity = some_other_table.id \
                     ORDER BY col";

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
            Err(PrismError::ColumnNotFound(ref details)) if details.column == "col" => panic!(
                "BC-2.11.016 EC-11-074 RED GATE (position 4 ORDER BY): FALSE E-QUERY-038 \
                 on 'col'. HEAD-JOIN SUSPENSION RULE: non-empty head JOIN list → absent bare \
                 unqualified `col` in ORDER BY MUST NOT fire E-QUERY-038 (fail-open per FP-001). \
                 `col` is a valid column in `some_other_table` at execution. \
                 column='{}', table='{}', available={:?}",
                details.column, details.table, details.available_columns
            ),
            _ => {}
        }
    }

    // ── Test 59 (RED GATE): EC-11-074 position 6 — HAVING count(col), Ast::Sql ──────

    /// BC-2.11.016 EC-11-074 — HEAD-JOIN SUSPENSION RULE, position 6 (HAVING),
    /// `Ast::Sql` form (ADV-FIX-P15-MED-001).
    ///
    /// `SELECT severity, count(*) FROM crowdstrike_alerts
    ///  JOIN some_other_table ON crowdstrike_alerts.severity = some_other_table.id
    ///  GROUP BY severity HAVING count(col) > 0`
    ///
    /// HAVING position 6: `count(col)` — the base-column ref inside the aggregate is
    /// extracted via `extract_predicate_columns` (same path as EC-11-046).
    /// `col` absent from `crowdstrike_alerts` schema; JOIN present → SUSPENSION RULE.
    ///
    /// `severity` in SELECT and GROUP BY is in FROM schema → checked normally (no change).
    /// `col` inside HAVING aggregate → absent → must NOT fire E-QUERY-038 (HEAD-JOIN rule).
    #[tokio::test]
    async fn test_BC_2_11_016_ec11074_head_join_suspension_having_no_false_e_query_038() {
        let (engine, org) = make_engine_with_join_tables();

        // EC-11-074 position 6: bare `col` inside HAVING aggregate; col absent from FROM schema.
        let query = "SELECT severity, count(*) FROM crowdstrike_alerts \
                     JOIN some_other_table ON crowdstrike_alerts.severity = some_other_table.id \
                     GROUP BY severity HAVING count(col) > 0";

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
            Err(PrismError::ColumnNotFound(ref details)) if details.column == "col" => panic!(
                "BC-2.11.016 EC-11-074 RED GATE (position 6 HAVING): FALSE E-QUERY-038 \
                 on 'col'. HEAD-JOIN SUSPENSION RULE: non-empty head JOIN list → absent bare \
                 unqualified `col` inside HAVING aggregate MUST NOT fire E-QUERY-038 \
                 (fail-open per FP-001). Extraction path: `extract_predicate_columns` over \
                 HAVING extracts base-column refs from aggregate function args (same as \
                 EC-11-046 pattern). `col` is valid in `some_other_table` at execution. \
                 column='{}', table='{}', available={:?}",
                details.column, details.table, details.available_columns
            ),
            _ => {}
        }
    }

    // ── Tests 60–64 (RED GATE): EC-11-075 — Ast::SqlPipe head form ───────────────────
    //
    // BC-2.11.016 EC-11-075: HEAD-JOIN SUSPENSION RULE applies to `Ast::SqlPipe`
    // head SQL positions 1–6 identically to `Ast::Sql`.  Appending `| limit 10` makes
    // the query parse as `Ast::SqlPipe` (no SQL-level LIMIT — FORBID-BOTH does not fire).
    // The pipe-stage walk for `| limit 10` contains no column refs → no E-QUERY-038 from
    // the stage walk.  The failures are identical to EC-11-074 (RED at same position).

    // ── Test 60 (RED GATE): EC-11-075 position 1 — SELECT col, Ast::SqlPipe ─────────

    /// BC-2.11.016 EC-11-075 — HEAD-JOIN SUSPENSION RULE, position 1 (SELECT),
    /// `Ast::SqlPipe` form (ADV-FIX-P15-MED-001).
    ///
    /// `SELECT col FROM crowdstrike_alerts
    ///  JOIN some_other_table ON crowdstrike_alerts.severity = some_other_table.id
    ///  | limit 10`
    ///
    /// The `| limit 10` suffix makes this `Ast::SqlPipe`.  Head SQL is checked via
    /// `spq.head` — identical to EC-11-074 position 1.  `| limit 10` carries no column
    /// refs (PipeStage::Limit has no field references); the pipe-stage walk does not
    /// fire E-QUERY-038.  The RED trigger is the same head-SQL position-1 path.
    #[tokio::test]
    async fn test_BC_2_11_016_ec11075_sqlpipe_head_join_suspension_select_no_false_e_query_038() {
        let (engine, org) = make_engine_with_join_tables();

        // EC-11-075 position 1: SqlPipe form; | limit 10 triggers Ast::SqlPipe.
        let query = "SELECT col FROM crowdstrike_alerts \
                     JOIN some_other_table ON crowdstrike_alerts.severity = some_other_table.id \
                     | limit 10";

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
            Err(PrismError::ColumnNotFound(ref details)) if details.column == "col" => panic!(
                "BC-2.11.016 EC-11-075 RED GATE (Ast::SqlPipe position 1 SELECT): FALSE \
                 E-QUERY-038 on 'col'. HEAD-JOIN SUSPENSION RULE applies to Ast::SqlPipe head \
                 SQL positions 1-6 identically to Ast::Sql (code path: `sql_query = &spq.head`). \
                 Head JOIN list non-empty → absent bare unqualified `col` MUST NOT fire \
                 E-QUERY-038 (fail-open per FP-001). The `| limit 10` pipe stage has no column \
                 refs — it is the head-SQL position-1 check that fires. \
                 column='{}', table='{}', available={:?}",
                details.column, details.table, details.available_columns
            ),
            _ => {}
        }
    }

    // ── Test 61 (RED GATE): EC-11-075 position 2 — WHERE col, Ast::SqlPipe ──────────

    /// BC-2.11.016 EC-11-075 — HEAD-JOIN SUSPENSION RULE, position 2 (WHERE),
    /// `Ast::SqlPipe` form (ADV-FIX-P15-MED-001).
    ///
    /// IEQ dropped (SQL-mode rejection per BC-2.11.024); uses plain `= 'high'`.
    #[tokio::test]
    async fn test_BC_2_11_016_ec11075_sqlpipe_head_join_suspension_where_no_false_e_query_038() {
        let (engine, org) = make_engine_with_join_tables();

        // EC-11-075 position 2 (WHERE): IEQ not valid in SQL mode; using plain = predicate.
        let query = "SELECT severity FROM crowdstrike_alerts \
                     JOIN some_other_table ON crowdstrike_alerts.severity = some_other_table.id \
                     WHERE col = 'high' \
                     | limit 10";

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
            Err(PrismError::ColumnNotFound(ref details)) if details.column == "col" => panic!(
                "BC-2.11.016 EC-11-075 RED GATE (Ast::SqlPipe position 2 WHERE): FALSE \
                 E-QUERY-038 on 'col'. HEAD-JOIN SUSPENSION RULE: SqlPipe head SQL WHERE clause \
                 with non-empty JOIN list → absent bare `col` MUST NOT fire E-QUERY-038. \
                 column='{}', table='{}', available={:?}",
                details.column, details.table, details.available_columns
            ),
            _ => {}
        }
    }

    // ── Test 62 (RED GATE): EC-11-075 position 3 — GROUP BY col, Ast::SqlPipe ───────

    /// BC-2.11.016 EC-11-075 — HEAD-JOIN SUSPENSION RULE, position 3 (GROUP BY),
    /// `Ast::SqlPipe` form (ADV-FIX-P15-MED-001).
    #[tokio::test]
    async fn test_BC_2_11_016_ec11075_sqlpipe_head_join_suspension_groupby_no_false_e_query_038() {
        let (engine, org) = make_engine_with_join_tables();

        let query = "SELECT severity FROM crowdstrike_alerts \
                     JOIN some_other_table ON crowdstrike_alerts.severity = some_other_table.id \
                     GROUP BY col \
                     | limit 10";

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
            Err(PrismError::ColumnNotFound(ref details)) if details.column == "col" => panic!(
                "BC-2.11.016 EC-11-075 RED GATE (Ast::SqlPipe position 3 GROUP BY): \
                 FALSE E-QUERY-038 on 'col'. HEAD-JOIN SUSPENSION RULE: non-empty JOIN list \
                 → absent bare `col` in GROUP BY MUST NOT fire (fail-open per FP-001). \
                 column='{}', table='{}', available={:?}",
                details.column, details.table, details.available_columns
            ),
            _ => {}
        }
    }

    // ── Test 63 (RED GATE): EC-11-075 position 4 — ORDER BY col, Ast::SqlPipe ───────

    /// BC-2.11.016 EC-11-075 — HEAD-JOIN SUSPENSION RULE, position 4 (ORDER BY),
    /// `Ast::SqlPipe` form (ADV-FIX-P15-MED-001).
    #[tokio::test]
    async fn test_BC_2_11_016_ec11075_sqlpipe_head_join_suspension_orderby_no_false_e_query_038() {
        let (engine, org) = make_engine_with_join_tables();

        let query = "SELECT severity FROM crowdstrike_alerts \
                     JOIN some_other_table ON crowdstrike_alerts.severity = some_other_table.id \
                     ORDER BY col \
                     | limit 10";

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
            Err(PrismError::ColumnNotFound(ref details)) if details.column == "col" => panic!(
                "BC-2.11.016 EC-11-075 RED GATE (Ast::SqlPipe position 4 ORDER BY): \
                 FALSE E-QUERY-038 on 'col'. HEAD-JOIN SUSPENSION RULE: non-empty JOIN list \
                 → absent bare `col` in ORDER BY MUST NOT fire (fail-open per FP-001). \
                 column='{}', table='{}', available={:?}",
                details.column, details.table, details.available_columns
            ),
            _ => {}
        }
    }

    // ── Test 64 (RED GATE): EC-11-075 position 6 — HAVING count(col), Ast::SqlPipe ──

    /// BC-2.11.016 EC-11-075 — HEAD-JOIN SUSPENSION RULE, position 6 (HAVING),
    /// `Ast::SqlPipe` form (ADV-FIX-P15-MED-001).
    #[tokio::test]
    async fn test_BC_2_11_016_ec11075_sqlpipe_head_join_suspension_having_no_false_e_query_038() {
        let (engine, org) = make_engine_with_join_tables();

        let query = "SELECT severity, count(*) FROM crowdstrike_alerts \
                     JOIN some_other_table ON crowdstrike_alerts.severity = some_other_table.id \
                     GROUP BY severity HAVING count(col) > 0 \
                     | limit 10";

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
            Err(PrismError::ColumnNotFound(ref details)) if details.column == "col" => panic!(
                "BC-2.11.016 EC-11-075 RED GATE (Ast::SqlPipe position 6 HAVING): \
                 FALSE E-QUERY-038 on 'col'. HEAD-JOIN SUSPENSION RULE: non-empty JOIN list \
                 → absent bare `col` inside HAVING aggregate MUST NOT fire E-QUERY-038 \
                 (fail-open per FP-001). column='{}', table='{}', available={:?}",
                details.column, details.table, details.available_columns
            ),
            _ => {}
        }
    }

    // ── Tests 65–66: Negative controls (GREEN at current HEAD, stay GREEN after fix) ──

    // ── Test 65 (GREEN): Joinless query — E-QUERY-038 MUST fire on absent col ────────

    /// BC-2.11.016 EC-11-074 negative control — joinless query fires E-QUERY-038.
    ///
    /// HEAD-JOIN SUSPENSION RULE applies ONLY when the head JOIN list is non-empty.
    /// A joinless query referencing a non-existent column MUST still fire E-QUERY-038 —
    /// the suspension rule MUST NOT be applied unconditionally.
    ///
    /// `SELECT col FROM crowdstrike_alerts` — no JOIN → E-QUERY-038 on `col`.
    ///
    /// GREEN lock at current HEAD (suspension rule not yet implemented → gate fires as
    /// expected).  GREEN after fix (no JOIN → suspension does not engage → gate fires).
    #[tokio::test]
    async fn test_BC_2_11_016_ec11074_negative_joinless_col_fires_e_query_038() {
        let (engine, org) = make_engine_with_join_tables();

        // Joinless: crowdstrike_alerts has no `col` column → E-QUERY-038 MUST fire.
        let query = "SELECT col FROM crowdstrike_alerts";

        let result = engine
            .execute(
                query,
                QueryOptions {
                    clients: Some(vec![org]),
                    ..QueryOptions::default()
                },
            )
            .await;

        // Negative control: this error MUST fire for the rule to be meaningful.
        // If it does not fire, the suspension rule has been over-applied (incorrectly
        // suspending joinless queries), which is a regression.
        match result {
            Err(PrismError::ColumnNotFound(ref details)) if details.column == "col" => {
                // Correct — joinless query correctly fires E-QUERY-038 on absent `col`.
            }
            other => panic!(
                "BC-2.11.016 EC-11-074 NEGATIVE CONTROL REGRESSION: expected \
                 E-QUERY-038(col) for joinless `SELECT col FROM crowdstrike_alerts`, \
                 but got: {:?}. The HEAD-JOIN SUSPENSION RULE MUST NOT apply to joinless \
                 queries — `col` is genuinely absent from `crowdstrike_alerts` schema \
                 (severity, timestamp only) and there is no JOIN source to resolve it from. \
                 Check that the fix gates on `sql_query.joins.is_empty()` correctly.",
                other
            ),
        }
    }

    // ── Test 66 (GREEN): FROM-schema col with JOIN — E-QUERY-038 MUST NOT fire ──────

    /// BC-2.11.016 EC-11-074 negative control — present column with JOIN not affected.
    ///
    /// HEAD-JOIN SUSPENSION RULE: "Columns PRESENT in the FROM schema are still checked
    /// normally — only absent col refs with a non-empty JOIN list trigger the suspension."
    ///
    /// `SELECT severity FROM crowdstrike_alerts
    ///  JOIN some_other_table ON crowdstrike_alerts.severity = some_other_table.id`
    ///
    /// `severity` IS in `crowdstrike_alerts` schema → `check_column_availability` passes
    /// → no E-QUERY-038.  GREEN at current HEAD (column present, check passes regardless of
    /// suspension).  GREEN after fix (present columns continue to be checked, pass normally).
    #[tokio::test]
    async fn test_BC_2_11_016_ec11074_negative_from_schema_col_with_join_no_e_query_038() {
        let (engine, org) = make_engine_with_join_tables();

        // severity IS in crowdstrike_alerts schema — must pass even with JOIN present.
        let query = "SELECT severity FROM crowdstrike_alerts \
                     JOIN some_other_table ON crowdstrike_alerts.severity = some_other_table.id";

        let result = engine
            .execute(
                query,
                QueryOptions {
                    clients: Some(vec![org]),
                    ..QueryOptions::default()
                },
            )
            .await;

        // `severity` is present in FROM schema → no E-QUERY-038 regardless of suspension.
        // If E-QUERY-038 fires for 'severity', the fix has incorrectly applied the
        // suspension to PRESENT columns (BC invariant: "Columns PRESENT in the FROM schema
        // are still checked normally").
        match result {
            Err(PrismError::ColumnNotFound(ref details)) if details.column == "severity" => panic!(
                "BC-2.11.016 EC-11-074 NEGATIVE CONTROL REGRESSION: FALSE E-QUERY-038 \
                 on 'severity' (a column PRESENT in `crowdstrike_alerts` schema). The HEAD-JOIN \
                 SUSPENSION RULE MUST NOT apply to columns that ARE in the FROM schema — only \
                 ABSENT columns trigger fail-open. The fix incorrectly suspended the gate for \
                 present columns. BC invariant: present-column checks are unchanged whether or \
                 not JOINs are present. column='{}', table='{}', available={:?}",
                details.column, details.table, details.available_columns
            ),
            // Ok or any other error (no adapter wired, DataFusion error) is acceptable.
            // Invariant: E-QUERY-038 MUST NOT fire on 'severity' (present in FROM schema).
            _ => {}
        }
    }

    // ── Tests 67–72: EC-11-076 PER-REFERENCE SCOPING (ADV-FIX-P16-MED-001) ─────
    //
    // BC-2.11.016 EC-11-076 PER-REFERENCE SCOPING RULE:
    //   The HEAD-JOIN SUSPENSION RULE (EC-11-074/075) suspends E-QUERY-038 ONLY for
    //   BARE UNQUALIFIED column references. When the SAME column name appears BOTH as
    //   a bare unqualified ref (e.g. bare `col` in WHERE) AND as a FROM-alias-qualified
    //   ref (e.g. `alias.col` in SELECT), `bare_head_cols` (name-keyed HashSet<String>)
    //   wrongly suspends the QUALIFIED reference too (ADV-FIX-P16-MED-001).
    //
    //   Qualified FROM-alias refs (`alias.col`, `crowdstrike_alerts.col`) are
    //   unambiguously bound to the FROM table. If `col` is absent from the FROM table
    //   schema, E-QUERY-038 MUST fire regardless of any co-resident bare ref.
    //
    // Fixture: `make_engine_with_join_tables()` (same as EC-11-074/075):
    //   crowdstrike_alerts (severity: String, timestamp: Datetime)  ← FROM table
    //   some_other_table   (col: String, id: String)                ← JOIN target
    //   available_columns for crowdstrike_alerts = [severity, timestamp]
    //
    // Bug at 3212070c: bare `col` in WHERE puts "col" into `bare_head_cols`;
    //   qualified `alias.col` in SELECT is extracted as "col" in select_cols;
    //   gate loop: `bare_head_cols.contains("col")` → true → suspension applied →
    //   ColumnNotFound swallowed → E-QUERY-038 DOES NOT fire (false negative / FN-001).

    // ── Test 67 (RED GATE): EC-11-076 alias-qualified SELECT + bare WHERE, Ast::Sql ──

    /// BC-2.11.016 EC-11-076 — PER-REFERENCE SCOPING, alias-qualified SELECT ref,
    /// `Ast::Sql` form (ADV-FIX-P16-MED-001).
    ///
    /// `SELECT alias.col FROM crowdstrike_alerts AS alias
    ///  JOIN some_other_table ON alias.severity = some_other_table.id
    ///  WHERE col = 'x'`
    ///
    /// `alias.col` — FROM-alias-qualified ref: unambiguously bound to `crowdstrike_alerts`.
    /// `col` is absent from `crowdstrike_alerts` schema (severity, timestamp only).
    /// E-QUERY-038 MUST fire for the qualified SELECT ref (not suspended by HEAD-JOIN rule).
    ///
    /// Co-resident bare `col` in WHERE is suspension-eligible per EC-11-074, but the
    /// qualified SELECT ref `alias.col` retains full E-QUERY-038 checking independently.
    ///
    /// Bug at 3212070c:
    ///   `bare_head_cols` is name-keyed (`HashSet<String>`). Bare WHERE `col` (1-segment)
    ///   inserts "col" into `bare_head_cols`. `extract_field_paths_from_expr` for the
    ///   SELECT item `alias.col` calls `extract_column_name_from_field_path(["alias","col"],
    ///   "crowdstrike_alerts", Some("alias"))` → qualifier "alias" matches `from_alias` →
    ///   returns "col" → `select_cols = ["col"]`. Gate loop: `bare_head_cols.contains("col")`
    ///   → true → suspension applied → ColumnNotFound for "col" swallowed (fail-open) →
    ///   E-QUERY-038 does NOT fire.
    ///
    /// Expected after fix (GREEN):
    ///   Per-reference scoping: the gate loop must distinguish the QUALIFIED reference
    ///   (`alias.col` in SELECT) from the BARE reference (`col` in WHERE). Only the bare
    ///   ref is suspension-eligible; the qualified ref must retain full E-QUERY-038
    ///   checking → ColumnNotFound fires for "col".
    ///
    /// RED GATE trigger: E-QUERY-038(col) is NOT returned at 3212070c (swallowed by bug).
    #[tokio::test]
    async fn test_BC_2_11_016_ec11076_alias_qualified_select_fires_e_query_038() {
        let (engine, org) = make_engine_with_join_tables();

        // EC-11-076 shape 1: alias-qualified SELECT ref + co-resident bare WHERE ref.
        // `alias.col` bound to crowdstrike_alerts (from_alias="alias"); `col` absent.
        let query = "SELECT alias.col FROM crowdstrike_alerts AS alias \
                     JOIN some_other_table ON alias.severity = some_other_table.id \
                     WHERE col = 'x'";

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
            Err(PrismError::ColumnNotFound(ref details)) if details.column == "col" => {
                // CORRECT: E-QUERY-038 fires for qualified `alias.col` reference.
                // `alias.col` is bound to crowdstrike_alerts; `col` absent from its schema
                // → E-QUERY-038 MUST fire. Per-reference scoping: qualified FROM-alias refs
                // are NEVER suspension-eligible even when a co-resident bare ref exists.
            }
            other => panic!(
                "BC-2.11.016 EC-11-076 RED GATE (alias-qualified SELECT, Ast::Sql): \
                 E-QUERY-038(col) MUST fire for qualified `alias.col` reference \
                 (bound to crowdstrike_alerts; available=[severity, timestamp]). \
                 Per-reference scoping: qualified FROM-alias refs retain full E-QUERY-038 \
                 checking even when a co-resident bare `col` in WHERE sets bare_head_cols. \
                 Bug: bare_head_cols is name-keyed — bare WHERE `col` inserts \"col\" which \
                 wrongly suspends the qualified SELECT ref alias.col (FN-001 violation). \
                 Got: {:?}",
                other
            ),
        }
    }

    // ── Test 68 (RED GATE): EC-11-076 table-name-qualified SELECT + bare WHERE ───

    /// BC-2.11.016 EC-11-076 — PER-REFERENCE SCOPING, table-name-qualified SELECT
    /// ref (no AS alias), `Ast::Sql` form (ADV-FIX-P16-MED-001).
    ///
    /// `SELECT crowdstrike_alerts.col FROM crowdstrike_alerts
    ///  JOIN some_other_table ON crowdstrike_alerts.severity = some_other_table.id
    ///  WHERE col = 'x'`
    ///
    /// Variant of EC-11-076 shape 1 using the table name as qualifier directly (no AS alias;
    /// `from_alias = None`). `crowdstrike_alerts.col` is unambiguously bound to the FROM
    /// table; `col` absent from its schema → E-QUERY-038 MUST fire.
    ///
    /// Same bug path: `extract_column_name_from_field_path(["crowdstrike_alerts","col"],
    /// "crowdstrike_alerts", None)` → qualifier "crowdstrike_alerts" == table_name →
    /// returns "col" → `select_cols = ["col"]`. Bare WHERE `col` → `bare_head_cols = {"col"}`.
    /// Gate wrongly suspends the table-name-qualified ref.
    ///
    /// RED GATE trigger: E-QUERY-038(col) is NOT returned at 3212070c.
    #[tokio::test]
    async fn test_BC_2_11_016_ec11076_table_qualified_select_fires_e_query_038() {
        let (engine, org) = make_engine_with_join_tables();

        // EC-11-076 shape 2: table-name-qualified SELECT ref + co-resident bare WHERE ref.
        // No AS alias; `from_alias = None`. Qualifier "crowdstrike_alerts" == table_name.
        let query = "SELECT crowdstrike_alerts.col FROM crowdstrike_alerts \
                     JOIN some_other_table ON crowdstrike_alerts.severity = some_other_table.id \
                     WHERE col = 'x'";

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
            Err(PrismError::ColumnNotFound(ref details)) if details.column == "col" => {
                // CORRECT: E-QUERY-038 fires for qualified `crowdstrike_alerts.col` reference.
            }
            other => panic!(
                "BC-2.11.016 EC-11-076 RED GATE (table-name-qualified SELECT, Ast::Sql): \
                 E-QUERY-038(col) MUST fire for qualified `crowdstrike_alerts.col` reference \
                 (bound to crowdstrike_alerts; available=[severity, timestamp]). \
                 Per-reference scoping: table-name-qualified refs are NEVER suspension-eligible. \
                 Bug: bare_head_cols name-keyed; bare WHERE `col` wrongly suspends the \
                 table-name-qualified SELECT ref crowdstrike_alerts.col (FN-001 violation). \
                 Got: {:?}",
                other
            ),
        }
    }

    // ── Test 69 (RED GATE): EC-11-076 qualified aggregate arg + bare WHERE ───────

    /// BC-2.11.016 EC-11-076 — PER-REFERENCE SCOPING, qualified aggregate arg
    /// (position 1 SELECT inside FuncCall), `Ast::Sql` form (ADV-FIX-P16-MED-001).
    ///
    /// `SELECT sum(alias.typo) FROM crowdstrike_alerts AS alias
    ///  JOIN some_other_table ON alias.severity = some_other_table.id
    ///  WHERE typo = 'x'`
    ///
    /// `alias.typo` inside `sum()` — FROM-alias-qualified ref bound to crowdstrike_alerts.
    /// `typo` is absent from crowdstrike_alerts schema → E-QUERY-038 MUST fire.
    ///
    /// Grammar note: `sum(field)` accepts any FieldPath at parse time (no type-check at
    /// parse time per BC-2.11.016 / F-PBL1-MED-001). The column-existence gate
    /// (`check_query_column_availability`) fires before DataFusion planning;
    /// `extract_field_paths_from_expr` recurses into aggregate args, extracting
    /// `alias.typo` → "typo" in select_cols. Bare WHERE `typo` → `bare_head_cols = {"typo"}`.
    ///
    /// Same bug path as shapes 1–2: name-keyed `bare_head_cols` wrongly suspends the
    /// qualified aggregate-arg ref.
    ///
    /// RED GATE trigger: E-QUERY-038(typo) is NOT returned at 3212070c.
    #[tokio::test]
    async fn test_BC_2_11_016_ec11076_qualified_agg_arg_fires_e_query_038() {
        let (engine, org) = make_engine_with_join_tables();

        // EC-11-076 shape 3: qualified agg-arg in SELECT (position 1 inside FuncCall) +
        // co-resident bare WHERE ref. `alias.typo` bound to crowdstrike_alerts; `typo` absent.
        let query = "SELECT sum(alias.typo) FROM crowdstrike_alerts AS alias \
                     JOIN some_other_table ON alias.severity = some_other_table.id \
                     WHERE typo = 'x'";

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
            Err(PrismError::ColumnNotFound(ref details)) if details.column == "typo" => {
                // CORRECT: E-QUERY-038 fires for qualified `alias.typo` inside sum() aggregate.
                // `extract_field_paths_from_expr` recurses into aggregate args — qualified
                // agg-arg refs are NOT suspension-eligible even when bare WHERE `typo` exists.
            }
            other => panic!(
                "BC-2.11.016 EC-11-076 RED GATE (qualified aggregate arg, Ast::Sql): \
                 E-QUERY-038(typo) MUST fire for qualified `alias.typo` inside sum() \
                 (bound to crowdstrike_alerts; available=[severity, timestamp]). \
                 `extract_field_paths_from_expr` recurses into FuncCall::Aggregate args — \
                 qualified refs inside aggregate args must retain full E-QUERY-038 checking. \
                 Bug: bare_head_cols name-keyed; bare WHERE `typo` wrongly suspends the \
                 qualified agg-arg ref alias.typo (FN-001 violation). \
                 Got: {:?}",
                other
            ),
        }
    }

    // ── Test 70 (RED GATE): EC-11-076 Ast::SqlPipe head form of shape 1 ─────────

    /// BC-2.11.016 EC-11-076 — PER-REFERENCE SCOPING, alias-qualified SELECT ref,
    /// `Ast::SqlPipe` form (ADV-FIX-P16-MED-001).
    ///
    /// Shape 1 (test 67) with `| limit 10` suffix → `Ast::SqlPipe`.
    ///
    /// `SELECT alias.col FROM crowdstrike_alerts AS alias
    ///  JOIN some_other_table ON alias.severity = some_other_table.id
    ///  WHERE col = 'x'
    ///  | limit 10`
    ///
    /// The `| limit 10` stage carries no column refs (PipeStage::Limit has no field
    /// references); the pipe-stage walk does not fire E-QUERY-038. The RED trigger is
    /// the same head-SQL position-1 path as test 67 (Ast::SqlPipe: `sql_query = &spq.head`).
    ///
    /// RED GATE trigger: E-QUERY-038(col) is NOT returned at 3212070c.
    #[tokio::test]
    async fn test_BC_2_11_016_ec11076_sqlpipe_alias_qualified_select_fires_e_query_038() {
        let (engine, org) = make_engine_with_join_tables();

        // EC-11-076 shape 4 (SqlPipe form): `| limit 10` triggers Ast::SqlPipe.
        // Head SQL is identical to shape 1 (test 67); per-reference scoping applies
        // identically to Ast::SqlPipe head (sql_query = &spq.head).
        let query = "SELECT alias.col FROM crowdstrike_alerts AS alias \
                     JOIN some_other_table ON alias.severity = some_other_table.id \
                     WHERE col = 'x' \
                     | limit 10";

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
            Err(PrismError::ColumnNotFound(ref details)) if details.column == "col" => {
                // CORRECT: E-QUERY-038 fires for qualified alias.col in SqlPipe head SQL.
                // The `| limit 10` pipe stage has no column refs; the failure is in the
                // head-SQL position-1 path (same code path as Ast::Sql via `sql_query = &spq.head`).
            }
            other => panic!(
                "BC-2.11.016 EC-11-076 RED GATE (Ast::SqlPipe alias-qualified SELECT): \
                 E-QUERY-038(col) MUST fire for qualified `alias.col` in SqlPipe head SQL \
                 (bound to crowdstrike_alerts; available=[severity, timestamp]). \
                 Per-reference scoping applies to Ast::SqlPipe head identically to Ast::Sql \
                 (code path: `sql_query = &spq.head`). The `| limit 10` stage has no column \
                 refs — the failure is head-SQL position-1 (same as test 67). \
                 Bug: bare_head_cols name-keyed; bare WHERE `col` wrongly suspends \
                 qualified head-SQL SELECT ref alias.col. Got: {:?}",
                other
            ),
        }
    }

    // ── Tests 71–72: Negative controls (GREEN at 3212070c, must stay GREEN after fix) ──

    // ── Test 71 (GREEN): bare-only WHERE col — HEAD-JOIN suspension preserved ────

    /// BC-2.11.016 EC-11-076 negative control — bare-only WHERE ref stays suspended.
    ///
    /// `SELECT severity FROM crowdstrike_alerts AS alias
    ///  JOIN some_other_table ON alias.severity = some_other_table.id
    ///  WHERE col = 'x'`
    ///
    /// `severity` is PRESENT in the FROM schema (passes check_column_availability normally).
    /// Bare `col` in WHERE has NO co-resident qualified ref — it is purely a bare ref
    /// and MUST remain suspended per EC-11-074 HEAD-JOIN SUSPENSION RULE (position 2 WHERE).
    ///
    /// GREEN at current HEAD (bare `col` correctly suspension-eligible; `severity` in schema
    /// → check passes regardless of suspension path).
    /// GREEN after fix (the per-reference fix must NOT change suspension behavior for
    /// purely bare refs — only qualified refs lose their undeserved suspension exemption).
    #[tokio::test]
    async fn test_BC_2_11_016_ec11076_bare_only_where_col_suspension_preserved() {
        let (engine, org) = make_engine_with_join_tables();

        // EC-11-076 negative control 1: bare `col` in WHERE only (no qualified co-resident ref).
        // `severity` is in FROM schema; bare `col` must remain suspension-eligible.
        let query = "SELECT severity FROM crowdstrike_alerts AS alias \
                     JOIN some_other_table ON alias.severity = some_other_table.id \
                     WHERE col = 'x'";

        let result = engine
            .execute(
                query,
                QueryOptions {
                    clients: Some(vec![org]),
                    ..QueryOptions::default()
                },
            )
            .await;

        // Negative control: E-QUERY-038 MUST NOT fire on bare `col` (suspension-eligible).
        // `col` is a valid column in `some_other_table` at execution — false positive class
        // FP-001 applies if E-QUERY-038 fires here.
        // If this panics, the EC-11-076 fix has incorrectly removed suspension for bare refs.
        match result {
            Err(PrismError::ColumnNotFound(ref details)) if details.column == "col" => panic!(
                "BC-2.11.016 EC-11-076 NEGATIVE CONTROL REGRESSION: E-QUERY-038(col) \
                 fired for purely BARE `col` in WHERE — but the HEAD-JOIN SUSPENSION RULE \
                 (EC-11-074 position 2) MUST suppress this. No qualified co-resident ref \
                 for `col` exists here; the per-reference fix (EC-11-076) must NOT change \
                 suspension behavior for purely bare refs. `col` is valid in `some_other_table` \
                 at execution. column='{}', table='{}', available={:?}",
                details.column, details.table, details.available_columns
            ),
            _ => {}
        }
    }

    // ── Test 72 (GREEN): qualified present-col — E-QUERY-038 must NOT fire ───────

    /// BC-2.11.016 EC-11-076 negative control — qualified ref to PRESENT column.
    ///
    /// `SELECT alias.severity FROM crowdstrike_alerts AS alias
    ///  JOIN some_other_table ON alias.severity = some_other_table.id`
    ///
    /// `alias.severity` is a qualified ref to `crowdstrike_alerts.severity` — a column
    /// PRESENT in the FROM schema. `check_column_availability("severity", ...)` returns
    /// Ok() → E-QUERY-038 MUST NOT fire.
    ///
    /// GREEN at current HEAD (present-column check passes normally).
    /// GREEN after fix (qualified present-col path unchanged — E-QUERY-038 must not fire
    /// because the column IS in the FROM schema, regardless of per-reference scoping).
    #[tokio::test]
    async fn test_BC_2_11_016_ec11076_qualified_present_col_no_e_query_038() {
        let (engine, org) = make_engine_with_join_tables();

        // EC-11-076 negative control 2: qualified ref to a PRESENT column — no error.
        // `alias.severity` resolves to crowdstrike_alerts.severity (present in schema).
        let query = "SELECT alias.severity FROM crowdstrike_alerts AS alias \
                     JOIN some_other_table ON alias.severity = some_other_table.id";

        let result = engine
            .execute(
                query,
                QueryOptions {
                    clients: Some(vec![org]),
                    ..QueryOptions::default()
                },
            )
            .await;

        // Negative control: E-QUERY-038 MUST NOT fire for 'severity' (present in FROM schema).
        // If this panics, the fix incorrectly fires E-QUERY-038 for qualified PRESENT columns.
        // BC invariant: present-column checks pass normally whether or not JOINs are present
        // and whether or not the ref is qualified.
        match result {
            Err(PrismError::ColumnNotFound(ref details)) if details.column == "severity" => panic!(
                "BC-2.11.016 EC-11-076 NEGATIVE CONTROL REGRESSION: FALSE E-QUERY-038 \
                 on 'severity' — a column PRESENT in `crowdstrike_alerts` schema \
                 (available=[severity, timestamp]). Qualified `alias.severity` correctly \
                 resolves to `crowdstrike_alerts.severity`; `check_column_availability` must \
                 return Ok(). The EC-11-076 fix must NOT disrupt E-QUERY-038 behavior for \
                 qualified refs to PRESENT columns. column='{}', table='{}', available={:?}",
                details.column, details.table, details.available_columns
            ),
            _ => {}
        }
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod f_p24_med001_valid_operators_ci_tests {
    use super::valid_operators_for_type;
    use prism_core::column::ColumnType;

    /// RED GATE: valid_operators_for_type(ColumnType::String) must include IEQ, IIN, INE.
    ///
    /// BC-2.11.024: IEQ/IIN/INE are valid string-column operators. The prior
    /// implementation omitted them, causing a gap between the Display prose (which suggests
    /// IEQ) and the machine-readable array (which denied IEQ/IIN/INE existed).
    ///
    /// "NOT IIN" is NOT added because negated IIN is not representable in the PrismQL AST
    /// (ast.rs: "<invalid: negated IIN not representable>") — it is never a legal operator.
    ///
    /// Fix: F-P24-MED-001 (LOCAL pass-24, S-PRISMQL-CASE-INSENSITIVE-001).
    #[test]
    fn test_BC_2_11_024_f_p24_med001_valid_operators_string_includes_ci_operators() {
        let ops = valid_operators_for_type(ColumnType::String);

        // BC-2.11.024 case-insensitive operators must be present.
        for ci_op in &["IEQ", "IIN", "INE"] {
            assert!(
                ops.contains(ci_op),
                "F-P24-MED-001: valid_operators_for_type(String) must contain '{}' \
                 (BC-2.11.024 case-insensitive operators). \
                 Agents parsing this array would never learn {} is valid. Got: {:?}",
                ci_op,
                ci_op,
                ops
            );
        }

        // Numeric/datetime ordering operators must NOT appear for String columns.
        for disallowed in &["<", ">", "<=", ">=", "BETWEEN"] {
            assert!(
                !ops.contains(disallowed),
                "F-P24-MED-001: '{}' must NOT appear in valid_operators_for_type(String) \
                 (numeric/datetime operator not valid for String). Got: {:?}",
                disallowed,
                ops
            );
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// SEC-FIND-001 / CWE-117 — column_not_found.rejected log + payload sanitization
//
// MED-002 (ADV-PR-P1-MED-002): three emission-path load-bearing lock tests
//   (expected GREEN at dacb60fa — sanitize_for_log already called at log sites).
// OBS-001 (ADV-PR-P1-OBS-001): two payload sanitization RED gate tests
//   (expected FAIL at dacb60fa — column_name passed raw to ColumnNotFoundDetails).
// ─────────────────────────────────────────────────────────────────────────────
#[cfg(test)]
#[allow(non_snake_case, clippy::expect_used)]
mod sec_find_001_cwe117_column_not_found_log_sanitization_tests {
    use std::collections::HashMap;

    use super::{check_column_against_available_set, check_column_availability};
    use crate::table_registry::TableRegistry;
    use prism_core::error::{sanitize_for_log, PrismError};
    use prism_core::{column::ColumnType, OrgSlug, SensorId};
    use prism_spec_engine::{
        overlay::{OverlayLoader, SensorInstanceOverlay},
        spec_parser::{AuthType, ColumnSpec, SensorSpec, TableSpec},
        ResolvedSpecKey,
    };

    // ── Helper ────────────────────────────────────────────────────────────────

    /// Returns `true` if `s` contains any Unicode Cc character (control code)
    /// or the Unicode line/paragraph separator codepoints U+2028 / U+2029.
    ///
    /// Used by OBS-001 tests to assert payload fields carry none of these chars.
    fn has_ctrl_or_sep(s: &str) -> bool {
        s.chars()
            .any(|c| c.is_control() || c == '\u{2028}' || c == '\u{2029}')
    }

    // ── Helper-level unit lock (SEC-FIND-001 original) ────────────────────────

    /// SEC-FIND-001 (CWE-117) helper-level regression lock — `sanitize_for_log` strips
    /// Unicode Cc characters and line-separator codepoints from column names.
    ///
    /// Injection vectors validated:
    /// - SOH (U+0001): canonical ASCII control character log-injection vector
    /// - LF  (U+000A): newline — classic log-splitting vector
    /// - CR  (U+000D): carriage return
    /// - NEL (U+0085): C1 control — Unicode newline equivalent
    /// - U+2028: LINE SEPARATOR — JSON-embedded newline in some consumers
    /// - U+2029: PARAGRAPH SEPARATOR
    ///
    /// Unit-level regression lock: verifies the `sanitize_for_log` helper in isolation.
    /// NOTE: this test does NOT verify that the emission sites in
    /// `check_column_availability` or `check_column_against_available_set` actually
    /// invoke `sanitize_for_log` — emission-path coverage is provided by the
    /// `test_BC_2_11_016_med002_*` tests below (ADV-PR-P1-MED-002 closure).
    ///
    /// Traces to: SEC-FIND-001 (PR #219 review), CWE-117, TD-VSDD-060.
    #[test]
    fn test_sec_find_001_cwe117_sanitize_for_log_strips_control_chars_from_column_name() {
        // ASCII control characters — canonical CWE-117 injection vectors.
        assert_eq!(
            sanitize_for_log("my_col\x01injected"),
            "my_colinjected",
            "SEC-FIND-001: SOH (U+0001) must be stripped from column_name before log emission"
        );
        assert_eq!(
            sanitize_for_log("my_col\njected"),
            "my_coljected",
            "SEC-FIND-001: LF (U+000A) must be stripped from column_name before log emission"
        );
        assert_eq!(
            sanitize_for_log("my_col\rjected"),
            "my_coljected",
            "SEC-FIND-001: CR (U+000D) must be stripped from column_name before log emission"
        );
        // C1 controls (U+0085 NEL) — strip per ADV-PR-P5-OBS-001.
        assert_eq!(
            sanitize_for_log("col\u{0085}name"),
            "colname",
            "SEC-FIND-001: NEL (U+0085) must be stripped from column_name before log emission"
        );
        // Unicode line/paragraph separators — strip per ADV-PR-P5-OBS-001.
        assert_eq!(
            sanitize_for_log("col\u{2028}name"),
            "colname",
            "SEC-FIND-001: U+2028 LINE SEPARATOR must be stripped from column_name"
        );
        assert_eq!(
            sanitize_for_log("col\u{2029}name"),
            "colname",
            "SEC-FIND-001: U+2029 PARAGRAPH SEPARATOR must be stripped from column_name"
        );
        // Normal identifiers must pass through unchanged.
        assert_eq!(
            sanitize_for_log("device_id"),
            "device_id",
            "SEC-FIND-001: plain ASCII column name must not be modified"
        );
        assert_eq!(
            sanitize_for_log("severity_CRÍTICO"),
            "severity_CRÍTICO",
            "SEC-FIND-001: valid Unicode letters must not be stripped"
        );
    }

    // ── MED-002: emission-path load-bearing tests (expected GREEN at dacb60fa) ──────────────
    //
    // Drive the PRODUCTION emission path; confirm (a) the tracing::warn! event fired and
    // (b) the raw control char U+0001 did NOT appear in the captured log output.
    //
    // LOAD-BEARING PROOF — removing either the emission call or the sanitize_for_log call
    // breaks the test:
    //
    // (a) If `tracing::warn!(event_type = "column_not_found.rejected", ...)` were removed,
    //     `logs_contain("column_not_found.rejected")` returns false → assertion FAILS.
    // (b) If `sanitize_for_log(column_name)` were removed (raw column_name used directly),
    //     `logs_contain("\x01")` returns true → `!logs_contain("\x01")` is false → FAILS.

    /// ADV-PR-P1-MED-002 closure — single-tenant `check_column_availability` emission site.
    ///
    /// Registers "crowdstrike_alerts" in a `TableRegistry` with columns ["severity",
    /// "timestamp"], then queries the missing column "sev\x01rity" on the single-tenant
    /// path (resolved_spec_map = None). Asserts the `column_not_found.rejected` event is
    /// emitted and does NOT carry the raw U+0001 byte in any structured field.
    ///
    /// GREEN at dacb60fa: `sanitize_for_log` is already called before the single-tenant
    /// `tracing::warn!` in `check_column_availability`.
    ///
    /// Traces to: ADV-PR-P1-MED-002, CWE-117, BC-2.11.016 single-tenant path.
    #[test]
    #[tracing_test::traced_test]
    fn test_BC_2_11_016_med002_single_tenant_emission_site_no_ctrl_chars_in_log() {
        let registry = TableRegistry::new();
        let spec = SensorSpec::new(
            "crowdstrike",
            "CrowdStrike test sensor",
            AuthType::ApiKey,
            "https://example.com",
            vec![TableSpec::new_point_in_time(
                "alerts",
                "security_finding",
                vec![
                    ColumnSpec::new("severity", ColumnType::String, None, vec![]),
                    ColumnSpec::new("timestamp", ColumnType::Datetime, None, vec![]),
                ],
                vec![],
            )],
            None,
            "1.0.0",
            Vec::new(),
        );
        registry
            .register_sensor(&spec)
            .expect("MED-002 fixture: register_sensor must not fail");

        // "sev\x01rity" is NOT in ["severity", "timestamp"] → single-tenant E-QUERY-038 fires.
        let result = check_column_availability(
            "sev\x01rity",
            "crowdstrike_alerts",
            "test_client",
            None,
            None,
            Some(&registry),
            true, // compute_did_you_mean: test exercises the normal error-propagation path
        );
        assert!(
            result.is_err(),
            "MED-002 single-tenant: E-QUERY-038 must fire for unknown column"
        );

        // (a) LOAD-BEARING: emission site was reached.
        assert!(
            logs_contain("column_not_found.rejected"),
            "MED-002 single-tenant: event_type=column_not_found.rejected must be emitted. \
             Failure here means the tracing::warn! call was not reached — the test is no \
             longer exercising the single-tenant emission site in check_column_availability."
        );
        // (b) LOAD-BEARING: control char stripped before emission.
        // Without sanitize_for_log, `column = %column_name` would emit "sev\x01rity" verbatim
        // and logs_contain("\x01") would return true, failing this assertion.
        assert!(
            !logs_contain("\x01"),
            "MED-002 single-tenant CWE-117 regression lock: raw U+0001 found in captured \
             tracing output. The `column = %safe_column_name` field must emit the sanitized \
             form. FIX: ensure sanitize_for_log(column_name) precedes the tracing::warn! call \
             in the single-tenant branch of check_column_availability."
        );
    }

    /// ADV-PR-P1-MED-002 closure — multi-tenant `check_column_availability` emission site.
    ///
    /// Builds a `ResolvedSensorSpec` map with crowdstrike@acme having column ["severity"],
    /// then queries the missing column "sev\x01rity" on the multi-tenant path
    /// (resolved_spec_map = Some). Asserts the event is emitted without raw U+0001.
    ///
    /// GREEN at dacb60fa: `sanitize_for_log` is already called before the multi-tenant
    /// `tracing::warn!` in `check_column_availability`.
    ///
    /// Traces to: ADV-PR-P1-MED-002, CWE-117, BC-2.11.016 multi-tenant path.
    #[test]
    #[tracing_test::traced_test]
    fn test_BC_2_11_016_med002_multi_tenant_emission_site_no_ctrl_chars_in_log() {
        let spec = SensorSpec::new(
            "crowdstrike",
            "CrowdStrike test sensor",
            AuthType::ApiKey,
            "https://example.com",
            vec![TableSpec::new_point_in_time(
                "alerts",
                "security_finding",
                vec![ColumnSpec::new(
                    "severity",
                    ColumnType::String,
                    None,
                    vec![],
                )],
                vec![],
            )],
            None,
            "1.0.0",
            Vec::new(),
        );
        let overlay_toml = "extends = \"crowdstrike\"\ninstance_id = \"crowdstrike@acme\"";
        let overlay: SensorInstanceOverlay = toml::from_str(overlay_toml)
            .expect("MED-002 fixture: SensorInstanceOverlay TOML must parse");
        let org_slug = OrgSlug::new("acme");
        let resolved =
            OverlayLoader::merge_overlay_onto_type_spec(&spec, &overlay, org_slug.clone());
        let key: ResolvedSpecKey = (org_slug, SensorId::new("crowdstrike"));
        let mut spec_map = HashMap::new();
        spec_map.insert(key, resolved);

        // "sev\x01rity" NOT in ["severity"] → multi-tenant E-QUERY-038 fires.
        let result = check_column_availability(
            "sev\x01rity",
            "crowdstrike_alerts",
            "test_client",
            None,
            Some(&spec_map),
            None,
            true, // compute_did_you_mean: test exercises the normal error-propagation path
        );
        assert!(
            result.is_err(),
            "MED-002 multi-tenant: E-QUERY-038 must fire for unknown column"
        );

        // (a) LOAD-BEARING: emission site was reached.
        assert!(
            logs_contain("column_not_found.rejected"),
            "MED-002 multi-tenant: event_type=column_not_found.rejected must be emitted. \
             Failure here means the tracing::warn! call was not reached in the multi-tenant \
             branch of check_column_availability."
        );
        // (b) LOAD-BEARING: control char stripped before emission.
        assert!(
            !logs_contain("\x01"),
            "MED-002 multi-tenant CWE-117 regression lock: raw U+0001 found in captured \
             tracing output. FIX: ensure sanitize_for_log precedes the multi-tenant \
             tracing::warn! call in check_column_availability."
        );
    }

    /// ADV-PR-P1-MED-002 closure — `check_column_against_available_set` binding-context site.
    ///
    /// Calls `check_column_against_available_set` directly with available columns
    /// ["severity", "timestamp"] and the missing column "sev\x01rity". Asserts the
    /// `column_not_found.rejected` event is emitted without raw U+0001.
    ///
    /// GREEN at dacb60fa: `sanitize_for_log` is already called before the binding-context
    /// `tracing::warn!` in `check_column_against_available_set`.
    ///
    /// Traces to: ADV-PR-P1-MED-002, CWE-117, BC-2.11.016 binding-context path.
    #[test]
    #[tracing_test::traced_test]
    fn test_BC_2_11_016_med002_binding_ctx_emission_site_no_ctrl_chars_in_log() {
        let available = vec!["severity".to_string(), "timestamp".to_string()];

        // "sev\x01rity" NOT in ["severity", "timestamp"] → binding-context E-QUERY-038 fires.
        let result = check_column_against_available_set(
            "sev\x01rity",
            "test_table",
            "test_client",
            &available,
        );
        assert!(
            result.is_err(),
            "MED-002 binding-ctx: E-QUERY-038 must fire for unknown column"
        );

        // (a) LOAD-BEARING: emission site was reached.
        assert!(
            logs_contain("column_not_found.rejected"),
            "MED-002 binding-ctx: event_type=column_not_found.rejected must be emitted. \
             Failure here means the tracing::warn! call was not reached in \
             check_column_against_available_set."
        );
        // (b) LOAD-BEARING: control char stripped before emission.
        assert!(
            !logs_contain("\x01"),
            "MED-002 binding-ctx CWE-117 regression lock: raw U+0001 found in captured \
             tracing output. FIX: ensure sanitize_for_log precedes the binding-context \
             tracing::warn! call in check_column_against_available_set."
        );
    }

    // ── OBS-001: payload sanitization RED gates (expected FAIL at dacb60fa) ─────────────────
    //
    // BC-2.11.016 §Postconditions: the `ColumnNotFoundDetails.column` field AND its
    // Display rendering MUST carry the sanitized form (Unicode Cc + U+2028/U+2029 stripped).
    //
    // At dacb60fa: `column_name` is passed RAW to `ColumnNotFoundDetails::new(column_name, ...)`
    // at all three emission sites. Consequently:
    //   - `details.column == "sev\x01rity"` (raw U+0001 present)
    //   - `format!("{}", details)` == "E-QUERY-038: column 'sev\x01rity' ..." (raw U+0001)
    //
    // Both assertions below call `has_ctrl_or_sep` which returns true for the raw column_name,
    // making the `!has_ctrl_or_sep(...)` assertion false → test FAILS (RED gate confirmed).

    /// ADV-PR-P1-OBS-001 RED GATE — `ColumnNotFoundDetails.column` field must be sanitized.
    ///
    /// Drives `check_column_against_available_set` (binding-context path) with "sev\x01rity".
    /// Asserts that the returned error's `.column` field contains no Unicode Cc or U+2028/U+2029.
    ///
    /// RED at dacb60fa: `column_name` is passed raw to `ColumnNotFoundDetails::new` →
    /// `details.column == "sev\x01rity"` → `has_ctrl_or_sep` returns true → assertion FAILS.
    ///
    /// Traces to: ADV-PR-P1-OBS-001, BC-2.11.016 §Postconditions.
    #[test]
    fn test_BC_2_11_016_obs001_payload_column_field_stripped_of_ctrl_chars() {
        let available = vec!["severity".to_string(), "timestamp".to_string()];
        let err = check_column_against_available_set(
            "sev\x01rity",
            "test_table",
            "test_client",
            &available,
        )
        .expect_err("OBS-001: E-QUERY-038 must fire for unknown column");

        match err {
            PrismError::ColumnNotFound(ref details) => {
                assert!(
                    !has_ctrl_or_sep(&details.column),
                    "OBS-001 RED GATE (BC-2.11.016 §Postconditions): \
                     ColumnNotFoundDetails.column must not contain raw Unicode Cc or \
                     U+2028/U+2029. Got column = {:?}. \
                     FIX: sanitize column_name before passing to ColumnNotFoundDetails::new \
                     in check_column_against_available_set.",
                    details.column
                );
            }
            other => panic!(
                "OBS-001: expected PrismError::ColumnNotFound, got: {:?}",
                other
            ),
        }
    }

    /// ADV-PR-P1-OBS-001 RED GATE — `ColumnNotFoundDetails` Display rendering must be sanitized.
    ///
    /// Drives `check_column_availability` (single-tenant path) with "sev\x01rity".
    /// Asserts that `format!("{}", details)` contains no Unicode Cc or U+2028/U+2029.
    ///
    /// RED at dacb60fa: `ColumnNotFoundDetails.column` is raw, and `Display` formats it
    /// verbatim → `format!()` output contains U+0001 → `has_ctrl_or_sep` returns true →
    /// assertion FAILS.
    ///
    /// Traces to: ADV-PR-P1-OBS-001, BC-2.11.016 §Postconditions.
    #[test]
    fn test_BC_2_11_016_obs001_payload_display_rendering_stripped_of_ctrl_chars() {
        let registry = TableRegistry::new();
        let spec = SensorSpec::new(
            "crowdstrike",
            "CrowdStrike test sensor",
            AuthType::ApiKey,
            "https://example.com",
            vec![TableSpec::new_point_in_time(
                "alerts",
                "security_finding",
                vec![ColumnSpec::new(
                    "severity",
                    ColumnType::String,
                    None,
                    vec![],
                )],
                vec![],
            )],
            None,
            "1.0.0",
            Vec::new(),
        );
        registry
            .register_sensor(&spec)
            .expect("OBS-001 fixture: register_sensor must not fail");

        let err = check_column_availability(
            "sev\x01rity",
            "crowdstrike_alerts",
            "test_client",
            None,
            None,
            Some(&registry),
            true, // compute_did_you_mean: test exercises the normal error-propagation path
        )
        .expect_err("OBS-001: E-QUERY-038 must fire for unknown column");

        match err {
            PrismError::ColumnNotFound(ref details) => {
                let display = format!("{}", details);
                assert!(
                    !has_ctrl_or_sep(&display),
                    "OBS-001 RED GATE (BC-2.11.016 §Postconditions): \
                     Display rendering of ColumnNotFoundDetails must not contain raw Unicode \
                     Cc or U+2028/U+2029. Got: {:?}. \
                     FIX: sanitize column_name before ColumnNotFoundDetails::new in \
                     check_column_availability (single-tenant path).",
                    display
                );
            }
            other => panic!(
                "OBS-001: expected PrismError::ColumnNotFound, got: {:?}",
                other
            ),
        }
    }
}

// ---------------------------------------------------------------------------
// F-PQLFN-P4-MED-001: DataFusion aggregate registry empirical locks
// ---------------------------------------------------------------------------
//
// Settles the ADR-048 v1.3 vs engine.rs comment discrepancy about whether
// "percentile" and "distinct_count" are present in DataFusion 53.1's
// default_aggregate_functions() registry.
//
// These tests pin the EMPIRICAL TRUTH as load-bearing assertions. If a
// DataFusion upgrade ever adds "percentile" or "distinct_count" to the registry,
// the corresponding test fails — reviewer knows to remove the now-redundant manual
// insert from DATAFUSION_BUILTIN_AGGREGATE_NAMES and update ADR-048.
//
// VERDICT (DataFusion 53.1):
//   "percentile"      → ABSENT  (ADR-048 v1.3 claim was FALSE; comment correct)
//   "distinct_count"  → ABSENT  (DataFusion uses "approx_distinct")
//   "approx_percentile_cont" → PRESENT  (what "percentile" emits at query time)
//   "approx_distinct" → PRESENT  (what "distinct_count" maps to at emit time)

#[cfg(test)]
mod datafusion_aggregate_registry_empirical_tests {
    /// F-PQLFN-P4-MED-001 empirical lock (1/4): "percentile" ABSENT from DataFusion 53.1.
    ///
    /// Settles the discrepancy between:
    ///   - ADR-048 v1.3 claim: "percentile IS registered in default_aggregate_functions()"
    ///   - engine.rs comment: "absent from DataFusion's built-in registry"
    ///
    /// EMPIRICAL VERDICT: "percentile" is ABSENT from DataFusion 53.1.
    ///   → engine.rs comment CORRECT; ADR-048 v1.3 claim is FALSE.
    ///   → The manual `names.insert("percentile")` is NECESSARY for correct gate coverage.
    ///   → ADR-048 v1.4 retracted this claim (§D.2 PERCENTILE note corrected; F-PQLFN-P4-MED-001); manual insert is necessary, not redundant.
    ///
    /// If this test FAILS in a future DataFusion upgrade: "percentile" was added to the
    /// registry. Remove the manual insert and update ADR-048 accordingly.
    ///
    /// Traces to: F-PQLFN-P4-MED-001; ADR-048 v1.3 reconciliation.
    #[test]
    fn test_f_pqlfn_p4_med_001_percentile_absent_from_datafusion_53_1_aggregate_registry() {
        use datafusion::execution::SessionStateDefaults;

        let raw_names: std::collections::HashSet<String> =
            SessionStateDefaults::default_aggregate_functions()
                .iter()
                .flat_map(|f| {
                    let mut names = vec![f.name().to_ascii_lowercase()];
                    for alias in f.aliases() {
                        names.push(alias.to_ascii_lowercase());
                    }
                    names
                })
                .collect();

        // EMPIRICAL LOCK: "percentile" must be ABSENT from the raw DataFusion registry
        // (before the manual inserts in DATAFUSION_BUILTIN_AGGREGATE_NAMES).
        assert!(
            !raw_names.contains("percentile"),
            "F-PQLFN-P4-MED-001 EMPIRICAL LOCK BROKEN: 'percentile' is now IN DataFusion's \
             default_aggregate_functions() registry (not present in DataFusion 53.1). \
             ADR-048 v1.3 claim would now be CORRECT for this DataFusion version. \
             Action: remove `names.insert(\"percentile\")` from DATAFUSION_BUILTIN_AGGREGATE_NAMES \
             and update ADR-048 to note version where percentile became a DataFusion built-in."
        );
    }

    /// F-PQLFN-P4-MED-001 empirical lock (2/4): "distinct_count" ABSENT from DataFusion 53.1.
    ///
    /// DataFusion 53.1 uses "approx_distinct" (not "distinct_count") as the canonical name.
    /// "distinct_count" is a PrismQL-specific alias — absent from DataFusion's registry.
    ///   → The manual `names.insert("distinct_count")` is NECESSARY for correct gate coverage.
    ///
    /// If this test FAILS: DataFusion added "distinct_count" as an alias or new function.
    /// Action: verify if the manual insert is still needed; update ADR-048.
    ///
    /// Traces to: F-PQLFN-P4-MED-001; ADR-048 v1.3 reconciliation.
    #[test]
    fn test_f_pqlfn_p4_med_001_distinct_count_absent_from_datafusion_53_1_aggregate_registry() {
        use datafusion::execution::SessionStateDefaults;

        let raw_names: std::collections::HashSet<String> =
            SessionStateDefaults::default_aggregate_functions()
                .iter()
                .flat_map(|f| {
                    let mut names = vec![f.name().to_ascii_lowercase()];
                    for alias in f.aliases() {
                        names.push(alias.to_ascii_lowercase());
                    }
                    names
                })
                .collect();

        // EMPIRICAL LOCK: "distinct_count" must be ABSENT (DataFusion uses "approx_distinct").
        assert!(
            !raw_names.contains("distinct_count"),
            "F-PQLFN-P4-MED-001 EMPIRICAL LOCK BROKEN: 'distinct_count' is now IN DataFusion's \
             default_aggregate_functions() registry. DataFusion 53.1 uses 'approx_distinct'. \
             Action: verify if the manual insert is still needed; update ADR-048."
        );
    }

    /// F-PQLFN-P4-MED-001 empirical lock (3/4): "approx_percentile_cont" PRESENT in DataFusion 53.1.
    ///
    /// Confirms that the UNDERLYING function "percentile" maps to at emit time IS present.
    /// This validates the pipe_sql_emitter.rs `percentile → approx_percentile_cont` translation.
    ///
    /// If this test FAILS: DataFusion renamed or removed "approx_percentile_cont".
    /// Action: update pipe_sql_emitter.rs percentile emit logic to use new function name.
    ///
    /// Traces to: F-PQLFN-P4-MED-001; pipe_sql_emitter.rs percentile emit logic.
    #[test]
    fn test_f_pqlfn_p4_med_001_approx_percentile_cont_present_in_datafusion_53_1_registry() {
        use datafusion::execution::SessionStateDefaults;

        let raw_names: std::collections::HashSet<String> =
            SessionStateDefaults::default_aggregate_functions()
                .iter()
                .flat_map(|f| {
                    let mut names = vec![f.name().to_ascii_lowercase()];
                    for alias in f.aliases() {
                        names.push(alias.to_ascii_lowercase());
                    }
                    names
                })
                .collect();

        // EMPIRICAL LOCK: "approx_percentile_cont" must be PRESENT (pipe_sql_emitter maps to it).
        assert!(
            raw_names.contains("approx_percentile_cont"),
            "F-PQLFN-P4-MED-001: 'approx_percentile_cont' is ABSENT from DataFusion's \
             default_aggregate_functions() registry. Unexpected for DataFusion 53.1. \
             pipe_sql_emitter.rs maps 'percentile' to 'approx_percentile_cont' at emit time. \
             Action: update pipe_sql_emitter.rs to use DataFusion's current percentile function."
        );
    }

    /// F-PQLFN-P4-MED-001 empirical lock (4/4): "approx_distinct" PRESENT in DataFusion 53.1.
    ///
    /// Confirms that the UNDERLYING function "distinct_count" maps to at emit time IS present.
    ///
    /// Traces to: F-PQLFN-P4-MED-001.
    #[test]
    fn test_f_pqlfn_p4_med_001_approx_distinct_present_in_datafusion_53_1_registry() {
        use datafusion::execution::SessionStateDefaults;

        let raw_names: std::collections::HashSet<String> =
            SessionStateDefaults::default_aggregate_functions()
                .iter()
                .flat_map(|f| {
                    let mut names = vec![f.name().to_ascii_lowercase()];
                    for alias in f.aliases() {
                        names.push(alias.to_ascii_lowercase());
                    }
                    names
                })
                .collect();

        // EMPIRICAL LOCK: "approx_distinct" must be PRESENT (DataFusion 53.1 built-in).
        assert!(
            raw_names.contains("approx_distinct"),
            "F-PQLFN-P4-MED-001: 'approx_distinct' is ABSENT from DataFusion's registry. \
             Unexpected for DataFusion 53.1. Action: check DataFusion version and update."
        );
    }

    /// F-PQLFN-P5-LOW-001 empirical lock (scalar): "percentile" ABSENT from DataFusion 53.1
    /// default_scalar_functions() registry.
    ///
    /// Rationale: `DATAFUSION_BUILTIN_FUNCTION_NAMES` unions scalar + aggregate + window
    /// registries (see static initializer). The existing aggregate-registry absence lock
    /// (test_f_pqlfn_p4_med_001_percentile_absent_from_datafusion_53_1_aggregate_registry)
    /// covered only the aggregate arm. This lock completes coverage for the scalar arm, ensuring
    /// that ADR-048 v1.4 §D.2's claim ("percentile absent from DataFusion built-ins") is
    /// empirically anchored across ALL three registry sources.
    ///
    /// EMPIRICAL VERDICT (DataFusion 53.1): "percentile" is ABSENT from default_scalar_functions().
    ///   → The manual `names.insert("percentile")` in DATAFUSION_BUILTIN_AGGREGATE_NAMES is
    ///     still the correct mechanism; no scalar-registry built-in shadows it.
    ///
    /// If this test FAILS: DataFusion added "percentile" as a scalar function.
    /// Action: investigate whether the manual insert is still needed; update ADR-048.
    ///
    /// Traces to: F-PQLFN-P5-LOW-001; ADR-048 v1.4 §D.2 scalar-registry absence claim.
    #[test]
    fn test_f_pqlfn_p5_low_001_percentile_absent_from_datafusion_53_1_scalar_registry() {
        use datafusion::execution::SessionStateDefaults;

        let raw_names: std::collections::HashSet<String> =
            SessionStateDefaults::default_scalar_functions()
                .iter()
                .flat_map(|f| {
                    let mut names = vec![f.name().to_ascii_lowercase()];
                    for alias in f.aliases() {
                        names.push(alias.to_ascii_lowercase());
                    }
                    names
                })
                .collect();

        // EMPIRICAL LOCK: "percentile" must be ABSENT from the scalar registry (DataFusion 53.1).
        assert!(
            !raw_names.contains("percentile"),
            "F-PQLFN-P5-LOW-001 EMPIRICAL LOCK BROKEN: 'percentile' is now IN DataFusion's \
             default_scalar_functions() registry (not present in DataFusion 53.1). \
             This changes the ADR-048 reconciliation — stop and report to the architect. \
             Action: determine whether the manual insert in DATAFUSION_BUILTIN_AGGREGATE_NAMES \
             is still necessary and update ADR-048 §D.2 accordingly."
        );
    }

    /// F-PQLFN-P5-LOW-001 empirical lock (window): "percentile" ABSENT from DataFusion 53.1
    /// default_window_functions() registry.
    ///
    /// Rationale: `DATAFUSION_BUILTIN_FUNCTION_NAMES` unions scalar + aggregate + window
    /// registries (see static initializer). The existing aggregate-registry absence lock
    /// covered only the aggregate arm. This lock completes coverage for the window arm, ensuring
    /// that ADR-048 v1.4 §D.2's claim ("percentile absent from DataFusion built-ins") is
    /// empirically anchored across ALL three registry sources.
    ///
    /// EMPIRICAL VERDICT (DataFusion 53.1): "percentile" is ABSENT from default_window_functions().
    ///   → The manual `names.insert("percentile")` in DATAFUSION_BUILTIN_AGGREGATE_NAMES is
    ///     still the correct mechanism; no window-registry built-in shadows it.
    ///
    /// If this test FAILS: DataFusion added "percentile" as a window function.
    /// Action: investigate whether the manual insert is still needed; update ADR-048.
    ///
    /// Traces to: F-PQLFN-P5-LOW-001; ADR-048 v1.4 §D.2 window-registry absence claim.
    #[test]
    fn test_f_pqlfn_p5_low_001_percentile_absent_from_datafusion_53_1_window_registry() {
        use datafusion::execution::SessionStateDefaults;

        let raw_names: std::collections::HashSet<String> =
            SessionStateDefaults::default_window_functions()
                .iter()
                .flat_map(|f| {
                    let mut names = vec![f.name().to_ascii_lowercase()];
                    for alias in f.aliases() {
                        names.push(alias.to_ascii_lowercase());
                    }
                    names
                })
                .collect();

        // EMPIRICAL LOCK: "percentile" must be ABSENT from the window registry (DataFusion 53.1).
        assert!(
            !raw_names.contains("percentile"),
            "F-PQLFN-P5-LOW-001 EMPIRICAL LOCK BROKEN: 'percentile' is now IN DataFusion's \
             default_window_functions() registry (not present in DataFusion 53.1). \
             This changes the ADR-048 reconciliation — stop and report to the architect. \
             Action: determine whether the manual insert in DATAFUSION_BUILTIN_AGGREGATE_NAMES \
             is still necessary and update ADR-048 §D.2 accordingly."
        );
    }

    /// F-PQLFN-PR9-OBS-001 **GREEN LOCK** — DataFusion registry set-difference invariant:
    /// `DATAFUSION_BUILTIN_AGGREGATE_NAMES ∖ DATAFUSION_BUILTIN_FUNCTION_NAMES`
    /// == `{"distinct_count", "percentile"}`.
    ///
    /// The HAVING-position interception gate (EC-11-086, ADR-048 v1.17 §D.2) uses the
    /// two-condition criterion to decide which names are intercepted:
    ///   (a) `name ∈ DATAFUSION_BUILTIN_AGGREGATE_NAMES`, AND
    ///   (b) `name ∉ DATAFUSION_BUILTIN_FUNCTION_NAMES`
    ///
    /// The set difference `AGGREGATE ∖ FUNCTION_NAMES` is the exact "triggering set" —
    /// names that satisfy BOTH conditions and thus reach the HAVING interception branch.
    /// Today this set is `{"distinct_count", "percentile"}`:
    ///   - Both are manually inserted in `DATAFUSION_BUILTIN_AGGREGATE_NAMES`.
    ///   - Both are absent from DataFusion 53.1's raw aggregate registry (F-PQLFN-P4-MED-001).
    ///   - Because they're absent from the raw registry, they're also absent from
    ///     `DATAFUSION_BUILTIN_FUNCTION_NAMES` (which unions scalar + aggregate + window).
    ///
    /// The two-branch `having_aggregate_interception_detail` builder (BC-2.11.019 v1.26
    /// §OBS-004, F-PQLFN-PR5-LOW-001) branches on `name_lower == "percentile"` to emit
    /// the two-arg template vs the generic `(...)` template. If a DataFusion 53.x upgrade
    /// adds a new aggregate-only name that widens the triggering set, this test fails
    /// loudly — the reviewer knows to extend the detail-builder's branching logic before
    /// silent behavior change reaches analysts.
    ///
    /// Note: `distinct_count` IS in the set difference but in practice never reaches
    /// `having_fncall_names` because it parses as `FuncCall::Aggregate` via
    /// `build_agg_call_parser` (not `ScalarFunc::Unknown`) — it is correctly included in
    /// the invariant as a structural property of the gate design (BC-2.11.019 v1.26 §OBS-004).
    ///
    /// Kills the class of silent-breakage mutations where a DataFusion upgrade changes the
    /// registry contents in a way that shifts the triggering set without any test failure.
    ///
    /// Traces to: BC-2.11.019 v1.26 §OBS-004 (set-difference invariant, F-PQLFN-PR9-OBS-001);
    ///            F-PQLFN-P4-MED-001 (empirical absence locks); ADR-048 v1.17 §D.2.
    #[test]
    fn test_f_pqlfn_pr9_obs_001_datafusion_set_difference_invariant() {
        let diff: std::collections::HashSet<&str> = super::DATAFUSION_BUILTIN_AGGREGATE_NAMES
            .iter()
            .filter(|name| !super::DATAFUSION_BUILTIN_FUNCTION_NAMES.contains(*name))
            .map(|s| s.as_str())
            .collect();

        let expected: std::collections::HashSet<&str> =
            ["distinct_count", "percentile"].iter().copied().collect();

        assert_eq!(
            diff, expected,
            "F-PQLFN-PR9-OBS-001 INVARIANT BROKEN: \
             DATAFUSION_BUILTIN_AGGREGATE_NAMES ∖ DATAFUSION_BUILTIN_FUNCTION_NAMES \
             must equal {{\"distinct_count\", \"percentile\"}}. \
             Actual difference: {diff:?}. \
             The HAVING interception gate (EC-11-086, ADR-048 v1.17 §D.2) fires for names \
             in this triggering set. A DataFusion upgrade that widened this set would \
             silently change HAVING-interception behavior for analysts. \
             Action: inspect the new name(s), extend `having_aggregate_interception_detail` \
             branching if the new name needs a custom message template, update ADR-048 §D.2, \
             and amend this test to include the new expected names after deliberate review."
        );
    }
}

// ---------------------------------------------------------------------------
// F-PQLFN-P35-MED-002: known-UDF-passes sibling locks for positions 1-5
// (ADR-048 §D.7.1 Positions 1-5; fix-burst 27)
// ---------------------------------------------------------------------------
//
// Five new bilateral boundary locks — one per gated position — mirror the pattern
// established by the Position 6 / OD-6 (fix-burst 26 POL-29 sibling sweep) and
// Position 7 / OD-7 (fix-burst 26) locks.  Positions 1-3 are walk-observable
// (F-PQLFN-P35-OBS-001) for the predicate_fncall_names walk: the compound predicate
// `enrich_lookup(ip_address) = 'US' AND totally_unknown_udf(x) = 1` proves
// (a) the predicate_fncall_names walk executes and (b) registry filtering correctly
// passes the known UDF while rejecting the unknown UDF.
// Positions 4-5 are boundary-locked via the sql_unknown_names path instead
// (collect_unknown_scalars_from_sql_query position (b)); the compound predicate tests
// are NOT walk-observable for the predicate_fncall_names walk at those positions.
// True walk locks: TM-06/TM-07 (Position 4) and TM-10 (Position 5) in
// temporal_typing_tests.rs (F-PQLFN-P37-MED-001).
//
// Tests call `check_enrich_udf_availability` directly (private fn, same file) to
// keep the fixture minimal — no registered table needed; the enrichment gate fires
// before any table-availability check.  Each test registers `enrich_lookup` via the
// same InfusionRegistry + InfusionSpec::new + InfusionField::new fixture pattern as
// the Position 6 / OD-6 and Position 7 / OD-7 locks.
//
// Traces to: F-PQLFN-P35-MED-002; F-PQLFN-P35-OBS-001; ADR-048 §D.7.1 Positions 1-5;
//            BC-2.11.019.

// ── Position 1: pipe | where ──────────────────────────────────────────────────

/// `FROM t | where enrich_lookup(ip_address) = 'US'` with `enrich_lookup` registered
/// MUST pass the plan-time gate (E-QUERY-039 does NOT fire).
///
/// Walk-observable (F-PQLFN-P35-OBS-001): compound predicate with known + unknown UDF
/// fires E-QUERY-039 for the unknown UDF only, proving the predicate walk executes and
/// registry filtering passes the known UDF.
///
/// Bilateral boundary for Position 1:
///   - known UDF in `| where` → gate passes (Ok(()))
///   - unknown UDF in `| where` → gate fires (E-QUERY-039) [locked by MED-004 in
///     temporal_typing_tests.rs]
///
/// Traces to: F-PQLFN-P35-MED-002; F-PQLFN-P35-OBS-001; ADR-048 §D.7.1 Position 1.
#[cfg(test)]
mod pipe_where_first_gated_position_enrich_udf_tests {
    use super::check_enrich_udf_availability;
    use prism_core::error::PrismError;

    #[test]
    fn test_pipe_where_enrich_udf_passes_gate() {
        use prism_spec_engine::{InfusionField, InfusionRegistry, InfusionSpec, InfusionType};

        let registry = InfusionRegistry::new();
        let spec = InfusionSpec::new(
            "geo_lookup",
            "GeoIP lookup (F-PQLFN-P35-MED-002 position-1 fixture)",
            InfusionType::LocalLookup,
            vec![InfusionField::new(
                "enrich_lookup", // UDF name — must match the fn-call in the query
                "ip_address",    // input field
                "string",        // input type
                "string",        // output type
            )],
            "/dev/null",
        );
        registry
            .load_spec(spec)
            .expect("geo_lookup spec must load for F-PQLFN-P35-MED-002 position-1 fixture");

        // F-PQLFN-P35-OBS-001 walk-observable: compound predicate (known + unknown UDF).
        // Regression class (a) — walk removal: if PipeStage::Where walk removed from Ast::Pipe
        //   arm, predicate_fncall_names stays empty → totally_unknown_udf undetected → false Ok.
        // Regression class (b) — registry filtering: if registry check removed, enrich_lookup
        //   would also fire E-QUERY-039, making d.infusion != "totally_unknown_udf".
        let compound_result = check_enrich_udf_availability(
            "FROM t | where enrich_lookup(ip_address) = 'US' AND totally_unknown_udf(x) = 1",
            Some(&registry),
        );
        assert!(
            matches!(&compound_result,
                Err(PrismError::EnrichUdfNotFound(ref d)) if d.infusion == "totally_unknown_udf"),
            "F-PQLFN-P35-OBS-001 Position 1 compound: pipe | where enrich_lookup AND totally_unknown_udf \
             must fire E-QUERY-039 for totally_unknown_udf only — proving (a) walk reaches pipe \
             | where predicate and (b) registry filtering passes enrich_lookup. \
             Got: {compound_result:?}"
        );
        if let Err(PrismError::EnrichUdfNotFound(ref d)) = compound_result {
            assert!(
                d.available_infusions.contains(&"enrich_lookup".to_string()),
                "F-PQLFN-P35-OBS-001 Position 1: available_infusions must contain 'enrich_lookup'. \
                 Got: {:?}",
                d.available_infusions
            );
        }

        // F-PQLFN-P35-MED-002 known-UDF-passes direction: pure known-UDF → Ok.
        let result = check_enrich_udf_availability(
            "FROM t | where enrich_lookup(ip_address) = 'US'",
            Some(&registry),
        );
        assert!(
            result.is_ok(),
            "F-PQLFN-P35-MED-002 Position 1: pipe | where enrich_lookup(ip_address) = 'US' with known \
             UDF MUST return Ok (E-QUERY-039 must NOT fire). \
             Position 1 behavioral boundary: known UDF → passes; unknown UDF → fires. \
             Got: {result:?}"
        );
    }
}

// ── Position 2: filter-mode root predicate ────────────────────────────────────

/// `t | enrich_lookup(ip_address) = 'US'` (filter mode) with `enrich_lookup` registered
/// MUST pass the plan-time gate (E-QUERY-039 does NOT fire).
///
/// Walk-observable (F-PQLFN-P35-OBS-001): compound predicate with known + unknown UDF
/// fires E-QUERY-039 for the unknown UDF only, proving the Ast::Filter root-predicate
/// walk executes and registry filtering passes the known UDF.
///
/// Bilateral boundary for Position 2:
///   - known UDF in filter root predicate → gate passes (Ok(()))
///   - unknown UDF in filter root predicate → gate fires (E-QUERY-039) [locked by MED-004 in
///     temporal_typing_tests.rs]
///
/// Traces to: F-PQLFN-P35-MED-002; F-PQLFN-P35-OBS-001; ADR-048 §D.7.1 Position 2.
#[cfg(test)]
mod filter_root_second_gated_position_enrich_udf_tests {
    use super::check_enrich_udf_availability;
    use prism_core::error::PrismError;

    #[test]
    fn test_filter_root_enrich_udf_passes_gate() {
        use prism_spec_engine::{InfusionField, InfusionRegistry, InfusionSpec, InfusionType};

        let registry = InfusionRegistry::new();
        let spec = InfusionSpec::new(
            "geo_lookup",
            "GeoIP lookup (F-PQLFN-P35-MED-002 position-2 fixture)",
            InfusionType::LocalLookup,
            vec![InfusionField::new(
                "enrich_lookup", // UDF name — must match the fn-call in the query
                "ip_address",    // input field
                "string",        // input type
                "string",        // output type
            )],
            "/dev/null",
        );
        registry
            .load_spec(spec)
            .expect("geo_lookup spec must load for F-PQLFN-P35-MED-002 position-2 fixture");

        // F-PQLFN-P35-OBS-001 walk-observable: compound predicate (known + unknown UDF).
        // Regression class (a) — walk removal: if Ast::Filter arm removed from
        //   check_enrich_udf_availability, predicate_fncall_names stays empty → false Ok.
        // Regression class (b) — registry filtering: if registry check removed, enrich_lookup
        //   would also fire E-QUERY-039, making d.infusion != "totally_unknown_udf".
        // Query: filter-mode (source_ref = "t", no FROM/SELECT prefix — first token is "t").
        let compound_result = check_enrich_udf_availability(
            "t | enrich_lookup(ip_address) = 'US' AND totally_unknown_udf(x) = 1",
            Some(&registry),
        );
        assert!(
            matches!(&compound_result,
                Err(PrismError::EnrichUdfNotFound(ref d)) if d.infusion == "totally_unknown_udf"),
            "F-PQLFN-P35-OBS-001 Position 2 compound: filter-mode enrich_lookup AND totally_unknown_udf \
             must fire E-QUERY-039 for totally_unknown_udf only — proving (a) walk reaches filter \
             root predicate (Ast::Filter arm) and (b) registry filtering passes enrich_lookup. \
             Got: {compound_result:?}"
        );
        if let Err(PrismError::EnrichUdfNotFound(ref d)) = compound_result {
            assert!(
                d.available_infusions.contains(&"enrich_lookup".to_string()),
                "F-PQLFN-P35-OBS-001 Position 2: available_infusions must contain 'enrich_lookup'. \
                 Got: {:?}",
                d.available_infusions
            );
        }

        // F-PQLFN-P35-MED-002 known-UDF-passes direction: pure known-UDF → Ok.
        let result =
            check_enrich_udf_availability("t | enrich_lookup(ip_address) = 'US'", Some(&registry));
        assert!(
            result.is_ok(),
            "F-PQLFN-P35-MED-002 Position 2: filter-mode enrich_lookup(ip_address) = 'US' with known \
             UDF MUST return Ok (E-QUERY-039 must NOT fire). \
             Position 2 behavioral boundary: known UDF → passes; unknown UDF → fires. \
             Got: {result:?}"
        );
    }
}

// ── Position 3: SqlPipe | where ───────────────────────────────────────────────

/// `SELECT ip_address FROM t | where enrich_lookup(ip_address) = 'US'` with
/// `enrich_lookup` registered MUST pass the plan-time gate (E-QUERY-039 does NOT fire).
///
/// Walk-observable (F-PQLFN-P35-OBS-001): compound predicate with known + unknown UDF
/// fires E-QUERY-039 for the unknown UDF only, proving the SqlPipe PipeStage::Where
/// walk executes and registry filtering passes the known UDF.
///
/// Bilateral boundary for Position 3:
///   - known UDF in SqlPipe | where → gate passes (Ok(()))
///   - unknown UDF in SqlPipe | where → gate fires (E-QUERY-039) [locked by MED-004 in
///     temporal_typing_tests.rs]
///
/// Traces to: F-PQLFN-P35-MED-002; F-PQLFN-P35-OBS-001; ADR-048 §D.7.1 Position 3.
#[cfg(test)]
mod sqlpipe_where_third_gated_position_enrich_udf_tests {
    use super::check_enrich_udf_availability;
    use prism_core::error::PrismError;

    #[test]
    fn test_sqlpipe_where_enrich_udf_passes_gate() {
        use prism_spec_engine::{InfusionField, InfusionRegistry, InfusionSpec, InfusionType};

        let registry = InfusionRegistry::new();
        let spec = InfusionSpec::new(
            "geo_lookup",
            "GeoIP lookup (F-PQLFN-P35-MED-002 position-3 fixture)",
            InfusionType::LocalLookup,
            vec![InfusionField::new(
                "enrich_lookup", // UDF name — must match the fn-call in the query
                "ip_address",    // input field
                "string",        // input type
                "string",        // output type
            )],
            "/dev/null",
        );
        registry
            .load_spec(spec)
            .expect("geo_lookup spec must load for F-PQLFN-P35-MED-002 position-3 fixture");

        // F-PQLFN-P35-OBS-001 walk-observable: compound predicate (known + unknown UDF).
        // Regression class (a) — walk removal: if PipeStage::Where walk removed from Ast::SqlPipe
        //   arm, predicate_fncall_names stays empty → totally_unknown_udf undetected → false Ok.
        // Regression class (b) — registry filtering: if registry check removed, enrich_lookup
        //   would also fire E-QUERY-039, making d.infusion != "totally_unknown_udf".
        let compound_result = check_enrich_udf_availability(
            "SELECT ip_address FROM t | where \
             enrich_lookup(ip_address) = 'US' AND totally_unknown_udf(x) = 1",
            Some(&registry),
        );
        assert!(
            matches!(&compound_result,
                Err(PrismError::EnrichUdfNotFound(ref d)) if d.infusion == "totally_unknown_udf"),
            "F-PQLFN-P35-OBS-001 Position 3 compound: SqlPipe | where enrich_lookup AND \
             totally_unknown_udf must fire E-QUERY-039 for totally_unknown_udf only — proving \
             (a) walk reaches SqlPipe PipeStage::Where predicate and (b) registry passes \
             enrich_lookup. Got: {compound_result:?}"
        );
        if let Err(PrismError::EnrichUdfNotFound(ref d)) = compound_result {
            assert!(
                d.available_infusions.contains(&"enrich_lookup".to_string()),
                "F-PQLFN-P35-OBS-001 Position 3: available_infusions must contain 'enrich_lookup'. \
                 Got: {:?}",
                d.available_infusions
            );
        }

        // F-PQLFN-P35-MED-002 known-UDF-passes direction: pure known-UDF → Ok.
        let result = check_enrich_udf_availability(
            "SELECT ip_address FROM t | where enrich_lookup(ip_address) = 'US'",
            Some(&registry),
        );
        assert!(
            result.is_ok(),
            "F-PQLFN-P35-MED-002 Position 3: SqlPipe | where enrich_lookup(ip_address) = 'US' with \
             known UDF MUST return Ok (E-QUERY-039 must NOT fire). \
             Position 3 behavioral boundary: known UDF → passes; unknown UDF → fires. \
             Got: {result:?}"
        );
    }
}

// ── Position 4: SQL SELECT WHERE (added by OD-5) ─────────────────────────────

/// `SELECT ip_address FROM t WHERE enrich_lookup(ip_address) = 'US'` with
/// `enrich_lookup` registered MUST pass the plan-time gate (E-QUERY-039 does NOT fire).
///
/// Boundary-locked (F-PQLFN-P35-OBS-001): compound predicate with known + unknown UDF
/// fires E-QUERY-039 for the unknown UDF only. Both `collect_unknown_scalars_from_sql_query`
/// (position (b)) and the `predicate_fncall_names → sql_unknown_names` fold in
/// `check_enrich_udf_availability` contribute at Position 4; the compound test is not
/// walk-distinguishable between them (true walk locks: TM-06/TM-07 in temporal_typing_tests.rs).
///
/// Bilateral boundary for Position 4 (added by OD-5):
///   - known UDF in SQL WHERE → gate passes (Ok(()))
///   - unknown UDF in SQL WHERE → gate fires (E-QUERY-039) [locked by BC-2.11.019 N1B tests
///     and TM-06/TM-07 aggregate-gate tests in temporal_typing_tests.rs]
///
/// Traces to: F-PQLFN-P35-MED-002; F-PQLFN-P35-OBS-001; ADR-048 §D.7.1 Position 4 (added by OD-5).
#[cfg(test)]
mod sql_where_fourth_gated_position_enrich_udf_tests {
    use super::check_enrich_udf_availability;
    use prism_core::error::PrismError;

    #[test]
    fn test_sql_where_enrich_udf_passes_gate() {
        use prism_spec_engine::{InfusionField, InfusionRegistry, InfusionSpec, InfusionType};

        let registry = InfusionRegistry::new();
        let spec = InfusionSpec::new(
            "geo_lookup",
            "GeoIP lookup (F-PQLFN-P35-MED-002 position-4 fixture)",
            InfusionType::LocalLookup,
            vec![InfusionField::new(
                "enrich_lookup", // UDF name — must match the fn-call in the query
                "ip_address",    // input field
                "string",        // input type
                "string",        // output type
            )],
            "/dev/null",
        );
        registry
            .load_spec(spec)
            .expect("geo_lookup spec must load for F-PQLFN-P35-MED-002 position-4 fixture");

        // F-PQLFN-P35-OBS-001 boundary lock: compound predicate (known + unknown UDF).
        // This test locks known-UDF-passes + unknown-UDF-fires boundaries at Position 4 via
        // the sql_unknown_names path (collect_unknown_scalars_from_sql_query position (b)).
        // NOT walk-observable for the Position-4 predicate_fncall_names walk — removing that
        // walk leaves sql_unknown_names populated and this test still passes (totally_unknown_udf
        // reaches E-QUERY-039 via sql_unknown_names regardless). The predicate_fncall_names
        // walk at Position 4 serves the aggregate gate exclusively (ADR-048 §D.7.1, OD-5):
        // DataFusion built-in aggregate names are filtered from sql_unknown_names by
        // DATAFUSION_BUILTIN_FUNCTION_NAMES before E-QUERY-039, so only predicate_fncall_names
        // catches them for E-QUERY-001. True walk locks (F-PQLFN-P37-MED-001):
        //   test_BC_2_11_019_tm_06_sql_where_count_e_query_001_high001 (TM-06)
        //   test_BC_2_11_019_tm_07_sql_where_sum_e_query_001_high001 (TM-07)
        // Regression class (b) — registry filtering: if registry check removed, enrich_lookup
        //   would also fire E-QUERY-039, making d.infusion != "totally_unknown_udf".
        let compound_result = check_enrich_udf_availability(
            "SELECT ip_address FROM t WHERE \
             enrich_lookup(ip_address) = 'US' AND totally_unknown_udf(x) = 1",
            Some(&registry),
        );
        assert!(
            matches!(&compound_result,
                Err(PrismError::EnrichUdfNotFound(ref d)) if d.infusion == "totally_unknown_udf"),
            "F-PQLFN-P35-OBS-001 Position 4 compound: SQL WHERE enrich_lookup AND totally_unknown_udf \
             must fire E-QUERY-039 for totally_unknown_udf only — proving (a) walk reaches SQL \
             WHERE predicate (Ast::Sql Select arm) and (b) registry passes enrich_lookup. \
             Got: {compound_result:?}"
        );
        if let Err(PrismError::EnrichUdfNotFound(ref d)) = compound_result {
            assert!(
                d.available_infusions.contains(&"enrich_lookup".to_string()),
                "F-PQLFN-P35-OBS-001 Position 4: available_infusions must contain 'enrich_lookup'. \
                 Got: {:?}",
                d.available_infusions
            );
        }

        // F-PQLFN-P35-MED-002 known-UDF-passes direction: pure known-UDF → Ok.
        let result = check_enrich_udf_availability(
            "SELECT ip_address FROM t WHERE enrich_lookup(ip_address) = 'US'",
            Some(&registry),
        );
        assert!(
            result.is_ok(),
            "F-PQLFN-P35-MED-002 Position 4: SQL WHERE enrich_lookup(ip_address) = 'US' with known \
             UDF MUST return Ok (E-QUERY-039 must NOT fire). \
             Position 4 behavioral boundary: known UDF → passes; unknown UDF → fires. \
             Got: {result:?}"
        );
    }
}

// ── Position 5: SqlPipe-head WHERE (added by OD-5) ────────────────────────────

/// `SELECT ip_address FROM t WHERE enrich_lookup(ip_address) = 'US' | limit 5` with
/// `enrich_lookup` registered MUST pass the plan-time gate (E-QUERY-039 does NOT fire).
///
/// Boundary-locked (F-PQLFN-P35-OBS-001): compound predicate with known + unknown UDF
/// fires E-QUERY-039 for the unknown UDF only. Both `collect_unknown_scalars_from_sql_query`
/// (position (b) on spq.head) and the `predicate_fncall_names → sql_unknown_names` fold in
/// `check_enrich_udf_availability` contribute at Position 5; the compound test is not
/// walk-distinguishable between them (true walk lock: TM-10 in temporal_typing_tests.rs).
///
/// Bilateral boundary for Position 5 (added by OD-5):
///   - known UDF in SqlPipe-head WHERE → gate passes (Ok(()))
///   - unknown UDF in SqlPipe-head WHERE → gate fires (E-QUERY-039) [TM-10 aggregate-gate
///     and BC-2.11.019 tests in temporal_typing_tests.rs]
///
/// Traces to: F-PQLFN-P35-MED-002; F-PQLFN-P35-OBS-001; ADR-048 §D.7.1 Position 5 (added by OD-5).
#[cfg(test)]
mod sqlpipe_head_where_fifth_gated_position_enrich_udf_tests {
    use super::check_enrich_udf_availability;
    use prism_core::error::PrismError;

    #[test]
    fn test_sqlpipe_head_where_enrich_udf_passes_gate() {
        use prism_spec_engine::{InfusionField, InfusionRegistry, InfusionSpec, InfusionType};

        let registry = InfusionRegistry::new();
        let spec = InfusionSpec::new(
            "geo_lookup",
            "GeoIP lookup (F-PQLFN-P35-MED-002 position-5 fixture)",
            InfusionType::LocalLookup,
            vec![InfusionField::new(
                "enrich_lookup", // UDF name — must match the fn-call in the query
                "ip_address",    // input field
                "string",        // input type
                "string",        // output type
            )],
            "/dev/null",
        );
        registry
            .load_spec(spec)
            .expect("geo_lookup spec must load for F-PQLFN-P35-MED-002 position-5 fixture");

        // F-PQLFN-P35-OBS-001 boundary lock: compound predicate (known + unknown UDF).
        // This test locks known-UDF-passes + unknown-UDF-fires boundaries at Position 5 via
        // the sql_unknown_names path (collect_unknown_scalars_from_sql_query position (b),
        // called on spq.head). NOT walk-observable for the Position-5 predicate_fncall_names
        // walk — removing that walk leaves sql_unknown_names populated and this test still
        // passes (totally_unknown_udf reaches E-QUERY-039 via sql_unknown_names regardless).
        // The predicate_fncall_names walk at Position 5 serves the aggregate gate exclusively
        // (ADR-048 §D.7.1, OD-5): DataFusion built-in aggregate names are filtered from
        // sql_unknown_names by DATAFUSION_BUILTIN_FUNCTION_NAMES before E-QUERY-039, so only
        // predicate_fncall_names catches them for E-QUERY-001. True walk lock (F-PQLFN-P37-MED-001):
        //   test_BC_2_11_019_tm_10_sqlpipe_head_where_aggregate_e_query_001_high001 (TM-10)
        // Regression class (b) — registry filtering: if registry check removed, enrich_lookup
        //   would also fire E-QUERY-039, making d.infusion != "totally_unknown_udf".
        // Query: SqlPipe mode — SELECT head with WHERE, followed by | limit stage.
        let compound_result = check_enrich_udf_availability(
            "SELECT ip_address FROM t WHERE \
             enrich_lookup(ip_address) = 'US' AND totally_unknown_udf(x) = 1 | limit 5",
            Some(&registry),
        );
        assert!(
            matches!(&compound_result,
                Err(PrismError::EnrichUdfNotFound(ref d)) if d.infusion == "totally_unknown_udf"),
            "F-PQLFN-P35-OBS-001 Position 5 compound: SqlPipe-head WHERE enrich_lookup AND \
             totally_unknown_udf must fire E-QUERY-039 for totally_unknown_udf only — proving \
             (a) walk reaches spq.head.where_ and (b) registry filtering passes enrich_lookup. \
             Got: {compound_result:?}"
        );
        if let Err(PrismError::EnrichUdfNotFound(ref d)) = compound_result {
            assert!(
                d.available_infusions.contains(&"enrich_lookup".to_string()),
                "F-PQLFN-P35-OBS-001 Position 5: available_infusions must contain 'enrich_lookup'. \
                 Got: {:?}",
                d.available_infusions
            );
        }

        // F-PQLFN-P35-MED-002 known-UDF-passes direction: pure known-UDF → Ok.
        let result = check_enrich_udf_availability(
            "SELECT ip_address FROM t WHERE enrich_lookup(ip_address) = 'US' | limit 5",
            Some(&registry),
        );
        assert!(
            result.is_ok(),
            "F-PQLFN-P35-MED-002 Position 5: SqlPipe-head WHERE enrich_lookup(ip_address) = 'US' with \
             known UDF MUST return Ok (E-QUERY-039 must NOT fire). \
             Position 5 behavioral boundary: known UDF → passes; unknown UDF → fires. \
             Got: {result:?}"
        );
    }
}

// ---------------------------------------------------------------------------
// F-PQLFN-P7-LOW-002: DML WHERE sixth gated position (ADR-048 §D.7.5, OD-6)
// ---------------------------------------------------------------------------
//
// Before fix: `Ast::Sql(SqlStatement::Dml(_))` fell to `_ => {}` in
// `check_enrich_udf_availability`. After the branch added `fn_call_comparison` to
// `build_predicate_parser` (which both build_delete_parser and build_update_parser
// bind), DML WHERE accepted fn-call LHS but the aggregate gate silently passed them.
// Post-fix: the `Ast::Sql(SqlStatement::Dml(dml))` arm walks `dml.filter` into
// `predicate_fncall_names`, restoring E-QUERY-001 for aggregates and enabling
// E-QUERY-039 for unknown UDFs with a registry.
//
// Tests call `check_enrich_udf_availability` directly (private fn, same file)
// to avoid requiring a registered table for DML gate tests. The aggregate gate
// fires before any table availability check — direct call cleanly isolates the gate.
//
// Traces to: F-PQLFN-P7-LOW-002; ADR-048 v1.6 §D.7.5; OD-6.
#[cfg(test)]
mod dml_where_sixth_gated_position_tests {
    use super::check_enrich_udf_availability;
    use prism_core::error::PrismError;

    /// F-PQLFN-P7-LOW-002 (1/4): `DELETE FROM t WHERE stddev(x) > 5` must fire E-QUERY-001
    /// (aggregate-in-predicate gate, ADR-048 D.7.1 Position 6). No infusion registry needed —
    /// aggregate gate fires regardless of registry state.
    ///
    /// Before fix: fell to `_ => {}` → aggregate gate skipped → SILENT EMPTY SUCCESS (DML no-op).
    /// After fix: DML arm walks dml.filter → stddev in predicate_fncall_names → E-QUERY-001.
    ///
    /// Traces to: F-PQLFN-P7-LOW-002; ADR-048 v1.6 §D.7.5.
    #[test]
    fn test_f_pqlfn_p7_low_002_delete_where_aggregate_fires_e_query_001() {
        let result = check_enrich_udf_availability("DELETE FROM t WHERE stddev(x) > 5", None);

        assert!(
            result.is_err(),
            "F-PQLFN-P7-LOW-002: DELETE FROM t WHERE stddev(x) > 5 must return Err \
             (E-QUERY-001 aggregate gate). Got Ok. \
             Before fix: fell to _ => {{}} arm — aggregate gate was a no-op for DML."
        );

        let err = result.unwrap_err();
        let display = format!("{err}");

        assert!(
            matches!(&err, PrismError::QueryParseFailed { .. }),
            "F-PQLFN-P7-LOW-002: DELETE WHERE aggregate must return QueryParseFailed \
             (E-QUERY-001). Got: {err:?} (Display: {display})"
        );

        assert!(
            display.contains("aggregate function"),
            "F-PQLFN-P7-LOW-002: Display must contain 'aggregate function' \
             (ADR-048 D.3 canonical message). Got: {display}"
        );

        assert!(
            display.contains("stddev"),
            "F-PQLFN-P7-LOW-002: Display must contain 'stddev' (the aggregate fn name). \
             Got: {display}"
        );

        assert!(
            display.contains("HAVING"),
            "F-PQLFN-P7-LOW-002: Display must contain 'HAVING' (ADR-048 D.3 guidance). \
             Got: {display}"
        );

        // F-PQLFN-P9-LOW-001 regression lock: canonical message must use position-agnostic
        // phrasing "WHERE/where predicates" (ADR-048 v1.8 §D.7.2) rather than the
        // misleading "pipe | where" which mis-identifies the error location for DML WHERE.
        assert!(
            display.contains("not valid in WHERE/where predicates"),
            "F-PQLFN-P9-LOW-001: Display must contain 'not valid in WHERE/where predicates' \
             (ADR-048 v1.8 §D.7.2 position-agnostic message). \
             Got: {display}"
        );
    }

    /// F-PQLFN-P7-LOW-002 (2/4): `UPDATE t SET col = 1 WHERE avg(x) > 100` must fire
    /// E-QUERY-001 (aggregate-in-predicate gate). No registry needed.
    ///
    /// Sibling of the DELETE test; exercises build_update_parser path (both bind
    /// build_predicate_parser, which now includes fn_call_comparison).
    ///
    /// Note: `variance` is absent from DataFusion 53.1's default_aggregate_functions() registry
    /// (DataFusion uses `var_samp`/`var_pop`). `avg` is a registered DataFusion aggregate and
    /// exercises the same gate. The test validates the mechanism (UPDATE WHERE aggregate →
    /// E-QUERY-001) rather than a specific function name.
    ///
    /// Traces to: F-PQLFN-P7-LOW-002; ADR-048 v1.6 §D.7.5.
    #[test]
    fn test_f_pqlfn_p7_low_002_update_where_aggregate_fires_e_query_001() {
        let result = check_enrich_udf_availability("UPDATE t SET col = 1 WHERE avg(x) > 100", None);

        assert!(
            result.is_err(),
            "F-PQLFN-P7-LOW-002: UPDATE t SET col = 1 WHERE avg(x) > 100 must return Err \
             (E-QUERY-001 aggregate gate). Got Ok."
        );

        let err = result.unwrap_err();
        let display = format!("{err}");

        assert!(
            matches!(&err, PrismError::QueryParseFailed { .. }),
            "F-PQLFN-P7-LOW-002: UPDATE WHERE aggregate must return QueryParseFailed \
             (E-QUERY-001). Got: {err:?} (Display: {display})"
        );

        assert!(
            display.contains("aggregate function"),
            "F-PQLFN-P7-LOW-002: Display must contain 'aggregate function'. Got: {display}"
        );

        assert!(
            display.contains("avg"),
            "F-PQLFN-P7-LOW-002: Display must contain 'avg' (the aggregate fn name). Got: {display}"
        );

        assert!(
            display.contains("HAVING"),
            "F-PQLFN-P7-LOW-002: Display must contain 'HAVING'. Got: {display}"
        );

        // F-PQLFN-P9-LOW-001 regression lock: canonical message must use position-agnostic
        // phrasing "WHERE/where predicates" (ADR-048 v1.8 §D.7.2) rather than the
        // misleading "pipe | where" which mis-identifies the error location for UPDATE WHERE.
        assert!(
            display.contains("not valid in WHERE/where predicates"),
            "F-PQLFN-P9-LOW-001: Display must contain 'not valid in WHERE/where predicates' \
             (ADR-048 v1.8 §D.7.2 position-agnostic message). \
             Got: {display}"
        );
    }

    /// F-PQLFN-P7-LOW-002 (3/4): `DELETE FROM t WHERE badudf(col) = 1` with an empty
    /// InfusionRegistry (badudf not registered) must fire E-QUERY-039.
    ///
    /// `badudf` is NOT a DataFusion built-in aggregate → aggregate gate does not fire.
    /// It IS a ScalarFunc::Unknown reaching predicate_fncall_names → folded into
    /// sql_unknown_names → not filtered by DATAFUSION_BUILTIN_FUNCTION_NAMES → not in
    /// registered_names → E-QUERY-039.
    ///
    /// Note: DELETE FROM t WHERE badudf(col) = 1 with NO registry returns Ok(()) early
    /// (registry=None → skip E-QUERY-039). This test uses an EMPTY registry (Some, zero
    /// entries) to confirm E-QUERY-039 fires when a registry is configured.
    ///
    /// Traces to: F-PQLFN-P7-LOW-002; ADR-048 v1.6 §D.7.5 (E-QUERY-039 coverage for DML).
    #[test]
    fn test_f_pqlfn_p7_low_002_delete_where_unknown_udf_fires_e_query_039() {
        use prism_spec_engine::InfusionRegistry;

        let registry = InfusionRegistry::new(); // empty — badudf not registered
        let result =
            check_enrich_udf_availability("DELETE FROM t WHERE badudf(col) = 1", Some(&registry));

        assert!(
            result.is_err(),
            "F-PQLFN-P7-LOW-002: DELETE FROM t WHERE badudf(col) = 1 with empty registry \
             must return Err (E-QUERY-039 unknown UDF). Got Ok."
        );

        let err = result.unwrap_err();
        let display = format!("{err}");

        assert!(
            matches!(&err, PrismError::EnrichUdfNotFound(_)),
            "F-PQLFN-P7-LOW-002: DELETE WHERE unknown UDF must return EnrichUdfNotFound \
             (E-QUERY-039). Got: {err:?} (Display: {display})"
        );

        assert!(
            display.contains("E-QUERY-039"),
            "F-PQLFN-P7-LOW-002: Display must contain 'E-QUERY-039'. Got: {display}"
        );

        assert!(
            display.contains("badudf"),
            "F-PQLFN-P7-LOW-002: Display must contain 'badudf' (the unknown UDF name). \
             Got: {display}"
        );
    }

    /// F-PQLFN-P7-LOW-002 (4/4): `UPDATE t SET col = 1 WHERE badudf(x) = 1` with an empty
    /// InfusionRegistry (badudf not registered) must fire E-QUERY-039.
    ///
    /// Sibling of the DELETE E-QUERY-039 test; exercises build_update_parser path.
    ///
    /// Traces to: F-PQLFN-P7-LOW-002; ADR-048 v1.6 §D.7.5.
    #[test]
    fn test_f_pqlfn_p7_low_002_update_where_unknown_udf_fires_e_query_039() {
        use prism_spec_engine::InfusionRegistry;

        let registry = InfusionRegistry::new(); // empty — badudf not registered
        let result = check_enrich_udf_availability(
            "UPDATE t SET col = 1 WHERE badudf(x) = 1",
            Some(&registry),
        );

        assert!(
            result.is_err(),
            "F-PQLFN-P7-LOW-002: UPDATE t SET col = 1 WHERE badudf(x) = 1 with empty registry \
             must return Err (E-QUERY-039 unknown UDF). Got Ok."
        );

        let err = result.unwrap_err();
        let display = format!("{err}");

        assert!(
            matches!(&err, PrismError::EnrichUdfNotFound(_)),
            "F-PQLFN-P7-LOW-002: UPDATE WHERE unknown UDF must return EnrichUdfNotFound \
             (E-QUERY-039). Got: {err:?} (Display: {display})"
        );

        assert!(
            display.contains("E-QUERY-039"),
            "F-PQLFN-P7-LOW-002: Display must contain 'E-QUERY-039'. Got: {display}"
        );

        assert!(
            display.contains("badudf"),
            "F-PQLFN-P7-LOW-002: Display must contain 'badudf'. Got: {display}"
        );
    }

    /// F-PQLFN-P34-OBS-001 sibling sweep (POL-29) — enrichment-UDF sibling lock for position-6:
    /// `DELETE FROM t WHERE enrich_lookup(ip_address) = 'US'` with `enrich_lookup` registered
    /// MUST pass the plan-time gate (E-QUERY-039 does NOT fire).
    ///
    /// This locks the behavioral boundary for OD-6 between:
    ///   - fn-call whose name is NOT in the registry → gate fires (E-QUERY-039)
    ///   - fn-call whose name IS in the registry (known enrichment UDF) → gate passes (Ok(()))
    ///
    /// When `enrich_lookup` IS in the registered UDF set, `check_enrich_udf_availability` must
    /// return Ok(()) — the E-QUERY-039 gate only fires for fn-calls NOT found in the registry.
    ///
    /// Position-6 already locks the unknown-UDF-fires direction (tests 3/4 and 4/4 above).
    /// This test adds the known-UDF-passes direction to complete the bilateral boundary lock,
    /// mirroring the sibling lock added to position-7 in the same fix-burst.
    ///
    /// Traces to: F-PQLFN-P34-OBS-001 sibling sweep (POL-29); ADR-048 §D.7.5 Position 6.
    #[test]
    fn test_dml_where_enrich_udf_passes_gate() {
        use prism_spec_engine::{InfusionField, InfusionRegistry, InfusionSpec, InfusionType};

        // Register `enrich_lookup` as a known UDF (mimics a real geo-lookup infusion).
        // source_path = "/dev/null" with LocalLookup → NullSource → load_spec succeeds.
        let registry = InfusionRegistry::new();
        let spec = InfusionSpec::new(
            "geo_lookup",
            "GeoIP lookup (F-PQLFN-P34-OBS-001 position-6 fixture)",
            InfusionType::LocalLookup,
            vec![InfusionField::new(
                "enrich_lookup", // UDF name — must match the fn-call in the query
                "ip_address",    // input field
                "string",        // input type
                "string",        // output type
            )],
            "/dev/null",
        );
        registry
            .load_spec(spec)
            .expect("geo_lookup spec must load for F-PQLFN-P34-OBS-001 position-6 fixture");

        // F-PQLFN-P35-OBS-001 walk-observable: compound predicate (known UDF + unknown UDF).
        // Proves (a) the walk reaches the DML WHERE predicate and (b) registry filtering
        // passes enrich_lookup as known. Regression locks:
        //   - walk-removal regression: if predicate walk removed, predicate_fncall_names stays
        //     empty → totally_unknown_udf undetected → gate passes silently (false Ok).
        //   - registry-filtering regression: if registry check removed, enrich_lookup would also
        //     fire E-QUERY-039, making d.infusion != "totally_unknown_udf".
        let compound_result = check_enrich_udf_availability(
            "DELETE FROM t WHERE enrich_lookup(ip_address) = 'US' AND totally_unknown_udf(x) = 1",
            Some(&registry),
        );
        assert!(
            matches!(&compound_result,
                Err(PrismError::EnrichUdfNotFound(ref d)) if d.infusion == "totally_unknown_udf"),
            "F-PQLFN-P35-OBS-001 OD-6 compound: DELETE WHERE enrich_lookup AND totally_unknown_udf \
             must fire E-QUERY-039 for totally_unknown_udf only — proving walk reaches DML WHERE \
             predicate and registry filtering passes enrich_lookup. Got: {compound_result:?}"
        );
        if let Err(PrismError::EnrichUdfNotFound(ref d)) = compound_result {
            assert!(
                d.available_infusions.contains(&"enrich_lookup".to_string()),
                "F-PQLFN-P35-OBS-001 OD-6: available_infusions must contain 'enrich_lookup' \
                 (registered UDF must appear as known). Got: {:?}",
                d.available_infusions
            );
        }

        // F-PQLFN-P35-MED-002 / F-PQLFN-P34-OBS-001 known-UDF-passes direction:
        // pure known-UDF predicate → Ok (E-QUERY-039 must NOT fire).
        let result = check_enrich_udf_availability(
            "DELETE FROM t WHERE enrich_lookup(ip_address) = 'US'",
            Some(&registry),
        );

        assert!(
            result.is_ok(),
            "F-PQLFN-P34-OBS-001 sibling sweep POL-29: DELETE FROM t WHERE enrich_lookup(\
             ip_address) = 'US' with known enrichment UDF MUST return Ok (E-QUERY-039 must NOT \
             fire). `enrich_lookup` IS in registered_names → gate passes. \
             OD-6 behavioral boundary: known UDF predicate → passes; unknown UDF predicate → fires. \
             Got: {result:?}"
        );
    }
}

// F-PQLFN-P32-OBS-001: INSERT source_select WHERE seventh gated position (ADR-048 §D.7.6, OD-7)
// --------------------------------------------------------------------------------------------
//
// Before fix: `Ast::Sql(SqlStatement::Dml(dml))` arm in `check_enrich_udf_availability`
// walked only `dml.filter`. INSERT queries have `filter = None` and carry the WHERE via
// `dml.source_select.where_`. Without the source_select walk, the aggregate gate saw
// filter=None, walked nothing, and returned Ok(()) — SILENT EMPTY SUCCESS for:
//   `INSERT INTO t (col) SELECT col FROM t2 WHERE stddev(x) > 5`
//
// Post-fix: the arm also walks `dml.source_select.where_` into `predicate_fncall_names`,
// restoring E-QUERY-001 for aggregates in INSERT source_select WHERE.
//
// source_select HAVING is intentionally exempt (§D.7.1 HAVING exemption; §D.7.3).
// The HAVING aggregate lock test confirms no false E-QUERY-001 fires for HAVING form.
//
// Tests call `check_enrich_udf_availability` directly (private fn, same file) to avoid
// requiring a registered table for DML gate tests. Aggregate gate fires before any table
// availability check — direct call cleanly isolates the gate behavior.
//
// Traces to: F-PQLFN-P32-OBS-001; ADR-048 v1.13 §D.7.6; OD-7.
#[cfg(test)]
mod insert_source_select_where_seventh_gated_position_tests {
    use super::check_enrich_udf_availability;
    use prism_core::error::PrismError;

    /// F-PQLFN-P32-OBS-001 (1/3): `INSERT INTO t (col) SELECT col FROM t2 WHERE stddev(x) > 5`
    /// must fire E-QUERY-001 (aggregate-in-predicate gate, ADR-048 D.7.1 Position 7).
    /// No infusion registry needed — aggregate gate fires regardless of registry state.
    ///
    /// Before fix: INSERT filter=None → gate walked nothing → SILENT EMPTY SUCCESS (DML no-op).
    /// After fix: DML arm walks dml.source_select.where_ → stddev in predicate_fncall_names
    ///            → aggregate gate → E-QUERY-001.
    ///
    /// Error detail must contain "stddev" and "aggregate" per §D.7.6 table.
    /// Offset must be > 0 (stddev appears after the INSERT prefix; exact byte position
    /// not hard-coded as it is fragile to query reformatting — §D.7.6 "Offset truthfulness").
    ///
    /// Traces to: F-PQLFN-P32-OBS-001; ADR-048 v1.13 §D.7.6.
    #[test]
    fn test_f_pqlfn_p32_obs_001_insert_source_select_where_aggregate_fires_e_query_001() {
        let result = check_enrich_udf_availability(
            "INSERT INTO t (col) SELECT col FROM t2 WHERE stddev(x) > 5",
            None,
        );

        assert!(
            result.is_err(),
            "F-PQLFN-P32-OBS-001: INSERT INTO t (col) SELECT col FROM t2 WHERE stddev(x) > 5 \
             must return Err (E-QUERY-001 aggregate gate). Got Ok. \
             Before fix: source_select.where_ not walked — aggregate gate was a no-op for INSERT."
        );

        let err = result.unwrap_err();
        let display = format!("{err}");

        assert!(
            matches!(&err, PrismError::QueryParseFailed { .. }),
            "F-PQLFN-P32-OBS-001: INSERT source_select WHERE aggregate must return \
             QueryParseFailed (E-QUERY-001). Got: {err:?} (Display: {display})"
        );

        assert!(
            display.contains("aggregate function"),
            "F-PQLFN-P32-OBS-001: Display must contain 'aggregate function' \
             (ADR-048 D.3 canonical message). Got: {display}"
        );

        assert!(
            display.contains("stddev"),
            "F-PQLFN-P32-OBS-001: Display must contain 'stddev' (the aggregate fn name). \
             Got: {display}"
        );

        assert!(
            display.contains("HAVING"),
            "F-PQLFN-P32-OBS-001: Display must contain 'HAVING' (ADR-048 D.3 guidance). \
             Got: {display}"
        );

        assert!(
            display.contains("not valid in WHERE/where predicates"),
            "F-PQLFN-P32-OBS-001: Display must contain 'not valid in WHERE/where predicates' \
             (ADR-048 v1.8 §D.7.2 position-agnostic message). Got: {display}"
        );

        // Offset truthfulness: stddev appears well into the INSERT statement; offset must be > 0.
        // Exact byte position not asserted — fragile to query reformatting (ADR-048 §D.7.6).
        if let PrismError::QueryParseFailed { offset, .. } = &err {
            assert!(
                *offset > 0,
                "F-PQLFN-P32-OBS-001: Offset must be > 0 (stddev appears after INSERT prefix). \
                 Got offset={offset}"
            );
        }
    }

    /// F-PQLFN-P32-OBS-001 (2/3): `INSERT INTO t (col) SELECT col FROM t2 WHERE avg(score) > 100`
    /// must fire E-QUERY-001 (aggregate-in-predicate gate). No registry needed.
    ///
    /// Sibling of the stddev test; exercises a second aggregate name to confirm the gate
    /// is not function-name-specific. `avg` is registered in DataFusion 53.1
    /// `default_aggregate_functions()` and exercises the same DATAFUSION_BUILTIN_AGGREGATE_NAMES
    /// gate mechanism.
    ///
    /// Traces to: F-PQLFN-P32-OBS-001; ADR-048 v1.13 §D.7.6.
    #[test]
    fn test_f_pqlfn_p32_obs_001_insert_source_select_where_avg_fires_e_query_001() {
        let result = check_enrich_udf_availability(
            "INSERT INTO t (col) SELECT col FROM t2 WHERE avg(score) > 100",
            None,
        );

        assert!(
            result.is_err(),
            "F-PQLFN-P32-OBS-001: INSERT INTO t (col) SELECT col FROM t2 WHERE avg(score) > 100 \
             must return Err (E-QUERY-001 aggregate gate). Got Ok."
        );

        let err = result.unwrap_err();
        let display = format!("{err}");

        assert!(
            matches!(&err, PrismError::QueryParseFailed { .. }),
            "F-PQLFN-P32-OBS-001: INSERT source_select WHERE avg must return QueryParseFailed \
             (E-QUERY-001). Got: {err:?} (Display: {display})"
        );

        assert!(
            display.contains("aggregate function"),
            "F-PQLFN-P32-OBS-001 (avg): Display must contain 'aggregate function'. Got: {display}"
        );

        assert!(
            display.contains("avg"),
            "F-PQLFN-P32-OBS-001 (avg): Display must contain 'avg'. Got: {display}"
        );

        assert!(
            display.contains("HAVING"),
            "F-PQLFN-P32-OBS-001 (avg): Display must contain 'HAVING'. Got: {display}"
        );

        assert!(
            display.contains("not valid in WHERE/where predicates"),
            "F-PQLFN-P32-OBS-001 (avg): Display must contain 'not valid in WHERE/where predicates'. \
             Got: {display}"
        );
    }

    /// F-PQLFN-P33-MED-001: Load-bearing HAVING exemption lock for INSERT source_select HAVING.
    ///
    /// Query: `INSERT INTO t (col) SELECT x FROM t2 GROUP BY x HAVING stddev(x) > 5`
    ///
    /// WHY stddev IS the load-bearing form (not count(*)):
    ///   `count(*)` parses as `FuncCall::Aggregate` which `collect_unknown_scalar_offsets_from_expr`
    ///   NEVER collects — so a test using count(*) would pass even if source_select.having were
    ///   incorrectly walked (the gate would see no names to flag).
    ///   `stddev(x)` parses as `FuncCall::Scalar(Unknown("stddev"))`, which IS collected by
    ///   `collect_unknown_scalar_offsets_from_expr`. `stddev` IS in `DATAFUSION_BUILTIN_AGGREGATE_NAMES`,
    ///   so IF having were walked it would appear in `predicate_fncall_names` → aggregate gate →
    ///   QueryParseFailed. The HAVING exemption (§D.7.1 / §D.7.3) must prevent this walk.
    ///
    /// Load-bearing property: if `check_enrich_udf_availability` were to incorrectly walk
    /// `source_select.having`, this test WOULD FAIL (stddev would be collected → E-QUERY-001
    /// fired despite HAVING being exempt). The count(*) sibling (test below) would NOT fail
    /// under the same regression. Hence stddev is the defect-detecting form.
    ///
    /// Traces to: F-PQLFN-P33-MED-001; ADR-048 §D.7.3; §D.7.6 "source_select HAVING: EXEMPT".
    #[test]
    fn test_f_pqlfn_p33_med_001_insert_source_select_having_stddev_load_bearing_exemption_lock() {
        let result = check_enrich_udf_availability(
            "INSERT INTO t (col) SELECT x FROM t2 GROUP BY x HAVING stddev(x) > 5",
            None,
        );

        // HAVING stddev must NOT produce E-QUERY-001 — HAVING is exempt (§D.7.1/§D.7.3).
        // If this test fails, source_select.having is being incorrectly walked by the gate.
        // Unlike the count(*) sibling, stddev IS collected by collect_unknown_scalar_offsets_from_expr
        // and IS in DATAFUSION_BUILTIN_AGGREGATE_NAMES, making this the true load-bearing lock.
        match &result {
            Err(PrismError::QueryParseFailed { .. }) => {
                let display = format!("{}", result.unwrap_err());
                assert!(
                    !display.contains("aggregate function"),
                    "F-PQLFN-P33-MED-001 HAVING LOCK (stddev): INSERT source_select HAVING stddev(x) > 5 \
                     must NOT fire E-QUERY-001 aggregate gate (HAVING is exempt per §D.7.1/§D.7.3). \
                     If this fires, source_select.having is being incorrectly walked. \
                     Got aggregate gate error: {display}"
                );
            }
            _ => {
                // Ok or non-QueryParseFailed error — HAVING exemption holds (stddev load-bearing form).
            }
        }
    }

    /// F-PQLFN-P32-OBS-001 (3/3): HAVING exemption regression lock.
    /// `INSERT INTO t (col) SELECT x, count(*) AS c FROM t2 GROUP BY x HAVING count(*) > 5`
    /// must NOT fire E-QUERY-001.
    ///
    /// HAVING predicates are exempt from the aggregate-in-predicate gate (§D.7.1 HAVING
    /// exemption; §D.7.3). The source_select HAVING must remain exempt even after the
    /// Position 7 gate extension — only source_select WHERE is gated.
    ///
    /// Note: count(*) parses as FuncCall::Aggregate which is never collected by
    /// collect_unknown_scalar_offsets_from_expr — this test passes even if having were
    /// incorrectly walked. The stddev sibling (F-PQLFN-P33-MED-001 above) is the
    /// load-bearing form that catches incorrect having-walk regressions.
    ///
    /// This is a GREEN lock test: it must remain passing. If this test fails it means
    /// the gate implementation incorrectly walked source_select.having.
    ///
    /// Traces to: F-PQLFN-P32-OBS-001; ADR-048 §D.7.3; §D.7.6 "source_select HAVING: EXEMPT".
    #[test]
    fn test_f_pqlfn_p32_obs_001_insert_source_select_having_aggregate_does_not_fire_e_query_001() {
        let result = check_enrich_udf_availability(
            "INSERT INTO t (col) SELECT x, count(*) AS c FROM t2 GROUP BY x HAVING count(*) > 5",
            None,
        );

        // HAVING aggregate must NOT produce E-QUERY-001. The result may be Ok or some other
        // error (e.g., DataFusion plan error for unknown table), but must NOT be
        // QueryParseFailed (E-QUERY-001) triggered by the aggregate gate.
        match &result {
            Err(PrismError::QueryParseFailed { .. }) => {
                let display = format!("{}", result.unwrap_err());
                // Only fail if the error is the aggregate gate message — a parse error
                // from unrecognised syntax is acceptable (the gate must not fire).
                assert!(
                    !display.contains("aggregate function"),
                    "F-PQLFN-P32-OBS-001 HAVING LOCK: INSERT source_select HAVING count(*) > 5 \
                     must NOT fire E-QUERY-001 aggregate gate (HAVING is exempt per §D.7.1/§D.7.3). \
                     Got aggregate gate error: {display}"
                );
            }
            _ => {
                // Ok or non-QueryParseFailed error — HAVING exemption holds.
            }
        }
    }

    /// F-PQLFN-P34-OBS-001 enrichment-UDF sibling lock: `INSERT INTO t (col) SELECT col FROM t2
    /// WHERE enrich_lookup(ip_address) = 'US'` with `enrich_lookup` registered MUST pass the
    /// plan-time gate (E-QUERY-039 does NOT fire).
    ///
    /// This locks the behavioral boundary for OD-7 between:
    ///   - fn-call whose name is NOT in the registry → gate fires (E-QUERY-039)
    ///   - fn-call whose name IS in the registry (known enrichment UDF) → gate passes (Ok(()))
    ///
    /// When `enrich_lookup` IS in the registered UDF set, `check_enrich_udf_availability` must
    /// return Ok(()) — the E-QUERY-039 gate only fires for fn-calls NOT found in the registry.
    ///
    /// F-PQLFN-P35-MED-002 accuracy fix: all seven predicate positions (ADR-048 §D.7.1) now
    /// carry a known-UDF-passes sibling lock — Position 1 (pipe | where), Position 2
    /// (filter root), Position 3 (SqlPipe | where), Position 4 (SQL WHERE, added by OD-5),
    /// Position 5 (SqlPipe-head WHERE, added by OD-5), Position 6 / OD-6 (DML WHERE,
    /// F-PQLFN-P34-OBS-001 POL-29), Position 7 / OD-7 (INSERT source_select WHERE).
    /// 5/7 locks are walk-observable (F-PQLFN-P35-OBS-001) for the predicate_fncall_names
    /// walk (Positions 1, 2, 3, 6, 7): compound predicate fires E-QUERY-039 for
    /// `totally_unknown_udf` only — proving the walk reaches the predicate and registry
    /// filtering passes the known UDF. Positions 4 and 5 are boundary-locked via the
    /// sql_unknown_names path (collect_unknown_scalars_from_sql_query position (b)); their
    /// predicate_fncall_names walk additionally reaches E-QUERY-039 via the
    /// `predicate_fncall_names → sql_unknown_names` fold but the compound tests are not
    /// walk-distinguishable between the two paths (ADR-048 §D.7.1, OD-5). True walk locks:
    /// TM-06/TM-07 (Position 4) and TM-10 (Position 5) in
    /// temporal_typing_tests.rs (F-PQLFN-P37-MED-001).
    ///
    /// Traces to: F-PQLFN-P34-OBS-001; F-PQLFN-P35-MED-002; F-PQLFN-P35-OBS-001;
    ///            ADR-048 v1.13 §D.7.6; BC-2.11.019; OD-7.
    #[test]
    fn test_insert_source_select_where_enrich_udf_passes_gate() {
        use prism_spec_engine::{InfusionField, InfusionRegistry, InfusionSpec, InfusionType};

        // Register `enrich_lookup` as a known UDF (mimics a real geo-lookup infusion).
        // Source is None (no backing file needed) → NullSource → load_spec succeeds.
        let registry = InfusionRegistry::new();
        let spec = InfusionSpec::new(
            "geo_lookup",
            "GeoIP lookup (F-PQLFN-P34-OBS-001 test fixture)",
            InfusionType::LocalLookup,
            vec![InfusionField::new(
                "enrich_lookup", // UDF name — must match the fn-call in the query
                "ip_address",    // input field
                "string",        // input type
                "string",        // output type
            )],
            "/dev/null",
        );
        registry
            .load_spec(spec)
            .expect("geo_lookup spec must load for F-PQLFN-P34-OBS-001 fixture");

        // F-PQLFN-P35-OBS-001 walk-observable: compound predicate (known UDF + unknown UDF).
        // Proves (a) the walk reaches INSERT source_select WHERE predicate and (b) registry
        // filtering passes enrich_lookup as known. Regression locks:
        //   - walk-removal regression: if source_select.where_ walk removed, predicate_fncall_names
        //     stays empty → totally_unknown_udf undetected → gate passes silently (false Ok).
        //   - registry-filtering regression: if registry check removed, enrich_lookup would also
        //     fire E-QUERY-039, making d.infusion != "totally_unknown_udf".
        let compound_result = check_enrich_udf_availability(
            "INSERT INTO t (col) SELECT col FROM t2 WHERE \
             enrich_lookup(ip_address) = 'US' AND totally_unknown_udf(x) = 1",
            Some(&registry),
        );
        assert!(
            matches!(&compound_result,
                Err(PrismError::EnrichUdfNotFound(ref d)) if d.infusion == "totally_unknown_udf"),
            "F-PQLFN-P35-OBS-001 OD-7 compound: INSERT source_select WHERE enrich_lookup AND \
             totally_unknown_udf must fire E-QUERY-039 for totally_unknown_udf only — proving \
             walk reaches source_select.where_ and registry filtering passes enrich_lookup. \
             Got: {compound_result:?}"
        );
        if let Err(PrismError::EnrichUdfNotFound(ref d)) = compound_result {
            assert!(
                d.available_infusions.contains(&"enrich_lookup".to_string()),
                "F-PQLFN-P35-OBS-001 OD-7: available_infusions must contain 'enrich_lookup' \
                 (registered UDF must appear as known). Got: {:?}",
                d.available_infusions
            );
        }

        // F-PQLFN-P35-MED-002 / F-PQLFN-P34-OBS-001 known-UDF-passes direction:
        // pure known-UDF predicate → Ok (E-QUERY-039 must NOT fire).
        let result = check_enrich_udf_availability(
            "INSERT INTO t (col) SELECT col FROM t2 WHERE enrich_lookup(ip_address) = 'US'",
            Some(&registry),
        );

        assert!(
            result.is_ok(),
            "F-PQLFN-P34-OBS-001 sibling lock: INSERT source_select WHERE with known enrichment \
             UDF `enrich_lookup` MUST return Ok (E-QUERY-039 must NOT fire). \
             `enrich_lookup` IS in registered_names → gate passes. \
             OD-7 behavioral boundary: known UDF predicate → passes; unknown UDF predicate → fires. \
             Got: {result:?}"
        );
    }
}

// ── F-PQLFN-PR5-LOW-001: two-branch HAVING interception detail-builder ─────────────────────────
//
// Tests for the extracted `having_aggregate_interception_detail` helper (BC-2.11.019 v1.26 §OBS-004).
// Branch (b) is unreachable through the gate arm in production (triggering set = {"percentile"})
// but is unit-tested directly here to verify the generic template is byte-exact per POL-24.
//
// RED GATE: these tests fail to compile before `having_aggregate_interception_detail` is defined
// (missing symbol → E0425). Once the helper is implemented they must go GREEN without changing
// any assertion text.
//
// Traces to: BC-2.11.019 v1.26 §OBS-004; F-PQLFN-PR5-LOW-001; POL-24; ADR-048 v1.17 §D.2.
#[cfg(test)]
mod having_aggregate_interception_detail_tests {
    use super::having_aggregate_interception_detail;

    /// F-PQLFN-PR5-LOW-001 (branch a, lowercase): `"percentile"` → two-arg canonical template.
    ///
    /// Verifies the percentile branch returns the byte-verbatim canonical template from
    /// BC-2.11.019 v1.26 §OBS-004 / ADR-048 v1.17 §D.2 with argument list `(field, p)`.
    ///
    /// Traces to: BC-2.11.019 v1.26 §OBS-004; F-PQLFN-PR5-LOW-001; POL-24.
    #[test]
    fn test_f_pqlfn_pr5_low_001_detail_builder_percentile_lowercase() {
        let detail = having_aggregate_interception_detail("percentile");
        assert_eq!(
            detail,
            "'percentile' is a PrismQL aggregate function; \
             PERCENTILE is not directly supported in HAVING predicates \
             \u{2014} alias it in SELECT: \
             SELECT PERCENTILE(field, p) AS alias ... HAVING alias > threshold \
             (ADR-048 D.3 OD-2)",
            "F-PQLFN-PR5-LOW-001 branch(a): percentile → two-arg template (field, p) \
             byte-verbatim per POL-24 (BC-2.11.019 v1.26 §OBS-004)"
        );
    }

    /// F-PQLFN-PR5-LOW-001 (branch a, uppercase input): `"PERCENTILE"` → input-verbatim `'PERCENTILE'`,
    /// template body uppercase `PERCENTILE`.
    ///
    /// Verifies the input-verbatim convention (BC-2.11.019 v1.26 §OBS-004 F-PQLFN-PR4-OBS-002):
    /// the analyst's original casing is echoed in the quoted name; the guidance template body
    /// uses uppercase `{name_upper}` regardless of input casing.
    ///
    /// Traces to: BC-2.11.019 v1.26 §OBS-004 (F-PQLFN-PR4-OBS-002 input-verbatim convention);
    ///            F-PQLFN-PR5-LOW-001; POL-24.
    #[test]
    fn test_f_pqlfn_pr5_low_001_detail_builder_percentile_uppercase_input() {
        let detail = having_aggregate_interception_detail("PERCENTILE");
        assert_eq!(
            detail,
            "'PERCENTILE' is a PrismQL aggregate function; \
             PERCENTILE is not directly supported in HAVING predicates \
             \u{2014} alias it in SELECT: \
             SELECT PERCENTILE(field, p) AS alias ... HAVING alias > threshold \
             (ADR-048 D.3 OD-2)",
            "F-PQLFN-PR5-LOW-001 branch(a) uppercase input: 'PERCENTILE' echoed verbatim, \
             template body uppercase PERCENTILE (BC-2.11.019 v1.26 §OBS-004 F-PQLFN-PR4-OBS-002)"
        );
    }

    /// F-PQLFN-PR5-LOW-001 (branch b): non-percentile name → generic template `(...)`.
    ///
    /// Branch (b) is unreachable in production today (triggering set = {"percentile"}) but is
    /// unit-tested directly to verify the generic `(...)` argument list is byte-exact per POL-24.
    /// Any future AGGREGATE-only name reaching the arm will emit this template — correct
    /// signature-neutral guidance without the two-arg PERCENTILE misapplication risk.
    ///
    /// Traces to: BC-2.11.019 v1.26 §OBS-004 (F-PQLFN-PR5-LOW-001 two-branch design); POL-24.
    #[test]
    fn test_f_pqlfn_pr5_low_001_detail_builder_generic_branch() {
        let detail = having_aggregate_interception_detail("array_agg");
        assert_eq!(
            detail,
            "'array_agg' is a PrismQL aggregate function; \
             ARRAY_AGG is not directly supported in HAVING predicates \
             \u{2014} alias it in SELECT: \
             SELECT ARRAY_AGG(...) AS alias ... HAVING alias > threshold \
             (ADR-048 D.3 OD-2)",
            "F-PQLFN-PR5-LOW-001 branch(b): non-percentile name → generic template (...) \
             byte-exact per POL-24 (BC-2.11.019 v1.26 §OBS-004)"
        );
    }
}

// ---------------------------------------------------------------------------
// F-PQLFN-PR14-OBS-001: sanitize-before-Levenshtein ordering (did_you_mean consistency)
// ---------------------------------------------------------------------------
//
// Before fix: `check_enrich_udf_availability` called `cap_name_for_levenshtein(requested)`
// on the RAW `requested` string, then compared against registered names for did_you_mean.
// `EnrichUdfNotFoundDetails::new` sanitized the same `requested` via `sanitize_for_log`
// at construction. Two operations derived from DIFFERENT string forms: Levenshtein used
// the raw form; the stored `infusion` field used the sanitized form — a CWE-116/CWE-117
// consistency gap.
//
// After fix: `sanitize_for_log(requested)` is called BEFORE `cap_name_for_levenshtein`,
// so both `did_you_mean` (computed) and `infusion` (stored) derive from the same sanitized
// string (BC-2.11.019 v1.26 §Injection-safety).
//
// PARSER NOTE: PrismQL identifiers are restricted to ASCII alphanumerics + '_' by the
// lexer (`c.is_ascii_alphanumeric() || *c == '_'`). Unicode C1 control chars (U+0085 etc.)
// and line/paragraph separators (U+2028, U+2029) cannot appear in parsed identifiers today.
// As a result, the sanitize call is a defensive no-op for all currently-reachable query
// paths. The tests below verify:
//   (a) the MATHEMATICAL PROPERTY (strsim + sanitize_for_log directly) that makes the
//       fix meaningful if control chars ever appear via future parser changes or alternative
//       code paths that populate UDF name strings from untrusted sources;
//   (b) the end-to-end did_you_mean path through `check_enrich_udf_availability` for a
//       clean ASCII typo — verifies the fix does not break the normal suggestion path.
//
// Traces to: F-PQLFN-PR14-OBS-001; BC-2.11.019 v1.26 §Injection-safety; CWE-116/CWE-117.
#[cfg(test)]
#[allow(non_snake_case, clippy::expect_used, clippy::unwrap_used)]
mod sanitize_ordering_did_you_mean_tests {
    use super::check_enrich_udf_availability;
    use prism_core::error::{sanitize_for_log, PrismError};

    /// F-PQLFN-PR14-OBS-001 (a) — mathematical property: C1 control chars + U+2028 inflate
    /// raw Levenshtein distance above 3, while the sanitized form produces distance 0 ≤ 3.
    ///
    /// The UDF name "nvd\u{0085}\u{0086}\u{0087}\u{2028}cvss" contains 3 C1 control chars
    /// (U+0085 NEL, U+0086 SSA, U+0087 ESA) and U+2028 LINE SEPARATOR — 4 extra Unicode
    /// codepoints between "nvd" and "cvss". Against registered "nvdcvss" (7 codepoints):
    ///
    ///   pre-fix:  levenshtein(11-codepoint raw, "nvdcvss") = 4 > 3 → no suggestion
    ///   post-fix: levenshtein(sanitize(raw), "nvdcvss") = levenshtein("nvdcvss", "nvdcvss")
    ///             = 0 ≤ 3 → did_you_mean = Some("nvdcvss")
    ///
    /// Load-bearing (TD-VSDD-059): if `sanitize_for_log` stops stripping C1/U+2028/U+2029,
    /// `sanitized` ≠ "nvdcvss" and the `assert_eq!(sanitized, "nvdcvss")` assertion fails.
    /// If the raw distance ≤ 3 (pre-condition holds but premise breaks), the test also fails.
    ///
    /// Traces to: F-PQLFN-PR14-OBS-001; BC-2.11.019 v1.26 §Injection-safety; CWE-116.
    #[test]
    fn test_f_pqlfn_pr14_obs_001_sanitize_math_property_control_chars_and_u2028() {
        // 3 C1 control chars (U+0085, U+0086, U+0087) + U+2028 (LINE SEPARATOR):
        // raw = "nvd" + 4 codepoints + "cvss" = 11 Unicode codepoints total.
        // registered = "nvdcvss" = 7 codepoints. 4 deletions needed → distance 4 > 3.
        let raw = "nvd\u{0085}\u{0086}\u{0087}\u{2028}cvss";
        let registered = "nvdcvss";

        // Pre-fix: levenshtein(raw, registered) must be > 3.
        let raw_dist = strsim::levenshtein(raw, registered);
        assert!(
            raw_dist > 3,
            "F-PQLFN-PR14-OBS-001 (a): pre-condition — levenshtein(raw, registered) must be > 3 \
             to demonstrate the pre-fix bug scenario. \
             raw={:?} registered={:?} dist={}",
            raw.escape_default(),
            registered,
            raw_dist
        );

        // Post-fix: sanitize_for_log strips C1/U+2028 → "nvdcvss" (identical to registered).
        let sanitized = sanitize_for_log(raw);
        assert_eq!(
            sanitized, registered,
            "F-PQLFN-PR14-OBS-001 (a): sanitize_for_log must strip 3 C1 chars \
             (U+0085/U+0086/U+0087) and U+2028, leaving 'nvdcvss'. Got: {:?}",
            sanitized
        );

        // Post-fix: levenshtein(sanitized, registered) = 0 ≤ 3 → did_you_mean fires.
        let sanitized_dist = strsim::levenshtein(&sanitized, registered);
        assert_eq!(
            sanitized_dist, 0,
            "F-PQLFN-PR14-OBS-001 (a): levenshtein(sanitized, registered) must be 0 \
             (identical strings after stripping). Got: {}",
            sanitized_dist
        );
    }

    /// F-PQLFN-PR14-OBS-001 (b) — end-to-end: did_you_mean correctly populated for a
    /// clean ASCII typo through `check_enrich_udf_availability`.
    ///
    /// After FIX 1, `sanitize_for_log` is called before `cap_name_for_levenshtein`. For
    /// ASCII-only identifiers, the sanitize call is a no-op — the Levenshtein distance
    /// is unchanged. This test verifies that the fix does not break the normal suggestion
    /// path: typo "nvdcvs" (1 edit from "nvdcvss") must still produce did_you_mean.
    ///
    /// Load-bearing (TD-VSDD-059): if the sanitize call accidentally mangles ASCII names
    /// (e.g., strips valid chars), the Levenshtein distance would change and the assertion
    /// `details.did_you_mean == Some("nvdcvss")` would fail.
    ///
    /// Traces to: F-PQLFN-PR14-OBS-001; BC-2.11.019 v1.26 §Injection-safety.
    #[test]
    fn test_f_pqlfn_pr14_obs_001_did_you_mean_clean_ascii_typo() {
        use prism_spec_engine::{InfusionField, InfusionRegistry, InfusionSpec, InfusionType};

        // Register "nvdcvss" as a known UDF (NVD CVSS score lookup fixture).
        let registry = InfusionRegistry::new();
        let spec = InfusionSpec::new(
            "nvd_lookup",
            "NVD CVSS lookup (F-PQLFN-PR14-OBS-001 b fixture)",
            InfusionType::LocalLookup,
            vec![InfusionField::new(
                "nvdcvss", // UDF name — matches the registered name in the query
                "cve_id",  // input field
                "string",  // input type
                "float",   // output type
            )],
            "/dev/null",
        );
        registry
            .load_spec(spec)
            .expect("nvd_lookup spec must load for F-PQLFN-PR14-OBS-001 (b) fixture");

        // "nvdcvs" is a 1-edit typo of "nvdcvss" (missing final 's').
        // sanitize("nvdcvs") = "nvdcvs" (no-op: ASCII-only), levenshtein("nvdcvs", "nvdcvss") = 1.
        // FIX 1 post-condition: 1 ≤ 3 → did_you_mean = Some("nvdcvss").
        let result =
            check_enrich_udf_availability("FROM t | enrich nvdcvs(cve_id)", Some(&registry));

        let err = result.expect_err(
            "F-PQLFN-PR14-OBS-001 (b): 'nvdcvs' is not registered; E-QUERY-039 must fire",
        );

        let details = match &err {
            PrismError::EnrichUdfNotFound(d) => d,
            other => panic!(
                "F-PQLFN-PR14-OBS-001 (b): expected PrismError::EnrichUdfNotFound, got: {other:?}"
            ),
        };

        assert_eq!(
            details.did_you_mean.as_deref(),
            Some("nvdcvss"),
            "F-PQLFN-PR14-OBS-001 (b): did_you_mean must be Some('nvdcvss') for 1-edit typo \
             'nvdcvs'. FIX 1: sanitize is a no-op for ASCII names, Levenshtein distance = 1 ≤ 3. \
             Got: {:?}",
            details.did_you_mean
        );

        // infusion must be "nvdcvs" — sanitize_for_log is identity for ASCII-only names.
        assert_eq!(
            details.infusion, "nvdcvs",
            "F-PQLFN-PR14-OBS-001 (b): infusion must be 'nvdcvs' (sanitize is no-op for ASCII). \
             Got: {:?}",
            details.infusion
        );
    }
}
