//! Red Gate test 14: BC-2.06.020 NVD CVE enrichment correlation
//!
//! test_BC_2_06_020_nvd_cve_correlation_high_cvss_base_score
//!
//! Traces to: BC-2.06.020 INV-NVD-CVE-CORRELATION-001 / PC-3, PC-4
//! Story: S-DEMO-DTU-LIVE-SCENARIO-001-B
//!
//! FAIL mode (Red Gate): NvdClone::new_with_scenario stub delegates to new()
//! without inserting synthetic CveRecord entries for scenario CVEs.
//! HTTP GET /rest/json/cves/2.0?cveId=<scenario_cve> returns 404 (totalResults=0).
//! Assertion that totalResults >= 1 FAILS.

#![cfg(feature = "fixture-gen")]
#![allow(clippy::unwrap_used, clippy::expect_used)]

use prism_dtu_common::{build_scenario_entity_catalog, BehavioralClone, OrgId};
use prism_dtu_nvd::NvdClone;

/// Org ID with well-known first 4 bytes [0xde, 0xad, 0xbe, 0xef] → org_slug = "deadbeef".
fn deadbeef_org() -> OrgId {
    OrgId([
        0xde, 0xad, 0xbe, 0xef, 0x00, 0x00, 0x70, 0x00, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00,
    ])
}

/// RED GATE TEST 14 — test_BC_2_06_020_nvd_cve_correlation_high_cvss_base_score
///
/// BC-2.06.020 INV-NVD-CVE-CORRELATION-001 / PC-3, PC-4
///
/// Given NvdClone::new_with_scenario(entities) (fallible, mirrors new() -> anyhow::Result<Self>),
/// when GET /rest/json/cves/2.0?cveId=<entities.device_cves[0]> is requested,
/// then the response contains:
/// - totalResults >= 1 (CVE was found in registry)
/// - vulnerabilities[0].cve.metrics.cvssMetricV31[0].cvssData.baseScore >= 7.0
/// - vulnerabilities[0].cve.metrics.cvssMetricV31[0].cvssData.baseSeverity == "HIGH"
///
/// AC-014 note: the JSON field is `cvssMetricV31` (camelCase via serde rename_all);
/// the Rust struct field is `cvss_metric_v31: Option<Vec<CvssMetricV31>>`.
/// Test code accesses via JSON path, not struct field.
///
/// FAIL mode: stub calls new() without inserting synthetic CVE records.
/// The scenario CVE is not in cve_registry → GET returns HTTP 404 (not 200 + data).
/// Assertion on HTTP 200 or totalResults FAILS.
#[tokio::test]
async fn test_BC_2_06_020_nvd_cve_correlation_high_cvss_base_score() {
    let org = deadbeef_org();
    let seed: u64 = 55;

    let catalog = build_scenario_entity_catalog(seed, &org);

    // Catalog must have non-empty device_cves (derived from secondary RNG stream).
    assert!(
        !catalog.device_cves.is_empty(),
        "ScenarioEntityCatalog must have non-empty device_cves; got empty \
         (secondary RNG derivation issue)"
    );

    // Construct NvdClone via new_with_scenario (fallible, mirrors new() -> anyhow::Result<Self>).
    // STUB: delegates to new() without inserting synthetic CVE records.
    let mut clone = NvdClone::new_with_scenario(&catalog).expect(
        "NvdClone::new_with_scenario must succeed; \
                  got Err — constructor failed unexpectedly",
    );

    // Start the server.
    clone.start().await.expect("NvdClone start must succeed");

    let base_url = clone.base_url();
    let client = prism_dtu_common::build_test_client();

    // Query for the first scenario CVE.
    let cve_id = &catalog.device_cves[0];

    // GET /rest/json/cves/2.0?cveId=<cve_id>
    // FAIL: stub doesn't insert scenario CVEs → registry misses this CVE → HTTP 404 or totalResults=0.
    let resp = client
        .get(format!("{base_url}/rest/json/cves/2.0"))
        .query(&[("cveId", cve_id.as_str()), ("apiKey", "test-key")])
        .send()
        .await
        .expect("GET /rest/json/cves/2.0 must reach the NVD server");

    // The response must be 200 (CVE found).
    // FAIL: stub returns 404 because CVE not in registry.
    assert_eq!(
        resp.status().as_u16(),
        200,
        "GET /rest/json/cves/2.0?cveId={cve_id} must return HTTP 200 (CVE found in registry); \
         got {} — stub did not inject scenario CVE into cve_registry \
         [RED GATE: INV-NVD-CVE-CORRELATION-001 / BC-2.06.020 PC-3]",
        resp.status().as_u16()
    );

    let body: serde_json::Value = resp.json().await.expect("response body must be valid JSON");

    let total = body["totalResults"].as_u64().unwrap_or(0);
    assert!(
        total >= 1,
        "totalResults must be >= 1 for scenario CVE '{cve_id}'; got {total} \
         — stub did not inject scenario CVE [RED GATE]"
    );

    let vuln = &body["vulnerabilities"][0]["cve"];

    // AC-014: cvss_metric_v31 is Option<Vec<CvssMetricV31>>; in JSON it's `cvssMetricV31`.
    // Access via JSON path, using .and_then to handle Option.
    let base_score = vuln["metrics"]["cvssMetricV31"][0]["cvssData"]["baseScore"].as_f64();

    assert!(
        base_score.is_some(),
        "vulnerabilities[0].cve.metrics.cvssMetricV31[0].cvssData.baseScore must be present \
         for scenario CVE '{cve_id}'; got null. \
         AC-014: field path is metrics.cvssMetricV31[0].cvssData.baseScore (f64). \
         BC-2.06.020 PC-4"
    );

    let score = base_score.unwrap();
    assert!(
        score >= 7.0,
        "cvssData.baseScore must be >= 7.0 for scenario CVE '{cve_id}'; \
         got {score}. Default construction value: 8.1. BC-2.06.020 PC-4 / AC-014"
    );

    let base_severity = vuln["metrics"]["cvssMetricV31"][0]["cvssData"]["baseSeverity"].as_str();

    assert!(
        base_severity.is_some(),
        "vulnerabilities[0].cve.metrics.cvssMetricV31[0].cvssData.baseSeverity must be present; \
         got null. Field name is 'baseSeverity' (NOT 'severity'). AC-014"
    );

    assert_eq!(
        base_severity.unwrap(),
        "HIGH",
        "cvssData.baseSeverity must be 'HIGH' for scenario CVE '{cve_id}'; \
         got '{}'. Default construction value: 'HIGH'. BC-2.06.020 PC-4 / AC-014",
        base_severity.unwrap()
    );

    clone.stop().await.expect("NvdClone stop must succeed");

    // Verify all scenario CVEs are in the registry (not just device_cves[0]).
    let mut verify_clone =
        NvdClone::new_with_scenario(&catalog).expect("second new_with_scenario must succeed");
    verify_clone
        .start()
        .await
        .expect("second NvdClone start must succeed");

    let verify_base = verify_clone.base_url();

    for (i, cve) in catalog.device_cves.iter().enumerate() {
        let verify_resp = client
            .get(format!("{verify_base}/rest/json/cves/2.0"))
            .query(&[("cveId", cve.as_str()), ("apiKey", "test-key")])
            .send()
            .await
            .expect("verify GET must succeed");

        assert_eq!(
            verify_resp.status().as_u16(),
            200,
            "device_cves[{i}]='{cve}' must resolve with HTTP 200 in new_with_scenario clone; \
             got {}. INV-NVD-CVE-CORRELATION-001",
            verify_resp.status().as_u16()
        );

        let verify_body: serde_json::Value = verify_resp
            .json()
            .await
            .expect("verify response must be JSON");
        let verify_score = verify_body["vulnerabilities"][0]["cve"]["metrics"]["cvssMetricV31"][0]
            ["cvssData"]["baseScore"]
            .as_f64()
            .unwrap_or(0.0);

        assert!(
            verify_score >= 7.0,
            "device_cves[{i}]='{cve}': baseScore must be >= 7.0; got {verify_score}"
        );
    }

    verify_clone
        .stop()
        .await
        .expect("verify clone stop must succeed");
}
