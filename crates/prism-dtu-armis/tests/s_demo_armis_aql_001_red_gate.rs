#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Red Gate tests for S-DEMO-ARMIS-AQL-001 — Armis AQL Search Endpoint Fidelity.
//!
//! # Red Gate discipline (BC-5.38.001)
//!
//! ALL tests in this file MUST FAIL before `get_search` is implemented.
//! The stub body in `routes/search.rs` is `todo!()`, which panics at runtime.
//! Tests will fail with a connection-reset or 500 from the panic — that is the
//! correct Red Gate failure mode.
//!
//! # AC traceability
//!
//! | Test | AC | BC clause |
//! |------|----|-----------|
//! | test_armis_aql_search_route_registered_returns_200_for_device_aql | AC-001 | BC-2.16.013 §Postconditions §1 DTU-Parity |
//! | test_armis_aql_search_returns_403_without_bearer | AC-001 EC-004 | BC-2.16.013 §Postconditions §1 DTU-Parity |
//! | test_armis_aql_search_devices_aql_returns_device_records | AC-002 | BC-2.16.013 §Postconditions §2 fixture-parity |
//! | test_armis_aql_search_alerts_aql_returns_alert_records | AC-003 (handler-unit) | BC-2.16.013 §Postconditions §2 handler routing only — pipeline-boundary AC-003 end-to-end claim carried by parity/armis.rs::test_BC_2_16_013_AC_005_aql_roundtrip_alerts_pipeline (F-P1-CRIT-002) |
//! | test_armis_aql_search_aql_captured_in_aql_log | AC-002 | BC-2.16.013 R-DTU-002 AQL capture |
//! | test_armis_aql_search_no_aql_defaults_to_devices | AC-001 EC-001 | BC-2.16.013 §Postconditions §1 safe fallback |
//! | test_armis_aql_search_toml_path_template_updated | AC-004 | BC-2.16.013 §Postconditions §2 DTU-TOML-column-parity |
//! | test_armis_aql_search_toml_response_path_updated | AC-004 | BC-2.16.013 §Postconditions §2 DTU-TOML-column-parity |
//! | test_armis_aql_search_dtu_toml_column_parity | AC-004 SAP-2 | BC-2.16.013 DTU↔TOML parity |
//!
//! # SAP-2 awareness
//!
//! `test_armis_aql_search_dtu_toml_column_parity` asserts that every column declared
//! in `armis.sensor.toml` under the `devices` and `alerts` tables maps to a field in
//! `types.rs::DeviceRecord` / `AlertRecord` respectively — per SAP-2 (DTU↔TOML parity
//! standing probe). This is a static (non-network) assertion that can run without the
//! DTU server.

#![cfg(feature = "dtu")]

use prism_dtu_armis::ArmisClone;
use prism_dtu_common::BehavioralClone;

// ---------------------------------------------------------------------------
// AC-001: GET /api/v1/search registered, returns 200 with valid Bearer
// (BC-2.16.013 §Postconditions §1 DTU-Parity; ADR-031 §D8-a)
// ---------------------------------------------------------------------------

/// AC-001 / BC-2.16.013 §Postconditions §1:
/// GET /api/v1/search?aql=in:type=Device with a valid Bearer token returns HTTP 200.
///
/// Red Gate failure: `get_search` stub is `todo!()` — the handler panics and axum
/// returns a 500 (or the connection is reset). The test asserts 200 → fails RED.
#[tokio::test]
async fn test_armis_aql_search_route_registered_returns_200_for_device_aql() {
    let mut clone =
        ArmisClone::new().expect("S-DEMO-ARMIS-AQL-001 AC-001: ArmisClone::new() must succeed");
    clone
        .start()
        .await
        .expect("S-DEMO-ARMIS-AQL-001 AC-001: start() must succeed");
    let base_url = clone.base_url();
    let client = reqwest::Client::new();

    let resp = client
        .get(format!("{base_url}/api/v1/search"))
        .query(&[("aql", "in:type=Device")])
        .header("Authorization", "Bearer test-token")
        .send()
        .await
        .expect("S-DEMO-ARMIS-AQL-001 AC-001: GET /api/v1/search request must not fail at transport layer");

    assert_eq!(
        resp.status().as_u16(),
        200,
        "S-DEMO-ARMIS-AQL-001 AC-001: GET /api/v1/search?aql=in:type=Device with valid Bearer must return HTTP 200 (BC-2.16.013 §Postconditions §1 DTU-Parity)"
    );
}

/// AC-001 EC-004 / BC-2.16.013 §Postconditions §1:
/// GET /api/v1/search without Authorization header returns HTTP 403.
///
/// This asserts the same auth model as existing Armis endpoints (403 not 401).
/// Red Gate failure: `get_search` stub is `todo!()` — panics before auth check,
/// returning 500 or connection-reset rather than 403.
#[tokio::test]
async fn test_armis_aql_search_returns_403_without_bearer() {
    let mut clone = ArmisClone::new()
        .expect("S-DEMO-ARMIS-AQL-001 AC-001/EC-004: ArmisClone::new() must succeed");
    clone
        .start()
        .await
        .expect("S-DEMO-ARMIS-AQL-001 AC-001/EC-004: start() must succeed");
    let base_url = clone.base_url();
    let client = reqwest::Client::new();

    // No Authorization header.
    let resp = client
        .get(format!("{base_url}/api/v1/search"))
        .query(&[("aql", "in:type=Device")])
        .send()
        .await
        .expect("S-DEMO-ARMIS-AQL-001 AC-001/EC-004: GET /api/v1/search without Bearer must not fail at transport layer");

    assert_eq!(
        resp.status().as_u16(),
        403,
        "S-DEMO-ARMIS-AQL-001 AC-001/EC-004: GET /api/v1/search without Authorization header must return HTTP 403 (matches Armis auth model — check_bearer_auth returns 403 not 401)"
    );
}

// ---------------------------------------------------------------------------
// AC-002: /api/v1/search with in:type=Device AQL returns device records
// (BC-2.16.013 §Postconditions §2 fixture-parity; ADR-031 §D8-a requirement 1)
// ---------------------------------------------------------------------------

/// AC-002 / BC-2.16.013 §Postconditions §2 fixture-parity:
/// GET /api/v1/search?aql=in:type=Device returns a JSON body with the shape
/// `{"data": {"results": [...], "total": N}}` where `results` is a non-empty array
/// of DeviceRecord-shaped objects.
///
/// Red Gate failure: `get_search` is `todo!()` — panics, axum returns 500.
/// The test asserts 200 + correct envelope → fails RED.
#[tokio::test]
async fn test_armis_aql_search_devices_aql_returns_device_records() {
    let mut clone =
        ArmisClone::new().expect("S-DEMO-ARMIS-AQL-001 AC-002: ArmisClone::new() must succeed");
    clone
        .start()
        .await
        .expect("S-DEMO-ARMIS-AQL-001 AC-002: start() must succeed");
    let base_url = clone.base_url();
    let client = reqwest::Client::new();

    let resp = client
        .get(format!("{base_url}/api/v1/search"))
        .query(&[("aql", "in:type=Device")])
        .header("Authorization", "Bearer test-token")
        .send()
        .await
        .expect("S-DEMO-ARMIS-AQL-001 AC-002: GET /api/v1/search?aql=in:type=Device must not fail at transport layer");

    assert_eq!(
        resp.status().as_u16(),
        200,
        "S-DEMO-ARMIS-AQL-001 AC-002: search with Device AQL must return HTTP 200"
    );

    let body: serde_json::Value = resp
        .json()
        .await
        .expect("S-DEMO-ARMIS-AQL-001 AC-002: response body must be valid JSON");

    // Assert the search envelope shape: {"data": {"results": [...], "total": N}}
    // (not {"data": {"devices": ...}} — that is the direct-endpoint shape).
    // Reference: routes/search.rs SearchResponse { data: SearchData { results, total } }.
    assert!(
        body["data"]["results"].is_array(),
        "S-DEMO-ARMIS-AQL-001 AC-002: GET /api/v1/search must return data.results array (real Armis search envelope, not data.devices); got: {body}"
    );

    let results = body["data"]["results"]
        .as_array()
        .expect("S-DEMO-ARMIS-AQL-001 AC-002: data.results must be an array");

    assert!(
        !results.is_empty(),
        "S-DEMO-ARMIS-AQL-001 AC-002: data.results must be non-empty for in:type=Device AQL against fixture data"
    );

    // Assert total field is present and > 0.
    let total = body["data"]["total"]
        .as_u64()
        .expect("S-DEMO-ARMIS-AQL-001 AC-002: data.total must be a number");
    assert!(
        total > 0,
        "S-DEMO-ARMIS-AQL-001 AC-002: data.total must be > 0 for Device AQL"
    );

    // Assert first result contains device_id — a DeviceRecord field.
    // This verifies results contain DeviceRecord-shaped objects, not AlertRecords.
    let first = &results[0];
    assert!(
        first["device_id"].is_string(),
        "S-DEMO-ARMIS-AQL-001 AC-002: first result must contain device_id string (DeviceRecord shape); got: {first}"
    );
}

/// AC-002 / BC-2.16.013 R-DTU-002 AQL capture:
/// After GET /api/v1/search?aql=in:type=Device, the AQL string "in:type=Device"
/// is visible in GET /dtu/aql-log as aql_strings[0].
///
/// Red Gate failure: `get_search` is `todo!()` — panics without calling
/// `state.capture_aql()`. AQL log remains empty → assertion fails RED.
#[tokio::test]
async fn test_armis_aql_search_aql_captured_in_aql_log() {
    let mut clone = ArmisClone::new()
        .expect("S-DEMO-ARMIS-AQL-001 AC-002 AQL-capture: ArmisClone::new() must succeed");
    clone
        .start()
        .await
        .expect("S-DEMO-ARMIS-AQL-001 AC-002 AQL-capture: start() must succeed");
    let base_url = clone.base_url();
    let client = reqwest::Client::new();

    // Issue search request with a specific AQL string.
    let _resp = client
        .get(format!("{base_url}/api/v1/search"))
        .query(&[("aql", "in:type=Device")])
        .header("Authorization", "Bearer test-token")
        .send()
        .await
        .expect("S-DEMO-ARMIS-AQL-001 AC-002 AQL-capture: GET /api/v1/search must not fail at transport layer");

    // Query the AQL log.
    let aql_resp = client
        .get(format!("{base_url}/dtu/aql-log"))
        .send()
        .await
        .expect("S-DEMO-ARMIS-AQL-001 AC-002 AQL-capture: GET /dtu/aql-log must succeed");

    assert_eq!(
        aql_resp.status().as_u16(),
        200,
        "S-DEMO-ARMIS-AQL-001 AC-002 AQL-capture: GET /dtu/aql-log must return 200"
    );

    let aql_body: serde_json::Value = aql_resp
        .json()
        .await
        .expect("S-DEMO-ARMIS-AQL-001 AC-002 AQL-capture: aql-log response must be valid JSON");

    let aql_strings = aql_body["aql_strings"]
        .as_array()
        .expect("S-DEMO-ARMIS-AQL-001 AC-002 AQL-capture: aql_strings must be an array");

    assert!(
        aql_strings
            .iter()
            .any(|s| s.as_str() == Some("in:type=Device")),
        "S-DEMO-ARMIS-AQL-001 AC-002 AQL-capture: aql_strings must contain 'in:type=Device' after search request (R-DTU-002: state.capture_aql() called by handler); got: {aql_strings:?}"
    );
}

// ---------------------------------------------------------------------------
// AC-003: /api/v1/search with alert AQL returns alert records
// (BC-2.16.013 §Postconditions §2 fixture-parity; ADR-031 §D8-a requirement 1)
// ---------------------------------------------------------------------------

/// AC-003 (handler-unit) / BC-2.16.013 §Postconditions §2 fixture-parity:
/// GET /api/v1/search?aql=in:type=Alert returns HTTP 200 with envelope
/// `{"data": {"results": [...AlertRecord objects...], "total": N}}`.
///
/// Scope: this test exercises the DTU handler directly (AQL sent as a raw query
/// param to the DTU, no PipelineExecutor involved). It proves the handler itself
/// routes Alert AQL to alert records. The end-to-end pipeline-boundary claim
/// (AQL forwarding through PipelineExecutor + armis.sensor.toml) is carried by
/// test_BC_2_16_013_AC_005_aql_roundtrip_alerts_pipeline in parity/armis.rs
/// (F-P1-CRIT-002 / AC-005).
///
/// Red Gate failure: `get_search` is `todo!()` — panics, axum returns 500.
/// The test asserts 200 + AlertRecord-shaped results → fails RED.
#[tokio::test]
async fn test_armis_aql_search_alerts_aql_returns_alert_records() {
    let mut clone =
        ArmisClone::new().expect("S-DEMO-ARMIS-AQL-001 AC-003: ArmisClone::new() must succeed");
    clone
        .start()
        .await
        .expect("S-DEMO-ARMIS-AQL-001 AC-003: start() must succeed");
    let base_url = clone.base_url();
    let client = reqwest::Client::new();

    let resp = client
        .get(format!("{base_url}/api/v1/search"))
        .query(&[("aql", "in:type=Alert")])
        .header("Authorization", "Bearer test-token")
        .send()
        .await
        .expect("S-DEMO-ARMIS-AQL-001 AC-003: GET /api/v1/search?aql=in:type=Alert must not fail at transport layer");

    assert_eq!(
        resp.status().as_u16(),
        200,
        "S-DEMO-ARMIS-AQL-001 AC-003: search with Alert AQL must return HTTP 200"
    );

    let body: serde_json::Value = resp
        .json()
        .await
        .expect("S-DEMO-ARMIS-AQL-001 AC-003: response body must be valid JSON");

    // Assert search envelope shape.
    assert!(
        body["data"]["results"].is_array(),
        "S-DEMO-ARMIS-AQL-001 AC-003: GET /api/v1/search with Alert AQL must return data.results array; got: {body}"
    );

    let results = body["data"]["results"]
        .as_array()
        .expect("S-DEMO-ARMIS-AQL-001 AC-003: data.results must be an array");

    assert!(
        !results.is_empty(),
        "S-DEMO-ARMIS-AQL-001 AC-003: data.results must be non-empty for in:type=Alert AQL against fixture data"
    );

    // Assert total is present.
    let total = body["data"]["total"]
        .as_u64()
        .expect("S-DEMO-ARMIS-AQL-001 AC-003: data.total must be a number");
    assert!(
        total > 0,
        "S-DEMO-ARMIS-AQL-001 AC-003: data.total must be > 0 for Alert AQL"
    );

    // Assert first result contains alert_id — an AlertRecord field.
    // This verifies the handler routed to alerts (not devices) for Alert AQL.
    let first = &results[0];
    assert!(
        first["alert_id"].is_string(),
        "S-DEMO-ARMIS-AQL-001 AC-003: first result must contain alert_id string (AlertRecord shape — handler must route Alert AQL to alert fixture); got: {first}"
    );
}

// ---------------------------------------------------------------------------
// EC-001: absent aql param defaults to devices
// (AC-001 safe fallback; BC-2.16.013 §Postconditions §1)
// ---------------------------------------------------------------------------

/// AC-001 EC-001 / BC-2.16.013 §Postconditions §1 safe fallback:
/// GET /api/v1/search without `aql` parameter returns HTTP 200 with device results
/// (safe default — EC-001 from story spec).
///
/// Red Gate failure: `get_search` is `todo!()` — panics, returning 500.
#[tokio::test]
async fn test_armis_aql_search_no_aql_defaults_to_devices() {
    let mut clone = ArmisClone::new()
        .expect("S-DEMO-ARMIS-AQL-001 AC-001/EC-001: ArmisClone::new() must succeed");
    clone
        .start()
        .await
        .expect("S-DEMO-ARMIS-AQL-001 AC-001/EC-001: start() must succeed");
    let base_url = clone.base_url();
    let client = reqwest::Client::new();

    // No aql param at all.
    let resp = client
        .get(format!("{base_url}/api/v1/search"))
        .header("Authorization", "Bearer test-token")
        .send()
        .await
        .expect("S-DEMO-ARMIS-AQL-001 EC-001: GET /api/v1/search with no aql must not fail at transport layer");

    assert_eq!(
        resp.status().as_u16(),
        200,
        "S-DEMO-ARMIS-AQL-001 EC-001: absent aql must return 200 (safe fallback to devices)"
    );

    let body: serde_json::Value = resp
        .json()
        .await
        .expect("S-DEMO-ARMIS-AQL-001 EC-001: response body must be valid JSON");

    // Default fallback returns device results in data.results envelope.
    assert!(
        body["data"]["results"].is_array(),
        "S-DEMO-ARMIS-AQL-001 EC-001: absent aql must return data.results array (device records by default); got: {body}"
    );

    let results = body["data"]["results"]
        .as_array()
        .expect("S-DEMO-ARMIS-AQL-001 EC-001: data.results must be array");

    // Default to devices: first result should have device_id, not alert_id.
    if !results.is_empty() {
        let first = &results[0];
        assert!(
            first["device_id"].is_string(),
            "S-DEMO-ARMIS-AQL-001 EC-001: default (no aql) must return device records (device_id field present); got: {first}"
        );
    }
}

// ---------------------------------------------------------------------------
// AC-004: armis.sensor.toml devices and alerts steps use /api/v1/search path
// These are static TOML-content tests — no network I/O, no todo!() panic path.
// They test the TOML content directly by parsing the file.
//
// NOTE: These tests DO pass today because the stub-architect already updated the
// TOML file (path_template = "/api/v1/search", response_path = "$.data.results").
// They are included per spec to cover AC-004 and as SAP-2 parity checks, and to
// ensure the TOML content is not accidentally reverted during implementation.
// The story spec lists them in the Red Gate test table — they are present here
// for AC traceability completeness.
// ---------------------------------------------------------------------------

/// AC-004 / BC-2.16.013 §Postconditions §2 DTU-TOML-column-parity:
/// armis.sensor.toml `devices` table `fetch_devices` step has
/// `path_template = "/api/v1/search"` (not the legacy `/api/v1/devices`).
#[test]
fn test_armis_aql_search_toml_path_template_updated() {
    let toml_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../prism-sensors/specs/armis.sensor.toml");

    let content = std::fs::read_to_string(&toml_path).expect(
        "S-DEMO-ARMIS-AQL-001 AC-004: armis.sensor.toml must exist at crates/prism-sensors/specs/",
    );

    // Parse as TOML to get structured access.
    let doc: toml::Value = toml::from_str(&content)
        .expect("S-DEMO-ARMIS-AQL-001 AC-004: armis.sensor.toml must be valid TOML");

    let tables = doc["tables"]
        .as_array()
        .expect("S-DEMO-ARMIS-AQL-001 AC-004: armis.sensor.toml must have [[tables]] array");

    // Find the devices table.
    let devices_table = tables
        .iter()
        .find(|t| t["table_name"].as_str() == Some("devices"))
        .expect("S-DEMO-ARMIS-AQL-001 AC-004: armis.sensor.toml must have a 'devices' table");

    let fetch_devices_step = devices_table["steps"]
        .as_array()
        .expect("S-DEMO-ARMIS-AQL-001 AC-004: devices table must have [[tables.steps]]")
        .iter()
        .find(|s| s["name"].as_str() == Some("fetch_devices"))
        .expect("S-DEMO-ARMIS-AQL-001 AC-004: devices table must have a 'fetch_devices' step");

    let path_template = fetch_devices_step["path_template"]
        .as_str()
        .expect("S-DEMO-ARMIS-AQL-001 AC-004: fetch_devices step must have path_template");

    // AC-004 intent: path_template must use the AQL search endpoint with the ?aql= push-down
    // parameter (not the legacy /api/v1/devices). The full expected value is
    // "/api/v1/search?aql=${query.filter.aql}". We assert starts_with("/api/v1/search?aql=")
    // to confirm both the correct path AND the presence of the AQL query parameter — a weaker
    // assertion (starts_with("/api/v1/search") only) would accept regressions where ?aql= is dropped.
    assert!(
        path_template.starts_with("/api/v1/search?aql="),
        "S-DEMO-ARMIS-AQL-001 AC-004: armis.sensor.toml devices.fetch_devices.path_template must \
         start with '/api/v1/search?aql=' (AQL push-down required; DTU-EXT-003 closed by \
         S-DEMO-ARMIS-AQL-001; BC-2.16.013 §Postconditions §2); got: {path_template}"
    );

    // Also assert alerts table path_template for completeness (AC-004 covers both).
    let alerts_table = tables
        .iter()
        .find(|t| t["table_name"].as_str() == Some("alerts"))
        .expect("S-DEMO-ARMIS-AQL-001 AC-004: armis.sensor.toml must have an 'alerts' table");

    let fetch_alerts_step = alerts_table["steps"]
        .as_array()
        .expect("S-DEMO-ARMIS-AQL-001 AC-004: alerts table must have [[tables.steps]]")
        .iter()
        .find(|s| s["name"].as_str() == Some("fetch_alerts"))
        .expect("S-DEMO-ARMIS-AQL-001 AC-004: alerts table must have a 'fetch_alerts' step");

    let alerts_path_template = fetch_alerts_step["path_template"]
        .as_str()
        .expect("S-DEMO-ARMIS-AQL-001 AC-004: fetch_alerts step must have path_template");

    assert!(
        alerts_path_template.starts_with("/api/v1/search?aql="),
        "S-DEMO-ARMIS-AQL-001 AC-004: armis.sensor.toml alerts.fetch_alerts.path_template must \
         start with '/api/v1/search?aql=' (AQL push-down required; DTU-EXT-004 closed by \
         S-DEMO-ARMIS-AQL-001; BC-2.16.013 §Postconditions §2); got: {alerts_path_template}"
    );
}

/// AC-004 / BC-2.16.013 §Postconditions §2 DTU-TOML-column-parity:
/// armis.sensor.toml `devices` and `alerts` table steps have
/// `response_path = "$.data.results"` (not legacy `$.data.devices` or `$.data.alerts`).
///
/// The search envelope uses `results` (not `devices`/`alerts`) per the real Armis API
/// (poller-coaster-broad-sweep §API response shape).
#[test]
fn test_armis_aql_search_toml_response_path_updated() {
    let toml_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../prism-sensors/specs/armis.sensor.toml");

    let content = std::fs::read_to_string(&toml_path)
        .expect("S-DEMO-ARMIS-AQL-001 AC-004: armis.sensor.toml must exist");

    let doc: toml::Value = toml::from_str(&content)
        .expect("S-DEMO-ARMIS-AQL-001 AC-004: armis.sensor.toml must be valid TOML");

    let tables = doc["tables"]
        .as_array()
        .expect("S-DEMO-ARMIS-AQL-001 AC-004: armis.sensor.toml must have [[tables]] array");

    // Devices response_path.
    let devices_table = tables
        .iter()
        .find(|t| t["table_name"].as_str() == Some("devices"))
        .expect("S-DEMO-ARMIS-AQL-001 AC-004: must have devices table");

    let fetch_devices_step = devices_table["steps"]
        .as_array()
        .expect("S-DEMO-ARMIS-AQL-001 AC-004: devices must have steps")
        .iter()
        .find(|s| s["name"].as_str() == Some("fetch_devices"))
        .expect("S-DEMO-ARMIS-AQL-001 AC-004: must have fetch_devices step");

    let devices_response_path = fetch_devices_step["response_path"]
        .as_str()
        .expect("S-DEMO-ARMIS-AQL-001 AC-004: fetch_devices must have response_path");

    assert_eq!(
        devices_response_path,
        "$.data.results",
        "S-DEMO-ARMIS-AQL-001 AC-004: armis.sensor.toml devices.fetch_devices.response_path must be '$.data.results' (search envelope uses 'results' not 'devices'; ADR-031 §D8-a + BC-2.16.013 §Postconditions §2)"
    );

    // Alerts response_path.
    let alerts_table = tables
        .iter()
        .find(|t| t["table_name"].as_str() == Some("alerts"))
        .expect("S-DEMO-ARMIS-AQL-001 AC-004: must have alerts table");

    let fetch_alerts_step = alerts_table["steps"]
        .as_array()
        .expect("S-DEMO-ARMIS-AQL-001 AC-004: alerts must have steps")
        .iter()
        .find(|s| s["name"].as_str() == Some("fetch_alerts"))
        .expect("S-DEMO-ARMIS-AQL-001 AC-004: must have fetch_alerts step");

    let alerts_response_path = fetch_alerts_step["response_path"]
        .as_str()
        .expect("S-DEMO-ARMIS-AQL-001 AC-004: fetch_alerts must have response_path");

    assert_eq!(
        alerts_response_path,
        "$.data.results",
        "S-DEMO-ARMIS-AQL-001 AC-004: armis.sensor.toml alerts.fetch_alerts.response_path must be '$.data.results' (search envelope uses 'results' not 'alerts'; ADR-031 §D8-a + BC-2.16.013 §Postconditions §2)"
    );
}

// ---------------------------------------------------------------------------
// SAP-2 parity test: TOML columns map to DTU types.rs fields
// AC-004 / BC-2.16.013 DTU↔TOML parity
// ---------------------------------------------------------------------------

/// AC-004 / SAP-2 / BC-2.16.013 DTU↔TOML parity:
/// Every column declared in `armis.sensor.toml` under the `devices` table must
/// map to a field in `types.rs::DeviceRecord`. Every column declared under `alerts`
/// must map to a field in `types.rs::AlertRecord`.
///
/// This is a static-analysis test — no network I/O. It reads the TOML file and
/// verifies column names against the known DTU type fields from types.rs.
///
/// Per SAP-2: "Column in TOML with no DTU equivalent → P1 CRITICAL".
/// Red Gate note: this test passes today because the stub-architect wrote the TOML
/// with correct column names. It guards against regression during implementation.
#[test]
fn test_armis_aql_search_dtu_toml_column_parity() {
    // Canonical DeviceRecord fields from types.rs (grounded against the struct definition).
    // Matches: device_id, name, type (device_type with rename), manufacturer, os_name,
    // os_version, risk_score, risk_factors, last_seen, first_seen, ip_address,
    // mac_address, network_id, site, tags.
    // Note: "type" is the TOML column name — DeviceRecord uses #[serde(rename = "type")]
    // on device_type, so the JSON/TOML key is "type".
    let device_record_fields: std::collections::HashSet<&str> = [
        "device_id",
        "name",
        "type",
        "manufacturer",
        "os_name",
        "os_version",
        "risk_score",
        "risk_factors",
        "last_seen",
        "first_seen",
        "ip_address",
        "mac_address",
        "network_id",
        "site",
        "tags",
    ]
    .iter()
    .copied()
    .collect();

    // Canonical AlertRecord fields from types.rs.
    // Matches: alert_id, name, severity, status, policy_name, device_id, created_at, updated_at.
    let alert_record_fields: std::collections::HashSet<&str> = [
        "alert_id",
        "name",
        "severity",
        "status",
        "policy_name",
        "device_id",
        "created_at",
        "updated_at",
    ]
    .iter()
    .copied()
    .collect();

    let toml_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../prism-sensors/specs/armis.sensor.toml");

    let content = std::fs::read_to_string(&toml_path)
        .expect("S-DEMO-ARMIS-AQL-001 SAP-2: armis.sensor.toml must exist");

    let doc: toml::Value = toml::from_str(&content)
        .expect("S-DEMO-ARMIS-AQL-001 SAP-2: armis.sensor.toml must be valid TOML");

    let tables = doc["tables"]
        .as_array()
        .expect("S-DEMO-ARMIS-AQL-001 SAP-2: armis.sensor.toml must have [[tables]]");

    for table in tables {
        let table_name = table["table_name"]
            .as_str()
            .expect("S-DEMO-ARMIS-AQL-001 SAP-2: each table must have table_name");

        let columns = match table.get("columns") {
            Some(c) => c
                .as_array()
                .expect("S-DEMO-ARMIS-AQL-001 SAP-2: columns must be an array"),
            None => continue, // No columns declared — not a parity violation
        };

        let dtu_fields = match table_name {
            "devices" => &device_record_fields,
            "alerts" => &alert_record_fields,
            _ => continue, // Unknown table — not in scope for this parity check
        };

        for col in columns {
            let col_name = col["name"]
                .as_str()
                .expect("S-DEMO-ARMIS-AQL-001 SAP-2: each column must have a name");

            assert!(
                dtu_fields.contains(col_name),
                "S-DEMO-ARMIS-AQL-001 SAP-2 CRITICAL: TOML column '{col_name}' in table '{table_name}' has no equivalent field in the DTU types.rs struct — runtime normalization will silently produce empty/wrong data (SAP-2 P1 CRITICAL finding pattern)"
            );
        }
    }
}
