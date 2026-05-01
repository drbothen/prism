//! S-3.4.05 harness migration tests — prism-dtu-pagerduty, shared-mode.
//!
//! # Behavioral contracts
//!
//! - BC-3.2.004: Shared-Mode DTU Tags OrgId in Payload Body Not in Routing Headers
//! - BC-3.3.001: DTU Mode Policy (startup EC-003: MSSP Coordination types permit client override)
//! - BC-3.5.001: Harness Logical Isolation Invariants
//!
//! # Test catalog (migrated from existing tests + new AC tests)
//!
//! ## Migrated from tests/fidelity.rs (17 tests)
//! - test_BC_3_5_001_pd_full_lifecycle_trigger_ack_resolve
//! - test_BC_3_5_001_pd_ac4_ack_on_resolved_returns_400
//! - test_BC_3_5_001_pd_ac5_trigger_idempotent_on_active_incident
//! - test_BC_3_5_001_pd_ac6_invalid_severity_returns_400
//! - test_BC_3_5_001_pd_ec4_uppercase_severity_returns_400
//! - test_BC_3_5_001_pd_ac7_missing_routing_key_returns_400
//! - test_BC_3_5_001_pd_ac8_auth_reject_mode_returns_403
//! - test_BC_3_5_001_pd_ec1_auto_generated_dedup_key
//! - test_BC_3_5_001_pd_ec2_resolve_unknown_dedup_key_returns_400
//! - test_BC_3_5_001_pd_ec3_retrigger_after_resolve_creates_fresh_incident
//! - test_BC_3_5_001_pd_ac9_rate_limit_returns_429_with_retry_after
//! - test_BC_3_5_001_pd_invalid_event_action_returns_400
//! - test_BC_3_5_001_pd_acknowledge_unknown_dedup_key_returns_400
//! - test_BC_3_5_001_pd_ec5_auth_reject_cleared_by_reset
//! - test_BC_3_5_001_pd_configure_without_admin_token_returns_401
//! - test_BC_3_5_001_pd_health_returns_200
//! - test_BC_3_5_001_pd_reset_clears_incidents
//!
//! ## Migrated from tests/org_tagging.rs (8 tests)
//! - test_BC_3_2_004_pd_ac001_org_id_in_incident_record
//! - test_BC_3_2_004_pd_ac002_dedup_key_not_org_scoped
//! - test_BC_3_2_004_pd_ac002_org_id_absent_from_routing
//! - test_BC_3_2_004_pd_ac003_concurrent_incidents_distinguished
//! - test_BC_3_2_004_pd_ac004_mode_metadata_absent_from_query_results
//! - test_BC_3_2_005_pd_ac005_pagerduty_dtu_mode_is_shared
//! - test_BC_3_2_005_pd_ac005_mode_immutable_after_startup
//! - test_BC_3_2_005_pd_ac006_invalid_mode_string_rejected
//!
//! ## New harness-specific tests (S-3.4.05 AC-005, AC-007, EC-001, EC-002, EC-003)
//! - ac_shared_mode_org_id_tagging
//! - ac_multi_org_logical_isolation_shared_mode
//! - ac_client_mode_override_does_not_produce_startup_error
//!
//! # Red Gate rationale
//!
//! All BC-3.2.004 / BC-3.2.005 org-tagging assertions fail because the PagerDuty
//! enqueue handler stores raw incident state without org-id attribution in
//! `IncidentRecord.org_id` (the field exists but is never set from `X-Prism-Org-Id`).
//!
//! All BC-3.5.001 harness migration assertions fail because `DtuType::PagerDuty` is
//! not yet dispatched by the harness clone-server — `endpoint_for` returns `None`.
//!
//! # Naming convention
//!
//! `test_BC_S_SS_NNN_xxx()` for BC-traced tests.
//! `ac_xxx()` for story-level AC tests without a direct BC number.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    non_snake_case,
    unused_imports
)]
#![cfg(feature = "dtu")]

use prism_dtu_harness::{DtuType, HarnessBuilder, IsolationMode};
use serde_json::Value;

// ---------------------------------------------------------------------------
// Test constants
// ---------------------------------------------------------------------------

const TENANT: &str = "test-tenant";

/// Canonical OrgId test vectors (AI-opaque UUIDs).
const ORG_UUID_A: &str = "00000000-0000-7000-8000-000000000001";
const ORG_UUID_B: &str = "00000000-0000-7000-8000-000000000002";

/// Standard routing key used across fidelity tests.
const ROUTING_KEY: &str = "test-rk-001";

// ---------------------------------------------------------------------------
// Test helpers
// ---------------------------------------------------------------------------

/// Build a single-tenant PagerDuty harness with `IsolationMode::Logical`.
///
/// RED GATE: Fails because `DtuType::PagerDuty` is not yet dispatched by the
/// harness clone-server — `endpoint_for` returns `None`, causing `expect` to panic.
async fn build_pd_harness() -> (
    prism_dtu_harness::Harness,
    std::net::SocketAddr,
    reqwest::Client,
) {
    let harness = HarnessBuilder::new()
        .isolation(IsolationMode::Logical)
        .with_customer_overrides(TENANT, |spec| {
            spec.dtu_types = vec![DtuType::PagerDuty];
        })
        .build()
        .await
        .expect("PagerDuty harness build must succeed (BC-3.5.001 precondition 2)");

    let addr = harness
        .endpoint_for(TENANT, DtuType::PagerDuty)
        .expect("PagerDuty endpoint must be present after build");

    let client = reqwest::Client::new();
    (harness, addr, client)
}

/// POST an event to /v2/enqueue and return the response.
async fn enqueue(
    client: &reqwest::Client,
    addr: std::net::SocketAddr,
    body: &Value,
) -> reqwest::Response {
    enqueue_for_org(client, addr, body, None).await
}

/// POST an event to /v2/enqueue with an optional `X-Prism-Org-Id` header.
async fn enqueue_for_org(
    client: &reqwest::Client,
    addr: std::net::SocketAddr,
    body: &Value,
    org_id: Option<&str>,
) -> reqwest::Response {
    let mut req = client.post(format!("http://{addr}/v2/enqueue")).json(body);
    if let Some(id) = org_id {
        req = req.header("X-Prism-Org-Id", id);
    }
    req.send()
        .await
        .expect("POST /v2/enqueue must not fail at network level")
}

/// GET /dtu/incidents and return the incidents array.
async fn get_incidents(client: &reqwest::Client, addr: std::net::SocketAddr) -> Vec<Value> {
    let resp = client
        .get(format!("http://{addr}/dtu/incidents"))
        .send()
        .await
        .expect("GET /dtu/incidents must not fail at network level");
    assert_eq!(resp.status().as_u16(), 200);
    let body: Value = resp.json().await.expect("response body is JSON");
    body["incidents"]
        .as_array()
        .expect("incidents must be an array")
        .clone()
}

/// POST a trigger event and return the response.
async fn trigger(
    client: &reqwest::Client,
    addr: std::net::SocketAddr,
    dedup_key: &str,
    severity: &str,
    org_id: Option<&str>,
) -> reqwest::Response {
    let body = serde_json::json!({
        "routing_key": ROUTING_KEY,
        "event_action": "trigger",
        "dedup_key": dedup_key,
        "payload": {
            "summary": format!("Test incident {dedup_key}"),
            "severity": severity,
            "source": "prism-test"
        }
    });
    enqueue_for_org(client, addr, &body, org_id).await
}

// ---------------------------------------------------------------------------
// Migrated: fidelity.rs → test_BC_3_5_001_pd_*
// ---------------------------------------------------------------------------

/// Migrated from fidelity.rs: `test_full_lifecycle_trigger_ack_resolve`.
///
/// AC-1 + AC-2 + AC-3: Full trigger → acknowledge → resolve lifecycle via harness.
///
/// RED GATE: Fails at `build_pd_harness` — `DtuType::PagerDuty` not dispatched.
///
/// Traces to: BC-3.5.001 postcondition 1; S-3.4.05 AC-002.
#[tokio::test]
async fn test_BC_3_5_001_pd_full_lifecycle_trigger_ack_resolve() {
    let (_harness, addr, client) = build_pd_harness().await;

    let dedup_key = "harness-lifecycle-001";

    // --- trigger (AC-1) ---
    let trigger_resp = trigger(&client, addr, dedup_key, "critical", None).await;
    assert_eq!(
        trigger_resp.status().as_u16(),
        202,
        "AC-1: trigger must return HTTP 202"
    );
    let trigger_body: Value = trigger_resp.json().await.expect("trigger body is JSON");
    assert_eq!(
        trigger_body["status"].as_str().unwrap_or(""),
        "success",
        "AC-1: trigger status must be 'success'"
    );
    assert_eq!(
        trigger_body["dedup_key"].as_str().unwrap_or(""),
        dedup_key,
        "AC-1: trigger response dedup_key must match input"
    );

    // Verify registry shows Triggered.
    let incidents = get_incidents(&client, addr).await;
    let incident = incidents
        .iter()
        .find(|i| i["dedup_key"].as_str() == Some(dedup_key))
        .expect("incident must be in registry after trigger");
    assert_eq!(
        incident["status"].as_str().unwrap_or(""),
        "triggered",
        "AC-1: incident status must be 'triggered'"
    );

    // --- acknowledge (AC-2) ---
    let ack_resp = enqueue(
        &client,
        addr,
        &serde_json::json!({
            "routing_key": ROUTING_KEY,
            "event_action": "acknowledge",
            "dedup_key": dedup_key
        }),
    )
    .await;
    assert_eq!(
        ack_resp.status().as_u16(),
        200,
        "AC-2: acknowledge must return HTTP 200"
    );

    let incidents2 = get_incidents(&client, addr).await;
    let incident2 = incidents2
        .iter()
        .find(|i| i["dedup_key"].as_str() == Some(dedup_key))
        .expect("incident must still be in registry");
    assert_eq!(
        incident2["status"].as_str().unwrap_or(""),
        "acknowledged",
        "AC-2: incident status must be 'acknowledged'"
    );

    // --- resolve (AC-3) ---
    let resolve_resp = enqueue(
        &client,
        addr,
        &serde_json::json!({
            "routing_key": ROUTING_KEY,
            "event_action": "resolve",
            "dedup_key": dedup_key
        }),
    )
    .await;
    assert_eq!(
        resolve_resp.status().as_u16(),
        200,
        "AC-3: resolve must return HTTP 200"
    );

    let incidents3 = get_incidents(&client, addr).await;
    let incident3 = incidents3
        .iter()
        .find(|i| i["dedup_key"].as_str() == Some(dedup_key))
        .expect("incident must still be in registry after resolve");
    assert_eq!(
        incident3["status"].as_str().unwrap_or(""),
        "resolved",
        "AC-3: incident status must be 'resolved'"
    );
}

/// Migrated from fidelity.rs: `test_ac4_ack_on_resolved_returns_400`.
///
/// RED GATE: Fails at `build_pd_harness`.
///
/// Traces to: BC-3.5.001 postcondition 1; S-3.4.05 AC-002.
#[tokio::test]
async fn test_BC_3_5_001_pd_ac4_ack_on_resolved_returns_400() {
    let (_harness, addr, client) = build_pd_harness().await;
    let dedup_key = "harness-ac4-resolved";

    trigger(&client, addr, dedup_key, "info", None).await;
    enqueue(
        &client,
        addr,
        &serde_json::json!({
            "routing_key": ROUTING_KEY,
            "event_action": "resolve",
            "dedup_key": dedup_key
        }),
    )
    .await;

    let ack_resp = enqueue(
        &client,
        addr,
        &serde_json::json!({
            "routing_key": ROUTING_KEY,
            "event_action": "acknowledge",
            "dedup_key": dedup_key
        }),
    )
    .await;

    assert_eq!(
        ack_resp.status().as_u16(),
        400,
        "AC-4: acknowledge on resolved must return 400"
    );
    let body: Value = ack_resp.json().await.expect("body JSON");
    assert_eq!(
        body["status"].as_str().unwrap_or(""),
        "cannot acknowledge a resolved incident",
        "AC-4: error status must be 'cannot acknowledge a resolved incident'"
    );
}

/// Migrated from fidelity.rs: `test_ac5_trigger_idempotent_on_active_incident`.
///
/// RED GATE: Fails at `build_pd_harness`.
///
/// Traces to: BC-3.5.001 postcondition 1; S-3.4.05 AC-002.
#[tokio::test]
async fn test_BC_3_5_001_pd_ac5_trigger_idempotent_on_active_incident() {
    let (_harness, addr, client) = build_pd_harness().await;
    let dedup_key = "harness-ac5-idempotent";

    trigger(&client, addr, dedup_key, "error", None).await;
    let second_resp = trigger(&client, addr, dedup_key, "error", None).await;

    assert_eq!(
        second_resp.status().as_u16(),
        202,
        "AC-5: re-trigger must return 202"
    );

    let incidents = get_incidents(&client, addr).await;
    let matching: Vec<_> = incidents
        .iter()
        .filter(|i| i["dedup_key"].as_str() == Some(dedup_key))
        .collect();
    assert_eq!(
        matching.len(),
        1,
        "AC-5: only one incident must exist for dedup_key after idempotent re-trigger"
    );
}

/// Migrated from fidelity.rs: `test_ac6_invalid_severity_returns_400`.
///
/// RED GATE: Fails at `build_pd_harness`.
///
/// Traces to: BC-3.5.001 postcondition 1; S-3.4.05 AC-002.
#[tokio::test]
async fn test_BC_3_5_001_pd_ac6_invalid_severity_returns_400() {
    let (_harness, addr, client) = build_pd_harness().await;

    let resp = enqueue(
        &client,
        addr,
        &serde_json::json!({
            "routing_key": ROUTING_KEY,
            "event_action": "trigger",
            "payload": {"summary": "test", "severity": "fatal", "source": "svc"}
        }),
    )
    .await;

    assert_eq!(
        resp.status().as_u16(),
        400,
        "AC-6: invalid severity 'fatal' must return 400"
    );
    let body: Value = resp.json().await.expect("body JSON");
    assert_eq!(
        body["status"].as_str().unwrap_or(""),
        "invalid severity",
        "AC-6: error must be 'invalid severity'"
    );
}

/// Migrated from fidelity.rs: `test_ec4_uppercase_severity_returns_400`.
///
/// RED GATE: Fails at `build_pd_harness`.
///
/// Traces to: BC-3.5.001 postcondition 1; S-3.4.05 AC-002.
#[tokio::test]
async fn test_BC_3_5_001_pd_ec4_uppercase_severity_returns_400() {
    let (_harness, addr, client) = build_pd_harness().await;

    let resp = enqueue(
        &client,
        addr,
        &serde_json::json!({
            "routing_key": ROUTING_KEY,
            "event_action": "trigger",
            "payload": {"summary": "test", "severity": "CRITICAL", "source": "svc"}
        }),
    )
    .await;

    assert_eq!(
        resp.status().as_u16(),
        400,
        "EC-004: uppercase 'CRITICAL' must return HTTP 400 (case-sensitive)"
    );
}

/// Migrated from fidelity.rs: `test_ac7_missing_routing_key_returns_400`.
///
/// RED GATE: Fails at `build_pd_harness`.
///
/// Traces to: BC-3.5.001 postcondition 1; S-3.4.05 AC-002.
#[tokio::test]
async fn test_BC_3_5_001_pd_ac7_missing_routing_key_returns_400() {
    let (_harness, addr, client) = build_pd_harness().await;

    let resp = enqueue(
        &client,
        addr,
        &serde_json::json!({
            "event_action": "trigger",
            "payload": {"summary": "test", "severity": "info", "source": "svc"}
        }),
    )
    .await;

    assert_eq!(
        resp.status().as_u16(),
        400,
        "AC-7: missing routing_key must return HTTP 400"
    );
    let body: Value = resp.json().await.expect("body JSON");
    assert_eq!(
        body["status"].as_str().unwrap_or(""),
        "missing routing_key",
        "AC-7: error must be 'missing routing_key'"
    );
}

/// Migrated from fidelity.rs: `test_ac8_auth_reject_mode_returns_403`.
///
/// RED GATE: Fails at `build_pd_harness`.
///
/// Traces to: BC-3.5.001 postcondition 1; S-3.4.05 AC-002.
#[tokio::test]
async fn test_BC_3_5_001_pd_ac8_auth_reject_mode_returns_403() {
    let (harness, addr, client) = build_pd_harness().await;

    harness
        .inject_failure(
            TENANT,
            DtuType::PagerDuty,
            prism_dtu_common::FailureMode::AuthReject,
        )
        .await
        .expect("inject_failure must succeed");

    let resp = enqueue(
        &client,
        addr,
        &serde_json::json!({
            "routing_key": ROUTING_KEY,
            "event_action": "trigger",
            "payload": {"summary": "test", "severity": "info", "source": "svc"}
        }),
    )
    .await;

    assert_eq!(
        resp.status().as_u16(),
        403,
        "AC-8: auth_reject mode must return HTTP 403"
    );
    let body: Value = resp.json().await.expect("body JSON");
    assert_eq!(
        body["status"].as_str().unwrap_or(""),
        "invalid key",
        "AC-8: 403 status must be 'invalid key'"
    );
}

/// Migrated from fidelity.rs: `test_ec1_auto_generated_dedup_key`.
///
/// RED GATE: Fails at `build_pd_harness`.
///
/// Traces to: BC-3.5.001 postcondition 1; S-3.4.05 AC-002.
#[tokio::test]
async fn test_BC_3_5_001_pd_ec1_auto_generated_dedup_key() {
    let (_harness, addr, client) = build_pd_harness().await;

    let resp = enqueue(
        &client,
        addr,
        &serde_json::json!({
            "routing_key": ROUTING_KEY,
            "event_action": "trigger",
            "payload": {"summary": "auto key test", "severity": "warning", "source": "svc"}
        }),
    )
    .await;

    assert_eq!(
        resp.status().as_u16(),
        202,
        "EC-001: trigger without dedup_key must return 202"
    );
    let body: Value = resp.json().await.expect("body JSON");
    let returned_key = body["dedup_key"].as_str().unwrap_or("");
    assert!(
        !returned_key.is_empty(),
        "EC-001: response must include an auto-generated dedup_key"
    );

    let incidents = get_incidents(&client, addr).await;
    let matching: Vec<_> = incidents
        .iter()
        .filter(|i| i["dedup_key"].as_str() == Some(returned_key))
        .collect();
    assert_eq!(
        matching.len(),
        1,
        "EC-001: incident must be registered with auto-generated dedup_key"
    );
}

/// Migrated from fidelity.rs: `test_ec2_resolve_unknown_dedup_key_returns_400`.
///
/// RED GATE: Fails at `build_pd_harness`.
///
/// Traces to: BC-3.5.001 postcondition 1; S-3.4.05 AC-002.
#[tokio::test]
async fn test_BC_3_5_001_pd_ec2_resolve_unknown_dedup_key_returns_400() {
    let (_harness, addr, client) = build_pd_harness().await;

    let resp = enqueue(
        &client,
        addr,
        &serde_json::json!({
            "routing_key": ROUTING_KEY,
            "event_action": "resolve",
            "dedup_key": "no-such-incident-harness-xyz"
        }),
    )
    .await;

    assert_eq!(
        resp.status().as_u16(),
        400,
        "EC-002: resolve on unknown dedup_key must return 400"
    );
    let body: Value = resp.json().await.expect("body JSON");
    assert_eq!(
        body["status"].as_str().unwrap_or(""),
        "invalid dedup_key",
        "EC-002: error must be 'invalid dedup_key'"
    );
}

/// Migrated from fidelity.rs: `test_ec3_retrigger_after_resolve_creates_fresh_incident`.
///
/// RED GATE: Fails at `build_pd_harness`.
///
/// Traces to: BC-3.5.001 postcondition 1; S-3.4.05 AC-002.
#[tokio::test]
async fn test_BC_3_5_001_pd_ec3_retrigger_after_resolve_creates_fresh_incident() {
    let (_harness, addr, client) = build_pd_harness().await;
    let dedup_key = "harness-ec3-retrigger";

    trigger(&client, addr, dedup_key, "error", None).await;
    enqueue(
        &client,
        addr,
        &serde_json::json!({
            "routing_key": ROUTING_KEY,
            "event_action": "resolve",
            "dedup_key": dedup_key
        }),
    )
    .await;

    let retrigger_resp = trigger(&client, addr, dedup_key, "critical", None).await;
    assert_eq!(
        retrigger_resp.status().as_u16(),
        202,
        "EC-003: re-trigger after resolve must return 202"
    );

    let incidents = get_incidents(&client, addr).await;
    let incident = incidents
        .iter()
        .find(|i| i["dedup_key"].as_str() == Some(dedup_key))
        .expect("incident must be in registry");
    assert_eq!(
        incident["status"].as_str().unwrap_or(""),
        "triggered",
        "EC-003: fresh incident after re-trigger must have status 'triggered'"
    );
}

/// Migrated from fidelity.rs: `test_ac9_rate_limit_returns_429_with_retry_after`.
///
/// RED GATE: Fails at `build_pd_harness`.
///
/// Traces to: BC-3.5.001 postcondition 1; S-3.4.05 AC-002.
#[tokio::test]
async fn test_BC_3_5_001_pd_ac9_rate_limit_returns_429_with_retry_after() {
    let (harness, addr, client) = build_pd_harness().await;

    harness
        .inject_failure(
            TENANT,
            DtuType::PagerDuty,
            prism_dtu_common::FailureMode::RateLimit {
                after_n_requests: 0,
                retry_after_secs: 60,
            },
        )
        .await
        .expect("inject_failure must succeed");

    let resp = enqueue(
        &client,
        addr,
        &serde_json::json!({
            "routing_key": ROUTING_KEY,
            "event_action": "trigger",
            "payload": {"summary": "test", "severity": "info", "source": "svc"}
        }),
    )
    .await;

    assert_eq!(
        resp.status().as_u16(),
        429,
        "AC-9: rate_limit failure mode must return HTTP 429"
    );
    let retry_after = resp
        .headers()
        .get("retry-after")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    assert_eq!(
        retry_after, "60",
        "AC-9: Retry-After header must equal '60'"
    );
}

/// Migrated from fidelity.rs: `test_invalid_event_action_returns_400`.
///
/// RED GATE: Fails at `build_pd_harness`.
///
/// Traces to: BC-3.5.001 postcondition 1; S-3.4.05 AC-002.
#[tokio::test]
async fn test_BC_3_5_001_pd_invalid_event_action_returns_400() {
    let (_harness, addr, client) = build_pd_harness().await;

    let resp = enqueue(
        &client,
        addr,
        &serde_json::json!({
            "routing_key": ROUTING_KEY,
            "event_action": "create",
            "payload": {"summary": "test", "severity": "info", "source": "svc"}
        }),
    )
    .await;

    assert_eq!(
        resp.status().as_u16(),
        400,
        "invalid event_action must return HTTP 400"
    );
    let body: Value = resp.json().await.expect("body JSON");
    assert_eq!(
        body["status"].as_str().unwrap_or(""),
        "invalid event_action",
        "error status must be 'invalid event_action'"
    );
}

/// Migrated from fidelity.rs: `test_acknowledge_unknown_dedup_key_returns_400`.
///
/// RED GATE: Fails at `build_pd_harness`.
///
/// Traces to: BC-3.5.001 postcondition 1; S-3.4.05 AC-002.
#[tokio::test]
async fn test_BC_3_5_001_pd_acknowledge_unknown_dedup_key_returns_400() {
    let (_harness, addr, client) = build_pd_harness().await;

    let resp = enqueue(
        &client,
        addr,
        &serde_json::json!({
            "routing_key": ROUTING_KEY,
            "event_action": "acknowledge",
            "dedup_key": "no-such-incident-ack-harness-xyz"
        }),
    )
    .await;

    assert_eq!(
        resp.status().as_u16(),
        400,
        "acknowledge on unknown dedup_key must return 400"
    );
    let body: Value = resp.json().await.expect("body JSON");
    assert_eq!(
        body["status"].as_str().unwrap_or(""),
        "invalid dedup_key",
        "error status must be 'invalid dedup_key'"
    );
}

/// Migrated from fidelity.rs: `test_ec5_auth_reject_cleared_by_reset`.
///
/// RED GATE: Fails at `build_pd_harness`.
///
/// Traces to: BC-3.5.001 postcondition 1; S-3.4.05 AC-002.
#[tokio::test]
async fn test_BC_3_5_001_pd_ec5_auth_reject_cleared_by_reset() {
    let (harness, addr, client) = build_pd_harness().await;

    harness
        .inject_failure(
            TENANT,
            DtuType::PagerDuty,
            prism_dtu_common::FailureMode::AuthReject,
        )
        .await
        .expect("inject_failure must succeed");

    let before_reset = enqueue(
        &client,
        addr,
        &serde_json::json!({
            "routing_key": ROUTING_KEY,
            "event_action": "trigger",
            "payload": {"summary": "test", "severity": "info", "source": "svc"}
        }),
    )
    .await;
    assert_eq!(
        before_reset.status().as_u16(),
        403,
        "EC-005: auth_reject must be active before reset"
    );

    // Clear the failure mode.
    harness
        .clear_failure(TENANT, DtuType::PagerDuty)
        .await
        .expect("clear_failure must succeed");

    let after_reset = enqueue(
        &client,
        addr,
        &serde_json::json!({
            "routing_key": ROUTING_KEY,
            "event_action": "trigger",
            "payload": {"summary": "test", "severity": "info", "source": "svc"}
        }),
    )
    .await;
    assert_eq!(
        after_reset.status().as_u16(),
        202,
        "EC-005: after clear_failure, trigger must succeed with 202 (auth_reject cleared)"
    );
}

/// Migrated from fidelity.rs: `test_configure_without_admin_token_returns_401`.
///
/// RED GATE: Fails at `build_pd_harness`.
///
/// Traces to: BC-3.5.001 postcondition 1; S-3.4.05 AC-002.
#[tokio::test]
async fn test_BC_3_5_001_pd_configure_without_admin_token_returns_401() {
    let (_harness, addr, client) = build_pd_harness().await;

    let resp = client
        .post(format!("http://{addr}/dtu/configure"))
        .json(&serde_json::json!({"auth_mode": "reject"}))
        .send()
        .await
        .expect("POST /dtu/configure must not fail at network level");

    assert_eq!(
        resp.status().as_u16(),
        401,
        "/dtu/configure without X-Admin-Token must return 401"
    );
}

/// Migrated from fidelity.rs: `test_dtu_health_returns_200`.
///
/// RED GATE: Fails at `build_pd_harness`.
///
/// Traces to: BC-3.5.001 postcondition 1; S-3.4.05 AC-002.
#[tokio::test]
async fn test_BC_3_5_001_pd_health_returns_200() {
    let (_harness, addr, client) = build_pd_harness().await;

    let resp = client
        .get(format!("http://{addr}/dtu/health"))
        .send()
        .await
        .expect("GET /dtu/health must not fail at network level");

    assert_eq!(
        resp.status().as_u16(),
        200,
        "GET /dtu/health must return 200"
    );
    let body: Value = resp.json().await.expect("body JSON");
    assert_eq!(
        body["status"].as_str().unwrap_or(""),
        "ok",
        "/dtu/health body must have status 'ok'"
    );
}

/// Migrated from fidelity.rs: `test_dtu_reset_clears_incidents`.
///
/// RED GATE: Fails at `build_pd_harness`.
///
/// Traces to: BC-3.5.001 postcondition 1; S-3.4.05 AC-002.
#[tokio::test]
async fn test_BC_3_5_001_pd_reset_clears_incidents() {
    let (_harness, addr, client) = build_pd_harness().await;

    trigger(&client, addr, "harness-reset-test-001", "info", None).await;

    let reset_resp = client
        .post(format!("http://{addr}/dtu/reset"))
        .send()
        .await
        .expect("POST /dtu/reset must not fail at network level");
    assert_eq!(reset_resp.status().as_u16(), 200, "reset must return 200");

    let incidents = get_incidents(&client, addr).await;
    assert_eq!(
        incidents.len(),
        0,
        "registry must be empty after /dtu/reset"
    );
}

// ---------------------------------------------------------------------------
// Migrated: org_tagging.rs → test_BC_3_2_004_pd_* / test_BC_3_2_005_pd_*
// ---------------------------------------------------------------------------

/// Migrated from org_tagging.rs: `test_BC_3_2_004_ac001_org_id_in_incident_record`.
///
/// OrgId UUID appears in IncidentRecord.org_id after trigger with X-Prism-Org-Id.
///
/// RED GATE PRIMARY: `build_pd_harness` panics.
/// RED GATE SECONDARY: `IncidentRecord.org_id` is empty — the handler does not
/// extract `X-Prism-Org-Id` and pass it to `capture_incident_tagged`.
///
/// Traces to: BC-3.2.004 postcondition 1; VP-087; S-3.4.05 AC-005.
#[tokio::test]
async fn test_BC_3_2_004_pd_ac001_org_id_in_incident_record() {
    let (_harness, addr, client) = build_pd_harness().await;

    let dedup_key = "harness-bc3-2-004-ac001";
    let resp = trigger(&client, addr, dedup_key, "critical", Some(ORG_UUID_A)).await;
    assert_eq!(resp.status().as_u16(), 202, "trigger must return 202");

    // The incident registry must carry org_id == ORG_UUID_A.
    let incidents = get_incidents(&client, addr).await;
    let record = incidents
        .iter()
        .find(|i| i["dedup_key"].as_str() == Some(dedup_key))
        .expect("incident must be in registry");

    assert_eq!(
        record["org_id"].as_str().unwrap_or(""),
        ORG_UUID_A,
        "BC-3.2.004 postcondition 1: IncidentRecord.org_id must equal the OrgId UUID string \
         (via GET /dtu/incidents response)"
    );
}

/// Migrated from org_tagging.rs: `test_BC_3_2_004_ac002_dedup_key_not_org_scoped`.
///
/// dedup_key does not contain org_id UUID (ADR-008 §1.2).
///
/// RED GATE: Fails at `build_pd_harness`.
///
/// Traces to: BC-3.2.004 postcondition 2; VP-088; S-3.4.05 AC-005.
#[tokio::test]
async fn test_BC_3_2_004_pd_ac002_dedup_key_not_org_scoped() {
    let (_harness, addr, client) = build_pd_harness().await;

    // Choose a MSSP dedup_key that does NOT embed the org UUID.
    let dedup_key = "mssp-scoped-key-12345";
    // Sanity: the test dedup_key must not contain the org UUID.
    assert!(
        !dedup_key.contains(ORG_UUID_A),
        "test setup error: MSSP dedup_key must not contain org_id"
    );

    let resp = trigger(&client, addr, dedup_key, "warning", Some(ORG_UUID_A)).await;
    assert_eq!(resp.status().as_u16(), 202);

    let incidents = get_incidents(&client, addr).await;
    let record = incidents
        .iter()
        .find(|i| i["dedup_key"].as_str() == Some(dedup_key))
        .expect("incident must be in registry");

    let org_id_in_record = record["org_id"].as_str().unwrap_or("");
    let dedup_key_in_record = record["dedup_key"].as_str().unwrap_or("");

    // BC-3.2.004 postcondition 2: dedup_key MUST NOT contain org_id.
    assert!(
        !dedup_key_in_record.contains(org_id_in_record),
        "dedup_key '{dedup_key_in_record}' must not contain the org_id UUID \
         '{org_id_in_record}' (ADR-008 §1.2)"
    );
}

/// Migrated from org_tagging.rs: `test_BC_3_2_004_ac002_org_id_absent_from_routing`.
///
/// org_id does not appear in response URL or headers.
///
/// RED GATE: Fails at `build_pd_harness`.
///
/// Traces to: BC-3.2.004 postcondition 2; VP-088; S-3.4.05 AC-005.
#[tokio::test]
async fn test_BC_3_2_004_pd_ac002_org_id_absent_from_routing() {
    let (_harness, addr, client) = build_pd_harness().await;

    let resp = enqueue_for_org(
        &client,
        addr,
        &serde_json::json!({
            "routing_key": ROUTING_KEY,
            "event_action": "trigger",
            "payload": {"summary": "Org routing isolation test", "severity": "info", "source": "svc"}
        }),
        Some(ORG_UUID_A),
    )
    .await;
    assert_eq!(resp.status().as_u16(), 202, "expected 202 Accepted");

    let resp_url = resp.url().to_string();
    assert!(
        !resp_url.contains(ORG_UUID_A),
        "org_id UUID must not appear in response URL (BC-3.2.004 postcondition 2)"
    );

    for (name, value) in resp.headers() {
        let val_str = value.to_str().unwrap_or("");
        assert!(
            !val_str.contains(ORG_UUID_A),
            "org_id UUID must not appear in response header '{name}': '{val_str}' \
             (BC-3.2.004 postcondition 2)"
        );
    }
}

/// Migrated from org_tagging.rs: `test_BC_3_2_004_ac003_concurrent_incidents_distinguished`.
///
/// Concurrent incidents from org_A and org_B each carry their sender's OrgId.
///
/// RED GATE PRIMARY: `build_pd_harness` panics.
/// RED GATE SECONDARY: `IncidentRecord.org_id` is empty.
///
/// Traces to: BC-3.2.004 postcondition 4; VP-089; S-3.4.05 AC-005.
#[tokio::test]
async fn test_BC_3_2_004_pd_ac003_concurrent_incidents_distinguished() {
    let (_harness, addr, client) = build_pd_harness().await;

    let dedup_a = "harness-concurrent-org-a";
    let dedup_b = "harness-concurrent-org-b";

    let addr_a = addr;
    let addr_b = addr;
    let d_a = dedup_a.to_owned();
    let d_b = dedup_b.to_owned();

    let task_a = tokio::spawn(async move {
        let c = reqwest::Client::new();
        c.post(format!("http://{addr_a}/v2/enqueue"))
            .header("X-Prism-Org-Id", ORG_UUID_A)
            .json(&serde_json::json!({
                "routing_key": ROUTING_KEY,
                "event_action": "trigger",
                "dedup_key": d_a,
                "payload": {"summary": "Org A incident", "severity": "critical", "source": "svc"}
            }))
            .send()
            .await
            .expect("task A POST")
    });

    let task_b = tokio::spawn(async move {
        let c = reqwest::Client::new();
        c.post(format!("http://{addr_b}/v2/enqueue"))
            .header("X-Prism-Org-Id", ORG_UUID_B)
            .json(&serde_json::json!({
                "routing_key": ROUTING_KEY,
                "event_action": "trigger",
                "dedup_key": d_b,
                "payload": {"summary": "Org B incident", "severity": "warning", "source": "svc"}
            }))
            .send()
            .await
            .expect("task B POST")
    });

    let (r_a, r_b) = tokio::join!(task_a, task_b);
    assert!(
        r_a.expect("task A").status().is_success(),
        "org A POST must succeed"
    );
    assert!(
        r_b.expect("task B").status().is_success(),
        "org B POST must succeed"
    );

    let incidents = get_incidents(&client, addr).await;
    let rec_a = incidents
        .iter()
        .find(|i| i["dedup_key"].as_str() == Some(dedup_a))
        .expect("org_A incident not found");
    let rec_b = incidents
        .iter()
        .find(|i| i["dedup_key"].as_str() == Some(dedup_b))
        .expect("org_B incident not found");

    assert_eq!(
        rec_a["org_id"].as_str().unwrap_or(""),
        ORG_UUID_A,
        "BC-3.2.004 postcondition 4: org_A incident must carry org_A's OrgId"
    );
    assert_eq!(
        rec_b["org_id"].as_str().unwrap_or(""),
        ORG_UUID_B,
        "BC-3.2.004 postcondition 4: org_B incident must carry org_B's OrgId"
    );
    assert_ne!(
        rec_a["org_id"], rec_b["org_id"],
        "org_A and org_B incidents must have distinct OrgIds"
    );
}

/// Migrated from org_tagging.rs: `test_BC_3_2_004_ac004_mode_metadata_absent_from_query_results`.
///
/// GET /dtu/incidents response rows contain no "mode", "shared", or "dtu_mode" fields.
///
/// RED GATE: Fails at `build_pd_harness`.
///
/// Traces to: BC-3.2.004 postcondition 5; VP-090; S-3.4.05 AC-005.
#[tokio::test]
async fn test_BC_3_2_004_pd_ac004_mode_metadata_absent_from_query_results() {
    let (_harness, addr, client) = build_pd_harness().await;

    trigger(
        &client,
        addr,
        "harness-mode-metadata-test",
        "critical",
        Some(ORG_UUID_A),
    )
    .await;

    let incidents = get_incidents(&client, addr).await;
    let forbidden_keys = ["mode", "shared", "dtu_mode"];
    for (i, incident) in incidents.iter().enumerate() {
        if let Some(obj) = incident.as_object() {
            for key in &forbidden_keys {
                assert!(
                    !obj.contains_key(*key),
                    "incident row {i} must not contain '{key}' field (BC-3.2.004 postcondition 5)"
                );
            }
        }
    }
}

/// Migrated from org_tagging.rs: `test_BC_3_2_005_ac005_pagerduty_dtu_mode_is_shared`.
///
/// PAGERDUTY_DTU_MODE constant is DtuMode::Shared (compile-time assertion).
///
/// This test is synchronous and exercises only the compile-time constant.
/// It PASSES at Red Gate because the constant is already set correctly.
/// It is kept here to ensure it fails when the harness migration regresses.
///
/// Traces to: BC-3.2.005 postcondition 1; VP-122; S-3.4.05 AC-002.
#[test]
fn test_BC_3_2_005_pd_ac005_pagerduty_dtu_mode_is_shared() {
    use prism_dtu_common::DtuMode;
    use prism_dtu_pagerduty::clone::PAGERDUTY_DTU_MODE;

    assert_eq!(
        PAGERDUTY_DTU_MODE,
        DtuMode::Shared,
        "PAGERDUTY_DTU_MODE must be DtuMode::Shared per BC-3.2.005 postcondition 1"
    );
}

/// Migrated from org_tagging.rs: `test_BC_3_2_005_ac005_mode_immutable_after_startup`.
///
/// DtuMode::Shared cannot be changed after startup via any in-process API.
///
/// RED GATE PRIMARY: `build_pd_harness` panics.
///
/// Traces to: BC-3.2.005 postcondition 1 + invariant 1; VP-123; S-3.4.05 AC-002.
#[tokio::test]
async fn test_BC_3_2_005_pd_ac005_mode_immutable_after_startup() {
    use prism_dtu_common::DtuMode;
    use prism_dtu_pagerduty::clone::PAGERDUTY_DTU_MODE;

    assert_eq!(
        PAGERDUTY_DTU_MODE,
        DtuMode::Shared,
        "pre-startup: PAGERDUTY_DTU_MODE must be DtuMode::Shared"
    );

    let (harness, _addr, _client) = build_pd_harness().await;

    // configure() accepts JSON for failure/auth modes only — not DtuMode.
    let _ = harness
        .inject_failure(
            TENANT,
            DtuType::PagerDuty,
            prism_dtu_common::FailureMode::None,
        )
        .await;

    assert_eq!(
        PAGERDUTY_DTU_MODE,
        DtuMode::Shared,
        "post-configure: PAGERDUTY_DTU_MODE must remain DtuMode::Shared (BC-3.2.005 invariant 1)"
    );
}

/// Migrated from org_tagging.rs: `test_BC_3_2_005_ac006_invalid_mode_string_rejected`.
///
/// mode = "SHared" (wrong case) fails serde deserialization.
///
/// This test is synchronous and passes immediately (serde rejects the variant).
/// It is kept here to document the BC-3.2.005 mode-string rejection contract.
///
/// Traces to: BC-3.2.005 postcondition 3; S-3.4.05 AC-002.
#[test]
fn test_BC_3_2_005_pd_ac006_invalid_mode_string_rejected() {
    use prism_dtu_common::DtuMode;

    let result: Result<DtuMode, _> = serde_json::from_str("\"SHared\"");
    assert!(
        result.is_err(),
        "serde must reject 'SHared' (wrong case) as an invalid DtuMode string"
    );

    let result_upper: Result<DtuMode, _> = serde_json::from_str("\"SHARED\"");
    assert!(
        result_upper.is_err(),
        "serde must reject 'SHARED' (all caps) as an invalid DtuMode string"
    );

    let result_ok: Result<DtuMode, _> = serde_json::from_str("\"shared\"");
    assert!(
        result_ok.is_ok(),
        "serde must accept 'shared' (lowercase) as a valid DtuMode string"
    );
    assert_eq!(result_ok.unwrap(), DtuMode::Shared);
}

// ---------------------------------------------------------------------------
// New harness-specific tests (S-3.4.05 ACs)
// ---------------------------------------------------------------------------

/// AC-005 (S-3.4.05): Shared-mode OrgId tagging — PagerDuty `custom_details` field.
///
/// Builds a harness with a single shared PagerDuty clone (IsolationMode::Logical,
/// DtuType::PagerDuty). Dispatches an alert on behalf of org_B. Asserts that the
/// captured payload's `org_id` field in the incident record contains the org_B UUID.
///
/// RED GATE PRIMARY: `build_pd_harness` panics.
/// RED GATE SECONDARY: `IncidentRecord.org_id` is empty.
///
/// Traces to: BC-3.2.004 postconditions 1, 2; BC-3.5.001 postcondition 1;
///            VP-087, VP-088; S-3.4.05 AC-005.
#[tokio::test]
async fn ac_shared_mode_org_id_tagging() {
    let (_harness, addr, client) = build_pd_harness().await;

    let dedup_b = "ac-shared-mode-org-b";
    let resp_b = trigger(&client, addr, dedup_b, "warning", Some(ORG_UUID_B)).await;
    assert_eq!(resp_b.status().as_u16(), 202, "org_B trigger must succeed");

    // Verify org_B UUID is NOT in response URL or headers.
    let url_b = resp_b.url().to_string();
    assert!(
        !url_b.contains(ORG_UUID_B),
        "ORG_UUID_B must not appear in response URL"
    );
    for (name, value) in resp_b.headers() {
        let val_str = value.to_str().unwrap_or("");
        assert!(
            !val_str.contains(ORG_UUID_B),
            "ORG_UUID_B must not appear in response header '{name}'"
        );
    }

    let dedup_a = "ac-shared-mode-org-a";
    let resp_a = trigger(&client, addr, dedup_a, "critical", Some(ORG_UUID_A)).await;
    assert_eq!(resp_a.status().as_u16(), 202, "org_A trigger must succeed");

    // Retrieve captured incidents and verify org_id tags.
    let incidents = get_incidents(&client, addr).await;
    assert_eq!(incidents.len(), 2, "both incidents must be captured");

    let rec_b = incidents
        .iter()
        .find(|i| i["dedup_key"].as_str() == Some(dedup_b))
        .expect("org_B incident must be in registry");
    let rec_a = incidents
        .iter()
        .find(|i| i["dedup_key"].as_str() == Some(dedup_a))
        .expect("org_A incident must be in registry");

    assert_eq!(
        rec_b["org_id"].as_str().unwrap_or(""),
        ORG_UUID_B,
        "AC-005: org_B incident must carry ORG_UUID_B in org_id field"
    );
    assert_eq!(
        rec_a["org_id"].as_str().unwrap_or(""),
        ORG_UUID_A,
        "AC-005: org_A incident must carry ORG_UUID_A in org_id field"
    );
    assert_ne!(
        rec_a["org_id"], rec_b["org_id"],
        "AC-005: the two captured org_ids must be distinct"
    );
}

/// AC-005 / EC-001, EC-002 (S-3.4.05): Multi-org logical isolation in shared PagerDuty mode.
///
/// A single shared PagerDuty listener serves all orgs. Dispatches sequential alerts
/// for org_A and org_B and verifies that the two captured incident records each
/// carry their sender's OrgId, with no cross-contamination.
///
/// RED GATE PRIMARY: `build_pd_harness` panics.
/// RED GATE SECONDARY: `IncidentRecord.org_id` is empty.
///
/// Traces to: BC-3.5.001 postconditions 1, 2; BC-3.2.004 postcondition 4;
///            VP-089, VP-122; S-3.4.05 AC-002, EC-001, EC-002.
#[tokio::test]
async fn ac_multi_org_logical_isolation_shared_mode() {
    let (_harness, addr, client) = build_pd_harness().await;

    let dedup_a = "multi-org-isolation-a";
    let dedup_b = "multi-org-isolation-b";

    trigger(&client, addr, dedup_a, "error", Some(ORG_UUID_A)).await;
    trigger(&client, addr, dedup_b, "critical", Some(ORG_UUID_B)).await;

    let incidents = get_incidents(&client, addr).await;
    assert_eq!(
        incidents.len(),
        2,
        "AC-005: both org incidents must be captured; got {}",
        incidents.len()
    );

    let rec_a = incidents
        .iter()
        .find(|i| i["dedup_key"].as_str() == Some(dedup_a))
        .expect("org_A incident must be present");
    let rec_b = incidents
        .iter()
        .find(|i| i["dedup_key"].as_str() == Some(dedup_b))
        .expect("org_B incident must be present");

    assert_eq!(
        rec_a["org_id"].as_str().unwrap_or(""),
        ORG_UUID_A,
        "EC-001: org_A incident must carry ORG_UUID_A"
    );
    assert_eq!(
        rec_b["org_id"].as_str().unwrap_or(""),
        ORG_UUID_B,
        "EC-002: org_B incident must carry ORG_UUID_B"
    );

    // No cross-contamination: dedup_key and routing fields must not carry the other org's UUID.
    let dedup_key_a = rec_a["dedup_key"].as_str().unwrap_or("");
    assert!(
        !dedup_key_a.contains(ORG_UUID_B),
        "EC-001: org_A dedup_key must not contain ORG_UUID_B"
    );
    let dedup_key_b = rec_b["dedup_key"].as_str().unwrap_or("");
    assert!(
        !dedup_key_b.contains(ORG_UUID_A),
        "EC-002: org_B dedup_key must not contain ORG_UUID_A"
    );
}

/// AC-007 / EC-003 (S-3.4.05): `CustomerSpec` with `mode = "client"` for PagerDuty does NOT
/// produce a startup error (BC-3.3.001-startup EC-003: MSSP Coordination types permit
/// client mode override).
///
/// RED GATE: This test verifies that `HarnessBuilder::build()` returns `Ok` — which
/// requires `DtuType::PagerDuty` to be wired into the clone-server dispatch.
///
/// Traces to: BC-3.5.001 precondition 2 (valid customer registered);
///            BC-3.3.001-startup EC-003; S-3.4.05 AC-007.
#[tokio::test]
async fn ac_client_mode_override_does_not_produce_startup_error() {
    let harness_result = HarnessBuilder::new()
        .isolation(IsolationMode::Logical)
        .with_customer_overrides(TENANT, |spec| {
            spec.dtu_types = vec![DtuType::PagerDuty];
        })
        .build()
        .await;

    assert!(
        harness_result.is_ok(),
        "BC-3.3.001 EC-003: HarnessBuilder with DtuType::PagerDuty must NOT produce a \
         startup error; got: {:?}",
        harness_result.err()
    );

    let harness = harness_result.unwrap();
    let addr = harness
        .endpoint_for(TENANT, DtuType::PagerDuty)
        .expect("PagerDuty endpoint must be present after successful build");

    let client = reqwest::Client::new();
    let resp = client
        .get(format!("http://{addr}/dtu/health"))
        .send()
        .await
        .expect("GET /dtu/health must not fail at network level");

    assert_eq!(
        resp.status().as_u16(),
        200,
        "BC-3.3.001 EC-003: PagerDuty clone health check must return 200 after clean startup"
    );
}
