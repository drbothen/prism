---
document_type: story
story_id: S-MAINT-VOLATILE-CITE-001
title: "TD-VSDD-091 backfill (part 1 of 2): replace volatile line-cites with durable anchors in live normative spec text"
wave: maintenance
epic_id: maintenance
priority: P2
status: draft
version: "1.0"
updated: "2026-07-25"
level: "L2"
producer: story-writer
timestamp: "2026-07-25T00:00:00Z"
tdd_mode: strict
# tdd_mode: strict is the mandatory default per BC-8.30.001 invariant 2.
# No Rust code is touched by this story (crates_touched: []) so the Red Gate /
# todo!() stub discipline does not apply. The acceptance gate is a mechanical
# script invocation (see AC-001). The tdd_mode field is present for compliance
# with the story schema; it has no behavioral effect on a records-only delivery.
subsystems: []
# Cross-cutting governance work. No single subsystem owns .factory/ records compliance.
crates_touched: []
target_module: .factory
capabilities: []
behavioral_contracts: []
# BC status: pending PO authorship.
# TD-VSDD-091 and TD-VSDD-092 are operational disciplines defined in
# SESSION-HANDOFF.md §Standing Rules, not behavioral contracts. A product-owner
# BC covering records-discipline correctness or governance audit-trail integrity
# would need to be authored before this story can reach status: ready.
# This story MUST remain status: draft until that BC is anchored.
verification_properties: []
depends_on: []
blocks:
  - S-MAINT-VOLATILE-CITE-002
# blocks anchor justification:
#   S-MAINT-VOLATILE-CITE-002 depends on this story to handle the normative
#   spec-file portion of the corpus, reducing the L9-hit count so the history-row
#   story can verify clean-slate for the remaining portion (history text only).
#   The split is by remediation class, not by file; a single file can contain both
#   normative and history-row text. Enforcing story-1-first simplifies the delta
#   verification for story 2.
points: 5
estimated_days: 2.0
risk: LOW
# Risk justification:
#   Pure records text changes — no production code, no type changes, no API surface.
#   The mechanical gate (--full-scan exit 0) is deterministic. Primary risk is
#   accidental semantic drift if an anchor replacement changes the meaning of the
#   original citation. AC-004 (no behavioral semantics changed) mitigates this.
assumption_validations: []
risk_mitigations: []
---

# S-MAINT-VOLATILE-CITE-001: TD-VSDD-091 Backfill (Part 1 of 2) — Volatile Line-Cites in Live Normative Spec Text

## Narrative

As a records-discipline maintainer, I want all live normative text in `.factory/` spec
artifacts (behavioral contracts, ADRs, verification properties, and non-history sections
of index files) to use durable section/symbol/anchor references instead of volatile file
line-number citations, so that `scripts/records-lint.sh --full-scan` exits 0 for its L9
arm on all files in scope for this story, making record text resilient to routine diff
re-numbering.

This is part 1 of a two-part backfill. Part 2 (S-MAINT-VOLATILE-CITE-002) covers
immutable history rows, which require distinct handling under POL-1.

## Background: Why This Story Exists

TD-VSDD-091 was amended on 2026-07-24 to retire the former "pass-report/changelog"
exception, banning volatile line-cites from **all** record-tier text including changelog
rows, adversary pass reports, and decision-log entries — not only narrative prose.

TD-VSDD-092's `scripts/records-lint.sh` check L9 is the mechanical enforcement gate for
that rule. However, L9 was inoperative from its introduction until 2026-07-24: `.factory/`
is a separate git worktree with its own index, so `git diff --cached` invoked from the
project root always returned empty and L9 early-exited without scanning a single
`.factory/` addition. The gate gained five detector arms and the worktree-index fix on
that date. All violations added before 2026-07-24 are grandfathered by the ratchet (L9
scans only staged additions; it does not fail on pre-existing lines).

A `--full-scan` invocation (which overrides the ratchet and scans all file content)
reports **537 pre-existing volatile cites** across the `.factory/` corpus at authoring
time (2026-07-25). The human has directed a full backfill via tracked stories rather than
forward-only enforcement.

**Cost/benefit tradeoff stated explicitly:** This story churns a large number of record
entries solely to change citation form. The justification is that a record citing a line
number it cannot independently verify has already drifted — the moment a later commit
shifts lines above the cite, the citation becomes silently incorrect, and the next
adversarial reader following it finds the wrong content. Durable anchors (section
headings, symbol names, invariant IDs, decision-table row labels) degrade gracefully.
The churn is a one-time cost; the benefit is a permanently verifiable citation graph.

## Behavioral Contracts

| BC | Title | Version | Scope in This Story |
|----|-------|---------|---------------------|
| (none) | — | — | This story has no behavioral contracts yet. See BC status comment in frontmatter. status: draft and will not advance to ready until PO authors a governance-integrity BC. |

## Scope

### In scope (this story)

Live normative text in the following artifact classes:
- Behavioral contracts (`BC-*.md` files under `.factory/specs/behavioral-contracts/`)
- Architecture decision records (ADR files under `.factory/specs/architecture/decisions/`)
- Verification property files (`VP-*.md` files under `.factory/specs/verification-properties/`)
- Non-history sections of index files (`BC-INDEX.md`, `ARCH-INDEX.md`, `VP-INDEX.md`,
  `STORY-INDEX.md`) — specifically any normative prose, table rows, and rationale sections
  that are NOT part of the changelog/version-history section of those files

"Live normative text" means: any text that is expected to be read as a current behavioral
assertion, architectural decision, or process requirement, as opposed to a historical record
of past decisions (which is covered by S-MAINT-VOLATILE-CITE-002).

### Out of scope (handled by S-MAINT-VOLATILE-CITE-002)

- Changelog/version-history rows in BC, ADR, VP files
- The `STORY-INDEX.md` decision-log section and historical registration entries
- `STATE.md` and `SESSION-HANDOFF.md` decision-log entries
- Adversary pass reports under `.factory/cycles/`
- Burst log entries

## Acceptance Criteria

### AC-001 — `--full-scan` exits 0 for L9 across all in-scope files
After all normative-text volatile-cite remediations are applied, running
`bash scripts/records-lint.sh --full-scan`
with the scope restricted to the artifact classes listed in §Scope (or equivalently,
running `--full-scan` across all `.factory/` and confirming that remaining L9 hits are
exclusively in history-row text covered by S-MAINT-VOLATILE-CITE-002) reports zero L9
violations attributable to normative prose in BCs, ADRs, VPs, and non-history index
sections. This is the primary mechanical acceptance gate for this story.
(traces to TD-VSDD-091 operative text and TD-VSDD-092 §L9 arm definition; both
anchored in SESSION-HANDOFF.md §Standing Rules)

### AC-002 — every replacement uses a durable section/symbol/anchor reference
Every volatile cite removed is replaced with at least one of:
- A markdown section heading anchor (e.g., `§Postconditions`, `§Security requirement`)
- A canonical symbol or function name (e.g., `validate_header_scheme`, `SpecLoader::parse`)
- A behavioral contract clause identifier (e.g., `BC-2.16.009 P9`)
- A decision-record label (e.g., `ADR-053 §D2`, `ADR-054 §D11`)
- A story/task identifier (e.g., `T-B03`, `AC-017`)
The replacement must convey the same navigational intent as the original line cite: a
reader following the anchor must reach the same content the cite was pointing to.
(traces to TD-VSDD-091 operative requirement: "section/symbol/anchor cites ONLY")

### AC-003 — the slash-continuation detector gap is explicitly documented
After this story completes, `.factory/docs/` or an equivalent records-governance location
contains a note (or the gap is recorded in the active STATE.md) documenting that
`scripts/records-lint.sh --full-scan` L9 arm-5 does NOT catch slash-continuation
volatile segments (e.g., `~LNNN/NNN/NNN` — the bare-digit continuation fragments lack the `L`
prefix required by the pattern). A `--full-scan` zero result therefore does NOT prove
zero volatile cites corpus-wide. The note must identify this as a known residual requiring
either: (a) a manual sweep for the slash-continuation form performed as part of this story,
or (b) an explicit future tracking anchor for a follow-up. If (a), document the sweep result.
If (b), the tracking anchor must be a specific story ID or governance ticket, not "TBD."
(traces to TD-VSDD-092 §arm-5 limitation; completeness requires explicit residual tracking)

### AC-004 — no behavioral semantics changed
No behavioral contract postcondition, precondition, or invariant clause changes meaning
as a result of this story's edits. Changes are restricted to the citation-form of
navigational references; the referenced content itself is untouched. If a normative
section's prose must be updated to remove a line cite AND that prose change incidentally
alters the semantics of a BC clause or ADR decision, that change must be STOPPED,
surfaced to the orchestrator, and routed to the owning specialist (product-owner for BC
prose; architect for ADR prose). This story's implementer does NOT self-authorize semantic
changes. AC-004 compliance is verified by the adversarial review cascade: the adversary
reads the before/after diff for any BC or ADR modification and confirms the behavioral
meaning is unchanged.
(traces to CLAUDE.md §Source-of-Truth Precedence rule: spec wins; changes require
specialist routing)

## Work-List Methodology

**Do NOT enumerate all 537 cites in this story.** The work-list is mechanically generated
and will drift as the corpus evolves. The authoritative work-list is the output of:

```
bash scripts/records-lint.sh --full-scan
```

This command is the work-list. The implementer runs it at the start of each sub-burst to
get the current hit list for the in-scope artifact classes, applies remediations, and
re-runs to verify progress.

**Baseline at authoring time (2026-07-25):** `--full-scan` reports 537 pre-existing L9
violations across all `.factory/` files. This figure is **arm-inclusive** (all five L9
arms contribute) and **corpus-inclusive** (both normative text and history rows). The
normative-text portion covered by this story is a subset of the 537. The exact breakdown
is unknown until `--full-scan` output is inspected by the implementer; there is no
pre-enumerated list.

**Why 537 will drift:** new stories, new BC amendments, new ADR decisions, and state
transitions add content daily. The baseline is a reference figure for scope awareness, not
a fixed target count. The acceptance criterion is a zero result from `--full-scan` for
in-scope files — not reaching exactly "537 minus X" remaining hits.

## Remediation Class: Live Normative Text

This story handles the **live normative text** class only:

> **Remediation rule:** Find each volatile cite pattern (e.g., `file.rs:NNN`, `~LNNN`,
> `vX.Y:NNN`, bare `LNNN` in navigational context) in the target file's non-history
> sections. Replace with a durable anchor that conveys the same navigational intent.
> The fix is an in-place text substitution — the surrounding prose retains its meaning.

**Examples of valid replacements:**

| Before (violates L9) | After (durable anchor) |
|---------------------|----------------------|
| `spec_parser.rs:[NNN]` | `validate_header_scheme` in `spec_parser.rs` |
| `BC-2.16.009 v1.23:NNN` | `BC-2.16.009 §Postcondition P9` |
| `ADR-053.md:line 147` | `ADR-053 §D2 dispatch table` |
| `pipeline.rs ~LNNN` | `build_request` free function in `pipeline.rs` |

Volatile cites in **history sections** (changelog rows, version history tables) are NOT
touched by this story — they belong to S-MAINT-VOLATILE-CITE-002.

## Architecture Mapping

| Component | Location | Pure/Effectful |
|-----------|----------|---------------|
| `scripts/records-lint.sh` | project root `scripts/` | Pure (read-only scan; exit code is the gate) |
| `.factory/specs/behavioral-contracts/BC-*.md` | `.factory/specs/behavioral-contracts/` | Pure (records-only text edits) |
| `.factory/specs/architecture/decisions/ADR-*.md` | `.factory/specs/architecture/decisions/` | Pure (records-only text edits) |
| `.factory/specs/verification-properties/VP-*.md` | `.factory/specs/verification-properties/` | Pure (records-only text edits) |
| Index files (BC-INDEX, ARCH-INDEX, VP-INDEX, STORY-INDEX) | `.factory/specs/` and `.factory/stories/` | Pure (non-history sections only) |

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | A volatile cite appears inside a markdown code block (fenced with ` ``` `) in normative prose | Evaluate context: if the code block is illustrative Rust/bash source showing a specific line, the cite is functional (needed for navigation) and should be replaced with a symbol anchor. If the code block is showing a shell command like `cargo test`, no cite present — skip. |
| EC-002 | A volatile cite in a normative section also appears verbatim in the file's changelog row | Replace the normative-section occurrence only; leave the changelog row for S-MAINT-VOLATILE-CITE-002. |
| EC-003 | The referenced line range covers a function body rather than a named symbol | Use the function name as the anchor. If no function name exists (e.g., a closure), use the nearest enclosing function or a descriptive behavioral phrase. |
| EC-004 | Slash-continuation segments (`~LNNN/NNN/NNN`) where only the first fragment has the `L` prefix | These are NOT caught by `--full-scan` L9 arm-5. They are a known residual. If encountered during normative-text sweep, fix them (they are also volatile cites). AC-003 requires documenting what was found. |
| EC-005 | A cite that points to a line currently within a history row of a different BC (cross-document history reference) | This is a normative prose cite if it appears in a normative section of the source file. Replace it with a durable anchor even if the TARGET line is within a history section of a different file. The restriction on history rows is about where the CITE APPEARS, not where it points. |

## Token Budget Estimate

| Artifact | Estimated Tokens | Notes |
|----------|-----------------|-------|
| This story file | ~8,000 | |
| `scripts/records-lint.sh` (full script text) | ~5,000 | Work-list authority; implementer reads before starting |
| BC files (variable — implementer processes per-file) | ~12,000 per file | Load only the file being edited in each sub-burst |
| ADR files (per-file) | ~10,000 per file | Load only the file being edited |
| VP files (per-file) | ~8,000 per file | Load only the file being edited |
| Index files (non-history sections) | ~15,000 per file | Larger files; context-manage carefully |
| **Per sub-burst ceiling** | ~40,000 | One file + story + script per sub-burst; do not load the full corpus simultaneously |

**Context management discipline:** This story MUST be implemented in sub-bursts of 1-2
files at a time. The implementer MUST NOT load more than 2-3 `.factory/` files
simultaneously. Each sub-burst: (1) run `--full-scan` to get current hits, (2) pick
one file, (3) read only that file, (4) apply remediations, (5) write, (6) repeat. The
adversary runs `--full-scan` as its primary verification tool, not per-file review.

## Tasks

### Phase A — Baseline and scope enumeration

- [ ] **T-A01**: Run `bash scripts/records-lint.sh --full-scan` and capture the output.
  Record the total L9 hit count at implementation-start time. Partition hits by artifact
  class: BC files vs ADR files vs VP files vs index non-history sections. This partition
  determines the in-scope work for this story vs the history-row work for
  S-MAINT-VOLATILE-CITE-002.

- [ ] **T-A02**: Identify slash-continuation segments (`NNN/NNN` fragments in cite context)
  that are missed by L9 arm-5. Enumerate the count. If nonzero, either fix them in-scope
  (they are also volatile cites) or record the count and a story anchor for a follow-up.
  This satisfies AC-003.

### Phase B — Per-file normative text remediation (BC files)

- [ ] **T-B01..T-BNN**: For each BC file with L9 hits in normative (non-changelog) sections:
  read the file, identify each volatile cite, replace with a durable anchor per the
  remediation rule in §Remediation Class, write the file. Verify the BC's behavioral
  meaning is unchanged (AC-004). Re-run `--full-scan` after each file to confirm hits
  decrease. Do NOT modify changelog/version-history rows — those are for
  S-MAINT-VOLATILE-CITE-002.

  TD-VSDD-060 sibling-site sweep: if a cite pattern appears in multiple sections of the
  same file, sweep all occurrences.

### Phase C — Per-file normative text remediation (ADR files)

- [ ] **T-C01..T-CNN**: Same pattern as Phase B, for ADR files. ADR prose is
  architect-owned; the implementer confirms that no ADR decision-body changes in meaning.
  Volatile cites in ADR version-history/supersession rows are left for
  S-MAINT-VOLATILE-CITE-002.

### Phase D — Per-file normative text remediation (VP files and index non-history sections)

- [ ] **T-D01..T-DNN**: VP files and normative sections of BC-INDEX, ARCH-INDEX,
  VP-INDEX, STORY-INDEX. For STORY-INDEX, normative text is the table header sections,
  epic headers, and current-status rows; historical registration prose (decision-log
  section) is out of scope.

### Phase E — Verification and gap documentation

- [ ] **T-E01**: Run `bash scripts/records-lint.sh --full-scan` and confirm zero L9 hits
  attributable to normative text in the in-scope artifact classes. Any remaining hits
  should be history-row text belonging to S-MAINT-VOLATILE-CITE-002.

- [ ] **T-E02**: Document the slash-continuation gap finding per AC-003. Write the gap
  note to `.factory/docs/records-lint-known-gaps.md` (create if not present) or add a
  STATE.md decision-log entry pointing to the residual. Include: arm-5 limitation
  description, count found during T-A02, and resolution (fixed in-scope or future anchor).

### Merge gate

- [ ] **MERGE-GATE-L9-NORMATIVE**: `bash scripts/records-lint.sh --full-scan` exits 0
  for all in-scope files (BCs, ADRs, VPs, index non-history sections). Any remaining L9
  hits must be exclusively in history-row text that is explicitly within S-MAINT-VOLATILE-CITE-002
  scope.

- [ ] **MERGE-GATE-SEMANTIC-CLEAN**: Adversarial review confirms no BC or ADR behavioral
  semantics changed. Adversary reads the full diff and certifies AC-004 compliance.

## Previous Story Intelligence

N/A — first story in E-RECORDS-DISCIPLINE / maintenance governance backfill chain.

Prior art from process-governance stories:
- `S-POL-14-STATUS-SYNC-001` (maintenance wave): pattern for a process-governance story
  with `behavioral_contracts: []` and BC authorship pending. Same draft status model applies.
- `S-POL-29-CANONICAL-TEMPLATE-REGISTRY-001` (maintenance): pattern for a records-metadata
  story where the acceptance criterion is a script gate.

## Architecture Compliance Rules

1. **TD-VSDD-091 operative text:** All record-tier text (including normative prose) must
   cite function names + behavioral anchors, NOT `file.rs:NNN` line numbers or `~LNNN`
   approximations. The amendment of 2026-07-24 retired all former exceptions, including
   the "pass-report changelog" exception. This is the rule being enforced by this story.

2. **TD-VSDD-092 §L9 arm scope:** The `records-lint.sh` L9 arm has five detector patterns
   as of 2026-07-24. A zero result from `--full-scan` does NOT catch slash-continuation
   fragments (arm-5 gap). AC-003 requires explicit residual documentation.

3. **POL-1 immutability constraint (NOT applicable here):** History rows are immutable
   under POL-1. This story does NOT touch history rows. Any edit touching a changelog
   row or decision-log entry must STOP and be routed to S-MAINT-VOLATILE-CITE-002.

4. **CLAUDE.md §Source-of-Truth Precedence:** Spec wins over code. Any change to a BC or
   ADR that incidentally modifies behavioral semantics is a spec amendment requiring
   specialist routing. The implementer's mandate is citation-form correction only.

5. **TD-VSDD-053 single-commit-per-burst:** Each logical sub-burst (one file processed)
   produces one `.factory/` commit. Do not accumulate uncommitted changes across multiple
   files before committing.

6. **TD-VSDD-060 sibling-site sweep:** When removing a volatile cite from a normative
   section, grep the same file for the same cite pattern to confirm all occurrences are
   addressed. Cross-file sweep is not required (each file is processed independently).

## Library and Framework Requirements

| Library/Tool | Version/Source | Purpose |
|-------------|---------------|---------|
| `scripts/records-lint.sh` | Project-local, `--self-probe` confirmed 6/6 on 2026-07-24 | Work-list generation and acceptance gate |

No external library dependencies. This story modifies `.factory/` text files only.

## File Structure Requirements

### Files to CREATE

- `.factory/docs/records-lint-known-gaps.md` (create if absent) — gap documentation per AC-003

### Files to MODIFY

All BC, ADR, VP, and index files that `scripts/records-lint.sh --full-scan` identifies as
having L9 hits in normative (non-history) sections. The specific file list is determined
by the script output at implementation time, not pre-enumerated here.

### Files NOT to modify

| File | Reason |
|------|--------|
| Any `.factory/` file's changelog/version-history section | Belongs to S-MAINT-VOLATILE-CITE-002 (history-row remediation class) |
| `STATE.md` | State-manager owns STATE.md; any L9 hits in STATE.md decision-log belong to S-MAINT-VOLATILE-CITE-002 |
| `.factory/cycles/**` (adversary pass reports, burst logs) | History-record class; belongs to S-MAINT-VOLATILE-CITE-002 |
| `scripts/records-lint.sh` | The gate script itself is not a `.factory/` record; it is production toolchain |
| Any `crates/**` file | No code changes in this story |

## Forbidden Dependencies

No new crate dependencies introduced. No changes to `Cargo.toml` files.

## UX Screen References

N/A — no user-facing surface changes.

## Dependency Graph Edges

```
S-MAINT-VOLATILE-CITE-001 (this story)
  depends_on: []
  blocks:
    → S-MAINT-VOLATILE-CITE-002  (history-row backfill; can only begin after normative
                                   fixes reduce the L9 count to history-row-only residual,
                                   enabling clean delta verification for that story)
```

## Version History

| Version | Date | Change Summary |
|---------|------|----------------|
| 1.0 | 2026-07-25 | Initial story creation. Records-discipline backfill story for TD-VSDD-091 amendment (2026-07-24) retiring the pass-report/changelog exception. Scope: live normative spec text (BCs, ADRs, VPs, index non-history sections). Excludes history rows (→ S-MAINT-VOLATILE-CITE-002). 537-site baseline at authoring time (arm-inclusive, corpus-inclusive). Primary acceptance gate: `bash scripts/records-lint.sh --full-scan` exits 0 for in-scope artifact classes. Split recommendation: 2 stories by remediation class (normative vs immutable-history) rather than by artifact class — distinct editorial policies require distinct review lenses. |
