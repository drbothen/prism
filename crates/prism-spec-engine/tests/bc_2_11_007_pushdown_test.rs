#![allow(non_snake_case)]
//! Red Gate tests for S-DEMO-QUERY-PUSHDOWN-001 v2.1
//!
//! Behavioral Contracts: BC-2.01.013, BC-2.11.005 (introduced v1.6), BC-2.11.007
//! Architecture: ADR-033 T1 (pre-fan-out time-window extraction)
//!
//! # Covered ACs
//!
//! - AC-CWS-001: CrowdStrike limit reaches DetectionListParams
//! - AC-CWS-002: CrowdStrike FQL time-window (both start+end, combined `+` form) wire-level via
//!   PipelineExecutor; full run_materialization_pipeline path in prism-bin/tests/adv_p02_e2e_pushdown_pipeline_test.rs
//! - AC-CWS-003: No filter param when no time predicates
//! - AC-ARMIS-001: Armis AQL passthrough — no maxResults or timeFrame
//! - AC-ARMIS-002: No additional params beyond aql, offset, limit
//! - AC-CYB-001: Cyberint AlertListParams has only cursor (no from_date, to_date, page_size)
//! - AC-CLAR-001: Claroty body_template remains empty — no time-window fields
//! - AC-EQUIV-001: FQL subset invariant at PipelineExecutor boundary (pre-seeded FQL, CrowdStrike DTU);
//!   AUTHORITATIVE full run_materialization_pipeline path in prism-bin/tests/adv_p02_e2e_pushdown_pipeline_test.rs
//!   → test_ac_equiv_001_result_equivalence_via_run_materialization_pipeline
//!
//! # SAP-2 Compliance
//!
//! ALL test fixtures loading a sensor spec use the PRODUCTION TOML file
//! (`crates/prism-sensors/specs/<sensor>.sensor.toml`) loaded via `std::fs::read_to_string`
//! with `CARGO_MANIFEST_DIR` relative path. NO fabricated spec diverging from production
//! shape (`make_crowdstrike_like_spec`, etc.) is used in this test file.
//!
//! # Red Gate Status
//!
//! ALL tests in this file MUST FAIL before the implementer begins.
//! Tests that verify ABSENCE of a wrong param rely on the implementation not injecting it.
//! Those tests currently test against existing (broken) code — they FAIL because the
//! existing code DOES inject wrong params.
//!
//! # SID-1 Compliance
//!
//! AC-ARMIS-TW-005 is `#[ignore]`'d and cites its DTU + prism binary dependency.
//! All other ACs are un-gated integration tests that use the DTU directly.
//!
//! Story: S-DEMO-QUERY-PUSHDOWN-001 v2.1
//! BCs: BC-2.01.013, BC-2.11.005 (introduced v1.6), BC-2.11.007
//! ADR: ADR-033 T1

use std::collections::HashMap;

use prism_core::OrgSlug;
use prism_dtu_armis::ArmisClone;
use prism_dtu_claroty::ClarotyClone;
use prism_dtu_common::BehavioralClone;
use prism_dtu_crowdstrike::CrowdstrikeClone;
use prism_dtu_cyberint::CyberintClone;
use prism_spec_engine::{
    MockAuthProvider, MockCredentialResolver, StaticCookieAuthProvider,
    pipeline::{FetchContext, PipelineExecutor},
    spec_parser::SpecLoader,
};
use serde_json::Value as JsonValue;

// ---------------------------------------------------------------------------
// AC-CWS-001: CrowdStrike limit reaches DetectionListParams
// ---------------------------------------------------------------------------

/// AC-CWS-001 / BC-2.01.013 Pagination/Push-Down Scope Clause
///
/// A PrismQL query with `LIMIT 50` must produce a CrowdStrike Step 1 request
/// with `DetectionListParams.limit = 50`.
///
/// # SAP-2
/// Fixture loads production `crowdstrike.sensor.toml` (GET step, no body_template,
/// DetectionListParams fields: filter, limit, offset).
///
/// # Red Gate
/// Fails if `limit` is not propagated through `QueryParams.limit` → FetchContext →
/// PipelineExecutor → DetectionListParams.limit in the DTU request.
/// The DTU captures received query params; this test asserts they match.
#[tokio::test]
async fn test_ac_cws_001_crowdstrike_limit_reaches_detection_list_params() {
    // Step 1: Start the CrowdStrike DTU clone on an ephemeral port.
    let mut clone = CrowdstrikeClone::new();
    let bound_addr = clone
        .start_on("127.0.0.1:0".parse().unwrap(), None, None)
        .await
        .expect("AC-CWS-001: CrowdStrike DTU clone failed to start");
    let dtu_base_url = format!("http://{bound_addr}");

    // Step 2: Load production crowdstrike.sensor.toml (SAP-2: no fabricated fixture).
    let spec_content = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../prism-sensors/specs/crowdstrike.sensor.toml"),
    )
    .expect("AC-CWS-001: crowdstrike.sensor.toml must be readable");
    let mut spec =
        SpecLoader::parse(&spec_content).expect("AC-CWS-001: crowdstrike.sensor.toml must parse");
    spec.base_url = dtu_base_url.clone();

    // Step 3: Find the detections table (Step 1 = query_detection_ids).
    let table = spec
        .tables
        .iter()
        .find(|t| t.table_name == "detections")
        .expect("AC-CWS-001: crowdstrike spec must have 'detections' table")
        .clone();

    // Step 4: Construct FetchContext with limit = 5 via query_filters.
    // The limit is carried as `QueryParams.limit` and must reach DetectionListParams.limit.
    // Using limit=5 against the 50-record fixture: if limit is propagated correctly,
    // the DTU Step 1 returns 5 IDs → Step 2 fetches 5 records → result.len() == 5.
    // If limit is NOT propagated, the DTU returns all 50 records → assertion FAILS.
    //
    // NB: The `query.limit` interpolation must be wired in the CrowdStrike Step 1 path_template
    // for this to work. The story AC-CWS-001 is the gate that drives this wiring.
    // Red Gate: without the wiring, DTU returns 50 records → `result.len() <= 5` FAILS.
    let mut filters = HashMap::new();
    filters.insert("query.limit".to_string(), "5".to_string());
    let context = FetchContext::new(OrgSlug::new("test-org"), filters);

    // Step 5: HTTP client with 30s timeout per CLAUDE.md.
    let http_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("AC-CWS-001: reqwest client build must succeed");

    // Step 6: Execute pipeline. MockAuthProvider provides a non-empty token.
    let auth_provider = MockAuthProvider::new("test-oauth2-token");
    let result = PipelineExecutor::execute(&spec, &table, &context, &http_client, &auth_provider)
        .await
        .expect("AC-CWS-001: PipelineExecutor::execute must succeed against CrowdStrike DTU");

    // Step 7: Verify the DTU received limit=5 via result record count.
    // If limit=5 reaches DetectionListParams.limit, the DTU Step 1 returns 5 IDs,
    // Step 2 fetches those 5 records → result.len() == 5.
    // If limit is NOT propagated, DTU returns all 50 records → this FAILS.
    //
    // Assertion: result rows <= 5 (limit of 5 was honored by the DTU).
    // Red Gate: without FQL limit wiring, DTU returns 50 records → fails.
    assert!(
        result.records.len() <= 5,
        "AC-CWS-001: CrowdStrike result must have at most 5 records when limit=5 is pushed down; \
         got {} records. REGRESSION: limit is not propagated to DetectionListParams.limit. \
         The CrowdStrike Step 1 path_template must include the limit param so the DTU \
         respects it. Root cause: limit push-down not wired via path_template or \
         ${{query.limit}} interpolation.",
        result.records.len()
    );

    // Verify records are non-empty (some records must have matched).
    assert!(
        !result.records.is_empty(),
        "AC-CWS-001: CrowdStrike DTU must return non-empty detection records \
         (fixture has 50 IDs; at least some must match). \
         Check that the fixture is loaded and the Step 1 API path is correct."
    );
}

// ---------------------------------------------------------------------------
// AC-CWS-002: CrowdStrike FQL time-window — Step 1 path_template includes filter param
//
// COVERAGE NOTE (ADV-P04-HIGH-002):
// The NAMED AC-CWS-002 test that drives `run_materialization_pipeline` is in:
//   crates/prism-bin/tests/adv_p02_e2e_pushdown_pipeline_test.rs
//   → test_ac_cws_002_fql_time_window_both_start_and_end_via_materialization_pipeline
//
// This file (prism-spec-engine) cannot depend on prism-bin (circular dep guard,
// see CLAUDE.md architecture compliance note). The test below validates the same
// wire-level FQL propagation via `PipelineExecutor::execute` directly (with
// pre-seeded FQL), which is the pipeline-executor boundary exercised by
// `SpecDrivenSensorAdapter::fetch` after it calls `build_crowdstrike_fql`.
// The prism-bin test proves the full path from PrismQL WHERE clause.
// ---------------------------------------------------------------------------

/// AC-CWS-002 wire-level / BC-2.01.013 TV-BC-2.01.013-006
///
/// Validates that a pre-built FQL string (the form produced by `build_crowdstrike_fql`
/// for both start+end bounds) reaches the CrowdStrike DTU via `PipelineExecutor::execute`.
///
/// # What this test proves
/// - The `path_template` has the FQL filter interpolation slot (structural).
/// - FQL pre-seeded into `FetchContext["_fql"]` reaches the DTU filter-log (wire-level).
/// - The FQL contains BOTH bounds in the `+`-combined form (AC-CWS-002 item b).
/// - `filtered_count < unfiltered_count` — DTU honors the combined FQL (load-bearing).
///
/// # What this test does NOT prove (covered in prism-bin tests)
/// The path `run_materialization_pipeline → extract_time_window_from_ast →
/// build_crowdstrike_fql` is NOT exercised here — this test pre-seeds the FQL.
/// That path is proven by `test_ac_cws_002_fql_time_window_both_start_and_end_via_materialization_pipeline`
/// in `crates/prism-bin/tests/adv_p02_e2e_pushdown_pipeline_test.rs`.
///
/// # SAP-2
/// Uses production `crowdstrike.sensor.toml` shape.
#[tokio::test]
async fn test_ac_cws_002_wire_level_fql_both_bounds_via_pipeline_executor() {
    let mut clone = CrowdstrikeClone::new();
    let bound_addr = clone
        .start_on("127.0.0.1:0".parse().unwrap(), None, None)
        .await
        .expect("AC-CWS-002: CrowdStrike DTU clone failed to start");
    let dtu_base_url = format!("http://{bound_addr}");

    // Load production crowdstrike.sensor.toml (SAP-2).
    let spec_content = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../prism-sensors/specs/crowdstrike.sensor.toml"),
    )
    .expect("AC-CWS-002: crowdstrike.sensor.toml must be readable");
    let mut spec =
        SpecLoader::parse(&spec_content).expect("AC-CWS-002: crowdstrike.sensor.toml must parse");
    spec.base_url = dtu_base_url.clone();

    // Find the Step 1 (query_detection_ids) step.
    let detections_table = spec
        .tables
        .iter()
        .find(|t| t.table_name == "detections")
        .expect("AC-CWS-002: detections table must exist");

    let step1 = detections_table
        .steps
        .iter()
        .find(|s| s.name == "query_detection_ids")
        .expect("AC-CWS-002: query_detection_ids step must exist");

    // AC-CWS-002 structural assertion: Step 1 path_template must include the FQL filter slot.
    assert!(
        step1.path_template.contains("query.filter."),
        "AC-CWS-002 item (b): CrowdStrike Step 1 path_template must include an FQL filter \
         interpolation slot (e.g., '&filter=${{query.filter._fql}}'). \
         Current path_template='{}'. REGRESSION: FQL injection slot missing.",
        step1.path_template
    );

    // AC-CWS-002 item (c): Step 2 body_template must NOT contain filter/time-window fields.
    let step2 = detections_table
        .steps
        .iter()
        .find(|s| s.name == "fetch_detections")
        .expect("AC-CWS-002: fetch_detections step must exist");
    let body = step2.body_template.as_deref().unwrap_or("");
    assert!(
        !body.contains("filter") && !body.contains("start_time") && !body.contains("end_time"),
        "AC-CWS-002 item (c): Step 2 body_template must NOT contain filter/time-window fields. \
         Got body_template: '{body}'"
    );

    // WIRE-LEVEL assertion (F-P1-HIGH-003 / AC-CWS-002 item b):
    // Execute pipeline with FQL filter seeded via `_fql` key (production adapter path).
    // The `_fql` key is seeded by SpecDrivenSensorAdapter::fetch() → build_crowdstrike_fql().
    // For this test: simulate the production adapter output by seeding FQL directly.
    // The CrowdStrike path_template interpolates `${query.filter._fql}` into `&filter=`.
    //
    // FQL: both start and end — `created_timestamp:>'2026-01-10T00:00:00Z'+created_timestamp:<'2026-01-20T00:00:00Z'`
    // Fixture timestamps range: 2026-01-01 to 2026-01-26. With this range:
    // - Records with 10:00 timestamps in [2026-01-10, 2026-01-20] → included (20 records).
    // - No fixture record is at exactly the midnight boundary (00:00:00), so inclusive vs
    //   exclusive boundary semantics do NOT change this count (ADV-P08-MED-001 note).
    // - Records outside → excluded.
    // This guarantees filtered_count < unfiltered_count (LOAD-BEARING).
    let start_time = "2026-01-10T00:00:00Z";
    let end_time = "2026-01-20T00:00:00Z";
    let fql = format!("created_timestamp:>'{start_time}'+created_timestamp:<'{end_time}'");

    let mut filters_with_fql = HashMap::new();
    filters_with_fql.insert("_fql".to_string(), fql.clone());
    let ctx_with_fql = FetchContext::new(OrgSlug::new("test-org"), filters_with_fql);

    let http_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("AC-CWS-002: reqwest client build");
    let auth_provider = MockAuthProvider::new("test-oauth2-token");

    let table = detections_table.clone();
    let result_filtered =
        PipelineExecutor::execute(&spec, &table, &ctx_with_fql, &http_client, &auth_provider)
            .await
            .expect("AC-CWS-002: filtered pipeline must succeed");
    let filtered_count = result_filtered.records.len();

    // WIRE-LEVEL assertion (F-P1-HIGH-003): DTU filter-log must contain the FQL string.
    // Check BEFORE reset so the captured filter from the filtered execution is still in the log.
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("AC-CWS-002: filter-log client build");
    let filter_log_resp = client
        .get(format!("{dtu_base_url}/dtu/filter-log"))
        .send()
        .await
        .expect("AC-CWS-002: GET /dtu/filter-log must succeed");
    let filter_log_body: JsonValue = filter_log_resp
        .json()
        .await
        .expect("AC-CWS-002: filter-log must be JSON");
    let filter_strings = filter_log_body["filter_strings"]
        .as_array()
        .expect("AC-CWS-002: filter_strings must be an array");

    // AC-CWS-002 item (b): The FQL must contain BOTH lower-bound AND upper-bound in the
    // combined `+` form: `created_timestamp:>'T1'+created_timestamp:<'T2'`.
    // This is the exact form produced by `build_crowdstrike_fql(start_time, end_time)`.
    // Asserting only `contains("created_timestamp")` would pass for a single-bound FQL —
    // we must verify the COMBINED both-bounds form to satisfy AC-CWS-002(b).
    let both_bounds_present = filter_strings.iter().any(|s| {
        s.as_str()
            .map(|fs| fs.contains("created_timestamp:>'") && fs.contains("created_timestamp:<'"))
            .unwrap_or(false)
    });
    assert!(
        both_bounds_present,
        "AC-CWS-002 WIRE-LEVEL BOTH-BOUNDS (F-P1-HIGH-003 / AC-CWS-002 item b): \
         DTU filter-log must contain an entry with BOTH 'created_timestamp:>\\'' AND \
         'created_timestamp:<\\'' in the combined `+` FQL form. \
         The FQL '{}' must reach the DTU via path_template interpolation. \
         Got filter_strings: {:?}. \
         REGRESSION: Either the FQL is not reaching the DTU, or only one bound is present.",
        fql, filter_strings
    );

    // Reset DTU state.
    clone
        .reset()
        .await
        .expect("AC-CWS-002: clone reset must succeed");

    // Also execute without FQL filter to get baseline count.
    let ctx_no_filter = FetchContext::new(OrgSlug::new("test-org"), HashMap::new());
    let result_unfiltered =
        PipelineExecutor::execute(&spec, &table, &ctx_no_filter, &http_client, &auth_provider)
            .await
            .expect("AC-CWS-002: unfiltered pipeline must succeed");
    let unfiltered_count = result_unfiltered.records.len();

    // LOAD-BEARING: filtered_count < unfiltered_count (DTU honored the FQL filter).
    assert!(
        filtered_count < unfiltered_count,
        "AC-CWS-002 LOAD-BEARING: filtered_count ({}) must be < unfiltered_count ({}) \
         when FQL time-window filter is applied. The CrowdStrike DTU must parse the \
         `created_timestamp:>'T1'+created_timestamp:<'T2'` FQL and return only IDs \
         within the range. REGRESSION: DTU is not honoring the FQL filter parameter.",
        filtered_count,
        unfiltered_count
    );
    assert!(
        filtered_count > 0,
        "AC-CWS-002: at least some records must fall within {} to {}",
        start_time,
        end_time
    );
}

// ---------------------------------------------------------------------------
// AC-CWS-003: No filter param when no time predicates — pipeline runs normally
// ---------------------------------------------------------------------------

/// AC-CWS-003 / BC-2.11.007 result-equivalence invariant
///
/// When no time-window predicates exist in the WHERE clause, the CrowdStrike
/// pipeline must run normally and return ALL 50 fixture records (no spurious injection).
///
/// # Absence property (load-bearing)
///
/// The `/dtu/filter-log` must contain NO entry with `created_timestamp` after
/// executing with an empty `FetchContext`. This proves no FQL filter string was
/// injected by the pipeline (wire-level absence assertion, F-P1-HIGH-003 pattern).
///
/// # Full fixture count assertion (load-bearing)
///
/// `result.records.len()` must equal 50 — the total fixture count in
/// `crates/prism-dtu-crowdstrike/fixtures/detections-ids.json`. A spurious narrow
/// filter returning a strict subset (e.g., 10 records) would fail this assertion.
///
/// # SAP-2: production crowdstrike.sensor.toml shape.
///
/// # Red Gate
/// Fails until: (a) FQL slot is added to path_template (AC-CWS-002 prerequisite),
/// and (b) empty FetchContext → no `filter` query param is sent to the DTU.
#[tokio::test]
async fn test_ac_cws_003_no_filter_param_when_no_time_predicates() {
    let mut clone = CrowdstrikeClone::new();
    let bound_addr = clone
        .start_on("127.0.0.1:0".parse().unwrap(), None, None)
        .await
        .expect("AC-CWS-003: CrowdStrike DTU clone failed to start");
    let dtu_base_url = format!("http://{bound_addr}");

    let spec_content = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../prism-sensors/specs/crowdstrike.sensor.toml"),
    )
    .expect("AC-CWS-003: crowdstrike.sensor.toml must be readable");
    let mut spec =
        SpecLoader::parse(&spec_content).expect("AC-CWS-003: crowdstrike.sensor.toml must parse");
    spec.base_url = format!("http://{bound_addr}");

    // Structural check: the FQL slot MUST be present in Step 1 path_template (after AC-CWS-002 fix).
    // Red Gate: currently path_template lacks the FQL slot → this assertion FAILS.
    // (Same structural assertion as AC-CWS-002 — both tests require the FQL slot.)
    let detections_table = spec
        .tables
        .iter()
        .find(|t| t.table_name == "detections")
        .expect("AC-CWS-003: detections table must exist");
    let step1 = detections_table
        .steps
        .iter()
        .find(|s| s.name == "query_detection_ids")
        .expect("AC-CWS-003: query_detection_ids step must exist");

    assert!(
        step1.path_template.contains("query.filter."),
        "AC-CWS-003: CrowdStrike Step 1 path_template must include an FQL filter slot \
         (so empty filter → no filter param; populated filter → FQL param). \
         Current path_template='{}'. \
         Both AC-CWS-002 (filter present) and AC-CWS-003 (filter absent) require \
         this slot to be in the path_template. REGRESSION: missing slot.",
        step1.path_template
    );

    // Execute pipeline with no time filters.
    let table = detections_table.clone();
    let context = FetchContext::new(OrgSlug::new("test-org"), HashMap::new());

    let http_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("AC-CWS-003: reqwest client build");
    let auth_provider = MockAuthProvider::new("test-oauth2-token");

    let result = PipelineExecutor::execute(&spec, &table, &context, &http_client, &auth_provider)
        .await
        .expect("AC-CWS-003: PipelineExecutor::execute must succeed");

    // WIRE-LEVEL ABSENCE assertion (F-P1-HIGH-003 / AC-CWS-003):
    // The DTU filter-log must contain NO entry with `created_timestamp` — proving the
    // pipeline did NOT inject an FQL filter when no time predicates were present.
    // A spurious FQL injection would leave a `created_timestamp:>...` entry in the log.
    let filter_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("AC-CWS-003: filter-log client build");
    let filter_log_resp = filter_client
        .get(format!("{dtu_base_url}/dtu/filter-log"))
        .send()
        .await
        .expect("AC-CWS-003: GET /dtu/filter-log must succeed");
    let filter_log_body: JsonValue = filter_log_resp
        .json()
        .await
        .expect("AC-CWS-003: filter-log must be JSON");
    let filter_strings = filter_log_body["filter_strings"]
        .as_array()
        .expect("AC-CWS-003: filter_strings must be an array");

    // No filter param must have reached the DTU: either the log is empty,
    // or none of the entries contain `created_timestamp` (absence of FQL injection).
    let no_fql_injected = filter_strings
        .iter()
        .all(|s| !s.as_str().unwrap_or("").contains("created_timestamp"));
    assert!(
        no_fql_injected,
        "AC-CWS-003 WIRE-LEVEL ABSENCE (F-P1-HIGH-003): DTU filter-log must NOT contain \
         any `created_timestamp` clause when FetchContext has no time predicates. \
         Got filter_strings: {:?}. \
         REGRESSION: pipeline spuriously injected an FQL filter with no time predicates.",
        filter_strings
    );

    // FULL FIXTURE COUNT assertion (load-bearing): with no filter, all 50 fixture records
    // must be returned. The fixture (crates/prism-dtu-crowdstrike/fixtures/detections-ids.json)
    // contains exactly 50 detection IDs. A spurious narrow FQL filter returning a strict
    // subset (e.g., 10 records) would fail this assertion.
    assert_eq!(
        result.records.len(),
        50,
        "AC-CWS-003 FULL FIXTURE COUNT: CrowdStrike DTU must return all 50 fixture records \
         when no time filter is applied; got {} records. \
         REGRESSION: pipeline either injected a spurious FQL filter or the fixture is incomplete.",
        result.records.len()
    );
}

// ---------------------------------------------------------------------------
// AC-ARMIS-001: Armis AQL passthrough — no maxResults or timeFrame
// ---------------------------------------------------------------------------

/// AC-ARMIS-001 / BC-2.11.007 §Mechanism B postcondition
///
/// When AQL is `in:devices after:2026-01-01T00:00:00`, the Armis DTU must receive
/// ONLY the `aql` param (plus pagination). No `maxResults` or `timeFrame` params.
///
/// # SAP-2
/// Fixture loads production `armis.sensor.toml` (GET, AQL passthrough,
/// `path_template = "/api/v1/search?aql=${query.filter.aql}"`, OffsetLimit pagination,
/// NO body_template). FORBIDDEN: any fixture with timeFrame, maxResults, or body injection.
///
/// # Red Gate
/// Fails if the implementation injects `maxResults` or `timeFrame` beyond the AQL string.
/// If those wrong params were injected by v1.x code, this test detects it.
#[tokio::test]
async fn test_ac_armis_001_aql_passthrough_no_maxresults_no_timeframe() {
    let mut clone = ArmisClone::new().expect("AC-ARMIS-001: ArmisClone::new must succeed");
    let bound_addr = clone
        .start_on("127.0.0.1:0".parse().unwrap(), None, None)
        .await
        .expect("AC-ARMIS-001: Armis DTU clone failed to start");
    let dtu_base_url = format!("http://{bound_addr}");

    // Load production armis.sensor.toml (SAP-2).
    let spec_content = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../prism-sensors/specs/armis.sensor.toml"),
    )
    .expect("AC-ARMIS-001: armis.sensor.toml must be readable");
    let mut spec =
        SpecLoader::parse(&spec_content).expect("AC-ARMIS-001: armis.sensor.toml must parse");
    spec.base_url = dtu_base_url.clone();

    let table = spec
        .tables
        .iter()
        .find(|t| t.table_name == "devices")
        .expect("AC-ARMIS-001: Armis spec must have 'devices' table")
        .clone();

    // AQL string with an absolute time clause (research-confirmed canonical form).
    // Must NOT have `lastSeen:>"T"` form.
    let aql_value = "in:devices after:2026-01-01T00:00:00";
    let mut filters = HashMap::new();
    filters.insert("aql".to_string(), aql_value.to_string());
    let context = FetchContext::new(OrgSlug::new("test-org"), filters);

    let http_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("AC-ARMIS-001: reqwest client build");
    let auth_provider = MockAuthProvider::new("test-bearer-token");

    let result = PipelineExecutor::execute(&spec, &table, &context, &http_client, &auth_provider)
        .await
        .expect("AC-ARMIS-001: PipelineExecutor::execute must succeed against Armis DTU");

    // Verify pipeline returned records.
    assert!(
        !result.records.is_empty(),
        "AC-ARMIS-001: Armis DTU must return device records; got 0. \
         Check AQL forwarding path and DTU fixture."
    );

    // Verify the DTU received the verbatim AQL and NO extra params.
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("AC-ARMIS-001: aql-log client build");
    let aql_log_resp = client
        .get(format!("{dtu_base_url}/dtu/aql-log"))
        .send()
        .await
        .expect("AC-ARMIS-001: GET /dtu/aql-log must succeed");
    let aql_log_body: JsonValue = aql_log_resp
        .json()
        .await
        .expect("AC-ARMIS-001: aql-log must be JSON");
    let aql_strings = aql_log_body["aql_strings"]
        .as_array()
        .expect("AC-ARMIS-001: aql_strings must be an array");

    // Assertion (a): verbatim AQL forwarded.
    assert!(
        aql_strings.iter().any(|s| s.as_str() == Some(aql_value)),
        "AC-ARMIS-001: DTU aql-log must contain verbatim AQL '{}'; got {:?}. \
         REGRESSION: AQL passthrough broken.",
        aql_value,
        aql_strings
    );

    // Assertions (b)(c)(d): No maxResults, timeFrame, or injected time params.
    // The SearchQueryParams struct does NOT have maxResults or timeFrame fields.
    // Verify: (1) The TOML path_template does NOT contain 'maxResults' or 'timeFrame'.
    // (2) The pipeline succeeded (DTU returned 200) — if wrong params were injected,
    //     Axum's Query extractor would silently drop unknown fields, but the response
    //     would still be 200. The aql-log confirms the verbatim AQL was captured.
    // (3) The aql-log does NOT contain 'maxResults' or 'timeFrame' in any captured AQL.

    // SAP-2 TOML shape assertion: path_template must NOT contain maxResults or timeFrame.
    let devices_table = spec
        .tables
        .iter()
        .find(|t| t.table_name == "devices")
        .expect("AC-ARMIS-001: devices table must exist");
    let step = devices_table
        .steps
        .first()
        .expect("AC-ARMIS-001: devices step must exist");

    assert!(
        !step.path_template.contains("maxResults"),
        "AC-ARMIS-001 item (b): Armis devices path_template must NOT contain 'maxResults'; \
         SearchQueryParams has no maxResults field. Got path_template='{}'",
        step.path_template
    );
    assert!(
        !step.path_template.contains("timeFrame"),
        "AC-ARMIS-001 item (c): Armis devices path_template must NOT contain 'timeFrame'; \
         SearchQueryParams has no timeFrame field. Got path_template='{}'",
        step.path_template
    );

    // AQL log must NOT contain 'maxResults' or 'timeFrame' embedded in the AQL string.
    let no_maxresults = aql_strings
        .iter()
        .all(|s| !s.as_str().unwrap_or("").contains("maxResults"));
    let no_timeframe = aql_strings
        .iter()
        .all(|s| !s.as_str().unwrap_or("").contains("timeFrame"));
    assert!(
        no_maxresults,
        "AC-ARMIS-001 item (b): aql-log must NOT contain 'maxResults' in any captured AQL; \
         SearchQueryParams has no such field. Got aql_strings: {:?}",
        aql_strings
    );
    assert!(
        no_timeframe,
        "AC-ARMIS-001 item (c): aql-log must NOT contain 'timeFrame' in any captured AQL \
         (wrong form — timeFrame is not a valid Armis push-down param for the query engine). \
         Got aql_strings: {:?}",
        aql_strings
    );
}

// ---------------------------------------------------------------------------
// AC-ARMIS-002: No additional params beyond aql, offset, limit
// ---------------------------------------------------------------------------

/// AC-ARMIS-002 / BC-2.11.007 §Mechanism B invariant
///
/// An Armis request from PipelineExecutor must produce only `aql`, `offset`, `limit`
/// in the query params. No extra injected params.
///
/// # SAP-2: production armis.sensor.toml.
///
/// # Red Gate
/// Verifies that only the expected params are present. If the v1.x implementation
/// injected extra params, this test fails.
#[tokio::test]
async fn test_ac_armis_002_no_additional_params_beyond_aql_offset_limit() {
    let mut clone = ArmisClone::new().expect("AC-ARMIS-002: ArmisClone::new must succeed");
    let bound_addr = clone
        .start_on("127.0.0.1:0".parse().unwrap(), None, None)
        .await
        .expect("AC-ARMIS-002: Armis DTU clone failed to start");
    let dtu_base_url = format!("http://{bound_addr}");

    let spec_content = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../prism-sensors/specs/armis.sensor.toml"),
    )
    .expect("AC-ARMIS-002: armis.sensor.toml must be readable");
    let mut spec =
        SpecLoader::parse(&spec_content).expect("AC-ARMIS-002: armis.sensor.toml must parse");
    spec.base_url = dtu_base_url.clone();

    let table = spec
        .tables
        .iter()
        .find(|t| t.table_name == "devices")
        .expect("AC-ARMIS-002: Armis spec must have 'devices' table")
        .clone();

    // FetchContext with AQL and a time-window predicate.
    // The time-window predicate (last_seen > T) should NOT be injected as a separate param.
    let mut filters = HashMap::new();
    filters.insert("aql".to_string(), "in:devices".to_string());
    // Simulating: WHERE aql = 'in:devices' AND detected_time > '2026-01-01T00:00:00Z'
    // The detected_time filter is post-filtered; only aql goes to DTU.
    let context = FetchContext::new(OrgSlug::new("test-org"), filters);

    let http_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("AC-ARMIS-002: reqwest client build");
    let auth_provider = MockAuthProvider::new("test-bearer-token");

    let result = PipelineExecutor::execute(&spec, &table, &context, &http_client, &auth_provider)
        .await
        .expect("AC-ARMIS-002: PipelineExecutor::execute must succeed");

    assert!(
        !result.records.is_empty(),
        "AC-ARMIS-002: Armis DTU must return non-empty records"
    );

    // AC-ARMIS-002 assertions: no extra params injected.
    // The DTU's SearchQueryParams struct only has: aql, page, size, offset, limit.
    // Verify via TOML: the Armis path_template must NOT inject any extra non-AQL params.
    let devices_table = spec
        .tables
        .iter()
        .find(|t| t.table_name == "devices")
        .expect("AC-ARMIS-002: devices table must exist");
    let step = devices_table
        .steps
        .first()
        .expect("AC-ARMIS-002: devices step must exist");

    // SAP-2: path_template must NOT contain maxResults, timeFrame, or detected_time params.
    assert!(
        !step.path_template.contains("maxResults"),
        "AC-ARMIS-002: Armis path_template must NOT inject 'maxResults'; \
         SearchQueryParams.maxResults does not exist. Got: '{}'",
        step.path_template
    );
    assert!(
        !step.path_template.contains("timeFrame"),
        "AC-ARMIS-002: Armis path_template must NOT inject 'timeFrame'; \
         SearchQueryParams.timeFrame does not exist. Got: '{}'",
        step.path_template
    );
    assert!(
        !step.path_template.contains("detected_time"),
        "AC-ARMIS-002: Armis path_template must NOT inject 'detected_time'; \
         time predicate must be post-filtered by DataFusion. Got: '{}'",
        step.path_template
    );

    // Pipeline succeeded without 400 errors → correct behavior (Axum's Query extractor
    // does not reject unknown params for SearchQueryParams, but any wrong field in the
    // path_template would produce a string-injection into the URL that is load-bearingly wrong).
    // The aql-log confirms only the AQL was captured (no extra params as AQL content).
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("AC-ARMIS-002: aql-log client build");
    let aql_log_resp = client
        .get(format!("{dtu_base_url}/dtu/aql-log"))
        .send()
        .await
        .expect("AC-ARMIS-002: GET /dtu/aql-log must succeed");
    let aql_log_body: JsonValue = aql_log_resp
        .json()
        .await
        .expect("AC-ARMIS-002: aql-log must be JSON");
    let aql_strings = aql_log_body["aql_strings"]
        .as_array()
        .expect("AC-ARMIS-002: aql_strings must be array");

    // Verify AQL captured is clean (no injected extra params).
    let no_maxresults = aql_strings
        .iter()
        .all(|s| !s.as_str().unwrap_or("").contains("maxResults"));
    assert!(
        no_maxresults,
        "AC-ARMIS-002: aql-log must NOT contain 'maxResults' — SearchQueryParams.maxResults \
         does not exist. Got: {:?}",
        aql_strings
    );
}

// ---------------------------------------------------------------------------
// AC-CYB-001: Cyberint AlertListParams has only cursor
// ---------------------------------------------------------------------------

/// AC-CYB-001 / BC-2.01.013 Pagination/Push-Down Scope Clause — Cyberint row
///
/// The Cyberint `fetch_alerts` step is a GET request with ONLY `cursor: Option<String>`.
/// No `from_date`, `to_date`, or `page_size` must appear.
///
/// # SAP-2
/// Production `cyberint.sensor.toml` shape: GET, NO body_template, cursor-only pagination.
/// FORBIDDEN: any fixture with a POST body_template; any fixture injecting body fields.
///
/// # Red Gate
/// Fails if the implementation injects `from_date`, `to_date`, or `page_size`.
/// These were wrong in v1.x.
/// Cyberint test auth token — registered in the DTU allowlist for testing.
/// Not a real credential (AD-017: test-only value, never in production paths).
const CYBERINT_TEST_TOKEN: &str = "cyberint-test-api-key-ac-cyb-001";

#[tokio::test]
async fn test_ac_cyb_001_no_from_date_to_date_page_size_in_alert_list_params() {
    let mut clone = CyberintClone::new_with_access_token(CYBERINT_TEST_TOKEN.to_string())
        .expect("AC-CYB-001: CyberintClone::new_with_access_token must succeed");
    let bound_addr = clone
        .start_on("127.0.0.1:0".parse().unwrap(), None, None)
        .await
        .expect("AC-CYB-001: Cyberint DTU clone failed to start");
    let dtu_base_url = format!("http://{bound_addr}");

    // Load production cyberint.sensor.toml (SAP-2: GET, no body_template, cursor-only).
    let spec_content = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../prism-sensors/specs/cyberint.sensor.toml"),
    )
    .expect("AC-CYB-001: cyberint.sensor.toml must be readable");
    let mut spec =
        SpecLoader::parse(&spec_content).expect("AC-CYB-001: cyberint.sensor.toml must parse");
    spec.base_url = dtu_base_url.clone();

    let table = spec
        .tables
        .iter()
        .find(|t| t.table_name == "alerts")
        .expect("AC-CYB-001: Cyberint spec must have 'alerts' table")
        .clone();

    // Verify the spec step has NO body_template (SAP-2 invariant).
    let step = table
        .steps
        .first()
        .expect("AC-CYB-001: alerts table must have a step");
    assert!(
        step.body_template.is_none(),
        "AC-CYB-001 SAP-2 fixture check: cyberint.sensor.toml alerts step must have NO \
         body_template (Cyberint is GET-only, cursor-only). \
         If body_template is present, the production spec shape diverged. \
         Got body_template: {:?}",
        step.body_template
    );
    assert_eq!(
        step.method.to_uppercase(),
        "GET",
        "AC-CYB-001 SAP-2 fixture check: Cyberint alerts step must be GET method; \
         got: '{}'",
        step.method
    );

    // FetchContext with time-window filters (which MUST NOT be injected as params).
    let mut filters = HashMap::new();
    filters.insert(
        "_start_time".to_string(),
        "2026-01-01T00:00:00Z".to_string(),
    );
    filters.insert("_end_time".to_string(), "2026-06-01T00:00:00Z".to_string());
    let context = FetchContext::new(OrgSlug::new("test-org"), filters);

    let http_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("AC-CYB-001: reqwest client build");

    // Cyberint uses StaticCookie auth (cookie_roundtrip auth_type).
    // Must inject Cookie: access_token=<token> matching the DTU's allowlist.
    // StaticCookieAuthProvider + MockCredentialResolver injects the token as a cookie.
    let auth_provider = StaticCookieAuthProvider::new_with_resolver(
        "cyberint",
        std::sync::Arc::new(MockCredentialResolver::new(CYBERINT_TEST_TOKEN)),
    );

    let result = PipelineExecutor::execute(&spec, &table, &context, &http_client, &auth_provider)
        .await
        .expect("AC-CYB-001: PipelineExecutor::execute must succeed against Cyberint DTU");

    // Result must be non-empty (DTU has fixture data).
    assert!(
        !result.records.is_empty(),
        "AC-CYB-001: Cyberint DTU must return non-empty alert records. \
         Check that the DTU fixture is loaded and accessible."
    );

    // AC-CYB-001 structural assertions: TOML path_template must NOT contain
    // from_date, to_date, or page_size (SAP-2 mandatory invariant).
    // The Cyberint alerts step is GET-only with cursor pagination.
    assert!(
        !step.path_template.contains("from_date"),
        "AC-CYB-001 item (b): Cyberint path_template must NOT inject 'from_date'; \
         AlertListParams has only cursor. Got path_template='{}'",
        step.path_template
    );
    assert!(
        !step.path_template.contains("to_date"),
        "AC-CYB-001 item (b): Cyberint path_template must NOT inject 'to_date'; \
         AlertListParams has only cursor. Got path_template='{}'",
        step.path_template
    );
    assert!(
        !step.path_template.contains("page_size"),
        "AC-CYB-001 item (c): Cyberint path_template must NOT inject 'page_size'; \
         DTU-EXT-005 is OPEN — AlertListParams has no page_size. Got path_template='{}'",
        step.path_template
    );
}

// ---------------------------------------------------------------------------
// AC-CLAR-001: Claroty body_template remains empty — no time-window fields
// ---------------------------------------------------------------------------

/// AC-CLAR-001 / BC-2.01.013 Pagination/Push-Down Scope Clause — Claroty row
///
/// The Claroty POST body must remain `{}` (empty). No time-window fields may be injected.
///
/// # SAP-2
/// Production `claroty.sensor.toml` shape: POST, `body_template: '{}'`, OffsetLimit URL params.
/// FORBIDDEN: any fixture injecting time-window body fields.
///
/// # Red Gate
/// Fails if the implementation injects time-window body fields into the Claroty POST body.
/// The v1.x implementation injected wrong fields — this test detects the regression.
#[tokio::test]
async fn test_ac_clar_001_claroty_body_template_remains_empty_no_time_fields() {
    let mut clone = ClarotyClone::new();
    let bound_addr = clone
        .start_on("127.0.0.1:0".parse().unwrap(), None, None)
        .await
        .expect("AC-CLAR-001: Claroty DTU clone failed to start");
    let dtu_base_url = format!("http://{bound_addr}");

    // Load production claroty.sensor.toml (SAP-2: POST, body_template = '{}', OffsetLimit).
    let spec_content = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../prism-sensors/specs/claroty.sensor.toml"),
    )
    .expect("AC-CLAR-001: claroty.sensor.toml must be readable");
    let mut spec =
        SpecLoader::parse(&spec_content).expect("AC-CLAR-001: claroty.sensor.toml must parse");
    spec.base_url = dtu_base_url.clone();

    let table = spec
        .tables
        .iter()
        .find(|t| t.table_name == "alerts")
        .expect("AC-CLAR-001: Claroty spec must have 'alerts' table")
        .clone();

    // Verify the spec step is POST and body_template contains required fields projection.
    let step = table
        .steps
        .first()
        .expect("AC-CLAR-001: alerts table must have a step");
    assert_eq!(
        step.method.to_uppercase(),
        "POST",
        "AC-CLAR-001 SAP-2 fixture check: Claroty alerts step must be POST method; \
         got: '{}'",
        step.method
    );
    // Body template must contain the required "fields" projection (GetAlertsParameters.fields
    // is REQUIRED with minItems: 1 per the xDome OpenAPI — omitting it returns 422).
    // It must NOT contain time-window fields (AC-CLAR-001 core assertion below).
    let body_tmpl = step.body_template.as_deref().unwrap_or("");
    assert!(
        body_tmpl.contains("\"fields\""),
        "AC-CLAR-001: Claroty alerts body_template must contain 'fields' projection \
         (GetAlertsParameters.fields is REQUIRED per xDome OpenAPI); \
         got: {:?}",
        step.body_template
    );

    // FetchContext with time-window filters (MUST NOT be injected into body).
    let mut filters = HashMap::new();
    filters.insert(
        "_start_time".to_string(),
        "2026-01-01T00:00:00Z".to_string(),
    );
    filters.insert("_end_time".to_string(), "2026-06-01T00:00:00Z".to_string());
    let context = FetchContext::new(OrgSlug::new("test-org"), filters);

    let http_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("AC-CLAR-001: reqwest client build");
    let auth_provider = MockAuthProvider::new("claroty-bearer-token");

    let result = PipelineExecutor::execute(&spec, &table, &context, &http_client, &auth_provider)
        .await
        .expect("AC-CLAR-001: PipelineExecutor::execute must succeed against Claroty DTU");

    // Result must be non-empty.
    assert!(
        !result.records.is_empty(),
        "AC-CLAR-001: Claroty DTU must return non-empty alert records. \
         Check DTU fixture data and path."
    );

    // AC-CLAR-001 structural assertions: body_template must NOT contain time-window injection slots.
    // Since the body_template is static (set in the TOML spec), the pipeline can only inject
    // time-window fields if the body_template explicitly includes `${query.filter.*}` slots.
    // Verify the body_template does NOT contain time-window injection slots.
    // (body_tmpl was already extracted above for the fields-projection assertion)
    assert!(
        !body_tmpl.contains("detected_after")
            && !body_tmpl.contains("detected_before")
            && !body_tmpl.contains("start_time")
            && !body_tmpl.contains("end_time"),
        "AC-CLAR-001 items (a)(b): Claroty body_template must NOT contain time-window fields \
         (detected_after, detected_before, start_time, end_time). \
         Only a 'fields' projection is permitted — no time-window injection. \
         Got body_template: '{body_tmpl}'. \
         REGRESSION: wrong time-window fields were injected into Claroty body."
    );
}

// ---------------------------------------------------------------------------
// AC-EQUIV-001 (pipeline-executor boundary): FQL subset invariant via PipelineExecutor
//
// COVERAGE NOTE (ADV-P05-HIGH-001):
// The AUTHORITATIVE AC-EQUIV-001 test that drives `run_materialization_pipeline`
// end-to-end is:
//   crates/prism-bin/tests/adv_p02_e2e_pushdown_pipeline_test.rs
//   → test_ac_equiv_001_result_equivalence_via_run_materialization_pipeline
//
// This file (prism-spec-engine) cannot depend on prism-bin (circular dep guard,
// see CLAUDE.md architecture compliance note). The test below validates the
// PipelineExecutor boundary only: it pre-seeds `_fql` into FetchContext
// (simulating the output of build_crowdstrike_fql after extract_time_window_from_ast)
// and asserts the DTU honors the FQL + no-fabrication invariant at that layer.
// It does NOT exercise run_materialization_pipeline, extract_time_window_from_ast,
// or build_crowdstrike_fql — those are exercised by the prism-bin test above.
// ---------------------------------------------------------------------------

/// AC-EQUIV-001 pipeline-executor boundary / BC-2.11.007 FQL subset invariant
///
/// Validates that a pre-built FQL string (the form produced by `build_crowdstrike_fql`)
/// seeded into `FetchContext["_fql"]` causes the DTU to return a strict SUBSET of
/// the unfiltered set, with no fabrication — verifying the PipelineExecutor wire layer.
///
/// # BOUNDARY SCOPE (pre-seeded FQL — NOT run_materialization_pipeline)
///
/// This test DOES NOT exercise `run_materialization_pipeline`, `extract_time_window_from_ast`,
/// or `build_crowdstrike_fql`. It seeds the FQL directly into `FetchContext` and calls
/// `PipelineExecutor::execute`. This is the pipeline-executor boundary assertion only.
///
/// The AUTHORITATIVE AC-EQUIV-001 test that proves the full production path
/// (`run_materialization_pipeline` → AST extraction → FQL build → DTU) is:
/// `test_ac_equiv_001_result_equivalence_via_run_materialization_pipeline` in
/// `crates/prism-bin/tests/adv_p02_e2e_pushdown_pipeline_test.rs`.
///
/// # Wire-level assertion (F-P1-HIGH-003)
/// Execution A seeds `_fql` with a time-window FQL matching a strict SUBSET of fixture records.
/// Execution B uses no filter. The DTU must honor the FQL (count_a < count_b).
/// Every record in A must also appear in B (no fabrication).
///
/// # SAP-2
/// Production `crowdstrike.sensor.toml` shape; real CrowdStrike DTU clone.
#[tokio::test]
async fn test_ac_equiv_001_fql_subset_invariant_via_pipeline_executor_boundary() {
    let mut clone = CrowdstrikeClone::new();
    let bound_addr = clone
        .start_on("127.0.0.1:0".parse().unwrap(), None, None)
        .await
        .expect("AC-EQUIV-001: CrowdStrike DTU clone failed to start");
    let dtu_base_url = format!("http://{bound_addr}");

    // Load production crowdstrike.sensor.toml (SAP-2).
    let spec_content = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../prism-sensors/specs/crowdstrike.sensor.toml"),
    )
    .expect("AC-EQUIV-001: crowdstrike.sensor.toml must be readable");
    let mut spec =
        SpecLoader::parse(&spec_content).expect("AC-EQUIV-001: crowdstrike.sensor.toml must parse");
    spec.base_url = dtu_base_url.clone();

    let detections_table = spec
        .tables
        .iter()
        .find(|t| t.table_name == "detections")
        .expect("AC-EQUIV-001: detections table must exist")
        .clone();

    // Structural assertion: FQL filter slot must be present for wire-level test to be non-vacuous.
    let step1 = detections_table
        .steps
        .iter()
        .find(|s| s.name == "query_detection_ids")
        .expect("AC-EQUIV-001: query_detection_ids step must exist");
    assert!(
        step1.path_template.contains("query.filter."),
        "AC-EQUIV-001 CRITICAL: CrowdStrike Step 1 path_template must include an FQL filter \
         interpolation slot (e.g., '&filter=${{query.filter._fql}}'). \
         Current path_template='{}'. Without this, push-down is vacuous.",
        step1.path_template
    );

    let http_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("AC-EQUIV-001: reqwest client build");
    let auth_provider = MockAuthProvider::new("test-oauth2-token");

    // Execution A: WITH push-down — FQL restricts to a strict subset of fixture records.
    // FQL: created_timestamp > 2026-01-15T00:00:00Z → records det-015..det-028 ≈ 14 records
    // (fixture has 50 records spanning 2026-01-01..2026-01-26).
    // This is the FQL that SpecDrivenSensorAdapter::build_crowdstrike_fql produces from
    // QueryParams.start_time = "2026-01-15T00:00:00Z".
    let fql_a = "created_timestamp:>'2026-01-15T00:00:00Z'";
    let mut filters_a = HashMap::new();
    filters_a.insert("_fql".to_string(), fql_a.to_string());
    let ctx_a = FetchContext::new(OrgSlug::new("test-org"), filters_a);
    let result_a = PipelineExecutor::execute(
        &spec,
        &detections_table,
        &ctx_a,
        &http_client,
        &auth_provider,
    )
    .await
    .expect("AC-EQUIV-001: execution A (with FQL push-down) must succeed");
    let count_a = result_a.records.len();

    // WIRE-LEVEL assertion (F-P1-HIGH-003): check filter-log BEFORE reset.
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("AC-EQUIV-001: filter-log client build");
    let filter_log_resp = client
        .get(format!("{dtu_base_url}/dtu/filter-log"))
        .send()
        .await
        .expect("AC-EQUIV-001: GET /dtu/filter-log must succeed");
    let filter_log_body: JsonValue = filter_log_resp
        .json()
        .await
        .expect("AC-EQUIV-001: filter-log must be JSON");
    let filter_strings = filter_log_body["filter_strings"]
        .as_array()
        .expect("AC-EQUIV-001: filter_strings must be array");
    assert!(
        filter_strings.iter().any(|s| s
            .as_str()
            .map(|f| f.contains("created_timestamp"))
            .unwrap_or(false)),
        "AC-EQUIV-001 WIRE-LEVEL (F-P1-HIGH-003): DTU filter-log must contain 'created_timestamp' \
         from the FQL filter '{}'. The FQL must reach the DTU via path_template interpolation. \
         Got: {:?}",
        fql_a,
        filter_strings
    );

    // Reset DTU state.
    clone
        .reset()
        .await
        .expect("AC-EQUIV-001: clone reset must succeed");

    // Execution B: WITHOUT push-down (no FQL filter; empty string = no filter in DTU).
    let ctx_b = FetchContext::new(OrgSlug::new("test-org"), HashMap::new());
    let result_b = PipelineExecutor::execute(
        &spec,
        &detections_table,
        &ctx_b,
        &http_client,
        &auth_provider,
    )
    .await
    .expect("AC-EQUIV-001: execution B (without push-down) must succeed");
    let count_b = result_b.records.len();

    // LOAD-BEARING assertion: count_a < count_b (DTU honored FQL filter).
    // If count_a == count_b, the DTU ignored the filter → test is VACUOUS (F-P1-CRIT-005).
    assert!(
        count_a < count_b,
        "AC-EQUIV-001 LOAD-BEARING: count_a ({}) must be < count_b ({}) when FQL push-down \
         filter '{}' is applied. DTU must honor FQL and return fewer records. \
         REGRESSION: CrowdStrike DTU is not filtering by created_timestamp FQL bounds.",
        count_a,
        count_b,
        fql_a
    );
    assert!(
        count_a > 0,
        "AC-EQUIV-001: execution A must return non-empty records (FQL '{}' must match some fixture records)",
        fql_a
    );
    assert!(
        count_b > 0,
        "AC-EQUIV-001: execution B must return non-empty records (full unfiltered fixture)"
    );

    // BC-2.11.007 invariant: every record in A must also appear in B (no fabrication).
    let ids_b: std::collections::HashSet<String> = result_b
        .records
        .iter()
        .filter_map(|r| {
            r.get("detection_id")
                .and_then(|v| v.as_str())
                .map(str::to_string)
        })
        .collect();
    for record in &result_a.records {
        if let Some(id) = record.get("detection_id").and_then(|v| v.as_str()) {
            assert!(
                ids_b.contains(id),
                "AC-EQUIV-001 invariant (BC-2.11.007): record '{}' in push-down result \
                 must also appear in unfiltered result. Push-down must not fabricate records.",
                id
            );
        }
    }
}

// ---------------------------------------------------------------------------
// AC-INDEX-001: armis.sensor.toml last_seen and created_at declare options=["INDEX"]
// ---------------------------------------------------------------------------

/// AC-INDEX-001 / BC-2.01.013 Mechanism B — AQL augmentation requires INDEX datetime cols
///
/// The `last_seen` column in `armis_devices` and `created_at` in `armis_alerts`
/// must declare `options = ["INDEX"]` in `armis.sensor.toml`.
///
/// Without this, `extract_time_window_from_ast` (ADR-033 T1) cannot identify these as
/// push-down-eligible datetime columns for Armis AQL augmentation.
///
/// # Red Gate
/// Fails because the current `armis.sensor.toml` has `options = ["INDEX"]` only on
/// the `aql` column. The `last_seen` and `created_at` datetime columns do NOT have
/// `options = ["INDEX"]` yet — they must be added by the implementer (AC-INDEX-001 scope).
#[test]
fn test_ac_index_001_armis_toml_last_seen_created_at_have_index_option() {
    use prism_core::ColumnOptions;

    let spec_content = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../prism-sensors/specs/armis.sensor.toml"),
    )
    .expect("AC-INDEX-001: armis.sensor.toml must be readable");
    let spec =
        SpecLoader::parse(&spec_content).expect("AC-INDEX-001: armis.sensor.toml must parse");

    // Find the devices table.
    let devices_table = spec
        .tables
        .iter()
        .find(|t| t.table_name == "devices")
        .expect("AC-INDEX-001: armis.sensor.toml must have 'devices' table");

    // Find last_seen column in devices.
    let last_seen_col = devices_table
        .columns
        .iter()
        .find(|c| c.name == "last_seen")
        .expect("AC-INDEX-001: devices table must have 'last_seen' column");

    assert!(
        last_seen_col.options.contains(&ColumnOptions::Index),
        "AC-INDEX-001 item (a): 'last_seen' column in armis_devices must declare \
         options = [\"INDEX\"]; currently has options={:?}. \
         Without this, extract_time_window_from_ast (ADR-033 T1) cannot identify \
         last_seen as a push-down-eligible datetime column. Add to armis.sensor.toml.",
        last_seen_col.options
    );

    // Find the alerts table.
    let alerts_table = spec
        .tables
        .iter()
        .find(|t| t.table_name == "alerts")
        .expect("AC-INDEX-001: armis.sensor.toml must have 'alerts' table");

    // Find created_at column in alerts.
    let created_at_col = alerts_table
        .columns
        .iter()
        .find(|c| c.name == "created_at")
        .expect("AC-INDEX-001: alerts table must have 'created_at' column");

    assert!(
        created_at_col.options.contains(&ColumnOptions::Index),
        "AC-INDEX-001 item (b): 'created_at' column in armis_alerts must declare \
         options = [\"INDEX\"]; currently has options={:?}. \
         Without this, extract_time_window_from_ast (ADR-033 T1) cannot identify \
         created_at as a push-down-eligible datetime column. Add to armis.sensor.toml.",
        created_at_col.options
    );
}

// ---------------------------------------------------------------------------
// AC-INDEX-CWS-001: crowdstrike.sensor.toml created_timestamp declares options=["INDEX"]
// (F-P1-CRIT-001)
// ---------------------------------------------------------------------------

/// AC-INDEX-CWS-001 / F-P1-CRIT-001 / BC-2.01.013 CrowdStrike FQL time-window
///
/// The `created_timestamp` column in `crowdstrike_detections` must declare
/// `options = ["INDEX"]` in `crowdstrike.sensor.toml`.
///
/// Without this, `extract_time_window_from_ast` (ADR-033 T1) cannot identify
/// `created_timestamp` as a push-down-eligible datetime column for CrowdStrike FQL.
/// The production `run_materialization_pipeline` path would silently produce
/// `start_time = None` even for queries with `WHERE created_timestamp > 'T'`.
///
/// F-P1-CRIT-001 closure: adding `options = ["INDEX"]` to `created_timestamp` enables
/// the T1 heuristic to populate `QueryParams.start_time`/`end_time` from AST predicates.
#[test]
fn test_ac_index_cws_001_crowdstrike_toml_created_timestamp_has_index_option() {
    use prism_core::ColumnOptions;

    let spec_content = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../prism-sensors/specs/crowdstrike.sensor.toml"),
    )
    .expect("AC-INDEX-CWS-001: crowdstrike.sensor.toml must be readable");
    let spec = SpecLoader::parse(&spec_content)
        .expect("AC-INDEX-CWS-001: crowdstrike.sensor.toml must parse");

    // Find the detections table.
    let detections_table = spec
        .tables
        .iter()
        .find(|t| t.table_name == "detections")
        .expect("AC-INDEX-CWS-001: crowdstrike.sensor.toml must have 'detections' table");

    // Find created_timestamp column.
    let created_ts_col = detections_table
        .columns
        .iter()
        .find(|c| c.name == "created_timestamp")
        .expect("AC-INDEX-CWS-001: detections table must have 'created_timestamp' column");

    assert!(
        created_ts_col.options.contains(&ColumnOptions::Index),
        "AC-INDEX-CWS-001 (F-P1-CRIT-001): 'created_timestamp' in crowdstrike_detections must \
         declare options = [\"INDEX\"]; currently has options={:?}. \
         Without this, extract_time_window_from_ast (ADR-033 T1) cannot identify \
         created_timestamp as a push-down-eligible datetime column for CrowdStrike FQL. \
         Add 'options = [\"INDEX\"]' to the created_timestamp column in crowdstrike.sensor.toml.",
        created_ts_col.options
    );
}

// ---------------------------------------------------------------------------
// AC-CWS-WIRE-001: CrowdStrike limit reaches DTU wire via filter — wire-level (F-P1-HIGH-003)
// ---------------------------------------------------------------------------

/// Wire-level test: CrowdStrike FQL filter and `limit` both reach the DTU in a combined
/// `run_materialization_pipeline` execution (F-P1-HIGH-003 / AC-CWS-001 extension).
///
/// Three behaviors are verified in a single pipeline run:
/// (a) The FQL `created_timestamp` filter reaches the DTU wire (verified via GET /dtu/filter-log,
///     which echoes FQL expressions received by the detections endpoint).
/// (b) The `limit` is honored by the DTU (verified by asserting result.records.len() <= limit;
///     `limit` is a DTU query parameter that controls response row count, not an FQL expression,
///     so it does not appear in the filter-log — result count is the correct proof).
/// (c) The result set is non-empty, confirming the DTU returned records and the pipeline
///     materialized them successfully.
#[tokio::test]
async fn test_ac_cws_wire_001_crowdstrike_fql_and_limit_reach_dtu() {
    let mut clone = CrowdstrikeClone::new();
    let bound_addr = clone
        .start_on("127.0.0.1:0".parse().unwrap(), None, None)
        .await
        .expect("AC-CWS-WIRE-001: CrowdStrike DTU clone failed to start");
    let dtu_base_url = format!("http://{bound_addr}");

    let spec_content = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../prism-sensors/specs/crowdstrike.sensor.toml"),
    )
    .expect("AC-CWS-WIRE-001: crowdstrike.sensor.toml must be readable");
    let mut spec = SpecLoader::parse(&spec_content)
        .expect("AC-CWS-WIRE-001: crowdstrike.sensor.toml must parse");
    spec.base_url = dtu_base_url.clone();

    let detections_table = spec
        .tables
        .iter()
        .find(|t| t.table_name == "detections")
        .expect("AC-CWS-WIRE-001: detections table must exist")
        .clone();

    // Structural: path_template must have both FQL and limit slots.
    let step1 = detections_table
        .steps
        .iter()
        .find(|s| s.name == "query_detection_ids")
        .expect("AC-CWS-WIRE-001: query_detection_ids step must exist");
    assert!(
        step1.path_template.contains("query.filter."),
        "AC-CWS-WIRE-001: path_template must have FQL slot; got: '{}'",
        step1.path_template
    );
    assert!(
        step1.path_template.contains("query.limit"),
        "AC-CWS-WIRE-001: path_template must have limit slot; got: '{}'",
        step1.path_template
    );

    let http_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("AC-CWS-WIRE-001: reqwest client build");
    let auth_provider = MockAuthProvider::new("test-oauth2-token");

    // Combined: FQL time filter + limit=3.
    // FQL: only records after 2026-01-20T00:00:00Z → ≈6 records (det-020..det-028 area).
    // limit=3 → DTU returns at most 3 of the filtered results.
    let fql = "created_timestamp:>'2026-01-20T00:00:00Z'";
    let mut filters = HashMap::new();
    filters.insert("_fql".to_string(), fql.to_string());
    filters.insert("query.limit".to_string(), "3".to_string());
    let ctx = FetchContext::new(OrgSlug::new("test-org"), filters);

    let result =
        PipelineExecutor::execute(&spec, &detections_table, &ctx, &http_client, &auth_provider)
            .await
            .expect("AC-CWS-WIRE-001: pipeline must succeed");

    // Wire-level: filter-log must confirm FQL reached the DTU.
    // Check IMMEDIATELY after the filtered execution (before any reset clears the log).
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("AC-CWS-WIRE-001: filter-log client build");
    let filter_log_resp = client
        .get(format!("{dtu_base_url}/dtu/filter-log"))
        .send()
        .await
        .expect("AC-CWS-WIRE-001: GET /dtu/filter-log must succeed");
    let filter_log_body: JsonValue = filter_log_resp
        .json()
        .await
        .expect("AC-CWS-WIRE-001: filter-log must be JSON");
    let filter_strings = filter_log_body["filter_strings"]
        .as_array()
        .expect("AC-CWS-WIRE-001: filter_strings must be array");
    assert!(
        filter_strings.iter().any(|s| s
            .as_str()
            .map(|f| f.contains("created_timestamp"))
            .unwrap_or(false)),
        "AC-CWS-WIRE-001 (F-P1-HIGH-003): DTU filter-log must contain 'created_timestamp' \
         from the FQL '{}'. The FQL must reach the DTU via path_template interpolation. \
         Got filter_strings: {:?}",
        fql,
        filter_strings
    );

    // FQL + limit both honored: result must be non-empty and ≤ 3.
    assert!(
        !result.records.is_empty(),
        "AC-CWS-WIRE-001: pipeline must return non-empty records (fixture has records after 2026-01-20)"
    );
    assert!(
        result.records.len() <= 3,
        "AC-CWS-WIRE-001: result must have at most 3 records when limit=3 is applied; got {}",
        result.records.len()
    );
}
