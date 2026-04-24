#![allow(clippy::unwrap_used, clippy::expect_used)]
//! AC-7: FailureMode::InternalError — 1st POST request after config returns HTTP 500.
//!
//! Exercises Prism's E-SENSOR-002 error path.

use prism_dtu_claroty::ClarotyClone;
use prism_dtu_common::BehavioralClone;
use serde_json::json;

async fn start_clone() -> (ClarotyClone, String, String) {
    let mut clone = ClarotyClone::new();
    clone.start().await.expect("ClarotyClone::start failed");
    let base_url = clone.base_url();
    let admin_token = clone.admin_token().to_string();
    (clone, base_url, admin_token)
}

/// AC-7: internal_error_at=1 makes first POST return HTTP 500.
#[tokio::test]
async fn test_ac7_internal_error_first_request_returns_500() {
    let (_clone, base_url, admin_token) = start_clone().await;
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

/// AC-7: internal_error_at=2 means the first request succeeds but the 2nd returns 500.
#[tokio::test]
async fn test_ac7_internal_error_at_n_only_fails_nth_request() {
    let (_clone, base_url, admin_token) = start_clone().await;
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

/// AC-7: After reset, the internal error is cleared and requests succeed again.
#[tokio::test]
async fn test_ac7_reset_clears_internal_error_mode() {
    let (_clone, base_url, admin_token) = start_clone().await;
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
