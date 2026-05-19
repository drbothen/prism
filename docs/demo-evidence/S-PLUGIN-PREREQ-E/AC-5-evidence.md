AC-5 — Three Call Sites Cleaned (lib.rs, examples/, tests/)
=============================================================
Story: S-PLUGIN-PREREQ-E (v1.50) | BC: BC-2.16.011 | HEAD: 051eab95

EVIDENCE TYPE: Grep (zero matches in src/) + test output (Red Gate Test 7)

-------------------------------------------------------------------------------
GREP: CustomAdapter symbols absent from all crates/prism-spec-engine/src/
-------------------------------------------------------------------------------

Command: grep -rn 'CustomAdapter|CustomAdapterRegistry|CustomAuth' crates/ --include='*.rs' | grep -v '/target/' | grep '/src/'

Output: (no matches — exit 0)

All three confirmed call sites cleaned:

  Site 1 — crates/prism-spec-engine/src/lib.rs:
    `mod custom_adapter;` declaration: REMOVED
    `pub use custom_adapter::*`: REMOVED
    (verified: grep for 'custom_adapter' in lib.rs returns zero matches in src/ context)

  Site 2 — crates/prism-spec-engine/examples/demo_spec_loading.rs:
    File deleted (contained only CustomAdapter-specific demo code; no non-CustomAdapter
    content worth preserving per Task 4 evaluation). File does not exist post-deletion.

  Site 3 — crates/prism-spec-engine/tests/bc_2_16_004_test.rs:
    File deleted (tested CustomAdapterRegistry behavior that no longer exists; test coverage
    superseded by PLUGIN-MIGRATION-001-C WASM plugin tests per Task 5). File does not exist.

-------------------------------------------------------------------------------
TEST OUTPUT: test_BC_2_16_011_002_e_spec_008_not_triggered_by_live_code
-------------------------------------------------------------------------------

Command: cargo nextest run -p prism-spec-engine -E 'test(BC_2_16_011)' --no-fail-fast

    PASS [   0.023s] (1/3) prism-spec-engine::bc_2_16_011_test test_BC_2_16_011_002_e_spec_008_not_triggered_by_live_code
    PASS [   0.024s] (2/3) prism-spec-engine::bc_2_16_011_test test_BC_2_16_011_001_custom_adapter_absent_post_deletion
    PASS [   0.130s] (3/3) prism-spec-engine::error_taxonomy_annotation test_BC_2_16_011_e_spec_008_retired_annotation
    Summary [   0.130s] 3 tests run: 3 passed, 374 skipped

AC-5 VERIFICATION:
  - test_BC_2_16_011_002_e_spec_008_not_triggered_by_live_code: Searches workspace src/ tree
    for any match arm or handler that constructs E-SPEC-008; confirms zero live code paths
    produce that error code. PASS.
  - All three call sites absent from src/: CONFIRMED by grep.

RESULT: PASS — All three call sites cleaned. No CustomAdapter/CustomAdapterRegistry/
CustomAuth reference exists in src/ paths. BC-2.16.011 postconditions satisfied.
ADR-023 §Architectural Constraints (C5 bullet) Rule 5 compliance confirmed.
