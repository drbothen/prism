---
document_type: story
story_id: "S-MAINT-BURST-COMMIT-COUNT-GATE-001"
title: "Factory Gate — Replace Trigger-Word Heuristic with Count-Based Single-Commit-Per-Burst Enforcement"
wave: maintenance
epic_id: maintenance
priority: P1
status: draft
version: "1.0"
updated: "2026-08-02"
level: ops
producer: story-writer
timestamp: "2026-08-02T00:00:00Z"
tdd_mode: strict
# tdd_mode: strict — this story modifies executable gate logic in
# .factory/hooks/verify-sha-currency.sh.
# Red Gate tests are --self-probe cases that FAIL before the count-based check
# is implemented (the new check does not yet exist; origin-divergence logic is
# absent and the existing trigger-word path would pass the violation cases).
# They PASS after implementation. Standard Rust todo!() discipline does not
# apply (no Rust crates touched), but the self-probe framework serves the
# identical TDD purpose.
subsystems: []
# Cross-cutting governance toolchain; no single product subsystem owns it.
crates_touched: []
target_module: ".factory/hooks/verify-sha-currency.sh, drbothen/vsdd-factory upstream"
capabilities: []
behavioral_contracts: []
# BC status: pending PO authorship.
# TD-VSDD-053 is a governance operational discipline, not a behavioral contract.
# A product-owner BC covering factory-process correctness would need to be authored
# before this story can reach status: ready (S-7.01 gate).
verification_properties: []
holdout_scenarios: []
depends_on: []
blocks: []
points: 5
estimated_days: 1.0
risk: LOW
# Risk justification:
#   All changes are in verify-sha-currency.sh (bash script) plus an upstream
#   issue. No Rust production code touched. Primary risk is false-positive
#   from the origin-divergence check if the push-after-every-burst invariant
#   is not unconditional — ARCH-QUES-001 must be answered before ready.
#   A false-positive blocks state-manager but causes no data loss; the escape
#   path is `git -C .factory push` (which should have happened anyway).
acceptance_criteria_count: 6
red_gate_tests: 6
estimated_passes: "tbd"
assumption_validations: []
risk_mitigations: []
tags:
  - process-gap
  - factory-tooling
  - td-vsdd-053
  - governance-gate
---

# S-MAINT-BURST-COMMIT-COUNT-GATE-001: Factory Gate — Replace Trigger-Word Heuristic with Count-Based Single-Commit-Per-Burst Enforcement

## Authority

`CLAUDE.md §TD-VSDD-053` is the primary governance source for this story. It defines
the single-commit-per-burst rule and the MULTI_COMMIT_CHAIN_NOT_ALLOWED detector.
Read it before implementing. `CLAUDE.md §Factory Hook Diagnostics` describes the
current detection mechanism and the hook infrastructure through which this gate runs.

The existing enforcement lives in `.factory/hooks/verify-sha-currency.sh §Multi-commit
chain guard`. Read that section before modifying it; it contains the trigger-word check
being replaced, plus context on why the two-commit protocol was retired.

Three confirmed recurrences establishing the codification threshold (CLAUDE.md
§codification threshold "recurs 3+ times") are recorded in STATE.md decision log as
D-2086, D-2087 (process-gap finding at D-2086), and D-2090.

No ADR governs the `.factory/` commit protocol — CLAUDE.md §TD-VSDD-053 is the
source of truth. This story does not require ADR adjudication; it requires one
architect answer (see ARCH-QUES-001 in §Acceptance Criteria).

---

## Origin

**Process-gap finding:** three confirmed recurrences of TD-VSDD-053 violations, all
passing the existing trigger-word gate (CLAUDE.md §codification threshold met
2026-08-02; human-approved "accept + codify" decision, S-7.02 cycle-close follow-up).

**Root cause:** the current check in `verify-sha-currency.sh §Multi-commit chain guard`
detects trigger words ("backfill") in **both** HEAD and HEAD^ simultaneously. This
enforces a naming convention, not a commit count. Three confirmed escape paths:

- **D-2086 shape:** first commit contains trigger word, second does not. Only one
  subject carries the word → gate passes. Recorded as `[process-gap]` at D-2087.
- **D-2086b shape (commit `c5229ddc0`):** a follow-up "records-only" correction
  commit landed immediately after the round-2 burst commit. Neither subject required
  trigger words → gate passes.
- **D-2090 shape (commits `313b8f2f6` + `201ac47c6`):** first commit's own subject
  asserts `(TD-VSDD-053 single-commit)` compliance. HEAD^ carried "backfill"; HEAD
  did not. Gate fires only when BOTH contain the trigger word → gate passes.

**Structural lesson:** this failure class is identical to the L9-inoperative-before-
2026-07-24 and L10-cannot-detect-content-falsification disclosures documented in
CLAUDE.md §TD-VSDD-092: *probe-passes ≠ gate-fires*, and a gate must be evidenced
against the property it claims to enforce. The trigger-word check produces a green
result that is not evidence of single-commit compliance.

---

## Narrative

As a state-manager commit workflow,
I want the factory hygiene gate to detect when a burst has already produced a commit
that has not yet been pushed to `origin/factory-artifacts`, by counting actual
unpushed commits rather than inspecting subject-line wording,
so that TD-VSDD-053 is enforced against the property it specifies (one commit per
burst) rather than against a naming convention — and a commit message asserting its
own compliance cannot pass the gate while being half of a two-commit chain.

---

## Behavioral Contracts

| BC | Title | Version | Scope in This Story |
|----|-------|---------|---------------------|
| (none) | — | — | No behavioral contracts yet. See BC status comment in frontmatter. Story MUST remain `status: draft` until a governance-integrity BC is authored and anchored. |

---

## Acceptance Criteria

### AC-001 — Gate counts actual unpushed commits; does not examine subject-line wording

After implementation, the `§Multi-commit chain guard` in `verify-sha-currency.sh`
MUST compute the number of commits on `factory-artifacts` that are ahead of
`origin/factory-artifacts` (unpushed). If that count is greater than zero when a
state-manager burst begins, the gate MUST emit MULTI_COMMIT_CHAIN_NOT_ALLOWED and
exit non-zero.

The check MUST NOT inspect subject-line text for trigger words ("backfill", "Stage 1",
"Stage 2", or any other naming-convention term) as evidence of compliance or
non-compliance. The commit count is the sole criterion.

A commit subject asserting its own compliance (e.g., containing the string
"TD-VSDD-053 single-commit") is NOT evidence that the commit is the sole burst
commit. The count-based check fires regardless of what the subject line claims.

The existing trigger-word code block (the `HEAD_IS_BACKFILL` / `PARENT_IS_BACKFILL`
logic in the `§Multi-commit chain guard` section of verify-sha-currency.sh) MUST be
replaced or superseded by the count-based check. The trigger-word check is not retained as a secondary guard —
it produces false-green results that contradict the primary count-based gate.

(verified by RG-001, RG-002, RG-005)

### AC-002 — Gate retro-detects all three historical D-2086/D-2086b/D-2090 shapes

Self-probe test fixtures MUST reproduce the three confirmed-escape shapes and confirm
that the new gate fires on each:

- **D-2086 shape:** synthetic repo where origin is at commit-1, HEAD is at commit-2;
  commit-1 subject contains "backfill", commit-2 subject does not. New gate: FAIL.
  Old gate would have passed (only one subject carried the trigger word).

- **D-2086b shape:** same origin/HEAD setup; both subjects are neutral (no trigger
  words). New gate: FAIL. Old gate would have passed (no trigger word in either).

- **D-2090 shape:** same origin/HEAD setup; commit-1 subject contains
  "(TD-VSDD-053 single-commit)", commit-2 subject does not. New gate: FAIL. Old gate
  would have passed (HEAD^ carried "backfill" but HEAD did not, so both-trigger-word
  condition was false; the compliance claim in HEAD^'s subject is irrelevant to count).

Passing all three fixtures is necessary for a gate that "cannot retro-detect its own
motivating failures" to meet the TD-VSDD-059 paper-fix standard.

(verified by RG-003, RG-004, RG-005)

### AC-003 — Burst boundary is determined by push separation (origin divergence)

The gate's burst-boundary signal is `git -C <factory_dir> log origin/factory-artifacts..HEAD --oneline | wc -l`. A result > 0 means a burst commit exists that has not yet been pushed to `origin/factory-artifacts` — the gate treats this as the current burst being open.

**ARCH-QUES-001 (architect decision required before `status: ready`):** Is the
push-after-every-burst invariant unconditional in the factory workflow? CLAUDE.md
§factory-artifacts states: "state-manager pushes it as part of each `.factory/` burst."
If this is unconditional (no permitted exceptions for network failures, batching, or
squash workflows), then origin-divergence > 0 is a reliable burst-boundary signal and
the gate needs no escape hatch. If there are permitted exceptions, the gate requires
a machine-readable escape hatch (proposed: `FACTORY_BURST_SKIP_CHAIN_GATE=1` env var
with mandatory justification logged) to avoid blocking legitimate workflows.

Until ARCH-QUES-001 is answered, the story MUST remain `status: draft`. The
architect answer is the sole required adjudication; no other architectural uncertainty
blocks implementation.

The gate MUST handle the case where `origin/factory-artifacts` does not exist
(initial push on a new repo): treat as zero divergence (no unpushed commits) → PASS.
This is AC-005.

(verified by RG-003, RG-004, RG-005, RG-006)

### AC-004 — Error message names the stale commit SHA and explains the resolution path

When the gate fires, the error output MUST include:
- The string "MULTI_COMMIT_CHAIN_NOT_ALLOWED" (for compatibility with existing
  tooling that matches on this identifier)
- The SHA of the unpushed burst commit already on HEAD
- A human-readable reason explaining why the count-based gate fired
- A resolution path: `git -C .factory push` to advance origin, then retry

The error MUST NOT suggest that rewriting the commit subject to remove trigger words
is a valid resolution. That anti-pattern is the root cause being fixed.

(verified by RG-001, RG-002)

### AC-005 — Initial-push exemption: origin/factory-artifacts not found → PASS

If `git -C <factory_dir> log origin/factory-artifacts..HEAD` fails with a non-zero
exit code because the `origin/factory-artifacts` ref does not exist (e.g., the repo
has never pushed to the remote), the gate MUST treat this as zero divergence and
allow the commit.

This handles the first-push case for new factory setups without requiring a
special-case flag.

(verified by RG-006)

### AC-006 — Upstream issue filed against drbothen/vsdd-factory

A GitHub issue is filed against `drbothen/vsdd-factory` documenting: (a) the three
confirmed-escape shapes (D-2086, D-2086b, D-2090) and the structural root cause
(trigger-word heuristic, not commit count); (b) the origin-divergence count-based
gate specification from AC-001 through AC-005; (c) the ARCH-QUES-001 design question;
(d) a pointer to `S-MAINT-L11-GATE-001` as a sibling gate story for structural
reference (same pattern: failing self-probe cases first, then implementation); and (e)
the disclosure class — "probe-passes ≠ gate-fires" — as the transferable lesson.
The upstream issue URL is recorded in §Deliverables.

---

## Red Gate Tests

All 6 RG items are self-probe test cases that FAIL before the origin-divergence count
check is implemented (the new logic does not exist; the trigger-word path would pass
violation cases) and PASS after. RG-003/004/005 are the historical-shape retro-
detection cases; RG-006 is the initial-push exemption case. All use synthetic temp
git repos with a simulated `origin/factory-artifacts` remote per the existing
self-probe pattern in `scripts/records-lint.sh`.

- [ ] **RG-001** (`test_burst_second_commit_neutral_subjects_fails`): self-probe case —
  synthetic repo where `origin/factory-artifacts` is at commit-1, HEAD is at commit-2;
  BOTH subjects are generic text (no trigger words). New gate must emit
  MULTI_COMMIT_CHAIN_NOT_ALLOWED and exit non-zero. FAILS before the count-based check
  exists (old trigger-word path would pass); PASSES after implementation. This is the
  foundational case: a neutral-subject second commit is the most common gap.

- [ ] **RG-002** (`test_burst_first_commit_passes`): self-probe case — synthetic repo
  where HEAD is at origin (zero divergence). Gate must exit zero (PASS). Confirms the
  gate does not fire on a clean first-burst commit. FAILS before implementation only if
  the stub returns permanent-fail; PASSES after correct implementation.

- [ ] **RG-003** (`test_d2086_shape_one_trigger_word_fails`): self-probe case — D-2086
  shape: origin at commit-1, HEAD at commit-2; commit-1 subject contains "backfill",
  commit-2 subject does not. New gate must FAIL. Retro-detects the first confirmed
  recurrence. Old gate would have passed (HEAD^ carries trigger word, HEAD does not;
  old logic requires BOTH to carry it).

- [ ] **RG-004** (`test_d2086b_shape_two_neutral_commits_fails`): self-probe case —
  D-2086b shape: origin at commit-1, HEAD at commit-2; both subjects are neutral. New
  gate must FAIL. Retro-detects the follow-up correction commit pattern. Old trigger-
  word path cannot catch this shape at all.

- [ ] **RG-005** (`test_d2090_shape_compliance_claim_in_subject_fails`): self-probe
  case — D-2090 shape: origin at commit-1, HEAD at commit-2; commit-1 subject contains
  "(TD-VSDD-053 single-commit)"; commit-2 subject is neutral. New gate must FAIL.
  Explicitly validates that a compliance assertion in the subject line is NOT evidence.
  Retro-detects the third confirmed recurrence.

- [ ] **RG-006** (`test_initial_push_no_origin_ref_passes`): self-probe case — synthetic
  repo where `origin/factory-artifacts` ref does not exist (initial push scenario).
  `git log origin/factory-artifacts..HEAD` exits non-zero. Gate must treat this as
  zero divergence and exit zero (PASS). Prevents false-positive blocking of first-ever
  `.factory/` commits in new factory deployments.

**Red Gate density check (BC-5.38.001):** 6 Red Gate tests (RG-001 through RG-006)
anchor to 6 acceptance criteria. Density verification
(`RED_TESTS * 2 >= (TOTAL_NEW_TESTS - EXEMPT_TESTS)`) is computed at dispatch by the
orchestrator per the per-story-delivery §Red Gate Density Check and BC-5.38.002/003.
No authoring-time ratio is pre-computed here; the orchestrator holds that gate.

---

## Architecture Mapping

| Component | Location | Pure/Effectful |
|-----------|----------|----------------|
| `§Multi-commit chain guard` (count-based replacement) | `.factory/hooks/verify-sha-currency.sh` | Effectful (reads git log from factory-artifacts worktree; exits non-zero on violation) |
| `--self-probe` RG cases (self-probe test harness) | `verify-sha-currency.sh` (if `--self-probe` mode exists) or standalone test script | Effectful (creates synthetic temp git repos) |
| upstream issue | `drbothen/vsdd-factory` GitHub | Effectful (network; AC-006) |

Note: `verify-sha-currency.sh` does not currently have a `--self-probe` mode. The
test cases may be implemented as a companion test script
(`.factory/hooks/test-verify-sha-currency.sh`) rather than embedded in the main hook.
The implementing agent chooses the approach; the test vehicle must fail before
implementation and pass after (the TDD criterion).

---

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `origin/factory-artifacts` exists but is an empty ref (no commits) | Treat as zero divergence → PASS. `git log origin/factory-artifacts..HEAD` against an empty ref may produce an error; handle gracefully. |
| EC-002 | `.factory/` worktree `origin` remote is not configured (no remote URL) | Same as AC-005 (no origin ref) → PASS with warning. Log that the push-separation check cannot run. |
| EC-003 | Count-based check fires during a deliberate `git rebase -i` cleanup that temporarily leaves HEAD ahead of origin | This is a false-positive. The escape hatch from ARCH-QUES-001 (`FACTORY_BURST_SKIP_CHAIN_GATE=1`) covers this if the architect confirms unconditional-push invariant has exceptions. |
| EC-004 | The escape hatch env var `FACTORY_BURST_SKIP_CHAIN_GATE=1` is set | Gate MUST log a WARN with the skipping reason, then exit zero. Never silently skip — the skip must be visible in the hook output. ARCH-QUES-001 determines whether this env var is included at all. |
| EC-005 | A single burst commit covers multiple STATE.md decisions and is legitimately large | Gate MUST pass. The count check (> 0 unpushed commits) correctly accepts a single large commit. Single-commit size is not constrained by this gate. |
| EC-006 | A VSDD pipeline runs with no `.factory/` worktree (e.g., greenfield initialization before first commit) | Gate must not hard-crash. If `git -C "$FACTORY_DIR"` fails because the directory is not a git repo, treat as zero divergence (initial setup) → PASS. |

---

## Token Budget Estimate

| Artifact | Estimated Tokens | Notes |
|----------|-----------------|-------|
| This story spec | ~7,000 | |
| `.factory/hooks/verify-sha-currency.sh` (full file) | ~6,000 | Read before modifying; understand existing checks and exit-code patterns |
| `CLAUDE.md §TD-VSDD-053 + §Factory Hook Diagnostics` | ~2,000 | Load only the relevant sections |
| `S-MAINT-L11-GATE-001` (structural reference, self-probe pattern) | ~8,000 | Test-harness pattern to replicate for self-probe cases |
| Upstream issue text authoring | ~1,000 | |
| **Total per implementation session** | ~24,000 | Well within 20–30% of agent context window |

Context management: implement the count-based check in `verify-sha-currency.sh` first
(a single function replacement), write and run self-probe cases, then file the upstream
issue. Do not load all artifacts simultaneously.

---

## Tasks

### Phase A — Write failing self-probe test cases (Red Gate — BEFORE implementation)

- [ ] **T-A01** (RG-001 setup): Write a self-probe test script
  `.factory/hooks/test-verify-sha-currency-count-gate.sh`. Add RG-001 case: create a
  synthetic temp git repo with an `origin/factory-artifacts` remote at commit-1 and
  HEAD at commit-2 (neutral subjects). Call the count-based check (which does not yet
  exist). Confirm the test returns FAIL (either because the check is absent and
  exits 0 when 1 is expected, or because a permanent-fail stub is added first).

- [ ] **T-A02** (RG-002 through RG-006 setup): Add remaining 5 self-probe cases to the
  test script: RG-002 (zero divergence → expect PASS), RG-003 (D-2086 shape →
  expect FAIL), RG-004 (D-2086b shape → expect FAIL), RG-005 (D-2090 shape →
  expect FAIL), RG-006 (no origin ref → expect PASS). Run the test script and confirm
  all 6 cases fail in the expected direction before implementation (RG-001/003/004/005
  should FAIL because the check is absent; RG-002/006 should also FAIL if a
  permanent-fail stub is used). Record the pre-implementation failure count.

### Phase B — Implement count-based check (Green — make failing tests pass)

- [ ] **T-B01**: Answer ARCH-QUES-001 (obtain architect confirmation that
  push-after-every-burst is unconditional, or that an escape hatch is needed).
  Record the answer in the PR description. If an escape hatch is needed, add
  `FACTORY_BURST_SKIP_CHAIN_GATE` env var support (EC-004) before T-B02.

- [ ] **T-B02**: In `verify-sha-currency.sh §Multi-commit chain guard`, REPLACE the
  trigger-word block with an origin-divergence count check:
  ```
  UNPUSHED=$(git -C "$FACTORY_DIR" log origin/factory-artifacts..HEAD \
      --oneline 2>/dev/null | wc -l | tr -d ' ')
  ```
  If `UNPUSHED > 0`: emit MULTI_COMMIT_CHAIN_NOT_ALLOWED with the SHA of HEAD and
  the resolution path (`git -C .factory push`). Exit non-zero. Preserve the
  MULTI_COMMIT_CHAIN_NOT_ALLOWED identifier string for tooling compatibility.
  Handle AC-005 (origin ref absent → PASS) via the `2>/dev/null` fallback.

- [ ] **T-B03**: Confirm that the error message does NOT suggest rewriting the commit
  subject as a resolution path (AC-004 negative requirement). The message MUST say
  `git -C .factory push` is the resolution, not "avoid trigger words."

### Phase C — Verify self-probe tests pass

- [ ] **T-C01**: Run `.factory/hooks/test-verify-sha-currency-count-gate.sh`. Confirm
  all 6 cases pass: RG-001 (neutral second commit → FAIL emitted), RG-002 (zero
  divergence → PASS), RG-003 (D-2086 shape → FAIL), RG-004 (D-2086b shape → FAIL),
  RG-005 (D-2090 compliance-claim shape → FAIL), RG-006 (no origin ref → PASS).

- [ ] **T-C02**: Run `bash .factory/hooks/verify-sha-currency.sh` on the current clean
  `.factory/` worktree. Confirm it still exits 0 (no false-positive on the real repo).
  This validates AC-005/EC-006 in a real environment — the gate does not fire when
  there are no unpushed burst commits.

### Phase D — Upstream issue and cleanup

- [ ] **T-D01**: File upstream issue against `drbothen/vsdd-factory` per AC-006.
  Record the URL in §Deliverables.

- [ ] **T-D02**: Verify that the existing `scripts/records-lint.sh` is unaffected
  (no modifications to records-lint.sh are part of this story's scope).

### Merge gate

- [ ] **MERGE-GATE-TEST-PASS**: `.factory/hooks/test-verify-sha-currency-count-gate.sh`
  exits 0 with all 6 cases passing.
- [ ] **MERGE-GATE-NO-FALSE-POSITIVE**: `bash .factory/hooks/verify-sha-currency.sh`
  on the clean repo exits 0.
- [ ] **MERGE-GATE-ARCH-QUES-001-ANSWERED**: T-B01 result documented in PR description;
  escape hatch included if needed or explicitly noted as absent.

---

## Previous Story Intelligence

First story in the TD-VSDD-053 count-based enforcement chain. No predecessor in this
specific enforcement track.

Related prior art:
- `S-MAINT-L11-GATE-001`: direct structural precedent. Established the pattern for
  adding a count/detection-based check to an existing governance script (records-lint.sh),
  with self-probe test cases that fail before implementation. Follow its Phase A → B → C
  task ordering exactly. Key lesson: the L9 worktree bypass went undetected because
  self-probe used synthetic temp repos while the real path was never exercised. RG-002
  and RG-006 exist specifically so the zero-divergence and no-origin-ref paths are
  tested against the real `.factory/` worktree (T-C02).
- `S-MAINT-RG-LIST-GATE-001` and `S-MAINT-ADR-ANCHOR-GATE-001`: sibling process-gap
  follow-up stories. Established the two-tier gate pattern (hard block vs warning).
  This story has no warning tier — the count-based check is a hard block only, because
  a "partial compliance" concept does not exist for commit count.
- `S-MAINT-POL29-HOOK-001`: older process-gap pattern — root cause → upstream issue →
  local gate script. Additional frontmatter template reference.

---

## Architecture Compliance Rules

1. **No prism crate modifications.** This story MUST NOT add, remove, or edit any file
   under `crates/`. Scope is `.factory/hooks/verify-sha-currency.sh` plus the test
   companion script and upstream issue.

2. **MULTI_COMMIT_CHAIN_NOT_ALLOWED identifier preserved.** The error identifier string
   MUST remain exactly `MULTI_COMMIT_CHAIN_NOT_ALLOWED` in the gate output. External
   tooling (CLAUDE.md §Factory Hook Diagnostics log parsers) matches on this string.

3. **Trigger-word check fully replaced, not layered.** The old trigger-word block MUST
   be removed, not kept as a secondary check. Retaining it alongside the count check
   creates a contradictory gate: the count check fires but the trigger-word check passes,
   producing ambiguous diagnostics. One gate, one criterion.

4. **TD-VSDD-053 single-commit-per-burst applies to this story's own delivery.**
   The `.factory/` changes for this story (if any) must go in one atomic commit.
   The story's own implementation is bound by the rule it enforces.

5. **No STATE.md edits.** STATE.md is state-manager territory. This story's
   implementation does not touch STATE.md or STORY-INDEX.md.

6. **Self-probe test must exercise the real `.factory/` worktree.** T-C02 is the
   real-worktree verification (mirrors RG-006 from S-MAINT-L11-GATE-001's AC-006
   pattern). The self-probe must not rely exclusively on synthetic temp repos.

---

## Library and Framework Requirements

| Library/Tool | Version/Source | Purpose |
|-------------|---------------|---------|
| `.factory/hooks/verify-sha-currency.sh` | Project-local | Gate to extend |
| `bash` | System bash (≥ 3.2) | Script runtime; no new dependency |
| `git` | System git | Origin-divergence count query |

No external dependencies. No changes to Cargo.toml files or Rust crates.
`wc -l` and `tr` are POSIX standard utilities already used by existing hooks.

---

## File Structure Requirements

### Files to MODIFY

| File | Change |
|------|--------|
| `.factory/hooks/verify-sha-currency.sh` | Replace `§Multi-commit chain guard` trigger-word block with origin-divergence count check (AC-001 through AC-005) |

### Files to CREATE

| File | Change |
|------|--------|
| `.factory/hooks/test-verify-sha-currency-count-gate.sh` | Self-probe test script with 6 RG cases (T-A01/T-A02/T-C01) |

### Files NOT to modify

| File | Reason |
|------|--------|
| Any `crates/**` file | Out of scope (Architecture Compliance Rule 1) |
| `scripts/records-lint.sh` | Out of scope; handled by S-MAINT-L11-GATE-001 |
| `.factory/STATE.md` | State-manager territory |
| `CLAUDE.md` | No CLAUDE.md amendment required for this story — the TD-VSDD-053 description
already accurately states the rule; this story fixes the gap between stated rule and
enforcement mechanism |

---

## Forbidden Dependencies

No new shell dependencies beyond POSIX utilities already used by verify-sha-currency.sh
(`bash`, `git`, `grep`, `wc`, `tr`). Do not introduce `python3`, `awk`, `yq`, or
external binaries into verify-sha-currency.sh to preserve the zero-external-dependency
property of the hook.

---

## §Deliverables

| Item | Status | Reference |
|------|--------|-----------|
| Upstream issue URL | Pending | (to be filled at T-D01 completion) |

---

## Changelog

| Version | Date | Author | Summary |
|---------|------|--------|---------|
| 1.0 | 2026-08-02 | story-writer | S-7.02 cycle-close follow-up: new story registered for three confirmed TD-VSDD-053 recurrences (D-2086, D-2086b, D-2090). Replaces trigger-word heuristic in verify-sha-currency.sh §Multi-commit chain guard with origin-divergence count check. SAC-1 compliant: 6 enumerated RG tests (RG-001 through RG-006), density check deferred to dispatch, red-then-green task ordering (Phase A before Phase B). ARCH-QUES-001 (push-separation unconditional?) is the sole gating architect question. status: draft pending PO BC authorship (S-7.01) and ARCH-QUES-001 answer. |
