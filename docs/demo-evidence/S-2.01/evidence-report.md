---
document_type: demo-evidence-report
story_id: S-2.01
story_title: "prism-storage: RocksDB Initialization and Domain Operations"
producer: demo-recorder
timestamp: 2026-04-24
version: "1.0"
phase: 3
wave: 2
implementation_sha: 533c6ea1
test_pass_count: 24
test_total_count: 24
acs_demonstrated: 7
acs_total: 7
---

# Demo Evidence — S-2.01 prism-storage RocksDB Foundation

## Summary

- **Story:** S-2.01 — prism-storage: RocksDB Initialization and Domain Operations
- **Implementation commit:** `533c6ea1`
- **Test results:** 24/24 passing (`cargo test -p prism-storage --test integration`)
- **ACs demonstrated:** 7 of 7
- **Demo medium:** VHS (`.tape` source + rendered `.gif`)
- **Font:** FiraCode Nerd Font Mono, 14pt, Dracula theme, 1000x600

## Per-AC Evidence

### AC-1: Fresh open initializes all 16 column families

![AC-1 demo](ac-1-open-16-cfs.gif)

- **Tape source:** `ac-1-open-16-cfs.tape`
- **Test:** `test_ac_1_open_initializes_all_16_column_families`
- **BC link:** BC-2.15.001 postcondition — `RocksDbBackend::open()` succeeds on fresh state dir
- **What it shows:** `open()` on a fresh temp directory succeeds; smoke `put()` to each of the first 16 `StorageDomain` variants returns `Ok(())`, proving all 16 CF handles are live and writable.
- **Result:** `test result: ok. 1 passed; 0 failed`

---

### AC-2: Idempotent reopen — no duplicate CFs

![AC-2 demo](ac-2-idempotent-reopen.gif)

- **Tape source:** `ac-2-idempotent-reopen.tape`
- **Test:** `test_ac_2_idempotent_reopen_no_duplicate_cfs`
- **BC link:** BC-2.15.001 invariant — reopening an existing DB must not create duplicate column families
- **What it shows:** `open()` called twice on the same path (first handle dropped before second); second open succeeds and `health_check()` returns `Ok(())`.
- **Result:** `test result: ok. 1 passed; 0 failed`

---

### AC-3: put/get round-trip on Alerts domain

![AC-3 demo](ac-3-put-get-roundtrip.gif)

- **Tape source:** `ac-3-put-get-roundtrip.tape`
- **Test:** `test_ac_3_put_get_roundtrip_alerts_domain`
- **BC link:** BC-2.15.002 postcondition — `put(domain, key, val)` followed by `get(domain, key)` returns `Some(val)`
- **What it shows:** Writes `("key1", "value1")` to `StorageDomain::Alerts`; reads back and asserts `Some(b"value1".to_vec())`.
- **Result:** `test result: ok. 1 passed; 0 failed`

---

### AC-4: scan returns only prefix-matching keys

![AC-4 demo](ac-4-scan-prefix.gif)

- **Tape source:** `ac-4-scan-prefix.tape`
- **Test:** `test_ac_4_scan_prefix_returns_only_matching`
- **BC link:** BC-2.15.002 postcondition — `scan(domain, prefix)` returns only entries whose key starts with the given prefix, in lexicographic order
- **What it shows:** Three keys written to `StorageDomain::Schedules` (`tenant:acme:a`, `tenant:acme:b`, `tenant:other:c`); scan with prefix `"tenant:acme:"` returns exactly 2 entries in order.
- **Result:** `test result: ok. 1 passed; 0 failed`

---

### AC-5: set_dirty writes with sync semantics

![AC-5 demo](ac-5-set-dirty-sync.gif)

- **Tape source:** `ac-5-set-dirty-sync.tape`
- **Test:** `test_ac_5_set_dirty_writes_with_sync`
- **BC link:** BC-2.15.005 postcondition — `set_dirty(backend, id)` persists the marker to the `dirty_bits` CF with `WriteOptions::sync = true`
- **What it shows:** Calls `set_dirty(&backend, "qhash-abc")`; then `check_dirty_on_startup(&backend)` returns a list containing `"qhash-abc"`.
- **Result:** `test result: ok. 1 passed; 0 failed`

---

### AC-6: check_dirty_on_startup returns uncleared bits after crash simulation

![AC-6 demo](ac-6-check-dirty-on-startup.gif)

- **Tape source:** `ac-6-check-dirty-on-startup.tape`
- **Test:** `test_ac_6_check_dirty_on_startup_returns_uncleared`
- **BC link:** BC-2.15.005 postcondition — dirty bits set before a crash (uncleared) must be visible to the next process that opens the DB
- **What it shows:** Opens DB, calls `set_dirty("qhash-crash")`, drops handle (lock released, crash simulated), reopens on same path, calls `check_dirty_on_startup()` — asserts `"qhash-crash"` is in the returned list.
- **Result:** `test result: ok. 1 passed; 0 failed`

---

### AC-7: StorageLockHeld error when DB locked by another process

![AC-7 demo](ac-7-storage-lock-held.gif)

- **Tape source:** `ac-7-storage-lock-held.tape`
- **Test:** `test_ac_7_storage_lock_held_error`
- **BC link:** BC-2.15.001 / E-STORE-005/006 — second `open()` on a locked path must return `Err(PrismError::StorageLockHeld { path })`
- **What it shows:** First `RocksDbBackend::open()` is kept alive; second `open()` on the same path returns `Err(PrismError::StorageLockHeld { .. })` with `path` matching the state directory.
- **Result:** `test result: ok. 1 passed; 0 failed`

---

## Full Suite Green

```
test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.58s
```

All 24 integration tests pass:
- 7 AC-derived tests (AC-1 through AC-7)
- 5 edge-case tests (EC-001 through EC-005)
- 12 BC-state tests (BC-2.15.001 x3, BC-2.15.002 x6, BC-2.15.005 x3)

## Implementation Deviations from Spec

### 19 CFs vs 16 spec'd (AC-1)

`StorageDomain::all()` returns 19 variants (16 from S-1.01 + 3 from S-1.02 added later). The implementation opens all 19 CFs. AC-1 smoke-writes only the first 16 (i.e., `&StorageDomain::all()[..16]`), so the AC-1 text "all 16 column families" is satisfied exactly. The 3 additional CFs are bonus coverage, not a violation.

### EC-002 — repair-success path only (no exit-3 subprocess test)

The original red-gate test description intended to verify `exit(3)` via `catch_unwind`. Because `std::process::exit(3)` terminates the entire test runner process rather than unwinding, an inline test cannot observe it. The test was redesigned to exercise `recover_or_exit()` on a healthy DB (repair is a no-op, subsequent `open()` succeeds). The `exit(3)` corruption path is verified by code inspection: `recover_or_exit()` calls `std::process::exit(3)` after a failed `DB::repair()`. A subprocess-based test would be required to cover that branch in an automated harness; it is tracked as technical debt.

## Verification Artifacts

| Artifact | Status |
|---|---|
| `cargo test -p prism-storage --test integration` | 24/24 ok |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | clean |
| `cargo fmt --check` | clean |
| 7 `.tape` files | present |
| 7 `.gif` files | present |
