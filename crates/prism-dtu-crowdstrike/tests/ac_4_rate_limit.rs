//! AC-4: FailureMode::RateLimit returns 429 with Retry-After header (S-6.07).
//!
//! Given `FailureMode::RateLimit { after_n_requests: 3, retry_after_secs: 60 }` is
//! configured, When the 4th request to any endpoint arrives, Then the response is
//! HTTP 429 with `Retry-After: 60` header.
//!
//! Was Red Gate at implementation start; CrowdstrikeClone::start() now implemented.
//! "not yet implemented".

#![allow(clippy::unwrap_used, clippy::expect_used)]

use prism_dtu_common::{BehavioralClone, FailureMode, StubConfig};
use prism_dtu_crowdstrike::CrowdstrikeClone;

/// AC-4: First 3 requests succeed; 4th returns 429 with Retry-After: 60.
#[tokio::test]
async fn ac_4_rate_limit_429_on_4th_request_with_retry_after_60() {
    // Expected failure: CrowdstrikeClone::start() panics with "not yet implemented".
    let mut clone = CrowdstrikeClone::with_config(StubConfig {
        seed: 42,
        latency_ms: 0,
        failure_mode: FailureMode::RateLimit {
            after_n_requests: 3,
            retry_after_secs: 60,
        },
        bind: None,
        ..Default::default()
    });
    clone.start().await.expect("AC-4: start() must succeed");

    let base_url = clone.base_url();
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .unwrap();

    // Requests 1–3 must succeed (within the rate limit window).
    for i in 1..=3u32 {
        let resp = client
            .get(format!("{base_url}/detects/queries/detects/v1"))
            .header("Authorization", "Bearer dtu-fake-cs-token")
            .send()
            .await
            .unwrap_or_else(|_| panic!("AC-4: request {i} must reach server"));

        assert_ne!(
            resp.status().as_u16(),
            429,
            "AC-4: request {i} must NOT return 429 (within limit of 3)"
        );
        assert_eq!(
            resp.status().as_u16(),
            200,
            "AC-4: request {i} must return 200"
        );
    }

    // 4th request must receive HTTP 429.
    let resp4 = client
        .get(format!("{base_url}/detects/queries/detects/v1"))
        .header("Authorization", "Bearer dtu-fake-cs-token")
        .send()
        .await
        .expect("AC-4: 4th request must reach server");

    assert_eq!(
        resp4.status().as_u16(),
        429,
        "AC-4: 4th request must return HTTP 429"
    );

    let retry_after = resp4
        .headers()
        .get("retry-after")
        .expect("AC-4: HTTP 429 must include Retry-After header");

    assert_eq!(
        retry_after.to_str().expect("Retry-After must be ASCII"),
        "60",
        "AC-4: Retry-After header value must be '60' (retry_after_secs from config)"
    );
}

/// AC-4: Rate limit applies across all endpoints (not just detection queries).
#[tokio::test]
async fn ac_4_rate_limit_applies_to_all_endpoints() {
    let mut clone = CrowdstrikeClone::with_config(StubConfig {
        seed: 42,
        latency_ms: 0,
        failure_mode: FailureMode::RateLimit {
            after_n_requests: 2,
            retry_after_secs: 60,
        },
        bind: None,
        ..Default::default()
    });
    clone.start().await.expect("AC-4 all: start() must succeed");

    let base_url = clone.base_url();
    let client = reqwest::Client::new();

    // Request 1 to detection endpoint.
    let r1 = client
        .get(format!("{base_url}/detects/queries/detects/v1"))
        .header("Authorization", "Bearer dtu-fake-cs-token")
        .send()
        .await
        .expect("AC-4 all: request 1 must reach server");
    assert_eq!(r1.status().as_u16(), 200, "AC-4 all: request 1 must be 200");

    // Request 2 to host endpoint (different endpoint — counter is shared).
    let r2 = client
        .get(format!("{base_url}/devices/queries/devices/v1"))
        .header("Authorization", "Bearer dtu-fake-cs-token")
        .send()
        .await
        .expect("AC-4 all: request 2 must reach server");
    assert_eq!(r2.status().as_u16(), 200, "AC-4 all: request 2 must be 200");

    // Request 3 to any endpoint — must be rate limited.
    let r3 = client
        .get(format!("{base_url}/detects/queries/detects/v1"))
        .header("Authorization", "Bearer dtu-fake-cs-token")
        .send()
        .await
        .expect("AC-4 all: request 3 must reach server");

    assert_eq!(
        r3.status().as_u16(),
        429,
        "AC-4 all: 3rd request must return 429 (rate limit after_n=2 is shared across endpoints)"
    );
}

/// AC-4: Retry-After header value exactly matches configured retry_after_secs.
#[tokio::test]
async fn ac_4_retry_after_header_matches_configured_secs() {
    let retry_after_secs = 120u32;
    let mut clone = CrowdstrikeClone::with_config(StubConfig {
        seed: 42,
        latency_ms: 0,
        failure_mode: FailureMode::RateLimit {
            after_n_requests: 1,
            retry_after_secs,
        },
        bind: None,
        ..Default::default()
    });
    clone
        .start()
        .await
        .expect("AC-4 header: start() must succeed");

    let base_url = clone.base_url();
    let client = reqwest::Client::new();

    // First request succeeds (after_n_requests = 1).
    client
        .get(format!("{base_url}/detects/queries/detects/v1"))
        .header("Authorization", "Bearer dtu-fake-cs-token")
        .send()
        .await
        .expect("AC-4 header: request 1 must reach server");

    // Second request must be rate limited with exact Retry-After value.
    let r2 = client
        .get(format!("{base_url}/detects/queries/detects/v1"))
        .header("Authorization", "Bearer dtu-fake-cs-token")
        .send()
        .await
        .expect("AC-4 header: request 2 must reach server");

    assert_eq!(r2.status().as_u16(), 429, "AC-4 header: must be 429");

    let header_val = r2
        .headers()
        .get("retry-after")
        .expect("AC-4 header: Retry-After must be present")
        .to_str()
        .expect("Retry-After must be ASCII");

    assert_eq!(
        header_val,
        retry_after_secs.to_string().as_str(),
        "AC-4 header: Retry-After must exactly match configured retry_after_secs={retry_after_secs}"
    );
}
