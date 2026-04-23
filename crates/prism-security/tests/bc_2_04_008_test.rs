// S-1.09: BC-2.04.008 — Dry-Run Default for Reversible Write Operations
//
// Tests verify:
//   - Default invocation (dry_run omitted) returns preview with `_meta.dry_run: true`.
//   - Explicit `dry_run: true` returns preview; no state change.
//   - Explicit `dry_run: false` executes the write (Allow gate decision).
//   - Dry-run response contains: target entity, proposed change, confirmation prompt.
//   - Dry-run response includes `dry_run: true` field (BC-2.04.008 postcondition).
//   - Dry-run mode NEVER modifies state.
//   - EC-05 (from S-1.09): Reversible operation without explicit dry_run=false returns preview.
//
// Naming: test_BC_2_04_008_<assertion>
#![allow(non_snake_case)]

use prism_security::risk_tier::{DryRunResponse, GateDecision, RiskTier};

// ─────────────────────────────────────────────────────────────
// BC-2.04.008 Postconditions
// ─────────────────────────────────────────────────────────────

/// Postcondition: Reversible tool with dry_run omitted → preview with dry_run: true.
///
/// Canonical test vector: "Default invocation" → preview; `_meta.dry_run: true`.
/// Traces to AC-5.
#[test]
fn test_BC_2_04_008_default_invocation_returns_dry_run_preview() {
    let tier = RiskTier::Reversible;
    let decision = tier.apply_gate(None);
    match decision {
        GateDecision::DryRunPreview(preview) => {
            assert!(
                preview.dry_run,
                "BC-2.04.008: dry-run preview must have dry_run: true"
            );
        }
        other => {
            panic!("BC-2.04.008: expected DryRunPreview for default invocation, got {other:?}")
        }
    }
}

/// Postcondition: Explicit `dry_run: true` → preview; dry_run field is true.
#[test]
fn test_BC_2_04_008_explicit_dry_run_true_returns_preview() {
    let tier = RiskTier::Reversible;
    let decision = tier.apply_gate(Some(true));
    assert!(
        matches!(decision, GateDecision::DryRunPreview(_)),
        "BC-2.04.008: explicit dry_run=true must return DryRunPreview"
    );
}

/// Postcondition: Explicit `dry_run: false` → Allow (execute). No preview.
///
/// Canonical test vector: "Explicit execute" → actual write; `_meta.dry_run: false`.
#[test]
fn test_BC_2_04_008_explicit_dry_run_false_returns_allow() {
    let tier = RiskTier::Reversible;
    let decision = tier.apply_gate(Some(false));
    assert!(
        matches!(decision, GateDecision::Allow),
        "BC-2.04.008: explicit dry_run=false must return Allow (execute)"
    );
}

/// Postcondition: dry-run response MUST contain target entity, proposed change,
/// and confirmation prompt (BC-2.04.008 postcondition).
///
/// EC-05: Reversible operation without dry_run=false returns dry-run preview.
#[test]
fn test_BC_2_04_008_dry_run_response_contains_required_fields() {
    let tier = RiskTier::Reversible;
    let decision = tier.apply_gate(None);
    match decision {
        GateDecision::DryRunPreview(preview) => {
            assert!(
                !preview.target_entity.is_empty(),
                "BC-2.04.008: dry-run response must contain non-empty target_entity"
            );
            assert!(
                !preview.proposed_change.is_empty(),
                "BC-2.04.008: dry-run response must contain non-empty proposed_change"
            );
            assert!(
                !preview.confirmation_prompt.is_empty(),
                "BC-2.04.008: dry-run response must contain non-empty confirmation_prompt"
            );
            assert!(
                preview.dry_run,
                "BC-2.04.008: dry-run response must have dry_run = true"
            );
        }
        other => panic!("BC-2.04.008: expected DryRunPreview, got {other:?}"),
    }
}

// ─────────────────────────────────────────────────────────────
// BC-2.04.008 Invariants
// ─────────────────────────────────────────────────────────────

/// Invariant: default is ALWAYS true — agent must explicitly opt in to execution.
///
/// Canonical test vector: "Default invocation" → dry_run: true.
#[test]
fn test_BC_2_04_008_invariant_default_is_always_dry_run_true() {
    let tier = RiskTier::Reversible;
    // Three ways to hit the default: None, Some(true), and absence in real MCP call.
    for dry_run in [None, Some(true)] {
        let decision = tier.apply_gate(dry_run);
        assert!(
            matches!(decision, GateDecision::DryRunPreview(_)),
            "BC-2.04.008 invariant: default (dry_run=None or true) must always return DryRunPreview"
        );
    }
}

/// Invariant: dry-run mode NEVER produces Allow (no state change).
///
/// If dry_run is true (or absent), the gate must not return Allow.
#[test]
fn test_BC_2_04_008_invariant_dry_run_mode_never_allow() {
    let tier = RiskTier::Reversible;
    for dry_run in [None, Some(true)] {
        let decision = tier.apply_gate(dry_run);
        assert!(
            !matches!(decision, GateDecision::Allow),
            "BC-2.04.008 invariant: dry-run mode must NEVER return Allow (no state change)"
        );
    }
}

/// Invariant: dry-run mode NEVER produces RequiresConfirmationToken
/// (only Irreversible needs a token; Reversible uses dry-run).
#[test]
fn test_BC_2_04_008_invariant_reversible_never_requires_token() {
    let tier = RiskTier::Reversible;
    for dry_run in [None, Some(true), Some(false)] {
        let decision = tier.apply_gate(dry_run);
        assert!(
            !matches!(decision, GateDecision::RequiresConfirmationToken),
            "BC-2.04.008 invariant: RiskTier::Reversible must NEVER require confirmation token"
        );
    }
}
