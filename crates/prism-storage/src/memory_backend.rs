// S-2.01 — InMemoryBackend compile-stub (TDD step b).
//
// BTreeMap-backed `RocksStorageBackend` for use in unit tests and downstream
// crate test suites.  Gated behind `#[cfg(any(test, feature = "test-utils"))]`
// so it is only compiled when tests are run or the feature is explicitly enabled.
//
// Implementer (step c) replaces the todo!() bodies with real BTreeMap logic:
//   - `BTreeMap<(StorageDomain, Vec<u8>), Vec<u8>>` guarded by `RwLock`
//   - `scan` returns all entries whose key starts with the given prefix
//   - `scan_range` returns all entries in [start, end) byte-wise

#[cfg(any(test, feature = "test-utils"))]
pub mod memory_backend_inner {
    use std::collections::BTreeMap;
    use std::sync::RwLock;

    use prism_core::{PrismError, StorageDomain};

    use crate::backend::RocksStorageBackend;

    /// In-memory storage backend implementing `RocksStorageBackend`.
    ///
    /// Uses a `BTreeMap<(StorageDomain, Vec<u8>), Vec<u8>>` under `RwLock` so
    /// it can be used from `&self` methods (matching the production trait API).
    ///
    /// Available only when `test` or the `test-utils` feature is active.
    pub struct InMemoryBackend {
        // Implementer adds: RwLock<BTreeMap<(StorageDomain, Vec<u8>), Vec<u8>>>
        _private: RwLock<BTreeMap<(StorageDomain, Vec<u8>), Vec<u8>>>,
    }

    impl InMemoryBackend {
        /// Create a new, empty in-memory backend.
        pub fn new() -> Self {
            InMemoryBackend {
                _private: RwLock::new(BTreeMap::new()),
            }
        }
    }

    impl Default for InMemoryBackend {
        fn default() -> Self {
            Self::new()
        }
    }

    // SAFETY: BTreeMap<_, _> is Send + Sync when wrapped in RwLock.
    unsafe impl Send for InMemoryBackend {}
    unsafe impl Sync for InMemoryBackend {}

    impl RocksStorageBackend for InMemoryBackend {
        fn get(
            &self,
            _domain: StorageDomain,
            _key: &[u8],
        ) -> Result<Option<Vec<u8>>, PrismError> {
            todo!("step c implementer — read_guard lookup")
        }

        fn put(
            &self,
            _domain: StorageDomain,
            _key: &[u8],
            _value: &[u8],
        ) -> Result<(), PrismError> {
            todo!("step c implementer — write_guard insert")
        }

        fn put_batch(
            &self,
            _domain: StorageDomain,
            _entries: &[(&[u8], &[u8])],
        ) -> Result<(), PrismError> {
            todo!("step c implementer — write_guard batch insert")
        }

        fn remove(&self, _domain: StorageDomain, _key: &[u8]) -> Result<(), PrismError> {
            todo!("step c implementer — write_guard remove")
        }

        fn scan(
            &self,
            _domain: StorageDomain,
            _prefix: &[u8],
        ) -> Result<Vec<(Vec<u8>, Vec<u8>)>, PrismError> {
            todo!("step c implementer — read_guard prefix scan")
        }

        fn scan_range(
            &self,
            _domain: StorageDomain,
            _start: &[u8],
            _end: &[u8],
        ) -> Result<Vec<(Vec<u8>, Vec<u8>)>, PrismError> {
            todo!("step c implementer — read_guard range scan")
        }
    }
}

#[cfg(any(test, feature = "test-utils"))]
pub use memory_backend_inner::InMemoryBackend;
