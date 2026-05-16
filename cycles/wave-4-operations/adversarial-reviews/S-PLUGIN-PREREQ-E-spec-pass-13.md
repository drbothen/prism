---
document_type: adversarial-review
story: S-PLUGIN-PREREQ-E
pass: 13
scope: spec
verdict: BLOCKED
total_findings: 3
severity_breakdown:
  critical: 0
  high: 3
  medium: 0
  low: 0
  observation: 0
in_scope_findings: 3
observations_queued: 0
produced_by: adversary
reviewed_at: 2026-05-16
fix_burst: fix-burst-12
fix_burst_closed_at: pending
streak_after_pass: "0/3"
streak_before_pass: "0/3"
novelty: HIGH
fb_introduces_defects_pattern: true
trajectory: "14→9→8→9→10→10→FB6→8→FB7→4→FB8→CLEAN★(1/3)→BLOCKED(0/3)→FB9-CLOSED→BLOCKED(0/3)→FB10-CLOSED→BLOCKED(0/3)→FB11-CLOSED→BLOCKED(0/3)"
---

# S-PLUGIN-PREREQ-E Adversarial Spec Review — Pass 13

**Verdict: BLOCKED — 3 in-scope HIGH findings, ALL introduced by FB11. Streak stays 0/3.**

**CRITICAL PATTERN: FB-INTRODUCES-NEW-DEFECTS.** Pass-13 surfaces 3 HIGH-severity defects that were created BY the FB11 burst (BC-2.16.002 catalog scope expansion + BC-2.16.012 cross-reference). This is the second instance of fix-burst-introduces-new-class pattern (first was FB6 → pass-7 within-burst sibling-sweep asymmetry). RECURRING process gap.

Novel-finding count trajectory: 14→9→8→9→10→10→8→4→0→3→1→1→**3** — re-elevation driven by FB11 quality issues, not new spec defects.

## Finding Inventory

### F-LP13-HIGH-001 — BC-2.16.012 POL-21 Phantom-Anchor Violation RE-INTRODUCED by FB11 (RECURRING class)

**Severity:** HIGH (POL-21 severity floor)
**Anchor policies:** POL-21 (phantom_section_anchor_prohibited)
**Routing:** product-owner (BC-2.16.012 body authoring)

**Sites (3 within BC-2.16.012):**
- Line 84 (twice): "per **BC-2.16.002 §Canonical Structured Event Catalog v1.18**" + "defined in BC-2.16.002 §Canonical Structured Event Catalog v1.18 (row 33)"
- Line 109 EC-016-012-005: "is emitted per BC-2.16.002 §Canonical Structured Event Catalog (v1.18 row)"

**Evidence:** BC-2.16.002 has NO `## Canonical Structured Event Catalog` heading. Verified `grep ^"## "` returns 12 headings (Description/Preconditions/Postconditions/etc.); "Canonical Structured Event Catalog" appears ONLY as a bold-labeled bullet inside §Postconditions. POL-21 verification step 3: bare §-sigil form is FORBIDDEN for bullet targets.

**Recurring-class evidence:** PREREQ-D F-LP34-MED-001 closed the same defect class for the same target heading. POL-21-correct precedent already exists at error-taxonomy.md:473 E-PIPELINE-001 row: `BC-2.16.002 §Postconditions (Canonical Structured Event Catalog bullet, v1.12)`.

**Fix:** Rewrite all 3 sites to `BC-2.16.002 §Postconditions (Canonical Structured Event Catalog bullet, v1.18)` matching error-taxonomy E-PIPELINE-001 precedent. Bump BC-2.16.012 v1.9 → v1.10.

### F-LP13-HIGH-002 — BC-2.16.002 Frontmatter modified/timestamp Stale (POL-23 + POL-27)

**Severity:** HIGH (POL-23 severity floor)
**Anchor policies:** POL-23 (bc_version_bump_sibling_grep_gate), POL-27 (bc_modified_field_iso_date_format)
**Routing:** state-manager (frontmatter sync routing)

**Sites:**
- BC-2.16.002 line 7: `timestamp: 2026-05-14T00:00:00Z` (stale)
- BC-2.16.002 line 14: `modified: 2026-05-14` (stale)
- BC-2.16.002 line 173: v1.18 changelog row dated 2026-05-16

**Fix:** Update both frontmatter fields to 2026-05-16 / 2026-05-16T00:00:00Z. No version bump (frontmatter-only edit). Per POL-23 verification step 3 + POL-27 verification step 3.

### F-LP13-HIGH-003 — BC-2.16.002 Catalog Row 33 plugin_name Field Unresolvable (Spec-Implementation Coherence Gap)

**Severity:** HIGH (blocks implementer)
**Anchor policies:** POL-22 Phase C (named-entity-existence-verification analogue)
**Routing:** architect (ADR-026 D7 + BC-2.16.002 catalog field schema authority); PO co-change

**Evidence:**
- BC-2.16.002 row 33 (FB11-added): mandates `fields: plugin_name: String, tool_name: String, error: "E-PLUGIN-020"`
- BC-2.16.012 line 84: `tracing::warn!(event_type = "write_tool_registration_after_boot", plugin_name, tool_name, error = "E-PLUGIN-020")`
- ADR-026 line 264: signature `pub fn register_write_tool(entry: WriteToolInvalidationMap) -> Result<(), SpecEngineError>` — no plugin_name parameter
- Story Task 7 + HS-003-03 + HS-003-04: `WriteToolInvalidationMap { sensor_id, tool_name, ... }` — no plugin_name field
- ADR-026 D7 narrative does not specify how plugin_name is sourced inside the emission site

The catalog row mandates a `plugin_name` field with NO defined source. Implementer cannot satisfy the catalog contract from the API surface.

**Companion gap:** Error-taxonomy E-PLUGIN-012 message template requires `{plugin}` + `{conflicting_plugin}` placeholders with same API-surface gap (pre-FB11; not novel to this pass but reinforces the systemic issue).

**Fix options (architect adjudicates):**
- Option A: Extend `WriteToolInvalidationMap` struct with `plugin_name: String` field (story+HS+ADR updates needed)
- Option B: Add `plugin_name: &str` parameter to `register_write_tool` signature (ADR-026 D7 v1.9 → v1.10)
- Option C: Remove `plugin_name` from catalog row field schema (use `sensor_id, tool_name, error` only); update BC-2.16.012 line 84 and error-taxonomy E-PLUGIN-012/020 message templates accordingly

Architect adjudicates one option in FB12; PO + state-manager propagate per choice.

## FB11 Closure Verification — PARTIAL

| Target | Result | Detail |
|---|---|---|
| BC-2.16.002 row 33 semantic | PARTIAL | row exists with source/error/recurrence; `plugin_name` field has no defined source (F-LP13-HIGH-003) |
| BC-2.16.012 §Postconditions cross-ref | PASS but POL-21 violation | macro template complete; bare §-sigil form (F-LP13-HIGH-001) |
| EC-016-012-005 event name explicit | PASS but POL-21 violation | event named; bare §-sigil form (F-LP13-HIGH-001) |
| BC-INDEX row tag bumps | PASS | BC-2.16.002 v1.18 ✓; BC-2.16.012 v1.9 ✓; BC-INDEX v4.87 ✓ |
| Reference chain completeness | LOGICAL PASS / NAVIGABILITY BLOCKED | chain intact; navigability blocked by F-LP13-HIGH-001 |
| BC-2.16.002 frontmatter modified sync | FAIL | stale 2026-05-14 (F-LP13-HIGH-002) |

## Pattern Observation — FB-Introduces-New-Defects

Pass-7 caught FB6 within-burst sibling-sweep asymmetry (closed by single-bump discipline in FB8). Pass-13 catches FB11 introducing 3 new defects (POL-21 violation + frontmatter staleness + plugin_name unresolvable). The pattern: fix-bursts that expand scope or introduce new artifacts can introduce process-gap defects.

**[process-gap] Codification candidate POL-29:** "fix-burst commit checklist" — before declaring done, fix-burst author runs (a) POL-21 sweep on any new §-citations, (b) POL-23 frontmatter modified/timestamp sync on any new file, (c) POL-22 Phase C field-source coherence check on any new structured data. Queued cycle-close.

## Trajectory Summary

| Pass | Findings | In-Scope | Novelty | Streak |
|------|----------|----------|---------|--------|
| 12 | 1 | 1 | HIGH (novel axis) | 0/3 |
| 13 | 3 | 3 | HIGH (FB11-introduced) | 0/3 |

Novel-finding count: 14→9→8→9→10→10→8→4→0→3→1→1→**3** (re-elevation; not plateau).

## Next Step

Fix-burst-12 dispatch:
- **architect** (F-LP13-HIGH-003 plugin_name resolution adjudication)
- **product-owner** (F-LP13-HIGH-001 POL-21 sweep BC-2.16.012; potential body changes per architect's Option A/B/C choice)
- **state-manager** (F-LP13-HIGH-002 BC-2.16.002 frontmatter sync + propagation per architect's Option choice)

Then adversary pass-14.

Pass-13 report: `cycles/wave-4-operations/adversarial-reviews/S-PLUGIN-PREREQ-E-spec-pass-13.md` (this file).
