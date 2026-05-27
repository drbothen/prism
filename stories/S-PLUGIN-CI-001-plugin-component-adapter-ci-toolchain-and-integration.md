---
document_type: story
story_id: S-PLUGIN-CI-001
title: "Plugin component-adapter CI toolchain + production .prx artifact + end-to-end plugin integration tests"
wave: 1
epic_id: PLUGIN-MIGRATION-001
priority: P1
status: merged
version: "v0.2"
level: "L4"
producer: state-manager
timestamp: "2026-05-23T00:00:00Z"
modified: "2026-05-27"
merged_at: "2026-05-27T15:05:34Z"
merged_via_pr: 159
merged_via_sha: "de1d5db7"
tdd_mode: strict
# BC status: pending PO authorship for BC-2.17.005 and any new boot-error BCs.
# Existing BCs below are sufficient for status=draft; all must be non-empty and
# verified before status transitions to ready per Spec-First Gate S-7.01.
subsystems: [SS-17, SS-01]
# Subsystem anchor justifications:
#   SS-17 (WASM Plugin Runtime, prism-spec-engine plugin/) owns PluginRuntime::load_plugin,
#   the .prx loader, WIT validation, and the boot-step-7.5 error path. EC-006 (missing .prx
#   at boot) and the CI toolchain that produces the binary both fall here.
#   SS-01 (Sensor Adapters, prism-sensors) is touched by the end-to-end double-401 path
#   (EC-009) which runs through PipelineExecutor::issue_request_with_retry wired with the
#   crowdstrike-oauth2 plugin as auth provider — a cross-subsystem integration boundary.
crates_touched: [prism-spec-engine, prism-bin]
target_module: prism-spec-engine
behavioral_contracts:
  - BC-2.17.001  # Plugin Panic Isolation — sandbox isolation invariant covers boot-error
                 #   continuation: a missing .prx triggers a recoverable PluginError, not a
                 #   host panic; host must catch and continue (AC-002 traces here)
  - BC-2.17.006  # Plugin WIT Validation — load_plugin gate; this story's CI-built .prx
                 #   must export the SensorAuth WIT interface and pass at load time (AC-001)
  - BC-2.17.007  # Plugin Manifest Schema Validation — plugin.toml must pass schema gate
                 #   when loaded by the CI-built .prx artifact (AC-001)
  - BC-2.22.001  # Boot Orchestration — step 7.5 plugin-load failure emits ERROR + continues;
                 #   AC-002 closes EC-006 deferral from PLUGIN-MIGRATION-001-E
# BC STATUS NOTE: BC-2.17.001/006/007 and BC-2.22.001 were promoted active at PREREQ-D
# merge (PR #149, develop@ec90fe8f, D-568 2026-05-15). No new BCs required for this stub.
# AC-003 (double-401 / EC-009) is covered by BC-2.17.001 sandbox invariant + BC-2.22.001
# boot continuation. A dedicated BC for double-401 AuthRefreshFailed behavior was authored
# in PREREQ-B (VP-150 / BC-2.16.002 §retry policy); this story exercises that path
# end-to-end — no new BC required, but PO should verify before status=ready.
verification_properties:
  - VP-150   # VP-PLUGIN-005: OAuth2 refresh-on-401 via PipelineExecutor retry path —
             #   AC-003 (EC-009) exercises the double-401 terminal failure case end-to-end.
             #   VP-150 is anchored to S-PLUGIN-PREREQ-B; this story adds the double-401
             #   terminal variant that PLUGIN-MIGRATION-001-E deferred.
depends_on:
  - PLUGIN-MIGRATION-001-E  # This story closes EC-006, EC-009, and MED-001 #[ignore]
                             # deferrals from PLUGIN-MIGRATION-001-E. The .prx artifact
                             # that PLUGIN-MIGRATION-001-E authors must exist before CI
                             # toolchain integration (AC-001) and runtime tests (AC-002/003)
                             # can be written.
blocks: []
# Dependency anchor justification:
#   S-PLUGIN-CI-001 depends on PLUGIN-MIGRATION-001-E because:
#   (a) AC-001 un-ignores test_PLUGIN_MIGRATION_001_E_med_001_built_prx_loads_via_plugin_runtime
#       which was written in PLUGIN-MIGRATION-001-E's scope with #[ignore] citing this story.
#   (b) AC-002 exercises PluginRuntime::load_plugin against a MISSING .prx — requires the .prx
#       artifact build convention (path, manifest, WIT exports) to be established first.
#   (c) AC-003 requires PipelineExecutor wired with the crowdstrike-oauth2 plugin, which is
#       built in PLUGIN-MIGRATION-001-E.
points: 5
# Points justification:
#   - CI toolchain: install wasm-tools at pinned version, configure wasi_snapshot_preview1.wasm
#     fixture, add just build-plugin-crowdstrike-oauth2 recipe: ~0.5 day
#   - tests/fixtures/wasi_snapshot_preview1.wasm check-in + fixture README: ~0.25 day
#   - AC-001: un-#[ignore] + verify existing test passes with built artifact: ~0.25 day
#   - AC-002: new unit test for missing-.prx boot-continue path: ~1 day
#   - AC-003: new integration test for double-401 → AuthRefreshFailed via plugin path: ~1.5 day
#   Total: ~3.5 days = 5 points. Below 13-point cap.
estimated_days: 4
risk: MEDIUM
# Risk justification: wasm-tools + wasi_snapshot_preview1.wasm shim version pinning in CI is
# the primary risk — WASM toolchain compatibility with Wasmtime's component model ABI changes
# across versions. Secondary risk: double-401 test (AC-003) requires DTU clone failure
# injection which is exercised in PLUGIN-MIGRATION-001-E AC-006; build on that pattern.
acceptance_criteria_count: 3
red_gate_tests: 3
estimated_passes: "2-4 LOCAL adversary passes"
holdout_scenarios: []
assumption_validations: []
risk_mitigations:
  - "Pin wasm-tools version in CI and document it in tests/fixtures/README.md to prevent
    silent version drift breaking the component adapter shim."
  - "AC-002 uses a unit test (not a subprocess integration test) to drive the missing-.prx
    code path — avoids SIGBUS risk from deep async boot sequence in test threads."
inputs:
  - "crates/prism-spec-engine/src/plugin/loader.rs"
  - "crates/prism-spec-engine/src/plugin/mod.rs"
  - "crates/prism-spec-engine/src/plugin/discovery.rs"
  - "crates/prism-bin/src/main.rs"
  - ".factory/specs/behavioral-contracts/BC-2.17.001-plugin-panic-isolation.md"
  - ".factory/specs/behavioral-contracts/BC-2.17.006-plugin-wit-validation.md"
  - ".factory/specs/behavioral-contracts/BC-2.17.007-plugin-manifest-schema-validation.md"
  - ".factory/specs/behavioral-contracts/BC-2.22.001-boot-orchestration.md"
  - ".factory/stories/PLUGIN-MIGRATION-001-E-crowdstrike-oauth2-refresh-on-401-prx-wasm-plugin.md"
  - ".github/workflows/ci.yml"
  - "Justfile"
input-hash: "[initial-stub]"
traces_to: []
cycle: "v1.0.0-greenfield"
phase: 3
---

# S-PLUGIN-CI-001: Plugin Component-Adapter CI Toolchain + Production .prx Artifact + End-to-End Plugin Integration Tests

**Story ID:** S-PLUGIN-CI-001
**Status:** draft
**Version:** v0.1
**Wave:** 1 (depends on PLUGIN-MIGRATION-001-E; dispatched after 001-E merges)

---

## Summary

This story closes three deferred scope items from PLUGIN-MIGRATION-001-E that share a common
dependency: a real `.prx` artifact built via the wasmtime component adapter toolchain
(wasm-tools + `wasi_snapshot_preview1.wasm` shim) available in CI.

**Deferrals closed:**

1. **MED-001 `#[ignore]` removal** — `test_PLUGIN_MIGRATION_001_E_med_001_built_prx_loads_via_plugin_runtime`
   was written in PLUGIN-MIGRATION-001-E with `#[ignore]` citing this story as the unblocking
   condition. AC-001 removes the `#[ignore]` and wires the CI build step.

2. **EC-006** (plugin binary missing at boot) — AC-002 adds a test asserting that
   `PluginRuntime::load_plugin` failure at boot step 7.5 emits ERROR-level tracing and
   continues without panic, as required by BC-2.22.001 §Sequencing Invariant.

3. **EC-009** (double-401 → AuthRefreshFailed) — AC-003 drives
   `PipelineExecutor::issue_request_with_retry` through the double-401 terminal failure
   case via the crowdstrike-oauth2 plugin auth path, asserting
   `SpecEngineError::AuthRefreshFailed` as the outcome.

---

## Behavioral Contracts

| BC ID | Title | Role in This Story |
|-------|-------|-------------------|
| BC-2.17.001 | Plugin Panic Isolation | **Boot error containment** — missing .prx triggers recoverable PluginError; host does not panic. AC-002 traces here. |
| BC-2.17.006 | Plugin WIT Validation | **Load gate** — CI-built .prx must export SensorAuth WIT interface; AC-001 verifies the un-ignored test passes this gate. |
| BC-2.17.007 | Plugin Manifest Schema Validation | **Load gate** — plugin.toml manifest must pass schema validation; AC-001 verifies via the un-ignored test. |
| BC-2.22.001 | Boot Orchestration | **Step 7.5 error path** — plugin-load failure emits ERROR + boot continues; AC-002 closes EC-006 deferral. |

---

## Token Budget Estimate

| Artifact | Estimated Tokens |
|----------|-----------------|
| This story spec | ~3,000 |
| BC-2.17.001/006/007 (sandbox + WIT + manifest, reads) | ~3,000 |
| BC-2.22.001 (boot orchestration, full read) | ~2,000 |
| PLUGIN-MIGRATION-001-E story (deferral source context) | ~5,000 |
| prism-spec-engine/src/plugin/loader.rs (partial read) | ~3,000 |
| prism-spec-engine/src/plugin/mod.rs (partial read) | ~2,000 |
| ci.yml + Justfile (recipe + step additions) | ~1,500 |
| Existing plugin integration test file (test_PLUGIN_MIGRATION_001_E_med_001) | ~2,000 |
| prism-dtu-crowdstrike routes (double-401 DTU config) | ~1,500 |
| **Total estimate** | **~23,000** |
| Agent context window (claude-sonnet-4-6) | ~200,000 |
| **% of context window** | **~12%** |

Well within the 20-30% target.

---

## Acceptance Criteria

Each AC traces to its BC clause and includes the Red Gate test name per SID-1 §5.

### AC-001: wasm-tools + wasi_snapshot_preview1.wasm fixture available in CI; MED-001 #[ignore] removed (traces to BC-2.17.006 postcondition — WIT export validated at load; BC-2.17.007 postcondition — manifest schema gate passes)

`tests/fixtures/wasi_snapshot_preview1.wasm` is checked into the repository (sourced from
an upstream wasmtime release; the release tag and download URL are documented in
`tests/fixtures/README.md`).

CI installs `wasm-tools` at a pinned version (document the version in `tests/fixtures/README.md`
and in the CI step). CI runs `just build-plugin-crowdstrike-oauth2` (or an equivalent recipe
in the Justfile) after `cargo build` to produce the `.prx` artifact. The build recipe is
deterministic and exits 0 on all CI platforms.

`test_PLUGIN_MIGRATION_001_E_med_001_built_prx_loads_via_plugin_runtime` has its `#[ignore]`
annotation removed. The test passes on CI with the CI-built `.prx` artifact in place.

**Red Gate Test:** `test_PLUGIN_MIGRATION_001_E_med_001_built_prx_loads_via_plugin_runtime`
(un-`#[ignore]`-d from PLUGIN-MIGRATION-001-E; this is the existing test that was written
with `#[ignore]` citing S-PLUGIN-CI-001).

### AC-002: EC-006 boot-with-missing-prx behavior covered (traces to BC-2.22.001 postcondition — boot step 7.5 failure emits ERROR and continues; BC-2.17.001 invariant — host does not panic on plugin load failure)

A non-`#[ignore]`'d unit test asserts the following scenario: when
`PluginRuntime::load_plugin` is called with a path to a non-existent `.prx` file, it
returns `Err(PluginError::PrxNotFound)` (or the equivalent error variant per
`plugin/mod.rs` — implementer must verify the exact error variant name). The boot step 7.5
logic (in `prism-bin/src/main.rs` or equivalent boot sequence) handles this error by
emitting an ERROR-level tracing event with `event_type = "plugin_load_failed"` (or the
event name registered in BC-2.16.002 — implementer must add a BC-2.16.002 catalog row
for this event if absent; defer to product-owner during implementation cascade) and
continuing boot without panicking.

The test is a unit test (not a subprocess integration test) driving the missing-.prx code
path without spawning the full `prism start` binary, to avoid SIGBUS risk on macOS aarch64
from deep async boot stacks in test threads.

**Red Gate Test:** `test_S_PLUGIN_CI_001_002_missing_prx_at_boot_continues_with_error_log`

**Cross-reference:** This AC explicitly closes PLUGIN-MIGRATION-001-E EC-006 deferral
("wasm32 Guest impl / WAT-fixture / integration tests (not closed in FB-IMPL-4)").

### AC-003: EC-009 double-401 → AuthRefreshFailed behavior covered (traces to BC-2.17.001 invariant — sandbox error paths do not panic host; BC-2.22.001 postcondition — plugin-provided auth participates in PipelineExecutor retry path)

A test drives `PipelineExecutor::issue_request_with_retry` end-to-end against the
CrowdStrike DTU clone configured to return HTTP 401 on BOTH the initial request AND the
post-refresh retry. The crowdstrike-oauth2 plugin (built by PLUGIN-MIGRATION-001-E) is
wired as the auth provider. The test asserts:

1. The final outcome is `Err(SpecEngineError::AuthRefreshFailed)` (or the equivalent
   error per `error-taxonomy.md` E-PIPELINE-001 namespace — implementer must verify the
   canonical error code; AC-5 of S-PLUGIN-PREREQ-B established the abort condition).
2. The host emits `event_type = "plugin.auth_refresh_failed"` (or the equivalent
   registered event) at ERROR level — implementer must add a BC-2.16.002 catalog row if
   absent; defer to product-owner during implementation cascade.
3. No panic occurs in the host process.

If the `#[ignore]` version of this test was already written in PLUGIN-MIGRATION-001-E's
scope, un-`#[ignore]` it and verify it passes. If it was not written, implement it fresh.

**Red Gate Test:** `test_S_PLUGIN_CI_001_003_double_401_returns_auth_refresh_failed`

**Cross-reference:** This AC explicitly closes PLUGIN-MIGRATION-001-E EC-009 deferral
("wasm32 Guest impl / WAT-fixture / integration tests (not closed in FB-IMPL-4)").

---

## Tasks

High-level TDD order. Full task breakdown happens during ready-for-implementation refinement.

1. **Read source files first** — `plugin/loader.rs`, `plugin/mod.rs`, `plugin/discovery.rs`,
   `prism-bin/src/main.rs` (boot sequence), `ci.yml`, `Justfile`. Verify existing MED-001
   test file location and exact `#[ignore]` annotation.

2. **Add `tests/fixtures/wasi_snapshot_preview1.wasm`** — source from upstream wasmtime
   release. Write `tests/fixtures/README.md` documenting release tag, download URL, and
   pinned wasm-tools version.

3. **Add `just build-plugin-crowdstrike-oauth2` recipe** — wraps `cargo build` with
   `--target wasm32-wasi` (or the correct WASM component target) for the
   `crowdstrike-oauth2-plugin` crate and runs `wasm-tools component new` to produce the
   `.prx` artifact. Verify the recipe produces a deterministic artifact path.

4. **Wire CI step** — add `wasm-tools` install + `just build-plugin-crowdstrike-oauth2`
   step to `ci.yml` after the `cargo build` step and before the `cargo nextest run` step.

5. **Write Red Gate tests for AC-002 and AC-003** (stub phase — tests must FAIL before
   implementation makes them pass). AC-001's Red Gate test already exists with `#[ignore]`.

6. **Remove `#[ignore]` from MED-001 test** and verify it passes with the CI-built artifact.

7. **Implement AC-002** — add missing-.prx error path handling in the boot step 7.5 caller
   if absent; verify BC-2.16.002 catalog row for `plugin_load_failed` event (or route to
   PO if absent).

8. **Implement AC-003** — configure DTU clone double-401 failure injection; verify
   `PipelineExecutor::issue_request_with_retry` returns `SpecEngineError::AuthRefreshFailed`.

9. **Pre-push gate** — `just check` GREEN workspace-wide. No `--no-verify`.

---

## Edge Cases

TBD during ready-for-implementation refinement. At minimum the following should be enumerated:

- What happens when the CI-built `.prx` artifact is present but corrupt (wrong magic bytes)?
- What happens when `wasm-tools` is not installed on a developer's local machine (non-CI)?
- Is the double-401 test hermetic (does it require the DTU clone process running)?

---

## Previous Story Intelligence

N/A for this stub — full previous story intelligence populated during ready-for-implementation
refinement once PLUGIN-MIGRATION-001-E has completed its implementation cascade.

Key lessons from PLUGIN-MIGRATION-001-E cascade that apply:

- `allowed_urls` in plugin manifests is `Vec<String>` (not `Option`), default-deny.
- `format_version = 1` is the current `CURRENT_SUPPORTED_VERSION` cap.
- Boot step 7.5 sequencing: plugins load between storage init (step 7) and query-engine
  init (step 8). Error at step 7.5 must NOT block step 8.
- `MAX_REQUESTS_PER_PIPELINE = 10_000` — plugin outbound HTTP counts against this cap.
- DTU clone failure injection pattern (`POST /dtu/configure {"auth_mode": "reject"}`) is
  established in PLUGIN-MIGRATION-001-E AC-006; reuse that pattern for AC-003.

---

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| Plugin WASM binary compiled to `wasm32-wasi` or equivalent component target | BC-2.17.006 WIT validation requires Component Model ABI | CI `just build-plugin-crowdstrike-oauth2` step |
| `wasi_snapshot_preview1.wasm` pinned to a specific upstream release tag | Avoid silent WASM ABI drift | `tests/fixtures/README.md` documents release tag |
| `wasm-tools` pinned to a specific version in CI | Deterministic component adapter shim output | CI install step + `tests/fixtures/README.md` |
| Missing-`.prx` at boot MUST emit ERROR + continue (not panic or abort boot) | BC-2.22.001 §Sequencing Invariant + BC-2.17.001 | AC-002 Red Gate test |
| Double-401 terminal failure MUST propagate as `SpecEngineError::AuthRefreshFailed` | PREREQ-B AC-5; BC-2.16.002 retry policy | AC-003 Red Gate test |
| New `tracing::*!(event_type=...)` sites MUST have BC-2.16.002 catalog row | PG-LP11-001; SAP-1 standing adversary probe | Adversary SAP-1 sweep on every pass |
| No `println!` in production code | CLAUDE.md conventions | `clippy::print_stdout` lint |

### Forbidden Dependencies

Boot-step-7.5 error-path code in `prism-bin` MUST NOT gain a direct dependency on
`prism-sensors` to handle the missing-.prx case — the error is surfaced via
`PluginError` from `prism-spec-engine` only. If this crate gains a dependency on
`prism-sensors`, the build MUST fail (perimeter violation).

---

## Library and Framework Requirements

| Library | Version | Justification |
|---------|---------|---------------|
| `wasmtime` | per `prism-spec-engine/Cargo.toml` workspace pin | Plugin runtime; do not add separate wasmtime dep |
| `wasm-tools` (CLI, not a Rust dep) | pinned version in CI | Component adapter shim; document version in `tests/fixtures/README.md` |

Do NOT pin new Rust library versions. Use workspace-inherited versions.

---

## File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `tests/fixtures/wasi_snapshot_preview1.wasm` | CREATE (binary) | Source from upstream wasmtime release; document in README |
| `tests/fixtures/README.md` | CREATE | Documents wasm-tools version pin, release tag, download URL |
| `Justfile` | MODIFY | Add `build-plugin-crowdstrike-oauth2` recipe |
| `.github/workflows/ci.yml` | MODIFY | Add wasm-tools install step + plugin build step |
| `crates/prism-spec-engine/tests/crowdstrike_oauth2_plugin_tests.rs` | MODIFY | Remove `#[ignore]` from MED-001 test; add AC-002 + AC-003 tests |

---

## Cross-References

**Closes deferrals from PLUGIN-MIGRATION-001-E:**

- **EC-006** (plugin binary missing at boot) — PLUGIN-MIGRATION-001-E EC table row: "wasm32
  Guest impl / WAT-fixture / integration tests (not closed in FB-IMPL-4)". Closed by AC-002
  (`test_S_PLUGIN_CI_001_002_missing_prx_at_boot_continues_with_error_log`).

- **EC-009** (double 401 → SpecEngineError::AuthRefreshFailed) — PLUGIN-MIGRATION-001-E EC
  table row: "wasm32 Guest impl / WAT-fixture / integration tests (not closed in FB-IMPL-4)".
  Closed by AC-003 (`test_S_PLUGIN_CI_001_003_double_401_returns_auth_refresh_failed`).

- **MED-001 `#[ignore]` removal** — `test_PLUGIN_MIGRATION_001_E_med_001_built_prx_loads_via_plugin_runtime`
  was written in PLUGIN-MIGRATION-001-E with `#[ignore]` citing "S-PLUGIN-CI-001" as the
  unblocking condition per F-LP5-HIGH-001 option (a) deferral. Closed by AC-001.

**F-LP7-MED-002 deferral origin:** The EC-006 and EC-009 scope items were surfaced in
PLUGIN-MIGRATION-001-E LOCAL adversary pass-7 as F-LP7-MED-002. This story is the
resolution target.

---

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| v0.1 | 2026-05-23 | story-writer | Initial stub — created to honor SID-1 §5 deferral discipline; full task breakdown deferred to ready-for-implementation refinement after PLUGIN-MIGRATION-001-E merges. |
