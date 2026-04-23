// S-1.08: BC-2.04.003 — Hierarchical Capability Resolution
// (BTreeMap, Most-Specific-Path Wins, Deny Support)
//
// Tests verify that:
//  - Deny-by-default when BTreeMap is empty.
//  - Parent Allow covers child path (hierarchy walk).
//  - More-specific Deny overrides less-specific Allow.
//  - More-specific Allow overrides less-specific Deny.
//  - 4+ level hierarchy walk works correctly.
//  - Resolution is deterministic (same inputs → same result).
//  - EC-04-005, EC-04-006, EC-04-007, EC-04-032.
//  - AC-8: {sensor.crowdstrike → Allow, sensor.crowdstrike.containment → Deny} → Deny.
//
// Naming: test_BC_2_04_003_<assertion>
#![allow(non_snake_case)]

use std::collections::BTreeMap;

use prism_core::capability::{CapabilityEffect, CapabilityPath, ClientCapabilities};
use prism_security::feature_flag::{CapabilityCheckResult, CompileTimeGate, FeatureFlagEvaluator};

fn cap(s: &str) -> CapabilityPath {
    CapabilityPath::new(s).expect("test helper: valid capability path")
}

fn evaluator_with(rules: Vec<(&str, CapabilityEffect)>) -> FeatureFlagEvaluator {
    let mut caps = ClientCapabilities::new();
    for (path, effect) in rules {
        caps.grant(cap(path), effect);
    }
    let mut client_map = BTreeMap::new();
    client_map.insert("acme".to_string(), caps);
    FeatureFlagEvaluator::new(client_map)
}

fn check(evaluator: &FeatureFlagEvaluator, path: &str) -> CapabilityCheckResult {
    evaluator.check_permission(CompileTimeGate::Present, "acme", path)
}

// ─────────────────────────────────────────────────────────────
// Deny-by-default (BC-2.04.003 canonical test vector 1)
// ─────────────────────────────────────────────────────────────

#[test]
fn test_BC_2_04_003_deny_by_default_empty_btreemap() {
    // Scenario: Deny-by-default — empty BTreeMap + path `sensor.crowdstrike.containment` → Deny.
    let evaluator = evaluator_with(vec![]);
    let result = check(&evaluator, "sensor.crowdstrike.containment");
    assert!(
        matches!(result, CapabilityCheckResult::DeniedRuntime { .. }),
        "BC-2.04.003: empty BTreeMap must deny any path, got: {:?}",
        result
    );
}

// ─────────────────────────────────────────────────────────────
// Parent allow, child absent (canonical test vector 2)
// ─────────────────────────────────────────────────────────────

#[test]
fn test_BC_2_04_003_parent_allow_covers_child_path() {
    // Scenario: {sensor.crowdstrike: Allow} + check `sensor.crowdstrike.containment` → Allow.
    let evaluator = evaluator_with(vec![("sensor.crowdstrike", CapabilityEffect::Allow)]);
    let result = check(&evaluator, "sensor.crowdstrike.containment");
    assert!(
        matches!(result, CapabilityCheckResult::Allowed),
        "BC-2.04.003: parent Allow must cover child path (hierarchy walk)"
    );
}

// ─────────────────────────────────────────────────────────────
// Parent allow, child deny (canonical test vector 3) — AC-8
// ─────────────────────────────────────────────────────────────

/// AC-8: Given `defaults.capabilities = {sensor.crowdstrike → Allow}` AND
/// `clients.acme.capabilities = {sensor.crowdstrike.containment → Deny}`,
/// when `check_permission("acme", "sensor.crowdstrike.containment")` is called,
/// then it returns `Deny` — most-specific path wins.
#[test]
fn test_BC_2_04_003_ac8_most_specific_deny_overrides_parent_allow() {
    let evaluator = evaluator_with(vec![
        ("sensor.crowdstrike", CapabilityEffect::Allow),
        ("sensor.crowdstrike.containment", CapabilityEffect::Deny),
    ]);
    let result = check(&evaluator, "sensor.crowdstrike.containment");
    assert!(
        matches!(result, CapabilityCheckResult::DeniedRuntime { .. }),
        "BC-2.04.003 AC-8: most-specific Deny must override parent Allow, got: {:?}",
        result
    );
}

// ─────────────────────────────────────────────────────────────
// Exact match wins (canonical test vector 4)
// ─────────────────────────────────────────────────────────────

/// EC-04-032: {sensor.crowdstrike: Deny, sensor.crowdstrike.read: Allow}
/// → check `sensor.crowdstrike.read` → Allow (more-specific wins).
#[test]
fn test_BC_2_04_003_exact_match_allow_overrides_parent_deny() {
    let evaluator = evaluator_with(vec![
        ("sensor.crowdstrike", CapabilityEffect::Deny),
        ("sensor.crowdstrike.read", CapabilityEffect::Allow),
    ]);
    let result = check(&evaluator, "sensor.crowdstrike.read");
    assert!(
        matches!(result, CapabilityCheckResult::Allowed),
        "BC-2.04.003 EC-04-032: exact-match Allow must win over parent Deny"
    );
}

// ─────────────────────────────────────────────────────────────
// EC-04-005: Parent allow, child deny → Deny wins
// ─────────────────────────────────────────────────────────────

#[test]
fn test_BC_2_04_003_ec_parent_allow_child_deny() {
    // EC-04-005: `sensor.crowdstrike` Allow, `sensor.crowdstrike.containment` Deny.
    let evaluator = evaluator_with(vec![
        ("sensor.crowdstrike", CapabilityEffect::Allow),
        ("sensor.crowdstrike.containment", CapabilityEffect::Deny),
    ]);
    let result = check(&evaluator, "sensor.crowdstrike.containment");
    assert!(
        matches!(result, CapabilityCheckResult::DeniedRuntime { .. }),
        "EC-04-005: more-specific Deny must win over parent Allow"
    );

    // The parent path (without the Deny override) should still be Allow.
    let parent_result = check(&evaluator, "sensor.crowdstrike.read");
    assert!(
        matches!(parent_result, CapabilityCheckResult::Allowed),
        "EC-04-005: sibling path under parent Allow should still be Allowed"
    );
}

// ─────────────────────────────────────────────────────────────
// EC-04-006: Broad grant via single parent `sensor: Allow`
// ─────────────────────────────────────────────────────────────

#[test]
fn test_BC_2_04_003_ec_broad_parent_allow_covers_all_descendants() {
    // EC-04-006: only `sensor` is Allow → all sensor.* paths match via hierarchy walk.
    let evaluator = evaluator_with(vec![("sensor", CapabilityEffect::Allow)]);

    for path in [
        "sensor.crowdstrike",
        "sensor.crowdstrike.read",
        "sensor.crowdstrike.containment",
        "sensor.claroty.read",
        "sensor.armis.read",
    ] {
        let result = check(&evaluator, path);
        assert!(
            matches!(result, CapabilityCheckResult::Allowed),
            "EC-04-006: broad parent Allow must cover all descendants; failed for '{}'",
            path
        );
    }
}

// ─────────────────────────────────────────────────────────────
// EC-04-007: 4+ level hierarchy walk
// ─────────────────────────────────────────────────────────────

#[test]
fn test_BC_2_04_003_ec_four_level_hierarchy_walk() {
    // EC-04-007: `sensor.crowdstrike.rtr.execute` → walk: exact, sensor.crowdstrike.rtr,
    // sensor.crowdstrike, sensor. Most-specific match wins.
    let evaluator = evaluator_with(vec![("sensor.crowdstrike", CapabilityEffect::Allow)]);
    let result = check(&evaluator, "sensor.crowdstrike.rtr.execute");
    assert!(
        matches!(result, CapabilityCheckResult::Allowed),
        "EC-04-007: 4-level path must inherit from grandparent Allow via hierarchy walk"
    );
}

#[test]
fn test_BC_2_04_003_ec_four_level_specific_deny_overrides_grandparent_allow() {
    // EC-04-007 variant: Allow at `sensor.crowdstrike`, Deny at `sensor.crowdstrike.rtr`.
    // `sensor.crowdstrike.rtr.execute` → `sensor.crowdstrike.rtr` matches first → Deny.
    let evaluator = evaluator_with(vec![
        ("sensor.crowdstrike", CapabilityEffect::Allow),
        ("sensor.crowdstrike.rtr", CapabilityEffect::Deny),
    ]);
    let result = check(&evaluator, "sensor.crowdstrike.rtr.execute");
    assert!(
        matches!(result, CapabilityCheckResult::DeniedRuntime { .. }),
        "EC-04-007: deny at intermediate level must override grandparent Allow"
    );
}

// ─────────────────────────────────────────────────────────────
// Determinism invariant: same inputs → same result
// ─────────────────────────────────────────────────────────────

#[test]
fn test_BC_2_04_003_invariant_resolution_is_deterministic() {
    let evaluator = evaluator_with(vec![
        ("sensor.crowdstrike", CapabilityEffect::Allow),
        ("sensor.crowdstrike.containment", CapabilityEffect::Deny),
    ]);

    // Call the same check 10 times — must always return the same result.
    for i in 0..10 {
        let result = check(&evaluator, "sensor.crowdstrike.containment");
        assert!(
            matches!(result, CapabilityCheckResult::DeniedRuntime { .. }),
            "BC-2.04.003 determinism: call {} returned unexpected result",
            i
        );
    }
}

// ─────────────────────────────────────────────────────────────
// Empty capability path → Deny (BC-2.04.003 error case)
// ─────────────────────────────────────────────────────────────

#[test]
fn test_BC_2_04_003_empty_capability_path_returns_error_or_deny() {
    // BC-2.04.003 error case: empty capability path string → Deny (no match possible).
    // The path validation in CapabilityPath::new should reject empty strings,
    // so check_permission with "" should either return DeniedRuntime or fail
    // gracefully (not panic).
    let evaluator = evaluator_with(vec![("sensor.crowdstrike", CapabilityEffect::Allow)]);

    // Empty string is an invalid path. The evaluator must not panic.
    // It should return DeniedRuntime (deny-by-default when path is invalid).
    let result = evaluator.check_permission(CompileTimeGate::Present, "acme", "");
    assert!(
        matches!(result, CapabilityCheckResult::DeniedRuntime { .. }),
        "BC-2.04.003: empty capability path must return DeniedRuntime (no match possible)"
    );
}
