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
//! - Security perimeter (INV-SEC-PERIMETER-001, BC-2.11.006 v1.10):
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
    pub(crate) resolved_spec_map: Option<
        Arc<
            std::collections::HashMap<
                prism_spec_engine::ResolvedSpecKey,
                prism_spec_engine::ResolvedSensorSpec,
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
            resolved_spec_map: Some(resolved_spec_map),
            alias_store: Some(alias_store),
            infusion_registry: None,
            // S-1.14-REDO HIGH-1: Tier 2/3 caches default to None; wired via with_infusion_registry.
            infusion_lru_cache: None,
            infusion_tier3_cache: None,
            // S-3.13: table_registry is None in new_full; callers that need it
            // (production boot path with spec engine loaded) use with_table_registry().
            table_registry: None,
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
        let lru = Arc::new(prism_spec_engine::InfusionLruCache::new(10_000));
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
    /// `with_infusion_registry` must be called before this method (silently no-op if the
    /// infusion registry is not set).
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

    /// Return the `TableRegistry` arc, if wired.
    ///
    /// Exposed for tests that need to inspect or update the registry.
    pub fn table_registry(&self) -> Option<Arc<TableRegistry>> {
        self.table_registry.as_ref().map(Arc::clone)
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
        // F-PASS9-LOW-1: alias_store is wired into QueryEngine via new_full() so both
        // the CRUD tools and the query executor share the same live AliasStore.
        //
        // Scope: "global" — queries expand against the global alias scope at execute time.
        // Per-client scope override is architecturally deferred to S-3.01-ALIAS-SCOPE (BC-2.11.014)
        // which will thread the OrgSlug from QueryContext into the AliasResolver.
        // (SUG-8 fix: replaced stale "for now" comment with proper deferral citation.)
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
        check_table_availability(
            effective_query,
            self.table_registry.as_deref(),
            options.clients.as_deref(),
            self.resolved_spec_map.as_ref().map(|m| m.as_ref()),
        )?;

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
        if let Some(ref registry) = self.infusion_registry {
            // HIGH-1 fix (BC-2.19.002): use three-tier cache path when caches are wired.
            // Falls back to Tier-1-only (no-cache) path in test/legacy mode when caches are None.
            match (&self.infusion_lru_cache, &self.infusion_tier3_cache) {
                (Some(lru), Some(t3)) => crate::infusion_udf::register_infusion_udfs_with_cache(
                    &session_ctx,
                    registry.udf_descriptors(),
                    Arc::clone(lru),
                    Arc::clone(t3),
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
            self.resolved_spec_map.clone(),
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
        if options.resolved_spec_map.is_none() {
            if let Some(ref spec_map) = self.resolved_spec_map {
                options.resolved_spec_map = Some(Arc::clone(spec_map));
            }
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
        // HIGH-004 / ADV-W3MT-P58-HIGH-004: add Layer 1 capability gate to execute_scheduled.
        // Scheduled queries run in system context with no capabilities — this means they
        // cannot reference prism_audit (correct secure-by-default for scheduled queries).
        // The gate is best-effort: if query_str fails to parse, the pipeline handles it.
        check_internal_table_capabilities(query_str, &[])?;

        // S-3.13: plan-time table availability gate for scheduled queries (AC-8 mode-agnostic).
        // ADR-039 / SEC-001: pass org_scope and resolved_spec_map for org-scoped filtering.
        // Gate fires BEFORE resolve_clients to avoid moving `clients` before the borrow.
        check_table_availability(
            query_str,
            self.table_registry.as_deref(),
            clients.as_deref(),
            self.resolved_spec_map.as_ref().map(|m| m.as_ref()),
        )?;

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
            self.resolved_spec_map.clone(),
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
            InfusionUdfDescriptor {
                name: "threat_score".to_string(),
                input_type: "ip".to_string(),
                output_type: "string".to_string(),
                infusion_id: "threatintel_v1".to_string(),
                source: Arc::new(NullSrc),
                source_column: None,
            },
            InfusionUdfDescriptor {
                name: "threat_score".to_string(), // duplicate name
                input_type: "ip".to_string(),
                output_type: "string".to_string(),
                infusion_id: "threatintel_v2".to_string(),
                source: Arc::new(NullSrc),
                source_column: None,
            },
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
        // Inject resolved_spec_map (pub(crate)) — same pattern as make_engine_with_alias_store.
        engine.resolved_spec_map = Some(Arc::new(spec_map));
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
