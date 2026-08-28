//! Plan-Shape Gate Red Gate Tests — RG-PSG-001..RG-PSG-019, RG-PSG-021..RG-PSG-024
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
//! | (PSG-020 is in execute_integration_tests.rs — F-R13-MED-002 truncation signal)         | —     | RED, E2E   |
//! | test_psg_filter_mode_temporal_suppresses_early_stop (PSG-021)                           | G v1.5| RED, E2E   |
//! | test_psg_pipe_where_temporal_suppresses_early_stop (PSG-022)                            | G v1.5| RED, E2E   |
//! | test_psg_sql_eq_temporal_suppresses_early_stop (PSG-023)                                | G v1.5| RED, E2E   |
//! | test_psg_sql_non_index_temporal_suppresses_early_stop (PSG-024)                         | G v1.5| RED, E2E   |
//! | (PSG-025 is in execute_integration_tests.rs — exact-limit is_truncated soundness)       | —     | RED, E2E   |
//! | test_psg_relative_temporal_now_interval_suppresses_early_stop (PSG-029)               | G v1.6| RED, E2E   |
//! | test_psg_rg030_redundant_lower_bound_suppresses_early_stop (PSG-030)                  | G v1.7| RED, E2E   |
//! | test_psg_rg031_ocsf_arrow_name_permits_early_stop (PSG-031)                           | G v1.7| RED, E2E   |
//! | test_psg_canonical_time_window_still_permitted (regression guard)                     | —     | GREEN, E2E |
//!
//! PSG-030 and PSG-031 are the **round-17 ADR-060 v1.7 soundness remediation** tests.
//! PSG-030 asserts that two same-direction lower temporal bounds (`> X AND > Y` on the same
//! INDEX col) are SUPPRESSED — the v1.5/v1.6 AND-arm `.all()` wrongly PERMITs this case
//! because `extract_time_bounds_from_predicate` is first-wins and silently drops the second
//! bound to DataFusion, creating a risk of incorrect LIMIT results (ADR-060 §D8.7, HIGH-001).
//! PSG-031 asserts that a temporal predicate on the OCSF-flattened Arrow name (`time`) is
//! PERMITTED — the v1.5/v1.6 gate only registered `col.name` ("timestamp") in
//! `datetime_index_cols`, missing the OCSF flattened name, so it wrongly SUPPRESSED for
//! `WHERE time > ...` on an `ocsf_column_naming=true` sensor (ADR-060 §D8.9, MED-001).
//! Both require a `resolved_spec_map` wired into `MaterializationContext` so
//! `datetime_index_cols` is non-empty; tests use `plan_gate_mat_ctx_with_spec(make_psg_spec_map(...))`.
//!
//! Tests marked "unit" live in `crates/prism-query/src/materialization.rs`
//! (module `plan_shape_gate_unit_tests`) and carry SAP-3 rule 3 reachability comments.
//!
//! PSG-021..PSG-024 are the **round-15 permitted-path soundness remediation** tests.
//! They assert that temporal predicates in Filter and Pipe-WHERE modes — as well as
//! SQL Eq temporal and non-INDEX datetime predicates — correctly suppress early-stop.
//! ADR-060 v1.4's temporal exemption in `has_client_side_where` is UNSOUND for these
//! modes because the predicates are applied client-side by DataFusion, not pushed
//! server-side. These tests are RED after Task 12 until the v1.5 soundness fix lands.
//!
//! PSG-029 is the **round-16 relative-temporal via `QueryEngine::execute`** test.
//! It verifies `now() - interval '7d'` suppresses early-stop end-to-end through
//! the public `QueryEngine::execute` surface (SAP-3 — end-to-end from public surface).
//! PSG-021..024 exercise the same gate via `run_materialization_pipeline` directly;
//! PSG-029 adds the missing public-surface path for the relative `now()` arithmetic
//! variant (AC-008(c) of S-ENGINE-LIMIT-EARLY-STOP-001).

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
use prism_credentials::{namespace::CredentialName, CredentialStore};
use prism_ocsf::OcsfNormalizer;
use prism_query::{
    engine::{QueryEngine, QueryEngineConfig, QueryOptions},
    materialization::{run_materialization_pipeline, MaterializationContext},
    memory::{build_session_context, QUERY_MEMORY_POOL_BYTES},
    scoping::ClientRegistry,
};
use prism_sensors::{
    adapter::{FetchOutput, QueryParams, SensorAdapter, SensorError, SensorSpec},
    auth::SensorAuth,
    AdapterRegistry, CredentialResolver,
};
use secrecy::SecretString;

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
    /// Sensor type reported by `sensor_type()`. Defaults to `"mock"` for existing
    /// tests; set to `"armis"` or `"crowdstrike"` for cross-sensor source-scope tests
    /// (PSG-032, PSG-033, PSG-030b) where the source table prefix must match.
    sensor_type_id: SensorId,
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
            .field("sensor_type_id", &self.sensor_type_id)
            .field("last_limit", &self.last_limit.load(Ordering::Relaxed))
            .field("fetch_count", &self.fetch_count.load(Ordering::Relaxed))
            .finish()
    }
}

#[async_trait]
impl SensorAdapter for PlanShapeGateMockAdapter {
    fn sensor_type(&self) -> SensorId {
        self.sensor_type_id.clone()
    }

    fn sensor_name(&self) -> &'static str {
        "mock"
    }

    async fn fetch(
        &self,
        _spec: &SensorSpec,
        params: &QueryParams,
        _auth: &dyn SensorAuth,
    ) -> Result<FetchOutput, SensorError> {
        // F-LENSB-P13-002: increment fetch_count so tests can assert >= 1.
        // A final fetch_count of 0 means the adapter was never invoked and any
        // `last_limit == 0` assertion would be vacuously satisfied by init state.
        self.fetch_count.fetch_add(1, Ordering::SeqCst);

        // Record the fetch_limit the pipeline wired into params.limit.
        self.last_limit.store(params.limit, Ordering::SeqCst);

        if params.limit == 0 {
            // fetch_limit == 0 means early-stop is suppressed — return all 300 rows.
            Ok(FetchOutput::new(self.full_batches.clone(), false, false))
        } else {
            // fetch_limit > 0 means early-stop is active — return page 1 only.
            Ok(FetchOutput::new(self.page1_batches.clone(), false, false))
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
// PsgNullCredentialStore — no-op CredentialStore for PSG-029 QueryEngine::new
// ---------------------------------------------------------------------------

/// No-op [`CredentialStore`] satisfying the `QueryEngine::new` constructor contract.
///
/// PSG-029 uses `QueryEngine::new` which requires an `Arc<dyn CredentialStore>`.
/// Actual credential storage is never consulted — `StubCredentialResolver`
/// handles sensor auth at the fan-out level.
struct PsgNullCredentialStore;

#[async_trait]
impl CredentialStore for PsgNullCredentialStore {
    async fn get(
        &self,
        _tenant: &OrgSlug,
        _sensor: &str,
        _name: &CredentialName,
    ) -> Result<Option<SecretString>, prism_core::PrismError> {
        Ok(None)
    }

    async fn set(
        &self,
        _tenant: &OrgSlug,
        _sensor: &str,
        _name: &CredentialName,
        _value: SecretString,
    ) -> Result<(), prism_core::PrismError> {
        Ok(())
    }

    async fn delete(
        &self,
        _tenant: &OrgSlug,
        _sensor: &str,
        _name: &CredentialName,
    ) -> Result<bool, prism_core::PrismError> {
        Ok(false)
    }

    async fn list(
        &self,
        _tenant: &OrgSlug,
    ) -> Result<Vec<(String, CredentialName)>, prism_core::PrismError> {
        Ok(vec![])
    }

    async fn exists(
        &self,
        _tenant: &OrgSlug,
        _sensor: &str,
        _name: &CredentialName,
    ) -> Result<bool, prism_core::PrismError> {
        Ok(false)
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
        sensor_type_id: SensorId::from("mock"),
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

// ===========================================================================
// PSG-021 — Temporal-exemption soundness: Filter mode (round-15)
// ===========================================================================

/// RG-PSG-021 — ADR-060 §D8.7 v1.5: `Ast::Filter` temporal predicate must suppress
///
/// `mock_events | timestamp > '2024-01-01T00:00:00Z'`
///
/// ## Soundness Defect (ADR-060 v1.4)
///
/// ADR-060 v1.4's `has_client_side_where` for `Ast::Filter` returns
/// `!is_pushed_temporal_predicate(&f.predicate)`. For a temporal GT comparison,
/// `is_pushed_temporal_predicate` returns `true` → `has_client_side_where` returns
/// `false` → early-stop is NOT suppressed.
///
/// This is UNSOUND: in `Ast::Filter` mode ALL predicates are applied client-side
/// by DataFusion. ADR-033 T1 server-side temporal push-down applies ONLY to
/// `Ast::Sql` queries on INDEX-designated datetime columns — it does NOT cover
/// Filter mode. The v1.4 exemption incorrectly treats filter-mode temporal
/// predicates as if they were pushed server-side, allowing early-stop to fire and
/// truncating results without signalling `is_truncated = true`.
///
/// ## Round-15 Fix (ADR-060 v1.5 §D8.7)
///
/// Remove the temporal exemption from the `Ast::Filter` arm of `has_client_side_where`.
/// Filter-mode predicates are ALWAYS client-side regardless of whether they are
/// temporal. After the fix: `has_client_side_where = true` → gate suppresses →
/// `fetch_limit = 0` → `last_limit = 0`.
///
/// ## RED / GREEN mechanics
///
/// RED (current code — no gate):
///   `fetch_limit = opts.limit = 25 > 0` → `last_limit = 25` ≠ 0.
/// RED (post-Task-12 with v1.4 temporal exemption):
///   `is_pushed_temporal_predicate(GT, Literal::Timestamp)` = true →
///   `has_client_side_where(Ast::Filter)` = false → `fetch_limit = 25` →
///   `last_limit = 25` ≠ 0.
/// GREEN (ADR-060 v1.5 fix applied):
///   Filter-mode temporal exemption removed → `has_client_side_where` = true →
///   `fetch_limit = 0` → `last_limit = 0`.
///
/// `'2024-01-01T00:00:00Z'` parses as `Literal::Timestamp` (RFC-3339 succeeds;
/// no `RawTemporalLiteral` schema-aware validation required).
///
/// SAP-3: `mock_events | timestamp > '2024-01-01T00:00:00Z'` is grammar-reachable
/// Filter mode; runs end-to-end through `run_materialization_pipeline`.
#[tokio::test]
async fn test_psg_filter_mode_temporal_suppresses_early_stop() {
    let (mut mat_ctx, last_limit, fetch_count) = plan_gate_mat_ctx();
    let session_ctx =
        build_session_context(QUERY_MEMORY_POOL_BYTES).expect("build_session_context");

    // Filter-mode temporal GT predicate.
    // Predicate: Compare { op: Gt, lhs: Field("timestamp"), rhs: Literal::Timestamp }.
    // is_pushed_temporal_predicate returns true (range comparison with Timestamp literal).
    // has_client_side_where(Ast::Filter) -> !true = false -> no suppression in v1.4.
    // fetch_limit = opts.limit = 25 > 0 -> adapter sees params.limit = 25.
    //
    // DataFusion may report no 'timestamp' column in mock data — assertion is on
    // last_limit which is recorded before DataFusion runs.
    let query = "mock_events | timestamp > '2024-01-01T00:00:00Z'";
    let _result = run_materialization_pipeline(query, &opts(25), &mut mat_ctx, &session_ctx).await;

    // F-LENSB-P13-002 hardening: adapter must have been called (not vacuous init value).
    let fc = fetch_count.load(Ordering::SeqCst);
    assert!(
        fc >= 1,
        "PSG-021 (filter-mode temporal): adapter must have been called at least once \
         (fetch_count={fc}). A count of 0 means last_limit==0 is the init value, not gate evidence."
    );

    let seen_limit = last_limit.load(Ordering::SeqCst);
    assert_eq!(
        seen_limit, 0,
        "PSG-021 (ADR-060 v1.5 — filter-mode temporal soundness): gate must suppress \
         early-stop for filter-mode temporal predicate (fetch_limit=0); adapter saw \
         params.limit={seen_limit}. If 25, the temporal exemption in \
         has_client_side_where(Ast::Filter) is incorrectly permitting early-stop for a \
         client-side predicate. ADR-033 T1 server-side push-down does NOT apply to \
         Filter mode — filter predicates are always evaluated by DataFusion client-side."
    );
}

// ===========================================================================
// PSG-022 — Temporal-exemption soundness: Pipe-WHERE mode (round-15)
// ===========================================================================

/// RG-PSG-022 — ADR-060 §D8.7 v1.5: `PipeStage::Where` temporal predicate must suppress
///
/// `mock_events | where timestamp > '2024-01-01T00:00:00Z'`
///
/// ## Soundness Defect (ADR-060 v1.4)
///
/// ADR-060 v1.4's `has_client_side_where` for `Ast::Pipe` returns `true` if ANY
/// `PipeStage::Where(pred)` satisfies `!is_pushed_temporal_predicate(pred)`. For a
/// temporal GT predicate, `is_pushed_temporal_predicate` returns `true`, so the
/// stage's contribution is `!true = false`. If all Where stages are temporal,
/// `has_client_side_where` returns `false` → no suppression.
///
/// This is UNSOUND: `PipeStage::Where` predicates are always applied client-side by
/// DataFusion. The pipe-WHERE stage runs AFTER the server fetch, not during it.
/// ADR-033 T1 push-down cannot apply to pipe-WHERE stages.
///
/// ## Round-15 Fix (ADR-060 v1.5 §D8.7)
///
/// Remove the temporal exemption from the `Ast::Pipe` arm's Where-stage check. Any
/// `PipeStage::Where` stage → `has_client_side_where = true` → gate suppresses →
/// `fetch_limit = 0` → `last_limit = 0`.
///
/// ## RED / GREEN mechanics
///
/// RED (current code — no gate):
///   `fetch_limit = opts.limit = 25 > 0` → `last_limit = 25` ≠ 0.
/// RED (post-Task-12 with v1.4 temporal exemption):
///   `is_pushed_temporal_predicate(GT, Literal::Timestamp)` = true →
///   Where-stage contribution = false → `has_client_side_where = false` →
///   `fetch_limit = 25` → `last_limit = 25` ≠ 0.
/// GREEN (ADR-060 v1.5 fix applied):
///   Pipe-WHERE temporal exemption removed → `has_client_side_where = true` →
///   `fetch_limit = 0` → `last_limit = 0`.
///
/// SAP-3: `mock_events | where timestamp > '2024-01-01T00:00:00Z'` is grammar-reachable
/// pipe syntax; runs end-to-end through `run_materialization_pipeline`.
#[tokio::test]
async fn test_psg_pipe_where_temporal_suppresses_early_stop() {
    let (mut mat_ctx, last_limit, fetch_count) = plan_gate_mat_ctx();
    let session_ctx =
        build_session_context(QUERY_MEMORY_POOL_BYTES).expect("build_session_context");

    // Pipe-WHERE temporal GT predicate.
    // PipeStage::Where(Compare { op: Gt, lhs: Field("timestamp"), rhs: Literal::Timestamp }).
    // is_pushed_temporal_predicate returns true -> Where-stage contribution = !true = false.
    // has_client_side_where(Ast::Pipe) -> false (all Where stages are temporal) -> no suppress.
    // fetch_limit = opts.limit = 25 > 0 -> adapter sees params.limit = 25.
    //
    // DataFusion may report no 'timestamp' column in mock data — assertion is on
    // last_limit which is recorded before DataFusion runs.
    let query = "mock_events | where timestamp > '2024-01-01T00:00:00Z'";
    let _result = run_materialization_pipeline(query, &opts(25), &mut mat_ctx, &session_ctx).await;

    // F-LENSB-P13-002 hardening: adapter must have been called.
    let fc = fetch_count.load(Ordering::SeqCst);
    assert!(
        fc >= 1,
        "PSG-022 (pipe-WHERE temporal): adapter must have been called at least once \
         (fetch_count={fc}). A count of 0 means last_limit==0 is the init value, not gate evidence."
    );

    let seen_limit = last_limit.load(Ordering::SeqCst);
    assert_eq!(
        seen_limit, 0,
        "PSG-022 (ADR-060 v1.5 — pipe-WHERE temporal soundness): gate must suppress \
         early-stop for pipe-WHERE temporal predicate (fetch_limit=0); adapter saw \
         params.limit={seen_limit}. If 25, the temporal exemption in \
         has_client_side_where(Ast::Pipe) is incorrectly permitting early-stop for a \
         client-side predicate. PipeStage::Where is applied by DataFusion after the \
         server fetch — it is always client-side."
    );
}

// ===========================================================================
// PSG-023 — Temporal-exemption soundness: SQL Eq (non-range) temporal (round-15)
// ===========================================================================

/// RG-PSG-023 — ADR-060 §D8.7 v1.5: SQL `WHERE timestamp = '<iso>'` must suppress
///
/// `SELECT * FROM mock_events WHERE timestamp = '2024-01-01T00:00:00Z' LIMIT 25`
///
/// ## Soundness Defect
///
/// If `is_pushed_temporal_predicate` accepts equality (`Eq`) operators in addition to
/// range operators (Gt, Lt, Gte, Lte), it would classify `timestamp = 'iso'` as a
/// "purely temporal predicate" and grant the temporal exemption. But `Eq` comparisons
/// on datetime columns are NOT pushed to the sensor server (only range-window queries
/// are pushed via ADR-033 T1). An `Eq` comparison is applied client-side by DataFusion.
///
/// ## Round-15 Fix
///
/// Restrict `is_pushed_temporal_predicate` to range operators ONLY (Gt, Gte, Lt, Lte,
/// Between). Eq on a temporal field → `is_pushed_temporal_predicate = false` →
/// `has_client_side_where = true` → gate suppresses → `fetch_limit = 0`.
///
/// ## RED / GREEN mechanics
///
/// RED (current code — no gate):
///   `fetch_limit = opts.limit = 25 > 0` → `last_limit = 25` ≠ 0.
/// RED (post-Task-12, if `is_pushed_temporal_predicate` includes Eq):
///   `is_pushed_temporal_predicate(Eq, Timestamp)` = true (incorrect) →
///   `has_client_side_where(Ast::Sql)` = false → `fetch_limit = 25` →
///   `last_limit = 25` ≠ 0.
/// GREEN (v1.5 fix — Eq excluded from purely-temporal):
///   `is_pushed_temporal_predicate(Eq, Timestamp)` = false →
///   `has_client_side_where` = true → `fetch_limit = 0` → `last_limit = 0`.
///
/// SAP-3: standard SQL `WHERE timestamp = '...' LIMIT 25` is grammar-reachable.
#[tokio::test]
async fn test_psg_sql_eq_temporal_suppresses_early_stop() {
    let (mut mat_ctx, last_limit, fetch_count) = plan_gate_mat_ctx();
    let session_ctx =
        build_session_context(QUERY_MEMORY_POOL_BYTES).expect("build_session_context");

    // SQL Eq temporal predicate (point equality, not a range comparison).
    // If is_pushed_temporal_predicate includes Eq: returns true -> no suppression.
    // fetch_limit = opts.limit = 25 > 0 -> adapter sees params.limit = 25.
    //
    // DataFusion may report no 'timestamp' column in mock data — assertion is on
    // last_limit which is recorded before DataFusion runs.
    let query = "SELECT * FROM mock_events WHERE timestamp = '2024-01-01T00:00:00Z' LIMIT 25";
    let _result = run_materialization_pipeline(query, &opts(25), &mut mat_ctx, &session_ctx).await;

    // F-LENSB-P13-002 hardening: adapter must have been called.
    let fc = fetch_count.load(Ordering::SeqCst);
    assert!(
        fc >= 1,
        "PSG-023 (SQL Eq temporal): adapter must have been called at least once \
         (fetch_count={fc}). A count of 0 means last_limit==0 is the init value, not gate evidence."
    );

    let seen_limit = last_limit.load(Ordering::SeqCst);
    assert_eq!(
        seen_limit, 0,
        "PSG-023 (ADR-060 v1.5 — SQL Eq temporal soundness): gate must suppress \
         early-stop for SQL timestamp equality predicate (fetch_limit=0); adapter saw \
         params.limit={seen_limit}. If 25, is_pushed_temporal_predicate is incorrectly \
         classifying an Eq comparison as a temporal range push-down candidate. \
         Only GT/GTE/LT/LTE/BETWEEN are pushed server-side (ADR-033 T1); Eq is always \
         applied client-side by DataFusion."
    );
}

// ===========================================================================
// PSG-024 — Temporal-exemption soundness: non-INDEX datetime column (round-15)
// ===========================================================================

/// RG-PSG-024 — ADR-060 §D8.7 v1.5: SQL non-INDEX datetime column must suppress
///
/// `SELECT * FROM mock_events WHERE created_at > '2024-01-01T00:00:00Z' LIMIT 25`
///
/// ## Soundness Defect
///
/// If `is_pushed_temporal_predicate` classifies any range comparison involving a
/// datetime-valued field as "purely temporal" without checking whether the column is
/// declared INDEX in the sensor TOML spec, it would grant the temporal exemption
/// for columns that cannot actually be pushed server-side. ADR-033 T1 server-side
/// temporal push-down requires the column to be designated INDEX; non-INDEX datetime
/// comparisons are applied client-side by DataFusion.
///
/// ## Round-15 Fix
///
/// Add INDEX-awareness to the temporal exemption: only grant the temporal exemption
/// (return false from `has_client_side_where`) when the datetime column is declared
/// INDEX in the resolved sensor spec. For unknown columns or non-INDEX columns,
/// return true (conservative — treat as client-side) → gate suppresses.
///
/// ## Mock arrangement note
///
/// The `PlanShapeGateMockAdapter` provides no sensor-spec column metadata. The round-15
/// fix must use a conservative default of "non-INDEX" for columns whose INDEX status
/// cannot be determined (e.g., columns not present in a resolved sensor spec). This
/// test uses `created_at` as a clearly non-primary datetime field distinct from any
/// INDEX-designated timestamp column. The implementer must ensure `has_client_side_where`
/// defaults to client-side (suppress) for any column it cannot verify is INDEX.
///
/// ## RED / GREEN mechanics
///
/// RED (current code — no gate):
///   `fetch_limit = opts.limit = 25 > 0` → `last_limit = 25` ≠ 0.
/// RED (post-Task-12, if temporal exemption ignores INDEX status):
///   `is_pushed_temporal_predicate(GT, Timestamp on 'created_at')` = true
///   (no INDEX check) → `has_client_side_where = false` → `fetch_limit = 25` →
///   `last_limit = 25` ≠ 0.
/// GREEN (v1.5 fix — conservative non-INDEX default):
///   `created_at` has no INDEX declaration → conservative default = client-side →
///   `has_client_side_where = true` → `fetch_limit = 0` → `last_limit = 0`.
///
/// SAP-3: standard SQL `WHERE created_at > '...' LIMIT 25` is grammar-reachable.
#[tokio::test]
async fn test_psg_sql_non_index_temporal_suppresses_early_stop() {
    let (mut mat_ctx, last_limit, fetch_count) = plan_gate_mat_ctx();
    let session_ctx =
        build_session_context(QUERY_MEMORY_POOL_BYTES).expect("build_session_context");

    // SQL GT predicate on a non-INDEX datetime column ('created_at').
    // 'created_at' is not a primary temporal INDEX column; it cannot be pushed server-side.
    // If is_pushed_temporal_predicate ignores INDEX status: returns true -> no suppression.
    // fetch_limit = opts.limit = 25 > 0 -> adapter sees params.limit = 25.
    //
    // DataFusion may report no 'created_at' column in mock data — assertion is on
    // last_limit which is recorded before DataFusion runs.
    let query = "SELECT * FROM mock_events WHERE created_at > '2024-01-01T00:00:00Z' LIMIT 25";
    let _result = run_materialization_pipeline(query, &opts(25), &mut mat_ctx, &session_ctx).await;

    // F-LENSB-P13-002 hardening: adapter must have been called.
    let fc = fetch_count.load(Ordering::SeqCst);
    assert!(
        fc >= 1,
        "PSG-024 (non-INDEX datetime): adapter must have been called at least once \
         (fetch_count={fc}). A count of 0 means last_limit==0 is the init value, not gate evidence."
    );

    let seen_limit = last_limit.load(Ordering::SeqCst);
    assert_eq!(
        seen_limit, 0,
        "PSG-024 (ADR-060 v1.5 — non-INDEX datetime soundness): gate must suppress \
         early-stop for a non-INDEX datetime column predicate (fetch_limit=0); adapter saw \
         params.limit={seen_limit}. If 25, is_pushed_temporal_predicate is classifying \
         'created_at > timestamp' as a server-side push-down candidate without verifying \
         the column is declared INDEX. ADR-033 T1 push-down requires INDEX designation; \
         unknown or non-INDEX datetime columns must default to client-side (suppress)."
    );
}

// ===========================================================================
// PSG-029 — RG-PSG-029 (AC-008(c)): relative-temporal now()-interval suppresses
//           early-stop — SAP-3 end-to-end via QueryEngine::execute
// ===========================================================================

/// RG-PSG-029 — AC-008(c) — `WHERE <dt> >= now() - interval '7d' LIMIT N` suppress
///
/// SQL: `SELECT * FROM mock_events WHERE timestamp >= now() - interval '7d' LIMIT 25`
/// via `QueryEngine::execute` (SAP-3: end-to-end from public surface).
///
/// PSG-021..PSG-024 exercise the same plan-shape gate via `run_materialization_pipeline`
/// directly.  PSG-029 adds the missing public-surface path for the relative `now()`
/// arithmetic variant — the full `QueryEngine::execute` stack (parser →
/// `check_temporal_literals` → `resolve_clients` → materialization pipeline →
/// `inject_now` → `ast_is_reducing_plan_with_index_cols`).
///
/// ## RED / GREEN mechanics
///
/// RED (current code — story `S-ENGINE-LIMIT-EARLY-STOP-001` is `status: draft`):
///   `ast_is_reducing_plan_with_index_cols` does not exist; the pipeline computes
///   `fetch_limit = opts.limit = 25` unconditionally. Adapter records
///   `params.limit = 25`. Assertion `last_limit == 0` fails (25 ≠ 0).
///
/// GREEN (post-Task-12, gate implemented):
///   `resolved_spec_map = None` → `datetime_index_cols = []` (conservative default).
///   `inject_now` folds `now() - interval '7d'` to `Literal::Timestamp(now - 7d)`.
///   `is_pushed_temporal_predicate(GtEq, folded-ts, "timestamp", [])` = false
///   (column "timestamp" absent from empty `datetime_index_cols`) →
///   `has_client_side_where = true` → gate returns `fetch_limit = 0` →
///   `last_limit = 0` → passes.
///
/// DataFusion may error on the unknown "timestamp" column after the adapter is called;
/// the assertion operates on `last_limit` which is set inside `fetch()` before DataFusion.
///
/// F-LENSB-P13-002: assert `fetch_count >= 1` before `last_limit == 0` — the init
/// value of `last_limit` is `0`, which would vacuously satisfy the assertion if the
/// adapter is never called.
///
/// SAP-3 compliance: query reaches the gate end-to-end through `QueryEngine::execute`
/// from a real SQL string, not via a synthetic AST injected into an internal handler.
#[tokio::test]
async fn test_psg_relative_temporal_now_interval_suppresses_early_stop() {
    let last_limit = Arc::new(AtomicU64::new(0));
    let fetch_count = Arc::new(AtomicU64::new(0));

    let page1 = make_status_batch("page1", 100);
    let page2 = make_status_batch("page2", 100);
    let page3 = make_status_batch("page3", 100);

    let adapter = Arc::new(PlanShapeGateMockAdapter {
        sensor_type_id: SensorId::from("mock"),
        last_limit: Arc::clone(&last_limit),
        fetch_count: Arc::clone(&fetch_count),
        full_batches: vec![page1.clone(), page2, page3],
        page1_batches: vec![page1],
    });

    let org_id = OrgId::new();
    let mut adapter_registry = AdapterRegistry::new();
    adapter_registry.register(org_id, adapter);

    // OrgSlug::new_unchecked: test-only client setup, same pattern as opts() helper
    // above (line ~303). Not a production code path.
    let client_slug = OrgSlug::new_unchecked("test-org");
    let client_registry = Arc::new(ClientRegistry::new(vec![client_slug.clone()]));
    let credential_store: Arc<dyn CredentialStore> = Arc::new(PsgNullCredentialStore);

    // QueryEngine::new: org_registry = None (test mode); resolve_org_id falls back
    // to Path 2 (first adapter registered for the "mock" sensor_id).
    // resolved_spec_map = None → datetime_index_cols = [] → conservative suppress.
    let engine = QueryEngine::new(
        Arc::new(adapter_registry),
        credential_store,
        Arc::new(OcsfNormalizer::new()),
        client_registry,
        QueryEngineConfig::default(),
    )
    .with_credential_resolver(Arc::new(StubCredentialResolver));

    let options = QueryOptions {
        clients: Some(vec![client_slug]),
        sensors: None,
        limit: Some(25),
        force_refresh: false,
        ..QueryOptions::default()
    };

    // `inject_now` folds `now() - interval '7d'` → `Literal::Timestamp(now - 7d)`.
    // DataFusion may reject the "timestamp" column as absent from mock schema;
    // `last_limit` is recorded inside `fetch()` before DataFusion evaluation.
    let _result = engine
        .execute(
            "SELECT * FROM mock_events WHERE timestamp >= now() - interval '7d' LIMIT 25",
            options,
        )
        .await;

    // F-LENSB-P13-002: adapter must be invoked at least once.
    let fc = fetch_count.load(Ordering::SeqCst);
    assert!(
        fc >= 1,
        "PSG-029 (relative-temporal suppress — QueryEngine::execute): adapter must be \
         called at least once (fetch_count={fc}). fetch_count=0 means last_limit==0 is \
         the init value, which vacuously satisfies the next assertion."
    );

    let seen_limit = last_limit.load(Ordering::SeqCst);
    assert_eq!(
        seen_limit, 0,
        "PSG-029 (ADR-060 §D8.7 — relative-temporal suppress via QueryEngine::execute): \
         gate must set fetch_limit=0 for `timestamp >= now() - interval '7d'`; adapter \
         saw params.limit={seen_limit}. If non-zero ({seen_limit}=25), the Task-12 gate \
         `ast_is_reducing_plan_with_index_cols` is not yet implemented \
         (story status: draft → pipeline uses fetch_limit=opts.limit=25 unconditionally)."
    );
}

// ===========================================================================
// PSG-030 / PSG-031 / Regression Guard — ADR-060 §D8.7 v1.7 soundness fixes
// ===========================================================================
//
// These three tests exercise the v1.7 soundness fixes:
//   PSG-030 (HIGH-001): redundant same-direction lower temporal bounds (`> X AND > Y`)
//     wrongly PERMIT early-stop under the v1.5/v1.6 AND-arm `.all()` logic.
//   PSG-031 (MED-001): OCSF-flattened Arrow name (`time`) is absent from
//     `datetime_index_cols` (which only contains `col.name="timestamp"`), causing the
//     gate to wrongly SUPPRESS for `WHERE time > …` on an `ocsf_column_naming=true` sensor.
//   Regression guard: canonical one-lower+one-upper time window (`>= X AND < Y`) must
//     still PERMIT early-stop after the PSG-030 fix (this test should pass before AND after).
//
// Both PSG-030 and PSG-031 require a `resolved_spec_map` wired into the
// `MaterializationContext` so `datetime_index_cols` is non-empty. With
// `resolved_spec_map = None` the conservative-suppress default would mask the defect
// under test: PSG-030 would vacuously SUPPRESS (correct behavior, wrong reason) and
// PSG-031 would also SUPPRESS (conservative), preventing the RED gate from firing.
//
// Helpers:
//   `make_psg_spec_map(ocsf_naming)` — builds the minimal resolved_spec_map.
//   `plan_gate_mat_ctx_with_spec(spec_map)` — like `plan_gate_mat_ctx` but wires the map.
// ---------------------------------------------------------------------------

/// Build a minimal `resolved_spec_map` for PSG-030 / PSG-031 / regression guard.
///
/// Produces a sensor spec with a single table `mock_events` that has one column:
/// - `name = "timestamp"`, `column_type = Datetime`, `ColumnOptions::Index`
/// - `ocsf_field = Some("time")` and `ocsf_column_naming = true` iff `ocsf_naming` is true
///
/// Registered under key `("test-org", "mock")` (matching the `opts()` client slug and
/// `PlanShapeGateMockAdapter::sensor_type()`).
///
/// With `ocsf_naming=false`: `datetime_index_cols = ["timestamp"]` → queries on "timestamp"
///   can be PERMITTED; queries on "time" are NOT in the set → SUPPRESSED.
/// With `ocsf_naming=true` (after v1.7 fix): `datetime_index_cols = ["timestamp", "time"]`
///   → queries on either name PERMITTED; before fix only ["timestamp"].
fn make_psg_spec_map(
    ocsf_naming: bool,
) -> Arc<
    std::collections::HashMap<
        prism_spec_engine::ResolvedSpecKey,
        prism_spec_engine::ResolvedSensorSpec,
    >,
> {
    use prism_core::{ColumnOptions, ColumnType};
    use prism_spec_engine::{
        ColumnSpec, OverlayLoader, SensorInstanceOverlay, SensorSpec as EngineSpec, TableSpec,
    };

    // ColumnSpec::new() is the public constructor — forward-compatible with #[non_exhaustive].
    // Signature: new(name, column_type, ocsf_field: Option<String>, options: Vec<ColumnOptions>).
    // Datetime INDEX column; ocsf_field only set when sensor uses OCSF naming.
    let ts_col = ColumnSpec::new(
        "timestamp",
        ColumnType::Datetime,
        if ocsf_naming {
            Some("time".to_string())
        } else {
            None
        },
        vec![ColumnOptions::Index],
    );

    // Empty steps — no HTTP fetching in the gate test; the mock adapter handles data.
    let table =
        TableSpec::new_point_in_time("mock_events", "security_finding", vec![ts_col], vec![]);

    // SensorSpec is #[non_exhaustive]; struct-literal construction is forbidden outside the
    // defining crate (E0639). SensorSpec::new() predates ocsf_column_naming and does not
    // expose it. Use TOML round-trip deserialization — the canonical external construction path.
    // All #[serde(default)] fields not listed default correctly; ocsf_column_naming must be
    // explicit because it governs which Arrow names appear in datetime_index_cols.
    let spec_toml = format!(
        "sensor_id = \"mock\"\n\
         name = \"Mock\"\n\
         auth_type = \"api_key\"\n\
         base_url = \"https://example.com\"\n\
         version = \"1.0.0\"\n\
         ocsf_column_naming = {}\n",
        ocsf_naming
    );
    let mut spec: EngineSpec =
        toml::from_str(&spec_toml).expect("PSG spec map fixture: SensorSpec TOML parse failed");
    // spec.tables is a pub field — direct assignment is allowed on #[non_exhaustive] types
    // (the restriction applies only to struct-expression syntax, not field access/mutation).
    spec.tables = vec![table];

    // OrgSlug::new_unchecked: test-only client identity — same pattern as `opts()` helper.
    let org_slug = OrgSlug::new_unchecked("test-org");
    let sensor_id = SensorId::from("mock");

    // Use the canonical external construction path (ResolvedSensorSpec is #[non_exhaustive]).
    let overlay_toml = "extends = \"mock\"\ninstance_id = \"mock@test-org\"";
    let overlay: SensorInstanceOverlay = toml::from_str(overlay_toml)
        .expect("PSG spec map fixture: SensorInstanceOverlay TOML parse failed");
    let resolved = OverlayLoader::merge_overlay_onto_type_spec(&spec, &overlay, org_slug.clone());

    let mut map = std::collections::HashMap::new();
    map.insert((org_slug, sensor_id), resolved);
    Arc::new(map)
}

/// Like `plan_gate_mat_ctx` but injects a `resolved_spec_map` so the plan-shape gate
/// derives `datetime_index_cols` from the sensor spec rather than using the conservative
/// empty default.
///
/// Used by PSG-030, PSG-031, and the canonical time-window regression guard.
fn plan_gate_mat_ctx_with_spec(
    spec_map: Arc<
        std::collections::HashMap<
            prism_spec_engine::ResolvedSpecKey,
            prism_spec_engine::ResolvedSensorSpec,
        >,
    >,
) -> (MaterializationContext, Arc<AtomicU64>, Arc<AtomicU64>) {
    let last_limit = Arc::new(AtomicU64::new(0));
    let fetch_count = Arc::new(AtomicU64::new(0));

    let page1 = make_status_batch("page1", 100);
    let page2 = make_status_batch("page2", 100);
    let page3 = make_status_batch("page3", 100);

    let adapter = Arc::new(PlanShapeGateMockAdapter {
        sensor_type_id: SensorId::from("mock"),
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
        None,           // org_registry absent — test mode (resolve_org_id falls back to Path 2)
        Some(spec_map), // resolved_spec_map → datetime_index_cols non-empty
    );

    (mat_ctx, last_limit, fetch_count)
}

// ---------------------------------------------------------------------------
// Cross-sensor source-scoping helpers — PSG-032 / PSG-033 / PSG-030b
// ---------------------------------------------------------------------------
//
// ADR-060 v1.8 (F-R16-P16-LENSA-HIGH-001): `datetime_index_cols` must be
// scoped to the source sensor being queried, not aggregated globally across all
// sensors in `resolved_spec_map`.  Without source-scoping, an INDEX designation
// in sensor A bleeds into sensor B's gate decision.
//
// The tests use three helpers:
//   `make_psg_armis_spec_map()`          — armis only; last_seen Datetime+INDEX.
//   `make_psg_cross_sensor_spec_map()`   — armis (INDEX) + crowdstrike (no INDEX).
//   `plan_gate_mat_ctx_for_sensor_type_and_spec(sensor_type, spec_map)` — mat_ctx
//       with a mock adapter whose `sensor_type_id` is set to `sensor_type` so the
//       source table prefix in the query (e.g. "crowdstrike_devices") routes to it.
// ---------------------------------------------------------------------------

/// Low-level helper: build one resolved spec entry for use in spec maps.
///
/// Parameters:
/// - `org_slug_str`: org slug for the key; uses `OrgSlug::new_unchecked` (test-only).
/// - `sensor_id_str`: sensor_id for both the key and the TOML spec.
/// - `table_name`: DataFusion-visible table name (source prefix + "_" + suffix), e.g.
///   `"armis_devices"` or `"crowdstrike_devices"`.
/// - `col_name`: Datetime column to add to the table spec.
/// - `has_index`: whether `col_name` carries `ColumnOptions::Index`.
///
/// OrgSlug::new_unchecked: test-only identity — never used in production code paths.
fn make_psg_sensor_spec_entry(
    org_slug_str: &str,
    sensor_id_str: &str,
    table_name: &str,
    col_name: &str,
    has_index: bool,
) -> (
    prism_spec_engine::ResolvedSpecKey,
    prism_spec_engine::ResolvedSensorSpec,
) {
    use prism_core::{ColumnOptions, ColumnType};
    use prism_spec_engine::{
        ColumnSpec, OverlayLoader, SensorInstanceOverlay, SensorSpec as EngineSpec, TableSpec,
    };

    let options = if has_index {
        vec![ColumnOptions::Index]
    } else {
        vec![]
    };
    let col = ColumnSpec::new(col_name, ColumnType::Datetime, None, options);
    let table = TableSpec::new_point_in_time(table_name, "security_finding", vec![col], vec![]);

    // SensorSpec is #[non_exhaustive] — construct via TOML round-trip (E0639 guard).
    let spec_toml = format!(
        "sensor_id = \"{sensor_id_str}\"\n\
         name = \"{sensor_id_str} Mock\"\n\
         auth_type = \"api_key\"\n\
         base_url = \"https://example.com\"\n\
         version = \"1.0.0\"\n\
         ocsf_column_naming = false\n"
    );
    let mut spec: EngineSpec = toml::from_str(&spec_toml)
        .unwrap_or_else(|e| panic!("make_psg_sensor_spec_entry: TOML parse failed: {e}"));
    spec.tables = vec![table];

    // OrgSlug::new_unchecked: test-only client identity — mirrors `opts()` helper pattern.
    let org_slug = OrgSlug::new_unchecked(org_slug_str);
    let sensor_id = SensorId::from(sensor_id_str);

    let overlay_toml =
        format!("extends = \"{sensor_id_str}\"\ninstance_id = \"{sensor_id_str}@{org_slug_str}\"");
    let overlay: SensorInstanceOverlay = toml::from_str(&overlay_toml)
        .unwrap_or_else(|e| panic!("make_psg_sensor_spec_entry: overlay TOML parse failed: {e}"));
    let resolved = OverlayLoader::merge_overlay_onto_type_spec(&spec, &overlay, org_slug.clone());

    ((org_slug, sensor_id), resolved)
}

/// Build a spec map with only the armis sensor.
///
/// armis: `sensor_id="armis"`, table `"armis_devices"`, column `"last_seen"`
/// Datetime + `ColumnOptions::Index`. Mirrors the `last_seen` INDEX designation in
/// the real `armis.sensor.toml`.
///
/// `datetime_index_cols` derived from this map: `["last_seen"]`.
fn make_psg_armis_spec_map() -> Arc<
    std::collections::HashMap<
        prism_spec_engine::ResolvedSpecKey,
        prism_spec_engine::ResolvedSensorSpec,
    >,
> {
    let (key, val) =
        make_psg_sensor_spec_entry("test-org", "armis", "armis_devices", "last_seen", true);
    let mut map = std::collections::HashMap::new();
    map.insert(key, val);
    Arc::new(map)
}

/// Build a spec map with BOTH armis and crowdstrike sensors.
///
/// armis:       `last_seen` Datetime + `ColumnOptions::Index` (INDEX).
/// crowdstrike: `last_seen` Datetime, NO Index.
///
/// This mirrors the real sensor TOML files:
///   - `armis.sensor.toml`: `name = "last_seen"`, `column_type = "datetime"`,
///     `options = ["INDEX"]`
///   - `crowdstrike.sensor.toml`: `name = "last_seen"`, `column_type = "datetime"`,
///     NO `options = ["INDEX"]`
///
/// Used by PSG-032 (cross-sensor source-scope SUPPRESS): the defect under test is
/// that the current global `datetime_index_cols` collection iterates ALL spec map
/// values, so armis's INDEX classification bleeds into crowdstrike's gate decision.
fn make_psg_cross_sensor_spec_map() -> Arc<
    std::collections::HashMap<
        prism_spec_engine::ResolvedSpecKey,
        prism_spec_engine::ResolvedSensorSpec,
    >,
> {
    let (armis_key, armis_val) = make_psg_sensor_spec_entry(
        "test-org",
        "armis",
        "armis_devices",
        "last_seen",
        true, // INDEX
    );
    let (cs_key, cs_val) = make_psg_sensor_spec_entry(
        "test-org",
        "crowdstrike",
        "crowdstrike_devices",
        "last_seen",
        false, // NO INDEX
    );
    let mut map = std::collections::HashMap::new();
    map.insert(armis_key, armis_val);
    map.insert(cs_key, cs_val);
    Arc::new(map)
}

/// Build a spec map for a single mock sensor whose table has TWO Datetime+INDEX columns.
///
/// sensor: `sensor_id="mock"`, table `"mock_events"`.
/// Columns: `"ts_start"` Datetime+INDEX and `"ts_end"` Datetime+INDEX.
///
/// Used by PSG-030d (Condition K — multi-index-datetime SUPPRESS): when the gate is
/// queried against a table with multiple INDEX datetime columns, it must SUPPRESS
/// because the push-down handling is ambiguous for a temporal predicate that targets
/// only one of the INDEX columns (Condition K, ADR-060 v1.8 §D8.7).
fn make_psg_multi_index_spec_map() -> Arc<
    std::collections::HashMap<
        prism_spec_engine::ResolvedSpecKey,
        prism_spec_engine::ResolvedSensorSpec,
    >,
> {
    use prism_core::{ColumnOptions, ColumnType};
    use prism_spec_engine::{
        ColumnSpec, OverlayLoader, SensorInstanceOverlay, SensorSpec as EngineSpec, TableSpec,
    };

    let ts_start_col = ColumnSpec::new(
        "ts_start",
        ColumnType::Datetime,
        None,
        vec![ColumnOptions::Index],
    );
    let ts_end_col = ColumnSpec::new(
        "ts_end",
        ColumnType::Datetime,
        None,
        vec![ColumnOptions::Index],
    );

    let table = TableSpec::new_point_in_time(
        "mock_events",
        "security_finding",
        vec![ts_start_col, ts_end_col],
        vec![],
    );

    let spec_toml = "sensor_id = \"mock\"\n\
                     name = \"Mock Multi-Index\"\n\
                     auth_type = \"api_key\"\n\
                     base_url = \"https://example.com\"\n\
                     version = \"1.0.0\"\n\
                     ocsf_column_naming = false\n";
    let mut spec: EngineSpec = toml::from_str(spec_toml)
        .expect("make_psg_multi_index_spec_map: SensorSpec TOML parse failed");
    spec.tables = vec![table];

    // OrgSlug::new_unchecked: test-only client identity.
    let org_slug = OrgSlug::new_unchecked("test-org");
    let sensor_id = SensorId::from("mock");

    let overlay_toml = "extends = \"mock\"\ninstance_id = \"mock@test-org\"";
    let overlay: SensorInstanceOverlay = toml::from_str(overlay_toml)
        .expect("make_psg_multi_index_spec_map: overlay TOML parse failed");
    let resolved = OverlayLoader::merge_overlay_onto_type_spec(&spec, &overlay, org_slug.clone());

    let mut map = std::collections::HashMap::new();
    map.insert((org_slug, sensor_id), resolved);
    Arc::new(map)
}

/// Like `plan_gate_mat_ctx_with_spec` but with a configurable `sensor_type_id`.
///
/// Used when the query source table prefix must match the adapter's sensor_type
/// for fan-out routing, e.g.:
/// - `sensor_type = "crowdstrike"` for `SELECT * FROM crowdstrike_devices …`
/// - `sensor_type = "armis"` for `SELECT * FROM armis_devices …`
fn plan_gate_mat_ctx_for_sensor_type_and_spec(
    sensor_type: &str,
    spec_map: Arc<
        std::collections::HashMap<
            prism_spec_engine::ResolvedSpecKey,
            prism_spec_engine::ResolvedSensorSpec,
        >,
    >,
) -> (MaterializationContext, Arc<AtomicU64>, Arc<AtomicU64>) {
    let last_limit = Arc::new(AtomicU64::new(0));
    let fetch_count = Arc::new(AtomicU64::new(0));

    let page1 = make_status_batch("page1", 100);
    let page2 = make_status_batch("page2", 100);
    let page3 = make_status_batch("page3", 100);

    let adapter = Arc::new(PlanShapeGateMockAdapter {
        sensor_type_id: SensorId::from(sensor_type),
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
        None,           // org_registry absent — test mode (Path 2 fallback)
        Some(spec_map), // resolved_spec_map → datetime_index_cols non-empty
    );

    (mat_ctx, last_limit, fetch_count)
}

// ===========================================================================
// PSG-030 — ADR-060 §D8.7 v1.7 HIGH-001: redundant lower bound SUPPRESSES
// ===========================================================================

/// RG-PSG-030 — ADR-060 §D8.7 v1.7 HIGH-001: two Gt bounds on the same INDEX col
///
/// `SELECT * FROM mock_events WHERE timestamp > '2026-01-01T00:00:00Z'
///   AND timestamp > '2026-06-01T00:00:00Z' LIMIT 1`
///
/// ## Soundness Defect (ADR-060 v1.5/v1.6)
///
/// `extract_time_bounds_from_predicate` is first-wins: it picks up `timestamp > '2026-01-01'`
/// as the server-side lower bound and silently drops `timestamp > '2026-06-01'` to DataFusion
/// client-side filtering. With LIMIT 1 and early-stop, the pipeline may stop after page 1
/// (100 rows), apply the dropped bound via DataFusion, and return fewer than the correct rows
/// — or return a row from the first page that would not survive the second bound if it were
/// evaluated on a larger result set.
///
/// The v1.5/v1.6 AND-arm uses `.all(|p| is_pushed_temporal_predicate(p, ...))`. Both
/// `Compare(Gt, "timestamp", A)` and `Compare(Gt, "timestamp", B)` satisfy
/// `is_pushed_temporal_predicate` (range op + INDEX col + temporal RHS), so `.all()` = true
/// → `has_client_side_where = false` → gate PERMITS (wrongly).
///
/// ## v1.7 Fix
///
/// Detect AND predicates that contain two or more same-direction lower bounds on the same
/// INDEX column. When found, classify the AND predicate as NOT fully server-pushed →
/// `has_client_side_where = true` → gate SUPPRESSES → `fetch_limit = 0`.
///
/// ## RED / GREEN mechanics
///
/// RED (current code — spec_map provides `datetime_index_cols = ["timestamp"]`):
///   Both Gt leaves pass `is_pushed_temporal_predicate` → AND `.all()` = true →
///   `has_client_side_where = false` → gate PERMITS → `fetch_limit = 1` → `last_limit = 1 ≠ 0`.
/// GREEN (v1.7 fix applied):
///   Redundant same-direction lower bounds detected → classified as client-side →
///   `fetch_limit = 0` → `last_limit = 0` → assertion passes.
///
/// F-LENSB-P13-002: assert `fetch_count >= 1` before `last_limit == 0` — the init value of
/// `last_limit` is `0`, which would vacuously satisfy the assertion if the adapter is never
/// called.
///
/// SAP-3: `WHERE timestamp > A AND timestamp > B LIMIT 1` is grammar-reachable standard SQL.
/// Anchors: S-ENGINE-LIMIT-EARLY-STOP-001 RG-PSG-030, BC-2.16.002 EC-01-034, ADR-060 §D8.7.
#[tokio::test]
async fn test_psg_rg030_redundant_lower_bound_suppresses_early_stop() {
    // Spec map: datetime_index_cols = ["timestamp"]. This is required so the gate
    // uses the permission path (with the INDEX col known) rather than the conservative
    // suppress default (datetime_index_cols = []). Without the spec map, the gate would
    // SUPPRESS regardless, masking the defect.
    let spec_map = make_psg_spec_map(false);
    let (mut mat_ctx, last_limit, fetch_count) = plan_gate_mat_ctx_with_spec(spec_map);
    let session_ctx =
        build_session_context(QUERY_MEMORY_POOL_BYTES).expect("build_session_context");

    // Two same-direction (Gt) lower bounds on the same INDEX datetime column.
    // extract_time_bounds_from_predicate consumes only `timestamp > '2026-01-01'`;
    // the second `timestamp > '2026-06-01'` is silently dropped to DataFusion.
    // DataFusion may report no 'timestamp' column in the mock schema — assertion is on
    // last_limit, which is recorded inside fetch() before DataFusion executes.
    let query = "SELECT * FROM mock_events \
                 WHERE timestamp > '2026-01-01T00:00:00Z' \
                 AND timestamp > '2026-06-01T00:00:00Z' \
                 LIMIT 1";
    let _result = run_materialization_pipeline(query, &opts(1), &mut mat_ctx, &session_ctx).await;

    // F-LENSB-P13-002: adapter must have been invoked (not vacuous init value).
    let fc = fetch_count.load(Ordering::SeqCst);
    assert!(
        fc >= 1,
        "PSG-030 (redundant lower bound): adapter must have been called at least once \
         (fetch_count={fc}). A count of 0 means last_limit==0 is the AtomicU64 init value, \
         not evidence of suppression."
    );

    // PRIMARY: gate must suppress early-stop (fetch_limit = 0).
    //
    // RED (current v1.5/v1.6 — AND-arm .all() wrongly PERMITs):
    //   Both `Compare(Gt, "timestamp", ts_A)` and `Compare(Gt, "timestamp", ts_B)` pass
    //   is_pushed_temporal_predicate → .all() = true → has_client_side_where = false →
    //   fetch_limit = opts.limit = 1 → last_limit = 1 ≠ 0 → assertion FAILS.
    //
    // GREEN (v1.7 fix: redundant same-direction bounds → suppress):
    //   Detected two Gt leaves on "timestamp" → classified as NOT fully server-pushed →
    //   has_client_side_where = true → fetch_limit = 0 → last_limit = 0 → passes.
    let seen_limit = last_limit.load(Ordering::SeqCst);
    assert_eq!(
        seen_limit, 0,
        "PSG-030 (ADR-060 §D8.7 v1.7 — HIGH-001 redundant lower bound): gate must \
         SUPPRESS early-stop when two same-direction Gt bounds target the same INDEX \
         column (fetch_limit=0); adapter saw params.limit={seen_limit}. \
         If 1: v1.5/v1.6 AND-arm .all() wrongly classified `timestamp > A AND \
         timestamp > B` as fully server-pushed. extract_time_bounds_from_predicate is \
         first-wins — the second Gt bound is dropped to DataFusion client-side, making \
         LIMIT-1 early-stop incorrect. \
         Anchors: S-ENGINE-LIMIT-EARLY-STOP-001 RG-PSG-030, BC-2.16.002 EC-01-034, \
         ADR-060 §D8.7."
    );
}

// ===========================================================================
// PSG-031 — ADR-060 §D8.9 MED-001: OCSF Arrow name PERMITS early-stop
// ===========================================================================

/// RG-PSG-031 — ADR-060 §D8.9 MED-001: OCSF-flattened Arrow name `time` PERMITS early-stop
///
/// Fixture sensor spec: `ocsf_column_naming = true`, `col.name = "timestamp"`,
/// `ocsf_field = "time"`, `column_type = Datetime`, `ColumnOptions::Index`.
/// Query: `SELECT * FROM mock_events WHERE time > '2026-01-01T00:00:00Z' LIMIT 5`
///
/// ## Soundness Defect (ADR-060 v1.5/v1.6)
///
/// The plan-shape gate computes `datetime_index_cols` by iterating `resolved_spec_map` and
/// collecting `col.name` for every `Datetime + Index` column. For the fixture spec:
///   `datetime_index_cols = ["timestamp"]`
/// The query uses the OCSF-flattened Arrow name `"time"` (the name DataFusion exposes and
/// LLM agents author after schema introspection on an `ocsf_column_naming=true` sensor).
/// `is_pushed_temporal_predicate("time", ["timestamp"])` returns `false` because `"time"` is
/// absent from `datetime_index_cols`. Consequently:
///   `has_client_side_where = true` → gate SUPPRESSES → `fetch_limit = 0`
/// This is WRONG: the `time` column IS an INDEX-designated datetime column (via its
/// `ocsf_field` mapping). Suppressing here causes unnecessary full-dataset fetches for every
/// temporal query authored by an LLM agent against a Claroty/OCSF-named sensor.
///
/// ## v1.7 Fix (ADR-060 §D8.9)
///
/// When iterating `resolved_spec_map` to build `datetime_index_cols`, also insert
/// `ocsf_field_to_arrow_name(col.ocsf_field)` when `spec.ocsf_column_naming = true` and
/// `col.ocsf_field` is non-empty. After the fix:
///   `datetime_index_cols = ["timestamp", "time"]`
/// `is_pushed_temporal_predicate("time", ["timestamp", "time"])` = true →
/// `has_client_side_where = false` → gate PERMITS → `fetch_limit = 5`.
///
/// ## RED / GREEN mechanics
///
/// RED (current v1.5/v1.6 — only col.name in datetime_index_cols):
///   `datetime_index_cols = ["timestamp"]`; "time" absent → `is_pushed_temporal_predicate`
///   returns false → gate SUPPRESSES → `fetch_limit = 0` → `last_limit = 0`.
///   Assertion `last_limit > 0` FAILS.
/// GREEN (v1.7 fix — OCSF Arrow name registered):
///   `datetime_index_cols = ["timestamp", "time"]`; "time" present → PERMITS →
///   `fetch_limit = 5` → `last_limit = 5 > 0` → passes.
///
/// DataFusion may report no "time" column in the mock schema (the mock returns only a
/// "status" column). The assertion operates on `last_limit`, recorded inside `fetch()`
/// before DataFusion executes, so a DataFusion error does not affect the test.
///
/// F-LENSB-P13-002: assert `fetch_count >= 1` before the `last_limit > 0` assertion.
///
/// SAP-3: `WHERE time > '...' LIMIT 5` is grammar-reachable standard SQL.
/// Anchors: S-ENGINE-LIMIT-EARLY-STOP-001 RG-PSG-031, BC-2.16.002 EC-01-035, ADR-060 §D8.9.
#[tokio::test]
async fn test_psg_rg031_ocsf_arrow_name_permits_early_stop() {
    // Spec map: ocsf_column_naming=true, col.name="timestamp", ocsf_field="time", INDEX+Datetime.
    // Before v1.7 fix: datetime_index_cols = ["timestamp"] only.
    // After v1.7 fix: datetime_index_cols = ["timestamp", "time"].
    let spec_map = make_psg_spec_map(true);
    let (mut mat_ctx, last_limit, fetch_count) = plan_gate_mat_ctx_with_spec(spec_map);
    let session_ctx =
        build_session_context(QUERY_MEMORY_POOL_BYTES).expect("build_session_context");

    // Query uses the OCSF-flattened Arrow name "time" — the column name an LLM agent
    // would use after introspecting the schema on an ocsf_column_naming=true sensor.
    // DataFusion may reject "time" as an unknown column in the mock schema; assertion is
    // on last_limit which is recorded before DataFusion evaluation.
    let query = "SELECT * FROM mock_events WHERE time > '2026-01-01T00:00:00Z' LIMIT 5";
    let _result = run_materialization_pipeline(query, &opts(5), &mut mat_ctx, &session_ctx).await;

    // F-LENSB-P13-002: adapter must have been invoked (not vacuous init value).
    let fc = fetch_count.load(Ordering::SeqCst);
    assert!(
        fc >= 1,
        "PSG-031 (OCSF Arrow name permits): adapter must have been called at least once \
         (fetch_count={fc}). A count of 0 means last_limit==0 is the AtomicU64 init value, \
         not evidence of gate behavior."
    );

    // PRIMARY: gate must PERMIT early-stop (fetch_limit = 5 > 0).
    //
    // RED (current v1.5/v1.6 — col.name only in datetime_index_cols):
    //   datetime_index_cols = ["timestamp"]. "time" is absent.
    //   is_pushed_temporal_predicate("time", ["timestamp"]) = false →
    //   has_client_side_where = true → gate SUPPRESSES → fetch_limit = 0 →
    //   last_limit = 0. Assertion last_limit > 0 FAILS.
    //
    // GREEN (v1.7 fix: OCSF Arrow name registered in datetime_index_cols):
    //   datetime_index_cols = ["timestamp", "time"]. "time" is present.
    //   is_pushed_temporal_predicate("time", ["timestamp", "time"]) = true →
    //   has_client_side_where = false → gate PERMITS → fetch_limit = 5 →
    //   last_limit = 5 > 0. Assertion passes.
    let seen_limit = last_limit.load(Ordering::SeqCst);
    assert!(
        seen_limit > 0,
        "PSG-031 (ADR-060 §D8.9 v1.7 — MED-001 OCSF Arrow name permits early-stop): \
         gate must PERMIT early-stop for `WHERE time > …` on an ocsf_column_naming=true \
         sensor with col.name=\"timestamp\" and ocsf_field=\"time\" (fetch_limit > 0); \
         adapter saw params.limit={seen_limit}. \
         If 0: v1.5/v1.6 gate only registered col.name=\"timestamp\" in \
         datetime_index_cols; the OCSF-flattened Arrow name \"time\" is absent, so \
         the predicate is classified as client-side and early-stop is wrongly suppressed. \
         Fix: for each datetime+INDEX column with non-empty ocsf_field on a sensor where \
         ocsf_column_naming=true, insert ocsf_field_to_arrow_name(ocsf_field) into \
         datetime_index_cols. \
         Anchors: S-ENGINE-LIMIT-EARLY-STOP-001 RG-PSG-031, BC-2.16.002 EC-01-035, \
         ADR-060 §D8.9."
    );
}

// ===========================================================================
// PSG canonical time-window — regression guard (must pass before AND after fix)
// ===========================================================================

/// Regression guard — canonical one-lower+one-upper time window still PERMITS early-stop.
///
/// `SELECT * FROM mock_events
///   WHERE timestamp >= '2026-01-01T00:00:00Z' AND timestamp < '2026-07-01T00:00:00Z'
///   LIMIT 5`
///
/// This is the standard time-window filter: one `Ge` (≥) lower bound and one `Lt` (<)
/// upper bound on the same single INDEX datetime column. Both bounds are of DIFFERENT
/// operators (no first-wins ambiguity). `extract_time_bounds_from_predicate` correctly
/// captures both: `start_time = '2026-01-01'`, `end_time = '2026-07-01'`. The full range
/// is pushed server-side; no DataFusion-side residual. Early-stop is SAFE → gate PERMITS.
///
/// This test PASSES before the v1.7 fix AND after. Its purpose is to confirm that the
/// PSG-030 soundness fix (which suppresses redundant SAME-direction lower bounds) does NOT
/// accidentally regress the canonical DIFFERENT-direction lower+upper time-window pattern.
///
/// SAP-3: `WHERE ts >= X AND ts < Y LIMIT N` is grammar-reachable standard SQL.
#[tokio::test]
async fn test_psg_canonical_time_window_still_permitted() {
    // Spec map: datetime_index_cols = ["timestamp"]. Standard non-OCSF sensor.
    let spec_map = make_psg_spec_map(false);
    let (mut mat_ctx, last_limit, fetch_count) = plan_gate_mat_ctx_with_spec(spec_map);
    let session_ctx =
        build_session_context(QUERY_MEMORY_POOL_BYTES).expect("build_session_context");

    // Canonical time-window: one Ge lower bound + one Lt upper bound — different operators.
    // Both fully pushed server-side. No DataFusion residual. Early-stop is safe.
    // DataFusion may report no 'timestamp' column in mock schema — assertion is on last_limit.
    let query = "SELECT * FROM mock_events \
                 WHERE timestamp >= '2026-01-01T00:00:00Z' \
                 AND timestamp < '2026-07-01T00:00:00Z' \
                 LIMIT 5";
    let _result = run_materialization_pipeline(query, &opts(5), &mut mat_ctx, &session_ctx).await;

    // F-LENSB-P13-002: adapter must have been called.
    let fc = fetch_count.load(Ordering::SeqCst);
    assert!(
        fc >= 1,
        "PSG canonical time-window (regression guard): adapter must have been called at \
         least once (fetch_count={fc}). A count of 0 means last_limit is the init value."
    );

    // Regression guard: gate must PERMIT early-stop (fetch_limit > 0).
    //
    // Before v1.7 fix: `Ge("timestamp", A)` and `Lt("timestamp", B)` both pass
    //   is_pushed_temporal_predicate → .all() = true → has_client_side_where = false →
    //   fetch_limit = 5 → last_limit = 5 > 0. PASSES.
    //
    // After v1.7 fix (PSG-030 redundant-lower-bound suppression): one Ge + one Lt are
    //   DIFFERENT operators → not "redundant same-direction" → still PERMITS →
    //   fetch_limit = 5 → last_limit = 5 > 0. PASSES.
    //
    // If this test FAILS after the PSG-030 fix, the fix over-suppressed the canonical
    // time-window pattern.
    let seen_limit = last_limit.load(Ordering::SeqCst);
    assert!(
        seen_limit > 0,
        "PSG canonical time-window (regression guard): gate must PERMIT early-stop for \
         `WHERE ts >= X AND ts < Y LIMIT N` on a single INDEX col \
         (fetch_limit > 0 expected); adapter saw params.limit={seen_limit}. \
         If 0: the PSG-030 v1.7 fix over-suppressed the canonical time-window pattern — \
         one Ge lower + one Lt upper bound are DIFFERENT operators and represent a fully \
         server-pushed range (no first-wins ambiguity). Only SAME-direction redundant \
         bounds (`> A AND > B`) should be suppressed."
    );
}

// ===========================================================================
// PSG-032 — ADR-060 v1.8 HIGH-001 (F-R16-P16-LENSA-HIGH-001): cross-sensor
// source-scope SUPPRESSES (TRUE RED GATE)
// ===========================================================================

/// RG-PSG-032 — Cross-sensor source-scope: crowdstrike query SUPPRESSES early-stop
///
/// Fixture: `resolved_spec_map` contains BOTH
///   - `("test-org", "armis")`:       `armis_devices.last_seen` Datetime + INDEX
///   - `("test-org", "crowdstrike")`: `crowdstrike_devices.last_seen` Datetime, NO INDEX
///
/// Query: `SELECT * FROM crowdstrike_devices WHERE last_seen > '2026-01-01T00:00:00Z' LIMIT 100`
///
/// ## Current defect (ADR-060 v1.7, F-R16-P16-LENSA-HIGH-001)
///
/// `datetime_index_cols` is built by iterating `resolved_spec_map.values()` globally —
/// it does not filter to the source sensor being queried. Because armis's `last_seen` is
/// Datetime+INDEX, "last_seen" appears in `datetime_index_cols`. The gate then checks:
///
///   `is_pushed_temporal_predicate("last_seen", ["last_seen"])` → `true`
///
/// → `has_client_side_where = false` → PERMIT → `fetch_limit = 100`.
///
/// But crowdstrike's `last_seen` has NO INDEX designation. The push-down guarantee
/// (ADR-033 T1) does not apply — early-stop is INCORRECT for this query.
///
/// ## v1.8 fix (ADR-060 §D8.7 source-scoping)
///
/// Filter `resolved_spec_map` to only the sensor(s) serving the source table being
/// queried before building `datetime_index_cols`. Armis's INDEX classification is
/// excluded when the query targets `crowdstrike_devices`.
///
/// After fix: `datetime_index_cols = []` (crowdstrike's `last_seen` has no INDEX) →
/// `is_pushed_temporal_predicate("last_seen", [])` → `false` →
/// `has_client_side_where = true` → SUPPRESS → `fetch_limit = 0`.
///
/// ## RED / GREEN mechanics
///
/// RED (current): global collection finds armis "last_seen" INDEX → PERMIT →
///   `last_limit = 100 ≠ 0` → assertion FAILS.
/// GREEN (v1.8 fix): source-scoped collection → crowdstrike "last_seen" has no INDEX →
///   SUPPRESS → `last_limit = 0` → assertion passes.
///
/// F-LENSB-P13-002: assert `fetch_count >= 1` before `last_limit == 0`.
///
/// SAP-3: end-to-end from `run_materialization_pipeline` with a real SQL query string.
/// Anchors: S-ENGINE-LIMIT-EARLY-STOP-001 RG-PSG-032, ADR-060 v1.8 §D8.7.
#[tokio::test]
async fn test_psg_rg032_cross_sensor_source_scope_suppresses_early_stop() {
    // Spec map: armis (last_seen INDEX) + crowdstrike (last_seen NO INDEX).
    // Current bug: datetime_index_cols collects from ALL sensors → includes "last_seen"
    // from armis → wrongly PERMITs the crowdstrike query.
    let spec_map = make_psg_cross_sensor_spec_map();
    // Mock adapter sensor_type="crowdstrike" so `crowdstrike_devices` routes to it.
    let (mut mat_ctx, last_limit, fetch_count) =
        plan_gate_mat_ctx_for_sensor_type_and_spec("crowdstrike", spec_map);
    let session_ctx =
        build_session_context(QUERY_MEMORY_POOL_BYTES).expect("build_session_context");

    // crowdstrike_devices.last_seen has NO INDEX — push-down not guaranteed.
    // Gate MUST suppress early-stop.
    // DataFusion may error (no "last_seen" in mock schema) — assertion is on last_limit,
    // recorded inside fetch() before DataFusion executes.
    let query = "SELECT * FROM crowdstrike_devices \
                 WHERE last_seen > '2026-01-01T00:00:00Z' \
                 LIMIT 100";
    let _result = run_materialization_pipeline(query, &opts(100), &mut mat_ctx, &session_ctx).await;

    // F-LENSB-P13-002: adapter must have been invoked.
    let fc = fetch_count.load(Ordering::SeqCst);
    assert!(
        fc >= 1,
        "PSG-032 (cross-sensor source-scope SUPPRESS): adapter must have been called at \
         least once (fetch_count={fc}). A count of 0 means last_limit=0 is the AtomicU64 \
         init value, not evidence of gate suppression."
    );

    // PRIMARY: gate must SUPPRESS early-stop (fetch_limit = 0).
    //
    // RED (current v1.7 — global datetime_index_cols):
    //   armis last_seen INDEX bleeds into crowdstrike's gate decision →
    //   datetime_index_cols = ["last_seen"] → is_pushed_temporal_predicate = true →
    //   has_client_side_where = false → PERMIT → fetch_limit = 100 → last_limit = 100 ≠ 0.
    //   Assertion FAILS.
    //
    // GREEN (v1.8 fix — source-scoped datetime_index_cols):
    //   datetime_index_cols scoped to crowdstrike → "last_seen" not INDEX → [] →
    //   is_pushed_temporal_predicate("last_seen", []) = false →
    //   has_client_side_where = true → SUPPRESS → fetch_limit = 0 → last_limit = 0.
    //   Assertion passes.
    let seen_limit = last_limit.load(Ordering::SeqCst);
    assert_eq!(
        seen_limit, 0,
        "PSG-032 (ADR-060 v1.8 §D8.7 — F-R16-P16-LENSA-HIGH-001 cross-sensor source-scope): \
         gate must SUPPRESS early-stop for crowdstrike_devices.last_seen (NO INDEX); \
         adapter saw params.limit={seen_limit}. \
         If 100: global datetime_index_cols includes armis last_seen INDEX, incorrectly \
         classifying crowdstrike's non-INDEX column as push-down-eligible. \
         Fix: scope datetime_index_cols construction to the source sensor being queried. \
         Anchors: S-ENGINE-LIMIT-EARLY-STOP-001 RG-PSG-032, ADR-060 v1.8 §D8.7."
    );
}

// ===========================================================================
// PSG-033 — ADR-060 v1.8 regression guard: armis source-scope PERMITS
// ===========================================================================

/// RG-PSG-033 — Armis source-scope: armis query PERMITS early-stop (regression guard)
///
/// Fixture: `resolved_spec_map` contains ONLY the armis sensor.
///   `("test-org", "armis")`: `armis_devices.last_seen` Datetime + INDEX.
///
/// Query: `SELECT * FROM armis_devices WHERE last_seen > '2026-01-01T00:00:00Z' LIMIT 100`
///
/// Guards against over-suppression: after the PSG-032 source-scoping fix, the gate
/// must still PERMIT early-stop for armis where `last_seen` IS INDEX-designated.
///
/// This test PASSES before AND after the v1.8 fix. It exists to discriminate a
/// regression where the source-scoping fix accidentally suppresses armis queries.
///
/// SAP-3: end-to-end from `run_materialization_pipeline` with a real SQL query string.
/// Anchors: S-ENGINE-LIMIT-EARLY-STOP-001 RG-PSG-033, ADR-060 v1.8 §D8.7.
#[tokio::test]
async fn test_psg_rg033_armis_source_scope_permits_early_stop() {
    // Spec map: armis only (last_seen Datetime+INDEX).
    // datetime_index_cols = ["last_seen"] → gate PERMITs for armis queries.
    let spec_map = make_psg_armis_spec_map();
    let (mut mat_ctx, last_limit, fetch_count) =
        plan_gate_mat_ctx_for_sensor_type_and_spec("armis", spec_map);
    let session_ctx =
        build_session_context(QUERY_MEMORY_POOL_BYTES).expect("build_session_context");

    // armis_devices.last_seen has INDEX — push-down is guaranteed (ADR-033 T1).
    // Gate MUST PERMIT early-stop.
    let query = "SELECT * FROM armis_devices \
                 WHERE last_seen > '2026-01-01T00:00:00Z' \
                 LIMIT 100";
    let _result = run_materialization_pipeline(query, &opts(100), &mut mat_ctx, &session_ctx).await;

    // F-LENSB-P13-002: adapter must have been invoked.
    let fc = fetch_count.load(Ordering::SeqCst);
    assert!(
        fc >= 1,
        "PSG-033 (armis source-scope PERMIT regression guard): adapter must have been \
         called at least once (fetch_count={fc}). A count of 0 means last_limit is the \
         init value, not gate evidence."
    );

    // Regression guard: gate must PERMIT early-stop (fetch_limit = 100 > 0).
    //
    // Before fix: datetime_index_cols = ["last_seen"] → is_pushed_temporal_predicate = true →
    //   PERMIT → last_limit = 100 > 0. PASSES.
    //
    // After v1.8 fix (source-scoped): datetime_index_cols scoped to armis only →
    //   still ["last_seen"] → is_pushed_temporal_predicate = true → PERMIT → last_limit > 0.
    //   PASSES.
    //
    // If this test FAILS after the PSG-032 fix, the fix over-suppressed armis queries.
    let seen_limit = last_limit.load(Ordering::SeqCst);
    assert!(
        seen_limit > 0,
        "PSG-033 (ADR-060 v1.8 §D8.7 — armis source-scope PERMIT regression guard): \
         gate must PERMIT early-stop for armis_devices.last_seen (INDEX) \
         (fetch_limit > 0 expected); adapter saw params.limit={seen_limit}. \
         If 0: the PSG-032 source-scoping fix over-suppressed armis queries — armis's \
         last_seen IS INDEX-designated and must still PERMIT. \
         Anchors: S-ENGINE-LIMIT-EARLY-STOP-001 RG-PSG-033, ADR-060 v1.8 §D8.7."
    );
}

// ===========================================================================
// PSG-030b — redundant UPPER bound SUPPRESSES (mutation-coverage guard)
// ===========================================================================

/// RG-PSG-030b — Redundant same-direction upper bounds SUPPRESS (mutation-coverage guard)
///
/// Query: `SELECT * FROM armis_devices
///   WHERE last_seen < '2026-06-01T00:00:00Z' AND last_seen < '2026-07-01T00:00:00Z'
///   LIMIT 100`
///
/// Fixture: armis only (`last_seen` Datetime+INDEX); `datetime_index_cols = ["last_seen"]`.
///
/// Both predicates are `Lt` (upper-direction) on the same INDEX column.
/// `count_temporal_bound_directions` returns `(lower=0, upper=2)`. The v1.7 check
/// `lower <= 1 && upper <= 1` evaluates to `false` → SUPPRESS.
///
/// This test PASSES against current code (v1.7 already implements `upper <= 1`).
/// Its purpose is mutation-coverage: killing the mutant that drops the `upper <= 1`
/// clause from the `is_pushed_temporal_predicate` AND arm.
///
/// PSG-030 covers `lower > 1` (redundant lower bound). PSG-030b covers `upper > 1`
/// (redundant upper bound). Together they ensure both halves of the `lower <= 1 && upper <= 1`
/// condition are exercised by a concrete test, preventing silent deletion of either
/// half by a mutation tool.
///
/// SAP-3: end-to-end from `run_materialization_pipeline` with a real SQL query string.
/// Anchors: S-ENGINE-LIMIT-EARLY-STOP-001 RG-PSG-030b, ADR-060 §D8.7 v1.7.
#[tokio::test]
async fn test_psg_rg030b_redundant_upper_bound_suppresses_early_stop() {
    let spec_map = make_psg_armis_spec_map();
    let (mut mat_ctx, last_limit, fetch_count) =
        plan_gate_mat_ctx_for_sensor_type_and_spec("armis", spec_map);
    let session_ctx =
        build_session_context(QUERY_MEMORY_POOL_BYTES).expect("build_session_context");

    // Two same-direction (Lt) upper bounds on the same INDEX datetime column.
    // count_temporal_bound_directions → (lower=0, upper=2) → upper > 1 → SUPPRESS.
    let query = "SELECT * FROM armis_devices \
                 WHERE last_seen < '2026-06-01T00:00:00Z' \
                 AND last_seen < '2026-07-01T00:00:00Z' \
                 LIMIT 100";
    let _result = run_materialization_pipeline(query, &opts(100), &mut mat_ctx, &session_ctx).await;

    // F-LENSB-P13-002: adapter must have been invoked.
    let fc = fetch_count.load(Ordering::SeqCst);
    assert!(
        fc >= 1,
        "PSG-030b (redundant upper bound — mutation-coverage guard): adapter must have \
         been called at least once (fetch_count={fc})."
    );

    // Guard: gate must SUPPRESS (fetch_limit = 0).
    // extract_time_bounds_from_predicate is first-wins: takes the first Lt as end_time,
    // drops the second Lt to DataFusion client-side. Two upper bounds → upper=2 > 1 →
    // `is_pushed_temporal_predicate` AND arm returns false → SUPPRESS.
    let seen_limit = last_limit.load(Ordering::SeqCst);
    assert_eq!(
        seen_limit, 0,
        "PSG-030b (ADR-060 §D8.7 v1.7 — redundant upper bound suppression): gate must \
         SUPPRESS when two same-direction Lt bounds target the same INDEX column \
         (fetch_limit=0); adapter saw params.limit={seen_limit}. \
         If non-zero: the `upper <= 1` clause in is_pushed_temporal_predicate is missing \
         or broken — mutation killing this clause must cause PSG-030b to fail. \
         Anchors: S-ENGINE-LIMIT-EARLY-STOP-001 RG-PSG-030b, ADR-060 §D8.7 v1.7."
    );
}

// ===========================================================================
// PSG-030d — Condition K: multi-index-datetime SUPPRESSES (TRUE RED GATE)
// ===========================================================================

/// RG-PSG-030d — Condition K: table with multiple Datetime+INDEX columns SUPPRESSES
///
/// Fixture: spec map with `sensor_id="mock"`, table `"mock_events"` containing
/// TWO Datetime+INDEX columns: `"ts_start"` and `"ts_end"`.
///
/// Query: `SELECT * FROM mock_events WHERE ts_start > '2026-01-01T00:00:00Z' LIMIT 100`
///
/// ## Defect under test (Condition K, ADR-060 v1.8)
///
/// When a table has multiple INDEX-designated Datetime columns, a temporal predicate
/// targeting only one of them is NOT fully server-pushed. `extract_time_bounds_from_predicate`
/// (pushdown.rs) can extract a push-down window on `ts_start`, but `ts_end` has no
/// constraint and the server may return rows that DataFusion later filters out by
/// post-fetch DataFusion evaluation. With early-stop active, those rows may never be
/// fetched — the result set could be incorrect.
///
/// The current code (v1.7) collects ALL INDEX datetime col names:
///   `datetime_index_cols = ["ts_start", "ts_end"]`
///
/// The predicate `ts_start > '2026-01-01'` satisfies:
///   `is_pushed_temporal_predicate` → `true` (ts_start ∈ datetime_index_cols, range op, ts RHS)
///   `has_client_side_where = false` → PERMIT → `fetch_limit = 100`.
///
/// Condition K says: when `|datetime_index_cols| > 1`, suppress early-stop regardless,
/// because multi-column INDEX tables have ambiguous push-down coverage.
///
/// ## RED / GREEN mechanics
///
/// RED (current v1.7 — no Condition K):
///   `datetime_index_cols = ["ts_start", "ts_end"]`; single predicate on ts_start →
///   `is_pushed_temporal_predicate` = true → PERMIT → `last_limit = 100 ≠ 0`.
///   Assertion FAILS.
///
/// GREEN (v1.8 Condition K):
///   `|datetime_index_cols| > 1` → SUPPRESS → `last_limit = 0`.
///   Assertion passes.
///
/// F-LENSB-P13-002: assert `fetch_count >= 1` before `last_limit == 0`.
///
/// SAP-3: end-to-end from `run_materialization_pipeline` with a real SQL query string.
/// Anchors: S-ENGINE-LIMIT-EARLY-STOP-001 RG-PSG-030d, ADR-060 v1.8 §D8.7 Condition K.
#[tokio::test]
async fn test_psg_rg030d_multi_index_datetime_suppresses_early_stop() {
    // Spec map: mock sensor with TWO Datetime+INDEX columns (ts_start, ts_end).
    // Current v1.7: datetime_index_cols = ["ts_start", "ts_end"] → PERMIT for ts_start query.
    // After Condition K: |datetime_index_cols| > 1 → SUPPRESS.
    let spec_map = make_psg_multi_index_spec_map();
    let (mut mat_ctx, last_limit, fetch_count) = plan_gate_mat_ctx_with_spec(spec_map);
    let session_ctx =
        build_session_context(QUERY_MEMORY_POOL_BYTES).expect("build_session_context");

    // ts_start is INDEX, ts_end is INDEX — two INDEX datetime cols on the same table.
    // Gate must SUPPRESS when Condition K is implemented.
    let query = "SELECT * FROM mock_events \
                 WHERE ts_start > '2026-01-01T00:00:00Z' \
                 LIMIT 100";
    let _result = run_materialization_pipeline(query, &opts(100), &mut mat_ctx, &session_ctx).await;

    // F-LENSB-P13-002: adapter must have been invoked.
    let fc = fetch_count.load(Ordering::SeqCst);
    assert!(
        fc >= 1,
        "PSG-030d (Condition K — multi-index-datetime SUPPRESS): adapter must have been \
         called at least once (fetch_count={fc}). A count of 0 means last_limit=0 is the \
         AtomicU64 init value, not evidence of gate suppression."
    );

    // PRIMARY: gate must SUPPRESS early-stop (fetch_limit = 0).
    //
    // RED (current v1.7 — no Condition K):
    //   datetime_index_cols = ["ts_start", "ts_end"]; ts_start ∈ set → PERMIT →
    //   last_limit = 100 ≠ 0. Assertion FAILS.
    //
    // GREEN (v1.8 Condition K):
    //   |datetime_index_cols| > 1 for this table → SUPPRESS → last_limit = 0. Passes.
    let seen_limit = last_limit.load(Ordering::SeqCst);
    assert_eq!(
        seen_limit, 0,
        "PSG-030d (ADR-060 v1.8 §D8.7 — Condition K multi-index-datetime SUPPRESS): \
         gate must SUPPRESS early-stop when the queried table has multiple INDEX datetime \
         columns (fetch_limit=0); adapter saw params.limit={seen_limit}. \
         If 100: Condition K is not yet implemented — the gate PERMITs because ts_start \
         is individually in datetime_index_cols, but a table with multiple INDEX datetime \
         cols has ambiguous push-down coverage and early-stop is incorrect. \
         Anchors: S-ENGINE-LIMIT-EARLY-STOP-001 RG-PSG-030d, ADR-060 v1.8 §D8.7 Condition K."
    );
}
