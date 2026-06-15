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

use prism_core::{error::TableNotAvailableDetails, PrismError};
use prism_spec_engine::{ConfigSnapshot, SensorSpec};

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
pub struct TableRegistry {
    /// Table names currently available (e.g. `"crowdstrike_alerts"`).
    registered: Arc<RwLock<HashSet<String>>>,
    /// Table name → sensor_id reverse mapping for error messages.
    sensor_by_table: Arc<RwLock<HashMap<String, String>>>,
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
        }

        // Insert new tables for this sensor (the register phase).
        // Executed under the same held locks — atomic with the remove above.
        for table in &spec.tables {
            let full_name = format!("{}_{}", spec.sensor_id, table.table_name);
            registered.insert(full_name.clone());
            sensor_by_table.insert(full_name, spec.sensor_id.clone());
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

        // Collect keys to remove (avoid modifying while iterating).
        let to_remove: Vec<String> = registered
            .iter()
            .filter(|name| name.starts_with(&prefix))
            .cloned()
            .collect();

        for name in to_remove {
            registered.remove(&name);
            sensor_by_table.remove(&name);
        }

        Ok(())
    }

    /// Return `true` if `table_name` is in the current registry.
    ///
    /// Uses `RwLock::read()` — non-exclusive. MUST NOT block query execution.
    /// On `RwLock` poison: returns `false` (conservative safe default — no access).
    /// (Architecture Compliance Rules, S-3.13)
    ///
    /// # BC-2.16.001 / BC-2.11.001
    /// Plan-time check in `engine.rs`; returns `false` for unregistered sensors.
    pub fn is_registered(&self, table_name: &str) -> bool {
        match self.registered.read() {
            Ok(guard) => guard.contains(table_name),
            Err(_) => false, // poisoned lock — conservative safe default
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
    /// On `RwLock` poison: returns empty `Vec` (safe default).
    pub fn registered_tables(&self) -> Vec<String> {
        let guard = match self.registered.read() {
            Ok(g) => g,
            Err(_) => return Vec::new(), // poisoned lock — return empty list
        };
        let mut tables: Vec<String> = guard.iter().cloned().collect();
        tables.sort();
        tables
    }

    /// Return the sensor_id that owns `table_name`, or `None` if not registered.
    ///
    /// Used by the plan-time gate to populate the `sensor` field of
    /// `PrismError::TableNotAvailable`. The sensor is the prefix of the table name
    /// (e.g. `"crowdstrike"` for `"crowdstrike_alerts"`).
    /// On `RwLock` poison: returns `None` (safe default).
    pub fn sensor_for_table(&self, table_name: &str) -> Option<String> {
        match self.sensor_by_table.read() {
            Ok(guard) => guard.get(table_name).cloned(),
            Err(_) => None, // poisoned lock — return None
        }
    }

    /// Compute the `did_you_mean` suggestion field for `TableNotAvailable`.
    ///
    /// Uses `strsim::levenshtein(candidate, requested)` over all registered table
    /// names. Returns `" Did you mean: 'X'?"` if the closest match has distance ≤ 3,
    /// or `""` if no match is within the threshold. (AC-3, EC-11-120, EC-11-126)
    ///
    /// # Architecture Compliance
    /// MUST use `strsim::levenshtein`, NOT `edit-distance`. (D-1163 ratification)
    pub fn did_you_mean(&self, requested: &str) -> String {
        let tables = self.registered_tables();
        if tables.is_empty() {
            return String::new();
        }

        let best = tables
            .iter()
            .map(|candidate| (strsim::levenshtein(requested, candidate), candidate))
            .min_by_key(|(dist, _)| *dist);

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
    /// On `RwLock` poison: returns empty `Vec` (safe default).
    pub fn registered_sensor_ids(&self) -> Vec<String> {
        let sensor_by_table = match self.sensor_by_table.read() {
            Ok(g) => g,
            Err(_) => return Vec::new(), // poisoned lock — safe default
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
    /// Called from `engine::check_table_availability` so that `engine.rs` itself
    /// remains free of `todo!()` stubs (preserving the POL-12 / AC-8 guard in
    /// `tests/execute_integration_tests.rs::test_AC_8_no_todo_or_unimplemented_remains`).
    ///
    /// # BC-2.11.001 / S-3.13 AC-2, AC-3, AC-8
    pub fn check_availability_gate(&self, query_str: &str) -> Result<(), PrismError> {
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
            // Custom kind: raw is already the full table name (e.g. "crowdstrike_alerts").
            // External kind: sensor.table dotted → "sensor_table" underscore convention.
            let table_name = match &source.kind {
                SourceRefKind::Custom => source.raw.clone(),
                SourceRefKind::External { sensor, table } => {
                    format!("{sensor}_{table}")
                }
                // Internal and Composite already handled above.
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

                let available_sensors = self.registered_sensor_ids().join(", ");
                let available_tables = self.registered_tables().join(", ");
                let did_you_mean = self.did_you_mean(&table_name);

                return Err(PrismError::TableNotAvailable(Box::new(
                    TableNotAvailableDetails::new(
                        table_name,
                        sensor,
                        available_sensors,
                        available_tables,
                        did_you_mean,
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
        }
        Ast::Sql(SqlStatement::Dml(dml)) => {
            if let Some(ref source_select) = dml.source_select {
                push_dedup(&mut sources, &source_select.from.source);
                for join in &source_select.joins {
                    push_dedup(&mut sources, &join.source);
                }
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
        // #[non_exhaustive] catch-all for future AST variants.
        #[allow(unreachable_patterns)]
        _ => {}
    }

    sources
}

// ---------------------------------------------------------------------------
// Unit tests (table_registry_tests.rs is the canonical test home)
// ---------------------------------------------------------------------------
// All Red Gate tests are in src/tests/table_registry_tests.rs.
// This file contains no inline tests to avoid duplication.
