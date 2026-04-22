//! VP-036 integration test — SessionContext dropped before error propagation (S-6.07 Task 10).
//!
//! Verifies that when Step 2 of a `crowdstrike_hosts` query returns HTTP 500,
//! the `SessionContext` is dropped (via `Arc::weak_count` check) BEFORE
//! `E-SENSOR-002` is returned to the caller.
//!
//! # Dependencies
//!
//! This test requires `prism-sensors::SessionContext` and the sensor query execution
//! pipeline, which are scheduled for Wave 2/3 stories S-3.06 and S-3.07. The
//! `Arc::weak_count` instrumentation also requires SessionContext to expose a
//! `weak_ref()` accessor.
//!
//! The test is marked `#[ignore = "needs-prism-audit"]` so it:
//! - Compiles and appears in `cargo test` output with clear traceability
//! - Does not block the Red Gate
//! - Will be un-ignored when S-3.06 lands

use prism_dtu_common::{BehavioralClone, FailureMode, StubConfig};
use prism_dtu_crowdstrike::CrowdstrikeClone;

/// VP-036: SessionContext is dropped before E-SENSOR-002 propagates to caller.
///
/// Wiring: Start CrowdstrikeClone with FailureMode::InternalError { at_request_n: 2 }
/// (Step 1 succeeds, Step 2 returns 500) →
/// Run a full `crowdstrike_hosts` query through SensorAdapter →
/// Capture Arc<SessionContext> weak reference before the query →
/// Assert: weak_count drops to 0 before the Err(E-SENSOR-002) is returned.
///
/// Blocked by: S-3.06 (SensorAdapter + SessionContext).
/// Un-ignore when: prism-sensors SessionContext is available as a dev-dependency.
#[tokio::test]
#[ignore = "needs-prism-audit"]
async fn crowdstrike_vp036_session_context_drops_before_error() {
    // Step 1: Start CrowdStrike DTU clone with Step-2 failure injection.
    let mut clone = CrowdstrikeClone::with_config(StubConfig {
        seed: 42,
        latency_ms: 0,
        failure_mode: FailureMode::InternalError { at_request_n: 2 },
    });
    clone.start().await.expect("VP-036: CrowdstrikeClone::start() must succeed");
    let base_url = clone.base_url();

    // Step 2: Execute a crowdstrike_hosts query.
    // TODO(S-3.06): let adapter = prism_sensors::SensorAdapter::new(&base_url);
    // TODO(S-3.06): let session_weak = adapter.session_weak_ref();
    // TODO(S-3.06): let result = adapter.query_hosts().await;

    // Step 3: Assert SessionContext was dropped before the error is returned.
    // TODO(S-3.06): assert_eq!(Arc::weak_count(&session_weak), 0,
    //     "VP-036: SessionContext must be dropped before E-SENSOR-002 propagates");
    // TODO(S-3.06): assert!(result.is_err(), "VP-036: query must return Err with E-SENSOR-002");

    let _ = base_url;

    panic!("VP-036: test body is a stub — un-ignore after S-3.06 lands (SensorAdapter + SessionContext)");
}

/// VP-036 smoke: Step 2 returns 500 when FailureMode::InternalError { at_request_n: 2 }.
///
/// This sub-test is NOT ignored — it verifies the failure injection plumbing VP-036
/// depends on, and will fail at runtime because start() is still unimplemented!().
#[tokio::test]
async fn crowdstrike_vp036_step2_returns_500_on_internal_error_injection() {
    // Expected failure: CrowdstrikeClone::start() panics with "not yet implemented".
    let mut clone = CrowdstrikeClone::with_config(StubConfig {
        seed: 42,
        latency_ms: 0,
        failure_mode: FailureMode::InternalError { at_request_n: 2 },
    });
    clone.start().await.expect("VP-036 smoke: start() must succeed");

    let base_url = clone.base_url();
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .unwrap();
    let session_id = "vp036-smoke-session";

    // Request 1 (Step 1): list host IDs — must succeed.
    let step1 = client
        .get(format!("{base_url}/devices/queries/devices/v1"))
        .header("Authorization", "Bearer dtu-fake-cs-token")
        .header("X-DTU-Session-Id", session_id)
        .send()
        .await
        .expect("VP-036 smoke: Step 1 request must reach server");

    assert_eq!(
        step1.status().as_u16(),
        200,
        "VP-036 smoke: Step 1 must return HTTP 200 (request #1 is not the injection target)"
    );

    // Request 2 (Step 2): batch detail fetch — must return 500 per FailureMode.
    let step2 = client
        .get(format!("{base_url}/devices/entities/devices/v2"))
        .query(&[("ids", "h-001")])
        .header("Authorization", "Bearer dtu-fake-cs-token")
        .header("X-DTU-Session-Id", session_id)
        .send()
        .await
        .expect("VP-036 smoke: Step 2 request must reach server");

    assert_eq!(
        step2.status().as_u16(),
        500,
        "VP-036 smoke: Step 2 must return HTTP 500 (FailureMode::InternalError at_request_n=2)"
    );
}
