# Evidence Report — S-ADR058-OCSF-ROUTING-001

**Story:** ADR-058 Stage 2 — OCSF Field-Name Routing  
**Branch:** feature/S-ADR058-OCSF-ROUTING-001  
**HEAD at recording:** 8aeaf06c4  
**Recorded:** 2026-08-23  
**Gate status:** LOCAL 3-CLEAN adversarial convergence passed; story-level holdout gate passed

---

## Coverage Matrix

| AC | Description | Evidence File(s) | Tests Run | Result |
|----|-------------|-----------------|-----------|--------|
| AC-001 | `ocsf_column_naming` field added to `SensorSpec` with `serde(default)` | `AC-001-002-ocsf-flag-and-flattening` (.gif/.webm) + transcript `AC-001-002.txt` | `test_sensor_spec_ocsf_column_naming_defaults_to_false`, `test_sensor_spec_ocsf_column_naming_parses_true_from_toml` | PASS |
| AC-002 | `ocsf_field_to_arrow_name` helper replaces dots with underscores | `AC-001-002-ocsf-flag-and-flattening` (.gif/.webm) + transcript `AC-001-002.txt` | `test_ocsf_field_to_arrow_name_replaces_dots_with_underscores`, `test_ocsf_field_to_arrow_name_empty_and_no_dots` | PASS |
| AC-003 | `pipeline_result_to_record_batch` uses flattened names when `ocsf_column_naming=true` | `AC-003-004-pipeline-routing-branches` (.gif/.webm) + transcript `AC-003-004-012.txt` | `test_pipeline_result_to_record_batch_ocsf_flag_true_uses_flattened_names` | PASS |
| AC-004 | `pipeline_result_to_record_batch` uses `col.name` when `ocsf_column_naming=false` | `AC-003-004-pipeline-routing-branches` (.gif/.webm) + transcript `AC-003-004-012.txt` | `test_pipeline_result_to_record_batch_ocsf_flag_false_uses_col_name` | PASS |
| AC-005 | `claroty.sensor.toml` gains `ocsf_column_naming = true` + 12 KF corrections (KF-01..KF-12) + OQ-005 | `AC-005-claroty-toml-corrections` (.gif/.webm) + transcript `AC-005-toml.txt` | grep shows `ocsf_column_naming = true`, 4 `ocsf_class` values, 27 `ocsf_field` entries | PASS |
| AC-006 | `prism_describe` returns OCSF-flattened names in `ColumnDescriptor.name`; `.description` uses dotted OCSF path | `AC-006-007-prism-describe-tier1-tier2` (.gif/.webm) + transcript `AC-006-007b-015.txt` | `test_prism_describe_ocsf_column_naming_true_returns_flattened_name_and_dotted_description` | PASS |
| AC-007 | `prism_describe` emits exactly ONE `raw_extensions` `ColumnDescriptor`; no individual Tier-2 descriptors | `AC-006-007-prism-describe-tier1-tier2` (.gif/.webm) + transcript `AC-006-007b-015.txt` | `test_prism_describe_ocsf_column_naming_true_raw_extensions_descriptor_and_no_phantom_col_names` | PASS |
| AC-007c | Multi-valued array source fields serialized as compact JSON-list strings in `raw_extensions` | `AC-007c-014-special-behaviors` (.gif/.webm) + transcript `AC-007c-014.txt` | `test_claroty_devices_ip_list_in_raw_extensions_is_compact_json_list_string` | PASS |
| AC-008 | `prism_describe` emits synthesized `class_uid` and `_sensor` `ColumnDescriptors` last | `AC-006-007-prism-describe-tier1-tier2` (.gif/.webm) + transcript `AC-006-007b-015.txt` | `test_prism_describe_ocsf_column_naming_true_emits_class_uid_and_sensor_descriptors` | PASS |
| AC-009 | `class_selector` wired for `entity_management` (3004) and `inventory_info` (5001) on Claroty tables; wire-shape assertions | `AC-009-class-selector-entity-management` (.gif/.webm) + transcript `AC-009.txt` | `test_class_selector_entity_management_and_inventory_info_arms`, `test_class_selector_claroty_audit_log_select_arm_maps_to_entity_management_3004`, `test_class_selector_armis_audit_log_maps_to_entity_management_3004`, `test_claroty_audit_logs_record_batch_class_uid_is_3004`, `test_claroty_devices_record_batch_class_uid_is_5001_regression_guard` | PASS |
| AC-010 | KF-01..KF-12 wire-shape corrections: `finding_info.*`, reserved→`raw_extensions`, `kf11_category`, `metadata_uid`, `device_type_label` | `AC-010-wire-shape-kf-corrections` (.gif/.webm) + transcript `AC-010.txt` | 6 wire-shape tests including `test_claroty_alerts_finding_info_fields_wire_shape`, `test_claroty_device_alert_relations_record_batch_finding_info_uid_wire_shape`, `test_claroty_audit_logs_id_produces_metadata_uid_top_level_arrow_field`, etc. | PASS |
| AC-011 | Unknown `ocsf_class` value emits `ocsf.zero_tier1_table` WARN and returns `Ok(class_uid=0)` | `AC-011-013-error-paths` (.gif/.webm) + transcript `AC-011-013.txt` | `test_pipeline_result_to_record_batch_unknown_ocsf_class_emits_warn` | PASS |
| AC-012 | `pipeline_result_to_record_batch` gains `sensor_spec: &SensorSpec` as explicit parameter | `AC-003-004-pipeline-routing-branches` (.gif/.webm) + transcript `AC-003-004-012.txt` | `test_pipeline_result_to_record_batch_sensor_spec_parameter_gates_both_branches` | PASS |
| AC-013 | `validate_ocsf_column_collisions` (Validation Rule 8) rejects `[§J1]`, `[§J2]`, `[§J4]` with `E-SPEC-030` | `AC-011-013-error-paths` (.gif/.webm) + `AC-021-spec-load-collision-rejection` (.gif/.webm) + transcript `AC-011-013.txt` + `AC-021.txt` | `test_pipeline_result_to_record_batch_ocsf_field_flattens_to_reserved_name_returns_error`, `test_pipeline_result_to_record_batch_ocsf_shadow_collision_returns_error`, `test_pipeline_result_to_record_batch_ocsf_collision_returns_error` + 4 spec-engine collision tests | PASS |
| AC-014 | `extract_time_window_from_ast` recognizes `ocsf_field_to_arrow_name` result as INDEX-eligible | `AC-007c-014-special-behaviors` (.gif/.webm) + transcript `AC-007c-014.txt` | `test_extract_time_window_from_ast_recognizes_ocsf_flattened_time_column_as_index_eligible` | PASS |
| AC-015 | `prism_describe` column names agree with `ocsf_projected_column_names` helper | `AC-006-007-prism-describe-tier1-tier2` (.gif/.webm) + transcript `AC-006-007b-015.txt` | `test_RG_Q_015_prism_describe_names_agree_with_projection_helper` | PASS |
| AC-016 | `E-QUERY-038` gate rejects unavailable columns using OCSF-flattened names; raw `col.name` rejected for OCSF tables | `AC-016-018-query-column-resolution` (.gif/.webm) + transcript `AC-016-017-018.txt` | `test_BC_2_11_016_RG_Q_001_ocsf_select_finding_info_uid_passes_e_query_038`, `test_BC_2_11_016_RG_Q_004_raw_colname_description_rejected_post_ocsf_fix` + 9 more | PASS |
| AC-017 | `SELECT` with OCSF-flattened names (`finding_info_uid`, `metadata_uid`) succeeds | `AC-016-018-query-column-resolution` (.gif/.webm) + transcript `AC-016-017-018.txt` | `test_BC_2_11_016_RG_Q_002_ocsf_select_finding_info_title_passes_e_query_038`, `test_BC_2_11_016_RG_Q_003_ocsf_where_message_passes_e_query_038` + others | PASS |
| AC-018 | `available_columns` in `E-QUERY-038` error lists OCSF-flattened names; multi-tenant head + pipe stage agree | `AC-016-018-query-column-resolution` (.gif/.webm) + transcript `AC-016-017-018.txt` | `test_BC_2_11_016_RG_Q_006_ocsf_error_available_columns_wire_shape`, `test_BC_2_11_016_RG_Q_008_multitenant_ocsf_head_projection`, `test_BC_2_11_016_RG_Q_009_multitenant_ocsf_pipe_stage` | PASS |
| AC-019 | Zero-Tier-1 OCSF table (no `ocsf_field`) registers `class_uid` + `_sensor`; A+W sub-case also adds `raw_extensions` + WARN | `AC-019-020-zero-col-and-projection-invariant` (.gif/.webm) + transcript `AC-019-020.txt` | `test_BC_2_11_016_zero_col_ocsf_table_st_gate_accepts_class_uid_and_sensor`, `test_BC_2_11_016_zero_col_ocsf_table_st_gate_rejects_raw_col_name`, `test_BC_2_11_016_zero_tier1_with_tier2_projects_raw_extensions_and_emits_warning` | PASS |
| AC-020 | `ocsf_projected_column_names` + `ocsf_projected_column_types` is the single authoritative impl; registry, engine, `prism_describe`, and pipeline all agree | `AC-019-020-zero-col-and-projection-invariant` (.gif/.webm) + transcript `AC-019-020.txt` | `test_ocsf_projected_names_all_surfaces_agree` | PASS |
| AC-021 | `validate_ocsf_column_collisions` at spec-load rejects all four collision classes with `E-SPEC-030` | `AC-021-spec-load-collision-rejection` (.gif/.webm) + transcript `AC-021.txt` | `test_BC_2_16_003_ocsf_collision_j1_shadow_tier1_vs_tier1_rejected_at_spec_load`, `test_BC_2_16_003_ocsf_collision_j2_reserved_name_rejected_at_spec_load`, `test_BC_2_16_003_ocsf_collision_j4_intra_table_duplicate_rejected_at_spec_load`, `test_BC_2_16_003_ocsf_collision_j1_shadow_rejected_at_spec_load` | PASS |

---

## Summary

- **21 / 21 acceptance criteria demonstrated**  
- **0 acceptance criteria blocked or undemo-able**  
- **10 VHS recordings** (GIF + WebM), covering success and error paths  
- **11 pre-captured transcript files** for deterministic backup evidence  
- **Total Red Gate tests confirmed passing:** 46 (RG-001..RG-028 across 5 crates, RG-Q-001..RG-Q-017 in prism-query, RG-PD-001 pushdown)

---

## Recording File Index

| Recording | GIF | WebM | Tape Source |
|-----------|-----|------|-------------|
| AC-001-002 | `AC-001-002-ocsf-flag-and-flattening.gif` | `AC-001-002-ocsf-flag-and-flattening.webm` | `AC-001-002-ocsf-flag-and-flattening.tape` |
| AC-003-004 | `AC-003-004-pipeline-routing-branches.gif` | `AC-003-004-pipeline-routing-branches.webm` | `AC-003-004-pipeline-routing-branches.tape` |
| AC-005 | `AC-005-claroty-toml-corrections.gif` | `AC-005-claroty-toml-corrections.webm` | `AC-005-claroty-toml-corrections.tape` |
| AC-006-007-008-015 | `AC-006-007-prism-describe-tier1-tier2.gif` | `AC-006-007-prism-describe-tier1-tier2.webm` | `AC-006-007-prism-describe-tier1-tier2.tape` |
| AC-007c-014 | `AC-007c-014-special-behaviors.gif` | `AC-007c-014-special-behaviors.webm` | `AC-007c-014-special-behaviors.tape` |
| AC-009 | `AC-009-class-selector-entity-management.gif` | `AC-009-class-selector-entity-management.webm` | `AC-009-class-selector-entity-management.tape` |
| AC-010 | `AC-010-wire-shape-kf-corrections.gif` | `AC-010-wire-shape-kf-corrections.webm` | `AC-010-wire-shape-kf-corrections.tape` |
| AC-011-013 | `AC-011-013-error-paths.gif` | `AC-011-013-error-paths.webm` | `AC-011-013-error-paths.tape` |
| AC-016-017-018 | `AC-016-018-query-column-resolution.gif` | `AC-016-018-query-column-resolution.webm` | `AC-016-018-query-column-resolution.tape` |
| AC-019-020 | `AC-019-020-zero-col-and-projection-invariant.gif` | `AC-019-020-zero-col-and-projection-invariant.webm` | `AC-019-020-zero-col-and-projection-invariant.tape` |
| AC-021 | `AC-021-spec-load-collision-rejection.gif` | `AC-021-spec-load-collision-rejection.webm` | `AC-021-spec-load-collision-rejection.tape` |

---

## Transcript File Index

Pre-captured deterministic evidence (exact test runner output):

| Transcript | ACs Covered |
|-----------|-------------|
| `transcripts/AC-001-002.txt` | AC-001, AC-002 |
| `transcripts/AC-003-004-012.txt` | AC-003, AC-004, AC-012 |
| `transcripts/AC-005-toml.txt` | AC-005 |
| `transcripts/AC-006-007b-015.txt` | AC-006, AC-007, AC-008, AC-015 |
| `transcripts/AC-007c-014.txt` | AC-007c, AC-014 |
| `transcripts/AC-009.txt` | AC-009 |
| `transcripts/AC-010.txt` | AC-010 |
| `transcripts/AC-011-013.txt` | AC-011, AC-013 (prism-bin error paths) |
| `transcripts/AC-016-017-018.txt` | AC-016, AC-017, AC-018 |
| `transcripts/AC-019-020.txt` | AC-019, AC-020 |
| `transcripts/AC-021.txt` | AC-021 (spec-engine collision rejection) |

---

## Key Behaviors Demonstrated

1. **OCSF flag + flattening** (AC-001/002): `ocsf_column_naming` defaults `false`; `ocsf_field_to_arrow_name("finding_info.uid")` → `"finding_info_uid"`.

2. **Pipeline routing branch** (AC-003/004): `flag=true` produces Arrow field named `finding_info_uid`; `flag=false` preserves raw `col.name`.

3. **claroty.sensor.toml corrections** (AC-005): `ocsf_column_naming = true`; `entity_management` + `inventory_info` `ocsf_class` values; 27 `ocsf_field` annotations including KF-07 `finding_info.uid`, OQ-005 `metadata.uid`, §J3 shadow fix `device.type_category`.

4. **prism_describe Tier-1/Tier-2 model** (AC-006/007/008/015): Flattened names in descriptors; single `raw_extensions` descriptor enumerating source keys; synthesized `class_uid` + `_sensor` last; name agreement with `ocsf_projected_column_names`.

5. **class_selector wiring** (AC-009): `entity_management` → 3004, `inventory_info` → 5001; wire-shape confirmed at record-batch serialization level.

6. **KF wire-shape corrections** (AC-010): `finding_info.*` fields correctly flattened; `status`/`severity`/`category` reserved names routed to `raw_extensions`; `audit_logs.id` → `metadata_uid` top-level Arrow field; `device_type` → `device_type_label`.

7. **Error paths** (AC-011/013): Unknown `ocsf_class` → `Ok(class_uid=0)` + WARN (not panic); reserved/shadow/duplicate `ocsf_field` → `Err` before record emission; all four collision classes at spec-load → `E-SPEC-030`.

8. **E-QUERY-038 OCSF column resolution** (AC-016/017/018): Raw `col.name` rejected for OCSF tables; flattened names pass; `available_columns` in error lists OCSF-flattened names; multi-tenant head + pipe stage consistent.

9. **Zero-Tier-1 + projection invariant** (AC-019/020): Tables with no `ocsf_field` register `class_uid` + `_sensor`; A+W tables also register `raw_extensions` + emit WARN; all surfaces agree on projected name set.

10. **Spec-load collision rejection** (AC-021): Four collision categories (`§J1` Tier-1 shadow, `§J2` reserved, `§J4` intra-table duplicate, `§J1` Tier-1-vs-Tier-2 shadow) all produce `E-SPEC-030` at `add_sensor_spec` time.
