//! `prism_describe` tool handler — L2 schema discovery (BC-2.10.012).
//!
//! Exposes per-client table and column catalog to AI agents so they can discover
//! which tables and columns are available before authoring PrismQL queries.
//!
//! # Data source
//!
//! Column schema is read from `query_engine.resolved_spec_map()` in multi-tenant
//! mode (filter by `OrgSlug`) or from `config_manager` in single-tenant/test
//! fallback mode (same pattern as `render_schema_resource` and
//! `render_client_sensors_resource` in `resources.rs`). `TableRegistry` holds
//! only table-name strings and is NOT the column-data source here (ADR-022 §C;
//! story v1.2 architecture compliance rules).
//!
//! # Response types
//!
//! Three `#[non_exhaustive]` public types (`PrismDescribeResponse`,
//! `TableDescriptor`, `ColumnDescriptor`) carry the schema catalog. The
//! `ColumnDescriptor.col_type` field uses `prism_core::column::ColumnType` — the
//! canonical sensor-schema enum (variants: String/Integer/Float/Boolean/Datetime/Json,
//! ADR-024). Do NOT use `prism_core::types::ColumnType` (internal table schemas).
//!
//! # Audit event
//!
//! Every `prism_describe` call emits a structured audit event (BC-2.10.012 §Audit).
//! If audit emission fails the call proceeds (fail-open, DI-004) and the response
//! includes `_meta.audit_warning: true`.
//!
//! # Wiring (CLAUDE.md §Conventions — Arc-DI plumbing)
//!
//! `prism_describe` is registered as an always-on tool (BC-2.10.012 precondition 1
//! — must never be feature-gated). It is wired into the `#[tool_router]` block in
//! `server.rs`. No `Arc<dyn TableRegistry>` injection — see architecture compliance.

use rmcp::model::CallToolResult;
use serde::{Deserialize, Serialize};

// ─── Response types (BC-2.10.012) ────────────────────────────────────────────

/// Top-level response from the `prism_describe` tool.
///
/// `#[non_exhaustive]` required: public prism-mcp API type (CLAUDE.md §Conventions).
/// Gate: ci.yml EXPECTED=82 (this story adds 3 new types; v80 in struct_violations.rs).
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrismDescribeResponse {
    /// The client identifier this catalog is scoped to (DI-008).
    pub client_id: String,
    /// Table descriptors — one entry per table visible to this client.
    ///
    /// Empty vec when the client has zero provisioned sensors or is unknown.
    pub tables: Vec<TableDescriptor>,
    /// Schema-agnostic query hints for this client (e.g., guidance for empty/unknown
    /// clients, discovery pointers, and example query patterns).
    pub pql_hints: Vec<String>,
}

/// Describes a single sensor table available to the client.
///
/// `#[non_exhaustive]` required: public prism-mcp API type (CLAUDE.md §Conventions).
/// Gate: ci.yml EXPECTED=82 (v81 in struct_violations.rs).
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableDescriptor {
    /// The qualified table name (e.g., `"crowdstrike_detections"`).
    pub name: String,
    /// Sensor type identifier (e.g., `"crowdstrike"`, `"claroty"`).
    pub sensor_type: String,
    /// Human-readable description of the table (empty string if absent from spec).
    pub description: String,
    /// Column descriptors — one entry per column declared in the sensor spec.
    ///
    /// Empty vec when the table has zero declared columns (EC-002 / BC-2.10.012
    /// EC-10-025: empty columns array is valid; `example_query` uses count-recent
    /// fallback in this case).
    pub columns: Vec<ColumnDescriptor>,
    /// Auto-generated example PQL query for this table.
    ///
    /// Uses the real table name. Count-recent fallback always provided; severity-filter
    /// variant included when a `severity` column is present; aggregate variant when an
    /// aggregatable column is present (BC-2.10.012 §Auto-generated example queries).
    pub example_query: String,
}

/// Describes a single column within a `TableDescriptor`.
///
/// `#[non_exhaustive]` required: public prism-mcp API type (CLAUDE.md §Conventions).
/// Gate: ci.yml EXPECTED=82 (v82 in struct_violations.rs).
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnDescriptor {
    /// Column name as declared in the sensor spec.
    pub name: String,
    /// Canonical sensor-schema column type (ADR-024, CLAUDE.md §Conventions).
    ///
    /// Uses `prism_core::column::ColumnType` (variants: String/Integer/Float/Boolean/
    /// Datetime/Json). Do NOT use `prism_core::types::ColumnType` (internal table schemas).
    pub col_type: prism_core::column::ColumnType,
    /// Optional human-readable column description (from spec, may be None/absent).
    pub description: Option<String>,
    /// Whether the column can contain null values.
    pub nullable: bool,
}

// ─── Tool parameters (BC-2.10.012) ───────────────────────────────────────────

/// Parameters for the `prism_describe` tool.
#[derive(Debug, Clone, serde::Deserialize, schemars::JsonSchema)]
pub struct PrismDescribeParams {
    /// Client identifier to describe the schema for.
    ///
    /// Validated via `OrgSlug::new()` — rejects path-traversal chars and invalid
    /// characters. Returns `E-MCP-001` on format failure (BC-2.10.012).
    pub client_id: String,
}

// ─── Handler (BC-2.10.012) ────────────────────────────────────────────────────

/// Execute the `prism_describe` tool for the given client.
///
/// Reads column schema from `resolved_spec_map` (multi-tenant) or `config_manager`
/// (single-tenant fallback). Emits a structured audit event on every call. On audit
/// emission failure the call proceeds with `_meta.audit_warning: true` in the response
/// (DI-004 fail-open).
///
/// Returns a `PrismDescribeResponse` with all tables and columns visible to
/// `client_id`, or an empty `tables` array for unknown/empty clients (not an error).
///
/// Validation failure for `client_id` format returns `E-MCP-001`.
///
/// BC-2.10.012: BC-DEMO-PRISMQL-ONBOARDING-001-A AC-001 through AC-005.
pub async fn handle_prism_describe(
    client_id: String,
    query_engine: Option<&std::sync::Arc<prism_query::engine::QueryEngine>>,
    config_manager: Option<
        &std::sync::Arc<arc_swap::ArcSwap<prism_spec_engine::config_manager::ConfigManager>>,
    >,
    audit_writer: Option<&std::sync::Arc<dyn prism_query::write_dispatch::AuditWriter>>,
) -> Result<CallToolResult, rmcp::model::ErrorData> {
    // BC-2.10.012: validate client_id format via OrgSlug::new() (rejects path traversal
    // and injection payloads). DI-006: do NOT echo the raw payload in the error message.
    // SAP-1: emit schema_enumeration.started AFTER validation succeeds, not before.
    let org_slug = prism_core::OrgSlug::new(&client_id);
    if org_slug.is_err() {
        // DI-006: do NOT echo the raw client_id in the error message.
        tracing::warn!(
            event_type = "schema_enumeration.rejected",
            "prism_describe: invalid client_id format (E-MCP-001)"
        );
        // BC-2.10.012 v1.1 §Audit: emit audit BEFORE returning error (fail-open DI-004).
        // Even on validation failure, the audit trail must record the attempt with outcome="error".
        if let Some(aw) = audit_writer {
            if let Err(e) = aw
                .write_tool_call(
                    "prism_describe",
                    None, // client_id unavailable — validation failed
                    "schema_enumeration",
                    "error",
                )
                .await
            {
                tracing::warn!(
                    error = %e,
                    "prism_describe: audit emission failed on validation error path (fail-open, DI-004)"
                );
            }
        }
        return Err(rmcp::model::ErrorData::invalid_params(
            "E-MCP-001: invalid client_id format — must match [a-zA-Z0-9_-]{1,64}",
            None,
        ));
    }

    // BC-2.10.012 §Audit (SAP-1): emit started AFTER validation, so rejected calls
    // emit only schema_enumeration.rejected, not .started (BC-2.16.002 v1.85 catalog).
    tracing::info!(
        client_id = %org_slug,
        event_type = "schema_enumeration.started",
        "prism_describe: schema enumeration started"
    );

    // Emit audit event (fail-open per DI-004).
    // BC-2.10.012 §Audit: operation must be "schema_enumeration" (canonical operation name);
    // outcome must be "success" or "error" per emit_tool_audit convention.
    let mut audit_warning = false;
    if let Some(aw) = audit_writer {
        if let Err(e) = aw
            .write_tool_call(
                "prism_describe",
                Some(org_slug.as_str()),
                "schema_enumeration",
                "success",
            )
            .await
        {
            tracing::warn!(
                client_id = %org_slug,
                error = %e,
                "prism_describe: audit emission failed (fail-open, DI-004)"
            );
            audit_warning = true;
        }
    }

    // Build table descriptors: multi-tenant path (resolved_spec_map via query_engine)
    // takes precedence when query_engine is wired; falls back to config_manager for
    // single-tenant / test scenarios (same pattern as render_client_sensors_resource).
    let tables = build_tables_for_client(org_slug.as_str(), query_engine, config_manager);

    // BC-2.10.012 §Non-existent client_id handling: when tables are empty,
    // consult org_registry (from query_engine) to distinguish two cases:
    //   - registered-but-empty: org is in OrgRegistry, no sensor overlays
    //   - not-registered: org is absent from OrgRegistry entirely
    let org_registry: Option<std::sync::Arc<prism_core::OrgRegistry>> =
        query_engine.and_then(|qe| qe.org_registry());

    // BC-2.10.012 v1.7 §pql_hints AC-CAT2: resolve infusion_registry for Category-2 hint.
    // `query_engine.infusion_registry()` returns `Option<Arc<InfusionRegistry>>`.
    // Pass `as_deref()` to get `Option<&InfusionRegistry>` for the pure hint builder.
    let infusion_registry: Option<std::sync::Arc<prism_spec_engine::InfusionRegistry>> =
        query_engine.and_then(|qe| qe.infusion_registry());
    let pql_hints = build_pql_hints(
        org_slug.as_str(),
        &tables,
        org_registry.as_deref(),
        infusion_registry.as_deref(),
    );

    let response = PrismDescribeResponse {
        client_id: org_slug.as_str().to_string(),
        tables,
        pql_hints,
    };

    tracing::info!(
        client_id = %org_slug,
        event_type = "schema_enumeration.success",
        "prism_describe: schema enumeration succeeded"
    );

    // BC-2.10.012 §Response envelope: wrap via SafetyEnvelopeBuilder::wrap.
    //
    // The schema catalog is Prism-generated data (not sensor-sourced external content),
    // so trust_level = "internal" per BC-2.09.005.  `SafetyEnvelopeBuilder::wrap` places the
    // `PrismDescribeResponse` under the top-level `results` field of `ResponseEnvelope` AND
    // populates the full `_meta` (tool, data_source, query_time, trust_level=Internal,
    // safety_flags:[], total_results, page, has_more, next_cursor, audit_warning).
    //
    // BC-2.09.008: `_meta.safety_flags` is ALWAYS present as a Vec (never None/absent).
    // The schema catalog payload contains no sensor-sourced rows to scan for injection,
    // so safety_flags will be an empty array — correct (not a scan gap).
    //
    // AC-005 parity: `render_pql_schema_resource` delegates to this function, so both the
    // tool path and the resource path produce semantically identical envelopes.
    // `_meta.query_time` is non-deterministic per call (Utc::now()); the parity test
    // compares semantic content (results.client_id, results.tables, results.pql_hints)
    // rather than byte-equality.
    let results_value = serde_json::to_value(&response).map_err(|e| {
        rmcp::model::ErrorData::internal_error(
            format!("E-MCP-500: failed to serialize prism_describe response: {e}"),
            None,
        )
    })?;

    let audit_warning_opt = if audit_warning {
        Some(crate::safety_envelope::AUDIT_EMISSION_FAILED_WARNING.to_string())
    } else {
        None
    };

    let envelope = crate::safety_envelope::SafetyEnvelopeBuilder::wrap(
        "prism_describe",
        crate::safety_envelope::DataSource::Single(org_slug.as_str().to_owned()),
        results_value,
        1,
        false,
        None,
        audit_warning_opt,
    );

    serde_json::to_value(&envelope)
        .map(rmcp::model::CallToolResult::structured)
        .map_err(|e| {
            rmcp::model::ErrorData::internal_error(
                format!("E-MCP-500: failed to serialize prism_describe envelope: {e}"),
                None,
            )
        })
}

/// Build table descriptors for a client.
///
/// Multi-tenant path: when `query_engine` is wired and its `resolved_spec_map` is
/// `Some`, filter the map by `OrgSlug == client_id` and walk `ResolvedSensorSpec.spec`
/// tables. DI-008: an org sees ONLY its own resolved overlays — no cross-tenant leakage.
///
/// Single-tenant fallback: when `query_engine` is None or its `resolved_spec_map` is
/// None, fall back to `config_manager.sensor_specs` filtered by sensor_id == client_id.
/// (Same pattern as `render_client_sensors_resource` in resources.rs.)
fn build_tables_for_client(
    client_id: &str,
    query_engine: Option<&std::sync::Arc<prism_query::engine::QueryEngine>>,
    config_manager: Option<
        &std::sync::Arc<arc_swap::ArcSwap<prism_spec_engine::config_manager::ConfigManager>>,
    >,
) -> Vec<TableDescriptor> {
    // ── Multi-tenant path: resolved_spec_map filtered by OrgSlug ─────────────────
    if let Some(qe) = query_engine {
        if let Some(spec_map) = qe.resolved_spec_map() {
            // DI-008: filter by OrgSlug string to isolate per-client entries.
            let mut tables: Vec<TableDescriptor> = spec_map
                .iter()
                .filter(|((org, _sensor), _spec)| org.as_str() == client_id)
                .flat_map(|((_org, sensor_id), resolved)| {
                    let spec = &resolved.spec;
                    spec.tables.iter().map(move |table| {
                        let columns: Vec<ColumnDescriptor> = table
                            .columns
                            .iter()
                            .map(|col| ColumnDescriptor {
                                name: col.name.clone(),
                                col_type: col.column_type.clone(),
                                description: col.ocsf_field.clone(),
                                nullable: true,
                            })
                            .collect();
                        // BC-2.10.012 AUDIT-001: table name must be sensor-prefixed so that
                        // AI agents build valid `FROM crowdstrike_alerts | ...` queries, NOT
                        // bare `FROM alerts | ...` (which silently routes to E-SENSOR-030).
                        let prefixed_name = format!("{}_{}", sensor_id.as_ref(), table.table_name);
                        let example_query = build_example_query(&prefixed_name, &columns);
                        // BC-2.10.012 sensor_type fix: derive from the sensor identity
                        // (sensor_id from the resolved spec), NOT from client_id.
                        TableDescriptor {
                            name: prefixed_name,
                            sensor_type: sensor_id.as_ref().to_string(),
                            description: table.ocsf_class.clone(),
                            columns,
                            example_query,
                        }
                    })
                })
                .collect();
            // Sort by name for deterministic output.
            tables.sort_by(|a, b| a.name.cmp(&b.name));
            return tables;
        }
    }

    // ── Single-tenant fallback: config_manager filtered by sensor_id == client_id ──
    let Some(cm) = config_manager else {
        return Vec::new();
    };

    // Pattern: config_manager.load().load() follows the same two-level deref as resources.rs.
    let cm_guard = cm.load();
    let snapshot = cm_guard.load();

    // DI-008: filter strictly to the sensor matching the client_id as sensor_id.
    // This prevents cross-client data leakage (acme must never see globex tables).
    let Some(sensor_spec) = snapshot.sensor_specs.get(client_id) else {
        return Vec::new();
    };

    sensor_spec
        .tables
        .iter()
        .map(|table| {
            let columns: Vec<ColumnDescriptor> = table
                .columns
                .iter()
                .map(|col| ColumnDescriptor {
                    name: col.name.clone(),
                    col_type: col.column_type.clone(),
                    description: col.ocsf_field.clone(),
                    nullable: true,
                })
                .collect();

            // BC-2.10.012 AUDIT-001: table name must be sensor-prefixed so that AI agents
            // build valid `FROM crowdstrike_alerts | ...` queries (not bare `FROM alerts`).
            let prefixed_name = format!("{}_{}", sensor_spec.sensor_id, table.table_name);
            let example_query = build_example_query(&prefixed_name, &columns);

            TableDescriptor {
                name: prefixed_name,
                // BC-2.10.012 sensor_type fix: derive from the sensor spec's sensor_id,
                // NOT from client_id (in single-tenant mode they happen to be the same,
                // but the canonical source is the spec's sensor_id).
                sensor_type: sensor_spec.sensor_id.clone(),
                description: table.ocsf_class.clone(),
                columns,
                example_query,
            }
        })
        .collect()
}

/// Build pql_hints for the response based on the discovered tables.
///
/// When `tables` is empty and `org_registry` is provided, consults the registry to
/// distinguish two cases (BC-2.10.012 §Non-existent client_id handling):
///
/// - Registered-but-empty (org_registry KNOWS the slug, zero sensor overlays):
///   `"No sensor tables are available for client '<client_id>'. The client may not have any sensor overlays configured."`
///
/// - Not-registered (slug absent from org_registry entirely):
///   `"Client '<client_id>' is not registered. Check prism.toml [[orgs]] configuration."`
///
/// When `org_registry` is None (single-tenant / config_manager-only path), uses the
/// registered-but-empty hint by default (no registry available to distinguish).
///
/// # Category-2 enrichment-discovery hint (BC-2.10.012 v1.7 §pql_hints, AC-CAT2)
///
/// When `tables` is non-empty, a third hint is appended:
///
/// - `infusion_registry` is `Some(reg)` and `reg.udf_descriptors()` is non-empty:
///   UDFs are sorted alphabetically by name. The first sorted UDF is used as the example.
///   Hint: `"Enrichment available via pipe syntax: | enrich <first>(input). Available UDFs
///   for this client: <name1>(input1), <name2>(input2)"`
///
/// - `infusion_registry` is `None` or `reg.udf_descriptors()` is empty:
///   Hint: `"No enrichment UDFs are registered for this client — enrichment is not available."`
///
/// Category-2 is suppressed entirely when `tables` is empty (N = 0 tables). This prevents
/// advertising enrichment for clients that have no sensor data at all.
fn build_pql_hints(
    client_id: &str,
    tables: &[TableDescriptor],
    org_registry: Option<&prism_core::OrgRegistry>,
    infusion_registry: Option<&prism_spec_engine::InfusionRegistry>,
) -> Vec<String> {
    if tables.is_empty() {
        // Consult org_registry when available (multi-tenant path) to distinguish
        // registered-but-empty from not-registered (BC-2.10.012).
        if let Some(registry) = org_registry {
            // client_id was validated by OrgSlug::new earlier in handle_prism_describe,
            // so new() here will always produce a valid slug. We use new() directly
            // (returns OrgSlug with is_ok()/is_err() semantics, not Result).
            let slug = prism_core::OrgSlug::new(client_id);
            if slug.is_ok() {
                if registry.slug_exists(&slug) {
                    // Registered in OrgRegistry but no sensor overlays.
                    return vec![format!(
                        "No sensor tables are available for client '{client_id}'. \
                         The client may not have any sensor overlays configured."
                    )];
                } else {
                    // Not registered in OrgRegistry at all.
                    return vec![format!(
                        "Client '{client_id}' is not registered. \
                         Check prism.toml [[orgs]] configuration."
                    )];
                }
            }
        }
        // Single-tenant fallback (no registry): use the registered-but-empty hint.
        return vec![format!(
            "No sensor tables are available for client '{client_id}'. \
             The client may not have any sensor overlays configured."
        )];
    }

    // N ≥ 1 tables: emit two Category-1 hints + one Category-2 enrichment hint.
    let mut hints = vec![
        format!(
            "Use 'SELECT * FROM <table> LIMIT 25' to query any of the {} table(s) above.",
            tables.len()
        ),
        "Consult prismql://reference for full PQL grammar and operator reference.".to_string(),
    ];

    // Category-2: enrichment-discovery hint (BC-2.10.012 v1.7 §pql_hints AC-CAT2).
    // Only emitted when N ≥ 1 tables (suppressed for zero-table case above).
    let cat2_hint = build_enrichment_hint(infusion_registry);
    hints.push(cat2_hint);

    hints
}

/// Build the Category-2 enrichment-discovery hint (BC-2.10.012 v1.7 §pql_hints AC-CAT2).
///
/// - Non-empty registry: sort UDFs alphabetically by name; build comma-joined list of
///   `"name(input_field)"` entries; use first sorted entry as the example call.
///   Format: `"Enrichment available via pipe syntax: | enrich <first>. Available UDFs
///   for this client: <list>"`
///
/// - None registry or empty registry: `"No enrichment UDFs are registered for this
///   client — enrichment is not available."`
///
/// SAP-1 note: this is pure string assembly — NO `event_type` tracing emission is added
/// (PO-confirmed: BC-2.10.012 v1.7 / AC-CAT2 does not require a new catalog row).
fn build_enrichment_hint(
    infusion_registry: Option<&prism_spec_engine::InfusionRegistry>,
) -> String {
    const ABSENT: &str =
        "No enrichment UDFs are registered for this client — enrichment is not available.";

    let Some(reg) = infusion_registry else {
        return ABSENT.to_string();
    };

    let mut descriptors = reg.udf_descriptors();
    if descriptors.is_empty() {
        return ABSENT.to_string();
    }

    // Sort alphabetically by UDF name for deterministic output (BC-2.10.012 EC-10-030).
    descriptors.sort_by(|a, b| a.name.cmp(&b.name));

    // Build the comma-joined list: "name(input_field)" for each UDF.
    let list: String = descriptors
        .iter()
        .map(|d| format!("{}({})", d.name, d.input_field))
        .collect::<Vec<_>>()
        .join(", ");

    // First sorted entry is the example call (BC-2.10.012 EC-10-030).
    // SAFETY: `descriptors` is non-empty (checked above).
    let first = format!("{}({})", descriptors[0].name, descriptors[0].input_field);

    format!(
        "Enrichment available via pipe syntax: | enrich {first}. \
         Available UDFs for this client: {list}"
    )
}

/// Per-sensor severity vocabulary — maps sensor name prefix to the DTU-emitted
/// severity literal casing.
///
/// # Demo sensor coverage (all 4 sensors)
///
/// Only sensors whose exact DTU-emitted severity casing is registered here will have
/// a severity-filter variant in their example query.  Sensors absent from this list
/// fall through to the count-recent or column-free query branch.
///
/// ## Sensors registered in this vocabulary
///
/// - `crowdstrike`: Title-case — `"High"`, `"Critical"`
///   Source: `crates/prism-dtu-crowdstrike/src/generator.rs`
///   `make_detection_with_ioc()` severity_id mapping: 1→"Low", 2→"Medium", 3→"High", _→"Critical"
///   Example query emits `WHERE severity IN ('High', 'Critical') LIMIT 50`.
///
/// - `armis`: UPPER-case — `"HIGH"`, `"CRITICAL"`
///   Source: `crates/prism-dtu-armis/src/generator.rs` `build_alert()` severity param
///   assigned as `"HIGH"`, `"CRITICAL"`, `"MEDIUM"`, `"LOW"`
///   Example query emits `WHERE severity IN ('HIGH', 'CRITICAL') LIMIT 50`.
///
/// - `cyberint`: lowercase — `"high"`, `"critical"`
///   Source: `crates/prism-dtu-cyberint/src/generator.rs`
///   `let severities = ["low", "medium", "high", "critical"];` (F-PHL2-MED-001 fix)
///   Example query emits `WHERE severity IN ('high', 'critical') LIMIT 50`.
///
/// ## Sensors intentionally omitted
///
/// - `claroty`: NO severity column of any kind in its normalized schema
///   (`has_severity = false`).  The claroty TOML spec declares no `severity` and
///   no `severity_id` column.  (The Claroty xDome API does emit a `severity_id`
///   integer field on the wire, but it was intentionally omitted from the declared
///   column set — see the Gap-CL-005 comment in claroty.sensor.toml for history.)
///   The `has_severity` check in `build_example_query` tests for a column named
///   `"severity"` (String), which claroty never declares, so claroty tables never
///   reach the severity-filter branch at all.  No vocabulary entry is needed or
///   appropriate.
///
/// ## Rule: adding a new sensor
///
/// Before adding an entry to this vocabulary, verify the EXACT casing of the
/// severity string values emitted by the corresponding DTU clone
/// (`crates/prism-dtu-<sensor>/src/generator.rs`).  Wrong casing causes the
/// severity filter to silently return 0 rows against live or DTU data.
///
/// F-L2-CRIT-001 fix (S-DEMO-FIDELITY-REMEDIATION-001).
/// F-PGL2-LOW-001 doc expansion (S-DEMO-FIDELITY-REMEDIATION-001).
/// F-PHL2-MED-001 cyberint entry (S-DEMO-FIDELITY-REMEDIATION-001 Pass-H).
/// F-PIL2-LOW-001 doc consolidation (S-DEMO-FIDELITY-REMEDIATION-001 Pass-I).
const SENSOR_SEVERITY_VOCABULARY: &[(&str, &str, &str)] = &[
    // (sensor_prefix, high_literal, critical_literal)
    ("crowdstrike", "High", "Critical"),
    ("armis", "HIGH", "CRITICAL"),
    // cyberint emits lowercase severity: "low", "medium", "high", "critical"
    // Source: crates/prism-dtu-cyberint/src/generator.rs severities array.
    // F-PHL2-MED-001 (S-DEMO-FIDELITY-REMEDIATION-001 Pass-H).
    ("cyberint", "high", "critical"),
];

/// Derive the severity literals for a table based on its sensor prefix.
///
/// Returns `Some((high_literal, critical_literal))` for sensors registered in
/// `SENSOR_SEVERITY_VOCABULARY`, `None` for all others.
///
/// For the four demo sensors:
/// - `crowdstrike_*` → `Some(("High", "Critical"))` (Title-case)
/// - `armis_*` → `Some(("HIGH", "CRITICAL"))` (UPPER-case)
/// - `cyberint_*` → `Some(("high", "critical"))` (lowercase; F-PHL2-MED-001 fix)
/// - `claroty_*` → `None` (no string `severity` column; never reaches this
///   function from `build_example_query` because `has_severity` is false
///   for claroty tables)
///
/// F-L2-CRIT-001: unknown sensor → `None` → severity filter is suppressed, falling
/// back to count-recent or column-free — never a silent zero-row filter.
fn severity_literals_for_table(table_name: &str) -> Option<(&'static str, &'static str)> {
    // Table names are sensor-prefixed: "crowdstrike_detections", "armis_alerts", etc.
    for (prefix, high, critical) in SENSOR_SEVERITY_VOCABULARY {
        if table_name.starts_with(prefix) {
            return Some((high, critical));
        }
    }
    None
}

/// Build an auto-generated example PQL query for a table.
///
/// Produces an executable example query that references ONLY columns that actually exist
/// in the table's spec.  All three query variants (count-recent, severity-filter, aggregate)
/// must be syntactically and semantically valid against the table's actual schema.
///
/// # CRIT-1 fix (S-DEMO-FIDELITY-REMEDIATION-001)
///
/// The previous implementation hardcoded `timestamp` in the count-recent fallback
/// (`WHERE timestamp > NOW() - INTERVAL '1h'`).  Tables without a `timestamp` column
/// (e.g. `claroty_devices`) produced a non-executable example_query that violated
/// AUDIT-001/AUDIT-004.
///
/// The fix:
/// - Derive the time column from the table's actual `Datetime`-typed columns (first one).
/// - If no datetime column exists, emit a column-free form: `SELECT * FROM <t> LIMIT 25`.
///
/// # F-L2-CRIT-001 fix (S-DEMO-FIDELITY-REMEDIATION-001)
///
/// The previous implementation hardcoded lowercase `'high'`/`'critical'` in the
/// severity filter variant. CrowdStrike DTU emits Title-case (`High`/`Critical`) and
/// Armis DTU emits UPPER-case (`HIGH`/`CRITICAL`). Lowercase literals match no rows.
///
/// The fix:
/// - Derive severity literals from `SENSOR_SEVERITY_VOCABULARY` keyed on sensor prefix.
/// - Only emit a severity filter when the sensor prefix is in the registered vocabulary.
/// - For unknown sensor prefixes with a severity column, fall back to count-recent or
///   column-free rather than emitting literals that silently return 0 rows.
///
/// # Variant selection (priority: aggregate > severity > count-recent/simple)
///
/// | Condition                                          | Query emitted |
/// |----------------------------------------------------|---------------|
/// | Integer/Float column present                       | GROUP BY aggregate |
/// | severity column present + known sensor vocabulary  | WHERE severity IN (...) LIMIT 50 |
/// | Datetime column found (no above)                   | COUNT(*) WHERE <dt_col> > NOW() - INTERVAL '1h' |
/// | No datetime column (no above)                      | SELECT * FROM <t> LIMIT 25 |
///
/// BC-2.10.012 / AUDIT-001 / AUDIT-004; S-DEMO-FIDELITY-REMEDIATION-001 CRIT-1 + F-L2-CRIT-001.
pub fn build_example_query(table_name: &str, columns: &[ColumnDescriptor]) -> String {
    use prism_core::column::ColumnType;

    // CRIT-1 fix: derive time column from actual Datetime-typed columns.
    // Prefer the first Datetime-typed column; do NOT assume a column named "timestamp" exists.
    let datetime_col: Option<&str> = columns
        .iter()
        .find(|c| matches!(c.col_type, ColumnType::Datetime))
        .map(|c| c.name.as_str());

    // Count-recent fallback (only when a datetime column exists).
    // BC-2.10.012 canonical: SELECT COUNT(*) FROM <t> WHERE <datetime_col> > NOW() - INTERVAL '1h'
    // CRIT-1 fix: use actual datetime column name, not hardcoded "timestamp".
    // If no datetime column exists, fall back to a column-free SELECT * LIMIT 25.
    let mut query = if let Some(dt_col) = datetime_col {
        format!("SELECT COUNT(*) FROM {table_name} WHERE {dt_col} > NOW() - INTERVAL '1h'")
    } else {
        // No datetime column — emit a simple scan that is always valid.
        format!("SELECT * FROM {table_name} LIMIT 25")
    };

    // Severity filter variant when a severity column is present AND the sensor has a
    // registered vocabulary.
    //
    // F-L2-CRIT-001 fix: only emit severity literals when we know the correct casing
    // for this sensor's DTU-emitted vocabulary. Unknown sensor prefix → skip severity
    // variant and keep count-recent/column-free to avoid silent zero-row queries.
    let has_severity = columns.iter().any(|c| c.name == "severity");
    if has_severity {
        if let Some((high, critical)) = severity_literals_for_table(table_name) {
            query = format!(
                "SELECT * FROM {table_name} WHERE severity IN ('{high}', '{critical}') LIMIT 50"
            );
        }
        // If no vocabulary → fall through, keeping count-recent or column-free query.
    }

    // Aggregate variant when an aggregatable column is present (overrides severity if both).
    // BC-2.10.012 canonical: SELECT <field>, COUNT(*) FROM <t> GROUP BY <field> ORDER BY COUNT(*) DESC LIMIT 10
    let agg_col = columns
        .iter()
        .find(|c| matches!(c.col_type, ColumnType::Integer | ColumnType::Float));
    if let Some(col) = agg_col {
        query = format!(
            "SELECT {col_name}, COUNT(*) FROM {table_name} GROUP BY {col_name} ORDER BY COUNT(*) DESC LIMIT 10",
            col_name = col.name
        );
    }

    query
}

#[cfg(test)]
mod build_example_query_tests {
    use super::{build_example_query, ColumnDescriptor};
    use prism_core::column::ColumnType;

    fn col(name: &str, col_type: ColumnType) -> ColumnDescriptor {
        ColumnDescriptor {
            name: name.to_string(),
            col_type,
            description: None,
            nullable: false,
        }
    }

    /// CRIT-1 load-bearing: table with NO datetime column must produce a column-free query.
    ///
    /// `claroty_devices` is the genuine no-datetime table (no Datetime-typed columns in spec).
    /// The old hardcoded `WHERE timestamp > NOW() - INTERVAL '1h'` produced a non-executable
    /// example. This test asserts the fallback `SELECT * FROM <t> LIMIT 25` is used instead.
    ///
    /// F-L2-LOW-001 fix: the comment previously (incorrectly) cited `crowdstrike_devices` as
    /// the no-datetime example. `crowdstrike_devices` actually has `first_seen` and `last_seen`
    /// (Datetime). `claroty_devices` is the correct no-datetime example — it has only String
    /// and Boolean columns (`uid`, `asset_id`, `device_category`, `device_type`, `risk_score`,
    /// `retired`). See `crates/prism-sensors/specs/claroty.sensor.toml`.
    ///
    /// Load-bearing (TD-VSDD-059): fails if the datetime-column derivation is replaced by
    /// a hardcoded "timestamp" again.
    #[test]
    fn test_crit1_no_datetime_column_produces_column_free_query() {
        // Simulate claroty_devices: genuinely no Datetime column.
        // (crowdstrike_devices has first_seen/last_seen Datetime — do NOT use it here.)
        let columns = vec![
            col("uid", ColumnType::String),
            col("asset_id", ColumnType::String),
            col("device_category", ColumnType::String),
            col("device_type", ColumnType::String),
            col("risk_score", ColumnType::String),
            col("retired", ColumnType::Boolean),
        ];

        let q = build_example_query("claroty_devices", &columns);

        assert_eq!(
            q, "SELECT * FROM claroty_devices LIMIT 25",
            "CRIT-1: table with no Datetime column must use column-free fallback. Got: {q}"
        );
        // Must NOT contain hardcoded "timestamp" (the bug being fixed).
        assert!(
            !q.contains("timestamp"),
            "CRIT-1: fallback query must NOT contain hardcoded 'timestamp'. Got: {q}"
        );
    }

    /// CRIT-1 load-bearing: table WITH a datetime column uses that column in count-recent.
    ///
    /// `crowdstrike_alerts` has `event_time` (Datetime). The example_query should use
    /// `event_time`, not hardcoded `timestamp`.
    #[test]
    fn test_crit1_datetime_column_named_event_time_used_in_count_recent() {
        let columns = vec![
            col("event_time", ColumnType::Datetime),
            col("hostname", ColumnType::String),
        ];

        let q = build_example_query("crowdstrike_alerts", &columns);

        assert!(
            q.contains("event_time"),
            "CRIT-1: Datetime column 'event_time' must appear in count-recent query. Got: {q}"
        );
        assert!(
            !q.contains("timestamp"),
            "CRIT-1: query must use actual Datetime column name, not hardcoded 'timestamp'. Got: {q}"
        );
        assert!(
            q.contains("NOW()"),
            "CRIT-1: count-recent query must contain NOW(). Got: {q}"
        );
    }

    /// CRIT-1 + F-L2-CRIT-001: table with `timestamp` datetime column and severity column
    /// uses severity variant with correct per-sensor casing (regression guard).
    ///
    /// `crowdstrike_detections` has `created_timestamp` (Datetime) and `severity` (String).
    /// The severity variant takes priority over count-recent. The literals must be Title-case
    /// (`'High'`, `'Critical'`) to match CrowdStrike DTU vocabulary.
    ///
    /// F-L2-CRIT-001 fix (TD-VSDD-059): the prior assertion `('high', 'critical')` was a
    /// paper-confirmation of the bug. Corrected to assert `('High', 'Critical')`.
    #[test]
    fn test_crit1_datetime_column_named_timestamp_still_works() {
        let columns = vec![
            col("timestamp", ColumnType::Datetime),
            col("severity", ColumnType::String),
        ];

        let q = build_example_query("crowdstrike_detections", &columns);

        // severity variant takes priority over count-recent.
        // F-L2-CRIT-001: CrowdStrike DTU emits Title-case — 'High'/'Critical', NOT 'high'/'critical'.
        assert_eq!(
            q,
            "SELECT * FROM crowdstrike_detections WHERE severity IN ('High', 'Critical') LIMIT 50",
            "CRIT-1 + F-L2-CRIT-001: severity variant must be selected with Title-case CrowdStrike \
             vocabulary. Got: {q}"
        );
    }

    /// CRIT-1: zero-column table (EC-002) uses column-free fallback.
    #[test]
    fn test_crit1_zero_columns_uses_column_free_fallback() {
        let q = build_example_query("empty_table", &[]);

        assert_eq!(
            q, "SELECT * FROM empty_table LIMIT 25",
            "CRIT-1 EC-002: zero-column table must use column-free fallback. Got: {q}"
        );
        assert!(
            !q.contains("timestamp"),
            "CRIT-1 EC-002: fallback must not contain hardcoded 'timestamp'. Got: {q}"
        );
    }

    // ── F-L2-CRIT-001 load-bearing tests ─────────────────────────────────────
    //
    // These tests assert that build_example_query emits severity literals in the
    // CORRECT per-sensor casing so the example returns real rows against DTU data.
    //
    // CrowdStrike DTU emits Title-case severity: "High", "Critical", "Medium", "Low"
    //   Source: crates/prism-dtu-crowdstrike/src/generator.rs make_detection_with_ioc()
    //           severity_id 1=>"Low", 2=>"Medium", 3=>"High", _=>"Critical"
    //
    // Armis DTU emits UPPER-case severity: "HIGH", "CRITICAL", "MEDIUM", "LOW"
    //   Source: crates/prism-dtu-armis/src/generator.rs build_alert() severity param
    //           "HIGH", "CRITICAL", "MEDIUM"
    //
    // The previous code emitted lowercase 'high'/'critical' which matches NEITHER DTU.
    // These tests FAIL RED against the old code and PASS GREEN after the fix.
    // Load-bearing (TD-VSDD-059): if the casing regresses, these tests catch it.

    /// F-L2-CRIT-001: crowdstrike_detections severity variant must use Title-case literals.
    ///
    /// DTU emits: "High", "Critical" — NOT lowercase "high"/"critical".
    /// A query with lowercase literals returns 0 rows from DTU data silently.
    ///
    /// Load-bearing: reverting to lowercase 'high'/'critical' makes this test fail.
    #[test]
    fn test_f_l2_crit001_crowdstrike_detections_severity_uses_title_case() {
        let columns = vec![
            col("created_timestamp", ColumnType::Datetime),
            col("severity", ColumnType::String),
            col("status", ColumnType::String),
        ];

        let q = build_example_query("crowdstrike_detections", &columns);

        // Must use Title-case (DTU emits "High"/"Critical", NOT "high"/"critical").
        assert!(
            q.contains("'High'") && q.contains("'Critical'"),
            "F-L2-CRIT-001: crowdstrike_detections severity IN must use Title-case \
             'High'/'Critical' to match DTU vocabulary. Got: {q}"
        );
        assert!(
            !q.contains("'high'") && !q.contains("'critical'"),
            "F-L2-CRIT-001: crowdstrike_detections must NOT use lowercase 'high'/'critical' \
             (DTU emits Title-case). Got: {q}"
        );
    }

    /// F-L2-CRIT-001: armis_alerts severity variant must use UPPER-case literals.
    ///
    /// DTU emits: "HIGH", "CRITICAL" — NOT lowercase "high"/"critical".
    /// A query with lowercase literals returns 0 rows from DTU data silently.
    ///
    /// Load-bearing: reverting to lowercase 'high'/'critical' makes this test fail.
    #[test]
    fn test_f_l2_crit001_armis_alerts_severity_uses_upper_case() {
        let columns = vec![
            col("alert_id", ColumnType::String),
            col("severity", ColumnType::String),
            col("status", ColumnType::String),
        ];

        let q = build_example_query("armis_alerts", &columns);

        // Must use UPPER-case (DTU emits "HIGH"/"CRITICAL", NOT "high"/"critical").
        assert!(
            q.contains("'HIGH'") && q.contains("'CRITICAL'"),
            "F-L2-CRIT-001: armis_alerts severity IN must use UPPER-case \
             'HIGH'/'CRITICAL' to match DTU vocabulary. Got: {q}"
        );
        assert!(
            !q.contains("'high'") && !q.contains("'critical'"),
            "F-L2-CRIT-001: armis_alerts must NOT use lowercase 'high'/'critical' \
             (DTU emits UPPER-case). Got: {q}"
        );
    }

    /// F-L2-CRIT-001: unknown sensor falls back to count-recent or column-free, NOT a
    /// severity filter with potentially wrong casing.
    ///
    /// If a table from an unknown sensor has a severity column but no registered
    /// vocabulary, build_example_query must NOT emit a hardcoded severity literal
    /// that could return 0 rows. It falls back to count-recent (if datetime present)
    /// or column-free (if no datetime).
    #[test]
    fn test_f_l2_crit001_unknown_sensor_with_severity_falls_back_to_count_recent() {
        let columns = vec![
            col("created_at", ColumnType::Datetime),
            col("severity", ColumnType::String),
        ];

        let q = build_example_query("unknown_sensor_events", &columns);

        // Unknown sensor: no vocabulary → no severity filter. Must use count-recent.
        assert!(
            q.contains("COUNT(*)") && q.contains("created_at"),
            "F-L2-CRIT-001: unknown sensor with severity column must fall back to \
             count-recent (no severity vocabulary registered for it). Got: {q}"
        );
        assert!(
            !q.contains("'high'") && !q.contains("'critical'"),
            "F-L2-CRIT-001: unknown sensor must NOT emit lowercase severity literals. Got: {q}"
        );
    }

    // ── F-PHL2-MED-001: cyberint severity vocabulary (lowercase) ─────────────
    //
    // Cyberint DTU emits lowercase severity: "low", "medium", "high", "critical"
    //   Source: crates/prism-dtu-cyberint/src/generator.rs
    //           let severities = ["low", "medium", "high", "critical"];
    //
    // Before fix: cyberint was NOT in SENSOR_SEVERITY_VOCABULARY →
    //   build_example_query("cyberint_alerts", …) with a severity column fell through
    //   to count-recent (no vocabulary registered). Querying with lowercase literals
    //   is correct (DTU emits lowercase), so the example returned real rows.
    //   Suppressing it on "demo-scope" grounds violated Rule 1 (no MVP deferrals).
    //
    // After fix: cyberint is added with ("cyberint", "high", "critical") →
    //   severity branch fires for cyberint_alerts when a severity String column exists.
    //   The example query emits WHERE severity IN ('high', 'critical') LIMIT 50.
    //
    // Load-bearing (TD-VSDD-059): these tests fail before the vocabulary entry is added
    // and pass after.

    /// F-PHL2-MED-001: cyberint_alerts severity variant must use lowercase literals.
    ///
    /// DTU emits: "high", "critical" — lowercase (NOT Title-case or UPPER-case).
    /// After adding cyberint to SENSOR_SEVERITY_VOCABULARY, severity branch must fire
    /// when severity String column is present and emit lowercase 'high'/'critical'.
    ///
    /// Load-bearing (TD-VSDD-059): before vocabulary entry, severity branch is suppressed
    /// and count-recent fires instead → this assert on `q.contains("'high'")` fails.
    #[test]
    fn test_f_phl2_med001_cyberint_alerts_severity_uses_lower_case() {
        let columns = vec![
            col("created_at", ColumnType::Datetime),
            col("severity", ColumnType::String),
            col("title", ColumnType::String),
        ];

        let q = build_example_query("cyberint_alerts", &columns);

        // Must use lowercase (DTU emits "high"/"critical", NOT Title-case or UPPER-case).
        assert!(
            q.contains("'high'") && q.contains("'critical'"),
            "F-PHL2-MED-001: cyberint_alerts severity IN must use lowercase 'high'/'critical' \
             to match DTU vocabulary. Got: {q}. \
             Before fix: cyberint had no vocabulary entry → count-recent branch fired."
        );
        // Must NOT use Title-case or UPPER-case (wrong casing returns 0 rows against DTU).
        assert!(
            !q.contains("'High'") && !q.contains("'CRITICAL'"),
            "F-PHL2-MED-001: cyberint_alerts must NOT use Title-case or UPPER-case. Got: {q}"
        );
        // Severity branch fires (not count-recent which uses COUNT(*)).
        assert!(
            q.contains("WHERE severity IN"),
            "F-PHL2-MED-001: cyberint_alerts with severity column must use severity-filter \
             branch, not count-recent. Got: {q}"
        );
    }

    /// F-PHL2-MED-001: cyberint_alerts with severity_id (Integer) must use aggregate
    /// branch (Integer wins over severity-filter in priority ladder).
    ///
    /// The cyberint spec has severity_id as a u64 (integer) per DTU generator. When
    /// the spec exposes severity_id as an Integer column, aggregate fires first.
    ///
    /// This test verifies the priority ladder is respected: aggregate > severity.
    #[test]
    fn test_f_phl2_med001_cyberint_alerts_integer_col_triggers_aggregate_not_severity() {
        let columns = vec![
            col("created_at", ColumnType::Datetime),
            col("severity", ColumnType::String),
            col("severity_id", ColumnType::Integer), // Integer → aggregate branch wins
        ];

        let q = build_example_query("cyberint_alerts", &columns);

        // Integer column present → aggregate branch wins (highest priority).
        assert!(
            q.contains("GROUP BY") && q.contains("COUNT(*)"),
            "F-PHL2-MED-001 priority: cyberint_alerts with Integer severity_id must use \
             aggregate branch (Integer > severity-filter in priority ladder). Got: {q}"
        );
    }

    // ── F-P1L2-OBS-001 parse-roundtrip regression guards ─────────────────────
    //
    // Purpose: verify that the output of `build_example_query` for each priority-ladder
    // branch (aggregate / severity-filter / count-recent / column-free) is syntactically
    // valid PrismQL — i.e., parses without error through `PrismQlParser::parse`.
    //
    // Guards against a future grammar or planner change silently making the demo
    // example non-executable. Each probe uses a representative table + column set that
    // exercises the branch.
    //
    // Branch mapping:
    //   armis_devices + risk_score Integer → AGGREGATE (Integer col → GROUP BY path)
    //   crowdstrike_detections + severity String + created_timestamp Datetime → SEVERITY-FILTER
    //   cyberint_alerts + event_time Datetime (no severity, no Integer) → COUNT-RECENT
    //   claroty_devices (no Datetime, no Integer) → COLUMN-FREE fallback
    //
    // Load-bearing (F-P1L2-OBS-001 / TD-VSDD-059): a grammar regression that makes
    // any branch non-parseable causes the corresponding assert to fail immediately,
    // surfacing the breakage before it reaches a demo recording.

    /// F-P1L2-OBS-001: aggregate branch (`SELECT <col>, COUNT(*) FROM <t> GROUP BY <col>
    /// ORDER BY COUNT(*) DESC LIMIT 10`) parses through PrismQlParser.
    ///
    /// Representative table: armis_devices with Integer risk_score.
    /// The Integer column triggers the aggregate branch (highest priority in the ladder).
    ///
    /// # End-to-end executability (F-PIL2-OBS-001)
    ///
    /// armis_devices uses the AQL-passthrough path (`path_template = /api/v1/search?aql=
    /// ${query.filter.aql}`).  When the example query carries no AQL predicate, the
    /// PipelineExecutor pre-seeds `${query.filter.aql}` to an empty string (pipeline.rs,
    /// ADR-033 T1 / AC-CWS-002), producing `GET /api/v1/search?aql=`.  The DTU
    /// `get_search` handler receives `params.aql = Some("")`, which does NOT contain
    /// `"in:alerts"`, so it defaults to returning all device records (EC-001 in search.rs).
    /// DataFusion then executes the `GROUP BY risk_score` aggregation locally over the
    /// fetched rows.  The aggregate example is therefore end-to-end executable against
    /// the DTU without any predicate — VERIFIED EXECUTABLE.
    #[test]
    fn test_obs001_roundtrip_aggregate_branch_parses() {
        use prism_query::filter_parser::PrismQlParser;
        let columns = vec![
            col("risk_score", ColumnType::Integer),
            col("name", ColumnType::String),
        ];
        let q = build_example_query("armis_devices", &columns);
        // Verify the aggregate branch was chosen.
        assert!(
            q.contains("GROUP BY") && q.contains("COUNT(*)"),
            "OBS-001 setup: expected aggregate branch for armis_devices+Integer; got: {q}"
        );
        let result = PrismQlParser::parse(&q);
        assert!(
            result.is_ok(),
            "F-P1L2-OBS-001: aggregate branch output must parse successfully. \
             Query: {q:?}. Error: {result:?}"
        );
    }

    /// F-P1L2-OBS-001: severity-filter branch (`SELECT * FROM <t> WHERE severity IN
    /// ('<High>', '<Critical>') LIMIT 50`) parses through PrismQlParser.
    ///
    /// Representative table: crowdstrike_detections (known vocabulary: 'High'/'Critical').
    /// Severity takes priority over count-recent when vocabulary is known.
    #[test]
    fn test_obs001_roundtrip_severity_filter_branch_parses() {
        use prism_query::filter_parser::PrismQlParser;
        let columns = vec![
            col("created_timestamp", ColumnType::Datetime),
            col("severity", ColumnType::String),
        ];
        let q = build_example_query("crowdstrike_detections", &columns);
        // Verify the severity-filter branch was chosen.
        assert!(
            q.contains("WHERE severity IN"),
            "OBS-001 setup: expected severity-filter branch for crowdstrike_detections; got: {q}"
        );
        let result = PrismQlParser::parse(&q);
        assert!(
            result.is_ok(),
            "F-P1L2-OBS-001: severity-filter branch output must parse successfully. \
             Query: {q:?}. Error: {result:?}"
        );
    }

    /// F-P1L2-OBS-001: count-recent branch (`SELECT COUNT(*) FROM <t> WHERE <datetime_col>
    /// > NOW() - INTERVAL '1h'`) parses through PrismQlParser.
    ///
    /// Representative table: cyberint_alerts with Datetime event_time, no severity/Integer.
    /// Datetime col + no severity vocabulary + no Integer → count-recent fallback.
    #[test]
    fn test_obs001_roundtrip_count_recent_branch_parses() {
        use prism_query::filter_parser::PrismQlParser;
        let columns = vec![
            col("event_time", ColumnType::Datetime),
            col("title", ColumnType::String),
        ];
        let q = build_example_query("cyberint_alerts", &columns);
        // Verify the count-recent branch was chosen (no severity vocabulary for cyberint_alerts).
        assert!(
            q.contains("COUNT(*)") && q.contains("NOW()") && q.contains("event_time"),
            "OBS-001 setup: expected count-recent branch for cyberint_alerts; got: {q}"
        );
        let result = PrismQlParser::parse(&q);
        assert!(
            result.is_ok(),
            "F-P1L2-OBS-001: count-recent branch output must parse successfully. \
             Query: {q:?}. Error: {result:?}"
        );
    }

    /// F-P1L2-OBS-001: column-free fallback (`SELECT * FROM <t> LIMIT 25`) parses
    /// through PrismQlParser.
    ///
    /// Representative table: claroty_devices (no Datetime, no Integer, no severity).
    /// All priority-ladder branches skipped → column-free SELECT * fallback.
    #[test]
    fn test_obs001_roundtrip_column_free_fallback_parses() {
        use prism_query::filter_parser::PrismQlParser;
        let columns = vec![
            col("uid", ColumnType::String),
            col("device_category", ColumnType::String),
        ];
        let q = build_example_query("claroty_devices", &columns);
        // Verify the column-free branch was chosen.
        assert_eq!(
            q, "SELECT * FROM claroty_devices LIMIT 25",
            "OBS-001 setup: expected column-free fallback for claroty_devices; got: {q}"
        );
        let result = PrismQlParser::parse(&q);
        assert!(
            result.is_ok(),
            "F-P1L2-OBS-001: column-free fallback output must parse successfully. \
             Query: {q:?}. Error: {result:?}"
        );
    }
}
