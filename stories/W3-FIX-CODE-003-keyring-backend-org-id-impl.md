---
story_id: W3-FIX-CODE-003
title: "prism-credentials: implement KeyringBackend::CredentialStoreOrgId — replace todo!() stubs"
wave: 3.1
level: "L4"
target_module: prism-credentials
subsystems: [SS-03]
priority: P0
depends_on: []
blocks: []
estimated_days: 1
points: 3
status: merged
document_type: story
version: "1.0"
producer: story-writer
timestamp: "2026-05-01T00:00:00Z"
input-hash: ""
inputs:
  - .factory/cycles/wave-3-multi-tenant/gate-step-d-security-review.md
  - .factory/specs/behavioral-contracts/BC-3.2.002-per-org-credential-isolation.md
  - .factory/specs/architecture/decisions/ADR-006-multi-tenant-dtu-topology.md
traces_to: []
cycle: "v1.0.0-greenfield"
epic_id: "E-3.5"
phase: 3
behavioral_contracts:
  - BC-3.2.002
verification_properties: [VP-112, VP-113]
assumption_validations: []
risk_mitigations: []
anchor_bcs: [BC-3.2.002]
anchor_capabilities: [CAP-004]
anchor_subsystem: ["SS-03"]
tdd_mode: strict
---

# W3-FIX-CODE-003: prism-credentials — implement KeyringBackend::CredentialStoreOrgId, replace todo!() stubs

## Narrative

As a Prism credential subsystem maintainer, I want `KeyringBackend`'s implementation
of `CredentialStoreOrgId` to use real OrgId-keyed namespace logic (`"{org_id_uuid}/{sensor}/{name}"`
matching the `EncryptedFileBackend` pattern), so that a process configured to use the
OS keyring backend does not panic at runtime and per-org credential isolation is
enforced in both backends consistently.

## Objective

Gate Step D identified SEC-004 (MEDIUM, CWE-284, OWASP A01): the `KeyringBackend`
implementation of `CredentialStoreOrgId` in
`crates/prism-credentials/src/keyring_backend.rs` consists entirely of `todo!()`
stubs. ADR-006 §4 Step 3 designates `CredentialStoreOrgId` as the "authoritative
interface after migration." If `KeyringBackend` is selected and `get_by_org` or
`set_by_org` is called, the process panics.

The `EncryptedFileBackend` already implements `CredentialStoreOrgId` correctly using
`"{org_id_uuid}/{sensor}/{name}"` as the namespace key. The `KeyringBackend` must
follow the same pattern: convert `OrgId` to its UUID string representation, construct
the keyring service name as `"prism/{org_id_uuid}/{sensor}"` and the username as
`name`, matching the existing namespace semantics.

## Behavioral Contracts

| BC ID | Title | Relevant Clause |
|-------|-------|-----------------|
| BC-3.2.002 | Per-Org Credential Isolation via OrgId-Keyed Namespace | Precondition 1 (namespace key uses OrgId UUID), Postcondition 1 (`get(org_id_A, sensor, name)` returns correct credential), Postcondition 2 (`get(org_id_A, sensor, name)` returns `Err(NotFound)` for org_id_B's credential), Invariant 1 (slug-keyed fallback eliminated) |

## Acceptance Criteria

### AC-001: get_by_org/set_by_org/delete_by_org are fully implemented (traces to BC-3.2.002 postcondition 1)
`KeyringBackend::get_by_org(org_id, sensor, name)`, `set_by_org(org_id, sensor, name,
secret)`, and `delete_by_org(org_id, sensor, name)` are fully implemented with no
`todo!()` bodies. They use the namespace key `"{org_id_uuid}/{sensor}/{name}"` — the
same format as `EncryptedFileBackend` — mapping to keyring service `"prism/{org_id_uuid}/{sensor}"`
and keyring username `name`.

### AC-002: OrgId-keyed get/set/delete work without panic (traces to BC-3.2.002 postcondition 1)
A test `test_keyring_org_id_get_set_delete` in `tests/keyring_org_id.rs`:
- `set_by_org(org_id_a, "claroty", "bearer_token", SecretString::from("tok-A"))` succeeds.
- `get_by_org(org_id_a, "claroty", "bearer_token")` returns `Ok("tok-A")`.
- `delete_by_org(org_id_a, "claroty", "bearer_token")` succeeds.
- No `todo!()` panic occurs at any point.

### AC-003: Cross-org isolation maintained (traces to BC-3.2.002 postcondition 2)
A test `test_keyring_cross_org_isolation` in `tests/keyring_org_id.rs`:
- Store `"tok-A"` for `org_id_a` under `("claroty", "bearer_token")`.
- `get_by_org(org_id_b, "claroty", "bearer_token")` returns `Err(NotFound)` — not
  `"tok-A"`.
- The OrgId UUID being distinct is the isolation mechanism; the keyring service name
  `"prism/{org_id_uuid}/{sensor}"` differs per org.

### AC-004: Namespace key format matches EncryptedFileBackend (traces to BC-3.2.002 precondition 1)
The keyring namespace key structure uses the OrgId UUID string (not OrgSlug) as the
org component. This is verified by inspecting the keyring entry's service name in the
test (or by reading the implementation and asserting the format with a unit test).

### AC-005: Legacy CredentialStore (slug-keyed) not added to KeyringBackend (traces to BC-3.2.002 invariant 1)
The `KeyringBackend` implementation does NOT implement or call the legacy `CredentialStore`
(OrgSlug-keyed) trait for the `CredentialStoreOrgId` methods. The OrgId UUID is the
exclusive namespace component.

## Tasks

1. Read `crates/prism-credentials/src/keyring_backend.rs` (full file) to understand
   the existing `KeyringBackend` struct, its `CredentialStore` (legacy) implementation,
   and the `todo!()` stubs in `CredentialStoreOrgId`.
2. Read `crates/prism-credentials/src/encrypted_file_backend.rs` (or equivalent) to
   understand the `EncryptedFileBackend::CredentialStoreOrgId` implementation — use
   this as the model for the namespace key format.
3. Read `crates/prism-credentials/src/namespace.rs` (if it exists) to understand
   `namespace_key_by_org_id` — use this helper in `KeyringBackend` rather than
   constructing the namespace string manually.
4. Implement `KeyringBackend::get_by_org`:
   ```rust
   fn get_by_org(&self, org_id: &OrgId, sensor: &str, name: &str)
       -> Result<SecretString, CredentialError>
   {
       let service = format!("prism/{}/{}", org_id.as_uuid(), sensor);
       let entry = keyring::Entry::new(&service, name)?;
       let password = entry.get_password()?;
       Ok(SecretString::from(password))
   }
   ```
5. Implement `KeyringBackend::set_by_org` and `delete_by_org` using the same pattern.
6. Ensure `CredentialError::NotFound` is the correct error variant for
   `keyring::Error::NoEntry` — map the keyring error variants correctly.
7. Create `crates/prism-credentials/tests/keyring_org_id.rs` with tests from AC-002
   and AC-003. Note: keyring tests require a real OS keyring — gate them with
   `#[cfg(not(feature = "mock-keyring"))]` or use a mock/stub keyring for CI.
8. Run `cargo test -p prism-credentials --all-features` — all tests pass (including
   any existing keyring tests).
9. Open PR to `develop`.

## Architecture Mapping

| Component | Module | File(s) | Pure/Effectful |
|-----------|--------|---------|----------------|
| `KeyringBackend::CredentialStoreOrgId` impl | prism-credentials | `crates/prism-credentials/src/keyring_backend.rs` | Effectful (OS keyring I/O) |
| `namespace_key_by_org_id` helper (if present) | prism-credentials | `crates/prism-credentials/src/namespace.rs` | Pure (string construction) |
| Keyring org_id tests | prism-credentials | `crates/prism-credentials/tests/keyring_org_id.rs` | Effectful (OS keyring I/O) |

**Subsystem anchor justification:** SS-03 (Credential Management) owns this story's
scope because `prism-credentials` is the credential store crate and this fix is
entirely within its `keyring_backend.rs` implementation, per the ARCH-INDEX Subsystem
Registry definition of SS-03.

**Dependency anchor justification:** `depends_on: []` — the keyring backend stub
implementation is self-contained; it does not depend on the SEC-001/SEC-002/SEC-003
fixes or the CODE-001/CODE-002 changes. `blocks: []` — no other W3-FIX story requires
the keyring backend to be non-panicking first.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | OS keyring unavailable (e.g., headless CI runner with no secret service) | `get_by_org` returns `Err(CredentialError::BackendUnavailable)` or equivalent — not a panic. Test is gated behind a feature flag or skipped on CI if keyring is not available |
| EC-002 | `sensor` string contains `/` character | The service name `"prism/{org_id}/{sensor}"` would contain double slashes. Apply the same `validate_sensor` component check as `namespace_key_by_org_id` if it validates `[a-zA-Z0-9_-]` — return `Err(InvalidSensorName)` for invalid sensor strings |
| EC-003 | `name` contains characters unsupported by the OS keyring username field | Return `Err(CredentialError::InvalidName)` with a clear message; do not panic |
| EC-004 | `set_by_org` called with an empty `SecretString` | Store the empty secret; keyring-rs supports empty passwords. No special handling required unless the OS keyring rejects empty passwords (platform-dependent — document if so) |
| EC-005 | `delete_by_org` called for a credential that was never stored | Return `Err(CredentialError::NotFound)` — same as `EncryptedFileBackend` semantics; idempotent delete is not required |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| `KeyringBackend::get_by_org` | effectful-shell | Calls `keyring::Entry::get_password()` — OS keychain I/O |
| `KeyringBackend::set_by_org` | effectful-shell | Calls `keyring::Entry::set_password()` — OS keychain write |
| `KeyringBackend::delete_by_org` | effectful-shell | Calls `keyring::Entry::delete_credential()` — OS keychain delete |
| Namespace key construction | pure-core | `format!("prism/{}/{}", org_id.as_uuid(), sensor)` — no I/O |
| Test setup/teardown | effectful-shell | Creates and deletes real OS keyring entries during test |

## Token Budget Estimate

| Item | Estimated Tokens |
|------|-----------------|
| Story spec (this file) | ~2 800 |
| BC file (1 BC: BC-3.2.002) | ~1 500 |
| `keyring_backend.rs` (full file, estimated ~300 lines) | ~2 200 |
| `encrypted_file_backend.rs` (reference implementation, ~400 lines) | ~3 000 |
| `namespace.rs` helper (~80 lines) | ~600 |
| New test file | ~600 |
| Cargo output | ~500 |
| **Total** | **~11 200** |

Well within a single agent context window.

## Previous Story Intelligence

- **S-3.1.04 (credentials-org-id-boundary):** Introduced `CredentialStoreOrgId` and
  implemented it on `EncryptedFileBackend`. The `KeyringBackend` stubs were explicitly
  left as `todo!()` with the comment "pending Red Gate test passage." Red Gate tests
  for S-3.1.04 have since passed; this story completes the migration for the keyring
  backend.
- **WGS-W2-002 (Wave 2 fix):** Required `SecretString` for derived bearer tokens —
  the keyring backend must use `SecretString` as the return type for `get_by_org`, which
  is consistent with the `EncryptedFileBackend` pattern.
- **ADR-006 §4 Step 3:** Designates `CredentialStoreOrgId` as the authoritative
  interface. The legacy `CredentialStore` (slug-keyed) must not be used for new code
  paths. This story does NOT remove the legacy implementation — that migration is a
  separate post-wave-3 concern. It only ensures the OrgId-keyed path does not panic.

## Architecture Compliance Rules

- The keyring service name MUST use the OrgId UUID string as the org component — NOT
  the OrgSlug. This is consistent with ADR-006 §2.2 and `namespace_key_by_org_id`.
- `expose_secret()` MUST be called only at the single keyring write site
  (`entry.set_password(secret.expose_secret())`), following the SecretString convention
  established in Wave 2 (WGS-W2-002 fix).
- Do NOT implement `CredentialStore` (slug-keyed) on `KeyringBackend` in this story.
  Only `CredentialStoreOrgId` is in scope.
- `keyring::Entry::new` may return an error on some platforms (e.g., when the keyring
  service is not available). Map this to `CredentialError::BackendUnavailable` rather
  than unwrapping.

## Library & Framework Requirements

| Library | Version (workspace pin) | Purpose |
|---------|------------------------|---------|
| keyring-rs | workspace pin | `Entry::new`, `get_password`, `set_password`, `delete_credential` |
| secrecy (SecretString) | workspace pin | Return type for `get_by_org`; `expose_secret()` at write site |
| prism-core (OrgId) | workspace | `OrgId::as_uuid()` for UUID string extraction |

No new Cargo dependencies. All three are already workspace dependencies.

## File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `crates/prism-credentials/src/keyring_backend.rs` | Modify | Replace `todo!()` stubs with real `get_by_org`, `set_by_org`, `delete_by_org` |
| `crates/prism-credentials/tests/keyring_org_id.rs` | Create | New: AC-002 + AC-003 tests |

No other files require modification for this fix.

## Forbidden Dependencies

- Do NOT add any new Cargo dependencies to `prism-credentials` for this fix. All
  required crates (`keyring-rs`, `secrecy`, `prism-core`) are already in the workspace.
- Do NOT use `OrgSlug` as any component of the keyring service or username string in
  `CredentialStoreOrgId` methods — OrgId UUID only.
- Do NOT remove or modify the existing `CredentialStore` (slug-keyed) implementation
  on `KeyringBackend` — that migration is out of scope for this story.
