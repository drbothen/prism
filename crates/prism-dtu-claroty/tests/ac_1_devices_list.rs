//! AC-1: POST /api/v1/devices with valid Bearer, no filters → 200 with 20 device objects.
//!
//! Each device object must contain `device_uid` (mapped to `uid`), `name`/`asset_id`,
//! `ip_list`, and `risk_score` (mapped from `risk_level`).

use prism_dtu_claroty::ClarotyClone;
use prism_dtu_common::BehavioralClone;
use serde_json::json;

/// Helper: start a fresh clone and return (clone, base_url).
async fn start_clone() -> (ClarotyClone, String) {
    let mut clone = ClarotyClone::new();
    clone.start().await.expect("ClarotyClone::start failed");
    let base_url = clone.base_url();
    (clone, base_url)
}

/// AC-1: 200 response with a `devices` array when no body is sent.
#[tokio::test]
async fn test_ac1_devices_list_no_body_returns_200() {
    let (_clone, base_url) = start_clone().await;
    let client = reqwest::Client::new();

    let resp = client
        .post(format!("{base_url}/api/v1/devices"))
        .header("Authorization", "Bearer test-token")
        .send()
        .await
        .expect("request failed");

    assert_eq!(resp.status().as_u16(), 200, "expected HTTP 200");
}

/// AC-1: Response body contains `devices` array.
#[tokio::test]
async fn test_ac1_devices_list_contains_devices_array() {
    let (_clone, base_url) = start_clone().await;
    let client = reqwest::Client::new();

    let resp = client
        .post(format!("{base_url}/api/v1/devices"))
        .header("Authorization", "Bearer test-token")
        .json(&json!({}))
        .send()
        .await
        .expect("request failed");

    let body: serde_json::Value = resp.json().await.expect("body is JSON");
    assert!(
        body.get("devices").is_some(),
        "response must have a `devices` field"
    );
    let devices = body["devices"].as_array().expect("`devices` must be an array");
    assert_eq!(devices.len(), 20, "fixture must contain exactly 20 devices");
}

/// AC-1: Every device object contains the required fields.
#[tokio::test]
async fn test_ac1_devices_list_each_device_has_required_fields() {
    let (_clone, base_url) = start_clone().await;
    let client = reqwest::Client::new();

    let resp = client
        .post(format!("{base_url}/api/v1/devices"))
        .header("Authorization", "Bearer test-token")
        .json(&json!({}))
        .send()
        .await
        .expect("request failed");

    let body: serde_json::Value = resp.json().await.expect("body is JSON");
    let devices = body["devices"].as_array().expect("`devices` must be an array");

    for (i, device) in devices.iter().enumerate() {
        // The canonical UID field from Claroty types is `uid`.
        assert!(
            device.get("uid").is_some(),
            "device[{i}] must have `uid`"
        );
        assert!(
            device.get("ip_list").is_some(),
            "device[{i}] must have `ip_list`"
        );
        assert!(
            device.get("risk_score").is_some(),
            "device[{i}] must have `risk_score`"
        );
        // asset_id is the xDome name identifier.
        assert!(
            device.get("asset_id").is_some(),
            "device[{i}] must have `asset_id`"
        );
    }
}

/// AC-1: Response also contains `total` and `page` fields.
#[tokio::test]
async fn test_ac1_devices_list_contains_total_and_page() {
    let (_clone, base_url) = start_clone().await;
    let client = reqwest::Client::new();

    let resp = client
        .post(format!("{base_url}/api/v1/devices"))
        .header("Authorization", "Bearer test-token")
        .json(&json!({}))
        .send()
        .await
        .expect("request failed");

    let body: serde_json::Value = resp.json().await.expect("body is JSON");
    assert!(body.get("total").is_some(), "response must have `total`");
    assert!(body.get("page").is_some(), "response must have `page`");
}
