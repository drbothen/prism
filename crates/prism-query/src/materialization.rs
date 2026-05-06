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

// S-3.02 stub functions: dead_code suppressed for stub phase (BC-5.38.001).
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
// SourceRef
// ---------------------------------------------------------------------------

/// A fully-resolved source reference for fan-out.
///
/// Produced by `resolve_source_refs` (Step 2 of the pipeline). Carries all
/// information needed to construct a `FanOutTarget`. (BC-2.11.005)
#[derive(Debug, Clone)]
pub struct SourceRef {
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
    pub adapter_registry: Arc<AdapterRegistry>,
    /// OCSF normalizer for raw JSON → Arrow RecordBatch conversion.
    pub ocsf_normalizer: Arc<OcsfNormalizer>,
    /// Running record count across all sources (10K cap enforcer). (BC-2.11.006)
    pub record_count: usize,
    /// Maximum records before aborting materialization. (BC-2.11.006)
    pub max_records: usize,
    /// Per-query in-query cache: avoids redundant API calls for self-joins.
    /// Key: canonical cache key string. Value: collected batches. (BC-2.11.005)
    pub in_query_cache: std::collections::HashMap<String, Vec<RecordBatch>>,
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
}

// ---------------------------------------------------------------------------
// run_materialization_pipeline
// ---------------------------------------------------------------------------

/// Execute the full 8-step ephemeral materialization pipeline.
///
/// # Steps (BC-2.11.005)
/// 1. Parse PrismQL string via `PrismQlParser::parse` — public API only
/// 2. Resolve source refs to `SourceRef` tuples
/// 3. Fan out to sensor adapters via `fan_out()` — all sources in parallel
/// 4. Normalize each response via `OcsfNormalizer` → `Vec<RecordBatch>`
/// 5. Inject virtual field columns (`_sensor`, `_client`, `_source_table`)
/// 6. Register each source as a DataFusion `MemTable` in `ctx`
/// 7. Execute the SQL plan against the registered MemTables
/// 8. Collect `SendableRecordBatchStream` → `Vec<RecordBatch>`
///
/// # Record Cap (BC-2.11.006, EC-003)
/// Streaming counter across all sources. If `mat_ctx.record_count` exceeds
/// `mat_ctx.max_records` during Step 3, abort with
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
pub async fn run_materialization_pipeline(
    _query_str: &str,
    _options: &QueryOptions,
    _mat_ctx: &mut MaterializationContext,
    _session_ctx: &SessionContext,
) -> Result<Vec<RecordBatch>, PrismError> {
    todo!("S-3.02 — run_materialization_pipeline")
}

// ---------------------------------------------------------------------------
// resolve_source_refs
// ---------------------------------------------------------------------------

/// Step 2: Resolve PrismQL source references to `SourceRef` tuples.
///
/// Each source reference in the AST (e.g., `crowdstrike.detections`) is
/// resolved against the sensor specs and the provided client scope to produce
/// one `SourceRef` per `(source, client)` combination. (BC-2.11.005)
///
/// # BC-2.11.011
/// If a client in `clients` does not have a sensor for the source, the
/// `(source, client)` pair is silently skipped (listed in metadata as
/// `clients_skipped`).
pub(crate) async fn resolve_source_refs(
    _source_names: &[String],
    _clients: &[OrgSlug],
    _adapter_registry: &AdapterRegistry,
) -> Result<Vec<SourceRef>, PrismError> {
    todo!("S-3.02 — resolve_source_refs")
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
    _ctx: &SessionContext,
    _table_name: &str,
    _batches: Vec<RecordBatch>,
) -> Result<(), PrismError> {
    todo!("S-3.02 — register_mem_table")
}

// ---------------------------------------------------------------------------
// collect_record_batch_stream
// ---------------------------------------------------------------------------

/// Step 8: Collect a DataFusion `SendableRecordBatchStream` to `Vec<RecordBatch>`.
///
/// Drains the stream until exhausted. Returns all collected batches.
/// The `SessionScope` is still live during collection; it is dropped after
/// this function returns (or on error). (BC-2.11.005)
pub(crate) async fn collect_record_batch_stream(
    _stream: datafusion::physical_plan::SendableRecordBatchStream,
) -> Result<Vec<RecordBatch>, PrismError> {
    todo!("S-3.02 — collect_record_batch_stream")
}
