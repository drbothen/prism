# Red Gate Log — S-PRISMQL-NATIVE-TEMPORAL-TYPING-001

**Story:** S-PRISMQL-NATIVE-TEMPORAL-TYPING-001 — PrismQL UTF-8 to Timestamp Arrow type migration  
**Phase:** Phase 3 / wave-5-e-demo-fidelity  
**Stubs commit (original):** `9401a6ca`  
**Stubs commit (Option-A skeleton):** `13b9c8ec`  
**Red Gate commit (original, 10 tests):** `3aac1e73`  
**Red Gate commit (Option-A tests, +25 tests):** `(this commit)`  
**Date:** 2026-07-04  
**Author:** test-writer  

---

## Pass 1 Red Gate (commit 3aac1e73) — SUPERSEDED

The original Pass 1 Red Gate (RG-001..RG-010, stubs commit 9401a6ca) established 10 failing
tests. These tests have been RESOLVED by subsequent implementation commits including
the Option-A skeleton commit 13b9c8ec. All 10 original RG tests now pass.

See the original Pass 1 log below (preserved for audit trail).

---

## Pass 2 Red Gate — Option-A Remaining Implementation

**Stubs commit:** `13b9c8ec` (Option-A skeleton — `Literal::RawTemporalLiteral`, `is_date_like`
stub, `check_temporal_literals_opt_a` stub, parser lenient-fallback hook, emitter guard)

**Test command:** `cargo nextest run -p prism-query -E 'test(S_PRISMQL_NATIVE_TEMPORAL_TYPING_001)' --no-fail-fast`

```
Summary [   0.198s] 42 tests run: 30 passed, 12 failed, 1246 skipped
```

**RED GATE: PASS** — 12 tests fail. Zero new tests pass vacuously. The gate is verified.

### Context: Why 30 Tests Pass

The stubs commit 13b9c8ec went beyond pure stubs and partially implemented Option-A:
- E-QUERY-041 detection for all 7 date-like forms against `ColumnType::Datetime` columns
- Full RFC-3339 regression guard (valid UTC timestamps not gated)
- Non-date-like string passthrough
- Dotted-column schema resolution
- Equality-operator temporal gating
- Unicode safety (VP-021 satisfied by construction)
- Exact E-QUERY-041 message format (POL-24)

The 30 passing tests verify this already-implemented behavior. The 12 failing tests represent the REMAINING Option-A work that the implementer must complete.

### Failing Tests (True Red Gate — 12 tests)

| # | Test Name | File | Failure Mode | BC/ADR Anchor |
|---|-----------|------|--------------|---------------|
| RG-013 | `test_..._string_col_coercion_date_only_succeeds` | `temporal_typing_tests.rs` | Assertion `!matches!(QueryParseFailed)` = false: parse still returns QueryParseFailed for date-only (not coerced) | ADR-052 §D4 coercion arm; RISK-5 |
| RG-014 | `test_..._string_col_coercion_offset_less_succeeds` | `temporal_typing_tests.rs` | Same: QueryParseFailed for `'2026-06-24T12:00:00'` vs String col | ADR-052 §D4 coercion arm |
| RG-015 | `test_..._integer_col_date_like_e_query_001` | `temporal_typing_tests.rs` | Assertion `!matches!(QueryParseFailed)` = false: parse fails before three-way dispatch | ADR-052 §D4 Step 3 third arm |
| RG-016 | `test_..._float_col_date_like_e_query_001` | `temporal_typing_tests.rs` | Same: QueryParseFailed before three-way dispatch | ADR-052 §D4 Step 3 third arm |
| RG-017 | `test_..._bool_col_date_like_e_query_001` | `temporal_typing_tests.rs` | Same: QueryParseFailed before three-way dispatch | ADR-052 §D4 Step 3 third arm |
| RG-020 | `test_..._qualified_nested_column_resolution` | `temporal_typing_tests.rs` | Assertion `!matches!(QueryParseFailed)` = false: `ghost_sensor_devices.hostname = '2026-06-24'` parse fails | ADR-052 §D4 Step 3 qualified column |
| RG-023 | `test_..._projection_position_e_query_001` | `temporal_typing_tests.rs` | Assertion `!matches!(QueryParseFailed)` = false: `SELECT '2026-06-24' FROM test_events` parse fails | ADR-052 §D4 Step 3 last row |
| RG-024 | `test_..._emitter_guard_raw_temporal_literal` | `pipe_sql_emitter.rs` | `todo!()` PANIC — blocked on Task 11B (return type change) | ADR-052 §D4 Step 5; Task 11B |
| RG-031 | `test_..._string_col_coercion_space_sep_succeeds` | `temporal_typing_tests.rs` | Assertion `!matches!(QueryParseFailed)` = false: space-sep `'2026-06-24 12:00:00'` vs String col | ADR-052 §D4 coercion arm |
| RG-033 | `test_..._string_col_coercion_unpadded_date_succeeds` | `temporal_typing_tests.rs` | Same: unpadded `'2026-6-24'` vs String col parse fails | ADR-052 §D4 over-match disposition |
| (stub l) | `test_..._parser_emits_raw_temporal_for_date_only` | `parser_tests.rs` | Assert parse OK with RawTemporalLiteral — FAILS because parse still returns Err | ADR-052 §D4 Step 2 |
| RG-034 | `test_..._near_miss_trailing_chars_stays_utf8` | `parser_tests.rs` | Assert parse OK with Literal::String for `'2026-06-24extra'` — FAILS because parse returns Err (looks_like_timestamp=true) | ADR-052 §D4 near-miss |

### Passing Tests (Already Implemented — 30 tests)

The 30 passing tests include:

**Pre-existing solid functional core (old tests):**
- `e_query_041_pipe_mode_date_only_string` — date-only vs Datetime in pipe mode → E-QUERY-041
- `e_query_041_sql_mode_date_only_string` — date-only vs Datetime in SQL mode → E-QUERY-041
- `ec006_offset_less_datetime_raises_e_query_041` — EC-006 case
- `dotted_external_source_pipe_date_only_raises_e_query_041` — dotted source E-QUERY-041
- `string_column_ordering_not_rejected` — String col ordering passes through
- `obs2_string_column_equality_not_gated` — String col equality not gated
- `valid_rfc3339_utc_string_not_rejected` — valid RFC-3339 passes
- `filter_mode_valid_rfc3339_not_rejected` — filter mode valid RFC-3339
- `obs2_equality_valid_rfc3339_not_rejected` — equality valid RFC-3339
- `low1_grammar_rejects_literal_lhs_comparison` — grammar constraint
- `f_local_low1_pipe_no_from_date_only_raises_e_query_041` — F-LOCAL-LOW-1(c)
- `f_local_low1_pipe_no_from_dotted_source_date_only_raises_e_query_041` — F-LOCAL-LOW-1(d)
- `f_local_low1_pipe_no_from_valid_rfc3339_not_rejected` — F-LOCAL-LOW-1 negative

**Option-A tests now passing (behavior already implemented by 13b9c8ec):**
- `full_rfc3339_regression_guard` (RG-011) — valid RFC-3339 not gated ✓
- `e_query_041_offset_less_datetime_col` (RG-012) — form 2 vs Datetime → E-QUERY-041 ✓
- `dotted_source_column_resolution` (RG-019) — dotted source → E-QUERY-041 ✓
- `filter_pipe_syntax_e_query_041` (RG-021) — equality operator gated ✓
- `unicode_input_no_panic` (RG-022) — VP-021 satisfied ✓
- `e_query_041_message_format_byte_identical` (RG-025) — POL-24 format ✓
- `e_query_041_fractional_t_sep_datetime_col` (RG-026) — form 3 ✓
- `e_query_041_no_seconds_t_sep_datetime_col` (RG-027) — form 4 ✓
- `e_query_041_space_sep_full_seconds_datetime_col` (RG-028) — form 5 ✓
- `e_query_041_space_sep_fractional_datetime_col` (RG-029) — form 6 ✓
- `e_query_041_space_sep_no_seconds_datetime_col` (RG-030) — form 7 ✓
- `e_query_041_unpadded_date_overmatch_datetime_col` (RG-032) — over-match ✓
- `non_date_like_stays_string_literal` (RG-018) — non-date-like passthrough ✓

**Emitter/plan-pinning tests (already implemented):**
- `pipe_sql_emitter_yields_arrow_cast_literal` (RG-003)
- `emitter_output_plans_against_timestamp_column` (RG-010)
- `high002_datetime_column_type_is_timestamp` (RG-009)
- `risk1_datafusion_arrow_cast_probe` (RG-002)

### Summary: What the Implementer Must Build

The 12 failing tests drive these remaining implementation tasks:

1. **Task 13 (parser `is_date_like` wire-up):** Wire `is_date_like` into `classify_string_literal`
   so that date-like non-RFC-3339 strings emit `Literal::RawTemporalLiteral(s)` instead of
   `Err(parse error)`. This unblocks stub l (parser test) and RG-034 (near-miss test).

2. **Task 3 (coercion arm):** `check_temporal_literals_opt_a` must resolve String columns and
   rewrite `RawTemporalLiteral → Literal::String` in-place. This closes RG-013, RG-014,
   RG-020 (qualified column), RG-031, RG-033.

3. **Task 4 (third arm — Integer/Float/Bool):** `check_temporal_literals_opt_a` must return
   E-QUERY-001 for `RawTemporalLiteral` against non-Datetime/non-String columns. Closes
   RG-015, RG-016, RG-017.

4. **Task (projection position):** `check_temporal_literals_opt_a` must handle
   `RawTemporalLiteral` in non-comparison context → E-QUERY-001. Closes RG-023.

5. **Task 11B (emitter guard):** Change `literal_to_sql` return type to
   `Result<String, PrismError>` and return `Err(E-QUERY-001)` for
   `Literal::RawTemporalLiteral`. Closes RG-024.

---

## Pass 1 Red Gate Log (original, preserved for audit trail)

**Stubs commit:** `9401a6ca`  
**Red Gate commit:** `3aac1e73`  
**Date:** 2026-07-03  
**Author:** test-writer  

```
Summary [   0.104s] 10 tests run: 0 passed, 10 failed, 1862 skipped
```

**RED GATE: PASS** — All 10 tests fail. Zero tests pass vacuously. The gate is verified.

| # | Test Name | File | Failure Mode | BC/ADR Anchor |
|---|-----------|------|--------------|---------------|
| RG-001 | `test_..._datetime_column_registers_as_timestamp_micros_utc` | `spec_driven_adapter.rs` | Assertion: `Utf8 ≠ Timestamp(Microsecond, Some("UTC"))` | ADR-052 D1/D2, BC-2.11.003 v1.6 |
| RG-002 | `test_..._risk1_datafusion_arrow_cast_probe` | `high002_plan_pinning_tests.rs` | `todo!()` panic | ADR-052 D3 RISK-1 |
| RG-003 | `test_..._pipe_sql_emitter_yields_arrow_cast_literal` | `pipe_sql_emitter.rs` | Assertion: bare string ≠ arrow_cast | ADR-052 D3, BC-2.11.004 v1.7 |
| RG-004 | `test_..._e_query_041_sql_mode_date_only_string` | `temporal_typing_tests.rs` | `todo!()` panic in `check_temporal_literals` | BC-2.11.021 v1.2 |
| RG-005 | `test_..._e_query_041_pipe_mode_date_only_string` | `temporal_typing_tests.rs` | `todo!()` panic | BC-2.11.021 v1.2 |
| RG-006 | `test_..._e_query_041_map_prism_error_invalid_params` | `error_mapping.rs` | Assertion: `-32000 ≠ -32602` | BC-2.11.021 v1.2 §MCP mapping |
| RG-007 | `test_..._valid_rfc3339_utc_string_not_rejected` | `temporal_typing_tests.rs` | `todo!()` panic | ADR-052 D4, BC-2.11.021 v1.2 |
| RG-008 | `test_..._sensor_datetime_string_parsed_to_micros` | `spec_driven_adapter.rs` | `todo!()` panic in `parse_datetime_to_micros` | ADR-052 D5, BC-2.11.003 v1.6 |
| RG-009 | `test_..._high002_datetime_column_type_is_timestamp` | `high002_plan_pinning_tests.rs` | Assertion: `Utf8 ≠ Timestamp(...)` | ADR-052 D2, BC-2.11.003 v1.6 |
| RG-010 | `test_..._emitter_output_plans_against_timestamp_column` | `pipe_sql_emitter.rs` | Stage-1 assertion: bare string ≠ arrow_cast | ADR-052 D3 |

---

## Status

**RED GATE VERIFIED (Pass 2).** 12 tests fail before implementation. 30 tests pass
(solid functional core + already-implemented stubs behaviors). Hand off to implementer:
make each failing test pass, one at a time, with minimum code — in the order:
Task 13 (parser wire-up) → Task 3 (coercion arm) → Task 4 (Integer/Float/Bool arm)
→ projection position → Task 11B (emitter guard).
