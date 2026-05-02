---
document_type: gate-step-report
gate_step: c
gate_step_name: code-review
cycle: wave-3-multi-tenant
gate: wave-3-integration-gate
scope: e4be29ae..ba3b10c7 (W3-FIX-CODE-006 PR #124 + W3-FIX-SEC-005 PR #125)
reviewer: vsdd-factory:code-reviewer
develop_sha: ba3b10c7
date: 2026-05-02
phase: 3
wave: 3
step: c
pass: 7
previous_review: gate-step-c-code-review-pass6.md
verdict: CONVERGENCE_REACHED
total_findings: 0
high: 0
medium: 0
low: 0
---

# Wave 3 Integration Gate — Gate Step C: Code Review (Pass 7, final convergence seal)

**Scope:** `e4be29ae..ba3b10c7` (W3-FIX-CODE-006 PR #124 + W3-FIX-SEC-005 PR #125)
**Reviewer:** vsdd-factory:code-reviewer (Sonnet 4.6 — independent of adversary)
**Date:** 2026-05-02
**Previous review:** `gate-step-c-code-review-pass6.md` (pass 6, CONVERGENCE_REACHED, 0 findings)
**Verdict:** CONVERGENCE_REACHED — pass-7 final-discipline review across all 37 changed
files, focusing on idiomatic Rust style, documentation completeness, dead code /
unreachable patterns, lint suppression cleanliness, and module re-exports consistency.
No new findings identified. Pass-5 and pass-6 CONVERGENCE_REACHED verdicts sustained.

---

## Part A — Fix Verification (pass-6 findings)

Pass 6 reported 0 findings; there is nothing to verify against prior open items.
All findings from passes 1–4 were confirmed RESOLVED in pass 5 and re-confirmed in
pass 6. No regressions have been introduced.

| ID | Previous Severity | Status | Notes |
|----|-------------------|--------|-------|
| CR-021 | MEDIUM (pass-4) | RESOLVED (sustained) | 5-DTU ct_eq gate confirmed at all 10 sites (5 DTUs x 2 handlers). Verified unchanged at ba3b10c7. |
| CR-022 | LOW (pass-4) | RESOLVED (sustained) | `provided != Some(...)` pattern absent from all production non-harness source. Zero results on taint sweep. |
| CR-023 | LOW (pass-4) | RESOLVED (sustained) | `cr023_activity_risk_org_id_guard.rs` present with 6 tests (AC-001..006) all PASS per demo evidence. |
| R1-001 | MEDIUM (cycle-2) | RESOLVED (sustained) | `threatintel/src/routes/lookup.rs` uses `subtle::ConstantTimeEq` in configure handler. |

---

## Part A — Fresh Perspective Checks (Pass 7 focus angles)

### Angle 1: Idiomatic Rust style — ct_eq call idiom consistency

The constant-time comparison pattern is applied identically across all 10 new call sites:

```rust
let provided = headers
    .get("x-admin-token")
    .and_then(|v| v.to_str().ok())
    .unwrap_or("");
let provided_bytes = provided.as_bytes();
let expected_bytes = state.admin_token.as_bytes();
let valid: bool = provided_bytes.ct_eq(expected_bytes).into();
if !valid { ... }
```

This is the canonical pattern used by the four previously-fixed DTUs (armis, claroty,
crowdstrike, slack) and the threatintel lookup.rs configure handler. The `.into()` coercion
from `subtle::Choice` to `bool` is the standard idiom. The explicit `let valid: bool =`
type annotation aids readability and prevents `.into()` coercions from being hidden in a
condition expression. The pattern is consistent across all 10 newly-added sites.

One alternative Rust idiom — `bool::from(provided_bytes.ct_eq(expected_bytes))` — is
semantically equivalent and arguably more explicit, but the `let valid: bool = ... .into()`
form used here is already the established convention across the entire 18-site corpus.
Consistency with the existing convention is the correct choice. No finding.

### Angle 2: Documentation comments on new public APIs

All six modified `pub async fn` signatures in production source have accompanying doc
comments updated or present:

- `cyberint/src/routes/dtu.rs:post_reset` — added ADR-003 Amendment #5 doc block.
- `jira/src/routes/dtu.rs:post_reset` — ADR-003 Amendment #5 doc added.
- `nvd/src/routes/dtu.rs:post_reset` — ADR-003 Amendment #5 doc added.
- `pagerduty/src/routes/dtu.rs:post_reset` — ADR-003 Amendment #5 doc added.
- `threatintel/src/routes/dtu.rs:dtu_reset` — ADR-003 Amendment #5 doc added; original
  doc note "No auth required" corrected to reflect the new requirement.
- `threatintel/src/routes/lookup.rs:configure` — pre-existing doc; the SEC-P3-003 inline
  comment plus the `use subtle::ConstantTimeEq` import are sufficient.

The `post_configure` functions in cyberint, jira, nvd, pagerduty all have their
pre-existing doc comments intact; only the body changed. No new public API surface was
introduced without documentation.

For new test files: the six test functions in `cr023_activity_risk_org_id_guard.rs` each
have a leading doc comment explaining the guard code location, what causes a failure, and
the BC/AC reference. The module-level `//!` doc block provides comprehensive context
including guard semantics, BC reference, and acceptance criteria. This is the best-
documented test file in the corpus.

The 5 `td_wv0_08_reset_requires_admin_token.rs` files carry inner-module doc comments
(`/// No token → 401`, `/// Wrong token → 401`, `/// Correct token → 200`) matching the
sibling `td_wv0_07` files' pattern exactly. Adequate for their scope.

The 2 `td_wv0_07_configure_requires_admin_token.rs` files (jira, pagerduty) use the same
inner-module doc comment style. Adequate.

No public API surface added without adequate documentation.

### Angle 3: Dead code and unreachable patterns

**Return-after-401 path:** In every new reset handler block, the early-return pattern is:

```rust
if !valid {
    return (StatusCode::UNAUTHORIZED, Json(...)).into_response();
}
// ... normal handler body
```

The normal handler body is reachable in all cases; the early return is conditional, not
unconditional. No dead code after unconditional returns.

**Unused variable check:** The `multi_tenant.rs` and `ac_8_reset_semantics.rs` test
updates introduce a `_admin_token` prefix for the case where the token is fetched from
`start_clone()` but not needed in a particular test function body. This is the correct
Rust idiom for suppressing unused-variable warnings. The pattern is explicit and clean.

**`non_snake_case` observation in cr023:** The file `cr023_activity_risk_org_id_guard.rs`
carries `#![allow(clippy::expect_used, clippy::unwrap_used, non_snake_case)]` at line 35
(copied from `cr017_tag_alert_org_id_guard.rs`). Unlike cr017, cr023 uses no
non-snake-case identifiers — all 6 test function names are fully lowercase_snake_case
(`test_get_device_activity_real_org_absent_header_returns_401`, etc.). The
`non_snake_case` allow is technically unnecessary but entirely harmless — it suppresses
a lint that would not have fired. Per this project's established test convention
(passing `non_snake_case` consistently in this test family to accommodate future BC_-
prefixed functions), this is acceptable. It is not a regression, and cr017 exhibits the
same pattern. This is an observation, not a finding.

### Angle 4: Lint suppression cleanliness

All `#![allow(...)]` attributes introduced in this diff are in test files only. The
complete inventory of new suppressions added:

| File | Suppression | Justification |
|------|-------------|---------------|
| `cr023_activity_risk_org_id_guard.rs` | `clippy::expect_used, clippy::unwrap_used, non_snake_case` | Test infra; matches cr017 sibling convention |
| `td_wv0_08_reset_requires_admin_token.rs` x5 | `clippy::unwrap_used, clippy::expect_used` | Test infra; matches td_wv0_07 sibling convention |
| `td_wv0_07_configure_requires_admin_token.rs` x2 | `clippy::unwrap_used, clippy::expect_used` | Test infra; matches existing td_wv0_07 pattern |

No production source files (`src/`) in this diff carry any added `#[allow]` or
`#![allow]` attributes. The `#[allow(clippy::expect_used)]` at `nvd/src/routes/dtu.rs:49`
and `threatintel/src/routes/lookup.rs:174-355` were pre-existing and not touched by this
diff (confirmed by checking the diff for those files — neither adds nor removes any
`#[allow]` line). No unjustified suppressions introduced.

### Angle 5: Module re-exports consistency

The five newly-changed route modules (`cyberint/routes/dtu.rs`, `jira/routes/dtu.rs`,
`nvd/routes/dtu.rs`, `pagerduty/routes/dtu.rs`, `threatintel/routes/dtu.rs`) are already
re-exported from their respective crate route registries (confirmed by examining the
`routes/mod.rs` files, which publish `pub mod dtu`). No new re-export plumbing was needed
— the changed functions are already-public handler functions within existing modules. The
`threatintel/routes/lookup.rs` module is similarly already-public. No module re-export
inconsistency introduced.

New test files are registered in `[[test]]` blocks in their respective `Cargo.toml` files
with `required-features = ["dtu"]`. This is consistent with every other DTU test file in
the corpus.

### Angle 6: `subtle` dependency placement correctness

Five crates add `subtle = "2"` to `[dependencies]` (production deps section):
cyberint, jira, nvd, pagerduty, threatintel. The armis Cargo.toml is not changed by
this diff — armis already had `subtle` in `[dependencies]` from a prior wave. The
placement in production dependencies (not `[dev-dependencies]`) is correct because the
`ct_eq` call is in production route handlers, not test code. Consistent with the
armis/claroty/crowdstrike/slack pattern established in earlier fix waves.

The `Cargo.lock` diff shows exactly 5 new `"subtle"` entries added (one per new crate
dependency). No other packages added to the lock file. No surprise transitive dependencies
introduced.

### Angle 7: Existing test compatibility (admin token threading)

The 8 modified test helper functions (`start_with_token`, `start_clone`, `start`) now
return an additional `admin_token: String` in their tuple return type, and all call sites
were updated to destructure the new tuple element. Spot-checked:

- `ac_7_rate_limit.rs`: `start_with_token()` returns `(clone, base_url, token, admin_token)`;
  line 49 correctly destructures as `let (_clone, base_url, token, admin_token) = ...`.
- `ac_8_reset_semantics.rs`: `start()` now returns `(clone, base_url, admin_token, client)`;
  all 4 call sites updated.
- `multi_tenant.rs`: `start_clone()` now returns `(clone, base_url, admin_token, client)`;
  call sites updated with `_admin_token` where the token is unused.
- `pagerduty/tests/fidelity.rs`: `admin_token` obtained via `clone.admin_token().to_string()`;
  used in the 2 reset calls. Clean.
- `fidelity_validator.rs` (nvd, threatintel, cyberint): `FidelityCheck` `headers` field
  populated with `X-Admin-Token` in place of `..Default::default()`. This correctly
  signals to the fidelity framework that the reset endpoint now requires auth.

No existing test left with a stale unauthenticated reset call in any changed file.

### Angle 8: `non_snake_case` in armis Cargo.toml [[test]] block ordering

The `cr017_tag_alert_org_id_guard` and `cr023_activity_risk_org_id_guard` test blocks
were added to `prism-dtu-armis/Cargo.toml` after the existing `dtu_reset_auth` block and
before the `[lints]` section. This is consistent with the pattern across all other DTU
crates where test blocks are appended after existing blocks and before `[lints]`. No
ordering inconsistency.

---

## Part B — Findings

*No findings.*

---

## Summary

| Severity | Count |
|----------|-------|
| CRITICAL | 0 |
| HIGH | 0 |
| MEDIUM | 0 |
| LOW | 0 |

---

## Workspace-Wide Audit Confirmation (Pass 7)

**ct_eq production call sites:** 18 confirmed (9 DTUs x 2 handlers each). Matches
pass-5 and pass-6 counts. No regression.

**Non-constant-time admin token comparisons in production source:** Zero. The
`provided != Some(...)` pattern is absent from all non-test, non-harness production
source files.

**`subtle` in `[dependencies]` (not `[dev-dependencies]`):** Confirmed for all 9 DTUs.

**Test file `#![allow]` suppressions:** Present only in test files; all justified by
existing corpus convention. No unjustified suppressions in production source.

**All new public `post_reset`/`dtu_reset` functions:** Carry ADR-003 Amendment #5 doc
blocks. No new public API without documentation.

**Dead code / unreachable patterns:** None. Early-return guards are conditional; no code
is unreachable after the guard.

**Module re-exports:** No new re-export plumbing required; all changed modules already
publicly exported. New `[[test]]` blocks correctly registered with `required-features = ["dtu"]`.

**Cargo.lock delta:** 5 new `"subtle"` package registrations, one per newly-updated
crate. No other packages added.

**Existing test compatibility:** All 8 modified test helpers correctly propagate
`admin_token` to call sites; no stale unauthenticated reset calls remain in any changed
test file.

---

## Deferred Items (carry-forward, unchanged)

| ID | Severity | Status | Description |
|----|----------|--------|-------------|
| CR-007 | LOW | DEFERRED | `archetype`/`scale` declared but unread in `build()` — Wave 4 |
| CR-008 | LOW | DEFERRED | Placeholder `CloneState` sentinel strings — Wave 4 |
| CR-009 | LOW | DEFERRED | Wall-clock startup assertion — Wave 4 |

---

## Convergence Verdict

`CONVERGENCE_REACHED`

This is an independent pass-7 final-discipline review examining all 37 changed files
from 8 angles: idiomatic Rust style, documentation on new public APIs, dead code and
unreachable patterns, lint suppression cleanliness, module re-exports consistency, subtle
dependency placement, existing test compatibility, and Cargo.lock delta. All 8 angles
return clean results.

The single observation noted (harmless `non_snake_case` allow in cr023 copied from cr017
where it is not needed) is pre-existing corpus convention and not a finding.

The pass-5 and pass-6 CONVERGENCE_REACHED verdicts are sustained at develop HEAD
`ba3b10c7`. The wave-3 integration gate code review 3-clean-pass window is complete
at 3 of 3 (pass-5 + pass-6 + pass-7).

`CONVERGENCE_REACHED` — Wave 3 integration gate code review sealed.
