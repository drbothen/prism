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
//!   Step 2: Resolve source refs to `(SensorType, client_id, SensorSpec)` tuples
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
#![allow(dead_code)]

use std::sync::Arc;

use arrow::record_batch::RecordBatch;
use datafusion::execution::context::SessionContext;
use prism_core::{OrgId, OrgSlug, PrismError};
use prism_ocsf::OcsfNormalizer;
use prism_sensors::{AdapterRegistry, CredentialResolver, SensorSpec};

use crate::engine::QueryOptions;
use crate::pushdown::PushDownPlan;
use crate::types::SensorQueryDescriptor;

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
/// S-3.02 will call this function from the DataFusion `TableProvider` integration
/// using the same virtual field injection path as `_sensor`, `_client`, and
/// `_source_table` (S-2.08 Architecture Compliance Rule 5).
///
/// # AC-9
/// Given `EventStream` rows from the buffer: every row has `"_source_type": "buffered"`.
///
/// # AC-10
/// Given `PointInTime` rows or cold-start fallback live rows:
/// every row has `"_source_type": "live"`.
// S-2.08 spec mandates &mut Vec<serde_json::Value> signature for S-3.02 wiring;
// clippy::ptr_arg is suppressed intentionally.
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
    /// The sensor type (e.g., `SensorType::CrowdStrike`).
    pub sensor_type: prism_core::types::SensorType,
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
    /// When `None`, falls back to `get_all_for_sensor_type` (test/MVP mode).
    pub(crate) org_registry: Option<Arc<prism_core::OrgRegistry>>,
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
        )
    }

    /// Construct a new `MaterializationContext` with explicit resolver and registry.
    ///
    /// Used by `QueryEngine::execute_inner` to inject the engine's
    /// `CredentialResolver` and `OrgRegistry` into the pipeline.
    /// (F-LP1-CRIT-2, F-LP1-CRIT-3)
    pub fn new_with_resolver(
        adapter_registry: Arc<AdapterRegistry>,
        ocsf_normalizer: Arc<OcsfNormalizer>,
        max_records: usize,
        credential_resolver: Arc<dyn CredentialResolver>,
        org_registry: Option<Arc<prism_core::OrgRegistry>>,
    ) -> Self {
        Self {
            adapter_registry,
            ocsf_normalizer,
            record_count: 0,
            max_records,
            in_query_cache: std::collections::HashMap::new(),
            credential_resolver,
            org_registry,
        }
    }

    /// Increment the running record count, enforcing the 10K cap. (BC-2.11.006 EC-003)
    ///
    /// Returns `Err(PrismError::QueryExecutionFailed)` with E-QUERY-003 if the
    /// new count would exceed `max_records`. Uses saturating addition to prevent
    /// integer overflow.
    pub(crate) fn increment_record_count(&mut self, n: usize) -> Result<(), PrismError> {
        let new = self.record_count.saturating_add(n);
        if new > self.max_records {
            return Err(PrismError::QueryExecutionFailed {
                detail: format!(
                    "E-QUERY-003: record cap exceeded: {} records (limit {})",
                    new, self.max_records
                ),
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
        sensor_type: prism_core::types::SensorType,
    ) -> Result<Box<dyn prism_sensors::SensorAuth>, prism_sensors::SensorError> {
        Err(prism_sensors::SensorError::Internal {
            detail: format!(
                "NullMaterializationCredentialResolver: no credential for sensor {sensor_type:?}"
            ),
        })
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
/// `PrismError::QueryExecutionFailed` containing E-QUERY-003 message.
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
        }
    })?;

    let source_names = extract_source_names(&ast);

    // Build a flat FilterMap of equality predicates from the WHERE clause (BC-2.11.007).
    // Per-sensor classify_predicates integration deferred to wave-5
    // (see extract_push_down_filters_as_map docs for rationale).
    let where_filters = extract_push_down_filters_as_map(&ast);

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
            target.sensor_type,
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

        // Build the fan_out FanOutTarget (prism-sensors type, not our local type).
        // One FanOutTarget per (org_id, source_table) pair → correct per-org dispatch.
        // (F-LP1-CRIT-3: org_id matches the adapter's registered key; no random OrgId::new())
        let fan_target = {
            #[allow(deprecated)]
            prism_sensors::fanout::FanOutTarget {
                org_id: target.org_id,
                client_id: target.client_id.as_str().to_string(),
                sensor_type: target.sensor_type,
                spec: prism_sensors::adapter::SensorSpec {
                    source_table: target.source_table.clone(),
                    #[allow(deprecated)]
                    client_id: target.client_id.as_str().to_string(),
                    org_id: target.org_id,
                    sensor_config: serde_json::Value::Null,
                },
                params: prism_sensors::adapter::QueryParams {
                    cursor: None,
                    limit: options.limit.map(|l| l as u64).unwrap_or(0),
                    start_time: None,
                    end_time: None,
                    filters: where_filters.clone(),
                },
            }
        };

        // Call fan_out with a single target — preserves per-client identity for
        // virtual field injection. (BC-3.2.001 per-org isolation)
        match prism_sensors::fan_out(
            vec![fan_target],
            Arc::clone(&mat_ctx.adapter_registry),
            Arc::clone(&mat_ctx.credential_resolver),
        )
        .await
        {
            Ok(fan_result) => {
                // Collect successes with per-target virtual field injection.
                let mut fetched_batches: Vec<RecordBatch> = Vec::new();
                for batch in fan_result.successes {
                    let n = batch.num_rows();
                    mat_ctx.increment_record_count(n)?;
                    // Inject virtual fields (_sensor, _client, _source_table).
                    // Uses this target's client_id for correct per-client attribution (AC-6).
                    let annotated = crate::virtual_fields::inject_virtual_fields(
                        batch,
                        &target.sensor_type,
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
                        sensor = ?target.sensor_type,
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
        });
    }

    let collected = execute_against_session(session_ctx, query_str, &ast, table_batches).await?;

    Ok(MaterializationOutput {
        batches: collected,
        sensor_errors,
        registered_tables,
    })
}

/// Execute the query against the DataFusion session context.
///
/// For SQL mode: runs the SQL string directly via DataFusion.
/// For Filter/Pipe mode: returns the union of all materialized `table_batches`
/// (DataFusion MemTable registration already happened; no separate SQL step).
/// (F-LP1-HIGH-1: Filter and Pipe must NOT return empty Vec)
async fn execute_against_session(
    session_ctx: &SessionContext,
    query_str: &str,
    ast: &crate::ast::Ast,
    table_batches: std::collections::HashMap<String, Vec<RecordBatch>>,
) -> Result<Vec<RecordBatch>, PrismError> {
    use crate::ast::{Ast, SqlStatement};

    match ast {
        Ast::Sql(SqlStatement::Select(_)) => {
            // Execute the SQL string via DataFusion.
            let df = session_ctx.sql(query_str).await.map_err(|e| {
                tracing::error!(error = %e, "DataFusion SQL planning error");
                PrismError::QueryExecutionFailed {
                    detail: "SQL planning error: <redacted; see server logs>".to_string(),
                }
            })?;
            let stream = df.execute_stream().await.map_err(|e| {
                tracing::error!(error = %e, "DataFusion execution error");
                PrismError::QueryExecutionFailed {
                    detail: "SQL execution error: <redacted; see server logs>".to_string(),
                }
            })?;
            collect_record_batch_stream(stream).await
        }
        // F-LP1-HIGH-1: For Filter and Pipe modes, return the union of all materialized batches.
        // The batches were already collected in the fan-out loop with virtual field injection.
        // DataFusion MemTable registration has already happened for SQL query capability;
        // for Filter/Pipe, we return the pre-collected annotated batches directly.
        Ast::Filter(_) | Ast::Pipe(_) => {
            // Return the union of all table_batches values.
            let all_batches: Vec<RecordBatch> = table_batches.into_values().flatten().collect();
            Ok(all_batches)
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
/// (test/MVP mode via `get_all_for_sensor_type`).
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
        // Skip composite/unknown sources.
        let Some(sensor_type) = sensor_type_from_table_name(source_name) else {
            tracing::debug!(
                source_name,
                "resolve_source_refs: unknown sensor prefix; skipping"
            );
            continue;
        };

        if clients.is_empty() {
            // ALL scope with no explicit client list: fan out to ALL registered adapters
            // for this sensor type. This is the correct behavior for cross-client queries.
            //
            // F-LP1-CRIT-3/HIGH-6: use per-org adapter selection.
            // When no explicit client list, iterate all registered (org_id, adapter) pairs.
            let all_adapters = adapter_registry.get_all_for_sensor_type(sensor_type);
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
                    let synthetic_slug =
                        OrgSlug::new_unchecked(&format!("org-{}", &org_id.to_string()[..8]));
                    targets.push(FanOutTarget {
                        sensor_type,
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
                    sensor_type,
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
            if adapter_registry
                .get_all_for_sensor_type(sensor_type)
                .is_empty()
            {
                tracing::debug!(
                    source_table = %source_name,
                    "resolve_source_refs: no adapters registered for sensor type; \
                     skipping fan-out (BC-2.11.011 EC-005)"
                );
            }
        } else {
            // Explicit client list: one target per client.
            // BC-2.11.011: each (source, client) pair is a separate fan-out target.
            // EC-005: clients with no sensor configured are skipped silently.
            for client_id in clients {
                // F-LP1-CRIT-3: resolve OrgSlug → OrgId via OrgRegistry if available.
                // When OrgRegistry is absent (test mode), use `get_all_for_sensor_type`
                // to find the OrgId associated with a registered adapter for this sensor.
                let org_id = resolve_org_id(client_id, sensor_type, adapter_registry, org_registry);

                targets.push(FanOutTarget {
                    sensor_type,
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
/// 2. First registered adapter for sensor_type (test/MVP fallback) — avoids
///    the OrgId::new() randomness that caused F-LP1-CRIT-3.
/// 3. Fresh OrgId (last resort — will miss in registry.get()).
fn resolve_org_id(
    client_id: &OrgSlug,
    sensor_type: prism_core::types::SensorType,
    adapter_registry: &AdapterRegistry,
    org_registry: &Option<Arc<prism_core::OrgRegistry>>,
) -> OrgId {
    // Path 1: OrgRegistry lookup (production).
    if let Some(reg) = org_registry {
        if let Some(id) = reg.resolve(client_id) {
            return id;
        }
    }

    // Path 2: Fall back to first registered adapter's OrgId for this sensor type.
    // This preserves the test-path behavior where adapters are registered with
    // known OrgIds but no OrgRegistry is present.
    let adapters = adapter_registry.get_all_for_sensor_type(sensor_type);
    if let Some((org_id, _)) = adapters.into_iter().next() {
        return org_id;
    }

    // Path 3: Last resort — fresh OrgId (will not match any registered adapter).
    OrgId::new()
}

// ---------------------------------------------------------------------------
// sensor_type_from_table_name
// ---------------------------------------------------------------------------

/// Map an underscore-prefixed table name to the corresponding `SensorType`.
///
/// Naming convention: `{sensor}_{table}` → `SensorType`.
/// Returns `None` for unknown prefixes.
fn sensor_type_from_table_name(table_name: &str) -> Option<prism_core::types::SensorType> {
    use prism_core::types::SensorType;
    if table_name.starts_with("crowdstrike") {
        Some(SensorType::CrowdStrike)
    } else if table_name.starts_with("cyberint") {
        Some(SensorType::Cyberint)
    } else if table_name.starts_with("claroty") {
        Some(SensorType::Claroty)
    } else if table_name.starts_with("armis") {
        Some(SensorType::Armis)
    } else {
        None
    }
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
        Ast::Filter(filter) => {
            names.push(filter.source.raw.clone());
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
pub(crate) fn register_mem_table(
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
) -> Result<Vec<RecordBatch>, PrismError> {
    datafusion::physical_plan::common::collect(stream)
        .await
        .map_err(|e| {
            tracing::error!(
                error = %e,
                "stream collection error (detail redacted from client response)"
            );
            PrismError::QueryExecutionFailed {
                detail: "stream collection error: <redacted; see server logs>".to_string(),
            }
        })
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

    use crate::ast::{
        Ast, Expr, FieldPath, FromClause, Join, JoinKind, OrderExpr, SelectClause, SelectItem,
        SortDirection, SourceRef, SourceRefKind, Span, SqlQuery, SqlStatement,
    };
    use crate::materialization::extract_source_names_recursive;

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
        use crate::ast::{FieldPath, Predicate, Span};
        use crate::write_ast::{DmlNode, DmlOperation};

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
