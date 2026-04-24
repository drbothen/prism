#![allow(clippy::unwrap_used, clippy::expect_used)]
//! AC-2: POST /api/v1/devices with `group_by` → returns grouped values, NOT full device objects.
//!
//! The response shape changes when `group_by` is present. The xDome API collapses
//! device objects into grouped field values only.

use prism_dtu_claroty::ClarotyClone;
use prism_dtu_common::BehavioralClone;
use serde_json::json;

async fn start_clone() -> (ClarotyClone, String) {
    let mut clone = ClarotyClone::new();
    clone.start().await.expect("ClarotyClone::start failed");
    let base_url = clone.base_url();
    (clone, base_url)
}

/// AC-2: group_by=type response does NOT contain a top-level `devices` array of objects.
#[tokio::test]
async fn test_ac2_group_by_does_not_return_full_device_objects() {
    let (_clone, base_url) = start_clone().await;
    let client = reqwest::Client::new();

    let resp = client
        .post(format!("{base_url}/api/v1/devices"))
        .header("Authorization", "Bearer test-token")
        .json(&json!({"group_by": "device_type"}))
        .send()
        .await
        .expect("request failed");

    assert_eq!(resp.status().as_u16(), 200, "expected HTTP 200");
    let body: serde_json::Value = resp.json().await.expect("body is JSON");

    // When group_by is present, the response must NOT be the standard devices list.
    // It returns grouped field values — either a `groups` array or similar collapsed shape.
    // Specifically, individual device-level fields like `uid`, `ip_list` must NOT appear
    // as direct top-level array items.
    let is_full_device_list = body
        .get("devices")
        .and_then(|d| d.as_array())
        .map(|arr| arr.first().map(|d| d.get("uid").is_some()).unwrap_or(false))
        .unwrap_or(false);

    assert!(
        !is_full_device_list,
        "group_by response must not be a list of full device objects with `uid` fields"
    );
}

/// AC-2: group_by response contains a `groups` or equivalent collapsed structure.
#[tokio::test]
async fn test_ac2_group_by_returns_grouped_structure() {
    let (_clone, base_url) = start_clone().await;
    let client = reqwest::Client::new();

    let resp = client
        .post(format!("{base_url}/api/v1/devices"))
        .header("Authorization", "Bearer test-token")
        .json(&json!({"group_by": "device_type"}))
        .send()
        .await
        .expect("request failed");

    let body: serde_json::Value = resp.json().await.expect("body is JSON");

    // Must have `groups` top-level key OR `total` without a full `devices` array.
    let has_groups = body.get("groups").is_some();
    let has_total = body.get("total").is_some();

    assert!(
        has_groups || has_total,
        "group_by response must contain `groups` or `total`, got: {body}"
    );
}

/// AC-2: group_by=device_category also collapses to grouped shape.
#[tokio::test]
async fn test_ac2_group_by_device_category_returns_grouped_shape() {
    let (_clone, base_url) = start_clone().await;
    let client = reqwest::Client::new();

    let resp = client
        .post(format!("{base_url}/api/v1/devices"))
        .header("Authorization", "Bearer test-token")
        .json(&json!({"group_by": "device_category"}))
        .send()
        .await
        .expect("request failed");

    assert_eq!(resp.status().as_u16(), 200, "expected HTTP 200");
    let body: serde_json::Value = resp.json().await.expect("body is JSON");

    // Must have `groups` key, not full device objects.
    assert!(
        body.get("groups").is_some(),
        "group_by=device_category must return `groups` key, got: {body}"
    );
}
