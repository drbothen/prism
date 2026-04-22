// S-1.09: BC-2.04.007 — Three-Tier Risk Classification for Operations
//
// Tests verify:
//   - Every tool invocation is routed through exactly one of three gate mechanisms.
//   - Read tools execute immediately (no gate, no dry-run preview, no token).
//   - Reversible Write tools default to dry-run preview when dry_run is omitted or true.
//   - Irreversible Write tools require a confirmation token — never execute on first call.
//   - Risk classification cannot change at runtime (tier is set at registration).
//   - EC-04-014: ambiguous operations are classified as Irreversible (conservative).
//   - EC-04-015: EC of unclassified tool at registration — covered by type system.
//   - EC-006 from S-1.09: Read operation going through gate check returns Allow immediately.
//
// Naming: test_BC_2_04_007_<assertion>
#![allow(non_snake_case)]

use prism_security::risk_tier::{GateDecision, RiskTier};

// ─────────────────────────────────────────────────────────────
// BC-2.04.007 Postcondition: Read tier → no gate
// ─────────────────────────────────────────────────────────────

/// Postcondition: RiskTier::Read → GateDecision::Allow (no gate, no dry-run, no token).
///
/// Traces to AC-7: Read tools execute immediately with no gate.
/// Traces to EC-006: Read operation going through confirmation gate check returns Allow.
#[test]
fn test_BC_2_04_007_read_tier_returns_allow_immediately() {
    let tier = RiskTier::Read;
    let decision = tier.apply_gate(None);
    assert!(
        matches!(decision, GateDecision::Allow),
        "BC-2.04.007: RiskTier::Read with no dry_run param must return GateDecision::Allow"
    );
}

/// Read tier with explicit dry_run = true still returns Allow (dry_run is not meaningful
/// for Read tools; the gate is a no-op).
#[test]
fn test_BC_2_04_007_read_tier_with_dry_run_true_still_allow() {
    let tier = RiskTier::Read;
    let decision = tier.apply_gate(Some(true));
    assert!(
        matches!(decision, GateDecision::Allow),
        "BC-2.04.007: RiskTier::Read must return Allow regardless of dry_run value"
    );
}

/// Read tier with explicit dry_run = false still returns Allow.
#[test]
fn test_BC_2_04_007_read_tier_with_dry_run_false_still_allow() {
    let tier = RiskTier::Read;
    let decision = tier.apply_gate(Some(false));
    assert!(
        matches!(decision, GateDecision::Allow),
        "BC-2.04.007: RiskTier::Read must return Allow even with dry_run: false"
    );
}

/// Invariant: Read tier MUST NEVER return DryRunPreview or RequiresConfirmationToken.
#[test]
fn test_BC_2_04_007_invariant_read_never_dry_run_or_token() {
    let tier = RiskTier::Read;
    for dry_run in [None, Some(true), Some(false)] {
        let decision = tier.apply_gate(dry_run);
        assert!(
            !matches!(decision, GateDecision::DryRunPreview(_)),
            "BC-2.04.007 invariant: RiskTier::Read must NEVER return DryRunPreview"
        );
        assert!(
            !matches!(decision, GateDecision::RequiresConfirmationToken),
            "BC-2.04.007 invariant: RiskTier::Read must NEVER return RequiresConfirmationToken"
        );
    }
}

// ─────────────────────────────────────────────────────────────
// BC-2.04.007 Postcondition: Reversible tier → dry-run default
// ─────────────────────────────────────────────────────────────

/// Postcondition: RiskTier::Reversible with no dry_run param defaults to dry-run preview.
///
/// AC-7: Reversible Write tools return dry-run preview when dry_run is not set to false.
#[test]
fn test_BC_2_04_007_reversible_tier_default_dry_run() {
    let tier = RiskTier::Reversible;
    let decision = tier.apply_gate(None);
    assert!(
        matches!(decision, GateDecision::DryRunPreview(_)),
        "BC-2.04.007: RiskTier::Reversible with no dry_run must default to DryRunPreview"
    );
}

/// Postcondition: RiskTier::Reversible with explicit dry_run = true returns preview.
#[test]
fn test_BC_2_04_007_reversible_tier_explicit_dry_run_true() {
    let tier = RiskTier::Reversible;
    let decision = tier.apply_gate(Some(true));
    assert!(
        matches!(decision, GateDecision::DryRunPreview(_)),
        "BC-2.04.007: RiskTier::Reversible with dry_run=true must return DryRunPreview"
    );
}

/// Postcondition: RiskTier::Reversible with explicit dry_run = false executes
/// (returns Allow — the dry-run default is bypassed per EC-04-017).
#[test]
fn test_BC_2_04_007_reversible_tier_explicit_dry_run_false_executes() {
    let tier = RiskTier::Reversible;
    let decision = tier.apply_gate(Some(false));
    assert!(
        matches!(decision, GateDecision::Allow),
        "BC-2.04.007: RiskTier::Reversible with dry_run=false must return Allow (execute)"
    );
}

// ─────────────────────────────────────────────────────────────
// BC-2.04.007 Postcondition: Irreversible tier → confirmation token
// ─────────────────────────────────────────────────────────────

/// Postcondition: RiskTier::Irreversible always returns RequiresConfirmationToken.
///
/// AC-1: irreversible operation (contain_host) without token → token returned.
/// AC-7: Irreversible Write tools return ConfirmationToken; no execution.
#[test]
fn test_BC_2_04_007_irreversible_tier_requires_confirmation_token() {
    let tier = RiskTier::Irreversible;
    let decision = tier.apply_gate(None);
    assert!(
        matches!(decision, GateDecision::RequiresConfirmationToken),
        "BC-2.04.007: RiskTier::Irreversible must always return RequiresConfirmationToken"
    );
}

/// Irreversible tier with dry_run = false still requires token (dry_run is not
/// meaningful for irreversible operations — the token gate cannot be bypassed).
#[test]
fn test_BC_2_04_007_irreversible_tier_dry_run_false_still_requires_token() {
    let tier = RiskTier::Irreversible;
    let decision = tier.apply_gate(Some(false));
    assert!(
        matches!(decision, GateDecision::RequiresConfirmationToken),
        "BC-2.04.007: RiskTier::Irreversible with dry_run=false must STILL require token"
    );
}

/// Irreversible tier with dry_run = true still requires token.
#[test]
fn test_BC_2_04_007_irreversible_tier_dry_run_true_still_requires_token() {
    let tier = RiskTier::Irreversible;
    let decision = tier.apply_gate(Some(true));
    assert!(
        matches!(decision, GateDecision::RequiresConfirmationToken),
        "BC-2.04.007: RiskTier::Irreversible with dry_run=true must STILL require token"
    );
}

// ─────────────────────────────────────────────────────────────
// EC-04-014: Ambiguous operations are classified as Irreversible
// ─────────────────────────────────────────────────────────────

/// EC-04-014: When in doubt, classify as Irreversible (conservative).
///
/// This test verifies that the Irreversible gate is NEVER bypassed by any
/// dry_run argument combination.
#[test]
fn test_BC_2_04_007_ec_ambiguous_classified_irreversible_gate_never_bypassed() {
    let tier = RiskTier::Irreversible; // conservative classification
    for dry_run in [None, Some(true), Some(false)] {
        let decision = tier.apply_gate(dry_run);
        assert!(
            matches!(decision, GateDecision::RequiresConfirmationToken),
            "EC-04-014: conservatively-classified Irreversible tier must always require token"
        );
    }
}

// ─────────────────────────────────────────────────────────────
// Invariant: exactly one tier per tool
// ─────────────────────────────────────────────────────────────

/// Invariant: the three tiers are distinct enum variants (each tool has exactly one tier).
#[test]
fn test_BC_2_04_007_invariant_tiers_are_distinct() {
    assert_ne!(RiskTier::Read, RiskTier::Reversible);
    assert_ne!(RiskTier::Read, RiskTier::Irreversible);
    assert_ne!(RiskTier::Reversible, RiskTier::Irreversible);
}
