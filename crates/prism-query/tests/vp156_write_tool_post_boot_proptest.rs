#![allow(non_snake_case, clippy::unwrap_used, clippy::expect_used)]
//! VP-156 prop 5/5: Post-boot registration rejected with WriteToolRegistrationAfterBoot.
//!
//! This test is in a SEPARATE BINARY from `vp156_write_tool_registration_uniqueness.rs`
//! because it calls `mark_query_phase_started()`, which permanently sets the
//! process-global `QUERY_PHASE_STARTED` AtomicBool to `true`. Any subsequent
//! `register_write_tool` call in the same process would observe stale state and
//! be unconditionally rejected — contaminating the proptests in the main file.
//!
//! By placing this test in a separate integration test binary, cargo (and nextest)
//! ensure it runs in a fresh process. See `invalidation_post_boot_test.rs` for
//! the established binary-isolation pattern.
//!
//! | Test | Name | VP-156 AC | BC Clause |
//! |------|------|-----------|-----------|
//! | 5 | prop_register_write_tool_post_boot_rejected_with_e_plugin_020 | Case 2 (post-boot) | EC-016-012-005 |
//!
//! Story: S-PLUGIN-PREREQ-E | BC-2.16.012 EC-016-012-005 | ADR-026 §D7 v1.23
//! VP: VP-156 (WriteToolInvalidationMap registration uniqueness)

use prism_core::SensorId;
use prism_query::invalidation::{
    mark_query_phase_started, register_write_tool, WriteToolInvalidationMap,
};
use prism_spec_engine::error::SpecEngineError;
use proptest::prelude::*;

proptest! {
    /// VP-156 prop 5/5: After `mark_query_phase_started()`, any `register_write_tool`
    /// call MUST return `Err(SpecEngineError::WriteToolRegistrationAfterBoot)` (E-PLUGIN-020).
    ///
    /// Exercises VP-156 §Acceptance Criteria (post-boot fail-closed path):
    /// - mark_query_phase_started() permanently closes the registration window
    /// - Any subsequent register_write_tool call returns Err(WriteToolRegistrationAfterBoot)
    /// - Error code E-PLUGIN-020 is cited (distinct from E-PLUGIN-021 / RwLock poisoning)
    ///
    /// State setup: mark_query_phase_started() is called ONCE at the TOP of this proptest
    /// block (outside the per-run body) to set the flag before any proptest case executes.
    /// Each case then exercises a different arbitrary entry against the permanently-closed
    /// registration window.
    ///
    /// Process isolation: this binary is fresh — QUERY_PHASE_STARTED starts false,
    /// DYNAMIC_WRITE_TOOLS starts empty. No reset hooks are needed because the flag
    /// is only ever set in one direction (false → true) and we want it true for all cases.
    ///
    /// Story: S-PLUGIN-PREREQ-E | BC-2.16.012 EC-016-012-005 | ADR-026 §D7 v1.23
    /// VP-156 prop 5/5 | BC clause: EC-016-012-005 (post-boot rejection)
    #[test]
    fn prop_register_write_tool_post_boot_rejected_with_e_plugin_020(
        tool_name in r"[a-z][a-z0-9_]{0,30}[a-z0-9]|[a-z]".prop_map(String::from),
        plugin_name in prop_oneof![
            r"[a-z]".prop_map(String::from),
            r"[a-z][a-z0-9_]{0,20}[a-z0-9]".prop_map(String::from),
        ],
        sensor_id_str in prop_oneof![
            r"[a-z]".prop_map(String::from),
            r"[a-z][a-z0-9_]{0,20}[a-z0-9]".prop_map(String::from),
        ],
        source_ids in prop::collection::vec(
            prop_oneof![
                r"[a-z]".prop_map(String::from),
                r"[a-z][a-z0-9_]{0,20}[a-z0-9]".prop_map(String::from),
            ],
            1..=4usize,
        ),
    ) {
        // Mark query phase started — permanently closes registration window.
        // Idempotent (AtomicBool::store true); safe to call multiple times across
        // proptest cases in the same process.
        mark_query_phase_started();

        let entry = WriteToolInvalidationMap::new(
            tool_name.clone(),
            source_ids,
            SensorId::from(sensor_id_str.as_str()),
            plugin_name.clone(),
        );

        let result = register_write_tool(entry);

        prop_assert!(
            result.is_err(),
            "VP-156 prop 5 (EC-016-012-005): register_write_tool after \
             mark_query_phase_started() must return Err; \
             tool_name={:?}; plugin_name={:?}; got Ok(())",
            tool_name,
            plugin_name
        );

        let err = result.unwrap_err();

        // Must be WriteToolRegistrationAfterBoot (E-PLUGIN-020), NOT
        // WriteToolRegistryPoisoned (E-PLUGIN-021). These are distinct variants
        // with distinct semantics (F-LP-IMPL-P1-004).
        prop_assert!(
            matches!(err, SpecEngineError::WriteToolRegistrationAfterBoot),
            "VP-156 prop 5 (EC-016-012-005): error variant must be \
             WriteToolRegistrationAfterBoot (E-PLUGIN-020), NOT \
             WriteToolRegistryPoisoned (E-PLUGIN-021); \
             tool_name={:?}; got: {:?}",
            tool_name,
            err
        );

        let err_str = format!("{err}");
        prop_assert!(
            err_str.contains("E-PLUGIN-020"),
            "VP-156 prop 5 (EC-016-012-005): post-boot error must cite E-PLUGIN-020; \
             tool_name={:?}; got: {}",
            tool_name,
            err_str
        );
    }
}
