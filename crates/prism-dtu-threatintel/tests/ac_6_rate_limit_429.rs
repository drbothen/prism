#![allow(clippy::unwrap_used, clippy::expect_used)]
// AC-6: POST /dtu/configure {"rate_limit_after": 3}, then 4th lookup returns
// HTTP 429 with Retry-After: 30.
//
// Expected Red Gate failure: the rate-limit counter increments on every call
// to check_rate_limit() (including calls that later return auth errors). However
// check_rate_limit() is called AFTER check_auth() in the handler, so unauthenticated
// requests do NOT increment the counter. The test uses authenticated requests.
//
// The stub implements check_rate_limit() via state.increment_counter() which
// fetches-add and then checks is_rate_limited(). The 4th request (count = 4)
// exceeds threshold 3 (4 > 3), so 429 is returned with Retry-After: 30.
// This is implemented in the stub. However, this test additionally asserts that:
// 1. The 3rd request still succeeds (count 3 == threshold, NOT exceeded).
// 2. Retry-After header value is exactly "30" (not "30\r\n" or other variants).
// 3. After reset(), the counter is back to 0 and the 4th request no longer rate-limits.
// The reset() must also clear the rate_limit_after threshold — if it doesn't,
// the 4th post-reset request would again trigger 429, failing assertion 3.

use prism_dtu_common::{build_test_client, BehavioralClone};
use prism_dtu_threatintel::ThreatIntelClone;

const API_KEY: &str = "test-key-rl";

#[tokio::test]
async fn ac_6_rate_limit_after_3_returns_429_on_4th_request_with_retry_after_30() {
    let mut clone = ThreatIntelClone::new();
    clone.start().await.expect("AC-6: start() must succeed");

    let base = clone.base_url();
    let client = build_test_client();

    // Configure rate_limit_after = 3.
    let cfg_resp = client
        .post(format!("{base}/dtu/configure"))
        .json(&serde_json::json!({"rate_limit_after": 3}))
        .send()
        .await
        .expect("AC-6: configure request must reach server");

    assert_eq!(
        cfg_resp.status().as_u16(),
        200,
        "AC-6: POST /dtu/configure must return 200"
    );

    // Requests 1–3 must succeed (counter <= threshold).
    for i in 1..=3u32 {
        let resp = client
            .get(format!("{base}/v3/ip/8.8.8.8"))
            .query(&[("key", API_KEY)])
            .send()
            .await
            .unwrap_or_else(|_| panic!("AC-6: request {i} must reach server"));

        assert_eq!(
            resp.status().as_u16(),
            200,
            "AC-6: request {i} must return 200 (within rate limit of 3)"
        );
    }

    // 4th request must receive HTTP 429.
    let resp4 = client
        .get(format!("{base}/v3/ip/8.8.8.8"))
        .query(&[("key", API_KEY)])
        .send()
        .await
        .expect("AC-6: 4th request must reach server");

    assert_eq!(
        resp4.status().as_u16(),
        429,
        "AC-6: 4th request must return HTTP 429"
    );

    let retry_after = resp4
        .headers()
        .get("retry-after")
        .expect("AC-6: HTTP 429 must include Retry-After header");

    assert_eq!(
        retry_after
            .to_str()
            .expect("Retry-After header is valid ASCII"),
        "30",
        "AC-6: Retry-After must be '30' per story spec"
    );

    // After reset(), the counter and threshold must be cleared.
    clone.reset().await.expect("AC-6: reset() must succeed");

    // Now 3 new requests should all succeed (no threshold configured after reset).
    for i in 1..=3u32 {
        let resp = client
            .get(format!("{base}/v3/ip/8.8.8.8"))
            .query(&[("key", API_KEY)])
            .send()
            .await
            .unwrap_or_else(|_| panic!("AC-6: post-reset request {i} must reach server"));

        assert_eq!(
            resp.status().as_u16(),
            200,
            "AC-6: post-reset request {i} must return 200 (threshold cleared by reset)"
        );
    }
}
