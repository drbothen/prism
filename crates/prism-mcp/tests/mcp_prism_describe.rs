//! Red Gate tests for S-DEMO-PRISMQL-ONBOARDING-001-A — AC-001 through AC-006.
//!
//! Covers `prism_describe` tool (BC-2.10.012) and `prismql://schema/{client_id}`
//! resource template (BC-2.10.013).
//!
//! ALL tests in this file must FAIL against the todo!() stubs (Red Gate per BC-5.38.001).
//!
//! # Test → AC mapping
//!
//! | Test | AC | BC |
//! |------|----|----|
//! | test_BC_2_10_012_prism_describe_happy_path_catalog | AC-001 + AC-002 | BC-2.10.012 |
//! | test_BC_2_10_012_prism_describe_audit_event_emitted | AC-002 | BC-2.10.012 |
//! | test_BC_2_10_012_prism_describe_empty_and_unknown_client | AC-003 | BC-2.10.012 |
//! | test_BC_2_10_012_prism_describe_invalid_client_id | AC-003 | BC-2.10.012 |
//! | test_BC_2_10_012_prism_describe_client_isolation | AC-004 | BC-2.10.012 DI-008 |
//! | test_BC_2_10_013_schema_resource_template_parity | AC-005 | BC-2.10.013 |
//! | test_BC_2_10_013_schema_resource_subscribe_notify | AC-006 | BC-2.10.013 |

use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use prism_core::{column::ColumnType, OrgSlug};
use prism_mcp::{
    resources::schema::{
        render_pql_schema_resource, SchemaSubscriberRegistry, SubscriberHandle,
        URI_TEMPLATE_PQL_SCHEMA,
    },
    tools::prism_describe::handle_prism_describe,
};
use prism_query::{
    write_dispatch::AuditWriter,
    write_pipeline::{QueryContext, WritePlan},
    write_result::WriteResult,
};
use prism_security::feature_flag::CapabilityCheckResult;
use prism_spec_engine::{
    spec_parser::{ColumnSpec, SensorSpec, TableSpec},
    types::ConfigSnapshot,
    AuthType, ConfigManager,
};
use ulid::Ulid;

// ─── Capturing AuditWriter for AC-002 ────────────────────────────────────────

/// In-memory audit writer that captures tool-call records for assertion.
///
/// Implements the full `AuditWriter` trait with no-op for write_intent / write_outcome,
/// and captures `write_tool_call` invocations in a shared vec.
#[derive(Clone, Default)]
struct CapturingAuditWriter {
    /// Captured (tool_name, client_id, outcome) tuples from write_tool_call.
    calls: Arc<Mutex<Vec<(String, Option<String>, String)>>>,
}

#[async_trait]
impl AuditWriter for CapturingAuditWriter {
    async fn write_intent(
        &self,
        _plan: &WritePlan,
        _context: &QueryContext,
        _capability_check: &CapabilityCheckResult,
    ) -> Result<Ulid, prism_core::error::PrismError> {
        Ok(Ulid::new())
    }

    async fn write_outcome(
        &self,
        _intent_id: Ulid,
        _result: &WriteResult,
    ) -> Result<(), prism_core::error::PrismError> {
        Ok(())
    }

    async fn write_tool_call(
        &self,
        tool_name: &str,
        client_id: Option<&str>,
        outcome: &str,
    ) -> Result<(), prism_core::error::PrismError> {
        self.calls.lock().unwrap().push((
            tool_name.to_string(),
            client_id.map(|s| s.to_string()),
            outcome.to_string(),
        ));
        Ok(())
    }
}

// ─── Test fixture helpers ─────────────────────────────────────────────────────

/// Build a `ConfigManager` with one sensor ("crowdstrike") that has 3 tables:
/// `alerts` (with severity + id + event_count columns), `devices` (with hostname),
/// `events` (zero columns — EC-002 zero-column table case).
///
/// Used by AC-001, AC-002, AC-005 happy-path fixtures.
fn make_config_manager_acme_crowdstrike() -> Arc<arc_swap::ArcSwap<ConfigManager>> {
    let alerts_table = TableSpec::new_point_in_time(
        "alerts",
        "security_finding",
        vec![
            ColumnSpec::new(
                "severity",
                ColumnType::String,
                Some("severity".to_string()),
                vec![],
            ),
            ColumnSpec::new("id", ColumnType::String, Some("id".to_string()), vec![]),
            ColumnSpec::new("event_count", ColumnType::Integer, None, vec![]),
        ],
        vec![],
    );

    let devices_table = TableSpec::new_point_in_time(
        "devices",
        "device_inventory_info",
        vec![ColumnSpec::new(
            "hostname",
            ColumnType::String,
            None,
            vec![],
        )],
        vec![],
    );

    // EC-002 (BC-2.10.012): zero-column table — valid, returns count-recent example.
    let events_table = TableSpec::new_point_in_time("events", "security_finding", vec![], vec![]);

    let cs_spec = SensorSpec::new(
        "crowdstrike",
        "CrowdStrike Falcon sensor",
        AuthType::ApiKey,
        "https://api.crowdstrike.com",
        vec![alerts_table, devices_table, events_table],
        None,
        "1.0.0",
        vec![],
    );

    let mut sensor_specs = std::collections::HashMap::new();
    sensor_specs.insert("crowdstrike".to_string(), cs_spec);

    let snapshot = ConfigSnapshot {
        sensor_specs,
        ..ConfigSnapshot::empty()
    };
    Arc::new(arc_swap::ArcSwap::from_pointee(ConfigManager::new(
        snapshot,
    )))
}

/// Build a minimal empty `ConfigManager` (no sensors).
///
/// Used by AC-003 empty/unknown-client fixture.
fn make_config_manager_empty() -> Arc<arc_swap::ArcSwap<ConfigManager>> {
    Arc::new(arc_swap::ArcSwap::from_pointee(ConfigManager::empty()))
}

/// Build a `ConfigManager` for multi-tenant isolation test.
/// "crowdstrike" sensor has `crowdstrike_alerts` table with `severity` column.
/// "claroty" sensor has `claroty_assets` table with `asset_name` column.
///
/// Used by AC-004 client isolation fixture.
fn make_config_manager_two_tenants() -> Arc<arc_swap::ArcSwap<ConfigManager>> {
    let cs_table = TableSpec::new_point_in_time(
        "crowdstrike_alerts",
        "security_finding",
        vec![ColumnSpec::new(
            "severity",
            ColumnType::String,
            None,
            vec![],
        )],
        vec![],
    );
    let claroty_table = TableSpec::new_point_in_time(
        "claroty_assets",
        "device_inventory_info",
        vec![ColumnSpec::new(
            "asset_name",
            ColumnType::String,
            None,
            vec![],
        )],
        vec![],
    );

    let cs_spec = SensorSpec::new(
        "crowdstrike",
        "CrowdStrike sensor",
        AuthType::ApiKey,
        "https://api.crowdstrike.com",
        vec![cs_table],
        None,
        "1.0.0",
        vec![],
    );
    let cl_spec = SensorSpec::new(
        "claroty",
        "Claroty sensor",
        AuthType::ApiKey,
        "https://api.claroty.com",
        vec![claroty_table],
        None,
        "1.0.0",
        vec![],
    );

    let mut sensor_specs = std::collections::HashMap::new();
    sensor_specs.insert("crowdstrike".to_string(), cs_spec);
    sensor_specs.insert("claroty".to_string(), cl_spec);

    let snapshot = ConfigSnapshot {
        sensor_specs,
        ..ConfigSnapshot::empty()
    };
    Arc::new(arc_swap::ArcSwap::from_pointee(ConfigManager::new(
        snapshot,
    )))
}

// ─── AC-001 + AC-002: prism_describe happy-path catalog ──────────────────────

/// AC-001 (BC-2.10.012 — Tool registration + annotations):
/// The `prism_describe` handler returns a `CallToolResult` with correct response
/// shape and annotations for a valid client.
///
/// AC-002 (BC-2.10.012 — Response shape, example queries, pql_hints):
/// For "crowdstrike" client with 3 tables configured, the response contains
/// `client_id: "crowdstrike"`, a `tables` array with 3 entries (each with
/// non-empty `name`, `sensor_type`, `columns`, and `example_query`), and
/// non-empty `pql_hints`.
///
/// RED GATE: Fails with todo!() panic from `handle_prism_describe` in
/// `crates/prism-mcp/src/tools/prism_describe.rs`.
#[tokio::test]
async fn test_BC_2_10_012_prism_describe_happy_path_catalog() {
    let config_manager = make_config_manager_acme_crowdstrike();

    // Call handle_prism_describe with no query_engine (single-tenant fallback path)
    // and no audit_writer (fail-open path — DI-004).
    let result = handle_prism_describe(
        "crowdstrike".to_string(),
        None,                  // no query_engine → config_manager fallback
        Some(&config_manager), // single-tenant spec source
        None,                  // no audit_writer → fail-open
    )
    .await;

    // The call must succeed (Ok).
    let call_result = result.expect(
        "BC-2.10.012 AC-001+AC-002: handle_prism_describe must return Ok for valid \
         client_id; got Err",
    );

    // BC-2.10.012 postcondition: tool result must not be is_error=true.
    assert!(
        !call_result.is_error.unwrap_or(false),
        "BC-2.10.012 AC-001: prism_describe must not return is_error=true for valid client. \
         Got call_result: {:?}",
        call_result
    );

    // Extract JSON content to verify response shape.
    let content_text: String = call_result
        .content
        .iter()
        .filter_map(|c| c.as_text().map(|t| t.text.clone()))
        .collect::<Vec<_>>()
        .join("");

    let parsed: serde_json::Value = serde_json::from_str(&content_text).expect(
        "BC-2.10.012 AC-002: prism_describe response must be valid JSON; \
         got non-JSON content",
    );

    // AC-002: client_id field must match the requested client.
    assert_eq!(
        parsed.get("client_id").and_then(|v| v.as_str()),
        Some("crowdstrike"),
        "BC-2.10.012 AC-002: response client_id must be 'crowdstrike'; \
         got: {:?}",
        parsed.get("client_id")
    );

    // AC-002: tables array must have exactly 3 entries (alerts, devices, events).
    let tables = parsed
        .get("tables")
        .and_then(|v| v.as_array())
        .expect("BC-2.10.012 AC-002: response must contain a 'tables' array");
    assert_eq!(
        tables.len(),
        3,
        "BC-2.10.012 AC-002: 3 tables configured for crowdstrike; got {} tables. \
         Response: {:?}",
        tables.len(),
        content_text
    );

    // AC-002: each table must have name, sensor_type, columns, and example_query.
    for table in tables {
        let name = table.get("name").and_then(|v| v.as_str()).unwrap_or("");
        assert!(
            !name.is_empty(),
            "BC-2.10.012 AC-002: each table entry must have a non-empty 'name' field; \
             got entry: {:?}",
            table
        );

        assert_eq!(
            table.get("sensor_type").and_then(|v| v.as_str()),
            Some("crowdstrike"),
            "BC-2.10.012 AC-002: sensor_type must be 'crowdstrike' for crowdstrike tables; \
             table entry: {:?}",
            table
        );

        assert!(
            table.get("columns").and_then(|v| v.as_array()).is_some(),
            "BC-2.10.012 AC-002: each table entry must have a 'columns' array; \
             table entry: {:?}",
            table
        );

        let example_query = table
            .get("example_query")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        assert!(
            !example_query.is_empty(),
            "BC-2.10.012 AC-002: each table entry must have a non-empty 'example_query'; \
             table '{}' has empty example_query",
            name
        );

        // AC-002: example_query must contain the real table name (not a generic placeholder).
        assert!(
            example_query.contains(name),
            "BC-2.10.012 AC-002: example_query must use the real table name '{}' (not a \
             generic placeholder like '<table>'); got example_query: {:?}",
            name,
            example_query
        );
    }

    // AC-002: pql_hints must be a non-empty array.
    let pql_hints = parsed
        .get("pql_hints")
        .and_then(|v| v.as_array())
        .expect("BC-2.10.012 AC-002: response must contain a 'pql_hints' array");
    assert!(
        !pql_hints.is_empty(),
        "BC-2.10.012 AC-002: pql_hints must be non-empty for a populated client; \
         got empty pql_hints"
    );
}

// ─── AC-002: audit event emission ────────────────────────────────────────────

/// AC-002 (BC-2.10.012 — Audit event emission):
/// Every call to `prism_describe` must invoke `audit_writer.write_tool_call` with
/// `tool_name: "prism_describe"`, the correct `client_id`, and an outcome tag.
///
/// RED GATE: Fails with todo!() panic from `handle_prism_describe`.
#[tokio::test]
async fn test_BC_2_10_012_prism_describe_audit_event_emitted() {
    let config_manager = make_config_manager_acme_crowdstrike();
    let audit_writer = CapturingAuditWriter::default();
    let audit_writer_arc: Arc<dyn AuditWriter> = Arc::new(audit_writer.clone());

    // Call handle_prism_describe with a real audit writer wired.
    let result = handle_prism_describe(
        "crowdstrike".to_string(),
        None,
        Some(&config_manager),
        Some(&audit_writer_arc),
    )
    .await;

    // The call must succeed.
    let call_result = result.expect(
        "BC-2.10.012 AC-002: handle_prism_describe must return Ok when audit writer is wired",
    );

    assert!(
        !call_result.is_error.unwrap_or(false),
        "BC-2.10.012 AC-002: prism_describe must succeed (not is_error) when audit writer is wired"
    );

    // BC-2.10.012 §Audit: at least one write_tool_call must have been invoked.
    let calls = audit_writer.calls.lock().unwrap();
    assert!(
        !calls.is_empty(),
        "BC-2.10.012 AC-002: audit_writer.write_tool_call must be invoked on every prism_describe \
         call; no calls were captured. The handler must call write_tool_call at invocation time."
    );

    // Find the call for prism_describe.
    let prism_describe_call = calls
        .iter()
        .find(|(tool_name, _, _)| tool_name == "prism_describe");
    assert!(
        prism_describe_call.is_some(),
        "BC-2.10.012 AC-002: write_tool_call must be invoked with tool_name='prism_describe'; \
         got calls: {:?}",
        calls.iter().map(|(t, _, _)| t).collect::<Vec<_>>()
    );

    let (_, client_id, outcome) = prism_describe_call.unwrap();

    // BC-2.10.012 §Audit: client_id must be passed.
    assert_eq!(
        client_id.as_deref(),
        Some("crowdstrike"),
        "BC-2.10.012 AC-002: write_tool_call must be invoked with client_id='crowdstrike'; \
         got: {:?}",
        client_id
    );

    // BC-2.10.012 §Audit: outcome must be a recognized tag (not empty).
    assert!(
        !outcome.is_empty(),
        "BC-2.10.012 AC-002: write_tool_call outcome must be non-empty (e.g. 'success', 'invoked'); \
         got empty string"
    );
}

// ─── AC-003: empty and unknown client_id handling ────────────────────────────

/// AC-003 (BC-2.10.012 — Non-existent/empty client handling):
/// When `prism_describe` is called for a client with zero sensor overlays (well-formed,
/// no tables configured), the response is `{tables: [], pql_hints: [...]}` with NO error.
/// When called for a valid-format but unregistered client, same success posture.
///
/// RED GATE: Fails with todo!() panic from `handle_prism_describe`.
#[tokio::test]
async fn test_BC_2_10_012_prism_describe_empty_and_unknown_client() {
    let empty_config = make_config_manager_empty();

    // Case 1: well-formed client_id "acme", zero tables (empty config).
    let result_empty =
        handle_prism_describe("acme".to_string(), None, Some(&empty_config), None).await;

    let empty_call = result_empty.expect(
        "BC-2.10.012 AC-003: prism_describe('acme') with empty config must return Ok (not error); \
         zero tables is a success case — not an error",
    );

    assert!(
        !empty_call.is_error.unwrap_or(false),
        "BC-2.10.012 AC-003: prism_describe for zero-table client must not return is_error=true"
    );

    let empty_text: String = empty_call
        .content
        .iter()
        .filter_map(|c| c.as_text().map(|t| t.text.clone()))
        .collect::<Vec<_>>()
        .join("");
    let empty_parsed: serde_json::Value = serde_json::from_str(&empty_text)
        .expect("BC-2.10.012 AC-003: zero-table response must be valid JSON");

    let empty_tables = empty_parsed
        .get("tables")
        .and_then(|v| v.as_array())
        .expect("BC-2.10.012 AC-003: zero-table response must contain 'tables' array");
    assert!(
        empty_tables.is_empty(),
        "BC-2.10.012 AC-003: zero-table response must have empty tables array; \
         got {} tables",
        empty_tables.len()
    );

    // pql_hints must be non-empty with a helpful message for zero-table clients.
    let empty_hints = empty_parsed
        .get("pql_hints")
        .and_then(|v| v.as_array())
        .expect("BC-2.10.012 AC-003: zero-table response must contain 'pql_hints' array");
    assert!(
        !empty_hints.is_empty(),
        "BC-2.10.012 AC-003: zero-table response must include at least one pql_hint \
         (e.g., 'No sensor tables are available for client ...')"
    );

    // The hint text must mention the client name or indicate no sensor tables.
    let hint_text = empty_hints
        .iter()
        .filter_map(|h| h.as_str())
        .collect::<Vec<_>>()
        .join(" ");
    assert!(
        hint_text.contains("acme") || hint_text.to_lowercase().contains("no sensor"),
        "BC-2.10.012 AC-003: hint for zero-table client must mention the client name or \
         indicate no sensor tables; got: {:?}",
        hint_text
    );

    // Case 2: valid-format client_id "nonexistent" — not in registry.
    let result_unknown =
        handle_prism_describe("nonexistent".to_string(), None, Some(&empty_config), None).await;

    let unknown_call = result_unknown.expect(
        "BC-2.10.012 AC-003: prism_describe('nonexistent') for unregistered client must \
         return Ok (not error); unknown clients return empty tables, not an error",
    );

    assert!(
        !unknown_call.is_error.unwrap_or(false),
        "BC-2.10.012 AC-003: prism_describe for unregistered client must not return is_error=true"
    );

    let unknown_text: String = unknown_call
        .content
        .iter()
        .filter_map(|c| c.as_text().map(|t| t.text.clone()))
        .collect::<Vec<_>>()
        .join("");
    let unknown_parsed: serde_json::Value = serde_json::from_str(&unknown_text)
        .expect("BC-2.10.012 AC-003: unknown-client response must be valid JSON");

    let unknown_tables = unknown_parsed
        .get("tables")
        .and_then(|v| v.as_array())
        .expect("BC-2.10.012 AC-003: unknown-client response must contain 'tables' array");
    assert!(
        unknown_tables.is_empty(),
        "BC-2.10.012 AC-003: unknown client must return empty tables; \
         got {} tables",
        unknown_tables.len()
    );
}

// ─── AC-003: invalid client_id (E-MCP-001) ───────────────────────────────────

/// AC-003 (BC-2.10.012 — Format validation, E-MCP-001):
/// When `prism_describe` is called with a path-traversal `client_id` such as
/// `"acme/../etc"`, the handler must return an `Err(ErrorData)` indicating
/// `E-MCP-001` — not a `todo!()` panic, and not a successful `Ok(CallToolResult)`.
///
/// DI-006: the raw path-traversal payload must NOT be echoed in the error message.
///
/// RED GATE: Fails with todo!() panic from `handle_prism_describe`.
#[tokio::test]
async fn test_BC_2_10_012_prism_describe_invalid_client_id() {
    let config_manager = make_config_manager_acme_crowdstrike();

    // Path-traversal client_id: must be rejected by OrgSlug::new() validation.
    let result =
        handle_prism_describe("acme/../etc".to_string(), None, Some(&config_manager), None).await;

    // BC-2.10.012: format validation failure returns Err(ErrorData) with E-MCP-001.
    assert!(
        result.is_err(),
        "BC-2.10.012 AC-003: prism_describe('acme/../etc') must return Err(ErrorData) for \
         invalid client_id format (E-MCP-001); got Ok(...)"
    );

    let err = result.unwrap_err();
    let err_msg = err.message.to_string();

    // DI-006 prompt-injection defense: the raw payload must NOT appear in the error message.
    assert!(
        !err_msg.contains("acme/../etc"),
        "BC-2.10.012 AC-003 DI-006: error message must NOT echo the raw path-traversal \
         client_id 'acme/../etc' (prompt-injection defense); got: {:?}",
        err_msg
    );

    // Error must indicate invalid format (without echoing the payload).
    assert!(
        err_msg.to_lowercase().contains("invalid")
            || err_msg.to_lowercase().contains("client_id")
            || err_msg.to_lowercase().contains("e-mcp-001"),
        "BC-2.10.012 AC-003: error message must indicate invalid client_id format; \
         got: {:?}",
        err_msg
    );
}

// ─── AC-004: client isolation (DI-008) ───────────────────────────────────────

/// AC-004 (BC-2.10.012 invariant DI-008 — Client isolation):
/// In a multi-tenant deployment with "crowdstrike" (crowdstrike_alerts) and
/// "claroty" (claroty_assets), calling `prism_describe("crowdstrike")` must return
/// ONLY crowdstrike table names. No claroty table names may appear in ANY field
/// of the response: tables, pql_hints, example_query strings, or column names.
///
/// RED GATE: Fails with todo!() panic from `handle_prism_describe`.
#[tokio::test]
async fn test_BC_2_10_012_prism_describe_client_isolation() {
    let config_manager = make_config_manager_two_tenants();

    // Call prism_describe for "crowdstrike" — must only see crowdstrike tables.
    let result =
        handle_prism_describe("crowdstrike".to_string(), None, Some(&config_manager), None).await;

    let call_result = result.expect(
        "BC-2.10.012 AC-004: prism_describe('crowdstrike') must return Ok in multi-tenant config",
    );

    let content_text: String = call_result
        .content
        .iter()
        .filter_map(|c| c.as_text().map(|t| t.text.clone()))
        .collect::<Vec<_>>()
        .join("");

    // DI-008: the entire response string must not contain claroty table names.
    assert!(
        !content_text.contains("claroty_assets"),
        "BC-2.10.012 AC-004 DI-008: prism_describe('crowdstrike') MUST NOT contain \
         claroty table 'claroty_assets' in any response field. \
         Full response: {:?}",
        content_text
    );

    // Positive assertion: crowdstrike tables must appear.
    assert!(
        content_text.contains("crowdstrike_alerts"),
        "BC-2.10.012 AC-004: prism_describe('crowdstrike') must contain the crowdstrike \
         table 'crowdstrike_alerts'. Full response: {:?}",
        content_text
    );

    // DI-008: claroty-specific column names must not leak.
    assert!(
        !content_text.contains("asset_name"),
        "BC-2.10.012 AC-004 DI-008: prism_describe('crowdstrike') MUST NOT contain \
         claroty column 'asset_name'. Full response: {:?}",
        content_text
    );

    // Mirror test: claroty client sees only claroty tables, not crowdstrike.
    let result_claroty =
        handle_prism_describe("claroty".to_string(), None, Some(&config_manager), None).await;

    let claroty_result = result_claroty.expect(
        "BC-2.10.012 AC-004: prism_describe('claroty') must return Ok in multi-tenant config",
    );

    let claroty_text: String = claroty_result
        .content
        .iter()
        .filter_map(|c| c.as_text().map(|t| t.text.clone()))
        .collect::<Vec<_>>()
        .join("");

    assert!(
        !claroty_text.contains("crowdstrike_alerts"),
        "BC-2.10.012 AC-004 DI-008: prism_describe('claroty') MUST NOT contain \
         crowdstrike table 'crowdstrike_alerts'. Full response: {:?}",
        claroty_text
    );
}

// ─── AC-005: prismql://schema/{client_id} parity with prism_describe ─────────

/// AC-005 (BC-2.10.013 — Resource template registration + parity):
/// `render_pql_schema_resource("crowdstrike", ...)` must produce JSON that is
/// structurally identical to `handle_prism_describe("crowdstrike", ...)`:
/// same client_id, same tables array length, same pql_hints array present.
///
/// Also verifies the URI template is registered by `build_resource_template_list`
/// with `mimeType: "application/json"`.
///
/// RED GATE: Fails with todo!() panic from `render_pql_schema_resource`.
#[tokio::test]
async fn test_BC_2_10_013_schema_resource_template_parity() {
    // Verify the URI template is registered in the resource template list.
    let template_list = prism_mcp::resources::build_resource_template_list();
    let has_schema_template = template_list
        .resource_templates
        .iter()
        .any(|t| t.uri_template.as_str() == URI_TEMPLATE_PQL_SCHEMA);
    assert!(
        has_schema_template,
        "BC-2.10.013 AC-005: 'prismql://schema/{{client_id}}' must appear in \
         list_resource_templates response; not found in template list"
    );

    // Verify the template has mimeType: "application/json".
    let schema_template = template_list
        .resource_templates
        .iter()
        .find(|t| t.uri_template.as_str() == URI_TEMPLATE_PQL_SCHEMA)
        .expect("BC-2.10.013 AC-005: prismql://schema/{client_id} template must exist");

    assert_eq!(
        schema_template.mime_type.as_deref(),
        Some("application/json"),
        "BC-2.10.013 AC-005: prismql://schema/{{client_id}} must have mimeType \
         'application/json'; got: {:?}",
        schema_template.mime_type
    );

    // Call render_pql_schema_resource and handle_prism_describe for the same client.
    let config_manager = make_config_manager_acme_crowdstrike();

    // RED GATE: render_pql_schema_resource will todo!() panic here.
    let resource_result = render_pql_schema_resource(
        "crowdstrike",
        None, // no query_engine → config_manager fallback
        Some(&config_manager),
    )
    .await
    .expect("BC-2.10.013 AC-005: render_pql_schema_resource must return Ok for valid client_id");

    let tool_result =
        handle_prism_describe("crowdstrike".to_string(), None, Some(&config_manager), None)
            .await
            .expect("BC-2.10.013 AC-005: handle_prism_describe must return Ok for parity check");

    // Extract JSON from both results.
    let resource_json: String = resource_result
        .contents
        .iter()
        .filter_map(|c| {
            if let rmcp::model::ResourceContents::TextResourceContents { text, .. } = c {
                Some(text.clone())
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
        .join("");

    let tool_json: String = tool_result
        .content
        .iter()
        .filter_map(|c| c.as_text().map(|t| t.text.clone()))
        .collect::<Vec<_>>()
        .join("");

    // BC-2.10.013 parity invariant: parse both and compare structural fields.
    let resource_parsed: serde_json::Value = serde_json::from_str(&resource_json)
        .expect("BC-2.10.013 AC-005: resource response must be valid JSON");
    let tool_parsed: serde_json::Value = serde_json::from_str(&tool_json)
        .expect("BC-2.10.013 AC-005: tool response must be valid JSON");

    // client_id must match.
    assert_eq!(
        resource_parsed.get("client_id"),
        tool_parsed.get("client_id"),
        "BC-2.10.013 AC-005 parity: resource and tool responses must have identical \
         client_id. resource: {:?}, tool: {:?}",
        resource_parsed.get("client_id"),
        tool_parsed.get("client_id")
    );

    // tables array length must match.
    let resource_tables = resource_parsed
        .get("tables")
        .and_then(|v| v.as_array())
        .expect("BC-2.10.013 AC-005 parity: resource response must have 'tables' array");
    let tool_tables = tool_parsed
        .get("tables")
        .and_then(|v| v.as_array())
        .expect("BC-2.10.013 AC-005 parity: tool response must have 'tables' array");

    assert_eq!(
        resource_tables.len(),
        tool_tables.len(),
        "BC-2.10.013 AC-005 parity: resource and tool responses must have same number of \
         tables. resource: {}, tool: {}",
        resource_tables.len(),
        tool_tables.len()
    );

    // pql_hints must be present in resource response.
    assert!(
        resource_parsed.get("pql_hints").is_some(),
        "BC-2.10.013 AC-005 parity: resource response must include 'pql_hints' (same structure \
         as prism_describe response)"
    );
}

// ─── AC-006: subscribe/notify per-client scoping ─────────────────────────────

/// AC-006 (BC-2.10.013 — Subscribe/notify; EC-10-029, EC-10-030):
/// The `SchemaSubscriberRegistry` correctly implements per-client subscription scoping:
/// - `subscribe` adds a handle for the given client
/// - `subscribers_for` returns only handles for the subscribed client (not other clients)
/// - `unsubscribe` removes the handle
/// - A change for "acme" MUST NOT notify "globex" subscribers (DI-008)
///
/// RED GATE: Fails with todo!() panic from `SchemaSubscriberRegistry::subscribe`,
/// `subscribers_for`, and `unsubscribe` in `resources/schema.rs`.
#[test]
fn test_BC_2_10_013_schema_resource_subscribe_notify() {
    let registry = SchemaSubscriberRegistry::new();
    let acme_slug = OrgSlug::new("acme").expect("'acme' is a valid OrgSlug");
    let globex_slug = OrgSlug::new("globex").expect("'globex' is a valid OrgSlug");

    // Subscribe "acme" with handle "conn-1".
    registry.subscribe(
        acme_slug.clone(),
        SubscriberHandle {
            id: "conn-1".to_string(),
        },
    );

    // Subscribe "globex" with handle "conn-2".
    registry.subscribe(
        globex_slug.clone(),
        SubscriberHandle {
            id: "conn-2".to_string(),
        },
    );

    // BC-2.10.013 EC-10-030: when a change fires for "acme", only acme's subscribers
    // should be notified. subscribers_for("acme") must return only acme's handles.
    let acme_subscribers = registry.subscribers_for(&acme_slug);
    assert_eq!(
        acme_subscribers.len(),
        1,
        "BC-2.10.013 AC-006: 'acme' should have exactly 1 subscriber; got {}",
        acme_subscribers.len()
    );
    assert!(
        acme_subscribers.contains(&"conn-1".to_string()),
        "BC-2.10.013 AC-006: acme's subscriber list must contain 'conn-1'; \
         got: {:?}",
        acme_subscribers
    );

    // DI-008: globex's subscriber must NOT appear in acme's subscriber list.
    assert!(
        !acme_subscribers.contains(&"conn-2".to_string()),
        "BC-2.10.013 AC-006 DI-008: 'conn-2' (globex subscriber) MUST NOT appear \
         in acme's subscriber list (per-client scoping prevents cross-tenant notification)"
    );

    // Globex subscribers are correctly isolated to globex scope.
    let globex_subscribers = registry.subscribers_for(&globex_slug);
    assert_eq!(
        globex_subscribers.len(),
        1,
        "BC-2.10.013 AC-006: 'globex' should have exactly 1 subscriber; got {}",
        globex_subscribers.len()
    );
    assert!(
        globex_subscribers.contains(&"conn-2".to_string()),
        "BC-2.10.013 AC-006: globex's subscriber list must contain 'conn-2'; \
         got: {:?}",
        globex_subscribers
    );

    // Unsubscribe "acme" conn-1 — should clear acme's list.
    registry.unsubscribe(&acme_slug, "conn-1");

    let acme_after = registry.subscribers_for(&acme_slug);
    assert!(
        acme_after.is_empty(),
        "BC-2.10.013 AC-006: after unsubscribe, 'acme' subscriber list must be empty; \
         got: {:?}",
        acme_after
    );

    // Globex subscriber must be unaffected by acme's unsubscribe (DI-008 isolation).
    let globex_after = registry.subscribers_for(&globex_slug);
    assert_eq!(
        globex_after.len(),
        1,
        "BC-2.10.013 AC-006: globex subscriber must not be affected by acme unsubscribe; \
         got {} subscribers",
        globex_after.len()
    );
    assert!(
        globex_after.contains(&"conn-2".to_string()),
        "BC-2.10.013 AC-006: globex 'conn-2' must still be subscribed after acme unsubscribe; \
         got: {:?}",
        globex_after
    );
}
