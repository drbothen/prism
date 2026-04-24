#![allow(clippy::unwrap_used, clippy::expect_used)]
// AC-6: Given `FailureMode::RateLimit` configured, When threshold exceeded,
// Then HTTP 429 is returned — maps to `E-SENSOR-003`.
//
// Also covers:
//   EC-006 — FailureLayer::MalformedResponse (exercises Prism's parse-error path).
//
// Red Gate: these tests WILL FAIL because the Armis stub does not yet wire
// FailureLayer into its router. The `ArmisClone` build_router() does not apply
// any tower middleware layer, so configure({failure_mode: rate_limit}) has no effect.
//
// Once FailureLayer is integrated into build_router, these tests will pass.

#![cfg(feature = "dtu")]

use prism_dtu_armis::ArmisClone;
use prism_dtu_common::BehavioralClone;

#[tokio::test]
async fn ac_6_rate_limit_429_after_threshold_exceeded_via_configure() {
    // Pre-exhaust the request budget by configuring the stub with RateLimit(after_n=0).
    let mut clone = ArmisClone::new().expect("AC-6: ArmisClone::new() must succeed");
    clone.start().await.expect("AC-6: start() must succeed");
    let base_url = clone.base_url();
    let client = reqwest::Client::new();

    // Configure failure injection: rate-limit after 0 successful requests.
    let configure_resp = client
        .post(format!("{base_url}/dtu/configure"))
        .json(&serde_json::json!({
            "failure_mode": "rate_limit",
            "after_n_requests": 0,
            "retry_after_secs": 30
        }))
        .send()
        .await
        .expect("AC-6: POST /dtu/configure must succeed");

    assert_eq!(
        configure_resp.status().as_u16(),
        200,
        "AC-6: POST /dtu/configure must return 200"
    );

    // Next request to vendor API must return 429.
    let resp = client
        .get(format!("{base_url}/api/v1/devices"))
        .header("Authorization", "Bearer test-token")
        .send()
        .await
        .expect("AC-6: rate-limited request must be sent");

    assert_eq!(
        resp.status().as_u16(),
        429,
        "AC-6: request after rate-limit exhaustion must return HTTP 429"
    );
}

#[tokio::test]
async fn ac_6_rate_limit_allows_requests_before_threshold() {
    // Verify requests below threshold succeed.
    let mut clone = ArmisClone::new().expect("AC-6 threshold: ArmisClone::new() must succeed");
    clone
        .start()
        .await
        .expect("AC-6 threshold: start() must succeed");
    let base_url = clone.base_url();
    let client = reqwest::Client::new();

    // Configure rate limit: allow 3 requests, then 429.
    let configure_resp = client
        .post(format!("{base_url}/dtu/configure"))
        .json(&serde_json::json!({
            "failure_mode": "rate_limit",
            "after_n_requests": 3,
            "retry_after_secs": 30
        }))
        .send()
        .await
        .expect("AC-6 threshold: POST /dtu/configure must succeed");

    assert_eq!(
        configure_resp.status().as_u16(),
        200,
        "AC-6 threshold: configure must return 200"
    );

    // First 3 requests must NOT be rate-limited (200 or 403, but not 429).
    for i in 1..=3 {
        let resp = client
            .get(format!("{base_url}/api/v1/devices"))
            .header("Authorization", "Bearer test-token")
            .send()
            .await
            .unwrap_or_else(|_| panic!("AC-6 threshold: request {i} must succeed"));

        assert_ne!(
            resp.status().as_u16(),
            429,
            "AC-6 threshold: request {i} must NOT be rate-limited (within budget)"
        );
    }

    // 4th request must return 429.
    let resp = client
        .get(format!("{base_url}/api/v1/devices"))
        .header("Authorization", "Bearer test-token")
        .send()
        .await
        .expect("AC-6 threshold: 4th request must be sent");

    assert_eq!(
        resp.status().as_u16(),
        429,
        "AC-6 threshold: 4th request must return HTTP 429 (rate limit exceeded)"
    );
}

#[tokio::test]
async fn ec_006_malformed_response_mode_returns_non_parseable_body() {
    // EC-006: FailureLayer::MalformedResponse returns an invalid JSON body.
    // This exercises Prism's parse-error handling path.
    //
    // Red Gate: will fail because FailureLayer is not wired into the Armis stub router,
    // so configure({failure_mode: malformed_response}) has no effect.
    let mut clone = ArmisClone::new().expect("EC-006: ArmisClone::new() must succeed");
    clone.start().await.expect("EC-006: start() must succeed");
    let base_url = clone.base_url();
    let client = reqwest::Client::new();

    // Configure malformed response mode.
    let configure_resp = client
        .post(format!("{base_url}/dtu/configure"))
        .json(&serde_json::json!({
            "failure_mode": "malformed_response"
        }))
        .send()
        .await
        .expect("EC-006: POST /dtu/configure must succeed");

    assert_eq!(
        configure_resp.status().as_u16(),
        200,
        "EC-006: configure must return 200"
    );

    // Request a device list — should get a malformed (non-JSON) response body.
    let resp = client
        .get(format!("{base_url}/api/v1/devices"))
        .header("Authorization", "Bearer test-token")
        .send()
        .await
        .expect("EC-006: malformed response request must be sent");

    // The response will arrive (any status), but the body must not be valid JSON.
    let raw_bytes = resp
        .bytes()
        .await
        .expect("EC-006: raw body bytes must be readable");

    let parse_result = serde_json::from_slice::<serde_json::Value>(&raw_bytes);

    assert!(
        parse_result.is_err(),
        "EC-006: malformed response mode must produce a body that fails JSON parsing"
    );
}
