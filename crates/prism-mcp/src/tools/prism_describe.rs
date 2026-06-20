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
///
/// Self-check (BC-5.38.005 invariant 1):
/// "If I include this real implementation, will the test for this function pass
/// trivially without any implementer work?" — Yes: real implementation would
/// immediately satisfy AC-001 through AC-005 tests. Body = todo!(). (BC-5.38.001)
pub async fn handle_prism_describe(
    _client_id: String,
    _query_engine: Option<&std::sync::Arc<prism_query::engine::QueryEngine>>,
    _config_manager: Option<
        &std::sync::Arc<arc_swap::ArcSwap<prism_spec_engine::config_manager::ConfigManager>>,
    >,
    _audit_writer: Option<&std::sync::Arc<dyn prism_query::write_dispatch::AuditWriter>>,
) -> Result<CallToolResult, rmcp::model::ErrorData> {
    todo!("BC-2.10.012 AC-001..AC-005: implement prism_describe handler — \
           validate client_id via OrgSlug::new(); emit audit event (event_type=schema_enumeration.started); \
           read column schema from resolved_spec_map (multi-tenant) or config_manager (fallback); \
           build PrismDescribeResponse with TableDescriptor/ColumnDescriptor; \
           emit schema_enumeration.success or schema_enumeration.rejected; \
           return E-MCP-001 on format failure; success with empty tables for unknown/empty client; \
           DI-008: filter strictly by OrgSlug (never leak other clients tables)")
}

/// Build an auto-generated example PQL query for a table.
///
/// Always includes a count-recent fallback. Adds severity-filter variant when a
/// `severity` column is present. Adds aggregate variant when an aggregatable column
/// (Integer/Float) is present.
///
/// Self-check (BC-5.38.005 invariant 1):
/// "If I include this real implementation, will the test for this function pass
/// trivially without any implementer work?" — Yes for AC-002 example_query checks.
/// Body = todo!(). (BC-5.38.001)
pub fn build_example_query(_table_name: &str, _columns: &[ColumnDescriptor]) -> String {
    todo!(
        "BC-2.10.012 AC-002: build example query for table_name using columns — \
           count-recent fallback always; severity-filter if 'severity' column present; \
           aggregate if Integer/Float column present"
    )
}
