---
document_type: adversarial-review
level: ops
version: "1.0"
status: complete
producer: vsdd-factory:adversary
timestamp: 2026-05-03T00:00:00Z
phase: 4.A
inputs:
  - specs/architecture/decisions/ADR-013-schedule-execution-semantics.md
  - specs/architecture/decisions/ADR-015-detection-rule-language.md
  - specs/architecture/decisions/ADR-016-action-delivery-framework.md
  - specs/architecture/decisions/ADR-017-case-lifecycle-invariants.md
  - specs/architecture/decisions/ADR-018-differential-result-pack-format.md
  - specs/architecture/decisions/ADR-019-siem-output-formats.md
  - stories/S-4.01-schedule-crud.md
  - stories/S-4.02-diff-results-packs.md
  - stories/S-4.05-alert-generation.md
  - stories/S-4.06-case-management.md
  - stories/STORY-INDEX.md
input-hash: "959a244"
traces_to: cycles/wave-4-operations/cycle-manifest.md
pass: 17
previous_review: cycles/wave-4-operations/adversarial-reviews/pass-16.md
wave: wave-4-operations
window_position: "0/3 (BLOCKED)"
disposition: BLOCKED
date: 2026-05-03
---

# Adversary Pass 17 — Wave 4 Phase 4.A

**Disposition:** BLOCKED (1 HIGH + 2 MEDIUM). Remediated in same burst. Ready for Pass 18 (window 1/3 attempt).

## Finding ID Convention

Finding IDs use the format: `F-P17-<SEV>-<SEQ>` (wave-4-operations Phase 4.A pass 17).

## Tally

| Severity | Count |
|----------|-------|
| CRITICAL | 0 |
| HIGH | 1 |
| MEDIUM | 2 |
| LOW | 0 |
| INFO | 0 |
| **TOTAL** | **3** |

**Overall Assessment:** block
**Convergence:** FINDINGS_REMAIN — Pass 18 required (window 1/3 attempt)
**Readiness:** requires revision

## Part A — Fix Verification

All Pass 16 findings verified as RESOLVED:

| ID | Previous Severity | Status | Notes |
|----|-------------------|--------|-------|
| F-P16-H-001 | HIGH | RESOLVED | STORY-INDEX 6-row VP enumeration corrected; v1.98 confirmed |
| F-P16-H-002 | HIGH | RESOLVED | ADR-015 v0.6 + ADR-018 v0.6 Status H2 synced; confirmed |
| F-P16-M-001 | MEDIUM | RESOLVED | ADR-016 §5.5 VP-143 anchor corrected to S-4.08 only; v0.8 confirmed |
| F-P16-M-002 | MEDIUM | RESOLVED | TD-VSDD-043 filed; process-gap codified |

## Part B — New Findings

### HIGH

#### F-P17-H-001 — STORY-INDEX W4 ADR Title-Annotation Drift (3 rows)

- **Severity:** HIGH
- **Substance:** SUBSTANTIVE (would route implementers to wrong ADR)
- **Category:** spec-fidelity
- **Class:** STORY-INDEX index↔frontmatter ADR-anchor sync (NEW class)
- **Location:** STORY-INDEX Full Story List rows S-4.02, S-4.05, S-4.06
- **Description:** Three W4 rows in the Full Story List carry incorrect ADR title-annotations that contradict each story's `anchor_adrs:` frontmatter. An implementer reading the STORY-INDEX would be routed to the wrong ADR for each story.
- **Evidence:**

  | Row | Observed in STORY-INDEX | Required (from frontmatter) |
  |-----|------------------------|------------------------------|
  | S-4.02 | `[v1.11 ADR-015]` | `[v1.11 ADR-018]` |
  | S-4.05 | `[v1.12 ADR-016]` | `[v1.12 ADR-015]` |
  | S-4.06 | `[v1.13 ADR-017,ADR-019]` | `[v1.13 ADR-017]` |

  - S-4.02 anchor is Differential Result Pack Format (ADR-018), not Detection Rule Language (ADR-015)
  - S-4.05 anchor is Detection Rule Language (ADR-015), not Action Delivery Framework (ADR-016); the v1.12 fix touched action_state CF but did not change anchor_adrs
  - S-4.06 frontmatter anchor_adrs has only ADR-017; ADR-019 was over-claimed in the STORY-INDEX annotation

- **Proposed Fix:** state-manager corrects 3 rows per frontmatter source-of-truth; bump STORY-INDEX version.

### MEDIUM

#### F-P17-M-001 — ADR-016 / ADR-017 Frontmatter Date Stale

- **Severity:** MEDIUM
- **Substance:** COSMETIC (no implementation impact)
- **Category:** spec-fidelity
- **Class:** ADR frontmatter date sync
- **Location:** ADR-016 frontmatter `date:`, ADR-017 frontmatter `date:`
- **Description:** Both ADRs have `date: 2026-05-02` in frontmatter while body content reflects edits made on 2026-05-03. Frontmatter date should match the last substantive edit date.
- **Evidence:** ADR-016 and ADR-017 frontmatter `date:` field reads 2026-05-02; body changelog entries reference 2026-05-03 edits.
- **Proposed Fix:** architect updates frontmatter `date:` to 2026-05-03 in both files; bump versions.

#### F-P17-M-002 — STORY-INDEX VP Assignment Matrix Structurally Missing W3/W4 VPs

- **Severity:** MEDIUM
- **Substance:** COSMETIC (matrix is secondary cross-reference; implementation unaffected)
- **Category:** coverage-gap
- **Class:** STORY-INDEX VP Assignment Matrix structural gap
- **Location:** STORY-INDEX VP Assignment Matrix section
- **Description:** The VP Assignment Matrix stops at VP-062. Wave 3 added VP-063..VP-136 and Wave 4 added VP-137..VP-145. These 83 VPs have no rows in the matrix. The gap is structural — the matrix section requires a major rebuild, not a targeted fix.
- **Evidence:** VP-INDEX shows VPs through VP-145; STORY-INDEX matrix has no rows beyond VP-062.
- **Proposed Fix:** Defer to post-Phase-4.A convergence cleanup or Wave 5 baseline. File TD-VSDD-045.

## Summary

| Severity | Count |
|----------|-------|
| CRITICAL | 0 |
| HIGH | 1 |
| MEDIUM | 2 |
| LOW | 0 |

**Overall Assessment:** block
**Convergence:** FINDINGS_REMAIN — Pass 18 required
**Readiness:** requires revision

## Novelty Assessment

| Field | Value |
|-------|-------|
| **Pass** | 17 |
| **New findings** | 2 (F-P17-H-001 STORY-INDEX ADR-anchor sync class; F-P17-M-002 VP Assignment Matrix structural gap) |
| **Duplicate/variant findings** | 1 (F-P17-M-001 is a variant of the recurring ADR frontmatter date sync class) |
| **Novelty score** | 0.67 (2 new / 3 total) |
| **Median severity** | 3.0 (1H + 2M; HIGH=4, MEDIUM=2; median=3.0) |
| **Trajectory** | 38→17→8→7→7→5→5→6→6→5→5→4→7→9→2→4→3 |
| **Verdict** | FINDINGS_REMAIN |

**Novelty narrative:** F-P17-H-001 is a NEW defect class — STORY-INDEX Full Story List ADR title-annotation drift (index↔frontmatter ADR-anchor sync). Distinct from Pass 16's F-P16-H-002 (ADR Status H2 sync) and F-P16-H-001 (VP enumeration drift). F-P17-M-001 is a known class (ADR frontmatter date sync); COSMETIC; duplicate variant. F-P17-M-002 is a new structural gap class in STORY-INDEX VP Assignment Matrix; properly deferred. HIGH count declining across 5 passes: 2→2→2→2→1. Pass 17 introduces explicit SUBSTANTIVE vs COSMETIC classification to improve triage precision.

## Resolution Burst Summary

| Finding | Severity | Substance | Resolution | Status |
|---------|----------|-----------|------------|--------|
| F-P17-H-001 | HIGH | SUBSTANTIVE | state-manager: STORY-INDEX v2.00 (3 rows corrected) | REMEDIATED |
| F-P17-M-001 | MEDIUM | COSMETIC | architect: ADR-016 v0.9, ADR-017 v0.5 (frontmatter date sync) | REMEDIATED |
| F-P17-M-002 | MEDIUM | COSMETIC | DEFERRED → TD-VSDD-045 (structural gap, major rebuild required) | DEFERRED |

**Window status:** 0/3 (BLOCKED → REMEDIATED). Pass 18 required (window 1/3 attempt).
