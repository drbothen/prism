//! VP-033 integration test — Audit buffer write-before-delivery (S-6.07 Task 9).
//!
//! Verifies that the `WRITE_INTENT` audit record is durable in the audit backend
//! BEFORE the DTU receives the HTTP call, and that `WRITE_OUTCOME` is recorded
//! AFTER the DTU returns 202.
//!
//! # Dependencies
//!
//! This test requires `prism-audit` (InMemoryBackend) and `SensorAdapter`, neither
//! of which exists in the current wave. These are scheduled for Wave 2/3 stories
//! S-3.06 (write operations integration) and S-3.07 (write safety system).
//!
//! The test is marked `#[ignore = "needs-prism-audit"]` so it:
//! - Compiles and appears in `cargo test` output with clear traceability
//! - Does not block the Red Gate (all non-ignored tests must fail at runtime)
//! - Will be un-ignored when S-3.07 lands and prism-audit is available

use prism_dtu_common::{BehavioralClone, FailureMode, StubConfig};
use prism_dtu_crowdstrike::CrowdstrikeClone;

/// VP-033: WRITE_INTENT audit record is durable before DTU receives the HTTP call.
///
/// Wiring: Start CrowdstrikeClone → configure prism-audit InMemoryBackend →
/// configure SensorAdapter (base_url = clone.base_url()) → issue contain write →
/// capture DTU request-arrival timestamp via in-memory log →
/// assert WRITE_INTENT.committed_at < dtu_arrival_at < WRITE_OUTCOME.committed_at.
///
/// Blocked by: S-3.07 (prism-audit InMemoryBackend + SensorAdapter).
/// Un-ignore when: prism-audit crate is available as a dev-dependency.
#[tokio::test]
#[ignore = "needs-prism-audit"]
async fn crowdstrike_vp033_write_intent_before_dtu_arrival() {
    // Step 1: Start the CrowdStrike DTU clone.
    let mut clone = CrowdstrikeClone::with_config(StubConfig {
        seed: 42,
        latency_ms: 0,
        failure_mode: FailureMode::None,
    });
    clone
        .start()
        .await
        .expect("VP-033: CrowdstrikeClone::start() must succeed");
    let base_url = clone.base_url();

    // Step 2: Configure prism-audit InMemoryBackend.
    // TODO(S-3.07): use prism_audit::InMemoryBackend::new() here.
    // let audit_backend = prism_audit::InMemoryBackend::new();

    // Step 3: Configure SensorAdapter with base_url = clone.base_url().
    // TODO(S-3.07): use prism_sensors::SensorAdapter::new(base_url).
    // let adapter = prism_sensors::SensorAdapter::new(&base_url);

    // Step 4: Issue a contain write through the SensorAdapter.
    // TODO(S-3.07): adapter.contain("h-001").await.expect("contain must succeed");

    // Step 5: Capture DTU request-arrival timestamp from in-memory log.
    // TODO(S-3.07): let dtu_arrival = audit_backend.last_dtu_request_arrival();

    // Step 6: Assert ordering invariant from VP-033:
    //   WRITE_INTENT.committed_at < dtu_arrival_at
    //   dtu_arrival_at < WRITE_OUTCOME.committed_at
    // TODO(S-3.07): assert!(write_intent.committed_at < dtu_arrival, "VP-033 ordering violated");
    // TODO(S-3.07): assert!(dtu_arrival < write_outcome.committed_at, "VP-033 ordering violated");

    // Suppress unused variable warning for base_url until wired.
    let _ = base_url;

    // This line is unreachable once the test is un-ignored and wired — it is here
    // only to document the expected assertion failure when the stub is used directly.
    panic!(
        "VP-033: test body is a stub — un-ignore after S-3.07 lands (prism-audit InMemoryBackend)"
    );
}

/// VP-033 smoke: the DTU clone starts and the contain endpoint responds 202.
///
/// This sub-test is NOT ignored — it verifies the DTU plumbing that VP-033 will
/// rely on, and it will fail at runtime because CrowdstrikeClone::start() is
/// still `unimplemented!()`.
#[tokio::test]
async fn crowdstrike_vp033_contain_endpoint_returns_202_smoke() {
    // Expected failure: CrowdstrikeClone::start() panics with "not yet implemented".
    let mut clone = CrowdstrikeClone::new();
    clone
        .start()
        .await
        .expect("VP-033 smoke: start() must succeed");

    let base_url = clone.base_url();
    let client = reqwest::Client::new();

    let resp = client
        .post(format!("{base_url}/devices/entities/devices-actions/v2"))
        .query(&[("action_name", "contain")])
        .header("Authorization", "Bearer dtu-fake-cs-token")
        .json(&serde_json::json!({"ids": ["h-001"]}))
        .send()
        .await
        .expect("VP-033 smoke: contain request must reach server");

    assert_eq!(
        resp.status().as_u16(),
        202,
        "VP-033 smoke: contain must return HTTP 202"
    );

    let body: serde_json::Value = resp
        .json()
        .await
        .expect("VP-033 smoke: body must be valid JSON");

    let resources = body["resources"]
        .as_array()
        .expect("VP-033 smoke: resources must be array");
    assert!(
        !resources.is_empty(),
        "VP-033 smoke: resources must not be empty"
    );
    assert_eq!(
        resources[0]["containment_status"].as_str().unwrap_or(""),
        "contained",
        "VP-033 smoke: containment_status must be 'contained'"
    );
}
