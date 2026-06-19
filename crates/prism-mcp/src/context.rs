//! PrismContext — per-client server context holding health cache and other per-client state.
//!
//! Owned by `PrismServer`. Provides:
//! - Per-client sensor health cache (BC-2.08.006): stores the last `check_sensor_health`
//!   result keyed by `(client_id, sensor_id)` with a TTL of 5 minutes.
//! - Last-successful-query timestamp map (BC-2.08.004 — S-5.04): in-memory cache of
//!   the most recent successful query time per (client_id, sensor_id).
//! - Rate-limit state map (BC-2.08.003 — S-5.04): tracks HTTP 429 per
//!   (client_id, sensor_id) with auto-expiry based on Retry-After.
//!
//! Health cache is written by `check_sensor_health` and read by the
//! `prism://sensors/health` resource handler.

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::Instant,
};

use chrono::{DateTime, Utc};
use prism_storage::backend::RocksStorageBackend;

use crate::health::rate_limit::RateLimitState;
use crate::resources::SensorHealthResult;

/// TTL for health cache entries: 5 minutes (BC-2.08.006 EC-003).
pub const HEALTH_CACHE_TTL_SECS: u64 = 300;

/// Composite key for health cache: (client_id, sensor_id).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct HealthCacheKey {
    pub client_id: String,
    pub sensor_id: String,
}

/// A cached health entry with the time it was written.
#[derive(Debug, Clone)]
pub struct CachedHealthEntry {
    pub result: SensorHealthResult,
    pub cached_at: Instant,
    pub cached_at_utc: DateTime<Utc>,
}

impl CachedHealthEntry {
    /// Returns true if the entry is older than HEALTH_CACHE_TTL_SECS.
    pub fn is_stale(&self) -> bool {
        self.cached_at.elapsed().as_secs() >= HEALTH_CACHE_TTL_SECS
    }
}

/// Per-client sensor health cache (BC-2.08.006).
///
/// Keyed by (client_id, sensor_id). Thread-safe via Arc<Mutex<>>.
#[derive(Debug, Clone, Default)]
pub struct HealthCache {
    inner: Arc<Mutex<HashMap<HealthCacheKey, CachedHealthEntry>>>,
}

impl HealthCache {
    /// Create a new empty health cache.
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert or update a cached health result for a (client_id, sensor_id) pair.
    pub fn insert(&self, client_id: String, sensor_id: String, result: SensorHealthResult) {
        let key = HealthCacheKey {
            client_id,
            sensor_id,
        };
        let entry = CachedHealthEntry {
            result,
            cached_at: Instant::now(),
            cached_at_utc: Utc::now(),
        };
        // F-006: poison-tolerant lock — recover via into_inner on PoisonError
        // (CLAUDE.md §Conventions: expect() on Result forbidden in production paths)
        let mut guard = match self.inner.lock() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };
        guard.insert(key, entry);
    }

    /// Get the cached health result for a (client_id, sensor_id) pair.
    ///
    /// Returns `None` if the key is not present (no health check has been run).
    pub fn get(&self, client_id: &str, sensor_id: &str) -> Option<CachedHealthEntry> {
        let key = HealthCacheKey {
            client_id: client_id.to_owned(),
            sensor_id: sensor_id.to_owned(),
        };
        // F-006: poison-tolerant lock
        let guard = match self.inner.lock() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };
        guard.get(&key).cloned()
    }

    /// Get all cached entries for a given client_id, sorted by sensor_id.
    ///
    /// Returns an empty Vec if no health check has been run for the client.
    pub fn get_all_for_client(&self, client_id: &str) -> Vec<CachedHealthEntry> {
        // F-006: poison-tolerant lock
        let guard = match self.inner.lock() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };
        let mut entries: Vec<CachedHealthEntry> = guard
            .iter()
            .filter(|(k, _)| k.client_id == client_id)
            .map(|(_, v)| v.clone())
            .collect();
        entries.sort_by(|a, b| a.result.sensor_id.cmp(&b.result.sensor_id));
        entries
    }

    /// Returns true if there are any entries (stale or fresh) for the given client.
    pub fn has_any_for_client(&self, client_id: &str) -> bool {
        // F-006: poison-tolerant lock
        let guard = match self.inner.lock() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };
        guard.keys().any(|k| k.client_id == client_id)
    }

    /// Returns true if the cache is empty (no health checks run for any client).
    pub fn is_empty(&self) -> bool {
        // F-006: poison-tolerant lock
        let guard = match self.inner.lock() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };
        guard.is_empty()
    }

    /// Get all entries across all clients, sorted by (client_id, sensor_id).
    pub fn get_all(&self) -> Vec<CachedHealthEntry> {
        // F-006: poison-tolerant lock
        let guard = match self.inner.lock() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };
        let mut entries: Vec<CachedHealthEntry> = guard.values().cloned().collect();
        entries.sort_by(|a, b| {
            a.result
                .client_id
                .cmp(&b.result.client_id)
                .then(a.result.sensor_id.cmp(&b.result.sensor_id))
        });
        entries
    }
}

/// Composite key for timestamp / rate-limit maps: (client_id, sensor_id).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SensorKey {
    pub client_id: String,
    pub sensor_id: String,
}

/// Newtype wrapper that provides a no-op `Debug` impl for `Arc<dyn RocksStorageBackend>`.
///
/// `RocksStorageBackend` trait objects don't require `Debug`, but `PrismContext` derives
/// `Debug`. This wrapper decouples the two — the backend is only logged as `"<storage>"`.
#[derive(Clone)]
pub struct StorageHolder(pub Arc<dyn RocksStorageBackend>);

impl std::fmt::Debug for StorageHolder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("<storage>")
    }
}

impl std::ops::Deref for StorageHolder {
    type Target = dyn RocksStorageBackend;
    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

/// PrismContext — server-level context shared across tool handlers.
///
/// Holds mutable per-server state that must persist across tool invocations:
/// - Health cache for `prism://sensors/health` resource (BC-2.08.006)
/// - Last-successful-query timestamp map (BC-2.08.004 — S-5.04)
/// - Rate-limit state map (BC-2.08.003 — S-5.04)
/// - Optional RocksDB backend for timestamp durability (BC-2.08.004 postcondition 2 — S-5.04)
///
/// Wired as `Arc<PrismContext>` on `PrismServer`; individual fields use interior
/// mutability (Arc<Mutex<>>) for safe concurrent access.
#[derive(Debug, Clone)]
pub struct PrismContext {
    /// Per-client sensor health cache — written by `check_sensor_health`,
    /// read by `prism://sensors/health` resource handler (BC-2.08.006).
    pub health_cache: HealthCache,

    /// Last-successful-query timestamps per (client_id, sensor_id) (BC-2.08.004 — S-5.04).
    ///
    /// Written by `SensorHealthChecker::record_successful_query` on every successful
    /// sensor fetch or health probe.  Read by `check_sensor_health` to populate
    /// `SensorHealthResult.last_successful_query_at`.
    ///
    /// In-memory write-through cache; RocksDB persistence is handled by
    /// `health::timestamp::write_timestamp` when `storage` is Some.
    pub last_query_timestamps: Arc<Mutex<HashMap<SensorKey, DateTime<Utc>>>>,

    /// Rate-limit state per (client_id, sensor_id) (BC-2.08.003 — S-5.04).
    ///
    /// Written by `SensorHealthChecker` when a 429 response is observed.
    /// Auto-expires when `RateLimitState::is_cleared()` returns `true`.
    pub rate_limit_states: Arc<Mutex<HashMap<SensorKey, RateLimitState>>>,

    /// Optional RocksDB storage backend for durable timestamp persistence
    /// (BC-2.08.004 postcondition 2 — F-S504-P1-005).
    ///
    /// When `Some`, `write_timestamp` persists the RFC-3339 value to `StorageDomain::Default`
    /// under `health_ts/{client_id}/{sensor_id}` and `read_timestamp` falls back to
    /// the storage on cache miss. When `None` (test construction), persistence is in-memory only.
    ///
    /// Wrapped in `StorageHolder` to provide a no-op `Debug` impl — `dyn RocksStorageBackend`
    /// does not require `Debug` and we cannot derive it on the trait object directly.
    pub storage: Option<StorageHolder>,
}

impl PrismContext {
    /// Create a new PrismContext with empty health cache, timestamp map, and rate-limit map.
    ///
    /// For production use, call `new_with_storage()` to enable durable timestamp persistence.
    pub fn new() -> Self {
        Self {
            health_cache: HealthCache::new(),
            last_query_timestamps: Arc::new(Mutex::new(HashMap::new())),
            rate_limit_states: Arc::new(Mutex::new(HashMap::new())),
            storage: None,
        }
    }

    /// Create a new PrismContext wired with a RocksDB backend for durable timestamp persistence
    /// (BC-2.08.004 postcondition 2 — F-S504-P1-005: AC-5 "survives restart").
    pub fn new_with_storage(storage: Arc<dyn RocksStorageBackend>) -> Self {
        Self {
            health_cache: HealthCache::new(),
            last_query_timestamps: Arc::new(Mutex::new(HashMap::new())),
            rate_limit_states: Arc::new(Mutex::new(HashMap::new())),
            storage: Some(StorageHolder(storage)),
        }
    }
}

impl Default for PrismContext {
    fn default() -> Self {
        Self::new()
    }
}
