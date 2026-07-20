---
document_type: story
story_id: S-MAINT-CIGATE-REMEDIATION-001
title: "CI-gate remediation — resolve pre-existing test failures + wire enforcement + actionlint SC2086 fix"
wave: tbd
epic_id: maintenance
priority: P2
status: draft
version: "0.3"
level: ops
producer: story-writer
timestamp: "2026-07-19"
modified: "2026-07-19"
input-hash: "[live-state]"
inputs:
  - .github/workflows/ci.yml
  - .github/workflows/post-merge.yml
  - tests/ci-gate/run.sh
  - tests/ci-gate/test_AC-3_matrix-5-platforms.sh
  - tests/ci-gate/test_AC-4_cargo-audit.sh
  - tests/ci-gate/test_AC-5_kani-proofs.sh
  - tests/ci-gate/test_AC-9_no-hardcoded-secrets.sh
traces_to: ""
cycle: "v1.0.0-release-engineering"
phase: 3
tdd_mode: strict
track: "Platform Engineering"
subsystems: []
# Subsystem anchor: this story touches only CI toolchain files (.github/workflows/*.yml,
# tests/ci-gate/, Justfile). No ARCH-INDEX subsystem (SS-01..SS-22) owns CI infrastructure.
# subsystems: [] is correct per S-MAINT-CI-DISK-EXHAUSTION-001 and W3-FIX-CI-001 precedent.
crates_touched: []
target_module: devops
behavioral_contracts: []
# BC status: CONFORMING (no BC required).
#
# This story is CI toolchain-only: it modifies .github/workflows/ci.yml (SC2086 fix,
# actionlint), tests/ci-gate/ test scripts (assertion corrections), and Justfile (new
# test-ci-gate recipe). No product subsystem (SS-01..SS-22) is touched; no production
# behavior observable by an MCP client is affected.
#
# Controlling precedent: W3-FIX-CI-001 (merged, PR #112, behavioral_contracts: []) and
# S-MAINT-CI-DISK-EXHAUSTION-001 (merged, behavioral_contracts: []) — both CI-toolchain-only
# stories accepted under the no-BC convention. behavioral_contracts: [] is CONFORMING.
# The S-7.01 draft-blocker does not apply under this adjudication.
#
# The ci-gate RED GATE tests embedded in tests/ci-gate/ ARE the correct VSDD artifact for
# CI structural invariants — they are self-describing CI assertions, not product behavioral
# contracts.
verification_properties: []
depends_on: [S-REL-001]
# Dependency anchor justification:
#   depends_on S-REL-001: AC-001 (test_AC-3 macos-13→macos-15-intel assertion update) was
#   superseded by S-REL-001 v0.8 (F-REL001-P6-001) — S-REL-001 now delivers both the ci.yml
#   runner change and the test script update. Dependency is retained because AC-007
#   (full assertion reconciliation) requires the ci.yml/e2e.yml staged residue to be
#   dispositioned, which is entangled with the S-REL-001 delivery, and because this story's
#   remaining 6 active ACs must execute against the ci.yml state that S-REL-001 establishes.
blocks: []
points: 5
estimated_days: 1
risk: LOW
# Risk justification: all changes are confined to CI test assertions and a YAML lint fix.
# No production Rust code is touched. The SC2086 fix is a cosmetic shell quoting correction;
# the test-script changes are assertion accuracy improvements. No regression risk.
acceptance_criteria_count: 8  # 7 active + 1 superseded (AC-001 delivered by S-REL-001 v0.8 F-REL001-P6-001)
red_gate_tests: 7
estimated_passes: "1"
holdout_scenarios: []
assumption_validations: []
risk_mitigations: []
triggered_by: "S-REL-001 LOCAL cascade findings F-REL001-P2-002 (harness hardening sweep), F-REL001-P3-004, F-REL001-P5-002, and OBS-1 [process-gap] (ci-gate hardened in S-0.01 but unwired from just / ci.yml enforcement step) — S-7.02 cycle-closing requirement. Also: 13 pre-existing ci-gate failures on develop @e116a587 (implementer pre-TDD analysis, 2026-07-19)."
---

# S-MAINT-CIGATE-REMEDIATION-001: CI-gate remediation — resolve pre-existing test failures + wire enforcement + actionlint SC2086 fix

## §Origin

This story codifies **OBS-1 [process-gap]** from the S-REL-001 LOCAL adversary cascade and
anchors the remaining pre-existing failures in the `tests/ci-gate/` suite discovered during
S-REL-001 delivery (S-7.02 cycle-closing requirement).

### S-REL-001 cascade findings that trigger this story

| Finding ID | Pass | Description | Resolution |
|------------|------|-------------|------------|
| F-REL001-P2-002 | LOCAL pass-2 | Harness hardening sweep: ci-gate run.sh missing TAP-plan reconciliation; tool-guard fail-closed not enforced for external tools (actionlint) | Wired in run.sh (S-0.01 AC-004/POL-34); residue: just recipe + ci.yml step still absent |
| F-REL001-P3-004 | LOCAL pass-3 | test_AC-9 false-positives on SHA-pinned `uses:` entries triggered by 40-hex heuristic | Heuristic must exclude `uses: owner/repo@<40-hex>` lines (not secrets; repo conventions require SHA pins) |
| F-REL001-P5-002 | LOCAL pass-5 | Remaining pre-existing ci-gate failures on develop: test_AC-4 ordering, test_AC-5 post-merge shape, test_AC-3 macos-13 stale runner | All three test scripts need assertion corrections |
| OBS-1 [process-gap] | LOCAL pass-5 | `tests/ci-gate/run.sh` comment line 6 says "once Justfile target is added by implementer" — `just test-ci-gate` recipe and a ci.yml enforcement step were never wired after S-0.01 merged | Wire `just test-ci-gate` recipe and add ci.yml step to make the gate fail-closed on every PR |

### Pre-existing failure count

S-REL-001 implementer pre-TDD analysis (2026-07-19) found **13 failing tests** in
`tests/ci-gate/` on develop HEAD (`@e116a587`). After S-REL-001 v0.8 fixes AC-3's stale
macos-13 runner assertion (both ci.yml and `tests/ci-gate/test_AC-3_matrix-5-platforms.sh`),
**12 failures remain** on develop, spread across the following files (test_AC-3 scope is
fully delivered by S-REL-001 v0.8 and is no longer in scope here):

- `test_AC-3_matrix-5-platforms.sh` — stale `macos-13` runner assertion [SUPERSEDED by
  S-REL-001 v0.8 F-REL001-P6-001 — S-REL-001 now delivers both the ci.yml runner change
  and the test script RUNNERS array update; do NOT re-apply in this story]
- `test_AC-4_cargo-audit.sh` — cargo deny step presence and ordering assertions do not
  match current ci.yml job structure
- `test_AC-5_kani-proofs.sh` — post-merge branch-scope, kani flags, and fuzz-target name
  assertions do not match current post-merge.yml
- `test_AC-9_no-hardcoded-secrets.sh` — SHA-pinned `uses:` entries (`@<40-hex>`) are
  incorrectly flagged as hardcoded secrets by the 40+ char heuristic

### Staged ci.yml/e2e.yml residue

At session start (2026-07-19), `git status` shows `M  .github/workflows/ci.yml` and
`M  .github/workflows/e2e.yml` — staged changes of **unknown provenance** on `develop`
(the staged diff removes ~620 lines including the S-MAINT-CI-DISK-EXHAUSTION-001 disk
reclaimer, neutralization steps, and other hardening). The origin and intent of these
changes have NOT been reviewed or approved. **AC-007 cannot proceed to implementation
until the human has dispositioned this residue.** This story's AC-007 records that
precondition explicitly.

---

## §Narrative

As a Prism CI maintainer, I want the `tests/ci-gate/` suite to accurately assert current
`ci.yml` / `post-merge.yml` intent and to be enforced automatically on every PR, so that
workflow drift is caught immediately rather than discovered during story delivery.

---

## §Behavioral Contracts

This story has no subsystem BCs — it is CI toolchain-only (see frontmatter rationale).
Compliance is verified by observing `just test-ci-gate` exit 0 on the repaired CI files.

---

## §Acceptance Criteria

### AC-001 — [SUPERSEDED by S-REL-001 v0.8 — delivered in feature/S-REL-001] Update test_AC-3 macos-13 → macos-15-intel

> **SUPERSEDED (F-REL001-P6-001, LOCAL pass-6):** S-REL-001 v0.8 now delivers both the
> ci.yml runner change and the `tests/ci-gate/test_AC-3_matrix-5-platforms.sh` RUNNERS array
> update (macos-13→macos-15-intel) as part of the same in-scope fix-burst. This AC is
> retained per POL-1 append_only_numbering — do NOT renumber remaining ACs. Do NOT re-apply
> this change in this story; implementing it here would double-apply the edit and conflict
> with S-REL-001's delivery.

`tests/ci-gate/test_AC-3_matrix-5-platforms.sh` currently asserts that `macos-13` appears
as a runner in ci.yml. The S-REL-001 story changes the Intel macOS runner to `macos-15-intel`
(research U20 from `release-engineering-uncertainties-2026.md`: macos-13 is RETIRED; macos-15-intel
is the replacement with Aug 2027 EOL). This assertion must be updated to `macos-15-intel`.

**Implementation note:** this AC is co-ordinate with S-REL-001 delivery. The ci.yml change
(if any) and the test assertion must be consistent after both stories are merged. If ci.yml
continues to declare `macos-13` in the test matrix (i.e., S-REL-001 only touches release.yml,
not ci.yml), the assertion must match whichever runner ci.yml actually uses. Read ci.yml before
editing the assertion; never update the test to match an intended-but-not-yet-landed ci.yml change.

### AC-002 — Remediate test_AC-4 cargo-deny presence and ordering assertions

`tests/ci-gate/test_AC-4_cargo-audit.sh` asserts:

1. `run: cargo audit` is a real step (not an echo).
2. `run: cargo deny check` is a real step (not an echo).
3. Step order: fmt → clippy → test → deny → audit → semver-checks (using first-occurrence
   line numbers in ci.yml).

Current ci.yml has `cargo deny check` but it may appear inside a multi-step `run:` block
(not as a standalone `run:` key). The test script's `grep -qE '^\s+run:\s+cargo deny check'`
pattern (leading whitespace + `run:` on the same line) may not match inline deny invocations
inside a `|` heredoc step. Verify the actual ci.yml structure and fix the assertion pattern
so it correctly detects whether deny is a real gate (not a TODO comment/echo) regardless of
whether it appears as a standalone `run:` or within a multi-line block.

If the ordering checks use grep-line-number heuristics that are fragile against multi-job
workflows (the first occurrence of `cargo fmt` may be in a different job than `cargo deny`),
document the limitation and scope each ordering check to the job it governs.

### AC-003 — Remediate test_AC-5 post-merge assertions

`tests/ci-gate/test_AC-5_kani-proofs.sh` asserts against `post-merge.yml`. Current failures
include one or more of:

- Branch-scope assertion (`branches: - main`) does not match current post-merge.yml trigger shape.
- `--timeout 300` and `--mem-limit 8192` flags do not appear in the kani invocation.
- Fuzz-target names (`fuzz_prismql_parser`, `fuzz_alias_expansion`, `fuzz_normalize`,
  `fuzz_spec_parser`, `fuzz_template_interpolation`, `fuzz_injection_scanner`) do not
  all appear in post-merge.yml.

Read `post-merge.yml` in full before editing any assertion. Fix each failing check to match
the CURRENT `post-merge.yml` content — do NOT change post-merge.yml to satisfy stale tests.
If the current workflow file genuinely lacks a required element (e.g., fuzz target is absent),
document the gap as a separate issue rather than masking it with a skip. A skip is only
appropriate when the gap is an intentional non-implementation (documented reason required).

### AC-004 — Fix test_AC-9 heuristic false-positives on SHA-pinned `uses:` entries

`tests/ci-gate/test_AC-9_no-hardcoded-secrets.sh` uses the heuristic:

```bash
grep -qE '^[^#]*[A-Za-z0-9]{40,}' "$file"
```

This matches SHA-pinned `uses:` entries such as:
```yaml
uses: actions/checkout@de0fac2e4500dabe0009e67214ff5f5447ce83dd # v6.0.2
```

40-character hex SHAs in `uses:` lines are the **required** form of action pinning per the
repo convention (CLAUDE.md §Conventions, S-MAINT-CI-DISK-EXHAUSTION-001 precedent, POL-34).
They are not secrets. The heuristic must be updated to exclude lines whose 40+ char blob
appears as the commit-SHA component of a `uses:` value.

The corrected exclusion rule: a line matching `^\s+uses:\s+[^@]+@[0-9a-f]{40}` is a valid
SHA-pinned action reference — it must NOT trigger the hardcoded-secret heuristic. The false-positive
filter can be implemented by pre-filtering `grep` output to exclude `uses:` lines before
applying the 40+ char blob test.

Example corrected logic:

```bash
# Exclude SHA-pinned 'uses:' lines — these are required action pins, not secrets.
secret_hits=$(grep -E '^[^#]*[A-Za-z0-9]{40,}' "$file" \
  | grep -vE '^\s+uses:\s+[^@]+@[0-9a-f]{40}' \
  | grep -vE '\$\{\{' \
  | wc -l)
if [ "$secret_hits" -gt 0 ]; then
  tap_fail "AC-9: possible hardcoded secret in ${wf}" ...
else
  tap_pass "AC-9: no obvious hardcoded secret pattern in ${wf}"
fi
```

Keep the negative assertion conceptually intact — only exempt lines that are structurally
`uses:` SHA pins; do not broaden the exclusion to all `@` references.

### AC-005 — Wire `just test-ci-gate` recipe and ci.yml enforcement step

`tests/ci-gate/run.sh` line 6 reads:
```
# just test-ci-gate   (once Justfile target is added by implementer)
```
This comment has been present since S-0.01 merged. The recipe was never added.

**Justfile change:** add a `test-ci-gate` recipe to `Justfile` that runs
`bash tests/ci-gate/run.sh` with proper error propagation (non-zero exit forwarded). The
recipe must appear adjacent to other test recipes (`just check`, `just check-fast`, etc.)
and be listed in `just --list` output.

**ci.yml change:** add a `ci-gate` job that runs `just test-ci-gate` on `ubuntu-latest` in
the PR gate pipeline. The job must:
- Run `just test-ci-gate` (not `bash tests/ci-gate/run.sh` directly, to exercise the recipe).
- Fail the PR if any ci-gate test fails.
- Be placed in the pipeline after `actions/checkout` but before or parallel to the test jobs
  (it is a structural validation, not a compilation gate).
- Use the same `actions/checkout` pin as other jobs in ci.yml.
- Not require Rust toolchain setup (ci-gate tests are pure bash + grep against the workflow files).

**Precondition for this AC:** ACs 001–004 must be green (all ci-gate tests pass) before the
wiring step is merged. A failing ci-gate test suite wired into CI becomes a permanent PR
blocker. Implement ACs 001–004 in the same commit or immediately prior commits.

### AC-006 — Fix actionlint SC2086 in ci.yml semver-checks job

The `semver-checks` job in ci.yml contains:

```bash
cargo semver-checks check-release --baseline-rev origin/develop $EXCLUDE_ARGS
```

`$EXCLUDE_ARGS` is unquoted. `actionlint` reports SC2086: "Double quote to prevent globbing
and word splitting." While `EXCLUDE_ARGS` contains `--exclude crate-name` fragments (no
glob characters), unquoted variable expansion is a shellcheck/actionlint violation and could
produce incorrect behavior if a crate name contained spaces or special characters.

**Fix:** quote `$EXCLUDE_ARGS` appropriately. Since `EXCLUDE_ARGS` is built by space-concatenation
of `--exclude <crate>` tokens, it must word-split when passed to cargo. The conventional fix
is to use a `bash` array (already available because `shell: bash` is the effective shell):

```bash
EXCLUDE_ARGS=()
for crate in $(cargo metadata ...); do
  if ! echo "$BASELINE_MEMBERS" | grep -qF "$crate"; then
    EXCLUDE_ARGS+=(--exclude "$crate")
  fi
done
if [ "${#EXCLUDE_ARGS[@]}" -eq 0 ]; then
  cargo semver-checks check-release --baseline-rev origin/develop
else
  echo "Excluding new crates from semver check: ${EXCLUDE_ARGS[*]}"
  cargo semver-checks check-release --baseline-rev origin/develop "${EXCLUDE_ARGS[@]}"
fi
```

After the fix, run `actionlint .github/workflows/ci.yml` locally (or CI) to confirm SC2086
is resolved. Do not introduce new actionlint findings.

### AC-007 — ci-gate suite assertion reconciliation (BLOCKED — human precondition required)

**EXPLICIT PRECONDITION: this AC cannot proceed to implementation until the human has
dispositioned the staged ci.yml/e2e.yml changes on `develop`.**

At session start 2026-07-19, `git status` shows the following staged changes on `develop`:
- `M  .github/workflows/ci.yml` (~620 line deletion, removing disk reclaimer, neutralization
  steps, and other S-MAINT-CI-DISK-EXHAUSTION-001 hardening)
- `M  .github/workflows/e2e.yml` (~26 line change)

These changes have **unknown provenance** — it is not clear whether they are intentional
(a human-authored rollback) or accidental (leftover from a tool invocation). Several
ci-gate assertions (AC-002, AC-006) and the wiring step (AC-005) depend on the final
state of ci.yml. Implementing against the staged version and then having the human discard
or amend it would require re-work.

**Human action required before this AC can be dispatched:**
1. Review `git diff --cached .github/workflows/ci.yml` and `git diff --cached .github/workflows/e2e.yml`.
2. Either commit the staged changes (with a rationale commit message) or discard them
   (`git restore --staged .github/workflows/ci.yml .github/workflows/e2e.yml`).
3. Record the disposition in `.factory/STATE.md` as a D-NNNN decision.

**After disposition:** reconcile all remaining ci-gate assertions that reference ci.yml
content areas affected by the staged changes. Update this story's version when the
precondition is resolved and implementation is authorized.

This AC is intentionally last — it becomes the integration test that all prior ACs and
the ci.yml content are consistent with each other after the staged residue is cleared.

### AC-008 — Committed musl cross-compile smoke in ci.yml (F-REL001-P12-OBS-002)

The ci.yml musl test leg currently installs the `x86_64-unknown-linux-musl` target via
`rustup target add` but the `cargo nextest run` command omits `--target`, so it builds
and tests for the `x86_64-unknown-linux-gnu` host. The sole committed CI regression
coverage for the musl release target is therefore absent — regressions in the musl
cross-compile path (including rocksdb-sys C++ linkage via `musl-g++` / `CC`/`CXX`
env-var configuration) are only caught at fork-tag dry-run time (S-REL-001 task 12),
which is ephemeral and not auditable from committed artifacts (F-REL001-P12-OBS-002).

**Required change:** amend ci.yml's musl leg, or add a dedicated `build-musl` job, so
that at minimum `cargo build --target x86_64-unknown-linux-musl -p prism-bin -p
prism-dtu-demo-server` is executed and must succeed on every PR touching `crates/**`,
`.github/workflows/*.yml`, or `Cargo.toml`.

**C++ toolchain prerequisite:** `x86_64-unknown-linux-musl` cross-compiles rocksdb-sys
(used by prism-storage) via a C++ compiler. The musl CI leg must either:

  (a) install `musl-g++` (or a musl-capable C++ cross-compiler such as `musl-tools` +
      `g++`), set `CC_x86_64_unknown_linux_musl` and `CXX_x86_64_unknown_linux_musl`
      env vars, AND set `CXXFLAGS_x86_64_unknown_linux_musl=-Wno-unused-result` (or
      equivalent) to avoid rocksdb-sys build failures; OR
  (b) document an explicit exclusion rationale in a ci.yml comment — e.g., if
      rocksdb-sys has been replaced by a pure-Rust alternative (cite the replacement
      crate and PR), or if prism-storage is excluded from the musl target via a
      `Cargo.toml` target-conditional (show the exclusion expression).

Whichever path is taken must be visible in committed ci.yml so the build signal is
reproducible and auditable without a fork-tag dry-run.

**Passing criterion:**
`grep -E '\-\-target x86_64-unknown-linux-musl' .github/workflows/ci.yml` returns at
least one match in a `run:` step that is not commented out, AND the corresponding CI
job is in the PR gate pipeline (not post-merge-only). If path (b) is taken, the comment
citing the exclusion rationale must be present adjacent to the musl target-add step.

(Origin: F-REL001-P12-OBS-002 [process-gap LOW] — release.yml 5-target cross-compile
has no committed re-runnable CI regression coverage; ci.yml musl leg omits --target)

---

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| `tests/ci-gate/*.sh` | `tests/ci-gate/` | Effectful (reads filesystem, spawns grep/awk subprocesses, emits TAP output) |
| `Justfile` `test-ci-gate` recipe | `Justfile` | Effectful (invokes bash subprocess, forwards exit code) |
| `.github/workflows/ci.yml` (ci-gate job + SC2086 fix) | `.github/workflows/` | Effectful (runs on GitHub-hosted runner, executes shell commands) |

Note: this story touches no Rust source files. The pure/effectful boundary applies to
shell scripts and CI YAML only. All components are effectful (I/O to filesystem, network,
or external processes). No pure-core functions are added or changed.

---

## §Token Budget Estimate

| Artifact | Estimated Tokens |
|----------|-----------------|
| This story spec | ~3,500 |
| `.github/workflows/ci.yml` (~510 lines current; ~130 lines staged-delta) | ~7,000 |
| `.github/workflows/post-merge.yml` | ~2,000 |
| `tests/ci-gate/*.sh` (9 files) | ~4,000 |
| `Justfile` (recipe section) | ~500 |
| Total | ~17,000 |

Within the 30% context window budget.

---

## §Tasks

1. **Read ci.yml, post-merge.yml, and all `tests/ci-gate/test_AC-*.sh` files in full** before
   any edits. Never edit a test assertion without reading what it asserts against.

2. **Human gates first:** confirm with the human that the staged ci.yml/e2e.yml residue has
   been dispositioned before starting AC-007 work.

3. **AC-001 [SUPERSEDED — NO-OP]:** AC-001 (test_AC-3 RUNNERS array macos-13→macos-15-intel)
   was superseded by S-REL-001 v0.8 (F-REL001-P6-001). S-REL-001 delivers both the ci.yml
   runner change and the test script update. Skip this task; do NOT modify test_AC-3 here.

4. **AC-002:** Read test_AC-4. Read ci.yml `deny` and `audit` job sections. Determine
   whether `cargo deny check` appears as a standalone `run: cargo deny check` line or
   within a `|` block. Update the grep pattern accordingly. Fix ordering checks to scope
   to the correct job boundaries. Run test and confirm passes.

5. **AC-003:** Read test_AC-5. Read post-merge.yml in full. Map each failing assertion to
   the actual post-merge.yml content. Fix assertions that test for content that genuinely
   exists (just with different syntax/structure). Document any genuine gaps (content missing
   from post-merge.yml that the test asserts should be there) as a separate issue. Run test
   and confirm all assertions either pass or are documented gaps.

6. **AC-004:** Read test_AC-9. Update the heuristic per AC-004 specification. Run test on
   ci.yml, post-merge.yml, and release.yml to confirm zero false-positives on SHA-pinned
   `uses:` entries and that the test would still catch an actual hardcoded token.

7. **AC-005:** Add `test-ci-gate` recipe to Justfile. Run `just --list` to confirm recipe
   appears. Add `ci-gate` job to ci.yml. Run `just test-ci-gate` locally to confirm it
   exercises the full suite and exits 0 (all ACs 001–004 must be green before this step).

8. **AC-006:** Replace string-based `$EXCLUDE_ARGS` with bash array in semver-checks job.
   Run `actionlint .github/workflows/ci.yml` (if available locally) to confirm SC2086 is
   resolved. Confirm no new findings introduced.

9. **Self-check:** run `just test-ci-gate` one final time across all test files. Confirm
   all pass. Run `just check-fast` to confirm no incidental Justfile breakage.

---

## §Previous Story Intelligence

- **S-0.01 (merged):** authored `tests/ci-gate/` suite with 9 test scripts and run.sh
  aggregator. Left `just test-ci-gate` and ci.yml wiring as a TODO with an explicit comment
  ("once Justfile target is added by implementer"). This story closes that leftover.

- **S-MAINT-CI-DISK-EXHAUSTION-001 (merged, PR #224):** established the `behavioral_contracts: []`
  convention for CI-toolchain-only maintenance stories. The `verify-workflow-structure` job
  in ci.yml (added by S-MAINT-CI-DISK-EXHAUSTION-001) is the in-workflow analog of the
  external ci-gate suite. The ci-gate suite tests broader structural properties; the
  verify-workflow-structure job tests disk-hardening specifics inline. They are complementary.

- **S-REL-001 (TDD in progress, 2026-07-19):** triggered this story via LOCAL cascade
  findings. S-REL-001 touches release.yml; this story touches ci.yml + post-merge.yml +
  tests/ci-gate/. Parallel worktrees must not conflict. This story's AC-001 must be
  co-ordinate with S-REL-001's ci.yml runner change (if any).

---

## §Architecture Compliance Rules

- **Single-commit-per-burst (TD-VSDD-053).** All `tests/ci-gate/` edits and any ci.yml/Justfile
  changes are committed in a single atomic commit per burst. No "Stage 1 / Stage 2" multi-commit
  patterns.

- **No `--no-verify` hooks (CLAUDE.md §Git Workflow).** lefthook pre-commit runs fmt + clippy +
  layout; pre-push runs `just check`. Both must pass before the feature PR is created.

- **ADR-050 (rustls-tls).** This story does not touch Cargo.toml. If any incidental dependency
  appears, `default-features = false, features = ["rustls-tls"]` is required. Not expected to
  apply here.

- **actionlint SC2086 fix must not introduce new actionlint violations.** Run actionlint on
  the modified ci.yml before committing (tool must be installed; `brew install actionlint`).

- **`just test-ci-gate` recipe must forward exit code.** A `@bash tests/ci-gate/run.sh` recipe
  that swallows exit code is a silent-failure (SID-1); use `bash tests/ci-gate/run.sh` with
  set -e or explicit `|| exit 1` to ensure non-zero exit propagates.

---

## §Library & Framework Requirements

| Tool | Source | Notes |
|------|--------|-------|
| `actionlint` | `brew install actionlint` (macOS) or [download-actionlint.bash](https://github.com/rhysd/actionlint/blob/main/docs/install.md) | No crates.io package; install via brew or official script. Research U4 from S-REL-001. |
| `bash` ≥ 3.2 | OS default | All ci-gate scripts use `#!/usr/bin/env bash` and bash 3.2+ compatible syntax. |
| `just` | Already in Justfile | `test-ci-gate` recipe must be added; no new tool install. |
| `grep` / `awk` | OS default | Test assertions use POSIX grep; no GNU-specific flags. |

---

## §File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `tests/ci-gate/test_AC-3_matrix-5-platforms.sh` | SUPERSEDED | [SUPERSEDED by S-REL-001 v0.8 F-REL001-P6-001] Runner-label refresh delivered by S-REL-001 in feature/S-REL-001; do NOT re-apply here (AC-001 superseded) |
| `tests/ci-gate/test_AC-4_cargo-audit.sh` | Modify | Fix cargo deny + ordering assertions (AC-002) |
| `tests/ci-gate/test_AC-5_kani-proofs.sh` | Modify | Fix post-merge assertions to match actual post-merge.yml (AC-003) |
| `tests/ci-gate/test_AC-9_no-hardcoded-secrets.sh` | Modify | Exclude SHA-pinned `uses:` lines from 40-char heuristic (AC-004) |
| `Justfile` | Modify | Add `test-ci-gate` recipe adjacent to existing test recipes (AC-005) |
| `.github/workflows/ci.yml` | Modify | Add `ci-gate` job (AC-005); fix semver-checks SC2086 (AC-006); AC-007 depends on human disposition |

Do NOT modify:
- `tests/ci-gate/run.sh` — no structural changes needed; the "once Justfile target..." comment
  may be updated to remove the forward-reference language, but the aggregator logic is correct.
- `tests/ci-gate/tap_lib.sh` — shared assertion library; do not modify unless a bug is found.
- Any `crates/` file — this story does not touch production Rust code.

---

## §Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `post-merge.yml` genuinely lacks a fuzz target that test_AC-5 asserts | Document the gap explicitly in the test as a `tap_fail` with a rationale comment; do NOT convert to `tap_skip` without human authorization. A failing gate is better than a vacuous pass. |
| EC-002 | Staged ci.yml/e2e.yml changes are committed before this story's AC-007 implementation | Re-read ci.yml in full after commit; all assertions must reflect the new committed state, not the pre-staged state. |
| EC-003 | `actionlint` not installed locally | ci.yml SC2086 fix can still be landed by correctness inspection; add a task comment that actionlint must be run in CI. The ci.yml job itself can run actionlint. |
| EC-004 | `just test-ci-gate` recipe is added but ci.yml ci-gate job fails due to missing tool dependencies | ci-gate suite is bash-only (no Rust toolchain). The ci-gate job must not `needs: clippy` or any other Rust-compilation job; it can run in parallel from checkout. |
| EC-005 | A future S-MAINT story modifies ci.yml in a way that breaks a ci-gate assertion | That story must update the relevant ci-gate assertion in the same PR. This is the intended enforcement loop created by AC-005. |

---

## Purity Classification

This story is CI-toolchain-only. No Rust source files are modified.

| Scope | Classification | Rationale |
|-------|---------------|-----------|
| `tests/ci-gate/*.sh` | Effectful | Reads filesystem, spawns subprocesses, emits TAP output to stdout |
| `Justfile` recipe | Effectful | Invokes bash subprocess; exit-code side-effect visible to caller |
| `.github/workflows/ci.yml` | Effectful | GitHub Actions runtime; all steps are I/O-bound |
| Production Rust crates | N/A — not touched | Pure-core / effectful-I/O boundary analysis does not apply |

No pure-core functions are introduced or changed by this story.

---

## §Dependency Graph

```
S-0.01 (merged) ──► S-MAINT-CIGATE-REMEDIATION-001
S-REL-001 (TDD) ──► (AC-001 co-ordination; depends_on)
S-MAINT-CI-DISK-EXHAUSTION-001 (merged) ──► (behavioral_contracts: [] precedent)
```

This story has no `blocks:` entries — it is a backlog maintenance item with no downstream
story that waits on it.

---

## §Traceability Notes

**BC Clause Coverage Matrix:** N/A — CI-toolchain-only story; no BCs applicable.

**Gap Register:** The staged ci.yml/e2e.yml residue (AC-007 precondition) is a project-level
human-decision gap, not a BC or story gap. Recorded here for traceability; resolution target
is the human disposition event (D-NNNN to be assigned).

---

## §Changelog

| Version | Date | Summary |
|---------|------|---------|
| 0.3 | 2026-07-19 | LOCAL pass-12 fix-burst: F-REL001-P12-OBS-002 [process-gap LOW] — AC-008 added (committed musl cross-compile smoke: ci.yml must build with --target x86_64-unknown-linux-musl including C++ toolchain prereq or documented exclusion rationale); acceptance_criteria_count 7→8 (7 active + 1 superseded) |
| 0.2 | 2026-07-19 | Coordination reconciliation: AC-001 marked SUPERSEDED by S-REL-001 v0.8 (F-REL001-P6-001 orchestrator-adjudicated in-scope); depends_on comment updated; acceptance_criteria_count annotated 6 active + 1 superseded; Origin prose updated; File Structure Requirements test_AC-3 row marked SUPERSEDED; Task 3 marked NO-OP |
| 0.1 | 2026-07-19 | Initial story creation (story-writer, S-REL-001 LOCAL cascade trigger) |
