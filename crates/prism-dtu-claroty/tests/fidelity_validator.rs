#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Fidelity validation test for `prism-dtu-claroty`.
//!
//! Starts `ClarotyClone`, runs `FidelityValidator` against all 9 routes
//! (7 API + 2 DTU control), and asserts `checks_failed == 0`.
//! Run as part of `just dtu-validate`.

use prism_dtu_claroty::ClarotyClone;
use prism_dtu_common::{BehavioralClone, FidelityCheck, FidelityValidator};
use serde_json::json;

#[tokio::test]
async fn claroty_dtu_fidelity() {
    let mut clone = ClarotyClone::new();
    clone.start().await.expect("ClarotyClone::start failed");
    let base_url = clone.base_url();
    let admin_token = clone.admin_token().to_string();

    let checks = vec![
        // Route 1: POST /api/v1/devices — normal list.
        FidelityCheck {
            endpoint: "/api/v1/devices".to_string(),
            method: http::Method::POST,
            body: Some(json!({})),
            expected_status: 401, // No auth header → must reject
            required_fields: vec!["error".to_string()],
            ..Default::default()
        },
        // Route 2: POST /api/v1/alerts.
        FidelityCheck {
            endpoint: "/api/v1/alerts".to_string(),
            method: http::Method::POST,
            body: Some(json!({})),
            expected_status: 401,
            required_fields: vec!["error".to_string()],
            ..Default::default()
        },
        // Route 3: POST /api/v1/alerts/:id/devices.
        FidelityCheck {
            endpoint: "/api/v1/alerts/1/devices".to_string(),
            method: http::Method::POST,
            body: Some(json!({})),
            expected_status: 401,
            required_fields: vec!["error".to_string()],
            ..Default::default()
        },
        // Route 4: POST /api/v1/vulnerabilities.
        FidelityCheck {
            endpoint: "/api/v1/vulnerabilities".to_string(),
            method: http::Method::POST,
            body: Some(json!({})),
            expected_status: 401,
            required_fields: vec!["error".to_string()],
            ..Default::default()
        },
        // Route 5: POST /api/v1/vulnerabilities/:id/devices.
        FidelityCheck {
            endpoint: "/api/v1/vulnerabilities/vuln-001/devices".to_string(),
            method: http::Method::POST,
            body: Some(json!({})),
            expected_status: 401,
            required_fields: vec!["error".to_string()],
            ..Default::default()
        },
        // Route 6: POST /api/v1/devices/:id/tags/ (write path — 401 without auth).
        FidelityCheck {
            endpoint: "/api/v1/devices/asset-001/tags/".to_string(),
            method: http::Method::POST,
            body: Some(json!({"tag_key": "fidelity-tag", "tag_value": "true"})),
            expected_status: 401,
            required_fields: vec!["error".to_string()],
            ..Default::default()
        },
        // Route 7: DELETE /api/v1/devices/:id/tags/:key (write path — 401 without auth).
        FidelityCheck {
            endpoint: "/api/v1/devices/asset-001/tags/fidelity-tag".to_string(),
            method: http::Method::DELETE,
            body: None,
            expected_status: 401,
            required_fields: vec!["error".to_string()],
            ..Default::default()
        },
        // Route 8: POST /dtu/configure (control endpoint — requires X-Admin-Token per ADR-003 Amendment #5).
        FidelityCheck {
            endpoint: "/dtu/configure".to_string(),
            method: http::Method::POST,
            body: Some(json!({"rate_limit_after": 100})),
            expected_status: 200,
            required_fields: vec![],
            headers: vec![("X-Admin-Token".to_string(), admin_token.clone())],
        },
        // Route 9: POST /dtu/reset (control endpoint — requires X-Admin-Token per W3-FIX-SEC-002 / ADR-003 Amendment #5).
        FidelityCheck {
            endpoint: "/dtu/reset".to_string(),
            method: http::Method::POST,
            body: None,
            expected_status: 200,
            required_fields: vec![],
            headers: vec![("X-Admin-Token".to_string(), admin_token.clone())],
        },
        // Route 10: GET /dtu/health (liveness — no auth required, no state access).
        FidelityCheck {
            endpoint: "/dtu/health".to_string(),
            method: http::Method::GET,
            body: None,
            expected_status: 200,
            required_fields: vec!["status".to_string()],
            ..Default::default()
        },
    ];

    let report = FidelityValidator::run(&base_url, checks).await;

    if report.checks_failed > 0 {
        for failure in &report.failures {
            eprintln!(
                "FIDELITY FAILURE [{}]: {}",
                failure.endpoint, failure.reason
            );
        }
    }

    assert_eq!(
        report.checks_failed,
        0,
        "FidelityValidator: {}/{} checks passed, {} failed",
        report.checks_passed,
        report.checks_passed + report.checks_failed,
        report.checks_failed,
    );
}
