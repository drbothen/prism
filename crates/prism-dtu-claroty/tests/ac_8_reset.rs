#![allow(clippy::unwrap_used, clippy::expect_used)]
//! AC-8: reset() clears tag store — subsequent device queries return empty `tags` arrays.

use prism_dtu_claroty::ClarotyClone;
use prism_dtu_common::BehavioralClone;
use serde_json::json;

async fn start_clone() -> (ClarotyClone, String) {
    let mut clone = ClarotyClone::new();
    clone.start().await.expect("ClarotyClone::start failed");
    let base_url = clone.base_url();
    (clone, base_url)
}

/// AC-8: POST /dtu/reset returns HTTP 200.
#[tokio::test]
async fn test_ac8_dtu_reset_returns_200() {
    let (_clone, base_url) = start_clone().await;
    let client = reqwest::Client::new();

    let resp = client
        .post(format!("{base_url}/dtu/reset"))
        .send()
        .await
        .expect("reset request failed");

    assert_eq!(resp.status().as_u16(), 200, "dtu/reset must return 200");
}

/// AC-8: After reset, device list returns devices with empty tags arrays.
#[tokio::test]
async fn test_ac8_reset_clears_all_tags() {
    let (_clone, base_url) = start_clone().await;
    let client = reqwest::Client::new();

    // Add tags to multiple devices.
    for (device_id, tag) in [("asset-001", "quarantine"), ("asset-002", "critical-asset")] {
        client
            .post(format!("{base_url}/api/v1/devices/{device_id}/tags/"))
            .header("Authorization", "Bearer test-token")
            .json(&json!({"tag_key": tag, "tag_value": "true"}))
            .send()
            .await
            .expect("add tag failed");
    }

    // Reset.
    client
        .post(format!("{base_url}/dtu/reset"))
        .send()
        .await
        .expect("reset failed");

    // All devices must have empty tags after reset.
    let resp = client
        .post(format!("{base_url}/api/v1/devices"))
        .header("Authorization", "Bearer test-token")
        .json(&json!({}))
        .send()
        .await
        .expect("device list failed");

    let body: serde_json::Value = resp.json().await.expect("body is JSON");
    let devices = body["devices"].as_array().expect("`devices` array");

    for device in devices {
        let tags = device["tags"].as_array().expect("`tags` must be an array");
        assert!(
            tags.is_empty(),
            "all device tags must be empty after reset; device={} tags={tags:?}",
            device["asset_id"]
        );
    }
}

/// AC-8: BehavioralClone::reset() method also clears tag state (not just /dtu/reset).
#[tokio::test]
async fn test_ac8_behavioral_clone_reset_clears_tags() {
    let (clone, base_url) = start_clone().await;
    let client = reqwest::Client::new();

    // Add a tag.
    client
        .post(format!("{base_url}/api/v1/devices/asset-001/tags/"))
        .header("Authorization", "Bearer test-token")
        .json(&json!({"tag_key": "test-tag", "tag_value": "x"}))
        .send()
        .await
        .expect("add tag failed");

    // Reset via BehavioralClone trait method.
    clone.reset().await.expect("BehavioralClone::reset failed");

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
        tags.is_empty(),
        "tags must be empty after BehavioralClone::reset(); got: {tags:?}"
    );
}

/// AC-8: reset also zeroes request counter (configures failure mode correctly after reset).
#[tokio::test]
async fn test_ac8_reset_zeroes_request_counter() {
    let (_clone, base_url) = start_clone().await;
    let client = reqwest::Client::new();

    // Configure rate limit after 2 requests.
    client
        .post(format!("{base_url}/dtu/configure"))
        .json(&json!({"rate_limit_after": 2, "retry_after_secs": 10}))
        .send()
        .await
        .expect("configure failed");

    // Fire 2 requests to hit the limit.
    for _ in 1..=2 {
        let _ = client
            .post(format!("{base_url}/api/v1/devices"))
            .header("Authorization", "Bearer test-token")
            .json(&json!({}))
            .send()
            .await;
    }

    // 3rd request should be 429.
    let before_reset = client
        .post(format!("{base_url}/api/v1/devices"))
        .header("Authorization", "Bearer test-token")
        .json(&json!({}))
        .send()
        .await
        .expect("request failed");
    assert_eq!(
        before_reset.status().as_u16(),
        429,
        "should be rate-limited before reset"
    );

    // Reset clears counter; requests should succeed again.
    client
        .post(format!("{base_url}/dtu/reset"))
        .send()
        .await
        .expect("reset failed");

    let after_reset = client
        .post(format!("{base_url}/api/v1/devices"))
        .header("Authorization", "Bearer test-token")
        .json(&json!({}))
        .send()
        .await
        .expect("request after reset failed");

    assert_eq!(
        after_reset.status().as_u16(),
        200,
        "request after reset must succeed (counter zeroed)"
    );
}
