// S-1.08: BC-2.04.004 — Two-Tier Gate — Both Compile-Time and Runtime Must Permit
//
// Tests verify the 2×2 truth table for the two-tier gate (VP-020):
//  - (Absent, Allow)  → DeniedCompileTime
//  - (Absent, Deny)   → DeniedCompileTime
//  - (Present, Deny)  → DeniedRuntime
//  - (Present, Allow) → Allowed
//
// Also tests:
//  - EC-04-008: tool availability is per-invocation client_id, not session-level.
//  - EC-04-009: all write features compiled in but all runtime flags deny → read-only effective.
//
// Naming: test_BC_2_04_004_<assertion>
#![allow(non_snake_case)]

use std::collections::BTreeMap;

use prism_core::capability::{CapabilityEffect, CapabilityPath, ClientCapabilities};
use prism_security::feature_flag::{CapabilityCheckResult, CompileTimeGate, FeatureFlagEvaluator};

fn cap(s: &str) -> CapabilityPath {
    CapabilityPath::new(s).expect("test helper: valid capability path")
}

fn make_evaluator(
    client_id: &str,
    rules: Vec<(&str, CapabilityEffect)>,
) -> FeatureFlagEvaluator {
    let mut caps = ClientCapabilities::new();
    for (path, effect) in rules {
        caps.grant(cap(path), effect);
    }
    let mut map = BTreeMap::new();
    map.insert(client_id.to_string(), caps);
    FeatureFlagEvaluator::new(map)
}

// ─────────────────────────────────────────────────────────────
// 2×2 truth table (VP-020 core assertion)
// ─────────────────────────────────────────────────────────────

/// Canonical test vector 2: Compile absent, runtime N/A → DeniedCompileTime.
#[test]
fn test_BC_2_04_004_absent_compile_any_runtime_is_denied_compile_time() {
    // Both runtime-allow and runtime-deny variants should both produce
    // DeniedCompileTime when compile gate is Absent.

    for (rules, label) in [
        (
            vec![("sensor.crowdstrike.containment", CapabilityEffect::Allow)],
            "runtime-Allow",
        ),
        (
            vec![("sensor.crowdstrike.containment", CapabilityEffect::Deny)],
            "runtime-Deny",
        ),
        (vec![], "runtime-empty"),
    ] {
        let evaluator = make_evaluator("acme", rules);
        let result = evaluator.check_permission(
            CompileTimeGate::Absent,
            "acme",
            "sensor.crowdstrike.containment",
        );
        assert!(
            matches!(result, CapabilityCheckResult::DeniedCompileTime { .. }),
            "BC-2.04.004 truth table ({label}): Absent compile must → DeniedCompileTime"
        );
    }
}

/// Canonical test vector 3: Compile present, runtime deny → DeniedRuntime.
#[test]
fn test_BC_2_04_004_present_compile_runtime_deny_is_denied_runtime() {
    // Runtime config has no entry for the path → deny-by-default.
    let evaluator = make_evaluator("acme", vec![]);

    let result = evaluator.check_permission(
        CompileTimeGate::Present,
        "acme",
        "sensor.crowdstrike.containment",
    );

    assert!(
        matches!(result, CapabilityCheckResult::DeniedRuntime { .. }),
        "BC-2.04.004 canonical vector 3: compile Present + runtime Deny → DeniedRuntime"
    );
}

/// Canonical test vector 1: Both gates pass → Allowed.
#[test]
fn test_BC_2_04_004_both_gates_pass_returns_allowed() {
    let evaluator = make_evaluator(
        "acme",
        vec![("sensor.crowdstrike.containment", CapabilityEffect::Allow)],
    );

    let result = evaluator.check_permission(
        CompileTimeGate::Present,
        "acme",
        "sensor.crowdstrike.containment",
    );

    assert!(
        matches!(result, CapabilityCheckResult::Allowed),
        "BC-2.04.004: both gates passing must return Allowed"
    );
}

/// Invariant: Neither gate alone is sufficient.
/// Specifically verify that compile-Present + runtime-Deny ≠ Allowed.
#[test]
fn test_BC_2_04_004_compile_alone_is_insufficient() {
    let evaluator = make_evaluator("acme", vec![]); // no runtime caps

    let result = evaluator.check_permission(
        CompileTimeGate::Present,
        "acme",
        "sensor.crowdstrike.containment",
    );

    assert!(
        !matches!(result, CapabilityCheckResult::Allowed),
        "BC-2.04.004 invariant: compile-time Present alone must NOT grant Allowed"
    );
}

/// Invariant: runtime-Allow alone (compile Absent) is also insufficient.
#[test]
fn test_BC_2_04_004_runtime_alone_is_insufficient() {
    let evaluator = make_evaluator(
        "acme",
        vec![("sensor.crowdstrike.containment", CapabilityEffect::Allow)],
    );

    let result = evaluator.check_permission(
        CompileTimeGate::Absent, // feature NOT compiled in
        "acme",
        "sensor.crowdstrike.containment",
    );

    assert!(
        !matches!(result, CapabilityCheckResult::Allowed),
        "BC-2.04.004 invariant: runtime Allow alone (compile Absent) must NOT grant Allowed"
    );
}

// ─────────────────────────────────────────────────────────────
// EC-04-008: tool availability is per-invocation client_id
// ─────────────────────────────────────────────────────────────

/// EC-04-008: Same evaluator with two clients — Client A can contain hosts,
/// Client B cannot. Invocation with client_id "a" sees it available;
/// "b" does not.
#[test]
fn test_BC_2_04_004_ec_per_invocation_client_id_determines_capability() {
    let mut map = BTreeMap::new();

    // "client_a": can contain CrowdStrike hosts.
    let mut caps_a = ClientCapabilities::new();
    caps_a.grant(
        cap("sensor.crowdstrike.containment"),
        CapabilityEffect::Allow,
    );
    map.insert("client_a".to_string(), caps_a);

    // "client_b": no write capabilities.
    let caps_b = ClientCapabilities::new();
    map.insert("client_b".to_string(), caps_b);

    let evaluator = FeatureFlagEvaluator::new(map);

    // client_a: both gates pass → Allowed.
    let result_a = evaluator.check_permission(
        CompileTimeGate::Present,
        "client_a",
        "sensor.crowdstrike.containment",
    );
    assert!(
        matches!(result_a, CapabilityCheckResult::Allowed),
        "EC-04-008: client_a with Allow must get Allowed"
    );

    // client_b: compile present but runtime deny → DeniedRuntime.
    let result_b = evaluator.check_permission(
        CompileTimeGate::Present,
        "client_b",
        "sensor.crowdstrike.containment",
    );
    assert!(
        matches!(result_b, CapabilityCheckResult::DeniedRuntime { .. }),
        "EC-04-008: client_b without capability must get DeniedRuntime"
    );
}

// ─────────────────────────────────────────────────────────────
// EC-04-009: all write features compiled in but all runtime flags deny
// ─────────────────────────────────────────────────────────────

/// EC-04-009: Binary has write code (compile Present) but all runtime configs
/// deny. Effectively read-only deployment with latent write capability.
#[test]
fn test_BC_2_04_004_ec_all_write_compiled_all_runtime_denied() {
    let clients = ["acme", "beta", "gamma"];
    let mut map = BTreeMap::new();
    for client in &clients {
        map.insert(client.to_string(), ClientCapabilities::new()); // all deny-by-default
    }
    let evaluator = FeatureFlagEvaluator::new(map);

    let write_paths = [
        "sensor.crowdstrike.containment",
        "sensor.cyberint.write",
        "sensor.claroty.write",
        "sensor.armis.write",
    ];

    for client in &clients {
        for path in &write_paths {
            let result =
                evaluator.check_permission(CompileTimeGate::Present, client, path);
            assert!(
                matches!(result, CapabilityCheckResult::DeniedRuntime { .. }),
                "EC-04-009: compile Present + runtime empty must → DeniedRuntime for {}/{}",
                client,
                path
            );
        }
    }
}

// ─────────────────────────────────────────────────────────────
// Denial reason content (BC-2.04.004 postconditions)
// ─────────────────────────────────────────────────────────────

/// Both tiers produce a clear reason when they block (BC-2.04.004 postcondition).
#[test]
fn test_BC_2_04_004_denied_results_carry_capability_and_client_id() {
    let evaluator = make_evaluator("acme", vec![]);

    // DeniedCompileTime carries capability and client_id.
    let ct_result = evaluator.check_permission(
        CompileTimeGate::Absent,
        "acme",
        "sensor.crowdstrike.containment",
    );
    if let CapabilityCheckResult::DeniedCompileTime {
        capability,
        client_id,
        ..
    } = ct_result
    {
        assert_eq!(capability, "sensor.crowdstrike.containment");
        assert_eq!(client_id, "acme");
    } else {
        panic!("BC-2.04.004: expected DeniedCompileTime");
    }

    // DeniedRuntime also carries capability and client_id.
    let rt_result = evaluator.check_permission(
        CompileTimeGate::Present,
        "acme",
        "sensor.crowdstrike.containment",
    );
    if let CapabilityCheckResult::DeniedRuntime {
        capability,
        client_id,
        ..
    } = rt_result
    {
        assert_eq!(capability, "sensor.crowdstrike.containment");
        assert_eq!(client_id, "acme");
    } else {
        panic!("BC-2.04.004: expected DeniedRuntime");
    }
}
