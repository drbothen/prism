---
type: research
topic: test-suite-performance-baseline
initiative: T-PERF-PROFILE
decision: D-1434
date: 2026-06-30
baseline_commit: 8bc0404e
branch: develop
machine: macOS darwin 25.5.0 (Apple Silicon, 16 logical CPUs)
nextest_wall_clock: 585.835s
just_check_estimated: ~798s
---

# Test Suite Performance Baseline Profile — 2026-06-30

Measurement target: `develop@8bc0404e` (current develop HEAD at profile time).
All measurements on a warm build (77 GB `target/` cache, RUSTFLAGS="" fingerprint).
Machine: macOS darwin 25.5.0, 16 logical CPUs.
No code modifications were made; this is measurement-only.

---

## 1. Per-Stage Wall-Clock (just check)

Measured by running each stage of the `just check` recipe separately on a warm build.
"Warm" = all prior compilation artifacts exist from a recent build; only incremental changes are recompiled.

| Stage | Command | Wall-Clock (warm) | Notes |
|-------|---------|-------------------|-------|
| fmt | `cargo fmt --check` | 1.49s | Near-instant (no compilation) |
| clippy | `cargo clippy --all-features -- -D warnings` | 43.85s | Incremental; 126% CPU (2 cores) |
| nextest build | compile test binaries (inside `cargo nextest run`) | ~157s | RUSTFLAGS="" differs from clippy → rebuild of test targets |
| nextest test execution | 4976 tests, `--profile prepush` | **585.84s** | See §2 |
| doctest (warm, sequential) | `cargo test --workspace --all-features --doc` | ~8s | Shares RUSTFLAGS="" cache with nextest; just doctest execution |
| check-layout | `scripts/check-crate-layout.sh` | 0.16s | Shell script, trivial |
| check-non-exhaustive | `scripts/check-non-exhaustive.sh` | 2.87s | 87 violations verified |
| **TOTAL (just check)** | | **~798s (≈ 13.3 minutes)** | Sequential execution |

**RUSTFLAGS build overhead note:** `cargo clippy` uses the default RUSTFLAGS fingerprint; `cargo nextest run` uses `RUSTFLAGS=""`. These are different fingerprints, so nextest must rebuild all test binary targets after clippy (→ the ~157s build phase). If the workspace was previously compiled under `RUSTFLAGS=""` (e.g., on a second consecutive `just check` run), the nextest build drops to ~5-10s (incremental). The 157s overhead is a real penalty for the typical `just check` invocation following a clippy-only run.

---

## 2. Per-Binary Nextest Timing

Total: 4976 tests run, 60 skipped (`#[ignore]`). Wall-clock: **585.84s**. Total serial time (if all sequential): 4481.5s (~74.7 min). Effective parallelism: **7.65x**.

### 2a. Top 25 Binaries by Serial Time

| Rank | Binary | Tests | Serial Sum | Max | Avg | Cap Group |
|------|--------|-------|-----------|-----|-----|-----------|
| 1 | `prism-spec-engine::plugin_integration_tests` | 34 | 277.4s | 14.94s | 8.16s | NONE |
| 2 | `prism-spec-engine::pipeline_http_integration` | 27 | 247.0s | 9.87s | 9.15s | NONE |
| 3 | `prism-spec-engine` (lib inline) | 175 | 219.6s | 11.13s | 1.25s | NONE |
| 4 | `prism-spec-engine::plugin_tests` | 25 | 204.7s | 14.62s | 8.19s | NONE |
| 5 | `prism-bin::bc_2_01_013_spec_driven_adapter` | 25 | 179.2s | 10.64s | 7.17s | `bc-2-01-013-serial` (max-threads=1) |
| 6 | `prism-bin::plugin_boot_tests` | 23 | 159.5s | 9.74s | 6.94s | NONE |
| 7 | `prism-dtu-armis::harness_tests` | 40 | 125.9s | 9.37s | 3.15s | `dtu-cap` (max-threads=4) |
| 8 | `prism-dtu-claroty::harness_tests` | 56 | 118.7s | 2.45s | 2.12s | `dtu-cap` (max-threads=4) |
| 9 | `prism-spec-engine::enrichment_pivot_002_tests` | 41 | 112.9s | 9.64s | 2.75s | NONE |
| 10 | `prism-spec-engine::crowdstrike_oauth2_plugin_tests` | 19 | 103.8s | 9.43s | 5.46s | NONE |
| 11 | `prism-ocsf::spec_driven_mapper_fixtures` | 13 | 94.6s | 8.88s | 7.27s | NONE |
| 12 | `prism-spec-engine::bc_2_11_007_pushdown_test` | 11 | 85.8s | 9.71s | 7.80s | NONE |
| 13 | `prism-dtu-crowdstrike::harness_tests` | 47 | 83.4s | 1.97s | 1.77s | `dtu-cap` (max-threads=4) |
| 14 | `prism-dtu-armis::f_p2_route_output_tests` | 9 | 82.5s | 9.27s | 9.17s | `dtu-cap` (max-threads=4) |
| 15 | `prism-dtu-armis::cr017_tag_alert_org_id_guard` | 8 | 74.4s | 9.77s | 9.30s | `dtu-cap` (max-threads=4) |
| 16 | `prism-dtu-armis::ac_5_missing_bearer_403` | 7 | 72.1s | 10.56s | 10.30s | `dtu-cap` (max-threads=4) |
| 17 | `prism-dtu-jira::harness_tests` | 39 | 71.6s | 2.11s | 1.84s | `dtu-cap` (max-threads=4) |
| 18 | `prism-bin::adv_p02_e2e_pushdown_pipeline_test` | 8 | 63.6s | 10.88s | 7.95s | `adv-p02-serial` (max-threads=1) |
| 19 | `prism-dtu-harness::logical_isolation_test` | 34 | 59.9s | 2.15s | 1.76s | `dtu-cap` (max-threads=4) |
| 20 | `prism-spec-engine::pipeline_oauth_retry` | 6 | 58.4s | 9.88s | 9.73s | NONE |
| 21 | `prism-dtu-armis::cr023_activity_risk_org_id_guard` | 6 | 56.1s | 9.77s | 9.35s | `dtu-cap` (max-threads=4) |
| 22 | `prism-dtu-jira::fidelity` | 28 | 53.3s | 2.05s | 1.90s | `dtu-cap` (max-threads=4) |
| 23 | `prism-dtu-claroty` (lib inline) | 22 | 48.7s | 2.41s | 2.22s | `dtu-cap` (max-threads=4) |
| 24 | `prism-dtu-pagerduty::harness_tests` | 28 | 48.6s | 1.93s | 1.74s | `dtu-cap` (max-threads=4) |
| 25 | `prism-bin::infusion_boot_integration` | 5 | 48.4s | 9.78s | 9.68s | NONE |

**Comparison baseline — prism-query in-process unit tests:** 947 tests, 41.9s serial, avg 0.044s per test. This is the healthy reference point; tests at 8-10x this average are oversubscribed.

### 2b. Top 30 Slowest Individual Tests

| Rank | Time | Binary | Test Name |
|------|------|--------|-----------|
| 1 | 14.94s | `prism-spec-engine::plugin_integration_tests` | `test_BC_2_17_004_cpu_timeout_enforced_infinite_loop` |
| 2 | 14.62s | `prism-spec-engine::plugin_tests` | `test_BC_2_17_004_ac3_infinite_loop_returns_err_timeout` |
| 3 | 11.13s | `prism-spec-engine` | `pipeline::execute_step_tests::test_BC_2_16_002_execute_step_emits_auth_initial_acquired_with_step_name_field` |
| 4 | 11.06s | `prism-spec-engine` | `pipeline::execute_step_tests::test_BC_2_16_002_auth_refresh_succeeded_emits_event_with_step_name` |
| 5-11 | 10.97-11.03s | `prism-spec-engine` | `pipeline::execute_step_tests::test_BC_2_16_002_*` (6 more tests) |
| 12 | 10.88s | `prism-bin::adv_p02_e2e_pushdown_pipeline_test` | `test_adv_p08_med001_armis_inclusive_boundary_via_run_materialization_pipeline` |
| 13-26 | 10.10-10.77s | `prism-spec-engine`, `prism-dtu-armis::*` | Various WASMtime + DTU armis tests |
| 27 | 9.93s | `prism-dtu-armis::bc_2_06_019_scenario_progression` | `test_BC_2_06_019_armis_device_cves_first_stagemask_served_route` |
| 28-30 | 9.87-9.93s | `prism-spec-engine`, `prism-sensors` | Various wiremock + WASMtime tests |

**Root cause of the 14.x second tests:** Tests `test_BC_2_17_004_*` enforce a 5-second epoch timeout on an infinite-loop WASMtime plugin. The 14.9s total = PluginRuntime::new() overhead (~9s under load) + 5s timeout + overhead. The Engine::new() overhead is inflated by CPU contention from concurrent WASMtime inits.

---

## 3. WASMtime Engine Init Cost Quantification

### 3a. Engine Init Sites

`wasmtime::Engine::new(&config)` is called once per `PluginRuntime::new_with_audit_sink()`. This is the only Engine::new() call site in production code (`crates/prism-spec-engine/src/plugin/mod.rs:154`). Tests that call `PluginRuntime::new()` pay this cost once per test function (NO LazyLock sharing exists).

### 3b. Per-Binary Engine Init Count

| Binary | PluginRuntime::new() calls | Tests in binary | Engine inits under full load |
|--------|---------------------------|-----------------|------------------------------|
| `plugin_tests` | 17+ (one per test, most tests) | 25 | 17+ |
| `plugin_integration_tests` | 9 (via PluginRuntime or direct Engine::new) | 34 | 9+ |
| `crowdstrike_oauth2_plugin_tests` | 2 | 19 | 2 |
| `enrichment_pivot_002_tests` | 7 | 41 | 7 |
| `infusion_tests` | 5 (helpers) | 54 | 3 active (others are fast unit tests) |
| `ocsf::spec_driven_mapper_fixtures` | 1 per test via `empty_plugin_runtime()` | 13 | 12+ |
| `prism-bin::plugin_boot_tests` | ~19 tests (boot integration tests) | 23 | ~19 |
| `prism-bin::infusion_boot_integration` | 5 | 5 | 5 |

**Total Engine inits during full workspace run:** approximately 70-90 concurrent/sequential Engine::new() calls across these 8 uncapped binaries.

### 3c. Engine Init Timing: Isolated vs Under Load

| Condition | PluginRuntime::new() cost | Source |
|-----------|---------------------------|--------|
| In isolation (single binary) | ~1-2s estimated | Derived: `test_AC3_infinite_loop` total 14.6s − 5s timeout − ~0.3s logic ≈ ~9.3s under load; isolation cost derives from expected Cranelift init time |
| Under full workspace concurrency | ~8-9s (measured) | `test_BC_2_17_004_ac3` total = 14.6s = ~9s init + 5s timeout |
| Oversubscription factor | ~5-9x inflation | CPU contention from concurrent WASMtime + DTU + wiremock processes |

**Root cause:** wasmtime v44 with `wasm_component_model(true)` + `epoch_interruption(true)` triggers Cranelift JIT compiler initialization. Under CPU contention (16 logical CPUs, ~10-16 concurrent test binaries, each with multi-threaded tokio runtimes), Cranelift initialization serializes on global compiler state, causing each Engine::new() to wait for others.

**Total WASMtime overhead (serial):** 1022.7s across 8 uncapped binaries.

---

## 4. Oversubscription Map

### 4a. Currently Capped Groups (nextest.toml prepush profile)

| Group | Filter | max-threads | Rationale |
|-------|--------|-------------|-----------|
| `dtu-cap` | `package(/^prism-dtu-/)` | 4 | S-PERF-GATE-004: prevents tokio thread storm (4×16=64 threads/16 cores=4x) |
| `serial-subprocess` | `binary(signal_handlers)` | 1 | S-PERF-GATE-001: RocksDB mmap SIGSEGV under parallelism |
| `adv-p02-serial` | `binary(adv_p02_e2e_pushdown_pipeline_test)` | 1 | S-PERF-GATE-002: DTU+RocksDB contention |
| `bc-2-01-013-serial` | `binary(bc_2_01_013_spec_driven_adapter)` | 1 | S-PERF-GATE-003: wiremock concurrent-startup contention |

### 4b. Uncapped Binaries with High CPU or I/O Load

These binaries run under default concurrency (= 16 on this machine), competing with all DTU processes and each other:

| Binary | Load Type | Avg Per-Test | Serial Sum | Recommended Cap |
|--------|-----------|-------------|-----------|-----------------|
| `prism-spec-engine::plugin_integration_tests` | WASMtime (Cranelift JIT) | 8.16s | 277.4s | `spec-engine-wasm-cap` (max-threads=4) |
| `prism-spec-engine::plugin_tests` | WASMtime (Cranelift JIT) | 8.19s | 204.7s | `spec-engine-wasm-cap` (max-threads=4) |
| `prism-spec-engine::pipeline_http_integration` | wiremock HTTP | 9.15s | 247.0s | `spec-engine-http-cap` (max-threads=4) |
| `prism-spec-engine::crowdstrike_oauth2_plugin_tests` | WASMtime | 5.46s | 103.8s | `spec-engine-wasm-cap` (max-threads=4) |
| `prism-spec-engine::enrichment_pivot_002_tests` | WASMtime | 2.75s | 112.9s | `spec-engine-wasm-cap` (max-threads=4) |
| `prism-spec-engine::bc_2_11_007_pushdown_test` | DTU clones (uncapped!) | 7.80s | 85.8s | `spec-engine-http-cap` (max-threads=4) |
| `prism-spec-engine::pipeline_oauth_retry` | wiremock HTTP | 9.73s | 58.4s | `spec-engine-http-cap` (max-threads=4) |
| `prism-ocsf::spec_driven_mapper_fixtures` | WASMtime | 7.27s | 94.6s | `spec-engine-wasm-cap` (max-threads=4) |
| `prism-bin::plugin_boot_tests` | WASMtime | 6.94s | 159.5s | `spec-engine-wasm-cap` (max-threads=4) |
| `prism-bin::infusion_boot_integration` | WASMtime | 9.68s | 48.4s | `spec-engine-wasm-cap` (max-threads=4) |

**Critical gap:** `prism-spec-engine::bc_2_11_007_pushdown_test` starts CrowdStrike + Armis DTU clones (it's a pushdown integration test against live in-process DTU servers). The `dtu-cap` filter only applies to `package(/^prism-dtu-/)` — packages in the `prism-dtu-*` namespace. `prism-spec-engine` is NOT captured by this filter. The 11 pushdown tests (avg 7.80s each) run completely uncapped, adding to DTU server oversubscription.

---

## 5. Wall-Clock Sleep / Polling Inventory

Tests whose PASS/FAIL depends on real elapsed time (flaky-under-load class):

### 5a. Deliberate Wall-Clock Sleeps in Tests

| File | Sleep | Purpose | Removable? |
|------|-------|---------|------------|
| `crates/prism-query/tests/execute_integration_tests.rs:2129` | `tokio::time::sleep(2s)` in `test_AC_timeout_returns_query_timeout_error` | SlowAdapter delays 2s so 1s timeout fires | NO — intentional timeout test |
| `crates/prism-spec-engine/tests/enrichment_pivot_002_tests.rs:930` | `std::thread::sleep(50ms)` | Rate-limiter test timing | Possibly reducible |
| `crates/prism-query/src/infusion_udf.rs:1368` | `std::thread::sleep(200ms)` | PRODUCTION: UDF polling delay | Reduce or make configurable |

### 5b. Stage-Clock-Dependent Tests (wall-clock race class)

These tests do NOT sleep but their pass/fail depends on `Utc::now()` at HTTP request time matching an expected stage index:

| Test | Binary | Pattern | Risk Under Load |
|------|--------|---------|-----------------|
| `test_BC_2_06_019_armis_primary_device_stage_visibility` | `prism-dtu-armis::bc_2_06_019_scenario_progression` | `scenario_start = now - 10s` (stage 0 threshold: 60s) | Under CPU load, HTTP request may be delayed 50s+ after setup → stage 0 becomes stage 1 |
| `test_BC_2_06_019_armis_device_cves_first_stagemask_served_route` | `prism-dtu-armis::bc_2_06_019_scenario_progression` | `now - N` offset | Same risk |
| `test_BC_2_06_019_crowdstrike_containment_visible_at_stage4_only` | `prism-dtu-crowdstrike::bc_2_06_019_scenario_progression` | `now - 200s` (stage 2), `now - 700s` (stage 4) | Stage boundaries wider (180s/360s/600s); lower risk but nonzero |
| DTU cyberint stagemask tests | `prism-dtu-cyberint::bc_2_06_019_scenario_stagemask` | Stage-clock offset | Same risk class |

**Documented evidence:** `prism-dtu-armis::bc_2_06_019_scenario_progression` measured at 55.846s under full workspace load vs 0.412s in isolation (commit 0a9fc5e7, S-PERF-GATE-004 comment). With a 60s stage-0 threshold and a 10s offset buffer, the test fails when CPU contention delays the HTTP request by >50s.

**Current measured time under dtu-cap=4:** These tests show 9.79-9.93s per test in the profiling run. The `now - 10s` buffer provides 50s headroom before stage-0 overflows. Under dtu-cap=4, this was safe in this run, but the headroom is thin.

### 5c. Polling Loops (deadline-based, not sleep-based)

| File | Pattern | Timeout | Impact |
|------|---------|---------|--------|
| `prism-bin/tests/helpers/mod.rs:176,207,1197,1812,1972` | `deadline = Instant::now() + Duration::from_secs(30)` then poll loop | 30s each | Subprocess wait; only in non-ignored tests that run the prism binary |
| `prism-bin/tests/e2e_smoke.rs:634,659` | `deadline = Instant::now() + 5s`, 100ms polls | 5s | In `#[ignore]`'d E2E tests — no impact on normal runs |

---

## 6. Subprocess Overhead

### 6a. Test Files That Spawn `prism` as Subprocess

| File | Spawn Count | Test Count | Est. Overhead/Test |
|------|------------|------------|-------------------|
| `bc_2_21_001_org_registry_init.rs` | 8 spawns | 14 tests | ~200-400ms/test |
| `bc_2_03_013_credential_init.rs` | 4 spawns | 20 tests | ~200-400ms/test |
| `bc_2_22_001_boot_orchestration.rs` | 8 spawns | 16 tests | ~200-800ms/test |
| `cli_subcommands.rs` | 8 spawns | 14 tests | ~200-400ms/test |
| `bc_2_06_011_config_load.rs` | 9 spawns | 16 tests | ~200-400ms/test |
| `bc_2_10_006_mcp_stdout_purity.rs` | 2 spawns | 2 tests | ~200-400ms/test |
| `bc_2_05_012_audit_init.rs` | 5 spawns | 10 tests | ~200-400ms/test |
| `signal_handlers.rs` (serialized) | ~6 spawns | 6 tests | ~1-5s/test |

**Ignored (not in normal nextest run):**
- `e2e_smoke.rs`: 13 tests × subprocess spawn, all `#[ignore]`'d
- `e2e_multi_org.rs`: 10 tests × subprocess spawn, all `#[ignore]`'d
- `bc_2_03_007_credential_set_no_echo.rs`: 1 subprocess test, `#[ignore]`'d
- `bc_2_03_007_credential_set_org_id_keyed.rs`: 1 subprocess test, `#[ignore]`'d

**Aggregate subprocess overhead estimate (non-ignored):** ~50-65 subprocess invocations × 300-500ms average = ~20-35s serial overhead. These tests are already fairly fast because each spawns prism once and checks output synchronously (no long boot wait). Not a primary optimization target.

### 6b. In-Process vs Subprocess Distribution

- Total tests: 4976
- In-process unit tests (prism-query, prism-core, prism-mcp, etc.): ~3600 tests, avg ~0.05s
- In-process integration tests with wiremock/DTU clones: ~1200 tests, avg ~2-4s
- Subprocess tests: ~100 tests, avg ~0.5-2s

---

## 7. Prioritized Recommendations

Ranked by (wall-clock impact) × (safety / effort ratio):

### REC-1 [CRITICAL, Easy, ~150-200s savings] — Add nextest cap groups for uncapped WASMtime binaries

**Problem:** 8 WASMtime-heavy binaries (total 1022.7s serial) run under default concurrency (16 on this machine), competing with dtu-cap=4 processes and each other for CPU. Cranelift JIT initialization inflates from ~1-2s (isolated) to ~8-9s (under load) per Engine::new() call.

**Fix:** Add two new cap groups to `.config/nextest.toml` prepush + ci profiles:

```toml
[test-groups]
spec-engine-wasm-cap = { max-threads = 4 }
spec-engine-http-cap = { max-threads = 4 }

[[profile.prepush.overrides]]
# WASMtime-heavy spec-engine + ocsf + prism-bin binaries: cap at 4 to reduce
# Cranelift JIT init contention. These binaries call PluginRuntime::new() per test.
filter = 'binary(plugin_integration_tests) | binary(plugin_tests) | binary(crowdstrike_oauth2_plugin_tests) | binary(enrichment_pivot_002_tests) | binary(spec_driven_mapper_fixtures) | binary(plugin_boot_tests) | binary(infusion_boot_integration)'
test-group = 'spec-engine-wasm-cap'

[[profile.prepush.overrides]]
# HTTP/wiremock-heavy spec-engine binaries: cap at 4 to reduce OS socket contention.
filter = 'binary(pipeline_http_integration) | binary(pipeline_oauth_retry) | binary(bc_2_11_007_pushdown_test) | binary(bc_2_16_002_crowdstrike_two_step)'
test-group = 'spec-engine-http-cap'
```

**Estimated savings:** With max 4 concurrent WASMtime inits (vs uncontrolled ~10-16), per-init time drops from ~8-9s to ~3-4s. For `plugin_tests` (25 tests × avg reduction 5s): −125s serial. For `plugin_integration_tests` (34 tests × avg reduction 4s): −136s serial. Wall-clock improvement depends on critical path, but −150-200s wall-clock is achievable.

**Safety:** No code changes; nextest.toml config only. Zero functional risk.

---

### REC-2 [HIGH, Easy, ~120-180s savings] — Shared `PluginRuntime` via `LazyLock` per test binary

**Problem:** Every test function that needs a `PluginRuntime` calls `PluginRuntime::new()`, paying the Engine::new() Cranelift JIT cost each time. There is no shared state across tests in the same binary.

**Fix:** In each of the 5 spec-engine test files (`plugin_tests.rs`, `plugin_integration_tests.rs`, `crowdstrike_oauth2_plugin_tests.rs`, `enrichment_pivot_002_tests.rs`, `ocsf/spec_driven_mapper_fixtures.rs`) and 2 prism-bin files (`plugin_boot_tests.rs`, `infusion_boot_integration.rs`):

```rust
// Add to top of test file:
use std::sync::LazyLock;

static TEST_PLUGIN_RUNTIME: LazyLock<Arc<PluginRuntime>> = LazyLock::new(|| {
    Arc::new(
        PluginRuntime::new(build_test_http_client())
            .expect("LazyLock: PluginRuntime::new must succeed")
    )
});
```

Then replace per-test `PluginRuntime::new()` calls with `Arc::clone(&TEST_PLUGIN_RUNTIME)`.

**Estimated savings:** Engine init drops from per-test (~8-9s under load) to once-per-binary (~1-2s amortized). For `plugin_tests` (25 tests, total 204.7s → est. ~50-60s): −145-155s serial. For all 7 affected binaries: −400-600s serial, translating to −120-180s wall-clock.

**Safety:** Moderate. Must verify that:
1. Tests that mutate plugin state (register/unregister plugins) use a fresh runtime (cannot use the shared one)
2. Tests that test error conditions (invalid plugin, memory limit) get an isolated runtime
3. The epoch ticker in the shared Engine doesn't interfere between tests
A prototype was referenced in the S-PERF-GATE-005 history; check git stash on the PR #208 branch.

**Effort:** 1-2 days for all 7 files + verification.

---

### REC-3 [HIGH, Easy, ~150s savings] — Fix RUSTFLAGS fingerprint alignment between clippy and nextest

**Problem:** `cargo clippy` uses the default RUSTFLAGS. `cargo nextest run` uses `RUSTFLAGS=""`. These are different compiler fingerprints, so clippy artifacts cannot be reused by nextest. Every `just check` run (which runs clippy before nextest) incurs a ~157s full recompile of test binary targets.

**Fix:** Update Justfile `check` recipe to add `RUSTFLAGS=""` to the clippy step:

```justfile
check:
    cargo fmt --check
    RUSTFLAGS="" cargo clippy --all-features -- -D warnings
    RUSTFLAGS="" PROPTEST_CASES=100 cargo nextest run --workspace --all-features --profile prepush
    RUSTFLAGS="" PROPTEST_CASES=100 cargo test --workspace --all-features --doc
    @scripts/check-crate-layout.sh
    @scripts/check-non-exhaustive.sh
```

**Estimated savings:** Clippy and nextest share the same fingerprint → nextest build phase drops from ~157s to ~5-10s (just incremental changes). Saves ~150s per `just check` invocation.

**Safety:** Low risk. `RUSTFLAGS=""` is already the canonical CI flag per the Justfile comment ("RUSTFLAGS="" is set explicitly on both the nextest and doctest steps so they share the same fingerprint cache"). Extending it to clippy is consistent with the documented intent.

**Effort:** 5 minutes (one-line Justfile change).

---

### REC-4 [MEDIUM, Medium effort, ~40-60s savings] — Cap `prism-spec-engine::bc_2_11_007_pushdown_test` under dtu-cap

**Problem:** `bc_2_11_007_pushdown_test` (11 tests, avg 7.80s, 85.8s serial) starts in-process CrowdStrike and Armis DTU clones (Axum HTTP servers + tokio runtimes). It is in the `prism-spec-engine` package, so it is NOT matched by the `dtu-cap` filter (`package(/^prism-dtu-/)`). These 11 tests run completely uncapped, adding DTU server load on top of the dtu-cap=4 budget.

**Fix:** Add an additional override in nextest.toml:
```toml
[[profile.prepush.overrides]]
filter = 'binary(bc_2_11_007_pushdown_test)'
test-group = 'dtu-cap'
```

**Estimated savings:** Reduces peak DTU concurrent server count; reduces per-test inflation from ~8s to ~3-4s (estimated based on isolated vs loaded DTU times). ~40-60s wall-clock reduction.

**Safety:** Low risk; nextest config change only.

---

### REC-5 [MEDIUM, Hard, ~80-120s savings] — Deterministic stage-control for bc_2_06_019 tests

**Problem:** `bc_2_06_019_scenario_progression` tests in armis and crowdstrike set `scenario_start_secs = now - N` and immediately make HTTP requests to the DTU clone. The DTU clone handler calls `current_stage_index(&timeline, Utc::now().timestamp())` to determine which stage data to return. Under CPU load, the HTTP request may be delayed by 40-50s, causing stage drift (e.g., `now - 10s` overflows the 60s stage-0 threshold). 4 tests are already quarantined on the PR #208 branch.

**Fix:** Inject a configurable `TestClock` trait into `ArmisClone`/`CrowdstrikeClone` constructors that tests can override. Replace `Utc::now().timestamp()` in route handlers with `self.clock.now_timestamp()`. In tests, use a mock clock that returns a fixed offset from the configured `scenario_start_secs`.

**Estimated savings:** Eliminates the entire flaky-under-load class for bc_2_06_019 tests. Removes the need for wide timing buffers. The 4 quarantined tests can be un-quarantined.

**Safety:** Requires code changes in DTU harness (clock injection in clone constructors). Medium scope.

---

### REC-6 [LOW, Very Hard, ~200-300s serial savings] — Convert subprocess integration tests to in-process

**Problem:** ~100 integration tests in prism-bin spawn the `prism` binary as a subprocess (200-800ms per spawn). These are in binaries like `bc_2_22_001_boot_orchestration`, `cli_subcommands`, `bc_2_06_011_config_load`, etc.

**Fix:** Replace subprocess invocations with in-process function calls where the prism binary's `main()` is refactored into a callable entry point that returns structured results rather than exiting.

**Estimated savings:** Reduces per-test cost from ~300-800ms to ~50-100ms. For ~100 tests: ~20-70s wall-clock (limited by parallelism).

**Safety:** High effort; requires prism-bin architecture changes to support in-process test invocation. Not recommended as near-term work.

---

## 8. Test Duration Distribution

| Bucket | Test Count | % of Total |
|--------|-----------|------------|
| < 1s | 3549 | 71.3% |
| 1-5s | 1164 | 23.4% |
| 5-10s | 237 | 4.8% |
| > 10s | 26 | 0.5% |

The 263 tests in the 5-10s and >10s buckets account for disproportionate serial time due to WASMtime and wiremock overhead.

---

## 9. NFR Compliance Check

No NFR-NNN targets for test-suite wall-clock were found in `.factory/specs/prd-supplements/nfr-catalog.md` (the existing NFRs cover runtime performance: query latency, throughput, memory). This is a gap.

**Suggested threshold candidates based on this profile:**
- `just check` (pre-push gate): target < 8 minutes (current: ~13.3 min)
- nextest test execution: target < 5 minutes (current: 9.75 min)
- No single test binary (uncapped) should average > 3s per test (current worst: 9.73s)

---

## 10. Headline Summary

| Metric | Measured Value |
|--------|----------------|
| Machine | macOS darwin 25.5.0, 16 logical CPUs |
| `just check` total wall-clock (warm) | ~798s (≈ 13.3 minutes) |
| nextest test execution wall-clock | **585.84s (9m 45.8s)** |
| nextest build phase (RUSTFLAGS mismatch) | ~157s (on each `just check` after clippy) |
| Total tests run | 4976 (60 skipped / `#[ignore]`'d) |
| Test serial time (if all sequential) | 4481.5s (74.7 min) |
| Effective parallelism factor | 7.65x |
| Fastest binary (unit tests) | `prism-query`: 947 tests, avg 0.044s each |
| Slowest binary (WASMtime) | `prism-spec-engine::pipeline_http_integration`: avg 9.15s/test |
| Single slowest test | `test_BC_2_17_004_cpu_timeout_enforced_infinite_loop`: 14.94s |
| Top 3 bottlenecks by serial time | (1) WASMtime uncapped: 1022.7s, (2) wiremock HTTP uncapped: 418.2s, (3) bc_2_01_013 serialized: 179.2s |

**Top 3 recommended fixes with estimated wall-clock savings:**

1. **REC-3: RUSTFLAGS alignment** (Justfile 1-liner): saves ~150s per `just check` invocation. Zero risk.
2. **REC-1: nextest cap groups for WASMtime + HTTP binaries** (nextest.toml config): saves ~150-200s test execution wall-clock. Zero code risk.
3. **REC-2: LazyLock shared PluginRuntime** (code change in 7 test files): saves ~120-180s test execution wall-clock. Low-medium risk; prototype existed on PR #208 branch.

**Combined potential savings (REC-1 + REC-2 + REC-3):** ~420-530s (7-8 minutes) — would bring `just check` from ~798s (13.3 min) to ~270-380s (4.5-6.3 min).
