#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Fidelity test: full incident lifecycle for the PagerDuty Events API v2 DTU.
//!
//! Tests the complete create→acknowledge→resolve lifecycle, dedup key idempotency,
//! and endpoint shape for all DTU routes.
//!
//! Run as part of `just dtu-validate`:
//! ```sh
//! cargo test -p prism-dtu-pagerduty --features dtu -- --test fidelity
//! ```

#![cfg(feature = "dtu")]

use prism_dtu_common::BehavioralClone;
use prism_dtu_pagerduty::PagerDutyClone;

/// Helper: build a reqwest client.
fn client() -> reqwest::Client {
    reqwest::Client::new()
}

/// AC-1 + AC-2 + AC-3: Full trigger → acknowledge → resolve lifecycle.
///
/// AC-1: trigger with new dedup_key → 202, status Triggered.
/// AC-2: acknowledge → 200, status Acknowledged.
/// AC-3: resolve → 200, status Resolved.
#[tokio::test]
async fn test_full_lifecycle_trigger_ack_resolve() {
    let mut clone = PagerDutyClone::new().expect("PagerDutyClone::new failed");
    clone.start().await.expect("start failed");
    let base = clone.base_url();
    let c = client();

    let dedup_key = "test-lifecycle-001";

    // --- trigger ---
    let trigger_resp = c
        .post(format!("{base}/v2/enqueue"))
        .json(&serde_json::json!({
            "routing_key": "test-rk-001",
            "event_action": "trigger",
            "dedup_key": dedup_key,
            "payload": {
                "summary": "Disk usage critical on srv01",
                "severity": "critical",
                "source": "srv01"
            }
        }))
        .send()
        .await
        .expect("trigger request failed");

    assert_eq!(
        trigger_resp.status().as_u16(),
        202,
        "AC-1: trigger must return HTTP 202"
    );

    let trigger_body: serde_json::Value = trigger_resp
        .json()
        .await
        .expect("trigger response body must be JSON");
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
    let incidents_resp = c
        .get(format!("{base}/dtu/incidents"))
        .send()
        .await
        .expect("GET /dtu/incidents failed");
    let incidents_body: serde_json::Value = incidents_resp.json().await.expect("incidents JSON");
    let incidents = incidents_body["incidents"]
        .as_array()
        .expect("incidents array");
    let incident = incidents
        .iter()
        .find(|i| i["dedup_key"].as_str() == Some(dedup_key))
        .expect("incident must be in registry after trigger");
    assert_eq!(
        incident["status"].as_str().unwrap_or(""),
        "triggered",
        "AC-1: incident status must be 'triggered'"
    );

    // --- acknowledge ---
    let ack_resp = c
        .post(format!("{base}/v2/enqueue"))
        .json(&serde_json::json!({
            "routing_key": "test-rk-001",
            "event_action": "acknowledge",
            "dedup_key": dedup_key
        }))
        .send()
        .await
        .expect("acknowledge request failed");

    assert_eq!(
        ack_resp.status().as_u16(),
        200,
        "AC-2: acknowledge must return HTTP 200"
    );

    let ack_body: serde_json::Value = ack_resp.json().await.expect("ack body JSON");
    assert_eq!(
        ack_body["status"].as_str().unwrap_or(""),
        "success",
        "AC-2: acknowledge status must be 'success'"
    );

    // Verify registry shows Acknowledged.
    let incidents_resp2 = c
        .get(format!("{base}/dtu/incidents"))
        .send()
        .await
        .expect("GET /dtu/incidents after ack failed");
    let incidents_body2: serde_json::Value = incidents_resp2.json().await.expect("incidents JSON");
    let incidents2 = incidents_body2["incidents"]
        .as_array()
        .expect("incidents array");
    let incident2 = incidents2
        .iter()
        .find(|i| i["dedup_key"].as_str() == Some(dedup_key))
        .expect("incident must still be in registry");
    assert_eq!(
        incident2["status"].as_str().unwrap_or(""),
        "acknowledged",
        "AC-2: incident status must be 'acknowledged' after ack"
    );

    // --- resolve ---
    let resolve_resp = c
        .post(format!("{base}/v2/enqueue"))
        .json(&serde_json::json!({
            "routing_key": "test-rk-001",
            "event_action": "resolve",
            "dedup_key": dedup_key
        }))
        .send()
        .await
        .expect("resolve request failed");

    assert_eq!(
        resolve_resp.status().as_u16(),
        200,
        "AC-3: resolve must return HTTP 200"
    );

    let resolve_body: serde_json::Value = resolve_resp.json().await.expect("resolve body JSON");
    assert_eq!(
        resolve_body["status"].as_str().unwrap_or(""),
        "success",
        "AC-3: resolve status must be 'success'"
    );

    // Verify registry shows Resolved.
    let incidents_resp3 = c
        .get(format!("{base}/dtu/incidents"))
        .send()
        .await
        .expect("GET /dtu/incidents after resolve failed");
    let incidents_body3: serde_json::Value = incidents_resp3.json().await.expect("incidents JSON");
    let incidents3 = incidents_body3["incidents"]
        .as_array()
        .expect("incidents array");
    let incident3 = incidents3
        .iter()
        .find(|i| i["dedup_key"].as_str() == Some(dedup_key))
        .expect("incident must still be in registry after resolve");
    assert_eq!(
        incident3["status"].as_str().unwrap_or(""),
        "resolved",
        "AC-3: incident status must be 'resolved' after resolve"
    );
}

/// AC-4: acknowledge on a resolved incident returns 400.
#[tokio::test]
async fn test_ac4_ack_on_resolved_returns_400() {
    let mut clone = PagerDutyClone::new().expect("PagerDutyClone::new failed");
    clone.start().await.expect("start failed");
    let base = clone.base_url();
    let c = client();

    let dedup_key = "test-ac4-resolved";

    // Trigger, then resolve.
    c.post(format!("{base}/v2/enqueue"))
        .json(&serde_json::json!({
            "routing_key": "rk",
            "event_action": "trigger",
            "dedup_key": dedup_key,
            "payload": {"summary": "test", "severity": "info", "source": "test"}
        }))
        .send()
        .await
        .expect("trigger failed");

    c.post(format!("{base}/v2/enqueue"))
        .json(&serde_json::json!({
            "routing_key": "rk",
            "event_action": "resolve",
            "dedup_key": dedup_key
        }))
        .send()
        .await
        .expect("resolve failed");

    // Now try to acknowledge the resolved incident.
    let ack_resp = c
        .post(format!("{base}/v2/enqueue"))
        .json(&serde_json::json!({
            "routing_key": "rk",
            "event_action": "acknowledge",
            "dedup_key": dedup_key
        }))
        .send()
        .await
        .expect("ack-after-resolve request failed");

    assert_eq!(
        ack_resp.status().as_u16(),
        400,
        "AC-4: acknowledge on resolved incident must return HTTP 400"
    );

    let body: serde_json::Value = ack_resp.json().await.expect("body JSON");
    assert_eq!(
        body["status"].as_str().unwrap_or(""),
        "cannot acknowledge a resolved incident",
        "AC-4: error status must be 'cannot acknowledge a resolved incident'"
    );
}

/// AC-5: trigger with same dedup_key as active incident is idempotent (202, no duplicate).
#[tokio::test]
async fn test_ac5_trigger_idempotent_on_active_incident() {
    let mut clone = PagerDutyClone::new().expect("PagerDutyClone::new failed");
    clone.start().await.expect("start failed");
    let base = clone.base_url();
    let c = client();

    let dedup_key = "test-ac5-idempotent";

    // First trigger.
    c.post(format!("{base}/v2/enqueue"))
        .json(&serde_json::json!({
            "routing_key": "rk",
            "event_action": "trigger",
            "dedup_key": dedup_key,
            "payload": {"summary": "first", "severity": "error", "source": "svc"}
        }))
        .send()
        .await
        .expect("first trigger failed");

    // Second trigger with same dedup_key.
    let second_resp = c
        .post(format!("{base}/v2/enqueue"))
        .json(&serde_json::json!({
            "routing_key": "rk",
            "event_action": "trigger",
            "dedup_key": dedup_key,
            "payload": {"summary": "second", "severity": "error", "source": "svc"}
        }))
        .send()
        .await
        .expect("second trigger failed");

    assert_eq!(
        second_resp.status().as_u16(),
        202,
        "AC-5: re-trigger on active incident must return HTTP 202"
    );

    // Verify only one incident in registry (not duplicated).
    let incidents_resp = c
        .get(format!("{base}/dtu/incidents"))
        .send()
        .await
        .expect("GET /dtu/incidents failed");
    let incidents_body: serde_json::Value = incidents_resp.json().await.expect("JSON");
    let matching: Vec<_> = incidents_body["incidents"]
        .as_array()
        .expect("incidents array")
        .iter()
        .filter(|i| i["dedup_key"].as_str() == Some(dedup_key))
        .collect();
    assert_eq!(
        matching.len(),
        1,
        "AC-5: only one incident must exist for the dedup_key after idempotent re-trigger"
    );
}

/// AC-6: severity "fatal" (invalid) returns 400.
#[tokio::test]
async fn test_ac6_invalid_severity_returns_400() {
    let mut clone = PagerDutyClone::new().expect("PagerDutyClone::new failed");
    clone.start().await.expect("start failed");
    let base = clone.base_url();
    let c = client();

    let resp = c
        .post(format!("{base}/v2/enqueue"))
        .json(&serde_json::json!({
            "routing_key": "rk",
            "event_action": "trigger",
            "payload": {"summary": "test", "severity": "fatal", "source": "svc"}
        }))
        .send()
        .await
        .expect("request failed");

    assert_eq!(
        resp.status().as_u16(),
        400,
        "AC-6: invalid severity 'fatal' must return HTTP 400"
    );

    let body: serde_json::Value = resp.json().await.expect("body JSON");
    assert_eq!(
        body["status"].as_str().unwrap_or(""),
        "invalid severity",
        "AC-6: error must be 'invalid severity'"
    );
}

/// AC-6 edge: "CRITICAL" (wrong casing) must also return 400 (case-sensitive per PagerDuty spec).
#[tokio::test]
async fn test_ec4_uppercase_severity_returns_400() {
    let mut clone = PagerDutyClone::new().expect("PagerDutyClone::new failed");
    clone.start().await.expect("start failed");
    let base = clone.base_url();
    let c = client();

    let resp = c
        .post(format!("{base}/v2/enqueue"))
        .json(&serde_json::json!({
            "routing_key": "rk",
            "event_action": "trigger",
            "payload": {"summary": "test", "severity": "CRITICAL", "source": "svc"}
        }))
        .send()
        .await
        .expect("request failed");

    assert_eq!(
        resp.status().as_u16(),
        400,
        "EC-004: uppercase 'CRITICAL' must return HTTP 400 (case-sensitive)"
    );
}

/// AC-7: missing routing_key returns 400.
#[tokio::test]
async fn test_ac7_missing_routing_key_returns_400() {
    let mut clone = PagerDutyClone::new().expect("PagerDutyClone::new failed");
    clone.start().await.expect("start failed");
    let base = clone.base_url();
    let c = client();

    let resp = c
        .post(format!("{base}/v2/enqueue"))
        .json(&serde_json::json!({
            "event_action": "trigger",
            "payload": {"summary": "test", "severity": "info", "source": "svc"}
        }))
        .send()
        .await
        .expect("request failed");

    assert_eq!(
        resp.status().as_u16(),
        400,
        "AC-7: missing routing_key must return HTTP 400"
    );

    let body: serde_json::Value = resp.json().await.expect("body JSON");
    assert_eq!(
        body["status"].as_str().unwrap_or(""),
        "missing routing_key",
        "AC-7: error must be 'missing routing_key'"
    );
}

/// AC-8: auth_mode "reject" causes 403 on all requests.
#[tokio::test]
async fn test_ac8_auth_reject_mode_returns_403() {
    let mut clone = PagerDutyClone::new().expect("PagerDutyClone::new failed");
    clone.start().await.expect("start failed");
    let base = clone.base_url();
    let admin_token = clone.admin_token().to_string();
    let c = client();

    // Configure auth_mode = reject.
    let configure_resp = c
        .post(format!("{base}/dtu/configure"))
        .header("X-Admin-Token", &admin_token)
        .json(&serde_json::json!({"auth_mode": "reject"}))
        .send()
        .await
        .expect("configure request failed");

    assert_eq!(
        configure_resp.status().as_u16(),
        200,
        "AC-8: /dtu/configure must return 200"
    );

    // Now any enqueue request must return 403.
    let resp = c
        .post(format!("{base}/v2/enqueue"))
        .json(&serde_json::json!({
            "routing_key": "rk",
            "event_action": "trigger",
            "payload": {"summary": "test", "severity": "info", "source": "svc"}
        }))
        .send()
        .await
        .expect("enqueue after auth_reject request failed");

    assert_eq!(
        resp.status().as_u16(),
        403,
        "AC-8: auth_reject mode must return HTTP 403"
    );

    let body: serde_json::Value = resp.json().await.expect("body JSON");
    assert_eq!(
        body["status"].as_str().unwrap_or(""),
        "invalid key",
        "AC-8: 403 status must be 'invalid key'"
    );
}

/// EC-001: trigger with no dedup_key in body — DTU generates UUID.
#[tokio::test]
async fn test_ec1_auto_generated_dedup_key() {
    let mut clone = PagerDutyClone::new().expect("PagerDutyClone::new failed");
    clone.start().await.expect("start failed");
    let base = clone.base_url();
    let c = client();

    let resp = c
        .post(format!("{base}/v2/enqueue"))
        .json(&serde_json::json!({
            "routing_key": "rk",
            "event_action": "trigger",
            "payload": {"summary": "auto key test", "severity": "warning", "source": "svc"}
        }))
        .send()
        .await
        .expect("request failed");

    assert_eq!(
        resp.status().as_u16(),
        202,
        "EC-001: trigger without dedup_key must succeed with 202"
    );

    let body: serde_json::Value = resp.json().await.expect("body JSON");
    let returned_key = body["dedup_key"].as_str().unwrap_or("");
    assert!(
        !returned_key.is_empty(),
        "EC-001: response must include an auto-generated dedup_key"
    );

    // Verify incident is in registry with the auto-generated key.
    let incidents_resp = c
        .get(format!("{base}/dtu/incidents"))
        .send()
        .await
        .expect("GET /dtu/incidents failed");
    let incidents_body: serde_json::Value = incidents_resp.json().await.expect("JSON");
    let matching: Vec<_> = incidents_body["incidents"]
        .as_array()
        .expect("incidents array")
        .iter()
        .filter(|i| i["dedup_key"].as_str() == Some(returned_key))
        .collect();
    assert_eq!(
        matching.len(),
        1,
        "EC-001: incident must be registered with auto-generated dedup_key"
    );
}

/// EC-002 / EC-003: resolve on unknown dedup_key returns 400.
#[tokio::test]
async fn test_ec2_resolve_unknown_dedup_key_returns_400() {
    let mut clone = PagerDutyClone::new().expect("PagerDutyClone::new failed");
    clone.start().await.expect("start failed");
    let base = clone.base_url();
    let c = client();

    let resp = c
        .post(format!("{base}/v2/enqueue"))
        .json(&serde_json::json!({
            "routing_key": "rk",
            "event_action": "resolve",
            "dedup_key": "no-such-incident-xyz"
        }))
        .send()
        .await
        .expect("request failed");

    assert_eq!(
        resp.status().as_u16(),
        400,
        "EC-002: resolve on unknown dedup_key must return 400"
    );

    let body: serde_json::Value = resp.json().await.expect("body JSON");
    assert_eq!(
        body["status"].as_str().unwrap_or(""),
        "invalid dedup_key",
        "EC-002: error must be 'invalid dedup_key'"
    );
}

/// EC-003: trigger on a previously resolved incident creates a fresh incident.
#[tokio::test]
async fn test_ec3_retrigger_after_resolve_creates_fresh_incident() {
    let mut clone = PagerDutyClone::new().expect("PagerDutyClone::new failed");
    clone.start().await.expect("start failed");
    let base = clone.base_url();
    let c = client();

    let dedup_key = "test-ec3-retrigger";

    // Trigger → resolve.
    c.post(format!("{base}/v2/enqueue"))
        .json(&serde_json::json!({
            "routing_key": "rk",
            "event_action": "trigger",
            "dedup_key": dedup_key,
            "payload": {"summary": "first", "severity": "error", "source": "svc"}
        }))
        .send()
        .await
        .expect("trigger failed");

    c.post(format!("{base}/v2/enqueue"))
        .json(&serde_json::json!({
            "routing_key": "rk",
            "event_action": "resolve",
            "dedup_key": dedup_key
        }))
        .send()
        .await
        .expect("resolve failed");

    // Re-trigger after resolve.
    let retrigger_resp = c
        .post(format!("{base}/v2/enqueue"))
        .json(&serde_json::json!({
            "routing_key": "rk",
            "event_action": "trigger",
            "dedup_key": dedup_key,
            "payload": {"summary": "second", "severity": "critical", "source": "svc"}
        }))
        .send()
        .await
        .expect("retrigger failed");

    assert_eq!(
        retrigger_resp.status().as_u16(),
        202,
        "EC-003: re-trigger after resolve must return 202"
    );

    // Verify incident is now Triggered again.
    let incidents_resp = c
        .get(format!("{base}/dtu/incidents"))
        .send()
        .await
        .expect("GET /dtu/incidents failed");
    let incidents_body: serde_json::Value = incidents_resp.json().await.expect("JSON");
    let incident = incidents_body["incidents"]
        .as_array()
        .expect("incidents array")
        .iter()
        .find(|i| i["dedup_key"].as_str() == Some(dedup_key))
        .expect("incident must be in registry");
    assert_eq!(
        incident["status"].as_str().unwrap_or(""),
        "triggered",
        "EC-003: fresh incident after re-trigger must have status 'triggered'"
    );
}

/// DTU health endpoint accessible without auth.
#[tokio::test]
async fn test_dtu_health_returns_200() {
    let mut clone = PagerDutyClone::new().expect("PagerDutyClone::new failed");
    clone.start().await.expect("start failed");
    let base = clone.base_url();
    let c = client();

    let resp = c
        .get(format!("{base}/dtu/health"))
        .send()
        .await
        .expect("GET /dtu/health failed");

    assert_eq!(
        resp.status().as_u16(),
        200,
        "GET /dtu/health must return 200"
    );

    let body: serde_json::Value = resp.json().await.expect("body JSON");
    assert_eq!(
        body["status"].as_str().unwrap_or(""),
        "ok",
        "/dtu/health body must have status 'ok'"
    );
}

/// DTU reset clears incident registry.
#[tokio::test]
async fn test_dtu_reset_clears_incidents() {
    let mut clone = PagerDutyClone::new().expect("PagerDutyClone::new failed");
    clone.start().await.expect("start failed");
    let base = clone.base_url();
    let c = client();

    // Create an incident.
    c.post(format!("{base}/v2/enqueue"))
        .json(&serde_json::json!({
            "routing_key": "rk",
            "event_action": "trigger",
            "dedup_key": "reset-test-001",
            "payload": {"summary": "test", "severity": "info", "source": "svc"}
        }))
        .send()
        .await
        .expect("trigger failed");

    // Reset.
    let reset_resp = c
        .post(format!("{base}/dtu/reset"))
        .send()
        .await
        .expect("POST /dtu/reset failed");
    assert_eq!(reset_resp.status().as_u16(), 200, "reset must return 200");

    // Verify incident registry is empty.
    let incidents_resp = c
        .get(format!("{base}/dtu/incidents"))
        .send()
        .await
        .expect("GET /dtu/incidents failed");
    let incidents_body: serde_json::Value = incidents_resp.json().await.expect("JSON");
    let count = incidents_body["incidents"]
        .as_array()
        .map(|a| a.len())
        .unwrap_or(usize::MAX);
    assert_eq!(count, 0, "registry must be empty after /dtu/reset");
}
