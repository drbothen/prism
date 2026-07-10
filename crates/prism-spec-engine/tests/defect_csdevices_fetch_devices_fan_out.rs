//! RED gate tests for DEFECT-CSDEVICES-EMPTY-PIPELINE-001 Sub-defect 1
//! (TOML spec — `fetch_devices` step missing fan-out anchor).
//!
//! # Defect summary
//!
//! `crowdstrike.sensor.toml` `fetch_devices` step uses `method = "GET"` with no
//! `body_template`. The spec-engine's `find_fan_out_array()` searches both
//! `path_template` and `body_template` for `${...}` variable references that
//! resolve to an array from a prior step. With no reference, it returns `None`,
//! so the engine issues GET /devices/entities/devices/v2 with ZERO `ids` params
//! — the DTU returns an empty resource list — 0 rows delivered.
//!
//! # Ratified fix (D-1650 architect ratification 2026-07-10)
//!
//! Convert `fetch_devices` step to:
//!   method = "POST"
//!   body_template = '{"ids": ${query_device_ids.resources}}'
//!
//! This gives `find_fan_out_array()` the array reference it needs.
//!
//! # Tests in this file
//!
//! ## Test 1: TOML structural assertion
//! `test_BC_DEFECT_CSDEVICES_001_fetch_devices_step_has_post_method_and_body_template`
//!   — Loads the committed `crowdstrike.sensor.toml`, finds the `devices` table,
//!     finds the `fetch_devices` step, and asserts:
//!       a) `step.method == "POST"`
//!       b) `step.body_template.is_some()`
//!       c) the body_template contains a `${...}` variable reference
//!     RED: both (a) and (b) fail. `method == "GET"`, `body_template == None`.
//!
//! ## Test 2: Pipeline execution assertion
//! `test_BC_DEFECT_CSDEVICES_001_devices_pipeline_returns_records_via_post`
//!   — Drives `PipelineExecutor::execute` against an in-process wiremock server
//!     mocking GET /devices/queries/devices/v1 (step 1) and
//!     POST /devices/entities/devices/v2 (step 2). Asserts `result.records.len() >= 3`.
//!     RED: the engine hits GET (not POST) for step 2 → wiremock mock expectation for
//!     the POST path is unmet → the MockServer drops with a panic, OR the
//!     engine fails to deliver records (0 returned from GET with no ids).
//!     The test asserts non-empty records — it FAILS in the current GET state.
//!
//! # BC anchors
//!
//! - BC-2.16.002 §Fan-out precondition (find_fan_out_array must return Some for
//!   `fetch_devices` after fix)
//! - D-1650 §Contract Part 1 (TOML change spec)
//! - ADR-028 §D1 (TOML spec grounds against DTU clone routes)
//!
//! # Red Gate (BC-5.38.001)
//!
//! Both tests must FAIL before the TOML change lands.
//! Test 1 fails on the method/body_template assertions.
//! Test 2 fails because 0 records are returned (GET step 2 with no ids yields empty).

#![allow(non_snake_case, clippy::expect_used, clippy::unwrap_used)]

use std::collections::HashMap;

use prism_core::OrgSlug;
use prism_spec_engine::{
    NullAuthProvider,
    pipeline::{FetchContext, PipelineExecutor},
    spec_parser::SpecLoader,
};

// ---------------------------------------------------------------------------
// Helper: load the bundled crowdstrike.sensor.toml
// ---------------------------------------------------------------------------

fn load_crowdstrike_spec() -> prism_spec_engine::spec_parser::SensorSpec {
    let spec_content = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../prism-sensors/specs/crowdstrike.sensor.toml"),
    )
    .expect("crowdstrike.sensor.toml must be readable from prism-spec-engine tests");

    SpecLoader::parse(&spec_content).expect("crowdstrike.sensor.toml must parse without error")
}

// ---------------------------------------------------------------------------
// Test 1: Structural assertion — fetch_devices step must use POST + body_template
//
// RED: `method == "GET"` (not POST) and `body_template == None`
//      → assertions (a) and (b) both fail immediately.
//
// PASSES after Contract Part 1 lands (method = "POST", body_template = '{"ids": ...}').
// ---------------------------------------------------------------------------

/// BC-DEFECT-CSDEVICES-001: The `fetch_devices` step in `crowdstrike.sensor.toml`
/// must use `method = "POST"` and declare a `body_template` containing a
/// `${query_device_ids.resources}` variable reference.
///
/// Without both of these, `find_fan_out_array()` returns `None` and the engine
/// issues a bare GET with no IDs, delivering 0 rows (the confirmed root cause).
///
/// RED: `method == "GET"`, `body_template == None` — assertions fail.
/// PASSES after the TOML `fetch_devices` step is converted to POST + body_template.
#[test]
fn test_BC_DEFECT_CSDEVICES_001_fetch_devices_step_has_post_method_and_body_template() {
    let spec = load_crowdstrike_spec();

    // Locate the `devices` table.
    let devices_table = spec
        .tables
        .iter()
        .find(|t| t.table_name == "devices")
        .expect("crowdstrike.sensor.toml must declare a 'devices' table");

    // Locate the `fetch_devices` step (Step 2 of the two-step pipeline).
    let fetch_devices_step = devices_table
        .steps
        .iter()
        .find(|s| s.name == "fetch_devices")
        .expect(
            "devices table must have a step named 'fetch_devices' \
             (Step 2: GET/POST /devices/entities/devices/v2)",
        );

    // (a) Method must be POST (not GET).
    //
    // RED: currently "GET" → assertion fails with:
    //   "fetch_devices step method must be POST; got: GET"
    assert_eq!(
        fetch_devices_step.method.to_uppercase(),
        "POST",
        "BC-DEFECT-CSDEVICES-001: fetch_devices step method must be POST; got: {}. \
         RED: currently GET — triggers find_fan_out_array returning None → 0 rows.",
        fetch_devices_step.method
    );

    // (b) body_template must be Some.
    //
    // RED: currently None → assertion fails with:
    //   "fetch_devices step must have a body_template"
    assert!(
        fetch_devices_step.body_template.is_some(),
        "BC-DEFECT-CSDEVICES-001: fetch_devices step must have a body_template; \
         RED: currently None — find_fan_out_array returns None when no body_template present."
    );

    // (c) body_template must contain a ${...} variable reference so
    //     find_fan_out_array() can locate the fan-out anchor.
    //
    // The ratified body_template is: '{"ids": ${query_device_ids.resources}}'
    let bt = fetch_devices_step.body_template.as_deref().unwrap_or("");
    assert!(
        bt.contains("${"),
        "BC-DEFECT-CSDEVICES-001: body_template must contain a '${{...}}' variable reference \
         (fan-out anchor for find_fan_out_array); got: {bt:?}. \
         Expected form: '{{\"ids\": ${{query_device_ids.resources}}}}'."
    );
}

// ---------------------------------------------------------------------------
// Test 2: Pipeline execution — PipelineExecutor returns records via POST step 2
//
// Uses wiremock in-process HTTP server to mock both pipeline steps.
// Mocks:
//   - GET /devices/queries/devices/v1 → {"resources": ["id-001", "id-002", "id-003"]}
//   - POST /devices/entities/devices/v2 → {"resources": [{...}, {...}, {...}]}
//     (expect = 1 ensures the POST is actually called by the engine)
//
// RED: The engine currently uses GET for step 2 with no IDs:
//   - The POST mock is never called → wiremock panics on MockServer drop (expect=1)
//   - Additionally, result.records.len() == 0 (GET with no ids → empty DTU response
//     if GET mock not also present, or empty mocked GET response if present)
//   - The test assertion `records.len() >= 3` fails.
//
// PASSES after:
//   1. TOML step converts to POST + body_template (Contract Part 1)
//   2. DTU gains post_host_details handler (Contract Part 2)
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// Test 3: Structural assertion — fetch_incidents step must use POST + body_template
//
// F-CSD-P1-005: mirrors the fetch_devices shape lock for the incidents table.
// RED (before TOML fix): `method == "GET"`, `body_template == None`
// GREEN (after TOML fix): `method == "POST"`, body_template contains `${`
// ---------------------------------------------------------------------------

/// BC-DEFECT-CSDEVICES-001: The `fetch_incidents` step in `crowdstrike.sensor.toml`
/// must use `method = "POST"` and declare a `body_template` containing a
/// `${query_incident_ids.resources}` variable reference.
///
/// Without both of these, `find_fan_out_array()` returns `None` and the engine
/// issues a bare GET/POST with no IDs body, delivering 0 rows (same root cause
/// as the `fetch_devices` defect, applied to the incidents two-step pipeline).
///
/// RED: `method == "GET"`, `body_template == None` — assertions fail.
/// GREEN after the TOML `fetch_incidents` step is converted to POST + body_template
/// per DEFECT-CSDEVICES-EMPTY-PIPELINE-001 ratification (F-CSD-P1-005).
#[test]
fn test_BC_DEFECT_CSDEVICES_001_fetch_incidents_step_has_post_method_and_body_template() {
    let spec = load_crowdstrike_spec();

    // Locate the `incidents` table.
    let incidents_table = spec
        .tables
        .iter()
        .find(|t| t.table_name == "incidents")
        .expect("crowdstrike.sensor.toml must declare an 'incidents' table");

    // Locate the `fetch_incidents` step (Step 2 of the two-step pipeline).
    let fetch_incidents_step = incidents_table
        .steps
        .iter()
        .find(|s| s.name == "fetch_incidents")
        .expect(
            "incidents table must have a step named 'fetch_incidents' \
             (Step 2: POST /incidents/entities/incidents/GET/v1)",
        );

    // (a) Method must be POST (not GET).
    assert_eq!(
        fetch_incidents_step.method.to_uppercase(),
        "POST",
        "F-CSD-P1-005: fetch_incidents step method must be POST; got: {}. \
         Without POST, find_fan_out_array returns None → 0 incident rows.",
        fetch_incidents_step.method
    );

    // (b) body_template must be Some.
    assert!(
        fetch_incidents_step.body_template.is_some(),
        "F-CSD-P1-005: fetch_incidents step must have a body_template; \
         currently None — find_fan_out_array returns None when no body_template present."
    );

    // (c) body_template must contain a ${...} variable reference so
    //     find_fan_out_array() can locate the fan-out anchor.
    //
    // The ratified body_template is: '{"ids": ${query_incident_ids.resources}}'
    let bt = fetch_incidents_step.body_template.as_deref().unwrap_or("");
    assert!(
        bt.contains("${"),
        "F-CSD-P1-005: body_template must contain a '${{...}}' variable reference \
         (fan-out anchor for find_fan_out_array); got: {bt:?}. \
         Expected form: '{{\"ids\": ${{query_incident_ids.resources}}}}'."
    );

    // (d) The variable reference must point to query_incident_ids step results
    //     (backward ref to step 1 of the incidents pipeline).
    assert!(
        bt.contains("query_incident_ids"),
        "F-CSD-P1-005: body_template must reference 'query_incident_ids' (step 1 results); \
         got: {bt:?}. The fan-out requires a backward ref to the IDs collected in step 1."
    );
}

/// BC-DEFECT-CSDEVICES-001: `PipelineExecutor::execute` for the `devices` table
/// must issue a POST to `/devices/entities/devices/v2` (not GET) in step 2,
/// forwarding IDs from step 1, and return the device records.
///
/// Exercises the full two-step fan-out path via in-process wiremock.
/// Mirrors the `test_BC_2_16_002_pipeline_executor_runs_crowdstrike_two_step_spec`
/// pattern from `bc_2_16_002_crowdstrike_two_step.rs`.
///
/// RED: currently the engine issues GET (not POST) for step 2.
///   The wiremock `.expect(1)` on POST fails → MockServer panic on drop.
///   Additionally result.records.len() == 0 (0 rows delivered via bare GET).
/// PASSES after TOML fix (Contract Part 1) + DTU route fix (Contract Part 2).
#[tokio::test]
async fn test_BC_DEFECT_CSDEVICES_001_devices_pipeline_returns_records_via_post() {
    use wiremock::{
        Mock, MockServer, ResponseTemplate,
        matchers::{method, path},
    };

    let mock_server = MockServer::start().await;

    // Step 1: GET /devices/queries/devices/v1 → 3 device IDs.
    Mock::given(method("GET"))
        .and(path("/devices/queries/devices/v1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "resources": ["id-001", "id-002", "id-003"]
        })))
        .expect(1)
        .mount(&mock_server)
        .await;

    // Step 2: POST /devices/entities/devices/v2 → 3 device records.
    // `expect(1)` ensures this mock is called exactly once.
    // If the engine issues GET instead of POST, this expectation is unmet →
    // MockServer panics at drop — causing the test to fail.
    Mock::given(method("POST"))
        .and(path("/devices/entities/devices/v2"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "resources": [
                {"device_id": "id-001", "hostname": "host-001.example.com"},
                {"device_id": "id-002", "hostname": "host-002.example.com"},
                {"device_id": "id-003", "hostname": "host-003.example.com"}
            ]
        })))
        .expect(1)
        .mount(&mock_server)
        .await;

    // Load the real TOML spec.
    let mut spec = load_crowdstrike_spec();
    spec.base_url = mock_server.uri();

    // Find the `devices` table.
    let devices_table = spec
        .tables
        .iter()
        .find(|t| t.table_name == "devices")
        .expect("crowdstrike spec must declare a 'devices' table");

    let context = FetchContext::new(OrgSlug::new("test-org"), HashMap::new());
    let http_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .expect("reqwest client must build");
    let auth_provider = NullAuthProvider;

    let result =
        PipelineExecutor::execute(&spec, devices_table, &context, &http_client, &auth_provider)
            .await
            .expect(
                "PipelineExecutor::execute must succeed for crowdstrike devices two-step pipeline",
            );

    // Assert >= 3 records were delivered (one per mocked device).
    //
    // RED: currently result.records.len() == 0 because the GET step 2 with no IDs
    //      delivers an empty resource list from the mock server (GET mock not present →
    //      wiremock 404 or unmatched → pipeline may propagate partial-failure or
    //      return 0 records). Either way, this assertion fails.
    assert!(
        result.records.len() >= 3,
        "BC-DEFECT-CSDEVICES-001: devices pipeline must deliver >= 3 records \
         (one per mocked device); got {}. \
         RED: currently 0 — step 2 is GET with no IDs instead of POST with fan-out.",
        result.records.len()
    );
}
