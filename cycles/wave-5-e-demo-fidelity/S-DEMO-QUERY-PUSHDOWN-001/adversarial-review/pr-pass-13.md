---
document_type: adversarial-review-report
scope: PR-LEVEL
story_id: S-DEMO-QUERY-PUSHDOWN-001
pr_number: 173
pass_number: 13
cascade: PR-LEVEL (distinct from LOCAL; LOCAL converged at pass 11 @69aafcc7)
base_develop: "752e407a"
feature_head_at_review: "ac75e84d"
feature_head_after_fix_burst: "6583e419"
clean_strict: false
clean_pr_merge: true
streak_after: "0/3"
produced: 2026-06-06
authority: BC-5.39.001 D-779
---

# PR-LEVEL Adversary Pass 13 — S-DEMO-QUERY-PUSHDOWN-001

**Story:** S-DEMO-QUERY-PUSHDOWN-001 — Push-Down Query Fidelity (Phase B Lane 2)
**PR:** #173 (base develop@752e407a, head ac75e84d at review)
**Pass:** PR-LEVEL pass 13
**Date:** 2026-06-06

## Pass-12 Closure Verification

Pass-12 was CLEAN(strict)=yes at HEAD ac75e84d; zero findings. Streak 0/3 → 1/3.

## Adversary Pass 13 Findings

### OBS-P13-001 (LOW) — Vacuous-but-redundant e2e assertion

**Finding ID:** OBS-P13-001
**Severity:** LOW
**Category:** Test strength / assertion quality

**Description:** An end-to-end assertion in the test suite performed a vacuous check —
asserting a condition that could pass even when the behavioral property it names was
violated. Specifically, the assertion was redundant given the surrounding test structure
and did not independently falsify the named behavioral invariant. Under the production-
grade default, every test assertion must independently falsify the named property.

**Impact:** Non-blocking correctness impact. The push-down behavior IS correct at HEAD
ac75e84d. The vacuous assertion merely provides weaker-than-named test coverage.

**CLEAN(strict):** no (1 LOW finding)
**CLEAN(PR-merge):** yes (no CRIT/HIGH/MED findings)

**Closure:** CLOSED by implementer. Vacuous assertion removed; the surrounding test
structure was confirmed to provide sufficient load-bearing coverage of the named
behavioral invariant without the redundant assertion. Feature HEAD ac75e84d → 6583e419
(via two serialized commits; commit 1: assertion removal; commit 2: demo-recorder
evidence refresh v2.7, updating evidence report to reflect the corrected test suite).

### OBS-P13-002 (LOW) — Evidence-report currency

**Finding ID:** OBS-P13-002
**Severity:** LOW
**Category:** Demo evidence / documentation currency

**Description:** The evidence report referenced test output that predated the pass-11
draft-comment sweep (ac75e84d). While the behavioral evidence was accurate, the report
contained test names/counts from the pre-ac75e84d state.

**Closure:** CLOSED by demo-recorder in the same fix-burst (evidence refresh commit;
feature HEAD updated to 6583e419). Evidence report refreshed to v2.7.

### OBS-P13-003 (PROCESS-GAP) — Adversary relative-glob path constraint from worktree

**Finding ID:** OBS-P13-003
**Severity:** PROCESS-GAP (non-finding; methodology note)
**Category:** Adversary tooling / path resolution

**Description:** The adversary could not read .factory/ artifacts using relative glob
patterns when operating from the feature worktree (.worktrees/S-DEMO-QUERY-PUSHDOWN-001).
The .factory/ directory is mounted as a git worktree at the repo root, not the feature
worktree. Relative paths from the worktree do not resolve to .factory/.

**Mitigation:** Orchestrator provided absolute paths to .factory/ artifacts for all
subsequent reads. This is a known constraint (same class as DRIFT-D904-002 worktree-
path-resolution issue). Non-blocking.

**Codification direction:** PR-LEVEL adversary dispatches on feature worktrees MUST
be given explicit absolute paths to .factory/ artifacts in the dispatch instructions.
This is a standing operational discipline, not a new finding class.

## Axes Checked

| Axis | Result | Notes |
|------|--------|-------|
| Correctness (BC-2.01.013, BC-2.11.007) | PASS | All correctness findings closed; push-down correct |
| Test strength | FAIL → FIXED | OBS-P13-001: vacuous assertion removed (implementer) |
| Evidence currency | FAIL → FIXED | OBS-P13-002: evidence refreshed to v2.7 (demo-recorder) |
| SAP-1 (catalog) | PASS | 71 rows; no unregistered event_type |
| SAP-2 (DTU↔TOML) | PASS | CrowdStrike + Armis parity confirmed |
| AC traceability (SAP-5) | PASS | ZERO dangling ACs |
| Security | PASS (CLEAR-TO-MERGE) | No new security surface |
| Adversary path resolution | PROCESS-GAP mitigated | OBS-P13-003: absolute .factory/ paths required |

## Summary

**CLEAN(strict):** no (2 LOW OBS-P13-001 + OBS-P13-002; 1 PROCESS-GAP OBS-P13-003)
**CLEAN(PR-merge):** yes (no CRIT/HIGH/MED)
**Streak:** 0/3 (RESET — findings reset streak from 1/3 → 0/3)
**Feature HEAD before fix:** ac75e84d
**Feature HEAD after fix:** 6583e419 (two serialized commits: assertion removal + evidence refresh v2.7)
**Story version:** v2.7 (unchanged)
**Next step:** PR-LEVEL pass 14 (fresh streak on 6583e419)
