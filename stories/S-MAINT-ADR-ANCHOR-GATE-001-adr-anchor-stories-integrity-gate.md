---
document_type: story
story_id: "S-MAINT-ADR-ANCHOR-GATE-001"
title: "Factory Validator Gate — Enforce ADR anchor_stories Key Presence and Population Integrity"
wave: tbd
epic_id: maintenance
priority: P2
status: draft
version: "0.1"
level: ops
producer: story-writer
timestamp: "2026-07-27"
modified: "2026-07-27"
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
acceptance_criteria_count: 5
red_gate_tests: 0
estimated_passes: "tbd"
holdout_scenarios: []
assumption_validations: []
risk_mitigations: []
tags:
  - process-gap
  - factory-tooling
  - spec-governance
---

# S-MAINT-ADR-ANCHOR-GATE-001: Factory Validator Gate — Enforce ADR anchor_stories Key Presence and Population Integrity

## Origin

**Process-gap finding:** F-WASE-P64-OBS-001 (LOCAL pass 64, Wave-A spec-evolution cascade, 2026-07-26)

ADR-053 and ADR-054 carried no `anchor_stories` key at all; ADR-050, ADR-051, ADR-052, and ADR-055 carried the key but three of the four populated it with `[]`. No validator enforced either key presence or population. As a result, ADR→story traceability in the Wave-A perimeter was one-directional: stories cited ADRs, but ADRs cited no stories.

The content half was fixed in FB62 (architect leg): every ADR-050..056 now carries `anchor_stories`, populated from ground-truth `§Authority` citations in story files. ADR-051 v1.5→v1.6, ADR-052 v1.15→v1.16, ADR-053 v0.36→v0.37, ADR-054 v0.55→v0.56.

This maintenance story proposes a structural gate so the gap cannot recur: a factory validator checks ADR files for `anchor_stories` key presence and cross-reference population integrity before those ADRs can be treated as governance-ready by downstream factory tooling.

---

## Narrative

As an architect committing a new or updated ADR to `.factory/specs/architecture/adr/`,
I want the factory dispatch system to detect ADRs missing the `anchor_stories` key entirely, and to detect ADRs whose `anchor_stories: []` contradicts story cross-references that demonstrably cite them,
so that ADR→story traceability remains bidirectional and stale-empty `anchor_stories` arrays cannot persist undetected after anchor stories exist.

---

## Acceptance Criteria

### AC-001 — Validator gate detects ADRs missing the anchor_stories key entirely
(Traceability to BCs is pending PO authorship)

A factory validator (script or factory-dispatcher hook plugin) MUST emit a hard-block error for any ADR file in `.factory/specs/architecture/adr/` whose frontmatter does NOT contain an `anchor_stories:` key at all.

The error MUST identify the ADR file by ID and filename, state that the key is absent, and include a pointer to the canonical ADR frontmatter schema.

This is the ADR-053/ADR-054 failure mode from F-WASE-P64-OBS-001: an entirely missing key passes any content check that assumes the key exists.

### AC-002 — Validator gate detects stale anchor_stories: [] when story cross-references exist
(Traceability to BCs is pending PO authorship)

The same gate MUST emit a hard-block error for any ADR file where `anchor_stories: []` (empty array) while at least one story file in `.factory/stories/` references that ADR's canonical ID in its `§Authority` section heading, its `traces_to:` frontmatter field, or within a `behavioral_contracts` body table.

The error MUST name the ADR ID, the contradicting story ID(s) found, and the type of citation that was detected (§Authority / traces_to / body table).

This is the harder and more valuable half: a bare key-presence check passes ADR-051 and ADR-052 as they stood before FB62, because both carried `anchor_stories: []` without error. The staleness check catches the contradiction between the empty claim and the existing story evidence.

### AC-003 — Gate accepts anchor_stories: [] accompanied by a verified-empty annotation
(Traceability to BCs is pending PO authorship)

An ADR with `anchor_stories: []` that also carries an inline comment immediately adjacent to the key — matching the pattern `# verified-empty:` followed by non-empty explanatory text — MUST pass the gate without any finding, even if no story cross-references exist.

This is the legitimate-empty path: it distinguishes "no anchor story has been authored yet, and this absence has been reviewed and acknowledged" from "the field was never filled in." The architect convention established in FB62 uses `# verified-empty: no anchor story as of vX.Y` as the annotation form. The gate must treat any `# verified-empty:` comment with non-empty trailing text as sufficient.

An ADR with `status: superseded` in frontmatter MUST be skipped entirely by the gate — superseded ADRs have no meaningful anchor_stories obligation.

### AC-004 — ADRs with anchor_stories: [] and no annotation and no cross-references emit a warning, not a hard block
(Traceability to BCs is pending PO authorship)

An ADR with `anchor_stories: []` and NO verified-empty annotation, but also NO story cross-references detected in `.factory/stories/`, MUST produce a non-blocking warning rather than a hard-block error. The warning MUST identify the ADR and suggest either populating the array or adding a `# verified-empty:` annotation.

Rationale: a newly authored ADR may legitimately predate its anchor story; requiring an annotation at ADR authorship time before any story exists is unnecessarily burdensome. The hard block in AC-002 applies only when story evidence already contradicts the empty claim. An unreviewed empty with no contrary evidence is a soft gap, not a confirmed staleness defect.

### AC-005 — Upstream issue filed against drbothen/vsdd-factory
(Traceability to BCs is pending PO authorship)

A GitHub issue is filed against `drbothen/vsdd-factory` documenting: (a) finding F-WASE-P64-OBS-001 and its two failure modes (missing key on ADR-053/ADR-054; stale empty on ADR-051/ADR-052), (b) the two-tier gate specification (hard block for missing key; hard block for stale empty with story evidence; verified-empty annotation as the legitimate-empty path; warning for unreviewed empty with no contrary evidence), (c) a pointer to `S-MAINT-RG-LIST-GATE-001` as a sibling gate story for structural reference, and (d) a reference to this story. The upstream issue URL is recorded in §Deliverables.

---

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|----------------|
| validator script or hook plugin | `.factory/hooks/` or `drbothen/vsdd-factory` upstream | Effectful (scans ADR frontmatter + reads story files for cross-references) |
| ADR files (read-only) | `.factory/specs/architecture/adr/` | Pure (read input) |
| story files (read-only for cross-reference scan) | `.factory/stories/` | Pure (read input) |

---

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | ADR has `status: superseded` in frontmatter | Gate skips the ADR entirely — superseded ADRs have no meaningful anchor_stories obligation |
| EC-002 | Multiple stories cite the same ADR | Any one citation is sufficient — gate passes on first match; error message lists all detected citations for context |
| EC-003 | `anchor_stories:` key is present but its value is neither a list nor null (malformed YAML) | Gate treats this as a MISSING_KEY hard block — same error class as AC-001 |
| EC-004 | Story file references the ADR ID only in a changelog narrative row, not in §Authority / traces_to / body table | Gate does NOT count changelog-row citations as cross-references — changelog rows are historical records, not active traceability anchors |
| EC-005 | A story has `behavioral_contracts: []` but mentions an ADR ID in its `§Authority` section heading | Gate counts the §Authority heading citation as cross-reference evidence for the AC-002 staleness check |

---

## Token Budget Estimate

| Item | Estimated tokens |
|------|-----------------|
| This story spec | ~2,000 |
| `S-MAINT-RG-LIST-GATE-001` (sibling gate story, structural reference for implementer) | ~2,000 |
| ADR frontmatter schema reference (sampled from existing ADR files) | ~2,000 |
| Hook script or plugin authoring (if in `.factory/hooks/`) | ~4,000 |
| Upstream issue text authoring | ~1,000 |
| Total | ~11,000 |

Well within a single agent context window. No split required.

---

## Tasks

### Red Gate tests (to be written by test-writer BEFORE implementation)

N/A — this story's deliverables are a validator script/hook and an upstream issue. There is no Rust production code with `todo!()` stubs. The story is `tdd_mode: strict` but its implementation is a shell/WASM script, not a Rust crate. When the implementing agent is dispatched, test-writer MUST write failing tests for the script behavior (e.g., a test harness feeding known-bad ADR fixtures asserting non-zero exit code for AC-001 and AC-002; known-good fixtures asserting zero exit code for AC-003 and AC-004) before the script logic is implemented.

**Red Gate density check** (BC-5.38.001): **0 pre-written named tests** at story-writing time. Tests will be enumerated in a follow-up story-writer pass when the implementing module (script vs WASM plugin) is confirmed. Density check deferred to implementation-time pre-pass (standard pattern for tooling stories where the test vehicle is not yet determined). This story's `status: draft` reflects the BC status pending PO authorship — it does not transition to `ready` until `behavioral_contracts:` is non-empty per S-7.01.

### Implementation tasks

- [ ] T-01: Confirm implementing module type — shell script in `.factory/hooks/` or upstream WASM plugin in `drbothen/vsdd-factory`. Record decision.
- [ ] T-02: Author the key-presence check (AC-001) — scan each ADR file's frontmatter for the `anchor_stories:` key; hard block on absence.
- [ ] T-03: Author the staleness/population check (AC-002) — for each ADR whose `anchor_stories: []`, scan `.factory/stories/` for §Authority, `traces_to:`, and body-table citations of that ADR ID; hard block on contradiction.
- [ ] T-04: Author the verified-empty annotation pass path (AC-003) — detect `# verified-empty:` inline comment; skip superseded ADRs (EC-001).
- [ ] T-05: Author the no-evidence warning path (AC-004) — warn on unreviewed empty with no cross-references.
- [ ] T-06: Wire the validator into the factory-dispatcher hook chain so it runs on ADR file commits.
- [ ] T-07: File upstream issue against `drbothen/vsdd-factory` (AC-005); record URL in §Deliverables.

---

## Previous Story Intelligence

**S-MAINT-RG-LIST-GATE-001** — direct structural precedent (FB61 sibling process-gap follow-up story, same cascade, same wave). Established the pattern for this class of factory validator gate: identify root cause → specify gate tiers → upstream issue. An adversary pass will compare the two sibling gate stories for consistency; match their frontmatter field set and section ordering.

**S-MAINT-POL29-HOOK-001** — established the general factory tooling pattern: root cause → upstream issue → local gate script. Older precedent for the task list shape.

**S-MAINT-PRMGR-HOOK-SCOPE-001** — another process-gap follow-up story proposing upstream WASM plugin changes. Additional frontmatter template reference.

**F-WASE-P64-OBS-001** — the originating finding. ADR-053/ADR-054 had no `anchor_stories` key; ADR-051/ADR-052 had stale `[]`. Corrected in FB62 by architect. This story prevents recurrence.

---

## Architecture Compliance Rules

1. **No prism crate modifications.** This story must not add, remove, or edit any file under `crates/`. Scope is `.factory/hooks/` + upstream issue.
2. **No STATE.md edits.** STATE.md is state-manager territory.
3. **No STORY-INDEX.md edits.** Registration is a state-manager burst, not this story's deliverable.
4. **TD-VSDD-053 single-commit-per-burst applies.** All `.factory/` changes must go in one atomic commit.
5. **No CLAUDE.md edits.** Codifying the `anchor_stories` convention in CLAUDE.md is a deferred human gate — the validator gate itself is the enforcement mechanism.

---

## Library & Framework Requirements

No Rust library dependencies. Deliverable is a shell script or WASM plugin. If shell: bash with standard POSIX tools (`grep`, `awk`, `find`). YAML frontmatter key extraction uses `awk` pattern matching on the raw frontmatter block; `yq` is acceptable if already present in the factory toolchain. If WASM: follows the `drbothen/vsdd-factory` plugin interface for hook plugins (see existing `.factory/hooks/*.sh` for local shell examples).

---

## File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `.factory/hooks/validate-adr-anchor-stories.sh` | Create (or equivalent WASM plugin) | AC-001/AC-002/AC-003/AC-004 gate logic |
| `drbothen/vsdd-factory` GitHub issue | Create | AC-005; URL recorded in §Deliverables |

---

## §Deliverables

| Item | Status | Reference |
|------|--------|-----------|
| Upstream issue URL | Pending | (to be filled at T-07 completion) |

---

## Changelog

| Version | Date | Author | Summary |
|---------|------|--------|---------|
| 0.1 | 2026-07-27 | story-writer | FB62 process-gap follow-up: new story registered per F-WASE-P64-OBS-001 — proposes factory validator gate for ADR anchor_stories key presence and stale-empty detection; two failure modes: missing key hard-block (AC-001), stale [] with story evidence hard-block (AC-002); verified-empty annotation as legitimate-empty path (AC-003); unreviewed empty with no evidence as warning (AC-004); upstream issue (AC-005); status: draft; behavioral_contracts: [] pending PO authorship per S-7.01 |
