---
document_type: adversarial-review
story: S-PLUGIN-PREREQ-E
pass: 8
scope: spec
verdict: BLOCKED
total_findings: 4
severity_breakdown:
  critical: 0
  high: 2
  medium: 1
  low: 0
  observation: 1
in_scope_findings: 3
observations_queued: 1
produced_by: adversary
reviewed_at: 2026-05-16
fix_burst: fix-burst-8
fix_burst_closed_at: pending
streak_after_pass: "0/3"
---

# S-PLUGIN-PREREQ-E Adversarial Spec Review — Pass 8

**Verdict: BLOCKED — 3 in-scope findings (2 HIGH + 1 MEDIUM) + 1 OBS process-gap. Streak resets to 0/3.**

Pass-8 fresh-context surfaces a RECURRING within-FB sibling-sweep asymmetry defect class — same pattern as F-LP7-HIGH-001 but at a different junction:

1. **F-LP8-HIGH-001 (within-FB7 sibling-sweep asymmetry — VP-156):** ADR-026 was bumped v1.8 → v1.9 in FB7 D-586 (F-LP7-HIGH-004 + F-LP7-MED-002 runtime_deliverables completeness). In the SAME burst, architect swept VP-156's 4 live-narrative D7 pins v1.7→v1.8 to close F-LP7-HIGH-001. The sweep target was the intermediate-version snapshot (v1.8), NOT the final-version snapshot (v1.9). All 4 VP-156 live-narrative D7 pins now show v1.8 while ADR-026 is v1.9.

2. **F-LP8-HIGH-002 (companion site — BC-2.16.012):** Same defect class. BC-2.16.012 §Verification Properties VP-156 row pin "ADR-026 D7 v1.8" stale relative to ADR-026 v1.9. The within-FB6 sibling-sweep that updated this pin v1.7→v1.8 was correct against THAT burst's final state, but FB7's ADR-026 v1.8→v1.9 bump didn't sibling-sweep BC-2.16.012.

3. **F-LP8-MED-001 (within-FB7 monotonic-order miss — VP-156 §Changelog):** VP-156 §Changelog row v0.4 appears at the BOTTOM after v0.5 and v0.6. FB7 D-588 reordered ADR-026, ADR-027, VP-155 §Changelog tables to monotonic-strict-ascending (F-LP7-HIGH-003 close), but VP-156 §Changelog was not in the 3-file repair scope despite exhibiting the same defect-class.

4. **OBS-LP8-001 [process-gap]:** Within-FB sibling-sweep asymmetry is now a RECURRING defect class (FB5→FB6→FB7 = 3 consecutive bursts produced this pattern). POL-23 amendment candidate: require sweep targets reflect the FINAL post-burst version of the source artifact, not an intermediate snapshot.

CASCADE LENGTH NOTE: 8 passes deep. Trajectory **14→9→8→9→10→10→FB6-CLOSED→8→FB7-CLOSED→4**. Pass-8 count is LOWEST of cascade. The 3 in-scope findings are all sibling-sweep-asymmetry instances — defect-class novelty has decayed (pass-8 found NO new class, only recurrence). This is a positive convergence signal: the cascade is converging on a single residual defect-class. BC-5.39.001 3-CLEAN protocol: streak resets 0/3, pass-9 NEXT.

---

## Finding Inventory

### F-LP8-HIGH-001 — VP-156 4 live-narrative ADR-026 D7 pins stale at v1.8; ADR-026 is v1.9 (within-FB7 sibling-sweep asymmetry recurring class)

**Severity:** HIGH
**Type:** POL-23 within-burst version-pin sweep-target asymmetry; RECURRING defect class
**Anchor policies:** POL-23 (D-571 amendment), TD-VSDD-060
**Routing:** architect (advance VP-156 D7 pins v1.8 → v1.9)

**Evidence:**
- ADR-026 frontmatter `version: "1.9"`; §Changelog v1.9 row dated 2026-05-16 (FB7 D-586)
- VP-156 4 live-narrative cites of `ADR-026 D7 v1.8`: §Property Statement, §Source Contract BC row, §Source Contract ADR row, proof harness skeleton comment

**Defect class:** F-LP7-HIGH-001 caught FB6's BC-was-swept-VP-was-not asymmetry. FB7 closure of F-LP7-HIGH-001 advanced VP-156 D7 pins v1.7→v1.8. But SAME FB7 burst (D-586) also bumped ADR-026 v1.8→v1.9. The VP-156 sweep targeted v1.8 (intermediate snapshot), not v1.9 (final post-burst snapshot).

**Fix:** Architect advances all 4 VP-156 live-narrative D7 pins v1.8 → v1.9 in FB8. Bump VP-156 v0.6 → v0.7. Sync VP-INDEX row. ADR-026 itself MUST NOT bump in FB8 (single-bump-per-source-artifact discipline; avoid re-creating the asymmetry).

---

### F-LP8-HIGH-002 — BC-2.16.012 §Verification Properties VP-156 row pin stale at "ADR-026 D7 v1.8"; ADR-026 is v1.9

**Severity:** HIGH
**Type:** POL-23 within-burst version-pin sweep-target asymmetry (companion site of F-LP8-HIGH-001)
**Anchor policies:** POL-23
**Routing:** architect (advance BC-2.16.012 VP-156 row pin v1.8 → v1.9)

**Evidence:**
- BC-2.16.012 §Verification Properties VP-156 row: `... resolved behavior (error-on-duplicate, per ADR-026 D7 v1.8)...`
- ADR-026 frontmatter `version: "1.9"`
- BC-2.16.012 §Changelog top row v1.7 dated 2026-05-16 (FB6): pin updated v1.7→v1.8 (correct against THAT burst's final state)

**Fix:** Architect advances BC-2.16.012 §Verification Properties VP-156 row pin v1.8 → v1.9. Bump BC-2.16.012 v1.7 → v1.8.

---

### F-LP8-MED-001 — VP-156 §Changelog v0.4 row out-of-order (positioned at bottom after v0.5 and v0.6)

**Severity:** MEDIUM
**Type:** POL-20 monotonic-ascending §Changelog ordering; within-FB7 D-588 sibling-sweep miss
**Anchor policies:** POL-20
**Routing:** state-manager (cosmetic row reorder; no version bump)

**Evidence:**
VP-156 §Changelog displays: v0.1 → v0.2 → v0.3 → v0.5 → v0.6 → v0.4 (v0.4 at bottom; should be between v0.3 and v0.5 per the ascending convention).

**Fix:** State-manager reorders VP-156 §Changelog row v0.4 to correct chronological position (between v0.3 and v0.5). No version bump.

---

### OBS-LP8-001 — [process-gap] Within-FB sibling-sweep asymmetry RECURRING defect class; POL-23 amendment candidate

**Severity:** OBSERVATION (process-gap codification candidate)
**Disposition:** QUEUED-CYCLE-CLOSE

Pattern evidence:
- FB5 → FB6: F-LP5-HIGH-003 renumber-repair-redo (multi-file)
- FB6 → FB7: F-LP7-HIGH-001 BC-was-swept-VP-was-not
- FB7 → FB8 (pass-8): F-LP8-HIGH-001/002 VP-was-swept-to-intermediate-version + F-LP8-MED-001 changelog-reorder-missed-1-of-4-affected

Root cause hypothesis: when a fix-burst bumps source artifact version MORE THAN ONCE, sweep author's mental model anchors on the bump-target-version at sweep planning, not the burst's final state.

POL-23 amendment proposal: require sweep targets reflect FINAL post-burst version of source artifact.

---

## Trajectory Summary

| Pass | Findings | In-Scope | OBS Queued | Delta | Note |
|------|----------|----------|------------|-------|------|
| 1 | 14 | 12 | 2 | — | Initial: 1C+4H+5M+2L+2OBS |
| 2 | 9 | 8 | 1 | -5 | FB1 regressions caught |
| 3 | 8 | 8 | 0 | -1 | FB2 sibling-sweep regressions |
| 4 | 9 | 9 | 0 | +1 | FLAT |
| 5 | 10 | 7 | 3 | +1 | REGRESSION |
| 6 | 10 | 10 | 3 | 0 | FLAT, NOVEL classes |
| 7 | 8 | 8 | 4 | -2 | DECREASING |
| 8 | 4 | 3 | 1 | -5 | **LOWEST** — recurring class only, no new novelty |

Trajectory: **14→9→8→9→10→10→FB6→8→FB7→4**. Pass-8 in-scope count = 3 (lowest). Defect-class novelty has DECAYED — pass-8 found only recurrence of FB-sibling-sweep-asymmetry class.

---

## Artifact Versions After Pass-8 (Pre-Fix-Burst)

| Artifact | Pin | Expected FB8 Bump |
|----------|-----|-------|
| ADR-026 | v1.9 | (MUST NOT BUMP — single-bump-per-source-artifact discipline to avoid re-creating asymmetry) |
| BC-2.16.012 | v1.7 | v1.8 (F-LP8-HIGH-002 D7 pin v1.8→v1.9) |
| VP-156 | v0.6 | v0.7 (F-LP8-HIGH-001 D7 pin v1.8→v1.9) |
| VP-INDEX | v1.43 | v1.44 |
| BC-INDEX | v4.84 | v4.85 |

## Next Step

Fix-burst-8 dispatch: architect (F-LP8-HIGH-001 + F-LP8-HIGH-002) — advance VP-156 + BC-2.16.012 D7 pins to v1.9. State-manager (F-LP8-MED-001 + closure) — reorder VP-156 §Changelog + STATE bump.

**Critical discipline:** FB8 MUST NOT bump ADR-026 (the source artifact). Single-bump-per-source-artifact-per-burst rule prevents within-FB8 sibling-sweep asymmetry recurrence.

Then adversary pass-9 dispatch. BC-5.39.001 3-CLEAN protocol — streak resets 0/3.

Pass-8 report: `cycles/wave-4-operations/adversarial-reviews/S-PLUGIN-PREREQ-E-spec-pass-8.md` (this file).
