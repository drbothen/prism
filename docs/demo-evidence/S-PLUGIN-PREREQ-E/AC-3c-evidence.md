AC-3c — Runtime Auth-Composition Rejection: Credential Type Mismatch (E-SPEC-014)
====================================================================================
Story: S-PLUGIN-PREREQ-E (v1.50) | BC: BC-2.01.016 §Error Cases E-SPEC-014 | HEAD: 051eab95

EVIDENCE TYPE: Test output (BC-2.01.016 Red Gate Tests 5 + VP-153 Rule C + hot-reload path)

-------------------------------------------------------------------------------
TEST OUTPUT: test_BC_2_01_016_e_spec_014_credential_type_mismatch_rejected
-------------------------------------------------------------------------------

Command: cargo nextest run -p prism-spec-engine -E 'test(BC_2_01_016)' --no-fail-fast

    PASS [   0.012s] (3/7) prism-spec-engine::bc_2_01_016_test test_BC_2_01_016_e_spec_014_credential_type_mismatch_rejected
    PASS [   0.014s] (6/7) prism-spec-engine::bc_2_01_016_test test_BC_2_01_016_e_spec_014_rejected_via_hot_reload

Both tests PASS.

-------------------------------------------------------------------------------
FULL BC-2.01.016 TEST SUITE: All 7 tests pass
-------------------------------------------------------------------------------

    Starting 7 tests across 23 binaries (370 tests skipped)
        PASS [   0.011s] (1/7) prism-spec-engine::bc_2_01_016_test test_BC_2_01_016_002_auth_composition_runtime_rejection
        PASS [   0.012s] (2/7) prism-spec-engine::bc_2_01_016_test test_BC_2_01_016_e_spec_013_multiple_credential_refs_rejected
        PASS [   0.012s] (3/7) prism-spec-engine::bc_2_01_016_test test_BC_2_01_016_e_spec_014_credential_type_mismatch_rejected
        PASS [   0.013s] (4/7) prism-spec-engine::bc_2_01_016_test test_BC_2_01_016_e_spec_012_rejected_via_add_sensor_spec_mcp_tool
        PASS [   0.014s] (5/7) prism-spec-engine::bc_2_01_016_test test_BC_2_01_016_e_spec_012_rejected_at_spec_load
        PASS [   0.014s] (6/7) prism-spec-engine::bc_2_01_016_test test_BC_2_01_016_e_spec_014_rejected_via_hot_reload
        PASS [   0.014s] (7/7) prism-spec-engine::bc_2_01_016_test test_BC_2_01_016_e_spec_013_rejected_via_parse_spec_directory
    Summary [   0.014s] 7 tests run: 7 passed, 370 skipped

AC-3c VERIFICATION:
  - test_BC_2_01_016_e_spec_014_credential_type_mismatch_rejected:
    SensorSpec with auth_type = "oauth2_client_credentials" paired with an API-key-shaped
    credential is rejected at credential-resolution time with E-SPEC-014 (structural mismatch
    between declared auth_type and resolved credential shape).
  - test_BC_2_01_016_e_spec_014_rejected_via_hot_reload: Same rejection via hot-reload
    code path (credential-resolution at runtime configuration update).

Error variant: SpecEngineError::AuthTypeCredentialMismatch { sensor_id, expected_shape, actual_shape }
BC anchor: BC-2.01.016 §Error Cases E-SPEC-014
ADR reference: ADR-023 §Architectural Constraints Rule 2, Rule C

RESULT: PASS — E-SPEC-014 validation active. Auth-type/credential structural mismatch
rejected at credential-resolution time per ADR-023 Rule 2, Rule C. Custom Debug impl
redacts credential values (AD-017 compliance).
