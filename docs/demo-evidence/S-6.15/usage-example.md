# Usage Example — NvdClone

The following shows canonical usage of `NvdClone` in an integration test context,
exercising single CVE lookup, request count assertion, rate-limit triggering, pagination,
and runtime reconfiguration.

```rust
use prism_dtu_common::BehavioralClone;
use prism_dtu_nvd::NvdClone;

#[tokio::test]
async fn nvd_dtu_usage_example() {
    // 1. Construct and start the DTU (binds ephemeral port, loads 10 fixture CVEs).
    let mut clone = NvdClone::new().expect("construction must succeed");
    clone.start().await.expect("start must succeed");
    let base = clone.base_url(); // e.g. "http://127.0.0.1:54321"

    let client = reqwest::Client::new();

    // 2. Single CVE lookup — authenticated (50/30s bucket).
    let resp = client
        .get(format!("{base}/rest/json/cves/2.0"))
        .query(&[("cveId", "CVE-2024-0001"), ("apiKey", "test-key")])
        .send().await.unwrap();
    assert_eq!(resp.status().as_u16(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["totalResults"], 1);
    assert_eq!(body["vulnerabilities"][0]["cve"]["id"], "CVE-2024-0001");

    // 3. Check rate-limit response header analog via in-process API.
    assert_eq!(clone.request_count_for("CVE-2024-0001"), 1);

    // 4. Paginate bulk fetch — page 1 of 2 (5 CVEs each).
    let page1 = client
        .get(format!("{base}/rest/json/cves/2.0"))
        .query(&[("startIndex", "0"), ("resultsPerPage", "5")])
        .send().await.unwrap().json::<serde_json::Value>().await.unwrap();
    assert_eq!(page1["totalResults"], 10);
    assert_eq!(page1["vulnerabilities"].as_array().unwrap().len(), 5);

    // 5. Apply runtime config — exhaust authenticated bucket.
    client.post(format!("{base}/dtu/configure"))
        .json(&serde_json::json!({"exhaust_authenticated_bucket": true}))
        .send().await.unwrap();

    // 6. Next authenticated request returns 429 (bucket exhausted).
    let throttled = client
        .get(format!("{base}/rest/json/cves/2.0"))
        .query(&[("cveId", "CVE-2024-0002"), ("apiKey", "test-key")])
        .send().await.unwrap();
    assert_eq!(throttled.status().as_u16(), 429);

    // 7. Reset and verify counters clear.
    clone.reset().await.unwrap();
    assert_eq!(clone.request_count_for("CVE-2024-0001"), 0);
}
```
