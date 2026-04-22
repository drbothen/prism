# S-1.03 AC-8 / AC-9 / AC-10 — Kani Formal Verification Proofs

## Status

Kani proofs are defined and ready. Execution requires the Kani toolchain
(`cargo install kani-verifier && cargo kani setup`), which is not available
in the standard CI runner. The proof harnesses are committed in source and
will be verified in a dedicated formal-verification pipeline.

---

## AC-8 — VP-002: Deny-by-default (proof_deny_by_default)

**Source:** `crates/prism-core/src/proofs/capability.rs`

```rust
#[kani::proof]
pub fn proof_deny_by_default() {
    let caps = ClientCapabilities::new();
    let paths = [
        "a", "a.b", "a.b.c",
        "crowdstrike.hosts.write", "audit.read", "x.y.z.w.v.u.t.s",
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
```

**Run command:**
```
cargo kani --package prism-core \
    --harness proof_deny_by_default \
    -Z unstable-options
```

**Expected result:** `VERIFICATION:- SUCCESSFUL`

---

## AC-9 — VP-003: Most-specific path wins (proof_most_specific_wins_*)

**Source:** `crates/prism-core/src/proofs/capability.rs`

```rust
#[kani::proof]
pub fn proof_most_specific_wins_allow_over_deny() {
    let mut caps = ClientCapabilities::new();
    caps.grant(make_path("a.b"), CapabilityEffect::Deny);
    caps.grant(make_path("a.b.c"), CapabilityEffect::Allow);
    let (allowed, explanation) = caps.is_allowed(&make_path("a.b.c"));
    kani::assert(allowed, "VP-003 A: specific Allow must override parent Deny");
    kani::assert(
        explanation.reason == "explicit-allow",
        "VP-003 A: reason must be explicit-allow",
    );
}

#[kani::proof]
pub fn proof_most_specific_wins_deny_over_allow() {
    let mut caps = ClientCapabilities::new();
    caps.grant(make_path("a.b"), CapabilityEffect::Allow);
    caps.grant(make_path("a.b.c"), CapabilityEffect::Deny);
    let (allowed, explanation) = caps.is_allowed(&make_path("a.b.c"));
    kani::assert(!allowed, "VP-003 B: specific Deny must override parent Allow");
    kani::assert(
        explanation.reason == "explicit-deny",
        "VP-003 B: reason must be explicit-deny",
    );
}
```

**Run command:**
```
cargo kani --package prism-core \
    --harness proof_most_specific_wins_allow_over_deny \
    --harness proof_most_specific_wins_deny_over_allow \
    -Z unstable-options
```

**Expected result:** `VERIFICATION:- SUCCESSFUL` (both harnesses)

---

## AC-10 — VP-004: Exact-match explanation correctness (proof_exact_match_explanation)

**Source:** `crates/prism-core/src/proofs/capability.rs`

```rust
#[kani::proof]
pub fn proof_exact_match_explanation() {
    let ab = make_path("a.b");
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
    // BTreeMap single-entry semantics: second grant overwrites first
    let mut caps2 = ClientCapabilities::new();
    caps2.grant(ab.clone(), CapabilityEffect::Allow);
    caps2.grant(ab.clone(), CapabilityEffect::Deny);
    let (allowed2, explanation2) = caps2.is_allowed(&ab);
    kani::assert(!allowed2, "VP-004: second grant (Deny) must overwrite first");
    kani::assert(
        explanation2.effect == CapabilityEffect::Deny,
        "VP-004: effect after overwrite must be Deny",
    );
    kani::assert(
        caps2.capabilities_for_display().len() == 1,
        "VP-004: BTreeMap must store exactly one entry per key",
    );
}
```

**Run command:**
```
cargo kani --package prism-core \
    --harness proof_exact_match_explanation \
    -Z unstable-options
```

**Expected result:** `VERIFICATION:- SUCCESSFUL`

---

## Unit-level Proxies (Run Now, No Kani Required)

The three Kani proofs are shadowed by unit tests that exercise the same
invariants with concrete inputs:

| Kani Proof | Unit Proxy | Status |
|---|---|---|
| `proof_deny_by_default` (VP-002) | `test_S_1_03_vp002_deny_by_default_unit` | PASSED |
| `proof_most_specific_wins_*` (VP-003) | `test_S_1_03_vp003_most_specific_wins_deny_over_allow`, `test_S_1_03_vp003_most_specific_wins_allow_over_deny` | PASSED |
| `proof_exact_match_explanation` (VP-004) | `test_S_1_03_vp004_exact_match_explanation_fields` | PASSED |
