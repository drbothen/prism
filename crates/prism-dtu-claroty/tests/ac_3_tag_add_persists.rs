//! AC-3: Tag add persists — after POST /api/v1/devices/{device_id}/tags/, subsequent
//! device list includes the new tag on that device.

use prism_dtu_claroty::ClarotyClone;
use prism_dtu_common::BehavioralClone;
use serde_json::json;

async fn start_clone() -> (ClarotyClone, String) {
    let mut clone = ClarotyClone::new();
    clone.start().await.expect("ClarotyClone::start failed");
    let base_url = clone.base_url();
    (clone, base_url)
}

/// AC-3: Adding a tag returns HTTP 201 with correct body.
#[tokio::test]
async fn test_ac3_add_tag_returns_201() {
    let (_clone, base_url) = start_clone().await;
    let client = reqwest::Client::new();

    let resp = client
        .post(format!("{base_url}/api/v1/devices/asset-001/tags/"))
        .header("Authorization", "Bearer test-token")
        .json(&json!({"tag_key": "quarantine", "tag_value": "true"}))
        .send()
        .await
        .expect("request failed");

    assert_eq!(resp.status().as_u16(), 201, "expected HTTP 201");
}

/// AC-3: Add tag response body contains device_id, tag_key, and status=added.
#[tokio::test]
async fn test_ac3_add_tag_response_body_correct() {
    let (_clone, base_url) = start_clone().await;
    let client = reqwest::Client::new();

    let resp = client
        .post(format!("{base_url}/api/v1/devices/asset-001/tags/"))
        .header("Authorization", "Bearer test-token")
        .json(&json!({"tag_key": "quarantine", "tag_value": "true"}))
        .send()
        .await
        .expect("request failed");

    let body: serde_json::Value = resp.json().await.expect("body is JSON");
    assert_eq!(body["device_id"], "asset-001", "device_id must match");
    assert_eq!(body["tag_key"], "quarantine", "tag_key must match");
    assert_eq!(body["status"], "added", "status must be `added`");
}

/// AC-3: After adding a tag, subsequent device list returns device with the tag.
#[tokio::test]
async fn test_ac3_tag_persists_in_subsequent_device_list() {
    let (_clone, base_url) = start_clone().await;
    let client = reqwest::Client::new();

    // Add tag to device asset-001.
    client
        .post(format!("{base_url}/api/v1/devices/asset-001/tags/"))
        .header("Authorization", "Bearer test-token")
        .json(&json!({"tag_key": "quarantine", "tag_value": "true"}))
        .send()
        .await
        .expect("add tag failed");

    // Fetch device list and verify the tag appears on asset-001.
    let resp = client
        .post(format!("{base_url}/api/v1/devices"))
        .header("Authorization", "Bearer test-token")
        .json(&json!({}))
        .send()
        .await
        .expect("device list failed");

    let body: serde_json::Value = resp.json().await.expect("body is JSON");
    let devices = body["devices"].as_array().expect("`devices` array");

    let device = devices
        .iter()
        .find(|d| d["asset_id"] == "asset-001")
        .expect("asset-001 not found in device list");

    let tags = device["tags"].as_array().expect("`tags` must be an array");
    assert!(
        tags.iter()
            .any(|t| t == "quarantine"
                || t.get("tag_key").map(|k| k == "quarantine").unwrap_or(false)),
        "tag `quarantine` must appear in asset-001.tags after being added; got: {tags:?}"
    );
}

/// AC-3: Multiple tags on the same device all persist.
#[tokio::test]
async fn test_ac3_multiple_tags_on_same_device_persist() {
    let (_clone, base_url) = start_clone().await;
    let client = reqwest::Client::new();

    // Add two tags.
    for tag in ["quarantine", "critical-asset"] {
        client
            .post(format!("{base_url}/api/v1/devices/asset-002/tags/"))
            .header("Authorization", "Bearer test-token")
            .json(&json!({"tag_key": tag, "tag_value": "true"}))
            .send()
            .await
            .expect("add tag failed");
    }

    let resp = client
        .post(format!("{base_url}/api/v1/devices"))
        .header("Authorization", "Bearer test-token")
        .json(&json!({}))
        .send()
        .await
        .expect("device list failed");

    let body: serde_json::Value = resp.json().await.expect("body is JSON");
    let devices = body["devices"].as_array().expect("`devices` array");

    let device = devices
        .iter()
        .find(|d| d["asset_id"] == "asset-002")
        .expect("asset-002 not found in device list");

    let tags = device["tags"].as_array().expect("`tags` array");
    assert!(
        tags.len() >= 2,
        "asset-002 must have at least 2 tags; got: {tags:?}"
    );
}
