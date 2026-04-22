//! VP-033 integration test — Audit buffer write-before-delivery.
//!
//! Verifies that `WRITE_INTENT` audit record is durable in the audit backend BEFORE
//! the DTU receives the HTTP call, and `WRITE_OUTCOME` is recorded AFTER 202 returns.
//!
//! Requires `prism-dtu-crowdstrike` with `feature = "dtu"`.
//! Must pass as part of `just integration-test`.

// TODO(S-6.07): Implement once CrowdstrikeClone::start() and prism-audit backend are wired.
// This test file is a compile-only stub for Red Gate step 1.
// The test body will be filled in during Red Gate step 2 (failing tests).

#[tokio::test]
async fn crowdstrike_vp033_write_intent_before_dtu_arrival() {
    // TODO(S-6.07): Wire CrowdstrikeClone, prism-audit InMemoryBackend, SensorAdapter.
    // For now this is a placeholder that panics so Red Gate is satisfied (test fails).
    panic!("VP-033 integration test not yet implemented — Red Gate step 2 pending");
}
