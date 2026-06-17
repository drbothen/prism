//! PrismContext — per-client server context holding health cache and other per-client state.
//!
//! Owned by `PrismServer`. Provides:
//! - Per-client sensor health cache (BC-2.08.006): stores the last `check_sensor_health`
//!   result keyed by `(client_id, sensor_id)` with a TTL of 5 minutes.
//!
//! Health cache is written by `check_sensor_health` and read by the
//! `prism://sensors/health` resource handler.

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::Instant,
};

use chrono::{DateTime, Utc};

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
        self.inner
            .lock()
            .expect("HealthCache mutex poisoned")
            .insert(key, entry);
    }

    /// Get the cached health result for a (client_id, sensor_id) pair.
    ///
    /// Returns `None` if the key is not present (no health check has been run).
    pub fn get(&self, client_id: &str, sensor_id: &str) -> Option<CachedHealthEntry> {
        let key = HealthCacheKey {
            client_id: client_id.to_owned(),
            sensor_id: sensor_id.to_owned(),
        };
        self.inner
            .lock()
            .expect("HealthCache mutex poisoned")
            .get(&key)
            .cloned()
    }

    /// Get all cached entries for a given client_id, sorted by sensor_id.
    ///
    /// Returns an empty Vec if no health check has been run for the client.
    pub fn get_all_for_client(&self, client_id: &str) -> Vec<CachedHealthEntry> {
        let guard = self.inner.lock().expect("HealthCache mutex poisoned");
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
        self.inner
            .lock()
            .expect("HealthCache mutex poisoned")
            .keys()
            .any(|k| k.client_id == client_id)
    }

    /// Returns true if the cache is empty (no health checks run for any client).
    pub fn is_empty(&self) -> bool {
        self.inner
            .lock()
            .expect("HealthCache mutex poisoned")
            .is_empty()
    }

    /// Get all entries across all clients, sorted by (client_id, sensor_id).
    pub fn get_all(&self) -> Vec<CachedHealthEntry> {
        let guard = self.inner.lock().expect("HealthCache mutex poisoned");
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

/// PrismContext — server-level context shared across tool handlers.
///
/// Holds mutable per-server state that must persist across tool invocations:
/// - Health cache for `prism://sensors/health` resource (BC-2.08.006)
///
/// Wired as `Arc<PrismContext>` on `PrismServer`; individual fields use interior
/// mutability (Arc<Mutex<>>) for safe concurrent access.
#[derive(Debug, Clone)]
pub struct PrismContext {
    /// Per-client sensor health cache — written by `check_sensor_health`,
    /// read by `prism://sensors/health` resource handler (BC-2.08.006).
    pub health_cache: HealthCache,
}

impl PrismContext {
    /// Create a new PrismContext with an empty health cache.
    pub fn new() -> Self {
        Self {
            health_cache: HealthCache::new(),
        }
    }
}

impl Default for PrismContext {
    fn default() -> Self {
        Self::new()
    }
}
