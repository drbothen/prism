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
//! | RG-006 | test_BC_2_01_013_claroty_audit_logs_layer2_end_only_single_less_or_equal | body.get("filter_by").is_some() |
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
//! Story: S-CLAROTY-AUDITLOG-TIMEBOX-001

#![allow(
    dead_code,
    unused_imports,
    non_snake_case,
    clippy::unwrap_used,
    clippy::expect_used
)]
extern crate toml;

use std::sync::Arc;

use std::collections::HashMap;

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
//   "operation": "greater_or_equal", "value": "<now-7d as ISO-8601 RFC3339>"},
//   "offset": 0, "limit": 1000}
// BC-2.01.013 EC-01-030: value MUST be an ISO-8601 STRING, NOT an epoch-ms integer.
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
         \"operation\": \"greater_or_equal\", \"value\": \"<now_minus_7d as ISO-8601 RFC3339 string>\"}}; \
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

    // Value must be an ISO-8601 string approximately 7 days ago (±60 seconds tolerance).
    // BC-2.01.013 EC-01-030: all value fields MUST be ISO-8601 strings, NOT epoch integers.
    let value_str = filter_by["value"].as_str();
    assert!(
        value_str.is_some(),
        "RG-001 ISO-8601 assertion: filter_by.value must be an ISO-8601 STRING \
         (serde_json::Value::String), NOT an epoch-millisecond integer. Got: {:?}. \
         BC-2.01.013 EC-01-030; S-CLAROTY-AUDITLOG-TIMEBOX-001 AC-001.",
        filter_by["value"]
    );
    let value_dt = chrono::DateTime::parse_from_rfc3339(value_str.unwrap());
    assert!(
        value_dt.is_ok(),
        "RG-001 ISO-8601 assertion: filter_by.value must be a parseable RFC3339/ISO-8601 \
         string. Got: {:?}. Parse error: {:?}. BC-2.01.013 EC-01-030.",
        value_str.unwrap(),
        value_dt.err()
    );
    let value_secs = value_dt.unwrap().timestamp();
    let now_secs = chrono::Utc::now().timestamp();
    let seven_days_secs: i64 = 7 * 24 * 3600;
    let expected_secs = now_secs - seven_days_secs;
    let tolerance_secs: i64 = 60; // 60-second window for test execution time
    assert!(
        value_secs >= expected_secs - tolerance_secs
            && value_secs <= expected_secs + tolerance_secs,
        "RG-001: filter_by.value must be ≈ now - 7 days (604,800 seconds). \
         Expected range [{}, {}] (seconds), got {} (seconds). Delta: {} s. \
         BC-2.01.013 EC-01-030; S-CLAROTY-AUDITLOG-TIMEBOX-001 AC-001.",
        expected_secs - tolerance_secs,
        expected_secs + tolerance_secs,
        value_secs,
        value_secs - expected_secs
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

    // Value must be an ISO-8601 string equal to the explicit start_time.
    // BC-2.01.013 EC-01-031: all value fields MUST be ISO-8601 strings, NOT epoch integers.
    let value_str = filter_by["value"].as_str();
    assert!(
        value_str.is_some(),
        "RG-002 ISO-8601 assertion: filter_by.value must be an ISO-8601 STRING \
         (serde_json::Value::String), NOT an epoch-millisecond integer. Got: {:?}. \
         BC-2.01.013 EC-01-031; S-CLAROTY-AUDITLOG-TIMEBOX-001 AC-002.",
        filter_by["value"]
    );
    let value_dt = chrono::DateTime::parse_from_rfc3339(value_str.unwrap());
    assert!(
        value_dt.is_ok(),
        "RG-002: filter_by.value must parse as RFC3339/ISO-8601. Got: {:?}. \
         BC-2.01.013 EC-01-031.",
        value_str
    );

    // Verify the parsed timestamp matches the explicit start_time.
    let explicit_dt = chrono::DateTime::parse_from_rfc3339(explicit_start)
        .expect("test fixture: explicit_start must be a valid RFC3339 datetime");
    let value_secs = value_dt.unwrap().timestamp();
    let explicit_secs = explicit_dt.timestamp();
    assert!(
        (value_secs - explicit_secs).abs() <= 60,
        "RG-002 no-truncation assertion: filter_by.value must equal the EXPLICIT \
         start_time ({explicit_start} = {explicit_secs}s). Got {value_secs}s \
         (delta: {}s). The implementation MUST NOT substitute the 7-day fallback. \
         BC-2.01.013 EC-01-031.",
        value_secs - explicit_secs
    );

    // Confirm it is NOT the 7-day default (≈ now - 604,800 s).
    let seven_day_default_secs = chrono::Utc::now().timestamp() - 7_i64 * 24 * 3600;
    let delta_from_7day_secs = (value_secs - seven_day_default_secs).abs();
    assert!(
        delta_from_7day_secs > 30_i64 * 24 * 3600, // more than 30 days apart
        "RG-002 no-truncation: filter_by.value MUST NOT be the 7-day default. \
         Got {value_secs}s which is only {delta_from_7day_secs}s from the 7-day \
         default {seven_day_default_secs}s. BC-2.01.013 EC-01-031.",
    );
}

// ---------------------------------------------------------------------------
// RG-003: BC-2.01.013 §Postcondition 3 — both bounds → compound AND filter
//
// When both `start_time` and `end_time` are provided, the POST body MUST
// contain a compound `filter_by` with `operation = "and"` and two operands:
// `greater_or_equal` on start_time and `less_or_equal` on end_time.
// Compound key MUST be "operands" (NOT "conditions"). BC-2.01.013 EC-01-033.
// All value fields MUST be ISO-8601 strings, NOT epoch-ms integers.
//
// CURRENT FAILURE: body = {"offset": 0, "limit": 1000} — no filter_by.
// ---------------------------------------------------------------------------

/// AC-003 / BC-2.01.013 §Postcondition 3:
/// When both `start_time` and `end_time` are provided, the POST body carries
/// a compound `filter_by` with `operation = "and"` and compound key `"operands"`
/// (NOT `"conditions"`) containing two operands: `greater_or_equal` (start_time)
/// and `less_or_equal` (end_time). All value fields are ISO-8601 strings.
/// BC-2.01.013 EC-01-033.
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

    // The AND filter must have two operands. Key MUST be "operands" (NOT "conditions").
    // BC-2.01.013 EC-01-033: compound filter uses `operands`, NOT `conditions`.
    assert!(
        filter_by.get("operands").is_some(),
        "RG-003 LOAD-BEARING: compound filter MUST use key 'operands' (NOT 'conditions'). \
         filter_by keys: {:?}. BC-2.01.013 EC-01-033; AC-003.",
        filter_by.as_object().map(|o| o.keys().collect::<Vec<_>>())
    );
    assert!(
        filter_by.get("conditions").is_none(),
        "RG-003: compound filter MUST NOT use key 'conditions' (wrong key — use 'operands'). \
         BC-2.01.013 EC-01-033; AC-003.",
    );
    let operands = filter_by["operands"].as_array().unwrap_or(&vec![]).clone();
    assert_eq!(
        operands.len(),
        2,
        "RG-003: filter_by.operands must have exactly 2 elements (start + end bound). \
         Got {} operands: {:?}. BC-2.01.013 EC-01-033.",
        operands.len(),
        operands
    );

    // One operand must be greater_or_equal (start_time lower bound).
    let has_gte = operands
        .iter()
        .any(|c| c["operation"].as_str() == Some("greater_or_equal"));
    assert!(
        has_gte,
        "RG-003: one operand must have operation = 'greater_or_equal' (lower bound). \
         Operands: {:?}. BC-2.01.013 EC-01-033.",
        operands
    );

    // One operand must be less_or_equal (end_time upper bound).
    let has_lte = operands
        .iter()
        .any(|c| c["operation"].as_str() == Some("less_or_equal"));
    assert!(
        has_lte,
        "RG-003: one operand must have operation = 'less_or_equal' (upper bound). \
         Operands: {:?}. BC-2.01.013 EC-01-033.",
        operands
    );

    // Validate the lower-bound value is an ISO-8601 string matching start_time.
    // BC-2.01.013 EC-01-033: all value fields MUST be ISO-8601 strings, NOT epoch integers.
    let gte_operand = operands
        .iter()
        .find(|c| c["operation"].as_str() == Some("greater_or_equal"))
        .expect("gte operand must exist");
    let gte_value_str = gte_operand["value"].as_str();
    assert!(
        gte_value_str.is_some(),
        "RG-003: greater_or_equal value must be an ISO-8601 STRING (serde_json::Value::String). \
         Got: {:?}. BC-2.01.013 EC-01-033.",
        gte_operand["value"]
    );
    let gte_dt = chrono::DateTime::parse_from_rfc3339(gte_value_str.unwrap())
        .expect("RG-003: gte operand value must parse as RFC3339");
    let start_secs = chrono::DateTime::parse_from_rfc3339(start_time)
        .expect("test fixture: start_time must be valid RFC3339")
        .timestamp();
    assert!(
        (gte_dt.timestamp() - start_secs).abs() <= 60,
        "RG-003: greater_or_equal value must match start_time ({start_time} = {start_secs}s). \
         Got {}s (delta: {}s). BC-2.01.013 EC-01-033.",
        gte_dt.timestamp(),
        gte_dt.timestamp() - start_secs
    );

    // Validate the upper-bound value is an ISO-8601 string matching end_time.
    let lte_operand = operands
        .iter()
        .find(|c| c["operation"].as_str() == Some("less_or_equal"))
        .expect("lte operand must exist");
    let lte_value_str = lte_operand["value"].as_str();
    assert!(
        lte_value_str.is_some(),
        "RG-003: less_or_equal value must be an ISO-8601 STRING (serde_json::Value::String). \
         Got: {:?}. BC-2.01.013 EC-01-033.",
        lte_operand["value"]
    );
    let lte_dt = chrono::DateTime::parse_from_rfc3339(lte_value_str.unwrap())
        .expect("RG-003: lte operand value must parse as RFC3339");
    let end_secs = chrono::DateTime::parse_from_rfc3339(end_time)
        .expect("test fixture: end_time must be valid RFC3339")
        .timestamp();
    assert!(
        (lte_dt.timestamp() - end_secs).abs() <= 60,
        "RG-003: less_or_equal value must match end_time ({end_time} = {end_secs}s). \
         Got {}s (delta: {}s). BC-2.01.013 EC-01-033.",
        lte_dt.timestamp(),
        lte_dt.timestamp() - end_secs
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

/// AC-006 / BC-2.01.013 §Postcondition 4:
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
/// Story: S-CLAROTY-AUDITLOG-TIMEBOX-001 AC-006 / RG-005.
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
         BC-2.01.013 §Postcondition 4; S-CLAROTY-AUDITLOG-TIMEBOX-001 AC-006."
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

// ---------------------------------------------------------------------------
// RG-006: BC-2.01.013 EC-01-032 — end-only filter, NO synthetic lower bound
//
// AC-007: When end_time is Some and start_time is None, the POST body MUST
// contain a SINGLE `less_or_equal` at the supplied end. NO compound `and`
// filter, NO `greater_or_equal` synthetic lower bound.
//
// Uses end_time = "2026-01-01T00:00:00Z" (>7 months before 2026-08-15) to
// prove no 7-day floor is silently injected. Adding a 7-day floor when
// end_time < now-7d produces an inverted/empty window — SOUL.md §4.
//
// CURRENT FAILURE: body = {"offset": 0, "limit": 1000} — no filter_by at all.
// ---------------------------------------------------------------------------

/// AC-007 / BC-2.01.013 EC-01-032:
/// When `start_time = None` and `end_time = Some(past_date_older_than_7d)`, the
/// POST body carries a SINGLE `less_or_equal` filter at `end_time`. No compound
/// `and` filter is produced. No synthetic 7-day lower bound is injected.
///
/// Uses `end_time = "2026-01-01T00:00:00Z"` (≈7.5 months ago) to prove that
/// the 7-day default floor is NOT applied when an explicit end_time is given.
///
/// # Red Gate Failure
///
/// `body.get("filter_by").is_some()` FAILS because current body is
/// `{"offset": 0, "limit": 1000}` — no filter at all.
///
/// BCs: BC-2.01.013 EC-01-032; BC-2.16.013
/// Story: S-CLAROTY-AUDITLOG-TIMEBOX-001 AC-007 / RG-006.
#[tokio::test]
async fn test_BC_2_01_013_claroty_audit_logs_layer2_end_only_single_less_or_equal() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/v1/audit_log/get"))
        .respond_with(ResponseTemplate::new(200).set_body_json(audit_log_response_one_record()))
        .mount(&mock_server)
        .await;

    let adapter = make_claroty_adapter(&mock_server.uri());
    let adapter_spec = make_audit_log_adapter_spec();

    // end_time = 2026-01-01 (≈7.5 months ago from 2026-08-15), well past the 7-day default
    // window. This proves the implementation does NOT inject a synthetic now-7d lower bound.
    // If a 7-day floor were added, the window would be inverted (end_time < floor), yielding
    // an empty result set — a SOUL.md §4 silent-wrong-result violation.
    let end_time = "2026-01-01T00:00:00Z";
    let params = QueryParams {
        cursor: None,
        limit: 10,
        start_time: None, // Explicitly None — end-only filter case (EC-01-032).
        end_time: Some(end_time.to_string()),
        filters: Default::default(),
    };

    let sensor_auth = BearerStaticSensorAuth::new("claroty-layer2-rg006-token");

    let _result = adapter.fetch(&adapter_spec, &params, &sensor_auth).await;

    let requests = mock_server.received_requests().await.unwrap_or_default();
    assert!(
        !requests.is_empty(),
        "RG-006: SpecDrivenSensorAdapter::fetch must have issued a POST to \
         /api/v1/audit_log/get."
    );

    let body = parse_received_body(&requests[0].body);

    // LOAD-BEARING Red Gate assertion (RG-006):
    // The POST body MUST contain 'filter_by' when end_time is provided.
    // FAILS BEFORE IMPLEMENTATION: body = {"offset": 0, "limit": 1000}
    assert!(
        body.get("filter_by").is_some(),
        "RG-006 LOAD-BEARING: POST body to xDome /api/v1/audit_log/get MUST contain \
         'filter_by' when QueryParams.end_time = Some(\"{end_time}\"). \
         Got body: {body}. \
         Root cause: spec_driven_adapter.rs does not inject _claroty_audit_filter_by for \
         the end-only case. BC-2.01.013 EC-01-032; S-CLAROTY-AUDITLOG-TIMEBOX-001 AC-007."
    );

    let filter_by = &body["filter_by"];

    // The filter MUST be a single less_or_equal — NOT compound.
    // A compound "operation": "and" would mean a synthetic lower bound was injected.
    assert_eq!(
        filter_by["operation"].as_str().unwrap_or(""),
        "less_or_equal",
        "RG-006: end-only filter MUST be a single 'less_or_equal'. \
         Got operation: {:?}. \
         If 'and', the implementation added a synthetic lower bound — FORBIDDEN by \
         BC-2.01.013 EC-01-032 (produces inverted empty window when end < now-7d). \
         BC-2.01.013 EC-01-032; S-CLAROTY-AUDITLOG-TIMEBOX-001 AC-007.",
        filter_by["operation"]
    );

    // Confirm the compound "and" key is absent.
    assert_ne!(
        filter_by["operation"].as_str().unwrap_or(""),
        "and",
        "RG-006: filter_by.operation MUST NOT be 'and' for end-only filter. \
         A compound 'and' indicates a synthetic lower bound was injected. \
         FORBIDDEN by BC-2.01.013 EC-01-032. S-CLAROTY-AUDITLOG-TIMEBOX-001 AC-007."
    );

    // Confirm NO greater_or_equal anywhere in the filter object.
    // Serialize the full filter_by to catch nested occurrences.
    let filter_by_str = serde_json::to_string(filter_by).unwrap_or_default();
    assert!(
        !filter_by_str.contains("\"greater_or_equal\""),
        "RG-006: filter_by MUST NOT contain 'greater_or_equal' for the end-only case. \
         Got filter_by: {filter_by}. \
         BC-2.01.013 EC-01-032: end-only means single less_or_equal, NO synthetic floor. \
         Adding a 7-day lower bound silently produces an inverted/empty window when \
         end_time ({end_time}) < now-7d. SOUL.md §4 silent-wrong-result violation. \
         S-CLAROTY-AUDITLOG-TIMEBOX-001 AC-007."
    );

    // Field must be "timestamp".
    assert_eq!(
        filter_by["field"].as_str().unwrap_or(""),
        "timestamp",
        "RG-006: filter_by.field must be 'timestamp'. Got: {:?}. \
         BC-2.01.013 EC-01-032.",
        filter_by["field"]
    );

    // Value must be an ISO-8601 string equal to end_time.
    // BC-2.01.013 EC-01-032: value is an ISO-8601 STRING, NOT an epoch integer.
    let value_str = filter_by["value"].as_str();
    assert!(
        value_str.is_some(),
        "RG-006: filter_by.value must be an ISO-8601 STRING (serde_json::Value::String). \
         Got: {:?}. BC-2.01.013 EC-01-032.",
        filter_by["value"]
    );
    let value_dt = chrono::DateTime::parse_from_rfc3339(value_str.unwrap());
    assert!(
        value_dt.is_ok(),
        "RG-006: filter_by.value must parse as RFC3339. Got: {:?}. Error: {:?}. \
         BC-2.01.013 EC-01-032.",
        value_str,
        value_dt.err()
    );
    let value_secs = value_dt.unwrap().timestamp();
    let end_secs = chrono::DateTime::parse_from_rfc3339(end_time)
        .expect("test fixture: end_time must be valid RFC3339")
        .timestamp();
    assert!(
        (value_secs - end_secs).abs() <= 60,
        "RG-006: filter_by.value must match end_time ({end_time} = {end_secs}s). \
         Got {value_secs}s (delta: {}s). BC-2.01.013 EC-01-032; AC-007.",
        value_secs - end_secs
    );
}

// ---------------------------------------------------------------------------
// RG-007: AC-INDEX-CLARO-001 / BC-2.01.013 EC-01-034 (SAP-3 reachability)
//
// Parser-surface (SAP-3) test: a PrismQL `WHERE timestamp > '<older-than-7d ISO>'`
// predicate MUST produce an explicit lower-bound in the xDome filter_by POST body —
// NOT the 7-day default.
//
// The defect: `audit_logs.timestamp` lacks `options = ["INDEX"]` in claroty.sensor.toml.
// `extract_time_window_from_ast` gates extraction on `col.options.contains(ColumnOptions::Index)`.
// Without INDEX, the WHERE predicate is treated as non-eligible, start_time stays None,
// and `build_claroty_audit_filter_by(None, None)` silently injects the 7-day default —
// discarding the user's explicit filter (SOUL.md §4 silent-wrong-result; EC-01-034).
//
// SAP-3 compliance: the test starts from a PrismQL query STRING, runs through
// `PrismQlParser::parse` → `extract_time_window_from_ast` → `QueryParams.start_time` →
// `build_claroty_audit_filter_by` (via adapter.fetch) → wire-level assertion on POST body.
// NOT a synthetic QueryParams unit test.
//
// EXPECTED FAIL (RED): filter_by.value = <now-7d>, NOT "2025-01-01T00:00:00Z".
// EXPECTED GREEN after implementer adds `options = ["INDEX"]` to claroty.sensor.toml §audit_logs
// timestamp column (Task 5 / AC-INDEX-CLARO-001).
//
// BCs: BC-2.01.013 EC-01-034; BC-2.16.013
// Story: S-CLAROTY-AUDITLOG-TIMEBOX-001 AC-INDEX-CLARO-001 / RG-007
// ---------------------------------------------------------------------------

/// AC-INDEX-CLARO-001 / BC-2.01.013 EC-01-034 — SAP-3 parser-surface reachability test.
///
/// Proves that `audit_logs.timestamp` must have `options = ["INDEX"]` for an explicit
/// `WHERE timestamp > '<older-than-7d>'` predicate to reach the xDome POST body.
///
/// # Production call graph exercised (SAP-3)
///
/// ```text
/// PrismQL string: "SELECT * FROM claroty_audit_logs WHERE timestamp > '2025-01-01T00:00:00Z'"
///   → PrismQlParser::parse → Ast::Sql(Select { where_: Some(predicate) })
///   → extract_time_window_from_ast [reads col_map, timestamp lacking INDEX → returns (None, None)]
///   → QueryParams { start_time: None, end_time: None }
///   → SpecDrivenSensorAdapter::fetch (sensor_id=claroty, table=audit_logs)
///       → build_claroty_audit_filter_by(None, None)
///       → filter_by = {"field": "timestamp", "operation": "greater_or_equal", "value": "<now-7d>"}
///   → POST /api/v1/audit_log/get → wiremock captures body
/// ```
///
/// # Red Gate Failure (expected until implementer adds `options = ["INDEX"]`)
///
/// `extract_time_window_from_ast` returns `(None, None)` because `timestamp` in
/// `claroty.sensor.toml §audit_logs` lacks `options = ["INDEX"]`.
/// `build_claroty_audit_filter_by(None, None)` injects the 7-day default.
/// The assertion `filter_by.value ≈ 2025-01-01T00:00:00Z` FAILS — the value is
/// `<now−604800s>`, not the user-supplied explicit bound.
///
/// EC-01-031/032/033 are all unreachable from the parser surface until this is fixed.
///
/// # Fix
///
/// Add `options = ["INDEX"]` to `audit_logs.timestamp` in `claroty.sensor.toml`.
/// After the fix, `extract_time_window_from_ast` extracts `start_time = Some("2025-01-01T00:00:00+00:00")`
/// and this assertion turns GREEN.
///
/// BCs: BC-2.01.013 EC-01-034; BC-2.16.013
/// Story: S-CLAROTY-AUDITLOG-TIMEBOX-001 AC-INDEX-CLARO-001 / RG-007
#[tokio::test]
async fn test_BC_2_01_013_claroty_audit_logs_timestamp_index_option_required_for_pushdown_eligibility()
 {
    // Step 1: Parse the PrismQL query string — SAP-3 parser-surface entry point.
    //
    // A real user would issue this query to retrieve audit logs from 2025-01-01 onward.
    // The query is > 7 months older than the 7-day default window to make the defect
    // visible: if the default is injected instead of the explicit bound, the filter
    // targets a completely different time range.
    let pql_query = "SELECT * FROM claroty_audit_logs WHERE timestamp > '2025-01-01T00:00:00Z'";
    let ast = prism_query::PrismQlParser::parse(pql_query).unwrap_or_else(|errs| {
        panic!(
            "RG-007: PrismQL query must parse. Errors: {:?}. Query: {:?}",
            errs, pql_query
        )
    });

    // Step 2: Extract the WHERE predicate from the parsed AST.
    let predicate = match &ast {
        prism_query::ast::Ast::Sql(prism_query::ast::SqlStatement::Select(sql)) => sql
            .where_
            .as_ref()
            .expect("RG-007: query must have a WHERE clause (timestamp > '2025-01-01T00:00:00Z')"),
        other => panic!(
            "RG-007: expected Ast::Sql(Select), got a different AST variant. Query: {:?}. Got: {:?}",
            pql_query, other
        ),
    };

    // Step 3: Load production claroty.sensor.toml (SAP-2: no fabricated fixture).
    //
    // This is the CURRENT disk state. The audit_logs.timestamp column lacks
    // `options = ["INDEX"]`, so `extract_time_window_from_ast` will not recognize
    // it as an eligible push-down column.
    let spec_content = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../prism-sensors/specs/claroty.sensor.toml"),
    )
    .expect(
        "RG-007: claroty.sensor.toml must be readable from \
         CARGO_MANIFEST_DIR/../prism-sensors/specs/",
    );

    let spec = SpecLoader::parse(&spec_content)
        .expect("RG-007: claroty.sensor.toml must parse cleanly via SpecLoader::parse");

    // Step 4: Build the column map for `extract_time_window_from_ast`.
    //
    // The function expects `HashMap<String, Vec<ColumnSpec>>` where keys are the
    // source names used in PrismQL queries. Both "claroty_audit_logs" (underscore form,
    // used in FROM clause) and "claroty.audit_logs" (dot form) are populated, matching
    // the key format that `build_source_column_map` in materialization.rs produces.
    let audit_logs_table = spec
        .tables
        .iter()
        .find(|t| t.table_name == "audit_logs")
        .expect("RG-007: claroty.sensor.toml must contain an audit_logs table");

    let mut col_map: HashMap<String, Vec<prism_spec_engine::spec_parser::ColumnSpec>> =
        HashMap::new();
    col_map.insert(
        "claroty_audit_logs".to_string(),
        audit_logs_table.columns.clone(),
    );
    col_map.insert(
        "claroty.audit_logs".to_string(),
        audit_logs_table.columns.clone(),
    );

    // Step 5: Call extract_time_window_from_ast — the ADR-033 T1 extraction path.
    //
    // SAP-3: the predicate was produced by the real parser in Step 1.
    // With the current TOML (timestamp lacks `options = ["INDEX"]`):
    //   - extract_time_window_from_ast returns (None, None)
    //   - start_time = None, end_time = None
    // After the fix (Task 5 adds `options = ["INDEX"]`):
    //   - extract_time_window_from_ast returns (Some("2025-01-01T00:00:00+00:00"), None)
    //   - start_time = Some("2025-01-01T00:00:00+00:00"), end_time = None
    let (extracted_start_time, extracted_end_time) =
        prism_query::pushdown::extract_time_window_from_ast(
            predicate,
            &["claroty_audit_logs"],
            Some(&col_map),
        );

    // Step 6: Set up a wiremock server to capture the outbound xDome POST body.
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/v1/audit_log/get"))
        .respond_with(ResponseTemplate::new(200).set_body_json(audit_log_response_one_record()))
        .mount(&mock_server)
        .await;

    // Step 7: Invoke the Claroty adapter with the extracted QueryParams.
    //
    // The adapter's fetch() calls build_claroty_audit_filter_by(start_time, end_time)
    // internally when sensor_id == "claroty" && source_table == "claroty_audit_logs".
    // Passing the extracted (None, None) causes the 7-day default to be injected.
    let adapter = make_claroty_adapter(&mock_server.uri());
    let adapter_spec = make_audit_log_adapter_spec();

    let params = QueryParams {
        cursor: None,
        limit: 10,
        start_time: extracted_start_time.clone(),
        end_time: extracted_end_time.clone(),
        filters: Default::default(),
    };

    let sensor_auth = BearerStaticSensorAuth::new("claroty-rg007-index-test-token");
    let _result = adapter.fetch(&adapter_spec, &params, &sensor_auth).await;

    // Step 8: Assert on the wire-level POST body.
    let requests = mock_server.received_requests().await.unwrap_or_default();
    assert!(
        !requests.is_empty(),
        "RG-007: SpecDrivenSensorAdapter::fetch must issue a POST to \
         /api/v1/audit_log/get. No requests received. \
         Check that source_table=\"claroty_audit_logs\" routes to the audit_logs \
         table in claroty.sensor.toml."
    );

    let body = parse_received_body(&requests[0].body);

    // filter_by must be present (this is also a prerequisite).
    assert!(
        body.get("filter_by").is_some(),
        "RG-007 prerequisite: POST body to xDome /api/v1/audit_log/get must contain \
         'filter_by'. Got body: {body}. \
         This would only fail if the spec_driven_adapter injection block is missing. \
         Check spec_driven_adapter.rs §Claroty audit_logs filter_by injection."
    );

    let filter_by = &body["filter_by"];

    // The filter_by.value must be an ISO-8601 string.
    let value_str = filter_by["value"].as_str().unwrap_or("");
    let filter_dt_result = chrono::DateTime::parse_from_rfc3339(value_str);
    assert!(
        filter_dt_result.is_ok(),
        "RG-007: filter_by.value must be a parseable ISO-8601 RFC3339 string. \
         Got: {:?}. filter_by: {}. BC-2.01.013 EC-01-034.",
        value_str,
        filter_by
    );

    let filter_secs = filter_dt_result.unwrap().timestamp();
    let explicit_bound_secs = chrono::DateTime::parse_from_rfc3339("2025-01-01T00:00:00Z")
        .expect("test fixture: explicit bound must be valid RFC3339")
        .timestamp();
    let seven_day_default_secs = chrono::Utc::now().timestamp() - 7_i64 * 24 * 3600;

    // LOAD-BEARING Red Gate assertion (RG-007, SAP-3 reachability):
    //
    // filter_by.value MUST equal the EXPLICIT user-supplied bound (2025-01-01T00:00:00Z),
    // NOT the 7-day default (≈ now − 604,800 s).
    //
    // CURRENT FAILURE (RED state):
    //   extract_time_window_from_ast returns (None, None) because audit_logs.timestamp
    //   lacks `options = ["INDEX"]` in claroty.sensor.toml.
    //   build_claroty_audit_filter_by(None, None) injects the 7-day default.
    //   filter_by.value ≈ now−604800s ≠ 2025-01-01T00:00:00Z → assertion FAILS.
    //
    // AFTER FIX (GREEN state):
    //   Task 5 adds `options = ["INDEX"]` to audit_logs.timestamp in claroty.sensor.toml.
    //   extract_time_window_from_ast returns (Some("2025-01-01T00:00:00+00:00"), None).
    //   build_claroty_audit_filter_by(Some(...), None) returns the explicit bound.
    //   filter_by.value ≈ 2025-01-01T00:00:00Z → assertion PASSES.
    //
    // Diagnostic context:
    //   extracted_start_time = {:?} (None until INDEX is added)
    //   filter_by.value = '{}' = {}s
    //   explicit_bound = 2025-01-01T00:00:00Z = {}s
    //   7-day default ≈ {}s
    assert!(
        (filter_secs - explicit_bound_secs).abs() <= 60,
        "RG-007 LOAD-BEARING (SAP-3): filter_by.value MUST equal the EXPLICIT user-supplied \
         bound (2025-01-01T00:00:00Z = {explicit_bound_secs}s), NOT the 7-day default \
         (~{seven_day_default_secs}s).\n\
         Got filter_by.value = '{value_str}' = {filter_secs}s (delta from explicit: {}s).\n\
         \n\
         Root cause: claroty.sensor.toml audit_logs.timestamp column lacks `options = [\"INDEX\"]`.\n\
         Without INDEX, `extract_time_window_from_ast` (ADR-033 T1) does not treat `timestamp` \
         as a push-down-eligible column.\n\
         `extract_time_window_from_ast` returned: start_time={:?}, end_time={:?}.\n\
         `build_claroty_audit_filter_by(None, None)` injects the 7-day default, silently \
         discarding the user's WHERE timestamp > '2025-01-01T00:00:00Z' predicate.\n\
         This is a SOUL.md §4 silent-wrong-result violation (EC-01-034): a query for \
         audit logs from 2025-01-01 receives only the last 7 days of results.\n\
         EC-01-031/032/033 are all unreachable from the parser surface (SAP-3 violation).\n\
         \n\
         Fix: add `options = [\"INDEX\"]` to the `timestamp` column in \
         claroty.sensor.toml §audit_logs (Task 5 / AC-INDEX-CLARO-001).\n\
         After the fix, `extract_time_window_from_ast` extracts start_time from the \
         WHERE clause and this assertion turns GREEN.\n\
         \n\
         BC-2.01.013 EC-01-034; BC-2.16.013; \
         S-CLAROTY-AUDITLOG-TIMEBOX-001 AC-INDEX-CLARO-001 / RG-007.",
        filter_secs - explicit_bound_secs,
        extracted_start_time,
        extracted_end_time
    );

    // Confirm filter_by.value is NOT the 7-day default (sanity check of the negative condition).
    let delta_from_7day = (filter_secs - seven_day_default_secs).abs();
    // After the fix, the explicit 2025-01-01 bound is ~7.5 months before 2026-08-15.
    // The 7-day default is ≈ 7 days before now. They should be far apart.
    // (This secondary assertion is not the load-bearing one — just provides diagnostic clarity.)
    if (filter_secs - explicit_bound_secs).abs() <= 60 {
        // RG-007 passed: the explicit bound was extracted. No need to check the negative.
    } else {
        // RG-007 failed: the 7-day default was injected. Confirm that's what happened.
        assert!(
            delta_from_7day <= 120,
            "RG-007 diagnostic: expected the 7-day default (~{seven_day_default_secs}s) \
             to be injected when INDEX is missing, but got filter_by.value = {filter_secs}s \
             which is neither the explicit bound ({explicit_bound_secs}s) nor the 7-day default. \
             Unexpected value — check build_claroty_audit_filter_by. BC-2.01.013 EC-01-034."
        );
    }
}
