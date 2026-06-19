//! No-op `CacheBackend` implementation for use when no real backing store is available.
//!
//! Used by `QueryEngine::with_infusion_registry` to satisfy the `InfusionTier3Cache`
//! constructor (which requires a `CacheBackend`) when no RocksDB backend has been wired
//! yet. Production boot path calls `with_infusion_caches` to replace this with the real
//! `RocksDbBackend` instance (S-1.14-REDO HIGH-1 / AC-7).
//!
//! All operations are no-ops: `get` always returns `None`, `set` is a silent discard,
//! `delete` is a silent discard. This is intentionally conservative: data correctness is
//! preserved (no stale data returned) at the cost of no Tier 3 persistence until the real
//! backend is wired.

use std::fmt;

use prism_core::{storage::StorageDomain, CacheBackend, PrismError};

/// A `CacheBackend` that never stores or returns anything.
///
/// `get` always returns `Ok(None)` — every lookup is a miss.
/// `set` silently discards — writes are no-ops.
/// `delete` silently discards — deletes are no-ops.
///
/// Used as a placeholder Tier 3 backend until the real `RocksDbBackend` is wired in.
#[derive(Debug)]
pub(crate) struct NullCacheBackend;

impl fmt::Display for NullCacheBackend {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "NullCacheBackend")
    }
}

#[async_trait::async_trait]
impl CacheBackend for NullCacheBackend {
    async fn get(
        &self,
        _domain: StorageDomain,
        _key: &[u8],
    ) -> Result<Option<Vec<u8>>, PrismError> {
        Ok(None)
    }

    async fn set(
        &self,
        _domain: StorageDomain,
        _key: &[u8],
        _value: &[u8],
    ) -> Result<(), PrismError> {
        Ok(())
    }

    async fn delete(&self, _domain: StorageDomain, _key: &[u8]) -> Result<(), PrismError> {
        Ok(())
    }
}
