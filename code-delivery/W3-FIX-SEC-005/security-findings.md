# Security Findings: W3-FIX-SEC-005

**Story:** 5-DTU admin-token uniformity — constant-time comparison + post_reset gate
**Source findings:** gate-step-c-code-review-pass4.md (Pass 4, scope a7f0d374..e4be29ae)
**Status:** Both findings CLOSED by this PR

---

## CR-021 — Missing Authorization on post_reset/dtu_reset (MEDIUM, CWE-863)

| Field | Value |
|-------|-------|
| ID | CR-021 |
| Severity | MEDIUM |
| CWE | CWE-863: Incorrect Authorization |
| BC Reference | BC-3.5.001 Invariant 3; BC-3.5.002 Precondition 6 |
| Fix Story | W3-FIX-SEC-005 |
| Status | **CLOSED** |

### Description

All 5 DTU clones (`prism-dtu-cyberint`, `prism-dtu-jira`, `prism-dtu-nvd`,
`prism-dtu-pagerduty`, `prism-dtu-threatintel`) had `post_reset`/`dtu_reset` handlers
with no admin-token gate. Any caller who knew the DTU URL could reset all accumulated
per-clone state (session store, alert store, failure injection state) without
authentication, violating BC-3.5.001 Invariant 3 (test isolation enforcement point).

This was discovered as a sibling-gap: W3-FIX-SEC-002 (PR #119) applied the gate to
armis, claroty, crowdstrike, and slack but left 5 DTU clones unaddressed.

### Fix Applied

Admin-token gate inserted as the **first check** in each handler, before any org-id
branching or reset logic. Uses `subtle::ConstantTimeEq::ct_eq` on byte slices.
Pattern matches the canonical reference at `crates/prism-dtu-armis/src/routes/dtu.rs:77-95`.

Cyberint special case: gate added before the `X-Prism-Org-Id` org-scoped reset branch.
An unauthenticated caller cannot reach either the global reset or the per-org reset path.

### Verification

- `td_wv0_08::test_reset_requires_admin_token_missing_returns_401` — missing header → 401: PASS × 5 DTUs
- `td_wv0_08::test_reset_requires_admin_token_wrong_returns_401` — wrong token → 401: PASS × 5 DTUs
- `td_wv0_08::test_reset_correct_admin_token_returns_200` — correct token → 200: PASS × 5 DTUs
- **15 tests, all PASS** (nextest run in 1.125s)

### Fix Sites

| DTU | Handler | File |
|-----|---------|------|
| cyberint | `post_reset` | `crates/prism-dtu-cyberint/src/routes/dtu.rs` |
| jira | `post_reset` | `crates/prism-dtu-jira/src/routes/dtu.rs` |
| nvd | `post_reset` | `crates/prism-dtu-nvd/src/routes/dtu.rs` |
| pagerduty | `post_reset` | `crates/prism-dtu-pagerduty/src/routes/dtu.rs` |
| threatintel | `dtu_reset` | `crates/prism-dtu-threatintel/src/routes/dtu.rs` |

---

## CR-022 — Non-Constant-Time Comparison in post_configure (LOW, CWE-208)

| Field | Value |
|-------|-------|
| ID | CR-022 |
| Severity | LOW |
| CWE | CWE-208: Observable Timing Discrepancy |
| BC Reference | BC-3.5.002 Precondition 6 |
| Fix Story | W3-FIX-SEC-005 |
| Status | **CLOSED** |

### Description

All 5 DTU `post_configure` handlers used short-circuit string inequality (`!=`) to compare
the provided admin token against the expected value. Short-circuit comparison exits on the
first differing byte, creating a timing oracle: an attacker can distinguish wrong tokens by
their position of first divergence. This is a low-severity theoretical concern in the test
isolation context (DTU never exposed externally), but ADR-003 Amendment #5 mandates uniform
`subtle::ConstantTimeEq` usage across all DTU clones.

This was a sibling-gap from W3-FIX-SEC-004 (PR #122), which migrated armis/claroty/
crowdstrike/slack's `post_configure` to constant-time comparison but left the 5 remaining
DTU clones using `!=`.

### Fix Applied

Replaced `provided != Some(state.admin_token.as_str())` with:
```rust
use subtle::ConstantTimeEq;
let provided_bytes = provided.unwrap_or("").as_bytes();
let expected_bytes = state.admin_token.as_bytes();
let valid: bool = provided_bytes.ct_eq(expected_bytes).into();
if !valid { ... }
```

Pattern matches the canonical reference at `crates/prism-dtu-armis/src/routes/dtu.rs:40-55`.

### Verification

- `grep -r 'provided != Some'` across all 5 affected crates: **0 matches** (confirmed in AC-001-ct-eq-presence.txt)
- `use subtle::ConstantTimeEq` and `.ct_eq(expected_bytes)` confirmed in all 5 crates (AC-001-ct-eq-presence.txt)
- td_wv0_07_configure_requires_admin_token test files present for all 5 crates

### AC-006 Deviation (documentation only)

The story specified `subtle = { workspace = true }` in per-crate Cargo.toml. The implementer
used `subtle = "2"` (direct pin) in all 5 crates. Functional behavior is identical — subtle 2.x
is present and `ConstantTimeEq` compiles correctly. The workspace dep form normalization is
a low-priority follow-up; it carries no security impact.

### Fix Sites

| DTU | Handler | File | Before | After |
|-----|---------|------|--------|-------|
| cyberint | `post_configure` | `crates/prism-dtu-cyberint/src/routes/dtu.rs:38` (approx) | `provided != Some(...)` | `ct_eq` on bytes |
| jira | `post_configure` | `crates/prism-dtu-jira/src/routes/dtu.rs:36` (approx) | same | same |
| nvd | `post_configure` | `crates/prism-dtu-nvd/src/routes/dtu.rs:69` (approx) | same | same |
| pagerduty | `post_configure` | `crates/prism-dtu-pagerduty/src/routes/dtu.rs:66` (approx) | same | same |
| threatintel | `post_configure` | `crates/prism-dtu-threatintel/src/routes/dtu.rs:308` (approx) | same | same |

---

## Summary

| Finding | Severity | CWE | Pre-fix | Post-fix |
|---------|----------|-----|---------|----------|
| CR-021 | MEDIUM | CWE-863 | 5 DTUs' post_reset unguarded | 5 DTUs gated, 15 tests PASS |
| CR-022 | LOW | CWE-208 | 5 DTUs' post_configure uses `!=` | 5 DTUs use `ct_eq`, 0 residues |

Both findings closed by this PR. Zero new security findings introduced.
