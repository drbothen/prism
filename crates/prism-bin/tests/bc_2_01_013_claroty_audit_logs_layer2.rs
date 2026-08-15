// SPDX-License-Identifier: Apache-2.0
//! Red Gate failing tests for S-CLAROTY-AUDITLOG-TIMEBOX-001.
//!
//! Five tests covering ADR-033 Option T1 time-filter push-down for the
//! Claroty xDome `audit_logs` table. The current production TOML has
//! `body_template = '{}'` with no time-filter injection. Every test FAILS
//! before implementation because the outbound POST body contains no
//! `filter_by` field.
//!
//! # Red Gate test list
//!
//! | ID     | Test name | Primary failing assertion |
//! |--------|-----------|--------------------------|
//! | RG-001 | test_BC_2_01_013_claroty_audit_logs_layer2_no_filter_injects_default_greater_or_equal | body.get("filter_by").is_some() |
//! | RG-002 | test_BC_2_01_013_claroty_audit_logs_layer2_explicit_start_time_honored_not_truncated | body.get("filter_by").is_some() |
//! | RG-003 | test_BC_2_01_013_claroty_audit_logs_layer2_both_bounds_compound_and | body.get("filter_by").is_some() |
//! | RG-005 | test_BC_2_01_013_claroty_audit_logs_layer2_filter_rejection_4xx_surfaces_e_sensor_001 | body.get("filter_by").is_some() |
//!
//! RG-004 (pipeline.rs backward-compat JSON parsing) lives in
//! `crates/prism-spec-engine/src/pipeline.rs` as an in-module unit test.
//!
//! # Wire-shape discipline (CLAUDE.md §Conventions)
//!
//! Every test asserts on the serialized POST body — the actual JSON bytes
//! sent over the wire to the mock xDome server — not on pre-serialization
//! Rust structures. `mock_server.received_requests()` captures the
//! outbound HTTP request body for wire-level assertions.
//!
//! # SID-1 compliance (CLAUDE.md §SID-1)
//!
//! No `#[ignore]` used. HTTP boundary mocked via `wiremock 0.6`.
//! No live DTU clone or external service required.
//!
//! # Failure mechanism (before implementation)
//!
//! Production TOML: `body_template = '{}'`. After OffsetLimit merge the body
//! is `{"offset": 0, "limit": 1000}` — no `filter_by` field. The load-bearing
//! assertion `body.get("filter_by").is_some()` fails with an assertion panic
//! (not a build error, not a `todo!()` panic).
//!
//! BCs: BC-2.01.013, BC-2.16.013
//! Story: S-CLAROTY-AUDITLOG-TIMEBOX-001 v2.0

#![allow(
    dead_code,
    unused_imports,
    non_snake_case,
    clippy::unwrap_used,
    clippy::expect_used
)]
extern crate toml;

use std::sync::Arc;

use prism_bin::spec_driven_adapter::{AdapterAuthStrategy, SpecDrivenSensorAdapter};
use prism_core::{OrgId, OrgSlug};
use prism_sensors::{
    BearerStaticSensorAuth, SensorAdapter, SensorError,
    adapter::{QueryParams, SensorSpec as SensorAdapterSpec},
};
use prism_spec_engine::{
    overlay::{OverlayLoader, SensorInstanceOverlay},
    spec_parser::SpecLoader,
};
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{method, path},
};

// ---------------------------------------------------------------------------
// Test fixtures
// ---------------------------------------------------------------------------

/// Build a `SpecDrivenSensorAdapter` from the production `claroty.sensor.toml`
/// directed at the given mock server URI.
///
/// Uses `OverlayLoader::merge_overlay_onto_type_spec` — the only valid
/// external construction path for `#[non_exhaustive]` `ResolvedSensorSpec`.
fn make_claroty_adapter(mock_server_uri: &str) -> SpecDrivenSensorAdapter {
    let spec_content = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../prism-sensors/specs/claroty.sensor.toml"),
    )
    .expect("S-CLAROTY-AUDITLOG-TIMEBOX-001: claroty.sensor.toml must be readable from CARGO_MANIFEST_DIR/../prism-sensors/specs/");

    let mut spec = SpecLoader::parse(&spec_content)
        .expect("S-CLAROTY-AUDITLOG-TIMEBOX-001: claroty.sensor.toml must parse cleanly");
    spec.base_url = mock_server_uri.to_string();

    let overlay_toml = "extends = \"claroty\"\ninstance_id = \"claroty@claroty-layer2-test-org\"";
    let overlay: SensorInstanceOverlay = toml::from_str(overlay_toml)
        .expect("test fixture: SensorInstanceOverlay TOML parse failed");
    let resolved = OverlayLoader::merge_overlay_onto_type_spec(
        &spec,
        &overlay,
        OrgSlug::new("claroty-layer2-test-org"),
    );

    let http_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("test fixture: reqwest::Client construction failed (ADR-050 rustls-tls)");

    SpecDrivenSensorAdapter::new(
        Arc::new(resolved),
        AdapterAuthStrategy::BearerStatic,
        http_client,
    )
}

/// Build a `SensorAdapterSpec` targeting the Claroty audit_logs table.
///
/// `source_table = "claroty_audit_logs"` → the adapter strips the "claroty_"
/// prefix and routes to the `audit_logs` table in the claroty sensor spec.
fn make_audit_log_adapter_spec() -> SensorAdapterSpec {
    #[allow(deprecated)]
    SensorAdapterSpec {
        source_table: "claroty_audit_logs".to_string(),
        org_id: OrgId::from_uuid(uuid::Uuid::now_v7()),
        #[allow(deprecated)]
        client_id: "claroty-layer2-test-org".to_string(),
        sensor_config: serde_json::json!({}),
    }
}

/// Minimal audit_log response for the wiremock mock server.
///
/// Returns 1 record (well below the page_size=1000 threshold), causing
/// OffsetLimit pagination to stop after the first page.
/// Shape: `{"audit_log": [...], "total": N}` — matches `response_path = "$.audit_log"`.
fn audit_log_response_one_record() -> serde_json::Value {
    serde_json::json!({
        "audit_log": [
            {
                "id": "audit-layer2-test-001",
                "action": "login",
                "user_display_name": "Layer2 Test User",
                "category": "authentication",
                "timestamp": "2026-08-01T12:00:00Z",
                "details": "Red Gate test login event",
                "username": "layer2-tester"
            }
        ],
        "total": 1
    })
}

/// Parse the POST body bytes from a received wiremock request as JSON.
///
/// Returns the parsed `serde_json::Value`. Panics with a descriptive message
/// if the body is not UTF-8 or not valid JSON.
fn parse_received_body(body_bytes: &[u8]) -> serde_json::Value {
    let body_str = std::str::from_utf8(body_bytes).expect("received POST body must be valid UTF-8");
    serde_json::from_str(body_str).unwrap_or_else(|e| {
        panic!(
            "received POST body must be valid JSON. Parse error: {e}. \
             Raw body (first 512 bytes): {:?}",
            &body_str[..body_str.len().min(512)]
        )
    })
}

// ---------------------------------------------------------------------------
// RG-001: BC-2.01.013 §Postcondition 1 — default 7-day look-back
//
// When `QueryParams.start_time = None` and `QueryParams.end_time = None`,
// the outbound POST body MUST contain a `filter_by` with
// `operation = "greater_or_equal"` and `value ≈ now() - 7 days`.
//
// CURRENT FAILURE: body = {"offset": 0, "limit": 1000} — no filter_by.
// POST body is missing the default look-back filter.
//
// After implementation: body = {"filter_by": {"field": "timestamp",
//   "operation": "greater_or_equal", "value": <now_ms - 604800000>},
//   "offset": 0, "limit": 1000}
// ---------------------------------------------------------------------------

/// AC-001 / BC-2.01.013 §Postcondition 1:
/// A Claroty `audit_logs` fetch with NO time filter produces a POST body
/// carrying `filter_by.operation = "greater_or_equal"` and a value ≈ 7 days ago.
///
/// # Red Gate Failure
///
/// `body.get("filter_by").is_some()` FAILS because the current body is
/// `{"offset": 0, "limit": 1000}` — the `body_template = '{}'` in
/// `claroty.sensor.toml` has no filter injection.
///
/// BCs: BC-2.01.013 §Postcondition 1; BC-2.16.013
/// Story: S-CLAROTY-AUDITLOG-TIMEBOX-001 AC-001 / RG-001.
#[tokio::test]
async fn test_BC_2_01_013_claroty_audit_logs_layer2_no_filter_injects_default_greater_or_equal() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/v1/audit_log/get"))
        .respond_with(ResponseTemplate::new(200).set_body_json(audit_log_response_one_record()))
        .mount(&mock_server)
        .await;

    let adapter = make_claroty_adapter(&mock_server.uri());
    let adapter_spec = make_audit_log_adapter_spec();

    let params = QueryParams {
        cursor: None,
        limit: 10,
        start_time: None, // No time filter → MUST inject 7-day default look-back.
        end_time: None,
        filters: Default::default(),
    };

    let sensor_auth = BearerStaticSensorAuth::new("claroty-layer2-rg001-token");

    // Run the fetch. With the current implementation (body_template = '{}'),
    // the POST body will be {"offset": 0, "limit": 1000} — no filter_by.
    let _result = adapter.fetch(&adapter_spec, &params, &sensor_auth).await;

    // Verify the mock received the outbound POST request.
    let requests = mock_server.received_requests().await.unwrap_or_default();
    assert!(
        !requests.is_empty(),
        "RG-001: SpecDrivenSensorAdapter::fetch must have issued a POST to \
         /api/v1/audit_log/get. Got no requests. \
         Check that source_table=\"claroty_audit_logs\" routes to the audit_logs \
         table in claroty.sensor.toml."
    );

    // Wire-shape assertion: parse the outbound POST body.
    // With the current body_template = '{}', the OffsetLimit merge produces
    // {"offset": 0, "limit": 1000} — valid JSON, but no filter_by.
    let body = parse_received_body(&requests[0].body);

    // LOAD-BEARING Red Gate assertion (RG-001):
    // The POST body MUST contain a 'filter_by' key when no time params provided.
    // FAILS BEFORE IMPLEMENTATION: body = {"offset": 0, "limit": 1000}
    assert!(
        body.get("filter_by").is_some(),
        "RG-001 LOAD-BEARING: POST body to xDome /api/v1/audit_log/get MUST contain \
         'filter_by' when QueryParams.start_time is None (7-day default look-back). \
         Got body: {}. \
         Root cause: spec_driven_adapter.rs does not inject _claroty_audit_filter_by \
         into FetchContext.query_filters; claroty.sensor.toml body_template = '{{}}' \
         has no filter slot; pipeline.rs step_vars seeding cannot see the filter. \
         Fix: (1) spec_driven_adapter.rs: when sensor_id == \"claroty\" and \
         source_table == \"claroty_audit_logs\", inject \
         query_filters[\"_claroty_audit_filter_by\"] = {{\"field\": \"timestamp\", \
         \"operation\": \"greater_or_equal\", \"value\": <now_ms - 7*24*3600*1000>}}; \
         (2) claroty.sensor.toml: update body_template to include \
         ${{query.filter._claroty_audit_filter_by}}; \
         (3) pipeline.rs: parse JSON-string query_filters to Value::Object \
         (BC-2.16.013). S-CLAROTY-AUDITLOG-TIMEBOX-001 AC-001.",
        body
    );

    // Secondary assertions (run after Red Gate passes):
    let filter_by = &body["filter_by"];
    assert_eq!(
        filter_by["operation"].as_str().unwrap_or(""),
        "greater_or_equal",
        "RG-001: filter_by.operation must be 'greater_or_equal' for a single lower-bound \
         time filter. Got: {:?}. BC-2.01.013 §Postcondition 1.",
        filter_by["operation"]
    );
    assert_eq!(
        filter_by["field"].as_str().unwrap_or(""),
        "timestamp",
        "RG-001: filter_by.field must be 'timestamp' (the xDome audit_log timestamp field). \
         Got: {:?}. BC-2.01.013 §Postcondition 1.",
        filter_by["field"]
    );

    // Value must be a number approximately 7 days ago (±60 seconds tolerance).
    let value = filter_by["value"].as_i64().unwrap_or(0);
    let now_ms = chrono::Utc::now().timestamp_millis();
    let seven_days_ms: i64 = 7 * 24 * 3600 * 1000;
    let expected = now_ms - seven_days_ms;
    let tolerance_ms: i64 = 60_000; // 60-second window for test execution time
    assert!(
        value >= expected - tolerance_ms && value <= expected + tolerance_ms,
        "RG-001: filter_by.value must be ≈ now_ms - 7 days (604,800,000 ms). \
         Expected range [{}, {}], got {}. Delta: {} ms. \
         BC-2.01.013 §Postcondition 1; S-CLAROTY-AUDITLOG-TIMEBOX-001 AC-001.",
        expected - tolerance_ms,
        expected + tolerance_ms,
        value,
        value - expected
    );
}

// ---------------------------------------------------------------------------
// RG-002: BC-2.01.013 §Postcondition 2 — explicit start_time not truncated
//
// When `QueryParams.start_time = Some(t)` where t is ~45 days ago, the
// outbound POST body MUST use t as the lower bound — NOT the 7-day default.
// Proves the implementation does not silently cap explicit look-backs.
//
// CURRENT FAILURE: body = {"offset": 0, "limit": 1000} — no filter_by.
// ---------------------------------------------------------------------------

/// AC-002 / BC-2.01.013 §Postcondition 2:
/// An explicit `start_time` of ~45 days ago is honored as-is in the POST body
/// — the filter uses the explicit timestamp, NOT the 7-day default look-back.
///
/// # Red Gate Failure
///
/// `body.get("filter_by").is_some()` FAILS because the current body is
/// `{"offset": 0, "limit": 1000}` — no time filter of any kind.
///
/// BCs: BC-2.01.013 §Postcondition 2; BC-2.16.013
/// Story: S-CLAROTY-AUDITLOG-TIMEBOX-001 AC-002 / RG-002.
#[tokio::test]
async fn test_BC_2_01_013_claroty_audit_logs_layer2_explicit_start_time_honored_not_truncated() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/v1/audit_log/get"))
        .respond_with(ResponseTemplate::new(200).set_body_json(audit_log_response_one_record()))
        .mount(&mock_server)
        .await;

    let adapter = make_claroty_adapter(&mock_server.uri());
    let adapter_spec = make_audit_log_adapter_spec();

    // Explicit start_time ≈ 45 days ago. Well outside the 7-day default window.
    // After implementation, the POST body must use this exact timestamp,
    // not the 7-day fallback — proving no silent truncation.
    let explicit_start = "2026-07-01T00:00:00Z"; // ~45 days before 2026-08-15
    let params = QueryParams {
        cursor: None,
        limit: 10,
        start_time: Some(explicit_start.to_string()),
        end_time: None,
        filters: Default::default(),
    };

    let sensor_auth = BearerStaticSensorAuth::new("claroty-layer2-rg002-token");

    let _result = adapter.fetch(&adapter_spec, &params, &sensor_auth).await;

    let requests = mock_server.received_requests().await.unwrap_or_default();
    assert!(
        !requests.is_empty(),
        "RG-002: SpecDrivenSensorAdapter::fetch must have issued a POST to \
         /api/v1/audit_log/get."
    );

    let body = parse_received_body(&requests[0].body);

    // LOAD-BEARING Red Gate assertion (RG-002):
    // The POST body MUST contain a 'filter_by' using the EXPLICIT 45-day start_time.
    // FAILS BEFORE IMPLEMENTATION: body = {"offset": 0, "limit": 1000}
    assert!(
        body.get("filter_by").is_some(),
        "RG-002 LOAD-BEARING: POST body to xDome /api/v1/audit_log/get MUST contain \
         'filter_by' when QueryParams.start_time = Some(\"{explicit_start}\"). \
         Got body: {body}. \
         Root cause: same as RG-001 — spec_driven_adapter.rs does not inject \
         _claroty_audit_filter_by into FetchContext.query_filters. \
         Fix: inject the EXPLICIT start_time as the filter value, NOT the 7-day default. \
         BC-2.01.013 §Postcondition 2; S-CLAROTY-AUDITLOG-TIMEBOX-001 AC-002."
    );

    // Secondary assertions (run after Red Gate passes):
    let filter_by = &body["filter_by"];
    assert_eq!(
        filter_by["operation"].as_str().unwrap_or(""),
        "greater_or_equal",
        "RG-002: filter_by.operation must be 'greater_or_equal' for a single lower-bound. \
         Got: {:?}.",
        filter_by["operation"]
    );

    // The filter value must correspond to the explicit start_time (2026-07-01T00:00:00Z).
    // Compute the expected millisecond epoch from the explicit ISO-8601 string.
    let explicit_dt = chrono::DateTime::parse_from_rfc3339(explicit_start)
        .expect("test fixture: explicit_start must be a valid RFC3339 datetime")
        .timestamp_millis();
    let actual_value = filter_by["value"].as_i64().unwrap_or(0);
    let tolerance_ms: i64 = 60_000; // 60-second tolerance
    assert!(
        actual_value >= explicit_dt - tolerance_ms && actual_value <= explicit_dt + tolerance_ms,
        "RG-002 no-truncation assertion: filter_by.value must equal the EXPLICIT \
         start_time ({explicit_start} = {explicit_dt} ms). \
         Got {actual_value} (delta: {} ms). \
         The implementation MUST NOT substitute the 7-day fallback when an explicit \
         start_time is provided. BC-2.01.013 §Postcondition 2.",
        actual_value - explicit_dt
    );

    // Confirm it is NOT the 7-day default (≈ now - 604,800,000 ms).
    let seven_day_default_ms = chrono::Utc::now().timestamp_millis() - 7_i64 * 24 * 3600 * 1000;
    let delta_from_7day = (actual_value - seven_day_default_ms).abs();
    assert!(
        delta_from_7day > 30_i64 * 24 * 3600 * 1000, // more than 30 days apart
        "RG-002 no-truncation assertion: filter_by.value must NOT be the 7-day default. \
         Got value {} which is only {} ms from the 7-day default {}. \
         The implementation must use the explicit start_time, not fall back to 7 days. \
         BC-2.01.013 §Postcondition 2.",
        actual_value,
        delta_from_7day,
        seven_day_default_ms
    );
}

// ---------------------------------------------------------------------------
// RG-003: BC-2.01.013 §Postcondition 3 — both bounds → compound AND filter
//
// When both `start_time` and `end_time` are provided, the POST body MUST
// contain a compound `filter_by` with `operation = "and"` and two conditions:
// `greater_or_equal` on start_time and `less_or_equal` on end_time.
//
// CURRENT FAILURE: body = {"offset": 0, "limit": 1000} — no filter_by.
// ---------------------------------------------------------------------------

/// AC-003 / BC-2.01.013 §Postcondition 3:
/// When both `start_time` and `end_time` are provided, the POST body carries
/// a compound `filter_by` with `operation = "and"` containing two sub-conditions:
/// `greater_or_equal` (start_time) and `less_or_equal` (end_time).
///
/// # Red Gate Failure
///
/// `body.get("filter_by").is_some()` FAILS because the current body is
/// `{"offset": 0, "limit": 1000}` — no filter at all.
///
/// BCs: BC-2.01.013 §Postcondition 3; BC-2.16.013
/// Story: S-CLAROTY-AUDITLOG-TIMEBOX-001 AC-003 / RG-003.
#[tokio::test]
async fn test_BC_2_01_013_claroty_audit_logs_layer2_both_bounds_compound_and() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/v1/audit_log/get"))
        .respond_with(ResponseTemplate::new(200).set_body_json(audit_log_response_one_record()))
        .mount(&mock_server)
        .await;

    let adapter = make_claroty_adapter(&mock_server.uri());
    let adapter_spec = make_audit_log_adapter_spec();

    let start_time = "2026-07-01T00:00:00Z";
    let end_time = "2026-08-10T00:00:00Z";
    let params = QueryParams {
        cursor: None,
        limit: 10,
        start_time: Some(start_time.to_string()),
        end_time: Some(end_time.to_string()),
        filters: Default::default(),
    };

    let sensor_auth = BearerStaticSensorAuth::new("claroty-layer2-rg003-token");

    let _result = adapter.fetch(&adapter_spec, &params, &sensor_auth).await;

    let requests = mock_server.received_requests().await.unwrap_or_default();
    assert!(
        !requests.is_empty(),
        "RG-003: SpecDrivenSensorAdapter::fetch must have issued a POST to \
         /api/v1/audit_log/get."
    );

    let body = parse_received_body(&requests[0].body);

    // LOAD-BEARING Red Gate assertion (RG-003):
    // The POST body MUST contain 'filter_by' when both bounds are provided.
    // FAILS BEFORE IMPLEMENTATION: body = {"offset": 0, "limit": 1000}
    assert!(
        body.get("filter_by").is_some(),
        "RG-003 LOAD-BEARING: POST body to xDome /api/v1/audit_log/get MUST contain \
         'filter_by' when both start_time and end_time are provided. \
         Got body: {body}. \
         Root cause: same as RG-001/002. Fix: build a compound AND filter \
         with greater_or_equal(start_time) and less_or_equal(end_time). \
         BC-2.01.013 §Postcondition 3; S-CLAROTY-AUDITLOG-TIMEBOX-001 AC-003."
    );

    // Secondary assertions (run after Red Gate passes):
    let filter_by = &body["filter_by"];
    assert_eq!(
        filter_by["operation"].as_str().unwrap_or(""),
        "and",
        "RG-003: filter_by.operation must be 'and' when both start_time and end_time \
         are provided. Got: {:?}. BC-2.01.013 §Postcondition 3.",
        filter_by["operation"]
    );

    // The AND filter must have two conditions.
    let conditions = filter_by["conditions"]
        .as_array()
        .unwrap_or(&vec![])
        .clone();
    assert_eq!(
        conditions.len(),
        2,
        "RG-003: filter_by.conditions must have exactly 2 elements (start + end bound). \
         Got {} conditions: {:?}. BC-2.01.013 §Postcondition 3.",
        conditions.len(),
        conditions
    );

    // One condition must be greater_or_equal (start_time lower bound).
    let has_gte = conditions
        .iter()
        .any(|c| c["operation"].as_str() == Some("greater_or_equal"));
    assert!(
        has_gte,
        "RG-003: one condition must have operation = 'greater_or_equal' (lower bound). \
         Conditions: {:?}. BC-2.01.013 §Postcondition 3.",
        conditions
    );

    // One condition must be less_or_equal (end_time upper bound).
    let has_lte = conditions
        .iter()
        .any(|c| c["operation"].as_str() == Some("less_or_equal"));
    assert!(
        has_lte,
        "RG-003: one condition must have operation = 'less_or_equal' (upper bound). \
         Conditions: {:?}. BC-2.01.013 §Postcondition 3.",
        conditions
    );

    // Validate the lower-bound value matches start_time.
    let gte_condition = conditions
        .iter()
        .find(|c| c["operation"].as_str() == Some("greater_or_equal"))
        .expect("gte condition must exist");
    let start_ms = chrono::DateTime::parse_from_rfc3339(start_time)
        .expect("test fixture: start_time must be valid RFC3339")
        .timestamp_millis();
    let gte_value = gte_condition["value"].as_i64().unwrap_or(0);
    assert!(
        (gte_value - start_ms).abs() <= 60_000,
        "RG-003: greater_or_equal value must correspond to start_time ({start_time} = {start_ms} ms). \
         Got {gte_value}. BC-2.01.013 §Postcondition 3.",
    );

    // Validate the upper-bound value matches end_time.
    let lte_condition = conditions
        .iter()
        .find(|c| c["operation"].as_str() == Some("less_or_equal"))
        .expect("lte condition must exist");
    let end_ms = chrono::DateTime::parse_from_rfc3339(end_time)
        .expect("test fixture: end_time must be valid RFC3339")
        .timestamp_millis();
    let lte_value = lte_condition["value"].as_i64().unwrap_or(0);
    assert!(
        (lte_value - end_ms).abs() <= 60_000,
        "RG-003: less_or_equal value must correspond to end_time ({end_time} = {end_ms} ms). \
         Got {lte_value}. BC-2.01.013 §Postcondition 3.",
    );
}

// ---------------------------------------------------------------------------
// RG-005: BC-2.01.013 §Postcondition 4 — 4xx response surfaces E-SENSOR-001
//
// When xDome returns a 4xx HTTP status, the adapter MUST surface
// `SensorError::HttpError { status }`, not panic or return empty Vec.
//
// This test has a DUAL Red Gate:
// (1) PRIMARY FAIL: body.get("filter_by").is_some() — proves filter injection works
//     before we even get to check the error type
// (2) SECONDARY (post-impl): result is Err(SensorError::HttpError { status: 400 })
//
// CURRENT FAILURE (primary): body = {"offset": 0, "limit": 1000} — no filter_by.
// ---------------------------------------------------------------------------

/// AC-004 / BC-2.01.013 §Postcondition 4:
/// A 400 response from xDome causes `fetch()` to return
/// `Err(SensorError::HttpError { status: 400 })`.
///
/// # Dual Red Gate Failure
///
/// Primary (fires first): `body.get("filter_by").is_some()` FAILS — the POST body
/// has no `filter_by` field. This failure is intentional: it ensures the FILTER
/// INJECTION is implemented before the error-path behavior is checked.
///
/// Secondary (fires after primary passes): `result.is_err()` and error type check.
/// The 4xx error path already works in the current code; the primary assertion
/// gates on filter injection being present first.
///
/// BCs: BC-2.01.013 §Postcondition 4 (error surfacing); BC-2.16.013 (filter injection)
/// Story: S-CLAROTY-AUDITLOG-TIMEBOX-001 AC-004 / RG-005.
#[tokio::test]
async fn test_BC_2_01_013_claroty_audit_logs_layer2_filter_rejection_4xx_surfaces_e_sensor_001() {
    let mock_server = MockServer::start().await;

    // Mount a 400 Bad Request response. The mock captures the request body even
    // when it returns an error status — wiremock always records received requests.
    Mock::given(method("POST"))
        .and(path("/api/v1/audit_log/get"))
        .respond_with(
            ResponseTemplate::new(400)
                .set_body_string(r#"{"error": "Bad Request", "message": "invalid filter"}"#),
        )
        .mount(&mock_server)
        .await;

    let adapter = make_claroty_adapter(&mock_server.uri());
    let adapter_spec = make_audit_log_adapter_spec();

    let params = QueryParams {
        cursor: None,
        limit: 10,
        start_time: None, // No time filter — 7-day default should be injected
        end_time: None,
        filters: Default::default(),
    };

    let sensor_auth = BearerStaticSensorAuth::new("claroty-layer2-rg005-token");

    // Execute the fetch. Expected: Err due to 4xx response.
    let result = adapter.fetch(&adapter_spec, &params, &sensor_auth).await;

    // The mock captured the outbound POST even though it returned 400.
    let requests = mock_server.received_requests().await.unwrap_or_default();
    assert!(
        !requests.is_empty(),
        "RG-005: SpecDrivenSensorAdapter::fetch must have issued a POST to \
         /api/v1/audit_log/get before receiving the 400."
    );

    let body = parse_received_body(&requests[0].body);

    // PRIMARY LOAD-BEARING Red Gate assertion (RG-005):
    // The POST body MUST contain 'filter_by' — proves filter injection works
    // (the error path is tested AFTER filter injection is verified).
    // FAILS BEFORE IMPLEMENTATION: body = {"offset": 0, "limit": 1000}
    assert!(
        body.get("filter_by").is_some(),
        "RG-005 PRIMARY LOAD-BEARING: POST body to xDome /api/v1/audit_log/get MUST \
         contain 'filter_by' even on paths that return 4xx. Got body: {body}. \
         Root cause: filter injection not yet implemented. \
         Fix: implement filter injection in spec_driven_adapter.rs first (see RG-001). \
         This assertion gates the error-type check below — both must pass. \
         BC-2.01.013 §Postcondition 4; S-CLAROTY-AUDITLOG-TIMEBOX-001 AC-004."
    );

    // SECONDARY assertion (runs after primary passes):
    // The 4xx MUST surface as SensorError::HttpError { status: 400 }.
    assert!(
        result.is_err(),
        "RG-005 SECONDARY: fetch() must return Err when xDome returns 400. \
         Got Ok. BC-2.01.013 §Postcondition 4."
    );

    let err = result.unwrap_err();
    match &err {
        SensorError::HttpError { status, .. } => {
            assert_eq!(
                *status, 400u16,
                "RG-005 SECONDARY: SensorError::HttpError status must be 400. \
                 Got status {}. BC-2.01.013 §Postcondition 4.",
                status
            );
        }
        other => {
            panic!(
                "RG-005 SECONDARY: fetch() must return SensorError::HttpError{{400}} \
                 for a 400 xDome response. Got: {other:?}. \
                 Check map_spec_engine_error_to_sensor_error in spec_driven_adapter.rs."
            );
        }
    }
}
