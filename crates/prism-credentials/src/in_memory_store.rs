//! In-memory `CredentialStoreOrgId` trait double for unit/integration tests.
//!
//! Provides a `HashMap<String, SecretString>`-backed implementation of the
//! `CredentialStoreOrgId` trait that does NOT touch the OS keyring. Used by:
//!
//! - `bc_2_06_003_tier3_keyring_resolution.rs` (RG-034-001, RG-034-002):
//!   inject this double to exercise the Tier-3 branch of `resolve_credential`
//!   without requiring real OS keyring access.
//!
//! - `bc_2_03_007_credential_set_org_id_keyed.rs` (RG-034-004):
//!   assert `set_by_org` was called with the OrgId-keyed namespace format.
//!
//! # AD-017 compliance
//!
//! `SecretString` values are stored in heap memory (zeroized on drop via
//! the `secrecy` crate's `Zeroize` impl). Values are never written to disk.
//! This double is TEST-ONLY (`#[cfg(test)]` gated in all callsites).
//!
//! # SID-1 compliance
//!
//! This double provides the load-bearing unit-test coverage for Tier-3 logic
//! that cannot be exercised via `#[ignore]`'d integration tests (which require
//! a live OS keyring service). See `keyring_org_id.rs` for the `#[ignore]`'d
//! real-keyring tests with rationale.

use std::{collections::HashMap, sync::Mutex};

use async_trait::async_trait;
use prism_core::{OrgId, PrismError};
use secrecy::{ExposeSecret, SecretString};

use crate::{namespace::CredentialName, trait_::CredentialStoreOrgId};

/// In-memory credential store for testing. Thread-safe via `Mutex`.
///
/// Namespace key format matches `KeyringBackend`: `"{org_id_uuid}/{sensor}/{name}"`.
/// Use `namespace_key_by_org_id` to compute keys for assertion.
pub struct InMemoryCredentialStore {
    /// Map from namespace key → secret value.
    store: Mutex<HashMap<String, String>>,
}

impl InMemoryCredentialStore {
    /// Create an empty in-memory store.
    pub fn new() -> Self {
        InMemoryCredentialStore {
            store: Mutex::new(HashMap::new()),
        }
    }

    /// Inspect the raw namespace keys present in the store (for assertion purposes).
    ///
    /// Returns a `Vec<String>` of all keys currently in the store.
    /// Used by tests to assert `set_by_org` wrote to the correct namespace.
    pub fn keys(&self) -> Vec<String> {
        self.store
            .lock()
            .expect("InMemoryCredentialStore lock poisoned")
            .keys()
            .cloned()
            .collect()
    }

    /// Check whether a specific namespace key is present in the store.
    ///
    /// Used by tests to assert `set_by_org` used the OrgId-keyed namespace
    /// (not the slug-keyed namespace — CRIT-2 assertion).
    pub fn contains_key(&self, key: &str) -> bool {
        self.store
            .lock()
            .expect("InMemoryCredentialStore lock poisoned")
            .contains_key(key)
    }
}

impl Default for InMemoryCredentialStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl CredentialStoreOrgId for InMemoryCredentialStore {
    async fn get_by_org(
        &self,
        org_id: &OrgId,
        sensor: &str,
        name: &CredentialName,
    ) -> Result<Option<SecretString>, PrismError> {
        let key = crate::namespace::namespace_key_by_org_id(org_id, sensor, name);
        let store = self
            .store
            .lock()
            .map_err(|e| PrismError::CredentialStoreError {
                backend: "in_memory".to_owned(),
                reason: format!("lock poisoned: {e}"),
            })?;
        Ok(store.get(&key).map(|v| SecretString::new(v.clone())))
    }

    async fn set_by_org(
        &self,
        org_id: &OrgId,
        sensor: &str,
        name: &CredentialName,
        value: SecretString,
    ) -> Result<(), PrismError> {
        let key = crate::namespace::namespace_key_by_org_id(org_id, sensor, name);
        let mut store = self
            .store
            .lock()
            .map_err(|e| PrismError::CredentialStoreError {
                backend: "in_memory".to_owned(),
                reason: format!("lock poisoned: {e}"),
            })?;
        store.insert(key, value.expose_secret().to_owned());
        Ok(())
    }

    async fn delete_by_org(
        &self,
        org_id: &OrgId,
        sensor: &str,
        name: &CredentialName,
    ) -> Result<bool, PrismError> {
        let key = crate::namespace::namespace_key_by_org_id(org_id, sensor, name);
        let mut store = self
            .store
            .lock()
            .map_err(|e| PrismError::CredentialStoreError {
                backend: "in_memory".to_owned(),
                reason: format!("lock poisoned: {e}"),
            })?;
        Ok(store.remove(&key).is_some())
    }

    async fn list_by_org(
        &self,
        org_id: &OrgId,
    ) -> Result<Vec<(String, CredentialName)>, PrismError> {
        let org_prefix = format!("{}/", org_id);
        let store = self
            .store
            .lock()
            .map_err(|e| PrismError::CredentialStoreError {
                backend: "in_memory".to_owned(),
                reason: format!("lock poisoned: {e}"),
            })?;
        let mut results = Vec::new();
        for key in store.keys() {
            if let Some(rest) = key.strip_prefix(&org_prefix) {
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

    async fn exists_by_org(
        &self,
        org_id: &OrgId,
        sensor: &str,
        name: &CredentialName,
    ) -> Result<bool, PrismError> {
        Ok(self.get_by_org(org_id, sensor, name).await?.is_some())
    }
}
