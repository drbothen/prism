---
document_type: adversary-review
pass_id: 16
wave: wave-4
phase: 4.A
window_position: "0/3 (BLOCKED)"
producer: vsdd-factory:adversary
date: 2026-05-03
disposition: BLOCKED
---

# Adversary Pass 16 — Wave 4 Phase 4.A

## Tally

| Severity | Count |
|----------|-------|
| CRITICAL | 0 |
| HIGH | 2 |
| MEDIUM | 2 |
| LOW | 0 |
| INFO | 0 |
| **TOTAL** | **4** |

## Findings

### F-P16-H-001 — STORY-INDEX Full Story List per-row VP enumeration drift (6 rows)

**Severity:** HIGH
**Class:** POLICY 9 cascade gap — per-row VP enumeration vs frontmatter source-of-truth
**Site:** STORY-INDEX.md Full Story List table, rows S-4.02 / S-4.03 / S-4.04 / S-4.06 / S-4.07 / S-4.08

**Finding:** The Full Story List VP column for 6 Wave 4 stories listed only the pre-Wave-4 VPs. The Wave 4 ADR burst (2026-05-02) added VP-137..145; the story frontmatter `verification_properties:` arrays were updated during story remediation, but the STORY-INDEX Full Story List per-row VP cells were never propagated.

Pass 15 F-P15-H-002 fixed the STORY-INDEX *aggregate* counts (total_vps_assigned: 136→145; proptests 77→86) but did NOT fix the per-row VP enumerations in the Full Story List table. The aggregate fix masked the per-row gap.

**Affected rows (before fix):**

| Row | Had | Missing |
|-----|-----|---------|
| S-4.02 | VP-019 | VP-141, VP-142 |
| S-4.03 | VP-018 | VP-139, VP-140 |
| S-4.04 | VP-027 | VP-140 |
| S-4.06 | VP-052,053,054,060 | VP-138, VP-145 |
| S-4.07 | -- | VP-145 |
| S-4.08 | VP-044..047, VP-137, VP-144 | VP-143 |

**Resolution:** state-manager — VP cells corrected per story frontmatter source-of-truth. STORY-INDEX v1.98.

---

### F-P16-H-002 — ADR-015 + ADR-018 body Status H2 vs frontmatter version drift

**Severity:** HIGH
**Class:** Sister-file partial-fix regression — P12-XADR-A-H-001 pattern recurrence
**Site:** ADR-015 body `## Status` H2 section; ADR-018 body `## Status` H2 section

**Finding:** ADR-015 and ADR-018 body `## Status` H2 sections still read "PROPOSED v0.4" after the Pass 14 cascade bumped both ADR frontmatter versions to v0.5. This is a recurrence of the P12-XADR-A-H-001 partial-fix-regression pattern: the ADR frontmatter `version:` field was updated but the corresponding body `## Status` H2 prose line was not swept.

TD-VSDD-039 codified the ADR Status H2 vs frontmatter sync check as a pre-pass sweep methodology step. However, the Pass 14 cascade (which bumped ADR-015 and ADR-018 via the F-P14-M-001 enum tuple form cascade) did not include a Status H2 sweep for those two files — the sweep focused only on the primary site (ADR-013) per the dispatch. The cascade was partial with respect to Status H2 sync.

**Resolution:** architect — Status H2 synced in ADR-015 and ADR-018. ADR-015 v0.6, ADR-018 v0.6. ADR-016 v0.8 incidentally bumped for F-P16-M-001 in the same burst.

---

### F-P16-M-001 — VP-143 anchor asymmetry in ADR-016 §5.5

**Severity:** MEDIUM
**Class:** Stale secondary anchor claim vs VP-INDEX source-of-truth
**Site:** ADR-016 §5.5 Verification Properties paragraph

**Finding:** ADR-016 §5.5 stated VP-143 anchors to "S-4.08 (primary) and S-4.01 (secondary)". VP-INDEX.md line 164 lists VP-143 anchor as `S-4.08` only (no S-4.01 secondary). The "S-4.01 (secondary)" claim in ADR-016 §5.5 was stale and created an asymmetric cross-reference: the VP-INDEX is the authoritative source of anchor assignments, and it did not corroborate the secondary claim.

**Resolution:** architect — dropped "S-4.01 (secondary)" from ADR-016 §5.5; VP-143 now anchored solely to S-4.08 in both ADR-016 §5.5 and VP-INDEX. ADR-016 v0.8.

---

### F-P16-M-002 — Process-gap: TD-VSDD-039 textual checklist insufficient for ADR Status H2 sync

**Severity:** MEDIUM
**Class:** Process-gap codification — structural enforcement needed
**Site:** Pre-pass sweep methodology (TD-VSDD-039 checklist)

**Finding:** TD-VSDD-039 codified ADR Status H2 vs frontmatter drift as a textual checklist item. Despite this codification, F-P16-H-002 recurred: ADR-015 and ADR-018 had Status H2 drift after the Pass 14 cascade. The textual checklist was not sufficient to prevent a cascade-burst from missing the Status H2 sync step for files bumped via cascade (as opposed to files bumped as primary targets).

Root cause: the TD-VSDD-039 checklist is a human-readable reminder; it has no structural enforcement. A cascade dispatch that bumps ADR frontmatter versions must also sweep the corresponding body `## Status` H2 lines, but there is no hook that enforces this invariant.

**Resolution:** TD-VSDD-043 filed — recommends writing `validate-adr-status-sync.sh` that grep-extracts ADR frontmatter `version:` and body `## Status` H2 line and errors on mismatch. Add to pre-commit hooks for `.factory/specs/architecture/decisions/`.

---

## Observations

**Trajectory pattern (passes 13–16):** 4 consecutive passes at 2 HIGH but DIFFERENT defect classes each pass:
- Pass 13: 2H (S-4.02 CF keys; verification-architecture VP-053 module)
- Pass 14: 2H (audit-event terminology; BC future-date)
- Pass 15: 2H (cron-tick sister-text; STORY-INDEX aggregate cascade)
- Pass 16: 2H (STORY-INDEX per-row enumeration; ADR Status H2 sister-file drift)

Content stability is achieved — no ADR or story body logic findings. The recurring class is cross-document and cross-index propagation gaps. Each new sweep methodology codification (TD-VSDD-039 through TD-VSDD-043) closes one defect class but new propagation classes surface in subsequent passes. Structural enforcement (lint hooks) rather than textual checklists is the path to convergence.

## Novelty Assessment

| Field | Value |
|-------|-------|
| **Pass** | 16 |
| **New findings** | 3 (F-P16-H-001 per-row enumeration class; F-P16-M-001 VP-143 anchor asymmetry; F-P16-M-002 process-gap codification) |
| **Duplicate/variant findings** | 1 (F-P16-H-002 is a variant of P12-XADR-A-H-001 sister-file partial-fix regression pattern) |
| **Novelty score** | 0.75 (3 new / 4 total) |
| **Median severity** | 3.0 (2H + 2M; HIGH=4, MEDIUM=2; median=3.0) |
| **Trajectory** | 38→17→8→7→7→5→5→6→6→5→5→4→7→9→2→4 |
| **Verdict** | FINDINGS_REMAIN |

**Novelty narrative:** F-P16-H-001 is a NEW per-row enumeration drift class — Pass 15 F-P15-H-002 was aggregate-only; the per-row gap was undetected. F-P16-H-002 is a variant of the P12-XADR-A-H-001 sister-file partial-fix regression (duplicate class). F-P16-M-001 is a new VP anchor asymmetry instance. F-P16-M-002 is a new process-gap codification finding. 4 consecutive passes at 2 HIGH but different defect classes: content-stable, cross-doc propagation gaps continue to surface. Structural enforcement (lint hooks) needed to break the pattern.

## Resolution Burst Summary

All 4 findings addressed in a single burst:
- F-P16-H-001: state-manager — STORY-INDEX 6-row per-row VP enumeration corrected; v1.98
- F-P16-H-002: architect — ADR-015 Status H2 synced (v0.6); ADR-018 Status H2 synced (v0.6)
- F-P16-M-001: architect — ADR-016 §5.5 VP-143 secondary anchor dropped; v0.8
- F-P16-M-002: process-gap → TD-VSDD-043 filed (structural lint-hook recommendation)

**ARCH-INDEX v2.13 (ADR-015 v0.6, ADR-016 v0.8, ADR-018 v0.6 propagated). STORY-INDEX v1.98.**

**Window status: 0/3 (BLOCKED → REMEDIATED). Pass 17 required (window 1/3 attempt).**
