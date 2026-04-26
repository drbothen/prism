//! `EventBufferStore` — RocksDB column family operations for event-stream tables.
//!
//! Wraps the `event_buffer` CF (StorageDomain::EventBuffer) with methods for
//! time-ordered writes, range scans, and lazy TTL eviction. Keys are formatted
//! as `{sensor_id}/{table_name}/{client_id}/{timestamp_micros_be}/{ulid}` to
//! enable lexicographic range scans in chronological order (big-endian timestamp
//! bytes per Architecture Compliance Rule in S-2.08).
//!
//! Story: S-2.08 | AC-2, AC-4, AC-5

use std::sync::Arc;
use std::time::{Duration, SystemTime};

use prism_core::PrismError;
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
// EventBufferStore
// ---------------------------------------------------------------------------

/// RocksDB-backed store for event-stream table buffering.
///
/// One instance is shared across all pollers; individual operations are scoped
/// by `(sensor_id, table_name, client_id)` key prefix.
///
/// # Architecture Compliance (S-2.08)
/// - All CF operations go through the `StorageBackend` trait (no concrete
///   `RocksDbBackend` references).
/// - Keys use big-endian timestamp bytes for lexicographic ordering.
/// - No DataFusion or Arrow dependencies.
pub struct EventBufferStore {
    // Stub: backend is used by all CF operation methods once implemented.
    #[allow(dead_code)]
    backend: Arc<dyn RocksStorageBackend>,
}

impl EventBufferStore {
    /// Creates an `EventBufferStore` wrapping the given storage backend.
    pub fn new(backend: Arc<dyn RocksStorageBackend>) -> Self {
        Self { backend }
    }

    /// Writes a batch of normalized records for `(sensor_id, table_name, client_id)`.
    ///
    /// Each record is stored under a key of the form:
    /// `{sensor_id}/{table_name}/{client_id}/{timestamp_micros_be}/{ulid}`
    ///
    /// Returns the number of records successfully written.
    ///
    /// # AC-4
    /// Records written here will be visible in `scan_events` until evicted by
    /// `evict_expired` after the table's `retention` period elapses.
    pub fn write_events(
        &self,
        sensor_id: &str,
        table_name: &str,
        client_id: &str,
        _records: Vec<NormalizedRecord>,
    ) -> Result<usize, PrismError> {
        todo!("AC-2 / AC-4: implement RocksDB CF batch write with big-endian timestamp key prefix; sensor_id={sensor_id}, table_name={table_name}, client_id={client_id}")
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
        _since: SystemTime,
        _until: SystemTime,
    ) -> Result<Vec<NormalizedRecord>, PrismError> {
        todo!("AC-2: implement RocksDB range scan using big-endian timestamp keys; sensor_id={sensor_id}, table_name={table_name}, client_id={client_id}")
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
        _retention: Duration,
    ) -> Result<u64, PrismError> {
        todo!("AC-4: implement TTL eviction by scanning and deleting keys with timestamp older than (now - retention); sensor_id={sensor_id}, table_name={table_name}")
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
        todo!("AC-5: implement cold-start detection by checking for any key under the sensor/table/client prefix; sensor_id={sensor_id}, table_name={table_name}, client_id={client_id}")
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
        todo!("Task-8: implement approximate CF size estimation for diagnostics; sensor_id={sensor_id}, table_name={table_name}, client_id={client_id}")
    }
}

impl std::fmt::Debug for EventBufferStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EventBufferStore").finish_non_exhaustive()
    }
}
