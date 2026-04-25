// S-2.01 — Dirty bit crash-recovery protocol (TDD step b compile-stub).
//
// These functions implement the BC-2.15.005 dirty bit protocol:
//   - `set_dirty`:              write key=query_id to `dirty_bits` CF with sync:true
//   - `clear_dirty`:            delete key=query_id from `dirty_bits` CF
//   - `check_dirty_on_startup`: scan all keys in `dirty_bits` CF; return uncleared IDs
//
// All writes use `WriteOptions { sync: true }` (BC-2.15.005 invariant).
// Implementer fills in the todo!() bodies during step (c).

use prism_core::PrismError;

use crate::rocksdb_backend::RocksDbBackend;

/// Set a dirty bit for the given query ID before query execution begins.
///
/// Writes key=`query_id`, value=current timestamp bytes to the `dirty_bits`
/// column family with `WriteOptions { sync: true }` so the entry survives
/// an OOM kill or power loss (BC-2.15.005).
///
/// Returns `Err(PrismError::StorageWriteFailed { .. })` (E-STORE-009) if the
/// write fails — the caller MUST abort the query on error (fail-closed).
pub fn set_dirty(_db: &RocksDbBackend, query_id: &str) -> Result<(), PrismError> {
    todo!(
        "step c implementer — put key={:?} in dirty_bits CF with sync:true WriteOptions",
        query_id
    )
}

/// Clear the dirty bit for the given query ID after successful query completion.
///
/// Deletes key=`query_id` from the `dirty_bits` column family.
/// No-op if the key is absent (not an error).
pub fn clear_dirty(_db: &RocksDbBackend, query_id: &str) -> Result<(), PrismError> {
    todo!(
        "step c implementer — delete key={:?} from dirty_bits CF",
        query_id
    )
}

/// Scan the `dirty_bits` CF at startup and return all uncleared query IDs.
///
/// Called once at startup before accepting any queries (BC-2.15.005).
/// For each returned ID, the caller:
///   1. Increments `consecutive_crashes`
///   2. If >= 3: adds to watchdog denylist (86400s)
///   3. Logs WARN
///   4. Clears the dirty bit
///
/// Returns an empty `Vec` if no dirty bits are present (clean shutdown).
pub fn check_dirty_on_startup(_db: &RocksDbBackend) -> Result<Vec<String>, PrismError> {
    todo!("step c implementer — scan dirty_bits CF, collect all keys as UTF-8 query IDs")
}
