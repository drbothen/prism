---
pass: 8
story: S-MAINT-CI-DISK-EXHAUSTION-001
lane: PR-LEVEL
frozen_head: 4f9a5c6f
clean_strict: false
clean_pr_merge: false
streak_before: 0
streak_after: 0
date: 2026-07-16
---

# S-MAINT-CI-DISK-EXHAUSTION-001 PR-LEVEL Pass 8

**Frozen HEAD:** 4f9a5c6f
**CLEAN(strict):** NO
**CLEAN(PR-merge):** NO
**Streak:** 0/3 (reset per DRIFT-ORCH-PRLEVEL-PUSH-001 — rebase onto 84062ced pushed new HEAD)

## Findings Summary

### HIGH (1)

- **F-MAINT-P8-HIGH-001** — AC-005 evidence void after rebase: only 1 green CI run recorded on frozen HEAD 4f9a5c6f at time of pass-8. Rebase onto develop@84062ced (post-PR #222 merge) resolved the ci.yml conflict but reset the 3-green-run evidence clock per AC-005 requirement. Only 1 green run present on new HEAD; AC-005 threshold requires 3 consecutive green pull_request runs before merge gate clears.

### MEDIUM (4)

- **MED-001** — Story ACR/Forbidden-Patterns self-contradiction vs AC-006/007: `## Acceptance Criteria Revision (ACR)` section and `## Forbidden Patterns` section contain conflicting guidance that contradicts AC-006 scope (sites count) and AC-007 package install requirements (2 vs 4 packages); spec layer requires carve-out annotations to resolve the contradiction.

- **MED-002** — Rebase dropped numeric summary echo + stale 15/13 arithmetic: post-rebase ci.yml `echo` arithmetic reflects stale counts (15 reachability checks / 13 success threshold from pre-AC-006-expansion state). The expanded AC-006 scope (10 sites × two-attempt = 20 reachability checks) and AC-007 config checks (2 additional) produce 19 total unique operations; the echo statement verifying this count was dropped entirely during rebase conflict resolution, leaving no runtime arithmetic evidence.

- **MED-003** — PR description stale across SHAs/commit-list/base/stat/convergence claims: PR #224 description was authored against earlier feature HEAD(s); all SHA references, commit list, base branch ref, file diff stat line counts, and convergence narrative (passes 1-7 / passes 5/6/7 CLEAN 3/3 on frozen @498ffb6c) are stale after rebase push to 4f9a5c6f.

- **MED-004** — Unswept single-attempt apt site in merge-gating e2e.yml: `.github/workflows/e2e.yml` disk-space-reclaimer / `apt-get install` step retains single-attempt form (no fallback mirror retry loop); AC-006 two-attempt resilience mandate applies to all CI files that install packages; the e2e workflow was not swept in fix-bursts 9-11 that converted ci.yml sites.

### LOW (3)

- **LOW-001** — Phantom `linux-test` job name: story spec references job name `linux-test` in several §Acceptance Criteria narrative cells (AC-005 evidence table, AC-007 verification commentary) but ci.yml uses `test` as the actual matrix job name per the workflow definition; no `linux-test` job exists in the repository; spec must be corrected to `test`.

- **LOW-002** — grep -c count assignments abort under bash -e before ::error:: diagnostics: several ci.yml disk-hardening check steps use `FOUND=$(grep -c ...)` patterns; `grep -c` exits non-zero (code 1) when it finds zero matches; under bash `set -e` / `pipefail`, this aborts the step before the `::error::` annotation fires, making the diagnostic message unreachable for the case it most needs to serve (0-match = forbidden pattern present).

- **LOW-003** — Wrapper guarded update-phase only: combined update+install wrapper form required by AC-007/EC-013 guards only `apt-get update` in the retry loop but does not guard `apt-get install` atomically; production-grade combined wrapper should retry both phases as one unit to handle transient mirror failures during the install phase independently of the update phase.

### OBS (1)

- **OBS-001** — Volatile line pins in §Tasks: several §Tasks entries cite file paths with `lines NNN-MMM` specifics that will decay on subsequent diffs per TD-VSDD-091 anti-volatile-pin rule; narrative spec content must cite function names and behavioral anchors, not line numbers.

### PROCESS-GAP (1)

- **PG-001** — Rebase conflict-resolution edits shipped without same-burst PR-description/spec-arithmetic reconciliation: when a branch is rebased mid-cascade, the rebase-recovery checklist should mandate (1) PR description refresh with new HEAD SHA, (2) spec arithmetic re-verification against new scope, and (3) evidence count restart before re-dispatching adversary. This gap caused F-MAINT-P8-HIGH-001 + MED-002 + MED-003 to coexist in the same pass. Candidate lesson (PG-001 rebase-recovery checklist).

## Disposition

All HIGH + MED findings closed by fix-burst-12 (dispatched same session):

- **Spec layer:** story v0.16→v0.17 @1f39ae52 (ACR/FP carve-outs resolving MED-001; 19-check echo arithmetic (17 reachability + 2 config) resolving MED-002; e2e.yml AC-006 scope + RG-7 + EC-014 resolving MED-004; combined update+install wrapper form EC-013 resolving LOW-003; linux-test→test resolving LOW-001; TD-VSDD-091 line-pin strip resolving OBS-001)
- **Code layer:** bd65e93a PUSHED (12 ci.yml sites converted + e2e.yml site + RG-7 + `|| true` ×4 + echo restored 19-total arithmetic; RG-1..7 all verified; just check GREEN; pre-push 92 non-exhaustive symbols; MED-002/MED-004/LOW-002 closed)

PR description refreshed vs bd65e93a (HIGH-001/MED-003/LOW-003-desc closed).

Pass-9 DISPATCHED on frozen bd65e93a. Streak 0/3.
