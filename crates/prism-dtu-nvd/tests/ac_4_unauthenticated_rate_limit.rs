#![allow(clippy::unwrap_used, clippy::expect_used)]
// AC-4: Unauthenticated rate limit: after 5 requests within a 30-second window,
// the 6th unauthenticated request returns HTTP 403 with the canonical error message.
//
// Expected failure mode: NvdClone::new() calls todo!() — panics at construction.

use prism_dtu_common::BehavioralClone;
use prism_dtu_nvd::NvdClone;

#[tokio::test]
async fn ac_4_unauthenticated_rate_limit_403_on_sixth_request() {
    let mut clone = NvdClone::new().expect("AC-4: NvdClone::new() must succeed");
    clone.start().await.expect("AC-4: start() must succeed");

    let base_url = clone.base_url();
    let client = reqwest::Client::new();

    // 5 unauthenticated requests must succeed (within 5/30s limit).
    for i in 1..=5 {
        let resp = client
            .get(format!("{base_url}/rest/json/cves/2.0"))
            .query(&[("cveId", format!("CVE-2024-{:04}", i).as_str())])
            .send()
            .await
            .unwrap_or_else(|_| panic!("AC-4: unauthenticated request {i} must succeed"));

        // May be 200 or 404 (fixture may not contain all IDs used) — but NOT 403.
        assert_ne!(
            resp.status().as_u16(),
            403,
            "AC-4: request {i} must NOT return 403 (within rate limit)"
        );
    }

    // 6th unauthenticated request must return HTTP 403.
    let resp = client
        .get(format!("{base_url}/rest/json/cves/2.0"))
        .query(&[("cveId", "CVE-2024-0001")])
        .send()
        .await
        .expect("AC-4: 6th request must be sent");

    assert_eq!(
        resp.status().as_u16(),
        403,
        "AC-4: 6th unauthenticated request must return HTTP 403"
    );

    let body: serde_json::Value = resp.json().await.expect("AC-4: body must be valid JSON");
    let error_msg = body["error"].as_str().unwrap_or("");

    assert!(
        error_msg.contains("Unauthenticated rate limit exceeded"),
        "AC-4: error must indicate unauthenticated rate limit, got: {error_msg:?}"
    );
}
