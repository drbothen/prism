# Demo Evidence — S-PLUGIN-PREREQ-C (TOML Grammar Extensions + Pub-API Hardening)

**Story:** S-PLUGIN-PREREQ-C v1.3 (status: ready, LOCAL CONVERGED at adversary pass-5)
**HEAD:** c9bb9d26 (post-fix-burst-4 + F-LP5-LOW-001 cleanup)
**Demo recorded:** 2026-05-12
**Convergence trajectory:** 18 → 8 → 5 → 5 → 1 (5 LOCAL passes, vs PREREQ-A 12 + PREREQ-B 16)

## AC Satisfaction Table

| AC | Description | Evidence File | Status |
|----|-------------|---------------|--------|
| AC-1 | page_size on cursor pagination first-call + continuation (TD-B-001) | [AC-1-evidence.md](./AC-1-evidence.md) | SATISFIED |
| AC-2 | JSONPath bracket index + wildcard enumeration + bounds-checked structured error (TD-B-003) | [AC-2-evidence.md](./AC-2-evidence.md) | SATISFIED |
| AC-3 | Proptest fixtures for fan_out, extract_at_path, interpolator totality + bounds + round-trip (TD-B-006) | [AC-3-evidence.md](./AC-3-evidence.md) | SATISFIED |
| AC-4 | Interpolator `$${...}` context-free literal escape mechanism (TD-B-008) | [AC-4-evidence.md](./AC-4-evidence.md) | SATISFIED |
| AC-5 | `#[non_exhaustive]` on 30 pub TOML-deserialized config-input types + CI EXPECTED=30 enforcement (TD-B-016) | [AC-5-evidence.md](./AC-5-evidence.md) | SATISFIED |
| AC-6 | Cross-newtype `*::new_unchecked` audit with symbol-keyed allowlist + OrgSlug validated constructor in materialization.rs (TD-A-006) | [AC-6-evidence.md](./AC-6-evidence.md) | SATISFIED |
| AC-7 | `SensorIdValidationError` crate-root re-export with match-exercising doctest (TD-A-008) | [AC-7-evidence.md](./AC-7-evidence.md) | SATISFIED |

## Red Gate Test Table

| Test Name | File | Crate | Status |
|-----------|------|-------|--------|
| test_BC_2_16_002_cursor_pagination_first_call_includes_page_size | crates/prism-spec-engine/tests/ac_1_cursor_page_size_test.rs | prism-spec-engine | PASS |
| test_BC_2_16_002_cursor_pagination_continuation_includes_page_size | crates/prism-spec-engine/tests/ac_1_cursor_page_size_test.rs | prism-spec-engine | PASS |
| test_BC_2_16_002_cursor_pagination_page_size_none_omitted | crates/prism-spec-engine/tests/ac_1_cursor_page_size_test.rs | prism-spec-engine | PASS |
| test_BC_2_16_002_cursor_pagination_first_call_includes_page_size (in-module) | crates/prism-spec-engine/src/pipeline.rs | prism-spec-engine | PASS |
| test_BC_2_16_002_cursor_pagination_continuation_includes_page_size (in-module) | crates/prism-spec-engine/src/pipeline.rs | prism-spec-engine | PASS |
| test_BC_2_16_002_cursor_pagination_page_size_none_omitted (in-module) | crates/prism-spec-engine/src/pipeline.rs | prism-spec-engine | PASS |
| test_BC_2_16_002_extract_bracket_index | crates/prism-spec-engine/src/pipeline.rs | prism-spec-engine | PASS |
| test_BC_2_16_002_extract_wildcard_enumeration | crates/prism-spec-engine/src/pipeline.rs | prism-spec-engine | PASS |
| test_BC_2_16_002_extract_backward_compat_dot_path | crates/prism-spec-engine/src/pipeline.rs | prism-spec-engine | PASS |
| test_BC_2_16_002_extract_bracket_out_of_bounds_structured_error | crates/prism-spec-engine/src/pipeline.rs | prism-spec-engine | PASS |
| proptest_fan_out_batches_total_count | crates/prism-spec-engine/tests/proptest_AC_3.rs | prism-spec-engine | PASS |
| proptest_fan_out_batches_max_batch_size | crates/prism-spec-engine/tests/proptest_AC_3.rs | prism-spec-engine | PASS |
| proptest_extract_at_path_totality | crates/prism-spec-engine/src/pipeline.rs (in-module) | prism-spec-engine | PASS |
| proptest_interpolate_totality | crates/prism-spec-engine/tests/proptest_AC_3.rs | prism-spec-engine | PASS |
| proptest_extract_references_round_trip | crates/prism-spec-engine/tests/proptest_AC_3.rs | prism-spec-engine | PASS |
| test_BC_2_16_002_interpolator_escape_double_dollar | crates/prism-spec-engine/src/interpolation.rs (in-module) | prism-spec-engine | PASS |
| test_BC_2_16_002_interpolator_live_reference_unaffected | crates/prism-spec-engine/src/interpolation.rs (in-module) | prism-spec-engine | PASS |
| test_BC_2_16_002_interpolator_triple_dollar_escape | crates/prism-spec-engine/src/interpolation.rs (in-module) | prism-spec-engine | PASS |
| test_AC4_escape_context_free_double_dollar_to_single | crates/prism-spec-engine/src/interpolation.rs (in-module) | prism-spec-engine | PASS |
| Compile-fail violation crate (30 violations: 19 E0639 + 11 E0004) | tests/external/non-exhaustive-violation/ | (cross-crate) | E0639+E0004 expected |
| check-non-exhaustive recipe (EXPECTED=30) | Justfile + scripts/count-non-exhaustive-errors.py | (cross-crate) | PASS |
| test_BC_2_01_013_new_unchecked_inventory_baseline | crates/prism-core/tests/new_unchecked_audit.rs | prism-core | PASS |
| SensorIdValidationError doctest (lib.rs line ~147) | crates/prism-core/src/lib.rs | prism-core | PASS |

## Cross-References

- BC-2.16.002 v1.10 (multi-step fetch pipeline + structured event catalog 16 rows including
  jsonpath_extraction_failed + jsonpath_size_cap_exceeded added for AC-2 bounds-check path)
- BC-2.01.013 v1.6 (SensorId open newtype, anchored)
- Adversarial passes 1-5: `.factory/code-delivery/S-PLUGIN-PREREQ-C/adversary-pass-{1,2,3,4,5}.md`
- Tech debt resolved: TD-S-PLUGIN-PREREQ-B-001, TD-S-PLUGIN-PREREQ-B-003, TD-S-PLUGIN-PREREQ-B-006,
  TD-S-PLUGIN-PREREQ-B-008, TD-S-PLUGIN-PREREQ-B-016, TD-S-PLUGIN-PREREQ-A-006, TD-S-PLUGIN-PREREQ-A-008
- CI workflow: `.github/workflows/ci.yml` `non-exhaustive-violation-compile-fail` job (EXPECTED=30)
- Local recipes: `just check-non-exhaustive`, `just check`, `just check-ci`
- Predecessor merges: S-PLUGIN-PREREQ-A (PR #142 develop@ae7e26c8), S-PLUGIN-PREREQ-B (PR #143 develop@ae7e26c8)
