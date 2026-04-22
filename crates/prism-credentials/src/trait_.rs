//! CredentialStore async trait — BC-2.03.001
//!
//! Defines the four-operation interface (`get`, `set`, `delete`, `list`) plus
//! `exists` for all credential backends. Every operation requires a `&TenantId`
//! parameter, enforcing namespace isolation at the type level.
//!
//! Story: S-1.06 | BC: BC-2.03.001

use async_trait::async_trait;
use prism_core::{PrismError, TenantId};
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
        tenant: &TenantId,
        sensor: &str,
        name: &CredentialName,
    ) -> Result<Option<SecretString>, PrismError>;

    /// Store or overwrite a credential. Previous value is not recoverable.
    ///
    /// STUB — unimplemented!() in both backends.
    async fn set(
        &self,
        tenant: &TenantId,
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
        tenant: &TenantId,
        sensor: &str,
        name: &CredentialName,
    ) -> Result<bool, PrismError>;

    /// List all credential (sensor, name) pairs for a tenant.
    ///
    /// Returns metadata only — no values are returned.
    ///
    /// STUB — unimplemented!() in both backends.
    async fn list(&self, tenant: &TenantId) -> Result<Vec<(String, CredentialName)>, PrismError>;

    /// Check whether a credential exists. Used by S-5.05 config validation.
    ///
    /// STUB — unimplemented!() in both backends.
    async fn exists(
        &self,
        tenant: &TenantId,
        sensor: &str,
        name: &CredentialName,
    ) -> Result<bool, PrismError>;
}
