---
document_type: story
story_id: S-PERF-GATE-002
title: "adv_p02 shared DTU fixtures — extract internal-reset pattern, then LazyLock-share CrowdStrike/Armis clones"
wave: maintenance
epic_id: maintenance
priority: P3
status: ready
version: "2.1"
spec_version: "v2.1"
level: ops
producer: story-writer
timestamp: "2026-06-27"
modified: "2026-06-28"
input-hash: ""
inputs:
  - .factory/research/test-suite-performance-diagnosis-2026-06-26.md
  - crates/prism-bin/tests/adv_p02_e2e_pushdown_pipeline_test.rs
  - crates/prism-dtu-crowdstrike/src/clone.rs
  - crates/prism-dtu-armis/src/clone.rs
  - crates/prism-dtu-common/src/clone.rs
  - .config/nextest.toml
traces_to: "test-suite-performance-diagnosis-2026-06-26"
cycle: "v1.0.0-greenfield"
phase: 3
tdd_mode: strict
track: "Platform Engineering"
subsystems: []
crates_touched:
  - prism-bin
  # .config/nextest.toml is project config, not a crate; noted separately in FSR.
target_module: "crates/prism-bin/tests/adv_p02_e2e_pushdown_pipeline_test.rs"
behavioral_contracts:
  - BC-5.39.001
# NOTE: This is a test-infrastructure maintenance story. It does NOT introduce new
# product behavioral contracts. BC-5.39.001 governs the story's own delivery
# (3-CLEAN convergence requirement). No product BCs are added or modified.
# BC status: BC-5.39.001 is active.
verification_properties: []
depends_on:
  - S-PERF-GATE-001
# S-PERF-GATE-001 is MERGED (PR #204). SOFT dependency: S-PERF-GATE-001 established
# the [profile.prepush] gate this story's tests will run under, and its nextest
# infrastructure (serial-subprocess test-group, [profile.prepush] + [profile.ci]
# profiles) is the foundation for the new adv-p02-serial test-group added here.
blocks: []
blocked_by: []
# UNBLOCKED: the internal-reset refactor is Phase 1 of THIS story's scope.
# The "internal-reset-refactor-prerequisite" listed in v1.0 blocked_by is resolved
# in-scope: the refactor (move 6 mid-test reset calls to test-start) and the
# LazyLock introduction (Phase 2) are both delivered by this story's implementer.
points: 5
# 5 pts: test-only file + config-only change. Phase 1 (reset refactor) = ~2h;
# Phase 2 (LazyLock introduction) = ~2h; Red Gate + 3-CLEAN cascade = ~1h overhead.
# Comparable to S-PERF-GATE-001 scope.
estimated_days: 1.0
risk: LOW
acceptance_criteria_count: 9
red_gate_tests: 2
estimated_passes: "2"
holdout_scenarios: []
assumption_validations: []
risk_mitigations: []
design_source: ".factory/research/test-suite-performance-diagnosis-2026-06-26.md §7c (item 4) and §10 skeleton (AC-006/AC-007)"
---

# S-PERF-GATE-002: adv_p02 shared DTU fixtures — extract internal-reset pattern, then LazyLock-share CrowdStrike/Armis clones

## Narrative

As a Prism developer, I want the `adv_p02_e2e_pushdown_pipeline_test` binary to boot
its CrowdStrike and Armis DTU clones once per test binary (shared via `LazyLock`) rather
than once per test, so that the adv_p02 test suite's per-test DTU startup overhead (5–15 s
idle, 60–300 s under oversubscription) is eliminated and the pre-push gate time budget
improves by an estimated 40–120 s warm.

## Scheduling Note

**IMMEDIATE — S-PERF-GATE-001 is merged (PR #204).** This story has no remaining
blocked-by prerequisite. The internal-reset refactor that was the blocker in v1.0 is
now Phase 1 of THIS story's scope. Both phases (reset refactor + LazyLock introduction)
are delivered atomically by the implementer in a single worktree.

`depends_on: [S-PERF-GATE-001]` is confirmed satisfied: `[profile.prepush]` and the
`serial-subprocess` test-group infrastructure are present in `.config/nextest.toml` per
the merged PR #204.

## Background

The 8 tests in `crates/prism-bin/tests/adv_p02_e2e_pushdown_pipeline_test.rs` each boot
their own DTU clone independently (no shared `LazyLock`/`OnceLock` fixtures). Under load
this is the dominant cost for this binary: each boot = Axum startup + Tokio thread pool
init + spec loading. See `.factory/research/test-suite-performance-diagnosis-2026-06-26.md`
§7c (item 4) for the full diagnosis and the PR #127 `LazyLock` precedent.

**Performance impact:** After item #4 (shared DTU fixtures), warm isolated `just check`
drops from ~350–720 s (after item #2 only) to ~280–560 s. See diagnosis §9.

**Source-verified facts (from devops investigation — D-1374, 2026-06-27):**

- Both DTU clones implement `BehavioralClone::reset(&self) -> anyhow::Result<()>` in
  their respective `clone.rs` files. `reset()` takes `&self`, so it works behind `Arc`.
- `CrowdstrikeClone::reset` clears all mutable state: containment store, detection store,
  session store.
- `ArmisClone::reset` clears all mutable state: tag store and AQL log.
- HTTP route `POST /dtu/reset` also exists (admin-token gated) in both clones. But the
  7 DTU-using tests (post-Phase-2) use the programmatic `clone.reset().await` path
  directly — NOT the HTTP route. This story continues on the programmatic path.
- `CrowdstrikeClone::new() -> CrowdstrikeClone` (infallible constructor).
- `ArmisClone::new() -> anyhow::Result<ArmisClone>` (fallible; `LazyLock` init must use
  `.expect("ArmisClone init")`).
- `BehavioralClone::start_on(&mut self, bind: SocketAddr, shutdown: Option<broadcast::Receiver<()>>, tls: Option<()>) -> anyhow::Result<SocketAddr>` takes `&mut self`, which means `start_on` must be called BEFORE the clone is moved into an `Arc`. After `start_on` returns, the `Arc` is constructed around the now-started clone.

**Test inventory (source-verified against adv_p02_e2e_pushdown_pipeline_test.rs):**

| Test | Has mid-test `.reset()` (pre-refactor) | Uses DTU clone type | `.reset()` line (pre-refactor) |
|------|----------------------------------------|---------------------|-------------------------------|
| `test_adv_p02_e2e_crowdstrike_fql_from_where_predicate` | YES | CrowdStrike | 356 |
| `test_adv_p02_e2e_crowdstrike_limit_from_pql_limit_clause` | NO (gains reset-at-start under shared fixture) | CrowdStrike | — |
| `test_adv_p02_e2e_armis_aql_augmentation_from_where_predicate` | YES | Armis | 747 |
| `test_adv_p02_sid1_armis_fetch_start_time_augments_aql` | NO | wiremock (no DTU) | — |
| `test_ac_cws_002_fql_time_window_both_start_and_end_via_materialization_pipeline` | YES | CrowdStrike | 1224 |
| `test_ac_equiv_001_result_equivalence_via_run_materialization_pipeline` | YES | CrowdStrike | 1502 |
| `test_adv_p08_med001_crowdstrike_inclusive_boundary_via_run_materialization_pipeline` | YES | CrowdStrike | 1844 |
| `test_adv_p08_med001_armis_inclusive_boundary_via_run_materialization_pipeline` | YES | Armis | 2098 |

6 tests have mid-test `.reset()` calls (between their filtered and unfiltered pipeline
runs) in the pre-refactor source. 1 test (`test_adv_p02_e2e_crowdstrike_limit_from_pql_limit_clause`)
had no mid-test reset but gains a test-start reset when it moves to the shared CS_DTU
fixture. 1 test uses a wiremock mock server (no DTU clone) and is exempt.

**Post-Phase-2 reset count:** All 7 DTU-using tests call `clone.reset().await` at
test-start (source-verified: 7 `.reset()` call sites in the shipped test file — confirmed
by `grep '\.reset()' adv_p02_e2e_pushdown_pipeline_test.rs`). The wiremock test has no
reset. Total reset call sites: 7.

## Two-Phase Implementation

### Phase 1: Internal-Reset Refactor

Move the 6 mid-test `clone.reset()` calls from their current positions (mid-test, between
filtered and unfiltered pipeline runs) to **test-start boundary** (immediately after the
shared DTU handle is obtained, before any pipeline run). The call becomes:
`cs_dtu.clone.reset().await.expect("reset at test start")`.

**Effect:** Each test that asserts on the wire-log arrives at its assertions with a clean
log. The unfiltered baseline run is then conducted WITHOUT a reset between runs (the
test-start reset is sufficient because the shared LazyLock fixture is used serially).

**Rationale for moving reset-to-start rather than removing it:** The tests that compare
filtered vs. unfiltered runs (`test_adv_p02_e2e_crowdstrike_fql_from_where_predicate`,
`test_ac_cws_002_fql_time_window_both_start_and_end`, etc.) use a two-execution pattern:
execution A (with time predicate) captures filter_strings, then the clone is reset, then
execution B (without predicate) provides the unfiltered baseline for the
`filtered_count < unfiltered_count` assertion. The reset separates the two executions'
wire-log entries. By moving the reset from between-the-two-executions to before-the-first,
and serializing tests via nextest test-group, the same isolation is preserved: test N's
reset clears state left by test N-1; executions A and B within test N proceed with a clean
log and accumulate wire-log entries independently. Since tests run serially (AC-007),
execution B's wire-log entries are never contaminated by a concurrent test.

### Phase 2: LazyLock Shared Fixtures

Introduce `LazyLock<(Arc<CrowdstrikeClone>, SocketAddr)>` (CrowdStrike) and
`LazyLock<(Arc<ArmisClone>, SocketAddr)>` (Armis) as `static` items. Each test obtains
the shared handle via `cs_dtu()` / `armis_dtu()` accessor functions, calls
`handle.clone.reset().await` at its start, then proceeds.

**LazyLock initializer pattern (CrowdStrike):**

```rust
use std::sync::{Arc, LazyLock};
use prism_dtu_common::BehavioralClone;

struct TestDtuHandle<C> {
    clone: Arc<C>,
    addr: std::net::SocketAddr,
}

static CS_DTU: LazyLock<TestDtuHandle<CrowdstrikeClone>> = LazyLock::new(|| {
    let rt = tokio::runtime::Runtime::new()
        .expect("adv_p02 CS_DTU: tokio Runtime::new failed");
    rt.block_on(async {
        let mut clone = CrowdstrikeClone::new();
        let addr = clone
            .start_on("127.0.0.1:0".parse().unwrap(), None, None)
            .await
            .expect("adv_p02 CS_DTU: CrowdStrike DTU clone failed to start");
        TestDtuHandle { clone: Arc::new(clone), addr }
    })
});

static ARMIS_DTU: LazyLock<TestDtuHandle<ArmisClone>> = LazyLock::new(|| {
    let rt = tokio::runtime::Runtime::new()
        .expect("adv_p02 ARMIS_DTU: tokio Runtime::new failed");
    rt.block_on(async {
        let mut clone = ArmisClone::new()
            .expect("adv_p02 ARMIS_DTU: ArmisClone::new failed");
        let addr = clone
            .start_on("127.0.0.1:0".parse().unwrap(), None, None)
            .await
            .expect("adv_p02 ARMIS_DTU: Armis DTU clone failed to start");
        TestDtuHandle { clone: Arc::new(clone), addr }
    })
});
```

**Important:** `start_on` takes `&mut self`. The clone is `mut` inside the initializer
block and moved into `Arc::new(clone)` AFTER `start_on` completes. The `Arc` is only
constructed at that point. `reset()` takes `&self` and works through `Arc::clone`.

Each `#[tokio::test]` that uses the shared CrowdStrike clone opens with:

```rust
let cs = &*CS_DTU;
cs.clone.reset().await.expect("reset at test start — S-PERF-GATE-002");
let dtu_base_url = format!("http://{}", cs.addr);
```

The `dtu_base_url` is then passed to `spec.base_url = dtu_base_url.clone()` as before.

**The `test_adv_p02_sid1_armis_fetch_start_time_augments_aql` test** uses a `wiremock::MockServer`,
NOT an Armis DTU clone. It must NOT be modified to use `ARMIS_DTU` — it continues to
construct its own `wiremock::MockServer` per-test.

**The two-execution tests (`test_adv_p02_e2e_crowdstrike_fql_from_where_predicate`,
`test_ac_cws_002_fql_time_window_both_start_and_end`, `test_ac_equiv_001_result_equivalence`,
and the p08 boundary tests):** After the test-start reset, execution A runs (with time
predicate), then execution B runs (unfiltered baseline). The mid-test `clone.reset().await`
call that previously separated A and B is REMOVED. The wire-log isolation is now ensured
by serialization (AC-007) + test-start reset rather than a mid-test reset. The
`filtered_count < unfiltered_count` assertion continues to hold: since no reset fires
between A and B, the wire-log ACCUMULATES both A's and B's entries. The filter_strings
assertion for A is verified before B runs, using only the entries from A. The
`filtered_count < unfiltered_count` assertion compares row counts from two separate
pipeline executions, not wire-log counts, so accumulation does not affect it.

**If the implementation review determines that the mid-test reset is LOAD-BEARING for
the wire-log assertion semantics** (i.e., execution B's wire-log entries must not appear
in execution A's `filter_strings` slice), the implementer MUST retain the mid-test reset
and instead use a per-entry correlation key (Option 2 fallback below). The implementer
must source-verify which path is correct before committing.

## Concurrency Safety: Chosen Mechanism (Option 1 — Serial Test-Group)

**OPTION 1 is selected.** All adv_p02 tests that share a DTU clone are placed in a
nextest serial test-group `adv-p02-serial` with `max-threads = 1`, scoped to the
`adv_p02_e2e_pushdown_pipeline_test` binary in both `[profile.prepush]` and
`[profile.ci]` via `[[profile.prepush.overrides]]` and `[[profile.ci.overrides]]`.

**Why Option 1:**
- The 8 tests in this binary already run in serial order within a single nextest process
  when test-thread parallelism for this binary is 1. The existing `serial-subprocess`
  group (S-PERF-GATE-001) demonstrates this pattern is stable.
- LazyLock is initialized once. The reset-at-start pattern then provides full inter-test
  isolation.
- Net result: 1 DTU boot per clone (CrowdStrike + Armis = 2 boots instead of 7), tests
  run serially, each test's assertions are deterministic.

**Why NOT Option 2:**
- Per-test correlation keys would require changes to the DTU wire-log structure (exposing
  a correlation-keyed view of the log), which touches DTU clone source outside prism-bin.
  That is architectural scope expansion requiring architect routing.
- The performance win from shared boot is NOT negated by serial execution because the
  bottleneck is boot time (5–15 s per DTU), not inter-test parallelism (the 8 tests take
  negligible time to run once booted).

**The `test_adv_p02_sid1_armis_fetch_start_time_augments_aql` test** (wiremock-based, no
DTU) is included in the `adv-p02-serial` group since it is in the same binary. It costs
nothing to serialize it alongside the DTU tests.

**Nextest config addition to `.config/nextest.toml`:**

```toml
[[profile.prepush.overrides]]
filter = 'binary(adv_p02_e2e_pushdown_pipeline_test)'
test-group = 'adv-p02-serial'

[[profile.ci.overrides]]
filter = 'binary(adv_p02_e2e_pushdown_pipeline_test)'
test-group = 'adv-p02-serial'
```

And in the `[test-groups]` table (adding one entry alongside the existing
`serial-subprocess`):

```toml
[test-groups]
serial-subprocess = { max-threads = 1 }
adv-p02-serial = { max-threads = 1 }
```

The group name is `adv-p02-serial` (kebab-case, mirrors `serial-subprocess` convention).
The filter is `binary(adv_p02_e2e_pushdown_pipeline_test)` — matches by binary name,
not test-name regex (same pattern as the signal_handlers group in S-PERF-GATE-001 AC-011).

## Behavioral Contracts

This is a test-infrastructure maintenance story. It does not introduce new product
behavioral contracts. BC-5.39.001 (3-CLEAN convergence) governs the delivery process
for this story's own PR cascade.

| BC ID | Title | Relevant Clause |
|-------|-------|-----------------|
| BC-5.39.001 | 3-CLEAN Convergence Protocol | All adversarial cascades require 3 consecutive clean passes (CLEAN-strict: zero findings of any severity). The 3-CLEAN gate for this story must use `PROPTEST_CASES=100` to match `just check` strength. |

**Anchor justification per POL-5:** BC-5.39.001 anchors this story as the governing
delivery-quality invariant. No product BCs apply to a test-infrastructure refactor.

## Acceptance Criteria

### AC-001: Phase 1 — all 6 mid-test `clone.reset()` calls moved to test-start boundary (traces to BC-5.39.001 §Delivery process — refactor is a prerequisite for safe shared fixture introduction)

The 6 mid-test `.reset()` calls in the following tests are removed from their mid-test
positions in `crates/prism-bin/tests/adv_p02_e2e_pushdown_pipeline_test.rs`:

1. `test_adv_p02_e2e_crowdstrike_fql_from_where_predicate`
2. `test_adv_p02_e2e_armis_aql_augmentation_from_where_predicate`
3. `test_ac_cws_002_fql_time_window_both_start_and_end_via_materialization_pipeline`
4. `test_ac_equiv_001_result_equivalence_via_run_materialization_pipeline`
5. `test_adv_p08_med001_crowdstrike_inclusive_boundary_via_run_materialization_pipeline`
6. `test_adv_p08_med001_armis_inclusive_boundary_via_run_materialization_pipeline`

Each of the 6 affected tests gains a reset call at test-start (immediately after
acquiring the shared DTU handle, before any pipeline execution).

Note: the pre-refactor mid-test `.reset()` line positions (356, 747, 1224, 1502, 1844,
2098 in the pre-Phase-1 source) are recorded in the §Background test inventory table
as historical reference only. Those line numbers are volatile and will shift once this
refactor lands — the test function names above are the durable anchors (TD-VSDD-091).

**Source verification:** After Phase 1, running
`grep -n '\.reset()' crates/prism-bin/tests/adv_p02_e2e_pushdown_pipeline_test.rs`
must return ZERO hits from mid-test positions. All `.reset()` calls must appear in the
per-test prologue before the first pipeline execution.

The test `test_adv_p02_sid1_armis_fetch_start_time_augments_aql` (wiremock-based, no DTU)
is NOT modified in Phase 1.

### AC-002: Phase 2 — `LazyLock<TestDtuHandle<CrowdstrikeClone>>` static initialized once per binary (traces to BC-5.39.001 §Delivery process — per-test boot cost eliminated)

`crates/prism-bin/tests/adv_p02_e2e_pushdown_pipeline_test.rs` gains a `static CS_DTU:
LazyLock<TestDtuHandle<CrowdstrikeClone>>` initialized with `LazyLock::new(|| { ... })`.
The initializer:
1. Creates a `tokio::runtime::Runtime` (blocking runtime, not the async test runtime).
2. Calls `CrowdstrikeClone::new()` (infallible).
3. Calls `clone.start_on("127.0.0.1:0".parse().unwrap(), None, None).await` via `rt.block_on`.
4. Wraps the started clone in `Arc::new(clone)` and returns `TestDtuHandle { clone, addr }`.

Running `grep 'CS_DTU\|LazyLock' crates/prism-bin/tests/adv_p02_e2e_pushdown_pipeline_test.rs`
must return at least one hit for the static declaration.

No `CrowdstrikeClone::new()` call may appear inside any individual `#[tokio::test]` body
after Phase 2 (all CrowdStrike DTU construction is in the LazyLock initializer only).
See Red Gate RG-PERF-002.

### AC-003: Phase 2 — `LazyLock<TestDtuHandle<ArmisClone>>` static initialized once per binary (traces to BC-5.39.001 §Delivery process — Armis per-test boot cost eliminated)

`crates/prism-bin/tests/adv_p02_e2e_pushdown_pipeline_test.rs` gains a `static ARMIS_DTU:
LazyLock<TestDtuHandle<ArmisClone>>` initialized with `LazyLock::new(|| { ... })`.
The initializer:
1. Creates a `tokio::runtime::Runtime`.
2. Calls `ArmisClone::new().expect("adv_p02 ARMIS_DTU: ArmisClone::new failed")` (fallible — LazyLock init uses `expect`).
3. Calls `clone.start_on(...).await` via `rt.block_on`.
4. Wraps in `Arc::new(clone)` and returns `TestDtuHandle { clone, addr }`.

Running `grep 'ARMIS_DTU\|LazyLock' crates/prism-bin/tests/adv_p02_e2e_pushdown_pipeline_test.rs`
must return at least one hit for the static declaration.

No `ArmisClone::new()` call may appear inside any individual `#[tokio::test]` body after
Phase 2 (all Armis DTU construction is in the LazyLock initializer only). See Red Gate
RG-PERF-003.

### AC-004: Each test calls DTU `reset` at test-start boundary before any pipeline execution (traces to BC-5.39.001 §Delivery process — inter-test isolation maintained)

Every `#[tokio::test]` that uses `CS_DTU` or `ARMIS_DTU` must call
`handle.clone.reset().await.expect("reset at test start — S-PERF-GATE-002")` as the
first action after acquiring the shared handle reference. No pipeline execution may
precede the per-test reset.

The `test_adv_p02_sid1_armis_fetch_start_time_augments_aql` test is exempt (it does not
use a DTU clone; it uses `wiremock::MockServer` which constructs independently per-test).

After Phase 2, `grep -n '\.reset()' crates/prism-bin/tests/adv_p02_e2e_pushdown_pipeline_test.rs`
must return exactly 7 hits (one per DTU-using test), all in the per-test prologue. The
7th reset (for `test_adv_p02_e2e_crowdstrike_limit_from_pql_limit_clause`) is NEW under
the shared-fixture model: this test had no reset in the original per-test-boot pattern,
but correctly gains a test-start reset when it migrates to share CS_DTU with the other
CrowdStrike tests. The `test_adv_p02_sid1_armis_fetch_start_time_augments_aql` test
(wiremock-based) has no reset and is exempt — 7 DTU-using tests + 1 wiremock test = 8
total tests; 7 resets expected.

### AC-005: `adv-p02-serial` nextest test-group added to `.config/nextest.toml` with `max-threads = 1` (traces to BC-5.39.001 §Delivery process — concurrency-safety isolation enforced structurally)

`.config/nextest.toml` gains an `adv-p02-serial` entry in the `[test-groups]` table:

```toml
[test-groups]
serial-subprocess = { max-threads = 1 }
adv-p02-serial = { max-threads = 1 }
```

And two `[[overrides]]` entries (one per profile):

```toml
[[profile.prepush.overrides]]
filter = 'binary(adv_p02_e2e_pushdown_pipeline_test)'
test-group = 'adv-p02-serial'

[[profile.ci.overrides]]
filter = 'binary(adv_p02_e2e_pushdown_pipeline_test)'
test-group = 'adv-p02-serial'
```

The group name is `adv-p02-serial`. The filter is
`binary(adv_p02_e2e_pushdown_pipeline_test)` (binary-name filter, same pattern as
`binary(signal_handlers)` from S-PERF-GATE-001 AC-011 — NOT a test-name regex filter).
The override applies to both `prepush` and `ci` profiles; it is NOT applied to
`[profile.default.overrides]` (same scoping as signal_handlers).

Running `grep 'adv-p02-serial' .config/nextest.toml` must return at least 3 hits:
the `[test-groups]` table entry, the `[[profile.prepush.overrides]]` entry, and the
`[[profile.ci.overrides]]` entry.

A comment above the `adv-p02-serial` group entry documents the rationale, referencing
this story ID (`S-PERF-GATE-002`) and the `LazyLock` shared-fixture pattern.

### AC-006: All 8 adv_p02 tests pass under `cargo nextest run -p prism-bin --test adv_p02_e2e_pushdown_pipeline_test --profile prepush` (traces to BC-5.39.001 §Delivery process — refactor must not alter test coverage)

After both phases and the nextest config change:

```bash
cargo nextest run -p prism-bin --test adv_p02_e2e_pushdown_pipeline_test --profile prepush --no-fail-fast
```

Must exit 0 with all 8 tests passing. No test is removed, `#[ignore]`'d, or weakened.
The 8 test names (source-verified) are:
1. `test_adv_p02_e2e_crowdstrike_fql_from_where_predicate`
2. `test_adv_p02_e2e_crowdstrike_limit_from_pql_limit_clause`
3. `test_adv_p02_e2e_armis_aql_augmentation_from_where_predicate`
4. `test_adv_p02_sid1_armis_fetch_start_time_augments_aql`
5. `test_ac_cws_002_fql_time_window_both_start_and_end_via_materialization_pipeline`
6. `test_ac_equiv_001_result_equivalence_via_run_materialization_pipeline`
7. `test_adv_p08_med001_crowdstrike_inclusive_boundary_via_run_materialization_pipeline`
8. `test_adv_p08_med001_armis_inclusive_boundary_via_run_materialization_pipeline`

Test isolation must be preserved: each test must query and assert independently,
verifying its own predicate against the DTU's request/filter log after the per-test reset.

### AC-007: `filter_strings` assertions in filtered pipeline runs are deterministic across repeated runs and under `--profile prepush` retries (traces to BC-5.39.001 §Delivery process — ADV-P02-CRIT-001 push-down correctness proof must not become flaky)

The `filter_strings` assertions (which verify FQL/AQL push-down predicate correctness —
ADV-P02-CRIT-001) must hold deterministically across repeated executions. Running the
full adv_p02 binary 3 consecutive times without code changes must produce 8/8 passing
each time:

```bash
for i in 1 2 3; do cargo nextest run -p prism-bin --test adv_p02_e2e_pushdown_pipeline_test --profile prepush; done
```

All 3 runs must exit 0. If any run fails on a `filter_strings` assertion, it is a
**serial isolation failure** (the test-start reset did not fully isolate the wire-log)
— this is a P1 CRITICAL finding that blocks the story's merge.

This AC is the load-bearing correctness proof that Option 1 serialization is sufficient.

### AC-008: No per-test `CrowdstrikeClone::new()` or `ArmisClone::new()` construction inside any `#[tokio::test]` body (traces to BC-5.39.001 §Delivery process — structural invariant enforced)

After Phase 2, there must be NO calls to `CrowdstrikeClone::new()` or `ArmisClone::new()`
inside any `#[tokio::test]` function body in
`crates/prism-bin/tests/adv_p02_e2e_pushdown_pipeline_test.rs`.

All DTU construction must be in the `LazyLock::new(|| { ... })` initializer only.

Verified by:
```bash
grep -n 'CrowdstrikeClone::new\|ArmisClone::new' crates/prism-bin/tests/adv_p02_e2e_pushdown_pipeline_test.rs
```

Must return exactly 2 hits: the CS_DTU initializer and the ARMIS_DTU initializer (both
inside `LazyLock::new(|| { ... })` blocks). Zero hits may appear inside `#[tokio::test]`
bodies.

### AC-009: Wall-clock improvement recorded; `just check` reflects adv_p02 boot amortization (traces to BC-5.39.001 §Delivery process — story purpose delivered and measurable)

After all changes:
```bash
time just check
```

The total pre-push gate time should reflect that the 7 DTU-booting adv_p02 tests now
share 2 boots (1 CrowdStrike + 1 Armis) instead of booting individually. The measured
warm-cache projection is 280–560 s (down from 350–720 s after S-PERF-GATE-001 only).

Record the measured wall-clock in the PR description. This AC is a regression benchmark,
not a hard gate — if the machine is contended during measurement, note the load conditions.
If `just check` exceeds 600 s on a warm idle machine, that is a P2 finding for the next
maintenance sweep (not a blocker for this story's merge).

## Red Gate Design

### RG-PERF-002: No `CrowdstrikeClone::new()` inside any `#[tokio::test]` body

**Test file:** `crates/prism-bin/tests/adv_p02_e2e_pushdown_pipeline_test.rs`

**Red Gate mechanism:** A compile-time assertion via a proc-macro is not available for
test-code position constraints, so the Red Gate is a **unit test** that parses the source
file and asserts the structural invariant:

```rust
// In adv_p02_e2e_pushdown_pipeline_test.rs, as a separate test function:
#[test]
fn rg_perf_002_no_per_test_crowdstrike_clone_construction() {
    // Parse this test file's own source and assert that CrowdstrikeClone::new()
    // does not appear inside any #[tokio::test] body.
    // Implemented by checking that the only CrowdstrikeClone::new() call is inside
    // a LazyLock::new(|| { ... }) block (i.e., NOT inside an `async fn test_*` body).
    //
    // Simple structural check: read the file, verify exactly 1 occurrence of
    // "CrowdstrikeClone::new()" and that it appears on a line within a LazyLock
    // initializer block (proxied by: the preceding 10 lines contain "LazyLock::new").
    let source = std::fs::read_to_string(file!())
        .expect("RG-PERF-002: must be able to read own source file");
    let occurrences: Vec<_> = source.match_indices("CrowdstrikeClone::new()").collect();
    assert_eq!(
        occurrences.len(),
        1,
        "RG-PERF-002: expected exactly 1 CrowdstrikeClone::new() call (in LazyLock init); \
         found {}. All per-test clone construction must be eliminated.",
        occurrences.len()
    );
    // Verify the single occurrence is near a LazyLock::new( call.
    let byte_pos = occurrences[0].0;
    let preceding = &source[..byte_pos];
    assert!(
        preceding.rfind("LazyLock::new").is_some(),
        "RG-PERF-002: CrowdstrikeClone::new() must only appear inside a LazyLock::new \
         initializer block"
    );
}
```

**Fails before Phase 2:** The test file has 5 `CrowdstrikeClone::new()` calls (one per
CrowdStrike-DTU test). The `assert_eq!(occurrences.len(), 1, ...)` fails.
**Passes after Phase 2:** Only 1 `CrowdstrikeClone::new()` call remains (in the
`CS_DTU` LazyLock initializer). `rfind("LazyLock::new")` finds the initializer.

### RG-PERF-003: No `ArmisClone::new()` inside any `#[tokio::test]` body

**Same structural pattern as RG-PERF-002, for ArmisClone:**

```rust
#[test]
fn rg_perf_003_no_per_test_armis_clone_construction() {
    let source = std::fs::read_to_string(file!())
        .expect("RG-PERF-003: must be able to read own source file");
    let occurrences: Vec<_> = source.match_indices("ArmisClone::new()").collect();
    assert_eq!(
        occurrences.len(),
        1,
        "RG-PERF-003: expected exactly 1 ArmisClone::new() call (in LazyLock init); \
         found {}. All per-test clone construction must be eliminated.",
        occurrences.len()
    );
    let byte_pos = occurrences[0].0;
    let preceding = &source[..byte_pos];
    assert!(
        preceding.rfind("LazyLock::new").is_some(),
        "RG-PERF-003: ArmisClone::new() must only appear inside a LazyLock::new \
         initializer block"
    );
}
```

**Fails before Phase 2:** 2 `ArmisClone::new()` calls in test bodies. Assertion
`occurrences.len() == 1` fails.
**Passes after Phase 2:** Only 1 `ArmisClone::new()` call (in `ARMIS_DTU` LazyLock).

**Both Red Gate tests are non-`#[tokio::test]` (plain `#[test]`).** They do not boot
any DTU; they read the file and assert structure. They run fast (<1 ms). They must be
added to the test file in Phase 1 (before the LazyLock introduction), so they are
FAILING during the Phase 1 → Phase 2 transition and PASSING after Phase 2.

**Red Gate verification command:**

```bash
cargo nextest run -p prism-bin --test adv_p02_e2e_pushdown_pipeline_test \
    -E 'test(rg_perf_002|rg_perf_003)' --no-fail-fast
```

Must fail during Phase 1 (before LazyLock introduction) and pass after Phase 2.

## Token Budget Estimate

| Item | Estimated Tokens |
|------|-----------------|
| Story spec (this file) | ~10 000 |
| `crates/prism-bin/tests/adv_p02_e2e_pushdown_pipeline_test.rs` (full file — 2173 lines; all 8 tests require reading for reset-call positions and DTU construction sites) | ~6 500 |
| `crates/prism-dtu-crowdstrike/src/clone.rs` (constructor + start_on + reset signatures; verify API) | ~800 |
| `crates/prism-dtu-armis/src/clone.rs` (constructor + start_on + reset signatures) | ~800 |
| `crates/prism-dtu-common/src/clone.rs` (BehavioralClone trait, start_on + reset signatures) | ~400 |
| `.config/nextest.toml` (full file; add adv-p02-serial group + overrides) | ~900 |
| BC-5.39.001 (1 BC file) | ~800 |
| S-PERF-GATE-001 story (Previous Story Intelligence; AC-011 nextest group pattern) | ~8 000 |
| `cargo nextest run` output (adv_p02 binary, 8 tests) | ~400 |
| **Total** | **~28 600** |

Context window headroom: ~29k tokens is ~8% of a 350k context window.
No story splitting required. Single implementer dispatch covers all phases.

## Tasks

### Phase 1: Internal-Reset Refactor

1. **Write Red Gate tests RG-PERF-002 and RG-PERF-003** at the bottom of
   `adv_p02_e2e_pushdown_pipeline_test.rs` (before any LazyLock changes).
   Verify they fail:
   ```bash
   cargo nextest run -p prism-bin --test adv_p02_e2e_pushdown_pipeline_test \
       -E 'test(rg_perf_002|rg_perf_003)' --no-fail-fast
   ```
   Expected: both fail with "found N, expected 1" (N = 5 for CS, N = 2 for Armis).

2. **Source-verify the 6 mid-test `.reset()` call positions** against the actual test
   file. The pre-authoring line numbers (356, 747, 1224, 1502, 1844, 2098) are provided
   as implementation start-points only (TD-VSDD-091 task-level exception per Architecture
   Compliance Rule 8). Confirm that each call is inside the correct test function per
   the §Background table. Re-verify at worktree start — lines shift if another PR has
   touched the file since this story was written.

3. **Remove the 6 mid-test `.reset()` calls.** For each of the 6 affected tests, remove
   the `clone.reset().await.expect(...)` call from its mid-test position. The tests now
   use a single DTU per test with no mid-test state reset between filtered and unfiltered
   runs.

4. **Verify all 8 tests still pass with per-test clone construction** (pre-LazyLock):
   ```bash
   cargo nextest run -p prism-bin --test adv_p02_e2e_pushdown_pipeline_test \
       --profile prepush --no-fail-fast
   ```
   Expected: 8/8 pass. If any `filter_strings` assertion fails after mid-test reset
   removal, the mid-test reset IS load-bearing — restore it and implement the per-entry
   correlation key approach (Option 2 fallback) instead, routing to architect before
   proceeding.

### Phase 2: LazyLock Introduction

5. **Introduce `TestDtuHandle<C>` struct and `CS_DTU` static** per the design in §Background.
   The struct holds `Arc<C>` and `SocketAddr`. `CS_DTU` uses `tokio::runtime::Runtime::new()`
   + `rt.block_on(async { ... })` to drive the async `start_on`. Infallible constructor
   for CrowdStrike (`CrowdstrikeClone::new()`).

6. **Introduce `ARMIS_DTU` static** for Armis. Fallible constructor
   (`ArmisClone::new().expect(...)`).

7. **Rewrite all 7 DTU-using tests** to use the shared handle (5 CrowdStrike + 2 Armis):
   - Replace `let mut clone = CrowdstrikeClone::new(); let addr = clone.start_on(...).await...`
     with `let cs = &*CS_DTU; cs.clone.reset().await.expect("reset at test start — S-PERF-GATE-002"); let dtu_base_url = format!("http://{}", cs.addr);`
   - Same pattern for Armis tests using `ARMIS_DTU`.
   - Include `test_adv_p02_e2e_crowdstrike_limit_from_pql_limit_clause` — this test had
     no reset in the original code but correctly gains a test-start reset here (it shares
     CS_DTU and must reset to clear any state left by the preceding test).
   - The `test_adv_p02_sid1_armis_fetch_start_time_augments_aql` test is NOT modified
     (it uses `wiremock::MockServer`).

8. **Verify Red Gate tests now pass:**
   ```bash
   cargo nextest run -p prism-bin --test adv_p02_e2e_pushdown_pipeline_test \
       -E 'test(rg_perf_002|rg_perf_003)' --no-fail-fast
   ```
   Expected: both pass (1 CS constructor, 1 Armis constructor, both in LazyLock).

9. **Add `adv-p02-serial` test-group to `.config/nextest.toml`** per AC-005. Add the
   `[test-groups]` entry and the two `[[profile.prepush.overrides]]`/`[[profile.ci.overrides]]`
   entries. Add the comment block citing S-PERF-GATE-002 and the LazyLock pattern.

10. **Run all 8 adv_p02 tests under the prepush profile** per AC-006:
    ```bash
    cargo nextest run -p prism-bin --test adv_p02_e2e_pushdown_pipeline_test \
        --profile prepush --no-fail-fast
    ```
    Expected: 8/8 pass.

11. **Determinism check** per AC-007 — run 3 consecutive times:
    ```bash
    for i in 1 2 3; do
        cargo nextest run -p prism-bin --test adv_p02_e2e_pushdown_pipeline_test \
            --profile prepush
    done
    ```
    All 3 runs must pass. If any `filter_strings` assertion fails, the serial isolation
    is insufficient — see Phase 1 Task 4 fallback.

12. **Run full per-crate gate:**
    ```bash
    just iter prism-bin
    ```
    Must exit 0.

13. **Run full pre-push gate and record wall-clock:**
    ```bash
    time just check
    ```
    Must exit 0. Record the time for AC-009 regression benchmark.

14. **TD-VSDD-060 sibling-site sweep:** Verify no other test file constructs
    `CrowdstrikeClone::new()` or `ArmisClone::new()` in a per-test pattern that should
    be LazyLocked. This story only touches adv_p02; other files are out-of-scope but
    must not be silently broken:
    ```bash
    grep -rn 'CrowdstrikeClone::new\|ArmisClone::new' crates/prism-bin/tests/
    ```
    Confirm any hits in OTHER test files are unaffected by this story's changes.

## Previous Story Intelligence

**S-PERF-GATE-001 (MERGED PR #204, predecessor):**

1. **Established `[profile.prepush]` and `[profile.ci]` with retries + terminate-after.**
   The `adv-p02-serial` test-group added in this story follows the same `[[profile.prepush.overrides]]` /
   `[[profile.ci.overrides]]` pattern as the `serial-subprocess` group from AC-011. Read
   AC-011 in S-PERF-GATE-001 for the exact group-name + filter syntax before writing the
   new override entries.

2. **`serial-subprocess = { max-threads = 1 }` is already in `[test-groups]`** (source-verified
   against current `.config/nextest.toml`). Adding `adv-p02-serial` is an additive change
   to the existing `[test-groups]` table — do NOT replace `serial-subprocess`.

3. **Narrative quantitative claims must be source-verified.** S-PERF-GATE-001's cascade
   had multiple findings caused by counts and line numbers in story prose that diverged
   from actual source. For this story: the 6 tests with mid-test resets are named in
   AC-001 (behavioral anchors per TD-VSDD-091). The pre-refactor `.reset()` line numbers
   (356, 747, 1224, 1502, 1844, 2098) are recorded in §Background as historical reference
   and in Task 2 as implementation start-points only. The implementer MUST re-verify these
   line numbers at the start of the worktree (they may shift if another PR has touched the
   file since this story was written). Post-Phase-2, 7 `.reset()` calls are expected —
   the original 6 mid-test calls move to test-start, and the limit test gains a new
   test-start reset when it joins the shared CS_DTU fixture.

4. **The `test_adv_p02_sid1_armis_fetch_start_time_augments_aql` test is NOT a DTU test.**
   It uses `wiremock::MockServer`. Previous story analysis (S-PERF-GATE-001 v1.5 fix)
   clarified that 7 of 8 tests use a DTU clone and 1 uses wiremock. Do not add
   `ARMIS_DTU` usage to the wiremock test.

5. **PR #127 LazyLock pattern is the canonical precedent.** The `[profile.ci]`
   `# Updated 2026-05-06 (PR #127)` commentary block in `.config/nextest.toml`
   documents the `LazyLock<TempDir>` + `LazyLock<Runtime>` pattern from prism-credentials
   proptest (18x speedup). Read it before writing the DTU `LazyLock` initializers. The
   key difference here: the DTU `LazyLock` holds `Arc<Clone>` + `SocketAddr` instead of
   `TempDir` — but the initialization pattern (blocking `Runtime::new()` + `rt.block_on`)
   is identical.

6. **AC-006 deferral rationale (from S-PERF-GATE-001 §AC-006 STATUS block):** The
   mid-test `.reset()` calls were the safety gate that blocked S-PERF-GATE-001 from
   delivering the LazyLock optimization. This story resolves that gate by moving the
   resets to test-start (Phase 1) before introducing the shared fixtures (Phase 2). The
   deferral is CLOSED by this story — there is no further prerequisite.

## Architecture Compliance Rules

1. **No `#[ignore]` without a specific story ID and test name citation (SID-1).** Do not
   mark any adv_p02 test as `#[ignore]` as part of this refactor. If a test fails after
   the reset refactor, investigate and fix in-scope.

2. **No expansion of prism-bin's public API surface.** This story only modifies the test
   file `adv_p02_e2e_pushdown_pipeline_test.rs` and the config file `.config/nextest.toml`.
   No production source files (`src/`) are touched. No `pub` declarations change.

3. **No DTU clone source modification.** Both `CrowdstrikeClone` and `ArmisClone` already
   expose `reset(&self)` via `BehavioralClone`. No changes to `prism-dtu-crowdstrike` or
   `prism-dtu-armis` source are permitted in this story. If any DTU modification is
   discovered to be necessary, stop and route to architect before proceeding.

4. **`test_adv_p02_sid1_armis_fetch_start_time_augments_aql` (wiremock test) must not
   be modified.** It constructs its own `wiremock::MockServer` per-test. Introducing
   ARMIS_DTU into this test would change its isolation model (currently no DTU) and is
   out-of-scope.

5. **ADV-P02-CRIT-001 correctness must not regress.** The `filter_strings` assertions in
   the filtered pipeline runs are the load-bearing proof of FQL/AQL push-down. Any
   regression in these assertions is a P1 CRITICAL finding — it means filter push-down
   has been silently broken by the refactor. The 3-consecutive-run determinism check
   (AC-007) is the safety gate.

6. **`just check` must exit 0 before the PR is opened.** Per CLAUDE.md §Building.

7. **No AI attribution in commits** per project git conventions (CLAUDE.md).

8. **TD-VSDD-091 — narrative behavioral anchors not line numbers.** This story's task
   checklist references pre-refactor line numbers (356, 747, etc.) as implementation
   start-points only (TD-VSDD-091 task-level exemption). The shipped story prose uses
   behavioral anchors: AC-001 names the 6 affected test functions; §Background records
   the pre-refactor line positions as historical reference. The implementer must
   re-verify line numbers in their worktree since they will shift after Phase 1.

9. **CLAUDE.md §Conventions — no `unwrap()` in non-test production code.** This story
   only touches test code (`#[allow(clippy::unwrap_used)]` is present in the test file).
   `expect()` in `LazyLock` initializers and test setup is acceptable per the existing
   adv_p02 pattern.

## Library & Framework Requirements

No new external dependencies. All constructs are already in the workspace.

| Library | Usage in this story | Version / Source |
|---------|---------------------|-----------------|
| `std::sync::LazyLock` | Shared DTU fixture initialization | Rust stdlib, stable since Rust 1.80 (workspace pinned to current stable per `rust-toolchain.toml`) |
| `std::sync::Arc` | Shared ownership of started DTU clone | Rust stdlib |
| `tokio::runtime::Runtime` | Blocking runtime to drive `start_on` in `LazyLock::new(|| { ... })` | tokio, existing workspace pin in root `Cargo.toml` |
| `prism_dtu_common::BehavioralClone` | `reset` + `start_on` trait methods | workspace crate `prism-dtu-common` |
| `prism_dtu_crowdstrike::CrowdstrikeClone` | CrowdStrike DTU (already imported in adv_p02) | workspace crate `prism-dtu-crowdstrike` |
| `prism_dtu_armis::ArmisClone` | Armis DTU (already imported in adv_p02) | workspace crate `prism-dtu-armis` |
| `cargo nextest` | Test runner with test-group support | existing workspace pin |

**Forbidden dependencies:** No new `Cargo.toml` entries. `prism-bin/Cargo.toml` must not
gain new dependencies. If the LazyLock pattern requires a dependency not currently in
`prism-bin/Cargo.toml` (e.g., a newer tokio feature), surface it to the implementer
immediately — it is likely already present via transitive dependencies.

## File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `crates/prism-bin/tests/adv_p02_e2e_pushdown_pipeline_test.rs` | MODIFY | Phase 1: remove 6 mid-test `.reset()` calls; add RG-PERF-002 + RG-PERF-003 Red Gate tests. Phase 2: add `TestDtuHandle<C>` struct, `CS_DTU` + `ARMIS_DTU` statics; rewrite 7 DTU-using tests to use shared handles with test-start reset. |
| `.config/nextest.toml` | MODIFY | Add `adv-p02-serial = { max-threads = 1 }` to `[test-groups]`; add `[[profile.prepush.overrides]]` and `[[profile.ci.overrides]]` entries with `filter = 'binary(adv_p02_e2e_pushdown_pipeline_test)'` and `test-group = 'adv-p02-serial'`. |

**No new files.** No DTU clone source files are modified. No production `src/` files are
modified. No new `Cargo.toml` entries.

**Subsystem anchor:** `subsystems: []` is correct. This story modifies test infrastructure
and build configuration, not product subsystems in the ARCH-INDEX Subsystem Registry.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `filter_strings` assertion fails after mid-test reset removal (Phase 1 Task 4) — mid-test reset IS load-bearing for wire-log isolation between two-execution tests | Restore the mid-test reset; switch to Option 2 (per-entry correlation key for wire-log filtering). Route to architect before modifying DTU clone source. This story's Phase 2 blocks until the architectural decision is made. |
| EC-002 | `LazyLock` initializer panics due to port exhaustion or DTU startup failure on a loaded CI runner | Using `127.0.0.1:0` (OS-assigned ephemeral port) eliminates port conflicts. DTU startup failures produce `expect()` panics in the LazyLock initializer — the panic is caught by nextest as a test-binary crash, all 8 tests fail with a clear message. The `[profile.prepush]` `retries = 1` absorbs one transient failure. |
| EC-003 | `ArmisClone::new()` returns `Err` in the LazyLock initializer on a fresh CI runner | `ArmisClone::new()` initializes a `LazyLock` internally. Its failure mode is documented in `prism-dtu-armis/src/clone.rs`. The `.expect("adv_p02 ARMIS_DTU: ArmisClone::new failed")` message must cite S-PERF-GATE-002 for traceability. |
| EC-004 | `adv-p02-serial` test-group override interacts with `[profile.e2e]` or `[profile.e2e-multi-org]` | The override is applied to `prepush` and `ci` profiles only. Other profiles are unaffected. |
| EC-005 | 3-consecutive-run determinism check (AC-007) fails on the 2nd or 3rd run but not the 1st | This indicates that `LazyLock` state is leaking between runs at the process level. Because nextest spawns a new process per binary invocation, there is no cross-invocation state leakage — `LazyLock` is re-initialized per process. If the check fails, the issue is within-process test ordering (one test corrupting the next). Diagnose by running with `NEXTEST_EXPERIMENTAL_LIBTEST_MIMIC=1` to see test ordering. |
| EC-006 | The `rg_perf_002`/`rg_perf_003` Red Gate tests produce false-positive passes if the file-read path fails or if the source-code-scanning logic is fragile | Both tests use `file!()` macro and `std::fs::read_to_string` — both will panic with a clear error if the file cannot be read. The `match_indices` count assertion provides the structural guarantee. The `rfind("LazyLock::new")` check provides the position guarantee. These two levels of check make the test robust against accidental relocation. |

## Architecture Mapping

| Component | Module | File | Pure/Effectful |
|-----------|--------|------|----------------|
| `TestDtuHandle<C>` struct | prism-bin tests | `crates/prism-bin/tests/adv_p02_e2e_pushdown_pipeline_test.rs` | Pure (data holder, no I/O) |
| `CS_DTU: LazyLock<TestDtuHandle<CrowdstrikeClone>>` | prism-bin tests | `crates/prism-bin/tests/adv_p02_e2e_pushdown_pipeline_test.rs` | Effectful (Axum HTTP server + Tokio runtime in initializer) |
| `ARMIS_DTU: LazyLock<TestDtuHandle<ArmisClone>>` | prism-bin tests | `crates/prism-bin/tests/adv_p02_e2e_pushdown_pipeline_test.rs` | Effectful (Axum HTTP server + Tokio runtime in initializer) |
| `rg_perf_002_no_per_test_crowdstrike_clone_construction` (Red Gate) | prism-bin tests | `crates/prism-bin/tests/adv_p02_e2e_pushdown_pipeline_test.rs` | Pure (file-read only) |
| `rg_perf_003_no_per_test_armis_clone_construction` (Red Gate) | prism-bin tests | `crates/prism-bin/tests/adv_p02_e2e_pushdown_pipeline_test.rs` | Pure (file-read only) |
| `adv-p02-serial` nextest test-group + overrides | build config | `.config/nextest.toml` | N/A — configuration |

## §References

Per POL-7 (verbatim BC H1 titles):

- BC-5.39.001 — *3-CLEAN Convergence Protocol* (delivery gate for this story itself)
- `.factory/research/test-suite-performance-diagnosis-2026-06-26.md` — diagnostic source,
  §7c item 4 (shared DTU fixtures design), §9 (wall-clock projection), §10 skeleton
  (AC-006/AC-007 origin)
- S-PERF-GATE-001 — predecessor story (MERGED PR #204); established `[profile.prepush]`,
  `serial-subprocess` test-group, AC-011 nextest override pattern
- PR #127 — canonical `LazyLock` shared-fixture precedent (prism-credentials proptest
  18x speedup); pattern documentation in the `# Updated 2026-05-06 (PR #127)` commentary
  block in `[profile.ci]` of `.config/nextest.toml`

## §Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| v2.1 | 2026-06-28 | story-writer | LOCAL Pass-1 adversary findings closed: (1) F-SPG2-P1-002 (MEDIUM) — AC-004 reset-count corrected from "exactly 6" to "exactly 7"; §Background test inventory table updated to record that `test_adv_p02_e2e_crowdstrike_limit_from_pql_limit_clause` gains a test-start reset under the shared-fixture model; all "6 reset-bearing tests" / "6 of the 8" prose reconciled to source-verified post-Phase-2 reality (7 DTU-using tests each call reset at test-start). (2) F-SPG2-P1-003 (MEDIUM) — §Changelog reordered newest-first per POL-32 (changelog_monotonic_descending). (3) OBS-1 (LOW) — AC-001 volatile line-pin citations (356, 747, 1224, 1502, 1844, 2098) replaced with the 6 affected test function names as durable behavioral anchors (TD-VSDD-091); historical line numbers moved to §Background table with note that they are pre-refactor positions. Task 2 updated to clarify the TD-VSDD-091 task-level exemption for implementation start-points per Architecture Compliance Rule 8. |
| v2.0 | 2026-06-28 | story-writer | Full elaboration from stub. Incorporated devops investigation facts (D-1374): DTU reset API verified, constructor signatures confirmed, `start_on` takes `&mut self` documented. Chose Option 1 (serial test-group) as concurrency-safety mechanism; rationale documented. Phased scope (Phase 1: reset refactor; Phase 2: LazyLock introduction) replaces blocked_by prerequisite — both phases in-scope. Removed `blocked_by` (internal-reset prerequisite is Phase 1 of this story). Bumped status to `ready`. Added full AC set (9 ACs), 2 Red Gate tests (RG-PERF-002/003), Token Budget, Tasks checklist, Previous Story Intelligence (S-PERF-GATE-001 lessons), Architecture Compliance Rules, Library & Framework Requirements, File Structure Requirements. All factual claims source-verified against adv_p02_e2e_pushdown_pipeline_test.rs, clone.rs files, and .config/nextest.toml. |
| v1.0 | 2026-06-27 | story-writer | Draft stub registered as Canonical Principle Rule 3 deferral anchor for S-PERF-GATE-001 AC-006/007 (D-1368, 2026-06-26). Closes F-P2R11-MED-001 (phantom file resolving POL-22). |
