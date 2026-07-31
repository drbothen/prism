---
document_type: story
story_id: S-MAINT-CHANGELOG-BACKFILL-001
title: "Story Changelog Section Corpus Repair — DRIFT-STORY-CHANGELOG-ABSENT-001"
wave: maintenance
epic_id: maintenance
priority: P2
status: draft
version: "1.0"
updated: "2026-07-31"
level: "L2"
producer: story-writer
timestamp: "2026-07-31T00:00:00Z"
tdd_mode: strict
# tdd_mode: strict — mandatory per BC-8.30.001 invariant 2.
# No Rust code is touched (crates_touched: []). Red Gate items are corpus-completeness
# checks that FAIL before the sweep (missing sections exist) and PASS after (zero missing).
# The test vehicle is a check script or grep that finds stories without ## Changelog sections.
# Standard Rust todo!() discipline does not apply; the corpus-check framework serves the
# identical TDD purpose.
subsystems: []
crates_touched: []
target_module: ".factory/stories"
capabilities: []
behavioral_contracts: []
# BC status: pending PO authorship.
# Audit-trail completeness is a governance convention, not a behavioral contract.
# This story MUST remain status: draft until a governance-integrity BC is authored.
verification_properties: []
holdout_scenarios: []
# POL-35 holdout_gate_infra_only_exemption applies: behavioral_contracts: [] (pure
# records governance sweep). holdout_scenarios: [] is compliant for this story.
depends_on: []
blocks:
  - S-MAINT-CONTENT-VERSION-GATE-001
# blocks justification:
#   S-MAINT-CONTENT-VERSION-GATE-001 adds a records-lint check that verifies staged
#   content changes to versioned artifacts include a version bump and changelog row.
#   That check's enforcement scope is limited to files that HAVE a ## Changelog section.
#   Before S-MAINT-CHANGELOG-BACKFILL-001 ships, 67 story files lack the section —
#   the content-version-bump check cannot be enforced against them. After this story
#   ships, all story files have the section and the check can enforce uniformly.
points: 3
estimated_days: 2.0
risk: MEDIUM
# Risk justification:
#   Sweep covers approximately 67 story files. The changelog section added must follow
#   the canonical format used in converged stories. The principal risk is adding a
#   changelog entry that describes a change that did NOT occur in this sweep (i.e.,
#   the entry should say "initial audit-trail section added" not "story content updated").
#   AC-002 and EC-001 document this requirement. The sweep must not add content-changing
#   entries; each new ## Changelog row must describe ONLY the act of adding the section.
assumption_validations: []
risk_mitigations: []
tags:
  - records-governance
  - sweep
  - stories
  - audit-trail
  - drift-resolution
---

# S-MAINT-CHANGELOG-BACKFILL-001: Story Changelog Section Corpus Repair

## Origin

**Corpus-drift registration:** DRIFT-STORY-CHANGELOG-ABSENT-001 (D-2076, 2026-07-31).
Authorized by human as part of FB105 (authorization covers this prerequisite story explicitly).

During FB104b, a burst edited normative content in `S-DEMO-QUERY-PUSHDOWN-001` (v2.8)
without bumping the version or adding a changelog row. The gap was invisible to records-lint
L1 because L1 verifies "declared version consistent with changelog top row" — it cannot
detect "content changed but version untouched." The root cause: `S-DEMO-QUERY-PUSHDOWN-001`
had no `## Changelog` section, so there was no in-artifact way to record the change.

Disk enumeration during FB104b confirmed: approximately 67 of 259 story files lack any
`## Changelog` or `## Version History` section. Among the named instances: `S-QUERY-SCOPE-PARAMS-001`,
`S-3.04-FOLLOWUP-MCP-001`, the PLUGIN-MIGRATION-001-A/B/C set, the S-3.3.0x harness cohort.

Any burst editing one of these 67 files replicates the FB104 gap: normative content can
be changed, the version left untouched, and records-lint L1 trivially passes. This corpus
repair resolves that structural blindspot by ensuring every story file has an in-artifact
audit trail surface before S-MAINT-CONTENT-VERSION-GATE-001 enforces the
content-change-version-bump gate.

---

## Narrative

As a records-discipline maintainer, I want every story file in `.factory/stories/` to
have a `## Changelog` section so that any future burst editing a story file has an
in-artifact location to record the change, so that S-MAINT-CONTENT-VERSION-GATE-001's
content-change-version-bump check can be enforced uniformly across the full story corpus,
and so that the structural blindspot documented in Lesson 117 (records-lint L1 passes
for content changes on files without a changelog section) cannot recur silently.

---

## Background

The canonical `## Changelog` format used in this project is a Markdown table:

```markdown
## Changelog

| Version | Date | Author | Summary |
|---------|------|--------|---------|
| 1.0 | YYYY-MM-DD | story-writer | Initial story creation. [summary] |
```

For the 67 affected stories, the initial changelog row MUST describe ONLY the act of
adding the audit-trail section: "added ## Changelog section (DRIFT-STORY-CHANGELOG-ABSENT-001
corpus repair; no content changes)." It MUST NOT fabricate a content history that did not
exist before this sweep.

The sweep does NOT add changelog rows for changes that occurred before this burst. If a
story is on version 2.8 (like `S-DEMO-QUERY-PUSHDOWN-001` was), the new `## Changelog`
section records the sweep as a single row at whatever version the story currently declares.
The pre-sweep content history is not reconstructed — only the structural section is added.

**IMPORTANT: symmetric-sibling constraint (POL-29 dimension 9a):** Adding a `## Changelog`
section to 67 stories while NOT adding it to the same stories' sibling artifacts (if any)
would create POL-29 9a sibling asymmetry. However, these stories have no named twin files —
each story is standalone in `.factory/stories/`. The POL-29 9a concern does not apply
because the cohort definition is "all 67 files lacking the section" (the cohort IS the
sibling set; adding the section to all of them is the symmetric action).

---

## Behavioral Contracts

| BC | Title | Version | Scope in This Story |
|----|-------|---------|---------------------|
| (none) | — | — | No behavioral contracts yet. Story MUST remain `status: draft` per S-7.01 gate. This MUST is anchored to: AC-001 governs completeness; BC authorship is a human gate. |

---

## Acceptance Criteria

### AC-001 — Zero story files in `.factory/stories/` lack a `## Changelog` section after the sweep
After the sweep completes, running `grep -rL "## Changelog\|## Version History" .factory/stories/*.md`
produces zero results. Every `.md` file in `.factory/stories/` has either a `## Changelog`
or a `## Version History` section. The sweep MUST process all files identified at
implementation time; the exact count is verified by disk enumeration at T-A01.
(verified by RG-001)

### AC-002 — Each new changelog entry describes ONLY the structural addition
Every new `## Changelog` row added by this sweep MUST use the form:
"Added `## Changelog` section per DRIFT-STORY-CHANGELOG-ABSENT-001 corpus repair;
no content changes in this sweep." It MUST NOT describe content changes, version
transitions, or other history that did not occur. The sweep is a structural addition,
not a content edit. Content changes (AC transitions, frontmatter field edits, narrative
edits) are forbidden by this story's scope.
(verified by RG-002)

### AC-003 — The sweep does NOT bump story versions
Adding a `## Changelog` section is a structural addition to the audit-trail surface;
it is NOT a content change to the story's normative content (ACs, architecture mapping,
tasks, narrative, behavioral contracts). Therefore, story `version:` frontmatter fields
MUST NOT be bumped by this sweep. The version remains at whatever value it had before.
The new `## Changelog` row cites the CURRENT version (unchanged) as its reference.
This aligns with how existing story changelog rows record when a section was added
(the section's own first changelog row is the record of its creation, at the current
version, with no content change implied).
(verified by RG-002)

---

## Red Gate Tests

All 3 RG items fail before the sweep and pass after.

- [ ] **RG-001** (`test_changelog_sweep_zero_missing_after`): corpus completeness check —
  before the sweep, run `grep -rL "## Changelog\|## Version History" .factory/stories/*.md`
  and confirm it returns a non-empty list (confirming the "failing" pre-sweep state). After
  the sweep, the same command returns empty. This is the primary acceptance gate. FAILS
  before sweep (missing sections exist); PASSES after sweep (zero missing).

- [ ] **RG-002** (`test_changelog_entries_describe_only_structural_addition`): content audit —
  after the sweep, scan each newly-added `## Changelog` section for the canonical phrase
  "DRIFT-STORY-CHANGELOG-ABSENT-001" and confirm no newly-added row contains terms
  indicating content changes ("updated", "fixed", "amended", "added AC", etc.). This ensures
  the sweep only adds the structural section without fabricating content history. FAILS
  before sweep (sections absent, can't check); PASSES after sweep with correct entries.

- [ ] **RG-003** (`test_changelog_sweep_no_version_bumps`): frontmatter audit — after the
  sweep, verify that no story's `version:` frontmatter field changed. Compare the version
  values before and after using git diff. FAILS if any version was bumped; PASSES if all
  versions remain unchanged (confirming the sweep was purely structural).

**Red Gate density check (BC-5.38.001):** 3 Red Gate tests (RG-001 through RG-003) anchor to
3 acceptance criteria (AC-001 through AC-003). Density ratio: 3 / 3 = 1.0, satisfying
BC-5.38.001. Density validation at dispatch time per `per-story-delivery.md §Red Gate Density
Check`.

---

## Architecture Mapping

| Component | Location | Pure/Effectful |
|-----------|----------|----------------|
| Story file edits (structural section addition) | `.factory/stories/*.md` (67 files) | Pure (additive structural edit; no normative content changes) |
| Post-sweep validation grep | POSIX grep | Pure (read-only verification) |

---

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | A story already has a `## Version History` section (which is equivalent to `## Changelog`) | Do NOT add a `## Changelog` section — `## Version History` serves the same purpose. Only files with neither section receive the new `## Changelog` entry. |
| EC-002 | A story file has `## Changelog` as a heading in a code block (e.g., ` ```markdown ## Changelog ``` `) | If this is the only instance, it does not satisfy the requirement — the heading must appear at the document level (not inside a code block). The validation grep must scan for the heading outside code blocks. Implementation note: if rare, handle manually rather than adding complex code-block detection. |
| EC-003 | A story file has no frontmatter `version:` field (cannot cite a version in the new changelog row) | Use version "0.1" as the initial version in the changelog row and add `version: "0.1"` to frontmatter (this is a structural completeness fix, not a content change). Note this is a rarer case; verify on disk at T-A01. |
| EC-004 | Story files added or completed after this sweep begins (race condition) | Story-writer performs the sweep in a single atomic burst under single-commit-per-burst (TD-VSDD-053). No new stories should be added to `.factory/stories/` during the sweep. The STORY-INDEX count at sweep-start is the authoritative scope. |
| EC-005 | STORY-INDEX.md itself lacks a `## Changelog` section | STORY-INDEX.md is a special aggregate index file. If it lacks `## Changelog`, add it. It is included in the `grep -rL` scan scope. |

---

## Token Budget Estimate

| Artifact | Estimated Tokens | Notes |
|----------|-----------------|-------|
| This story spec | ~5,000 | |
| 67 affected story files (batch processing, 3–5 per context window sub-session) | ~20,000 | Do NOT load all 67 at once; process in batches of 5 per context window |
| Canonical `## Changelog` format reference (from S-MAINT-L11-GATE-001) | ~500 | |
| **Total per implementation session (batched)** | Batch processing required | Exceeds single-session budget; split into sub-batches per T-B01 |

Context management: batch the sweep in groups of 5 story files per sub-session to prevent
context overflow. Each sub-session: (1) identify 5 files lacking `## Changelog`, (2) add
sections, (3) verify with grep. Repeat until all 67 are covered.

---

## Tasks

### Phase A — Pre-sweep enumeration (Red Gate — establish failing state before sweep)

- [ ] **T-A01** (RG-001 failing state): Run `grep -rL "## Changelog\|## Version History" .factory/stories/*.md` and record the count and file list. This is the pre-sweep failing state for RG-001. Document the exact count (expected ≈ 67; verify on disk). Record the file list for batch processing.

- [ ] **T-A02** (version snapshot): Record the current `version:` frontmatter value for each of
  the 67 affected files. This is the baseline for RG-003 (no version bumps after sweep).

### Phase B — Sweep execution (Green — add sections to all affected files)

- [ ] **T-B01** (batched): For each file in the T-A01 list, add the canonical `## Changelog`
  section with a single row describing the structural addition per AC-002. Process in batches
  of 5 files per sub-session to prevent context overflow.
  The canonical row format:
  `| 1.0 | 2026-07-31 | story-writer | Added \`## Changelog\` section per DRIFT-STORY-CHANGELOG-ABSENT-001 corpus repair; no content changes. |`
  (Where "1.0" is replaced with the story's current frontmatter `version:` value.)
  Verify each batch with `grep -rL "## Changelog\|## Version History" .factory/stories/*.md`
  before moving to the next batch.

### Phase C — Verify sweep complete

- [ ] **T-C01** (RG-001 passing state): Run `grep -rL "## Changelog\|## Version History" .factory/stories/*.md`. Confirm zero results. This is the post-sweep passing state for RG-001.

- [ ] **T-C02** (RG-002 entry audit): Scan each newly-added changelog row for the canonical
  phrase "DRIFT-STORY-CHANGELOG-ABSENT-001". Confirm no row contains terms indicating
  content changes.

- [ ] **T-C03** (RG-003 version audit): Confirm via git diff that no story's `version:` field
  changed from the T-A02 baseline.

### Merge gate

- [ ] **MERGE-GATE-ZERO-MISSING**: `grep -rL "## Changelog\|## Version History" .factory/stories/*.md` returns zero results.
- [ ] **MERGE-GATE-NO-CONTENT-CHANGES**: git diff for this PR contains ONLY `## Changelog` section additions and `version:` unchanged. No AC, narrative, task, or behavioral-contract edits.
- [ ] **MERGE-GATE-NO-VERSION-BUMPS**: All story `version:` fields unchanged from pre-sweep state.

---

## Previous Story Intelligence

N/A — first and only story for DRIFT-STORY-CHANGELOG-ABSENT-001.

Related prior art:
- `S-MAINT-ANTIPIN-SWEEP-001` and `S-MAINT-ANTIPIN-SWEEP-002`: sweep stories covering similar
  structural repair of story and spec files. Same batched-execution pattern; same `depends_on`
  a gate story that the sweep enables. Note: ANTIPIN sweeps depend on S-MAINT-L11-GATE-001;
  this sweep does NOT depend on any other story (it creates the prerequisite for
  S-MAINT-CONTENT-VERSION-GATE-001).
- `S-MAINT-VOLATILE-CITE-001` and `S-MAINT-VOLATILE-CITE-002`: earlier sweep pattern for
  `.factory/` files.

---

## Architecture Compliance Rules

1. **Additive-only changes.** This story MUST NOT modify any normative content (ACs,
   tasks, narratives, behavioral contracts, frontmatter other than documented exceptions).
   Only the `## Changelog` section and its first row are new content. Anchor: AC-002.

2. **No prism crate modifications.** No file under `crates/` is touched.

3. **Batch-execute to prevent context overflow.** The 67-file scope exceeds a single context
   window. Sub-batches of 5 files per context window are required (see §Token Budget Estimate
   and T-B01).

4. **No STORY-INDEX.md structural edits** (story content only). STORY-INDEX.md is state-manager
   territory for index rows. However, if STORY-INDEX.md itself lacks a `## Changelog` section
   (EC-005), the structural addition is in scope for this sweep.

5. **TD-VSDD-053 single-commit-per-burst:** All 67 file additions in one commit, not one commit
   per file or one commit per batch. State-manager owns the commit.

---

## Library and Framework Requirements

No external tools required. Uses POSIX `grep`, `find`, and standard text editing tools only.

---

## File Structure Requirements

### Files to MODIFY (structural addition only)

Approximately 67 story files in `.factory/stories/` that currently lack `## Changelog` or
`## Version History` sections. Exact list determined by T-A01 disk enumeration.

### Files NOT to modify

| File | Reason |
|------|--------|
| Any `crates/**` file | No Rust code changes |
| `scripts/records-lint.sh` | Out of scope |
| `.factory/STATE.md` | State-manager territory |
| BC-INDEX.md, ARCH-INDEX.md, VP-INDEX.md, STORY-INDEX.md content rows | State-manager territory; only structural `## Changelog` additions in scope |

---

## Forbidden Dependencies

No new shell tools or scripts introduced. This story uses standard text editing only.

---

## Changelog

| Version | Date | Author | Summary |
|---------|------|--------|---------|
| 1.0 | 2026-07-31 | story-writer | FB105 — initial story creation. Resolves DRIFT-STORY-CHANGELOG-ABSENT-001 (67 story files lack ## Changelog sections; D-2076). Human authorized in FB105 dispatch. Additive-only sweep: adds ## Changelog section to all affected files; no content changes; no version bumps. Blocks S-MAINT-CONTENT-VERSION-GATE-001 (provides the prerequisite surface for the content-version-bump gate). 3 ACs, 3 RG tests. status: draft; behavioral_contracts: [] pending PO authorship. |
