---
document_type: story
story_id: S-MAINT-CI-DISK-EXHAUSTION-001
title: "CI disk-exhaustion hardening — preflight + free-disk-space + CARGO_PROFILE_DEV_DEBUG=1 + failure annotation"
wave: tbd
epic_id: maintenance
priority: P2
status: draft
version: "0.1"
level: ops
producer: story-writer
timestamp: "2026-07-15"
modified: "2026-07-15"
input-hash: "[live-state]"
inputs:
  - .github/workflows/ci.yml
traces_to: ""
cycle: "v1.0.0-greenfield"
phase: 3
tdd_mode: strict
track: "Platform Engineering"
subsystems: []
# Subsystem anchor: this story touches only the CI toolchain (.github/workflows/ci.yml),
# not any functional product subsystem. No SS-NN anchor is appropriate; devops/CI stories
# (e.g., W3-FIX-CI-001, S-MAINT-REQWEST-RUSTLS-GATE-001) follow the same pattern of
# subsystems: [] per the ARCH-INDEX Subsystem Registry, which defines SS-01..22 as product
# subsystems only. The CI toolchain has no registered SS-NN.
crates_touched: []
target_module: devops
behavioral_contracts: []
# BC status: pending PO authorship — no formal BC governs CI runner disk utilization.
# This story is CI toolchain-only; no production behavior is affected. Status must
# remain draft until a PO authors BCs (S-7.01 spec-first gate).
verification_properties: []
depends_on: []
blocks: []
points: 5
estimated_days: 1
risk: MEDIUM
acceptance_criteria_count: 5
red_gate_tests: 2
estimated_passes: "1"
holdout_scenarios: []
assumption_validations: []
triggered_by: "D-1780 watch-note (3rd disk-exhaustion occurrence on PR #223 CI runs 2026-07-15)"
---

# S-MAINT-CI-DISK-EXHAUSTION-001: CI disk-exhaustion hardening — preflight + free-disk-space + CARGO_PROFILE_DEV_DEBUG=1 + failure annotation

## §Origin

D-1780 recorded a watch-note: "3rd disk-exhaustion occurrence → maintenance story." This
story materializes that commitment. Three consecutive GitHub-hosted-runner disk-full
failures occurred on PR #223 during the `cargo nextest run --workspace --all-features
--profile ci` build phase — before any test executed:

| Run | Target | HEAD | Error |
|-----|--------|------|-------|
| 29394488318 | x86_64-unknown-linux-musl | 76c0fa60 (2026-07-15) | `mold: failed to write to an output file. Disk full?` + `rustc-LLVM ERROR: IO failure on output stream: No space left on device` |
| 29399778005 | x86_64-unknown-linux-gnu | 72d8ed8d (2026-07-15) | `couldn't create a temp dir: No space left on device (os error 28)` |
| 29404746333 | x86_64-unknown-linux-gnu | 97cb070e (2026-07-15) | `mold: Disk full` + `collect2: fatal error: ld terminated with signal 7 [Bus error]` |

All three re-runs succeeded. Failures occurred exclusively on Linux legs (musl + gnu),
which use the mold linker (W3-FIX-CI-001) and carry the largest debug-info artifact
footprint.

## §Root Cause Hypothesis

GitHub `ubuntu-latest` hosted runners provision approximately 14 GB of free disk space
by default. A 26-crate Rust workspace compiled in dev mode (full DWARF debug symbols)
generates large `.d`, `.rlib`, `.rmeta`, and `.o` artifacts under `target/`. The
`Swatinem/rust-cache` action restores prior artifacts, which reduces compile time but
increases the effective starting disk footprint — the cache restore consumes disk before
the incremental build begins. Under high-concurrency PR activity, a runner may land
with materially less than 14 GB available.

Re-runs succeed because the failed run wrote partial artifacts to the cache; the
re-run increments on those artifacts and does less linking work, consuming less peak
disk.

**This is load-dependent and therefore flaky** — it does not occur on every run, only
when the runner disk is near the threshold.

## §Narrative

As a Prism CI maintainer, I want the Linux Test jobs to reliably complete without
disk-exhaustion failures so that contributors are not blocked by spurious CI failures
that require manual re-runs and obscure genuine test failures.

## §Acceptance Criteria

### AC-001 — Disk-free preflight step present in Test job

A step named "Report initial disk space" (or equivalent) runs at the START of the Test
job, before `actions/checkout`, and emits `df -h` output to the job log. This provides
a baseline disk snapshot for every run, enabling post-hoc analysis of whether a future
failure was disk-related.

The `verify-workflow-structure` job gains an assertion confirming this step is present
in `ci.yml` (Red Gate test 1):

```bash
grep -qE 'Report initial disk space|df -h' .github/workflows/ci.yml || {
  echo "::error::S-MAINT-CI-DISK-EXHAUSTION-001 AC-001: disk preflight step missing from ci.yml"
  exit 1
}
```

### AC-002 — Pre-build disk reclaim targeting ≥25 GB free

A disk-reclaim step using `jlumbroso/free-disk-space` (pinned to a specific commit SHA)
runs in the Test job after `actions/checkout` but before the `Swatinem/rust-cache`
restore and build phase. It removes preinstalled toolsets not needed by the Rust build:
Android SDK (~8 GB), Haskell GHC (~3 GB), .NET SDK (~4 GB), and large apt/docker
package caches. Total expected reclaim: 14–18 GB.

A verification step immediately after the reclaim step confirms at least 25 GB free
via `df -h`. If post-reclaim free space is below 25 GB, the job fails early with a
diagnostic message rather than proceeding to a late-stage OOM/disk-full linker crash:

```bash
AVAIL_GB=$(df / | awk 'NR==2 { gsub(/G/, "", $4); print int($4 / 1024 / 1024) }')
[ "$AVAIL_GB" -ge 25 ] || {
  df -h
  echo "::error::Disk reclaim insufficient: only ${AVAIL_GB} GB free (need ≥25 GB). See S-MAINT-CI-DISK-EXHAUSTION-001."
  exit 1
}
```

The `verify-workflow-structure` job gains an assertion confirming the free-disk-space
action is present in `ci.yml` (Red Gate test 2):

```bash
grep -qE 'free-disk-space' .github/workflows/ci.yml || {
  echo "::error::S-MAINT-CI-DISK-EXHAUSTION-001 AC-002: free-disk-space action missing from ci.yml"
  exit 1
}
```

### AC-003 — Build-artifact footprint bounding via CARGO_PROFILE_DEV_DEBUG=1

The Test job's `env:` block gains `CARGO_PROFILE_DEV_DEBUG: 1`. In Rust ≥ 1.71 this
maps to `debug = "line-tables-only"`, producing minimal DWARF sections (source file and
line numbers only) rather than the full variable/type information that drives large
artifact footprint. This preserves source-location backtraces in test failure output
while eliminating the 10-20× size multiplier of full debug info.

A comment at the env block documents the mapping and the rationale:

```yaml
# CARGO_PROFILE_DEV_DEBUG=1: maps to "line-tables-only" (Rust ≥1.71).
# Preserves file:line in backtraces; eliminates full DWARF for smaller
# target/ artifacts on runners with limited disk. S-MAINT-CI-DISK-EXHAUSTION-001 AC-003.
# DO NOT use 0 (no debug) — that removes backtraces entirely.
CARGO_PROFILE_DEV_DEBUG: 1
```

The implementer records approximate before/after `du -sh target/` for a local dev
debug build (with and without `CARGO_PROFILE_DEV_DEBUG=1`) in the PR description as
size-reduction evidence.

### AC-004 — Disk-exhaustion failure annotation step

A `if: failure()` step at the END of the Test job runs `df -h` and emits a
`::warning::` workflow command if disk utilization is above 95%:

```bash
USED_PCT=$(df / | awk 'NR==2 { gsub(/%/, "", $5); print $5 }')
df -h
if [ "${USED_PCT}" -ge 95 ]; then
  echo "::warning::Disk near-full (${USED_PCT}% used) on runner — likely S-MAINT-CI-DISK-EXHAUSTION-001 class failure. Evidence runs: 29394488318 / 29399778005 / 29404746333 (PR #223, 2026-07-15)."
fi
```

This makes disk-exhaustion failures distinguishable in the GitHub Actions job summary
from genuine test failures without requiring log scraping.

### AC-005 — Regression evidence: three consecutive green CI runs

After the `.github/workflows/ci.yml` changes are pushed to a maintenance branch
`maintenance/ci-disk-hardening`, three consecutive full-workspace PR CI runs complete
green (all Linux legs pass without re-run) before the PR is merged. The PR description
records the three GitHub Actions run IDs as evidence.

The changes in this story MUST ride `maintenance/ci-disk-hardening` and NOT be merged
through any open defect PR branch (e.g., `fix/DEFECT-*`), to keep CI infrastructure
changes cleanly bisectable.

## §Implementation Notes

**Free-disk-space action choice and SHA pinning:** `jlumbroso/free-disk-space` is the
community-standard action for ubuntu-latest disk reclaim. Configure it to remove
`android: true`, `haskell: true`, `dotnet: true`; leave Docker unless build logs show
it contributes to the failure. The action MUST be pinned to a specific commit SHA with
a `# vN.N.N` comment before merge — identical requirement to all other actions in
`ci.yml`. Fetch the current commit SHA for the desired tag at PR-author time.

**CARGO_PROFILE_DEV_DEBUG=1 scope:** Set on the Test job's `env:` block, NOT at
workflow level, to avoid affecting unrelated jobs (fmt, clippy, deny). Both the
`x86_64-unknown-linux-gnu` and `x86_64-unknown-linux-musl` legs need it; applying to
all five legs is also acceptable (macOS/Windows runners have more headroom but no
harm).

**Ordering constraint:** The free-disk-space step MUST run after `actions/checkout`
(so git metadata is intact) and BEFORE `Swatinem/rust-cache` (so the cache restore does
not fill disk before reclaim). The preflight `df -h` step runs BEFORE checkout as a
pristine baseline.

**verify-workflow-structure assertions:** Add the two new grep assertions to the
existing `run:` block of the `verify-workflow-structure` job. Follow the exact pattern
of the existing assertions (error message includes AC reference and story ID). The
`verify-workflow-structure` job itself must NOT otherwise be modified.

**CARGO_PROFILE_DEV_DEBUG merging:** The Test job's `env:` block currently sets
`PROPTEST_CASES` and `RUSTFLAGS`. Add `CARGO_PROFILE_DEV_DEBUG` as a third entry.
Do not replace either existing key. Comment explains all three entries.

## §Token Budget Estimate

| Item | Tokens (approx.) |
|------|-----------------|
| This story spec | ~3 k |
| `.github/workflows/ci.yml` (read + modify Test job + verify-workflow-structure job) | ~6 k |
| `jlumbroso/free-disk-space` README / SHA lookup | ~1 k |
| Implementation scratch + comments | ~1 k |
| **Total** | **~11 k** |

Well within a single agent context window; no splitting required.

## §Tasks

- [ ] Create maintenance branch `maintenance/ci-disk-hardening` from `develop`
- [ ] Read `.github/workflows/ci.yml` in full (especially the Test job and `verify-workflow-structure` job)
- [ ] Fetch the current commit SHA for `jlumbroso/free-disk-space` (latest stable tag)
- [ ] Add "Report initial disk space" step (`run: df -h`) as the FIRST step of the Test job (before `actions/checkout`)
- [ ] Add `jlumbroso/free-disk-space@<SHA> # vN.N.N` step to Test job after checkout and before rust-cache; configure `android: true, haskell: true, dotnet: true`
- [ ] Add "Verify ≥25 GB free" step immediately after the free-disk-space step (see AC-002 snippet)
- [ ] Add `CARGO_PROFILE_DEV_DEBUG: 1` to the Test job `env:` block with the standard comment (AC-003); preserve existing `PROPTEST_CASES` and `RUSTFLAGS` entries
- [ ] Add `if: failure()` disk-annotation step at the END of the Test job (after JUnit upload; see AC-004 snippet)
- [ ] Add the two new grep assertions (AC-001 + AC-002) to the `verify-workflow-structure` job's existing `run:` block, following the pattern of the existing `non-exhaustive-violation-compile-fail` and `wasm32-compile-check` assertions
- [ ] Record before/after `du -sh target/` sizes in the PR description (AC-003 evidence)
- [ ] Record three consecutive green CI run IDs in the PR description (AC-005 evidence)

## §Previous Story Intelligence

**W3-FIX-CI-001** (merged PR #112 a3bd5a0f): Introduced the mold linker on Linux legs.
Mold writes its output file in a single large write rather than incrementally, making
it more sensitive to near-full disk conditions than the legacy BFD linker. The error
`mold: Disk full` / `mold: failed to write to an output file. Disk full?` is the direct
symptom. This story does NOT remove mold — the performance benefit (2–5 min savings)
is retained; this story addresses the underlying disk capacity gap.

**S-MAINT-REQWEST-RUSTLS-GATE-001** (draft v0.1, D-1497): The canonical recent
maintenance story using the same frontmatter schema and section structure. No
functional overlap.

**Key lesson from W3-FIX-CI-001:** Every new GitHub Actions step MUST be pinned to a
specific commit SHA with a `# vN.N.N` comment — the project has zero mutable-tag
action references in `ci.yml`. Fetch the current SHA at PR-author time; do not use
`@main`, `@v1`, or any mutable reference.

**Key lesson from S-MAINT-REQWEST-RUSTLS-GATE-001:** Red Gate tests for CI toolchain
stories belong in the `verify-workflow-structure` job (structural assertions), not in
a separate test script. This matches the existing `non-exhaustive-violation-compile-fail`
and `wasm32-compile-check` assertion pattern established during S-PLUGIN-PREREQ-C.

## §Architecture Compliance Rules

- No production Rust crate source files (`crates/**/src/**`) may be modified
- No changes to `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, or `.config/nextest.toml`
- The `fmt`, `clippy`, `deny`, `audit`, `semver-checks`, `test-no-default-features`,
  and `non-exhaustive-violation-compile-fail` jobs in `ci.yml` must NOT be modified
- The `verify-workflow-structure` job's existing assertions (AC-5 `TARGET_COUNT >= 5`,
  AC-6 cargo-deny/audit, AC-7 semver, AC-8 no-default-features, non-exhaustive, wasm32
  checks) must ALL pass unchanged after this story's modifications; the two new
  assertions are additive only
- All new GitHub Actions steps must be pinned to a specific commit SHA with a `# vN.N.N`
  comment per the project SHA-pinning policy (every existing action in ci.yml is
  commit-pinned; no exceptions)
- `CARGO_PROFILE_DEV_DEBUG=0` is FORBIDDEN — it eliminates backtraces entirely; use
  `1` (line-tables-only) only
- The Test job's `env:` block additions must be additive (do NOT replace or rename
  existing `PROPTEST_CASES` or `RUSTFLAGS` entries)
- The free-disk-space step must appear AFTER `actions/checkout` and BEFORE
  `Swatinem/rust-cache` to ensure the reclaim occurs before cache restore fills disk
- This story's changes ride `maintenance/ci-disk-hardening`; they MUST NOT be merged
  through an open defect PR branch

## §Library & Framework Requirements

No new Rust dependencies. CI tooling only:

| Tool | Version constraint | Justification |
|------|--------------------|---------------|
| `jlumbroso/free-disk-space` | pin to specific commit SHA at PR-author time (not `@main`) | Community-standard ubuntu-latest disk reclaim; removes Android/Haskell/.NET |
| `CARGO_PROFILE_DEV_DEBUG` env var | value `1` (line-tables-only; Rust ≥ 1.71) | Reduces debug artifact size; pinned stable toolchain is well past 1.71 |
| `df` / `awk` | system utilities present on all ubuntu-latest runners | Used in preflight, ≥25 GB gate, and failure annotation steps |

No new `apt-get` packages. No Python packages. No compiled Cargo dev-dependencies.
No new GitHub Actions secrets required.

## §File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `.github/workflows/ci.yml` | MODIFY | Test job: add preflight, free-disk-space, ≥25 GB gate, `CARGO_PROFILE_DEV_DEBUG: 1` env entry, failure annotation; `verify-workflow-structure` job: add AC-001 + AC-002 assertions; no other jobs touched |

No new files required. The `jlumbroso/free-disk-space` action is invoked as an inline
`uses:` step; it does not require a new config file in the repository.

## §Forbidden Patterns

| Pattern | Reason |
|---------|--------|
| `CARGO_PROFILE_DEV_DEBUG: 0` | Eliminates all backtraces; `1` (line-tables-only) is the only acceptable value |
| `CARGO_PROFILE_RELEASE_*` modification | Release profile is not in scope; only dev/test builds are affected |
| `docker system prune -af` in CI steps | Removes Docker layers needed by other actions; unsafe without explicit scope guard |
| `sudo rm -rf /usr/share/dotnet` (direct removal) | Fragile manual removal; use `jlumbroso/free-disk-space` which handles ordering and dependencies correctly |
| Floating action reference (`@main`, `@v1`, `@latest`) | SHA-pinning policy; all ci.yml actions are commit-pinned |
| Mixing this change into an open defect PR branch | Branch isolation required per AC-005; complicates bisection |
| Modifying `fmt`, `clippy`, `deny`, `audit`, `semver-checks`, `test-no-default-features`, or `non-exhaustive-violation-compile-fail` jobs | Out of scope for this story; single-responsibility |
| Removing or renaming existing `PROPTEST_CASES` or `RUSTFLAGS` env entries | Existing entries must be preserved; only add `CARGO_PROFILE_DEV_DEBUG` |

## §Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `jlumbroso/free-disk-space` action fails or hangs | `continue-on-error: false` (default); if the action fails, the job fails before the expensive build — early failure is preferable to a late OOM linker crash |
| EC-002 | Post-reclaim disk still below 25 GB on an unusual runner topology | AC-002 gate exits 1 with a human-readable `::error::` message identifying available space; job fails early |
| EC-003 | `CARGO_PROFILE_DEV_DEBUG=1` breaks a test that requires full DWARF | Extremely unlikely in test builds; if it occurs, the test will error explicitly with a compiler or runtime message identifying the gap |
| EC-004 | A future story adds `CARGO_PROFILE_DEV_DEBUG` with a different value | Duplicate env key conflict will be visible in CI; the newcomer story author must merge with a comment (same resolution pattern as RUSTFLAGS merging) |
| EC-005 | `verify-workflow-structure` AC-5 `target:` grep matches a new step name | The existing grep counts `^            target:` at 12-space indent (matrix field name). New step `name:` fields use `      - name:` (6-space indent). No conflict. |
| EC-006 | macOS or Windows leg fails due to `CARGO_PROFILE_DEV_DEBUG` env var | Both platforms support this env var; no platform-specific issue expected. Setting on all legs is acceptable. |
| EC-007 | Preflight `df -h` step placed after `actions/checkout` instead of before | The preflight MUST be before checkout to capture the true baseline; checkout restores git metadata and may trigger cache actions. Verify ordering in the delivered YAML. |

## §Changelog

- v0.1 (2026-07-15): Initial draft — story-writer. D-1780 watch-note 3rd-occurrence materialization. 5 ACs, 2 Red Gate tests, 5 pts, P2.
