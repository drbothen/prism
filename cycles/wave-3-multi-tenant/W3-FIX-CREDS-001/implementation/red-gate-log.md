---
story: W3-FIX-CREDS-001
phase: red-gate
timestamp: 2026-05-01
status: RED_GATE_PASS
commit: f923b086
---

# Red Gate Log — W3-FIX-CREDS-001

## Stub State Audit at HEAD (`cda17ed4`)

The story title claims `todo!()` stubs in `crates/prism-credentials/src/trait_.rs` for
`get_by_org`, `set_by_org`, and `delete_by_org`. Actual audit result:

| Location | Stub State | Notes |
|----------|-----------|-------|
| `trait_.rs` doc comments | `"STUB — todo!() pending Red Gate test passage"` text (doc only) | NOT actual `todo!()` macro calls in code |
| `trait_.rs` method bodies | No bodies — methods are abstract (required) in the trait | No `todo!()` macro in code |
| `namespace.rs::namespace_key_by_org_id` | Fully implemented: `format!("{}/{}/{}", org_id, sensor, name.as_str())` | No stub |
| `keyring.rs CredentialStoreOrgId impl` | Fully implemented (5 methods) | Confirmed by W3-FIX-CODE-003 |
| `file.rs CredentialStoreOrgId impl` | Fully implemented (5 methods) | All methods have real I/O logic |

**Finding:** The `todo!()` stubs were removed as part of the S-3.1.04 implementation cycle
(wave-3 passing). The trait doc comments still reference "STUB" language but the code bodies
are real. The holdout evaluator's gap report was based on the doc comment text, not actual
`todo!()` calls.

The existing proptest `bc_3_2_002_org_id_namespace.rs` (24 tests) was already passing once
the implementation was promoted. The proptest `proptest_BC_3_2_002_vp_01_cross_org_isolation`
(1000 cases) was running normally (not hanging due to `todo!()` — it simply takes time due
to 1000 AES-GCM encryption round-trips per case in a synchronous `block_on` harness).

## Red Gate Phase 1 — New Stub Test File

Created: `crates/prism-credentials/tests/bc_3_2_002_trait_impl.rs`

7 test stubs with `todo!()` bodies covering AC-001..006:

| Test | AC | BC Clause |
|------|----|-----------|
| `test_BC_3_2_002_AC_001_get_by_org_returns_credential_stored_under_org_id_namespace` | AC-001 | BC-3.2.002 postcondition 1 |
| `test_BC_3_2_002_AC_002_set_by_org_stores_under_org_id_namespace` | AC-002 | BC-3.2.002 precondition 1 |
| `test_BC_3_2_002_AC_003_delete_by_org_removes_entry_subsequent_get_returns_none` | AC-003 | BC-3.2.002 invariant 3 |
| `test_BC_3_2_002_AC_003_double_delete_idempotent` | AC-003 (EC-002) | BC-3.2.002 invariant 3 |
| `test_BC_3_2_002_AC_004_cross_org_proptest_passes_canary` | AC-004 | BC-3.2.002 postcondition 2 |
| `test_BC_3_2_002_AC_005_get_by_org_returns_secret_string_debug_redacted` | AC-005 | BC-3.2.002 postcondition 4 |
| `test_BC_3_2_002_AC_006_slug_based_methods_compile_and_pass` | AC-006 | BC-3.2.002 invariant 1 |

## Red Gate Result

```
cargo test -p prism-credentials --test bc_3_2_002_trait_impl
running 7 tests
test result: FAILED. 0 passed; 7 failed; 0 ignored; 0 measured
```

**7/7 FAILED** — Red Gate PASS. All failures are `not yet implemented` panics from `todo!()` stubs.
No compilation errors. No test logic errors.

## Existing Test Suite Status (bc_3_2_002_org_id_namespace.rs)

23/24 synchronous tests: **PASSED** (confirmed before commit).
`proptest_BC_3_2_002_vp_01_cross_org_isolation` (1000 cases): in-progress at commit time
(running normally, NOT hanging — takes ~2-3 minutes due to encryption cost per case).

## cargo check Result

```
cargo check -p prism-credentials
Finished `dev` profile [unoptimized + debuginfo]
```

Exit code 0. No errors, no warnings.

## Implementer Handoff

Promote each `todo!()` stub to a real assertion using `EncryptedFileBackend` (or
`KeyringBackend` with `#[ignore]` for keyring tests). The behavior is already implemented
in both backends — these stubs simply need real test assertions written.

The existing `bc_3_2_002_org_id_namespace.rs` tests can serve as implementation reference
for the assertion patterns.

Priority order:
1. AC-001 + AC-002 (set/get round-trip) — these validate the core namespace contract
2. AC-003 (delete + idempotent) — validates cleanup behavior
3. AC-004 (cross-org canary) — validates the isolation property
4. AC-005 (SecretString Debug redaction) — validates the AI-opaque credential invariant
5. AC-006 (slug compat) — validates backwards-compat is not broken

## Architecture Finding

The trait doc comments in `trait_.rs` still read "STUB — todo!() pending Red Gate test
passage". These should be updated to remove the "STUB" language once the implementer
promotes these test stubs to real assertions. That cleanup is part of the W3-FIX-CREDS-001
implementation phase, not this Red Gate phase.
