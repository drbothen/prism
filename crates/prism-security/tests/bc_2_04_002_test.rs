// S-1.08: BC-2.04.002 — Runtime Per-Client TOML Feature Flag Configuration
//
// Tests verify that:
//  - Global defaults apply to all clients when no per-client override exists.
//  - Per-client overrides merge with and override global defaults.
//  - Deny-by-default applies when no flag explicitly enables a capability (AC-2).
//  - Capabilities resolved at config load time are used for every check.
//  - EC-04-003: Client with no capabilities section inherits defaults.
//  - EC-04-004: Empty defaults → all capabilities denied unless explicitly per-client.
//
// Naming: test_BC_2_04_002_<assertion>
#![allow(non_snake_case)]

use std::collections::BTreeMap;

use prism_core::capability::{CapabilityEffect, CapabilityPath, ClientCapabilities};
use prism_security::feature_flag::{CapabilityCheckResult, CompileTimeGate, FeatureFlagEvaluator};

fn cap(s: &str) -> CapabilityPath {
    CapabilityPath::new(s).expect("test helper: valid capability path")
}

fn evaluator_with_caps(
    client_caps: Vec<(&str, Vec<(&str, CapabilityEffect)>)>,
) -> FeatureFlagEvaluator {
    let mut client_map = BTreeMap::new();
    for (client_id, rules) in client_caps {
        let mut caps = ClientCapabilities::new();
        for (path, effect) in rules {
            caps.grant(cap(path), effect);
        }
        client_map.insert(client_id.to_string(), caps);
    }
    FeatureFlagEvaluator::new(client_map)
}

// ─────────────────────────────────────────────────────────────
// AC-2: runtime config grants Allow → check_permission returns Allowed
// ─────────────────────────────────────────────────────────────

/// AC-2: Given runtime config `{"crowdstrike.hosts.read" → Allow}`, when
/// `check_permission` is called with compile gate `Present` for that path,
/// then it returns `Allowed`.
#[test]
fn test_BC_2_04_002_runtime_allow_returns_allowed() {
    let evaluator = evaluator_with_caps(vec![(
        "acme",
        vec![("sensor.crowdstrike.read", CapabilityEffect::Allow)],
    )]);

    let result = evaluator.check_permission(
        CompileTimeGate::Present,
        "acme",
        "sensor.crowdstrike.read",
    );

    assert!(
        matches!(result, CapabilityCheckResult::Allowed),
        "BC-2.04.002 AC-2: runtime Allow + compile Present must return Allowed, got: {:?}",
        result
    );
}

/// Postcondition: global default `Allow` for sensor path applies to all clients
/// when no per-client override exists (BC-2.04.002 postcondition 1).
#[test]
fn test_BC_2_04_002_global_default_applies_to_all_clients() {
    // Two clients: "acme" and "beta" both inherit the global default.
    // Global default is modeled as a pre-merged `ClientCapabilities` per client.
    // (The config loader merges defaults + per-client overrides at load time.)
    let global_default_caps = vec![("sensor.crowdstrike.read", CapabilityEffect::Allow)];

    let evaluator = evaluator_with_caps(vec![
        ("acme", global_default_caps.clone()),
        ("beta", global_default_caps),
    ]);

    for client_id in ["acme", "beta"] {
        let result = evaluator.check_permission(
            CompileTimeGate::Present,
            client_id,
            "sensor.crowdstrike.read",
        );
        assert!(
            matches!(result, CapabilityCheckResult::Allowed),
            "BC-2.04.002: global default Allow must apply to all clients; failed for '{}'",
            client_id
        );
    }
}

/// Postcondition: per-client override denies a capability that global default allows.
/// Models `[clients.acme.capabilities]` overriding `[defaults.capabilities]`.
#[test]
fn test_BC_2_04_002_per_client_override_denies_default_allow() {
    // "acme" has a specific Deny for containment despite a broader Allow.
    // Models: defaults allow sensor.crowdstrike, acme denies containment.
    let evaluator = evaluator_with_caps(vec![(
        "acme",
        vec![
            ("sensor.crowdstrike", CapabilityEffect::Allow), // from defaults
            ("sensor.crowdstrike.containment", CapabilityEffect::Deny), // per-client override
        ],
    )]);

    let result = evaluator.check_permission(
        CompileTimeGate::Present,
        "acme",
        "sensor.crowdstrike.containment",
    );

    assert!(
        matches!(result, CapabilityCheckResult::DeniedRuntime { .. }),
        "BC-2.04.002: per-client Deny must override default Allow, got: {:?}",
        result
    );
}

/// Postcondition: per-client override allows a capability that global default denies.
#[test]
fn test_BC_2_04_002_per_client_override_allows_default_deny() {
    // Global default: deny writes. "acme" per-client override: allow containment.
    let evaluator = evaluator_with_caps(vec![(
        "acme",
        vec![("sensor.crowdstrike.containment", CapabilityEffect::Allow)],
    )]);

    let result = evaluator.check_permission(
        CompileTimeGate::Present,
        "acme",
        "sensor.crowdstrike.containment",
    );

    assert!(
        matches!(result, CapabilityCheckResult::Allowed),
        "BC-2.04.002: per-client Allow must override default Deny, got: {:?}",
        result
    );
}

/// EC-04-003: Client with no capabilities section inherits all defaults.
/// When no per-client entry exists and defaults are empty → deny-by-default.
#[test]
fn test_BC_2_04_002_ec_client_no_caps_inherits_empty_defaults() {
    // "acme" exists but has zero capability rules (empty defaults inherited).
    let evaluator = evaluator_with_caps(vec![("acme", vec![])]);

    let result = evaluator.check_permission(
        CompileTimeGate::Present,
        "acme",
        "sensor.crowdstrike.read",
    );

    assert!(
        matches!(result, CapabilityCheckResult::DeniedRuntime { .. }),
        "EC-04-003: client with no caps inherits empty defaults → DeniedRuntime"
    );
}

/// EC-04-004: Empty defaults + no per-client entries → all clients denied.
#[test]
fn test_BC_2_04_002_ec_empty_defaults_all_denied() {
    // "acme" and "beta" both have zero capability rules.
    let evaluator = evaluator_with_caps(vec![("acme", vec![]), ("beta", vec![])]);

    for (client_id, path) in [
        ("acme", "sensor.crowdstrike.read"),
        ("beta", "sensor.crowdstrike.read"),
        ("acme", "sensor.crowdstrike.containment"),
    ] {
        let result =
            evaluator.check_permission(CompileTimeGate::Present, client_id, path);
        assert!(
            matches!(result, CapabilityCheckResult::DeniedRuntime { .. }),
            "EC-04-004: empty defaults must deny all; failed for client '{}' path '{}'",
            client_id,
            path
        );
    }
}

/// Invariant: unknown client_id (not in config) → DeniedRuntime (not a panic).
/// BC-2.04.006 edge case EC-04-004: missing client returns empty capability set.
#[test]
fn test_BC_2_04_002_unknown_client_returns_denied_not_panic() {
    let evaluator = evaluator_with_caps(vec![(
        "acme",
        vec![("sensor.crowdstrike.read", CapabilityEffect::Allow)],
    )]);

    // "unknown-client" is not in the config.
    let result = evaluator.check_permission(
        CompileTimeGate::Present,
        "unknown-client",
        "sensor.crowdstrike.read",
    );

    assert!(
        matches!(result, CapabilityCheckResult::DeniedRuntime { .. }),
        "BC-2.04.002: unknown client must return DeniedRuntime, not panic, got: {:?}",
        result
    );
}
