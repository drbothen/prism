//! AC-7: Missing Authorization header returns 401 on all auth-required endpoints (S-6.07).
//!
//! Given a request to any auth-required endpoint without an `Authorization` header,
//! Then the response is HTTP 401 with `{"errors": [{"code": 401, "message": "..."}]}`.
//!
//! Expected Red Gate failure: `CrowdstrikeClone::start()` panics with
//! "not yet implemented".

use prism_dtu_common::BehavioralClone;
use prism_dtu_crowdstrike::CrowdstrikeClone;

/// Helper: assert 401 error body shape.
fn assert_401_error_body(body: &serde_json::Value, context: &str) {
    let errors = body["errors"].as_array().unwrap_or_else(|| {
        panic!("{context}: response body must contain 'errors' array, got: {body}")
    });
    assert!(!errors.is_empty(), "{context}: errors array must not be empty");
    assert_eq!(
        errors[0]["code"].as_u64().unwrap_or(0),
        401,
        "{context}: error code must be 401"
    );
    assert!(
        errors[0]["message"].as_str().is_some(),
        "{context}: error message must be a string"
    );
}

/// AC-7: GET /detects/queries/detects/v1 without Authorization returns 401.
#[tokio::test]
async fn ac_7_detection_list_without_auth_returns_401() {
    // Expected failure: CrowdstrikeClone::start() panics with "not yet implemented".
    let mut clone = CrowdstrikeClone::new();
    clone.start().await.expect("AC-7 det: start() must succeed");

    let base_url = clone.base_url();
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .unwrap();

    let resp = client
        .get(format!("{base_url}/detects/queries/detects/v1"))
        // No Authorization header.
        .send()
        .await
        .expect("AC-7 det: request must reach server");

    assert_eq!(
        resp.status().as_u16(),
        401,
        "AC-7 det: missing Authorization must return HTTP 401 on detection list endpoint"
    );

    let body: serde_json::Value = resp.json().await.expect("AC-7 det: body must be JSON");
    assert_401_error_body(&body, "AC-7 det");
}

/// AC-7: POST /detects/entities/summaries/GET/v1 without Authorization returns 401.
#[tokio::test]
async fn ac_7_detection_summaries_without_auth_returns_401() {
    let mut clone = CrowdstrikeClone::new();
    clone.start().await.expect("AC-7 summ: start() must succeed");

    let base_url = clone.base_url();
    let client = reqwest::Client::new();

    let resp = client
        .post(format!("{base_url}/detects/entities/summaries/GET/v1"))
        .json(&serde_json::json!({"ids": ["det-001"]}))
        .send()
        .await
        .expect("AC-7 summ: request must reach server");

    assert_eq!(
        resp.status().as_u16(),
        401,
        "AC-7 summ: missing Authorization must return HTTP 401 on detection summaries endpoint"
    );

    let body: serde_json::Value = resp.json().await.expect("AC-7 summ: body must be JSON");
    assert_401_error_body(&body, "AC-7 summ");
}

/// AC-7: GET /devices/queries/devices/v1 without Authorization returns 401.
#[tokio::test]
async fn ac_7_host_list_without_auth_returns_401() {
    let mut clone = CrowdstrikeClone::new();
    clone.start().await.expect("AC-7 host: start() must succeed");

    let base_url = clone.base_url();
    let client = reqwest::Client::new();

    let resp = client
        .get(format!("{base_url}/devices/queries/devices/v1"))
        .send()
        .await
        .expect("AC-7 host: request must reach server");

    assert_eq!(
        resp.status().as_u16(),
        401,
        "AC-7 host: missing Authorization must return HTTP 401 on host list endpoint"
    );

    let body: serde_json::Value = resp.json().await.expect("AC-7 host: body must be JSON");
    assert_401_error_body(&body, "AC-7 host");
}

/// AC-7: GET /devices/entities/devices/v2 without Authorization returns 401.
#[tokio::test]
async fn ac_7_host_detail_without_auth_returns_401() {
    let mut clone = CrowdstrikeClone::new();
    clone.start().await.expect("AC-7 hostd: start() must succeed");

    let base_url = clone.base_url();
    let client = reqwest::Client::new();

    let resp = client
        .get(format!("{base_url}/devices/entities/devices/v2"))
        .query(&[("ids", "h-001")])
        .send()
        .await
        .expect("AC-7 hostd: request must reach server");

    assert_eq!(
        resp.status().as_u16(),
        401,
        "AC-7 hostd: missing Authorization must return HTTP 401 on host detail endpoint"
    );

    let body: serde_json::Value = resp.json().await.expect("AC-7 hostd: body must be JSON");
    assert_401_error_body(&body, "AC-7 hostd");
}

/// AC-7: POST /devices/entities/devices-actions/v2 without Authorization returns 401.
#[tokio::test]
async fn ac_7_contain_without_auth_returns_401() {
    let mut clone = CrowdstrikeClone::new();
    clone.start().await.expect("AC-7 contain: start() must succeed");

    let base_url = clone.base_url();
    let client = reqwest::Client::new();

    let resp = client
        .post(format!("{base_url}/devices/entities/devices-actions/v2"))
        .query(&[("action_name", "contain")])
        .json(&serde_json::json!({"ids": ["h-001"]}))
        .send()
        .await
        .expect("AC-7 contain: request must reach server");

    assert_eq!(
        resp.status().as_u16(),
        401,
        "AC-7 contain: missing Authorization must return HTTP 401 on contain endpoint"
    );

    let body: serde_json::Value = resp.json().await.expect("AC-7 contain: body must be JSON");
    assert_401_error_body(&body, "AC-7 contain");
}

/// AC-7: Empty Authorization header (no value) returns 401.
#[tokio::test]
async fn ac_7_empty_authorization_header_returns_401() {
    let mut clone = CrowdstrikeClone::new();
    clone.start().await.expect("AC-7 empty: start() must succeed");

    let base_url = clone.base_url();
    let client = reqwest::Client::new();

    let resp = client
        .get(format!("{base_url}/detects/queries/detects/v1"))
        .header("Authorization", "")
        .send()
        .await
        .expect("AC-7 empty: request must reach server");

    assert_eq!(
        resp.status().as_u16(),
        401,
        "AC-7 empty: empty Authorization value must return HTTP 401"
    );
}

/// AC-7: "Bearer " with no token returns 401.
#[tokio::test]
async fn ac_7_bearer_with_no_token_returns_401() {
    let mut clone = CrowdstrikeClone::new();
    clone.start().await.expect("AC-7 bare: start() must succeed");

    let base_url = clone.base_url();
    let client = reqwest::Client::new();

    let resp = client
        .get(format!("{base_url}/detects/queries/detects/v1"))
        .header("Authorization", "Bearer ")
        .send()
        .await
        .expect("AC-7 bare: request must reach server");

    assert_eq!(
        resp.status().as_u16(),
        401,
        "AC-7 bare: 'Bearer ' with no token must return HTTP 401"
    );
}
