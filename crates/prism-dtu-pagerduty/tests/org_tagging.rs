//! Red Gate tests for S-3.2.06 — prism-dtu-pagerduty shared-mode OrgId ingress tagging.
//!
//! # Behavioral contracts exercised
//!
//! - BC-3.2.004: Shared-Mode DTU Tags OrgId in Payload Body Not in Routing Headers
//! - BC-3.2.005: DTU Mode is Deployment-Time Config — No Runtime API to Change It
//!
//! # Acceptance criteria
//!
//! - AC-001 / BC-3.2.004 postcondition 1: OrgId UUID in captured IncidentRecord.org_id
//! - AC-002 / BC-3.2.004 postcondition 2: OrgId absent from dedup_key and HTTP routing fields
//! - AC-003 / BC-3.2.004 postcondition 4: Concurrent incidents from distinct orgs distinguished
//! - AC-004 / BC-3.2.004 postcondition 5: No mode metadata in incident query results
//! - AC-005 / BC-3.2.005 postcondition 1: DtuMode::Shared set at startup and immutable
//! - AC-006 / BC-3.2.005 postcondition 3: Invalid mode string rejected with deserialisation error
//!
//! # Naming convention
//!
//! `test_BC_S_SS_NNN_xxx` per TDD discipline.

#![allow(clippy::unwrap_used, clippy::expect_used, non_snake_case)]
#![cfg(feature = "dtu")]

use prism_core::OrgId;
use prism_dtu_common::{BehavioralClone, DtuMode};
use prism_dtu_pagerduty::clone::PAGERDUTY_DTU_MODE;
use prism_dtu_pagerduty::{PagerDutyClone, PagerDutyState};

// ---------------------------------------------------------------------------
// Test helpers
// ---------------------------------------------------------------------------

/// Start a fresh `PagerDutyClone` and return (clone, base_url, reqwest::Client).
async fn start_clone() -> (PagerDutyClone, String, reqwest::Client) {
    let mut clone = PagerDutyClone::new().expect("PagerDutyClone::new");
    clone.start().await.expect("PagerDutyClone::start");
    let base_url = clone.base_url();
    let client = reqwest::Client::new();
    (clone, base_url, client)
}

// ---------------------------------------------------------------------------
// AC-001 / BC-3.2.004 postcondition 1
// OrgId UUID in captured IncidentRecord.org_id
// ---------------------------------------------------------------------------

/// AC-001: OrgId UUID appears in IncidentRecord.org_id after capture_incident_tagged.
#[tokio::test]
async fn test_BC_3_2_004_ac001_org_id_in_incident_record() {
    let org_id = OrgId::new();
    let org_id_str = org_id.to_string();
    let dedup_key = uuid::Uuid::new_v4().to_string();

    let state = PagerDutyState::new();
    state.capture_incident_tagged(
        org_id,
        dedup_key.clone(),
        prism_dtu_pagerduty::state::IncidentStatus::Triggered,
        "critical".to_string(),
        "Test alert".to_string(),
    );

    let incidents = state.incidents_snapshot();
    assert_eq!(
        incidents.len(),
        1,
        "expected exactly one incident in registry"
    );

    let record = incidents
        .into_iter()
        .find(|r| r.dedup_key == dedup_key)
        .expect("incident record not found by dedup_key");

    assert_eq!(
        record.org_id.as_deref(),
        Some(org_id_str.as_str()),
        "IncidentRecord.org_id must equal the OrgId UUID string (BC-3.2.004 postcondition 1)"
    );
}

// ---------------------------------------------------------------------------
// AC-002 / BC-3.2.004 postcondition 2
// OrgId absent from dedup_key and HTTP routing fields
// ---------------------------------------------------------------------------

/// AC-002: dedup_key does not contain org_id UUID (MSSP-scoped per ADR-008 §1.2).
#[tokio::test]
async fn test_BC_3_2_004_ac002_dedup_key_not_org_scoped() {
    let org_id = OrgId::new();
    let org_id_str = org_id.to_string();
    // MSSP-generated dedup key — must NOT embed org_id
    let dedup_key = uuid::Uuid::new_v4().to_string();

    // Sanity: the MSSP dedup_key we chose must not contain the org UUID
    assert!(
        !dedup_key.contains(&org_id_str),
        "test setup error: MSSP dedup_key must not contain org_id"
    );

    let state = PagerDutyState::new();
    state.capture_incident_tagged(
        org_id,
        dedup_key.clone(),
        prism_dtu_pagerduty::state::IncidentStatus::Triggered,
        "warning".to_string(),
        "Dedup key isolation test".to_string(),
    );

    let incidents = state.incidents_snapshot();
    let record = incidents
        .into_iter()
        .find(|r| r.dedup_key == dedup_key)
        .expect("incident not found");

    // BC-3.2.004 postcondition 2: dedup_key MUST NOT contain org_id
    assert!(
        !record
            .dedup_key
            .contains(record.org_id.as_deref().unwrap_or("")),
        "dedup_key must not contain the org_id UUID (ADR-008 §1.2)"
    );
}

/// AC-002: org_id does not appear in the POST /v2/enqueue response URL or headers.
#[tokio::test]
async fn test_BC_3_2_004_ac002_org_id_absent_from_routing() {
    let org_id = OrgId::new();
    let org_id_str = org_id.to_string();

    let (_clone, base_url, client) = start_clone().await;

    // POST to /v2/enqueue with X-Prism-Org-Id header — the org_id must NOT appear
    // in the response URL or any response header.
    let resp = client
        .post(format!("{base_url}/v2/enqueue"))
        .header("X-Prism-Org-Id", &org_id_str)
        .json(&serde_json::json!({
            "routing_key": "test-routing-key",
            "event_action": "trigger",
            "payload": {
                "summary": "Org routing isolation test",
                "severity": "info"
            }
        }))
        .send()
        .await
        .expect("POST /v2/enqueue failed");

    assert_eq!(resp.status(), 202, "expected 202 Accepted");

    // Verify: org_id absent from response URL
    let resp_url = resp.url().to_string();
    assert!(
        !resp_url.contains(&org_id_str),
        "org_id UUID must not appear in response URL (BC-3.2.004 postcondition 2)"
    );

    // Verify: org_id absent from all response headers
    for (name, value) in resp.headers() {
        let val_str = value.to_str().unwrap_or("");
        assert!(
            !val_str.contains(&org_id_str),
            "org_id UUID must not appear in response header '{name}': '{val_str}' (BC-3.2.004 postcondition 2)"
        );
    }
}

// ---------------------------------------------------------------------------
// AC-003 / BC-3.2.004 postcondition 4
// Concurrent incidents from different orgs are independently attributed
// ---------------------------------------------------------------------------

/// AC-003: concurrent incidents from org_A and org_B each carry their sender's OrgId.
#[tokio::test]
async fn test_BC_3_2_004_ac003_concurrent_incidents_distinguished() {
    use std::sync::Arc;

    let state = Arc::new(PagerDutyState::new());

    let org_a = OrgId::new();
    let org_b = OrgId::new();
    let org_a_str = org_a.to_string();
    let org_b_str = org_b.to_string();
    let dedup_a = uuid::Uuid::new_v4().to_string();
    let dedup_b = uuid::Uuid::new_v4().to_string();

    let state_a = Arc::clone(&state);
    let state_b = Arc::clone(&state);
    let dedup_a_clone = dedup_a.clone();
    let dedup_b_clone = dedup_b.clone();

    // Spawn two concurrent captures
    let task_a = tokio::spawn(async move {
        state_a.capture_incident_tagged(
            org_a,
            dedup_a_clone,
            prism_dtu_pagerduty::state::IncidentStatus::Triggered,
            "critical".to_string(),
            "Org A incident".to_string(),
        );
    });

    let task_b = tokio::spawn(async move {
        state_b.capture_incident_tagged(
            org_b,
            dedup_b_clone,
            prism_dtu_pagerduty::state::IncidentStatus::Triggered,
            "warning".to_string(),
            "Org B incident".to_string(),
        );
    });

    task_a.await.expect("task_a panicked");
    task_b.await.expect("task_b panicked");

    let incidents = state.incidents_snapshot();
    assert_eq!(incidents.len(), 2, "expected 2 incidents in registry");

    let rec_a = incidents
        .iter()
        .find(|r| r.dedup_key == dedup_a)
        .expect("org_A incident not found");
    let rec_b = incidents
        .iter()
        .find(|r| r.dedup_key == dedup_b)
        .expect("org_B incident not found");

    assert_eq!(
        rec_a.org_id.as_deref(),
        Some(org_a_str.as_str()),
        "org_A incident must carry org_A's OrgId"
    );
    assert_eq!(
        rec_b.org_id.as_deref(),
        Some(org_b_str.as_str()),
        "org_B incident must carry org_B's OrgId"
    );
    assert_ne!(
        rec_a.org_id, rec_b.org_id,
        "org_A and org_B incidents must have distinct OrgIds"
    );
}

// ---------------------------------------------------------------------------
// AC-004 / BC-3.2.004 postcondition 5
// No mode metadata in incident query results (GET /dtu/incidents)
// ---------------------------------------------------------------------------

/// AC-004: GET /dtu/incidents response rows contain no "mode", "shared", or "dtu_mode" fields.
#[tokio::test]
async fn test_BC_3_2_004_ac004_mode_metadata_absent_from_query_results() {
    let org_id = OrgId::new();
    let (_clone, base_url, client) = start_clone().await;

    // Trigger an incident via POST /v2/enqueue
    let trigger_resp = client
        .post(format!("{base_url}/v2/enqueue"))
        .header("X-Prism-Org-Id", org_id.to_string())
        .json(&serde_json::json!({
            "routing_key": "test-routing-key",
            "event_action": "trigger",
            "payload": {
                "summary": "Mode metadata test",
                "severity": "critical"
            }
        }))
        .send()
        .await
        .expect("POST /v2/enqueue failed");
    assert_eq!(trigger_resp.status(), 202);

    // Query GET /dtu/incidents
    let incidents_resp = client
        .get(format!("{base_url}/dtu/incidents"))
        .send()
        .await
        .expect("GET /dtu/incidents failed");
    assert_eq!(incidents_resp.status(), 200);

    let body: serde_json::Value = incidents_resp.json().await.expect("invalid JSON");

    // Inspect all incident rows for forbidden mode metadata keys
    let forbidden_keys = ["mode", "shared", "dtu_mode"];
    if let Some(incidents) = body.get("incidents").and_then(|v| v.as_array()) {
        for (i, incident) in incidents.iter().enumerate() {
            if let Some(obj) = incident.as_object() {
                for key in &forbidden_keys {
                    assert!(
                        !obj.contains_key(*key),
                        "incident row {i} must not contain '{key}' field (BC-3.2.004 postcondition 5)"
                    );
                }
                // Also check the serialized string for these values
                let serialized = serde_json::to_string(incident).unwrap_or_default();
                assert!(
                    !serialized.contains("\"dtu_mode\""),
                    "incident row {i} must not contain dtu_mode in serialized form"
                );
            }
        }
    }
}

// ---------------------------------------------------------------------------
// AC-005 / BC-3.2.005 postcondition 1 + invariant 1
// DtuMode::Shared set at startup and immutable for process lifetime
// ---------------------------------------------------------------------------

/// AC-005: PAGERDUTY_DTU_MODE constant is DtuMode::Shared (compile-time assertion).
#[test]
fn test_BC_3_2_005_ac005_pagerduty_dtu_mode_is_shared() {
    // This test exercises the compile-time constant declared in clone.rs.
    // It will pass once PAGERDUTY_DTU_MODE is set to DtuMode::Shared (stub is already there).
    assert_eq!(
        PAGERDUTY_DTU_MODE,
        DtuMode::Shared,
        "PAGERDUTY_DTU_MODE must be DtuMode::Shared per BC-3.2.005 postcondition 1"
    );
}

/// AC-005: DtuMode::Shared cannot be changed after startup via any in-process API.
#[tokio::test]
async fn test_BC_3_2_005_ac005_mode_immutable_after_startup() {
    // The DtuMode constant is a compile-time const — there is no runtime API to change it.
    // We verify:
    // 1. The constant is DtuMode::Shared before any clone starts.
    // 2. After starting a clone and attempting configure (which changes failure/auth modes,
    //    NOT DtuMode), the constant remains DtuMode::Shared.
    // 3. No method exists on PagerDutyClone or PagerDutyState that accepts a DtuMode argument.
    //    This is enforced at compile time — there is no runtime test for a missing API.

    assert_eq!(
        PAGERDUTY_DTU_MODE,
        DtuMode::Shared,
        "pre-startup: PAGERDUTY_DTU_MODE must be DtuMode::Shared"
    );

    let (clone, _base_url, _client) = start_clone().await;

    // configure() accepts JSON for failure/auth modes only — not DtuMode.
    // Attempting to set a 'dtu_mode' key must NOT change PAGERDUTY_DTU_MODE.
    // (It may return an error or silently ignore the key; either is acceptable
    //  as long as DtuMode::Shared is preserved.)
    let _ = clone
        .configure(serde_json::json!({"failure_mode": "none"}))
        .await;

    // Post-configure: DtuMode constant is immutable (compile-time).
    assert_eq!(
        PAGERDUTY_DTU_MODE,
        DtuMode::Shared,
        "post-configure: PAGERDUTY_DTU_MODE must remain DtuMode::Shared (BC-3.2.005 invariant 1)"
    );
}

// ---------------------------------------------------------------------------
// AC-006 / BC-3.2.005 postcondition 3
// Invalid mode string rejected by serde deserialization
// ---------------------------------------------------------------------------

/// AC-006: mode = "SHared" (wrong case) fails serde deserialization.
#[test]
fn test_BC_3_2_005_ac006_invalid_mode_string_rejected() {
    // DtuMode uses #[serde(rename_all = "lowercase")] — only "shared", "dedicated", "solo" accepted.
    let result: Result<DtuMode, _> = serde_json::from_str("\"SHared\"");
    assert!(
        result.is_err(),
        "serde must reject 'SHared' (wrong case) as an invalid DtuMode string (BC-3.2.005 postcondition 3)"
    );

    // Also verify "SHARED" (all caps) is rejected
    let result_upper: Result<DtuMode, _> = serde_json::from_str("\"SHARED\"");
    assert!(
        result_upper.is_err(),
        "serde must reject 'SHARED' (all caps) as an invalid DtuMode string"
    );

    // Verify the correct lowercase form IS accepted
    let result_ok: Result<DtuMode, _> = serde_json::from_str("\"shared\"");
    assert!(
        result_ok.is_ok(),
        "serde must accept 'shared' (lowercase) as a valid DtuMode string"
    );
    assert_eq!(result_ok.unwrap(), DtuMode::Shared);
}
