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

pub mod backend;
pub mod mock;
pub mod recovery;

// ── Proof modules ─────────────────────────────────────────────────────────────
pub mod proofs;

// ── Re-exports ────────────────────────────────────────────────────────────────
pub use backend::StorageBackend;
pub use mock::MockStorageEngine;
pub use recovery::{advance_crash_counter, DirtyBitEntry, RecoveryAction};
