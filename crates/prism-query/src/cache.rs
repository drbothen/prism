//! `cache` — Sensor-fetch response cache with configurable TTL and LRU eviction.
//!
//! Implements BC-2.07.003 and BC-2.07.006.
//!
//! # Cache model
//! A single in-memory sensor-fetch cache keyed by [`crate::cache_key::CacheKey`]
//! (4-tuple: `client_id`, `sensor_id`, `source_id`, `push_down_hash`). Cache
//! entries store the raw sensor API response pre-OCSF normalization. OCSF
//! normalization and PrismQL post-filters are applied after cache retrieval
//! (BC-2.07.003 §Postconditions).
//!
//! # TTL semantics
//! TTL is measured from `created_at` (absolute expiry), not from last access
//! (BC-2.07.003 — "TTL, not sliding expiration"). Default TTLs by data type:
//! - Alerts / detections: 60 seconds
//! - Devices / hosts / assets: 300 seconds
//! - Health / status: not cached (BC-2.07.003)
//!
//! # Memory bounds
//! Each `(client_id, sensor_id)` partition is independently bounded at 50 entries
//! (configurable via TOML). When insertion would exceed the bound, the LRU entry
//! is evicted first (BC-2.07.006 §Postconditions — DI-018). Eviction is
//! synchronous with the insert operation.
//!
//! # Concurrency
//! `QueryCache` is `Send + Sync` and designed to be shared via `Arc<QueryCache>`.
//! The `moka::sync::Cache` inner type is itself thread-safe.
//!
//! # BC References
//! - BC-2.07.003 — Query Engine Sensor-Fetch Cache with Configurable TTL
//! - BC-2.07.006 — Cache Memory Bounds and Eviction Policy
//!
//! Story: S-3.05

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use moka::sync::Cache as MokaCache;

use crate::cache_key::CacheKey;

// ---------------------------------------------------------------------------
// TTL constants (BC-2.07.003)
// ---------------------------------------------------------------------------

/// Default TTL for alert / detection source entries: 60 seconds.
/// High-churn data requiring freshness (BC-2.07.003).
pub const CACHE_TTL_ALERTS_SECS: u64 = 60;

/// Default TTL for device / host / asset source entries: 300 seconds (5 min).
/// Lower-churn inventory data (BC-2.07.003).
pub const CACHE_TTL_DEVICES_SECS: u64 = 300;

/// Default entry count bound per `(client_id, sensor_id)` partition (BC-2.07.006).
pub const DEFAULT_MAX_ENTRIES_PER_SENSOR: usize = 50;

// ---------------------------------------------------------------------------
// SourceDataType
// ---------------------------------------------------------------------------

/// Classification of sensor source data types for TTL selection.
///
/// Determined from the `source_id` during cache insertion.
/// `Health` / status sources are not cached (BC-2.07.003).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceDataType {
    /// Alerts, detections — TTL 60s.
    AlertsDetections,
    /// Devices, hosts, assets — TTL 300s.
    DevicesAssets,
    /// Health / status endpoints — not cached.
    HealthStatus,
}

impl SourceDataType {
    /// Return the configured TTL for this data type, or `None` if uncacheable.
    ///
    /// GREEN-BY-DESIGN: pure match on enum variants; zero branching beyond
    /// pattern, no I/O, no helpers, 4 lines.
    pub fn ttl(&self) -> Option<Duration> {
        match self {
            Self::AlertsDetections => Some(Duration::from_secs(CACHE_TTL_ALERTS_SECS)),
            Self::DevicesAssets => Some(Duration::from_secs(CACHE_TTL_DEVICES_SECS)),
            Self::HealthStatus => None,
        }
    }

    /// Classify a `source_id` string into a `SourceDataType`.
    ///
    /// Classification rules (BC-2.07.003):
    /// - Sources containing `"alert"` or `"detection"` → `AlertsDetections`
    /// - Sources containing `"health"` or `"status"` → `HealthStatus`
    /// - Everything else (devices, hosts, assets, etc.) → `DevicesAssets`
    pub fn from_source_id(source_id: &str) -> Self {
        let s = source_id.to_lowercase();
        if s.contains("health") || s.contains("status") {
            Self::HealthStatus
        } else if s.contains("alert") || s.contains("detection") {
            Self::AlertsDetections
        } else {
            Self::DevicesAssets
        }
    }
}

// ---------------------------------------------------------------------------
// CacheEntry
// ---------------------------------------------------------------------------

/// A single cached sensor-fetch response.
///
/// Stores the raw sensor API response rows (pre-OCSF normalization) along with
/// metadata for TTL enforcement and metrics (BC-2.07.003 §Postconditions).
#[derive(Debug, Clone)]
pub struct CacheEntry {
    /// Raw sensor API response rows stored as JSON (pre-OCSF normalization).
    pub rows: Vec<serde_json::Value>,
    /// Absolute creation timestamp — TTL is measured from this (BC-2.07.003).
    pub created_at: Instant,
    /// TTL duration for this entry (data-type dependent).
    pub ttl: Duration,
    /// Cache hit counter — incremented on each cache hit for metrics
    /// visibility via `check_sensor_health` (BC-2.07.003).
    pub hit_count: u64,
}

impl CacheEntry {
    /// Returns `true` if this entry's TTL has elapsed (BC-2.07.003).
    ///
    /// TTL is measured from `created_at` (absolute), not from last access.
    pub fn is_expired(&self) -> bool {
        self.created_at.elapsed() > self.ttl
    }
}

// ---------------------------------------------------------------------------
// CacheConfig
// ---------------------------------------------------------------------------

/// Configuration for the `QueryCache`.
///
/// Settable via `[defaults.cache]` in `prism.toml` (BC-2.07.006).
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Maximum number of entries per `(client_id, sensor_id)` partition.
    /// Default: 50 (BC-2.07.006 §Postconditions).
    pub max_entries_per_sensor: usize,
}

impl Default for CacheConfig {
    /// GREEN-BY-DESIGN: constructs `CacheConfig` with default constant.
    /// Zero branching, no I/O, no helpers, 1 line.
    fn default() -> Self {
        CacheConfig {
            max_entries_per_sensor: DEFAULT_MAX_ENTRIES_PER_SENSOR,
        }
    }
}

// ---------------------------------------------------------------------------
// Partition key for per-(client_id, sensor_id) tracking
// ---------------------------------------------------------------------------

/// Partition key for per-`(client_id, sensor_id)` entry counting.
type PartitionKey = (String, String);

fn partition_key(key: &CacheKey) -> PartitionKey {
    (key.client_id.clone(), key.sensor_id.clone())
}

// ---------------------------------------------------------------------------
// QueryCache
// ---------------------------------------------------------------------------

/// Thread-safe sensor-fetch response cache.
///
/// Implements BC-2.07.003 (TTL-based caching) and BC-2.07.006 (LRU eviction
/// with per-partition entry count bound). Intended to be held in a single
/// `Arc<QueryCache>` shared across all `QueryEngine` tasks.
///
/// Internally uses `moka::sync::Cache` for thread-safe LRU eviction and
/// TTL-based entry expiry (story §Caching Context Summary — moka 0.12).
pub struct QueryCache {
    config: CacheConfig,
    /// moka LRU cache: provides O(1) get/put with background TTL eviction.
    /// Large capacity; per-partition bounds enforced by `partition_counts`.
    inner: MokaCache<CacheKey, CacheEntry>,
    /// Per-`(client_id, sensor_id)` entry counts for DI-018 bound enforcement.
    /// Each element tracks how many entries exist in that partition.
    partition_counts: Mutex<HashMap<PartitionKey, Vec<CacheKey>>>,
}

impl QueryCache {
    /// Construct a new `QueryCache` with the given configuration.
    pub fn new(config: CacheConfig) -> Self {
        // moka capacity: large global pool; per-partition bounds via partition_counts.
        // We use a large moka capacity so it never evicts by itself —
        // per-partition eviction is handled manually in `put`.
        let moka_cap: u64 = 100_000;
        let inner = MokaCache::builder().max_capacity(moka_cap).build();
        QueryCache {
            config,
            inner,
            partition_counts: Mutex::new(HashMap::new()),
        }
    }

    /// Construct a `QueryCache` with default configuration.
    pub fn with_defaults() -> Self {
        Self::new(CacheConfig::default())
    }

    /// Look up a cache entry by key.
    ///
    /// Returns `Some(rows)` if the entry exists and is not expired.
    /// Returns `None` (cache miss) if the key is absent or the entry has
    /// exceeded its TTL. Expired entries are removed on miss (BC-2.07.003).
    ///
    /// On a cache hit, increments `hit_count` on the entry (BC-2.07.003).
    pub fn get(&self, key: &CacheKey) -> Option<Vec<serde_json::Value>> {
        let entry = self.inner.get(key)?;
        if entry.is_expired() {
            // Remove expired entry — treat as cache miss.
            self.remove_entry(key);
            return None;
        }
        // Increment hit_count: replace the entry with updated count.
        let mut updated = entry.clone();
        updated.hit_count += 1;
        self.inner.insert(key.clone(), updated.clone());

        // Update LRU position: move this key to the end of the partition Vec
        // (most-recently-used position) so eviction targets the front (LRU).
        let pk = partition_key(key);
        let mut counts = self
            .partition_counts
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        if let Some(partition_keys) = counts.get_mut(&pk) {
            // Move key to end (most-recently-used).
            partition_keys.retain(|k| k != key);
            partition_keys.push(key.clone());
        }

        Some(updated.rows)
    }

    /// Insert a new cache entry.
    ///
    /// Before insertion, checks the `(client_id, sensor_id)` partition count.
    /// If insertion would exceed `config.max_entries_per_sensor`, the LRU
    /// entry for that partition is evicted first (BC-2.07.006 — synchronous
    /// eviction before insert; loop until space is available).
    ///
    /// If `SourceDataType::from_source_id(key.source_id)` is `HealthStatus`,
    /// the put is a no-op (health endpoints are not cached — BC-2.07.003).
    pub fn put(&self, key: CacheKey, rows: Vec<serde_json::Value>) {
        let data_type = SourceDataType::from_source_id(&key.source_id);
        let ttl = match data_type.ttl() {
            Some(t) => t,
            None => return, // HealthStatus — not cached
        };
        self.put_with_ttl(key, rows, ttl);
    }

    /// Insert with explicit TTL override (for testing or admin bypass).
    pub fn put_with_ttl(&self, key: CacheKey, rows: Vec<serde_json::Value>, ttl: Duration) {
        // HealthStatus check: if the source is a health/status type, skip.
        let data_type = SourceDataType::from_source_id(&key.source_id);
        if data_type == SourceDataType::HealthStatus {
            return;
        }

        // max_entries_per_sensor == 0 → caching disabled.
        if self.config.max_entries_per_sensor == 0 {
            return;
        }

        let pk = partition_key(&key);

        // Enforce per-partition bound synchronously before insert (DI-018).
        let mut counts = self
            .partition_counts
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let partition_keys = counts.entry(pk.clone()).or_default();

        // Evict LRU entries until there is space for the new entry.
        // LRU = oldest by position; moka handles get-based recency internally.
        // For partition-level eviction we track insertion order in the Vec.
        while partition_keys.len() >= self.config.max_entries_per_sensor {
            // Remove the first entry in the Vec (oldest = LRU for FIFO tiebreaker).
            let evict_key = partition_keys.remove(0);
            self.inner.invalidate(&evict_key);
        }

        // Track the new key for this partition.
        // If the key already exists in the partition list (update/refresh), remove it first.
        partition_keys.retain(|k| k != &key);
        partition_keys.push(key.clone());

        drop(counts); // release lock before insert

        let entry = CacheEntry {
            rows,
            created_at: Instant::now(),
            ttl,
            hit_count: 0,
        };
        self.inner.insert(key, entry);
    }

    /// Bypass the cache and replace an existing entry with fresh data.
    ///
    /// Implements `force_refresh: true` semantics (BC-2.07.003 §Postconditions).
    /// The `push_down_hash` of `key` matches the non-forced version; the entry
    /// is overwritten.
    pub fn force_refresh(&self, key: CacheKey, rows: Vec<serde_json::Value>) {
        // Remove existing entry if present, then insert fresh.
        self.remove_entry(&key);
        self.put(key, rows);
    }

    /// Remove all entries whose key matches a `(client_id, sensor_id, source_id)`
    /// prefix (for invalidation by source).
    ///
    /// This is the low-level primitive used by [`crate::invalidation::CacheInvalidator`].
    pub fn invalidate_by_prefix(&self, client_id: &str, sensor_id: &str, source_id: &str) {
        let mut counts = self
            .partition_counts
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let pk = (client_id.to_string(), sensor_id.to_string());

        // Find and remove all keys matching the (client_id, sensor_id, source_id) prefix.
        if let Some(partition_keys) = counts.get_mut(&pk) {
            let to_evict: Vec<CacheKey> = partition_keys
                .iter()
                .filter(|k| k.source_id == source_id)
                .cloned()
                .collect();

            for k in &to_evict {
                self.inner.invalidate(k);
                partition_keys.retain(|pk| pk != k);
            }
        }
    }

    /// Remove all entries whose `client_id` matches `client_id`.
    ///
    /// Used for client management write operations (BC-2.07.004).
    pub fn invalidate_by_client(&self, client_id: &str) {
        let mut counts = self
            .partition_counts
            .lock()
            .unwrap_or_else(|e| e.into_inner());

        // Find all partitions for this client.
        let client_partitions: Vec<PartitionKey> = counts
            .keys()
            .filter(|(cid, _)| cid == client_id)
            .cloned()
            .collect();

        for pk in client_partitions {
            if let Some(partition_keys) = counts.remove(&pk) {
                for k in partition_keys {
                    self.inner.invalidate(&k);
                }
            }
        }
    }

    /// Returns the total number of entries currently in the cache (for metrics).
    pub fn entry_count(&self) -> u64 {
        // moka's entry_count may lag by a tick; sync first.
        self.inner.run_pending_tasks();
        self.inner.entry_count()
    }

    // Internal: remove a single entry from both moka and partition tracker.
    fn remove_entry(&self, key: &CacheKey) {
        self.inner.invalidate(key);
        let mut counts = self
            .partition_counts
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let pk = partition_key(key);
        if let Some(partition_keys) = counts.get_mut(&pk) {
            partition_keys.retain(|k| k != key);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// GREEN-BY-DESIGN: `SourceDataType::ttl()` for AlertsDetections returns 60s.
    /// Pure match, zero branching beyond pattern, no I/O, 1 line.
    #[test]
    fn test_alerts_detections_ttl_is_60s() {
        assert_eq!(
            SourceDataType::AlertsDetections.ttl(),
            Some(Duration::from_secs(60))
        );
    }

    /// GREEN-BY-DESIGN: `SourceDataType::ttl()` for DevicesAssets returns 300s.
    /// Pure match, zero branching beyond pattern, no I/O, 1 line.
    #[test]
    fn test_devices_assets_ttl_is_300s() {
        assert_eq!(
            SourceDataType::DevicesAssets.ttl(),
            Some(Duration::from_secs(300))
        );
    }

    /// GREEN-BY-DESIGN: `SourceDataType::ttl()` for HealthStatus returns None.
    /// Pure match, zero branching beyond pattern, no I/O, 1 line.
    #[test]
    fn test_health_status_ttl_is_none() {
        assert_eq!(SourceDataType::HealthStatus.ttl(), None);
    }

    /// GREEN-BY-DESIGN: `CacheConfig::default()` has max_entries_per_sensor == 50.
    /// Pure constructor check, zero branching, no I/O, 1 line.
    #[test]
    fn test_default_config_max_entries_is_50() {
        assert_eq!(CacheConfig::default().max_entries_per_sensor, 50);
    }

    /// AC-5 / BC-2.07.003: Cache hit on identical query within TTL window.
    #[test]
    fn test_ac5_cache_hit_within_ttl_returns_cached_rows() {
        let cache = QueryCache::with_defaults();
        let key = crate::cache_key::CacheKey {
            client_id: "acme".to_string(),
            sensor_id: "crowdstrike".to_string(),
            source_id: "crowdstrike_detections".to_string(),
            push_down_hash: "a".repeat(64),
        };
        let rows = vec![serde_json::json!({"id": "det-1"})];
        cache.put(key.clone(), rows.clone());
        let result = cache.get(&key);
        assert_eq!(
            result,
            Some(rows),
            "AC-5: cache hit must return the stored rows"
        );
    }

    /// BC-2.07.003: Cache miss on unseen key returns None.
    #[test]
    fn test_cache_miss_on_unseen_key_returns_none() {
        let cache = QueryCache::with_defaults();
        let key = crate::cache_key::CacheKey {
            client_id: "acme".to_string(),
            sensor_id: "crowdstrike".to_string(),
            source_id: "crowdstrike_detections".to_string(),
            push_down_hash: "b".repeat(64),
        };
        assert!(
            cache.get(&key).is_none(),
            "cache miss on unseen key must return None"
        );
    }

    /// AC-8 / BC-2.07.006: At capacity, inserting a new entry evicts LRU.
    #[test]
    fn test_ac8_lru_eviction_at_capacity() {
        let config = CacheConfig {
            max_entries_per_sensor: 2,
        };
        let cache = QueryCache::new(config);

        let make_key = |n: u8| crate::cache_key::CacheKey {
            client_id: "acme".to_string(),
            sensor_id: "armis".to_string(),
            source_id: "armis_devices".to_string(),
            push_down_hash: format!("{:0<64}", n),
        };
        // Fill to capacity.
        cache.put(make_key(1), vec![serde_json::json!({"id": 1})]);
        cache.put(make_key(2), vec![serde_json::json!({"id": 2})]);
        // Third insert must evict LRU — total stays at most 2.
        cache.put(make_key(3), vec![serde_json::json!({"id": 3})]);

        // Entry count must not exceed configured bound for this partition.
        assert!(
            cache.entry_count() <= 2,
            "AC-8: partition must not exceed max_entries_per_sensor after eviction"
        );
    }

    /// BC-2.07.003: `force_refresh` overwrites an existing cache entry.
    #[test]
    fn test_force_refresh_overwrites_existing_entry() {
        let cache = QueryCache::with_defaults();
        let key = crate::cache_key::CacheKey {
            client_id: "acme".to_string(),
            sensor_id: "crowdstrike".to_string(),
            source_id: "crowdstrike_hosts".to_string(),
            push_down_hash: "c".repeat(64),
        };
        let old_rows = vec![serde_json::json!({"host": "old"})];
        let new_rows = vec![serde_json::json!({"host": "new"})];

        cache.put(key.clone(), old_rows);
        cache.force_refresh(key.clone(), new_rows.clone());

        assert_eq!(
            cache.get(&key),
            Some(new_rows),
            "force_refresh must overwrite existing entry with fresh data"
        );
    }
}
