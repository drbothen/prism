AC-3b — Runtime Auth-Composition Rejection: Multiple credential_refs (E-SPEC-013)
====================================================================================
Story: S-PLUGIN-PREREQ-E (v1.50) | BC: BC-2.01.016 §Error Cases E-SPEC-013 | HEAD: 051eab95

EVIDENCE TYPE: Test output (BC-2.01.016 Red Gate Tests 4 + VP-153 proptest Rule B)

-------------------------------------------------------------------------------
TEST OUTPUT: test_BC_2_01_016_e_spec_013_multiple_credential_refs_rejected
-------------------------------------------------------------------------------

Command: cargo nextest run -p prism-spec-engine -E 'test(BC_2_01_016)' --no-fail-fast

    PASS [   0.012s] (2/7) prism-spec-engine::bc_2_01_016_test test_BC_2_01_016_e_spec_013_multiple_credential_refs_rejected
    PASS [   0.014s] (7/7) prism-spec-engine::bc_2_01_016_test test_BC_2_01_016_e_spec_013_rejected_via_parse_spec_directory

Both tests PASS.

-------------------------------------------------------------------------------
VP-153 PROPTEST: prop_rule_b_multi_credential_refs_rejected_with_e_spec_013
-------------------------------------------------------------------------------

    PASS [   0.024s] prism-spec-engine::vp153_sensorauth_cross_composition prop_rule_b_credential_count_boundary
    PASS [   0.024s] prism-spec-engine::vp153_sensorauth_cross_composition prop_rule_b_single_credential_ref_accepted
    PASS [   0.029s] prism-spec-engine::vp153_sensorauth_cross_composition prop_rule_b_multi_credential_refs_rejected_with_e_spec_013

AC-3b VERIFICATION:
  - test_BC_2_01_016_e_spec_013_multiple_credential_refs_rejected:
    SensorSpec with [[sensor.credential_refs]] declared twice for the same auth method
    is rejected at spec-load with E-SPEC-013 (multiple credential_refs; exactly one required).
  - test_BC_2_01_016_e_spec_013_rejected_via_parse_spec_directory: Same rejection via
    parse_spec_directory code path.
  - VP-153 proptest prop_rule_b_multi_credential_refs_rejected_with_e_spec_013:
    Arbitrary multi-credential-ref configurations rejected with E-SPEC-013 (Rule B enforcement).
  - VP-153 proptest prop_rule_b_credential_count_boundary: Boundary-condition coverage
    at credential_count = 1 (accepted) vs 2+ (rejected).

Error variant: SpecEngineError::MultipleCredentialRefs { sensor_id, credential_count }
BC anchor: BC-2.01.016 §Error Cases E-SPEC-013
ADR reference: ADR-023 §Architectural Constraints Rule 2, Rule B

RESULT: PASS — E-SPEC-013 validation active. Multiple credential_refs per auth method
rejected at spec-load time per ADR-023 Rule 2, Rule B. Credential values never appear
in error output (AD-017 redacted Debug impl verified by review).
