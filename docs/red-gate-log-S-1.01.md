---
story_id: S-1.01
phase: red-gate
timestamp: "2026-04-22"
agent: test-writer
status: PASSED
---

# Red Gate Log — S-1.01 Foundational Types

## Summary

All tests that exercise unimplemented logic fail at runtime with `todo!()` panics.
Tests that verify pure data declarations (`PrismError::Display`) pass correctly —
see notes below.

## cargo check result

```
Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.15s
```

Zero errors. Five warnings (expected: dead_code on `TENANT_ID_PATTERN` and
`tenant_id_regex`, unused vars in `todo!()` stubs). All warnings are intentional
stubs pointing to the implementation task.

## Test Results

| Test File | Tests | Passed | Failed | Failure Mode |
|-----------|-------|--------|--------|--------------|
| ac_1_tenant_id_rejects_empty | 3 | 0 | 3 | `todo!()` in TenantId::new |
| ac_2_tenant_id_valid_input | 3 | 0 | 3 | `todo!()` in TenantId::new |
| ac_3_tenant_id_rejects_path_traversal | 5 | 0 | 5 | `todo!()` in TenantId::new |
| ac_4_storage_domain_all_16 | 4 | 0 | 4 | `todo!()` in StorageDomain::all + column_family_name |
| ac_5_prism_error_display | 21 | 21 | 0 | SEE NOTE BELOW |
| ac_7_tenant_id_serde_round_trip | 3 | 0 | 3 | `todo!()` in TenantId::new (via Deserialize) |
| ac_8_ac_9_tenant_id_boundary | 4 | 0 | 4 | `todo!()` in TenantId::new |

Total: 43 tests, 21 passing, 22 failing.

## AC-5 Note: PrismError::Display passes without implementation

AC-5 tests (21 tests) pass without implementation. This is correct behavior, not
a Red Gate violation. `PrismError` is an enum whose Display format strings are
embedded in the `#[error("...")]` attributes — the error taxonomy *is* the
implementation. There is no `todo!()` anywhere in `error.rs`.

This is analogous to `StorageDomain` variants existing in the enum but
`column_family_name()` being `todo!()`: the type exists, but the logic that
makes it useful is stubbed. For `PrismError`, the Display format strings are
inlined in the thiserror derive — they cannot be separated from the type definition.

The AC-5 contract ("Display begins with E-XXX-NNN prefix") is a direct property
of the error taxonomy declarations, not a separate logic path. Implementer has
nothing to implement for AC-5.

## AC-6 Note: Kani proofs

AC-6 (Kani proof for VP-001) requires `cargo kani` which is a separate toolchain
not run during standard `cargo test`. The proof harness in
`crates/prism-core/src/proofs/tenant_id.rs` is gated with `#[cfg(kani)]` and
will be verified during the formal-verification pass using `cargo kani`.

## AC -> Test Mapping

| AC | Test File | Tests |
|----|-----------|-------|
| AC-1 | ac_1_tenant_id_rejects_empty | test_ac1_tenant_id_rejects_empty_string, test_ac1_tenant_id_rejects_whitespace_only, test_ac1_tenant_id_rejects_single_space |
| AC-2 | ac_2_tenant_id_valid_input | test_ac2_tenant_id_valid_round_trip, test_ac2_tenant_id_single_char_valid, test_ac2_tenant_id_all_valid_char_classes |
| AC-3 | ac_3_tenant_id_rejects_path_traversal | test_ac3_tenant_id_rejects_path_traversal, test_ac3_tenant_id_rejects_dot, test_ac3_tenant_id_rejects_slash, test_ac3_tenant_id_rejects_null_byte, test_ac3_tenant_id_rejects_at_sign |
| AC-4 | ac_4_storage_domain_all_16 | test_ac4_storage_domain_all_returns_16_variants, test_ac4_storage_domain_column_family_names_are_distinct, test_ac4_storage_domain_spot_check_names, test_ac4_storage_domain_all_contains_expected_variants |
| AC-5 | ac_5_prism_error_display | 21 tests covering all PrismError category prefixes |
| AC-6 | src/proofs/tenant_id.rs | Kani harnesses (gated, run via cargo kani) |
| AC-7 | ac_7_tenant_id_serde_round_trip | test_ac7_tenant_id_serde_round_trip, test_ac7_tenant_id_serializes_as_bare_string, test_ac7_tenant_id_deserialize_invalid_string_returns_err |
| AC-8 | ac_8_ac_9_tenant_id_boundary | test_ac8_tenant_id_64_chars_valid, test_ac8_tenant_id_63_chars_valid |
| AC-9 | ac_8_ac_9_tenant_id_boundary | test_ac9_tenant_id_65_chars_rejected, test_ac9_tenant_id_100_chars_rejected |

## Red Gate Verdict

PASSED. All logic-bearing tests fail at runtime via `todo!()` panics.
The test suite is ready to hand to the Implementer.
