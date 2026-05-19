#![allow(non_snake_case, clippy::unwrap_used, clippy::expect_used)]
//! VP-156: WriteToolInvalidationMap — Registration Uniqueness
//!
//! Property: For any sequence of `register_write_tool(entry)` calls, if two calls
//! carry the same `tool_name`, the second call MUST return
//! `Err(SpecEngineError::DuplicateWriteToolRegistration(tool_name))`.
//! The first registration persists unchanged; no silent last-writer-wins override.
//!
//! Uniqueness invariant: keyed on `tool_name` alone (VP-156 §Property Statement;
//! BC-2.16.012 EC-016-012-004; ADR-026 D7 v1.23).
//!
//! ## Test inventory
//!
//! | Test | Name | VP-156 AC | BC Clause |
//! |------|------|-----------|-----------|
//! | 1 | prop_register_write_tool_first_call_succeeds | Case 1 (unique) | INV-INVALIDATION-EXT-001 postcondition |
//! | 2 | prop_register_write_tool_duplicate_rejected_with_e_plugin_012 | Case 2 (duplicate) | EC-016-012-004 |
//! | 3 | prop_register_write_tool_distinct_keys_accepted | Case 3 (mixed/distinct) | INV-INVALIDATION-EXT-001 |
//! | 4 | prop_register_write_tool_idempotency_under_dedup_key | Case 2 variant (full-key equality) | EC-016-012-004 — uniqueness on tool_name, not logical-equivalence |
//!
//! PROPTEST_CASES: respects env var (32 for `just iter`, 256 for `just check`).
//!
//! NOTE: The post-boot-rejection proptest (Case prop_register_write_tool_post_boot_rejected)
//! is in `vp156_write_tool_post_boot_proptest.rs` (separate binary) because it calls
//! `mark_query_phase_started()`, which permanently sets the process-global
//! `QUERY_PHASE_STARTED` AtomicBool. Placing it in this binary would contaminate
//! all subsequent `register_write_tool` calls in this process.
//! See `invalidation_post_boot_test.rs` for the established pattern.
//!
//! Story: S-PLUGIN-PREREQ-E | BC: BC-2.16.012 EC-016-012-004/005 | ADR-026 §D7 v1.23
//! VP: VP-156 (WriteToolInvalidationMap registration uniqueness)
//! Anchored: TD-S-PLUGIN-PREREQ-A-003 closure semantic
//! Sibling-sweep: F-LP-IMPL-P8-IMP-001 (same class as VP-153 proptest blind-spot)

use prism_core::SensorId;
use prism_query::invalidation::{
    dynamic_write_tool_count, register_write_tool, reset_dynamic_registry_global,
    reset_query_phase_global, WriteToolInvalidationMap,
};
use prism_spec_engine::error::SpecEngineError;
use proptest::prelude::*;

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

/// Strategy: a valid `tool_name` string — ASCII alphanumeric + underscore, 1..=32 chars.
///
/// Realistic bounds for plugin-registered write tool names (e.g.,
/// `"crowdstrike_contain_host"`, `"custom_plugin_write_record"`). Bounded length
/// prevents OOM in the uniqueness-check Vec scan.
///
/// Uniqueness semantics are on `tool_name` alone (VP-156 §Property Statement;
/// ADR-026 D7 v1.23). Other fields (plugin_name, sensor_id, source_ids) do NOT
/// affect uniqueness keying.
fn arb_tool_name() -> impl Strategy<Value = String> {
    r"[a-z][a-z0-9_]{0,31}".prop_map(String::from)
}

/// Strategy: a valid `plugin_name` string — starts and ends with lowercase alpha,
/// may contain lowercase alphanumeric and underscore in the middle, 1..=22 chars total.
///
/// AD-017: only logical names, not credential values.
fn arb_plugin_name() -> impl Strategy<Value = String> {
    // Single-char or multi-char: start=[a-z], middle=[a-z0-9_]{0,20}, end=[a-z0-9]
    prop_oneof![
        r"[a-z]".prop_map(String::from),
        r"[a-z][a-z0-9_]{0,20}[a-z0-9]".prop_map(String::from),
    ]
}

/// Strategy: a valid `sensor_id` string conforming to SensorId::from constraints:
/// 1..=64 lowercase alphanumeric + `_` + `-`; no leading/trailing `_` or `-`.
///
/// Bounded to 1..=22 chars for proptest performance.
fn arb_sensor_id() -> impl Strategy<Value = String> {
    // Single-char or multi-char: start=[a-z], middle=[a-z0-9_]{0,20}, end=[a-z0-9]
    prop_oneof![
        r"[a-z]".prop_map(String::from),
        r"[a-z][a-z0-9_]{0,20}[a-z0-9]".prop_map(String::from),
    ]
}

/// Strategy: a list of 1..=4 `source_id` strings.
///
/// Each source_id follows the same naming convention as sensor_id: starts and ends
/// with lowercase alphanumeric (no leading/trailing underscore).
fn arb_source_ids() -> impl Strategy<Value = Vec<String>> {
    prop::collection::vec(
        prop_oneof![
            r"[a-z]".prop_map(String::from),
            r"[a-z][a-z0-9_]{0,20}[a-z0-9]".prop_map(String::from),
        ],
        1..=4usize,
    )
}

/// Strategy: two distinct `tool_name` strings (guaranteed not equal).
fn arb_two_distinct_tool_names() -> impl Strategy<Value = (String, String)> {
    (arb_tool_name(), arb_tool_name()).prop_filter("tool names must be distinct", |(a, b)| a != b)
}

// ---------------------------------------------------------------------------
// Proptest 1 of 4: First call succeeds (VP-156 AC Case 1)
// ---------------------------------------------------------------------------

proptest! {
    /// VP-156 prop 1/4: First `register_write_tool(entry)` call MUST return `Ok(())`.
    ///
    /// Exercises VP-156 §Acceptance Criteria Case 1 (unique registrations all succeed):
    /// for a single arbitrary entry, the first registration succeeds and the entry is
    /// observable in the map (len == 1).
    ///
    /// Global state is reset before each proptest run to guarantee isolation.
    ///
    /// Story: S-PLUGIN-PREREQ-E | BC-2.16.012 INV-INVALIDATION-EXT-001 | ADR-026 §D7 v1.23
    /// VP-156 prop 1/4 | BC clause: postcondition (first registration succeeds)
    #[test]
    fn prop_register_write_tool_first_call_succeeds(
        tool_name in arb_tool_name(),
        plugin_name in arb_plugin_name(),
        sensor_id_str in arb_sensor_id(),
        source_ids in arb_source_ids(),
    ) {
        // Reset process-global state for isolation.
        // QUERY_PHASE_STARTED must be false; DYNAMIC_WRITE_TOOLS must be empty.
        reset_query_phase_global();
        reset_dynamic_registry_global();

        let entry = WriteToolInvalidationMap {
            tool_name: tool_name.clone(),
            source_ids,
            sensor_id: SensorId::from(sensor_id_str.as_str()),
            plugin_name,
        };

        let result = register_write_tool(entry);

        prop_assert!(
            result.is_ok(),
            "VP-156 Case 1: first register_write_tool call must return Ok(()); \
             tool_name={:?}; got: {:?}",
            tool_name,
            result.err()
        );

        // Verify the entry is now observable (VP-156 AC Case 1: len == N after N unique regs).
        let count = dynamic_write_tool_count();
        prop_assert_eq!(
            count,
            1usize,
            "VP-156 Case 1: after first registration, dynamic_write_tool_count must be 1; \
             tool_name={:?}; got count={}",
            tool_name,
            count
        );
    }
}

// ---------------------------------------------------------------------------
// Proptest 2 of 4: Duplicate tool_name rejected with DuplicateWriteToolRegistration
// (VP-156 AC Case 2)
// ---------------------------------------------------------------------------

proptest! {
    /// VP-156 prop 2/4: A second `register_write_tool` call with the same `tool_name`
    /// MUST return `Err(SpecEngineError::DuplicateWriteToolRegistration(tool_name))`.
    /// The map must still contain exactly one entry for that tool_name (the original).
    ///
    /// Exercises VP-156 §Acceptance Criteria Case 2 (duplicate name → error):
    /// - Second call returns Err matching E-PLUGIN-012
    /// - Map still contains exactly ONE entry for the duplicated tool_name
    /// - Last-writer-wins is explicitly forbidden (ADR-026 D7 v1.23)
    ///
    /// Uniqueness is keyed on `tool_name` alone — other fields (plugin_name, sensor_id,
    /// source_ids) may differ between the two entries; the second is still rejected.
    ///
    /// Story: S-PLUGIN-PREREQ-E | BC-2.16.012 EC-016-012-004 | ADR-026 §D7 v1.23
    /// VP-156 prop 2/4 | BC clause: EC-016-012-004 (duplicate rejected)
    #[test]
    fn prop_register_write_tool_duplicate_rejected_with_e_plugin_012(
        tool_name in arb_tool_name(),
        plugin_name_a in arb_plugin_name(),
        plugin_name_b in arb_plugin_name(),
        sensor_id_str_a in arb_sensor_id(),
        sensor_id_str_b in arb_sensor_id(),
        source_ids_a in arb_source_ids(),
        source_ids_b in arb_source_ids(),
    ) {
        reset_query_phase_global();
        reset_dynamic_registry_global();

        // First registration — any entry with this tool_name must succeed.
        let entry_a = WriteToolInvalidationMap {
            tool_name: tool_name.clone(),
            source_ids: source_ids_a,
            sensor_id: SensorId::from(sensor_id_str_a.as_str()),
            plugin_name: plugin_name_a,
        };
        let first = register_write_tool(entry_a);
        prop_assume!(first.is_ok(), "first registration must succeed (precondition for duplicate test)");

        // Second registration with the SAME tool_name (but arbitrary other fields).
        // Per VP-156: uniqueness is keyed on tool_name alone.
        let entry_b = WriteToolInvalidationMap {
            tool_name: tool_name.clone(),
            source_ids: source_ids_b,
            sensor_id: SensorId::from(sensor_id_str_b.as_str()),
            plugin_name: plugin_name_b,
        };
        let second = register_write_tool(entry_b);

        // Must be Err — last-writer-wins is explicitly forbidden.
        prop_assert!(
            second.is_err(),
            "VP-156 Case 2 (EC-016-012-004): duplicate tool_name must return Err; \
             tool_name={:?}; second call returned Ok(())",
            tool_name
        );

        // The error must be DuplicateWriteToolRegistration and cite E-PLUGIN-012.
        let err = second.unwrap_err();
        prop_assert!(
            matches!(err, SpecEngineError::DuplicateWriteToolRegistration(_)),
            "VP-156 Case 2: error variant must be DuplicateWriteToolRegistration; \
             tool_name={:?}; got: {:?}",
            tool_name,
            err
        );
        let err_str = format!("{err}");
        prop_assert!(
            err_str.contains("E-PLUGIN-012"),
            "VP-156 Case 2: DuplicateWriteToolRegistration must cite E-PLUGIN-012; \
             tool_name={:?}; got: {}",
            tool_name,
            err_str
        );

        // The map must still contain exactly ONE entry (the original A).
        let count = dynamic_write_tool_count();
        prop_assert_eq!(
            count,
            1usize,
            "VP-156 Case 2: after duplicate rejection, map must contain exactly 1 entry \
             (the original); tool_name={:?}; got count={}",
            tool_name,
            count
        );
    }
}

// ---------------------------------------------------------------------------
// Proptest 3 of 4: Distinct tool_names both accepted (VP-156 AC Case 3)
// ---------------------------------------------------------------------------

proptest! {
    /// VP-156 prop 3/4: Two entries with DISTINCT `tool_name` values must BOTH succeed.
    ///
    /// Exercises VP-156 §Acceptance Criteria Case 1 (N unique entries → N Ok results)
    /// and Case 3 (mixed sequence → Ok for every unique tool_name):
    /// - First call with tool_name A succeeds
    /// - Second call with tool_name B (B ≠ A) also succeeds
    /// - Final map length == 2
    ///
    /// Story: S-PLUGIN-PREREQ-E | BC-2.16.012 INV-INVALIDATION-EXT-001 | ADR-026 §D7 v1.23
    /// VP-156 prop 3/4 | BC clause: postcondition (all unique registrations succeed)
    #[test]
    fn prop_register_write_tool_distinct_keys_accepted(
        (tool_name_a, tool_name_b) in arb_two_distinct_tool_names(),
        plugin_name_a in arb_plugin_name(),
        plugin_name_b in arb_plugin_name(),
        sensor_id_str in arb_sensor_id(),
        source_ids_a in arb_source_ids(),
        source_ids_b in arb_source_ids(),
    ) {
        reset_query_phase_global();
        reset_dynamic_registry_global();

        let entry_a = WriteToolInvalidationMap {
            tool_name: tool_name_a.clone(),
            source_ids: source_ids_a,
            sensor_id: SensorId::from(sensor_id_str.as_str()),
            plugin_name: plugin_name_a,
        };
        let result_a = register_write_tool(entry_a);
        prop_assert!(
            result_a.is_ok(),
            "VP-156 Case 3: first distinct tool_name {:?} must succeed; got: {:?}",
            tool_name_a,
            result_a.err()
        );

        let entry_b = WriteToolInvalidationMap {
            tool_name: tool_name_b.clone(),
            source_ids: source_ids_b,
            sensor_id: SensorId::from(sensor_id_str.as_str()),
            plugin_name: plugin_name_b,
        };
        let result_b = register_write_tool(entry_b);
        prop_assert!(
            result_b.is_ok(),
            "VP-156 Case 3: second distinct tool_name {:?} must succeed; got: {:?}",
            tool_name_b,
            result_b.err()
        );

        // Both entries visible — map length must be 2.
        let count = dynamic_write_tool_count();
        prop_assert_eq!(
            count,
            2usize,
            "VP-156 Case 3: after two distinct registrations, map length must be 2; \
             tool_names=({:?}, {:?}); got count={}",
            tool_name_a,
            tool_name_b,
            count
        );
    }
}

// ---------------------------------------------------------------------------
// Proptest 4 of 4: Full-key idempotency — uniqueness enforced on tool_name,
// not on logical equivalence of all fields (VP-156 §Property Statement)
// ---------------------------------------------------------------------------

proptest! {
    /// VP-156 prop 4/4: Even with byte-identical entries (same tool_name AND same other
    /// fields), the second call MUST fail with DuplicateWriteToolRegistration.
    ///
    /// This verifies that uniqueness is enforced on `tool_name` and that there is
    /// NO "idempotent registration" shortcut: identical entries are still duplicates.
    /// The contract is error-on-duplicate for any second call sharing the same tool_name,
    /// regardless of whether the entry is byte-for-byte the same or different.
    ///
    /// Anchored to: VP-156 §Property Statement ("if two calls carry the same tool_name")
    /// and ADR-026 D7 v1.23 ("DuplicateWriteToolRegistration(tool_name)").
    ///
    /// Story: S-PLUGIN-PREREQ-E | BC-2.16.012 EC-016-012-004 | ADR-026 §D7 v1.23
    /// VP-156 prop 4/4 | BC clause: EC-016-012-004 (uniqueness on full-key equality)
    #[test]
    fn prop_register_write_tool_idempotency_under_dedup_key(
        tool_name in arb_tool_name(),
        plugin_name in arb_plugin_name(),
        sensor_id_str in arb_sensor_id(),
        source_ids in arb_source_ids(),
    ) {
        reset_query_phase_global();
        reset_dynamic_registry_global();

        // Build two byte-identical entries (same all fields including tool_name).
        let entry_first = WriteToolInvalidationMap {
            tool_name: tool_name.clone(),
            source_ids: source_ids.clone(),
            sensor_id: SensorId::from(sensor_id_str.as_str()),
            plugin_name: plugin_name.clone(),
        };
        let entry_second = WriteToolInvalidationMap {
            tool_name: tool_name.clone(),
            source_ids: source_ids.clone(),
            sensor_id: SensorId::from(sensor_id_str.as_str()),
            plugin_name: plugin_name.clone(),
        };

        // First registration must succeed.
        let first = register_write_tool(entry_first);
        prop_assert!(
            first.is_ok(),
            "VP-156 prop 4: first registration of {:?} must succeed; got: {:?}",
            tool_name,
            first.err()
        );

        // Second registration with byte-identical entry must STILL be rejected.
        // No idempotent registration shortcut exists — the contract is
        // error-on-duplicate for ANY second call sharing the same tool_name.
        let second = register_write_tool(entry_second);
        prop_assert!(
            second.is_err(),
            "VP-156 prop 4: second registration with identical tool_name {:?} \
             must return Err (no idempotent-registration shortcut); got Ok(())",
            tool_name
        );

        let err = second.unwrap_err();
        prop_assert!(
            matches!(err, SpecEngineError::DuplicateWriteToolRegistration(_)),
            "VP-156 prop 4: error variant must be DuplicateWriteToolRegistration; \
             tool_name={:?}; got: {:?}",
            tool_name,
            err
        );
    }
}
