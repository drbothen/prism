---
document_type: story
story_id: S-MAINT-LEDGER-CITE-GATE-001
title: "records-lint Ledger-Citation Content Cross-Reference Check"
wave: maintenance
epic_id: maintenance
priority: P1
status: draft
version: "1.0"
updated: "2026-07-31"
level: "L2"
producer: story-writer
timestamp: "2026-07-31T00:00:00Z"
tdd_mode: strict
# tdd_mode: strict — this story adds executable gate logic to scripts/records-lint.sh.
# Red Gate tests are --self-probe cases that FAIL before the new check is implemented
# (the check does not exist; any violation-test on a non-existent function returns 0,
# producing false-green). They PASS after implementation.
# Standard Rust todo!() discipline does not apply (no Rust crates touched), but the
# --self-probe test framework serves the identical TDD purpose.
subsystems: []
# Cross-cutting governance toolchain: no single subsystem owns it.
crates_touched: []
target_module: "scripts/records-lint.sh, CLAUDE.md"
capabilities: []
behavioral_contracts: []
# BC status: pending PO authorship.
# Ledger-content integrity is a governance discipline, not a behavioral contract.
# This story MUST remain status: draft until a governance-integrity BC is authored and
# anchored (S-7.01 gate).
verification_properties: []
holdout_scenarios: []
# POL-35 holdout_gate_infra_only_exemption applies: behavioral_contracts: [] (pure
# toolchain governance scope). holdout_scenarios: [] is compliant for this story.
depends_on: []
blocks: []
points: 5
estimated_days: 2.0
risk: LOW
# Risk justification:
#   All changes are toolchain-only (bash script + CLAUDE.md). No production Rust code
#   touched. Primary risk is false-positive matches on legitimate finding-ID references
#   in artifacts (e.g., an artifact that discusses a finding in its prose without that
#   finding being a direct ledger attribution). The exemption rules in AC-003 mitigate this.
assumption_validations: []
risk_mitigations: []
tags:
  - toolchain
  - governance-gate
  - td-vsdd-092
  - records-lint
  - index-integrity
---

# S-MAINT-LEDGER-CITE-GATE-001: records-lint Ledger-Citation Content Cross-Reference Check

## Origin

**Process-gap finding:** Lesson 113 (D-2072, 2026-07-30), codification threshold met at 5 occurrences.

Three findings from pass-72 (F-WASE-P72-HIGH-004, F-WASE-P72-MED-001, F-WASE-P72-MED-002) plus one prior annotated instance in BC-2.01.016's registry row plus the live session instance (FB104 STATE.md `total_stories` field) constitute the codification-threshold event for this defect class: an index ledger row carries content — finding attribution, section reference, or fix-burst citation — that contradicts the artifact it claims to describe. The version number in the row is technically current, so the existing L10 check passes. The falsification is invisible to every deployed mechanical gate.

The L10 capability boundary (CLAUDE.md §TD-VSDD-092) documents this gap explicitly: "L10 detects the version-number half of index drift only. It cannot detect content falsification — a row describing a change that does not exist in the target artifact."

---

## Narrative

As a records-discipline maintainer, I want `scripts/records-lint.sh` to gain a new check that
cross-references the finding-ID citations in BC-INDEX, ARCH-INDEX, and VP-INDEX ledger rows
against the target artifacts they claim to describe — emitting a PHANTOM citation error for each
finding ID that appears in the index row but greps to zero hits in the target artifact's own
`## Changelog` section or body — and I want CLAUDE.md updated to document this new check as an
extension to TD-VSDD-092, so that index ledger-content falsification is mechanically detected
at commit time and the "L10 detects version-number drift only" boundary note is updated to
accurately reflect the new coverage.

---

## Background

Every BC-INDEX row that cites a specific finding ID (e.g., `F-WASE-P71-HIGH-005`) is making a
content claim: that finding ID is evidence of a change in the referenced artifact at the stated
version. A PHANTOM citation is a row whose finding ID appears nowhere in the target artifact's
own file. A STALE citation is a row whose finding ID appears in the wrong artifact's file.

The three pass-72 instances:
- HIGH-004: BC-INDEX row attributes a pass-71 finding to BC-2.02.014; the finding's actual
  target is BC-2.02.006. The finding ID greps to zero in BC-2.02.014.
- MED-001: BC-INDEX row cites a phantom ADR section anchor. The section does not exist in
  the target ADR.
- MED-002: ARCH-INDEX row attributes an error-code registration to the wrong fix-burst;
  the cited fix-burst ID does not appear in the target architecture file.

**Live session instance (FB104):** STATE.md's `total_stories:` frontmatter field read 258
while a NOTE in the decisions log claimed "total_stories 258→259." The burst's own report
claimed "total_stories verified by disk enumeration: 259." Records-lint L1 passed. The field
mismatch was caught only by a subsequent burst re-reading the field.

The STATE.md field consistency arm (detecting NOTE claims that contradict frontmatter field
values) is architecturally distinct from the finding-ID citation arm. Arm B (STATE.md) is
documented in §Background as a recommended extension but is a BONUS item for v1.0 of this
story — not a required AC. Reason: finding IDs in index ledger rows are parseable by a
well-defined regex; NOTE claims in STATE.md decisions are free-form text with no canonical
structure, making reliable mechanical detection substantially harder. The primary arm
(finding-ID citation cross-reference) addresses the indexed instances; Arm B is a stretch goal.

---

## Behavioral Contracts

| BC | Title | Version | Scope in This Story |
|----|-------|---------|---------------------|
| (none) | — | — | No behavioral contracts yet. See BC status comment in frontmatter. Story MUST remain `status: draft` until a governance-integrity BC is authored and anchored. This MUST is anchored to: AC-001 / RG-001 execute the gate; BC authorship is a human gate. |

---

## Acceptance Criteria

### AC-001 — New check detects finding-ID PHANTOM citations in index ledger rows
After implementation, `scripts/records-lint.sh` (corpus-audit mode; see AC-002) run against
a workspace where an index ledger row cites a finding ID matching the pattern
`F-[A-Z0-9]+-[A-Z0-9]+-[A-Z0-9-]+` (or the pattern refined by the implementer to match the
project's finding-ID naming convention) AND that finding ID greps to zero hits in the target
artifact's own file MUST produce a hard-block error identifying the index file, the row, the
cited finding ID, and the target artifact path.
(verified by RG-001)

### AC-002 — New check is corpus-audit mode (not ratchet-scoped) and emits scanned-row count
Unlike L9 and L11 (which are ratchet-scoped to staged additions), this check targets pre-existing
ledger-content falsification that may have accumulated before the gate existed. The check MUST run
in corpus-audit mode: it scans all rows in BC-INDEX, ARCH-INDEX, and VP-INDEX that contain a
recognizable finding-ID pattern, regardless of whether those rows are staged. The check MUST emit
a line of the form "Scanned N ledger rows across N index files" where N is a runtime-computed
integer greater than 0. A corpus where N=0 indicates a configuration or path error; the check
MUST exit 1 if the scanned-row count is zero (positive-coverage requirement: a check that scans
zero items is structurally indistinguishable from one that never ran).
(verified by RG-002 and RG-003)

### AC-003 — New check correctly exempts legitimate citations
The new check MUST correctly pass for:
(a) A finding-ID that appears in the target artifact's `## Changelog` section OR body prose,
    even if not in the exact version row (the artifact may discuss a finding found in a
    predecessor pass in its §Background or §Origin section).
(b) A ledger row that does NOT contain a recognizable finding-ID pattern (it may cite a burst
    number like "FB103" — the new check does not validate those citations).
(c) The VP-INDEX, BC-INDEX, and ARCH-INDEX files themselves (the check targets the artifacts
    those index files point to, not the index files themselves).
(verified by RG-004)

### AC-004 — `--self-probe` extended with new check cases; expected total stated
The `--self-probe` mode gains at minimum 4 new pass/fail cases covering:
(a) A PHANTOM citation case: index row cites a finding ID that greps to zero in the target artifact → FAIL.
(b) A valid citation case: index row cites a finding ID that appears in the target artifact → PASS.
(c) A zero-corpus-protection case: a synthetic repo where no index files exist or all rows lack
    finding IDs → check exits 1 with "Scanned 0 rows" rather than silently exiting 0 (per AC-002
    positive-coverage requirement).
(d) An exemption case: a row without a finding-ID pattern → PASS (not flagged).
The implementation states the new `--self-probe` expected total (previous total 34 or the
then-current total after preceding deployments; new total ≥ previous + 4) and verifies that total
via a `--self-probe` run before declaring done.
(verified by RG-001 through RG-004)

### AC-005 — CLAUDE.md updated: TD-VSDD-092 capability boundary note extended
CLAUDE.md §TD-VSDD-092 "L10 capability boundary" note is amended in the same PR as the
`records-lint.sh` changes:
- The note "L10 detects the version-number half of index drift only. It cannot detect content
  falsification" is extended to state: "This limitation is partially addressed by the new
  ledger-citation check (see §check-list); the ledger-citation check covers finding-ID PHANTOM
  citations. It does NOT cover free-form narrative falsification in rows that carry no
  structured finding-ID pattern."
- §check-list is extended to describe the new check: target files, citation pattern, PHANTOM
  detection, corpus-audit mode, positive-coverage emission.
- The new check name must be added to the `L9_CHECK_NAME_EXEMPT` list in records-lint.sh so
  that references to the check name in future `.factory/` staged additions are not flagged by
  L9 arm-5. The check's designated name token (assigned by implementer per the CONFIG BLOCK
  naming convention) MUST appear in `L9_CHECK_NAME_EXEMPT` before the PR merges.
(verified by RG-005)

---

## Red Gate Tests

All 5 RG items are --self-probe cases that FAIL before the new check is implemented and PASS
after. The --self-probe pattern uses synthetic temp git repos per the existing implementation
in `scripts/records-lint.sh`.

- [ ] **RG-001** (`test_ledger_cite_phantom_finding_id`): self-probe case — a synthetic repo
  with a BC-INDEX.md row citing a finding ID that does NOT appear in the target BC file
  triggers a hard-block error with the citation identified. FAILS before the new check
  function exists (no function → implicit 0 exit → false-green); PASSES after implementation.

- [ ] **RG-002** (`test_ledger_cite_valid_finding_id`): self-probe case — a synthetic repo
  with a BC-INDEX.md row citing a finding ID that DOES appear in the target BC file passes
  the new check without error. Establishes the non-regression baseline for legitimate ledger
  rows. FAILS before implementation (the function doesn't exist and can't return pass);
  PASSES after implementation as the false-positive guardrail.

- [ ] **RG-003** (`test_ledger_cite_zero_corpus_exits_nonzero`): self-probe case — a synthetic
  repo where no index files exist (or all index rows lack finding-ID patterns) produces a
  "Scanned 0 rows" message and exits 1. Mechanizes the positive-coverage requirement from
  AC-002: a zero-item scan must fail loud rather than green. FAILS before implementation
  (no function → exit 0); PASSES after implementation.

- [ ] **RG-004** (`test_ledger_cite_exempt_no_finding_id`): self-probe case — a synthetic BC-INDEX
  row that cites only a burst number (e.g., "FB103") or a plain text description with no
  finding-ID pattern produces no error. Prevents false-positives on the majority of index rows
  that don't carry structured finding IDs. FAILS before implementation; PASSES after.

- [ ] **RG-005** (`test_ledger_cite_check_name_exempt_l9`): self-probe case — a staged addition
  to a `.factory/` file that references the new check's designated name token (as assigned in
  the CONFIG BLOCK) passes L9 arm-5 without flagging. Confirms the check name was correctly
  added to `L9_CHECK_NAME_EXEMPT`. FAILS before the exempt entry is added; PASSES after.

**Red Gate density check (BC-5.38.001):** 5 Red Gate tests (RG-001 through RG-005) anchor to
5 acceptance criteria (AC-001 through AC-005). Each AC has one RG item. The density ratio is
5 RG items / 5 ACs = 1.0, which satisfies the `RED_TESTS * 2 >= (TOTAL_NEW_TESTS - EXEMPT_TESTS)`
requirement. Density validation at dispatch time per `per-story-delivery.md §Red Gate Density Check`
and BC-5.38.002/BC-5.38.003.

---

## Architecture Mapping

| Component | Location | Pure/Effectful |
|-----------|----------|----------------|
| New ledger-citation check function | `scripts/records-lint.sh` | Pure (read-only scan; exit code is the gate) |
| `--self-probe` new cases | same file | Effectful (creates temp git repos in `$TMPDIR`) |
| CLAUDE.md §TD-VSDD-092 amendment | project root `CLAUDE.md` | Pure (governance document text edit) |

---

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Finding ID appears in the index row as part of a narrative description ("this row corrects findings F-WASE-P72-HIGH-004 and F-WASE-P72-MED-001") but the target artifact is a story file (not a BC/ADR/VP) | The check MUST still grep the target artifact. Story files CAN appear in ARCH-INDEX rows. The finding ID must be found in the story file's body or changelog. |
| EC-002 | Target artifact file does not exist at the path extracted from the index row | Check reports a "TARGET_NOT_FOUND" error for that row (not a PHANTOM citation — a separate error class). The implementer decides whether to hard-block or warn for missing targets. |
| EC-003 | A finding ID matches the pattern but is a partial substring of a longer ID (e.g., "F-WASE-P7" is a substring of "F-WASE-P72-HIGH-004") | The regex MUST use word-boundary or full-token matching to avoid false positive on partial matches. |
| EC-004 | Index row cites finding ID in a `## Changelog` ROW (e.g., the BC-INDEX row itself is in a changelog table) | The new check targets index LEDGER rows (the content rows), not the index file's own changelog. Implementation note: if BC-INDEX has a `## Changelog` section, those rows are not index ledger rows and should be skipped. |
| EC-005 | Same finding ID cited by multiple index rows pointing to different artifacts | Each (row, artifact) pair is independently evaluated. A finding ID found in artifact A but not artifact B produces a PHANTOM citation for the B row only. |

---

## Token Budget Estimate

| Artifact | Estimated Tokens | Notes |
|----------|-----------------|-------|
| This story spec | ~6,000 | |
| `scripts/records-lint.sh` (full text) | ~18,000 | Required to understand --self-probe structure and CONFIG BLOCK |
| BC-INDEX.md (sample rows, finding-ID pattern research) | ~5,000 | Load for pattern calibration |
| CLAUDE.md §TD-VSDD-092 section | ~3,000 | Load only the relevant section |
| **Total per implementation session** | ~32,000 | Within 20–30% of agent context window |

Context management: implement the check function first, verify via --self-probe, then amend CLAUDE.md.

---

## Tasks

### Phase A — Write failing self-probe cases (Red Gate — do BEFORE implementation)

- [ ] **T-A01** (RG-001 and RG-003 setup): Add new check function stub with permanent-fail body
  (`return 1`) to `scripts/records-lint.sh`. Add RG-001 (PHANTOM detection) and RG-003
  (zero-corpus exits nonzero) self-probe cases that EXPECT the stub to fail on violations.
  Confirm `--self-probe` outputs the expected failures for both cases.

- [ ] **T-A02** (RG-002, RG-004, RG-005 setup): Add RG-002 (valid citation), RG-004 (no-finding-ID
  exemption), and RG-005 (check-name L9 exempt) cases against the permanent-fail stub. All five
  cases must fail in expected directions before implementation. Record new `--self-probe`
  expected-fail count.

### Phase B — Implement new check (Green — make failing tests pass)

- [ ] **T-B01**: Decide check name token (per CONFIG BLOCK naming convention) and add to:
  (a) the CONFIG BLOCK constant list in `scripts/records-lint.sh`, and (b) `L9_CHECK_NAME_EXEMPT`.
  Document the naming decision.

- [ ] **T-B02**: Define `_LEDGER_FINDING_ID_PATTERN` constant (regex matching project finding-ID
  form: `F-[A-Z0-9]+-[A-Z0-9]+-[A-Z0-9-]+` or implementer-refined form). Add to CONFIG BLOCK
  alongside existing pattern constants.

- [ ] **T-B03**: Implement target-artifact path extraction helper — given an index row, extract
  the BC/ADR/VP artifact ID and resolve to a filesystem path under `.factory/`. Returns empty
  string if no recognizable artifact ID found in the row.

- [ ] **T-B04**: Implement the main check function. For each index file (BC-INDEX, ARCH-INDEX,
  VP-INDEX), for each ledger row containing `_LEDGER_FINDING_ID_PATTERN`: extract all finding IDs,
  resolve the target artifact, grep the target artifact for each finding ID using word-boundary
  matching. Accumulate violations. Emit "Scanned N ledger rows across M index files" before
  reporting. Exit 1 if N=0 or if violations found; exit 0 otherwise.

- [ ] **T-B05**: Integrate the new check function into the main gate execution path. Decide
  whether it runs in default (pre-commit) mode, `--full-scan` mode only, or both. Document the
  decision. Recommendation: run in `--full-scan` mode since it's a corpus-audit check, not a
  staged-diff check. Anchor: AC-002.

### Phase C — Verify self-probe passes

- [ ] **T-C01**: Run `scripts/records-lint.sh --self-probe`. Confirm all 5 new cases pass.
  Confirm existing cases still pass (no regression). State new total and update CLAUDE.md
  reference count.

### Phase D — CLAUDE.md amendment

- [ ] **T-D01**: Amend CLAUDE.md §TD-VSDD-092 "L10 capability boundary" note per AC-005. Add new
  check to the §check-list. Verify the amended text does not itself contain volatile line cites
  or version pins (TD-VSDD-091/092 self-compliance).

---

### Merge gate

- [ ] **MERGE-GATE-SELF-PROBE**: `scripts/records-lint.sh --self-probe` exits 0 with all new cases
  passing and existing cases unaffected.
- [ ] **MERGE-GATE-CHECK-NAME-EXEMPT**: The new check's name token appears in `L9_CHECK_NAME_EXEMPT`.
  RG-005 confirms this.
- [ ] **MERGE-GATE-CORPUS-CLEAN**: `scripts/records-lint.sh --full-scan` including the new check
  exits 0 on the current workspace corpus (or documents known violations for triage).

---

## Previous Story Intelligence

N/A — first story in the ledger-citation integrity chain.

Related prior art:
- `S-MAINT-L11-GATE-001`: established the template for this class of records-lint extension (new
  check function, self-probe cases, CLAUDE.md amendment, check name in L9_CHECK_NAME_EXEMPT). The
  implementer MUST follow that story's shape — especially the CONFIG BLOCK discipline and the
  self-probe test framework. Check S-MAINT-L11-GATE-001 status before shipping: if L11 is not
  yet deployed in records-lint.sh, this story ships a new check into an undeployed surface. That
  is acceptable; both checks can ship in the same PR if the implementer chooses. Surface overlap
  with S-MAINT-L11-GATE-001 and S-MAINT-CONTENT-VERSION-GATE-001 (both extend records-lint.sh):
  note this in the PR description; coordinate with state-manager so parallel edits are not made
  to records-lint.sh simultaneously.
- `S-MAINT-ANTIPIN-SWEEP-001`: sweep story that depends on L11 being deployed; illustrates the
  dependency chain pattern if a future sweep story depends on this gate.

---

## Architecture Compliance Rules

1. **CONFIG BLOCK discipline (TD-VSDD-092 pattern):** The finding-ID pattern constant belongs in
   the CONFIG BLOCK section of `scripts/records-lint.sh` alongside `_L9_ARM*` pattern constants.
   Do not hardcode the pattern inline in the check function.

2. **Check name must be in `L9_CHECK_NAME_EXEMPT` before PR merge.** The check name token
   (whatever the implementer designates) must be added to `L9_CHECK_NAME_EXEMPT` so future staged
   additions that reference the check by name are not flagged as line cites by L9 arm-5.
   Anchor: AC-005 / RG-005. This MUST is anchored to AC-005 and RG-005.

3. **Corpus-audit mode, not staged-diff (AC-002):** This check runs on the full corpus, not only
   staged additions, because falsification can exist in pre-existing ledger rows. Distinguish
   from L9/L11 (staged-diff ratchet) and L10 (corpus-audit, same pattern as this story).

4. **No prism crate modifications.** No file under `crates/` is touched.

5. **No STATE.md edits.** State-manager owns STATE.md.

6. **POL-29 TD-VSDD-097 three-dimension sweep (for the CLAUDE.md amendment):**
   (a) Sibling pair: TD-VSDD-092 is co-documented with TD-VSDD-091 in CLAUDE.md — amend both
       in the same commit if the §check-list structure requires changes in both sections.
   (b) Downstream copy target: SESSION-HANDOFF.md §Standing Rules may reference TD-VSDD-092;
       sweep for staleness if the capability boundary note changes form.
   (c) Mandate anchor: any MUST added to the amended sections cites this story (S-MAINT-LEDGER-CITE-GATE-001)
       plus the specific AC that enforces it.

---

## Library and Framework Requirements

| Library/Tool | Version/Source | Purpose |
|-------------|---------------|---------|
| `scripts/records-lint.sh` | Project-local | Gate to extend |
| `bash` | System bash (≥ 3.2) | Script runtime; no new dependency |
| `git` | System git | Worktree-index-aware diff queries (for --self-probe infrastructure) |
| `grep` | System grep | Finding-ID citation cross-reference |

No external dependencies. No changes to Cargo.toml files.

---

## File Structure Requirements

### Files to MODIFY

| File | Change |
|------|--------|
| `scripts/records-lint.sh` | Add finding-ID pattern constant, check function, positive-coverage emission, self-probe cases, check name in `L9_CHECK_NAME_EXEMPT`, update `--self-probe` expected total |
| `CLAUDE.md` | Extend §TD-VSDD-092 check list and capability boundary note |

### Files NOT to modify

| File | Reason |
|------|--------|
| Any `crates/**` file | No Rust code changes |
| `.factory/**` files | State-manager handles .factory/ commits |
| `scripts/records-lint.sh` L9/L11 check bodies | Out of scope; do not modify existing check logic |

---

## Forbidden Dependencies

No new shell dependencies beyond standard POSIX utilities already used by `scripts/records-lint.sh`
(`awk`, `grep`, `sed`, `git`, `mktemp`). Do not introduce `perl`, `python`, or external binaries
to preserve the zero-external-dependency property of the script.

---

## Changelog

| Version | Date | Author | Summary |
|---------|------|--------|---------|
| 1.0 | 2026-07-31 | story-writer | FB105 — initial story creation. Codifies Lesson 113 (index ledger-content falsification, 5 recurrences). Adds records-lint ledger-citation content cross-reference check: PHANTOM finding-ID citations in BC-INDEX/ARCH-INDEX/VP-INDEX ledger rows. Corpus-audit mode with positive-coverage emission (N > 0 scanned rows required). 5 ACs, 5 RG self-probe tests, CLAUDE.md amendment to TD-VSDD-092 capability boundary note. status: draft; behavioral_contracts: [] pending PO authorship per S-7.01. |
