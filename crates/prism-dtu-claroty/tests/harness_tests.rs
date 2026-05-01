//! Harness Migration Test Suite — S-3.4.01 Red Gate
//!
//! Migrates all prism-dtu-claroty tests from direct `ClarotyClone::start()` to
//! `prism-dtu-harness`. All existing ACs continue to pass via the harness-hosted
//! clone. New multi-org logical isolation and network cross-creds tests added.
//!
//! # Behavioral Contracts
//!
//! | BC | Title |
//! |----|-------|
//! | BC-3.5.001 | Harness Logical Isolation Invariants |
//! | BC-3.5.002 | Harness Network Isolation Invariants |
//!
//! # Acceptance Criteria
//!
//! | AC | Description |
//! |----|-------------|
//! | AC-001 | All 11 original S-6.08 ACs pass against harness-hosted clone in Logical mode |
//! | AC-002 | fidelity_validator runs against harness clone with checks_failed == 0 |
//! | AC-003 | 2-org logical harness returns pairwise-disjoint device ID sets |
//! | AC-004 | 2-org network harness returns HTTP 401 on cross-org credential mismatch |
//! | AC-005 | All edge case tests (malformed, auth, pagination) continue to pass |
//! | AC-006 | No test directly instantiates ClarotyClone outside the harness |
//!
//! # Test naming
//!
//! `test_BC_S_SS_NNN_xxx()` pattern throughout (Factory TDD spec).
// Allow test-file conventions: expect() in assertions and BC-tracing names.
#![allow(clippy::expect_used, non_snake_case, dead_code)]

use prism_dtu_harness::{DtuType, IsolationMode};
use serde_json::json;

// ============================================================================
// Shared helper: build a single-tenant Claroty harness and return the base URL.
//
// RED: will fail once Claroty is wired into the harness because:
//   - harness.endpoint_for() returns None until Claroty is registered.
//   - The resulting panics are the Red Gate assertions.
// ============================================================================

/// Build a logical-mode Claroty harness for a single tenant and return (harness, base_url).
///
/// All migrated tests use this helper. The test is RED because `endpoint_for` returns
/// `None` until the implementer wires Claroty registration into `HarnessBuilder::build()`.
async fn build_claroty_harness(tenant: &str) -> (prism_dtu_harness::Harness, String) {
    let harness = prism_dtu_harness::Harness::builder()
        .isolation(IsolationMode::Logical)
        .with_customer_overrides(tenant, |spec| {
            spec.dtu_types = vec![DtuType::Claroty];
        })
        .build()
        .await
        .expect("harness build must succeed");

    let addr = harness
        .endpoint_for(tenant, DtuType::Claroty)
        .expect("Claroty endpoint must be registered — implementer must wire Claroty into harness");

    let base_url = format!("http://{addr}");
    (harness, base_url)
}

/// Build a logical-mode Claroty harness and also return the admin token.
async fn build_claroty_harness_with_token(
    tenant: &str,
) -> (prism_dtu_harness::Harness, String, String) {
    let harness = prism_dtu_harness::Harness::builder()
        .isolation(IsolationMode::Logical)
        .with_customer_overrides(tenant, |spec| {
            spec.dtu_types = vec![DtuType::Claroty];
        })
        .build()
        .await
        .expect("harness build must succeed");

    let addr = harness
        .endpoint_for(tenant, DtuType::Claroty)
        .expect("Claroty endpoint must be registered");

    let admin_token = harness
        .admin_token_for(tenant, DtuType::Claroty)
        .expect("admin token must be available for Claroty endpoint")
        .to_string();

    let base_url = format!("http://{addr}");
    (harness, base_url, admin_token)
}

// ============================================================================
// Migrated AC-001 tests (ac_1_devices_list.rs → harness)
// BC-3.5.001 postcondition 1 — query scoped to registered org returns only that
// org's records
// ============================================================================

/// AC-001 (migrated): GET /assets returns HTTP 200 with no request body.
///
/// Replaces: `test_ac1_devices_list_no_body_returns_200`
#[tokio::test]
async fn migrated_test_ac1_devices_list_no_body_returns_200() {
    let (_harness, base_url) = build_claroty_harness("test-tenant").await;
    let client = reqwest::Client::new();

    let resp = client
        .post(format!("{base_url}/api/v1/devices"))
        .header("Authorization", "Bearer test-token")
        .send()
        .await
        .expect("request failed");

    assert_eq!(resp.status().as_u16(), 200, "expected HTTP 200");
}

/// AC-001 (migrated): GET /assets response body contains a `devices` array.
///
/// Replaces: `test_ac1_devices_list_contains_devices_array`
#[tokio::test]
async fn migrated_test_ac1_devices_list_contains_devices_array() {
    let (_harness, base_url) = build_claroty_harness("test-tenant").await;
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
    let devices = body["devices"]
        .as_array()
        .expect("`devices` must be an array");
    assert_eq!(devices.len(), 20, "fixture must contain exactly 20 devices");
}

/// AC-001 (migrated): Each device in the list has the required schema fields.
///
/// Replaces: `test_ac1_devices_list_each_device_has_required_fields`
#[tokio::test]
async fn migrated_test_ac1_devices_list_each_device_has_required_fields() {
    let (_harness, base_url) = build_claroty_harness("test-tenant").await;
    let client = reqwest::Client::new();

    let resp = client
        .post(format!("{base_url}/api/v1/devices"))
        .header("Authorization", "Bearer test-token")
        .json(&json!({}))
        .send()
        .await
        .expect("request failed");

    let body: serde_json::Value = resp.json().await.expect("body is JSON");
    let devices = body["devices"]
        .as_array()
        .expect("`devices` must be an array");

    for (i, device) in devices.iter().enumerate() {
        assert!(device.get("uid").is_some(), "device[{i}] must have `uid`");
        assert!(
            device.get("ip_list").is_some(),
            "device[{i}] must have `ip_list`"
        );
        assert!(
            device.get("risk_score").is_some(),
            "device[{i}] must have `risk_score`"
        );
        assert!(
            device.get("asset_id").is_some(),
            "device[{i}] must have `asset_id`"
        );
    }
}

/// AC-001 (migrated): GET /assets response contains `total` and `page` fields.
///
/// Replaces: `test_ac1_devices_list_contains_total_and_page`
#[tokio::test]
async fn migrated_test_ac1_devices_list_contains_total_and_page() {
    let (_harness, base_url) = build_claroty_harness("test-tenant").await;
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

// ============================================================================
// Migrated AC-002 tests (ac_2_group_by.rs → harness)
// ============================================================================

/// AC-002 (migrated): group_by response does not return full device objects.
///
/// Replaces: `test_ac2_group_by_does_not_return_full_device_objects`
#[tokio::test]
async fn migrated_test_ac2_group_by_does_not_return_full_device_objects() {
    let (_harness, base_url) = build_claroty_harness("test-tenant").await;
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

/// AC-002 (migrated): group_by returns a grouped structure keyed by field.
///
/// Replaces: `test_ac2_group_by_returns_grouped_structure`
#[tokio::test]
async fn migrated_test_ac2_group_by_returns_grouped_structure() {
    let (_harness, base_url) = build_claroty_harness("test-tenant").await;
    let client = reqwest::Client::new();

    let resp = client
        .post(format!("{base_url}/api/v1/devices"))
        .header("Authorization", "Bearer test-token")
        .json(&json!({"group_by": "device_type"}))
        .send()
        .await
        .expect("request failed");

    let body: serde_json::Value = resp.json().await.expect("body is JSON");

    let has_groups = body.get("groups").is_some();
    let has_total = body.get("total").is_some();

    assert!(
        has_groups || has_total,
        "group_by response must contain `groups` or `total`, got: {body}"
    );
}

/// AC-002 (migrated): group_by=device_category returns the expected grouped shape.
///
/// Replaces: `test_ac2_group_by_device_category_returns_grouped_shape`
#[tokio::test]
async fn migrated_test_ac2_group_by_device_category_returns_grouped_shape() {
    let (_harness, base_url) = build_claroty_harness("test-tenant").await;
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

    assert!(
        body.get("groups").is_some(),
        "group_by=device_category must return `groups` key, got: {body}"
    );
}

// ============================================================================
// Migrated AC-003 tests (ac_3_tag_add_persists.rs → harness)
// ============================================================================

/// AC-003 (migrated): POST /tags returns HTTP 201.
///
/// Replaces: `test_ac3_add_tag_returns_201`
#[tokio::test]
async fn migrated_test_ac3_add_tag_returns_201() {
    let (_harness, base_url) = build_claroty_harness("test-tenant").await;
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

/// AC-003 (migrated): POST /tags response body is correct.
///
/// Replaces: `test_ac3_add_tag_response_body_correct`
#[tokio::test]
async fn migrated_test_ac3_add_tag_response_body_correct() {
    let (_harness, base_url) = build_claroty_harness("test-tenant").await;
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

/// AC-003 (migrated): Added tag persists in subsequent device list response.
///
/// Replaces: `test_ac3_tag_persists_in_subsequent_device_list`
#[tokio::test]
async fn migrated_test_ac3_tag_persists_in_subsequent_device_list() {
    let (_harness, base_url) = build_claroty_harness("test-tenant").await;
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

/// AC-003 (migrated): Multiple tags on the same device all persist.
///
/// Replaces: `test_ac3_multiple_tags_on_same_device_persist`
#[tokio::test]
async fn migrated_test_ac3_multiple_tags_on_same_device_persist() {
    let (_harness, base_url) = build_claroty_harness("test-tenant").await;
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

// ============================================================================
// Migrated AC-004 tests (ac_4_tag_remove.rs → harness)
// ============================================================================

/// AC-004 (migrated): DELETE /tags/{id} returns HTTP 200.
///
/// Replaces: `test_ac4_delete_tag_returns_200`
#[tokio::test]
async fn migrated_test_ac4_delete_tag_returns_200() {
    let (_harness, base_url) = build_claroty_harness("test-tenant").await;
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

/// AC-004 (migrated): Deleted tag is absent from subsequent device list.
///
/// Replaces: `test_ac4_deleted_tag_absent_from_device_list`
#[tokio::test]
async fn migrated_test_ac4_deleted_tag_absent_from_device_list() {
    let (_harness, base_url) = build_claroty_harness("test-tenant").await;
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

/// AC-004 (migrated): Other tags remain unaffected after one tag is deleted.
///
/// Replaces: `test_ac4_other_tags_unaffected_after_delete`
#[tokio::test]
async fn migrated_test_ac4_other_tags_unaffected_after_delete() {
    let (_harness, base_url) = build_claroty_harness("test-tenant").await;
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

// ============================================================================
// Migrated AC-005 tests (ac_5_auth.rs → harness)
// ============================================================================

/// AC-005 (migrated): GET /assets without auth header returns HTTP 401.
///
/// Replaces: `test_ac5_devices_no_auth_returns_401`
#[tokio::test]
async fn migrated_test_ac5_devices_no_auth_returns_401() {
    let (_harness, base_url) = build_claroty_harness("test-tenant").await;
    let client = reqwest::Client::new();

    let resp = client
        .post(format!("{base_url}/api/v1/devices"))
        .json(&json!({}))
        .send()
        .await
        .expect("request failed");

    assert_eq!(resp.status().as_u16(), 401, "missing auth must return 401");
}

/// AC-005 (migrated): GET /assets without auth returns JSON error body.
///
/// Replaces: `test_ac5_devices_no_auth_returns_json_body`
#[tokio::test]
async fn migrated_test_ac5_devices_no_auth_returns_json_body() {
    let (_harness, base_url) = build_claroty_harness("test-tenant").await;
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

/// AC-005 (migrated): GET /alerts without auth returns HTTP 401.
///
/// Replaces: `test_ac5_alerts_no_auth_returns_401`
#[tokio::test]
async fn migrated_test_ac5_alerts_no_auth_returns_401() {
    let (_harness, base_url) = build_claroty_harness("test-tenant").await;
    let client = reqwest::Client::new();

    let resp = client
        .post(format!("{base_url}/api/v1/alerts"))
        .json(&json!({}))
        .send()
        .await
        .expect("request failed");

    assert_eq!(resp.status().as_u16(), 401, "missing auth must return 401");
}

/// AC-005 (migrated): GET /vulnerabilities without auth returns HTTP 401.
///
/// Replaces: `test_ac5_vulnerabilities_no_auth_returns_401`
#[tokio::test]
async fn migrated_test_ac5_vulnerabilities_no_auth_returns_401() {
    let (_harness, base_url) = build_claroty_harness("test-tenant").await;
    let client = reqwest::Client::new();

    let resp = client
        .post(format!("{base_url}/api/v1/vulnerabilities"))
        .json(&json!({}))
        .send()
        .await
        .expect("request failed");

    assert_eq!(resp.status().as_u16(), 401, "missing auth must return 401");
}

/// AC-005 (migrated): POST /tags without auth returns HTTP 401.
///
/// Replaces: `test_ac5_tag_add_no_auth_returns_401`
#[tokio::test]
async fn migrated_test_ac5_tag_add_no_auth_returns_401() {
    let (_harness, base_url) = build_claroty_harness("test-tenant").await;
    let client = reqwest::Client::new();

    let resp = client
        .post(format!("{base_url}/api/v1/devices/asset-001/tags/"))
        .json(&json!({"tag_key": "quarantine", "tag_value": "true"}))
        .send()
        .await
        .expect("request failed");

    assert_eq!(resp.status().as_u16(), 401, "missing auth must return 401");
}

/// AC-005 (migrated): DELETE /tags/{id} without auth returns HTTP 401.
///
/// Replaces: `test_ac5_tag_delete_no_auth_returns_401`
#[tokio::test]
async fn migrated_test_ac5_tag_delete_no_auth_returns_401() {
    let (_harness, base_url) = build_claroty_harness("test-tenant").await;
    let client = reqwest::Client::new();

    let resp = client
        .delete(format!(
            "{base_url}/api/v1/devices/asset-001/tags/quarantine"
        ))
        .send()
        .await
        .expect("request failed");

    assert_eq!(resp.status().as_u16(), 401, "missing auth must return 401");
}

/// AC-005 (migrated): Empty bearer token returns HTTP 401.
///
/// Replaces: `test_ac5_empty_bearer_token_returns_401`
#[tokio::test]
async fn migrated_test_ac5_empty_bearer_token_returns_401() {
    let (_harness, base_url) = build_claroty_harness("test-tenant").await;
    let client = reqwest::Client::new();

    let resp = client
        .post(format!("{base_url}/api/v1/devices"))
        .header("Authorization", "Bearer ")
        .json(&json!({}))
        .send()
        .await
        .expect("request failed");

    assert_eq!(
        resp.status().as_u16(),
        401,
        "empty bearer token must return 401"
    );
}

/// AC-005 (migrated): Non-Bearer authorization scheme returns HTTP 401.
///
/// Replaces: `test_ac5_non_bearer_scheme_returns_401`
#[tokio::test]
async fn migrated_test_ac5_non_bearer_scheme_returns_401() {
    let (_harness, base_url) = build_claroty_harness("test-tenant").await;
    let client = reqwest::Client::new();

    let resp = client
        .post(format!("{base_url}/api/v1/devices"))
        .header("Authorization", "Basic dXNlcjpwYXNz")
        .json(&json!({}))
        .send()
        .await
        .expect("request failed");

    assert_eq!(
        resp.status().as_u16(),
        401,
        "non-Bearer scheme must return 401"
    );
}

// ============================================================================
// Migrated AC-006 tests (ac_6_rate_limit.rs → harness)
// ============================================================================

/// AC-006 (migrated): The 6th request returns HTTP 429 (rate limit).
///
/// Replaces: `test_ac6_rate_limit_6th_request_returns_429`
#[tokio::test]
async fn migrated_test_ac6_rate_limit_6th_request_returns_429() {
    let (_harness, base_url, admin_token) = build_claroty_harness_with_token("test-tenant").await;
    let client = reqwest::Client::new();

    // Configure rate limit: reject after 5 requests.
    client
        .post(format!("{base_url}/dtu/configure"))
        .header("X-Admin-Token", &admin_token)
        .json(&json!({"rate_limit_after": 5, "retry_after_secs": 30}))
        .send()
        .await
        .expect("configure failed");

    // Fire 5 successful requests.
    for i in 1..=5 {
        let resp = client
            .post(format!("{base_url}/api/v1/devices"))
            .header("Authorization", "Bearer test-token")
            .json(&json!({}))
            .send()
            .await
            .unwrap_or_else(|_| panic!("request {i} failed"));
        assert!(
            resp.status().as_u16() < 429,
            "request {i} should succeed before rate limit"
        );
    }

    // 6th request must be rate-limited.
    let resp = client
        .post(format!("{base_url}/api/v1/devices"))
        .header("Authorization", "Bearer test-token")
        .json(&json!({}))
        .send()
        .await
        .expect("6th request failed");

    assert_eq!(
        resp.status().as_u16(),
        429,
        "6th request must return HTTP 429"
    );
}

/// AC-006 (migrated): Rate-limit response includes a Retry-After header.
///
/// Replaces: `test_ac6_rate_limit_response_has_retry_after_header`
#[tokio::test]
async fn migrated_test_ac6_rate_limit_response_has_retry_after_header() {
    let (_harness, base_url, admin_token) = build_claroty_harness_with_token("test-tenant").await;
    let client = reqwest::Client::new();

    client
        .post(format!("{base_url}/dtu/configure"))
        .header("X-Admin-Token", &admin_token)
        .json(&json!({"rate_limit_after": 5, "retry_after_secs": 30}))
        .send()
        .await
        .expect("configure failed");

    // Exhaust the quota.
    for _ in 1..=5 {
        let _ = client
            .post(format!("{base_url}/api/v1/devices"))
            .header("Authorization", "Bearer test-token")
            .json(&json!({}))
            .send()
            .await;
    }

    let resp = client
        .post(format!("{base_url}/api/v1/devices"))
        .header("Authorization", "Bearer test-token")
        .json(&json!({}))
        .send()
        .await
        .expect("6th request failed");

    assert_eq!(resp.status().as_u16(), 429, "must be 429");

    let retry_after = resp
        .headers()
        .get("retry-after")
        .expect("Retry-After header must be present on 429")
        .to_str()
        .expect("Retry-After must be valid string");

    assert_eq!(retry_after, "30", "Retry-After must be 30");
}

/// AC-006 (migrated): POST /dtu/configure returns HTTP 200.
///
/// Replaces: `test_ac6_dtu_configure_returns_200`
#[tokio::test]
async fn migrated_test_ac6_dtu_configure_returns_200() {
    let (_harness, base_url, admin_token) = build_claroty_harness_with_token("test-tenant").await;
    let client = reqwest::Client::new();

    let resp = client
        .post(format!("{base_url}/dtu/configure"))
        .header("X-Admin-Token", &admin_token)
        .json(&json!({"rate_limit_after": 10, "retry_after_secs": 60}))
        .send()
        .await
        .expect("configure failed");

    assert_eq!(
        resp.status().as_u16(),
        200,
        "configure endpoint must return 200"
    );
}

// ============================================================================
// Migrated AC-007 tests (ac_7_internal_error.rs → harness)
// ============================================================================

/// AC-007 (migrated): First request in internal_error mode returns HTTP 500.
///
/// Replaces: `test_ac7_internal_error_first_request_returns_500`
#[tokio::test]
async fn migrated_test_ac7_internal_error_first_request_returns_500() {
    let (_harness, base_url, admin_token) = build_claroty_harness_with_token("test-tenant").await;
    let client = reqwest::Client::new();

    // Configure internal error on request #1.
    client
        .post(format!("{base_url}/dtu/configure"))
        .header("X-Admin-Token", &admin_token)
        .json(&json!({"internal_error_at": 1}))
        .send()
        .await
        .expect("configure failed");

    let resp = client
        .post(format!("{base_url}/api/v1/devices"))
        .header("Authorization", "Bearer test-token")
        .json(&json!({}))
        .send()
        .await
        .expect("request failed");

    assert_eq!(
        resp.status().as_u16(),
        500,
        "first request must return HTTP 500"
    );
}

/// AC-007 (migrated): internal_error_at_n only fails the Nth request.
///
/// Replaces: `test_ac7_internal_error_at_n_only_fails_nth_request`
#[tokio::test]
async fn migrated_test_ac7_internal_error_at_n_only_fails_nth_request() {
    let (_harness, base_url, admin_token) = build_claroty_harness_with_token("test-tenant").await;
    let client = reqwest::Client::new();

    // Configure error on request #2.
    client
        .post(format!("{base_url}/dtu/configure"))
        .header("X-Admin-Token", &admin_token)
        .json(&json!({"internal_error_at": 2}))
        .send()
        .await
        .expect("configure failed");

    // First request must succeed.
    let resp1 = client
        .post(format!("{base_url}/api/v1/devices"))
        .header("Authorization", "Bearer test-token")
        .json(&json!({}))
        .send()
        .await
        .expect("request 1 failed");

    assert_eq!(
        resp1.status().as_u16(),
        200,
        "request 1 must succeed before injected error"
    );

    // Second request must return 500.
    let resp2 = client
        .post(format!("{base_url}/api/v1/devices"))
        .header("Authorization", "Bearer test-token")
        .json(&json!({}))
        .send()
        .await
        .expect("request 2 failed");

    assert_eq!(
        resp2.status().as_u16(),
        500,
        "request 2 must return HTTP 500"
    );
}

/// AC-007 (migrated): Reset clears internal_error mode.
///
/// Replaces: `test_ac7_reset_clears_internal_error_mode`
#[tokio::test]
async fn migrated_test_ac7_reset_clears_internal_error_mode() {
    let (_harness, base_url, admin_token) = build_claroty_harness_with_token("test-tenant").await;
    let client = reqwest::Client::new();

    // Configure error on request #1.
    client
        .post(format!("{base_url}/dtu/configure"))
        .header("X-Admin-Token", &admin_token)
        .json(&json!({"internal_error_at": 1}))
        .send()
        .await
        .expect("configure failed");

    // Trigger the error.
    let resp_err = client
        .post(format!("{base_url}/api/v1/devices"))
        .header("Authorization", "Bearer test-token")
        .json(&json!({}))
        .send()
        .await
        .expect("request failed");
    assert_eq!(resp_err.status().as_u16(), 500, "should have errored");

    // Reset.
    client
        .post(format!("{base_url}/dtu/reset"))
        .send()
        .await
        .expect("reset failed");

    // Next request must succeed.
    let resp_ok = client
        .post(format!("{base_url}/api/v1/devices"))
        .header("Authorization", "Bearer test-token")
        .json(&json!({}))
        .send()
        .await
        .expect("request after reset failed");

    assert_eq!(
        resp_ok.status().as_u16(),
        200,
        "request after reset must succeed"
    );
}

// ============================================================================
// Migrated AC-008 tests (ac_8_reset.rs → harness)
// ============================================================================

/// AC-008 (migrated): POST /dtu/reset returns HTTP 200.
///
/// Replaces: `test_ac8_dtu_reset_returns_200`
#[tokio::test]
async fn migrated_test_ac8_dtu_reset_returns_200() {
    let (_harness, base_url) = build_claroty_harness("test-tenant").await;
    let client = reqwest::Client::new();

    let resp = client
        .post(format!("{base_url}/dtu/reset"))
        .send()
        .await
        .expect("reset request failed");

    assert_eq!(resp.status().as_u16(), 200, "dtu/reset must return 200");
}

/// AC-008 (migrated): Reset clears all tags from the store.
///
/// Replaces: `test_ac8_reset_clears_all_tags`
#[tokio::test]
async fn migrated_test_ac8_reset_clears_all_tags() {
    let (_harness, base_url) = build_claroty_harness("test-tenant").await;
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

/// AC-008 (migrated): Behavioral clone reset clears tags (via harness).
///
/// Replaces: `test_ac8_behavioral_clone_reset_clears_tags`
///
/// Note: The harness does not expose `BehavioralClone::reset()` directly.
/// We exercise the equivalent behavior via `POST /dtu/reset` which is what
/// `BehavioralClone::reset()` calls internally.
#[tokio::test]
async fn migrated_test_ac8_behavioral_clone_reset_clears_tags() {
    let (_harness, base_url) = build_claroty_harness("test-tenant").await;
    let client = reqwest::Client::new();

    // Add a tag.
    client
        .post(format!("{base_url}/api/v1/devices/asset-001/tags/"))
        .header("Authorization", "Bearer test-token")
        .json(&json!({"tag_key": "test-tag", "tag_value": "x"}))
        .send()
        .await
        .expect("add tag failed");

    // Reset via POST /dtu/reset (equivalent to BehavioralClone::reset()).
    client
        .post(format!("{base_url}/dtu/reset"))
        .send()
        .await
        .expect("reset failed");

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
        "tags must be empty after /dtu/reset; got: {tags:?}"
    );
}

/// AC-008 (migrated): Reset zeroes the request counter.
///
/// Replaces: `test_ac8_reset_zeroes_request_counter`
#[tokio::test]
async fn migrated_test_ac8_reset_zeroes_request_counter() {
    let (_harness, base_url, admin_token) = build_claroty_harness_with_token("test-tenant").await;
    let client = reqwest::Client::new();

    // Configure rate limit after 2 requests.
    client
        .post(format!("{base_url}/dtu/configure"))
        .header("X-Admin-Token", &admin_token)
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

// ============================================================================
// Migrated edge case tests (edge_cases.rs → harness)
// AC-005: All existing edge case tests continue to pass after migration.
// ============================================================================

/// EC-001 (migrated): Unrecognized filter field is ignored (no error).
///
/// Replaces: `test_ec001_unrecognized_filter_field_ignored`
#[tokio::test]
async fn migrated_test_ec001_unrecognized_filter_field_ignored() {
    let (_harness, base_url) = build_claroty_harness("test-tenant").await;
    let client = reqwest::Client::new();

    let resp = client
        .post(format!("{base_url}/api/v1/devices"))
        .header("Authorization", "Bearer test-token")
        .json(&json!({
            "filter_by": {"unknown_field_xyz": "some_value"},
            "totally_unknown_param": 99
        }))
        .send()
        .await
        .expect("request failed");

    assert_eq!(
        resp.status().as_u16(),
        200,
        "EC-001: unrecognized filter fields must be silently ignored"
    );

    let body: serde_json::Value = resp.json().await.expect("body is JSON");
    let devices = body["devices"].as_array().expect("`devices` array");
    assert_eq!(
        devices.len(),
        20,
        "EC-001: all 20 devices returned when filter is unrecognized"
    );
}

/// EC-002 (migrated): DELETE non-existent tag returns HTTP 404.
///
/// Replaces: `test_ec002_delete_nonexistent_tag_returns_404`
#[tokio::test]
async fn migrated_test_ec002_delete_nonexistent_tag_returns_404() {
    let (_harness, base_url) = build_claroty_harness("test-tenant").await;
    let client = reqwest::Client::new();

    let resp = client
        .delete(format!(
            "{base_url}/api/v1/devices/asset-001/tags/does-not-exist"
        ))
        .header("Authorization", "Bearer test-token")
        .send()
        .await
        .expect("request failed");

    assert_eq!(
        resp.status().as_u16(),
        404,
        "EC-002: deleting a tag that was never added must return 404"
    );
}

/// EC-002 (migrated): DELETE non-existent tag returns error body.
///
/// Replaces: `test_ec002_delete_nonexistent_tag_error_body`
#[tokio::test]
async fn migrated_test_ec002_delete_nonexistent_tag_error_body() {
    let (_harness, base_url) = build_claroty_harness("test-tenant").await;
    let client = reqwest::Client::new();

    let resp = client
        .delete(format!(
            "{base_url}/api/v1/devices/asset-001/tags/nonexistent"
        ))
        .header("Authorization", "Bearer test-token")
        .send()
        .await
        .expect("request failed");

    let body: serde_json::Value = resp.json().await.expect("body is JSON");
    assert_eq!(
        body["error"], "tag not found",
        "EC-002: error message must be `tag not found`; got: {body}"
    );
}

/// EC-002 (migrated): DELETE tag for unknown device returns HTTP 404.
///
/// Replaces: `test_ec002_delete_tag_unknown_device_returns_404`
#[tokio::test]
async fn migrated_test_ec002_delete_tag_unknown_device_returns_404() {
    let (_harness, base_url) = build_claroty_harness("test-tenant").await;
    let client = reqwest::Client::new();

    let resp = client
        .delete(format!(
            "{base_url}/api/v1/devices/unknown-device-xyz/tags/quarantine"
        ))
        .header("Authorization", "Bearer test-token")
        .send()
        .await
        .expect("request failed");

    assert_eq!(
        resp.status().as_u16(),
        404,
        "EC-002: unknown device + tag must return 404"
    );
}

/// EC-003 (migrated): group_by unknown field returns no error.
///
/// Replaces: `test_ec003_group_by_unknown_field_no_error`
#[tokio::test]
async fn migrated_test_ec003_group_by_unknown_field_no_error() {
    let (_harness, base_url) = build_claroty_harness("test-tenant").await;
    let client = reqwest::Client::new();

    let resp = client
        .post(format!("{base_url}/api/v1/devices"))
        .header("Authorization", "Bearer test-token")
        .json(&json!({"group_by": "completely_unknown_field_xyz"}))
        .send()
        .await
        .expect("request failed");

    assert_eq!(
        resp.status().as_u16(),
        200,
        "EC-003: unknown group_by field must not error"
    );
}

/// EC-003 (migrated): group_by unknown field returns valid JSON.
///
/// Replaces: `test_ec003_group_by_unknown_field_returns_valid_json`
#[tokio::test]
async fn migrated_test_ec003_group_by_unknown_field_returns_valid_json() {
    let (_harness, base_url) = build_claroty_harness("test-tenant").await;
    let client = reqwest::Client::new();

    let resp = client
        .post(format!("{base_url}/api/v1/devices"))
        .header("Authorization", "Bearer test-token")
        .json(&json!({"group_by": "completely_unknown_field_xyz"}))
        .send()
        .await
        .expect("request failed");

    let body: serde_json::Value = resp.json().await.expect("body is JSON");
    let has_groups = body.get("groups").is_some();
    let has_devices = body.get("devices").is_some();
    assert!(
        has_groups || has_devices,
        "EC-003: response must contain `groups` or `devices`; got: {body}"
    );
}

/// EC-004 (migrated): Pagination beyond last page returns empty devices array.
///
/// Replaces: `test_ec004_pagination_beyond_last_page_returns_empty`
#[tokio::test]
async fn migrated_test_ec004_pagination_beyond_last_page_returns_empty() {
    let (_harness, base_url) = build_claroty_harness("test-tenant").await;
    let client = reqwest::Client::new();

    let resp = client
        .post(format!("{base_url}/api/v1/devices"))
        .header("Authorization", "Bearer test-token")
        .json(&json!({"page": 9999, "page_size": 20}))
        .send()
        .await
        .expect("request failed");

    assert_eq!(
        resp.status().as_u16(),
        200,
        "beyond-last-page must still return 200"
    );

    let body: serde_json::Value = resp.json().await.expect("body is JSON");
    let devices = body["devices"].as_array().expect("`devices` array");
    assert!(
        devices.is_empty(),
        "EC-004: devices array must be empty when paging beyond last page; got: {devices:?}"
    );
}

/// EC-004 (migrated): Total count unchanged when paging beyond last page.
///
/// Replaces: `test_ec004_total_unchanged_when_paging_beyond_last`
#[tokio::test]
async fn migrated_test_ec004_total_unchanged_when_paging_beyond_last() {
    let (_harness, base_url) = build_claroty_harness("test-tenant").await;
    let client = reqwest::Client::new();

    // First get the total from a normal request.
    let resp_normal = client
        .post(format!("{base_url}/api/v1/devices"))
        .header("Authorization", "Bearer test-token")
        .json(&json!({}))
        .send()
        .await
        .expect("normal request failed");
    let normal_body: serde_json::Value = resp_normal.json().await.expect("body is JSON");
    let total = normal_body["total"]
        .as_u64()
        .expect("`total` must be numeric");

    // Now page beyond last page.
    let resp_oob = client
        .post(format!("{base_url}/api/v1/devices"))
        .header("Authorization", "Bearer test-token")
        .json(&json!({"page": 9999, "page_size": 20}))
        .send()
        .await
        .expect("out-of-bounds request failed");

    let oob_body: serde_json::Value = resp_oob.json().await.expect("body is JSON");
    let oob_total = oob_body["total"].as_u64().expect("`total` must be numeric");

    assert_eq!(
        total, oob_total,
        "EC-004: `total` must equal {total} even when devices is empty; got {oob_total}"
    );
}

/// EC-004 (migrated): Offset beyond fixture returns empty.
///
/// Replaces: `test_ec004_offset_beyond_fixture_returns_empty`
#[tokio::test]
async fn migrated_test_ec004_offset_beyond_fixture_returns_empty() {
    let (_harness, base_url) = build_claroty_harness("test-tenant").await;
    let client = reqwest::Client::new();

    let resp = client
        .post(format!("{base_url}/api/v1/devices"))
        .header("Authorization", "Bearer test-token")
        .json(&json!({"offset": 10000, "limit": 20}))
        .send()
        .await
        .expect("request failed");

    assert_eq!(resp.status().as_u16(), 200, "must return 200");
    let body: serde_json::Value = resp.json().await.expect("body is JSON");
    let devices = body["devices"].as_array().expect("`devices` array");
    assert!(
        devices.is_empty(),
        "EC-004: devices must be empty when offset beyond fixture; got: {devices:?}"
    );
}

/// EC-005 (migrated): 422 failure mode returns HTTP 422.
///
/// Replaces: `test_ec005_422_failure_mode_returns_422`
#[tokio::test]
async fn migrated_test_ec005_422_failure_mode_returns_422() {
    let (_harness, base_url, admin_token) = build_claroty_harness_with_token("test-tenant").await;
    let client = reqwest::Client::new();

    // Configure 422 injection.
    client
        .post(format!("{base_url}/dtu/configure"))
        .header("X-Admin-Token", &admin_token)
        .json(&json!({"unprocessable_at": 1}))
        .send()
        .await
        .expect("configure failed");

    let resp = client
        .post(format!("{base_url}/api/v1/devices"))
        .header("Authorization", "Bearer test-token")
        .json(&json!({}))
        .send()
        .await
        .expect("request failed");

    assert_eq!(
        resp.status().as_u16(),
        422,
        "EC-005: FailureLayer 422 mode must return HTTP 422"
    );
}

/// EC-006 (migrated): Latency layer delays response by configured ms.
///
/// Replaces: `test_ec006_latency_layer_delays_response`
#[tokio::test]
async fn migrated_test_ec006_latency_layer_delays_response() {
    let (_harness, base_url, admin_token) = build_claroty_harness_with_token("test-tenant").await;
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .expect("build client");

    // Configure 100ms artificial latency.
    client
        .post(format!("{base_url}/dtu/configure"))
        .header("X-Admin-Token", &admin_token)
        .json(&json!({"latency_ms": 100}))
        .send()
        .await
        .expect("configure failed");

    let t0 = std::time::Instant::now();
    let resp = client
        .post(format!("{base_url}/api/v1/devices"))
        .header("Authorization", "Bearer test-token")
        .json(&json!({}))
        .send()
        .await
        .expect("request failed");
    let elapsed = t0.elapsed();

    assert_eq!(
        resp.status().as_u16(),
        200,
        "should still return 200 with latency"
    );
    assert!(
        elapsed.as_millis() >= 90,
        "EC-006: elapsed {elapsed:?} must be >= 90ms when latency_ms=100"
    );
}

/// EC-006 (migrated): Zero latency causes no observable delay.
///
/// Replaces: `test_ec006_zero_latency_no_delay`
#[tokio::test]
async fn migrated_test_ec006_zero_latency_no_delay() {
    let (_harness, base_url) = build_claroty_harness("test-tenant").await;
    let client = reqwest::Client::new();

    let t0 = std::time::Instant::now();
    let resp = client
        .post(format!("{base_url}/api/v1/devices"))
        .header("Authorization", "Bearer test-token")
        .json(&json!({}))
        .send()
        .await
        .expect("request failed");
    let elapsed = t0.elapsed();

    assert_eq!(resp.status().as_u16(), 200, "must return 200");
    assert!(
        elapsed.as_millis() < 2000,
        "EC-006: zero latency response must be fast; got {elapsed:?}"
    );
}

/// Edge case (migrated): GET /alerts returns HTTP 200 with alerts array.
///
/// Replaces: `test_alerts_list_returns_200_with_alerts_array`
#[tokio::test]
async fn migrated_test_alerts_list_returns_200_with_alerts_array() {
    let (_harness, base_url) = build_claroty_harness("test-tenant").await;
    let client = reqwest::Client::new();

    let resp = client
        .post(format!("{base_url}/api/v1/alerts"))
        .header("Authorization", "Bearer test-token")
        .json(&json!({}))
        .send()
        .await
        .expect("request failed");

    assert_eq!(resp.status().as_u16(), 200, "alerts list must return 200");
    let body: serde_json::Value = resp.json().await.expect("body is JSON");
    assert!(body.get("alerts").is_some(), "response must have `alerts`");
    let alerts = body["alerts"].as_array().expect("`alerts` must be array");
    assert_eq!(alerts.len(), 10, "fixture must contain exactly 10 alerts");
}

/// Edge case (migrated): GET /vulnerabilities returns HTTP 200 with vulns array.
///
/// Replaces: `test_vulnerabilities_list_returns_200_with_vulns_array`
#[tokio::test]
async fn migrated_test_vulnerabilities_list_returns_200_with_vulns_array() {
    let (_harness, base_url) = build_claroty_harness("test-tenant").await;
    let client = reqwest::Client::new();

    let resp = client
        .post(format!("{base_url}/api/v1/vulnerabilities"))
        .header("Authorization", "Bearer test-token")
        .json(&json!({}))
        .send()
        .await
        .expect("request failed");

    assert_eq!(
        resp.status().as_u16(),
        200,
        "vulnerabilities list must return 200"
    );
    let body: serde_json::Value = resp.json().await.expect("body is JSON");
    assert!(
        body.get("vulnerabilities").is_some(),
        "response must have `vulnerabilities`"
    );
    let vulns = body["vulnerabilities"]
        .as_array()
        .expect("`vulnerabilities` array");
    assert_eq!(
        vulns.len(),
        15,
        "fixture must contain exactly 15 vulnerabilities"
    );
}

/// Edge case (migrated): GET /alerts/{id} returns associated devices.
///
/// Replaces: `test_alerts_by_id_returns_devices`
#[tokio::test]
async fn migrated_test_alerts_by_id_returns_devices() {
    let (_harness, base_url) = build_claroty_harness("test-tenant").await;
    let client = reqwest::Client::new();

    let resp = client
        .post(format!("{base_url}/api/v1/alerts/1/devices"))
        .header("Authorization", "Bearer test-token")
        .json(&json!({}))
        .send()
        .await
        .expect("request failed");

    assert_eq!(
        resp.status().as_u16(),
        200,
        "alerted devices must return 200"
    );
    let body: serde_json::Value = resp.json().await.expect("body is JSON");
    assert!(
        body.get("devices").is_some(),
        "response must have `devices`"
    );
    assert!(body.get("total").is_some(), "response must have `total`");
}

/// Edge case (migrated): GET /vulnerabilities/{id} returns associated devices.
///
/// Replaces: `test_vulnerability_by_id_returns_devices`
#[tokio::test]
async fn migrated_test_vulnerability_by_id_returns_devices() {
    let (_harness, base_url) = build_claroty_harness("test-tenant").await;
    let client = reqwest::Client::new();

    let resp = client
        .post(format!(
            "{base_url}/api/v1/vulnerabilities/vuln-001/devices"
        ))
        .header("Authorization", "Bearer test-token")
        .json(&json!({}))
        .send()
        .await
        .expect("request failed");

    assert_eq!(
        resp.status().as_u16(),
        200,
        "vulnerability devices must return 200"
    );
    let body: serde_json::Value = resp.json().await.expect("body is JSON");
    assert!(
        body.get("devices").is_some(),
        "response must have `devices`"
    );
    assert!(body.get("total").is_some(), "response must have `total`");
}

// ============================================================================
// Migrated TD tests (td_wv0_04, td_wv0_07 → harness)
// ============================================================================

/// TD-WV0-04 (migrated): POST /dtu/configure with known field returns HTTP 200.
///
/// Replaces: `configure_known_field_returns_200`
#[tokio::test]
async fn migrated_configure_known_field_returns_200() {
    let (_harness, base_url, admin_token) = build_claroty_harness_with_token("test-tenant").await;
    let client = reqwest::Client::new();

    let resp = client
        .post(format!("{base_url}/dtu/configure"))
        .header("X-Admin-Token", &admin_token)
        .json(&serde_json::json!({"latency_ms": 0}))
        .send()
        .await
        .expect("request must succeed");

    assert_eq!(resp.status(), 200, "known field must return 200");
}

/// TD-WV0-04 (migrated): POST /dtu/configure with unknown field returns HTTP 400.
///
/// Replaces: `configure_unknown_field_returns_400`
#[tokio::test]
async fn migrated_configure_unknown_field_returns_400() {
    let (_harness, base_url, admin_token) = build_claroty_harness_with_token("test-tenant").await;
    let client = reqwest::Client::new();

    let resp = client
        .post(format!("{base_url}/dtu/configure"))
        .header("X-Admin-Token", &admin_token)
        .json(&serde_json::json!({"bogus": "val"}))
        .send()
        .await
        .expect("request must succeed");

    assert_eq!(
        resp.status(),
        400,
        "unknown field must return 400 Bad Request, not silently accept"
    );
}

/// TD-WV0-07 (migrated): POST /dtu/configure without token returns HTTP 401.
///
/// Replaces: `configure_without_token_returns_401`
#[tokio::test]
async fn migrated_configure_without_token_returns_401() {
    let (_harness, base_url) = build_claroty_harness("test-tenant").await;
    let client = reqwest::Client::new();

    let resp = client
        .post(format!("{base_url}/dtu/configure"))
        .json(&serde_json::json!({"latency_ms": 0}))
        .send()
        .await
        .expect("request must succeed");

    assert_eq!(
        resp.status(),
        401,
        "TD-WV0-07: missing X-Admin-Token must return 401"
    );
}

/// TD-WV0-07 (migrated): POST /dtu/configure with wrong token returns HTTP 401.
///
/// Replaces: `configure_with_wrong_token_returns_401`
#[tokio::test]
async fn migrated_configure_with_wrong_token_returns_401() {
    let (_harness, base_url) = build_claroty_harness("test-tenant").await;
    let client = reqwest::Client::new();

    let resp = client
        .post(format!("{base_url}/dtu/configure"))
        .header("X-Admin-Token", "wrong-token-that-will-never-match")
        .json(&serde_json::json!({"latency_ms": 0}))
        .send()
        .await
        .expect("request must succeed");

    assert_eq!(
        resp.status(),
        401,
        "TD-WV0-07: incorrect X-Admin-Token must return 401"
    );
}

/// TD-WV0-07 (migrated): POST /dtu/configure with correct token returns HTTP 200.
///
/// Replaces: `configure_with_correct_token_returns_200`
#[tokio::test]
async fn migrated_configure_with_correct_token_returns_200() {
    let (_harness, base_url, admin_token) = build_claroty_harness_with_token("test-tenant").await;
    let client = reqwest::Client::new();

    let resp = client
        .post(format!("{base_url}/dtu/configure"))
        .header("X-Admin-Token", &admin_token)
        .json(&serde_json::json!({"latency_ms": 0}))
        .send()
        .await
        .expect("request must succeed");

    assert_eq!(
        resp.status(),
        200,
        "TD-WV0-07: correct X-Admin-Token must return 200"
    );
}

// ============================================================================
// Migrated fidelity_validator test (fidelity_validator.rs → harness)
// AC-002: fidelity_validator runs against harness clone, checks_failed == 0.
// BC-3.5.001 precondition 3 — all clone tasks running and bound to assigned ports.
// ============================================================================

/// AC-002 (migrated): FidelityValidator runs against the harness-hosted clone
/// and reports `checks_failed == 0` for all Claroty endpoints.
///
/// The `base_url` now comes from `harness.endpoints()` rather than a hardcoded
/// localhost address. (EC-002 guard)
///
/// Replaces: `claroty_dtu_fidelity`
#[tokio::test]
async fn migrated_claroty_dtu_fidelity() {
    use prism_dtu_common::{FidelityCheck, FidelityValidator};

    let (_harness, base_url, admin_token) = build_claroty_harness_with_token("test-tenant").await;

    let checks = vec![
        // Route 1: POST /api/v1/devices — no auth → 401
        FidelityCheck {
            endpoint: "/api/v1/devices".to_string(),
            method: http::Method::POST,
            body: Some(json!({})),
            expected_status: 401,
            required_fields: vec!["error".to_string()],
            ..Default::default()
        },
        // Route 2: POST /api/v1/alerts — no auth → 401
        FidelityCheck {
            endpoint: "/api/v1/alerts".to_string(),
            method: http::Method::POST,
            body: Some(json!({})),
            expected_status: 401,
            required_fields: vec!["error".to_string()],
            ..Default::default()
        },
        // Route 3: POST /api/v1/alerts/:id/devices — no auth → 401
        FidelityCheck {
            endpoint: "/api/v1/alerts/1/devices".to_string(),
            method: http::Method::POST,
            body: Some(json!({})),
            expected_status: 401,
            required_fields: vec!["error".to_string()],
            ..Default::default()
        },
        // Route 4: POST /api/v1/vulnerabilities — no auth → 401
        FidelityCheck {
            endpoint: "/api/v1/vulnerabilities".to_string(),
            method: http::Method::POST,
            body: Some(json!({})),
            expected_status: 401,
            required_fields: vec!["error".to_string()],
            ..Default::default()
        },
        // Route 5: POST /api/v1/vulnerabilities/:id/devices — no auth → 401
        FidelityCheck {
            endpoint: "/api/v1/vulnerabilities/vuln-001/devices".to_string(),
            method: http::Method::POST,
            body: Some(json!({})),
            expected_status: 401,
            required_fields: vec!["error".to_string()],
            ..Default::default()
        },
        // Route 6: POST /api/v1/devices/:id/tags/ — no auth → 401
        FidelityCheck {
            endpoint: "/api/v1/devices/asset-001/tags/".to_string(),
            method: http::Method::POST,
            body: Some(json!({"tag_key": "fidelity-tag", "tag_value": "true"})),
            expected_status: 401,
            required_fields: vec!["error".to_string()],
            ..Default::default()
        },
        // Route 7: DELETE /api/v1/devices/:id/tags/:key — no auth → 401
        FidelityCheck {
            endpoint: "/api/v1/devices/asset-001/tags/fidelity-tag".to_string(),
            method: http::Method::DELETE,
            body: None,
            expected_status: 401,
            required_fields: vec!["error".to_string()],
            ..Default::default()
        },
        // Route 8: POST /dtu/configure — with admin token → 200
        FidelityCheck {
            endpoint: "/dtu/configure".to_string(),
            method: http::Method::POST,
            body: Some(json!({"rate_limit_after": 100})),
            expected_status: 200,
            required_fields: vec![],
            headers: vec![("X-Admin-Token".to_string(), admin_token.clone())],
        },
        // Route 9: POST /dtu/reset — no auth required → 200
        FidelityCheck {
            endpoint: "/dtu/reset".to_string(),
            method: http::Method::POST,
            body: None,
            expected_status: 200,
            required_fields: vec![],
            ..Default::default()
        },
        // Route 10: GET /dtu/health — no auth required → 200
        FidelityCheck {
            endpoint: "/dtu/health".to_string(),
            method: http::Method::GET,
            body: None,
            expected_status: 200,
            required_fields: vec!["status".to_string()],
            ..Default::default()
        },
    ];

    let report = FidelityValidator::run(&base_url, checks).await;

    if report.checks_failed > 0 {
        for failure in &report.failures {
            eprintln!(
                "FIDELITY FAILURE [{}]: {}",
                failure.endpoint, failure.reason
            );
        }
    }

    assert_eq!(
        report.checks_failed,
        0,
        "FidelityValidator: {}/{} checks passed, {} failed",
        report.checks_passed,
        report.checks_passed + report.checks_failed,
        report.checks_failed,
    );
}

// ============================================================================
// NEW — AC-003: Multi-org logical isolation (Task 3)
// BC-3.5.001 postcondition 2; TV-2
// VP-123 — disjoint device ID sets across orgs
// ============================================================================

/// AC-003: 2-org logical harness returns pairwise-disjoint device ID sets.
///
/// Build a 2-org harness with `IsolationMode::Logical`:
///   - "test-tenant" (Claroty)
///   - "other-tenant" (Claroty)
///
/// Query each org independently and assert the device ID sets are disjoint
/// (no shared IDs across org boundaries).
///
/// (BC-3.5.001 postcondition 2; TV-2)
#[tokio::test]
async fn ac_multi_org_logical_isolation() {
    let harness = prism_dtu_harness::Harness::builder()
        .isolation(IsolationMode::Logical)
        .with_customer_overrides("test-tenant", |spec| {
            spec.dtu_types = vec![DtuType::Claroty];
            spec.seed = 1;
        })
        .with_customer_overrides("other-tenant", |spec| {
            spec.dtu_types = vec![DtuType::Claroty];
            spec.seed = 2;
        })
        .build()
        .await
        .expect("2-org logical harness build must succeed");

    let test_addr = harness
        .endpoint_for("test-tenant", DtuType::Claroty)
        .expect(
            "test-tenant Claroty endpoint must exist — implementer must wire Claroty into harness",
        );
    let other_addr = harness
        .endpoint_for("other-tenant", DtuType::Claroty)
        .expect("other-tenant Claroty endpoint must exist");

    let client = reqwest::Client::new();

    // Fetch devices for test-tenant.
    let resp_a = client
        .post(format!("http://{test_addr}/api/v1/devices"))
        .header("Authorization", "Bearer test-token")
        .json(&json!({}))
        .send()
        .await
        .expect("test-tenant device list failed");
    assert_eq!(resp_a.status().as_u16(), 200, "test-tenant must return 200");
    let body_a: serde_json::Value = resp_a.json().await.expect("body is JSON");
    let devices_a = body_a["devices"]
        .as_array()
        .expect("`devices` array for test-tenant");

    // Collect device IDs for test-tenant.
    let ids_a: std::collections::HashSet<String> = devices_a
        .iter()
        .filter_map(|d| d["asset_id"].as_str().map(|s| s.to_owned()))
        .collect();
    assert!(
        !ids_a.is_empty(),
        "test-tenant must have at least one device (AC-003; BC-3.5.001 postcondition 2)"
    );

    // Fetch devices for other-tenant.
    let resp_b = client
        .post(format!("http://{other_addr}/api/v1/devices"))
        .header("Authorization", "Bearer test-token")
        .json(&json!({}))
        .send()
        .await
        .expect("other-tenant device list failed");
    assert_eq!(
        resp_b.status().as_u16(),
        200,
        "other-tenant must return 200"
    );
    let body_b: serde_json::Value = resp_b.json().await.expect("body is JSON");
    let devices_b = body_b["devices"]
        .as_array()
        .expect("`devices` array for other-tenant");

    // Collect device IDs for other-tenant.
    let ids_b: std::collections::HashSet<String> = devices_b
        .iter()
        .filter_map(|d| d["asset_id"].as_str().map(|s| s.to_owned()))
        .collect();
    assert!(
        !ids_b.is_empty(),
        "other-tenant must have at least one device (AC-003)"
    );

    // Assert pairwise disjoint: no ID appears in both sets.
    let intersection: std::collections::HashSet<&String> = ids_a.intersection(&ids_b).collect();
    assert!(
        intersection.is_empty(),
        "AC-003 (BC-3.5.001 postcondition 2; VP-123): device ID sets must be pairwise disjoint \
         across org boundaries; shared IDs found: {intersection:?}"
    );
}

// ============================================================================
// NEW — AC-004: Network isolation cross-creds 401 (Task 4)
// BC-3.5.002 postcondition 2; TV-3
// VP-124 — cross-org credential mismatch returns HTTP 401
// ============================================================================

/// AC-004: 2-org network harness returns HTTP 401 on cross-org credential mismatch.
///
/// Build a 2-org harness with `IsolationMode::Network`:
///   - "test-tenant" (Claroty)
///   - "other-tenant" (Claroty)
///
/// Send "test-tenant" credentials to "other-tenant"'s endpoint and assert the
/// response is HTTP 401, not HTTP 200.
///
/// (BC-3.5.002 postcondition 2; TV-3)
#[tokio::test]
async fn ac_network_cross_creds_401() {
    let harness = prism_dtu_harness::Harness::builder()
        .isolation(IsolationMode::Network)
        .with_customer_overrides("test-tenant", |spec| {
            spec.dtu_types = vec![DtuType::Claroty];
            spec.seed = 10;
        })
        .with_customer_overrides("other-tenant", |spec| {
            spec.dtu_types = vec![DtuType::Claroty];
            spec.seed = 20;
        })
        .build()
        .await
        .expect("2-org network harness build must succeed");

    // Obtain test-tenant's admin token (which is NOT valid for other-tenant).
    let test_tenant_admin_token = harness
        .admin_token_for("test-tenant", DtuType::Claroty)
        .expect("test-tenant admin token must exist in network-mode harness")
        .to_string();

    // Get other-tenant's endpoint address.
    let other_addr = harness
        .endpoint_for("other-tenant", DtuType::Claroty)
        .expect(
            "other-tenant Claroty endpoint must exist — implementer must wire Network mode Claroty",
        );

    let client = reqwest::Client::new();

    // Send test-tenant's bearer token to other-tenant's endpoint.
    // In Network mode each clone has its own distinct token; cross-org token must be rejected.
    let resp = client
        .post(format!("http://{other_addr}/api/v1/devices"))
        .header("Authorization", format!("Bearer {test_tenant_admin_token}"))
        .json(&json!({}))
        .send()
        .await
        .expect("cross-org request must not fail at network level");

    assert_eq!(
        resp.status().as_u16(),
        401,
        "AC-004 (BC-3.5.002 postcondition 2; VP-124): sending test-tenant's token to \
         other-tenant's endpoint must return HTTP 401, not HTTP 200. \
         Network isolation requires per-org distinct credentials."
    );
}

// ============================================================================
// Compile-time guard: verify the harness import resolves
// ============================================================================

/// Compile-time guard: ensures `DtuType::Claroty` and `IsolationMode::Logical`
/// are in scope from `prism_dtu_harness`. If this fails to compile, the
/// `prism-dtu-harness` dev-dependency or `dtu` feature is misconfigured.
#[allow(dead_code)]
fn _compile_guard_harness_types() {
    let _dtu = DtuType::Claroty;
    let _iso = IsolationMode::Logical;
}
