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
pass: 5
previous_review: gate-step-c-code-review-pass4.md
verdict: CONVERGENCE_REACHED
total_findings: 0
high: 0
medium: 0
low: 0
---

# Wave 3 Integration Gate — Gate Step C: Code Review (Pass 5, post-W3.4)

**Scope:** `e4be29ae..ba3b10c7` (W3-FIX-CODE-006 PR #124 + W3-FIX-SEC-005 PR #125)
**Reviewer:** vsdd-factory:code-reviewer (Sonnet 4.6 — independent of adversary)
**Date:** 2026-05-02
**Previous review:** `gate-step-c-code-review-pass4.md` (pass 4, SHA `e4be29ae`)
**Verdict:** CONVERGENCE_REACHED — all three pass-4 findings (CR-021 MEDIUM, CR-022 LOW,
CR-023 LOW) are fully resolved and R1-001 (cycle-2 ThreatIntel `lookup.rs` ct_eq) is
confirmed closed. The workspace-wide audit finds no remaining non-constant-time admin
token comparisons in production source. No new findings.

---

## Part A — Fix Verification (pass-4 findings)

| ID | Previous Severity | Status | Notes |
|----|-------------------|--------|-------|
| CR-021 | MEDIUM | RESOLVED | `crates/prism-dtu-cyberint/src/routes/dtu.rs:67-86` — `post_reset` now carries the `subtle::ConstantTimeEq` ct_eq gate identical in structure to Armis. New test file `tests/td_wv0_08_reset_requires_admin_token.rs` (3 tests: missing → 401, wrong → 401, correct → 200) is registered in `Cargo.toml`. Existing `ac_8_reset_semantics.rs`, `multi_tenant.rs`, `ac_7_rate_limit.rs`, and `fidelity_validator.rs` all updated to supply `X-Admin-Token` on reset calls. |
| CR-022 | LOW | RESOLVED | `crates/prism-dtu-cyberint/src/routes/dtu.rs:36-48` — `post_configure` now uses ct_eq. `subtle = "2"` added to `[dependencies]` in `Cargo.toml`. The previous `provided != Some(state.admin_token.as_str())` pattern is gone. Jira, NVD, PagerDuty, and ThreatIntel also received the same migration (all had the same `!=` pattern), making all 9 DTUs uniform. |
| CR-023 | LOW | RESOLVED | `crates/prism-dtu-armis/tests/cr023_activity_risk_org_id_guard.rs` exists with exactly 6 test functions (AC-001 through AC-006): `test_get_device_activity_real_org_absent_header_returns_401`, `test_get_device_activity_real_org_correct_header_returns_200`, `test_get_device_activity_default_instance_absent_header_returns_200`, `test_get_device_risk_real_org_absent_header_returns_401`, `test_get_device_risk_real_org_correct_header_returns_200`, `test_get_device_risk_default_instance_absent_header_returns_200`. Registered in `Cargo.toml` under `[[test]]` with `required-features = ["dtu"]`. |

### R1-001 Verification (cycle-2 carry-forward — ThreatIntel `lookup.rs`)

| ID | Previous Severity | Status | Notes |
|----|-------------------|--------|-------|
| R1-001 | MEDIUM (cycle-2) | RESOLVED | `crates/prism-dtu-threatintel/src/routes/lookup.rs:15,315` — `use subtle::ConstantTimeEq` import present; `configure` handler uses `ct_eq` for admin token comparison. The `!=` pattern is gone. `subtle = "2"` added to `Cargo.toml`. |

---

## Part B — Findings

*No findings.*

---

## Positive Observations (Non-Finding)

**Complete 9-DTU constant-time uniformity achieved.** The workspace-wide audit
confirms `ct_eq` is now present in every production admin token comparison site across
all nine DTU crates (armis, claroty, crowdstrike, cyberint, jira, nvd, pagerduty, slack,
threatintel). The `grep -rn "admin_token" --include="*.rs" crates/ | grep "!=" | grep -v
/tests/ | grep -v prism-dtu-harness` sweep returns zero results. No remaining
short-circuit `!=` comparisons exist in production source.

**5-DTU `post_reset` admin token gate is complete and correct.** Cyberint, Jira, NVD,
PagerDuty, and ThreatIntel all received the gate in PR #125. The implementations are
structurally identical across all nine DTUs: `provided.unwrap_or("").as_bytes()`,
`state.admin_token.as_bytes()`, `ct_eq`, `!valid → 401`. Armis, Claroty, CrowdStrike,
and Slack already had the gate from W3-FIX-SEC-002 (PR #119); the PR-125 scope correctly
covers the remaining five.

**`subtle = "2"` dependency placement is correct.** All five newly-updated crates add
`subtle` to `[dependencies]` (not `[dev-dependencies]`), consistent with the production
code location of the ct_eq call. Workspace-managed version ensures no drift.

**Test regression quality is high.** Each of the five new `td_wv0_08_reset_requires_admin_token.rs`
files follows the same 3-test structure (AC-001: missing → 401, AC-002: wrong → 401,
AC-003: correct → 200). All test files are wrapped in `#[cfg(feature = "dtu")]` and
registered with `required-features = ["dtu"]` in `Cargo.toml`. The pattern is consistent
with the previously reviewed `dtu_reset_auth.rs` files in Armis, Claroty, CrowdStrike,
and Slack.

**Existing test suite updated completely.** The Cyberint `ac_8_reset_semantics.rs` fix
threads `admin_token` through the `start()` helper and supplies it on all four reset
calls. The `multi_tenant.rs`, `ac_7_rate_limit.rs`, and `fidelity_validator.rs` updates
are consistent. No reset call against the standalone `CyberintClone` endpoint was left
un-tokened. The `harness_tests.rs` calls to the logical harness `dtu_reset` endpoint
(which correctly has no auth gate, being a test scaffold) are pre-existing and out of
scope.

**CR-023 test structure matches the CR-017 corpus.** The `cr023_activity_risk_org_id_guard.rs`
file mirrors the documentation format, test naming convention, and guard-code citation
style established by `cr017_tag_alert_org_id_guard.rs`. The module-level doc correctly
cites BC-3.5.001 invariant 3, and each test function documents which guard line it would
fail against. This provides robust regression coverage.

**Pure-core / effectful-shell boundary preserved.** Neither PR introduces any direct I/O
or side effects in core types. All new production code is in route handlers (effectful
shell). The `subtle` crate dependency is purely computational with no I/O.

---

## Workspace-Wide Audit Results

**Command 1:** `grep -rE "admin_token|secret|password|api_key" --include="*.rs" crates/ | grep -E "==|!=" | head -50`

Matches in production source (excluding `prism-dtu-harness` and test files):
- `prism-dtu-nvd/src/state.rs` — `if *mode == AuthMode::Reject && api_key.is_some()` — this is a state enum comparison, not a secret comparison. Not a finding.
- All other matches are in test files or in `prism-dtu-harness` (test-only scaffold).

No non-constant-time comparisons against actual secret values remain in production source.

**Command 2:** `grep -rE "ct_eq|ConstantTimeEq" --include="*.rs" crates/ | head -30`

Coverage confirmed across all 9 DTUs. Sites verified:
- armis: `src/routes/dtu.rs` (configure + reset) — 2 ct_eq calls
- claroty: `src/routes/devices.rs` (configure + reset) — 2 ct_eq calls
- crowdstrike: `src/routes/mod.rs` (configure + reset) — 2 ct_eq calls
- cyberint: `src/routes/dtu.rs` (configure + reset) — 2 ct_eq calls
- jira: `src/routes/dtu.rs` (configure + reset) — 2 ct_eq calls
- nvd: `src/routes/dtu.rs` (configure + reset) — 2 ct_eq calls
- pagerduty: `src/routes/dtu.rs` (configure + reset) — 2 ct_eq calls
- slack: `src/routes/dtu.rs` (configure + reset) — 2 ct_eq calls
- threatintel: `src/routes/lookup.rs` (configure) + `src/routes/dtu.rs` (reset) — 2 ct_eq calls

Total: 18 ct_eq call sites covering all admin token validation paths in the workspace.

---

## Summary of Open Items

| ID | Severity | Status | Description |
|----|----------|--------|-------------|
| CR-007 | LOW | DEFERRED | archetype/scale declared but unread in build() — Wave 4 |
| CR-008 | LOW | DEFERRED | Placeholder CloneState sentinel strings — Wave 4 |
| CR-009 | LOW | DEFERRED | Wall-clock startup assertion — Wave 4 |

No new open items from this pass.

---

## Convergence Verdict

`CONVERGENCE_REACHED`

All pass-4 findings are resolved (CR-021 MEDIUM, CR-022 LOW, CR-023 LOW), and the
cycle-2 carry-forward R1-001 (ThreatIntel `lookup.rs` ct_eq) is confirmed closed.
The workspace-wide admin token audit returns no remaining non-constant-time comparisons
in production source. Both PRs in scope (#124, #125) are clean: no new CRITICAL, HIGH,
MEDIUM, or LOW findings were identified. The wave-3 integration gate code review
3-clean-pass window has begun with this pass.
