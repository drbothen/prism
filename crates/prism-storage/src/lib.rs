// prism-storage — Storage engine abstraction (RocksDB backend with domain isolation).
//
// Layer 1 of the Prism platform.  Depends on prism-core for `StorageDomain` and
// `PrismError`.
//
// Types introduced here for S-1.02 Red Gate:
//   - `StorageBackend` trait
//   - `MockStorageEngine` (test-only implementation for VP-055)
//   - `DirtyBitEntry` and `RecoveryAction` (for VP-057)
//   - `advance_crash_counter` function (stub, for VP-057)
//
// Modules added by S-2.01 (stubs; implementations in step c):
//   - `rocksdb_backend` — RocksDbBackend (16 CFs, RocksDB options, health check)
//   - `memory_backend`  — InMemoryBackend (BTreeMap, test-utils feature gate)
//   - `dirty_bits`      — set_dirty / clear_dirty / check_dirty_on_startup
//
// Modules added by S-2.02:
//   - `audit_buffer` — append_audit_entry, check_and_purge_overflow (BC-2.15.003/004)
//   - `watchdog`     — ResourceWatchdog, WatchdogLevel, WatchdogStatus (BC-2.15.006/007)
//   - `denylist`     — record_failure, is_denylisted, clear_denylist (BC-2.15.008)
//
// Modules added by S-2.03:
//   - `decorators`      — DecorationStore (Phase 1 in-memory map, Phase 3 RocksDB cache,
//                         merge logic) (BC-2.15.010)
//   - `internal_tables` — INTERNAL_TABLES static, get_descriptor, all_descriptors,
//                         scan_limit, check_table_access, column schema helpers
//                         (BC-2.15.011)

pub mod backend;
pub mod dirty_bits;
pub mod memory_backend;
pub mod mock;
pub mod recovery;
pub mod rocksdb_backend;

// ── S-2.02 modules ────────────────────────────────────────────────────────────
pub mod audit_buffer;
pub mod denylist;
pub mod watchdog;

// ── S-2.03 modules ────────────────────────────────────────────────────────────
pub mod decorators;
pub mod internal_tables;

// ── Proof modules ─────────────────────────────────────────────────────────────
pub mod proofs;

// ── Test modules ──────────────────────────────────────────────────────────────
#[cfg(test)]
pub mod tests;

// ── Re-exports ────────────────────────────────────────────────────────────────
pub use backend::{RocksStorageBackend, StorageBackend};
pub use mock::MockStorageEngine;
pub use recovery::{advance_crash_counter, DirtyBitEntry, RecoveryAction};
