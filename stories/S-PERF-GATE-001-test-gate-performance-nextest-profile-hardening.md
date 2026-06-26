---
document_type: story
story_id: "S-PERF-GATE-001"
title: "Test gate performance — nextest profile hardening + build_http_client timeout fix + adv_p02 shared fixtures"
wave: maintenance
epic_id: maintenance
priority: P2
status: draft
version: "1.0"
spec_version: "v1.0"
level: ops
producer: story-writer
timestamp: "2026-06-26"
modified: "2026-06-26"
input-hash: ""
inputs:
  - .factory/research/test-suite-performance-diagnosis-2026-06-26.md
  - .config/nextest.toml
  - Justfile
  - .cargo/config.toml
  - crates/prism-bin/src/spec_driven_adapter.rs
  - crates/prism-bin/tests/bc_2_01_013_spec_driven_adapter.rs
  - crates/prism-bin/tests/adv_p02_e2e_pushdown_pipeline_test.rs
  - crates/prism-bin/tests/signal_handlers.rs
traces_to: "test-suite-performance-diagnosis-2026-06-26"
cycle: "v1.0.0-greenfield"
phase: 3
tdd_mode: strict
track: "Platform Engineering"
subsystems: []
crates_touched:
  - prism-bin
target_module: ".config/nextest.toml, Justfile, .cargo/config.toml, crates/prism-bin/src/spec_driven_adapter.rs"
behavioral_contracts:
  - BC-5.39.001
# NOTE: This is a test-infrastructure maintenance story. It does NOT introduce new
# product behavioral contracts. BC-5.39.001 governs the story's own delivery
# (3-CLEAN convergence requirement). No product BCs are added or modified.
# BC status: BC-5.39.001 is active. Story can advance to ready when PO confirms
# that no additional product BCs are required for this scope.
verification_properties: []
depends_on: []
blocks: []
points: 5
estimated_days: 1.0
risk: LOW
acceptance_criteria_count: 11
red_gate_tests: 1
estimated_passes: "2"
holdout_scenarios: []
assumption_validations: []
risk_mitigations: []
design_source: ".factory/research/test-suite-performance-diagnosis-2026-06-26.md §10"
---

# S-PERF-GATE-001: Test gate performance — nextest profile hardening + build_http_client timeout fix + adv_p02 shared fixtures

## Narrative

As a Prism developer, I want the pre-push gate (`just check`) to complete in ≤600
seconds on a warm cache with no concurrent cargo jobs, so that the gate is not a
productivity bottleneck and flaky tests do not force full re-runs.

## Scheduling Note

**IMMEDIATE — no sequencing constraint.** This story has no `depends_on` and can be
scheduled in the next available maintenance wave. It does not block or conflict with
any in-flight feature work.

## Background

The pre-push gate (`just check`) has a **measured isolated wall-clock of 421–864
seconds** on clean idle runs on the developer's 16-core aarch64 machine, degrading
to **1076–3952 seconds** under concurrency oversubscription. The diagnosis document
at `.factory/research/test-suite-performance-diagnosis-2026-06-26.md` identifies
five actionable optimizations ranked by win × fidelity-safety / effort.

This story implements items #1, #2, #4, #5, and #6 from the diagnosis. Item #3
(RUSTFLAGS alignment for doctest compile-pass deduplication) is deferred — the
macOS RUSTFLAGS situation is already correct (both steps use `RUSTFLAGS=""`) and
the deduplication benefit is 0 min warm on macOS.

**Key data points:**
- `test_BC_2_01_013_build_http_client_with_timeout_succeeds` was observed hanging
  for **324.913 seconds** under load due to reqwest networking-subsystem init under
  memory pressure on macOS.
- `adv_p02_e2e_pushdown_pipeline_test.rs` has 8 tests, each booting its own DTU clone
  (Axum + Tokio startup per test). Total per-test boot cost: 5–15s idle, 60–300s under
  oversubscription.
- SIGTERM flake (`test_BC_2_10_010_sigterm_causes_graceful_exit_zero`) hits SIGSEGV
  (signal=11) in ~3/30 runs due to RocksDB mmap region invalidation under memory pressure
  when multiple prism-bin subprocess tests run in parallel.
- `test_BC_2_06_019_armis_primary_device_stage_visibility` is a **designed-to-fail Red
  Gate** (StageMask projection not yet implemented) — it is NOT a flake and is explicitly
  excluded from this story's scope.

## Behavioral Contracts

**This is a test-infrastructure maintenance story. It does not introduce new product
behavioral contracts.** BC-5.39.001 (3-CLEAN convergence) governs the delivery process
for this story itself — the story's own PR cascade must achieve three consecutive
CLEAN(strict) passes before merge.

| BC ID | Title | Relevant Clause |
|-------|-------|-----------------|
| BC-5.39.001 | 3-CLEAN Convergence Protocol | All adversarial cascades require 3 consecutive clean passes (CLEAN-strict: zero findings of any severity). The 3-CLEAN gate for this story must use `PROPTEST_CASES=100` to match `just check` strength. |

**Anchor justification per POL-5:** BC-5.39.001 anchors AC-009 (gate completion time)
as the governing delivery-quality invariant for the story itself. It does not anchor
any individual code-change AC — those are test-infrastructure correctness obligations
that have no product BC equivalent.

## Acceptance Criteria

### AC-001: `[profile.prepush]` added to `.config/nextest.toml` (traces to BC-5.39.001 §Delivery process — gate must not hang indefinitely)

`.config/nextest.toml` gains a new `[profile.prepush]` section:

```toml
[profile.prepush]
fail-fast = false
slow-timeout = { period = "90s", terminate-after = 2 }
retries = 1
final-status-level = "slow"
failure-output = "immediate"
```

- `retries = 1`: mirrors the existing `[profile.e2e]` policy; one retry absorbs
  transient SIGSEGV-on-shutdown flakes without masking real failures.
- `terminate-after = 2`: any test that exceeds 90 s twice across retries is a
  genuine hang (RocksDB deadlock, tokio runtime stuck). Killing it is the correct
  signal. The `proptest_BC_3_2_002_vp_01_cross_org_isolation` suite takes ~75 s at
  1000 cases (documented in the existing `[profile.ci]` comment lines 38–53 in
  `.config/nextest.toml`); `terminate-after = 2` gives 180 s total, well above that.
- A comment above the section documents the rationale, referencing PR #127 precedent
  and this story ID.

Running `rg 'profile.prepush' .config/nextest.toml` returns exactly one hit.

### AC-002: `[profile.ci]` in `.config/nextest.toml` updated — `retries = 1` and `terminate-after = 2` added (traces to BC-5.39.001 §Delivery process — CI gate must not hang on flake)

The existing `[profile.ci]` section gains:
- `retries = 1` (new line)
- `slow-timeout` amended from `{ period = "60s" }` to `{ period = "60s", terminate-after = 2 }`

The existing comment explaining why `terminate-after` was previously omitted
(proptest taking ~75 s) is updated to explain why `terminate-after = 2` is now safe
(total timeout = 120 s across 2 attempts, above the 75 s proptest maximum).

Running `grep -A5 'profile.ci\]' .config/nextest.toml | grep 'retries'` returns
exactly one hit showing `retries = 1`.

### AC-003: Justfile `check` recipe updated to use `--profile prepush` (traces to BC-5.39.001 §Delivery process — local gate matches CI retry policy)

The `Justfile` `check` recipe's `cargo nextest run` invocation gains `--profile prepush`:

```
PROPTEST_CASES=100 cargo nextest run --workspace --all-features --no-fail-fast --profile prepush
```

Running `grep 'profile prepush' Justfile` returns exactly one hit in the `check`
recipe.

The `check-fast` recipe is NOT modified (it does not run nextest).

### AC-004: `build_http_client_with_custom_timeout(Duration)` extracted in `crates/prism-bin/src/spec_driven_adapter.rs`; production `build_http_client_with_timeout()` signature and behavior unchanged (traces to BC-5.39.001 §Delivery process — test must not block gate for >5 s)

`crates/prism-bin/src/spec_driven_adapter.rs` gains an inner `pub(crate)` function:

```rust
pub(crate) fn build_http_client_with_custom_timeout(
    timeout: Duration,
) -> Result<reqwest::Client, reqwest::Error> {
    reqwest::Client::builder().timeout(timeout).build()
}
```

The existing public `build_http_client_with_timeout()` is unchanged in signature and
in behavior — it delegates to `build_http_client_with_custom_timeout(Duration::from_secs(30))`:

```rust
pub fn build_http_client_with_timeout() -> Result<reqwest::Client, reqwest::Error> {
    build_http_client_with_custom_timeout(Duration::from_secs(30))
}
```

Running `grep 'build_http_client_with_custom_timeout' crates/prism-bin/src/spec_driven_adapter.rs`
returns at least 2 hits (definition + delegation call).

No other call sites for `build_http_client_with_timeout()` are modified — production
behavior is unchanged.

### AC-005: `test_BC_2_01_013_build_http_client_with_timeout_succeeds` passes in ≤5 s on a loaded machine (traces to BC-5.39.001 §Delivery process — single-test gate contribution bounded)

The test in `crates/prism-bin/tests/bc_2_01_013_spec_driven_adapter.rs` is updated
to call `build_http_client_with_custom_timeout(Duration::from_millis(1))` instead of
`build_http_client_with_timeout()`.

Rationale: the test verifies that `reqwest::ClientBuilder::build()` returns `Ok(_)`
— that the builder chain is correctly configured. Passing `1ms` avoids exercising the
networking subsystem's 30-second initialization path under load. The production 30-second
timeout is still exercised by the adv_p02 integration tests which run real pipeline
queries end-to-end.

Verified by:

```bash
cargo nextest run -p prism-bin -E 'test(build_http_client_with_timeout_succeeds)' --no-fail-fast
```

Must exit 0 in < 5 s (measured wall-clock, not bounded by the test framework — the
test should complete in < 1 s under normal conditions; the 5 s budget accounts for
build + process spawn overhead).

### AC-006: `adv_p02_e2e_pushdown_pipeline_test.rs` introduces `LazyLock` shared DTU handles for CrowdStrike and Armis clones; per-test DTU reset called at start of each test (traces to BC-5.39.001 §Delivery process — repeated DTU boot cost eliminated)

> **STATUS: DEFERRED-to-follow-up → S-PERF-GATE-002 (D-1368 2026-06-26)**
>
> **Safety gate fired:** The implementer verified that both `prism-dtu-crowdstrike` and
> `prism-dtu-armis` expose `POST /dtu/reset`. However, 4 of the 8 adv_p02 tests call
> `clone.reset()` INTERNALLY mid-test — specifically between the "no-filter" and
> "with-filter" pipeline runs within the same test body. Sharing a `LazyLock` clone
> across parallel tests would allow one test's internal mid-test reset to destroy
> another concurrent test's accumulated wire-log state, breaking the `filter_strings`
> assertions that verify filter push-down (ADV-P02-CRIT-001). Per-test DTU boot
> remains the correct isolation boundary until the internal-reset pattern is refactored.
>
> **Concrete future dependency (Canonical Principle Rule 3):** All 8 adv_p02 tests must
> be refactored to move internal `clone.reset()` calls from mid-test to before-test-start
> position. This is the prerequisite work anchored to S-PERF-GATE-002.
>
> **Deferral is NOT a speed shortcut.** Per-test DTU boot is the production-grade
> isolation boundary given the current test structure. The deferral is because changing
> that boundary without the prerequisite refactor would introduce silent test-correctness
> failures (ADV-P02-CRIT-001). This is a concrete future dependency, not a "we can fix
> later" rationalization.

`crates/prism-bin/tests/adv_p02_e2e_pushdown_pipeline_test.rs` is refactored to
introduce `LazyLock<Arc<TestDtuHandle>>` (or equivalent) shared fixtures for the
CrowdStrike and Armis DTU clones, following the PR #127 pattern documented in
`.config/nextest.toml` comment lines 41–52.

Each DTU clone is started once per test binary. Each test calls the DTU's reset
endpoint at its start to clear filter/request log state, preserving per-test isolation.

Running `grep 'LazyLock' crates/prism-bin/tests/adv_p02_e2e_pushdown_pipeline_test.rs`
returns at least one hit.

**Pre-implementation obligation:** The implementer MUST verify that `POST /reset`
(or the equivalent DTU reset route) is available in both `prism-dtu-crowdstrike` and
`prism-dtu-armis` before writing the shared fixture. If the reset endpoint is absent,
the implementer MUST add it (per SID-1: no deferral without a specific story ID and
test name citation). Check via:

```bash
grep -r 'reset' crates/prism-dtu-crowdstrike/src/routes/
grep -r 'reset' crates/prism-dtu-armis/src/routes/
```

### AC-007: All tests in `adv_p02_e2e_pushdown_pipeline_test.rs` continue to pass with shared fixtures (traces to BC-5.39.001 §Delivery process — refactor must not alter test coverage)

> **STATUS: DEFERRED-to-follow-up → S-PERF-GATE-002 (D-1368 2026-06-26)**
> See AC-006 deferral rationale above. AC-007 is the correctness gate for the
> LazyLock-shared fixture work; it cannot be satisfied until AC-006's prerequisite
> internal-reset refactor is complete.

After introducing shared DTU fixtures:

```bash
cargo nextest run -p prism-bin --test adv_p02_e2e_pushdown_pipeline_test --no-fail-fast
```

Must exit 0 with all 8 tests passing. No test is removed, `#[ignore]`'d, or weakened.

Test isolation must be preserved: each test must query and assert independently,
verifying its own predicate against the DTU's request/filter log after the per-test
reset.

### AC-008: `.cargo/config.toml` updated with documented sccache opt-in stanza (traces to BC-5.39.001 §Delivery process — cross-worktree cold-build path documented)

`.cargo/config.toml` gains a commented-out `sccache` opt-in stanza:

```toml
# -----------------------------------------------------------------------------
# sccache: optional rustc-wrapper for cross-invocation and cross-worktree caching.
#   Enable with: RUSTC_WRAPPER=sccache or by uncommenting the line below.
#   Requires: cargo install sccache
#   Do NOT enable unconditionally — sccache must be installed first.
#   Docs: https://github.com/mozilla/sccache
#   Estimated speedup: 40-70% on cold builds after prism-query changes.
#   Note: CI uses Swatinem/rust-cache (not sccache); this stanza is local-dev only.
# -----------------------------------------------------------------------------
# [build]
# rustc-wrapper = "sccache"   # uncomment after `cargo install sccache`
```

The stanza is commented out. CI is unaffected. Running `grep 'sccache' .cargo/config.toml`
returns at least 2 hits (comment + the commented-out line).

### AC-009: `just check` completes in ≤600 s on a warm cache with no concurrent cargo jobs (traces to BC-5.39.001 §Delivery process — gate is not a productivity bottleneck)

After all code changes:

```bash
time just check
```

Must complete in ≤600 seconds on the developer machine under idle conditions (single
concurrent cargo job, warm incremental cache). The historical warm-cache baseline is
421–864 s; items #1–#4 project a warm-cache reduction to 280–480 s per diagnosis §9.

This AC is measured, not formally gated — it serves as a regression benchmark. If
the gate exceeds 600 s on a warm idle machine after this story ships, that is a P2
finding for the next maintenance sweep.

### AC-010: SIGTERM flake root cause documented in `signal_handlers.rs` test comments; `retries = 1` provides the safety net (traces to BC-5.39.001 §Delivery process — flake root cause must be recorded before structural fix is deferred)

`crates/prism-bin/tests/signal_handlers.rs` gains a code comment directly above
`test_BC_2_10_010_sigterm_causes_graceful_exit_zero` documenting:

1. The observed failure mode: SIGSEGV (signal=11) on the prism child process.
2. Root cause hypothesis: RocksDB mmap region invalidated during forced teardown
   under memory pressure when multiple prism-bin subprocess tests run in parallel.
3. The safety net: `retries = 1` in `[profile.prepush]` and `[profile.ci]` absorbs
   the ~1-in-10 flake rate.
4. The structural fix path: serializing subprocess tests via nextest test-group
   `--test-threads 1` (see AC-011 below), which eliminates the parallel RocksDB
   init collision.

Running `grep -A5 'mmap\|SIGSEGV\|RocksDB.*signal' crates/prism-bin/tests/signal_handlers.rs`
returns hits in the comment block above `test_BC_2_10_010_sigterm_causes_graceful_exit_zero`.

### AC-011: `signal_handlers` subprocess tests serialized via nextest `[test-group]` `--test-threads 1` (traces to BC-5.39.001 §Delivery process — structural SIGTERM flake fix implemented)

`.config/nextest.toml` gains a `[[test-groups]]` entry that serializes the
subprocess-spawning tests in `signal_handlers.rs` to `--test-threads 1`:

```toml
[[test-groups]]
name = "signal-handlers-serial"
max-threads = 1

[[profile.default.overrides]]
filter = "binary(signal_handlers)"
test-group = "signal-handlers-serial"
```

This prevents parallel RocksDB init collisions between concurrent `prism start`
subprocess invocations. The override must apply to all profiles (via
`[profile.default.overrides]` or equivalent nextest `[[overrides]]` syntax).

After this change:

```bash
cargo nextest run -p prism-bin --test signal_handlers --no-fail-fast
```

Must exit 0 with all signal-handler tests passing in all 10 consecutive runs
on the developer machine (practical verification; the flake rate drops from ~10%
to ~0%).

## Exclusions

The following items are explicitly OUT OF SCOPE for this story:

1. **StageMask projection in `prism-dtu-armis/src/routes/devices.rs`** — The failing
   test `test_BC_2_06_019_armis_primary_device_stage_visibility` is a **designed-to-fail
   Red Gate** for BC-2.06.019 AC-007 (StageMask projection). It will remain failing after
   this story ships. It is a P1 implementation blocker for the separate story implementing
   BC-2.06.019 PC-4. Do NOT add it to this story, `#[ignore]` it, or quarantine it. It
   must stay failing until the StageMask projection feature is implemented.

2. **sccache CI integration** — CI already uses `Swatinem/rust-cache` for cross-job
   caching. Adding sccache to `.github/workflows/ci.yml` is a separate ci.yml story.
   AC-008 only adds the local-dev opt-in comment stanza.

3. **RUSTFLAGS alignment for doctest compile-pass deduplication** (diagnosis item #3) —
   On macOS aarch64 both the nextest step and the doctest step already use `RUSTFLAGS=""`
   (empty). No mismatch exists. This item provides zero warm-cache benefit on the primary
   developer platform and is deferred.

4. **Linker re-evaluation** (`just timings`) — The `.cargo/config.toml` explicitly defers
   this to `cargo build --timings` showing linking > 15% of total time. This measurement
   was not taken as part of this story's scope. It remains deferred per the existing
   comment in `.cargo/config.toml`.

5. **adv_p02 Red Gate test `test_BC_2_06_019_armis_primary_device_stage_visibility`** —
   Confirmed the same as exclusion #1 above (it appears in the adv_p02 scope context but
   the exclusion applies regardless of which test binary surfaces it).

## Red Gate Test

### RG-PERF-001: `build_http_client_with_custom_timeout` exists and accepts `Duration`

**File:** `crates/prism-bin/tests/bc_2_01_013_spec_driven_adapter.rs`

**Test name:** `test_BC_2_01_013_build_http_client_with_custom_timeout_accepts_duration`

**Behavior:** Call `build_http_client_with_custom_timeout(Duration::from_millis(1))`
and assert the result is `Ok(_)`. This test fails before AC-004 is implemented (the
function does not yet exist). It passes after the extraction.

**Fails before:** `build_http_client_with_custom_timeout` is not defined.
**Passes after:** Function is extracted per AC-004.

This is the sole formal Red Gate for this story. The other ACs are configuration and
refactoring changes where the existing test suite (all 8 adv_p02 tests must still
pass) acts as the correctness gate.

## Tasks

### Implementer tasks (develop-based worktree)

1. **Write RG-PERF-001 Red Gate test** in `bc_2_01_013_spec_driven_adapter.rs`. Verify it
   fails with "unresolved import" or "cannot find function" before proceeding.

2. **Extract `build_http_client_with_custom_timeout`** in `spec_driven_adapter.rs` per AC-004.
   Verify RG-PERF-001 now passes:
   ```bash
   cargo nextest run -p prism-bin -E 'test(build_http_client_with_custom_timeout)' --no-fail-fast
   ```

3. **Update `test_BC_2_01_013_build_http_client_with_timeout_succeeds`** to call
   `build_http_client_with_custom_timeout(Duration::from_millis(1))` per AC-005. Verify
   it passes in < 5 s:
   ```bash
   cargo nextest run -p prism-bin -E 'test(build_http_client_with_timeout_succeeds)' --no-fail-fast
   ```

4. **Pre-implementation check for DTU reset endpoints** per AC-006:
   ```bash
   grep -rn 'reset' crates/prism-dtu-crowdstrike/src/routes/
   grep -rn 'reset' crates/prism-dtu-armis/src/routes/
   ```
   If the reset route is absent, add it to the DTU clone in-scope before writing the
   shared fixtures (SID-1: no deferral without specific story ID + test name).

5. **Introduce `LazyLock` shared DTU fixtures in `adv_p02_e2e_pushdown_pipeline_test.rs`**
   per AC-006 and AC-007. Follow the PR #127 pattern in `.config/nextest.toml` lines 41–52.
   Verify all 8 adv_p02 tests still pass:
   ```bash
   cargo nextest run -p prism-bin --test adv_p02_e2e_pushdown_pipeline_test --no-fail-fast
   ```

6. **Update `.config/nextest.toml`** per AC-001, AC-002, and AC-011:
   - Add `[profile.prepush]` with `retries = 1`, `slow-timeout = { period = "90s", terminate-after = 2 }`.
   - Amend `[profile.ci]` to add `retries = 1` and `terminate-after = 2` to `slow-timeout`.
   - Update the `[profile.ci]` comment to document why `terminate-after = 2` is now safe.
   - Add `[[test-groups]]` and `[[profile.default.overrides]]` for signal_handlers serialization.

7. **Update `Justfile` check recipe** per AC-003 to add `--profile prepush`.

8. **Update `.cargo/config.toml`** per AC-008 with the commented-out sccache stanza.

9. **Add SIGTERM root-cause comment** in `signal_handlers.rs` per AC-010.

10. **Run per-crate gate:**
    ```bash
    just iter prism-bin
    ```
    Must exit 0.

11. **Run full pre-push gate:**
    ```bash
    time just check
    ```
    Must exit 0. Record wall-clock for AC-009 benchmark.

12. **Commit** with message citing `S-PERF-GATE-001`. No AI attribution per project
    git conventions.

## Previous Story Intelligence

- **PR #127 (2026-05-05, S-DEMO-002 era):** Established the `LazyLock<TempDir>` +
  `LazyLock<Runtime>` shared-fixture pattern for prism-credentials proptest, achieving
  an 18x speedup (342 s → 19 s at 256 cases). The `.config/nextest.toml` comment at
  lines 41–52 is the canonical documentation of this pattern — read it before writing
  the adv_p02 shared fixtures.

- **S-MAINT-EAUTH-COLLISION-001 (2026-06-16):** Precedent for maintenance story format,
  Red Gate discipline, and CLAUDE.md §Canonical Principle compliance in a maintenance
  context. Follow the same structure here.

- **`.config/nextest.toml` `[profile.e2e]` pattern:** The `retries = 1` +
  `slow-timeout = { period = "120s", terminate-after = 1 }` pattern in `[profile.e2e]`
  is the pre-existing template for `[profile.prepush]`. The proposed prepush differs
  only in the period (90 s vs 120 s) and terminate-after (2 vs 1) to account for the
  more aggressive proptest budget.

- **Concurrency oversubscription root cause (diagnosis §2):** The dominant cause of
  gate slowness (421–3952 s range) is multiple parallel Agent dispatches sharing the
  Cargo target directory. This story's changes address the isolated-run budget (flake
  absorption, per-test boot cost, construction-test hang) — the oversubscription itself
  was already resolved by the serialize-heavy-gates discipline adopted after the
  diagnosis.

## Architecture Compliance Rules

(Derived from CLAUDE.md §Conventions, `just check` gate discipline, and the performance
diagnosis.)

1. **No `#[ignore]` without a specific story ID and test name citation (SID-1).** If the
   adv_p02 DTU reset endpoint is absent, add it in-scope rather than `#[ignore]`-ing the
   shared-fixture tests.

2. **Production `build_http_client_with_timeout()` signature and 30 s timeout are
   immutable.** The extracted `build_http_client_with_custom_timeout` is `pub(crate)` —
   it must not be promoted to `pub` without a deliberate API-surface decision.

3. **`reqwest::Client::new()` without `.timeout()` is forbidden in production code**
   (CLAUDE.md §Forbidden patterns). The refactor must preserve the 30-second timeout
   on the production call path.

4. **nextest `[profile.ci]` changes must not break CI.** `terminate-after = 2` at 60 s
   period gives 120 s total — safely above the documented 75 s proptest maximum.

5. **TD-VSDD-060 sibling-site sweep applies.** After changing `build_http_client_with_timeout`
   to delegate to `build_http_client_with_custom_timeout`, run:
   ```bash
   rg 'build_http_client_with_timeout' crates/ --type rust
   ```
   Confirm all existing callers still work (they call the unchanged public function).

6. **`.factory/` is not modified by this story.** The only `.factory/` artifact
   referenced by this story is the diagnosis research document (read-only input). No
   spec, BC, or error-taxonomy changes are required. State-manager handles the story
   file commit and STORY-INDEX registration.

7. **No AI attribution in commits** per project git conventions (CLAUDE.md).

8. **`just check` must exit 0 before the PR is opened.**

## Library & Framework Requirements

No new dependencies. All changes are to existing configuration files and existing
Rust source files.

| Tool | Purpose | Version |
|------|---------|---------|
| `cargo nextest` | Pre-push and CI gate runner | existing workspace pin |
| `reqwest` | HTTP client (existing dep; `build_http_client_with_custom_timeout` uses same builder) | existing workspace pin |
| `std::sync::LazyLock` | Shared DTU fixture initialization (stable since Rust 1.80; stable on the project's pinned toolchain) | stdlib |
| `just check` | Final pre-push gate | workspace Justfile |
| sccache (opt-in only) | Local cross-invocation compiler cache (AC-008 comment stanza only; not installed as a hard dep) | system, user-installed |

## File Structure Requirements (§FSR)

| File | Action | Notes |
|------|--------|-------|
| `.config/nextest.toml` | Modify | Add `[profile.prepush]`; amend `[profile.ci]`; add signal_handlers `[[test-groups]]` + `[[overrides]]` (AC-001, AC-002, AC-011) |
| `Justfile` | Modify | Add `--profile prepush` to `check` recipe's `cargo nextest run` invocation (AC-003) |
| `.cargo/config.toml` | Modify | Add commented-out sccache opt-in stanza (AC-008) |
| `crates/prism-bin/src/spec_driven_adapter.rs` | Modify | Extract `pub(crate) build_http_client_with_custom_timeout(Duration)` + delegate from `build_http_client_with_timeout()` (AC-004) |
| `crates/prism-bin/tests/bc_2_01_013_spec_driven_adapter.rs` | Modify | Add RG-PERF-001 Red Gate; update `test_BC_2_01_013_build_http_client_with_timeout_succeeds` to use 1ms variant (AC-005) |
| `crates/prism-bin/tests/adv_p02_e2e_pushdown_pipeline_test.rs` | Modify | Introduce `LazyLock` shared DTU handles for CrowdStrike and Armis (AC-006, AC-007) |
| `crates/prism-bin/tests/signal_handlers.rs` | Modify | Add SIGTERM flake root-cause comment above `test_BC_2_10_010_sigterm_causes_graceful_exit_zero` (AC-010) |

**Crate/location scope:** All code changes are in `prism-bin`. Config changes are in
project-root config files. No changes to `prism-core`, `prism-query`, `prism-spec-engine`,
`prism-mcp`, or any DTU clone crates (unless the pre-implementation check in AC-006
discovers a missing reset endpoint — in that case the relevant DTU crate is added
in-scope).

**Subsystem anchor:** This story does not trace to a subsystem in the ARCH-INDEX
Subsystem Registry because it modifies test infrastructure and build configuration,
not product subsystems. `subsystems: []` is correct.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | DTU reset endpoint is absent from `prism-dtu-crowdstrike` or `prism-dtu-armis` at implementation time | Add the reset endpoint to the affected DTU clone in-scope per SID-1. Do NOT `#[ignore]` the shared-fixture tests. |
| EC-002 | `test_BC_2_06_019_armis_primary_device_stage_visibility` appears in the `adv_p02` test binary and causes the binary to fail | This is a Red Gate test — leave it failing. Do not suppress, skip, or `#[ignore]` it. It is out-of-scope (see Exclusions §1). |
| EC-003 | `retries = 1` in `[profile.ci]` causes CI to mask a real intermittent failure (false stability) | If a test fails twice (both the initial run and the retry), nextest reports FAIL — masking does not occur. A retry only converts a 1-of-1 flake to a pass, not a 2-of-2 failure. This is the correct behavior per the existing `[profile.e2e]` policy. |
| EC-004 | `terminate-after = 2` kills the proptest suite at ~80 s (just above the 75 s documented max) on a slow CI runner | If proptest takes > 90 s on a slow runner, the first "slow" marker fires at 90 s; the second fires at 180 s (second pass after retry); at 180 s the test is terminated. At 1000 cases, proptest typically finishes in 75 s. If slow CI runners regularly hit 90 s+, raise `period` to `120s` as a follow-on tweak. Document in the `.config/nextest.toml` comment. |
| EC-005 | The `just check` time measurement (AC-009) exceeds 600 s even after all changes, on a warm idle machine | Record the measurement and open a new P2 performance finding for the next maintenance sweep. Do NOT block this story's merge — AC-009 is a regression benchmark, not a hard gate. |
| EC-006 | `adv_p02` shared fixture startup fails (Axum/Tokio port bind error) due to port exhaustion in `LazyLock::new` | Use `127.0.0.1:0` (OS-assigned ephemeral port) for DTU bind address, same as the existing per-test pattern. The `LazyLock` initialization is infallible at the Rust type level — use `expect("adv_p02 DTU startup failed")` with a descriptive message; this is acceptable in test code per CLAUDE.md test exemptions. |
| EC-007 | The `[[test-groups]] signal-handlers-serial` override causes an unexpected interaction with the `[profile.e2e]` or `[profile.e2e-multi-org]` profiles | nextest test-group overrides apply globally by default. Verify that the signal_handlers serialization does not interfere with `--profile e2e` runs (those tests are `#[ignore]`'d in the e2e profile; the serialization constraint is harmless). |

## Architecture Mapping

| Component | Module | File | Pure/Effectful |
|-----------|--------|------|----------------|
| `build_http_client_with_custom_timeout` (new) | prism-bin | `crates/prism-bin/src/spec_driven_adapter.rs` | Pure (no I/O, no network in the builder path) |
| `build_http_client_with_timeout` (delegate) | prism-bin | `crates/prism-bin/src/spec_driven_adapter.rs` | Pure (constructor; no network call in builder) |
| `LazyLock<Arc<TestDtuHandle>>` shared fixtures | prism-bin tests | `crates/prism-bin/tests/adv_p02_e2e_pushdown_pipeline_test.rs` | Effectful (Axum HTTP server + Tokio runtime) |
| nextest profiles (`prepush`, `ci` amendment) | build config | `.config/nextest.toml` | N/A — configuration |
| sccache opt-in stanza (commented-out) | build config | `.cargo/config.toml` | N/A — configuration |
| signal_handlers test comment + serialization | prism-bin tests | `crates/prism-bin/tests/signal_handlers.rs` + `.config/nextest.toml` | Effectful (subprocess spawning) |
| Justfile check recipe | build config | `Justfile` | N/A — configuration |

## Token Budget Estimate

| Item | Estimated Tokens |
|------|-----------------|
| Story spec (this file) | ~8 000 |
| `.config/nextest.toml` (full file, current state) | ~900 |
| `Justfile` check recipe (relevant section) | ~500 |
| `.cargo/config.toml` (relevant section) | ~400 |
| `crates/prism-bin/src/spec_driven_adapter.rs` (build_http_client function + surrounding context) | ~1 500 |
| `crates/prism-bin/tests/bc_2_01_013_spec_driven_adapter.rs` (build_http_client test + file header) | ~2 500 |
| `crates/prism-bin/tests/adv_p02_e2e_pushdown_pipeline_test.rs` (full file — LazyLock refactor requires reading all 8 tests) | ~4 000 |
| `crates/prism-bin/tests/signal_handlers.rs` (SIGTERM test + surrounding context) | ~1 500 |
| BC-5.39.001 (1 BC file) | ~800 |
| `just check` output | ~500 |
| `cargo nextest run` per-test output | ~400 |
| **Total** | **~21 000** |

Context window headroom: ~21k tokens is ~6% of a 350k context window.
No story splitting required. Single implementer dispatch covers all ACs.

## §References

Per POL-7 (verbatim BC H1 titles):

- BC-5.39.001 — *3-CLEAN Convergence Protocol* (delivery gate for this story itself)
- `.factory/research/test-suite-performance-diagnosis-2026-06-26.md` — diagnostic
  source: 237 push-log files, bimodal timing data, per-test root cause analysis
- PR #127 — LazyLock shared-fixture precedent (prism-credentials proptest 18x speedup)
- `.config/nextest.toml` lines 41–52 — LazyLock pattern commentary (canonical reference)

## §Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.0 | 2026-06-26 | story-writer | Initial materialization from performance diagnosis §10 skeleton + §6/§7 detail |
