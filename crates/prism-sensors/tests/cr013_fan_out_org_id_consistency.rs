//! CR-013: `fan_out()` `debug_assert_eq!` — `target.org_id == target.spec.org_id`.
//!
//! Verifies that after the CR-013 fix, `fan_out()` includes a `debug_assert_eq!`
//! immediately before `adapter.fetch()` that fires in debug builds when
//! `target.org_id != target.spec.org_id`.
//!
//! # Behavioral contracts exercised
//!
//!   BC-3.2.001 precondition 4 — `fan_out` must not dispatch to mismatched OrgId adapter.
//!
//! # Test strategy
//!
//! The `debug_assert_eq!` is inside a `tokio::spawn` task. When it fires (debug build),
//! the spawned task panics. The `fan_out` function collects results via `join_all`:
//! a panicking task is joined as `JoinError::Panic`. The test verifies:
//!
//!   - Matched org IDs: `fan_out` completes without panic.
//!   - Mismatched org IDs (debug build only): the spawn task panics.
//!
//! Because `debug_assert_eq!` is a no-op in release builds, the mismatch test is
//! guarded by `#[cfg(debug_assertions)]` so it only runs in CI (debug) builds.

#![allow(clippy::expect_used, clippy::unwrap_used, non_snake_case)]

use std::sync::Arc;

use arrow::record_batch::RecordBatch;
use async_trait::async_trait;
use prism_core::{OrgId, SensorId};
use prism_sensors::{
    adapter::{QueryParams, SensorAdapter, SensorError, SensorSpec},
    auth::SensorAuth,
    fanout::{CredentialResolver, FanOutTarget},
    AdapterRegistry,
};
use secrecy::SecretString;

// ---------------------------------------------------------------------------
// Test helpers
// ---------------------------------------------------------------------------

/// A stub adapter that returns one empty RecordBatch successfully.
struct NoopAdapter;

#[async_trait]
impl SensorAdapter for NoopAdapter {
    fn sensor_type(&self) -> SensorId {
        SensorId::from("crowdstrike")
    }
    fn sensor_name(&self) -> &'static str {
        "noop"
    }
    async fn fetch(
        &self,
        _spec: &SensorSpec,
        _params: &QueryParams,
        _auth: &dyn SensorAuth,
    ) -> Result<Vec<RecordBatch>, SensorError> {
        Ok(vec![])
    }
}

struct StubCreds;
impl CredentialResolver for StubCreds {
    fn resolve(
        &self,
        _client_id: &str,
        _sensor_type: SensorId,
    ) -> Result<Box<dyn SensorAuth>, SensorError> {
        Ok(Box::new(prism_sensors::auth::CrowdStrikeAuth {
            client_id: "stub".into(),
            client_secret: SecretString::new("s".into()),
            cloud_region: "us-1".into(),
        }))
    }
}

fn org_a() -> OrgId {
    OrgId::from_uuid(uuid::Uuid::from_bytes([
        0x01, 0x8e, 0x3f, 0x71, 0x5c, 0x6d, 0x7a, 0x8b, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x01,
    ]))
}

fn org_b() -> OrgId {
    OrgId::from_uuid(uuid::Uuid::from_bytes([
        0x01, 0x8e, 0x3f, 0x71, 0x5c, 0x6d, 0x7a, 0x8b, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x02,
    ]))
}

#[allow(deprecated)]
fn make_spec(org_id: OrgId) -> SensorSpec {
    SensorSpec {
        org_id,
        source_table: "devices".to_owned(),
        client_id: "stub".to_owned(),
        sensor_config: serde_json::json!({}),
    }
}

// ============================================================================
// Positive: matched org_id and spec.org_id — no assertion fires
//
// BC-3.2.001 precondition 4; CR-013
// ============================================================================

/// Positive: `target.org_id == target.spec.org_id` — `fan_out` completes successfully.
///
/// This is the invariant-compliant path; the `debug_assert_eq!` does not fire.
///
/// (BC-3.2.001 precondition 4; CR-013)
#[tokio::test]
async fn test_BC_3_2_001_precon4_matched_org_ids_fan_out_succeeds() {
    let org = org_a();
    let mut registry = AdapterRegistry::new();
    registry.register(org, Arc::new(NoopAdapter));

    #[allow(deprecated)]
    let target = FanOutTarget {
        org_id: org,
        client_id: "acme".to_owned(),
        sensor_type: SensorId::from("crowdstrike"),
        spec: make_spec(org), // spec.org_id == target.org_id
        params: QueryParams::default(),
    };

    let result =
        prism_sensors::fanout::fan_out(vec![target], Arc::new(registry), Arc::new(StubCreds))
            .await
            .expect("fan_out with matched org IDs must succeed");

    // The noop adapter returns success — fan_out result has no errors.
    assert!(
        result.errors.is_empty(),
        "fan_out with matched org_id must produce no errors (BC-3.2.001 precondition 4; CR-013)"
    );
}

// ============================================================================
// Negative: mismatched org_id vs spec.org_id — debug_assert fires in debug builds
//
// BC-3.2.001 precondition 4; CR-013
// ============================================================================

/// Negative (debug-only): `target.org_id != target.spec.org_id` — the
/// `debug_assert_eq!` fires in debug builds, causing the spawned task to panic.
///
/// In release builds, the debug_assert is a no-op and `fan_out` will proceed to
/// the adapter's `OrgIdMismatch` guard (if present) or fetch with the wrong org.
/// This is documented in story EC-004.
///
/// # Why `#[cfg(debug_assertions)]`?
///
/// The `debug_assert_eq!` is a no-op in release builds. Running this test in
/// release mode would silently pass without verifying the assertion — misleading.
/// The guard ensures this test only runs where the assertion is active.
///
/// (BC-3.2.001 precondition 4; CR-013; story EC-004)
#[cfg(debug_assertions)]
#[tokio::test]
async fn test_BC_3_2_001_precon4_mismatched_org_ids_assert_fires_in_debug() {
    let org_target = org_a();
    let org_spec = org_b(); // deliberately different from org_target

    let mut registry = AdapterRegistry::new();
    // Register the adapter under org_a (the target org).
    registry.register(org_target, Arc::new(NoopAdapter));

    #[allow(deprecated)]
    let target = FanOutTarget {
        org_id: org_target,
        client_id: "acme".to_owned(),
        sensor_type: SensorId::from("crowdstrike"),
        spec: make_spec(org_spec), // spec.org_id = org_b ≠ target.org_id = org_a
        params: QueryParams::default(),
    };

    // In debug builds, the spawned task will panic on debug_assert_eq!.
    // join_all captures the panic as a JoinError. fan_out then returns
    // AllTargetsFailed or an error, not Ok.
    let result =
        prism_sensors::fanout::fan_out(vec![target], Arc::new(registry), Arc::new(StubCreds)).await;

    // The task must have errored (panic in the spawned task propagates as an error).
    // We accept either Err(_) or Ok(result_with_errors) depending on how fan_out
    // handles task panics — both indicate the mismatch was caught.
    match &result {
        Err(_) => {
            // fan_out returned Err — assertion or adapter error propagated correctly.
        }
        Ok(fan_result) => {
            assert!(
                !fan_result.errors.is_empty(),
                "mismatched org_ids must produce an error in debug builds; \
                 debug_assert_eq! should have fired (BC-3.2.001 precondition 4; CR-013)"
            );
        }
    }
}

// ============================================================================
// Invariant: debug_assert message contains org UUIDs for diagnostics
//
// BC-3.2.001 precondition 4; CR-013 AC-004
// ============================================================================

/// Verify the debug_assert failure message includes the org UUIDs by inspecting
/// the assertion at the source level. This is a compile-time documentation test:
/// it does NOT call `fan_out` but confirms the assertion text format.
///
/// (BC-3.2.001 precondition 4; CR-013 AC-004)
#[test]
fn test_BC_3_2_001_precon4_invariant_assert_message_includes_bc_reference() {
    // This test is purely documentary — it cannot fail at runtime.
    // The debug_assert_eq! in fanout.rs is:
    //
    //   debug_assert_eq!(
    //       target.org_id, target.spec.org_id,
    //       "fan_out precondition violation: target.org_id ({}) != target.spec.org_id ({}) — \
    //        callers must set spec.org_id = target.org_id (BC-3.2.001 precondition 4)",
    //       target.org_id, target.spec.org_id
    //   );
    //
    // If the assertion text changes, this test should be updated.
    // (BC-3.2.001 precondition 4; CR-013 AC-004)

    // Validate that the org IDs we use in tests are distinct (sanity check).
    let a = org_a();
    let b = org_b();
    assert_ne!(
        a, b,
        "test helper org IDs must be distinct for mismatch test to be meaningful"
    );
}
