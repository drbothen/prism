//! Fidelity validation test for the CrowdStrike DTU (S-6.07 Task 11).
//!
//! Runs `FidelityValidator` against the unauthenticated / DTU-internal
//! endpoints only.  Auth-required endpoint shapes are covered by per-AC
//! integration tests (ac_1_happy_path.rs through ac_7_auth.rs), which carry
//! valid bearer tokens.  See ADR-003 for rationale.
//!
//! Covered endpoints (3 total — per ADR-003 §Decision Conflict #2 normative table):
//!   1. POST /oauth2/token  → 200, access_token field
//!   2. GET  /dtu/health    → 200, status field
//!   3. POST /dtu/reset     → 200, status field

#![allow(clippy::unwrap_used, clippy::expect_used)]

use prism_dtu_common::{BehavioralClone, FidelityCheck, FidelityValidator};
use prism_dtu_crowdstrike::CrowdstrikeClone;

/// Unauthenticated / DTU-introspection endpoints — scoped per ADR-003 §Conflict-2 Option C.
///
/// Covered (3 endpoints per ADR-003 normative table):
///   1. POST /oauth2/token  → 200, access_token field (unauthenticated by design)
///   2. GET  /dtu/health    → 200, status field (DTU introspection — no auth required)
///   3. POST /dtu/reset     → 200, status field (DTU introspection — no auth required)
///
/// Excluded (auth-required, covered by per-AC tests carrying bearer tokens):
///   - GET  /detects/queries/detects/v1
///   - POST /detects/entities/summaries/GET/v1
///   - GET  /devices/queries/devices/v1
///   - GET  /devices/entities/devices/v2
///   - POST /devices/entities/devices-actions/v2?action_name=contain
///   - POST /devices/entities/devices-actions/v2?action_name=lift_containment
///   - PATCH /detects/entities/detects/v2
///
/// When TD-WV1-01 is resolved (FidelityCheck gains a `headers` field), expand
/// to all 8 auth-required endpoints with bearer tokens and update count to 11.
// Scope: unauthenticated / DTU-internal endpoints only. Auth-required shapes
// are covered by per-AC integration tests. See ADR-003 for rationale.
#[tokio::test]
async fn crowdstrike_dtu_fidelity() {
    let mut clone = CrowdstrikeClone::new();
    clone
        .start()
        .await
        .expect("fidelity: CrowdstrikeClone::start() must succeed");
    let base_url = clone.base_url();
    let admin_token = clone.admin_token().to_string();

    let checks = vec![
        // Endpoint 1: OAuth2 token (unauthenticated by design).
        FidelityCheck {
            endpoint: "/oauth2/token".to_owned(),
            method: http::Method::POST,
            body: Some(serde_json::json!({
                "client_id": "fidelity-test",
                "client_secret": "fidelity-secret",
                "grant_type": "client_credentials"
            })),
            expected_status: 200,
            required_fields: vec!["access_token".to_owned()],
            ..Default::default()
        },
        // Endpoint 2: DTU health (introspection — no auth required).
        FidelityCheck {
            endpoint: "/dtu/health".to_owned(),
            method: http::Method::GET,
            body: None,
            expected_status: 200,
            required_fields: vec!["status".to_owned()],
            ..Default::default()
        },
        // Endpoint 3: DTU reset (requires X-Admin-Token per W3-FIX-SEC-002 / ADR-003 Amendment #5).
        FidelityCheck {
            endpoint: "/dtu/reset".to_owned(),
            method: http::Method::POST,
            body: None,
            expected_status: 200,
            required_fields: vec!["status".to_owned()],
            headers: vec![("X-Admin-Token".to_owned(), admin_token.clone())],
        },
    ];

    let report = FidelityValidator::run(&base_url, checks).await;

    assert_eq!(
        report.checks_failed, 0,
        "fidelity: {} of 3 endpoint check(s) failed:\n{:#?}",
        report.checks_failed, report.failures
    );
    assert_eq!(
        report.checks_passed, 3,
        "fidelity: expected 3 checks passed, got {}",
        report.checks_passed
    );
}
