AC-7 — spec_parser.rs Open Dispatch: No Hardcoded Sensor Names in Dispatch Context
====================================================================================
Story: S-PLUGIN-PREREQ-E (v1.50) | BC: BC-2.16.012 | HEAD: 051eab95

EVIDENCE TYPE: Grep (one doc-comment match only) + test output (BC-2.16.012 Test 8)

-------------------------------------------------------------------------------
GREP: Sensor name strings in spec_parser.rs
-------------------------------------------------------------------------------

Command: grep -rn '"crowdstrike"|"cyberint"|"claroty"|"armis"' crates/prism-spec-engine/src/spec_parser.rs

Output:
  crates/prism-spec-engine/src/spec_parser.rs:483:    ///     sensor_id: "crowdstrike".to_string(),

ANALYSIS: The single match at line 483 is in a DOC COMMENT (preceded by `///`), not in
a production dispatch match arm. Doc comments are explicitly acceptable per AC-7: "Sensor
name strings may still appear in doc comments or test fixture values (those are acceptable)."

ZERO matches in production dispatch match-arm contexts.

-------------------------------------------------------------------------------
TEST OUTPUT: test_BC_2_16_012_001_spec_parser_no_hardcoded_sensor_dispatch
-------------------------------------------------------------------------------

Command: cargo nextest run -p prism-spec-engine -E 'test(BC_2_16_012)' --no-fail-fast

    Starting 5 tests across 23 binaries (372 tests skipped)
        PASS [   0.014s] (1/5) prism-spec-engine::bc_2_16_012_test test_BC_2_16_012_002_spec_parser_behavioral_equivalence_crowdstrike
        PASS [   0.014s] (2/5) prism-spec-engine::bc_2_16_012_test test_BC_2_16_012_002_spec_parser_behavioral_equivalence_armis
        PASS [   0.014s] (3/5) prism-spec-engine::bc_2_16_012_test test_BC_2_16_012_002_spec_parser_behavioral_equivalence_claroty
        PASS [   0.014s] (4/5) prism-spec-engine::bc_2_16_012_test test_BC_2_16_012_002_spec_parser_behavioral_equivalence_cyberint
        PASS [   0.015s] (5/5) prism-spec-engine::bc_2_16_012_test test_BC_2_16_012_001_spec_parser_no_hardcoded_sensor_dispatch
    Summary [   0.015s] 5 tests run: 5 passed, 372 skipped

AC-7 VERIFICATION:
  - test_BC_2_16_012_001_spec_parser_no_hardcoded_sensor_dispatch: Calls SpecParser with
    a novel "hypothetical_sensor" TOML SensorSpec and asserts it parses without error.
    This confirms no hardcoded match arm rejects unknown sensors. PASS.
  - Grep confirms zero production dispatch match arms containing literal sensor name strings.

RESULT: PASS — spec_parser.rs uses open dispatch. Hardcoded sensor name match arms are
absent. BC-2.16.012 postcondition (open dispatch; INV-SPEC-PARSER-OPEN-001) satisfied.
