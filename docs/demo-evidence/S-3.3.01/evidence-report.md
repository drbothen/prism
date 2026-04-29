# Demo Evidence Report — S-3.3.01

**Story:** S-3.3.01 — prism-customer-config: TOML schema, parser, and startup validator
**Anchor BCs:** BC-3.3.001, BC-3.3.002, BC-3.3.003, BC-3.3.004
**Implementation commit:** 87940e3f
**Recorded:** 2026-04-29
**Tool:** VHS 0.10.0

---

## Coverage Summary

| Recording | AC | BC Anchor | Result |
|-----------|-----|-----------|--------|
| AC-001-all-46-tests-green | AC-001 through AC-019 (full suite) | BC-3.3.001 + BC-3.3.002 + BC-3.3.003 + BC-3.3.004 | PASS — 46/46 |
| AC-002-error-codes | AC-002 through AC-019 (E-CFG-XXX paths) | BC-3.3.004 postconditions + BC-3.3.001 | PASS — 18/18 |
| AC-003-credential-heuristics | AC-011 + AC-012 | BC-3.3.002 postconditions + invariant 3, 4 | PASS — 5/5 |
| AC-004-structural-validation | AC-001 + AC-009 + AC-010 + AC-015 | BC-3.3.004 all postconditions + invariants | PASS — 8/8 |

---

## AC-001 — All 46 Tests GREEN (Full Suite)

**Traces to:** BC-3.3.001 (postcondition 1, 2), BC-3.3.002 (postcondition 1-4, invariant 3, 4),
BC-3.3.003 (postcondition 1-4, invariants 1-4), BC-3.3.004 (all postconditions + invariants)

**Command:**
```
cargo test -p prism-customer-config
```

**Terminal output captured (final frame):**
```
running 46 tests
test test_bc_3_3_001_all_st_types_reject_shared_mode ... ok
test test_bc_3_3_001_mssp_types_allow_client_mode ... ok
test test_bc_3_3_002_all_four_schemes_accepted ... ok
test test_bc_3_3_002_bearer_token_literal_rejected ... ok
test test_bc_3_3_002_client_secret_with_vault_scheme_passes ... ok
test test_bc_3_3_002_nested_api_key_literal_rejected ... ok
test test_bc_3_3_002_password_literal_rejected ... ok
test test_bc_3_3_004_empty_dir_returns_ok_empty ... ok
test test_bc_3_3_004_error_names_offending_file ... ok
test test_bc_3_3_004_errors_in_lexicographic_file_order ... ok
test test_bc_3_3_004_multi_error_three_violations ... ok
test test_bc_3_3_004_multi_file_multi_error ... ok
test test_bc_3_3_004_non_toml_file_skipped ... ok
test test_bc_3_3_004_valid_config_returns_ok ... ok
test test_bc_3_3_004_validation_error_means_no_configs_registered ... ok
... (31 additional tests) ...

test result: ok. 46 passed; 0 failed; 0 ignored; 0 measured; finished in 0.03s
```

**Recordings:**
- [AC-001-all-46-tests-green.gif](AC-001-all-46-tests-green.gif)
- [AC-001-all-46-tests-green.webm](AC-001-all-46-tests-green.webm)
- [AC-001-all-46-tests-green.tape](AC-001-all-46-tests-green.tape)

---

## AC-002 — Error Code Coverage (E-CFG-XXX Tests)

**Traces to:** BC-3.3.004 postconditions (R-CUST-001 through R-CUST-017), BC-3.3.001 (E-CFG-017)

**Command:**
```
cargo test -p prism-customer-config test_e_cfg -- --nocapture
```

**Terminal output captured (final frame):**
```
running 18 tests
test test_e_cfg_001_missing_org_id ... ok
test test_e_cfg_002_slug_mismatch ... ok
test test_e_cfg_003_uuid_v4_rejected ... ok
test test_e_cfg_004_unknown_dtu_type ... ok
test test_e_cfg_005_invalid_credential_ref_scheme ... ok
test test_e_cfg_006_unknown_archetype ... ok
test test_e_cfg_007_invalid_seed_negative ... ok
test test_e_cfg_008_invalid_scale_nan ... ok
test test_e_cfg_008_invalid_scale_zero ... ok
test test_e_cfg_009_invalid_mode_value ... ok
test test_e_cfg_010_allow_shared_override_rejected_wave3 ... ok
test test_e_cfg_010_unknown_field_in_dtu ... ok
test test_e_cfg_011_duplicate_org_id ... ok
test test_e_cfg_012_duplicate_org_slug ... ok
test test_e_cfg_013_demo_server_rejected_in_production ... ok
test test_e_cfg_014_client_mode_missing_spec ... ok
test test_e_cfg_015_spec_file_not_found ... ok
test test_e_cfg_016_shared_mode_with_spec ... ok

test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 28 filtered out; finished in 0.01s
```

Covers error codes E-CFG-001 through E-CFG-016 plus E-CFG-010 (allow_shared_override Wave 3 guard).
E-CFG-017 (SecurityTelemetrySharedMode) is covered by the BC-3.3.001 test set (see AC-001).

**Recordings:**
- [AC-002-error-codes.gif](AC-002-error-codes.gif)
- [AC-002-error-codes.webm](AC-002-error-codes.webm)
- [AC-002-error-codes.tape](AC-002-error-codes.tape)

---

## AC-003 — Credential Heuristics (BC-3.3.002)

**Traces to:** BC-3.3.002 postconditions 1-4 ("On credential value detected" clauses 2, 3;
"On valid config" clause 1), invariants 3 and 4 (error omits value; scheme prefixes pass)

**Command:**
```
cargo test -p prism-customer-config test_bc_3_3_002 -- --nocapture
```

**Terminal output captured (final frame):**
```
running 5 tests
test test_bc_3_3_002_password_literal_rejected ... ok
test test_bc_3_3_002_bearer_token_literal_rejected ... ok
test test_bc_3_3_002_nested_api_key_literal_rejected ... ok
test test_bc_3_3_002_client_secret_with_vault_scheme_passes ... ok
test test_bc_3_3_002_all_four_schemes_accepted ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 41 filtered out; finished in 0.01s
```

Tests prove: literal `bearer_token`, `password`, and nested `api_key` values are rejected
with E-CFG-020 (naming the field, never the value). All four allowed scheme prefixes
(`vault://`, `env://`, `file://`, `keyring://`) pass without error (AC-012).

**Recordings:**
- [AC-003-credential-heuristics.gif](AC-003-credential-heuristics.gif)
- [AC-003-credential-heuristics.webm](AC-003-credential-heuristics.webm)
- [AC-003-credential-heuristics.tape](AC-003-credential-heuristics.tape)

---

## AC-004 — Structural Validation Rules (BC-3.3.004)

**Traces to:** BC-3.3.004 postconditions ("On successful validation" clause 1;
"On any validation failure" clauses 1-4), invariants 1-4 (multi-error, lexicographic order,
non-.toml skipped, schema_version checked first)

**Command:**
```
cargo test -p prism-customer-config test_bc_3_3_004 -- --nocapture
```

**Terminal output captured (final frame):**
```
running 8 tests
test test_bc_3_3_004_empty_dir_returns_ok_empty ... ok
test test_bc_3_3_004_non_toml_file_skipped ... ok
test test_bc_3_3_004_error_names_offending_file ... ok
test test_bc_3_3_004_multi_error_three_violations ... ok
test test_bc_3_3_004_validation_error_means_no_configs_registered ... ok
test test_bc_3_3_004_multi_file_multi_error ... ok
test test_bc_3_3_004_errors_in_lexicographic_file_order ... ok
test test_bc_3_3_004_valid_config_returns_ok ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 38 filtered out; finished in 0.00s
```

Tests cover: empty directory returns `Ok([])` (AC-001), valid config returns `Ok([config])` (AC-009),
multi-error collection in single file (AC-010), non-.toml files silently skipped (AC-015),
errors from multiple files in lexicographic order (EC-007), zero registrations on any failure.

**Recordings:**
- [AC-004-structural-validation.gif](AC-004-structural-validation.gif)
- [AC-004-structural-validation.webm](AC-004-structural-validation.webm)
- [AC-004-structural-validation.tape](AC-004-structural-validation.tape)

---

## Acceptance Criteria Coverage

| AC | Description | Test(s) | Demo | Status |
|----|-------------|---------|------|--------|
| AC-001 | Empty customers/ dir returns Ok([]) | test_bc_3_3_004_empty_dir_returns_ok_empty | AC-004 | PASS |
| AC-002 | Missing org_id → E-CFG-001 | test_e_cfg_001_missing_org_id | AC-002 | PASS |
| AC-003 | Slug mismatch → E-CFG-002 | test_e_cfg_002_slug_mismatch | AC-002 | PASS |
| AC-004 | UUID v4 → E-CFG-003 | test_e_cfg_003_uuid_v4_rejected | AC-002 | PASS |
| AC-005 | demo-server → E-CFG-013 | test_e_cfg_013_demo_server_rejected_in_production | AC-002 | PASS |
| AC-006 | Bad credential_ref scheme → E-CFG-005 | test_e_cfg_005_invalid_credential_ref_scheme | AC-002 | PASS |
| AC-007 | scale=0.0 / NaN → E-CFG-008 | test_e_cfg_008_invalid_scale_zero / _nan | AC-002 | PASS |
| AC-008 | Duplicate org_id → E-CFG-011 | test_e_cfg_011_duplicate_org_id | AC-002 | PASS |
| AC-009 | Valid config returns Ok([config]) | test_bc_3_3_004_valid_config_returns_ok | AC-004 | PASS |
| AC-010 | Three violations in one file → 3 errors (multi-error) | test_bc_3_3_004_multi_error_three_violations | AC-004 | PASS |
| AC-011 | bearer_token literal → E-CFG-020 (value not echoed) | test_bc_3_3_002_bearer_token_literal_rejected | AC-003 | PASS |
| AC-012 | All four schemes pass credential check | test_bc_3_3_002_all_four_schemes_accepted | AC-003 | PASS |
| AC-013 | Missing schema_version → E-CFG-030 | test_bc_3_3_003_* (in full suite) | AC-001 | PASS |
| AC-014 | schema_version=2 → E-CFG-031 + migration hint | test_bc_3_3_003_* (in full suite) | AC-001 | PASS |
| AC-015 | README.md silently skipped | test_bc_3_3_004_non_toml_file_skipped | AC-004 | PASS |
| AC-016 | claroty + shared mode → E-CFG-017 | test_bc_3_3_001_all_st_types_reject_shared_mode | AC-001 | PASS |
| AC-017 | allow_shared_override → E-CFG-010 (unknown field) | test_e_cfg_010_allow_shared_override_rejected_wave3 | AC-002 | PASS |
| AC-018 | mode=client + missing spec file → E-CFG-015 | test_e_cfg_015_spec_file_not_found | AC-002 | PASS |
| AC-019 | mode=shared + spec field → E-CFG-016 | test_e_cfg_016_shared_mode_with_spec | AC-002 | PASS |

**Total: 19/19 AC coverage — all PASS**
