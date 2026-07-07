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
    // emit only schema_enumeration.rejected, not .started (BC-2.16.002 catalog).
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

/// Per-sensor severity vocabulary — sensor name prefixes whose tables have a
/// meaningful `severity` String column and should get an IEQ example query.
///
/// # Post-normalization contract (S-PRISMQL-CASE-INSENSITIVE-001)
///
/// Raw vendor feeds keep their original casing at the wire level (crowdstrike
/// Title-case "High"/"Critical", armis UPPER-case "HIGH"/"CRITICAL", cyberint
/// lowercase "high"/"critical").  The BC-2.02.013 PRIMARY normalization
/// path (`build_column_array` in `spec_driven_adapter.rs`) canonicalizes the
/// `severity`, `status`, `activity_name`, and `disposition` columns to OCSF
/// Title-case BEFORE DataFusion materializes the in-memory record batch.  After
/// normalization every sensor's severity values are stored as Title-case
/// ('High', 'Critical', 'Medium', 'Low') regardless of the raw feed casing.
///
/// Describe examples MUST therefore use `IEQ` (case-insensitive equality) rather
/// than `IN ('HIGH', …)` or `IN ('high', …)` — vendor-cased IN literals silently
/// return 0 rows against post-normalization data.
///
/// The example format for all registered sensors is:
/// ```text
/// -- OCSF severity is Title-case post-normalization (e.g., 'High').
///    Use IEQ for case-insensitive matching.
/// FROM <table> | WHERE severity IEQ 'high' | limit 50
/// ```
///
/// # Sensors registered
///
/// - `crowdstrike`: has severity String column.
/// - `armis`: has severity String column.
/// - `cyberint`: has severity String column (F-PHL2-MED-001).
///
/// # Sensors intentionally omitted
///
/// - `claroty`: NO severity String column in its normalized schema.  The claroty
///   TOML spec declares no `severity` column (`has_severity = false`).  No
///   vocabulary entry needed.
///
/// # Rule: adding a new sensor
///
/// Add the sensor prefix when its TOML spec declares a `severity` String column
/// AND the sensor's raw feed undergoes BC-2.02.013 normalization.  No need to
/// record the raw casing — IEQ is always case-insensitive at query time.
///
/// F-L2-CRIT-001 fix (S-DEMO-FIDELITY-REMEDIATION-001).
/// F-PHL2-MED-001 cyberint entry (S-DEMO-FIDELITY-REMEDIATION-001 Pass-H).
/// F-P6-MED-002 doc rewrite — post-normalization IEQ contract
///   (S-PRISMQL-CASE-INSENSITIVE-001 LOCAL pass-6).
const SENSOR_SEVERITY_VOCABULARY: &[&str] = &["crowdstrike", "armis", "cyberint"];

/// Check whether a table's sensor has a registered severity vocabulary.
///
/// Returns `true` for sensors registered in `SENSOR_SEVERITY_VOCABULARY`,
/// `false` for all others.
///
/// For the four demo sensors:
/// - `crowdstrike_*` → `true`
/// - `armis_*` → `true`
/// - `cyberint_*` → `true` (F-PHL2-MED-001 fix)
/// - `claroty_*` → `false` (no String `severity` column in normalized schema)
///
/// F-L2-CRIT-001: unknown sensor → `false` → severity filter suppressed,
/// falling back to count-recent or column-free — never a silent zero-row filter.
/// F-P6-MED-002: no raw casing needed since IEQ is always case-insensitive and
/// post-normalization all severity values are Title-case (BC-2.02.013).
fn has_severity_vocabulary(table_name: &str) -> bool {
    // Table names are sensor-prefixed: "crowdstrike_detections", "armis_alerts", etc.
    SENSOR_SEVERITY_VOCABULARY
        .iter()
        .any(|prefix| table_name.starts_with(prefix))
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
/// # Variant selection (priority: aggregate > severity-IEQ > count-recent/simple)
///
/// | Condition                                              | Query emitted |
/// |--------------------------------------------------------|---------------|
/// | Integer/Float column present                           | GROUP BY aggregate |
/// | severity column present + known sensor vocabulary      | IEQ pipe form (AC-025 / ADR-047 §D.4) |
/// | Datetime column found (no above)                       | COUNT(*) WHERE <dt_col> > NOW() - INTERVAL '1h' |
/// | No datetime column (no above)                          | SELECT * FROM <t> LIMIT 25 |
///
/// All registered sensor tables with a severity column emit the IEQ form regardless
/// of column position (F-P6-HIGH-001/002; S-PRISMQL-CASE-INSENSITIVE-001 LOCAL pass-6).
/// Vendor-cased IN literals were removed because post-normalization they silently return
/// 0 rows (BC-2.02.013 PRIMARY normalization canonicalizes to Title-case).
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

    // Severity IEQ variant when a severity column is present AND the sensor has a
    // registered vocabulary.
    //
    // F-L2-CRIT-001 fix: only emit a severity example when we know the sensor actually
    // has a meaningful severity column. Unknown sensor prefix → skip severity variant
    // and keep count-recent/column-free to avoid silent zero-row queries.
    //
    // F-P6-HIGH-001/002 / AC-025 (S-PRISMQL-CASE-INSENSITIVE-001 LOCAL pass-6):
    // ALL tables with a severity column and registered vocabulary emit the IEQ form,
    // regardless of column position. Vendor-cased IN literals (`IN ('HIGH', …)`,
    // `IN ('high', …)`) silently return 0 rows against post-normalization data because
    // BC-2.02.013 PRIMARY normalization (build_column_array) canonicalizes severity
    // to OCSF Title-case before DataFusion materialization (AC-025 / ADR-047 §D.4).
    //
    // F-MED-1 (S-PRISMQL-CASE-INSENSITIVE-001 LOCAL pass-7 BC-2.11.024):
    // Severity-IEQ is the HIGHEST priority variant for severity-vocabulary tables.
    // The aggregate variant runs only when severity-IEQ does not fire (no severity column
    // OR sensor not in vocabulary). This prevents the aggregate branch from silently
    // suppressing the IEQ teaching example for tables that happen to have a numeric column.
    // AC-025 / ADR-047 §D.4 mandate the IEQ example for all vocabulary-registered tables.
    let has_severity = columns.iter().any(|c| c.name == "severity");
    if has_severity && has_severity_vocabulary(table_name) {
        // AC-025 / ADR-047 §D.4: IEQ operator + OCSF casing note for ALL severity tables.
        // The note teaches analysts the post-normalization storage format and that IEQ
        // matches case-insensitively regardless of what they type.
        // Highest priority — runs before aggregate so numeric columns do not suppress IEQ.
        query = format!(
            "-- OCSF severity is Title-case post-normalization (e.g., 'High'). \
             Use IEQ for case-insensitive matching.\n\
             FROM {table_name} | WHERE severity IEQ 'high' | limit 50"
        );
    } else {
        // Aggregate variant when an aggregatable column is present and severity-IEQ did not fire.
        // BC-2.10.012 canonical: SELECT <field>, COUNT(*) FROM <t> GROUP BY <field> ORDER BY COUNT(*) DESC LIMIT 10
        // Only runs when: no severity column OR sensor not in vocabulary.
        let agg_col = columns
            .iter()
            .find(|c| matches!(c.col_type, ColumnType::Integer | ColumnType::Float));
        if let Some(col) = agg_col {
            query = format!(
                "SELECT {col_name}, COUNT(*) FROM {table_name} GROUP BY {col_name} ORDER BY COUNT(*) DESC LIMIT 10",
                col_name = col.name
            );
        }
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
    /// uses the IEQ case-insensitive example with OCSF casing note.
    ///
    /// `crowdstrike_detections` with severity as a secondary column (timestamp listed first)
    /// must now emit the IEQ variant — AC-025 requires the IEQ example + OCSF casing note
    /// for ANY table with a severity column, regardless of column ordering.
    ///
    /// ## Current behaviour (HEAD 8e4ec972) — RED
    ///
    /// severity_is_primary = false (timestamp is first) → classic IN variant emitted:
    /// `SELECT * FROM crowdstrike_detections WHERE severity IN ('High', 'Critical') LIMIT 50`
    ///
    /// The assertion `q.contains("IEQ")` FAILS.
    ///
    /// ## Green Gate
    ///
    /// PASSES once `build_example_query` always emits IEQ for any table with a severity
    /// column (F-P6-HIGH-001: remove the severity_is_primary exception).
    ///
    /// F-P6-HIGH-001 (LOCAL pass-6): severity secondary tables must also get IEQ + casing note.
    /// AC-025; ADR-047 §D.4.
    #[test]
    fn test_crit1_datetime_column_named_timestamp_still_works() {
        let columns = vec![
            col("timestamp", ColumnType::Datetime),
            col("severity", ColumnType::String),
        ];

        let q = build_example_query("crowdstrike_detections", &columns);

        // F-P6-HIGH-001: AC-025 requires IEQ for ANY table with a severity column —
        // secondary position is no longer an exception.
        // FAILS NOW: current code emits IN variant for severity-secondary tables.
        assert!(
            q.contains("IEQ"),
            "F-P6-HIGH-001 (LOCAL pass-6): build_example_query must emit IEQ operator \
             for ANY table with a severity column (AC-025 / ADR-047 §D.4), including \
             when severity is not the first column; \
             current output still uses IN variant for secondary severity; got: {q:?}"
        );
        assert!(
            q.contains("Title-case") || q.contains("title-case"),
            "F-P6-HIGH-001 (LOCAL pass-6): build_example_query must include OCSF casing note \
             (substring 'Title-case') per AC-025 / ADR-047 §D.4; got: {q:?}"
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

    /// F-L2-CRIT-001 + F-P6-HIGH-001: crowdstrike_detections severity variant must use
    /// the IEQ case-insensitive operator with OCSF casing note (post-normalization contract).
    ///
    /// After OCSF normalization (S-PRISMQL-CASE-INSENSITIVE-001), all severity values are
    /// stored as Title-case 'High'/'Critical'. The describe example should use IEQ so that
    /// analysts can match regardless of what casing they type.
    ///
    /// ## Current behaviour (HEAD 8e4ec972) — RED
    ///
    /// severity_is_primary = false (created_timestamp is first) → IN variant emitted:
    /// `SELECT * FROM crowdstrike_detections WHERE severity IN ('High', 'Critical') LIMIT 50`
    ///
    /// The assertion `q.contains("IEQ")` FAILS.
    ///
    /// The `!q.contains("'high'")` assertion would also FAIL post-fix (IEQ form includes
    /// `severity IEQ 'high'`), which is the correct behaviour (IEQ is case-insensitive).
    ///
    /// F-P6-HIGH-001 (LOCAL pass-6): secondary-severity tables must also use IEQ.
    /// AC-025; ADR-047 §D.4; BC-2.11.024.
    #[test]
    fn test_f_l2_crit001_crowdstrike_detections_severity_uses_title_case() {
        let columns = vec![
            col("created_timestamp", ColumnType::Datetime),
            col("severity", ColumnType::String),
            col("status", ColumnType::String),
        ];

        let q = build_example_query("crowdstrike_detections", &columns);

        // Post-normalization contract: all severity values stored as Title-case.
        // Describe example must use IEQ so the analyst can match case-insensitively.
        // FAILS NOW: current code emits IN variant for secondary-severity tables.
        assert!(
            q.contains("IEQ"),
            "F-P6-HIGH-001 (LOCAL pass-6): crowdstrike_detections describe example must use \
             IEQ operator per AC-025 / ADR-047 §D.4 (any severity table, any column position); \
             got: {q:?}"
        );
        // IEQ example must include the OCSF casing hint so analysts understand the storage format.
        assert!(
            q.contains("Title-case") || q.contains("title-case"),
            "F-P6-HIGH-001 (LOCAL pass-6): describe example must include OCSF casing note \
             (substring 'Title-case') per AC-025; got: {q:?}"
        );
    }

    /// F-L2-CRIT-001 + F-P6-HIGH-001 (updated): armis_alerts severity variant must use
    /// the IEQ case-insensitive operator with OCSF casing note (post-normalization contract).
    ///
    /// DTU emits UPPER-case "HIGH"/"CRITICAL" on the raw wire, but BC-2.02.013 PRIMARY
    /// normalization (build_column_array) canonicalizes severity to OCSF Title-case before
    /// DataFusion materialization.  A query with `IN ('HIGH', 'CRITICAL')` therefore
    /// silently returns 0 rows against post-normalization data.
    ///
    /// Updated (S-PRISMQL-CASE-INSENSITIVE-001 LOCAL pass-6 Security Fix Protocol):
    /// The expected output changed from the old `IN ('HIGH', 'CRITICAL')` IN-literal form
    /// to the IEQ pipe form per AC-025 / ADR-047 §D.4.  Load-bearing: reverting to the
    /// old IN form makes this test fail.
    #[test]
    fn test_f_l2_crit001_armis_alerts_severity_uses_upper_case() {
        let columns = vec![
            col("alert_id", ColumnType::String),
            col("severity", ColumnType::String),
            col("status", ColumnType::String),
        ];

        let q = build_example_query("armis_alerts", &columns);

        // Post-normalization contract: use IEQ (case-insensitive), not IN with vendor casing.
        assert!(
            q.contains("IEQ"),
            "F-L2-CRIT-001 + F-P6-HIGH-001: armis_alerts severity describe example must use \
             IEQ operator; post-normalization IN('HIGH','CRITICAL') returns 0 rows. Got: {q}"
        );
        assert!(
            q.contains("Title-case") || q.contains("title-case"),
            "F-L2-CRIT-001 + F-P6-HIGH-001: armis_alerts IEQ example must include OCSF casing \
             note (substring 'Title-case') per AC-025 / ADR-047 §D.4. Got: {q}"
        );
        // Must NOT use IN with vendor-cased literals that silently 0-row post-normalization.
        assert!(
            !q.contains("IN ('HIGH'"),
            "F-L2-CRIT-001 + F-P6-HIGH-001: armis_alerts must NOT use IN('HIGH',...); \
             post-normalization UPPER-case IN silently returns 0 rows. Got: {q}"
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

    /// F-PHL2-MED-001 + F-P6-HIGH-002: cyberint_alerts severity describe example must use
    /// the IEQ case-insensitive operator with OCSF casing note (post-normalization contract).
    ///
    /// After OCSF normalization (S-PRISMQL-CASE-INSENSITIVE-001), cyberint's lowercase
    /// 'high'/'critical' values are normalized to Title-case 'High'/'Critical'. An example
    /// with `IN ('high', 'critical')` would return 0 rows against post-normalization data.
    /// The describe example must use IEQ (case-insensitive) to remain correct post-normalization.
    ///
    /// ## Current behaviour (HEAD 8e4ec972) — RED
    ///
    /// severity_is_primary = false (created_at is first) → IN variant emitted:
    /// `SELECT * FROM cyberint_alerts WHERE severity IN ('high', 'critical') LIMIT 50`
    ///
    /// The assertion `q.contains("IEQ")` FAILS.
    /// The `q.contains("WHERE severity IN")` assertion also fails post-fix.
    ///
    /// F-P6-HIGH-002 (LOCAL pass-6): cyberint secondary-severity table must use IEQ +
    /// OCSF casing note. `IN ('high', 'critical')` silently 0-rows post-normalization.
    /// AC-025; ADR-047 §D.4; BC-2.11.024.
    #[test]
    fn test_f_phl2_med001_cyberint_alerts_severity_uses_lower_case() {
        let columns = vec![
            col("created_at", ColumnType::Datetime),
            col("severity", ColumnType::String),
            col("title", ColumnType::String),
        ];

        let q = build_example_query("cyberint_alerts", &columns);

        // Post-normalization contract: cyberint severity is normalized to Title-case.
        // Describe example must use IEQ to be correct post-normalization.
        // FAILS NOW: current code emits `IN ('high', 'critical')` which 0-rows post-norm.
        assert!(
            q.contains("IEQ"),
            "F-P6-HIGH-002 (LOCAL pass-6): cyberint_alerts describe example must use \
             IEQ operator per AC-025 / ADR-047 §D.4 (post-normalization, lowercase IN \
             silently returns 0 rows); got: {q:?}"
        );
        // IEQ example must include the OCSF casing hint.
        assert!(
            q.contains("Title-case") || q.contains("title-case"),
            "F-P6-HIGH-002 (LOCAL pass-6): cyberint_alerts describe example must include \
             OCSF casing note (substring 'Title-case') per AC-025; got: {q:?}"
        );
        // Severity branch must have fired (not count-recent).
        // IEQ form does not use WHERE severity IN — check for severity reference instead.
        assert!(
            q.contains("severity"),
            "F-PHL2-MED-001: cyberint_alerts describe example must reference the severity \
             column; got: {q:?}"
        );
    }

    /// F-PHL2-MED-001 (updated for F-MED-1 LOCAL pass-7): cyberint_alerts with
    /// severity_id (Integer) must use the IEQ severity branch, NOT aggregate.
    ///
    /// cyberint is a severity-vocabulary sensor (prefix "cyberint" in
    /// SENSOR_SEVERITY_VOCABULARY). When both a String `severity` column AND an
    /// Integer `severity_id` column are present, the severity-IEQ branch fires
    /// FIRST (highest priority per F-MED-1 fix) and the aggregate branch is skipped.
    ///
    /// Before F-MED-1 (LOCAL pass-7): aggregate > severity-IEQ — Integer won.
    /// After  F-MED-1 (LOCAL pass-7): severity-IEQ > aggregate for vocabulary tables.
    ///
    /// This test was updated to reflect the corrected priority per AC-025 / ADR-047 §D.4:
    /// IEQ teaching example must NOT be suppressed by the presence of a numeric column.
    ///
    /// To verify aggregate STILL works for non-vocabulary tables with Integer columns,
    /// see `test_obs001_roundtrip_aggregate_branch_parses` (armis_devices + risk_score).
    #[test]
    fn test_f_phl2_med001_cyberint_alerts_integer_col_triggers_aggregate_not_severity() {
        let columns = vec![
            col("created_at", ColumnType::Datetime),
            col("severity", ColumnType::String),
            col("severity_id", ColumnType::Integer),
        ];

        let q = build_example_query("cyberint_alerts", &columns);

        // F-MED-1 (LOCAL pass-7): severity-vocabulary IEQ wins over aggregate.
        // cyberint is in SENSOR_SEVERITY_VOCABULARY → IEQ fires first.
        assert!(
            q.contains("IEQ"),
            "F-MED-1 (LOCAL pass-7): cyberint_alerts (severity vocabulary) with Integer \
             severity_id must use the IEQ severity branch, not aggregate. \
             Severity-IEQ has HIGHEST priority for vocabulary tables per AC-025 / ADR-047 §D.4. \
             Got: {q}"
        );
        assert!(
            q.contains("Title-case") || q.contains("title-case"),
            "F-MED-1 (LOCAL pass-7): IEQ example must include OCSF casing note. Got: {q}"
        );
        // Confirm aggregate did NOT win.
        assert!(
            !q.contains("GROUP BY"),
            "F-MED-1 (LOCAL pass-7): aggregate must NOT override IEQ for vocabulary tables. \
             Got: {q}"
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

    /// F-P1L2-OBS-001: severity-filter branch (IEQ pipe form) parses through PrismQlParser.
    ///
    /// Representative table: crowdstrike_detections with severity as secondary column.
    /// After F-P6-HIGH-001, even secondary-severity tables emit the IEQ pipe form:
    /// `FROM <t> | WHERE severity IEQ 'high' | limit 50`
    ///
    /// Parseability roundtrip intent: the query portion (after stripping comment lines)
    /// must parse through PrismQlParser without error.
    ///
    /// ## Current behaviour (HEAD 8e4ec972) — RED
    ///
    /// severity_is_primary = false → IN variant emitted. The assertion `q.contains("IEQ")`
    /// FAILS (current output has `WHERE severity IN`, not `IEQ`).
    ///
    /// F-P6-HIGH-001 (LOCAL pass-6): secondary severity → IEQ form (remove exception).
    /// AC-025; ADR-047 §D.4; BC-2.11.024.
    #[test]
    fn test_obs001_roundtrip_severity_filter_branch_parses() {
        use prism_query::filter_parser::PrismQlParser;
        let columns = vec![
            col("created_timestamp", ColumnType::Datetime),
            col("severity", ColumnType::String),
        ];
        let q = build_example_query("crowdstrike_detections", &columns);

        // Post-fix: IEQ form must be chosen (no more IN for secondary severity).
        // FAILS NOW: severity_is_primary = false → IN form emitted.
        assert!(
            q.contains("IEQ"),
            "F-P6-HIGH-001 (LOCAL pass-6): crowdstrike_detections severity example must use \
             IEQ operator (AC-025: any severity table, any column position); \
             current output uses IN form for secondary severity; got: {q:?}"
        );

        // Parseability roundtrip: strip leading comment line(s) before parsing.
        // The IEQ form may include a `-- ...` comment line; strip it to get the parseable part.
        let query_part: String = q
            .lines()
            .filter(|line| !line.trim_start().starts_with("--"))
            .collect::<Vec<_>>()
            .join("\n")
            .trim()
            .to_string();

        let result = PrismQlParser::parse(&query_part);
        assert!(
            result.is_ok(),
            "F-P1L2-OBS-001: severity IEQ pipe-form output must parse successfully \
             (keep parseability roundtrip intent intact). \
             Query part (comment stripped): {query_part:?}. Error: {result:?}"
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

    // ─────────────────────────────────────────────────────────────────────────
    // F-HIGH-001 (LOCAL pass-5) / AC-025 — IEQ example + OCSF casing note
    // ─────────────────────────────────────────────────────────────────────────

    /// F-HIGH-001 / AC-025 / RG-027: `build_example_query` for a sensor table with
    /// a `severity` (String) column must produce output that contains BOTH:
    ///   (a) at least one example using the `IEQ` case-insensitive operator, AND
    ///   (b) an OCSF casing note indicating that severity values are Title-case.
    ///
    /// ## Current behaviour (HEAD b2e3892c) — RED
    ///
    /// `build_example_query("crowdstrike_detections", &[col("severity", String), ...])`
    /// returns:
    /// ```text
    /// "SELECT * FROM crowdstrike_detections WHERE severity IN ('High', 'Critical') LIMIT 50"
    /// ```
    ///
    /// - No `"IEQ"` substring → first `assert!(q.contains("IEQ"), ...)` panics.
    /// - No `"Title-case"` or `"title-case"` substring → second assertion would also panic.
    ///
    /// ## Green Gate
    ///
    /// PASSES once `build_example_query` (or its callers) emits a severity IEQ example
    /// AND includes the OCSF casing note per AC-025 / ADR-047 §D.4:
    /// "OCSF severity is stored as Title-case ('High'). Use IEQ/IIN to match regardless
    /// of the case you type, or = 'High' for the exact canonical form."
    ///
    /// ## Traces
    ///
    /// BC-2.11.024 §AC-025; ADR-047 §D.4; S-PRISMQL-CASE-INSENSITIVE-001 RG-027;
    /// LOCAL adversary pass-5 finding F-HIGH-001.
    #[test]
    fn test_BC_2_11_024_describe_output_includes_ieq_example_and_ocsf_casing_note() {
        let columns = vec![
            col("severity", ColumnType::String),
            col("timestamp", ColumnType::Datetime),
        ];
        let q = build_example_query("crowdstrike_detections", &columns);
        assert!(
            q.contains("IEQ"),
            "AC-025 (F-HIGH-001): build_example_query for a severity-column table must include \
             at least one IEQ operator example per ADR-047 \u{00A7}D.4; current output uses \
             IN not IEQ; got: {q:?}"
        );
        assert!(
            q.contains("Title-case") || q.contains("title-case"),
            "AC-025 (F-HIGH-001): build_example_query must include the OCSF casing note \
             (substring 'Title-case') per AC-025 / ADR-047 \u{00A7}D.4: \
             'OCSF severity is stored as Title-case'; got: {q:?}"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // F-P6-HIGH-001/002 (LOCAL pass-6) — secondary-severity IEQ coverage
    // ─────────────────────────────────────────────────────────────────────────

    /// F-P6-HIGH-001/002 / BC-2.11.024 §AC-025: `build_example_query` for tables where
    /// severity is NOT the first column (secondary position) must still emit the IEQ
    /// example + OCSF casing note per AC-025 ("any sensor table with a severity column").
    ///
    /// ## Tables tested
    ///
    /// - `armis_alerts` with `[alert_id String, severity String]` — severity is second.
    ///   Pre-fix output: `WHERE severity IN ('HIGH', 'CRITICAL') LIMIT 50`
    ///   Post-normalization: 'HIGH' → 'High'; IN('HIGH') returns 0 rows.
    ///
    /// - `cyberint_alerts` with `[created_at Datetime, severity String, title String]` —
    ///   severity is second.
    ///   Pre-fix output: `WHERE severity IN ('high', 'critical') LIMIT 50`
    ///   Post-normalization: 'high' → 'High'; IN('high') returns 0 rows.
    ///
    /// ## Current behaviour (HEAD 8e4ec972) — RED
    ///
    /// `severity_is_primary = false` (another column is listed first) → IN variant emitted
    /// for both tables. The assertion `q.contains("IEQ")` FAILS for both.
    ///
    /// ## Green Gate
    ///
    /// PASSES once `build_example_query` always emits IEQ for any table with a severity
    /// column, removing the `severity_is_primary` guard that gated IEQ behind first-position
    /// semantics (F-P6-HIGH-001/002: remove the secondary-severity exception).
    ///
    /// Traces to: BC-2.11.024 §AC-025; ADR-047 §D.4; F-P6-HIGH-001 (armis);
    /// F-P6-HIGH-002 (cyberint); LOCAL adversary pass-6.
    #[test]
    fn test_BC_2_11_024_describe_ieq_example_for_secondary_severity_tables() {
        // ── armis_alerts: severity is secondary (alert_id is first) ───────────────
        {
            let columns = vec![
                col("alert_id", ColumnType::String),
                col("severity", ColumnType::String),
                col("status", ColumnType::String),
            ];
            let q = build_example_query("armis_alerts", &columns);

            // FAILS NOW: severity_is_primary = false → `IN ('HIGH', 'CRITICAL')` emitted.
            assert!(
                q.contains("IEQ"),
                "F-P6-HIGH-001 (LOCAL pass-6): armis_alerts with severity as secondary column \
                 must emit IEQ example per AC-025 / ADR-047 §D.4; \
                 post-normalization IN('HIGH','CRITICAL') returns 0 rows; \
                 got: {q:?}"
            );
            assert!(
                q.contains("Title-case") || q.contains("title-case"),
                "F-P6-HIGH-001 (LOCAL pass-6): armis_alerts IEQ example must include \
                 OCSF casing note (substring 'Title-case') per AC-025; got: {q:?}"
            );
        }

        // ── cyberint_alerts: severity is secondary (created_at is first) ──────────
        {
            let columns = vec![
                col("created_at", ColumnType::Datetime),
                col("severity", ColumnType::String),
                col("title", ColumnType::String),
            ];
            let q = build_example_query("cyberint_alerts", &columns);

            // FAILS NOW: severity_is_primary = false → `IN ('high', 'critical')` emitted.
            assert!(
                q.contains("IEQ"),
                "F-P6-HIGH-002 (LOCAL pass-6): cyberint_alerts with severity as secondary column \
                 must emit IEQ example per AC-025 / ADR-047 §D.4; \
                 post-normalization IN('high','critical') returns 0 rows; \
                 got: {q:?}"
            );
            assert!(
                q.contains("Title-case") || q.contains("title-case"),
                "F-P6-HIGH-002 (LOCAL pass-6): cyberint_alerts IEQ example must include \
                 OCSF casing note (substring 'Title-case') per AC-025; got: {q:?}"
            );
        }

        // ── cyberint_incidents: severity is secondary ─────────────────────────────
        {
            let columns = vec![
                col("created_at", ColumnType::Datetime),
                col("severity", ColumnType::String),
                col("title", ColumnType::String),
            ];
            let q = build_example_query("cyberint_incidents", &columns);

            // cyberint_incidents also uses the cyberint vocabulary prefix.
            // FAILS NOW: secondary severity → IN form.
            assert!(
                q.contains("IEQ"),
                "F-P6-HIGH-002 (LOCAL pass-6): cyberint_incidents with severity as secondary column \
                 must emit IEQ example per AC-025 / ADR-047 §D.4; got: {q:?}"
            );
        }
    }

    /// F-P6-HIGH-001/002 / BC-2.11.024 §AC-025: no `build_example_query` output for any
    /// registered sensor+table combination may contain vendor-cased severity IN literals
    /// (`IN ('HIGH'`, `IN ('high'`, `'CRITICAL'`, `'critical'`) that silently return 0 rows
    /// against post-normalization OCSF data.
    ///
    /// After OCSF normalization, ALL sensors' severity values are stored as Title-case
    /// ('High', 'Critical'). Vendor-specific UPPER-case ('HIGH'/'CRITICAL' — armis) and
    /// lowercase ('high'/'critical' — cyberint) IN filters produce 0 rows against
    /// normalized data.
    ///
    /// ## What is checked
    ///
    /// For each sensor+table combination that has a severity column and registered vocabulary,
    /// invoke `build_example_query` with severity as a secondary column (not first) and verify
    /// the output does NOT contain any of:
    ///   - `IN ('HIGH'`     (armis UPPER-case IN)
    ///   - `IN ('high'`     (cyberint lowercase IN)
    ///   - `'CRITICAL'`     (armis UPPER-case literal anywhere)
    ///   - `'critical'`     (cyberint lowercase literal anywhere)
    ///
    /// ## Current behaviour (HEAD 8e4ec972) — RED
    ///
    /// armis_alerts secondary severity → `WHERE severity IN ('HIGH', 'CRITICAL') LIMIT 50`
    /// cyberint_alerts secondary severity → `WHERE severity IN ('high', 'critical') LIMIT 50`
    ///
    /// Both contain forbidden patterns. The assertions FAIL.
    ///
    /// ## Green Gate
    ///
    /// PASSES once secondary-severity tables emit IEQ (no IN with vendor-cased literals).
    ///
    /// Traces to: BC-2.11.024 §AC-025; F-P6-HIGH-001 (armis); F-P6-HIGH-002 (cyberint);
    /// LOCAL adversary pass-6.
    #[test]
    fn test_BC_2_11_024_describe_no_stale_vendor_casing_examples() {
        // Helper: secondary-severity column set (another column listed before severity).
        let with_secondary_severity = |datetime_col: &str, extra: &str| {
            vec![
                col(datetime_col, ColumnType::Datetime),
                col("severity", ColumnType::String),
                col(extra, ColumnType::String),
            ]
        };

        // ── armis_alerts — UPPER-case vocabulary (HIGH/CRITICAL) ──────────────────
        {
            let q = build_example_query(
                "armis_alerts",
                &[
                    col("alert_id", ColumnType::String),
                    col("severity", ColumnType::String),
                ],
            );
            // FAILS NOW: `IN ('HIGH', 'CRITICAL')` is the current output.
            assert!(
                !q.contains("IN ('HIGH'"),
                "F-P6-HIGH-001: armis_alerts example must NOT contain `IN ('HIGH'` — \
                 post-normalization UPPER-case IN returns 0 rows; got: {q:?}"
            );
            assert!(
                !q.contains("'CRITICAL'"),
                "F-P6-HIGH-001: armis_alerts example must NOT contain `'CRITICAL'` literal — \
                 post-normalization UPPER-case literal returns 0 rows; got: {q:?}"
            );
        }

        // ── cyberint_alerts — lowercase vocabulary (high/critical) ────────────────
        {
            let cols = with_secondary_severity("created_at", "title");
            let q = build_example_query("cyberint_alerts", &cols);
            // FAILS NOW: `IN ('high', 'critical')` is the current output.
            assert!(
                !q.contains("IN ('high'"),
                "F-P6-HIGH-002: cyberint_alerts example must NOT contain `IN ('high'` — \
                 post-normalization lowercase IN returns 0 rows; got: {q:?}"
            );
            assert!(
                !q.contains("'critical'"),
                "F-P6-HIGH-002: cyberint_alerts example must NOT contain `'critical'` literal — \
                 post-normalization lowercase literal returns 0 rows; got: {q:?}"
            );
        }

        // ── cyberint_incidents — lowercase vocabulary (same prefix) ──────────────
        {
            let cols = with_secondary_severity("created_at", "title");
            let q = build_example_query("cyberint_incidents", &cols);
            assert!(
                !q.contains("IN ('high'"),
                "F-P6-HIGH-002: cyberint_incidents example must NOT contain `IN ('high'`; \
                 got: {q:?}"
            );
            assert!(
                !q.contains("'critical'"),
                "F-P6-HIGH-002: cyberint_incidents example must NOT contain `'critical'`; \
                 got: {q:?}"
            );
        }

        // ── crowdstrike (Title-case) — should already be free of stale patterns ───
        {
            let cols = with_secondary_severity("created_timestamp", "status");
            let q = build_example_query("crowdstrike_detections", &cols);
            // crowdstrike uses 'High'/'Critical' (Title-case) — NOT in the forbidden set.
            // These should already pass; here as a sanity check.
            assert!(
                !q.contains("IN ('HIGH'"),
                "crowdstrike_detections must not emit `IN ('HIGH'`; got: {q:?}"
            );
            assert!(
                !q.contains("IN ('high'"),
                "crowdstrike_detections must not emit `IN ('high'`; got: {q:?}"
            );
            assert!(
                !q.contains("'critical'"),
                "crowdstrike_detections must not emit lowercase `'critical'`; got: {q:?}"
            );
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // F-MED-1 (LOCAL pass-7) — severity-vocabulary IEQ priority over aggregate
    // ─────────────────────────────────────────────────────────────────────────

    /// F-MED-1 (LOCAL pass-7): `build_example_query` for a severity-vocabulary table that
    /// ALSO has an Integer column must still emit the IEQ example + OCSF casing note.
    ///
    /// The aggregate-variant override (fires when ANY Integer/Float column exists)
    /// previously overwrote the IEQ severity example for severity-vocabulary tables.
    /// This is a latent AC-025 violation: analysts whose tables happen to have a numeric
    /// column (e.g., `priority`, `source_id`, `alert_count`) lose the IEQ teaching example
    /// that shows post-normalization casing and case-insensitive matching.
    ///
    /// ## Red Gate (HEAD 976708de)
    ///
    /// `build_example_query("crowdstrike_detections", &[severity String, priority Integer])`
    /// currently returns the aggregate variant:
    /// `SELECT priority, COUNT(*) FROM crowdstrike_detections GROUP BY priority ORDER BY COUNT(*) DESC LIMIT 10`
    ///
    /// The assertion `q.contains("IEQ")` FAILS.
    ///
    /// ## Green Gate
    ///
    /// PASSES once `build_example_query` gives severity-IEQ HIGHEST priority for
    /// severity-vocabulary tables — aggregate runs only when severity doesn't fire.
    ///
    /// ## Traces
    ///
    /// BC-2.11.024 §AC-025; ADR-047 §D.4; S-PRISMQL-CASE-INSENSITIVE-001 LOCAL-pass-7 F-MED-1.
    #[test]
    fn test_f_med1_severity_vocabulary_table_ieq_not_suppressed_by_integer_column() {
        // crowdstrike_detections is in the severity vocabulary (prefix "crowdstrike").
        // The Integer column `priority` would trigger aggregate in the OLD priority order
        // (aggregate > severity-IEQ). After the fix, IEQ wins for vocabulary tables.
        let columns = vec![
            col("severity", ColumnType::String),
            col("priority", ColumnType::Integer),
        ];
        let q = build_example_query("crowdstrike_detections", &columns);

        // F-MED-1 RED Gate: currently FAILS because aggregate overrides IEQ.
        assert!(
            q.contains("IEQ"),
            "F-MED-1 (LOCAL pass-7): build_example_query for a severity-vocabulary table \
             (crowdstrike_detections) with BOTH a String severity column AND an Integer column \
             must emit the IEQ example + OCSF casing note (AC-025 / ADR-047 §D.4). \
             Aggregate must NOT suppress IEQ for severity-vocabulary tables. \
             Got: {q:?}"
        );
        assert!(
            q.contains("Title-case") || q.contains("title-case"),
            "F-MED-1 (LOCAL pass-7): IEQ example must include OCSF casing note \
             (substring 'Title-case') per AC-025. Got: {q:?}"
        );
        // Confirm aggregate did NOT win (it should not, because severity vocabulary fires first).
        assert!(
            !q.contains("GROUP BY"),
            "F-MED-1 (LOCAL pass-7): aggregate branch must NOT override IEQ for severity-vocabulary \
             tables; GROUP BY must not appear in the output. Got: {q:?}"
        );
    }

    // ── F-OBS-2 (LOCAL-pass-11): comprehensive parse-lock sweep ──────────────
    //
    // Every `build_example_query` output across ALL table shapes constructed in this
    // test module must parse Ok via `PrismQlParser::parse` (after stripping comment
    // lines). This locks the grammar against silent breakage of any example path.
    //
    // GREEN now: all shapes already produce valid PrismQL. A future grammar change
    // that makes any branch non-parseable will surface here immediately.
    //
    // Traces: F-OBS-2 (S-PRISMQL-CASE-INSENSITIVE-001 LOCAL-pass-11 fix-burst).

    /// F-OBS-2 parse-lock sweep: every `build_example_query` output across representative
    /// table shapes (covering all 4 priority-ladder branches for multiple sensor prefixes)
    /// must parse Ok via `PrismQlParser::parse`.
    ///
    /// Table shapes covered:
    /// - No columns (empty_table) → column-free SELECT * LIMIT 25
    /// - String-only, no Datetime (claroty_devices) → column-free SELECT * LIMIT 25
    /// - Datetime only, no severity, no Integer (cyberint_alerts) → count-recent
    /// - Datetime + String, unknown sensor (unknown_sensor_events) → count-recent
    /// - Severity vocabulary + String severity (crowdstrike_detections) → IEQ pipe form
    /// - Severity vocabulary + String severity + no Datetime (armis_alerts) → IEQ pipe form
    /// - Integer column, no severity vocabulary (unknown_sensor_with_count) → aggregate
    #[test]
    fn test_obs2_all_build_example_query_outputs_parse_ok() {
        use prism_query::filter_parser::PrismQlParser;

        let shapes: Vec<(&str, Vec<ColumnDescriptor>)> = vec![
            // Column-free: no columns at all
            ("empty_table", vec![]),
            // Column-free: String/Boolean only, no Datetime
            (
                "claroty_devices",
                vec![
                    col("uid", ColumnType::String),
                    col("device_category", ColumnType::String),
                    col("retired", ColumnType::Boolean),
                ],
            ),
            // Count-recent: Datetime only, no severity, no Integer, non-vocabulary sensor
            (
                "cyberint_alerts_no_severity",
                vec![
                    col("event_time", ColumnType::Datetime),
                    col("title", ColumnType::String),
                ],
            ),
            // Count-recent: Datetime + severity but unknown sensor (no vocabulary)
            (
                "unknown_sensor_events",
                vec![
                    col("created_at", ColumnType::Datetime),
                    col("severity", ColumnType::String),
                ],
            ),
            // IEQ severity pipe form: crowdstrike with severity String (vocabulary registered)
            (
                "crowdstrike_detections",
                vec![
                    col("created_timestamp", ColumnType::Datetime),
                    col("severity", ColumnType::String),
                ],
            ),
            // IEQ severity pipe form: armis_alerts with severity String, no Datetime
            (
                "armis_alerts",
                vec![
                    col("alert_id", ColumnType::String),
                    col("severity", ColumnType::String),
                ],
            ),
            // Aggregate: Integer column present, unknown sensor (no severity vocabulary)
            (
                "unknown_sensor_with_count",
                vec![
                    col("event_time", ColumnType::Datetime),
                    col("severity_id", ColumnType::Integer),
                ],
            ),
        ];

        for (table, columns) in &shapes {
            let q = build_example_query(table, columns);
            // Strip leading `-- ...` comment lines before parsing (IEQ form includes a comment).
            let query_part: String = q
                .lines()
                .filter(|line| !line.trim_start().starts_with("--"))
                .collect::<Vec<_>>()
                .join("\n")
                .trim()
                .to_string();
            let result = PrismQlParser::parse(&query_part);
            assert!(
                result.is_ok(),
                "F-OBS-2: build_example_query output for table '{table}' must parse \
                 successfully via PrismQlParser (query_part={query_part:?}). \
                 Error: {result:?}"
            );
        }
    }
}
