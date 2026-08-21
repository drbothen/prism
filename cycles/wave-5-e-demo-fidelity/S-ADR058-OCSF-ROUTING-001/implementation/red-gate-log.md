# Red Gate Log — S-ADR058-OCSF-ROUTING-001

**Story:** S-ADR058-OCSF-ROUTING-001 — ADR-058 Stage 2: OCSF field-name routing
**Phase:** 3 (TDD Implementation) — Red Gate Step
**Wave:** wave-5-e-demo-fidelity
**Date:** 2026-08-21
**Author:** test-writer
**Worktree:** .worktrees/S-ADR058-OCSF-ROUTING-001 (branch: feature/S-ADR058-OCSF-ROUTING-001)

---

## Red Gate Status: RED

23 of 27 tests FAIL. 4 tests are GREEN-BY-DESIGN (GBD — stubs already implement those behaviors per story spec). Workspace COMPILES. Red Gate discipline satisfied per BC-5.38.001.

---

## Compile Fixes Applied (Stub Compatibility)

The stub commit added `ocsf_column_naming: bool` to `SensorSpec` but did not backfill existing struct literal constructions in internal crates. To satisfy Red Gate discipline (tests must COMPILE and fail, not fail to compile), the following SensorSpec struct literals were updated with `ocsf_column_naming: false`:

| File | Change |
|------|--------|
| `crates/prism-spec-engine/src/pipeline.rs` | Added `ocsf_column_naming: false,` to 5 SensorSpec struct literals (before `probe_table: None`) |
| `crates/prism-spec-engine/src/proofs/spec_validator.rs` | Added `ocsf_column_naming: false,` to `minimal_valid_spec()` SensorSpec struct literal |

These are mechanical backfills — no behavioral change, no production logic added.

---

## Test Files Written

| File | Tests |
|------|-------|
| `crates/prism-spec-engine/src/spec_parser.rs` — `#[cfg(test)] mod tests` | RG-001, RG-002 |
| `crates/prism-spec-engine/src/column_mapping.rs` — `#[cfg(test)] mod tests` | RG-003, RG-004 |
| `crates/prism-bin/src/spec_driven_adapter.rs` — `#[cfg(test)] mod tests` | RG-005, RG-006, RG-008, RG-009, RG-010, RG-014, RG-015, RG-016, RG-017, RG-018, RG-019, RG-020, RG-021, RG-022, RG-024, RG-026, RG-027 |
| `crates/prism-mcp/src/tools/prism_describe.rs` — `#[cfg(test)] mod ocsf_routing_tests` | RG-007, RG-025 |
| `crates/prism-ocsf/tests/spec_driven_mapper_fixtures.rs` | RG-011, RG-012, RG-023 |
| `crates/prism-ocsf/src/mappers/spec_driven.rs` — `#[cfg(test)] mod tests` | RG-013 |

---

## Red Gate Results

| Test ID | Test Name | Crate | Status | Failure Reason |
|---------|-----------|-------|--------|----------------|
| RG-001 | `test_BC_2_ADR058_001_sensor_spec_parses_ocsf_column_naming_true` | prism-spec-engine | GBD (PASS) | TOML `ocsf_column_naming = true` parses via stub field; behavior already present |
| RG-002 | `test_BC_2_ADR058_001_sensor_spec_defaults_ocsf_column_naming_false` | prism-spec-engine | GBD (PASS) | Default `ocsf_column_naming = false` is present in stub |
| RG-003 | `test_BC_2_ADR058_002_ocsf_field_to_arrow_name_dot_becomes_underscore` | prism-spec-engine | FAIL | `todo!()` panic in stub |
| RG-004 | `test_BC_2_ADR058_002_ocsf_field_to_arrow_name_nested_path` | prism-spec-engine | FAIL | `todo!()` panic in stub |
| RG-005 | `test_BC_2_ADR058_003_pipeline_ocsf_branch_taken_when_flag_set` | prism-bin | FAIL | `todo!()` panic in stub |
| RG-006 | `test_BC_2_ADR058_003_pipeline_standard_branch_taken_when_flag_not_set` | prism-bin | GBD (PASS) | Standard (non-OCSF) branch already implemented |
| RG-007 | `test_BC_2_ADR058_004_describe_tier1_column_names_use_ocsf_names` | prism-mcp | FAIL | `todo!()` panic in single-tenant OCSF describe branch |
| RG-008 | `test_BC_2_ADR058_005_ocsf_arrow_schema_uses_ocsf_field_names` | prism-bin | FAIL | `todo!()` panic in stub |
| RG-009 | `test_BC_2_ADR058_006_ocsf_collision_dot_notation_different_paths_same_flat_name` | prism-bin | FAIL | `todo!()` panic; assertion on `is_err()` |
| RG-010 | `test_BC_2_ADR058_006_ocsf_shadow_collision_reserved_name_vs_ocsf_field` | prism-bin | FAIL | `todo!()` panic; assertion on `is_err()` for shadow case |
| RG-011 | `test_class_selector_entity_management_and_inventory_info_arms` | prism-ocsf | FAIL | `todo!()` panic in `select_by_class_name("entity_management")` |
| RG-012 | `test_class_selector_armis_audit_log_maps_to_entity_management_3004` | prism-ocsf | FAIL | Assertion mismatch: got 3001, expected 3004 |
| RG-013 | `test_claroty_note_comment_not_silently_dropped_under_entity_management` | prism-ocsf | GBD (PASS) | `set_nested_field` already implemented; graceful skip when pool returns None for EntityManagement |
| RG-014 | `test_BC_2_ADR058_007_warn_log_emitted_for_unknown_ocsf_class` | prism-bin | FAIL | `todo!()` panic in stub |
| RG-015 | `test_BC_2_ADR058_007_no_warn_for_known_ocsf_class` | prism-bin | FAIL | `todo!()` panic in stub |
| RG-016 | `test_BC_2_ADR058_008_entity_management_class_uid_3004` | prism-bin | FAIL | `todo!()` panic; Arrow `class_uid` column assertion would fail |
| RG-017 | `test_BC_2_ADR058_008_inventory_info_class_uid_5001` | prism-bin | FAIL | `todo!()` panic; Arrow `class_uid` column assertion would fail |
| RG-018 | `test_BC_2_ADR058_009_category_uid_derived_from_class_uid` | prism-bin | FAIL | `todo!()` panic in stub |
| RG-019 | `test_BC_2_ADR058_010_sensor_column_appended_to_ocsf_batch` | prism-bin | FAIL | `todo!()` panic in stub |
| RG-020 | `test_BC_2_ADR058_011_raw_extensions_column_present_in_ocsf_batch` | prism-bin | FAIL | `todo!()` panic in stub |
| RG-021 | `test_BC_2_ADR058_012_ocsf_batch_column_order_ocsf_fields_first` | prism-bin | FAIL | `todo!()` panic in stub |
| RG-022 | `test_BC_2_ADR058_013_non_ocsf_sensor_unaffected_by_ocsf_routing` | prism-bin | FAIL | `todo!()` panic; non-OCSF path should succeed but stub panics |
| RG-023 | `test_class_selector_claroty_audit_log_select_arm_maps_to_entity_management_3004` | prism-ocsf | FAIL | Assertion mismatch: got 3001, expected 3004 |
| RG-024 | `test_BC_2_ADR058_014_ocsf_field_names_queryable_in_prismql` | prism-bin | FAIL | `todo!()` panic in stub |
| RG-025 | `test_BC_2_ADR058_015_describe_tier2_ocsf_field_column_descriptor_wire_shape` | prism-mcp | FAIL | `todo!()` panic in single-tenant OCSF describe branch |
| RG-026 | `test_BC_2_ADR058_016_ocsf_column_names_include_declared_ocsf_field_names` | prism-bin | FAIL | `todo!()` panic in stub |
| RG-027 | `test_BC_2_ADR058_017_reserved_names_rejected` | prism-bin | FAIL | `todo!()` panic; all 4 reserved-name sub-cases (class_uid, category_uid, _sensor, raw_extensions) assert `is_err()` |

**Summary:** 23 FAIL / 4 GBD / 0 ERROR (no compile failures)

GBD tests (expected per story spec, stubs implement these behaviors):
- RG-001: TOML `ocsf_column_naming = true` parse — stub field present
- RG-002: default `ocsf_column_naming = false` — stub default present
- RG-006: standard (non-OCSF) pipeline branch — already implemented
- RG-013: `set_nested_field` already implemented in prism-ocsf

---

## Contamination Control Confirmation

The holdout scenarios directory (`/Users/jmagady/Dev/prism/.factory/holdout-scenarios/`) was NOT read at any point during Red Gate test authoring. Strict contamination control maintained per story requirements.

---

## Handoff to Implementer

All 27 Red Gate tests are written and verified. The implementer should make each test pass, one at a time, starting with RG-003 (simplest: `ocsf_field_to_arrow_name` dot-to-underscore conversion).

Suggested implementation order (dependency chain):
1. RG-003 / RG-004: `ocsf_field_to_arrow_name` helper (no dependencies)
2. RG-011 / RG-012 / RG-023: OCSF class selector (entity_management=3004, inventory_info=5001)
3. RG-005: pipeline OCSF branch dispatch
4. RG-008: Arrow schema OCSF field names
5. RG-016 / RG-017 / RG-018: class_uid / category_uid injection
6. RG-019 / RG-020 / RG-021: _sensor, raw_extensions, column ordering
7. RG-009 / RG-010 / RG-027: collision detection and reserved-names guard
8. RG-014 / RG-015: WARN log for unknown OCSF class
9. RG-022 / RG-024: non-OCSF passthrough, PrismQL queryability
10. RG-007 / RG-025 / RG-026: `prism_describe` Tier-1/Tier-2 OCSF model
