---
document_type: adversarial-review-report
scope: PR-LEVEL
story_id: S-DEMO-QUERY-PUSHDOWN-001
pr_number: 173
pass_number: 14
cascade: PR-LEVEL (distinct from LOCAL; LOCAL converged at pass 11 @69aafcc7)
base_develop: "752e407a"
feature_head_at_review: "6583e419"
feature_head_after_fix_burst: "6835e4fa"
clean_strict: false
clean_pr_merge: true
streak_after: "0/3"
produced: 2026-06-06
authority: BC-5.39.001 D-779
---

# PR-LEVEL Adversary Pass 14 — S-DEMO-QUERY-PUSHDOWN-001

**Story:** S-DEMO-QUERY-PUSHDOWN-001 — Push-Down Query Fidelity (Phase B Lane 2)
**PR:** #173 (base develop@752e407a, head 6583e419 at review)
**Pass:** PR-LEVEL pass 14
**Date:** 2026-06-06

## Pass-13 Closure Verification

Pass-13 found OBS-P13-001 (vacuous e2e assertion) and OBS-P13-002 (evidence currency).
Both CLOSED: implementer removed vacuous assertion; demo-recorder refreshed evidence to
v2.7. Two serialized commits advanced feature HEAD ac75e84d → 6583e419. OBS-P13-003
(process-gap — adversary worktree path constraint) mitigated with absolute path protocol.

Closures verified at HEAD 6583e419:
- OBS-P13-001: vacuous assertion absent; surrounding test structure confirmed
  load-bearing for the named behavioral invariant.
- OBS-P13-002: evidence report at v2.7 with current test suite state.

## Adversary Pass 14 Findings

### F-P14-LOW-001 (LOW) — Evidence report cited stale feature-tip SHA (self-referential SHA regression / TD-VSDD-091)

**Finding ID:** F-P14-LOW-001
**Severity:** LOW
**Category:** Demo evidence / TD-VSDD-091 anti-volatile-pin

**Description:** The evidence report refreshed at pass-13 (as part of the OBS-P13-002
fix) cited the feature-tip SHA of the branch at the time of the refresh (6583e419)
as a "current feature HEAD" reference in the evidence header. This is a self-referential
volatile SHA — a forward-decaying reference that will be stale as soon as the feature
branch advances.

This is the TD-VSDD-091 anti-volatile-pin violation applied to demo evidence: citing
a mutable branch HEAD SHA in an evidence document creates a document that becomes
self-refuting on the next commit. It is structurally parallel to the class closed at
pass-5 (OBS-P05-001 stale test comment volatile SHA) and the class that motivated the
"permanent class fix" recorded in PR-LEVEL passes 1-9 (evidence-SHA-depin at f290a43d).

**CLEAN(strict):** no (1 LOW finding)
**CLEAN(PR-merge):** yes (no CRIT/HIGH/MED)

**Root cause:** The pass-13 demo-recorder evidence refresh (OBS-P13-002 fix) introduced
a new volatile feature-tip SHA cite in the process of updating the evidence. The
refresher used the current HEAD (6583e419) as a "version anchor" in the header section,
not realizing this created a new forward-decaying reference.

This is a recurrence of the evidence-SHA class — the same class that was closed via
"permanent class fix" at feature HEAD f290a43d → eab62613 in the early passes. The
permanent fix eliminated SHA pins from the evidence body and header, anchoring to stable
PR#/story-version references instead. The pass-13 refresh re-introduced a SHA pin in
the refresh process.

**Closure:** CLOSED by demo-recorder applying TD-VSDD-091 de-pin discipline:
- Removed forward-decaying feature-tip SHA (6583e419) from evidence report header.
- Anchored evidence to stable LOCAL-converged SHA (69aafcc7 — the LOCAL 3-CLEAN
  convergence HEAD, which is immutable and will not change) + story version v2.7.
- Evidence refreshed to v2.7 with stable references only.
- Feature HEAD 6583e419 → 6835e4fa (evidence-only change; no behavioral code change).

## Axes Checked

| Axis | Result | Notes |
|------|--------|-------|
| Correctness | PASS | All correctness findings remain closed |
| Evidence SHA discipline (TD-VSDD-091) | FAIL → FIXED | F-P14-LOW-001: volatile 6583e419 SHA replaced with stable LOCAL-converged 69aafcc7 + v2.7 |
| SAP-1 (catalog) | PASS | 71 rows; no unregistered event_type |
| SAP-2 (DTU↔TOML) | PASS | CrowdStrike + Armis confirmed |
| AC traceability (SAP-5) | PASS | ZERO dangling ACs |
| Security | PASS (CLEAR-TO-MERGE) | No new security surface |
| Demo evidence stability | PASS after fix | Stable PR#/story-version refs; no forward-decaying SHAs |

## Summary

**CLEAN(strict):** no (1 LOW F-P14-LOW-001 — volatile SHA in evidence header)
**CLEAN(PR-merge):** yes (no CRIT/HIGH/MED)
**Streak:** 0/3 (RESET — finding resets streak)
**Feature HEAD before fix:** 6583e419
**Feature HEAD after fix:** 6835e4fa (evidence de-pin; LOCAL-converged 69aafcc7 anchor + v2.7; no code change)
**Story version:** v2.7 (unchanged)
**Next step:** PR-LEVEL pass 15 (fresh streak on stable 6835e4fa)

## Codification Note — Evidence-SHA-Depin Recurrence

This is the second occurrence of evidence-SHA class introduction during a fix-burst
evidence refresh. The first occurrence was addressed with a "permanent class fix" at
eab62613 (pass-1 fix burst). The recurrence at pass-13/14 demonstrates that the
permanent fix must be remembered by the demo-recorder agent at every evidence refresh,
not just at initial creation.

Lesson: When any demo-recorder refreshes evidence during a fix-burst, the refresh MUST
check for newly-introduced volatile SHA pins before declaring the refresh complete.
The stable-reference pattern (PR#/story-version/LOCAL-converged-SHA) must be
re-applied at every refresh, not just at initial recording.
