---
document_type: story
story_id: S-REL-011
title: "docs: operator setup and installation documentation for v1.0.0 archive-bundled model"
wave: post-v1.0
epic_id: E-REL
priority: P1
status: draft
# BC status: N/A — operator setup documentation is a process/docs artifact.
# No subsystem behavioral contract governs installation documentation.
# Conforming per S-REL-005 / S-REL-006 precedent (behavioral_contracts: [] for pure-docs stories).
version: "0.1"
level: "L4"
producer: story-writer
timestamp: "2026-09-03T00:00:00Z"
modified: "2026-09-03"
phase: 3
cycle: v1.0.0-brownfield
tdd_mode: strict
# tdd_mode: strict — document content authoring uses strict discipline to verify all
# referenced file paths, commands, and steps are accurate. Red Gate test is a CI
# completeness check (all required sections exist, no broken internal references).
subsystems: []
# Subsystem anchor justification:
#   docs/SETUP.md is operator-facing installation documentation. No ARCH-INDEX subsystem
#   owns installation runbooks. subsystems: [] per S-REL-005 / S-REL-006 docs-story precedent.
crates_touched: []
target_module: docs
capabilities: []
behavioral_contracts: []
verification_properties: []
holdout_scenarios: []
assumption_validations: []
risk_mitigations:
  - "Document the v1.0.0 archive-bundled model accurately: operators must manually extract
    specs/ from the demo bundle and place it at spec_dir. Do NOT describe the future embedded
    model (S-REL-010) in this document — that model does not exist in v1.0.0."
  - "Command accuracy: every command in the setup guide must be verified against the actual
    v1.0.0 binary and scripts before the story is closed. Aspirational commands that differ
    from the real CLI output are a defect (S-REL-005 lesson: 'RELEASING.md must reference
    actual file paths, not aspirational ones')."
  - "prism.toml.example accuracy: the setup guide references prism.toml.example for the
    config template. The example file must exist and be complete before this story ships.
    If prism.toml.example is missing or incomplete, that is a blocker (not a doc-only deferral)."
  - "UUID-v7 org_id generation: the setup guide must document the exact command or tool
    for generating a UUID v7 (e.g., a CLI tool, a Rust snippet, or an online generator).
    Do NOT recommend UUID v4 — org_id must be monotonically sortable per prism's identity model."
  - "Attestation verification: the guide must cite `gh attestation verify` with the exact
    flags for verifying sigstore build provenance. Verify the command against the actual
    v1.0.0 attestation before documenting."
  - "This document describes the v1.0.0 model. When S-REL-010 (embedded specs) ships in a
    future release, this document will need a revision removing the 'extract specs/' step.
    Add a note in the doc: 'Note: future releases will embed built-in specs directly in the
    binary; the spec extraction step will become optional.'"
depends_on: [S-REL-003, S-REL-005]
# Dependency anchor justifications:
#   depends_on S-REL-003: SETUP.md documents the install.sh / install.ps1 scripts as the
#     primary consumer download path. S-REL-003 creates these scripts. The setup guide
#     cannot accurately describe commands that do not exist yet.
#   depends_on S-REL-005: SETUP.md cross-references RELEASING.md (the operator runbook) and
#     must not contradict its version bump or tagging procedure. S-REL-005 also creates
#     prism.toml.example (boot.rs CWD fix + example content), which SETUP.md references as
#     the config template. The setup guide cannot accurately describe prism.toml configuration
#     until prism.toml.example exists with correct content.
blocks: []
points: 3
estimated_days: 1
risk: LOW
# Risk justification: Documentation-only story. No Rust code changes. No CI changes.
# Primary risk is command inaccuracy — mitigated by verify-before-document discipline.
acceptance_criteria_count: 0
# acceptance_criteria_count: 0 — draft stub; ACs to be authored when story is materialized.
red_gate_tests: 0
# red_gate_tests: 0 — draft stub. One CI-enforced Red Gate test (section completeness check)
# expected at scheduling time per S-REL-005/S-REL-006 docs-story pattern.
# SAC-1: Enumerated RG-001..RG-NNN list and density check required before status: ready.
inputs:
  - "scripts/install.sh"
  - "scripts/install.ps1"
  - "prism.toml.example"
  - "docs/DEMO-RUNBOOK.md"
  - "docs/RELEASING.md"
input-hash: "[pending-recompute]"
traces_to: []
cycle_note: "D-2439 human-directed deferral 2026-09-03 — v1.0.0 ships the archive bundle but lacks a setup/installation guide; this story closes that doc gap."
---

# S-REL-011 — Operator Setup and Installation Documentation (v1.0.0 Model)

> **DRAFT STUB — NOT FOR IMMEDIATE DELIVERY.** This story captures the human-directed
> post-v1.0.0 deferral (D-2439, 2026-09-03). The v1.0.0 release archive bundles specs/
> and infusions/ but provides no installation guide for new operators. This story
> closes that gap. PO may waive BC authorship for docs stories (see behavioral_contracts
> note above). Status transitions to `ready` when scope is confirmed and RG list authored.
>
> SAC-1: Before status: ready, an enumerated Red Gate list (RG-001..RG-NNN) and
> BC-5.38.001 density check paragraph are required.

## Authority

- **S-REL-003** — install.sh / install.ps1 scripts (checksum-verified consumer install paths)
- **S-REL-004** — demo bundle packaging (bundle structure: specs/, infusions/, plugins/, scripts)
- **S-REL-005** — docs/RELEASING.md (operator runbook; SETUP.md complements this)
- **docs/DEMO-RUNBOOK.md** — demo execution guide (SETUP.md covers pre-demo installation)
- **D-2439** — human-directed deferral origin (2026-09-03)
- **D-2440** — renumbered S-REL-009→S-REL-011 (S-REL-009 slot reserved for registry-publish meaning); depends_on updated S-REL-004→S-REL-005

---

## Narrative

As a new MSSP operator deploying Prism for the first time, I want a step-by-step setup
and installation guide, so that I can go from a downloaded release archive to a running
`prism start` with working sensor connections without consulting support or guessing at
the correct file layout.

---

## Background

The v1.0.0 release ships the prism binary and a per-platform demo bundle that includes:
- `specs/` — built-in sensor specs (claroty, crowdstrike, cyberint, armis)
- `infusions/` — enrichment specs (threatintel, nvd)
- `prism.toml.example` — config template
- `plugins/` — WASM plugins
- Demo scripts

However, no `docs/SETUP.md` exists. Operators must currently:
1. Infer the file layout from the demo bundle structure
2. Discover the `prism credential set` command independently
3. Know to generate a UUID-v7 org_id without guidance

This gap was called out during v1.0.0 release prep (D-2439, 2026-09-03):
> "capture what to do with those specs as part of the setup and installation documentation
> we still need to create."

---

## Scope

Create `docs/SETUP.md` covering the complete new-operator setup flow for v1.0.0:

1. **Downloading the release** — via `install.sh` (Unix) or `install.ps1` (Windows), or
   direct download from the GitHub Release page.

2. **Verifying checksums and build provenance** — SHA-256 checksum file + `gh attestation
   verify` for sigstore build attestation.

3. **Extracting the demo bundle** — extract the per-platform `.tar.gz` / `.zip` archive;
   identify the `specs/`, `infusions/`, and `prism.toml.example` contents.

4. **Placing `specs/` at `spec_dir`** — create the `spec_dir` path referenced in prism.toml
   and copy the extracted `specs/` TOML files there.

5. **Placing `infusions/` alongside prism.toml** — copy the extracted infusion specs to the
   infusions directory configured in prism.toml.

6. **Filling out `prism.toml`** — copy `prism.toml.example` to `prism.toml`, fill in
   `spec_dir`, `infusions_dir`, listener address, log level, and org block.

7. **Generating a UUID-v7 `org_id`** — document the recommended tool/command for generating
   a UUID v7 (monotonically sortable; not UUID v4).

8. **Configuring credentials** — `prism credential set <sensor> --key <KEY>` for each
   configured sensor; document the credential key names per sensor.

9. **First boot** — `prism start`, verify startup log output, run a test query via MCP
   or the `prism query` CLI to confirm sensor connectivity.

10. **Next steps** — pointer to `docs/DEMO-RUNBOOK.md` for the full demo workflow;
    pointer to `docs/RELEASING.md` for upgrade and release procedure.

---

## Acceptance Criteria (sketch — authoritative ACs authored when story is materialized)

*These are scope sketches, NOT final ACs.*

- AC-001 (placeholder): `docs/SETUP.md` exists with all 10 required sections documented above.
- AC-002 (placeholder): Every CLI command in SETUP.md has been verified against the actual
  v1.0.0 binary output (no aspirational / unverified commands).
- AC-003 (placeholder): SETUP.md documents the exact `gh attestation verify` command with
  correct flags for the v1.0.0 release.
- AC-004 (placeholder): SETUP.md documents UUID-v7 (not UUID v4) for org_id generation,
  with the recommended tool/command.
- AC-005 (placeholder): SETUP.md correctly describes the v1.0.0 archive-bundled spec model
  (manual extraction of specs/ into spec_dir) and includes a forward-looking note that future
  releases will embed built-in specs (S-REL-010).
- AC-006 (placeholder): All internal cross-references in SETUP.md (to DEMO-RUNBOOK.md,
  RELEASING.md, prism.toml.example) resolve to files that actually exist.

---

## Behavioral Contracts

This story has no subsystem BCs — operator setup documentation is a process artifact.
Conforming per S-REL-005 / S-REL-006 precedent.

| Architecture Source | Clause |
|--------------------|--------|
| `S-REL-003` | install.sh / install.ps1 scripts are the documented consumer install path |
| `S-REL-004` | Demo bundle archive layout (specs/, infusions/, plugins/, prism.toml.example) |
| `S-REL-005 §RELEASING.md` | SETUP.md must not contradict RELEASING.md version bump or tagging procedure |
| `prism.toml.example` | Config template; SETUP.md references this as the starting point |

---

## Token Budget Estimate

| Artifact | Estimated Tokens |
|----------|-----------------|
| This story spec | ~2,500 |
| `scripts/install.sh` + `scripts/install.ps1` (~200 lines combined) | ~3,000 |
| `prism.toml.example` (~60 lines) | ~800 |
| `docs/DEMO-RUNBOOK.md` (cross-reference scan only) | ~1,000 |
| `docs/RELEASING.md` (cross-reference scan only) | ~1,200 |
| Total | ~8,500 |

Well within the 30% context window budget.

---

## Tasks

*N/A — draft stub. Tasks authored when story is materialized for delivery.*

Likely task structure (for planning purposes only — NOT authoritative):

1. Read `scripts/install.sh`, `scripts/install.ps1` to document actual download + install commands.
2. Read `prism.toml.example` to document all required config fields.
3. Research / confirm UUID-v7 generation tool recommendation.
4. Verify `gh attestation verify` command flags against an actual v1.0.0 release asset.
5. Write Red Gate test: section completeness check script (`grep` all required H2 headings).
6. Create `docs/SETUP.md` with all 10 sections.
7. Verify every command in SETUP.md against the actual binary.
8. Cross-check all internal links resolve (DEMO-RUNBOOK.md, RELEASING.md, prism.toml.example).
9. Make Red Gate test green.
10. LOCAL adversary review.

---

## Previous Story Intelligence

N/A — first documentation story in the post-v1.0 release wave.
Prior docs stories S-REL-005 (RELEASING.md) and S-REL-006 (consumer-contract graduation)
established the pattern: documentation stories use `behavioral_contracts: []`, `tdd_mode: strict`,
one Red Gate test (section completeness), and `subsystems: []`.

---

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| Describe v1.0.0 archive-bundled model accurately | Risk mitigation #1 | AC-002: every command verified |
| UUID-v7 for org_id (not v4) | Identity model (prism-core) | AC-004 |
| No aspirational commands | S-REL-005 lesson | AC-002 |
| All cross-references resolve | Canonical Principle Rule 3 | AC-006 |
| Forward-looking S-REL-010 note | Scope item 10 | AC-005 |

---

## Library & Framework Requirements

No code dependencies — documentation only.

---

## File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `docs/SETUP.md` | Create | New operator installation guide; 10-section structure defined in scope above |

---

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|----------------|
| `docs/SETUP.md` | `docs/` | N/A (documentation) |

---

## Purity Classification

| Module | Classification | Justification |
|--------|----------------|---------------|
| `docs/SETUP.md` | N/A | Documentation — no Rust purity boundary applies |

---

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Operator uses UUID v4 for org_id instead of UUID v7 | SETUP.md explicitly notes UUID v4 is NOT recommended (not monotonically sortable) |
| EC-002 | Windows operator (PowerShell) vs Unix operator (bash) | SETUP.md has platform-specific sections for download + extraction commands |
| EC-003 | S-REL-010 lands before S-REL-011 is written | SETUP.md author must assess whether the spec extraction step still applies; see risk mitigation #1 |
| EC-004 | prism.toml.example does not yet exist when S-REL-011 is delivered | Blocker — see risk mitigation #3; do NOT write SETUP.md with aspirational prism.toml.example contents |

---

## Forbidden Dependencies

- No Rust code changes (documentation story only).
- No modifications to CI/CD or release scripts (those are owned by S-REL-001 through S-REL-007).
- Do NOT describe the future S-REL-010 embedded-specs model as if it exists in v1.0.0.

---

## Changelog

| Version | Date | Summary |
|---------|------|---------|
| 0.1 | 2026-09-03 | Initial draft stub — D-2439 human-directed post-v1.0.0 deferral (2026-09-03). Captures scope, design constraints, and acceptance sketch. D-2440: renumbered S-REL-009→S-REL-011 (slot collision avoidance); depends_on updated [S-REL-003, S-REL-004]→[S-REL-003, S-REL-005] (prism.toml.example + RELEASING.md deps). |
