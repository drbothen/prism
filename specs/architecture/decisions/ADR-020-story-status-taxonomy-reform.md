---
document_type: adr
adr_id: "ADR-020"
title: "Story Status Taxonomy Reform — Closed Enum, Partial-Merge Semantics, and Graduation Contract"
status: ACCEPTED
date: "2026-05-08"
version: "1.1"
producer: architect
subsystems_affected: []
supersedes: null
superseded_by: null
inputs:
  - .factory/cycles/wave-4-operations/workspace-audit-2026-05-08.md
  - .factory/proposals/vsdd-stub-merge-policy-2026-05-08.md
  - .factory/proposals/vsdd-prevention-layers-2026-05-08.md
  - .factory/policies.yaml
  - .factory/stories/STORY-INDEX.md
anchor_stories: []
references_phase3_siblings: []
locked_decisions: []
runtime_deliverables: []  # Methodology/process decision — defines story status enum and graduation contract; no production code units
wiring_deferred_to: null  # No runtime wiring required; enforcement via consistency-validator and pre-burst hooks (POL-13, POL-16)
---

# ADR-020: Story Status Taxonomy Reform — Closed Enum, Partial-Merge Semantics, and Graduation Contract

## Status

ACCEPTED 2026-05-08, v1.0. Effective immediately; reconciliation of existing stories is
tracked in the Reconciliation Worklist section and executed in Bundle A.2.

## Context

### Problem: STORY-INDEX is an unreliable status oracle

The 2026-05-08 workspace audit (findings F-AUD-D2-01, F-AUD-D2-11, F-AUD-D3-01) surfaced
three overlapping failures in the story-status taxonomy:

**F-AUD-D2-01 — STORY-INDEX vs. frontmatter drift.** Four or more stories are indexed as
MERGED in STORY-INDEX (via free-text annotation in the title column) while their story-file
frontmatter `status:` field still reads `ready` or `draft`. The STORY-INDEX has evolved into a
manual edit log; the canonical status field in story-file frontmatter is never updated when
STORY-INDEX is updated. Two systems of truth exist with no reconciliation hook.

**F-AUD-D2-11 — One-off `delivered` enum value.** S-1.10 carries `status: delivered` — a
value that does not appear in any defined enum, is not recognized by consistency-validator,
and silently skips downstream checks. This is an undocumented drift value.

**F-AUD-D3-01 — Inverted-polarity test codifying the stub.** At least one test in
`prism-spec-engine/tests/hot_reload_tests.rs` (`test_BC_2_16_007_hot_reload_watcher_start_is_stub`)
carries `#[should_panic(expected = "not yet implemented")]`. This test is green precisely because
the production function is broken. It would break the day HotReloadWatcher is actually implemented,
creating a regression-detection hazard. The story that owns this test (S-1.12) is indexed as MERGED.

### Root cause: no enum definition and no stub-merge distinction

The VSDD methodology never formally defined a closed set of valid `status:` values for story
frontmatter. The practical enum in use (`draft`, `ready`, `in-progress`, `merged`) has no
enforcement, allows free-text drift (`delivered`), and — critically — treats stub-phase
scaffolding (production code with `todo!()`/`unimplemented!()` panics) identically to
fully-delivered implementation. This allows stories to claim `status: merged` while their BCs
remain unreachable from production code paths.

The companion proposal (`vsdd-stub-merge-policy-2026-05-08.md`) identifies this as a
methodology gap: no graduation contract exists requiring downstream completion of stub-phase
code, and no signal distinguishes "fully-delivered merged" from "scaffolding-only merged."

## Decision

### 1. Closed status enum

The valid set of `status:` values for story-file frontmatter is:

```
draft          — spec under development; implementation has not begun
ready          — spec frozen; implementation may begin
in-progress    — implementation underway; PR not yet merged
partial-merge  — PR merged; production stubs remain under a graduation contract
merged         — PR merged; ZERO production-path stubs; all BCs reachable
retired        — story superseded or permanently descoped; implementation will not be completed
```

`delivered` is NOT a valid value. The one story carrying it (S-1.10) must be remapped to
`merged` if it is stub-free, or `partial-merge` if stub residue exists, as part of
Bundle A.2 reconciliation.

No other values are permitted. Consistency-validator and pre-burst hooks reject unrecognized
values (see POL-13 and POL-16).

### 2. `partial-merge` semantics

A story MUST use `status: partial-merge` (not `status: merged`) when either of the following
holds after the PR merges:

- Any `todo!()`, `unimplemented!()`, or `panic!`-with-stub-literal exists in a production
  code path reachable from a `pub fn` in a non-test module.
- An inverted-polarity test (a test whose `#[should_panic]` annotation expects a
  stub-indicating message) remains in the test suite and would break if the stub were implemented.

`status: partial-merge` communicates: "This story's code has landed; the scaffolding is real;
but implementation is not complete. Stubs are tracked and must graduate before this story
can claim `status: merged`."

Production stub residue is permitted ONLY for stories with `status: partial-merge`. Each stub
site must be annotated with the deferred TD ID (e.g., `// TODO(TD-S302-001): see S-3.02 graduation contract`).

`status: merged` is a hard guarantee: zero production stub residue in paths reachable from any `pub fn`.

### 3. Graduation contract

Before a story may transition from `status: partial-merge` to `status: merged`, the story
owner must produce a graduation note in the story file. The graduation note consists of:

**Frontmatter fields:**

```yaml
graduated_from_partial_merge: "2026-05-15"   # ISO date of the transition
closed_tds: [TD-S302-001, TD-S302-002]       # TDs closed to enable graduation
graduation_pr: "#142"                         # PR that completed the stubs
```

**Body section `## Graduation Note`** (appended to the story file):

A human-readable checklist enumerating:
- Each previously-deferred TD that has been closed, with the PR link that closed it
- Each stub site (file:line) that was filled in
- Confirmation that no inverted-polarity tests remain

The graduation note is the story owner's attestation that the BC postconditions previously
blocked by stub residue are now reachable from production code.

State-manager performs the status flip (`partial-merge → merged`) atomically with the
graduation note commit. No status flip is permitted without the graduation note fields
present in frontmatter.

### 4. STORY-INDEX as a derivative document

STORY-INDEX status column is derived from story-file frontmatter, not edited independently.
The canonical status for any story is the `status:` field in the story's frontmatter file.

STORY-INDEX title-column annotations (PR numbers, commit SHAs, dates, test counts) are factual
log entries that may be appended independently. They are metadata about the merge event, not
authoritative status signals.

When a story's `status:` frontmatter changes (for any reason — initial flip, partial-merge
reclassification, graduation, retirement), state-manager MUST update the STORY-INDEX status
column for that story in the same burst. The two must always agree.

### 5. Reconciliation hook

A pre-burst hook (`hooks/check-story-index-consistency.sh`, specification in
`hook-specs-bundle-a.md`) walks `.factory/stories/*.md`, reads the `status:` field from
each story's YAML frontmatter, and verifies that the corresponding STORY-INDEX row reflects
the same status. A mismatch causes the hook to exit non-zero, blocking the burst until
state-manager reconciles the discrepancy.

This closes the two-systems-of-truth failure mode identified in F-AUD-D2-01.

## Consequences

### What changes immediately

- The `delivered` status value is deprecated. S-1.10 must be reclassified in Bundle A.2.
- `partial-merge` is a legitimate status value. Consistency-validator, adversary prompt,
  and pre-burst hooks must recognize it.
- Stories currently indexed as MERGED in STORY-INDEX but carrying `status: ready` or
  `status: draft` in frontmatter must be reconciled (see worklist below).
- Inverted-polarity tests must be tracked as a graduation-blocking condition for any
  story claiming `partial-merge` or aiming for `merged`.

### What changes for future stories

- Story-writers assign `partial-merge` at PR time whenever stub residue exists.
- Implementers fill stubs and produce a graduation note before requesting a `merged` flip.
- Adversary checks stub residue on every pass for `status: merged` stories (POL-12).
- State-manager enforces BC promotion (POL-14) only when status flips to `merged`,
  NOT to `partial-merge`.

### Risk: retroactive reclassification burden

Remapping 8+ stories from `merged` → `partial-merge` requires each story owner to produce
a graduation contract. This is intentionally deferred to Bundle A.2 (story-file mutations
are out of scope for this bundle). This ADR defines the policy; the remediation is a
separate delivery tracked in the reconciliation worklist.

## Alternatives Considered

### A: Deprecate `merged` and introduce a new terminal state

Rejected. Existing STORY-INDEX tooling, VP promotion triggers, and BC promotion logic all
key on `merged`. Replacing it with a new terminal state would break all downstream automations.
Adding `partial-merge` as a pre-terminal holding state preserves the existing meaning of
`merged` (now strengthened with the stub-free guarantee) without rewriting downstream logic.

### B: Boolean per-AC completion tracking

Rejected. Per-AC booleans (e.g., `ac_1_complete: true`) add schema complexity without providing
the graduation narrative that auditors and reviewers need. The graduation contract (a story-file
section with a PR link) is human-readable and auditable; boolean flags are not.

### C: STORY-INDEX as the source of truth

Rejected. STORY-INDEX is a summary artifact; it cannot be the source of truth because it has
no per-story detail (graduation notes, TD lists, PR evidence). Story files are the only
artifact that can carry this evidence. Making STORY-INDEX derivative (not canonical) is the
correct information architecture.

## Reconciliation Worklist

Stories requiring status reconciliation in Bundle A.2. This worklist is derived from the
audit's recommended-reopens table (audit §Recommendations, "Stories That Should Be Re-Opened").
No story-file changes are made in this bundle (Bundle A is schema + policy only).

| Story | Current Status (file) | STORY-INDEX Status | Recommended New Status | Rationale |
|-------|----------------------|-------------------|----------------------|-----------|
| S-1.10 | `delivered` | MERGED | `merged` (if stub-free) or `partial-merge` | `delivered` is not a valid enum value; must be remapped; stub audit required |
| S-1.11 | `merged` | MERGED | `partial-merge` | TD-PLUGIN-P0-001 (PipelineExecutor stub) — production wiring unresolved |
| S-1.12 | `merged` | MERGED | `partial-merge` | F-AUD-D1-07 (HotReloadWatcher start/stop are `unimplemented!()`); inverted-polarity test F-AUD-D3-01 codifies the stub |
| S-1.14 | `merged` | MERGED | `partial-merge` | F-AUD-D1-08/09/10/11 — InfusionLoader, InfusionLruCache, MmdbSource, CsvSource, JsonLookupSource, load_source, plugin_bridge: 100% `unimplemented!()` |
| S-1.15 | `merged` | MERGED | `partial-merge` | TD-PLUGIN-P0-008 (action-plugin dispatch stubbed; `fire_alert`/`fire_case`/`fire_report` deferred); no production wiring |
| S-3.02 | `ready` | MERGED | `partial-merge` | F-AUD-D1-01/02/03 — QueryEngine::execute, run_materialization_pipeline, RocksDbTableProvider: all `todo!()`; index claims MERGED PR #129 but file frontmatter says `ready` |
| S-3.06 | `ready` | MERGED | `merged` | Audit indicates parser is real; index annotation vs. file frontmatter drift only; verify no stub residue before confirming |
| S-3.07 | `draft` | MERGED | `partial-merge` | F-AUD-D1-04/05/06 — write pipeline Phase 3 fetch hardcoded empty; no concrete SensorAdapter::write() overrides; SQL DML returns NotImplemented |

Bundle A.2 owner: state-manager + story-writer. Each row requires:
1. Read the story file frontmatter.
2. Run stub-residue check on delivery files.
3. Set status to `partial-merge` (with graduation contract stub) or `merged` (if clean).
4. Update STORY-INDEX status column atomically.
