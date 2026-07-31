---
document_type: story
story_id: S-MAINT-POL29-GREP-EVIDENCE-001
title: "POL-29 Dimension Discharge Grep-Evidence Requirement — TD-VSDD-097 Amendment and SAP-4 Probe"
wave: maintenance
epic_id: maintenance
priority: P1
status: draft
version: "1.1"
updated: "2026-07-31"
level: "L2"
producer: story-writer
timestamp: "2026-07-31T00:00:00Z"
tdd_mode: strict
# tdd_mode: strict — this story delivers a CLAUDE.md amendment (TD-VSDD-097 extension +
# new SAP-4 adversary probe) and a validation script (scripts/validate-pol29-discharge.sh).
# Red Gate tests are self-probe cases for the validation script that FAIL before the script
# exists (no function → 0 exit on every input → false-green) and PASS after.
# Standard Rust todo!() discipline does not apply (no Rust crates touched).
subsystems: []
crates_touched: []
target_module: "scripts/validate-pol29-discharge.sh, CLAUDE.md"
capabilities: []
behavioral_contracts: []
# BC status: pending PO authorship.
# POL-29 discharge quality is a governance discipline, not a behavioral contract.
# This story MUST remain status: draft until a governance-integrity BC is authored.
verification_properties: []
holdout_scenarios: []
# POL-35 holdout_gate_infra_only_exemption applies: behavioral_contracts: [] (pure
# governance/process scope). holdout_scenarios: [] is compliant for this story.
depends_on: []
blocks: []
points: 5
estimated_days: 1.5
risk: LOW
# Risk justification:
#   Deliverables are a bash validation script + CLAUDE.md text amendment. No production
#   code touched. Primary risk is over-prescribing the grep format, which causes SAP-4
#   to flag legitimate discharge reports that use a different (valid) format. EC-001
#   addresses this by requiring content equivalence, not exact format matching.
assumption_validations: []
risk_mitigations: []
tags:
  - toolchain
  - governance-gate
  - pol-29
  - td-vsdd-097
  - process-gap
---

# S-MAINT-POL29-GREP-EVIDENCE-001: POL-29 Dimension Discharge Grep-Evidence Requirement

## Origin

**Process-gap finding:** Lesson 114 (D-2073, 2026-07-30) and Lesson 116 (D-2075, 2026-07-31).
TD-VSDD-097 codification threshold met at 3 independent instances in a single adversary pass,
plus two consecutive fix-burst self-certification failures on the same dimension class.

Lesson 114 documents three independent instances of POL-29 dimension-9a failure in pass-72b
where fixing agents stopped exactly at the dispatch perimeter and reported the dimension as
discharged. Lesson 116 documents the structural failure mechanism: a fixer's attention
concentrates on named dispatch sites; un-named sites are processed in the background, and the
fixer reports a clean dimension verdict without running a discriminating grep.

**Critical finding that motivated the specific mechanism in this story (from Lesson 116):**
"Three consecutive fix-burst legs this session self-reported a POL-29 dimension verdict that
independent verification contradicted: an architect reported 9c clean while the same edit
introduced four unanchored deferrals; a story-writer reported 9a 'not applicable' while a
residual site remained in a file it had just edited; a state-manager reported a closing leg
complete with gates green while content had been edited with versions left unbumped. In every
case the *work* was substantially sound and the *attestation* was not."

---

## Narrative

As an orchestrator validating POL-29 dimension discharge quality, I want:
(1) CLAUDE.md §TD-VSDD-097 amended to require that every POL-29 dimension discharge report
    includes the exact grep command run and the per-file hit counts — not a verdict word —
    and that the grep scope covers ALL files edited by the burst, not only the dispatched files;
(2) a new adversary standing probe SAP-4 in CLAUDE.md that checks discharge reports for grep
    evidence and flags verdict-word-only reports as P1 findings; and
(3) a lightweight validation script `scripts/validate-pol29-discharge.sh` with a `--self-probe`
    mode that serves as the executable test vehicle for this story's Red Gate tests;
so that the "self-certification is not authoritative" principle (TD-VSDD-059) is extended from
content correctness to dimension attestation quality, and future adversary passes have a
standing probe that prevents the verdict-word discharge pattern from recurring silently.

---

## Background

TD-VSDD-097 defines the three-dimension POL-29 sweep checklist. Each dimension requires a
"discharge": evidence that the sweep was performed and the result was clean (or the non-clean
result was justified). The current text of TD-VSDD-097 describes what each dimension checks
but does NOT specify the form of evidence required. This gap is the structural root cause.

**The discharge failure modes documented this session:**
- Dimension 9a discharge: "sibling swept — string not present in twin" — this phrasing confirms
  the failure condition, not the clean condition. The correct discharge says "twin carries
  `anchor_stories: [...]` (verified from frontmatter)" or "twin carries VP-161 in
  `verification_properties:` (verified from story body)."
- Dimension 9b discharge: "no downstream copy target" — a verdict, not a reproducible grep.
  The correct discharge says "searched `grep -r 'section text' .factory/specs/` — 0 hits."
- Dimension 9c discharge: "not applicable" — this form is indistinguishable from a skipped
  sweep. The correct discharge says "searched `.factory/stories/*.md` frontmatter for
  `depends_on\|blocks` referencing this story's scope: `grep -r 'wave-5' .factory/specs/architecture/decisions/` — N hits at [file:section]."

**Scope of the sweep (critical — from Lesson 116):** The grep scope MUST cover ALL files the
burst edited, not only the files named in the dispatch. Fixer self-certification failures
occurred in files the fixer edited while fixing the named dispatch files. The sweep boundary
is: "all files I touched or read-with-intent-to-modify," not "all files in the dispatch prompt."

---

## Behavioral Contracts

| BC | Title | Version | Scope in This Story |
|----|-------|---------|---------------------|
| (none) | — | — | No behavioral contracts yet. See BC status comment in frontmatter. Story MUST remain `status: draft` until a governance-integrity BC is authored. This MUST is anchored to: AC-001 governs the mechanism; BC authorship is a human gate. |

---

## Acceptance Criteria

### AC-001 — CLAUDE.md §TD-VSDD-097 amended to require grep evidence for each dimension discharge
CLAUDE.md §TD-VSDD-097 is amended (in the three-dimension checklist section) to add an explicit
requirement for each of dimensions 9a, 9b, and 9c:
- Fixers MUST include the exact grep command run (or equivalent structured query) and the
  per-file hit counts in their dimension discharge report. A verdict word alone ("clean",
  "not applicable", "n/a", "none found") without supporting grep output is insufficient.
- The grep scope MUST cover all files the burst edited — not only the files named in the
  dispatch prompt. A fixer who edits file A while fixing file B must include file A in the
  grep scope.
- For dimension 9a specifically: the discharge report MUST name the twin artifact AND describe
  its current state (e.g., "twin carries `anchor_stories: [S-NNN]`" or "twin has no
  `anchor_stories:` key — gap confirmed"). Reporting only "string absent in twin" confirms
  the failure condition; it does NOT confirm the clean condition.
The amendment MUST be self-compliant: no volatile line cites or version pins in the amended text.
(verified by RG-001 and RG-004)

### AC-002 — SAP-4 adversary standing probe added to CLAUDE.md
CLAUDE.md gains a new adversary standing probe SAP-4 alongside SAP-1, SAP-2, SAP-3:

```
SAP-4 — Adversary standing probe: POL-29 dimension discharge quality

For EVERY adversarial pass on any burst that claims to have discharged one or more
POL-29 dimensions (9a, 9b, 9c):

1. For each dimension discharge claim, verify the report contains:
   (a) The exact grep command run, or an equivalent structured query specifying
       the pattern searched and the files/directories scanned.
   (b) Per-file output: for each file in the grep scope, the hit count or
       explicit "0 hits."
   (c) For dimension 9a: the twin artifact is NAMED and its CURRENT STATE is
       described — not just "string absent."

2. A discharge report that contains ONLY a verdict word for any dimension
   (e.g., "clean", "not applicable", "n/a", "none found") → P1 FINDING.

3. A discharge report whose grep scope does not cover all files edited by the
   burst (not just dispatched files) → P2 FINDING.

4. The orchestrator independently verifies by running the same grep; if the
   results differ from the reported output → P1 FINDING.
```

(verified by RG-002 and RG-003)

### AC-003 — Validation script `scripts/validate-pol29-discharge.sh` with `--self-probe`
A new script `scripts/validate-pol29-discharge.sh` is created that:
- Accepts a burst report text as input (via stdin or a file argument)
- Scans the report for POL-29 dimension discharge claims (recognizing the patterns "dimension 9a",
  "dimension 9b", "dimension 9c", "(9a)", "(9b)", "(9c)" or equivalent markers from the
  TD-VSDD-097 section headings)
- For each discharge claim found, checks whether the claim contains a grep command or equivalent
  structured query evidence
- Reports "Found N dimension discharge claims; M lack grep evidence" where N and M are
  runtime-computed integers
- If N = 0: exits 1 with "No POL-29 dimension discharge claims detected in input — cannot
  validate" (positive-coverage requirement: a zero-claim input is structurally ambiguous and
  must not silently pass)
- If M > 0: exits 1 with specific findings
- If N > 0 and M = 0: exits 0
- Has a `--self-probe` mode that runs synthetic test cases (see RG items)
(verified by RG-002 and RG-003)

### AC-004 — SESSION-HANDOFF.md §Standing Orchestrator Rules amended
SESSION-HANDOFF.md §Standing Orchestrator Rules is amended (in the same PR) to include:
"Standing Rule: POL-29 dimension discharge reports MUST include grep command + per-file hit
counts, not verdict words only. For dimension 9a: twin artifact named AND current state
described. Scope: all files edited by burst. Reference: TD-VSDD-097 §three-dimension checklist
and SAP-4."
The amendment MUST cite this story (S-MAINT-POL29-GREP-EVIDENCE-001) as the source.
(verified by RG-001)

### AC-005 — `--spec-check` mode: present-tense corpus state claim detection in spec artifacts
`scripts/validate-pol29-discharge.sh` gains a `--spec-check <file>` mode that scans a spec
artifact file (story or ADR) for sentences meeting all three conditions simultaneously:
(a) names a specific story file ID matching `S-[A-Z0-9][A-Z0-9.-]*`;
(b) contains a present-tense state verb from the set: "has", "still has", "remain",
    "remains", "contains", "is open", "is not found"; AND
(c) has no backtick-delimited grep command in the same paragraph (within 5 lines up or down
    from the matching sentence).

When all three conditions match, the mode emits a P2 finding:
`"SPEC-CORPUS-CLAIM: ungrounded present-tense state claim for [story-id] in paragraph
starting '[first 60 chars]' — add (verified: grep -n '<pattern>' <path> → N hits) or
recast as past tense."`

Findings from this mode are **records-tier P2**: they route through the records-only
micro-burst path (TD-VSDD-096) rather than full cascade ceremony. This keeps the gate
cheap to satisfy and reduces the incentive to disable it.

The mode MUST exit 1 if the target file does not exist or cannot be read (prevents
silent non-scan). Past-tense constructions (e.g., "was corrected", "had been") and
general-narrative sentences that name a story ID without making a state claim about it
are explicitly exempt — see RG-006 for the must-NOT-flag fixtures that enforce this.
(verified by RG-005 and RG-006)

---

## Red Gate Tests

All 4 RG items are test cases that FAIL before the validation script and CLAUDE.md amendments
exist and PASS after. RG-001/RG-004 validate the CLAUDE.md amendment (non-regression of
compliance rules); RG-002/RG-003 validate the validation script behavior.

- [ ] **RG-001** (`test_pol29_verdict_word_only_fails`): self-probe case — a synthetic burst
  report containing "dimension 9a: clean" (verdict word only, no grep) is fed to
  `scripts/validate-pol29-discharge.sh`; script exits 1 with "lacks grep evidence" finding.
  FAILS before the script exists (no script → implicit pass); PASSES after implementation.

- [ ] **RG-002** (`test_pol29_grep_evidence_passes`): self-probe case — a synthetic burst
  report containing "dimension 9a: ran `grep -r anchor_stories .factory/specs/architecture/decisions/ADR-053.md` — 0 hits; twin ADR-054 carries `anchor_stories: []` (verified from frontmatter)" passes
  the script without error. FAILS before the script exists; PASSES after implementation.

- [ ] **RG-003** (`test_pol29_zero_claims_exits_nonzero`): self-probe case — a burst report
  with no POL-29 dimension discharge language (e.g., a report about a code change with no
  spec sweep) causes the script to emit "No POL-29 dimension discharge claims detected" and
  exit 1 (positive-coverage requirement). FAILS before implementation; PASSES after.

- [ ] **RG-004** (`test_pol29_claude_md_amendment_self_compliant`): validation case — after
  the CLAUDE.md amendment is authored, `scripts/records-lint.sh --full-scan` on CLAUDE.md
  exits 0 (the amended TD-VSDD-097 text does not itself contain volatile line cites or
  version pins). FAILS if the amendment introduces a violation; PASSES if clean.

- [ ] **RG-005** (`test_spec_check_present_tense_fires`): `--spec-check` mode, must-flag
  case. A synthetic spec fragment fed as the target file:
  `"Two open instances remain: S-DEMO-X-001 and S-QUERY-Y-001 both defer ADR-033 T2 to wave-6."`
  (present-tense verb "remain", two story IDs named, no backtick grep cite in paragraph)
  causes `--spec-check` to emit a P2 finding for both story IDs and exit 1. FAILS before
  the `--spec-check` mode is implemented; PASSES after. This is the load-bearing positive
  case: it is the exact form of the defect that motivated this check.

- [ ] **RG-006** (`test_spec_check_benign_cases_exempt`): `--spec-check` mode, must-NOT-flag
  cases. TWO named synthetic spec fragments that MUST produce 0 findings (exit 0):
  (a) **`spec_past_tense_exempt`**: "`S-QUERY-SCOPE-PARAMS-001` was corrected in an earlier
      burst — no longer an open violation." (past-tense verb "was corrected"; present-tense
      trigger absent → exempt from detection).
  (b) **`spec_general_narrative_exempt`**: "This story builds the gate later consumed by
      `S-MAINT-WAVE-ANCHOR-GATE-001`." (story ID present, no state-verb claim about the
      story's corpus property → general narrative, exempt).
  Both fragments MUST emit 0 findings and exit 0. FAILS before implementation (stub's
  default exit 1 causes false-positive on both); PASSES after. Mirrors the
  `scripts/records-lint.sh` convention of carrying both PASS and FAIL fixtures per check
  so the implementer cannot ship a detector that passes its positive case while flagging
  every benign mention.

**Red Gate density check (BC-5.38.001):** 6 Red Gate tests (RG-001 through RG-006; RG-005
and RG-006 both anchor to AC-005) across 5 acceptance criteria (AC-001 through AC-005).
Density ratio: 6 RG items / 5 ACs = 1.2, satisfying BC-5.38.001. Density validation at
dispatch time per `per-story-delivery.md §Red Gate Density Check` and
BC-5.38.002/BC-5.38.003.

---

## Architecture Mapping

| Component | Location | Pure/Effectful |
|-----------|----------|----------------|
| `scripts/validate-pol29-discharge.sh` | `scripts/validate-pol29-discharge.sh` | Pure (reads burst report text; exit code is the result) |
| `--self-probe` cases | same file | Effectful (synthetic burst report strings fed as input) |
| CLAUDE.md §TD-VSDD-097 amendment | project root `CLAUDE.md` | Pure (governance document text edit) |
| CLAUDE.md §SAP-4 addition | project root `CLAUDE.md` | Pure (governance document text edit) |
| SESSION-HANDOFF.md §Standing Orchestrator Rules amendment | `.factory/SESSION-HANDOFF.md` | Pure (governance document text edit) |

---

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | A fixer reports grep output using a structured format that is equivalent but not literally the `grep -r ... file` form (e.g., describes files scanned and results in a table) | The SAP-4 probe and validation script MUST accept content-equivalent forms. Over-prescribing the exact format causes false positives on legitimate high-quality reports. The probe checks for the PRESENCE of per-file output and an explicit hit-count, not the literal syntax of the search command. |
| EC-002 | A dimension is genuinely not applicable (e.g., dimension 9b: the fixer confirms there is no downstream copy target, and names the specific section that would be copied if one existed) | "Not applicable" accompanied by the name of the section checked and a confirmation that no copy target exists IS a legitimate discharge. The SAP-4 probe should distinguish "not applicable [alone]" (verdict word; P1) from "not applicable — section X checked, no copy target: searched `grep -r 'section heading' .factory/specs/` — 0 hits" (supported claim; pass). |
| EC-003 | A burst report covers multiple dimensions in one combined paragraph | The validation script should extract dimension claims from the combined paragraph; a single grep-evidence block covering multiple dimensions is acceptable if the evidence clearly covers each claimed dimension. |
| EC-004 | SESSION-HANDOFF.md amendment conflicts with an existing Standing Rule | Amend the existing rule rather than duplicating it. If a standing rule already covers this aspect partially, extend it. |

---

## Token Budget Estimate

| Artifact | Estimated Tokens | Notes |
|----------|-----------------|-------|
| This story spec | ~6,000 | |
| CLAUDE.md §TD-VSDD-097 and §SAP-1..SAP-3 sections | ~5,000 | Load only the relevant standing-probe and dimension-checklist sections |
| SESSION-HANDOFF.md §Standing Orchestrator Rules section | ~3,000 | Load only the standing-rules section |
| Existing `scripts/records-lint.sh` --self-probe structure (reference) | ~5,000 | Reference for the validate-pol29-discharge.sh --self-probe pattern |
| **Total per implementation session** | ~19,000 | Well within 20–30% of agent context window |

---

## Tasks

### Phase A — Write failing validation script stubs (Red Gate — do BEFORE implementation)

- [ ] **T-A01** (RG-001 and RG-003 setup): Create `scripts/validate-pol29-discharge.sh` with a
  permanent-fail stub body (`exit 1` on all input). Add `--self-probe` with RG-001 (verdict-word
  case expects exit 1 with "lacks grep evidence" message) and RG-003 (zero-claims case expects
  exit 1 with "No POL-29 dimension discharge claims detected" message). Run `--self-probe` and
  confirm both cases report expected failures from the stub.

- [ ] **T-A02** (RG-002 setup): Add RG-002 (grep-evidence case expects exit 0) self-probe case
  against the permanent-fail stub. Confirm it reports expected failure (stub always exits 1;
  RG-002 expects 0, so this is a correct "test fails before impl" state).

### Phase B — Implement validation script (Green — make failing tests pass)

- [ ] **T-B01**: Implement burst-report dimension-claim extraction — scan input text for
  dimension-discharge markers ("dimension 9a", "(9a)", etc.). Emit "Found N dimension discharge
  claims" count.

- [ ] **T-B02**: Implement grep-evidence presence check — for each claim, scan the claim's
  adjacent context for grep-output indicators (a `grep` command literal, per-file hit counts,
  or structured equivalents per EC-001). Set M (claims lacking grep evidence).

- [ ] **T-B03**: Implement exit-code logic: N=0 → exit 1 + zero-claims message; M>0 → exit 1 +
  findings; N>0 AND M=0 → exit 0. Add `--self-probe` mode orchestrating RG-001/RG-002/RG-003.

- [ ] **T-B04** (RG-004): Run `scripts/records-lint.sh --full-scan` on CLAUDE.md BEFORE the
  CLAUDE.md amendment to establish baseline. This is the RG-004 "FAILS before amendment"
  confirmation (the amendment doesn't exist yet, but the baseline must be clean so RG-004
  tests the amendment itself, not a pre-existing violation). Document the baseline result.

### Phase C — CLAUDE.md and SESSION-HANDOFF.md amendments

- [ ] **T-C01** (AC-001, RG-001): Amend CLAUDE.md §TD-VSDD-097 three-dimension checklist to
  add the grep-evidence requirement for each of 9a, 9b, 9c. Ensure 9a includes the
  "twin-current-state, not just string-absent" requirement.

- [ ] **T-C02** (AC-002): Add SAP-4 probe specification to CLAUDE.md alongside SAP-1, SAP-2,
  SAP-3. Follow the same structural format as existing SAP entries.

- [ ] **T-C03** (AC-004): Amend SESSION-HANDOFF.md §Standing Orchestrator Rules to include the
  POL-29 dimension discharge quality rule with reference to this story.

- [ ] **T-C04** (RG-004): Run `scripts/records-lint.sh --full-scan` on the amended CLAUDE.md.
  Confirm exit 0 (no L9/L11 violations introduced).

### Merge gate

- [ ] **MERGE-GATE-SELF-PROBE**: `scripts/validate-pol29-discharge.sh --self-probe` exits 0
  with all RG cases passing.
- [ ] **MERGE-GATE-CLAUDE-MD-CLEAN**: `scripts/records-lint.sh --full-scan` on the amended
  CLAUDE.md exits 0 (RG-004 confirmed).
- [ ] **MERGE-GATE-SESSION-HANDOFF-AMENDED**: SESSION-HANDOFF.md §Standing Orchestrator Rules
  contains the new POL-29 discharge-quality rule with story reference.

---

## Previous Story Intelligence

N/A — first story in the POL-29 discharge-evidence chain.

Related prior art:
- `S-MAINT-POL29-HOOK-001`: mechanizes the POL-29 cite-pin sweep (step 8). Different concern:
  that story detects stale cite-pins in committed artifacts; this story governs the quality of
  dimension discharge attestations in burst reports. The two stories are complementary.
- `S-MAINT-L11-GATE-001`: model for CLAUDE.md amendment stories — same pattern (governance text
  + validation mechanism + self-probe tests).
- SAP-1, SAP-2, SAP-3 in CLAUDE.md: standing probe pattern to follow for SAP-4.

---

## Architecture Compliance Rules

1. **No prism crate modifications.** No file under `crates/` is touched.

2. **SESSION-HANDOFF.md is state-manager territory for structural STATE changes, but
   §Standing Orchestrator Rules additions are governance text.** The architect or orchestrator
   may amend §Standing Orchestrator Rules for governance rule additions. This story's T-C03 is
   a governance text edit, not a state change.

3. **Self-compliance of CLAUDE.md amendment (AC-001, RG-004):** The amended TD-VSDD-097 text
   MUST NOT itself contain volatile line cites or version pins. Run records-lint before and
   after to confirm. Anchor: AC-001 / RG-004. This MUST is anchored to AC-001 and RG-004.

4. **POL-29 TD-VSDD-097 three-dimension sweep for this burst's own POL-29 discharge:**
   (a) Sibling pair: TD-VSDD-097 is the single TD amended; no named twin file.
   (b) Downstream copy target: SESSION-HANDOFF.md §Standing Orchestrator Rules is the
       downstream copy target for TD-VSDD-097 standing-rule language; sweep and update per T-C03.
   (c) Mandate anchor: every MUST in the amended TD-VSDD-097 text cites this story
       (S-MAINT-POL29-GREP-EVIDENCE-001) as the anchor. Anchor: AC-001. This MUST is
       anchored to AC-001 and RG-001.

---

## Library and Framework Requirements

| Library/Tool | Version/Source | Purpose |
|-------------|---------------|---------|
| `bash` | System bash (≥ 3.2) | Script runtime for validate-pol29-discharge.sh |
| `grep` | System grep | Burst report scanning |

No external dependencies. No changes to Cargo.toml files.

---

## File Structure Requirements

### Files to CREATE

| File | Purpose |
|------|---------|
| `scripts/validate-pol29-discharge.sh` | POL-29 dimension discharge quality validator |

### Files to MODIFY

| File | Change |
|------|--------|
| `CLAUDE.md` | Extend §TD-VSDD-097 + add SAP-4 probe |
| `.factory/SESSION-HANDOFF.md` | Extend §Standing Orchestrator Rules |

### Files NOT to modify

| File | Reason |
|------|--------|
| Any `crates/**` file | No Rust code changes |
| `scripts/records-lint.sh` | Out of scope for this story |
| `.factory/STATE.md` | State-manager territory |

---

## Forbidden Dependencies

No new shell dependencies beyond POSIX standard utilities already used in the scripts directory
(`awk`, `grep`, `sed`, `mktemp`). The validate-pol29-discharge.sh script must be self-contained.

---

## Changelog

| Version | Date | Author | Summary |
|---------|------|--------|---------|
| 1.1 | 2026-07-31 | story-writer | FB105 continuation — adds AC-005 (--spec-check mode for present-tense corpus state claims in spec artifacts) and RG-005/RG-006 (must-flag + must-NOT-flag polarity fixtures). Both-polarity self-probe convention mirrors records-lint.sh. Findings are records-tier P2 (TD-VSDD-096 routing). Points 3→5. Density 6 RG / 5 ACs = 1.2. Motivated by this cascade's fifth consecutive leg where an agent's self-reported corpus state claim did not survive independent verification. |
| 1.0 | 2026-07-31 | story-writer | FB105 — initial story creation. Codifies Lesson 114 (POL-29 9a dispatch-perimeter-terminates-at-boundary, 3 instances) and Lesson 116 (fixer self-certification: 2 consecutive bursts reported clean dimension while residuals remained). Delivers CLAUDE.md §TD-VSDD-097 amendment requiring grep-evidence for POL-29 dimension discharges (not verdict words), SAP-4 adversary probe, and validate-pol29-discharge.sh validation script. 4 ACs, 4 RG tests. status: draft; behavioral_contracts: [] pending PO authorship. |
