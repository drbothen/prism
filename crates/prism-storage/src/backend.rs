// S-1.02 — StorageBackend trait (stub).
//
// Defines the interface for all storage engines in Prism.  The production
// implementation (RocksDB) lives in S-2.xx.  This stub exists so VP-055 and
// VP-057 can compile and be written now.

use prism_core::StorageDomain;

/// Key-value storage backend with domain isolation.
///
/// All reads and writes are scoped to a `StorageDomain` — data written to one
/// domain is never visible from another domain, regardless of key.
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
