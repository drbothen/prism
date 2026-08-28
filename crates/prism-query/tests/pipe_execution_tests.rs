//! Pipe execution engine integration tests (ENRICH-4-B).
//!
//! Verifies that `Ast::Pipe` queries execute through the DataFusion SQL-lowering
//! path rather than returning raw unioned rows.
//!
//! Traces to: BC-2.11.004 (Pipe Mode), BC-2.19.001 (Infusion UDFs),
//!            BC-2.11.006 (Security Limits)
//!
//! Design doc: `.factory/specs/architecture/scoping/pipe-execution-engine-design.md`
//!
//! # Test approach
//!
//! Tests 2-6 use `run_materialization_pipeline` with a `FixedBatchAdapter` that returns
//! known data and a pipe-syntax query string (full pipeline path).
//!
//! Tests 1, 7, 8 use `execute_against_session` directly (pre-registered UDF + MemTable)
//! because they need custom async UDF registration that isn't wired through
//! `InfusionRegistry` for test purposes.  `execute_against_session` is exposed `pub`
//! to enable this integration test pattern (same rationale as `register_mem_table`).
//!
//! # Red Gate behaviour
//!
//! Before ENRICH-4-B, the combined `Ast::Filter | Ast::Pipe` arm returns
//! `table_batches.into_values().flatten().collect()` — raw rows, no DataFusion
//! execution.  All 8 tests assert on the CORRECT post-implementation result;
//! they fail on the pre-implementation raw-return path.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    non_snake_case,
    unused_imports
)]

use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use arrow::array::{Array, Int64Array, StringArray};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use async_trait::async_trait;
use prism_core::{OrgId, OrgSlug, PrismError, SensorId};
use prism_ocsf::OcsfNormalizer;
use prism_query::{
    engine::QueryOptions,
    materialization::{
        execute_against_session, register_mem_table, run_materialization_pipeline,
        MaterializationContext,
    },
    memory::{build_session_context, QUERY_MEMORY_POOL_BYTES},
};
use prism_sensors::{
    adapter::{FetchOutput, QueryParams, SensorAdapter, SensorError, SensorSpec},
    auth::SensorAuth,
    AdapterRegistry, CredentialResolver,
};

// ---------------------------------------------------------------------------
// FixedBatchAdapter: returns caller-supplied RecordBatches
// ---------------------------------------------------------------------------

struct FixedBatchAdapter {
    sensor_id: SensorId,
    batches: Vec<RecordBatch>,
}

impl std::fmt::Debug for FixedBatchAdapter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FixedBatchAdapter")
            .field("sensor_id", &self.sensor_id)
            .finish()
    }
}

#[async_trait]
impl SensorAdapter for FixedBatchAdapter {
    fn sensor_type(&self) -> SensorId {
        self.sensor_id.clone()
    }
    fn sensor_name(&self) -> &'static str {
        "test_sensor"
    }

    async fn fetch(
        &self,
        _spec: &SensorSpec,
        _params: &QueryParams,
        _auth: &dyn SensorAuth,
    ) -> Result<FetchOutput, SensorError> {
        Ok(FetchOutput::new(self.batches.clone(), false))
    }
}

// ---------------------------------------------------------------------------
// StubCredentialResolver
// ---------------------------------------------------------------------------

struct StubCredentialResolver;

impl CredentialResolver for StubCredentialResolver {
    fn resolve(
        &self,
        _client_id: &str,
        _sensor_id: SensorId,
    ) -> Result<Box<dyn SensorAuth>, prism_sensors::SensorError> {
        struct StubAuth;
        impl SensorAuth for StubAuth {
            fn as_any(&self) -> &dyn std::any::Any {
                self
            }
            fn auth_type_name(&self) -> &'static str {
                "custom_via_plugin"
            }
        }
        Ok(Box::new(StubAuth))
    }
}

// ---------------------------------------------------------------------------
// Pipeline helpers
// ---------------------------------------------------------------------------

/// Construct table name "{sensor}_{suffix}" matching the materializer convention.
fn tbl(sensor: &str, suffix: &str) -> String {
    format!("{}_{}", sensor, suffix)
}

/// Build a `MaterializationContext` with a single `FixedBatchAdapter` for `sensor`.
fn mat_ctx_with(sensor: &str, batches: Vec<RecordBatch>) -> MaterializationContext {
    let sensor_id = SensorId::from(sensor);
    let org_id = OrgId::new();
    let mut registry = AdapterRegistry::new();
    registry.register(org_id, Arc::new(FixedBatchAdapter { sensor_id, batches }));
    MaterializationContext::new_with_resolver(
        Arc::new(registry),
        Arc::new(OcsfNormalizer::new()),
        prism_query::memory::MAX_MATERIALIZED_RECORDS,
        Arc::new(StubCredentialResolver),
        None,
        None,
    )
}

/// Default `QueryOptions` for pipe tests: one org, no extra constraints.
fn query_opts() -> QueryOptions {
    QueryOptions {
        clients: Some(vec![OrgSlug::new_unchecked("test-org")]),
        sensors: None,
        limit: None,
        force_refresh: false,
        ..QueryOptions::default()
    }
}

// ---------------------------------------------------------------------------
// Batch helpers
// ---------------------------------------------------------------------------

fn make_string_batch(col: &str, values: &[&str]) -> RecordBatch {
    let schema = Arc::new(Schema::new(vec![Field::new(col, DataType::Utf8, true)]));
    let array = Arc::new(StringArray::from(values.to_vec())) as Arc<dyn Array>;
    RecordBatch::try_new(schema, vec![array]).expect("make_string_batch must succeed")
}

fn make_string_int_batch(sc: &str, sv: &[&str], ic: &str, iv: &[i64]) -> RecordBatch {
    assert_eq!(sv.len(), iv.len());
    let schema = Arc::new(Schema::new(vec![
        Field::new(sc, DataType::Utf8, true),
        Field::new(ic, DataType::Int64, true),
    ]));
    let sa = Arc::new(StringArray::from(sv.to_vec())) as Arc<dyn Array>;
    let ia = Arc::new(Int64Array::from(iv.to_vec())) as Arc<dyn Array>;
    RecordBatch::try_new(schema, vec![sa, ia]).expect("make_string_int_batch must succeed")
}

fn total_rows(batches: &[RecordBatch]) -> usize {
    batches.iter().map(|b| b.num_rows()).sum()
}

fn str_col(batches: &[RecordBatch], col: &str) -> Vec<String> {
    let mut out = Vec::new();
    for b in batches {
        let idx = b.schema().index_of(col).expect("column must exist");
        let arr = b
            .column(idx)
            .as_any()
            .downcast_ref::<StringArray>()
            .expect("StringArray");
        for i in 0..arr.len() {
            out.push(arr.value(i).to_string());
        }
    }
    out
}

fn int_col(batches: &[RecordBatch], col: &str) -> Vec<i64> {
    let mut out = Vec::new();
    for b in batches {
        let idx = b.schema().index_of(col).expect("column must exist");
        let arr = b
            .column(idx)
            .as_any()
            .downcast_ref::<Int64Array>()
            .expect("Int64Array");
        for i in 0..arr.len() {
            out.push(arr.value(i));
        }
    }
    out
}

// ---------------------------------------------------------------------------
// Mini async-UDF builder for tests 1, 7, 8
// ---------------------------------------------------------------------------
//
// These tests call `execute_against_session` directly (the function is exposed
// `pub` by ENRICH-4-B) with a pre-configured SessionContext that has:
//   - UDF registered via `ctx.register_udf(AsyncScalarUDF::new(...).into_scalar_udf())`
//   - MemTable registered via `register_mem_table`
//
// This avoids threading through `InfusionRegistry` for test-only UDFs.

use datafusion::common::not_impl_err;
use datafusion::error::Result as DataFusionResult;
use datafusion::logical_expr::async_udf::{AsyncScalarUDF, AsyncScalarUDFImpl};
use datafusion::logical_expr::{
    ColumnarValue, ScalarFunctionArgs, ScalarUDFImpl, Signature, TypeSignature, Volatility,
};
use std::hash::{Hash, Hasher};

const TEST_SENTINEL: &str = "ENRICH-4B-SENTINEL";

/// Minimal async UDF for enrich-stage tests.
/// Returns `TEST_SENTINEL` for all rows, except `null_for` which returns NULL.
#[derive(Debug)]
struct TestAsyncUdf {
    udf_name: String,
    call_count: Arc<AtomicUsize>,
    null_for: Option<String>,
}

impl PartialEq for TestAsyncUdf {
    fn eq(&self, other: &Self) -> bool {
        self.udf_name == other.udf_name
    }
}
impl Eq for TestAsyncUdf {}
impl Hash for TestAsyncUdf {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.udf_name.hash(state);
    }
}

impl ScalarUDFImpl for TestAsyncUdf {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn name(&self) -> &str {
        &self.udf_name
    }
    fn signature(&self) -> &Signature {
        Box::leak(Box::new(Signature::new(
            TypeSignature::Exact(vec![DataType::Utf8]),
            Volatility::Volatile,
        )))
    }
    fn return_type(&self, _: &[DataType]) -> DataFusionResult<DataType> {
        Ok(DataType::Utf8)
    }
    fn invoke_with_args(&self, _: ScalarFunctionArgs) -> DataFusionResult<ColumnarValue> {
        not_impl_err!("TestAsyncUdf: use async path")
    }
}

#[async_trait]
impl AsyncScalarUDFImpl for TestAsyncUdf {
    async fn invoke_async_with_args(
        &self,
        args: ScalarFunctionArgs,
    ) -> DataFusionResult<ColumnarValue> {
        self.call_count.fetch_add(1, Ordering::SeqCst);

        let results: Vec<Option<&str>> = if let Some(null_str) = &self.null_for {
            let n = args
                .args
                .first()
                .map(|c| match c {
                    ColumnarValue::Array(a) => a.len(),
                    ColumnarValue::Scalar(_) => 1,
                })
                .unwrap_or(1);

            args.args
                .first()
                .and_then(|c| match c {
                    ColumnarValue::Array(arr) => {
                        let sa = arr.as_any().downcast_ref::<StringArray>()?;
                        Some(
                            (0..sa.len())
                                .map(|i| {
                                    if sa.value(i) == null_str.as_str() {
                                        None
                                    } else {
                                        Some(TEST_SENTINEL)
                                    }
                                })
                                .collect::<Vec<_>>(),
                        )
                    }
                    _ => None,
                })
                .unwrap_or_else(|| vec![Some(TEST_SENTINEL); n])
        } else {
            let n = args
                .args
                .first()
                .map(|c| match c {
                    ColumnarValue::Array(a) => a.len(),
                    ColumnarValue::Scalar(_) => 1,
                })
                .unwrap_or(1);
            vec![Some(TEST_SENTINEL); n]
        };

        Ok(ColumnarValue::Array(Arc::new(StringArray::from(results))))
    }
}

fn make_udf_ctx(
    udf_name: &str,
    table: &str,
    batches: Vec<RecordBatch>,
    null_for: Option<String>,
) -> (
    datafusion::execution::context::SessionContext,
    Arc<AtomicUsize>,
) {
    let ctx =
        build_session_context(QUERY_MEMORY_POOL_BYTES).expect("build_session_context must succeed");
    let call_count = Arc::new(AtomicUsize::new(0));
    let udf = TestAsyncUdf {
        udf_name: udf_name.to_string(),
        call_count: Arc::clone(&call_count),
        null_for,
    };
    ctx.register_udf(AsyncScalarUDF::new(Arc::new(udf)).into_scalar_udf());
    register_mem_table(&ctx, table, batches).expect("register_mem_table must succeed");
    (ctx, call_count)
}

// ===========================================================================
// Test 1: enrich stage invokes a registered UDF and adds the enriched column
// ===========================================================================

/// ENRICH-4-B §10.1 Test 1
///
/// `FROM test_e1_tbl | enrich test_e1_udf(ip)` executes through DataFusion:
/// - The UDF call counter increments > 0 (not a silent no-op).
/// - Output contains the enriched column with the sentinel value.
///
/// Red Gate: `Ast::Pipe` arm returns raw `table_batches` — no DataFusion execution,
///   call_count stays 0, no enriched column.
/// Green: `Ast::Pipe` arm generates `SELECT *, test_e1_udf(ip) AS test_e1_udf FROM …`
///   and routes through `session_ctx.sql()`.
#[tokio::test]
async fn test_pipe_enrich_stage_invokes_registered_udf() {
    let table = "test_e1_tbl";
    let udf_name = "test_e1_udf";
    let batch = make_string_batch("ip", &["192.168.1.1", "10.0.0.1"]);
    let (ctx, call_count) = make_udf_ctx(udf_name, table, vec![batch], None);

    use prism_query::ast::{Ast, EnrichStage, FieldPath, PipeQuery, PipeStage, SourceRef};
    let ast = Ast::Pipe(PipeQuery::new(
        SourceRef::from_raw(table),
        vec![PipeStage::Enrich(EnrichStage::new(
            udf_name,
            FieldPath::new(["ip"]),
        ))],
    ));

    let batches = execute_against_session(&ctx, "", &ast, Default::default())
        .await
        .expect("enrich stage execution must succeed");

    assert!(
        call_count.load(Ordering::SeqCst) > 0,
        "ENRICH Test 1: call_count must be > 0; got 0. \
         The enrich stage is a silent no-op — ENRICH-4-B not yet implemented"
    );
    let n = total_rows(&batches);
    assert_eq!(n, 2, "ENRICH Test 1: 2 input rows → 2 output rows; got {n}");

    let enriched = str_col(&batches, udf_name);
    assert!(
        enriched.iter().all(|v| v == TEST_SENTINEL),
        "ENRICH Test 1: enriched column must contain sentinel; got: {enriched:?}"
    );
}

// ===========================================================================
// Test 2: where stage filters rows
// ===========================================================================

/// ENRICH-4-B §10.1 Test 2
///
/// `testw2_detections | where status = 'active'`
/// Adapter: 3 rows (2×active, 1×inactive) → expect 2 rows returned.
///
/// Red Gate: all 3 raw rows returned (no WHERE filtering).
/// Green: WHERE clause applied by DataFusion → 2 rows.
#[tokio::test]
async fn test_pipe_where_stage_filters_rows() {
    let sensor = "testw2";
    let batch = make_string_batch("status", &["active", "inactive", "active"]);
    let mut mat_ctx = mat_ctx_with(sensor, vec![batch]);
    let session_ctx =
        build_session_context(QUERY_MEMORY_POOL_BYTES).expect("build_session_context must succeed");

    let q = format!("{} | where status = 'active'", tbl(sensor, "detections"));
    let out = run_materialization_pipeline(&q, &query_opts(), &mut mat_ctx, &session_ctx)
        .await
        .expect("where stage pipeline must succeed");

    let n = total_rows(&out.batches);
    assert_eq!(
        n, 2,
        "WHERE Test 2: must return 2 rows; got {n}. \
         If 3, the where stage is a silent no-op — ENRICH-4-B not yet implemented"
    );
    let statuses = str_col(&out.batches, "status");
    assert!(
        statuses.iter().all(|s| s == "active"),
        "WHERE Test 2: all rows must have status='active'; got: {statuses:?}"
    );
}

// ===========================================================================
// Test 3: limit stage truncates rows
// ===========================================================================

/// ENRICH-4-B §10.1 Test 3
///
/// `testl3_detections | head 2` — adapter returns 5 rows, expect 2 returned.
///
/// Red Gate: all 5 rows returned.
/// Green: `LIMIT 2` applied → 2 rows.
#[tokio::test]
async fn test_pipe_limit_stage_truncates_rows() {
    let sensor = "testl3";
    let batch = make_string_batch("val", &["a", "b", "c", "d", "e"]);
    let mut mat_ctx = mat_ctx_with(sensor, vec![batch]);
    let session_ctx =
        build_session_context(QUERY_MEMORY_POOL_BYTES).expect("build_session_context must succeed");

    let q = format!("{} | head 2", tbl(sensor, "detections"));
    let out = run_materialization_pipeline(&q, &query_opts(), &mut mat_ctx, &session_ctx)
        .await
        .expect("limit stage pipeline must succeed");

    let n = total_rows(&out.batches);
    assert_eq!(
        n, 2,
        "LIMIT Test 3: must return 2 rows; got {n}. \
         If 5, the limit stage is a silent no-op — ENRICH-4-B not yet implemented"
    );
}

// ===========================================================================
// Test 4: stats count(*) returns a single aggregate row
// ===========================================================================

/// ENRICH-4-B §10.1 Test 4
///
/// `tests4_detections | stats count(*)` — adapter returns 4 rows.
/// Expected: 1 aggregate row with count(*) = 4.
///
/// Red Gate: all 4 raw rows returned (no aggregation).
/// Green: `SELECT count(*) FROM …` via DataFusion → 1 row.
#[tokio::test]
async fn test_pipe_stats_count_stage() {
    let sensor = "tests4";
    let batch = make_string_batch("val", &["a", "b", "c", "d"]);
    let mut mat_ctx = mat_ctx_with(sensor, vec![batch]);
    let session_ctx =
        build_session_context(QUERY_MEMORY_POOL_BYTES).expect("build_session_context must succeed");

    let q = format!("{} | stats count(*)", tbl(sensor, "detections"));
    let out = run_materialization_pipeline(&q, &query_opts(), &mut mat_ctx, &session_ctx)
        .await
        .expect("stats stage pipeline must succeed");

    let n = total_rows(&out.batches);
    assert_eq!(
        n, 1,
        "STATS Test 4: must return 1 aggregate row; got {n}. \
         If 4, the stats stage is a silent no-op — ENRICH-4-B not yet implemented"
    );

    let count_vals = int_col(&out.batches, "count(*)");
    assert_eq!(
        count_vals,
        vec![4],
        "STATS Test 4: count(*) must equal 4; got: {count_vals:?}"
    );
}

// ===========================================================================
// Test 5: sort stage orders rows descending
// ===========================================================================

/// ENRICH-4-B §10.1 Test 5
///
/// `tests5_detections | sort severity desc`
/// Input severities [10, 30, 20] → expected output [30, 20, 10].
///
/// Red Gate: rows returned in insertion order [10, 30, 20].
/// Green: `ORDER BY severity DESC` applied → [30, 20, 10].
#[tokio::test]
async fn test_pipe_sort_stage_orders_rows() {
    let sensor = "tests5";
    let batch = make_string_int_batch("name", &["c", "a", "b"], "severity", &[10, 30, 20]);
    let mut mat_ctx = mat_ctx_with(sensor, vec![batch]);
    let session_ctx =
        build_session_context(QUERY_MEMORY_POOL_BYTES).expect("build_session_context must succeed");

    let q = format!("{} | sort severity desc", tbl(sensor, "detections"));
    let out = run_materialization_pipeline(&q, &query_opts(), &mut mat_ctx, &session_ctx)
        .await
        .expect("sort stage pipeline must succeed");

    let n = total_rows(&out.batches);
    assert_eq!(n, 3, "SORT Test 5: must preserve 3 rows; got {n}");

    let severities = int_col(&out.batches, "severity");
    assert_eq!(
        severities,
        vec![30, 20, 10],
        "SORT Test 5: severities must be [30, 20, 10] (desc); got: {severities:?}. \
         If [10, 30, 20], the sort stage is a silent no-op — ENRICH-4-B not yet implemented"
    );
}

// ===========================================================================
// Test 6: fields include stage projects to requested columns only
// ===========================================================================

/// ENRICH-4-B §10.1 Test 6
///
/// `tests6_detections | fields + col1, col2`
/// Adapter returns 4-column rows (col1, col2, col3, col4).
/// Expected: output contains col1 and col2 but NOT col3 or col4.
///
/// Note: virtual fields (_sensor, _client, _source_table) may be present;
/// the assertion only checks that col3/col4 are absent.
///
/// Red Gate: all 4 columns returned.
/// Green: `SELECT col1, col2, <virtual_fields> FROM …` → col3/col4 absent.
#[tokio::test]
async fn test_pipe_fields_include_stage() {
    let sensor = "tests6";
    let schema = Arc::new(Schema::new(vec![
        Field::new("col1", DataType::Utf8, true),
        Field::new("col2", DataType::Utf8, true),
        Field::new("col3", DataType::Utf8, true),
        Field::new("col4", DataType::Utf8, true),
    ]));
    let batch = RecordBatch::try_new(
        schema,
        vec![
            Arc::new(StringArray::from(vec!["a"])) as Arc<dyn Array>,
            Arc::new(StringArray::from(vec!["b"])) as Arc<dyn Array>,
            Arc::new(StringArray::from(vec!["c"])) as Arc<dyn Array>,
            Arc::new(StringArray::from(vec!["d"])) as Arc<dyn Array>,
        ],
    )
    .expect("4-column batch must succeed");

    let mut mat_ctx = mat_ctx_with(sensor, vec![batch]);
    let session_ctx =
        build_session_context(QUERY_MEMORY_POOL_BYTES).expect("build_session_context must succeed");

    let q = format!("{} | fields + col1, col2", tbl(sensor, "detections"));
    let out = run_materialization_pipeline(&q, &query_opts(), &mut mat_ctx, &session_ctx)
        .await
        .expect("fields stage pipeline must succeed");

    assert_eq!(
        total_rows(&out.batches),
        1,
        "FIELDS Test 6: must preserve 1 row"
    );

    let schema = out.batches[0].schema();
    let col_names: Vec<&str> = schema.fields().iter().map(|f| f.name().as_str()).collect();

    assert!(
        col_names.contains(&"col1"),
        "FIELDS Test 6: col1 must be present; got: {col_names:?}"
    );
    assert!(
        col_names.contains(&"col2"),
        "FIELDS Test 6: col2 must be present; got: {col_names:?}"
    );
    assert!(
        !col_names.contains(&"col3"),
        "FIELDS Test 6: col3 must NOT be present; got: {col_names:?}. \
         If present, fields stage is a silent no-op — ENRICH-4-B not yet implemented"
    );
    assert!(
        !col_names.contains(&"col4"),
        "FIELDS Test 6: col4 must NOT be present; got: {col_names:?}. \
         If present, fields stage is a silent no-op — ENRICH-4-B not yet implemented"
    );
}

// ===========================================================================
// Test 7: chained enrich then where — CTE pattern
// ===========================================================================

/// ENRICH-4-B §10.1 Test 7
///
/// `FROM test_e7_tbl | enrich test_e7_udf(ip) | where test_e7_udf IS NOT NULL`
///
/// Adapter returns 2 rows: "match" (UDF returns sentinel), "null_me" (UDF returns NULL).
/// Expected: 1 row returned (only the non-null enrichment survives the WHERE filter).
///
/// This test verifies the CTE pattern where the WHERE clause references the enriched
/// column produced by the prior ENRICH stage.
///
/// Red Gate: 2 raw rows returned (no UDF call, no WHERE filter).
/// Green: CTE-based SQL lowers both stages → 1 row.
#[tokio::test]
async fn test_pipe_chained_enrich_then_where() {
    let table = "test_e7_tbl";
    let udf_name = "test_e7_udf";
    let batch = make_string_batch("ip", &["match", "null_me"]);
    let (ctx, _call_count) =
        make_udf_ctx(udf_name, table, vec![batch], Some("null_me".to_string()));

    use prism_query::ast::{
        Ast, EnrichStage, FieldPath, PipeQuery, PipeStage, Predicate, SourceRef,
    };
    let ast = Ast::Pipe(PipeQuery::new(
        SourceRef::from_raw(table),
        vec![
            PipeStage::Enrich(EnrichStage::new(udf_name, FieldPath::new(["ip"]))),
            PipeStage::Where(Predicate::IsNull {
                field: FieldPath::new([udf_name]),
                negated: true, // IS NOT NULL
            }),
        ],
    ));

    let batches = execute_against_session(&ctx, "", &ast, Default::default())
        .await
        .expect("chained enrich+where must succeed");

    let n = total_rows(&batches);
    assert_eq!(
        n, 1,
        "CHAIN Test 7: must return 1 row; got {n}. \
         If 2, CTE chaining/WHERE filter broken — ENRICH-4-B not yet implemented"
    );

    let enriched = str_col(&batches, udf_name);
    assert_eq!(
        enriched,
        vec![TEST_SENTINEL],
        "CHAIN Test 7: remaining row must have sentinel; got: {enriched:?}"
    );
}

// ===========================================================================
// Test 8: memory budget error surfaces E-WATCHDOG-001 (not silent Ok)
// ===========================================================================

/// ENRICH-4-B §10.1 Test 8
///
/// Pipe query with sort stage on 1-byte pool returns an Err — verifying the
/// pipe arm routes through DataFusion (and therefore through `map_datafusion_memory_error`).
///
/// A `| sort ip desc` stage forces DataFusion to create a `SortExec`, which
/// calls `try_grow()` on the memory pool before processing any batch.
/// With a 1-byte pool, this reservation fails → `ResourcesExhausted` → mapped
/// to `PrismError::QueryMemoryBudgetExceeded` (E-WATCHDOG-001) by
/// `map_datafusion_memory_error`.
///
/// This pattern is identical to `test_qry03_memory_pool_trip_in_sql_execution_maps_to_memory_variant`
/// in integration_tests.rs which tests the SQL arm; here we exercise the Pipe arm.
///
/// Red Gate: before ENRICH-4-B, pipe arm bypasses DataFusion entirely → Ok(raw rows).
/// Green: pipe arm calls `session_ctx.sql(...)` → SortExec trips pool → Err propagates.
#[tokio::test]
async fn test_pipe_memory_budget_error_surfaces_e_watchdog_001() {
    let table = "test_e8_tbl";
    let values: Vec<&str> = (0..100).map(|_| "192.168.1.1").collect();
    let batch = make_string_batch("ip", &values);

    // 1-byte pool — SortExec must call try_grow() which will immediately fail.
    let ctx =
        build_session_context(1).expect("build_session_context must succeed with 1-byte pool");
    register_mem_table(&ctx, table, vec![batch]).expect("register_mem_table must succeed");

    use prism_query::ast::{
        Ast, FieldPath, PipeQuery, PipeStage, SortDirection, SortExpr, SourceRef,
    };
    let ast = Ast::Pipe(PipeQuery::new(
        SourceRef::from_raw(table),
        vec![PipeStage::Sort(vec![SortExpr::new(
            FieldPath::new(["ip"]),
            SortDirection::Desc,
        )])],
    ));

    let result = execute_against_session(&ctx, "", &ast, Default::default()).await;
    let err = result.expect_err(
        "MEM Test 8: 1-byte pool + SortExec must fail — got Ok. \
         If Ok, the pipe sort stage bypassed DataFusion — ENRICH-4-B not yet implemented",
    );

    // Strict assertion mirrors sibling test_qry03_memory_pool_trip_in_sql_execution_maps_to_memory_variant:
    // The pipe arm routes through map_datafusion_memory_error identically to the SQL arm,
    // so a ResourcesExhausted error from SortExec MUST produce QueryMemoryBudgetExceeded,
    // not the generic QueryExecutionFailed.  If this assertion fails, it means the pipe
    // arm is NOT correctly mapping memory errors — a real code bug, not just a test gap.
    assert!(
        matches!(err, PrismError::QueryMemoryBudgetExceeded { .. }),
        "MEM Test 8: 1-byte pool trip must produce QueryMemoryBudgetExceeded (E-WATCHDOG-001), \
         not {:?}. If QueryExecutionFailed, the pipe arm is NOT routing through \
         map_datafusion_memory_error — BC-2.11.006 invariant violated.",
        err
    );

    let msg = err.to_string();
    assert!(
        msg.contains("E-WATCHDOG-001"),
        "MEM Test 8: memory-budget error must carry E-WATCHDOG-001 code; got: {msg}"
    );
}
