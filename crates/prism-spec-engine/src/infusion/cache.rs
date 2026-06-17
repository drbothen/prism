//! Three-tier infusion caching (BC-2.19.002 / INV-INFUSE-002).
//!
//! - Tier 1 — Per-query dedup: `QueryScopedInfusionCache` — HashMap scoped to a
//!   single query execution, dropped at query end. Ensures unique-value lookups only.
//! - Tier 2 — In-memory LRU: cross-query shared LRU with configurable capacity and TTL.
//! - Tier 3 — Persistent: RocksDB `infusion_cache` CF via `CacheBackend` trait injection.
//!   Key: SHA-256(`"{infusion_id}:{input_value}"`). Value: bincode 2.x encoded
//!   `(Option<Value>, expiry_unix_secs: u64)`.  Lazy TTL eviction on read.
//!
//! Lookup order: Tier 1 → Tier 2 → Tier 3 → call InfusionSource → populate all tiers.
//!
//! # Key design (VP-049)
//! Per-query dedup MUST be allocated fresh for each `QueryEngine::execute()` call.
//! Cross-query sharing of dedup state is PROHIBITED.

use std::collections::HashMap;
use std::sync::Arc;

use prism_core::{CacheBackend, storage::StorageDomain};
use serde::{Deserialize, Serialize};
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
    /// Key format is consistent with `QueryScopedInfusionCache` (BC-2.19.002).
    pub async fn get(&self, infusion_id: &str, input_value: &str) -> Option<Value> {
        let key = format!("{}:{}", infusion_id, input_value);
        let mut cache = self.inner.lock().await;
        if let Some(entry) = cache.get(&key) {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            if entry.expiry_unix_secs > now {
                return Some(entry.value.clone());
            }
            // Expired — lazy eviction.
            cache.pop(&key);
        }
        None
    }

    /// Insert an entry with the given TTL (seconds).
    ///
    /// Key format: `"{infusion_id}:{input_value}"`.
    /// `expiry_unix_secs = now + ttl_secs`.
    pub async fn insert(&self, infusion_id: &str, input_value: &str, value: Value, ttl_secs: u64) {
        let key = format!("{}:{}", infusion_id, input_value);
        let expiry_unix_secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
            .saturating_add(ttl_secs);
        self.inner.lock().await.put(
            key,
            LruCacheEntry {
                value,
                expiry_unix_secs,
            },
        );
    }
}

// ---------------------------------------------------------------------------
// Tier 3 — Persistent RocksDB cache via CacheBackend trait injection
// ---------------------------------------------------------------------------

/// Wire format for a Tier-3 cache entry: value + expiry_unix_secs.
///
/// Serialized with bincode 2.x (`encode_to_vec` / `decode_from_slice`).
/// `value` is the JSON-serialized enrichment result (None = negative cache entry).
///
/// `#[non_exhaustive]`: forward-compat per CLAUDE.md §Conventions — pub types in
/// prism-spec-engine require `#[non_exhaustive]` (MED-1-RESIDUAL, S-1.14-REDO burst 2).
#[non_exhaustive]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Tier3CacheEntry {
    /// JSON-serialized enrichment result. `None` = negative cache hit (no enrichment).
    pub value_json: Option<String>,
    /// Unix timestamp (seconds) at which this entry expires.
    pub expiry_unix_secs: u64,
}

/// Tier 3 persistent infusion cache backed by `CacheBackend` (RocksDB `infusion_cache` CF).
///
/// Key: SHA-256(`"{infusion_id}:{input_value}"`) as raw bytes.
/// Value: bincode 2.x encoded `Tier3CacheEntry`.
/// TTL enforced lazily on read: expired entries are treated as misses (not deleted eagerly).
///
/// Injected at boot by `prism-bin` which wires the `RocksDbBackend` implementation.
/// Tests may inject an in-memory `CacheBackend` implementation.
#[derive(Clone)]
pub struct InfusionTier3Cache {
    backend: Arc<dyn CacheBackend>,
}

impl std::fmt::Debug for InfusionTier3Cache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InfusionTier3Cache").finish_non_exhaustive()
    }
}

impl InfusionTier3Cache {
    /// Construct a new Tier-3 cache with the given `CacheBackend`.
    pub fn new(backend: Arc<dyn CacheBackend>) -> Self {
        Self { backend }
    }

    /// Compute the SHA-256 key for `"{infusion_id}:{input_value}"`.
    fn cache_key(infusion_id: &str, input_value: &str) -> Vec<u8> {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(infusion_id.as_bytes());
        hasher.update(b":");
        hasher.update(input_value.as_bytes());
        hasher.finalize().to_vec()
    }

    /// Look up an enrichment result in the RocksDB cache.
    ///
    /// Returns `Some(Some(value))` on hit with value, `Some(None)` on hit with NULL
    /// (negative cache), `None` on cache miss or TTL expiry (lazy eviction: treat as miss).
    pub async fn get(&self, infusion_id: &str, input_value: &str) -> Option<Option<Value>> {
        let key = Self::cache_key(infusion_id, input_value);
        let raw = match self.backend.get(StorageDomain::InfusionCache, &key).await {
            Ok(Some(data)) => data,
            Ok(None) => return None,
            Err(_) => return None, // Backend error → treat as miss (non-blocking)
        };

        // Decode the bincode 2.x wire format.
        let entry: Tier3CacheEntry =
            match bincode::serde::decode_from_slice(&raw, bincode::config::standard()) {
                Ok((e, _)) => e,
                Err(_) => return None, // Corrupt entry → treat as miss
            };

        // Lazy TTL eviction: check expiry.
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        if entry.expiry_unix_secs <= now {
            // Entry expired — treat as miss (we do NOT eagerly delete here per spec: lazy eviction).
            return None;
        }

        // Deserialize the JSON value.
        match entry.value_json {
            None => Some(None), // Negative cache hit
            Some(ref json_str) => match serde_json::from_str(json_str) {
                Ok(v) => Some(Some(v)),
                Err(_) => None, // Corrupt JSON → treat as miss
            },
        }
    }

    /// Insert an enrichment result into the RocksDB cache with TTL.
    ///
    /// `value = None` stores a negative cache entry (enrichment miss — prevents re-lookup).
    pub async fn set(
        &self,
        infusion_id: &str,
        input_value: &str,
        value: Option<Value>,
        ttl_secs: u64,
    ) {
        let key = Self::cache_key(infusion_id, input_value);
        let expiry_unix_secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
            .saturating_add(ttl_secs);

        let value_json = value.as_ref().map(|v| v.to_string());
        let entry = Tier3CacheEntry {
            value_json,
            expiry_unix_secs,
        };

        let encoded = match bincode::serde::encode_to_vec(&entry, bincode::config::standard()) {
            Ok(b) => b,
            Err(_) => return, // Encoding failure → skip cache set (non-blocking)
        };

        // Ignore write errors — cache write failures are non-blocking.
        let _ = self
            .backend
            .set(StorageDomain::InfusionCache, &key, &encoded)
            .await;
    }
}
