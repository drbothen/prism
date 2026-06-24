//! `materialization` — ephemeral query materialization pipeline.
//!
//! Combines two layers:
//!
//! ## S-2.08 layer: `inject_source_type`
//! Pure-data `_source_type` virtual field injection (no DataFusion, no Arrow).
//! Sets `"_source_type"` on each row map based on `EventStream`/`PointInTime`
//! delivery model and whether rows came from the buffer. S-3.02 wires this
//! into the DataFusion pipeline.
//!
//! ## S-3.02 layer: `MaterializationPipeline`
//! Full 8-step ephemeral materialization pipeline (BC-2.11.005):
//!   Step 1: Parse PrismQL string via `PrismQlParser::parse` (public API only)
//!   Step 2: Resolve source refs to `(SensorId, client_id, SensorSpec)` tuples
//!   Step 3: Fan out to sensor adapters via `fan_out()` — all sources in parallel
//!   Step 4: Normalize each `Vec<serde_json::Value>` via `OcsfNormalizer`
//!   Step 5: Inject virtual field columns into each RecordBatch
//!   Step 6: Register each source as a DataFusion `MemTable`
//!   Step 7: Execute the SQL plan against the registered MemTables
//!   Step 8: Collect `SendableRecordBatchStream` → `Vec<RecordBatch>` → `QueryResult`
//!
//! # BC References
//! - BC-2.11.005 — Ephemeral Materialization
//! - BC-2.11.006 — Security Limits (10K record cap, 30s timeout, 200MB pool)
//! - BC-2.11.007 — Sensor Filter Push-Down
//! - BC-2.11.011 — Cross-Client Query Scoping
//! - BC-2.11.012 — Virtual Fields
//!
//! # Architecture Compliance (BC-2.11.006 / INV-SEC-PERIMETER-001)
//! Parser consumed ONLY via `PrismQlParser::parse`. Restricted symbols
//! (`parse_filter`, `parse_pipe`, `parse_sql`, builder factories, ParseLimits
//! thread-local API) MUST NOT appear in this module.
//!
//! Story: S-2.08 (inject_source_type) | S-3.02 (pipeline)

// S-3.02 stub functions: dead_code suppressed pending implementation (stub-phase convention).
// dead_code suppression removed — all items are now used (ADV-W3MT-P58-MED-002)

use std::sync::Arc;

use arrow::record_batch::RecordBatch;
use datafusion::execution::context::SessionContext;
use prism_core::{OrgId, OrgSlug, PrismError, SensorId};
use prism_ocsf::OcsfNormalizer;
use prism_sensors::{AdapterRegistry, CredentialResolver, SensorSpec};

use crate::{
    cache::SensorResponseCache,
    cache_key::{CacheKey, PushDownParams},
    engine::QueryOptions,
    pushdown::PushDownPlan,
    types::SensorQueryDescriptor,
};

// ---------------------------------------------------------------------------
// inject_source_type
// ---------------------------------------------------------------------------

/// Injects `"_source_type"` virtual field into every row in `rows`.
///
/// - When `descriptor.table_type == EventStream` **and** `descriptor.rows_from_buffer`:
///   sets `"_source_type": "buffered"` on every row (AC-9).
/// - Otherwise (PointInTime table, or EventStream cold-start live fallback):
///   sets `"_source_type": "live"` on every row (AC-10).
///
/// Operates on `serde_json::Value` row maps only — no DataFusion, no Arrow.
/// Non-object values in the slice are skipped without error.
///
/// Zero production callers as of S-3.02: `run_materialization_pipeline` shipped via
/// the MemTable path without calling this function. The cold-start EventStream buffer
/// routing (`SensorQueryDescriptor.rows_from_buffer` / `EventBufferStore` integration
/// per S-2.08 Architecture Compliance Rule 5) is not yet wired into the pipeline;
/// end-to-end wiring is tracked under TD-S302-005 alongside the deferred
/// integration-test assertions in `tests/integration_tests.rs`.
///
/// # AC-9
/// Given `EventStream` rows from the buffer: every row has `"_source_type": "buffered"`.
///
/// # AC-10
/// Given `PointInTime` rows or cold-start fallback live rows:
/// every row has `"_source_type": "live"`.
// S-2.08 spec mandates &mut Vec<serde_json::Value> signature for the pipeline
// wiring (TD-S302-005); clippy::ptr_arg is suppressed intentionally.
#[allow(clippy::ptr_arg)]
pub fn inject_source_type(rows: &mut Vec<serde_json::Value>, descriptor: &SensorQueryDescriptor) {
    use prism_core::TableType;

    let source_type =
        if descriptor.table_type == TableType::EventStream && descriptor.rows_from_buffer {
            "buffered"
        } else {
            "live"
        };

    for row in rows.iter_mut() {
        if let Some(obj) = row.as_object_mut() {
            obj.insert(
                "_source_type".to_string(),
                serde_json::Value::String(source_type.to_string()),
            );
        }
    }
}

// ============================================================================
// S-3.02 — Ephemeral Materialization Pipeline
// ============================================================================

// ---------------------------------------------------------------------------
// FanOutTarget
// ---------------------------------------------------------------------------

/// A fully-resolved fan-out target for a single (sensor, client) pair.
///
/// Produced by `resolve_source_refs` (Step 2 of the pipeline). Carries all
/// information needed to drive a sensor adapter call and subsequent
/// normalization. (BC-2.11.005)
///
/// Note: this type is distinct from `ast::SourceRef`, which is the parse-time
/// query source reference (`{ raw: String, kind: SourceRefKind }`). This type
/// represents the post-resolution fan-out target after client-scope expansion.
#[derive(Debug, Clone)]
pub struct FanOutTarget {
    /// The sensor id (e.g., `SensorId::from("crowdstrike")`).
    pub sensor_id: SensorId,
    /// The client ID owning this sensor instance. (BC-2.11.011)
    pub client_id: OrgSlug,
    /// The resolved OrgId for per-org adapter selection. (BC-3.2.001)
    pub org_id: OrgId,
    /// The sensor spec for this (sensor, client) pair.
    pub sensor_spec: SensorSpec,
    /// The source table name (e.g., `"crowdstrike_detections"`).
    pub source_table: String,
    /// Push-down plan computed for this source. (BC-2.11.007)
    pub push_down_plan: PushDownPlan,
}

// ---------------------------------------------------------------------------
// MaterializationOutput
// ---------------------------------------------------------------------------

/// Output of the `run_materialization_pipeline`.
///
/// Carries both result batches and per-sensor error messages so that partial
/// failures are surfaced to callers rather than silently discarded.
/// (F-LP1-CRIT-5, BC-2.11.005, BC-2.11.011, SOUL.md #4)
#[derive(Debug)]
pub struct MaterializationOutput {
    /// OCSF-normalized result RecordBatches.
    pub batches: Vec<RecordBatch>,
    /// Per-sensor error messages for partial failures. (BC-2.11.011 postcondition)
    pub sensor_errors: Vec<String>,
    /// Table names registered in the session context.
    pub registered_tables: Vec<String>,
    /// Sensor type names queried during fan-out. Populated from fan-out targets.
    /// Used to populate `QueryResultContext.sensors_queried` (BC-2.11.001).
    /// (ADV-W3MT-P58-HIGH-005)
    pub sensors_queried: Vec<String>,
}

// ---------------------------------------------------------------------------
// MaterializationContext
// ---------------------------------------------------------------------------

/// Context threaded through the materialization pipeline.
///
/// Holds shared dependencies and running state (e.g., record counter for
/// the 10K cap). Created at the start of each `execute()` call and dropped
/// with the `SessionScope` when the call returns.
///
/// # BC-2.11.005
/// The per-query in-query cache is keyed on
/// `(client_id, sensor_id, source_id, push_down_params)` to prevent
/// redundant API calls within one query.
pub struct MaterializationContext {
    /// Shared adapter registry for sensor fan-out.
    pub(crate) adapter_registry: Arc<AdapterRegistry>,
    /// OCSF normalizer for raw JSON → Arrow RecordBatch conversion.
    /// Stored for future DataFusion integration; not read directly in current pipeline.
    /// (ADV-W3MT-P58-MED-002: targeted allow)
    #[allow(dead_code)]
    pub(crate) ocsf_normalizer: Arc<OcsfNormalizer>,
    /// Running record count across all sources (10K cap enforcer). (BC-2.11.006)
    /// Private to prevent callers from bypassing the cap by zeroing this field.
    pub(crate) record_count: usize,
    /// Maximum records before aborting materialization. (BC-2.11.006)
    /// Private to prevent callers from bypassing the cap by setting usize::MAX.
    pub(crate) max_records: usize,
    /// Per-query in-query cache: avoids redundant API calls for self-joins.
    /// Key: canonical cache key string. Value: collected batches. (BC-2.11.005)
    /// Private to prevent cache poisoning; access via typed accessors.
    pub(crate) in_query_cache: std::collections::HashMap<String, Vec<RecordBatch>>,
    /// Credential resolver for fan_out() dispatch. (F-LP1-CRIT-2)
    pub(crate) credential_resolver: Arc<dyn CredentialResolver>,
    /// OrgSlug → OrgId registry for per-org adapter selection. (F-LP1-CRIT-3)
    /// When `None`, falls back to `get_all_for_sensor` (test/MVP mode).
    pub(crate) org_registry: Option<Arc<prism_core::OrgRegistry>>,
    /// Per-org overlay resolved spec map for per-org endpoint dispatch (ADR-029).
    /// When `Some`, `fan_out_with_overlay_map` is used; when `None`, bare `fan_out` is used.
    /// (F-LP2-CRIT-001 wiring — S-CONFIG-MULTI-TENANT-OVERRIDE-001)
    pub(crate) resolved_spec_map: Option<
        Arc<
            std::collections::HashMap<
                prism_spec_engine::ResolvedSpecKey,
                prism_spec_engine::ResolvedSensorSpec,
            >,
        >,
    >,
    /// Cross-query sensor-fetch response cache (BC-2.07.003/005/006).
    ///
    /// When `Some`, the pipeline checks this cache BEFORE issuing sensor API
    /// calls (keyed by the BC-2.07.005 4-tuple) and stores the complete fetch
    /// response after a successful fan-out. `force_refresh: true` bypasses the
    /// read and replaces the entry. When `None` (test/query-only mode), every
    /// fetch goes to the sensor API. Wired from `QueryEngine` via
    /// `with_response_cache` (QRY-02 closure — the engine-owned cache was
    /// previously constructed but never consulted).
    pub(crate) response_cache: Option<Arc<SensorResponseCache>>,
}

impl MaterializationContext {
    /// Construct a new `MaterializationContext` for a single query execution.
    ///
    /// Uses `NullCredentialResolver`; use `new_with_resolver` for production.
    pub fn new(
        adapter_registry: Arc<AdapterRegistry>,
        ocsf_normalizer: Arc<OcsfNormalizer>,
        max_records: usize,
    ) -> Self {
        Self::new_with_resolver(
            adapter_registry,
            ocsf_normalizer,
            max_records,
            Arc::new(crate::materialization::NullMaterializationCredentialResolver),
            None,
            None,
        )
    }

    /// Construct a new `MaterializationContext` with explicit resolver and registry.
    ///
    /// Used by `QueryEngine::execute_inner` to inject the engine's
    /// `CredentialResolver`, `OrgRegistry`, and `resolved_spec_map` into the pipeline.
    /// (F-LP1-CRIT-2, F-LP1-CRIT-3, F-LP2-CRIT-001)
    pub fn new_with_resolver(
        adapter_registry: Arc<AdapterRegistry>,
        ocsf_normalizer: Arc<OcsfNormalizer>,
        max_records: usize,
        credential_resolver: Arc<dyn CredentialResolver>,
        org_registry: Option<Arc<prism_core::OrgRegistry>>,
        resolved_spec_map: Option<
            Arc<
                std::collections::HashMap<
                    prism_spec_engine::ResolvedSpecKey,
                    prism_spec_engine::ResolvedSensorSpec,
                >,
            >,
        >,
    ) -> Self {
        Self {
            adapter_registry,
            ocsf_normalizer,
            record_count: 0,
            max_records,
            in_query_cache: std::collections::HashMap::new(),
            credential_resolver,
            org_registry,
            resolved_spec_map,
            response_cache: None,
        }
    }

    /// Attach the cross-query sensor-fetch response cache (BC-2.07.003).
    ///
    /// Called by `QueryEngine` so the pipeline shares the engine-owned
    /// `SensorResponseCache` (the same instance the write path invalidates
    /// through `CacheInvalidator`, BC-2.07.004).
    pub fn with_response_cache(mut self, cache: Arc<SensorResponseCache>) -> Self {
        self.response_cache = Some(cache);
        self
    }

    /// Increment the running record count, enforcing the 10K cap. (BC-2.11.006 EC-003)
    ///
    /// Returns `Err(PrismError::QueryMaterializationLimitExceeded)` (E-QUERY-005,
    /// error-taxonomy.md materialization limit) if the new count would exceed
    /// `max_records`. Uses saturating addition to prevent integer overflow.
    pub(crate) fn increment_record_count(&mut self, n: usize) -> Result<(), PrismError> {
        let new = self.record_count.saturating_add(n);
        if new > self.max_records {
            return Err(PrismError::QueryMaterializationLimitExceeded {
                count: new,
                max: self.max_records,
            });
        }
        self.record_count = new;
        Ok(())
    }

    /// Look up a cached batch set by cache key. (BC-2.11.005, F-LP1-MED-2)
    pub(crate) fn cache_lookup(&self, key: &str) -> Option<&Vec<RecordBatch>> {
        self.in_query_cache.get(key)
    }

    /// Insert a batch set into the in-query cache. (BC-2.11.005, F-LP1-MED-2)
    pub(crate) fn cache_insert(&mut self, key: String, batches: Vec<RecordBatch>) {
        self.in_query_cache.insert(key, batches);
    }
}

// ---------------------------------------------------------------------------
// NullMaterializationCredentialResolver — used by legacy `new()` constructor
// ---------------------------------------------------------------------------

/// No-op `CredentialResolver` for `MaterializationContext::new`.
///
/// Returns `SensorError::Internal` for any resolution attempt.
/// Tests registering `StubAdapter` instances don't trigger credential
/// resolution because `StubAdapter::fetch` ignores the `_auth` parameter.
pub(crate) struct NullMaterializationCredentialResolver;

impl CredentialResolver for NullMaterializationCredentialResolver {
    fn resolve(
        &self,
        _client_id: &str,
        sensor_id: SensorId,
    ) -> Result<Box<dyn prism_sensors::SensorAuth>, prism_sensors::SensorError> {
        Err(prism_sensors::SensorError::Internal {
            detail: format!(
                "NullMaterializationCredentialResolver: no credential for sensor {sensor_id:?}"
            ),
        })
    }
}

// ---------------------------------------------------------------------------
// derive_response_cache_key
// ---------------------------------------------------------------------------

/// Derive the BC-2.07.005 response-cache key for a fan-out target.
///
/// The `push_down_hash` input is the canonicalized set of sensor-native
/// push-down parameters for the fetch: the WHERE-clause equality `FilterMap`
/// (namespaced `filter.<column>`), the ADR-033 extracted time-window bounds
/// (`start_time` / `end_time`), and the **effective fetch-limit** (parameter
/// key `fetch.limit` — BC-2.07.005 v4.4). The original PrismQL query string,
/// the `force_refresh` flag, and PrismQL post-filters are excluded per
/// BC-2.07.005 §Hash Input. Two different PrismQL queries that produce
/// identical push-down parameters therefore share a cache entry
/// (BC-2.07.003 §Postconditions).
///
/// # Effective fetch-limit (P1-01 / BC-2.07.005 v4.4)
///
/// `fetch_limit` is the exact `u64` pushed into the fan-out target's
/// `QueryParams.limit` (BC-2.01.013 v1.14 / F-P1-CRIT-004). Because fetched
/// responses are limit-truncated at the sensor API, an entry fetched under
/// limit L is valid only for queries fetching under the same L. The
/// tool-level `limit`'s *post-materialization truncation role* remains
/// excluded — what is hashed is the fetch-limit actually pushed. `0` is the
/// no-limit sentinel (EC-08 of BC-2.01.013 v1.14): when 0, the parameter is
/// **omitted** from the canonical form per the null/absent-omission rule, so
/// unlimited fetches share entries with unlimited fetches (EC-07-044).
///
/// Coherence invariant (architect adjudication D1,
/// `proposals/cache-envelope-adjudication-2026-06-10.md`): the caller must
/// feed the SAME local binding into this function and into the fan-out
/// target's `QueryParams.limit` — the limit hashed is the limit fetched.
pub(crate) fn derive_response_cache_key(
    client_id: &OrgSlug,
    sensor_id: &SensorId,
    source_table: &str,
    filters: &prism_sensors::types::FilterMap,
    start_time: &Option<String>,
    end_time: &Option<String>,
    fetch_limit: u64,
) -> CacheKey {
    let mut params = PushDownParams::new();
    // Namespace filter keys so a sensor column literally named "start_time"
    // cannot collide with the time-window parameters below. (The `fetch.limit`
    // key below cannot collide either: filter keys are `filter.<column>`.)
    for (k, v) in filters {
        params.insert(format!("filter.{k}"), v.clone());
    }
    if let Some(st) = start_time {
        params.insert("start_time", serde_json::Value::String(st.clone()));
    }
    if let Some(et) = end_time {
        params.insert("end_time", serde_json::Value::String(et.clone()));
    }
    // BC-2.07.005 v4.4: hash the effective fetch-limit; omit the 0 / no-limit
    // sentinel per the null/absent-omission rule (EC-07-044).
    if fetch_limit > 0 {
        params.insert("fetch.limit", serde_json::Value::from(fetch_limit));
    }
    CacheKey::derive(client_id.as_str(), sensor_id.clone(), source_table, &params)
}

// ---------------------------------------------------------------------------
// store_or_invalidate_response_cache
// ---------------------------------------------------------------------------

/// Store/invalidate decision for the cross-query response cache after a
/// fan-out fetch (BC-2.07.003 §Postconditions; P1-05 / architect adjudication
/// D3, `proposals/cache-envelope-adjudication-2026-06-10.md`).
///
/// - `complete_response = Some(rows)` (fetch succeeded with NO per-target
///   errors): store the complete response — `force_refresh` replaces any
///   existing entry, the normal path inserts. TTL selection by source data
///   type happens inside `put` (60s alerts / 300s devices / health not cached).
/// - `complete_response = None` (the fetch cannot produce a complete
///   replacement: all targets failed OR per-target errors made the result
///   partial — partial responses are never cached):
///   - `force_refresh: true` → the existing entry is **invalidated (removed)**
///     (EC-07-033 / EC-07-034). `force_refresh` is an explicit analyst
///     distrust signal; retaining the distrusted entry would silently serve it
///     to later non-forced queries. Subsequent non-forced queries for the
///     tuple miss the cache and re-attempt the fetch.
///   - `force_refresh: false` → the existing unexpired entry is **retained**
///     (availability asymmetry — a normal fetch failure never invalidates).
///
/// Invalidation is per-entry (per cache key); sibling entries at other
/// fetch-limits age out by TTL. Eviction accounting stays atomic with
/// partition mutation via `remove_entry` (TD-PRISM-QUERY-CACHE-001).
pub(crate) fn store_or_invalidate_response_cache(
    cache: &crate::cache::SensorResponseCache,
    key: &CacheKey,
    force_refresh: bool,
    complete_response: Option<Vec<RecordBatch>>,
) -> Result<(), PrismError> {
    match complete_response {
        Some(rows) => {
            if force_refresh {
                cache.force_refresh(key.clone(), rows)
            } else {
                cache.put(key.clone(), rows)
            }
        }
        None if force_refresh => cache.remove_entry(key),
        None => Ok(()),
    }
}

// ---------------------------------------------------------------------------
// run_materialization_pipeline
// ---------------------------------------------------------------------------

/// Execute the full 8-step ephemeral materialization pipeline.
///
/// # Steps (BC-2.11.005)
/// 1. Parse PrismQL string via `PrismQlParser::parse` — public API only
/// 2. Resolve source refs to `FanOutTarget` tuples
/// 3. Fan out to sensor adapters via `fan_out()` — all sources in parallel
/// 4. Normalize each response via `OcsfNormalizer` → `Vec<RecordBatch>`
/// 5. Inject virtual field columns (`_sensor`, `_client`, `_source_table`)
/// 6. Register each source as a DataFusion `MemTable` in `ctx`
/// 7. Execute the SQL plan against the registered MemTables
/// 8. Collect `SendableRecordBatchStream` → `Vec<RecordBatch>`
///
/// # Record Cap (BC-2.11.006, EC-003)
/// Streaming counter across all sources. If the record counter exceeds
/// the maximum during Step 3, abort with
/// `PrismError::QueryMaterializationLimitExceeded` (E-QUERY-005).
///
/// # Returns
/// `MaterializationOutput` containing batches, sensor_errors, and registered_tables.
/// Sensor errors are accumulated (partial failure, BC-2.11.011) and returned to the
/// caller rather than silently discarded (SOUL.md #4 / F-LP1-CRIT-5).
///
/// # Architecture Compliance (INV-SEC-PERIMETER-001)
/// Parser consumed ONLY via `PrismQlParser::parse`. Restricted sub-parser
/// symbols MUST NOT appear in this function body.
pub async fn run_materialization_pipeline(
    query_str: &str,
    options: &QueryOptions,
    mat_ctx: &mut MaterializationContext,
    session_ctx: &SessionContext,
) -> Result<MaterializationOutput, PrismError> {
    // Step 1: Parse the query to extract source table names.
    // Parse-time security guards (size, nesting depth, stage count) are enforced
    // inside `PrismQlParser::parse` via the security module (BC-2.11.006 / F-LP1-MED-4).
    let ast = crate::filter_parser::PrismQlParser::parse(query_str).map_err(|errs| {
        PrismError::QueryParseFailed {
            offset: errs.first().map(|e| e.offset).unwrap_or(0),
            detail: errs
                .iter()
                .map(|e| e.to_string())
                .collect::<Vec<_>>()
                .join("; "),
            query: query_str.to_string(),
        }
    })?;

    let source_names = extract_source_names(&ast);

    // Build a flat FilterMap of equality predicates from the WHERE clause (BC-2.11.007).
    // Per-sensor classify_predicates integration deferred to wave-5
    // (see extract_push_down_filters_as_map docs for rationale).
    let where_filters = extract_push_down_filters_as_map(&ast);

    // ADR-033 T1: extract time-window bounds from the PrismQL AST predicate tree.
    // Walk Predicate::Compare nodes with op ∈ {Gt, Ge, Lt, Le} and match lhs column
    // names against datetime INDEX columns from the resolved sensor specs.
    // Safe default: (None, None) when resolved_spec_map is None or no datetime columns match.
    // The extracted bounds are passed into QueryParams.start_time / end_time below.
    let resolved_col_map = mat_ctx
        .resolved_spec_map
        .as_deref()
        .map(|spec_map| build_source_column_map(spec_map, &source_names));
    let (extracted_start_time, extracted_end_time) =
        extract_time_window_from_ast_from_query(&ast, &source_names, resolved_col_map.as_ref());

    // Step 2: Resolve client scope.
    let all_clients: Vec<OrgSlug> = options.clients.clone().unwrap_or_default();

    // Step 3: Resolve source refs to fan-out targets.
    let targets = resolve_source_refs(
        &source_names,
        &all_clients,
        &mat_ctx.adapter_registry,
        &mat_ctx.org_registry,
    )
    .await?;

    // Step 4: Fan out to sensor adapters, collecting results per source table.
    // Group results by source table name for MemTable registration.
    let mut table_batches: std::collections::HashMap<String, Vec<RecordBatch>> =
        std::collections::HashMap::new();

    // Track all sensor errors for partial-failure reporting (F-LP1-CRIT-5).
    let mut sensor_errors: Vec<String> = Vec::new();

    // Track sensor types queried for QueryResultContext.sensors_queried (BC-2.11.001).
    // ADV-W3MT-P58-HIGH-005: sensors_queried was always empty before this fix.
    let mut sensors_queried: std::collections::HashSet<String> = std::collections::HashSet::new();

    // Cross-query response cache (BC-2.07.003): clone the Arc out of mat_ctx so
    // the per-target loop can hold it alongside &mut mat_ctx borrows.
    let response_cache = mat_ctx.response_cache.clone();

    // Effective fetch-limit (BC-2.01.013 v1.14 / F-P1-CRIT-004): the limit
    // pushed into every fan-out target's `QueryParams.limit`. 0 = no-limit
    // sentinel (EC-008).
    //
    // SINGLE-BINDING COHERENCE (P1-01 / BC-2.07.005 v4.4 §Invariants, architect
    // adjudication D1): this binding feeds BOTH the response-cache key
    // derivation AND the fan-out target construction below. Do NOT introduce a
    // second derivation of the pushed limit — the limit hashed into
    // `push_down_hash` must always be the limit actually fetched, including
    // under any future pushdown-suppression logic.
    let fetch_limit: u64 = options.limit.map(|l| l as u64).unwrap_or(0);

    // F-LP1-CRIT-2/3: use fan_out() with CredentialResolver.
    // Process each target independently so virtual field injection uses the
    // correct per-target (org_id, client_id) — grouping by source_table would
    // lose per-client attribution (F-LP1-HIGH-6, AC-6).
    for target in &targets {
        // F-LP2-MED-2: cache key includes where_filters so different WHERE clauses
        // targeting the same (client, sensor, source_table) are NOT collapsed into
        // the same cache entry. This prevents stale filter leakage. (BC-2.11.005)
        let cache_key = format!(
            "{}:{:?}:{}:{}",
            target.client_id.as_str(),
            target.sensor_id,
            &target.source_table,
            serde_json::to_string(&where_filters).unwrap_or_default()
        );

        // F-LP1-MED-2: check in-query cache first (BC-2.11.005).
        if let Some(cached) = mat_ctx.cache_lookup(&cache_key) {
            // Cache hit: accumulate cached batches directly.
            for batch in cached.clone() {
                let n = batch.num_rows();
                mat_ctx.increment_record_count(n)?;
                table_batches
                    .entry(target.source_table.clone())
                    .or_default()
                    .push(batch);
            }
            continue;
        }

        // BC-2.07.005: derive the cross-query response-cache key from the
        // sensor-native push-down parameters for this target.
        let response_cache_key = response_cache.as_ref().map(|_| {
            derive_response_cache_key(
                &target.client_id,
                &target.sensor_id,
                &target.source_table,
                &where_filters,
                &extracted_start_time,
                &extracted_end_time,
                fetch_limit,
            )
        });

        // BC-2.07.003: check the cross-query response cache BEFORE issuing the
        // sensor API call. `force_refresh: true` bypasses the read (the fresh
        // response replaces the entry below). Cache errors are E-CACHE-001
        // (poisoned mutex) — unrecoverable per BC-2.07.004 §Error Cases, so
        // they propagate rather than degrade to a miss.
        if !options.force_refresh {
            if let (Some(cache), Some(key)) = (&response_cache, &response_cache_key) {
                if let Some(raw_batches) = cache.get(key)? {
                    // Cached complete response: apply the post-retrieval
                    // transformations (virtual-field injection) exactly as on
                    // the fresh-fetch path (BC-2.07.003 — "normalization and
                    // post-filters are applied after cache retrieval").
                    sensors_queried.insert(target.sensor_id.to_string());
                    let mut fetched_batches: Vec<RecordBatch> = Vec::new();
                    for batch in raw_batches {
                        let n = batch.num_rows();
                        mat_ctx.increment_record_count(n)?;
                        let annotated = crate::virtual_fields::inject_virtual_fields(
                            batch,
                            &target.sensor_id,
                            &target.client_id,
                            &target.source_table,
                        )
                        .map_err(|e| PrismError::QueryExecutionFailed {
                            detail: format!("virtual field injection failed: {e}"),
                        })?;
                        fetched_batches.push(annotated.clone());
                        table_batches
                            .entry(target.source_table.clone())
                            .or_default()
                            .push(annotated);
                    }
                    // Also seed the in-query cache so self-joins reuse the hit.
                    mat_ctx.cache_insert(cache_key, fetched_batches);
                    continue;
                }
            }
        }

        // Build the fan_out FanOutTarget (prism-sensors type, not our local type).
        // One FanOutTarget per (org_id, source_table) pair → correct per-org dispatch.
        // (F-LP1-CRIT-3: org_id matches the adapter's registered key; no random OrgId::new())
        let fan_target = {
            #[allow(deprecated)]
            prism_sensors::fanout::FanOutTarget {
                org_id: target.org_id,
                client_id: target.client_id.as_str().to_string(),
                sensor_id: target.sensor_id.clone(),
                spec: prism_sensors::adapter::SensorSpec {
                    source_table: target.source_table.clone(),
                    #[allow(deprecated)]
                    client_id: target.client_id.as_str().to_string(),
                    org_id: target.org_id,
                    sensor_config: serde_json::Value::Null,
                },
                params: prism_sensors::adapter::QueryParams {
                    cursor: None,
                    // Single-binding coherence (P1-01 / BC-2.07.005 v4.4): the
                    // SAME `fetch_limit` binding feeds the cache key above.
                    limit: fetch_limit,
                    // ADR-033 T1: populate start_time/end_time from pre-fan-out AST extraction.
                    // These were hardcoded None (F-P6-CRIT-001 dead-code gap); now wired per ADR-033.
                    start_time: extracted_start_time.clone(),
                    end_time: extracted_end_time.clone(),
                    filters: where_filters.clone(),
                },
            }
        };

        // Call fan_out with a single target — preserves per-client identity for
        // virtual field injection. (BC-3.2.001 per-org isolation)
        //
        // F-LP2-CRIT-001 (ADR-029): use fan_out_with_overlay_map when both
        // org_registry and resolved_spec_map are present, so per-org overlay
        // base_url endpoints reach the HTTP client. Falls back to bare fan_out
        // when either is absent (test/MVP mode — no overlay config loaded).
        let fan_result = match (&mat_ctx.org_registry, &mat_ctx.resolved_spec_map) {
            (Some(org_registry), Some(resolved_spec_map)) => {
                prism_sensors::fan_out_with_overlay_map(
                    vec![fan_target],
                    Arc::clone(&mat_ctx.adapter_registry),
                    Arc::clone(&mat_ctx.credential_resolver),
                    Arc::clone(org_registry),
                    Arc::clone(resolved_spec_map),
                )
                .await
            }
            _ => {
                prism_sensors::fan_out(
                    vec![fan_target],
                    Arc::clone(&mat_ctx.adapter_registry),
                    Arc::clone(&mat_ctx.credential_resolver),
                )
                .await
            }
        };
        match fan_result {
            Ok(fan_result) => {
                // Record sensor type in sensors_queried (BC-2.11.001, ADV-W3MT-P58-HIGH-005).
                // F-PASS12-HIGH-1: use Display (to_string) not Debug format — Debug produces
                // `SensorId("crowdstrike")` while the safety envelope expects `"crowdstrike"`.
                sensors_queried.insert(target.sensor_id.to_string());

                // BC-2.07.003: store the COMPLETE response (pre-virtual-field
                // injection — no query-engine transformation applied before
                // caching) in the cross-query response cache. Only complete
                // responses are cached: if this target had partial errors, the
                // result set is incomplete and caching it would serve partial
                // data for the TTL duration. `force_refresh` replaces any
                // existing entry; a forced refresh whose result is PARTIAL
                // additionally invalidates the existing entry instead of
                // retaining it (EC-07-034, P1-05 / architect adjudication D3).
                // Non-forced partial results never invalidate (availability
                // asymmetry). TTL selection by source data type happens inside
                // `put` (60s alerts / 300s devices / health not cached).
                if let (Some(cache), Some(key)) = (&response_cache, &response_cache_key) {
                    let complete = fan_result.errors.is_empty();
                    store_or_invalidate_response_cache(
                        cache,
                        key,
                        options.force_refresh,
                        complete.then(|| fan_result.successes.clone()),
                    )?;
                }

                // Collect successes with per-target virtual field injection.
                let mut fetched_batches: Vec<RecordBatch> = Vec::new();
                for batch in fan_result.successes {
                    let n = batch.num_rows();
                    mat_ctx.increment_record_count(n)?;
                    // Inject virtual fields (_sensor, _client, _source_table).
                    // Uses this target's client_id for correct per-client attribution (AC-6).
                    let annotated = crate::virtual_fields::inject_virtual_fields(
                        batch,
                        &target.sensor_id,
                        &target.client_id,
                        &target.source_table,
                    )
                    .map_err(|e| PrismError::QueryExecutionFailed {
                        detail: format!("virtual field injection failed: {e}"),
                    })?;
                    fetched_batches.push(annotated.clone());
                    table_batches
                        .entry(target.source_table.clone())
                        .or_default()
                        .push(annotated);
                }

                // Collect partial errors (BC-2.11.011).
                for fan_err in fan_result.errors {
                    // Redact internal detail — expose error code only (OBS-1 / CWE-209).
                    tracing::warn!(
                        source_table = %target.source_table,
                        sensor = ?target.sensor_id,
                        error = %fan_err,
                        "fan_out partial failure"
                    );
                    sensor_errors.push(format!(
                        "{}: sensor error ({})",
                        target.source_table,
                        fan_err.error.error_code()
                    ));
                }

                // Insert into in-query cache (BC-2.11.005, F-LP1-MED-2).
                mat_ctx.cache_insert(cache_key, fetched_batches);
            }
            Err(e) => {
                // All targets failed for this (source_table, client_id) pair.
                tracing::warn!(
                    source_table = %target.source_table,
                    client = %target.client_id,
                    error = %e,
                    "fan_out all-targets-failed (partial failure)"
                );
                sensor_errors.push(format!(
                    "{}: all targets failed ({})",
                    target.source_table,
                    e.error_code()
                ));

                // EC-07-033 (P1-05 / architect adjudication D3): a FORCED
                // refresh whose fetch failed for all targets cannot store a
                // complete replacement — invalidate the distrusted entry so
                // later non-forced queries miss and re-attempt the fetch.
                // Non-forced failures never invalidate (availability asymmetry).
                if let (Some(cache), Some(key)) = (&response_cache, &response_cache_key) {
                    store_or_invalidate_response_cache(cache, key, options.force_refresh, None)?;
                }
            }
        }
    }

    // Step 5: Register each source as a DataFusion MemTable.
    // Track how many external tables were successfully registered with data.
    let mut any_external_table_registered = false;
    let mut registered_tables: Vec<String> = Vec::new();

    for source_name in &source_names {
        // Skip internal tables (prism_*) — registered via register_internal_tables.
        if source_name.starts_with("prism_") {
            // Internal tables are registered separately by execute_inner; consider them "available".
            any_external_table_registered = true;
            continue;
        }
        let batches = table_batches.remove(source_name).unwrap_or_default();
        if !batches.is_empty() {
            register_mem_table(session_ctx, source_name, batches)?;
            any_external_table_registered = true;
            registered_tables.push(source_name.clone());
        }
        // If batches is empty, the table is NOT registered — DataFusion can't plan for it.
        // This is the "no adapter" case. We skip SQL execution in this case.
    }

    // Step 6: Execute the DataFusion SQL plan and collect results.
    // If no tables were registered (all sources empty), return empty results without
    // attempting DataFusion execution (which would fail with "table not found").
    if !any_external_table_registered {
        return Ok(MaterializationOutput {
            batches: Vec::new(),
            sensor_errors,
            registered_tables,
            sensors_queried: sensors_queried.into_iter().collect(),
        });
    }

    let collected = execute_against_session(session_ctx, query_str, &ast, table_batches).await?;

    Ok(MaterializationOutput {
        batches: collected,
        sensor_errors,
        registered_tables,
        sensors_queried: sensors_queried.into_iter().collect(),
    })
}

/// Execute the query against the DataFusion session context.
///
/// For SQL mode: runs the SQL string directly via DataFusion.
/// For Filter/Pipe mode: returns the union of all materialized `table_batches`
/// (DataFusion MemTable registration already happened; no separate SQL step).
/// (F-LP1-HIGH-1: Filter and Pipe must NOT return empty Vec)
/// Execute a PrismQL AST against a pre-configured `SessionContext`.
///
/// `pub` so that integration tests can call it directly with a manually-configured
/// `SessionContext` (e.g. with custom async UDFs + pre-registered MemTables).
/// Production callers use `run_materialization_pipeline` which calls this internally.
pub async fn execute_against_session(
    session_ctx: &SessionContext,
    query_str: &str,
    ast: &crate::ast::Ast,
    table_batches: std::collections::HashMap<String, Vec<RecordBatch>>,
) -> Result<Vec<RecordBatch>, PrismError> {
    use crate::ast::{Ast, SqlStatement};

    match ast {
        Ast::Sql(SqlStatement::Select(_)) => {
            // P5-04: read the executing session's ACTUAL pool capacity so
            // budget-exceeded errors report the configured limit (engine
            // config `memory_pool_bytes`), not the 200MB default constant.
            let pool_bytes = crate::memory::session_memory_pool_bytes(session_ctx);
            // Execute the SQL string via DataFusion.
            let df = session_ctx.sql(query_str).await.map_err(|e| {
                tracing::error!(error = %e, "DataFusion SQL planning error");
                PrismError::QueryExecutionFailed {
                    detail: "SQL planning error: <redacted; see server logs>".to_string(),
                }
            })?;
            // QRY-03: route execution errors through map_datafusion_memory_error
            // so a GreedyMemoryPool trip (ResourcesExhausted) surfaces as
            // PrismError::QueryMemoryBudgetExceeded (E-WATCHDOG-001) instead of
            // a generic QueryExecutionFailed. Non-memory errors are logged and
            // redacted inside the mapper (BC-2.11.006 EC-001).
            let stream = df
                .execute_stream()
                .await
                .map_err(|e| crate::memory::map_datafusion_memory_error(e, pool_bytes))?;
            collect_record_batch_stream(stream, pool_bytes).await
        }
        // F-LP1-HIGH-1: For Filter mode, return the union of all materialized batches.
        // The batches were already collected in the fan-out loop with virtual field injection.
        // DataFusion MemTable registration has already happened for SQL query capability;
        // for Filter mode we return the pre-collected annotated batches directly.
        // (ENRICH-4-C: Filter-mode SQL execution is a follow-up story.)
        Ast::Filter(_) => {
            // Return the union of all table_batches values (unchanged from pre-ENRICH-4-B).
            let all_batches: Vec<RecordBatch> = table_batches.into_values().flatten().collect();
            Ok(all_batches)
        }
        // ENRICH-4-B: Pipe mode now routes through the SQL-lowering path.
        //
        // `pipe_to_executable_sql` lowers the PipeQuery AST to a DataFusion SQL string,
        // which is then executed against the registered MemTables via `session_ctx.sql()`.
        // This path mirrors the `Ast::Sql(Select)` arm exactly — same pool, same stream
        // collection, same memory-error mapper — preserving all BC-2.11.006 invariants.
        //
        // The `table_batches` map is passed to the emitter for schema inspection (needed
        // by the Fields-exclude projection). The MemTables have already been registered
        // by `run_materialization_pipeline` before this function is called.
        Ast::Pipe(pipe) => {
            let pool_bytes = crate::memory::session_memory_pool_bytes(session_ctx);
            let sql = crate::pipe_sql_emitter::pipe_to_executable_sql(pipe, &table_batches)?;
            tracing::debug!(
                pipe_sql = %sql,
                event_type = "pipe.sql_lowering",
                "pipe-to-SQL lowering complete"
            );
            let df = session_ctx.sql(&sql).await.map_err(|e| {
                tracing::error!(
                    error = %e,
                    pipe_sql = %sql,
                    event_type = "pipe.sql_planning_error",
                    "pipe-to-sql DataFusion planning error"
                );
                PrismError::QueryExecutionFailed {
                    detail: "pipe SQL planning error: <redacted; see server logs>".to_string(),
                }
            })?;
            // QRY-03: route execution errors through map_datafusion_memory_error
            // so a GreedyMemoryPool trip (ResourcesExhausted) surfaces as
            // PrismError::QueryMemoryBudgetExceeded (E-WATCHDOG-001) instead of
            // a generic QueryExecutionFailed (BC-2.11.006 EC-001).
            let stream = df
                .execute_stream()
                .await
                .map_err(|e| crate::memory::map_datafusion_memory_error(e, pool_bytes))?;
            collect_record_batch_stream(stream, pool_bytes).await
        }
        _ => {
            // Other AST variants: return empty (no sensor data applicable).
            Ok(Vec::new())
        }
    }
}

// ---------------------------------------------------------------------------
// resolve_source_refs
// ---------------------------------------------------------------------------

/// Step 2: Resolve PrismQL source references to `FanOutTarget` tuples.
///
/// Each source reference in the AST (e.g., `crowdstrike_detections`) is
/// resolved against the sensor specs and the provided client scope to produce
/// one `FanOutTarget` per `(source, client)` combination. (BC-2.11.005)
///
/// # BC-2.11.011
/// If a client in `clients` does not have a sensor for the source, the
/// `(source, client)` pair is silently skipped (listed in metadata as
/// `clients_skipped`).
///
/// # F-LP1-CRIT-3 / BC-3.2.001
/// When `org_registry` is provided, resolves `OrgSlug → OrgId` for per-org
/// adapter selection. When `None`, uses a per-adapter `OrgId` from the registry
/// (test/MVP mode via `get_all_for_sensor`).
pub(crate) async fn resolve_source_refs(
    source_names: &[String],
    clients: &[OrgSlug],
    adapter_registry: &AdapterRegistry,
    org_registry: &Option<Arc<prism_core::OrgRegistry>>,
) -> Result<Vec<FanOutTarget>, PrismError> {
    let mut targets = Vec::new();

    for source_name in source_names {
        // Skip internal tables (prism_*) — handled by register_internal_tables.
        if source_name.starts_with("prism_") {
            continue;
        }
        // ADV-W3MT-P58-LOW-002 / F-LP1-CRITICAL-001: unknown table names (not prism_*,
        // not a prefix for a registered sensor) return E-QUERY-036 per EC-001.
        // This prevents silent empty results for typos or unregistered sensor names.
        //
        // Two-stage check:
        //   1. sensor_id_from_table_name: extracts and validates the prefix (returns None
        //      for empty or invalid-charset prefixes — cannot be a valid SensorId).
        //   2. is_sensor_registered: checks adapter_registry membership (returns E-QUERY-036
        //      for valid-looking prefixes with no registered adapter — unknown sensor name).
        //
        // P6-02 adjudication 2026-06-11: returns PrismError::UnknownSourceTable (E-QUERY-036)
        // instead of QueryExecutionFailed with embedded E-QUERY-006 string. The dedicated
        // variant maps to -32602 INVALID_PARAMS in error_mapping.rs (caller-resolvable).
        let Some(sensor_id) = sensor_id_from_table_name(source_name) else {
            tracing::debug!(
                source_name,
                "resolve_source_refs: unknown sensor prefix; returning E-QUERY-036"
            );
            return Err(PrismError::UnknownSourceTable {
                source_name: source_name.to_string(),
            });
        };

        // F-LP1-CRITICAL-001: after extracting the sensor prefix, verify that at least
        // one adapter is registered for it. Without this check, unknown sensor names
        // (e.g. "unknown_table") silently produce empty results rather than E-QUERY-036.
        //
        // Guard: only apply when the registry is non-empty. An empty registry indicates
        // test mode or early boot where no adapters are wired yet — in that state we
        // cannot distinguish "unknown sensor" from "known sensor not yet registered".
        // In production, the registry is always populated at boot with at least the
        // four built-in sensors; any table prefix absent from a populated registry is
        // genuinely unknown and must return E-QUERY-036.
        if !adapter_registry.is_empty() && !adapter_registry.is_sensor_registered(&sensor_id) {
            tracing::debug!(
                source_name,
                sensor_id = %sensor_id,
                "resolve_source_refs: no adapter registered for sensor prefix; returning E-QUERY-036"
            );
            return Err(PrismError::UnknownSourceTable {
                source_name: source_name.to_string(),
            });
        }

        if clients.is_empty() {
            // ALL scope with no explicit client list: fan out to ALL registered adapters
            // for this sensor type. This is the correct behavior for cross-client queries.
            //
            // F-LP1-CRIT-3/HIGH-6: use per-org adapter selection.
            // When no explicit client list, iterate all registered (org_id, adapter) pairs.
            let all_adapters = adapter_registry.get_all_for_sensor(&sensor_id);
            for (org_id, _adapter) in all_adapters {
                // Derive client_id from OrgRegistry (reverse lookup).
                // F-LP2-LOW-2: if no slug is found, emit a warn and SKIP this target.
                // BC-2.11.011 EC-005: orgs with no configured sensors are skipped silently.
                // Using a sentinel `_all` value would expose implementation details in result rows.
                let Some(client_slug) = org_registry.as_ref().and_then(|reg| reg.slug_for(&org_id))
                else {
                    // OrgRegistry absent (test/MVP mode) — fall back to test slug if available,
                    // or skip. In production (OrgRegistry present), this path means the adapter
                    // is registered for an OrgId not in the registry (configuration inconsistency).
                    tracing::warn!(
                        org_id = %org_id,
                        source_table = %source_name,
                        "resolve_source_refs: OrgId has no slug mapping in OrgRegistry; \
                         skipping target (BC-2.11.011 EC-005)"
                    );
                    // When OrgRegistry is absent (test mode), fall back to a synthetic slug
                    // derived from the org_id hex rather than `_all` sentinel.
                    // HIGH-006 (S-PLUGIN-PREREQ-C): use OrgSlug::new() (validated constructor)
                    // instead of new_unchecked(). The 8-char prefix of a UUID v7 is always
                    // valid for OrgSlug ([a-zA-Z0-9_-]{1,64}), but using the validated path
                    // removes the silent dependency on OrgId::Display format.
                    let synthetic_candidate = format!("org-{}", &org_id.to_string()[..8]);
                    let synthetic_slug_candidate = OrgSlug::new(&synthetic_candidate);
                    let synthetic_slug = if synthetic_slug_candidate.is_ok() {
                        synthetic_slug_candidate
                    } else {
                        // Fallback: if somehow the UUID prefix produces an invalid slug,
                        // use a hardcoded sentinel rather than crashing or corrupting state.
                        OrgSlug::new("synthetic-unmapped")
                    };
                    targets.push(FanOutTarget {
                        sensor_id: sensor_id.clone(),
                        client_id: synthetic_slug.clone(),
                        org_id,
                        sensor_spec: SensorSpec {
                            source_table: source_name.clone(),
                            #[allow(deprecated)]
                            client_id: synthetic_slug.as_str().to_string(),
                            org_id,
                            sensor_config: serde_json::Value::Null,
                        },
                        source_table: source_name.clone(),
                        push_down_plan: PushDownPlan::default(),
                    });
                    continue;
                };

                targets.push(FanOutTarget {
                    sensor_id: sensor_id.clone(),
                    client_id: client_slug.clone(),
                    org_id,
                    sensor_spec: SensorSpec {
                        source_table: source_name.clone(),
                        #[allow(deprecated)]
                        client_id: client_slug.as_str().to_string(),
                        org_id,
                        sensor_config: serde_json::Value::Null,
                    },
                    source_table: source_name.clone(),
                    push_down_plan: PushDownPlan::default(),
                });
            }

            // When no adapters registered: target list is empty; fan-out produces nothing.
            // BC-2.11.011 EC-005: sources with no adapters produce empty results without error.
            // F-LP2-LOW-2: no sentinel `_all` target is added — that would expose internal details.
            if adapter_registry.get_all_for_sensor(&sensor_id).is_empty() {
                tracing::debug!(
                    source_table = %source_name,
                    "resolve_source_refs: no adapters registered for sensor type; \
                     skipping fan-out (BC-2.11.011 EC-005)"
                );
            }
        } else {
            // Explicit client list: one target per client.
            // BC-2.11.011: each (source, client) pair is a separate fan-out target.
            // BC-3.2.001 postcondition 5 / E-QUERY-032: when an explicit client scope is
            // provided and the sensor is not registered for that org, RAISE a surfaced
            // operational error rather than silently returning an empty result.
            // This is "wiring not redesign" (ADR-022 §C) — the global is_sensor_registered
            // guard above confirms the sensor exists globally; this per-org check confirms
            // the requesting org is entitled to query it.
            // Reference: ADR-007 §2.2 (cross-org isolation) + BC-3.2.001 postcondition 5.
            for client_id in clients {
                // F-LP1-CRIT-3: resolve OrgSlug → OrgId via OrgRegistry if available.
                // When OrgRegistry is absent (test mode), use `get_all_for_sensor`
                // to find the OrgId associated with a registered adapter for this sensor.
                let org_id =
                    resolve_org_id(client_id, sensor_id.clone(), adapter_registry, org_registry);

                // BC-3.2.001 postcondition 5 / E-QUERY-032:
                // When the OrgRegistry is present (production), perform a per-org adapter
                // lookup. The global `is_sensor_registered` guard above confirmed the sensor
                // exists globally; this check enforces that the requesting org has a registered
                // adapter for this specific sensor. If not → RAISE E-QUERY-032 (not a silent
                // empty result). OrgRegistry absent (test mode) skips this check — no
                // org-scoped enforcement without the registry.
                //
                // `adapter_registry.get(org_id, sensor_id)` → None means the sensor is
                // registered globally but NOT for this org: cross-org isolation violation.
                if org_registry.is_some()
                    && !adapter_registry.is_empty()
                    && adapter_registry.get(org_id, &sensor_id).is_none()
                {
                    tracing::warn!(
                        sensor_id = %sensor_id,
                        org_slug = %client_id,
                        org_id = %org_id,
                        "resolve_source_refs: sensor registered globally but not for org; \
                         returning E-QUERY-032 (BC-3.2.001 postcondition 5)"
                    );
                    return Err(PrismError::SensorNotRegisteredForOrg {
                        sensor_id: sensor_id.to_string(),
                        org_slug: client_id.as_str().to_string(),
                    });
                }

                targets.push(FanOutTarget {
                    sensor_id: sensor_id.clone(),
                    client_id: client_id.clone(),
                    org_id,
                    sensor_spec: SensorSpec {
                        source_table: source_name.clone(),
                        #[allow(deprecated)]
                        client_id: client_id.as_str().to_string(),
                        org_id,
                        sensor_config: serde_json::Value::Null,
                    },
                    source_table: source_name.clone(),
                    push_down_plan: PushDownPlan::default(),
                });
            }
        }
    }

    Ok(targets)
}

/// Resolve an `OrgSlug` to its `OrgId` for adapter selection.
///
/// Priority:
/// 1. OrgRegistry lookup (production path) — exact slug → id mapping.
/// 2. First registered adapter for sensor_id (test/MVP fallback) — avoids
///    the OrgId::new() randomness that caused F-LP1-CRIT-3.
/// 3. Fresh OrgId (last resort — will miss in registry.get()).
fn resolve_org_id(
    client_id: &OrgSlug,
    sensor_id: SensorId,
    adapter_registry: &AdapterRegistry,
    org_registry: &Option<Arc<prism_core::OrgRegistry>>,
) -> OrgId {
    // Path 1: OrgRegistry lookup (production).
    if let Some(reg) = org_registry {
        if let Some(id) = reg.resolve(client_id) {
            return id;
        }
    }

    // Path 2: Fall back to first registered adapter's OrgId for this sensor id.
    // This preserves the test-path behavior where adapters are registered with
    // known OrgIds but no OrgRegistry is present.
    let adapters = adapter_registry.get_all_for_sensor(&sensor_id);
    if let Some((org_id, _)) = adapters.into_iter().next() {
        return org_id;
    }

    // Path 3: Last resort — fresh OrgId (will not match any registered adapter).
    OrgId::new()
}

// ---------------------------------------------------------------------------
// sensor_id_from_table_name
// ---------------------------------------------------------------------------

/// Map an underscore-prefixed table name to the corresponding `SensorId`.
///
/// Naming convention: `{sensor}_{table}` — extract the sensor prefix.
/// Open dispatch: any recognized prefix returns a `SensorId`. Unknown prefixes
/// return `None` (not an error — composite or internal tables have no sensor prefix).
fn sensor_id_from_table_name(table_name: &str) -> Option<SensorId> {
    // Extract the sensor prefix by splitting at the first underscore.
    // Convention: `crowdstrike_hosts` → "crowdstrike", `armis_devices` → "armis".
    // MED-002: apply .to_lowercase() to match explain.rs convention and the
    // SensorId validation charset (lowercase only).
    //
    // BC-2.11.023 / AC-011 extension: filter-mode source refs use dot notation
    // (`crowdstrike.detections`) rather than underscore notation. If the
    // underscore-split path yields a prefix with a `.` (not a valid SensorId),
    // also try splitting by `.` to extract the sensor component.
    let underscore_prefix = table_name.split('_').next()?;
    if !underscore_prefix.is_empty() && !underscore_prefix.contains('.') {
        let prefix_lower = underscore_prefix.to_lowercase();
        if let Ok(sid) = SensorId::try_from_str(prefix_lower.as_str()) {
            return Some(sid);
        }
    }
    // Dot-notation fallback: `crowdstrike.detections` → sensor = "crowdstrike".
    if let Some(dot_prefix) = table_name.split('.').next() {
        if !dot_prefix.is_empty() {
            let prefix_lower = dot_prefix.to_lowercase();
            if let Ok(sid) = SensorId::try_from_str(prefix_lower.as_str()) {
                return Some(sid);
            }
        }
    }
    None
}

// ---------------------------------------------------------------------------
// ---------------------------------------------------------------------------
// ADR-033 T1: time-window extraction helpers for run_materialization_pipeline
// ---------------------------------------------------------------------------

/// Build a `HashMap<String, Vec<ColumnSpec>>` mapping source names to their
/// column specs from the resolved spec map.
///
/// Used by `extract_time_window_from_ast` (ADR-033 T1) to identify datetime
/// INDEX columns across source tables at the pre-fan-out stage.
///
/// Key format: both `"{sensor_id}.{table_name}"` (dot-separated, PrismQL form) and
/// `"{sensor_id}_{table_name}"` (underscore-separated, common alternate form) are
/// inserted so the lookup succeeds regardless of how the source name is formed.
///
/// Only source names matching entries in `source_names` (from the PrismQL AST) are
/// included to minimize allocation.
fn build_source_column_map(
    spec_map: &std::collections::HashMap<
        prism_spec_engine::ResolvedSpecKey,
        prism_spec_engine::ResolvedSensorSpec,
    >,
    source_names: &[String],
) -> std::collections::HashMap<String, Vec<prism_spec_engine::spec_parser::ColumnSpec>> {
    let mut result: std::collections::HashMap<
        String,
        Vec<prism_spec_engine::spec_parser::ColumnSpec>,
    > = std::collections::HashMap::new();

    for resolved in spec_map.values() {
        let sensor_id = resolved.spec.sensor_id.as_str();
        for table in &resolved.spec.tables {
            // Build both common key forms for lookup.
            let dot_key = format!("{sensor_id}.{}", table.table_name);
            let underscore_key = format!("{sensor_id}_{}", table.table_name);
            // Only include if referenced by the query (avoids unnecessary work).
            let is_referenced = source_names
                .iter()
                .any(|s| *s == dot_key || *s == underscore_key);
            if is_referenced {
                result
                    .entry(dot_key.clone())
                    .or_insert_with(|| table.columns.clone());
                result
                    .entry(underscore_key)
                    .or_insert_with(|| table.columns.clone());
            }
        }
    }

    result
}

/// Wrapper that extracts time-window bounds from the PrismQL AST by delegating
/// to `pushdown::extract_time_window_from_ast` with the pre-built column map.
///
/// Returns `(start_time, end_time)` as `Option<String>` (ISO8601 formatted).
/// Both are `None` when no datetime INDEX column Compare predicates are present,
/// or when `resolved_col_map` is `None` (safe default per ADR-033 §Consequences).
fn extract_time_window_from_ast_from_query(
    ast: &crate::ast::Ast,
    source_names: &[String],
    resolved_col_map: Option<
        &std::collections::HashMap<String, Vec<prism_spec_engine::spec_parser::ColumnSpec>>,
    >,
) -> (Option<String>, Option<String>) {
    use crate::ast::{Ast, SqlStatement};

    // Only SELECT queries have a WHERE clause with time predicates.
    let Some(pred) = (match ast {
        Ast::Sql(SqlStatement::Select(sql)) => sql.where_.as_ref(),
        _ => None,
    }) else {
        return (None, None);
    };

    let source_name_refs: Vec<&str> = source_names.iter().map(String::as_str).collect();

    crate::pushdown::extract_time_window_from_ast(pred, &source_name_refs, resolved_col_map)
}

// ---------------------------------------------------------------------------
// extract_push_down_filters_as_map
// ---------------------------------------------------------------------------

/// Extract push-down filters from the AST as a `FilterMap` for `QueryParams`.
///
/// Delegates to `pushdown::predicate_tree_to_filter_map`, which collects
/// simple `field = 'value'` equality predicates from the WHERE clause and
/// builds a flat `FilterMap` from them. The result is passed to
/// `SensorAdapter::fetch` as sensor-level pre-filters.
///
/// Per-sensor `classify_predicates` integration (REQUIRED/INDEX/ADDITIONAL
/// column taxonomy) is deferred to wave-5 when per-sensor `ColumnSpec` is
/// available at the pre-fan-out stage (F-LP3-MED-1 scope decision).
/// (F-LP1-HIGH-5 / F-LP2-MED-1: replaces the previous local `collect_eq_filters` call)
fn extract_push_down_filters_as_map(ast: &crate::ast::Ast) -> prism_sensors::types::FilterMap {
    use crate::ast::{Ast, SqlStatement};

    let where_pred = match ast {
        Ast::Sql(SqlStatement::Select(sql)) => sql.where_.as_ref(),
        _ => None,
    };

    let Some(pred) = where_pred else {
        return prism_sensors::types::FilterMap::new();
    };

    crate::pushdown::predicate_tree_to_filter_map(pred)
}

/// Extract all source table names from a PrismQL AST (shallow — top-level only).
///
/// Used for fan-out target resolution (Step 2 of pipeline) where only top-level
/// sources are relevant. Subquery references are handled by `extract_source_names_recursive`.
fn extract_source_names(ast: &crate::ast::Ast) -> Vec<String> {
    extract_source_names_shallow(ast)
}

/// Shallow extraction: top-level FROM/JOIN sources only (no subquery walk).
///
/// Used for fan-out resolution — subqueries reference internal tables that are
/// registered separately; they don't drive external sensor fan-out.
fn extract_source_names_shallow(ast: &crate::ast::Ast) -> Vec<String> {
    use crate::ast::{Ast, SqlStatement};
    let mut names = Vec::new();
    match ast {
        Ast::Sql(SqlStatement::Select(sql)) => {
            names.push(sql.from.source.raw.clone());
            for join in &sql.joins {
                names.push(join.source.raw.clone());
            }
        }
        // BC-2.11.002 / BC-2.11.023 AC-011: a bare-predicate filter query has
        // source.raw = "" (no explicit sensor source). An empty source means
        // "fan out to all sensors" — do NOT add the empty string to source_names
        // or resolve_source_refs will fail with UnknownSourceTable { source_name: "" }.
        // A source-qualified filter (e.g. "crowdstrike.detections | pred") has a
        // non-empty raw source and IS pushed normally for targeted fan-out.
        Ast::Filter(filter) if !filter.source.raw.is_empty() => {
            names.push(filter.source.raw.clone());
        }
        Ast::Filter(_) => {
            // Bare predicate (no source): fan-out-to-all — add nothing to source_names.
        }
        Ast::Pipe(pipe) => {
            names.push(pipe.source.raw.clone());
            // F-LP5-LOW-1 / C-LOCAL-001 sibling fix: also collect JOIN stage
            // sources so that pipe-mode `<source> | join <internal_table> on ...`
            // is caught by the Layer 1 capability gate. Mirrors explain.rs:489-499.
            for stage in &pipe.stages {
                if let crate::ast::PipeStage::Join(js) = stage {
                    names.push(js.source.raw.clone());
                }
            }
        }
        // Non-exhaustive: ignore other variants
        _ => {}
    }
    names
}

/// Extract source table names from a PrismQL AST for capability checking (shallow).
///
/// Kept for backward compatibility. New callers should use `extract_source_names_recursive`.
/// (F-LP1-HIGH-3 — original gate; superseded by F-LP2-CRIT-1 for security gate)
/// (ADV-W3MT-P58-MED-002: targeted allow rather than blanket module-level allow)
#[allow(dead_code)]
pub(crate) fn extract_source_names_for_capability_check(ast: &crate::ast::Ast) -> Vec<String> {
    extract_source_names_shallow(ast)
}

/// Recursively extract ALL source table names from a PrismQL AST.
///
/// Walks all AST positions including:
/// - Top-level FROM clause and JOINs (source names)
/// - JOIN ON conditions (`Expr` — may contain `InSubquery`)
/// - WHERE clause predicates (including `InSubquery` / `NotInSubquery`)
/// - GROUP BY expressions (`Expr` — may contain `InSubquery`)
/// - HAVING clause predicates (including subqueries)
/// - ORDER BY expressions (`OrderExpr.expr` — may contain `InSubquery`)
/// - SELECT projection subqueries
/// - DML source_select clauses (`INSERT INTO … SELECT … FROM <source>`)
/// - DML filter predicates (`UPDATE`/`DELETE WHERE` — including `InSubquery`)
/// - Nested subqueries (recursive descent into each `SqlQuery`)
///
/// This is required for the F-LP2-CRIT-1 security fix: a subquery like
/// `WHERE id IN (SELECT trace_id FROM prism_audit)` must be caught even
/// though `prism_audit` only appears in the WHERE subquery, not the top-level FROM.
/// Coverage extended in F-LP3-CRIT-1 to also cover JOIN ON, GROUP BY, and ORDER BY
/// positions where `InSubquery` can appear.
/// Coverage extended in F-LP6-LOW-1 to also cover DML source_select and filter clauses.
///
/// Returns a deduplicated list of source table names.
pub(crate) fn extract_source_names_recursive(ast: &crate::ast::Ast) -> Vec<String> {
    use crate::ast::{Ast, SqlStatement};
    let mut names = std::collections::HashSet::new();

    match ast {
        Ast::Sql(SqlStatement::Select(sql)) => {
            walk_sql_query(sql, &mut names);
        }
        Ast::Sql(SqlStatement::Dml(dml)) => {
            // F-LP6-LOW-1: DML carries source_select (INSERT … SELECT …) and filter
            // (UPDATE/DELETE WHERE) — both can reference internal tables via subqueries.
            // target_table is parse-time write-protected for prism_* but READ access
            // through source_select / filter must still be gated by AuditRead.
            // Layer 1 sibling-pattern lineage: F-LP3-CRIT-1 → F-LP4-MED-1 → F-LP5-LOW-1 → F-LP6-LOW-1.
            if let Some(ref source_select) = dml.source_select {
                walk_sql_query(source_select, &mut names);
            }
            if let Some(ref filter) = dml.filter {
                walk_predicate(filter, &mut names);
            }
        }
        Ast::Filter(filter) => {
            names.insert(filter.source.raw.clone());
        }
        Ast::Pipe(pipe) => {
            names.insert(pipe.source.raw.clone());
            // F-LP5-LOW-1 / C-LOCAL-001 sibling fix: also collect JOIN stage
            // sources so that pipe-mode `<source> | join <internal_table> on ...`
            // is caught by the Layer 1 capability gate. Mirrors explain.rs:489-499.
            for stage in &pipe.stages {
                if let crate::ast::PipeStage::Join(js) = stage {
                    names.insert(js.source.raw.clone());
                }
            }
        }
        // SqlStatement and Ast are #[non_exhaustive]; wildcard required for future variants.
        #[allow(unreachable_patterns)]
        _ => {}
    }

    names.into_iter().collect()
}

/// Recursively walk a `SqlQuery`, collecting all referenced source table names.
///
/// Walks ALL AST positions where a subquery can appear:
/// - Top-level FROM clause and JOINs (source names)
/// - JOIN ON conditions (Expr — may contain InSubquery)
/// - WHERE clause predicates (including InSubquery / NotInSubquery)
/// - GROUP BY expressions (Expr — may contain InSubquery)
/// - HAVING clause predicates (including subqueries)
/// - ORDER BY expressions (OrderExpr.expr — may contain InSubquery)
/// - SELECT projection subqueries
/// - Function call argument lists (FuncCall::Scalar / Aggregate args — may contain InSubquery; F-LP4-MED-1)
/// - Nested subqueries (recursive descent into each SqlQuery)
fn walk_sql_query(sql: &crate::ast::SqlQuery, names: &mut std::collections::HashSet<String>) {
    use crate::ast::SelectItem;

    // Top-level FROM source.
    names.insert(sql.from.source.raw.clone());

    // JOINs: source name + ON condition expression (may contain InSubquery).
    for join in &sql.joins {
        names.insert(join.source.raw.clone());
        // Walk JOIN ON expression for subquery references (F-LP3-CRIT-1).
        walk_expr(&join.on, names);
    }

    // WHERE clause — recursively walk predicates for subqueries.
    if let Some(ref pred) = sql.where_ {
        walk_predicate(pred, names);
    }

    // GROUP BY expressions — walk each Expr for InSubquery (F-LP3-CRIT-1).
    for expr in &sql.group_by {
        walk_expr(expr, names);
    }

    // HAVING clause — recursively walk predicates for subqueries.
    if let Some(ref pred) = sql.having {
        walk_predicate(pred, names);
    }

    // ORDER BY expressions — walk each OrderExpr.expr for InSubquery (F-LP3-CRIT-1).
    for order_item in &sql.order_by {
        walk_expr(&order_item.expr, names);
    }

    // SELECT projections — walk expressions for scalar subqueries.
    for item in &sql.select.items {
        if let SelectItem::Expr { expr, .. } = item {
            walk_expr(expr, names);
        }
    }
}

/// Recursively walk a `Predicate`, collecting source table names from any subqueries.
fn walk_predicate(pred: &crate::ast::Predicate, names: &mut std::collections::HashSet<String>) {
    use crate::ast::Predicate;

    match pred {
        // `field IN (SELECT ... FROM table)` — recurse into subquery body.
        Predicate::InSubquery { subquery, .. } => {
            walk_sql_query(subquery, names);
        }
        // `AND`/`OR` with N children.
        Predicate::Logical { predicates, .. } => {
            for child in predicates {
                walk_predicate(child, names);
            }
        }
        // `NOT predicate`.
        Predicate::Not(inner) => {
            walk_predicate(inner, names);
        }
        // Compare: lhs/rhs are Expr — walk them for scalar subqueries.
        Predicate::Compare { lhs, rhs, .. } => {
            walk_expr(lhs, names);
            walk_expr(rhs, names);
        }
        // Other predicate variants have no nested subqueries.
        _ => {}
    }
}

/// Recursively walk an `Expr`, collecting source table names from any subqueries.
fn walk_expr(expr: &crate::ast::Expr, names: &mut std::collections::HashSet<String>) {
    use crate::ast::{Expr, FuncCall};

    match expr {
        // `field IN (SELECT ... FROM table)` — recurse into subquery body.
        Expr::InSubquery { subquery, .. } => {
            walk_sql_query(subquery, names);
        }
        // Binary comparison: walk both sides.
        Expr::Compare { lhs, rhs, .. } => {
            walk_expr(lhs, names);
            walk_expr(rhs, names);
        }
        // Logical: walk both sides.
        Expr::Logical { lhs, rhs, .. } => {
            walk_expr(lhs, names);
            walk_expr(rhs, names);
        }
        // NOT: walk inner.
        Expr::Not(inner) => {
            walk_expr(inner, names);
        }
        // FuncCall: walk all argument expressions — args may contain InSubquery
        // (F-LP4-MED-1: e.g. `severity_label(id IN (SELECT trace_id FROM prism_audit))`).
        Expr::FuncCall(func_call) => match func_call {
            FuncCall::Scalar { args, .. } | FuncCall::Aggregate { args, .. } => {
                for arg in args {
                    walk_expr(arg, names);
                }
            }
            // Window stub has no args yet (S-3.06 will extend this).
            FuncCall::Window { .. } => {}
        },
        // Other Expr variants (Literal, Field, VirtualField, In, Star) have no subqueries.
        _ => {}
    }
}

// ---------------------------------------------------------------------------
// register_mem_table
// ---------------------------------------------------------------------------

/// Step 6: Register a set of RecordBatches as a DataFusion `MemTable`.
///
/// The table name is the source ref string (e.g., `"crowdstrike_detections"`).
/// DataFusion table names containing dots must be quoted with backticks in
/// SQL. (BC-2.11.005 dev note)
/// Register a `Vec<RecordBatch>` as a DataFusion `MemTable` under `table_name`.
///
/// `pub` so that integration tests can pre-register tables in a manually-configured
/// `SessionContext` before calling `execute_against_session` directly.
pub fn register_mem_table(
    ctx: &SessionContext,
    table_name: &str,
    batches: Vec<RecordBatch>,
) -> Result<(), PrismError> {
    use datafusion::datasource::MemTable;

    if batches.is_empty() {
        // Empty batch list — nothing to register; skip silently.
        tracing::debug!(table_name, "register_mem_table: skipping empty batch list");
        return Ok(());
    }

    let schema = batches[0].schema();
    let mem_table = MemTable::try_new(schema, vec![batches]).map_err(|e| {
        tracing::error!(
            table_name,
            error = %e,
            "failed to create MemTable (detail redacted from client response)"
        );
        PrismError::QueryExecutionFailed {
            detail: format!(
                "failed to create MemTable for '{table_name}': <redacted; see server logs>"
            ),
        }
    })?;

    ctx.register_table(table_name, std::sync::Arc::new(mem_table))
        .map_err(|e| {
            tracing::error!(
                table_name,
                error = %e,
                "failed to register table (detail redacted from client response)"
            );
            PrismError::QueryExecutionFailed {
                detail: format!(
                    "failed to register table '{table_name}': <redacted; see server logs>"
                ),
            }
        })?;

    Ok(())
}

// ---------------------------------------------------------------------------
// collect_record_batch_stream
// ---------------------------------------------------------------------------

/// Step 8: Collect a DataFusion `SendableRecordBatchStream` to `Vec<RecordBatch>`.
///
/// Drains the stream until exhausted. Returns all collected batches.
/// The `SessionScope` is still live during collection; it is dropped after
/// this function returns (or on error). (BC-2.11.005)
///
/// # CWE-209 (Information Disclosure)
/// DataFusion error messages can contain table names, column names, and schema
/// details. The raw error is logged at `tracing::error!` for server-side
/// investigation, but the client-facing `PrismError::QueryExecutionFailed`
/// detail is redacted to `<redacted; see server logs>` to prevent internal
/// schema exposure via MCP responses.
pub(crate) async fn collect_record_batch_stream(
    stream: datafusion::physical_plan::SendableRecordBatchStream,
    pool_bytes: usize,
) -> Result<Vec<RecordBatch>, PrismError> {
    // QRY-03: route collection errors through map_datafusion_memory_error so a
    // GreedyMemoryPool trip during streaming (ResourcesExhausted) surfaces as
    // PrismError::QueryMemoryBudgetExceeded (E-WATCHDOG-001). Non-memory
    // errors are logged and redacted inside the mapper (BC-2.11.006 EC-001).
    // P5-04: `pool_bytes` is the executing session's actual pool capacity
    // (threaded from `execute_against_session` via `session_memory_pool_bytes`)
    // so the reported limit matches the pool that tripped.
    datafusion::physical_plan::common::collect(stream)
        .await
        .map_err(|e| crate::memory::map_datafusion_memory_error(e, pool_bytes))
}

// ---------------------------------------------------------------------------
// Unit tests — Layer 1 AST walker coverage (F-LP3-CRIT-1)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod walker_coverage_tests {
    //! Layer 1 AST walker coverage tests (F-LP3-CRIT-1).
    //!
    //! These tests build AST structures directly (no parser) and assert that
    //! `extract_source_names_recursive` discovers every source table name,
    //! including those hidden in JOIN ON, GROUP BY, and ORDER BY expressions.
    //!
    //! Layer 1 is a pure function test — no I/O, no async.

    use crate::{
        ast::{
            Ast, Expr, FieldPath, FromClause, Join, JoinKind, OrderExpr, SelectClause, SelectItem,
            SortDirection, SourceRef, SourceRefKind, Span, SqlQuery, SqlStatement,
        },
        materialization::extract_source_names_recursive,
    };

    // Helper: build a minimal SourceRef with raw table name.
    fn source_ref(name: &str) -> SourceRef {
        SourceRef {
            raw: name.to_string(),
            kind: SourceRefKind::Custom,
        }
    }

    // Helper: build a minimal FromClause.
    fn from(name: &str) -> FromClause {
        FromClause {
            source: source_ref(name),
            alias: None,
        }
    }

    // Helper: build a minimal SqlQuery with a single SELECT * FROM <name>.
    fn minimal_select(table: &str) -> SqlQuery {
        SqlQuery {
            select: SelectClause {
                distinct: false,
                items: vec![SelectItem::Star],
            },
            from: from(table),
            joins: vec![],
            where_: None,
            group_by: vec![],
            having: None,
            order_by: vec![],
            limit: None,
        }
    }

    // Helper: build a subquery referencing prism_audit.
    fn prism_audit_subquery() -> Box<SqlQuery> {
        Box::new(SqlQuery {
            select: SelectClause {
                distinct: false,
                items: vec![SelectItem::Expr {
                    expr: Expr::Field(FieldPath {
                        segments: vec!["trace_id".to_string()],
                        span: Span::ZERO,
                    }),
                    alias: None,
                }],
            },
            from: from("prism_audit"),
            joins: vec![],
            where_: None,
            group_by: vec![],
            having: None,
            order_by: vec![],
            limit: None,
        })
    }

    /// F-LP3-CRIT-1: JOIN ON condition containing Expr::InSubquery must be walked.
    ///
    /// Represents: `JOIN sensor_data ON id IN (SELECT trace_id FROM prism_audit)`
    ///
    /// Layer 1 must discover `prism_audit` from the JOIN ON expression.
    #[test]
    fn test_LP3_CRIT_1_join_on_subquery_discovered_by_layer1() {
        // Build: SELECT * FROM crowdstrike_detections
        //        JOIN sensor_data ON id IN (SELECT trace_id FROM prism_audit)
        let on_expr = Expr::InSubquery {
            field: FieldPath {
                segments: vec!["id".to_string()],
                span: Span::ZERO,
            },
            subquery: prism_audit_subquery(),
        };

        let mut sql = minimal_select("crowdstrike_detections");
        sql.joins = vec![Join {
            kind: JoinKind::Inner,
            source: source_ref("sensor_data"),
            alias: None,
            on: on_expr,
        }];

        let ast = Ast::Sql(SqlStatement::Select(sql));
        let names = extract_source_names_recursive(&ast);

        assert!(
            names.iter().any(|n| n == "prism_audit"),
            "F-LP3-CRIT-1: extract_source_names_recursive must discover `prism_audit` \
             hidden in JOIN ON InSubquery expression; got names: {names:?}"
        );
    }

    /// F-LP3-CRIT-1: GROUP BY expression containing Expr::InSubquery must be walked.
    ///
    /// Represents: `SELECT * FROM t GROUP BY (id IN (SELECT trace_id FROM prism_audit))`
    ///
    /// Layer 1 must discover `prism_audit` from the GROUP BY expression.
    #[test]
    fn test_LP3_CRIT_1_group_by_subquery_discovered_by_layer1() {
        // Build: SELECT * FROM crowdstrike_detections
        //        GROUP BY (id IN (SELECT trace_id FROM prism_audit))
        let group_expr = Expr::InSubquery {
            field: FieldPath {
                segments: vec!["id".to_string()],
                span: Span::ZERO,
            },
            subquery: prism_audit_subquery(),
        };

        let mut sql = minimal_select("crowdstrike_detections");
        sql.group_by = vec![group_expr];

        let ast = Ast::Sql(SqlStatement::Select(sql));
        let names = extract_source_names_recursive(&ast);

        assert!(
            names.iter().any(|n| n == "prism_audit"),
            "F-LP3-CRIT-1: extract_source_names_recursive must discover `prism_audit` \
             hidden in GROUP BY InSubquery expression; got names: {names:?}"
        );
    }

    /// F-LP3-CRIT-1: ORDER BY expression containing Expr::InSubquery must be walked.
    ///
    /// Represents: `SELECT * FROM t ORDER BY (id IN (SELECT trace_id FROM prism_audit))`
    ///
    /// Layer 1 must discover `prism_audit` from the ORDER BY expression.
    #[test]
    fn test_LP3_CRIT_1_order_by_subquery_discovered_by_layer1() {
        // Build: SELECT * FROM crowdstrike_detections
        //        ORDER BY (id IN (SELECT trace_id FROM prism_audit))
        let order_expr = Expr::InSubquery {
            field: FieldPath {
                segments: vec!["id".to_string()],
                span: Span::ZERO,
            },
            subquery: prism_audit_subquery(),
        };

        let mut sql = minimal_select("crowdstrike_detections");
        sql.order_by = vec![OrderExpr {
            expr: order_expr,
            direction: SortDirection::Asc,
        }];

        let ast = Ast::Sql(SqlStatement::Select(sql));
        let names = extract_source_names_recursive(&ast);

        assert!(
            names.iter().any(|n| n == "prism_audit"),
            "F-LP3-CRIT-1: extract_source_names_recursive must discover `prism_audit` \
             hidden in ORDER BY InSubquery expression; got names: {names:?}"
        );
    }

    /// F-LP5-LOW-1: pipe-mode JOIN sources must be walked by Layer 1.
    ///
    /// Represents queries like:
    ///   `armis_devices | join prism_audit on host == id`
    ///
    /// Prior to the fix, `extract_source_names_recursive` and
    /// `extract_source_names_shallow` only collected `pipe.source.raw`
    /// (`armis_devices`) and silently skipped `PipeStage::Join` sources
    /// (`prism_audit`). This means the Layer 1 capability gate never saw
    /// `prism_audit`, leaving a latent bypass for S-3.06+ pipe-mode JOINs.
    ///
    /// Mirror test for the C-LOCAL-001 fix already applied to explain.rs:489-499.
    #[test]
    fn test_LP5_LOW_1_pipe_join_internal_table_discovered_by_layer1() {
        use super::extract_source_names_shallow;
        use crate::ast::{JoinCondition, JoinKind, JoinStage, PipeQuery, PipeStage};

        // Build: armis_devices | join prism_audit on host == id
        let join_stage = JoinStage {
            kind: JoinKind::Inner,
            source: SourceRef {
                raw: "prism_audit".to_string(),
                kind: SourceRefKind::Internal(crate::ast::InternalTable::Audit),
            },
            on: JoinCondition::Pair(
                FieldPath {
                    segments: vec!["host".to_string()],
                    span: Span::ZERO,
                },
                FieldPath {
                    segments: vec!["id".to_string()],
                    span: Span::ZERO,
                },
            ),
        };
        let pipe_ast = Ast::Pipe(PipeQuery {
            source: SourceRef {
                raw: "armis_devices".to_string(),
                kind: SourceRefKind::Custom,
            },
            stages: vec![PipeStage::Join(join_stage)],
            write: None,
        });

        // extract_source_names_recursive must discover both sources.
        let recursive_names = extract_source_names_recursive(&pipe_ast);
        assert!(
            recursive_names.iter().any(|n| n == "armis_devices"),
            "F-LP5-LOW-1: extract_source_names_recursive must include `armis_devices` \
             (pipe primary source); got names: {recursive_names:?}"
        );
        assert!(
            recursive_names.iter().any(|n| n == "prism_audit"),
            "F-LP5-LOW-1: extract_source_names_recursive must discover `prism_audit` \
             from PipeStage::Join source; got names: {recursive_names:?}"
        );

        // extract_source_names_shallow must also discover both sources.
        let shallow_names = extract_source_names_shallow(&pipe_ast);
        assert!(
            shallow_names.iter().any(|n| n == "armis_devices"),
            "F-LP5-LOW-1: extract_source_names_shallow must include `armis_devices` \
             (pipe primary source); got names: {shallow_names:?}"
        );
        assert!(
            shallow_names.iter().any(|n| n == "prism_audit"),
            "F-LP5-LOW-1: extract_source_names_shallow must discover `prism_audit` \
             from PipeStage::Join source; got names: {shallow_names:?}"
        );
    }

    /// F-LP4-MED-1: FuncCall args containing Expr::InSubquery must be walked by Layer 1.
    ///
    /// Represents queries like:
    ///   `SELECT severity_label(id IN (SELECT trace_id FROM prism_audit)) FROM crowdstrike_detections`
    ///   (scalar FuncCall arg contains InSubquery)
    ///
    /// and:
    ///   `SELECT count(id IN (SELECT trace_id FROM prism_audit)) FROM crowdstrike_detections`
    ///   (aggregate FuncCall arg contains InSubquery)
    ///
    /// Layer 1 must discover `prism_audit` from function call argument lists.
    /// Prior to the fix, walk_expr's wildcard arm silently skipped FuncCall args.
    #[test]
    fn test_LP4_MED_1_func_call_args_subquery_discovered_by_layer1() {
        use crate::ast::{AggFunc, FuncCall, ScalarFunc};

        // ── Scalar FuncCall variant ──────────────────────────────────────────
        // Build: SELECT severity_label(id IN (SELECT trace_id FROM prism_audit))
        //        FROM crowdstrike_detections
        let in_subquery_arg = Expr::InSubquery {
            field: FieldPath {
                segments: vec!["id".to_string()],
                span: Span::ZERO,
            },
            subquery: prism_audit_subquery(),
        };
        let scalar_func_expr = Expr::FuncCall(FuncCall::Scalar {
            func: ScalarFunc::Unknown("severity_label".to_string()),
            args: vec![in_subquery_arg],
        });

        let mut sql = minimal_select("crowdstrike_detections");
        sql.select = crate::ast::SelectClause {
            distinct: false,
            items: vec![SelectItem::Expr {
                expr: scalar_func_expr,
                alias: None,
            }],
        };

        let ast = Ast::Sql(SqlStatement::Select(sql));
        let names = extract_source_names_recursive(&ast);

        assert!(
            names.iter().any(|n| n == "prism_audit"),
            "F-LP4-MED-1 (scalar): extract_source_names_recursive must discover \
             `prism_audit` hidden in FuncCall::Scalar args; got names: {names:?}"
        );

        // ── Aggregate FuncCall variant ───────────────────────────────────────
        // Build: SELECT count(id IN (SELECT trace_id FROM prism_audit))
        //        FROM crowdstrike_detections
        let in_subquery_arg2 = Expr::InSubquery {
            field: FieldPath {
                segments: vec!["id".to_string()],
                span: Span::ZERO,
            },
            subquery: prism_audit_subquery(),
        };
        let agg_func_expr = Expr::FuncCall(FuncCall::Aggregate {
            func: AggFunc::Count,
            args: vec![in_subquery_arg2],
            distinct: false,
        });

        let mut sql2 = minimal_select("crowdstrike_detections");
        sql2.select = crate::ast::SelectClause {
            distinct: false,
            items: vec![SelectItem::Expr {
                expr: agg_func_expr,
                alias: None,
            }],
        };

        let ast2 = Ast::Sql(SqlStatement::Select(sql2));
        let names2 = extract_source_names_recursive(&ast2);

        assert!(
            names2.iter().any(|n| n == "prism_audit"),
            "F-LP4-MED-1 (aggregate): extract_source_names_recursive must discover \
             `prism_audit` hidden in FuncCall::Aggregate args; got names: {names2:?}"
        );
    }

    /// F-LP6-LOW-1: DML source_select subquery must be walked by Layer 1.
    ///
    /// Represents: `INSERT INTO crowdstrike_contained_hosts SELECT host_id FROM prism_audit`
    ///
    /// The capability gate must discover `prism_audit` from the DML source_select
    /// so that AuditRead is enforced even on INSERT INTO ... SELECT queries.
    /// Lineage: F-LP3-CRIT-1 → F-LP4-MED-1 → F-LP5-LOW-1 → F-LP6-LOW-1.
    #[test]
    #[allow(non_snake_case)]
    fn test_LP6_LOW_1_dml_source_select_subquery_discovered_by_layer1() {
        use crate::write_ast::{DmlNode, DmlOperation};

        // Build: INSERT INTO crowdstrike_contained_hosts
        //        SELECT host_id FROM prism_audit
        let source_select = SqlQuery {
            select: SelectClause {
                distinct: false,
                items: vec![SelectItem::Expr {
                    expr: Expr::Field(FieldPath {
                        segments: vec!["host_id".to_string()],
                        span: Span::ZERO,
                    }),
                    alias: None,
                }],
            },
            from: from("prism_audit"),
            joins: vec![],
            where_: None,
            group_by: vec![],
            having: None,
            order_by: vec![],
            limit: None,
        };

        let dml = DmlNode {
            operation: DmlOperation::InsertInto,
            target_table: "crowdstrike_contained_hosts".to_string(),
            columns: None,
            assignments: vec![],
            filter: None,
            source_select: Some(source_select),
        };

        let ast = Ast::Sql(SqlStatement::Dml(dml));
        let names = extract_source_names_recursive(&ast);

        assert!(
            names.iter().any(|n| n == "prism_audit"),
            "F-LP6-LOW-1: extract_source_names_recursive must discover `prism_audit` \
             in DML source_select (INSERT INTO ... SELECT FROM prism_audit); got names: {names:?}"
        );
    }

    /// F-LP6-LOW-1: DML filter predicate (WHERE clause) must be walked by Layer 1.
    ///
    /// Represents: `DELETE FROM crowdstrike_contained_hosts
    ///              WHERE host_id IN (SELECT trace_host FROM prism_audit)`
    ///
    /// The capability gate must discover `prism_audit` from the DML filter
    /// so that AuditRead is enforced on DELETE/UPDATE WHERE subqueries.
    /// Lineage: F-LP3-CRIT-1 → F-LP4-MED-1 → F-LP5-LOW-1 → F-LP6-LOW-1.
    #[test]
    #[allow(non_snake_case)]
    fn test_LP6_LOW_1_dml_filter_subquery_discovered_by_layer1() {
        use crate::{
            ast::{FieldPath, Predicate, Span},
            write_ast::{DmlNode, DmlOperation},
        };

        // Build: DELETE FROM crowdstrike_contained_hosts
        //        WHERE host_id IN (SELECT trace_host FROM prism_audit)
        let subquery = Box::new(SqlQuery {
            select: SelectClause {
                distinct: false,
                items: vec![SelectItem::Expr {
                    expr: Expr::Field(FieldPath {
                        segments: vec!["trace_host".to_string()],
                        span: Span::ZERO,
                    }),
                    alias: None,
                }],
            },
            from: from("prism_audit"),
            joins: vec![],
            where_: None,
            group_by: vec![],
            having: None,
            order_by: vec![],
            limit: None,
        });

        let filter = Predicate::InSubquery {
            field: FieldPath {
                segments: vec!["host_id".to_string()],
                span: Span::ZERO,
            },
            subquery,
            negated: false,
        };

        let dml = DmlNode {
            operation: DmlOperation::Delete,
            target_table: "crowdstrike_contained_hosts".to_string(),
            columns: None,
            assignments: vec![],
            filter: Some(filter),
            source_select: None,
        };

        let ast = Ast::Sql(SqlStatement::Dml(dml));
        let names = extract_source_names_recursive(&ast);

        assert!(
            names.iter().any(|n| n == "prism_audit"),
            "F-LP6-LOW-1: extract_source_names_recursive must discover `prism_audit` \
             in DML filter (DELETE WHERE host_id IN (SELECT FROM prism_audit)); got names: {names:?}"
        );
    }
}

#[cfg(test)]
mod sensors_queried_format_tests {
    //! Unit tests for F-PASS12-HIGH-1: sensors_queried must use Display format (not Debug).
    //!
    //! SensorId::fmt (Display) → "crowdstrike"
    //! SensorId::fmt (Debug)   → "SensorId(\"crowdstrike\")"
    //!
    //! The safety envelope DataSource carries the Display form; the Debug form is unusable
    //! by downstream consumers and violates BC-2.09.008 data provenance expectations.

    use prism_core::SensorId;

    #[test]
    fn sensor_id_display_is_bare_slug_not_debug_wrapper() {
        let id = SensorId::new("crowdstrike");
        let display_form = id.to_string();
        let debug_form = format!("{:?}", id);

        // Display must be the bare slug — no wrapper.
        assert_eq!(
            display_form, "crowdstrike",
            "SensorId Display must be the bare slug; got: {display_form:?}"
        );

        // Verify Debug is the wrapped form so the distinction is clear in test output.
        assert!(
            debug_form.contains("SensorId"),
            "SensorId Debug should contain 'SensorId(...)' wrapper; got: {debug_form:?}"
        );

        // The sensors_queried insert uses to_string() (Display) — assert it produces
        // the bare slug, NOT the Debug-wrapped form with quotes and type prefix.
        let mut sensors_queried: std::collections::HashSet<String> =
            std::collections::HashSet::new();
        sensors_queried.insert(id.to_string()); // F-PASS12-HIGH-1 fix — was format!("{:?}", id)

        assert!(
            sensors_queried.contains("crowdstrike"),
            "sensors_queried must contain bare slug 'crowdstrike', not Debug form; \
             set contents: {sensors_queried:?}"
        );
        assert!(
            !sensors_queried.contains(debug_form.as_str()),
            "sensors_queried must NOT contain Debug-wrapped form '{debug_form}'"
        );
    }
}

// ---------------------------------------------------------------------------
// SID-1 unit test: BC-3.2.001 postcondition 5 — E-QUERY-032 from resolve_source_refs
// ---------------------------------------------------------------------------

#[cfg(test)]
mod cross_org_isolation_tests {
    //! SID-1 unit test: `resolve_source_refs` must return `PrismError::SensorNotRegisteredForOrg`
    //! (E-QUERY-032) when an explicitly-scoped org does not have the queried sensor registered.
    //!
    //! This test drives `resolve_source_refs` directly with a mock `AdapterRegistry`
    //! and an `OrgRegistry` — NO external DTU or live subprocess required (SID-1 compliance).
    //! The `#[ignore]`'d e2e subprocess tests (AC-012) exercise the same path end-to-end
    //! but require live DTU binaries (E2E-001 gate).
    //!
    //! BC-3.2.001 postcondition 5: when an explicit `clients` scope is given and the sensor
    //! is registered globally (for other orgs) but NOT for the requesting org, the query
    //! engine MUST return E-QUERY-032, not a silent empty result.
    //!
    //! Story: S-DEMO-002 | CRIT-001 | architect-mandated SID-1 unit test
    //! Ref: ADR-007 §2.2; BC-3.2.001 postcondition 5; error-taxonomy.md E-QUERY-032

    use std::sync::Arc;

    use async_trait::async_trait;
    use prism_core::{OrgId, OrgRegistry, OrgSlug, PrismError, SensorId};
    use prism_sensors::{
        adapter::{QueryParams, SensorSpec},
        AdapterRegistry, SensorAdapter, SensorAuth, SensorError,
    };

    /// Minimal stub adapter used only to populate the AdapterRegistry.
    ///
    /// `fetch` is never called in this unit test — the E-QUERY-032 error is returned
    /// at the query-planning boundary before any adapter dispatch.
    struct MinimalStubAdapter {
        sensor_id: SensorId,
    }

    #[async_trait]
    impl SensorAdapter for MinimalStubAdapter {
        fn sensor_type(&self) -> SensorId {
            self.sensor_id.clone()
        }

        fn sensor_name(&self) -> &'static str {
            "stub"
        }

        async fn fetch(
            &self,
            _spec: &SensorSpec,
            _params: &QueryParams,
            _auth: &dyn SensorAuth,
        ) -> Result<Vec<arrow::record_batch::RecordBatch>, SensorError> {
            // Never called in this test — the path under test returns early with E-QUERY-032.
            unreachable!("MinimalStubAdapter::fetch must not be called in the isolation test")
        }
    }

    /// SID-1 / BC-3.2.001 postcondition 5: `resolve_source_refs` with a populated
    /// OrgRegistry raises E-QUERY-032 when the explicitly-scoped org does NOT have
    /// the queried sensor registered (cross-org isolation enforcement).
    ///
    /// Test setup:
    /// - `org_b` has `claroty` registered in AdapterRegistry.
    /// - `org_a` does NOT have `claroty` registered.
    /// - OrgRegistry has both `org_a` and `org_b` mapped.
    /// - Query scope: `clients: [org_a_slug]` (explicit — NOT empty).
    ///
    /// Expected: `resolve_source_refs` returns `Err(PrismError::SensorNotRegisteredForOrg { .. })`.
    ///
    /// Mental-deletion proof: if the E-QUERY-032 guard in `resolve_source_refs` is
    /// removed, the function proceeds to build a `FanOutTarget` with the `org_a` OrgId
    /// (from `resolve_org_id` path 2 or 3) and the test FAILS — `Ok(targets)` is
    /// returned instead of the expected `Err`.
    ///
    /// This test does NOT require a live DTU or any external process (SID-1 compliance).
    #[tokio::test]
    async fn test_BC_3_2_001_unit_resolve_source_refs_cross_org_sensor_query_returns_e_query_032() {
        // --- Setup ---
        let org_a_id = OrgId::new();
        let org_b_id = OrgId::new();
        let org_a_slug = OrgSlug::new("demo-org-a").expect("valid slug");
        let org_b_slug = OrgSlug::new("demo-org-b").expect("valid slug");

        // OrgRegistry: both orgs are registered.
        let org_registry = Arc::new(OrgRegistry::new());
        org_registry
            .register(org_a_slug.clone(), org_a_id)
            .expect("org_a registration must succeed");
        org_registry
            .register(org_b_slug.clone(), org_b_id)
            .expect("org_b registration must succeed");

        // AdapterRegistry: `claroty` is registered for org_b ONLY.
        // org_a has NO adapter for claroty — this is the cross-org isolation scenario.
        let mut adapter_registry = AdapterRegistry::new();
        let claroty_sensor_id = SensorId::new("claroty");
        let stub_adapter: Arc<dyn SensorAdapter> = Arc::new(MinimalStubAdapter {
            sensor_id: claroty_sensor_id.clone(),
        });
        adapter_registry.register(org_b_id, stub_adapter);

        // Sanity checks: claroty IS registered globally; org_a has NO adapter.
        assert!(
            adapter_registry.is_sensor_registered(&claroty_sensor_id),
            "claroty must be globally registered (for org_b)"
        );
        assert!(
            adapter_registry.get(org_a_id, &claroty_sensor_id).is_none(),
            "org_a must NOT have claroty registered"
        );
        assert!(
            adapter_registry.get(org_b_id, &claroty_sensor_id).is_some(),
            "org_b MUST have claroty registered"
        );

        let adapter_registry = Arc::new(adapter_registry);
        let org_registry_opt = Some(org_registry);

        // Query scope: explicit client list → [org_a_slug].
        // Source table: claroty_alerts uses sensor prefix "claroty"
        // (convention: {sensor}_{table} — split at first underscore).
        let source_names = vec!["claroty_alerts".to_string()];
        let clients = vec![org_a_slug.clone()];

        // --- Execute ---
        let result = super::resolve_source_refs(
            &source_names,
            &clients,
            &adapter_registry,
            &org_registry_opt,
        )
        .await;

        // --- Assert ---
        // Must return Err, not Ok with empty targets.
        assert!(
            result.is_err(),
            "resolve_source_refs must return Err when sensor is not registered for the scoped org; \
             got Ok({:?})",
            result.ok()
        );

        let err = result.unwrap_err();
        match err {
            PrismError::SensorNotRegisteredForOrg {
                ref sensor_id,
                ref org_slug,
            } => {
                assert_eq!(
                    sensor_id, "claroty",
                    "E-QUERY-032: sensor_id must be 'claroty'; got: {sensor_id:?}"
                );
                assert_eq!(
                    org_slug, "demo-org-a",
                    "E-QUERY-032: org_slug must be 'demo-org-a'; got: {org_slug:?}"
                );
            }
            other => {
                panic!(
                    "resolve_source_refs must return PrismError::SensorNotRegisteredForOrg \
                     (E-QUERY-032) for cross-org isolation; got: {other:?}"
                );
            }
        }

        // The error message must contain E-QUERY-032 prefix (BC-5.39.001 error code discipline).
        let err_msg = format!("{err}");
        assert!(
            err_msg.contains("E-QUERY-032"),
            "Error display must contain 'E-QUERY-032'; got: {err_msg:?}"
        );
        assert!(
            err_msg.contains("claroty"),
            "Error display must mention sensor 'claroty'; got: {err_msg:?}"
        );
        assert!(
            err_msg.contains("demo-org-a"),
            "Error display must mention org 'demo-org-a'; got: {err_msg:?}"
        );
    }

    /// SID-1 complement: org_b CAN query claroty (positive case).
    ///
    /// Confirms the E-QUERY-032 guard does not incorrectly block org_b,
    /// which IS registered for claroty. Mental-deletion proof: if the guard
    /// is over-broad and fires for any org, this test FAILS.
    #[tokio::test]
    async fn test_BC_3_2_001_unit_resolve_source_refs_registered_org_does_not_return_error() {
        let org_b_id = OrgId::new();
        let org_b_slug = OrgSlug::new("demo-org-b").expect("valid slug");

        let org_registry = Arc::new(OrgRegistry::new());
        org_registry
            .register(org_b_slug.clone(), org_b_id)
            .expect("org_b registration must succeed");

        let mut adapter_registry = AdapterRegistry::new();
        let claroty_sensor_id = SensorId::new("claroty");
        let stub_adapter: Arc<dyn SensorAdapter> = Arc::new(MinimalStubAdapter {
            sensor_id: claroty_sensor_id.clone(),
        });
        adapter_registry.register(org_b_id, stub_adapter);
        let adapter_registry = Arc::new(adapter_registry);
        let org_registry_opt = Some(org_registry);

        let source_names = vec!["claroty_alerts".to_string()];
        let clients = vec![org_b_slug.clone()];

        let result = super::resolve_source_refs(
            &source_names,
            &clients,
            &adapter_registry,
            &org_registry_opt,
        )
        .await;

        // org_b IS registered for claroty — must NOT return E-QUERY-032.
        assert!(
            result.is_ok(),
            "resolve_source_refs must return Ok for an org that IS registered for the sensor; \
             got Err({:?})",
            result.err()
        );
        let targets = result.unwrap();
        assert_eq!(
            targets.len(),
            1,
            "Must produce exactly one FanOutTarget for org_b + claroty; got: {targets:?}"
        );
    }
}

// ---------------------------------------------------------------------------
// P6-02 unit tests: E-QUERY-036 UnknownSourceTable from resolve_source_refs
// ---------------------------------------------------------------------------

#[cfg(test)]
mod unknown_source_table_tests {
    //! SID-1 unit test: `resolve_source_refs` must return
    //! `PrismError::UnknownSourceTable` (E-QUERY-036) when:
    //! (a) the table name prefix fails `sensor_id_from_table_name` validation, OR
    //! (b) the prefix is valid but not registered in a non-empty AdapterRegistry.
    //!
    //! This is the P6-02 adjudication 2026-06-11 regression test.
    //! Before the fix, both sites returned `QueryExecutionFailed { detail: "E-QUERY-006: ..." }`,
    //! routing caller-resolvable errors to -32000 INTERNAL_ERROR with a redacted message.
    //! After the fix, `UnknownSourceTable` routes to -32602 INVALID_PARAMS.
    //!
    //! No external DTU or subprocess required (SID-1 compliance).
    //! Ref: error-taxonomy.md v1.73 E-QUERY-036; BC-2.11.007 EC-001; P6-02 adjudication.

    use std::sync::Arc;

    use async_trait::async_trait;
    use prism_core::{OrgId, PrismError, SensorId};
    use prism_sensors::{
        adapter::{QueryParams, SensorSpec},
        AdapterRegistry, SensorAdapter, SensorAuth, SensorError,
    };

    struct StubAdapterForUnknownTest {
        sensor_id: SensorId,
    }

    #[async_trait]
    impl SensorAdapter for StubAdapterForUnknownTest {
        fn sensor_type(&self) -> SensorId {
            self.sensor_id.clone()
        }

        fn sensor_name(&self) -> &'static str {
            "stub-unknown-test"
        }

        async fn fetch(
            &self,
            _spec: &SensorSpec,
            _params: &QueryParams,
            _auth: &dyn SensorAuth,
        ) -> Result<Vec<arrow::record_batch::RecordBatch>, SensorError> {
            // Never called — the path under test returns UnknownSourceTable before adapter dispatch.
            unreachable!("StubAdapterForUnknownTest::fetch must not be called in E-QUERY-036 test")
        }
    }

    /// P6-02: `resolve_source_refs` returns `PrismError::UnknownSourceTable` (E-QUERY-036)
    /// when the registry is non-empty and the queried table prefix is not registered.
    ///
    /// Mental-deletion proof: removing the `UnknownSourceTable` return from
    /// `resolve_source_refs` would cause this test to fail with `Ok(...)` or a
    /// different error variant — neither of which is `UnknownSourceTable`.
    #[tokio::test]
    async fn test_BC_2_11_007_resolve_source_refs_unregistered_prefix_returns_unknown_source_table()
    {
        let org_id = OrgId::new();
        let mut registry = AdapterRegistry::new();
        registry.register(
            org_id,
            Arc::new(StubAdapterForUnknownTest {
                sensor_id: SensorId::new("crowdstrike"),
            }),
        );

        let source_names = vec!["ghost_sensor.devices".to_string()];
        let clients = vec![];
        let org_registry = None;

        let result =
            super::resolve_source_refs(&source_names, &clients, &registry, &org_registry).await;

        let err = result.expect_err(
            "resolve_source_refs must return Err for unregistered sensor prefix; got Ok",
        );
        assert!(
            matches!(err, PrismError::UnknownSourceTable { .. }),
            "error must be PrismError::UnknownSourceTable (E-QUERY-036); got: {err:?}"
        );
        let display = err.to_string();
        assert!(
            display.contains("E-QUERY-036"),
            "error display must contain 'E-QUERY-036'; got: {display}"
        );
        assert!(
            display.contains("ghost_sensor.devices"),
            "error display must include the source_name; got: {display}"
        );
    }

    /// P6-02: `resolve_source_refs` returns `PrismError::UnknownSourceTable` (E-QUERY-036)
    /// when the table name prefix fails `sensor_id_from_table_name` validation
    /// (e.g. empty prefix, invalid charset).
    ///
    /// The `...` table name has no valid sensor-id prefix — `sensor_id_from_table_name`
    /// returns `None`. Even with an empty registry (test-mode guard inactive), the
    /// prefix-extraction failure fires unconditionally.
    #[tokio::test]
    async fn test_BC_2_11_007_resolve_source_refs_invalid_prefix_returns_unknown_source_table() {
        let mut registry = AdapterRegistry::new();
        // Register a sensor to make the registry non-empty (triggers the guard).
        let org_id = OrgId::new();
        registry.register(
            org_id,
            Arc::new(StubAdapterForUnknownTest {
                sensor_id: SensorId::new("crowdstrike"),
            }),
        );

        // A dot-only name: sensor_id_from_table_name returns None for ".".
        let source_names = vec![".invalid".to_string()];
        let clients = vec![];
        let org_registry = None;

        let result =
            super::resolve_source_refs(&source_names, &clients, &registry, &org_registry).await;

        let err = result
            .expect_err("resolve_source_refs must return Err for invalid table prefix; got Ok");
        assert!(
            matches!(err, PrismError::UnknownSourceTable { .. }),
            "error must be PrismError::UnknownSourceTable (E-QUERY-036) for invalid prefix; got: {err:?}"
        );
    }
}
