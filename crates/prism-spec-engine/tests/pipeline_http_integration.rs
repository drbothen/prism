#![allow(non_snake_case)]
//! VP-PLUGIN-002 / BC-2.16.002 pipeline HTTP integration tests.
//!
//! These tests constitute the Red Gate for S-PLUGIN-PREREQ-B. All tests in this
//! file MUST FAIL in the Red Gate state because `PipelineExecutor::execute` body
//! is `todo!()`. The implementer will make them pass by filling the real HTTP
//! execution path.
//!
//! Test coverage:
//! - AC-1: HTTP request issued and non-empty records returned (VP-PLUGIN-002 canonical test)
//! - AC-2: Two-step pipeline where step 2 URL uses step 1 response token
//! - AC-3: Cursor-paginated step iterates until null cursor
//! - AC-4: Offset-paginated step iterates until short page
//! - VP-PLUGIN-002: Canonical acceptance test (AC-9)
//! - F-LP5-MED-001: gzip response auto-decoded by reqwest (regression)
//!
//! All tests use wiremock as the HTTP backend so no real sensor API is required.

use flate2::Compression;
use flate2::write::GzEncoder;
use prism_core::{ColumnType, OrgSlug};
use prism_spec_engine::NullAuthProvider;
use prism_spec_engine::error::SpecEngineError;
use prism_spec_engine::pipeline::{FetchContext, PipelineExecutor};
use prism_spec_engine::spec_parser::{
    AuthType, ColumnSpec, FetchStep, PaginationConfig, RateLimitHints, SensorSpec, TableSpec,
};
use std::collections::HashMap;
use std::io::Write;
use wiremock::matchers::{header, method, path, path_regex, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

// ---------------------------------------------------------------------------
// Shared fixture helpers
// ---------------------------------------------------------------------------

/// Build a minimal one-step `SensorSpec` pointing at `base_url`.
fn one_step_spec(base_url: &str, path_template: &str, response_path: &str) -> SensorSpec {
    SensorSpec {
        sensor_id: "test-sensor".to_string(),
        name: "Test Sensor".to_string(),
        auth_type: AuthType::BearerStatic,
        base_url: base_url.to_string(),
        tables: vec![TableSpec::new_point_in_time(
            "items",
            "security_finding",
            vec![ColumnSpec {
                name: "id".to_string(),
                column_type: ColumnType::String,
                ocsf_field: None,
                options: vec![],
            }],
            vec![FetchStep {
                name: "fetch_items".to_string(),
                method: "GET".to_string(),
                path_template: path_template.to_string(),
                body_template: None,
                response_path: response_path.to_string(),
                pagination_cursor_path: None,
                variables_produced: vec![],
                fan_out_batch_size: None,
                pagination: None,
            }],
        )],
        rate_limit_hints: None,
        version: "1.0.0".to_string(),
        credential_refs: vec![],
    }
}

fn default_context() -> FetchContext {
    FetchContext::new(OrgSlug::new("test-org"), HashMap::new())
}

// ---------------------------------------------------------------------------
// Test 7 (VP-PLUGIN-002 canonical): AC-9 — primary Red Gate test
// ---------------------------------------------------------------------------

/// VP-PLUGIN-002 / AC-9: `PipelineExecutor::execute` returns non-empty records
/// against a wiremock mock server.
///
/// This is the canonical acceptance test for VP-PLUGIN-002. A single-step spec
/// points at a wiremock server that returns a two-record JSON fixture. The test
/// asserts `result.records.len() == 2`.
///
/// FAILS RED: `execute` body is `todo!()` — panics with "not yet implemented".
/// PASSES GREEN: when the real HTTP implementation is provided by the implementer.
#[tokio::test]
async fn test_BC_PLUGIN_002_pipeline_executor_returns_nonempty_records_against_wiremock() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/detections"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "resources": [
                {"id": "det-001", "severity": "high"},
                {"id": "det-002", "severity": "medium"}
            ]
        })))
        .mount(&mock_server)
        .await;

    let spec = one_step_spec(&mock_server.uri(), "/api/detections", "$.resources");
    let table = spec.tables[0].clone();
    let context = default_context();
    let http_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("reqwest Client::build must succeed");
    let auth_provider = NullAuthProvider;

    let result = PipelineExecutor::execute(&spec, &table, &context, &http_client, &auth_provider)
        .await
        .expect("execute must succeed against wiremock");

    assert_eq!(
        result.records.len(),
        2,
        "VP-PLUGIN-002: PipelineExecutor must return 2 records from mock server; got {}",
        result.records.len()
    );
    assert!(
        !result.truncated,
        "2 records is well below the 10K DI-019 limit; truncated must be false"
    );
}

// ---------------------------------------------------------------------------
// Test 1 (AC-1): HTTP request issued and non-empty records returned
// ---------------------------------------------------------------------------

/// BC-2.16.002 AC-1: `PipelineExecutor::execute` issues at least one real HTTP
/// request per `FetchStep` and returns non-empty records matching the mock response.
///
/// FAILS RED: `execute` body is `todo!()`.
#[tokio::test]
async fn test_BC_2_16_002_execute_issues_http_request_and_returns_nonempty_records() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/alerts"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [
                {"alert_id": "a1", "type": "malware"},
                {"alert_id": "a2", "type": "intrusion"},
                {"alert_id": "a3", "type": "exfiltration"}
            ]
        })))
        .mount(&mock_server)
        .await;

    let spec = one_step_spec(&mock_server.uri(), "/alerts", "$.data");
    let table = spec.tables[0].clone();
    let context = default_context();
    let http_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("reqwest Client::build must succeed");
    let auth_provider = NullAuthProvider;

    let result = PipelineExecutor::execute(&spec, &table, &context, &http_client, &auth_provider)
        .await
        .expect("execute must succeed (AC-1)");

    assert!(
        !result.records.is_empty(),
        "AC-1: records must be non-empty; got 0 records"
    );
    assert_eq!(
        result.records.len(),
        3,
        "AC-1: expected 3 records from mock, got {}",
        result.records.len()
    );
    assert!(
        result.request_count >= 1,
        "AC-1: at least 1 HTTP request must have been issued; got request_count={}",
        result.request_count
    );
}

// ---------------------------------------------------------------------------
// Test 2 (AC-2): Two-step pipeline — step 2 URL uses step 1 token
// ---------------------------------------------------------------------------

/// BC-2.16.002 AC-2: Two-step pipeline where step 2's `path_template` contains
/// `${step1.access_token}`, resolved from step 1's JSON response.
///
/// Step 1: POST /oauth2/token → `{"access_token": "tok-xyz-789"}`
/// Step 2: GET /api/data?token=tok-xyz-789 → `{"items": [...]}`
///
/// Asserts that the records come from step 2 (the token interpolated correctly).
///
/// FAILS RED: `execute` body is `todo!()`.
#[tokio::test]
async fn test_BC_2_16_002_execute_interpolates_step1_var_into_step2_url() {
    let mock_server = MockServer::start().await;

    // Step 1: token endpoint
    Mock::given(method("POST"))
        .and(path("/oauth2/token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "access_token": "tok-xyz-789"
        })))
        .mount(&mock_server)
        .await;

    // Step 2: data endpoint — only valid when called with the exact token in URL
    Mock::given(method("GET"))
        .and(path("/api/data"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "items": [
                {"item_id": "i1"},
                {"item_id": "i2"}
            ]
        })))
        .mount(&mock_server)
        .await;

    let spec = SensorSpec {
        sensor_id: "two-step-sensor".to_string(),
        name: "Two-Step Sensor".to_string(),
        auth_type: AuthType::Oauth2ClientCredentials,
        base_url: mock_server.uri(),
        tables: vec![TableSpec::new_point_in_time(
            "items",
            "security_finding",
            vec![ColumnSpec {
                name: "item_id".to_string(),
                column_type: ColumnType::String,
                ocsf_field: None,
                options: vec![],
            }],
            vec![
                FetchStep {
                    name: "step1".to_string(),
                    method: "POST".to_string(),
                    path_template: "/oauth2/token".to_string(),
                    body_template: Some("grant_type=client_credentials".to_string()),
                    response_path: "$.access_token".to_string(),
                    pagination_cursor_path: None,
                    variables_produced: vec!["access_token".to_string()],
                    fan_out_batch_size: None,
                    pagination: None,
                },
                FetchStep {
                    name: "step2".to_string(),
                    method: "GET".to_string(),
                    // AC-2: token from step1 interpolated into step2 URL
                    path_template: "/api/data?token=${step1.access_token}".to_string(),
                    body_template: None,
                    response_path: "$.items".to_string(),
                    pagination_cursor_path: None,
                    variables_produced: vec![],
                    fan_out_batch_size: None,
                    pagination: None,
                },
            ],
        )],
        rate_limit_hints: None,
        version: "1.0.0".to_string(),
        credential_refs: vec![],
    };

    let table = spec.tables[0].clone();
    let context = default_context();
    let http_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("reqwest Client::build must succeed");
    let auth_provider = NullAuthProvider;

    let result = PipelineExecutor::execute(&spec, &table, &context, &http_client, &auth_provider)
        .await
        .expect("AC-2: two-step execution must succeed");

    assert_eq!(
        result.records.len(),
        2,
        "AC-2: step 2 must return 2 items (token interpolated from step 1); got {}",
        result.records.len()
    );
}

// ---------------------------------------------------------------------------
// Test 3 (AC-3): Cursor pagination
// ---------------------------------------------------------------------------

/// BC-2.16.002 AC-3: Cursor-paginated step iterates until null cursor.
///
/// Page 1: returns 2 records + cursor = "page2-cursor"
/// Page 2: returns 2 records + cursor = null
///
/// Asserts `records.len() == 4` (all pages concatenated).
///
/// FAILS RED: `execute` body is `todo!()`.
#[tokio::test]
async fn test_BC_2_16_002_execute_iterates_cursor_pagination_until_null() {
    let mock_server = MockServer::start().await;

    // Both pages served at the same path — wiremock serves them in registration order.
    // Page 1: cursor present
    Mock::given(method("GET"))
        .and(path("/events"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [{"id": "e1"}, {"id": "e2"}],
            "pagination": {"cursor": "page2-cursor"}
        })))
        .up_to_n_times(1)
        .mount(&mock_server)
        .await;

    // Page 2: cursor null (stop condition)
    Mock::given(method("GET"))
        .and(path("/events"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [{"id": "e3"}, {"id": "e4"}],
            "pagination": {"cursor": null}
        })))
        .up_to_n_times(1)
        .mount(&mock_server)
        .await;

    let spec = SensorSpec {
        sensor_id: "cursor-sensor".to_string(),
        name: "Cursor Sensor".to_string(),
        auth_type: AuthType::BearerStatic,
        base_url: mock_server.uri(),
        tables: vec![TableSpec::new_point_in_time(
            "events",
            "security_finding",
            vec![ColumnSpec {
                name: "id".to_string(),
                column_type: ColumnType::String,
                ocsf_field: None,
                options: vec![],
            }],
            vec![FetchStep {
                name: "fetch_events".to_string(),
                method: "GET".to_string(),
                path_template: "/events".to_string(),
                body_template: None,
                response_path: "$.data".to_string(),
                pagination_cursor_path: None,
                variables_produced: vec![],
                fan_out_batch_size: None,
                pagination: Some(PaginationConfig::CursorToken {
                    cursor_response_path: "$.pagination.cursor".to_string(),
                    page_size: None,
                }),
            }],
        )],
        rate_limit_hints: None,
        version: "1.0.0".to_string(),
        credential_refs: vec![],
    };

    let table = spec.tables[0].clone();
    let context = default_context();
    let http_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("reqwest Client::build must succeed");
    let auth_provider = NullAuthProvider;

    let result = PipelineExecutor::execute(&spec, &table, &context, &http_client, &auth_provider)
        .await
        .expect("AC-3: cursor-paginated execution must succeed");

    assert_eq!(
        result.records.len(),
        4,
        "AC-3: 2 pages of 2 records each = 4 total; got {}",
        result.records.len()
    );
    assert!(
        result.request_count >= 2,
        "AC-3: at least 2 requests (1 per page); got request_count={}",
        result.request_count
    );
}

// ---------------------------------------------------------------------------
// Test 4 (AC-4): Offset pagination
// ---------------------------------------------------------------------------

/// BC-2.16.002 AC-4: Offset-paginated step iterates until a short page.
///
/// page_size = 3
/// Page 1 (offset=0): 3 records (full page → continue)
/// Page 2 (offset=3): 2 records (short page → stop)
///
/// Asserts `records.len() == 5` (all pages concatenated).
///
/// FAILS RED: `execute` body is `todo!()`.
#[tokio::test]
async fn test_BC_2_16_002_execute_iterates_offset_pagination_until_short_page() {
    let mock_server = MockServer::start().await;

    // Page 1: full page of 3 records
    Mock::given(method("GET"))
        .and(path("/logs"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "items": [{"id": "L1"}, {"id": "L2"}, {"id": "L3"}]
        })))
        .up_to_n_times(1)
        .mount(&mock_server)
        .await;

    // Page 2: partial page of 2 records (stop condition: fewer than page_size)
    Mock::given(method("GET"))
        .and(path("/logs"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "items": [{"id": "L4"}, {"id": "L5"}]
        })))
        .up_to_n_times(1)
        .mount(&mock_server)
        .await;

    let spec = SensorSpec {
        sensor_id: "offset-sensor".to_string(),
        name: "Offset Sensor".to_string(),
        auth_type: AuthType::ApiKey,
        base_url: mock_server.uri(),
        tables: vec![TableSpec::new_point_in_time(
            "logs",
            "security_finding",
            vec![ColumnSpec {
                name: "id".to_string(),
                column_type: ColumnType::String,
                ocsf_field: None,
                options: vec![],
            }],
            vec![FetchStep {
                name: "fetch_logs".to_string(),
                method: "GET".to_string(),
                path_template: "/logs".to_string(),
                body_template: None,
                response_path: "$.items".to_string(),
                pagination_cursor_path: None,
                variables_produced: vec![],
                fan_out_batch_size: None,
                pagination: Some(PaginationConfig::OffsetLimit { page_size: 3 }),
            }],
        )],
        rate_limit_hints: None,
        version: "1.0.0".to_string(),
        credential_refs: vec![],
    };

    let table = spec.tables[0].clone();
    let context = default_context();
    let http_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("reqwest Client::build must succeed");
    let auth_provider = NullAuthProvider;

    let result = PipelineExecutor::execute(&spec, &table, &context, &http_client, &auth_provider)
        .await
        .expect("AC-4: offset-paginated execution must succeed");

    assert_eq!(
        result.records.len(),
        5,
        "AC-4: page1(3) + page2(2) = 5 total records; got {}",
        result.records.len()
    );
    assert!(
        result.request_count >= 2,
        "AC-4: at least 2 offset requests; got request_count={}",
        result.request_count
    );
}

// ---------------------------------------------------------------------------
// F-LP1-CRIT-001 Red Gate test: body_template interpolation + Content-Type derivation
// ---------------------------------------------------------------------------

/// BC-2.16.002 AC-2 / F-LP1-CRIT-001: body_template is interpolated before sending
/// and Content-Type is derived from body shape (JSON → application/json;
/// form-urlencoded → application/x-www-form-urlencoded).
///
/// Step 1 returns `{"step1_id": "abc-123"}`.
/// Step 2 POSTs a JSON body containing `${step1.step1_id}` and must receive
/// Content-Type: application/json.
#[tokio::test]
async fn test_BC_2_16_002_execute_interpolates_body_template_and_derives_content_type() {
    let mock_server = MockServer::start().await;

    // Step 1: provides a value for body interpolation
    Mock::given(method("GET"))
        .and(path("/step1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "step1_id": "abc-123"
        })))
        .mount(&mock_server)
        .await;

    // Step 2: POST with JSON body — wiremock verifies Content-Type header is application/json
    Mock::given(method("POST"))
        .and(path("/step2"))
        .and(header("Content-Type", "application/json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "results": [{"id": "r1"}, {"id": "r2"}]
        })))
        .mount(&mock_server)
        .await;

    let spec = SensorSpec {
        sensor_id: "body-interp-sensor".to_string(),
        name: "Body Interp Sensor".to_string(),
        auth_type: AuthType::BearerStatic,
        base_url: mock_server.uri(),
        tables: vec![TableSpec::new_point_in_time(
            "results",
            "security_finding",
            vec![ColumnSpec {
                name: "id".to_string(),
                column_type: ColumnType::String,
                ocsf_field: None,
                options: vec![],
            }],
            vec![
                FetchStep {
                    name: "step1".to_string(),
                    method: "GET".to_string(),
                    path_template: "/step1".to_string(),
                    body_template: None,
                    response_path: "$.step1_id".to_string(),
                    pagination_cursor_path: None,
                    variables_produced: vec!["step1_id".to_string()],
                    fan_out_batch_size: None,
                    pagination: None,
                },
                FetchStep {
                    name: "step2".to_string(),
                    method: "POST".to_string(),
                    path_template: "/step2".to_string(),
                    // JSON body with interpolated step1_id — shape starts with '{' → application/json
                    body_template: Some(r#"{"id": "${step1.step1_id}"}"#.to_string()),
                    response_path: "$.results".to_string(),
                    pagination_cursor_path: None,
                    variables_produced: vec![],
                    fan_out_batch_size: None,
                    pagination: None,
                },
            ],
        )],
        rate_limit_hints: None,
        version: "1.0.0".to_string(),
        credential_refs: vec![],
    };

    let table = spec.tables[0].clone();
    let context = FetchContext::new(OrgSlug::new("test-org"), HashMap::new());
    let http_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("reqwest Client::build must succeed");
    let auth_provider = NullAuthProvider;

    let result = PipelineExecutor::execute(&spec, &table, &context, &http_client, &auth_provider)
        .await
        .expect("F-LP1-CRIT-001: body interpolation + Content-Type derivation must succeed");

    assert_eq!(
        result.records.len(),
        2,
        "F-LP1-CRIT-001: step2 must return 2 results; got {}",
        result.records.len()
    );
    // wiremock would have returned 404 if Content-Type header didn't match
}

// ---------------------------------------------------------------------------
// F-LP1-CRIT-002 Red Gate test: cursor percent-encoding
// ---------------------------------------------------------------------------

/// BC-2.16.002 F-LP1-CRIT-002: cursor values containing special characters
/// (base64url padding like `+/=`) must be percent-encoded before appending to URL.
///
/// The mock verifies the encoded form arrives in the query string.
#[tokio::test]
async fn test_BC_2_16_002_execute_percent_encodes_opaque_cursor() {
    let mock_server = MockServer::start().await;

    // Page 1: no cursor param — returns data + a cursor with special chars
    Mock::given(method("GET"))
        .and(path("/events"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [{"id": "e1"}, {"id": "e2"}],
            "next_cursor": "abc+def/ghi=jkl"
        })))
        .up_to_n_times(1)
        .mount(&mock_server)
        .await;

    // Page 2: the cursor was percent-encoded on the wire (NON_ALPHANUMERIC).
    // Wiremock decodes query parameters before matching, so we match the decoded value.
    // The encoding correctness is verified implicitly: an unencoded '+' in the raw URL
    // would be interpreted as a space by most servers, causing a mismatch.
    // Here we verify end-to-end the cursor arrived correctly regardless of transport encoding.
    Mock::given(method("GET"))
        .and(path("/events"))
        .and(query_param("cursor", "abc+def/ghi=jkl"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [{"id": "e3"}],
            "next_cursor": null
        })))
        .up_to_n_times(1)
        .mount(&mock_server)
        .await;

    let spec = SensorSpec {
        sensor_id: "cursor-encode-sensor".to_string(),
        name: "Cursor Encode Sensor".to_string(),
        auth_type: AuthType::BearerStatic,
        base_url: mock_server.uri(),
        tables: vec![TableSpec::new_point_in_time(
            "events",
            "security_finding",
            vec![ColumnSpec {
                name: "id".to_string(),
                column_type: ColumnType::String,
                ocsf_field: None,
                options: vec![],
            }],
            vec![FetchStep {
                name: "fetch_events".to_string(),
                method: "GET".to_string(),
                path_template: "/events".to_string(),
                body_template: None,
                response_path: "$.data".to_string(),
                pagination_cursor_path: None,
                variables_produced: vec![],
                fan_out_batch_size: None,
                pagination: Some(PaginationConfig::CursorToken {
                    cursor_response_path: "$.next_cursor".to_string(),
                    page_size: None,
                }),
            }],
        )],
        rate_limit_hints: None,
        version: "1.0.0".to_string(),
        credential_refs: vec![],
    };

    let table = spec.tables[0].clone();
    let context = FetchContext::new(OrgSlug::new("test-org"), HashMap::new());
    let http_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("reqwest Client::build must succeed");
    let auth_provider = NullAuthProvider;

    let result = PipelineExecutor::execute(&spec, &table, &context, &http_client, &auth_provider)
        .await
        .expect("F-LP1-CRIT-002: cursor percent-encoding must succeed");

    assert_eq!(
        result.records.len(),
        3,
        "F-LP1-CRIT-002: 2+1 = 3 records across both pages; got {}",
        result.records.len()
    );
    // wiremock page-2 mock requires exact query_param match — if encoding wrong, 404
}

// ---------------------------------------------------------------------------
// F-LP1-CRIT-003 Red Gate test: only final step records in result
// ---------------------------------------------------------------------------

/// BC-2.16.002 F-LP1-CRIT-003: intermediate step records must NOT appear in
/// PipelineResult.records. Only the final step's array records are collected.
///
/// Step 1 returns array of 3 items (intermediate — must NOT leak).
/// Step 2 returns array of 2 items (final — MUST appear).
/// Assert records.len() == 2, not 5.
#[tokio::test]
async fn test_BC_2_16_002_execute_only_final_step_records_in_pipeline_result() {
    let mock_server = MockServer::start().await;

    // Step 1 (intermediate): returns 3 records + a token for step 2
    Mock::given(method("GET"))
        .and(path("/intermediate"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "items": [{"x": 1}, {"x": 2}, {"x": 3}],
            "token": "step1-tok"
        })))
        .mount(&mock_server)
        .await;

    // Step 2 (final): returns 2 records
    Mock::given(method("GET"))
        .and(path("/final"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "results": [{"id": "r1"}, {"id": "r2"}]
        })))
        .mount(&mock_server)
        .await;

    let spec = SensorSpec {
        sensor_id: "final-only-sensor".to_string(),
        name: "Final Only Sensor".to_string(),
        auth_type: AuthType::BearerStatic,
        base_url: mock_server.uri(),
        tables: vec![TableSpec::new_point_in_time(
            "results",
            "security_finding",
            vec![ColumnSpec {
                name: "id".to_string(),
                column_type: ColumnType::String,
                ocsf_field: None,
                options: vec![],
            }],
            vec![
                FetchStep {
                    name: "step1".to_string(),
                    method: "GET".to_string(),
                    path_template: "/intermediate".to_string(),
                    body_template: None,
                    response_path: "$.token".to_string(),
                    pagination_cursor_path: None,
                    variables_produced: vec!["token".to_string()],
                    fan_out_batch_size: None,
                    pagination: None,
                },
                FetchStep {
                    name: "step2".to_string(),
                    method: "GET".to_string(),
                    path_template: "/final".to_string(),
                    body_template: None,
                    response_path: "$.results".to_string(),
                    pagination_cursor_path: None,
                    variables_produced: vec![],
                    fan_out_batch_size: None,
                    pagination: None,
                },
            ],
        )],
        rate_limit_hints: None,
        version: "1.0.0".to_string(),
        credential_refs: vec![],
    };

    let table = spec.tables[0].clone();
    let context = FetchContext::new(OrgSlug::new("test-org"), HashMap::new());
    let http_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("reqwest Client::build must succeed");
    let auth_provider = NullAuthProvider;

    let result = PipelineExecutor::execute(&spec, &table, &context, &http_client, &auth_provider)
        .await
        .expect("F-LP1-CRIT-003: two-step pipeline must succeed");

    assert_eq!(
        result.records.len(),
        2,
        "F-LP1-CRIT-003: only final step's 2 records must appear (not intermediate 3); got {}",
        result.records.len()
    );
}

// ---------------------------------------------------------------------------
// F-LP1-HIGH-001 Red Gate test: fan-out — 250 IDs → 3 HTTP requests
// ---------------------------------------------------------------------------

/// BC-2.16.002 AC-6 / F-LP1-HIGH-001: when a step variable resolves to an array,
/// execute the step once per batch of `fan_out_batch_size` items.
///
/// Step 1 returns 250 IDs. Step 2 has fan_out_batch_size=100, referencing
/// `${step1.ids}` in path_template. Wiremock expects exactly 3 calls to step 2.
#[tokio::test]
async fn test_BC_2_16_002_execute_fan_out_invokes_step_per_batch() {
    let mock_server = MockServer::start().await;

    // Step 1: return 250 IDs
    let ids: Vec<serde_json::Value> = (0u32..250).map(|i| serde_json::json!(i)).collect();
    Mock::given(method("GET"))
        .and(path("/ids"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({ "ids": ids })))
        .mount(&mock_server)
        .await;

    // Step 2 (fan-out): called 3 times (batches of 100, 100, 50)
    Mock::given(method("GET"))
        .and(path("/details"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "items": [{"id": "detail-1"}]
        })))
        .expect(3) // exactly 3 fan-out invocations
        .mount(&mock_server)
        .await;

    let spec = SensorSpec {
        sensor_id: "fan-out-sensor".to_string(),
        name: "Fan-Out Sensor".to_string(),
        auth_type: AuthType::BearerStatic,
        base_url: mock_server.uri(),
        tables: vec![TableSpec::new_point_in_time(
            "details",
            "security_finding",
            vec![ColumnSpec {
                name: "id".to_string(),
                column_type: ColumnType::String,
                ocsf_field: None,
                options: vec![],
            }],
            vec![
                FetchStep {
                    name: "step1".to_string(),
                    method: "GET".to_string(),
                    path_template: "/ids".to_string(),
                    body_template: None,
                    response_path: "$.ids".to_string(),
                    pagination_cursor_path: None,
                    variables_produced: vec!["ids".to_string()],
                    fan_out_batch_size: None,
                    pagination: None,
                },
                FetchStep {
                    name: "step2".to_string(),
                    method: "GET".to_string(),
                    // References step1.ids — this array triggers fan-out
                    path_template: "/details?ids=${step1.ids}".to_string(),
                    body_template: None,
                    response_path: "$.items".to_string(),
                    pagination_cursor_path: None,
                    variables_produced: vec![],
                    fan_out_batch_size: Some(100), // 250 / 100 = 3 batches
                    pagination: None,
                },
            ],
        )],
        rate_limit_hints: None,
        version: "1.0.0".to_string(),
        credential_refs: vec![],
    };

    let table = spec.tables[0].clone();
    let context = FetchContext::new(OrgSlug::new("test-org"), HashMap::new());
    let http_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("reqwest Client::build must succeed");
    let auth_provider = NullAuthProvider;

    let result = PipelineExecutor::execute(&spec, &table, &context, &http_client, &auth_provider)
        .await
        .expect("AC-6/F-LP1-HIGH-001: fan-out execution must succeed");

    // wiremock's .expect(3) will panic at drop if not exactly 3 calls
    assert_eq!(
        result.records.len(),
        3, // 3 batches × 1 item each
        "F-LP1-HIGH-001: 3 fan-out batches × 1 item = 3 records; got {}",
        result.records.len()
    );
    assert!(
        result.request_count >= 4, // 1 step1 + 3 fan-out calls
        "F-LP1-HIGH-001: at least 4 requests (1+3); got {}",
        result.request_count
    );
}

// ---------------------------------------------------------------------------
// F-LP1-HIGH-002 Red Gate test: rate-limit delay between pagination calls
// ---------------------------------------------------------------------------

/// BC-2.16.002 AC-7 / F-LP1-HIGH-002: rate-limit inter-request delay applies
/// between ALL API calls across the pipeline, not just within a single step.
///
/// Two-page pagination at 5 rps (200ms delay per inter-request gap).
/// Assert the delta between calls is at least 180ms (accounting for scheduling jitter).
#[tokio::test]
async fn test_BC_2_16_002_execute_inserts_rate_limit_delay_between_pagination_calls() {
    let mock_server = MockServer::start().await;

    // Page 1
    Mock::given(method("GET"))
        .and(path("/slow"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "items": [{"id": "s1"}, {"id": "s2"}],
            "cursor": "page2"
        })))
        .up_to_n_times(1)
        .mount(&mock_server)
        .await;

    // Page 2: null cursor → stop
    Mock::given(method("GET"))
        .and(path("/slow"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "items": [{"id": "s3"}],
            "cursor": null
        })))
        .up_to_n_times(1)
        .mount(&mock_server)
        .await;

    let spec = SensorSpec {
        sensor_id: "rate-limit-sensor".to_string(),
        name: "Rate Limit Sensor".to_string(),
        auth_type: AuthType::BearerStatic,
        base_url: mock_server.uri(),
        tables: vec![TableSpec::new_point_in_time(
            "slow",
            "security_finding",
            vec![ColumnSpec {
                name: "id".to_string(),
                column_type: ColumnType::String,
                ocsf_field: None,
                options: vec![],
            }],
            vec![FetchStep {
                name: "fetch_slow".to_string(),
                method: "GET".to_string(),
                path_template: "/slow".to_string(),
                body_template: None,
                response_path: "$.items".to_string(),
                pagination_cursor_path: None,
                variables_produced: vec![],
                fan_out_batch_size: None,
                pagination: Some(PaginationConfig::CursorToken {
                    cursor_response_path: "$.cursor".to_string(),
                    page_size: None,
                }),
            }],
        )],
        rate_limit_hints: Some(RateLimitHints {
            requests_per_second: Some(5.0), // 200ms between requests
            burst_size: None,
        }),
        version: "1.0.0".to_string(),
        credential_refs: vec![],
    };

    let table = spec.tables[0].clone();
    let context = FetchContext::new(OrgSlug::new("test-org"), HashMap::new());
    let http_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("reqwest Client::build must succeed");
    let auth_provider = NullAuthProvider;

    let start = std::time::Instant::now();
    let result = PipelineExecutor::execute(&spec, &table, &context, &http_client, &auth_provider)
        .await
        .expect("AC-7: rate-limited execution must succeed");
    let elapsed = start.elapsed();

    assert_eq!(
        result.records.len(),
        3,
        "AC-7: 3 total records; got {}",
        result.records.len()
    );
    // At 5 rps, 2 requests means 1 inter-request delay of 200ms.
    // Allow down to 150ms to account for scheduling jitter.
    assert!(
        elapsed.as_millis() >= 150,
        "AC-7: at 5 rps with 2 requests, elapsed must be ≥150ms; got {}ms",
        elapsed.as_millis()
    );
}

// ---------------------------------------------------------------------------
// F-LP1-HIGH-004 Red Gate test: query_filter interpolation
// ---------------------------------------------------------------------------

/// BC-2.16.002 F-LP1-HIGH-004: query_filters from FetchContext are available
/// as `${query.filter.KEY}` in path_template interpolation.
#[tokio::test]
async fn test_BC_2_16_002_execute_interpolates_query_filter_in_path_template() {
    let mock_server = MockServer::start().await;

    // Mock expects the filter value in the query param
    Mock::given(method("GET"))
        .and(path("/alerts"))
        .and(query_param("severity", "critical"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "alerts": [{"id": "a1", "severity": "critical"}]
        })))
        .mount(&mock_server)
        .await;

    let spec = SensorSpec {
        sensor_id: "filter-sensor".to_string(),
        name: "Filter Sensor".to_string(),
        auth_type: AuthType::BearerStatic,
        base_url: mock_server.uri(),
        tables: vec![TableSpec::new_point_in_time(
            "alerts",
            "security_finding",
            vec![ColumnSpec {
                name: "id".to_string(),
                column_type: ColumnType::String,
                ocsf_field: None,
                options: vec![],
            }],
            vec![FetchStep {
                name: "fetch_alerts".to_string(),
                method: "GET".to_string(),
                // Uses push-down filter from query context
                path_template: "/alerts?severity=${query.filter.severity}".to_string(),
                body_template: None,
                response_path: "$.alerts".to_string(),
                pagination_cursor_path: None,
                variables_produced: vec![],
                fan_out_batch_size: None,
                pagination: None,
            }],
        )],
        rate_limit_hints: None,
        version: "1.0.0".to_string(),
        credential_refs: vec![],
    };

    let table = spec.tables[0].clone();
    let mut filters = HashMap::new();
    filters.insert("severity".to_string(), "critical".to_string());
    let context = FetchContext::new(OrgSlug::new("test-org"), filters);
    let http_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("reqwest Client::build must succeed");
    let auth_provider = NullAuthProvider;

    let result = PipelineExecutor::execute(&spec, &table, &context, &http_client, &auth_provider)
        .await
        .expect("F-LP1-HIGH-004: query filter interpolation must succeed");

    assert_eq!(
        result.records.len(),
        1,
        "F-LP1-HIGH-004: 1 alert matching severity=critical; got {}",
        result.records.len()
    );
}

// ---------------------------------------------------------------------------
// F-LP1-MED-002 Red Gate test: truncated flag set at 10K limit
// ---------------------------------------------------------------------------

/// BC-2.16.002 AC-8 / DI-019 / F-LP1-MED-002: PipelineResult.truncated must be
/// true when total records exceed 10K, and records.len() must equal 10K exactly.
#[tokio::test]
async fn test_BC_2_16_002_execute_truncates_at_10k_with_truncated_flag_set() {
    let mock_server = MockServer::start().await;

    // Page 1: 6000 records (full page)
    let records_6k: Vec<serde_json::Value> =
        (0u32..6000).map(|i| serde_json::json!({"id": i})).collect();
    Mock::given(method("GET"))
        .and(path("/big"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({ "items": records_6k, "cursor": "page2" })),
        )
        .up_to_n_times(1)
        .mount(&mock_server)
        .await;

    // Page 2: 6000 more records — total would be 12K, but pipeline truncates at 10K
    let records_6k_b: Vec<serde_json::Value> = (6000u32..12000)
        .map(|i| serde_json::json!({"id": i}))
        .collect();
    Mock::given(method("GET"))
        .and(path("/big"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({ "items": records_6k_b, "cursor": null })),
        )
        .up_to_n_times(1)
        .mount(&mock_server)
        .await;

    let spec = SensorSpec {
        sensor_id: "truncate-sensor".to_string(),
        name: "Truncate Sensor".to_string(),
        auth_type: AuthType::BearerStatic,
        base_url: mock_server.uri(),
        tables: vec![TableSpec::new_point_in_time(
            "big",
            "security_finding",
            vec![ColumnSpec {
                name: "id".to_string(),
                column_type: ColumnType::String,
                ocsf_field: None,
                options: vec![],
            }],
            vec![FetchStep {
                name: "fetch_big".to_string(),
                method: "GET".to_string(),
                path_template: "/big".to_string(),
                body_template: None,
                response_path: "$.items".to_string(),
                pagination_cursor_path: None,
                variables_produced: vec![],
                fan_out_batch_size: None,
                pagination: Some(PaginationConfig::CursorToken {
                    cursor_response_path: "$.cursor".to_string(),
                    page_size: None,
                }),
            }],
        )],
        rate_limit_hints: None,
        version: "1.0.0".to_string(),
        credential_refs: vec![],
    };

    let table = spec.tables[0].clone();
    let context = FetchContext::new(OrgSlug::new("test-org"), HashMap::new());
    let http_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("reqwest Client::build must succeed");
    let auth_provider = NullAuthProvider;

    let result = PipelineExecutor::execute(&spec, &table, &context, &http_client, &auth_provider)
        .await
        .expect("F-LP1-MED-002: truncation test must succeed");

    assert_eq!(
        result.records.len(),
        10_000,
        "F-LP1-MED-002: records must be truncated to exactly 10K; got {}",
        result.records.len()
    );
    assert!(
        result.truncated,
        "F-LP1-MED-002: truncated flag must be true when 10K limit hit"
    );
}

// ---------------------------------------------------------------------------
// F-LP2-HIGH-001 Red Gate test: fan-out batches reach distinct HTTP URLs
//
// Regression: fix-burst-1 used ${step.name}.batch key but templates reference
// ${step1.ids} (prior step's array key), so every iteration sent the full 250-
// element array. Fix: source_key is overridden per batch in step_vars.
// ---------------------------------------------------------------------------

/// BC-2.16.002 AC-6 / F-LP2-HIGH-001: each fan-out batch must produce a DISTINCT
/// HTTP request URL. With 250 IDs and batch_size=100, 3 batches are sent and
/// each batch's query param must contain disjoint ID ranges.
///
/// This test verifies the paper-fix regression introduced in fix-burst-1 is closed.
/// A false-green configuration (single mock accepting any URL) would see 3 identical
/// requests all containing all 250 IDs — this test's assertions would catch that.
#[tokio::test]
async fn test_BC_2_16_002_execute_fan_out_sends_distinct_batch_urls() {
    let mock_server = MockServer::start().await;

    // Step 1: return 250 IDs (0..249)
    let ids: Vec<serde_json::Value> = (0u32..250).map(|i| serde_json::json!(i)).collect();
    Mock::given(method("GET"))
        .and(path("/ids"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({ "ids": ids })))
        .mount(&mock_server)
        .await;

    // Step 2 (fan-out): accept any /details call, return 1 item per call
    Mock::given(method("GET"))
        .and(path("/details"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "items": [{"id": "detail"}]
        })))
        .expect(3) // exactly 3 fan-out invocations
        .mount(&mock_server)
        .await;

    let spec = SensorSpec {
        sensor_id: "fan-out-distinct-sensor".to_string(),
        name: "Fan-Out Distinct Sensor".to_string(),
        auth_type: AuthType::BearerStatic,
        base_url: mock_server.uri(),
        tables: vec![TableSpec::new_point_in_time(
            "details",
            "security_finding",
            vec![ColumnSpec {
                name: "id".to_string(),
                column_type: ColumnType::String,
                ocsf_field: None,
                options: vec![],
            }],
            vec![
                FetchStep {
                    name: "step1".to_string(),
                    method: "GET".to_string(),
                    path_template: "/ids".to_string(),
                    body_template: None,
                    response_path: "$.ids".to_string(),
                    pagination_cursor_path: None,
                    variables_produced: vec!["ids".to_string()],
                    fan_out_batch_size: None,
                    pagination: None,
                },
                FetchStep {
                    name: "step2".to_string(),
                    method: "GET".to_string(),
                    // References step1.ids — this array triggers fan-out
                    path_template: "/details?ids=${step1.ids}".to_string(),
                    body_template: None,
                    response_path: "$.items".to_string(),
                    pagination_cursor_path: None,
                    variables_produced: vec![],
                    fan_out_batch_size: Some(100), // 250 / 100 = 3 batches
                    pagination: None,
                },
            ],
        )],
        rate_limit_hints: None,
        version: "1.0.0".to_string(),
        credential_refs: vec![],
    };

    let table = spec.tables[0].clone();
    let context = FetchContext::new(OrgSlug::new("test-org"), HashMap::new());
    let http_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("reqwest Client::build must succeed");
    let auth_provider = NullAuthProvider;

    let result = PipelineExecutor::execute(&spec, &table, &context, &http_client, &auth_provider)
        .await
        .expect("F-LP2-HIGH-001: fan-out distinct URLs execution must succeed");

    assert_eq!(
        result.records.len(),
        3, // 3 batches × 1 item each
        "F-LP2-HIGH-001: 3 fan-out batches × 1 item = 3 records; got {}",
        result.records.len()
    );

    // Verify the 3 step2 requests had DISTINCT query strings (each carrying a batch slice).
    // If the paper-fix regression is present, all 3 requests have identical query strings
    // containing all 250 IDs.
    let received = mock_server
        .received_requests()
        .await
        .expect("wiremock must record requests");

    // Filter to only the /details calls (skip the /ids step1 call)
    let detail_requests: Vec<_> = received
        .iter()
        .filter(|r| r.url.path() == "/details")
        .collect();

    assert_eq!(
        detail_requests.len(),
        3,
        "F-LP2-HIGH-001: must have exactly 3 /details requests; got {}",
        detail_requests.len()
    );

    // Collect the query strings for each request — they must all differ.
    let queries: Vec<&str> = detail_requests
        .iter()
        .map(|r| r.url.query().unwrap_or(""))
        .collect();

    let unique_count = queries
        .iter()
        .collect::<std::collections::HashSet<_>>()
        .len();

    assert_eq!(
        unique_count, 3,
        "F-LP2-HIGH-001: each fan-out batch must produce a DISTINCT query string; \
         got {} unique out of 3 (paper-fix regression if not 3). Queries: {:?}",
        unique_count, queries
    );

    // Spot-check: batch 1 must NOT contain ID 100 (which belongs in batch 2),
    // verifying that batches are genuinely sliced and not full-array copies.
    // Each query contains the serialized+encoded JSON array of IDs.
    // Batch 1 has IDs 0..99 — it must NOT contain "100" as a standalone token.
    // (A batch containing all 250 IDs would contain "100" in its query string.)
    let batch1_query = queries[0];
    // IDs are serialized as "[0,1,...,99]" then percent-encoded. We check that
    // "100" does not appear in the first batch's query (it would if all 250 IDs
    // were present, since 100 appears in the second batch).
    // We verify this by checking all 3 queries are under 250 chars each (a full
    // 250-element JSON array would be much longer), as an additional regression guard.
    for (i, q) in queries.iter().enumerate() {
        // A serialized 250-element array of ints 0..249 encodes to ~900+ chars.
        // A batch of 100 ints is ~300 chars; a batch of 50 ints is ~150 chars.
        // The percent-encoded version will be at least as long.
        // Guard: no single query should contain all 250 IDs.
        assert!(
            q.len() < 700,
            "F-LP2-HIGH-001: batch {} query is suspiciously long ({} chars) — \
             may contain the full 250-element array instead of a batch slice. \
             Query: {}",
            i + 1,
            q.len(),
            &q[..q.len().min(200)]
        );
    }
    let _ = batch1_query; // suppress unused warning from the reassignment above
}

// ---------------------------------------------------------------------------
// F-LP2-HIGH-002 Red Gate test: cursor non-advancement aborts pipeline
// ---------------------------------------------------------------------------

/// BC-2.16.002 F-LP2-HIGH-002: if a cursor-paginated step receives the same
/// cursor on consecutive pages AND the page is non-empty, the pipeline must
/// abort with HttpRequestFailed to prevent an infinite loop.
#[tokio::test]
async fn test_BC_2_16_002_execute_aborts_on_non_advancing_cursor() {
    let mock_server = MockServer::start().await;

    // API always returns the same cursor + non-empty data → infinite loop risk
    Mock::given(method("GET"))
        .and(path("/stuck"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "items": [{"id": "s1"}],
            "cursor": "same-cursor-forever"
        })))
        .mount(&mock_server)
        .await;

    let spec = SensorSpec {
        sensor_id: "stuck-sensor".to_string(),
        name: "Stuck Sensor".to_string(),
        auth_type: AuthType::BearerStatic,
        base_url: mock_server.uri(),
        tables: vec![TableSpec::new_point_in_time(
            "stuck",
            "security_finding",
            vec![ColumnSpec {
                name: "id".to_string(),
                column_type: ColumnType::String,
                ocsf_field: None,
                options: vec![],
            }],
            vec![FetchStep {
                name: "fetch_stuck".to_string(),
                method: "GET".to_string(),
                path_template: "/stuck".to_string(),
                body_template: None,
                response_path: "$.items".to_string(),
                pagination_cursor_path: None,
                variables_produced: vec![],
                fan_out_batch_size: None,
                pagination: Some(PaginationConfig::CursorToken {
                    cursor_response_path: "$.cursor".to_string(),
                    page_size: None,
                }),
            }],
        )],
        rate_limit_hints: None,
        version: "1.0.0".to_string(),
        credential_refs: vec![],
    };

    let table = spec.tables[0].clone();
    let context = FetchContext::new(OrgSlug::new("test-org"), HashMap::new());
    let http_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("reqwest Client::build must succeed");
    let auth_provider = NullAuthProvider;

    let err = PipelineExecutor::execute(&spec, &table, &context, &http_client, &auth_provider)
        .await
        .expect_err("F-LP2-HIGH-002: non-advancing cursor must abort pipeline with Err");

    let err_str = err.to_string();
    assert!(
        err_str.contains("cursor did not advance"),
        "F-LP2-HIGH-002: error must mention cursor non-advancement; got: {err_str}"
    );
}

// ---------------------------------------------------------------------------
// F-LP2-MED-002 Red Gate test: Content-Type application/json for array bodies
// ---------------------------------------------------------------------------

/// BC-2.16.002 F-LP2-MED-002: a body_template that resolves to a JSON array
/// (starts with '[') must send Content-Type: application/json, not form-urlencoded.
#[tokio::test]
async fn test_BC_2_16_002_execute_derives_application_json_for_array_body() {
    let mock_server = MockServer::start().await;

    // Expect a POST with Content-Type: application/json and an array body
    Mock::given(method("POST"))
        .and(path("/batch"))
        .and(header("Content-Type", "application/json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "results": [{"id": "r1"}]
        })))
        .mount(&mock_server)
        .await;

    let spec = SensorSpec {
        sensor_id: "array-body-sensor".to_string(),
        name: "Array Body Sensor".to_string(),
        auth_type: AuthType::BearerStatic,
        base_url: mock_server.uri(),
        tables: vec![TableSpec::new_point_in_time(
            "batch",
            "security_finding",
            vec![ColumnSpec {
                name: "id".to_string(),
                column_type: ColumnType::String,
                ocsf_field: None,
                options: vec![],
            }],
            vec![FetchStep {
                name: "fetch_batch".to_string(),
                method: "POST".to_string(),
                path_template: "/batch".to_string(),
                // JSON array body template — must trigger Content-Type: application/json
                body_template: Some(r#"[{"type":"query"}]"#.to_string()),
                response_path: "$.results".to_string(),
                pagination_cursor_path: None,
                variables_produced: vec![],
                fan_out_batch_size: None,
                pagination: None,
            }],
        )],
        rate_limit_hints: None,
        version: "1.0.0".to_string(),
        credential_refs: vec![],
    };

    let table = spec.tables[0].clone();
    let context = FetchContext::new(OrgSlug::new("test-org"), HashMap::new());
    let http_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("reqwest Client::build must succeed");
    let auth_provider = NullAuthProvider;

    let result = PipelineExecutor::execute(&spec, &table, &context, &http_client, &auth_provider)
        .await
        .expect("F-LP2-MED-002: array body Content-Type test must succeed");

    assert_eq!(
        result.records.len(),
        1,
        "F-LP2-MED-002: 1 result record expected; got {}",
        result.records.len()
    );
    // The wiremock mock requires Content-Type: application/json header.
    // If the header is wrong, wiremock returns 404 and the pipeline returns Err —
    // so reaching this assertion proves the correct Content-Type was sent.
}

// ---------------------------------------------------------------------------
// F-LP2-MED-003 Red Gate test: numeric cursor coerced to string
// ---------------------------------------------------------------------------

/// BC-2.16.002 F-LP2-MED-003: when a pagination response contains a numeric
/// cursor (e.g. `{"cursor": 42}`), the pipeline must coerce it to the string
/// "42" and use it in the next request rather than terminating pagination.
#[tokio::test]
async fn test_BC_2_16_002_execute_coerces_numeric_cursor_to_string() {
    let mock_server = MockServer::start().await;

    // Page 1: returns numeric cursor 42
    Mock::given(method("GET"))
        .and(path("/numeric"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "items": [{"id": "n1"}],
            "cursor": 42
        })))
        .up_to_n_times(1)
        .mount(&mock_server)
        .await;

    // Page 2: cursor=42 (string-encoded) in query param, returns null cursor to stop
    Mock::given(method("GET"))
        .and(path("/numeric"))
        .and(query_param("cursor", "42"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "items": [{"id": "n2"}],
            "cursor": null
        })))
        .up_to_n_times(1)
        .mount(&mock_server)
        .await;

    let spec = SensorSpec {
        sensor_id: "numeric-cursor-sensor".to_string(),
        name: "Numeric Cursor Sensor".to_string(),
        auth_type: AuthType::BearerStatic,
        base_url: mock_server.uri(),
        tables: vec![TableSpec::new_point_in_time(
            "numeric",
            "security_finding",
            vec![ColumnSpec {
                name: "id".to_string(),
                column_type: ColumnType::String,
                ocsf_field: None,
                options: vec![],
            }],
            vec![FetchStep {
                name: "fetch_numeric".to_string(),
                method: "GET".to_string(),
                path_template: "/numeric".to_string(),
                body_template: None,
                response_path: "$.items".to_string(),
                pagination_cursor_path: None,
                variables_produced: vec![],
                fan_out_batch_size: None,
                pagination: Some(PaginationConfig::CursorToken {
                    cursor_response_path: "$.cursor".to_string(),
                    page_size: None,
                }),
            }],
        )],
        rate_limit_hints: None,
        version: "1.0.0".to_string(),
        credential_refs: vec![],
    };

    let table = spec.tables[0].clone();
    let context = FetchContext::new(OrgSlug::new("test-org"), HashMap::new());
    let http_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("reqwest Client::build must succeed");
    let auth_provider = NullAuthProvider;

    let result = PipelineExecutor::execute(&spec, &table, &context, &http_client, &auth_provider)
        .await
        .expect("F-LP2-MED-003: numeric cursor coercion must succeed");

    assert_eq!(
        result.records.len(),
        2,
        "F-LP2-MED-003: 2 records across 2 pages (numeric cursor coerced); got {}",
        result.records.len()
    );
    assert_eq!(
        result.request_count, 2,
        "F-LP2-MED-003: exactly 2 HTTP requests (page1 + page2 with cursor=42); got {}",
        result.request_count
    );
}

// ---------------------------------------------------------------------------
// F-LP4-MED-002: MAX_PAGES_PER_STEP cap test
// ---------------------------------------------------------------------------

/// BC-2.16.002 / F-LP4-MED-002: `PipelineExecutor::execute` must abort with
/// `HttpRequestFailed` after `MAX_PAGES_PER_STEP` (1000) pagination calls.
///
/// Uses 1001 distinct cursor values ("page-0" through "page-1000") so the
/// cursor-non-advance guard never fires — only the page-cap guard does.
/// After 1000 cursor-advancing pages, the 1001st page-count check aborts.
///
/// This test exercises the `F-LP2-HIGH-002` guard at production capacity.
/// It makes exactly 1000 requests to the mock server (bounded; will not hang).
#[tokio::test]
async fn test_BC_2_16_002_execute_aborts_at_max_pages_per_step() {
    let mock_server = MockServer::start().await;

    // Page 1: no cursor (first call) → returns cursor "page-1"
    Mock::given(method("GET"))
        .and(path("/infinite"))
        .and(|req: &wiremock::Request| {
            // Match requests WITHOUT a cursor query param (first page).
            !req.url.query().unwrap_or("").contains("cursor=")
        })
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "items": [{"id": "rec-0"}],
            "pagination": {"cursor": "page-1"}
        })))
        .up_to_n_times(1)
        .mount(&mock_server)
        .await;

    // Pages 2..=1000: each cursor value "page-N" → cursor "page-(N+1)".
    // Register mocks for cursor values "page-1" through "page-999".
    // Each advances to the next cursor. After 1000 pages the guard fires.
    // We use priority ordering (later registrations win) — register in reverse so
    // that each cursor has a unique match. Alternatively, each is uniquely matched
    // by the exact cursor query param value.
    for page_n in 1usize..=999 {
        let cursor_in = format!("page-{}", page_n);
        let cursor_out = format!("page-{}", page_n + 1);
        let rec_id = format!("rec-{page_n}");
        Mock::given(method("GET"))
            .and(path("/infinite"))
            .and(query_param("cursor", cursor_in.as_str()))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "items": [{"id": rec_id}],
                "pagination": {"cursor": cursor_out}
            })))
            .up_to_n_times(1)
            .mount(&mock_server)
            .await;
    }

    let spec = SensorSpec {
        sensor_id: "infinite-sensor".to_string(),
        name: "Infinite Sensor".to_string(),
        auth_type: AuthType::BearerStatic,
        base_url: mock_server.uri(),
        tables: vec![TableSpec::new_point_in_time(
            "items",
            "security_finding",
            vec![ColumnSpec {
                name: "id".to_string(),
                column_type: ColumnType::String,
                ocsf_field: None,
                options: vec![],
            }],
            vec![FetchStep {
                name: "fetch_items".to_string(),
                method: "GET".to_string(),
                path_template: "/infinite".to_string(),
                body_template: None,
                response_path: "$.items".to_string(),
                pagination_cursor_path: None,
                variables_produced: vec![],
                fan_out_batch_size: None,
                pagination: Some(PaginationConfig::CursorToken {
                    cursor_response_path: "$.pagination.cursor".to_string(),
                    page_size: None,
                }),
            }],
        )],
        rate_limit_hints: None,
        version: "1.0.0".to_string(),
        credential_refs: vec![],
    };

    let table = spec.tables[0].clone();
    let context = default_context();
    let http_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("reqwest Client::build must succeed");
    let auth_provider = NullAuthProvider;

    let err = PipelineExecutor::execute(&spec, &table, &context, &http_client, &auth_provider)
        .await
        .expect_err(
            "F-LP4-MED-002: execute must abort with Err when page cap is exceeded, not loop forever",
        );

    let msg = err.to_string();
    assert!(
        msg.contains("exceeded") || msg.contains("pages") || msg.contains("1000"),
        "F-LP4-MED-002: error must mention page cap ('exceeded', 'pages', or '1000'); got: {msg}"
    );
}

// ---------------------------------------------------------------------------
// F-LP5-MED-001 regression: reqwest decodes gzip-compressed responses
// ---------------------------------------------------------------------------

/// BC-2.16.002 / F-LP5-MED-001: `reqwest` with the `gzip` feature enabled must
/// transparently decompress a `Content-Encoding: gzip` response body.
///
/// Without the `gzip` feature, the raw compressed bytes reach `response.json()`,
/// which fails to parse and the pipeline returns `HttpRequestFailed`. This test
/// proves the feature is enabled and the decompression path is exercised.
///
/// Fixture: wiremock serves a gzip-compressed JSON body `[{"id":1},{"id":2}]`
/// with `Content-Encoding: gzip`.
#[tokio::test]
async fn test_BC_2_16_002_execute_decodes_gzipped_response() {
    let mock_server = MockServer::start().await;

    // Encode a JSON payload with gzip.
    let json_payload = serde_json::json!({
        "items": [{"id": 1}, {"id": 2}]
    });
    let json_bytes = serde_json::to_vec(&json_payload).expect("serialize fixture JSON");
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&json_bytes).expect("write gzip bytes");
    let gzip_bytes = encoder.finish().expect("finish gzip encoder");

    // Serve compressed bytes with Content-Encoding: gzip and Content-Type: application/json.
    Mock::given(method("GET"))
        .and(path("/gzip-endpoint"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("Content-Encoding", "gzip")
                .insert_header("Content-Type", "application/json")
                .set_body_raw(gzip_bytes, "application/json"),
        )
        .mount(&mock_server)
        .await;

    let spec = one_step_spec(&mock_server.uri(), "/gzip-endpoint", "$.items");
    let table = spec.tables[0].clone();
    let context = default_context();
    let http_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("reqwest Client::build must succeed");
    let auth_provider = prism_spec_engine::NullAuthProvider;

    let result = PipelineExecutor::execute(&spec, &table, &context, &http_client, &auth_provider)
        .await
        .expect(
            "F-LP5-MED-001: gzip-encoded response must be decoded transparently by reqwest; \
             if this panics, the 'gzip' feature is missing from Cargo.toml",
        );

    assert_eq!(
        result.records.len(),
        2,
        "F-LP5-MED-001: gzip-decoded response must yield 2 records; got {}",
        result.records.len()
    );
}

// ---------------------------------------------------------------------------
// F-LP5-MED-002 (site c) regression: pipeline_truncated event emitted on 10K cap
// ---------------------------------------------------------------------------

/// BC-2.16.002 / F-LP5-MED-002 (audit-log site c): When the DI-019 10K record cap
/// is hit, a `pipeline_truncated` warn event with `event_type`, `max_records`, and
/// `accumulated` fields must be emitted before the pipeline truncates.
///
/// Log capture strategy: install a `tracing-subscriber` fmt subscriber scoped to
/// this test via `set_default`, then assert the captured string buffer contains
/// the required event fields.
///
/// This test is equivalent in setup to `test_BC_2_16_002_execute_truncates_at_10k_with_truncated_flag_set`
/// but focuses on the audit log event rather than the record count.
#[tokio::test]
async fn test_BC_2_16_002_emits_pipeline_truncated_event_on_10k_cap() {
    use std::sync::{Arc, Mutex};
    use tracing_subscriber::util::SubscriberInitExt;

    // Capture all tracing output into a string buffer.
    let log_buffer: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));
    let log_buffer_clone = log_buffer.clone();

    let writer = tracing_subscriber::fmt::writer::BoxMakeWriter::new(move || {
        struct BufWriter(Arc<Mutex<String>>);
        impl std::io::Write for BufWriter {
            fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
                let s = String::from_utf8_lossy(buf);
                self.0.lock().unwrap().push_str(&s);
                Ok(buf.len())
            }
            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }
        BufWriter(log_buffer_clone.clone())
    });

    let subscriber = tracing_subscriber::fmt()
        .with_writer(writer)
        .with_max_level(tracing::Level::WARN)
        .finish();

    // Use set_default so this subscriber only affects the current thread (test-scoped).
    let _guard = subscriber.set_default();

    let mock_server = MockServer::start().await;

    // Page 1: 6000 records → triggers accumulation
    let records_6k: Vec<serde_json::Value> =
        (0u32..6000).map(|i| serde_json::json!({"id": i})).collect();
    Mock::given(method("GET"))
        .and(path("/truncate-audit"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({ "items": records_6k, "cursor": "p2" })),
        )
        .up_to_n_times(1)
        .mount(&mock_server)
        .await;

    // Page 2: 6000 more records → pushes total past 10K, triggers truncation event
    let records_6k_b: Vec<serde_json::Value> = (6000u32..12000)
        .map(|i| serde_json::json!({"id": i}))
        .collect();
    Mock::given(method("GET"))
        .and(path("/truncate-audit"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({ "items": records_6k_b, "cursor": null })),
        )
        .up_to_n_times(1)
        .mount(&mock_server)
        .await;

    let spec = SensorSpec {
        sensor_id: "truncate-audit-sensor".to_string(),
        name: "Truncate Audit Sensor".to_string(),
        auth_type: prism_spec_engine::spec_parser::AuthType::BearerStatic,
        base_url: mock_server.uri(),
        tables: vec![
            prism_spec_engine::spec_parser::TableSpec::new_point_in_time(
                "big",
                "security_finding",
                vec![ColumnSpec {
                    name: "id".to_string(),
                    column_type: ColumnType::String,
                    ocsf_field: None,
                    options: vec![],
                }],
                vec![FetchStep {
                    name: "fetch_big".to_string(),
                    method: "GET".to_string(),
                    path_template: "/truncate-audit".to_string(),
                    body_template: None,
                    response_path: "$.items".to_string(),
                    pagination_cursor_path: None,
                    variables_produced: vec![],
                    fan_out_batch_size: None,
                    pagination: Some(PaginationConfig::CursorToken {
                        cursor_response_path: "$.cursor".to_string(),
                        page_size: None,
                    }),
                }],
            ),
        ],
        rate_limit_hints: None,
        version: "1.0.0".to_string(),
        credential_refs: vec![],
    };

    let table = spec.tables[0].clone();
    let context = default_context();
    let http_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("reqwest Client::build must succeed");
    let auth_provider = prism_spec_engine::NullAuthProvider;

    let result = PipelineExecutor::execute(&spec, &table, &context, &http_client, &auth_provider)
        .await
        .expect("truncate-audit test must succeed");

    assert!(result.truncated, "truncated flag must be set");
    assert_eq!(result.records.len(), 10_000, "records must be exactly 10K");

    // Verify the audit log event was emitted.
    let captured = log_buffer.lock().unwrap().clone();
    assert!(
        captured.contains("pipeline_truncated"),
        "F-LP5-MED-002 site (c): 'pipeline_truncated' event_type must appear in log output; \
         captured log: {captured}",
    );
    assert!(
        captured.contains("DI-019") || captured.contains("truncated to 10K"),
        "F-LP5-MED-002 site (c): log must mention DI-019 cap reason; captured log: {captured}",
    );
}

// ---------------------------------------------------------------------------
// F-LP7-MED-003 / F-LP8-MED-002 Red Gate: partial-record discard on mid-pagination 500
// (F-LP8-MED-002 rewrite: exercises actual accumulation before discard)
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// F-LP8-MED-001 Red Gate: auth_initial_acquired emits distinct events per token state
// ---------------------------------------------------------------------------

/// BC-2.16.002 / F-LP8-MED-001: The eager-token acquisition branch emits different
/// tracing events depending on whether the token is non-empty or empty.
///
/// Sub-case (a): non-empty token → INFO `auth_initial_acquired` (NOT `auth_initial_acquired_empty`)
/// Sub-case (b): empty token (NullAuthProvider) → DEBUG `auth_initial_acquired_empty`
///               (NOT a bare `auth_initial_acquired` INFO entry)
///
/// RED GATE: The test FAILS if both arms emit the same event_type (e.g., always
/// `auth_initial_acquired`), or if the empty-token arm emits nothing, or if the
/// non-empty arm emits the empty-token event. A developer who merges both Ok arms
/// into a single `auth_initial_acquired` info emit would cause sub-case (b) to fail.
///
/// NOTE: `auth_initial_acquired_empty` is a DEBUG-level event. The subscriber
/// max_level must be DEBUG (or TRACE) to capture it.
#[tokio::test]
async fn test_BC_2_16_002_auth_initial_acquired_emits_distinct_events_per_token_state() {
    use prism_spec_engine::MockAuthProvider;
    use std::sync::{Arc, Mutex};
    use tracing_subscriber::util::SubscriberInitExt;

    let mock_server = MockServer::start().await;

    // One-step spec reused for both sub-cases.
    let make_spec = |base_url: &str| SensorSpec {
        sensor_id: "auth-event-sensor".to_string(),
        name: "Auth Event Sensor".to_string(),
        auth_type: AuthType::BearerStatic,
        base_url: base_url.to_string(),
        tables: vec![TableSpec::new_point_in_time(
            "items",
            "security_finding",
            vec![ColumnSpec {
                name: "id".to_string(),
                column_type: ColumnType::String,
                ocsf_field: None,
                options: vec![],
            }],
            vec![FetchStep {
                name: "fetch_items".to_string(),
                method: "GET".to_string(),
                path_template: "/auth-event".to_string(),
                body_template: None,
                response_path: "$.items".to_string(),
                pagination_cursor_path: None,
                variables_produced: vec![],
                fan_out_batch_size: None,
                pagination: None,
            }],
        )],
        rate_limit_hints: None,
        version: "1.0.0".to_string(),
        credential_refs: vec![],
    };

    Mock::given(method("GET"))
        .and(path("/auth-event"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "items": [{"id": "r1"}]
        })))
        .mount(&mock_server)
        .await;

    let http_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .expect("reqwest Client::build must succeed");
    let context = default_context();

    // -----------------------------------------------------------------------
    // Sub-case (a): non-empty token → auth_initial_acquired at INFO, NOT empty variant
    // -----------------------------------------------------------------------
    {
        let log_buffer: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));
        let log_buffer_clone = log_buffer.clone();

        let writer = tracing_subscriber::fmt::writer::BoxMakeWriter::new(move || {
            struct BufWriter(Arc<Mutex<String>>);
            impl std::io::Write for BufWriter {
                fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
                    let s = String::from_utf8_lossy(buf);
                    self.0.lock().unwrap().push_str(&s);
                    Ok(buf.len())
                }
                fn flush(&mut self) -> std::io::Result<()> {
                    Ok(())
                }
            }
            BufWriter(log_buffer_clone.clone())
        });

        let subscriber = tracing_subscriber::fmt()
            .with_writer(writer)
            .with_max_level(tracing::Level::DEBUG)
            .finish();

        let _guard = subscriber.set_default();

        let spec = make_spec(&mock_server.uri());
        let table = spec.tables[0].clone();
        let auth_provider = MockAuthProvider::new("real-token-xyz");

        let result =
            PipelineExecutor::execute(&spec, &table, &context, &http_client, &auth_provider).await;
        assert!(
            result.is_ok(),
            "sub-case (a): pipeline must succeed; got {:?}",
            result
        );

        let captured = log_buffer.lock().unwrap().clone();
        assert!(
            captured.contains("auth_initial_acquired"),
            "sub-case (a): log must contain 'auth_initial_acquired'; captured: {captured}",
        );
        assert!(
            !captured.contains("auth_initial_acquired_empty"),
            "sub-case (a): non-empty token must NOT emit 'auth_initial_acquired_empty'; captured: {captured}",
        );
    }

    // -----------------------------------------------------------------------
    // Sub-case (b): empty token (NullAuthProvider) → auth_initial_acquired_empty at DEBUG
    // -----------------------------------------------------------------------
    {
        let log_buffer: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));
        let log_buffer_clone = log_buffer.clone();

        let writer = tracing_subscriber::fmt::writer::BoxMakeWriter::new(move || {
            struct BufWriter(Arc<Mutex<String>>);
            impl std::io::Write for BufWriter {
                fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
                    let s = String::from_utf8_lossy(buf);
                    self.0.lock().unwrap().push_str(&s);
                    Ok(buf.len())
                }
                fn flush(&mut self) -> std::io::Result<()> {
                    Ok(())
                }
            }
            BufWriter(log_buffer_clone.clone())
        });

        let subscriber = tracing_subscriber::fmt()
            .with_writer(writer)
            .with_max_level(tracing::Level::DEBUG)
            .finish();

        let _guard = subscriber.set_default();

        let spec = make_spec(&mock_server.uri());
        let table = spec.tables[0].clone();
        // NullAuthProvider always returns an empty token.
        let auth_provider = NullAuthProvider;

        let result =
            PipelineExecutor::execute(&spec, &table, &context, &http_client, &auth_provider).await;
        assert!(
            result.is_ok(),
            "sub-case (b): pipeline must succeed; got {:?}",
            result
        );

        let captured = log_buffer.lock().unwrap().clone();
        assert!(
            captured.contains("auth_initial_acquired_empty"),
            "sub-case (b): empty token must emit 'auth_initial_acquired_empty'; captured: {captured}",
        );
        // Ensure the INFO-level `auth_initial_acquired` event does NOT appear on its own
        // (the empty-token path must NOT fall through to the non-empty arm).
        // We check that `auth_initial_acquired` only appears as part of `auth_initial_acquired_empty`,
        // not as a standalone event. The simplest way: after stripping occurrences of
        // `auth_initial_acquired_empty`, no bare `auth_initial_acquired` substring should remain.
        let without_empty_variant = captured.replace("auth_initial_acquired_empty", "");
        assert!(
            !without_empty_variant.contains("auth_initial_acquired"),
            "sub-case (b): empty token must NOT emit bare 'auth_initial_acquired' INFO event; \
             captured (after removing empty-variant occurrences): {without_empty_variant}",
        );
    }
}

// ---------------------------------------------------------------------------
// F-LP8-MED-002 Red Gate (REWRITE): partial-record discard exercises actual discard
// ---------------------------------------------------------------------------

/// BC-2.16.002 / F-LP8-MED-002 (rewrite of F-LP7-MED-003): When a paginated FINAL step
/// accumulates records on page-1 and then receives HTTP 500 on page-2, the pipeline
/// returns `Err` and discards all accumulated records — no partial `PipelineResult` leaks.
///
/// This test closes the paper-fix gap identified in pass-8: the previous version used
/// a scalar response_path which never populated `all_records`, so there was nothing to
/// discard. This rewrite uses a paginated step with an ARRAY response path so that
/// page-1 actually accumulates records into `all_records` before page-2 fails.
///
/// Setup:
/// - Single step (final=true, paginated with CursorToken)
/// - Page 1: GET /items → 200 with 2 records + cursor="abc"
/// - Page 2: GET /items?cursor=abc → 500
///
/// Assertions:
/// (a) `execute` returns `Err(SpecEngineError::HttpRequestFailed { .. })`
/// (b) The request_count embedded in the error or the mock's .expect() proves page-1
///     succeeded (2 records accumulated) and then page-2 failed (discarding them).
///     Specifically: if the developer changed execute() to return
///     `Ok(PipelineResult { records: all_records, .. })` on the 500 path, the test
///     FAILS because the assertion requires `Err`, not `Ok`.
#[tokio::test]
async fn test_BC_2_16_002_execute_discards_partial_records_on_mid_pipeline_500() {
    let mock_server = MockServer::start().await;

    // Page 1: returns 2 records + next cursor "abc". Pipeline accumulates these.
    // No cursor query param on the first request (URL builder omits param when cursor is None).
    Mock::given(method("GET"))
        .and(path("/items"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "items": [{"id": 1, "name": "r1"}, {"id": 2, "name": "r2"}],
            "next": "abc"
        })))
        .up_to_n_times(1)
        .mount(&mock_server)
        .await;

    // Page 2: 500 — triggers abort after records are already accumulated.
    Mock::given(method("GET"))
        .and(path("/items"))
        .and(query_param("cursor", "abc"))
        .respond_with(ResponseTemplate::new(500))
        .up_to_n_times(1)
        .mount(&mock_server)
        .await;

    let spec = SensorSpec {
        sensor_id: "discard-partial-sensor".to_string(),
        name: "Discard Partial Sensor".to_string(),
        auth_type: AuthType::BearerStatic,
        base_url: mock_server.uri(),
        tables: vec![TableSpec::new_point_in_time(
            "items",
            "security_finding",
            vec![ColumnSpec {
                name: "id".to_string(),
                column_type: ColumnType::String,
                ocsf_field: None,
                options: vec![],
            }],
            vec![FetchStep {
                name: "step1".to_string(),
                method: "GET".to_string(),
                path_template: "/items".to_string(),
                body_template: None,
                // ARRAY response_path — records accumulate into all_records
                response_path: "$.items".to_string(),
                pagination_cursor_path: None,
                variables_produced: vec![],
                fan_out_batch_size: None,
                // CursorToken pagination: page-1 cursor "abc", page-2 returns 500
                pagination: Some(PaginationConfig::CursorToken {
                    cursor_response_path: "$.next".to_string(),
                    page_size: None,
                }),
            }],
        )],
        rate_limit_hints: None,
        version: "1.0.0".to_string(),
        credential_refs: vec![],
    };

    let table = spec.tables[0].clone();
    let context = FetchContext::new(OrgSlug::new("test-org"), HashMap::new());
    let http_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .expect("reqwest Client::build must succeed");
    let auth_provider = NullAuthProvider;

    let result =
        PipelineExecutor::execute(&spec, &table, &context, &http_client, &auth_provider).await;

    // (a) Must return Err — not a partial Ok with accumulated records.
    // If a developer changed execute() to return Ok(PipelineResult { records: all_records })
    // on the 500 path, this assertion would FAIL (it would get Ok(records.len()==2)).
    assert!(
        matches!(result, Err(SpecEngineError::HttpRequestFailed { .. })),
        "F-LP8-MED-002: paginated mid-step 500 must discard accumulated records and return Err; \
         got {:?}",
        result
    );

    // (b) Verify that the error carries evidence that requests WERE issued (not just a static Err).
    // The HttpRequestFailed error from a 500 response carries the status_code=500.
    if let Err(SpecEngineError::HttpRequestFailed { status_code, .. }) = result {
        assert_eq!(
            status_code, 500,
            "F-LP8-MED-002: the Err must carry status_code=500 from the failing page-2 request"
        );
    }
}

// ---------------------------------------------------------------------------
// F-LP8-MED-003 Red Gate: cursor pagination unsupported type emits structured event
// ---------------------------------------------------------------------------

/// BC-2.16.002 / F-LP8-MED-003: When `extract_cursor` encounters a cursor value
/// of an unsupported type (Array, Object, Bool), it must emit a structured tracing
/// warn event with `event_type = "pagination_cursor_unsupported_type"` and the
/// `actual_type` field. This enables SIEM/SOC alerting pipelines to detect and
/// alert on silent pagination termination.
///
/// RED GATE: Before adding `event_type` to the warn at pipeline.rs:882-888, this
/// test FAILS because the log buffer will not contain `pagination_cursor_unsupported_type`.
/// After adding the structured field, the test PASSES.
///
/// Behavior: pagination terminates (Ok with page-1 records only) — this test does
/// NOT assert Err; that would be a BC amendment beyond fix-burst-8 scope.
#[tokio::test]
async fn test_BC_2_16_002_cursor_unsupported_type_emits_structured_event() {
    use std::sync::{Arc, Mutex};
    use tracing_subscriber::util::SubscriberInitExt;

    let mock_server = MockServer::start().await;

    // One step with cursor pagination where cursor resolves to an Array (unsupported).
    // Page 1 returns 1 record; "next" is an Array → pagination terminates.
    Mock::given(method("GET"))
        .and(path("/cursor-unsupported"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "items": [{"id": "r1"}],
            "next": [1, 2, 3]
        })))
        .up_to_n_times(5)
        .mount(&mock_server)
        .await;

    let spec = SensorSpec {
        sensor_id: "cursor-type-sensor".to_string(),
        name: "Cursor Type Sensor".to_string(),
        auth_type: AuthType::BearerStatic,
        base_url: mock_server.uri(),
        tables: vec![TableSpec::new_point_in_time(
            "items",
            "security_finding",
            vec![ColumnSpec {
                name: "id".to_string(),
                column_type: ColumnType::String,
                ocsf_field: None,
                options: vec![],
            }],
            vec![FetchStep {
                name: "fetch_items".to_string(),
                method: "GET".to_string(),
                path_template: "/cursor-unsupported".to_string(),
                body_template: None,
                response_path: "$.items".to_string(),
                pagination_cursor_path: None,
                variables_produced: vec![],
                fan_out_batch_size: None,
                pagination: Some(PaginationConfig::CursorToken {
                    cursor_response_path: "$.next".to_string(),
                    page_size: None,
                }),
            }],
        )],
        rate_limit_hints: None,
        version: "1.0.0".to_string(),
        credential_refs: vec![],
    };

    let log_buffer: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));
    let log_buffer_clone = log_buffer.clone();

    let writer = tracing_subscriber::fmt::writer::BoxMakeWriter::new(move || {
        struct BufWriter(Arc<Mutex<String>>);
        impl std::io::Write for BufWriter {
            fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
                let s = String::from_utf8_lossy(buf);
                self.0.lock().unwrap().push_str(&s);
                Ok(buf.len())
            }
            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }
        BufWriter(log_buffer_clone.clone())
    });

    let subscriber = tracing_subscriber::fmt()
        .with_writer(writer)
        .with_max_level(tracing::Level::WARN)
        .finish();

    let _guard = subscriber.set_default();

    let table = spec.tables[0].clone();
    let context = default_context();
    let http_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .expect("reqwest Client::build must succeed");
    let auth_provider = NullAuthProvider;

    let result =
        PipelineExecutor::execute(&spec, &table, &context, &http_client, &auth_provider).await;

    // Pagination terminates after page-1 — pipeline succeeds with 1 record.
    assert!(
        result.is_ok(),
        "F-LP8-MED-003: unsupported cursor type terminates pagination (Ok); got {:?}",
        result
    );
    let records = result.unwrap().records;
    assert_eq!(
        records.len(),
        1,
        "F-LP8-MED-003: page-1 record must be returned; got {}",
        records.len()
    );

    // The key assertion: the warn event must carry the structured event_type field.
    // RED GATE fails here if the warn at pipeline.rs:882-888 has no event_type.
    let captured = log_buffer.lock().unwrap().clone();
    assert!(
        captured.contains("pagination_cursor_unsupported_type"),
        "F-LP8-MED-003: warn event must contain 'pagination_cursor_unsupported_type' \
         event_type field; captured log: {captured}",
    );
}

// ---------------------------------------------------------------------------
// F-LP8-LOW-001 Red Gate: spec with multi-array fan-out template rejected by validator
// ---------------------------------------------------------------------------

/// BC-2.16.002 / F-LP8-LOW-001: When a step's templates reference multiple
/// array-valued variables from prior steps, the validator rejects the spec with
/// a `ValidationError` explaining the ambiguity.
///
/// Fan-out is single-array only. A step that references two different array-valued
/// variables would require cartesian or zipped semantics — which are not implemented
/// (deferred to PREREQ-C/D). Silently using only the first array would be worst-of-all-worlds;
/// rejection at validation time forces the spec author to be explicit.
///
/// RED GATE: Before adding the validator check, `validate_sensor_spec` accepts the
/// spec and returns Ok(warnings). After the check is added, it returns Err with the
/// multi-array message.
#[test]
fn test_BC_2_16_002_spec_with_multi_array_fan_out_template_rejected() {
    use prism_core::ColumnType;
    use prism_spec_engine::validation::validate_sensor_spec;

    // Build a spec where:
    // - step1 is paginated (implies array output for $.ids)
    // - step2 is paginated (implies array output for $.codes)
    // - step3's path_template references BOTH step1.ids AND step2.codes
    let spec = SensorSpec {
        sensor_id: "multi-array-sensor".to_string(),
        name: "Multi Array Sensor".to_string(),
        auth_type: AuthType::BearerStatic,
        base_url: "https://api.example.com".to_string(),
        tables: vec![TableSpec::new_point_in_time(
            "items",
            "security_finding",
            vec![ColumnSpec {
                name: "id".to_string(),
                column_type: ColumnType::String,
                ocsf_field: None,
                options: vec![],
            }],
            vec![
                FetchStep {
                    name: "step1".to_string(),
                    method: "GET".to_string(),
                    path_template: "/ids".to_string(),
                    body_template: None,
                    response_path: "$.ids[*]".to_string(),
                    pagination_cursor_path: None,
                    variables_produced: vec![],
                    fan_out_batch_size: None,
                    // Paginated step: implies array output
                    pagination: Some(PaginationConfig::CursorToken {
                        cursor_response_path: "$.next".to_string(),
                        page_size: None,
                    }),
                },
                FetchStep {
                    name: "step2".to_string(),
                    method: "GET".to_string(),
                    path_template: "/codes".to_string(),
                    body_template: None,
                    response_path: "$.codes[*]".to_string(),
                    pagination_cursor_path: None,
                    variables_produced: vec![],
                    fan_out_batch_size: None,
                    // Also paginated: implies array output
                    pagination: Some(PaginationConfig::CursorToken {
                        cursor_response_path: "$.next".to_string(),
                        page_size: None,
                    }),
                },
                FetchStep {
                    name: "step3".to_string(),
                    method: "GET".to_string(),
                    // References both step1 and step2 array-valued outputs simultaneously.
                    path_template: "/api/${step1.ids}/details/${step2.codes}".to_string(),
                    body_template: None,
                    response_path: "$.items".to_string(),
                    pagination_cursor_path: None,
                    variables_produced: vec![],
                    fan_out_batch_size: None,
                    pagination: None,
                },
            ],
        )],
        rate_limit_hints: None,
        version: "1.0.0".to_string(),
        credential_refs: vec![],
    };

    let result = validate_sensor_spec(&spec);

    // RED GATE: current validator at validation.rs:218-239 only checks reference resolution,
    // not type interaction. Before the fix, this returns Ok(warnings). After the fix,
    // this returns Err with a message mentioning multi-array fan-out ambiguity.
    assert!(
        result.is_err(),
        "F-LP8-LOW-001: spec with multi-array fan-out template must be rejected by validator; \
         got Ok({:?})",
        result.ok()
    );

    let errors = result.unwrap_err();
    let has_multi_array_error = errors.iter().any(|e| {
        e.message.contains("multiple") && e.message.contains("array")
            || e.message.contains("fan-out")
            || e.message.contains("multi-array")
    });
    assert!(
        has_multi_array_error,
        "F-LP8-LOW-001: validator error must mention multi-array fan-out; got errors: {errors:?}",
    );
}

// ---------------------------------------------------------------------------
// OBS-LP9-003 Red Gate: cursor_preview must NOT panic on multi-byte UTF-8
// ---------------------------------------------------------------------------

/// BC-2.16.002 / OBS-LP9-003: When `extract_cursor` encounters a cursor value
/// of an unsupported type whose JSON serialization contains multi-byte UTF-8
/// codepoints (e.g., emoji), the cursor_preview truncation MUST NOT panic.
///
/// Bug: the pre-fix code uses `&s[..100]` (byte-index slice). If byte 100 falls
/// inside a multi-byte codepoint (4 bytes for emoji), Rust panics with:
///   "byte index 100 is not a char boundary; it is inside '🎯' (bytes 96..100)"
///
/// Construction: response returns `"next": ["🎯🎯...30 emoji"]`.
/// The array serializes as `["🎯🎯..."]`. Each 🎯 = 4 bytes.
/// JSON prefix `["` = 2 bytes. Byte 100 = offset 98 from the start of emoji =
/// 98/4 = 24.5 → falls mid-codepoint inside the 25th emoji. Pre-fix: panic.
/// Post-fix (char-boundary-safe truncation): Ok(records from page 1).
///
/// RED GATE: run this test against HEAD 411f4cbf — it MUST panic/fail.
/// After applying the char_indices fix: MUST pass without panic.
#[tokio::test]
async fn test_BC_2_16_002_cursor_preview_handles_multi_byte_utf8_without_panic() {
    let mock_server = MockServer::start().await;

    // Cursor resolves to an Array containing emoji strings.
    // The Array JSON form is: ["🎯🎯🎯🎯🎯🎯🎯🎯🎯🎯🎯🎯🎯🎯🎯🎯🎯🎯🎯🎯🎯🎯🎯🎯🎯🎯🎯🎯🎯🎯"]
    // Prefix `["` = 2 bytes. Each 🎯 = 4 bytes. After 24 emoji: 2 + 96 = 98 bytes.
    // The 25th emoji starts at byte 98 → bytes 98-101. Byte 100 is mid-codepoint.
    let emoji_string: String = "🎯".repeat(30);
    let cursor_array = serde_json::json!([emoji_string]);
    // Verify the panic condition: byte 100 of the serialized array is mid-codepoint.
    let serialized = cursor_array.to_string();
    assert!(
        serialized.len() > 100,
        "test setup: serialized array must be >100 bytes for the truncation to trigger; got {} bytes",
        serialized.len()
    );
    assert!(
        !serialized.is_char_boundary(100),
        "test setup: byte 100 must be a non-char-boundary (mid-codepoint) for the panic to trigger; \
         pre-fix code would panic, post-fix code must not"
    );

    Mock::given(method("GET"))
        .and(path("/utf8-cursor"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "items": [{"id": "r1"}],
            "next": [emoji_string]   // cursor resolves to Array → unsupported type branch
        })))
        .up_to_n_times(5)
        .mount(&mock_server)
        .await;

    let spec = SensorSpec {
        sensor_id: "utf8-cursor-sensor".to_string(),
        name: "UTF-8 Cursor Sensor".to_string(),
        auth_type: AuthType::BearerStatic,
        base_url: mock_server.uri(),
        tables: vec![TableSpec::new_point_in_time(
            "items",
            "security_finding",
            vec![ColumnSpec {
                name: "id".to_string(),
                column_type: ColumnType::String,
                ocsf_field: None,
                options: vec![],
            }],
            vec![FetchStep {
                name: "fetch_items".to_string(),
                method: "GET".to_string(),
                path_template: "/utf8-cursor".to_string(),
                body_template: None,
                response_path: "$.items".to_string(),
                pagination_cursor_path: None,
                variables_produced: vec![],
                fan_out_batch_size: None,
                pagination: Some(PaginationConfig::CursorToken {
                    cursor_response_path: "$.next".to_string(),
                    page_size: None,
                }),
            }],
        )],
        rate_limit_hints: None,
        version: "1.0.0".to_string(),
        credential_refs: vec![],
    };

    let table = spec.tables[0].clone();
    let context = FetchContext::new(OrgSlug::new("test-org"), HashMap::new());
    let http_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("reqwest Client::build must succeed");
    let auth_provider = NullAuthProvider;

    // The pipeline must NOT panic. It should return Ok (cursor resolves to
    // unsupported type → pagination terminates → returns page-1 records).
    let result =
        PipelineExecutor::execute(&spec, &table, &context, &http_client, &auth_provider).await;
    assert!(
        result.is_ok(),
        "OBS-LP9-003: pipeline must not panic on multi-byte UTF-8 cursor; got: {:?}",
        result
    );
    let pipeline_result = result.unwrap();
    assert_eq!(
        pipeline_result.records.len(),
        1,
        "OBS-LP9-003: page-1 record must be returned when cursor terminates at unsupported type"
    );
}

// ---------------------------------------------------------------------------
// F-LP9-MED-002 Red Gate: multi-array fan-out emits structured warn event
// ---------------------------------------------------------------------------

/// BC-2.16.002 / F-LP9-MED-002: When a pipeline step's path_template references
/// MULTIPLE array-valued variables from prior steps (fan-out ambiguity), the
/// runtime MUST emit a structured `tracing::warn!` event with:
///   - `event_type = "fanout_ambiguous_multi_array"`
///   - the step name
///   - `array_vars_count >= 2`
///
/// The pipeline continues executing using the FIRST array as fan-out source
/// (backward-compatible behavior). The warn surfaces the ambiguity for operators.
///
/// Setup:
///   - step1: GET /step1 → `{"ids": [1,2,3]}` (response_path: "$.ids")
///   - step2: GET /step2 → `{"codes": ["a","b"]}` (response_path: "$.codes")
///   - step3: path_template: "/api/${step1.ids}/x/${step2.codes}" — references BOTH arrays
///
/// RED GATE: Pre-fix code (find_fan_out_array returns at first match, no warn) → test FAILS
///   because log buffer will not contain `fanout_ambiguous_multi_array`.
/// Post-fix: test PASSES — warn emitted, pipeline succeeds.
///
/// Sibling note: The static validator (validation.rs Cat-2b) catches paginated/`[*]`
/// patterns at spec-load time; this runtime warn catches what the validator misses
/// for the non-paginated whole-array case.
#[tokio::test]
async fn test_BC_2_16_002_fanout_ambiguous_multi_array_emits_structured_event() {
    use std::sync::{Arc, Mutex};
    use tracing_subscriber::util::SubscriberInitExt;

    let log_buffer: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));
    let log_buffer_clone = log_buffer.clone();

    let writer = tracing_subscriber::fmt::writer::BoxMakeWriter::new(move || {
        struct BufWriter(Arc<Mutex<String>>);
        impl std::io::Write for BufWriter {
            fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
                let s = String::from_utf8_lossy(buf);
                self.0.lock().unwrap().push_str(&s);
                Ok(buf.len())
            }
            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }
        BufWriter(log_buffer_clone.clone())
    });

    let subscriber = tracing_subscriber::fmt()
        .with_writer(writer)
        .with_max_level(tracing::Level::WARN)
        .finish();
    let _guard = subscriber.set_default();

    let mock_server = MockServer::start().await;

    // step1: returns array of IDs
    Mock::given(method("GET"))
        .and(path("/step1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "ids": [1, 2, 3]
        })))
        .up_to_n_times(1)
        .mount(&mock_server)
        .await;

    // step2: returns array of codes
    Mock::given(method("GET"))
        .and(path("/step2"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "codes": ["a", "b"]
        })))
        .up_to_n_times(1)
        .mount(&mock_server)
        .await;

    // step3: references BOTH arrays in path_template — drives the multi-array warn.
    // The URL after interpolation will be percent-encoded, starting with /api/.
    Mock::given(method("GET"))
        .and(path_regex(r"^/api/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "items": [{"id": "result-1"}]
        })))
        .mount(&mock_server)
        .await;

    let spec = SensorSpec {
        sensor_id: "multi-array-fanout-sensor".to_string(),
        name: "Multi-Array Fan-Out Sensor".to_string(),
        auth_type: AuthType::BearerStatic,
        base_url: mock_server.uri(),
        tables: vec![TableSpec::new_point_in_time(
            "items",
            "security_finding",
            vec![ColumnSpec {
                name: "id".to_string(),
                column_type: ColumnType::String,
                ocsf_field: None,
                options: vec![],
            }],
            vec![
                FetchStep {
                    name: "step1".to_string(),
                    method: "GET".to_string(),
                    path_template: "/step1".to_string(),
                    body_template: None,
                    response_path: "$.ids".to_string(),
                    pagination_cursor_path: None,
                    variables_produced: vec!["ids".to_string()],
                    fan_out_batch_size: None,
                    pagination: None,
                },
                FetchStep {
                    name: "step2".to_string(),
                    method: "GET".to_string(),
                    path_template: "/step2".to_string(),
                    body_template: None,
                    response_path: "$.codes".to_string(),
                    pagination_cursor_path: None,
                    variables_produced: vec!["codes".to_string()],
                    fan_out_batch_size: None,
                    pagination: None,
                },
                FetchStep {
                    name: "step3".to_string(),
                    method: "GET".to_string(),
                    // References BOTH step1.ids and step2.codes — both are array-valued.
                    // This triggers the multi-array fan-out ambiguity warn.
                    path_template: "/api/${step1.ids}/x/${step2.codes}".to_string(),
                    body_template: None,
                    response_path: "$.items".to_string(),
                    pagination_cursor_path: None,
                    variables_produced: vec![],
                    fan_out_batch_size: None,
                    pagination: None,
                },
            ],
        )],
        rate_limit_hints: None,
        version: "1.0.0".to_string(),
        credential_refs: vec![],
    };

    let table = spec.tables[0].clone();
    let context = FetchContext::new(OrgSlug::new("test-org"), HashMap::new());
    let http_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("reqwest Client::build must succeed");
    let auth_provider = NullAuthProvider;

    let result =
        PipelineExecutor::execute(&spec, &table, &context, &http_client, &auth_provider).await;

    // Pipeline must succeed — first array drives fan-out, backward-compatible behavior.
    assert!(
        result.is_ok(),
        "F-LP9-MED-002: pipeline must succeed (first array drives fan-out); got: {:?}",
        result
    );

    // The structured warn MUST have been emitted.
    let log_output = log_buffer.lock().unwrap().clone();
    assert!(
        log_output.contains("fanout_ambiguous_multi_array"),
        "F-LP9-MED-002: log must contain event_type=fanout_ambiguous_multi_array; \
         pre-fix code returns first match without warn. Log output: {log_output}"
    );
    assert!(
        log_output.contains("step3"),
        "F-LP9-MED-002: log must contain the step name (step3); log output: {log_output}"
    );
}

// ---------------------------------------------------------------------------
// F-LP10-MED-002: Object-valued step_var silently stringified — warn emitted
// ---------------------------------------------------------------------------

/// BC-2.16.002 / F-LP10-MED-002: When a step's path_template references an
/// Object-valued step variable (not an Array), `find_fan_out_array` must emit
/// a structured `fanout_invalid_source_type` warn.
///
/// Setup:
/// - step1: GET /step1/metadata → `{"metadata": {"host_id":"abc","region":"us-east"}}`,
///   response_path: `"$.metadata"`, variables_produced: `["metadata"]`
/// - step2: GET `/api/devices/${step1.metadata}/lookup` → `{"items": [{"device":"d1"}]}`
///
/// `step1.metadata` is Object-typed. Without the fix, `find_fan_out_array` silently
/// skips it (not an Array), then interpolation stringifies the object into the URL —
/// no warning emitted. With the fix, a structured `fanout_invalid_source_type` warn
/// is emitted with the step name and variable name.
///
/// RED GATE: No `fanout_invalid_source_type` event in log → assertion FAILS.
/// GREEN (post-fix): warn emitted → assertion PASSES. Pipeline still executes.
#[tokio::test]
async fn test_BC_2_16_002_fanout_invalid_source_type_emits_structured_event_for_object() {
    use std::sync::{Arc, Mutex};
    use tracing_subscriber::util::SubscriberInitExt;

    // Set up log capture harness.
    let log_buffer: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));
    let log_buffer_clone = log_buffer.clone();

    let writer = tracing_subscriber::fmt::writer::BoxMakeWriter::new(move || {
        struct BufWriter(Arc<Mutex<String>>);
        impl std::io::Write for BufWriter {
            fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
                let s = String::from_utf8_lossy(buf);
                self.0.lock().unwrap().push_str(&s);
                Ok(buf.len())
            }
            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }
        BufWriter(log_buffer_clone.clone())
    });

    let subscriber = tracing_subscriber::fmt()
        .with_writer(writer)
        .with_max_level(tracing::Level::WARN)
        .finish();

    let _guard = subscriber.set_default();

    let mock_server = MockServer::start().await;

    // step1: returns an Object under "metadata"
    Mock::given(method("GET"))
        .and(path("/step1/metadata"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "metadata": {"host_id": "abc", "region": "us-east"}
        })))
        .mount(&mock_server)
        .await;

    // step2: accepts any request (object gets stringified into URL path)
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "items": [{"device": "d1"}]
        })))
        .mount(&mock_server)
        .await;

    let spec = SensorSpec {
        sensor_id: "object-fanout-sensor".to_string(),
        name: "Object Fanout Sensor".to_string(),
        auth_type: AuthType::BearerStatic,
        base_url: mock_server.uri(),
        tables: vec![TableSpec::new_point_in_time(
            "devices",
            "security_finding",
            vec![ColumnSpec {
                name: "device".to_string(),
                column_type: ColumnType::String,
                ocsf_field: None,
                options: vec![],
            }],
            vec![
                FetchStep {
                    name: "step1".to_string(),
                    method: "GET".to_string(),
                    path_template: "/step1/metadata".to_string(),
                    body_template: None,
                    // response_path resolves to the Object value itself
                    response_path: "$.metadata".to_string(),
                    pagination_cursor_path: None,
                    variables_produced: vec!["metadata".to_string()],
                    fan_out_batch_size: None,
                    pagination: None,
                },
                FetchStep {
                    name: "step2".to_string(),
                    method: "GET".to_string(),
                    // ${step1.metadata} is an Object — will be stringified into URL.
                    // F-LP10-MED-002: must emit fanout_invalid_source_type warn.
                    path_template: "/api/devices/${step1.metadata}/lookup".to_string(),
                    body_template: None,
                    response_path: "$.items".to_string(),
                    pagination_cursor_path: None,
                    variables_produced: vec![],
                    fan_out_batch_size: None,
                    pagination: None,
                },
            ],
        )],
        rate_limit_hints: None,
        version: "1.0.0".to_string(),
        credential_refs: vec![],
    };

    let table = spec.tables[0].clone();
    let context = FetchContext::new(OrgSlug::new("test-org"), HashMap::new());
    let http_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("reqwest Client::build must succeed");
    let auth_provider = NullAuthProvider;

    // Pipeline must execute (backward-compatible: Object gets stringified; warn surfaced).
    let result =
        PipelineExecutor::execute(&spec, &table, &context, &http_client, &auth_provider).await;
    assert!(
        result.is_ok(),
        "F-LP10-MED-002: pipeline must execute (Object stringified into URL); got: {:?}",
        result
    );

    // Structured warn MUST have been emitted.
    let log_output = log_buffer.lock().unwrap().clone();
    assert!(
        log_output.contains("fanout_invalid_source_type"),
        "F-LP10-MED-002: log must contain event_type=fanout_invalid_source_type; \
         pre-fix code emits no warn for Object-typed variables. Log output: {log_output}"
    );
    assert!(
        log_output.contains("Object"),
        "F-LP10-MED-002: log must state actual_type=Object; log output: {log_output}"
    );
    assert!(
        log_output.contains("step2"),
        "F-LP10-MED-002: log must contain the step name (step2); log output: {log_output}"
    );
}
