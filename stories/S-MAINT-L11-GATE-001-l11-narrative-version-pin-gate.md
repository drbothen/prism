---
document_type: story
story_id: S-MAINT-L11-GATE-001
title: "Add L11 Narrative-Version-Pin Ban to records-lint.sh and Extend TD-VSDD-091/TD-VSDD-092"
wave: maintenance
epic_id: maintenance
priority: P1
status: draft
version: "1.1"
updated: "2026-07-30"
level: "L2"
producer: story-writer
timestamp: "2026-07-30T00:00:00Z"
tdd_mode: strict
# tdd_mode: strict — this story adds executable gate logic to scripts/records-lint.sh.
# Red Gate tests are --self-probe cases that FAIL before L11 is implemented (the check
# does not yet exist and would return 0/pass on violations). They PASS after implementation.
# Standard Rust todo!() discipline does not apply (no Rust crates touched), but the
# --self-probe test framework serves the identical TDD purpose.
subsystems: []
# Cross-cutting governance toolchain: no single subsystem owns it.
crates_touched: []
target_module: "scripts/records-lint.sh, CLAUDE.md"
capabilities: []
behavioral_contracts: []
# BC status: pending PO authorship.
# TD-VSDD-091/TD-VSDD-092 and POL-39 are governance disciplines, not behavioral contracts.
# A product-owner BC covering governance-tool correctness would need to be authored before
# this story can reach status: ready (S-7.01 gate).
verification_properties: []
holdout_scenarios: []
# POL-35 holdout_gate_infra_only_exemption applies: behavioral_contracts: [] (pure toolchain
# scope). holdout_scenarios: [] is compliant for this story.
depends_on: []
blocks:
  - S-MAINT-CAPREF-SWEEP-001
  - S-MAINT-ANTIPIN-SWEEP-001
  - S-MAINT-ANTIPIN-SWEEP-002
# blocks justification:
#   S-MAINT-CAPREF-SWEEP-001: the gate must be in place before any sweep story ships
#   so no new pins accumulate during in-flight restructure work.
#   S-MAINT-ANTIPIN-SWEEP-001 and SWEEP-002: the L11 gate is their primary acceptance
#   criterion. Both depend on L11 being deployed to verify their completion.
points: 5
estimated_days: 1.5
risk: LOW
# Risk justification:
#   All changes are toolchain-only (bash script + CLAUDE.md). No production Rust code
#   touched. Primary risk is false-positive L11 matches in changelog rows or frontmatter
#   fields; the section-range exemption mechanism mitigates this. AC-003 requires explicit
#   section-range computation (not a line-shape heuristic) to control false-positive rate.
assumption_validations: []
risk_mitigations: []
tags:
  - toolchain
  - governance-gate
  - td-vsdd-091
  - td-vsdd-092
  - pol-39
---

# S-MAINT-L11-GATE-001: Add L11 Narrative-Version-Pin Ban to records-lint.sh and Extend TD-VSDD-091/TD-VSDD-092

## Narrative

As a records-discipline maintainer, I want `scripts/records-lint.sh` to gain a new check L11
that bans narrative version pins (e.g., `BC-2.01.001 vX.Y` — where X.Y stands for any version number) from staged additions to `.factory/`
files — ratchet-scoped and worktree-index-aware, with proper exemptions for changelog rows,
frontmatter `version:` fields, and index-tier files — and I want `CLAUDE.md` updated to document
L11 as part of TD-VSDD-091 and TD-VSDD-092, so that POL-39 (`anti_volatile_pin_versions`,
HIGH, being registered concurrently by spec-steward) is mechanically enforced from day one and
the sweep stories (CAPREF, ANTIPIN-001, ANTIPIN-002) can verify their own completeness using the gate.

## Background

TD-VSDD-091 was amended on 2026-07-24 to retire the "pass-report changelogs" exception for
volatile line cites. The governance principle being extended here — **narrative prose must not pin
the version of a referenced artifact** — is the natural complement: cite the artifact's ID and a
durable section anchor (e.g., `BC-2.01.001 §Postconditions`), not its current version number.
Version numbers decay on every amendment; durable section anchors do not.

This extension is being registered as POL-39 (`anti_volatile_pin_versions`, HIGH) concurrently
with this story. Four-tier exception boundary:

1. **Banned:** narrative prose version pins (`BC-2.01.001 vX.Y` form, where X.Y is any version number — cite ID + section anchor instead)
2. **Exempt:** index tier (BC-INDEX, ARCH-INDEX, VP-INDEX, STORY-INDEX — gated by L10)
3. **Exempt:** `## Changelog` section rows (version progression is the point of those rows)
4. **Exempt:** frontmatter `version:` field (canonical source of artifact version)

The L11 check mirrors L9 architecturally: ratchet-scoped to staged additions, queries both
the main project index AND the `.factory/` worktree index directly. The L9 worktree bypass gap
(inoperative from introduction until 2026-07-24 because the main project index is blind to
`.factory/` additions) was the most significant operational failure in this gate's history.
L11 MUST NOT repeat it. Explicitly: at least one self-probe case MUST prove that L11 fires
on a REAL staged `.factory/` addition, not only on a synthetic temp repo, demonstrating that
"probe-passes ≠ gate-fires" cannot recur silently.

## Behavioral Contracts

| BC | Title | Version | Scope in This Story |
|----|-------|---------|---------------------|
| (none) | — | — | No behavioral contracts yet. See BC status comment in frontmatter. Story MUST remain `status: draft` until a governance-integrity BC is authored and anchored. |

## Acceptance Criteria

### AC-001 — L11 pattern correctly detects narrative version pins in staged additions
After implementation, running `scripts/records-lint.sh` (staged-diff mode) against a staged
addition containing a version pin of the form `BC-2.01.001 v(digits).(digits)` (or equivalent `(BC|VP|ADR|DI|CAP|HS|TD)-[A-Z0-9.-]+\s+v[0-9]+\.[0-9]+`
form) in narrative prose produces an L11 FAIL exit code and a human-readable error identifying
the file and the offending cite. The pattern MUST require: (a) a recognized artifact-ID prefix
followed by an ID component, (b) a space, and (c) a `v`-prefixed version number. The pattern
MUST NOT match bare version numbers without an artifact-ID prefix.
(traces to POL-39 `anti_volatile_pin_versions` §enforcement scope and TD-VSDD-092 §check-list;
verified by RG-001)

### AC-002 — L11 is ratchet-scoped to staged additions and queries both indexes
L11 operates ONLY on staged additions (new `+` lines in the diff), not pre-existing lines —
mirroring the L9 ratchet design. Pre-existing version pins are grandfathered until the sweep
stories address them. L11 MUST query BOTH the main project git index (via
`git -C WORKSPACE_ROOT diff --cached`) AND the `.factory/` worktree's own index (via
`git -C WORKSPACE_ROOT/.factory diff --cached`), combining both diffs before scanning.
Failure to query the `.factory/` worktree index directly causes silent bypass identical to
the L9 operational gap of 2026-07-24.
(traces to TD-VSDD-092 §WORKTREE NOTE pattern established by L9; verified by RG-002 and RG-006)

### AC-003 — Changelog section rows are correctly exempted via real section-range computation
When a staged addition introduces a line containing a version pin AND that line is inside a
`## Changelog` (or `## Version History`) section of the target file, L11 does NOT flag it.
The exemption mechanism MUST use real section-range computation (determining whether a line
falls within a `## Changelog` section boundary), NOT a line-shape heuristic such as "if the
line matches the table row pattern `| N.M | ...`." A version pin appearing mid-sentence
in changelog prose must also be exempted by the section-range check.
(traces to TD-VSDD-092 §ratchet-scoping and the POL-39 four-tier exception boundary;
verified by RG-003)

### AC-004 — Frontmatter `version:` fields and index-tier filenames are correctly exempted
L11 MUST NOT flag: (a) the frontmatter `version:` field itself (`version: "1.5"` or
`version: 1.5` within the YAML frontmatter block); (b) any line in the frontmatter block
(between the opening and closing `---` delimiters); (c) files whose basename matches
`BC-INDEX.md`, `ARCH-INDEX.md`, `VP-INDEX.md`, or `STORY-INDEX.md` (index tier, gated by
L10 instead).
(traces to POL-39 §four-tier exception boundary; verified by RG-004 and RG-005)

### AC-005 — `--self-probe` extended with L11 cases; new expected total stated
The `--self-probe` mode gains at minimum 7 new pass/fail cases covering: (a) an L11 violation
case (version pin of the form `BC-NNN vN.M` in staged addition → FAIL), (b) a clean case (no version pin
→ PASS), (c) a changelog-section exemption case (version pin inside `## Changelog` section
→ PASS), (d) a frontmatter exemption case (`version: "1.5"` field → PASS), (e) an index-tier
filename exemption case (file is BC-INDEX.md → PASS), (f) the real-worktree case described
in AC-006, and (g) the full-scan CLAUDE.md inclusion case described in AC-007. The
implementation states the new `--self-probe` expected total (previous total 34;
new total ≥ 41) and verifies that total via a `--self-probe` run before declaring done.
(traces to TD-VSDD-092 §self-probe discipline and CLAUDE.md §MECHANICAL-GATE-COVERAGE-PARITY;
verified by RG-001 through RG-007)

### AC-006 — At least one self-probe case proves L11 fires on a real staged `.factory/` addition
Because the L9 gate was silently inoperative for its first worktree-bypass period (self-probe
passed 6/6 the entire time while the gate never fired on a real `.factory/` commit), L11 MUST
include an explicit integration-level test that stages an actual `.factory/` file addition using
the real `.factory/` worktree and confirms that L11 catches the violation. This test differs
from the synthetic-temp-repo approach used by other self-probe cases: it uses the actual
`WORKSPACE_ROOT/.factory` worktree path so that the worktree-index code path is exercised under
real conditions. If the real worktree is not available in the test environment, the case must
be explicitly skipped with a human-readable message (rather than silently returning 0).
(traces to the "probe-passes ≠ gate-fires" lesson documented in CLAUDE.md §TD-VSDD-092;
this AC exists because the prior gap was structurally undetectable without a real-worktree probe;
verified by RG-006)

### AC-007 — CLAUDE.md updated: TD-VSDD-091 and TD-VSDD-092 extended to document L11
`CLAUDE.md` is amended in the same PR as the `records-lint.sh` changes:
- `§Operational Discipline TDs §TD-VSDD-091` is retitled/extended to cover narrative version
  pins alongside volatile line cites, noting that the four-tier exception boundary for L11
  mirrors the four-tier boundary that TD-VSDD-091 established for L9
- `§Operational Discipline TDs §TD-VSDD-092` is extended to describe L11 in the check list
  alongside L1/L7/L9/L10, including: pattern description, ratchet scope, exemption arms, and
  the `--self-probe` total
- Both sections reference POL-39 (`anti_volatile_pin_versions`) as the policy being enforced
- TD-VSDD-091 and TD-VSDD-092 in CLAUDE.md MUST NOT themselves contain version pins; CLAUDE.md
  is in the `--full-scan` scope so any version pin introduced during the amendment will be
  caught by RG-007
(traces to the CLAUDE.md amendment obligation stated in this story's background section;
verified by RG-007)

## Red Gate Tests

All 7 RG items are self-probe test cases that FAIL before L11 is implemented (the check function
does not exist; any violation-test on a non-existent check returns 0/pass, producing false-green),
and PASS after. RG-006 is an integration-level test using the real `.factory/` worktree; RG-007
is a full-scan case using a synthetic repo containing CLAUDE.md; others use synthetic temp repos
per the existing self-probe pattern.

- [ ] **RG-001** (`test_l11_violation_artifact_id_vN`): self-probe case — a staged addition
  containing a version pin of the form `BC-2.01.001 v(digit).(digit)` in narrative prose
  triggers L11 FAIL. Establishes the primary detection arm. The self-probe test file uses
  the actual matched form; only the story description uses the illustrative placeholder.

- [ ] **RG-002** (`test_l11_clean_no_version_pin`): self-probe case — a staged addition
  referencing `BC-2.01.001 §Postconditions` (no `v`-pin) passes L11. Prevents false-positive
  on the correct citation form that the sweep stories will use.

- [ ] **RG-003** (`test_l11_exempt_changelog_section`): self-probe case — a staged addition
  that places a version pin of the form `BC-2.01.001 v(digit).(digit)` inside a `## Changelog`
  table row passes L11. Validates the section-range exemption mechanism.

- [ ] **RG-004** (`test_l11_exempt_frontmatter_version_field`): self-probe case — a staged
  addition whose `version: "1.5"` appears in the frontmatter block (between `---` delimiters)
  passes L11. Prevents false-positive on the canonical frontmatter pattern.

- [ ] **RG-005** (`test_l11_exempt_index_tier_filename`): self-probe case — a staged addition
  to `BC-INDEX.md` containing a version pin of the form `BC-2.01.001 v(digit).(digit)` passes
  L11 (index tier is exempt; L10 handles it). Prevents the gate from conflicting with the
  legitimate index-maintenance workflow.

- [ ] **RG-006** (`test_l11_fires_real_factory_worktree`): integration case — a real staged
  `.factory/` addition containing a version pin triggers L11 FAIL via the `.factory/` worktree
  index code path. This case explicitly exercises the worktree-aware diff path rather than a
  synthetic repo, proving the "probe-passes ≠ gate-fires" gap cannot recur silently for L11.
  If the `.factory/` worktree is unavailable in the test environment, the case logs
  "SKIP — .factory/ worktree not present; real-worktree probe cannot run" and does NOT return 0.

- [ ] **RG-007** (`test_l11_full_scan_includes_claude_md`): self-probe case — a synthetic repo
  where `CLAUDE.md` contains a narrative version pin of the form `BC-2.01.001 v(digit).(digit)`
  in narrative prose is scanned via `scripts/records-lint.sh --full-scan`; L11 returns FAIL.
  Confirms that CLAUDE.md (a project-root governance document, not a `.factory/` file) is in
  the `--full-scan` scope and is not excluded by any path filter. FAILS before `run_l11`'s
  `--full-scan` arm includes CLAUDE.md; PASSES after. This mechanizes AC-007: any version pin
  introduced in the CLAUDE.md amendment during T-D01/T-D02/T-D03 will be caught by this gate
  rather than relying on manual adversarial review. Preferred location: inside the `--self-probe`
  case block in `scripts/records-lint.sh`.

**Red Gate density check (BC-5.38.001):** 7 Red Gate tests (RG-001 through RG-007) anchor to
7 acceptance criteria. Density verification (`RED_TESTS * 2 >= (TOTAL_NEW_TESTS − EXEMPT_TESTS)`)
is computed at dispatch by the orchestrator per `per-story-delivery.md §Red Gate Density Check`
and BC-5.38.002/BC-5.38.003. No authoring-time ratio is pre-computed here.

## Architecture Mapping

| Component | Location | Pure/Effectful |
|-----------|----------|----------------|
| `scripts/records-lint.sh` L11 check | `scripts/records-lint.sh` | Pure (read-only scan; exit code is the gate) |
| `scripts/records-lint.sh --self-probe` L11 cases | same file | Effectful (creates temp git repos in `$TMPDIR`) |
| `CLAUDE.md` §TD-VSDD-091 + §TD-VSDD-092 amendment | project root `CLAUDE.md` | Pure (governance document text edit) |

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Version pin appears inside a fenced code block (` ``` `) in narrative prose | L11 flags it — the content inside code blocks is still a navigational cite in records context; code-block fencing does not confer changelog-section exemption. If the intent is to SHOW the banned form as an example (as in this very story), wrap in a prose explanation that the example is illustrative. |
| EC-002 | Artifact ID prefix is lowercase (`bc-2.01.001 v1.5`) | The pattern SHOULD match case-insensitively or include lowercase variants. False-negatives from missed lowercase forms are a P1; implementer decides match strategy. |
| EC-003 | A `BC-NNN` reference followed by a version number that is NOT prefixed with `v` (e.g., `BC-2.01.001 1.5`) | Not flagged by L11 — the pattern requires `v`-prefix. This is acceptable; the `v`-prefix form is the dominant violation pattern in the corpus. |
| EC-004 | Version pin appears in the `# BC status: pending PO authorship` YAML comment within frontmatter | This is inside the frontmatter block (between `---` delimiters) and is exempted by AC-004(b). |
| EC-005 | L11 is run via `--l9-only` flag (which currently skips non-L9 checks) | Determine whether to add an `--l11-only` flag or to treat L11 as part of the same staged-diff path as L9. Either approach is valid; the implementer chooses and documents the CLI behavior in the CLAUDE.md amendment. |
| EC-006 | A `MUST` statement in this story references story + AC but no RG test | This story follows POL-29 9c: every `MUST` in an AC body traces to an RG item (verified by RG-NNN in the AC). `MUST` statements in prose sections are anchored to the AC that governs that surface (e.g., AC-002 governs worktree-index requirements). No unanchored `MUST` appears in this story. |

## Token Budget Estimate

| Artifact | Estimated Tokens | Notes |
|----------|-----------------|-------|
| This story spec | ~8,000 | |
| `scripts/records-lint.sh` (full text) | ~15,000 | Required to understand L9 pattern, --self-probe structure, and CONFIG BLOCK |
| `CLAUDE.md` relevant sections (TD-VSDD-091, TD-VSDD-092) | ~5,000 | Load only the relevant §Operational Discipline TDs sections |
| `.factory/policies.yaml` (POL-39 context) | ~2,000 | Skim for POL-38/39 context |
| **Total per implementation session** | ~30,000 | Well within 20–30% of agent context window |

Context management: implement L11 in records-lint.sh first (a single file), verify via --self-probe,
then amend CLAUDE.md. Do not load both simultaneously.

## Tasks

### Phase A — Write failing self-probe cases (Red Gate — do BEFORE implementation)

- [ ] **T-A01** (RG-001 setup): In `scripts/records-lint.sh`, add the `run_l11` function stub
  with a permanent-fail body (`return 1`) and add RG-001 self-probe case that EXPECTS `run_l11`
  to fail on a violation. Confirm `--self-probe` outputs `L11-violation probe FAIL (permanent-fail body)`.

- [ ] **T-A02** (RG-002 through RG-007 setup): Add remaining 6 self-probe cases against the
  permanent-fail stub. Confirm all 7 new cases fail in the expected direction before implementation
  (RG-001, RG-006, and RG-007 should show FAIL; RG-002/003/004/005 should show false-FAIL because
  the stub always fails). Record the new `--self-probe` expected-fail count.

### Phase B — Implement L11 check (Green — make failing tests pass)

- [ ] **T-B01**: Define the `_L11_VERSION_PIN_PATTERN` constant using the
  `(BC|VP|ADR|DI|CAP|HS|TD)-[A-Z0-9][A-Z0-9.-]*\s+v[0-9]+\.[0-9]+` family (implementer refines
  exact form). Add to the pattern-constants section alongside `_L9_ARM*`.

- [ ] **T-B02**: Implement `_l11_in_changelog_section()` helper — given a file path and a line
  string, returns 0 (true) if the line falls within a `## Changelog` or `## Version History`
  section. Uses section-range computation (scan file from top, track last `##` heading seen).

- [ ] **T-B03**: Implement `_l11_in_frontmatter_block()` helper — returns 0 (true) if the line
  is within the YAML frontmatter block (between the first and second `---` delimiters).

- [ ] **T-B04**: Implement `run_l11()` — mirrors `run_l9()` structure: accumulates combined diff
  from main project index AND `.factory/` worktree index, scans `+` lines, applies exemptions
  via helpers from T-B02/T-B03, skips index-tier filenames, reports violations.

- [ ] **T-B05**: Integrate `run_l11` into the main gate execution path alongside `run_l9`. Ensure
  L11 runs in default mode (staged-diff), `--full-scan` mode, and is skipped by `--l9-only`.
  In `--full-scan` mode, CLAUDE.md MUST be included in the scan scope — project-root governance
  documents are not excluded from full-scan (only index-tier filenames are exempt per AC-004(c)).
  This requirement is mechanized by RG-007. Anchor: AC-007/RG-007.
  Decide whether to add `--l11-only` flag (document the decision).

### Phase C — Verify self-probe passes

- [ ] **T-C01**: Run `scripts/records-lint.sh --self-probe`. Confirm all 7 new L11 cases pass.
  Confirm existing 34 cases still pass (no regression). State the new total (≥ 41) and update
  the CLAUDE.md reference count.

- [ ] **T-C02** (RG-006 real-worktree verification): If the `.factory/` worktree is present
  (normal development environment), stage a real `.factory/` file with a version pin, run
  `scripts/records-lint.sh`, confirm L11 FAIL. Unstage and confirm L11 PASS. Document the
  result in the PR description.

### Phase D — CLAUDE.md amendment

- [ ] **T-D01**: Amend `CLAUDE.md §Operational Discipline TDs §TD-VSDD-091` to state that the
  volatile-cite ban now covers both line-number cites AND narrative version pins, with the
  four-tier exception boundary for version pins (banned / index-tier exempt / changelog-section
  exempt / frontmatter exempt).

- [ ] **T-D02**: Amend `CLAUDE.md §Operational Discipline TDs §TD-VSDD-092` to add L11 to the
  check list alongside L1/L7/L9/L10. Include: pattern description, ratchet scope, exemption arms,
  new `--self-probe` total.

- [ ] **T-D03**: Reference POL-39 (`anti_volatile_pin_versions`) in both amended sections.
  Verify neither amended section itself contains a version pin (self-compliance check).

### Merge gate

- [ ] **MERGE-GATE-SELF-PROBE**: `scripts/records-lint.sh --self-probe` exits 0 with ≥ 41 expected
  cases, all passing.
- [ ] **MERGE-GATE-NO-L11-REGRESSION**: `scripts/records-lint.sh` on the PR diff exits 0 (no L11
  violations in the CLAUDE.md amendment or the story files themselves).
- [ ] **MERGE-GATE-CLAUDE-MD-SELF-COMPLIANT**: RG-007 (`test_l11_full_scan_includes_claude_md`)
  passes, confirming CLAUDE.md is in the `--full-scan` scope and any version pin introduced
  during the CLAUDE.md amendment (T-D01/T-D02/T-D03) is mechanically caught by L11.

## Previous Story Intelligence

First story in the POL-39 / narrative-version-pin governance chain. No predecessor in this epic.

Related prior art:
- `S-MAINT-VOLATILE-CITE-001` / `S-MAINT-VOLATILE-CITE-002`: structurally similar (adds check
  to records-lint.sh, then sweeps corpus). Key lesson: the L9 worktree bypass went undetected
  because --self-probe uses synthetic temp repos; the real worktree path was never exercised.
  AC-006 and RG-006 exist specifically to close this repeat-failure vector for L11.
- `S-MAINT-REQWEST-RUSTLS-GATE-001`: pattern for adding a CI gate that enforces a policy with
  ratchet scoping so pre-existing violations are grandfathered.

## Architecture Compliance Rules

1. **Worktree-index discipline (TD-VSDD-092 §WORKTREE NOTE):** `.factory/` is a separate git
   worktree. Any check that scans staged additions MUST query `git -C WORKSPACE_ROOT/.factory
   diff --cached` in addition to `git -C WORKSPACE_ROOT diff --cached`. The self-probe MUST
   exercise the real worktree path (AC-006/RG-006) — not only a synthetic temp repo.

2. **Ratchet-scoped by default:** L11, like L9, runs only on staged additions in default mode.
   Pre-existing corpus violations are not immediately blocking — they are addressed by
   S-MAINT-ANTIPIN-SWEEP-001 and S-MAINT-ANTIPIN-SWEEP-002. The gate prevents NEW pins from
   accumulating from merge day forward.

3. **Section-range computation required for changelog exemption (AC-003):** A line-shape
   heuristic (e.g., "flag only if not a table row") is INSUFFICIENT. The exemption must
   determine section membership by scanning the file from the top and tracking the most recent
   `## ` heading. If the most recent heading is `## Changelog` or `## Version History`, the
   line is exempt regardless of its form.

4. **CONFIG BLOCK discipline:** The `_L11_VERSION_PIN_PATTERN` constant belongs in the CONFIG
   BLOCK pattern-constant section (alongside `_L9_ARM1`..`_L9_ARM5`). Do not hardcode the
   pattern inline in `run_l11`. This makes the pattern visible and auditable.

5. **CLAUDE.md is a governance document, not a spec:** Changes to CLAUDE.md do not require
   product-owner routing. The architect or orchestrator may amend CLAUDE.md for governance
   toolchain changes. The amendment MUST NOT introduce version pins (self-compliance).

6. **POL-29 TD-VSDD-097 three-dimension sweep (for the CLAUDE.md amendment):**
   (a) Sibling pair: TD-VSDD-091 and TD-VSDD-092 are co-documented in CLAUDE.md — amend both
       in the same commit. Neither is the "twin" of a different file.
   (b) Downstream copy target: SESSION-HANDOFF.md §Standing Rules may reference these TDs —
       sweep and update if version pins appear there.
   (c) Mandate anchor: any `MUST` added to the amended sections cites this story (S-MAINT-L11-GATE-001)
       plus the specific AC (AC-001..AC-007) that enforces it.

## Library and Framework Requirements

| Library/Tool | Version/Source | Purpose |
|-------------|---------------|---------|
| `scripts/records-lint.sh` | Project-local | Gate to extend |
| `bash` | System bash (≥ 3.2) | Script runtime; no new dependency |
| `git` | System git | Worktree-index-aware diff queries |

No external dependencies. No changes to Cargo.toml files.

## File Structure Requirements

### Files to MODIFY

| File | Change |
|------|--------|
| `scripts/records-lint.sh` | Add `_L11_VERSION_PIN_PATTERN` constant, `_l11_in_changelog_section()` helper, `_l11_in_frontmatter_block()` helper, `run_l11()` function, integrate into gate dispatch, extend `--self-probe` with ≥ 6 new L11 cases |
| `CLAUDE.md` | Extend §TD-VSDD-091 and §TD-VSDD-092 to document L11; reference POL-39 |

### Files NOT to modify

| File | Reason |
|------|--------|
| Any `crates/**` file | No Rust code changes |
| `.factory/**` files | State-manager handles .factory/ commits; this story ships via main-repo PR |
| `scripts/check-non-exhaustive*.sh` or `scripts/check-non-exhaustive*.py` | Out of scope |

## Forbidden Dependencies

No new shell dependencies beyond standard POSIX utilities already used by records-lint.sh
(`awk`, `grep`, `sed`, `git`, `mktemp`). Do not introduce `perl`, `python`, or external binaries
into records-lint.sh to preserve the zero-external-dependency property.

## Dependency Graph Edges

```
S-MAINT-L11-GATE-001 (this story)
  depends_on: []
  blocks:
    → S-MAINT-CAPREF-SWEEP-001  (gate must be deployed before CAPREF ships)
    → S-MAINT-ANTIPIN-SWEEP-001 (primary acceptance criterion uses L11)
    → S-MAINT-ANTIPIN-SWEEP-002 (primary acceptance criterion uses L11)
```

## Version History

| Version | Date | Change Summary |
|---------|------|----------------|
| 1.1 | 2026-07-30 | FB101 story leg — close F-WASE-P71-LOW-002. AC-007 mechanized: replaced "verified by manual adversarial review of the CLAUDE.md diff" with "verified by RG-007". Added RG-007 (`test_l11_full_scan_includes_claude_md`): self-probe case confirming CLAUDE.md is in the `--full-scan` scope by staging a synthetic CLAUDE.md containing a version pin and asserting L11 FAIL. Updated AC-005: "at minimum 6 new pass/fail cases" → "at minimum 7 new pass/fail cases"; self-probe expected total ≥ 40 → ≥ 41. Updated Red Gate density check: "6 Red Gate tests (RG-001 through RG-006)" → "7 Red Gate tests (RG-001 through RG-007)". Updated RG preamble sentence. Updated T-A02: "remaining 5 self-probe cases" → "remaining 6 self-probe cases" (now covers RG-002 through RG-007). Updated T-B05: added CLAUDE.md full-scan inclusion MUST with RG-007 anchor. Updated T-C01: "all 6 new L11 cases" → "all 7"; "(≥ 40)" → "(≥ 41)". Updated MERGE-GATE-SELF-PROBE: ≥ 40 → ≥ 41. Updated MERGE-GATE-CLAUDE-MD-SELF-COMPLIANT: from manual adversarial review to RG-007 reference. POL-29 9a: no sibling-pair twin for L11-GATE-001. 9b: no downstream copy target affected. 9c: T-B05 new MUST carries AC-007/RG-007 anchor. |
| 1.0 | 2026-07-30 | Initial story creation. Adds records-lint.sh L11 check banning narrative version pins, ratchet-scoped and worktree-index-aware, with section-range changelog exemption, frontmatter exemption, and index-tier exemption. CLAUDE.md extended to document L11 under TD-VSDD-091 and TD-VSDD-092. Motivated by POL-39 (`anti_volatile_pin_versions`, HIGH) registration and measured corpus exposure of approximately 5,679 pre-existing narrative version pins across 219 files. Includes explicit AC-006/RG-006 to prevent repeat of L9 worktree-bypass silent-inoperability gap. |
