---
document_type: story
story_id: S-MAINT-ANTIPIN-SWEEP-002
title: "POL-39 Compliance Sweep: Remove Narrative Version Pins from .factory/specs/ (136 Files) and Remove Superseded FB93 Anchor Blockquotes"
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
# No Rust code is touched (crates_touched: []). Red Gate items are L11-gate invocations
# that fail before the sweep (version pins exist) and pass after. tdd_mode present for
# schema compliance; Rust-specific TDD machinery does not apply.
subsystems: []
crates_touched: []
target_module: ".factory/specs"
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
  - S-MAINT-ANTIPIN-SWEEP-001
# depends_on justification:
#   S-MAINT-L11-GATE-001 must ship first because it provides the L11 gate that mechanically
#   validates version pins; this sweep cannot declare success without that gate operational
#   (the gate must be present before the sweep can be verified as having produced a clean state).
#   S-MAINT-ANTIPIN-SWEEP-001 must also complete first because: (a) the stories and specs
#   tiers have overlapping citation graphs — some spec files cite story files and vice
#   versa; running them concurrently creates conflicting edits; (b) the strictly serial
#   chain (L11-GATE → CAPREF → SWEEP-001 → SWEEP-002) ensures each sweep verifies
#   clean delta against the prior tier's completed baseline; (c) the SWEEP-002 implementer
#   can confirm that L11 hits remaining after SWEEP-001 are exclusively in the specs tier.
blocks: []
points: 8
estimated_days: 3.0
risk: MEDIUM
# Risk justification:
#   Larger corpus (136 files, approximately 2,866 pins) than SWEEP-001. Same per-site
#   adjudication obligation. Additional complexity: the FB93 inline anchor-form blockquote
#   removal in BC-2.02.006 and BC-2.02.014 (AC-005) requires confirming the blockquote
#   is superseded by POL-39 and does not carry active behavioral content.
assumption_validations: []
risk_mitigations: []
tags:
  - pol-39
  - records-governance
  - sweep
  - specs
  - bc
  - adr
  - vp
---

# S-MAINT-ANTIPIN-SWEEP-002: POL-39 Compliance Sweep — Narrative Version Pins in .factory/specs/ (136 Files) and Superseded FB93 Anchor Blockquotes

## Narrative

As a records-discipline maintainer, I want all narrative version pins in `.factory/specs/`
(BC bodies, ADR bodies, VP bodies, prd-supplements, error-taxonomy, domain-spec files) removed
and replaced with durable artifact-ID + section/symbol anchors — and the superseded FB93
inline anchor-form blockquotes removed from BC-2.02.006 and BC-2.02.014 — so that
`scripts/records-lint.sh --full-scan` exits 0 for L11 across all 136 spec files, making spec
records compliant with POL-39 (`anti_volatile_pin_versions`, HIGH).

## Background: Measured Scope

**Measured exposure at authoring time (2026-07-30):** approximately 2,866 narrative version pins
across 136 files under `.factory/specs/`. Breakdown by spec class: BC bodies (~1,900), ADR bodies
(~600), VP bodies (~150), prd-supplements including error-taxonomy (~150), domain-spec (~66).
These figures drift; the authoritative live count is `scripts/records-lint.sh --full-scan`.

**Exemption boundary** (same as SWEEP-001, enforced by L11):
1. Index tier (BC-INDEX, ARCH-INDEX, VP-INDEX) — exempt (L10 governs)
2. `## Changelog` / `## Version History` section rows — exempt
3. Frontmatter `version:` fields — exempt
4. Correctly-scoped historical prose — requires per-site adjudication

**Additional in-scope task — FB93 inline anchor-form blockquote removal:** BC-2.02.006 and
BC-2.02.014 contain inline anchor-form blockquotes introduced by FB93 as an asymmetric
convention for pinning specific BC versions at annotation sites. POL-39 supersedes this
convention: the FB93 blockquote form is not one of POL-39's four exempt tiers and was never
ratified as a corpus-wide pattern (it was asymmetric — only two BCs carried it). AC-005
requires removing these blockquotes from both files. The annotations should be replaced with
durable section-anchor citations per POL-39's replacement-form requirement.

## Behavioral Contracts

| BC | Title | Version | Scope in This Story |
|----|-------|---------|---------------------|
| (none) | — | — | No behavioral contracts yet. See BC status comment in frontmatter. |

## Acceptance Criteria

### AC-001 — `--full-scan` exits 0 for L11 across all in-scope spec files
After all per-site remediations are applied, running
`bash scripts/records-lint.sh --full-scan`
with scope restricted to `.factory/specs/` (or equivalently, running `--full-scan` across
all `.factory/` and confirming zero L11 hits attributable to spec files) reports zero L11
violations for the spec tier. This is the primary mechanical acceptance gate.
(traces to POL-39 `anti_volatile_pin_versions` §enforcement scope and TD-VSDD-092 §L11 arm;
verified by RG-001)

### AC-002 — Every removal replaces the pin with a durable artifact-ID + section anchor
Same replacement obligation as S-MAINT-ANTIPIN-SWEEP-001 AC-002: each removed version pin
MUST be replaced with a durable form (section heading anchor, canonical symbol name, decision-
record label, or story/task identifier) that conveys the same navigational intent.
"Remove the pin without replacement" is only valid when the pin provides no navigational value
beyond the artifact ID alone.
(traces to POL-39 §replacement-form requirement; verified by adversarial review of diff)

### AC-003 — Sweep is adjudicated per site by form; correctly-scoped historical prose is retained
Identical obligation to S-MAINT-ANTIPIN-SWEEP-001 AC-003 (per-site form adjudication, not
raw grep count compliance). In the specs tier, correctly-scoped historical prose appears most
commonly in BC and ADR §Changelog rows (exempt by L11) and in ADR §Supersession rationale
sections (where past-version references are records of a specific decision). The worked example
from SWEEP-001 (ARMIS-SPEC-001 retained pin) applies equally here: a past-tense statement about
which BC version was current at a named decision point is a record, not a live navigational pin.
(traces to per-site adjudication obligation; verified by adversarial review)

### AC-004 — No BC, ADR, or VP behavioral meaning changed
This sweep MUST NOT alter the behavioral meaning of any BC postcondition, precondition,
invariant clause, ADR decision body, or VP proof property. Version pins in spec bodies are
replaced with section anchors for the same artifact; the behavioral assertion is unchanged.
If removing a pin requires a semantic change (e.g., the version number disambiguates between
two BC postcondition sets that changed between versions), STOP and route to the orchestrator.
The implementer MUST NOT self-authorize semantic changes to spec content.
(traces to CLAUDE.md §Source-of-Truth Precedence; verified by adversarial review of spec diffs)

### AC-005 — Superseded FB93 inline anchor-form blockquotes removed from BC-2.02.006 and BC-2.02.014
BC-2.02.006 and BC-2.02.014 carry inline anchor-form blockquotes introduced by FB93 as an
asymmetric convention for pinning specific BC versions at annotation sites. POL-39 supersedes
this convention — it is not one of POL-39's four exempt tiers and was never ratified corpus-wide.
Both blockquotes MUST be removed from the respective BC files. For each removal:
(a) Confirm the blockquote is purely a navigational annotation (no behavioral content embedded
in the blockquote text itself);
(b) Replace the annotation's navigational intent with a durable `§Section-anchor` form;
(c) Verify via adversarial review that the BC's behavioral postconditions are unchanged after
the blockquote removal.
If the blockquote contains behavioral content (not just a version pin annotation), STOP and
route to the product-owner for specialist review.
(traces to the FB93 supersession stated in §Background; verified by adversarial review of
the two BC diffs for AC-004 compliance and by confirming blockquote-free state)

## Red Gate Tests

- [ ] **RG-001** (`verify_l11_full_scan_specs_before`): Before sweep, run
  `bash scripts/records-lint.sh --full-scan` and confirm L11 reports violations in
  `.factory/specs/` files (failing state). Record the initial L11 violation count for
  the specs tier.

- [ ] **RG-002** (`verify_l11_full_scan_specs_after`): After sweep, run
  `bash scripts/records-lint.sh --full-scan` and confirm zero L11 violations attributable
  to spec files (passing state).

**Red Gate density check (BC-5.38.001):** 2 Red Gate verification commands (RG-001 and RG-002)
anchor to 5 acceptance criteria. For records-only stories, density verification is adapted to
the L11 gate invocation paradigm. Computed at dispatch per `per-story-delivery.md §Red Gate
Density Check` and BC-5.38.002/BC-5.38.003.

## Architecture Mapping

| Component | Location | Pure/Effectful |
|-----------|----------|----------------|
| `.factory/specs/behavioral-contracts/BC-*.md` (BC bodies) | `.factory/specs/behavioral-contracts/` | Pure (text edits) |
| `.factory/specs/architecture/decisions/ADR-*.md` (ADR bodies) | `.factory/specs/architecture/decisions/` | Pure (text edits) |
| `.factory/specs/verification-properties/VP-*.md` (VP bodies) | `.factory/specs/verification-properties/` | Pure (text edits) |
| `.factory/specs/prd-supplements/**` | `.factory/specs/prd-supplements/` | Pure (text edits) |
| `.factory/specs/domain-spec/**` (non-index) | `.factory/specs/domain-spec/` | Pure (text edits) |
| `BC-2.02.006-*.md` | `.factory/specs/behavioral-contracts/` | Pure (text edit + FB93 blockquote removal) |
| `BC-2.02.014-*.md` | `.factory/specs/behavioral-contracts/` | Pure (text edit + FB93 blockquote removal) |
| `scripts/records-lint.sh` | project root | Pure (read-only verification gate) |

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | A BC §Changelog row contains a version pin of the form `BC-N.NN.NNN v(digit).(digit)` | Exempt from L11 (POL-39 tier 3 — changelog section). Do NOT touch. |
| EC-002 | An ADR §Supersession rationale contains a past-tense version reference ("ADR-026 was at the version in effect when..." — historical prose) | Adjudicate per site: if past-tense and records a decision, it is correctly-scoped historical prose (retain). If a live navigational claim ("this ADR supersedes..."), de-pin. |
| EC-003 | The FB93 blockquote in BC-2.02.006 or BC-2.02.014 contains a behavioral assertion beyond the version pin | Stop; route to product-owner. The blockquote may not be removed without behavioral-content preservation. |
| EC-004 | A VP §Proof Harness Skeleton section contains a version pin in a comment | Evaluate whether the comment is navigational or behavioral. VP harness comments are typically behavioral (citing the BC version whose contract the proof establishes). Adjudicate per site; these are likely correctly-scoped historical prose. |
| EC-005 | An ADR §Decisions section cites a BC with a version pin to indicate the BC's state at decision time | This is a common correctly-scoped historical prose form in ADRs. Retain per the per-site adjudication rule; annotate in PR. |
| EC-006 | L11 reports a violation in a flat architecture section doc under `.factory/specs/architecture/` (non-ADR) | In-scope for this sweep. Apply the same per-site adjudication as BC/ADR files. |

## Work-List Methodology

Same discipline as S-MAINT-ANTIPIN-SWEEP-001: `--full-scan` is the authoritative work-list.
Do NOT pre-enumerate all ~2,866 pins. Process in sub-bursts of 1–2 files at a time.

The FB93 blockquote removal in BC-2.02.006 and BC-2.02.014 MUST be done as part of the main
sweep (not as a separate story), since both files will already be opened for version-pin
remediation. The blockquote removal is an additional in-scope task per AC-005.

## Token Budget Estimate

| Artifact | Estimated Tokens | Notes |
|----------|-----------------|-------|
| This story spec | ~8,000 | |
| `scripts/records-lint.sh` | ~15,000 | Read once |
| Spec file being processed (per sub-burst) | ~15,000 | BCs and ADRs are larger than story files |
| **Per sub-burst ceiling** | ~38,000 | One spec file + story + lint script; do not load multiple spec files |

**Context management discipline:** BC bodies and ADR bodies are the largest files in this tier.
Load one file per sub-burst. For BC-2.02.006 and BC-2.02.014 (AC-005 FB93 blockquotes), read
the full file once, perform both the version-pin sweep and the blockquote removal in the same
edit, then write once. Do not reopen the file for a second pass.

## Tasks

### Phase A — Baseline (Red Gate)

- [ ] **T-A01**: Run `bash scripts/records-lint.sh --full-scan` and capture L11 hits for
  `.factory/specs/`. Record the baseline count. This satisfies RG-001. Also note that the
  stories tier should now be clean (after SWEEP-001), so all remaining hits are specs-tier.

### Phase B — Per-file remediation: BC bodies

- [ ] **T-B01..T-BNN**: For each BC file with L11 hits:
  1. `--full-scan` to identify the current hit list; pick one BC.
  2. Read the full BC file.
  3. For each L11 hit site: adjudicate (live pin vs. correctly-scoped historical prose).
  4. For live pins: replace with durable `BC-S.SS.NNN §Section` form.
  5. Write the amended BC. Re-run `--full-scan` to confirm zero L11 hits for this file.
  6. For BC-2.02.006 and BC-2.02.014: additionally remove the FB93 inline anchor-form
     blockquote (AC-005) in the same write operation.

### Phase C — Per-file remediation: ADR bodies

- [ ] **T-C01..T-CNN**: Same pattern for ADR files. ADR prose is architect-owned; confirm no
  ADR decision-body behavioral meaning changes (AC-004). ADR §Supersession rationale sections
  often contain correctly-scoped historical prose (EC-002) — apply per-site adjudication.

### Phase D — Per-file remediation: VP bodies, prd-supplements, domain-spec

- [ ] **T-D01..T-DNN**: VP files, prd-supplements (including error-taxonomy.md), and any
  domain-spec files with L11 hits. For VP §Proof Harness Skeleton comments, apply EC-004
  adjudication. For error-taxonomy.md, version pins are likely live navigational references
  (the taxonomy is the current authoritative form — de-pin and use section anchors).

### Phase E — Verification

- [ ] **T-E01**: Run `bash scripts/records-lint.sh --full-scan`. Confirm zero L11 hits for
  `.factory/specs/` files. Any remaining hits must be annotated as correctly-scoped historical
  prose per AC-003.
- [ ] **T-E02**: Confirm BC-2.02.006 and BC-2.02.014 contain no FB93 blockquotes (AC-005).
- [ ] **T-E03**: Run L1/L7 on all modified spec files to confirm no changelog ordering
  regressions were introduced by the sweep edits.

### Merge gate

- [ ] **MERGE-GATE-L11-SPECS**: `scripts/records-lint.sh --full-scan` exits 0 for L11
  across all `.factory/specs/` files.
- [ ] **MERGE-GATE-FB93-REMOVED**: BC-2.02.006 and BC-2.02.014 contain no FB93 inline
  anchor-form blockquotes.
- [ ] **MERGE-GATE-SEMANTIC-CLEAN**: Adversarial review confirms no BC, ADR, or VP behavioral
  meaning changed.
- [ ] **MERGE-GATE-HISTORICAL-PROSE-ANNOTATED**: All retained pins have PR-description
  annotations per AC-003.

## Previous Story Intelligence

Predecessors in the POL-39 sweep chain:
- `S-MAINT-L11-GATE-001`: deployed the L11 gate that is this story's primary acceptance criterion.
- `S-MAINT-CAPREF-SWEEP-001`: restructured capabilities.md and invariants.md; domain-spec files
  with CAP/DI citations now have valid section anchors.
- `S-MAINT-ANTIPIN-SWEEP-001`: cleaned the stories tier; the specs tier is the final tier.

**Key lessons from SWEEP-001 (apply here):**
- Do NOT treat raw grep hit counts as the de-pin target. Use `--full-scan` L11 output.
- Apply per-site form adjudication for every hit. BC and ADR §Changelog rows and §Supersession
  rationale sections are the most common correctly-scoped historical prose sites in this tier.
- Process in sub-bursts (one file at a time). Do not load the full 136-file corpus.

**FB93 blockquote context:** The FB93 convention was introduced as an ad hoc annotation pattern
for two specific BCs. It was never ratified as a corpus-wide convention, never added to the
spec-authoring guidelines, and POL-39 now supersedes it. The removal in AC-005 is a one-time
cleanup; no BC should carry this form going forward. If a similar annotation need arises post-
POL-39, the correct form is a prose sentence with a durable section-anchor citation.

## Architecture Compliance Rules

1. **POL-39 `anti_volatile_pin_versions`:** Same as SWEEP-001 §Architecture Compliance Rules
   rule 1. Live narrative version pins in spec bodies are banned. Correctly-scoped historical
   prose is retained with per-site adjudication and PR-description annotation.

2. **POL-1 immutability:** Do NOT modify BC-INDEX, ARCH-INDEX, or VP-INDEX rows. Those are
   index-tier, exempt from L11, and owned by state-manager.

3. **Spec wins over code (CLAUDE.md §Source-of-Truth Precedence):** The spec is the source of
   truth. If removing a pin requires a semantic change, route to the owning specialist rather
   than proceeding independently.

4. **TD-VSDD-060 sibling-site sweep:** When amending a pattern in a spec file, grep the same
   file for the same pattern before writing to confirm all occurrences are handled in the
   same edit.

5. **TD-VSDD-097 three-dimension sweep (POL-29 9c):**
   (a) Sibling pair: BC-2.02.006 and BC-2.02.014 both carry FB93 blockquotes — they MUST be
       swept in the same commit (not separately). Per the amendment precedent: sweeping one BC
       of a pair without sweeping its twin is the named failure mode. Anchor: this story's
       AC-005 covers both files.
   (b) Downstream copy target: if the FB93 blockquote text was copied verbatim into any OTHER
       spec artifact, sweep that copy in the same burst as the BC-2.02.006/014 edits.
   (c) Mandate anchor: AC-005's `MUST` statements trace to RG-001/RG-002 as the mechanical
       gate. The FB93 blockquote removal verifies clean via L11 (the blockquote, once removed,
       cannot be re-introduced without triggering L11 if it contains a version pin).

## Library and Framework Requirements

| Library/Tool | Version/Source | Purpose |
|-------------|---------------|---------|
| `scripts/records-lint.sh` | Project-local | Primary work-list generator and acceptance gate |
| `bash` / `grep` | System | Per-file spot verification |

No library changes.

## File Structure Requirements

### Files to MODIFY

All spec files under `.factory/specs/` that `scripts/records-lint.sh --full-scan` identifies
as having L11 hits (approximately 136 files at authoring time). Plus:
- `BC-2.02.006-*.md` — also removes FB93 blockquote (AC-005)
- `BC-2.02.014-*.md` — also removes FB93 blockquote (AC-005)

### Files NOT to modify

| File | Reason |
|------|--------|
| `BC-INDEX.md`, `ARCH-INDEX.md`, `VP-INDEX.md` | Index-tier; exempt from L11; owned by state-manager |
| `STORY-INDEX.md` | Index-tier; exempt from L11 |
| `.factory/stories/**` | Covered by S-MAINT-ANTIPIN-SWEEP-001 (predecessor) |
| Any `crates/**` file | No code changes |

## Forbidden Dependencies

No new dependencies. No changes to Cargo.toml files.

## Dependency Graph Edges

```
S-MAINT-ANTIPIN-SWEEP-002 (this story)
  depends_on:
    ← S-MAINT-L11-GATE-001         (L11 gate deployed)
    ← S-MAINT-CAPREF-SWEEP-001     (domain-spec source file structure finalized)
    ← S-MAINT-ANTIPIN-SWEEP-001    (stories tier clean; specs tier is the final tier)
  blocks: []
```

## Version History

| Version | Date | Change Summary |
|---------|------|----------------|
| 1.1 | 2026-07-30 | FB101 story leg — close F-WASE-P71-MED-007 (partial): add `S-MAINT-L11-GATE-001` to `depends_on` frontmatter array. The §Dependency Graph Edges body section already correctly listed L11-GATE-001 as a prerequisite; the frontmatter was stale. L11-GATE-001 must ship before this sweep because it provides the gate that mechanically validates the sweep's success criterion. Dependency justification comment updated to name both predecessors explicitly. POL-29 9a: ANTIPIN-SWEEP-001 twin updated in same burst (same missing `depends_on` entry). 9b: no downstream copy target affected. 9c: no new MUSTs introduced. |
| 1.0 | 2026-07-30 | Initial story creation. POL-39 compliance sweep for narrative version pins in .factory/specs/ tier, approximately 2,866 pins across 136 files at authoring time. Includes AC-005: removal of superseded FB93 inline anchor-form blockquotes from BC-2.02.006 and BC-2.02.014 (POL-39 supersedes the FB93 asymmetric convention; blockquotes are not one of POL-39's four exempt tiers). Same per-site adjudication discipline as SWEEP-001. TD-VSDD-097 three-dimension sweep explicitly covers the BC-2.02.006/014 sibling pair to prevent the named failure mode of sweeping one BC of a pair without sweeping its twin. |
