#![allow(clippy::unwrap_used, clippy::expect_used)]
//! RED tests for WGC-W2-002: evict_expired must scan and delete from backend.
//!
//! The bug: `evict_expired` only iterates `self.write_cache`. After a process
//! restart, `write_cache` is empty but stale keys in `StorageDomain::EventBuffer`
//! are never discovered — they persist beyond their TTL (AC-4 violation).
//!
//! # RED gate
//! These tests write stale event keys directly to the backend (bypassing
//! `write_cache`) to simulate "post-restart" state, then verify that
//! `evict_expired` removes them. Both tests FAIL on the current code because
//! `evict_expired` never calls `self.backend.scan()` or `self.backend.remove()`
//! for keys it didn't track in `write_cache`.
//!
//! # Test IDs
//! - WGC-W2-002-backend-only   — stale keys in backend only are evicted
//! - WGC-W2-002-cache-and-backend — stale keys in both cache and backend are evicted

use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use prism_core::StorageDomain;
use prism_storage::backend::RocksStorageBackend;
use prism_storage::memory_backend::InMemoryBackend;
use serde_json::json;

use crate::event_buffer::{EventBufferStore, NormalizedRecord};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_in_memory_store() -> (Arc<InMemoryBackend>, EventBufferStore) {
    let backend = Arc::new(InMemoryBackend::new());
    let store = EventBufferStore::new(Arc::clone(&backend) as Arc<dyn RocksStorageBackend>);
    (backend, store)
}

/// Build an EventBuffer key with a stale timestamp directly, matching the
/// internal key format: `{sensor_id}/{table_name}/{client_id}/{ts_be:8}/{nanos:4}`
fn build_stale_event_key(
    sensor_id: &str,
    table_name: &str,
    client_id: &str,
    ingested_at: SystemTime,
) -> Vec<u8> {
    let micros = ingested_at
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_micros() as u64;
    let ts_bytes = micros.to_be_bytes();
    let nanos_suffix = [0u8; 4]; // deterministic suffix for tests
    format!("{sensor_id}/{table_name}/{client_id}/")
        .into_bytes()
        .into_iter()
        .chain(ts_bytes)
        .chain(std::iter::once(b'/'))
        .chain(nanos_suffix)
        .collect()
}

fn encode_record(payload: serde_json::Value, ingested_at: SystemTime) -> Vec<u8> {
    let record = NormalizedRecord {
        payload,
        ingested_at,
    };
    serde_json::to_vec(&record).expect("serialize must succeed")
}

// ---------------------------------------------------------------------------
// WGC-W2-002-backend-only
// ---------------------------------------------------------------------------

/// WGC-W2-002: evict_expired must delete stale keys that live in the backend
/// but NOT in write_cache (simulating post-restart state).
///
/// RED: current evict_expired only iterates write_cache. If write_cache is
/// empty (as it is after a restart), no backend keys are ever discovered or
/// deleted — they persist beyond their TTL indefinitely.
#[test]
fn test_WGC_W2_002_evict_expired_removes_stale_keys_from_backend_only() {
    let (backend, store) = make_in_memory_store();

    // Simulate post-restart: write a stale key directly to the backend,
    // bypassing write_cache entirely (write_cache is empty = restart state).
    let two_days_ago = SystemTime::now() - Duration::from_secs(2 * 86400);
    let stale_key = build_stale_event_key("crowdstrike", "process_events", "acme", two_days_ago);
    let stale_value = encode_record(json!({"post_restart": true}), two_days_ago);

    backend
        .put(StorageDomain::EventBuffer, &stale_key, &stale_value)
        .expect("put must succeed");

    // Verify the key is present in the backend before eviction.
    let before = backend
        .scan(
            StorageDomain::EventBuffer,
            b"crowdstrike/process_events/acme/",
        )
        .expect("scan must succeed");
    assert_eq!(
        before.len(),
        1,
        "setup: backend must contain 1 stale key before eviction"
    );

    // Call evict_expired with 24h retention — the 2-day-old key must be deleted.
    let deleted = store
        .evict_expired("crowdstrike", "process_events", Duration::from_secs(86400))
        .expect("evict_expired must not return error");

    assert_eq!(
        deleted, 1,
        "WGC-W2-002: evict_expired must return 1 (deleted the stale backend key). \
         Current code returns 0 because it only scans write_cache, which is empty \
         after a simulated restart. Stale backend key was NOT removed."
    );

    // Verify the key has been removed from the backend.
    let after = backend
        .scan(
            StorageDomain::EventBuffer,
            b"crowdstrike/process_events/acme/",
        )
        .expect("scan must succeed after eviction");
    assert!(
        after.is_empty(),
        "WGC-W2-002: stale backend key must be absent after evict_expired. \
         Found {} key(s) still in backend.",
        after.len()
    );
}

// ---------------------------------------------------------------------------
// WGC-W2-002-cache-and-backend
// ---------------------------------------------------------------------------

/// WGC-W2-002: evict_expired must evict stale keys from BOTH write_cache AND
/// backend in a single call (interleaved scenario).
///
/// RED: current code only evicts from write_cache. Backend keys are ignored
/// regardless of whether cache entries also exist.
#[test]
fn test_WGC_W2_002_evict_expired_removes_stale_keys_from_both_cache_and_backend() {
    let (backend, store) = make_in_memory_store();

    let two_days_ago = SystemTime::now() - Duration::from_secs(2 * 86400);

    // Write one stale record through write_events (goes to both cache + backend).
    let stale_via_api = NormalizedRecord {
        payload: json!({"via": "api"}),
        ingested_at: two_days_ago,
    };
    store
        .write_events("crowdstrike", "process_events", "acme", vec![stale_via_api])
        .expect("write_events must succeed");

    // Write a second stale key directly to the backend only (simulates a key
    // that arrived in a previous process run — not in this run's write_cache).
    let backend_only_key = build_stale_event_key(
        "crowdstrike",
        "process_events",
        "acme",
        two_days_ago - Duration::from_secs(3600), // 1 hour earlier
    );
    let backend_only_value = encode_record(
        json!({"via": "direct_backend"}),
        two_days_ago - Duration::from_secs(3600),
    );
    backend
        .put(
            StorageDomain::EventBuffer,
            &backend_only_key,
            &backend_only_value,
        )
        .expect("direct backend put must succeed");

    // Before eviction: backend has 2 stale keys (one from write_events, one direct).
    let before = backend
        .scan(
            StorageDomain::EventBuffer,
            b"crowdstrike/process_events/acme/",
        )
        .expect("scan must succeed");
    assert_eq!(
        before.len(),
        2,
        "setup: backend must contain 2 stale keys before eviction"
    );

    // Evict with 24h retention — both 2-day-old keys must be deleted.
    let deleted = store
        .evict_expired("crowdstrike", "process_events", Duration::from_secs(86400))
        .expect("evict_expired must not return error");

    assert_eq!(
        deleted, 2,
        "WGC-W2-002: evict_expired must delete both stale keys (cache + backend). \
         Current code only deletes the write_cache entry (returns 1 at best), \
         leaving the direct-backend key untouched. Expected deleted=2, got deleted={deleted}."
    );

    // Verify all stale keys are gone.
    let after = backend
        .scan(
            StorageDomain::EventBuffer,
            b"crowdstrike/process_events/acme/",
        )
        .expect("scan must succeed after eviction");
    assert!(
        after.is_empty(),
        "WGC-W2-002: all stale keys must be absent after evict_expired. \
         Found {} key(s) still in backend.",
        after.len()
    );
}
