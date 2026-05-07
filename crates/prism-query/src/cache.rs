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
//! Total cache byte usage is bounded at 50 MB (BC-2.07.006). Per-entry size is
//! estimated at [`AVG_ROW_SIZE_BYTES`] bytes × row count. Entries exceeding
//! [`MAX_ENTRY_BYTES`] are rejected as a defense-in-depth measure (SEC-003).
//!
//! # Concurrency
//! `QueryCache` is `Send + Sync` and designed to be shared via `Arc<QueryCache>`.
//! The `moka::sync::Cache` inner type is itself thread-safe. If the `partition_counts`
//! mutex is poisoned (a thread panicked while holding the lock), all subsequent
//! cache operations return `Err(PrismError::Internal { detail: "E-CACHE-001: ..." })`
//! instead of silently continuing with potentially corrupted state (BC-2.07.004
//! E-CACHE-001, SEC-001).
//!
//! # BC References
//! - BC-2.07.003 — Query Engine Sensor-Fetch Cache with Configurable TTL
//! - BC-2.07.006 — Cache Memory Bounds and Eviction Policy
//!
//! Story: S-3.05

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Mutex, MutexGuard};
use std::time::{Duration, Instant};

use moka::sync::Cache as MokaCache;

use crate::cache_key::CacheKey;
use prism_core::error::PrismError;

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

/// Total cache byte budget — 50 MB hard limit (BC-2.07.006, CR-006).
pub const DEFAULT_MAX_CACHE_BYTES: usize = 50 * 1024 * 1024;

/// Estimated average row size in bytes for byte-budget accounting (CR-006).
/// Conservative estimate: 512 bytes per JSON row.
pub const AVG_ROW_SIZE_BYTES: usize = 512;

/// Per-entry byte cap — reject entries exceeding 5 MB as defense-in-depth (SEC-003).
pub const MAX_ENTRY_BYTES: usize = 5 * 1024 * 1024;

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
    /// Classification rules (BC-2.07.003, CR-007):
    /// - Sources with suffix `"_health"` or `"_status"` → `HealthStatus` (not cached)
    /// - Sources with suffix `"_alerts"`, `"_detections"`, or `"_alert"` / `"_detection"` →
    ///   `AlertsDetections`
    /// - Everything else (devices, hosts, assets, etc.) → `DevicesAssets`
    ///
    /// Suffix matching (`ends_with`) is used instead of substring (`contains`) to
    /// avoid false positives from source IDs like `"crowdstrike_health_incidents"`
    /// that contain `"health"` but are not health endpoints (CR-007).
    pub fn from_source_id(source_id: &str) -> Self {
        let s = source_id.to_lowercase();
        if s.ends_with("_health") || s.ends_with("_status") {
            Self::HealthStatus
        } else if s.ends_with("_alerts")
            || s.ends_with("_detections")
            || s.ends_with("_alert")
            || s.ends_with("_detection")
        {
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
/// metadata for TTL enforcement (BC-2.07.003 §Postconditions).
///
/// Hit metrics are tracked at the cache level via `QueryCache::total_hits()`
/// rather than per-entry to avoid cloning the full `rows` vec on every hit
/// (CR-005). Aggregate counts are sufficient for `check_sensor_health` visibility.
#[derive(Debug, Clone)]
pub struct CacheEntry {
    /// Raw sensor API response rows stored as JSON (pre-OCSF normalization).
    pub rows: Vec<serde_json::Value>,
    /// Absolute creation timestamp — TTL is measured from this (BC-2.07.003).
    pub created_at: Instant,
    /// TTL duration for this entry (data-type dependent).
    pub ttl: Duration,
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
    /// Total byte budget for the entire cache. Default: 50 MB (BC-2.07.006, CR-006).
    pub max_bytes: usize,
}

impl Default for CacheConfig {
    /// GREEN-BY-DESIGN: constructs `CacheConfig` with default constants.
    /// Zero branching, no I/O, no helpers, 1 line.
    fn default() -> Self {
        CacheConfig {
            max_entries_per_sensor: DEFAULT_MAX_ENTRIES_PER_SENSOR,
            max_bytes: DEFAULT_MAX_CACHE_BYTES,
        }
    }
}

// ---------------------------------------------------------------------------
// Partition key for per-(client_id, sensor_id) tracking
// ---------------------------------------------------------------------------

/// Partition key for per-`(client_id, sensor_id)` entry counting.
type PartitionKey = (String, String);

/// Per-partition entry list: each element is `(key, estimated_byte_size)`.
/// The byte size is stored so removal paths can decrement `total_bytes` by the
/// actual per-entry size rather than a fixed constant (CR-014/CR-015).
type PartitionVec = Vec<(CacheKey, usize)>;

fn partition_key(key: &CacheKey) -> PartitionKey {
    (key.client_id.clone(), key.sensor_id.clone())
}

// ---------------------------------------------------------------------------
// QueryCache
// ---------------------------------------------------------------------------

/// Thread-safe sensor-fetch response cache.
///
/// Implements BC-2.07.003 (TTL-based caching) and BC-2.07.006 (LRU eviction
/// with per-partition entry count bound and 50 MB total byte budget). Intended
/// to be held in a single `Arc<QueryCache>` shared across all `QueryEngine` tasks.
///
/// Internally uses `moka::sync::Cache` for thread-safe LRU eviction and
/// TTL-based entry expiry (story §Caching Context Summary — moka 0.12).
///
/// ## Mutex poison safety (SEC-001 / BC-2.07.004 E-CACHE-001)
/// If the `partition_counts` mutex is poisoned, all cache operations that
/// require the lock return `Err(PrismError::Internal { detail: "E-CACHE-001: ..." })`.
/// Silent recovery via `unwrap_or_else(|e| e.into_inner())` is prohibited.
pub struct QueryCache {
    config: CacheConfig,
    /// moka LRU cache: provides O(1) get/put with background TTL eviction.
    /// Large capacity; per-partition bounds enforced by `partition_counts`.
    inner: MokaCache<CacheKey, CacheEntry>,
    /// Per-`(client_id, sensor_id)` entry tracking for DI-018 bound enforcement.
    /// Each element is `(key, estimated_byte_size)` — the byte size is stored
    /// alongside the key so removal paths can decrement `total_bytes` accurately
    /// (CR-014/CR-015: previously only `CacheKey` was stored, so eviction
    /// decremented by the fixed `AVG_ROW_SIZE_BYTES` regardless of actual size).
    partition_counts: Mutex<HashMap<PartitionKey, PartitionVec>>,
    /// Total estimated byte usage across all cache entries (BC-2.07.006, CR-006).
    total_bytes: AtomicUsize,
    /// Aggregate cache hit counter — incremented on every successful get()
    /// (BC-2.07.003, CR-005). Using an aggregate counter avoids cloning the
    /// full `rows` Vec on every hot-path hit; per-entry counts are not required
    /// by the BC.
    total_hits: AtomicU64,
}

impl QueryCache {
    /// Construct a new `QueryCache` with the given configuration.
    pub fn new(config: CacheConfig) -> Self {
        // moka capacity: large global pool; per-partition bounds via partition_counts.
        // We use a large moka capacity so it never evicts by itself —
        // per-partition eviction is handled manually in `put`.
        let moka_cap: u64 = 100_000;
        // Configure moka's native TTL using the longest TTL (devices: 300s) as the
        // ceiling. Manual is_expired() checks remain as defense-in-depth (CR-004).
        let max_ttl = Duration::from_secs(CACHE_TTL_DEVICES_SECS);
        let inner = MokaCache::builder()
            .max_capacity(moka_cap)
            .time_to_live(max_ttl)
            .build();
        QueryCache {
            config,
            inner,
            partition_counts: Mutex::new(HashMap::new()),
            total_bytes: AtomicUsize::new(0),
            total_hits: AtomicU64::new(0),
        }
    }

    /// Construct a `QueryCache` with default configuration.
    pub fn with_defaults() -> Self {
        Self::new(CacheConfig::default())
    }

    /// Acquire the `partition_counts` mutex, propagating poison as E-CACHE-001.
    ///
    /// SEC-001 / BC-2.07.004 E-CACHE-001: a poisoned mutex means a thread panicked
    /// while holding the lock, leaving the partition map in an unknown state.
    /// Silently recovering with `into_inner()` would operate on corrupted state.
    /// We propagate as `PrismError::Internal` so the caller can terminate cleanly.
    fn lock_partition_counts(
        &self,
    ) -> Result<MutexGuard<'_, HashMap<PartitionKey, PartitionVec>>, PrismError> {
        self.partition_counts
            .lock()
            .map_err(|_| PrismError::Internal {
                detail: "E-CACHE-001: cache mutex poisoned — internal state may be inconsistent; \
                     terminate and restart the query engine"
                    .to_string(),
            })
    }

    /// Look up a cache entry by key.
    ///
    /// Returns `Ok(Some(rows))` if the entry exists and is not expired.
    /// Returns `Ok(None)` (cache miss) if the key is absent or the entry has
    /// exceeded its TTL. Expired entries are removed on miss (BC-2.07.003).
    ///
    /// On a cache hit, increments the aggregate `total_hits` counter (BC-2.07.003,
    /// CR-005). Per-entry hit counts are not tracked to avoid cloning the full
    /// `rows` Vec on every hot-path access.
    ///
    /// Returns `Err(PrismError::Internal)` if the mutex is poisoned (E-CACHE-001).
    pub fn get(&self, key: &CacheKey) -> Result<Option<Vec<serde_json::Value>>, PrismError> {
        let entry = match self.inner.get(key) {
            Some(e) => e,
            None => return Ok(None),
        };
        if entry.is_expired() {
            // Remove expired entry — treat as cache miss.
            self.remove_entry(key)?;
            return Ok(None);
        }
        // Increment aggregate hit counter (CR-005: no per-entry clone needed).
        self.total_hits.fetch_add(1, Ordering::Relaxed);

        // Update LRU position: move this key to the end of the partition Vec
        // (most-recently-used position) so eviction targets the front (LRU).
        let pk = partition_key(key);
        let mut counts = self.lock_partition_counts()?;
        if let Some(partition_keys) = counts.get_mut(&pk) {
            // Move key to end (most-recently-used), preserving stored byte size.
            if let Some(pos) = partition_keys.iter().position(|(k, _)| k == key) {
                let item = partition_keys.remove(pos);
                partition_keys.push(item);
            }
        }

        Ok(Some(entry.rows.clone()))
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
    ///
    /// Returns `Err(PrismError::Internal)` if the mutex is poisoned (E-CACHE-001).
    pub fn put(&self, key: CacheKey, rows: Vec<serde_json::Value>) -> Result<(), PrismError> {
        let data_type = SourceDataType::from_source_id(&key.source_id);
        let ttl = match data_type.ttl() {
            Some(t) => t,
            None => return Ok(()), // HealthStatus — not cached
        };
        self.put_with_ttl(key, rows, ttl)
    }

    /// Insert with explicit TTL override (for testing or admin bypass).
    ///
    /// Callers are responsible for not passing HealthStatus source keys — the
    /// TTL contract is undefined for uncacheable types. The public `put()` method
    /// enforces this by checking `SourceDataType` before calling `put_with_ttl`
    /// (CR-003: duplicate check removed from this method).
    ///
    /// Returns `Err(PrismError::Internal)` if the mutex is poisoned (E-CACHE-001).
    pub(crate) fn put_with_ttl(
        &self,
        key: CacheKey,
        rows: Vec<serde_json::Value>,
        ttl: Duration,
    ) -> Result<(), PrismError> {
        // max_entries_per_sensor == 0 → caching disabled.
        if self.config.max_entries_per_sensor == 0 {
            return Ok(());
        }

        // Per-entry byte cap: reject entries exceeding MAX_ENTRY_BYTES (SEC-003).
        let entry_size = rows.len() * AVG_ROW_SIZE_BYTES;
        if entry_size > MAX_ENTRY_BYTES {
            return Ok(()); // silent drop: caller receives no cache benefit but no error
        }

        // Total byte budget enforcement (BC-2.07.006, CR-006).
        // If adding this entry would exceed max_bytes, reject it.
        //
        // Note: This budget check uses Relaxed atomics + a separate fetch_add after
        // the partition lock. Under concurrent insert load, multiple threads can
        // pass the check window before any of them increments, causing transient
        // soft over-commitment of the 50MB budget by up to N × MAX_ENTRY_BYTES.
        // This is acceptable for a non-security-boundary cache: the budget is a
        // resource-management heuristic, not a hard memory bound. Tracked as
        // SEC-NEW-002 (pre-existing); future hardening could move fetch_add inside
        // the partition lock or use compare_exchange.
        let current_bytes = self.total_bytes.load(Ordering::Relaxed);
        if current_bytes + entry_size > self.config.max_bytes {
            // Byte budget exceeded — reject to avoid unbounded memory (CR-006 intent).
            return Ok(());
        }

        let pk = partition_key(&key);

        // Enforce per-partition bound synchronously before insert (DI-018).
        let mut counts = self.lock_partition_counts()?;
        let partition_keys = counts.entry(pk.clone()).or_default();

        // Evict LRU entries until there is space for the new entry.
        while partition_keys.len() >= self.config.max_entries_per_sensor {
            // Remove the first entry in the Vec (oldest = LRU for FIFO tiebreaker).
            // CR-015: use the stored byte size, not the fixed AVG_ROW_SIZE_BYTES.
            let (evict_key, evicted_size) = partition_keys.remove(0);
            self.inner.invalidate(&evict_key);
            // CR-014/CR-015: decrement by the evicted entry's actual stored size.
            // SEC-NEW-001: use saturating fetch_update to prevent usize underflow
            // wrapping to usize::MAX (which would DoS all future inserts).
            let _ =
                self.total_bytes
                    .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
                        Some(current.saturating_sub(evicted_size))
                    });
        }

        // Track the new key for this partition.
        // CR-014: if this key already exists (force_refresh path), remove the old
        // tuple first so the byte size is updated to the new entry's size.
        partition_keys.retain(|(k, _)| k != &key);
        partition_keys.push((key.clone(), entry_size));

        drop(counts); // release lock before insert

        // Update total byte count.
        self.total_bytes.fetch_add(entry_size, Ordering::Relaxed);

        let entry = CacheEntry {
            rows,
            created_at: Instant::now(),
            ttl,
        };
        self.inner.insert(key, entry);
        Ok(())
    }

    /// Bypass the cache and replace an existing entry with fresh data.
    ///
    /// Implements `force_refresh: true` semantics (BC-2.07.003 §Postconditions).
    /// The `push_down_hash` of `key` matches the non-forced version; the entry
    /// is overwritten.
    ///
    /// Returns `Err(PrismError::Internal)` if the mutex is poisoned (E-CACHE-001).
    pub fn force_refresh(
        &self,
        key: CacheKey,
        rows: Vec<serde_json::Value>,
    ) -> Result<(), PrismError> {
        // Remove existing entry if present, then insert fresh.
        self.remove_entry(&key)?;
        self.put(key, rows)
    }

    /// Remove all entries whose key matches a `(client_id, sensor_id, source_id)`
    /// prefix (for invalidation by source).
    ///
    /// This is the low-level primitive used by [`crate::invalidation::CacheInvalidator`].
    ///
    /// Returns `Err(PrismError::Internal)` if the mutex is poisoned (E-CACHE-001).
    pub fn invalidate_by_prefix(
        &self,
        client_id: &str,
        sensor_id: &str,
        source_id: &str,
    ) -> Result<(), PrismError> {
        let mut counts = self.lock_partition_counts()?;
        let pk = (client_id.to_string(), sensor_id.to_string());

        // Single-pass eviction: collect matching entries and strip them from the
        // partition vec in one retain() call — O(n) instead of O(n×m) (CR-001).
        let mut evicted_keys: Vec<CacheKey> = Vec::new();
        let mut to_decrement: usize = 0;

        if let Some(partition_keys) = counts.get_mut(&pk) {
            partition_keys.retain(|(k, size)| {
                if k.source_id == source_id {
                    evicted_keys.push(k.clone());
                    to_decrement = to_decrement.saturating_add(*size);
                    false // remove from partition
                } else {
                    true // keep
                }
            });

            // CR-004: remove empty partition vec so the HashMap doesn't accumulate
            // stale entries for fully-evicted (client_id, sensor_id) pairs.
            if partition_keys.is_empty() {
                counts.remove(&pk);
            }
        }

        // Drop the partition lock before invalidating moka entries to avoid
        // holding the mutex across potentially-blocking moka operations.
        drop(counts);

        // Invalidate moka entries (idempotent — safe to call even if already evicted).
        for k in &evicted_keys {
            self.inner.invalidate(k);
        }

        // Decrement total_bytes once for the entire evicted batch.
        // SEC-NEW-001: saturating to prevent usize underflow on unexpected double-evict.
        if to_decrement > 0 {
            let _ =
                self.total_bytes
                    .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
                        Some(current.saturating_sub(to_decrement))
                    });
        }

        Ok(())
    }

    /// Remove all entries whose `client_id` matches `client_id`.
    ///
    /// Used for client management write operations (BC-2.07.004).
    ///
    /// Returns `Err(PrismError::Internal)` if the mutex is poisoned (E-CACHE-001).
    pub fn invalidate_by_client(&self, client_id: &str) -> Result<(), PrismError> {
        let mut counts = self.lock_partition_counts()?;

        // Find all partitions for this client.
        let client_partitions: Vec<PartitionKey> = counts
            .keys()
            .filter(|(cid, _)| cid == client_id)
            .cloned()
            .collect();

        for pk in client_partitions {
            if let Some(partition_keys) = counts.remove(&pk) {
                for (k, stored_size) in partition_keys {
                    self.inner.invalidate(&k);
                    // CR-014: decrement total_bytes by the stored size of each evicted entry.
                    // SEC-NEW-001: saturating to prevent usize underflow on unexpected double-evict.
                    let _ = self.total_bytes.fetch_update(
                        Ordering::Relaxed,
                        Ordering::Relaxed,
                        |current| Some(current.saturating_sub(stored_size)),
                    );
                }
            }
        }
        Ok(())
    }

    /// Returns the current entry count after draining moka's write-op buffer.
    ///
    /// **Note:** This method calls `run_pending_tasks()` to ensure an accurate
    /// snapshot, which can block under high write contention. Do NOT call this
    /// in hot paths. For approximate metrics, use the approximate entry count
    /// from moka directly, or add a separate `AtomicU64` counter (CR-002).
    ///
    /// Currently only called in test code — if external callers are added,
    /// consider gating the `run_pending_tasks()` call behind `#[cfg(test)]`.
    pub fn entry_count(&self) -> u64 {
        // moka's entry_count may lag by a tick; sync first.
        self.inner.run_pending_tasks();
        self.inner.entry_count()
    }

    /// Returns the estimated total bytes currently tracked in the cache (for metrics).
    pub fn total_bytes(&self) -> usize {
        self.total_bytes.load(Ordering::Relaxed)
    }

    /// Returns the aggregate number of cache hits since the cache was created.
    ///
    /// Incremented on every successful `get()` call that returns `Some(rows)`.
    /// Provides `check_sensor_health` visibility per BC-2.07.003 without the
    /// hot-path cost of cloning per-entry `rows` vecs (CR-005).
    pub fn total_hits(&self) -> u64 {
        self.total_hits.load(Ordering::Relaxed)
    }

    // Internal: remove a single entry from both moka and partition tracker.
    //
    // Returns `Err(PrismError::Internal)` if the mutex is poisoned (E-CACHE-001).
    fn remove_entry(&self, key: &CacheKey) -> Result<(), PrismError> {
        self.inner.invalidate(key);
        let mut counts = self.lock_partition_counts()?;
        let pk = partition_key(key);
        if let Some(partition_keys) = counts.get_mut(&pk) {
            // CR-014: look up the stored byte size before removing the tuple so
            // we can decrement total_bytes accurately.
            // SEC-NEW-001: saturating to prevent usize underflow on unexpected double-remove.
            if let Some(pos) = partition_keys.iter().position(|(k, _)| k == key) {
                let (_, stored_size) = partition_keys.remove(pos);
                let _ = self.total_bytes.fetch_update(
                    Ordering::Relaxed,
                    Ordering::Relaxed,
                    |current| Some(current.saturating_sub(stored_size)),
                );
            }
            // CR-006: remove the partition entry if now empty to prevent unbounded
            // growth of the partition_counts map (TTL-expiry and force_refresh paths).
            if partition_keys.is_empty() {
                counts.remove(&pk);
            }
        }
        Ok(())
    }

    /// Insert directly into the moka cache without byte accounting or partition
    /// tracking. For use in regression tests that need to prime the cache with
    /// a specific entry bypassing `put_with_ttl` invariants (e.g., to test
    /// SEC-001 mutex-poison behaviour). (CR-016)
    #[cfg(test)]
    pub fn insert_raw_for_test(&self, key: CacheKey, entry: CacheEntry) {
        self.inner.insert(key, entry);
    }

    /// Returns the number of partition keys currently tracked in `partition_counts`.
    /// A value of 0 means the map is fully clean — no stale empty-Vec entries remain.
    /// For use in regression tests only (CR-006).
    #[cfg(test)]
    pub fn partition_count_map_len(&self) -> usize {
        self.partition_counts.lock().map(|g| g.len()).unwrap_or(0)
    }
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
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

    /// CR-006 / BC-2.07.006: default max_bytes is 50 MB.
    #[test]
    fn test_default_config_max_bytes_is_50mb() {
        assert_eq!(CacheConfig::default().max_bytes, 50 * 1024 * 1024);
    }

    /// CR-007: suffix match — sources ending with `_health` or `_status` are HealthStatus.
    #[test]
    fn test_source_data_type_suffix_health_status() {
        assert_eq!(
            SourceDataType::from_source_id("crowdstrike_health"),
            SourceDataType::HealthStatus,
        );
        assert_eq!(
            SourceDataType::from_source_id("armis_status"),
            SourceDataType::HealthStatus,
        );
    }

    /// CR-007: suffix match — sources ending with `_alerts` or `_detections` are AlertsDetections.
    #[test]
    fn test_source_data_type_suffix_alerts_detections() {
        assert_eq!(
            SourceDataType::from_source_id("crowdstrike_detections"),
            SourceDataType::AlertsDetections,
        );
        assert_eq!(
            SourceDataType::from_source_id("cyberint_alerts"),
            SourceDataType::AlertsDetections,
        );
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
        cache
            .put(key.clone(), rows.clone())
            .expect("put must succeed");
        let result = cache.get(&key).expect("get must not fail");
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
        let result = cache.get(&key).expect("get must not fail");
        assert!(
            result.is_none(),
            "cache miss on unseen key must return None"
        );
    }

    /// AC-8 / BC-2.07.006: At capacity, inserting a new entry evicts LRU.
    #[test]
    fn test_ac8_lru_eviction_at_capacity() {
        let config = CacheConfig {
            max_entries_per_sensor: 2,
            max_bytes: DEFAULT_MAX_CACHE_BYTES,
        };
        let cache = QueryCache::new(config);

        let make_key = |n: u8| crate::cache_key::CacheKey {
            client_id: "acme".to_string(),
            sensor_id: "armis".to_string(),
            source_id: "armis_devices".to_string(),
            push_down_hash: format!("{:0<64}", n),
        };
        // Fill to capacity.
        cache
            .put(make_key(1), vec![serde_json::json!({"id": 1})])
            .expect("put 1");
        cache
            .put(make_key(2), vec![serde_json::json!({"id": 2})])
            .expect("put 2");
        // Third insert must evict LRU — total stays at most 2.
        cache
            .put(make_key(3), vec![serde_json::json!({"id": 3})])
            .expect("put 3");

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

        cache.put(key.clone(), old_rows).expect("put old");
        cache
            .force_refresh(key.clone(), new_rows.clone())
            .expect("force_refresh");

        assert_eq!(
            cache.get(&key).expect("get must not fail"),
            Some(new_rows),
            "force_refresh must overwrite existing entry with fresh data"
        );
    }

    /// SEC-001 / BC-2.07.004 E-CACHE-001: poisoned mutex returns E-CACHE-001 error.
    ///
    /// Regression test: a thread that panics while holding the partition_counts lock
    /// poisons the mutex. Subsequent operations must return E-CACHE-001 instead of
    /// silently recovering with potentially corrupted state.
    #[test]
    fn test_sec001_poisoned_mutex_returns_e_cache_001() {
        use std::sync::Arc;

        let cache = Arc::new(QueryCache::with_defaults());

        // Insert an entry first so a subsequent get() reaches the lock path.
        // (get() acquires the lock only on a cache hit to update LRU position.)
        let key = crate::cache_key::CacheKey {
            client_id: "acme".to_string(),
            sensor_id: "crowdstrike".to_string(),
            source_id: "crowdstrike_detections".to_string(),
            push_down_hash: "e".repeat(64),
        };
        // Insert directly into moka without going through put() (which would also lock).
        // CR-016: use insert_raw_for_test instead of accessing the private inner field.
        let entry = CacheEntry {
            rows: vec![serde_json::json!({"id": "det-1"})],
            created_at: std::time::Instant::now(),
            ttl: std::time::Duration::from_secs(300),
        };
        cache.insert_raw_for_test(key.clone(), entry);

        let cache_clone = Arc::clone(&cache);

        // Spawn a thread that panics while holding the lock.
        // Poison the mutex directly via raw field access (same-module access to
        // private field). We intentionally bypass lock_partition_counts() to acquire
        // the lock without E-CACHE-001 propagation — the point of the test is to
        // poison the mutex, not to guard against it.
        let handle = std::thread::spawn(move || {
            let _guard = cache_clone
                .partition_counts
                .lock()
                .expect("lock must succeed before poison");
            panic!("simulated panic while holding cache mutex");
        });

        // The thread panics — this poisons the mutex. Ignore the join error.
        let _ = handle.join();

        // Now attempt a get() on an existing entry — it hits the lock (for LRU update).
        // Must return E-CACHE-001.
        let result = cache.get(&key);
        match result {
            Err(PrismError::Internal { detail }) => {
                assert!(
                    detail.contains("E-CACHE-001"),
                    "poisoned mutex must return E-CACHE-001; got: {detail}"
                );
            }
            other => panic!(
                "SEC-001: poisoned mutex must return Err(PrismError::Internal{{E-CACHE-001}}); \
                 got: {other:?}"
            ),
        }
    }

    /// CR-006 / BC-2.07.006: byte-based eviction — cache rejects entries when
    /// total_bytes would exceed max_bytes.
    #[test]
    fn test_cr006_byte_budget_exceeded_rejects_entry() {
        // Set a very small byte budget so a few entries fill it.
        let max_bytes_budget = AVG_ROW_SIZE_BYTES * 5; // only 5 "rows" worth of budget
        let config = CacheConfig {
            max_entries_per_sensor: 1000,
            max_bytes: max_bytes_budget,
        };
        let cache = QueryCache::new(config);

        // Insert entries until budget is exceeded.
        // Each entry has 3 rows × 512 bytes = 1536 bytes estimated.
        let rows = vec![
            serde_json::json!({"id": "a"}),
            serde_json::json!({"id": "b"}),
            serde_json::json!({"id": "c"}),
        ];

        let make_key = |n: u8| crate::cache_key::CacheKey {
            client_id: "acme".to_string(),
            sensor_id: "crowdstrike".to_string(),
            source_id: "crowdstrike_detections".to_string(),
            push_down_hash: format!("{n:0<64}"),
        };

        // First entry: 3 rows × 512 bytes = 1536 bytes (within 5 × 512 = 2560 budget).
        cache
            .put(make_key(1), rows.clone())
            .expect("first entry must fit");
        cache.put(make_key(2), rows.clone()).expect("second entry");
        // Any further put may be silently rejected (budget is likely exceeded).
        let result = cache.put(make_key(3), rows.clone());
        // The put itself must not error.
        assert!(result.is_ok(), "byte budget rejection must not error");

        // The cache must not have grown substantially beyond the budget.
        assert!(
            cache.total_bytes() <= max_bytes_budget + AVG_ROW_SIZE_BYTES * 3,
            "total_bytes must not substantially exceed max_bytes budget"
        );
    }

    /// CR-014: force-refresh the same key 10 times; total_bytes must reflect
    /// approximately one entry's worth of bytes, not 10×.
    ///
    /// Before the fix, `remove_entry` (called by `force_refresh`) did not
    /// decrement `total_bytes`, causing monotonic growth on repeated refreshes
    /// until the byte budget was exhausted and inserts were silently rejected.
    #[test]
    fn test_cr014_force_refresh_does_not_inflate_total_bytes() {
        let config = CacheConfig {
            max_entries_per_sensor: 50,
            max_bytes: DEFAULT_MAX_CACHE_BYTES,
        };
        let cache = QueryCache::new(config);
        let key = crate::cache_key::CacheKey {
            client_id: "acme".to_string(),
            sensor_id: "crowdstrike".to_string(),
            source_id: "crowdstrike_detections".to_string(),
            push_down_hash: "f".repeat(64),
        };
        let rows = vec![
            serde_json::json!({"id": "det-1"}),
            serde_json::json!({"id": "det-2"}),
        ];
        let one_entry_bytes = rows.len() * AVG_ROW_SIZE_BYTES;

        // Force-refresh the same key 10 times.
        for i in 0..10u32 {
            let mut r = rows.clone();
            r[0]["seq"] = serde_json::json!(i);
            cache
                .force_refresh(key.clone(), r)
                .expect("force_refresh must not error");
        }

        // total_bytes must reflect approximately one entry (not 10×).
        let total = cache.total_bytes();
        assert!(
            total <= one_entry_bytes * 2,
            "CR-014: after 10 force-refreshes, total_bytes should be ~{one_entry_bytes} \
             (one entry); got {total} (expected at most {})",
            one_entry_bytes * 2
        );
    }

    /// CR-014: after filling the cache for a given prefix and then invalidating
    /// by prefix, total_bytes must drop back to approximately zero.
    ///
    /// Before the fix, `invalidate_by_prefix` called `inner.invalidate()` but
    /// never decremented `total_bytes`, causing the byte counter to stay
    /// elevated even after entries were evicted.
    #[test]
    fn test_cr014_invalidate_decrements_total_bytes() {
        let config = CacheConfig {
            max_entries_per_sensor: 100,
            max_bytes: DEFAULT_MAX_CACHE_BYTES,
        };
        let cache = QueryCache::new(config);

        let n = 10usize;
        let rows_per_entry = 3usize;
        let rows = (0..rows_per_entry)
            .map(|i| serde_json::json!({"id": i}))
            .collect::<Vec<_>>();

        // Insert n entries under the same (client, sensor, source) prefix.
        for i in 0..n {
            let key = crate::cache_key::CacheKey {
                client_id: "acme".to_string(),
                sensor_id: "crowdstrike".to_string(),
                source_id: "crowdstrike_detections".to_string(),
                push_down_hash: format!("{i:0<64}"),
            };
            cache.put(key, rows.clone()).expect("put must not error");
        }

        let bytes_after_fill = cache.total_bytes();
        let expected_per_entry = rows_per_entry * AVG_ROW_SIZE_BYTES;
        assert!(
            bytes_after_fill >= expected_per_entry * n,
            "should have ~{} bytes after filling; got {bytes_after_fill}",
            expected_per_entry * n
        );

        // Invalidate the entire prefix.
        cache
            .invalidate_by_prefix("acme", "crowdstrike", "crowdstrike_detections")
            .expect("invalidate_by_prefix must not error");

        let bytes_after_invalidate = cache.total_bytes();
        assert_eq!(
            bytes_after_invalidate, 0,
            "CR-014/CR-019: total_bytes must be exactly 0 after full prefix invalidation; \
             got {bytes_after_invalidate} — every byte added by the {n} entries must be \
             decremented by invalidate_by_prefix"
        );
    }
}
