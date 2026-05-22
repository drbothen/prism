---
document_type: red-gate-log
story_id: PLUGIN-MIGRATION-001-D
phase: 3
cycle: v1.0.0-greenfield
authored_by: test-writer
timestamp: "2026-05-21T00:00:00Z"
red_gate_status: VERIFIED
---

# Red Gate Log — PLUGIN-MIGRATION-001-D

**Story:** Author 4 Production TOML Sensor Specs — Reverse-Engineered + DTU-Parity Tests  
**Date:** 2026-05-21  
**Branch:** `feature/PLUGIN-MIGRATION-001-D`  
**Commits:** 60081cb5, d6b197fa

---

## Red Gate Verification Summary

**Total failing/compile-failing tests:** 5 (RG-03 x2, RG-09 x3 compile-errors)  
**Ignored tests (DTU parity):** 7 (RG-04 x2, RG-05 x2, RG-06 x1, RG-07 x2)  
**Passing tests (structural/regression gates):** 16  
**Red Gate VERIFIED:** YES — no test passes vacuously; all load-bearing assertions are anchored to real BC clauses

---

## Individual Test Status

| RG # | Test Name | File | Status | Failure Class |
|------|-----------|------|--------|---------------|
| RG-01 | test_BC_2_16_001_loads_4_bundled_specs_at_boot | bc_2_16_001_bundled_spec_load.rs | PASSES | Skeleton sufficient — structural gate |
| RG-01 | test_BC_2_16_001_bundled_specs_produce_canonical_table_namespaces | bc_2_16_001_bundled_spec_load.rs | PASSES | Skeleton sufficient — structural gate |
| RG-01 | test_BC_2_16_001_bundled_specs_declare_correct_auth_types | bc_2_16_001_bundled_spec_load.rs | PASSES | Skeleton has correct auth_types |
| RG-01 | test_BC_2_16_001_invalid_spec_does_not_block_valid_specs | bc_2_16_001_bundled_spec_load.rs | PASSES | DI-030 already implemented |
| RG-01 | test_BC_2_16_001_empty_credential_scenario_not_an_error | bc_2_16_001_bundled_spec_load.rs | PASSES | DEC-036 already implemented |
| RG-02 | test_BC_2_16_009_validates_all_4_bundled_specs | bc_2_16_009_bundled_spec_validation.rs | PASSES | Skeleton parses cleanly |
| RG-02 | test_BC_2_16_009_crowdstrike_rate_limit_hints_valid | bc_2_16_009_bundled_spec_validation.rs | PASSES | Skeleton declares rate_limit_hints |
| RG-02 | test_BC_2_16_009_crowdstrike_spec_has_3_tables | bc_2_16_009_bundled_spec_validation.rs | PASSES | Skeleton has 3 tables |
| RG-02 | test_HS_017_BC_2_16_009_invalid_column_type_rejected | bc_2_16_009_bundled_spec_validation.rs | PASSES | Serde already enforces this |
| RG-02 | test_HS_017_BC_2_16_009_missing_sensor_id_rejected | bc_2_16_009_bundled_spec_validation.rs | PASSES | Serde already enforces this |
| RG-03 | test_BC_2_16_002_pipeline_executor_runs_crowdstrike_two_step_spec | bc_2_16_002_crowdstrike_two_step.rs | **FAILS** | Assertion-fail: body template interpolation for ${query_detection_ids.detection_ids} not found in step response |
| RG-03 | test_BC_2_16_002_crowdstrike_batch_boundary_100_ids_one_batch | bc_2_16_002_crowdstrike_two_step.rs | **FAILS** | Assertion-fail: same variable interpolation issue |
| RG-04 | test_BC_2_16_013_dtu_parity_crowdstrike | parity/crowdstrike.rs | IGNORED | #[ignore] — DTU-EXT-001..004 / S-6.07 not merged |
| RG-04 | test_BC_2_16_013_dtu_parity_crowdstrike_batch_cap_100_ids | parity/crowdstrike.rs | IGNORED | #[ignore] — same |
| RG-05 | test_BC_2_16_013_dtu_parity_claroty | parity/claroty.rs | IGNORED | #[ignore] — S-6.08 not merged |
| RG-05 | test_BC_2_16_013_dtu_parity_claroty_polymorphic_integer_id_normalized_to_string | parity/claroty.rs | IGNORED | #[ignore] — same |
| RG-06 | test_BC_2_16_013_dtu_parity_cyberint | parity/cyberint.rs | IGNORED | #[ignore] — S-6.09 not merged |
| RG-06 | test_BC_2_16_013_dtu_parity_cyberint_incidents_explicit_skip | parity/cyberint.rs | **PASSES** | Explicit SKIP assertion — NOT #[ignore] per EC-016-013-002 |
| RG-06 | test_HS_015_BC_2_16_013_cyberint_spec_declares_cookie_roundtrip_auth | parity/cyberint.rs | **PASSES** | Skeleton has correct auth_type |
| RG-07 | test_BC_2_16_013_dtu_parity_armis | parity/armis.rs | IGNORED | #[ignore] — S-6.10 not merged |
| RG-07 | test_BC_2_16_013_dtu_parity_armis_timestamp_fallback_pass_by_convention | parity/armis.rs | IGNORED | #[ignore] — same |
| RG-07 | test_HS_016_BC_2_16_013_armis_spec_declares_bearer_static_auth | parity/armis.rs | **PASSES** | Skeleton has correct auth_type |
| RG-08 | test_BC_2_16_012_plugin_dispatch_uses_spec_catalog_not_hardcoded_names | bc_2_16_012_open_dispatch_bundled_specs.rs | PASSES | No hardcoded dispatch exists yet |
| RG-08 | test_BC_2_16_012_spec_parse_is_idempotent_for_crowdstrike | bc_2_16_012_open_dispatch_bundled_specs.rs | PASSES | Parse is deterministic |
| RG-08 | test_BC_2_16_012_open_dispatch_handles_all_4_bundled_sensors | bc_2_16_012_open_dispatch_bundled_specs.rs | PASSES | Open dispatch works with skeleton |
| RG-09 | test_BC_2_16_001_RG_09_filename_stem_mismatch_emits_E_SPEC_017 | bc_2_16_013_spec_id_mismatch.rs | **COMPILE-FAIL** | E0599: SpecErrorCode::ESpec017 variant not found — Task 11 required |
| RG-09 | test_HS_018_BC_2_16_001_different_sensor_name_mismatch_rejected | bc_2_16_013_spec_id_mismatch.rs | **COMPILE-FAIL** | E0599: same |
| RG-09 | test_HS_018_BC_2_16_001_correct_filename_sensor_id_pair_no_error | bc_2_16_013_spec_id_mismatch.rs | **COMPILE-FAIL** | E0599: same |

---

## Analysis

### Tests That Pass In Red Gate State (Structural Gates)

Tests for RG-01, RG-02, and RG-08 pass because:
1. The skeleton TOML files are production-grade (the VSDD canonical principle forbids intentionally broken artifacts)
2. The spec infrastructure (SpecLoader, AuthType enum) was implemented in prior stories (PREREQ-A..E)
3. These tests are structural regression gates — they verify the framework accepts valid specs

These tests are load-bearing: they will FAIL if the implementer accidentally introduces a parsing regression, adds a hardcoded dispatch arm, or changes auth_type values.

### Tests That Fail In Red Gate State (Blocking Implementation Gates)

**RG-03 (assertion-fail):** The two-step pipeline test fails because the body template interpolation syntax `${query_detection_ids.detection_ids}` is not yet wired correctly for the crowdstrike spec's step variable forwarding. This is the primary behavioral Red Gate for the pipeline implementation (Tasks 3, 5).

**RG-09 (compile-fail):** The `SpecErrorCode::ESpec017` variant does not exist yet. This is the canonical Red Gate for Tasks 11 and 12. Three tests in `bc_2_16_013_spec_id_mismatch.rs` fail to compile, which prevents `cargo nextest run -p prism-spec-engine` from running at all until the variant is added.

### DTU Parity Tests (Intentionally Ignored)

RG-04..07 are tagged `#[ignore]` per EC-016-013-006 (DTU clone stories S-6.07..6.10 not yet merged). The test bodies are complete and load-bearing (per TD-VSDD-059). When the DTU stories merge, remove the `#[ignore]` tags to activate full VP-148 verification.

---

## Implementer Checklist (from test failures)

To make the Red Gate tests pass, the implementer must:

1. **Task 11 (unblocks RG-09):** Add `SpecErrorCode::ESpec017` variant + `#[non_exhaustive]` to `prism-core/src/error.rs`. Also add the unit test per story §Task 11.
2. **Task 12 (unblocks RG-09 assertions):** Add filename-stem-vs-sensor_id check in `SpecLoader::load_all()` emitting `E-SPEC-017`.
3. **Tasks 3-6 (refines RG-01/02/03):** Author production-grade TOML spec content — full column schemas, OCSF field mappings, correct step variable names (e.g., change `variables_produced = ["detection_ids"]` to match what PipelineExecutor extracts from the response path).
4. **Task 10a (enables RG-04..07):** Record reference OCSF fixture JSON when DTU stories merge (S-6.07..6.10).
5. **Tasks 1/Option A or B (enables RG-03 full):** Implement grammar extensions for `timestamp_format` (Cyberint) and `timestamp_fallback_chain` (Armis).
