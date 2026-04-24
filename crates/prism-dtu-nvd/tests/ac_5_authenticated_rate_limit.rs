#![allow(clippy::unwrap_used, clippy::expect_used)]
// AC-5: Authenticated rate limit: after 50 requests with a valid apiKey within a
// 30-second window, the 51st request returns HTTP 429.
//
// Because firing 50 real HTTP round-trips is slow, this test pre-exhausts the bucket
// via POST /dtu/configure before sending the triggering request.
//
// Expected failure mode: NvdClone::new() calls todo!() — panics at construction.

use prism_dtu_common::BehavioralClone;
use prism_dtu_nvd::NvdClone;

#[tokio::test]
async fn ac_5_authenticated_rate_limit_429_after_50_requests() {
    let mut clone = NvdClone::new().expect("AC-5: NvdClone::new() must succeed");
    clone.start().await.expect("AC-5: start() must succeed");

    let base_url = clone.base_url();
    let client = reqwest::Client::new();

    // Pre-exhaust the authenticated bucket by setting count to limit via configure.
    let configure_body = serde_json::json!({
        "exhaust_authenticated_bucket": true
    });
    let configure_resp = client
        .post(format!("{base_url}/dtu/configure"))
        .json(&configure_body)
        .send()
        .await
        .expect("AC-5: POST /dtu/configure must be reachable");

    assert_eq!(
        configure_resp.status().as_u16(),
        200,
        "AC-5: configure must return 200"
    );

    // Next authenticated request must return 429 (bucket exhausted).
    let resp = client
        .get(format!("{base_url}/rest/json/cves/2.0"))
        .query(&[("cveId", "CVE-2024-0001"), ("apiKey", "valid-key")])
        .send()
        .await
        .expect("AC-5: request after bucket exhaustion must be sent");

    assert_eq!(
        resp.status().as_u16(),
        429,
        "AC-5: 51st authenticated request must return HTTP 429"
    );

    let body: serde_json::Value = resp.json().await.expect("AC-5: body must be valid JSON");
    let error_msg = body["error"].as_str().unwrap_or("");

    assert!(
        !error_msg.is_empty(),
        "AC-5: HTTP 429 response must include an error message"
    );
}
