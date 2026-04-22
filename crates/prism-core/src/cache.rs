//! CacheBackend trait — a subset of StorageBackend for get/set/delete only.
//!
//! This trait exists so prism-spec-engine can depend on prism-core (trait)
//! instead of prism-storage (implementation). Injected at startup by prism-bin.
//! `RocksDbBackend` (S-2.01) implements both `StorageBackend` and `CacheBackend`.

use async_trait::async_trait;

use crate::{error::PrismError, storage::StorageDomain};

/// Minimal key-value cache interface — get, set, delete over a `StorageDomain`.
///
/// Implementors MUST be `Send + Sync + 'static`.
#[async_trait]
pub trait CacheBackend: Send + Sync + 'static {
    /// Retrieve the value for `key` in `domain`, or `None` if absent.
    async fn get(&self, domain: StorageDomain, key: &[u8]) -> Result<Option<Vec<u8>>, PrismError>;

    /// Store `value` at `key` in `domain`, overwriting any existing value.
    async fn set(&self, domain: StorageDomain, key: &[u8], value: &[u8]) -> Result<(), PrismError>;

    /// Remove the value at `key` in `domain`. No-op if absent.
    async fn delete(&self, domain: StorageDomain, key: &[u8]) -> Result<(), PrismError>;
}
