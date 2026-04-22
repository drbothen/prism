// AC-1: GET /rest/json/cves/2.0?cveId=CVE-2024-0001 with a valid apiKey returns HTTP 200
// with totalResults: 1, vulnerabilities[0].cve.id == "CVE-2024-0001", baseScore 9.8,
// baseSeverity "CRITICAL", and a CISA KEV date.
//
// Expected failure mode: NvdClone::new() calls todo!() — panics at construction.

use prism_dtu_common::BehavioralClone;
use prism_dtu_nvd::NvdClone;

#[tokio::test]
async fn ac_1_cve_lookup_returns_fixture_cve_with_kev_and_cvss() {
    let mut clone = NvdClone::new().expect("AC-1: NvdClone::new() must succeed");
    clone.start().await.expect("AC-1: start() must succeed");

    let base_url = clone.base_url();
    let client = reqwest::Client::new();

    let resp = client
        .get(format!("{base_url}/rest/json/cves/2.0"))
        .query(&[("cveId", "CVE-2024-0001"), ("apiKey", "valid-key")])
        .send()
        .await
        .expect("AC-1: request must succeed");

    assert_eq!(
        resp.status().as_u16(),
        200,
        "AC-1: single CVE lookup must return HTTP 200"
    );

    let body: serde_json::Value = resp.json().await.expect("AC-1: body must be valid JSON");

    assert_eq!(
        body["totalResults"].as_u64().unwrap_or(0),
        1,
        "AC-1: totalResults must be 1"
    );

    let vuln = &body["vulnerabilities"][0]["cve"];

    assert_eq!(
        vuln["id"].as_str().unwrap_or(""),
        "CVE-2024-0001",
        "AC-1: cve.id must be CVE-2024-0001"
    );

    let base_score = vuln["metrics"]["cvssMetricV31"][0]["cvssData"]["baseScore"]
        .as_f64()
        .unwrap_or(0.0);
    assert!(
        (base_score - 9.8).abs() < 0.01,
        "AC-1: baseScore must be 9.8, got {base_score}"
    );

    let base_severity = vuln["metrics"]["cvssMetricV31"][0]["cvssData"]["baseSeverity"]
        .as_str()
        .unwrap_or("");
    assert_eq!(
        base_severity, "CRITICAL",
        "AC-1: baseSeverity must be CRITICAL"
    );

    assert!(
        !vuln["cisaKevVulnAdded"].is_null(),
        "AC-1: cisaKevVulnAdded must be present for CVE-2024-0001 (CISA KEV)"
    );
}
