---
document_type: story
story_id: PLUGIN-MIGRATION-001-H
title: ".factory: Story Supersession — Mark S-2.06, S-2.07, W3-FIX-S307-001, S-3.1.06-ImplPhase Superseded by ADR-023"
wave: 2
epic_id: PLUGIN-MIGRATION-001
priority: P0
status: done
version: "v1.2"
level: "L4"
producer: story-writer
timestamp: "2026-05-27T00:00:00Z"
modified: "2026-05-27"
tdd_mode: strict
subsystems: [SS-01]
# Subsystem anchor justifications:
#   SS-01 (Sensor Adapters) — all 4 stories being superseded describe sensor adapter
#   behavior (S-2.06: DataSource trait and auth patterns; S-2.07: per-sensor auth and
#   pagination; W3-FIX-S307-001: concrete sensor adapter write overrides;
#   S-3.1.06-ImplPhase: adapter OrgId binding). The supersession records that ADR-023
#   plugin-only architecture replaces these sensor-specific Rust implementations.
#   SS-01 is the subsystem whose evolution is being documented.
crates_touched: []
# This is a .factory-only story. Only STORY-INDEX.md and individual story files are modified.
target_module: ".factory/stories"
capabilities: [CAP-029]
behavioral_contracts:
  - BC-2.01.013  # DataSource Trait — the 4 superseded stories described sensor-specific
                 # implementations of the DataSource/SensorAuth pattern; the supersession
                 # records that ADR-023 plugin-only architecture is the authoritative successor.
                 # BC-2.01.013 is active and is the canonical reference for what supersedes
                 # the per-sensor Rust adapter approach.
# BC status: BC-2.01.013 is active per BC-INDEX.md v5.53 (promoted D-398 per POL-14).
# Single BC is sufficient for this administrative story — it is the anchor BC for the
# plugin-only sensor architecture that supersedes the 4 targeted stories.
# No BC-TBD placeholders.
verification_properties: []
# Administrative .factory/ story. No executable verification properties. The gate is
# the consistency-validator confirming all 4 superseded stories have correct status
# and the STORY-INDEX rows are updated.
depends_on:
  - PLUGIN-MIGRATION-001-A  # 001-A is the cutover commit that deleted the sensor Rust
                            # adapters that S-2.06, S-2.07, and W3-FIX-S307-001 were
                            # implementing. The supersession records the causal chain:
                            # 001-A deleted what these stories built; they are now
                            # architecturally superseded.
blocks: []
# 001-G (doc/ADR/BC sweep) does not depend on 001-H. Both are Wave 2 cleanup stories
# with no inter-dependency. They can be dispatched in parallel.
points: 3
# Points justification:
#   - 4 story file status updates (status: draft → status: superseded): ~0.5 pts each = 2 pts
#     (frontmatter change + supersession note added to each story body)
#   - STORY-INDEX.md 4 row annotation updates: ~0.5 pt
#     (add [SUPERSEDED by ADR-023 PLUGIN-MIGRATION-001-A 2026-05-27] annotations)
#   - STORY-INDEX.md overview text update: ~0.25 pt
#     (update total_stories count context, add supersession note to W3-FIX-S307-001 row)
#   - Verify W3-FIX-S307-002 and related stories: already noted as BLOCKED; confirm
#     annotation is consistent with supersession of W3-FIX-S307-001: ~0.25 pt
#   Total: 3 pts. ADR-023 Wave 2 estimate: 2–3 SP.
estimated_days: 1
risk: LOW
# Risk justification: Administrative story. The risk is mis-superseding a story that
# still has active implementation scope. The analysis below confirms each of the 4
# targeted stories is fully superseded. No Rust code changes.
acceptance_criteria_count: 4
red_gate_tests: 0
# Administrative story — no executable tests. Gate is adversary consistency-validator pass.
estimated_passes: "1 LOCAL adversary pass"
holdout_scenarios: []
assumption_validations: []
risk_mitigations:
  - "Scope verification before supersession: read each story file before marking superseded
    to confirm no scope is still active. S-3.1.06-ImplPhase is listed in STORY-INDEX as
    MERGED (PR #117); its status is merged, not draft. The supersession annotation here
    is to record that ADR-023 makes some of its implementation scope architecturally
    superseded — the merged status is preserved."
  - "W3-FIX-S307-001 BLOCKED annotation: STORY-INDEX already has '[BLOCKED — superseded
    by PLUGIN-MIGRATION-001-A per D-333]'. This story formalizes that annotation into a
    status: superseded transition in the story file itself."
inputs:
  - ".factory/stories/STORY-INDEX.md"
  - ".factory/specs/architecture/decisions/ADR-023-plugin-only-sensor-architecture.md"
  - ".factory/specs/behavioral-contracts/BC-2.01.013-datasource-trait-adapter-pattern.md"
input-hash: null
traces_to: []
cycle: "v1.0.0-greenfield"
phase: 3
---

# PLUGIN-MIGRATION-001-H: .factory — Story Supersession — Mark 4 Stories Superseded by ADR-023

**Story ID:** PLUGIN-MIGRATION-001-H
**Status:** in_progress
**Version:** v1.1
**Wave:** 2 (ordered after PLUGIN-MIGRATION-001-A; can be dispatched in parallel with 001-F and 001-G)

---

## §Origin

Registered in STORY-INDEX at D-334 (2026-05-10) as Wave 2 of the PLUGIN-MIGRATION saga.

ADR-023 §Migration Plan Wave 2 scope: "Story supersession — mark S-2.06, S-2.07,
W3-FIX-S307-001, S-3.1.06-ImplPhase superseded in STORY-INDEX (2–3 SP)."

The STORY-INDEX already has BLOCKED annotations on W3-FIX-S307-001 and W3-FIX-S307-002
per D-333 (2026-05-08). This story formalizes those annotations into proper
`status: superseded` transitions with `superseded_by: ADR-023` references.

---

## §Supersession Analysis

Each of the 4 stories is analyzed for the basis of supersession:

### S-2.06 — DataSource Trait and Auth Patterns (merged PR #54)
- **Status before this story:** merged
- **Supersession basis:** S-2.06 implemented the `DataSource` trait and the four
  sensor-specific auth patterns (`CrowdStrikeAuth`, `ClarotyAuth`, `CyberintAuth`,
  `ArmisAuth`) as compiled-in Rust code. ADR-023 retires the per-sensor Rust adapter
  approach entirely. The `DataSource` trait is superseded by the open `SensorAuth` trait
  (BC-2.01.016) + `SensorId(Arc<str>)` keying. The four auth implementations were
  deleted in PLUGIN-MIGRATION-001-A.
- **Supersession action:** Add a supersession note to the story file body. Do NOT change
  `status: merged` — the story was correctly merged; the ADR-023 migration superseded
  the architectural approach it represented, not the work itself. The note records the
  architectural succession.

### S-2.07 — Per-Sensor Auth and Pagination (merged PR #60)
- **Status before this story:** merged (RED_RATIO=83.9%)
- **Supersession basis:** S-2.07 implemented the per-sensor auth flows and pagination
  patterns (CrowdStrike two-step, Cyberint cookie, Claroty bearer, Armis bearer+AQL).
  These sensor-specific pagination patterns are now delivered by TOML spec
  `[fetch_step]` and `[fetch_step.enrich]` configuration (PLUGIN-MIGRATION-001-D).
  The Rust implementations were deleted in PLUGIN-MIGRATION-001-A.
- **Supersession action:** Same as S-2.06: add supersession note to story body.
  Preserve `status: merged`.

### W3-FIX-S307-001 — Concrete Sensor Adapter Write Overrides (never merged)
- **Status before this story:** draft, BLOCKED
- **Supersession basis:** This story was registered to implement `fn write(...)` overrides
  in each of the four built-in sensor adapters. ADR-023 Rule 1 makes this moot:
  there are no built-in sensor adapters in Rust. Write operations for sensors are
  handled by the plugin-only path. D-333 (2026-05-08) recorded this BLOCKED status.
- **Supersession action:** Transition `status: draft` → `status: superseded` in the
  story file AND the STORY-INDEX row. Add `superseded_by: ADR-023` frontmatter.
  The STORY-INDEX row already has the BLOCKED annotation; extend it with the formal
  supersession marker.

### S-3.1.06-ImplPhase — prism-sensors: complete adapter OrgId binding (merged PR #117)
- **Status before this story:** merged
- **Supersession basis:** S-3.1.06-ImplPhase completed the adapter OrgId binding for the
  four built-in sensor adapters. That binding work was then superseded by
  PLUGIN-MIGRATION-001-A which deleted those adapters. The OrgId binding contract
  is now enforced by BC-3.2.001 at the spec-catalog + SensorId layer, not via the
  deleted Rust adapter constructors.
- **Supersession action:** Same as S-2.06 and S-2.07: add supersession note to story
  body. Preserve `status: merged`.

**Note on W3-FIX-S307-002 (WriteExecutor Phase 3):** The STORY-INDEX row for
W3-FIX-S307-002 also has a `[BLOCKED — superseded by PLUGIN-MIGRATION-001-B per D-333]`
annotation. ADR-023 Wave 2/H scope explicitly lists only S-2.06, S-2.07, W3-FIX-S307-001,
and S-3.1.06-ImplPhase. W3-FIX-S307-002 is NOT in scope for this story. Its BLOCKED
annotation is already recorded and no further action is needed here (the WriteExecutor
path will be implemented differently, not via the sensor-named adapter approach).

---

## Story-Level Goal

At merge:

1. Four story files receive `superseded_by: ADR-023` frontmatter and a supersession note
   in their bodies.
2. STORY-INDEX.md is updated: the 4 rows reflect the supersession status.
3. The STORY-INDEX overview note is updated to document the supersession count.

No Rust code changes. No BC changes. No VP changes. This is a pure `.factory/`
administrative commit.

---

## Narrative

As the Prism platform specification, I want the four stories whose sensor-specific Rust
adapter implementations were superseded by the ADR-023 plugin-only architecture to be
formally marked as superseded with `superseded_by: ADR-023`, so that future contributors
do not attempt to implement or extend the deleted adapter patterns and instead use the
plugin-only sensor architecture.

---

## Behavioral Contracts

| BC ID | Version | Title | Subsystem | Role in This Story |
|-------|---------|-------|-----------|-------------------|
| BC-2.01.013 | 1.7 | DataSource Trait Eliminates Per-Sensor Code Duplication | SS-01 | **Anchor** — the plugin-only sensor architecture defined by BC-2.01.013 is the authoritative successor to the per-sensor Rust adapter patterns described by the 4 superseded stories; this BC is cited in each story's supersession note |

---

## Token Budget Estimate

| Artifact | Estimated Tokens |
|----------|-----------------|
| This story spec | ~5,000 |
| BC-2.01.013 (anchor BC for supersession) | ~3,000 |
| ADR-023 §Migration Plan Wave 2 + §Constraints | ~3,000 |
| STORY-INDEX.md (relevant rows + overview section) | ~4,000 |
| 4 story files to update (stub reads — no full story text needed) | ~4,000 |
| **Total estimate** | **~19,000** |
| Agent context window (claude-sonnet-4-6) | ~200,000 |
| **% of context window** | **~9.5%** |

Well within budget. This is the smallest story in Wave 2.

---

## Acceptance Criteria

### AC-001: S-2.06 story file — supersession note added; merged status preserved (traces to BC-2.01.013 invariant — architectural succession of the DataSource trait pattern is recorded in the spec; no per-sensor Rust adapter implementation is valid post-ADR-023)

The story file for S-2.06 (`S-2.06-*.md` or equivalent filename) receives:

**Frontmatter addition:**
```yaml
superseded_by: ADR-023
supersession_note: >
  S-2.06 implemented the DataSource trait and four sensor-specific auth patterns
  (CrowdStrikeAuth, ClarotyAuth, CyberintAuth, ArmisAuth) as compiled-in Rust code.
  The architectural approach was superseded by ADR-023 (Plugin-Only Sensor Architecture).
  The sensor-specific Rust implementations were deleted in PLUGIN-MIGRATION-001-A (PR #156).
  The DataSource trait pattern is now implemented via the open SensorAuth trait (BC-2.01.016)
  and TOML sensor specs in .prism/specs/sensors/. status: merged is preserved — the work was
  correctly completed; the architectural approach it represented was subsequently superseded.
```

**Body addition** (appended as a new section at the end of the story file):
```markdown
## §Supersession Note (ADR-023)

**Superseded by:** ADR-023 Plugin-Only Sensor Architecture
**Supersession date:** 2026-05-27 (PLUGIN-MIGRATION-001-H)
**Reason:** S-2.06 implemented per-sensor Rust auth adapters. ADR-023 mandates that
all sensor behavior is delivered by TOML specs and (where required) .prx WASM plugins.
The four auth adapter Rust types built in this story were deleted in PLUGIN-MIGRATION-001-A.
**Successor:** BC-2.01.013 (DataSource Trait) + BC-2.01.016 (SensorAuth Open Trait)
+ TOML sensor specs (.prism/specs/sensors/*.sensor.toml).
```

`status: merged` is preserved (not changed to superseded). The story was correctly
merged; the supersession documents architectural succession, not implementation failure.

(traces to BC-2.01.013 invariant — the spec catalog is the source of truth for the
sensor architecture; the supersession note ensures no contributor attempts to restore
the deleted per-sensor Rust adapter pattern)

### AC-002: S-2.07 story file — supersession note added; merged status preserved (traces to BC-2.01.013 invariant — per-sensor pagination Rust implementations are superseded by TOML spec [fetch_step] configuration)

Same pattern as AC-001 for S-2.07:

**Supersession note content (S-2.07 specific):**
```markdown
## §Supersession Note (ADR-023)

**Superseded by:** ADR-023 Plugin-Only Sensor Architecture
**Supersession date:** 2026-05-27 (PLUGIN-MIGRATION-001-H)
**Reason:** S-2.07 implemented per-sensor auth flows (CrowdStrike OAuth2 two-step,
Cyberint cookie, Claroty bearer, Armis bearer+AQL) and pagination patterns as compiled-in
Rust code. ADR-023 delivers these via TOML spec [fetch_step], [fetch_step.enrich], and
[auth] configuration blocks. The Rust implementations were deleted in PLUGIN-MIGRATION-001-A.
**Successor:** PLUGIN-MIGRATION-001-D TOML sensor specs + BC-2.01.005/006/007/008
(as amended by PLUGIN-MIGRATION-001-G).
```

(traces to BC-2.01.013 invariant — spec-driven sensor behavior supersedes per-sensor Rust compilation)

### AC-003: W3-FIX-S307-001 story file — status transitioned to superseded; STORY-INDEX row updated (traces to BC-2.01.013 postcondition — sensor write operations are spec-driven, not implemented as per-sensor Rust adapter overrides)

W3-FIX-S307-001 was never merged (status: draft, BLOCKED per D-333):

**Frontmatter change:**
```yaml
# Before:
status: draft

# After:
status: superseded
superseded_by: ADR-023
supersession_note: >
  W3-FIX-S307-001 was planned to implement `fn write(...)` overrides in the four
  built-in sensor adapters. ADR-023 makes this moot: there are no built-in sensor
  adapters. Write operations are handled by the plugin-only path per ADR-023 Rule 1.
  Recorded as BLOCKED per D-333 (2026-05-08). Formally superseded per PLUGIN-MIGRATION-001-H.
```

**STORY-INDEX row update:**
```
# Before:
| W3-FIX-S307-001 | ... [BLOCKED — superseded by PLUGIN-MIGRATION-001-A per D-333] |

# After:
| W3-FIX-S307-001 | ... [BLOCKED — superseded by PLUGIN-MIGRATION-001-A per D-333] [SUPERSEDED status:superseded per PLUGIN-MIGRATION-001-H 2026-05-27] |
```

The body of the story file receives the §Supersession Note section per AC-001 pattern.

(traces to BC-2.01.013 postcondition — the DataSource trait pattern makes per-sensor write overrides unnecessary; plugin-only architecture handles write operations through the spec-engine dispatch layer)

### AC-004: S-3.1.06-ImplPhase story file — supersession note added; merged status preserved; STORY-INDEX row annotated (traces to BC-2.01.013 invariant — adapter OrgId binding work superseded by spec-catalog SensorId keying at the spec-engine layer)

S-3.1.06-ImplPhase was merged (PR #117, cda17ed4, 2026-05-02):

**Supersession note content:**
```markdown
## §Supersession Note (ADR-023)

**Superseded by:** ADR-023 Plugin-Only Sensor Architecture
**Supersession date:** 2026-05-27 (PLUGIN-MIGRATION-001-H)
**Reason:** S-3.1.06-ImplPhase completed OrgId binding for the four built-in sensor
adapter constructors. Those adapter constructors (CrowdStrikeAdapter, ClarotyAdapter,
CyberintAdapter, ArmisAdapter) were subsequently deleted in PLUGIN-MIGRATION-001-A (PR #156).
The OrgId binding contract is now enforced at the spec-catalog + SensorId(Arc<str>) layer
per BC-3.2.001 (Per-Org Sensor Data Isolation). The merged status is preserved — the
OrgId binding work was correctly implemented; the underlying adapters were later deleted.
**Successor:** BC-3.2.001 + SensorId(Arc<str>) keying per ADR-023 C1.
```

STORY-INDEX row addition:
```
[SUPERSEDED architectural approach per PLUGIN-MIGRATION-001-H 2026-05-27 (merged status preserved)]
```

`status: merged` is preserved. The supersession documents that the approach used
(binding OrgId to the four specific Rust adapter constructors) is no longer valid,
even though the work was correctly completed and merged at the time.

(traces to BC-2.01.013 invariant — the plugin-only sensor architecture defines SensorId(Arc<str>)
as the canonical sensor identifier; no adapter-constructor OrgId binding exists)

---

## Tasks

- [ ] **Task 1:** Locate the story files for S-2.06, S-2.07, W3-FIX-S307-001, and S-3.1.06-ImplPhase
      in `.factory/stories/` (verify exact filenames via `ls .factory/stories/ | grep -E 'S-2.06|S-2.07|W3-FIX-S307-001|S-3.1.06-ImplPhase'`)
- [ ] **Task 2:** Add `superseded_by: ADR-023` and `supersession_note:` to S-2.06 frontmatter;
      append §Supersession Note section to S-2.06 body; bump `version:` +0.1; update `modified:`
- [ ] **Task 3:** Same for S-2.07
- [ ] **Task 4:** Update W3-FIX-S307-001: change `status: draft` → `status: superseded`;
      add `superseded_by: ADR-023`; append §Supersession Note section; bump version; update modified
- [ ] **Task 5:** Add `superseded_by: ADR-023` and `supersession_note:` to S-3.1.06-ImplPhase
      frontmatter; preserve `status: merged`; append §Supersession Note section; bump version
- [ ] **Task 6:** Update STORY-INDEX.md: add supersession annotations to the 4 story rows
      per AC-003 and AC-004 patterns; bump STORY-INDEX version +0.001
- [ ] **Task 7:** Verify W3-FIX-S307-002 row in STORY-INDEX is correctly annotated as BLOCKED
      (not in scope for status change per §Supersession Analysis); confirm no action needed

---

## Architecture Compliance Rules

1. **`status: merged` is preserved for S-2.06, S-2.07, S-3.1.06-ImplPhase.** The supersession
   documents architectural succession, not implementation failure. These stories were correctly
   merged. The `status:` field reflects the implementation lifecycle of the story, not
   whether the approach was later superseded architecturally.

2. **`status: superseded` is applied only to W3-FIX-S307-001.** This story was never merged
   (draft/BLOCKED) and its scope was fully superseded before implementation. It is the
   only one of the 4 that receives a `status: superseded` transition.

3. **Spec-First Gate S-7.01:** `behavioral_contracts:` is non-empty (`[BC-2.01.013]`) and
   `status: draft` (not `ready`) in this story's frontmatter. The AC traces above satisfy
   the bidirectional trace requirement: BC-2.01.013 is cited in all 4 ACs, and AC-001
   through AC-004 are cited in the BC body section above. This story may transition to
   `ready` once a product-owner reviews and confirms the supersession analysis is correct.

4. **W3-FIX-S307-002 is OUT OF SCOPE.** Only the 4 stories named in ADR-023 §Migration Plan
   Wave 2/H are in scope. W3-FIX-S307-002's BLOCKED annotation already exists; no
   `status: superseded` transition is performed for it in this story.

---

## Library & Framework Requirements

| Library | Version | Usage |
|---------|---------|-------|
| N/A — doc-only story | — | No Rust code changes |

---

## File Structure Requirements

| Action | File Path | Notes |
|--------|-----------|-------|
| MODIFY | `.factory/stories/S-2.06-*.md` (exact filename TBD via ls) | Add supersession frontmatter + body note; preserve status: merged |
| MODIFY | `.factory/stories/S-2.07-*.md` (exact filename TBD via ls) | Same |
| MODIFY | `.factory/stories/W3-FIX-S307-001-*.md` (exact filename TBD via ls) | Change status: draft → superseded; add supersession frontmatter + body note |
| MODIFY | `.factory/stories/S-3.1.06-ImplPhase-*.md` (or equivalent) | Add supersession frontmatter + body note; preserve status: merged |
| MODIFY | `.factory/stories/STORY-INDEX.md` | 4 row annotations + version bump |

Note: The exact filenames are determined by reading the actual `.factory/stories/` directory
at implementation time. The implementer MUST `ls .factory/stories/` before modifying any file.

---

## Previous Story Intelligence

This is the third Wave 2 cleanup story. Previous context:

1. **D-333 (2026-05-08):** Orchestrator recorded W3-FIX-S307-001 and W3-FIX-S307-002 as
   BLOCKED due to PLUGIN-MIGRATION-001-A supersession. The STORY-INDEX already has these
   BLOCKED annotations. This story formalizes the supersession for 3 of the 4 targeted stories
   (S-2.06, S-2.07, and S-3.1.06-ImplPhase are merged stories with a supersession note
   approach; W3-FIX-S307-001 gets a status transition).

2. **PLUGIN-MIGRATION-001-A (merged PR #156):** The cutover commit that deleted the
   sensor-specific Rust adapters. This is the causal trigger for the supersession of all
   4 stories.

3. **ADR-023 §Migration Plan Wave 2/H:** "Story supersession — mark S-2.06, S-2.07,
   W3-FIX-S307-001, S-3.1.06-ImplPhase superseded in STORY-INDEX (2–3 SP)." — this
   is the authoritative scope definition for this story. Nothing outside this list is
   in scope.

Key lesson: **do not scope-creep into W3-FIX-S307-002** or other BLOCKED stories not
listed in ADR-023 §Migration Plan Wave 2/H. Supersessing the wrong story creates
consistency-validator failures.

---

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | S-2.06 or S-2.07 story file does not exist in .factory/stories/ | This should not happen (both are merged stories); if a file is missing, report to orchestrator — do not create a stub |
| EC-002 | W3-FIX-S307-001 story file is missing | Same as EC-001; report and do not create a stub |
| EC-003 | S-3.1.06-ImplPhase filename does not match expected pattern | Use `ls .factory/stories/ | grep ImplPhase` to find the correct filename before modifying |
| EC-004 | STORY-INDEX row for a target story is missing | This should not happen; if missing, report to orchestrator — do not silently skip |
| EC-005 | W3-FIX-S307-002 is accidentally included in the supersession | Out of scope per ADR-023 §Migration Plan Wave 2/H. Remove the W3-FIX-S307-002 change and confirm it is not included in the commit. |

---

## Forbidden Dependencies

This story produces only `.factory/` artifacts. No `crates_touched`. No `Cargo.toml`
changes. No BC changes (beyond cite-pin consistency if needed). No VP changes.

The implementer must NOT touch:
- Any Rust source file (out of scope)
- W3-FIX-S307-002 (out of scope per §Supersession Analysis)
- BC files (that is 001-G scope, not 001-H scope)
- VP-INDEX.md (no VP changes needed)

---

## Changelog

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| v1.1 | 2026-05-27 | story-writer | IMP-001 fix: corrected W3-FIX-S307-001 pre-change status from `planned` to `draft` (confirmed via git history of v1.0 frontmatter) |
| v1.0 | 2026-05-27 | story-writer | Initial draft — 4 ACs + 7 tasks; PLUGIN-MIGRATION-001-H Wave 2 materialization |
