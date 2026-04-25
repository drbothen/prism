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
    // Test bodies are written in the Test Writer dispatch (S-2.02 step b).
    // This mod declaration is the required empty placeholder.
}
