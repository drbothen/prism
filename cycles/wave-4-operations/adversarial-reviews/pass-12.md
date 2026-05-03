---
document_type: adversarial-review-pass
phase: 4.A
pass_number: 12
producer: adversary (verbatim findings reconstructed by state-manager)
timestamp: 2026-05-04T19:00:00Z
predecessor: pass-11.md (BLOCKED 5 findings; structural prevention adopted)
verdict: BLOCKED
findings_count: 4
severity_breakdown: { CRITICAL: 0, HIGH: 2, MEDIUM: 1, LOW: 1, OBS: 0 }
window_status: 0/3 (reset)
remediation_status: COMPLETED_2026-05-04
remediation_commits: [<Stage 1 SHA>]
---

# Adversarial Review — Wave 4 Phase 4.A Pass 12

**Verdict: BLOCKED**
**Findings: 4 (0C / 2H / 1M / 1L / 0OBS)**
**Trajectory: 38→17→8→7→7→5→5→6→6→5→5→4 (descent resumed after Pass 11 structural prevention)**

---

## F-P12-H-001 — ADR-013 body Status `v0.4` vs frontmatter `v0.5` — P4 pattern recurrence (HIGH)

**Severity:** HIGH
**Pattern:** P4 partial-fix regression recurrence — body Status line not updated when version was bumped.

**Finding:**
ADR-013 frontmatter declares `version: "0.5"` but the body Status section (line 57) still reads `Status: Proposed (v0.4)`. This is an exact recurrence of the P4 defect class (F-P4-ADR-A-H-001): the version was bumped in the frontmatter but the prose Status line was not propagated. The body also contains a secondary stale cite on line 65 (addressed in F-P12-H-002 below).

**Expected (post-fix):**
- ADR-013 line 57: `Status: Proposed (v0.5)` — body Status synchronized with frontmatter version.

**Root cause:** Partial-fix pattern — when Pass 8 bumped ADR-013 to v0.5, only the YAML frontmatter was updated; the inline `Status: Proposed (vX.Y)` prose line was not swept.

**Remediation:** Update ADR-013 line 57 to reflect `v0.5`. No version bump required; this is a body-sync correction within the existing v0.5 increment.

---

## F-P12-H-002 — ADR-013 line 65 SS-04 inline ref vs frontmatter `subsystems: [SS-12]` — P1 prose propagation gap (HIGH)

**Severity:** HIGH
**Pattern:** P1 prose propagation gap — inline subsystem reference in body not updated when frontmatter was changed.

**Finding:**
ADR-013 frontmatter `subsystems` field lists `[SS-12]` (Scheduler). However, line 65 in the body prose contains an inline reference to `SS-04` (a stale pre-remediation subsystem identifier). This contradicts the frontmatter declaration and would mislead an implementer reading only the body section.

**Expected (post-fix):**
- ADR-013 line 65: the `SS-04` reference removed or replaced so the body is consistent with `subsystems: [SS-12]` in the frontmatter. No version bump needed (body-sync within v0.5).

**Root cause:** Pass 1 remediation (P1) updated the frontmatter subsystems field from `SS-04` to `SS-12` per the correct architecture mapping, but the body prose at line 65 was not swept for the stale `SS-04` inline cite.

**Remediation:** Update or remove the `SS-04` reference at ADR-013 line 65.

---

## F-P12-M-001 — BC-2.12.004 fire-loop model contradiction with ADR-013 §2.5/§2.6 (MEDIUM)

**Severity:** MEDIUM
**Pattern:** Spec-to-spec contradiction — BC body describes a fire-loop model inconsistent with the canonical ADR.

**Finding:**
BC-2.12.004 (Schedule Execution Loop — Tick-Based with Splay and In-Flight Skip) version 1.5 contains body language describing the fire-loop execution model in terms that contradict ADR-013 §2.5 (tick semantics) and §2.6 (in-flight skip logic). Specifically:

1. BC-2.12.004 body implies a continuous-loop model where schedules are checked on every tick without the splay mechanism. ADR-013 §2.5 requires per-schedule splay jitter (±10% of interval) to prevent thundering-herd startup bursts.
2. BC-2.12.004 body omits the in-flight skip guard specified in ADR-013 §2.6 — if a schedule's prior execution is still running when the tick fires, the new execution must be skipped (not queued). The BC body language could be read as allowing queue accumulation.

**Expected (post-fix):**
BC-2.12.004 v1.6 body aligns execution loop model to ADR-013 §2.5 (splay jitter) and §2.6 (in-flight skip semantics). The EC (error condition) table should add an EC for skipped execution when in-flight guard fires.

**Remediation:** Bump BC-2.12.004 to v1.6 with body updates aligning fire-loop model to ADR-013 §2.5/§2.6.

---

## F-P12-L-001 — S-4.05 frontmatter `subsystems: [SS-13]` vs body `SS-13` + `SS-14` — LOW

**Severity:** LOW
**Pattern:** Frontmatter–body subsystem list inconsistency.

**Finding:**
S-4.05 (Alert Generation) frontmatter `subsystems` field lists only `[SS-13]` (Alert Manager). However, the story body (prior to Pass 11 remediation, line 242 area) referenced `SS-13` and `SS-14` (Notification Dispatcher). Pass 11 removed the SS-14 reference from line 242, but the frontmatter `subsystems` list still lists only `[SS-13]` — the frontmatter was already correct before Pass 11 and remains correct. This finding captures the residual verification burden: confirm that after SS-14 removal from the body, no remaining body prose still references SS-14 in a load-bearing way that would require frontmatter re-addition.

**Expected (post-fix):**
Confirm S-4.05 body is fully consistent with `subsystems: [SS-13]` — no residual SS-14 load-bearing references. If confirmed clean, close this finding as a verification pass. Frontmatter requires no change.

**Remediation:** S-4.05 body sweep for any remaining `SS-14` references post-Pass-11. If clean: no version bump needed beyond Pass 11 v1.9. If stale refs remain: bump to v1.10 with residual SS-14 references removed.

---

## Novelty Assessment

| Field | Value |
|-------|-------|
| **Pass** | 12 |
| **New findings** | 4 |
| **Duplicate/variant findings** | 0 |
| **Novelty score** | 4 / (4 + 0) = 1.0 |
| **Median severity** | 3.5 (2H + 1M + 1L) |
| **Trajectory** | 38→17→8→7→7→5→5→6→6→5→5→4 |
| **Verdict** | FINDINGS_REMAIN |

## Convergence Assessment

**Trajectory:** 38→17→8→7→7→5→5→6→6→5→5→4

Pass 12 continues the descent resumed after Pass 11's structural prevention (dropping version pins). The 4-finding set is bounded: 2 HIGH are body-sync issues (ADR-013 line 57 Status text + line 65 SS-04 inline ref), 1 MEDIUM is a BC-to-ADR alignment gap, 1 LOW is a residual verification burden from Pass 11. No new structural classes introduced.

**Pass 13 prognosis:** If F-P12-H-001/H-002/M-001 are remediated and F-P12-L-001 confirms clean, Pass 13 should reach 0–1 findings. The partial-fix regression treadmill pattern (4–7 findings per pass without structural progress) may be approaching natural floor if no new body-sync misses exist across the ADR/BC corpus.

**Strategic observation:** 12 passes consumed (~60 dispatches total). The trajectory 38→4 represents genuine convergence but the partial-fix regression treadmill (each fix exposing a sibling miss) suggests a proactive structural sweep of the ADR/BC corpus for (a) body Status version sync, (b) subsystem inline refs vs frontmatter, and (c) BC-to-ADR §-section alignment gaps would close the remaining floor faster than pass-by-pass remediation.

---

## Remediation Record

All 4 findings remediated 2026-05-04:

| Finding | Fix | Files Changed |
|---------|-----|---------------|
| F-P12-H-001 | ADR-013 line 57 body Status v0.4→v0.5 | ADR-013.md |
| F-P12-H-002 | ADR-013 line 65 SS-04 inline ref removed | ADR-013.md |
| F-P12-M-001 | BC-2.12.004 v1.5→v1.6 fire-loop model aligned to ADR-013 §2.5/§2.6 | BC-2.12.004.md |
| F-P12-L-001 | S-4.05 v1.9→v1.10 body sweep — SS-14 at line 242 confirmed removed; no residual SS-14 references | S-4.05.md |

Convergence window: reset to 0/3 (pass-12 BLOCKED). Pass 13 queued.
