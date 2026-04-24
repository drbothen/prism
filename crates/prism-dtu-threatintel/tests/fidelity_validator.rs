#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Fidelity validation test for `prism-dtu-threatintel` (ADR-003 Amendment #4).
//!
//! Starts `ThreatIntelClone`, runs `FidelityValidator` against key routes and asserts
//! `checks_failed == 0`. Covers the DTU control plane (health/reset/configure) plus
//! the IP lookup endpoint with and without auth.
//!
//! Per ADR-003 Amendment #3, auth-required endpoint checks use the `headers` field
//! to inject `Authorization: Bearer demo-token`.

use prism_dtu_common::{BehavioralClone, FidelityCheck, FidelityValidator};
use prism_dtu_threatintel::ThreatIntelClone;

#[tokio::test]
async fn threatintel_dtu_fidelity() {
    let mut clone = ThreatIntelClone::new();
    clone
        .start()
        .await
        .expect("ThreatIntelClone::start() must succeed");
    let base_url = clone.base_url();

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
        // Check 2: POST /dtu/reset — resets mutable state, no auth required.
        FidelityCheck {
            endpoint: "/dtu/reset".to_string(),
            method: http::Method::POST,
            body: None,
            expected_status: 200,
            required_fields: vec!["status".to_string()],
            ..Default::default()
        },
        // Check 3: POST /dtu/configure — runtime reconfiguration, no auth required.
        FidelityCheck {
            endpoint: "/dtu/configure".to_string(),
            method: http::Method::POST,
            body: Some(serde_json::json!({"rate_limit_after": 100})),
            expected_status: 200,
            required_fields: vec!["status".to_string()],
            ..Default::default()
        },
        // Check 4: GET /v3/ip/<known-malicious> with auth header → 200 with threat_score field.
        // Uses ADR-003 Amendment #3 headers field to inject Authorization.
        FidelityCheck {
            endpoint: "/v3/ip/45.55.100.1".to_string(),
            method: http::Method::GET,
            body: None,
            expected_status: 200,
            required_fields: vec!["threat_score".to_string()],
            headers: vec![(
                "Authorization".to_string(),
                "Bearer demo-token".to_string(),
            )],
        },
        // Check 5: GET /v3/ip/<known-benign> with auth header → 200 with threat_score field.
        FidelityCheck {
            endpoint: "/v3/ip/8.8.8.8".to_string(),
            method: http::Method::GET,
            body: None,
            expected_status: 200,
            required_fields: vec!["threat_score".to_string()],
            headers: vec![(
                "Authorization".to_string(),
                "Bearer demo-token".to_string(),
            )],
        },
        // Check 6: GET /v3/ip/<known-malicious> without auth header → 401 (missing API key).
        FidelityCheck {
            endpoint: "/v3/ip/45.55.100.1".to_string(),
            method: http::Method::GET,
            body: None,
            expected_status: 401,
            required_fields: vec!["error".to_string()],
            ..Default::default()
        },
    ];

    let report = FidelityValidator::run(&base_url, checks).await;

    if report.checks_failed > 0 {
        for failure in &report.failures {
            eprintln!(
                "THREATINTEL FIDELITY FAILURE [{}]: {}",
                failure.endpoint, failure.reason
            );
        }
    }

    assert_eq!(
        report.checks_failed,
        0,
        "ThreatIntel FidelityValidator: {}/{} checks passed, {} failed",
        report.checks_passed,
        report.checks_passed + report.checks_failed,
        report.checks_failed,
    );
}
