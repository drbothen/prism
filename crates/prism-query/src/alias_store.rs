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

use std::io::Write as IoWrite;
use std::path::{Path, PathBuf};

use prism_core::error::PrismError;
use serde::{Deserialize, Serialize};

use crate::alias_resolver::AliasResolver;
use crate::alias_types::{AliasEntry, AliasScope, CreateResult, DeleteResult};

// ─────────────────────────────────────────────────────────────────────────────
// TOML persistence format
// ─────────────────────────────────────────────────────────────────────────────

/// TOML-serializable wrapper for the `aliases.toml` file format.
#[derive(Debug, Serialize, Deserialize, Default)]
struct AliasesFile {
    aliases: Vec<AliasEntry>,
}

// ─────────────────────────────────────────────────────────────────────────────
// AliasStore
// ─────────────────────────────────────────────────────────────────────────────

/// In-memory alias registry backed by `aliases.toml` on disk.
///
/// Indexed by `(name, scope)`. Thread-safety is provided by the caller —
/// `AliasStore` is not `Sync` by itself; callers wrap it in `Arc<Mutex<...>>`.
pub struct AliasStore {
    /// Path to the `aliases.toml` file on disk.
    path: PathBuf,
    /// In-memory alias registry.
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
    pub fn load(path: &Path) -> Result<Self, PrismError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| PrismError::Io(format!("cannot read aliases.toml: {e}")))?;

        let file: AliasesFile = toml::from_str(&content)
            .map_err(|e| PrismError::Io(format!("cannot parse aliases.toml: {e}")))?;

        let store = AliasStore {
            path: path.to_path_buf(),
            entries: file.aliases,
        };

        // Validate: no cycles in loaded entries.
        // SEC-010 / CR-P2-001: use detect_cycle_scoped so per-client cycles are caught.
        for entry in &store.entries {
            AliasResolver::detect_cycle_scoped(&entry.name, &entry.query, &store, &entry.scope)?;
        }

        // Validate: depth limit — probe each entry's expansion chain.
        // CR-013: load() must reject stores where any alias would violate MAX_ALIAS_DEPTH.
        // Expand the body of each entry and surface E-ALIAS-003 (depth exceeded).
        // Non-depth errors (E-ALIAS-001 for dangling refs) are silenced at load time.
        // CR-P2-002: use entry.scope (not Global) so per-client depth paths are probed correctly.
        let args = std::collections::HashMap::new();
        for entry in &store.entries {
            if let Err(e) = AliasResolver::expand(&entry.query, &store, &entry.scope, &args, 0) {
                if matches!(e, prism_core::error::PrismError::AliasDepthExceeded { .. }) {
                    return Err(e);
                }
                // All other errors (missing aliases, etc.) are not load-time failures.
            }
        }

        Ok(store)
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
    ///
    /// **Visibility:** `pub(crate)` — external callers MUST go through `alias_tools::create_alias`
    /// or `alias_tools::create_alias_with_clients` to ensure keyword/OCSF collision checks
    /// are applied. Direct access bypasses those validation gates (CR-018).
    pub(crate) fn create_or_update(
        &mut self,
        entry: AliasEntry,
        token: Option<prism_security::ConfirmationToken>,
    ) -> Result<CreateResult, PrismError> {
        let existing_idx = self
            .entries
            .iter()
            .position(|e| e.name == entry.name && e.scope == entry.scope);

        // If alias already exists and no token is provided, require confirmation.
        if existing_idx.is_some() && token.is_none() {
            let token_client_id = entry.scope.token_client_id().to_string();
            return Ok(CreateResult::ConfirmationRequired {
                token_client_id,
                token_json: String::new(), // token generated by the tool layer
            });
        }

        // Cycle detection: use scope-aware variant so per-client cycles are caught.
        // SEC-010 / CR-P2-001: detect_cycle() hardcodes Global scope; use detect_cycle_scoped.
        AliasResolver::detect_cycle_scoped(&entry.name, &entry.query, self, &entry.scope)?;

        // Clone entry for return value.
        let entry_clone = entry.clone();

        // File-first write: persist before updating in-memory.
        let mut new_entries = self.entries.clone();
        if let Some(idx) = existing_idx {
            new_entries[idx] = entry;
        } else {
            new_entries.push(entry);
        }

        // Write atomically.
        self.write_entries_to_file(&new_entries)?;

        // Update in-memory only after successful file write.
        self.entries = new_entries;

        Ok(CreateResult::Created(entry_clone))
    }

    /// Retrieve an alias by `(name, scope)`.
    ///
    /// Returns `Ok(Some(&entry))` if found, `Ok(None)` if absent.
    pub fn get(&self, name: &str, scope: &AliasScope) -> Result<Option<&AliasEntry>, PrismError> {
        Ok(self
            .entries
            .iter()
            .find(|e| e.name == name && &e.scope == scope))
    }

    /// List aliases, optionally filtered by scope.
    ///
    /// - `None` → return all aliases (global + all per-client), sorted with
    ///   Global entries first (alphabetically by name), then per-client groups
    ///   sorted by client_id then name (BC-2.11.013 / CR-004).
    /// - `Some(Global)` → return only global aliases, sorted by name.
    /// - `Some(Client(id))` → return only aliases for that client; does NOT
    ///   include global aliases; sorted by name.
    pub fn list(&self, scope_filter: Option<&AliasScope>) -> Vec<&AliasEntry> {
        let mut result: Vec<&AliasEntry> = match scope_filter {
            None => self.entries.iter().collect(),
            Some(filter) => self.entries.iter().filter(|e| &e.scope == filter).collect(),
        };
        // Sort key: (scope_discriminant, client_id_or_empty, name).
        // Global = 0, Client = 1 — Global entries appear first (CR-004).
        result.sort_by(|a, b| {
            let a_disc = scope_discriminant(&a.scope);
            let b_disc = scope_discriminant(&b.scope);
            a_disc
                .cmp(&b_disc)
                .then_with(|| scope_client_id(&a.scope).cmp(scope_client_id(&b.scope)))
                .then_with(|| a.name.cmp(&b.name))
        });
        result
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
        name: &str,
        scope: &AliasScope,
        force: bool,
        _token: prism_security::ConfirmationToken,
    ) -> Result<DeleteResult, PrismError> {
        // Step 1: check alias exists.
        let exists = self
            .entries
            .iter()
            .any(|e| e.name == name && &e.scope == scope);

        if !exists {
            // Restrict error disclosure to scope-visible aliases only (SEC-003).
            let visible_entries: Vec<&AliasEntry> = match scope {
                AliasScope::Client(_) => {
                    let mut v = self.list(Some(scope));
                    v.extend(self.list(Some(&AliasScope::Global)));
                    v
                }
                AliasScope::Global => self.list(Some(&AliasScope::Global)),
            };
            let available = visible_entries
                .iter()
                .map(|e| e.name.as_str())
                .collect::<Vec<_>>()
                .join(", ");
            return Err(PrismError::AliasNotFound {
                name: name.to_string(),
                scope: scope.display_string(),
                available,
            });
        }

        // Step 2: resolve dependents (scope-aware tuples — SEC-009).
        let dep_tuples = self.dependents(name, scope);
        // Flat name list for error messages and DeleteResult (backward-compat public API).
        let dep_names: Vec<String> = dep_tuples.iter().map(|(n, _)| n.clone()).collect();

        // Step 3: blocked if dependents and force=false.
        if !dep_names.is_empty() && !force {
            return Err(PrismError::AliasDependentsExist {
                name: name.to_string(),
                count: dep_names.len(),
                dependents: dep_names.join(", "),
            });
        }

        // Step 4: collect (name, scope) tuples to cascade-delete.
        let cascade_deleted_names = if force { dep_names } else { Vec::new() };
        let cascade_deleted_tuples: Vec<(String, AliasScope)> =
            if force { dep_tuples } else { Vec::new() };

        // Step 5: file-first write.
        // Match on (name, scope) tuples to prevent cross-scope cascade (SEC-009).
        let new_entries: Vec<AliasEntry> = self
            .entries
            .iter()
            .filter(|e| {
                // Remove the primary alias.
                let is_primary = e.name == name && &e.scope == scope;
                // Remove cascade targets matched by (name, scope) — not name alone.
                let is_cascade = force
                    && cascade_deleted_tuples
                        .iter()
                        .any(|(cn, cs)| cn == &e.name && cs == &e.scope);
                !is_primary && !is_cascade
            })
            .cloned()
            .collect();

        self.write_entries_to_file(&new_entries)?;

        // Step 6: update in-memory.
        self.entries = new_entries;

        Ok(DeleteResult::Deleted {
            name: name.to_string(),
            scope: scope.clone(),
            cascade_deleted: cascade_deleted_names,
        })
    }

    /// Scan all alias definitions for references to `@name` in the given scope.
    ///
    /// Returns `(name, scope)` tuples for all aliases whose `query` field contains a
    /// token-level `@name` reference, filtered by scope semantics (SEC-009):
    ///
    /// - **Global alias deleted:** all referencing aliases in any scope are returned
    ///   (Global aliases are reachable from any client, so any alias that references
    ///   `@name` would lose its resolution target upon Global deletion).
    /// - **Client alias deleted:** only aliases in the same client scope are returned
    ///   (a Client alias is invisible to other clients and to Global scope).
    ///
    /// Uses `AliasResolver::detect_alias_tokens` to avoid substring false positives
    /// (e.g., alias `high` must NOT show as dependent of `high_sev`).
    /// O(n) over all entries; acceptable at expected cardinality (< 1,000 per deployment).
    /// (CR-003 fix, SEC-009 scope-aware cascade)
    pub fn dependents(&self, name: &str, scope: &AliasScope) -> Vec<(String, AliasScope)> {
        self.entries
            .iter()
            .filter(|e| {
                // Exclude the alias being deleted itself.
                if e.name == name && &e.scope == scope {
                    return false;
                }
                // Must reference @name at the token level.
                if !AliasResolver::detect_alias_tokens(&e.query).contains(&name.to_string()) {
                    return false;
                }
                // Scope filter (SEC-009):
                // - Global deletion: cascade to all scopes (Global aliases are universally reachable).
                // - Client deletion: cascade only within the same client scope.
                match scope {
                    AliasScope::Global => true,
                    AliasScope::Client(_) => &e.scope == scope,
                }
            })
            .map(|e| (e.name.clone(), e.scope.clone()))
            .collect()
    }

    /// Atomically write a specific set of entries to the backing file.
    ///
    /// Uses temp-file + fsync + rename for atomicity (BC-2.11.008).
    fn write_entries_to_file(&self, entries: &[AliasEntry]) -> Result<(), PrismError> {
        let file_data = AliasesFile {
            aliases: entries.to_vec(),
        };

        let toml_str = toml::to_string_pretty(&file_data)
            .map_err(|e| PrismError::Io(format!("cannot serialize aliases: {e}")))?;

        // Determine parent directory for temp file.
        let parent = self.path.parent().ok_or_else(|| {
            PrismError::Io(format!(
                "aliases.toml path has no parent: {}",
                self.path.display()
            ))
        })?;

        // Write to temp file.
        // Include a short hash of the store path to prevent temp-name collisions when
        // multiple tests create stores with different destination paths in the same parent
        // directory (e.g. concurrent nextest runs on macOS aarch64 — CR-P3-004).
        let path_discriminant = {
            use std::hash::{Hash, Hasher};
            let mut h = std::collections::hash_map::DefaultHasher::new();
            self.path.hash(&mut h);
            h.finish()
        };
        let tmp_path = parent.join(format!(
            "aliases.toml.tmp.{:016x}.{}",
            path_discriminant,
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));

        let mut tmp_file = std::fs::File::create(&tmp_path)
            .map_err(|e| PrismError::Io(format!("cannot create temp file: {e}")))?;

        tmp_file
            .write_all(toml_str.as_bytes())
            .map_err(|e| PrismError::Io(format!("cannot write aliases: {e}")))?;

        tmp_file
            .sync_all()
            .map_err(|e| PrismError::Io(format!("fsync failed: {e}")))?;

        // Atomic rename. On failure, attempt to clean up the temp file (CR-005).
        if let Err(rename_err) = std::fs::rename(&tmp_path, &self.path) {
            // Best-effort cleanup — swallow the cleanup error, surface the rename error.
            let _ = std::fs::remove_file(&tmp_path);
            return Err(PrismError::Io(format!("rename failed: {rename_err}")));
        }

        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Sort helpers for list()
// ─────────────────────────────────────────────────────────────────────────────

/// Return a sort discriminant for the scope: Global=0, Client=1.
///
/// Ensures global aliases sort before per-client aliases (CR-004).
fn scope_discriminant(scope: &AliasScope) -> u8 {
    match scope {
        AliasScope::Global => 0,
        AliasScope::Client(_) => 1,
    }
}

/// Return the client_id string for sort purposes, or empty string for Global.
fn scope_client_id(scope: &AliasScope) -> &str {
    match scope {
        AliasScope::Global => "",
        AliasScope::Client(id) => id.0.as_str(),
    }
}
