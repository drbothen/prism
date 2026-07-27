---
document_type: story
story_id: "S-MAINT-RG-LIST-GATE-001"
title: "Factory Validator Gate — Enforce Enumerated Red Gate List + BC-5.38.001 Density Check Before status: ready on tdd_mode: strict Stories"
wave: tbd
epic_id: maintenance
priority: P2
status: draft
version: "0.1"
level: ops
producer: story-writer
timestamp: "2026-07-26"
modified: "2026-07-26"
input-hash: "[live-state]"
inputs: []
traces_to: ""
cycle: "wave-5-e-demo-fidelity"
phase: 3
tdd_mode: strict
track: "Platform Engineering"
subsystems: []
crates_touched: []
target_module: ".factory/hooks, drbothen/vsdd-factory upstream"
capabilities: []
behavioral_contracts: []
# BC status: pending PO authorship
verification_properties: []
depends_on: []
blocks: []
points: 2
estimated_days: 0.5
risk: MEDIUM
acceptance_criteria_count: 4
red_gate_tests: 0
estimated_passes: "tbd"
holdout_scenarios: []
assumption_validations: []
risk_mitigations: []
tags:
  - process-gap
  - factory-tooling
  - tdd-discipline
---

# S-MAINT-RG-LIST-GATE-001: Factory Validator Gate — Enforce Enumerated Red Gate List + BC-5.38.001 Density Check Before status: ready on tdd_mode: strict Stories

## Origin

**Process-gap finding:** F-WASE-P64-MED-016 (LOCAL pass 64, Wave-A spec-evolution cascade, 2026-07-26)

Six of seven perimeter stories were `tdd_mode: strict` with no enumerated Red Gate test list and no BC-5.38.001 density check. Only `S-WAVE-A-ENGINE-001` had the correct structure (RG-001..RG-039 format, density count, implementation tasks following test tasks). The other six stories embedded test-writing tasks inline with implementation tasks, inverting the red-then-green ordering that TDD requires.

FB61 applied corrective edits to all six outlier stories. This maintenance story proposes a structural gate so the gap cannot recur: before a `tdd_mode: strict` story can advance to `status: ready`, a factory validator checks for the presence of an enumerated Red Gate test list and a BC-5.38.001 density check paragraph.

---

## Narrative

As an orchestrator dispatching a `tdd_mode: strict` story to the test-writer,
I want the factory dispatch system to block `status: ready` on any story that lacks an enumerated Red Gate test list and a BC-5.38.001 density check,
so that test-writer agents always receive an explicit list of named failing tests to write before implementation begins — preventing the pattern where test-authoring is embedded inside implementation tasks and the red-gate phase is skipped or inverted.

---

## Acceptance Criteria

### AC-001 — Validator gate detects tdd_mode: strict stories missing a Red Gate section
(Traceability to BCs is pending PO authorship)

A factory validator script (or factory-dispatcher hook plugin) MUST reject a `status: ready` transition for any story where:
- `tdd_mode: strict` is set in frontmatter, AND
- The story body does NOT contain a `### Red Gate tests` section heading (or equivalent heading matching the ENGINE-001 normative pattern), OR
- The `### Red Gate tests` section body contains zero `- [ ] **RG-` checklist entries

The rejection produces a human-readable error identifying the story ID, the missing section, and a pointer to `S-WAVE-A-ENGINE-001` as the normative pattern.

### AC-002 — Validator gate detects missing BC-5.38.001 density check paragraph
(Traceability to BCs is pending PO authorship)

The same gate MUST also reject `status: ready` if the `### Red Gate tests` section is present but does NOT contain a `**Red Gate density check**` paragraph (matching the phrase `BC-5.38.001`) with an explicit RGT count and computed density ratio (e.g., `X / Y ACs = Z`).

A density ratio below 0.5 (fewer than 0.5 RGTs per AC) MUST produce a warning but MUST NOT block dispatch — the density requirement is a soft floor per BC-5.38.001; low-complexity stories with density between 0.3 and 0.5 may be valid. Absent density check paragraph entirely: hard block.

### AC-003 — Gate is non-blocking for tdd_mode: facade stories
(Traceability to BCs is pending PO authorship)

Stories with `tdd_mode: facade` in frontmatter MUST NOT be blocked by this gate. Facade-mode stories use a combined scaffold+impl delivery model (DTU API clones, mock servers, structural fakes) and are exempt from the Red Gate list requirement per the `tdd_mode: facade` definition in the story template spec.

### AC-004 — Upstream issue filed against drbothen/vsdd-factory
(Traceability to BCs is pending PO authorship)

A GitHub issue is filed against `drbothen/vsdd-factory` documenting: (a) the finding F-WASE-P64-MED-016 and the six affected stories, (b) the ENGINE-001 normative pattern as the target format, (c) a proposed gate specification (script or WASM plugin), and (d) a reference to this story. The upstream issue URL is recorded in §Deliverables.

---

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|----------------|
| validator script or hook plugin | `.factory/hooks/` or `drbothen/vsdd-factory` upstream | Effectful (blocks `status: ready` transitions) |
| story files (read-only) | `.factory/stories/*.md` | Pure (read input) |

---

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Story has `### Red Gate tests` heading but all RGT entries are already checked `- [x]` | Gate still passes — checked entries count toward the density; already-written tests are not a problem |
| EC-002 | Story has a non-standard heading like `### Red Gate Tests` (capital T) | Gate uses case-insensitive heading match; all variants accepted |
| EC-003 | Story has `behavioral_contracts: []` (S-7.01 gate not yet satisfied) | S-7.01 gate blocks `status: ready` independently; this gate is additive and runs after S-7.01 |
| EC-004 | Density < 0.5 but > 0 with explicit justification in the density paragraph | Warn but do not block — human override path via explicit justification text in density paragraph |

---

## Token Budget Estimate

| Item | Estimated tokens |
|------|-----------------|
| This story spec | ~2,000 |
| `S-WAVE-A-ENGINE-001` (normative template, read for AC-001 spec) | ~5,000 |
| Hook plugin or script authoring (if in `.factory/hooks/`) | ~3,000 |
| Upstream issue text authoring | ~1,000 |
| Total | ~11,000 |

Well within a single agent context window. No split required.

---

## Tasks

### Red Gate tests (to be written by test-writer BEFORE implementation)

N/A — this story's deliverables are a validator script/hook and an upstream issue. There is no Rust production code with `todo!()` stubs. The story is `tdd_mode: strict` but its implementation is a shell/WASM script, not a Rust crate. When the implementing agent is dispatched, test-writer MUST write failing tests for the script behavior (e.g., a test harness that feeds known-bad story fixtures and asserts non-zero exit code) before the script logic is implemented.

**Red Gate density check** (BC-5.38.001): **0 pre-written named tests** at story-writing time. Tests will be enumerated in a follow-up story-writer pass when the implementing module (script vs WASM plugin) is confirmed. Density check deferred to implementation-time pre-pass (standard pattern for tooling stories where the test vehicle is not yet determined). This story's `status: draft` reflects the BC status pending PO authorship — it does not transition to `ready` until `behavioral_contracts:` is non-empty per S-7.01.

### Implementation tasks

- [ ] T-01: Confirm implementing module type — shell script in `.factory/hooks/` or upstream WASM plugin in `drbothen/vsdd-factory`. Record decision.
- [ ] T-02: Author the validator (script or plugin) implementing AC-001, AC-002, AC-003 logic.
- [ ] T-03: Wire the validator into the factory-dispatcher hook chain so it runs on `status: ready` transitions.
- [ ] T-04: File upstream issue against `drbothen/vsdd-factory` (AC-004); record URL in §Deliverables.

---

## Previous Story Intelligence

**S-MAINT-POL29-HOOK-001** — established the pattern for factory tooling stories: identify root cause → upstream issue → local gate script. Direct structural precedent for this story.

**S-MAINT-PRMGR-HOOK-SCOPE-001** — another process-gap follow-up story proposing upstream WASM plugin changes. Provides the frontmatter template and task structure.

**F-WASE-P64-MED-016** — the originating finding. Six of seven perimeter stories (S-WAVE-A-CYBERINT-PATCH-001, S-WAVE-A-CYBERINT-SPEC-001, S-WAVE-A-MCP-001, S-ADR054-WAVE-A-001, S-ADR055-WAVE-A-001, S-WAVE-A-ARMIS-REMEDIATION-001) lacked enumerated Red Gate sections. Corrected in FB61 by story-writer. This story prevents recurrence.

---

## Architecture Compliance Rules

1. **No prism crate modifications.** This story must not add, remove, or edit any file under `crates/`. Scope is `.factory/hooks/` + upstream issue.
2. **No STATE.md edits.** STATE.md is state-manager territory.
3. **No STORY-INDEX.md edits.** Registration is a state-manager burst, not this story's deliverable.
4. **TD-VSDD-053 single-commit-per-burst applies.** All `.factory/` changes must go in one atomic commit.

---

## Library & Framework Requirements

No Rust library dependencies. Deliverable is a shell script or WASM plugin. If shell: bash with standard POSIX tools (`grep`, `awk`, `find`). If WASM: follows the `drbothen/vsdd-factory` plugin interface for hook plugins (see existing `.factory/hooks/*.sh` for local examples).

---

## File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `.factory/hooks/validate-red-gate-list.sh` | Create (or equivalent WASM plugin) | AC-001/AC-002/AC-003 gate logic |
| `drbothen/vsdd-factory` GitHub issue | Create | AC-004; URL recorded in §Deliverables |

---

## §Deliverables

| Item | Status | Reference |
|------|--------|-----------|
| Upstream issue URL | Pending | (to be filled at T-04 completion) |

---

## Changelog

| Version | Date | Author | Summary |
|---------|------|--------|---------|
| 0.1 | 2026-07-26 | story-writer | FB61 process-gap follow-up: new story registered per F-WASE-P64-MED-016 — proposes factory validator gate for tdd_mode: strict stories missing enumerated Red Gate list + BC-5.38.001 density check; status: draft; behavioral_contracts: [] pending PO authorship per S-7.01 |
