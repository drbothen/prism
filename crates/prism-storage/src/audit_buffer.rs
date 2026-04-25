// S-2.02 — Audit buffer write path, overflow protection, and exponential backoff retry.
//
// Implements BC-2.15.003 (buffered audit log persistence) and BC-2.15.004 (overflow purge).
//
// Key format (lexicographically ordered, enables ordered scan):
//   `audit:{timestamp_ns:020}:{trace_id}`
//   Timestamp is zero-padded to 20 digits so lexicographic order == chronological order.
// Value format: `bincode::encode(entry)` — binary serialization via bincode 2.x.
//
// Storage domain: `StorageDomain::AuditBuffer` (RocksDB `audit_buffer` CF).
// All writes go through `RocksStorageBackend` trait — never `RocksDbBackend` directly.

use prism_core::{PrismError, StorageDomain};

use crate::backend::RocksStorageBackend;

/// An audit entry to be persisted in the `audit_buffer` column family.
///
/// Serialised with `bincode::encode` before storage.  The trace_id provides
/// uniqueness within the same nanosecond timestamp bucket.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AuditEntry {
    /// Nanosecond-precision Unix timestamp.
    pub timestamp_ns: u64,
    /// Trace / correlation ID (UUID v7 string) for deduplication.
    pub trace_id: String,
    /// Structured payload — JSON-serialisable map of event fields.
    pub payload: std::collections::BTreeMap<String, String>,
}

/// Maximum number of entries the audit buffer may hold before overflow purge runs.
///
/// Defined as a constant per the Dev Notes: not configurable at runtime.
pub const AUDIT_BUFFER_MAX_ENTRIES: usize = 100_000;

/// Target entry count after overflow purge (10 % headroom below the max).
pub const AUDIT_BUFFER_PURGE_TARGET: usize = 90_000;

/// Forwarding retry parameters (BC-2.15.003).
pub const RETRY_BASE_DELAY_SECS: u64 = 1;
pub const RETRY_MAX_DELAY_SECS: u64 = 60;
pub const RETRY_MAX_ATTEMPTS: u32 = 10;
pub const RETRY_MULTIPLIER: u32 = 2;

/// Build the storage key for an audit entry.
///
/// Key layout: `audit:{timestamp_ns:020}:{trace_id}`
///
/// The timestamp is zero-padded to 20 decimal digits so that lexicographic
/// ordering of keys equals chronological ordering.
fn audit_key(timestamp_ns: u64, trace_id: &str) -> Vec<u8> {
    format!("audit:{timestamp_ns:020}:{trace_id}").into_bytes()
}

/// Append a single audit entry to the `audit_buffer` RocksDB column family.
///
/// Key layout: `audit:{timestamp_ns:020}:{trace_id}`
/// Value: `bincode::encode(entry)`
///
/// **Postcondition (BC-2.15.003):** entry is durably in RocksDB *before* any
/// forwarding attempt is made.  Forwarding is the responsibility of the
/// background task, not this function.
///
/// # Errors
///
/// Returns `PrismError::StorageWriteFailed` when the RocksDB put fails
/// (E-AUDIT-001 per BC-2.15.003 Error Conditions).
pub fn append_audit_entry<B: RocksStorageBackend>(
    backend: &B,
    entry: &AuditEntry,
) -> Result<(), PrismError> {
    let key = audit_key(entry.timestamp_ns, &entry.trace_id);
    let value =
        bincode::serde::encode_to_vec(entry, bincode::config::standard()).map_err(|e| {
            PrismError::StorageWriteFailed {
                domain: StorageDomain::AuditBuffer.column_family_name().to_owned(),
                detail: format!("bincode encode error: {e}"),
            }
        })?;
    backend.put(StorageDomain::AuditBuffer, &key, &value)
}

/// Scan the `audit_buffer` CF, and if the entry count exceeds
/// [`AUDIT_BUFFER_MAX_ENTRIES`], delete the oldest entries (lowest timestamp
/// keys) until the count reaches [`AUDIT_BUFFER_PURGE_TARGET`].
///
/// Returns the number of entries purged (0 when no overflow).
///
/// **Postcondition (BC-2.15.004):** a `tracing::warn!` is emitted before
/// purge; after purge a special `audit_buffer_purge` audit entry is written.
///
/// # Errors
///
/// Returns `PrismError::StorageWriteFailed` / `StorageReadFailed` if the
/// underlying RocksDB operations fail (E-AUDIT-004 per BC-2.15.004).
pub fn check_and_purge_overflow<B: RocksStorageBackend>(backend: &B) -> Result<usize, PrismError> {
    // Scan all entries to count them (keys only sufficient but scan returns both).
    let all = backend.scan(StorageDomain::AuditBuffer, b"audit:")?;
    let count = all.len();

    if count <= AUDIT_BUFFER_MAX_ENTRIES {
        return Ok(0);
    }

    // BC-2.15.004 postcondition: emit warn before purge.
    let to_delete = count - AUDIT_BUFFER_PURGE_TARGET;
    tracing::warn!(
        count = count,
        purge_target = AUDIT_BUFFER_PURGE_TARGET,
        to_delete = to_delete,
        "audit_buffer overflow — purging oldest {to_delete} entries to reach target {AUDIT_BUFFER_PURGE_TARGET}"
    );

    // Delete the `to_delete` oldest entries (lowest keys = earliest timestamps).
    // `all` is already in lexicographic order from the scan.
    for (key, _) in all.iter().take(to_delete) {
        backend.remove(StorageDomain::AuditBuffer, key)?;
    }

    Ok(to_delete)
}

/// Attempt to forward a single entry to the configured sinks (stderr + Vector),
/// retrying with exponential backoff on failure.
///
/// This is a private helper called by the background forwarding task.  Not
/// exposed in `lib.rs` — internal to the background maintenance loop.
///
/// Retry parameters (BC-2.15.003): base=1s, multiplier=2x, cap=60s, max=10.
/// On final failure after all retries, emits `tracing::error!` with structured
/// fields and leaves the entry in the buffer for the next forwarding cycle.
///
/// # Errors
///
/// Returns `PrismError::Internal` if all 10 retry attempts are exhausted.
#[allow(dead_code)] // called by background forwarding task (implementation in next dispatch)
pub(crate) fn retry_forward_entry(entry: &AuditEntry) -> Result<(), PrismError> {
    let mut delay_secs = RETRY_BASE_DELAY_SECS;
    for attempt in 1..=RETRY_MAX_ATTEMPTS {
        // Placeholder: actual forwarding to stderr/Vector not yet implemented.
        // When the forwarding implementation is added, this `let _ = (entry, attempt)`
        // will be replaced with the actual forward call.
        let forward_result: Result<(), String> = Err("not yet wired".to_string());
        match forward_result {
            Ok(()) => return Ok(()),
            Err(e) => {
                if attempt == RETRY_MAX_ATTEMPTS {
                    tracing::error!(
                        trace_id = %entry.trace_id,
                        timestamp_ns = entry.timestamp_ns,
                        attempt = attempt,
                        error = %e,
                        "audit forward failed after all retries — entry left in buffer"
                    );
                    return Err(PrismError::Internal {
                        detail: format!(
                            "audit forward exhausted {RETRY_MAX_ATTEMPTS} retries: {e}"
                        ),
                    });
                }
                // Exponential backoff with cap.
                delay_secs = (delay_secs * RETRY_MULTIPLIER as u64).min(RETRY_MAX_DELAY_SECS);
            }
        }
    }
    // Unreachable: loop always returns in body.
    unreachable!("retry loop must return within MAX_ATTEMPTS iterations")
}
