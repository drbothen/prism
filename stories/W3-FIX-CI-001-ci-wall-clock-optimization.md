---
story_id: W3-FIX-CI-001
title: "CI wall-clock optimization — cargo-nextest, per-platform PROPTEST_CASES, mold linker"
wave: 3
level: "L4"
target_module: workspace-tooling
subsystems: [SS-00]
priority: P1
depends_on: [W3-FIX-LEFTHOOK-001]
blocks: []
estimated_days: 1
points: 5
status: ready
document_type: story
version: "1.0"
producer: story-writer
timestamp: "2026-04-30T22:00:00Z"
input-hash: "[live-state]"
inputs:
  - .github/workflows/ci.yml
  - .factory/research/2026-04-30-ci-free-tier-optimization.md
  - lefthook.yml
  - justfile
traces_to: []
cycle: "v1.0.0-greenfield"
epic_id: "E-3.5"
phase: 3
behavioral_contracts: []
# BC status: pending PO authorship — no formal BC governs dev toolchain speed.
# This story is tooling-only; no production behavior is affected.
verification_properties: []
anchor_bcs: []
anchor_capabilities: [CAP-DEV-SPEED]
anchor_subsystem: ["SS-00"]
tdd_mode: lenient
assumption_validations: []
risk_mitigations: []
---

# W3-FIX-CI-001: CI wall-clock optimization — cargo-nextest, per-platform PROPTEST_CASES, mold linker

## Narrative

As a Prism CI maintainer, I want PR feedback in under 45 min wall-clock (down from
current ~70 min) so that developers are unblocked sooner and the workflow stays in
their flow window.

## Problem Statement

The current long-pole in the CI matrix is the Windows MSVC test job at ~70 min
wall-clock per PR. This is above the 45-minute threshold where the CI wait crosses
from "go do something else briefly" into "context switch and lose the thread."

**Repo is PUBLIC** — unlimited free minutes for standard GitHub-hosted runners apply.
Cost is not a constraint. The goal is pure wall-clock speed for developer iteration.

**Root cause breakdown (Windows, ~70 min total):**
1. Compile phase: ~40-45 min (Windows linker is slow; no mold equivalent on MSVC)
2. Test phase: ~25-30 min — `cargo test` runs tests sequentially within a binary and
   serially across binaries; proptest runs 1000 cases on every platform including
   Windows where each test iteration is slower than Linux

**Linux baseline (current ~25 min) is also improvable:**
- Parallel test execution via cargo-nextest reduces the test phase by 2-2.5x
- mold linker reduces link time on cold-cache builds by 2-5 min

**Research source:** `.factory/research/2026-04-30-ci-free-tier-optimization.md`
(committed at ca2846b5). All benchmarks, compatibility notes, and YAML diffs are
derived from that document.

## Objective

Reduce CI wall-clock by:

1. **cargo-nextest** (all 5 platforms): replaces `cargo test` with parallel test
   execution; 2.0-2.5x speedup on the test phase. Doctests split to a separate
   `cargo test --doc` step on linux-gnu only (nextest does not run doctests).
2. **Per-platform PROPTEST_CASES** (matrix field): linux-gnu keeps 1000 for full
   statistical power; other 4 platforms drop to 256 (proptest default — adequate for
   platform-specific regression coverage; regression seeds replay from
   `proptest-regressions/` regardless of case count).
3. **mold linker on Linux** (linux-gnu + linux-musl): 2-5 min link-time savings on
   cold-cache builds. Installed via `rui314/setup-mold@v1`; configured via RUSTFLAGS
   explicitly (`-C link-arg=-fuse-ld=mold`) for reliability over symlink approach.

**Expected outcome:**
- Windows long-pole: ~70 min -> ~45 min (nextest 2x test phase + proptest 256 cases)
- Linux gnu/musl: ~25 min -> ~15-18 min (nextest + mold)
- macOS: ~30 min -> ~22 min (nextest + proptest 256 cases)

## Acceptance Criteria

### AC-001: Windows wall-clock under 45 min

The `Test (x86_64-pc-windows-msvc)` job completes in under 45 min wall-clock on a
real PR after merge. Implementer records before/after timings in the PR description
(use a test PR or observe on the next natural PR after merge).

### AC-002: All platforms still pass — no regressions

All 5 platforms pass in CI after the change. `cargo nextest run` covers the same test
universe as the previous `cargo test --workspace --all-features` invocation.

### AC-003: Doctests still run on linux-gnu only

A separate step `cargo test --workspace --all-features --doc` runs after nextest on
the `x86_64-unknown-linux-gnu` matrix entry (controlled by `run_doctests: true`
matrix field). No doctest step runs on the other 4 platforms.

### AC-004: PROPTEST_CASES=1000 on linux-gnu; PROPTEST_CASES=256 on other 4 platforms

The matrix includes a `proptest_cases` field per entry:
- `x86_64-unknown-linux-gnu`: `proptest_cases: 1000`
- All other 4 targets: `proptest_cases: 256`

The workflow injects `PROPTEST_CASES: ${{ matrix.proptest_cases }}` via the job-level
`env:` block. The `proptest-regressions/` directory is committed to the repo so that
previously-found seeds replay on all platforms regardless of case count.

### AC-005: mold linker active on linux-gnu and linux-musl

The `rui314/setup-mold@v1` action runs (with `make-default: false`) on Linux runners.
RUSTFLAGS is set to `-C link-arg=-fuse-ld=mold` for Linux jobs. Build logs confirm
mold is invoked (search for `mold` in the linker invocation line).

### AC-006: PR annotations still work — nextest JUnit reporter integrated

A `.config/nextest.toml` file at workspace root defines a `[profile.ci]` with JUnit
output at `target/nextest/ci/junit.xml`. The workflow uploads the JUnit artifact via
`actions/upload-artifact@v4` after tests (using `if: always()`). PR annotations
continue to work via the existing JUnit upload mechanism.

### AC-007: Local `just check` continues to pass

The Justfile `check` target (modified by W3-FIX-LEFTHOOK-001) is unaffected by this
story. `just check` still exits 0 after this CI change. No Justfile changes are
required unless the implementer opts to add nextest fallback (Task 4 below).

### AC-008: verify-workflow-structure job continues to pass

The existing `verify-workflow-structure` job in `ci.yml` must pass without
modification. Its AC-5 assertion (`TARGET_COUNT >= 5 targets`) continues to hold
because the 5-platform matrix structure is preserved.

## Tasks

- [ ] **1. Add `.config/nextest.toml`** at workspace root:
  ```toml
  [profile.ci]
  fail-fast = false
  slow-timeout = { period = "60s", terminate-after = 2 }
  final-status-level = "slow"
  failure-output = "immediate-final"

  [profile.ci.junit]
  path = "junit.xml"
  store-success-output = false
  store-failure-output = true
  ```

- [ ] **2. Modify `ci.yml` — test job steps:**
  - Add `proptest_cases` and `run_doctests` fields to each matrix `include:` entry:
    - `x86_64-unknown-linux-gnu`: `proptest_cases: 1000`, `run_doctests: true`
    - `x86_64-unknown-linux-musl`: `proptest_cases: 256`
    - `aarch64-apple-darwin`: `proptest_cases: 256`
    - `x86_64-apple-darwin`: `proptest_cases: 256`
    - `x86_64-pc-windows-msvc`: `proptest_cases: 256`
  - Add job-level `env:` block:
    ```yaml
    env:
      PROPTEST_CASES: ${{ matrix.proptest_cases }}
    ```
  - Add `taiki-e/install-action` step for nextest (before the test run step):
    ```yaml
    - name: Install nextest
      uses: taiki-e/install-action@cf525cb33f51aca27cd6fa02034117ab963ff9f1 # v2.75.22
      with:
        tool: nextest
    ```
  - Replace `run: cargo test --workspace --all-features` with:
    ```yaml
    - name: Run tests (nextest)
      run: cargo nextest run --workspace --all-features --profile ci
    ```
  - Add doctest step after nextest:
    ```yaml
    - name: Run doctests (linux-gnu only)
      if: matrix.run_doctests
      run: cargo test --workspace --all-features --doc
    ```
  - Add JUnit upload step:
    ```yaml
    - name: Upload JUnit results
      if: always()
      uses: actions/upload-artifact@v4
      with:
        name: junit-${{ matrix.target }}
        path: target/nextest/ci/junit.xml
        retention-days: 7
    ```

- [ ] **3. Add mold linker steps to the test job (Linux only):**
  - Add `rui314/setup-mold@v1` step (Linux-gated, after toolchain, before rust-cache):
    ```yaml
    - name: Install mold linker (Linux only)
      if: runner.os == 'Linux'
      uses: rui314/setup-mold@v1
      with:
        make-default: false
    ```
  - Add `RUSTFLAGS` to the env block (extend the block from step 2):
    ```yaml
    env:
      PROPTEST_CASES: ${{ matrix.proptest_cases }}
      RUSTFLAGS: ${{ runner.os == 'Linux' && '-C link-arg=-fuse-ld=mold' || '' }}
    ```
  - Verify `rocksdb-sys` still links correctly (check build log for mold invocation
    on linux-gnu; expect no undefined symbol errors).

- [ ] **4. (Optional) Update `just check` for nextest parity:**
  - If nextest is installed locally, `just check` uses `cargo nextest run --workspace
    --all-features --profile ci`; otherwise falls back to `cargo test`. Add a comment
    noting the fallback logic. This is a nice-to-have — do not block on it.
  - Add `cargo install cargo-nextest --locked` to `docs/dev-setup.md`.

- [ ] **5. Verify `proptest-regressions/` is committed:**
  - Run `git ls-files proptest-regressions/` and confirm the directory is tracked.
  - If untracked, add any existing `*.txt` regression files and update `.gitignore` to
    NOT ignore `proptest-regressions/`.

- [ ] **6. Record before/after timings in PR description:**
  - Record wall-clock for all 5 platforms from the last CI run before merging this
    story vs. the first CI run after. Table format:
    | Platform | Before | After |
    |---|---|---|
    | linux-gnu | ~25 min | |
    | linux-musl | ~25 min | |
    | macOS aarch64 | ~30 min | |
    | macOS x86 | ~30 min | |
    | Windows MSVC | ~70 min | |

- [ ] **7. Confirm `verify-workflow-structure` passes unchanged:**
  - Do not modify the `verify-workflow-structure` job.
  - Confirm the AC-5 grep (`TARGET_COUNT >= 5`) still passes after the matrix
    modifications (5 `target:` entries must remain at 12-space indentation).

## Do NOT Modify

- The 7-step `just check-ci` order (fmt -> clippy -> test -> deny -> audit ->
  semver-checks -> check-layout). These steps are defined in Justfile; this story
  does not touch them.
- Workflow concurrency / cancel-in-progress semantics (`concurrency:` block).
- The 5-platform matrix structure — only add per-platform tuning fields.
- The `test-no-default-features`, `deny`, `audit`, `semver-checks`, and
  `verify-workflow-structure` jobs — these are unchanged.
- Pinned action commit SHAs — preserve all existing `@<sha> # vN.N.N` pins for
  existing actions; for `rui314/setup-mold@v1` and the nextest install-action step,
  pin at the current SHA of the tag.

## Testing Strategy

This is a tooling-only story. No production Rust crate behavior changes. Validation:

1. **Before:** Record CI wall-clock times for all 5 platforms on a recent PR (or on
   the develop branch push just before merging this story).
2. **Apply changes** (nextest, proptest_cases matrix, mold, nextest.toml).
3. **After:** Record CI wall-clock times for the first PR that runs after merge.
4. **Check:** All 5 platforms green; doctest step runs on linux-gnu; JUnit artifacts
   uploaded; mold invoked on Linux (grep build log for "mold").
5. **Check:** `verify-workflow-structure` job passes.
6. **Check:** proptest-regressions/ directory tracked in git.

## Risk Mitigation

**Risk: nextest does not run doctests — doctest regressions silently pass.**

Mitigation: AC-003 mandates a separate `cargo test --doc` step on linux-gnu. Doctests
are platform-invariant (no syscall surface) so linux-gnu-only coverage is sufficient.

**Risk: PROPTEST_CASES=256 on non-Linux platforms misses a platform-specific
regression that 1000 cases would catch.**

Mitigation: `proptest-regressions/` seeds are committed; all previously-found
regression seeds replay on every platform regardless of case count. New low-probability
platform-specific bugs (failure rate <1%) would require 256+ cases to catch — the
statistical miss rate at 256 cases for a 1%-failure-rate bug is ~7.5%. This is
acceptable for the non-canonical platforms; linux-gnu at 1000 cases remains the
authoritative sweep. A nightly `PROPTEST_CASES=5000` workflow is a recommended
follow-up (not in scope for this story).

**Risk: mold breaks rocksdb-sys linking.**

Mitigation: mold has been validated against Chromium (100MB+ C++ binary). rocksdb-sys
uses static C++ archives that mold handles correctly. First CI run will confirm; if
linking fails, the `rui314/setup-mold` step can be reverted without touching the rest
of the story.

**Risk: serial_test / within-process shared state breaks under nextest process-per-test model.**

Mitigation: nextest runs each test binary in its own process but NOT each test
function in a separate process (that is `--test-threads=1` per binary, not
process-per-test in the function sense). Tests within a binary still share process
state within that binary. If any test uses `lazy_static!` shared mutable state
cross-function, it may be affected. Audit for `lazy_static` + `Mutex` patterns in
test fixtures before merge.

**Risk: RUSTFLAGS=-C link-arg=-fuse-ld=mold conflicts with existing RUSTFLAGS set
elsewhere.**

Mitigation: Check existing `RUSTFLAGS` env usage in ci.yml. The current workflow has
no job-level RUSTFLAGS set. If a future story adds RUSTFLAGS, the two must be merged
(space-separated). Document this constraint in a comment on the env block.

## Architecture Mapping

| Component | Module | Pure/Effectful | Notes |
|-----------|--------|---------------|-------|
| `.config/nextest.toml` | workspace-tooling | Pure (config) | New file; defines CI profile + JUnit output |
| `.github/workflows/ci.yml` | workspace-tooling | Effectful (shell) | Modify test job only |
| `docs/dev-setup.md` | workspace-tooling | Pure (docs) | Optional: document nextest install |

No production Rust crates are modified by this story.

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| `.config/nextest.toml` | pure-core | Static TOML configuration; no runtime side effects; read by nextest at invocation |
| `.github/workflows/ci.yml` (test job) | effectful-shell | Executes shell commands on GitHub runners; installs tools, runs cargo, uploads artifacts |
| `docs/dev-setup.md` | pure-core | Static documentation; no side effects |
| `rui314/setup-mold` action step | effectful-shell | Installs the mold linker binary to /usr/local/bin/mold on the runner |
| RUSTFLAGS env injection | pure-config | Environment variable passed to rustc; no direct side effect beyond influencing linker selection |

## Subsystem Anchor Justification

SS-00 owns this story's scope because it covers workspace-level CI pipeline tooling
(ci.yml, nextest.toml, linker configuration) with no production domain behavior,
per the ARCH-INDEX Subsystem Registry definition of SS-00 as workspace/toolchain
infrastructure.

## Dependency Anchor Justification

`depends_on: [W3-FIX-LEFTHOOK-001]` — this story modifies the CI pipeline to adopt
nextest and per-platform proptest tuning. W3-FIX-LEFTHOOK-001 sets
`PROPTEST_CASES=100` on local pre-push and documents the local/CI split. Both stories
touch the proptest case-count story; W3-FIX-LEFTHOOK-001 must land first so that the
local and CI proptest counts are documented coherently (local: 100, CI linux-gnu:
1000, CI others: 256) and developers understand the three-tier model after both are
merged.

## File Structure Requirements

| File | Action | Change |
|------|--------|--------|
| `.config/nextest.toml` | Create | New file; CI profile with JUnit output |
| `.github/workflows/ci.yml` | Modify | test job: nextest, proptest_cases, mold, doctest step, JUnit upload |
| `proptest-regressions/` | Verify | Confirm tracked in git; add any untracked files |
| `docs/dev-setup.md` | Modify (optional) | Add `cargo install cargo-nextest --locked` |

## Architecture Compliance Rules

- No production crate source files (`crates/**/src/**`) may be modified.
- No changes to `Cargo.toml` or `Cargo.lock` other than those that cargo resolves
  automatically (none expected — nextest is installed via `taiki-e/install-action`,
  not as a Cargo.toml dependency).
- The `test-no-default-features`, `deny`, `audit`, `semver-checks`, and
  `verify-workflow-structure` jobs in `ci.yml` must not be modified.
- The `verify-workflow-structure` job's AC-5 grep (`TARGET_COUNT >= 5`) must
  continue to pass after this story's matrix modifications.
- Preserve all existing action SHA pins. New actions (`rui314/setup-mold`) must be
  pinned at a specific commit SHA, not a mutable tag.
- The CI pipeline order (fmt -> clippy -> test -> deny/audit/semver parallel) must
  be preserved.

## Library & Framework Requirements

| Tool | Version / Pin | Notes |
|------|--------------|-------|
| cargo-nextest | latest via `taiki-e/install-action@cf525cb33f51aca27cd6fa02034117ab963ff9f1` | Prebuilt binary; no compile cost |
| rui314/setup-mold | `@v1` (pin to current v1 SHA before merge) | Linux only; `make-default: false` |
| actions/upload-artifact | `@v4` | Already used elsewhere in ci.yml (verify SHA pin) |
| taiki-e/install-action | `@cf525cb33f51aca27cd6fa02034117ab963ff9f1` (v2.75.22) | Already pinned in ci.yml for cargo-audit |

## Token Budget Estimate

| Item | Estimated tokens |
|------|-----------------|
| This story file | ~3 500 |
| `.github/workflows/ci.yml` (read + modify) | ~2 500 |
| `.config/nextest.toml` (create) | ~200 |
| Research file (reference) | ~5 000 |
| `docs/dev-setup.md` (optional read/modify) | ~500 |
| Implementation scratch + lint output | ~1 000 |
| **Total** | **~12 700** |

Well within a single agent context window. No splitting required.

## Previous Story Intelligence

**W3-FIX-LEFTHOOK-001** (depends-on): Establishes the three-tier proptest count model
(local: 100, CI: depends on platform). This story completes the CI tier by setting
1000 on linux-gnu and 256 on other platforms. The `check-ci` Justfile target added by
W3-FIX-LEFTHOOK-001 documents `PROPTEST_CASES=1000` for local CI-parity runs — this
story's CI change does NOT affect the `check-ci` target (CI runs cargo directly, not
via Justfile).

**W3-FIX-WIN-001** (sibling, no dependency): Windows port-release cross-platform test
fix. No toolchain or workflow overlap.

**Key lesson from W3-FIX-LEFTHOOK-001:** Tooling stories that touch proptest case
counts MUST document the three-tier model clearly in code comments, or the next
developer will be confused by 100 vs 256 vs 1000 appearing in different places.
Add a comment block to the ci.yml env section explaining the tier rationale.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `proptest-regressions/` directory absent from repo | Before merge, run `git ls-files proptest-regressions/`; if absent, this is a gap. Create the dir with a `.gitkeep` and ensure `.gitignore` does not exclude it |
| EC-002 | nextest version mismatch causes test binary discovery failure | Install via `taiki-e/install-action` always fetches the latest stable nextest; should be stable. If binary discovery fails, the CI error message will identify the specific binary |
| EC-003 | mold not found after setup-mold action | The action installs to `/usr/local/bin/mold`; RUSTFLAGS `-fuse-ld=mold` tells `cc` to invoke it. If mold is missing, the link step fails with a clear error |
| EC-004 | Existing `RUSTFLAGS` env in ci.yml conflicts with mold flag | Current ci.yml has no job-level RUSTFLAGS. If a future story adds one, the two must be merged. Document this in a comment on the env block |
| EC-005 | `verify-workflow-structure` AC-5 grep breaks after matrix field additions | The grep counts `target:` lines at 12-space indentation. New matrix fields (`proptest_cases`, `run_doctests`) use different key names and will not match the grep pattern. Safe |
| EC-006 | rocksdb-sys fails to link under mold | Extremely unlikely (mold supports all major C++ static linking patterns). If it occurs, add `if: matrix.target == 'x86_64-unknown-linux-gnu'` to the mold step as a temporary escape hatch while investigating |
| EC-007 | JUnit XML not found at expected path on test failure | nextest writes to `target/nextest/ci/junit.xml` only when tests run; if tests fail before generating any output, the artifact upload step may fail silently. `if: always()` on the upload step handles partial runs |
| EC-008 | Developer runs `just check` and nextest is not installed | `just check` (modified by W3-FIX-LEFTHOOK-001) uses plain `cargo test`. The optional Task 4 nextest fallback in `just check` is a nice-to-have; the AC does not require it |
