// VP-057: Crash Recovery — Denylist Triggered at Three Consecutive Crashes.
//
// Kani proof: symbolic u32 for consecutive_crashes; asserts `advance_crash_counter`
// returns Denylist{86400s} iff consecutive_crashes + 1 >= 3, Warn otherwise.
// Also asserts idempotency.
//
// Traces to: BC-2.15.005 postconditions.

#[cfg(kani)]
mod kani_proofs {
    use crate::recovery::{advance_crash_counter, DirtyBitEntry, RecoveryAction};

    /// VP-057 — denylist threshold is exactly 3; idempotency holds.
    #[kani::proof]
    fn proof_denylist_threshold_three() {
        let consecutive_crashes: u32 = kani::any();
        let entry = DirtyBitEntry { consecutive_crashes };

        let action = advance_crash_counter(entry);

        // Threshold is `consecutive_crashes + 1 >= 3`, i.e., `>= 2`.
        if consecutive_crashes >= 2 {
            kani::assert(
                matches!(action, RecoveryAction::Denylist { expiry_seconds: 86400 }),
                "must denylist with 86400s expiry when consecutive_crashes >= 2",
            );
        } else {
            kani::assert(
                matches!(action, RecoveryAction::Warn),
                "must warn (not denylist) when consecutive_crashes < 2",
            );
        }

        // Idempotency: calling again produces the same action.
        let action2 = advance_crash_counter(entry);
        kani::assert(
            action == action2,
            "advance_crash_counter must be idempotent",
        );

        // Exact boundary: consecutive_crashes == 1 is Warn (2 total crashes).
        if consecutive_crashes == 1 {
            kani::assert(
                matches!(action, RecoveryAction::Warn),
                "exactly 2 crashes must not trigger denylist",
            );
        }

        // Exact boundary: consecutive_crashes == 2 is Denylist (3 total crashes).
        if consecutive_crashes == 2 {
            kani::assert(
                matches!(action, RecoveryAction::Denylist { .. }),
                "exactly 3 crashes must trigger denylist",
            );
        }
    }
}

// ── Unit tests for advance_crash_counter ─────────────────────────────────────
// These run under `cargo test`.  They call the unimplemented stub and MUST FAIL.

#[cfg(test)]
mod tests {
    use crate::recovery::{advance_crash_counter, DirtyBitEntry, RecoveryAction};

    #[test]
    fn test_BC_S_02_vp057_zero_crashes_returns_warn() {
        let entry = DirtyBitEntry { consecutive_crashes: 0 };
        assert_eq!(advance_crash_counter(entry), RecoveryAction::Warn);
    }

    #[test]
    fn test_BC_S_02_vp057_one_crash_returns_warn() {
        let entry = DirtyBitEntry { consecutive_crashes: 1 };
        assert_eq!(advance_crash_counter(entry), RecoveryAction::Warn);
    }

    #[test]
    fn test_BC_S_02_vp057_two_crashes_returns_denylist() {
        // consecutive_crashes == 2 means total = 3 → Denylist.
        let entry = DirtyBitEntry { consecutive_crashes: 2 };
        assert_eq!(
            advance_crash_counter(entry),
            RecoveryAction::Denylist { expiry_seconds: 86400 }
        );
    }

    #[test]
    fn test_BC_S_02_vp057_three_crashes_returns_denylist() {
        let entry = DirtyBitEntry { consecutive_crashes: 3 };
        assert_eq!(
            advance_crash_counter(entry),
            RecoveryAction::Denylist { expiry_seconds: 86400 }
        );
    }

    #[test]
    fn test_BC_S_02_vp057_idempotent_same_entry_same_action() {
        let entry = DirtyBitEntry { consecutive_crashes: 5 };
        let a = advance_crash_counter(entry);
        let b = advance_crash_counter(entry);
        assert_eq!(a, b, "advance_crash_counter must be idempotent");
    }

    #[test]
    fn test_BC_S_02_vp057_denylist_expiry_is_exactly_86400() {
        let entry = DirtyBitEntry { consecutive_crashes: 2 };
        let action = advance_crash_counter(entry);
        assert_eq!(
            action,
            RecoveryAction::Denylist { expiry_seconds: 86400 },
            "denylist expiry must be exactly 86400 seconds (24 hours)"
        );
    }
}
