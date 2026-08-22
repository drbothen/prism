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

use prism_spec_engine::column_mapping::ocsf_field_to_arrow_name;
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
    ///
    /// BC-2.10.012: this field contains ONLY parseable PQL — no `--` comment lines.
    /// The OCSF casing note (previously embedded as a `-- ...` comment) moved to
    /// `example_note` (F-MED-002, LOCAL adversary pass-15).
    pub example_query: String,
    /// Optional contextual note accompanying the example query (BC-2.10.012).
    ///
    /// `Some(...)` for severity-vocabulary tables: explains that OCSF normalises
    /// severity to Title-case and that IEQ is the correct case-insensitive operator.
    /// `None` for all other tables (column-free, count-recent, aggregate shapes).
    ///
    /// F-MED-002 (LOCAL adversary pass-15): the OCSF casing note was previously
    /// embedded as a `-- ...` comment in `example_query`.  PrismQL does not support
    /// `--` comment syntax, so the note was moved here and `example_query` now carries
    /// only the bare parseable PQL.
    #[serde(default)]
    pub example_note: Option<String>,
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
        // BC-2.10.012 §Audit: emit audit BEFORE returning error (fail-open DI-004).
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

    // BC-2.10.012 §pql_hints AC-CAT2: resolve infusion_registry for Category-2 hint.
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

/// Build OCSF-mode ColumnDescriptors for a table when `ocsf_column_naming == true`.
///
/// Partitions columns into Tier-1 (ocsf_field == Some) and Tier-2 (ocsf_field == None):
/// - Tier-1 → ColumnDescriptor with name = ocsf_field_to_arrow_name(ocsf_field),
///   description = Some(ocsf_field), nullable = true
/// - Tier-2 → suppressed (not emitted individually)
/// - If any Tier-2 exist → ONE `raw_extensions` ColumnDescriptor (col_type=Json)
///   whose description enumerates the Tier-2 source column names
/// - Always appends `class_uid` (Integer, nullable=false) and `_sensor` (String,
///   nullable=false) as the last two entries (OQ-003, AC-015, RG-028).
///
/// ADR-058 §G; BC-2.16.003 EC-016-013-028/029.
fn build_ocsf_column_descriptors(
    table: &prism_spec_engine::spec_parser::TableSpec,
) -> Vec<ColumnDescriptor> {
    let tier2_names: Vec<&str> = table
        .columns
        .iter()
        .filter(|col| col.ocsf_field.is_none())
        .map(|col| col.name.as_str())
        .collect();

    let mut descriptors: Vec<ColumnDescriptor> = table
        .columns
        .iter()
        .filter(|col| col.ocsf_field.is_some())
        .map(|col| {
            let ocsf_field = col.ocsf_field.as_deref().unwrap();
            ColumnDescriptor {
                name: ocsf_field_to_arrow_name(ocsf_field),
                col_type: col.column_type.clone(),
                description: Some(ocsf_field.to_string()),
                nullable: true,
            }
        })
        .collect();

    if !tier2_names.is_empty() {
        let desc = format!(
            "JSON object containing un-mapped source columns: {}",
            tier2_names.join(", ")
        );
        descriptors.push(ColumnDescriptor {
            name: "raw_extensions".to_string(),
            col_type: prism_core::column::ColumnType::Json,
            description: Some(desc),
            nullable: true,
        });
    }

    // OQ-003 (AC-015): synthesize class_uid and _sensor as the last two descriptors
    // so LLM agents can filter on `WHERE class_uid = 3004` and `WHERE _sensor = 'claroty'`.
    // AC-015 canonical descriptions from ADR-058 §G v2.28 / BC-2.16.003 v1.23 / story AC-015 v1.48.
    descriptors.push(ColumnDescriptor {
        name: "class_uid".to_string(),
        col_type: prism_core::column::ColumnType::Integer,
        description: Some(
            "OCSF event class identifier derived from sensor TOML ocsf_class. \
             Example: 3004 for entity_management (audit_logs), \
             2004 for detection_finding (alerts, device_alert_relations), \
             5001 for inventory_info (devices)."
                .to_string(),
        ),
        nullable: false,
    });
    descriptors.push(ColumnDescriptor {
        name: "_sensor".to_string(),
        col_type: prism_core::column::ColumnType::String,
        description: Some("Sensor identifier. Value: <sensor_id> (e.g., 'claroty').".to_string()),
        nullable: false,
    });

    descriptors
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
                        // ADR-058 §G: when ocsf_column_naming is true, emit Tier-1/Tier-2
                        // ColumnDescriptor model (AC-006, RG-007/RG-025/RG-028).
                        let columns: Vec<ColumnDescriptor> = if spec.ocsf_column_naming {
                            build_ocsf_column_descriptors(table)
                        } else {
                            table
                                .columns
                                .iter()
                                .map(|col| ColumnDescriptor {
                                    name: col.name.clone(),
                                    col_type: col.column_type.clone(),
                                    description: col.ocsf_field.clone(),
                                    nullable: true,
                                })
                                .collect()
                        };
                        // BC-2.10.012 AUDIT-001: table name must be sensor-prefixed so that
                        // AI agents build valid `FROM crowdstrike_alerts | ...` queries, NOT
                        // bare `FROM alerts | ...` (which silently routes to E-SENSOR-030).
                        let prefixed_name = format!("{}_{}", sensor_id.as_ref(), table.table_name);
                        // BC-2.10.012 F-MED-002: use build_example_with_note to populate
                        // both example_query (pure PQL) and example_note (OCSF casing hint).
                        let (example_query, example_note) =
                            build_example_with_note(&prefixed_name, &columns);
                        // BC-2.10.012 sensor_type fix: derive from the sensor identity
                        // (sensor_id from the resolved spec), NOT from client_id.
                        TableDescriptor {
                            name: prefixed_name,
                            sensor_type: sensor_id.as_ref().to_string(),
                            description: table.ocsf_class.clone(),
                            columns,
                            example_query,
                            example_note,
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
            // ADR-058 §G: when ocsf_column_naming is true, emit Tier-1/Tier-2
            // ColumnDescriptor model (AC-006, RG-007/RG-025/RG-028).
            let columns: Vec<ColumnDescriptor> = if sensor_spec.ocsf_column_naming {
                build_ocsf_column_descriptors(table)
            } else {
                table
                    .columns
                    .iter()
                    .map(|col| ColumnDescriptor {
                        name: col.name.clone(),
                        col_type: col.column_type.clone(),
                        description: col.ocsf_field.clone(),
                        nullable: true,
                    })
                    .collect()
            };

            // BC-2.10.012 AUDIT-001: table name must be sensor-prefixed so that AI agents
            // build valid `FROM crowdstrike_alerts | ...` queries (not bare `FROM alerts`).
            let prefixed_name = format!("{}_{}", sensor_spec.sensor_id, table.table_name);
            // BC-2.10.012 F-MED-002: use build_example_with_note to populate both
            // example_query (pure PQL) and example_note (OCSF casing hint).
            let (example_query, example_note) = build_example_with_note(&prefixed_name, &columns);

            TableDescriptor {
                name: prefixed_name,
                // BC-2.10.012 sensor_type fix: derive from the sensor spec's sensor_id,
                // NOT from client_id (in single-tenant mode they happen to be the same,
                // but the canonical source is the spec's sensor_id).
                sensor_type: sensor_spec.sensor_id.clone(),
                description: table.ocsf_class.clone(),
                columns,
                example_query,
                example_note,
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
/// # Category-2 enrichment-discovery hint (BC-2.10.012 §pql_hints, AC-CAT2)
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

    // Category-2: enrichment-discovery hint (BC-2.10.012 §pql_hints AC-CAT2).
    // Only emitted when N ≥ 1 tables (suppressed for zero-table case above).
    let cat2_hint = build_enrichment_hint(infusion_registry);
    hints.push(cat2_hint);

    hints
}

/// Build the Category-2 enrichment-discovery hint (BC-2.10.012 §pql_hints AC-CAT2).
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
/// (PO-confirmed: BC-2.10.012 / AC-CAT2 does not require a new catalog row).
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
/// # F-L2-CRIT-001 fix (S-DEMO-FIDELITY-REMEDIATION-001) + OBS-2 (LOCAL pass-18)
///
/// The previous implementation hardcoded lowercase `'high'`/`'critical'` in the
/// severity filter variant. CrowdStrike DTU emits Title-case (`High`/`Critical`) and
/// Armis DTU emits UPPER-case (`HIGH`/`CRITICAL`). Lowercase literals match no rows.
///
/// Initial fix: derive severity literals from a sensor prefix allowlist
/// (`SENSOR_SEVERITY_VOCABULARY`).  OBS-2 (LOCAL adversary pass-18) identified that
/// this allowlist gate would silently omit IEQ examples for future sensors not yet
/// registered.  Since BC-2.02.013 PRIMARY normalization canonicalizes ALL sensors'
/// severity to OCSF Title-case before DataFusion materialization, IEQ is correct for
/// every table that exposes a `severity` String column — no per-sensor allowlist needed.
///
/// Final fix: column-presence gate — if the column list contains a String `severity`
/// column, the IEQ example fires unconditionally.
///
/// # Variant selection (priority: severity-IEQ > aggregate > count-recent/simple)
///
/// | Condition                                              | Query emitted |
/// |--------------------------------------------------------|---------------|
/// | severity String column present                         | IEQ pipe form (AC-025 / ADR-047 §D.4) |
/// | Integer/Float column present (no severity column)      | GROUP BY aggregate |
/// | Datetime column found (no above)                       | COUNT(*) WHERE <dt_col> > NOW() - INTERVAL '1h' |
/// | No datetime column (no above)                          | SELECT * FROM <t> LIMIT 25 |
///
/// All tables with a severity String column emit the IEQ form regardless of sensor
/// prefix or column position (AC-025 / ADR-047 §D.4 / OBS-2 LOCAL pass-18).
/// Vendor-cased IN literals were removed because post-normalization they silently return
/// 0 rows (BC-2.02.013 PRIMARY normalization canonicalizes to Title-case).
///
/// BC-2.10.012 / AUDIT-001 / AUDIT-004; S-DEMO-FIDELITY-REMEDIATION-001 CRIT-1 + F-L2-CRIT-001.
///
/// BC-2.10.012 note: returns pure PQL with NO `--` comment lines. The OCSF
/// casing note (previously embedded as a leading `-- ...` comment) moved to
/// `build_example_with_note` (F-MED-002, LOCAL adversary pass-15). Use
/// `build_example_with_note` when both the query and the note are needed.
pub fn build_example_query(table_name: &str, columns: &[ColumnDescriptor]) -> String {
    build_example_with_note(table_name, columns).0
}

/// Build an auto-generated example PQL query AND an optional contextual note.
///
/// Returns `(example_query, example_note)` where:
/// - `example_query` is always a parseable PQL string with NO `--` comment lines.
/// - `example_note` is `Some(...)` for any table with a `severity` String column
///   (contains the OCSF Title-case casing hint) and `None` for all other tables.
///
/// This is the authoritative implementation; `build_example_query` is a thin wrapper
/// that discards the note.
///
/// BC-2.10.012 §example_query + §example_note; F-MED-002 LOCAL adversary pass-15.
/// OBS-2 (LOCAL pass-18): gate changed from sensor-prefix allowlist to column-presence.
pub fn build_example_with_note(
    table_name: &str,
    columns: &[ColumnDescriptor],
) -> (String, Option<String>) {
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

    // Severity IEQ variant fires for ANY table whose column list includes a String
    // `severity` column (column-presence gate, OBS-2 LOCAL pass-18).
    //
    // F-L2-CRIT-001 / F-P6-HIGH-001/002 / AC-025 (S-PRISMQL-CASE-INSENSITIVE-001):
    // ALL tables with a severity column emit the IEQ form regardless of column position
    // or sensor prefix. Vendor-cased IN literals (`IN ('HIGH', …)`, `IN ('high', …)`)
    // silently return 0 rows against post-normalization data because BC-2.02.013 PRIMARY
    // normalization (build_column_array) canonicalizes severity to OCSF Title-case before
    // DataFusion materialization (AC-025 / ADR-047 §D.4).
    //
    // OBS-2 (LOCAL pass-18): removed the SENSOR_SEVERITY_VOCABULARY allowlist gate.
    // BC-2.02.013 normalizes ALL sensors' severity to OCSF Title-case, so IEQ is correct
    // for every table with a severity column — no per-sensor registration needed.
    //
    // F-MED-1 (S-PRISMQL-CASE-INSENSITIVE-001 LOCAL pass-7 BC-2.11.024):
    // Severity-IEQ is the HIGHEST priority variant. The aggregate variant runs only when
    // severity-IEQ does not fire (no severity column). This prevents the aggregate branch
    // from silently suppressing the IEQ teaching example when a numeric column is present.
    // AC-025 / ADR-047 §D.4 mandate the IEQ example for all severity-column tables.
    // F-P20-LOW-001 (LOCAL pass-20): gate checks BOTH name AND type.  A column named
    // `severity` with ColumnType::Integer (ordinal severity) must NOT receive the IEQ
    // example — applying `IEQ 'high'` to an Integer column fails E-QUERY-002 type check.
    // Only ColumnType::String severity columns get the IEQ teaching example.
    let has_severity = columns
        .iter()
        .any(|c| c.name == "severity" && matches!(c.col_type, ColumnType::String));
    if has_severity {
        // AC-025 / ADR-047 §D.4: IEQ operator for ANY table with a severity String column.
        // F-MED-002 (LOCAL pass-15): the OCSF casing note moves out of `example_query`
        // into `example_note` so that `example_query` remains pure parseable PQL.
        // PrismQL does NOT support `--` comment syntax; embedding the note as a comment
        // caused PrismQlParser to reject the query (BC-2.10.012).
        query = format!("FROM {table_name} | where severity IEQ 'high' | limit 50");
        let note = "OCSF severity is stored as Title-case ('High'). \
                    Use IEQ/IIN to match regardless of the case you type, \
                    or = 'High' for the exact canonical form."
            .to_string();
        return (query, Some(note));
    }

    // Aggregate variant when an aggregatable column is present and severity-IEQ did not fire.
    // BC-2.10.012 canonical: SELECT <field>, COUNT(*) FROM <t> GROUP BY <field> ORDER BY COUNT(*) DESC LIMIT 10
    // Only runs when: no severity column.
    let agg_col = columns
        .iter()
        .find(|c| matches!(c.col_type, ColumnType::Integer | ColumnType::Float));
    if let Some(col) = agg_col {
        query = format!(
            "SELECT {col_name}, COUNT(*) FROM {table_name} GROUP BY {col_name} ORDER BY COUNT(*) DESC LIMIT 10",
            col_name = col.name
        );
    }

    (query, None)
}

#[cfg(test)]
mod build_example_query_tests {
    use super::{build_example_query, build_example_with_note, ColumnDescriptor};
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

        // F-MED-002 (LOCAL pass-15): use build_example_with_note to access example_note separately.
        // example_query is now pure PQL; OCSF casing note is in example_note (BC-2.10.012).
        let (q, note) = build_example_with_note("crowdstrike_detections", &columns);

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
        // F-MED-002 (LOCAL pass-15): casing note moved to example_note (BC-2.10.012).
        let note_str = note.as_deref().unwrap_or("");
        assert!(
            note_str.contains("Title-case") || note_str.contains("title-case"),
            "F-P6-HIGH-001 (LOCAL pass-6): example_note must include OCSF casing note \
             (substring 'Title-case') per AC-025 / ADR-047 §D.4 / BC-2.10.012 F-MED-002; \
             got note: {note_str:?}"
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

        // F-MED-002 (LOCAL pass-15): use build_example_with_note; OCSF note is now in example_note.
        let (q, note) = build_example_with_note("crowdstrike_detections", &columns);

        // Post-normalization contract: all severity values stored as Title-case.
        // Describe example must use IEQ so the analyst can match case-insensitively.
        // FAILS NOW: current code emits IN variant for secondary-severity tables.
        assert!(
            q.contains("IEQ"),
            "F-P6-HIGH-001 (LOCAL pass-6): crowdstrike_detections describe example must use \
             IEQ operator per AC-025 / ADR-047 §D.4 (any severity table, any column position); \
             got: {q:?}"
        );
        // F-MED-002 (LOCAL pass-15): OCSF casing note moved to example_note (BC-2.10.012).
        let note_str = note.as_deref().unwrap_or("");
        assert!(
            note_str.contains("Title-case") || note_str.contains("title-case"),
            "F-P6-HIGH-001 (LOCAL pass-6): example_note must include OCSF casing note \
             (substring 'Title-case') per AC-025 / BC-2.10.012 F-MED-002; \
             got note: {note_str:?}"
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

        // F-MED-002 (LOCAL pass-15): use build_example_with_note; OCSF note is now in example_note.
        let (q, note) = build_example_with_note("armis_alerts", &columns);

        // Post-normalization contract: use IEQ (case-insensitive), not IN with vendor casing.
        assert!(
            q.contains("IEQ"),
            "F-L2-CRIT-001 + F-P6-HIGH-001: armis_alerts severity describe example must use \
             IEQ operator; post-normalization IN('HIGH','CRITICAL') returns 0 rows. Got: {q}"
        );
        // F-MED-002 (LOCAL pass-15): OCSF casing note moved to example_note (BC-2.10.012).
        let note_str = note.as_deref().unwrap_or("");
        assert!(
            note_str.contains("Title-case") || note_str.contains("title-case"),
            "F-L2-CRIT-001 + F-P6-HIGH-001: example_note must include OCSF casing note \
             (substring 'Title-case') per AC-025 / ADR-047 §D.4 / BC-2.10.012 F-MED-002. \
             Got note: {note_str}"
        );
        // Must NOT use IN with vendor-cased literals that silently 0-row post-normalization.
        assert!(
            !q.contains("IN ('HIGH'"),
            "F-L2-CRIT-001 + F-P6-HIGH-001: armis_alerts must NOT use IN('HIGH',...); \
             post-normalization UPPER-case IN silently returns 0 rows. Got: {q}"
        );
    }

    /// OBS-2 (LOCAL pass-18): unknown sensor with a severity String column now emits
    /// the IEQ form — column-presence gate, not sensor-prefix allowlist.
    ///
    /// Before OBS-2 (F-L2-CRIT-001 approach): unknown sensors fell back to count-recent
    /// because they were absent from the `SENSOR_SEVERITY_VOCABULARY` allowlist.
    ///
    /// After OBS-2: the allowlist gate is removed.  BC-2.02.013 normalizes ALL sensors'
    /// severity to OCSF Title-case, so IEQ is universally correct for any table whose
    /// column list includes a String `severity` column.
    ///
    /// Load-bearing (TD-VSDD-059): reverting the gate to an allowlist check makes this
    /// test fail, catching the regression.
    #[test]
    fn test_f_l2_crit001_unknown_sensor_with_severity_gets_ieq() {
        let columns = vec![
            col("created_at", ColumnType::Datetime),
            col("severity", ColumnType::String),
        ];

        let (q, note) = build_example_with_note("unknown_sensor_events", &columns);

        // OBS-2: column-presence gate fires — IEQ form, not count-recent.
        assert!(
            q.contains("IEQ"),
            "OBS-2 (LOCAL pass-18): unknown sensor with severity column must emit IEQ \
             form (column-presence gate removes allowlist dependency). Got: {q}"
        );
        let note_str = note.as_deref().unwrap_or("");
        assert!(
            note_str.contains("Title-case") || note_str.contains("title-case"),
            "OBS-2 (LOCAL pass-18): example_note must contain the OCSF casing hint \
             for any severity-column table; got note: {note_str}"
        );
        // Confirm the result is pure parseable PQL (BC-2.10.012 pure-PQL invariant).
        assert!(
            !q.contains("--"),
            "OBS-2 (LOCAL pass-18): IEQ form must be pure PQL without '--' comments. Got: {q}"
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

        // F-MED-002 (LOCAL pass-15): use build_example_with_note; OCSF note is now in example_note.
        let (q, note) = build_example_with_note("cyberint_alerts", &columns);

        // Post-normalization contract: cyberint severity is normalized to Title-case.
        // Describe example must use IEQ to be correct post-normalization.
        // FAILS NOW: current code emits `IN ('high', 'critical')` which 0-rows post-norm.
        assert!(
            q.contains("IEQ"),
            "F-P6-HIGH-002 (LOCAL pass-6): cyberint_alerts describe example must use \
             IEQ operator per AC-025 / ADR-047 §D.4 (post-normalization, lowercase IN \
             silently returns 0 rows); got: {q:?}"
        );
        // F-MED-002 (LOCAL pass-15): OCSF casing note moved to example_note (BC-2.10.012).
        let note_str = note.as_deref().unwrap_or("");
        assert!(
            note_str.contains("Title-case") || note_str.contains("title-case"),
            "F-P6-HIGH-002 (LOCAL pass-6): example_note must include OCSF casing note \
             (substring 'Title-case') per AC-025 / BC-2.10.012 F-MED-002; \
             got note: {note_str:?}"
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

        // F-MED-002 (LOCAL pass-15): use build_example_with_note; OCSF note is now in example_note.
        let (q, note) = build_example_with_note("cyberint_alerts", &columns);

        // F-MED-1 (LOCAL pass-7): severity-vocabulary IEQ wins over aggregate.
        // cyberint is in SENSOR_SEVERITY_VOCABULARY → IEQ fires first.
        assert!(
            q.contains("IEQ"),
            "F-MED-1 (LOCAL pass-7): cyberint_alerts (severity vocabulary) with Integer \
             severity_id must use the IEQ severity branch, not aggregate. \
             Severity-IEQ has HIGHEST priority for vocabulary tables per AC-025 / ADR-047 §D.4. \
             Got: {q}"
        );
        // F-MED-002 (LOCAL pass-15): OCSF casing note moved to example_note (BC-2.10.012).
        let note_str = note.as_deref().unwrap_or("");
        assert!(
            note_str.contains("Title-case") || note_str.contains("title-case"),
            "F-MED-1 (LOCAL pass-7): example_note must include OCSF casing note \
             per BC-2.10.012 F-MED-002. Got note: {note_str}"
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
    /// `FROM <t> | where severity IEQ 'high' | limit 50`
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
        // F-MED-002 (LOCAL pass-15): use build_example_with_note; OCSF note is now in example_note.
        let (q, note) = build_example_with_note("crowdstrike_detections", &columns);
        assert!(
            q.contains("IEQ"),
            "AC-025 (F-HIGH-001): build_example_query for a severity-column table must include \
             at least one IEQ operator example per ADR-047 \u{00A7}D.4; current output uses \
             IN not IEQ; got: {q:?}"
        );
        // F-MED-002 (LOCAL pass-15): OCSF casing note moved to example_note (BC-2.10.012).
        let note_str = note.as_deref().unwrap_or("");
        assert!(
            note_str.contains("Title-case") || note_str.contains("title-case"),
            "AC-025 (F-HIGH-001): example_note must include the OCSF casing note \
             (substring 'Title-case') per AC-025 / ADR-047 \u{00A7}D.4 / BC-2.10.012 F-MED-002: \
             'OCSF severity is stored as Title-case'; got note: {note_str:?}"
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
            // F-MED-002 (LOCAL pass-15): use build_example_with_note; note is now in example_note.
            let (q, note) = build_example_with_note("armis_alerts", &columns);

            // FAILS NOW: severity_is_primary = false → `IN ('HIGH', 'CRITICAL')` emitted.
            assert!(
                q.contains("IEQ"),
                "F-P6-HIGH-001 (LOCAL pass-6): armis_alerts with severity as secondary column \
                 must emit IEQ example per AC-025 / ADR-047 §D.4; \
                 post-normalization IN('HIGH','CRITICAL') returns 0 rows; \
                 got: {q:?}"
            );
            // F-MED-002 (LOCAL pass-15): OCSF note moved to example_note (BC-2.10.012).
            let note_str = note.as_deref().unwrap_or("");
            assert!(
                note_str.contains("Title-case") || note_str.contains("title-case"),
                "F-P6-HIGH-001 (LOCAL pass-6): example_note must include OCSF casing note \
                 (substring 'Title-case') per AC-025 / BC-2.10.012 F-MED-002; \
                 got note: {note_str:?}"
            );
        }

        // ── cyberint_alerts: severity is secondary (created_at is first) ──────────
        {
            let columns = vec![
                col("created_at", ColumnType::Datetime),
                col("severity", ColumnType::String),
                col("title", ColumnType::String),
            ];
            // F-MED-002 (LOCAL pass-15): use build_example_with_note; note is now in example_note.
            let (q, note) = build_example_with_note("cyberint_alerts", &columns);

            // FAILS NOW: severity_is_primary = false → `IN ('high', 'critical')` emitted.
            assert!(
                q.contains("IEQ"),
                "F-P6-HIGH-002 (LOCAL pass-6): cyberint_alerts with severity as secondary column \
                 must emit IEQ example per AC-025 / ADR-047 §D.4; \
                 post-normalization IN('high','critical') returns 0 rows; \
                 got: {q:?}"
            );
            // F-MED-002 (LOCAL pass-15): OCSF note moved to example_note (BC-2.10.012).
            let note_str = note.as_deref().unwrap_or("");
            assert!(
                note_str.contains("Title-case") || note_str.contains("title-case"),
                "F-P6-HIGH-002 (LOCAL pass-6): example_note must include OCSF casing note \
                 (substring 'Title-case') per AC-025 / BC-2.10.012 F-MED-002; \
                 got note: {note_str:?}"
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
        // F-MED-002 (LOCAL pass-15): use build_example_with_note; OCSF note is now in example_note.
        let (q, note) = build_example_with_note("crowdstrike_detections", &columns);

        // F-MED-1 RED Gate: currently FAILS because aggregate overrides IEQ.
        assert!(
            q.contains("IEQ"),
            "F-MED-1 (LOCAL pass-7): build_example_query for a severity-vocabulary table \
             (crowdstrike_detections) with BOTH a String severity column AND an Integer column \
             must emit the IEQ example + OCSF casing note (AC-025 / ADR-047 §D.4). \
             Aggregate must NOT suppress IEQ for severity-vocabulary tables. \
             Got: {q:?}"
        );
        // F-MED-002 (LOCAL pass-15): OCSF casing note moved to example_note (BC-2.10.012).
        let note_str = note.as_deref().unwrap_or("");
        assert!(
            note_str.contains("Title-case") || note_str.contains("title-case"),
            "F-MED-1 (LOCAL pass-7): example_note must include OCSF casing note \
             (substring 'Title-case') per AC-025 / BC-2.10.012 F-MED-002. \
             Got note: {note_str:?}"
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
            // F-MED-002 (LOCAL-pass-15): parse the RAW output — NO comment stripping.
            // The `--` stripping that was here previously masked a defect: the IEQ
            // severity form was embedding a `-- OCSF ...` comment line that PrismQL
            // does not support.  This test now asserts raw parseability so that
            // `build_example_query` must emit pure PQL without any `--` comments.
            //
            // RED NOW (HEAD 065a1b60): severity shapes emit a leading `-- OCSF ...`
            // comment line; PrismQlParser rejects it → this test FAILS for
            // `crowdstrike_detections` and `armis_alerts` shapes.
            //
            // GREEN GATE: PASSES once `build_example_query` moves the comment text
            // into `example_note: Option<String>` (BC-2.10.012) and
            // `example_query` contains only the pipe-form PQL.
            //
            // Traces: F-MED-002 (LOCAL adversary pass-15); TD-VSDD-059-adjacent
            // (stripping filter was masking the defect).
            let result = PrismQlParser::parse(&q);
            assert!(
                result.is_ok(),
                "F-MED-002 (LOCAL-pass-15): build_example_query output for table '{table}' \
                 must parse successfully via PrismQlParser WITHOUT comment stripping; \
                 currently FAILS for severity-vocabulary shapes because `build_example_query` \
                 embeds a `-- OCSF ...` comment line that PrismQL does not support \
                 (raw output={q:?}). \
                 Error: {result:?}"
            );
        }
    }

    // ── F-MED-002 (LOCAL-pass-15): example_query pure PQL + example_note field ──
    //
    // BC-2.10.012 requires:
    //   (a) `example_query` MUST NOT contain `--` comment syntax.
    //   (b) `TableDescriptor` gains `example_note: Option<String>`.
    //       - Severity-vocabulary tables → `Some("OCSF severity is stored as Title-case
    //         ('High'). Use IEQ/IIN to match regardless of the case you type,
    //         or = 'High' for the exact canonical form.")`  [BC-2.10.012 canonical]
    //       - All other tables → `None`
    //
    // Current behaviour at HEAD (post-pass-17 fix):
    //   `build_example_query` for severity tables returns the bare pipe form:
    //   `"FROM crowdstrike_detections | where severity IEQ 'high' | limit 50"`
    //   `TableDescriptor::example_note` carries the OCSF note for severity tables.
    //   `example_query` contains no `--` comment syntax.

    /// F-MED-002 (1/3): `build_example_query` output must contain NO `--` comment lines
    /// across ALL table shapes (column-free, count-recent, severity-IEQ, aggregate).
    ///
    /// ## Behaviour (post-pass-17 — GREEN)
    ///
    /// Severity-vocabulary tables (e.g., `crowdstrike_detections`, `armis_alerts`)
    /// now produce the bare pipe form with no comment lines:
    /// ```text
    /// FROM crowdstrike_detections | where severity IEQ 'high' | limit 50
    /// ```
    /// `q.contains("--")` is false → assertion passes for all shapes.
    ///
    /// ## Traces
    ///
    /// BC-2.10.012 §example_query no-comment requirement;
    /// LOCAL adversary pass-15 finding F-MED-002.
    #[test]
    fn test_BC_2_10_012_example_query_contains_no_comment_lines() {
        // Representative shapes covering all four priority-ladder branches.
        let shapes: Vec<(&str, Vec<ColumnDescriptor>)> = vec![
            // Column-free branch
            (
                "claroty_devices",
                vec![
                    col("uid", ColumnType::String),
                    col("device_category", ColumnType::String),
                    col("retired", ColumnType::Boolean),
                ],
            ),
            // Count-recent branch (Datetime, no severity vocabulary)
            (
                "cyberint_alerts_no_severity",
                vec![
                    col("event_time", ColumnType::Datetime),
                    col("title", ColumnType::String),
                ],
            ),
            // Severity-IEQ branch (crowdstrike — vocabulary registered)
            (
                "crowdstrike_detections",
                vec![
                    col("created_timestamp", ColumnType::Datetime),
                    col("severity", ColumnType::String),
                ],
            ),
            // Severity-IEQ branch (armis — vocabulary registered, no Datetime)
            (
                "armis_alerts",
                vec![
                    col("alert_id", ColumnType::String),
                    col("severity", ColumnType::String),
                ],
            ),
            // Aggregate branch (Integer column, no severity vocabulary)
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
            // BC-2.10.012: example_query must be pure PQL — no `--` comment syntax.
            // Currently FAILS for crowdstrike_detections and armis_alerts shapes:
            // build_example_query embeds `-- OCSF severity is Title-case ...` as the first line.
            assert!(
                !q.contains("--"),
                "F-MED-002: build_example_query output for table '{table}' must NOT contain \
                 `--` comment syntax; `example_query` must be pure parseable PQL per \
                 BC-2.10.012 (the OCSF casing note moves to `example_note: Option<String>`); \
                 got: {q:?}"
            );
        }
    }

    /// F-MED-002 (2/3): RAW `build_example_query` output must parse via `PrismQlParser`
    /// without any comment-stripping preprocessing, for ALL table shapes.
    ///
    /// This is a stricter sibling of the updated `test_obs2_all_build_example_query_outputs_parse_ok`.
    /// That test covers a broad set of shapes; this test is the canonical F-MED-002 trace.
    ///
    /// ## Behaviour (post-pass-17 — GREEN)
    ///
    /// Severity shapes are now bare pipe-form PQL with no `--` comment lines.
    /// `PrismQlParser::parse` succeeds for all shapes. The OCSF casing note is
    /// in `example_note`, not embedded in `example_query`.
    ///
    /// ## Traces
    ///
    /// BC-2.10.012 §Raw parseability; LOCAL adversary pass-15 F-MED-002.
    #[test]
    fn test_BC_2_10_012_example_query_parses_without_stripping() {
        use prism_query::filter_parser::PrismQlParser;

        // Same representative shapes as test_BC_2_10_012_example_query_contains_no_comment_lines.
        let shapes: Vec<(&str, Vec<ColumnDescriptor>)> = vec![
            (
                "claroty_devices",
                vec![
                    col("uid", ColumnType::String),
                    col("device_category", ColumnType::String),
                    col("retired", ColumnType::Boolean),
                ],
            ),
            (
                "cyberint_alerts_no_severity",
                vec![
                    col("event_time", ColumnType::Datetime),
                    col("title", ColumnType::String),
                ],
            ),
            (
                "crowdstrike_detections",
                vec![
                    col("created_timestamp", ColumnType::Datetime),
                    col("severity", ColumnType::String),
                ],
            ),
            (
                "armis_alerts",
                vec![
                    col("alert_id", ColumnType::String),
                    col("severity", ColumnType::String),
                ],
            ),
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
            // No stripping — raw output must parse directly.
            // Currently FAILS for crowdstrike_detections and armis_alerts:
            // the `-- OCSF ...` comment line causes PrismQlParser to return Err.
            let result = PrismQlParser::parse(&q);
            assert!(
                result.is_ok(),
                "F-MED-002: build_example_query output for table '{table}' must parse via \
                 PrismQlParser WITHOUT comment stripping per BC-2.10.012; \
                 currently FAILS because `build_example_query` embeds a `-- OCSF ...` \
                 comment line that PrismQL does not support \
                 (raw output={q:?}). Error: {result:?}"
            );
        }
    }

    // PASS-15: activate with example_note field
    //
    // F-MED-002 (3/3): `example_note` field on `TableDescriptor`.
    //
    // BC-2.10.012 adds `example_note: Option<String>` to `TableDescriptor`.
    // `build_example_query` (or a companion `build_example_with_note(table_name, columns)`)
    // must return:
    //   - `Some("OCSF severity is stored as Title-case ('High'). Use IEQ/IIN to match
    //      regardless of the case you type, or = 'High' for the exact canonical form.")`
    //     for severity-vocabulary tables  [BC-2.10.012 canonical, pass-17 F-MED-1]
    //   - `None` for all other tables (column-free, count-recent, aggregate shapes)
    //
    // PASS-15: activated — build_example_with_note + example_note field added (F-MED-002).
    // PASS-17: canonical note text updated to ADR-047-authoritative fuller form (F-MED-1).
    #[test]
    fn test_BC_2_10_012_example_note_some_for_severity_tables_none_otherwise() {
        use prism_query::filter_parser::PrismQlParser;

        const OCSF_NOTE: &str = "OCSF severity is stored as Title-case ('High'). \
            Use IEQ/IIN to match regardless of the case you type, \
            or = 'High' for the exact canonical form.";

        // Severity-vocabulary table: note must be Some(OCSF_NOTE).
        let sev_cols = vec![
            col("created_timestamp", ColumnType::Datetime),
            col("severity", ColumnType::String),
        ];
        let (sev_q, sev_note) = build_example_with_note("crowdstrike_detections", &sev_cols);
        assert_eq!(
            sev_note,
            Some(OCSF_NOTE.to_string()),
            "F-MED-002: severity-vocabulary table must produce example_note = Some(OCSF_NOTE); \
             got: {sev_note:?}"
        );
        // example_query must still be parseable after the split.
        let result = PrismQlParser::parse(&sev_q);
        assert!(
            result.is_ok(),
            "F-MED-002: example_query for crowdstrike_detections must parse after \
             note is moved to example_note; got: {sev_q:?}, error: {result:?}"
        );
        // example_query must NOT contain '--'.
        assert!(
            !sev_q.contains("--"),
            "F-MED-002: example_query must be comment-free; got: {sev_q:?}"
        );

        // Non-severity table: note must be None.
        let non_sev_cols = vec![
            col("uid", ColumnType::String),
            col("device_category", ColumnType::String),
            col("retired", ColumnType::Boolean),
        ];
        let (_non_q, non_note) = build_example_with_note("claroty_devices", &non_sev_cols);
        assert_eq!(
            non_note, None,
            "F-MED-002: non-severity table must produce example_note = None; \
             got: {non_note:?}"
        );
    }

    // ─── RG-062 (pass-17 F-LOW-2) ─────────────────────────────────────────────
    //
    // Story S-PRISMQL-CASE-INSENSITIVE-001 RG-062 requires a dedicated test that
    // verifies ALL `build_example_with_note` query outputs (the `.0` tuple element) parse
    // RAW via PrismQlParser with NO comment-stripping preprocessing.
    //
    // `build_example_query` delegates to `build_example_with_note(...).0`, so the two
    // functions produce identical query strings.  This test is the canonical RG-062
    // anchor that explicitly calls `build_example_with_note` to confirm the split function
    // satisfies BC-2.10.012 pure-PQL invariant for all table shapes.

    /// RG-062 (S-PRISMQL-CASE-INSENSITIVE-001 LOCAL pass-17 F-LOW-2):
    /// ALL `build_example_with_note` query outputs (tuple `.0`) parse RAW via
    /// `PrismQlParser` with NO stripping — confirming pure-PQL invariant
    /// for the split `build_example_with_note` function.
    ///
    /// This is the RG-062 canonical anchor.  The companion test
    /// `test_BC_2_10_012_example_query_parses_without_stripping` covers
    /// `build_example_query` (the thin wrapper); this test covers
    /// `build_example_with_note` directly.
    ///
    /// ## Traces
    ///
    /// BC-2.10.012 §Raw parseability; S-PRISMQL-CASE-INSENSITIVE-001 RG-062;
    /// LOCAL adversary pass-17 F-LOW-2.
    #[test]
    fn test_BC_2_10_012_build_example_with_note_query_parses_without_stripping() {
        use prism_query::filter_parser::PrismQlParser;

        // Full representative set covering all four priority-ladder branches.
        let shapes: Vec<(&str, Vec<ColumnDescriptor>)> = vec![
            // Column-free branch
            (
                "claroty_devices",
                vec![
                    col("uid", ColumnType::String),
                    col("device_category", ColumnType::String),
                    col("retired", ColumnType::Boolean),
                ],
            ),
            // Count-recent branch (Datetime, no severity vocabulary)
            (
                "cyberint_alerts_no_severity",
                vec![
                    col("event_time", ColumnType::Datetime),
                    col("title", ColumnType::String),
                ],
            ),
            // Severity-IEQ branch (crowdstrike — vocabulary registered)
            (
                "crowdstrike_detections",
                vec![
                    col("created_timestamp", ColumnType::Datetime),
                    col("severity", ColumnType::String),
                ],
            ),
            // Severity-IEQ branch (armis — vocabulary registered, no Datetime)
            (
                "armis_alerts",
                vec![
                    col("alert_id", ColumnType::String),
                    col("severity", ColumnType::String),
                ],
            ),
            // Aggregate branch (Integer column, no severity vocabulary)
            (
                "unknown_sensor_with_count",
                vec![
                    col("event_time", ColumnType::Datetime),
                    col("severity_id", ColumnType::Integer),
                ],
            ),
        ];

        for (table, columns) in &shapes {
            let (q, _note) = build_example_with_note(table, columns);
            // Raw parse — no stripping. BC-2.10.012: example_query must be pure PQL.
            let result = PrismQlParser::parse(&q);
            assert!(
                result.is_ok(),
                "RG-062 F-LOW-2: build_example_with_note query for table '{table}' must parse \
                 via PrismQlParser WITHOUT stripping (BC-2.10.012 pure-PQL invariant). \
                 Got: {q:?}. Error: {result:?}"
            );
            // Also verify no '--' comment syntax leaks into the query part.
            assert!(
                !q.contains("--"),
                "RG-062 F-LOW-2: build_example_with_note query for table '{table}' MUST NOT \
                 contain '--' comment syntax (pure-PQL invariant). Got: {q:?}"
            );
        }
    }

    // ── OBS-2 (LOCAL pass-18): column-presence gate lock tests ───────────────

    /// OBS-2 (LOCAL pass-18): a synthetic table under an UNKNOWN sensor prefix with a
    /// severity String column must receive the IEQ example + OCSF casing note.
    ///
    /// This test locks AC-025's broadened rule: ANY sensor table with a `severity`
    /// String column gets the IEQ example regardless of sensor prefix.  Before OBS-2 a
    /// new sensor like "sentinel" would silently miss the IEQ example until its prefix
    /// was manually added to SENSOR_SEVERITY_VOCABULARY.
    ///
    /// Load-bearing (TD-VSDD-059): reverting to an allowlist gate makes this test fail,
    /// catching the regression before it reaches demo recording.
    #[test]
    fn test_obs2_sentinel_alerts_with_severity_gets_ieq_example() {
        let columns = vec![
            col("event_time", ColumnType::Datetime),
            col("severity", ColumnType::String),
            col("alert_name", ColumnType::String),
        ];

        let (q, note) = build_example_with_note("sentinel_alerts", &columns);

        // Column-presence gate: IEQ form fires for any severity-column table.
        assert!(
            q.contains("IEQ"),
            "OBS-2 (LOCAL pass-18): sentinel_alerts (unknown sensor prefix) with a \
             severity String column must emit the IEQ form per AC-025 column-presence gate. \
             Got: {q}"
        );
        let note_str = note.as_deref().unwrap_or("");
        assert!(
            note_str.contains("Title-case") || note_str.contains("title-case"),
            "OBS-2 (LOCAL pass-18): example_note must contain the OCSF casing hint for \
             sentinel_alerts severity column; got note: {note_str}"
        );
        // Pure-PQL invariant: no '--' comment lines in the query.
        assert!(
            !q.contains("--"),
            "OBS-2 (LOCAL pass-18): IEQ form for sentinel_alerts must be pure PQL. Got: {q}"
        );
    }

    /// F-P20-LOW-001 (LOCAL pass-20): a sensor whose `severity` column has type
    /// `ColumnType::Integer` (ordinal severity, not a string label) must NOT receive
    /// the IEQ severity example or OCSF casing note.
    ///
    /// ## Problem
    ///
    /// The `has_severity` gate (line ~663) previously checked column NAME only:
    /// `columns.iter().any(|c| c.name == "severity")`.  A future sensor TOML that
    /// declares `severity` as `ColumnType::Integer` (ordinal) would incorrectly fire
    /// the IEQ branch and emit:
    ///   `FROM <table> | where severity IEQ 'high' | limit 50`
    /// — a type-mismatch query that would fail the `check_ci_column_types` pre-flight
    /// with E-QUERY-002 (string literal compared to Integer column).
    ///
    /// ## Red Gate
    ///
    /// With the name-only gate, the Integer-severity table triggers the IEQ branch and
    /// `note.is_some()` → assertion FAILS.
    ///
    /// ## Green Gate
    ///
    /// PASSES once the gate is tightened to:
    /// `columns.iter().any(|c| c.name == "severity" && matches!(c.col_type, ColumnType::String))`
    ///
    /// Traces: S-PRISMQL-CASE-INSENSITIVE-001 F-P20-LOW-001; LOCAL adversary pass-20.
    #[test]
    fn test_f_p20_low001_severity_integer_type_does_not_get_ieq() {
        // Sensor whose `severity` column is ordinal (Integer), NOT a String label.
        let columns = vec![
            col("event_time", ColumnType::Datetime),
            col("severity", ColumnType::Integer),
            col("alert_name", ColumnType::String),
        ];

        let (q, note) = build_example_with_note("future_sensor_ordinal_severity", &columns);

        // Integer `severity` must NOT trigger the IEQ branch.
        // F-P20-LOW-001 RED Gate: name-only gate fires even for Integer severity → note is Some.
        assert!(
            note.is_none(),
            "F-P20-LOW-001: a column named 'severity' with ColumnType::Integer must NOT \
             trigger the IEQ example — the gate must check both name AND type. \
             An IEQ example against an Integer column would fail E-QUERY-002 type check. \
             Got example_note: {note:?}"
        );
        // The query must NOT contain IEQ (an invalid operator for Integer columns).
        assert!(
            !q.contains("IEQ"),
            "F-P20-LOW-001: IEQ operator must NOT appear in the example query for a \
             severity Integer column (type mismatch); got: {q:?}"
        );
        // With a non-String severity column (Integer), the IEQ branch skips. The Integer
        // column triggers the aggregate branch instead (name-only gate no longer fires).
        // Verify the aggregate form is chosen (SELECT <col>, COUNT(*) ... GROUP BY ...).
        assert!(
            q.contains("GROUP BY") && q.contains("COUNT(*)"),
            "F-P20-LOW-001: Integer-severity table must fall back to the aggregate branch \
             (GROUP BY / COUNT(*)) since the IEQ branch requires String type; got: {q:?}"
        );
    }

    /// OBS-2 (LOCAL pass-18) — claroty tables have NO severity column; no IEQ example
    /// should be emitted (column-presence gate does not fire when there is no severity
    /// column, regardless of how broadly the rule is stated).
    ///
    /// This confirms that the broadened gate (`has_severity` alone) is not wider than
    /// intended: claroty_devices has only String and Boolean columns and must still fall
    /// through to the column-free fallback.
    #[test]
    fn test_obs2_claroty_devices_no_severity_column_still_no_ieq_note() {
        let columns = vec![
            col("uid", ColumnType::String),
            col("asset_id", ColumnType::String),
            col("device_category", ColumnType::String),
            col("retired", ColumnType::Boolean),
        ];

        let (q, note) = build_example_with_note("claroty_devices", &columns);

        // No severity column → column-presence gate does not fire → no IEQ example.
        assert!(
            note.is_none(),
            "OBS-2 (LOCAL pass-18): claroty_devices has no severity column, so \
             example_note must be None; got: {note:?}"
        );
        // Without severity column, falls through to column-free fallback.
        assert_eq!(
            q, "SELECT * FROM claroty_devices LIMIT 25",
            "OBS-2 (LOCAL pass-18): claroty_devices (no severity, no datetime) must \
             use column-free fallback; got: {q}"
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// S-ADR058-OCSF-ROUTING-001 Red Gate Tests — RG-007 and RG-025
//
// These tests exercise `build_tables_for_client` with ocsf_column_naming=true
// sensors (AC-006 / ADR-058 §G Tier-1/Tier-2 ColumnDescriptor model).
// Both exercise `build_ocsf_column_descriptors` in the ocsf_column_naming=true branch.
// ─────────────────────────────────────────────────────────────────────────────
#[cfg(test)]
mod ocsf_routing_tests {
    use std::collections::HashMap;
    use std::sync::Arc;

    use arc_swap::ArcSwap;
    use prism_core::column::ColumnType;
    use prism_spec_engine::config_manager::ConfigManager;
    use prism_spec_engine::spec_parser::{AuthType, ColumnSpec, FetchStep, SensorSpec, TableSpec};
    use prism_spec_engine::types::ConfigSnapshot;

    use super::build_tables_for_client;

    /// Build a `SensorSpec` with `ocsf_column_naming = true`.
    ///
    /// `SensorSpec::new()` hardcodes `ocsf_column_naming = false`; `#[non_exhaustive]`
    /// blocks struct-literal construction outside the defining crate. TOML deserialization
    /// is the only external construction path per ADR-058 §I1.
    fn ocsf_sensor_spec_toml(sensor_id: &str, toml_tables: &str) -> SensorSpec {
        let toml = format!(
            r#"
sensor_id = "{sensor_id}"
name = "OCSF Test"
auth_type = "api_key"
base_url = "https://example.com"
version = "1.0.0"
ocsf_column_naming = true
{toml_tables}
"#
        );
        toml::from_str::<SensorSpec>(&toml).expect("OCSF TOML must parse")
    }

    /// Minimal FetchStep for test TableSpec construction.
    fn minimal_fetch_step() -> FetchStep {
        FetchStep::new(
            "fetch",
            "GET",
            "/api/v1/items",
            None,
            "$.data",
            None,
            vec![],
            None,
            None,
        )
    }

    /// Wrap `ConfigManager` into the `Arc<ArcSwap<ConfigManager>>` required by
    /// `build_tables_for_client`.
    fn arc_cm(sensor_id: &str, spec: SensorSpec) -> Arc<ArcSwap<ConfigManager>> {
        let snapshot = ConfigSnapshot {
            sensor_specs: HashMap::from([(sensor_id.to_string(), spec)]),
            failed_specs: HashMap::new(),
            snapshot_hash: String::new(),
            org_display_names: HashMap::new(),
        };
        Arc::new(ArcSwap::new(Arc::new(ConfigManager::new(snapshot))))
    }

    /// RG-007 / AC-006 / ADR-058 §G / BC-2.16.003 EC-016-013-028/029
    ///
    /// When `sensor_spec.ocsf_column_naming == true`, `build_tables_for_client` MUST
    /// return `ColumnDescriptor.name == ocsf_field_to_arrow_name(ocsf_field)` for Tier-1
    /// columns (those with `ocsf_field == Some`), NOT `col.name`.
    ///
    /// The LLM agent calls `prism_describe` before authoring PrismQL queries. The column
    /// names returned here ARE the queryable identifiers the LLM uses — if they are
    /// pre-flattening `col.name` values, the LLM writes queries that DataFusion rejects.
    ///
    /// Wire-shape annotation (SAP-3): this test calls the public surface (`build_tables_for_client`)
    /// rather than the internal column-builder directly. The `build_tables_for_client` function
    /// is a module-private function; this test is in a `#[cfg(test)]` module in the same file
    /// (same-module access, not synthetic-AST coverage).
    ///
    /// **Red gate resolved:** branch now implemented via `build_ocsf_column_descriptors`.
    #[test]
    fn test_prism_describe_ocsf_column_naming_true_returns_flattened_name_and_dotted_description() {
        // Use Claroty audit_logs (post-KF-01): note→comment (Tier-1, ocsf_field=Some).
        let cols = vec![
            ColumnSpec::new(
                "note",
                ColumnType::String,
                Some("comment".to_string()),
                vec![],
            ),
            ColumnSpec::new(
                "action",
                ColumnType::String,
                Some("activity_name".to_string()),
                vec![],
            ),
            ColumnSpec::new("category", ColumnType::String, None, vec![]), // Tier-2
        ];
        let table = TableSpec::new_point_in_time(
            "audit_logs",
            "entity_management",
            cols,
            vec![minimal_fetch_step()],
        );
        let spec = toml::from_str::<SensorSpec>(
            r#"
sensor_id = "claroty"
name = "Claroty"
auth_type = "api_key"
base_url = "https://example.com"
version = "1.0.0"
ocsf_column_naming = true
"#,
        )
        .expect("Claroty TOML must parse");
        // Inject the table manually via TOML struct construction is not possible for tables;
        // instead use a full TOML spec with embedded table definition.
        let full_toml = r#"
sensor_id = "claroty"
name = "Claroty"
auth_type = "api_key"
base_url = "https://example.com"
version = "1.0.0"
ocsf_column_naming = true

[[tables]]
table_name = "audit_logs"
ocsf_class = "entity_management"

[[tables.columns]]
name = "note"
column_type = "string"
ocsf_field = "comment"

[[tables.columns]]
name = "action"
column_type = "string"
ocsf_field = "activity_name"

[[tables.columns]]
name = "category"
column_type = "string"

[[tables.steps]]
name = "fetch"
method = "GET"
path_template = "/api/v1/audit_logs"
response_path = "$.data"
variables_produced = []
"#;
        let spec_with_tables = toml::from_str::<SensorSpec>(full_toml)
            .expect("Full Claroty TOML with tables must parse");

        let cm = arc_cm("claroty", spec_with_tables);
        // Calls build_ocsf_column_descriptors via the ocsf_column_naming=true branch.
        let tables = build_tables_for_client("claroty", None, Some(&cm));

        // Assert Tier-1 columns have flattened ocsf_field names.
        // column "note" with ocsf_field="comment" → name="comment" (single-segment, unchanged).
        let audio_table = tables
            .iter()
            .find(|t| t.name == "claroty_audit_logs")
            .expect("AC-006 (RG-007): 'claroty_audit_logs' table MUST exist in describe response");

        let comment_col = audio_table.columns.iter().find(|c| c.name == "comment");
        assert!(
            comment_col.is_some(),
            "AC-006 (RG-007): Tier-1 column 'note' (ocsf_field='comment') MUST return \
             ColumnDescriptor.name='comment' (single-segment ocsf_field is unchanged). \
             LLM must query `WHERE comment = 'value'`, not `WHERE note = 'value'`."
        );
        assert_eq!(
            comment_col.unwrap().description.as_deref(),
            Some("comment"),
            "AC-006 (RG-007): description MUST equal the original dotted ocsf_field path \
             ('comment') so LLM agents understand the OCSF provenance."
        );

        // col.name "note" MUST NOT appear as a ColumnDescriptor (Tier-1 replaces it).
        let note_col = audio_table.columns.iter().find(|c| c.name == "note");
        assert!(
            note_col.is_none(),
            "AC-006 (RG-007): col.name 'note' MUST NOT appear as a ColumnDescriptor \
             when ocsf_column_naming=true; it is replaced by the flattened ocsf_field name."
        );

        // col.name "category" (Tier-2, ocsf_field=None) MUST NOT appear as individual col.
        let cat_col = audio_table.columns.iter().find(|c| c.name == "category");
        assert!(
            cat_col.is_none(),
            "AC-006 (RG-007): Tier-2 column 'category' (ocsf_field=None) MUST NOT appear \
             as an individual ColumnDescriptor. LLM agents must not query it directly."
        );
    }

    /// RG-025 / AC-006 / ADR-058 §G — raw_extensions descriptor presence + no phantom names
    ///
    /// When `ocsf_column_naming == true`, `build_tables_for_client` MUST emit exactly ONE
    /// `ColumnDescriptor` with `name == "raw_extensions"` and `col_type == Json` when any
    /// Tier-2 column (ocsf_field == None) exists, AND must not emit any individual
    /// ColumnDescriptors for those Tier-2 columns.
    ///
    /// The raw_extensions descriptor's `description` MUST enumerate the source Tier-2 column
    /// names so LLM agents know what vendor-specific data they can parse from the blob.
    ///
    /// Wire-shape annotation (SAP-3): same-module access to `build_tables_for_client`.
    ///
    /// **Red gate resolved:** branch now implemented via `build_ocsf_column_descriptors`.
    #[test]
    fn test_prism_describe_ocsf_column_naming_true_raw_extensions_descriptor_and_no_phantom_col_names(
    ) {
        let full_toml = r#"
sensor_id = "claroty"
name = "Claroty"
auth_type = "api_key"
base_url = "https://example.com"
version = "1.0.0"
ocsf_column_naming = true

[[tables]]
table_name = "devices"
ocsf_class = "inventory_info"

[[tables.columns]]
name = "device_uid"
column_type = "string"
ocsf_field = "device.uid"

[[tables.columns]]
name = "ip_list"
column_type = "string"

[[tables.columns]]
name = "device_type"
column_type = "string"

[[tables.steps]]
name = "fetch"
method = "GET"
path_template = "/api/v1/devices"
response_path = "$.data"
variables_produced = []
"#;
        let spec =
            toml::from_str::<SensorSpec>(full_toml).expect("Full Claroty devices TOML must parse");

        let cm = arc_cm("claroty", spec);
        // Calls build_ocsf_column_descriptors via the ocsf_column_naming=true branch.
        let tables = build_tables_for_client("claroty", None, Some(&cm));

        let devices_table = tables
            .iter()
            .find(|t| t.name == "claroty_devices")
            .expect("AC-006 (RG-025): 'claroty_devices' table MUST exist in describe response");

        // (i): exactly ONE raw_extensions ColumnDescriptor.
        let raw_ext_cols: Vec<_> = devices_table
            .columns
            .iter()
            .filter(|c| c.name == "raw_extensions")
            .collect();
        assert_eq!(
            raw_ext_cols.len(),
            1,
            "AC-006 (RG-025): MUST emit exactly ONE 'raw_extensions' ColumnDescriptor \
             when any Tier-2 column (ocsf_field==None) exists; got {} entries.",
            raw_ext_cols.len()
        );

        // (ii): col_type == Json.
        assert_eq!(
            raw_ext_cols[0].col_type,
            ColumnType::Json,
            "AC-006 (RG-025): raw_extensions ColumnDescriptor MUST have col_type=Json; \
             LLM agents use this to know they must parse JSON from the blob."
        );

        // (iii): description enumerates the Tier-2 source column names.
        let desc = raw_ext_cols[0].description.as_deref().unwrap_or("");
        assert!(
            desc.contains("ip_list"),
            "AC-006 (RG-025): raw_extensions description MUST enumerate Tier-2 source \
             column names so LLM agents know what vendor data is in the blob; \
             'ip_list' MUST appear in description. Got: {:?}",
            desc
        );
        assert!(
            desc.contains("device_type"),
            "AC-006 (RG-025): 'device_type' MUST appear in raw_extensions description. \
             Got: {:?}",
            desc
        );

        // (iv): no phantom col.name entries for Tier-2 columns.
        let ip_list_col = devices_table.columns.iter().find(|c| c.name == "ip_list");
        assert!(
            ip_list_col.is_none(),
            "AC-006 (RG-025): Tier-2 column 'ip_list' (ocsf_field==None) MUST NOT appear \
             as an individual ColumnDescriptor; LLM would build broken queries like \
             `WHERE ip_list = 'x'` which DataFusion rejects because 'ip_list' is \
             not a first-class Arrow field when ocsf_column_naming=true."
        );

        // (v): Tier-1 column "device.uid" → "device_uid" name appears.
        let device_uid_col = devices_table
            .columns
            .iter()
            .find(|c| c.name == "device_uid");
        assert!(
            device_uid_col.is_some(),
            "AC-006 (RG-025): Tier-1 column 'device_uid' (device.uid flattened) MUST appear \
             as an individual ColumnDescriptor."
        );
    }

    /// RG-028 / AC-015 / OQ-003 — `prism_describe` emits `class_uid` and `_sensor`
    /// synthesized ColumnDescriptors when `ocsf_column_naming = true`.
    ///
    /// When `ocsf_column_naming = true`, `build_tables_for_client` MUST emit two synthesized
    /// ColumnDescriptors appended after the Tier-1 OCSF-flattened descriptors and the single
    /// `raw_extensions` descriptor:
    ///   1. `class_uid` (ColumnType::Integer, nullable=false) — the OCSF class UID injected by
    ///      `pipeline_result_to_record_batch` so the LLM agent can use it as a filter target.
    ///   2. `_sensor` (ColumnType::String, nullable=false) — the sensor identifier column.
    ///
    /// These synthesized columns exist in the Arrow schema produced by
    /// `pipeline_result_to_record_batch` but are currently invisible to the LLM agent via
    /// `prism_describe`. Without these ColumnDescriptors, the LLM agent cannot learn that
    /// `WHERE class_uid = 3004` and `WHERE _sensor = 'claroty'` are valid filter targets.
    ///
    /// **Red gate resolved:** branch now implemented via `build_ocsf_column_descriptors`;
    /// all four assertions pass.
    ///
    /// Wire-shape assertion (CLAUDE.md §Conventions): serialize the `prism_describe` table
    /// list to JSON and assert both ColumnDescriptor entries appear with the exact `name`,
    /// `col_type`, and `nullable` values at the wire level.
    ///
    /// SAP-3: end-to-end from `build_tables_for_client` (the MCP surface) not internal handler.
    /// Covers AC-015.
    /// Traces to BC-2.16.003 §Interpretation A (synthesized columns produced by
    /// `pipeline_result_to_record_batch` must be advertised by `prism_describe`; OQ-003
    /// human decision 2026-08-21).
    #[test]
    fn test_prism_describe_ocsf_column_naming_true_emits_class_uid_and_sensor_descriptors() {
        let toml_tables = r#"
[[tables]]
table_name = "audit_logs"
ocsf_class = "entity_management"

[[tables.columns]]
name = "note"
column_type = "string"
ocsf_field = "comment"

[[tables.columns]]
name = "category"
column_type = "string"

[[tables.steps]]
name = "fetch"
method = "GET"
path_template = "/api/v1/audit_logs"
response_path = "$.data"
variables_produced = []
"#;
        let spec = ocsf_sensor_spec_toml("claroty", toml_tables);
        let cm = arc_cm("claroty", spec);

        // Calls build_ocsf_column_descriptors via the ocsf_column_naming=true branch.
        let tables = build_tables_for_client("claroty", None, Some(&cm));
        let audit_table = tables
            .iter()
            .find(|t| t.name == "claroty_audit_logs")
            .expect("AC-015 (RG-028): 'claroty_audit_logs' table must exist");

        // (i): class_uid ColumnDescriptor — Integer, non-nullable.
        let class_uid_col = audit_table.columns.iter().find(|c| c.name == "class_uid");
        assert!(
            class_uid_col.is_some(),
            "AC-015 (RG-028/OQ-003): 'class_uid' ColumnDescriptor MUST be emitted by \
             prism_describe when ocsf_column_naming=true. Got columns: {:?}",
            audit_table
                .columns
                .iter()
                .map(|c| &c.name)
                .collect::<Vec<_>>()
        );
        let class_uid_descriptor = class_uid_col.unwrap();
        assert_eq!(
            class_uid_descriptor.col_type,
            prism_core::column::ColumnType::Integer,
            "AC-015 (RG-028/OQ-003): class_uid ColumnDescriptor MUST have col_type=Integer"
        );
        assert!(
            !class_uid_descriptor.nullable,
            "AC-015 (RG-028/OQ-003): class_uid ColumnDescriptor MUST have nullable=false"
        );

        // (ii): _sensor ColumnDescriptor — String, non-nullable.
        let sensor_col = audit_table.columns.iter().find(|c| c.name == "_sensor");
        assert!(
            sensor_col.is_some(),
            "AC-015 (RG-028/OQ-003): '_sensor' ColumnDescriptor MUST be emitted by \
             prism_describe when ocsf_column_naming=true. Got columns: {:?}",
            audit_table
                .columns
                .iter()
                .map(|c| &c.name)
                .collect::<Vec<_>>()
        );
        let sensor_descriptor = sensor_col.unwrap();
        assert_eq!(
            sensor_descriptor.col_type,
            prism_core::column::ColumnType::String,
            "AC-015 (RG-028/OQ-003): _sensor ColumnDescriptor MUST have col_type=String"
        );
        assert!(
            !sensor_descriptor.nullable,
            "AC-015 (RG-028/OQ-003): _sensor ColumnDescriptor MUST have nullable=false"
        );

        // (iii)+(iv): ordering — class_uid and _sensor MUST appear AFTER all Tier-1 and
        // raw_extensions descriptors. They MUST be the last two entries.
        let ncols = audit_table.columns.len();
        assert!(
            ncols >= 4,
            "AC-015 (RG-028/OQ-003): MUST have at least 4 ColumnDescriptors: \
             Tier-1 (comment), raw_extensions, class_uid, _sensor. Got {ncols}"
        );
        let last_two: Vec<&str> = audit_table
            .columns
            .iter()
            .rev()
            .take(2)
            .map(|c| c.name.as_str())
            .collect();
        assert!(
            last_two.contains(&"class_uid") && last_two.contains(&"_sensor"),
            "AC-015 (RG-028/OQ-003): 'class_uid' and '_sensor' MUST be the last two \
             ColumnDescriptors (after Tier-1 and raw_extensions). Last two: {last_two:?}"
        );

        // Wire-shape assertion: serialize to JSON and assert both entries appear with
        // exact name, col_type, nullable values (CLAUDE.md §Conventions wire-shape discipline).
        let json_bytes = serde_json::to_string(&audit_table.columns)
            .expect("ColumnDescriptor list must serialize to JSON");
        assert!(
            json_bytes.contains(r#""name":"class_uid""#),
            "AC-015 (RG-028/OQ-003): wire-level JSON must contain '\"name\":\"class_uid\"'. \
             Got: {json_bytes}"
        );
        assert!(
            json_bytes.contains(r#""name":"_sensor""#),
            "AC-015 (RG-028/OQ-003): wire-level JSON must contain '\"name\":\"_sensor\"'. \
             Got: {json_bytes}"
        );
        assert!(
            json_bytes.contains(r#""nullable":false"#),
            "AC-015 (RG-028/OQ-003): wire-level JSON must contain '\"nullable\":false' \
             for the synthesized descriptors. Got: {json_bytes}"
        );

        // ── F-P2-HIGH-1: description text assertions (RG-028 v1.48 strengthening) ──
        //
        // The previous test was paper-green: it checked name/col_type/nullable/ordering but NOT
        // the description text, allowing `description: None` to slip through.  These assertions
        // require the EXACT canonical strings from ADR-058 §G v2.28 / BC-2.16.003 v1.23 / AC-015.
        //
        // Rust struct level (description field value):
        let class_uid_desc = class_uid_descriptor.description.as_deref().unwrap_or("");
        assert_eq!(
            class_uid_desc,
            "OCSF event class identifier derived from sensor TOML ocsf_class. \
             Example: 3004 for entity_management (audit_logs), \
             2004 for detection_finding (alerts, device_alert_relations), \
             5001 for inventory_info (devices).",
            "AC-015 (RG-028/OQ-003/F-P2-HIGH-1): class_uid ColumnDescriptor MUST have \
             the canonical description text per ADR-058 §G / BC-2.16.003 AC-015; \
             current code emits description:None"
        );
        let sensor_desc = sensor_descriptor.description.as_deref().unwrap_or("");
        assert_eq!(
            sensor_desc, "Sensor identifier. Value: <sensor_id> (e.g., 'claroty').",
            "AC-015 (RG-028/OQ-003/F-P2-HIGH-1): _sensor ColumnDescriptor MUST have \
             the canonical description text per ADR-058 §G / BC-2.16.003 AC-015; \
             current code emits description:None"
        );

        // Wire-shape level: description text must appear in the serialized JSON that the
        // LLM agent receives (CLAUDE.md §Conventions wire-shape discipline, SID-2).
        assert!(
            json_bytes.contains("OCSF event class identifier derived from sensor TOML ocsf_class"),
            "AC-015 (RG-028/OQ-003/F-P2-HIGH-1): wire-level JSON must contain class_uid \
             canonical description text (LLM agent receives this field). Got: {json_bytes}"
        );
        assert!(
            json_bytes.contains("Sensor identifier. Value: <sensor_id>"),
            "AC-015 (RG-028/OQ-003/F-P2-HIGH-1): wire-level JSON must contain _sensor \
             canonical description text (LLM agent receives this field). Got: {json_bytes}"
        );
    }
}
