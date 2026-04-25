// S-1.02 — StorageBackend trait (mock/test variant).
// S-2.01 — RocksStorageBackend trait (RocksDB production variant).
//
// S-1.02: Defines the mock/test interface used by MockStorageEngine (VP-055).
//         Uses an associated `Error` type so MockStorageEngine can use its own error type.
//
// S-2.01: Adds `RocksStorageBackend` — the production interface over RocksDB column
//         families.  All methods return `Result<_, PrismError>` (no associated type).
//         RocksDbBackend and InMemoryBackend implement this trait.

use prism_core::{PrismError, StorageDomain};

/// Key-value storage backend with domain isolation (S-1.02 mock/test variant).
///
/// All reads and writes are scoped to a `StorageDomain` — data written to one
/// domain is never visible from another domain, regardless of key.
///
/// Used by `MockStorageEngine` and proof modules (VP-055).  Production RocksDB
/// backend implements `RocksStorageBackend` instead.
pub trait StorageBackend {
    type Error: std::fmt::Debug;

    /// Read a value from the given domain.
    fn get(&self, domain: StorageDomain, key: &[u8]) -> Option<Vec<u8>>;

    /// Write a single key-value pair to the given domain.
    fn put(
        &mut self,
        domain: StorageDomain,
        key: Vec<u8>,
        value: Vec<u8>,
    ) -> Result<(), Self::Error>;

    /// Write a batch of key-value pairs atomically.
    ///
    /// Either all entries are written or none are (all-or-nothing semantics).
    fn put_batch(
        &mut self,
        domain: StorageDomain,
        entries: &[(Vec<u8>, Vec<u8>)],
    ) -> Result<(), Self::Error>;

    /// Remove a key from the given domain.
    fn remove(&mut self, domain: StorageDomain, key: &[u8]) -> Result<(), Self::Error>;
}

/// Key-value storage backend with domain isolation (S-2.01 RocksDB production variant).
///
/// All reads and writes are scoped to a `StorageDomain` (column family). Methods
/// return `PrismError` rather than an associated error type so callers do not need
/// to be generic.
///
/// Implemented by `RocksDbBackend` (production) and `InMemoryBackend` (test-utils).
/// Must be `Send + Sync + 'static` to be used from async tokio tasks.
pub trait RocksStorageBackend: Send + Sync + 'static {
    /// Read a single value. Returns `None` if the key does not exist (not an error).
    fn get(&self, domain: StorageDomain, key: &[u8]) -> Result<Option<Vec<u8>>, PrismError>;

    /// Write a single key-value pair. Overwrites if key exists.
    fn put(&self, domain: StorageDomain, key: &[u8], value: &[u8]) -> Result<(), PrismError>;

    /// Atomically write multiple key-value pairs via RocksDB WriteBatch.
    /// Either all entries are written or none.
    fn put_batch(
        &self,
        domain: StorageDomain,
        entries: &[(&[u8], &[u8])],
    ) -> Result<(), PrismError>;

    /// Remove a single key. No-op if the key does not exist.
    fn remove(&self, domain: StorageDomain, key: &[u8]) -> Result<(), PrismError>;

    /// Scan all keys matching the given prefix within the domain.
    /// Returns results in lexicographic order.
    #[allow(clippy::type_complexity)]
    fn scan(
        &self,
        domain: StorageDomain,
        prefix: &[u8],
    ) -> Result<Vec<(Vec<u8>, Vec<u8>)>, PrismError>;

    /// Scan all keys in the range [start, end) within the domain.
    /// Returns results in lexicographic order.
    #[allow(clippy::type_complexity)]
    fn scan_range(
        &self,
        domain: StorageDomain,
        start: &[u8],
        end: &[u8],
    ) -> Result<Vec<(Vec<u8>, Vec<u8>)>, PrismError>;
}
