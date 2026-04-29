//! CredentialStore async trait — BC-2.03.001, BC-3.2.002
//!
//! ## CredentialStore (legacy, OrgSlug-keyed)
//! Defines the four-operation interface (`get`, `set`, `delete`, `list`) plus
//! `exists` for all credential backends. Every operation requires a `&OrgSlug`
//! parameter, enforcing namespace isolation at the type level.
//!
//! ## CredentialStoreOrgId (new, OrgId-keyed, S-3.1.04)
//! Mirrors `CredentialStore` but accepts `&OrgId` (UUID) instead of
//! `&OrgSlug`. This is the target interface after the BC-3.2.002 migration.
//! All methods are `todo!()` stubs pending Red Gate test passage.
//!
//! Story: S-1.06 | BC: BC-2.03.001
//! Story: S-3.1.04 | BC: BC-3.2.002

use async_trait::async_trait;
use prism_core::{OrgId, OrgSlug, PrismError};
use secrecy::SecretString;

use crate::namespace::CredentialName;

/// Tenant-scoped credential store interface.
///
/// All implementations MUST be `Send + Sync` (used from async tokio tasks).
#[async_trait]
pub trait CredentialStore: Send + Sync {
    /// Retrieve a credential value by (tenant, sensor, name).
    ///
    /// Returns `Ok(Some(value))` if found, `Ok(None)` if not present.
    ///
    /// STUB — unimplemented!() in both backends.
    async fn get(
        &self,
        tenant: &OrgSlug,
        sensor: &str,
        name: &CredentialName,
    ) -> Result<Option<SecretString>, PrismError>;

    /// Store or overwrite a credential. Previous value is not recoverable.
    ///
    /// STUB — unimplemented!() in both backends.
    async fn set(
        &self,
        tenant: &OrgSlug,
        sensor: &str,
        name: &CredentialName,
        value: SecretString,
    ) -> Result<(), PrismError>;

    /// Delete a credential. Returns `true` if deleted, `false` if not found
    /// (idempotent — BC-2.03.001 EC-03-002).
    ///
    /// STUB — unimplemented!() in both backends.
    async fn delete(
        &self,
        tenant: &OrgSlug,
        sensor: &str,
        name: &CredentialName,
    ) -> Result<bool, PrismError>;

    /// List all credential (sensor, name) pairs for a tenant.
    ///
    /// Returns metadata only — no values are returned.
    ///
    /// STUB — unimplemented!() in both backends.
    async fn list(&self, tenant: &OrgSlug) -> Result<Vec<(String, CredentialName)>, PrismError>;

    /// Check whether a credential exists. Used by S-5.05 config validation.
    ///
    /// STUB — unimplemented!() in both backends.
    async fn exists(
        &self,
        tenant: &OrgSlug,
        sensor: &str,
        name: &CredentialName,
    ) -> Result<bool, PrismError>;
}

// ---------------------------------------------------------------------------
// S-3.1.04 / BC-3.2.002 — OrgId-keyed credential store (migration target)
// ---------------------------------------------------------------------------

/// OrgId-keyed credential store interface (BC-3.2.002).
///
/// Mirrors [`CredentialStore`] but accepts `&OrgId` (stable UUID) instead of
/// `&OrgSlug` (mutable display string). This is the authoritative interface
/// after the ADR-006 §4 Step 3 migration.
///
/// All method bodies are `todo!()` stubs during the Stub Architect phase
/// (S-3.1.04). The Red Gate test suite in `tests/bc_3_2_002_org_id_namespace.rs`
/// drives the implementation.
///
/// # Architecture Compliance Rule
/// `prism-credentials` MUST NOT import `OrgRegistry`. Callers obtain a
/// resolved `OrgId` via `OrgRegistry::resolve(slug)` before calling these
/// methods (ADR-006 §2.3).
#[async_trait]
pub trait CredentialStoreOrgId: Send + Sync {
    /// Retrieve a credential by (`org_id`, `sensor`, `name`).
    ///
    /// Returns `Ok(Some(value))` if a credential is stored under namespace
    /// `"{org_id_uuid}/{sensor}/{name}"`, `Ok(None)` if not present.
    ///
    /// # Contract
    /// BC-3.2.002 postcondition 1 — lookup is isolated by `OrgId` UUID;
    /// credentials stored under `org_id_B` are never returned for `org_id_A`.
    ///
    /// STUB — todo!() pending Red Gate test passage.
    async fn get_by_org(
        &self,
        org_id: &OrgId,
        sensor: &str,
        name: &CredentialName,
    ) -> Result<Option<SecretString>, PrismError>;

    /// Store or overwrite a credential under `"{org_id_uuid}/{sensor}/{name}"`.
    ///
    /// Previous value is not recoverable after overwrite.
    ///
    /// STUB — todo!() pending Red Gate test passage.
    async fn set_by_org(
        &self,
        org_id: &OrgId,
        sensor: &str,
        name: &CredentialName,
        value: SecretString,
    ) -> Result<(), PrismError>;

    /// Delete a credential under `"{org_id_uuid}/{sensor}/{name}"`.
    ///
    /// Returns `true` if deleted, `false` if not found (idempotent).
    ///
    /// STUB — todo!() pending Red Gate test passage.
    async fn delete_by_org(
        &self,
        org_id: &OrgId,
        sensor: &str,
        name: &CredentialName,
    ) -> Result<bool, PrismError>;

    /// List all credential (`sensor`, `name`) pairs for an org.
    ///
    /// Returns metadata only — credential values are never returned.
    ///
    /// STUB — todo!() pending Red Gate test passage.
    async fn list_by_org(
        &self,
        org_id: &OrgId,
    ) -> Result<Vec<(String, CredentialName)>, PrismError>;

    /// Check whether a credential exists under `"{org_id_uuid}/{sensor}/{name}"`.
    ///
    /// STUB — todo!() pending Red Gate test passage.
    async fn exists_by_org(
        &self,
        org_id: &OrgId,
        sensor: &str,
        name: &CredentialName,
    ) -> Result<bool, PrismError>;
}
