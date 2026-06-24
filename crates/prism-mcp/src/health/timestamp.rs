//! Last-successful-query timestamp tracking (BC-2.08.004, S-5.04).
//!
//! Maintains a `HashMap<(client_id, sensor_id), DateTime<Utc>>` in `PrismContext`
//! as a write-through cache backed by RocksDB for durable persistence.
//!
//! # Persistence contract (BC-2.08.004 postcondition 2 — F-S504-P1-005)
//! - `write_timestamp`: writes to the in-memory map AND to RocksDB when
//!   `context.storage` is `Some`. Persisted value survives server restarts.
//! - `read_timestamp`: checks the in-memory map first (fast path). On cache miss,
//!   reads from RocksDB (cold path after restart) and repopulates the in-memory map.
//!
//! # Key format
//! RocksDB key: `b"health_ts/{client_id}/{sensor_id}"`
//! Value: RFC-3339 encoded `DateTime<Utc>` as UTF-8 bytes.
//!
//! # StorageDomain
//! Uses `StorageDomain::Default` — the general-purpose column family (prism-core §storage.rs).

use chrono::{DateTime, Utc};
use prism_core::StorageDomain;

/// RocksDB key prefix for health timestamps (BC-2.08.004 postcondition 2).
pub const HEALTH_TS_KEY_PREFIX: &[u8] = b"health_ts/";

/// Serialize a timestamp key for RocksDB.
///
/// Format: `health_ts/{client_id}/{sensor_id}` (UTF-8 bytes).
pub fn timestamp_key(client_id: &str, sensor_id: &str) -> Vec<u8> {
    format!("health_ts/{client_id}/{sensor_id}").into_bytes()
}

/// Read the last-successful-query timestamp from `PrismContext`.
///
/// Fast path: returns the in-memory cached value if present.
/// Cold path (after restart): on cache miss, reads from RocksDB (when `context.storage`
/// is `Some`), parses the RFC-3339 value, repopulates the in-memory map, and returns it.
///
/// Returns `None` if no timestamp has been recorded for this (client_id, sensor_id).
pub fn read_timestamp(
    client_id: &str,
    sensor_id: &str,
    context: &crate::context::PrismContext,
) -> Option<DateTime<Utc>> {
    use crate::context::SensorKey;

    let key = SensorKey {
        client_id: client_id.to_owned(),
        sensor_id: sensor_id.to_owned(),
    };

    // Fast path: check in-memory map first.
    {
        let guard = match context.last_query_timestamps.lock() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };
        if let Some(ts) = guard.get(&key).copied() {
            return Some(ts);
        }
    }

    // Cold path (after restart): read from RocksDB and repopulate in-memory cache.
    if let Some(storage) = &context.storage {
        let raw_key = timestamp_key(client_id, sensor_id);
        if let Ok(Some(bytes)) = storage.get(StorageDomain::Default, &raw_key) {
            if let Ok(rfc3339) = std::str::from_utf8(&bytes) {
                if let Ok(ts) = DateTime::parse_from_rfc3339(rfc3339) {
                    let ts_utc: DateTime<Utc> = ts.with_timezone(&Utc);
                    // Repopulate in-memory map to avoid repeated RocksDB reads.
                    let mut guard = match context.last_query_timestamps.lock() {
                        Ok(g) => g,
                        Err(p) => p.into_inner(),
                    };
                    guard.insert(key, ts_utc);
                    return Some(ts_utc);
                }
            }
        }
    }

    None
}

/// Write the last-successful-query timestamp to `PrismContext` and, if wired,
/// to RocksDB for durable persistence (BC-2.08.004 postcondition 1 + postcondition 2).
///
/// Called by `SensorHealthChecker::record_successful_query` after every successful
/// sensor probe or query.
///
/// Writes the RFC-3339 encoded timestamp to `StorageDomain::Default` under key
/// `health_ts/{client_id}/{sensor_id}` when `context.storage` is `Some`, so that
/// `read_timestamp` can recover the value after a server restart (F-S504-P1-005).
pub fn write_timestamp(
    client_id: &str,
    sensor_id: &str,
    at: DateTime<Utc>,
    context: &crate::context::PrismContext,
) {
    use crate::context::SensorKey;

    let sensor_key = SensorKey {
        client_id: client_id.to_owned(),
        sensor_id: sensor_id.to_owned(),
    };

    // Update in-memory map (fast path for subsequent reads in the same process).
    {
        let mut guard = match context.last_query_timestamps.lock() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };
        guard.insert(sensor_key, at);
    }

    // Persist to RocksDB for durability across restarts (BC-2.08.004 postcondition 2).
    if let Some(storage) = &context.storage {
        let raw_key = timestamp_key(client_id, sensor_id);
        let rfc3339 = at.to_rfc3339();
        // Best-effort write: if RocksDB is unavailable, in-memory value is still valid
        // for the current process lifetime. Log on failure but do not propagate —
        // timestamp persistence is not required for in-flight health checks.
        if let Err(e) = storage.put(StorageDomain::Default, &raw_key, rfc3339.as_bytes()) {
            tracing::warn!(
                client_id = %client_id,
                sensor_id = %sensor_id,
                error = %e,
                "health timestamp RocksDB write failed; in-memory value retained"
            );
        }
    }
}

// BC-5.38.005 self-check (S-5.04 F-S504-P1-005 complete):
// timestamp_key — non-trivial (format! + into_bytes). IMPLEMENTED.
// read_timestamp — non-trivial (in-memory fast path + RocksDB cold path on cache miss). IMPLEMENTED.
// write_timestamp — non-trivial (in-memory write-through + RocksDB persist when storage wired). IMPLEMENTED.
