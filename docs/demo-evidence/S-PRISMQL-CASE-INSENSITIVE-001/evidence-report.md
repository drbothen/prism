# Demo Evidence Report — S-PRISMQL-CASE-INSENSITIVE-001

Story: PrismQL Case-Insensitive Operators (IEQ/IIN/INE) + Adapter-Boundary OCSF Enum-Label Normalization (ADR-047)
Branch: feature/S-PRISMQL-CASE-INSENSITIVE-001
HEAD SHA: f9be96fa
Captured: 2026-07-08
Provenance: LOCAL 3-CLEAN converged at de89b557 (passes 33/34/35, CLEAN strict; BC-5.39.001 satisfied); pre-PR-LEVEL fix-burst 54c89898 (CR-002/CR-003/CR-004/SEC-001; RG-075..078); PR-LEVEL pass-1 fix-burst 56fb83d8+f9be96fa (ADV-PR-P1-MED-001/MED-002/LOW-001/OBS-002; RG-079..081)

## Workspace Gate

```
just check
  cargo fmt --check           PASS
  cargo clippy -D warnings    PASS (0 warnings)
  cargo nextest run --workspace --all-features --profile prepush
    5317 tests run: 5317 passed, 60 skipped, 0 failures
  cargo test (doctests)       PASS
  check-non-exhaustive.sh     PASS: 89 types correctly reject external construction (expected: 89)
```

Result: ALL GREEN. No regressions.

---

## AC Evidence Table

Evidence method key:
- **RGT** = Red Gate Test — cargo nextest run targeted invocation, output captured
- **compile-enforced** = compilation failure enforces the constraint; no standalone stub needed
- **grep** = structural verification via rg/grep over source files

| AC | Description | Evidence Method | RGT(s) | Status |
|----|-------------|-----------------|--------|--------|
| AC-001 | IEQ parses to Predicate::Compare{case_insensitive:true} | RGT (prism-query) | RG-001 `test_S_PRISMQL_CASE_INSENSITIVE_001_ieq_parses_to_compare_case_insensitive_true` | PASS |
| AC-002 | IIN parses to Predicate::In{case_insensitive:true} | RGT (prism-query) | RG-002 `test_S_PRISMQL_CASE_INSENSITIVE_001_iin_parses_to_in_case_insensitive_true` | PASS |
| AC-003 | INE parses to Predicate::Compare{op:Ne,case_insensitive:true} | RGT (prism-query) | RG-003 `test_S_PRISMQL_CASE_INSENSITIVE_001_ine_parses_to_compare_ne_case_insensitive_true` | PASS |
| AC-004 | Keyword parsing case-insensitive: ieq/IEQ/Ieq identical AST | RGT (prism-query) | RG-004 `test_S_PRISMQL_CASE_INSENSITIVE_001_ieq_keyword_case_insensitive_parsing` | PASS |
| AC-005 | IIN parses before IN — no prefix-match collision | RGT (prism-query) | RG-005 `test_S_PRISMQL_CASE_INSENSITIVE_001_iin_before_in_no_collision` | PASS |
| AC-006 | Sibling-site sweep: Predicate::Compare construction sites add case_insensitive:false | compile-enforced + grep | See grep evidence below | PASS |
| AC-007 | Sibling-site sweep: Predicate::In construction sites add case_insensitive:false | compile-enforced + grep | See grep evidence below | PASS |
| AC-008 | IEQ lowers to lower(field) = lower('val') | RGT (prism-query) | RG-006 `test_S_PRISMQL_CASE_INSENSITIVE_001_ieq_emits_lower_equals_lower`; RG-076 `test_cr003_normalize_predicate_invalid_ci_op_emits_warn` (warn before IEQ placeholder fallback on invalid case_insensitive+non-Eq/Ne combination; CR-003) | PASS |
| AC-009 | INE lowers to lower(field) != lower('val') | RGT (prism-query) | RG-007 `test_S_PRISMQL_CASE_INSENSITIVE_001_ine_emits_lower_ne_lower` | PASS |
| AC-010 | IIN lowers to lower(field) IN (lower('v1'), lower('v2')) | RGT (prism-query) | RG-008 `test_S_PRISMQL_CASE_INSENSITIVE_001_iin_emits_lower_in_lower_list` | PASS |
| AC-011 | Case-sensitive =, !=, IN emit unchanged (no lower() wrapping) | RGT (prism-query) | RG-009 `test_S_PRISMQL_CASE_INSENSITIVE_001_case_sensitive_eq_no_lower_wrapping` | PASS |
| AC-012 | IEQ execution: matches rows regardless of casing | RGT (prism-query) | RG-010 `test_S_PRISMQL_CASE_INSENSITIVE_001_ieq_execution_case_insensitive_match` | PASS |
| AC-013 | Case-sensitive = returns 0 rows when casing differs (regression guard) | RGT (prism-query) | RG-011 `test_S_PRISMQL_CASE_INSENSITIVE_001_case_sensitive_eq_returns_zero_on_casing_mismatch` | PASS |
| AC-013b | IEQ/IIN available in pipe-mode \| where stage | RGT (prism-query) | RG-012 `test_S_PRISMQL_CASE_INSENSITIVE_001_ieq_in_pipe_where_stage` | PASS |
| AC-014 | normalized_pql reflects IEQ/IIN/INE in uppercase canonical form | RGT (prism-query) | RG-013 `test_S_PRISMQL_CASE_INSENSITIVE_001_normalized_pql_reflects_ieq_uppercase` | PASS |
| AC-015 | normalized_pql round-trip: parse -> normalize -> re-parse -> same AST | RGT (prism-query) | RG-014 `test_S_PRISMQL_CASE_INSENSITIVE_001_normalized_pql_round_trip_ast_equality` | PASS |
| AC-016 | OCSF enum-label fields normalized to Title-case via build_column_array (PRIMARY path) | RGT (prism-bin + prism-ocsf) | RG-032 `test_BC_2_02_013_build_column_array_normalizes_severity_to_title_case` (PRIMARY); RG-019 `test_S_PRISMQL_CASE_INSENSITIVE_001_adapter_normalization_critical_to_title_case` (SECONDARY) | PASS |
| AC-017 | Normalization covers activity_name and disposition; idempotent (PRIMARY + guards) | RGT (prism-bin + prism-ocsf) | RG-033 `test_BC_2_02_013_build_column_array_normalizes_status_and_disposition`; RG-035 `test_BC_2_02_013_build_column_array_non_enum_string_column_untouched`; RG-036 `test_BC_2_02_013_build_column_array_non_string_column_untouched`; RG-020 `test_S_PRISMQL_CASE_INSENSITIVE_001_adapter_normalization_idempotent_high` (SECONDARY); RG-075 `test_cr002_normalize_enum_label_already_canonical_returns_some` (canonical-label no-op write guard: normalize_enum_label returns Some(value) unchanged when already canonical; idempotence BC-2.02.013 invariant; CR-002) | PASS |
| AC-018 | Unrecognized vendor values left as-received with warning logged | RGT (prism-bin + prism-ocsf) | RG-034 `test_BC_2_02_013_build_column_array_unrecognized_left_as_received_with_warn` (PRIMARY); RG-047 `test_BC_2_02_013_build_column_array_empty_string_enum_value_no_warn`; RG-021 `test_S_PRISMQL_CASE_INSENSITIVE_001_adapter_normalization_unrecognized_value_left_as_received` (SECONDARY); RG-054 `test_BC_2_02_013_normalizer_secondary_empty_string_enum_value_no_warn`; RG-077 `test_cr004_build_column_array_enum_label_warn_strips_control_chars` (CWE-117 PRIMARY: sanitize_for_log strips control chars from value before logging; CR-004/SEC-001); RG-078 `test_cr004_sanitize_for_log_strips_control_chars_for_secondary_site` (CWE-117 SECONDARY: same strip at normalize_with_mappers; CR-004/SEC-001); RG-079 `test_rg079_secondary_sanitize_enum_label_order_spec_wins` (sanitize-BEFORE-truncate order at SECONDARY site; ADV-PR-P1-MED-001); RG-080 `test_rg080_low001_build_column_array_enum_label_warn_order_of_operations` (order-of-operations at PRIMARY site, warn-capture; ADV-PR-P1-MED-001/LOW-001) | PASS |
| AC-019 | GROUP BY severity produces at most 7 buckets after normalization | RGT (prism-query + prism-bin) | RG-022 `test_S_PRISMQL_CASE_INSENSITIVE_001_group_by_severity_no_case_fragmentation`; RG-044 `test_BC_2_02_013_triage_alerts_prompt_no_stale_vendor_casing`; RG-071 `test_BC_2_02_013_build_column_array_group_by_severity_cross_sensor_no_fragmentation` | PASS |
| AC-020 | E-QUERY-001: IEQ/INE with non-string literal RHS rejected at parse time | RGT (prism-query) | RG-016 `test_S_PRISMQL_CASE_INSENSITIVE_001_ieq_non_string_rhs_e_query_001`; RG-055/056/057 date-like RHS accepted as string | PASS |
| AC-021 | E-QUERY-001: IIN with empty membership list rejected at parse time | RGT (prism-query) | RG-017 `test_S_PRISMQL_CASE_INSENSITIVE_001_iin_empty_list_e_query_001`; RG-059 `test_BC_2_11_024_iin_integer_elements_rejected_e_query_001`; RG-060 `test_BC_2_11_024_iin_boolean_elements_rejected_e_query_001` | PASS |
| AC-022 | E-QUERY-002: IEQ/IIN/INE on non-string column returns QueryTypeMismatch with suggested_column | RGT (prism-query + prism-core) | RG-018 `test_S_PRISMQL_CASE_INSENSITIVE_001_ieq_integer_column_e_query_002`; RG-029 `test_BC_2_11_024_query_type_mismatch_display_with_suggestion_exact`; RG-030 `test_BC_2_11_024_query_type_mismatch_display_without_suggestion_exact`; RG-041/042 SqlPipe pipe-stage E-QUERY-002; RG-081 `test_rg081_obs002_suggested_suffix_display_some_and_none` (SuggestedSuffix Display lock — Some/None variants produce correct output; ADV-PR-P1-OBS-002) | PASS |
| AC-023 | SQL-mode IEQ/IIN/INE rejection — structured E-QUERY-001 | RGT (prism-query + prism-mcp) | RG-023/024/025 SQL-mode SELECT; RG-037/038/039 DML DELETE/UPDATE/INSERT; RG-046 all prompt-embedded queries parse Ok; RG-058 SqlPipe head WHERE IEQ rejected | PASS |
| AC-024 | PrismQL grammar reference resource includes IEQ/IIN/INE in operator table | RGT (prism-query + prism-mcp) | RG-026 `test_S_PRISMQL_CASE_INSENSITIVE_001_grammar_resource_includes_ieq_iin_ine` (prism-query); RG-043 `test_BC_2_11_024_reference_content_no_stale_vendor_cased_enum_examples` (prism-mcp) | PASS |
| AC-025 | prism describe output includes IEQ example with OCSF casing note in example_note field | RGT (prism-mcp + prism-query) | RG-028 `test_BC_2_11_024_describe_output_includes_ieq_example_and_ocsf_casing_note`; RG-027 `test_S_PRISMQL_CASE_INSENSITIVE_001_describe_output_includes_ieq_example` (supplementary, prism-query); RG-040 suppression guard; RG-051 all describe outputs parse Ok; RG-061/062/063 example_query purity + example_note contract; RG-067/068/069/070 query-tool description + pipe-mode skeleton; RG-072 severity-Integer-type gate | PASS |
| AC-026 | No panic: IEQ/IIN expressions with multiple predicates (VP-021 regression) | RGT (prism-query) | RG-015 `test_S_PRISMQL_CASE_INSENSITIVE_001_repeated_ieq_no_panic` | PASS |
| AC-027 | Non-exhaustive compile-fail gate count UNCHANGED at 89 | just check + grep | `grep EXPECTED= scripts/check-non-exhaustive.sh` → `EXPECTED=89`; `just check` non-exhaustive gate PASS 89/89 | PASS |

---

## AC-006 / AC-007 Sibling-Site Sweep Evidence

### AC-006: Predicate::Compare construction sites

Command:
```bash
rg 'Predicate::Compare\s*\{' crates/prism-query/src/ --type rust
```

Result: All construction sites in production source (`engine.rs`, `ast.rs`, `materialization.rs`, `sql_parser.rs`, `filter_parser.rs`, `pipe_sql_emitter.rs`) include `case_insensitive: false` (existing callers) or `case_insensitive: true` (new IEQ/INE parse sites). No struct-update syntax (`..`) drops the field. Compilation enforces exhaustiveness on `#[non_exhaustive]` matches.

Sample (engine.rs production sites):
```
case_insensitive: false  -- appears 2 times (engine.rs construction sites)
case_insensitive: false  -- materialization.rs construction sites (4 occurrences)
case_insensitive: true   -- sql_parser.rs (1 IEQ detection parse site)
case_insensitive: true   -- filter_parser.rs (IEQ/INE grammar parse sites)
```

### AC-007: Predicate::In construction sites

Command:
```bash
rg 'Predicate::In\s*\{' crates/prism-query/src/ --type rust
```

Result: All construction sites in `filter_parser.rs` (IN + NOT IN + IIN grammar), `ast.rs` (normalizer), `materialization.rs` (test helper), `sql_parser.rs` include `case_insensitive` field. Compilation enforces correctness via `#[non_exhaustive]` match arms.

---

## AC-027 Non-Exhaustive Gate Evidence

```bash
grep EXPECTED= scripts/check-non-exhaustive.sh
```

Output: `EXPECTED=89`

`just check` non-exhaustive gate output:
```
Verifying #[non_exhaustive] forward-compat enforcement (expected: 89 violations)...
PASS: 89 types correctly reject external construction (expected: 89)
```

No new `#[non_exhaustive]`-annotated public types were introduced by this story (verified: `case_insensitive: bool` is a new field on existing variants `Predicate::Compare` and `Predicate::In`, not new types).

---

## Red Gate Test Inventory — Full Coverage

All 81 RGTs verified GREEN via targeted `cargo nextest run` invocations and the `just check` workspace gate.

### Cluster A: Parser acceptance (AC-001 to AC-005, AC-026)
All tests in `crates/prism-query/src/tests/test_case_insensitive_operators.rs`:
```
RG-001  test_S_PRISMQL_CASE_INSENSITIVE_001_ieq_parses_to_compare_case_insensitive_true      PASS
RG-002  test_S_PRISMQL_CASE_INSENSITIVE_001_iin_parses_to_in_case_insensitive_true           PASS
RG-003  test_S_PRISMQL_CASE_INSENSITIVE_001_ine_parses_to_compare_ne_case_insensitive_true   PASS
RG-004  test_S_PRISMQL_CASE_INSENSITIVE_001_ieq_keyword_case_insensitive_parsing             PASS
RG-005  test_S_PRISMQL_CASE_INSENSITIVE_001_iin_before_in_no_collision                      PASS
RG-015  test_S_PRISMQL_CASE_INSENSITIVE_001_repeated_ieq_no_panic                           PASS
```

### Cluster B: DataFusion emitter (AC-008 to AC-011)
```
RG-006  test_S_PRISMQL_CASE_INSENSITIVE_001_ieq_emits_lower_equals_lower                    PASS
RG-007  test_S_PRISMQL_CASE_INSENSITIVE_001_ine_emits_lower_ne_lower                        PASS
RG-008  test_S_PRISMQL_CASE_INSENSITIVE_001_iin_emits_lower_in_lower_list                   PASS
RG-009  test_S_PRISMQL_CASE_INSENSITIVE_001_case_sensitive_eq_no_lower_wrapping             PASS
RG-045  test_BC_2_11_024_ieq_predicate_excluded_from_equality_pushdown                      PASS (pushdown.rs)
RG-073  test_BC_2_11_024_f_p21_obs001_explain_ieq_iin_not_classified_pushdownable           PASS (explain.rs)
RG-074  test_BC_2_11_024_f_p24_med001_valid_operators_string_includes_ci_operators          PASS (engine.rs)
```

### Cluster C: Execution semantics (AC-012, AC-013, AC-013b, AC-014, AC-015)
```
RG-010  test_S_PRISMQL_CASE_INSENSITIVE_001_ieq_execution_case_insensitive_match            PASS
RG-011  test_S_PRISMQL_CASE_INSENSITIVE_001_case_sensitive_eq_returns_zero_on_casing_mismatch PASS
RG-012  test_S_PRISMQL_CASE_INSENSITIVE_001_ieq_in_pipe_where_stage                        PASS
RG-013  test_S_PRISMQL_CASE_INSENSITIVE_001_normalized_pql_reflects_ieq_uppercase           PASS
RG-014  test_S_PRISMQL_CASE_INSENSITIVE_001_normalized_pql_round_trip_ast_equality          PASS
RG-031  test_BC_2_11_024_negated_case_insensitive_in_returns_query_plan_failed              PASS
RG-048  test_low1_negated_iin_emits_invalid_marker_not_plain_positive_iin                  PASS (ast.rs)
RG-049  test_low2_detect_ci_operator_gt_mentions_real_op_not_ieq                           PASS (sql_parser.rs)
RG-050  test_BC_2_11_003_sql_in_literal_list_parses_to_predicate_in                        PASS (parser_tests.rs)
```

### Cluster D: Adapter-boundary OCSF normalization — PRIMARY path (AC-016 to AC-019)
All tests in `crates/prism-bin/src/spec_driven_adapter.rs` test module:
```
RG-032  test_BC_2_02_013_build_column_array_normalizes_severity_to_title_case               PASS
RG-033  test_BC_2_02_013_build_column_array_normalizes_status_and_disposition               PASS
RG-034  test_BC_2_02_013_build_column_array_unrecognized_left_as_received_with_warn         PASS
RG-035  test_BC_2_02_013_build_column_array_non_enum_string_column_untouched                PASS
RG-036  test_BC_2_02_013_build_column_array_non_string_column_untouched                    PASS
RG-047  test_BC_2_02_013_build_column_array_empty_string_enum_value_no_warn                 PASS
RG-071  test_BC_2_02_013_build_column_array_group_by_severity_cross_sensor_no_fragmentation PASS
```

### Cluster E: Adapter-boundary OCSF normalization — SECONDARY path (prism-ocsf)
All tests in `crates/prism-ocsf/src/tests/test_adapter_normalization.rs`:
```
RG-019  test_S_PRISMQL_CASE_INSENSITIVE_001_adapter_normalization_critical_to_title_case    PASS
RG-020  test_S_PRISMQL_CASE_INSENSITIVE_001_adapter_normalization_idempotent_high           PASS
RG-021  test_S_PRISMQL_CASE_INSENSITIVE_001_adapter_normalization_unrecognized_value_left_as_received PASS
RG-052  test_obs4_ocsf_enum_map_new_no_collisions_in_production_data                       PASS (bc_2_02_010_enum_map.rs)
RG-053  test_obs4_collision_detection_panics_on_ambiguous_captions                         PASS (bc_2_02_010_enum_map.rs)
RG-054  test_BC_2_02_013_normalizer_secondary_empty_string_enum_value_no_warn              PASS
```

### Cluster F: GROUP BY de-fragmentation (AC-019)
```
RG-022  test_S_PRISMQL_CASE_INSENSITIVE_001_group_by_severity_no_case_fragmentation        PASS (prism-query)
RG-044  test_BC_2_02_013_triage_alerts_prompt_no_stale_vendor_casing                       PASS (prism-mcp)
RG-071  (listed under Cluster D above — PRIMARY cross-sensor GROUP BY guard)               PASS
```

### Cluster G: Error handling (AC-020, AC-021, AC-022)
```
RG-016  test_S_PRISMQL_CASE_INSENSITIVE_001_ieq_non_string_rhs_e_query_001                 PASS
RG-017  test_S_PRISMQL_CASE_INSENSITIVE_001_iin_empty_list_e_query_001                     PASS
RG-018  test_S_PRISMQL_CASE_INSENSITIVE_001_ieq_integer_column_e_query_002                 PASS
RG-029  test_BC_2_11_024_query_type_mismatch_display_with_suggestion_exact                 PASS (prism-core)
RG-030  test_BC_2_11_024_query_type_mismatch_display_without_suggestion_exact              PASS (prism-core)
RG-041  test_S_PRISMQL_CASE_INSENSITIVE_001_ieq_integer_column_sqlpipe_pipe_stage_e_query_002 PASS
RG-042  test_S_PRISMQL_CASE_INSENSITIVE_001_iin_integer_column_sqlpipe_pipe_stage_e_query_002 PASS
RG-055  test_BC_2_11_024_ieq_date_like_rhs_accepted_as_string                             PASS
RG-056  test_BC_2_11_024_ine_date_like_rhs_accepted_as_string                             PASS
RG-057  test_BC_2_11_024_iin_date_like_elements_accepted_as_string                        PASS
RG-059  test_BC_2_11_024_iin_integer_elements_rejected_e_query_001                        PASS
RG-060  test_BC_2_11_024_iin_boolean_elements_rejected_e_query_001                        PASS
RG-064  test_BC_2_11_024_compound_violation_string_literal_precedence                     PASS
RG-065  test_check_ci_column_types_unregistered_table_ok                                  PASS (materialization.rs)
RG-066  test_check_ci_column_types_empty_fields_ok                                        PASS (materialization.rs)
```

### Cluster H: SQL-mode rejection (AC-023)
All tests in `crates/prism-query/src/tests/test_case_insensitive_operators.rs`:
```
RG-023  test_BC_2_11_024_sql_mode_ieq_rejected                                            PASS
RG-024  test_BC_2_11_024_sql_mode_iin_rejected                                            PASS
RG-025  test_BC_2_11_024_sql_mode_ine_rejected                                            PASS
RG-037  test_BC_2_11_024_dml_delete_where_ieq_rejected                                    PASS
RG-038  test_BC_2_11_024_dml_update_where_iin_rejected                                    PASS
RG-039  test_BC_2_11_024_dml_insert_select_where_ine_rejected                             PASS
RG-046  test_BC_2_11_024_all_prompt_embedded_queries_parse                                 PASS (prism-mcp)
RG-058  test_BC_2_11_024_sqlpipe_head_where_ieq_rejected                                  PASS
```

### Cluster I: Discoverability — grammar resource and prism describe (AC-024, AC-025)
```
RG-026  test_S_PRISMQL_CASE_INSENSITIVE_001_grammar_resource_includes_ieq_iin_ine          PASS (prism-query)
RG-027  test_S_PRISMQL_CASE_INSENSITIVE_001_describe_output_includes_ieq_example           PASS (prism-query, supplementary)
RG-028  test_BC_2_11_024_describe_output_includes_ieq_example_and_ocsf_casing_note         PASS (prism-mcp, authoritative)
RG-040  test_f_med1_severity_vocabulary_table_ieq_not_suppressed_by_integer_column         PASS (prism-mcp)
RG-043  test_BC_2_11_024_reference_content_no_stale_vendor_cased_enum_examples             PASS (prism-mcp)
RG-051  test_obs2_all_build_example_query_outputs_parse_ok                                 PASS (prism-mcp)
RG-061  test_BC_2_10_012_example_query_contains_no_comment_lines                          PASS (prism-mcp)
RG-062  test_BC_2_10_012_build_example_note_query_parses_without_stripping                 PASS (prism-mcp)
RG-063  test_BC_2_10_012_example_note_some_for_severity_tables_none_otherwise              PASS (prism-mcp)
RG-067  test_BC_2_10_009_query_tool_description_no_vendor_casing_teaches_ieq               PASS (prism-mcp)
RG-068  test_RG_067_query_tool_pipe_mode_example_uses_ieq_not_equals                      PASS (prism-mcp server.rs)
RG-069  test_obs2_sentinel_alerts_with_severity_gets_ieq_example                          PASS (prism-mcp)
RG-070  test_obs2_claroty_devices_no_severity_column_still_no_ieq_note                    PASS (prism-mcp)
RG-072  test_f_p20_low001_severity_integer_type_does_not_get_ieq                          PASS (prism-mcp)
```

### Cluster J: Pre-PR-LEVEL and PR-LEVEL pass-1 fix-burst additions (RG-075..081)
Adapter normalization guards, emitter safety, and error display (commits 54c89898, 56fb83d8, f9be96fa):
```
RG-075  test_cr002_normalize_enum_label_already_canonical_returns_some           PASS (prism-ocsf/src/normalizer.rs; CR-002 idempotence no-op write guard → AC-017)
RG-076  test_cr003_normalize_predicate_invalid_ci_op_emits_warn                  PASS (prism-query/src/tests/test_case_insensitive_operators.rs; CR-003 warn before IEQ fallback → AC-008)
RG-077  test_cr004_build_column_array_enum_label_warn_strips_control_chars       PASS (prism-bin/src/spec_driven_adapter.rs; CR-004/SEC-001 CWE-117 PRIMARY → AC-018)
RG-078  test_cr004_sanitize_for_log_strips_control_chars_for_secondary_site      PASS (prism-ocsf/src/normalizer.rs; CR-004/SEC-001 CWE-117 SECONDARY → AC-018)
RG-079  test_rg079_secondary_sanitize_enum_label_order_spec_wins                 PASS (prism-ocsf/src/normalizer.rs; ADV-PR-P1-MED-001 sanitize-before-truncate SECONDARY → AC-018)
RG-080  test_rg080_low001_build_column_array_enum_label_warn_order_of_operations  PASS (prism-bin/src/spec_driven_adapter.rs; ADV-PR-P1-MED-001/LOW-001 order-of-ops PRIMARY → AC-018)
RG-081  test_rg081_obs002_suggested_suffix_display_some_and_none                 PASS (prism-core/src/error.rs; ADV-PR-P1-OBS-002 SuggestedSuffix Display lock → AC-022)
```

---

## Summary

- **All 27 ACs: PASS**
- **All 81 Red Gate tests: PASS** (RG-001 through RG-081)
- **just check workspace gate: 5317/5317 tests PASS, 0 failures**
- **Non-exhaustive gate: 89/89 PASS, count unchanged**
- **Evidence method**: test-transcript captures (cargo nextest run targeted invocations) per prior-story precedent (PLUGIN-MIGRATION-001-A); no live DTU/sensor session required — all ACs demonstrable via in-process unit tests and integration tests
- **AC-006 and AC-007**: compile-enforced (struct field exhaustiveness) + verified via grep sweep
- **AC-027**: verified via grep of `scripts/check-non-exhaustive.sh` + just check gate
- **Convergence provenance**: LOCAL 3-CLEAN converged at de89b557 (passes 33/34/35, CLEAN strict; BC-5.39.001 satisfied); pre-PR-LEVEL fix-burst 54c89898 (CR-002/CR-003/CR-004/SEC-001; RG-075..078); PR-LEVEL pass-1 fix-burst 56fb83d8+f9be96fa (ADV-PR-P1-MED-001/MED-002/LOW-001/OBS-002; RG-079..081). PR-LEVEL pass-1 streak reset to 0/3 per DRIFT-ORCH-PRLEVEL-PUSH-001; cascade re-gates on frozen HEAD f9be96fa
