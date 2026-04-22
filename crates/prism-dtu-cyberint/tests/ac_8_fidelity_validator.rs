//! AC-8 Fidelity Validator — verifies endpoint shape compliance for Cyberint DTU
//! endpoints that do not require cookie auth (ADR-002 §8).
//!
//! Authenticated-endpoint shape checks are covered in ac_1_cookie_auth_roundtrip.rs
//! and the per-AC tests; the FidelityValidator itself does not carry session cookies.

#[cfg(feature = "dtu")]
#[tokio::test]
async fn fidelity_validator_passes() {
    use prism_dtu_common::{BehavioralClone, FidelityCheck, FidelityValidator};
    use prism_dtu_cyberint::CyberintClone;

    let mut clone = CyberintClone::new().expect("clone init");
    clone.start().await.expect("clone start");
    let base_url = clone.base_url();

    // Fidelity checks limited to endpoints that do not require cookie auth,
    // plus the 401 shape check for unauthenticated access.
    let checks = vec![
        // DTU health endpoint (ADR-002 required, no auth).
        FidelityCheck {
            endpoint: "/dtu/health".to_string(),
            method: http::Method::GET,
            body: None,
            expected_status: 200,
            required_fields: vec!["status".to_string()],
        },
        // Login endpoint shape check.
        FidelityCheck {
            endpoint: "/login".to_string(),
            method: http::Method::POST,
            body: Some(serde_json::json!({})),
            expected_status: 200,
            required_fields: vec!["message".to_string()],
        },
        // Unauthenticated access to alerts must return 401 with "error" field.
        FidelityCheck {
            endpoint: "/api/v1/alerts".to_string(),
            method: http::Method::GET,
            body: None,
            expected_status: 401,
            required_fields: vec!["error".to_string()],
        },
        // DTU configure and reset return {"status": "ok"}.
        FidelityCheck {
            endpoint: "/dtu/configure".to_string(),
            method: http::Method::POST,
            body: Some(serde_json::json!({})),
            expected_status: 200,
            required_fields: vec!["status".to_string()],
        },
        FidelityCheck {
            endpoint: "/dtu/reset".to_string(),
            method: http::Method::POST,
            body: None,
            expected_status: 200,
            required_fields: vec!["status".to_string()],
        },
    ];

    let report = FidelityValidator::run(&base_url, checks).await;
    assert_eq!(
        report.checks_failed, 0,
        "fidelity failures: {:?}",
        report.failures
    );
}
