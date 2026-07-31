---
document_type: story
story_id: S-MAINT-WAVE-ANCHOR-GATE-001
title: "ADR Wave-Granularity Deferral Gate — Untrackable Obligation Detector"
wave: maintenance
epic_id: maintenance
priority: P1
status: draft
version: "1.2"
updated: "2026-07-31"
level: "L2"
producer: story-writer
timestamp: "2026-07-31T00:00:00Z"
tdd_mode: strict
# tdd_mode: strict — this story creates a new validator script
# scripts/validate-wave-anchors.sh with a --self-probe mode. Red Gate tests are
# self-probe cases that FAIL before the script exists and PASS after. No Rust crates
# touched; the --self-probe framework serves the identical TDD purpose.
subsystems: []
crates_touched: []
target_module: "scripts/validate-wave-anchors.sh, CLAUDE.md"
capabilities: []
behavioral_contracts: []
# BC status: pending PO authorship.
# ADR deferral discipline is a governance requirement, not a behavioral contract.
# This story MUST remain status: draft until a governance-integrity BC is authored.
verification_properties: []
holdout_scenarios: []
# POL-35 holdout_gate_infra_only_exemption applies: behavioral_contracts: [] (pure
# toolchain governance scope). holdout_scenarios: [] is compliant for this story.
depends_on: []
blocks: []
points: 5
estimated_days: 2.0
risk: MEDIUM
# Risk justification:
#   The check must distinguish legitimate deferral language (OPEN OBLIGATION with a
#   real story ID) from violating deferral language (wave name, code comment paraphrase).
#   False positives on legitimate deferrals block architect commits. EC-001 through
#   EC-004 document the exemption paths. Pattern calibration requires reading several
#   ADR files to tune the detection pattern.
assumption_validations: []
risk_mitigations: []
tags:
  - toolchain
  - governance-gate
  - pol-29
  - canonical-principle-rule-3
  - adr-authoring
---

# S-MAINT-WAVE-ANCHOR-GATE-001: ADR Wave-Granularity Deferral Gate — Untrackable Obligation Detector

## Origin

**Process-gap finding:** Lesson 115 (D-2074, 2026-07-31). Three confirmed live instances.

FB103 found six occurrences of "wave-5" or "deferred to wave-5" in normative ADR body text
(ADR-057 §D7/§D5 body text — corrected in FB103). The same pattern appeared in story files:
`S-DEMO-QUERY-PUSHDOWN-001` §Out-of-Scope and §Architecture Compliance Rules both carried
wave-granularity deferral language; `S-QUERY-SCOPE-PARAMS-001` had two occurrences. Both
story files were corrected in FB104 and are no longer open violations. (`S-DEMO-QUERY-PUSHDOWN-001`
retains one occurrence inside its `## Changelog` section — a historical record,
exempt under EC-001.)

Three open violations exist at story-authoring time — two in the ADR corpus (in scope for
this gate), one in a non-ADR file (out of scope, separately routed):

**ADR-resident violations** (independently verified via
`grep -n "wave-[0-9]\|future wave"` against `.factory/specs/architecture/decisions/`):
- **`ADR-033 §Decision`**: "a future wave-6 story" — no real story ID, no OPEN OBLIGATION
  language, in normative decision text. Architect-owned. In scope for this gate.
- **`ADR-033 §Context`**: quotes the code doc-comment "deferred to wave-5" verbatim in
  normative context text. Architect-owned; the underlying doc-comment in
  `extract_push_down_filters_as_map` is implementer-owned if it still carries that text.
  In scope for this gate.

**Non-ADR violation** (independently verified via
`grep -n "deferred to wave-5"` against `specs/prd-supplements/error-taxonomy.md`):
- **`error-taxonomy.md` E-QUERY-034 row**: within the nested-code adjudication paragraph,
  quotes the `extract_push_down_filters_as_map` doc-comment ("Per-sensor
  `classify_predicates` integration deferred to wave-5") as contextual explanation for why
  E-QUERY-009 is latent. Product-owner-owned. Outside this gate's ADR-only scan range —
  the gate scans `.factory/specs/architecture/decisions/` only. Routing: product-owner for
  separate resolution; not in the architect-owned ADR-033 fix scope. Recorded here so it
  is not lost.

The Canonical Principle Rule 3 requirement — deferral target must be grepped from
`.factory/stories/*.md` frontmatter — is violated by wave names.
The pattern meets the TD-VSDD-097 codification threshold.

**Operational test (from Lesson 115):** If the deferral target cannot be grepped from
`.factory/stories/*.md` frontmatter as a real `story_id`, it is not a trackable anchor.
A wave name, a phase name, or a code comment is not a story anchor.

**Root cause from Lesson 115:** The ADR-057 instance originated by quoting a *code* scope
note verbatim into normative ADR text. In an implementation file "deferred to wave-5" is
a discoverable TODO; the same words in an ADR become a normative architectural obligation
anchored to nothing.

---

## Narrative

As an architect committing to `.factory/specs/architecture/decisions/`, I want a validator
script `scripts/validate-wave-anchors.sh` to block normative ADR body text containing
wave-granularity deferral language (e.g., "deferred to wave-N", "wave-N scope",
"future wave", "wave-N story") unless the same sentence also cites a real story ID that
exists in `.factory/stories/` frontmatter OR uses explicit OPEN OBLIGATION language — and I
want CLAUDE.md §TD-VSDD-097 and §ADR authoring conventions amended to encode this
operational test — so that the "find in stories frontmatter or it is not trackable" rule
is mechanically enforced and wave-granularity deferrals cannot silently accumulate in the
ADR corpus.

---

## Background

Wave names ("wave-5", "wave-6") are planning vocabulary, not spec anchors. A deferral in
ADR normative text that cites only a wave name is invisible to:
- `grep -r 'story_id: ' .factory/stories/*.md` — no story ID present
- `grep` searches on the wave name alone — returns multiple unrelated stories
- POL-29 dimension-9c sweeps — the pattern does not match a story ID

The check introduced by this story operationalizes the Canonical Principle Rule 3 test:
"If the deferral target cannot be grepped from `.factory/stories/*.md` frontmatter, it is
not a trackable anchor." The check scans `.factory/specs/architecture/decisions/` for
wave-granularity deferral language in normative body text (outside exempt sections) and
verifies each match is accompanied by a real story ID or explicit OPEN OBLIGATION language.

**ADR directory path verified on disk:** `.factory/specs/architecture/decisions/`
(not `specs/architecture/adr/` — per dispatch correction).

---

## Behavioral Contracts

| BC | Title | Version | Scope in This Story |
|----|-------|---------|---------------------|
| (none) | — | — | No behavioral contracts yet. See BC status comment in frontmatter. Story MUST remain `status: draft` until a governance-integrity BC is authored. This MUST is anchored to: AC-001 governs detection; BC authorship is a human gate. |

---

## Acceptance Criteria

### AC-001 — Validator detects wave-granularity deferral language without adjacent story ID
After implementation, `scripts/validate-wave-anchors.sh` run against `.factory/specs/architecture/decisions/`
MUST emit a hard-block error (exit 1) for each ADR body-text match of the wave-deferral
pattern (see §deferral-pattern in §Background) where neither:
(a) a real story ID matching `S-[A-Z0-9][A-Z0-9.-]*` or `W3-[A-Z0-9-]+` or equivalent
    story-ID forms that exist in `.factory/stories/` appears in the same sentence or
    adjacent sentence (implementation detail: same paragraph, within 3 lines), NOR
(b) the phrase "OPEN OBLIGATION" appears in the same paragraph.
The error MUST identify the ADR file, the line content, and state the required fix
(add a real story ID or use OPEN OBLIGATION language).
(verified by RG-001)

### AC-002 — Validator passes for legitimate deferral language
The validator MUST exit 0 for:
(a) A deferral citing a real story ID from `.factory/stories/` frontmatter:
    e.g., "deferred to `S-REQUIRED-COL-GATE-001`" — story ID exists on disk.
(b) A deferral using explicit OPEN OBLIGATION language:
    e.g., "OPEN OBLIGATION — no story ID yet; follow-up story required."
(c) Deferral language in a `## Changelog` or `## Version History` section of an ADR
    (historical records; not normative obligations).
(d) Deferral language in a `# verified-empty:` comment or frontmatter block.
(verified by RG-002)

### AC-003 — Positive coverage: validator emits ADR-file scanned count > 0
The validator MUST emit "Scanned N ADR files" where N is a runtime-computed integer.
If N = 0, the validator exits 1 with "No ADR files found at `.factory/specs/architecture/decisions/` — check path configuration." A zero-item scan is structurally indistinguishable from a misconfigured or missing path; the validator MUST fail loud rather than green when scanning no items.
(verified by RG-003)

### AC-004 — Validator has `--self-probe` mode with at least 4 cases
`scripts/validate-wave-anchors.sh --self-probe` runs at least 4 synthetic test cases:
(a) ADR with wave-deferral language and no story ID → FAIL case.
(b) ADR with wave-deferral language and a valid story ID → PASS case.
(c) ADR with OPEN OBLIGATION language → PASS case.
(d) Zero-ADR-corpus protection case → validator exits 1 with zero-corpus message.
The implementation states the `--self-probe` expected total (≥ 4) and verifies that total
via a `--self-probe` run before declaring done.
(verified by RG-001 through RG-004)

### AC-005 — CLAUDE.md and ADR authoring conventions amended
CLAUDE.md is amended (in the same PR) to add:
- Under §Standing Adversary Probes or a new §ADR Authoring Conventions section: the
  operational test from Lesson 115: "If the deferral target cannot be grepped from
  `.factory/stories/*.md` frontmatter as a real `story_id`, it is not a trackable anchor."
- A prohibition on quoting code scope notes verbatim into normative ADR body text: "Code
  TODOs and implementation-file scope notes are discoverable implementation details; the
  same words in normative ADR text become governance obligations anchored to nothing."
The amendment MUST NOT introduce volatile line cites or version pins (records-lint self-compliance).
(verified by RG-005)

---

## Red Gate Tests

All 5 RG items are test cases that FAIL before the validator script exists and PASS after.
RG-001 through RG-004 are `--self-probe` cases for the script; RG-005 validates the CLAUDE.md
amendment is self-compliant.

- [ ] **RG-001** (`test_wave_anchor_violation_exits_nonzero`): self-probe case — a synthetic ADR
  file containing "deferred to wave-5 scope — T2 filter enforcement" (no story ID, no OPEN
  OBLIGATION language) causes `validate-wave-anchors.sh` to exit 1 with error identifying
  the match. FAILS before script exists; PASSES after implementation.

- [ ] **RG-002** (`test_wave_anchor_story_id_passes`): self-probe case — a synthetic ADR file
  containing "deferred to `S-REQUIRED-COL-GATE-001` (wave-A story)" where `S-REQUIRED-COL-GATE-001`
  exists in `.factory/stories/` passes the validator. Implementation note: the `--self-probe`
  must use the real `.factory/stories/` path or a synthetic stories directory with the story
  file present. FAILS before script exists; PASSES after implementation.

- [ ] **RG-003** (`test_wave_anchor_zero_adr_corpus_exits_nonzero`): self-probe case — a synthetic
  workspace with no ADR files at the decisions path causes the validator to emit "No ADR files
  found" and exit 1. Mechanizes the positive-coverage requirement from AC-003. FAILS before
  implementation; PASSES after.

- [ ] **RG-004** (`test_wave_anchor_open_obligation_passes`): self-probe case — a synthetic ADR
  file containing "OPEN OBLIGATION — no story ID yet; follow-up story required" passes the
  validator without error. Confirms the legitimate-deferral escape hatch works. FAILS before
  implementation; PASSES after.

- [ ] **RG-005** (`test_wave_anchor_claude_md_amendment_self_compliant`): validation case —
  after the CLAUDE.md amendment is authored, `scripts/records-lint.sh --full-scan` on
  CLAUDE.md exits 0 (no volatile line cites or version pins introduced). FAILS if the
  amendment introduces a violation; PASSES if clean.

**Red Gate density check (BC-5.38.001):** 5 Red Gate tests (RG-001 through RG-005) anchor to
5 acceptance criteria (AC-001 through AC-005). Density ratio: 5 / 5 = 1.0, satisfying
BC-5.38.001. Density validation at dispatch time per `per-story-delivery.md §Red Gate Density
Check` and BC-5.38.002/BC-5.38.003.

---

## Architecture Mapping

| Component | Location | Pure/Effectful |
|-----------|----------|----------------|
| `scripts/validate-wave-anchors.sh` | `scripts/validate-wave-anchors.sh` | Pure (scans ADR files; exit code is the gate) |
| `--self-probe` mode | same file | Effectful (creates synthetic ADR fixtures in `$TMPDIR`) |
| CLAUDE.md amendment | project root `CLAUDE.md` | Pure (governance document text edit) |

---

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Wave-deferral language appears in a `## Changelog` section of an ADR | Exempt — changelog rows are historical records. The validator MUST skip lines within `## Changelog` or `## Version History` sections (same section-range detection as L11 in S-MAINT-L11-GATE-001). |
| EC-002 | Story ID cited in the deferral does NOT exist in `.factory/stories/` | Hard-block — the cited story ID is phantom. The error message must state "Story ID `S-NNN` cited but not found in `.factory/stories/`." |
| EC-003 | ADR has `status: superseded` in frontmatter | The validator MUST skip superseded ADRs — they have no meaningful deferral obligation. Consistent with S-MAINT-ADR-ANCHOR-GATE-001 EC-001 precedent. |
| EC-004 | Deferral language inside a fenced code block (between ` ``` `) | The validator flags it — code blocks in ADR normative text still carry obligations. If the intent is to show a code example containing the phrase, wrap it in a prose explanation that the example is illustrative. |
| EC-005 | ADR amendment file (`document_type: adr-amendment`) contains wave-deferral language | The validator MUST check amendment files too — amendments are normative patches to the ADR they modify. Consistent with Lesson 112 on amendment-file coverage gaps. |

---

## Token Budget Estimate

| Artifact | Estimated Tokens | Notes |
|----------|-----------------|-------|
| This story spec | ~6,000 | |
| CLAUDE.md §SAP-1..SAP-3 and §ADR authoring sections | ~4,000 | Load for amendment context |
| Sample ADR files (3–4 for pattern calibration) | ~8,000 | Implementer selects representative examples |
| S-MAINT-ADR-ANCHOR-GATE-001 (structural reference) | ~3,000 | Same pattern: factory validator, ADR scope |
| **Total per implementation session** | ~21,000 | Within 20–30% of agent context window |

---

## Tasks

### Phase A — Write failing self-probe cases (Red Gate — do BEFORE implementation)

- [ ] **T-A01** (RG-001, RG-003): Create `scripts/validate-wave-anchors.sh` with permanent-fail
  stub body. Add `--self-probe` with RG-001 (violation case expects exit 1) and RG-003
  (zero-corpus case expects exit 1). Run `--self-probe` and confirm both fail as expected.

- [ ] **T-A02** (RG-002, RG-004): Add RG-002 (story-ID pass case expects exit 0) and RG-004
  (OPEN OBLIGATION pass case expects exit 0) against the stub. Confirm the stub causes them
  to fail as expected (stub exits 1 on everything; the "pass" cases therefore show the FAIL
  state correctly). Record expected-fail count.

### Phase B — Implement the check (Green — make failing tests pass)

- [ ] **T-B01**: Define the wave-deferral detection pattern constant. Pattern candidates:
  `(deferred to wave-[0-9]|wave-[0-9] scope|future wave|wave-[0-9]+ story)` — implementer
  refines for the project's actual ADR corpus. Add to CONFIG BLOCK section.

- [ ] **T-B02**: Implement the section-range exemption helper (skip `## Changelog` and
  `## Version History` lines; skip frontmatter block; skip superseded ADR files per EC-003).
  Reuse or copy the section-range pattern from records-lint.sh if already implemented.

- [ ] **T-B03**: Implement the story-ID existence check helper — given a story ID string,
  verify that a file matching it exists in `.factory/stories/`. Returns 0 (found) or 1 (not
  found). Uses `find .factory/stories/ -name "*${story_id}*"` or equivalent.

- [ ] **T-B04**: Implement the main scan loop — for each `.md` file in `.factory/specs/architecture/decisions/`
  (including amendment files per EC-005): scan non-exempt lines for the wave-deferral pattern;
  for each match, check for adjacent story ID or OPEN OBLIGATION language; accumulate violations.
  Emit "Scanned N ADR files" count before reporting. Exit 1 if N=0 or violations found.

### Phase C — Verify self-probe passes

- [ ] **T-C01**: Run `scripts/validate-wave-anchors.sh --self-probe`. Confirm all 4 cases pass.
  State the expected total.

- [ ] **T-C02** (live corpus run): Run the validator against the actual
  `.factory/specs/architecture/decisions/` corpus. At story-authoring time, two known
  ADR-resident open violations were independently verified via `grep -n`:
  - `ADR-033 §Decision`: "a future wave-6 story" (no real story ID; architect-owned).
  - `ADR-033 §Context`: code doc-comment quote "deferred to wave-5" in normative context
    text (architect-owned; the underlying code doc-comment is implementer-owned).
  Do NOT fix these — each is separately routed. A third open instance in
  `specs/prd-supplements/error-taxonomy.md` (E-QUERY-034 row, product-owner-owned)
  is outside this gate's ADR-only scan range and is NOT expected to appear in the live
  corpus run output — do not include it in the ADR-fix scope. Document the complete set
  of violations emitted by the live corpus run in the PR description.
  (Note: `S-DEMO-QUERY-PUSHDOWN-001` and `S-QUERY-SCOPE-PARAMS-001` are corrected
  precedents from FB104, not open violations; they appear in §Origin as historical examples.)

### Phase D — CLAUDE.md amendment

- [ ] **T-D01** (AC-005, RG-005): Amend CLAUDE.md to add the operational test and code-todo
  prohibition per AC-005. Run `scripts/records-lint.sh --full-scan` on the amended CLAUDE.md
  to confirm RG-005.

### Merge gate

- [ ] **MERGE-GATE-SELF-PROBE**: `scripts/validate-wave-anchors.sh --self-probe` exits 0 with
  all cases passing.
- [ ] **MERGE-GATE-CORPUS-REPORT**: Live corpus run output documented in PR description.
- [ ] **MERGE-GATE-CLAUDE-MD-CLEAN**: `scripts/records-lint.sh --full-scan` on amended CLAUDE.md
  exits 0 (RG-005).

---

## Previous Story Intelligence

N/A — first story in the wave-anchor gate chain.

Related prior art:
- `S-MAINT-ADR-ANCHOR-GATE-001`: direct structural precedent — same ADR-file target scope,
  same factory-hook/script pattern, same superseded-ADR exemption (EC-001 / EC-003 here).
  The implementer SHOULD read both stories before implementing to maintain structural consistency.
  ADR path confirmed: `.factory/specs/architecture/decisions/` (not `specs/architecture/adr/`).
- `S-MAINT-L11-GATE-001`: model for the section-range exemption mechanism and self-probe pattern.
- S-REQUIRED-COL-GATE-001 is a real story on disk at `.factory/stories/S-REQUIRED-COL-GATE-001-required-column-enforcement-gate-e-query-009.md` — usable as a test fixture for RG-002.

---

## Architecture Compliance Rules

1. **No prism crate modifications.** No file under `crates/` is touched.

2. **ADR directory path:** `.factory/specs/architecture/decisions/` — NOT `specs/architecture/adr/`.
   The positive-coverage check (AC-003) verifies the path is correct by requiring N > 0. If the
   path is wrong, N=0 and the check fails.

3. **Amendment files must be covered (EC-005):** The validator covers `document_type: adr-amendment`
   files as well as `document_type: adr` files. Lesson 112 documented the gap from gating only
   on file type without specifying how amendment files are treated.

4. **CLAUDE.md self-compliance:** The amended CLAUDE.md text MUST NOT introduce volatile line
   cites or version pins. RG-005 mechanizes this check. This MUST is anchored to AC-005 and RG-005.

5. **POL-29 TD-VSDD-097 three-dimension sweep for this story's own burst:**
   (a) Sibling pair: S-MAINT-ADR-ANCHOR-GATE-001 is the closest structural sibling (both gate
       ADR files). Sweep it: does it already cover wave-deferral language? Answer: No — that
       story covers `anchor_stories` key presence and population. These are distinct checks.
       No content update needed.
   (b) Downstream copy target: SESSION-HANDOFF.md §ADR authoring conventions may need a pointer
       to the operational test; sweep and update if the amendment is a verbatim copy-source.
   (c) Mandate anchor: every MUST in this story cites a specific AC + RG item. Confirmed above.

---

## Library and Framework Requirements

| Library/Tool | Version/Source | Purpose |
|-------------|---------------|---------|
| `bash` | System bash (≥ 3.2) | Script runtime |
| `grep` | System grep | Wave-deferral pattern matching |
| `find` | POSIX find | ADR file enumeration and story-ID existence check |

No external dependencies. No changes to Cargo.toml files.

---

## File Structure Requirements

### Files to CREATE

| File | Purpose |
|------|---------|
| `scripts/validate-wave-anchors.sh` | ADR wave-granularity deferral detector |

### Files to MODIFY

| File | Change |
|------|--------|
| `CLAUDE.md` | Add operational test + code-todo prohibition per AC-005 |

### Files NOT to modify

| File | Reason |
|------|--------|
| Any `crates/**` file | No Rust code changes |
| `.factory/**` files | State-manager handles .factory/ commits |
| `scripts/records-lint.sh` | Out of scope for this story |

---

## Forbidden Dependencies

No new shell dependencies beyond POSIX standard utilities. `validate-wave-anchors.sh` must be
self-contained with `bash`, `grep`, `find`, and `mktemp` only.

---

## Changelog

| Version | Date | Author | Summary |
|---------|------|--------|---------|
| 1.2 | 2026-07-31 | story-writer | FB105 continuation — elevates error-taxonomy.md E-QUERY-034 row instance from "noted" to a named open instance in §Origin and §Tasks T-C02. Product-owner-owned, outside this gate's ADR-only scan range; routing recorded explicitly so it is not lost. Three open violations now documented: two ADR-resident (architect-owned, in scope), one non-ADR (product-owner-owned, out of scope). No AC or RG changes. |
| 1.1 | 2026-07-31 | story-writer | FB105 continuation — corrected §Origin and §Tasks T-C02 factual error: S-DEMO-QUERY-PUSHDOWN-001 and S-QUERY-SCOPE-PARAMS-001 were corrected in FB104 and are not open violations; recasts them as corrected precedent. Replaces "two known open instances" claim with the actual ADR-resident open violations verified independently: ADR-033 §Decision ("a future wave-6 story") and ADR-033 §Context (code doc-comment quote in normative context text). Notes error-taxonomy.md E-QUERY-034 row instance as out-of-scope for this ADR-only gate, product-owner-owned. No AC or RG changes. |
| 1.0 | 2026-07-31 | story-writer | FB105 — initial story creation. Codifies Lesson 115 (wave-granularity deferrals, 3 instances in FB103). Creates scripts/validate-wave-anchors.sh to detect wave-granularity deferral language in ADR normative text that lacks a real story ID or OPEN OBLIGATION. CLAUDE.md amendment adds operational test and code-todo prohibition. 5 ACs, 5 RG tests. ADR path `.factory/specs/architecture/decisions/` confirmed on disk. Amendment-file coverage included (Lesson 112 precedent). status: draft; behavioral_contracts: [] pending PO authorship. |
