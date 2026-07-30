---
document_type: story
story_id: S-MAINT-CAPREF-SWEEP-001
title: "Restructure capabilities.md and invariants.md to Convert CAP-NNN/DI-NNN Rows into ## Sections, Making All 102 Existing Citations Valid"
wave: maintenance
epic_id: maintenance
priority: P2
status: draft
version: "1.0"
updated: "2026-07-30"
level: "L2"
producer: story-writer
timestamp: "2026-07-30T00:00:00Z"
tdd_mode: strict
# tdd_mode: strict — mandatory per BC-8.30.001 invariant 2.
# No Rust code is touched (crates_touched: []). The Red Gate / todo!() stub discipline
# has no behavioral effect on a records-only restructure. Red Gate items below are
# mechanical verification commands (grep / heading-count checks) that establish a
# failing state before the restructure is done and pass after. tdd_mode is present
# for schema compliance; it does not trigger Rust-specific TDD machinery.
subsystems: []
crates_touched: []
target_module: ".factory/specs/domain-spec"
capabilities: []
behavioral_contracts: []
# BC status: pending PO authorship.
# POL-21 phantom_section_anchor_prohibited is a governance policy, not a behavioral contract.
# A PO-authored BC covering domain-spec structural integrity would need to be anchored
# before this story can advance to status: ready (S-7.01 gate).
verification_properties: []
holdout_scenarios: []
# POL-35 holdout_gate_infra_only_exemption applies: behavioral_contracts: [] (pure
# domain-spec restructure, no behavioral surface). holdout_scenarios: [] is compliant.
depends_on:
  - S-MAINT-L11-GATE-001
# depends_on justification:
#   S-MAINT-L11-GATE-001 must ship first so the L11 gate is in place before this
#   story's changes land. Without L11, new version pins in the restructured files
#   would not be caught by the gate, defeating the governance chain. Additionally,
#   the CAPREF restructure adds many new ## section headings; having L11 deployed
#   ensures any accidental version pins in those new sections are immediately blocked.
blocks:
  - S-MAINT-ANTIPIN-SWEEP-001
# blocks justification:
#   S-MAINT-ANTIPIN-SWEEP-001 sweeps .factory/stories/ for version pins; some of
#   those stories cite capabilities.md §CAP-NNN or invariants.md §DI-NNN. If the
#   restructure is not done first, the sweep stories cannot verify that their
#   converted citations resolve correctly. The dependency also prevents concurrent
#   editing of the same domain-spec files.
points: 5
estimated_days: 1.5
risk: LOW
# Risk justification:
#   Structural restructure of two files with well-defined transformations (row → section).
#   No behavioral content changes. 69 rows → 69 ## sections. Primary risk is accidental
#   content drift during the per-row conversion. AC-003 (cell-content preservation verified
#   by diff) mitigates this. The 72 citing files are not touched, so citation breakage is
#   not possible from this story's changes.
assumption_validations: []
risk_mitigations: []
tags:
  - domain-spec
  - pol-21
  - section-anchors
  - restructure
---

# S-MAINT-CAPREF-SWEEP-001: Restructure capabilities.md and invariants.md to Convert CAP-NNN/DI-NNN Rows into ## Sections, Making All 102 Existing Citations Valid

## Narrative

As a spec maintainer, I want `domain-spec/capabilities.md` and `domain-spec/invariants.md`
to use `## CAP-NNN` and `## DI-NNN` section headings for each capability and invariant
entry (instead of plain table rows), so that the 102 existing citations of the form
`capabilities.md §CAP-NNN` and `invariants.md §DI-NNN` across 72 spec files become valid
section-anchor references per POL-21 (`phantom_section_anchor_prohibited`, HIGH), without
requiring any edit to the 72 citing files.

## Background: Measured Scope

POL-21 requires that a `§X` citation resolve to a real `## X` heading in the referenced
file. Currently:

- `domain-spec/capabilities.md` has 39 `CAP-NNN` table rows and only **1** `##` heading
  (`## Changelog`). Every `capabilities.md §CAP-NNN` citation is a POL-21 violation.
- `domain-spec/invariants.md` has 30 `DI-NNN` table rows and **2** `##` headings.
  Every `invariants.md §DI-NNN` citation is a POL-21 violation.
- **102 citations** across **72 files** use the `§CAP-NNN` or `§DI-NNN` form covering
  **17 unique CAP/DI targets** across the cited corpus.

Implementation (a) — chosen by the human — converts the two enumeration documents so
existing citations resolve unchanged. This avoids editing 72 citing files.

## Tradeoff Analysis

**Cost:** Two compact scannable tables (one row per capability/invariant) become longer
heading-per-entry documents. Navigation within the files shifts from visual table scanning
to anchor-linked jumps.

**Benefit:** Each CAP/DI entry becomes individually citable (via `§CAP-NNN` anchor), individually
versionable in version-control history, and structurally compliant with POL-21. This is
consistent with the sharded domain-spec convention already in use for BC and ADR files.

**Scope recommendation — convert ALL 69 rows, not only the 17 cited targets:** Converting
only the 17 cited targets leaves 52 non-cited rows as table rows. A mixed file (some entries
as `##` sections, others as table rows) is harder to navigate than a uniform structure.
Additionally, any future citation of a non-converted row would immediately create a POL-21
violation. Converting all 69 rows once produces a permanently compliant file structure.
This story implements the full-conversion approach.

**Mitigation for navigation cost:** Both files gain a **summary index table** at the top
(before the individual sections) that lists all IDs with a one-line description and an anchor
link. This preserves the "scan all entries at a glance" use case.

## Behavioral Contracts

| BC | Title | Version | Scope in This Story |
|----|-------|---------|---------------------|
| (none) | — | — | No behavioral contracts yet. See BC status comment in frontmatter. Story MUST remain `status: draft` until a domain-spec-integrity BC is authored and anchored. |

## Acceptance Criteria

### AC-001 — All CAP-NNN entries in capabilities.md are converted to ## sections
After implementation, `domain-spec/capabilities.md` contains a `## CAP-NNN` heading for
each of the 39 capability entries (CAP-001 through CAP-039, or the actual IDs present at
implementation time — the exact IDs are the authoritative set; 39 is the measured count at
authoring time). A heading-count check `grep -c "^## CAP-" capabilities.md` MUST return 39.
Each section MUST preserve the full cell content from the original table row (description,
BC traceability citations, invariant references, etc.) under the heading.
(traces to POL-21 `phantom_section_anchor_prohibited` §verification_steps step 1; verified by RG-001)

### AC-002 — All DI-NNN entries in invariants.md are converted to ## sections
After implementation, `domain-spec/invariants.md` contains a `## DI-NNN` heading for each
of the 30 invariant entries (DI-001 through DI-030, or the actual IDs present). A
heading-count check `grep -c "^## DI-" invariants.md` MUST return 30. Each section MUST
preserve the full cell content from the original table row.
(traces to POL-21 `phantom_section_anchor_prohibited` §verification_steps step 1; verified by RG-002)

### AC-003 — Original table-row cell content is fully preserved; no behavioral meaning changed
The conversion is structural only: table-row content becomes section body prose or a sub-table.
No capability description, no invariant statement, no BC traceability citation, and no
enforcement-scope reference changes in meaning. Verified by: (a) adversarial review reads the
before/after diff and confirms no semantic changes; (b) the diff for each CAP/DI entry shows
only the addition of the `## CAP-NNN` heading line and the removal of the markdown table row
delimiters (`| ... |`), with all cell content retained.
(traces to CLAUDE.md §Source-of-Truth Precedence — spec wins; structural-only edits must not
alter behavioral meaning; verified by adversarial diff review)

### AC-004 — A summary index table is added at the top of each file
`capabilities.md` gains a summary index table at the top listing all 39 CAP-NNN entries
with their one-line description and an anchor link (`[CAP-001](#cap-001)` or equivalent).
`invariants.md` gains an equivalent table for DI-NNN entries. This preserves the
"scan all entries at a glance" use case that the original flat-table format provided.
(traces to the tradeoff mitigation stated in §Background; verified by reading the output files)

### AC-005 — Known citation count confirmed: a spot-check of 5 previously-invalid citations now resolves
After restructure, a spot-check of at least 5 citations from the 72 citing files (chosen to
cover at least 3 distinct CAP IDs and 2 distinct DI IDs from the 17 measured unique targets)
resolves to a real `## CAP-NNN` or `## DI-NNN` heading. The spot-check is documented in
the PR description and verified by the adversarial review cascade.
(traces to POL-21 §verification_steps step 2 "verify anchor resolves to a real heading";
verified by RG-003)

## Red Gate Tests

These are mechanical verification commands that establish the failing state BEFORE the
restructure is done. They pass AFTER. No Rust test runner; verification is via shell
commands run at the start of implementation (to confirm the failing state) and again
at completion (to confirm the passing state).

- [ ] **RG-001** (`verify_cap_headings_count`): Run
  `grep -c "^## CAP-" .factory/specs/domain-spec/capabilities.md` before restructure.
  MUST return 0 (failing state — no section headings exist yet). MUST return 39 after.

- [ ] **RG-002** (`verify_di_headings_count`): Run
  `grep -c "^## DI-" .factory/specs/domain-spec/invariants.md` before restructure.
  MUST return 0. MUST return 30 after.

- [ ] **RG-003** (`verify_spot_citation_resolves`): Before restructure, confirm that
  `capabilities.md §CAP-001` does NOT correspond to a `## CAP-001` heading (failing state).
  After restructure, confirm `## CAP-001` heading exists and content matches the original row.

**Red Gate density check (BC-5.38.001):** 3 Red Gate verification commands (RG-001 through
RG-003) anchor to 5 acceptance criteria. For records-only stories with no Rust test runner,
density verification (`RED_TESTS * 2 >= (TOTAL_NEW_TESTS − EXEMPT_TESTS)`) is adapted: the
"tests" are mechanical verification commands that fail before and pass after. Computed at
dispatch per `per-story-delivery.md §Red Gate Density Check` and BC-5.38.002/BC-5.38.003.

## Architecture Mapping

| Component | Location | Pure/Effectful |
|-----------|----------|----------------|
| `domain-spec/capabilities.md` | `.factory/specs/domain-spec/capabilities.md` | Pure (records-only restructure) |
| `domain-spec/invariants.md` | `.factory/specs/domain-spec/invariants.md` | Pure (records-only restructure) |
| 72 citing files | Various `.factory/specs/` locations | NOT touched (citations already resolve after restructure) |

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | A CAP-NNN entry has multi-paragraph description spanning multiple table cells | Convert all cells to sub-sections under the `## CAP-NNN` heading; preserve all prose. |
| EC-002 | A CAP-NNN or DI-NNN entry has a BC traceability citation like `BC-2.01.001` in its content | Preserve exactly; this is NOT a version pin (no `v`-prefix). L11 will not flag it. |
| EC-003 | The existing `## Changelog` section in capabilities.md conflicts with the heading hierarchy after conversion | Place the Changelog section AFTER all CAP-NNN sections. Ensure the summary index table links do not break the anchor structure. |
| EC-004 | The exact count of CAP-NNN or DI-NNN IDs differs from the measured 39/30 at implementation time | Use the actual IDs present in the file; adjust RG-001/RG-002 expected counts accordingly. State the actual count in the PR description. |
| EC-005 | A citing file uses `§CAP-NNN` with a different separator or casing | This story does NOT modify citing files. If a citation form does not resolve to a `## CAP-NNN` heading after restructure, it remains a POL-21 violation to be addressed in a follow-up. The scope of this story is the two source files. |
| EC-006 | L11 fires on the restructured files because the summary index table contains a version pin | The summary index table MUST NOT contain version pins — entries are `CAP-NNN` IDs with one-line descriptions only. If L11 fires on the restructured files during merge, this is a bug in the restructure, not a false positive. |

## Token Budget Estimate

| Artifact | Estimated Tokens | Notes |
|----------|-----------------|-------|
| This story spec | ~7,000 | |
| `domain-spec/capabilities.md` (current form) | ~8,000 | Load once to understand current structure |
| `domain-spec/invariants.md` (current form) | ~5,000 | Load once to understand current structure |
| 72 citing files (spot-check 5 for AC-005) | ~2,000 | Load only 5; do not load all 72 |
| **Total per implementation session** | ~22,000 | Both files can be processed in a single context window |

## Tasks

### Phase A — Establish failing state (Red Gate)

- [ ] **T-A01**: Run RG-001: `grep -c "^## CAP-" .factory/specs/domain-spec/capabilities.md`.
  Confirm returns 0 (failing state). Record actual CAP-NNN ID count from current file.
- [ ] **T-A02**: Run RG-002: `grep -c "^## DI-" .factory/specs/domain-spec/invariants.md`.
  Confirm returns 0. Record actual DI-NNN ID count.
- [ ] **T-A03**: Run RG-003: confirm no `## CAP-001` heading exists.

### Phase B — Restructure capabilities.md

- [ ] **T-B01**: Read `domain-spec/capabilities.md` and map all CAP-NNN IDs and their content.
- [ ] **T-B02**: Write new `capabilities.md` structure:
  1. Header + introductory prose (unchanged)
  2. Summary index table (AC-004): columns `ID | Description | Anchor`
  3. For each CAP-NNN in original order: `## CAP-NNN` heading + full cell content as body prose
  4. `## Changelog` section (preserved from original, at end)
- [ ] **T-B03**: Verify `grep -c "^## CAP-" capabilities.md` returns the expected count (AC-001).

### Phase C — Restructure invariants.md

- [ ] **T-C01**: Read `domain-spec/invariants.md` and map all DI-NNN IDs and their content.
- [ ] **T-C02**: Write new `invariants.md` structure (same pattern: summary index table + per-DI sections + Changelog).
- [ ] **T-C03**: Verify `grep -c "^## DI-" invariants.md` returns the expected count (AC-002).

### Phase D — Verification

- [ ] **T-D01**: Run `scripts/records-lint.sh --full-scan` on both restructured files.
  Confirm zero L11 violations (no version pins introduced in new sections or summary table).
  Confirm L1/L7 pass for both files.
- [ ] **T-D02**: Spot-check 5 citations from the 72 citing files (RG-003 and AC-005).
  Confirm each resolves to a real `## CAP-NNN` or `## DI-NNN` heading in the restructured file.
- [ ] **T-D03**: Review diff for AC-003 (no semantic changes). Note: the diff will be large
  (structural transformation). Verify that all original cell text appears under the corresponding
  `## CAP-NNN` / `## DI-NNN` heading in the new version.

### Merge gate

- [ ] **MERGE-GATE-CAP-COUNT**: `grep -c "^## CAP-" capabilities.md` returns expected count.
- [ ] **MERGE-GATE-DI-COUNT**: `grep -c "^## DI-" invariants.md` returns expected count.
- [ ] **MERGE-GATE-L11-CLEAN**: `scripts/records-lint.sh --full-scan` exits 0 for both restructured files.
- [ ] **MERGE-GATE-SEMANTIC-CLEAN**: Adversarial review confirms no semantic changes to any CAP or DI entry content.

## Previous Story Intelligence

N/A — first story in the CAPREF restructure chain.

Prior art:
- The pattern of "convert enumeration document from flat table to per-entry ## sections" is
  consistent with how BC and ADR files are structured (each BC is a separate file with its own
  heading hierarchy). This story applies the same principle within a single file.

## Architecture Compliance Rules

1. **POL-21 `phantom_section_anchor_prohibited`:** A `§X` citation MUST resolve to a real `## X`
   heading in the referenced file. After this story, all `§CAP-NNN` and `§DI-NNN` citations
   will resolve. The gate for new violations in the 72 citing files is POL-21 itself (enforced
   by adversarial review).

2. **POL-1 append-only numbering:** The CAP-NNN and DI-NNN IDs are immutable. Restructuring
   table rows into sections uses the same IDs — no renumbering. If an ID is discovered to be
   retired, it becomes a retired section (with a note) rather than being removed.

3. **Spec wins over code (CLAUDE.md §Source-of-Truth Precedence):** This story touches spec
   files, not code. Any edit that incidentally modifies a CAP or DI behavioral meaning MUST
   STOP and be routed to the product-owner or domain-BA. The implementer's mandate is
   structural-only conversion.

4. **POL-39 compliance from day one:** The restructured files MUST NOT introduce version pins
   in the new section headings, summary index tables, or section body prose. The L11 gate
   (from S-MAINT-L11-GATE-001) enforces this on merge.

5. **POL-29 TD-VSDD-097 three-dimension sweep:**
   (a) Sibling pair: `capabilities.md` and `invariants.md` are restructured in the same commit.
       Neither is done without the other.
   (b) Downstream copy target: If any downstream artifact contains verbatim prose from these
       files' introductory sections, sweep and update in the same burst.
   (c) Mandate anchor: This story's `MUST` statements trace to AC-001/AC-002/AC-003 above.

## Library and Framework Requirements

| Library/Tool | Version/Source | Purpose |
|-------------|---------------|---------|
| `bash` / `grep` | System | RG verification commands |
| `scripts/records-lint.sh` | Project-local | L11 and L1/L7 verification after restructure |

No library changes.

## File Structure Requirements

### Files to MODIFY

| File | Change |
|------|--------|
| `.factory/specs/domain-spec/capabilities.md` | Convert all CAP-NNN table rows to `## CAP-NNN` sections; add summary index table |
| `.factory/specs/domain-spec/invariants.md` | Convert all DI-NNN table rows to `## DI-NNN` sections; add summary index table |

### Files NOT to modify

| File | Reason |
|------|--------|
| The 72 citing files under `.factory/specs/` | Implementation (a): restructure sources so citations resolve without editing citators |
| Any `crates/**` file | No code changes |

## Forbidden Dependencies

No new dependencies. No changes to Cargo.toml files.

## Dependency Graph Edges

```
S-MAINT-CAPREF-SWEEP-001 (this story)
  depends_on:
    ← S-MAINT-L11-GATE-001  (L11 gate must be deployed before this story ships)
  blocks:
    → S-MAINT-ANTIPIN-SWEEP-001  (stories sweep depends on correct source file structure)
```

## Version History

| Version | Date | Change Summary |
|---------|------|----------------|
| 1.0 | 2026-07-30 | Initial story creation. Restructures capabilities.md and invariants.md to convert 39 CAP-NNN and 30 DI-NNN table rows into ## section headings, making all 102 existing citations across 72 files compliant with POL-21 phantom_section_anchor_prohibited, without editing any citing file. Full-conversion approach chosen (all 69 rows, not only the 17 cited targets) for structural consistency. Summary index tables added to preserve scan-all-entries use case. |
