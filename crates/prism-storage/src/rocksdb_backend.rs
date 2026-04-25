// S-2.01 — RocksDbBackend: RocksDB-backed storage with all column families.
//
// Opens the database at `{state_dir}/prism.db` and manages all CFs.
// Holds an exclusive OS-level LOCK for the lifetime of the struct.
//
// Key-pattern documentation for `action_state` CF:
//   `{action_id}:last_fired`        → u64 LE timestamp bytes of last action execution
//   `{action_id}:fire_count:{hour}` → u32 LE count of fires in the given UTC hour
//                                     window (hour encoded as `YYYYMMDDHH` ASCII string)
//   `{action_id}:dedup:{hash}`      → empty bytes; presence = already fired for this
//                                     alert hash; used for deduplication window (S-4.08)
//   `{action_id}:retry:{alert_id}`  → retry state bytes (attempt count + next_retry_at)

use std::path::{Path, PathBuf};
use std::sync::Arc;

use rocksdb::{
    BlockBasedOptions, ColumnFamily, ColumnFamilyDescriptor, DBCompressionType, Direction,
    IteratorMode, Options, WriteBatch, WriteOptions, DB,
};

use prism_core::{PrismError, StorageDomain};

use crate::backend::RocksStorageBackend;

/// Current schema version written to the `default` CF on first open.
///
/// Bump this constant when the CF layout changes in an incompatible way.
const SCHEMA_VERSION: &str = "prism-storage:v1";

/// Key used to store the schema version tag in the `default` CF.
const SCHEMA_VERSION_KEY: &[u8] = b"_schema_version";

/// Write buffer size per column family: 64 MB (BC-2.15.001).
const WRITE_BUFFER_SIZE: usize = 64 * 1024 * 1024;

/// Maximum number of open files across all CFs.
const MAX_OPEN_FILES: i32 = 256;

/// Bloom filter bits per key for point-lookup acceleration.
const BLOOM_BITS_PER_KEY: f64 = 10.0;

/// RocksDB-backed storage engine for Prism (S-2.01).
///
/// Opens the database at `{state_dir}/prism.db` and manages all column families.
/// Holds an exclusive OS-level lock for the lifetime of the struct.
///
/// Construct via `RocksDbBackend::open(state_dir)`.
pub struct RocksDbBackend {
    db: Arc<DB>,
    /// State directory passed to `open()` — used in error messages.
    state_dir: PathBuf,
    /// Set of domain names that are registered in the CF name map.
    /// A domain missing from this set returns `StorageDomainNotFound`.
    active_domains: std::collections::HashSet<StorageDomain>,
}

// SAFETY: `rocksdb::DB` is `Send` upstream. We assert `Sync` because all
// `RocksStorageBackend` methods take `&self` and RocksDB's internal locking
// is sufficient for single-threaded CF mode (no `multi-threaded-cf` feature).
// Concurrent write contention is the caller's responsibility. Wave 3 concurrent
// access work must revisit this assertion if multi-CF concurrent writes become
// a hot path (DEV-004, tracked in tech-debt-register.md).
unsafe impl Send for RocksDbBackend {}
unsafe impl Sync for RocksDbBackend {}

impl std::fmt::Debug for RocksDbBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RocksDbBackend")
            .field("state_dir", &self.state_dir)
            .finish_non_exhaustive()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Internal helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Build a per-CF `Options` with LZ4 compression, 64 MB write buffer,
/// and a 10-bit bloom filter for point lookups (BC-2.15.001).
fn cf_options() -> Options {
    let mut block_opts = BlockBasedOptions::default();
    block_opts.set_bloom_filter(BLOOM_BITS_PER_KEY, false);

    let mut opts = Options::default();
    opts.set_block_based_table_factory(&block_opts);
    opts.set_compression_type(DBCompressionType::Lz4);
    opts.set_write_buffer_size(WRITE_BUFFER_SIZE);
    opts
}

/// Build the shared DB-level `Options`.
pub(crate) fn db_options() -> Options {
    let mut opts = Options::default();
    opts.create_if_missing(true);
    opts.create_missing_column_families(true);
    opts.set_max_open_files(MAX_OPEN_FILES);
    // WAL is enabled by default — no explicit opt-in required.
    opts
}

/// Construct the list of `ColumnFamilyDescriptor`s for every `StorageDomain` variant.
///
/// The `default` CF is included explicitly because RocksDB always expects it in
/// the descriptor list when opening with `open_cf_descriptors`.
fn build_cf_descriptors() -> Vec<ColumnFamilyDescriptor> {
    StorageDomain::all()
        .iter()
        .map(|domain| {
            ColumnFamilyDescriptor::new(domain.column_family_name().to_owned(), cf_options())
        })
        .collect()
}

/// Map a RocksDB open error to the appropriate `PrismError`.
///
/// If the error string contains "lock" (case-insensitive), another process holds
/// the exclusive LOCK file → `StorageLockHeld`.  Otherwise → `StorageOpenFailed`.
fn map_open_error(err: rocksdb::Error, state_dir: &Path) -> PrismError {
    let msg = err.to_string();
    if msg.to_lowercase().contains("lock") {
        PrismError::StorageLockHeld {
            path: state_dir.to_path_buf(),
        }
    } else {
        PrismError::StorageOpenFailed { detail: msg }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// RocksDbBackend — inherent methods
// ─────────────────────────────────────────────────────────────────────────────

impl RocksDbBackend {
    /// Open (or create) the RocksDB database at `{state_dir}/prism.db`.
    ///
    /// On success, all column families for every `StorageDomain` variant are
    /// initialized and accessible.
    ///
    /// Returns `Err(PrismError::StorageLockHeld { path })` if the LOCK file is
    /// held by another process (E-STORE-006, BC-2.15.001).
    pub fn open(state_dir: PathBuf) -> Result<Self, PrismError> {
        let db_path = state_dir.join("prism.db");

        let mut db_opts = db_options();
        // Apply bloom filter + LZ4 to the default CF via db-level opts too.
        let block_opts = {
            let mut b = BlockBasedOptions::default();
            b.set_bloom_filter(BLOOM_BITS_PER_KEY, false);
            b
        };
        db_opts.set_block_based_table_factory(&block_opts);
        db_opts.set_compression_type(DBCompressionType::Lz4);
        db_opts.set_write_buffer_size(WRITE_BUFFER_SIZE);

        let descriptors = build_cf_descriptors();

        let db = DB::open_cf_descriptors(&db_opts, &db_path, descriptors)
            .map_err(|e| map_open_error(e, &state_dir))?;

        let active_domains: std::collections::HashSet<StorageDomain> =
            StorageDomain::all().iter().copied().collect();

        Ok(RocksDbBackend {
            db: Arc::new(db),
            state_dir,
            active_domains,
        })
    }

    /// Perform the startup health check: write, read, delete on the `default` CF.
    ///
    /// Returns `Err(PrismError::StorageHealthCheckFailed { .. })` if any step fails.
    pub fn health_check(&self) -> Result<(), PrismError> {
        const HEALTH_KEY: &[u8] = b"__prism_health_check__";
        const HEALTH_VAL: &[u8] = b"ok";

        let cf = self.resolve_cf(StorageDomain::Default)?;

        // Write.
        self.db.put_cf(cf, HEALTH_KEY, HEALTH_VAL).map_err(|e| {
            PrismError::StorageHealthCheckFailed {
                detail: format!("health write failed: {e}"),
            }
        })?;

        // Read back and verify.
        let got =
            self.db
                .get_cf(cf, HEALTH_KEY)
                .map_err(|e| PrismError::StorageHealthCheckFailed {
                    detail: format!("health read failed: {e}"),
                })?;
        if got.as_deref() != Some(HEALTH_VAL) {
            return Err(PrismError::StorageHealthCheckFailed {
                detail: format!("health read returned unexpected value: {:?}", got),
            });
        }

        // Delete.
        self.db
            .delete_cf(cf, HEALTH_KEY)
            .map_err(|e| PrismError::StorageHealthCheckFailed {
                detail: format!("health delete failed: {e}"),
            })?;

        // Verify all CF handles are accessible.
        for domain in StorageDomain::all() {
            if self.db.cf_handle(domain.column_family_name()).is_none() {
                return Err(PrismError::StorageHealthCheckFailed {
                    detail: format!(
                        "CF handle missing for domain {:?} ({})",
                        domain,
                        domain.column_family_name()
                    ),
                });
            }
        }

        Ok(())
    }

    /// Attempt corruption recovery: call `DB::repair()`; if repair fails, exit(3).
    ///
    /// If repair succeeds, retries `DB::open()` once and returns the recovered backend.
    pub fn recover_or_exit(state_dir: PathBuf) -> Result<Self, PrismError> {
        let db_path = state_dir.join("prism.db");

        // Attempt repair.
        if let Err(e) = DB::repair(&db_options(), &db_path) {
            tracing::error!(
                path = %db_path.display(),
                error = %e,
                "RocksDB repair failed — exiting with code 3 (BC-2.15.001 EC-002)"
            );
            std::process::exit(3);
        }

        tracing::info!(path = %db_path.display(), "RocksDB repair succeeded; retrying open");

        // Retry open after repair.
        Self::open(state_dir)
    }

    /// Check that the `_schema_version` tag in the `default` CF matches the
    /// current Prism schema version.
    ///
    /// - Fresh DB (no tag): writes the current version and returns `Ok(())`.
    /// - Matching version: returns `Ok(())`.
    /// - Mismatched version: returns `Err(PrismError::SchemaMismatch { .. })`.
    pub fn check_schema_version(&self) -> Result<(), PrismError> {
        let cf = self.resolve_cf(StorageDomain::Default)?;

        match self
            .db
            .get_cf(cf, SCHEMA_VERSION_KEY)
            .map_err(|e| PrismError::StorageReadFailed {
                domain: "default".to_owned(),
                detail: e.to_string(),
            })? {
            None => {
                // Fresh DB — write the version tag.
                self.db
                    .put_cf(cf, SCHEMA_VERSION_KEY, SCHEMA_VERSION.as_bytes())
                    .map_err(|e| PrismError::StorageWriteFailed {
                        domain: "default".to_owned(),
                        detail: e.to_string(),
                    })?;
                Ok(())
            }
            Some(stored_bytes) => {
                let stored = String::from_utf8_lossy(&stored_bytes).into_owned();
                if stored == SCHEMA_VERSION {
                    Ok(())
                } else {
                    Err(PrismError::SchemaMismatch {
                        stored,
                        current: SCHEMA_VERSION.to_owned(),
                    })
                }
            }
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Private / crate-internal helpers
    // ─────────────────────────────────────────────────────────────────────────

    /// Resolve a `StorageDomain` to its `&ColumnFamily` handle.
    ///
    /// Returns `Err(PrismError::StorageDomainNotFound)` if the domain is not in
    /// the active set or its CF handle cannot be found in the DB.
    fn resolve_cf(&self, domain: StorageDomain) -> Result<&ColumnFamily, PrismError> {
        if !self.active_domains.contains(&domain) {
            return Err(PrismError::StorageDomainNotFound {
                domain: domain.column_family_name().to_owned(),
            });
        }
        self.db
            .cf_handle(domain.column_family_name())
            .ok_or_else(|| PrismError::StorageDomainNotFound {
                domain: domain.column_family_name().to_owned(),
            })
    }

    /// Write options for dirty-bit writes (sync = true per BC-2.15.005).
    pub(crate) fn sync_write_options() -> WriteOptions {
        let mut opts = WriteOptions::default();
        opts.set_sync(true);
        opts
    }

    /// Return a reference to the inner `Arc<DB>` for use by `dirty_bits` module.
    pub(crate) fn db(&self) -> &Arc<DB> {
        &self.db
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// RocksStorageBackend implementation
// ─────────────────────────────────────────────────────────────────────────────

impl RocksStorageBackend for RocksDbBackend {
    fn get(&self, domain: StorageDomain, key: &[u8]) -> Result<Option<Vec<u8>>, PrismError> {
        let cf = self.resolve_cf(domain)?;
        self.db
            .get_cf(cf, key)
            .map_err(|e| PrismError::StorageReadFailed {
                domain: domain.column_family_name().to_owned(),
                detail: e.to_string(),
            })
    }

    fn put(&self, domain: StorageDomain, key: &[u8], value: &[u8]) -> Result<(), PrismError> {
        let cf = self.resolve_cf(domain)?;
        self.db
            .put_cf(cf, key, value)
            .map_err(|e| PrismError::StorageWriteFailed {
                domain: domain.column_family_name().to_owned(),
                detail: e.to_string(),
            })
    }

    fn put_batch(
        &self,
        domain: StorageDomain,
        entries: &[(&[u8], &[u8])],
    ) -> Result<(), PrismError> {
        let cf = self.resolve_cf(domain)?;
        let mut batch = WriteBatch::default();
        for (key, value) in entries {
            batch.put_cf(cf, key, value);
        }
        self.db
            .write(batch)
            .map_err(|e| PrismError::StorageBatchFailed {
                detail: e.to_string(),
            })
    }

    fn remove(&self, domain: StorageDomain, key: &[u8]) -> Result<(), PrismError> {
        let cf = self.resolve_cf(domain)?;
        self.db
            .delete_cf(cf, key)
            .map_err(|e| PrismError::StorageWriteFailed {
                domain: domain.column_family_name().to_owned(),
                detail: e.to_string(),
            })
    }

    fn scan(
        &self,
        domain: StorageDomain,
        prefix: &[u8],
    ) -> Result<Vec<(Vec<u8>, Vec<u8>)>, PrismError> {
        let cf = self.resolve_cf(domain)?;
        let iter = self.db.prefix_iterator_cf(cf, prefix);
        let mut results = Vec::new();
        for item in iter {
            let (k, v) = item.map_err(|e| PrismError::StorageReadFailed {
                domain: domain.column_family_name().to_owned(),
                detail: e.to_string(),
            })?;
            // Stop as soon as the key no longer starts with the prefix.
            if !k.starts_with(prefix) {
                break;
            }
            results.push((k.to_vec(), v.to_vec()));
        }
        Ok(results)
    }

    fn scan_range(
        &self,
        domain: StorageDomain,
        start: &[u8],
        end: &[u8],
    ) -> Result<Vec<(Vec<u8>, Vec<u8>)>, PrismError> {
        let cf = self.resolve_cf(domain)?;
        let iter = self
            .db
            .iterator_cf(cf, IteratorMode::From(start, Direction::Forward));
        let mut results = Vec::new();
        for item in iter {
            let (k, v) = item.map_err(|e| PrismError::StorageReadFailed {
                domain: domain.column_family_name().to_owned(),
                detail: e.to_string(),
            })?;
            // Exclusive end bound.
            if k.as_ref() >= end {
                break;
            }
            results.push((k.to_vec(), v.to_vec()));
        }
        Ok(results)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Test-only helpers (available in integration tests and unit tests)
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(any(test, feature = "test-utils"))]
impl RocksDbBackend {
    /// Open a backend but exclude the given domain from the active-domains set so
    /// that KV operations on that domain return `StorageDomainNotFound`.
    ///
    /// Used by EC-005 and BC-2.15.002 missing-domain tests.
    pub fn open_excluding_domain(
        state_dir: PathBuf,
        excluded: StorageDomain,
    ) -> Result<Self, PrismError> {
        let mut backend = Self::open(state_dir)?;
        backend.active_domains.remove(&excluded);
        Ok(backend)
    }
}
