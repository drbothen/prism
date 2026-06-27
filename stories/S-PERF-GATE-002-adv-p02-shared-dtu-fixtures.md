---
document_type: story
story_id: S-PERF-GATE-002
title: "adv_p02 shared DTU fixtures — extract internal-reset pattern, then LazyLock-share CrowdStrike/Armis clones"
wave: maintenance
epic_id: maintenance
priority: P3
status: draft
version: "1.0"
spec_version: "v1.0"
level: ops
producer: story-writer
timestamp: "2026-06-27"
modified: "2026-06-27"
input-hash: ""
inputs:
  - .factory/research/test-suite-performance-diagnosis-2026-06-26.md
  - crates/prism-bin/tests/adv_p02_e2e_pushdown_pipeline_test.rs
traces_to: "test-suite-performance-diagnosis-2026-06-26"
cycle: "v1.0.0-greenfield"
phase: 3
tdd_mode: strict
track: "Platform Engineering"
subsystems: []
crates_touched:
  - prism-bin
target_module: "crates/prism-bin/tests/adv_p02_e2e_pushdown_pipeline_test.rs"
behavioral_contracts:
  - BC-5.39.001
# NOTE: This is a test-infrastructure maintenance story. It does NOT introduce new
# product behavioral contracts. BC-5.39.001 governs the story's own delivery
# (3-CLEAN convergence requirement). No product BCs are added or modified.
# BC status: BC-5.39.001 is active. Story can advance to ready when PO confirms
# that no additional product BCs are required for this scope.
verification_properties: []
depends_on:
  - S-PERF-GATE-001
# SOFT dependency: S-PERF-GATE-001 delivers items 1/2/3/5/6 of the performance
# diagnosis. S-PERF-GATE-002 delivers item 4 (adv_p02 shared DTU fixtures).
# S-PERF-GATE-001 must be merged first (it establishes the [profile.prepush] gate
# this story's tests will run under), but there is no build-order hard dependency.
blocks: []
blocked_by:
  - internal-reset-refactor-prerequisite
# PREREQUISITE (blocker): 6 of the 8 adv_p02 tests call clone.reset() mid-test
# (between their filtered/unfiltered pipeline runs). The 6 reset-bearing tests are:
#   - crowdstrike_fql_from_where_predicate
#   - armis_aql_augmentation_from_where_predicate
#   - ac_cws_002_fql_time_window
#   - ac_equiv_001_result_equivalence
#   - adv_p08_med001_crowdstrike_inclusive_boundary
#   - adv_p08_med001_armis_inclusive_boundary
# Sharing a LazyLock clone across parallel tests while these tests call clone.reset()
# internally would allow one test's mid-test reset to destroy another's wire-log
# state, breaking the filter_strings assertions (ADV-P02-CRIT-001). The internal-reset
# refactor MUST land first (moved to before-test-start or equivalent isolation
# guarantee) before introducing shared fixtures.
points: null
# TBD — requires full AC decomposition at pickup. Likely 5-8 pts (PR #127 precedent
# suggests 3h implementation; Red Gate + 3-CLEAN cascade adds overhead).
estimated_days: null
risk: LOW
acceptance_criteria_count: 0
# 0 ACs: stub only. Full AC decomposition required at pickup.
red_gate_tests: 0
# 0 Red Gate tests: stub only. Red Gate design required at pickup.
estimated_passes: null
holdout_scenarios: []
assumption_validations: []
risk_mitigations: []
design_source: ".factory/research/test-suite-performance-diagnosis-2026-06-26.md §7c (item 4)"
---

# S-PERF-GATE-002: adv_p02 shared DTU fixtures — extract internal-reset pattern, then LazyLock-share CrowdStrike/Armis clones

> **STUB — not yet elaborated.** This is a Canonical Principle Rule 3 explicit
> future-story deferral anchor registered during S-PERF-GATE-001 (D-1368, 2026-06-26).
> The story must be fully elaborated (full ACs, Red Gate design, Token Budget, Tasks,
> Previous Story Intelligence, Architecture Compliance Rules, Library & Framework
> Requirements, File Structure Requirements) before dispatch to implementer.

## Narrative

As a Prism developer, I want the `adv_p02_e2e_pushdown_pipeline_test` binary to boot
its CrowdStrike and Armis DTU clones once per test binary (shared via `LazyLock`) rather
than once per test, so that the adv_p02 test suite's per-test DTU startup overhead (5–15s
idle, 60–300s under oversubscription) is eliminated and the pre-push gate time budget
improves by an estimated 40–120s warm.

## Scheduling Note

**BLOCKED — do not dispatch to implementer.** This story has a hard prerequisite: the
internal-reset refactor described below must land first. See `blocked_by` frontmatter.

`depends_on: [S-PERF-GATE-001]` is SOFT — S-PERF-GATE-001 must be merged before this
story is picked up (it establishes the nextest profiles under which adv_p02 runs), but
there is no Cargo build-order dependency.

## Background

The 8 tests in `crates/prism-bin/tests/adv_p02_e2e_pushdown_pipeline_test.rs` each boot
their own DTU clone independently (no shared `LazyLock`/`OnceLock` fixtures). Under load
this is the dominant cost for this binary: each boot = Axum startup + Tokio thread pool
init + spec loading. See `.factory/research/test-suite-performance-diagnosis-2026-06-26.md`
§7c (item 4) for the full diagnosis and the PR #127 `LazyLock` precedent.

**Performance impact:** After item #4 (shared DTU fixtures), estimated wall-clock for
`adv_p02` drops from the per-test amortization baseline to a shared-boot model:
warm isolated: 280–560s total gate (down from 350–720s after item #2 only). See
diagnosis §9 (Estimated Isolated `just check` After Fixes table).

## Prerequisite: Internal-Reset Refactor (The Blocker)

6 of the 8 adv_p02 tests call `clone.reset()` **internally mid-test** — between
their filtered and unfiltered pipeline runs. The 6 reset-bearing tests (source-verified
against the test file, D-1374):

| Test Name | Reset Pattern |
|-----------|--------------|
| `crowdstrike_fql_from_where_predicate` | mid-test between filtered/unfiltered runs |
| `armis_aql_augmentation_from_where_predicate` | mid-test between filtered/unfiltered runs |
| `ac_cws_002_fql_time_window` | mid-test between filtered/unfiltered runs |
| `ac_equiv_001_result_equivalence` | mid-test between filtered/unfiltered runs |
| `adv_p08_med001_crowdstrike_inclusive_boundary` | mid-test between filtered/unfiltered runs |
| `adv_p08_med001_armis_inclusive_boundary` | mid-test between filtered/unfiltered runs |

The remaining 2 tests do NOT call `clone.reset()` internally (1 uses wiremock; 1 has no
mid-test reset).

**Why this blocks the LazyLock optimization:** If a `LazyLock` shared clone is introduced
while these 6 tests still call `clone.reset()` internally, a mid-test reset in one test
(running in parallel) will destroy the wire-log state of another test's in-flight request,
breaking the load-bearing `filter_strings` assertions (ADV-P02-CRIT-001 — the core
push-down predicate correctness proof).

**Refactor required:** Move all 6 reset calls from mid-test to before-test-start (or
introduce an equivalent per-test setup mechanism that guarantees DTU state isolation
without per-test DTU boot). Once all 8 tests reset at the start boundary only, introducing
`LazyLock<Arc<TestDtuHandle>>` shared fixtures is safe.

## Scope (to be elaborated at pickup)

This story implements diagnosis item #4 from `.factory/research/test-suite-performance-diagnosis-2026-06-26.md` §7c:

1. **Refactor phase:** Move the 6 internal mid-test `clone.reset()` calls to
   before-test-start reset (or equivalent isolation mechanism). Verify all 8 tests still
   pass after the refactor and the `filter_strings` assertions are correct.

2. **LazyLock phase:** Introduce `LazyLock<Arc<TestDtuHandle>>` shared fixtures for the
   CrowdStrike clone (shared across `cs_*` tests) and Armis clone (shared across `armis_*`
   tests), following the PR #127 LazyLock pattern. Each test calls the DTU reset endpoint
   at test-start to clear filter/request log state before its pipeline run.

3. **Verification:** All 8 adv_p02 tests must pass under `nextest --profile prepush`
   (the profile introduced by S-PERF-GATE-001). The filter_strings assertions must be
   correct for both the filtered and unfiltered pipeline runs in each test.

## Design Input

Primary design reference: `.factory/research/test-suite-performance-diagnosis-2026-06-26.md`
§7c (item 4). The LazyLock pattern skeleton from that document:

```rust
// (from diagnosis §7c — NOT a shipped excerpt; elaborated at pickup)
use std::sync::{Arc, LazyLock};

static CS_DTU: LazyLock<Arc<TestDtuHandle>> = LazyLock::new(|| {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let mut clone = CrowdstrikeClone::new();
        let addr = clone.start_on("127.0.0.1:0".parse().unwrap(), None, None).await.unwrap();
        Arc::new(TestDtuHandle { clone, addr })
    })
});
```

**Prerequisite verification at pickup:** confirm that `CrowdstrikeClone` and `ArmisClone`
expose a reset endpoint (analogous to `POST /reset` in `prism-dtu-slack`) before
implementing shared fixtures. Source: diagnosis §7c "Important constraint."

## Behavioral Contracts

This is a test-infrastructure maintenance story. It does not introduce new product
behavioral contracts. BC-5.39.001 (3-CLEAN convergence) governs the delivery process for
this story's own PR cascade.

| BC ID | Title | Relevant Clause |
|-------|-------|-----------------|
| BC-5.39.001 | 3-CLEAN Convergence Protocol | All adversarial cascades require 3 consecutive clean passes (CLEAN-strict: zero findings of any severity). |

**Anchor justification per POL-5:** BC-5.39.001 anchors this story as the governing
delivery-quality invariant. No product BCs apply to a test-infrastructure refactor.

## Acceptance Criteria

**STUB — ACs not yet authored.** ACs must be elaborated at pickup, tracing to
BC-5.39.001 clauses per the Spec-First Gate (S-7.01). The AC decomposition should cover:

- Refactor: each of the 6 reset-bearing tests calls reset at test-start boundary only (not mid-test)
- All 8 adv_p02 tests pass under `nextest --profile prepush` with shared DTU fixtures active
- `filter_strings` assertions in each filtered pipeline run remain correct
- `LazyLock` shared CrowdStrike and Armis DTU handles are introduced (pattern per PR #127)
- No regression in filter push-down correctness (ADV-P02-CRIT-001)

## Red Gate Design

**STUB — Red Gate tests not yet designed.** At pickup, design a Red Gate test that fails
until the LazyLock fixtures are introduced (proving per-test boot is eliminated). The
Red Gate must be non-trivially distinct from just running the tests — it should assert
a structural invariant (e.g., no local clone construction inside any `#[tokio::test]`
body in the file).

## Token Budget Estimate

**STUB — not calculated.** At elaboration time, estimate:
- Story spec (this file)
- `crates/prism-bin/tests/adv_p02_e2e_pushdown_pipeline_test.rs` (full file — the only file under modification)
- DTU clone source: `crates/prism-dtu-crowdstrike/src/` and `crates/prism-dtu-armis/src/` (reset endpoint verification)
- Research: `.factory/research/test-suite-performance-diagnosis-2026-06-26.md` §7c
- S-PERF-GATE-001 story (Previous Story Intelligence)

Estimated: LOW complexity; test-only file; likely 30–60k tokens total.

## Tasks

**STUB — task checklist not yet authored.** At elaboration, decompose into:
- [ ] Verify DTU reset endpoint exists in CrowdStrike and Armis clones
- [ ] Refactor 6 reset-bearing tests: move `clone.reset()` to test-start boundary
- [ ] Run all 8 tests under `nextest --profile prepush`; confirm all pass + `filter_strings` correct
- [ ] Design and add Red Gate test
- [ ] Introduce `LazyLock<Arc<TestDtuHandle>>` for CrowdStrike clone
- [ ] Introduce `LazyLock<Arc<TestDtuHandle>>` for Armis clone
- [ ] Update each test to call DTU reset at test-start (not in LazyLock initializer)
- [ ] Run full `just check` and record wall-clock improvement

## Previous Story Intelligence

**S-PERF-GATE-001** (predecessor):
- Delivered nextest profile hardening (`[profile.prepush]`, `[profile.ci]` with `retries = 1`
  and `terminate-after = 2`), `Justfile` gate recipe update, `build_http_client_with_timeout`
  fix, sccache stanza, and `signal_handlers` serial-subprocess group.
- The AC-006/AC-007 work (adv_p02 LazyLock shared fixtures) was deferred to this story
  (S-PERF-GATE-002) because of the internal-reset safety issue documented above.
- The `[profile.prepush]` section introduced by S-PERF-GATE-001 is the gate profile this
  story's tests will run under. Confirm it is present before implementing.
- Lesson from S-PERF-GATE-001 cascade: narrative quantitative claims (counts, line numbers,
  test names) must be SOURCE-VERIFIED against the actual test files — not derived from
  internal story consistency alone. Apply this discipline to all ACs in this story.

## Architecture Compliance Rules

**STUB — to be elaborated from architecture section files.** At minimum:
- No `unwrap()` or `expect()` in non-test production code — this story only touches
  test files, so test-file `unwrap()` in async setup blocks is acceptable per the existing
  adv_p02 pattern.
- No `println!` — use `tracing::*!` with structured fields (but test setup code is exempt).
- No new product BCs may be authored by the implementer — this is test-infrastructure only.
- If the refactor touches DTU clone source (e.g., to expose a reset endpoint), route to
  architect + product-owner before modifying DTU clone behavior.

## Library & Framework Requirements

**STUB — to be confirmed at pickup against shipped Cargo.toml.**

Key dependencies (from S-PERF-GATE-001 context and adv_p02 test file):
- `std::sync::LazyLock` (stable since Rust 1.80; workspace is on current stable per `rust-toolchain.toml`)
- `tokio::runtime::Runtime` (tokio version pinned in workspace `Cargo.toml`)
- `Arc` from `std::sync`
- Existing `CrowdstrikeClone` / `ArmisClone` DTU test handle types (from `prism-dtu-crowdstrike` and `prism-dtu-armis` crates)

No new external dependencies should be required by this story.

## File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `crates/prism-bin/tests/adv_p02_e2e_pushdown_pipeline_test.rs` | MODIFY | Only file under modification; refactor internal resets + introduce LazyLock fixtures |

No new files should be required. If DTU clones require a reset-endpoint addition, that
expands scope into `crates/prism-dtu-crowdstrike/` and/or `crates/prism-dtu-armis/` — flag
this as scope expansion and route to architect before proceeding.

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| v1.0 | 2026-06-27 | story-writer | Draft stub registered as Canonical Principle Rule 3 deferral anchor for S-PERF-GATE-001 AC-006/007 (D-1368, 2026-06-26). Closes F-P2R11-MED-001 (phantom file resolving POL-22). |
