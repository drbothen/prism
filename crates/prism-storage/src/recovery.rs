// S-1.02 — Crash recovery types and logic (VP-057).
//
// `advance_crash_counter` is a pure function that inspects a `DirtyBitEntry` and
// returns a `RecoveryAction`.  The denylist threshold is exactly 3 consecutive
// crashes (i.e., `consecutive_crashes + 1 >= 3`).

/// A dirty-bit entry recording crash history for a sensor or subsystem.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DirtyBitEntry {
    /// Number of consecutive crashes observed so far (0-indexed before this
    /// invocation).
    pub consecutive_crashes: u32,
}

/// Action to take after evaluating a `DirtyBitEntry`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RecoveryAction {
    /// Fewer than 3 consecutive crashes — log a warning and continue.
    Warn,
    /// Three or more consecutive crashes — deny the sensor for `expiry_seconds`.
    Denylist {
        /// Duration of the denylist in seconds.  Always 86400 (24 hours).
        expiry_seconds: u32,
    },
}

/// Evaluate a dirty-bit entry and return the appropriate `RecoveryAction`.
///
/// Pure function with no side effects (required for Kani feasibility).
///
/// Returns `RecoveryAction::Denylist { expiry_seconds: 86400 }` iff
/// `entry.consecutive_crashes + 1 >= 3` (i.e., `consecutive_crashes >= 2`).
/// Returns `RecoveryAction::Warn` otherwise.
///
/// Idempotent: the same entry always produces the same action.
///
/// VP-057: proved correct across all symbolic `u32` values by Kani.
pub fn advance_crash_counter(entry: DirtyBitEntry) -> RecoveryAction {
    // Threshold: consecutive_crashes + 1 >= 3, i.e., consecutive_crashes >= 2.
    if entry.consecutive_crashes >= 2 {
        RecoveryAction::Denylist {
            expiry_seconds: 86400,
        }
    } else {
        RecoveryAction::Warn
    }
}
