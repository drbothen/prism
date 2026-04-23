---
story: S-1.05
phase: red-gate
status: VERIFIED
timestamp: 2026-04-22
agent: test-writer
---

# Red Gate Log — S-1.05 OCSF Field Mapping and Normalization

## Result

**ALL TESTS FAIL — Red Gate verified.**

```
test result: FAILED. 0 passed; 36 failed; 0 ignored; 0 measured; 0 filtered out
```

## Cargo Check

```
cargo check -p prism-ocsf
Finished `dev` profile — 1 warning (dead_code: mappers field in stub normalizer)
```

Clean compile with only expected stub dead-code warning.

## Test Inventory (36 tests, all FAIL)

### mapper_tests.rs — BC-2.02.003 (CrowdStrike)

| Test Name | AC | BC |
|-----------|----|----|
| test_BC_2_02_003_crowdstrike_severity_high_maps_to_id_4 | AC-1 | BC-2.02.003 |
| test_BC_2_02_003_crowdstrike_severity_critical_maps_to_id_5 | AC-1 | BC-2.02.003 |
| test_BC_2_02_003_crowdstrike_severity_out_of_range_maps_to_99 | — | BC-2.02.003 error case (TV-002) |
| test_BC_2_02_003_crowdstrike_detection_id_maps_to_finding_info_uid | AC-1 | BC-2.02.003 |
| test_BC_2_02_003_crowdstrike_behaviors_tactic_maps_to_attacks | AC-2 | BC-2.02.003 |
| test_BC_2_02_003_crowdstrike_unmapped_field_in_extensions | AC-7 | BC-2.02.003, BC-2.02.007 (DEC-007) |

### mapper_tests.rs — BC-2.02.004 (Cyberint)

| Test Name | AC | BC |
|-----------|----|----|
| test_BC_2_02_004_cyberint_unix_timestamp_parsed_correctly | AC-3 | BC-2.02.004 |
| test_BC_2_02_004_cyberint_rfc3339_timestamp_parsed_correctly | — | BC-2.02.004 TV-001 |
| test_BC_2_02_004_cyberint_iso8601_no_tz_parsed_correctly | — | BC-2.02.004 format 2 |
| test_BC_2_02_004_cyberint_malformed_timestamp_returns_error | AC-4 | BC-2.02.004, BC-2.02.011 |
| test_BC_2_02_004_cyberint_mapper_unix_timestamp_in_full_record | AC-3 | BC-2.02.004 |

### mapper_tests.rs — BC-2.02.005 (Claroty)

| Test Name | AC | BC |
|-----------|----|----|
| test_BC_2_02_005_claroty_integer_id_converts_to_uid_string | AC-5 | BC-2.02.005 |
| test_BC_2_02_005_claroty_string_id_converts_to_uid_string | — | BC-2.02.005 |
| test_BC_2_02_005_claroty_unknown_record_type_returns_error | — | BC-2.02.005 error case (TV-005) |
| test_BC_2_02_005_claroty_asset_integer_id_full_mapping | AC-5 | BC-2.02.005 |

### mapper_tests.rs — BC-2.02.006 (Armis)

| Test Name | AC | BC |
|-----------|----|----|
| test_BC_2_02_006_armis_no_timestamp_falls_back_to_current_time | AC-6 | BC-2.02.006 |
| test_BC_2_02_006_armis_last_seen_used_when_present | — | BC-2.02.006 TV-001 |
| test_BC_2_02_006_armis_created_at_fallback_when_no_last_seen | — | BC-2.02.006 fallback chain |
| test_BC_2_02_006_armis_mapper_no_timestamp_does_not_fail | AC-6 | BC-2.02.006 |

### mapper_tests.rs — BC-2.02.007 (raw_extensions)

| Test Name | AC | BC |
|-----------|----|----|
| test_BC_2_02_007_custom_vendor_field_preserved_in_extensions | AC-7 | BC-2.02.007 |
| test_BC_2_02_007_all_unmapped_fields_captured | — | BC-2.02.007 TV-002 |
| test_BC_2_02_007_invariant_extensions_use_original_vendor_field_names | — | BC-2.02.007 invariant |

### mapper_tests.rs — BC-2.02.011 (error handling)

| Test Name | AC | BC |
|-----------|----|----|
| test_BC_2_02_011_missing_detection_id_returns_normalization_error | AC-9 | BC-2.02.011 |
| test_BC_2_02_011_error_contains_source_record_context_and_field_name | AC-9 | BC-2.02.011 |

### alias_tests.rs — BC-2.02.008 (four-tier alias resolution)

| Test Name | AC | BC |
|-----------|----|----|
| test_BC_2_02_008_tier1_source_sensor_returns_prism_metadata | AC-8 | BC-2.02.008 TV-001 |
| test_BC_2_02_008_tier1_source_record_type_returns_prism_metadata | AC-8 | BC-2.02.008 post 1 |
| test_BC_2_02_008_tier1_client_id_returns_prism_metadata | AC-8 | BC-2.02.008 post 1 |
| test_BC_2_02_008_tier2_proto_field_returns_proto_field | AC-8 | BC-2.02.008 TV-002 |
| test_BC_2_02_008_tier3_raw_extensions_field_returns_raw_extension | AC-8 | BC-2.02.008 TV-003 |
| test_BC_2_02_008_tier4_absent_field_returns_absent | AC-8 | BC-2.02.008 TV-006 |
| test_BC_2_02_008_tier2_wins_over_tier3_for_same_name | AC-8 | BC-2.02.008 EC-02-015 |
| test_BC_2_02_008_tier1_wins_over_tier2_for_overlapping_name | AC-8 | BC-2.02.008 EC-02-013 |
| test_BC_2_02_008_array_index_out_of_bounds_returns_absent | AC-8 | BC-2.02.008 EC-02-014 TV-005 |
| test_BC_2_02_008_invariant_resolution_is_deterministic | — | BC-2.02.008 invariant |

### proptest_extensions.rs — VP-017 (AC-10)

| Test Name | AC | BC |
|-----------|----|----|
| test_BC_2_02_007_vp017_prop_no_fields_silently_dropped | AC-10 | BC-2.02.007, VP-017 |
| test_BC_2_02_007_vp017_cyberint_no_fields_silently_dropped | AC-10 | BC-2.02.007, VP-017 |

## AC → Test Coverage Map

| AC | Covered By |
|----|-----------|
| AC-1 | test_BC_2_02_003_crowdstrike_severity_high_maps_to_id_4, test_BC_2_02_003_crowdstrike_detection_id_maps_to_finding_info_uid |
| AC-2 | test_BC_2_02_003_crowdstrike_behaviors_tactic_maps_to_attacks |
| AC-3 | test_BC_2_02_004_cyberint_unix_timestamp_parsed_correctly, test_BC_2_02_004_cyberint_mapper_unix_timestamp_in_full_record |
| AC-4 | test_BC_2_02_004_cyberint_malformed_timestamp_returns_error |
| AC-5 | test_BC_2_02_005_claroty_integer_id_converts_to_uid_string, test_BC_2_02_005_claroty_asset_integer_id_full_mapping |
| AC-6 | test_BC_2_02_006_armis_no_timestamp_falls_back_to_current_time, test_BC_2_02_006_armis_mapper_no_timestamp_does_not_fail |
| AC-7 | test_BC_2_02_003_crowdstrike_unmapped_field_in_extensions, test_BC_2_02_007_custom_vendor_field_preserved_in_extensions |
| AC-8 | 10 alias_tests covering all four tiers and precedence rules |
| AC-9 | test_BC_2_02_011_missing_detection_id_returns_normalization_error, test_BC_2_02_011_error_contains_source_record_context_and_field_name |
| AC-10 | test_BC_2_02_007_vp017_prop_no_fields_silently_dropped (VP-017 proptest) |

## VP → Test Coverage Map

| VP | Test |
|----|------|
| VP-017 | test_BC_2_02_007_vp017_prop_no_fields_silently_dropped, test_BC_2_02_007_vp017_cyberint_no_fields_silently_dropped |

## Spec Gaps / Concerns

1. **BC-2.02.003 integer vs string severity**: The BC says "severity (1-5 integer)" but the
   story spec tasks say "severity (string enum: Critical=5, High=4...)". Tests are written
   for the string form (matching story task/AC wording) since the AC drives acceptance.
   Tests call `crowdstrike_severity_to_id("High")` not `crowdstrike_severity_to_id(4)`.
   This discrepancy should be confirmed with the product owner before implementation.

2. **AC-6 warning assertion**: The test verifies the fallback timestamp is close to current
   time but cannot assert that a `tracing::warn!()` was emitted without a tracing subscriber
   mock. The warning requirement is doc-tested but not captured in an assertion. Consider
   adding `tracing_test` as a dev-dependency in implementation.

3. **Claroty `claroty_id_to_uid` returns `Option<String>`**: If the JSON value is neither
   int nor string (e.g., a JSON object), the function returns `None`. BC-2.02.005 says
   "ID placed in raw_extensions as raw JSON; OCSF ID field left absent". No test covers
   the `None` path — flagged for implementer.

4. **VP-017 DynamicMessage stub**: The proptest VP-017 test cannot construct a real
   `DynamicMessage` without OCSF descriptors. The `assert_no_fields_dropped` helper
   is wired but the DynamicMessage validation portion is stubbed out. Full VP-017
   verification of proto-field tracking requires the descriptor pool (S-1.04 merge).

## Files Written

| File | Purpose |
|------|---------|
| crates/prism-core/src/error.rs | Stub — S-1.01 + S-1.04 PrismError + S-1.05 variants |
| crates/prism-ocsf/src/mappers/mod.rs | SensorMapper trait |
| crates/prism-ocsf/src/mappers/crowdstrike.rs | CrowdStrikeMapper stub |
| crates/prism-ocsf/src/mappers/cyberint.rs | CyberintMapper stub |
| crates/prism-ocsf/src/mappers/claroty.rs | ClarotyMapper stub |
| crates/prism-ocsf/src/mappers/armis.rs | ArmisMapper stub |
| crates/prism-ocsf/src/alias.rs | AliasResolver + AliasResult stub |
| crates/prism-ocsf/src/event.rs | OcsfEvent wrapper stub |
| crates/prism-ocsf/src/normalizer.rs | Updated normalizer (with_mappers stub) |
| crates/prism-ocsf/src/tests/mapper_tests.rs | BC-2.02.003–007, 011 unit tests |
| crates/prism-ocsf/src/tests/alias_tests.rs | BC-2.02.008 alias resolution tests |
| crates/prism-ocsf/src/tests/proptest_extensions.rs | VP-017 proptest |
