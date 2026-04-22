//! Fidelity validation test for the CrowdStrike DTU (S-6.07 Task 11).
//!
//! Runs `FidelityValidator` against all 8 in-scope endpoints and asserts
//! `FidelityReport.checks_failed == 0`. Must run as part of `just dtu-validate`.
//!
//! Expected Red Gate failure: `CrowdstrikeClone::start()` is `unimplemented!()`,
//! so the server never starts and all 8 fidelity checks fail.

use prism_dtu_common::{BehavioralClone, FidelityCheck, FidelityValidator};
use prism_dtu_crowdstrike::CrowdstrikeClone;

/// All 8 in-scope CrowdStrike DTU endpoints checked for fidelity.
///
/// Endpoints:
/// 1. GET /detects/queries/detects/v1          → 200, resources field
/// 2. POST /detects/entities/summaries/GET/v1  → 200, resources field
/// 3. GET /devices/queries/devices/v1          → 200, resources field
/// 4. GET /devices/entities/devices/v2         → 200, resources field
/// 5. POST /devices/entities/devices-actions/v2?action_name=contain → 202, resources field
/// 6. POST /devices/entities/devices-actions/v2?action_name=lift_containment → 202, resources field
/// 7. PATCH /detects/entities/detects/v2 (update_status) → 200
/// 8. POST /oauth2/token                       → 200, access_token field
#[tokio::test]
async fn crowdstrike_dtu_fidelity() {
    // Expected failure: CrowdstrikeClone::start() panics with "not yet implemented".
    let mut clone = CrowdstrikeClone::new();
    clone
        .start()
        .await
        .expect("fidelity: CrowdstrikeClone::start() must succeed");
    let base_url = clone.base_url();

    // Build fidelity check suite for all 8 endpoints.
    // FidelityValidator does NOT send auth headers; the DTU must allow these through
    // for the fidelity probe (or the check records the actual status code seen).
    // We use the expected status codes from the story spec.
    let checks = vec![
        // Endpoint 1: Detection ID list (Step 1).
        FidelityCheck {
            endpoint: "/detects/queries/detects/v1".to_owned(),
            method: http::Method::GET,
            body: None,
            expected_status: 200,
            required_fields: vec!["resources".to_owned(), "meta".to_owned()],
        },
        // Endpoint 2: Detection detail batch (Step 2).
        FidelityCheck {
            endpoint: "/detects/entities/summaries/GET/v1".to_owned(),
            method: http::Method::POST,
            body: Some(serde_json::json!({"ids": ["det-001"]})),
            expected_status: 200,
            required_fields: vec!["resources".to_owned()],
        },
        // Endpoint 3: Host ID list (Step 1).
        FidelityCheck {
            endpoint: "/devices/queries/devices/v1".to_owned(),
            method: http::Method::GET,
            body: None,
            expected_status: 200,
            required_fields: vec!["resources".to_owned(), "meta".to_owned()],
        },
        // Endpoint 4: Host detail batch (Step 2).
        FidelityCheck {
            endpoint: "/devices/entities/devices/v2?ids=h-001".to_owned(),
            method: http::Method::GET,
            body: None,
            expected_status: 200,
            required_fields: vec!["resources".to_owned()],
        },
        // Endpoint 5: Contain device (write).
        FidelityCheck {
            endpoint: "/devices/entities/devices-actions/v2?action_name=contain".to_owned(),
            method: http::Method::POST,
            body: Some(serde_json::json!({"ids": ["h-fidelity-001"]})),
            expected_status: 202,
            required_fields: vec!["resources".to_owned()],
        },
        // Endpoint 6: Lift containment (write).
        FidelityCheck {
            endpoint: "/devices/entities/devices-actions/v2?action_name=lift_containment"
                .to_owned(),
            method: http::Method::POST,
            body: Some(serde_json::json!({"ids": ["h-fidelity-001"]})),
            expected_status: 202,
            required_fields: vec!["resources".to_owned()],
        },
        // Endpoint 7: Update detection status (write).
        FidelityCheck {
            endpoint: "/detects/entities/detects/v2".to_owned(),
            method: http::Method::PATCH,
            body: Some(serde_json::json!({"ids": ["det-001"], "status": "in_progress"})),
            expected_status: 200,
            required_fields: vec![],
        },
        // Endpoint 8: OAuth2 token.
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
    ];

    let report = FidelityValidator::run(&base_url, checks).await;

    assert_eq!(
        report.checks_failed, 0,
        "fidelity: {} of 8 endpoint checks failed:\n{:#?}",
        report.checks_failed, report.failures
    );
    assert_eq!(
        report.checks_passed, 8,
        "fidelity: expected 8 checks passed, got {}",
        report.checks_passed
    );
}
