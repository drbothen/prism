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
use prism_core::{OrgId, OrgSlug, PrismError, SensorId, UnknownSourceTableDetails};
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
    /// Table registry for plan-time temporal literal validation (ADR-052 §D4 Option A).
    ///
    /// When `Some`, `check_temporal_literals` can resolve column types for
    /// `Literal::RawTemporalLiteral` nodes found in the parsed AST. When `None`
    /// (legacy / test mode without spec engine wiring), the temporal check is skipped.
    /// Wired from `QueryEngine` via `with_table_registry` (S-PRISMQL-NATIVE-TEMPORAL-TYPING-001).
    pub(crate) table_registry: Option<Arc<crate::table_registry::TableRegistry>>,
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
            table_registry: None,
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

    /// Attach the table registry for plan-time temporal literal validation (ADR-052 §D4).
    ///
    /// Called by `QueryEngine` so `check_temporal_literals` can resolve column
    /// types for `Literal::RawTemporalLiteral` nodes in the parsed query AST.
    /// (S-PRISMQL-NATIVE-TEMPORAL-TYPING-001)
    pub fn with_table_registry(
        mut self,
        registry: Arc<crate::table_registry::TableRegistry>,
    ) -> Self {
        self.table_registry = Some(registry);
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
// seed_armis_entity_discriminator
// ---------------------------------------------------------------------------

/// Seed the Armis entity discriminator AQL into `filters` when absent.
///
/// Armis uses a single `/api/v1/search?aql=<value>` endpoint for BOTH the
/// `devices` and `alerts` tables. The `in:alerts` / `in:devices` AQL prefix
/// is the sole entity discriminator: omitting it causes the DTU to default
/// to device records (EC-001 in `prism-dtu-armis/src/routes/search.rs`).
///
/// Without an explicit `WHERE aql = '...'` predicate the query planner
/// produces an empty `aql` entry in `filters`, so the path template
/// `/api/v1/search?aql=${query.filter.aql}` sends `?aql=` (blank) → DTU
/// returns device records → `armis_alerts` queries silently return 0 rows
/// after OCSF normalization filters severity/status (F-L2-CRIT-001).
///
/// ## Behaviour
///
/// - `source_table == "armis_alerts"` → seeds `filters["aql"] = "in:alerts"`
///   when `"aql"` is absent or empty.
/// - `source_table == "armis_devices"` → seeds `filters["aql"] = "in:devices"`
///   when `"aql"` is absent or empty.
/// - User-supplied non-empty `WHERE aql = '...'` predicates are preserved
///   verbatim; this function does NOT overwrite them.
/// - All other `source_table` values are left untouched (no mutation).
///
/// ## Injection point
///
/// Called per-target in `run_materialization_pipeline` immediately before
/// constructing the `prism_sensors::adapter::QueryParams` for the fan-out.
/// The returned `FilterMap` is used instead of the shared `where_filters`
/// clone so cross-target contamination is impossible.
///
/// S-DEMO-FIDELITY-REMEDIATION-001 / F-L2-CRIT-001.
pub(crate) fn seed_armis_entity_discriminator(
    source_table: &str,
    mut filters: prism_sensors::types::FilterMap,
) -> prism_sensors::types::FilterMap {
    // Determine the discriminator value for this source_table.
    let discriminator = match source_table {
        "armis_alerts" => Some("in:alerts"),
        "armis_devices" => Some("in:devices"),
        _ => None,
    };

    if let Some(disc) = discriminator {
        // Only seed when absent or empty — never clobber a user-supplied AQL predicate.
        let existing = filters.get("aql").and_then(|v| v.as_str()).unwrap_or("");
        if existing.trim().is_empty() {
            filters.insert(
                "aql".to_string(),
                serde_json::Value::String(disc.to_string()),
            );
        }
    }

    filters
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

    // Step 1a: Inject NOW() constant (BC-2.11.021 / AC-004 / ADR-044).
    //
    // Capture Utc::now() ONCE per query so all NOW() occurrences resolve to the
    // same instant (consistency invariant). The injected AST is used for:
    //   - Pipe-mode stages: `pipe_to_executable_sql` → `expr_to_sql` must receive
    //     `Literal::Timestamp(now)` not `Expr::Now` (TimestampArithmetic emission).
    //   - Filter-mode predicates: injected before push-down filter extraction.
    //   - SqlPipe pipe stages: same as Pipe-mode above.
    //
    // NOTE: For SQL-mode, the injected AST is re-emitted via PqlNormalizer::normalize
    // (BC-2.11.021 / ADR-044 D4 / D-1333 Option A) so DataFusion receives the
    // plan-pinned `'<iso>'` constant literal rather than a runtime NOW() call.
    // For SqlPipe-head SQL, the head is normalized from the injected spq.head AST
    // (see plan_pinned_head_sql below). For Pipe-mode, inject_now fires on stage
    // expressions (Expr::Now replaced inline). All modes are covered.
    //
    // NOTE: `mut` is required here for Step 1c (check_temporal_literals) which
    // mutably coerces RawTemporalLiteral → Literal::String for String-column comparisons
    // (ADR-052 §D4 Option-A coercion arm; S-PRISMQL-NATIVE-TEMPORAL-TYPING-001).
    let mut ast = {
        use crate::ast::{Expr, Literal, TimestampLiteral};
        use chrono::Utc;
        let now: chrono::DateTime<Utc> = Utc::now();
        let now_iso = now.to_rfc3339();
        let now_ts = TimestampLiteral {
            iso8601: now_iso,
            instant: now,
        };
        let now_literal_expr = Expr::Literal(Literal::Timestamp(now_ts));
        // F-P3-FRESH-CRIT-001 (Site 2): inject_now is now fallible — DateTime range
        // overflow in the constant-fold arm produces E-QUERY-001 instead of panicking.
        crate::inject_now(ast, &now_literal_expr).map_err(|errs| PrismError::QueryParseFailed {
            offset: errs.first().map(|e| e.offset).unwrap_or(0),
            detail: errs
                .iter()
                .map(|e| e.to_string())
                .collect::<Vec<_>>()
                .join("; "),
            query: query_str.to_string(),
        })?
    };

    // Step 1b: Plan-time FORBID-BOTH check (BC-2.11.020 INV-FORBID-BOTH-PERMANENT /
    // ADR-043 §C §D4): hoisted to BEFORE fan-out so it fires data-independently.
    //
    // Previously `plan_sqlpipe_query` was called inside `execute_against_session`
    // (after fan-out). If the adapter returned Ok(vec![]) (no batches), the Step-6
    // early-return guard `if !any_external_table_registered` fired first, bypassing
    // FORBID-BOTH. By hoisting here — right after parse and inject_now — the check
    // runs unconditionally regardless of adapter row count.
    if let crate::ast::Ast::SqlPipe(ref spq) = ast {
        crate::plan_sqlpipe_query(spq)?;
    }

    // Step 1c: ADR-052 §D4 Option-A four-way dispatch for RawTemporalLiteral nodes.
    // Fires after inject_now (so Timestamp literals are already resolved) and before
    // fan-out (no sensor I/O has occurred yet — this is still a plan-time gate).
    //
    // Gate ordering: E-QUERY-037 → E-QUERY-038 → E-QUERY-039 → E-QUERY-041 → DataFusion.
    // E-QUERY-037/038/039 fire in engine.rs before run_materialization_pipeline is called.
    //
    // Four-way dispatch on each RawTemporalLiteral found in a comparison:
    //   - vs Datetime column  → Err(TemporalLiteralUnparseable)  [E-QUERY-041]
    //   - vs String column    → COERCE in-place to Literal::String
    //   - vs Integer/Float/Bool column → Err(QueryTypeMismatch)  [E-QUERY-002]
    // RawTemporalLiteral in non-comparison position (SELECT / GROUP BY / ORDER BY)
    //   → COERCE in-place to Literal::String (ADR-052 §D4 v1.8 OBS-2)
    // skip_projection=false: full walk (this runs after check_table_availability, so
    // the table is confirmed to exist and the projection check is appropriate).
    check_temporal_literals(&mut ast, mat_ctx.table_registry.as_deref(), false)?;

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
    let mut targets = resolve_source_refs(
        &source_names,
        &all_clients,
        &mat_ctx.adapter_registry,
        &mat_ctx.org_registry,
    )
    .await?;

    // Step 3b: Bare-filter fan-out-to-all.
    //
    // BC-2.11.023 AC-011 / BC-2.11.002: A bare-predicate filter query (`severity = 'HIGH'`
    // with no explicit source) fans out to ALL registered sensors. `resolve_source_refs`
    // is called with empty `source_names` and returns empty targets — the bare-filter
    // semantics require a separate path that enumerates the registry directly.
    //
    // This path synthesizes one FanOutTarget per (OrgId, SensorId) entry in the adapter
    // registry. The `source_table` for each target is the sensor_id string (no explicit
    // table qualifier), matching the bare-filter DataFusion enumeration in
    // `execute_against_session::Ast::Filter` (which iterates session catalog tables and
    // excludes `prism_*` internal tables).
    if matches!(&ast, crate::ast::Ast::Filter(f) if f.source.raw.is_empty())
        && targets.is_empty()
        && !mat_ctx.adapter_registry.is_empty()
    {
        // Enumerate all (OrgId, SensorId) pairs registered in the adapter registry.
        // For each pair, synthesize a FanOutTarget using sensor_id as source_table.
        // client_id uses the same synthetic-slug fallback as resolve_source_refs lines
        // 1284-1311 (no OrgRegistry available in bare-filter test path).
        for sensor_id in mat_ctx.adapter_registry.registered_sensor_ids() {
            let adapters = mat_ctx.adapter_registry.get_all_for_sensor(&sensor_id);
            for (org_id, _adapter) in adapters {
                let client_id = OrgSlug::new(format!("org-{}", &org_id.to_string()[..8]));
                let source_table = sensor_id.as_ref().to_string();
                targets.push(FanOutTarget {
                    sensor_id: sensor_id.clone(),
                    client_id: client_id.clone(),
                    org_id,
                    sensor_spec: SensorSpec {
                        source_table: source_table.clone(),
                        #[allow(deprecated)]
                        client_id: client_id.as_str().to_string(),
                        org_id,
                        sensor_config: serde_json::Value::Null,
                    },
                    source_table,
                    push_down_plan: PushDownPlan::default(),
                });
            }
        }
    }

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
        //
        // F-L2-CRIT-001 (S-DEMO-FIDELITY-REMEDIATION-001): seed the Armis entity
        // discriminator AQL when absent. `armis_alerts` → "in:alerts"; `armis_devices` →
        // "in:devices". Without this, a query without an explicit `WHERE aql = '...'`
        // clause sends a blank `?aql=` to the DTU, which defaults to device records,
        // causing `armis_alerts` queries to silently return 0 matching rows.
        let target_filters =
            seed_armis_entity_discriminator(&target.source_table, where_filters.clone());
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
                    // F-L2-CRIT-001: use target_filters (discriminator-seeded) rather than
                    // raw where_filters.clone() so armis_alerts gets "in:alerts" AQL.
                    filters: target_filters,
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
            // Normalize the table name for DataFusion registration: replace the first dot with
            // underscore (e.g. "crowdstrike.detections" → "crowdstrike_detections").
            // DataFusion's `register_table` treats dots as catalog/schema separators and rejects
            // names like "crowdstrike.detections". Filter-mode source refs use dot notation
            // (BC-2.11.023), so normalization is required here. SQL-mode queries already use
            // underscore notation by convention (BC-2.11.002), so normalization is a no-op for them.
            let normalized_name = datafusion_table_name(source_name);
            register_mem_table(session_ctx, &normalized_name, batches)?;
            any_external_table_registered = true;
            registered_tables.push(normalized_name);
        }
        // If batches is empty, the table is NOT registered — DataFusion can't plan for it.
        // This is the "no adapter" case. We skip SQL execution in this case.
    }

    // Step 5b: Bare-filter fallthrough — register any remaining `table_batches` entries.
    //
    // For bare-filter fan-out (BC-2.11.023 AC-011 / Step 3b above), `source_names` is empty
    // so the step-5 loop above does nothing. After the fan-out in step 4, `table_batches`
    // holds entries keyed by sensor_id string (e.g. "stub", "crowdstrike"). Register them
    // here so `execute_against_session::Ast::Filter` can enumerate them via the DataFusion
    // session catalog and apply the predicate.
    //
    // Only runs when `source_names` is empty (bare filter) AND there are remaining batches
    // (non-empty fan-out produced data). Non-bare-filter queries fully drain `table_batches`
    // in the step-5 loop above — this block is a no-op for those paths.
    if source_names.is_empty() {
        for (table_key, batches) in table_batches.drain() {
            if table_key.starts_with("prism_") {
                any_external_table_registered = true;
                continue;
            }
            if !batches.is_empty() {
                let normalized_name = datafusion_table_name(&table_key);
                register_mem_table(session_ctx, &normalized_name, batches)?;
                any_external_table_registered = true;
                registered_tables.push(normalized_name);
            }
        }
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
    // F-P1-MED-002: `_query_str` was previously used by the `Ast::Sql(Select)` fallback
    // `unwrap_or_else(|| query_str.to_string())`. That fallback is replaced by
    // `ok_or_else(|| PrismError::QueryExecutionFailed{...})?` — consistent with the
    // OBS-1 fix on the `Ast::SqlPipe` arm. The parameter is retained for API stability.
    _query_str: &str,
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
            // BC-2.11.021 / ADR-044 D4 / D-1333 Option A (plan-time pinning):
            // Re-emit the SQL from the inject_now-ed AST (which has folded all
            // TimestampArithmetic nodes into bare Literal::Timestamp constants).
            // DataFusion receives the plan-pinned constant rather than runtime `NOW()`
            // or `NOW() - INTERVAL '...'`. This ensures:
            //   1. The temporal bound used by DataFusion's post-filter is IDENTICAL
            //      to the plan-time bound used for ADR-033 T1 push-down (QueryParams).
            //   2. No PrismQL INTERVAL syntax (`'24h'`) reaches DataFusion (which uses
            //      different syntax), eliminating the parsing ambiguity.
            //   3. Cross-mode consistency: SQL/filter/pipe all see the same pinned instant.
            //
            // ADR-052 §D4 v1.5 SQL-Mode DataFusion Emission Addendum (HIGH-1):
            // `normalize_for_datafusion` emits `arrow_cast('<iso>', 'Timestamp(Microsecond,
            // Some("UTC"))')` for `Literal::Timestamp` values instead of the bare `'<iso>'`
            // that `normalize` emits.  The bare form relies on DataFusion's implicit
            // string→timestamp coercion (RISK-1), which is non-deterministic across
            // DataFusion minor versions.  The `arrow_cast` form produces an explicit
            // `Timestamp(Microsecond, Some("UTC"))` literal that compares directly against
            // `Timestamp(Microsecond, UTC)` columns without implicit coercion.
            //
            // BC-2.11.018 round-trip invariant: `normalize` (PQL round-trip path) is NOT
            // used here — keeping the two paths separate ensures `normalize_literal` is
            // never changed to emit `arrow_cast`.
            //
            // F-P1-MED-002 / OBS-1 sibling hardening: `normalize_for_datafusion` returns
            // `None` on the same conditions as `normalize` (unfolded temporal expressions,
            // both-quote-string guard). `ok_or_else` converts None → structured
            // PrismError::QueryExecutionFailed. (BC-2.11.021, ADR-044)
            let plan_pinned_sql = crate::ast::PqlNormalizer::normalize_for_datafusion(ast)
                .ok_or_else(|| PrismError::QueryExecutionFailed {
                    detail: "SQL normalization failed: query contains unfolded temporal \
                                 expression (Expr::Now / Expr::Interval / \
                                 Expr::TimestampArithmetic). This indicates inject_now did not \
                                 fully constant-fold the AST before normalization. Retry the \
                                 query or report to support."
                        .to_string(),
                })?;
            // Execute the plan-pinned SQL string via DataFusion.
            let df = session_ctx.sql(&plan_pinned_sql).await.map_err(|e| {
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
        // BC-2.11.023 AC-011 / ADR-046 D4: Filter mode predicate application via DataFusion.
        //
        // Filter mode lowers to `SELECT * FROM <table> WHERE <predicate>` and executes through
        // DataFusion, applying the predicate to the materialized rows. This is the ENRICH-4-C
        // implementation: filter predicates are now genuinely applied (not just pass-through).
        //
        // Source-qualified filter (`crowdstrike.detections | severity = 'HIGH'`):
        //   → `SELECT * FROM crowdstrike_detections WHERE severity = 'HIGH'`
        //
        // Bare filter (`severity = 'HIGH'`, fan-out-to-all):
        //   → `SELECT * FROM <table> WHERE severity = 'HIGH'` for each registered table,
        //   results unioned. Each registered table gets its own SQL scan so DataFusion can
        //   apply the predicate independently per schema.
        //
        // Returns empty Vec when no tables were registered (no-sensor engine: table_batches
        // is empty because run_materialization_pipeline early-returned before this function
        // for the no-adapter case; this arm handles only the with-data path).
        //
        // Memory budget: uses the same GreedyMemoryPool session as SQL/Pipe modes.
        Ast::Filter(filter) => {
            let pool_bytes = crate::memory::session_memory_pool_bytes(session_ctx);

            // LOW-1 fix — F-P1-MED-001 sibling parity (BC-2.11.021 / ADR-044):
            // Guard against bare `Expr::Interval` (or other unfolded temporal expressions —
            // `Expr::Now`, `Expr::TimestampArithmetic`) reaching `normalize_predicate_pub`.
            //
            // The SQL/SqlPipe arms are protected by `PqlNormalizer::normalize`'s
            // `ast_has_unfolded_temporal_expr` pre-check (F-P1-MED-001). Without this
            // guard, a bare `Expr::Interval` RHS (e.g. `timestamp > INTERVAL '24h'` —
            // accepted by `build_temporal_rhs_parser`, NOT folded by `inject_now`) reaches
            // `normalize_expr`'s catch-all → emits empty string → `WHERE timestamp > `
            // (malformed) to DataFusion → generic redacted `QueryExecutionFailed` instead
            // of a clear structured error.
            //
            // This guard makes the Filter arm consistent: any unfolded temporal expression
            // returns the same E-QUERY-034 structured error the SQL/SqlPipe arms return.
            if crate::ast::PqlNormalizer::predicate_has_unfolded_temporal_pub(&filter.predicate) {
                return Err(PrismError::QueryExecutionFailed {
                    detail: "filter SQL normalization failed: predicate contains unfolded \
                             temporal expression (Expr::Now / Expr::Interval / \
                             Expr::TimestampArithmetic). Bare interval comparisons such as \
                             `timestamp > INTERVAL '24h'` are not supported in filter mode — \
                             use `timestamp > NOW() - INTERVAL '24h'` which constant-folds at \
                             plan time. Retry or report to support."
                        .to_string(),
                });
            }

            // Lower the predicate to a DataFusion-compatible SQL WHERE clause.
            //
            // OBS-1 fix (S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001 PR-LEVEL): use
            // `predicate_to_datafusion_sql` (from `pipe_sql_emitter`) instead of
            // `PqlNormalizer::normalize_predicate_pub`.
            //
            // `normalize_predicate_pub` emits double-quoted form (`"O'Brien"`) for string
            // literals containing `'` — correct for PQL round-trips (BC-2.11.018) but WRONG
            // for DataFusion SQL: DataFusion follows ANSI SQL and treats double-quoted tokens as
            // IDENTIFIER references (column names), not string literals.  DataFusion would
            // receive `WHERE name = "O'Brien"` and look for a column named `O'Brien` → no match /
            // planning error instead of the expected string equality filter.
            //
            // `predicate_to_datafusion_sql` uses `escape_sql_string` which replaces `'` with
            // `''` (standard SQL single-quote escaping) and always wraps in single quotes,
            // producing `WHERE name = 'O''Brien'` — the correct DataFusion SQL form.
            let where_clause = crate::pipe_sql_emitter::predicate_to_datafusion_sql(
                &filter.predicate,
            )
            .map_err(|e| PrismError::QueryExecutionFailed {
                detail: format!("filter SQL lowering failed: {e}. Retry or report to support."),
            })?;

            // Determine the set of registered source tables to apply the predicate against.
            // Source-qualified: only the specified source. Bare: all registered tables.
            //
            // NOTE: `table_batches` is EMPTY at this point — run_materialization_pipeline step 5
            // already consumed all entries via `table_batches.remove(source_name)` and registered
            // them as DataFusion MemTables. For the bare-filter case we enumerate the session
            // context's default catalog to find registered external tables (excluding prism_*
            // internal tables, which are registered separately).
            let table_names: Vec<String> = if filter.source.raw.is_empty() {
                // Bare predicate: enumerate all tables registered in the DataFusion session.
                // DataFusion registers MemTables into the default catalog ("datafusion") /
                // default schema ("public"). Exclude prism_* tables (internal tables not
                // relevant to sensor-data filter queries).
                session_ctx
                    .catalog("datafusion")
                    .and_then(|cat| cat.schema("public"))
                    .map(|schema| {
                        schema
                            .table_names()
                            .into_iter()
                            .filter(|n| !n.starts_with("prism_"))
                            .collect()
                    })
                    .unwrap_or_default()
            } else {
                // Source-qualified: normalize the source ref using `datafusion_table_name`
                // (dot→underscore) to match the normalized name used when registering the
                // MemTable in step 5. The raw source ref "crowdstrike.detections" is
                // registered as "crowdstrike_detections" — the SQL must use the same form.
                vec![datafusion_table_name(&filter.source.raw)]
            };

            // Execute `SELECT * FROM <table> WHERE <predicate>` for each table and union.
            let mut all_batches: Vec<RecordBatch> = Vec::new();
            for table_name in &table_names {
                // Table names are DataFusion-normalized (dots replaced with underscores)
                // matching the normalization applied in step 5. Use plain identifier quoting.
                let filter_sql = format!("SELECT * FROM {table_name} WHERE {where_clause}");
                tracing::debug!(
                    filter_sql = %filter_sql,
                    event_type = "filter.sql_lowering",
                    "filter-to-SQL lowering complete"
                );
                let df = session_ctx.sql(&filter_sql).await.map_err(|e| {
                    tracing::error!(
                        error = %e,
                        filter_sql = %filter_sql,
                        event_type = "filter.sql_planning_error",
                        "filter-to-sql DataFusion planning error"
                    );
                    PrismError::QueryExecutionFailed {
                        detail: "filter SQL planning error: <redacted; see server logs>"
                            .to_string(),
                    }
                })?;
                let stream = df
                    .execute_stream()
                    .await
                    .map_err(|e| crate::memory::map_datafusion_memory_error(e, pool_bytes))?;
                let batches = collect_record_batch_stream(stream, pool_bytes).await?;
                all_batches.extend(batches);
            }
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
        // BC-2.11.020: SQL→Pipe composition mode.
        //
        // FORBID-BOTH invariant (E-QUERY-040, BC-2.11.020 INV-FORBID-BOTH-PERMANENT,
        // ADR-043 §C §D4) is now enforced by `run_materialization_pipeline` Step 1b
        // (hoisted BEFORE fan-out and data-availability guard). Duplicate call removed
        // here to avoid confusion — the check fires exactly once per query, at parse time.
        //
        // For valid queries, the head SQL is wrapped in a CTE (`_sqlpipe_head`)
        // and the pipe stages are applied on top via the same SQL lowering path
        // as `Ast::Pipe` — same pool, same stream collection, same memory-error
        // mapper — preserving all BC-2.11.006 invariants.
        Ast::SqlPipe(spq) => {
            let pool_bytes = crate::memory::session_memory_pool_bytes(session_ctx);
            // BC-2.11.021 / ADR-044 D4 / D-1333 Option A (plan-time pinning):
            // Compute the SqlPipe head SQL from the inject_now-ed AST (spq.head has
            // the folded Literal::Timestamp) rather than the raw query_str[..split].
            // This ensures DataFusion receives the plan-pinned constant for the head SQL.
            let plan_pinned_head_sql = {
                use crate::ast::{Ast as InnerAst, SqlStatement};
                // OBS-1 (BC-2.11.021 / ADR-044 D4): normalize MUST succeed for a
                // well-formed SqlPipe head. If it returns None, the fallback would
                // silently pass `query_str` (which may contain runtime NOW() or
                // INTERVAL) to DataFusion, violating BC-2.11.021 plan-pinning.
                // Return a structured error instead — the query can be retried.
                crate::ast::PqlNormalizer::normalize_for_datafusion(&InnerAst::Sql(
                    SqlStatement::Select(spq.head.clone()),
                ))
                .ok_or_else(|| PrismError::QueryExecutionFailed {
                    detail: "SqlPipe head SQL normalization failed: plan-pinned SQL could not be \
                             derived. This is an internal error; retry the query or report to \
                             support."
                        .to_string(),
                })?
            };
            let sql = crate::pipe_sql_emitter::sqlpipe_to_executable_sql(
                &plan_pinned_head_sql,
                spq,
                &table_batches,
            )?;
            // SAP-1: reuse existing catalog event type `pipe.sql_lowering` (BC-2.16.002 row 178).
            // SqlPipe lowering is semantically identical to Pipe lowering — same execution path,
            // same diagnostic information. No new catalog row needed.
            tracing::debug!(
                pipe_sql = %sql,
                event_type = "pipe.sql_lowering",
                "sqlpipe-to-SQL lowering complete"
            );
            let df = session_ctx.sql(&sql).await.map_err(|e| {
                // SAP-1: reuse existing catalog event type `pipe.sql_planning_error` (BC-2.16.002 row 179).
                tracing::error!(
                    error = %e,
                    pipe_sql = %sql,
                    event_type = "pipe.sql_planning_error",
                    "sqlpipe-to-SQL DataFusion planning error"
                );
                PrismError::QueryExecutionFailed {
                    detail: "sqlpipe SQL planning error: <redacted; see server logs>".to_string(),
                }
            })?;
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
            // Populate available_tables from the registry for actionable diagnostics (AC-021).
            let available_tables: Vec<String> = adapter_registry
                .registered_sensor_ids()
                .into_iter()
                .map(|s| s.to_string())
                .collect();
            // No valid sensor prefix — cannot compute did_you_mean (nothing to compare to).
            return Err(PrismError::UnknownSourceTable(Box::new(
                UnknownSourceTableDetails::new(source_name.to_string(), available_tables, None),
            )));
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
            // Populate available_tables and did_you_mean for actionable diagnostics (AC-021).
            let registered: Vec<String> = adapter_registry
                .registered_sensor_ids()
                .into_iter()
                .map(|s| s.to_string())
                .collect();
            let sensor_str: &str = sensor_id.as_ref();
            // Levenshtein ≤ 3 suggestion — matches E-QUERY-037 / E-QUERY-038 threshold (D-1163).
            // CWE-407 sweep: cap `sensor_str` at 128 bytes before the O(m×n) computation.
            // `sensor_str` is derived from the table name in the query AST (untrusted input).
            let sensor_str_capped = crate::table_registry::cap_name_for_levenshtein(sensor_str);
            let did_you_mean = registered
                .iter()
                .map(|candidate| {
                    (
                        candidate.as_str(),
                        strsim::levenshtein(sensor_str_capped, candidate.as_str()),
                    )
                })
                .filter(|(_, dist)| *dist <= 3)
                .min_by(|a, b| a.1.cmp(&b.1).then_with(|| a.0.cmp(b.0)))
                .map(|(candidate, _)| candidate.to_string());
            return Err(PrismError::UnknownSourceTable(Box::new(
                UnknownSourceTableDetails::new(source_name.to_string(), registered, did_you_mean),
            )));
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

    // Only SELECT queries (and the head of a SqlPipe) have a WHERE clause with
    // time predicates. BC-2.11.020 / HIGH-1 sibling sweep: the SqlPipe head is a
    // full SQL SELECT — its WHERE clause must be propagated to the adapter as
    // ADR-033 T1 time-window bounds, exactly like a bare Ast::Sql(Select) query.
    // Without this arm, a SqlPipe query like
    //   `SELECT * FROM crowdstrike.detections WHERE timestamp > NOW() - INTERVAL '24h' | enrich ...`
    // would not push the time-window to the adapter, causing a full-table scan
    // against the 200MB/query memory budget (ADR-033 over-fetch risk).
    let Some(pred) = (match ast {
        Ast::Sql(SqlStatement::Select(sql)) => sql.where_.as_ref(),
        Ast::SqlPipe(spq) => spq.head.where_.as_ref(),
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

    // BC-2.11.020 / HIGH-1 sibling sweep: the SqlPipe head carries the WHERE clause
    // that drives push-down equality filters. Without this arm, equality predicates
    // in a SqlPipe head's WHERE (e.g. `WHERE severity = 'HIGH'`) would not be pushed
    // to the sensor adapter, causing a full-table scan + post-materialization filter.
    let where_pred = match ast {
        Ast::Sql(SqlStatement::Select(sql)) => sql.where_.as_ref(),
        Ast::SqlPipe(spq) => spq.head.where_.as_ref(),
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
        // BC-2.11.020: SqlPipe mode — extract source table from the head SQL's FROM clause.
        // The head SELECT drives the fan-out; pipe stages operate on the fetched rows.
        // OBS-1 parity fix: also collect PipeStage::Join sources from spq.stages so
        // that `SELECT … | join <table> on …` pipe stages reach the E-QUERY-011
        // AuditRead gate and E-QUERY-037 availability gate. Mirrors Ast::Pipe arm above.
        Ast::SqlPipe(spq) => {
            names.push(spq.head.from.source.raw.clone());
            for join in &spq.head.joins {
                names.push(join.source.raw.clone());
            }
            for stage in &spq.stages {
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
        // BC-2.11.020 / HIGH-1 sibling sweep: SqlPipe head drives the E-QUERY-011
        // AuditRead capability gate. `check_internal_table_capabilities` calls
        // `extract_source_names_recursive` — without this arm, a SqlPipe query with
        // head `SELECT * FROM prism_audit …` would bypass the AuditRead gate entirely.
        // Walk `walk_sql_query` on the head so that prism_* references in JOINs and
        // WHERE subqueries are also collected (mirrors `Ast::Sql(Select)` arm above).
        // OBS-1 parity fix: also collect PipeStage::Join sources from spq.stages so
        // that `SELECT … | join <table> on …` pipe stages reach the AuditRead gate.
        // Mirrors the Ast::Pipe arm above. (TD-VSDD-060)
        Ast::SqlPipe(spq) => {
            walk_sql_query(&spq.head, &mut names);
            for stage in &spq.stages {
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
// datafusion_table_name
// ---------------------------------------------------------------------------

/// Normalize a PrismQL source ref to a valid DataFusion table name.
///
/// DataFusion's `register_table` treats dots in table names as catalog/schema
/// separators and rejects names like `"crowdstrike.detections"`. Filter-mode
/// source refs use dot notation (BC-2.11.023 AC-011); SQL-mode queries use
/// underscore notation by convention. This function replaces the first dot with
/// an underscore so both forms resolve to the same registered MemTable name.
///
/// Examples:
/// - `"crowdstrike.detections"` → `"crowdstrike_detections"`
/// - `"crowdstrike_detections"` → `"crowdstrike_detections"` (no-op)
/// - `"prism_audit"` → `"prism_audit"` (no-op; internal table prefix preserved)
fn datafusion_table_name(source_name: &str) -> String {
    match source_name.find('.') {
        Some(pos) => {
            let mut s = source_name.to_string();
            s.replace_range(pos..=pos, "_");
            s
        }
        None => source_name.to_string(),
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
// check_temporal_literals — ADR-052 §D4 Option-A AST-walker
// ---------------------------------------------------------------------------

/// Plan-time four-way dispatch for `Literal::RawTemporalLiteral` nodes (ADR-052 §D4 Option A).
///
/// Called inside `run_materialization_pipeline` after `inject_now` and before fan-out
/// (full mode, `skip_projection = false`), and as an early gate in `engine::execute` before
/// `check_table_availability` (predicate-only mode, `skip_projection = true`).
///
/// Walks the parsed AST for every `RawTemporalLiteral` node (produced by the lenient parser
/// for date-like strings that are NOT valid RFC-3339). For each node found in a comparison
/// context, resolves the compared column's type via the `TableRegistry` and dispatches:
///
/// - vs `ColumnType::Datetime`              → `Err(TemporalLiteralUnparseable)` [E-QUERY-041]
/// - vs `ColumnType::String`                → COERCE in-place to `Literal::String`
/// - vs `ColumnType::Integer/Float/Boolean` → `Err(QueryTypeMismatch)`           [E-QUERY-002]
/// - In non-comparison position (SELECT / GROUP BY / ORDER BY, no column context) →
///   COERCE in-place to `Literal::String` (ADR-052 §D4 v1.8 OBS-2)
///   (only when `skip_projection = false`; skipped when `true` to let E-QUERY-037 win)
/// - Unknown/unresolvable column type       → fail-open (skip, DataFusion handles)
///
/// ## `skip_projection` flag
///
/// When `skip_projection = true` (early gate in `engine::execute`):
/// - SELECT items, GROUP BY, and ORDER BY expression checks are **not** run.
/// - Only WHERE predicates, HAVING predicates, and JOIN ON expressions are checked.
/// - This preserves the canonical gate ordering (BC-2.11.019): for an unregistered table,
///   `check_table_availability` (E-QUERY-037) wins over projection-position coercion.
/// - EC-013 is still enforced: dotted external-source WHERE predicates with Datetime columns
///   fire E-QUERY-041 before E-QUERY-037 (the predicate check still runs in this mode).
///
/// When `skip_projection = false` (in-pipeline call after `check_table_availability` passes):
/// - Full walk — SELECT items, GROUP BY, and ORDER BY are also checked.
/// - Bare `RawTemporalLiteral` in non-comparison position is COERCED to `Literal::String`.
///
/// When `registry` is `None`, the column-type dispatch arm fails-open (same as
/// E-QUERY-037/038 legacy mode). Bare `RawTemporalLiteral` in non-comparison position
/// is still coerced to `Literal::String` regardless of registry state (OBS-2).
/// `RawTemporalLiteral` in comparisons with a non-Field LHS still returns
/// `Err(QueryPlanFailed)` regardless of registry state when `skip_projection = false`.
///
/// Traces to: ADR-052 §D4 v1.8 Steps 3–5 OBS-2; BC-2.11.021 v1.6 §Postconditions;
/// S-PRISMQL-NATIVE-TEMPORAL-TYPING-001 Task 14; FIX-2 (early gate scoping).
pub(crate) fn check_temporal_literals(
    ast: &mut crate::ast::Ast,
    registry: Option<&crate::table_registry::TableRegistry>,
    skip_projection: bool,
) -> Result<(), PrismError> {
    use crate::ast::{Ast, PipeStage, SqlStatement};

    let primary_table = primary_table_from_ast(ast);

    // `Ast` and `SqlStatement` are both `#[non_exhaustive]` — within this crate all current
    // variants are exhaustively matched; the `_ => {}` arm is intentional future-proofing.
    #[allow(unreachable_patterns)]
    match ast {
        Ast::Sql(SqlStatement::Select(q)) => {
            // Walk WHERE predicate.
            if let Some(pred) = &mut q.where_ {
                check_pred_raw_temporal(pred, primary_table.as_deref(), registry)?;
            }
            // Walk HAVING predicate.
            if let Some(pred) = &mut q.having {
                check_pred_raw_temporal(pred, primary_table.as_deref(), registry)?;
            }
            // Walk JOIN ON expressions (Expr trees, not Predicates).
            for join in &mut q.joins {
                check_expr_temporal(&mut join.on, primary_table.as_deref(), registry)?;
            }
            if !skip_projection {
                // Walk SELECT items for projection-position RawTemporalLiteral.
                // Skipped in early-gate (skip_projection=true) so that E-QUERY-037
                // (table availability) wins over projection-position E-QUERY-002
                // for unregistered tables (BC-2.11.019 gate ordering, FIX-2).
                check_select_items_raw_temporal(
                    &mut q.select.items,
                    primary_table.as_deref(),
                    registry,
                )?;
                // MED-1 fix: walk GROUP BY and ORDER BY expressions.
                // ADR-052 §D4 v1.10: GROUP BY / ORDER BY use position-aware rejection.
                for expr in &mut q.group_by {
                    check_expr_temporal_pos(
                        expr,
                        primary_table.as_deref(),
                        registry,
                        TemporalCheckPos::GroupBy,
                    )?;
                }
                for order_expr in &mut q.order_by {
                    check_expr_temporal_pos(
                        &mut order_expr.expr,
                        primary_table.as_deref(),
                        registry,
                        TemporalCheckPos::OrderBy,
                    )?;
                }
            }
        }
        Ast::Pipe(pq) => {
            // Pipe mode has no SELECT items — skip_projection has no effect here.
            for stage in &mut pq.stages {
                if let PipeStage::Where(pred) = stage {
                    check_pred_raw_temporal(pred, primary_table.as_deref(), registry)?;
                }
            }
        }
        Ast::Filter(f) => {
            // Filter mode has no SELECT items — skip_projection has no effect here.
            check_pred_raw_temporal(&mut f.predicate, primary_table.as_deref(), registry)?;
        }
        Ast::SqlPipe(spq) => {
            if let Some(pred) = &mut spq.head.where_ {
                check_pred_raw_temporal(pred, primary_table.as_deref(), registry)?;
            }
            if let Some(pred) = &mut spq.head.having {
                check_pred_raw_temporal(pred, primary_table.as_deref(), registry)?;
            }
            // Walk JOIN ON expressions.
            for join in &mut spq.head.joins {
                check_expr_temporal(&mut join.on, primary_table.as_deref(), registry)?;
            }
            if !skip_projection {
                // Skipped in early-gate for the same reason as Sql(Select) above.
                check_select_items_raw_temporal(
                    &mut spq.head.select.items,
                    primary_table.as_deref(),
                    registry,
                )?;
                // MED-1 fix: walk GROUP BY and ORDER BY expressions.
                // ADR-052 §D4 v1.10: GROUP BY / ORDER BY use position-aware rejection.
                for expr in &mut spq.head.group_by {
                    check_expr_temporal_pos(
                        expr,
                        primary_table.as_deref(),
                        registry,
                        TemporalCheckPos::GroupBy,
                    )?;
                }
                for order_expr in &mut spq.head.order_by {
                    check_expr_temporal_pos(
                        &mut order_expr.expr,
                        primary_table.as_deref(),
                        registry,
                        TemporalCheckPos::OrderBy,
                    )?;
                }
            }
            for stage in &mut spq.stages {
                if let PipeStage::Where(pred) = stage {
                    check_pred_raw_temporal(pred, primary_table.as_deref(), registry)?;
                }
            }
        }
        // F-P4-LOW-1 fix: walk DML nodes so a future S-3.06 write-path wiring doesn't
        // silently pass unvalidated RawTemporalLiteral to DataFusion.
        // Currently DML falls back to Ok(vec![]) at materialization.rs execute path —
        // this walk is defense-in-depth so the temporal gate runs if DML is ever wired.
        Ast::Sql(SqlStatement::Dml(dml)) => {
            // Thread target_table as primary_table so unqualified column references in the
            // WHERE predicate (e.g., `WHERE timestamp > '2026-06-24'`) resolve correctly.
            let dml_table = dml.target_table.clone();
            // WHERE predicate (UPDATE / DELETE).
            if let Some(filter) = &mut dml.filter {
                check_pred_raw_temporal(filter, Some(dml_table.as_str()), registry)?;
            }
            // SET assignments (UPDATE) — value is Expr, can contain RawTemporalLiteral.
            // Apply four-way dispatch using the assignment's target column name, mirroring
            // the behavior of check_pred_raw_temporal::Compare for field comparisons
            // (ADR-052 §D4 v1.8: Datetime→E-QUERY-041; String→coerce; numeric/bool→E-QUERY-002;
            // unknown→coerce to Literal::String per OBS-2).
            for assignment in &mut dml.assignments {
                use crate::ast::{Expr, Literal};
                use prism_core::column::ColumnType;

                if let Expr::Literal(Literal::RawTemporalLiteral(ref raw_val)) = assignment.value {
                    let raw_val = raw_val.clone();
                    let col_type = registry.and_then(|r| {
                        r.column_type_for(dml_table.as_str(), assignment.column.as_str())
                    });
                    match col_type {
                        Some(ColumnType::Datetime) => {
                            let value_prefix: String = raw_val.chars().take(50).collect();
                            return Err(PrismError::TemporalLiteralUnparseable { value_prefix });
                        }
                        Some(ColumnType::String) => {
                            // Coerce in-place — no recursion needed (Literal::String is terminal;
                            // check_expr_temporal on it would be a no-op via the `_ => Ok(())` arm).
                            assignment.value = Expr::Literal(Literal::String(raw_val.clone()));
                        }
                        Some(ct @ ColumnType::Integer)
                        | Some(ct @ ColumnType::Float)
                        | Some(ct @ ColumnType::Boolean) => {
                            return Err(PrismError::QueryTypeMismatch {
                                column: assignment.column.clone(),
                                table: dml_table.clone(),
                                actual_type: ct,
                                operator: "=".to_string(),
                            });
                        }
                        None | Some(_) => {
                            // Unknown/unresolvable column type (None: registry absent or column
                            // not found; Some(_): Json or a future type variant) — coerce in-place
                            // to Literal::String. ADR-052 §D4 v1.8 OBS-2: mirrors
                            // check_expr_temporal's bare-RawTemporalLiteral arm, which COERCES to
                            // Literal::String and returns Ok(()). The previous comment
                            // ("do NOT recurse: bare arm would Err unconditionally") was stale
                            // after OBS-2 landed. DML execution returns Ok(vec![]) pending S-3.06
                            // wiring; this coercion is defense-in-depth so the literal is clean if
                            // DML execution is wired later.
                            assignment.value = Expr::Literal(Literal::String(raw_val.clone()));
                        }
                    }
                } else {
                    // Non-RawTemporalLiteral top-level expression — recurse for nested literals.
                    check_expr_temporal(&mut assignment.value, Some(dml_table.as_str()), registry)?;
                }
            }
            // Source SELECT for INSERT INTO … SELECT ….
            if let Some(src_q) = &mut dml.source_select {
                let sub_primary = normalized_table_name_for_source(&src_q.from.source);
                if let Some(pred) = &mut src_q.where_ {
                    check_pred_raw_temporal(pred, sub_primary.as_deref(), registry)?;
                }
                if let Some(pred) = &mut src_q.having {
                    check_pred_raw_temporal(pred, sub_primary.as_deref(), registry)?;
                }
                for join in &mut src_q.joins {
                    check_expr_temporal(&mut join.on, sub_primary.as_deref(), registry)?;
                }
                check_select_items_raw_temporal(
                    &mut src_q.select.items,
                    sub_primary.as_deref(),
                    registry,
                )?;
                // ADR-052 §D4 v1.10: DML source_select GROUP BY / ORDER BY use position-aware rejection.
                for expr in &mut src_q.group_by {
                    check_expr_temporal_pos(
                        expr,
                        sub_primary.as_deref(),
                        registry,
                        TemporalCheckPos::GroupBy,
                    )?;
                }
                for order_expr in &mut src_q.order_by {
                    check_expr_temporal_pos(
                        &mut order_expr.expr,
                        sub_primary.as_deref(),
                        registry,
                        TemporalCheckPos::OrderBy,
                    )?;
                }
            }
        }
        _ => {} // non_exhaustive: future AST variants pass through
    }
    Ok(())
}

/// Normalize a `SourceRef` to its registered table name.
///
/// Mirrors the normalization in `primary_table_from_ast`:
/// - `SourceRefKind::Custom` → `source.raw` (already the table name)
/// - `SourceRefKind::External { sensor, table }` → `"{sensor}_{table}"` (dot-notation
///   normalization, e.g., `crowdstrike.detections` → `crowdstrike_detections`)
/// - `Composite` / `Internal` → `None` (fail-open; no schema lookup for aggregates)
///
/// HIGH-1 fix: used by InSubquery and Expr::InSubquery walkers so that External-source
/// subqueries resolve columns correctly against the `TableRegistry`.
fn normalized_table_name_for_source(source: &crate::ast::SourceRef) -> Option<String> {
    use crate::ast::SourceRefKind;
    match &source.kind {
        SourceRefKind::Custom => Some(source.raw.clone()),
        SourceRefKind::External { sensor, table } => Some(format!("{sensor}_{table}")),
        _ => None, // Composite/Internal — fail-open
    }
}

/// Extract the registered table name from the primary source of an AST.
///
/// Returns `None` for composite/internal sources or AST variants without a single
/// primary table. Primary-table-only scope per ADR-052 §D4: JOINs and subqueries
/// fail open (unresolved column type → DataFusion handles).
fn primary_table_from_ast(ast: &crate::ast::Ast) -> Option<String> {
    use crate::ast::{Ast, SqlStatement};
    let source = match ast {
        Ast::Sql(SqlStatement::Select(q)) => &q.from.source,
        Ast::Pipe(pq) => &pq.source,
        Ast::Filter(f) => &f.source,
        Ast::SqlPipe(spq) => &spq.head.from.source,
        _ => return None,
    };
    normalized_table_name_for_source(source)
}

/// Resolve the `ColumnType` for a `FieldPath` against the `TableRegistry`.
///
/// For qualified 2-segment paths (`table_name.col_name`): uses `segments[0]` as the
/// table name and `segments[last]` as the column name.
/// For 3-segment paths (`sensor.table.column`): the External source dotted-notation pattern
/// (e.g., `crowdstrike.detections.timestamp`) is normalized to `sensor_table` composite key
/// (e.g., `crowdstrike_detections`) matching the `normalized_table_name_for_source` convention.
/// If the composite key is not found, returns `None` (fail-open) — no fallback to `segments[0]`
/// is attempted, to prevent over-resolution of nested struct paths (ADR-052 §D4 OBS-P7-2).
/// For unqualified 1-segment paths (`col_name`): uses `primary_table` as the table name.
/// Falls open (`None`) when the table or column is not found.
fn resolve_col_type(
    fp: &crate::ast::FieldPath,
    primary_table: Option<&str>,
    registry: &crate::table_registry::TableRegistry,
) -> Option<prism_core::column::ColumnType> {
    let col_name = fp.segments.last()?.as_str();

    if fp.segments.len() >= 3 {
        // Three-segment: `sensor.table.column` pattern (External source dotted notation).
        // Normalized table name follows the `sensor_table` convention from
        // `normalized_table_name_for_source` (e.g., `crowdstrike.detections` → `crowdstrike_detections`).
        // No fallback to segments[0] — prevents over-resolution of nested struct paths where a
        // sensor name coincidentally matches a bare table name in the registry (OBS-P7-2 fix).
        let composite = format!("{}_{}", fp.segments[0], fp.segments[1]);
        registry.column_type_for(composite.as_str(), col_name)
    } else if fp.segments.len() == 2 {
        // Qualified: `ghost_sensor_devices.timestamp` → table = "ghost_sensor_devices", col = "timestamp"
        registry.column_type_for(fp.segments[0].as_str(), col_name)
    } else if fp.segments.len() == 1 {
        // Unqualified: use primary_table from AST source
        registry.column_type_for(primary_table?, col_name)
    } else {
        None
    }
}

/// Convert a `CompareOp` to its canonical query-language string representation.
///
/// P3-MED-1 fix: used to populate `PrismError::QueryTypeMismatch { operator }` with the
/// actual operator from the query rather than the hardcoded `"="` sentinel.
///
/// The `#[allow(unreachable_patterns)]` is intentional: `CompareOp` is `#[non_exhaustive]`,
/// so the `_ =>` arm is currently unreachable within this crate but required to handle future
/// variants without a breaking change. Same pattern as `ast.rs normalize_expr`.
#[allow(unreachable_patterns)]
fn compare_op_to_str(op: &crate::ast::CompareOp) -> &'static str {
    use crate::ast::CompareOp;
    match op {
        CompareOp::Eq => "=",
        CompareOp::Ne => "!=",
        CompareOp::Gt => ">",
        CompareOp::Lt => "<",
        CompareOp::Ge => ">=",
        CompareOp::Le => "<=",
        CompareOp::Like => "LIKE",
        CompareOp::Cidr => "IN CIDR",
        CompareOp::NotCidr => "NOT IN CIDR",
        // #[non_exhaustive] — future variants fall through to a generic label.
        _ => "op",
    }
}

/// Apply four-way temporal dispatch to a `Literal` in a filter position where the
/// column field path is known.
///
/// Used by `check_pred_raw_temporal` for `Between.low`/`Between.high` and `In.values`
/// positions (which hold `Literal` directly, not `Box<Expr>`).
///
/// `operator_label` is the operator as it appears in the query (e.g., `"BETWEEN"`, `"IN"`,
/// `"="`) — used to populate `PrismError::QueryTypeMismatch { operator }` with accurate
/// context for the analyst (P3-MED-1 fix).
///
/// Four-way dispatch (ADR-052 §D4):
/// - `ColumnType::Datetime`  → `Err(TemporalLiteralUnparseable)` (E-QUERY-041)
/// - `ColumnType::String`    → coerce `*lit` in-place to `Literal::String`
/// - `Integer/Float/Boolean` → `Err(QueryTypeMismatch)` (E-QUERY-002)
/// - unknown / fail-open     → `Ok(())` (DataFusion remains the correctness gate)
fn apply_literal_dispatch(
    lit: &mut crate::ast::Literal,
    field: &crate::ast::FieldPath,
    primary_table: Option<&str>,
    registry: Option<&crate::table_registry::TableRegistry>,
    operator_label: &str,
) -> Result<(), PrismError> {
    use crate::ast::Literal;
    use prism_core::column::ColumnType;

    let raw_val = if let Literal::RawTemporalLiteral(s) = lit {
        s.clone()
    } else {
        return Ok(()); // Nothing to dispatch — literal is not a RawTemporalLiteral.
    };

    let col_type = registry.and_then(|r| resolve_col_type(field, primary_table, r));

    match col_type {
        Some(ColumnType::Datetime) => {
            let value_prefix: String = raw_val.chars().take(50).collect();
            Err(PrismError::TemporalLiteralUnparseable { value_prefix })
        }
        Some(ColumnType::String) => {
            // COERCE in-place: RawTemporalLiteral → Literal::String.
            *lit = Literal::String(raw_val);
            Ok(())
        }
        Some(ct @ ColumnType::Integer)
        | Some(ct @ ColumnType::Float)
        | Some(ct @ ColumnType::Boolean) => {
            let col_name = field.segments.last().cloned().unwrap_or_default();
            let table_name = if field.segments.len() >= 2 {
                field.segments[0].clone()
            } else {
                primary_table.unwrap_or("unknown").to_string()
            };
            Err(PrismError::QueryTypeMismatch {
                column: col_name,
                table: table_name,
                actual_type: ct,
                operator: operator_label.to_string(),
            })
        }
        None | Some(_) => Ok(()), // Unknown column, Json, or other type → fail-open.
    }
}

/// Recursively walk a `Predicate` tree and apply four-way dispatch to every
/// `RawTemporalLiteral` found in filter-position comparisons.
///
/// Handles:
/// - `Predicate::Compare { lhs: Field(fp), rhs: Literal(RawTemporalLiteral) }` — dispatch
/// - `Predicate::Between { field, low, high }` — dispatch to both `low` and `high`
/// - `Predicate::In { field, values }` — dispatch to each value
/// - `Predicate::InSubquery { subquery }` — recurse into subquery WHERE and HAVING
/// - `Predicate::Logical` / `Predicate::Not` — recurse
///
/// Mutates literals in-place for the String-column coercion arm.
fn check_pred_raw_temporal(
    pred: &mut crate::ast::Predicate,
    primary_table: Option<&str>,
    registry: Option<&crate::table_registry::TableRegistry>,
) -> Result<(), PrismError> {
    use crate::ast::{Expr, Literal, Predicate};
    use prism_core::column::ColumnType;
    use prism_core::error::TemporalLiteralPosition;

    match pred {
        // P3-MED-1 fix: capture `op` (not `..`) to populate QueryTypeMismatch { operator }
        // with the actual comparison operator rather than the hardcoded "=" sentinel.
        // P3-LOW-1 fix: recurse into lhs and (conditionally) rhs after dispatch to catch
        // nested temporal literals in subexpressions; mirrors check_expr_temporal::Compare.
        Predicate::Compare { lhs, rhs, op } => {
            // Only gate when rhs is RawTemporalLiteral — extract value early.
            let rhs_is_top_level_raw_temporal =
                matches!(rhs.as_ref(), Expr::Literal(Literal::RawTemporalLiteral(_)));
            let raw_val = if let Expr::Literal(Literal::RawTemporalLiteral(s)) = rhs.as_ref() {
                Some(s.clone())
            } else {
                None
            };
            let op_str = compare_op_to_str(op);

            if let Some(raw_val) = raw_val {
                if let Expr::Field(fp) = lhs.as_ref() {
                    // Dispatch when lhs is a FieldPath (the column being compared).
                    let fp_clone = fp.clone();
                    let col_type =
                        registry.and_then(|r| resolve_col_type(&fp_clone, primary_table, r));

                    match col_type {
                        Some(ColumnType::Datetime) => {
                            // E-QUERY-041: RawTemporalLiteral vs Datetime column.
                            let value_prefix: String = raw_val.chars().take(50).collect();
                            return Err(PrismError::TemporalLiteralUnparseable { value_prefix });
                        }
                        Some(ColumnType::String) => {
                            // COERCE in-place: RawTemporalLiteral → Literal::String.
                            // Emitted SQL will be byte-identical to pre-ADR-052 behavior.
                            **rhs = Expr::Literal(Literal::String(raw_val));
                        }
                        Some(ct @ ColumnType::Integer)
                        | Some(ct @ ColumnType::Float)
                        | Some(ct @ ColumnType::Boolean) => {
                            // E-QUERY-002: type mismatch — date-like string vs numeric/bool column.
                            let col_name = fp_clone.segments.last().cloned().unwrap_or_default();
                            let table_name = if fp_clone.segments.len() >= 2 {
                                fp_clone.segments[0].clone()
                            } else {
                                primary_table.unwrap_or("unknown").to_string()
                            };
                            return Err(PrismError::QueryTypeMismatch {
                                column: col_name,
                                table: table_name,
                                actual_type: ct,
                                operator: op_str.to_string(),
                            });
                        }
                        None | Some(_) => {
                            // Unknown column, Json, or other type → fail-open (ADR-052 §D4).
                            // The RawTemporalLiteral remains in the AST.
                            // Pipe/Filter mode: caught by `pipe_sql_emitter::literal_to_sql`
                            //   belt-and-suspenders guard → E-QUERY-002 QueryPlanFailed.
                            // SQL mode: RawTemporalLiteral is normalized to a plain quoted string
                            //   by PqlNormalizer → DataFusion is the tertiary correctness gate.
                        }
                    }
                } else {
                    // E-QUERY-042 (NonColumnLhsComparison): non-Field LHS in Predicate path
                    // (e.g., `MAX(timestamp) > '2026-06-24'` in a HAVING clause). No column
                    // context for four-way dispatch; silently coercing would reintroduce
                    // RISK-1 for datetime-valued expressions like `to_timestamp(col)`.
                    //
                    // ADR-052 §D4 v1.10 arm (4). Mirrors check_expr_temporal_pos::Expr::Compare
                    // else-branch (same error, both Predicate and Expr paths now return
                    // E-QUERY-042 / -32602 INVALID_PARAMS).
                    // Replaces prior `QueryPlanFailed → -32000 INTERNAL_ERROR` (analyst-hostile).
                    let value_prefix: String = raw_val.chars().take(50).collect();
                    return Err(PrismError::TemporalLiteralInvalidPosition {
                        position: TemporalLiteralPosition::NonColumnLhsComparison,
                        value_prefix,
                    });
                }
            }
            // P3-LOW-1 fix: recurse into lhs and, if rhs wasn't a top-level
            // RawTemporalLiteral (already dispatched above), also into rhs.
            check_expr_temporal(lhs, primary_table, registry)?;
            if !rhs_is_top_level_raw_temporal {
                check_expr_temporal(rhs, primary_table, registry)?;
            }
            Ok(())
        }

        // CRIT-1 fix: Between — apply dispatch to both `low` and `high` (F-LP3-CRIT-1-BETWEEN).
        // P3-MED-1 fix: pass "BETWEEN" as operator_label for accurate E-QUERY-002 messages.
        Predicate::Between {
            field, low, high, ..
        } => {
            apply_literal_dispatch(low, field, primary_table, registry, "BETWEEN")?;
            apply_literal_dispatch(high, field, primary_table, registry, "BETWEEN")?;
            Ok(())
        }

        // CRIT-2 fix: In — apply dispatch to each value (F-LP3-CRIT-1-IN).
        // P3-MED-1 fix: pass "IN" as operator_label for accurate E-QUERY-002 messages.
        Predicate::In { field, values, .. } => {
            for val in values.iter_mut() {
                apply_literal_dispatch(val, field, primary_table, registry, "IN")?;
            }
            Ok(())
        }

        // HIGH-2 fix: InSubquery — recurse into the subquery's WHERE and HAVING.
        // HIGH-1 fix: normalize the subquery's FROM source via normalized_table_name_for_source
        // to ensure External dot-notation sources (e.g., crowdstrike.detections →
        // crowdstrike_detections) resolve correctly against the TableRegistry.
        // MED-1 fix (subquery internals): also walk subquery GROUP BY and ORDER BY Expr trees.
        // F-P4-MED-2 fix: also walk subquery JOIN ON expressions and SELECT items.
        Predicate::InSubquery { subquery, .. } => {
            let sub_primary = normalized_table_name_for_source(&subquery.from.source);
            if let Some(where_pred) = &mut subquery.where_ {
                check_pred_raw_temporal(where_pred, sub_primary.as_deref(), registry)?;
            }
            if let Some(having_pred) = &mut subquery.having {
                check_pred_raw_temporal(having_pred, sub_primary.as_deref(), registry)?;
            }
            for join in &mut subquery.joins {
                check_expr_temporal(&mut join.on, sub_primary.as_deref(), registry)?;
            }
            check_select_items_raw_temporal(
                &mut subquery.select.items,
                sub_primary.as_deref(),
                registry,
            )?;
            // ADR-052 §D4 v1.10: subquery GROUP BY / ORDER BY use position-aware rejection.
            for expr in &mut subquery.group_by {
                check_expr_temporal_pos(
                    expr,
                    sub_primary.as_deref(),
                    registry,
                    TemporalCheckPos::GroupBy,
                )?;
            }
            for order_expr in &mut subquery.order_by {
                check_expr_temporal_pos(
                    &mut order_expr.expr,
                    sub_primary.as_deref(),
                    registry,
                    TemporalCheckPos::OrderBy,
                )?;
            }
            Ok(())
        }

        Predicate::Logical { predicates, .. } => {
            // Recurse into AND/OR children.
            for p in predicates.iter_mut() {
                check_pred_raw_temporal(p, primary_table, registry)?;
            }
            Ok(())
        }

        Predicate::Not(inner) => check_pred_raw_temporal(inner.as_mut(), primary_table, registry),

        // StringOp, Regex, Cidr, Has, Missing, IsNull, Wildcard, RecoveryError —
        // none contain RawTemporalLiteral in a position requiring dispatch.
        _ => Ok(()),
    }
}

/// Internal position context for temporal literal checks.
///
/// Used by `check_expr_temporal_pos` to determine behavior for bare
/// `RawTemporalLiteral` nodes outside comparison context.
///
/// ADR-052 §D4 v1.10:
/// - GROUP BY and ORDER BY positions REJECT with E-QUERY-042 (tightening OBS-2).
/// - All other non-comparison positions COERCE to `Literal::String` (OBS-2 preserved).
///
/// Compare arms (Field LHS) are not affected by position — their behavior is determined
/// by the column type resolved from the registry, not the clause position.
#[derive(Clone, Copy)]
enum TemporalCheckPos {
    /// Default: SELECT projection, JOIN ON, FuncCall args, subexpressions.
    /// Bare `RawTemporalLiteral` → COERCE to `Literal::String` (OBS-2 preserved).
    Other,
    /// GROUP BY key position.
    /// Bare `RawTemporalLiteral` → E-QUERY-042 (TemporalLiteralPosition::GroupBy).
    GroupBy,
    /// ORDER BY key position.
    /// Bare `RawTemporalLiteral` → E-QUERY-042 (TemporalLiteralPosition::OrderBy).
    OrderBy,
}

/// Thin shim: walk an `Expr` with the default `Other` position context.
///
/// All call sites that do NOT need position-specific behavior use this function.
/// GROUP BY and ORDER BY call sites use `check_expr_temporal_pos` with the
/// appropriate `TemporalCheckPos` variant.
///
/// HIGH-2 fix (F-LP3-HIGH-2-EXPR-WALKER): walk JOIN ON and SELECT Expr trees.
fn check_expr_temporal(
    expr: &mut crate::ast::Expr,
    primary_table: Option<&str>,
    registry: Option<&crate::table_registry::TableRegistry>,
) -> Result<(), PrismError> {
    check_expr_temporal_pos(expr, primary_table, registry, TemporalCheckPos::Other)
}

/// Recursively walk an `Expr` tree and apply temporal literal dispatch,
/// with awareness of the clause position for bare `RawTemporalLiteral` nodes.
///
/// Used for JOIN ON conditions (which are `Expr`, not `Predicate`) and for
/// expressions inside SELECT items that contain comparisons, and for GROUP BY /
/// ORDER BY keys (with position-specific rejection per ADR-052 §D4 v1.10).
///
/// **Seven-arm dispatch (ADR-052 §D4 v1.10):**
///
/// For `Expr::Literal(Literal::RawTemporalLiteral)` (bare, non-comparison):
/// - `TemporalCheckPos::GroupBy` → E-QUERY-042 (GroupBy)
/// - `TemporalCheckPos::OrderBy` → E-QUERY-042 (OrderBy)
/// - `TemporalCheckPos::Other` → COERCE to `Literal::String` (OBS-2 preserved for
///   SELECT projection, JOIN ON, FuncCall args, DML SET unknown-column, etc.)
///
/// For `Expr::Compare { lhs: Field(fp), rhs: Literal(RawTemporalLiteral) }`:
/// - `ColumnType::Datetime` → E-QUERY-041 (TemporalLiteralUnparseable)
/// - `ColumnType::String` → COERCE to `Literal::String`
/// - `ColumnType::Integer/Float/Boolean` → E-QUERY-002 (QueryTypeMismatch)
/// - Unknown/Json/None → fail-open (RawTemporalLiteral remains; emitter guard fires)
///
/// For `Expr::Compare` with a non-Field LHS and `RawTemporalLiteral` RHS:
/// - E-QUERY-042 (NonColumnLhsComparison) — caller must use RFC-3339 for datetime columns,
///   a non-date-shaped string for string columns, or wrap in CAST.
///   (Replaces prior `QueryPlanFailed → -32000 INTERNAL_ERROR` behavior.)
fn check_expr_temporal_pos(
    expr: &mut crate::ast::Expr,
    primary_table: Option<&str>,
    registry: Option<&crate::table_registry::TableRegistry>,
    pos: TemporalCheckPos,
) -> Result<(), PrismError> {
    use crate::ast::{Expr, Literal};
    use prism_core::column::ColumnType;
    use prism_core::error::TemporalLiteralPosition;

    match expr {
        Expr::Literal(Literal::RawTemporalLiteral(raw)) => {
            // Non-comparison position — dispatch on clause context.
            //
            // ADR-052 §D4 v1.10:
            // - GROUP BY / ORDER BY: REJECT with E-QUERY-042 (grouping/ordering by a
            //   constant is a degenerate no-op; almost always an analyst mistake).
            // - Other (SELECT projection, JOIN ON, FuncCall args, DML SET, etc.): COERCE
            //   to Literal::String (OBS-2 preserved for non-GROUP-BY/ORDER-BY positions).
            //
            // NLL: `raw` is last used in `std::mem::take(raw)` or `raw.chars()`;
            // the borrow ends there. Reassigning `*expr` is safe after that point.
            match pos {
                TemporalCheckPos::GroupBy => {
                    let value_prefix: String = raw.chars().take(50).collect();
                    Err(PrismError::TemporalLiteralInvalidPosition {
                        position: TemporalLiteralPosition::GroupBy,
                        value_prefix,
                    })
                }
                TemporalCheckPos::OrderBy => {
                    let value_prefix: String = raw.chars().take(50).collect();
                    Err(PrismError::TemporalLiteralInvalidPosition {
                        position: TemporalLiteralPosition::OrderBy,
                        value_prefix,
                    })
                }
                TemporalCheckPos::Other => {
                    // COERCE: preserve OBS-2 for non-GROUP-BY/ORDER-BY positions.
                    let s = std::mem::take(raw);
                    *expr = Expr::Literal(Literal::String(s));
                    Ok(())
                }
            }
        }
        // P3-MED-1 fix: capture `op` to populate QueryTypeMismatch { operator } with the actual
        // operator from the query (previously hardcoded "=" regardless of actual op used).
        Expr::Compare { lhs, rhs, op } => {
            // Gate when rhs is RawTemporalLiteral with a field LHS.
            // Track whether rhs was originally a top-level RawTemporalLiteral so we can
            // decide whether to recurse into rhs afterwards (LOW-1 fix: recurse when rhs
            // contains nested temporal literals; skip when already dispatched at top level).
            let rhs_is_top_level_raw_temporal =
                matches!(rhs.as_ref(), Expr::Literal(Literal::RawTemporalLiteral(_)));
            let raw_val = if let Expr::Literal(Literal::RawTemporalLiteral(s)) = rhs.as_ref() {
                Some(s.clone())
            } else {
                None
            };
            let op_str = compare_op_to_str(op);
            if let Some(raw_val) = raw_val {
                if let Expr::Field(fp) = lhs.as_ref() {
                    let fp_clone = fp.clone();
                    let col_type =
                        registry.and_then(|r| resolve_col_type(&fp_clone, primary_table, r));
                    match col_type {
                        Some(ColumnType::Datetime) => {
                            let value_prefix: String = raw_val.chars().take(50).collect();
                            return Err(PrismError::TemporalLiteralUnparseable { value_prefix });
                        }
                        Some(ColumnType::String) => {
                            **rhs = Expr::Literal(Literal::String(raw_val));
                        }
                        Some(ct @ ColumnType::Integer)
                        | Some(ct @ ColumnType::Float)
                        | Some(ct @ ColumnType::Boolean) => {
                            let col_name = fp_clone.segments.last().cloned().unwrap_or_default();
                            let table_name = if fp_clone.segments.len() >= 2 {
                                fp_clone.segments[0].clone()
                            } else {
                                primary_table.unwrap_or("unknown").to_string()
                            };
                            return Err(PrismError::QueryTypeMismatch {
                                column: col_name,
                                table: table_name,
                                actual_type: ct,
                                operator: op_str.to_string(),
                            });
                        }
                        None | Some(_) => {
                            // Unknown column, Json, or other type → fail-open (ADR-052 §D4).
                            // The RawTemporalLiteral remains in the AST.
                            // Pipe/Filter mode: caught by `pipe_sql_emitter::literal_to_sql`
                            //   belt-and-suspenders guard → E-QUERY-002 QueryPlanFailed.
                            // SQL mode: normalized to a plain quoted string by PqlNormalizer;
                            //   DataFusion is the tertiary correctness gate.
                        }
                    }
                } else {
                    // LHS is not a FieldPath — can't dispatch at plan time.
                    //
                    // E-QUERY-042 (NonColumnLhsComparison): the walker cannot resolve the LHS
                    // type (e.g., `lower(hostname)` is a FuncCall). Silently coercing would
                    // reintroduce RISK-1 for datetime-valued expressions like `to_timestamp(col)`.
                    //
                    // ADR-052 §D4 v1.10 arm (4). Replaces prior `QueryPlanFailed → -32000`
                    // (analyst-hostile) with analyst-readable `-32602 INVALID_PARAMS`.
                    let value_prefix: String = raw_val.chars().take(50).collect();
                    return Err(PrismError::TemporalLiteralInvalidPosition {
                        position: TemporalLiteralPosition::NonColumnLhsComparison,
                        value_prefix,
                    });
                }
            }
            // Recurse into lhs and, if rhs wasn't a top-level RawTemporalLiteral (already
            // dispatched above), also into rhs. This catches nested temporal literals in
            // subexpressions like `field > fn_call(expr)` (LOW-1 fix).
            // Recursive calls use Other position — sub-expressions inside Compare arms are
            // not directly in GROUP BY / ORDER BY key position.
            check_expr_temporal(lhs, primary_table, registry)?;
            if !rhs_is_top_level_raw_temporal {
                check_expr_temporal(rhs, primary_table, registry)?;
            }
            Ok(())
        }
        Expr::Logical { lhs, rhs, .. } => {
            check_expr_temporal(lhs, primary_table, registry)?;
            check_expr_temporal(rhs, primary_table, registry)
        }
        Expr::Not(inner) => check_expr_temporal(inner, primary_table, registry),
        Expr::In { field, values, .. } => {
            // For In values in Expr context (e.g., JOIN ON position).
            // P3-MED-1 fix: pass "IN" as operator_label for accurate E-QUERY-002 messages.
            for val in values.iter_mut() {
                apply_literal_dispatch(val, field, primary_table, registry, "IN")?;
            }
            Ok(())
        }
        Expr::InSubquery { subquery, .. } => {
            // Recurse into the subquery's WHERE and HAVING.
            // HIGH-1 fix: use normalized_table_name_for_source to correctly map External
            // dot-notation sources (e.g., crowdstrike.detections → crowdstrike_detections).
            // MED-1 fix (subquery internals): also walk subquery GROUP BY and ORDER BY.
            // F-P4-MED-2 fix: also walk subquery JOIN ON expressions and SELECT items.
            // ADR-052 §D4 v1.10: subquery GROUP BY/ORDER BY use position-aware checks.
            let sub_primary = normalized_table_name_for_source(&subquery.from.source);
            if let Some(where_pred) = &mut subquery.where_ {
                check_pred_raw_temporal(where_pred, sub_primary.as_deref(), registry)?;
            }
            if let Some(having_pred) = &mut subquery.having {
                check_pred_raw_temporal(having_pred, sub_primary.as_deref(), registry)?;
            }
            for join in &mut subquery.joins {
                check_expr_temporal(&mut join.on, sub_primary.as_deref(), registry)?;
            }
            check_select_items_raw_temporal(
                &mut subquery.select.items,
                sub_primary.as_deref(),
                registry,
            )?;
            for expr in &mut subquery.group_by {
                check_expr_temporal_pos(
                    expr,
                    sub_primary.as_deref(),
                    registry,
                    TemporalCheckPos::GroupBy,
                )?;
            }
            for order_expr in &mut subquery.order_by {
                check_expr_temporal_pos(
                    &mut order_expr.expr,
                    sub_primary.as_deref(),
                    registry,
                    TemporalCheckPos::OrderBy,
                )?;
            }
            Ok(())
        }
        // F-P4-MED-1 fix: FuncCall args CAN contain RawTemporalLiteral (the SQL parser's
        // recursive expr builder accepts any Literal in function argument position).
        // Recurse into args for both Aggregate and Scalar variants.
        // Window is a stub with no args field yet (S-3.06); falls to _ arm.
        Expr::FuncCall(fc) => {
            use crate::ast::FuncCall;
            match fc {
                FuncCall::Aggregate { args, .. } | FuncCall::Scalar { args, .. } => {
                    for arg in args.iter_mut() {
                        check_expr_temporal(arg, primary_table, registry)?;
                    }
                    Ok(())
                }
                // Window stub (S-3.06) has no args field — fall-through is safe.
                _ => Ok(()),
            }
        }
        // F-P4-OBS-3: TimestampArithmetic base is currently always Expr::Now (grammar
        // restriction in filter_parser.rs::build_temporal_rhs_parser). Recurse defensively
        // in case a future grammar extension permits non-Now bases.
        Expr::TimestampArithmetic { base, .. } => {
            check_expr_temporal(base, primary_table, registry)
        }
        // Field, VirtualField, Now, Interval, Star, Literal(non-Raw), etc.
        // — do not contain nested RawTemporalLiteral positions.
        _ => Ok(()),
    }
}

/// Walk SELECT items and apply temporal literal dispatch.
///
/// For `SelectItem::Expr { expr, .. }`: calls `check_expr_temporal` which handles
/// direct `RawTemporalLiteral` as well as `Expr::Compare`, `Expr::Logical`, `Expr::Not`,
/// `Expr::In`, `Expr::InSubquery` positions.
///
/// Belt-and-suspenders guard ensuring `RawTemporalLiteral` never reaches the SQL emitter
/// in projection position (ADR-052 §D4 Step 3 last row). HIGH-2 fix extends coverage
/// to all expression positions in SELECT items.
fn check_select_items_raw_temporal(
    items: &mut [crate::ast::SelectItem],
    primary_table: Option<&str>,
    registry: Option<&crate::table_registry::TableRegistry>,
) -> Result<(), PrismError> {
    use crate::ast::SelectItem;

    for item in items.iter_mut() {
        if let SelectItem::Expr { expr, .. } = item {
            check_expr_temporal(expr, primary_table, registry)?;
        }
    }
    Ok(())
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

    /// OBS-1: SqlPipe pipe-stage JOIN sources must be collected by both Layer 1 walkers.
    ///
    /// Represents queries like:
    ///   `SELECT * FROM crowdstrike_detections | join prism_audit on id == trace_id`
    ///
    /// Prior to the OBS-1 fix, `extract_source_names_recursive` and
    /// `extract_source_names_shallow` only collected the SqlPipe head sources
    /// (`crowdstrike_detections`) and silently skipped `PipeStage::Join` sources
    /// (`prism_audit`) from `spq.stages`. The AuditRead gate (E-QUERY-011) and
    /// availability gate (E-QUERY-037) therefore never saw the join source.
    ///
    /// This is a defensive parity fix — `Ast::SqlPipe` must mirror `Ast::Pipe`
    /// for Join source collection. (TD-VSDD-060)
    #[test]
    #[allow(non_snake_case)]
    fn test_OBS_1_sql_pipe_join_stage_source_discovered_by_layer1() {
        use super::extract_source_names_shallow;
        use crate::ast::{
            InternalTable, JoinCondition, JoinKind, JoinStage, PipeStage, SqlPipeQuery,
        };

        // Build: SELECT * FROM crowdstrike_detections | join prism_audit on id == trace_id
        let join_stage = JoinStage {
            kind: JoinKind::Inner,
            source: SourceRef {
                raw: "prism_audit".to_string(),
                kind: SourceRefKind::Internal(InternalTable::Audit),
            },
            on: JoinCondition::Pair(
                FieldPath {
                    segments: vec!["id".to_string()],
                    span: Span::ZERO,
                },
                FieldPath {
                    segments: vec!["trace_id".to_string()],
                    span: Span::ZERO,
                },
            ),
        };

        let sql_pipe_ast = Ast::SqlPipe(SqlPipeQuery {
            head: minimal_select("crowdstrike_detections"),
            stages: vec![PipeStage::Join(join_stage)],
        });

        // extract_source_names_recursive must discover both the head source and the join source.
        let recursive_names = extract_source_names_recursive(&sql_pipe_ast);
        assert!(
            recursive_names
                .iter()
                .any(|n| n == "crowdstrike_detections"),
            "OBS-1: extract_source_names_recursive must include 'crowdstrike_detections' \
             (SqlPipe head source); got names: {recursive_names:?}"
        );
        assert!(
            recursive_names.iter().any(|n| n == "prism_audit"),
            "OBS-1: extract_source_names_recursive must discover 'prism_audit' \
             from SqlPipe PipeStage::Join source; got names: {recursive_names:?}"
        );

        // extract_source_names_shallow must also discover both sources.
        let shallow_names = extract_source_names_shallow(&sql_pipe_ast);
        assert!(
            shallow_names.iter().any(|n| n == "crowdstrike_detections"),
            "OBS-1: extract_source_names_shallow must include 'crowdstrike_detections' \
             (SqlPipe head source); got names: {shallow_names:?}"
        );
        assert!(
            shallow_names.iter().any(|n| n == "prism_audit"),
            "OBS-1: extract_source_names_shallow must discover 'prism_audit' \
             from SqlPipe PipeStage::Join source; got names: {shallow_names:?}"
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
    //! Ref: error-taxonomy.md §E-QUERY-036; BC-2.11.007 EC-001; P6-02 adjudication.

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
            matches!(err, PrismError::UnknownSourceTable(..)),
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

    /// AC-021 / GRAMMAR-004 / E-QUERY-036: UnknownSourceTable error carries available_tables
    /// and did_you_mean when the registry is non-empty.
    ///
    /// RED GATE: currently UnknownSourceTable { source_name: String } has no available_tables
    /// or did_you_mean fields.  After the fix the variant becomes a boxed struct and this
    /// test must pass.
    ///
    /// Mental-deletion proof: removing available_tables population at the emit site causes
    /// the available_tables assertion to fail; removing did_you_mean computation causes the
    /// did_you_mean assertion to fail.
    #[tokio::test]
    async fn test_BC_2_11_007_grammar004_unknown_source_table_carries_available_tables_and_did_you_mean(
    ) {
        let org_id = OrgId::new();
        let mut registry = AdapterRegistry::new();
        // Register "crowdstrike" — the user typo'd "crowdstrke" so did_you_mean should suggest it.
        registry.register(
            org_id,
            Arc::new(StubAdapterForUnknownTest {
                sensor_id: SensorId::new("crowdstrike"),
            }),
        );

        // "crowdstrke" is 1 Levenshtein distance from "crowdstrike" — within the ≤3 threshold.
        let source_names = vec!["crowdstrke_detections".to_string()];
        let clients = vec![];
        let org_registry = None;

        let result =
            super::resolve_source_refs(&source_names, &clients, &registry, &org_registry).await;

        let err = result.expect_err(
            "resolve_source_refs must return Err for unregistered 'crowdstrke'; got Ok",
        );

        // Must be the new boxed-struct variant.
        let PrismError::UnknownSourceTable(ref details) = err else {
            panic!(
                "AC-021: error must be PrismError::UnknownSourceTable(Box<UnknownSourceTableDetails>); got: {err:?}"
            );
        };

        // available_tables must list "crowdstrike" (the registered sensor prefix).
        assert!(
            details.available_tables.iter().any(|s| s == "crowdstrike"),
            "AC-021: available_tables must contain 'crowdstrike'; got: {:?}",
            details.available_tables
        );

        // did_you_mean must suggest "crowdstrike" (distance 1 ≤ 3).
        assert_eq!(
            details.did_you_mean.as_deref(),
            Some("crowdstrike"),
            "AC-021: did_you_mean must suggest 'crowdstrike' for 'crowdstrke'; got: {:?}",
            details.did_you_mean
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
            matches!(err, PrismError::UnknownSourceTable(..)),
            "error must be PrismError::UnknownSourceTable (E-QUERY-036) for invalid prefix; got: {err:?}"
        );
    }
}

// ---------------------------------------------------------------------------
// seed_armis_entity_discriminator unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod armis_discriminator_tests {
    //! F-L2-CRIT-001 (S-DEMO-FIDELITY-REMEDIATION-001) — unit tests for
    //! `seed_armis_entity_discriminator`.
    //!
    //! These tests prove the production seeding path: when `armis_alerts` is the
    //! source_table and no explicit `aql` predicate is present, the discriminator
    //! `"in:alerts"` is seeded into the filters map so the Armis DTU search
    //! endpoint selects alert records rather than defaulting to device records.
    //!
    //! SID-1 compliance: all tests run in-process with no external DTU dependency.
    //! The DTU round-trip assertion (verifying the Armis DTU actually receives
    //! `GET /api/v1/search?aql=in:alerts`) is deferred to the e2e_smoke.rs integration
    //! test suite (requires a running DTU; marked `#[ignore]` per SID-1 §4).

    use super::seed_armis_entity_discriminator;
    use prism_sensors::types::FilterMap;

    // ─── F-L2-CRIT-001 / AC-DISC-001 ─────────────────────────────────────────

    /// F-L2-CRIT-001 / AC-DISC-001 — Red Gate load-bearing test.
    ///
    /// `armis_alerts` with no prior `aql` entry → discriminator `"in:alerts"` seeded.
    ///
    /// This is the PRIMARY regression guard for F-L2-CRIT-001: before the fix,
    /// `where_filters.clone()` was passed directly, leaving `aql` absent. The DTU
    /// `get_search` route defaults to device records when `aql` is absent or
    /// does not contain `"in:alerts"` (EC-001 in routes/search.rs), so
    /// `armis_alerts` queries silently returned 0 rows after OCSF severity filtering.
    ///
    /// FAILS without `seed_armis_entity_discriminator` because the function did
    /// not exist; PASSES once it is implemented and wired.
    #[test]
    fn test_f_l2_crit001_armis_alerts_no_aql_seeds_in_alerts_discriminator() {
        let filters: FilterMap = FilterMap::new(); // no aql entry — mirrors a plain WHERE-free query
        let result = seed_armis_entity_discriminator("armis_alerts", filters);

        assert_eq!(
            result.get("aql").and_then(|v| v.as_str()),
            Some("in:alerts"),
            "F-L2-CRIT-001: seed_armis_entity_discriminator must set filters[\"aql\"] = \
             \"in:alerts\" for source_table \"armis_alerts\" when no aql predicate is present; \
             got: {:?}. Without this, the Armis DTU defaults to device records and \
             armis_alerts queries silently return 0 rows.",
            result.get("aql")
        );
    }

    /// F-L2-CRIT-001 / AC-DISC-002 — `armis_devices` with no prior `aql` entry
    /// must seed `"in:devices"`.
    ///
    /// Mirror of AC-DISC-001 for the devices table; ensures devices path is also
    /// explicit rather than relying on DTU default.
    #[test]
    fn test_f_l2_crit001_armis_devices_no_aql_seeds_in_devices_discriminator() {
        let filters: FilterMap = FilterMap::new();
        let result = seed_armis_entity_discriminator("armis_devices", filters);

        assert_eq!(
            result.get("aql").and_then(|v| v.as_str()),
            Some("in:devices"),
            "F-L2-CRIT-001: seed_armis_entity_discriminator must set filters[\"aql\"] = \
             \"in:devices\" for source_table \"armis_devices\" when no aql predicate present; \
             got: {:?}.",
            result.get("aql")
        );
    }

    /// F-L2-CRIT-001 / AC-DISC-003 — user-supplied `WHERE aql = 'in:alerts status:Open'`
    /// must NOT be overwritten.
    ///
    /// Preserves the verbatim-AQL-passthrough contract (BC-2.11.007 §Mechanism B):
    /// user-provided AQL strings reach the sensor API unchanged.
    #[test]
    fn test_f_l2_crit001_armis_alerts_existing_aql_not_overwritten() {
        let mut filters: FilterMap = FilterMap::new();
        filters.insert(
            "aql".to_string(),
            serde_json::Value::String("in:alerts status:Open".to_string()),
        );
        let result = seed_armis_entity_discriminator("armis_alerts", filters);

        assert_eq!(
            result.get("aql").and_then(|v| v.as_str()),
            Some("in:alerts status:Open"),
            "F-L2-CRIT-001: seed_armis_entity_discriminator must NOT overwrite a \
             user-supplied non-empty AQL predicate; expected \"in:alerts status:Open\", \
             got: {:?}.",
            result.get("aql")
        );
    }

    /// F-L2-CRIT-001 / AC-DISC-004 — non-armis source_tables are passed through unchanged.
    ///
    /// Guards against accidental AQL injection on CrowdStrike/Claroty/Cyberint tables.
    #[test]
    fn test_f_l2_crit001_non_armis_table_filters_unchanged() {
        let filters: FilterMap = FilterMap::new();
        let result = seed_armis_entity_discriminator("crowdstrike_alerts", filters);

        assert!(
            result.get("aql").is_none(),
            "F-L2-CRIT-001: seed_armis_entity_discriminator must NOT inject aql for \
             non-armis source_table \"crowdstrike_alerts\"; got: {:?}.",
            result.get("aql")
        );

        let filters2: FilterMap = FilterMap::new();
        let result2 = seed_armis_entity_discriminator("claroty_alerts", filters2);
        assert!(
            result2.get("aql").is_none(),
            "F-L2-CRIT-001: seed_armis_entity_discriminator must NOT inject aql for \
             \"claroty_alerts\"; got: {:?}.",
            result2.get("aql")
        );
    }
}

// ---------------------------------------------------------------------------
// F-LENS4-MED-001 — armis discriminator WIRING SEAM tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod armis_discriminator_wiring_seam_tests {
    //! F-LENS4-MED-001 (S-DEMO-FIDELITY-REMEDIATION-001) — load-bearing wiring seam tests
    //! for the Armis entity discriminator AQL injection in `run_materialization_pipeline`.
    //!
    //! ## Gap closed
    //!
    //! The four `armis_discriminator_tests` above call `seed_armis_entity_discriminator`
    //! directly in isolation. They do NOT exercise the CALL SITE in
    //! `run_materialization_pipeline` (line: `seed_armis_entity_discriminator(&target.source_table,
    //! where_filters.clone())`). A regression that reverts that line to `where_filters.clone()`
    //! — re-introducing the exact F-L2-CRIT-001 bug — would leave all isolation tests GREEN.
    //!
    //! These wiring seam tests drive `run_materialization_pipeline` with a
    //! `RecordingAdapter` that captures the `QueryParams.filters` it receives from the
    //! pipeline, then asserts `filters["aql"]` contains the expected discriminator value.
    //!
    //! ## SID-1 compliance
    //!
    //! All tests run in-process (no DTU, no subprocess). The `RecordingAdapter` returns
    //! `Ok(vec![])` (no rows) so the pipeline exits cleanly with an empty result;
    //! the assertion target is solely the captured `QueryParams.filters`.
    //!
    //! ## Mental-deletion / Red→Green proof
    //!
    //! If the call site in `run_materialization_pipeline` is reverted from:
    //!
    //! ```text
    //! seed_armis_entity_discriminator(&target.source_table, where_filters.clone())
    //! ```
    //!
    //! back to the pre-fix form:
    //!
    //! ```text
    //! where_filters.clone()
    //! ```
    //!
    //! then `RecordingAdapter::fetch` receives `params.filters` with NO `"aql"` key
    //! (because `where_filters` is empty for a no-WHERE-clause query), and
    //! **AC-WIRE-001** (`armis_alerts` no-WHERE) and **AC-WIRE-002** (`armis_devices`
    //! no-WHERE) FAIL with the assertion message: `filters["aql"]` is `None`.
    //!
    //! **AC-WIRE-003** (`armis_alerts` with `WHERE aql='in:alerts status:Open'`) does
    //! NOT depend on the seed call site. A `WHERE aql='...'` predicate populates
    //! `where_filters["aql"]` via `extract_push_down_filters_as_map` →
    //! `predicate_tree_to_filter_map` regardless of whether
    //! `seed_armis_entity_discriminator` is called. AC-WIRE-003 is therefore a
    //! **passthrough-contract guard** (user-supplied AQL preserved through the full
    //! pipeline), not a seam-revert guard. It stays GREEN on call-site revert.
    //!
    //! F-LENS4-MED-001 / TD-VSDD-059 / TD-VSDD-060.

    use std::sync::{Arc, Mutex};

    use async_trait::async_trait;
    use prism_core::{OrgId, SensorId};
    use prism_sensors::{
        adapter::{QueryParams, SensorSpec},
        AdapterRegistry, BearerStaticSensorAuth, CredentialResolver, SensorAdapter, SensorAuth,
        SensorError,
    };

    use crate::{
        engine::QueryOptions,
        materialization::{run_materialization_pipeline, MaterializationContext},
        memory::build_session_context,
    };

    // ──────────────────────────────────────────────────────────────────────────
    // RecordingAdapter
    // ──────────────────────────────────────────────────────────────────────────

    /// A `SensorAdapter` stub that records every `QueryParams.filters` map it receives
    /// and returns zero rows. Used to assert the discriminator AQL is present in the
    /// `filters` the pipeline hands to the adapter — proving the WIRING SEAM is intact.
    ///
    /// `Arc<Mutex<Vec<...>>>` is the test-scoped recording channel.  No DTU or
    /// external process is involved (SID-1 compliance).
    struct RecordingAdapter {
        sensor_id: SensorId,
        /// Accumulates the `QueryParams.filters` for each `fetch` call.
        captured_filters: Arc<Mutex<Vec<prism_sensors::types::FilterMap>>>,
    }

    #[async_trait]
    impl SensorAdapter for RecordingAdapter {
        fn sensor_type(&self) -> SensorId {
            self.sensor_id.clone()
        }

        fn sensor_name(&self) -> &'static str {
            "recording-adapter-wiring-seam"
        }

        async fn fetch(
            &self,
            _spec: &SensorSpec,
            params: &QueryParams,
            _auth: &dyn SensorAuth,
        ) -> Result<Vec<arrow::record_batch::RecordBatch>, SensorError> {
            // Record the filters map received from the pipeline.
            self.captured_filters
                .lock()
                .expect("RecordingAdapter: captured_filters lock must not be poisoned")
                .push(params.filters.clone());
            // Return zero rows — we only care about the captured filters, not the result.
            Ok(vec![])
        }
    }

    // ──────────────────────────────────────────────────────────────────────────
    // StubCredentialResolver
    // ──────────────────────────────────────────────────────────────────────────

    /// Stub `CredentialResolver` that always returns a test bearer token.
    ///
    /// The `NullMaterializationCredentialResolver` returns `SensorError::Internal` for
    /// every resolve, which would prevent `fan_out()` from calling `fetch()`. This stub
    /// returns a minimal `BearerStaticSensorAuth("wiring-seam-test-token")` so the fan-out
    /// reaches the adapter without credential failures. The `RecordingAdapter::fetch`
    /// ignores `_auth` — only the captured filters matter.
    struct StubCredentialResolver;

    impl CredentialResolver for StubCredentialResolver {
        fn resolve(
            &self,
            _client_id: &str,
            _sensor_id: SensorId,
        ) -> Result<Box<dyn SensorAuth>, SensorError> {
            Ok(Box::new(BearerStaticSensorAuth::new(
                "wiring-seam-test-token",
            )))
        }
    }

    /// Build a minimal `MaterializationContext` with a single `RecordingAdapter` for
    /// sensor `sensor_id`, wired with a `StubCredentialResolver` so `fan_out()` reaches
    /// the adapter's `fetch()` method.
    ///
    /// Returns the `MaterializationContext` and the shared `captured_filters` channel.
    fn make_context_with_recording_adapter(
        sensor_id: SensorId,
    ) -> (
        MaterializationContext,
        Arc<Mutex<Vec<prism_sensors::types::FilterMap>>>,
    ) {
        let org_id = OrgId::new();
        let captured = Arc::new(Mutex::new(Vec::new()));
        let adapter: Arc<dyn SensorAdapter> = Arc::new(RecordingAdapter {
            sensor_id: sensor_id.clone(),
            captured_filters: Arc::clone(&captured),
        });
        let mut registry = AdapterRegistry::new();
        registry.register(org_id, adapter);

        // Use new_with_resolver so fan_out() succeeds past the credential check.
        // NullMaterializationCredentialResolver would short-circuit at credentials.resolve()
        // and produce a FanOutError before fetch() is ever called.
        let ctx = MaterializationContext::new_with_resolver(
            Arc::new(registry),
            Arc::new(prism_ocsf::OcsfNormalizer::new()),
            10_000,
            Arc::new(StubCredentialResolver),
            None, // no OrgRegistry — test mode synthetic slug fallback
            None, // no resolved_spec_map — test mode
        );
        (ctx, captured)
    }

    // ──────────────────────────────────────────────────────────────────────────
    // F-LENS4-MED-001 / AC-WIRE-001: armis_alerts → filters["aql"] == "in:alerts"
    // ──────────────────────────────────────────────────────────────────────────

    /// F-LENS4-MED-001 / AC-WIRE-001 — WIRING SEAM load-bearing test.
    ///
    /// Drives `run_materialization_pipeline` with a `FROM armis_alerts` query (no WHERE
    /// clause, so `where_filters` is empty). The `RecordingAdapter` captures the
    /// `QueryParams.filters` the pipeline passes to `fetch()`. Asserts that
    /// `filters["aql"] == "in:alerts"` — proving the
    /// `seed_armis_entity_discriminator(&target.source_table, where_filters.clone())`
    /// CALL SITE is intact.
    ///
    /// ## Red→Green proof
    ///
    /// Revert the call site to `where_filters.clone()` → `RecordingAdapter::fetch`
    /// receives `params.filters = {}` (empty, no `"aql"` key) → assertion FAILS.
    /// Restore the call site → test PASSES.
    #[allow(non_snake_case)]
    #[tokio::test]
    async fn test_F_LENS4_MED001_armis_alerts_pipeline_seeds_in_alerts_aql_filter() {
        let armis_sensor_id = SensorId::new("armis");
        let (mut mat_ctx, captured_filters) = make_context_with_recording_adapter(armis_sensor_id);

        // 50 MiB session pool — sufficient for a zero-row result.
        let session_ctx = build_session_context(50 * 1024 * 1024)
            .expect("build_session_context must succeed for wiring seam test");

        // No WHERE clause → `where_filters` is empty inside the pipeline.
        // Without the discriminator call site, `fetch` receives `filters = {}`.
        let query = "SELECT * FROM armis_alerts";
        let options = QueryOptions::default(); // clients: None, no filters

        let result = run_materialization_pipeline(query, &options, &mut mat_ctx, &session_ctx)
            .await
            .expect("run_materialization_pipeline must succeed for armis_alerts wiring seam test");

        // The pipeline returns empty batches (RecordingAdapter returned no rows),
        // but the adapter MUST have been called exactly once.
        let calls = captured_filters
            .lock()
            .expect("captured_filters lock must not be poisoned");

        assert_eq!(
            calls.len(),
            1,
            "F-LENS4-MED-001 / AC-WIRE-001: RecordingAdapter::fetch must have been called \
             exactly once for FROM armis_alerts; got {} calls. \
             If 0 calls: the pipeline returned before fan-out (adapter not registered or \
             sensor_id mismatch). If >1: unexpected fan-out to multiple targets.",
            calls.len()
        );

        let received_filters = &calls[0];
        assert_eq!(
            received_filters.get("aql").and_then(|v| v.as_str()),
            Some("in:alerts"),
            "F-LENS4-MED-001 / AC-WIRE-001 (WIRING SEAM): \
             run_materialization_pipeline must seed filters[\"aql\"] = \"in:alerts\" for \
             FROM armis_alerts with no WHERE clause. \
             Got: {:?}. \
             Root cause: the call site `seed_armis_entity_discriminator(&target.source_table, \
             where_filters.clone())` was removed or reverted to `where_filters.clone()`, \
             re-introducing F-L2-CRIT-001 (armis_alerts returns 0 rows).",
            received_filters.get("aql")
        );

        // Batches are empty (RecordingAdapter returned no rows) — that's expected.
        assert!(
            result.batches.is_empty(),
            "wiring seam test expects empty batches (RecordingAdapter returns no rows); \
             got {} batch(es)",
            result.batches.len()
        );
    }

    // ──────────────────────────────────────────────────────────────────────────
    // F-LENS4-MED-001 / AC-WIRE-002: armis_devices → filters["aql"] == "in:devices"
    // ──────────────────────────────────────────────────────────────────────────

    /// F-LENS4-MED-001 / AC-WIRE-002 — WIRING SEAM companion for `armis_devices`.
    ///
    /// Mirror of AC-WIRE-001 for the devices table. A query `FROM armis_devices`
    /// with no WHERE clause must arrive at `RecordingAdapter::fetch` with
    /// `filters["aql"] == "in:devices"`.
    #[allow(non_snake_case)]
    #[tokio::test]
    async fn test_F_LENS4_MED001_armis_devices_pipeline_seeds_in_devices_aql_filter() {
        let armis_sensor_id = SensorId::new("armis");
        let (mut mat_ctx, captured_filters) = make_context_with_recording_adapter(armis_sensor_id);

        let session_ctx = build_session_context(50 * 1024 * 1024)
            .expect("build_session_context must succeed for armis_devices wiring seam test");

        let query = "SELECT * FROM armis_devices";
        let options = QueryOptions::default();

        run_materialization_pipeline(query, &options, &mut mat_ctx, &session_ctx)
            .await
            .expect("run_materialization_pipeline must succeed for armis_devices wiring seam test");

        let calls = captured_filters
            .lock()
            .expect("captured_filters lock must not be poisoned");

        assert_eq!(
            calls.len(),
            1,
            "F-LENS4-MED-001 / AC-WIRE-002: RecordingAdapter::fetch must have been called \
             exactly once for FROM armis_devices; got {} calls.",
            calls.len()
        );

        assert_eq!(
            calls[0].get("aql").and_then(|v| v.as_str()),
            Some("in:devices"),
            "F-LENS4-MED-001 / AC-WIRE-002 (WIRING SEAM): \
             run_materialization_pipeline must seed filters[\"aql\"] = \"in:devices\" for \
             FROM armis_devices with no WHERE clause. \
             Got: {:?}. \
             Root cause: call site reverted to `where_filters.clone()` (F-L2-CRIT-001 regression).",
            calls[0].get("aql")
        );
    }

    // ──────────────────────────────────────────────────────────────────────────
    // F-LENS4-MED-001 / AC-WIRE-003: user-supplied WHERE aql='...' passes through
    // ──────────────────────────────────────────────────────────────────────────

    /// F-LENS4-MED-001 / AC-WIRE-003 — WIRING SEAM passthrough test.
    ///
    /// A query `FROM armis_alerts WHERE aql = 'in:alerts status:Open'` must arrive
    /// at `RecordingAdapter::fetch` with the user-supplied AQL preserved verbatim —
    /// the discriminator must NOT overwrite a non-empty user-supplied `aql` predicate.
    ///
    /// This tests BC-2.11.007 §Mechanism B passthrough via the full pipeline (not just
    /// the helper in isolation).
    #[allow(non_snake_case)]
    #[tokio::test]
    async fn test_F_LENS4_MED001_armis_alerts_user_supplied_aql_passes_through_pipeline() {
        let armis_sensor_id = SensorId::new("armis");
        let (mut mat_ctx, captured_filters) = make_context_with_recording_adapter(armis_sensor_id);

        let session_ctx = build_session_context(50 * 1024 * 1024)
            .expect("build_session_context must succeed for armis passthrough test");

        // User supplies an explicit AQL predicate — must not be overwritten.
        let query = "SELECT * FROM armis_alerts WHERE aql = 'in:alerts status:Open'";
        let options = QueryOptions::default();

        run_materialization_pipeline(query, &options, &mut mat_ctx, &session_ctx)
            .await
            .expect(
                "run_materialization_pipeline must succeed for armis_alerts passthrough wiring test",
            );

        let calls = captured_filters
            .lock()
            .expect("captured_filters lock must not be poisoned");

        assert_eq!(
            calls.len(),
            1,
            "F-LENS4-MED-001 / AC-WIRE-003: RecordingAdapter::fetch must have been called \
             exactly once; got {} calls.",
            calls.len()
        );

        assert_eq!(
            calls[0].get("aql").and_then(|v| v.as_str()),
            Some("in:alerts status:Open"),
            "F-LENS4-MED-001 / AC-WIRE-003 (WIRING SEAM passthrough): \
             run_materialization_pipeline must preserve a user-supplied non-empty \
             WHERE aql = '...' predicate verbatim. \
             Expected \"in:alerts status:Open\", got: {:?}. \
             Root cause: seed_armis_entity_discriminator overwrote the user predicate, \
             or the WHERE clause was not pushed down to QueryParams.filters.",
            calls[0].get("aql")
        );
    }
}

// ---------------------------------------------------------------------------
// Unit tests — temporal walker new-predicate coverage (OBS-1 fix)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod temporal_walker_unit_tests {
    //! Unit tests for the temporal literal walker covering `Predicate::Between`,
    //! `Predicate::In`, `Predicate::InSubquery`, `Expr::Compare` in JOIN ON,
    //! and `Expr::Compare` in SELECT items (OBS-1 coverage from LOCAL adversary pass 1).

    use std::sync::Arc;

    use prism_core::error::PrismError;

    use crate::{
        ast::{
            AggFunc, Ast, CompareOp, Expr, FieldPath, FromClause, FuncCall, Join, JoinKind,
            Literal, LogicalOp, Predicate, ScalarFunc, SelectClause, SelectItem, SourceRef,
            SourceRefKind, Span, SqlQuery, SqlStatement,
        },
        table_registry::TableRegistry,
    };
    // Private helpers in the parent (materialization) module — accessible via `super::`.
    use super::{
        apply_literal_dispatch, check_expr_temporal, check_pred_raw_temporal,
        check_temporal_literals,
    };

    fn make_registry() -> Arc<TableRegistry> {
        use prism_core::ColumnType;
        use prism_spec_engine::spec_parser::{AuthType, ColumnSpec, SensorSpec, TableSpec};

        let registry = Arc::new(TableRegistry::new());
        let spec = SensorSpec::new(
            "test",
            "Test sensor",
            AuthType::ApiKey,
            "https://test.invalid",
            vec![TableSpec::new_point_in_time(
                "events",
                "security_finding",
                vec![
                    ColumnSpec::new("timestamp", ColumnType::Datetime, None, vec![]),
                    ColumnSpec::new("hostname", ColumnType::String, None, vec![]),
                    ColumnSpec::new("count", ColumnType::Integer, None, vec![]),
                ],
                vec![],
            )],
            None,
            "1.0.0",
            vec![],
        );
        registry
            .register_sensor(&spec)
            .expect("register test sensor must not fail");
        registry
    }

    fn fp(col: &str) -> FieldPath {
        FieldPath {
            segments: vec![col.to_string()],
            span: Span::ZERO,
        }
    }

    fn raw_lit(s: &str) -> Literal {
        Literal::RawTemporalLiteral(s.to_string())
    }

    fn minimal_select(table: &str) -> SqlQuery {
        SqlQuery {
            select: SelectClause {
                distinct: false,
                items: vec![SelectItem::Star],
            },
            from: FromClause {
                source: SourceRef {
                    raw: table.to_string(),
                    kind: SourceRefKind::Custom,
                },
                alias: None,
            },
            joins: vec![],
            where_: None,
            group_by: vec![],
            having: None,
            order_by: vec![],
            limit: None,
        }
    }

    // ── CRIT-1: Predicate::Between ─────────────────────────────────────────────

    #[test]
    fn test_between_datetime_column_fires_e_query_041() {
        let registry = make_registry();
        let mut pred = Predicate::Between {
            field: fp("timestamp"),
            low: raw_lit("2026-06-24"),
            high: raw_lit("2026-06-25"),
            negated: false,
        };
        let result =
            check_pred_raw_temporal(&mut pred, Some("test_events"), Some(registry.as_ref()));
        assert!(
            matches!(&result, Err(PrismError::TemporalLiteralUnparseable { .. })),
            "OBS-1/CRIT-1: Between+Datetime must → E-QUERY-041, got {result:?}"
        );
    }

    #[test]
    fn test_between_string_column_coerces_literals() {
        let registry = make_registry();
        let mut pred = Predicate::Between {
            field: fp("hostname"),
            low: raw_lit("2026-06-24"),
            high: raw_lit("2026-06-25"),
            negated: false,
        };
        check_pred_raw_temporal(&mut pred, Some("test_events"), Some(registry.as_ref()))
            .expect("Between+String must not error");
        match &pred {
            Predicate::Between { low, high, .. } => {
                assert!(
                    matches!(low, Literal::String(_)),
                    "low must be Literal::String"
                );
                assert!(
                    matches!(high, Literal::String(_)),
                    "high must be Literal::String"
                );
            }
            _ => panic!("pred must remain Between after coercion"),
        }
    }

    // ── CRIT-2: Predicate::In ──────────────────────────────────────────────────

    #[test]
    fn test_in_datetime_column_fires_e_query_041() {
        let registry = make_registry();
        let mut pred = Predicate::In {
            field: fp("timestamp"),
            values: vec![raw_lit("2026-06-24"), raw_lit("2026-06-25")],
            negated: false,
        };
        let result =
            check_pred_raw_temporal(&mut pred, Some("test_events"), Some(registry.as_ref()));
        assert!(
            matches!(&result, Err(PrismError::TemporalLiteralUnparseable { .. })),
            "OBS-1/CRIT-2: In+Datetime must → E-QUERY-041, got {result:?}"
        );
    }

    #[test]
    fn test_in_string_column_coerces_all_values() {
        let registry = make_registry();
        let mut pred = Predicate::In {
            field: fp("hostname"),
            values: vec![raw_lit("2026-06-24"), raw_lit("2026-06-25")],
            negated: false,
        };
        check_pred_raw_temporal(&mut pred, Some("test_events"), Some(registry.as_ref()))
            .expect("In+String must not error");
        match &pred {
            Predicate::In { values, .. } => {
                for (i, val) in values.iter().enumerate() {
                    assert!(
                        matches!(val, Literal::String(_)),
                        "value[{i}] must be Literal::String"
                    );
                }
            }
            _ => panic!("pred must remain In after coercion"),
        }
    }

    // ── HIGH-2: Predicate::InSubquery ──────────────────────────────────────────

    #[test]
    fn test_in_subquery_where_clause_fires_e_query_041() {
        let registry = make_registry();
        let mut subquery = minimal_select("test_events");
        subquery.where_ = Some(Predicate::Compare {
            lhs: Box::new(Expr::Field(fp("timestamp"))),
            op: CompareOp::Gt,
            rhs: Box::new(Expr::Literal(raw_lit("2026-06-24"))),
        });
        let mut pred = Predicate::InSubquery {
            field: fp("id"),
            subquery: Box::new(subquery),
            negated: false,
        };
        let result =
            check_pred_raw_temporal(&mut pred, Some("test_events"), Some(registry.as_ref()));
        assert!(
            matches!(&result, Err(PrismError::TemporalLiteralUnparseable { .. })),
            "OBS-1/HIGH-2: InSubquery with datetime RawTemporalLiteral → E-QUERY-041, got {result:?}"
        );
    }

    // ── HIGH-2: Expr::Compare in JOIN ON ───────────────────────────────────────

    #[test]
    fn test_join_on_expr_compare_datetime_fires_e_query_041() {
        let registry = make_registry();
        let mut sql = minimal_select("test_events");
        sql.joins = vec![Join {
            kind: JoinKind::Inner,
            source: SourceRef {
                raw: "other_table".to_string(),
                kind: SourceRefKind::Custom,
            },
            alias: None,
            on: Expr::Compare {
                lhs: Box::new(Expr::Field(fp("timestamp"))),
                op: CompareOp::Gt,
                rhs: Box::new(Expr::Literal(raw_lit("2026-06-24"))),
            },
        }];
        let mut ast = Ast::Sql(SqlStatement::Select(sql));
        let result = check_temporal_literals(&mut ast, Some(registry.as_ref()), false);
        assert!(
            matches!(&result, Err(PrismError::TemporalLiteralUnparseable { .. })),
            "OBS-1/HIGH-2: JOIN ON Expr::Compare+Datetime → E-QUERY-041, got {result:?}"
        );
    }

    // ── HIGH-2: Expr::Compare in SELECT items ──────────────────────────────────

    #[test]
    fn test_select_item_expr_compare_datetime_fires_e_query_041() {
        let registry = make_registry();
        let mut sql = minimal_select("test_events");
        sql.select = SelectClause {
            distinct: false,
            items: vec![SelectItem::Expr {
                expr: Expr::Compare {
                    lhs: Box::new(Expr::Field(fp("timestamp"))),
                    op: CompareOp::Gt,
                    rhs: Box::new(Expr::Literal(raw_lit("2026-06-24"))),
                },
                alias: Some("is_recent".to_string()),
            }],
        };
        let mut ast = Ast::Sql(SqlStatement::Select(sql));
        let result = check_temporal_literals(&mut ast, Some(registry.as_ref()), false);
        assert!(
            matches!(&result, Err(PrismError::TemporalLiteralUnparseable { .. })),
            "OBS-1/HIGH-2: SELECT Expr::Compare+Datetime → E-QUERY-041, got {result:?}"
        );
    }

    // ── apply_literal_dispatch direct tests ───────────────────────────────────

    #[test]
    fn test_apply_literal_dispatch_datetime_errors() {
        let registry = make_registry();
        let mut lit = raw_lit("2026-06-24");
        let result = apply_literal_dispatch(
            &mut lit,
            &fp("timestamp"),
            Some("test_events"),
            Some(registry.as_ref()),
            "=",
        );
        assert!(
            matches!(&result, Err(PrismError::TemporalLiteralUnparseable { .. })),
            "apply_literal_dispatch+Datetime → E-QUERY-041, got {result:?}"
        );
    }

    #[test]
    fn test_apply_literal_dispatch_string_coerces() {
        let registry = make_registry();
        let mut lit = raw_lit("2026-06-24");
        apply_literal_dispatch(
            &mut lit,
            &fp("hostname"),
            Some("test_events"),
            Some(registry.as_ref()),
            "=",
        )
        .expect("apply_literal_dispatch+String must not error");
        assert!(
            matches!(&lit, Literal::String(_)),
            "must coerce to Literal::String, got {lit:?}"
        );
    }

    #[test]
    fn test_apply_literal_dispatch_non_raw_noop() {
        let registry = make_registry();
        let mut lit = Literal::String("hello".to_string());
        apply_literal_dispatch(
            &mut lit,
            &fp("timestamp"),
            Some("test_events"),
            Some(registry.as_ref()),
            "=",
        )
        .expect("non-RawTemporalLiteral must be no-op");
        assert!(
            matches!(&lit, Literal::String(s) if s == "hello"),
            "must not modify non-RawTemporalLiteral"
        );
    }

    // ── check_expr_temporal direct test ──────────────────────────────────────

    #[test]
    fn test_check_expr_temporal_bare_raw_temporal_errors() {
        // ADR-052 §D4 v1.8 OBS-2: bare RawTemporalLiteral in non-comparison position
        // COERCES in-place to Literal::String instead of returning QueryPlanFailed.
        let registry = make_registry();
        let mut expr = Expr::Literal(raw_lit("2026-06-24"));
        let result = check_expr_temporal(&mut expr, Some("test_events"), Some(registry.as_ref()));
        assert!(
            result.is_ok(),
            "OBS-2: bare RawTemporalLiteral in non-comparison Expr → coerce + Ok(()), got {result:?}"
        );
        assert!(
            matches!(&expr, Expr::Literal(Literal::String(s)) if s == "2026-06-24"),
            "OBS-2: coerced expression must be Literal::String('2026-06-24'), got {expr:?}"
        );
    }

    // ── P3-MED-1 regression: operator field accuracy ──────────────────────────

    #[test]
    fn test_apply_literal_dispatch_between_carries_operator_label() {
        // P3-MED-1: BETWEEN position must report operator "BETWEEN" in E-QUERY-002,
        // not the hardcoded "=" that was present before this fix.
        let registry = make_registry();
        let mut lit = raw_lit("2026-06-24");
        let result = apply_literal_dispatch(
            &mut lit,
            &fp("count"), // ColumnType::Integer in test_events
            Some("test_events"),
            Some(registry.as_ref()),
            "BETWEEN",
        );
        match result {
            Err(PrismError::QueryTypeMismatch { operator, .. }) => {
                assert_eq!(
                    operator, "BETWEEN",
                    "P3-MED-1: operator field must reflect actual BETWEEN, got '{operator}'"
                );
            }
            other => panic!("expected QueryTypeMismatch, got {other:?}"),
        }
    }

    #[test]
    fn test_apply_literal_dispatch_in_carries_operator_label() {
        // P3-MED-1: IN position must report operator "IN" in E-QUERY-002.
        let registry = make_registry();
        let mut lit = raw_lit("2026-06-24");
        let result = apply_literal_dispatch(
            &mut lit,
            &fp("count"), // ColumnType::Integer
            Some("test_events"),
            Some(registry.as_ref()),
            "IN",
        );
        match result {
            Err(PrismError::QueryTypeMismatch { operator, .. }) => {
                assert_eq!(
                    operator, "IN",
                    "P3-MED-1: operator field must reflect actual IN, got '{operator}'"
                );
            }
            other => panic!("expected QueryTypeMismatch, got {other:?}"),
        }
    }

    // ── F-P4-OBS-1: Between high-arm isolation ────────────────────────────────

    #[test]
    fn test_between_high_arm_fires_when_low_passes() {
        // F-P4-OBS-1: ensures the HIGH arm of Between dispatch is actually exercised.
        // If apply_literal_dispatch(high, ...) were accidentally removed/skipped,
        // this test would pass silently (String coerce for hostname) but no error would
        // fire for timestamp high arm.
        // Here: low = String literal (passes apply_literal_dispatch no-op),
        //       high = RawTemporalLiteral against Datetime → must fire E-QUERY-041.
        let registry = make_registry();
        let mut pred = Predicate::Between {
            field: fp("timestamp"),
            low: Literal::String("2026-06-24".to_string()), // not RawTemporalLiteral → no-op
            high: raw_lit("2026-06-25"),                    // RawTemporalLiteral → E-QUERY-041
            negated: false,
        };
        let result =
            check_pred_raw_temporal(&mut pred, Some("test_events"), Some(registry.as_ref()));
        assert!(
            matches!(&result, Err(PrismError::TemporalLiteralUnparseable { .. })),
            "F-P4-OBS-1: Between high arm must fire E-QUERY-041 even when low passes; got {result:?}"
        );
    }

    // ── F-P4-OBS-2: Predicate::Not and Predicate::Logical recursion ──────────

    #[test]
    fn test_predicate_not_recurses_into_inner() {
        // F-P4-OBS-2: NOT (timestamp > '2026-06-24') must fire E-QUERY-041 via recursion
        // through Predicate::Not(inner).
        let registry = make_registry();
        let inner = Predicate::Compare {
            lhs: Box::new(Expr::Field(fp("timestamp"))),
            op: CompareOp::Gt,
            rhs: Box::new(Expr::Literal(raw_lit("2026-06-24"))),
        };
        let mut pred = Predicate::Not(Box::new(inner));
        let result =
            check_pred_raw_temporal(&mut pred, Some("test_events"), Some(registry.as_ref()));
        assert!(
            matches!(&result, Err(PrismError::TemporalLiteralUnparseable { .. })),
            "F-P4-OBS-2: NOT(temporal compare) must propagate E-QUERY-041, got {result:?}"
        );
    }

    #[test]
    fn test_predicate_logical_recurses_into_children() {
        // F-P4-OBS-2: hostname = 'ok' AND timestamp > '2026-06-24' must fire E-QUERY-041
        // via recursion through Predicate::Logical { predicates }.
        let registry = make_registry();
        let safe_pred = Predicate::Compare {
            lhs: Box::new(Expr::Field(fp("hostname"))),
            op: CompareOp::Eq,
            rhs: Box::new(Expr::Literal(Literal::String("ok".to_string()))),
        };
        let temporal_pred = Predicate::Compare {
            lhs: Box::new(Expr::Field(fp("timestamp"))),
            op: CompareOp::Gt,
            rhs: Box::new(Expr::Literal(raw_lit("2026-06-24"))),
        };
        let mut pred = Predicate::Logical {
            op: LogicalOp::And,
            predicates: vec![safe_pred, temporal_pred],
        };
        let result =
            check_pred_raw_temporal(&mut pred, Some("test_events"), Some(registry.as_ref()));
        assert!(
            matches!(&result, Err(PrismError::TemporalLiteralUnparseable { .. })),
            "F-P4-OBS-2: Logical(AND) with nested temporal compare must propagate E-QUERY-041, got {result:?}"
        );
    }

    // ── F-P4-MED-1 regression: FuncCall args are walked ──────────────────────

    #[test]
    fn test_funcall_arg_raw_temporal_fires_plan_failed() {
        // F-P4-MED-1 (updated for OBS-2): RawTemporalLiteral inside a scalar function argument
        // is still caught by check_expr_temporal's FuncCall arm (not silently ignored via
        // the old `_ => Ok(())`) — the walking behavior is preserved. Under ADR-052 §D4 v1.8
        // OBS-2, the bare RawTemporalLiteral in arg position is now COERCED to Literal::String
        // (non-comparison position, no column type context) instead of returning QueryPlanFailed.
        // The arg is coerced in-place; the function call receives a plain string constant.
        let mut expr = Expr::FuncCall(FuncCall::Scalar {
            func: ScalarFunc::IocMatch,
            args: vec![Expr::Literal(raw_lit("2026-06-24"))],
        });
        let result = check_expr_temporal(&mut expr, Some("test_events"), None);
        // Walking happens — OBS-2 coerces the arg instead of erroring.
        assert!(
            result.is_ok(),
            "F-P4-MED-1 OBS-2: bare RawTemporalLiteral in FuncCall arg → coerce + Ok(()), got {result:?}"
        );
        // Verify the arg was coerced to Literal::String.
        if let Expr::FuncCall(FuncCall::Scalar { args, .. }) = &expr {
            assert!(
                matches!(&args[0], Expr::Literal(Literal::String(s)) if s == "2026-06-24"),
                "F-P4-MED-1 OBS-2: FuncCall arg must be coerced to Literal::String, got {args:?}"
            );
        } else {
            panic!("F-P4-MED-1: outer Expr must remain FuncCall::Scalar, got {expr:?}");
        }
    }

    // ── 3-segment FieldPath resolution (LOW-3 coverage) ────────────────────────

    /// Build a FieldPath with multiple dotted segments (e.g., `sensor.table.column`).
    fn fp3(seg0: &str, seg1: &str, seg2: &str) -> FieldPath {
        FieldPath {
            segments: vec![seg0.to_string(), seg1.to_string(), seg2.to_string()],
            span: Span::ZERO,
        }
    }

    /// Build a registry with an External-source-style composite table name.
    fn make_external_registry() -> Arc<TableRegistry> {
        use prism_core::ColumnType;
        use prism_spec_engine::spec_parser::{AuthType, ColumnSpec, SensorSpec, TableSpec};

        // Registers as "test_events" (Custom) — composite name is "test_events".
        // To simulate External dotted-notation we register under "sensor_table" naming.
        let registry = Arc::new(TableRegistry::new());
        let spec = SensorSpec::new(
            "sensor",
            "External sensor",
            AuthType::ApiKey,
            "https://sensor.invalid",
            vec![TableSpec::new_point_in_time(
                "table",
                "security_finding",
                vec![
                    ColumnSpec::new("timestamp", ColumnType::Datetime, None, vec![]),
                    ColumnSpec::new("hostname", ColumnType::String, None, vec![]),
                ],
                vec![],
            )],
            None,
            "1.0.0",
            vec![],
        );
        registry
            .register_sensor(&spec)
            .expect("register external sensor must not fail");
        registry
    }

    #[test]
    fn test_resolve_col_type_three_segment_composite_key() {
        // LOW-3 coverage: sensor.table.column → composite key "sensor_table" lookup.
        // The registry has a table registered as "sensor_table" (External source convention).
        // A 3-segment FieldPath ["sensor", "table", "timestamp"] must resolve to Datetime.
        use super::resolve_col_type;
        use prism_core::ColumnType;

        let registry = make_external_registry();
        let fp = fp3("sensor", "table", "timestamp");
        let result = resolve_col_type(&fp, None, &registry);
        assert_eq!(
            result,
            Some(ColumnType::Datetime),
            "3-segment FieldPath ['sensor', 'table', 'timestamp'] must resolve via composite \
             key 'sensor_table' → Datetime. Got: {result:?}"
        );
    }

    #[test]
    fn test_resolve_col_type_three_segment_missing_composite_fallback() {
        // LOW-3 / OBS-P7-2 coverage: 3-segment path where composite key is NOT in registry.
        // No fallback to segments[0] (OBS-P7-2 fix — prevents over-resolution of nested struct paths).
        // Must return None (fail-open) when composite key misses.
        use super::resolve_col_type;

        let registry = make_external_registry();
        let fp = fp3("other", "sensor", "timestamp"); // "other_sensor" not in registry
        let result = resolve_col_type(&fp, None, &registry);
        assert_eq!(
            result, None,
            "3-segment FieldPath with no matching composite key must return None. Got: {result:?}"
        );
    }

    #[test]
    fn test_dml_walker_assignment_datetime_col_fires_e_query_041() {
        // MED-1 coverage: DML assignment to Datetime column with RawTemporalLiteral.
        // `UPDATE test_events SET timestamp = '2026-06-24'` should fire E-QUERY-041.
        use super::check_temporal_literals;
        use crate::write_ast::{Assignment, DmlNode, DmlOperation};

        let registry = make_registry();
        let mut ast = Ast::Sql(SqlStatement::Dml(DmlNode {
            operation: DmlOperation::Update,
            target_table: "test_events".to_string(),
            columns: None,
            assignments: vec![Assignment {
                column: "timestamp".to_string(),
                value: Expr::Literal(raw_lit("2026-06-24")),
            }],
            filter: None,
            source_select: None,
        }));

        let result = check_temporal_literals(&mut ast, Some(registry.as_ref()), false);
        assert!(
            matches!(&result, Err(PrismError::TemporalLiteralUnparseable { .. })),
            "MED-1: DML assignment of RawTemporalLiteral to Datetime column must → E-QUERY-041. \
             Got: {result:?}"
        );
    }

    #[test]
    fn test_dml_walker_assignment_string_col_coerces() {
        // MED-1 coverage: DML assignment to String column with RawTemporalLiteral → COERCE.
        // `UPDATE test_events SET hostname = '2026-06-24'` should coerce → Literal::String.
        use super::check_temporal_literals;
        use crate::write_ast::{Assignment, DmlNode, DmlOperation};

        let registry = make_registry();
        let mut ast = Ast::Sql(SqlStatement::Dml(DmlNode {
            operation: DmlOperation::Update,
            target_table: "test_events".to_string(),
            columns: None,
            assignments: vec![Assignment {
                column: "hostname".to_string(),
                value: Expr::Literal(raw_lit("2026-06-24")),
            }],
            filter: None,
            source_select: None,
        }));

        let result = check_temporal_literals(&mut ast, Some(registry.as_ref()), false);
        assert!(
            result.is_ok(),
            "MED-1: DML assignment of RawTemporalLiteral to String column must coerce → Ok. \
             Got: {result:?}"
        );
        // Verify coercion happened in-place.
        if let Ast::Sql(SqlStatement::Dml(ref dml)) = ast {
            assert!(
                matches!(&dml.assignments[0].value, Expr::Literal(Literal::String(_))),
                "MED-1: coerced assignment value must be Literal::String. Got: {:?}",
                dml.assignments[0].value
            );
        }
    }

    #[test]
    fn test_dml_walker_filter_uses_target_table_as_primary() {
        // MED-1 coverage: DML WHERE predicate uses target_table as primary_table, so
        // unqualified column references resolve correctly.
        // `UPDATE test_events SET hostname = '...' WHERE timestamp > '2026-06-24'` → E-QUERY-041.
        use super::check_temporal_literals;
        use crate::ast::Span;
        use crate::write_ast::{Assignment, DmlNode, DmlOperation};

        let registry = make_registry();
        let ts_field = Expr::Field(FieldPath {
            segments: vec!["timestamp".to_string()],
            span: Span::ZERO,
        });
        let where_pred = Predicate::Compare {
            lhs: Box::new(ts_field),
            rhs: Box::new(Expr::Literal(raw_lit("2026-06-24"))),
            op: CompareOp::Gt,
        };

        let mut ast = Ast::Sql(SqlStatement::Dml(DmlNode {
            operation: DmlOperation::Update,
            target_table: "test_events".to_string(),
            columns: None,
            assignments: vec![Assignment {
                column: "hostname".to_string(),
                value: Expr::Literal(Literal::String("new_name".to_string())),
            }],
            filter: Some(where_pred),
            source_select: None,
        }));

        let result = check_temporal_literals(&mut ast, Some(registry.as_ref()), false);
        assert!(
            matches!(&result, Err(PrismError::TemporalLiteralUnparseable { .. })),
            "MED-1: DML WHERE with unqualified timestamp vs RawTemporalLiteral must fire \
             E-QUERY-041 when target_table is threaded as primary_table. Got: {result:?}"
        );
    }

    // ── F-HIGH-1: Predicate::Compare non-Field-LHS HAVING path → E-QUERY-042 ──

    /// F-HIGH-1 unit test (hand-constructed AST — NOT a parser-driven test):
    ///
    /// Directly calls `check_temporal_literals` with a manually built
    /// `Ast::Sql(Select)` containing a HAVING predicate whose LHS is a
    /// `Expr::FuncCall::Aggregate(Max(timestamp))` (a non-Field expression).
    ///
    /// # Why this is a unit test, not engine.execute()
    /// This test bypasses the PrismQL parser and constructs the AST at the Rust level to
    /// isolate the `check_pred_raw_temporal` non-Field-LHS else-branch from parser behavior.
    /// The corresponding parser-driven end-to-end test is
    /// `test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_having_agg_date_only_raises_e_query_042_parser_driven`
    /// in `temporal_typing_tests.rs`, which calls
    /// `engine.execute("... HAVING max(timestamp) > '2026-06-24'", ...)` and verifies the
    /// same E-QUERY-042 NonColumnLhsComparison result end-to-end.
    ///
    /// # What this test verifies (unit level)
    /// The `check_pred_raw_temporal` Predicate non-Field-LHS else-branch MUST return
    /// E-QUERY-042 TemporalLiteralInvalidPosition::NonColumnLhsComparison
    /// (-32602 INVALID_PARAMS), NOT the pre-refinement QueryPlanFailed (-32000 INTERNAL_ERROR).
    ///
    /// ADR-052 §D4 v1.10 arm (4); BC-2.11.003; error-taxonomy.md E-QUERY-042 v2.14.
    ///
    /// Keep the existing check_expr_temporal_pos::Expr::Compare NonColumnLhsComparison
    /// test alongside this one — both Predicate and Expr paths must be covered.
    #[test]
    fn test_having_non_field_lhs_raw_temporal_fires_e_query_042_non_column_lhs_comparison() {
        use prism_core::error::TemporalLiteralPosition;

        let registry = make_registry();

        // HAVING max(timestamp) > '2026-06-24'
        // lhs: Expr::FuncCall::Aggregate (NOT a Field) → non-Field else-branch
        // rhs: RawTemporalLiteral("2026-06-24")
        let having_pred = Predicate::Compare {
            lhs: Box::new(Expr::FuncCall(FuncCall::Aggregate {
                func: AggFunc::Max(fp("timestamp")),
                args: vec![],
                distinct: false,
            })),
            rhs: Box::new(Expr::Literal(raw_lit("2026-06-24"))),
            op: CompareOp::Gt,
        };

        // SELECT count(*) FROM test_events GROUP BY hostname HAVING max(timestamp) > '2026-06-24'
        let mut sql = minimal_select("test_events");
        sql.select = SelectClause {
            distinct: false,
            items: vec![SelectItem::Expr {
                expr: Expr::FuncCall(FuncCall::Aggregate {
                    func: AggFunc::Count,
                    args: vec![],
                    distinct: false,
                }),
                alias: None,
            }],
        };
        sql.group_by = vec![Expr::Field(fp("hostname"))];
        sql.having = Some(having_pred);

        let mut ast = Ast::Sql(SqlStatement::Select(sql));

        // RED GATE (before fix): check_pred_raw_temporal non-Field else-branch returns
        // PrismError::QueryPlanFailed (pre-refinement behavior, -32000 INTERNAL_ERROR).
        //
        // GREEN (after fix): must return E-QUERY-042 TemporalLiteralInvalidPosition::
        // NonColumnLhsComparison (-32602 INVALID_PARAMS).
        let result = check_temporal_literals(&mut ast, Some(registry.as_ref()), false);
        assert!(
            matches!(
                &result,
                Err(PrismError::TemporalLiteralInvalidPosition {
                    position: TemporalLiteralPosition::NonColumnLhsComparison,
                    ..
                })
            ),
            "F-HIGH-1: HAVING max(timestamp) > '2026-06-24' must fire E-QUERY-042 \
             TemporalLiteralInvalidPosition::NonColumnLhsComparison, NOT QueryPlanFailed. \
             MCP mapping: -32602 INVALID_PARAMS (not -32000). Got: {result:?}"
        );
    }

    /// F-HIGH-1 MCP mapping verification: TemporalLiteralInvalidPosition::NonColumnLhsComparison
    /// must map to -32602 INVALID_PARAMS (not -32000 INTERNAL_ERROR).
    ///
    /// Drives prism-mcp's `map_prism_error` directly with the error that the HAVING path now
    /// returns, confirming the end-to-end MCP JSON-RPC error code is correct.
    ///
    /// This test is in prism-query rather than prism-mcp because prism-mcp is a downstream
    /// crate; instead we verify the PrismError variant maps correctly by checking the
    /// existing TemporalLiteralInvalidPosition MCP mapping contract (already tested in
    /// prism-mcp's own test suite at line ~3628) — the HAVING fix is load-bearing here.
    ///
    /// Spec: BC-2.11.003; ADR-052 §D4 v1.10; error-taxonomy.md E-QUERY-042 v2.14.
    #[test]
    fn test_having_non_field_lhs_predicate_not_query_plan_failed() {
        // Confirm the fix also applies when calling check_pred_raw_temporal directly
        // (the Predicate path, distinct from the Expr path in check_expr_temporal_pos).
        use prism_core::error::TemporalLiteralPosition;

        let registry = make_registry();

        let mut pred = Predicate::Compare {
            lhs: Box::new(Expr::FuncCall(FuncCall::Aggregate {
                func: AggFunc::Max(fp("timestamp")),
                args: vec![],
                distinct: false,
            })),
            rhs: Box::new(Expr::Literal(raw_lit("2026-06-24"))),
            op: CompareOp::Gt,
        };

        let result =
            check_pred_raw_temporal(&mut pred, Some("test_events"), Some(registry.as_ref()));

        // Must be TemporalLiteralInvalidPosition::NonColumnLhsComparison (E-QUERY-042 / -32602),
        // NOT QueryPlanFailed (pre-refinement behavior, would map to -32000).
        assert!(
            matches!(
                &result,
                Err(PrismError::TemporalLiteralInvalidPosition {
                    position: TemporalLiteralPosition::NonColumnLhsComparison,
                    ..
                })
            ),
            "F-HIGH-1 Predicate path: non-Field LHS comparison must return E-QUERY-042 \
             NonColumnLhsComparison, got: {result:?}"
        );
        // Confirm the value_prefix is populated from the raw temporal literal.
        if let Err(PrismError::TemporalLiteralInvalidPosition { value_prefix, .. }) = &result {
            assert!(
                value_prefix.contains("2026-06-24"),
                "F-HIGH-1: value_prefix must contain the raw literal substring; got: {value_prefix:?}"
            );
        }
    }
}
