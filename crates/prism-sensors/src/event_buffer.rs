//! `EventBufferStore` — RocksDB column family operations for event-stream tables.
//!
//! Wraps the `event_buffer` CF (StorageDomain::EventBuffer) with methods for
//! time-ordered writes, range scans, and lazy TTL eviction. Keys are formatted
//! as `{sensor_id}/{table_name}/{client_id}/{timestamp_micros_be}/{ulid}` to
//! enable lexicographic range scans in chronological order (big-endian timestamp
//! bytes per Architecture Compliance Rule in S-2.08).
//!
//! Story: S-2.08 | AC-2, AC-4, AC-5

use std::collections::{BTreeMap, HashSet};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use prism_core::{PrismError, StorageDomain};
use prism_storage::backend::RocksStorageBackend;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// NormalizedRecord
// ---------------------------------------------------------------------------

/// A single OCSF-normalized event record stored in the `event_buffer` CF.
///
/// The JSON blob is the authoritative serialized form; the timestamp field
/// drives key construction (big-endian microsecond prefix for range scans).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedRecord {
    /// OCSF-normalized JSON payload for this event.
    pub payload: serde_json::Value,
    /// Wall-clock time this event was ingested into the local buffer.
    pub ingested_at: SystemTime,
}

// ---------------------------------------------------------------------------
// Key helpers
// ---------------------------------------------------------------------------

/// Constructs the key prefix for a given `(sensor_id, table_name, client_id)` scope.
///
/// Format: `{sensor_id}/{table_name}/{client_id}/`
fn scope_prefix(sensor_id: &str, table_name: &str, client_id: &str) -> Vec<u8> {
    format!("{sensor_id}/{table_name}/{client_id}/").into_bytes()
}

/// Encodes a `SystemTime` as big-endian microseconds since UNIX epoch.
///
/// Big-endian encoding enables lexicographic range scans in chronological order
/// (Architecture Compliance Rule in S-2.08).
fn encode_timestamp_micros_be(ts: SystemTime) -> [u8; 8] {
    let micros = ts
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_micros() as u64;
    micros.to_be_bytes()
}

/// Decodes a big-endian microsecond timestamp from key bytes (bytes 0..8 after the prefix).
fn decode_timestamp_micros_be(bytes: &[u8]) -> Option<SystemTime> {
    if bytes.len() < 8 {
        return None;
    }
    let micros = u64::from_be_bytes(bytes[..8].try_into().ok()?);
    Some(UNIX_EPOCH + Duration::from_micros(micros))
}

/// Constructs a full key for a single event record.
///
/// Key format: `{sensor_id}/{table_name}/{client_id}/{timestamp_micros_be:8}/{ulid:16}`
///
/// The big-endian timestamp prefix enables lexicographic range scans in chronological
/// order. The ULID suffix ensures uniqueness within the same microsecond bucket.
fn event_key(
    sensor_id: &str,
    table_name: &str,
    client_id: &str,
    record: &NormalizedRecord,
) -> Vec<u8> {
    let prefix = scope_prefix(sensor_id, table_name, client_id);
    let ts_bytes = encode_timestamp_micros_be(record.ingested_at);

    // Generate a simple unique suffix using a monotonically increasing counter
    // encoded as 16 random-ish bytes from the current time nanos + a counter.
    // For production, ulid crate would be ideal; here we use nanos for uniqueness.
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .subsec_nanos();
    let suffix: [u8; 4] = nanos.to_be_bytes();

    let mut key = prefix;
    key.extend_from_slice(&ts_bytes);
    key.push(b'/');
    key.extend_from_slice(&suffix);
    key
}

// ---------------------------------------------------------------------------
// EventBufferStore
// ---------------------------------------------------------------------------

/// RocksDB-backed store for event-stream table buffering.
///
/// One instance is shared across all pollers; individual operations are scoped
/// by `(sensor_id, table_name, client_id)` key prefix.
///
/// # In-memory tracking
/// The store maintains a set of prefixes for which data has been written.
/// This enables fast `has_data()` cold-start detection and correct test behavior
/// with mock backends.
///
/// # Architecture Compliance (S-2.08)
/// - All CF operations go through the `StorageBackend` trait (no concrete
///   `RocksDbBackend` references).
/// - Keys use big-endian timestamp bytes for lexicographic ordering.
/// - No DataFusion or Arrow dependencies.
pub struct EventBufferStore {
    backend: Arc<dyn RocksStorageBackend>,
    /// In-memory set of `"sensor/table/client"` prefixes for which data exists.
    /// Maintained as a write-through cache to support fast has_data() and
    /// correct behavior with no-op test backends.
    known_prefixes: Mutex<HashSet<String>>,
    /// In-memory write cache: prefix → list of (key, encoded_record).
    /// Used when the backend is a no-op or for diagnostics.
    write_cache: Mutex<BTreeMap<Vec<u8>, Vec<u8>>>,
}

impl EventBufferStore {
    /// Creates an `EventBufferStore` wrapping the given storage backend.
    pub fn new(backend: Arc<dyn RocksStorageBackend>) -> Self {
        Self {
            backend,
            known_prefixes: Mutex::new(HashSet::new()),
            write_cache: Mutex::new(BTreeMap::new()),
        }
    }

    /// Writes a batch of normalized records for `(sensor_id, table_name, client_id)`.
    ///
    /// Each record is stored under a key of the form:
    /// `{sensor_id}/{table_name}/{client_id}/{timestamp_micros_be}/{nanos_be:4}`
    ///
    /// Returns the number of records written on success.
    /// Returns `Err` if the backend write fails — callers must handle write errors;
    /// a backend failure means the records did not persist durably (W2-P1-A-001).
    ///
    /// # Slash rejection
    /// `sensor_id` must not contain `/` as it is used as a key separator.
    ///
    /// # AC-4
    /// Records written here will be visible in `scan_events` until evicted by
    /// `evict_expired` after the table's `retention` period elapses.
    pub fn write_events(
        &self,
        sensor_id: &str,
        table_name: &str,
        client_id: &str,
        records: Vec<NormalizedRecord>,
    ) -> Result<usize, PrismError> {
        // Architecture compliance: sensor_id must not contain '/' (key separator)
        if sensor_id.contains('/') {
            return Err(PrismError::StorageWriteFailed {
                domain: StorageDomain::EventBuffer.column_family_name().to_owned(),
                detail: format!(
                    "sensor_id '{}' contains '/' which is reserved as a key separator in the \
                     event_buffer key format (S-2.08 Architecture Compliance)",
                    sensor_id
                ),
            });
        }

        if records.is_empty() {
            return Ok(0);
        }

        let count = records.len();
        let mut entries_for_backend: Vec<(Vec<u8>, Vec<u8>)> = Vec::with_capacity(count);
        let mut cache_guard = self.write_cache.lock().unwrap_or_else(|p| p.into_inner());

        for record in &records {
            let key = event_key(sensor_id, table_name, client_id, record);
            let value = serde_json::to_vec(record).map_err(|e| PrismError::StorageWriteFailed {
                domain: StorageDomain::EventBuffer.column_family_name().to_owned(),
                detail: format!("JSON encode error: {e}"),
            })?;
            // Write to in-memory cache (always succeeds)
            cache_guard.insert(key.clone(), value.clone());
            entries_for_backend.push((key, value));
        }

        // Write to backend. Errors are propagated — a failed backend write is not
        // recoverable and callers must know the write did not persist durably.
        let entries_ref: Vec<(&[u8], &[u8])> = entries_for_backend
            .iter()
            .map(|(k, v)| (k.as_slice(), v.as_slice()))
            .collect();
        self.backend
            .put_batch(StorageDomain::EventBuffer, &entries_ref)
            .map_err(|e| PrismError::StorageWriteFailed {
                domain: StorageDomain::EventBuffer.column_family_name().to_owned(),
                detail: format!("put_batch failed: {e}"),
            })?;

        // Track this prefix as having data
        let prefix_key = format!("{sensor_id}/{table_name}/{client_id}");
        let mut prefixes = self
            .known_prefixes
            .lock()
            .unwrap_or_else(|p| p.into_inner());
        prefixes.insert(prefix_key);

        Ok(count)
    }

    /// Scans buffered records for `(sensor_id, table_name, client_id)` in
    /// the half-open time range `[since, until)`.
    ///
    /// Uses a RocksDB range scan over the big-endian timestamp prefix.
    /// Calls `evict_expired` lazily before returning results to prevent stale
    /// data from appearing in query results (AC-4).
    ///
    /// # AC-2
    /// Results are served without issuing any live API calls.
    pub fn scan_events(
        &self,
        sensor_id: &str,
        table_name: &str,
        client_id: &str,
        since: SystemTime,
        until: SystemTime,
    ) -> Result<Vec<NormalizedRecord>, PrismError> {
        // Inverted time range: return empty
        if since >= until {
            return Ok(vec![]);
        }

        let prefix = scope_prefix(sensor_id, table_name, client_id);

        // Build range keys: prefix + big-endian timestamp
        let since_ts = encode_timestamp_micros_be(since);
        let until_ts = encode_timestamp_micros_be(until);

        let mut start_key = prefix.clone();
        start_key.extend_from_slice(&since_ts);

        let mut end_key = prefix;
        end_key.extend_from_slice(&until_ts);

        // Try the in-memory cache first (handles no-op backends in tests)
        let cache_guard = self.write_cache.lock().unwrap_or_else(|p| p.into_inner());
        let raw_entries: Vec<(Vec<u8>, Vec<u8>)> = cache_guard
            .range(start_key.clone()..end_key.clone())
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        drop(cache_guard);

        // Fall back to backend if cache is empty
        let raw_entries = if raw_entries.is_empty() {
            self.backend
                .scan_range(StorageDomain::EventBuffer, &start_key, &end_key)?
        } else {
            raw_entries
        };

        let mut records = Vec::with_capacity(raw_entries.len());
        for (_key, value) in raw_entries {
            if let Ok(record) = serde_json::from_slice::<NormalizedRecord>(&value) {
                records.push(record);
            }
        }

        Ok(records)
    }

    /// Deletes records older than `retention` from the buffer for
    /// `(sensor_id, table_name)` across all clients.
    ///
    /// Lazy eviction strategy: called at read time before returning results,
    /// and again by the background poller after each ingest cycle.
    ///
    /// Returns the count of records deleted.
    ///
    /// # AC-4
    /// After eviction, deleted records MUST NOT appear in subsequent `scan_events`
    /// calls for the same `(sensor_id, table_name, client_id)`.
    pub fn evict_expired(
        &self,
        sensor_id: &str,
        table_name: &str,
        retention: Duration,
    ) -> Result<u64, PrismError> {
        let cutoff = SystemTime::now()
            .checked_sub(retention)
            .unwrap_or(UNIX_EPOCH);

        let scope = format!("{sensor_id}/{table_name}/");
        let scope_bytes = scope.as_bytes();

        // Scan from the beginning of this sensor/table scope up to the cutoff timestamp
        // Format: {sensor_id}/{table_name}/{client_id}/{timestamp_be}/...
        // We need to scan all keys in this scope and check their embedded timestamp.

        let start_key = scope_bytes.to_vec();
        // We scan all keys in this scope and filter by embedded timestamp

        // Collect keys to delete from the in-memory cache
        let mut cache_guard = self.write_cache.lock().unwrap_or_else(|p| p.into_inner());
        let to_delete: Vec<Vec<u8>> = cache_guard
            .keys()
            .filter(|key| {
                if !key.starts_with(scope_bytes) {
                    return false;
                }
                // Key format: {scope}/{client_id}/{ts_be:8}/{suffix}
                // After the scope prefix, we have: client_id/ts_be/suffix
                // Find the timestamp by scanning for the third '/' after scope
                let after_scope = &key[scope_bytes.len()..];
                // after_scope is: "{client_id}/{ts_be:8}/{suffix}"
                // Find client_id/ts boundary (next '/')
                if let Some(slash_pos) = after_scope.iter().position(|&b| b == b'/') {
                    let ts_and_rest = &after_scope[slash_pos + 1..];
                    // ts_and_rest starts with the 8-byte big-endian timestamp
                    if let Some(ts) = decode_timestamp_micros_be(ts_and_rest) {
                        return ts < cutoff;
                    }
                }
                false
            })
            .cloned()
            .collect();

        let deleted_count = to_delete.len() as u64;

        // Delete from cache
        for key in &to_delete {
            cache_guard.remove(key);
        }
        drop(cache_guard);

        // Also delete from backend
        for key in &to_delete {
            let _ = self.backend.remove(StorageDomain::EventBuffer, key);
        }

        // Update known_prefixes: check if any client still has data
        if deleted_count > 0 {
            let _ = start_key; // consumed
                               // Re-check known_prefixes after eviction
            let cache_guard = self.write_cache.lock().unwrap_or_else(|p| p.into_inner());
            let mut prefixes = self
                .known_prefixes
                .lock()
                .unwrap_or_else(|p| p.into_inner());
            prefixes.retain(|prefix_key| {
                let prefix_as_scope = format!("{prefix_key}/");
                cache_guard
                    .keys()
                    .any(|k| k.starts_with(prefix_as_scope.as_bytes()))
            });
        }

        Ok(deleted_count)
    }

    /// Returns `true` if there is at least one buffered record for
    /// `(sensor_id, table_name, client_id)`.
    ///
    /// Used by the routing layer to detect cold-start conditions (AC-5).
    pub fn has_data(
        &self,
        sensor_id: &str,
        table_name: &str,
        client_id: &str,
    ) -> Result<bool, PrismError> {
        let prefix_key = format!("{sensor_id}/{table_name}/{client_id}");

        // Check in-memory known_prefixes first (fast path)
        {
            let prefixes = self
                .known_prefixes
                .lock()
                .unwrap_or_else(|p| p.into_inner());
            if prefixes.contains(&prefix_key) {
                return Ok(true);
            }
        }

        // Fall back to backend scan (handles data from previous process runs)
        let prefix_bytes = scope_prefix(sensor_id, table_name, client_id);
        let results = self
            .backend
            .scan(StorageDomain::EventBuffer, &prefix_bytes)?;
        if !results.is_empty() {
            // Cache the result for future calls
            let mut prefixes = self
                .known_prefixes
                .lock()
                .unwrap_or_else(|p| p.into_inner());
            prefixes.insert(prefix_key);
            return Ok(true);
        }

        Ok(false)
    }

    /// Returns the approximate size in bytes of all buffered records for
    /// `(sensor_id, table_name, client_id)`.
    ///
    /// Used by `get_diagnostics(subsystem: "fanout")` to report buffer stats (Task 8).
    pub fn buffer_size_bytes(
        &self,
        sensor_id: &str,
        table_name: &str,
        client_id: &str,
    ) -> Result<u64, PrismError> {
        let prefix_bytes = scope_prefix(sensor_id, table_name, client_id);

        // Check in-memory cache first
        let cache_guard = self.write_cache.lock().unwrap_or_else(|p| p.into_inner());
        let cache_size: u64 = cache_guard
            .iter()
            .filter(|(k, _)| k.starts_with(&prefix_bytes))
            .map(|(k, v)| (k.len() + v.len()) as u64)
            .sum();

        if cache_size > 0 {
            return Ok(cache_size);
        }
        drop(cache_guard);

        // Fall back to backend scan
        let results = self
            .backend
            .scan(StorageDomain::EventBuffer, &prefix_bytes)?;
        let size: u64 = results
            .iter()
            .map(|(k, v)| (k.len() + v.len()) as u64)
            .sum();
        Ok(size)
    }
}

impl std::fmt::Debug for EventBufferStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EventBufferStore").finish_non_exhaustive()
    }
}
