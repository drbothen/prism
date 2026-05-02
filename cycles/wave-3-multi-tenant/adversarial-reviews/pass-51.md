---
document_type: adversarial-review
level: ops
version: "1.0"
status: complete
producer: adversary
timestamp: 2026-05-02T19:00:00Z
phase: 3
inputs:
  - .factory/specs/prd.md
  - .factory/specs/behavioral-contracts/**
  - .factory/stories/**
  - .factory/cycles/wave-3-multi-tenant/gate-step-c-code-review-pass4.md
  - .factory/cycles/wave-3-multi-tenant/gate-step-d-security-review-pass4.md
  - .factory/cycles/wave-3-multi-tenant/gate-step-e-consistency-validation-pass4.md
  - .factory/cycles/wave-3-multi-tenant/gate-step-f-holdout-evaluation-pass4.md
input-hash: "66c1215"
traces_to: prd.md
pass: 51
previous_review: pass-50.md
verdict: CLEAN_WITH_LOW
findings_critical: 0
findings_high: 0
findings_medium: 0
findings_low: 1
findings_observation: 4
findings_process_gap: 1
---

# Adversarial Review: Prism (Pass 51) — Wave 3 Integration Gate

**Scope:** develop@e4be29ae (W3-FIX-CODE-005 — Wave 3.3 final PR, post-W3.3 fix wave)
**Pass:** 51
**Verdict:** CLEAN_WITH_LOW — 0 CRITICAL, 0 HIGH, 0 MEDIUM, 1 LOW, 4 OBS, 1 PG

## Finding ID Convention

Finding IDs use the format: `ADV-W3GATE-P51-<SEV>-<SEQ>`

## Part A — Fix Verification

| ID | Previous Severity | Status | Notes |
|----|-------------------|--------|-------|
| SEC-P3-001 | MEDIUM | RESOLVED | W3-FIX-SEC-004 PR #122 (4e053105) — TOML inline-table redaction |
| SEC-P3-002 | MEDIUM | RESOLVED | W3-FIX-SEC-004 PR #122 — pipe-finder anchor |
| SEC-P3-003 | MEDIUM | RESOLVED | W3-FIX-SEC-004 PR #122 — subtle::ConstantTimeEq for Armis/Claroty/CrowdStrike/Slack |
| CR-016 | MEDIUM | RESOLVED | W3-FIX-CODE-005 PR #123 (e4be29ae) — poll_test_hook cadence 50ms in all 3 clone files |
| CR-017 | MEDIUM | RESOLVED | W3-FIX-CODE-005 PR #123 — Armis org-id guard on tags/alerts/activity/risk |
| CR-018 | MEDIUM | RESOLVED | W3-FIX-CODE-005 PR #123 — CrowdStrike detections nil-instance guard |
| CR-019 | LOW | RESOLVED | W3-FIX-SEC-004 PR #122 — find_snippet_pipe anchored |
| CR-020 | LOW | RESOLVED | W3-FIX-CODE-005 PR #123 — validate_spec_path visibility documented |
| L-50-004 | LOW | RESOLVED | W3-FIX-CODE-005 PR #123 — TD-W3-POLL-NOTIFY-001 documented |
| M-50-001 | MEDIUM | RESOLVED | W3-FIX-CODE-005 PR #123 — Armis dual-mode fix |

## Part B — New Findings

### CRITICAL

_None._

### HIGH

_None._

### MEDIUM

_None._

### LOW

#### ADV-W3GATE-P51-L-001: Cyberint `post_reset` admin token gap — scope omission from W3-FIX-SEC-002

- **Severity:** LOW
- **Category:** spec-fidelity
- **Location:** `crates/prism-dtu-cyberint/src/routes/dtu.rs:61-72`
- **BC Reference:** BC-3.5.002 precondition 6
- **Description:** W3-FIX-SEC-002 (PR #119, f89e7044) gated `POST /dtu/reset` with an
  admin token check in Armis, Claroty, CrowdStrike, and Slack. Cyberint's `post_reset`
  (lines 61-72) has no corresponding guard. The code reviewer (gate-step-c pass-4, CR-021)
  independently flagged this as MEDIUM. The adversary assesses this as LOW given that (a)
  all Cyberint DTU endpoints are `#[cfg(feature="dtu")]`-gated for test isolation and (b)
  the threat model for DTU endpoints is controlled test environments. However, the code
  reviewer's MEDIUM designation is noted and the combined gate coordinator should adjudicate;
  the more conservative MEDIUM from independent code review governs the combined gate verdict.
- **Evidence:** `crates/prism-dtu-cyberint/src/routes/dtu.rs:61-72` has no admin token
  check. `ac_8_reset_semantics.rs` calls `POST /dtu/reset` without `X-Admin-Token` and
  expects HTTP 200. Armis, Claroty, CrowdStrike, Slack all gate this endpoint.

### OBSERVATIONS

#### O-51-001: Cyberint `post_configure` ct_eq pattern not applied (CR-022)

- **Severity:** OBS
- **Category:** pattern-consistency
- **Description:** W3-FIX-SEC-004 applied `subtle::ConstantTimeEq` to 4 DTUs but not
  Cyberint. Cyberint's `post_configure` retains the `!=` comparison. Code reviewer CR-022
  flags this LOW. Pattern uniformity is desirable and the two fixes are naturally bundled
  with CR-021 in a W3.4 fix story.

#### O-51-002: Armis `get_device_activity` / `get_device_risk` guard test coverage gap (CR-023)

- **Severity:** OBS
- **Category:** test-coverage
- **Description:** `get_device_activity` and `get_device_risk` received the org-id guard
  in W3-FIX-CODE-005 but `cr017_tag_alert_org_id_guard.rs` does not test these endpoints.
  Code reviewer CR-023 flags LOW. Test coverage for the guard is absent for 2 of the 5
  Armis endpoints that received it.

#### O-51-003: Consistency-validator carry-over WGCV3-P3-007 still open

- **Severity:** OBS
- **Category:** state-hygiene
- **Description:** WGCV3-P3-007 (STORY-INDEX epic-view BC column for W3-FIX-CODE-002
  lists BC-3.2.005 but story frontmatter does not) remains OPEN as non-blocking carry-over.
  gate-step-e-consistency-validation-pass3.md verdict was CONDITIONAL_PASS;
  gate-step-f-holdout-evaluation-pass3.md verdict was PASS at 0.86 / 26-of-30.
  Requires hygiene burst W3.4-G to close.

#### O-51-004: Cycle-manifest adversarial-passes count stale

- **Severity:** OBS
- **Category:** state-hygiene
- **Description:** cycle-manifest.md line 25 states `47 Phase 3.A spec passes + integration
  gate passes (in progress)`. This should now reflect 51 total gate passes. Defer to
  W3.4-G hygiene burst.

### PROCESS GAPS

#### PG-51-001: 5-DTU admin-token sibling gap not caught during W3-FIX-SEC-002 story review

- **Severity:** PG
- **Category:** story-scope-definition
- **Description:** W3-FIX-SEC-002 story AC-001 listed Armis, Claroty, CrowdStrike, and
  Slack explicitly but did not enumerate all DTUs in the workspace. The story writer and
  reviewer did not verify exhaustive DTU coverage. Post-merge independent code review
  (gate-step-c pass-4) identified Cyberint as missed. A story acceptance criterion pattern
  requiring "verify all DTU crates in workspace" would prevent recurrence.

## Sub-reviewer Gate Verdicts

| Reviewer | Pass 4 Verdict | Key Findings |
|----------|----------------|--------------|
| Code Reviewer | APPROVE_WITH_CONCERNS | CR-021 MEDIUM, CR-022 LOW, CR-023 LOW |
| Security Reviewer | APPROVED | 0 findings |
| Consistency Validator | PASS | WGCV3-P3-007 carry-over LOW (non-blocking) |
| Holdout Evaluator | PASS | mean_satisfaction: 0.886, must_pass_ratio: 27/30 ABOVE_BAR |

## Summary

| Severity | Count |
|----------|-------|
| CRITICAL | 0 |
| HIGH | 0 |
| MEDIUM | 0 |
| LOW | 1 |
| Observations | 4 |
| Process Gaps | 1 |

**Overall Assessment:** CLEAN_WITH_LOW (adversary: 0 HIGH/MEDIUM; code reviewer independently found CR-021 MEDIUM — combined gate NOT_CLEAN)
**Convergence:** NOT_CLEAN — combined gate verdict defers to code-reviewer MEDIUM (CR-021). W3.4 fix wave required before next clean-pass attempt.
**Readiness:** File W3-FIX-SEC-005 (5-DTU admin-token uniformity: cyberint+jira+nvd+pagerduty+threatintel post_configure + post_reset), W3-FIX-CODE-006 (CR-023 test coverage), W3.4-G hygiene burst; then dispatch pass-52.

## Novelty Assessment

| Field | Value |
|-------|-------|
| **Pass** | 51 |
| **New findings** | 1L + 4OBS + 1PG |
| **Duplicate/variant findings** | 0 (CR-021/022/023 are new gaps discovered independently of pass-50) |
| **Novelty score** | 1.0 (all new — pass-50 findings fully resolved; Cyberint admin-token gap is new class) |
| **Median severity** | LOW |
| **Trajectory** | 9→1L+4OBS |
| **Verdict** | FINDINGS_REMAIN |
