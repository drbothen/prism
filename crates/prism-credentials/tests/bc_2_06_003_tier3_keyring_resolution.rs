// SPDX-License-Identifier: Apache-2.0
//! Red Gate tests for BC-2.06.003 Tier-3 OS-keyring credential resolution (ADR-034).
//!
//! # Test approach — in-memory trait double (SID-1 compliance)
//!
//! These tests inject an `InMemoryCredentialStore` (a `HashMap`-backed
//! `CredentialStoreOrgId` trait double) into `resolve_credential` instead of the
//! real `KeyringBackend`. This avoids the macOS unsigned-test-binary OS keychain
//! cross-process ACL limitation while still exercising the FULL Tier-3 logic branch
//! in `resolve_credential` end-to-end.
//!
//! The real-OS-keyring roundtrip test is in `keyring_org_id.rs` (all tests there are
//! `#[ignore]`'d with rationale: "requires a live OS keyring service").
//!
//! # Why the in-memory double gives load-bearing coverage
//!
//! `resolve_credential` calls `keyring.get_by_org(org_id, sensor, &cred_name)` via the
//! `CredentialStoreOrgId` trait — the production implementation (`KeyringBackend`) and
//! the test double (`InMemoryCredentialStore`) both implement the same trait contract.
//! The Tier-3 dispatch logic in `resolution.rs` is IDENTICAL regardless of which
//! implementation is injected. The trait boundary IS the seam.
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

use prism_core::{CredentialName, OrgId};
use prism_credentials::{CredentialResolutionError, CredentialStoreOrgId, InMemoryCredentialStore};
use secrecy::ExposeSecret;

// ---------------------------------------------------------------------------
// RG-034-001: End-to-end Tier-3 write → resolution via in-memory double
// ---------------------------------------------------------------------------

/// BC-2.06.003 Tier-3 postcondition: a credential written via `set_by_org` is
/// resolved at Tier-3 by `resolve_credential`.
///
/// **Why in-memory double?**
/// On macOS, unsigned test binaries cannot perform cross-process OS keychain reads
/// (ACL limitation). The real-keyring roundtrip test is in `keyring_org_id.rs`
/// (`#[ignore]`). This test uses `InMemoryCredentialStore` as a trait double,
/// which exercises the IDENTICAL Tier-3 dispatch logic in `resolution.rs`
/// (`keyring.get_by_org(...)`) without requiring OS keyring access.
///
/// **Before the Tier-3 implementation (red gate):** `resolve_credential` had a
/// `let _ = (org_id, keyring);` stub that fell through to Tier 4. This test
/// would fail because the credential was set in the in-memory store but
/// `resolve_credential` never called `get_by_org`.
///
/// **After implementation (current state):** Tier-3 calls `keyring.get_by_org`,
/// finds the entry, and returns `Ok(SecretString("test-secret-value-rg034001"))`.
///
/// Canonical test vector from BC-2.06.003 §Canonical Test Vectors (Tier 3 keyring row):
///   org_slug = "acme", sensor = "armis", ref = "bearer_token",
///   setup = set_by_org(org_id, "armis", "bearer_token", "secret-value"),
///   expected = Ok(SecretString("secret-value")) + audit "keyring".
///
/// RG-034-001 (ADR-034 §Red Gate Tests); AC-009 of S-DEMO-003.
/// BC-2.06.003 Tier-3 postcondition: `get_by_org Ok(Some(secret))` → `Ok(secret)` + audit "keyring".
#[tokio::test]
async fn test_BC_2_06_003_tier3_credential_written_by_set_by_org_is_resolved() {
    let store = Arc::new(InMemoryCredentialStore::new());

    let uuid = uuid::Uuid::now_v7();
    let org_id = OrgId::from_uuid(uuid);
    let org_slug = "acme";

    let secret_value = "test-secret-value-rg034001";
    let cred_name =
        CredentialName::new("bearer_token").expect("test fixture: CredentialName::new failed");

    // Write via set_by_org (OrgId-keyed namespace — ADR-034 §D3, CRIT-2 reconciliation).
    store
        .set_by_org(
            &org_id,
            "armis",
            &cred_name,
            secrecy::SecretString::new(secret_value.to_string()),
        )
        .await
        .expect("test fixture: set_by_org must succeed on InMemoryCredentialStore");

    // Ensure no Tier-1/Tier-2 env vars are set (this test exercises Tier 3 only).
    let tier2_var =
        prism_credentials::resolution::per_client_env_var(org_slug, "armis", "bearer_token");
    std::env::remove_var(&tier2_var);
    let tier1_var =
        prism_credentials::resolution::per_client_file_env_var(org_slug, "armis", "bearer_token");
    std::env::remove_var(&tier1_var);

    // Resolve via the Tier-3 path (ADR-034 §D1).
    // The in-memory double is injected as the `keyring` param — resolve_credential
    // calls `keyring.get_by_org(org_id, "armis", "bearer_token")` which returns
    // `Ok(Some(SecretString("test-secret-value-rg034001")))` from the in-memory store.
    let result = prism_credentials::resolve_credential(
        org_slug,       // client_id (org slug)
        "armis",        // sensor_id
        "bearer_token", // credential_name
        Some(&org_id),  // pre-resolved OrgId (ADR-034 §D1)
        Some(&(store as Arc<dyn CredentialStoreOrgId>)),
    )
    .await;

    // ASSERTION: Tier-3 found the credential.
    assert!(
        result.is_ok(),
        "RG-034-001 (CRIT-1 gap closure): resolve_credential must return Ok(secret) \
         for a credential written via set_by_org. \
         The in-memory double exercises the same Tier-3 dispatch logic as KeyringBackend. \
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
/// **Why in-memory double?**
/// Same rationale as RG-034-001: avoids macOS unsigned-test-binary OS keychain ACL.
/// An empty `InMemoryCredentialStore` returns `Ok(None)` from `get_by_org` — the
/// same behavior as a keyring miss (NoEntry). The Tier-3 miss logic in `resolution.rs`
/// is exercised identically.
///
/// Verifies that:
/// 1. A Tier-3 miss (Ok(None)) falls through to Tier 4 silently.
/// 2. A Tier-4 miss → NotFound (not BackendUnavailable).
/// 3. The implementer did not accidentally turn a miss into a hard error.
///
/// RG-034-002 (ADR-034 §Red Gate Tests); AC-011 Case A of S-DEMO-003.
/// BC-2.06.003 Tier-3 postcondition: `get_by_org Ok(None)` → fall through to Tier 4.
#[tokio::test]
async fn test_BC_2_06_003_tier3_miss_falls_through_to_tier4() {
    // Empty store — no entries. get_by_org returns Ok(None) (Tier-3 miss).
    let store = Arc::new(InMemoryCredentialStore::new());

    let uuid = uuid::Uuid::now_v7();
    let org_id = OrgId::from_uuid(uuid);
    let org_slug = "acme";

    // Ensure no Tier-1/Tier-2 env vars are set.
    let tier2_var =
        prism_credentials::resolution::per_client_env_var(org_slug, "armis", "bearer_token");
    std::env::remove_var(&tier2_var);
    let tier1_var =
        prism_credentials::resolution::per_client_file_env_var(org_slug, "armis", "bearer_token");
    std::env::remove_var(&tier1_var);

    // DO NOT write any entry — tests the miss path.
    let result = prism_credentials::resolve_credential(
        org_slug,
        "armis",
        "bearer_token",
        Some(&org_id),
        Some(&(store as Arc<dyn CredentialStoreOrgId>)),
    )
    .await;

    // Must be NotFound (Tier-3 miss → fall-through → Tier-4 miss → NotFound).
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

    // Double-check: must NOT be BackendUnavailable.
    assert!(
        !matches!(err, CredentialResolutionError::BackendUnavailable { .. }),
        "RG-034-002: a Tier-3 keyring miss (Ok(None)) must NOT produce BackendUnavailable. \
         BackendUnavailable is reserved for backend errors (Err(...)). \
         Got BackendUnavailable: {:?}",
        err
    );
}
