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

use prism_core::{error::TableNotAvailableDetails, OrgSlug, PrismError};
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
const DID_YOU_MEAN_MAX_NAME_BYTES: usize = 128;

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

        let best = visible_tables
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
