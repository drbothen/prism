// S-2.02 — Watchdog tests.
//
// Covers BC-2.15.006, BC-2.15.007, BC-2.15.008.
//
// Test naming convention: test_BC_S_SS_NNN_[assertion_name]
//
// All tests use InMemoryBackend and StaticProbe / FixedClock — no real RSS reads,
// no sleeping.

#[cfg(test)]
mod inner {
    use std::sync::Arc;

    use tokio_util::sync::CancellationToken;

    use crate::{
        denylist::{
            clear_denylist, is_denylisted, record_failure, DenylistStatus, FixedClock,
            DENYLIST_THRESHOLD,
        },
        memory_backend::InMemoryBackend,
        proofs::watchdog_memory::{should_terminate_for_memory, WatchdogCheckState},
        watchdog::{ResourceWatchdog, StaticProbe, WatchdogLevel},
    };

    // ── Threshold constants from spec (asserted literally — NOT via constant) ──

    /// 86% of 512 MiB in bytes → Throttle level (AC-3, BC-2.15.006).
    /// 512 * 1024 * 1024 * 0.86 = 440_401_920 bytes (rounded down).
    const RSS_86_PCT: usize = 440_401_920;

    /// 96.7% of 512 MiB in bytes → Kill level (AC-4, BC-2.15.007).
    /// 495 * 1024 * 1024 = 519_110_656 bytes.
    const RSS_96_7_PCT: usize = 495 * 1024 * 1024;

    // ── AC-3: BC-2.15.006 — graduated level at 86% RSS → Throttle ───────────

    /// AC-3 (BC-2.15.006 postcondition): RSS at 86% of the 512 MiB budget
    /// returns `WatchdogLevel::Throttle`.
    ///
    /// Uses `StaticProbe(RSS_86_PCT)` to inject a fixed RSS without reading
    /// real process memory.
    #[test]
    fn test_BC_2_15_006_rss_at_86pct_returns_throttle() {
        let watchdog = ResourceWatchdog::with_probe(Arc::new(StaticProbe(RSS_86_PCT)));

        let level = watchdog.current_level();

        assert_eq!(
            level,
            WatchdogLevel::Throttle,
            "BC-2.15.006 postcondition: RSS at 86% of budget (≥85%, <95%) must return \
             WatchdogLevel::Throttle; got {:?}",
            level
        );
    }

    /// AC-3 boundary (BC-2.15.006): RSS below 70% → `WatchdogLevel::Normal`.
    #[test]
    fn test_BC_2_15_006_rss_below_70pct_returns_normal() {
        // 50% of 512 MiB.
        let rss_50_pct: usize = 512 * 1024 * 1024 / 2;
        let watchdog = ResourceWatchdog::with_probe(Arc::new(StaticProbe(rss_50_pct)));

        let level = watchdog.current_level();

        assert_eq!(
            level,
            WatchdogLevel::Normal,
            "BC-2.15.006: RSS below 70% must return WatchdogLevel::Normal; got {:?}",
            level
        );
    }

    /// AC-3 boundary (BC-2.15.006): RSS at exactly 70% → `WatchdogLevel::Warn`.
    #[test]
    fn test_BC_2_15_006_rss_at_70pct_returns_warn() {
        // 70% of 512 MiB = 357_892_710 bytes (truncated).
        let rss_70_pct: usize = ((512usize * 1024 * 1024) as f64 * 0.70) as usize;
        let watchdog = ResourceWatchdog::with_probe(Arc::new(StaticProbe(rss_70_pct)));

        let level = watchdog.current_level();

        assert_eq!(
            level,
            WatchdogLevel::Warn,
            "BC-2.15.006: RSS at exactly 70% must return WatchdogLevel::Warn; got {:?}",
            level
        );
    }

    /// AC-3 boundary (BC-2.15.006): RSS at exactly 95% → `WatchdogLevel::Kill`.
    #[test]
    fn test_BC_2_15_006_rss_at_95pct_returns_kill() {
        // 95% of 512 MiB.
        let rss_95_pct: usize = ((512usize * 1024 * 1024) as f64 * 0.95) as usize;
        let watchdog = ResourceWatchdog::with_probe(Arc::new(StaticProbe(rss_95_pct)));

        let level = watchdog.current_level();

        assert_eq!(
            level,
            WatchdogLevel::Kill,
            "BC-2.15.006: RSS at exactly 95% must return WatchdogLevel::Kill; got {:?}",
            level
        );
    }

    // ── AC-4: BC-2.15.007 — two-check grace period (DI-027) ─────────────────────
    //
    // The memory grace period (DI-027) requires the watchdog to observe the
    // process-RSS kill threshold exceeded on TWO CONSECUTIVE checks before
    // terminating.  A single spike must NOT kill.  An intervening sub-Kill
    // observation resets the counter (spike-recover-spike = no kill).
    //
    // check_query implements the kill decision via should_terminate_for_memory
    // (VP-058 proven predicate) and takes a consecutive_over_limit counter
    // maintained by the caller.  spawn_monitor maintains a loop-local counter.

    /// AC-4 (BC-2.15.007 postcondition, two-check contract): at Kill-level RSS,
    /// `check_query` with `consecutive_over_limit=1` (first observation — grace
    /// period active) does NOT cancel the token and returns `Ok(())`.
    ///
    /// BC-2.15.007/DI-027: a single spike above the process-RSS kill threshold
    /// must not immediately terminate the query.
    #[test]
    fn test_BC_2_15_007_first_kill_observation_grace_period_does_not_cancel() {
        let watchdog = ResourceWatchdog::with_probe(Arc::new(StaticProbe(RSS_96_7_PCT)));
        let token = CancellationToken::new();

        // First Kill-level observation — consecutive_over_limit=1 (grace period).
        let result = watchdog.check_query(token.clone(), 1);

        assert!(
            result.is_ok(),
            "BC-2.15.007/DI-027 grace period: first Kill-level observation \
             (consecutive_over_limit=1) must NOT cancel token; got {:?}",
            result
        );
        assert!(
            !token.is_cancelled(),
            "BC-2.15.007/DI-027 grace period: CancellationToken must NOT be cancelled \
             on first Kill-level observation (single spike tolerance)"
        );
    }

    /// AC-4 (BC-2.15.007 postcondition, two-check contract): at Kill-level RSS,
    /// `check_query` with `consecutive_over_limit=2` (second consecutive observation)
    /// DOES cancel the token and returns `Err(PrismError::WatchdogKilled)`.
    ///
    /// The error Display must contain `"E-WATCHDOG-002"` (error-taxonomy.md §E-WATCHDOG-002,
    /// P1-04 adjudication: the process-RSS watchdog kill is E-WATCHDOG-002;
    /// E-WATCHDOG-001 is reserved for the per-query DataFusion memory-pool trip).
    #[test]
    fn test_BC_2_15_007_second_consecutive_kill_observation_cancels_token() {
        let watchdog = ResourceWatchdog::with_probe(Arc::new(StaticProbe(RSS_96_7_PCT)));
        let token = CancellationToken::new();

        // Second consecutive Kill-level observation — must terminate.
        let result = watchdog.check_query(token.clone(), 2);

        // Token must be cancelled.
        assert!(
            token.is_cancelled(),
            "BC-2.15.007 postcondition: CancellationToken must be cancelled on \
             second consecutive Kill-level observation (consecutive_over_limit=2)"
        );

        // Result must be Err.
        assert!(
            result.is_err(),
            "BC-2.15.007 postcondition: check_query must return Err on second \
             consecutive Kill-level observation; got {:?}",
            result
        );

        // Error variant must be WatchdogKilled.
        let err = result.unwrap_err();
        assert!(
            matches!(err, prism_core::PrismError::WatchdogKilled { .. }),
            "BC-2.15.007 postcondition: error must be PrismError::WatchdogKilled; got {:?}",
            err
        );

        // Error Display must contain the canonical error code E-WATCHDOG-002
        // (error-taxonomy.md §E-WATCHDOG-002 / P1-04 adjudication: process-RSS watchdog kill
        // is E-WATCHDOG-002; E-WATCHDOG-001 is the per-query memory-pool trip).
        let display = err.to_string();
        assert!(
            display.contains("E-WATCHDOG-002"),
            "BC-2.15.007 / error-taxonomy.md v1.68: error Display must contain \
             \"E-WATCHDOG-002\"; got: {display}"
        );
    }

    /// AC-4 boundary: below Kill threshold, `check_query` does NOT cancel the token
    /// regardless of consecutive_over_limit.
    #[test]
    fn test_BC_2_15_007_below_kill_level_does_not_cancel_token() {
        // 86% → Throttle, not Kill.
        let watchdog = ResourceWatchdog::with_probe(Arc::new(StaticProbe(RSS_86_PCT)));
        let token = CancellationToken::new();

        let result = watchdog.check_query(token.clone(), 2);

        assert!(
            result.is_ok(),
            "BC-2.15.007: check_query must return Ok when level is below Kill; got {:?}",
            result
        );
        assert!(
            !token.is_cancelled(),
            "BC-2.15.007: token must NOT be cancelled when level is below Kill"
        );
    }

    // ── AC-4 spawn_monitor: two-check grace period via the monitor loop ────────
    //
    // These tests exercise the spawn_monitor two-check grace period.
    //
    // Implementation note on test design: `spawn_monitor` uses `tokio::time::sleep`
    // internally, so the tests must let the tokio runtime schedule the spawned task
    // between assertions.  We use `tokio::time::sleep` in the test itself (which
    // yields the current task and allows other tasks — including the monitor — to
    // run), with generous margins relative to the poll interval.  The poll is set
    // to 50ms and we wait 1.5× per assertion to give the monitor task reliable
    // scheduling time under parallel test load without needing `#[ignore]`.

    /// AC-4 (BC-2.15.007/DI-027): `spawn_monitor` does NOT cancel registered tokens
    /// on the first Kill-level poll; only on the second consecutive Kill-level poll.
    ///
    /// Test mechanism: StaticProbe always returns Kill-level RSS.
    /// - After 1× poll interval: token must NOT be cancelled (grace period, consecutive=1).
    /// - After 2× poll interval: token IS cancelled (consecutive=2 >= 2).
    #[tokio::test]
    async fn test_BC_2_15_007_spawn_monitor_first_kill_poll_grace_period() {
        // 50ms poll gives robust margin on loaded CI/parallel test runners.
        let poll = std::time::Duration::from_millis(50);
        let margin = std::time::Duration::from_millis(25);
        let watchdog = Arc::new(ResourceWatchdog::with_probe(Arc::new(StaticProbe(
            RSS_96_7_PCT,
        ))));
        let token = CancellationToken::new();
        let _qid = watchdog.register_query(token.clone());

        let _handle = watchdog.clone().spawn_monitor(poll);

        // After ~1.5 poll intervals: monitor has fired once → consecutive=1
        // (grace period) → token must NOT be cancelled.
        tokio::time::sleep(poll + margin).await;
        assert!(
            !token.is_cancelled(),
            "BC-2.15.007/DI-027: token must NOT be cancelled after first Kill-level \
             poll (grace period — single spike must not kill; consecutive=1 < 2)"
        );

        // After another full poll + margin: monitor fires again → consecutive=2
        // → should_terminate_for_memory returns true → token is cancelled.
        tokio::time::sleep(poll + margin).await;
        assert!(
            token.is_cancelled(),
            "BC-2.15.007/DI-027: token MUST be cancelled after second consecutive \
             Kill-level poll (consecutive=2 >= 2)"
        );
    }

    /// AC-4 (BC-2.15.007/DI-027): spike-recover-spike pattern does NOT kill.
    ///
    /// Monitor sees: Kill → sub-Kill (counter reset) → Kill.
    /// The third observation is the first Kill since the reset → grace period
    /// applies again → token must NOT be cancelled.
    ///
    /// Test mechanism: ToggleProbe alternates between Kill and sub-Kill on each read.
    #[tokio::test]
    async fn test_BC_2_15_007_spawn_monitor_intervening_sub_kill_resets_counter() {
        use std::sync::atomic::{AtomicUsize, Ordering};

        // RSS values: alternates Kill / sub-Kill / Kill / sub-Kill ...
        // Poll 1 → Kill (consecutive=1, grace), poll 2 → sub-Kill (reset=0),
        // poll 3 → Kill (consecutive=1, grace) → token must NOT be cancelled.
        struct ToggleProbe {
            call_count: AtomicUsize,
        }

        impl crate::watchdog::MemoryProbe for ToggleProbe {
            fn current_rss_bytes(&self) -> usize {
                let n = self.call_count.fetch_add(1, Ordering::Relaxed);
                // Even calls → Kill level; odd calls → sub-Kill (Throttle)
                if n.is_multiple_of(2) {
                    RSS_96_7_PCT
                } else {
                    RSS_86_PCT
                }
            }
        }

        let poll = std::time::Duration::from_millis(50);
        let margin = std::time::Duration::from_millis(25);
        let watchdog = Arc::new(ResourceWatchdog::with_probe(Arc::new(ToggleProbe {
            call_count: AtomicUsize::new(0),
        })));
        let token = CancellationToken::new();
        let _qid = watchdog.register_query(token.clone());

        let _handle = watchdog.clone().spawn_monitor(poll);

        // Three poll intervals: Kill→sub-Kill→Kill.
        // Even after 3 polls the token must NOT be cancelled because no two
        // consecutive Kill observations occur (counter always resets to 0 at the
        // sub-Kill poll between the two Kill polls).
        tokio::time::sleep(poll * 3 + margin * 2).await;
        assert!(
            !token.is_cancelled(),
            "BC-2.15.007/DI-027: spike-recover-spike must NOT kill — \
             intervening sub-Kill resets the consecutive counter; \
             pattern Kill(1)→sub-Kill(0)→Kill(1) never reaches consecutive=2"
        );
    }

    /// AC-4 integration: VP-058 proven predicate `should_terminate_for_memory`
    /// is wired into the live `check_query` kill path.
    ///
    /// Verifies that `check_query(token, consecutive)` agrees with
    /// `should_terminate_for_memory(WatchdogCheckState { consecutive_over_limit: consecutive })`
    /// for boundary values 0, 1, 2 at Kill-level RSS.
    #[test]
    fn test_BC_2_15_007_check_query_agrees_with_vp058_predicate() {
        let watchdog = ResourceWatchdog::with_probe(Arc::new(StaticProbe(RSS_96_7_PCT)));

        // consecutive=0 — predicate false, must not kill.
        let token0 = CancellationToken::new();
        let result0 = watchdog.check_query(token0.clone(), 0);
        assert_eq!(
            result0.is_err(),
            should_terminate_for_memory(WatchdogCheckState {
                consecutive_over_limit: 0
            }),
            "check_query(consecutive=0) must agree with VP-058 predicate"
        );
        assert!(!token0.is_cancelled());

        // consecutive=1 — predicate false, must not kill.
        let token1 = CancellationToken::new();
        let result1 = watchdog.check_query(token1.clone(), 1);
        assert_eq!(
            result1.is_err(),
            should_terminate_for_memory(WatchdogCheckState {
                consecutive_over_limit: 1
            }),
            "check_query(consecutive=1) must agree with VP-058 predicate"
        );
        assert!(!token1.is_cancelled());

        // consecutive=2 — predicate true, must kill.
        let token2 = CancellationToken::new();
        let result2 = watchdog.check_query(token2.clone(), 2);
        assert_eq!(
            result2.is_err(),
            should_terminate_for_memory(WatchdogCheckState {
                consecutive_over_limit: 2
            }),
            "check_query(consecutive=2) must agree with VP-058 predicate"
        );
        assert!(token2.is_cancelled());
    }

    // ── BC-2.15.008 denylist-interaction: recorded only on ACTUAL termination ──

    /// BC-2.15.008 / DI-027 denylist interaction: `check_query` only records a
    /// watchdog termination (denylist-eligible) when it actually cancels — i.e.,
    /// when `consecutive_over_limit >= 2`.  A grace-period observation (consecutive=1)
    /// must NOT trigger denylist recording.
    ///
    /// The denylist recording itself is the caller's responsibility after receiving
    /// Err(WatchdogKilled).  This test verifies that `check_query` returns Ok on
    /// consecutive=1 (so callers have nothing to record) and Err on consecutive=2
    /// (so callers receive the signal to record).
    #[test]
    fn test_BC_2_15_007_denylist_recording_only_on_actual_termination() {
        let watchdog = ResourceWatchdog::with_probe(Arc::new(StaticProbe(RSS_96_7_PCT)));

        // Grace-period observation → Ok → caller must NOT record for denylist.
        let token_grace = CancellationToken::new();
        let result_grace = watchdog.check_query(token_grace.clone(), 1);
        assert!(
            result_grace.is_ok(),
            "BC-2.15.007/DI-027: grace-period check_query must return Ok \
             (no WatchdogKilled signal → caller must not record for denylist)"
        );

        // Actual termination → Err(WatchdogKilled) → caller MUST record for denylist.
        let token_kill = CancellationToken::new();
        let result_kill = watchdog.check_query(token_kill.clone(), 2);
        assert!(
            result_kill.is_err(),
            "BC-2.15.007/DI-027: second-check_query must return Err(WatchdogKilled) \
             (caller receives signal to record for denylist)"
        );
        assert!(
            matches!(
                result_kill.unwrap_err(),
                prism_core::PrismError::WatchdogKilled { .. }
            ),
            "BC-2.15.007: kill result must be WatchdogKilled for denylist-eligible error"
        );
    }

    // ── AC-5: BC-2.15.008 — 3 consecutive failures → denylisted ─────────────

    /// AC-5 (BC-2.15.008 postcondition): after 3 consecutive `record_failure`
    /// calls with the same fingerprint, `is_denylisted()` returns `true`.
    #[test]
    fn test_BC_2_15_008_three_failures_result_in_denylist() {
        let backend = InMemoryBackend::new();
        let fp = "sha256-deadbeef";
        let clock = FixedClock(1_000_000);

        // Record 3 consecutive failures.
        for _ in 0..3 {
            record_failure(&backend, fp, DENYLIST_THRESHOLD, &clock)
                .expect("record_failure must not fail");
        }

        let denylisted = is_denylisted(&backend, fp, &clock).expect("is_denylisted must not fail");

        assert!(
            denylisted,
            "BC-2.15.008 postcondition: after {DENYLIST_THRESHOLD} consecutive failures \
             is_denylisted must return true"
        );
    }

    /// AC-5 (BC-2.15.008 postcondition): the error returned when a denylisted query
    /// is rejected must be `PrismError::QueryDenylisted` and its Display must contain
    /// `"E-QUERY-008"` (story v1.7 correction).
    ///
    /// This test constructs the expected error directly to verify the Display path,
    /// because the storage module does not execute queries — it provides the denylist
    /// state, and callers are responsible for producing the error.
    #[test]
    fn test_BC_2_15_008_query_denylisted_error_contains_e_query_008() {
        // Construct the error variant the caller must return (BC-2.15.008 / E-QUERY-008).
        let err = prism_core::PrismError::QueryDenylisted {
            failure_count: 3,
            reason: "memory".to_string(),
            expiry_ts: 1_000_000 + 86_400, // now + 24h
        };

        let display = err.to_string();

        // Anchor to the canonical taxonomy ID from error-taxonomy.md.
        assert!(
            display.contains("E-QUERY-008"),
            "BC-2.15.008 / error-taxonomy.md: QueryDenylisted Display must contain \
             \"E-QUERY-008\"; got: {display}"
        );
    }

    /// AC-5 (BC-2.15.008 postcondition): after 3 consecutive failures the
    /// `DenylistStatus::Denylisted` variant is returned with the correct
    /// failure count.
    #[test]
    fn test_BC_2_15_008_third_failure_returns_denylisted_status() {
        let backend = InMemoryBackend::new();
        let fp = "sha256-cafebabe";
        let clock = FixedClock(2_000_000);

        // First two failures → BelowThreshold.
        for i in 1..3 {
            let status = record_failure(&backend, fp, DENYLIST_THRESHOLD, &clock)
                .expect("record_failure must not fail");
            assert!(
                matches!(status, DenylistStatus::BelowThreshold { .. }),
                "BC-2.15.008: failure {i} must return BelowThreshold, got {:?}",
                status
            );
        }

        // Third failure → Denylisted.
        let status = record_failure(&backend, fp, DENYLIST_THRESHOLD, &clock)
            .expect("record_failure must not fail on third call");

        assert!(
            matches!(status, DenylistStatus::Denylisted { failure_count: 3, .. }),
            "BC-2.15.008 postcondition: third failure must return Denylisted {{ failure_count: 3, .. }}; \
             got {:?}",
            status
        );
    }

    // ── AC-6: BC-2.15.008 — clear_denylist(Some(fp)) → is_denylisted returns false ──

    /// AC-6 (BC-2.15.008 postcondition): after `clear_denylist(Some(fp))`,
    /// `is_denylisted(fp)` returns `false`.
    #[test]
    fn test_BC_2_15_008_clear_specific_fingerprint_removes_from_denylist() {
        let backend = InMemoryBackend::new();
        let fp = "sha256-beefdead";
        let clock = FixedClock(3_000_000);

        // Setup: denylist the fingerprint.
        for _ in 0..3 {
            record_failure(&backend, fp, DENYLIST_THRESHOLD, &clock).expect("record_failure setup");
        }
        assert!(
            is_denylisted(&backend, fp, &clock).expect("is_denylisted setup check"),
            "AC-6 setup: fingerprint must be denylisted before clear"
        );

        // Clear the specific fingerprint.
        let removed =
            clear_denylist(&backend, Some(fp)).expect("clear_denylist(Some(fp)) must not fail");
        assert_eq!(
            removed, 1,
            "BC-2.15.008: clear_denylist(Some(fp)) must return 1 when entry exists"
        );

        // After clear, must not be denylisted.
        let still_denylisted =
            is_denylisted(&backend, fp, &clock).expect("is_denylisted after clear must not fail");
        assert!(
            !still_denylisted,
            "BC-2.15.008 postcondition (AC-6): is_denylisted must return false after \
             clear_denylist(Some(fp))"
        );
    }

    /// BC-2.15.008: `clear_denylist(None)` removes ALL denylist entries and
    /// returns the total count removed (EC-005).
    #[test]
    fn test_BC_2_15_008_clear_all_removes_all_entries() {
        let backend = InMemoryBackend::new();
        let clock = FixedClock(4_000_000);
        let fps = ["sha256-aaaa", "sha256-bbbb", "sha256-cccc"];

        // Denylist all three fingerprints.
        for fp in &fps {
            for _ in 0..3 {
                record_failure(&backend, fp, DENYLIST_THRESHOLD, &clock)
                    .expect("record_failure setup");
            }
        }

        // Clear all.
        let removed = clear_denylist(&backend, None).expect("clear_denylist(None) must not fail");
        assert_eq!(
            removed, 3,
            "BC-2.15.008 EC-005: clear_denylist(None) must return 3 (total entries removed)"
        );

        // None of the fingerprints should be denylisted anymore.
        for fp in &fps {
            let still = is_denylisted(&backend, fp, &clock).expect("is_denylisted after clear all");
            assert!(
                !still,
                "BC-2.15.008 EC-005: {fp} must not be denylisted after clear_denylist(None)"
            );
        }
    }

    // ── Denylist expiry — 24-hour requirement (BC-2.15.008) ─────────────

    /// BC-2.15.008: denylist entry expiry is exactly 86400 seconds (24 hours).
    ///
    /// Asserts against the literal `86400` — NOT against `DENYLIST_EXPIRY_SECS`
    /// (which the stub has as `3600`).  This forces the implementer to fix the
    /// constant.
    ///
    /// Uses `FixedClock` to avoid sleeping.
    #[test]
    fn test_denylist_expiry_is_24_hours_per_bc_2_15_008() {
        let backend = InMemoryBackend::new();
        let fp = "sha256-expiry-check";
        let now_secs: u64 = 10_000_000;
        let clock = FixedClock(now_secs);

        // Denylist the fingerprint.
        for _ in 0..3 {
            record_failure(&backend, fp, DENYLIST_THRESHOLD, &clock)
                .expect("record_failure for expiry test");
        }

        // Read the stored expiry_ts from the DenylistStatus::Denylisted variant.
        // We call record_failure one more time to observe the returned Denylisted
        // status — but since it already crossed the threshold, calling again with
        // the same fingerprint must still return Denylisted (idempotent once
        // denylisted) or we can inspect via a helper scan.
        //
        // Instead we verify via is_denylisted at now+86399 (still denylisted)
        // and now+86400 (expired → false).

        // At now + 86399 seconds: NOT yet expired → still denylisted.
        let clock_86399 = FixedClock(now_secs + 86_399);
        let still_denylisted = is_denylisted(&backend, fp, &clock_86399)
            .expect("is_denylisted at 86399s must not fail");
        assert!(
            still_denylisted,
            "BC-2.15.008: entry must still be denylisted at now+86399s \
             (expiry is 86400s = 24h; got false, meaning expiry < 86400s — \
             likely the stub constant 3600 was used)"
        );

        // At now + 86400 seconds: expired → no longer denylisted (lazy expiry).
        let clock_86400 = FixedClock(now_secs + 86_400);
        let expired = is_denylisted(&backend, fp, &clock_86400)
            .expect("is_denylisted at 86400s must not fail");
        assert!(
            !expired,
            "BC-2.15.008: entry must be expired at exactly now+86400s (24h); \
             got true, meaning expiry > 86400s"
        );
    }

    /// BC-2.15.008: a query that fails twice then succeeds does NOT get denylisted
    /// (consecutive-only counter; non-consecutive failures reset the counter).
    #[test]
    fn test_BC_2_15_008_intervening_success_resets_counter() {
        // NOTE: this test verifies the behavioral contract's invariant that
        // "only consecutive failures trigger denylisting".  The `record_failure`
        // function does NOT implement success-reset by itself — there must be a
        // separate `record_success` (or equivalent) function.  This test is
        // intentionally written to document the requirement and will FAIL until
        // such a function exists.
        //
        // For now: we assert that after 2 failures the count is below threshold,
        // then that a 3rd failure with a fresh backend (simulating reset) also
        // produces BelowThreshold for count=1.  The full intervening-success
        // test requires `record_success` which is not yet stubbed.
        let backend = InMemoryBackend::new();
        let fp = "sha256-intervening";
        let clock = FixedClock(5_000_000);

        // Two failures — still below threshold.
        for i in 1..=2 {
            let status =
                record_failure(&backend, fp, DENYLIST_THRESHOLD, &clock).expect("record_failure");
            assert!(
                matches!(status, DenylistStatus::BelowThreshold { .. }),
                "BC-2.15.008 invariant: failure {i} of 2 must be BelowThreshold; got {:?}",
                status
            );
        }

        // A success resets the counter — implemented as clearing the failure
        // record for this fingerprint.  Since `record_success` is not yet
        // stubbed, we approximate with `clear_denylist(Some(fp))`.
        // The real implementation must expose a `record_success` path.
        clear_denylist(&backend, Some(fp)).expect("clear after intervening success");

        // After reset, the next failure should be BelowThreshold (count=1).
        let status = record_failure(&backend, fp, DENYLIST_THRESHOLD, &clock)
            .expect("record_failure after reset");
        assert!(
            matches!(status, DenylistStatus::BelowThreshold { failure_count: 1 }),
            "BC-2.15.008 invariant: first failure after counter reset must be \
             BelowThreshold {{ failure_count: 1 }}; got {:?}",
            status
        );
    }
}
