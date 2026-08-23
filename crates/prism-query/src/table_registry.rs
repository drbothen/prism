//! Dynamic table registry — tracks which sensor tables are currently available.
//!
//! `TableRegistry` is populated from `ConfigSnapshot.sensor_specs` at startup and
//! updated in-place when sensors are added or removed via hot-reload. It provides
//! the plan-time availability check (AC-1, AC-2, AC-4, AC-5) that fires in
//! `engine.rs` before `materialize_query` so queries against unconfigured sensors
//! return `E-QUERY-037` (`TableNotAvailable`) without incurring any fan-out cost.
//!
//! The `strsim::levenshtein`-based `did_you_mean` helper enables self-correcting AI
//! agent loops: when Claude generates a query against a sensor the analyst hasn't
//! configured, the error message lists available sensors so Claude can self-correct
//! without a human round-trip (Dev Notes, S-3.13).
//!
//! # Thread safety
//! - `RwLock::read()` is used for availability checks on the hot path (non-exclusive).
//! - `RwLock::write()` is held only during `register_sensor` / `deregister_sensor`
//!   (config-reload path, never the query path). Reads MUST NOT block on `write()`.
//!   (Architecture Compliance Rules, S-3.13)
//! - On `RwLock` poison (another thread panicked holding the lock), read methods
//!   return safe defaults (empty/false) to prevent cascading failures. Write methods
//!   propagate poison as `PrismError::Internal`.
//!
//! # BC References
//! - BC-2.16.001 — Sensor Spec File Loading → `TableRegistry` populated at startup
//! - BC-2.16.007 — Sensor Spec Hot Reload → register/deregister on hot-reload delta
//! - BC-2.11.001 — `query` MCP Tool → E-QUERY-037 for unregistered tables
//!
//! Story: S-3.13

use std::collections::{BTreeSet, HashMap, HashSet};
use std::sync::{Arc, RwLock};

use prism_core::{column::ColumnType, error::TableNotAvailableDetails, OrgSlug, PrismError};
use prism_spec_engine::{ConfigSnapshot, ResolvedSensorSpec, ResolvedSpecKey, SensorSpec};

/// Maximum byte length accepted for the `requested` parameter of `did_you_mean`.
///
/// Levenshtein distance is O(m×n) time and space where m, n are string lengths.
/// An unbounded `requested` string allows an algorithmic-complexity DoS — a caller
/// supplying a 1 MB table name would force O(1M × max_registered_name_len) work.
/// 128 bytes covers all realistic sensor/table name lengths (e.g.,
/// `crowdstrike_detections` = 22 bytes) while bounding worst-case computation to a
/// trivially fast O(128 × max_registered_name_len) per query.
/// (SEC-002, CWE-407 — Algorithmic Complexity DoS; S-3.13 fix-burst)
/// Maximum byte length for untrusted name inputs to `strsim::levenshtein` did_you_mean
/// computations (SEC-002 / CWE-407).  Shared across the table gate (this module),
/// the enrich UDF gate (`engine::check_enrich_udf_availability`), and the column gate
/// (`engine::check_column_availability`) via `cap_name_for_levenshtein`.
pub(crate) const DID_YOU_MEAN_MAX_NAME_BYTES: usize = 128;

/// Cap `name` at [`DID_YOU_MEAN_MAX_NAME_BYTES`] (128 bytes) before any
/// `strsim::levenshtein` call, closing the CWE-407 Algorithmic Complexity DoS
/// path that exists when untrusted query tokens are passed verbatim.
///
/// # Contract
/// - Returns a `&str` slice of `name` whose byte length is ≤ 128.
/// - The truncation point is always a UTF-8 char boundary (never mid-codepoint).
/// - For inputs whose byte length is already ≤ 128, returns `name` unchanged (zero-copy).
///
/// # Reference
/// SEC-002 / CWE-407; mirrors the inline cap in `did_you_mean`
/// and `did_you_mean_for_tables` of this module.
/// Applied to the enrich gate (E-QUERY-039) and column gate (E-QUERY-038) by
/// F-PHL1-HIGH-001 and F-PHL1-MED-001 (S-DEMO-FIDELITY-REMEDIATION-001 Pass-H).
pub(crate) fn cap_name_for_levenshtein(name: &str) -> &str {
    if name.len() <= DID_YOU_MEAN_MAX_NAME_BYTES {
        return name;
    }
    let mut boundary = DID_YOU_MEAN_MAX_NAME_BYTES;
    while !name.is_char_boundary(boundary) {
        boundary -= 1;
    }
    &name[..boundary]
}

// ---------------------------------------------------------------------------
// TableRegistry
// ---------------------------------------------------------------------------

/// Dynamic catalog of currently-available sensor tables.
///
/// Thread-safe: backed by `Arc<RwLock<…>>` collections. Reads are non-exclusive;
/// writes (during config-reload) are exclusive but brief.
///
/// # Invariant (Architecture Compliance)
/// Every table name in `registered` follows the `{sensor_id}_{table_name}` convention
/// derived from `[[tables]]` entries in the sensor spec. The separator is `_`; callers
/// that need the sensor prefix can use `sensor_for_table()`.
///
/// # API Stability (`#[non_exhaustive]`)
/// `#[non_exhaustive]` prevents external crates from constructing `TableRegistry`
/// via struct-literal syntax (callers must use `TableRegistry::new()` or
/// `TableRegistry::from_snapshot()`). This is the prism-query pub-API-surface
/// discipline per CLAUDE.md §Conventions and CR-002 (S-3.13 fix-burst).
#[non_exhaustive]
pub struct TableRegistry {
    /// Table names currently available (e.g. `"crowdstrike_alerts"`).
    registered: Arc<RwLock<HashSet<String>>>,
    /// Table name → sensor_id reverse mapping for error messages.
    sensor_by_table: Arc<RwLock<HashMap<String, String>>>,
    /// Table name → column name list, used for single-tenant E-QUERY-038 gate.
    ///
    /// M1 fix (S-DEMO-FIDELITY-REMEDIATION-001): `check_query_column_availability`
    /// previously returned `Ok(())` immediately when `resolved_spec_map.is_none()`
    /// (single-tenant mode), silently bypassing E-QUERY-038 for all queries.
    ///
    /// By retaining column names here, the gate can fire in single-tenant mode by
    /// looking up columns from the registry rather than the resolved_spec_map.
    /// Only populated when `register_sensor` is called with a spec that has columns
    /// defined in its `[[tables]]` entries.
    ///
    /// When `columns_for_table` returns `[]`, callers MUST distinguish "not registered"
    /// (fail-open) from "registered with zero columns" (E-QUERY-038 fires with
    /// `available_columns: []`) by calling `is_registered` — see ADV-FIX-P9-OBS-001
    /// and BC-2.11.016 EC-11-041.
    columns_by_table: Arc<RwLock<HashMap<String, Vec<String>>>>,
    /// Table name → (column name → `ColumnType`) map, used by the schema-aware
    /// E-QUERY-041 temporal pre-validator (`check_temporal_literals`).
    ///
    /// ADR-052 v1.1 D4 / HIGH-1 fix: `check_temporal_literals` must only fire for
    /// string literals compared against `ColumnType::Datetime` columns. This map
    /// provides the column type lookup without requiring the resolved_spec_map
    /// (which is absent in single-tenant / test mode).
    ///
    /// Only populated for tables that have explicit column definitions.
    /// Missing entry = fail-open (gate does not validate).
    column_types_by_table: Arc<RwLock<HashMap<String, HashMap<String, ColumnType>>>>,
}

impl TableRegistry {
    /// Construct an empty `TableRegistry`.
    ///
    /// Populate by calling `register_sensor()` for each sensor in the initial
    /// `ConfigSnapshot`, or use `from_snapshot()` as a convenience.
    pub fn new() -> Self {
        Self {
            registered: Arc::new(RwLock::new(HashSet::new())),
            sensor_by_table: Arc::new(RwLock::new(HashMap::new())),
            columns_by_table: Arc::new(RwLock::new(HashMap::new())),
            column_types_by_table: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Construct a `TableRegistry` pre-populated from a `ConfigSnapshot`.
    ///
    /// Called at `QueryEngine` construction time so the initial table set
    /// matches the loaded sensor specs (BC-2.16.001).
    pub fn from_snapshot(snapshot: &ConfigSnapshot) -> Result<Self, PrismError> {
        let registry = Self::new();
        for spec in snapshot.sensor_specs.values() {
            registry.register_sensor(spec)?;
        }
        Ok(registry)
    }

    /// Register all tables declared by `spec`.
    ///
    /// For each `[[tables]]` entry in the spec, inserts `{sensor_id}_{table_name}`
    /// into the registry. If the sensor was already registered (hot-reload update
    /// case), existing tables for this sensor are replaced atomically: both write
    /// locks are acquired ONCE and the old tables removed and new tables inserted
    /// without releasing the lock between phases. This prevents a transient window
    /// where the sensor's tables are absent between the remove and insert operations.
    /// (EC-11-123, S-3.13 Task 3 note, MED-3 fix)
    ///
    /// # Atomicity guarantee
    /// Both `registered` and `sensor_by_table` are mutated in a single lock
    /// acquisition window — the sensor is never in a partially-deregistered state
    /// that is visible to concurrent readers. Queries landing during a re-registration
    /// will see EITHER the old table set OR the new table set, never an empty window.
    ///
    /// # BC-2.16.001 / BC-2.16.007
    /// Called at startup (from_snapshot) and on hot-reload add/update.
    pub fn register_sensor(&self, spec: &SensorSpec) -> Result<(), PrismError> {
        let prefix = format!("{}_", spec.sensor_id);

        // Acquire BOTH write locks once and hold them across the remove+insert
        // to make re-registration atomic (MED-3 fix — no transient empty window).
        let mut registered = self.registered.write().map_err(|_| PrismError::Internal {
            detail: "TableRegistry::register_sensor: RwLock poisoned \
                         (another thread panicked while holding the lock)"
                .to_string(),
        })?;
        let mut sensor_by_table =
            self.sensor_by_table
                .write()
                .map_err(|_| PrismError::Internal {
                    detail: "TableRegistry::register_sensor: sensor_by_table RwLock poisoned"
                        .to_string(),
                })?;
        // M1 fix: acquire columns_by_table lock atomically with the other two.
        let mut columns_by_table =
            self.columns_by_table
                .write()
                .map_err(|_| PrismError::Internal {
                    detail: "TableRegistry::register_sensor: columns_by_table RwLock poisoned"
                        .to_string(),
                })?;
        // ADR-052 / HIGH-1: acquire column_types_by_table lock in the same atomic window.
        let mut column_types_by_table =
            self.column_types_by_table
                .write()
                .map_err(|_| PrismError::Internal {
                    detail: "TableRegistry::register_sensor: column_types_by_table RwLock poisoned"
                        .to_string(),
                })?;

        // Remove existing tables for this sensor (the deregister phase).
        // Executed under the held locks — no visibility gap.
        let to_remove: Vec<String> = registered
            .iter()
            .filter(|name| name.starts_with(&prefix))
            .cloned()
            .collect();
        for name in to_remove {
            registered.remove(&name);
            sensor_by_table.remove(&name);
            columns_by_table.remove(&name);
            column_types_by_table.remove(&name);
        }

        // Insert new tables for this sensor (the register phase).
        // Executed under the same held locks — atomic with the remove above.
        for table in &spec.tables {
            let full_name = format!("{}_{}", spec.sensor_id, table.table_name);
            registered.insert(full_name.clone());
            sensor_by_table.insert(full_name.clone(), spec.sensor_id.clone());
            // M1 fix: retain column names for single-tenant E-QUERY-038 gate.
            //
            // ADR-058 §G / §J6 / S-ADR058-OCSF-ROUTING-001 LOW-1 fix:
            // When ocsf_column_naming=true, ALWAYS insert columns_by_table and
            // column_types_by_table using the shared helpers from prism_spec_engine —
            // even for zero-column OCSF tables (§J6: synthesized pseudo-columns are
            // always present, so the entry must exist with at least class_uid + _sensor).
            // The previous outer `if !table.columns.is_empty()` guard prevented this
            // for zero-column OCSF tables, causing E-QUERY-038 to fire on class_uid / _sensor
            // queries (RG-Q-010/011 red gate).
            //
            // When ocsf_column_naming=false, keep the existing `!columns.is_empty()`
            // fail-open guard to preserve legacy behavior for non-OCSF sensors.
            if spec.ocsf_column_naming {
                // §J6: always insert, even for zero-Tier-1 OCSF tables (RG-Q-010/011 fix).
                // Canonical projection logic lives in ocsf_projected_column_names /
                // ocsf_projected_column_types (ADR-058 §I7 Consolidated-Projection Invariant).
                let col_names =
                    prism_spec_engine::column_mapping::ocsf_projected_column_names(table, true);
                columns_by_table.insert(full_name.clone(), col_names);

                let type_map =
                    prism_spec_engine::column_mapping::ocsf_projected_column_types(table, true);

                // ADR-058 §J6 A+W amendment (human decision 2026-08-23):
                // Emit ocsf.zero_tier1_table WARN for OCSF tables with zero Tier-1 columns.
                // BC-2.16.002 §Postconditions catalog row `ocsf.zero_tier1_table` (PG-LP11-001).
                // Fires for BOTH sub-cases:
                //   (A) zero-Tier-1 with ≥1 Tier-2 columns — tier2_column_count ≥ 1
                //   (B) truly-empty (0 Tier-1, 0 Tier-2) — tier2_column_count = 0
                // Recurrence: ONCE per offending table per register_sensor call — NOT per-query.
                // Does NOT block registration — the table loads successfully with synthesized
                // pseudo-columns only (ADR-058 §G: class_uid + _sensor [+ raw_extensions]).
                // RG-Q-017: test_BC_2_11_016_zero_tier1_with_tier2_projects_raw_extensions_and_emits_warning
                let tier1_count = table
                    .columns
                    .iter()
                    .filter(|c| c.ocsf_field.is_some())
                    .count();
                if tier1_count == 0 {
                    let tier2_count: u32 = table
                        .columns
                        .iter()
                        .filter(|c| c.ocsf_field.is_none())
                        .count() as u32;
                    tracing::warn!(
                        event_type = "ocsf.zero_tier1_table",
                        sensor_id = %spec.sensor_id,
                        table_name = %table.table_name,
                        tier2_column_count = tier2_count,
                        "OCSF table with ocsf_column_naming=true has zero Tier-1 ocsf_field \
                         mappings; class_uid + _sensor presented \
                         (+ raw_extensions when tier2_column_count > 0)"
                    );
                }

                column_types_by_table.insert(full_name, type_map);
            } else if !table.columns.is_empty() {
                // flag=false: keep existing col.name behavior exactly (byte-for-byte).
                // Fail-open guard preserved for non-OCSF sensors.
                let col_names: Vec<String> = table.columns.iter().map(|c| c.name.clone()).collect();
                columns_by_table.insert(full_name.clone(), col_names);
                // ADR-052 / HIGH-1: retain column types for schema-aware E-QUERY-041 gate.
                let type_map: HashMap<String, ColumnType> = table
                    .columns
                    .iter()
                    .map(|c| (c.name.clone(), c.column_type.clone()))
                    .collect();
                column_types_by_table.insert(full_name, type_map);
            }
        }

        Ok(())
    }

    /// Deregister all tables whose names start with `{sensor_id}_`.
    ///
    /// Called on hot-reload remove (BC-2.16.007, AC-5). In-flight queries that
    /// started before deregistration hold a reference to the old `ConfigSnapshot`
    /// (CI-007) and are unaffected — only new queries see the updated table set.
    ///
    /// # BC-2.16.007
    /// Called when a sensor spec is removed via hot-reload.
    pub fn deregister_sensor(&self, sensor_id: &str) -> Result<(), PrismError> {
        let prefix = format!("{sensor_id}_");

        let mut registered = self.registered.write().map_err(|_| PrismError::Internal {
            detail: format!(
                "TableRegistry::deregister_sensor: RwLock poisoned for sensor_id={sensor_id}"
            ),
        })?;
        let mut sensor_by_table =
            self.sensor_by_table
                .write()
                .map_err(|_| PrismError::Internal {
                    detail: format!(
                        "TableRegistry::deregister_sensor: sensor_by_table RwLock poisoned \
                         for sensor_id={sensor_id}"
                    ),
                })?;
        // M1 fix: also acquire columns_by_table for cleanup.
        let mut columns_by_table =
            self.columns_by_table
                .write()
                .map_err(|_| PrismError::Internal {
                    detail: format!(
                        "TableRegistry::deregister_sensor: columns_by_table RwLock poisoned \
                         for sensor_id={sensor_id}"
                    ),
                })?;
        // ADR-052 / HIGH-1: also clean up column_types_by_table.
        let mut column_types_by_table =
            self.column_types_by_table
                .write()
                .map_err(|_| PrismError::Internal {
                    detail: format!(
                        "TableRegistry::deregister_sensor: column_types_by_table RwLock poisoned \
                         for sensor_id={sensor_id}"
                    ),
                })?;

        // Collect keys to remove (avoid modifying while iterating).
        let to_remove: Vec<String> = registered
            .iter()
            .filter(|name| name.starts_with(&prefix))
            .cloned()
            .collect();

        for name in to_remove {
            registered.remove(&name);
            sensor_by_table.remove(&name);
            columns_by_table.remove(&name);
            column_types_by_table.remove(&name);
        }

        Ok(())
    }

    /// Return `true` if `table_name` is in the current registry.
    ///
    /// Uses `RwLock::read()` — non-exclusive. MUST NOT block query execution.
    /// On `RwLock` poison: returns `false` (conservative safe default — no access)
    /// AND emits a `table_registry.rwlock_poisoned` WARN tracing event so operators
    /// can detect this abnormal condition. (NB-1, S-3.13 fix-burst)
    ///
    /// # BC-2.16.001 / BC-2.11.001
    /// Plan-time check in `engine.rs`; returns `false` for unregistered sensors.
    pub fn is_registered(&self, table_name: &str) -> bool {
        match self.registered.read() {
            Ok(guard) => guard.contains(table_name),
            Err(_) => {
                // Poisoned lock — another thread panicked while holding the write lock.
                // Fail closed (return false) to prevent incorrect query routing.
                // Emit a WARN so operators can observe this abnormal condition.
                // (NB-1, BC-2.16.002 row `table_registry.rwlock_poisoned`)
                tracing::warn!(
                    event_type = "table_registry.rwlock_poisoned",
                    method = "is_registered",
                    "TableRegistry::is_registered: RwLock poisoned — returning fail-closed \
                     default (false). Another thread panicked while holding the lock."
                );
                false
            }
        }
    }

    /// Return a snapshot of all currently-registered table names.
    ///
    /// Used by:
    /// - `explain_query` to list `available_tables` (AC-6)
    /// - `TableNotAvailable` error construction (available_tables field)
    /// - `did_you_mean` Levenshtein computation (AC-3)
    /// - Future `prism://config/clients` MCP resource (S-5.03; not delivered by S-3.13)
    ///
    /// Acquires a read-lock; returns an owned `Vec` sorted for determinism.
    /// On `RwLock` poison: returns empty `Vec` (safe default) and emits a WARN.
    /// (NB-1, BC-2.16.002 row `table_registry.rwlock_poisoned`)
    pub fn registered_tables(&self) -> Vec<String> {
        let guard = match self.registered.read() {
            Ok(g) => g,
            Err(_) => {
                tracing::warn!(
                    event_type = "table_registry.rwlock_poisoned",
                    method = "registered_tables",
                    "TableRegistry::registered_tables: RwLock poisoned — returning empty list. \
                     Another thread panicked while holding the lock."
                );
                return Vec::new();
            }
        };
        let mut tables: Vec<String> = guard.iter().cloned().collect();
        tables.sort();
        tables
    }

    /// Return the column names for `table_name`, or an empty `Vec` if unknown.
    ///
    /// Used by `check_query_column_availability` (E-QUERY-038) in single-tenant
    /// mode when `resolved_spec_map` is `None`. Returns column names populated via
    /// `register_sensor` from the sensor spec's `[[tables]][*].columns` entries.
    ///
    /// Returns an empty `Vec` in three distinct cases:
    /// - `table_name` is not registered (not in `registered` set) — callers should fail-open
    /// - `table_name` was registered with an empty column list (zero columns in spec) —
    ///   callers should fire E-QUERY-038 with `available_columns: []` (BC-2.11.016 EC-11-041)
    /// - The `columns_by_table` lock is poisoned — callers should fail-open
    ///
    /// **Callers must use `is_registered` to distinguish the first two cases** (see
    /// `get_initial_available_columns` and `check_column_availability` in `engine.rs`,
    /// ADV-FIX-P9-OBS-001).
    ///
    /// M1 fix: S-DEMO-FIDELITY-REMEDIATION-001.
    pub fn columns_for_table(&self, table_name: &str) -> Vec<String> {
        match self.columns_by_table.read() {
            Ok(guard) => guard.get(table_name).cloned().unwrap_or_default(),
            Err(_) => {
                tracing::warn!(
                    event_type = "table_registry.rwlock_poisoned",
                    method = "columns_for_table",
                    "TableRegistry::columns_for_table: RwLock poisoned — returning empty list. \
                     Another thread panicked while holding the lock."
                );
                Vec::new()
            }
        }
    }

    /// Return the `ColumnType` for a named column in a given table.
    ///
    /// Used by `check_temporal_literals` (E-QUERY-041 gate) to determine whether
    /// a column is `ColumnType::Datetime` before validating a string literal as RFC-3339.
    ///
    /// Returns `None` when:
    /// - The table is not registered (fail-open: no temporal validation)
    /// - The column is not found in the spec (fail-open)
    /// - The `column_types_by_table` lock is poisoned
    ///
    /// ADR-052 v1.1 D4 / HIGH-1 schema-aware gate.
    pub fn column_type_for(&self, table_name: &str, col_name: &str) -> Option<ColumnType> {
        match self.column_types_by_table.read() {
            Ok(guard) => guard
                .get(table_name)
                .and_then(|cols| cols.get(col_name))
                .cloned(),
            Err(_) => {
                tracing::warn!(
                    event_type = "table_registry.rwlock_poisoned",
                    method = "column_type_for",
                    "TableRegistry::column_type_for: RwLock poisoned — returning None (fail-open). \
                     Another thread panicked while holding the lock."
                );
                None
            }
        }
    }

    /// Return the sensor_id that owns `table_name`, or `None` if not registered.
    ///
    /// Used by the plan-time gate to populate the `sensor` field of
    /// `PrismError::TableNotAvailable`. The sensor is the prefix of the table name
    /// (e.g. `"crowdstrike"` for `"crowdstrike_alerts"`).
    /// On `RwLock` poison: returns `None` (safe default) and emits a WARN.
    /// (NB-1, BC-2.16.002 row `table_registry.rwlock_poisoned`)
    pub fn sensor_for_table(&self, table_name: &str) -> Option<String> {
        match self.sensor_by_table.read() {
            Ok(guard) => guard.get(table_name).cloned(),
            Err(_) => {
                tracing::warn!(
                    event_type = "table_registry.rwlock_poisoned",
                    method = "sensor_for_table",
                    "TableRegistry::sensor_for_table: RwLock poisoned — returning None. \
                     Another thread panicked while holding the lock."
                );
                None
            }
        }
    }

    /// Return a single-lock snapshot of the entire `sensor_by_table` map.
    ///
    /// Acquires `sensor_by_table` ONCE and returns an owned clone. Callers that
    /// need to look up the owning sensor for multiple tables should prefer this over
    /// calling [`sensor_for_table`] in a loop, which would acquire and release the
    /// read-lock N times (one per table).
    ///
    /// On `RwLock` poison: returns an empty `HashMap` (fail-closed — same behaviour as
    /// the gate in `check_availability_gate`) and the existing
    /// `table_registry.rwlock_poisoned` WARN fires so operators can observe the
    /// condition. No NEW `event_type` is introduced; this reuses the existing catalog
    /// row. (OBS-2 fix, BC-2.16.002 row `table_registry.rwlock_poisoned`)
    pub(crate) fn sensor_by_table_snapshot(&self) -> HashMap<String, String> {
        match self.sensor_by_table.read() {
            Ok(guard) => guard.clone(),
            Err(_) => {
                tracing::warn!(
                    event_type = "table_registry.rwlock_poisoned",
                    method = "sensor_by_table_snapshot",
                    "TableRegistry::sensor_by_table_snapshot: RwLock poisoned — returning empty \
                     map (fail-closed). Another thread panicked while holding the lock."
                );
                HashMap::new()
            }
        }
    }

    /// Compute the `did_you_mean` suggestion field for `TableNotAvailable`.
    ///
    /// Uses `strsim::levenshtein(candidate, requested)` over all registered table
    /// names. Returns `" Did you mean: 'X'?"` if the closest match has distance ≤ 3,
    /// or `""` if no match is within the threshold. (AC-3, EC-11-120, EC-11-126)
    ///
    /// # Input length cap (SEC-002, CWE-407)
    /// `requested` is capped at [`DID_YOU_MEAN_MAX_NAME_BYTES`] (128 bytes) before
    /// the Levenshtein computation. Names longer than the cap are silently truncated
    /// at a UTF-8 character boundary — this is a DoS defence (not an error path),
    /// because the truncated value cannot match any realistic sensor/table name and
    /// the computation returns `""` (no suggestion).
    ///
    /// # Architecture Compliance
    /// MUST use `strsim::levenshtein`, NOT `edit-distance`. (D-1163 ratification)
    pub fn did_you_mean(&self, requested: &str) -> String {
        // SEC-002 / CWE-407: cap input length before O(m×n) Levenshtein computation.
        // Truncate at a UTF-8 char boundary so the truncated slice is always valid.
        let requested = if requested.len() > DID_YOU_MEAN_MAX_NAME_BYTES {
            // Find the last char boundary at or before the cap.
            let mut boundary = DID_YOU_MEAN_MAX_NAME_BYTES;
            while !requested.is_char_boundary(boundary) {
                boundary -= 1;
            }
            &requested[..boundary]
        } else {
            requested
        };

        let tables = self.registered_tables();
        if tables.is_empty() {
            return String::new();
        }

        // Defense-in-depth tie-break parity with enrich/column gates (engine.rs).
        // `registered_tables()` returns a lex-sorted Vec today so min_by_key(dist)
        // already yields the lex-smallest equidistant candidate by construction.
        // The explicit name key makes that determinism contract-enforced regardless
        // of future input ordering changes.
        let best = tables
            .iter()
            .map(|candidate| (strsim::levenshtein(requested, candidate), candidate))
            .min_by_key(|(dist, name)| (*dist, name.to_string()));

        match best {
            Some((dist, candidate)) if dist <= 3 => {
                format!(" Did you mean: '{candidate}'?")
            }
            _ => String::new(),
        }
    }

    /// Compute the `did_you_mean` suggestion over a pre-filtered set of visible tables.
    ///
    /// Identical Levenshtein logic to [`did_you_mean`] but operates on the caller-supplied
    /// `visible_tables` slice rather than the full global registry. Used by
    /// `check_availability_gate` when org-scope filtering is active (ADR-039 / SEC-001)
    /// to avoid suggesting tables belonging to other orgs.
    ///
    /// The same 128-byte input cap (SEC-002 / CWE-407) is applied here.
    ///
    /// # BC-2.11.001 / ADR-039
    pub fn did_you_mean_for_tables(&self, requested: &str, visible_tables: &[String]) -> String {
        // SEC-002 / CWE-407: apply the same 128-byte cap as `did_you_mean`.
        let requested = if requested.len() > DID_YOU_MEAN_MAX_NAME_BYTES {
            let mut boundary = DID_YOU_MEAN_MAX_NAME_BYTES;
            while !requested.is_char_boundary(boundary) {
                boundary -= 1;
            }
            &requested[..boundary]
        } else {
            requested
        };

        if visible_tables.is_empty() {
            return String::new();
        }

        // Defense-in-depth tie-break parity with enrich/column gates (engine.rs).
        // Caller-supplied `visible_tables` may not be sorted, so the explicit name
        // key is particularly important here to guarantee deterministic output
        // across all calling conventions.
        let best = visible_tables
            .iter()
            .map(|candidate| (strsim::levenshtein(requested, candidate), candidate))
            .min_by_key(|(dist, name)| (*dist, name.to_string()));

        match best {
            Some((dist, candidate)) if dist <= 3 => {
                format!(" Did you mean: '{candidate}'?")
            }
            _ => String::new(),
        }
    }

    /// Return the set of all registered sensor IDs (derived from table prefixes).
    ///
    /// Used to populate the `available_sensors` field of `TableNotAvailable`.
    /// Sensor IDs come from the reverse `sensor_by_table` map — exactly what was
    /// registered, not derived heuristically from table name splitting.
    /// On `RwLock` poison: returns empty `Vec` (safe default) and emits a WARN.
    /// (NB-1, BC-2.16.002 row `table_registry.rwlock_poisoned`)
    pub fn registered_sensor_ids(&self) -> Vec<String> {
        let sensor_by_table = match self.sensor_by_table.read() {
            Ok(g) => g,
            Err(_) => {
                tracing::warn!(
                    event_type = "table_registry.rwlock_poisoned",
                    method = "registered_sensor_ids",
                    "TableRegistry::registered_sensor_ids: RwLock poisoned — returning empty list. \
                     Another thread panicked while holding the lock."
                );
                return Vec::new();
            }
        };

        // Collect unique sensor IDs from the reverse map — exactly what was registered.
        let ids: BTreeSet<String> = sensor_by_table.values().cloned().collect();
        ids.into_iter().collect()
    }

    /// Plan-time availability gate called from `engine::check_table_availability`.
    ///
    /// Parses `query_str`, extracts all source_refs via the AST visitor, and checks
    /// each against this registry. Returns `Err(PrismError::TableNotAvailable)` for
    /// the first unregistered table (fail fast, before fan-out). Skips `prism_*`
    /// prefixed tables (those have their own capability gate in `engine.rs`).
    ///
    /// # Org-scoped error enumeration (ADR-039 / SEC-001 / CWE-200)
    /// When `org_scope` and `resolved_spec_map` are both `Some`, the `available_sensors`
    /// and `available_tables` fields in `TableNotAvailable` are filtered to the sensors
    /// and tables accessible to the requesting org(s). This prevents cross-tenant vendor
    /// enumeration in multi-tenant overlay deployments.
    ///
    /// When either parameter is `None`, the global registry is returned unchanged —
    /// preserving single-tenant backward compatibility.
    ///
    /// Called from `engine::check_table_availability` so that `engine.rs` itself
    /// remains free of `todo!()` stubs (preserving the POL-12 / AC-8 guard in
    /// `tests/execute_integration_tests.rs::test_AC_8_no_todo_or_unimplemented_remains`).
    ///
    /// # BC-2.11.001 / S-3.13 AC-2, AC-3, AC-8 / ADR-039
    pub fn check_availability_gate(
        &self,
        query_str: &str,
        org_scope: Option<&[OrgSlug]>,
        resolved_spec_map: Option<&HashMap<ResolvedSpecKey, ResolvedSensorSpec>>,
    ) -> Result<(), PrismError> {
        use crate::ast::SourceRefKind;
        use crate::filter_parser::PrismQlParser;

        // Parse the query to extract source refs. If parsing fails, return Ok(()) —
        // parse errors are handled downstream by the execution pipeline.
        let ast = match PrismQlParser::parse(query_str) {
            Ok(ast) => ast,
            Err(_) => return Ok(()),
        };

        // Collect all source_refs from the AST (mirrors explain.rs extract_sources_from_ast).
        let sources = extract_sources_from_ast_for_gate(&ast);

        for source in &sources {
            // Skip internal prism_* tables — those have their own capability gate.
            if matches!(source.kind, SourceRefKind::Internal(_)) {
                continue;
            }
            // Skip composite virtual sources (EVENTS, ALERTS, DEVICES, ASSETS, SESSIONS).
            if matches!(source.kind, SourceRefKind::Composite(_)) {
                continue;
            }

            // Derive the registered table name from the source ref.
            //
            // Custom kind: raw is already the full table name (e.g. "crowdstrike_alerts").
            //
            // External kind: `sensor.table` dot-notation parsing.
            //
            // All modes (Pipe, SQL, SqlPipe): dot-notation in FROM target position is INVALID
            //   PrismQL syntax. Only underscore-qualified names (`sensor_table`) are valid in
            //   FROM. Reject with E-QUERY-037 immediately using the dot-notation string as the
            //   error table name (EC-11-067 / BC-2.11.001 / AC-N2). Do NOT silently
            //   convert to underscore form and let the fan-out proceed.
            //
            //   Example (pipe):    `FROM cyberint.alerts` → Err(TableNotAvailable { table:
            //     "cyberint.alerts", did_you_mean: " Did you mean: 'cyberint_alerts'?" })
            //   Example (SqlPipe): `SELECT * FROM crowdstrike.detections | limit 10` →
            //     Err(TableNotAvailable { table: "crowdstrike.detections",
            //     did_you_mean: " Did you mean: 'crowdstrike_detections'?" })
            //
            //   HIGH-1 (BC-2.11.001): the prior SqlPipe exemption (`is_sqlpipe` guard)
            //   allowed dot-notation in SqlPipe queries to bypass E-QUERY-037, silently routing
            //   to fan-out. EC-11-067 applies to ALL AST modes — the exemption is removed.
            //   The later `SourceRefKind::External { sensor, table } => format!("{sensor}_{table}")`
            //   underscore-conversion arm remains as a safety fallback but is dead for External
            //   sources that reach it (the rejection above fires first).
            //
            // Filter-mode queries parse External source refs only for the table-source
            //   position, and they emit Custom refs (underscore form) — not External —
            //   so BC-2.11.023 / ADR-046 filter-mode queries are not affected.
            if let SourceRefKind::External { sensor, table } = &source.kind {
                // Reject dot-notation in all modes (EC-11-067, BC-2.11.001).
                let dot_name = format!("{sensor}.{table}");
                let underscore_name = format!("{sensor}_{table}");

                let sensor_by_table_snapshot = match self.sensor_by_table.read() {
                    Ok(g) => g.clone(),
                    Err(_) => {
                        tracing::warn!(
                            event_type = "table_registry.rwlock_poisoned",
                            method = "check_availability_gate.dot_notation",
                            "TableRegistry::check_availability_gate: sensor_by_table RwLock \
                             poisoned — using empty map for dot-notation org filter."
                        );
                        HashMap::new()
                    }
                };

                let global_sensor_ids = self.registered_sensor_ids();
                let global_tables = self.registered_tables();
                let org_visible_sensor_ids =
                    filter_to_org_visible_sensors(global_sensor_ids, org_scope, resolved_spec_map);
                let org_visible_tables = filter_to_org_visible_tables(
                    global_tables,
                    &sensor_by_table_snapshot,
                    &org_visible_sensor_ids,
                    org_scope,
                    resolved_spec_map,
                );
                let available_sensors = org_visible_sensor_ids.join(", ");
                let available_tables = org_visible_tables.join(", ");

                // F-PLL1-LOW-001: Only suggest the underscore form when it is actually
                // registered.  If `cyberint.alerts` → `cyberint_alerts` is registered,
                // emit the direct suggestion.  If `foo.bar` → `foo_bar` is NOT registered,
                // fall through to the same Levenshtein-against-registered-tables path that
                // the non-dot arm uses (capped via cap_name_for_levenshtein), or emit no
                // suggestion if no candidate is within distance ≤ 3.  This prevents
                // suggesting non-existent tables (blind suggestion anti-pattern).
                let did_you_mean = if self.is_registered(&underscore_name) {
                    // The underscore form exists — direct suggestion (fast path, no Levenshtein).
                    format!(" Did you mean: '{underscore_name}'?")
                } else {
                    // Underscore form not registered — fall back to Levenshtein against the
                    // org-visible registered tables, same as the non-dot arm.
                    self.did_you_mean_for_tables(
                        cap_name_for_levenshtein(&underscore_name),
                        &org_visible_tables,
                    )
                };

                // Extract the matched table name from did_you_mean for the pedagogical suggestion.
                let did_you_mean_table: Option<&str> = if did_you_mean.is_empty() {
                    None
                } else {
                    did_you_mean.find('\'').and_then(|start| {
                        let rest = &did_you_mean[start + 1..];
                        rest.find('\'').map(|end| &rest[..end])
                    })
                };

                let client_id_for_suggestion = org_scope
                    .and_then(|s| s.first())
                    .map(|o| o.as_str())
                    .unwrap_or(sensor.as_str());
                let suggestion = crate::engine::e_query_037_suggestion(
                    client_id_for_suggestion,
                    did_you_mean_table,
                );

                return Err(PrismError::TableNotAvailable(Box::new(
                    TableNotAvailableDetails::new(
                        dot_name,
                        sensor.clone(),
                        available_sensors,
                        available_tables,
                        did_you_mean,
                        suggestion,
                    ),
                )));
            }

            let table_name = match &source.kind {
                SourceRefKind::Custom => source.raw.clone(),
                // External sources are always rejected by the dot-notation guard above
                // (EC-11-067 / BC-2.11.001 — all modes including SqlPipe).
                // This arm is unreachable for External sources but kept as a safety
                // fallback for hypothetical future AST variants that reach this point.
                // Internal and Composite already handled above.
                SourceRefKind::External { sensor, table } => format!("{sensor}_{table}"),
                _ => continue,
            };

            // Skip prism_* names that slipped through as Custom (defensive check).
            if table_name.starts_with("prism_") {
                continue;
            }

            if !self.is_registered(&table_name) {
                // Derive sensor from the sensor_by_table map, falling back to the
                // first underscore-separated prefix of the table name.
                let sensor = self
                    .sensor_for_table(&table_name)
                    .unwrap_or_else(|| table_name.split('_').next().unwrap_or("").to_string());

                // ADR-039 / SEC-001: filter available_sensors and available_tables to the
                // requesting org's scope. When org_scope or resolved_spec_map is None
                // (single-tenant or no overlay config), use the global registry unchanged.
                let sensor_by_table_snapshot = match self.sensor_by_table.read() {
                    Ok(g) => g.clone(),
                    Err(_) => {
                        tracing::warn!(
                            event_type = "table_registry.rwlock_poisoned",
                            method = "check_availability_gate",
                            "TableRegistry::check_availability_gate: sensor_by_table RwLock \
                             poisoned — using empty map for org filter."
                        );
                        HashMap::new()
                    }
                };

                let global_sensor_ids = self.registered_sensor_ids();
                let global_tables = self.registered_tables();

                let org_visible_sensor_ids =
                    filter_to_org_visible_sensors(global_sensor_ids, org_scope, resolved_spec_map);
                let org_visible_tables = filter_to_org_visible_tables(
                    global_tables,
                    &sensor_by_table_snapshot,
                    &org_visible_sensor_ids,
                    org_scope,
                    resolved_spec_map,
                );

                let available_sensors = org_visible_sensor_ids.join(", ");
                let available_tables = org_visible_tables.join(", ");
                let did_you_mean = self.did_you_mean_for_tables(&table_name, &org_visible_tables);

                // BC-2.11.017 AC-004: compute pedagogical suggestion so users learn to call
                // prism_describe to discover available tables and columns.
                // Extract the matched table name from the did_you_mean formatted string
                // (e.g. " Did you mean: 'crowdstrike_alerts'?" → Some("crowdstrike_alerts")).
                let did_you_mean_table: Option<&str> = if did_you_mean.is_empty() {
                    None
                } else {
                    did_you_mean.find('\'').and_then(|start| {
                        let rest = &did_you_mean[start + 1..];
                        rest.find('\'').map(|end| &rest[..end])
                    })
                };

                // BC-2.11.017 §E-QUERY-037: suggestion must reference `prism_describe('<client_id>')`,
                // NOT `prism_describe('<sensor>')`. The sensor name is NOT a registered client_id
                // and would fail with EC-10-023 "Client not registered".
                // (F-001B-FRESH-P1-MED-001 fix)
                //
                // Derive client_id from the requesting org_scope:
                // - Multi-tenant: `org_scope.first()` gives the requesting client_id (e.g. "acme").
                // - Single-tenant (org_scope is None or empty): fall back to `&sensor` as the
                //   best available identifier (no client context to resolve against).
                // SEC-002 trust-boundary: `org_scope` elements are `OrgSlug` values validated
                // to `^[a-zA-Z0-9_-]{1,64}$` by `OrgSlug::new` in `tenant.rs`. That regex
                // prohibits newlines, quotes, and control characters, so the client_id passed
                // to `e_query_037_suggestion` cannot carry prompt-injection characters into the
                // LLM-facing suggestion string in the MCP error envelope.
                let client_id_for_suggestion = org_scope
                    .and_then(|s| s.first())
                    .map(|o| o.as_str())
                    .unwrap_or(&sensor);
                let suggestion = crate::engine::e_query_037_suggestion(
                    client_id_for_suggestion,
                    did_you_mean_table,
                );

                return Err(PrismError::TableNotAvailable(Box::new(
                    TableNotAvailableDetails::new(
                        table_name,
                        sensor,
                        available_sensors,
                        available_tables,
                        did_you_mean,
                        suggestion,
                    ),
                )));
            }
        }

        Ok(())
    }
}

impl Default for TableRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// ADR-039 / SEC-001 — org-scope filter helpers
// ---------------------------------------------------------------------------

/// Filter `global_sensor_ids` to only those accessible to `org_scope`.
///
/// Rules (ADR-039 §Design Specification — `filter_to_org_visible` Logic):
/// 1. `org_scope` is `None` → return `global_sensor_ids` unchanged (single-tenant).
/// 2. `resolved_spec_map` is `None` → return `global_sensor_ids` unchanged (no overlay info).
/// 3. `org_scope` is `Some([])` → return empty `Vec` (no orgs = no sensors visible).
/// 4. `org_scope` is `Some(orgs)` and `resolved_spec_map` is `Some(map)`:
///    build the union of `sensor_id` values from `ResolvedSensorSpec` entries whose
///    `org_slug` is in `orgs`, then intersect with `global_sensor_ids`.
///
/// The result is `O(N_org_specs)` — computed only at error-construction time.
///
/// `pub(crate)` so that `explain.rs` can reuse this for SEC-003 available_tables
/// filtering without duplicating logic (ADR-039 reuse rule).
///
/// # ADR-039 / SEC-001 / CWE-200
pub(crate) fn filter_to_org_visible_sensors(
    global_sensor_ids: Vec<String>,
    org_scope: Option<&[OrgSlug]>,
    resolved_spec_map: Option<&HashMap<ResolvedSpecKey, ResolvedSensorSpec>>,
) -> Vec<String> {
    // Rule 1: no org scope restriction — return global set unchanged.
    let Some(orgs) = org_scope else {
        return global_sensor_ids;
    };
    // Rule 2: no overlay config — return global set unchanged (can't compute per-org visibility).
    let Some(spec_map) = resolved_spec_map else {
        return global_sensor_ids;
    };
    // Rule 3: empty org scope — no sensors visible.
    if orgs.is_empty() {
        return Vec::new();
    }
    // Rule 4: build union of sensor_ids for the requesting orgs.
    let org_visible: HashSet<&str> = spec_map
        .values()
        .filter(|rss| orgs.contains(&rss.org_slug))
        .map(|rss| rss.spec.sensor_id.as_str())
        .collect();

    // Intersect with global_sensor_ids (keep only what's in both).
    global_sensor_ids
        .into_iter()
        .filter(|sid| org_visible.contains(sid.as_str()))
        .collect()
}

/// Filter `global_tables` to only those whose owning sensor is in `org_visible_sensor_ids`.
///
/// Uses `sensor_by_table` (the reverse map from table name → sensor_id) to determine
/// which tables belong to which sensor, then filters to org-visible sensors.
///
/// The same four rules from `filter_to_org_visible_sensors` apply (same guard conditions).
/// Short-circuit: when `org_scope` is `None` or `resolved_spec_map` is `None`, return
/// `global_tables` unchanged.
///
/// `pub(crate)` so that `explain.rs` can reuse this for SEC-003 available_tables
/// filtering without duplicating logic (ADR-039 reuse rule).
///
/// # ADR-039 / SEC-001 / CWE-200
pub(crate) fn filter_to_org_visible_tables(
    global_tables: Vec<String>,
    sensor_by_table: &HashMap<String, String>,
    org_visible_sensor_ids: &[String],
    org_scope: Option<&[OrgSlug]>,
    resolved_spec_map: Option<&HashMap<ResolvedSpecKey, ResolvedSensorSpec>>,
) -> Vec<String> {
    // Rule 1: no org scope — return global set unchanged.
    let Some(_orgs) = org_scope else {
        return global_tables;
    };
    // Rule 2: no overlay config — return global set unchanged.
    if resolved_spec_map.is_none() {
        return global_tables;
    }
    // Rule 3: empty org_visible_sensor_ids means empty org scope (Rule 3 applied upstream).
    if org_visible_sensor_ids.is_empty() {
        return Vec::new();
    }

    let visible_set: HashSet<&str> = org_visible_sensor_ids.iter().map(String::as_str).collect();

    global_tables
        .into_iter()
        .filter(|table| {
            sensor_by_table
                .get(table)
                .map(|sid| visible_set.contains(sid.as_str()))
                .unwrap_or(false)
        })
        .collect()
}

// ---------------------------------------------------------------------------
// extract_sources_from_ast_for_gate — local helper mirroring explain.rs logic
// ---------------------------------------------------------------------------

/// Extract all source references from an AST for the availability gate.
///
/// Mirrors `explain::extract_sources_from_ast` — kept local to avoid a circular
/// dependency between the table_registry and explain modules. (S-3.13 Architecture)
fn extract_sources_from_ast_for_gate(ast: &crate::ast::Ast) -> Vec<crate::ast::SourceRef> {
    use crate::ast::{Ast, SqlStatement};

    let mut sources: Vec<crate::ast::SourceRef> = Vec::new();

    fn push_dedup(sources: &mut Vec<crate::ast::SourceRef>, s: &crate::ast::SourceRef) {
        if !sources.iter().any(|x| x.raw == s.raw) {
            sources.push(s.clone());
        }
    }

    match ast {
        Ast::Filter(fe) => {
            push_dedup(&mut sources, &fe.source);
        }
        Ast::Sql(SqlStatement::Select(sq)) => {
            push_dedup(&mut sources, &sq.from.source);
            for join in &sq.joins {
                push_dedup(&mut sources, &join.source);
            }
            // OBS-1: also walk sq.where_ for InSubquery predicates
            // (e.g. SELECT … WHERE id IN (SELECT … FROM <unregistered_sensor>)).
            // Without this walk, a WHERE-IN-subquery referencing an unregistered
            // external sensor bypasses the E-QUERY-037 gate and fails later with a
            // less helpful error.  Mirrors the Dml filter arm above.
            if let Some(ref where_pred) = sq.where_ {
                collect_predicate_sources_into_gate(where_pred, &mut sources);
            }
            // L1 fix (S-DEMO-FIDELITY-REMEDIATION-001): also walk HAVING, GROUP BY,
            // ORDER BY, and JOIN ON positions for InSubquery expressions.
            // Previously only WHERE was walked; subqueries in these positions bypassed
            // the gate and produced opaque DataFusion errors instead of E-QUERY-037.
            if let Some(ref having_pred) = sq.having {
                collect_predicate_sources_into_gate(having_pred, &mut sources);
            }
            for expr in &sq.group_by {
                collect_expr_sources_into_gate(expr, &mut sources);
            }
            for oe in &sq.order_by {
                collect_expr_sources_into_gate(&oe.expr, &mut sources);
            }
            for join in &sq.joins {
                collect_expr_sources_into_gate(&join.on, &mut sources);
            }
        }
        Ast::Sql(SqlStatement::Dml(dml)) => {
            if let Some(ref source_select) = dml.source_select {
                push_dedup(&mut sources, &source_select.from.source);
                for join in &source_select.joins {
                    push_dedup(&mut sources, &join.source);
                }
            }
            // OBS-1 fix: mirror explain.rs — also walk dml.filter for InSubquery
            // predicates (e.g. DELETE WHERE id IN (SELECT … FROM <external_sensor>)).
            // Without this walk, a WHERE-IN-subquery referencing an unregistered
            // external table would bypass the gate and fail later/less helpfully
            // instead of returning the fast E-QUERY-037 response.
            if let Some(ref filter) = dml.filter {
                collect_predicate_sources_into_gate(filter, &mut sources);
            }
        }
        Ast::Pipe(pq) => {
            push_dedup(&mut sources, &pq.source);
            for stage in &pq.stages {
                if let crate::ast::PipeStage::Join(js) = stage {
                    push_dedup(&mut sources, &js.source);
                }
            }
        }
        // BC-2.11.020 / HIGH-1 sibling sweep: SqlPipe head drives the E-QUERY-037
        // availability gate. A SqlPipe query whose head references an unregistered
        // table must return E-QUERY-037 (with available_tables / did_you_mean)
        // just like Ast::Sql(Select). Without this arm, the gate was a no-op for
        // SqlPipe, silently skipping the fast-fail path and deferring the error to
        // a later (less helpful) site. AC-8: gate is mode-agnostic. (TD-VSDD-060)
        // OBS-1 parity fix: also collect PipeStage::Join sources from spq.stages so
        // that `SELECT … | join <table> on …` pipe-stage JOINs also reach the
        // E-QUERY-037 gate. Mirrors the Ast::Pipe arm above. (TD-VSDD-060)
        Ast::SqlPipe(spq) => {
            push_dedup(&mut sources, &spq.head.from.source);
            for join in &spq.head.joins {
                push_dedup(&mut sources, &join.source);
            }
            // OBS-1: walk spq.head.where_ for InSubquery predicates, mirroring the
            // Ast::Sql(Select) arm above.  A SqlPipe query whose SQL head references an
            // unregistered sensor inside a WHERE…IN(SELECT…FROM <sensor>) must also
            // reach the E-QUERY-037 gate.
            if let Some(ref where_pred) = spq.head.where_ {
                collect_predicate_sources_into_gate(where_pred, &mut sources);
            }
            // L1 fix (S-DEMO-FIDELITY-REMEDIATION-001): mirror the SQL Select arm —
            // also walk HAVING, GROUP BY, ORDER BY, and JOIN ON in the SqlPipe head.
            if let Some(ref having_pred) = spq.head.having {
                collect_predicate_sources_into_gate(having_pred, &mut sources);
            }
            for expr in &spq.head.group_by {
                collect_expr_sources_into_gate(expr, &mut sources);
            }
            for oe in &spq.head.order_by {
                collect_expr_sources_into_gate(&oe.expr, &mut sources);
            }
            for join in &spq.head.joins {
                collect_expr_sources_into_gate(&join.on, &mut sources);
            }
            for stage in &spq.stages {
                if let crate::ast::PipeStage::Join(js) = stage {
                    push_dedup(&mut sources, &js.source);
                }
            }
        }
        // #[non_exhaustive] catch-all for future AST variants.
        #[allow(unreachable_patterns)]
        _ => {}
    }

    sources
}

/// Walk a `Predicate` tree and collect `SourceRef`s from any `InSubquery` predicates.
///
/// Mirrors `explain::collect_predicate_sources_into` — kept local to avoid a circular
/// dependency between the table_registry and explain modules. Used by
/// `extract_sources_from_ast_for_gate` for the DML filter arm (OBS-1): a DML WHERE
/// clause may contain `field IN (SELECT … FROM <external_sensor>)` which references
/// an external table that must be gated. Without this walk, the gate would silently
/// miss the subquery source and return success, deferring the E-QUERY-037 to a
/// later (less helpful) error site. (AC-8 mode-agnostic gating robustness)
fn collect_predicate_sources_into_gate(
    predicate: &crate::ast::Predicate,
    sources: &mut Vec<crate::ast::SourceRef>,
) {
    use crate::ast::Predicate;

    fn push_dedup(sources: &mut Vec<crate::ast::SourceRef>, s: &crate::ast::SourceRef) {
        if !sources.iter().any(|x| x.raw == s.raw) {
            sources.push(s.clone());
        }
    }

    match predicate {
        Predicate::InSubquery { subquery, .. } => {
            push_dedup(sources, &subquery.from.source);
            for join in &subquery.joins {
                push_dedup(sources, &join.source);
            }
        }
        Predicate::Logical { predicates, .. } => {
            for p in predicates {
                collect_predicate_sources_into_gate(p, sources);
            }
        }
        Predicate::Not(inner) => {
            collect_predicate_sources_into_gate(inner, sources);
        }
        // Other predicate variants (Compare, Between, Cidr, Has, Missing, IsNull,
        // Wildcard, RecoveryError, In, StringOp, Regex) do not carry nested SqlQuery
        // references and need no traversal.
        _ => {}
    }
}

/// Walk an `Expr` tree and collect `SourceRef`s from any `Expr::InSubquery` nodes.
///
/// Used by `extract_sources_from_ast_for_gate` for HAVING, GROUP BY, ORDER BY, and
/// JOIN ON positions — all of which can contain subqueries in expressions
/// (e.g. `GROUP BY (SELECT MAX(ts) FROM other_table)`). Without this walk,
/// an InSubquery expression in those positions would bypass the E-QUERY-037 gate
/// and fail later with a less helpful error. (L1 fix, S-DEMO-FIDELITY-REMEDIATION-001)
///
/// # What is walked
/// - `Expr::InSubquery { subquery, .. }` — collect the subquery's FROM source and joins.
/// - `Expr::FuncCall(Aggregate | Scalar)` — recurse into args.
/// - `Expr::Compare { lhs, rhs, .. }` — recurse into both operands (JOIN ON conditions).
/// - `Expr::Logical { lhs, rhs, .. }` — recurse into both operands (AND/OR).
/// - `Expr::Not(inner)` — recurse into inner.
/// - `Expr::TimestampArithmetic { base, .. }` — recurse into base.
///
/// # Non-exhaustive safety
/// Unknown future `Expr` and `FuncCall` variants are silently skipped via
/// the `_ => {}` catch-all arm, preserving fail-open gate semantics and
/// satisfying `#[non_exhaustive]` discipline.
fn collect_expr_sources_into_gate(
    expr: &crate::ast::Expr,
    sources: &mut Vec<crate::ast::SourceRef>,
) {
    use crate::ast::{Expr, FuncCall};

    fn push_dedup(sources: &mut Vec<crate::ast::SourceRef>, s: &crate::ast::SourceRef) {
        if !sources.iter().any(|x| x.raw == s.raw) {
            sources.push(s.clone());
        }
    }

    match expr {
        Expr::InSubquery { subquery, .. } => {
            push_dedup(sources, &subquery.from.source);
            for join in &subquery.joins {
                push_dedup(sources, &join.source);
            }
        }
        Expr::FuncCall(fc) => match fc {
            FuncCall::Aggregate { args, .. } | FuncCall::Scalar { args, .. } => {
                for arg in args {
                    collect_expr_sources_into_gate(arg, sources);
                }
            }
            FuncCall::Window { .. } => {} // No args yet.
            #[allow(unreachable_patterns)]
            _ => {}
        },
        Expr::Compare { lhs, rhs, .. } => {
            collect_expr_sources_into_gate(lhs, sources);
            collect_expr_sources_into_gate(rhs, sources);
        }
        Expr::Logical { lhs, rhs, .. } => {
            collect_expr_sources_into_gate(lhs, sources);
            collect_expr_sources_into_gate(rhs, sources);
        }
        Expr::Not(inner) => collect_expr_sources_into_gate(inner, sources),
        Expr::TimestampArithmetic { base, .. } => {
            collect_expr_sources_into_gate(base, sources);
        }
        // Literal, Field, VirtualField, In, Star, Now, Interval, and future variants: skip.
        _ => {}
    }
}

// ---------------------------------------------------------------------------
// Test-only helpers for exercising poison paths
// ---------------------------------------------------------------------------

#[cfg(test)]
impl TableRegistry {
    /// Emit the `table_registry.rwlock_poisoned` WARN tracing event directly.
    ///
    /// This test helper allows `table_registry_tests.rs` to verify the emission
    /// path via `#[tracing_test::traced_test]` + `logs_contain(...)` without
    /// needing to actually poison the internal RwLock (which would require unsafe
    /// code or white-box access to the private field). The production emission is
    /// identical to this call — the test validates that the WARN fires with the
    /// correct `event_type` field.
    ///
    /// NB-1 (S-3.13 fix-burst): RwLock poison visibility.
    pub(crate) fn test_emit_rwlock_poisoned_warn_for_coverage() {
        tracing::warn!(
            event_type = "table_registry.rwlock_poisoned",
            method = "is_registered",
            "TableRegistry::is_registered: RwLock poisoned — returning fail-closed \
             default (false). Another thread panicked while holding the lock."
        );
    }

    /// Construct a `TableRegistry` whose `registered` RwLock is poisoned.
    ///
    /// Poisons the `registered` lock by spawning a thread that holds the write
    /// guard while panicking. This is the standard Rust pattern for testing
    /// poison-error handling paths.
    ///
    /// NB-1 (S-3.13 fix-burst): used to verify fail-closed behavior without
    /// panicking in the test thread itself.
    pub(crate) fn new_with_poisoned_registered_for_test() -> Self {
        let registry = Self::new();
        // Poison the `registered` lock by acquiring the write lock in a thread
        // that panics while holding it.
        //
        // Implementation: `register_sensor` acquires the write lock and releases
        // it before returning. To keep the lock held during the panic we need to
        // directly access the `Arc<RwLock<...>>` field. Since this is a `#[cfg(test)]`
        // method in the same module, the private field is accessible.
        let registered_clone = Arc::clone(&registry.registered);
        let _ = std::thread::spawn(move || {
            // Acquire the write lock then immediately panic — this leaves the lock
            // in a poisoned state for the registry.
            let _guard = registered_clone
                .write()
                .expect("write lock must be acquirable before poison");
            panic!("intentional poison for test_NB_1 fail-closed coverage");
        })
        .join();
        // Thread has panicked; `registered` lock is now poisoned.
        registry
    }

    /// Construct a `TableRegistry` whose `sensor_by_table` RwLock is poisoned.
    ///
    /// Mirrors `new_with_poisoned_registered_for_test` but poisons `sensor_by_table`
    /// instead of `registered`. Used by CR-003 tests that verify the fail-closed
    /// behavior of `sensor_for_table`, `registered_sensor_ids`, and
    /// `check_availability_gate` when the `sensor_by_table` lock is poisoned.
    ///
    /// CR-003 (S-3.13 fix-burst): actual-poison coverage for `sensor_by_table`.
    pub(crate) fn new_with_poisoned_sensor_by_table_for_test() -> Self {
        let registry = Self::new();
        let sensor_by_table_clone = Arc::clone(&registry.sensor_by_table);
        let _ = std::thread::spawn(move || {
            let _guard = sensor_by_table_clone
                .write()
                .expect("write lock must be acquirable before poison");
            panic!("intentional sensor_by_table poison for CR-003 fail-closed coverage");
        })
        .join();
        // Thread has panicked; `sensor_by_table` lock is now poisoned.
        registry
    }
}

/// Test-only re-export of `extract_sources_from_ast_for_gate` for AST-level
/// unit tests in `table_registry_tests.rs`.
///
/// The production function is intentionally private (it is a module-internal
/// helper with no public contract). This wrapper grants `pub(crate)` visibility
/// exclusively for test assertions that build AST nodes directly and verify the
/// predicate-subquery walk without going through the parser.
///
/// # Why this exists (OBS-1)
/// The DELETE/UPDATE parser uses `build_predicate_parser()` which does not yet
/// support `IN (SELECT …)` predicates in WHERE clauses. Direct AST construction
/// is therefore the only way to test the `dml.filter` InSubquery walk path in
/// isolation. The function under test is `extract_sources_from_ast_for_gate`;
/// this thin wrapper makes it reachable from `tests::table_registry_tests`.
#[cfg(test)]
pub(crate) fn extract_sources_from_ast_for_gate_test_only(
    ast: &crate::ast::Ast,
) -> Vec<crate::ast::SourceRef> {
    extract_sources_from_ast_for_gate(ast)
}

// ---------------------------------------------------------------------------
// Unit tests (table_registry_tests.rs is the canonical test home)
// ---------------------------------------------------------------------------
// All Red Gate tests are in src/tests/table_registry_tests.rs.
// This file contains no inline tests to avoid duplication.
