// S-2.01 — InMemoryBackend: BTreeMap-backed RocksStorageBackend for tests.
//
// Gated behind `#[cfg(any(test, feature = "test-utils"))]` so it is only
// compiled when tests are run or the `test-utils` feature is explicitly enabled.
//
// Downstream crates add `prism-storage = { ..., features = ["test-utils"] }`
// as a dev-dependency to gain access to `InMemoryBackend`.

#[cfg(any(test, feature = "test-utils"))]
pub mod memory_backend_inner {
    use std::collections::BTreeMap;
    use std::sync::RwLock;

    use prism_core::{PrismError, StorageDomain};

    use crate::backend::RocksStorageBackend;

    /// Inner map type: domain + key → value.
    type InnerMap = BTreeMap<(StorageDomain, Vec<u8>), Vec<u8>>;

    /// In-memory storage backend implementing `RocksStorageBackend`.
    ///
    /// Uses a `BTreeMap<(StorageDomain, Vec<u8>), Vec<u8>>` under `RwLock` so
    /// it can be used from `&self` methods (matching the production trait API).
    ///
    /// Available only when `test` or the `test-utils` feature is active.
    pub struct InMemoryBackend {
        inner: RwLock<InnerMap>,
    }

    impl InMemoryBackend {
        /// Create a new, empty in-memory backend.
        pub fn new() -> Self {
            InMemoryBackend {
                inner: RwLock::new(BTreeMap::new()),
            }
        }
    }

    impl Default for InMemoryBackend {
        fn default() -> Self {
            Self::new()
        }
    }

    // SAFETY: BTreeMap<_, _> under RwLock is Send + Sync.
    unsafe impl Send for InMemoryBackend {}
    unsafe impl Sync for InMemoryBackend {}

    impl RocksStorageBackend for InMemoryBackend {
        fn get(&self, domain: StorageDomain, key: &[u8]) -> Result<Option<Vec<u8>>, PrismError> {
            let guard = self
                .inner
                .read()
                .map_err(|e| PrismError::StorageReadFailed {
                    domain: domain.column_family_name().to_owned(),
                    detail: e.to_string(),
                })?;
            Ok(guard.get(&(domain, key.to_vec())).cloned())
        }

        fn put(&self, domain: StorageDomain, key: &[u8], value: &[u8]) -> Result<(), PrismError> {
            let mut guard = self
                .inner
                .write()
                .map_err(|e| PrismError::StorageWriteFailed {
                    domain: domain.column_family_name().to_owned(),
                    detail: e.to_string(),
                })?;
            guard.insert((domain, key.to_vec()), value.to_vec());
            Ok(())
        }

        fn put_batch(
            &self,
            domain: StorageDomain,
            entries: &[(&[u8], &[u8])],
        ) -> Result<(), PrismError> {
            let mut guard = self
                .inner
                .write()
                .map_err(|e| PrismError::StorageWriteFailed {
                    domain: domain.column_family_name().to_owned(),
                    detail: e.to_string(),
                })?;
            for (key, value) in entries {
                guard.insert((domain, key.to_vec()), value.to_vec());
            }
            Ok(())
        }

        fn remove(&self, domain: StorageDomain, key: &[u8]) -> Result<(), PrismError> {
            let mut guard = self
                .inner
                .write()
                .map_err(|e| PrismError::StorageWriteFailed {
                    domain: domain.column_family_name().to_owned(),
                    detail: e.to_string(),
                })?;
            guard.remove(&(domain, key.to_vec()));
            Ok(())
        }

        fn scan(
            &self,
            domain: StorageDomain,
            prefix: &[u8],
        ) -> Result<Vec<(Vec<u8>, Vec<u8>)>, PrismError> {
            let guard = self
                .inner
                .read()
                .map_err(|e| PrismError::StorageReadFailed {
                    domain: domain.column_family_name().to_owned(),
                    detail: e.to_string(),
                })?;
            let results = guard
                .range((domain, prefix.to_vec())..)
                .take_while(|((d, k), _)| *d == domain && k.starts_with(prefix))
                .map(|((_, k), v)| (k.clone(), v.clone()))
                .collect();
            Ok(results)
        }

        fn scan_range(
            &self,
            domain: StorageDomain,
            start: &[u8],
            end: &[u8],
        ) -> Result<Vec<(Vec<u8>, Vec<u8>)>, PrismError> {
            let guard = self
                .inner
                .read()
                .map_err(|e| PrismError::StorageReadFailed {
                    domain: domain.column_family_name().to_owned(),
                    detail: e.to_string(),
                })?;
            let results = guard
                .range((domain, start.to_vec())..(domain, end.to_vec()))
                .map(|((_, k), v)| (k.clone(), v.clone()))
                .collect();
            Ok(results)
        }
    }
}

#[cfg(any(test, feature = "test-utils"))]
pub use memory_backend_inner::InMemoryBackend;
