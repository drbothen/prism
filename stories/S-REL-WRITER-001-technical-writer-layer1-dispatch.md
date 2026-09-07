---
document_type: story
story_id: S-REL-WRITER-001
title: "devops: technical-writer Layer-1 dispatch contract — Highlights/Breaking Changes/Upgrade Notes top-block INSIDE ## [VERSION] block in release-prep.yml"
wave: F-A
epic_id: E-REL-NOTES
priority: P0
status: ready
version: "1.1"
level: "L4"
producer: story-writer
timestamp: "2026-09-05T00:00:00Z"
tdd_mode: facade
# tdd_mode: facade — this story adds a workflow step to release-prep.yml (YAML) and
# documents the Layer-1 draft contract in RELEASING.md. No Rust production code.
subsystems: [SS-22]
# Subsystem anchor justification:
#   SS-22 (Process Lifecycle) owns release toolchain infrastructure. The
#   technical-writer dispatch in release-prep.yml is a release-engineering
#   workflow change within SS-22's scope.
crates_touched: []
target_module: devops
capabilities: []
behavioral_contracts: []
# BC status: N/A — release workflow documentation step; no subsystem BC.
verification_properties: []
depends_on: [S-REL-CLIFF-001]
blocks: [S-REL-BETA1-NOTES-001]
# Dependency anchor justifications:
#   depends_on S-REL-CLIFF-001: the technical-writer step runs AFTER git-cliff
#     prepends the ## [VERSION] block; CLIFF-001 must establish the git-cliff step
#     first so WRITER-001 can insert its step immediately after it.
#   blocks S-REL-BETA1-NOTES-001: the first-release CHANGELOG production requires
#     both git-cliff (CLIFF-001) and the technical-writer Layer-1 draft to be in
#     place; WRITER-001 defines the Layer-1 contract that BETA1-NOTES-001 exercises.
points: 3
estimated_days: 0.5
risk: LOW
acceptance_criteria_count: 5
red_gate_tests: 0
# red_gate_tests: 0 — facade mode. No Rust code; no Red Gate.
estimated_passes: "1 LOCAL adversary pass"
holdout_scenarios: []
# HOLDOUT-N/A: tdd_mode=facade; crates_touched=[]; no built prism binary; no MCP-visible
# surface. Deliverable is a release-prep.yml YAML step (echo placeholder notices) and
# RELEASING.md prose. All verification is structural grep/text checks the implementer
# explicitly performs (AC-001..AC-005: grep on YAML file + RELEASING.md text). The
# dispatch step itself contains only echo commands — it is a placeholder notice, not
# executable runtime code. No info-asymmetry surface exists. Gate definitionally
# inapplicable per CLAUDE.md story-level holdout gate definition (requires built prism
# binary + MCP stdio wire assertions, scoped to the story's touched surface).
assumption_validations: []
risk_mitigations:
  - "Layer-1 content (### Highlights etc.) is placed INSIDE the ## [VERSION] block,
    AFTER git-cliff prepends it, and BEFORE the first git-cliff commit-derived ###
    section. This placement ensures the existing release.yml awk extraction captures
    both layers without modification to release.yml (ADR-063 D4 v1.1)."
  - "The technical-writer dispatch in release-prep.yml is a NEW step ordered AFTER
    the git-cliff step. The git-cliff step creates the ## [VERSION] block; the
    technical-writer step then edits CHANGELOG.md to insert Layer-1 sections inside
    that block."
  - "This story defines the dispatch contract and RELEASING.md documentation.
    The actual technical-writer agent invocation during beta.1 release is part of
    S-REL-BETA1-NOTES-001."
inputs:
  - ".github/workflows/release-prep.yml"
  - "RELEASING.md"
  - ".factory/specs/architecture/decisions/ADR-063-changelog-release-notes-architecture.md"
input-hash: "[pending-recompute]"
traces_to: []
cycle: "v1.0.0-beta.1-release-identity"
phase: "3"
---

# S-REL-WRITER-001 — Technical-Writer Layer-1 Dispatch Contract

**Story ID:** S-REL-WRITER-001
**Status:** ready
**Version:** v1.1
**Wave:** F-A
**Priority:** P0
**Points:** 3
**Beta.1-blocking:** YES — the Layer-1 draft contract must be defined before
S-REL-BETA1-NOTES-001 can produce the beta.1 CHANGELOG section.

---

## Origin

ADR-063 D4 defines a two-layer CHANGELOG model. Layer 2 (git-cliff body) is handled
by S-REL-CLIFF-001. Layer 1 is a human-curated top-block drafted by
`vsdd-factory:technical-writer` agent at release-prep time. ADR-063 D4 v1.1 specifies:
- Layer-1 sections (`### Highlights`, `### Breaking Changes (narrative)`,
  `### Upgrade Notes`) are placed INSIDE the `## [VERSION]` block
- Placement is AFTER git-cliff prepends the `## [VERSION]` block
- Placement is BEFORE the first git-cliff commit-derived `###` section
- The human reviews and curates the draft in the release-prep PR

This story adds the technical-writer dispatch step to `release-prep.yml` (ordered
after the git-cliff step) and documents the Layer-1 contract in `RELEASING.md §5`.

---

## Narrative

As a release engineer, I want a technical-writer agent to draft `### Highlights`,
`### Breaking Changes (narrative)`, and `### Upgrade Notes` sections inside the
`## [VERSION]` CHANGELOG block at release-prep time, so that every release includes
a curated human-readable narrative that operators see immediately, before the
auto-generated commit listing.

---

## Authority

- ADR-063 D4 — Two-Layer CHANGELOG Model
- ADR-063 D5 — release-prep.yml step order (git-cliff → technical-writer → human curation)

(No BC: release workflow documentation; no subsystem behavioral contract.)

---

## Behavioral Contracts

This story has no subsystem behavioral contracts. Authority is ADR-063 D4/D5.

| Architecture Source | Clause |
|---------------------|--------|
| ADR-063 D4 | Layer-1 sections placed INSIDE the `## [VERSION]` block, AFTER git-cliff, BEFORE first commit-derived `###` |
| ADR-063 D4 | Technical-writer drafts this block from the tag range; documents CURRENT behavior only |
| ADR-063 D4 | Human reviews and curates the draft in the release-prep PR; it is an aid, not the final |
| ADR-063 D4 | Empty sections (### Breaking Changes when no breaking changes) are omitted |
| ADR-063 D5 | Step order: (1) git-cliff prepends ## [VERSION]; (2) technical-writer inserts Layer-1; (3) human curates |

---

## Token Budget Estimate

| Artifact | Estimated Tokens |
|----------|-----------------|
| This story spec | ~2,500 |
| `.github/workflows/release-prep.yml` (full) | ~5,000 |
| `RELEASING.md` (§5 section) | ~1,500 |
| ADR-063 D4/D5 sections | ~2,000 |
| Total | ~11,000 |

Well within the 30% context window budget.

---

## Red Gate Test List (SAC-1)

**tdd_mode: facade — Red Gate is N/A.** No Rust production code; no Red Gate tests.
Verification is via structural grep/text checks the implementer explicitly performs:
- AC-001: `grep -n 'technical-writer\|Layer-1' .github/workflows/release-prep.yml` → ≥1 match positioned AFTER the git-cliff step
- AC-002: `grep -n 'Layer 1\|Layer-1\|Highlights' RELEASING.md` → matches in §5 section
- AC-003: Read RELEASING.md §5 text → confirms placement language is INSIDE (not above) `## [VERSION]` block
- AC-004: `grep -n 'Layer-1\|Highlights.*reviewed' .github/workflows/release-prep.yml` → ≥1 PR checklist item
- AC-005: `git diff HEAD -- .github/workflows/release.yml` → no changes

---

## Tasks

1. **Read `.github/workflows/release-prep.yml` in full** (post-S-REL-CLIFF-001 state
   — the git-cliff step should already be in place).

2. **Add technical-writer dispatch step to release-prep.yml**, AFTER the git-cliff step:
   ```yaml
   - name: Draft Layer-1 release notes (technical-writer)
     run: |
       echo "::notice::Dispatch vsdd-factory:technical-writer to draft Layer-1 release notes"
       echo "Technical-writer should:"
       echo "  1. Read CHANGELOG.md to find the newly-prepended ## [${VERSION_TAG}] block"
       echo "  2. Insert ### Highlights, ### Breaking Changes (narrative), ### Upgrade Notes"
       echo "     inside the ## [${VERSION_TAG}] block, BEFORE the first git-cliff ### section"
       echo "  3. Content: current behavior only; 5-8 bullets for Highlights; omit empty sections"
       echo "  4. Human curates the draft in this PR before merge"
       echo "See RELEASING.md §5 for the Layer-1 contract."
   ```
   Note: The step is a placeholder dispatch notice; the actual technical-writer agent
   is invoked manually by the release engineer using the vsdd-factory dispatch mechanism.
   This step documents the expected action in the CI log.

3. **Update `RELEASING.md §5` (Release Notes Convention):**
   Replace or supplement the existing §5 prose with the ADR-063 D4 v1.1 two-layer
   model. The updated §5 must specify:
   - Layer 1: `### Highlights` / `### Breaking Changes (narrative)` / `### Upgrade Notes`
     sections placed INSIDE the `## [VERSION]` block, BEFORE git-cliff's first `###`
   - Layer 2: git-cliff categorized body (Added/Fixed/Performance/Changed/Security)
   - Placement invariant: Layer-1 is placed AFTER git-cliff prepends the block,
     BEFORE first commit-derived section
   - Human-curation gate: the technical-writer draft is reviewed and curated in
     the release-prep PR; it is an aid, not the final
   - Empty sections (Breaking Changes when no breaking changes) are omitted

4. **Update the release-prep.yml PR checklist** to include:
   - `[ ] Layer-1 Highlights section reviewed and curated`
   - `[ ] Layer-1 Breaking Changes (narrative) section reviewed (or confirmed empty)`

5. **Verify AC-001..AC-005.**

---

## Acceptance Criteria

### AC-001: Technical-writer dispatch step exists in release-prep.yml after git-cliff step
`grep -n 'technical-writer\|Layer-1' .github/workflows/release-prep.yml` returns at
least one match in a step ordered AFTER the git-cliff step.
(traces to ADR-063 D5 — "technical-writer step ordered AFTER git-cliff")

### AC-002: RELEASING.md §5 documents the two-layer model
`grep -n 'Layer 1\|Layer-1\|Layer 2\|Layer-2\|Highlights' RELEASING.md` returns
matches in the §5 section. The layer placement rule (inside ## [VERSION], before
first git-cliff ###) is stated. (traces to ADR-063 D4 — two-layer CHANGELOG model)

### AC-003: RELEASING.md §5 states Layer-1 placement is INSIDE the ## [VERSION] block
The §5 text explicitly states that `### Highlights` and companion sections are placed
INSIDE (not above) the `## [VERSION]` block. (traces to ADR-063 D4 v1.1 correction —
placement moved inside so release.yml awk extraction captures Layer-1)

### AC-004: release-prep.yml PR checklist includes Layer-1 review item
`grep -n 'Layer-1\|Highlights.*reviewed' .github/workflows/release-prep.yml` returns
at least one checklist item. (traces to ADR-063 D4 — "human reviews and curates the
draft in the release-prep PR")

### AC-005: release.yml is NOT modified
`git diff HEAD -- .github/workflows/release.yml` returns no changes. The `--notes-file`
awk extraction mechanism remains unchanged. (traces to ADR-063 D5 — "release.yml
--notes-file mechanism unchanged")

---

## Previous Story Intelligence

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| S-REL-CLIFF-001 | git-cliff 2.14.1 installed in release-prep.yml | git-cliff step uses `--unreleased --tag "${VERSION_TAG}" --prepend CHANGELOG.md` (NOT `--latest`; ADR-063 D5/D6) | `--output` must NOT be combined with `--prepend`; `--latest` MUST NOT be used (unreliable for pre-tag invocations) |

This story adds the Layer-1 step AFTER the S-REL-CLIFF-001 git-cliff step. Read the
release-prep.yml post-CLIFF-001 state before inserting the new step.

---

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| Layer-1 step ordered AFTER git-cliff step | ADR-063 D5 | Verify step order in release-prep.yml |
| Layer-1 sections placed INSIDE ## [VERSION] block | ADR-063 D4 v1.1 | AC-003 RELEASING.md §5 text |
| release.yml NOT modified | ADR-063 D5 | AC-005 git diff check |
| Empty Layer-1 sections omitted | ADR-063 D4 | RELEASING.md §5 prose |
| Technical-writer documents CURRENT behavior only | ADR-063 D4 | RELEASING.md §5 authoring guidance |

---

## Library & Framework Requirements

| Dependency | Version | Notes |
|------------|---------|-------|
| `vsdd-factory:technical-writer` | N/A | Agent invoked manually at release time; not a binary dep |
| `.github/workflows/release-prep.yml` | N/A | Existing workflow; modified in-place |

---

## File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `.github/workflows/release-prep.yml` | Modify | Add Layer-1 dispatch step after git-cliff step; add checklist item |
| `RELEASING.md` | Modify | §5 updated to document two-layer model per ADR-063 D4 v1.1 |
| `.github/workflows/release.yml` | DO NOT modify | --notes-file mechanism explicitly unchanged |

---

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| Layer-1 dispatch step | `.github/workflows/release-prep.yml` | Effectful (CI workflow step; modifies CHANGELOG.md) |
| `RELEASING.md §5` | `RELEASING.md` | Pure (documentation) |

---

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| `.github/workflows/release-prep.yml` | effectful-shell | CI workflow; writes CHANGELOG.md and opens PRs |
| `RELEASING.md` | pure-core | Documentation file; no runtime behavior |

---

## Holdout Applicability

**Determination: N/A**

The story-level holdout gate (CLAUDE.md, human-approved 2026-07-13) requires a built
prism binary and an MCP-visible surface with wire-level assertions. This story has
`tdd_mode: facade` and `crates_touched: []` — it produces no prism binary. Its entire
deliverable is a new step in `.github/workflows/release-prep.yml` (composed of `echo`
placeholder notices that document the expected manual dispatch action) and updated prose
in `RELEASING.md §5`.

All verification surfaces are structural text checks: AC-001 through AC-005 are grep
patterns on the YAML workflow file and RELEASING.md. The technical-writer dispatch step
contains only `echo` commands; it is a workflow documentation artifact, not executable
product code. There is no runtime execution path, no MCP surface, and no binary
buildable from this story's changes.

No machine-verifiable hidden surface with genuine info-asymmetry exists. Do NOT
fabricate a holdout for a gate that definitionally does not apply.

`holdout_scenarios: []` — no scenarios authored; HOLDOUT-INDEX unchanged.

---

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | git-cliff step fails before technical-writer step | The dispatch step is skipped (it is a `run:` step in the same job; job fails at git-cliff step first) |
| EC-002 | Release has no breaking changes | `### Breaking Changes (narrative)` section omitted per ADR-063 D4 |
| EC-003 | Technical-writer drafts aspirational features | Human curation gate removes them; RELEASING.md §5 must explicitly state "documents current behavior only" |
| EC-004 | release.yml awk extraction window | Layer-1 content is INSIDE `## [VERSION]` block → captured by `## [VERSION]`..next `## [` window unchanged |

---

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.1 | 2026-09-06 | story-writer | Sweep #13: Previous-Story-Intelligence `--latest --prepend` → `--unreleased --tag "${VERSION_TAG}" --prepend` per ADR-063 D5/D6; Red Gate N/A (facade) note made explicit with enumerated verification steps; status draft→ready |
| 1.0 | 2026-09-05 | story-writer | Initial — ADR-063 D4/D5 materialization |
