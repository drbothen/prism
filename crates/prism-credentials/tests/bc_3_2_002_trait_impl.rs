//! Trait-contract tests for BC-3.2.002 — W3-FIX-CREDS-001 (regression coverage)
//!
//! These tests verify the `CredentialStoreOrgId` trait contract as specified in
//! the W3-FIX-CREDS-001 Acceptance Criteria (AC-001..006). The implementation
//! on `EncryptedFileBackend` was already complete (commit f923b086); these tests
//! serve as regression coverage confirming BC-3.2.002 is fully satisfied.
//!
//! ## AC Coverage
//!
//! | AC | Test | BC Clause |
//! |----|------|-----------|
//! | AC-001 | `test_BC_3_2_002_AC_001_get_by_org_returns_credential_stored_under_org_id_namespace` | BC-3.2.002 postcondition 1 |
//! | AC-002 | `test_BC_3_2_002_AC_002_set_by_org_stores_under_org_id_namespace` | BC-3.2.002 precondition 1 |
//! | AC-003 | `test_BC_3_2_002_AC_003_delete_by_org_removes_entry_subsequent_get_returns_none` | BC-3.2.002 invariant 3 |
//! | AC-003 | `test_BC_3_2_002_AC_003_double_delete_idempotent` | BC-3.2.002 invariant 3 (EC-002) |
//! | AC-004 | `test_BC_3_2_002_AC_004_cross_org_proptest_passes_canary` | BC-3.2.002 postcondition 2 / VP-3.2.002-01 |
//! | AC-005 | `test_BC_3_2_002_AC_005_get_by_org_returns_secret_string_debug_redacted` | BC-3.2.002 postcondition 4 |
//! | AC-006 | `test_BC_3_2_002_AC_006_slug_based_methods_compile_and_pass` | BC-3.2.002 invariant 1 |
//!
//! Story: W3-FIX-CREDS-001 | BC: BC-3.2.002

use prism_core::{CredentialName, OrgId, OrgSlug};
use prism_credentials::{
    file::EncryptedFileBackend,
    trait_::{CredentialStore, CredentialStoreOrgId},
};
use secrecy::{ExposeSecret, SecretString};
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Build a test-mode `EncryptedFileBackend` under a temporary directory.
fn make_backend(dir: &TempDir) -> EncryptedFileBackend {
    EncryptedFileBackend::new(
        dir.path().to_path_buf(),
        SecretString::new("test-passphrase-W3-FIX-CREDS-001".to_owned()),
    )
}

/// A `CredentialName` valid for test use.
fn cred_name(s: &str) -> CredentialName {
    CredentialName::new_from_validated_storage(s)
}

// ---------------------------------------------------------------------------
// AC-001 — get_by_org returns credential stored under {org_id_uuid}/{sensor}/{name}
// (BC-3.2.002 postcondition 1)
// ---------------------------------------------------------------------------

/// AC-001 / BC-3.2.002 postcondition 1:
/// `CredentialStoreOrgId::get_by_org(&org_id, sensor, name)` returns
/// `Ok(Some(SecretString))` containing the value stored under the namespace key
/// `"{org_id_uuid}/{sensor}/{name}"` (as produced by `namespace_key_by_org_id`).
#[tokio::test]
async fn test_BC_3_2_002_AC_001_get_by_org_returns_credential_stored_under_org_id_namespace() {
    let dir = TempDir::new().unwrap();
    let backend = make_backend(&dir);
    let org_id = OrgId::new();
    let name = cred_name("api_key");
    let secret_value = "ac-001-secret-value";

    backend
        .set_by_org(
            &org_id,
            "crowdstrike",
            &name,
            SecretString::new(secret_value.to_owned()),
        )
        .await
        .expect("AC-001: set_by_org must succeed");

    let result = backend
        .get_by_org(&org_id, "crowdstrike", &name)
        .await
        .expect("AC-001: get_by_org must not return an error");

    assert!(
        result.is_some(),
        "AC-001: get_by_org must return Some after set_by_org"
    );
    assert_eq!(
        result.unwrap().expose_secret(),
        secret_value,
        "AC-001: retrieved credential must match stored value"
    );
}

// ---------------------------------------------------------------------------
// AC-002 — set_by_org stores under "{org_id_uuid}/{sensor}/{name}" namespace
// (BC-3.2.002 precondition 1)
// ---------------------------------------------------------------------------

/// AC-002 / BC-3.2.002 precondition 1:
/// `CredentialStoreOrgId::set_by_org(&org_id, sensor, name, secret)` stores the
/// value under the key produced by `namespace_key_by_org_id(org_id, sensor, name)`.
/// A subsequent `get_by_org` with the same arguments returns the same secret.
/// Also verifies that the namespace key contains the org_id UUID string prefix.
#[tokio::test]
async fn test_BC_3_2_002_AC_002_set_by_org_stores_under_org_id_namespace() {
    let dir = TempDir::new().unwrap();
    let backend = make_backend(&dir);
    let org_id = OrgId::new();
    let name = cred_name("bearer_token");
    let secret_value = "ac-002-bearer-token";

    // Verify that the namespace key format matches {org_id_uuid}/{sensor}/{name}.
    let expected_key = prism_credentials::namespace_key_by_org_id(&org_id, "armis", &name);
    let org_uuid_str = org_id.to_string();
    assert!(
        expected_key.starts_with(&org_uuid_str),
        "AC-002: namespace key must start with OrgId UUID; key={expected_key:?}"
    );
    assert_eq!(
        expected_key,
        format!("{org_uuid_str}/armis/bearer_token"),
        "AC-002: namespace key must follow {{org_id_uuid}}/{{sensor}}/{{name}} format"
    );

    // Round-trip: set then get returns the same secret.
    backend
        .set_by_org(
            &org_id,
            "armis",
            &name,
            SecretString::new(secret_value.to_owned()),
        )
        .await
        .expect("AC-002: set_by_org must succeed");

    let result = backend
        .get_by_org(&org_id, "armis", &name)
        .await
        .expect("AC-002: get_by_org must not return an error");

    assert_eq!(
        result.unwrap().expose_secret(),
        secret_value,
        "AC-002: round-trip set→get must return same secret"
    );
}

// ---------------------------------------------------------------------------
// AC-003 — delete_by_org removes entry; subsequent get returns None
// (BC-3.2.002 invariant 3)
// ---------------------------------------------------------------------------

/// AC-003 / BC-3.2.002 invariant 3:
/// After `delete_by_org(&org_id, sensor, name)` returns `Ok(true)`,
/// a subsequent `get_by_org(&org_id, sensor, name)` returns `Ok(None)`.
#[tokio::test]
async fn test_BC_3_2_002_AC_003_delete_by_org_removes_entry_subsequent_get_returns_none() {
    let dir = TempDir::new().unwrap();
    let backend = make_backend(&dir);
    let org_id = OrgId::new();
    let name = cred_name("client_secret");

    // Set then delete.
    backend
        .set_by_org(
            &org_id,
            "claroty",
            &name,
            SecretString::new("ac-003-value".to_owned()),
        )
        .await
        .expect("AC-003: set_by_org must succeed");

    let deleted = backend
        .delete_by_org(&org_id, "claroty", &name)
        .await
        .expect("AC-003: delete_by_org must not return an error");

    assert!(
        deleted,
        "AC-003: delete_by_org must return true when credential exists"
    );

    // Subsequent get must return None.
    let result = backend
        .get_by_org(&org_id, "claroty", &name)
        .await
        .expect("AC-003: get_by_org after delete must not return an error");

    assert!(
        result.is_none(),
        "AC-003: get_by_org must return None after delete_by_org (BC-3.2.002 invariant 3)"
    );
}

/// AC-003 (EC-002) / BC-3.2.002 invariant 3:
/// Double-delete (calling `delete_by_org` on an already-deleted key) does NOT panic.
/// Returns `Ok(false)` (idempotent).
#[tokio::test]
async fn test_BC_3_2_002_AC_003_double_delete_idempotent() {
    let dir = TempDir::new().unwrap();
    let backend = make_backend(&dir);
    let org_id = OrgId::new();
    let name = cred_name("api_key");

    // Store and then delete once.
    backend
        .set_by_org(
            &org_id,
            "crowdstrike",
            &name,
            SecretString::new("ac-003-ec002-value".to_owned()),
        )
        .await
        .expect("AC-003 EC-002: set_by_org must succeed");

    let first_delete = backend
        .delete_by_org(&org_id, "crowdstrike", &name)
        .await
        .expect("AC-003 EC-002: first delete_by_org must not return an error");
    assert!(first_delete, "AC-003 EC-002: first delete must return true");

    // Second delete must return Ok(false) without panic.
    let second_delete = backend
        .delete_by_org(&org_id, "crowdstrike", &name)
        .await
        .expect("AC-003 EC-002: second delete_by_org must not return an error (no panic)");

    assert!(
        !second_delete,
        "AC-003 EC-002: second delete_by_org on same key must return false (idempotent)"
    );
}

// ---------------------------------------------------------------------------
// AC-004 — Cross-org proptest passes: Org A credential not retrievable by Org B
// (BC-3.2.002 postcondition 2 / VP-3.2.002-01)
// ---------------------------------------------------------------------------

/// AC-004 / BC-3.2.002 postcondition 2:
/// Credential stored under org_id_a is NOT returned by get_by_org(org_id_b, ...).
/// This is the single-case canary form; the full proptest (1000 cases) lives
/// in `bc_3_2_002_org_id_namespace.rs::proptest_BC_3_2_002_vp_01_cross_org_isolation`.
#[tokio::test]
async fn test_BC_3_2_002_AC_004_cross_org_proptest_passes_canary() {
    let dir = TempDir::new().unwrap();
    let backend = make_backend(&dir);
    let org_a = OrgId::new();
    let org_b = OrgId::new();
    let name = cred_name("api_key");
    let secret_for_a = "ac-004-secret-for-org-a-only";

    // Store under org_a only.
    backend
        .set_by_org(
            &org_a,
            "claroty",
            &name,
            SecretString::new(secret_for_a.to_owned()),
        )
        .await
        .expect("AC-004: set_by_org for org_a must succeed");

    // org_b must not see org_a's credential.
    let result = backend
        .get_by_org(&org_b, "claroty", &name)
        .await
        .expect("AC-004: get_by_org(org_b) must not return an error");

    assert!(
        result.is_none(),
        "AC-004 VP-3.2.002-01: credential stored under org_a must NOT be returned \
         by get_by_org(org_b); cross-org isolation violated"
    );

    // Confirm org_a can still retrieve its own credential (not destroyed by the miss).
    let org_a_result = backend
        .get_by_org(&org_a, "claroty", &name)
        .await
        .expect("AC-004: get_by_org(org_a) must not return an error");

    assert_eq!(
        org_a_result.unwrap().expose_secret(),
        secret_for_a,
        "AC-004: org_a credential must remain intact after org_b lookup miss"
    );
}

// ---------------------------------------------------------------------------
// AC-005 — Credential bytes returned as SecretString; no leak in Debug
// (BC-3.2.002 postcondition 4)
// ---------------------------------------------------------------------------

/// AC-005 / BC-3.2.002 postcondition 4:
/// `get_by_org` returns `Ok(Some(SecretString))`. The `Debug` output of the
/// return type does NOT expose the raw secret bytes — `secrecy::SecretString`
/// renders as `"[REDACTED]"` in Debug, never as the raw credential value.
#[tokio::test]
async fn test_BC_3_2_002_AC_005_get_by_org_returns_secret_string_debug_redacted() {
    let dir = TempDir::new().unwrap();
    let backend = make_backend(&dir);
    let org_id = OrgId::new();
    let name = cred_name("api_key");
    let raw_secret = "super-secret-must-not-appear-in-debug-output";

    backend
        .set_by_org(
            &org_id,
            "cyberint",
            &name,
            SecretString::new(raw_secret.to_owned()),
        )
        .await
        .expect("AC-005: set_by_org must succeed");

    let result = backend
        .get_by_org(&org_id, "cyberint", &name)
        .await
        .expect("AC-005: get_by_org must not return an error");

    assert!(result.is_some(), "AC-005: get_by_org must return Some");

    let secret_string = result.unwrap();

    // Debug output of SecretString must NOT expose the raw secret.
    let debug_output = format!("{:?}", secret_string);
    assert!(
        !debug_output.contains(raw_secret),
        "AC-005 BC-3.2.002 postcondition 4: Debug output of SecretString must not \
         expose the raw credential value; got debug={debug_output:?}"
    );

    // The secret value itself is accessible only via expose_secret().
    assert_eq!(
        secret_string.expose_secret(),
        raw_secret,
        "AC-005: expose_secret() must return the original value"
    );
}

// ---------------------------------------------------------------------------
// AC-006 — Backwards-compat slug-based methods continue to compile and pass
// (BC-3.2.002 invariant 1)
// ---------------------------------------------------------------------------

/// AC-006 / BC-3.2.002 invariant 1:
/// The `CredentialStore::{get, set, delete}` methods keyed by `OrgSlug`
/// (slug-based) continue to compile and return correct results.
/// Adding `CredentialStoreOrgId` methods must NOT remove or break the slug-keyed paths.
#[tokio::test]
async fn test_BC_3_2_002_AC_006_slug_based_methods_compile_and_pass() {
    let dir = TempDir::new().unwrap();
    let backend = make_backend(&dir);
    let slug = OrgSlug::new("acme-corp");
    let name = cred_name("api_key");
    let secret_value = "ac-006-slug-keyed-secret";

    // set (OrgSlug-keyed) must succeed.
    backend
        .set(
            &slug,
            "crowdstrike",
            &name,
            SecretString::new(secret_value.to_owned()),
        )
        .await
        .expect("AC-006: CredentialStore::set (slug-keyed) must succeed");

    // get (OrgSlug-keyed) must return the value just stored.
    let result = backend
        .get(&slug, "crowdstrike", &name)
        .await
        .expect("AC-006: CredentialStore::get (slug-keyed) must not return an error");

    assert!(
        result.is_some(),
        "AC-006: CredentialStore::get must return Some after set"
    );
    assert_eq!(
        result.unwrap().expose_secret(),
        secret_value,
        "AC-006: CredentialStore::get must return the stored value"
    );

    // delete (OrgSlug-keyed) must remove the entry.
    let deleted = backend
        .delete(&slug, "crowdstrike", &name)
        .await
        .expect("AC-006: CredentialStore::delete (slug-keyed) must not return an error");

    assert!(
        deleted,
        "AC-006: CredentialStore::delete must return true when credential exists"
    );

    // Confirm the entry is gone.
    let after_delete = backend
        .get(&slug, "crowdstrike", &name)
        .await
        .expect("AC-006: CredentialStore::get after delete must not return an error");

    assert!(
        after_delete.is_none(),
        "AC-006: CredentialStore::get must return None after delete"
    );
}
