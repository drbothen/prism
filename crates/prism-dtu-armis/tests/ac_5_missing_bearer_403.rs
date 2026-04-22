// AC-5: Given a request without `Authorization` header, Then the response is HTTP 403
// `{"error": "invalid or missing bearer token", "code": 403}` — note: Armis uses 403
// not 401, per API spec.
//
// Tests all vendor API endpoints for consistent 403 behavior.
//
// Red Gate: will fail if:
//   - Auth middleware is absent or returns 401 instead of 403.
//   - Error body fields ("error", "code") are missing or wrong type.
//   - DTU internal endpoints (/dtu/*) incorrectly require auth.

#![cfg(feature = "dtu")]

use prism_dtu_armis::ArmisClone;
use prism_dtu_common::BehavioralClone;

#[tokio::test]
async fn ac_5_get_devices_without_auth_returns_403() {
    let mut clone = ArmisClone::new().expect("AC-5: ArmisClone::new() must succeed");
    clone.start().await.expect("AC-5: start() must succeed");
    let base_url = clone.base_url();
    let client = reqwest::Client::new();

    let resp = client
        .get(format!("{base_url}/api/v1/devices"))
        .send()
        .await
        .expect("AC-5: unauthenticated GET /api/v1/devices must be sent");

    assert_eq!(
        resp.status().as_u16(),
        403,
        "AC-5: missing Authorization header must return HTTP 403 (not 401)"
    );

    let body: serde_json::Value = resp.json().await.expect("AC-5: 403 body must be valid JSON");

    let error = body["error"].as_str().unwrap_or("");
    assert!(
        !error.is_empty(),
        "AC-5: 403 response must include 'error' field"
    );
    assert!(
        error.contains("bearer token") || error.contains("missing"),
        "AC-5: error must mention bearer token, got: {error:?}"
    );

    let code = body["code"].as_u64().unwrap_or(0);
    assert_eq!(
        code, 403,
        "AC-5: 403 response body 'code' field must be 403"
    );
}

#[tokio::test]
async fn ac_5_get_alerts_without_auth_returns_403() {
    let mut clone = ArmisClone::new().expect("AC-5: ArmisClone::new() must succeed");
    clone.start().await.expect("AC-5: start() must succeed");
    let base_url = clone.base_url();
    let client = reqwest::Client::new();

    let resp = client
        .get(format!("{base_url}/api/v1/alerts"))
        .send()
        .await
        .expect("AC-5: unauthenticated GET /api/v1/alerts must be sent");

    assert_eq!(
        resp.status().as_u16(),
        403,
        "AC-5: GET /api/v1/alerts without auth must return HTTP 403"
    );

    let body: serde_json::Value = resp.json().await.expect("AC-5: 403 body must be JSON");
    assert!(
        body["error"].is_string(),
        "AC-5: 403 body must have 'error' string field"
    );
    assert_eq!(
        body["code"].as_u64().unwrap_or(0),
        403,
        "AC-5: 403 body 'code' must be 403"
    );
}

#[tokio::test]
async fn ac_5_get_device_activity_without_auth_returns_403() {
    let mut clone = ArmisClone::new().expect("AC-5: ArmisClone::new() must succeed");
    clone.start().await.expect("AC-5: start() must succeed");
    let base_url = clone.base_url();
    let client = reqwest::Client::new();

    let resp = client
        .get(format!("{base_url}/api/v1/devices/d-001/activity"))
        .send()
        .await
        .expect("AC-5: unauthenticated GET /api/v1/devices/d-001/activity must be sent");

    assert_eq!(
        resp.status().as_u16(),
        403,
        "AC-5: activity endpoint without auth must return HTTP 403"
    );
}

#[tokio::test]
async fn ac_5_get_device_risk_without_auth_returns_403() {
    let mut clone = ArmisClone::new().expect("AC-5: ArmisClone::new() must succeed");
    clone.start().await.expect("AC-5: start() must succeed");
    let base_url = clone.base_url();
    let client = reqwest::Client::new();

    let resp = client
        .get(format!("{base_url}/api/v1/devices/d-001/risk"))
        .send()
        .await
        .expect("AC-5: unauthenticated GET risk must be sent");

    assert_eq!(
        resp.status().as_u16(),
        403,
        "AC-5: risk endpoint without auth must return HTTP 403"
    );
}

#[tokio::test]
async fn ac_5_empty_bearer_value_returns_403() {
    // Edge case: "Authorization: Bearer " with no token value (empty after "Bearer ").
    let mut clone = ArmisClone::new().expect("AC-5 empty bearer: ArmisClone::new() must succeed");
    clone.start().await.expect("AC-5 empty bearer: start() must succeed");
    let base_url = clone.base_url();
    let client = reqwest::Client::new();

    let resp = client
        .get(format!("{base_url}/api/v1/devices"))
        .header("Authorization", "Bearer ")
        .send()
        .await
        .expect("AC-5 empty bearer: request must be sent");

    assert_eq!(
        resp.status().as_u16(),
        403,
        "AC-5 empty bearer: 'Bearer ' with no token must return HTTP 403"
    );
}

#[tokio::test]
async fn ac_5_wrong_scheme_returns_403() {
    // "Authorization: Basic ..." instead of Bearer should also return 403.
    let mut clone = ArmisClone::new().expect("AC-5 basic: ArmisClone::new() must succeed");
    clone.start().await.expect("AC-5 basic: start() must succeed");
    let base_url = clone.base_url();
    let client = reqwest::Client::new();

    let resp = client
        .get(format!("{base_url}/api/v1/devices"))
        .header("Authorization", "Basic dXNlcjpwYXNz")
        .send()
        .await
        .expect("AC-5 basic: request with Basic auth must be sent");

    assert_eq!(
        resp.status().as_u16(),
        403,
        "AC-5 basic: Basic auth scheme must return HTTP 403 (Armis requires Bearer)"
    );
}

#[tokio::test]
async fn ac_5_dtu_internal_endpoints_do_not_require_auth() {
    // DTU-internal /dtu/* endpoints must NOT require auth.
    // They are test infrastructure endpoints — no bearer validation.
    let mut clone = ArmisClone::new().expect("AC-5 dtu: ArmisClone::new() must succeed");
    clone.start().await.expect("AC-5 dtu: start() must succeed");
    let base_url = clone.base_url();
    let client = reqwest::Client::new();

    let health = client
        .get(format!("{base_url}/dtu/health"))
        .send()
        .await
        .expect("AC-5 dtu: GET /dtu/health must succeed");

    assert_eq!(
        health.status().as_u16(),
        200,
        "AC-5 dtu: GET /dtu/health must return 200 without auth"
    );

    let aql_log = client
        .get(format!("{base_url}/dtu/aql-log"))
        .send()
        .await
        .expect("AC-5 dtu: GET /dtu/aql-log must succeed");

    assert_eq!(
        aql_log.status().as_u16(),
        200,
        "AC-5 dtu: GET /dtu/aql-log must return 200 without auth"
    );

    let reset = client
        .post(format!("{base_url}/dtu/reset"))
        .send()
        .await
        .expect("AC-5 dtu: POST /dtu/reset must succeed");

    assert_eq!(
        reset.status().as_u16(),
        200,
        "AC-5 dtu: POST /dtu/reset must return 200 without auth"
    );
}
