# Red Gate Log — S-PRISMQL-NATIVE-TEMPORAL-TYPING-001

**Story:** S-PRISMQL-NATIVE-TEMPORAL-TYPING-001 — PrismQL UTF-8 to Timestamp Arrow type migration  
**Phase:** Phase 3 / wave-5-e-demo-fidelity  
**Stubs commit:** `9401a6ca`  
**Red Gate commit:** `3aac1e73`  
**Date:** 2026-07-03  
**Author:** test-writer  

## Red Gate Result

```
Summary [   0.104s] 10 tests run: 0 passed, 10 failed, 1862 skipped
```

**RED GATE: PASS** — All 10 tests fail. Zero tests pass vacuously. The gate is verified.

## Test Inventory

| # | Test Name | File | Failure Mode | BC/ADR Anchor |
|---|-----------|------|--------------|---------------|
| RG-001 | `test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_datetime_column_registers_as_timestamp_micros_utc` | `crates/prism-bin/src/spec_driven_adapter.rs` | Assertion: `Utf8 ≠ Timestamp(Microsecond, Some("UTC"))` | ADR-052 D1/D2, BC-2.11.003 v1.6 |
| RG-002 | `test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_risk1_datafusion_arrow_cast_probe` | `crates/prism-query/src/tests/high002_plan_pinning_tests.rs` | `todo!()` panic — implementer fills probe | ADR-052 D3 RISK-1 |
| RG-003 | `test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_pipe_sql_emitter_yields_arrow_cast_literal` | `crates/prism-query/src/pipe_sql_emitter.rs` | Assertion: bare `'...'` ≠ `arrow_cast('...', 'Timestamp(Microsecond, Some("UTC"))')` | ADR-052 D3, BC-2.11.004 v1.7 |
| RG-004 | `test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_e_query_041_sql_mode_date_only_string` | `crates/prism-query/src/tests/temporal_typing_tests.rs` | `todo!()` panic in `check_temporal_literals` (engine.rs:1845) | BC-2.11.021 v1.2 E-QUERY-041 |
| RG-005 | `test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_e_query_041_pipe_mode_date_only_string` | `crates/prism-query/src/tests/temporal_typing_tests.rs` | `todo!()` panic in `check_temporal_literals` (engine.rs:1845) | BC-2.11.021 v1.2 E-QUERY-041 |
| RG-006 | `test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_e_query_041_map_prism_error_invalid_params` | `crates/prism-mcp/src/error_mapping.rs` | Assertion: `-32000 ≠ -32602` (INTERNAL_ERROR instead of INVALID_PARAMS) | BC-2.11.021 v1.2 §MCP mapping |
| RG-007 | `test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_valid_rfc3339_utc_string_not_rejected` | `crates/prism-query/src/tests/temporal_typing_tests.rs` | `todo!()` panic in `check_temporal_literals` (engine.rs:1845) — not TemporalLiteralUnparseable | ADR-052 D4, BC-2.11.021 v1.2 |
| RG-008 | `test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_sensor_datetime_string_parsed_to_micros` | `crates/prism-bin/src/spec_driven_adapter.rs` | `todo!()` panic in `parse_datetime_to_micros` | ADR-052 D5, BC-2.11.003 v1.6 |
| RG-009 | `test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_high002_datetime_column_type_is_timestamp` | `crates/prism-query/src/tests/high002_plan_pinning_tests.rs` | Assertion: `Utf8 ≠ Timestamp(Microsecond, Some("UTC"))` in `make_timestamp_batch` | ADR-052 D2, BC-2.11.003 v1.6 |
| RG-010 | `test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_emitter_output_plans_against_timestamp_column` | `crates/prism-query/src/pipe_sql_emitter.rs` | Stage-1 assertion: bare string ≠ arrow_cast form (fails before DataFusion stage) | ADR-052 D3 |

## Pre-Existing Failure Note

`test_BC_2_11_001_query_engine_execute_after_boot` (prism-bin) fails due to the `check_temporal_literals` `todo!()` stub wired into `engine.rs:1845` by the stubs commit `9401a6ca`. This failure was verified as pre-existing before the Red Gate tests were added (confirmed via `git stash` + test run on the stubs commit alone). It is NOT introduced by the Red Gate test commit.

## Discovery: DataFusion 53.1.0 Implicit Coercion

During Red Gate construction, an empirical discovery was made: DataFusion 53.1.0 with arrow-cast 58.2.0 DOES implicitly coerce bare string literals to `Timestamp` when compared against a `Timestamp(Microsecond, UTC)` column. The story spec claimed bare string form "fails to plan without error" — this claim is incorrect for this DataFusion version.

**Impact on RG-010:** RG-010 was redesigned from "assert plan succeeds with bare string" to a two-stage test. Stage 1 (Red Gate) asserts the emitter output equals the `arrow_cast(...)` string form — this fails immediately before implementation since the emitter still returns bare `'...'`. Stage 2 (post-implementation) verifies the emitter output also plans correctly in DataFusion, catching quoting/escaping mistakes. The Red Gate property is preserved; the story spec's DataFusion claim should be noted to the implementer as incorrect.

**Implementer note:** The implementer does NOT need to verify that bare strings fail planning (they don't in DataFusion 53.1.0). They DO need to ensure `literal_to_sql` emits the `arrow_cast(...)` form as specified in ADR-052 D3, and that the emitted fragment plans successfully.

## Files Modified

| File | Change |
|------|--------|
| `crates/prism-bin/src/spec_driven_adapter.rs` | Added `parse_datetime_to_micros` stub + RG-001, RG-008 tests |
| `crates/prism-mcp/src/error_mapping.rs` | Added RG-006 test |
| `crates/prism-query/src/pipe_sql_emitter.rs` | Added RG-003, RG-010 tests |
| `crates/prism-query/src/tests/high002_plan_pinning_tests.rs` | Added RG-002, RG-009 tests |
| `crates/prism-query/src/tests/mod.rs` | Registered `temporal_typing_tests` module |
| `crates/prism-query/src/tests/temporal_typing_tests.rs` | New file — RG-004, RG-005, RG-007 tests |

## Status

RED GATE VERIFIED. Hand off to implementer: make each test pass, one at a time, with minimum code.
