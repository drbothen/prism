# Demo Evidence Report — S-SPEC-HTTP-METHOD-VALIDATION-001

**Story:** S-SPEC-HTTP-METHOD-VALIDATION-001 v1.4 — HTTP Method Whitelist Validation in Sensor Spec  
**Wave:** wave-5-e-demo-fidelity  
**BC:** BC-2.16.009 v1.10 §Validation Rules 7  
**Error code:** E-SPEC-025 (error-taxonomy.md v1.60)  
**LOCAL adversary cascade:** CONVERGED 3/3 at commit b1b81cd0  
**PR-level adversary cascade:** CONVERGED — all findings closed (F-PR1-OBS-001, SEC-001, F-LOCAL-P3-MED-002, F-LOCAL-P4-MED-001)  
**Demo mode:** LIBRARY (test-harness nextest capture) — prism-spec-engine is a library crate with no CLI binary surface. Test harness is the production-representative evidence vehicle, following the S-SPEC-ENV-VAR-001 / S-PLUGIN-PREREQ-D precedent.

---

## Coverage Summary

| AC | Description | Evidence File | Red Gate Test | Status |
|----|-------------|---------------|---------------|--------|
| AC-001 | All 7 whitelist methods pass validation | AC-001-valid-http-methods-pass-validation.txt | test_BC_2_16_009_valid_http_method_passes_validation | PASS |
| AC-002 | Invalid method returns structured E-SPEC-025 | AC-002-invalid-method-returns-e-spec-025.txt | test_BC_2_16_009_invalid_http_method_returns_structured_e_spec_025 | PASS |
| AC-003 | Env-resolved method validated post-Rule-6 | AC-003-env-resolved-method-validated-post-resolution.txt | test_BC_2_16_009_env_resolved_invalid_method_caught_post_resolution | PASS |
| AC-004 | Overlong method_value truncated to 32 codepoints (SEC-001 / CWE-400) | AC-004-overlong-method-value-truncated.txt | test_BC_2_16_009_sec_001_overlong_method_truncated_in_error | PASS |
| AC-005 | Full-match skip-guard — partial embeddings produce E-SPEC-025 (F-PR1-OBS-001) | AC-005-full-match-skip-guard.txt | test_BC_2_16_009_f_pr1_obs_001_partial_token_embedding_not_skipped | PASS |

---

## AC-001 — Valid HTTP Methods Pass Validation

**BC anchor:** BC-2.16.009 v1.10 §Validation Rules 7 — "valid method passes"  
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

**BC anchor:** BC-2.16.009 v1.10 §Validation Rules 7 — "invalid method produces E-SPEC-025";  
§Error Conditions E-SPEC-025 canonical entry  
**Test vectors:** §Canonical Test Vectors "HTTP method — CONNECT rejected" and "HTTP method — lowercase rejected"  
**Error message template (error-taxonomy.md v1.60, byte-verbatim, POL-24):**  
`"Step '<step_name>' in '<sensor_id>.<table_name>' declares method '<method_value>' which is not a supported HTTP method. Supported: GET, POST, PUT, PATCH, DELETE, HEAD, OPTIONS"`

**Red Gate test:** `test_BC_2_16_009_invalid_http_method_returns_structured_e_spec_025`

**Evidence:** AC-002-invalid-method-returns-e-spec-025.txt

**Tests demonstrating AC-002:**

| Test name | What it proves |
|-----------|----------------|
| test_BC_2_16_009_invalid_http_method_returns_structured_e_spec_025 | CONNECT → exactly 1 InvalidHttpMethod error; all 4 fields (step_name, sensor_id, table_name, method_value) verified |
| test_BC_2_16_009_e_spec_025_display_matches_error_taxonomy_template_byte_for_byte | error.to_string() == exact E-SPEC-025 template byte-for-byte (POL-24) |
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

**BC anchor:** BC-2.16.009 v1.10 §Validation Rules 7 — "Rule 7 ordering: runs AFTER Rule 6";  
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

## AC-004 — Overlong Method Value Truncated to 32 Codepoints (SEC-001 / CWE-400)

**BC anchor:** BC-2.16.009 v1.10 §VR7 Point-3 — SEC-001 truncation (32-codepoint cap in method_value)  
**Security:** SEC-001 / CWE-400 unbounded echo: A 256 KiB TOML allows a method field up to that size; echoing verbatim in E-SPEC-025 would produce a 256 KiB error string to the MCP caller. 32-codepoint cap prevents. Normal HTTP methods are ≤7 chars; 32 is generous for legibility.  
**Non-regression:** Method values ≤32 codepoints pass through unchanged (POL-24 byte-exact Display).

**Red Gate test:** `test_BC_2_16_009_sec_001_overlong_method_truncated_in_error`

**Evidence:** AC-004-overlong-method-value-truncated.txt

**Tests demonstrating AC-004:**

| Test name | What it proves |
|-----------|----------------|
| test_BC_2_16_009_sec_001_overlong_method_truncated_in_error | 33-char method → method_value.len() ≤ 32 in InvalidHttpMethod (SEC-001 / CWE-400) |
| test_BC_2_16_009_sec_001_exactly_32_chars_not_truncated | 32-char method → preserved at cap (not truncated) |
| test_BC_2_16_009_sec_001_normal_length_method_not_truncated | 7-char "CONNECT" → method_value == "CONNECT" byte-exact (POL-24 non-regression) |

---

## AC-005 — Full-Match Skip-Guard Prevents Skipping Partial Token Embeddings (F-PR1-OBS-001)

**BC anchor:** BC-2.16.009 v1.10 §VR7 Point-3 — full-match skip-guard (F-PR1-OBS-001 fix)  
**Fix:** Old code used `ENV_TOKEN_REGEX.is_match(&step.method)` (substring check). That incorrectly skipped "GET${env.X}", "${env.X}GET", "${env.A}${env.B}" — partial interpolation residues that Rule 6 cannot fully replace. Fix uses full-match: `ENV_TOKEN_REGEX.find(...).is_some_and(|m| m.start()==0 && m.end()==len)`.  
**Non-regression:** Exact single well-formed tokens ("${env.X}", "${env.VALID_NAME}") are still skipped after the fix.

**Red Gate test:** `test_BC_2_16_009_f_pr1_obs_001_partial_token_embedding_not_skipped`

**Evidence:** AC-005-full-match-skip-guard.txt

**Tests demonstrating AC-005:**

| Test name | What it proves |
|-----------|----------------|
| test_BC_2_16_009_f_pr1_obs_001_partial_token_embedding_not_skipped | "GET${env.X}" is NOT skipped; produces E-SPEC-025 (suffix embedding) |
| test_BC_2_16_009_f_pr1_obs_001_token_prefix_not_skipped | "${env.X}GET" is NOT skipped; produces E-SPEC-025 (prefix embedding) |
| test_BC_2_16_009_f_pr1_obs_001_two_tokens_concatenated_not_skipped | "${env.A}${env.B}" is NOT skipped; two tokens ≠ single token → E-SPEC-025 |
| test_BC_2_16_009_f_pr1_obs_001_exact_single_token_still_skipped | "${env.X}", "${env.VALID_NAME}", "${env.A1_B2}" are still correctly skipped |

---

## Source Code Anchors

**Source excerpt file:** source-excerpt-validate-step-methods.txt

Key production code paths:

1. `ALLOWED_HTTP_METHODS` constant — `crates/prism-spec-engine/src/validation.rs`  
   7 values; compile-time `const`; never runtime-configurable

2. `validate_step_methods()` function — `crates/prism-spec-engine/src/validation.rs`  
   Signature: `pub fn validate_step_methods(spec: &SensorSpec) -> Vec<(usize, usize, SpecEngineError)>`  
   Returns (table_index, step_index, error) tuples from enumerate() — no name-reverse-lookup (F-LOCAL-P4-MED-001)

3. Full-match skip-guard (F-PR1-OBS-001 fix):  
   `crate::env_resolver::ENV_TOKEN_REGEX.find(&step.method).is_some_and(|m| m.start() == 0 && m.end() == step.method.len())`  
   — single source of truth for well-formed token detection; prevents double-reporting with Rule 6  
   — NOT the old substring `is_match` (which incorrectly skipped partial embeddings)

4. SEC-001 truncation (AC-004):  
   `let method_value = truncate_at_char_boundary(&step.method, 32).to_string();`  
   — caps method_value at 32 codepoints before embedding in InvalidHttpMethod error

5. `SpecEngineError::InvalidHttpMethod` variant — `crates/prism-spec-engine/src/error.rs`  
   `#[error("Step '{step_name}' in '{sensor_id}.{table_name}' declares method '{method_value}' which is not a supported HTTP method. Supported: GET, POST, PUT, PATCH, DELETE, HEAD, OPTIONS")]`

6. Rule 7 wiring: `validate_step_methods()` called after `resolve_env_var_tokens()` (Rule 6) in both `load_path()` and `load_all()` — AC-003 Rule 6→Rule 7 ordering contract

---

## Full Test Suite

**Evidence file:** full-suite-BC-2-16-009.txt  
**Result:** 93/93 BC-2.16.009 tests pass (authoritative: "93 tests run: 93 passed" nextest summary)

Breakdown (PASS-line counts derived from the captured nextest output):

| Module | Location | Tests |
|--------|----------|-------|
| `validation::http_method_whitelist_tests` | `crates/prism-spec-engine/src/validation.rs` (`#[cfg(test)] mod`) | 35 (of which 5 are Red Gate tests, one per AC) |
| `bc_2_16_009_test` | `crates/prism-spec-engine/tests/bc_2_16_009_test.rs` | 26 |
| `bc_2_16_009_bundled_spec_validation` | `crates/prism-spec-engine/tests/` | 5 |
| `proofs::spec_validator` | `crates/prism-spec-engine/src/proofs/` | 10 |
| `write_endpoint_tests` | `crates/prism-spec-engine/tests/` | 17 |
| **Total** | | **93** (35+26+5+10+17) |

---

## Traceability

| Artifact | Version / Anchor |
|----------|-----------------|
| Story | S-SPEC-HTTP-METHOD-VALIDATION-001 v1.4 |
| BC | BC-2.16.009 v1.10 §Validation Rules 7 (Wave-5 Phase-A PO burst + PR-level cascade closures) |
| Error taxonomy | error-taxonomy.md v1.60 E-SPEC-025 |
| Drift anchor | DRIFT-D926-001 (RESOLVED by this story merge) |
| PR | feature/S-SPEC-HTTP-METHOD-VALIDATION-001 → develop |
| LOCAL cascade | CONVERGED 3/3 at b1b81cd0 |
| PR-level cascade | CONVERGED — F-PR1-OBS-001, SEC-001, F-LOCAL-P3-MED-002, F-LOCAL-P4-MED-001 all closed |

Note: citations use story version + BC version per TD-VSDD-091 (anti-volatile-pin rule). No HEAD-SHA pins in narrative evidence.
