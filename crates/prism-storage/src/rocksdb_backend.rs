// S-2.01 — RocksDbBackend compile-stub (TDD step b).
//
// Defines the public struct and method signatures required for integration.rs
// to compile and run.  All method bodies are `todo!("step c implementer")`.
//
// Implementer (step c) replaces every `todo!()` with real RocksDB logic:
//   - Opens/creates the DB at `{state_dir}/prism.db`
//   - Initializes all 16 (or 19) column families via `create_missing_column_families: true`
//   - Maps `StorageDomain` -> CF handle via `StorageDomain::column_family_name()`
//   - Implements health check write/read/delete cycle on `default` CF
//   - Implements corruption repair via `DB::repair()` + exit(3)
//   - Implements schema version tag check in `default` CF
//   - Maps OS-level LOCK error to `PrismError::StorageLockHeld { path }`

use std::path::PathBuf;

use prism_core::{PrismError, StorageDomain};

use crate::backend::RocksStorageBackend;

/// RocksDB-backed storage engine for Prism (S-2.01).
///
/// Opens the database at `{state_dir}/prism.db` and manages all column families.
/// Holds an exclusive OS-level lock for the lifetime of the struct.
///
/// Construct via `RocksDbBackend::open(state_dir)`.
#[derive(Debug)]
pub struct RocksDbBackend {
    // Implementer adds: Arc<DB>, CF handle map, state_dir
    _private: (),
}

impl RocksDbBackend {
    /// Open (or create) the RocksDB database at `{state_dir}/prism.db`.
    ///
    /// On success, all column families are initialized and accessible.
    /// Returns `Err(PrismError::StorageLockHeld { path })` if the LOCK file is
    /// held by another process (E-STORE-005 / E-STORE-006, BC-2.15.001).
    pub fn open(_state_dir: PathBuf) -> Result<Self, PrismError> {
        todo!("step c implementer — open RocksDB, init all CFs, map lock error to StorageLockHeld")
    }

    /// Perform the startup health check: write, read, delete on the `default` CF.
    ///
    /// Returns `Err(PrismError::StorageHealthCheckFailed { .. })` if any step fails.
    pub fn health_check(&self) -> Result<(), PrismError> {
        todo!("step c implementer — write/read/delete cycle on default CF")
    }

    /// Attempt corruption recovery: call `DB::repair()`; if repair fails, exit(3).
    ///
    /// If repair succeeds, retries `DB::open()` once and returns the recovered backend.
    pub fn recover_or_exit(_state_dir: PathBuf) -> Result<Self, PrismError> {
        todo!("step c implementer — DB::repair() then retry open, else exit(3)")
    }

    /// Check that the `_schema_version` tag in the `default` CF matches the
    /// current Prism schema version.
    ///
    /// - Fresh DB (no tag): writes the current version and returns `Ok(())`.
    /// - Matching version: returns `Ok(())`.
    /// - Mismatched version: returns `Err(PrismError::SchemaMismatch { .. })`.
    pub fn check_schema_version(&self) -> Result<(), PrismError> {
        todo!("step c implementer — read _schema_version from default CF, write if absent, error on mismatch")
    }
}

impl RocksStorageBackend for RocksDbBackend {
    fn get(&self, _domain: StorageDomain, _key: &[u8]) -> Result<Option<Vec<u8>>, PrismError> {
        todo!("step c implementer — resolve domain to CF handle, call DB::get_cf")
    }

    fn put(&self, _domain: StorageDomain, _key: &[u8], _value: &[u8]) -> Result<(), PrismError> {
        todo!("step c implementer — resolve domain to CF handle, call DB::put_cf")
    }

    fn put_batch(
        &self,
        _domain: StorageDomain,
        _entries: &[(&[u8], &[u8])],
    ) -> Result<(), PrismError> {
        todo!("step c implementer — build WriteBatch, apply via DB::write")
    }

    fn remove(&self, _domain: StorageDomain, _key: &[u8]) -> Result<(), PrismError> {
        todo!("step c implementer — resolve domain to CF handle, call DB::delete_cf")
    }

    fn scan(
        &self,
        _domain: StorageDomain,
        _prefix: &[u8],
    ) -> Result<Vec<(Vec<u8>, Vec<u8>)>, PrismError> {
        todo!("step c implementer — prefix_iterator on CF, collect results")
    }

    fn scan_range(
        &self,
        _domain: StorageDomain,
        _start: &[u8],
        _end: &[u8],
    ) -> Result<Vec<(Vec<u8>, Vec<u8>)>, PrismError> {
        todo!("step c implementer — IteratorMode::From(start, Forward), filter < end")
    }
}
