//! AC-7 — Fidelity validator: asserts endpoint shape for all Armis Centrix DTU endpoints.
//!
//! Per ADR-002 §8: every L2 clone must include a fidelity test that calls
//! `FidelityValidator::run` and asserts `checks_failed == 0`.
//!
//! This test validates endpoint shape (status codes, required response fields),
//! not business logic. AC-specific business logic tests live in separate test files.
//!
//! Note: `FidelityCheck` does not carry request headers, so checks that require
//! bearer auth will return 403. Bearer-auth success paths are validated in
//! per-AC integration tests. This test validates DTU health, AQL log, and
//! auth-error response shapes.

#![cfg(feature = "dtu")]

use prism_dtu_armis::ArmisClone;
use prism_dtu_common::{BehavioralClone, FidelityCheck, FidelityValidator};

#[tokio::test]
async fn fidelity_validator_passes() {
    let mut clone = ArmisClone::new().expect("ArmisClone::new failed");
    clone.start().await.expect("ArmisClone::start failed");
    let base_url = clone.base_url();

    let checks = vec![
        // AC-5 shape: missing bearer → HTTP 403 with error and code fields
        FidelityCheck {
            endpoint: "/api/v1/devices".to_string(),
            method: http::Method::GET,
            body: None,
            expected_status: 403,
            required_fields: vec!["error".to_string(), "code".to_string()],
        },
        // Auth required for alerts endpoint too
        FidelityCheck {
            endpoint: "/api/v1/alerts".to_string(),
            method: http::Method::GET,
            body: None,
            expected_status: 403,
            required_fields: vec!["error".to_string(), "code".to_string()],
        },
        // DTU health endpoint — no auth required, per ADR-002 §6
        FidelityCheck {
            endpoint: "/dtu/health".to_string(),
            method: http::Method::GET,
            body: None,
            expected_status: 200,
            required_fields: vec!["status".to_string()],
        },
        // DTU reset endpoint — no auth required
        FidelityCheck {
            endpoint: "/dtu/reset".to_string(),
            method: http::Method::POST,
            body: None,
            expected_status: 200,
            required_fields: vec!["status".to_string()],
        },
        // DTU configure endpoint — no auth required
        FidelityCheck {
            endpoint: "/dtu/configure".to_string(),
            method: http::Method::POST,
            body: Some(serde_json::json!({})),
            expected_status: 200,
            required_fields: vec!["status".to_string()],
        },
        // AQL log endpoint — no auth required
        FidelityCheck {
            endpoint: "/dtu/aql-log".to_string(),
            method: http::Method::GET,
            body: None,
            expected_status: 200,
            required_fields: vec!["aql_strings".to_string()],
        },
    ];

    let report = FidelityValidator::run(&base_url, checks).await;
    assert_eq!(
        report.checks_failed,
        0,
        "fidelity failures: {:?}",
        report.failures
    );
}
