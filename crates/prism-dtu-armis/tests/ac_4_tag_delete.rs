// AC-4: Given `DELETE /api/v1/devices/{device_id}/tags/ot-critical` after the tag is
// added, Then the response is HTTP 200 `{"status": "removed"}` AND subsequent device
// query returns the device without that tag.
//
// Also covers:
//   EC-003 — DELETE tag that was never added → HTTP 404 `{"error": "tag not found"}`.
//
// Red Gate: will fail until:
//   - DELETE /api/v1/devices/{device_id}/tags/{tag_key} is wired and returns 200.
//   - Tag store correctly removes tags; removed tags absent from subsequent device query.
//   - Deleting non-existent tag returns 404 with error field.

#![cfg(feature = "dtu")]

use prism_dtu_armis::ArmisClone;
use prism_dtu_common::BehavioralClone;

#[tokio::test]
async fn ac_4_delete_tag_returns_200_removed() {
    let mut clone = ArmisClone::new().expect("AC-4: ArmisClone::new() must succeed");
    clone.start().await.expect("AC-4: start() must succeed");
    let base_url = clone.base_url();
    let client = reqwest::Client::new();

    // First add the tag.
    let add_resp = client
        .post(format!("{base_url}/api/v1/devices/d-001/tags/"))
        .header("Authorization", "Bearer test-token")
        .json(&serde_json::json!({ "tag_key": "ot-critical" }))
        .send()
        .await
        .expect("AC-4: POST tag must succeed");

    assert_eq!(
        add_resp.status().as_u16(),
        201,
        "AC-4: POST tag must return 201"
    );

    // Now delete it.
    let del_resp = client
        .delete(format!("{base_url}/api/v1/devices/d-001/tags/ot-critical"))
        .header("Authorization", "Bearer test-token")
        .send()
        .await
        .expect("AC-4: DELETE /api/v1/devices/d-001/tags/ot-critical must succeed");

    assert_eq!(
        del_resp.status().as_u16(),
        200,
        "AC-4: DELETE tag must return HTTP 200"
    );

    let body: serde_json::Value = del_resp
        .json()
        .await
        .expect("AC-4: DELETE response body must be valid JSON");

    assert_eq!(
        body["status"].as_str().unwrap_or(""),
        "removed",
        "AC-4: DELETE response status must be 'removed'"
    );
}

#[tokio::test]
async fn ac_4_device_does_not_have_tag_after_delete() {
    let mut clone = ArmisClone::new().expect("AC-4 state: ArmisClone::new() must succeed");
    clone
        .start()
        .await
        .expect("AC-4 state: start() must succeed");
    let base_url = clone.base_url();
    let client = reqwest::Client::new();

    // Add tag.
    client
        .post(format!("{base_url}/api/v1/devices/d-001/tags/"))
        .header("Authorization", "Bearer test-token")
        .json(&serde_json::json!({ "tag_key": "removable-tag" }))
        .send()
        .await
        .expect("AC-4 state: POST tag must succeed");

    // Delete it.
    client
        .delete(format!(
            "{base_url}/api/v1/devices/d-001/tags/removable-tag"
        ))
        .header("Authorization", "Bearer test-token")
        .send()
        .await
        .expect("AC-4 state: DELETE tag must succeed");

    // Query devices — d-001 must not have the tag.
    let resp = client
        .get(format!("{base_url}/api/v1/devices"))
        .query(&[("size", "100")])
        .header("Authorization", "Bearer test-token")
        .send()
        .await
        .expect("AC-4 state: GET /api/v1/devices must succeed");

    let body: serde_json::Value = resp
        .json()
        .await
        .expect("AC-4 state: body must be valid JSON");

    let devices = body["data"]["devices"]
        .as_array()
        .expect("AC-4 state: data.devices must be array");

    let d001 = devices
        .iter()
        .find(|d| d["device_id"].as_str() == Some("d-001"))
        .unwrap_or_else(|| panic!("AC-4 state: d-001 must be in device list"));

    let tags = d001["tags"]
        .as_array()
        .expect("AC-4 state: d-001.tags must be array");

    let tag_values: Vec<&str> = tags.iter().filter_map(|t| t.as_str()).collect();

    assert!(
        !tag_values.contains(&"removable-tag"),
        "AC-4 state: d-001 must NOT have 'removable-tag' after DELETE, got: {tag_values:?}"
    );
}

#[tokio::test]
async fn ec_003_delete_nonexistent_tag_returns_404() {
    // EC-003: DELETE tag that was never added → HTTP 404 {"error": "tag not found"}.
    let mut clone = ArmisClone::new().expect("EC-003: ArmisClone::new() must succeed");
    clone.start().await.expect("EC-003: start() must succeed");
    let base_url = clone.base_url();
    let client = reqwest::Client::new();

    let resp = client
        .delete(format!(
            "{base_url}/api/v1/devices/d-001/tags/never-added-tag"
        ))
        .header("Authorization", "Bearer test-token")
        .send()
        .await
        .expect("EC-003: DELETE non-existent tag must be sent");

    assert_eq!(
        resp.status().as_u16(),
        404,
        "EC-003: DELETE non-existent tag must return HTTP 404"
    );

    let body: serde_json::Value = resp
        .json()
        .await
        .expect("EC-003: 404 response must be valid JSON");

    let error = body["error"].as_str().unwrap_or("");
    assert!(
        !error.is_empty(),
        "EC-003: 404 response must include an error field"
    );
    assert!(
        error.contains("not found"),
        "EC-003: error must indicate tag not found, got: {error:?}"
    );
}

#[tokio::test]
async fn ac_4_delete_tag_endpoint_requires_bearer_auth() {
    // DELETE without auth must return 403 (not 401).
    let mut clone = ArmisClone::new().expect("AC-4 auth: ArmisClone::new() must succeed");
    clone
        .start()
        .await
        .expect("AC-4 auth: start() must succeed");
    let base_url = clone.base_url();
    let client = reqwest::Client::new();

    let resp = client
        .delete(format!("{base_url}/api/v1/devices/d-001/tags/some-tag"))
        .send()
        .await
        .expect("AC-4 auth: DELETE without auth must be sent");

    assert_eq!(
        resp.status().as_u16(),
        403,
        "AC-4 auth: DELETE tag without auth must return HTTP 403 (not 401)"
    );
}
