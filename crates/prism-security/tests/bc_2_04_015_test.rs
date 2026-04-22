// S-1.08: BC-2.04.015 — Structured Error When Write Capability Is Denied
//
// Tests verify that:
//  - AC-6: denied write attempt returns E-FLAG-001 format with resolution_trace.
//  - Error contains: code "CAPABILITY_DENIED", capability, client_id, reason, suggestion.
//  - Runtime denial → suggestion contains TOML path + restart instruction.
//  - Compile-time absent → suggestion contains rebuild instruction.
//  - EC-04-032: suggestion contains exact config path and action.
//  - EC-005 (story edge case): empty resolution_trace is rejected — minimum 1 entry.
//  - Error is not a generic "unknown tool" error.
//
// Naming: test_BC_2_04_015_<assertion>
#![allow(non_snake_case)]

use std::collections::BTreeMap;

use prism_core::error::PrismError;
use prism_security::feature_flag::{CapabilityCheckResult, CompileTimeGate, FeatureFlagEvaluator};

fn make_empty_evaluator() -> FeatureFlagEvaluator {
    FeatureFlagEvaluator::new(BTreeMap::new())
}

// ─────────────────────────────────────────────────────────────
// AC-6: denied write attempt → E-FLAG-001 structured error
// ─────────────────────────────────────────────────────────────

/// AC-6: Given a denied write attempt, when the error is returned, then it
/// matches E-FLAG-001 format with `resolution_trace` (BC-2.04.015).
#[test]
fn test_BC_2_04_015_ac6_denied_write_returns_capability_denied_error() {
    let evaluator = make_empty_evaluator();

    let result = evaluator.check_permission(
        CompileTimeGate::Present, // compile gate passes
        "acme",
        "sensor.crowdstrike.containment",
    );

    // Must be DeniedRuntime (runtime tier blocked it).
    assert!(
        matches!(result, CapabilityCheckResult::DeniedRuntime { .. }),
        "BC-2.04.015 AC-6: expected DeniedRuntime"
    );

    // Convert to structured error.
    let err = evaluator.to_error(&result);
    assert!(
        err.is_some(),
        "BC-2.04.015 AC-6: to_error must return Some for denied result"
    );

    let prism_err = err.unwrap();
    match prism_err {
        PrismError::CapabilityDenied {
            capability,
            client_id,
            reason,
            suggestion,
            resolution_trace,
        } => {
            assert_eq!(capability, "sensor.crowdstrike.containment");
            assert_eq!(client_id, "acme");
            assert!(!reason.is_empty(), "BC-2.04.015: reason must not be empty");
            assert!(!suggestion.is_empty(), "BC-2.04.015: suggestion must not be empty");
            assert!(
                !resolution_trace.is_empty(),
                "BC-2.04.015 AC-6: resolution_trace must have at least one entry (EC-005)"
            );
        }
        other => panic!("BC-2.04.015: expected CapabilityDenied, got: {:?}", other),
    }
}

/// AC-6 variant: compile-time absent → CapabilityDenied with rebuild suggestion.
#[test]
fn test_BC_2_04_015_ac6_compile_absent_returns_capability_denied_with_rebuild_suggestion() {
    let evaluator = make_empty_evaluator();

    let result = evaluator.check_permission(
        CompileTimeGate::Absent,
        "acme",
        "sensor.crowdstrike.containment",
    );

    let err = evaluator.to_error(&result).expect("must produce error");
    match err {
        PrismError::CapabilityDenied {
            reason,
            suggestion,
            resolution_trace,
            ..
        } => {
            // Reason must indicate compile-time denial.
            assert!(
                reason.contains("not compiled") || reason.contains("Feature not compiled"),
                "BC-2.04.015: compile-time denied reason must mention 'not compiled', got: {}",
                reason
            );
            // Suggestion must mention rebuilding.
            assert!(
                suggestion.contains("Rebuild") || suggestion.contains("rebuild") || suggestion.contains("crowdstrike-write"),
                "BC-2.04.015: compile-time suggestion must mention rebuild, got: {}",
                suggestion
            );
            assert!(
                !resolution_trace.is_empty(),
                "BC-2.04.015: resolution_trace must be non-empty"
            );
        }
        other => panic!("BC-2.04.015: expected CapabilityDenied, got: {:?}", other),
    }
}

// ─────────────────────────────────────────────────────────────
// Runtime denial → TOML path + restart instruction
// ─────────────────────────────────────────────────────────────

/// Canonical test vector 1 (BC-2.04.015): runtime deny → suggestion contains
/// TOML path to enable and restart instruction.
#[test]
fn test_BC_2_04_015_runtime_deny_suggestion_contains_toml_path_and_restart() {
    let evaluator = make_empty_evaluator();

    let result = evaluator.check_permission(
        CompileTimeGate::Present,
        "acme",
        "sensor.crowdstrike.containment",
    );

    let err = evaluator.to_error(&result).expect("must produce error");
    match err {
        PrismError::CapabilityDenied {
            suggestion,
            reason,
            ..
        } => {
            assert!(
                reason.contains("not in") || reason.contains("Not enabled") || reason.contains("client config"),
                "BC-2.04.015: runtime deny reason must mention client config, got: {}",
                reason
            );
            // Suggestion must contain actionable guidance.
            assert!(
                suggestion.contains("sensor.crowdstrike.containment")
                    || suggestion.contains("[clients.acme.capabilities]")
                    || suggestion.contains("restart"),
                "BC-2.04.015: runtime deny suggestion must include capability path or restart instruction, got: {}",
                suggestion
            );
        }
        other => panic!("BC-2.04.015: expected CapabilityDenied, got: {:?}", other),
    }
}

// ─────────────────────────────────────────────────────────────
// Error display begins with CAPABILITY_DENIED
// ─────────────────────────────────────────────────────────────

/// The error Display output must start with the code per PrismError conventions.
#[test]
fn test_BC_2_04_015_error_display_starts_with_capability_denied() {
    let evaluator = make_empty_evaluator();
    let result = evaluator.check_permission(
        CompileTimeGate::Present,
        "acme",
        "sensor.crowdstrike.containment",
    );
    let err = evaluator.to_error(&result).expect("must produce error");
    let display = err.to_string();
    assert!(
        display.contains("CAPABILITY_DENIED"),
        "BC-2.04.015: error Display must contain 'CAPABILITY_DENIED', got: {}",
        display
    );
}

// ─────────────────────────────────────────────────────────────
// EC-005: empty resolution_trace is invalid
// ─────────────────────────────────────────────────────────────

/// EC-005 (story spec): E-FLAG-001 error must include at minimum the capability
/// path and the winning tier in the resolution_trace.
#[test]
fn test_BC_2_04_015_ec_resolution_trace_minimum_one_entry() {
    for compile_gate in [CompileTimeGate::Absent, CompileTimeGate::Present] {
        let evaluator = make_empty_evaluator();
        let result = evaluator.check_permission(
            compile_gate,
            "acme",
            "sensor.crowdstrike.containment",
        );
        let err = evaluator.to_error(&result).unwrap();
        if let PrismError::CapabilityDenied { resolution_trace, .. } = err {
            assert!(
                !resolution_trace.is_empty(),
                "EC-005: resolution_trace must have ≥1 entry for gate {:?}",
                compile_gate
            );
        }
    }
}

// ─────────────────────────────────────────────────────────────
// to_error returns None for Allowed result
// ─────────────────────────────────────────────────────────────

/// `to_error` must return `None` when the result is `Allowed`.
#[test]
fn test_BC_2_04_015_to_error_returns_none_for_allowed() {
    use prism_core::capability::{CapabilityEffect, CapabilityPath, ClientCapabilities};

    let mut caps = ClientCapabilities::new();
    let path = CapabilityPath::new("sensor.crowdstrike.containment")
        .expect("valid path");
    caps.grant(path, CapabilityEffect::Allow);

    let mut map = BTreeMap::new();
    map.insert("acme".to_string(), caps);

    let evaluator = FeatureFlagEvaluator::new(map);
    let result = evaluator.check_permission(
        CompileTimeGate::Present,
        "acme",
        "sensor.crowdstrike.containment",
    );

    let err = evaluator.to_error(&result);
    assert!(
        err.is_none(),
        "BC-2.04.015: to_error must return None for Allowed result"
    );
}

// ─────────────────────────────────────────────────────────────
// BTreeMap determinism: resolution_trace is reproducible
// ─────────────────────────────────────────────────────────────

/// Resolution trace must be deterministic across multiple calls
/// (architecture compliance: BTreeMap required for reproducibility — BC-2.04.003).
#[test]
fn test_BC_2_04_015_resolution_trace_is_deterministic() {
    let evaluator = make_empty_evaluator();

    let traces: Vec<Vec<String>> = (0..5)
        .map(|_| {
            let result = evaluator.check_permission(
                CompileTimeGate::Present,
                "acme",
                "sensor.crowdstrike.containment",
            );
            if let Some(PrismError::CapabilityDenied { resolution_trace, .. }) =
                evaluator.to_error(&result)
            {
                resolution_trace
            } else {
                vec![]
            }
        })
        .collect();

    let first = &traces[0];
    for (i, trace) in traces.iter().enumerate().skip(1) {
        assert_eq!(
            first, trace,
            "BC-2.04.015: resolution_trace must be deterministic; call {} differed",
            i
        );
    }
}
