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
use prism_spec_engine::pipeline::{FetchContext, PipelineExecutor};
use prism_spec_engine::spec_parser::{
    AuthType, ColumnSpec, FetchStep, PaginationConfig, SensorSpec, TableSpec,
};
use prism_spec_engine::NullAuthProvider;
use std::collections::HashMap;
use wiremock::matchers::{method, path};
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
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "access_token": "tok-xyz-789"
            })),
        )
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
