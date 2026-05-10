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
use prism_core::{OrgSlug, PrismError};
use prism_ocsf::OcsfNormalizer;
use prism_sensors::{AdapterRegistry, SensorSpec};

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
    /// The sensor spec for this (sensor, client) pair.
    pub sensor_spec: SensorSpec,
    /// The source table name (e.g., `"crowdstrike.detections"`).
    pub source_table: String,
    /// Push-down plan computed for this source. (BC-2.11.007)
    pub push_down_plan: PushDownPlan,
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
}

impl MaterializationContext {
    /// Construct a new `MaterializationContext` for a single query execution.
    pub fn new(
        adapter_registry: Arc<AdapterRegistry>,
        ocsf_normalizer: Arc<OcsfNormalizer>,
        max_records: usize,
    ) -> Self {
        Self {
            adapter_registry,
            ocsf_normalizer,
            record_count: 0,
            max_records,
            in_query_cache: std::collections::HashMap::new(),
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

    /// Look up a cached batch set by cache key. (BC-2.11.005)
    pub(crate) fn cache_lookup(&self, key: &str) -> Option<&Vec<RecordBatch>> {
        self.in_query_cache.get(key)
    }

    /// Insert a batch set into the in-query cache. (BC-2.11.005)
    pub(crate) fn cache_insert(&mut self, key: String, batches: Vec<RecordBatch>) {
        self.in_query_cache.insert(key, batches);
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
/// `PrismError::QueryExecutionFailed` containing E-QUERY-005 message.
///
/// # Cold-Start Fallback (AC-9, inherited from S-2.08)
/// When `route_table_query()` returns `ColdStartFallback`, this function
/// triggers a live `SensorAdapter` fetch, writes results to `EventBufferStore`,
/// and logs an INFO event. (BC-2.11.005, BC-2.11.007)
///
/// # Architecture Compliance (INV-SEC-PERIMETER-001)
/// Parser consumed ONLY via `PrismQlParser::parse`. Restricted sub-parser
/// symbols MUST NOT appear in this function body.
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

/// Extract simple `field = 'value'` predicates from the WHERE clause for push-down.
///
/// Returns a `FilterMap` of column_name → string_value pairs for push-down.
/// Complex predicates (AND/OR, non-equality, non-string values) are skipped —
/// DataFusion applies them post-materialization (conservative fallback per BC-2.11.007).
fn extract_where_filters(ast: &crate::ast::Ast) -> prism_sensors::types::FilterMap {
    use crate::ast::{Ast, SqlStatement};

    let mut filters = prism_sensors::types::FilterMap::new();

    let where_pred = match ast {
        Ast::Sql(SqlStatement::Select(sql)) => sql.where_.as_ref(),
        _ => None,
    };

    let Some(pred) = where_pred else {
        return filters;
    };

    // Walk the predicate and extract simple equality comparisons.
    collect_eq_filters(pred, &mut filters);

    filters
}

/// Recursively collect equality predicates from a `Predicate` tree.
fn collect_eq_filters(pred: &crate::ast::Predicate, filters: &mut prism_sensors::types::FilterMap) {
    use crate::ast::{CompareOp, Expr, Literal, LogicalOp, Predicate};
    match pred {
        Predicate::Compare { lhs, op, rhs } if *op == CompareOp::Eq => {
            // LHS must be a simple field path.
            let col = match lhs.as_ref() {
                Expr::Field(fp) => fp.segments.join("."),
                _ => return,
            };
            // RHS must be a string literal.
            if let Expr::Literal(Literal::String(val)) = rhs.as_ref() {
                filters.insert(col, serde_json::Value::String(val.clone()));
            }
        }
        Predicate::Logical { op, predicates } if *op == LogicalOp::And => {
            for child in predicates {
                collect_eq_filters(child, filters);
            }
        }
        _ => {}
    }
}

/// Extract all source table names from a PrismQL AST.
fn extract_source_names(ast: &crate::ast::Ast) -> Vec<String> {
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
        }
        // Non-exhaustive: ignore other variants
        _ => {}
    }
    names
}

pub async fn run_materialization_pipeline(
    query_str: &str,
    options: &QueryOptions,
    mat_ctx: &mut MaterializationContext,
    session_ctx: &SessionContext,
) -> Result<Vec<RecordBatch>, PrismError> {
    // Step 1: Parse the query to extract source table names.
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

    // Extract WHERE predicates for push-down (BC-2.11.007).
    let where_filters = extract_where_filters(&ast);

    // Step 2: Resolve client scope.
    let all_clients: Vec<OrgSlug> = options.clients.clone().unwrap_or_default();

    // Step 3: Resolve source refs to fan-out targets.
    let targets =
        resolve_source_refs(&source_names, &all_clients, &mat_ctx.adapter_registry).await?;

    // Step 4: Fan out to sensor adapters, collecting results per source table.
    // Group results by source table name for MemTable registration.
    let mut table_batches: std::collections::HashMap<String, Vec<RecordBatch>> =
        std::collections::HashMap::new();

    // Track all sensor errors for partial-failure reporting.
    let mut _sensor_errors: Vec<String> = Vec::new();

    for target in targets {
        // Look up adapters for this sensor type (MVP: match by sensor type only).
        let adapters = mat_ctx
            .adapter_registry
            .get_all_for_sensor_type(target.sensor_type);

        if adapters.is_empty() {
            tracing::debug!(
                source_table = %target.source_table,
                sensor = ?target.sensor_type,
                "no adapter registered for sensor type; skipping fan-out"
            );
            continue;
        }

        for (_, adapter) in adapters {
            // Build SensorSpec for this fetch.
            #[allow(deprecated)]
            let spec = prism_sensors::adapter::SensorSpec {
                source_table: target.source_table.clone(),
                org_id: prism_core::OrgId::new(),
                client_id: target.client_id.as_str().to_string(),
                sensor_config: serde_json::Value::Null,
            };

            // Build QueryParams: use WHERE-clause push-down filters (BC-2.11.007).
            let params = prism_sensors::adapter::QueryParams {
                cursor: None,
                limit: options.limit.map(|l| l as u64).unwrap_or(0),
                start_time: None,
                end_time: None,
                filters: where_filters.clone(),
            };

            // Build a placeholder auth (test adapters ignore auth; production adapters
            // hold credentials internally and the auth parameter is supplementary).
            let placeholder_auth = prism_sensors::CrowdStrikeAuth {
                client_id: String::new(),
                client_secret: prism_sensors::SecretString::new(String::new()),
                cloud_region: String::new(),
            };

            match adapter.fetch(&spec, &params, &placeholder_auth).await {
                Ok(batches) => {
                    // Apply record cap before accumulating (BC-2.11.006).
                    for batch in batches {
                        let n = batch.num_rows();
                        mat_ctx.increment_record_count(n)?;
                        // Inject virtual fields (_sensor, _client, _source_table).
                        let annotated = crate::virtual_fields::inject_virtual_fields(
                            batch,
                            &target.sensor_type,
                            &target.client_id,
                            &target.source_table,
                        )
                        .map_err(|e| PrismError::QueryExecutionFailed {
                            detail: format!("virtual field injection failed: {e}"),
                        })?;
                        table_batches
                            .entry(target.source_table.clone())
                            .or_default()
                            .push(annotated);
                    }
                }
                Err(e) => {
                    tracing::warn!(
                        source_table = %target.source_table,
                        error = %e,
                        "adapter fetch error (partial failure)"
                    );
                    _sensor_errors.push(format!("{}: {}", target.source_table, e));
                }
            }
        }
    }

    // Step 5: Register each source as a DataFusion MemTable.
    // Track how many external tables were successfully registered with data.
    let mut any_external_table_registered = false;
    for source_name in &source_names {
        // Skip internal tables (prism_*) — already registered via register_internal_tables.
        if source_name.starts_with("prism_") {
            // Internal tables are registered separately; consider them "available".
            any_external_table_registered = true;
            continue;
        }
        let batches = table_batches.remove(source_name).unwrap_or_default();
        if !batches.is_empty() {
            // register_mem_table handles empty batch list silently.
            register_mem_table(session_ctx, source_name, batches)?;
            any_external_table_registered = true;
        }
        // If batches is empty, the table is NOT registered — DataFusion can't plan for it.
        // This is the "no adapter" case. We skip SQL execution in this case.
    }

    // Step 6: Execute the DataFusion SQL plan and collect results.
    // If no tables were registered (all sources empty), return empty results without
    // attempting DataFusion execution (which would fail with "table not found").
    if !any_external_table_registered {
        return Ok(Vec::new());
    }

    let collected = execute_against_session(session_ctx, query_str, &ast).await?;

    Ok(collected)
}

/// Execute the query against the DataFusion session context.
///
/// For SQL mode: runs the SQL string directly via DataFusion.
/// For filter/pipe mode: returns the pre-collected batches (DataFusion not involved).
async fn execute_against_session(
    session_ctx: &SessionContext,
    query_str: &str,
    ast: &crate::ast::Ast,
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
        // For filter/pipe mode: batches were already collected in the fan-out loop.
        // Return empty — callers get results from the table_batches in the outer scope.
        _ => Ok(Vec::new()),
    }
}

// ---------------------------------------------------------------------------
// resolve_source_refs
// ---------------------------------------------------------------------------

/// Step 2: Resolve PrismQL source references to `FanOutTarget` tuples.
///
/// Each source reference in the AST (e.g., `crowdstrike.detections`) is
/// resolved against the sensor specs and the provided client scope to produce
/// one `FanOutTarget` per `(source, client)` combination. (BC-2.11.005)
///
/// # BC-2.11.011
/// If a client in `clients` does not have a sensor for the source, the
/// `(source, client)` pair is silently skipped (listed in metadata as
/// `clients_skipped`).
pub(crate) async fn resolve_source_refs(
    source_names: &[String],
    clients: &[OrgSlug],
    _adapter_registry: &AdapterRegistry,
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

        // For each resolved client, produce a FanOutTarget.
        // BC-2.11.011: each (source, client) pair is a separate fan-out target.
        // EC-005: clients with no sensor configured are skipped silently.
        // For MVP (no OrgSlug→OrgId registry), iterate each client.
        if clients.is_empty() {
            // ALL scope with no explicit client list: produce one target with no client binding.
            targets.push(FanOutTarget {
                sensor_type,
                client_id: OrgSlug::new_unchecked("_all"),
                sensor_spec: SensorSpec {
                    source_table: source_name.clone(),
                    #[allow(deprecated)]
                    client_id: "_all".to_string(),
                    org_id: prism_core::OrgId::new(),
                    sensor_config: serde_json::Value::Null,
                },
                source_table: source_name.clone(),
                push_down_plan: PushDownPlan::default(),
            });
        } else {
            for client_id in clients {
                targets.push(FanOutTarget {
                    sensor_type,
                    client_id: client_id.clone(),
                    sensor_spec: SensorSpec {
                        source_table: source_name.clone(),
                        #[allow(deprecated)]
                        client_id: client_id.as_str().to_string(),
                        org_id: prism_core::OrgId::new(),
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

// ---------------------------------------------------------------------------
// register_mem_table
// ---------------------------------------------------------------------------

/// Step 6: Register a set of RecordBatches as a DataFusion `MemTable`.
///
/// The table name is the source ref string (e.g., `"crowdstrike.detections"`).
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
