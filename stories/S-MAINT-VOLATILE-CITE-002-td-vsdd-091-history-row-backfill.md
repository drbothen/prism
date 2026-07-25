---
document_type: story
story_id: S-MAINT-VOLATILE-CITE-002
title: "TD-VSDD-091 backfill (part 2 of 2): correct volatile line-cites in immutable history rows per correction-for-accuracy exception"
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
# tdd_mode: strict is mandatory per BC-8.30.001 invariant 2.
# No Rust code is touched (crates_touched: []) so todo!()/Red Gate discipline has
# no behavioral effect. Present for schema compliance.
subsystems: []
crates_touched: []
target_module: .factory
capabilities: []
behavioral_contracts: []
# BC status: pending PO authorship.
# Same governance-integrity BC requirement as S-MAINT-VOLATILE-CITE-001.
# This story MUST remain status: draft until that BC is authored and anchored.
verification_properties: []
depends_on:
  - S-MAINT-VOLATILE-CITE-001
# depends_on justification:
#   S-MAINT-VOLATILE-CITE-001 reduces the L9 hit count to the history-row-only
#   residual. This story can then verify its own delta cleanly without ambiguity
#   about whether a remaining hit belongs to normative text or history rows.
#   Additionally, the POL-1 editorial policy applied here is more restrictive than
#   the normative-text policy; the implementer needs the normative-text baseline
#   to be clean before isolating the history-row problem space.
blocks: []
points: 8
estimated_days: 2.5
risk: MEDIUM
# Risk justification:
#   POL-1 immutability of record rows requires explicit justification for each
#   change (correction-for-accuracy exception). This is judgment-intensive compared
#   to the mechanical find-and-replace of S-MAINT-VOLATILE-CITE-001. Risk of
#   incorrect correction-for-accuracy invocation is real: modifying a history row
#   "because it had a line cite" is only valid when the original cite is factually
#   wrong (e.g., the line number was already stale when written). If the cite was
#   accurate at the time of writing, a history row correction requires additional
#   justification. Adversarial review cascade verifies all corrections satisfy the
#   exception criteria.
assumption_validations: []
risk_mitigations: []
---

# S-MAINT-VOLATILE-CITE-002: TD-VSDD-091 Backfill (Part 2 of 2) — Volatile Line-Cites in Immutable History Rows

## Narrative

As a records-discipline maintainer, I want all volatile file line-number citations in
immutable history rows (changelog rows in BCs/ADRs/VPs, STATE.md decision-log entries,
adversary pass reports, and burst log entries under `.factory/cycles/`) to be corrected
to durable anchors under the correction-for-accuracy exception, so that future readers
following these citations reach the content they are intended to reference, and so that
`scripts/records-lint.sh --full-scan` exits 0 across all five L9 arms for the full
`.factory/` corpus.

This is part 2 of a two-part backfill. Part 1 (S-MAINT-VOLATILE-CITE-001) handles
live normative spec text, which uses a simpler find-and-replace remediation. This story
handles the higher-judgment class of immutable history rows.

## Background: POL-1 and the Correction-for-Accuracy Exception

**POL-1** states that history rows (changelog rows, decision-log entries, pass report rows,
burst log entries) are immutable: they record what was believed at the time of writing,
and that record must not be retroactively altered to match subsequent changes.

The **correction-for-accuracy** exception permits modifying a history row when the
original entry contained a factual error — something that was already wrong at the time
of writing, not something that became wrong later. The exception was established by
precedent in ARCH-INDEX v2.272 (which corrected stale version references in subsystem
registration rows). The rationale: a record that cited `BC-2.16.009 v1.23:line 382` where
line 382 in v1.23 contained something different from what was described is a factual error
in the record, not merely a drift. A correction-for-accuracy edit that replaces this with
`BC-2.16.009 v1.23 §Postcondition P9` is permissible because it improves navigational
accuracy without altering the historical fact being recorded.

**The distinction that matters:** A line cite that was accurate-at-write-time but has
since drifted due to later edits is a *time-decay error*, not a *factual error*. Under
POL-1 strict reading, time-decay alone does not justify correction. However, TD-VSDD-091
amended 2026-07-24 establishes that line cites were NEVER valid in record-tier text — the
original emit was always a violation of the (then-to-be-codified) rule. Every volatile
cite in a history row is therefore retroactively a factual error in citation form, and
the correction-for-accuracy exception applies across the board. This is the project-local
rationale for applying the exception.

This story implements that rationale, row by row, with explicit per-row justification
for each correction.

## Scope

### In scope (this story)

Immutable history rows containing volatile line cites:
- Changelog/version-history rows in BC, ADR, and VP files
- `STATE.md` decision-log entries (D-NNN rows) citing line numbers
- `SESSION-HANDOFF.md` historical sections citing line numbers
- Adversary pass report files under `.factory/cycles/**/*` — only history-row text
  within those reports (pass reports themselves are history artifacts; volatile cites
  within them are correctable under the same exception rationale)
- Burst log entries that cite line numbers in their rationale text
- The `STORY-INDEX.md` historical registration section and decision-log entries

### Out of scope (handled by S-MAINT-VOLATILE-CITE-001)

- Normative prose sections of BCs, ADRs, VPs — already covered
- Non-history sections of index files — already covered

## Acceptance Criteria

### AC-001 — `--full-scan` exits 0 across ALL `.factory/` files
After all history-row remediations are applied, running
`bash scripts/records-lint.sh --full-scan`
reports zero L9 violations across the full `.factory/` corpus (both normative and
history-row text). This is the primary mechanical acceptance gate. When combined with
S-MAINT-VOLATILE-CITE-001's gate, the full corpus is clean.
(traces to TD-VSDD-091 operative text and TD-VSDD-092 §L9 arm definition; both in
SESSION-HANDOFF.md §Standing Rules; the complete zero-L9-hit criterion establishes
the correction-for-accuracy exception is exhaustively applied)

### AC-002 — every history-row correction documents its POL-1 justification
For each history row modified, the implementer records a correction justification in
the commit message (or in a correction-memo file if the volume warrants it). The
justification must state: (a) the original cite text, (b) the replacement anchor,
(c) the POL-1 exception rationale (correction-for-accuracy per TD-VSDD-091 retroactive
classification), and (d) whether the original cite was accurate-at-write-time (known
or unknown). This provides an audit trail for any future review of the immutable record
corpus.
(traces to POL-1 correction-for-accuracy exception; ARCH-INDEX v2.272 precedent
establishes the pattern)

### AC-003 — no historical facts altered, only citation form corrected
Every history row correction changes only the citation-form of a navigational reference
(from a line number to a section/symbol anchor). No date, no decision rationale, no
finding classification, no status value, no version number, and no behavioral claim in
the history row is altered. If a volatile cite removal incidentally reveals that the
surrounding historical fact is also stale, that fact is NOT corrected in this story —
it is surfaced to the orchestrator as a separate finding. The scope of this story is
citation form only.
(traces to POL-1 immutability principle; the exception is narrow — factual content of
history rows is preserved verbatim)

### AC-004 — slash-continuation residuals from S-MAINT-VOLATILE-CITE-001 addressed
If S-MAINT-VOLATILE-CITE-001 documented any uncorrected slash-continuation segments
(arm-5 detector gap) in its AC-003 gap note, this story addresses those residuals for
the history-row portion of the corpus. The gap note from story 1 guides the manual
sweep. If the gap note recorded zero slash-continuation residuals, this AC is
vacuously satisfied.
(traces to TD-VSDD-092 §arm-5 limitation; the two-story sequence must jointly achieve
full coverage)

## Work-List Methodology

The authoritative work-list for this story is the output of:
```
bash scripts/records-lint.sh --full-scan
```
run **after** S-MAINT-VOLATILE-CITE-001 is merged and the normative-text fixes are in
place. At that point, all remaining L9 hits are attributable to history-row text covered
by this story.

If S-MAINT-VOLATILE-CITE-001 is not yet merged, the implementer can still work by
running `--full-scan` and manually filtering the output to history-row text (changelog
sections, D-NNN rows, `.factory/cycles/` files, burst log entries). The filter is manual
— the script reports file + matching line content but does not classify normative vs
history. This is the primary implementation complexity of this story.

**Why history-row work is harder than normative-text work:** Normative text changes are
straightforward — the current correct anchor replaces the stale line cite. History-row
changes require tracing what the original cite pointed to at the time of writing, which
may require: (1) reading the git history of the cited file to find what was at that line
at that version, (2) identifying the nearest durable anchor for that content, and (3)
verifying the replacement anchor conveys the same navigational intent. For cites that
are simply stale (the referenced content moved), this is a ~5-minute exercise per cite.
For cites where the referenced file was significantly restructured, more research is
needed.

## Remediation Class: Immutable History Rows

This story handles the **immutable history row** class:

> **Remediation rule:** Find each volatile cite pattern in a history row (changelog row,
> decision-log entry, pass report body, burst log row). Using the correction-for-accuracy
> exception: replace the line-number cite with the durable section/symbol anchor that
> refers to the same content. Document the correction per AC-002. Do NOT alter any
> historical fact in the surrounding row.

**Policy constraint:** If replacing the line cite requires understanding what the line
pointed to at a past point in history, use `git log` on the cited file to inspect the
historical content. Do NOT guess. A corrected cite that points to a different location
than the original is a worse error than the original stale cite.

## Architecture Mapping

| Component | Location | Pure/Effectful |
|-----------|----------|---------------|
| `scripts/records-lint.sh` | `scripts/` | Pure (read-only scan) |
| BC changelog sections | `.factory/specs/behavioral-contracts/BC-*.md` | Pure (records-only text edits) |
| ADR changelog sections | `.factory/specs/architecture/decisions/ADR-*.md` | Pure (records-only text edits) |
| VP changelog sections | `.factory/specs/verification-properties/VP-*.md` | Pure (records-only text edits) |
| `STATE.md` D-NNN rows | `.factory/STATE.md` — NOTE: state-manager must perform STATE.md edits; see §Architecture Compliance Rules | Effectful (special routing required) |
| `.factory/cycles/**` (pass reports, burst logs) | `.factory/cycles/` | Pure (correction-for-accuracy text edits; no state-machine implications) |

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | A history-row cite points to a file that no longer exists in the repo | Correct the cite to the best-known durable anchor. If no anchor exists (file was deleted entirely), replace with a text description of what the cite was pointing to and note "file deleted." Record the correction per AC-002 with "unknown original accuracy" classification. |
| EC-002 | A history-row cite that was accurate-at-write-time but the referenced line content has since moved | Replace with the durable anchor for where that content is now. The cite was a violation of the rule even when written; correction-for-accuracy applies. |
| EC-003 | A history-row entry in `STATE.md` (D-NNN decision rows) | STATE.md is state-manager-owned. The implementer identifies the correction needed and routes it to state-manager via the orchestrator, rather than directly editing STATE.md. This is a routing-constraint, not a scope-exclusion: the correction still happens in the same work cycle. |
| EC-004 | A history row that contains multiple volatile cites for a single event | Replace all cites in the row in the same atomic correction. Record each replacement in the AC-002 justification. |
| EC-005 | A slash-continuation segment in a history row (`NNN/NNN` format) | These are not caught by L9 arm-5. They must be found by manual inspection (see S-MAINT-VOLATILE-CITE-001 AC-003 gap note). If any such residuals were documented in story 1, handle them here for history rows. |
| EC-006 | A history row in a BC file is referenced by another BC's changelog row | Both cites are independently in scope. Correct each file where the violating cite appears; do not follow cross-reference chains beyond two hops. |

## Token Budget Estimate

| Artifact | Estimated Tokens | Notes |
|----------|-----------------|-------|
| This story file | ~8,500 | |
| `scripts/records-lint.sh` | ~5,000 | Required reading before starting |
| Per BC file (changelog section only) | ~3,000 | Load only the file being edited |
| Per cycle/pass-report file | ~8,000–30,000 | Pass reports vary widely; large reports need sub-burst isolation |
| `STATE.md` (if consulted for D-NNN context) | ~15,000 | Read only the decision-log section relevant to the cite |
| **Per sub-burst ceiling** | ~40,000 | One file at a time; do not load multiple `.factory/cycles/` reports simultaneously |

**Context management discipline:** Pass report files under `.factory/cycles/` can be
very large (adversary cascade reports for 9-pass cascades may exceed 20,000 tokens each).
The implementer MUST process one file per sub-burst. Do not attempt to process the
entire `.factory/cycles/` directory in a single context.

## Tasks

### Phase A — Baseline and S-MAINT-VOLATILE-CITE-001 dependency check

- [ ] **T-A01**: Confirm S-MAINT-VOLATILE-CITE-001 is merged. Run
  `bash scripts/records-lint.sh --full-scan` and capture the output. All remaining
  L9 hits should be in history-row text. If any normative-text hits remain from
  story 1, do not proceed — re-block on story 1 completion.

- [ ] **T-A02**: Inspect the gap note from S-MAINT-VOLATILE-CITE-001 AC-003. Note any
  slash-continuation residuals in history-row text that need manual sweep (EC-005).

### Phase B — BC, ADR, VP changelog section corrections

- [ ] **T-B01..T-BNN**: For each BC/ADR/VP file with L9 hits in its changelog/version-
  history section: read the file, identify each volatile cite in history rows, research
  the original navigational target (git history if needed), replace with durable anchor,
  record the correction per AC-002, write the file. Do not alter surrounding historical
  facts.

  STATE.md routing note (EC-003): if a BC changelog row cites a specific D-NNN entry by
  line-number rather than by D-NNN label, the cite can be replaced inline in the BC
  (the BC is implementer-owned). Do not route this to state-manager unless the STATE.md
  file itself needs editing.

### Phase C — STATE.md D-NNN row corrections (state-manager routed)

- [ ] **T-C01**: Enumerate any D-NNN decision-log rows in STATE.md containing volatile
  line cites. For each, prepare the corrected text with durable anchor and
  the AC-002 justification. Route to state-manager for the actual edit (EC-003):
  the implementer does NOT directly edit STATE.md.

### Phase D — `.factory/cycles/` pass report and burst log corrections

- [ ] **T-D01..T-DNN**: For each adversary pass report or burst log file under
  `.factory/cycles/` with L9 hits: read the file (one per sub-burst), identify
  volatile cites, research original navigational targets, replace with durable anchors,
  record corrections per AC-002, write the file.

  Volume note: this phase is likely the largest by file count. Process in alpha order,
  one file per sub-burst, and re-run `--full-scan` after each file to track progress.

### Phase E — STORY-INDEX and SESSION-HANDOFF historical sections

- [ ] **T-E01**: STORY-INDEX historical registration entries and any decision-log prose
  citing line numbers. Replace volatile cites per the remediation rule.

- [ ] **T-E02**: SESSION-HANDOFF historical sections (not current-session content, which
  is state-manager-owned). Correct volatile cites per the remediation rule.

### Phase F — Final verification

- [ ] **T-F01**: Run `bash scripts/records-lint.sh --full-scan`. Confirm exit 0 across
  all five L9 arms for the entire `.factory/` corpus. If any hits remain, address them
  before declaring done.

- [ ] **T-F02**: Confirm slash-continuation residuals from S-MAINT-VOLATILE-CITE-001
  AC-003 gap note are addressed (AC-004). Document the result (zero remaining, or a
  new story anchor if any are deferred with proper justification and explicit story ID).

### Merge gate

- [ ] **MERGE-GATE-L9-FULL-CORPUS**: `bash scripts/records-lint.sh --full-scan` exits 0
  across all `.factory/` files. Zero L9 violations in normative text (already clean from
  story 1) and zero in history-row text (cleaned by this story).

- [ ] **MERGE-GATE-POL1-JUSTIFICATION-LOG**: Every history row modified has a documented
  correction justification in the commit message (or correction-memo file) per AC-002.

- [ ] **MERGE-GATE-HISTORICAL-FACTS-UNCHANGED**: Adversarial review confirms no
  historical facts (dates, decision outcomes, finding classifications, version numbers,
  status values) changed. Only citation form changed.

## Previous Story Intelligence

- **S-MAINT-VOLATILE-CITE-001** (predecessor): This story directly depends on story 1's
  output. The gap note from story 1's AC-003 informs this story's Phase F slash-continuation
  sweep. The implementer reads story 1's version history row and AC-003 gap note before
  beginning Phase A.
- **ARCH-INDEX v2.272 correction-for-accuracy precedent**: Established the pattern for
  correcting stale version references in history rows. That correction amended registration
  row entries that cited wrong version numbers for subsystems. This story applies the same
  exception class to a different citation form (line numbers vs version numbers). The
  POL-1 justification structure mirrors the v2.272 precedent.

## Architecture Compliance Rules

1. **POL-1 immutability + correction-for-accuracy exception:** History rows are immutable
   except for factual errors. TD-VSDD-091 retroactively classifies volatile line cites as
   factual errors in citation form. Correction-for-accuracy applies. Each correction must
   be documented per AC-002.

2. **TD-VSDD-091 operative text (2026-07-24 amendment):** All record-tier text — including
   changelog rows, decision-log entries, and adversary pass reports — must use
   section/symbol/anchor cites. The former "pass-report changelog" exception is retired.

3. **STATE.md routing (EC-003):** The state-manager owns STATE.md. Implementer prepares
   the corrected text; state-manager applies it. Implementer does NOT directly edit
   STATE.md.

4. **TD-VSDD-060 sibling-site sweep:** When correcting a volatile cite in a history row,
   grep the same file for the same cite pattern to find sibling occurrences. A file that
   cites `BC-2.16.009:NNN` in three places gets all three corrected in one sub-burst.

5. **TD-VSDD-053 single-commit-per-burst:** Each sub-burst (one file processed) produces
   one `.factory/` commit. The volume of changes in this story makes multi-commit chains
   likely without explicit discipline; each file must be a distinct sub-burst commit.

6. **CLAUDE.md §Source-of-Truth Precedence:** If researching a volatile cite's original
   target reveals that the cited content is also substantively wrong (not just the citation
   form), that substantive issue is SURFACE AND ROUTE — it is not fixed silently in this
   story. Citation form is the only scope.

## Library and Framework Requirements

| Library/Tool | Version/Source | Purpose |
|-------------|---------------|---------|
| `scripts/records-lint.sh` | Project-local, `--self-probe` 6/6 confirmed 2026-07-24 | Work-list and acceptance gate |
| `git log` | Standard git toolchain | Researching historical cite targets in modified files |

No external library dependencies.

## File Structure Requirements

### Files to CREATE

None. This story modifies existing files only.

### Files to MODIFY

All `.factory/` history-row files that `scripts/records-lint.sh --full-scan` identifies
as having L9 hits, after S-MAINT-VOLATILE-CITE-001 normative-text fixes are merged.
The specific file list is determined by script output at implementation time.

**Exception:** STATE.md edits are routed to state-manager (EC-003). The implementer
writes the correction text but does not apply it to STATE.md directly.

### Files NOT to modify

| File | Reason |
|------|--------|
| Any BC/ADR/VP normative (non-changelog) section | Belongs to S-MAINT-VOLATILE-CITE-001; already clean after story 1 merges |
| `scripts/records-lint.sh` | Production toolchain, not a `.factory/` record |
| Any `crates/**` file | No code changes in this story |

## Forbidden Dependencies

No new crate dependencies. No changes to `Cargo.toml` files.

## UX Screen References

N/A — no user-facing surface changes.

## Dependency Graph Edges

```
S-MAINT-VOLATILE-CITE-001 (normative text backfill)
  → S-MAINT-VOLATILE-CITE-002 (this story — history row backfill)
    → (no blockers; this completes the TD-VSDD-091 backfill sequence)
```

## Version History

| Version | Date | Change Summary |
|---------|------|----------------|
| 1.0 | 2026-07-25 | Initial story creation. Records-discipline backfill story for TD-VSDD-091 (2026-07-24 amendment). Scope: immutable history rows (BC/ADR/VP changelogs, STATE.md D-NNN rows, SESSION-HANDOFF historical sections, `.factory/cycles/` pass reports and burst logs, STORY-INDEX historical entries). Depends on S-MAINT-VOLATILE-CITE-001. POL-1 correction-for-accuracy exception applies per TD-VSDD-091 retroactive classification. Primary acceptance gate: `bash scripts/records-lint.sh --full-scan` exits 0 across full `.factory/` corpus. Points: 8 (higher than story 1 due to POL-1 judgment overhead and git-history research per volatile cite). |
