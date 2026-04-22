//! VP-036 integration test — SessionContext drop before error propagation.
//!
//! Verifies that when Step 2 of a `crowdstrike_hosts` query returns HTTP 500,
//! the `SessionContext` is dropped (via `Arc::weak_count` check) BEFORE `E-SENSOR-002`
//! is returned to the caller.
//!
//! Requires `prism-dtu-crowdstrike` with `feature = "dtu"`.
//! Must pass as part of `just integration-test`.

// TODO(S-6.07): Implement once CrowdstrikeClone::start() and SessionContext drop path wired.
// This test file is a compile-only stub for Red Gate step 1.

#[tokio::test]
async fn crowdstrike_vp036_session_context_drops_before_error() {
    // TODO(S-6.07): Wire CrowdstrikeClone with FailureMode::InternalError { at_request_n: 2 },
    // execute crowdstrike_hosts query, verify Arc::weak_count drop before E-SENSOR-002.
    panic!("VP-036 integration test not yet implemented — Red Gate step 2 pending");
}
