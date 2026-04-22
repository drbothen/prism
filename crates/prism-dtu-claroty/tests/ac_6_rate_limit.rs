//! AC-6: FailureMode::RateLimit — after N requests, 6th returns HTTP 429 with Retry-After.

use prism_dtu_claroty::ClarotyClone;
use prism_dtu_common::BehavioralClone;
use serde_json::json;

async fn start_clone() -> (ClarotyClone, String) {
    let mut clone = ClarotyClone::new();
    clone.start().await.expect("ClarotyClone::start failed");
    let base_url = clone.base_url();
    (clone, base_url)
}

/// AC-6: After configuring rate_limit_after=5, the 6th request returns HTTP 429.
#[tokio::test]
async fn test_ac6_rate_limit_6th_request_returns_429() {
    let (_clone, base_url) = start_clone().await;
    let client = reqwest::Client::new();

    // Configure rate limit: reject after 5 requests.
    client
        .post(format!("{base_url}/dtu/configure"))
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

/// AC-6: HTTP 429 response includes Retry-After header with configured value.
#[tokio::test]
async fn test_ac6_rate_limit_response_has_retry_after_header() {
    let (_clone, base_url) = start_clone().await;
    let client = reqwest::Client::new();

    client
        .post(format!("{base_url}/dtu/configure"))
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

/// AC-6: Configuring rate_limit_after via /dtu/configure returns 200.
#[tokio::test]
async fn test_ac6_dtu_configure_returns_200() {
    let (_clone, base_url) = start_clone().await;
    let client = reqwest::Client::new();

    let resp = client
        .post(format!("{base_url}/dtu/configure"))
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
