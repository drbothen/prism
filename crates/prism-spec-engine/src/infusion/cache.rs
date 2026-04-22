//! Three-tier infusion caching (BC-2.19.002 / INV-INFUSE-002).
//!
//! - Tier 1 — Per-query dedup: `QueryScopedInfusionCache` — HashMap scoped to a
//!   single query execution, dropped at query end. Ensures unique-value lookups only.
//! - Tier 2 — In-memory LRU: cross-query shared LRU with configurable capacity and TTL.
//! - Tier 3 — Persistent: RocksDB `infusion_cache` CF via `CacheBackend` trait injection.
//!
//! Lookup order: Tier 1 → Tier 2 → Tier 3 → call InfusionSource → populate all tiers.
//!
//! # Key design (VP-049)
//! Per-query dedup MUST be allocated fresh for each `QueryEngine::execute()` call.
//! Cross-query sharing of dedup state is PROHIBITED.
//!
//! # Stub
//! All methods are `unimplemented!()` — implementation in S-1.14.

use std::collections::HashMap;

use serde_json::Value;

/// Tier 1 per-query dedup cache.
///
/// Allocated at query start, dropped at query end.
/// Key format: `"{infusion_id}:{input_value}"`.
/// Ensures 10K events with 200 unique IPs = 200 source calls, not 10K (INV-INFUSE-002).
///
/// # IMPORTANT
/// This structure MUST NOT be shared across queries. Allocate fresh per query.
#[derive(Debug, Default)]
pub struct QueryScopedInfusionCache {
    inner: HashMap<String, Option<Value>>,
}

impl QueryScopedInfusionCache {
    /// Create a new per-query dedup cache.
    pub fn new() -> Self {
        QueryScopedInfusionCache {
            inner: HashMap::new(),
        }
    }

    /// Look up a cached enrichment result.
    ///
    /// Key format: `"{infusion_id}:{input_value}"`.
    /// Returns `Some(Some(value))` on hit with value, `Some(None)` on hit with NULL,
    /// `None` on cache miss.
    pub fn get(&self, infusion_id: &str, input_value: &str) -> Option<&Option<Value>> {
        let key = format!("{}:{}", infusion_id, input_value);
        self.inner.get(&key)
    }

    /// Insert an enrichment result into the dedup cache.
    ///
    /// Key format: `"{infusion_id}:{input_value}"`.
    pub fn insert(&mut self, infusion_id: &str, input_value: &str, result: Option<Value>) {
        let key = format!("{}:{}", infusion_id, input_value);
        self.inner.insert(key, result);
    }

    /// Return the number of entries (= number of unique input values processed).
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Returns `true` if the cache is empty.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

/// Tier 2 in-memory LRU cache entry with TTL.
#[derive(Debug, Clone)]
pub struct LruCacheEntry {
    pub value: Value,
    pub expiry_unix_secs: u64,
}

/// Tier 2 in-memory LRU cache (shared across queries).
///
/// Key format: `"{infusion_id}:{input_value}"`.
/// Default capacity: 10,000 entries. Per-infusion TTL (default 3600s).
/// Guarded by `tokio::sync::Mutex`.
pub struct InfusionLruCache {
    _inner: tokio::sync::Mutex<lru::LruCache<String, LruCacheEntry>>,
    capacity: usize,
}

impl std::fmt::Debug for InfusionLruCache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InfusionLruCache")
            .field("capacity", &self.capacity)
            .finish()
    }
}

impl InfusionLruCache {
    /// Create a new in-memory LRU cache with the given capacity.
    pub fn new(capacity: usize) -> Self {
        InfusionLruCache {
            _inner: tokio::sync::Mutex::new(lru::LruCache::new(
                std::num::NonZeroUsize::new(capacity).expect("capacity must be > 0"),
            )),
            capacity,
        }
    }

    /// Look up a cached entry. Returns `None` on miss or TTL expiry (lazy eviction).
    pub async fn get(&self, _infusion_id: &str, _input_value: &str) -> Option<Value> {
        unimplemented!("InfusionLruCache::get — implement in S-1.14 (BC-2.19.002)")
    }

    /// Insert an entry with the given TTL.
    pub async fn insert(
        &self,
        _infusion_id: &str,
        _input_value: &str,
        _value: Value,
        _ttl_secs: u64,
    ) {
        unimplemented!("InfusionLruCache::insert — implement in S-1.14 (BC-2.19.002)")
    }
}
