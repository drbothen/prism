//! In-memory `CredentialStoreOrgId` trait double for unit/integration tests.
//!
//! Provides a `HashMap<String, SecretString>`-backed implementation of the
//! `CredentialStoreOrgId` trait that does NOT touch the OS keyring. Used by:
//!
//! - `bc_2_06_003_tier3_keyring_resolution.rs` (RG-034-001, RG-034-002, RG-034-005):
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
//!
//! # Error injection (RG-034-005 — AC-011 Case B)
//!
//! `with_get_error(err)` constructs a store that returns `Err(err)` from
//! `get_by_org` on every call. This exercises the Tier-3 backend-error branch
//! (`Err(e)` → `BackendUnavailable`) without requiring a live keyring that can
//! be made to fail. AD-017 compliance: the injected error is a system error
//! value (`PrismError::CredentialStoreError`) and NEVER contains a credential
//! value — safe to include in error detail.

use std::{collections::HashMap, sync::Mutex};

use async_trait::async_trait;
use prism_core::{OrgId, PrismError};
use secrecy::{ExposeSecret, SecretString};

use crate::{namespace::CredentialName, trait_::CredentialStoreOrgId};

/// In-memory credential store for testing. Thread-safe via `Mutex`.
///
/// Namespace key format matches `KeyringBackend`: `"{org_id_uuid}/{sensor}/{name}"`.
/// Use `namespace_key_by_org_id` to compute keys for assertion.
///
/// When constructed via [`InMemoryCredentialStore::with_get_error`], every call to
/// `get_by_org` returns `Err(PrismError::CredentialStoreError { .. })` — allowing
/// tests to drive the Tier-3 backend-error branch (RG-034-005, AC-011 Case B).
pub struct InMemoryCredentialStore {
    /// Map from namespace key → secret value.
    store: Mutex<HashMap<String, String>>,
    /// Optional error fields returned by `get_by_org` on every call.
    ///
    /// `None` → normal operation (returns `Ok(Some(secret))` or `Ok(None)`).
    /// `Some((backend, reason))` → always returns
    ///   `Err(PrismError::CredentialStoreError { backend, reason })`.
    ///
    /// Stored as `(String, String)` because `PrismError` does not implement `Clone`;
    /// the error is reconstructed on each `get_by_org` call from the stored fields.
    ///
    /// AD-017: the `reason` string MUST be a system-level message, NEVER a credential value.
    error_mode: Option<(String, String)>,
}

impl InMemoryCredentialStore {
    /// Create an empty in-memory store.
    ///
    /// `get_by_org` returns `Ok(Some(secret))` for entries written via `set_by_org`,
    /// or `Ok(None)` for entries that were never written (Tier-3 miss path).
    pub fn new() -> Self {
        InMemoryCredentialStore {
            store: Mutex::new(HashMap::new()),
            error_mode: None,
        }
    }

    /// Create an in-memory store that always returns a `PrismError::CredentialStoreError`
    /// from `get_by_org`.
    ///
    /// Used by RG-034-005 (AC-011 Case B) to exercise the Tier-3 backend-error path:
    /// `keyring.get_by_org(...)` returns `Err(e)` → `CredentialResolutionError::BackendUnavailable`.
    ///
    /// # Parameters
    ///
    /// - `backend`: the backend name included in the `PrismError::CredentialStoreError`.
    ///   E.g. `"mock_keyring"`. Surfaces in `BackendUnavailable.detail` as
    ///   `"E-CRED-005: OS keyring unavailable: backend={backend}: {reason}. ..."` —
    ///   the format built by `resolution.rs` via `inner_detail` (F-P6-OBS-003 fix,
    ///   single E-CRED-005 code, no embedded E-CRED-004 prefix).
    /// - `reason`: the reason string included in the `PrismError::CredentialStoreError`.
    ///
    /// # AD-017 compliance
    ///
    /// `reason` MUST be a system-level error message (e.g., `"access denied by OS"`).
    /// Tests MUST NOT pass a credential value as `reason` — it would appear in the
    /// `BackendUnavailable.detail` string.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let store = InMemoryCredentialStore::with_get_error(
    ///     "mock_keyring",
    ///     "simulated OS keyring failure for RG-034-005",
    /// );
    /// // get_by_org returns Err(PrismError::CredentialStoreError { backend: "mock_keyring", .. })
    /// // resolve_credential maps this to BackendUnavailable { detail: "E-CRED-005: OS keyring unavailable: ..." }
    /// ```
    ///
    /// RG-034-005 (AC-011 Case B of S-DEMO-003); SID-1 compliance.
    pub fn with_get_error(backend: impl Into<String>, reason: impl Into<String>) -> Self {
        InMemoryCredentialStore {
            store: Mutex::new(HashMap::new()),
            error_mode: Some((backend.into(), reason.into())),
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
        // Error injection: if constructed via `with_get_error`, reconstruct and return
        // a `PrismError::CredentialStoreError` on every call. This exercises the
        // Tier-3 backend-error branch of `resolve_credential` (RG-034-005, AC-011 Case B).
        // `PrismError` does not implement `Clone`; we store the fields and reconstruct.
        if let Some((ref backend, ref reason)) = self.error_mode {
            return Err(PrismError::CredentialStoreError {
                backend: backend.clone(),
                reason: reason.clone(),
            });
        }
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
