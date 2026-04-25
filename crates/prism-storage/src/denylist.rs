// S-2.02 — Query denylist with lazy expiry.
//
// Implements BC-2.15.008 (query denylisting after N consecutive failures).
//
// Storage: `watchdog` CF in RocksDB.
//   Key:   `denylist:{fingerprint}`
//   Value: `{failure_count}:{last_failure_ts}:{expiry_ts}`
//
// Expiry check is LAZY (at query-start time), not eager — no background reaper
// (Architecture Compliance Rule).
//
// The denylist threshold is configurable via `prism.toml`
// `[watchdog] denylist_threshold = 3`; read from `ConfigSnapshot` at startup.
//
// The `clear_denylist` capability check is enforced by the MCP tool layer;
// `prism-storage` provides the operation unconditionally (Dev Notes).

use prism_core::{PrismError, StorageDomain};

use crate::backend::RocksStorageBackend;

/// Default number of consecutive watchdog-triggered failures before a query
/// fingerprint is denylisted.
///
/// Configurable via `[watchdog] denylist_threshold` in `prism.toml`.
pub const DENYLIST_THRESHOLD: u32 = 3;

/// Default denylist expiry duration in seconds (1 hour).
pub const DENYLIST_EXPIRY_SECS: u64 = 3600;

/// Status returned by [`record_failure`] indicating the current denylist state
/// for a query fingerprint after recording the latest failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DenylistStatus {
    /// Failure recorded; threshold not yet reached.  `failure_count` is the
    /// new consecutive failure count after this recording.
    BelowThreshold { failure_count: u32 },
    /// Threshold reached; fingerprint is now denylisted until `expiry_ts`.
    Denylisted { failure_count: u32, expiry_ts: u64 },
}

/// A denylist entry as returned in [`crate::watchdog::WatchdogStatus`].
///
/// Populated by reading the `watchdog` CF and parsing stored values.
#[derive(Debug, Clone)]
pub struct DenylistEntry {
    /// SHA-256 fingerprint (hex string) of the denylisted query.
    pub fingerprint: String,
    /// Number of consecutive failures that triggered denylisting.
    pub failure_count: u32,
    /// Unix timestamp (seconds) of the most recent failure.
    pub last_failure_ts: u64,
    /// Unix timestamp (seconds) at which this entry expires.
    pub expiry_ts: u64,
}

/// Record a watchdog-triggered failure for the given query fingerprint.
///
/// Increments the consecutive failure counter.  If the new count is ≥
/// `DENYLIST_THRESHOLD`, marks the fingerprint as denylisted for
/// `DENYLIST_EXPIRY_SECS` seconds and returns `DenylistStatus::Denylisted`.
///
/// **Postcondition (BC-2.15.008):** entry is persisted to `watchdog` CF before
/// returning; expiry is set to `now + DENYLIST_EXPIRY_SECS`.
///
/// AC-5: after 3 consecutive failures, `is_denylisted()` returns `true`.
///
/// # Errors
///
/// Returns `PrismError::StorageWriteFailed` if the RocksDB put fails.
pub fn record_failure<B: RocksStorageBackend>(
    backend: &B,
    fingerprint: &str,
    threshold: u32,
) -> Result<DenylistStatus, PrismError> {
    // BC-2.15.008 postcondition: increment failure counter in watchdog CF;
    // if count >= threshold set expiry_ts = now + DENYLIST_EXPIRY_SECS and
    // return DenylistStatus::Denylisted
    let _ = (backend, fingerprint, threshold, StorageDomain::Watchdog);
    todo!("BC-2.15.008 postcondition: read existing failure record from watchdog CF, increment count, if count >= threshold write denylisted entry with expiry_ts=now+DENYLIST_EXPIRY_SECS")
}

/// Lazily check whether a query fingerprint is currently denylisted.
///
/// If an entry exists and `expiry_ts < now`, removes the entry from the `watchdog`
/// CF and returns `false` (lazy expiry — no background reaper per Architecture
/// Compliance Rule).
///
/// AC-5: returns `true` for a fingerprint denylisted after 3 consecutive failures.
/// AC-6: returns `false` after `clear_denylist(Some(fingerprint))` is called.
///
/// # Errors
///
/// Returns `PrismError::StorageReadFailed` if the RocksDB read fails.
pub fn is_denylisted<B: RocksStorageBackend>(
    backend: &B,
    fingerprint: &str,
) -> Result<bool, PrismError> {
    // BC-2.15.008 postcondition: read denylist:{fingerprint} key from watchdog CF;
    // if expiry_ts < now remove entry and return false; otherwise return true
    let _ = (backend, fingerprint);
    todo!("BC-2.15.008 postcondition: read denylist:{{fingerprint}} from watchdog CF; if expiry_ts < now delete entry and return Ok(false); else return Ok(true)")
}

/// Remove denylist entries from the `watchdog` CF.
///
/// - `Some(fingerprint)`: remove only that entry (AC-6).
/// - `None`: remove **all** `denylist:*` entries (EC-005).
///
/// Returns the count of entries removed.
///
/// AC-6: after `clear_denylist(Some(fp))`, `is_denylisted(fp)` returns `false`.
/// EC-005: `clear_denylist(None)` removes all entries; returns total count removed.
///
/// # Errors
///
/// Returns `PrismError::StorageWriteFailed` if any RocksDB remove fails.
pub fn clear_denylist<B: RocksStorageBackend>(
    backend: &B,
    fingerprint: Option<&str>,
) -> Result<usize, PrismError> {
    // AC-6 / EC-005: remove one or all denylist:* entries from watchdog CF
    let _ = (backend, fingerprint);
    todo!("BC-2.15.008: if Some(fp) remove denylist:{{fp}} and return 1 (or 0 if absent); if None scan all denylist:* keys and remove them all, returning count")
}
