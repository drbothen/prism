---
document_type: adversarial-review
story: S-PLUGIN-PREREQ-E
pass: 17
scope: spec
verdict: BLOCKED
total_findings: 1
severity_breakdown:
  critical: 0
  high: 0
  medium: 1
  low: 0
  observation: 2
in_scope_findings: 1
observations_queued: 2
produced_by: adversary
reviewed_at: 2026-05-16
fix_burst: fix-burst-16
fix_burst_closed_at: pending
streak_after_pass: "0/3"
streak_before_pass: "0/3"
novelty: LOW-MEDIUM (8th manifestation BC-2.16.002 citation defect family; NEW dimension phrasing-form inconsistency)
trajectory: "14→9→8→9→10→10→FB6→8→FB7→4→FB8→CLEAN★(1/3)→BLOCKED(0/3)→FB9-CLOSED→...→FB15-CLOSED→BLOCKED(0/3)"
---

# S-PLUGIN-PREREQ-E Adversarial Spec Review — Pass 17

**Verdict: BLOCKED — 1 MEDIUM F-LP17-MED-001. Streak stays 0/3.**

FB15's POL-25 variant-phrasing grep mandate **partially succeeded**: it prevented the 8th occurrence of version-pin-staleness. It did NOT prevent phrasing-form-inconsistency — a NEW dimension of the same FB12-era root cause.

## FB15 Verification — ALL PIN-DIMENSION TARGETS PASS

| Target | Result |
|---|---|
| Story 3 sites at `v1.20` | PASS (pin dimension) |
| ADR-026 §D7 at `v1.20` | PASS |
| ADR-026 stays v1.10 (single-bump) | PASS |
| STORY-INDEX v2.114 + PREREQ-E row v1.10 | PASS |
| Workspace zero v1.[1-9].x stale pins | PASS |

But pass-17 fresh-context surfaces phrasing-form dimension still inconsistent:

## Finding Inventory

### F-LP17-MED-001 — POL-25 multi-cite propagation gap: 3 story sites use non-canonical no-parens phrasing form (8th manifestation)

**Severity:** MEDIUM
**Type:** POL-25 multi-cite propagation gap (phrasing-form dimension); 8th manifestation of BC-2.16.002 catalog citation defect family at NEW dimension
**Routing:** product-owner

**Evidence — non-canonical no-parens phrasing form at 3 story sites:**

1. Story Task 7 line 170: `BC-2.16.002 §Postconditions Canonical Structured Event Catalog v1.20 row 33`
2. Story AC-9 line 238: same pattern
3. Story §File Structure Requirements line 345: same pattern

**Evidence — canonical workspace parens-ancestry form at sister sites:**

- BC-2.16.012 line 84 (×2): `BC-2.16.002 §Postconditions (Canonical Structured Event Catalog bullet, v1.20) row 33`
- BC-2.16.012 line 109 EC-016-012-005: `BC-2.16.002 §Postconditions (Canonical Structured Event Catalog bullet, v1.20 row 33)`
- error-taxonomy line 467 (E-PLUGIN-020): canonical form
- error-taxonomy line 473 (E-PIPELINE-001): canonical form precedent

**Root cause inheritance chain:** FB12 PO POL-21 sweep at BC-2.16.012 converted bare-bullet-§ → canonical parens-ancestry form at BC-2.16.012 + error-taxonomy. SAME burst added 3 NEW story sites for Option A propagation but used no-parens form (pre-canonicalization phrasing). FB14/FB15 each closed only pin-staleness; never canonicalized phrasing form.

**Fix:** Convert 3 story sites to canonical parens-ancestry form. Story v1.10 → v1.11. STORY-INDEX row tag sync.

## Observations

### OBS-LP17-001 [process-gap] POL-29 codification scope expanded to phrasing-form discipline

Pass-16 OBS-LP16-001 recommended POL-29 codification for version-pin propagation. Pass-17 demonstrates the SAME root cause has phrasing-form dimension that pass-16 didn't anticipate. POL-29 cycle-close text should explicitly enumerate phrasing-form-canonicalization-on-introduction.

### OBS-LP17-002 — Pass-17 audit confirms FB15 pin-dimension closure correct

All pin-dimension verification targets PASS. The MEDIUM finding is phrasing-form dimension only (cosmetic/canonical-pattern, not anchor-resolution failure).

## Trajectory Summary

| Pass | In-Scope | Streak |
|------|----------|--------|
| 9 | 0 | 1/3 ★ |
| 10-17 | 1-3 each | 0/3 |

## Next Step

Fix-burst-16: PO single-burst converts 3 story sites to canonical parens-ancestry phrasing form. State-manager closes STORY-INDEX sync. Then pass-18.

Pass-17 report: `cycles/wave-4-operations/adversarial-reviews/S-PLUGIN-PREREQ-E-spec-pass-17.md` (this file).
