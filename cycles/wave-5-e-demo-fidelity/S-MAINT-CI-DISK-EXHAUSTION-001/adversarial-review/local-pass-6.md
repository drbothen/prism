---
document_type: adversarial-review
scope: LOCAL
story_id: S-MAINT-CI-DISK-EXHAUSTION-001
passes: [6]
feature_head_at_review: 0d1add9f
date: 2026-07-15
clean_strict: false
clean_pr_merge: false
finding_counts:
  total: 3
  crit: 0
  high: 0
  med: 1
  low: 1
  obs: 1
  process_gap: 0
streak_after: 0/3
convergence: NOT_CONVERGED
authored_by: orchestrator-relay
---

# LOCAL Adversary Pass 6 — S-MAINT-CI-DISK-EXHAUSTION-001

---

## Pass 6 (frozen 0d1add9f; fresh-context adversary; CI disk-exhaustion hardening; streak 1/3 → RESET 0/3)

**Pass result:** CLEAN(strict)=NO, CLEAN(PR-merge)=NO

**Findings:** 3 total (0 CRIT / 0 HIGH / 1 MED / 1 LOW / 1 OBS / 0 PROCESS-GAP)

**STREAK RESET: 1/3 → 0/3** — 1 MED finding; novelty MEDIUM (spec self-contradiction surviving from v0.6 exclusion-list).

**Code HEAD at review:** 0d1add9f (SAME frozen HEAD as pass-5; fresh adversary dispatched against unchanged HEAD per BC-5.39.001)

**CLEAN(strict):** NO — 1 MED + 1 LOW + 1 OBS findings present

**CLEAN(PR-merge):** NO — 1 MED merge-blocking

---

## Finding Register

### F-CIDISK-P6-MED-001 [MED] §Architecture Compliance Rules section retains test-no-default-features in exclusion list — spec self-contradiction with v0.6 AC-001/AC-002 scope

**Severity:** MED

**Classification:** spec self-contradiction — story body contains two incompatible statements about the same target

**Description:**
Story v0.6 correctly scoped `test-no-default-features` into AC-001 and AC-002 (closing F-CIDISK-P4-MED-002). However, the story's `§Architecture Compliance Rules` section (or equivalent "do not modify" / exclusion-list prose) still listed `test-no-default-features` as a job that must NOT receive new steps. The v0.6 spec body had two contradictory directives for the same job:

1. AC-001/AC-002: MUST add disk-hardening steps to both `linux-test` AND `test-no-default-features`.
2. §Architecture Compliance Rules: MUST NOT add new steps to `test-no-default-features`.

A fresh-context reader of v0.6 would be blocked by this contradiction. The spec is internally inconsistent and would confuse implementers or reviewers re-reading from scratch.

**Fix required:** Remove `test-no-default-features` from the §Architecture Compliance Rules exclusion list (or equivalent "do not modify" annotation). The controlling statement is AC-001/AC-002 scope — the exclusion-list entry is the stale residue from pre-v0.6 scope.

---

### F-CIDISK-P6-LOW-001 [LOW] Story summary echo count mismatch — text claims 9 assertions, actual count is 11

**Severity:** LOW

**Classification:** documentation accuracy — stale count in narrative summary

**Description:**
The story's narrative summary (or run-block summary echo) cited "9 assertions" in the verify-workflow-structure run-block. The actual assertion count at v0.6 is 11: AC-001 (1 count≥2), AC-002 (1 count≥2), AC-003 (2 config-invariant assertions), AC-004 (1 annotation), AC-005 narrative note, plus the 5 existing assertions carried from prior versions (AC-007 semver-checks, AC-008 no-default-features). The count was never updated when AC-003 expanded from 1 to 2 assertions.

**Fix required:** Update the summary echo and narrative count to reflect the correct total (11 assertions in the run-block + 2 in the config-invariant step = 13 total) per the v0.6 Red Gate test count. The code-authoritative count from the ci.yml run-block exit-0 check in fix-burst-4 is the ground truth.

---

### F-CIDISK-P6-OBS-002 [OBS] Annotation step uses `df` without `-P` flag — style inconsistency with the preflight + gate steps

**Severity:** OBS

**Classification:** documentation style — minor inconsistency in the failure-annotation step prose

**Description:**
The failure annotation step (`if: failure()` block) used `df -h` (human-readable, locale-dependent column headers) while all other disk-measurement steps in the story used `df -P` (POSIX, predictable column positions). The inconsistency is cosmetic — the annotation step emits a warning rather than performing a numeric check — but creates a style inconsistency that future readers may interpret as intentional.

**Fix required:** Replace `df -h` with `df -P` in the annotation step prose/spec. Consistent `df -P` usage across all steps.

---

## Fix-Burst 5 Closure Audit

All 3 findings closed in fix-burst-5 via PO + implementer:

**PO adjudications — story v0.6→v0.7:**
- F-CIDISK-P6-MED-001: `test-no-default-features` removed from §Architecture Compliance Rules exclusion list; AC-001/AC-002 scope confirmed authoritative
- F-CIDISK-P6-LOW-001: Summary echo updated to enumerate correct count (11+2=13); code-authoritative arithmetic documented
- F-CIDISK-P6-OBS-002: Annotation step prose corrected to `df -P` throughout

**implementer @22cb83ad:**
- `test-no-default-features` removed from any exclusion-list reference in ci.yml
- Summary echo updated to enumerate 13 assertions
- Annotation step `df` → `df -P` in both Linux job annotation blocks

**Result after FB-5:** HEAD @22cb83ad on maintenance/ci-disk-hardening (LOCAL-ONLY; not pushed). Streak RESET 0/3 (FB-5 commit advances HEAD). Pass-7 dispatched.

---

## Standing Probe Results

**SAP-1:** N/A — `.github/workflows/ci.yml` only; no `event_type =` assignments.

**SAP-2:** N/A — no sensor TOML spec modifications.

**SID-1:** N/A — verify-workflow-structure bash assertions, not `#[ignore]`'d Rust tests.

---

## Convergence Assessment

**Pass 6 on frozen 0d1add9f:** NOT CLEAN strict (1 MED + 1 LOW + 1 OBS); novelty MEDIUM (spec self-contradiction); streak RESET 1/3 → 0/3.

**Cascade tally at FB-5 close:** 6 passes / 5 fix-bursts.

**New HEAD after FB-5:** @22cb83ad (LOCAL-ONLY; not pushed).

**NEXT:** LOCAL pass 7 on frozen @22cb83ad (streak 0/3; BC-5.39.001 requires 3 consecutive CLEAN(strict) passes).
