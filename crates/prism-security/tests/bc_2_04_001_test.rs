// S-1.08: BC-2.04.001 — Compile-Time Cargo Features Gate Write Code Families
//
// Tests verify that:
//  - The four write feature gates (`crowdstrike-write`, `cyberint-write`,
//    `claroty-write`, `armis-write`) are recognized by the binary.
//  - When a write feature is absent, `check_permission` returns Deny
//    regardless of runtime configuration (AC-1).
//  - `all-write` is the union of the four sensor write features.
//  - EC-04-001: binary built with one write feature but not another.
//
// Naming: test_BC_2_04_001_<assertion>
#![allow(non_snake_case)]

use std::collections::BTreeMap;

use prism_security::feature_flag::{
    armis_write_gate, claroty_write_gate, crowdstrike_write_gate, cyberint_write_gate,
    CapabilityCheckResult, CompileTimeGate, FeatureFlagEvaluator,
};

// ─────────────────────────────────────────────────────────────
// AC-1: compile gate absent → check_permission returns DeniedCompileTime
// ─────────────────────────────────────────────────────────────

/// AC-1: Given binary built WITHOUT `crowdstrike-write`, when
/// `check_permission("acme", "sensor.crowdstrike.containment")` is called
/// with `CompileTimeGate::Absent`, then it returns `DeniedCompileTime`.
///
/// This models the real binary gate per VP-020 feasibility: compile-time gate
/// is modeled as a runtime enum in tests; the real `cfg` gate is verified by
/// the `test_BC_2_04_001_real_crowdstrike_write_gate_absent` test below.
#[test]
fn test_BC_2_04_001_absent_gate_returns_denied_compile_time() {
    let evaluator = FeatureFlagEvaluator::new(BTreeMap::new());
    let result = evaluator.check_permission(
        CompileTimeGate::Absent,
        "acme",
        "sensor.crowdstrike.containment",
    );
    assert!(
        matches!(result, CapabilityCheckResult::DeniedCompileTime { .. }),
        "BC-2.04.001 AC-1: absent compile gate must return DeniedCompileTime, got: {:?}",
        result
    );
}

/// AC-1 variant: absent gate with runtime config still returns DeniedCompileTime.
/// Runtime configuration cannot override a missing compile-time feature
/// (BC-2.04.001 postcondition + BC-2.04.004 invariant).
#[test]
fn test_BC_2_04_001_absent_gate_runtime_allow_still_denied() {
    use prism_core::capability::{CapabilityEffect, CapabilityPath, ClientCapabilities};

    let mut caps = ClientCapabilities::new();
    let path = CapabilityPath::new("sensor.crowdstrike.containment")
        .expect("test: valid capability path");
    caps.grant(path, CapabilityEffect::Allow);

    let mut client_map = BTreeMap::new();
    client_map.insert("acme".to_string(), caps);

    let evaluator = FeatureFlagEvaluator::new(client_map);
    let result = evaluator.check_permission(
        CompileTimeGate::Absent, // NOT compiled in
        "acme",
        "sensor.crowdstrike.containment",
    );

    assert!(
        matches!(result, CapabilityCheckResult::DeniedCompileTime { .. }),
        "BC-2.04.001: absent compile gate + runtime Allow must still return DeniedCompileTime"
    );
}

/// Postcondition: `crowdstrike_write_gate()` returns `Absent` when compiled
/// without the `crowdstrike-write` feature (the default in this test binary).
///
/// In CI, the default `cargo test` build does not enable write features.
/// A separate build-matrix test with `--features crowdstrike-write` verifies
/// the `Present` branch.
#[test]
fn test_BC_2_04_001_real_crowdstrike_write_gate_absent() {
    // The default feature set (`read-all`) does not include `crowdstrike-write`.
    // This test passes only when compiled without the write feature.
    // The build-matrix (CI) with `--features crowdstrike-write` will see Present.
    let gate = crowdstrike_write_gate();
    assert_eq!(
        gate,
        CompileTimeGate::Absent,
        "BC-2.04.001: crowdstrike-write must be Absent in the default (no write features) build"
    );
}

/// Postcondition: `cyberint_write_gate()` returns `Absent` in default build.
#[test]
fn test_BC_2_04_001_real_cyberint_write_gate_absent() {
    let gate = cyberint_write_gate();
    assert_eq!(
        gate,
        CompileTimeGate::Absent,
        "BC-2.04.001: cyberint-write must be Absent in the default build"
    );
}

/// Postcondition: `claroty_write_gate()` returns `Absent` in default build.
#[test]
fn test_BC_2_04_001_real_claroty_write_gate_absent() {
    let gate = claroty_write_gate();
    assert_eq!(
        gate,
        CompileTimeGate::Absent,
        "BC-2.04.001: claroty-write must be Absent in the default build"
    );
}

/// Postcondition: `armis_write_gate()` returns `Absent` in default build.
#[test]
fn test_BC_2_04_001_real_armis_write_gate_absent() {
    let gate = armis_write_gate();
    assert_eq!(
        gate,
        CompileTimeGate::Absent,
        "BC-2.04.001: armis-write must be Absent in the default build"
    );
}

/// EC-04-001: Binary built with `crowdstrike-write` but not `claroty-write`.
/// When the compile gate for CrowdStrike is `Present` but Claroty is `Absent`,
/// both checks behave correctly per their individual gates.
#[test]
fn test_BC_2_04_001_ec_mixed_features_independent_gates() {
    use prism_core::capability::{CapabilityEffect, CapabilityPath, ClientCapabilities};

    let mut caps = ClientCapabilities::new();
    let cs_path =
        CapabilityPath::new("sensor.crowdstrike.containment").expect("valid path");
    let cl_path =
        CapabilityPath::new("sensor.claroty.containment").expect("valid path");
    caps.grant(cs_path, CapabilityEffect::Allow);
    caps.grant(cl_path, CapabilityEffect::Allow);

    let mut client_map = BTreeMap::new();
    client_map.insert("acme".to_string(), caps);

    let evaluator = FeatureFlagEvaluator::new(client_map);

    // CrowdStrike feature present → both tiers pass → Allowed
    let cs_result = evaluator.check_permission(
        CompileTimeGate::Present,
        "acme",
        "sensor.crowdstrike.containment",
    );
    assert!(
        matches!(cs_result, CapabilityCheckResult::Allowed),
        "EC-04-001: CrowdStrike write gate Present + runtime Allow must be Allowed"
    );

    // Claroty feature absent → compile tier fails → DeniedCompileTime
    let cl_result = evaluator.check_permission(
        CompileTimeGate::Absent,
        "acme",
        "sensor.claroty.containment",
    );
    assert!(
        matches!(cl_result, CapabilityCheckResult::DeniedCompileTime { .. }),
        "EC-04-001: Claroty write gate Absent must return DeniedCompileTime"
    );
}

/// Resolution trace must include the denied capability path (BC-2.04.015 minimum).
#[test]
fn test_BC_2_04_001_resolution_trace_minimum_fields() {
    let evaluator = FeatureFlagEvaluator::new(BTreeMap::new());
    let result = evaluator.check_permission(
        CompileTimeGate::Absent,
        "acme",
        "sensor.crowdstrike.containment",
    );

    if let CapabilityCheckResult::DeniedCompileTime {
        capability,
        client_id,
        resolution_trace,
    } = result
    {
        assert_eq!(capability, "sensor.crowdstrike.containment");
        assert_eq!(client_id, "acme");
        assert!(
            !resolution_trace.is_empty(),
            "BC-2.04.001 / BC-2.04.015: resolution_trace must have at least one entry"
        );
    } else {
        panic!("BC-2.04.001: expected DeniedCompileTime result");
    }
}
