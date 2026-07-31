---
document_type: story
story_id: S-MAINT-CONTENT-VERSION-GATE-001
title: "records-lint Content-Change-Without-Version-Bump Check"
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
# tdd_mode: strict — this story adds a new check to scripts/records-lint.sh.
# Red Gate tests are --self-probe cases that FAIL before the check exists (no function →
# exit 0 on violations → false-green) and PASS after. Standard Rust todo!() discipline
# does not apply (no Rust crates touched); --self-probe serves the identical TDD purpose.
subsystems: []
crates_touched: []
target_module: "scripts/records-lint.sh, CLAUDE.md"
capabilities: []
behavioral_contracts: []
# BC status: pending PO authorship.
# Audit-trail completeness enforcement is a governance discipline, not a behavioral contract.
# This story MUST remain status: draft until a governance-integrity BC is authored.
verification_properties: []
holdout_scenarios: []
# POL-35 holdout_gate_infra_only_exemption applies: behavioral_contracts: [] (pure
# toolchain governance scope). holdout_scenarios: [] is compliant for this story.
depends_on:
  - S-MAINT-CHANGELOG-BACKFILL-001
# depends_on justification:
#   S-MAINT-CHANGELOG-BACKFILL-001 must ship first because the content-change-version-bump
#   check can only enforce on files that HAVE a ## Changelog section. Before
#   CHANGELOG-BACKFILL-001 ships, 67 story files lack the section — the check would either
#   skip those files (incorrect partial enforcement) or block on them with false-positive
#   "missing changelog section" errors. After CHANGELOG-BACKFILL-001 ships, all story
#   files have the section and the check enforces uniformly.
blocks: []
points: 8
estimated_days: 3.0
risk: MEDIUM
# Risk justification:
#   Detecting "content changed without version bump" requires comparing staged diff content
#   against the pre-staged version — a more complex operation than the ratchet-scoped
#   pattern checks used by L9 and L11. The primary technical risk is incorrectly
#   classifying a structural-only change (e.g., adding ## Changelog) as a content change.
#   AC-002 defines the structural-vs-normative distinction and the exemption paths.
#   The second risk is false-negatives on files where content and version change in
#   separate commits; the check only sees the currently staged diff.
assumption_validations: []
risk_mitigations: []
tags:
  - toolchain
  - governance-gate
  - td-vsdd-092
  - records-lint
  - audit-trail
---

# S-MAINT-CONTENT-VERSION-GATE-001: records-lint Content-Change-Without-Version-Bump Check

## Origin

**Process-gap finding:** Lesson 117 (D-2075, 2026-07-31). Gate capability boundary (fourth).

FB104 edited the §Out-of-Scope T2 row in `S-DEMO-QUERY-PUSHDOWN-001` — a normative content
change — and left the story at v2.8 with no new changelog row. Records-lint L1 returned green:
L1 asks "is the declared version consistent with the changelog top row?" and the answer was yes
(both said v2.8). L1 does NOT ask "did content change without a version bump?" The gap was
found by orchestrator independent verification (D-2075), not by any mechanical gate.

Lesson 117 documents this as the fourth gate capability boundary: L1 answers one question and
is blind to a related but different question. The current check suite covers:
- L1: declared version consistent with changelog top row
- L7: changelog rows in descending order
- L9/L11: volatile cites in staged additions
- L10: index row version ≠ artifact frontmatter version

The new check this story adds: "did staged content change in a versioned artifact without a
version bump AND without a new changelog row?" These three conditions together constitute an
audit-trail gap.

**Prerequisite dependency:** This check's enforcement scope is limited to files that HAVE a
`## Changelog` section. Before S-MAINT-CHANGELOG-BACKFILL-001 ships, 67 story files lack the
section and cannot be reliably enforced. After the backfill, the check enforces uniformly.

---

## Narrative

As a records-discipline maintainer, I want `scripts/records-lint.sh` to gain a new check that
detects staged content changes to versioned `.factory/` artifacts where neither the `version:`
frontmatter field changed NOR a new `## Changelog` row was added — producing a hard-block
error identifying the file and the content change type — and I want CLAUDE.md updated to
document this check as an extension to TD-VSDD-092 (the fourth gate capability boundary
note is revised to acknowledge partial resolution), so that the audit-trail completeness
gap documented in Lesson 117 is mechanically enforced at commit time.

---

## Background

**Why three conditions (content changed + version not bumped + no new changelog row)?**

Consider the cases:
1. Content changed, version NOT bumped, NO new changelog row → **AUDIT TRAIL GAP** (this check catches it)
2. Content changed, version NOT bumped, new changelog row added → L1 FAIL (existing check catches it — changelog top row version ≠ frontmatter version)
3. Content changed, version bumped, NO new changelog row → L1 FAIL (changelog top row still old version ≠ new frontmatter version)
4. Content changed, version bumped, new changelog row added → CLEAN (audit trail complete)

Only case 1 is currently uncaught. The new check specifically targets case 1.

**Defining "normative content change" for this check:**

A staged addition counts as a normative content change if:
- It adds or removes lines in the file body (outside frontmatter, outside `## Changelog`
  section, and outside `## Version History` section)
- It modifies a line in the file body outside those exempted sections

A staged change that ONLY adds/modifies lines inside `## Changelog` or `## Version History`
sections, or inside the frontmatter block (between `---` delimiters), is a structural or
records-tier change and MUST NOT be flagged (it would produce false-positives on the very
changelog entries this check requires).

**Ratchet scoping:** Like L9 and L11, this check operates on staged additions only. Pre-existing
content/version mismatches (the FB104 gap was found and corrected before this gate existed) are
grandfathered. Only new staged changes are checked. This prevents the gate from blocking
legitimate work on files that already had pre-existing content/version drift.

---

## Behavioral Contracts

| BC | Title | Version | Scope in This Story |
|----|-------|---------|---------------------|
| (none) | — | — | No behavioral contracts yet. Story MUST remain `status: draft` per S-7.01. This MUST is anchored to: AC-001 governs detection; BC authorship is a human gate. |

---

## Acceptance Criteria

### AC-001 — New check detects normative content change without version bump + changelog row
After implementation, `scripts/records-lint.sh` run against a staged diff where a versioned
`.factory/` artifact has:
- Staged additions or removals in the file body (outside frontmatter and `## Changelog` section), AND
- NO change to the `version:` frontmatter field, AND
- NO new row added to the `## Changelog` section
MUST produce a hard-block error identifying the file, the type of content change detected
(body text additions/removals), and the required fix (bump version and add changelog row).
(verified by RG-001)

### AC-002 — Exemptions: structural-only and records-tier changes do not trigger the check
The new check MUST NOT flag:
(a) A staged change that ONLY modifies lines inside the `## Changelog` or `## Version History`
    section (e.g., adding a new changelog row as part of a version bump).
(b) A staged change that ONLY modifies lines inside the frontmatter block (between `---`).
(c) A staged change where the `version:` field DID change in the same diff (the version bump
    is present; the check's third condition is not met).
(d) Files without a `## Changelog` section (pre-backfill grace period — the check MUST skip
    files that lack a changelog section entirely rather than flagging them, because the
    content-version-bump requirement cannot be enforced on files with no audit-trail surface;
    see §Background and `depends_on: [S-MAINT-CHANGELOG-BACKFILL-001]`).
(verified by RG-002 and RG-004)

### AC-003 — Check queries both the main project index and the `.factory/` worktree index
Like L9 and L11, this check MUST query BOTH `git -C WORKSPACE_ROOT diff --cached` AND
`git -C WORKSPACE_ROOT/.factory diff --cached` to obtain the staged diff. Failure to query
the `.factory/` worktree index directly causes silent bypass identical to the L9 operational
gap (2026-07-24). The check MUST emit "Scanned N versioned files in staged diff" (positive
coverage: if N=0 and the commit contains `.factory/` changes, the check MUST investigate
whether the worktree-index query is working).
(verified by RG-003)

### AC-004 — `--self-probe` extended with new check cases; expected total stated
The `--self-probe` mode gains at minimum 4 new pass/fail cases:
(a) A violation case: staged body addition to a versioned file, no version bump, no changelog
    row → FAIL.
(b) A pass case: staged body addition WITH version bump AND a new changelog row → PASS.
(c) An exemption case: staged addition ONLY to the `## Changelog` section → PASS.
(d) A worktree-index case: a real staged `.factory/` addition with a normative content change
    and no version bump triggers the check (mirrors L11's AC-006 / RG-006 real-worktree probe
    to prevent the "probe-passes ≠ gate-fires" gap from recurring).
The implementation states the new `--self-probe` expected total and verifies it before declaring done.
(verified by RG-001 through RG-004)

### AC-005 — CLAUDE.md updated: TD-VSDD-092 capability boundary note revised and new check documented
CLAUDE.md §TD-VSDD-092 is amended in the same PR:
- The "L1 gate capability boundary" note is updated: "L1 answers the question 'is the declared
  version consistent with the top changelog row?' It does NOT answer 'did content change without
  a version bump?' The content-change-version-bump check (see §check-list) partially addresses
  this boundary: it catches case 1 (content change + no version bump + no changelog row) in
  staged diffs, but NOT pre-existing drift in unstaged files."
- §check-list is extended to describe the new check: conditions, exemptions, ratchet scope,
  worktree-index awareness, positive-coverage emission.
- The new check name must be added to `L9_CHECK_NAME_EXEMPT` before the PR merges, per the
  CONFIG BLOCK naming convention. Anchor: AC-005. This MUST is anchored to AC-005.
(verified by RG-005)

---

## Red Gate Tests

All 5 RG items are --self-probe cases that FAIL before the new check is implemented and PASS
after. RG-004 uses the real `.factory/` worktree (mirrors L11-GATE-001 AC-006/RG-006 design).

- [ ] **RG-001** (`test_content_version_body_change_no_bump_fails`): self-probe case — a synthetic
  versioned `.factory/` file has a staged body-line addition, no `version:` change, no new
  changelog row. Triggers a hard-block error. FAILS before new check exists (no function →
  exit 0); PASSES after implementation.

- [ ] **RG-002** (`test_content_version_with_bump_and_row_passes`): self-probe case — a synthetic
  file has a staged body-line addition AND a `version:` change AND a new changelog row.
  Passes the new check without error. Establishes the clean-state non-regression baseline.
  FAILS before implementation; PASSES after.

- [ ] **RG-003** (`test_content_version_changelog_only_change_exempt`): self-probe case — a
  staged addition that ONLY adds a row to the `## Changelog` section (no body changes)
  passes the new check without error. Prevents the check from blocking the very changelog
  entries it requires. FAILS before implementation; PASSES after.

- [ ] **RG-004** (`test_content_version_fires_real_factory_worktree`): integration case — a real
  staged `.factory/` file with a normative body-line addition and no version bump triggers
  the check via the `.factory/` worktree index code path. Mirrors L11's real-worktree probe
  to prevent "probe-passes ≠ gate-fires" recurrence. If the `.factory/` worktree is unavailable
  in the test environment, the case logs "SKIP — .factory/ worktree not present; real-worktree
  probe cannot run" and does NOT return 0.

- [ ] **RG-005** (`test_content_version_check_name_exempt_l9`): self-probe case — a staged
  addition referencing the new check's designated name token passes L9 arm-5 without flagging.
  Confirms the check name was correctly added to `L9_CHECK_NAME_EXEMPT`. FAILS before the
  exempt entry is added; PASSES after.

**Red Gate density check (BC-5.38.001):** 5 Red Gate tests (RG-001 through RG-005) anchor to
5 acceptance criteria (AC-001 through AC-005). Density ratio: 5 / 5 = 1.0, satisfying
BC-5.38.001. Density validation at dispatch time per `per-story-delivery.md §Red Gate Density
Check`.

---

## Architecture Mapping

| Component | Location | Pure/Effectful |
|-----------|----------|----------------|
| New content-change-version-bump check function | `scripts/records-lint.sh` | Pure (read-only scan; exit code is gate) |
| `--self-probe` new cases | same file | Effectful (creates temp git repos in `$TMPDIR`; RG-004 uses real worktree) |
| CLAUDE.md §TD-VSDD-092 amendment | project root `CLAUDE.md` | Pure (governance document text edit) |

---

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | File has a `## Changelog` section but it's inside a code block | Do NOT count this as a real changelog section for exemption purposes. The section-range detection must use document-level heading detection, not a raw string match. |
| EC-002 | File has BOTH `## Changelog` and `## Version History` sections | Either section suffices. The check should treat them as equivalent. |
| EC-003 | A staged diff modifies lines in BOTH the body AND the `## Changelog` section | If the diff includes a new `## Changelog` row in addition to the body changes, check whether the `version:` field also changed. If yes (case 4 above) → PASS. If no → L1 will catch it (case 2 above); the new check should not double-flag. |
| EC-004 | File does not have a frontmatter `version:` field | The check cannot determine if a version bump occurred (no version to compare). Skip the file (exempt, same rationale as files without `## Changelog` section). Note this for corpus cleanup follow-up. |
| EC-005 | A multi-file staged commit where file A has a content change + version bump but file B (related) has a content change + no bump | Each file is evaluated independently. File B triggers a hard-block error; file A does not. The check does not infer "related files share a version." |

---

## Token Budget Estimate

| Artifact | Estimated Tokens | Notes |
|----------|-----------------|-------|
| This story spec | ~7,000 | |
| `scripts/records-lint.sh` (full text) | ~18,000 | Required to understand L9 and L11 diff-query structure, --self-probe pattern, CONFIG BLOCK |
| CLAUDE.md §TD-VSDD-092 section | ~3,000 | Load only the relevant section |
| S-MAINT-L11-GATE-001 (structural reference for worktree-probe pattern) | ~8,000 | Specifically: AC-006/RG-006 real-worktree probe pattern |
| **Total per implementation session** | ~36,000 | Within 20–30% of agent context window (single session feasible) |

Context management: implement the check function first (referencing L9/L11 diff-query structure),
verify via --self-probe, then amend CLAUDE.md.

---

## Tasks

### Phase A — Write failing self-probe cases (Red Gate — do BEFORE implementation)

- [ ] **T-A01** (RG-001, RG-003 setup): Add new check function stub with permanent-fail body.
  Add RG-001 (violation case) and RG-003 (changelog-section-only change exempt) self-probe
  cases against the stub. Confirm `--self-probe` reports expected failures.

- [ ] **T-A02** (RG-002, RG-005 setup): Add RG-002 (with-bump-and-row passes) and RG-005
  (check-name L9 exempt) cases. Confirm all 4 non-RG-004 cases fail as expected (stub
  permanent-fail causes RG-002 to fail — correct pre-impl state). Record expected-fail count.

### Phase B — Implement the check (Green — make failing tests pass)

- [ ] **T-B01**: Decide check name token (CONFIG BLOCK naming convention) and add to:
  (a) the CONFIG BLOCK pattern-constants section, and (b) `L9_CHECK_NAME_EXEMPT`.

- [ ] **T-B02**: Implement the section-range helpers — `_content_check_in_changelog_section()`
  (returns true if a line is in `## Changelog` or `## Version History`) and
  `_content_check_in_frontmatter()` (returns true if a line is in the frontmatter block).
  These mirror the helpers in S-MAINT-L11-GATE-001 — reuse that implementation if L11 is
  deployed before this story ships.

- [ ] **T-B03**: Implement the per-file analysis function — given a file path and its staged diff,
  determine: (a) does it have a `## Changelog` section? (b) do staged additions include
  body-text changes outside exempted sections? (c) does the staged diff include a `version:`
  field change? (d) does the staged diff include a new `## Changelog` row? If b=YES and
  c=NO and d=NO → violation. Emit the violation with file path and evidence.

- [ ] **T-B04**: Implement the main check function `run_content_version_check()` — mirrors
  `run_l9()` structure: accumulates staged diff from both main project index AND `.factory/`
  worktree index, iterates over versioned files in the diff, applies per-file analysis.
  Emits "Scanned N versioned files in staged diff" before reporting. Exit 1 on violations.

- [ ] **T-B05**: Add RG-004 real-worktree self-probe case. Stage a real `.factory/` file with
  a normative body change and no version bump; run the check; confirm it fires via the
  `.factory/` worktree index path. Unstage; confirm exit 0. Document result.

- [ ] **T-B06**: Integrate the new check function into the main gate execution path. Decide
  CLI behavior (default mode, `--full-scan`, or both). Document the decision.
  Recommendation: default (staged-diff) mode, since it's a commit-time gate.

### Phase C — Verify self-probe passes

- [ ] **T-C01**: Run `scripts/records-lint.sh --self-probe`. Confirm all 5 new cases pass.
  Confirm existing cases still pass (no regression). State new total and update CLAUDE.md
  reference count.

### Phase D — CLAUDE.md amendment

- [ ] **T-D01** (AC-005): Amend CLAUDE.md §TD-VSDD-092 "L1 gate capability boundary" note
  and §check-list per AC-005. Verify amended text does not introduce volatile line cites
  or version pins.

---

### Merge gate

- [ ] **MERGE-GATE-SELF-PROBE**: `scripts/records-lint.sh --self-probe` exits 0 with all new
  cases passing and existing cases unaffected.
- [ ] **MERGE-GATE-CHECK-NAME-EXEMPT**: New check name in `L9_CHECK_NAME_EXEMPT`. RG-005 confirmed.
- [ ] **MERGE-GATE-CHANGELOG-BACKFILL-PREREQUISITE**: Confirm `S-MAINT-CHANGELOG-BACKFILL-001`
  PR has merged before this PR is merged (per `depends_on` constraint). Do NOT merge before
  the backfill ships.
- [ ] **MERGE-GATE-REAL-WORKTREE-PROBE**: RG-004 (`test_content_version_fires_real_factory_worktree`)
  confirmed in PR description, proving the "probe-passes ≠ gate-fires" gap cannot recur.

---

## Previous Story Intelligence

First story in the content-change-version-bump chain.

Related prior art:
- `S-MAINT-CHANGELOG-BACKFILL-001` (prerequisite): that story's backfill must ship before this
  gate can enforce uniformly. `depends_on: [S-MAINT-CHANGELOG-BACKFILL-001]` is the hard gate.
- `S-MAINT-L11-GATE-001`: primary structural reference. The real-worktree probe (AC-006/RG-006
  in L11-GATE-001) is mirrored here as RG-004 to prevent the "probe-passes ≠ gate-fires" gap.
  The implementer SHOULD read S-MAINT-L11-GATE-001's T-B04/T-B05 implementation before writing
  the analogous logic here.
- `S-MAINT-LEDGER-CITE-GATE-001` and records-lint.sh surface overlap: both this story and
  LEDGER-CITE-GATE-001 extend records-lint.sh. If both are in-flight simultaneously, coordinate
  with state-manager so records-lint.sh edits are not made in parallel. Both can ship in the
  same PR if the implementer chooses.

---

## Architecture Compliance Rules

1. **Worktree-index discipline (TD-VSDD-092 §WORKTREE NOTE):** Same as L11 and the existing
   L9 design. The check MUST query `git -C WORKSPACE_ROOT/.factory diff --cached` in addition
   to `git -C WORKSPACE_ROOT diff --cached`. RG-004 mechanizes this requirement.
   This MUST is anchored to AC-003 and RG-004.

2. **Ratchet-scoped by default (AC-002):** Pre-existing content/version drift is grandfathered.
   The gate prevents NEW violations from accumulating from merge day forward.

3. **Section-range computation required for exemptions (AC-002):** Line-shape heuristics
   (e.g., "skip if the line looks like a changelog table row") are insufficient. Section
   membership must be determined by scanning from the file top and tracking the most recent
   `## ` heading. Mirrors L11-GATE-001 AC-003 principle.
   This MUST is anchored to AC-002.

4. **Check name in `L9_CHECK_NAME_EXEMPT` before PR merge (AC-005):** The check name token
   must be added before merge. Anchor: AC-005 / RG-005. This MUST is anchored to AC-005 and RG-005.

5. **No prism crate modifications.** No file under `crates/` is touched.

6. **POL-29 TD-VSDD-097 three-dimension sweep for this story's CLAUDE.md amendment:**
   (a) Sibling pair: TD-VSDD-092 and TD-VSDD-091 are co-documented; amend both if the
       capability-boundary note structure requires cross-section updates.
   (b) Downstream copy target: SESSION-HANDOFF.md §Standing Rules — sweep for stale
       references if the TD-VSDD-092 gate-capability-boundary structure changes.
   (c) Mandate anchor: every MUST in the amended CLAUDE.md text cites this story
       (S-MAINT-CONTENT-VERSION-GATE-001) plus the specific AC. Verified above.

---

## Library and Framework Requirements

| Library/Tool | Version/Source | Purpose |
|-------------|---------------|---------|
| `scripts/records-lint.sh` | Project-local | Gate to extend |
| `bash` | System bash (≥ 3.2) | Script runtime; no new dependency |
| `git` | System git | Worktree-index-aware diff queries |

No external dependencies. No changes to Cargo.toml files.

---

## File Structure Requirements

### Files to MODIFY

| File | Change |
|------|--------|
| `scripts/records-lint.sh` | Add check name to CONFIG BLOCK, section-range helpers (if not already present from L11), main check function, self-probe cases (RG-001 through RG-005), update `--self-probe` expected total, add check name to `L9_CHECK_NAME_EXEMPT` |
| `CLAUDE.md` | Extend §TD-VSDD-092 L1 capability boundary note and §check-list |

### Files NOT to modify

| File | Reason |
|------|--------|
| Any `crates/**` file | No Rust code changes |
| `.factory/**` files | State-manager handles .factory/ commits |
| `scripts/records-lint.sh` L9/L11/L10 check bodies | Out of scope |

---

## Forbidden Dependencies

No new shell dependencies beyond standard POSIX utilities already used by `scripts/records-lint.sh`
(`awk`, `grep`, `sed`, `git`, `mktemp`). Do not introduce external tools.

---

## Changelog

| Version | Date | Author | Summary |
|---------|------|--------|---------|
| 1.0 | 2026-07-31 | story-writer | FB105 — initial story creation. Codifies Lesson 117 (records-lint L1 gate capability boundary — gate-green is not audit-trail-complete). Adds content-change-without-version-bump check to records-lint.sh: detects staged normative content changes to versioned .factory/ artifacts without a version bump AND without a new changelog row (case 1 in the three-condition analysis). Ratchet-scoped, worktree-index-aware. Real-worktree probe (RG-004) prevents probe-passes-gate-fires gap. depends_on S-MAINT-CHANGELOG-BACKFILL-001 (backfill prerequisite). 5 ACs, 5 RG tests. status: draft; behavioral_contracts: [] pending PO authorship per S-7.01. |
