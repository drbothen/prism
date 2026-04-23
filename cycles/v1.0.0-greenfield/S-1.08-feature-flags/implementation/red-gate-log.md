---
story: S-1.08
title: "prism-security: Feature Flags (P0 Core)"
phase: red-gate
date: 2026-04-22
author: test-writer
---

# Red Gate Log — S-1.08

## Result: RED GATE VERIFIED

All tests fail against `unimplemented!()` stubs as required by TDD protocol.

## Test Run Summary

| Test File | Total | Passed | Failed | Notes |
|-----------|-------|--------|--------|-------|
| bc_2_04_001_test | 8 | 4 | 4 | 4 passing are `#[cfg]` compile-gate detection tests — correct (see note) |
| bc_2_04_002_test | 7 | 0 | 7 | All fail on unimplemented stubs |
| bc_2_04_003_test | 10 | 0 | 10 | All fail on unimplemented stubs |
| bc_2_04_004_test | 8 | 0 | 8 | All fail on unimplemented stubs |
| bc_2_04_005_test | 8 | 0 | 8 | All fail on unimplemented stubs |
| bc_2_04_006_test | 8 | 0 | 8 | All fail on unimplemented stubs |
| bc_2_04_013_test | 8 | 0 | 8 | All fail on unimplemented stubs |
| bc_2_04_015_test | 7 | 0 | 7 | All fail on unimplemented stubs |
| vp_020_test | 7 | 0 | 7 | All fail on unimplemented stubs |
| **Total** | **71** | **4** | **67** | |

## Note on 4 Passing Tests

The 4 passing tests in `bc_2_04_001_test` are:
- `test_BC_2_04_001_real_crowdstrike_write_gate_absent`
- `test_BC_2_04_001_real_cyberint_write_gate_absent`
- `test_BC_2_04_001_real_claroty_write_gate_absent`
- `test_BC_2_04_001_real_armis_write_gate_absent`

These tests call the `crowdstrike_write_gate()` / `cyberint_write_gate()` / etc. functions,
which are pure `#[cfg(feature = "...")]` expressions requiring **no implementation** — the
gate value is determined at compile time by `rustc`, not by any runtime data structure.
These tests correctly verify that the default build (without write features) returns
`CompileTimeGate::Absent`. They are expected to pass as-is and would flip to `Present`
in a build compiled with `--features crowdstrike-write`.

This is not a vacuous test — it verifies a real compile-time property (BC-2.04.001).

## Failure Cause

All failing tests panic on `unimplemented!()` stubs in:
- `prism-core/src/capability.rs`: `ClientCapabilities::new()`, `grant()`, `is_allowed()`, etc.
- `prism-security/src/feature_flag.rs`: `FeatureFlagEvaluator::new()`, `check_permission()`, `to_error()`
- `prism-security/src/hidden_tools.rs`: `HiddenToolsRegistry::new()`, `tools_list()`, `get_tool()`
- `prism-security/src/list_capabilities.rs`: `ListCapabilitiesEngine::new()`, `execute()`
- `prism-security/src/flag_audit.rs`: `FlagAuditEmitter::new()`, `emit_write_check()`, `allowed_event()`, `denied_event()`

## AC Coverage

| AC | Test(s) | File |
|----|---------|------|
| AC-1 | `test_BC_2_04_001_absent_gate_returns_denied_compile_time`, `test_BC_2_04_001_absent_gate_runtime_allow_still_denied` | bc_2_04_001_test |
| AC-2 | `test_BC_2_04_002_runtime_allow_returns_allowed` | bc_2_04_002_test |
| AC-3 | `test_BC_2_04_005_ac3_denied_write_tool_absent_from_list` | bc_2_04_005_test |
| AC-4 | `test_BC_2_04_006_ac4_returns_capability_matrix_for_known_client` | bc_2_04_006_test |
| AC-5 | `test_BC_2_04_013_ac5_allowed_event_has_correct_fields`, `test_BC_2_04_013_ac5_denied_event_has_correct_fields` | bc_2_04_013_test |
| AC-6 | `test_BC_2_04_015_ac6_denied_write_returns_capability_denied_error`, `test_BC_2_04_015_ac6_compile_absent_returns_capability_denied_with_rebuild_suggestion` | bc_2_04_015_test |
| AC-7 | `test_BC_2_04_004_vp020_unit_assertion_counterpart_to_kani_proof`, `test_VP_020_result_equals_logical_and_of_both_gates` | vp_020_test |
| AC-8 | `test_BC_2_04_003_ac8_most_specific_deny_overrides_parent_allow` | bc_2_04_003_test |

## VP Coverage

| VP | Test(s) | File |
|----|---------|------|
| VP-020 | `test_VP_020_truth_table_*` (4 tests), `test_VP_020_result_equals_logical_and_of_both_gates`, `test_VP_020_applies_to_all_sensor_write_families`, `test_BC_2_04_004_vp020_unit_assertion_counterpart_to_kani_proof` | vp_020_test |

VP-020 Kani proof harness: `crates/prism-security/kani/feature_flag_proof.rs` (will be run by `cargo kani` — not run in this Red Gate since Kani toolchain is separate).

## BC Coverage

| BC | Tests Covering It |
|----|-------------------|
| BC-2.04.001 | bc_2_04_001_test (8 tests) |
| BC-2.04.002 | bc_2_04_002_test (7 tests) |
| BC-2.04.003 | bc_2_04_003_test (10 tests) |
| BC-2.04.004 | bc_2_04_004_test (8 tests) + vp_020_test (7 tests) |
| BC-2.04.005 | bc_2_04_005_test (8 tests) |
| BC-2.04.006 | bc_2_04_006_test (8 tests) |
| BC-2.04.013 | bc_2_04_013_test (8 tests) |
| BC-2.04.015 | bc_2_04_015_test (7 tests) |

## Edge Case Coverage

| ID | Test |
|----|------|
| EC-04-001 | `test_BC_2_04_001_ec_mixed_features_independent_gates` |
| EC-04-003 | `test_BC_2_04_002_ec_client_no_caps_inherits_empty_defaults` |
| EC-04-004 | `test_BC_2_04_002_ec_empty_defaults_all_denied` |
| EC-04-005 | `test_BC_2_04_003_ec_parent_allow_child_deny` |
| EC-04-006 | `test_BC_2_04_003_ec_broad_parent_allow_covers_all_descendants` |
| EC-04-007 | `test_BC_2_04_003_ec_four_level_hierarchy_walk`, `test_BC_2_04_003_ec_four_level_specific_deny_overrides_grandparent_allow` |
| EC-04-008 | `test_BC_2_04_004_ec_per_invocation_client_id_determines_capability` |
| EC-04-009 | `test_BC_2_04_004_ec_all_write_compiled_all_runtime_denied` |
| EC-04-010 | `test_BC_2_04_005_ec_write_enabled_for_one_of_two_clients_appears_in_list` |
| EC-04-011 | `test_BC_2_04_005_ec_no_write_clients_only_read_visible` |
| EC-04-012 | `test_BC_2_04_006_ec_null_client_id_returns_all_clients` |
| EC-04-013 | `test_BC_2_04_006_ec_zero_write_features_all_compile_time_false` |
| EC-04-027 | `test_BC_2_04_013_ec_cross_client_fan_out_one_event_per_client` |
| EC-04-028 | `test_BC_2_04_013_ec_compile_time_denial_emits_event_with_feature_not_compiled_reason` |
| EC-04-032 | `test_BC_2_04_003_ec_parent_allow_child_deny` (sensor.crowdstrike.read exact match) |
| EC-04-033 | `test_BC_2_04_005_ec_null_client_id_returns_e_flag_006` |
| EC-003 (story) | `test_BC_2_04_013_emit_does_not_panic_without_subscriber` |
| EC-005 (story) | `test_BC_2_04_015_ec_resolution_trace_minimum_one_entry` |

## Status: DONE — Handoff to Implementer

The test suite is complete and Red Gate is verified. The implementer must make
each test pass by implementing the stubs in:

1. `crates/prism-core/src/capability.rs` — port from S-1.03 (bring S-1.03 Red Gate → Green)
2. `crates/prism-security/src/feature_flag.rs` — `FeatureFlagEvaluator`, `check_permission`, `to_error`
3. `crates/prism-security/src/hidden_tools.rs` — `HiddenToolsRegistry`, `tools_list`, `get_tool`
4. `crates/prism-security/src/list_capabilities.rs` — `ListCapabilitiesEngine`, `execute`
5. `crates/prism-security/src/flag_audit.rs` — `FlagAuditEmitter`, `emit_write_check`, `allowed_event`, `denied_event`

After all unit tests are green, run the Kani proof:
```
cargo kani --harness proof_vp020_two_tier_gate_truth_table
cargo kani --harness proof_vp020_compile_absent_runtime_allow_still_denies
```
