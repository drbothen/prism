// SPDX-License-Identifier: Apache-2.0
//! Red Gate tests for BC-2.06.003 Tier-3 OS-keyring credential resolution (ADR-034).
//!
//! **Red Gate discipline (BC-5.38.001):** These tests MUST FAIL before the Tier-3
//! implementation is in place. They fail because `resolve_credential` does not yet
//! implement the Tier-3 branch — it has a `let _ = (org_id, keyring);` stub that
//! falls through to Tier 4 (CRUD store), which also returns `Ok(None)` → `NotFound`.
//!
//! The implementer's job: replace the stub with the real `get_by_org` call (ADR-034 §D2).
//!
//! # Test → AC / BC mapping
//!
//! | Test | RG ID | AC | BC |
//! |------|-------|----|----|
//! | test_BC_2_06_003_tier3_credential_written_by_set_by_org_is_resolved | RG-034-001 | AC-009 | BC-2.06.003 Tier-3 postcondition |
//! | test_BC_2_06_003_tier3_miss_falls_through_to_tier4 | RG-034-002 | AC-011 Case A | BC-2.06.003 Tier-3 miss postcondition |
//!
//! Story: S-DEMO-003 | ADR: ADR-034

#![allow(non_snake_case, clippy::unwrap_used)]

use std::sync::Arc;

use prism_core::{CredentialName, OrgId, OrgRegistry, OrgSlug};
use prism_credentials::{CredentialIndex, CredentialResolutionError, KeyringBackend};
use secrecy::ExposeSecret;

/// Build a test `KeyringBackend` with an in-memory index backed by a temp file.
///
/// Returns `(Arc<KeyringBackend>, temp_dir)` — `temp_dir` must be kept alive for
/// the duration of the test so the index file is not deleted.
fn make_test_keyring() -> (Arc<KeyringBackend>, tempfile::TempDir) {
    let tmp = tempfile::TempDir::new().expect("temp dir");
    let index_path = tmp.path().join("cred_index.json");
    let index = CredentialIndex::new(index_path);
    let backend = KeyringBackend::new("prism-test", index);
    (Arc::new(backend), tmp)
}

/// Build an `OrgRegistry` containing one (slug, OrgId) pair.
///
/// Returns `(Arc<OrgRegistry>, OrgId, OrgSlug)`.
fn make_org_registry_with(slug: &str) -> (Arc<OrgRegistry>, OrgId, OrgSlug) {
    let registry = OrgRegistry::new();
    let uuid = uuid::Uuid::now_v7();
    let org_id = OrgId::from_uuid(uuid);
    let org_slug = OrgSlug::new(slug);
    registry
        .register(org_slug.clone(), org_id)
        .expect("test fixture: OrgRegistry::register failed");
    (Arc::new(registry), org_id, org_slug)
}

// ---------------------------------------------------------------------------
// RG-034-001: End-to-end keyring write → resolution (CRIT-1 gap closure)
// ---------------------------------------------------------------------------

/// BC-2.06.003 Tier-3 postcondition: a credential written via `set_by_org` is
/// resolved at Tier-3 by `resolve_credential`.
///
/// **Red Gate:** FAILS before the Tier-3 branch is implemented because
/// `resolve_credential` stubs Tier 3 as a no-op `let _ = (org_id, keyring);`
/// and falls through to Tier 4 (CRUD store). Tier 4 returns `Ok(None)` →
/// `CredentialResolutionError::NotFound`. The assertion `Ok(secret)` fails.
///
/// **After implementation:** Tier-3 calls `keyring.get_by_org(org_id, "armis", "bearer_token")`,
/// finds the entry written by `set_by_org`, and returns `Ok(SecretString("test-secret-value"))`.
///
/// Canonical test vector from BC-2.06.003 §Canonical Test Vectors (Tier 3 keyring row):
///   org_slug = "acme", sensor = "armis", ref = "bearer_token",
///   setup = KeyringBackend::set_by_org(org_id, "armis", "bearer_token", "secret-value"),
///   expected = Ok(SecretString("secret-value")) + audit "keyring".
///
/// RG-034-001 (ADR-034 §Red Gate Tests); AC-009 of S-DEMO-003.
/// BC-2.06.003 Tier-3 postcondition: `get_by_org Ok(Some(secret))` → `Ok(secret)` + audit "keyring".
#[tokio::test]
async fn test_BC_2_06_003_tier3_credential_written_by_set_by_org_is_resolved() {
    let (keyring, _tmp) = make_test_keyring();
    let (_registry, org_id, _org_slug) = make_org_registry_with("acme");

    let secret_value = "test-secret-value-rg034001";
    let cred_name =
        CredentialName::new("bearer_token").expect("test fixture: CredentialName::new failed");

    // Write via set_by_org (OrgId-keyed namespace — ADR-034 §D3, CRIT-2 reconciliation).
    use prism_credentials::CredentialStoreOrgId;
    keyring
        .set_by_org(
            &org_id,
            "armis",
            &cred_name,
            secrecy::SecretString::new(secret_value.to_string()),
        )
        .await
        .expect("test fixture: set_by_org must succeed");

    // Ensure no Tier-1/Tier-2 env vars are set (this test exercises Tier 3 only).
    // Unset the per-client env var for "acme/armis/bearer_token".
    let tier2_var =
        prism_credentials::resolution::per_client_env_var("acme", "armis", "bearer_token");
    let _guard = std::env::remove_var(&tier2_var);
    let tier1_var =
        prism_credentials::resolution::per_client_file_env_var("acme", "armis", "bearer_token");
    let _guard2 = std::env::remove_var(&tier1_var);

    // Resolve via the updated signature (ADR-034 §D1).
    // Red Gate: Tier 3 stub is `let _ = (org_id, keyring);` — falls through to Tier 4 → NotFound.
    // After implementation: Tier 3 calls get_by_org → finds the entry → Ok(secret).
    let result = prism_credentials::resolve_credential(
        "acme",         // client_id (org slug)
        "armis",        // sensor_id
        "bearer_token", // credential_name
        Some(&org_id),  // pre-resolved OrgId (ADR-034 §D1)
        Some(&(keyring as Arc<dyn prism_credentials::CredentialStoreOrgId>)),
    )
    .await;

    // RED GATE ASSERTION — fails until Tier-3 is implemented.
    assert!(
        result.is_ok(),
        "RG-034-001 (CRIT-1 gap closure): resolve_credential must return Ok(secret) \
         for a credential written via set_by_org. \
         Before Tier-3 implementation: falls through to Tier 4 → NotFound. \
         Got: {:?}",
        result
    );

    let secret = result.unwrap();
    assert_eq!(
        secret.expose_secret(),
        secret_value,
        "RG-034-001: resolved credential value must equal the value written by set_by_org. \
         BC-2.06.003 Tier-3 postcondition: get_by_org Ok(Some(secret)) → Ok(secret)."
    );
}

// ---------------------------------------------------------------------------
// RG-034-002: Tier-3 keyring miss → Tier-4 fallthrough (AC-011 Case A)
// ---------------------------------------------------------------------------

/// BC-2.06.003 Tier-3 postcondition: when the keyring has no entry for the
/// OrgId-keyed key, Tier 3 falls through silently to Tier 4; Tier 4 also
/// misses → `CredentialResolutionError::NotFound` (NOT `BackendUnavailable`).
///
/// **Red Gate:** This test verifies the miss → fallthrough semantics. Currently the
/// Tier-3 stub already falls through (the no-op `let _ = (org_id, keyring);`
/// skips Tier 3). The Tier-4 CRUD store also returns Ok(None) → NotFound.
/// So THIS test MAY PASS even without implementation.
///
/// However, it is included as a Red Gate because the implementer must not accidentally
/// change a Tier-3 miss (Ok(None) from get_by_org) into a BackendUnavailable error.
/// If the Tier-3 implementation uses `.unwrap()` or panics on miss, this test catches it.
///
/// Canonical test vector: org_slug = "acme", sensor = "armis", ref = "bearer_token",
///   no keyring entry, CRUD store empty → `NotFound` (not `BackendUnavailable`).
///
/// RG-034-002 (ADR-034 §Red Gate Tests); AC-011 Case A of S-DEMO-003.
/// BC-2.06.003 Tier-3 postcondition: `get_by_org Ok(None)` → fall through to Tier 4.
///
/// NOTE: This test has dual Red Gate behavior:
///   - With stub: PASSES (no-op fall-through + Tier-4 miss → NotFound).
///   - After implementation: must also PASS (real get_by_org miss → Tier-4 miss → NotFound).
///   - If implementer introduces a bug (miss → BackendUnavailable): FAILS (catching the bug).
#[tokio::test]
async fn test_BC_2_06_003_tier3_miss_falls_through_to_tier4() {
    let (keyring, _tmp) = make_test_keyring();
    let (_registry, org_id, _org_slug) = make_org_registry_with("acme");

    // DO NOT write any entry to the keyring — this tests the miss path.

    // Ensure no Tier-1/Tier-2 env vars are set.
    let tier2_var =
        prism_credentials::resolution::per_client_env_var("acme", "armis", "bearer_token");
    std::env::remove_var(&tier2_var);
    let tier1_var =
        prism_credentials::resolution::per_client_file_env_var("acme", "armis", "bearer_token");
    std::env::remove_var(&tier1_var);

    let result = prism_credentials::resolve_credential(
        "acme",
        "armis",
        "bearer_token",
        Some(&org_id),
        Some(&(keyring as Arc<dyn prism_credentials::CredentialStoreOrgId>)),
    )
    .await;

    // Must be NotFound (miss → fall-through → Tier-4 miss → NotFound).
    // Must NOT be BackendUnavailable (which would indicate an incorrect hard error on miss).
    assert!(
        result.is_err(),
        "RG-034-002: resolve_credential must return Err when keyring has no entry \
         and CRUD store is empty. Got Ok — unexpected credential."
    );

    let err = result.unwrap_err();
    assert!(
        matches!(err, CredentialResolutionError::NotFound { .. }),
        "RG-034-002 (AC-011 Case A): Tier-3 miss must fall through silently to Tier 4; \
         Tier-4 miss must produce NotFound — NOT BackendUnavailable. \
         Got: {:?}. \
         BC-2.06.003 Tier-3 postcondition: get_by_org Ok(None) → fall through to Tier 4.",
        err
    );

    // Double-check: must NOT be BackendUnavailable (that would mean the miss was treated as
    // a hard error — an ADR-034 §D4 violation).
    assert!(
        !matches!(err, CredentialResolutionError::BackendUnavailable { .. }),
        "RG-034-002: a Tier-3 keyring miss (Ok(None)) must NOT produce BackendUnavailable. \
         BackendUnavailable is reserved for backend errors (Err(...)). \
         Got BackendUnavailable: {:?}",
        err
    );
}
