// S-1.03: Capability Resolution Engine — Unit Tests (Red Gate)
//
// The test naming convention (test_S_1_03_*) uses uppercase letters to encode
// the story ID per the factory BC-naming standard.  Suppress the non_snake_case
// lint for this module only — the convention is intentional.
#![allow(non_snake_case)]
//
//
// Naming convention: test_BC_S_SS_NNN_<assertion>
// All tests call into unimplemented!() stubs and will panic (fail) until the
// implementation is written.  Red Gate must be verified before implementation.
//
// AC map:
//   AC-1  → test_S_1_03_ac1_empty_caps_deny_by_default
//   AC-2  → test_S_1_03_ac2_parent_allow_covers_child
//   AC-3  → test_S_1_03_ac3_specific_deny_overrides_parent_allow
//   AC-4  → test_S_1_03_ac4_exact_match_explanation_matched_path
//   AC-5  → test_S_1_03_ac5_rejects_empty_segment_path
//   AC-6  → test_S_1_03_ac6_parent_called_twice_reaches_grandparent
//   AC-7  → test_S_1_03_ac7_parent_allow_covers_grandchild
//
// VP map (unit-level assertions; Kani proofs are in src/proofs/capability.rs):
//   VP-002 → test_S_1_03_vp002_deny_by_default_unit
//   VP-003 → test_S_1_03_vp003_most_specific_wins_deny_over_allow
//             test_S_1_03_vp003_most_specific_wins_allow_over_deny
//   VP-004 → test_S_1_03_vp004_exact_match_explanation_fields
//
// Additional edge-case tests derived from story spec § "Edge Cases":
//   test_S_1_03_ec_rejects_empty_string
//   test_S_1_03_ec_rejects_nine_segments
//   test_S_1_03_ec_rejects_exceeds_256_chars
//   test_S_1_03_ec_rejects_invalid_chars
//   test_S_1_03_ec_parent_of_single_segment_is_none
//   test_S_1_03_ec_is_prefix_of_correct
//   test_S_1_03_ec_is_prefix_of_rejects_partial_segment_match
//   test_S_1_03_ec_grant_last_write_wins
//   test_S_1_03_ec_exact_allow_beats_parent_deny
//   test_S_1_03_ec_capabilities_for_display_sorted

use crate::capability::{CapabilityEffect, CapabilityPath, ClientCapabilities};

// ─────────────────────────────────────────────────────────────
// Helper: construct a validated CapabilityPath or panic in test context.
// Panics are expected here because the underlying stub panics.
// ─────────────────────────────────────────────────────────────
fn cap(s: &str) -> CapabilityPath {
    CapabilityPath::new(s).expect("test helper: expected valid CapabilityPath")
}

// ─────────────────────────────────────────────────────────────
// AC-1: Empty ClientCapabilities → deny for any path
// ─────────────────────────────────────────────────────────────

#[test]
fn test_S_1_03_ac1_empty_caps_deny_by_default() {
    // AC-1: Given an empty ClientCapabilities, When is_allowed("any.path") is
    // called, Then it returns (false, explanation) where
    // explanation.reason == "deny-by-default".
    let caps = ClientCapabilities::new();
    let path = cap("any.path");
    let (allowed, explanation) = caps.is_allowed(&path);
    assert!(!allowed, "AC-1: empty caps must deny");
    assert_eq!(
        explanation.reason, "deny-by-default",
        "AC-1: reason must be deny-by-default"
    );
    assert!(
        explanation.matched_path.is_none(),
        "AC-1: matched_path must be None for default deny"
    );
    assert_eq!(
        explanation.effect,
        CapabilityEffect::Deny,
        "AC-1: effect must be Deny"
    );
    assert!(
        !explanation.allowed,
        "AC-1: explanation.allowed must be false"
    );
}

// ─────────────────────────────────────────────────────────────
// AC-2: Parent Allow covers child path
// ─────────────────────────────────────────────────────────────

#[test]
fn test_S_1_03_ac2_parent_allow_covers_child() {
    // AC-2: Given {"crowdstrike" → Allow}, When is_allowed("crowdstrike.hosts.write")
    // is called, Then it returns true (inherits Allow from parent).
    let mut caps = ClientCapabilities::new();
    caps.grant(cap("crowdstrike"), CapabilityEffect::Allow);
    let (allowed, explanation) = caps.is_allowed(&cap("crowdstrike.hosts.write"));
    assert!(allowed, "AC-2: parent Allow must cover child path");
    assert_eq!(
        explanation.matched_path,
        Some(cap("crowdstrike")),
        "AC-2: matched_path must be the parent entry"
    );
    assert!(
        matches!(explanation.reason, "parent-allow"),
        "AC-2: reason must be parent-allow, got: {}",
        explanation.reason
    );
}

// ─────────────────────────────────────────────────────────────
// AC-3: Most-specific Deny overrides parent Allow
// ─────────────────────────────────────────────────────────────

#[test]
fn test_S_1_03_ac3_specific_deny_overrides_parent_allow() {
    // AC-3: Given {"crowdstrike" → Allow, "crowdstrike.hosts.write" → Deny},
    // When is_allowed("crowdstrike.hosts.write") is called, Then it returns
    // false (most-specific path wins — Deny at exact path overrides parent Allow).
    let mut caps = ClientCapabilities::new();
    caps.grant(cap("crowdstrike"), CapabilityEffect::Allow);
    caps.grant(cap("crowdstrike.hosts.write"), CapabilityEffect::Deny);
    let (allowed, explanation) = caps.is_allowed(&cap("crowdstrike.hosts.write"));
    assert!(!allowed, "AC-3: specific Deny must override parent Allow");
    assert_eq!(
        explanation.matched_path,
        Some(cap("crowdstrike.hosts.write")),
        "AC-3: matched_path must be the most-specific entry"
    );
    assert_eq!(
        explanation.reason, "explicit-deny",
        "AC-3: reason must be explicit-deny"
    );
}

// ─────────────────────────────────────────────────────────────
// AC-4: Exact match — explanation.matched_path equals the path
// ─────────────────────────────────────────────────────────────

#[test]
fn test_S_1_03_ac4_exact_match_explanation_matched_path() {
    // AC-4: Given {"audit.read" → Allow}, When is_allowed("audit.read") is
    // called, Then it returns true and explanation.matched_path equals "audit.read".
    let mut caps = ClientCapabilities::new();
    caps.grant(cap("audit.read"), CapabilityEffect::Allow);
    let (allowed, explanation) = caps.is_allowed(&cap("audit.read"));
    assert!(allowed, "AC-4: exact Allow must return true");
    assert_eq!(
        explanation.matched_path,
        Some(cap("audit.read")),
        "AC-4: matched_path must equal the exact queried path"
    );
    assert_eq!(
        explanation.reason, "explicit-allow",
        "AC-4: reason must be explicit-allow"
    );
}

// ─────────────────────────────────────────────────────────────
// AC-5: Rejects path with empty segment ("a..b")
// ─────────────────────────────────────────────────────────────

#[test]
fn test_S_1_03_ac5_rejects_empty_segment_path() {
    // AC-5: Given CapabilityPath::new("a..b"), When validation runs, Then it
    // returns Err (empty segment between dots is invalid).
    let result = CapabilityPath::new("a..b");
    assert!(
        result.is_err(),
        "AC-5: path with empty segment must be rejected"
    );
}

// ─────────────────────────────────────────────────────────────
// AC-6: parent() called twice reaches grandparent
// ─────────────────────────────────────────────────────────────

#[test]
fn test_S_1_03_ac6_parent_called_twice_reaches_grandparent() {
    // AC-6: Given CapabilityPath("a.b.c"), When parent() is called twice,
    // Then the result is Some(CapabilityPath("a")).
    let path = cap("a.b.c");
    let parent = path.parent().expect("a.b.c must have parent a.b");
    assert_eq!(
        parent,
        cap("a.b"),
        "AC-6: first parent() call must return a.b"
    );
    let grandparent = parent.parent().expect("a.b must have parent a");
    assert_eq!(
        grandparent,
        cap("a"),
        "AC-6: second parent() call must return a"
    );
}

// ─────────────────────────────────────────────────────────────
// AC-7: Parent Allow at "crowdstrike.hosts" covers grandchild read
// ─────────────────────────────────────────────────────────────

#[test]
fn test_S_1_03_ac7_parent_allow_covers_grandchild() {
    // AC-7: Given {"crowdstrike.hosts" → Allow}, When
    // is_allowed("crowdstrike.hosts.read") is called, Then it returns true
    // (parent "crowdstrike.hosts" = Allow covers child paths).
    let mut caps = ClientCapabilities::new();
    caps.grant(cap("crowdstrike.hosts"), CapabilityEffect::Allow);
    let (allowed, _) = caps.is_allowed(&cap("crowdstrike.hosts.read"));
    assert!(allowed, "AC-7: parent Allow must cover grandchild path");
}

// ─────────────────────────────────────────────────────────────
// VP-002 unit assertion (mirrors Kani proof intent at unit level)
// ─────────────────────────────────────────────────────────────

#[test]
fn test_S_1_03_vp002_deny_by_default_unit() {
    // VP-002: deny-by-default.  Test a range of path structures.
    let caps = ClientCapabilities::new();
    let paths = [
        "a",
        "a.b",
        "a.b.c",
        "crowdstrike.hosts.write",
        "audit.read",
        "x.y.z.w",
    ];
    for p in paths {
        let path = cap(p);
        let (allowed, explanation) = caps.is_allowed(&path);
        assert!(
            !allowed,
            "VP-002: empty caps must deny path '{p}'"
        );
        assert_eq!(
            explanation.reason, "deny-by-default",
            "VP-002: reason must be deny-by-default for path '{p}'"
        );
    }
}

// ─────────────────────────────────────────────────────────────
// VP-003 unit assertions (mirrors Kani proof intent at unit level)
// ─────────────────────────────────────────────────────────────

#[test]
fn test_S_1_03_vp003_most_specific_wins_deny_over_allow() {
    // VP-003 direction A: {"a.b" → Allow, "a.b.c" → Deny}
    // → is_allowed("a.b.c") == false (specific Deny wins over parent Allow).
    let mut caps = ClientCapabilities::new();
    caps.grant(cap("a.b"), CapabilityEffect::Allow);
    caps.grant(cap("a.b.c"), CapabilityEffect::Deny);
    let (allowed, _) = caps.is_allowed(&cap("a.b.c"));
    assert!(
        !allowed,
        "VP-003 A: specific Deny must override parent Allow"
    );
}

#[test]
fn test_S_1_03_vp003_most_specific_wins_allow_over_deny() {
    // VP-003 direction B: {"a.b" → Deny, "a.b.c" → Allow}
    // → is_allowed("a.b.c") == true (specific Allow wins over parent Deny).
    let mut caps = ClientCapabilities::new();
    caps.grant(cap("a.b"), CapabilityEffect::Deny);
    caps.grant(cap("a.b.c"), CapabilityEffect::Allow);
    let (allowed, _) = caps.is_allowed(&cap("a.b.c"));
    assert!(
        allowed,
        "VP-003 B: specific Allow must override parent Deny"
    );
}

// ─────────────────────────────────────────────────────────────
// VP-004 unit assertion (mirrors Kani proof intent at unit level)
// ─────────────────────────────────────────────────────────────

#[test]
fn test_S_1_03_vp004_exact_match_explanation_fields() {
    // VP-004: exact match → explanation fields correct.
    // {"a.b" → Allow} → is_allowed("a.b") returns true,
    // matched_path == Some("a.b"), reason == "explicit-allow".
    let mut caps = ClientCapabilities::new();
    caps.grant(cap("a.b"), CapabilityEffect::Allow);
    let (allowed, explanation) = caps.is_allowed(&cap("a.b"));
    assert!(allowed, "VP-004: exact Allow must return true");
    assert_eq!(
        explanation.matched_path,
        Some(cap("a.b")),
        "VP-004: matched_path must equal the exact path"
    );
    assert_eq!(
        explanation.reason, "explicit-allow",
        "VP-004: reason must be explicit-allow"
    );
    assert_eq!(
        explanation.effect,
        CapabilityEffect::Allow,
        "VP-004: effect must be Allow"
    );
}

// ─────────────────────────────────────────────────────────────
// Edge-case tests from story spec § "Edge Cases"
// ─────────────────────────────────────────────────────────────

#[test]
fn test_S_1_03_ec_rejects_empty_string() {
    // Edge case: CapabilityPath::new("") must return Err.
    let result = CapabilityPath::new("");
    assert!(result.is_err(), "EC: empty string must be rejected");
}

#[test]
fn test_S_1_03_ec_rejects_nine_segments() {
    // Edge case: max 8 segments; 9 segments must be rejected.
    let nine_seg = "a.b.c.d.e.f.g.h.i";
    let result = CapabilityPath::new(nine_seg);
    assert!(
        result.is_err(),
        "EC: 9-segment path must be rejected (max is 8)"
    );
}

#[test]
fn test_S_1_03_ec_rejects_exceeds_256_chars() {
    // Edge case: path longer than 256 characters must be rejected.
    // 257 'a's, no dots, one segment — only the length rule fires.
    let long_path = "a".repeat(257);
    let result = CapabilityPath::new(&long_path);
    assert!(
        result.is_err(),
        "EC: path exceeding 256 chars must be rejected"
    );
}

#[test]
fn test_S_1_03_ec_rejects_invalid_chars() {
    // Edge case: segment containing '!' must be rejected.
    let result = CapabilityPath::new("a.b!c");
    assert!(
        result.is_err(),
        "EC: segment with invalid char '!' must be rejected"
    );
}

#[test]
fn test_S_1_03_ec_parent_of_single_segment_is_none() {
    // Edge case: CapabilityPath("a").parent() must return None.
    let path = cap("a");
    let parent = path.parent();
    assert!(
        parent.is_none(),
        "EC: parent of single-segment path must be None"
    );
}

#[test]
fn test_S_1_03_ec_is_prefix_of_correct() {
    // Edge case: "a.b" is a prefix of "a.b.c" but not "a.bc".
    let ab = cap("a.b");
    let abc = cap("a.b.c");
    let abc_no_dot = cap("a.bc"); // different segment

    assert!(
        ab.is_prefix_of(&abc),
        "EC: a.b must be a prefix of a.b.c"
    );
    assert!(
        !ab.is_prefix_of(&abc_no_dot),
        "EC: a.b must NOT be a prefix of a.bc"
    );
}

#[test]
fn test_S_1_03_ec_is_prefix_of_self() {
    // Edge case: a path is a prefix of itself.
    let ab = cap("a.b");
    assert!(
        ab.is_prefix_of(&ab),
        "EC: a path must be a prefix of itself"
    );
}

#[test]
fn test_S_1_03_ec_is_prefix_of_rejects_partial_segment_match() {
    // Edge case: "crown" must NOT be a prefix of "crowdstrike.read".
    // This ensures prefix matching is segment-boundary-aware.
    let crown = cap("crown");
    let crowdstrike = cap("crowdstrike.read");
    assert!(
        !crown.is_prefix_of(&crowdstrike),
        "EC: 'crown' must NOT be a prefix of 'crowdstrike.read'"
    );
}

#[test]
fn test_S_1_03_ec_grant_last_write_wins() {
    // Edge case: calling grant() twice for the same path — last write wins.
    // The second grant must overwrite the first.
    let mut caps = ClientCapabilities::new();
    caps.grant(cap("audit.read"), CapabilityEffect::Allow);
    caps.grant(cap("audit.read"), CapabilityEffect::Deny); // overwrites
    let (allowed, _) = caps.is_allowed(&cap("audit.read"));
    assert!(
        !allowed,
        "EC: second grant (Deny) must overwrite first grant (Allow)"
    );
}

#[test]
fn test_S_1_03_ec_exact_allow_beats_parent_deny() {
    // Edge case: exact-path Allow wins over parent-path Deny.
    // {"crowdstrike" → Deny, "crowdstrike.read" → Allow}
    // → is_allowed("crowdstrike.read") must return true.
    let mut caps = ClientCapabilities::new();
    caps.grant(cap("crowdstrike"), CapabilityEffect::Deny);
    caps.grant(cap("crowdstrike.read"), CapabilityEffect::Allow);
    let (allowed, explanation) = caps.is_allowed(&cap("crowdstrike.read"));
    assert!(
        allowed,
        "EC: exact-path Allow must win over parent Deny"
    );
    assert_eq!(
        explanation.reason, "explicit-allow",
        "EC: reason must be explicit-allow"
    );
}

#[test]
fn test_S_1_03_ec_capabilities_for_display_sorted() {
    // Edge case: capabilities_for_display() must return rules in sorted order.
    let mut caps = ClientCapabilities::new();
    caps.grant(cap("z.last"), CapabilityEffect::Allow);
    caps.grant(cap("a.first"), CapabilityEffect::Deny);
    caps.grant(cap("m.middle"), CapabilityEffect::Allow);
    let display = caps.capabilities_for_display();
    assert_eq!(display.len(), 3, "EC: display must return all 3 rules");
    // BTreeMap iteration is lexicographically sorted.
    let paths: Vec<&str> = display.iter().map(|(p, _)| p.as_str()).collect();
    let mut sorted = paths.clone();
    sorted.sort();
    assert_eq!(
        paths, sorted,
        "EC: capabilities_for_display must return rules in sorted order"
    );
}

#[test]
fn test_S_1_03_ec_eight_segments_accepted() {
    // Edge case: exactly 8 segments must be accepted (boundary condition).
    let eight_seg = "a.b.c.d.e.f.g.h"; // 8 segments — must succeed
    let result = CapabilityPath::new(eight_seg);
    assert!(
        result.is_ok(),
        "EC: 8-segment path must be accepted (at boundary)"
    );
}

#[test]
fn test_S_1_03_ec_exactly_256_chars_accepted() {
    // Edge case: exactly 256 character path must be accepted (boundary).
    // Build a single-segment name of exactly 256 chars.
    let at_limit = "a".repeat(256);
    let result = CapabilityPath::new(&at_limit);
    assert!(
        result.is_ok(),
        "EC: 256-char path must be accepted (at boundary)"
    );
}

#[test]
fn test_S_1_03_ec_from_iter_builds_correctly() {
    // from_iter should produce the same result as sequential grant() calls.
    let entries = vec![
        (cap("crowdstrike"), CapabilityEffect::Allow),
        (cap("crowdstrike.hosts.write"), CapabilityEffect::Deny),
    ];
    let caps = ClientCapabilities::from_iter(entries);
    let (allowed_parent, _) = caps.is_allowed(&cap("crowdstrike.hosts.read"));
    let (allowed_specific, _) = caps.is_allowed(&cap("crowdstrike.hosts.write"));
    assert!(allowed_parent, "EC: from_iter parent Allow must cover child");
    assert!(
        !allowed_specific,
        "EC: from_iter specific Deny must override parent Allow"
    );
}

#[test]
fn test_S_1_03_ec_unrelated_rule_does_not_affect_other_path() {
    // Granting a rule for one subtree must not affect a completely separate path.
    let mut caps = ClientCapabilities::new();
    caps.grant(cap("crowdstrike"), CapabilityEffect::Allow);
    let (allowed, explanation) = caps.is_allowed(&cap("audit.read"));
    assert!(
        !allowed,
        "EC: rule for 'crowdstrike' must not affect 'audit.read'"
    );
    assert_eq!(
        explanation.reason, "deny-by-default",
        "EC: unrelated path must fall back to deny-by-default"
    );
}
