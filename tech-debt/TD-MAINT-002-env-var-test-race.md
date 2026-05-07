# TD-MAINT-002 — Pre-existing env-var race in `test_parse_limits_snapshot_is_immutable_after_capture`

**Status:** open  
**Severity:** medium  
**Filed:** 2026-05-07  
**Source:** maintenance/clippy-unwrap-cleanup adversary pass-2 I-3 (deferred, out of scope)

---

## Summary

`test_parse_limits_snapshot_is_immutable_after_capture` reads an environment
variable to construct a `ParseLimits` snapshot, then asserts that a subsequent
mutation of the environment variable does not affect the already-snapshotted
instance. This test has a latent race condition: if another test running in the
same process in parallel modifies the same environment variable between the
snapshot and the assertion, the test can produce a non-deterministic result.

This defect is **pre-existing** — it was not introduced by `maintenance/clippy-unwrap-cleanup`.
The maintenance branch touched `integration_tests.rs` and `bc_gap_fill_tests.rs`
only to add `#![allow(clippy::unwrap_used, clippy::expect_used)]` at the file
level; no logic was changed in the snapshot test.

---

## Root Cause

Rust's test runner by default runs tests in the same process in parallel.
Environment variable reads/writes via `std::env::set_var` / `std::env::var`
are not synchronized. If test A sets an env var while test B is simultaneously
reading it (or mid-snapshot), the read order is non-deterministic.

`cargo nextest` isolates each test in a separate process by default, which
suppresses this race in practice on CI (nextest is the project's test runner
per `just iter`). However, the test would be racy under `cargo test` (single
process, parallel threads).

---

## Impact

- **nextest (CI):** Not currently observed; each test is process-isolated.
- **cargo test:** Potentially racy if other tests touch the same env var.
- **Non-blocking:** This does not affect CI green/red status under nextest.

---

## Recommended Fix

Option A (preferred): Use `std::sync::Mutex` (or `once_cell::sync::Lazy<Mutex<()>>`)
to serialize env-var access within the test binary. Example pattern:

```rust
static ENV_MUTEX: once_cell::sync::Lazy<std::sync::Mutex<()>> =
    once_cell::sync::Lazy::new(|| std::sync::Mutex::new(()));

#[test]
fn test_parse_limits_snapshot_is_immutable_after_capture() {
    let _guard = ENV_MUTEX.lock().unwrap();
    // ... test body using env var ...
}
```

Option B: Mark the test `#[serial]` using the `serial_test` crate to prevent
parallel execution with other env-var-touching tests.

Option C: Refactor the test to not rely on environment variables at all
(pass limits directly via constructor).

---

## References

- Maintenance branch: `maintenance/clippy-unwrap-cleanup`
- Adversary finding: I-3 (deferred — pre-existing, out of scope for maintenance branch)
- Test location: `crates/prism-query/src/tests/regression_tests.rs:612`
