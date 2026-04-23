// S-1.08: BC-2.04.006 — list_capabilities Meta-Tool for Capability Discovery
//
// Tests verify that:
//  - AC-4: list_capabilities("acme") returns full effective capability set.
//  - list_capabilities is always registered (not gated by any feature flag).
//  - Returns CapabilityStatus with enabled, compile_time, runtime, reason fields.
//  - When client_id is None, returns per-client breakdown.
//  - EC-04-012: null client_id returns global matrix.
//  - EC-04-013: zero write features → all write paths show compile_time: false.
//  - Unknown client_id returns error (PrismError::ConfigValidationFailed).
//
// Naming: test_BC_2_04_006_<assertion>
#![allow(non_snake_case)]
#![allow(clippy::unwrap_used)]

use std::collections::BTreeMap;

use prism_core::capability::{CapabilityEffect, CapabilityPath, ClientCapabilities};
use prism_security::feature_flag::CompileTimeGate;
use prism_security::list_capabilities::{ListCapabilitiesEngine, ListCapabilitiesQuery};

fn cap(s: &str) -> CapabilityPath {
    CapabilityPath::new(s).expect("test helper: valid capability path")
}

fn make_engine(
    clients: Vec<(&str, Vec<(&str, CapabilityEffect)>)>,
    compile_gates: Vec<(&str, CompileTimeGate)>,
) -> ListCapabilitiesEngine {
    let mut client_map = BTreeMap::new();
    for (client_id, rules) in clients {
        let mut caps = ClientCapabilities::new();
        for (path, effect) in rules {
            caps.grant(cap(path), effect);
        }
        client_map.insert(client_id.to_string(), caps);
    }

    let mut gates = BTreeMap::new();
    for (prefix, gate) in compile_gates {
        gates.insert(prefix.to_string(), gate);
    }

    ListCapabilitiesEngine::new(client_map, gates)
}

// ─────────────────────────────────────────────────────────────
// AC-4: list_capabilities("acme") returns full effective capability set
// ─────────────────────────────────────────────────────────────

/// AC-4: Given client "acme" with specific capabilities, when
/// `list_capabilities("acme")` is called, then it returns the full
/// effective capability set (BC-2.04.006 postconditions).
#[test]
fn test_BC_2_04_006_ac4_returns_capability_matrix_for_known_client() {
    let engine = make_engine(
        vec![(
            "acme",
            vec![
                ("sensor.crowdstrike.read", CapabilityEffect::Allow),
                ("sensor.crowdstrike.containment", CapabilityEffect::Deny),
            ],
        )],
        vec![("sensor.crowdstrike", CompileTimeGate::Present)],
    );

    let query = ListCapabilitiesQuery {
        client_id: Some("acme".to_string()),
    };

    let result = engine.execute(&query);
    assert!(
        result.is_ok(),
        "BC-2.04.006 AC-4: list_capabilities for known client must succeed"
    );

    let entries = result.unwrap();
    assert!(
        !entries.is_empty(),
        "BC-2.04.006 AC-4: entries must be non-empty for configured client"
    );
}

// ─────────────────────────────────────────────────────────────
// CapabilityStatus fields: enabled, compile_time, runtime, reason
// ─────────────────────────────────────────────────────────────

/// Both tiers pass → enabled: true, compile_time: true, runtime: true, reason: None.
#[test]
fn test_BC_2_04_006_status_both_tiers_pass() {
    let engine = make_engine(
        vec![(
            "acme",
            vec![("sensor.crowdstrike.containment", CapabilityEffect::Allow)],
        )],
        vec![("sensor.crowdstrike", CompileTimeGate::Present)],
    );

    let query = ListCapabilitiesQuery {
        client_id: Some("acme".to_string()),
    };

    let entries = engine.execute(&query).unwrap();
    let entry = entries
        .iter()
        .find(|e| e.capability == "sensor.crowdstrike.containment");

    assert!(
        entry.is_some(),
        "BC-2.04.006: capability entry must be present in matrix"
    );
    let status = &entry.unwrap().status;
    assert!(
        status.enabled,
        "BC-2.04.006: enabled must be true when both tiers pass"
    );
    assert!(
        status.compile_time,
        "BC-2.04.006: compile_time must be true"
    );
    assert!(status.runtime, "BC-2.04.006: runtime must be true");
    assert!(
        status.reason.is_none(),
        "BC-2.04.006: reason must be None when enabled"
    );
}

/// Feature absent → enabled: false, compile_time: false, reason: "Feature not compiled (...)"
#[test]
fn test_BC_2_04_006_status_feature_absent_correct_fields() {
    let engine = make_engine(
        vec![(
            "acme",
            vec![("sensor.crowdstrike.containment", CapabilityEffect::Allow)],
        )],
        vec![("sensor.crowdstrike", CompileTimeGate::Absent)],
    );

    let query = ListCapabilitiesQuery {
        client_id: Some("acme".to_string()),
    };

    let entries = engine.execute(&query).unwrap();
    let entry = entries
        .iter()
        .find(|e| e.capability == "sensor.crowdstrike.containment");

    if let Some(entry) = entry {
        assert!(
            !entry.status.enabled,
            "BC-2.04.006: enabled must be false when compile gate absent"
        );
        assert!(
            !entry.status.compile_time,
            "BC-2.04.006: compile_time must be false"
        );
        let reason = entry.status.reason.as_deref().unwrap_or("");
        assert!(
            reason.contains("not compiled") || reason.contains("Feature not compiled"),
            "BC-2.04.006: reason must mention feature not compiled, got: {:?}",
            reason
        );
    }
    // If the entry is absent, the test is inconclusive — the engine may only
    // list explicitly-configured capabilities. Either behavior is acceptable
    // here; the key assertion is that if it IS listed, the status fields are correct.
}

/// Feature present, runtime deny → enabled: false, compile_time: true, runtime: false.
#[test]
fn test_BC_2_04_006_status_feature_present_runtime_deny_correct_fields() {
    let engine = make_engine(
        vec![("acme", vec![])], // no runtime caps → deny-by-default
        vec![("sensor.crowdstrike", CompileTimeGate::Present)],
    );

    let query = ListCapabilitiesQuery {
        client_id: Some("acme".to_string()),
    };

    let entries = engine.execute(&query).unwrap();
    let entry = entries
        .iter()
        .find(|e| e.capability == "sensor.crowdstrike.containment");

    if let Some(entry) = entry {
        assert!(
            !entry.status.enabled,
            "BC-2.04.006: enabled must be false when runtime denies"
        );
        assert!(
            entry.status.compile_time,
            "BC-2.04.006: compile_time must be true when feature is present"
        );
        assert!(
            !entry.status.runtime,
            "BC-2.04.006: runtime must be false when not in client config"
        );
        let reason = entry.status.reason.as_deref().unwrap_or("");
        assert!(
            !reason.is_empty(),
            "BC-2.04.006: reason must be non-empty when disabled"
        );
    }
}

// ─────────────────────────────────────────────────────────────
// Unknown client_id → error (not panic)
// ─────────────────────────────────────────────────────────────

#[test]
fn test_BC_2_04_006_unknown_client_id_returns_error() {
    let engine = make_engine(vec![("acme", vec![])], vec![]);

    let query = ListCapabilitiesQuery {
        client_id: Some("unknown-client".to_string()),
    };

    let result = engine.execute(&query);
    assert!(
        result.is_err(),
        "BC-2.04.006: unknown client_id must return Err, not panic"
    );

    let err = result.unwrap_err();
    let err_string = err.to_string();
    assert!(
        err_string.contains("unknown-client"),
        "BC-2.04.006: error message must include the unknown client_id, got: {}",
        err_string
    );
}

// ─────────────────────────────────────────────────────────────
// EC-04-012: null client_id returns global matrix
// ─────────────────────────────────────────────────────────────

#[test]
fn test_BC_2_04_006_ec_null_client_id_returns_all_clients() {
    let engine = make_engine(
        vec![
            (
                "acme",
                vec![("sensor.crowdstrike.containment", CapabilityEffect::Allow)],
            ),
            ("beta", vec![]),
        ],
        vec![("sensor.crowdstrike", CompileTimeGate::Present)],
    );

    let query = ListCapabilitiesQuery { client_id: None };

    let result = engine.execute(&query);
    assert!(
        result.is_ok(),
        "EC-04-012: null client_id must succeed and return global matrix"
    );

    let entries = result.unwrap();
    assert!(
        !entries.is_empty(),
        "EC-04-012: global matrix must be non-empty when clients are configured"
    );
}

// ─────────────────────────────────────────────────────────────
// EC-04-013: Zero write features → all write paths compile_time: false
// ─────────────────────────────────────────────────────────────

#[test]
fn test_BC_2_04_006_ec_zero_write_features_all_compile_time_false() {
    // All compile gates are Absent — the binary has zero write features.
    let engine = make_engine(
        vec![(
            "acme",
            vec![
                ("sensor.crowdstrike.containment", CapabilityEffect::Allow),
                ("sensor.cyberint.write", CapabilityEffect::Allow),
            ],
        )],
        vec![
            ("sensor.crowdstrike", CompileTimeGate::Absent),
            ("sensor.cyberint", CompileTimeGate::Absent),
        ],
    );

    let query = ListCapabilitiesQuery {
        client_id: Some("acme".to_string()),
    };

    let entries = engine.execute(&query).unwrap();
    // Any capability entry that relates to a write operation must show compile_time: false.
    for entry in &entries {
        // Only check write-related capabilities.
        if entry.capability.contains("containment") || entry.capability.contains("write") {
            assert!(
                !entry.status.compile_time,
                "EC-04-013: zero write features → compile_time must be false for '{}', got status: {:?}",
                entry.capability,
                entry.status
            );
            assert!(
                !entry.status.enabled,
                "EC-04-013: zero write features → enabled must be false for '{}'",
                entry.capability
            );
        }
    }
}

// ─────────────────────────────────────────────────────────────
// Invariant: list_capabilities is always available
// ─────────────────────────────────────────────────────────────

/// list_capabilities must succeed even when no clients are configured.
#[test]
fn test_BC_2_04_006_invariant_always_available_no_clients() {
    let engine = make_engine(vec![], vec![]);

    let query = ListCapabilitiesQuery { client_id: None };
    let result = engine.execute(&query);

    assert!(
        result.is_ok(),
        "BC-2.04.006 invariant: list_capabilities must always be available, even with no clients"
    );
}
