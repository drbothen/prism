// AC-2: After the same CVE is requested twice, GET /dtu/request-count/CVE-2024-0001
// returns count: 2 (no Prism cache). The test asserts count == 2 from the DTU's
// perspective; a cache-hit assertion (count == 1) is validated in the Prism integration
// suite where caching is controlled. Here we verify the counter increments correctly.
//
// Expected failure mode: NvdClone::new() calls todo!() — panics at construction.

use prism_dtu_nvd::NvdClone;
use prism_dtu_common::BehavioralClone;

#[tokio::test]
async fn ac_2_request_count_increments_per_cve_lookup() {
    let mut clone = NvdClone::new().expect("AC-2: NvdClone::new() must succeed");
    clone.start().await.expect("AC-2: start() must succeed");

    let base_url = clone.base_url();
    let client = reqwest::Client::new();

    // First request.
    client
        .get(format!("{base_url}/rest/json/cves/2.0"))
        .query(&[("cveId", "CVE-2024-0001"), ("apiKey", "valid-key")])
        .send()
        .await
        .expect("AC-2: first request must succeed");

    // Second request.
    client
        .get(format!("{base_url}/rest/json/cves/2.0"))
        .query(&[("cveId", "CVE-2024-0001"), ("apiKey", "valid-key")])
        .send()
        .await
        .expect("AC-2: second request must succeed");

    // Test API: confirm counter is 2 (both requests reached the DTU).
    let count_resp = client
        .get(format!("{base_url}/dtu/request-count/CVE-2024-0001"))
        .send()
        .await
        .expect("AC-2: /dtu/request-count must be reachable");

    assert_eq!(
        count_resp.status().as_u16(),
        200,
        "AC-2: /dtu/request-count must return 200"
    );

    let body: serde_json::Value = count_resp
        .json()
        .await
        .expect("AC-2: request-count body must be valid JSON");

    assert_eq!(
        body["cve_id"].as_str().unwrap_or(""),
        "CVE-2024-0001",
        "AC-2: response cve_id must match"
    );
    assert_eq!(
        body["count"].as_u64().unwrap_or(0),
        2,
        "AC-2: count must be 2 after two direct DTU requests"
    );

    // Also exercise the in-process test API on NvdClone.
    assert_eq!(
        clone.request_count_for("CVE-2024-0001"),
        2,
        "AC-2: NvdClone::request_count_for must return 2"
    );
}
