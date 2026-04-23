---
story: S-1.06
phase: red-gate
date: 2026-04-22
agent: test-writer
---

# Red Gate Log — S-1.06: Credential Store Trait and Backends

## Result

RED GATE PASSED — all 35 tests fail before implementation.

## Test Run Summary

```
test result: FAILED. 0 passed; 35 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.20s
```

## Failure Mode

All tests fail with `not implemented: <stub description>` panics. No test passes
vacuously. No compilation errors — all stubs compile cleanly.

## Test Inventory

### BC-2.03.001 — CredentialStore trait
| Test | BC Clause | Status |
|------|-----------|--------|
| `test_BC_2_03_001_set_then_get_returns_value` | postcondition 1, AC-1 | FAIL (unimplemented) |
| `test_BC_2_03_001_get_missing_credential_returns_not_found` | EC-03-001 | FAIL |
| `test_BC_2_03_001_set_overwrites_existing_value` | EC-03-001 (TV-003) | FAIL |
| `test_BC_2_03_001_delete_nonexistent_returns_false` | EC-03-002, TV-004 | FAIL |
| `test_BC_2_03_001_list_empty_tenant_returns_empty_vec` | EC-03-003, TV-005 | FAIL |
| `test_BC_2_03_001_list_returns_all_stored_entries` | postcondition, AC-10 | FAIL |

### BC-2.03.003 — EncryptedFileBackend (unit + proptests)
| Test | BC Clause / VP | Status |
|------|----------------|--------|
| `test_BC_2_03_003_prop_encrypt_decrypt_round_trip` | VP-034 | FAIL (proptest, unimplemented) |
| `test_BC_2_03_003_prop_key_derivation_deterministic` | VP-035 | FAIL (proptest, unimplemented) |
| `test_BC_2_03_003_encrypted_file_set_get_round_trip` | postcondition 1, AC-3 | FAIL |
| `test_BC_2_03_003_two_encryptions_differ_in_nonce_and_salt` | TV-006 | FAIL |
| `test_BC_2_03_003_decrypt_zero_byte_file_returns_err` | EC-03-008 | FAIL |
| `test_BC_2_03_003_decrypt_truncated_file_returns_err` | EC-03-009 | FAIL |
| `test_BC_2_03_003_decrypt_wrong_key_returns_err` | TV-002 | FAIL |
| `test_BC_2_03_003_empty_passphrase_returns_err` | EC-005 | FAIL |

### BC-2.03.004 — Namespace isolation
| Test | BC Clause | Status |
|------|-----------|--------|
| `test_BC_2_03_004_namespace_key_format` | postcondition 1 | FAIL |
| `test_BC_2_03_004_cross_tenant_isolation` | invariant 1, AC-4 | FAIL |
| `test_BC_2_03_004_invariant_client_credentials_are_independent` | TV-001 | FAIL |
| `test_BC_2_03_004_client_id_with_dashes` | TV-003 | FAIL |
| `namespace::tests::test_BC_2_03_004_namespace_key_format_stub_panics` | postcondition | FAIL |

### BC-2.03.008 — CredentialName sanitization
| Test | BC Clause | Status |
|------|-----------|--------|
| `test_BC_2_03_008_rejects_path_traversal_in_credential_name` | precondition 1, AC-5, TV-001 | FAIL |
| `test_BC_2_03_008_rejects_null_byte_in_credential_name` | TV-002 | FAIL |
| `test_BC_2_03_008_rejects_empty_credential_name` | TV-003 | FAIL |
| `test_BC_2_03_008_accepts_valid_credential_name` | TV-004 | FAIL |
| `test_BC_2_03_008_accepts_leading_dot_in_credential_name` | TV-005 | FAIL |
| `test_BC_2_03_008_rejects_spaces_in_credential_name` | TV-006 | FAIL |

### BC-2.03.011 — Startup probe
| Test | BC Clause | Status |
|------|-----------|--------|
| `test_BC_2_03_011_probe_returns_status_without_panic` | postcondition 1, AC-6 | FAIL |
| `test_BC_2_03_011_probe_available_with_empty_keyring` | TV-003 | FAIL |

### BC-2.03.012 — BackendSelector
| Test | BC Clause | Status |
|------|-----------|--------|
| `test_BC_2_03_012_auto_with_unavailable_keyring_selects_file` | postcondition 3, AC-2 | FAIL |
| `test_BC_2_03_012_auto_with_available_keyring_selects_keyring` | TV-001 | FAIL |
| `test_BC_2_03_012_explicit_keyring_with_unavailable_probe_is_hard_error` | error case 1, AC-7 | FAIL |
| `test_BC_2_03_012_explicit_file_with_missing_passphrase_is_hard_error` | TV-004 | FAIL |
| `test_BC_2_03_012_container_auto_falls_back_to_file` | TV-005, EC-03-028 | FAIL |

### CredentialIndex
| Test | Clause | Status |
|------|--------|--------|
| `test_credential_index_add_and_list` | add/list | FAIL |
| `test_credential_index_remove_reduces_count` | remove | FAIL |
| `test_credential_index_remove_nonexistent_is_noop` | no-op | FAIL |

## Acceptance Criteria Coverage

| AC | Test | Red Gate |
|----|------|----------|
| AC-1 (set→get KeyringBackend) | `test_BC_2_03_001_set_then_get_returns_value` | FAIL |
| AC-2 (auto→file on keyring unavail) | `test_BC_2_03_012_auto_with_unavailable_keyring_selects_file` | FAIL |
| AC-3 (EncryptedFile round-trip) | `test_BC_2_03_003_encrypted_file_set_get_round_trip` | FAIL |
| AC-4 (namespace isolation) | `test_BC_2_03_004_cross_tenant_isolation` | FAIL |
| AC-5 (path traversal rejected) | `test_BC_2_03_008_rejects_path_traversal_in_credential_name` | FAIL |
| AC-6 (startup probe) | `test_BC_2_03_011_probe_returns_status_without_panic` | FAIL |
| AC-7 (explicit keyring hard error) | `test_BC_2_03_012_explicit_keyring_with_unavailable_probe_is_hard_error` | FAIL |
| AC-8 (VP-034 proptest) | `test_BC_2_03_003_prop_encrypt_decrypt_round_trip` | FAIL |
| AC-9 (VP-035 proptest) | `test_BC_2_03_003_prop_key_derivation_deterministic` | FAIL |
| AC-10 (list 3 entries) | `test_BC_2_03_001_list_returns_all_stored_entries` | FAIL |

## Notes

- `test_BC_2_03_001_set_then_get_returns_value` tests the AC-1 behaviour via
  `EncryptedFileBackend` because `KeyringBackend` requires OS keyring availability
  which is not guaranteed in CI. The test structure applies equally to both backends.
- BC-2.03.008 tests use `#[should_panic(expected = "unimplemented")]` at Red Gate
  because `CredentialName::new` is a stub. After S-1.02 is implemented those
  `#[should_panic]` annotations must be removed and the tests must assert `Err` variants.
- `test_BC_2_03_003_empty_passphrase_returns_err` calls `derive_key` directly (not
  through the stub constructor) so it fails differently — `not implemented: derive_key`.
- All 7 BCs have at least one test.
- Both VPs (VP-034, VP-035) have proptest coverage.

## Files Created

| File | Purpose |
|------|---------|
| `crates/prism-core/` | S-1.01/S-1.02 stub crate (TenantId, CredentialName, PrismError) |
| `crates/prism-credentials/src/trait_.rs` | CredentialStore trait stub |
| `crates/prism-credentials/src/namespace.rs` | namespace_key() stub |
| `crates/prism-credentials/src/keyring.rs` | KeyringBackend stub |
| `crates/prism-credentials/src/file.rs` | EncryptedFileBackend stub |
| `crates/prism-credentials/src/index.rs` | CredentialIndex stub |
| `crates/prism-credentials/src/probe.rs` | probe_keyring() stub |
| `crates/prism-credentials/src/selector.rs` | BackendSelector stub |
| `crates/prism-credentials/src/tests/proptest_crypto.rs` | VP-034, VP-035 proptests |
| `crates/prism-credentials/src/tests/store_tests.rs` | BC/AC unit tests |
