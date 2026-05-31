# Demo Evidence Report — S-SPEC-ENV-VAR-001

**Story ID:** S-SPEC-ENV-VAR-001  
**Title:** `${env.VAR}` interpolation resolution in sensor-spec string fields  
**Story version:** v1.2  
**Branch:** feature/S-SPEC-ENV-VAR-001  
**Demo mode:** LIBRARY (test-harness nextest capture)  
**Captured:** 2026-05-31  
**BC:** BC-2.16.009 §Validation Rules 6  

---

## Demo Mode Rationale

`prism-spec-engine` is a Rust library crate with no CLI surface. The `resolve_env_var_tokens`
function is not invokable via a command-line binary. For library features, the production-representative
demo vehicle is the nextest test suite: each Red Gate test exercises the production code path
at the function boundary, sets up realistic inputs (TOML-parsed `SensorSpec`, env vars), and
asserts the exact postconditions specified in the AC. VHS terminal recordings of `cargo nextest`
output are equivalent evidence to VHS recordings of a CLI tool — both show the binary executing
and reporting PASS.

---

## AC → Evidence Mapping

| AC | Description | Evidence File | Test Name | Verdict |
|----|-------------|---------------|-----------|---------|
| AC-001 | Full-token resolution — single token, var set | `AC-001-full-token-resolution.txt` | `test_env_var_full_token_resolves_to_value` | PASS |
| AC-002 | Partial-token interpolation — token within URL prefix/suffix | `AC-002-partial-token-interpolation.txt` | `test_env_var_partial_token_resolves_preserving_surrounding_literals` | PASS |
| AC-003 | Multi-token field — two tokens, both set | `AC-003-multi-token-field.txt` | `test_env_var_multi_token_single_field_both_resolve` | PASS |
| AC-004 | Missing var → E-SPEC-024, spec rejected | `AC-004-missing-var-e-spec-024.txt` | `test_env_var_missing_var_produces_e_spec_024` | PASS |
| AC-005 | Empty var → E-SPEC-024, treated as missing | `AC-005-empty-var-e-spec-024.txt` | `test_env_var_empty_var_produces_e_spec_024` | PASS |
| AC-006 | Multi-error collection — no fail-fast; two missing vars → two errors | `AC-006-multi-error-collection.txt` | `test_env_var_multi_missing_tokens_collect_multiple_errors` | PASS |
| AC-007 | Resolution ordering — resolver runs pre-URL-format-validation | `AC-007-resolution-ordering.txt` | `test_env_var_resolution_runs_before_url_format_validation` + 2 production-path tests | PASS |
| AC-008 | AD-017 no-value-leak — error contains NAME, never VALUE | `AC-008-ad017-no-value-leak.txt` | `test_env_var_error_contains_name_not_value` | PASS |
| (regression) | Full prism-spec-engine suite: 508 tests, 0 failures | `no-regression-full-suite.txt` | All tests | PASS |

---

## AC-008 / AD-017 No-Value-Leak Detail

The AC-008 evidence file (`AC-008-ad017-no-value-leak.txt`) specifically demonstrates that:

1. The sentinel value `"https://secret.internal.sentinel-value-do-not-log"` is NOT present in
   either the `Display` or `Debug` representation of the `E-SPEC-024` error — even after the
   var was set to that sentinel and then unset (simulating prior-session exposure).

2. The `EnvVarNotSet` variant in `error.rs` contains only `var_name`, `toml_path`, and
   `file_path` — no field is capable of holding the resolved value by construction.

3. The `Display` format (pinned by `test_E_SPEC_024_display_matches_error_taxonomy_template_byte_for_byte`
   to error-taxonomy.md v1.56) names the variable (`'ARMIS_INSTANCE_URL'`) twice but
   does not reference any resolved URL value.

4. `env_resolver.rs` construction sites pass only the `var_name` string from the regex
   capture group into `EnvVarNotSet` — `std::env::var()` is used only to branch on
   Ok-vs-Err, and the Ok value is never forwarded to the error.

The test explicitly asserts:
```
format!("{}", err)  contains  "PRISM_TEST_ENV_VAR_AC008_SECRET"   // NAME present
format!("{}", err)  does NOT contain  "secret.internal.sentinel"  // VALUE absent
format!("{:?}", err) does NOT contain  "secret.internal.sentinel" // VALUE absent in Debug
```

---

## Coverage Summary

| Count | Detail |
|-------|--------|
| 8 | Story Red Gate tests (one per AC, in `env_var_resolution_tests.rs`) |
| 2 | Adversary-added production-path ordering tests (F-LOCAL-P1-HIGH-001, AC-007 coverage) |
| 10 | Total env-var-resolution tests, all PASS |
| 508 | Total prism-spec-engine tests in full suite run, 0 failures |
| 10 | Tests skipped (#[ignore] DTU integration tests — external-service gate per SID-1) |

---

## Error Path Coverage

Each AC with a failure mode has both paths recorded:

| AC | Success path | Error path |
|----|-------------|-----------|
| AC-001 | var set → token replaced | n/a (pure success AC) |
| AC-002 | var set → partial replaced | n/a (pure success AC) |
| AC-003 | both vars set → both replaced | n/a (pure success AC) |
| AC-004 | n/a | absent var → EnvVarNotSet |
| AC-005 | n/a | empty var → EnvVarNotSet (empty == absent) |
| AC-006 | n/a | two absent vars → two EnvVarNotSet, no fail-fast |
| AC-007 | var set → resolved URL passes url-format check | absent var → EnvVarNotSet NOT E-SPEC-001 |
| AC-008 | var set → sentinel in spec.base_url (not in errors) | absent var → Display has NAME not VALUE |

---

## Implementation Files

| File | Role |
|------|------|
| `crates/prism-spec-engine/src/env_resolver.rs` | `resolve_env_var_tokens()` — the resolver implementation |
| `crates/prism-spec-engine/src/error.rs` | `SpecEngineError::EnvVarNotSet` variant definition (E-SPEC-024) |
| `crates/prism-spec-engine/src/add_sensor_spec.rs` | `parse_and_validate_spec_toml()` — production load path that calls resolver |
| `crates/prism-spec-engine/tests/env_var_resolution_tests.rs` | All 10 env-var Red Gate tests |
