// S-1.08: VP-020 — Feature Flag: Compile AND Runtime Must Both Permit
//
// Unit-level assertion of the VP-020 property statement (mirrors the Kani
// proof harness in kani/feature_flag_proof.rs). These unit tests exercise the
// same 2×2 truth table that the Kani proof verifies symbolically.
//
// The Kani harness provides the formal proof; these unit tests provide rapid
// feedback in `cargo test` without running the full verification toolchain.
//
// AC-7: VP-020 Kani proof passes — the unit tests here verify the identical
// behavioral property at the unit test level.
//
// Naming: test_BC_2_04_004_vp020_* (traceable to BC-2.04.004, source BC for VP-020)
// Also: test_VP_020_* for the VP-level property assertions.
#![allow(non_snake_case)]

use std::collections::BTreeMap;

use prism_core::capability::{CapabilityEffect, CapabilityPath, ClientCapabilities};
use prism_security::feature_flag::{CapabilityCheckResult, CompileTimeGate, FeatureFlagEvaluator};

fn cap(s: &str) -> CapabilityPath {
    CapabilityPath::new(s).expect("test helper: valid capability path")
}

fn evaluator_with_allow(client_id: &str, path: &str) -> FeatureFlagEvaluator {
    let mut caps = ClientCapabilities::new();
    caps.grant(cap(path), CapabilityEffect::Allow);
    let mut map = BTreeMap::new();
    map.insert(client_id.to_string(), caps);
    FeatureFlagEvaluator::new(map)
}

fn evaluator_empty() -> FeatureFlagEvaluator {
    FeatureFlagEvaluator::new(BTreeMap::new())
}

// ─────────────────────────────────────────────────────────────
// 2×2 Truth Table (VP-020 property: result == compile_ok AND runtime_allow)
// ─────────────────────────────────────────────────────────────

/// Truth table cell (false, false): compile absent + runtime deny → Denied.
#[test]
fn test_VP_020_truth_table_absent_deny_is_denied() {
    let evaluator = evaluator_empty();
    let result = evaluator.check_permission(
        CompileTimeGate::Absent,
        "acme",
        "sensor.crowdstrike.containment",
    );
    assert!(
        !matches!(result, CapabilityCheckResult::Allowed),
        "VP-020 [F,F]: compile absent + runtime deny must NOT be Allowed"
    );
    assert!(
        matches!(result, CapabilityCheckResult::DeniedCompileTime { .. }),
        "VP-020 [F,F]: must be DeniedCompileTime (compile gate checked first)"
    );
}

/// Truth table cell (false, true): compile absent + runtime allow → DeniedCompileTime.
/// This is the critical security property: runtime CANNOT override missing compile feature.
#[test]
fn test_VP_020_truth_table_absent_allow_is_denied_compile_time() {
    let evaluator = evaluator_with_allow("acme", "sensor.crowdstrike.containment");
    let result = evaluator.check_permission(
        CompileTimeGate::Absent, // NOT compiled in
        "acme",
        "sensor.crowdstrike.containment",
    );
    assert!(
        matches!(result, CapabilityCheckResult::DeniedCompileTime { .. }),
        "VP-020 [F,T]: compile absent + runtime allow must be DeniedCompileTime (cannot override compile gate)"
    );
}

/// Truth table cell (true, false): compile present + runtime deny → DeniedRuntime.
#[test]
fn test_VP_020_truth_table_present_deny_is_denied_runtime() {
    let evaluator = evaluator_empty(); // no runtime caps → deny-by-default
    let result = evaluator.check_permission(
        CompileTimeGate::Present,
        "acme",
        "sensor.crowdstrike.containment",
    );
    assert!(
        matches!(result, CapabilityCheckResult::DeniedRuntime { .. }),
        "VP-020 [T,F]: compile present + runtime deny must be DeniedRuntime"
    );
}

/// Truth table cell (true, true): compile present + runtime allow → Allowed.
#[test]
fn test_VP_020_truth_table_present_allow_is_allowed() {
    let evaluator = evaluator_with_allow("acme", "sensor.crowdstrike.containment");
    let result = evaluator.check_permission(
        CompileTimeGate::Present,
        "acme",
        "sensor.crowdstrike.containment",
    );
    assert!(
        matches!(result, CapabilityCheckResult::Allowed),
        "VP-020 [T,T]: compile present + runtime allow must be Allowed"
    );
}

// ─────────────────────────────────────────────────────────────
// VP-020 corollary: result == (compile_ok AND runtime_allow)
// ─────────────────────────────────────────────────────────────

/// The combined result must equal the logical AND of both gates.
/// Exhaustively tests all 4 combinations.
#[test]
fn test_VP_020_result_equals_logical_and_of_both_gates() {
    let capability = "sensor.crowdstrike.containment";
    let client_id = "acme";

    // We need a helper that can express all 4 combinations.
    let test_cases: Vec<(CompileTimeGate, bool, bool)> = vec![
        (CompileTimeGate::Absent, false, false), // [F,F] → not Allowed
        (CompileTimeGate::Absent, true, false),  // [F,T] → not Allowed (compile gate wins)
        (CompileTimeGate::Present, false, false), // [T,F] → not Allowed
        (CompileTimeGate::Present, true, true),  // [T,T] → Allowed
    ];

    for (compile_gate, runtime_allow, expected_allowed) in test_cases {
        let evaluator = if runtime_allow {
            evaluator_with_allow(client_id, capability)
        } else {
            evaluator_empty()
        };

        let result = evaluator.check_permission(compile_gate, client_id, capability);
        let is_allowed = matches!(result, CapabilityCheckResult::Allowed);

        assert_eq!(
            is_allowed, expected_allowed,
            "VP-020 truth table: compile={:?}, runtime_allow={}, expected_allowed={}; got is_allowed={}",
            compile_gate, runtime_allow, expected_allowed, is_allowed
        );
    }
}

// ─────────────────────────────────────────────────────────────
// VP-020 applies to all four sensor write code families
// ─────────────────────────────────────────────────────────────

/// VP-020 property holds for CrowdStrike, Cyberint, Claroty, and Armis.
#[test]
fn test_VP_020_applies_to_all_sensor_write_families() {
    let sensor_paths = [
        "sensor.crowdstrike.containment",
        "sensor.cyberint.write",
        "sensor.claroty.write",
        "sensor.armis.write",
    ];

    for path in &sensor_paths {
        // [T,T] → Allowed
        let evaluator = evaluator_with_allow("acme", path);
        let result = evaluator.check_permission(CompileTimeGate::Present, "acme", path);
        assert!(
            matches!(result, CapabilityCheckResult::Allowed),
            "VP-020: [T,T] must be Allowed for path '{}'",
            path
        );

        // [F,T] → DeniedCompileTime
        let evaluator = evaluator_with_allow("acme", path);
        let result = evaluator.check_permission(CompileTimeGate::Absent, "acme", path);
        assert!(
            matches!(result, CapabilityCheckResult::DeniedCompileTime { .. }),
            "VP-020: [F,T] must be DeniedCompileTime for path '{}'",
            path
        );
    }
}

// ─────────────────────────────────────────────────────────────
// AC-7: VP-020 unit assertion (counterpart to Kani proof)
// ─────────────────────────────────────────────────────────────

/// AC-7: The VP-020 property holds at the unit level — the same property
/// that the Kani harness proves symbolically.
///
/// This is not a replacement for the Kani proof — it is a unit-level regression
/// test that runs in `cargo test` for rapid feedback.
#[test]
fn test_BC_2_04_004_vp020_unit_assertion_counterpart_to_kani_proof() {
    // All 4 gate combinations verified against the expected result.
    let matrix = [
        (false, false, false), // compile absent, runtime deny → not allowed
        (false, true, false),  // compile absent, runtime allow → not allowed
        (true, false, false),  // compile present, runtime deny → not allowed
        (true, true, true),    // compile present, runtime allow → allowed
    ];

    let capability = "sensor.crowdstrike.containment";
    let client_id = "acme";

    for (compile_ok, runtime_allow, expected) in matrix {
        let evaluator = if runtime_allow {
            evaluator_with_allow(client_id, capability)
        } else {
            evaluator_empty()
        };

        let compile_gate = if compile_ok {
            CompileTimeGate::Present
        } else {
            CompileTimeGate::Absent
        };

        let result = evaluator.check_permission(compile_gate, client_id, capability);
        let is_allowed = matches!(result, CapabilityCheckResult::Allowed);

        assert_eq!(
            is_allowed, expected,
            "VP-020 AC-7: compile_ok={}, runtime_allow={} → expected_allowed={}; got={}",
            compile_ok, runtime_allow, expected, is_allowed
        );
    }
}
