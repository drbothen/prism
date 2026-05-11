#![allow(non_snake_case)]
//! VP-PLUGIN-005 / BC-2.16.002 AC-5 — 401 retry integration tests.
//!
//! These tests constitute the Red Gate for S-PLUGIN-PREREQ-B auth-retry behavior.
//! All tests MUST FAIL in the Red Gate state because `PipelineExecutor::execute`
//! body is `todo!()`.
//!
//! Test coverage:
//! - AC-5a: 401 on first request → `acquire_token` called → retry succeeds → non-empty result
//! - AC-5b: 401 on retry too → pipeline aborts with structured error (double-401)

use prism_core::{ColumnType, OrgSlug};
use prism_spec_engine::pipeline::{FetchContext, PipelineExecutor};
use prism_spec_engine::spec_parser::{AuthType, ColumnSpec, FetchStep, SensorSpec, TableSpec};
use prism_spec_engine::MockAuthProvider;
use std::collections::HashMap;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn default_context() -> FetchContext {
    FetchContext {
        client_id: OrgSlug::new("test-org"),
        query_filters: HashMap::new(),
    }
}

/// Build a one-step `SensorSpec` for auth-retry tests.
fn auth_retry_spec(base_url: &str) -> SensorSpec {
    SensorSpec {
        sensor_id: "retry-sensor".to_string(),
        name: "Retry Sensor".to_string(),
        auth_type: AuthType::Oauth2ClientCredentials,
        base_url: base_url.to_string(),
        tables: vec![TableSpec::new_point_in_time(
            "findings",
            "security_finding",
            vec![ColumnSpec {
                name: "id".to_string(),
                column_type: ColumnType::String,
                ocsf_field: None,
                options: vec![],
            }],
            vec![FetchStep {
                name: "fetch_findings".to_string(),
                method: "GET".to_string(),
                path_template: "/api/findings".to_string(),
                body_template: None,
                response_path: "$.findings".to_string(),
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

// ---------------------------------------------------------------------------
// Test 5 (AC-5a): 401 → acquire_token → retry succeeds
// ---------------------------------------------------------------------------

/// BC-2.16.002 AC-5: When the data endpoint returns HTTP 401, `execute` calls
/// `auth_provider.acquire_token` exactly once, then retries ONCE with the new token.
/// The retry succeeds (200) and returns non-empty records.
///
/// Assertions:
/// (a) `MockAuthProvider::calls()` == 1 after completion (acquire_token was called once)
/// (b) `result.records.len() > 0` (retry succeeded)
/// (c) `result.request_count >= 2` (initial 401 + 1 retry)
///
/// FAILS RED: `execute` body is `todo!()`.
#[tokio::test]
async fn test_BC_2_16_002_execute_calls_auth_provider_acquire_token_on_401() {
    let mock_server = MockServer::start().await;

    // First request: HTTP 401 (triggers auth refresh)
    Mock::given(method("GET"))
        .and(path("/api/findings"))
        .respond_with(ResponseTemplate::new(401))
        .up_to_n_times(1)
        .mount(&mock_server)
        .await;

    // Second request (retry after token refresh): HTTP 200 with data
    Mock::given(method("GET"))
        .and(path("/api/findings"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "findings": [
                {"id": "f1", "severity": "critical"},
                {"id": "f2", "severity": "high"}
            ]
        })))
        .up_to_n_times(1)
        .mount(&mock_server)
        .await;

    let spec = auth_retry_spec(&mock_server.uri());
    let table = spec.tables[0].clone();
    let context = default_context();
    let http_client = reqwest::Client::new();
    // MockAuthProvider records every acquire_token call.
    let auth_provider = MockAuthProvider::new("fresh-bearer-token-after-refresh");

    let result = PipelineExecutor::execute(&spec, &table, &context, &http_client, &auth_provider)
        .await
        .expect("AC-5a: retry after 401 must succeed");

    // (a) acquire_token was called once (on 401 receipt)
    assert_eq!(
        auth_provider.calls(),
        1,
        "AC-5a: acquire_token must be called exactly once on 401; called {} times",
        auth_provider.calls()
    );

    // (b) retry produced non-empty records
    assert!(
        !result.records.is_empty(),
        "AC-5a: retry must produce non-empty records; got 0"
    );
    assert_eq!(
        result.records.len(),
        2,
        "AC-5a: 2 findings expected after successful retry; got {}",
        result.records.len()
    );

    // (c) request count reflects initial + retry
    assert!(
        result.request_count >= 2,
        "AC-5a: at least 2 requests (401 + retry); got request_count={}",
        result.request_count
    );
}

// ---------------------------------------------------------------------------
// Test 6 (AC-5b): Double-401 → pipeline aborts
// ---------------------------------------------------------------------------

/// BC-2.16.002 AC-5 abort condition: When the retry ALSO returns HTTP 401,
/// `execute` aborts with a structured `SpecEngineError::AuthRefreshFailed`
/// (wrapped in `PrismError`). No further retries are attempted.
///
/// Assertions:
/// (a) `execute` returns `Err(...)` — not Ok
/// (b) `auth_provider.calls()` == 1 — token was refreshed once before the second 401
/// (c) No infinite retry loop (test must terminate promptly)
///
/// FAILS RED: `execute` body is `todo!()`.
#[tokio::test]
async fn test_BC_2_16_002_execute_aborts_on_double_401() {
    let mock_server = MockServer::start().await;

    // All requests return 401 — both the initial and the retry.
    Mock::given(method("GET"))
        .and(path("/api/findings"))
        .respond_with(ResponseTemplate::new(401))
        .mount(&mock_server)
        .await;

    let spec = auth_retry_spec(&mock_server.uri());
    let table = spec.tables[0].clone();
    let context = default_context();
    let http_client = reqwest::Client::new();
    let auth_provider = MockAuthProvider::new("token-that-wont-work");

    let result = PipelineExecutor::execute(&spec, &table, &context, &http_client, &auth_provider)
        .await;

    // (a) must be an error — double-401 is not recoverable
    assert!(
        result.is_err(),
        "AC-5b: double-401 must produce Err; got Ok with {} records",
        result.as_ref().map(|r| r.records.len()).unwrap_or(0)
    );

    // (b) acquire_token was called once (for the retry; not zero, not two)
    assert_eq!(
        auth_provider.calls(),
        1,
        "AC-5b: acquire_token must be called exactly once before aborting; called {} times",
        auth_provider.calls()
    );
}
