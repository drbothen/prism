// S-2.01 — Dirty bit crash-recovery protocol (BC-2.15.005).
//
// These functions implement the BC-2.15.005 dirty bit protocol:
//   - `set_dirty`:              write key=query_id to `dirty_bits` CF with sync:true
//   - `clear_dirty`:            delete key=query_id from `dirty_bits` CF
//   - `check_dirty_on_startup`: scan all keys in `dirty_bits` CF; return uncleared IDs
//
// All writes use `WriteOptions { sync: true }` (BC-2.15.005 invariant).

use prism_core::{PrismError, StorageDomain};

use crate::rocksdb_backend::RocksDbBackend;

/// Set a dirty bit for the given query ID before query execution begins.
///
/// Writes key=`query_id`, value=current timestamp bytes to the `dirty_bits`
/// column family with `WriteOptions { sync: true }` so the entry survives
/// an OOM kill or power loss (BC-2.15.005).
///
/// Returns `Err(PrismError::StorageWriteFailed { .. })` (E-STORE-002) if the
/// write fails — the caller MUST abort the query on error (fail-closed).
pub fn set_dirty(db: &RocksDbBackend, query_id: &str) -> Result<(), PrismError> {
    let cf_name = StorageDomain::DirtyBits.column_family_name();
    let cf = db
        .db()
        .cf_handle(cf_name)
        .ok_or_else(|| PrismError::StorageDomainNotFound {
            domain: cf_name.to_owned(),
        })?;

    // Value = current Unix timestamp as little-endian u64 bytes.
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let opts = RocksDbBackend::sync_write_options();
    db.db()
        .put_cf_opt(&cf, query_id.as_bytes(), ts.to_le_bytes(), &opts)
        .map_err(|e| PrismError::StorageWriteFailed {
            domain: cf_name.to_owned(),
            detail: e.to_string(),
        })
}

/// Clear the dirty bit for the given query ID after successful query completion.
///
/// Deletes key=`query_id` from the `dirty_bits` column family.
/// No-op if the key is absent (not an error).
pub fn clear_dirty(db: &RocksDbBackend, query_id: &str) -> Result<(), PrismError> {
    let cf_name = StorageDomain::DirtyBits.column_family_name();
    let cf = db
        .db()
        .cf_handle(cf_name)
        .ok_or_else(|| PrismError::StorageDomainNotFound {
            domain: cf_name.to_owned(),
        })?;

    db.db()
        .delete_cf(&cf, query_id.as_bytes())
        .map_err(|e| PrismError::StorageWriteFailed {
            domain: cf_name.to_owned(),
            detail: e.to_string(),
        })
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
pub fn check_dirty_on_startup(db: &RocksDbBackend) -> Result<Vec<String>, PrismError> {
    let cf_name = StorageDomain::DirtyBits.column_family_name();
    let cf = db
        .db()
        .cf_handle(cf_name)
        .ok_or_else(|| PrismError::StorageDomainNotFound {
            domain: cf_name.to_owned(),
        })?;

    let iter = db.db().full_iterator_cf(&cf, rocksdb::IteratorMode::Start);

    let mut ids = Vec::new();
    for item in iter {
        let (key, _val) = item.map_err(|e| PrismError::StorageReadFailed {
            domain: cf_name.to_owned(),
            detail: e.to_string(),
        })?;
        let id = String::from_utf8_lossy(&key).into_owned();
        ids.push(id);
    }

    if !ids.is_empty() {
        tracing::warn!(
            uncleared_count = ids.len(),
            ids = ?ids,
            "dirty bits found on startup — previous run may have crashed (BC-2.15.005)"
        );
    }

    Ok(ids)
}
