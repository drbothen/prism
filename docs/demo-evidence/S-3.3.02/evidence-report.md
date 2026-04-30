# Demo Evidence Report — S-3.3.02

**Story:** S-3.3.02 — OrgRegistry boot from customer config
**Anchor BCs:** BC-3.3.004, BC-3.1.003
**Implementation commit:** 73c0bbe3
**Recorded:** 2026-04-29
**Tool:** VHS 0.10.0

---

## Coverage Summary

| Recording | AC | BC Anchor | Result |
|-----------|-----|-----------|--------|
| AC-001-all-15-tests-green | AC-001 | BC-3.3.004 + BC-3.1.003 | PASS — 15/15 |
| AC-002-validate-before-register | AC-002 | BC-3.3.004 postcondition (validate-before-register) | PASS — 1/1 |

---

## AC-001 — All 15 Boot Tests GREEN

**Traces to:** BC-3.3.004 (full OrgRegistry boot from TOML customer config) + BC-3.1.003 (forward/reverse map bijectivity)

**Command:**
```
cargo test -p prism-customer-config --test startup_boot_test 2>&1 | tail -25
```

**Terminal output captured (final frame):**
```
running 15 tests
test test_BC_3_3_004_empty_dir_returns_ok_zero ... ok
test test_BC_3_3_004_invalid_toml_returns_validation_failed ... ok
test test_BC_3_3_004_duplicate_org_id_returns_validation_failed ... ok
test test_BC_3_1_004_duplicate_org_id_error_contains_both_files ... ok
test test_BC_3_3_004_duplicate_org_slug_returns_validation_failed ... ok
test test_BC_3_1_004_duplicate_org_slug_error_contains_both_files ... ok
test test_BC_3_3_004_validate_before_register_no_partial_state ... ok
test test_BC_3_1_003_bijectivity_after_valid_boot ... ok
test test_BC_3_1_003_registry_unchanged_on_validation_failure ... ok
test test_BC_3_3_004_two_valid_files_both_registered ... ok
test test_BC_3_3_004_all_errors_aggregated_multi_error ... ok
test test_BC_3_3_004_n_valid_files_exactly_n_entries ... ok
test test_BC_3_3_004_non_toml_files_silently_skipped ... ok
test test_BC_3_3_004_registration_failed_when_registry_already_has_conflict ... ok
test test_BC_3_1_003_forward_reverse_map_sizes_equal ... ok

test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

**Recordings:**
- [AC-001-all-15-tests-green.gif](AC-001-all-15-tests-green.gif)
- [AC-001-all-15-tests-green.webm](AC-001-all-15-tests-green.webm)
- [AC-001-all-15-tests-green.tape](AC-001-all-15-tests-green.tape)

---

## AC-002 — Validate-Before-Register (No Partial State)

**Traces to:** BC-3.3.004 postcondition — validation must complete before any registration; partial state must not be written on failure

**Command:**
```
cargo test -p prism-customer-config --test startup_boot_test \
  test_BC_3_3_004_validate_before_register_no_partial_state -- --nocapture
```

**Terminal output captured (final frame):**
```
running 1 test
test test_BC_3_3_004_validate_before_register_no_partial_state ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 14 filtered out; finished in 0.00s
```

The test verifies that when a batch of customer TOML files contains a conflict (duplicate OrgId
or slug), zero entries are written to the OrgRegistry — no partial registrations survive. The
registry is left unchanged, satisfying the all-or-nothing boot invariant required by BC-3.3.004.

**Recordings:**
- [AC-002-validate-before-register.gif](AC-002-validate-before-register.gif)
- [AC-002-validate-before-register.webm](AC-002-validate-before-register.webm)
- [AC-002-validate-before-register.tape](AC-002-validate-before-register.tape)

---

## Acceptance Criteria Coverage

| AC | Description | Test Name | Status |
|----|-------------|-----------|--------|
| AC-001 | All 15 startup_boot_test tests pass GREEN | Full startup_boot_test suite | PASS |
| AC-002 | Validate-before-register — no partial state on conflict | test_BC_3_3_004_validate_before_register_no_partial_state | PASS |

**Total: 2/2 AC demos recorded — full coverage**
