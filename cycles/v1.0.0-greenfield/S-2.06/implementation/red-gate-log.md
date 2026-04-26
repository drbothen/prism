# Red Gate Log — S-2.06 (DataSource Trait and Auth Patterns)

**Date:** 2026-04-25
**Step:** 3 — Test Writer
**Story version:** v1.5 (bc-alignment burst)
**Worktree:** `.worktrees/S-2.06-datasource-trait` @ commit `1e26701b`
**Stub baseline commit:** `e86d03f2` (generated against v1.3)

---

## Summary

51 tests written across 5 modules. Red Gate confirmed: 11 tests fail on the
stub implementation. 40 structural/constant tests are green-by-design (they
test already-implemented stub scaffolding — registry, error types, constants).

---

## CRITICAL: Stub vs v1.5 Mismatch — base_delay_ms

**BC-2.01.014 mandates `base_delay_ms = 2000` ("2s base").**

Stub commit `e86d03f2` was generated against story v1.3 and has:

```rust
// crates/prism-sensors/src/retry.rs — stub (WRONG)
base_delay_ms: 1_000,   // <-- must be 2_000 per BC-2.01.014 + v1.5 spec
```

Test `test_BC_2_01_014_retry_config_default_base_delay_is_2000ms` asserts the
**literal `2000`** — not `RetryConfig::default().base_delay_ms` — specifically
to catch this mismatch.

**Implementer action required:** Change `base_delay_ms: 1_000` to
`base_delay_ms: 2_000` in `crates/prism-sensors/src/retry.rs` `Default` impl.

---

## Test Files Written

| File | Tests | Module |
|------|-------|--------|
| `src/tests/bc_2_01_002.rs` | 8 | BC-2.01.002 — Cross-Client Fan-Out |
| `src/tests/bc_2_01_010.rs` | 6 | BC-2.01.010 — Partial Failure Handling |
| `src/tests/bc_2_01_013.rs` | 10 | BC-2.01.013 — DataSource Trait / Registry |
| `src/tests/bc_2_01_014.rs` | 21 | BC-2.01.014 — Retry / Backoff |
| `src/tests/bc_2_01_http_semaphore.rs` | 6 | AC-5 / EC-003 / EC-004 — HTTP Semaphore |

---

## Red Gate Results

```
test result: FAILED. 40 passed; 11 failed; 0 ignored; 0 measured
```

### Failing Tests (RED — must stay red until implementation)

| Test | Reason |
|------|--------|
| `test_BC_2_01_014_retry_config_default_base_delay_is_2000ms` | `1000 != 2000` — stub has wrong default |
| `test_BC_2_01_014_retry_with_backoff_succeeds_on_second_attempt_for_503` | `todo!()` in `retry_with_backoff` |
| `test_BC_2_01_014_retry_with_backoff_returns_immediately_for_400` | `todo!()` in `retry_with_backoff` |
| `test_BC_2_01_014_retry_with_backoff_returns_immediately_for_404` | `todo!()` in `retry_with_backoff` |
| `test_BC_2_01_014_retry_budget_exhausted_after_max_attempts` | `todo!()` in `retry_with_backoff` |
| `test_BC_2_01_002_fan_out_six_targets_all_succeed` | `todo!()` in `fan_out` |
| `test_BC_2_01_002_fan_out_empty_targets_returns_empty_result` | `todo!()` in `fan_out` |
| `test_BC_2_01_010_fan_out_all_targets_fail_returns_all_targets_failed` | `todo!()` in `fan_out` |
| `test_BC_2_01_010_fan_out_five_succeed_one_503_returns_partial_result` | `todo!()` in `fan_out` |
| `test_BC_2_01_http_semaphore_acquire_succeeds_when_permits_available` | `todo!()` in `acquire_http_permit` |
| `test_BC_2_01_http_semaphore_exhausted_returns_connection_pool_exhausted` | `todo!()` in `acquire_http_permit` |

### Passing Tests (GREEN-BY-DESIGN)

40 tests covering already-implemented stub code pass:
- Registry: all 10 `bc_2_01_013` tests (registry is fully implemented in stub)
- Retry constants: `multiplier`, `max_delay`, `max_attempts`, `transient_codes`,
  delay schedule, jitter range, `SensorError::is_transient()` classifications
- Fan-out: `MAX_FANOUT_CONCURRENCY == 10`, `FanOutTarget` fields, `error_to_retry_metadata`
- BC-2.01.010: `FanOutError` / `FanOutResult` struct tests, `AllTargetsFailed` Display
- HTTP semaphore: `HTTP_SEMAPHORE_PERMITS == 200`, `HTTP_SEMAPHORE_TIMEOUT == 30s`,
  init idempotency, `available_http_permits()`, blocking-not-rejecting semaphore test

---

## Compile / Clippy / Fmt

| Check | Result |
|-------|--------|
| `cargo build -p prism-sensors` | PASS |
| `cargo test -p prism-sensors --no-run` | PASS |
| `cargo clippy -p prism-sensors --tests` | PASS (0 errors) |
| `cargo fmt -p prism-sensors -- --check` | PASS (0 diffs) |

---

## Workspace Impact

| Metric | Value |
|--------|-------|
| Baseline (pre-S-2.06 tests) | 1058 pass / 0 fail |
| After S-2.06 tests | 1098 pass / 11 fail |
| Delta pass | +40 (green-by-design) |
| Delta fail | +11 (red — unimplemented stubs + base_delay_ms mismatch) |
| Other crates broken | 0 |

---

## Implementer Handoff Instructions

Implement each function in this order (each makes specific tests turn green):

1. **`retry.rs`**: Fix `base_delay_ms: 1_000` → `2_000` in `Default`.
   Then implement `retry_with_backoff()` — makes 5 tests green.

2. **`http.rs`**: Implement `acquire_http_permit()` — makes 2 tests green.

3. **`fanout.rs`**: Implement `fan_out()` and `execute_target()` — makes 4 tests green.

All 11 red tests must turn green. The 40 green-by-design tests must remain green.

---

## Spec / API Gaps Discovered

1. **`SensorError::AllTargetsFailed` `count` field**: The struct has both
   `count: usize` and `errors: Vec<FanOutError>`. The `count` should always
   equal `errors.len()`. The implementer should ensure these stay consistent
   (or drop `count` in favor of `errors.len()`). Not a blocker — tests work
   with both fields present.

2. **`HTTP_SEMAPHORE` OnceLock global + test isolation**: The global semaphore
   is initialized with 200 permits via `OnceLock`. Multiple tests that call
   `init_http_semaphore()` + `acquire_http_permit()` in the same test binary
   share the same semaphore instance. The test
   `test_BC_2_01_http_semaphore_exhausted_returns_connection_pool_exhausted`
   takes all 200 permits — if run in parallel with
   `test_BC_2_01_http_semaphore_acquire_succeeds_when_permits_available`, the
   latter will see 0 available permits. The implementer should either:
   (a) Use `cargo test -- --test-threads=1` for the semaphore tests, or
   (b) Add `#[serial_test::serial]` annotations, or
   (c) Restructure `acquire_http_permit()` to accept an optional semaphore
       parameter for testability. Recommendation: (c) for clean design.

3. **`retry_with_backoff` zero-delay in tests**: Tests use `base_delay_ms: 0`
   to avoid actual sleeps. The implementer must ensure `tokio::time::sleep(Duration::ZERO)`
   does not panic and that `Duration::from_millis(0)` is a valid sleep call.
