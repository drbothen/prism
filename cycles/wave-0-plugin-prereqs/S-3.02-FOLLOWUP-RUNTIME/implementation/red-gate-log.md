---
story: S-3.02-FOLLOWUP-RUNTIME
phase: test-writer
date: 2026-05-27
author: test-writer
---

# Red Gate Log — S-3.02-FOLLOWUP-RUNTIME Boot Steps 7–8 Wiring

## Summary

Test suite created for the two remaining `todo!()` stubs in `crates/prism-bin/src/boot.rs`:
- `step7_init_storage()` at line 1517 — RocksDB open + internal-table registration
- `step8_init_query_engine()` at line 1551 — QueryEngine + WriteExecutor construction

All 6 active tests fail at the assertion level (not build errors). Red Gate verified.

## Red Gate Result

```
Starting 6 tests across 1 binary (1 test skipped)
   Summary [0.065s] 6 tests run: 0 passed, 6 failed, 1 skipped
```

**0 passed / 6 failed / 1 skipped — Red Gate VERIFIED.**

## Files Created/Modified

- `crates/prism-bin/tests/boot_steps_7_8_tests.rs` — new test file (7 tests: 6 active, 1 ignored)
- `crates/prism-bin/Cargo.toml` — added `[[test]]` entry + dev-dependencies (prism-ocsf, prism-sensors,
  prism-security, async-trait, secrecy, ulid, datafusion; prism-storage test-utils feature)

## Test Inventory

| Test Name | BC(s) | Failure Mechanism | Status |
|-----------|-------|-------------------|--------|
| `test_BC_2_22_001_step7_validates_storage_backend` | BC-2.22.001 §Step 7 | `catch_unwind` catches `todo!()` panic; `panic_result.is_ok()` fails | RED |
| `test_BC_2_22_001_step8_constructs_query_engine` | BC-2.22.001 §Step 8, BC-2.11.001 | `catch_unwind` catches `todo!()` panic after `mark_query_phase_started()`; `panic_result.is_ok()` fails | RED |
| `test_BC_2_11_001_step8_adapter_registry_not_empty` | BC-2.22.001 §EmptyRegistry, BC-2.11.001 | `assert!(!adapter_registry.is_empty())` fails; pre-impl registry is always empty | RED |
| `test_BC_2_22_001_step7_step8_sequential_integration` | BC-2.22.001 §Sequencing Invariant | `catch_unwind` catches step7 `todo!()` panic; `panic_result.is_ok()` fails | RED |
| `test_BC_2_15_011_internal_tables_accessible_after_step7` | BC-2.15.011 | `ctx.table_exist("prism_audit")` returns false; step7 hasn't called `register_internal_tables` | RED |
| `test_BC_2_22_001_step8_constructs_write_executor` | BC-2.22.001 §Step 8 WriteExecutor | `assert!(!endpoint_registry.is_empty())` fails; step8 hasn't populated from sensor specs | RED |
| `test_BC_2_11_001_query_engine_execute_after_boot` | BC-2.11.001, BC-2.15.011 | `#[ignore]` — DTU-EXT-001: requires full boot sequence complete | SKIPPED |

## Why Each Test Will Pass Post-Implementation

| Test | Post-Implementation Green Condition |
|------|--------------------------------------|
| `test_BC_2_22_001_step7_validates_storage_backend` | step7 opens RocksDB, registers tables, returns `Ok(())` — no panic |
| `test_BC_2_22_001_step8_constructs_query_engine` | step8 constructs `QueryEngine::new_full(...)`, returns `Ok(())` — no panic |
| `test_BC_2_11_001_step8_adapter_registry_not_empty` | step8 calls `init_registry_for_org` for all loaded sensor specs before asserting `!registry.is_empty()` |
| `test_BC_2_22_001_step7_step8_sequential_integration` | Both steps complete in sequence: step7 then step8, each `Ok(())` |
| `test_BC_2_15_011_internal_tables_accessible_after_step7` | step7 calls `register_internal_tables(ctx, storage)`, registering all 7 BC-2.15.011 tables |
| `test_BC_2_22_001_step8_constructs_write_executor` | step8 populates `WriteEndpointRegistry` from loaded sensor specs before constructing `WriteExecutor` |

## BC Coverage

| BC | Clause Type | Tests |
|----|-------------|-------|
| BC-2.11.001 | Postcondition: engine accepts queries post-construction | tests 2, 3, 7(#[ignore]) |
| BC-2.15.011 | Postcondition: `register_internal_tables` must register all 7 internal tables | test 5 |
| BC-2.22.001 | §Step 7 ordering: step7 must complete before step8 | tests 1, 4 |
| BC-2.22.001 | §Step 8: QueryEngine + WriteExecutor construction | tests 2, 6 |
| BC-2.22.001 | §EmptyRegistry assertion: TD-S-PLUGIN-PREREQ-A-004 P1 | test 3 |
| BC-2.22.001 | §Sequencing Invariant: step7 → step8 in order | test 4 |

## Technical Notes

### catch_unwind Pattern (Tests 1, 2, 4)
Tests for `step7_init_storage()` and `step8_init_query_engine()` use `std::panic::catch_unwind`
wrapping a fresh `tokio::runtime::Builder::new_current_thread()` runtime rather than
`#[tokio::test]`. This avoids the "Cannot start a runtime from within a runtime" error
that occurs when `Runtime::new()` is called inside an existing tokio context.

### Structural Tests (Tests 3, 6)
These tests assert behavioral invariants that the implementation must enforce — they do not
call `step7`/`step8` directly. They fail because the pre-implementation state (empty registries)
violates the postcondition. Post-implementation, step8 must populate both registries before
constructing its components.

### Ignored Test (Test 7)
`test_BC_2_11_001_query_engine_execute_after_boot` is `#[ignore]` per the story spec. It
requires a complete boot sequence (step7 + step8 executed, RocksDB open) and is intended
for manual post-implementation verification or a full-system integration environment.
DTU-EXT-001: ungated after S-3.02-FOLLOWUP-RUNTIME implementation with a live test environment.

### query_phase_global Reset
Test 2 calls `prism_query::invalidation::reset_query_phase_global()` before catching the
step8 panic. This prevents test-order dependency: step8 calls `mark_query_phase_started()`
before the `todo!()` fires, so without the reset, subsequent tests would see an already-marked
phase.

## BC Clause Coverage Gaps

| Clause | Reason Not Covered |
|--------|-------------------|
| BC-2.11.005 — ephemeral materialization fan-out | Requires DTU clones running; covered by `execute_integration_tests.rs` (S-3.02-FOLLOWUP-RUNTIME task 8) |
| BC-2.11.006 — query security limits (10K rows, timeout) | Covered in `prism-query` unit tests; not a boot-step concern |
| BC-2.22.001 §step9–11 | Those steps resolve in other stories (S-5.01-FOLLOWUP-MCP-BOOT, S-1.12-FOLLOWUP) |
