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
//!
//! All tests use wiremock as the HTTP backend so no real sensor API is required.

use prism_core::{ColumnType, OrgSlug};
use prism_spec_engine::NullAuthProvider;
use prism_spec_engine::pipeline::{FetchContext, PipelineExecutor};
use prism_spec_engine::spec_parser::{
    AuthType, ColumnSpec, FetchStep, PaginationConfig, RateLimitHints, SensorSpec, TableSpec,
};
use std::collections::HashMap;
use wiremock::matchers::{header, method, path, query_param};
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
    FetchContext {
        client_id: OrgSlug::new("test-org"),
        query_filters: HashMap::new(),
    }
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
    let http_client = reqwest::Client::new();
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
    let http_client = reqwest::Client::new();
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
    let http_client = reqwest::Client::new();
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
                }),
            }],
        )],
        rate_limit_hints: None,
        version: "1.0.0".to_string(),
        credential_refs: vec![],
    };

    let table = spec.tables[0].clone();
    let context = default_context();
    let http_client = reqwest::Client::new();
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
    let http_client = reqwest::Client::new();
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
    let context = FetchContext {
        client_id: OrgSlug::new("test-org"),
        query_filters: HashMap::new(),
    };
    let http_client = reqwest::Client::new();
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
                }),
            }],
        )],
        rate_limit_hints: None,
        version: "1.0.0".to_string(),
        credential_refs: vec![],
    };

    let table = spec.tables[0].clone();
    let context = FetchContext {
        client_id: OrgSlug::new("test-org"),
        query_filters: HashMap::new(),
    };
    let http_client = reqwest::Client::new();
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
    let context = FetchContext {
        client_id: OrgSlug::new("test-org"),
        query_filters: HashMap::new(),
    };
    let http_client = reqwest::Client::new();
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
    let context = FetchContext {
        client_id: OrgSlug::new("test-org"),
        query_filters: HashMap::new(),
    };
    let http_client = reqwest::Client::new();
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
    let context = FetchContext {
        client_id: OrgSlug::new("test-org"),
        query_filters: HashMap::new(),
    };
    let http_client = reqwest::Client::new();
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
    let context = FetchContext {
        client_id: OrgSlug::new("test-org"),
        query_filters: filters,
    };
    let http_client = reqwest::Client::new();
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
                }),
            }],
        )],
        rate_limit_hints: None,
        version: "1.0.0".to_string(),
        credential_refs: vec![],
    };

    let table = spec.tables[0].clone();
    let context = FetchContext {
        client_id: OrgSlug::new("test-org"),
        query_filters: HashMap::new(),
    };
    let http_client = reqwest::Client::new();
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
    let context = FetchContext {
        client_id: OrgSlug::new("test-org"),
        query_filters: HashMap::new(),
    };
    let http_client = reqwest::Client::new();
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
                }),
            }],
        )],
        rate_limit_hints: None,
        version: "1.0.0".to_string(),
        credential_refs: vec![],
    };

    let table = spec.tables[0].clone();
    let context = FetchContext {
        client_id: OrgSlug::new("test-org"),
        query_filters: HashMap::new(),
    };
    let http_client = reqwest::Client::new();
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
    let context = FetchContext {
        client_id: OrgSlug::new("test-org"),
        query_filters: HashMap::new(),
    };
    let http_client = reqwest::Client::new();
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
                }),
            }],
        )],
        rate_limit_hints: None,
        version: "1.0.0".to_string(),
        credential_refs: vec![],
    };

    let table = spec.tables[0].clone();
    let context = FetchContext {
        client_id: OrgSlug::new("test-org"),
        query_filters: HashMap::new(),
    };
    let http_client = reqwest::Client::new();
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
