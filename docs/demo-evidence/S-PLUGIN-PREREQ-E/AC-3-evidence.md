AC-3 — Runtime Auth-Composition Rejection (E-SPEC-012)
========================================================
Story: S-PLUGIN-PREREQ-E (v1.50) | BC: BC-2.01.016 | HEAD: 051eab95

EVIDENCE TYPE: Test output (BC-2.01.016 Red Gate Tests 2 + VP-153 proptest)

-------------------------------------------------------------------------------
TEST OUTPUT: test_BC_2_01_016_002_auth_composition_runtime_rejection (prism-spec-engine)
-------------------------------------------------------------------------------

Command: cargo nextest run -p prism-spec-engine -E 'test(BC_2_01_016)' --no-fail-fast

    Starting 7 tests across 23 binaries (370 tests skipped)
        PASS [   0.011s] (1/7) prism-spec-engine::bc_2_01_016_test test_BC_2_01_016_002_auth_composition_runtime_rejection
        PASS [   0.012s] (2/7) prism-spec-engine::bc_2_01_016_test test_BC_2_01_016_e_spec_013_multiple_credential_refs_rejected
        PASS [   0.012s] (3/7) prism-spec-engine::bc_2_01_016_test test_BC_2_01_016_e_spec_014_credential_type_mismatch_rejected
        PASS [   0.013s] (4/7) prism-spec-engine::bc_2_01_016_test test_BC_2_01_016_e_spec_012_rejected_via_add_sensor_spec_mcp_tool
        PASS [   0.014s] (5/7) prism-spec-engine::bc_2_01_016_test test_BC_2_01_016_e_spec_012_rejected_at_spec_load
        PASS [   0.014s] (6/7) prism-spec-engine::bc_2_01_016_test test_BC_2_01_016_e_spec_014_rejected_via_hot_reload
        PASS [   0.014s] (7/7) prism-spec-engine::bc_2_01_016_test test_BC_2_01_016_e_spec_013_rejected_via_parse_spec_directory
    Summary [   0.014s] 7 tests run: 7 passed, 370 skipped

-------------------------------------------------------------------------------
VP-153 PROPTEST: prop_rule_a_invalid_auth_type_rejected_with_e_spec_012
-------------------------------------------------------------------------------

Command: cargo nextest run -p prism-spec-engine --no-fail-fast (VP-153 proptest suite)

    PASS [   0.022s] prism-spec-engine::vp153_sensorauth_cross_composition prop_rule_a_valid_auth_type_accepted
    PASS [   0.033s] prism-spec-engine::vp153_sensorauth_cross_composition prop_rule_a_boundary_whitespace_and_empty_rejected
    PASS [   0.035s] prism-spec-engine::vp153_sensorauth_cross_composition prop_rule_a_invalid_auth_type_rejected_with_e_spec_012
    PASS [   0.024s] prism-spec-engine::vp153_sensorauth_cross_composition prop_rule_b_credential_count_boundary
    PASS [   0.024s] prism-spec-engine::vp153_sensorauth_cross_composition prop_rule_b_single_credential_ref_accepted
    PASS [   0.029s] prism-spec-engine::vp153_sensorauth_cross_composition prop_rule_b_multi_credential_refs_rejected_with_e_spec_013

AC-3 VERIFICATION:
  - test_BC_2_01_016_002_auth_composition_runtime_rejection: SensorSpec with
    auth_type = ["oauth2_client_credentials", "bearer_static"] rejected at spec-load
    with E-SPEC-012 (array value is out-of-set).
  - test_BC_2_01_016_e_spec_012_rejected_at_spec_load: Rejected via SensorSpec::load.
  - test_BC_2_01_016_e_spec_012_rejected_via_add_sensor_spec_mcp_tool: Rejected via MCP tool path.
  - VP-153 proptest prop_rule_a_invalid_auth_type_rejected_with_e_spec_012: Arbitrary
    invalid auth_type strings rejected with E-SPEC-012 (Rule A enforcement).

RESULT: PASS — Runtime cross-composition rejection via E-SPEC-012 is active.
Sealed-trait removal does NOT weaken threat model — rejection moves from compile-time
to runtime enforcement per ADR-026 D3 + BC-2.01.016 Rule 2. INV-AUTH-OPEN-003 holds.
