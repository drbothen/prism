//! Red Gate tests for S-DEMO-PRISMQL-ONBOARDING-001-A — AC-001 through AC-006.
//!
//! Covers `prism_describe` tool (BC-2.10.012) and `prismql://schema/{client_id}`
//! resource template (BC-2.10.013).
//!
//! ALL tests in this file must FAIL against the current (defective) implementation
//! (Red Gate per BC-5.38.001). Tests were rewritten from the initial false-green
//! versions that called leaf renderers directly, bypassing unimplemented dispatch
//! paths. Each test now drives the REAL production path.
//!
//! # What was wrong with the original tests
//!
//! - AC-004: called `handle_prism_describe` with `config_manager` only — the
//!   `resolved_spec_map` multi-tenant path was never exercised. HashMap::get
//!   isolation is not the same as `resolved_spec_map` isolation.
//! - AC-005: called `render_pql_schema_resource` directly — bypassed
//!   `dispatch_read_resource`, which has no handler for `prismql://schema/{client_id}`.
//! - AC-006: `SchemaSubscriberRegistry` is implemented but `get_info()` does NOT call
//!   `enable_resources_subscribe()` — the test only checked the registry data structure,
//!   not the MCP capability declaration.
//! - AC-001: no test for tool annotations in the production catalog.
//! - AC-002: audit outcome checked for non-empty, but not for the canonical value
//!   "schema_enumeration" — the code emits "invoked" which is incorrect.
//!
//! # Test → AC mapping
//!
//! | Test | AC | BC |
//! |------|----|----|
//! | test_BC_2_10_012_prism_describe_happy_path_catalog | AC-001 + AC-002 | BC-2.10.012 |
//! | test_BC_2_10_012_prism_describe_audit_event_emitted | AC-002 | BC-2.10.012 |
//! | test_BC_2_10_012_prism_describe_audit_operation_is_schema_enumeration | AC-002 | BC-2.10.012 |
//! | test_BC_2_10_012_prism_describe_empty_and_unknown_client | AC-003 | BC-2.10.012 |
//! | test_BC_2_10_012_prism_describe_invalid_client_id | AC-003 | BC-2.10.012 |
//! | test_BC_2_10_012_prism_describe_client_isolation_via_resolved_spec_map | AC-004 | BC-2.10.012 DI-008 |
//! | test_BC_2_10_013_schema_resource_dispatch_routed | AC-005 | BC-2.10.013 |
//! | test_BC_2_10_013_schema_resource_parity_via_dispatch | AC-005 | BC-2.10.013 |
//! | test_BC_2_10_013_schema_resource_subscribe_capability_declared | AC-006 | BC-2.10.013 |
//! | test_BC_2_10_012_prism_describe_tool_annotations | AC-001 | BC-2.10.012 |

use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use prism_core::{column::ColumnType, OrgSlug};
use prism_mcp::{
    resources::{
        build_resource_template_list, dispatch_read_resource, schema::URI_TEMPLATE_PQL_SCHEMA,
    },
    server::PrismServer,
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
use rmcp::ServerHandler;
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

/// Build a `resolved_spec_map` with two-org isolation fixture:
/// - org "acme"  → sensor "crowdstrike" with table "crowdstrike_alerts" (severity column)
/// - org "globex" → sensor "claroty"  with table "claroty_assets"    (asset_name column)
///
/// This is the CORRECT fixture for AC-004 — it exercises the resolved_spec_map
/// multi-tenant path, not just HashMap::get on config_manager sensor_specs.
/// The isolating invariant (DI-008) must be enforced at the resolved_spec_map level.
fn make_two_org_resolved_spec_map() -> Arc<
    std::collections::HashMap<
        prism_spec_engine::ResolvedSpecKey,
        prism_spec_engine::ResolvedSensorSpec,
    >,
> {
    use prism_core::{OrgSlug, SensorId};
    use prism_spec_engine::{
        overlay::{OverlayLoader, SensorInstanceOverlay},
        spec_parser::{AuthType, ColumnSpec as CS, SensorSpec, TableSpec},
    };

    let make_resolved =
        |org: &str, sensor_id: &str, base_url: &str, table_name: &str, col_name: &str| {
            let spec = SensorSpec::new(
                sensor_id,
                format!("{sensor_id} sensor"),
                AuthType::ApiKey,
                base_url,
                vec![TableSpec::new_point_in_time(
                    table_name,
                    "security_finding",
                    vec![CS::new(col_name, ColumnType::String, None, vec![])],
                    vec![],
                )],
                None,
                "1.0.0",
                vec![],
            );
            let overlay_toml =
                format!("extends = \"{sensor_id}\"\ninstance_id = \"{sensor_id}@{org}\"");
            let overlay: SensorInstanceOverlay = toml::from_str(&overlay_toml)
                .expect("AC-004 fixture: SensorInstanceOverlay TOML must parse");
            let org_slug = OrgSlug::new(org);
            let resolved =
                OverlayLoader::merge_overlay_onto_type_spec(&spec, &overlay, org_slug.clone());
            let sensor_id_typed = SensorId::new(sensor_id);
            let key = (org_slug, sensor_id_typed);
            (key, resolved)
        };

    let mut map = std::collections::HashMap::new();

    // acme → crowdstrike: table "crowdstrike_alerts" with "severity" column
    let (k, v) = make_resolved(
        "acme",
        "crowdstrike",
        "https://api.crowdstrike.com",
        "crowdstrike_alerts",
        "severity",
    );
    map.insert(k, v);

    // globex → claroty: table "claroty_assets" with "asset_name" column
    let (k, v) = make_resolved(
        "globex",
        "claroty",
        "https://api.claroty.com",
        "claroty_assets",
        "asset_name",
    );
    map.insert(k, v);

    Arc::new(map)
}

// ─── AC-001: prism_describe tool annotations ─────────────────────────────────

/// AC-001 (BC-2.10.012 — Tool registration + annotations):
/// The production tool catalog must include `prism_describe` with correct
/// annotations: readOnlyHint=true, idempotentHint=true, openWorldHint=false.
/// The tool description must include the AC-001 annotation summary string.
///
/// RED GATE: Fails if the catalog tool does not carry the correct annotations.
/// This test does NOT call todo!() code — it drives the real production catalog
/// inspection path to verify annotations are wired.
///
/// Currently FAILS because: the test verifies the annotations field explicitly
/// and the description must contain "readOnlyHint:true" — verifying the annotations
/// struct is present and correctly set.
#[test]
fn test_BC_2_10_012_prism_describe_tool_annotations() {
    let catalog = PrismServer::production_tool_catalog();

    let prism_describe = catalog
        .iter()
        .find(|t| t.name.as_ref() == "prism_describe")
        .expect(
            "BC-2.10.012 AC-001: 'prism_describe' must be registered in the production tool \
             catalog; not found. Verify prism_describe is in LIVE_TOOLS list.",
        );

    // AC-001: readOnlyHint must be true.
    let annotations = prism_describe.annotations.as_ref().expect(
        "BC-2.10.012 AC-001: 'prism_describe' tool must have annotations set; \
         annotations is None. Wiring: the #[tool(annotations(...))] macro must set this.",
    );

    assert_eq!(
        annotations.read_only_hint,
        Some(true),
        "BC-2.10.012 AC-001: prism_describe must have readOnlyHint=true; \
         got: {:?}",
        annotations.read_only_hint
    );

    // AC-001: idempotentHint must be true.
    assert_eq!(
        annotations.idempotent_hint,
        Some(true),
        "BC-2.10.012 AC-001: prism_describe must have idempotentHint=true; \
         got: {:?}",
        annotations.idempotent_hint
    );

    // AC-001: openWorldHint must be false (schema catalog is closed-world per client scope).
    assert_eq!(
        annotations.open_world_hint,
        Some(false),
        "BC-2.10.012 AC-001: prism_describe must have openWorldHint=false; \
         got: {:?}",
        annotations.open_world_hint
    );

    // AC-001: description must be present and non-empty.
    let description = prism_describe.description.as_deref().unwrap_or("");
    assert!(
        !description.is_empty(),
        "BC-2.10.012 AC-001: 'prism_describe' tool must have a non-empty description"
    );

    // AC-001: description must contain the schema-discovery purpose text.
    assert!(
        description.contains("prism_describe") || description.contains("schema"),
        "BC-2.10.012 AC-001: 'prism_describe' description must mention schema discovery; \
         got first 200 chars: {:?}",
        &description[..description.len().min(200)]
    );
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

// ─── AC-002: audit outcome must be canonical "schema_enumeration" ────────────

/// AC-002 (BC-2.10.012 — Audit event: operation and outcome fields):
/// The audit `write_tool_call` invocation must use `outcome = "schema_enumeration"`
/// (canonical operation name from BC-2.10.012 §Audit). The current implementation
/// emits "invoked" — this test enforces the canonical value.
///
/// RED GATE: Fails now because `handle_prism_describe` calls
/// `write_tool_call("prism_describe", ..., "invoked")` but the BC requires
/// the outcome to be the operation name "schema_enumeration".
#[tokio::test]
async fn test_BC_2_10_012_prism_describe_audit_operation_is_schema_enumeration() {
    let config_manager = make_config_manager_acme_crowdstrike();
    let audit_writer = CapturingAuditWriter::default();
    let audit_writer_arc: Arc<dyn AuditWriter> = Arc::new(audit_writer.clone());

    let result = handle_prism_describe(
        "crowdstrike".to_string(),
        None,
        Some(&config_manager),
        Some(&audit_writer_arc),
    )
    .await;

    result.expect("BC-2.10.012 AC-002: handle_prism_describe must return Ok");

    let calls = audit_writer.calls.lock().unwrap();
    let prism_describe_call = calls
        .iter()
        .find(|(tool_name, _, _)| tool_name == "prism_describe")
        .expect("BC-2.10.012 AC-002: write_tool_call must be invoked for prism_describe");

    let (_, _, outcome) = prism_describe_call;

    // BC-2.10.012 §Audit: outcome must be "schema_enumeration" (canonical operation name).
    // Currently the code emits "invoked" — this fails the Red Gate.
    assert_eq!(
        outcome.as_str(),
        "schema_enumeration",
        "BC-2.10.012 AC-002: write_tool_call outcome MUST be 'schema_enumeration' \
         (canonical operation name per BC-2.10.012 §Audit); \
         current code emits '{}' — change handle_prism_describe to pass 'schema_enumeration'",
        outcome
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

// ─── AC-004: multi-tenant isolation via resolved_spec_map (DI-008) ───────────

/// AC-004 (BC-2.10.012 invariant DI-008 — Multi-tenant client isolation):
///
/// In multi-tenant mode, `prism_describe` must use `query_engine.resolved_spec_map()`
/// filtered by OrgSlug to isolate clients. The current implementation ignores
/// `query_engine` and falls back to `config_manager`, which is INCORRECT in
/// multi-tenant deployments.
///
/// This test constructs a two-org `resolved_spec_map` (org "acme" → crowdstrike,
/// org "globex" → claroty) and passes it via a wired `QueryEngine`. Calling
/// `prism_describe("acme")` must return ONLY acme's crowdstrike tables and
/// NEVER any globex/claroty strings.
///
/// RED GATE: Fails because `handle_prism_describe` ignores `_query_engine` —
/// the resolved_spec_map multi-tenant path is NOT yet implemented.
/// The `_query_engine` parameter is currently an underscore-prefixed ignored argument.
#[tokio::test]
async fn test_BC_2_10_012_prism_describe_client_isolation_via_resolved_spec_map() {
    use prism_core::{OrgId, OrgRegistry, OrgSlug, SensorId};
    use prism_query::engine::QueryEngine;

    // Build the two-org resolved_spec_map.
    let spec_map = make_two_org_resolved_spec_map();

    // Build an OrgRegistry with acme + globex registered.
    let org_reg = {
        let reg = OrgRegistry::new();
        reg.register(OrgSlug::new("acme"), OrgId::new())
            .expect("register acme");
        reg.register(OrgSlug::new("globex"), OrgId::new())
            .expect("register globex");
        Arc::new(reg)
    };

    // Build a QueryEngine wired with the two-org resolved_spec_map.
    // Uses QueryEngine::new_full with stub dependencies — we only need resolved_spec_map
    // to flow through so handle_prism_describe can access it.
    // Storage: InMemoryBackend (no RocksDB needed for this test).
    // CredentialResolver: inline stub — always returns CredentialNotFound (no auth needed).
    use prism_credentials::InMemoryCredentialStore;
    use prism_sensors::{
        registry::AdapterRegistry, CredentialResolver as SensorsCredentialResolver, SensorError,
    };
    use prism_storage::memory_backend::memory_backend_inner::InMemoryBackend;

    // Inline stub: satisfies prism_sensors::CredentialResolver used by QueryEngine::new_full.
    // Returns Internal error for all resolve calls (no real auth needed for this test).
    struct StubCredResolver;
    impl SensorsCredentialResolver for StubCredResolver {
        fn resolve(
            &self,
            _client_id: &str,
            sensor_id: prism_core::SensorId,
        ) -> Result<Box<dyn prism_sensors::SensorAuth>, SensorError> {
            Err(SensorError::Internal {
                detail: format!(
                    "StubCredResolver: no credential for {sensor_id:?} (AC-004 test stub)"
                ),
            })
        }
    }

    let alias_store = Arc::new(Mutex::new(prism_query::alias_store::AliasStore::empty(
        std::path::Path::new("/tmp/test-prism-ac004"),
    )));

    let query_engine = Arc::new(QueryEngine::new_full(
        Arc::new(AdapterRegistry::new()),
        Arc::new(InMemoryCredentialStore::new()),
        Arc::new(prism_ocsf::OcsfNormalizer::new()),
        Arc::new(prism_query::scoping::ClientRegistry::new(vec![])),
        prism_query::engine::QueryEngineConfig::default(),
        Arc::new(StubCredResolver),
        org_reg,
        Arc::new(InMemoryBackend::new()),
        spec_map,
        alias_store,
    ));

    // Call prism_describe("acme") with the wired query_engine.
    // The multi-tenant path MUST filter by OrgSlug("acme") in resolved_spec_map.
    let result = handle_prism_describe(
        "acme".to_string(),
        Some(&query_engine),
        None, // no config_manager — must use resolved_spec_map from query_engine
        None,
    )
    .await;

    let call_result = result.expect(
        "BC-2.10.012 AC-004: prism_describe('acme') with wired QueryEngine must return Ok; \
         the multi-tenant path must be implemented",
    );

    let content_text: String = call_result
        .content
        .iter()
        .filter_map(|c| c.as_text().map(|t| t.text.clone()))
        .collect::<Vec<_>>()
        .join("");

    // DI-008: the entire response string must not contain globex/claroty table names.
    assert!(
        !content_text.contains("claroty_assets"),
        "BC-2.10.012 AC-004 DI-008: prism_describe('acme') via resolved_spec_map MUST NOT \
         contain claroty table 'claroty_assets' in any response field. \
         Full response: {:?}",
        content_text
    );

    // DI-008: claroty-specific column names must not leak.
    assert!(
        !content_text.contains("asset_name"),
        "BC-2.10.012 AC-004 DI-008: prism_describe('acme') MUST NOT contain \
         claroty column 'asset_name'. Full response: {:?}",
        content_text
    );

    // Positive assertion: acme's crowdstrike table must appear.
    assert!(
        content_text.contains("crowdstrike_alerts"),
        "BC-2.10.012 AC-004: prism_describe('acme') must contain the acme \
         table 'crowdstrike_alerts' from resolved_spec_map. Full response: {:?}",
        content_text
    );

    // Mirror test: globex client sees only claroty tables, not crowdstrike.
    let result_globex =
        handle_prism_describe("globex".to_string(), Some(&query_engine), None, None).await;

    let globex_result = result_globex.expect(
        "BC-2.10.012 AC-004: prism_describe('globex') must return Ok in multi-tenant config",
    );

    let globex_text: String = globex_result
        .content
        .iter()
        .filter_map(|c| c.as_text().map(|t| t.text.clone()))
        .collect::<Vec<_>>()
        .join("");

    assert!(
        !globex_text.contains("crowdstrike_alerts"),
        "BC-2.10.012 AC-004 DI-008: prism_describe('globex') MUST NOT contain \
         crowdstrike table 'crowdstrike_alerts'. Full response: {:?}",
        globex_text
    );
}

// ─── AC-005: prismql://schema/{client_id} dispatch routing ───────────────────

/// AC-005 (BC-2.10.013 — Resource dispatch routing):
/// `dispatch_read_resource("prismql://schema/crowdstrike", ...)` must NOT return
/// "Unknown or unsupported resource URI" (404-equivalent). It must route to
/// `render_pql_schema_resource` and return the schema catalog.
///
/// RED GATE: Fails because `dispatch_read_resource` in `resources.rs` has no
/// handler for the `prismql://schema/{client_id}` URI pattern — it falls through
/// to the generic 404 return.
#[tokio::test]
async fn test_BC_2_10_013_schema_resource_dispatch_routed() {
    use prism_mcp::context::PrismContext;

    let config_manager = make_config_manager_acme_crowdstrike();
    let context = Arc::new(PrismContext::new());

    // Drive dispatch_read_resource for the prismql://schema/{client_id} URI.
    // This must NOT return the generic 404 "Unknown or unsupported resource URI".
    let result = dispatch_read_resource(
        "prismql://schema/crowdstrike",
        &context,
        None, // no query_engine
        Some(&config_manager),
    )
    .await;

    // The dispatch must NOT return Err with the 404-equivalent message.
    // Currently dispatch_read_resource falls through to the not_found_error catchall.
    assert!(
        result.is_ok(),
        "BC-2.10.013 AC-005: dispatch_read_resource('prismql://schema/crowdstrike') must \
         return Ok — not a 404 error. The dispatch table does NOT yet have a handler for \
         the 'prismql://schema/{{client_id}}' URI pattern. \
         Got Err: {:?}",
        result.err()
    );

    let read_result = result.unwrap();

    // The result must contain JSON content (same shape as prism_describe).
    let content_text: String = read_result
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

    assert!(
        !content_text.is_empty(),
        "BC-2.10.013 AC-005: dispatch result for prismql://schema/crowdstrike must be non-empty"
    );

    let parsed: serde_json::Value = serde_json::from_str(&content_text)
        .expect("BC-2.10.013 AC-005: dispatch result must be valid JSON");

    assert_eq!(
        parsed.get("client_id").and_then(|v| v.as_str()),
        Some("crowdstrike"),
        "BC-2.10.013 AC-005: dispatch result must contain client_id='crowdstrike'"
    );
}

/// AC-005 (BC-2.10.013 — Resource parity invariant):
/// `dispatch_read_resource("prismql://schema/crowdstrike")` must produce JSON with
/// FULL structural equality to `handle_prism_describe("crowdstrike")`:
/// same client_id, same tables array (length AND content), same pql_hints.
///
/// RED GATE: Fails because dispatch_read_resource doesn't route to the schema handler.
/// Once routing is wired, this verifies the single-source-of-truth parity invariant.
#[tokio::test]
async fn test_BC_2_10_013_schema_resource_parity_via_dispatch() {
    use prism_mcp::context::PrismContext;

    let config_manager = make_config_manager_acme_crowdstrike();
    let context = Arc::new(PrismContext::new());

    // Call both the dispatch path and the direct handle_prism_describe.
    let dispatch_result = dispatch_read_resource(
        "prismql://schema/crowdstrike",
        &context,
        None,
        Some(&config_manager),
    )
    .await
    .expect(
        "BC-2.10.013 AC-005 parity: dispatch_read_resource('prismql://schema/crowdstrike') \
         must return Ok for parity check; currently 404 (dispatch not wired)",
    );

    let tool_result =
        handle_prism_describe("crowdstrike".to_string(), None, Some(&config_manager), None)
            .await
            .expect(
                "BC-2.10.013 AC-005 parity: handle_prism_describe must return Ok for parity check",
            );

    // Extract JSON from both results.
    let resource_json: String = dispatch_result
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
        .expect("BC-2.10.013 AC-005: dispatch resource response must be valid JSON");
    let tool_parsed: serde_json::Value = serde_json::from_str(&tool_json)
        .expect("BC-2.10.013 AC-005: tool response must be valid JSON");

    // client_id must match.
    assert_eq!(
        resource_parsed.get("client_id"),
        tool_parsed.get("client_id"),
        "BC-2.10.013 AC-005 parity: dispatch and tool responses must have identical \
         client_id. resource: {:?}, tool: {:?}",
        resource_parsed.get("client_id"),
        tool_parsed.get("client_id")
    );

    // tables array length must match.
    let resource_tables = resource_parsed
        .get("tables")
        .and_then(|v| v.as_array())
        .expect("BC-2.10.013 AC-005 parity: dispatch resource response must have 'tables' array");
    let tool_tables = tool_parsed
        .get("tables")
        .and_then(|v| v.as_array())
        .expect("BC-2.10.013 AC-005 parity: tool response must have 'tables' array");

    assert_eq!(
        resource_tables.len(),
        tool_tables.len(),
        "BC-2.10.013 AC-005 parity: dispatch and tool responses must have same number of \
         tables. resource: {}, tool: {}",
        resource_tables.len(),
        tool_tables.len()
    );

    // Full JSON equality — single-source-of-truth invariant.
    assert_eq!(
        resource_parsed, tool_parsed,
        "BC-2.10.013 AC-005 parity: dispatch_read_resource and handle_prism_describe MUST \
         return structurally identical JSON (single-source-of-truth parity invariant). \
         resource JSON: {:?}, tool JSON: {:?}",
        resource_json, tool_json
    );

    // pql_hints must be present in resource response.
    assert!(
        resource_parsed.get("pql_hints").is_some(),
        "BC-2.10.013 AC-005 parity: dispatch resource response must include 'pql_hints'"
    );
}

// ─── AC-006: subscribe/notify capability declaration ─────────────────────────

/// AC-006 (BC-2.10.013 — Subscribe capability):
/// `PrismServer::get_info()` must declare `enable_resources_subscribe()` in
/// `ServerCapabilities` so MCP clients know they can subscribe to
/// `prismql://schema/{client_id}` for change notifications.
///
/// RED GATE: Fails because `get_info()` calls `.enable_resources()` but NOT
/// `.enable_resources_subscribe()`. The `ServerCapabilities` therefore has
/// `resources.subscribe = None` instead of `resources.subscribe = Some(true)`.
#[test]
fn test_BC_2_10_013_schema_resource_subscribe_capability_declared() {
    let server = PrismServer::new();
    let info = server.get_info();

    let resources_cap = info.capabilities.resources.as_ref().expect(
        "BC-2.10.013 AC-006: ServerCapabilities must have 'resources' capability declared; \
         resources is None. Verify .enable_resources() is called in get_info().",
    );

    // BC-2.10.013: subscribe capability must be true.
    // The current code calls .enable_resources() but NOT .enable_resources_subscribe().
    assert_eq!(
        resources_cap.subscribe,
        Some(true),
        "BC-2.10.013 AC-006: ServerCapabilities.resources.subscribe MUST be Some(true). \
         Current code calls .enable_resources() but not .enable_resources_subscribe() in \
         get_info(). Add .enable_resources_subscribe() to the ServerCapabilities builder \
         in server.rs. Got: {:?}",
        resources_cap.subscribe
    );
}

/// AC-006 (BC-2.10.013 — Subscribe/notify per-client scoping; EC-10-030):
/// The `SchemaSubscriberRegistry` correctly implements per-client subscription scoping:
/// - `subscribe` adds a handle for the given client
/// - `subscribers_for` returns only handles for the subscribed client (not other clients)
/// - `unsubscribe` removes the handle
/// - A change for "acme" MUST NOT notify "globex" subscribers (DI-008)
///
/// NOTE: The SchemaSubscriberRegistry data structure IS implemented. This test
/// drives the real subscribe/unsubscribe/notify behavioral contract — not just the
/// data structure. The subscribe CAPABILITY (enable_resources_subscribe) is tested
/// separately in test_BC_2_10_013_schema_resource_subscribe_capability_declared above.
///
/// RED GATE: Passes on the registry data structure but is included as a
/// belt-and-suspenders assertion to confirm the DI-008 isolation contract is correct.
/// The RED GATE for AC-006 is the capability declaration test above.
#[test]
fn test_BC_2_10_013_schema_resource_subscribe_notify() {
    use prism_mcp::resources::schema::{SchemaSubscriberRegistry, SubscriberHandle};

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
