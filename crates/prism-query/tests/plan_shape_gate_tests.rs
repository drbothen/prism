//! Plan-Shape Gate Red Gate Tests — RG-PSG-001..RG-PSG-009
//!
//! Traces to: S-ENGINE-LIMIT-EARLY-STOP-001 AC-007, ADR-060 §D8.7,
//!            BC-2.16.002 (Multi-Step Fetch Pipeline Execution)
//!
//! # Purpose
//!
//! These tests are the RED GATE for the plan-shape gate feature of story
//! S-ENGINE-LIMIT-EARLY-STOP-001. They MUST FAIL before `ast_is_reducing_plan`
//! is implemented in `materialization.rs` (Task 11). They PASS after the gate
//! is wired (Task 11 green).
//!
//! # SAP-3 Compliance
//!
//! Every test reaches the gate end-to-end through `run_materialization_pipeline`
//! from a real PrismQL/SQL query string — not via a synthetic AST injected into
//! an internal handler. This satisfies the spec-arm reachability requirement
//! from SAP-3.
//!
//! # Mock Adapter Design
//!
//! `PlanShapeGateMockAdapter` observes `params.limit` (the `fetch_limit` value
//! from `run_materialization_pipeline` line 840):
//!
//! - `params.limit == 0`: early-stop is SUPPRESSED → return all 300 rows (3 pages × 100)
//! - `params.limit > 0`: early-stop is ACTIVE → return 100 rows (page 1 only)
//!
//! Data layout:
//! - Page 1: 100 rows, status = "page1"
//! - Page 2: 100 rows, status = "page2"  (only available when gate suppresses)
//! - Page 3: 100 rows, status = "page3"  (only available when gate suppresses)
//!
//! # Red Gate Mechanics
//!
//! Before the gate (`fetch_limit = options.limit` unconditionally):
//! - Suppression tests: `options.limit = Some(25)` → `fetch_limit = 25 > 0`
//!   → mock returns 100 rows → assertions on 300-row results FAIL ✓
//!
//! After the gate (`ast_is_reducing_plan` short-circuits to `fetch_limit = 0`):
//! - Suppression tests: `fetch_limit = 0` → mock returns 300 rows
//!   → assertions on 300-row results PASS ✓
//!
//! Positive controls (PSG-007, PSG-008) are designed to PASS both before AND
//! after the gate — they confirm the gate does NOT suppress bare projections
//! or ORDER BY–only queries (ADR-060 §D8.5).
//!
//! # Test Matrix
//!
//! | Test name (RG-ID)                                       | Condition | Red? |
//! |--------------------------------------------------------|-----------|------|
//! | test_BC_2_16_002_plan_shape_gate_count_suppresses_early_stop           (PSG-001) | A | RED  |
//! | test_BC_2_16_002_plan_shape_gate_group_by_suppresses_early_stop        (PSG-002) | B | RED  |
//! | test_BC_2_16_002_plan_shape_gate_distinct_suppresses_early_stop        (PSG-003) | C | RED  |
//! | test_BC_2_16_002_plan_shape_gate_non_temporal_where_suppresses_early_stop (PSG-004) | G | RED  |
//! | test_BC_2_16_002_plan_shape_gate_pipe_stats_suppresses_early_stop      (PSG-005) | E | RED  |
//! | test_BC_2_16_002_plan_shape_gate_pipe_dedup_suppresses_early_stop      (PSG-006) | F | RED  |
//! | test_BC_2_16_002_plan_shape_gate_bare_projection_early_stop_fires      (PSG-007) | — | GREEN|
//! | test_BC_2_16_002_plan_shape_gate_order_by_limit_early_stop_fires       (PSG-008) | — | GREEN|
//! | test_BC_2_16_002_plan_shape_gate_having_suppresses_early_stop          (PSG-009) | D | RED  |

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    non_snake_case,
    unused_imports
)]

use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};

use arrow::array::{Array, Int64Array, StringArray};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use async_trait::async_trait;
use prism_core::{OrgId, OrgSlug, SensorId};
use prism_ocsf::OcsfNormalizer;
use prism_query::{
    engine::QueryOptions,
    materialization::{run_materialization_pipeline, MaterializationContext},
    memory::{build_session_context, QUERY_MEMORY_POOL_BYTES},
};
use prism_sensors::{
    adapter::{QueryParams, SensorAdapter, SensorError, SensorSpec},
    auth::SensorAuth,
    AdapterRegistry, CredentialResolver,
};

// ---------------------------------------------------------------------------
// PlanShapeGateMockAdapter
// ---------------------------------------------------------------------------

/// Plan-shape gate mock adapter.
///
/// Records the `params.limit` value passed by `run_materialization_pipeline`
/// and returns 300 or 100 rows accordingly — simulating the downstream
/// `SpecDrivenSensorAdapter` + `PipelineExecutor` early-stop behaviour at the
/// `SensorAdapter::fetch` boundary:
///
/// - `params.limit == 0`: gate suppressed early-stop → all 300 rows available
/// - `params.limit > 0`: early-stop active → only page 1 (100 rows) returned
struct PlanShapeGateMockAdapter {
    /// The last `params.limit` received — lets tests assert on gate output.
    last_limit: Arc<AtomicU64>,
    /// 300 rows (3 pages × 100) — returned when `params.limit == 0`.
    full_batches: Vec<RecordBatch>,
    /// 100 rows (page 1 only) — returned when `params.limit > 0`.
    page1_batches: Vec<RecordBatch>,
}

impl std::fmt::Debug for PlanShapeGateMockAdapter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PlanShapeGateMockAdapter")
            .field("last_limit", &self.last_limit.load(Ordering::Relaxed))
            .finish()
    }
}

#[async_trait]
impl SensorAdapter for PlanShapeGateMockAdapter {
    fn sensor_type(&self) -> SensorId {
        SensorId::from("mock")
    }

    fn sensor_name(&self) -> &'static str {
        "mock"
    }

    async fn fetch(
        &self,
        _spec: &SensorSpec,
        params: &QueryParams,
        _auth: &dyn SensorAuth,
    ) -> Result<Vec<RecordBatch>, SensorError> {
        // Record the fetch_limit the pipeline wired into params.limit.
        self.last_limit.store(params.limit, Ordering::SeqCst);

        if params.limit == 0 {
            // fetch_limit == 0 means early-stop is suppressed — return all 300 rows.
            Ok(self.full_batches.clone())
        } else {
            // fetch_limit > 0 means early-stop is active — return page 1 only.
            Ok(self.page1_batches.clone())
        }
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
// Test infrastructure helpers
// ---------------------------------------------------------------------------

/// Build a RecordBatch with `n` rows where every row has `value` in "status".
fn make_status_batch(value: &str, n: usize) -> RecordBatch {
    let schema = Arc::new(Schema::new(vec![Field::new(
        "status",
        DataType::Utf8,
        true,
    )]));
    let values: Vec<Option<&str>> = std::iter::repeat_n(Some(value), n).collect();
    let array = Arc::new(StringArray::from(values)) as Arc<dyn Array>;
    RecordBatch::try_new(schema, vec![array]).expect("make_status_batch must succeed")
}

/// Sum total rows across all batches.
fn total_rows(batches: &[RecordBatch]) -> usize {
    batches.iter().map(|b| b.num_rows()).sum()
}

/// Extract the first `Int64` value from a named column across all batches.
fn first_i64(batches: &[RecordBatch], col: &str) -> i64 {
    for b in batches {
        if let Ok(idx) = b.schema().index_of(col) {
            let arr = b
                .column(idx)
                .as_any()
                .downcast_ref::<Int64Array>()
                .expect("Int64Array");
            if arr.len() > 0 {
                return arr.value(0);
            }
        }
    }
    panic!("column '{col}' not found or empty in result batches");
}

/// Construct a fresh `MaterializationContext` with the plan-shape gate mock adapter.
///
/// Returns the context AND the shared `last_limit` arc so individual tests can
/// assert on the `params.limit` value the pipeline wired to the adapter.
fn plan_gate_mat_ctx() -> (MaterializationContext, Arc<AtomicU64>) {
    let last_limit = Arc::new(AtomicU64::new(0));

    // Data pages — each 100 rows with a distinct status value.
    let page1 = make_status_batch("page1", 100);
    let page2 = make_status_batch("page2", 100);
    let page3 = make_status_batch("page3", 100);

    let adapter = Arc::new(PlanShapeGateMockAdapter {
        last_limit: Arc::clone(&last_limit),
        full_batches: vec![page1.clone(), page2, page3], // 300 rows total
        page1_batches: vec![page1],                      // 100 rows — early-stop fires
    });

    let org_id = OrgId::new();
    let mut registry = AdapterRegistry::new();
    registry.register(org_id, adapter);

    let mat_ctx = MaterializationContext::new_with_resolver(
        Arc::new(registry),
        Arc::new(OcsfNormalizer::new()),
        prism_query::memory::MAX_MATERIALIZED_RECORDS,
        Arc::new(StubCredentialResolver),
        None, // org_registry absent — test mode (resolve_org_id falls back to Path 2)
        None, // resolved_spec_map absent — no push-down spec
    );

    (mat_ctx, last_limit)
}

/// Build `QueryOptions` with an explicit `limit`.
///
/// Uses `"test-org"` as the client slug — `resolve_org_id` falls back to the
/// first registered adapter (Path 2) when OrgRegistry is absent (test mode).
fn opts(limit: usize) -> QueryOptions {
    QueryOptions {
        clients: Some(vec![OrgSlug::new_unchecked("test-org")]),
        sensors: None,
        limit: Some(limit),
        force_refresh: false,
        ..QueryOptions::default()
    }
}

// ===========================================================================
// PSG-001 — Condition A: SQL aggregate (COUNT) suppresses early-stop
// ===========================================================================

/// RG-PSG-001 — AC-007 Condition A: `FuncCall::Aggregate` in SELECT
///
/// `SELECT COUNT(*) as cnt FROM mock_events LIMIT 25`
///
/// Before gate: `fetch_limit = 25` → mock returns 100 rows → COUNT = 100.
/// After gate: `ast_is_reducing_plan` detects aggregate → `fetch_limit = 0`
///   → mock returns 300 rows → COUNT = 300.
///
/// SAP-3: reaches `ast_is_reducing_plan` through `run_materialization_pipeline`
/// from a SQL query string — NOT via a synthetic AST.
#[tokio::test]
async fn test_BC_2_16_002_plan_shape_gate_count_suppresses_early_stop() {
    let (mut mat_ctx, last_limit) = plan_gate_mat_ctx();
    let session_ctx =
        build_session_context(QUERY_MEMORY_POOL_BYTES).expect("build_session_context");

    // Condition A: COUNT aggregate.
    let query = "SELECT COUNT(*) as cnt FROM mock_events LIMIT 25";
    let out = run_materialization_pipeline(query, &opts(25), &mut mat_ctx, &session_ctx)
        .await
        .expect("PSG-001: pipeline must not error");

    // PRIMARY behavioral assertion (SAP-3 end-to-end).
    // RED: COUNT = 100 (fetch_limit=25 → 100 rows fetched → count(100) ≠ 300).
    // GREEN: COUNT = 300 (fetch_limit=0 → 300 rows fetched → count(300) = 300).
    let count_val = first_i64(&out.batches, "cnt");
    assert_eq!(
        count_val, 300,
        "PSG-001 (Condition A — COUNT): COUNT must aggregate all 300 rows when \
         plan-shape gate suppresses early-stop; got {count_val}. \
         If 100, the gate is not yet implemented (fetch_limit=25 → only 1 page fetched)."
    );

    // SECONDARY mechanism assertion: gate must set fetch_limit=0.
    let seen_limit = last_limit.load(Ordering::SeqCst);
    assert_eq!(
        seen_limit, 0,
        "PSG-001 (Condition A — COUNT): gate must suppress early-stop (fetch_limit=0); \
         adapter saw params.limit={seen_limit}. \
         If 25, the gate is not yet implemented."
    );
}

// ===========================================================================
// PSG-002 — Condition B: GROUP BY suppresses early-stop
// ===========================================================================

/// RG-PSG-002 — AC-007 Condition B: non-empty GROUP BY clause
///
/// `SELECT status, COUNT(*) as cnt FROM mock_events GROUP BY status LIMIT 25`
///
/// Before gate: 100 rows (all "page1") → 1 group in result.
/// After gate: 300 rows (3 pages) → 3 groups: page1, page2, page3.
///
/// SAP-3: reaches `ast_is_reducing_plan` through `run_materialization_pipeline`.
#[tokio::test]
async fn test_BC_2_16_002_plan_shape_gate_group_by_suppresses_early_stop() {
    let (mut mat_ctx, last_limit) = plan_gate_mat_ctx();
    let session_ctx =
        build_session_context(QUERY_MEMORY_POOL_BYTES).expect("build_session_context");

    // Condition B: GROUP BY non-empty.
    let query = "SELECT status, COUNT(*) as cnt FROM mock_events GROUP BY status LIMIT 25";
    let out = run_materialization_pipeline(query, &opts(25), &mut mat_ctx, &session_ctx)
        .await
        .expect("PSG-002: pipeline must not error");

    // PRIMARY: expect 3 groups (page1, page2, page3) — one per distinct status page.
    // RED: 1 group (only page1 from 100 rows).
    // GREEN: 3 groups (all 300 rows → 3 distinct statuses).
    let group_count = total_rows(&out.batches);
    assert_eq!(
        group_count, 3,
        "PSG-002 (Condition B — GROUP BY): must produce 3 groups when gate suppresses \
         early-stop; got {group_count}. \
         If 1, the gate is not yet implemented (fetch_limit=25 → only page1 fetched)."
    );

    // SECONDARY: gate must have set fetch_limit=0.
    let seen_limit = last_limit.load(Ordering::SeqCst);
    assert_eq!(
        seen_limit, 0,
        "PSG-002 (Condition B — GROUP BY): gate must suppress early-stop (fetch_limit=0); \
         adapter saw params.limit={seen_limit}."
    );
}

// ===========================================================================
// PSG-003 — Condition C: SELECT DISTINCT suppresses early-stop
// ===========================================================================

/// RG-PSG-003 — AC-007 Condition C: `SELECT DISTINCT`
///
/// `SELECT DISTINCT status FROM mock_events LIMIT 25`
///
/// Before gate: 100 rows (all "page1") → 1 distinct value.
/// After gate: 300 rows → 3 distinct values: page1, page2, page3.
///
/// SAP-3: reaches `ast_is_reducing_plan` through `run_materialization_pipeline`.
#[tokio::test]
async fn test_BC_2_16_002_plan_shape_gate_distinct_suppresses_early_stop() {
    let (mut mat_ctx, last_limit) = plan_gate_mat_ctx();
    let session_ctx =
        build_session_context(QUERY_MEMORY_POOL_BYTES).expect("build_session_context");

    // Condition C: SELECT DISTINCT.
    let query = "SELECT DISTINCT status FROM mock_events LIMIT 25";
    let out = run_materialization_pipeline(query, &opts(25), &mut mat_ctx, &session_ctx)
        .await
        .expect("PSG-003: pipeline must not error");

    // PRIMARY: 3 distinct status values when gate suppresses.
    // RED: 1 distinct value ("page1" only, from 100 rows).
    // GREEN: 3 distinct values (300 rows → page1, page2, page3).
    let distinct_count = total_rows(&out.batches);
    assert_eq!(
        distinct_count, 3,
        "PSG-003 (Condition C — DISTINCT): must return 3 distinct status values when \
         gate suppresses early-stop; got {distinct_count}. \
         If 1, the gate is not yet implemented."
    );

    // SECONDARY: gate must have set fetch_limit=0.
    let seen_limit = last_limit.load(Ordering::SeqCst);
    assert_eq!(
        seen_limit, 0,
        "PSG-003 (Condition C — DISTINCT): gate must suppress early-stop (fetch_limit=0); \
         adapter saw params.limit={seen_limit}."
    );
}

// ===========================================================================
// PSG-004 — Condition G: non-temporal WHERE suppresses early-stop
// ===========================================================================

/// RG-PSG-004 — AC-007 Condition G: non-temporal WHERE predicate
///
/// `SELECT * FROM mock_events WHERE status = 'page2' LIMIT 25`
///
/// `extract_push_down_filters_as_map` collects `status = 'page2'` as a string
/// equality filter → `where_filters` is non-empty → Condition G fires.
///
/// Before gate: 100 rows (all "page1") → WHERE status = 'page2' → 0 matches.
/// After gate: 300 rows → 100 rows with status = 'page2' → LIMIT 25 → 25 rows.
///
/// Safety of temporal-only test (ADR-060 §D8.7): a temporal predicate like
/// `WHERE timestamp > X` uses a Gt comparator, which `predicate_tree_to_filter_map`
/// does NOT collect (only Eq is collected). So `where_filters` remains empty for
/// temporal-only WHERE → gate does not suppress → early-stop remains safe.
///
/// SAP-3: reaches `ast_is_reducing_plan` through `run_materialization_pipeline`.
#[tokio::test]
async fn test_BC_2_16_002_plan_shape_gate_non_temporal_where_suppresses_early_stop() {
    let (mut mat_ctx, last_limit) = plan_gate_mat_ctx();
    let session_ctx =
        build_session_context(QUERY_MEMORY_POOL_BYTES).expect("build_session_context");

    // Condition G: non-temporal WHERE push-down filter.
    // `status = 'page2'` is a string equality predicate → appears in where_filters.
    let query = "SELECT * FROM mock_events WHERE status = 'page2' LIMIT 25";
    let out = run_materialization_pipeline(query, &opts(25), &mut mat_ctx, &session_ctx)
        .await
        .expect("PSG-004: pipeline must not error");

    // PRIMARY: 25 rows matching status='page2' after gate suppresses early-stop.
    // RED: 0 rows (page1 only from 100 rows, none match 'page2').
    // GREEN: 25 rows (from the 100 'page2' rows in full data, limited to 25).
    let row_count = total_rows(&out.batches);
    assert_eq!(
        row_count, 25,
        "PSG-004 (Condition G — non-temporal WHERE): must return 25 rows matching \
         status='page2' when gate suppresses early-stop; got {row_count}. \
         If 0, the gate is not yet implemented (fetch_limit=25 → only page1 fetched, \
         none match 'page2')."
    );

    // SECONDARY: gate must have set fetch_limit=0.
    let seen_limit = last_limit.load(Ordering::SeqCst);
    assert_eq!(
        seen_limit, 0,
        "PSG-004 (Condition G — non-temporal WHERE): gate must suppress early-stop \
         (fetch_limit=0); adapter saw params.limit={seen_limit}."
    );
}

// ===========================================================================
// PSG-005 — Condition E: PipeStage::Stats suppresses early-stop
// ===========================================================================

/// RG-PSG-005 — AC-007 Condition E: `PipeStage::Stats`
///
/// `mock_events | stats count(*)`
///
/// `extract_push_down_filters_as_map` returns empty for pipe queries (only SQL
/// WHERE is extracted). `ast_is_reducing_plan` must detect `PipeStage::Stats`
/// directly from the pipe stages.
///
/// Before gate: `fetch_limit = 25` → mock returns 100 rows → count(*) = 100.
/// After gate: `fetch_limit = 0` → mock returns 300 rows → count(*) = 300.
///
/// SAP-3: reaches `ast_is_reducing_plan` through `run_materialization_pipeline`.
#[tokio::test]
async fn test_BC_2_16_002_plan_shape_gate_pipe_stats_suppresses_early_stop() {
    let (mut mat_ctx, last_limit) = plan_gate_mat_ctx();
    let session_ctx =
        build_session_context(QUERY_MEMORY_POOL_BYTES).expect("build_session_context");

    // Condition E: PipeStage::Stats — pipe stats aggregation.
    let query = "mock_events | stats count(*)";
    let out = run_materialization_pipeline(query, &opts(25), &mut mat_ctx, &session_ctx)
        .await
        .expect("PSG-005: pipeline must not error");

    // PRIMARY: count(*) = 300 when gate suppresses early-stop.
    // RED: count(*) = 100 (fetch_limit=25 → 100 rows fetched).
    // GREEN: count(*) = 300 (fetch_limit=0 → 300 rows fetched).
    let count_val = first_i64(&out.batches, "count(*)");
    assert_eq!(
        count_val, 300,
        "PSG-005 (Condition E — PipeStage::Stats): count(*) must aggregate all 300 rows \
         when gate suppresses early-stop; got {count_val}. \
         If 100, the gate is not yet implemented (fetch_limit=25 → only 1 page fetched)."
    );

    // SECONDARY: gate must have set fetch_limit=0.
    let seen_limit = last_limit.load(Ordering::SeqCst);
    assert_eq!(
        seen_limit, 0,
        "PSG-005 (Condition E — PipeStage::Stats): gate must suppress early-stop \
         (fetch_limit=0); adapter saw params.limit={seen_limit}."
    );
}

// ===========================================================================
// PSG-006 — Condition F: PipeStage::Dedup suppresses early-stop
// ===========================================================================

/// RG-PSG-006 — AC-007 Condition F: `PipeStage::Dedup`
///
/// `mock_events | dedup status | limit 25`
///
/// `apply_dedup` in `pipe_sql_emitter.rs` lowers to `SELECT DISTINCT status FROM …
/// LIMIT 25`. The `PipeStage::Dedup` stage must be detected by `ast_is_reducing_plan`.
///
/// Before gate: 100 rows (all "page1") → DISTINCT status → 1 unique value.
/// After gate: 300 rows → DISTINCT status → 3 unique values.
///
/// SAP-3: reaches `ast_is_reducing_plan` through `run_materialization_pipeline`.
#[tokio::test]
async fn test_BC_2_16_002_plan_shape_gate_pipe_dedup_suppresses_early_stop() {
    let (mut mat_ctx, last_limit) = plan_gate_mat_ctx();
    let session_ctx =
        build_session_context(QUERY_MEMORY_POOL_BYTES).expect("build_session_context");

    // Condition F: PipeStage::Dedup — dedup on status field.
    let query = "mock_events | dedup status | limit 25";
    let out = run_materialization_pipeline(query, &opts(25), &mut mat_ctx, &session_ctx)
        .await
        .expect("PSG-006: pipeline must not error");

    // PRIMARY: 3 unique statuses when gate suppresses early-stop.
    // RED: 1 unique status ("page1" only, from 100 rows).
    // GREEN: 3 unique statuses (page1, page2, page3 from 300 rows, capped at 25).
    let distinct_count = total_rows(&out.batches);
    assert_eq!(
        distinct_count, 3,
        "PSG-006 (Condition F — PipeStage::Dedup): must return 3 distinct statuses when \
         gate suppresses early-stop; got {distinct_count}. \
         If 1, the gate is not yet implemented (fetch_limit=25 → only page1 fetched)."
    );

    // SECONDARY: gate must have set fetch_limit=0.
    let seen_limit = last_limit.load(Ordering::SeqCst);
    assert_eq!(
        seen_limit, 0,
        "PSG-006 (Condition F — PipeStage::Dedup): gate must suppress early-stop \
         (fetch_limit=0); adapter saw params.limit={seen_limit}."
    );
}

// ===========================================================================
// PSG-007 — Positive control: bare SELECT * LIMIT N does NOT suppress early-stop
// ===========================================================================

/// RG-PSG-007 — AC-007 positive control: bare projection
///
/// `SELECT * FROM mock_events LIMIT 5`
///
/// A bare projection with no aggregation, GROUP BY, DISTINCT, HAVING, pipe stats,
/// pipe dedup, or non-temporal WHERE → `ast_is_reducing_plan` returns FALSE.
/// Early-stop MUST fire (fetch_limit = 5 > 0).
///
/// This test PASSES both before AND after the gate. It confirms the gate does NOT
/// over-suppress: bare SELECT * queries can safely use early-stop.
///
/// SAP-3: reaches the gate decision path through `run_materialization_pipeline`.
#[tokio::test]
async fn test_BC_2_16_002_plan_shape_gate_bare_projection_early_stop_fires() {
    let (mut mat_ctx, last_limit) = plan_gate_mat_ctx();
    let session_ctx =
        build_session_context(QUERY_MEMORY_POOL_BYTES).expect("build_session_context");

    // No reducing operation: bare SELECT * with LIMIT 5.
    let query = "SELECT * FROM mock_events LIMIT 5";
    let out = run_materialization_pipeline(query, &opts(5), &mut mat_ctx, &session_ctx)
        .await
        .expect("PSG-007: pipeline must not error");

    // PRIMARY: DataFusion LIMIT 5 applied → exactly 5 rows returned.
    // PASSES before gate (fetch_limit=5 → 100 rows → LIMIT 5 → 5).
    // PASSES after gate (bare projection → NOT reducing → fetch_limit=5 → same path).
    let row_count = total_rows(&out.batches);
    assert_eq!(
        row_count, 5,
        "PSG-007 (positive control — bare projection): must return exactly 5 rows; \
         got {row_count}. LIMIT 5 must be applied regardless of gate state."
    );

    // SECONDARY: early-stop must NOT be suppressed for bare projections.
    // fetch_limit = 5 > 0 → mock returns 100 rows → LIMIT 5 → 5 rows.
    let seen_limit = last_limit.load(Ordering::SeqCst);
    assert!(
        seen_limit > 0,
        "PSG-007 (positive control — bare projection): gate must NOT suppress early-stop \
         for bare SELECT *; adapter saw params.limit={seen_limit} (expected > 0)."
    );
}

// ===========================================================================
// PSG-008 — Positive control: ORDER BY + LIMIT does NOT suppress early-stop
// ===========================================================================

/// RG-PSG-008 — AC-007 positive control: ORDER BY alone does not suppress
///
/// `SELECT * FROM mock_events ORDER BY status LIMIT 5`
///
/// Per ADR-060 §D8.5: ORDER BY alone does NOT trigger suppression. ORDER BY is
/// safe with early-stop because the ordering is applied post-fetch — result set
/// correctness is not affected by whether pages 2 and 3 were fetched or not (for
/// a bounded result).
///
/// This test PASSES both before AND after the gate.
///
/// SAP-3: reaches the gate decision path through `run_materialization_pipeline`.
#[tokio::test]
async fn test_BC_2_16_002_plan_shape_gate_order_by_limit_early_stop_fires() {
    let (mut mat_ctx, last_limit) = plan_gate_mat_ctx();
    let session_ctx =
        build_session_context(QUERY_MEMORY_POOL_BYTES).expect("build_session_context");

    // ORDER BY alone: does NOT suppress early-stop (ADR-060 §D8.5).
    let query = "SELECT * FROM mock_events ORDER BY status LIMIT 5";
    let out = run_materialization_pipeline(query, &opts(5), &mut mat_ctx, &session_ctx)
        .await
        .expect("PSG-008: pipeline must not error");

    // PRIMARY: exactly 5 rows (LIMIT 5 applied after ORDER BY).
    let row_count = total_rows(&out.batches);
    assert_eq!(
        row_count, 5,
        "PSG-008 (positive control — ORDER BY + LIMIT): must return exactly 5 rows; \
         got {row_count}. ORDER BY alone must not suppress early-stop (ADR-060 §D8.5)."
    );

    // SECONDARY: early-stop must NOT be suppressed for ORDER BY–only queries.
    let seen_limit = last_limit.load(Ordering::SeqCst);
    assert!(
        seen_limit > 0,
        "PSG-008 (positive control — ORDER BY + LIMIT): gate must NOT suppress \
         early-stop for ORDER BY–only queries; adapter saw params.limit={seen_limit} \
         (expected > 0). ADR-060 §D8.5 explicitly excludes ORDER BY from suppression."
    );
}

// ===========================================================================
// PSG-009 — Condition D: HAVING suppresses early-stop
// ===========================================================================

/// RG-PSG-009 — AC-007 Condition D: HAVING clause present
///
/// `SELECT status, COUNT(*) as cnt FROM mock_events GROUP BY status HAVING COUNT(*) > 50 LIMIT 25`
///
/// HAVING is parsed by `build_having_predicate_parser` in `sql_parser.rs` and
/// stored in `SqlQuery.having: Option<Predicate>`. `ast_is_reducing_plan` must
/// detect `having.is_some()` as Condition D, independently of Condition B (GROUP BY).
///
/// Before gate: 100 rows (all "page1") → 1 group (cnt=100, qualifies HAVING > 50)
///   → 1 result row.
/// After gate: 300 rows → 3 groups (page1:100, page2:100, page3:100, all qualify)
///   → 3 result rows.
///
/// SAP-3: HAVING is end-to-end reachable from the SQL parser input; this test
/// reaches `ast_is_reducing_plan` via `run_materialization_pipeline` from a
/// real SQL string.
#[tokio::test]
async fn test_BC_2_16_002_plan_shape_gate_having_suppresses_early_stop() {
    let (mut mat_ctx, last_limit) = plan_gate_mat_ctx();
    let session_ctx =
        build_session_context(QUERY_MEMORY_POOL_BYTES).expect("build_session_context");

    // Condition D: HAVING clause present.
    // All groups have cnt=100 which is > 50, so HAVING does not filter any out.
    // The distinction between RED and GREEN is the number of groups fetched.
    let query =
        "SELECT status, COUNT(*) as cnt FROM mock_events GROUP BY status HAVING COUNT(*) > 50 LIMIT 25";
    let out = run_materialization_pipeline(query, &opts(25), &mut mat_ctx, &session_ctx)
        .await
        .expect("PSG-009: pipeline must not error");

    // PRIMARY: 3 result groups when gate suppresses early-stop.
    // RED: 1 group (only page1 from 100 rows, cnt=100 > 50 qualifies — still 1 group).
    // GREEN: 3 groups (page1, page2, page3 — all have cnt=100 > 50).
    let group_count = total_rows(&out.batches);
    assert_eq!(
        group_count, 3,
        "PSG-009 (Condition D — HAVING): must produce 3 groups when gate suppresses \
         early-stop; got {group_count}. \
         If 1, the gate is not yet implemented (fetch_limit=25 → only page1 fetched)."
    );

    // SECONDARY: gate must have set fetch_limit=0.
    let seen_limit = last_limit.load(Ordering::SeqCst);
    assert_eq!(
        seen_limit, 0,
        "PSG-009 (Condition D — HAVING): gate must suppress early-stop (fetch_limit=0); \
         adapter saw params.limit={seen_limit}."
    );
}
