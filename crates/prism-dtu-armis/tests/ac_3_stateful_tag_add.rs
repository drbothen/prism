#![allow(clippy::unwrap_used, clippy::expect_used)]
// AC-3: Given `POST /api/v1/devices/{device_id}/tags/` with `{"tag_key": "ot-critical"}`,
// Then the response is HTTP 201 AND subsequent `GET /api/v1/devices` returns that device
// with `"ot-critical"` in its `tags` array (stateful tagging).
//
// Red Gate: will fail until:
//   - POST /api/v1/devices/{device_id}/tags/ exists, requires Bearer auth, returns 201.
//   - Tag store is persisted in ArmisState and merged into device records at query time.

#![cfg(feature = "dtu")]

use prism_dtu_armis::ArmisClone;
use prism_dtu_common::BehavioralClone;

#[tokio::test]
async fn ac_3_post_tag_returns_201_with_device_id_and_tag_key() {
    let mut clone = ArmisClone::new().expect("AC-3: ArmisClone::new() must succeed");
    clone.start().await.expect("AC-3: start() must succeed");
    let base_url = clone.base_url();
    let client = reqwest::Client::new();

    let resp = client
        .post(format!("{base_url}/api/v1/devices/d-001/tags/"))
        .header("Authorization", "Bearer test-token")
        .json(&serde_json::json!({ "tag_key": "ot-critical" }))
        .send()
        .await
        .expect("AC-3: POST /api/v1/devices/d-001/tags/ must succeed");

    assert_eq!(
        resp.status().as_u16(),
        201,
        "AC-3: POST tag must return HTTP 201"
    );

    let body: serde_json::Value = resp
        .json()
        .await
        .expect("AC-3: response body must be valid JSON");

    assert_eq!(
        body["device_id"].as_str().unwrap_or(""),
        "d-001",
        "AC-3: response device_id must be 'd-001'"
    );

    assert_eq!(
        body["tag_key"].as_str().unwrap_or(""),
        "ot-critical",
        "AC-3: response tag_key must be 'ot-critical'"
    );

    assert_eq!(
        body["status"].as_str().unwrap_or(""),
        "added",
        "AC-3: response status must be 'added'"
    );
}

#[tokio::test]
async fn ac_3_added_tag_appears_in_subsequent_device_query() {
    let mut clone = ArmisClone::new().expect("AC-3 state: ArmisClone::new() must succeed");
    clone
        .start()
        .await
        .expect("AC-3 state: start() must succeed");
    let base_url = clone.base_url();
    let client = reqwest::Client::new();

    // Add tag to device d-001.
    let tag_resp = client
        .post(format!("{base_url}/api/v1/devices/d-001/tags/"))
        .header("Authorization", "Bearer test-token")
        .json(&serde_json::json!({ "tag_key": "ot-critical" }))
        .send()
        .await
        .expect("AC-3 state: POST tag must succeed");

    assert_eq!(
        tag_resp.status().as_u16(),
        201,
        "AC-3 state: POST tag must return 201"
    );

    // Now query devices and check d-001 has the tag.
    let devices_resp = client
        .get(format!("{base_url}/api/v1/devices"))
        .query(&[("size", "100")])
        .header("Authorization", "Bearer test-token")
        .send()
        .await
        .expect("AC-3 state: GET /api/v1/devices must succeed");

    assert_eq!(
        devices_resp.status().as_u16(),
        200,
        "AC-3 state: GET /api/v1/devices must return HTTP 200"
    );

    let body: serde_json::Value = devices_resp
        .json()
        .await
        .expect("AC-3 state: body must be valid JSON");

    let devices = body["data"]["devices"]
        .as_array()
        .expect("AC-3 state: data.devices must be array");

    let d001 = devices
        .iter()
        .find(|d| d["device_id"].as_str() == Some("d-001"))
        .unwrap_or_else(|| panic!("AC-3 state: d-001 must be in device list"));

    let tags = d001["tags"]
        .as_array()
        .expect("AC-3 state: d-001.tags must be an array");

    let tag_values: Vec<&str> = tags.iter().filter_map(|t| t.as_str()).collect();

    assert!(
        tag_values.contains(&"ot-critical"),
        "AC-3 state: d-001 must have 'ot-critical' tag after POST, got: {tag_values:?}"
    );
}

#[tokio::test]
async fn ac_3_tag_endpoint_requires_bearer_auth_returns_403() {
    // POST /api/v1/devices/.../tags/ without auth must return 403, not 401.
    let mut clone = ArmisClone::new().expect("AC-3 auth: ArmisClone::new() must succeed");
    clone
        .start()
        .await
        .expect("AC-3 auth: start() must succeed");
    let base_url = clone.base_url();
    let client = reqwest::Client::new();

    let resp = client
        .post(format!("{base_url}/api/v1/devices/d-001/tags/"))
        .json(&serde_json::json!({ "tag_key": "ot-critical" }))
        .send()
        .await
        .expect("AC-3 auth: request without auth must be sent");

    assert_eq!(
        resp.status().as_u16(),
        403,
        "AC-3 auth: POST tag without auth must return HTTP 403 (not 401)"
    );
}
