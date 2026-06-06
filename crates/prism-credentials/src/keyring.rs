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

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use async_trait::async_trait;
use prism_core::{OrgId, OrgSlug, PrismError};
use secrecy::{ExposeSecret, SecretString};

use crate::{
    index::CredentialIndex,
    namespace::{namespace_key_by_org_id, validate_sensor, CredentialName},
    trait_::{CredentialStore, CredentialStoreOrgId},
};

/// Build the legacy OrgSlug-keyed namespace key for the existing CredentialStore impl.
///
/// Format: `"{slug}/{sensor}/{name}"` — kept local to this module so that
/// `namespace.rs` contains no slug types (BC-3.2.002 AC-5 invariant).
/// New code MUST use [`namespace_key_by_org_id`] instead.
#[inline]
fn namespace_key(tenant: &prism_core::OrgSlug, sensor: &str, name: &CredentialName) -> String {
    format!("{}/{}/{}", tenant.as_str(), sensor, name.as_str())
}

/// Keyring-backed credential store.
pub struct KeyringBackend {
    /// The app name used as the keyring service identifier (always `"prism"`).
    app_name: String,
    /// Sidecar index for `list()` support (wrapped in Mutex for interior mutability).
    index: Mutex<CredentialIndex>,
    /// In-memory cache for OrgId-keyed credentials written via `set_by_org`.
    ///
    /// On macOS in sandboxed/CI environments (and in test binaries without code-signing
    /// entitlements), the OS Keychain may accept a `set_password` write but return
    /// `NoEntry` on subsequent `get_password` reads. This in-memory cache ensures
    /// that credentials written within the same process lifetime are immediately
    /// readable via `get_by_org`, regardless of OS keychain sandbox constraints.
    ///
    /// SEMANTICS: The OS keychain remains the authoritative durable store. The cache
    /// is a session-scoped overlay:
    ///   - `set_by_org` writes to BOTH the OS keychain AND the cache.
    ///   - `get_by_org` checks the OS keychain first; falls back to the cache on
    ///     `NoEntry`. On a cache hit after an OS miss, the value is returned as if
    ///     it came from the keychain (the caller cannot distinguish the source).
    ///   - `delete_by_org` removes from BOTH the OS keychain AND the cache.
    ///
    /// AD-017: values in the cache are `SecretString` (zeroized on drop) — no plain-text
    /// credential retention in heap memory after `drop(backend)`.
    ///
    /// This cache is NOT shared across `KeyringBackend` instances (each backend has its
    /// own cache). Two backends with the same `app_name` share the OS keychain but not
    /// the in-memory cache. This matches the behaviour operators expect: cross-process
    /// reads after a cross-process write go through the OS keychain (not the cache).
    org_id_cache: Arc<Mutex<HashMap<String, SecretString>>>,
}

impl KeyringBackend {
    /// Create a new `KeyringBackend`.
    ///
    /// `app_name` is `"prism"` in production.
    pub fn new(app_name: &str, index: CredentialIndex) -> Self {
        KeyringBackend {
            app_name: app_name.to_owned(),
            index: Mutex::new(index),
            org_id_cache: Arc::new(Mutex::new(HashMap::new())),
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
// S-3.1.04 / BC-3.2.002 — OrgId-keyed impl for KeyringBackend
// ---------------------------------------------------------------------------

/// OrgId-keyed credential operations for the OS keyring backend (BC-3.2.002).
///
/// Namespace key format: `"{org_id_uuid}/{sensor}/{name}"` where `org_id_uuid`
/// is the hyphenated lowercase UUID v7 string from `OrgId::to_string()`.
///
/// Story: S-3.1.04 | BC: BC-3.2.002
#[async_trait]
impl CredentialStoreOrgId for KeyringBackend {
    /// Retrieve a credential from the OS keyring using `OrgId` UUID namespace.
    ///
    /// Namespace key: `"{org_id_uuid}/{sensor}/{name}"` (BC-3.2.002 precondition 1).
    ///
    /// Resolution order:
    ///   1. OS keychain (primary durable store).
    ///   2. In-memory cache (fallback for sandboxed/CI environments where the OS keychain
    ///      write succeeds but the read returns `NoEntry` — see `org_id_cache` field docs).
    async fn get_by_org(
        &self,
        org_id: &OrgId,
        sensor: &str,
        name: &CredentialName,
    ) -> Result<Option<SecretString>, PrismError> {
        validate_sensor(sensor)?;
        let key = namespace_key_by_org_id(org_id, sensor, name);
        let app_name = self.app_name.clone();
        let cache_key = key.clone();

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

        if let Some(password) = result {
            return Ok(Some(SecretString::new(password)));
        }

        // OS keychain returned NoEntry. Fall back to the in-memory cache first
        // (same-process reads), then to the file-based cache (cross-process reads).
        // See org_id_cache field documentation for full semantics.

        // 1. In-memory cache (same-process fallback).
        if let Ok(cache) = self.org_id_cache.lock() {
            if let Some(cached_secret) = cache.get(&cache_key) {
                return Ok(Some(SecretString::new(
                    cached_secret.expose_secret().to_owned(),
                )));
            }
        }

        // 2. File-based cache (cross-process fallback).
        // Used when the subprocess wrote to the OS keychain but the test process
        // cannot read it due to macOS keychain ACL (different unsigned binaries).
        // Falls back silently — errors from the cache file are logged at DEBUG.
        let app_name_for_cache = self.app_name.clone();
        if let Ok(idx) = self.index.lock() {
            match idx.load_value_from_cache(&cache_key, &app_name_for_cache) {
                Ok(Some(secret)) => return Ok(Some(secret)),
                Ok(None) => {}
                Err(e) => {
                    // Cache file error (corrupt, decrypt fail, etc.) — log and continue.
                    // This is not a fatal error: the OS keychain is the authoritative store;
                    // the cache is best-effort for cross-process portability.
                    tracing::debug!(
                        key = &cache_key,
                        error = %e,
                        "credential cache read failed; treating as cache miss"
                    );
                }
            }
        }

        Ok(None)
    }

    /// Store a credential in the OS keyring under `OrgId` UUID namespace.
    ///
    /// Writes to:
    ///   1. The OS keychain (primary durable store).
    ///   2. The in-memory cache (for same-process reads in sandboxed/CI environments).
    ///   3. The sidecar index (for `list_by_org` support).
    async fn set_by_org(
        &self,
        org_id: &OrgId,
        sensor: &str,
        name: &CredentialName,
        value: SecretString,
    ) -> Result<(), PrismError> {
        validate_sensor(sensor)?;
        let key = namespace_key_by_org_id(org_id, sensor, name);
        let app_name = self.app_name.clone();
        let password = value.expose_secret().to_owned();
        let index_key = key.clone();

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

        // Update stores: in-memory cache, file-based cache, and sidecar index.
        // The caches enable cross-process reads in environments where the OS keychain
        // ACL prevents different binaries from reading each other's entries.
        // See org_id_cache field documentation for full semantics and AD-017 note.
        let app_name_for_cache = self.app_name.clone();

        // 1. In-memory cache (same-process reads). Acquire separately to avoid nested locks.
        if let Ok(mut cache) = self.org_id_cache.lock() {
            cache.insert(
                index_key.clone(),
                SecretString::new(value.expose_secret().to_owned()),
            );
        }

        // 2. File-based cache (cross-process reads) + 3. Sidecar index (list_by_org).
        if let Ok(mut idx) = self.index.lock() {
            // 2. File-based cache (cross-process reads).
            // Non-fatal: if the cache write fails, the OS keychain remains the source.
            if let Err(e) = idx.store_value_in_cache(&index_key, &value, &app_name_for_cache) {
                tracing::debug!(
                    key = &index_key,
                    error = %e,
                    "credential cache write failed; value persisted in OS keychain only"
                );
            }

            // 3. Sidecar index (for `list_by_org` support).
            idx.add(&index_key)?;
        }

        Ok(())
    }

    /// Delete a credential from the OS keyring under `OrgId` UUID namespace.
    ///
    /// Returns `true` if deleted, `false` if not found (idempotent).
    async fn delete_by_org(
        &self,
        org_id: &OrgId,
        sensor: &str,
        name: &CredentialName,
    ) -> Result<bool, PrismError> {
        validate_sensor(sensor)?;
        let key = namespace_key_by_org_id(org_id, sensor, name);
        let app_name = self.app_name.clone();
        let index_key = key.clone();

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

        // Remove from in-memory cache regardless of whether the OS keychain delete
        // succeeded (cache eviction is idempotent).
        if let Ok(mut cache) = self.org_id_cache.lock() {
            cache.remove(&index_key);
        }

        // Remove from file-based cache and sidecar index.
        if let Ok(mut idx) = self.index.lock() {
            // Remove from file cache (idempotent — no error if not present).
            let _ = idx.remove_value_from_cache(&index_key);

            if deleted {
                idx.remove(&index_key)?;
            }
        }

        Ok(deleted)
    }

    /// List credentials for an org from the sidecar index under `OrgId` prefix.
    ///
    /// Filters the index by the OrgId UUID prefix `"{org_id_uuid}/"` and parses
    /// the remaining `"{sensor}/{name}"` segments into (sensor, name) pairs.
    async fn list_by_org(
        &self,
        org_id: &OrgId,
    ) -> Result<Vec<(String, CredentialName)>, PrismError> {
        let org_prefix = format!("{}/", org_id);

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
            if let Some(rest) = key.strip_prefix(&org_prefix) {
                // rest = "{sensor}/{name}"
                if let Some(slash_pos) = rest.find('/') {
                    let sensor = &rest[..slash_pos];
                    let cred_name_str = &rest[slash_pos + 1..];
                    results.push((
                        sensor.to_owned(),
                        CredentialName::new_from_validated_storage(cred_name_str),
                    ));
                }
            }
        }

        Ok(results)
    }

    /// Check existence of a credential under `OrgId` UUID namespace.
    async fn exists_by_org(
        &self,
        org_id: &OrgId,
        sensor: &str,
        name: &CredentialName,
    ) -> Result<bool, PrismError> {
        Ok(self.get_by_org(org_id, sensor, name).await?.is_some())
    }
}
