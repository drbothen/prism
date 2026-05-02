---
document_type: gate-step-f-holdout-evaluation
level: ops
version: "1.0"
status: complete
producer: holdout-evaluator
timestamp: 2026-05-02T00:00:00Z
phase: 3
inputs: []
input-hash: "a7f0d37"
traces_to: prd.md
pass: 3
previous_review: gate-step-f-holdout-evaluation-pass2.md
---

# Gate Step F — Holdout Evaluation: Prism Wave 3 (Pass 3)

## Summary

| Metric | Value |
|--------|-------|
| Pass | 3 |
| Date | 2026-05-02 |
| Verdict | PASS |
| Mean satisfaction | 0.86 |
| Must-pass ratio | 26/30 ABOVE_BAR |
| Holdout scenario | HS-003 (multi-tenant) |
| Bar threshold | 0.85 / 26-of-30 |
| Result | ABOVE_BAR — holdout gate PASSES |

## Pass-2 Fix Verification

| Gap | Status | Closed By |
|----|--------|-----------|
| BC-3.2.002 CredentialStoreOrgId todo!() stubs | RESOLVED | W3-FIX-CREDS-001 PR #121 |
| /dtu/reset unauthenticated | RESOLVED | W3-FIX-SEC-002 PR #119 |
| Config validation hardening (E-CFG-019) | RESOLVED | W3-FIX-CODE-002 PR #120 |
| CR-010..015 hygiene bundle | RESOLVED | W3-FIX-CODE-004 PR #118 |

## Evaluation Against HS-003 Sub-Scenarios

HS-003 covers 7 multi-tenant isolation sub-scenarios (HS-003-01 through HS-003-07).
At pass-3 (develop@a7f0d374):

- HS-003-01 (OrgId/OrgSlug isolation): PASS — S-3.1.01..07 + S-3.1.06-ImplPhase
- HS-003-02 (DTU state segregation): PASS — S-3.2.01..08
- HS-003-03 (customer config schema): PASS — S-3.3.01..06 + W3-FIX-SEC-003 + W3-FIX-CODE-002
- HS-003-04 (test harness isolation): PASS — S-3.3.03..05 + S-3.4.01..05
- HS-003-05 (credential isolation): PASS — W3-FIX-CODE-003 + W3-FIX-CREDS-001
- HS-003-06 (admin endpoint auth): PASS — W3-FIX-SEC-001 + W3-FIX-SEC-002
- HS-003-07 (path traversal rejection): PASS — W3-FIX-SEC-003

Remaining concerns (below bar — not blocking at 0.86):
- BC-3.5.001/002 timing tests remain #[ignore] (TD-W3-TIMING-001 — formal BC amendment or Criterion migration needed)
- W3-FIX-CODE-005 (sibling endpoint + poll cleanup, 5 pts) queued for W3.3 fix wave
- W3-FIX-SEC-004 (TOML redaction edge cases + admin-token timing, 3 pts) queued for W3.3 fix wave

## Finding ID Convention

IDs use `HS-003-P3-<SEQ>` format for this pass.

## Novelty Assessment

| Field | Value |
|-------|-------|
| **Pass** | 3 |
| **New findings** | 2 (non-blocking — W3.3 fix wave items) |
| **Duplicate/variant findings** | 4 (resolved from pass-2) |
| **Novelty score** | 0.33 |
| **Median severity** | 3.5 |
| **Trajectory** | 0.71→0.75→0.86 (satisfaction) |
| **Verdict** | CONVERGENCE_REACHED (above 0.85 bar; 26/30 must-pass above threshold) |
