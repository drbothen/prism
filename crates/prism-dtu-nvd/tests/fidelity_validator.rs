#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Fidelity validation test for `prism-dtu-nvd` (ADR-003 Amendment #4).
//!
//! Starts `NvdClone`, runs `FidelityValidator` against key routes and asserts
//! `checks_failed == 0`. Covers the DTU health/reset/configure control plane
//! plus the main CVE lookup endpoint (unauthenticated shape).
//!
//! Auth-required endpoint shapes (apiKey parameter) are covered by per-AC
//! integration tests (ac_4_unauthenticated_rate_limit, ac_5_authenticated_rate_limit).

use prism_dtu_common::{BehavioralClone, FidelityCheck, FidelityValidator};
use prism_dtu_nvd::NvdClone;

#[tokio::test]
async fn nvd_dtu_fidelity() {
    let mut clone = NvdClone::new().expect("NvdClone::new must succeed");
    clone.start().await.expect("NvdClone::start() must succeed");
    let base_url = clone.base_url();
    let admin_token = clone.admin_token().to_string();

    let checks = vec![
        // Check 1: GET /dtu/health — liveness probe, no auth required.
        FidelityCheck {
            endpoint: "/dtu/health".to_string(),
            method: http::Method::GET,
            body: None,
            expected_status: 200,
            required_fields: vec!["status".to_string()],
            ..Default::default()
        },
        // Check 2: POST /dtu/reset — resets mutable state, requires X-Admin-Token per ADR-003 Amendment #5.
        FidelityCheck {
            endpoint: "/dtu/reset".to_string(),
            method: http::Method::POST,
            body: None,
            expected_status: 200,
            required_fields: vec!["status".to_string()],
            headers: vec![("X-Admin-Token".to_string(), admin_token.clone())],
        },
        // Check 3: POST /dtu/configure — runtime reconfiguration (requires X-Admin-Token per ADR-003 Amendment #5).
        FidelityCheck {
            endpoint: "/dtu/configure".to_string(),
            method: http::Method::POST,
            body: Some(serde_json::json!({"auth_mode": "accept"})),
            expected_status: 200,
            required_fields: vec!["status".to_string()],
            headers: vec![("X-Admin-Token".to_string(), admin_token.clone())],
        },
        // Check 4: GET /rest/json/cves/2.0?cveId=CVE-2024-0001 — unauthenticated CVE lookup.
        // The NVD fixture ships with CVE-2024-0001 so this must return 200 with `vulnerabilities`.
        FidelityCheck {
            endpoint: "/rest/json/cves/2.0?cveId=CVE-2024-0001".to_string(),
            method: http::Method::GET,
            body: None,
            expected_status: 200,
            required_fields: vec!["vulnerabilities".to_string()],
            ..Default::default()
        },
        // Check 5: GET /rest/json/cves/2.0?cveId=CVE-9999-0000 — unknown CVE → 404.
        FidelityCheck {
            endpoint: "/rest/json/cves/2.0?cveId=CVE-9999-0000".to_string(),
            method: http::Method::GET,
            body: None,
            expected_status: 404,
            required_fields: vec![],
            ..Default::default()
        },
    ];

    let report = FidelityValidator::run(&base_url, checks).await;

    if report.checks_failed > 0 {
        for failure in &report.failures {
            eprintln!(
                "NVD FIDELITY FAILURE [{}]: {}",
                failure.endpoint, failure.reason
            );
        }
    }

    assert_eq!(
        report.checks_failed,
        0,
        "NVD FidelityValidator: {}/{} checks passed, {} failed",
        report.checks_passed,
        report.checks_passed + report.checks_failed,
        report.checks_failed,
    );
}
