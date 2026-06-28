---
document_type: story
story_id: S-PERF-GATE-002
title: "Serialize adv_p02 test binary via nextest max-threads=1 group to eliminate oversubscription blowup"
# NOTE: The filename slug (adv-p02-shared-dtu-fixtures) reflects the original v1.0–v2.1
# LazyLock shared-fixtures approach, which was abandoned before delivery (see §Changelog
# v3.0 entry). Per POL-1 filename slugs are immutable; the slug is retained as-is.
wave: maintenance
epic_id: maintenance
priority: P3
status: ready
version: "3.0"
spec_version: "v3.0"
level: ops
producer: story-writer
timestamp: "2026-06-27"
modified: "2026-06-28"
input-hash: ""
inputs:
  - .factory/research/test-suite-performance-diagnosis-2026-06-26.md
  - .config/nextest.toml
traces_to: "test-suite-performance-diagnosis-2026-06-26"
cycle: "v1.0.0-greenfield"
phase: 3
tdd_mode: "n/a"
# tdd_mode: n/a — CONFIG-ONLY change. A nextest test-group cannot be driven by a
# failing unit test; no TDD Red Gate applies. The gate is structural: the overrides
# either exist in .config/nextest.toml or they do not.
track: "Platform Engineering"
subsystems: []
crates_touched: []
# crates_touched: [] — test source file (prism-bin/tests/adv_p02_e2e_pushdown_pipeline_test.rs)
# is UNCHANGED from develop (git diff develop...HEAD returns empty for that file).
# Only .config/nextest.toml is modified.
target_module: ".config/nextest.toml"
behavioral_contracts:
  - BC-5.39.001
# NOTE: This is a test-infrastructure maintenance story. It does NOT introduce new
# product behavioral contracts. BC-5.39.001 governs the story's own delivery
# (3-CLEAN convergence requirement). No product BCs are added or modified.
# BC status: BC-5.39.001 is active.
verification_properties: []
depends_on:
  - S-PERF-GATE-001
# S-PERF-GATE-001 is MERGED (PR #204). S-PERF-GATE-001 established the [profile.prepush]
# gate and the serial-subprocess test-group + [profile.prepush]/[profile.ci] profiles
# that this story extends with the adv-p02-serial group.
blocks: []
blocked_by: []
points: 2
# 2 pts: config-only change; no test-logic modification; no Red Gate; no TDD cycle.
# Estimated effort: ~30 min edit + 3-CLEAN cascade overhead.
estimated_days: 0.5
risk: LOW
acceptance_criteria_count: 5
red_gate_tests: 0
# red_gate_tests: 0 — CONFIG-ONLY maintenance change. No TDD Red Gate applicable.
# A nextest test-group serialization cannot be driven by a failing unit test.
estimated_passes: "1"
holdout_scenarios: []
assumption_validations: []
risk_mitigations: []
design_source: ".factory/research/test-suite-performance-diagnosis-2026-06-26.md Finding #1 (oversubscription as dominant cost amplifier)"
---

# S-PERF-GATE-002: Serialize adv_p02 test binary via nextest max-threads=1 group to eliminate oversubscription blowup

## Narrative

As a Prism developer, I want the `adv_p02_e2e_pushdown_pipeline_test` binary placed in
a nextest serial test-group (`adv-p02-serial`, `max-threads = 1`) so that its 8 tests
cannot run in parallel with each other, eliminating the oversubscription-driven
60–300 s wall-clock blowup that made `just check` convergence iterations painful.

## Scheduling Note

**DELIVERED — this story is complete.** `S-PERF-GATE-001` (PR #204) is merged; the
`[profile.prepush]` / `[profile.ci]` infrastructure it established is the foundation
for the `adv-p02-serial` group added here. The worktree HEAD is `dacae05e`.

## Background

The 8 tests in `crates/prism-bin/tests/adv_p02_e2e_pushdown_pipeline_test.rs` each boot
their own DTU clone (CrowdStrike or Armis) per-test. The in-process DTU boots are
individually cheap (~200 ms each). The problem is oversubscription: when `just check`
runs the full workspace in parallel, the adv_p02 binary's 7–8 heavy DTU-booting tests
spawn on top of 16-core saturation, causing non-deterministic `filter_strings` assertion
failures (ADV-P02-CRIT-001) and 60–300 s wall-clock blowup.

**Diagnosis source:** `.factory/research/test-suite-performance-diagnosis-2026-06-26.md`
Finding #1 identifies oversubscription as the dominant cost amplifier for this binary.
The fix is structural: cap this binary to serial execution via a nextest test-group.

**Why the original LazyLock shared-fixture approach (v1.0–v2.1) was abandoned:**

The v1.0–v2.1 design proposed `LazyLock<TestDtuHandle<CrowdstrikeClone>>` and
`LazyLock<TestDtuHandle<ArmisClone>>` statics to share one DTU boot across all tests in
the binary, reducing from 7 boots to 2. This premise is INVALID: nextest runs each test
in its own OS process (`--process-per-test` is the nextest model). A `LazyLock` static
re-initializes independently in every test's process — there is no cross-process static
sharing. Adding `LazyLock` statics would add complexity for zero boot-amortization
benefit.

**Actual shipped change (Finding F-SPG2-P5-001 HIGH, human-directed Option A):**

- `crates/prism-bin/tests/adv_p02_e2e_pushdown_pipeline_test.rs` — UNCHANGED from
  develop. `git diff develop...HEAD -- crates/prism-bin/tests/adv_p02_e2e_pushdown_pipeline_test.rs`
  is empty. Per-test clone construction, mid-test resets, and test logic are identical
  to develop.
- `.config/nextest.toml` — adds `adv-p02-serial = { max-threads = 1 }` to `[test-groups]`
  and two profile overrides (prepush + ci) that assign the
  `adv_p02_e2e_pushdown_pipeline_test` binary to this group.

**Net result:** The 8 tests run serially within the binary. Oversubscription-driven
blowup is eliminated. Per-test DTU boots are individually cheap and are not a problem
when not oversubscribed. The `filter_strings` assertions are deterministic under serial
execution (confirmed: 3/3 consecutive `just check` runs pass at HEAD `dacae05e`).

## Behavioral Contracts

This is a test-infrastructure maintenance story. It does not introduce new product
behavioral contracts. BC-5.39.001 (3-CLEAN convergence) governs the delivery process
for this story's own PR cascade.

| BC ID | Title | Relevant Clause |
|-------|-------|-----------------|
| BC-5.39.001 | 3-CLEAN Convergence Protocol | All adversarial cascades require 3 consecutive clean passes (CLEAN-strict: zero findings of any severity). |

**Anchor justification per POL-5:** BC-5.39.001 anchors this story as the governing
delivery-quality invariant. No product BCs apply to a test-infrastructure config change.

## Acceptance Criteria

### AC-001: `adv-p02-serial = { max-threads = 1 }` present in `[test-groups]` of `.config/nextest.toml` (traces to BC-5.39.001 §Delivery process — serial group structurally present)

`.config/nextest.toml` contains `adv-p02-serial = { max-threads = 1 }` in the
`[test-groups]` table, alongside the existing `serial-subprocess` entry.

Self-verifying check (source-verified against worktree HEAD `dacae05e`):
```bash
grep 'adv-p02-serial' .config/nextest.toml
```
Returns at least 3 lines: the `[test-groups]` entry and the two override entries.

The `serial-subprocess` entry must NOT be removed or modified. This is an additive change
to the existing table.

### AC-002: `[[profile.prepush.overrides]]` and `[[profile.ci.overrides]]` each contain an entry assigning `adv_p02_e2e_pushdown_pipeline_test` to the serial group (traces to BC-5.39.001 §Delivery process — oversubscription eliminated in both profiles)

Both profile override blocks are present in `.config/nextest.toml`:

```toml
[[profile.prepush.overrides]]
filter = 'binary(adv_p02_e2e_pushdown_pipeline_test)'
test-group = 'adv-p02-serial'

[[profile.ci.overrides]]
filter = 'binary(adv_p02_e2e_pushdown_pipeline_test)'
test-group = 'adv-p02-serial'
```

The filter is `binary(adv_p02_e2e_pushdown_pipeline_test)` (binary-name filter, same
pattern as `binary(signal_handlers)` from S-PERF-GATE-001). The override is scoped to
`prepush` and `ci` profiles only; it is NOT applied to `[profile.default.overrides]`.

Self-verifying check (source-verified against worktree HEAD `dacae05e`):
```bash
grep -A2 'adv_p02_e2e_pushdown_pipeline_test' .config/nextest.toml
```
Returns two blocks — one under `[[profile.prepush.overrides]]` and one under
`[[profile.ci.overrides]]`.

### AC-003: All 8 adv_p02 functional tests pass under `cargo nextest run -E 'binary(adv_p02_e2e_pushdown_pipeline_test)' --profile prepush` (traces to BC-5.39.001 §Delivery process — serial group does not break existing tests)

```bash
cargo nextest run -p prism-bin -E 'binary(adv_p02_e2e_pushdown_pipeline_test)' --profile prepush --no-fail-fast
```

Must exit 0 with all 8 tests passing. The 8 test names (source-verified against develop,
which is identical to HEAD for this file):

1. `test_adv_p02_e2e_crowdstrike_fql_from_where_predicate`
2. `test_adv_p02_e2e_crowdstrike_limit_from_pql_limit_clause`
3. `test_adv_p02_e2e_armis_aql_augmentation_from_where_predicate`
4. `test_adv_p02_sid1_armis_fetch_start_time_augments_aql`
5. `test_ac_cws_002_fql_time_window_both_start_and_end_via_materialization_pipeline`
6. `test_ac_equiv_001_result_equivalence_via_run_materialization_pipeline`
7. `test_adv_p08_med001_crowdstrike_inclusive_boundary_via_run_materialization_pipeline`
8. `test_adv_p08_med001_armis_inclusive_boundary_via_run_materialization_pipeline`

No test is removed, `#[ignore]`'d, or weakened. Test source is unchanged from develop.

### AC-004: The adv_p02 test source file is unchanged from develop — no test-logic modification (traces to BC-5.39.001 §Delivery process — CONFIG-ONLY scope; no behavioral change)

```bash
git diff develop...HEAD -- crates/prism-bin/tests/adv_p02_e2e_pushdown_pipeline_test.rs
```

Must produce empty output. Per-test clone construction, mid-test resets, and all
assertions are identical to develop. The serialization is structural (nextest config),
not behavioral (test source).

Self-verified at worktree HEAD `dacae05e`: diff is empty.

### AC-005: `just check` exits 0 with the overrides active (traces to BC-5.39.001 §Delivery process — full workspace gate passes)

```bash
just check
```

Must exit 0 with all 4974 tests passing (source-verified: 4974/4974 at HEAD `dacae05e`).
The `adv-p02-serial` group is active during this run. Record the wall-clock in the PR
description as a regression benchmark (not a hard gate — load conditions may vary).

## Red Gate Design

**No TDD Red Gate applicable.** This is a CONFIG-ONLY change. A nextest test-group
serialization cannot be driven by a failing unit test — the overrides either exist in
`.config/nextest.toml` or they do not. `red_gate_tests: 0` in frontmatter is correct.

The correctness gate is structural: AC-001 and AC-002 verify the config entries exist;
AC-003 verifies the 8 functional tests pass under the new profile; AC-004 verifies the
test source is unchanged; AC-005 verifies the full workspace gate passes.

## Token Budget Estimate

| Item | Estimated Tokens |
|------|-----------------|
| Story spec (this file) | ~4 000 |
| `.config/nextest.toml` (full file; verify adv-p02-serial group + overrides) | ~900 |
| BC-5.39.001 (1 BC file) | ~800 |
| S-PERF-GATE-001 story (Previous Story Intelligence; serial-subprocess pattern) | ~4 000 |
| `cargo nextest run` output (adv_p02 binary, 8 tests) | ~400 |
| **Total** | **~10 100** |

Context window headroom: ~10k tokens is ~3% of a 350k context window.
CONFIG-ONLY change; no test-file reading required.

## Tasks

**STATUS: DELIVERED.** The following tasks document what was done; they serve as a
post-delivery audit trail and as implementation guidance if the change needs to be
reproduced or cherry-picked.

1. **Verify test source is unchanged from develop:**
   ```bash
   git diff develop...HEAD -- crates/prism-bin/tests/adv_p02_e2e_pushdown_pipeline_test.rs
   ```
   Expected: empty output. (Verified at HEAD `dacae05e`.)

2. **Verify nextest config changes are present (AC-001, AC-002):**
   ```bash
   grep 'adv-p02-serial' .config/nextest.toml
   ```
   Expected: at least 3 hits (group entry, prepush override, ci override).

3. **Run all 8 adv_p02 tests under prepush profile (AC-003):**
   ```bash
   cargo nextest run -p prism-bin -E 'binary(adv_p02_e2e_pushdown_pipeline_test)' \
       --profile prepush --no-fail-fast
   ```
   Expected: 8/8 pass.

4. **Determinism check — 3 consecutive runs:**
   ```bash
   for i in 1 2 3; do
       cargo nextest run -p prism-bin -E 'binary(adv_p02_e2e_pushdown_pipeline_test)' \
           --profile prepush
   done
   ```
   All 3 runs must exit 0. (Verified 3/3 at HEAD `dacae05e`.)

5. **Full workspace gate (AC-005):**
   ```bash
   just check
   ```
   Must exit 0. (Verified 4974/4974 at HEAD `dacae05e`.)

## Previous Story Intelligence

**S-PERF-GATE-001 (MERGED PR #204, predecessor):**

1. **Established `[profile.prepush]` and `[profile.ci]` with retries + terminate-after.**
   The `adv-p02-serial` test-group follows the same `[[profile.prepush.overrides]]` /
   `[[profile.ci.overrides]]` pattern as the `serial-subprocess` group from
   S-PERF-GATE-001 AC-011. `serial-subprocess` is the structural precedent.

2. **`serial-subprocess = { max-threads = 1 }` is already in `[test-groups]`** (additive
   change only — do NOT remove or modify `serial-subprocess`).

3. **LazyLock shared-fixture approach (v1.0–v2.1) was abandoned (F-SPG2-P5-001 HIGH,
   human Option A):** nextest runs each test in its own OS process. A `LazyLock` static
   re-initializes per process — there is no cross-test sharing. The correct mechanism is
   the serial test-group alone. This story records the rationale so it is not re-proposed
   in future maintenance passes.

4. **AC self-verification discipline (lesson from S-PERF-GATE-001 cascade):** Every AC
   verification command in this story was source-verified against the actual worktree at
   HEAD `dacae05e` before being written. The prior cascade had recurring findings from ACs
   whose grep commands returned different counts than the story claimed. This story's ACs
   are self-consistent with the shipped diff.

## Architecture Compliance Rules

1. **No test-source modification.** This story modifies ONLY `.config/nextest.toml`. The
   test file `crates/prism-bin/tests/adv_p02_e2e_pushdown_pipeline_test.rs` is unchanged
   from develop (AC-004). No production source files (`src/`) are touched.

2. **No expansion of prism-bin's public API surface.** No `pub` declarations change.

3. **ADV-P02-CRIT-001 correctness must not regress.** The serial test-group preserves
   the `filter_strings` assertion semantics. Verified deterministic 3/3 consecutive runs
   at HEAD `dacae05e`.

4. **`just check` must exit 0 before the PR is opened.** Per CLAUDE.md §Building.
   Verified 4974/4974 at HEAD `dacae05e`.

5. **No AI attribution in commits** per project git conventions (CLAUDE.md).

## Library & Framework Requirements

No new external dependencies.

| Library | Usage in this story | Version / Source |
|---------|---------------------|-----------------|
| `cargo nextest` | Test runner with test-group support | existing workspace pin |

**Forbidden dependencies:** No new `Cargo.toml` entries.

## File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `crates/prism-bin/tests/adv_p02_e2e_pushdown_pipeline_test.rs` | UNCHANGED | Identical to develop. `git diff develop...HEAD` returns empty. |
| `.config/nextest.toml` | MODIFY | Add `adv-p02-serial = { max-threads = 1 }` to `[test-groups]`; add `[[profile.prepush.overrides]]` and `[[profile.ci.overrides]]` with `filter = 'binary(adv_p02_e2e_pushdown_pipeline_test)'` and `test-group = 'adv-p02-serial'`. |

**No new files.** No DTU clone source files are modified. No production `src/` files are
modified. No new `Cargo.toml` entries.

**Subsystem anchor:** `subsystems: []` is correct. This story modifies build
configuration, not product subsystems in the ARCH-INDEX Subsystem Registry.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `adv-p02-serial` test-group override interacts with `[profile.e2e]` or `[profile.e2e-multi-org]` | The override is applied to `prepush` and `ci` profiles only. Other profiles are unaffected. Verified: source-checked `.config/nextest.toml` at HEAD `dacae05e` — no `[profile.default.overrides]` or `[profile.e2e.overrides]` entries exist for this binary. |
| EC-002 | `just check` passes but adv_p02 tests fail when the profile is not specified | Tests run outside the `prepush` or `ci` profile do not get the serial-group override and may still encounter oversubscription if the default profile allows full parallelism. This is accepted behavior — the serial group is intentionally scoped to the two profiles used in pre-push and CI flows. |
| EC-003 | Nextest version in CI does not support `binary(...)` filter syntax | `binary(...)` filter syntax is supported since cargo-nextest 0.9.52 (2023-09-12). The workspace already uses nextest with this filter for `signal_handlers` (S-PERF-GATE-001, verified in `.config/nextest.toml`). No version bump required. |

## Architecture Mapping

| Component | Module | File | Pure/Effectful |
|-----------|--------|------|----------------|
| `adv-p02-serial` nextest test-group + overrides | build config | `.config/nextest.toml` | N/A — configuration |
| `adv_p02_e2e_pushdown_pipeline_test` (8 tests, unchanged) | prism-bin tests | `crates/prism-bin/tests/adv_p02_e2e_pushdown_pipeline_test.rs` | Effectful (in-process DTU clone servers + RocksDB) |

## §References

Per POL-7 (verbatim BC H1 titles):

- BC-5.39.001 — *3-CLEAN Convergence Protocol* (delivery gate for this story itself)
- `.factory/research/test-suite-performance-diagnosis-2026-06-26.md` — diagnostic source,
  Finding #1 (oversubscription as dominant cost amplifier)
- S-PERF-GATE-001 — predecessor story (MERGED PR #204); established `[profile.prepush]`,
  `serial-subprocess` test-group, nextest override pattern

## §Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| v3.0 | 2026-06-28 | story-writer | MAJOR REFRAME. Original LazyLock shared-fixtures approach (v1.0–v2.1) abandoned: nextest process-per-test model makes cross-test `LazyLock` sharing impossible (F-SPG2-P5-001 HIGH finding). Human chose Option A: strip LazyLock, keep serial group only, revert test file to develop-verbatim. Scope simplified to CONFIG-ONLY: `.config/nextest.toml` is the only changed file. Test source `adv_p02_e2e_pushdown_pipeline_test.rs` is unchanged from develop (verified `git diff develop...HEAD` empty at HEAD `dacae05e`). Title, Narrative, Background, ACs (9 → 5, all self-verifying), Red Gate (2 → 0), Tasks, Previous Story Intelligence, Architecture Mapping, File Structure Requirements, Library Requirements, Edge Cases, References, Token Budget rewritten to match shipped reality. Version: v2.1 → v3.0. acceptance_criteria_count: 9 → 5. red_gate_tests: 2 → 0. points: 5 → 2. tdd_mode: strict → n/a. crates_touched: [prism-bin] → []. |
| v2.1 | 2026-06-28 | story-writer | LOCAL Pass-1 adversary findings closed: AC-004 reset-count corrected from "exactly 6" to "exactly 7"; §Changelog reordered newest-first per POL-32; AC-001 volatile line-pin citations replaced with behavioral anchors (TD-VSDD-091). |
| v2.0 | 2026-06-28 | story-writer | Full elaboration from stub. Two-phase scope (reset refactor + LazyLock). 9 ACs, 2 Red Gate tests. All factual claims source-verified. |
| v1.0 | 2026-06-27 | story-writer | Draft stub registered as Canonical Principle Rule 3 deferral anchor for S-PERF-GATE-001 AC-006/007 (D-1368, 2026-06-26). Closes F-P2R11-MED-001 (phantom file resolving POL-22). |
