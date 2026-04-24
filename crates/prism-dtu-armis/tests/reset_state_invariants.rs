#![allow(clippy::unwrap_used, clippy::expect_used)]
// Story AC-7: Given `reset()` is called, Then the tag store is cleared, the AQL log is
// cleared, and subsequent device queries return devices with empty `tags` arrays.
//
// Also covers activity and alerts endpoints (shape verification).
//
// Red Gate: these tests assert behavioral state contracts.
// - The reset test will fail if reset() does not clear the tag_store or aql_log.
// - The activity/alert tests will fail if routes return incorrect shapes.

#![cfg(feature = "dtu")]

use prism_dtu_armis::ArmisClone;
use prism_dtu_common::BehavioralClone;

#[tokio::test]
async fn ac_story_7_reset_clears_tag_store_and_aql_log() {
    let mut clone = ArmisClone::new().expect("AC-7: ArmisClone::new() must succeed");
    clone.start().await.expect("AC-7: start() must succeed");
    let base_url = clone.base_url();
    let client = reqwest::Client::new();

    // Step 1: Add a tag to d-001.
    let tag_resp = client
        .post(format!("{base_url}/api/v1/devices/d-001/tags/"))
        .header("Authorization", "Bearer test-token")
        .json(&serde_json::json!({ "tag_key": "pre-reset-tag" }))
        .send()
        .await
        .expect("AC-7: POST tag must succeed");

    assert_eq!(
        tag_resp.status().as_u16(),
        201,
        "AC-7: POST tag must return 201"
    );

    // Step 2: Send a device query with AQL to populate the AQL log.
    client
        .get(format!("{base_url}/api/v1/devices"))
        .query(&[("aql", "in:type=switch")])
        .header("Authorization", "Bearer test-token")
        .send()
        .await
        .expect("AC-7: GET devices with AQL must succeed");

    // Verify AQL log is non-empty before reset.
    let aql_before = client
        .get(format!("{base_url}/dtu/aql-log"))
        .send()
        .await
        .expect("AC-7: GET /dtu/aql-log must succeed");

    let aql_before_body: serde_json::Value = aql_before
        .json()
        .await
        .expect("AC-7: aql-log body must be JSON");

    let aql_before_strings = aql_before_body["aql_strings"]
        .as_array()
        .expect("AC-7: aql_strings must be array before reset");

    assert!(
        !aql_before_strings.is_empty(),
        "AC-7 pre-condition: AQL log must be non-empty before reset"
    );

    // Step 3: Call reset() via POST /dtu/reset.
    let reset_resp = client
        .post(format!("{base_url}/dtu/reset"))
        .send()
        .await
        .expect("AC-7: POST /dtu/reset must succeed");

    assert_eq!(
        reset_resp.status().as_u16(),
        200,
        "AC-7: POST /dtu/reset must return HTTP 200"
    );

    let reset_body: serde_json::Value = reset_resp
        .json()
        .await
        .expect("AC-7: reset response must be valid JSON");

    assert_eq!(
        reset_body["status"].as_str().unwrap_or(""),
        "ok",
        "AC-7: reset response status must be 'ok'"
    );

    // Step 4: Verify AQL log is empty after reset.
    let aql_after = client
        .get(format!("{base_url}/dtu/aql-log"))
        .send()
        .await
        .expect("AC-7: GET /dtu/aql-log after reset must succeed");

    let aql_after_body: serde_json::Value = aql_after
        .json()
        .await
        .expect("AC-7: aql-log body after reset must be JSON");

    let aql_after_strings = aql_after_body["aql_strings"]
        .as_array()
        .expect("AC-7: aql_strings after reset must be array");

    assert!(
        aql_after_strings.is_empty(),
        "AC-7: AQL log must be empty after reset, got: {aql_after_strings:?}"
    );

    // Step 5: Verify d-001 has no tags after reset.
    let devices_resp = client
        .get(format!("{base_url}/api/v1/devices"))
        .query(&[("size", "100")])
        .header("Authorization", "Bearer test-token")
        .send()
        .await
        .expect("AC-7: GET devices after reset must succeed");

    let devices_body: serde_json::Value = devices_resp
        .json()
        .await
        .expect("AC-7: devices body after reset must be JSON");

    let devices = devices_body["data"]["devices"]
        .as_array()
        .expect("AC-7: data.devices must be array after reset");

    let d001 = devices
        .iter()
        .find(|d| d["device_id"].as_str() == Some("d-001"))
        .unwrap_or_else(|| panic!("AC-7: d-001 must be present after reset"));

    let tags = d001["tags"]
        .as_array()
        .expect("AC-7: d-001.tags must be array after reset");

    let tag_values: Vec<&str> = tags.iter().filter_map(|t| t.as_str()).collect();

    assert!(
        !tag_values.contains(&"pre-reset-tag"),
        "AC-7: d-001 must NOT have 'pre-reset-tag' after reset, got: {tag_values:?}"
    );
}

#[tokio::test]
async fn ac_story_7_reset_does_not_remove_fixture_data() {
    // Fixtures (devices, activity, alerts) must remain after reset.
    // Only mutable state (tags, AQL log) is cleared.
    let mut clone = ArmisClone::new().expect("AC-7 fixtures: ArmisClone::new() must succeed");
    clone
        .start()
        .await
        .expect("AC-7 fixtures: start() must succeed");
    let base_url = clone.base_url();
    let client = reqwest::Client::new();

    // Reset.
    client
        .post(format!("{base_url}/dtu/reset"))
        .send()
        .await
        .expect("AC-7 fixtures: POST /dtu/reset must succeed");

    // Devices must still be present.
    let resp = client
        .get(format!("{base_url}/api/v1/devices"))
        .query(&[("size", "100")])
        .header("Authorization", "Bearer test-token")
        .send()
        .await
        .expect("AC-7 fixtures: GET devices must succeed after reset");

    let body: serde_json::Value = resp.json().await.expect("AC-7 fixtures: body must be JSON");

    let total = body["data"]["total"]
        .as_u64()
        .expect("AC-7 fixtures: data.total must be a number");

    assert!(
        total >= 25,
        "AC-7 fixtures: all 25 fixture devices must be present after reset, got total={total}"
    );
}

// ---- Activity and Alerts endpoint shape tests ----

#[tokio::test]
async fn activity_endpoint_returns_200_with_activities_array() {
    let mut clone = ArmisClone::new().expect("activity: ArmisClone::new() must succeed");
    clone.start().await.expect("activity: start() must succeed");
    let base_url = clone.base_url();
    let client = reqwest::Client::new();

    let resp = client
        .get(format!("{base_url}/api/v1/devices/d-001/activity"))
        .header("Authorization", "Bearer test-token")
        .send()
        .await
        .expect("activity: GET /api/v1/devices/d-001/activity must succeed");

    assert_eq!(
        resp.status().as_u16(),
        200,
        "activity: endpoint must return HTTP 200 with valid Bearer"
    );

    let body: serde_json::Value = resp
        .json()
        .await
        .expect("activity: body must be valid JSON");

    assert!(
        body["data"]["activities"].is_array(),
        "activity: data.activities must be present and be an array"
    );

    assert!(
        body["data"]["total"].is_number(),
        "activity: data.total must be present and be a number"
    );
}

#[tokio::test]
async fn alerts_endpoint_returns_200_with_alerts_array() {
    let mut clone = ArmisClone::new().expect("alerts: ArmisClone::new() must succeed");
    clone.start().await.expect("alerts: start() must succeed");
    let base_url = clone.base_url();
    let client = reqwest::Client::new();

    let resp = client
        .get(format!("{base_url}/api/v1/alerts"))
        .header("Authorization", "Bearer test-token")
        .send()
        .await
        .expect("alerts: GET /api/v1/alerts must succeed");

    assert_eq!(
        resp.status().as_u16(),
        200,
        "alerts: endpoint must return HTTP 200 with valid Bearer"
    );

    let body: serde_json::Value = resp.json().await.expect("alerts: body must be valid JSON");

    assert!(
        body["data"]["alerts"].is_array(),
        "alerts: data.alerts must be present and be an array"
    );

    assert!(
        body["data"]["total"].is_number(),
        "alerts: data.total must be present and be a number"
    );

    let total = body["data"]["total"].as_u64().unwrap_or(0);
    assert!(
        total >= 12,
        "alerts: fixture must have at least 12 alerts, got total={total}"
    );
}

#[tokio::test]
async fn alerts_pagination_beyond_last_returns_empty_array() {
    let mut clone = ArmisClone::new().expect("alerts page: ArmisClone::new() must succeed");
    clone
        .start()
        .await
        .expect("alerts page: start() must succeed");
    let base_url = clone.base_url();
    let client = reqwest::Client::new();

    let resp = client
        .get(format!("{base_url}/api/v1/alerts"))
        .query(&[("page", "999"), ("size", "25")])
        .header("Authorization", "Bearer test-token")
        .send()
        .await
        .expect("alerts page: GET /api/v1/alerts page=999 must succeed");

    assert_eq!(
        resp.status().as_u16(),
        200,
        "alerts page: page beyond last must return HTTP 200"
    );

    let body: serde_json::Value = resp.json().await.expect("alerts page: body must be JSON");

    let alerts = body["data"]["alerts"]
        .as_array()
        .expect("alerts page: data.alerts must be array");

    assert!(
        alerts.is_empty(),
        "alerts page: page beyond last must return empty alerts array"
    );

    let total = body["data"]["total"].as_u64().unwrap_or(0);
    assert!(
        total > 0,
        "alerts page: total must reflect fixture size even on empty page"
    );
}

/// Verify that `POST /dtu/reset` also resets the failure mode to None.
///
/// After configuring rate-limit injection, reset must restore normal operation
/// so that subsequent requests succeed (test isolation invariant, per L2 sibling
/// pattern — prism-dtu-cyberint resets all configured modes in reset()).
#[tokio::test]
async fn reset_clears_failure_mode_to_none() {
    let mut clone = ArmisClone::new().expect("reset_failure_mode: ArmisClone::new() must succeed");
    clone
        .start()
        .await
        .expect("reset_failure_mode: start() must succeed");
    let base_url = clone.base_url();
    let client = reqwest::Client::new();

    // Step 1: Configure rate-limit so all requests return 429.
    let configure_resp = client
        .post(format!("{base_url}/dtu/configure"))
        .json(&serde_json::json!({
            "failure_mode": "rate_limit",
            "after_n_requests": 0,
            "retry_after_secs": 1
        }))
        .send()
        .await
        .expect("reset_failure_mode: POST /dtu/configure must succeed");

    assert_eq!(
        configure_resp.status().as_u16(),
        200,
        "reset_failure_mode: configure must return 200"
    );

    // Confirm rate-limit is active: vendor request returns 429.
    let rate_limited_resp = client
        .get(format!("{base_url}/api/v1/devices"))
        .header("Authorization", "Bearer test-token")
        .send()
        .await
        .expect("reset_failure_mode: rate-limited request must be sent");

    assert_eq!(
        rate_limited_resp.status().as_u16(),
        429,
        "reset_failure_mode: pre-condition — request must be rate-limited before reset"
    );

    // Step 2: Reset — must clear failure mode.
    let reset_resp = client
        .post(format!("{base_url}/dtu/reset"))
        .send()
        .await
        .expect("reset_failure_mode: POST /dtu/reset must succeed");

    assert_eq!(
        reset_resp.status().as_u16(),
        200,
        "reset_failure_mode: reset must return 200"
    );

    // Step 3: Verify vendor requests are no longer rate-limited (failure mode is None).
    let normal_resp = client
        .get(format!("{base_url}/api/v1/devices"))
        .header("Authorization", "Bearer test-token")
        .send()
        .await
        .expect("reset_failure_mode: post-reset request must be sent");

    assert_ne!(
        normal_resp.status().as_u16(),
        429,
        "reset_failure_mode: after reset, requests must NOT be rate-limited (failure_mode cleared)"
    );

    assert_eq!(
        normal_resp.status().as_u16(),
        200,
        "reset_failure_mode: after reset, device query must return HTTP 200"
    );
}
