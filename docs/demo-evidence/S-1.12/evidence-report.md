# S-1.12 Demo Evidence Report

**Story:** S-1.12 — prism-spec-engine: Hot Reload and Runtime Management  
**Branch:** feature/S-1.12-hot-reload  
**Policy:** POL-010  
**Recorded:** 2026-04-22  
**Tool:** VHS (CLI recordings via `cargo test`)  
**Font:** FiraCode Nerd Font Mono  
**Test result:** 36/37 pass (1 known issue — see below)

---

## Coverage Map

| AC | BC | Path | GIF | WEBM | Tape |
|----|----|------|-----|------|------|
| AC-1 | BC-2.16.005 | success | [gif](AC-001-reload-config-applies-new-config.gif) | [webm](AC-001-reload-config-applies-new-config.webm) | [tape](AC-001-reload-config-applies-new-config.tape) |
| AC-1 | BC-2.16.005 / EC-001 | error | [gif](AC-001-reload-config-validation-failure.gif) | [webm](AC-001-reload-config-validation-failure.webm) | [tape](AC-001-reload-config-validation-failure.tape) |
| AC-2 | BC-2.16.006 | success | [gif](AC-002-arc-swap-lock-free-reads.gif) | [webm](AC-002-arc-swap-lock-free-reads.webm) | [tape](AC-002-arc-swap-lock-free-reads.tape) |
| AC-2 | BC-2.16.006 / DEC-037 | error | [gif](AC-002-arc-swap-guard-stable-after-swap.gif) | [webm](AC-002-arc-swap-guard-stable-after-swap.webm) | [tape](AC-002-arc-swap-guard-stable-after-swap.tape) |
| AC-3 | BC-2.16.007 | success | [gif](AC-003-process-spec-changes-new-file.gif) | [webm](AC-003-process-spec-changes-new-file.webm) | [tape](AC-003-process-spec-changes-new-file.tape) |
| AC-3 | BC-2.16.007 / EC-003 | error | [gif](AC-003-process-spec-changes-validation-failure.gif) | [webm](AC-003-process-spec-changes-validation-failure.webm) | [tape](AC-003-process-spec-changes-validation-failure.tape) |
| AC-4 | BC-2.16.008 | success | [gif](AC-004-add-sensor-spec-success.gif) | [webm](AC-004-add-sensor-spec-success.webm) | [tape](AC-004-add-sensor-spec-success.tape) |
| AC-4 | BC-2.16.008 / EC-002 | error | [gif](AC-004-add-sensor-spec-invalid-toml.gif) | [webm](AC-004-add-sensor-spec-invalid-toml.webm) | [tape](AC-004-add-sensor-spec-invalid-toml.tape) |
| AC-5 | BC-2.16.010 | success | [gif](AC-005-list-sensor-specs-success.gif) | [webm](AC-005-list-sensor-specs-success.webm) | [tape](AC-005-list-sensor-specs-success.tape) |
| AC-5 | BC-2.16.010 / EC-005 | error | [gif](AC-005-list-sensor-specs-failed-validation.gif) | [webm](AC-005-list-sensor-specs-failed-validation.webm) | [tape](AC-005-list-sensor-specs-failed-validation.tape) |
| AC-6 | VP-032 | success | [gif](AC-006-vp032-proptest-success.gif) | [webm](AC-006-vp032-proptest-success.webm) | [tape](AC-006-vp032-proptest-success.tape) |
| AC-6 | VP-032 full suite | full run | [gif](AC-006-vp032-full-suite.gif) | [webm](AC-006-vp032-full-suite.webm) | [tape](AC-006-vp032-full-suite.tape) |

**Total recordings:** 12 (6 success paths + 6 error/edge paths)  
**ACs covered:** AC-1, AC-2, AC-3, AC-4, AC-5, AC-6  
**BCs covered:** BC-2.16.005, BC-2.16.006, BC-2.16.007, BC-2.16.008, BC-2.16.010  
**VP covered:** VP-032

---

## AC Summaries

### AC-1 — reload_config (BC-2.16.005)

**Success path** (`test_BC_2_16_005_reload_applies_new_config_on_success`): Writes a
`vendor_a.sensor.toml` into a temp dir, calls `reload_config` against an empty
`ConfigManager`, and asserts `ReloadStatus::Ok` with `vendor_a.events` in the `added`
list. The new snapshot is active for subsequent `manager.load()` calls.

**Error path** (`test_BC_2_16_005_validation_failure_retains_old_config`, EC-001): Seeds
a `ConfigManager` with `original_sensor`, writes a broken TOML file (syntax error), calls
`reload_config`. Asserts the result is `ValidationFailed` or `PartialReload`, the
`snapshot_hash` is unchanged, and `original_sensor` is still present. DI-031 fail-closed
invariant holds.

### AC-2 — ArcSwap lock-free reads (BC-2.16.006)

**Success path** (`test_BC_2_16_006_config_load_is_lock_free`): Spawns 100 concurrent
threads each calling `ConfigManager::load()`. No deadlock or panic. Each thread asserts
`sensor_a` is present — verifying lock-free concurrent access.

**Error path** (`test_BC_2_16_006_guard_holds_old_snapshot_after_swap`, DEC-037):
Simulates an in-flight query holding a `Guard` while a new snapshot is swapped in via
`manager.store()`. Asserts the held guard still sees `old_sensor` and does not see
`new_sensor`. After the guard drops, a fresh `load()` returns the new snapshot.

### AC-3 — process_spec_changes (BC-2.16.007)

**Success path** (`test_BC_2_16_007_new_spec_file_registers_tables`): Writes
`brand_new_vendor.sensor.toml`, fires a `SpecChangeEvent::Added`, calls
`process_spec_changes`. Asserts `brand_new_vendor.events` appears in `result.added` and
the sensor is in the active snapshot.

**Error path** (`test_BC_2_16_007_modified_spec_validation_failure_retains_previous_version`,
EC-003): Writes a broken TOML file at `flaky_vendor.sensor.toml`, fires
`SpecChangeEvent::Modified` against a `ConfigManager` seeded with `flaky_vendor`.
Asserts the previous version is retained in the snapshot and validation errors are
reported.

### AC-4 — add_sensor_spec (BC-2.16.008)

**Success path** (`test_BC_2_16_008_valid_new_spec_is_written_and_registered`): Uploads
a valid TOML string for `upload_vendor`. Asserts `AddSensorSpecResult::Added` is
returned, the spec file is written to disk, and table descriptors are registered.

**Error path** (`test_BC_2_16_008_invalid_toml_returns_validation_failed_no_write`,
EC-002): Uploads invalid TOML (syntax error). Asserts `ValidationFailed` is returned,
no file is written to the spec directory, and validation errors are populated.

### AC-5 — list_sensor_specs (BC-2.16.010)

**Success path** (`test_BC_2_16_010_returns_all_loaded_specs_with_tables_and_status`):
Seeds a `ConfigManager` with `list_vendor`. Calls `list_sensor_specs` with no filter.
Asserts `total_specs == 1`, `total_tables >= 1`, and the entry includes tables and
`SpecStatus::Loaded`.

**Error path** (`test_BC_2_16_010_failed_spec_shows_failed_validation_status`, EC-005):
Seeds a `ConfigManager` with a `failed_specs` entry for `broken_vendor`. Calls
`list_sensor_specs`. Asserts `broken_vendor` appears in results with
`SpecStatus::FailedValidation`.

### AC-6 — VP-032 proptest

**Success path** (`test_VP_032_failed_validation_retains_old_config` +
`test_VP_032_invariant_hash_matches_last_successful_reload`): Proptest runs 10,000
randomly-generated cases. For each case where validation fails, asserts `manager.store()`
is not called and the snapshot hash matches the last successful reload. Both proptest
cases pass.

**Full suite** (`AC-006-vp032-full-suite`): Runs the complete `hot_reload_tests` suite
showing 36 pass / 1 fail with the failure clearly identified as
`test_BC_2_16_007_unchanged_spec_skipped`.

---

## Known Failing Test

**Test:** `test_BC_2_16_007_unchanged_spec_skipped`  
**File:** `crates/prism-spec-engine/tests/hot_reload_tests.rs:486`  
**Status:** Test-writer bug — queued for triage by pr-manager during convergence cycle

**Failure message:**
```
assertion failed: result.unchanged.contains(&"stable_vendor".to_string())
```

**Root cause (implementer analysis):** The `snapshot_with_one_spec` helper constructs a
`SensorSpec` with a hardcoded `file_hash` of `"abc123"`. The actual spec file written by
`write_sensor_file` is hashed by the real `compute_file_hash` implementation, which
produces a SHA-256 of the TOML content — never `"abc123"`. Consequently the hash
comparison in `process_spec_changes` finds a mismatch and classifies the spec as
`Modified` rather than `Unchanged`.

**Impact:** Zero. The production code path (`process_spec_changes`, hash comparison, and
schema-change detection) is correct. The test helper is the defect: it needs to compute
the real file hash from the same TOML content `write_sensor_file` produces, or use a
fixture that seeds the manager from the actual parsed file rather than a hand-rolled
`SensorSpec` with a synthetic hash.

**Fix scope:** One-line change to `snapshot_with_one_spec` to call `compute_file_hash`
against `minimal_valid_sensor_toml("stable_vendor")` rather than hardcoding `"abc123"`.
No production code changes required.

**Triage owner:** pr-manager convergence cycle.

---

## File Inventory

```
docs/demo-evidence/S-1.12/
  evidence-report.md
  AC-001-reload-config-applies-new-config.tape
  AC-001-reload-config-applies-new-config.gif
  AC-001-reload-config-applies-new-config.webm
  AC-001-reload-config-validation-failure.tape
  AC-001-reload-config-validation-failure.gif
  AC-001-reload-config-validation-failure.webm
  AC-002-arc-swap-lock-free-reads.tape
  AC-002-arc-swap-lock-free-reads.gif
  AC-002-arc-swap-lock-free-reads.webm
  AC-002-arc-swap-guard-stable-after-swap.tape
  AC-002-arc-swap-guard-stable-after-swap.gif
  AC-002-arc-swap-guard-stable-after-swap.webm
  AC-003-process-spec-changes-new-file.tape
  AC-003-process-spec-changes-new-file.gif
  AC-003-process-spec-changes-new-file.webm
  AC-003-process-spec-changes-validation-failure.tape
  AC-003-process-spec-changes-validation-failure.gif
  AC-003-process-spec-changes-validation-failure.webm
  AC-004-add-sensor-spec-success.tape
  AC-004-add-sensor-spec-success.gif
  AC-004-add-sensor-spec-success.webm
  AC-004-add-sensor-spec-invalid-toml.tape
  AC-004-add-sensor-spec-invalid-toml.gif
  AC-004-add-sensor-spec-invalid-toml.webm
  AC-005-list-sensor-specs-success.tape
  AC-005-list-sensor-specs-success.gif
  AC-005-list-sensor-specs-success.webm
  AC-005-list-sensor-specs-failed-validation.tape
  AC-005-list-sensor-specs-failed-validation.gif
  AC-005-list-sensor-specs-failed-validation.webm
  AC-006-vp032-proptest-success.tape
  AC-006-vp032-proptest-success.gif
  AC-006-vp032-proptest-success.webm
  AC-006-vp032-full-suite.tape
  AC-006-vp032-full-suite.gif
  AC-006-vp032-full-suite.webm
```

Total: 37 files (1 report + 12 tapes + 12 GIFs + 12 WEBMs)
