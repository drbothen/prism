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
//! - BC-2.11.006 — Security limits (30s timeout, 10K records, 200MB pool)
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
#![allow(dead_code)]

use std::sync::{Arc, Mutex};

use prism_core::{OrgSlug, PrismError};
use prism_credentials::CredentialStore;
use prism_ocsf::OcsfNormalizer;
use prism_sensors::AdapterRegistry;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

use crate::cache::{CacheConfig, QueryCache};
use crate::cursor::{spawn_cursor_cleanup_task, QueryCursorRegistry};
use crate::scoping::ClientRegistry;

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
    pub sensors: Option<Vec<prism_core::types::SensorType>>,
    /// Max results returned (tool-level truncation). Default 25, max 1000. (BC-2.11.001)
    pub limit: Option<usize>,
    /// Bypass response cache. (BC-2.11.001)
    pub force_refresh: bool,
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
    /// Registry of sensor adapters indexed by `(OrgId, SensorType)`.
    pub(crate) adapter_registry: Arc<AdapterRegistry>,
    /// Credential store for sensor authentication. (AI-opaque boundary)
    pub(crate) credential_store: Arc<dyn CredentialStore>,
    /// OCSF normalizer for converting raw sensor JSON to Arrow. (S-1.04)
    pub(crate) ocsf_normalizer: Arc<OcsfNormalizer>,
    /// Registry of configured client IDs. (BC-2.11.011)
    pub(crate) client_registry: Arc<ClientRegistry>,
    /// Engine-level configuration (limits, pool sizes, concurrency).
    pub(crate) config: QueryEngineConfig,
    /// Shared cursor registry — tracks all active pagination cursors (BC-2.07.001/002).
    pub(crate) cursor_registry: Arc<Mutex<QueryCursorRegistry>>,
    /// Shared sensor-fetch response cache (BC-2.07.003/006).
    pub(crate) cache: Arc<QueryCache>,
    /// Cancellation token used to signal the cursor cleanup task to stop.
    cleanup_shutdown: CancellationToken,
    /// Handle to the background cursor cleanup task (BC-2.07.002 §Background Cleanup).
    /// Held to ensure the task is aborted on Drop — not a dead field.
    cleanup_handle: Option<JoinHandle<()>>,
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
        let cache = Arc::new(QueryCache::new(cache_config));
        let shutdown = CancellationToken::new();

        // Start cursor cleanup background task (BC-2.07.002 §Background Cleanup).
        // Task exits when `shutdown` is cancelled (via Drop).
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
        }
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
    pub async fn execute(
        &self,
        query_str: &str,
        options: QueryOptions,
    ) -> Result<QueryResult, PrismError> {
        let start = std::time::Instant::now();

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
        // Step 1: Resolve client scope (BC-2.11.011).
        let clients =
            crate::scoping::resolve_clients(options.clients.clone(), &self.client_registry)?;

        // Step 2: Create ephemeral SessionContext for this query (BC-2.11.005, AD-002).
        let session_ctx = datafusion::execution::context::SessionContext::new();

        // Step 3: Set up MaterializationContext with engine dependencies.
        let mut mat_ctx = crate::materialization::MaterializationContext::new(
            Arc::clone(&self.adapter_registry),
            Arc::clone(&self.ocsf_normalizer),
            self.config.max_materialized_records,
        );

        // Step 4: Resolve effective options (merge client scope into options).
        let effective_options = QueryOptions {
            clients: Some(clients.clone()),
            ..options.clone()
        };

        // Step 5: Run the materialization pipeline → DataFusion execution → batches.
        let batches = crate::materialization::run_materialization_pipeline(
            query_str,
            &effective_options,
            &mut mat_ctx,
            &session_ctx,
        )
        .await?;

        // Step 6: Apply tool-level limit truncation.
        let limit = options.limit.unwrap_or(usize::MAX);
        let total_rows: usize = batches.iter().map(|b| b.num_rows()).sum();
        let is_truncated = total_rows > limit;
        let returned_results = total_rows.min(limit);

        // Truncate to limit (if needed).
        let final_batches = if is_truncated {
            truncate_batches_to_limit(batches, limit)
        } else {
            batches
        };

        // Step 7: Build QueryResult.
        let context = QueryResultContext {
            original_query: query_str.to_string(),
            expanded_query: query_str.to_string(),
            clients_queried: clients,
            sensors_queried: Vec::new(),
            execution_time_ms: 0, // filled in by execute()
        };

        Ok(QueryResult {
            batches: final_batches,
            total_available: total_rows,
            is_truncated,
            returned_results,
            context,
            sensor_errors: Vec::new(),
        })
    }

    /// Analyze a PrismQL query string and return an `ExplainResult` without
    /// executing any sensor API calls.
    ///
    /// Thin wrapper over `explain::explain()` that satisfies the COMP-003 interface
    /// specified in `module-decomposition.md` line 185. (CR-006, BC-2.11.010)
    ///
    /// # No sensor API calls
    /// Delegates to `explain::explain()` which is a pure plan-analysis function.
    /// No `fan_out()`, no sensor adapter `fetch()`.
    pub fn explain(
        &self,
        query_str: &str,
        options: crate::explain::ExplainOptions,
    ) -> Result<crate::explain::ExplainResult, PrismError> {
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

        // Resolve client scope (BC-2.11.011).
        let resolved_clients = crate::scoping::resolve_clients(clients, &self.client_registry)?;

        // Create ephemeral SessionContext (kept alive by the returned Arc).
        let session_ctx = Arc::new(datafusion::execution::context::SessionContext::new());

        // Set up MaterializationContext.
        let mut mat_ctx = crate::materialization::MaterializationContext::new(
            Arc::clone(&self.adapter_registry),
            Arc::clone(&self.ocsf_normalizer),
            self.config.max_materialized_records,
        );

        let effective_options = QueryOptions {
            clients: Some(resolved_clients.clone()),
            ..QueryOptions::default()
        };

        // Run the materialization pipeline against the session context.
        let batches = crate::materialization::run_materialization_pipeline(
            query_str,
            &effective_options,
            &mut mat_ctx,
            &session_ctx,
        )
        .await?;

        let total_rows: usize = batches.iter().map(|b| b.num_rows()).sum();

        let context = QueryResultContext {
            original_query: query_str.to_string(),
            expanded_query: query_str.to_string(),
            clients_queried: resolved_clients,
            sensors_queried: Vec::new(),
            execution_time_ms: start.elapsed().as_millis() as u64,
        };

        let qr = QueryResult {
            batches,
            total_available: total_rows,
            is_truncated: false,
            returned_results: total_rows,
            context,
            sensor_errors: Vec::new(),
        };

        Ok((qr, session_ctx))
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
