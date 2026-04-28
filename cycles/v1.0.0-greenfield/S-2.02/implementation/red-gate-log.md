---
document_type: red-gate-log
level: ops
version: "1.0"
status: complete
producer: test-writer
timestamp: 2026-04-25T00:00:00Z
phase: 3
inputs:
  - .factory/stories/S-2.02-audit-buffer-watchdog.md
  - .factory/specs/behavioral-contracts/BC-2.15.003-buffered-audit-log-persistence.md
  - .factory/specs/behavioral-contracts/BC-2.15.004-audit-buffer-overflow.md
  - .factory/specs/behavioral-contracts/BC-2.15.006-resource-watchdog-initialization.md
  - .factory/specs/behavioral-contracts/BC-2.15.007-watchdog-query-termination.md
  - .factory/specs/behavioral-contracts/BC-2.15.008-query-denylisting.md
  - .factory/specs/verification-properties/vp-058-watchdog-memory-grace-period-two-check-policy.md
input-hash: "9198fe0"
traces_to: .factory/stories/S-2.02-audit-buffer-watchdog.md
stub_architect_agent: "[stub commit 2eb9decd]"
stub_compile_verified: true
test_writer_agent: "[test-writer dispatch 2026-04-25]"
red_gate_verified: true
---

# Red Gate Log: S-2.02 — prism-storage: Audit Buffer and Watchdog

## Summary

| Story | Tests Written | All Fail (Red)? | Gate |
|-------|--------------|-----------------|------|
| S-2.02 | 25 | YES — all 25 fail for correct reasons | PASS |

Pre-existing tests: **24/24 integration tests pass** (unaffected).

## Test-Driven Design Decisions

### TDD-001: MemoryProbe trait introduction

**Problem:** The stub `ResourceWatchdog::current_level()` reads live process RSS via
`sysinfo`. Tests for AC-3 (86% → Throttle) and AC-4 (96% → Kill) cannot control the
RSS value in CI without risking flakiness.

**Decision:** Introduce `MemoryProbe` trait with two implementations:
- `SysinfoProbe` — production impl (reads real RSS via `sysinfo`); `todo!()` stub
- `StaticProbe(usize)` — test impl; returns fixed bytes

Add `ResourceWatchdog::with_probe(Arc<dyn MemoryProbe>)` constructor for tests.
`ResourceWatchdog::new()` uses `SysinfoProbe` (no change to public API).

**Rationale:** The Architecture Compliance Rule requires `sysinfo` for memory
measurement but does not preclude injection of a probe interface. The `todo!()`
body of `current_level()` in the stub means no production logic was modified.

**Files changed:** `crates/prism-storage/src/watchdog.rs`

### TDD-002: ClockProbe trait introduction

**Problem:** BC-2.15.008 v1.7 requires denylist expiry at exactly 86400 seconds.
Testing this without a clock seam requires a 24-hour sleep, which is not acceptable.

**Decision:** Introduce `ClockProbe` trait with two implementations:
- `SystemClock` — production impl (reads `SystemTime::now()`)
- `FixedClock(u64)` — test impl; returns a fixed Unix second

Modify `record_failure` and `is_denylisted` to accept `&dyn ClockProbe`.
`clear_denylist` does not use the clock and is unchanged in signature.

**Rationale:** Same pattern as `MemoryProbe`. Enables the `test_denylist_expiry_is_24_hours_per_bc_2_15_008` test to advance the clock by 86399 and 86400 seconds without sleeping. The `todo!()` stubs are unchanged in their implementations.

**Files changed:** `crates/prism-storage/src/denylist.rs`

### TDD-003: Story v1.7 error code corrections anchored to literals

**Problem:** The `error.rs` Display strings say `E-WATCH-001` (for `WatchdogKilled`)
and `E-WATCH-002` (for `QueryDenylisted`). Story v1.7 corrected these to
`E-WATCHDOG-001` and `E-QUERY-008` per `error-taxonomy.md`.

**Decision:** Tests assert against string literals `"E-WATCHDOG-001"` and
`"E-QUERY-008"` — NOT against `DENYLIST_EXPIRY_SECS` or any other constant.
This forces the implementer to update the Display attributes in `error.rs`.

**Rationale:** The task description explicitly states: "anchor the test to the literal
taxonomy ID, not just the variant." The assertion failure message clearly shows the
current wrong value: `"E-WATCH-002: query denylisted after 3..."`.

### TDD-004: DENYLIST_EXPIRY_SECS constant anchored to 86400 literal

**Problem:** The stub has `DENYLIST_EXPIRY_SECS = 3600` (1 hour, from old story v1.6).
Story v1.7 requires 86400 seconds (24 hours).

**Decision:** The `test_denylist_expiry_is_24_hours_per_bc_2_15_008` test uses
`FixedClock` to assert that an entry is still denylisted at `now + 86_399` and
expired at `now + 86_400`. It does NOT reference `DENYLIST_EXPIRY_SECS`. This
forces the implementer to change the constant from 3600 to 86400.

## Stubs Created

Production stubs unchanged. New test-infrastructure stubs:

- `trait MemoryProbe: Send + Sync { fn current_rss_bytes(&self) -> usize; }` — probe seam
- `struct SysinfoProbe` — `impl MemoryProbe` — `todo!()` body (production RSS read)
- `struct StaticProbe(pub usize)` — `impl MemoryProbe` — returns `self.0` (test use)
- `fn ResourceWatchdog::with_probe(probe: Arc<dyn MemoryProbe>) -> Self` — test constructor
- `trait ClockProbe { fn unix_secs(&self) -> u64; }` — clock seam
- `struct SystemClock` — `impl ClockProbe` — reads `SystemTime::now()` (production)
- `struct FixedClock(pub u64)` — `impl ClockProbe` — returns `self.0` (test use)

## Red Gate Verification

### S-2.02 — Audit Buffer Tests (BC-2.15.003 / BC-2.15.004)

| AC | Test Name | File | Failure | Reason |
|----|-----------|------|---------|--------|
| AC-1 | `test_BC_2_15_003_entry_persisted_before_forwarding` | `audit_buffer_tests.rs` | `panicked at audit_buffer.rs:62` | `todo!()` in `append_audit_entry` |
| AC-1 | `test_BC_2_15_003_entries_lex_ordered_by_timestamp` | `audit_buffer_tests.rs` | `panicked at audit_buffer.rs:62` | `todo!()` in `append_audit_entry` |
| AC-1 | `test_BC_2_15_003_entries_survive_simulated_restart` | `audit_buffer_tests.rs` | `panicked at audit_buffer.rs:62` | `todo!()` in `append_audit_entry` |
| AC-2 | `test_BC_2_15_004_overflow_purges_to_target` | `audit_buffer_tests.rs` | `panicked at audit_buffer.rs:62` | `todo!()` in `append_audit_entry` (called during setup) |
| AC-2 | `test_BC_2_15_004_no_purge_below_threshold` | `audit_buffer_tests.rs` | `panicked at audit_buffer.rs:62` | `todo!()` in `append_audit_entry` |
| AC-2 | `test_BC_2_15_004_purge_removes_oldest_entries` | `audit_buffer_tests.rs` | `panicked at audit_buffer.rs:62` | `todo!()` in `append_audit_entry` |

### S-2.02 — Watchdog Tests (BC-2.15.006 / BC-2.15.007 / BC-2.15.008)

| AC | Test Name | File | Failure | Reason |
|----|-----------|------|---------|--------|
| AC-3 | `test_BC_2_15_006_rss_at_86pct_returns_throttle` | `watchdog_tests.rs` | `panicked at watchdog.rs:210` | `todo!()` in `current_level()` |
| AC-3 | `test_BC_2_15_006_rss_below_70pct_returns_normal` | `watchdog_tests.rs` | `panicked at watchdog.rs:210` | `todo!()` in `current_level()` |
| AC-3 | `test_BC_2_15_006_rss_at_70pct_returns_warn` | `watchdog_tests.rs` | `panicked at watchdog.rs:210` | `todo!()` in `current_level()` |
| AC-3 | `test_BC_2_15_006_rss_at_95pct_returns_kill` | `watchdog_tests.rs` | `panicked at watchdog.rs:210` | `todo!()` in `current_level()` |
| AC-4 | `test_BC_2_15_007_kill_level_cancels_token_and_returns_watchdog_killed` | `watchdog_tests.rs` | `panicked at watchdog.rs:247` | `todo!()` in `check_query()` |
| AC-4 | `test_BC_2_15_007_below_kill_level_does_not_cancel_token` | `watchdog_tests.rs` | `panicked at watchdog.rs:247` | `todo!()` in `check_query()` |
| AC-5 | `test_BC_2_15_008_three_failures_result_in_denylist` | `watchdog_tests.rs` | `panicked at denylist.rs:142` | `todo!()` in `record_failure()` |
| AC-5 | `test_BC_2_15_008_query_denylisted_error_contains_e_query_008` | `watchdog_tests.rs` | `assertion failed: display.contains("E-QUERY-008"); got: "E-WATCH-002: query denylisted..."` | **Correct assertion failure** — forces implementer to update `error.rs` Display string |
| AC-5 | `test_BC_2_15_008_third_failure_returns_denylisted_status` | `watchdog_tests.rs` | `panicked at denylist.rs:142` | `todo!()` in `record_failure()` |
| AC-6 | `test_BC_2_15_008_clear_specific_fingerprint_removes_from_denylist` | `watchdog_tests.rs` | `panicked at denylist.rs:142` | `todo!()` in `record_failure()` (setup) |
| AC-6 | `test_BC_2_15_008_clear_all_removes_all_entries` | `watchdog_tests.rs` | `panicked at denylist.rs:142` | `todo!()` in `record_failure()` (setup) |
| — | `test_BC_2_15_008_intervening_success_resets_counter` | `watchdog_tests.rs` | `panicked at denylist.rs:142` | `todo!()` in `record_failure()` |
| — | `test_denylist_expiry_is_24_hours_per_bc_2_15_008` | `watchdog_tests.rs` | `panicked at denylist.rs:142` | `todo!()` in `record_failure()` |

### S-2.02 — VP-058 Proptest (BC-2.15.007 / AC-7)

| VP | Test Name | File | Failure | Reason |
|----|-----------|------|---------|--------|
| VP-058 | `test_BC_2_15_007_VP058_terminate_iff_consecutive_over_limit_gte_2` | `watchdog_memory.rs` | `panicked at watchdog_memory.rs:44` | `todo!()` in `should_terminate_for_memory()` |
| VP-058 | `test_BC_2_15_007_VP058_full_u8_range` | `watchdog_memory.rs` | `panicked at watchdog_memory.rs:44` | `todo!()` in `should_terminate_for_memory()` |
| VP-058 | `test_BC_2_15_007_VP058_single_check_does_not_terminate` | `watchdog_memory.rs` | `panicked at watchdog_memory.rs:44` | `todo!()` in `should_terminate_for_memory()` |
| VP-058 | `test_BC_2_15_007_VP058_two_consecutive_checks_terminate` | `watchdog_memory.rs` | `panicked at watchdog_memory.rs:44` | `todo!()` in `should_terminate_for_memory()` |
| VP-058 | `test_BC_2_15_007_VP058_zero_checks_does_not_terminate` | `watchdog_memory.rs` | `panicked at watchdog_memory.rs:44` | `todo!()` in `should_terminate_for_memory()` |
| VP-058 | `test_BC_2_15_007_VP058_threshold_is_exactly_2` | `watchdog_memory.rs` | `panicked at watchdog_memory.rs:44` | `todo!()` in `should_terminate_for_memory()` |

## Red Gate Result

```
test result: FAILED. 9 passed; 25 failed; 0 ignored; 0 measured; 0 filtered out
```

- **9 passing**: pre-existing proof tests (crash_recovery, storage_batch)
- **25 failing**: all new S-2.02 tests — each fails for the correct reason

## Regression Check

| Test Suite | Status |
|------------|--------|
| 24 pre-existing integration tests (`tests/integration.rs`) | **all pass** |
| New S-2.02 lib tests | 25 failing (correct) |

## Deviations from BC Text

None. All tests use the exact threshold values, error codes, and behaviors
specified in the BCs. The test-driven design decisions (MemoryProbe, ClockProbe)
add infrastructure around the stubs without modifying production logic.

**Notable spec defect corrected (story v1.7):** Tests assert `E-WATCHDOG-001`
and `E-QUERY-008` per the canonical `error-taxonomy.md`, not `E-WATCH-001` /
`E-WATCH-002` which appeared in the old story v1.6. The stub `error.rs` still
has the old codes — the assertion failures force the implementer to fix them.

## Hand-Off to Implementer

Stories ready for implementation: **S-2.02**

Implementation guidance:

1. **`should_terminate_for_memory`** — implement as `state.consecutive_over_limit >= 2`
   (one-liner, but must satisfy the VP-058 proptest over the full u8 range).

2. **`ResourceWatchdog::current_level`** — call `self.probe.current_rss_bytes()`,
   compare against `self.budget_bytes * self.{warn,throttle,kill}_pct`, return level.

3. **`ResourceWatchdog::check_query`** — call `self.current_level()`, if `Kill`
   call `cancel_token.cancel()` and return `Err(PrismError::WatchdogKilled { budget_bytes: self.budget_bytes })`.

4. **`append_audit_entry`** — build key `audit:{timestamp_ns:020}:{trace_id}`,
   bincode-encode the entry, call `backend.put(StorageDomain::AuditBuffer, key, value)`.

5. **`check_and_purge_overflow`** — scan `AuditBuffer` CF, if count > 100K delete
   the oldest (count - 90K) entries, return purged count.

6. **`record_failure` / `is_denylisted` / `clear_denylist`** — implement denylist
   CRUD in `StorageDomain::Watchdog` CF with lazy expiry.

7. **Fix `error.rs`**: change `WatchdogKilled` Display from `E-WATCH-001` → `E-WATCHDOG-001`;
   change `QueryDenylisted` Display from `E-WATCH-002` → `E-QUERY-008`.

8. **Fix `DENYLIST_EXPIRY_SECS`**: change from `3600` to `86400` (24 hours per BC-2.15.008 v1.7).
