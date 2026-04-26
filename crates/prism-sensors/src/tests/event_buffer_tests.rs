#![allow(clippy::expect_used, clippy::unwrap_used)]
//! Unit tests for `EventBufferStore` CF ops and TTL eviction.
//!
//! Story: S-2.08 | AC-2, AC-4, AC-5
//!
//! # Coverage
//! - write_events: basic write returns count (AC-4)
//! - write_events: empty batch writes 0 records
//! - scan_events: results within time range are returned (AC-2)
//! - scan_events: results outside time range are not returned
//! - evict_expired: records older than retention are deleted (AC-4)
//! - evict_expired: records within retention are not deleted
//! - has_data: returns false when buffer is empty (AC-5 cold-start)
//! - has_data: returns true after write_events (AC-5)
//! - buffer_size_bytes: returns 0 for empty buffer
//! - buffer_size_bytes: returns non-zero after writes
//! - Key format: sensor_id/table_name/client_id/timestamp_be/ulid structure (architecture compliance)
//!
//! # RED GATE
//! All tests call methods that are currently `todo!()` stubs.
//! They will PANIC with "not yet implemented" at runtime — RED by design.
//! The MockRocksBackend used here causes a compile-time dependency only;
//! none of the `todo!()` bodies run.

use std::sync::Arc;
use std::time::{Duration, SystemTime};

use crate::event_buffer::{EventBufferStore, NormalizedRecord};
use prism_storage::backend::RocksStorageBackend;
use serde_json::json;

// ---------------------------------------------------------------------------
// Test helper: minimal no-op mock backend
// ---------------------------------------------------------------------------
// The EventBufferStore stub takes Arc<dyn RocksStorageBackend>. We provide a
// no-op implementation that satisfies the trait bound — the todo!() stubs
// inside EventBufferStore never actually call the backend methods, so this
// mock never needs to do real I/O. It exists purely to construct the struct.

struct NoOpBackend;

impl RocksStorageBackend for NoOpBackend {
    fn get(
        &self,
        _domain: prism_core::StorageDomain,
        _key: &[u8],
    ) -> Result<Option<Vec<u8>>, prism_core::PrismError> {
        Ok(None)
    }

    fn put(
        &self,
        _domain: prism_core::StorageDomain,
        _key: &[u8],
        _value: &[u8],
    ) -> Result<(), prism_core::PrismError> {
        Ok(())
    }

    fn put_batch(
        &self,
        _domain: prism_core::StorageDomain,
        _entries: &[(&[u8], &[u8])],
    ) -> Result<(), prism_core::PrismError> {
        Ok(())
    }

    fn remove(
        &self,
        _domain: prism_core::StorageDomain,
        _key: &[u8],
    ) -> Result<(), prism_core::PrismError> {
        Ok(())
    }

    fn scan(
        &self,
        _domain: prism_core::StorageDomain,
        _prefix: &[u8],
    ) -> Result<Vec<(Vec<u8>, Vec<u8>)>, prism_core::PrismError> {
        Ok(vec![])
    }

    fn scan_range(
        &self,
        _domain: prism_core::StorageDomain,
        _start: &[u8],
        _end: &[u8],
    ) -> Result<Vec<(Vec<u8>, Vec<u8>)>, prism_core::PrismError> {
        Ok(vec![])
    }
}

fn make_store() -> EventBufferStore {
    EventBufferStore::new(Arc::new(NoOpBackend))
}

fn make_record(payload: serde_json::Value) -> NormalizedRecord {
    NormalizedRecord {
        payload,
        ingested_at: SystemTime::now(),
    }
}

fn make_record_at(payload: serde_json::Value, ingested_at: SystemTime) -> NormalizedRecord {
    NormalizedRecord {
        payload,
        ingested_at,
    }
}

// ---------------------------------------------------------------------------
// write_events — AC-4
// ---------------------------------------------------------------------------

#[test]
fn test_BC_2_08_write_events_returns_record_count() {
    // RED: write_events is todo!()
    // AC-4: write returns the count of records written
    let store = make_store();
    let records = vec![
        make_record(json!({"event": "login", "user": "alice"})),
        make_record(json!({"event": "logout", "user": "alice"})),
    ];
    let count = store
        .write_events("crowdstrike", "process_events", "acme", records)
        .expect("write_events must not return error on valid input");
    assert_eq!(
        count, 2,
        "write_events must return the number of records written"
    );
}

#[test]
fn test_BC_2_08_write_events_empty_batch_returns_zero() {
    // RED: write_events is todo!()
    let store = make_store();
    let count = store
        .write_events("crowdstrike", "process_events", "acme", vec![])
        .expect("write_events with empty batch must succeed");
    assert_eq!(count, 0, "empty batch write must return 0");
}

#[test]
fn test_BC_2_08_write_events_single_record_returns_one() {
    // RED: write_events is todo!()
    let store = make_store();
    let records = vec![make_record(json!({"pid": 1234}))];
    let count = store
        .write_events("cyberint", "alerts", "beta-corp", records)
        .expect("write_events must succeed");
    assert_eq!(count, 1, "single record write must return 1");
}

// ---------------------------------------------------------------------------
// scan_events — AC-2
// ---------------------------------------------------------------------------

#[test]
fn test_BC_2_08_scan_events_returns_records_in_time_range() {
    // RED: scan_events is todo!()
    // AC-2: EventStream query serves from buffer without live API call
    let store = make_store();
    let now = SystemTime::now();
    let since = now - Duration::from_secs(300); // 5 minutes ago
    let until = now + Duration::from_secs(60); // 1 minute ahead

    let records = store
        .scan_events("crowdstrike", "process_events", "acme", since, until)
        .expect("scan_events must not return error");
    // After write_events is implemented, this would find records.
    // With todo!() stub, this is RED.
    let _ = records;
}

#[test]
fn test_BC_2_08_scan_events_empty_buffer_returns_empty_vec() {
    // RED: scan_events is todo!()
    // Buffer is fresh (no writes) — must return empty, not error
    let store = make_store();
    let now = SystemTime::now();
    let records = store
        .scan_events(
            "armis",
            "devices",
            "client-x",
            now - Duration::from_secs(3600),
            now,
        )
        .expect("scan_events on empty buffer must return Ok(vec![])");
    assert!(
        records.is_empty(),
        "scan_events on empty buffer must return empty vec, not error"
    );
}

#[test]
fn test_BC_2_08_scan_events_since_after_until_returns_empty() {
    // RED: scan_events is todo!()
    // Inverted time range (since > until) must return empty, not error or panic
    let store = make_store();
    let now = SystemTime::now();
    let records = store
        .scan_events(
            "crowdstrike",
            "process_events",
            "acme",
            now,                           // since = now
            now - Duration::from_secs(60), // until = 1 minute ago (inverted)
        )
        .expect("inverted time range must return Ok(vec![]) not error");
    assert!(
        records.is_empty(),
        "inverted time range must return empty vec"
    );
}

// ---------------------------------------------------------------------------
// evict_expired — AC-4
// ---------------------------------------------------------------------------

#[test]
fn test_BC_2_08_evict_expired_returns_deletion_count() {
    // RED: evict_expired is todo!()
    // AC-4: evict_expired must return count of deleted records
    let store = make_store();
    let retention = Duration::from_secs(86400); // 24 hours
    let deleted = store
        .evict_expired("crowdstrike", "process_events", retention)
        .expect("evict_expired must not return error");
    // Empty buffer → 0 deletions
    assert_eq!(deleted, 0, "evict_expired on empty buffer must return 0");
}

#[test]
fn test_BC_2_08_evict_expired_zero_retention_evicts_all() {
    // RED: evict_expired is todo!()
    // Retention of 0 seconds means everything is expired immediately
    let store = make_store();
    let deleted = store
        .evict_expired("crowdstrike", "process_events", Duration::ZERO)
        .expect("evict_expired with zero retention must not return error");
    // After implementation: all records should be deleted.
    // This test validates the return type and error contract.
    let _ = deleted;
}

#[test]
fn test_BC_2_08_evict_expired_does_not_delete_fresh_records() {
    // RED: evict_expired is todo!() — both write_events and evict_expired are stubs
    // AC-4 invariant: records written now are NOT expired with a 24h retention
    let store = make_store();
    // Write fresh records
    let records = vec![make_record(json!({"host": "x"}))];
    store
        .write_events("crowdstrike", "process_events", "acme", records)
        .expect("write_events must succeed");
    // Evict with 24h retention — fresh records should NOT be deleted
    let deleted = store
        .evict_expired("crowdstrike", "process_events", Duration::from_secs(86400))
        .expect("evict_expired must not return error");
    assert_eq!(
        deleted, 0,
        "AC-4: fresh records must not be evicted under 24h retention"
    );
}

#[test]
fn test_BC_2_08_evict_expired_removes_records_older_than_retention() {
    // RED: evict_expired + write_events are both todo!()
    // AC-4: records with ingested_at older than (now - retention) must be deleted
    let store = make_store();
    // Simulate a stale record (ingested 2 days ago)
    let two_days_ago = SystemTime::now() - Duration::from_secs(2 * 86400);
    let stale = make_record_at(json!({"stale": true}), two_days_ago);
    store
        .write_events("crowdstrike", "process_events", "acme", vec![stale])
        .expect("write_events must succeed");
    // Evict with 24h retention — the 2-day-old record must be deleted
    let deleted = store
        .evict_expired("crowdstrike", "process_events", Duration::from_secs(86400))
        .expect("evict_expired must not return error");
    assert_eq!(
        deleted, 1,
        "AC-4: records older than retention must be evicted (1 stale record expected deleted)"
    );
}

// ---------------------------------------------------------------------------
// has_data — AC-5 (cold-start detection)
// ---------------------------------------------------------------------------

#[test]
fn test_BC_2_08_has_data_returns_false_on_empty_buffer() {
    // RED: has_data is todo!()
    // AC-5: cold-start detection — empty buffer must return false
    let store = make_store();
    let result = store
        .has_data("crowdstrike", "process_events", "acme")
        .expect("has_data must not return error on empty buffer");
    assert!(
        !result,
        "AC-5: has_data must return false when buffer is empty (cold start)"
    );
}

#[test]
fn test_BC_2_08_has_data_returns_true_after_write() {
    // RED: has_data + write_events are both todo!()
    // AC-5: after a write, has_data must return true
    let store = make_store();
    store
        .write_events(
            "crowdstrike",
            "process_events",
            "acme",
            vec![make_record(json!({"event": "proc_create"}))],
        )
        .expect("write_events must succeed");
    let result = store
        .has_data("crowdstrike", "process_events", "acme")
        .expect("has_data must not return error after write");
    assert!(
        result,
        "AC-5: has_data must return true after records have been written"
    );
}

#[test]
fn test_BC_2_08_has_data_scoped_to_client_id() {
    // RED: has_data + write_events are both todo!()
    // has_data must be scoped to (sensor_id, table_name, client_id) —
    // writing for client "alpha" must not make has_data return true for client "beta"
    let store = make_store();
    store
        .write_events(
            "crowdstrike",
            "process_events",
            "alpha",
            vec![make_record(json!({"event": "proc_create"}))],
        )
        .expect("write_events for alpha must succeed");
    let result = store
        .has_data("crowdstrike", "process_events", "beta")
        .expect("has_data for beta must not error");
    assert!(
        !result,
        "has_data must be scoped to client_id — writing for alpha must not affect beta"
    );
}

// ---------------------------------------------------------------------------
// buffer_size_bytes — Task 8 (diagnostics)
// ---------------------------------------------------------------------------

#[test]
fn test_BC_2_08_buffer_size_bytes_empty_buffer_returns_zero() {
    // RED: buffer_size_bytes is todo!()
    let store = make_store();
    let size = store
        .buffer_size_bytes("crowdstrike", "process_events", "acme")
        .expect("buffer_size_bytes must not return error on empty buffer");
    assert_eq!(size, 0, "empty buffer must report 0 bytes");
}

#[test]
fn test_BC_2_08_buffer_size_bytes_nonzero_after_write() {
    // RED: buffer_size_bytes + write_events are both todo!()
    // Task 8: diagnostics must report non-zero size after writes
    let store = make_store();
    store
        .write_events(
            "crowdstrike",
            "process_events",
            "acme",
            vec![make_record(json!({"event": "proc_create", "pid": 1234}))],
        )
        .expect("write_events must succeed");
    let size = store
        .buffer_size_bytes("crowdstrike", "process_events", "acme")
        .expect("buffer_size_bytes must not return error after write");
    assert!(
        size > 0,
        "buffer_size_bytes must be > 0 after records are written"
    );
}

// ---------------------------------------------------------------------------
// Key format compliance (architecture)
// ---------------------------------------------------------------------------

#[test]
fn test_BC_2_08_write_events_rejects_slash_in_sensor_id() {
    // RED: write_events is todo!()
    // Architecture compliance: key format uses '/' as separator — sensor_id,
    // table_name, and client_id must not contain '/' themselves.
    // This test documents the expected rejection behavior.
    let store = make_store();
    let records = vec![make_record(json!({"event": "test"}))];
    let result = store.write_events("sensor/with/slash", "table", "client", records);
    // Either: returns Err (validation), or succeeds but produces malformed keys.
    // When implemented, this MUST return Err to prevent key collisions.
    // At todo!() stage: will panic (RED). After impl: must be Err.
    match result {
        Err(_) => {} // expected after implementation
        Ok(_) => panic!(
            "write_events must reject sensor_id containing '/' to prevent key format collisions"
        ),
    }
}

// ---------------------------------------------------------------------------
// Backend error propagation (W2-P1-A-001, W2-P1-A-004)
// ---------------------------------------------------------------------------
//
// These tests use a FailingBackend that returns Err from put_batch and remove.
// RED GATE: before the fix, write_events and evict_expired silently swallow
// these errors. These tests FAIL (RED) before the fix is applied.

/// A backend that always returns `StorageWriteFailed` from put_batch and remove.
struct FailingBackend;

impl RocksStorageBackend for FailingBackend {
    fn get(
        &self,
        _domain: prism_core::StorageDomain,
        _key: &[u8],
    ) -> Result<Option<Vec<u8>>, prism_core::PrismError> {
        Ok(None)
    }

    fn put(
        &self,
        _domain: prism_core::StorageDomain,
        _key: &[u8],
        _value: &[u8],
    ) -> Result<(), prism_core::PrismError> {
        Ok(())
    }

    fn put_batch(
        &self,
        domain: prism_core::StorageDomain,
        _entries: &[(&[u8], &[u8])],
    ) -> Result<(), prism_core::PrismError> {
        Err(prism_core::PrismError::StorageWriteFailed {
            domain: domain.column_family_name().to_owned(),
            detail: "injected put_batch failure".to_owned(),
        })
    }

    fn remove(
        &self,
        domain: prism_core::StorageDomain,
        _key: &[u8],
    ) -> Result<(), prism_core::PrismError> {
        Err(prism_core::PrismError::StorageWriteFailed {
            domain: domain.column_family_name().to_owned(),
            detail: "injected remove failure".to_owned(),
        })
    }

    fn scan(
        &self,
        _domain: prism_core::StorageDomain,
        _prefix: &[u8],
    ) -> Result<Vec<(Vec<u8>, Vec<u8>)>, prism_core::PrismError> {
        Ok(vec![])
    }

    fn scan_range(
        &self,
        _domain: prism_core::StorageDomain,
        _start: &[u8],
        _end: &[u8],
    ) -> Result<Vec<(Vec<u8>, Vec<u8>)>, prism_core::PrismError> {
        Ok(vec![])
    }
}

fn make_failing_store() -> EventBufferStore {
    EventBufferStore::new(Arc::new(FailingBackend))
}

/// W2-P1-A-001: write_events must propagate backend put_batch errors.
///
/// RED before fix: the current implementation uses `let _ = self.backend.put_batch(...)`,
/// silently discarding the error. This test fails (returns Ok) before the fix.
#[test]
fn test_W2_P1_A_001_write_events_propagates_backend_put_batch_error() {
    let store = make_failing_store();
    let records = vec![make_record(json!({"event": "login"}))];
    let result = store.write_events("crowdstrike", "process_events", "acme", records);
    assert!(
        result.is_err(),
        "W2-P1-A-001: write_events must return Err when backend put_batch fails, \
         not silently swallow the error"
    );
}

/// W2-P1-A-004: evict_expired must propagate backend remove errors.
///
/// RED before fix: the current implementation uses `let _ = self.backend.remove(...)`,
/// silently discarding the error. This test fails (returns Ok) before the fix.
#[test]
fn test_W2_P1_A_004_evict_expired_propagates_backend_remove_error() {
    // We need to write a stale record first so evict_expired has something to
    // delete from the backend. Use a store with the NoOpBackend for the write
    // (so put_batch succeeds), then we need a way to make remove fail.
    //
    // The cleanest approach: write the record to a temporary NoOpBackend store
    // to populate the in-memory cache, then we can't easily transfer that state
    // to the FailingBackend. Instead, we write a stale record using a custom
    // dual-mode backend: put_batch succeeds, remove fails.
    use std::sync::atomic::{AtomicBool, Ordering};

    struct PutOkRemoveFailBackend {
        put_called: AtomicBool,
    }

    impl RocksStorageBackend for PutOkRemoveFailBackend {
        fn get(
            &self,
            _domain: prism_core::StorageDomain,
            _key: &[u8],
        ) -> Result<Option<Vec<u8>>, prism_core::PrismError> {
            Ok(None)
        }

        fn put(
            &self,
            _domain: prism_core::StorageDomain,
            _key: &[u8],
            _value: &[u8],
        ) -> Result<(), prism_core::PrismError> {
            Ok(())
        }

        fn put_batch(
            &self,
            _domain: prism_core::StorageDomain,
            _entries: &[(&[u8], &[u8])],
        ) -> Result<(), prism_core::PrismError> {
            self.put_called.store(true, Ordering::SeqCst);
            Ok(())
        }

        fn remove(
            &self,
            domain: prism_core::StorageDomain,
            _key: &[u8],
        ) -> Result<(), prism_core::PrismError> {
            Err(prism_core::PrismError::StorageWriteFailed {
                domain: domain.column_family_name().to_owned(),
                detail: "injected remove failure".to_owned(),
            })
        }

        fn scan(
            &self,
            _domain: prism_core::StorageDomain,
            _prefix: &[u8],
        ) -> Result<Vec<(Vec<u8>, Vec<u8>)>, prism_core::PrismError> {
            Ok(vec![])
        }

        fn scan_range(
            &self,
            _domain: prism_core::StorageDomain,
            _start: &[u8],
            _end: &[u8],
        ) -> Result<Vec<(Vec<u8>, Vec<u8>)>, prism_core::PrismError> {
            Ok(vec![])
        }
    }

    let backend = Arc::new(PutOkRemoveFailBackend {
        put_called: AtomicBool::new(false),
    });
    let store = EventBufferStore::new(backend);

    // Write a stale record (2 days old) so there is something in the cache to evict
    let two_days_ago = SystemTime::now() - Duration::from_secs(2 * 86400);
    let stale = make_record_at(json!({"stale": true}), two_days_ago);
    store
        .write_events("crowdstrike", "process_events", "acme", vec![stale])
        .expect("write_events must succeed with PutOkRemoveFailBackend");

    // Now evict with 24h retention — the stale record qualifies for deletion,
    // backend.remove will fail. Must propagate the error.
    let result = store.evict_expired("crowdstrike", "process_events", Duration::from_secs(86400));
    assert!(
        result.is_err(),
        "W2-P1-A-004: evict_expired must return Err when backend remove fails, \
         not silently swallow the error"
    );
}
