---
story_id: S-1.13
phase: red-gate
timestamp: "2026-04-22"
status: RED_GATE_VERIFIED
---

# Red Gate Log — S-1.13: Sensor Spec Write Endpoints

## Result: VERIFIED

28 of 29 tests FAIL. 1 test passes (expected — see note below).

## Test Run Summary

```
test result: FAILED. 1 passed; 28 failed; 0 ignored; 0 measured; 0 filtered out
```

## Passing Test (Expected)

| Test | Reason |
|------|--------|
| `test_BC_2_16_009_risk_tier_invalid_string_parse_error` | Tests serde deserialization rejection of `risk_tier = "read"` via the already-implemented `RiskTierSpec` enum. The enum is part of the stub data model, not a stub function. This test correctly validates BC-2.16.009 rule 3 at the type level and will catch regressions. It is NOT vacuously true. |

## Failing Tests (All 28 — Each Fails via `unimplemented!` Panic)

### BC-2.16.001 — WriteEndpointRegistry (8 tests)
- `test_BC_2_16_001_write_registry_get_crowdstrike_contain` — AC-1
- `test_BC_2_16_001_crowdstrike_risk_tier_irreversible` — AC-1
- `test_BC_2_16_001_crowdstrike_batch_limit_ten` — AC-1
- `test_BC_2_16_001_write_table_descriptor_write_only_flag` — AC-4
- `test_BC_2_16_001_write_table_descriptor_sql_table_name` — AC-4
- `test_BC_2_16_001_all_four_sensors_ten_write_verbs` — AC-5
- `test_BC_2_16_001_verbs_for_crowdstrike_returns_four` — AC-5
- `test_BC_2_16_001_registry_is_empty_before_register` — BC-2.16.001 precondition

### BC-2.16.009 — Validation (16 tests)
- `test_BC_2_16_009_reserved_keyword_where_returns_e_spec_011` — AC-2, EC-001
- `test_BC_2_16_009_reserved_keyword_sort_returns_e_spec_011` — EC-001
- `test_BC_2_16_009_reserved_keyword_limit_returns_e_spec_011` — EC-001
- `test_BC_2_16_009_reserved_keyword_join_returns_e_spec_011` — EC-001
- `test_BC_2_16_009_reserved_keyword_enrich_returns_e_spec_011` — EC-001
- `test_BC_2_16_009_reserved_keyword_head_returns_e_spec_011` — EC-001
- `test_BC_2_16_009_non_reserved_verb_passes` — BC-2.16.009 valid path
- `test_BC_2_16_009_batch_limit_zero_irreversible_emits_warning` — AC-3, EC-003
- `test_BC_2_16_009_batch_limit_zero_irreversible_spec_loads` — AC-3, EC-003
- `test_BC_2_16_009_empty_steps_rejected` — EC-004
- `test_BC_2_16_009_record_id_field_uppercase_rejected` — EC-005
- `test_BC_2_16_009_record_id_field_special_chars_rejected` — EC-005
- `test_BC_2_16_009_record_id_field_valid_lowercase` — EC-005 (valid path)
- `test_BC_2_16_009_cross_sensor_verb_uniqueness_collision` — EC-002
- `test_BC_2_16_009_all_errors_collected_no_fail_fast` — VP-059 invariant
- `test_BC_2_16_009_valid_spec_no_errors_no_warnings` — BC-2.16.009 happy path

### S-1.13 Task 6 — Write-side interpolation (4 tests)
- `test_interpolation_record_ids_resolved_in_body_template`
- `test_interpolation_params_key_resolved`
- `test_interpolation_params_key_default_used_when_missing`
- `test_interpolation_url_context_percent_encodes`

## Files Written

| File | Purpose |
|------|---------|
| `crates/prism-spec-engine/src/write_endpoint.rs` | WriteEndpointSpec, WriteStep, BatchMode, WriteEndpointRegistry, WriteTableDescriptor, validate_write_endpoints, check_reserved_keyword, validate_record_id_field — all unimplemented! |
| `crates/prism-spec-engine/src/interpolation.rs` | Extended with interpolate_record_ids, interpolate_write_params — both unimplemented! |
| `crates/prism-spec-engine/tests/write_endpoint_tests.rs` | 29 tests covering all ACs, ECs, and BC clauses |
| `sensors/crowdstrike.sensor.toml` | 4 write endpoints: contain, uncontain, update_status, assign |
| `sensors/cyberint.sensor.toml` | 2 write endpoints: acknowledge, close_alert |
| `sensors/claroty.sensor.toml` | 2 write endpoints: tag, remove_tag |
| `sensors/armis.sensor.toml` | 2 write endpoints: tag, remove_tag |

## BC Coverage

| BC | Clause Type | Tests |
|----|-------------|-------|
| BC-2.16.001 | Postcondition: WriteEndpointRegistry get | test_BC_2_16_001_write_registry_get_crowdstrike_contain |
| BC-2.16.001 | Postcondition: WriteTableDescriptor write_only=true | test_BC_2_16_001_write_table_descriptor_write_only_flag |
| BC-2.16.001 | Postcondition: 10+ write verbs across 4 sensors | test_BC_2_16_001_all_four_sensors_ten_write_verbs |
| BC-2.16.001 | Postcondition: verbs_for_sensor | test_BC_2_16_001_verbs_for_crowdstrike_returns_four |
| BC-2.16.009 | Validation rule 1: reserved keyword rejection | 6 tests (one per reserved keyword) |
| BC-2.16.009 | Validation rule: batch_limit=0+irreversible = warning | test_BC_2_16_009_batch_limit_zero_irreversible_* |
| BC-2.16.009 | Validation rule: empty steps rejected | test_BC_2_16_009_empty_steps_rejected |
| BC-2.16.009 | Validation rule: record_id_field regex | test_BC_2_16_009_record_id_field_* |
| BC-2.16.009 | EC-002: cross-sensor verb collision | test_BC_2_16_009_cross_sensor_verb_uniqueness_collision |
| BC-2.16.009 | VP-059: all-errors-collected | test_BC_2_16_009_all_errors_collected_no_fail_fast |
| BC-2.16.009 | Rule 3: invalid risk_tier rejected at parse | test_BC_2_16_009_risk_tier_invalid_string_parse_error (PASSES — data model) |

## Handoff to Implementer

All 28 failing tests are the implementation targets. Make each pass with minimum code:

1. Implement `WriteEndpointRegistry::register`, `get`, `verbs_for_sensor`, `table_descriptors`, `len`
2. Implement `validate_write_endpoints` with all-errors-collected pass (VP-059)
3. Implement `check_reserved_keyword` returning E-SPEC-011 for RESERVED_KEYWORDS collisions
4. Implement `validate_record_id_field` checking `^[a-z0-9_]+$`
5. Implement `Interpolator::interpolate_record_ids` and `interpolate_write_params`
6. Cross-sensor verb uniqueness enforced in `WriteEndpointRegistry::register`
