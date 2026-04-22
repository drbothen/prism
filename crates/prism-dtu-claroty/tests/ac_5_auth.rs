//! AC-5: Any request without Authorization header → HTTP 401 with JSON body.

use prism_dtu_claroty::ClarotyClone;
use prism_dtu_common::BehavioralClone;
use serde_json::json;

async fn start_clone() -> (ClarotyClone, String) {
    let mut clone = ClarotyClone::new();
    clone.start().await.expect("ClarotyClone::start failed");
    let base_url = clone.base_url();
    (clone, base_url)
}

/// AC-5: POST /api/v1/devices with no Authorization → 401.
#[tokio::test]
async fn test_ac5_devices_no_auth_returns_401() {
    let (_clone, base_url) = start_clone().await;
    let client = reqwest::Client::new();

    let resp = client
        .post(format!("{base_url}/api/v1/devices"))
        .json(&json!({}))
        .send()
        .await
        .expect("request failed");

    assert_eq!(resp.status().as_u16(), 401, "missing auth must return 401");
}

/// AC-5: 401 response must have a JSON body.
#[tokio::test]
async fn test_ac5_devices_no_auth_returns_json_body() {
    let (_clone, base_url) = start_clone().await;
    let client = reqwest::Client::new();

    let resp = client
        .post(format!("{base_url}/api/v1/devices"))
        .json(&json!({}))
        .send()
        .await
        .expect("request failed");

    let body: serde_json::Value = resp.json().await.expect("401 body must be valid JSON");
    assert!(
        body.get("error").is_some(),
        "401 body must contain `error` field; got: {body}"
    );
}

/// AC-5: POST /api/v1/alerts with no Authorization → 401.
#[tokio::test]
async fn test_ac5_alerts_no_auth_returns_401() {
    let (_clone, base_url) = start_clone().await;
    let client = reqwest::Client::new();

    let resp = client
        .post(format!("{base_url}/api/v1/alerts"))
        .json(&json!({}))
        .send()
        .await
        .expect("request failed");

    assert_eq!(resp.status().as_u16(), 401, "missing auth must return 401");
}

/// AC-5: POST /api/v1/vulnerabilities with no Authorization → 401.
#[tokio::test]
async fn test_ac5_vulnerabilities_no_auth_returns_401() {
    let (_clone, base_url) = start_clone().await;
    let client = reqwest::Client::new();

    let resp = client
        .post(format!("{base_url}/api/v1/vulnerabilities"))
        .json(&json!({}))
        .send()
        .await
        .expect("request failed");

    assert_eq!(resp.status().as_u16(), 401, "missing auth must return 401");
}

/// AC-5: POST /api/v1/devices/{device_id}/tags/ with no Authorization → 401.
#[tokio::test]
async fn test_ac5_tag_add_no_auth_returns_401() {
    let (_clone, base_url) = start_clone().await;
    let client = reqwest::Client::new();

    let resp = client
        .post(format!("{base_url}/api/v1/devices/asset-001/tags/"))
        .json(&json!({"tag_key": "quarantine", "tag_value": "true"}))
        .send()
        .await
        .expect("request failed");

    assert_eq!(resp.status().as_u16(), 401, "missing auth must return 401");
}

/// AC-5: DELETE /api/v1/devices/{device_id}/tags/{tag_key} with no Authorization → 401.
#[tokio::test]
async fn test_ac5_tag_delete_no_auth_returns_401() {
    let (_clone, base_url) = start_clone().await;
    let client = reqwest::Client::new();

    let resp = client
        .delete(format!("{base_url}/api/v1/devices/asset-001/tags/quarantine"))
        .send()
        .await
        .expect("request failed");

    assert_eq!(resp.status().as_u16(), 401, "missing auth must return 401");
}

/// AC-5: Bearer header with empty token value → 401.
#[tokio::test]
async fn test_ac5_empty_bearer_token_returns_401() {
    let (_clone, base_url) = start_clone().await;
    let client = reqwest::Client::new();

    let resp = client
        .post(format!("{base_url}/api/v1/devices"))
        .header("Authorization", "Bearer ")
        .json(&json!({}))
        .send()
        .await
        .expect("request failed");

    assert_eq!(resp.status().as_u16(), 401, "empty bearer token must return 401");
}

/// AC-5: Non-Bearer scheme (e.g. Basic) → 401.
#[tokio::test]
async fn test_ac5_non_bearer_scheme_returns_401() {
    let (_clone, base_url) = start_clone().await;
    let client = reqwest::Client::new();

    let resp = client
        .post(format!("{base_url}/api/v1/devices"))
        .header("Authorization", "Basic dXNlcjpwYXNz")
        .json(&json!({}))
        .send()
        .await
        .expect("request failed");

    assert_eq!(resp.status().as_u16(), 401, "non-Bearer scheme must return 401");
}
