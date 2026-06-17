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
    // S-1.14-REDO: renamed from `_inner` to `inner` — implementer uses `.inner.lock().await`
    // in get/insert bodies. Field was `_inner` in the S-1.14 stub to suppress dead-code
    // warnings; removing the underscore prefix prepares it for real implementation.
    // `#[allow(dead_code)]` suppresses the warning until S-1.14-REDO implements get/insert.
    #[allow(dead_code)]
    inner: tokio::sync::Mutex<lru::LruCache<String, LruCacheEntry>>,
    capacity: usize,
}

// SAFETY: `InfusionLruCache` is `UnwindSafe` because:
// 1. `lru::LruCache<String, LruCacheEntry>` holds only `serde_json::Value` (plain data) and
//    `u64` — no invariants that can be broken by a panic.
// 2. `tokio::sync::Mutex` does NOT poison on panic (unlike `std::sync::Mutex`); it remains
//    locked until the owning task is dropped. No lock-poisoning state machine exists.
// 3. The `capacity: usize` field has no invariants at all.
//
// The `UnwindSafe` impl was an accidental property on develop (where tokio compiled without
// the `parking_lot` feature, causing `batch_semaphore::Semaphore` to use `std::sync::Mutex`
// which IS `UnwindSafe`). Adding `prism-credentials` (which pulls `tokio = { features = ["full"] }`,
// including `parking_lot`) flipped the feature union: `batch_semaphore::Semaphore` now uses
// `parking_lot::Mutex` (not `UnwindSafe` via auto-derivation), transitively removing
// `InfusionLruCache`'s auto-impl. This explicit impl restores the guarantee intentionally,
// with the above safety argument, rather than relying on transitive feature-union accidents.
//
// Story: PLUGIN-MIGRATION-001-E / semver-checks CI fix
// Tracks: cargo-semver-checks `auto_trait_impl_removed` (prism-spec-engine 0.9.0)
impl std::panic::UnwindSafe for InfusionLruCache {}
impl std::panic::RefUnwindSafe for InfusionLruCache {}

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
            inner: tokio::sync::Mutex::new(lru::LruCache::new(
                std::num::NonZeroUsize::new(capacity).expect("capacity must be > 0"),
            )),
            capacity,
        }
    }

    /// Look up a cached entry by composite key `"{infusion_id}:{input_value}"`.
    ///
    /// Returns `None` on cache miss or on TTL expiry (lazy eviction: pop the stale key).
    /// On hit: return `Some(value)` if `entry.expiry_unix_secs > now`.
    ///
    /// Implementer: use `self.inner.lock().await` + lru 0.17 `.get(&key)` API.
    /// Key format must be consistent with `QueryScopedInfusionCache` (S-1.14-REDO Task 6).
    pub async fn get(&self, _infusion_id: &str, _input_value: &str) -> Option<Value> {
        todo!(
            "S-1.14-REDO: implement InfusionLruCache::get — composite key lookup with lazy TTL \
             eviction using lru 0.17 API — BC-2.19.002 / AC-7"
        )
    }

    /// Insert an entry with the given TTL (seconds).
    ///
    /// Key format: `"{infusion_id}:{input_value}"`.
    /// `expiry_unix_secs = now + ttl_secs`.
    /// Implementer: use `self.inner.lock().await` + lru 0.17 `.put(key, LruCacheEntry { value, expiry_unix_secs })`.
    pub async fn insert(
        &self,
        _infusion_id: &str,
        _input_value: &str,
        _value: Value,
        _ttl_secs: u64,
    ) {
        todo!(
            "S-1.14-REDO: implement InfusionLruCache::insert — TTL-bounded put with lru 0.17 API \
             — BC-2.19.002 / AC-7"
        )
    }
}
