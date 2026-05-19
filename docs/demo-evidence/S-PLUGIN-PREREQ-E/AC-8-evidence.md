AC-8 — Behavioral Equivalence: Four Initial Sensors Parse Identically via Open Dispatch
========================================================================================
Story: S-PLUGIN-PREREQ-E (v1.50) | BC: BC-2.16.012 | HEAD: 051eab95

EVIDENCE TYPE: Test output (BC-2.16.012 Red Gate Tests 9-12 — all four sensors)

-------------------------------------------------------------------------------
TEST OUTPUT: Four behavioral equivalence tests (one per built-in sensor)
-------------------------------------------------------------------------------

Command: cargo nextest run -p prism-spec-engine -E 'test(BC_2_16_012)' --no-fail-fast

    Starting 5 tests across 23 binaries (372 tests skipped)
        PASS [   0.014s] (1/5) prism-spec-engine::bc_2_16_012_test test_BC_2_16_012_002_spec_parser_behavioral_equivalence_crowdstrike
        PASS [   0.014s] (2/5) prism-spec-engine::bc_2_16_012_test test_BC_2_16_012_002_spec_parser_behavioral_equivalence_armis
        PASS [   0.014s] (3/5) prism-spec-engine::bc_2_16_012_test test_BC_2_16_012_002_spec_parser_behavioral_equivalence_claroty
        PASS [   0.014s] (4/5) prism-spec-engine::bc_2_16_012_test test_BC_2_16_012_002_spec_parser_behavioral_equivalence_cyberint
        PASS [   0.015s] (5/5) prism-spec-engine::bc_2_16_012_test test_BC_2_16_012_001_spec_parser_no_hardcoded_sensor_dispatch
    Summary [   0.015s] 5 tests run: 5 passed, 372 skipped

AC-8 VERIFICATION:

  test_BC_2_16_012_002_spec_parser_behavioral_equivalence_crowdstrike: PASS
    Parses crowdstrike.sensor.toml via migrated SpecParser; resulting SensorSpec is identical
    to pre-migration baseline (snapshot comparison).

  test_BC_2_16_012_002_spec_parser_behavioral_equivalence_cyberint: PASS
    Parses cyberint.sensor.toml; SensorSpec identical to baseline.

  test_BC_2_16_012_002_spec_parser_behavioral_equivalence_claroty: PASS
    Parses claroty.sensor.toml; SensorSpec identical to baseline.

  test_BC_2_16_012_002_spec_parser_behavioral_equivalence_armis: PASS
    Parses armis.sensor.toml; SensorSpec identical to baseline.

  test_BC_2_16_012_001_spec_parser_no_hardcoded_sensor_dispatch: PASS
    Novel "hypothetical_sensor" spec parses via generic path without error.

SIBLING SWEEP NOTE:
  grep -rn 'CustomAdapter|custom_adapter' crates/prism-spec-engine/src/spec_parser.rs
  Output: (no matches — exit 0)
  No stale CustomAdapter references remain in spec_parser.rs.

RESULT: PASS — Four built-in sensor TOML specs parse with byte-identical SensorSpec output
before and after dispatch migration. Novel sensor name parses without error via generic path.
BC-2.16.012 invariants INV-SPEC-PARSER-OPEN-002 and INV-SPEC-PARSER-OPEN-003 satisfied.
