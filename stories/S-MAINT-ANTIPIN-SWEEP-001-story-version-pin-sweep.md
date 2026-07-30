---
document_type: story
story_id: S-MAINT-ANTIPIN-SWEEP-001
title: "POL-39 Compliance Sweep: Remove Narrative Version Pins from .factory/stories/ (83 Files)"
wave: maintenance
epic_id: maintenance
priority: P2
status: draft
version: "1.1"
updated: "2026-07-30"
level: "L2"
producer: story-writer
timestamp: "2026-07-30T00:00:00Z"
tdd_mode: strict
# tdd_mode: strict — mandatory per BC-8.30.001 invariant 2.
# No Rust code is touched (crates_touched: []). Red Gate items are mechanical L11-gate
# invocations that fail before the sweep (version pins exist) and pass after (zero violations).
# tdd_mode present for schema compliance; Rust-specific TDD machinery does not apply.
subsystems: []
crates_touched: []
target_module: ".factory/stories"
capabilities: []
behavioral_contracts: []
# BC status: pending PO authorship.
# POL-39 anti_volatile_pin_versions is a governance policy, not a behavioral contract.
# This story MUST remain status: draft until a governance-integrity BC is authored.
verification_properties: []
holdout_scenarios: []
# POL-35 holdout_gate_infra_only_exemption applies: behavioral_contracts: [] (pure
# records-only sweep). holdout_scenarios: [] is compliant for this story.
depends_on:
  - S-MAINT-L11-GATE-001
  - S-MAINT-CAPREF-SWEEP-001
# depends_on justification:
#   S-MAINT-L11-GATE-001 must ship first because it provides the L11 gate that mechanically
#   validates version pins; this sweep cannot declare success without that gate operational
#   (the gate must be present before the sweep can be verified as having produced a clean state).
#   S-MAINT-CAPREF-SWEEP-001 must also ship first because: (a) after that story ships, citations
#   like `capabilities.md §CAP-NNN` are valid section-anchor forms — these citations exist
#   in the story files being swept here, and the sweep must not accidentally remove them; and
#   (b) the two stories have overlapping file scopes (both touch .factory/ files) and
#   sequential ordering prevents conflicting concurrent edits. The full dependency chain is
#   strictly serial: L11-GATE → CAPREF → this story → ANTIPIN-SWEEP-002.
blocks:
  - S-MAINT-ANTIPIN-SWEEP-002
# blocks justification:
#   S-MAINT-ANTIPIN-SWEEP-002 sweeps .factory/specs/. Once story files are clean,
#   the specs sweep can run cleanly without ambiguity about whether a remaining L11
#   hit is in the stories tier or the specs tier.
points: 8
estimated_days: 3.0
risk: MEDIUM
# Risk justification:
#   The sweep spans 83 story files with approximately 2,813 narrative version pins
#   (~2,434 story→BC pins + 364 story→ADR + 15 story→VP). The principal risk is
#   per-site adjudication errors: a raw grep hit count overstates actionable scope
#   because some hits are legitimately-scoped historical prose (correctly-scoped past-
#   tense statements that are records of prior decisions, not live navigational pins).
#   Blind de-pinning of these sites corrupts correct historical records. AC-003 and
#   EC-001 require per-site form adjudication before removal.
assumption_validations: []
risk_mitigations: []
tags:
  - pol-39
  - records-governance
  - sweep
  - stories
---

# S-MAINT-ANTIPIN-SWEEP-001: POL-39 Compliance Sweep — Narrative Version Pins in .factory/stories/ (83 Files)

## Narrative

As a records-discipline maintainer, I want all narrative version pins in `.factory/stories/`
(story bodies, Previous Story Intelligence sections, Architecture Compliance Rules sections, and
other non-exempt prose) removed and replaced with durable artifact-ID + section/symbol anchors,
so that `scripts/records-lint.sh --full-scan` exits 0 for L11 across all 83 story files, making
story records resilient to artifact version churn and compliant with POL-39
(`anti_volatile_pin_versions`, HIGH).

## Background: Measured Scope and Exemption Boundary

**Measured exposure at authoring time (2026-07-30):** approximately 2,813 narrative version pins
across 83 files under `.factory/stories/` (approximately 2,434 story→BC pins, 364 story→ADR
pins, 15 story→VP pins). These figures will drift as new stories ship and existing stories are
amended; the authoritative live count is the output of `scripts/records-lint.sh --full-scan`.

**POL-39 four-tier exception boundary (what L11 exempts and what this sweep does NOT touch):**

1. Index tier (STORY-INDEX.md, BC-INDEX.md, ARCH-INDEX.md, VP-INDEX.md) — exempt from L11
2. `## Changelog` / `## Version History` section rows — exempt from L11
3. Frontmatter `version:` fields — exempt from L11
4. Correctly-scoped historical prose — **requires per-site adjudication by this sweep**

**What "correctly-scoped historical prose" means:** A true past-tense statement about a prior
version is a record of what happened, not a live navigational pin. Example: "In the pass-70
fix-burst, BC-2.02.006 was at the version cited in the §Previous Story Intelligence of
S-WAVE-A-ARMIS-SPEC-001" — this is a record of a historical fact. It is DISTINCT from a live
citation like "this story traces to BC-2.02.006 §Postconditions" which does NOT need a version
number to be navigable.

**Worked example (the canonical adjudication precedent for this sweep):**
`S-WAVE-A-ARMIS-SPEC-001` and `S-WAVE-A-ARMIS-ACTIVITY-001` were de-pinned during the
pass-70 fix-burst: 13 version pins were removed across the two files. However, SPEC-001
deliberately **retains** a version pin in §Previous Story Intelligence because the FB86
adversarial pass adjudicated it as correctly-scoped historical prose — the section records
the specific version of a BC that existed at a past decision point. That pin is a historical
record of a prior state, not a navigational claim about the current version. This pair is
the worked example of the adjudication every site must receive.

**The critical adjudication rule (AC-003):** The sweep MUST be adjudicated **per site by
form**, never by raw grep hit count. During the pass-70 burst, a raw grep reported 3 residual
pins of which only 2 were genuine violations. In the same burst, a `RED_RATIO` grep returned
5 hits that were ALL legitimate (none were violations). Four consecutive bursts (D-2059,
D-2060, D-2063, and the pass-70 FB96 leg) had raw grep counts overstate actionable scope.
Blind compliance with raw counts corrupts correct historical records.

## Behavioral Contracts

| BC | Title | Version | Scope in This Story |
|----|-------|---------|---------------------|
| (none) | — | — | No behavioral contracts yet. See BC status comment in frontmatter. |

## Acceptance Criteria

### AC-001 — `--full-scan` exits 0 for L11 across all in-scope story files
After all per-site remediations are applied, running
`bash scripts/records-lint.sh --full-scan`
with scope restricted to `.factory/stories/` (or equivalently, running `--full-scan` across
all `.factory/` and confirming zero L11 hits attributable to story files) reports zero L11
violations for the story tier. This is the primary mechanical acceptance gate.
(traces to POL-39 `anti_volatile_pin_versions` §enforcement scope and TD-VSDD-092 §L11 arm;
verified by RG-001)

### AC-002 — Every removal replaces the pin with a durable artifact-ID + section anchor
For each version pin removed, the replacement MUST use one of: (a) a durable section heading
anchor (`BC-2.01.001 §Postconditions`), (b) a canonical symbol or function name, (c) a
decision-record label (`ADR-053 §D2`), or (d) a story/task identifier. The replacement MUST
convey the same navigational intent as the original pin: a reader following the anchor MUST
reach the same content the pin was pointing to. "Remove the pin without replacement" is only
valid when the pin provides no navigational value beyond the artifact ID alone.
(traces to POL-39 §replacement-form requirement; verified by adversarial review of removed-pin
diff, confirming replacements are durable)

### AC-003 — Sweep is adjudicated per site by form; correctly-scoped historical prose is retained
Before removing any pin, the implementer MUST read the surrounding context and determine
whether the site is: (a) a live navigational reference (pin that must be replaced), or
(b) correctly-scoped historical prose (past-tense statement about a prior version — MUST be
retained). Sites in category (b) are annotated with a comment in the PR description justifying
the retention. The worked example is S-WAVE-A-ARMIS-SPEC-001 §Previous Story Intelligence,
which retains its pin per FB86 adjudication. This AC is verified by adversarial review: the
adversary independently reads each retained pin and confirms it qualifies as correctly-scoped
historical prose.
(traces to the per-site adjudication obligation stated in §Background; verified by adversarial
review of the PR diff)

### AC-004 — No story's behavioral acceptance criteria or BC traceability assertions changed
This sweep MUST NOT alter the behavioral meaning of any story's acceptance criteria, BC
traceability statements, or architecture compliance rules. Version pins in AC bodies are
replaced with section anchors for the same artifact; the behavioral assertion is unchanged.
If removing a pin would require a semantic change to an AC body (e.g., the pin is the only
way to disambiguate which version of a BC is being referenced), STOP and route to the
orchestrator. The implementer MUST NOT self-authorize semantic changes to AC bodies.
(traces to CLAUDE.md §Source-of-Truth Precedence and §Correct Agent Routing; verified by
adversarial review of AC-body changes in the diff)

## Red Gate Tests

These are L11 gate invocations that fail before the sweep (version pins exist) and pass after.

- [ ] **RG-001** (`verify_l11_full_scan_stories_before`): Before sweep, run
  `bash scripts/records-lint.sh --full-scan` and confirm L11 reports violations in
  `.factory/stories/` files (failing state — pins exist). Record the initial L11 violation
  count for the stories tier as the baseline.

- [ ] **RG-002** (`verify_l11_full_scan_stories_after`): After sweep, run
  `bash scripts/records-lint.sh --full-scan` and confirm zero L11 violations attributable
  to story files (passing state).

**Red Gate density check (BC-5.38.001):** 2 Red Gate verification commands (RG-001 and RG-002)
anchor to 4 acceptance criteria. For records-only stories, density verification is adapted
to the L11 gate invocation paradigm. Computed at dispatch per `per-story-delivery.md §Red Gate
Density Check` and BC-5.38.002/BC-5.38.003.

## Architecture Mapping

| Component | Location | Pure/Effectful |
|-----------|----------|----------------|
| `.factory/stories/*.md` (83 files) | `.factory/stories/` | Pure (text edits to records) |
| `scripts/records-lint.sh` | project root `scripts/` | Pure (read-only verification gate) |

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | A story's §Previous Story Intelligence section contains a version pin that is correctly-scoped historical prose (like the S-WAVE-A-ARMIS-SPEC-001 retained pin) | Retain it; annotate in PR description as correctly-scoped historical prose per AC-003. Do NOT remove. |
| EC-002 | A version pin appears in a story's `## Changelog` section row | Exempt from L11 (POL-39 tier 3). Do NOT touch these rows. |
| EC-003 | A version pin appears in story frontmatter (e.g., a `depends_on:` entry accidentally carrying a version suffix) | This is unusual; evaluate whether the frontmatter field is within the frontmatter `---` block. If so, exempt (POL-39 tier 4). If outside the block, de-pin. |
| EC-004 | A version pin appears in a story's `## Behavioral Contracts` table (e.g., "BC-2.01.001 | Title | v1.5 | ...") | The table's "Version" column is a known navigational element. The appropriate fix is to remove the version from the "Version" column body and replace with a durable form or remove the column if it serves no navigational purpose beyond the artifact ID. Adjudicate per site. |
| EC-005 | A story references `BC-2.02.006` in a §Previous Story Intelligence section that was specifically written to record a historical BC state (like SPEC-001/ACTIVITY-001 pair) | Apply the FB86 adjudication precedent: if the version pin records a historically-specific BC state at a named decision point, retain it. Document the retention in the PR. |
| EC-006 | Raw grep for the L11 pattern returns a count that includes non-violations (changelog rows, frontmatter, historical prose) | Do NOT treat the raw grep count as the de-pin target. The authoritative count of actionable pins is: `--full-scan` L11 hits MINUS exempted sites. Per AC-003, adjudicate per site. |

## Work-List Methodology

**Do NOT pre-enumerate all ~2,813 pins.** The work-list is mechanically generated and will
drift as the corpus evolves. The authoritative work-list is:

```bash
bash scripts/records-lint.sh --full-scan
```

This command is the work-list. The implementer runs it at the start of each sub-burst, picks
one story file, reads it in full, applies remediations with per-site adjudication, writes it,
and re-runs `--full-scan` to confirm hit count decreases. Process continues until zero L11
hits remain for story files.

**Baseline at authoring time:** approximately 2,813 L11 violations across 83 story files. This
figure is the measured count at 2026-07-30; it will drift as stories ship between now and the
time this story is implemented. The acceptance criterion is a zero result from `--full-scan` for
story files — not reaching exactly "2,813 minus X" remaining hits.

## Token Budget Estimate

| Artifact | Estimated Tokens | Notes |
|----------|-----------------|-------|
| This story spec | ~8,000 | |
| `scripts/records-lint.sh` | ~15,000 | Read once for --full-scan invocation context |
| Story file being processed (per sub-burst) | ~12,000 | Load only one file at a time |
| **Per sub-burst ceiling** | ~35,000 | One story + this spec + lint script per sub-burst |

**Context management discipline:** Process in sub-bursts of 1–3 files at a time. Run
`--full-scan` to identify the next batch of files, load one, remediate, write, repeat. Do
NOT load the full 83-file corpus simultaneously.

## Tasks

### Phase A — Baseline (Red Gate)

- [ ] **T-A01**: Run `bash scripts/records-lint.sh --full-scan` and capture L11 hits for
  `.factory/stories/`. Record the baseline count. This satisfies RG-001.

### Phase B — Per-file remediation (83 story files)

- [ ] **T-B01..T-BNN**: For each story file with L11 hits:
  1. Run `--full-scan` to get current hit list; pick one file with violations.
  2. Read the file in full.
  3. For each L11 hit site: read surrounding context, adjudicate (live pin vs. correctly-scoped
     historical prose per AC-003).
  4. For live pins: replace with durable `artifact-ID §Section` form.
  5. For historical prose: annotate in a per-file note (carries to PR description).
  6. Write the amended file.
  7. Re-run `--full-scan`; confirm hit count for this file is now zero.

  **TD-VSDD-060 sibling-site sweep:** when removing a pin pattern from a story, grep the same
  file for the same pattern to confirm all occurrences are addressed in the same edit.

### Phase C — Verification

- [ ] **T-C01**: Run `bash scripts/records-lint.sh --full-scan`. Confirm zero L11 hits
  for `.factory/stories/` files. Any remaining hits MUST be explicitly justified as
  correctly-scoped historical prose with a PR-description annotation.

- [ ] **T-C02**: Verify L1/L7 pass for all story files touched (no changelog ordering regressions
  introduced by the sweep edits).

### Merge gate

- [ ] **MERGE-GATE-L11-STORIES**: `scripts/records-lint.sh --full-scan` exits 0 for L11
  across all `.factory/stories/` files (or all retained hits are annotated as correctly-scoped
  historical prose per AC-003).
- [ ] **MERGE-GATE-AC-BODY-CLEAN**: Adversarial review confirms no AC body's behavioral meaning
  changed (AC-004).
- [ ] **MERGE-GATE-HISTORICAL-PROSE-ANNOTATED**: All retained pins have PR-description annotations
  per AC-003.

## Previous Story Intelligence

Predecessor sweeps in the records-governance chain:
- `S-MAINT-VOLATILE-CITE-001` and `S-MAINT-VOLATILE-CITE-002` established the per-file
  sub-burst pattern for corpus-scale sweep stories. Same discipline applies here.
- **Key lesson from pass-70 / D-2059–D-2063:** Raw grep hit counts do NOT equal actionable
  pins. Four consecutive bursts inflated scope by including changelog rows, historical prose,
  and RED_RATIO strings in the count. The implementer MUST use `--full-scan` L11 output (which
  exempts changelog rows and frontmatter) as the authoritative work-list, then apply per-site
  form adjudication for the remaining historical-prose determination.
- `S-MAINT-L11-GATE-001` (predecessor): ships the L11 gate that is this story's primary
  acceptance criterion.
- `S-MAINT-CAPREF-SWEEP-001` (predecessor): restructures capabilities.md and invariants.md
  so that `§CAP-NNN` and `§DI-NNN` citations in story files resolve correctly after de-pinning.

## Architecture Compliance Rules

1. **POL-39 `anti_volatile_pin_versions`:** Live narrative version pins in story bodies are
   banned. Allowed exceptions (handled by L11 exemptions): changelog rows, frontmatter fields,
   index-tier files, correctly-scoped historical prose.

2. **POL-1 immutability:** Story metadata in STORY-INDEX.md (changelog rows, registration
   entries) is handled by state-manager, not by this sweep story. Do NOT touch STORY-INDEX.md.

3. **TD-VSDD-060 sibling-site sweep:** When amending a pattern in a story file, grep the same
   file for the same pattern before writing to confirm all occurrences are handled. Cross-file
   sweep within the stories tier is handled by the per-file `--full-scan` iteration.

4. **Source-of-Truth Precedence (CLAUDE.md):** If removing a version pin from an AC body would
   change its behavioral meaning, STOP and route to orchestrator. Story-writer and implementer
   do NOT self-authorize semantic changes to BCs or ACs.

5. **POL-29 TD-VSDD-097 three-dimension sweep:**
   (a) Sibling pair: `.factory/stories/` files sometimes come in named pairs (e.g.,
       SPEC-001 / ACTIVITY-001). When de-pinning one file of a pair, always check its sibling.
   (b) Downstream copy target: if a story body section is copied verbatim into a downstream
       artifact, sweep the copy in the same sub-burst.
   (c) Mandate anchor: `MUST` statements in ACs trace to RG-001/RG-002 as the verification anchor.

## Library and Framework Requirements

| Library/Tool | Version/Source | Purpose |
|-------------|---------------|---------|
| `scripts/records-lint.sh` | Project-local | Primary work-list generator and acceptance gate |
| `bash` / `grep` | System | Per-file spot verification |

No library changes.

## File Structure Requirements

### Files to MODIFY

All story files under `.factory/stories/` that `scripts/records-lint.sh --full-scan` identifies
as having L11 hits. Specific list determined by `--full-scan` output at implementation time.

### Files NOT to modify

| File | Reason |
|------|--------|
| `STORY-INDEX.md` | State-manager owns STORY-INDEX.md; index-tier files are L11-exempt |
| `BC-INDEX.md`, `ARCH-INDEX.md`, `VP-INDEX.md` | Index-tier; handled by L10 |
| `.factory/specs/**` | Covered by S-MAINT-ANTIPIN-SWEEP-002 |
| Any `crates/**` file | No code changes |

## Forbidden Dependencies

No new dependencies. No changes to Cargo.toml files.

## Dependency Graph Edges

```
S-MAINT-ANTIPIN-SWEEP-001 (this story)
  depends_on:
    ← S-MAINT-L11-GATE-001   (L11 gate must be deployed)
    ← S-MAINT-CAPREF-SWEEP-001 (source file structure must be finalized)
  blocks:
    → S-MAINT-ANTIPIN-SWEEP-002  (stories sweep must be clean before specs sweep)
```

## Version History

| Version | Date | Change Summary |
|---------|------|----------------|
| 1.1 | 2026-07-30 | FB101 story leg — close F-WASE-P71-MED-007 (partial): add `S-MAINT-L11-GATE-001` to `depends_on` frontmatter array. The §Dependency Graph Edges body section already correctly listed L11-GATE-001 as a prerequisite; the frontmatter was stale. L11-GATE-001 must ship before this sweep because it provides the gate that mechanically validates the sweep's success criterion. Dependency justification comment updated to name both predecessors explicitly. POL-29 9a: ANTIPIN-SWEEP-002 twin updated in same burst (same missing `depends_on` entry). 9b: no downstream copy target affected. 9c: no new MUSTs introduced. |
| 1.0 | 2026-07-30 | Initial story creation. POL-39 compliance sweep for narrative version pins in .factory/stories/ tier, approximately 2,813 pins across 83 files at authoring time. Per-site adjudication required; worked example is the S-WAVE-A-ARMIS-SPEC-001 retained pin (FB86 correctly-scoped historical prose adjudication). Acceptance gate is scripts/records-lint.sh --full-scan L11 exit 0 for story files. Context management discipline: one file per sub-burst, --full-scan as work-list, no corpus-level pre-enumeration. |
