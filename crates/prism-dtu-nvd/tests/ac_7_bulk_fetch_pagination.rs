// AC-7: GET /rest/json/cves/2.0 with startIndex=0&resultsPerPage=5 returns:
// - resultsPerPage: 5
// - totalResults: 10 (full fixture set)
// - vulnerabilities: array of 5 CVE entries
//
// Also covers edge cases EC-002 (resultsPerPage=0 treated as 1) and
// EC-003 (startIndex beyond total returns empty array).
//
// Expected failure mode: NvdClone::new() calls todo!() — panics at construction.

use prism_dtu_common::BehavioralClone;
use prism_dtu_nvd::NvdClone;

#[tokio::test]
async fn ac_7_bulk_fetch_first_page_returns_five_of_ten() {
    let mut clone = NvdClone::new().expect("AC-7: NvdClone::new() must succeed");
    clone.start().await.expect("AC-7: start() must succeed");

    let base_url = clone.base_url();
    let client = reqwest::Client::new();

    let resp = client
        .get(format!("{base_url}/rest/json/cves/2.0"))
        .query(&[("startIndex", "0"), ("resultsPerPage", "5")])
        .send()
        .await
        .expect("AC-7: bulk fetch request must succeed");

    assert_eq!(
        resp.status().as_u16(),
        200,
        "AC-7: bulk fetch must return HTTP 200"
    );

    let body: serde_json::Value = resp.json().await.expect("AC-7: body must be valid JSON");

    assert_eq!(
        body["resultsPerPage"].as_u64().unwrap_or(0),
        5,
        "AC-7: resultsPerPage must be 5"
    );

    assert_eq!(
        body["totalResults"].as_u64().unwrap_or(0),
        10,
        "AC-7: totalResults must be 10 (full fixture set)"
    );

    let vuln_count = body["vulnerabilities"]
        .as_array()
        .map(|a| a.len())
        .unwrap_or(0);
    assert_eq!(
        vuln_count, 5,
        "AC-7: vulnerabilities array must contain 5 entries"
    );

    assert_eq!(
        body["startIndex"].as_u64().unwrap_or(99),
        0,
        "AC-7: startIndex must echo 0"
    );
}

#[tokio::test]
async fn ac_7_bulk_fetch_second_page_returns_remaining_five() {
    let mut clone = NvdClone::new().expect("AC-7b: NvdClone::new() must succeed");
    clone.start().await.expect("AC-7b: start() must succeed");

    let base_url = clone.base_url();
    let client = reqwest::Client::new();

    let resp = client
        .get(format!("{base_url}/rest/json/cves/2.0"))
        .query(&[("startIndex", "5"), ("resultsPerPage", "5")])
        .send()
        .await
        .expect("AC-7b: second page request must succeed");

    assert_eq!(resp.status().as_u16(), 200, "AC-7b: must return 200");

    let body: serde_json::Value = resp.json().await.expect("AC-7b: body must be JSON");

    let vuln_count = body["vulnerabilities"]
        .as_array()
        .map(|a| a.len())
        .unwrap_or(0);
    assert_eq!(
        vuln_count, 5,
        "AC-7b: second page must return remaining 5 CVEs"
    );
    assert_eq!(
        body["totalResults"].as_u64().unwrap_or(0),
        10,
        "AC-7b: totalResults always reflects full fixture count"
    );
}

// EC-003: startIndex beyond total results returns empty vulnerabilities array.
#[tokio::test]
async fn ac_7_ec003_start_index_beyond_total_returns_empty() {
    let mut clone = NvdClone::new().expect("EC-003: NvdClone::new() must succeed");
    clone.start().await.expect("EC-003: start() must succeed");

    let base_url = clone.base_url();
    let client = reqwest::Client::new();

    let resp = client
        .get(format!("{base_url}/rest/json/cves/2.0"))
        .query(&[("startIndex", "100"), ("resultsPerPage", "5")])
        .send()
        .await
        .expect("EC-003: request must succeed");

    assert_eq!(resp.status().as_u16(), 200, "EC-003: must return 200");

    let body: serde_json::Value = resp.json().await.expect("EC-003: body must be JSON");

    let vuln_count = body["vulnerabilities"]
        .as_array()
        .map(|a| a.len())
        .unwrap_or(99);
    assert_eq!(
        vuln_count, 0,
        "EC-003: startIndex beyond total must return empty vulnerabilities array"
    );
    assert_eq!(
        body["totalResults"].as_u64().unwrap_or(0),
        10,
        "EC-003: totalResults must still be 10"
    );
}

// EC-001: Both cveId and startIndex present — cveId takes precedence.
#[tokio::test]
async fn ac_7_ec001_cve_id_takes_precedence_over_pagination() {
    let mut clone = NvdClone::new().expect("EC-001: NvdClone::new() must succeed");
    clone.start().await.expect("EC-001: start() must succeed");

    let base_url = clone.base_url();
    let client = reqwest::Client::new();

    let resp = client
        .get(format!("{base_url}/rest/json/cves/2.0"))
        .query(&[
            ("cveId", "CVE-2024-0001"),
            ("startIndex", "0"),
            ("resultsPerPage", "5"),
            ("apiKey", "valid-key"),
        ])
        .send()
        .await
        .expect("EC-001: request must succeed");

    assert_eq!(resp.status().as_u16(), 200, "EC-001: must return 200");

    let body: serde_json::Value = resp.json().await.expect("EC-001: body must be JSON");

    assert_eq!(
        body["totalResults"].as_u64().unwrap_or(0),
        1,
        "EC-001: cveId takes precedence — totalResults must be 1"
    );
}
