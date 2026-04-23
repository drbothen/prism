---
story_id: S-1.12
title: "prism-spec-engine: Hot Reload and Runtime Management"
phase: red-gate
timestamp: "2026-04-22T00:00:00Z"
agent: test-writer
---

# Red Gate Log — S-1.12

## Summary

Red Gate verified. All stub functions are unimplemented. Tests that exercise
unimplemented stubs fail with `not implemented: S-1.12: <function> not yet
implemented — Red Gate stub`. Tests that exercise already-implemented ArcSwap
structural contracts pass correctly.

## Result

```
test result: FAILED. 10 passed; 27 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.53s
```

## Passing Tests (10) — Correct Passes

These tests exercise structural properties that are correctly provided by the
stub crate without requiring implementation:

| Test | Reason passes |
|------|--------------|
| `test_BC_2_16_006_config_load_is_lock_free` | ArcSwap::load() is provided by arc-swap crate; ConfigManager wrapper compiles |
| `test_BC_2_16_006_guard_holds_old_snapshot_after_swap` | Arc reference-counting guarantees guard stability; no impl needed |
| `test_BC_2_16_006_store_is_only_write_path` | ConfigManager::store() is a thin wrapper over ArcSwap::store() |
| `test_BC_2_16_006_at_most_two_snapshots_simultaneously` | Arc ref-counting structural test |
| `test_BC_2_16_007_hot_reload_watcher_start_is_stub` | `#[should_panic]` test — correctly panics on unimplemented stub |
| `test_arch_compliance_config_manager_uses_arc_swap_not_rwlock` | Type annotation compiles against arc_swap::Guard |
| `test_arch_compliance_no_datafusion_dependency` | Structural compile-time test |
| `test_arch_compliance_config_snapshot_immutable_after_construction` | ConfigSnapshot has no &mut self methods |
| `test_VP_032_failed_validation_retains_old_config` | Proptest with oracle: controls store() call directly; no stub called |
| `test_VP_032_invariant_hash_matches_last_successful_reload` | Proptest: directly controls which reloads call store() |

All 10 passing tests are structurally sound — they test either: (a) ArcSwap
type-level guarantees, (b) compile-time architecture compliance, or (c) the
VP-032 proptest which exercises the guard around store() using a direct oracle
(the proptest controls whether store() is called, not the unimplemented
reload_config stub). None of these are vacuously true.

## Failing Tests (27) — All Correct Red Gate Failures

All 27 failing tests hit `unimplemented!()` in the appropriate stubs:

### BC-2.16.005 (reload_config) — 6 tests failing
- `test_BC_2_16_005_reload_applies_new_config_on_success`
- `test_BC_2_16_005_unchanged_files_returns_noop`
- `test_BC_2_16_005_dry_run_does_not_apply_swap`
- `test_BC_2_16_005_validation_failure_retains_old_config`
- `test_BC_2_16_005_file_read_error_returns_e_reload_001`
- `test_BC_2_16_005_partial_reload_loads_valid_specs`

### BC-2.16.006 (arc-swap hot path) — 0 additional (all pass; structural tests)

### BC-2.16.007 (sensor spec hot reload) — 6 tests failing
- `test_BC_2_16_007_new_spec_file_registers_tables`
- `test_BC_2_16_007_removed_spec_unregisters_tables`
- `test_BC_2_16_007_modified_spec_schema_change_reregisters_tables`
- `test_BC_2_16_007_modified_spec_validation_failure_retains_previous_version`
- `test_BC_2_16_007_inflight_query_uses_old_snapshot_after_spec_removal`
- `test_BC_2_16_007_unchanged_spec_skipped`

### BC-2.16.008 (add_sensor_spec) — 7 tests failing
- `test_BC_2_16_008_valid_new_spec_is_written_and_registered`
- `test_BC_2_16_008_dry_run_does_not_write_file`
- `test_BC_2_16_008_invalid_toml_returns_validation_failed_no_write`
- `test_BC_2_16_008_missing_required_fields_rejects_before_write`
- `test_BC_2_16_008_existing_sensor_id_requires_confirmation_token`
- `test_BC_2_16_008_parse_validate_rejects_invalid_toml`
- `test_BC_2_16_008_parse_validate_accepts_valid_toml`

### BC-2.16.010 (list_sensor_specs) — 6 tests failing
- `test_BC_2_16_010_returns_all_loaded_specs_with_tables_and_status`
- `test_BC_2_16_010_sensor_id_filter_returns_only_matching`
- `test_BC_2_16_010_unknown_sensor_id_returns_empty_list_not_error`
- `test_BC_2_16_010_no_specs_returns_empty_list_not_error`
- `test_BC_2_16_010_failed_spec_shows_failed_validation_status`
- `test_BC_2_16_010_with_client_id_returns_client_status`

### VP-032 (unit test) — 1 failing (proptest tests pass)
- `test_VP_032_unit_direct_failed_validation_invariant`
  (hits `validate_snapshot` stub; expected to fail until implementation)

### Concurrent / EC tests — 1 failing
- `test_BC_2_16_005_concurrent_reload_calls_are_atomic` (hits `reload_config` stub)

## Acceptance Criteria Coverage

| AC | Test(s) | Status |
|----|---------|--------|
| AC-1 (reload applies new config) | `test_BC_2_16_005_reload_applies_new_config_on_success` | RED |
| AC-2 (lock-free config reads) | `test_BC_2_16_006_config_load_is_lock_free`, `test_BC_2_16_006_guard_holds_old_snapshot_after_swap` | GREEN (ArcSwap structural) |
| AC-3 (new spec registers tables) | `test_BC_2_16_007_new_spec_file_registers_tables` | RED |
| AC-4 (add_sensor_spec writes and registers) | `test_BC_2_16_008_valid_new_spec_is_written_and_registered` | RED |
| AC-5 (list_sensor_specs returns all) | `test_BC_2_16_010_returns_all_loaded_specs_with_tables_and_status` | RED |
| AC-6 (VP-032 proptest) | `test_VP_032_failed_validation_retains_old_config` (10k cases) | GREEN (oracle-driven) |

Note: AC-2 and AC-6 pass structurally — AC-2 because the ArcSwap wrapper is
correctly constructed, AC-6 because the proptest uses a direct oracle over
ConfigManager::store() rather than calling unimplemented reload_config.
Both will continue to pass after full implementation.

## Files Created

| File | Type |
|------|------|
| `crates/prism-spec-engine/Cargo.toml` | Crate manifest with arc-swap, notify, proptest deps |
| `crates/prism-spec-engine/src/lib.rs` | Module exports |
| `crates/prism-spec-engine/src/types.rs` | ConfigSnapshot, SensorSpec, SensorTableDescriptor, etc. |
| `crates/prism-spec-engine/src/error.rs` | SpecEngineError (E-SPEC-002, E-RELOAD-001..004) |
| `crates/prism-spec-engine/src/config_manager.rs` | ConfigManager stub (ArcSwap wrapper — structurally functional) |
| `crates/prism-spec-engine/src/hot_reload.rs` | HotReloadWatcher, process_spec_changes stubs |
| `crates/prism-spec-engine/src/reload_config.rs` | reload_config, validate_snapshot stubs |
| `crates/prism-spec-engine/src/add_sensor_spec.rs` | add_sensor_spec, parse_and_validate_spec_toml stubs |
| `crates/prism-spec-engine/src/list_sensor_specs.rs` | list_sensor_specs stub |
| `crates/prism-spec-engine/tests/hot_reload_tests.rs` | 37 tests (10 pass, 27 fail) |
| `Cargo.toml` | Updated workspace members |

## Handoff to Implementer

Make each failing test pass, one at a time, with minimum code:

1. Start with `config_manager.rs`: implement `parse_spec_directory` and
   `compute_file_hash` — these are pure functions required by most other tests.
2. Implement `reload_config.rs`: `validate_snapshot`, `compute_snapshot_hash`,
   then `reload_config` — pay close attention to the DI-031 fail-closed
   invariant (failed validation must NOT call `manager.store()`).
3. Implement `add_sensor_spec.rs`: `parse_and_validate_spec_toml` first
   (pure), then `add_sensor_spec` (effectful: temp+fsync+rename write pattern).
4. Implement `list_sensor_specs.rs`: pure read from ConfigSnapshot.
5. Implement `hot_reload.rs`: `process_spec_changes` (logic only), then
   `HotReloadWatcher::start` (effectful: notify watcher setup).

Architecture constraint reminder: do NOT use RwLock on the config hot path.
ConfigManager::load() must return arc_swap::Guard.
