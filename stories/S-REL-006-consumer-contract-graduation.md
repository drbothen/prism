---
document_type: story
story_id: S-REL-006
title: "devops: graduate prism-consumer-contract.md to docs/consumer-contract.md + DEMO-RUNBOOK.md Windows update"
wave: F-B
epic_id: E-REL
priority: P1
status: draft
version: "0.1"
level: "L4"
producer: story-writer
timestamp: "2026-07-19T00:00:00Z"
tdd_mode: strict
subsystems: []
# Subsystem anchor justification:
#   docs/consumer-contract.md and DEMO-RUNBOOK.md Windows section are consumer-facing
#   documentation artifacts. No ARCH-INDEX subsystem owns consumer contracts or runbooks.
#   subsystems: [] per S-0.01 infra story precedent.
crates_touched: []
target_module: devops
capabilities: []
behavioral_contracts: []
# BC status: N/A — consumer contract graduation is documentation. No subsystem BC governs
# consumer-facing docs. Conforming per W3-FIX-CI-001 precedent.
verification_properties: []
depends_on: [S-REL-002, S-REL-007]
# Dependency anchor justifications:
#   depends_on S-REL-002: consumer-contract.md §5.2 pins `prism --version` to `prism 1.0.0-rc.1`.
#     This can only be finalized after S-REL-002 establishes the canonical version string.
#     The graduated contract must cite the actual version output, not a placeholder.
#   depends_on S-REL-007: DEMO-RUNBOOK.md already gets its Windows section from S-REL-007.
#     S-REL-006 cross-references the Windows section in the graduated consumer contract;
#     the Windows demo must exist before the consumer contract can reference it accurately.
blocks: []
points: 2
estimated_days: 1
risk: LOW
# Risk justification: Documentation copy/edit only. No Rust code changes. No CI changes.
# The consumer contract content already exists in .factory/planning/; this story graduates
# it to docs/ with any needed updates (version pin, Windows demo reference).
acceptance_criteria_count: 6
red_gate_tests: 1
estimated_passes: "1 LOCAL adversary pass"
holdout_scenarios: []
assumption_validations: []
risk_mitigations:
  - "Version pin accuracy: consumer-contract.md §5.2 must cite `prism 1.0.0-rc.1` as the
    exact --version output. Read the actual prism binary (or prism-bin/Cargo.toml) to
    confirm before writing, not this story's text."
  - "Windows demo section cross-reference: if S-REL-007's DEMO-RUNBOOK.md Windows section
    uses a heading name different from '## Windows (PowerShell)', update the cross-reference
    in consumer-contract.md to match the actual heading."
  - "planning/ artifact remains: the original .factory/planning/feature-release-engineering/
    prism-consumer-contract.md is NOT deleted — it is a planning artifact and part of the
    delta-analysis input record. docs/consumer-contract.md is a new graduated copy."
inputs:
  - ".factory/planning/feature-release-engineering/prism-consumer-contract.md"
  - ".factory/planning/feature-release-engineering/secops-factory-handoff-brief.md"
  - "docs/DEMO-RUNBOOK.md"
input-hash: "a1b92a0"
traces_to: []
cycle: "v1.0.0-release-engineering"
phase: "F3"
---

# S-REL-006 — devops: graduate prism-consumer-contract.md to docs/consumer-contract.md

**Story ID:** S-REL-006
**Status:** draft
**Version:** v0.1
**Wave:** F-B
**Priority:** P1
**Points:** 2

---

## Origin

Delta-analysis §6 (S-REL-006 scope): `prism-consumer-contract.md` currently lives in
`.factory/planning/feature-release-engineering/` — a planning artifact, not a consumer-facing
document. The secops-factory `activate` skill references a consumer contract. For RC, the
contract must be at `docs/consumer-contract.md` where external users can find it, with the
version pin updated to match the actual 1.0.0-rc.1 release and a Windows demo cross-reference
added.

Wave F-B placement: this story depends on S-REL-002 (version alignment) and S-REL-007
(Windows demo scripts), which are Wave F-A. Consumer contract graduation cannot be finalized
until the version string it pins and the Windows demo it references are both complete.

---

## Narrative

As a secops-factory user, I want a stable `docs/consumer-contract.md` that documents the
prism MCP launch shape, credential model, and version expectations, so that I can integrate
prism into my toolchain without consulting internal planning files.

---

## Behavioral Contracts

This story has no subsystem BCs — consumer contract graduation is documentation.

| Architecture Source | Clause |
|--------------------|--------|
| `prism-consumer-contract.md` §1-§5 | Source content to graduate |
| `prism-consumer-contract.md` §5.2 | Version pin: `prism 1.0.0-rc.1` |
| `secops-factory-handoff-brief.md` §2 | `activate` skill version check references consumer contract |
| `delta-analysis.md` §6 S-REL-006 scope | Graduate to docs/; pin version; Windows demo update |

---

## Token Budget Estimate

| Artifact | Estimated Tokens |
|----------|-----------------|
| This story spec | ~2,000 |
| `prism-consumer-contract.md` (source, full) | ~4,000 |
| `secops-factory-handoff-brief.md` (reference) | ~2,000 |
| `docs/DEMO-RUNBOOK.md` (Windows section heading) | ~500 |
| Total | ~8,500 |

Well within the 30% context window budget.

---

## Tasks

1. **Read `.factory/planning/feature-release-engineering/prism-consumer-contract.md`** in full.

2. **Read `docs/DEMO-RUNBOOK.md`** to find the exact heading of the Windows demo section
   (added by S-REL-007) to cross-reference accurately.

3. **Create `docs/consumer-contract.md`** by graduating the planning artifact:
   - Copy all content from `prism-consumer-contract.md`
   - Update §5.2 version pin to confirm `prism --version` outputs `prism 1.0.0-rc.1`
     (this should already match; verify against crates/prism-bin/Cargo.toml)
   - Add a note at the top: "This document is the canonical consumer contract for prism
     1.0.0-rc.1. For the planning artifact that preceded this, see
     `.factory/planning/feature-release-engineering/prism-consumer-contract.md`."
   - Add a cross-reference in the Demo section pointing to `docs/DEMO-RUNBOOK.md`
     including the Windows section heading (from S-REL-007)
   - Do NOT delete or modify the original planning file

4. **Update `docs/DEMO-RUNBOOK.md`** (minimal change): Add a note at the top of the runbook
   referencing the consumer contract for environment variable and credential setup:
   "See `docs/consumer-contract.md` for the full MCP launch shape and credential model."

   NOTE: If S-REL-007 already adds a Windows section to DEMO-RUNBOOK.md, this task is
   additive — do NOT remove or replace S-REL-007's additions.

---

## Acceptance Criteria

### AC-001: `docs/consumer-contract.md` exists
Given: The story is implemented.
When: `ls docs/consumer-contract.md` is run.
Then: File exists. It is NOT empty.
(traces to delta-analysis.md §6 S-REL-006: "graduate to docs/consumer-contract.md")

### AC-002: consumer-contract.md contains all sections from the planning artifact
Given: `docs/consumer-contract.md` is read.
When: The section headers are compared to `.factory/planning/feature-release-engineering/prism-consumer-contract.md`.
Then: All sections from the planning artifact are present. No content is omitted.
The graduated doc must be a superset (not a subset) of the original.
(traces to prism-consumer-contract.md: full content graduation required)

### AC-003: Version pin is `prism 1.0.0-rc.1`
Given: `docs/consumer-contract.md` is read.
When: `grep '1.0.0-rc.1' docs/consumer-contract.md` is run.
Then: At least one match, specifically in the version/versioning section (§5.2 equivalent).
The document pins `prism --version` output to `prism 1.0.0-rc.1`.
(traces to prism-consumer-contract.md §5.2: version pin; delta-analysis §6: "pin version")

### AC-004: consumer-contract.md cross-references DEMO-RUNBOOK.md
Given: `docs/consumer-contract.md` is read.
When: `grep 'DEMO-RUNBOOK' docs/consumer-contract.md` is run.
Then: At least one reference to `docs/DEMO-RUNBOOK.md` is present, pointing consumers to
the demo runbook for the setup walkthrough.
(traces to delta-analysis §6: "Windows demo cross-reference")

### AC-005: Original planning artifact is NOT deleted
Given: The story is implemented.
When: `ls .factory/planning/feature-release-engineering/prism-consumer-contract.md` is run.
Then: File still exists. It is unchanged from before this story ran.
(traces to risk_mitigations: "planning/ artifact remains — planning audit record preserved")

### AC-006: DEMO-RUNBOOK.md references consumer-contract.md for credential setup
Given: `docs/DEMO-RUNBOOK.md` is read.
When: `grep 'consumer-contract' docs/DEMO-RUNBOOK.md` is run.
Then: At least one match, pointing to `docs/consumer-contract.md` for MCP launch shape
and credential model.
(traces to delta-analysis §6: "DEMO-RUNBOOK.md cross-reference update")

---

## Previous Story Intelligence

S-REL-007 modifies `docs/DEMO-RUNBOOK.md` by adding a `## Windows (PowerShell)` section.
This story makes a SEPARATE, ADDITIVE edit to DEMO-RUNBOOK.md (adding a consumer-contract
cross-reference at the top). The implementer must read the post-S-REL-007 state of
DEMO-RUNBOOK.md before editing to avoid clobbering the Windows section.

---

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| Original planning file not deleted | risk_mitigations | AC-005 |
| Graduate is superset of original | delta-analysis §6 | AC-002 |
| Version pin matches actual binary output | S-REL-002 AC-002 | AC-003 |
| DEMO-RUNBOOK.md edit is additive only | Previous story intelligence | AC-006 |

---

## Library & Framework Requirements

No code dependencies — documentation copy/edit only.

---

## File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `docs/consumer-contract.md` | Create | Graduated from planning artifact; superset |
| `docs/DEMO-RUNBOOK.md` | Modify | Add consumer-contract.md cross-reference at top |
| `.factory/planning/feature-release-engineering/prism-consumer-contract.md` | DO NOT TOUCH | Planning record; AC-005 verifies it is unchanged |

---

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|----------------|
| `docs/consumer-contract.md` | `docs/` | N/A (documentation) |
| `docs/DEMO-RUNBOOK.md` (additive edit) | `docs/` | N/A (documentation) |

---

## Purity Classification

| Module | Classification | Justification |
|--------|----------------|---------------|
| `docs/consumer-contract.md` | N/A | Documentation — no Rust purity boundary applies |
| `docs/DEMO-RUNBOOK.md` | N/A | Documentation — no Rust purity boundary applies |

---

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | S-REL-007 heading name differs from assumed | Read actual DEMO-RUNBOOK.md heading; use exact heading in cross-reference |
| EC-002 | DEMO-RUNBOOK.md already has a consumer-contract reference (from S-REL-007) | Skip additive edit; AC-006 passes either way |
| EC-003 | Future 1.0.0 release requires version bump in consumer-contract.md | Implementer notes in §5 that version pin is release-specific; each release graduation updates it |

---

## Forbidden Dependencies

- No Rust code changes (documentation only)
- No deletion of `.factory/planning/` artifacts
- No modification to `.factory/planning/feature-release-engineering/prism-consumer-contract.md`

---

## Changelog

| Version | Date | Summary |
|---------|------|---------|
| 0.1 | 2026-07-19 | Initial story creation (story-writer F3 burst) |
