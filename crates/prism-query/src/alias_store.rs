//! `AliasStore` — `aliases.toml`-backed CRUD with file-first atomic writes.
//!
//! Persistence target: `aliases.toml` (NOT RocksDB).
//! Write pattern: temp file + fsync + rename (same as credential state files in
//! S-1.08 / S-1.09).
//!
//! File-first ordering (mandatory per BC-2.11.008):
//! 1. Validate against in-memory state (cycle/depth checks, keyword/OCSF collision,
//!    parse validation).
//! 2. Write `aliases.toml` atomically. If write fails → `E-IO-001`; in-memory
//!    registry unchanged.
//! 3. THEN update the in-memory alias registry.
//!
//! Story: S-3.04 — prism-query: Alias System (P1)
//! BCs:   BC-2.11.008, BC-2.11.013, BC-2.11.014

use std::path::Path;

use prism_core::error::PrismError;

use crate::alias_types::{AliasEntry, AliasScope, CreateResult, DeleteResult};

// ─────────────────────────────────────────────────────────────────────────────
// AliasStore
// ─────────────────────────────────────────────────────────────────────────────

/// In-memory alias registry backed by `aliases.toml` on disk.
///
/// Indexed by `(name, scope)`. Thread-safety is provided by the caller —
/// `AliasStore` is not `Sync` by itself; callers wrap it in `Arc<Mutex<...>>`.
pub struct AliasStore {
    /// Path to the `aliases.toml` file on disk.
    #[allow(dead_code)]
    path: std::path::PathBuf,
    /// In-memory alias registry.
    #[allow(dead_code)]
    entries: Vec<AliasEntry>,
}

impl AliasStore {
    /// Deserialize `aliases.toml` at startup and validate all entries.
    ///
    /// Validation includes:
    /// - All referenced alias names exist in the store.
    /// - No cycles (DI-020 invariant — store must be cycle-free at load time).
    /// - All composition depths ≤ 3.
    ///
    /// Returns `Err(E-IO-001)` if the file cannot be read.
    /// Returns `Err(E-ALIAS-002)` or `Err(E-ALIAS-003)` on constraint violations.
    pub fn load(_path: &Path) -> Result<Self, PrismError> {
        todo!()
    }

    /// Create an empty `AliasStore` with no aliases (used for testing and
    /// when `aliases.toml` does not yet exist at startup).
    pub fn empty(path: impl AsRef<Path>) -> Self {
        AliasStore {
            path: path.as_ref().to_path_buf(),
            entries: Vec::new(),
        }
    }

    /// Create or update an alias entry.
    ///
    /// # File-first write sequence
    /// 1. Validate capability (`alias.write`), name format, keyword/OCSF collision,
    ///    parameter defaults, cycle detection, and depth limit.
    /// 2. If alias already exists at `(entry.name, entry.scope)`: return
    ///    `CreateResult::ConfirmationRequired` with a `ConfirmationToken` — do NOT
    ///    persist until `confirm_action` is called.
    /// 3. If alias is new: write `aliases.toml` atomically (temp + fsync + rename),
    ///    then update in-memory registry.
    ///
    /// `token` is `Some(...)` when this call is the `confirm_action` path for an
    /// update operation.
    ///
    /// Returns `Err(E-ALIAS-002)` on cycle, `Err(E-ALIAS-003)` on depth exceeded,
    /// `Err(E-ALIAS-006)` on keyword/OCSF collision, `Err(E-IO-001)` on write
    /// failure.
    pub fn create_or_update(
        &mut self,
        _entry: AliasEntry,
        _token: Option<prism_security::ConfirmationToken>,
    ) -> Result<CreateResult, PrismError> {
        todo!()
    }

    /// Retrieve an alias by `(name, scope)`.
    ///
    /// Returns `Ok(Some(&entry))` if found, `Ok(None)` if absent.
    pub fn get(&self, _name: &str, _scope: &AliasScope) -> Result<Option<&AliasEntry>, PrismError> {
        todo!()
    }

    /// List aliases, optionally filtered by scope.
    ///
    /// - `None` → return all aliases (global + all per-client), sorted
    ///   alphabetically by name within each scope group (BC-2.11.013).
    /// - `Some(Global)` → return only global aliases.
    /// - `Some(Client(id))` → return only aliases for that client; does NOT
    ///   include global aliases.
    pub fn list(&self, _scope_filter: Option<&AliasScope>) -> Vec<&AliasEntry> {
        todo!()
    }

    /// Delete an alias after confirmation.
    ///
    /// Deletion ALWAYS requires a `ConfirmationToken` (BC-2.11.014).
    ///
    /// # Steps
    /// 1. Check alias exists; return `E-ALIAS-001` if not.
    /// 2. Resolve dependents via `AliasStore::dependents()`.
    /// 3. If dependents exist and `force` is `false`: return `E-ALIAS-005`.
    /// 4. If dependents exist and `force` is `true`: cascade-delete all dependents
    ///    (re-resolved at confirmation time).
    /// 5. Write `aliases.toml` atomically; return `E-IO-001` on failure without
    ///    touching in-memory registry.
    /// 6. Update in-memory registry.
    /// 7. Emit audit entry (DI-004).
    pub fn delete(
        &mut self,
        _name: &str,
        _scope: &AliasScope,
        _force: bool,
        _token: prism_security::ConfirmationToken,
    ) -> Result<DeleteResult, PrismError> {
        todo!()
    }

    /// Scan all alias definitions for references to `@name` in the given scope.
    ///
    /// Returns the names of all aliases (in any scope) whose `query` field
    /// contains a `@name` reference. O(n) over all entries; acceptable at
    /// expected cardinality (< 1,000 per deployment).
    pub fn dependents(&self, _name: &str, _scope: &AliasScope) -> Vec<String> {
        todo!()
    }

    /// Atomically write all current entries to `aliases.toml`.
    ///
    /// Uses temp-file + fsync + rename pattern. Returns `Err(E-IO-001)` on
    /// any I/O failure. In-memory state is NOT modified by this method.
    #[allow(dead_code)]
    fn write_file(&self) -> Result<(), PrismError> {
        todo!()
    }
}
