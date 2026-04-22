// AC-3: GET /rest/json/cves/2.0?cveId=CVE-9999-9999 (not in fixture) returns HTTP 404
// with body {"error": "CVE not found", "cveId": "CVE-9999-9999"}.
//
// Expected failure mode: NvdClone::new() calls todo!() — panics at construction.

use prism_dtu_nvd::NvdClone;
use prism_dtu_common::BehavioralClone;

#[tokio::test]
async fn ac_3_unknown_cve_id_returns_404_not_found() {
    let mut clone = NvdClone::new().expect("AC-3: NvdClone::new() must succeed");
    clone.start().await.expect("AC-3: start() must succeed");

    let base_url = clone.base_url();
    let client = reqwest::Client::new();

    let resp = client
        .get(format!("{base_url}/rest/json/cves/2.0"))
        .query(&[("cveId", "CVE-9999-9999"), ("apiKey", "valid-key")])
        .send()
        .await
        .expect("AC-3: request must succeed");

    assert_eq!(
        resp.status().as_u16(),
        404,
        "AC-3: unknown CVE must return HTTP 404"
    );

    let body: serde_json::Value = resp.json().await.expect("AC-3: body must be valid JSON");

    assert!(
        body["error"].as_str().unwrap_or("").contains("CVE not found"),
        "AC-3: error message must contain 'CVE not found', got: {:?}",
        body["error"]
    );

    assert_eq!(
        body["cveId"].as_str().unwrap_or(""),
        "CVE-9999-9999",
        "AC-3: response must echo the requested cveId"
    );
}
