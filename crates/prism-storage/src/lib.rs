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

pub mod backend;
pub mod dirty_bits;
pub mod memory_backend;
pub mod mock;
pub mod recovery;
pub mod rocksdb_backend;

// ── Proof modules ─────────────────────────────────────────────────────────────
pub mod proofs;

// ── Re-exports ────────────────────────────────────────────────────────────────
pub use backend::StorageBackend;
pub use mock::MockStorageEngine;
pub use recovery::{advance_crash_counter, DirtyBitEntry, RecoveryAction};
