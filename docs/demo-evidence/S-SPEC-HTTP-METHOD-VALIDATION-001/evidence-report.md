# Demo Evidence Report — S-SPEC-HTTP-METHOD-VALIDATION-001

**Story:** S-SPEC-HTTP-METHOD-VALIDATION-001 v1.2 — HTTP Method Whitelist Validation in Sensor Spec  
**Wave:** wave-5-e-demo-fidelity  
**BC:** BC-2.16.009 v1.8 §Validation Rules 7  
**Error code:** E-SPEC-025 (error-taxonomy.md v1.59)  
**LOCAL adversary cascade:** CONVERGED 3/3 at commit b1b81cd0  
**Demo mode:** LIBRARY (test-harness nextest capture) — prism-spec-engine is a library crate with no CLI binary surface. Test harness is the production-representative evidence vehicle, following the S-SPEC-ENV-VAR-001 / S-PLUGIN-PREREQ-D precedent.

---

## Coverage Summary

| AC | Description | Evidence File | Red Gate Test | Status |
|----|-------------|---------------|---------------|--------|
| AC-001 | All 7 whitelist methods pass validation | AC-001-valid-http-methods-pass-validation.txt | test_BC_2_16_009_valid_http_method_passes_validation | PASS |
| AC-002 | Invalid method returns structured E-SPEC-025 | AC-002-invalid-method-returns-e-spec-025.txt | test_BC_2_16_009_invalid_http_method_returns_structured_e_spec_025 | PASS |
| AC-003 | Env-resolved method validated post-resolution | AC-003-env-resolved-method-validated-post-resolution.txt | test_BC_2_16_009_env_resolved_invalid_method_caught_post_resolution | PASS |

---

## AC-001 — Valid HTTP Methods Pass Validation

**BC anchor:** BC-2.16.009 v1.8 §Validation Rules 7 — "valid method passes"  
**Test vectors:** §Canonical Test Vectors "HTTP method — valid GET" and "HTTP method — valid POST"

**Red Gate test:** `test_BC_2_16_009_valid_http_method_passes_validation`

**Evidence:** AC-001-valid-http-methods-pass-validation.txt

**Tests demonstrating AC-001:**

| Test name | What it proves |
|-----------|----------------|
| test_BC_2_16_009_valid_http_method_passes_validation | All 7 whitelist values (parameterized) produce zero errors |
| test_BC_2_16_009_allowed_http_methods_has_exactly_7_entries | ALLOWED_HTTP_METHODS constant has exactly 7 values |
| test_BC_2_16_009_allowed_http_methods_contains_canonical_values | All 7 canonical values (GET/POST/PUT/PATCH/DELETE/HEAD/OPTIONS) present |
| test_BC_2_16_009_ec009_010_get_passes_rule_7 | EC-009-010: "GET" passes Rule 7 |
| test_BC_2_16_009_ec009_011_post_passes_rule_7 | EC-009-011: "POST" passes (Claroty/Armis POST-for-read pattern) |
| test_BC_2_16_009_ec009_017_absent_method_defaults_get_no_e_spec_025 | EC-009-017: absent method defaults to "GET", no error |
| test_BC_2_16_009_delete_put_patch_head_options_pass_validation | DELETE/PUT/PATCH/HEAD/OPTIONS all pass (full whitelist coverage) |
| test_BC_2_16_009_no_tables_produces_zero_errors | Spec with no tables → zero errors |

**Load-path integration:** `test_BC_2_16_009_validates_all_4_bundled_specs` confirms all 4 bundled sensor TOML specs (CrowdStrike, Armis, Claroty, Cyberint) pass Rule 7 validation via the full load_all() pipeline.

---

## AC-002 — Invalid Method Returns Structured E-SPEC-025 Error

**BC anchor:** BC-2.16.009 v1.8 §Validation Rules 7 — "invalid method produces E-SPEC-025";  
§Error Conditions E-SPEC-025 canonical entry  
**Test vectors:** §Canonical Test Vectors "HTTP method — CONNECT rejected" and "HTTP method — lowercase rejected"  
**Error message template (error-taxonomy.md v1.59, byte-verbatim, POL-24):**  
`"Step '<step_name>' in '<sensor_id>.<table_name>' declares method '<method_value>' which is not a supported HTTP method. Supported: GET, POST, PUT, PATCH, DELETE, HEAD, OPTIONS"`

**Red Gate test:** `test_BC_2_16_009_invalid_http_method_returns_structured_e_spec_025`

**Evidence:** AC-002-invalid-method-returns-e-spec-025.txt

**Tests demonstrating AC-002:**

| Test name | What it proves |
|-----------|----------------|
| test_BC_2_16_009_invalid_http_method_returns_structured_e_spec_025 | CONNECT → exactly 1 InvalidHttpMethod error; all 4 fields (step_name, sensor_id, table_name, method_value) verified |
| test_BC_2_16_009_e_spec_025_display_matches_error_taxonomy_v1_59_template_byte_for_byte | error.to_string() == exact E-SPEC-025 template byte-for-byte (POL-24) |
| test_BC_2_16_009_ec009_012_connect_produces_e_spec_025 | EC-009-012: CONNECT rejected |
| test_BC_2_16_009_ec009_013_trace_produces_e_spec_025 | EC-009-013: TRACE rejected |
| test_BC_2_16_009_ec009_014_typo_gett_produces_e_spec_025 | EC-009-014: GETT (typo) rejected |
| test_BC_2_16_009_ec009_015_lowercase_get_produces_e_spec_025 | EC-009-015: "get" (lowercase) rejected — case-sensitive, NOT normalized to GET |
| test_BC_2_16_009_lowercase_post_produces_e_spec_025 | "post" (lowercase) rejected |
| test_BC_2_16_009_ec009_016_empty_string_produces_e_spec_025 | EC-009-016: empty string rejected |
| test_BC_2_16_009_ec009_018_multi_error_collection_two_invalid_methods | EC-009-018: CONNECT + TRACE → exactly 2 errors (INV-ERR-003 no fail-fast) |
| test_BC_2_16_009_mixed_case_methods_produce_e_spec_025 | Get/Post/Delete/Put/Patch/Head/Options (mixed-case) all rejected |
| test_BC_2_16_009_mixed_valid_invalid_produces_one_error | GET (valid) + CONNECT (invalid) → exactly 1 error |

**Load-path integration:** `test_BC_2_16_009_load_path_connect_method_produces_e_spec_025` and  
`test_BC_2_16_009_load_all_invalid_method_produces_e_spec_025_with_numeric_toml_path` confirm  
E-SPEC-025 surfaces at the load_path() / load_all() boundary with canonical TOML path  
`sensor.tables[{ti}].steps[{si}].method`.

---

## AC-003 — Env-Resolved Method Validated Post-Resolution (Rule 6 → Rule 7 Ordering)

**BC anchor:** BC-2.16.009 v1.8 §Validation Rules 7 — "Rule 7 ordering: runs AFTER Rule 6";  
§Canonical Test Vectors "HTTP method — env-resolved invalid" + EC-009-019/020  
**Skip-guard:** ENV_TOKEN_REGEX (`\$\{env\.[A-Z0-9_]+\}`) from env_resolver.rs — well-formed tokens only  
**F-LOCAL-P3-MED-002:** Malformed pseudo-tokens (lowercase VAR_NAME, hyphen, empty) are NOT skipped

**Red Gate test:** `test_BC_2_16_009_env_resolved_invalid_method_caught_post_resolution`

**Evidence:** AC-003-env-resolved-method-validated-post-resolution.txt

**Tests demonstrating AC-003:**

| Test name | What it proves |
|-----------|----------------|
| test_BC_2_16_009_env_resolved_invalid_method_caught_post_resolution | After Rule 6 resolves ${env.M}→"CONNECT", Rule 7 fires E-SPEC-025 on resolved value |
| test_BC_2_16_009_ec009_020_unresolved_env_token_skipped_by_rule_7 | EC-009-020: ${env.SENSOR_STEP_METHOD} (unresolved, Rule 6 failed) → Rule 7 skips it |
| test_BC_2_16_009_any_env_token_in_method_skipped_by_rule_7 | ${env.METHOD}, ${env.HTTP_METHOD}, ${env.CROWDSTRIKE_METHOD} all skipped |
| test_BC_2_16_009_malformed_env_lowercase_var_name_produces_e_spec_025 | F-LOCAL-P3-MED-002: ${env.lower} → NOT skipped → E-SPEC-025 |
| test_BC_2_16_009_malformed_env_hyphen_in_var_name_produces_e_spec_025 | F-LOCAL-P3-MED-002: ${env.foo-bar} → NOT skipped → E-SPEC-025 |
| test_BC_2_16_009_malformed_env_empty_var_name_produces_e_spec_025 | F-LOCAL-P3-MED-002: ${env.} → NOT skipped → E-SPEC-025 |
| test_BC_2_16_009_well_formed_unresolved_token_still_skipped_after_f_med_002_fix | Non-regression: ${env.VALID_NAME}, ${env.A} still skipped after F-MED-002 fix |

**Load-path integration:** `test_BC_2_16_009_load_path_env_resolved_invalid_method_produces_e_spec_025` confirms end-to-end: TOML parse → env-resolve (Rule 6) → validate_step_methods (Rule 7) pipeline fires E-SPEC-025 for an env-resolved invalid method at the load_path() boundary.

**F-LOCAL-P4-MED-001 (index-carry fix):** `test_BC_2_16_009_load_all_invalid_method_on_second_step_produces_steps_1` proves that when two steps share the same name and the second step has an invalid method, the returned tuple carries step_index=1 (from enumerate), not 0 (from a name-reverse-lookup). TOML path `sensor.tables[0].steps[1].method` is correct.

---

## Source Code Anchors

**Source excerpt file:** source-excerpt-validate-step-methods.txt

Key production code paths:

1. `ALLOWED_HTTP_METHODS` constant — `crates/prism-spec-engine/src/validation.rs`  
   7 values; compile-time `const`; never runtime-configurable

2. `validate_step_methods()` function — `crates/prism-spec-engine/src/validation.rs`  
   Signature: `pub fn validate_step_methods(spec: &SensorSpec) -> Vec<(usize, usize, SpecEngineError)>`  
   Returns (table_index, step_index, error) tuples from enumerate() — no name-reverse-lookup

3. Skip-guard: `crate::env_resolver::ENV_TOKEN_REGEX.is_match(&step.method)` — single source of truth for well-formed token detection; prevents double-reporting with Rule 6

4. `SpecEngineError::InvalidHttpMethod` variant — `crates/prism-spec-engine/src/error.rs`  
   `#[error("Step '{step_name}' in '{sensor_id}.{table_name}' declares method '{method_value}' which is not a supported HTTP method. Supported: GET, POST, PUT, PATCH, DELETE, HEAD, OPTIONS")]`

5. Rule 7 wiring: `validate_step_methods()` called after `resolve_env_var_tokens()` (Rule 6) in both `load_path()` and `load_all()` — AC-003 Rule 6→Rule 7 ordering contract

---

## Full Test Suite

**Evidence file:** full-suite-BC-2-16-009.txt  
**Result:** 84/84 BC-2.16.009 tests pass — 38 new (http_method_whitelist_tests) + 46 preexisting (zero regressions)

---

## Traceability

| Artifact | Version / Anchor |
|----------|-----------------|
| Story | S-SPEC-HTTP-METHOD-VALIDATION-001 v1.2 |
| BC | BC-2.16.009 v1.8 §Validation Rules 7 (Wave-5 Phase-A PO burst) |
| Error taxonomy | error-taxonomy.md v1.59 E-SPEC-025 |
| Drift anchor | DRIFT-D926-001 (RESOLVED by this story merge) |
| PR | feature/S-SPEC-HTTP-METHOD-VALIDATION-001 → develop |
| LOCAL cascade | CONVERGED 3/3 at b1b81cd0 |

Note: citations use story version + BC version per TD-VSDD-091 (anti-volatile-pin rule). No HEAD-SHA pins in narrative evidence.
