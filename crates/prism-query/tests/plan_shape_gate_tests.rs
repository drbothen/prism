//! Plan-Shape Gate Red Gate Tests — RG-PSG-001..RG-PSG-019
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
//! Every E2E test reaches the gate end-to-end through `run_materialization_pipeline`
//! from a real PrismQL/SQL query string — not via a synthetic AST injected into
//! an internal handler. This satisfies the spec-arm reachability requirement
//! from SAP-3.
//!
//! Three tests (PSG-009, PSG-012, PSG-019) are in-crate unit tests in
//! `materialization.rs` and carry explicit SAP-3 rule 3 reachability comments.
//!
//! # Mock Adapter Design
//!
//! `PlanShapeGateMockAdapter` observes `params.limit` (the `fetch_limit` value
//! from `run_materialization_pipeline` — the `fetch_limit` binding):
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
//! | Test name (RG-ID)                                                                       | Cond  | Kind       |
//! |----------------------------------------------------------------------------------------|-------|------------|
//! | test_BC_2_16_002_plan_shape_gate_count_suppresses_early_stop (PSG-001)                 | A     | RED, E2E   |
//! | test_BC_2_16_002_plan_shape_gate_group_by_suppresses_early_stop (PSG-002)              | B     | RED, E2E   |
//! | test_BC_2_16_002_plan_shape_gate_distinct_suppresses_early_stop (PSG-003)              | C     | RED, E2E   |
//! | test_BC_2_16_002_plan_shape_gate_non_temporal_where_suppresses_early_stop (PSG-004)    | G     | RED, E2E   |
//! | test_BC_2_16_002_plan_shape_gate_pipe_stats_suppresses_early_stop (PSG-005)            | E     | RED, E2E   |
//! | test_BC_2_16_002_plan_shape_gate_pipe_dedup_suppresses_early_stop (PSG-006)            | F     | RED, E2E   |
//! | test_BC_2_16_002_plan_shape_gate_bare_projection_early_stop_fires (PSG-007)            | —     | GREEN, E2E |
//! | test_BC_2_16_002_plan_shape_gate_order_by_limit_early_stop_fires (PSG-008)             | —     | GREEN, E2E |
//! | test_BC_2_16_002_plan_shape_gate_having_suppresses_early_stop (PSG-009)                | D     | RED, unit  |
//! | test_BC_2_16_002_plan_shape_gate_nested_agg_in_scalar_suppresses_early_stop (PSG-010)  | A rev | RED, E2E   |
//! | test_BC_2_16_002_plan_shape_gate_order_by_aggregate_suppresses_early_stop (PSG-011)    | A rev | RED, E2E   |
//! | test_BC_2_16_002_plan_shape_gate_window_function_suppresses_early_stop (PSG-012)       | A rev | RED, unit  |
//! | test_BC_2_16_002_plan_shape_gate_filter_mode_where_suppresses_early_stop (PSG-013)     | G rev | RED, E2E   |
//! | test_BC_2_16_002_plan_shape_gate_pipe_where_suppresses_early_stop (PSG-014)            | G rev | RED, E2E   |
//! | test_BC_2_16_002_plan_shape_gate_non_equality_sql_where_suppresses_early_stop (PSG-015)| G rev | RED, E2E   |
//! | test_BC_2_16_002_plan_shape_gate_sql_join_suppresses_early_stop (PSG-016)              | H     | RED, E2E   |
//! | test_BC_2_16_002_plan_shape_gate_pipe_tail_suppresses_early_stop (PSG-017)             | I     | RED, E2E   |
//! | test_BC_2_16_002_plan_shape_gate_pipe_join_suppresses_early_stop (PSG-018)             | J     | RED, E2E   |
//! | test_BC_2_16_002_plan_shape_gate_conservative_default_suppresses_early_stop (PSG-019)  | def   | RED, unit  |
//!
//! Tests marked "unit" live in `crates/prism-query/src/materialization.rs`
//! (module `plan_shape_gate_unit_tests`) and carry SAP-3 rule 3 reachability comments.

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
///
/// F-LENSB-P13-002 hardening: `fetch_count` increments on every `fetch()` call.
/// Tests that assert only `last_limit == 0` must ALSO assert `fetch_count >= 1`
/// so the init value of `0` cannot vacuously satisfy `last_limit == 0` when
/// the adapter was never actually called.
struct PlanShapeGateMockAdapter {
    /// The last `params.limit` received — lets tests assert on gate output.
    last_limit: Arc<AtomicU64>,
    /// Total number of `fetch()` calls received (hardening counter).
    /// A value of 0 after the pipeline completes means the adapter was never
    /// invoked — any `last_limit == 0` assertion would be vacuously true.
    fetch_count: Arc<AtomicU64>,
    /// 300 rows (3 pages × 100) — returned when `params.limit == 0`.
    full_batches: Vec<RecordBatch>,
    /// 100 rows (page 1 only) — returned when `params.limit > 0`.
    page1_batches: Vec<RecordBatch>,
}

impl std::fmt::Debug for PlanShapeGateMockAdapter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PlanShapeGateMockAdapter")
            .field("last_limit", &self.last_limit.load(Ordering::Relaxed))
            .field("fetch_count", &self.fetch_count.load(Ordering::Relaxed))
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
        // F-LENSB-P13-002: increment fetch_count so tests can assert >= 1.
        // A final fetch_count of 0 means the adapter was never invoked and any
        // `last_limit == 0` assertion would be vacuously satisfied by init state.
        self.fetch_count.fetch_add(1, Ordering::SeqCst);

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
/// Returns the context, the shared `last_limit` arc, and the `fetch_count` arc so
/// individual tests can assert on both:
/// - the `params.limit` value the pipeline wired to the adapter, AND
/// - that the adapter was actually called (fetch_count >= 1, per F-LENSB-P13-002).
///
/// Tests that do not need `fetch_count` should bind it as `_fetch_count` or `_`.
fn plan_gate_mat_ctx() -> (MaterializationContext, Arc<AtomicU64>, Arc<AtomicU64>) {
    let last_limit = Arc::new(AtomicU64::new(0));
    let fetch_count = Arc::new(AtomicU64::new(0));

    // Data pages — each 100 rows with a distinct status value.
    let page1 = make_status_batch("page1", 100);
    let page2 = make_status_batch("page2", 100);
    let page3 = make_status_batch("page3", 100);

    let adapter = Arc::new(PlanShapeGateMockAdapter {
        last_limit: Arc::clone(&last_limit),
        fetch_count: Arc::clone(&fetch_count),
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

    (mat_ctx, last_limit, fetch_count)
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
    let (mut mat_ctx, last_limit, _fetch_count) = plan_gate_mat_ctx();
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

/// RG-PSG-002 — AC-007 Condition B: non-empty GROUP BY clause (isolated)
///
/// `SELECT status FROM mock_events GROUP BY status LIMIT 25`
///
/// No aggregate in SELECT (A=false), no DISTINCT (C=false) — Condition B in
/// isolation. GROUP BY without an aggregate is valid SQL (equivalent to DISTINCT
/// on the grouped columns) and exercises only Condition B.
///
/// Before gate: 100 rows (all "page1") → 1 distinct status value.
/// After gate: 300 rows (3 pages) → 3 distinct statuses: page1, page2, page3.
///
/// SAP-3: reaches `ast_is_reducing_plan` through `run_materialization_pipeline`.
/// F-LENSB-MED-001 fix: prior query used COUNT(*) (Condition A), which masked B.
#[tokio::test]
async fn test_BC_2_16_002_plan_shape_gate_group_by_suppresses_early_stop() {
    let (mut mat_ctx, last_limit, _fetch_count) = plan_gate_mat_ctx();
    let session_ctx =
        build_session_context(QUERY_MEMORY_POOL_BYTES).expect("build_session_context");

    // Condition B isolated: GROUP BY without aggregate.
    let query = "SELECT status FROM mock_events GROUP BY status LIMIT 25";
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
    let (mut mat_ctx, last_limit, _fetch_count) = plan_gate_mat_ctx();
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
    let (mut mat_ctx, last_limit, _fetch_count) = plan_gate_mat_ctx();
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
    let (mut mat_ctx, last_limit, _fetch_count) = plan_gate_mat_ctx();
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
    let (mut mat_ctx, last_limit, _fetch_count) = plan_gate_mat_ctx();
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
    let (mut mat_ctx, last_limit, _fetch_count) = plan_gate_mat_ctx();
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
    let (mut mat_ctx, last_limit, _fetch_count) = plan_gate_mat_ctx();
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
// PSG-009 — Condition D: HAVING (in-crate unit test in materialization.rs)
// ===========================================================================
//
// PSG-009 is an in-crate unit test in `crates/prism-query/src/materialization.rs`
// (module `plan_shape_gate_unit_tests`) because Condition D (HAVING) already
// exists in v1.2, making an isolated E2E test vacuously green against v1.2.
// The in-crate test uses the v1.3 signature change as the Red Gate mechanism:
// calling `ast_is_reducing_plan(&ast)` (no `where_filters` arg) fails to compile
// against v1.2, making the test RED. See F-LENSB-MED-001 isolation fix.

// ===========================================================================
// PSG-010 — Condition A revised: nested aggregate inside FuncCall::Scalar args
// ===========================================================================

/// RG-PSG-010 — AC-007 Condition A revised: `FuncCall::Scalar` args recursion
///
/// `SELECT severity_label(max(status)) FROM mock_events LIMIT 5`
///
/// `severity_label(max(status))` is a `FuncCall::Scalar` whose args contain
/// `FuncCall::Aggregate(max)`. v1.2's `expr_contains_aggregate` stops at the
/// outer `FuncCall` (`Expr::FuncCall(_) => false`) without recursing into args.
/// v1.3's `expr_contains_aggregate_or_window` recurses into `FuncCall::Scalar::args`
/// and detects the nested `max` aggregate.
///
/// Because `severity_label` is an unknown UDF, DataFusion will error at execution.
/// However the adapter is called BEFORE DataFusion runs (during the fan-out phase),
/// so `last_limit` is set correctly by the time the DataFusion error is returned.
///
/// RED:  v1.2 misses the nested aggregate → `fetch_limit=5` → `last_limit=5`.
/// GREEN: v1.3 detects it → `fetch_limit=0` → `last_limit=0`.
///
/// SAP-3: query is parsed from a real SQL string. The scalar UDF wrapper is
/// grammar-reachable (any function call can wrap an aggregate).
#[tokio::test]
async fn test_BC_2_16_002_plan_shape_gate_nested_agg_in_scalar_suppresses_early_stop() {
    let (mut mat_ctx, last_limit, fetch_count) = plan_gate_mat_ctx();
    let session_ctx =
        build_session_context(QUERY_MEMORY_POOL_BYTES).expect("build_session_context");

    // Condition A revised: scalar UDF wrapping aggregate.
    let query = "SELECT severity_label(max(status)) FROM mock_events LIMIT 5";
    // Do NOT .expect() — DataFusion will error (unknown UDF), but last_limit
    // is already set by the adapter call before DataFusion runs.
    let _result = run_materialization_pipeline(query, &opts(5), &mut mat_ctx, &session_ctx).await;

    // F-LENSB-P13-002 hardening: adapter MUST have been called at least once.
    // A fetch_count of 0 means the pipeline short-circuited before reaching the
    // adapter — the `last_limit == 0` assertion below would be vacuously satisfied
    // by the AtomicU64 init state, not by the gate's suppression.
    let fc = fetch_count.load(Ordering::SeqCst);
    assert!(
        fc >= 1,
        "PSG-010 (Condition A revised — nested aggregate in Scalar args): \
         adapter must have been called at least once (fetch_count={fc}). \
         A fetch_count of 0 means last_limit==0 is the init value, not gate evidence."
    );

    // PRIMARY mechanism assertion: gate must set fetch_limit=0.
    // RED: last_limit=5 (v1.2 misses nested aggregate, no recursion into Scalar args).
    // GREEN: last_limit=0 (v1.3 recurses, detects max → suppresses early-stop).
    let seen_limit = last_limit.load(Ordering::SeqCst);
    assert_eq!(
        seen_limit, 0,
        "PSG-010 (Condition A revised — nested aggregate in Scalar args): gate must \
         suppress early-stop (fetch_limit=0); adapter saw params.limit={seen_limit}. \
         If 5, v1.2 `expr_contains_aggregate` does not recurse into `FuncCall::Scalar::args`."
    );
}

// ===========================================================================
// PSG-011 — Condition A revised: aggregate in ORDER BY
// ===========================================================================

/// RG-PSG-011 — AC-007 Condition A revised: aggregate expression in ORDER BY
///
/// `SELECT * FROM mock_events ORDER BY MAX(status) LIMIT 5`
///
/// v1.2's `expr_contains_aggregate` only checks `select_clause.items`. It does not
/// check ORDER BY expressions. v1.3's `ast_is_reducing_plan` extends Condition A to
/// also check ORDER BY expression items for aggregates/windows.
///
/// DataFusion may error at execution (aggregate in ORDER BY without GROUP BY),
/// but the adapter is called in the fan-out phase before DataFusion runs.
///
/// RED:  v1.2 skips ORDER BY scan → `fetch_limit=5` → `last_limit=5`.
/// GREEN: v1.3 scans ORDER BY → detects MAX → `fetch_limit=0` → `last_limit=0`.
///
/// SAP-3: `ORDER BY MAX(status)` is grammar-reachable (standard SQL).
#[tokio::test]
async fn test_BC_2_16_002_plan_shape_gate_order_by_aggregate_suppresses_early_stop() {
    let (mut mat_ctx, last_limit, fetch_count) = plan_gate_mat_ctx();
    let session_ctx =
        build_session_context(QUERY_MEMORY_POOL_BYTES).expect("build_session_context");

    // Condition A revised: aggregate in ORDER BY.
    let query = "SELECT * FROM mock_events ORDER BY MAX(status) LIMIT 5";
    // DataFusion may reject aggregate in ORDER BY without GROUP BY, but
    // last_limit is set before DataFusion executes.
    let _result = run_materialization_pipeline(query, &opts(5), &mut mat_ctx, &session_ctx).await;

    // F-LENSB-P13-002 hardening: adapter must have been invoked.
    let fc = fetch_count.load(Ordering::SeqCst);
    assert!(
        fc >= 1,
        "PSG-011 (Condition A revised — aggregate in ORDER BY): \
         adapter must have been called at least once (fetch_count={fc}). \
         A fetch_count of 0 means last_limit==0 is the init value, not gate evidence."
    );

    let seen_limit = last_limit.load(Ordering::SeqCst);
    assert_eq!(
        seen_limit, 0,
        "PSG-011 (Condition A revised — aggregate in ORDER BY): gate must suppress \
         early-stop (fetch_limit=0); adapter saw params.limit={seen_limit}. \
         If 5, v1.2 only checks SELECT items for aggregates, missing ORDER BY."
    );
}

// ===========================================================================
// PSG-012 — Condition A revised: FuncCall::Window (in-crate unit test)
// ===========================================================================
//
// PSG-012 is an in-crate unit test in `crates/prism-query/src/materialization.rs`
// (module `plan_shape_gate_unit_tests`). `FuncCall::Window` (S-3.06 stub with no
// fields) is not producible from the PrismQL grammar (no OVER clause syntax exists
// yet), so a SAP-3 rule 3 defense-in-depth in-crate test with a manually constructed
// AST is the correct form. The Red Gate is the v1.3 signature change.

// ===========================================================================
// PSG-013 — Condition G revised: Filter mode non-temporal predicate
// ===========================================================================

/// RG-PSG-013 — AC-007 Condition G revised: `Ast::Filter` non-temporal predicate
///
/// `mock_events | status = 'page2'` — Filter mode.
///
/// v1.2's `ast_is_reducing_plan` uses `_ => false` as the catch-all, which covers
/// `Ast::Filter`. Any non-SQL, non-Pipe AST shape — including Filter mode — returns
/// false. This is the v1.2 BUG for Condition G in Filter mode.
///
/// v1.3's `has_client_side_where(&ast)` handles all 4 AST modes including Filter.
/// A non-temporal predicate in Filter mode suppresses early-stop.
///
/// RED:  `_ => false` → `fetch_limit=25` → `last_limit=25`.
/// GREEN: `has_client_side_where` detects Filter predicate → `fetch_limit=0` → `last_limit=0`.
///
/// Note: our mock adapter does not apply the filter predicate (it returns all rows
/// for the given limit), so the assertion is on `last_limit` only, not row count.
///
/// SAP-3: `mock_events | status = 'page2'` is Filter mode, grammar-reachable.
#[tokio::test]
async fn test_BC_2_16_002_plan_shape_gate_filter_mode_where_suppresses_early_stop() {
    let (mut mat_ctx, last_limit, fetch_count) = plan_gate_mat_ctx();
    let session_ctx =
        build_session_context(QUERY_MEMORY_POOL_BYTES).expect("build_session_context");

    // Condition G revised: Filter mode with non-temporal predicate.
    let query = "mock_events | status = 'page2'";
    // Pipeline result may succeed (returning rows of any status from mock).
    // Assertion is on last_limit — the effect that matters for early-stop correctness.
    let _result = run_materialization_pipeline(query, &opts(25), &mut mat_ctx, &session_ctx).await;

    // F-LENSB-P13-002 hardening: adapter must have been invoked.
    let fc = fetch_count.load(Ordering::SeqCst);
    assert!(
        fc >= 1,
        "PSG-013 (Condition G revised — Filter mode): \
         adapter must have been called at least once (fetch_count={fc}). \
         A fetch_count of 0 means last_limit==0 is the init value, not gate evidence."
    );

    let seen_limit = last_limit.load(Ordering::SeqCst);
    assert_eq!(
        seen_limit, 0,
        "PSG-013 (Condition G revised — Filter mode): gate must suppress early-stop \
         (fetch_limit=0); adapter saw params.limit={seen_limit}. \
         If 25, v1.2 `_ => false` catch-all does not handle Ast::Filter mode."
    );
}

// ===========================================================================
// PSG-014 — Condition G revised: Pipe WHERE non-temporal predicate
// ===========================================================================

/// RG-PSG-014 — AC-007 Condition G revised: `PipeStage::Where` non-temporal
///
/// `mock_events | where status = 'page2'` — pipe WHERE, NO explicit LIMIT clause.
///
/// v1.2 only checks `PipeStage::Stats` and `PipeStage::Dedup` inside `Ast::Pipe`.
/// It does not detect `PipeStage::Where` as a client-side reducing operation when
/// the predicate is non-temporal. v1.3's `has_client_side_where` checks for any
/// pipe WHERE stage with a non-temporal predicate.
///
/// ## F-R13-MED-001 remediation — raw filtered count (pre-cap layer fix)
///
/// The CORRECT postcondition for `run_materialization_pipeline` is to return the
/// FULL filtered result set (100 page2 rows) — the tool-level cap (`is_truncated`
/// signal) belongs in `engine.rs::execute` Step 6, NOT inside materialization.
///
/// The buggy `truncate_result_to_limit` pre-cap in `run_materialization_pipeline`
/// silently caps the output to `options.limit` (25), making `engine.rs` see
/// `total_rows = 25 ≤ 25`, so `is_truncated = false` — the truncation signal
/// is lost.  The implementer's task is to REMOVE that pre-cap.
///
/// Assertion semantics:
///   RED  (pre-cap present, current): 100 page2 rows → capped to 25 → 25 ≠ 100 FAIL.
///   GREEN (pre-cap removed):         100 page2 rows returned as-is → 100 == 100 PASS.
///
/// Note: `opts(25).limit = Some(25)` but the query has NO explicit `LIMIT k` clause,
/// so DataFusion does not apply a LIMIT internally. Only the pre-cap (to be removed)
/// can reduce the 100 filtered rows below 100. Do NOT change this test to carry an
/// explicit LIMIT clause — that would route through DataFusion's cap and be correct.
///
/// SAP-3: `mock_events | where status = 'page2'` is grammar-reachable pipe syntax.
#[tokio::test]
async fn test_BC_2_16_002_plan_shape_gate_pipe_where_suppresses_early_stop() {
    let (mut mat_ctx, last_limit, _fetch_count) = plan_gate_mat_ctx();
    let session_ctx =
        build_session_context(QUERY_MEMORY_POOL_BYTES).expect("build_session_context");

    // Condition G revised: pipe WHERE with non-temporal predicate, no explicit LIMIT.
    let query = "mock_events | where status = 'page2'";
    let out = run_materialization_pipeline(query, &opts(25), &mut mat_ctx, &session_ctx)
        .await
        .expect("PSG-014: pipeline must not error for pipe WHERE query");

    // PRIMARY behavioral assertion — RAW filtered count, NOT the tool-level cap.
    //
    // F-R13-MED-001: `run_materialization_pipeline` MUST return the full filtered
    // set (100 page2 rows) after the pre-cap is removed.  The tool-level
    // is_truncated / returned_results computation belongs in engine.rs Step 6.
    //
    // RED  (pre-cap present): gate suppresses → 300 rows fetched → 100 page2 rows →
    //   truncate_result_to_limit caps to 25 → row_count = 25 ≠ 100 FAIL.
    // GREEN (pre-cap removed): 100 page2 rows returned unchanged → row_count = 100 PASS.
    let row_count = total_rows(&out.batches);
    assert_eq!(
        row_count, 100,
        "PSG-014 (Condition G revised — pipe WHERE): run_materialization_pipeline must \
         return the RAW filtered count (100 page2 rows) without pre-capping to \
         opts.limit; got {row_count}. \
         If 25, truncate_result_to_limit is still applying the tool-level cap inside \
         materialization — that cap must move to engine.rs Step 6 (F-R13-MED-001). \
         If 0, the gate is not yet implemented (fetch_limit=25 → only page1 fetched, \
         none match page2)."
    );

    // SECONDARY mechanism assertion: gate must have suppressed early-stop.
    let seen_limit = last_limit.load(Ordering::SeqCst);
    assert_eq!(
        seen_limit, 0,
        "PSG-014 (Condition G revised — pipe WHERE): gate must suppress early-stop \
         (fetch_limit=0); adapter saw params.limit={seen_limit}."
    );
}

// ===========================================================================
// PSG-015 — Condition G revised: SQL WHERE LIKE (non-equality client-side filter)
// ===========================================================================

/// RG-PSG-015 — AC-007 Condition G revised: non-equality SQL WHERE predicate
///
/// `SELECT * FROM mock_events WHERE status LIKE '%page2%' LIMIT 100`
///
/// v1.2's `where_filters` (Condition G) only captures equality predicates passed
/// as external query parameters, NOT predicates embedded in the SQL AST's WHERE
/// clause. A LIKE predicate in the SQL AST is not in `where_filters` and is thus
/// invisible to v1.2's Condition G.
///
/// v1.3's `has_client_side_where` inspects the SQL `WHERE` clause in the AST
/// directly and detects non-temporal predicates including LIKE.
///
/// DataFusion applies the LIKE predicate to the fetched rows.
///   RED: 100 rows of "page1" → WHERE status LIKE '%page2%' → 0 rows.
///   GREEN: 300 rows → LIKE filter → 100 rows of "page2" → LIMIT 100 → 100 rows.
///
/// SAP-3: `WHERE status LIKE '%page2%'` is grammar-reachable standard SQL.
#[tokio::test]
async fn test_BC_2_16_002_plan_shape_gate_non_equality_sql_where_suppresses_early_stop() {
    let (mut mat_ctx, last_limit, _fetch_count) = plan_gate_mat_ctx();
    let session_ctx =
        build_session_context(QUERY_MEMORY_POOL_BYTES).expect("build_session_context");

    // Condition G revised: SQL WHERE LIKE predicate (non-equality).
    let query = "SELECT * FROM mock_events WHERE status LIKE '%page2%' LIMIT 100";
    let out = run_materialization_pipeline(query, &opts(100), &mut mat_ctx, &session_ctx)
        .await
        .expect("PSG-015: pipeline must not error for SQL LIKE query");

    // PRIMARY behavioral assertion.
    // RED: 0 rows (100 page1 fetched; none match LIKE '%page2%').
    // GREEN: 100 rows (300 fetched; 100 page2 rows match; LIMIT 100 applied).
    let row_count = total_rows(&out.batches);
    assert_eq!(
        row_count, 100,
        "PSG-015 (Condition G revised — SQL LIKE): must return 100 rows when gate \
         suppresses early-stop (300 rows fetched, LIKE filter yields 100 page2 rows); \
         got {row_count}. \
         If 0, v1.2 does not inspect SQL WHERE AST for non-temporal predicates."
    );

    // SECONDARY mechanism assertion.
    let seen_limit = last_limit.load(Ordering::SeqCst);
    assert_eq!(
        seen_limit, 0,
        "PSG-015 (Condition G revised — SQL LIKE): gate must suppress early-stop \
         (fetch_limit=0); adapter saw params.limit={seen_limit}."
    );
}

// ===========================================================================
// PSG-016 — Condition H: SQL JOIN suppresses early-stop
// ===========================================================================

/// RG-PSG-016 — AC-007 Condition H: SQL JOIN present
///
/// `SELECT a.status FROM mock_events a JOIN mock_events b ON a.status = b.status LIMIT 5`
///
/// v1.2 does not check `sql.joins`. v1.3 detects `!sql.joins.is_empty()` as
/// Condition H.
///
/// DataFusion may error on the self-join with aliases, but the adapter is called
/// before DataFusion runs and `last_limit` is set correctly.
///
/// RED:  `sql.joins` not checked → `fetch_limit=5` → `last_limit=5`.
/// GREEN: Condition H detected → `fetch_limit=0` → `last_limit=0`.
///
/// SAP-3: SQL JOIN is grammar-reachable (standard SQL syntax).
#[tokio::test]
async fn test_BC_2_16_002_plan_shape_gate_sql_join_suppresses_early_stop() {
    let (mut mat_ctx, last_limit, fetch_count) = plan_gate_mat_ctx();
    let session_ctx =
        build_session_context(QUERY_MEMORY_POOL_BYTES).expect("build_session_context");

    // Condition H: SQL JOIN.
    let query =
        "SELECT a.status FROM mock_events a JOIN mock_events b ON a.status = b.status LIMIT 5";
    // DataFusion may reject the self-join or succeed; assertion is on last_limit.
    let _result = run_materialization_pipeline(query, &opts(5), &mut mat_ctx, &session_ctx).await;

    // F-LENSB-P13-002 hardening: adapter must have been invoked.
    let fc = fetch_count.load(Ordering::SeqCst);
    assert!(
        fc >= 1,
        "PSG-016 (Condition H — SQL JOIN): \
         adapter must have been called at least once (fetch_count={fc}). \
         A fetch_count of 0 means last_limit==0 is the init value, not gate evidence."
    );

    let seen_limit = last_limit.load(Ordering::SeqCst);
    assert_eq!(
        seen_limit, 0,
        "PSG-016 (Condition H — SQL JOIN): gate must suppress early-stop \
         (fetch_limit=0); adapter saw params.limit={seen_limit}. \
         If 5, v1.2 does not check `sql.joins` for Condition H."
    );
}

// ===========================================================================
// PSG-017 — Condition I: PipeStage::Tail suppresses early-stop
// ===========================================================================

/// RG-PSG-017 — AC-007 Condition I: `PipeStage::Tail` present
///
/// `mock_events | tail 250` with `opts(300)`.
///
/// `| tail N` requests the last N rows — the pipeline must fetch ALL pages to
/// guarantee correctness. v1.2 only checks `PipeStage::Stats` and `PipeStage::Dedup`
/// in the Pipe arm; `PipeStage::Tail` returns false → early-stop fires.
/// v1.3 adds Condition I: `PipeStage::Tail` → returns true → suppresses.
///
/// `opts(300)` ensures LIMIT 300 does not mask the row count difference:
///   RED:  `fetch_limit=300` → mock returns 100 rows (limit > 0) → tail applied
///         to 100 rows → ≤ 100 rows in result.
///   GREEN: `fetch_limit=0` → mock returns 300 rows → tail 250 → 250 rows.
///
/// DataFusion may error if the pipe tail emitter is not yet implemented.
/// `last_limit` is set before DataFusion runs.
///
/// SAP-3: `mock_events | tail 250` is grammar-reachable pipe syntax.
#[tokio::test]
async fn test_BC_2_16_002_plan_shape_gate_pipe_tail_suppresses_early_stop() {
    let (mut mat_ctx, last_limit, fetch_count) = plan_gate_mat_ctx();
    let session_ctx =
        build_session_context(QUERY_MEMORY_POOL_BYTES).expect("build_session_context");

    // Condition I: PipeStage::Tail.
    let query = "mock_events | tail 250";
    // DataFusion may error if the tail emitter is not implemented; assertion
    // is on last_limit which is set before DataFusion runs.
    let _result = run_materialization_pipeline(query, &opts(300), &mut mat_ctx, &session_ctx).await;

    // F-LENSB-P13-002 hardening: adapter must have been invoked.
    let fc = fetch_count.load(Ordering::SeqCst);
    assert!(
        fc >= 1,
        "PSG-017 (Condition I — PipeStage::Tail): \
         adapter must have been called at least once (fetch_count={fc}). \
         A fetch_count of 0 means last_limit==0 is the init value, not gate evidence."
    );

    let seen_limit = last_limit.load(Ordering::SeqCst);
    assert_eq!(
        seen_limit, 0,
        "PSG-017 (Condition I — PipeStage::Tail): gate must suppress early-stop \
         (fetch_limit=0); adapter saw params.limit={seen_limit}. \
         If 300, v1.2 pipe arm does not detect PipeStage::Tail."
    );
}

// ===========================================================================
// PSG-018 — Condition J: PipeStage::Join suppresses early-stop
// ===========================================================================

/// RG-PSG-018 — AC-007 Condition J (defensive): `PipeStage::Join` present
///
/// `mock_events | join inner mock_events on status` with `opts(25)`.
///
/// `PipeStage::Join` in a pipe query requires fetching complete data from all
/// joined sources. v1.2 only checks `PipeStage::Stats` and `PipeStage::Dedup`;
/// `PipeStage::Join` returns false. v1.3 adds Condition J (defensive).
///
/// The join emitter may error during DataFusion execution. The adapter is called
/// during the fan-out phase before DataFusion runs, so `last_limit` is set.
///
/// RED:  v1.2 misses PipeStage::Join → `fetch_limit=25` → `last_limit=25`.
/// GREEN: Condition J detected → `fetch_limit=0` → `last_limit=0`.
///
/// SAP-3: `mock_events | join inner mock_events on status` is grammar-reachable
/// pipe syntax (PipeStage::Join has grammar productions unlike FuncCall::Window).
#[tokio::test]
async fn test_BC_2_16_002_plan_shape_gate_pipe_join_suppresses_early_stop() {
    let (mut mat_ctx, last_limit, fetch_count) = plan_gate_mat_ctx();
    let session_ctx =
        build_session_context(QUERY_MEMORY_POOL_BYTES).expect("build_session_context");

    // Condition J: PipeStage::Join.
    let query = "mock_events | join inner mock_events on status";
    // DataFusion may error on the pipe join emitter; assertion is on last_limit.
    let _result = run_materialization_pipeline(query, &opts(25), &mut mat_ctx, &session_ctx).await;

    // F-LENSB-P13-002 hardening: adapter must have been invoked.
    let fc = fetch_count.load(Ordering::SeqCst);
    assert!(
        fc >= 1,
        "PSG-018 (Condition J — PipeStage::Join): \
         adapter must have been called at least once (fetch_count={fc}). \
         A fetch_count of 0 means last_limit==0 is the init value, not gate evidence."
    );

    let seen_limit = last_limit.load(Ordering::SeqCst);
    assert_eq!(
        seen_limit, 0,
        "PSG-018 (Condition J — PipeStage::Join): gate must suppress early-stop \
         (fetch_limit=0); adapter saw params.limit={seen_limit}. \
         If 25, v1.2 pipe arm does not detect PipeStage::Join."
    );
}

// ===========================================================================
// PSG-019 — Conservative default (in-crate unit test in materialization.rs)
// ===========================================================================
//
// PSG-019 is an in-crate unit test in `crates/prism-query/src/materialization.rs`
// (module `plan_shape_gate_unit_tests`). The `_ => true` conservative default for
// unknown Ast/PipeStage variants cannot be exercised from a grammar query string
// (the grammar only produces known variants). The in-crate test uses the v1.3
// signature change as the Red Gate mechanism and carries a SAP-3 rule 3 comment.
