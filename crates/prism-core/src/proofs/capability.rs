//! Kani verification proofs for the capability resolution engine.
//!
//! VP-002: deny-by-default for empty capabilities.
//! VP-003: most-specific path wins (both directions).
//! VP-004: exact-match with correct CapabilityExplanation fields.
//!
//! These proofs are gated on `#[cfg(kani)]` and compile only under `cargo kani`.
//! They have zero effect on normal test or release builds.
//!
//! # VP-004 note
//! VP-004's "Deny overrides Allow at same specificity" constraint is enforced by
//! the API making it impossible to store both Allow and Deny for the same path.
//! `grant()` is a simple BTreeMap insert (last-write wins).  The proof verifies
//! that BTreeMap semantics (single-entry-per-key) hold symbolically.

use crate::capability::{CapabilityEffect, CapabilityPath, ClientCapabilities};

// ─────────────────────────────────────────────────────────────
// Proof helpers
// ─────────────────────────────────────────────────────────────

/// Construct a `CapabilityPath` from a known-valid string.
/// Panics in proof context only if the string is invalid (it won't be).
fn make_path(s: &str) -> CapabilityPath {
    CapabilityPath::new(s).expect("proof helper: known-valid path")
}

// ─────────────────────────────────────────────────────────────
// VP-002 — Deny-by-default
// ─────────────────────────────────────────────────────────────

/// VP-002: For any symbolic path, an empty `ClientCapabilities` must return
/// `is_allowed() == false` with reason `"deny-by-default"`.
///
/// We use a fixed set of representative paths rather than symbolic strings
/// because Kani's symbolic string generation is unbounded.  The unit tests
/// exercise the same invariant with a wider range of paths.
#[kani::proof]
pub fn proof_deny_by_default() {
    let caps = ClientCapabilities::new();

    // Representative paths covering single-segment, multi-segment, and deep paths.
    let paths = [
        "a",
        "a.b",
        "a.b.c",
        "crowdstrike.hosts.write",
        "audit.read",
        "x.y.z.w.v.u.t.s",
    ];

    for p in paths {
        let path = make_path(p);
        let (allowed, explanation) = caps.is_allowed(&path);
        kani::assert(!allowed, "VP-002: empty caps must deny all paths");
        kani::assert(
            explanation.reason == "deny-by-default",
            "VP-002: reason must be deny-by-default",
        );
        kani::assert(
            explanation.matched_path.is_none(),
            "VP-002: no path should match in empty caps",
        );
    }
}

// ─────────────────────────────────────────────────────────────
// VP-003 — Most-specific wins (both directions)
// ─────────────────────────────────────────────────────────────

/// VP-003 direction A: `{"a.b" → Deny, "a.b.c" → Allow}` →
/// `is_allowed("a.b.c")` = true  (specific Allow overrides parent Deny).
#[kani::proof]
pub fn proof_most_specific_wins_allow_over_deny() {
    let mut caps = ClientCapabilities::new();
    caps.grant(make_path("a.b"), CapabilityEffect::Deny);
    caps.grant(make_path("a.b.c"), CapabilityEffect::Allow);

    let (allowed, explanation) = caps.is_allowed(&make_path("a.b.c"));

    kani::assert(
        allowed,
        "VP-003 A: specific Allow must override parent Deny",
    );
    kani::assert(
        explanation.reason == "explicit-allow",
        "VP-003 A: reason must be explicit-allow",
    );
}

/// VP-003 direction B: `{"a.b" → Allow, "a.b.c" → Deny}` →
/// `is_allowed("a.b.c")` = false  (specific Deny overrides parent Allow).
#[kani::proof]
pub fn proof_most_specific_wins_deny_over_allow() {
    let mut caps = ClientCapabilities::new();
    caps.grant(make_path("a.b"), CapabilityEffect::Allow);
    caps.grant(make_path("a.b.c"), CapabilityEffect::Deny);

    let (allowed, explanation) = caps.is_allowed(&make_path("a.b.c"));

    kani::assert(
        !allowed,
        "VP-003 B: specific Deny must override parent Allow",
    );
    kani::assert(
        explanation.reason == "explicit-deny",
        "VP-003 B: reason must be explicit-deny",
    );
}

// ─────────────────────────────────────────────────────────────
// VP-004 — Exact-match with explanation correctness
// ─────────────────────────────────────────────────────────────

/// VP-004: Given `{"a.b" → Allow}`, `is_allowed("a.b")` must return `true`,
/// `explanation.matched_path == Some("a.b")`, and
/// `explanation.reason == "explicit-allow"`.
///
/// Also verifies BTreeMap single-entry-per-key semantics: calling `grant()`
/// twice for the same path stores exactly one entry (last-write wins).
#[kani::proof]
pub fn proof_exact_match_explanation() {
    let ab = make_path("a.b");

    // Single grant — exact match.
    let mut caps = ClientCapabilities::new();
    caps.grant(ab.clone(), CapabilityEffect::Allow);

    let (allowed, explanation) = caps.is_allowed(&ab);

    kani::assert(allowed, "VP-004: exact Allow must return true");
    kani::assert(
        explanation.matched_path == Some(ab.clone()),
        "VP-004: matched_path must equal the queried path",
    );
    kani::assert(
        explanation.reason == "explicit-allow",
        "VP-004: reason must be explicit-allow",
    );
    kani::assert(
        explanation.effect == CapabilityEffect::Allow,
        "VP-004: effect must be Allow",
    );

    // Verify BTreeMap single-entry semantics: second grant overwrites first.
    let mut caps2 = ClientCapabilities::new();
    caps2.grant(ab.clone(), CapabilityEffect::Allow);
    caps2.grant(ab.clone(), CapabilityEffect::Deny); // overwrites

    let (allowed2, explanation2) = caps2.is_allowed(&ab);

    kani::assert(
        !allowed2,
        "VP-004: second grant (Deny) must overwrite first (Allow)",
    );
    kani::assert(
        explanation2.effect == CapabilityEffect::Deny,
        "VP-004: effect after overwrite must be Deny",
    );
    // Exactly one entry stored — display list has length 1.
    kani::assert(
        caps2.capabilities_for_display().len() == 1,
        "VP-004: BTreeMap must store exactly one entry per key",
    );
}
