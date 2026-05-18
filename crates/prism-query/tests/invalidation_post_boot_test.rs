//! Process-isolated integration test for post-boot write tool registration.
//!
//! This test MUST run in its own binary because it calls `mark_query_phase_started()`,
//! which sets the process-global `QUERY_PHASE_STARTED` AtomicBool to `true`.
//! Any test in the same process that subsequently calls `register_write_tool`
//! before phase-started would observe stale state and fail.
//!
//! By placing this test in a separate integration test binary, cargo (and nextest)
//! ensure it runs in a fresh process with its own global-state context.
//!
//! This resolves F-LP-IMPL-P2-003 (F-005 paper-fix: `#[ignore]`'d the unit test
//! instead of fixing the structural race in the integration test binary).
//!
//! Story: S-PLUGIN-PREREQ-E / F-LP-IMPL-P2-003 | ADR-026 §D7

#![allow(clippy::unwrap_used, clippy::expect_used, non_snake_case)]

use prism_core::SensorId;
use prism_query::invalidation::{
    mark_query_phase_started, register_write_tool, WriteToolInvalidationMap,
};

/// F-LP-IMPL-P1-004 (post-boot registration): After `mark_query_phase_started()`,
/// `register_write_tool` must return `WriteToolRegistrationAfterBoot` (E-PLUGIN-020),
/// NOT `WriteToolRegistryPoisoned` (E-PLUGIN-021). This confirms the two variants
/// have DISTINCT semantics — timing violation vs. state corruption.
///
/// Runs in a SEPARATE BINARY from `invalidation_integration_test.rs` to avoid
/// contaminating the `QUERY_PHASE_STARTED` global state seen by other tests
/// in that binary (F-LP-IMPL-P2-003 structural fix).
///
/// Story: S-PLUGIN-PREREQ-E / F-LP-IMPL-P1-004 / F-LP-IMPL-P2-003 | ADR-026 §D7
#[test]
#[tracing_test::traced_test]
fn test_BC_2_07_004_post_boot_registration_returns_after_boot_not_poisoned() {
    mark_query_phase_started();

    let entry = WriteToolInvalidationMap {
        tool_name: "post_boot_integration_tool".to_string(),
        source_ids: vec!["some_data".to_string()],
        sensor_id: SensorId::from("late_plugin"),
        plugin_name: "late_registrar".to_string(),
    };

    let result = register_write_tool(entry);
    assert!(
        result.is_err(),
        "must return Err after mark_query_phase_started()"
    );

    let err = result.unwrap_err();
    let err_str = format!("{err}");
    assert!(
        err_str.contains("E-PLUGIN-020"),
        "F-LP-IMPL-P1-004: post-boot error must be E-PLUGIN-020 (WriteToolRegistrationAfterBoot), \
         NOT E-PLUGIN-021 (WriteToolRegistryPoisoned); got: {err_str}"
    );
}
