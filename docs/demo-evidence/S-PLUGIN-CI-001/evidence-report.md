# S-PLUGIN-CI-001 Demo Evidence Report

**Story:** S-PLUGIN-CI-001 — Plugin CI Toolchain + 3 Deferred Test Closures
**Branch:** feature/S-PLUGIN-CI-001
**Recorded:** 2026-05-27
**Test Suite:** `crates/prism-spec-engine/tests/crowdstrike_oauth2_plugin_tests.rs`

---

## Summary

All 3 acceptance criteria verified via passing tests. 18 tests pass, 1 ignored (unrelated
Component Model WAT limitation in `wat` crate — pre-existing, tracked separately).

```
test result: ok. 18 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.19s
```

---

## AC-001: wasm-tools + wasi_snapshot_preview1.wasm fixture + MED-001 un-ignored

**Test:** `test_PLUGIN_MIGRATION_001_E_med_001_built_prx_loads_via_plugin_runtime`

**Command:**
```
cargo test -p prism-spec-engine --test crowdstrike_oauth2_plugin_tests \
  -- test_PLUGIN_MIGRATION_001_E_med_001_built_prx_loads_via_plugin_runtime --nocapture
```

**Output:**
```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.28s
     Running tests/crowdstrike_oauth2_plugin_tests.rs (target/debug/deps/crowdstrike_oauth2_plugin_tests-0ff51a4622ca4b48)

running 1 test
test test_PLUGIN_MIGRATION_001_E_med_001_built_prx_loads_via_plugin_runtime ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 18 filtered out; finished in 0.18s
```

**Evidence:** The `#[ignore]` annotation was removed from the MED-001 test. The test loads the
committed `.prx` artifact at `crates/prism-spec-engine/plugins/crowdstrike-oauth2/crowdstrike-oauth2.prx`
(129KB) via `PluginRuntime::load_plugin`, verifies the `plugin_id` is `"crowdstrike-oauth2"`, and
confirms the plugin is registered in the runtime after load. `wasi_snapshot_preview1.wasm` (52KB)
is committed to `tests/fixtures/` with full provenance documented in `tests/fixtures/README.md`.

**BCs satisfied:** BC-2.17.006 (WIT validation passes), BC-2.17.007 (manifest schema passes)

---

## AC-002: EC-006 missing-.prx boot-continue behavior

**Test:** `test_S_PLUGIN_CI_001_002_missing_prx_at_boot_continues_with_error_log`

**Command:**
```
cargo test -p prism-spec-engine --test crowdstrike_oauth2_plugin_tests \
  -- test_S_PLUGIN_CI_001_002_missing_prx_at_boot_continues_with_error_log --nocapture
```

**Output:**
```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.27s
     Running tests/crowdstrike_oauth2_plugin_tests.rs (target/debug/deps/crowdstrike_oauth2_plugin_tests-0ff51a4622ca4b48)

running 1 test
test test_S_PLUGIN_CI_001_002_missing_prx_at_boot_continues_with_error_log ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 18 filtered out; finished in 0.01s
```

**Evidence:** Test verifies three sub-assertions:
- (a) `load_plugin` with a non-existent path returns `Err`, not `Ok` and not a panic
- (b) Error variant is `PluginError::CompilationFailed { path, message }` with the missing file path
- (c) Runtime is NOT poisoned — a valid plugin loads successfully after the failed attempt

Closes PLUGIN-MIGRATION-001-E EC-006 deferral (boot-with-missing-prx behavior).

**BCs satisfied:** BC-2.17.001 (panic isolation — n-1 survivor rule), BC-2.22.001 (boot continues)

---

## AC-003: EC-009 double-401 → AuthRefreshFailed

**Test:** `test_S_PLUGIN_CI_001_003_double_401_returns_auth_refresh_failed`

**Command:**
```
cargo test -p prism-spec-engine --test crowdstrike_oauth2_plugin_tests \
  -- test_S_PLUGIN_CI_001_003_double_401_returns_auth_refresh_failed --nocapture
```

**Output:**
```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.28s
     Running tests/crowdstrike_oauth2_plugin_tests.rs (target/debug/deps/crowdstrike_oauth2_plugin_tests-0ff51a4622ca4b48)

running 1 test
test test_S_PLUGIN_CI_001_003_double_401_returns_auth_refresh_failed ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 18 filtered out; finished in 0.01s
```

**Evidence:** Test uses wiremock to return HTTP 401 for ALL requests to the detection query
endpoint (both initial request and post-refresh retry). `PipelineExecutor::execute` is wired
with the crowdstrike-oauth2 WAT plugin via `PluginAuthProvider`. Test verifies:
- (a) `execute` returns `Err(...)`, not `Ok`
- (b) Error contains `"AuthRefreshFailed"` or `"E-AUTH-002"`
- (c) No panic occurs — sandbox error paths do not panic the host (BC-2.17.001)

Closes PLUGIN-MIGRATION-001-E EC-009 deferral (double-401 terminal failure case).

**BCs satisfied:** BC-2.17.001 (sandbox error paths do not panic host), BC-2.22.001 (plugin-provided
auth participates in PipelineExecutor retry path)

---

## Full Suite Run

```
cargo test -p prism-spec-engine --test crowdstrike_oauth2_plugin_tests 2>&1

running 19 tests
test test_F_LP7_MED_001_host_dispatch_acquire_token_kv_miss_emits_audit_event ... ignored, requires Component Model WAT parse support
test test_PLUGIN_MIGRATION_001_E_task1_sensor_spec_without_auth_plugin_parses_to_none ... ok
test test_PLUGIN_MIGRATION_001_E_008_vp148_parity_green_after_toml_amendment ... ok
test test_PLUGIN_MIGRATION_001_E_007d_no_auth_plugin_field_passes_validation ... ok
test test_PLUGIN_MIGRATION_001_E_007_crowdstrike_toml_declares_auth_plugin ... ok
test test_PLUGIN_MIGRATION_001_E_007b_unknown_auth_plugin_emits_e_spec_012 ... ok
test test_PLUGIN_MIGRATION_001_E_007c_registered_auth_plugin_passes_validation ... ok
test test_PLUGIN_MIGRATION_001_E_010_token_not_in_tracing_output ... ok
test test_PLUGIN_MIGRATION_001_E_004_token_cached_within_ttl_no_second_request ... ok
test test_PLUGIN_MIGRATION_001_E_003_acquire_token_calls_oauth2_token_endpoint ... ok
test test_PLUGIN_MIGRATION_001_E_005_expired_token_triggers_reacquisition ... ok
test test_PLUGIN_MIGRATION_001_E_001_plugin_compiles_and_manifest_validates ... ok
test test_PLUGIN_MIGRATION_001_E_crit_001_kv_store_arc_shared_across_dispatches ... ok
test test_PLUGIN_MIGRATION_001_E_009_plugin_loaded_at_boot_step_7_5_emits_warn ... ok
test test_S_PLUGIN_CI_001_002_missing_prx_at_boot_continues_with_error_log ... ok
test test_PLUGIN_MIGRATION_001_E_002_auth_type_name_returns_oauth2_client_credentials ... ok
test test_S_PLUGIN_CI_001_003_double_401_returns_auth_refresh_failed ... ok
test test_PLUGIN_MIGRATION_001_E_006_401_triggers_plugin_token_refresh_and_retry ... ok
test test_PLUGIN_MIGRATION_001_E_med_001_built_prx_loads_via_plugin_runtime ... ok

test result: ok. 18 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.19s
```

---

## Traceability Summary

| AC | Test Name | Status | BCs |
|----|-----------|--------|-----|
| AC-001 | `test_PLUGIN_MIGRATION_001_E_med_001_built_prx_loads_via_plugin_runtime` | PASS | BC-2.17.006, BC-2.17.007 |
| AC-002 | `test_S_PLUGIN_CI_001_002_missing_prx_at_boot_continues_with_error_log` | PASS | BC-2.17.001, BC-2.22.001 |
| AC-003 | `test_S_PLUGIN_CI_001_003_double_401_returns_auth_refresh_failed` | PASS | BC-2.17.001, BC-2.22.001 |
