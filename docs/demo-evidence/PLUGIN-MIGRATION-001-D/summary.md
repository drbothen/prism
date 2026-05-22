# Demo Evidence Summary — PLUGIN-MIGRATION-001-D

Story: Author 4 production TOML sensor specs (v1.21)
Branch: feature/PLUGIN-MIGRATION-001-D
HEAD SHA: 55b4f72daf3514599a87cd31866bc361e43fc1d6
Captured: 2026-05-22T08:05:20Z
Evidence recorder: demo-recorder agent

## AC -> Test -> Status Table

| AC | Description | Test Name | Evidence File | Status |
|----|-------------|-----------|---------------|--------|
| AC-001 | Load CrowdStrike TOML spec at boot | test_BC_2_16_001_loads_4_bundled_specs_at_boot | AC-001-004-bundled-spec-load.md | PASS |
| AC-002 | Load Cyberint TOML spec at boot | test_BC_2_16_001_loads_4_bundled_specs_at_boot | AC-001-004-bundled-spec-load.md | PASS |
| AC-003 | Load Claroty TOML spec at boot | test_BC_2_16_001_loads_4_bundled_specs_at_boot | AC-001-004-bundled-spec-load.md | PASS |
| AC-004 | Load Armis TOML spec at boot | test_BC_2_16_001_loads_4_bundled_specs_at_boot | AC-001-004-bundled-spec-load.md | PASS |
| AC-002 tables | CrowdStrike spec has 3 tables | test_BC_2_16_009_crowdstrike_spec_has_3_tables | AC-001-004-bundled-spec-load.md | PASS |
| AC-001-004 ns | Canonical table namespaces all 4 sensors | test_BC_2_16_001_bundled_specs_produce_canonical_table_namespaces | AC-001-004-bundled-spec-load.md | PASS |
| AC-005 | Validation of all 4 bundled specs | test_BC_2_16_009_validates_all_4_bundled_specs | AC-005-bundled-spec-validation.md | PASS |
| AC-006 | Empty credential scenario not an error | test_BC_2_16_001_empty_credential_scenario_not_an_error | AC-006-empty-credential-not-error.md | PASS |
| AC-007 | CrowdStrike DTU parity DTU-EXT-001 | test_BC_2_16_013_dtu_parity_crowdstrike #[ignore] | AC-007-010-dtu-ext-ignored-tests.md | DEFERRED S-6.07 |
| AC-008 | Claroty DTU parity DTU-EXT-002 | test_BC_2_16_013_dtu_parity_claroty #[ignore] | AC-007-010-dtu-ext-ignored-tests.md | DEFERRED S-6.08 |
| AC-009 | Cyberint DTU parity DTU-EXT-003 | alerts #[ignore]; incidents SKIP-assertion PASS | AC-007-010-dtu-ext-ignored-tests.md | DEFERRED alerts S-6.09; incidents SKIP PASS |
| AC-010 | Armis DTU parity DTU-EXT-004 | test_BC_2_16_013_dtu_parity_armis #[ignore] | AC-007-010-dtu-ext-ignored-tests.md | DEFERRED S-6.10 |
| AC-011 | Bundled specs declare correct auth types | test_BC_2_16_001_bundled_specs_declare_correct_auth_types | AC-011-auth-types.md | PASS |
| AC-012 | Plugin dispatch uses spec catalog | test_BC_2_16_012_001_spec_parser_no_hardcoded_sensor_dispatch | AC-012-plugin-dispatch-spec-catalog.md | PASS |
| AC-013 | Workspace-wide green gate | just check 3724 tests | AC-013-workspace-green-gate.md | PASS |

## Evidence Files

| File | Covers |
|------|--------|
| AC-001-004-bundled-spec-load.md | AC-001 through AC-004 + table counts + canonical namespaces |
| AC-005-bundled-spec-validation.md | AC-005 |
| AC-006-empty-credential-not-error.md | AC-006 |
| AC-007-010-dtu-ext-ignored-tests.md | AC-007, AC-008, AC-009, AC-010 |
| AC-011-auth-types.md | AC-011 |
| AC-012-plugin-dispatch-spec-catalog.md | AC-012 |
| AC-013-workspace-green-gate.md | AC-013 |

## Deferred ACs

AC-007..010 legitimately deferred per Canonical Principle Rule 3: human-directed (Option B approval
2026-05-21), concrete future dependency (DTU clone crates), attached to specific future stories
S-6.07 through S-6.10. The #[ignore] attribute is the correct deferral mechanism per story
Implementation Discipline TD-VSDD-059.
