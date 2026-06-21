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

use rmcp::model::{CallToolResult, Content};
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
    /// The qualified table name (e.g., `"crowdstrike.alerts"`).
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
    let pql_hints = build_pql_hints(org_slug.as_str(), &tables, org_registry.as_deref());

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

    let mut json = serde_json::to_string(&response).map_err(|e| {
        rmcp::model::ErrorData::internal_error(
            format!("E-MCP-500: failed to serialize prism_describe response: {e}"),
            None,
        )
    })?;

    // Append audit_warning to JSON if needed (DI-004 fail-open).
    if audit_warning {
        // Inject _meta.audit_warning by deserializing to Value, inserting, re-serializing.
        if let Ok(mut val) = serde_json::from_str::<serde_json::Value>(&json) {
            if let Some(obj) = val.as_object_mut() {
                let mut meta = serde_json::Map::new();
                meta.insert("audit_warning".to_string(), serde_json::Value::Bool(true));
                obj.insert("_meta".to_string(), serde_json::Value::Object(meta));
            }
            if let Ok(augmented) = serde_json::to_string(&val) {
                json = augmented;
            }
        }
    }

    Ok(CallToolResult::success(vec![Content::text(json)]))
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
                        let example_query = build_example_query(&table.table_name, &columns);
                        // BC-2.10.012 sensor_type fix: derive from the sensor identity
                        // (sensor_id from the resolved spec), NOT from client_id.
                        TableDescriptor {
                            name: table.table_name.clone(),
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

            let example_query = build_example_query(&table.table_name, &columns);

            TableDescriptor {
                name: table.table_name.clone(),
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
fn build_pql_hints(
    client_id: &str,
    tables: &[TableDescriptor],
    org_registry: Option<&prism_core::OrgRegistry>,
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
        vec![format!(
            "No sensor tables are available for client '{client_id}'. \
             The client may not have any sensor overlays configured."
        )]
    } else {
        vec![
            format!(
                "Use 'SELECT * FROM <table> LIMIT 25' to query any of the {} table(s) above.",
                tables.len()
            ),
            "Consult prismql://reference for full PQL grammar and operator reference.".to_string(),
        ]
    }
}

/// Build an auto-generated example PQL query for a table.
///
/// Always includes a count-recent fallback. Adds severity-filter variant when a
/// `severity` column is present. Adds aggregate variant when an aggregatable column
/// (Integer/Float) is present.
pub fn build_example_query(table_name: &str, columns: &[ColumnDescriptor]) -> String {
    use prism_core::column::ColumnType;

    // Count-recent fallback (always present, EC-002 zero-column case).
    // BC-2.10.012 canonical: SELECT COUNT(*) FROM <t> WHERE timestamp > NOW() - INTERVAL '1h'
    let mut query =
        format!("SELECT COUNT(*) FROM {table_name} WHERE timestamp > NOW() - INTERVAL '1h'");

    // Severity filter variant when a severity column is present.
    // BC-2.10.012 canonical: SELECT * FROM <t> WHERE severity IN ('high', 'critical') LIMIT 50
    let has_severity = columns.iter().any(|c| c.name == "severity");
    if has_severity {
        query =
            format!("SELECT * FROM {table_name} WHERE severity IN ('high', 'critical') LIMIT 50");
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
