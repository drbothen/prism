//! KeyringBackend — OS keyring via keyring-rs 3.x — BC-2.03.002
//!
//! Wraps keyring-rs `keyring::Entry`. All methods delegate to the OS keyring
//! (macOS Keychain, Windows Credential Vault, Linux libsecret/D-Bus).
//!
//! Design notes:
//! - keyring-rs is synchronous; all calls MUST be wrapped in
//!   `tokio::task::spawn_blocking` to avoid blocking the async runtime.
//! - `list()` is not natively supported by keyring-rs; `CredentialIndex` is
//!   used as a sidecar plaintext JSON index of namespace keys (no values).
//!
//! Story: S-1.06 | BC: BC-2.03.002, BC-2.03.004

use async_trait::async_trait;
use prism_core::{OrgId, OrgSlug, PrismError};
use secrecy::{ExposeSecret, SecretString};
use std::sync::Mutex;

use crate::{
    index::CredentialIndex,
    namespace::{namespace_key, namespace_key_by_org_id, validate_sensor, CredentialName},
    trait_::{CredentialStore, CredentialStoreOrgId},
};

/// Keyring-backed credential store.
pub struct KeyringBackend {
    /// The app name used as the keyring service identifier (always `"prism"`).
    app_name: String,
    /// Sidecar index for `list()` support (wrapped in Mutex for interior mutability).
    index: Mutex<CredentialIndex>,
}

impl KeyringBackend {
    /// Create a new `KeyringBackend`.
    ///
    /// `app_name` is `"prism"` in production.
    pub fn new(app_name: &str, index: CredentialIndex) -> Self {
        KeyringBackend {
            app_name: app_name.to_owned(),
            index: Mutex::new(index),
        }
    }
}

#[async_trait]
impl CredentialStore for KeyringBackend {
    /// Retrieve credential from the OS keyring.
    ///
    /// - Returns `Ok(Some(value))` if entry exists.
    /// - Returns `Ok(None)` if `NoEntry` (EC-001 — not found is not an error).
    async fn get(
        &self,
        tenant: &OrgSlug,
        sensor: &str,
        name: &CredentialName,
    ) -> Result<Option<SecretString>, PrismError> {
        // SEC-001: validate sensor before any keyring key construction (BC-2.03.004).
        validate_sensor(sensor)?;
        let key = namespace_key(tenant, sensor, name);
        let app_name = self.app_name.clone();

        let result = tokio::task::spawn_blocking(move || {
            let entry = keyring::Entry::new(&app_name, &key).map_err(|e| {
                PrismError::CredentialStoreError {
                    backend: "keyring".to_owned(),
                    reason: format!("failed to create entry: {e}"),
                }
            })?;

            match entry.get_password() {
                Ok(password) => Ok(Some(password)),
                Err(keyring::Error::NoEntry) => Ok(None),
                Err(e) => Err(PrismError::CredentialStoreError {
                    backend: "keyring".to_owned(),
                    reason: format!("get_password failed: {e}"),
                }),
            }
        })
        .await
        .map_err(|e| PrismError::CredentialStoreError {
            backend: "keyring".to_owned(),
            reason: format!("spawn_blocking panicked: {e}"),
        })??;

        Ok(result.map(SecretString::new))
    }

    /// Store credential in the OS keyring.
    async fn set(
        &self,
        tenant: &OrgSlug,
        sensor: &str,
        name: &CredentialName,
        value: SecretString,
    ) -> Result<(), PrismError> {
        // SEC-001: validate sensor before any keyring key construction (BC-2.03.004).
        validate_sensor(sensor)?;
        let key = namespace_key(tenant, sensor, name);
        let app_name = self.app_name.clone();
        let password = value.expose_secret().to_owned();

        tokio::task::spawn_blocking(move || {
            let entry = keyring::Entry::new(&app_name, &key).map_err(|e| {
                PrismError::CredentialStoreError {
                    backend: "keyring".to_owned(),
                    reason: format!("failed to create entry: {e}"),
                }
            })?;

            entry
                .set_password(&password)
                .map_err(|e| PrismError::CredentialStoreError {
                    backend: "keyring".to_owned(),
                    reason: format!("set_password failed: {e}"),
                })
        })
        .await
        .map_err(|e| PrismError::CredentialStoreError {
            backend: "keyring".to_owned(),
            reason: format!("spawn_blocking panicked: {e}"),
        })??;

        // Update the sidecar index.
        if let Ok(mut idx) = self.index.lock() {
            idx.add(&namespace_key(tenant, sensor, name))?;
        }

        Ok(())
    }

    /// Delete credential from the OS keyring.
    ///
    /// Returns `true` if deleted, `false` if `NoEntry` (idempotent).
    async fn delete(
        &self,
        tenant: &OrgSlug,
        sensor: &str,
        name: &CredentialName,
    ) -> Result<bool, PrismError> {
        // SEC-001: validate sensor before any keyring key construction (BC-2.03.004).
        validate_sensor(sensor)?;
        let key = namespace_key(tenant, sensor, name);
        let app_name = self.app_name.clone();

        let deleted = tokio::task::spawn_blocking(move || {
            let entry = keyring::Entry::new(&app_name, &key).map_err(|e| {
                PrismError::CredentialStoreError {
                    backend: "keyring".to_owned(),
                    reason: format!("failed to create entry: {e}"),
                }
            })?;

            match entry.delete_credential() {
                Ok(()) => Ok(true),
                Err(keyring::Error::NoEntry) => Ok(false),
                Err(e) => Err(PrismError::CredentialStoreError {
                    backend: "keyring".to_owned(),
                    reason: format!("delete_credential failed: {e}"),
                }),
            }
        })
        .await
        .map_err(|e| PrismError::CredentialStoreError {
            backend: "keyring".to_owned(),
            reason: format!("spawn_blocking panicked: {e}"),
        })??;

        if deleted {
            // Remove from sidecar index.
            if let Ok(mut idx) = self.index.lock() {
                idx.remove(&namespace_key(tenant, sensor, name))?;
            }
        }

        Ok(deleted)
    }

    /// List credentials via the sidecar `CredentialIndex`.
    ///
    /// Returns (sensor, name) pairs by parsing namespace keys.
    async fn list(&self, tenant: &OrgSlug) -> Result<Vec<(String, CredentialName)>, PrismError> {
        let tenant_prefix = format!("{}/", tenant.as_str());

        let all_keys = {
            let mut idx = self
                .index
                .lock()
                .map_err(|e| PrismError::CredentialStoreError {
                    backend: "keyring".to_owned(),
                    reason: format!("index lock poisoned: {e}"),
                })?;
            idx.list()?
        };

        let mut results = Vec::new();
        for key in &all_keys {
            if let Some(rest) = key.strip_prefix(&tenant_prefix) {
                // rest = "{sensor}/{name}"
                if let Some(slash_pos) = rest.find('/') {
                    let sensor = &rest[..slash_pos];
                    let cred_name_str = &rest[slash_pos + 1..];
                    // Values in the index were written through namespace_key() from validated
                    // inputs — safe to reconstruct without re-validation (SEC-002).
                    results.push((
                        sensor.to_owned(),
                        CredentialName::new_from_validated_storage(cred_name_str),
                    ));
                }
            }
        }

        Ok(results)
    }

    /// Check existence via `get()`.
    async fn exists(
        &self,
        tenant: &OrgSlug,
        sensor: &str,
        name: &CredentialName,
    ) -> Result<bool, PrismError> {
        Ok(self.get(tenant, sensor, name).await?.is_some())
    }
}

// ---------------------------------------------------------------------------
// S-3.1.04 / BC-3.2.002 — OrgId-keyed impl for KeyringBackend (STUBS)
// ---------------------------------------------------------------------------

/// OrgId-keyed credential operations for the OS keyring backend.
///
/// All methods are `todo!()` stubs. Implementation is driven by the
/// `bc_3_2_002_org_id_namespace` Red Gate test suite in S-3.1.04.
#[async_trait]
impl CredentialStoreOrgId for KeyringBackend {
    /// Retrieve a credential from the OS keyring using `OrgId` UUID namespace.
    ///
    /// Namespace key: `"{org_id_uuid}/{sensor}/{name}"` (BC-3.2.002 precondition 1).
    ///
    /// STUB — todo!() pending Red Gate test passage (S-3.1.04).
    async fn get_by_org(
        &self,
        org_id: &OrgId,
        sensor: &str,
        name: &CredentialName,
    ) -> Result<Option<SecretString>, PrismError> {
        // Suppress unused variable warnings on the stubs so `cargo check` is clean.
        let _ = namespace_key_by_org_id(org_id, sensor, name);
        todo!(
            "S-3.1.04 stub: implement KeyringBackend::get_by_org — \
             resolve keyring entry under OrgId-keyed namespace"
        )
    }

    /// Store a credential in the OS keyring under `OrgId` UUID namespace.
    ///
    /// STUB — todo!() pending Red Gate test passage (S-3.1.04).
    async fn set_by_org(
        &self,
        org_id: &OrgId,
        sensor: &str,
        name: &CredentialName,
        value: SecretString,
    ) -> Result<(), PrismError> {
        let _ = (namespace_key_by_org_id(org_id, sensor, name), value);
        todo!(
            "S-3.1.04 stub: implement KeyringBackend::set_by_org — \
             store credential under OrgId-keyed namespace"
        )
    }

    /// Delete a credential from the OS keyring under `OrgId` UUID namespace.
    ///
    /// Returns `true` if deleted, `false` if not found (idempotent).
    ///
    /// STUB — todo!() pending Red Gate test passage (S-3.1.04).
    async fn delete_by_org(
        &self,
        org_id: &OrgId,
        sensor: &str,
        name: &CredentialName,
    ) -> Result<bool, PrismError> {
        let _ = namespace_key_by_org_id(org_id, sensor, name);
        todo!(
            "S-3.1.04 stub: implement KeyringBackend::delete_by_org — \
             delete credential under OrgId-keyed namespace"
        )
    }

    /// List credentials for an org from the sidecar index under `OrgId` prefix.
    ///
    /// STUB — todo!() pending Red Gate test passage (S-3.1.04).
    async fn list_by_org(
        &self,
        org_id: &OrgId,
    ) -> Result<Vec<(String, CredentialName)>, PrismError> {
        let _ = org_id;
        todo!(
            "S-3.1.04 stub: implement KeyringBackend::list_by_org — \
             filter index by OrgId UUID prefix"
        )
    }

    /// Check existence of a credential under `OrgId` UUID namespace.
    ///
    /// STUB — todo!() pending Red Gate test passage (S-3.1.04).
    async fn exists_by_org(
        &self,
        org_id: &OrgId,
        sensor: &str,
        name: &CredentialName,
    ) -> Result<bool, PrismError> {
        let _ = namespace_key_by_org_id(org_id, sensor, name);
        todo!(
            "S-3.1.04 stub: implement KeyringBackend::exists_by_org — \
             check keyring entry under OrgId-keyed namespace"
        )
    }
}
