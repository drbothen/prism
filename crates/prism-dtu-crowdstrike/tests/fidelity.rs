//! Fidelity validation test for the CrowdStrike DTU (S-6.07 Task 11).
//!
//! Runs `FidelityValidator` against the unauthenticated / DTU-internal
//! endpoints only.  Auth-required endpoint shapes are covered by per-AC
//! integration tests (ac_1_happy_path.rs through ac_7_auth.rs), which carry
//! valid bearer tokens.  See ADR-003 for rationale.
//!
//! Expected Red Gate failure: `CrowdstrikeClone::start()` is `unimplemented!()`,
//! so the server never starts and all fidelity checks fail.
//!
//! NOTE: `/dtu/health` and `/dtu/reset` are not yet exposed by this clone.
//! When those routes are added (ADR-002 compliance follow-up), add two more
//! FidelityCheck entries here and update the `checks_passed` assertion to 3.

use prism_dtu_common::{BehavioralClone, FidelityCheck, FidelityValidator};
use prism_dtu_crowdstrike::CrowdstrikeClone;

/// Unauthenticated endpoints only — scoped per ADR-003 §Conflict-2 Option C.
///
/// Covered:
///   1. POST /oauth2/token  → 200, access_token field
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
/// Pending (DTU-internal endpoints not yet exposed — follow-up before ADR-002
/// full compliance can be claimed):
///   - GET  /dtu/health
///   - POST /dtu/reset
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
        },
        // /dtu/health and /dtu/reset are not yet exposed by this clone.
        // When implemented, add FidelityCheck entries here and update the
        // checks_passed assertion from 1 to 3.
    ];

    let report = FidelityValidator::run(&base_url, checks).await;

    assert_eq!(
        report.checks_failed,
        0,
        "fidelity: {} of 1 endpoint check(s) failed:\n{:#?}",
        report.checks_failed,
        report.failures
    );
    assert_eq!(
        report.checks_passed,
        1,
        "fidelity: expected 1 check passed, got {}",
        report.checks_passed
    );
}
