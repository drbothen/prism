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

use std::collections::HashMap;

use prism_core::{ColumnType, OrgSlug};
use prism_spec_engine::{
    FailingAuthProvider, MockAuthProvider,
    error::SpecEngineError,
    pipeline::{FetchContext, PipelineExecutor},
    spec_parser::{AuthType, ColumnSpec, FetchStep, SensorSpec, TableSpec},
};
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{method, path},
};

fn default_context() -> FetchContext {
    FetchContext::new(OrgSlug::new("test-org"), HashMap::new(), None)
}

/// Build a one-step `SensorSpec` for auth-retry tests.
fn auth_retry_spec(base_url: &str) -> SensorSpec {
    SensorSpec::new(
        "retry-sensor",
        "Retry Sensor",
        AuthType::Oauth2ClientCredentials,
        base_url,
        vec![TableSpec::new_point_in_time(
            "findings",
            "security_finding",
            vec![ColumnSpec::new("id", ColumnType::String, None, vec![])],
            vec![FetchStep::new(
                "fetch_findings",
                "GET",
                "/api/findings",
                None,
                "$.findings",
                None,
                vec![],
                None,
                None,
            )],
        )],
        None,
        "1.0.0",
        vec![],
    )
}

// ---------------------------------------------------------------------------
// Test 5 (AC-5a): 401 → acquire_token → retry succeeds
// ---------------------------------------------------------------------------

/// BC-2.16.002 AC-5: When the data endpoint returns HTTP 401 mid-pipeline,
/// `execute` calls `auth_provider.acquire_token` a second time (token-expiry refresh)
/// and retries ONCE with the fresh token. The retry succeeds (200).
///
/// With eager-token acquisition (F-LP5-LOW-003 closure), `acquire_token` is called
/// TWICE total: once eagerly at pipeline start, and once on 401 token-expiry refresh.
///
/// Assertions:
/// (a) `MockAuthProvider::calls()` == 2 after completion (1 eager + 1 on-401 refresh)
/// (b) `result.records.len() == 2` (retry succeeded, 2 records returned)
/// (c) `result.request_count >= 2` (initial 401 request + 1 retry request)
#[tokio::test]
async fn test_BC_2_16_002_execute_calls_auth_provider_acquire_token_on_401() {
    let mock_server = MockServer::start().await;

    // First request: HTTP 401 (triggers token-expiry refresh)
    Mock::given(method("GET"))
        .and(path("/api/findings"))
        .respond_with(ResponseTemplate::new(401))
        .up_to_n_times(1)
        .mount(&mock_server)
        .await;

    // Second request (retry after token-expiry refresh): HTTP 200 with data
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
    let http_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("reqwest Client::build must succeed");
    // MockAuthProvider records every acquire_token call.
    let auth_provider = MockAuthProvider::new("fresh-bearer-token-after-refresh");

    let result = PipelineExecutor::execute(&spec, &table, &context, &http_client, &auth_provider)
        .await
        .expect("AC-5a: retry after 401 must succeed");

    // (a) acquire_token was called twice: 1 eager (pipeline start) + 1 on-401 refresh
    assert_eq!(
        auth_provider.calls(),
        2,
        "AC-5a: acquire_token must be called twice (1 eager + 1 on-401 refresh); called {} times",
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

    // (c) request count reflects initial 401 request + retry request
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
/// With eager-token acquisition (F-LP5-LOW-003 closure), `acquire_token` is called
/// TWICE total: once eagerly at pipeline start, and once on the first 401 refresh.
/// The second 401 (on the retry) triggers abort — no third acquire_token call.
///
/// Assertions:
/// (a) `execute` returns `Err(...)` — not Ok
/// (b) `auth_provider.calls()` == 2 (1 eager + 1 on-first-401 refresh; no third call)
/// (c) No infinite retry loop (test must terminate promptly)
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
    let http_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("reqwest Client::build must succeed");
    let auth_provider = MockAuthProvider::new("token-that-wont-work");

    let result =
        PipelineExecutor::execute(&spec, &table, &context, &http_client, &auth_provider).await;

    // (a) must be an error — double-401 is not recoverable
    assert!(
        result.is_err(),
        "AC-5b: double-401 must produce Err; got Ok with {} records",
        result.as_ref().map(|r| r.records.len()).unwrap_or(0)
    );

    // (b) acquire_token called twice: 1 eager at start + 1 on first-401 refresh.
    // No third call — abort fires on the double-401.
    assert_eq!(
        auth_provider.calls(),
        2,
        "AC-5b: acquire_token must be called exactly twice (1 eager + 1 on-401); called {} times",
        auth_provider.calls()
    );
}

// ---------------------------------------------------------------------------
// New Red Gate tests: F-LP5-LOW-003 closure — eager-token semantics
// ---------------------------------------------------------------------------

/// F-LP5-LOW-003 (eager-token): `execute` acquires the bearer token BEFORE issuing
/// the first HTTP request. A single-step pipeline against a 200 endpoint must
/// produce `request_count == 1` (no spurious 401 round-trip on the initial request).
///
/// Prior to F-LP5-LOW-003 closure (lazy-token design), the first request was
/// always sent with an empty token, guaranteeing a 401 and a second request.
/// Eager acquisition eliminates that round-trip.
///
/// Assertions:
/// (a) `result.request_count == 1` — exactly one HTTP request (not 2)
/// (b) `auth_provider.calls() == 1` — acquire_token called once (eagerly, not on 401)
/// (c) `result.records.len() == 2` — non-empty result (pipeline succeeded)
#[tokio::test]
async fn test_BC_2_16_002_execute_acquires_token_eagerly_before_first_request() {
    let mock_server = MockServer::start().await;

    // Single endpoint — returns 200 immediately (no 401 round-trip).
    // With eager-token, the bearer token is set before this request fires,
    // so the server never needs to return 401.
    Mock::given(method("GET"))
        .and(path("/api/findings"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "findings": [
                {"id": "f1", "severity": "critical"},
                {"id": "f2", "severity": "high"}
            ]
        })))
        .mount(&mock_server)
        .await;

    let spec = auth_retry_spec(&mock_server.uri());
    let table = spec.tables[0].clone();
    let context = default_context();
    let http_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("reqwest Client::build must succeed");
    let auth_provider = MockAuthProvider::new("eager-token");

    let result = PipelineExecutor::execute(&spec, &table, &context, &http_client, &auth_provider)
        .await
        .expect("eager-token: single-step 200 pipeline must succeed");

    // (a) exactly one HTTP request — no spurious 401 round-trip
    assert_eq!(
        result.request_count, 1,
        "F-LP5-LOW-003: eager-token must produce request_count=1 (not 2); got {}",
        result.request_count
    );

    // (b) acquire_token called once (eagerly at pipeline start, not on 401)
    assert_eq!(
        auth_provider.calls(),
        1,
        "F-LP5-LOW-003: eager-token must call acquire_token exactly once; called {} times",
        auth_provider.calls()
    );

    // (c) pipeline produced the expected records
    assert_eq!(
        result.records.len(),
        2,
        "F-LP5-LOW-003: 2 records expected; got {}",
        result.records.len()
    );
}

// ---------------------------------------------------------------------------
// F-LP7-MED-002 Red Gate test: auth_initial_failed aborts pipeline immediately
// ---------------------------------------------------------------------------

/// BC-2.16.002 / F-LP7-MED-002: When `acquire_token` fails at pipeline start,
/// `execute` aborts immediately with `SpecEngineError::AuthAcquisitionFailed`
/// and NO HTTP requests are issued to the data endpoint.
///
/// `FailingAuthProvider` always returns `AuthAcquisitionFailed`, giving a clean
/// test surface without needing a real auth endpoint.
///
/// Assertions:
/// (a) `execute` returns `Err(SpecEngineError::AuthAcquisitionFailed { .. })`
/// (b) `auth_provider.calls() == 1` — acquire_token called exactly once before abort
/// (c) Zero HTTP requests made to the mock data server (wiremock `.expect(0)` enforces this)
#[tokio::test]
async fn test_BC_2_16_002_eager_auth_initial_failed_aborts_pipeline_immediately() {
    let mock_server = MockServer::start().await;

    // Mount a mock that would respond if the executor somehow got past auth.
    // `.expect(0)` asserts ZERO requests are made — wiremock enforces this in drop.
    Mock::given(method("GET"))
        .and(path("/api/findings"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"findings": []})))
        .expect(0)
        .mount(&mock_server)
        .await;

    let spec = auth_retry_spec(&mock_server.uri());
    let table = spec.tables[0].clone();
    let context = default_context();
    let http_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .expect("reqwest Client::build must succeed");
    let auth_provider = FailingAuthProvider::new();

    let result =
        PipelineExecutor::execute(&spec, &table, &context, &http_client, &auth_provider).await;

    // (a) must error with AuthAcquisitionFailed
    assert!(
        matches!(result, Err(SpecEngineError::AuthAcquisitionFailed { .. })),
        "F-LP7-MED-002: auth_initial_failed must propagate as AuthAcquisitionFailed; got {:?}",
        result
    );

    // (b) acquire_token called exactly once before abort
    assert_eq!(
        auth_provider.calls(),
        1,
        "F-LP7-MED-002: acquire_token must be called exactly once before abort; called {} times",
        auth_provider.calls()
    );
    // (c) wiremock .expect(0) enforces zero HTTP requests in drop
}

/// F-LP5-LOW-003 (no spurious auth_refresh_triggered): On a legitimate 200 execution,
/// `auth_refresh_triggered` must NOT fire. Only `auth_initial_acquired` fires.
///
/// This test verifies the audit-log semantics: `auth_refresh_triggered` is now reserved
/// for genuine token-expiry mid-pipeline (401 on a request after the initial eager
/// acquisition). The first-request 401 anti-pattern is eliminated.
///
/// Assertions:
/// (a) Pipeline succeeds (200 response, non-empty records)
/// (b) `auth_provider.calls() == 1` — acquire_token called once (no on-401 refresh)
/// (c) `result.request_count == 1` — no 401 round-trip triggered
#[tokio::test]
async fn test_BC_2_16_002_no_auth_refresh_triggered_on_legitimate_execution() {
    let mock_server = MockServer::start().await;

    // 200 on first attempt — no auth refresh needed.
    Mock::given(method("GET"))
        .and(path("/api/findings"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "findings": [{"id": "f3", "severity": "low"}]
        })))
        .mount(&mock_server)
        .await;

    let spec = auth_retry_spec(&mock_server.uri());
    let table = spec.tables[0].clone();
    let context = default_context();
    let http_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("reqwest Client::build must succeed");
    let auth_provider = MockAuthProvider::new("valid-token-no-refresh");

    let result = PipelineExecutor::execute(&spec, &table, &context, &http_client, &auth_provider)
        .await
        .expect("no-refresh path: legitimate 200 execution must succeed");

    // (a) pipeline succeeded with non-empty records
    assert_eq!(
        result.records.len(),
        1,
        "no-refresh path: 1 record expected; got {}",
        result.records.len()
    );

    // (b) acquire_token called exactly once (only the eager initial acquisition)
    // — no on-401 refresh call, proving auth_refresh_triggered did NOT fire
    assert_eq!(
        auth_provider.calls(),
        1,
        "F-LP5-LOW-003: no-refresh path must call acquire_token exactly once (eager only); \
         called {} times — auth_refresh_triggered must NOT fire on legitimate 200 execution",
        auth_provider.calls()
    );

    // (c) exactly one HTTP request (no spurious 401 round-trip)
    assert_eq!(
        result.request_count, 1,
        "F-LP5-LOW-003: no-refresh path must produce request_count=1; got {}",
        result.request_count
    );
}

// ---------------------------------------------------------------------------
// F-PR2-MED-001 Red Gate test: CookieRoundtrip 401 → E-AUTH-004, no retry
// ---------------------------------------------------------------------------

/// BC-2.01.017 EC-017-002 / TV-BC-2.01.017-006 / F-PR2-MED-001:
/// When `PipelineExecutor::execute` receives HTTP 401 on a request from a
/// `CookieRoundtrip` auth sensor, it MUST:
///
/// (a) Surface `E-AUTH-004` in the error (`SpecEngineError::CookieAuthFailed`),
///     NOT `AuthRefreshFailed` (which implies a refresh mechanism that doesn't
///     exist for static API-key auth).
/// (b) Issue exactly ONE HTTP request (no retry — `acquire_token()` on a static
///     cookie provider just re-reads the same key, so retry is provably futile).
/// (c) NOT emit `auth_refresh_triggered` / `auth_refresh_succeeded` /
///     `auth_refresh_double_401` events (those are OAuth2-path only).
///
/// **Precondition:** `auth_type = CookieRoundtrip`, mock HTTP server returns 401.
/// **Postcondition (BC-2.01.017 TV-BC-2.01.017-006):**
///   - `execute` returns `Err(SpecEngineError::CookieAuthFailed { .. })`
///   - `auth_provider.calls() == 1` (eagerly acquired once; NOT called again on 401)
///   - `mock_server` received exactly 1 request
///   - Error string contains "E-AUTH-004"
///   - Error string contains sensor name "cookie-sensor"
///   - Error string contains client ID "test-org"
///
/// Red Gate state: current `issue_request_with_retry` applies the OAuth2 retry
/// path to ALL auth types, so it calls `acquire_token` again (calls == 2) and
/// issues 2 requests, returning `AuthRefreshFailed` (double-401 path).
/// This test FAILS against current code.
#[tokio::test]
async fn test_BC_2_01_017_dtu_401_surfaces_e_auth_004_no_retry() {
    let mock_server = MockServer::start().await;

    // All requests return 401 — simulating a DTU returning 401 on cookie auth.
    // For CookieRoundtrip: the correct behavior is to NOT retry at all.
    // The mock must receive exactly ONE request — wiremock .expect(1) enforces this.
    Mock::given(method("GET"))
        .and(path("/api/alerts"))
        .respond_with(ResponseTemplate::new(401))
        .expect(1) // CookieRoundtrip path: exactly 1 request, no retry
        .mount(&mock_server)
        .await;

    let spec = SensorSpec::new(
        "cookie-sensor",
        "Cookie Sensor",
        AuthType::CookieRoundtrip,
        mock_server.uri(),
        vec![TableSpec::new_point_in_time(
            "alerts",
            "security_finding",
            vec![ColumnSpec::new("id", ColumnType::String, None, vec![])],
            vec![FetchStep::new(
                "fetch_alerts",
                "GET",
                "/api/alerts",
                None,
                "$.alerts",
                None,
                vec![],
                None,
                None,
            )],
        )],
        None,
        "1.0.0",
        vec![],
    );

    let table = spec.tables[0].clone();
    let context = FetchContext::new(OrgSlug::new("test-org"), HashMap::new(), None);
    let http_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("reqwest Client::build must succeed");
    // MockAuthProvider records every acquire_token call.
    // For CookieRoundtrip: acquire_token must be called ONCE (eager at pipeline start)
    // and NOT called again on 401 (no refresh for static auth).
    let auth_provider = MockAuthProvider::new("static-api-key-value");

    let result =
        PipelineExecutor::execute(&spec, &table, &context, &http_client, &auth_provider).await;

    // (a) Must return Err — 401 with CookieRoundtrip is not recoverable
    assert!(
        result.is_err(),
        "TV-BC-2.01.017-006: CookieRoundtrip 401 must produce Err; got Ok with {} records",
        result.as_ref().map(|r| r.records.len()).unwrap_or(0)
    );

    let err = result.unwrap_err();
    let err_str = err.to_string();

    // (b) Error MUST contain "E-AUTH-004" (cookie auth failure code)
    assert!(
        err_str.contains("E-AUTH-004"),
        "TV-BC-2.01.017-006: error must contain 'E-AUTH-004'; got: {err_str}"
    );

    // (c) Error MUST NOT be AuthRefreshFailed — that implies a refresh mechanism
    assert!(
        !matches!(err, SpecEngineError::AuthRefreshFailed { .. }),
        "TV-BC-2.01.017-006: CookieRoundtrip 401 must NOT return AuthRefreshFailed; got: {err_str}"
    );

    // (d) acquire_token called ONCE (eager at start) — NOT called again on 401
    assert_eq!(
        auth_provider.calls(),
        1,
        "TV-BC-2.01.017-006: acquire_token must be called exactly once (eager only, no refresh); \
         called {} times",
        auth_provider.calls()
    );

    // (e) wiremock .expect(1) enforces exactly 1 HTTP request in drop —
    // if the pipeline retried, wiremock would panic at drop with "expected 1, got 2"
}
