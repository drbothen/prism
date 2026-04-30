---
story: S-3.1.04
phase: red-gate
timestamp: 2026-04-29
status: RED_GATE_PASS
---

# Red Gate Log — S-3.1.04

## Summary — Stub Architect phase (prior)

The Stub Architect phase for S-3.1.04 is complete. The OrgId-keyed credential
namespace API surface has been scaffolded with `todo!()` stubs across
`prism-credentials`.

## Red Gate Result (Test Writer phase)

`cargo test -p prism-credentials --test bc_3_2_002_org_id_namespace`

24 tests added in `crates/prism-credentials/tests/bc_3_2_002_org_id_namespace.rs`.
**24/24 FAILED** — Red Gate PASS.

All failures are `not yet implemented` panics from the stub bodies.
No compilation errors. No test logic errors. Prior 79 tests unaffected.

### Test names

```
test_BC_3_2_002_namespace_key_format_uses_org_id_uuid
test_BC_3_2_002_distinct_org_ids_produce_distinct_keys
test_BC_3_2_002_invariant_namespace_key_always_from_org_id
test_BC_3_2_002_invariant_no_slug_keyed_fallback_in_namespace_key
test_BC_3_2_002_tv_01_same_org_round_trip
test_BC_3_2_002_get_by_org_returns_credential_for_correct_org
test_BC_3_2_002_tv_02_cross_org_isolation
test_BC_3_2_002_cross_org_get_returns_not_found
test_BC_3_2_002_tv_04_rename_stability
test_BC_3_2_002_rename_stable_lookup
test_BC_3_2_002_credential_value_not_in_error_message
test_BC_3_2_002_ec_001_org_with_credentials
test_BC_3_2_002_ec_002_org_without_credentials
test_BC_3_2_002_tv_03_per_sensor_isolation
test_BC_3_2_002_ec_003_per_sensor_not_found
test_BC_3_2_002_ec_004_rename_slug_org_id_stable
test_BC_3_2_002_ec_005_sequential_slug_reuse_no_collision
test_BC_3_2_002_list_by_org_scoped_to_org
test_BC_3_2_002_delete_by_org_removes_only_target
test_BC_3_2_002_delete_by_org_idempotent_returns_false
test_BC_3_2_002_exists_by_org_after_set
test_BC_3_2_002_invariant_exists_by_org_keyed_by_org_id
test_BC_3_2_002_invariant_physical_isolation_by_namespace_prefix
proptest_BC_3_2_002_vp_01_cross_org_isolation   (1000 cases, proptest)
```

### BC-3.2.002 clause coverage

| Clause | Test |
|--------|------|
| Precondition 1 — key format uses UUID | `test_BC_3_2_002_namespace_key_format_uses_org_id_uuid` |
| Precondition 4 — no slug fallback | `test_BC_3_2_002_invariant_no_slug_keyed_fallback_in_namespace_key` |
| Postcondition 1 — get correct org | `test_BC_3_2_002_get_by_org_returns_credential_for_correct_org`, `test_BC_3_2_002_tv_01_same_org_round_trip` |
| Postcondition 2 — cross-org NotFound | `test_BC_3_2_002_cross_org_get_returns_not_found`, `test_BC_3_2_002_tv_02_cross_org_isolation` |
| Postcondition 3 — rename stable | `test_BC_3_2_002_rename_stable_lookup`, `test_BC_3_2_002_tv_04_rename_stability` |
| Postcondition 4 — value not in error | `test_BC_3_2_002_credential_value_not_in_error_message` |
| Invariant 1 — key always from OrgId | `test_BC_3_2_002_invariant_namespace_key_always_from_org_id`, `test_BC_3_2_002_distinct_org_ids_produce_distinct_keys` |
| Invariant 3 — physical isolation | `test_BC_3_2_002_invariant_physical_isolation_by_namespace_prefix`, `test_BC_3_2_002_list_by_org_scoped_to_org`, `test_BC_3_2_002_delete_by_org_removes_only_target` |
| Invariant 4 — exists keyed by OrgId | `test_BC_3_2_002_exists_by_org_after_set`, `test_BC_3_2_002_invariant_exists_by_org_keyed_by_org_id` |
| EC-001 | `test_BC_3_2_002_ec_001_org_with_credentials` |
| EC-002 | `test_BC_3_2_002_ec_002_org_without_credentials` |
| EC-003 | `test_BC_3_2_002_tv_03_per_sensor_isolation`, `test_BC_3_2_002_ec_003_per_sensor_not_found` |
| EC-004 | `test_BC_3_2_002_ec_004_rename_slug_org_id_stable` |
| EC-005 | `test_BC_3_2_002_ec_005_sequential_slug_reuse_no_collision` |
| VP-3.2.002-01 proptest | `proptest_BC_3_2_002_vp_01_cross_org_isolation` |
| delete idempotent | `test_BC_3_2_002_delete_by_org_idempotent_returns_false` |

### Implementer handoff

Implement in this order to make tests pass one at a time:

1. `namespace_key_by_org_id` in `src/namespace.rs` — pure `format!("{}/{}/{}", org_id, sensor, name.as_str())`.
2. `EncryptedFileBackend::get_by_org` / `set_by_org` / `delete_by_org` / `list_by_org` / `exists_by_org` in `src/file.rs`.
3. `KeyringBackend` OrgId methods in `src/keyring.rs` (not exercised by these tests but stubs remain).


## Stub Status

| File | Change | Compile Status |
|------|--------|----------------|
| `crates/prism-credentials/src/namespace.rs` | Added `namespace_key_by_org_id(&OrgId, ...)` stub | `cargo check` PASS |
| `crates/prism-credentials/src/trait_.rs` | Added `CredentialStoreOrgId` trait (5 methods, all `todo!()`) | `cargo check` PASS |
| `crates/prism-credentials/src/keyring.rs` | Added `impl CredentialStoreOrgId for KeyringBackend` (all `todo!()`) | `cargo check` PASS |
| `crates/prism-credentials/src/file.rs` | Added `impl CredentialStoreOrgId for EncryptedFileBackend` (all `todo!()`) | `cargo check` PASS |
| `crates/prism-credentials/src/lib.rs` | Re-exported `namespace_key_by_org_id` and `CredentialStoreOrgId` | `cargo check` PASS |

## cargo check Result

```
Checking prism-credentials v0.1.0 (crates/prism-credentials)
Checking prism-sensors v0.1.0 (crates/prism-sensors)
Finished `dev` profile [unoptimized + debuginfo] target(s) in 3.31s
```

Workspace `cargo check` passes with zero errors and zero warnings.

## Test Count Verification

Test executables (same as before stub phase — count UNCHANGED):
1. `unittests src/lib.rs` (prism_credentials unit tests)
2. `tests/bc_2_03_005_credential_crud.rs`
3. `tests/bc_2_03_006_credential_resolution.rs`
4. `tests/bc_2_03_007_secret_redaction.rs`
5. `tests/bc_2_03_009_resolve_secret.rs`
6. `tests/bc_2_03_010_audit_logging.rs`

No new test files were added in this phase. Test count is unchanged per
hard requirement.

## Red Gate Prediction

When the Test Writer phase adds `tests/bc_3_2_002_org_id_namespace.rs`, all
new tests will fail at runtime with `todo!()` panics from the stub methods.
The stubs call `namespace_key_by_org_id` (which is also `todo!()`) so even
the pure-function namespace tests will panic.

Expected failure pattern:
```
thread 'test_BC_3_2_002_namespace_key_uses_org_id_uuid' panicked at
'not yet implemented: S-3.1.04 stub: implement OrgId-keyed namespace key ...'
```

This constitutes a proper Red Gate: tests fail for the right reason
(unimplemented behavior) not for compilation errors.

## BC-3.2.002 Coverage Map

| BC Clause | Stub Target | Test (pending) |
|-----------|-------------|----------------|
| Precondition 1: `namespace_key` accepts `&OrgId` | `namespace_key_by_org_id` stub | `test_BC_3_2_002_namespace_key_uses_org_id_uuid` |
| Postcondition 1: `get_by_org(A)` returns stored cred | `get_by_org` stub | `test_BC_3_2_002_same_org_get_returns_stored_cred` |
| Postcondition 2: `get_by_org(B)` returns NotFound | `get_by_org` stub | `test_BC_3_2_002_cross_org_get_returns_not_found` |
| Postcondition 3: rename-stable lookup | `get_by_org` stub | `test_BC_3_2_002_rename_stable_lookup` |
| Postcondition 4: cred values not in errors | `CredentialStoreOrgId` error types | `test_BC_3_2_002_credential_value_not_in_error` |
| Invariant 1: namespace key always uses OrgId | `namespace_key_by_org_id` | grep CI check + `test_BC_3_2_002_namespace_key_uses_org_id_uuid` |
| Invariant 4: cache keyed by OrgId | n/a (no cache layer yet) | future story |

## Architecture Compliance

- `prism-credentials` does NOT import `OrgRegistry` (ADR-006 §2.3 enforced).
- The legacy `namespace_key(&OrgSlug, ...)` shim remains to prevent existing
  test compilation breakage.
- `OrgSlug` and `TenantId` do not appear in any new stub code paths.
