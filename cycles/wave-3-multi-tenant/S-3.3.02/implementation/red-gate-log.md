---
document_type: red-gate-log
level: ops
version: "1.0"
status: open
producer: test-writer
timestamp: "2026-04-29T00:00:00Z"
phase: 3
inputs: []
input-hash: "[live-state]"
traces_to: ""
stub_architect_agent: "[S-3.3.02-prior-phase-d1e3cf38]"
stub_compile_verified: true
test_writer_agent: "[wave-3-phase-c-S-3.3.02]"
red_gate_verified: true
---

# Red Gate Log: S-3.3.02 — OrgRegistry Boot Orchestrator

## Summary

| Story | Tests Written | All Fail (Red)? | Gate |
|-------|--------------|-----------------|------|
| S-3.3.02 | 15 (startup_boot_test.rs) | yes — 15/15 FAILED via `todo!()` panic | PASS — Red Gate confirmed |

## Test File

`crates/prism-customer-config/tests/startup_boot_test.rs`

## Tests Written

| Test Name | BC Clause | AC |
|-----------|-----------|-----|
| `test_BC_3_3_004_two_valid_files_both_registered` | BC-3.3.004 postcond "On successful validation" §1; BC-3.1.003 postcond §1 | AC-001 |
| `test_BC_3_1_003_bijectivity_after_valid_boot` | BC-3.1.003 postcond §1, invariant §1; TV-3.1.003-01 | AC-004 |
| `test_BC_3_3_004_n_valid_files_exactly_n_entries` | BC-3.3.004 postcond §1; VP-105 | AC-004 |
| `test_BC_3_3_004_invalid_toml_returns_validation_failed` | BC-3.3.004 postcond §1-4; VP-106 | AC-002, AC-006 |
| `test_BC_3_3_004_all_errors_aggregated_multi_error` | BC-3.3.004 postcond §2, invariant §2; EC-3.3.004-03 | AC-006 |
| `test_BC_3_3_004_duplicate_org_id_returns_validation_failed` | BC-3.1.004 postcond §2; BC-3.3.004 R-CUST-011; TV-3.1.004-04, TV-3.3.004-12 | AC-002 |
| `test_BC_3_3_004_duplicate_org_slug_returns_validation_failed` | BC-3.1.004 postcond §3; BC-3.3.004 R-CUST-012; TV-3.1.004-04 | AC-003 |
| `test_BC_3_3_004_empty_dir_returns_ok_zero` | BC-3.3.004 EC-3.3.004-01, precond §5 | AC-005 |
| `test_BC_3_3_004_validate_before_register_no_partial_state` | BC-3.3.004 invariant §1; ADR-010 §2.5 | AC-002 partial-state |
| `test_BC_3_1_003_registry_unchanged_on_validation_failure` | BC-3.1.003 precond §2; BC-3.3.004 postcond §4; VP-106 | |
| `test_BC_3_1_004_duplicate_org_id_error_contains_both_files` | BC-3.1.004 postcond §4; BC-3.3.004 postcond §3; TV-3.3.004-12 | |
| `test_BC_3_1_004_duplicate_org_slug_error_contains_both_files` | BC-3.1.004 postcond §4; BC-3.3.004 R-CUST-012 | |
| `test_BC_3_3_004_non_toml_files_silently_skipped` | BC-3.3.004 EC-3.3.004-07 | |
| `test_BC_3_1_003_forward_reverse_map_sizes_equal` | BC-3.1.003 invariant §1; EC-005 | AC-004 |
| `test_BC_3_3_004_registration_failed_when_registry_already_has_conflict` | BC-3.3.04 Task 6; EC-003; BC-3.1.004 postcond §5 | |

## Stubs Created

The prior phase (commit d1e3cf38) created the following stubs for this story:

- `crates/prism-customer-config/src/boot.rs` — `pub fn boot_org_registry(customers_dir: &Path, registry: &OrgRegistry) -> Result<usize, BootError>` stub; body is `todo!("S-3.3.02: boot_org_registry not yet implemented — Red Gate stub")`
- `pub enum BootError { ValidationFailed(Vec<ConfigError>), RegistrationFailed(RegistrationError) }` — defined in same file; `Display` and `Error` implemented
- `crates/prism-customer-config/tests/startup_boot_test.rs` — empty scaffold (0 test functions); replaced in this phase with 15 failing tests

## Red Gate Verification

All 15 tests FAIL with:

```
panicked at crates/prism-customer-config/src/boot.rs:103:5:
not yet implemented: S-3.3.02: boot_org_registry not yet implemented — Red Gate stub
```

Failure is the correct `todo!()` panic — no spurious passes. No tests produce false
positives (vacuously true assertions). All assertions are load-bearing.

```
test result: FAILED. 0 passed; 15 failed; 0 ignored; 0 measured; 0 filtered out
```

## Regression Check

| Existing Tests | Status |
|---------------|--------|
| 46 `validation_tests.rs` tests (S-3.3.01) | all pass — 0 regressions |

## Cargo check (--no-run)

```
Finished `test` profile [unoptimized + debuginfo] target(s) in 0.43s
Executable tests/startup_boot_test.rs (target/debug/deps/startup_boot_test-bf4af137e3b1504b)
```

Compiles clean. No warnings.

## Hand-Off to Implementer

- Replace `todo!()` in `crates/prism-customer-config/src/boot.rs:boot_org_registry`
  with the algorithm documented in the function's doc-comment (ADR-010 §2.5):
  1. Call `load_and_validate(customers_dir)` — validates ALL files.
  2. On `Err(errors)` → return `Err(BootError::ValidationFailed(errors))` without touching registry.
  3. On `Ok(configs)` → call `registry.register(slug, id)` for each config in order.
  4. On register `Err(e)` → return `Err(BootError::RegistrationFailed(e))`.
  5. Return `Ok(n)` where `n = configs.len()`.
- `OrgSlug::new(config.org_slug)` — valid by R-CUST-002 guarantee.
- `OrgId::from_uuid(config.org_id)` — valid by R-CUST-003 guarantee.
- Do NOT call `register` before `load_and_validate` returns `Ok` (ADR-010 §2.5).
