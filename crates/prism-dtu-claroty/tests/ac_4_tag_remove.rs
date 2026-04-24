#![allow(clippy::unwrap_used, clippy::expect_used)]
//! AC-4: DELETE tag after add → device list returns device with tag removed.

use prism_dtu_claroty::ClarotyClone;
use prism_dtu_common::BehavioralClone;
use serde_json::json;

async fn start_clone() -> (ClarotyClone, String) {
    let mut clone = ClarotyClone::new();
    clone.start().await.expect("ClarotyClone::start failed");
    let base_url = clone.base_url();
    (clone, base_url)
}

/// AC-4: DELETE after add returns HTTP 200 with status=removed.
#[tokio::test]
async fn test_ac4_delete_tag_returns_200() {
    let (_clone, base_url) = start_clone().await;
    let client = reqwest::Client::new();

    // First add the tag.
    client
        .post(format!("{base_url}/api/v1/devices/asset-001/tags/"))
        .header("Authorization", "Bearer test-token")
        .json(&json!({"tag_key": "quarantine", "tag_value": "true"}))
        .send()
        .await
        .expect("add tag failed");

    // Now remove it.
    let resp = client
        .delete(format!(
            "{base_url}/api/v1/devices/asset-001/tags/quarantine"
        ))
        .header("Authorization", "Bearer test-token")
        .send()
        .await
        .expect("delete tag failed");

    assert_eq!(
        resp.status().as_u16(),
        200,
        "expected HTTP 200 on tag removal"
    );
    let body: serde_json::Value = resp.json().await.expect("body is JSON");
    assert_eq!(body["status"], "removed", "status must be `removed`");
}

/// AC-4: After deleting the tag, device list no longer includes it.
#[tokio::test]
async fn test_ac4_deleted_tag_absent_from_device_list() {
    let (_clone, base_url) = start_clone().await;
    let client = reqwest::Client::new();

    // Add then immediately remove.
    client
        .post(format!("{base_url}/api/v1/devices/asset-001/tags/"))
        .header("Authorization", "Bearer test-token")
        .json(&json!({"tag_key": "quarantine", "tag_value": "true"}))
        .send()
        .await
        .expect("add tag failed");

    client
        .delete(format!(
            "{base_url}/api/v1/devices/asset-001/tags/quarantine"
        ))
        .header("Authorization", "Bearer test-token")
        .send()
        .await
        .expect("delete tag failed");

    // Verify tag is absent from subsequent list.
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
        .expect("asset-001 not found");

    let tags = device["tags"].as_array().expect("`tags` array");
    assert!(
        !tags
            .iter()
            .any(|t| t == "quarantine"
                || t.get("tag_key").map(|k| k == "quarantine").unwrap_or(false)),
        "tag `quarantine` must be absent after deletion; got: {tags:?}"
    );
}

/// AC-4: Remaining tags on a device are unaffected when one is deleted.
#[tokio::test]
async fn test_ac4_other_tags_unaffected_after_delete() {
    let (_clone, base_url) = start_clone().await;
    let client = reqwest::Client::new();

    // Add two tags.
    for tag in ["quarantine", "critical-asset"] {
        client
            .post(format!("{base_url}/api/v1/devices/asset-003/tags/"))
            .header("Authorization", "Bearer test-token")
            .json(&json!({"tag_key": tag, "tag_value": "true"}))
            .send()
            .await
            .expect("add tag failed");
    }

    // Remove only "quarantine".
    client
        .delete(format!(
            "{base_url}/api/v1/devices/asset-003/tags/quarantine"
        ))
        .header("Authorization", "Bearer test-token")
        .send()
        .await
        .expect("delete tag failed");

    // "critical-asset" must still be present.
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
        .find(|d| d["asset_id"] == "asset-003")
        .expect("asset-003 not found");

    let tags = device["tags"].as_array().expect("`tags` array");
    assert!(
        tags.iter().any(|t| t == "critical-asset"
            || t.get("tag_key")
                .map(|k| k == "critical-asset")
                .unwrap_or(false)),
        "tag `critical-asset` must still be present after deleting `quarantine`; got: {tags:?}"
    );
}
