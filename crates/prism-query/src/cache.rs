//! `cache` — Sensor-fetch response cache with configurable TTL and LRU eviction.
//!
//! Implements BC-2.07.003 and BC-2.07.006.
//!
//! # Cache model
//! A single in-memory sensor-fetch cache keyed by [`crate::cache_key::CacheKey`]
//! (4-tuple: `client_id`, `sensor_id`, `source_id`, `push_down_hash`).
//! BC-2.07.003 specifies entries store the raw sensor API response pre-OCSF
//! normalization; the production instantiation ([`SensorResponseValue`])
//! caches the adapter's normalized Arrow output instead — a human-authorized
//! deviation (P1-03 adjudication, 2026-06-10; spec compliance lands in
//! S-CACHE-SPEC-COMPLIANCE-001; interim mitigation: hot-reload cache flush).
//! See [`CacheValue`] for the full deviation record. PrismQL post-filters are
//! applied after cache retrieval (BC-2.07.003 §Postconditions).
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
//! cache operations return `Err(PrismError::Internal)` with a prose-only detail
//! instead of silently continuing with potentially corrupted state (BC-2.07.004
//! mutex-poison condition, SEC-001).
//!
//! # BC References
//! - BC-2.07.003 — Query Engine Sensor-Fetch Cache with Configurable TTL
//! - BC-2.07.006 — Cache Memory Bounds and Eviction Policy
//!
//! Story: S-3.05

use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicU64, AtomicUsize, Ordering},
        Mutex, MutexGuard,
    },
    time::{Duration, Instant},
};

use moka::sync::Cache as MokaCache;
use prism_core::error::PrismError;
use tracing::debug;

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
// CacheValue
// ---------------------------------------------------------------------------

/// A value type storable in the sensor-fetch response cache.
///
/// The cache logic (TTL, LRU, per-partition bounds, byte budget, prefix
/// invalidation) is value-type agnostic; only the byte-size estimation differs.
/// Two instantiations exist:
/// - `Vec<serde_json::Value>` — raw JSON rows (the BC-2.07.003 "raw sensor
///   response" form, used by unit tests and JSON-level callers).
/// - [`SensorResponseValue`] (`Vec<RecordBatch>`) — the production query-engine
///   instantiation, storing the post-normalization Arrow output of
///   `SensorAdapter::fetch`.
///
/// # Authorized deviation from BC-2.07.003 (P1-03 adjudication, 2026-06-10)
///
/// BC-2.07.003 specifies that cache entries store the RAW sensor API response
/// pre-OCSF normalization. The production instantiation deviates: it caches
/// the adapter's normalized `RecordBatch` output, because normalization
/// happens inside the as-built `SensorAdapter::fetch` boundary. This is a
/// HUMAN-AUTHORIZED deviation from the BC-2.07.003 raw-response model (P1-03
/// adjudication, 2026-06-10); the BC stays unamended. Spec compliance is
/// implemented by S-CACHE-SPEC-COMPLIANCE-001 (post-demo). Interim mitigation:
/// every config hot-reload flushes this cache (`invalidate_all`, registered as
/// a `ConfigManager` swap listener at boot), evicting all pre-swap-epoch
/// entries. Note the flush is forward-provisioning today — fetch-path
/// normalization is boot-frozen (adapters hold the boot-time resolved spec),
/// and in-flight pre-swap queries may insert after the flush; see
/// [`GenericQueryCache::invalidate_all`] for the full caveats.
/// Full reload-effectiveness arrives with S-CACHE-SPEC-COMPLIANCE-001's
/// normalize-on-read model.
pub trait CacheValue: Clone + std::fmt::Debug + Send + Sync + 'static {
    /// Estimated in-memory size of this value in bytes, used for the
    /// BC-2.07.006 byte-budget accounting and the SEC-003 per-entry cap.
    fn estimated_bytes(&self) -> usize;
}

impl CacheValue for Vec<serde_json::Value> {
    /// Conservative estimate: [`AVG_ROW_SIZE_BYTES`] per JSON row (CR-006).
    fn estimated_bytes(&self) -> usize {
        self.len() * AVG_ROW_SIZE_BYTES
    }
}

impl CacheValue for Vec<arrow::record_batch::RecordBatch> {
    /// Actual Arrow buffer memory for the batch set.
    ///
    /// `get_array_memory_size` reports the total bytes of all buffers backing
    /// the batch (shared `Arc` buffers may be counted per referencing batch —
    /// a conservative over-estimate, which is the safe direction for a budget).
    fn estimated_bytes(&self) -> usize {
        self.iter()
            .map(arrow::record_batch::RecordBatch::get_array_memory_size)
            .sum()
    }
}

/// Production cache value: complete sensor-fetch response as Arrow batches.
///
/// Stores the adapter's NORMALIZED output — a human-authorized deviation from
/// the BC-2.07.003 raw-response model (P1-03 adjudication, 2026-06-10); spec
/// compliance implemented by S-CACHE-SPEC-COMPLIANCE-001 (post-demo); interim
/// mitigation: hot-reload response-cache flush. See [`CacheValue`].
pub type SensorResponseValue = Vec<arrow::record_batch::RecordBatch>;

// ---------------------------------------------------------------------------
// CacheEntry
// ---------------------------------------------------------------------------

/// A single cached sensor-fetch response.
///
/// Stores the complete sensor API response (no query-engine transformation
/// applied before caching) along with metadata for TTL enforcement
/// (BC-2.07.003 §Postconditions).
///
/// Hit metrics are tracked at the cache level via `QueryCache::total_hits()`
/// rather than per-entry to avoid cloning the full `rows` value on every hit
/// (CR-005). Aggregate counts are sufficient for `check_sensor_health` visibility.
#[derive(Debug, Clone)]
pub struct CacheEntry<V: CacheValue = Vec<serde_json::Value>> {
    /// Complete sensor API response (pre-query-engine transformation).
    pub rows: V,
    /// Absolute creation timestamp — TTL is measured from this (BC-2.07.003).
    pub created_at: Instant,
    /// TTL duration for this entry (data-type dependent).
    pub ttl: Duration,
}

impl<V: CacheValue> CacheEntry<V> {
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
    (key.client_id.clone(), key.sensor_id.as_ref().to_owned())
}

// ---------------------------------------------------------------------------
// QueryCache
// ---------------------------------------------------------------------------

/// Thread-safe sensor-fetch response cache, generic over the cached value type.
///
/// Implements BC-2.07.003 (TTL-based caching) and BC-2.07.006 (LRU eviction
/// with per-partition entry count bound and 50 MB total byte budget). Intended
/// to be held in a single `Arc` shared across all `QueryEngine` tasks.
///
/// Two instantiations are exported:
/// - [`QueryCache`] (`GenericQueryCache<Vec<serde_json::Value>>`) — JSON-row form.
/// - [`SensorResponseCache`] (`GenericQueryCache<SensorResponseValue>`) — the
///   production query-engine instantiation storing Arrow `RecordBatch` sets.
///
/// Internally uses `moka::sync::Cache` for thread-safe LRU eviction and
/// TTL-based entry expiry (story §Caching Context Summary — moka 0.12).
///
/// ## Mutex poison safety (SEC-001 / BC-2.07.004 mutex-poison condition)
/// If the `partition_counts` mutex is poisoned, all cache operations that
/// require the lock return `Err(PrismError::Internal)` with a prose-only detail.
/// Silent recovery via `unwrap_or_else(|e| e.into_inner())` is prohibited.
pub struct GenericQueryCache<V: CacheValue> {
    config: CacheConfig,
    /// moka LRU cache: provides O(1) get/put with background TTL eviction.
    /// Large capacity; per-partition bounds enforced by `partition_counts`.
    inner: MokaCache<CacheKey, CacheEntry<V>>,
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
    /// full `rows` value on every hot-path hit; per-entry counts are not required
    /// by the BC.
    total_hits: AtomicU64,
}

/// JSON-row instantiation of the sensor-fetch cache (historical name).
///
/// All pre-existing unit tests and JSON-level callers use this alias; the
/// production query engine uses [`SensorResponseCache`].
pub type QueryCache = GenericQueryCache<Vec<serde_json::Value>>;

/// Production sensor-fetch response cache instantiation (BC-2.07.003).
///
/// Stores the complete per-`(client_id, sensor_id, source_id, push_down_hash)`
/// sensor fetch response as Arrow `RecordBatch`es, shared between
/// `QueryEngine` (read/populate path) and `CacheInvalidator` (write
/// invalidation path, BC-2.07.004).
pub type SensorResponseCache = GenericQueryCache<SensorResponseValue>;

impl<V: CacheValue> GenericQueryCache<V> {
    /// Construct a new cache with the given configuration.
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
        GenericQueryCache {
            config,
            inner,
            partition_counts: Mutex::new(HashMap::new()),
            total_bytes: AtomicUsize::new(0),
            total_hits: AtomicU64::new(0),
        }
    }

    /// Construct a cache with default configuration.
    pub fn with_defaults() -> Self {
        Self::new(CacheConfig::default())
    }

    /// Acquire the `partition_counts` mutex, propagating poison as `PrismError::Internal`.
    ///
    /// SEC-001 / BC-2.07.004: a poisoned mutex means a thread panicked while holding
    /// the lock, leaving the partition map in an unknown state. Silently recovering
    /// with `into_inner()` would operate on corrupted state. We propagate as
    /// `PrismError::Internal` (prose-only detail) so the caller can terminate cleanly.
    fn lock_partition_counts(
        &self,
    ) -> Result<MutexGuard<'_, HashMap<PartitionKey, PartitionVec>>, PrismError> {
        self.partition_counts
            .lock()
            .map_err(|_| PrismError::Internal {
                detail:
                    "cache partition_counts mutex poisoned — internal state may be inconsistent; \
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
    pub fn get(&self, key: &CacheKey) -> Result<Option<V>, PrismError> {
        let entry = match self.inner.get(key) {
            Some(e) => e,
            None => {
                // I-1: log cache miss with diagnostic fields (no query string / PII).
                debug!(
                    sensor_id = %key.sensor_id,
                    source_id = %key.source_id,
                    client_id = %key.client_id,
                    "cache miss"
                );
                return Ok(None);
            }
        };
        if entry.is_expired() {
            // Remove expired entry — treat as cache miss.
            debug!(
                sensor_id = %key.sensor_id,
                source_id = %key.source_id,
                client_id = %key.client_id,
                "cache miss (expired)"
            );
            self.remove_entry(key)?;
            return Ok(None);
        }
        // Increment aggregate hit counter (CR-005: no per-entry clone needed).
        self.total_hits.fetch_add(1, Ordering::Relaxed);
        // I-1: log cache hit with diagnostic fields (no query string / PII).
        debug!(
            sensor_id = %key.sensor_id,
            source_id = %key.source_id,
            client_id = %key.client_id,
            "cache hit"
        );

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
    pub fn put(&self, key: CacheKey, rows: V) -> Result<(), PrismError> {
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
    ///
    /// ## Atomicity (TD-PRISM-QUERY-CACHE-001 closure)
    ///
    /// ALL cache mutations — partition-tracker updates, moka invalidations for
    /// LRU-evicted entries, `total_bytes` adjustments, and the final moka insert —
    /// occur INSIDE the partition lock. This closes the SEC-NEW-002 evict-vs-put
    /// race where a concurrent `put_with_ttl(evict_key, ...)` could interleave
    /// between the lock release and the post-lock `inner.invalidate(evict_key)`,
    /// leaving the partition tracker orphaned (tracked-but-not-in-moka) and
    /// `total_bytes` inflated. moka operations are in-memory and never call back
    /// into this module, so holding the mutex across them cannot deadlock.
    pub(crate) fn put_with_ttl(
        &self,
        key: CacheKey,
        rows: V,
        ttl: Duration,
    ) -> Result<(), PrismError> {
        // max_entries_per_sensor == 0 → caching disabled.
        if self.config.max_entries_per_sensor == 0 {
            return Ok(());
        }

        // Per-entry byte cap: reject entries exceeding MAX_ENTRY_BYTES (SEC-003).
        let entry_size = rows.estimated_bytes();
        if entry_size > MAX_ENTRY_BYTES {
            return Ok(()); // silent drop: caller receives no cache benefit but no error
        }

        let pk = partition_key(&key);

        // Enforce per-partition bound synchronously before insert (DI-018).
        //
        // Lock pattern (TD-PRISM-QUERY-CACHE-001): partition accounting, moka
        // invalidations for LRU-evicted entries, total_bytes adjustments, AND the
        // final moka insert ALL happen inside the partition lock so the tracker
        // and moka can never diverge (SEC-NEW-002 evict-vs-put race closed).
        // moka operations are in-memory and never re-enter this module, so
        // holding the mutex across them cannot deadlock.
        //
        // C10-001 fix: the net byte-change budget check factors in evicted_bytes so
        // that LRU eviction can free budget for the incoming entry. On rejection,
        // evicted entries are restored to partition_keys so the partition remains
        // consistent (no orphan moka entries, no inflated total_bytes). moka
        // invalidation for evicted entries happens only AFTER the budget check
        // passes, so the restore path never has to undo a moka invalidation.
        {
            let mut counts = self.lock_partition_counts()?;
            let partition_keys = counts.entry(pk.clone()).or_default();

            // Evict LRU entries until there is space for the new entry.
            let mut evicted: Vec<(CacheKey, usize)> = Vec::new();
            while partition_keys.len() >= self.config.max_entries_per_sensor {
                // Remove the first entry in the Vec (oldest = LRU for FIFO tiebreaker).
                // CR-015: use the stored byte size, not the fixed AVG_ROW_SIZE_BYTES.
                evicted.push(partition_keys.remove(0));
            }

            // Sum of bytes freed by LRU eviction above (C10-001).
            let evicted_bytes: usize = evicted.iter().map(|(_, sz)| *sz).sum();

            // Track the new key for this partition.
            // CR-014: if this key already exists (force_refresh / repeated-put path),
            // capture its stored byte size BEFORE retain so we can subtract it from
            // total_bytes.
            // CRITICAL-P8-001: the previous code retained without capturing the old size,
            // causing monotonic total_bytes growth on repeated puts to the same key.
            let existing_size = partition_keys
                .iter()
                .find(|(k, _)| k == &key)
                .map(|(_, sz)| *sz)
                .unwrap_or(0);

            // Total byte budget enforcement (BC-2.07.006, CR-006, I9-002, C10-001).
            // Compute net byte change accounting for: the existing entry being replaced
            // AND the bytes freed by LRU eviction above. This ensures that when LRU
            // eviction frees sufficient budget for the incoming entry, the put succeeds
            // rather than being incorrectly rejected. Same-key replacement always passes
            // (net ≤ 0 when new ≤ old).
            //
            // If net_change > 0 and budget is exceeded even AFTER eviction, we must
            // restore the evicted entries to partition_keys before returning so the
            // partition remains consistent (no orphan moka entries, no inflated
            // total_bytes) — C10-001 closure. Nothing has been invalidated in moka at
            // this point, so the restore is purely a tracker operation.
            let current_bytes = self.total_bytes.load(Ordering::Relaxed);
            let net_change = entry_size as i64 - existing_size as i64 - evicted_bytes as i64;
            if net_change > 0
                && current_bytes.saturating_add(net_change as usize) > self.config.max_bytes
            {
                // Byte budget exceeded even after accounting for LRU eviction.
                for evicted_entry in evicted.into_iter().rev() {
                    partition_keys.insert(0, evicted_entry);
                }
                return Ok(());
            }

            // Budget check passed — commit the LRU evictions: invalidate moka entries
            // and decrement total_bytes INSIDE the lock, atomic with the partition
            // mutation above (TD-PRISM-QUERY-CACHE-001 / SEC-NEW-002 closure).
            for (evict_key, evicted_size) in &evicted {
                // I-1: log LRU eviction event with diagnostic fields.
                debug!(
                    sensor_id = %evict_key.sensor_id,
                    source_id = %evict_key.source_id,
                    client_id = %evict_key.client_id,
                    evicted_bytes = evicted_size,
                    "cache LRU eviction"
                );
                self.inner.invalidate(evict_key);
                // CR-014/CR-015: decrement by the evicted entry's actual stored size.
                // SEC-NEW-001: use saturating fetch_update to prevent usize underflow
                // wrapping to usize::MAX (which would DoS all future inserts).
                let _ = self.total_bytes.fetch_update(
                    Ordering::Relaxed,
                    Ordering::Relaxed,
                    |current| Some(current.saturating_sub(*evicted_size)),
                );
            }

            partition_keys.retain(|(k, _)| k != &key);
            partition_keys.push((key.clone(), entry_size));

            // FIX-OBS-007: apply the net byte change for this key replacement INSIDE
            // the partition lock so that the subtraction-then-addition is serialized
            // with respect to other concurrent puts to the same key.
            // SEC-NEW-001: saturating arithmetic to prevent underflow wrapping.
            if existing_size > 0 {
                let _ = self
                    .total_bytes
                    .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |b| {
                        Some(b.saturating_sub(existing_size))
                    });
            }
            self.total_bytes.fetch_add(entry_size, Ordering::Relaxed);

            // I-1: log insert with diagnostic fields (entry size, TTL — no query
            // string / PII).
            debug!(
                sensor_id = %key.sensor_id,
                source_id = %key.source_id,
                client_id = %key.client_id,
                entry_bytes = entry_size,
                ttl_secs = ttl.as_secs(),
                "cache insert"
            );

            // Insert into moka INSIDE the lock so a concurrent remove/invalidate
            // cannot interleave between the tracker push above and this insert
            // (which would orphan the tracker entry).
            let entry = CacheEntry {
                rows,
                created_at: Instant::now(),
                ttl,
            };
            self.inner.insert(key, entry);
        } // partition lock released here

        Ok(())
    }

    /// Bypass the cache and replace an existing entry with fresh data.
    ///
    /// Implements `force_refresh: true` semantics (BC-2.07.003 §Postconditions).
    /// The `push_down_hash` of `key` matches the non-forced version; the entry
    /// is overwritten.
    ///
    /// Returns `Err(PrismError::Internal)` if the mutex is poisoned (E-CACHE-001).
    ///
    /// ## Atomicity note (TD-PRISM-QUERY-CACHE-001)
    ///
    /// `force_refresh` composes `remove_entry` and `put`, each of which is
    /// individually atomic (tracker + moka + byte accounting mutate under the
    /// partition lock). The pair is NOT atomic: a concurrent reader may observe
    /// a cache miss between the remove and the put. That is acceptable under
    /// BC-2.07.003 force_refresh semantics (bypass + replace) — the miss simply
    /// causes a fresh sensor fetch. No accounting drift or orphan state is
    /// possible in any interleaving.
    pub fn force_refresh(&self, key: CacheKey, rows: V) -> Result<(), PrismError> {
        // Remove existing entry if present, then insert fresh.
        self.remove_entry(&key)?;
        self.put(key, rows)
    }

    /// Remove all entries whose key matches a `(client_id, sensor_id, source_id)`
    /// prefix (for invalidation by source).
    ///
    /// This is the low-level primitive used by [`crate::invalidation::CacheInvalidator`].
    ///
    /// Returns `Ok(n)` where `n` is the number of entries evicted (I-2: audit
    /// postcondition — BC-2.07.004 §Postconditions (audit count) requires the evicted
    /// count to be available for audit logging in the write operation's `AuditEntry`).
    ///
    /// Returns `Err(PrismError::Internal)` if the mutex is poisoned (E-CACHE-001).
    ///
    /// ## Atomicity (TD-PRISM-QUERY-CACHE-001 closure)
    ///
    /// moka invalidation and the `total_bytes` decrement occur INSIDE the
    /// partition lock, atomic with the tracker mutation. A concurrent
    /// `put_with_ttl(k, ...)` either completes fully before this invalidation
    /// (and is then evicted here) or starts after it (and inserts a fresh,
    /// correctly-tracked entry). No orphan tracker entries or byte-accounting
    /// drift are possible (SEC-NEW-002 closed).
    pub fn invalidate_by_prefix(
        &self,
        client_id: &str,
        sensor_id: &str,
        source_id: &str,
    ) -> Result<usize, PrismError> {
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

        let evicted_count = evicted_keys.len();

        // Invalidate moka entries INSIDE the lock — atomic with the tracker
        // mutation above (TD-PRISM-QUERY-CACHE-001 / SEC-NEW-002 closure).
        for k in &evicted_keys {
            self.inner.invalidate(k);
        }

        // Decrement total_bytes once for the entire evicted batch, also inside
        // the lock so accounting is serialized with concurrent puts.
        // SEC-NEW-001: saturating to prevent usize underflow on unexpected double-evict.
        if to_decrement > 0 {
            let _ =
                self.total_bytes
                    .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
                        Some(current.saturating_sub(to_decrement))
                    });
        }

        // Release the lock before emitting the diagnostic.
        drop(counts);

        // O-2: emit diagnostic after lock release so operators can trace "where did my cache go".
        debug!(
            client_id,
            sensor_id, source_id, evicted_count, "cache prefix invalidation"
        );

        Ok(evicted_count)
    }

    /// Remove all entries whose `client_id` matches `client_id`.
    ///
    /// Used for client management write operations (BC-2.07.004).
    ///
    /// Returns `Ok(n)` where `n` is the number of entries evicted (I-2: audit
    /// postcondition — BC-2.07.004 §Postconditions (audit count) requires the evicted
    /// count to be available for audit logging in the write operation's `AuditEntry`).
    ///
    /// Returns `Err(PrismError::Internal)` if the mutex is poisoned (E-CACHE-001).
    ///
    /// ## Atomicity (TD-PRISM-QUERY-CACHE-001 closure)
    ///
    /// moka invalidation and the `total_bytes` decrement occur INSIDE the
    /// partition lock, atomic with the tracker mutation — same invariant as
    /// `invalidate_by_prefix` (SEC-NEW-002 closed).
    pub fn invalidate_by_client(&self, client_id: &str) -> Result<usize, PrismError> {
        let mut counts = self.lock_partition_counts()?;

        // Find all partitions for this client.
        let client_partitions: Vec<PartitionKey> = counts
            .keys()
            .filter(|(cid, _)| cid == client_id)
            .cloned()
            .collect();

        let mut evicted_entries: Vec<(CacheKey, usize)> = Vec::new();
        for pk in client_partitions {
            if let Some(partition_keys) = counts.remove(&pk) {
                evicted_entries.extend(partition_keys);
            }
        }

        let evicted_count = evicted_entries.len();

        // Invalidate moka entries and decrement total_bytes INSIDE the lock —
        // atomic with the tracker mutation (TD-PRISM-QUERY-CACHE-001 / SEC-NEW-002).
        for (k, stored_size) in &evicted_entries {
            self.inner.invalidate(k);
            // CR-014: decrement total_bytes by the stored size of each evicted entry.
            // SEC-NEW-001: saturating to prevent usize underflow on unexpected double-evict.
            let _ =
                self.total_bytes
                    .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
                        Some(current.saturating_sub(*stored_size))
                    });
        }

        // Release the lock before emitting the diagnostic.
        drop(counts);

        // O-2: emit diagnostic after lock release so operators can trace "where did my cache go".
        debug!(client_id, evicted_count, "cache client invalidation");

        Ok(evicted_count)
    }

    /// Remove ALL entries from the cache across every partition.
    ///
    /// Config hot-reload flush (P1-03 interim mitigation, review 2026-06-10):
    /// cached entries store the sensor response in its normalized form (see the
    /// human-authorized BC-2.07.003 deviation documented on [`CacheValue`]).
    /// Every config snapshot swap (BC-2.16.005) invokes this flush; the
    /// guarantee is eviction of all pre-swap-epoch entries. Two caveats bound
    /// what that buys today:
    ///
    /// 1. Fetch-path normalization is boot-frozen — `SpecDrivenSensorAdapter`
    ///    holds the boot-time resolved spec and a reload never rebuilds the
    ///    `AdapterRegistry` — so a reload does not change how fresh fetches
    ///    normalize. The flush is forward-provisioning; today's risk posture
    ///    is "the cache can never be staler than fresh fetches".
    /// 2. Under BC-2.16.006 in-flight Guard semantics, a query that started
    ///    before the swap completes under the OLD snapshot and may insert its
    ///    entry after this flush runs — swap-before-notify covers only
    ///    queries that start after the swap.
    ///
    /// Full reload-effectiveness arrives with S-CACHE-SPEC-COMPLIANCE-001's
    /// normalize-on-read model.
    ///
    /// Returns `Ok(n)` where `n` is the number of entries evicted (audit
    /// visibility, consistent with the other invalidation primitives).
    ///
    /// Returns `Err(PrismError::Internal)` if the mutex is poisoned (E-CACHE-001).
    ///
    /// ## Atomicity (TD-PRISM-QUERY-CACHE-001 pattern)
    ///
    /// The partition tracker drain, the per-key moka invalidations, and the
    /// `total_bytes` decrement all occur INSIDE the partition lock — atomic
    /// with respect to concurrent `put_with_ttl` / `remove_entry` /
    /// `invalidate_by_*` calls, which serialize on the same lock. A concurrent
    /// put either completes fully before the flush (and is evicted here) or
    /// starts after it (and inserts a fresh, correctly-tracked entry).
    pub fn invalidate_all(&self) -> Result<usize, PrismError> {
        let mut counts = self.lock_partition_counts()?;

        let mut evicted_count: usize = 0;
        let mut to_decrement: usize = 0;
        for (_, partition_keys) in counts.drain() {
            for (key, stored_size) in &partition_keys {
                self.inner.invalidate(key);
                to_decrement = to_decrement.saturating_add(*stored_size);
            }
            evicted_count = evicted_count.saturating_add(partition_keys.len());
        }

        // Decrement total_bytes once for the entire flushed batch, inside the
        // lock so accounting is serialized with concurrent puts.
        // SEC-NEW-001: saturating to prevent usize underflow.
        if to_decrement > 0 {
            let _ =
                self.total_bytes
                    .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
                        Some(current.saturating_sub(to_decrement))
                    });
        }

        // Release the lock before emitting the diagnostic.
        drop(counts);

        // O-2: emit diagnostic after lock release so operators can trace "where did my cache go".
        debug!(
            evicted_count,
            "cache full invalidation (config reload flush)"
        );

        Ok(evicted_count)
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

    /// Remove a single entry from both moka and the partition tracker.
    ///
    /// Crate-visible for the BC-2.07.003 forced-refresh invalidation path
    /// (`materialization::store_or_invalidate_response_cache`, P1-05 /
    /// architect adjudication D3): a forced refresh that cannot store a
    /// complete replacement removes the distrusted entry. Also used internally
    /// by `force_refresh` (replace) and `get` (lazy TTL expiry).
    ///
    /// Returns `Err(PrismError::Internal)` if the mutex is poisoned (E-CACHE-001).
    ///
    /// ## Atomicity (TD-PRISM-QUERY-CACHE-001 closure)
    ///
    /// The tracker removal, the moka invalidation, and the total_bytes decrement
    /// all occur INSIDE the partition lock. The previously documented residual
    /// race (pass-9 I9-003: T1.remove drops the lock, T2.put completes, T1's
    /// post-lock invalidate wipes T2's fresh entry → orphan tracker entry +
    /// overstated total_bytes) is structurally impossible: a concurrent
    /// put_with_ttl serializes on the same lock and either completes fully
    /// before this removal or starts fully after it (SEC-NEW-002 closed).
    pub(crate) fn remove_entry(&self, key: &CacheKey) -> Result<(), PrismError> {
        let pk = partition_key(key);
        let mut counts = self.lock_partition_counts()?;
        let mut dropped_size = 0usize;
        if let Some(partition_keys) = counts.get_mut(&pk) {
            // CR-014: look up the stored byte size before removing the tuple so
            // we can decrement total_bytes accurately.
            if let Some(pos) = partition_keys.iter().position(|(k, _)| k == key) {
                let (_, sz) = partition_keys.remove(pos);
                dropped_size = sz;
            }
            // CR-006: remove the partition entry if now empty to prevent unbounded
            // growth of the partition_counts map (TTL-expiry and force_refresh paths).
            if partition_keys.is_empty() {
                counts.remove(&pk);
            }
        }

        // Invalidate moka and decrement total_bytes INSIDE the lock — atomic
        // with the tracker mutation (TD-PRISM-QUERY-CACHE-001 / SEC-NEW-002).
        self.inner.invalidate(key);

        // SEC-NEW-001: saturating_sub prevents usize underflow on unexpected double-remove.
        if dropped_size > 0 {
            let _ =
                self.total_bytes
                    .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
                        Some(current.saturating_sub(dropped_size))
                    });
        }
        Ok(())
    }

    /// Insert directly into the moka cache without byte accounting or partition
    /// tracking. For use in regression tests that need to prime the cache with
    /// a specific entry bypassing `put_with_ttl` invariants (e.g., to test
    /// SEC-001 mutex-poison behaviour). (CR-016)
    #[cfg(test)]
    pub fn insert_raw_for_test(&self, key: CacheKey, entry: CacheEntry<V>) {
        self.inner.insert(key, entry);
    }

    /// Returns the number of partition keys currently tracked in `partition_counts`.
    /// A value of 0 means the map is fully clean — no stale empty-Vec entries remain.
    /// For use in regression tests only (CR-006).
    #[cfg(test)]
    pub fn partition_count_map_len(&self) -> usize {
        self.partition_counts.lock().map(|g| g.len()).unwrap_or(0)
    }

    /// Verify tracker ↔ moka ↔ byte-accounting consistency. Test-only.
    ///
    /// TD-PRISM-QUERY-CACHE-001 invariant: after any quiescent point,
    /// 1. every key tracked in `partition_counts` is present in moka
    ///    (no tracked-but-not-in-moka orphans), and
    /// 2. `total_bytes` equals the sum of all tracked entry sizes.
    ///
    /// Returns `Err(description)` on the first violation found.
    #[cfg(test)]
    pub fn check_consistency_for_test(&self) -> Result<(), String> {
        // Drain moka's write-op buffer so contains_key reflects all completed ops.
        self.inner.run_pending_tasks();
        let counts = self
            .partition_counts
            .lock()
            .map_err(|_| "partition_counts mutex poisoned".to_string())?;
        let mut tracked_bytes: usize = 0;
        for ((client, sensor), entries) in counts.iter() {
            for (key, size) in entries {
                tracked_bytes = tracked_bytes.saturating_add(*size);
                if !self.inner.contains_key(key) {
                    return Err(format!(
                        "orphan tracker entry: partition ({client}, {sensor}) tracks key \
                         (source_id={}, hash={}…) but moka has no such entry",
                        key.source_id,
                        &key.push_down_hash[..8]
                    ));
                }
            }
        }
        let total = self.total_bytes.load(Ordering::Relaxed);
        if total != tracked_bytes {
            return Err(format!(
                "byte-accounting drift: total_bytes={total} but sum of tracked sizes={tracked_bytes}"
            ));
        }
        Ok(())
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
            sensor_id: prism_core::SensorId::from("crowdstrike"),
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
            sensor_id: prism_core::SensorId::from("crowdstrike"),
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
            sensor_id: prism_core::SensorId::from("armis"),
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

    /// C-2 regression: `put` LRU eviction releases the partition lock before calling
    /// moka invalidate. Verifies that total_bytes accounting remains consistent
    /// (entry_count ≤ max, total_bytes does not underflow) after repeated evictions.
    /// SEC-NEW-001 regression guard.
    #[test]
    fn test_c2_put_lru_eviction_accounting_consistent_after_repeated_eviction() {
        let config = CacheConfig {
            max_entries_per_sensor: 3,
            max_bytes: DEFAULT_MAX_CACHE_BYTES,
        };
        let cache = QueryCache::new(config);

        let make_key = |n: u8| crate::cache_key::CacheKey {
            client_id: "acme".to_string(),
            sensor_id: prism_core::SensorId::from("crowdstrike"),
            source_id: "cs_devices".to_string(),
            push_down_hash: format!("{:0<64}", n),
        };

        // Fill to max_entries_per_sensor.
        for i in 1u8..=3 {
            cache
                .put(make_key(i), vec![serde_json::json!({"id": i})])
                .expect("put should not fail");
        }

        // Insert beyond capacity — each insert must evict the LRU entry without deadlock.
        for i in 4u8..=8 {
            cache
                .put(make_key(i), vec![serde_json::json!({"id": i})])
                .expect("put beyond capacity should not deadlock or fail");
        }

        // After all inserts the partition must remain within its bound.
        assert!(
            cache.entry_count() <= 3,
            "C-2: total_bytes accounting must stay consistent; partition bound must hold after repeated LRU evictions"
        );

        // total_bytes must not have wrapped to usize::MAX (SEC-NEW-001 regression).
        let bytes = cache.total_bytes();
        assert!(
            bytes < 1024 * 1024,
            "C-2: total_bytes must not overflow after LRU eviction (got {bytes})"
        );
    }

    /// C-2 concurrency: two threads racing on `put` must not deadlock when LRU
    /// eviction is triggered. Uses `std::thread::scope` to ensure the test
    /// terminates (Rust's scoped threads join implicitly on scope exit). Each
    /// thread does 50 inserts beyond the 3-entry partition cap, forcing eviction
    /// on every insert. If the lock-vs-moka split is wrong the threads will
    /// deadlock and the scope will hang indefinitely (test will time out in CI).
    #[test]
    fn test_c2_put_concurrent_lru_does_not_deadlock() {
        use std::sync::Arc;

        let config = CacheConfig {
            max_entries_per_sensor: 3,
            max_bytes: DEFAULT_MAX_CACHE_BYTES,
        };
        let cache = Arc::new(QueryCache::new(config));

        let make_key = |thread: u8, n: u8| crate::cache_key::CacheKey {
            client_id: format!("t{thread}"),
            sensor_id: prism_core::SensorId::from("crowdstrike"),
            source_id: "cs_devices".to_string(),
            push_down_hash: format!("{:0<64}", n),
        };

        std::thread::scope(|s| {
            let c1 = Arc::clone(&cache);
            let c2 = Arc::clone(&cache);

            let h1 = s.spawn(move || {
                for i in 0u8..50 {
                    c1.put(make_key(1, i), vec![serde_json::json!({"t": 1, "i": i})])
                        .expect("thread-1 put must not fail");
                }
            });

            let h2 = s.spawn(move || {
                for i in 0u8..50 {
                    c2.put(make_key(2, i), vec![serde_json::json!({"t": 2, "i": i})])
                        .expect("thread-2 put must not fail");
                }
            });

            // scope join is implicit, but surface panics explicitly
            h1.join().expect("thread-1 must not panic");
            h2.join().expect("thread-2 must not panic");
        });

        // Accounting invariants hold after concurrent evictions.
        assert!(
            cache.entry_count() <= 6, // 2 clients × 3 max per partition
            "C-2 concurrent: entry_count must respect partition bounds (got {})",
            cache.entry_count()
        );
        assert!(
            cache.total_bytes() < 1024 * 1024,
            "C-2 concurrent: total_bytes must not underflow after concurrent evictions (got {})",
            cache.total_bytes()
        );
    }

    /// BC-2.07.003: `force_refresh` overwrites an existing cache entry.
    #[test]
    fn test_force_refresh_overwrites_existing_entry() {
        let cache = QueryCache::with_defaults();
        let key = crate::cache_key::CacheKey {
            client_id: "acme".to_string(),
            sensor_id: prism_core::SensorId::from("crowdstrike"),
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

    /// SEC-001 / BC-2.07.004: poisoned mutex returns prose-only Internal detail.
    ///
    /// Regression test: a thread that panics while holding the partition_counts lock
    /// poisons the mutex. Subsequent operations must return a prose-only
    /// PrismError::Internal detail (no embedded E- code prefix) instead of
    /// silently recovering with potentially corrupted state.
    #[test]
    fn test_sec001_poisoned_mutex_returns_e_cache_001() {
        use std::sync::Arc;

        let cache = Arc::new(QueryCache::with_defaults());

        // Insert an entry first so a subsequent get() reaches the lock path.
        // (get() acquires the lock only on a cache hit to update LRU position.)
        let key = crate::cache_key::CacheKey {
            client_id: "acme".to_string(),
            sensor_id: prism_core::SensorId::from("crowdstrike"),
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
        // the lock without Internal-error propagation — the point of the test is to
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
        // Must return PrismError::Internal with prose-only detail (no E- prefix).
        let result = cache.get(&key);
        match result {
            Err(PrismError::Internal { detail }) => {
                assert!(
                    detail.contains("cache partition_counts mutex poisoned"),
                    "poisoned mutex must return prose-only Internal detail; got: {detail}"
                );
            }
            other => panic!(
                "SEC-001: poisoned mutex must return Err(PrismError::Internal) with prose detail; \
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
            sensor_id: prism_core::SensorId::from("crowdstrike"),
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
            sensor_id: prism_core::SensorId::from("crowdstrike"),
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
                sensor_id: prism_core::SensorId::from("crowdstrike"),
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

        // Invalidate the entire prefix — returns evicted count (I-2: BC-2.07.004 audit).
        let evicted = cache
            .invalidate_by_prefix("acme", "crowdstrike", "crowdstrike_detections")
            .expect("invalidate_by_prefix must not error");

        assert_eq!(
            evicted, n,
            "I-2/BC-2.07.004: invalidate_by_prefix must return the number of entries evicted"
        );

        let bytes_after_invalidate = cache.total_bytes();
        assert_eq!(
            bytes_after_invalidate, 0,
            "CR-014/CR-019: total_bytes must be exactly 0 after full prefix invalidation; \
             got {bytes_after_invalidate} — every byte added by the {n} entries must be \
             decremented by invalidate_by_prefix"
        );
    }

    /// I-2 / BC-2.07.004: `invalidate_by_client` returns the number of entries evicted.
    #[test]
    fn test_i2_invalidate_by_client_returns_evicted_count() {
        let config = CacheConfig {
            max_entries_per_sensor: 100,
            max_bytes: DEFAULT_MAX_CACHE_BYTES,
        };
        let cache = QueryCache::new(config);

        // Insert 3 entries under the same client across two sensors.
        for (sensor, source, hash_char) in &[
            ("crowdstrike", "crowdstrike_detections", 'a'),
            ("crowdstrike", "crowdstrike_hosts", 'b'),
            ("armis", "armis_devices", 'c'),
        ] {
            let key = crate::cache_key::CacheKey {
                client_id: "tenant-x".to_string(),
                sensor_id: prism_core::SensorId::from(*sensor),
                source_id: source.to_string(),
                push_down_hash: hash_char.to_string().repeat(64),
            };
            cache
                .put(key, vec![serde_json::json!({"id": "row-1"})])
                .expect("put must succeed");
        }
        // Insert 1 entry under a different client — must NOT be evicted.
        let other_key = crate::cache_key::CacheKey {
            client_id: "other-tenant".to_string(),
            sensor_id: prism_core::SensorId::from("armis"),
            source_id: "armis_devices".to_string(),
            push_down_hash: "d".repeat(64),
        };
        cache
            .put(other_key, vec![serde_json::json!({"id": "other"})])
            .expect("put other must succeed");

        let evicted = cache
            .invalidate_by_client("tenant-x")
            .expect("invalidate_by_client must not error");

        assert_eq!(
            evicted, 3,
            "I-2: invalidate_by_client must return 3 — the number of entries evicted for tenant-x"
        );
    }

    /// C10-001 regression: when LRU eviction frees sufficient budget for the
    /// incoming entry, the put must SUCCEED (not be silently rejected). Evicted
    /// entries must be removed from BOTH partition_keys AND moka (no orphans).
    /// total_bytes must reflect the post-eviction, post-insert state exactly.
    ///
    /// Before the fix, the budget check used `net_change = entry_size - existing_size`,
    /// ignoring evicted_bytes. When the cache was full:
    ///   (a) partition evicted k1 from partition_keys
    ///   (b) budget check saw current_bytes still at max → rejected k3
    ///   (c) k1 was orphaned in moka, total_bytes stayed inflated permanently.
    #[test]
    fn test_c10_001_budget_rejection_after_eviction_no_orphan_no_inflation() {
        let config = CacheConfig {
            max_entries_per_sensor: 2,
            max_bytes: 2 * AVG_ROW_SIZE_BYTES, // exactly 2 entries fit
        };
        let cache = QueryCache::new(config);

        let make_key = |src: &str| crate::cache_key::CacheKey {
            client_id: "acme".to_string(),
            sensor_id: prism_core::SensorId::from("crowdstrike"),
            source_id: src.to_string(),
            push_down_hash: src.repeat(64)[..64].to_string(),
        };

        let one_row = || vec![serde_json::json!({"id": "row"})]; // 1 × AVG_ROW_SIZE_BYTES

        let k1 = make_key("cs_devices_src1");
        let k2 = make_key("cs_devices_src2");
        let k3 = make_key("cs_devices_src3");

        // Fill cache to capacity: 2 entries × AVG_ROW_SIZE_BYTES = max_bytes.
        cache.put(k1.clone(), one_row()).expect("put k1");
        cache.put(k2.clone(), one_row()).expect("put k2");

        assert_eq!(
            cache.total_bytes(),
            2 * AVG_ROW_SIZE_BYTES,
            "pre-condition: cache at capacity"
        );

        // Put k3 — LRU eviction removes k1 (512 bytes freed), net_change = 512 - 0 - 512 = 0.
        // Post C10-001 fix: 0 > 0 is false → put succeeds.
        cache
            .put(k3.clone(), one_row())
            .expect("C10-001: k3 put must succeed after k1 eviction");

        // moka sync.
        cache.inner.run_pending_tasks();

        // k1 must be evicted from moka (not an orphan).
        assert!(
            cache.get(&k1).expect("get k1 must not error").is_none(),
            "C10-001: k1 must be evicted from moka (not orphaned) after LRU eviction"
        );
        // k2 must still be present.
        assert!(
            cache.get(&k2).expect("get k2 must not error").is_some(),
            "k2 must still be present"
        );
        // k3 must be present.
        assert!(
            cache.get(&k3).expect("get k3 must not error").is_some(),
            "C10-001: k3 must be present after successful put-with-eviction"
        );
        // total_bytes = 2 × AVG (k2 + k3), not inflated by orphan.
        assert_eq!(
            cache.total_bytes(),
            2 * AVG_ROW_SIZE_BYTES,
            "C10-001: total_bytes must reflect actual 2-entry cache (k2 + k3), not inflated by orphan k1"
        );
    }

    /// TD-PRISM-QUERY-CACHE-001 regression: concurrent evict-vs-put must leave the
    /// cache in a fully consistent state — no tracked-but-not-in-moka orphans and
    /// no byte-accounting drift.
    ///
    /// Pre-fix failure mode (SEC-NEW-002): `invalidate_by_prefix` / `remove_entry` /
    /// `put_with_ttl` performed their moka invalidations and `total_bytes`
    /// decrements AFTER releasing the partition lock. A concurrent `put` on an
    /// evicted key could interleave between the lock release and the post-lock
    /// `inner.invalidate`, wiping the fresh moka entry while the tracker still
    /// recorded it (orphan) and leaving `total_bytes` overstated.
    ///
    /// This test hammers the exact interleaving: writer threads `put` a small set
    /// of keys in a tight loop while invalidator threads concurrently
    /// `invalidate_by_prefix` the same prefix and `force_refresh` the same keys.
    /// After all threads join, `check_consistency_for_test()` must report a fully
    /// consistent tracker ↔ moka ↔ byte-accounting state.
    #[test]
    fn test_td_cache_001_concurrent_evict_vs_put_consistency() {
        use std::sync::Arc;

        let config = CacheConfig {
            max_entries_per_sensor: 4, // small bound → constant LRU eviction pressure
            max_bytes: DEFAULT_MAX_CACHE_BYTES,
        };
        let cache = Arc::new(QueryCache::new(config));

        let make_key = |n: u8| crate::cache_key::CacheKey {
            client_id: "acme".to_string(),
            sensor_id: prism_core::SensorId::from("crowdstrike"),
            source_id: "cs_devices".to_string(),
            push_down_hash: format!("{:0<64}", n),
        };

        std::thread::scope(|s| {
            // Two writer threads: tight put loops over an overlapping key set
            // (8 keys > 4-entry bound → every put evicts).
            for t in 0..2u8 {
                let c = Arc::clone(&cache);
                s.spawn(move || {
                    for i in 0..200u32 {
                        let key = make_key((i % 8) as u8);
                        c.put(key, vec![serde_json::json!({"t": t, "i": i})])
                            .expect("writer put must not fail");
                    }
                });
            }
            // One force-refresh thread: remove+put composition on the same keys.
            {
                let c = Arc::clone(&cache);
                s.spawn(move || {
                    for i in 0..200u32 {
                        let key = make_key((i % 8) as u8);
                        c.force_refresh(key, vec![serde_json::json!({"fr": i})])
                            .expect("force_refresh must not fail");
                    }
                });
            }
            // One invalidator thread: prefix invalidation racing the puts.
            {
                let c = Arc::clone(&cache);
                s.spawn(move || {
                    for _ in 0..200u32 {
                        c.invalidate_by_prefix("acme", "crowdstrike", "cs_devices")
                            .expect("invalidate_by_prefix must not fail");
                    }
                });
            }
        });

        // Post-quiescence: tracker ↔ moka ↔ byte accounting must be consistent.
        cache.check_consistency_for_test().expect(
            "TD-PRISM-QUERY-CACHE-001: cache must be consistent after concurrent evict-vs-put",
        );

        // Partition bound must still hold.
        assert!(
            cache.entry_count() <= 4,
            "partition bound must hold after concurrent churn (got {})",
            cache.entry_count()
        );
    }

    /// C10-001b regression: when the incoming entry is larger than the bytes freed
    /// by LRU eviction, the put must be REJECTED and the evicted entries must be
    /// restored to partition_keys (no orphan moka entries, no inflated total_bytes).
    ///
    /// Scenario: max_bytes = 2 × AVG, cache holds k1 + k2. Inserting k3 with
    /// 3 rows (3 × AVG) triggers eviction of k1 (frees AVG), but
    /// net_change = 3*AVG - 0 - AVG = 2*AVG > 0 and current_bytes + 2*AVG > max_bytes.
    /// k3 must be rejected, k1 must be restored, state must be unchanged.
    #[test]
    fn test_c10_001b_oversized_entry_rejected_no_orphan_after_restore() {
        let config = CacheConfig {
            max_entries_per_sensor: 2,
            max_bytes: 2 * AVG_ROW_SIZE_BYTES, // exactly 2 single-row entries fit
        };
        let cache = QueryCache::new(config);

        let make_key = |src: &str| crate::cache_key::CacheKey {
            client_id: "acme".to_string(),
            sensor_id: prism_core::SensorId::from("crowdstrike"),
            source_id: src.to_string(),
            push_down_hash: src.repeat(64)[..64].to_string(),
        };

        let one_row = || vec![serde_json::json!({"id": "row"})]; // 1 × AVG_ROW_SIZE_BYTES

        let k1 = make_key("cs_devices_src1");
        let k2 = make_key("cs_devices_src2");
        let k3 = make_key("cs_devices_src3");

        // Fill cache to capacity.
        cache.put(k1.clone(), one_row()).expect("put k1");
        cache.put(k2.clone(), one_row()).expect("put k2");

        assert_eq!(
            cache.total_bytes(),
            2 * AVG_ROW_SIZE_BYTES,
            "pre-condition: cache at capacity"
        );

        // Try to insert k3 with 3 rows — oversized even after evicting k1.
        // net_change = 3*AVG - 0 - 1*AVG = 2*AVG; current_bytes (2*AVG) + 2*AVG > max_bytes (2*AVG).
        let three_rows = vec![
            serde_json::json!({"id": "r1"}),
            serde_json::json!({"id": "r2"}),
            serde_json::json!({"id": "r3"}),
        ];
        cache
            .put(k3.clone(), three_rows)
            .expect("put k3 must not error (silent drop)");

        // moka sync.
        cache.inner.run_pending_tasks();

        // k3 must be absent (rejected).
        assert!(
            cache.get(&k3).expect("get k3 must not error").is_none(),
            "C10-001b: k3 must be rejected (oversized after eviction)"
        );
        // k1 must be RESTORED (not orphaned in moka with missing partition entry).
        assert!(
            cache.get(&k1).expect("get k1 must not error").is_some(),
            "C10-001b: k1 must be restored to partition after rejection (eviction undone)"
        );
        // k2 must be present.
        assert!(
            cache.get(&k2).expect("get k2 must not error").is_some(),
            "k2 must still be present"
        );
        // total_bytes must be unchanged (2 entries still).
        assert_eq!(
            cache.total_bytes(),
            2 * AVG_ROW_SIZE_BYTES,
            "C10-001b: total_bytes must be unchanged after oversized-entry rejection"
        );
    }

    // -----------------------------------------------------------------------
    // P1-03 interim mitigation — full-cache flush on config hot-reload
    // (BC-2.16.005 reload → response-cache flush; review 2026-06-10 Item 1)
    // -----------------------------------------------------------------------

    /// P1-03: `invalidate_all` flushes every entry across all partitions —
    /// entries that hit before the flush must miss after it.
    #[test]
    fn test_p1_03_invalidate_all_flushes_all_entries_hit_then_miss() {
        let cache = QueryCache::with_defaults();

        let make_key =
            |client: &str, sensor: &str, src: &str, h: char| crate::cache_key::CacheKey {
                client_id: client.to_string(),
                sensor_id: prism_core::SensorId::from(sensor),
                source_id: src.to_string(),
                push_down_hash: h.to_string().repeat(64),
            };

        // Multiple partitions: 2 clients × 3 sensors.
        let keys = vec![
            make_key("acme", "crowdstrike", "cs_detections", 'a'),
            make_key("acme", "armis", "armis_devices", 'b'),
            make_key("globex", "crowdstrike", "cs_devices", 'c'),
            make_key("globex", "claroty", "claroty_alerts", 'd'),
        ];
        for (i, key) in keys.iter().enumerate() {
            cache
                .put(key.clone(), vec![serde_json::json!({"id": i})])
                .expect("put must succeed");
        }

        // Pre-condition: all entries hit.
        for key in &keys {
            assert!(
                cache.get(key).expect("get must not fail").is_some(),
                "pre-condition: entry must hit before invalidate_all"
            );
        }

        let evicted = cache.invalidate_all().expect("invalidate_all must succeed");
        assert_eq!(
            evicted, 4,
            "P1-03: invalidate_all must report all 4 evicted entries"
        );

        // Post-condition: every entry misses.
        for key in &keys {
            assert!(
                cache.get(key).expect("get must not fail").is_none(),
                "P1-03: entry must miss after invalidate_all (stale normalization purged)"
            );
        }
    }

    /// P1-03 / TD-PRISM-QUERY-CACHE-001: accounting must be consistent after a
    /// full flush — total_bytes returns to 0, the partition map is empty, and
    /// the tracker ↔ moka ↔ byte-accounting consistency invariant holds.
    #[test]
    fn test_p1_03_invalidate_all_accounting_consistent_after_flush() {
        let cache = QueryCache::with_defaults();

        let make_key = |src: &str, h: char| crate::cache_key::CacheKey {
            client_id: "acme".to_string(),
            sensor_id: prism_core::SensorId::from("crowdstrike"),
            source_id: src.to_string(),
            push_down_hash: h.to_string().repeat(64),
        };

        cache
            .put(
                make_key("cs_devices", 'a'),
                vec![serde_json::json!({"i": 1})],
            )
            .expect("put 1");
        cache
            .put(
                make_key("cs_detections", 'b'),
                vec![serde_json::json!({"i": 2}), serde_json::json!({"i": 3})],
            )
            .expect("put 2");
        assert!(cache.total_bytes() > 0, "pre-condition: bytes tracked");

        cache.invalidate_all().expect("invalidate_all must succeed");

        assert_eq!(
            cache.total_bytes(),
            0,
            "P1-03: total_bytes must return to 0 after full flush"
        );
        assert_eq!(
            cache.partition_count_map_len(),
            0,
            "P1-03: partition map must be empty after full flush (no stale partitions)"
        );
        assert_eq!(
            cache.entry_count(),
            0,
            "P1-03: moka must hold no entries after full flush"
        );
        cache
            .check_consistency_for_test()
            .expect("P1-03: tracker/moka/byte accounting must be consistent after flush");
    }

    /// P1-03: `invalidate_all` on an empty cache is a no-op returning Ok(0).
    #[test]
    fn test_p1_03_invalidate_all_empty_cache_returns_zero() {
        let cache = QueryCache::with_defaults();
        let evicted = cache.invalidate_all().expect("invalidate_all must succeed");
        assert_eq!(evicted, 0, "P1-03: empty cache flush must evict 0 entries");
        assert_eq!(cache.total_bytes(), 0);
    }

    /// P1-03: the cache remains fully usable after `invalidate_all` —
    /// subsequent puts and gets behave normally with consistent accounting.
    #[test]
    fn test_p1_03_invalidate_all_cache_usable_after_flush() {
        let cache = QueryCache::with_defaults();
        let key = crate::cache_key::CacheKey {
            client_id: "acme".to_string(),
            sensor_id: prism_core::SensorId::from("crowdstrike"),
            source_id: "cs_devices".to_string(),
            push_down_hash: "f".repeat(64),
        };

        cache
            .put(key.clone(), vec![serde_json::json!({"gen": 1})])
            .expect("put gen-1");
        cache.invalidate_all().expect("invalidate_all must succeed");

        let fresh = vec![serde_json::json!({"gen": 2})];
        cache.put(key.clone(), fresh.clone()).expect("put gen-2");
        assert_eq!(
            cache.get(&key).expect("get must not fail"),
            Some(fresh),
            "P1-03: cache must serve fresh entries inserted after a full flush"
        );
        cache
            .check_consistency_for_test()
            .expect("P1-03: accounting must be consistent after post-flush insert");
    }
}
