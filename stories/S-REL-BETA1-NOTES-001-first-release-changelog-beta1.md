---
document_type: story
story_id: S-REL-BETA1-NOTES-001
title: "devops: first-release CHANGELOG production for v1.0.0-beta.1 — dry-run pass, noise control, Layer-1 top-block curation"
wave: F-A
epic_id: E-REL-NOTES
priority: P0
status: ready
version: "1.1"
level: "L4"
producer: story-writer
timestamp: "2026-09-05T00:00:00Z"
tdd_mode: facade
# tdd_mode: facade — this story is an operations/execution story: run git-cliff
# dry-run, tune cliff.toml skip rules, produce CHANGELOG.md content. No Rust code.
subsystems: [SS-22]
# Subsystem anchor justification:
#   SS-22 (Process Lifecycle) owns release toolchain operations.
crates_touched: []
target_module: devops
capabilities: []
behavioral_contracts: []
# BC status: N/A — release notes production; no subsystem BC.
verification_properties: []
depends_on: [S-REL-CLIFF-001, S-REL-WRITER-001]
blocks: []
# Dependency anchor justifications:
#   depends_on S-REL-CLIFF-001: cliff.toml must exist and be tuned before the
#     dry-run produces reliable categorization output.
#   depends_on S-REL-WRITER-001: the Layer-1 top-block contract must be defined
#     before the technical-writer can draft the Highlights section.
points: 3
estimated_days: 1
risk: LOW
acceptance_criteria_count: 5
red_gate_tests: 0
# red_gate_tests: 0 — facade mode. No Rust code.
estimated_passes: "1 human-in-the-loop cycle"
holdout_scenarios: []
# HOLDOUT-N/A: tdd_mode=facade; crates_touched=[]; no built prism binary; no MCP-visible
# surface. This is an operations/execution story: it runs git-cliff to produce CHANGELOG.md
# content, then dispatches vsdd-factory:technical-writer for the Layer-1 draft, then
# requires a MANDATORY human curation gate before merge (story explicitly states
# estimated_passes: "1 human-in-the-loop cycle"). All verification is structural: grep on
# CHANGELOG.md (AC-001..AC-004) and bullet-count check (AC-005). The Layer-1 Highlights
# content requires human editorial judgment — no machine holdout can substitute for that.
# git-cliff is an external CLI tool, not the prism MCP binary. No info-asymmetry surface
# exists. Gate definitionally inapplicable per CLAUDE.md story-level holdout gate
# definition (requires built prism binary + MCP stdio wire assertions, scoped to the
# story's touched surface). Do NOT fabricate a machine holdout for human-in-the-loop
# CHANGELOG curation.
assumption_validations: []
risk_mitigations:
  - "For beta.1 the full history range is used (no --latest; ADR-063 D6). If a prior
    tag exists (v1.0.0-rc.1), the range is v1.0.0-rc.1..v1.0.0-beta.1. If no prior
    tag exists, cliff generates from the initial commit."
  - "Noise control: ADR-063 D6 permits additional cliff.toml skip rules if the
    dry-run output is still noisy. Additional skip rules are committed to cliff.toml
    before the beta.1 tag is cut."
  - "The human curation gate is MANDATORY for beta.1 (ADR-063 D6 §3). The
    technical-writer draft is intentionally short (5-8 bullets)."
  - "No retroactive entry authoring: commits already in history are not re-worded
    retroactively. Quality gaps are absorbed by the curation gate (ADR-063 D6 §4)."
inputs:
  - "cliff.toml"
  - "CHANGELOG.md"
  - ".factory/specs/architecture/decisions/ADR-063-changelog-release-notes-architecture.md"
input-hash: "[pending-recompute]"
traces_to: []
cycle: "v1.0.0-beta.1-release-identity"
phase: "3"
---

# S-REL-BETA1-NOTES-001 — First-Release CHANGELOG for v1.0.0-beta.1

**Story ID:** S-REL-BETA1-NOTES-001
**Status:** ready
**Version:** v1.1
**Wave:** F-A
**Priority:** P0
**Points:** 3
**Beta.1-blocking:** YES — CHANGELOG.md must contain the beta.1 section before the
release PR is opened.

---

## Origin

v1.0.0-beta.1 is the first meaningful public release of Prism. Its CHANGELOG section
covers the full development history (or the `v1.0.0-rc.1..v1.0.0-beta.1` range if
that prior tag exists). ADR-063 D6 specifies first-release handling:
- No `--latest` flag; use full history or explicit range
- Mandatory dry-run pass before cutting the tag
- Noise control via cliff.toml skip rules
- Mandatory human curation gate (Layer-1 Highlights, 5-8 bullets)

This story executes the dry-run, tunes skip rules, dispatches technical-writer for the
Layer-1 draft, and produces the final CHANGELOG.md beta.1 section.

---

## Narrative

As a release engineer, I want CHANGELOG.md to contain a complete, well-categorized
`## [1.0.0-beta.1]` section with a curated Layer-1 Highlights block, so that operators
who download the beta release can immediately understand what the platform delivers and
whether any upgrade actions are required.

---

## Authority

- ADR-063 D6 — First-Release Handling for v1.0.0-beta.1
- ADR-063 D4 — Two-Layer CHANGELOG Model (Layer-1 curation gate mandatory for beta.1)

(No BC: release notes operations; no subsystem behavioral contract.)

---

## Behavioral Contracts

This story has no subsystem behavioral contracts. Authority is ADR-063 D6.

| Architecture Source | Clause |
|---------------------|--------|
| ADR-063 D6 | Tag range: `git cliff --tag v1.0.0-beta.1` (no `--latest`; full history unless prior tag exists) |
| ADR-063 D6 | Dry-run mandatory BEFORE the tag is cut |
| ADR-063 D6 | Human curation gate MANDATORY for beta.1; technical-writer draft is 5-8 bullets max |
| ADR-063 D6 | No retroactive re-wording of existing commits |
| ADR-063 D4 | Layer-1 sections (### Highlights / ### Breaking Changes / ### Upgrade Notes) inside ## [VERSION] block |

---

## Token Budget Estimate

| Artifact | Estimated Tokens |
|----------|-----------------|
| This story spec | ~2,500 |
| `cliff.toml` | ~800 |
| `CHANGELOG.md` (relevant section + existing content) | ~3,000 |
| ADR-063 D6 section | ~1,200 |
| git-cliff dry-run output (sample) | ~2,000 |
| Total | ~9,500 |

Well within the 30% context window budget.

---

## Red Gate Test List (SAC-1)

**tdd_mode: facade — Red Gate is N/A.** No Rust production code; no Red Gate tests.
Verification is operational (implementer must explicitly perform and document each check):
- AC-001: `grep '## \[1.0.0-beta.1\]' CHANGELOG.md` → exactly 1 match
- AC-002: `grep '### Highlights' CHANGELOG.md` → match inside `## [1.0.0-beta.1]` section (before next `## [` header)
- AC-003: confirm `## [1.0.0-beta.1]` header precedes first `### Highlights` in file; no `### Highlights` appears above it
- AC-004: `grep -E '### (Added|Fixed|Performance|Changed|Security|Breaking Changes)' CHANGELOG.md` → ≥1 match inside beta.1 section
- AC-005: count `-` bullet lines in `### Highlights` section → 5–8 bullets
- Dry-run `git cliff --tag v1.0.0-beta.1 --unreleased --output /dev/stdout` output snippet documented in PR description

---

## Tasks

1. **Determine the tag range (`--unreleased` handles both cases — do NOT use `--latest`):**
   Check if `v1.0.0-rc.1` exists as a git tag:
   - If yes: `--unreleased` selects commits since the last existing tag (v1.0.0-rc.1..HEAD).
     Dry-run invocation: `git cliff --tag v1.0.0-beta.1 --unreleased --output /dev/stdout`.
   - If no: `--unreleased` selects all commits from the initial commit.
     Dry-run invocation: `git cliff --tag v1.0.0-beta.1 --unreleased --output /dev/stdout`.
   In both cases the invocation is identical: `--tag v1.0.0-beta.1 --unreleased`.
   DO NOT use `--latest`; `--latest` requires the tag to already exist on HEAD and is
   unreliable for the pre-tag first-release scenario (ADR-063 D5/D6).

2. **Run the dry-run pass:**
   ```bash
   git cliff --tag v1.0.0-beta.1 --output /dev/stdout
   ```
   (Or use `--unreleased --output /dev/stdout` if the tag does not yet exist.)
   Inspect the output for noise (excessive chore/ci/docs entries that passed the skip
   rules).

3. **Noise control (if needed):**
   If the output is still noisy, add additional skip rules to `cliff.toml` per ADR-063 D6.
   Common additions: skip test commits, skip MCP-related infra changes, skip any
   project-specific categories. Commit the updated `cliff.toml` before continuing.

4. **Generate Layer-2 content:**
   Run:
   ```bash
   git cliff --tag v1.0.0-beta.1 --prepend CHANGELOG.md
   ```
   This prepends the `## [1.0.0-beta.1]` block (with categorized commit sections) to
   `CHANGELOG.md`.

5. **Dispatch `vsdd-factory:technical-writer` for Layer-1 draft:**
   Provide the agent:
   - The git-cliff output (Layer-2 body) as context
   - Instruction: draft `### Highlights` (5-8 bullets), `### Breaking Changes (narrative)`
     (if any BREAKING CHANGE commits exist), and `### Upgrade Notes` (if any migration
     steps needed); document current behavior only; no aspirational features.
   - Target placement: INSIDE the `## [1.0.0-beta.1]` block, BEFORE the first
     git-cliff `###` section.

6. **Insert Layer-1 draft into CHANGELOG.md:**
   Edit `CHANGELOG.md` to insert the technical-writer draft:
   ```
   ## [1.0.0-beta.1] - YYYY-MM-DD

   ### Highlights
   [technical-writer draft — 5-8 bullets]

   ### Breaking Changes (narrative)
   [technical-writer draft — or OMIT if empty]

   ### Upgrade Notes
   [technical-writer draft — or OMIT if empty]

   ### Breaking Changes (from commits)
   [git-cliff BREAKING CHANGE footers — if any]

   ### Added
   [git-cliff feat commits]

   ...
   ```

7. **Human curation gate (in PR description):**
   The PR description must include a review checklist:
   - `[ ] Layer-1 Highlights reviewed and curated`
   - `[ ] Layer-1 Breaking Changes (narrative) reviewed (or confirmed empty)`
   - `[ ] Aspirational features removed from Highlights`
   - `[ ] Layer-2 commit categorization spot-checked`

8. **Verify AC-001..AC-005.**

---

## Acceptance Criteria

### AC-001: CHANGELOG.md contains ## [1.0.0-beta.1] section
`grep '## \[1.0.0-beta.1\]' CHANGELOG.md` returns exactly one match.
(traces to ADR-063 D6 — first-release handling produces the beta.1 section)

### AC-002: beta.1 section contains Layer-1 ### Highlights
`grep '### Highlights' CHANGELOG.md` returns a match inside the `## [1.0.0-beta.1]`
section (i.e., before the next `## [` header). (traces to ADR-063 D4 — Layer-1
### Highlights section mandatory for beta.1)

### AC-003: Layer-1 sections are INSIDE the ## [1.0.0-beta.1] block (not above it)
The `## [1.0.0-beta.1]` header precedes the first `### Highlights` in the file.
No `### Highlights` appears above `## [1.0.0-beta.1]`.
(traces to ADR-063 D4 v1.1 — Layer-1 inside ## [VERSION] block)

### AC-004: Layer-2 body contains at least one categorized section
`grep -E '### (Added|Fixed|Performance|Changed|Security|Breaking Changes)' CHANGELOG.md`
returns at least one match inside the `## [1.0.0-beta.1]` section.
(traces to ADR-063 D3 — git-cliff categorized body)

### AC-005: Layer-1 Highlights is 5-8 bullets maximum
The `### Highlights` section under `## [1.0.0-beta.1]` contains 5-8 bullet items
(lines starting with `-`). (traces to ADR-063 D6 §3 — "intentionally short Highlights
block; 5-8 bullets maximum")

---

## Previous Story Intelligence

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| S-REL-CLIFF-001 | cliff.toml created; git-cliff 2.14.1 installed | Use `git cliff --output /dev/stdout` for dry-run; `--prepend` for in-place update | Do NOT combine `--output` and `--prepend` |
| S-REL-WRITER-001 | Layer-1 top-block contract defined in RELEASING.md §5 | technical-writer drafts 5-8 bullet Highlights; inserts INSIDE ## [VERSION] block | Empty sections are omitted |

---

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| Full history range for first release (no --latest) | ADR-063 D6 §1 | git cliff invocation without --latest |
| Dry-run BEFORE tag is cut | ADR-063 D6 §2 | Task 2 runs before tag creation |
| Human curation gate MANDATORY | ADR-063 D6 §3 | PR checklist per Task 7 |
| No retroactive commit re-wording | ADR-063 D6 §4 | Authors CHANGELOG only; does not amend git history |
| Layer-1 inside ## [VERSION] block | ADR-063 D4 v1.1 | AC-003 structural check |

---

## Library & Framework Requirements

| Dependency | Version | Notes |
|------------|---------|-------|
| `git-cliff` | `2.14.1` | Must be installed (`cargo install git-cliff --version 2.14.1 --locked`); established by S-REL-CLIFF-001 |
| `vsdd-factory:technical-writer` | N/A | Agent dispatched manually for Layer-1 draft |

---

## File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `CHANGELOG.md` | Modify | Prepend `## [1.0.0-beta.1]` section with Layer-1 + Layer-2 content |
| `cliff.toml` | Possibly modify | Additional skip rules if dry-run is noisy (ADR-063 D6 §2 noise control) |

---

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| `CHANGELOG.md` beta.1 section | repo root | Pure (text document; no runtime behavior) |
| `git-cliff` dry-run + generate | CLI tool | Effectful (reads git history; writes to CHANGELOG.md) |

---

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| `CHANGELOG.md` | pure-core | Static text document; no runtime behavior |
| `cliff.toml` (possible skip rule additions) | pure-core | Static TOML configuration |
| `git cliff` CLI operations | effectful-shell | Reads git history and writes CHANGELOG.md |

---

## Holdout Applicability

**Determination: N/A**

The story-level holdout gate (CLAUDE.md, human-approved 2026-07-13) requires a built
prism binary and an MCP-visible surface with wire-level assertions. This story has
`tdd_mode: facade` and `crates_touched: []` — it produces no prism binary. Its entire
deliverable is CHANGELOG.md content (Layer-2 git-cliff output + Layer-1 human-drafted
narrative).

Three reasons the gate does not apply:

1. **No built prism binary.** The story's outputs are static text files. git-cliff is
   an external CLI tool, not the prism MCP binary. No MCP stdio surface is created or
   exercised.

2. **All structural verification is implementer-visible.** AC-001..AC-005 are grep/count
   assertions on CHANGELOG.md that the implementer explicitly performs. There is no
   hidden machine-verifiable surface with genuine info-asymmetry.

3. **Layer-1 content requires human editorial judgment.** The story mandates a human
   curation gate (`estimated_passes: "1 human-in-the-loop cycle"`; ADR-063 D6 §3
   "MANDATORY human curation gate for beta.1"). The Layer-1 Highlights draft is authored
   by vsdd-factory:technical-writer, reviewed and curated by the release engineer in the
   PR. A machine holdout cannot substitute for this human gate and must NOT be fabricated
   as a substitute for it.

`holdout_scenarios: []` — no scenarios authored; HOLDOUT-INDEX unchanged.

---

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | v1.0.0-rc.1 tag does NOT exist in the repo | Use full history from initial commit; output may be very long; noise control is critical |
| EC-002 | v1.0.0-rc.1 tag EXISTS | Range is v1.0.0-rc.1..v1.0.0-beta.1; use `--unreleased --tag v1.0.0-beta.1` (git-cliff auto-selects commits since the last existing tag via `--unreleased`). DO NOT use `--latest` (ADR-063 D5/D6) |
| EC-003 | No BREAKING CHANGE commits in the range | Omit `### Breaking Changes (narrative)` and `### Breaking Changes (from commits)` sections |
| EC-004 | Technical-writer draft includes aspirational features | Human curation gate removes them before merge |
| EC-005 | Layer-2 output is empty (all commits were skipped) | Do not ship an empty section; add representative feat/fix entries manually or revisit skip rules |

---

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.1 | 2026-09-06 | story-writer | Sweep #13: Task 1 `--latest` reference replaced with `--unreleased --tag` (both cases); EC-002 updated to use `--unreleased --tag v1.0.0-beta.1` per ADR-063 D5/D6; Red Gate N/A (facade) note made explicit with enumerated verification steps; status draft→ready |
| 1.0 | 2026-09-05 | story-writer | Initial — ADR-063 D6 first-release handling materialization |
