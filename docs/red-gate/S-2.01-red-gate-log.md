# Red Gate Log — S-2.01 (TDD Step b)

## Story

S-2.01 — prism-storage: RocksDB Initialization and Domain Operations

## Date

2026-04-24

## Status

RED GATE VERIFIED

## Summary

All 24 integration tests in `crates/prism-storage/tests/integration.rs` now exercise
real assertions against the SUT (RocksDbBackend, dirty_bits functions). All 24 fail
because the SUT bodies contain `todo!("step c implementer")` panics.

Test code is free of `todo!()`. Failures originate from SUT methods, not test code.

## Test Run Output

```
running 24 tests
test result: FAILED. 0 passed; 24 failed; 0 ignored; 0 measured; 0 filtered out
```

## Failure Source

Every failing test panics at:
- `crates/prism-storage/src/rocksdb_backend.rs:40:9` — `RocksDbBackend::open()` todo!()
- `crates/prism-storage/src/rocksdb_backend.rs:54:9` — `RocksDbBackend::recover_or_exit()` todo!()

Example panic message:
```
not yet implemented: step c implementer — open RocksDB, init all CFs, map lock error to StorageLockHeld
```

## Tests by Category

### AC-Derived (7)
- test_ac_1_open_initializes_all_16_column_families — FAILED (open todo!())
- test_ac_2_idempotent_reopen_no_duplicate_cfs — FAILED (open todo!())
- test_ac_3_put_get_roundtrip_alerts_domain — FAILED (open todo!())
- test_ac_4_scan_prefix_returns_only_matching — FAILED (open todo!())
- test_ac_5_set_dirty_writes_with_sync — FAILED (open todo!())
- test_ac_6_check_dirty_on_startup_returns_uncleared — FAILED (open todo!())
- test_ac_7_storage_lock_held_error — FAILED (open todo!())

### Edge Case (5)
- test_ec_001_lock_held_returns_error — FAILED (open todo!())
- test_ec_002_corruption_repair_then_exit_3 — FAILED (recover_or_exit todo!() + explicit panic)
- test_ec_003_schema_mismatch_error — FAILED (open todo!())
- test_ec_004_dirty_bit_warning_on_startup — FAILED (open todo!())
- test_ec_005_storage_domain_not_found — FAILED (open todo!())

### BC-2.15.001 State (3)
- test_BC_2_15_001_health_check_passes_after_open — FAILED (open todo!())
- test_BC_2_15_001_recover_or_exit_repair_success — FAILED (recover_or_exit todo!() + explicit panic)
- test_BC_2_15_001_check_schema_version_writes_on_fresh_db — FAILED (open todo!())

### BC-2.15.002 State (6)
- test_BC_2_15_002_get_missing_key_returns_none — FAILED (open todo!())
- test_BC_2_15_002_put_batch_writes_all_entries_atomically — FAILED (open todo!())
- test_BC_2_15_002_remove_nonexistent_key_is_noop — FAILED (open todo!())
- test_BC_2_15_002_scan_range_returns_bounded_entries — FAILED (open todo!())
- test_BC_2_15_002_invariant_domain_isolation — FAILED (open todo!())
- test_BC_2_15_002_missing_domain_cf_returns_domain_not_found — FAILED (open todo!())

### BC-2.15.005 State (3)
- test_BC_2_15_005_clear_dirty_removes_entry — FAILED (open todo!())
- test_BC_2_15_005_clean_shutdown_no_uncleared_bits — FAILED (open todo!())
- test_BC_2_15_005_invariant_startup_recovery_idempotent — FAILED (open todo!())

## Changes Made

### prism-core/src/error.rs
Added 3 new PrismError variants required by BC-2.15.001:
- `StorageLockHeld { path: PathBuf }` — E-STORE-006 (BC-2.15.001 lock conflict)
- `StorageHealthCheckFailed { detail: String }` — E-STORE-007 (health check failure)
- `SchemaMismatch { stored: String, current: String }` — E-STORE-008 (schema version mismatch)

### prism-storage/src/backend.rs
Added `RocksStorageBackend` trait (S-2.01 production variant) alongside the
existing `StorageBackend` trait (S-1.02 mock/test variant). The new trait:
- Returns `PrismError` (no associated Error type)
- Is `Send + Sync + 'static`
- Has 6 methods: get, put, put_batch, remove, scan, scan_range

### prism-storage/src/rocksdb_backend.rs
Added `RocksDbBackend` stub struct with:
- `open(state_dir)`, `health_check()`, `recover_or_exit(state_dir)`, `check_schema_version()`
- Full `RocksStorageBackend` impl with todo!() bodies
- `#[derive(Debug)]` for test error formatting

### prism-storage/src/memory_backend.rs
Added `InMemoryBackend` stub struct with:
- `#[cfg(any(test, feature = "test-utils"))]` gate
- Full `RocksStorageBackend` impl with todo!() bodies

### prism-storage/src/dirty_bits.rs
Added 3 function stubs with todo!() bodies:
- `set_dirty(db, query_id)` — BC-2.15.005 dirty bit write (sync:true)
- `clear_dirty(db, query_id)` — BC-2.15.005 dirty bit removal
- `check_dirty_on_startup(db)` — BC-2.15.005 startup scan

### prism-storage/Cargo.toml
Added `tempfile = "3"` to `[dev-dependencies]`.

## Compile Verification

- `cargo test -p prism-storage --no-run` — exit 0 (compiles cleanly)
- `cargo build --workspace` — exit 0 (full workspace builds)

## Implementer Instructions

Step (c): Make each test pass by replacing `todo!()` in SUT methods:
1. `RocksDbBackend::open()` — open RocksDB with all CFs, map lock error
2. `RocksDbBackend::health_check()` — write/read/delete on default CF
3. `RocksDbBackend::recover_or_exit()` — DB::repair() + retry or exit(3)
4. `RocksDbBackend::check_schema_version()` — read/write _schema_version key
5. All `RocksStorageBackend` methods on `RocksDbBackend`
6. All three `dirty_bits` functions
7. All `RocksStorageBackend` methods on `InMemoryBackend`

Make one test pass at a time with minimum code.
