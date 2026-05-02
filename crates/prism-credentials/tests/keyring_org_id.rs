//! Keyring backend OrgId-keyed credential tests — W3-FIX-CODE-003
//!
//! These tests exercise `KeyringBackend`'s `CredentialStoreOrgId` implementation
//! against a real OS keyring service (macOS Keychain, Linux libsecret/D-Bus,
//! Windows Credential Vault).
//!
//! ## Test strategy (EC-001)
//!
//! The `keyring` crate requires a live platform keyring service. On headless CI
//! runners (GitHub Actions, Docker) without a secret service, `keyring::Entry::new`
//! or `get_password` will fail. All tests are annotated `#[ignore]` so that
//! `cargo test` skips them by default. To run them on a machine with a keyring
//! service, use:
//!
//! ```text
//! cargo test -p prism-credentials --test keyring_org_id -- --ignored
//! ```
//!
//! ## BC Coverage
//!
//! | AC | Test | BC Clause |
//! |----|------|-----------|
//! | AC-001, AC-002 | `test_AC_001_keyring_org_id_namespaced_get_set_delete` | BC-3.2.002 postcondition 1 |
//! | AC-003, AC-002 | `test_AC_002_cross_org_isolation_org_a_credential_not_visible_to_org_b` | BC-3.2.002 postcondition 2 |
//! | AC-004 | `test_AC_003_namespace_format_matches_uuid_sensor_name_pattern` | BC-3.2.002 precondition 1 |
//!
//! Story: W3-FIX-CODE-003 | BC: BC-3.2.002

use prism_core::{CredentialName, OrgId};
use prism_credentials::{
    index::CredentialIndex, keyring::KeyringBackend, namespace::namespace_key_by_org_id,
    trait_::CredentialStoreOrgId,
};
use secrecy::{ExposeSecret, SecretString};
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Build a `KeyringBackend` backed by a temporary sidecar index file.
///
/// Uses `"prism-test"` as the app name to avoid polluting the production
/// keyring namespace during test runs.
fn make_keyring_backend(dir: &TempDir) -> KeyringBackend {
    let index_path = dir.path().join("test-credential-index.json");
    let index = CredentialIndex::new(index_path);
    KeyringBackend::new("prism-test", index)
}

/// Helper: a valid `CredentialName` for tests.
fn cred_name(s: &str) -> CredentialName {
    CredentialName::new_from_validated_storage(s)
}

// ---------------------------------------------------------------------------
// AC-001 / BC-3.2.002 postcondition 1 — set/get/delete round-trip
// ---------------------------------------------------------------------------

/// AC-001 / AC-002: `set_by_org` → `get_by_org` → `delete_by_org` round-trip
/// succeeds without panic (BC-3.2.002 postcondition 1).
///
/// - `set_by_org(org_id_a, "claroty", "bearer_token", SecretString("tok-A"))` succeeds.
/// - `get_by_org(org_id_a, "claroty", "bearer_token")` returns `Ok(Some("tok-A"))`.
/// - `delete_by_org(org_id_a, "claroty", "bearer_token")` succeeds and returns `true`.
/// - After delete, `get_by_org` returns `Ok(None)`.
///
/// Requires a live OS keyring service.
/// Run with: `cargo test --test keyring_org_id -- --ignored`
#[tokio::test]
#[ignore = "requires a live OS keyring service; run with --ignored on a machine with macOS Keychain / libsecret / Windows Credential Vault"]
async fn test_AC_001_keyring_org_id_namespaced_get_set_delete() {
    let dir = TempDir::new().unwrap();
    let backend = make_keyring_backend(&dir);
    let org_id_a = OrgId::new();
    let name = cred_name("bearer_token");
    let secret_val = "tok-A-W3-FIX-CODE-003";

    // AC-001 step 1: set_by_org must succeed without panic (no todo!() body).
    backend
        .set_by_org(
            &org_id_a,
            "claroty",
            &name,
            SecretString::new(secret_val.to_owned()),
        )
        .await
        .expect("set_by_org must succeed for a valid OrgId + sensor + name");

    // AC-001 step 2: get_by_org returns the stored value.
    let result = backend
        .get_by_org(&org_id_a, "claroty", &name)
        .await
        .expect("get_by_org must not return an error for a stored credential");

    assert!(
        result.is_some(),
        "get_by_org must return Some after set_by_org; got None"
    );
    assert_eq!(
        result.unwrap().expose_secret(),
        secret_val,
        "BC-3.2.002 postcondition 1: retrieved value must match stored value"
    );

    // AC-001 step 3: delete_by_org succeeds.
    let deleted = backend
        .delete_by_org(&org_id_a, "claroty", &name)
        .await
        .expect("delete_by_org must not return an error for an existing credential");

    assert!(
        deleted,
        "delete_by_org must return true when the credential exists"
    );

    // AC-001 step 4: after delete, get_by_org returns None (no panic, no stale value).
    let after_delete = backend
        .get_by_org(&org_id_a, "claroty", &name)
        .await
        .expect("get_by_org after delete must not return an error");

    assert!(
        after_delete.is_none(),
        "get_by_org must return None after delete_by_org"
    );
}

// ---------------------------------------------------------------------------
// AC-002 / AC-003 / BC-3.2.002 postcondition 2 — cross-org isolation
// ---------------------------------------------------------------------------

/// AC-002 / AC-003: Cross-org isolation — org_a's credential is NOT visible to org_b
/// (BC-3.2.002 postcondition 2).
///
/// - Store `"tok-A"` for `org_id_a` under `("claroty", "bearer_token")`.
/// - `get_by_org(org_id_b, "claroty", "bearer_token")` returns `Ok(None)`.
/// - The keyring service name `"prism/{org_id_uuid}/{sensor}"` differs per org
///   because UUIDs differ — this is the physical isolation mechanism.
///
/// Requires a live OS keyring service.
/// Run with: `cargo test --test keyring_org_id -- --ignored`
#[tokio::test]
#[ignore = "requires a live OS keyring service; run with --ignored on a machine with macOS Keychain / libsecret / Windows Credential Vault"]
async fn test_AC_002_cross_org_isolation_org_a_credential_not_visible_to_org_b() {
    let dir = TempDir::new().unwrap();
    let backend = make_keyring_backend(&dir);
    let org_id_a = OrgId::new();
    let org_id_b = OrgId::new();
    let name = cred_name("bearer_token");

    // Store credential under org_a only.
    backend
        .set_by_org(
            &org_id_a,
            "claroty",
            &name,
            SecretString::new("tok-A-cross-org-test".to_owned()),
        )
        .await
        .expect("set_by_org for org_a must succeed");

    // AC-002 / AC-003: org_b must NOT see org_a's credential.
    let result = backend
        .get_by_org(&org_id_b, "claroty", &name)
        .await
        .expect("get_by_org for org_b must not error (NotFound is Ok(None))");

    assert!(
        result.is_none(),
        "BC-3.2.002 postcondition 2 violated: \
         get_by_org(org_b) returned org_a's credential — cross-org isolation broken. \
         org_a UUID: {org_id_a}, org_b UUID: {org_id_b}"
    );

    // Cleanup: delete org_a's test credential from the real keyring.
    let _ = backend.delete_by_org(&org_id_a, "claroty", &name).await;
}

// ---------------------------------------------------------------------------
// AC-003 / BC-3.2.002 precondition 1 — namespace format is UUID/sensor/name
// ---------------------------------------------------------------------------

/// AC-003 / AC-004: The keyring namespace key structure uses the OrgId UUID string
/// (not OrgSlug) as the org component (BC-3.2.002 precondition 1).
///
/// This is a pure unit test — it does NOT touch the OS keyring.
/// It verifies `namespace_key_by_org_id` produces the correct format that
/// `KeyringBackend` uses internally for its service names.
///
/// The keyring service name is `"prism/{org_id_uuid}/{sensor}"` and username is
/// `name` — derived from the namespace key `"{org_id_uuid}/{sensor}/{name}"`.
///
/// Does NOT require a live OS keyring service.
#[test]
fn test_AC_003_namespace_format_matches_uuid_sensor_name_pattern() {
    let org_id = OrgId::new();
    let org_uuid_str = org_id.to_string();
    let name = cred_name("bearer_token");

    let key = namespace_key_by_org_id(&org_id, "claroty", &name);

    // Verify the three-segment structure: {uuid}/{sensor}/{name}.
    let segments: Vec<&str> = key.splitn(3, '/').collect();
    assert_eq!(
        segments.len(),
        3,
        "AC-003: namespace key must have exactly 3 segments separated by '/'; got: {key:?}"
    );

    let (seg_uuid, seg_sensor, seg_name) = (segments[0], segments[1], segments[2]);

    // Segment 0: UUID v7 (36 chars, hyphenated, lowercase).
    assert_eq!(
        seg_uuid.len(),
        36,
        "AC-003: first segment must be 36-char UUID v7; got: {seg_uuid:?}"
    );
    assert_eq!(
        seg_uuid, org_uuid_str,
        "AC-003: first segment must equal OrgId UUID string (not slug); got: {seg_uuid:?}"
    );

    // Segment 1: sensor name.
    assert_eq!(
        seg_sensor, "claroty",
        "AC-003: second segment must be the sensor name; got: {seg_sensor:?}"
    );

    // Segment 2: credential name.
    assert_eq!(
        seg_name, "bearer_token",
        "AC-003: third segment must be the credential name; got: {seg_name:?}"
    );

    // BC-3.2.002 precondition 1: key starts with the UUID (not a slug prefix).
    assert!(
        key.starts_with(&org_uuid_str),
        "AC-003 / BC-3.2.002 precondition 1: namespace key must start with OrgId UUID; \
         key={key:?}, uuid={org_uuid_str:?}"
    );

    // AC-005 / BC-3.2.002 invariant 1: UUID must be 36 chars (not a short slug).
    // A slug is typically 3-30 chars; UUID v7 is always exactly 36.
    assert!(
        seg_uuid.len() > 30,
        "AC-003: namespace key org component appears to be a slug, not a UUID; \
         first segment is only {} chars: {seg_uuid:?}",
        seg_uuid.len()
    );
}
