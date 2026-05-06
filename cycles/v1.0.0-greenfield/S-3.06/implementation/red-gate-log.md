# Red Gate Log — S-3.06 (PrismQL Write Parser Extensions)

## Date
2026-05-06

## Agent
test-writer (gap-fill audit pass)

## Status
RED GATE VERIFIED

## Baseline (stub-architect commit cdcb4b38)
- Tests run: 296
- Passing: 279
- Failing: 17 (all stub-architect scaffolds, all `todo!()` RED)

## After gap-fill audit (this commit)
- Tests run: 370
- Passing: 279 (unchanged — no regressions)
- Failing: 91 (17 stub-architect + 74 new gap-fill)

## New tests added (74 total)

### Integration tests (tests/write_parser_tests.rs) — 45 new

#### Filter-mode rejection per verb (AC-4 extensions) — 4 tests
- `test_BC_2_11_004_filter_mode_rejects_tag_verb_with_args`
- `test_BC_2_11_004_filter_mode_rejects_acknowledge_verb`
- `test_BC_2_11_004_filter_mode_field_named_contain_is_not_rejected`
- `test_BC_2_11_004_filter_mode_empty_registry_no_panic`

#### Internal prism table protection — all known tables (AC-3/EC-11-061) — 7 tests
- `test_BC_2_11_004_internal_table_delete_prism_cases`
- `test_BC_2_11_004_internal_table_insert_prism_rules`
- `test_BC_2_11_004_internal_table_update_prism_schedules`
- `test_BC_2_11_004_internal_table_delete_prism_audit`
- `test_BC_2_11_004_internal_table_delete_prism_aliases`
- `test_BC_2_11_004_internal_table_unknown_prism_prefix`
- `test_BC_2_11_004_table_named_prism_no_underscore_is_allowed`

#### Unbounded write protection — INSERT path (EC-11-062) — 3 tests
- `test_BC_2_11_004_insert_select_without_limit_or_where_is_unbounded`
- `test_BC_2_11_004_insert_select_with_where_is_bounded`
- `test_BC_2_11_004_insert_select_with_limit_is_bounded`

#### WriteVerbRegistry trait impls (HashSet path) — 7 tests
- `test_BC_2_11_004_hashset_verb_source_is_registered_verb`
- `test_BC_2_11_004_hashset_verb_source_all_verbs`
- `test_BC_2_11_004_hashset_verb_source_verbs_for_sensor`
- `test_BC_2_11_004_registry_is_write_verb`
- `test_BC_2_11_004_registry_is_empty_populated_vs_default`
- `test_BC_2_11_004_registry_all_verbs_matches_source`
- `test_BC_2_11_004_registry_verbs_for_sensor_unknown_returns_empty`

#### DML statement parsing — positive and negative — 5 tests
- `test_BC_2_11_004_update_multiple_assignments`
- `test_BC_2_11_004_insert_missing_select_is_parse_error`
- `test_BC_2_11_004_delete_missing_from_is_parse_error`
- `test_BC_2_11_004_update_malformed_no_set_clause`
- `test_BC_2_11_004_dml_node_target_table_preserved_exactly`

#### Error constructor message content — 4 tests
- `test_BC_2_11_004_error_010_message_contains_code_and_table`
- `test_BC_2_11_004_error_022_message_contains_code_and_suggestion`
- `test_BC_2_11_004_error_023_message_contains_code_verb_and_suggestions`
- `test_BC_2_11_004_error_024_message_contains_code_verb_and_position`

#### Pipe mode write verb — additional edge cases — 6 tests
- `test_BC_2_11_004_two_write_verbs_in_sequence_rejected`
- `test_BC_2_11_004_pipe_write_verb_case_sensitivity_policy`
- `test_BC_2_11_004_write_stage_no_source_prefix_sensor_is_none`
- `test_BC_2_11_004_write_stage_no_intermediate_stages`
- `test_BC_2_11_004_write_arg_integer_literal`
- `test_BC_2_11_004_write_arg_boolean_literal`

#### Security guard applies to write queries — 1 test
- `test_BC_2_11_004_oversized_write_query_rejected_before_parse`

#### VP-021 corpus extension — 6 additional panic-safety seeds — 6 tests
- `test_vp021_corpus_seed_single_verb_with_one_arg`
- `test_vp021_corpus_seed_update_with_where`
- `test_vp021_corpus_seed_delete_without_where`
- `test_vp021_corpus_seed_internal_table_attempt`
- `test_vp021_corpus_seed_malformed_verb`
- `test_vp021_corpus_seed_insert_values_not_select`

#### Perimeter compliance — 1 test
- `test_BC_2_11_004_write_query_reachable_via_public_entry_point`

#### Property test — 1 test
- `test_BC_2_11_004_proptest_write_node_roundtrip`

### Unit tests (src/tests/write_parser_unit_tests.rs) — 29 new

#### extract_sensor_prefix — 7 tests
- `test_BC_2_11_004_extract_sensor_prefix_underscore_notation`
- `test_BC_2_11_004_extract_sensor_prefix_dotted_notation`
- `test_BC_2_11_004_extract_sensor_prefix_empty_string`
- `test_BC_2_11_004_extract_sensor_prefix_no_separator`
- `test_BC_2_11_004_extract_sensor_prefix_multiple_underscores`
- `test_BC_2_11_004_extract_sensor_prefix_multiple_dots`
- `test_BC_2_11_004_extract_sensor_prefix_leading_underscore_no_panic`

#### is_internal_prism_table — 6 tests
- `test_BC_2_11_004_is_internal_prism_table_prism_alerts`
- `test_BC_2_11_004_is_internal_prism_table_prism_cases`
- `test_BC_2_11_004_is_internal_prism_table_external_table_false`
- `test_BC_2_11_004_is_internal_prism_table_prism_no_suffix_false`
- `test_BC_2_11_004_is_internal_prism_table_any_prism_prefix_true`
- `test_BC_2_11_004_is_internal_prism_table_empty_string_false`

#### check_unbounded_write — 5 tests
- `test_BC_2_11_004_check_unbounded_write_delete_no_where`
- `test_BC_2_11_004_check_unbounded_write_update_no_where`
- `test_BC_2_11_004_check_unbounded_write_delete_with_where_is_safe`
- `test_BC_2_11_004_check_unbounded_write_insert_no_limit_no_where`
- `test_BC_2_11_004_check_unbounded_write_insert_with_limit_is_safe`

#### reject_write_verbs_in_filter — 3 tests
- `test_BC_2_11_004_reject_write_verbs_in_filter_with_verb_in_input`
- `test_BC_2_11_004_reject_write_verbs_in_filter_clean_input_ok`
- `test_BC_2_11_004_reject_write_verbs_in_filter_empty_registry_always_ok`

#### parse_pipe_with_write — 4 tests
- `test_BC_2_11_004_parse_pipe_with_write_happy_path`
- `test_BC_2_11_004_parse_pipe_with_write_unknown_verb`
- `test_BC_2_11_004_parse_pipe_with_write_verb_not_terminal`
- `test_BC_2_11_004_parse_pipe_with_write_empty_registry_any_verb_023`

#### parse_sql_dml — 4 tests
- `test_BC_2_11_004_parse_sql_dml_delete_with_where`
- `test_BC_2_11_004_parse_sql_dml_delete_no_where_022`
- `test_BC_2_11_004_parse_sql_dml_update_prism_table_010`
- `test_BC_2_11_004_parse_sql_dml_not_dml_input_parse_error`

## Cargo check exit code
0 (clean)

## Cargo clippy exit code
Non-zero — but ALL clippy errors are PRE-EXISTING from stub-architect commit cdcb4b38.
No new clippy errors introduced by this gap-fill pass.
Pre-existing errors:
- 8x "function X is never used" (dead_code on stub functions — will resolve when implementer wires them)
- 2x "used expect() on Result" (in existing test helper, not in new gap-fill tests)

Pre-existing errors verified by: `git stash && cargo clippy ... && git stash pop` showed the same errors before and after my changes.

## VP-021 fuzz corpus seeds added
Path: fuzz/corpus/vp021_parse_fuzz/
Seeds added: 10
- write_seed_01_single_verb: `FROM crowdstrike_hosts | where last_seen < 7d | contain`
- write_seed_02_multi_verb_args: `FROM crowdstrike_hosts | where zone = "OT" | tag key="review" value="pending"`
- write_seed_03_dml_delete_with_where: `DELETE FROM armis_device_tags WHERE device_id = '123'`
- write_seed_04_internal_table_attempt: `UPDATE prism_alerts SET status = 'resolved' WHERE id = '1'`
- write_seed_05_malformed_verb: `FROM crowdstrike_hosts | nonexistent_verb_xyz_abc`
- write_seed_06_filter_mode_write_injection: `severity_id >= 4 | contain`
- write_seed_07_insert_into_select: `INSERT INTO crowdstrike_contained_hosts (device_id) SELECT device_id FROM crowdstrike_hosts WHERE last_seen < 7d LIMIT 100`
- write_seed_08_non_terminal_write: `FROM crowdstrike_hosts | contain | where severity >= 3`
- write_seed_09_update_with_where: `UPDATE armis_devices SET status = 'quarantined' WHERE device_id = '42'`
- write_seed_10_delete_unbounded: `DELETE FROM armis_device_tags`

## BC contract gaps flagged for PO follow-up

1. **BC-2.11.004 does not specify case sensitivity of write verbs.** The story says "write verbs are dynamically registered" but does not define whether matching is case-sensitive. Test `test_BC_2_11_004_pipe_write_verb_case_sensitivity_policy` marks this as a contract gap. PO/implementer should clarify: is `CONTAIN` equivalent to `contain`?

2. **BC-2.11.004 does not specify behaviour of `reject_write_verbs_in_filter` when the registry is empty.** Test `test_BC_2_11_004_reject_write_verbs_in_filter_empty_registry_always_ok` documents the expected behaviour (always OK when empty), but the BC does not explicitly state this. Should be added as an invariant to BC-2.11.004.

3. **BC-2.11.004 section on INSERT INTO SELECT unboundedness** is specified in the story task §4 but not in the BC's own Error Cases table. The BC's Error Cases table only mentions E-QUERY-022 implicitly via the story text. For completeness, E-QUERY-022 should be an explicit entry in BC-2.11.004 §Error Cases.

4. **BC-2.11.006 restricted_symbols list** does not yet include S-3.06 new parser symbols (`parse_pipe_with_write`, `build_write_stage_parser`, `build_write_arg_parser`, `extract_sensor_prefix`, `parse_sql_dml`, `build_dml_parser`, `is_internal_prism_table`, `check_unbounded_write`, `reject_write_verbs_in_filter`). Per story Architecture Compliance Rules: "If any symbol added here needs to be listed in BC-2.11.006's `restricted_symbols` frontmatter, flag it for product-owner amendment before merging." All 9 new pub(crate) symbols should be added.

## Red Gate confirmation
All 91 failing tests fail with:
```
panicked at ...: not yet implemented: S-3.06 ...
```
No test passes vacuously. No test fails with a compilation error. The Red Gate is verified.
