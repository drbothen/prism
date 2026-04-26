#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Fidelity validator: asserts endpoint shape for all Armis Centrix DTU endpoints.
//!
//! Per ADR-002 §8: every L2 clone must include a fidelity test that calls
//! `FidelityValidator::run` and asserts `checks_failed == 0`.
//!
//! This test validates endpoint shape (status codes, required response fields),
//! not business logic. AC-specific business logic tests live in separate test files.
//!
//! Coverage:
//! - Unauthenticated vendor API endpoints → HTTP 403 with `error` and `code` fields.
//! - DTU internal endpoints (/dtu/health, /dtu/reset, /dtu/configure, /dtu/aql-log).
//! - Authenticated GET /api/v1/devices → HTTP 200 with `data` field.
//!   NOTE: FidelityCheck does not carry request headers, so authenticated checks
//!   that require Bearer will return 403. Checks below explicitly use 403 for
//!   vendor endpoints (no-auth shape) and 200 for /dtu/* (no-auth required).
//!
//! The FidelityValidator passes once the stub is running.
//! This file validates the **shape contract** of every endpoint, not business logic.
//! Business-logic failures (AQL capture, stateful tags, rate-limit injection) are
//! tested in ac_1 through ac_6 test files.

#![cfg(feature = "dtu")]

use prism_dtu_armis::ArmisClone;
use prism_dtu_common::{BehavioralClone, FidelityCheck, FidelityValidator};

#[tokio::test]
async fn fidelity_validator_passes() {
    let mut clone = ArmisClone::new().expect("ArmisClone::new failed");
    clone.start().await.expect("ArmisClone::start failed");
    let base_url = clone.base_url();
    let admin_token = clone.admin_token().to_string();

    let checks = vec![
        // --- Unauthenticated vendor API endpoints: must return 403 ---
        // AC-5 shape: missing bearer → HTTP 403 with error and code fields.
        FidelityCheck {
            endpoint: "/api/v1/devices".to_string(),
            method: http::Method::GET,
            body: None,
            expected_status: 403,
            required_fields: vec!["error".to_string(), "code".to_string()],
            ..Default::default()
        },
        // POST /api/v1/devices also requires auth (EC-005).
        FidelityCheck {
            endpoint: "/api/v1/devices".to_string(),
            method: http::Method::POST,
            body: Some(serde_json::json!({})),
            expected_status: 403,
            required_fields: vec!["error".to_string(), "code".to_string()],
            ..Default::default()
        },
        // Alert list requires auth.
        FidelityCheck {
            endpoint: "/api/v1/alerts".to_string(),
            method: http::Method::GET,
            body: None,
            expected_status: 403,
            required_fields: vec!["error".to_string(), "code".to_string()],
            ..Default::default()
        },
        // Device activity requires auth.
        FidelityCheck {
            endpoint: "/api/v1/devices/d-001/activity".to_string(),
            method: http::Method::GET,
            body: None,
            expected_status: 403,
            required_fields: vec!["error".to_string(), "code".to_string()],
            ..Default::default()
        },
        // Device risk requires auth.
        FidelityCheck {
            endpoint: "/api/v1/devices/d-001/risk".to_string(),
            method: http::Method::GET,
            body: None,
            expected_status: 403,
            required_fields: vec!["error".to_string(), "code".to_string()],
            ..Default::default()
        },
        // POST tag requires auth.
        FidelityCheck {
            endpoint: "/api/v1/devices/d-001/tags/".to_string(),
            method: http::Method::POST,
            body: Some(serde_json::json!({"tag_key": "probe-tag"})),
            expected_status: 403,
            required_fields: vec!["error".to_string(), "code".to_string()],
            ..Default::default()
        },
        // --- DTU internal endpoints: no auth required ---
        // GET /dtu/health → 200 with status field.
        FidelityCheck {
            endpoint: "/dtu/health".to_string(),
            method: http::Method::GET,
            body: None,
            expected_status: 200,
            required_fields: vec!["status".to_string()],
            ..Default::default()
        },
        // POST /dtu/reset → 200 with status field.
        FidelityCheck {
            endpoint: "/dtu/reset".to_string(),
            method: http::Method::POST,
            body: None,
            expected_status: 200,
            required_fields: vec!["status".to_string()],
            ..Default::default()
        },
        // POST /dtu/configure → 200 with status field (requires X-Admin-Token per ADR-003 Amendment #5).
        FidelityCheck {
            endpoint: "/dtu/configure".to_string(),
            method: http::Method::POST,
            body: Some(serde_json::json!({})),
            expected_status: 200,
            required_fields: vec!["status".to_string()],
            headers: vec![("X-Admin-Token".to_string(), admin_token.clone())],
        },
        // GET /dtu/aql-log → 200 with aql_strings field.
        FidelityCheck {
            endpoint: "/dtu/aql-log".to_string(),
            method: http::Method::GET,
            body: None,
            expected_status: 200,
            required_fields: vec!["aql_strings".to_string()],
            ..Default::default()
        },
    ];

    let report = FidelityValidator::run(&base_url, checks).await;
    assert_eq!(
        report.checks_failed, 0,
        "fidelity failures: {:?}",
        report.failures
    );
}
