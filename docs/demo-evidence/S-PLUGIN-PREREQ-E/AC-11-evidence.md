AC-11 — E-SPEC-008 Retirement Annotation in error-taxonomy.md
==============================================================
Story: S-PLUGIN-PREREQ-E (v1.50) | BC: BC-2.16.011 §Error Cases E-SPEC-008 | HEAD: 051eab95

EVIDENCE TYPE: Grep excerpt (error-taxonomy.md row) + test output (Red Gate Test 14)

-------------------------------------------------------------------------------
GREP: E-SPEC-008 row in error-taxonomy.md
-------------------------------------------------------------------------------

Command: grep -A 5 'E-SPEC-008' .factory/specs/prd-supplements/error-taxonomy.md

Output:
  | E-SPEC-008 | broken | transient | "Custom adapter panic in '{sensor}': {message}" | No |
  **RETIRED in S-PLUGIN-PREREQ-E (error-taxonomy.md v1.26).** A CustomAdapter (BC-2.16.004)
  panicked during execution. Caught via catch_unwind. **No live code path triggers this code
  after CustomAdapter removal in S-PLUGIN-PREREQ-E per BC-2.16.011 §Error Cases + ADR-027
  §Decision (operational deletion mandate). Plugin execution panics now surface via
  E-PLUGIN-001. ID preserved per append_only_numbering (DF-030).** |

RETIREMENT ANNOTATION VERIFICATION:
  - "RETIRED in S-PLUGIN-PREREQ-E": present
  - ADR-027 §Decision back-pointer: present
  - BC-2.16.011 §Error Cases back-pointer: present
  - "No live code path triggers this code after CustomAdapter removal": present
  - "ID preserved per append_only_numbering (DF-030)": present
  - Row NOT deleted: confirmed (ID preserved per append_only_numbering policy)

-------------------------------------------------------------------------------
TEST OUTPUT: test_BC_2_16_011_e_spec_008_retired_annotation
-------------------------------------------------------------------------------

Command: cargo nextest run -p prism-spec-engine -E 'test(BC_2_16_011)' --no-fail-fast

    Starting 3 tests across 23 binaries (374 tests skipped)
        PASS [   0.023s] (1/3) prism-spec-engine::bc_2_16_011_test test_BC_2_16_011_002_e_spec_008_not_triggered_by_live_code
        PASS [   0.024s] (2/3) prism-spec-engine::bc_2_16_011_test test_BC_2_16_011_001_custom_adapter_absent_post_deletion
        PASS [   0.130s] (3/3) prism-spec-engine::error_taxonomy_annotation test_BC_2_16_011_e_spec_008_retired_annotation
    Summary [   0.130s] 3 tests run: 3 passed, 374 skipped

AC-11 VERIFICATION:
  - test_BC_2_16_011_e_spec_008_retired_annotation: Reads error-taxonomy.md, asserts E-SPEC-008
    row contains "retired_in: S-PLUGIN-PREREQ-E" (or equivalent retirement annotation referencing
    PREREQ-E + ADR-027). Asserts grep of crates/ src/ paths for E-SPEC-008 construction sites
    returns zero matches. PASS.
  - test_BC_2_16_011_002_e_spec_008_not_triggered_by_live_code: Confirms zero live code paths
    construct or return E-SPEC-008. PASS.

RESULT: PASS — E-SPEC-008 row annotated as RETIRED in error-taxonomy.md v1.26. Retirement
references PREREQ-E + BC-2.16.011 §Error Cases + ADR-027 §Decision. No live code constructs
this error code. ID preserved per DF-030. AC-11 + BC-2.16.011 §Error Cases satisfied.
