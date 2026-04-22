//! Edge cases for prism-dtu-claroty (EC-001 through EC-006).
//!
//! | ID | Description |
//! |----|-------------|
//! | EC-001 | Unrecognized filter field in POST body → ignored, normal response |
//! | EC-002 | DELETE tag that was never added → 404 `{"error": "tag not found"}` |
//! | EC-003 | group_by with non-device field → full objects returned, no error |
//! | EC-004 | Pagination beyond last page → empty `devices` array, `total` unchanged |
//! | EC-005 | 422 simulation via FailureLayer → maps to E-SENSOR-004 |
//! | EC-006 | Latency simulation via LatencyLayer → elapsed time ≥ configured delay |

use prism_dtu_claroty::ClarotyClone;
use prism_dtu_common::BehavioralClone;
use serde_json::json;
use std::time::Instant;

async fn start_clone() -> (ClarotyClone, String) {
    let mut clone = ClarotyClone::new();
    clone.start().await.expect("ClarotyClone::start failed");
    let base_url = clone.base_url();
    (clone, base_url)
}

// ---------------------------------------------------------------------------
// EC-001: Unrecognized filter field is ignored
// ---------------------------------------------------------------------------

/// EC-001: POST with an unknown filter field returns HTTP 200 (permissive API).
#[tokio::test]
async fn test_ec001_unrecognized_filter_field_ignored() {
    let (_clone, base_url) = start_clone().await;
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

// ---------------------------------------------------------------------------
// EC-002: DELETE tag never added → 404
// ---------------------------------------------------------------------------

/// EC-002: DELETE a tag that was never added → HTTP 404.
#[tokio::test]
async fn test_ec002_delete_nonexistent_tag_returns_404() {
    let (_clone, base_url) = start_clone().await;
    let client = reqwest::Client::new();

    let resp = client
        .delete(format!("{base_url}/api/v1/devices/asset-001/tags/does-not-exist"))
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

/// EC-002: 404 response body contains `{"error": "tag not found"}`.
#[tokio::test]
async fn test_ec002_delete_nonexistent_tag_error_body() {
    let (_clone, base_url) = start_clone().await;
    let client = reqwest::Client::new();

    let resp = client
        .delete(format!("{base_url}/api/v1/devices/asset-001/tags/nonexistent"))
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

/// EC-002: DELETE on unknown device (tag store has no entry) → 404.
#[tokio::test]
async fn test_ec002_delete_tag_unknown_device_returns_404() {
    let (_clone, base_url) = start_clone().await;
    let client = reqwest::Client::new();

    let resp = client
        .delete(format!("{base_url}/api/v1/devices/unknown-device-xyz/tags/quarantine"))
        .header("Authorization", "Bearer test-token")
        .send()
        .await
        .expect("request failed");

    assert_eq!(resp.status().as_u16(), 404, "EC-002: unknown device + tag must return 404");
}

// ---------------------------------------------------------------------------
// EC-003: group_by with non-device field → full objects, no error
// ---------------------------------------------------------------------------

/// EC-003: group_by with a non-device field → HTTP 200, no error.
#[tokio::test]
async fn test_ec003_group_by_unknown_field_no_error() {
    let (_clone, base_url) = start_clone().await;
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

/// EC-003: group_by with a non-device field returns a valid JSON response.
#[tokio::test]
async fn test_ec003_group_by_unknown_field_returns_valid_json() {
    let (_clone, base_url) = start_clone().await;
    let client = reqwest::Client::new();

    let resp = client
        .post(format!("{base_url}/api/v1/devices"))
        .header("Authorization", "Bearer test-token")
        .json(&json!({"group_by": "completely_unknown_field_xyz"}))
        .send()
        .await
        .expect("request failed");

    let body: serde_json::Value = resp.json().await.expect("body is JSON");
    // Must have either groups (empty) or devices (full objects) — either is valid per EC-003.
    let has_groups = body.get("groups").is_some();
    let has_devices = body.get("devices").is_some();
    assert!(
        has_groups || has_devices,
        "EC-003: response must contain `groups` or `devices`; got: {body}"
    );
}

// ---------------------------------------------------------------------------
// EC-004: Pagination beyond last page → empty devices array, total unchanged
// ---------------------------------------------------------------------------

/// EC-004: Requesting a page beyond the last page returns empty `devices` array.
#[tokio::test]
async fn test_ec004_pagination_beyond_last_page_returns_empty() {
    let (_clone, base_url) = start_clone().await;
    let client = reqwest::Client::new();

    // Page 9999 with page_size 20 is well beyond the 20-device fixture.
    let resp = client
        .post(format!("{base_url}/api/v1/devices"))
        .header("Authorization", "Bearer test-token")
        .json(&json!({"page": 9999, "page_size": 20}))
        .send()
        .await
        .expect("request failed");

    assert_eq!(resp.status().as_u16(), 200, "beyond-last-page must still return 200");

    let body: serde_json::Value = resp.json().await.expect("body is JSON");
    let devices = body["devices"].as_array().expect("`devices` array");
    assert!(
        devices.is_empty(),
        "EC-004: devices array must be empty when paging beyond last page; got: {devices:?}"
    );
}

/// EC-004: Total count is unchanged when paging beyond last page.
#[tokio::test]
async fn test_ec004_total_unchanged_when_paging_beyond_last() {
    let (_clone, base_url) = start_clone().await;
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
    let total = normal_body["total"].as_u64().expect("`total` must be numeric");

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

/// EC-004: offset beyond fixture size also returns empty.
#[tokio::test]
async fn test_ec004_offset_beyond_fixture_returns_empty() {
    let (_clone, base_url) = start_clone().await;
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

// ---------------------------------------------------------------------------
// EC-005: 422 simulation via FailureLayer
// ---------------------------------------------------------------------------

/// EC-005: Configuring 422 failure mode makes next request return HTTP 422.
///
/// Note: 422 is simulated via the DTU configure endpoint. The spec notes this maps to
/// E-SENSOR-004 in Prism's error taxonomy (invalid filter syntax from sensor).
#[tokio::test]
async fn test_ec005_422_failure_mode_returns_422() {
    let (_clone, base_url) = start_clone().await;
    let client = reqwest::Client::new();

    // Configure 422 injection (auth_mode="unprocessable" or a dedicated field).
    // Using a generic payload — implementation must honour this.
    client
        .post(format!("{base_url}/dtu/configure"))
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

// ---------------------------------------------------------------------------
// EC-006: Latency simulation via LatencyLayer
// ---------------------------------------------------------------------------

/// EC-006: Configured latency ≥ latency_ms is observable by the caller.
#[tokio::test]
async fn test_ec006_latency_layer_delays_response() {
    let (_clone, base_url) = start_clone().await;
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .expect("build client");

    // Configure 100ms artificial latency.
    client
        .post(format!("{base_url}/dtu/configure"))
        .json(&json!({"latency_ms": 100}))
        .send()
        .await
        .expect("configure failed");

    let t0 = Instant::now();
    let resp = client
        .post(format!("{base_url}/api/v1/devices"))
        .header("Authorization", "Bearer test-token")
        .json(&json!({}))
        .send()
        .await
        .expect("request failed");
    let elapsed = t0.elapsed();

    assert_eq!(resp.status().as_u16(), 200, "should still return 200 with latency");
    assert!(
        elapsed.as_millis() >= 90,
        "EC-006: elapsed {elapsed:?} must be ≥ 90ms when latency_ms=100"
    );
}

/// EC-006: Zero latency (default) does not add artificial delay.
#[tokio::test]
async fn test_ec006_zero_latency_no_delay() {
    let (_clone, base_url) = start_clone().await;
    let client = reqwest::Client::new();

    let t0 = Instant::now();
    let resp = client
        .post(format!("{base_url}/api/v1/devices"))
        .header("Authorization", "Bearer test-token")
        .json(&json!({}))
        .send()
        .await
        .expect("request failed");
    let elapsed = t0.elapsed();

    assert_eq!(resp.status().as_u16(), 200, "must return 200");
    // No latency configured → response should be fast (< 2s even on loaded CI).
    assert!(
        elapsed.as_millis() < 2000,
        "EC-006: zero latency response must be fast; got {elapsed:?}"
    );
}

// ---------------------------------------------------------------------------
// Additional coverage: alert and vulnerability endpoints have auth
// ---------------------------------------------------------------------------

/// Alerts endpoint returns 200 with `alerts` array.
#[tokio::test]
async fn test_alerts_list_returns_200_with_alerts_array() {
    let (_clone, base_url) = start_clone().await;
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

/// Vulnerabilities endpoint returns 200 with `vulnerabilities` array.
#[tokio::test]
async fn test_vulnerabilities_list_returns_200_with_vulns_array() {
    let (_clone, base_url) = start_clone().await;
    let client = reqwest::Client::new();

    let resp = client
        .post(format!("{base_url}/api/v1/vulnerabilities"))
        .header("Authorization", "Bearer test-token")
        .json(&json!({}))
        .send()
        .await
        .expect("request failed");

    assert_eq!(resp.status().as_u16(), 200, "vulnerabilities list must return 200");
    let body: serde_json::Value = resp.json().await.expect("body is JSON");
    assert!(body.get("vulnerabilities").is_some(), "response must have `vulnerabilities`");
    let vulns = body["vulnerabilities"].as_array().expect("`vulnerabilities` array");
    assert_eq!(vulns.len(), 15, "fixture must contain exactly 15 vulnerabilities");
}

/// Alerts by ID endpoint returns devices for a given alert.
#[tokio::test]
async fn test_alerts_by_id_returns_devices() {
    let (_clone, base_url) = start_clone().await;
    let client = reqwest::Client::new();

    let resp = client
        .post(format!("{base_url}/api/v1/alerts/1/devices"))
        .header("Authorization", "Bearer test-token")
        .json(&json!({}))
        .send()
        .await
        .expect("request failed");

    assert_eq!(resp.status().as_u16(), 200, "alerted devices must return 200");
    let body: serde_json::Value = resp.json().await.expect("body is JSON");
    assert!(body.get("devices").is_some(), "response must have `devices`");
    assert!(body.get("total").is_some(), "response must have `total`");
}

/// Vulnerabilities by ID endpoint returns devices for a given vulnerability.
#[tokio::test]
async fn test_vulnerability_by_id_returns_devices() {
    let (_clone, base_url) = start_clone().await;
    let client = reqwest::Client::new();

    let resp = client
        .post(format!("{base_url}/api/v1/vulnerabilities/vuln-001/devices"))
        .header("Authorization", "Bearer test-token")
        .json(&json!({}))
        .send()
        .await
        .expect("request failed");

    assert_eq!(resp.status().as_u16(), 200, "vulnerability devices must return 200");
    let body: serde_json::Value = resp.json().await.expect("body is JSON");
    assert!(body.get("devices").is_some(), "response must have `devices`");
    assert!(body.get("total").is_some(), "response must have `total`");
}
