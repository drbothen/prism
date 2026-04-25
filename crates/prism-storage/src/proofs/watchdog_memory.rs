// VP-058: Watchdog Memory Grace Period — Two-Check Policy.
//
// Proves BC-2.15.007 postcondition: `should_terminate_for_memory(state)` returns
// `true` if and only if `state.consecutive_over_limit >= 2`.
//
// A single check with memory above the limit does NOT terminate (grace period,
// DI-027).  Two consecutive checks DO terminate.  Threshold is exactly 2.
//
// Method: proptest over the full u8 range of `consecutive_over_limit`.
// Traces to: BC-2.15.007 postconditions, AC-7.

/// State passed to `should_terminate_for_memory` to determine whether the
/// watchdog should cancel the query for exceeding the per-query memory limit.
///
/// The `consecutive_over_limit` counter is incremented by the watchdog background
/// task each polling interval (500 ms) while the query's estimated memory usage
/// exceeds `per_query_memory_budget`.  It is reset to 0 when usage drops below
/// the budget.
///
/// Threshold: exactly 2 (VP-058 / BC-2.15.007 DI-027 grace period).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WatchdogCheckState {
    /// Number of consecutive watchdog checks where per-query memory exceeded budget.
    ///
    /// `u8` is sufficient: the watchdog reacts within two checks (1 second at
    /// the 500 ms poll interval); values > 255 are structurally impossible.
    pub consecutive_over_limit: u8,
}

/// Returns `true` if and only if `state.consecutive_over_limit >= 2`.
///
/// This is the pure predicate extracted from the watchdog loop per VP-058 and
/// BC-2.15.007.  It is separate from `ResourceWatchdog` so proptest can call
/// it in isolation without async or I/O.
///
/// # Correctness invariant (VP-058)
///
/// - `consecutive_over_limit == 0` → `false` (no violation observed)
/// - `consecutive_over_limit == 1` → `false` (grace period; single spike tolerated)
/// - `consecutive_over_limit >= 2` → `true`  (two consecutive checks; terminate)
pub fn should_terminate_for_memory(_state: WatchdogCheckState) -> bool {
    // VP-058 property: threshold is exactly 2 (not 1, not 3)
    // AC-7: proptest asserts this for all u8 values
    todo!("VP-058 / BC-2.15.007 postcondition: return state.consecutive_over_limit >= 2")
}

// ── VP-058 proptest harness ───────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::{should_terminate_for_memory, WatchdogCheckState};

    // ── AC-7: proptest — terminate iff consecutive_over_limit >= 2 ───────────

    // AC-7 (VP-058): `should_terminate_for_memory` returns `true` iff
    // `consecutive_over_limit >= 2` for all values in the range 0..=5.
    // Named per BC-ID naming convention. Threshold is exactly 2.
    proptest! {
        #[test]
        fn test_BC_2_15_007_VP058_terminate_iff_consecutive_over_limit_gte_2(
            consecutive_over_limit in 0u8..=5u8,
        ) {
            let state = WatchdogCheckState { consecutive_over_limit };
            let result = should_terminate_for_memory(state);

            if consecutive_over_limit >= 2 {
                prop_assert!(
                    result,
                    "VP-058: should_terminate_for_memory must return true when \
                     consecutive_over_limit={consecutive_over_limit} (>= 2)"
                );
            } else {
                prop_assert!(
                    !result,
                    "VP-058: should_terminate_for_memory must return false when \
                     consecutive_over_limit={consecutive_over_limit} (< 2, grace period)"
                );
            }
        }
    }

    // VP-058 boundary: the full u8 range (0..=255) is covered.
    // proptest generates random u8 values providing broad coverage.
    proptest! {
        #[test]
        fn test_BC_2_15_007_VP058_full_u8_range(
            consecutive_over_limit in 0u8..=u8::MAX,
        ) {
            let state = WatchdogCheckState { consecutive_over_limit };
            let result = should_terminate_for_memory(state);

            if consecutive_over_limit >= 2 {
                prop_assert!(
                    result,
                    "VP-058 full range: must terminate at consecutive_over_limit={consecutive_over_limit}"
                );
            } else {
                prop_assert!(
                    !result,
                    "VP-058 full range: must NOT terminate at consecutive_over_limit={consecutive_over_limit}"
                );
            }
        }
    }

    // ── Explicit boundary tests (AC-7 spec literals) ──────────────────────────

    /// VP-058 boundary: single over-limit check (consecutive=1) does NOT terminate.
    ///
    /// This is the grace period defined in DI-027 — one transient spike is tolerated.
    #[test]
    fn test_BC_2_15_007_VP058_single_check_does_not_terminate() {
        let single_check = WatchdogCheckState {
            consecutive_over_limit: 1,
        };
        assert!(
            !should_terminate_for_memory(single_check),
            "VP-058 boundary: single check above limit (consecutive=1) must NOT terminate \
             (grace period, DI-027)"
        );
    }

    /// VP-058 boundary: two consecutive checks (consecutive=2) DO terminate.
    #[test]
    fn test_BC_2_15_007_VP058_two_consecutive_checks_terminate() {
        let two_checks = WatchdogCheckState {
            consecutive_over_limit: 2,
        };
        assert!(
            should_terminate_for_memory(two_checks),
            "VP-058 boundary: two consecutive checks above limit (consecutive=2) must terminate"
        );
    }

    /// VP-058 boundary: zero checks (consecutive=0) does NOT terminate.
    #[test]
    fn test_BC_2_15_007_VP058_zero_checks_does_not_terminate() {
        let zero_checks = WatchdogCheckState {
            consecutive_over_limit: 0,
        };
        assert!(
            !should_terminate_for_memory(zero_checks),
            "VP-058 boundary: zero checks above limit (consecutive=0) must NOT terminate"
        );
    }

    /// VP-058: threshold is exactly 2 — not 1 (too eager) and not 3 (too lenient).
    ///
    /// This test explicitly verifies the boundaries adjacent to the threshold to
    /// rule out off-by-one implementations.
    #[test]
    fn test_BC_2_15_007_VP058_threshold_is_exactly_2() {
        // Below threshold: 0 and 1 must be false.
        assert!(
            !should_terminate_for_memory(WatchdogCheckState {
                consecutive_over_limit: 0
            }),
            "VP-058: threshold must be exactly 2; consecutive=0 must return false"
        );
        assert!(
            !should_terminate_for_memory(WatchdogCheckState {
                consecutive_over_limit: 1
            }),
            "VP-058: threshold must be exactly 2; consecutive=1 must return false \
             (would fail if threshold were 1)"
        );

        // At threshold: 2 must be true.
        assert!(
            should_terminate_for_memory(WatchdogCheckState {
                consecutive_over_limit: 2
            }),
            "VP-058: threshold must be exactly 2; consecutive=2 must return true"
        );

        // Above threshold: 3, 4, 5 must also be true.
        for n in 3u8..=5u8 {
            assert!(
                should_terminate_for_memory(WatchdogCheckState {
                    consecutive_over_limit: n
                }),
                "VP-058: consecutive={n} must return true (>= 2 threshold)"
            );
        }
    }
}
