// S-1.03: Kani verification proofs — STUB (Red Gate)
//
// These proofs are gated on `#[cfg(kani)]` so they compile only under
// `cargo kani`.  All proof bodies call `unimplemented!()` so the Red Gate
// is satisfied: each proof will panic (fail) before any implementation
// exists.
//
// VP-002: deny-by-default for empty capabilities.
// VP-003: most-specific path wins (both directions).
// VP-004: exact-match with correct CapabilityExplanation fields.
//
// NOTE (from story spec § "Dev Notes"):
//   VP-004's "Deny overrides Allow at same specificity" is enforced by the
//   API making it impossible to store both Allow and Deny for the same path.
//   `grant()` is a simple BTreeMap insert (last-write wins).  The Kani proof
//   verifies that BTreeMap semantics (single-entry-per-key) hold symbolically.

use crate::capability::{CapabilityEffect, CapabilityPath, ClientCapabilities};

/// VP-002 — Deny-by-default: for any symbolic path, an empty
/// `ClientCapabilities` must return `is_allowed() == false`.
///
/// Proof name matches story spec task 5.
#[kani::proof]
pub fn proof_deny_by_default() {
    unimplemented!(
        "S-1.03 VP-002 stub: implement proof_deny_by_default — \
         construct empty ClientCapabilities and assert is_allowed returns false \
         for all symbolic paths within Kani bounds"
    )
}

/// VP-003 — Most-specific wins (both directions):
///
/// Direction A: `{"a.b" → Deny, "a.b.c" → Allow}` → `is_allowed("a.b.c")` = true.
/// Direction B: `{"a.b" → Allow, "a.b.c" → Deny}` → `is_allowed("a.b.c")` = false.
///
/// Proof name matches story spec task 6.
#[kani::proof]
pub fn proof_most_specific_wins() {
    unimplemented!(
        "S-1.03 VP-003 stub: implement proof_most_specific_wins — \
         verify that the most-specific path entry (longest prefix match) \
         determines the outcome in both Allow-over-Deny and Deny-over-Allow \
         configurations"
    )
}

/// VP-004 — Exact-match with explanation correctness:
///
/// Given `{"a.b" → Allow}`, `is_allowed("a.b")` must return `true`,
/// `explanation.matched_path == Some("a.b")`, and
/// `explanation.reason == "explicit-allow"`.
///
/// Also verifies BTreeMap single-entry-per-key semantics under Kani (last
/// `grant()` call for the same path is the only entry stored).
///
/// Proof name matches story spec task 7.
#[kani::proof]
pub fn proof_exact_match_explanation() {
    unimplemented!(
        "S-1.03 VP-004 stub: implement proof_exact_match_explanation — \
         verify matched_path and reason fields in the explanation returned \
         by is_allowed for an exact path match, and confirm BTreeMap \
         single-entry semantics under symbolic last-write-wins inputs"
    )
}
