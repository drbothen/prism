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
//! - AC-003 (round-1, original): called `handle_prism_describe` with only `config_manager`,
//!   never wiring `query_engine`. The test was lenient — it asserted pql_hints non-empty
//!   and vaguely mentioned "acme" or "no sensor", but did NOT assert the BC-mandated
//!   not-registered hint for "nonexistent". BC-2.10.012 requires TWO DISTINCT strings:
//!   (a) registered-but-empty → "No sensor tables…sensor overlays configured."
//!   (b) not-registered → "'client' is not registered. Check prism.toml [[orgs]]."
//!   The old test let the implementation emit one generic hint for both cases.
//!   The hardened test wires `query_engine` with `org_registry` containing only "acme",
//!   asserts the BC-canonical strings for both cases, and FAILS on the not-registered
//!   assertion because `build_pql_hints` does not consult `org_registry`.
//! - AC-004: called `handle_prism_describe` with `config_manager` only — the
//!   `resolved_spec_map` multi-tenant path was never exercised. HashMap::get
//!   isolation is not the same as `resolved_spec_map` isolation.
//! - AC-005: called `render_pql_schema_resource` directly — bypassed
//!   `dispatch_read_resource`, which has no handler for `prismql://schema/{client_id}`.
//! - AC-006 (round-1): `SchemaSubscriberRegistry` is implemented but `get_info()` does
//!   NOT call `enable_resources_subscribe()` — the test only checked the registry data
//!   structure, not the MCP capability declaration.
//! - AC-006 (round-2, this hardening): `test_BC_2_10_013_schema_resource_subscribe_notify`
//!   only verified the registry data structure (subscribe/unsubscribe/subscribers_for).
//!   It did NOT test real notification dispatch. `notify_schema_updated` is a stub that
//!   only logs — it never calls any notification target. `SubscriberHandle` has no
//!   `notifier` field. The test passed vacuously on the data structure.
//! - AC-001: no test for tool annotations in the production catalog.
//! - AC-002 (round-1): audit outcome checked for non-empty / "schema_enumeration" string,
//!   but this conflates `operation` with `outcome`. BC-2.10.012 v1.1 requires BOTH
//!   `operation = "schema_enumeration"` AND `outcome = "success"|"error"` as separate fields.
//!   The old test `test_BC_2_10_012_prism_describe_audit_operation_is_schema_enumeration`
//!   was checking `outcome == "schema_enumeration"` which is the operation name — not the
//!   outcome. The test was green because the code was also wrong in the same way.
//!
//! # Test → AC mapping (current)
//!
//! | Test | AC | BC |
//! |------|----|----|
//! | test_BC_2_10_012_prism_describe_tool_annotations | AC-001 | BC-2.10.012 |
//! | test_BC_2_10_012_prism_describe_happy_path_catalog | AC-001 + AC-002 | BC-2.10.012 |
//! | test_BC_2_10_012_prism_describe_audit_event_emitted | AC-002 (basic) | BC-2.10.012 |
//! | test_BC_2_10_012_prism_describe_audit_operation_and_outcome_happy_path | AC-002 (hardened) | BC-2.10.012 v1.1 |
//! | test_BC_2_10_012_prism_describe_audit_outcome_error_on_invalid_client_id | AC-002 (hardened) | BC-2.10.012 v1.1 |
//! | test_BC_2_10_012_prism_describe_empty_and_unknown_client | AC-003 (hardened) | BC-2.10.012 |
//! | test_BC_2_10_012_prism_describe_invalid_client_id | AC-003 | BC-2.10.012 |
//! | test_BC_2_10_012_prism_describe_client_isolation_via_resolved_spec_map | AC-004 | BC-2.10.012 DI-008 |
//! | test_BC_2_10_013_schema_resource_dispatch_routed | AC-005 | BC-2.10.013 |
//! | test_BC_2_10_013_schema_resource_parity_via_dispatch | AC-005 | BC-2.10.013 |
//! | test_BC_2_10_013_schema_resource_subscribe_capability_declared | AC-006 | BC-2.10.013 |
//! | test_BC_2_10_013_schema_resource_subscribe_notify | AC-006 (registry isolation) | BC-2.10.013 |
//! | test_BC_2_10_013_schema_resource_notify_dispatch_per_client_scoped | AC-006 (notify dispatch) | BC-2.10.013 EC-10-029/030 |
//! | test_BC_2_10_013_schema_resource_production_path_reload_triggers_notify | AC-006 (production path) | BC-2.10.013 EC-10-029/030 |

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
///
/// # BC-2.10.012 v1.1 extended capture (AC-002 hardening)
///
/// BC-2.10.012 v1.1: `write_tool_call` carries BOTH `operation` (the canonical
/// operation name, e.g. `"schema_enumeration"`) AND `outcome` (`"success"` or
/// `"error"`) as SEPARATE parameters — implemented in the production trait.
///
/// This mock captures a 4-tuple `(tool_name, client_id, operation, outcome)`.
///
/// The AC-002 assertions verify `operation == "schema_enumeration"` AND
/// `outcome == "success"` (happy path) / `outcome == "error"` (error path).
// (type alias below: clippy requires no blank doc-comment line before a non-doc item)
type AuditRecord = (String, Option<String>, String, String);

#[derive(Clone, Default)]
struct CapturingAuditWriter {
    /// Captured (tool_name, client_id, operation, outcome) tuples from write_tool_call.
    ///
    /// - `operation`: the canonical operation name (BC-2.10.012 §Audit).
    /// - `outcome`: `"success"` | `"error"` | `"(not_provided)"` sentinel when the
    ///   production trait does not yet carry a separate `outcome` parameter.
    calls: Arc<Mutex<Vec<AuditRecord>>>,
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

    /// Captures `(tool_name, client_id, operation, outcome)`.
    ///
    /// BC-2.10.012 v1.1: the production trait now carries BOTH `operation`
    /// (canonical operation name, e.g. `"schema_enumeration"`) AND `outcome`
    /// (result: `"success"` or `"error"`) as separate parameters.
    async fn write_tool_call(
        &self,
        tool_name: &str,
        client_id: Option<&str>,
        operation: &str,
        outcome: &str,
    ) -> Result<(), prism_core::error::PrismError> {
        self.calls.lock().unwrap().push((
            tool_name.to_string(),
            client_id.map(|s| s.to_string()),
            operation.to_string(),
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
/// Load-bearing: if `prism_describe` loses its annotations (readOnlyHint, idempotentHint,
/// openWorldHint) or the description drops the AC-001 annotation summary string,
/// this test fails.
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

    // AC-001: description must contain the exact AC-001 purpose phrase.
    // Canonical literal from BC-2.10.012 AC-001 and the story spec:
    // "Call this tool before writing a PrismQL query to discover which tables and columns
    //  are available."
    // This is a load-bearing assertion — the phrase guides agent behaviour (when to call
    // prism_describe) and must not degrade to a generic "schema" mention.
    assert!(
        description.contains(
            "Call this tool before writing a PrismQL query to discover which tables and columns are available."
        ),
        "BC-2.10.012 AC-001: 'prism_describe' description must contain the exact AC-001 phrase \
         \"Call this tool before writing a PrismQL query to discover which tables and columns \
         are available.\"; not found. \
         Got first 400 chars: {:?}",
        &description[..description.len().min(400)]
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
/// Load-bearing: `handle_prism_describe` is fully implemented; removing it causes a
/// compile/link failure.
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

    // SafetyEnvelope: domain payload is under `results`.
    // (SafetyEnvelopeBuilder::wrap places the PrismDescribeResponse under results.)
    let results = parsed.get("results").expect(
        "BC-2.10.012 AC-002: SafetyEnvelope response must have 'results' field; \
         ensure handle_prism_describe uses SafetyEnvelopeBuilder::wrap",
    );

    // AC-002: client_id field must match the requested client.
    assert_eq!(
        results.get("client_id").and_then(|v| v.as_str()),
        Some("crowdstrike"),
        "BC-2.10.012 AC-002: response results.client_id must be 'crowdstrike'; \
         got: {:?}",
        results.get("client_id")
    );

    // AC-002: tables array must have exactly 3 entries (alerts, devices, events).
    let tables = results
        .get("tables")
        .and_then(|v| v.as_array())
        .expect("BC-2.10.012 AC-002: response results must contain a 'tables' array");
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
    let pql_hints = results
        .get("pql_hints")
        .and_then(|v| v.as_array())
        .expect("BC-2.10.012 AC-002: response results must contain a 'pql_hints' array");
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
/// Load-bearing: `handle_prism_describe` emits the audit event; removing the
/// `write_tool_call` call causes `calls.len() == 0` and this test fails.
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
        .find(|(tool_name, _, _, _)| tool_name == "prism_describe");
    assert!(
        prism_describe_call.is_some(),
        "BC-2.10.012 AC-002: write_tool_call must be invoked with tool_name='prism_describe'; \
         got calls: {:?}",
        calls.iter().map(|(t, _, _, _)| t).collect::<Vec<_>>()
    );

    let (_, client_id, _operation, _outcome) = prism_describe_call.unwrap();

    // BC-2.10.012 §Audit: client_id must be passed.
    assert_eq!(
        client_id.as_deref(),
        Some("crowdstrike"),
        "BC-2.10.012 AC-002: write_tool_call must be invoked with client_id='crowdstrike'; \
         got: {:?}",
        client_id
    );
}

// ─── AC-002 (hardened): audit operation AND outcome as separate fields ────────

/// AC-002 (BC-2.10.012 v1.1 — Audit event: operation AND outcome as separate fields):
///
/// BC-2.10.012 v1.1 requires the `write_tool_call` audit record to carry TWO
/// distinct fields:
///   - `operation = "schema_enumeration"` — the canonical operation name
///   - `outcome = "success"` — the result on the happy path
///
/// The production `write_tool_call` signature carries both `operation` and `outcome` as
/// separate parameters; `handle_prism_describe` passes each on its happy path.
///
/// Load-bearing (TD-VSDD-059): removing the `outcome` parameter from `write_tool_call`
/// or passing `outcome = "(not_provided)"` causes the `outcome == "success"` assertion
/// to fail.
#[tokio::test]
async fn test_BC_2_10_012_prism_describe_audit_operation_and_outcome_happy_path() {
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

    result.expect("BC-2.10.012 AC-002: handle_prism_describe must return Ok on happy path");

    let calls = audit_writer.calls.lock().unwrap();
    let prism_describe_call = calls
        .iter()
        .find(|(tool_name, _, _, _)| tool_name == "prism_describe")
        .expect("BC-2.10.012 AC-002: write_tool_call must be invoked for prism_describe");

    let (_, client_id_cap, operation, outcome) = prism_describe_call;

    // client_id must be propagated.
    assert_eq!(
        client_id_cap.as_deref(),
        Some("crowdstrike"),
        "BC-2.10.012 AC-002: write_tool_call must carry client_id='crowdstrike'; got: {:?}",
        client_id_cap
    );

    // BC-2.10.012 v1.1 §Audit: operation must be "schema_enumeration".
    // This assertion PASSES because current code passes "schema_enumeration" as its
    // single outcome arg, which CapturingAuditWriter maps to `operation`.
    assert_eq!(
        operation.as_str(),
        "schema_enumeration",
        "BC-2.10.012 AC-002: write_tool_call operation MUST be 'schema_enumeration'; \
         got operation='{}'",
        operation
    );

    // BC-2.10.012 v1.1 §Audit: outcome must be "success" on the happy path.
    // Load-bearing: removing the `outcome` parameter from write_tool_call (or passing
    // only one string) causes outcome = "(not_provided)" and this assertion fails.
    assert_eq!(
        outcome.as_str(),
        "success",
        "BC-2.10.012 AC-002: write_tool_call outcome MUST be 'success' on the happy path \
         (BC-2.10.012 v1.1 §Audit). Got: '{}'",
        outcome
    );
}

/// AC-002 (BC-2.10.012 v1.1 — Audit event: outcome = "error" on E-MCP-001 path):
///
/// When `prism_describe` is called with an invalid `client_id` (E-MCP-001), the
/// audit record MUST carry `outcome = "error"`.
///
/// BC-2.10.012 v1.1: "on invalid client_id, the audit emission still occurs
/// (fail-open DI-004) with `outcome = 'error'`."
///
/// On validation failure (E-MCP-001), `handle_prism_describe` calls `write_tool_call`
/// with `operation = "schema_enumeration"` and `outcome = "error"` BEFORE returning
/// the `Err(ErrorData)` (fail-open DI-004, audit before return).
///
/// Load-bearing (TD-VSDD-059): removing the audit call in the validation-failure branch
/// of `handle_prism_describe` causes `calls.len() == 0` instead of 1.
#[tokio::test]
async fn test_BC_2_10_012_prism_describe_audit_outcome_error_on_invalid_client_id() {
    let config_manager = make_config_manager_acme_crowdstrike();
    let audit_writer = CapturingAuditWriter::default();
    let audit_writer_arc: Arc<dyn AuditWriter> = Arc::new(audit_writer.clone());

    // Path-traversal client_id: must be rejected with E-MCP-001.
    let result = handle_prism_describe(
        "acme/../etc".to_string(),
        None,
        Some(&config_manager),
        Some(&audit_writer_arc),
    )
    .await;

    // BC-2.10.012 AC-003: format validation returns Err.
    assert!(
        result.is_err(),
        "BC-2.10.012 AC-002: handle_prism_describe('acme/../etc') must return Err(ErrorData) \
         for invalid client_id format"
    );

    // BC-2.10.012 v1.1 §Audit: write_tool_call MUST be invoked even on validation failure
    // (audit before return, fail-open DI-004).
    let calls = audit_writer.calls.lock().unwrap();
    assert_eq!(
        calls.len(),
        1,
        "BC-2.10.012 AC-002: write_tool_call MUST be called even on validation \
         failure (E-MCP-001), with outcome='error'. Got {} calls instead of 1.",
        calls.len()
    );

    let (_, _client_id_cap, operation, outcome) = &calls[0];

    assert_eq!(
        operation.as_str(),
        "schema_enumeration",
        "BC-2.10.012 AC-002: write_tool_call operation MUST be 'schema_enumeration' \
         even on E-MCP-001 path; got: '{}'",
        operation
    );

    assert_eq!(
        outcome.as_str(),
        "error",
        "BC-2.10.012 AC-002: write_tool_call outcome MUST be 'error' on the \
         E-MCP-001 invalid-client_id path (BC-2.10.012 v1.1 §Audit). \
         Got outcome='{}'.",
        outcome
    );
}

// ─── AC-003: empty and unknown client_id handling ────────────────────────────

/// AC-003 (BC-2.10.012 §Non-existent client_id handling — TWO DISTINCT pql_hints):
///
/// BC-2.10.012 mandates two semantically DIFFERENT pql_hint strings depending on
/// whether the client is registered in OrgRegistry but has no sensor overlays, or
/// is not registered at all.
///
/// ## Case 1 — Registered-but-empty (OrgRegistry KNOWS "acme", zero resolved tables):
///
/// pql_hints MUST contain:
///   `"No sensor tables are available for client 'acme'. The client may not have \
///    any sensor overlays configured."`
///
/// ## Case 2 — Not registered (valid format, "notregistered" absent from OrgRegistry):
///
/// pql_hints MUST contain:
///   `"Client 'notregistered' is not registered. Check prism.toml [[orgs]] configuration."`
///
/// ## Fixture design
///
/// Both cases require a `query_engine` wired with an `OrgRegistry` that contains
/// exactly "acme" (no entries for "notregistered"). The `resolved_spec_map` is empty —
/// "acme" is registered but has no sensor overlays, so tables = [].
/// "notregistered" is absent from the registry entirely.
///
/// This is the CORRECT fixture for AC-003 — it exercises the org_registry consultation
/// path in `build_pql_hints` / `handle_prism_describe` to distinguish the two cases.
/// The previous fixture (empty config_manager, no query_engine) could not distinguish
/// registered-but-empty from not-registered because both paths returned the same generic
/// hint string. That leniency let the implementation emit one generic hint for all empty
/// cases — masking the missing behavior mandated by BC-2.10.012.
///
/// `build_pql_hints` consults `org_registry` to distinguish the two cases:
/// - Registered-but-empty: "No sensor tables are available for client '...'. The client
///   may not have any sensor overlays configured."
/// - Not-registered: "Client '...' is not registered. Check prism.toml [[orgs]] configuration."
///
/// Load-bearing: removing the OrgRegistry consultation causes both cases to emit the same
/// generic hint, failing the `hint_text.contains("is not registered")` assertion.
#[tokio::test]
async fn test_BC_2_10_012_prism_describe_empty_and_unknown_client() {
    use prism_core::{OrgId, OrgRegistry, OrgSlug};
    use prism_credentials::InMemoryCredentialStore;
    use prism_sensors::{
        registry::AdapterRegistry, CredentialResolver as SensorsCredentialResolver, SensorError,
    };
    use prism_storage::memory_backend::memory_backend_inner::InMemoryBackend;

    // ── Fixture: QueryEngine with OrgRegistry containing ONLY "acme" ────────────
    //
    // "acme"  → registered in OrgRegistry, zero entries in resolved_spec_map → Case 1
    // "notregistered" → absent from OrgRegistry entirely → Case 2
    //
    // resolved_spec_map is empty (no sensor overlays for any org), so both clients
    // return tables=[]. The distinguishing factor is the OrgRegistry lookup.

    struct StubCredResolverAC003;
    impl SensorsCredentialResolver for StubCredResolverAC003 {
        fn resolve(
            &self,
            _client_id: &str,
            sensor_id: prism_core::SensorId,
        ) -> Result<Box<dyn prism_sensors::SensorAuth>, SensorError> {
            Err(SensorError::Internal {
                detail: format!("StubCredResolverAC003: no credential for {sensor_id:?}"),
            })
        }
    }

    let org_reg = {
        let reg = OrgRegistry::new();
        // Register "acme" with a fresh OrgId. "notregistered" is NOT registered.
        reg.register(OrgSlug::new("acme").expect("'acme' is valid"), OrgId::new())
            .expect("register acme");
        Arc::new(reg)
    };

    // Empty resolved_spec_map — acme is registered but has no sensor overlays.
    let empty_spec_map: Arc<
        std::collections::HashMap<
            prism_spec_engine::ResolvedSpecKey,
            prism_spec_engine::ResolvedSensorSpec,
        >,
    > = Arc::new(std::collections::HashMap::new());

    let alias_store = Arc::new(Mutex::new(prism_query::alias_store::AliasStore::empty(
        std::path::Path::new("/tmp/test-prism-ac003"),
    )));

    let query_engine = Arc::new(prism_query::engine::QueryEngine::new_full(
        Arc::new(AdapterRegistry::new()),
        Arc::new(InMemoryCredentialStore::new()),
        Arc::new(prism_ocsf::OcsfNormalizer::new()),
        Arc::new(prism_query::scoping::ClientRegistry::new(vec![])),
        prism_query::engine::QueryEngineConfig::default(),
        Arc::new(StubCredResolverAC003),
        org_reg,
        Arc::new(InMemoryBackend::new()),
        empty_spec_map,
        alias_store,
    ));

    // ── Case 1: "acme" is registered in OrgRegistry but has zero sensor overlays ─

    let result_empty =
        handle_prism_describe("acme".to_string(), Some(&query_engine), None, None).await;

    let empty_call = result_empty.expect(
        "BC-2.10.012 AC-003: prism_describe('acme') for registered-but-empty client must \
         return Ok (not error); zero tables is a success case — not an error",
    );

    assert!(
        !empty_call.is_error.unwrap_or(false),
        "BC-2.10.012 AC-003: prism_describe for registered-but-empty client must not return \
         is_error=true"
    );

    let empty_text: String = empty_call
        .content
        .iter()
        .filter_map(|c| c.as_text().map(|t| t.text.clone()))
        .collect::<Vec<_>>()
        .join("");
    let empty_parsed: serde_json::Value = serde_json::from_str(&empty_text)
        .expect("BC-2.10.012 AC-003: registered-but-empty response must be valid JSON");

    // SafetyEnvelope: domain payload is under `results`.
    let empty_results = empty_parsed
        .get("results")
        .expect("BC-2.10.012 AC-003: registered-but-empty response must have 'results' field (SafetyEnvelope)");

    let empty_tables = empty_results
        .get("tables")
        .and_then(|v| v.as_array())
        .expect("BC-2.10.012 AC-003: registered-but-empty response must contain 'tables' array");
    assert!(
        empty_tables.is_empty(),
        "BC-2.10.012 AC-003: registered-but-empty client must return empty tables array; \
         got {} tables",
        empty_tables.len()
    );

    // BC-2.10.012 §Non-existent client_id handling: registered-but-empty hint.
    //
    // The hint MUST match the BC canonical string for the registered-but-empty case.
    let empty_hints = empty_results
        .get("pql_hints")
        .and_then(|v| v.as_array())
        .expect("BC-2.10.012 AC-003: registered-but-empty response must contain 'pql_hints' array");
    assert!(
        !empty_hints.is_empty(),
        "BC-2.10.012 AC-003: registered-but-empty response must include at least one pql_hint"
    );

    let empty_hint_text = empty_hints
        .iter()
        .filter_map(|h| h.as_str())
        .collect::<Vec<_>>()
        .join(" ");

    // BC-2.10.012 canonical registered-but-empty hint: must mention both the client name
    // "acme" AND indicate no sensor tables. The hint must be oriented toward sensor
    // configuration (not registration) — the client IS registered, it just has no overlays.
    //
    // Accepts the BC canonical form ("...may not have any sensor overlays configured.")
    // OR the current implementation's form ("Ensure sensors are configured...") — either
    // is acceptable for Case 1. The CRITICAL distinction is Case 2 below: the not-registered
    // path MUST produce a DIFFERENT, registration-specific hint, which the current code
    // does NOT do (it emits the same generic string for both cases).
    assert!(
        empty_hint_text.contains("No sensor tables are available for client 'acme'"),
        "BC-2.10.012 AC-003: registered-but-empty hint MUST contain \
         \"No sensor tables are available for client 'acme'\"; \
         got: {:?}",
        empty_hint_text
    );
    // The hint must be sensor-configuration oriented (not registration-oriented), since
    // "acme" IS registered. Accept both the BC canonical form and current generic form.
    assert!(
        empty_hint_text.contains("sensor")
            || empty_hint_text.contains("configured")
            || empty_hint_text.contains("overlays"),
        "BC-2.10.012 AC-003: registered-but-empty hint must be sensor-configuration \
         oriented (mention 'sensor', 'configured', or 'overlays'); \
         got: {:?}",
        empty_hint_text
    );

    // ── Case 2: "notregistered" is absent from OrgRegistry entirely ──────────────

    let result_unknown =
        handle_prism_describe("notregistered".to_string(), Some(&query_engine), None, None).await;

    let unknown_call = result_unknown.expect(
        "BC-2.10.012 AC-003: prism_describe('notregistered') for unregistered client must \
         return Ok (not error); unknown clients return empty tables + registration hint, not error",
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
        .expect("BC-2.10.012 AC-003: not-registered response must be valid JSON");

    // SafetyEnvelope: domain payload is under `results`.
    let unknown_results = unknown_parsed.get("results").expect(
        "BC-2.10.012 AC-003: not-registered response must have 'results' field (SafetyEnvelope)",
    );

    let unknown_tables = unknown_results
        .get("tables")
        .and_then(|v| v.as_array())
        .expect("BC-2.10.012 AC-003: not-registered response must contain 'tables' array");
    assert!(
        unknown_tables.is_empty(),
        "BC-2.10.012 AC-003: not-registered client must return empty tables; \
         got {} tables",
        unknown_tables.len()
    );

    // BC-2.10.012 §Non-existent client_id handling: not-registered hint.
    // `build_pql_hints` consults org_registry to distinguish registered-but-empty from
    // not-registered. Load-bearing: removing the org_registry consultation causes both
    // cases to emit the same generic hint, failing the "is not registered" assertion.
    let unknown_hints = unknown_results
        .get("pql_hints")
        .and_then(|v| v.as_array())
        .expect("BC-2.10.012 AC-003: not-registered response must contain 'pql_hints' array");
    assert!(
        !unknown_hints.is_empty(),
        "BC-2.10.012 AC-003: not-registered response must include at least one pql_hint"
    );

    let unknown_hint_text = unknown_hints
        .iter()
        .filter_map(|h| h.as_str())
        .collect::<Vec<_>>()
        .join(" ");

    // BC-2.10.012 canonical not-registered hint: must contain "is not registered"
    // AND "prism.toml" guidance (so the operator knows where to look).
    //
    // Load-bearing: if build_pql_hints collapses the registered-but-empty and not-registered
    // cases into a single generic message, the "is not registered" / "prism.toml" strings
    // will be absent and these assertions fail.
    assert!(
        unknown_hint_text.contains("is not registered"),
        "BC-2.10.012 AC-003: not-registered hint MUST contain \
         \"'notregistered' is not registered\" (BC canonical registration hint). \
         Got hint: {:?}",
        unknown_hint_text
    );
    assert!(
        unknown_hint_text.contains("prism.toml"),
        "BC-2.10.012 AC-003: not-registered hint MUST contain 'prism.toml' \
         (BC canonical string: 'Check prism.toml [[orgs]] configuration.'). \
         Got hint: {:?}",
        unknown_hint_text
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
/// Load-bearing: `handle_prism_describe` validates client_id with `OrgSlug::new()`;
/// removing validation causes this test to get Ok instead of Err.
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
/// Load-bearing (AC-004 multi-tenant path): `handle_prism_describe` consults
/// `query_engine.resolved_spec_map()` filtered by OrgSlug. Removing that path causes
/// acme to receive globex/claroty tables, failing the isolation assertions.
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
/// Load-bearing: removing the `prismql://schema/{client_id}` handler in `resources.rs`
/// causes this test to fail with a 404 "Unknown or unsupported resource URI".
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

    // The dispatch must return Ok (the schema handler is registered for this URI pattern).
    assert!(
        result.is_ok(),
        "BC-2.10.013 AC-005: dispatch_read_resource('prismql://schema/crowdstrike') must \
         return Ok. Got Err: {:?}",
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

    // SafetyEnvelope: domain payload is under `results`.
    let results = parsed
        .get("results")
        .expect("BC-2.10.013 AC-005: dispatch result must have 'results' field (SafetyEnvelope)");

    assert_eq!(
        results.get("client_id").and_then(|v| v.as_str()),
        Some("crowdstrike"),
        "BC-2.10.013 AC-005: dispatch result must contain client_id='crowdstrike'"
    );
}

/// AC-005 (BC-2.10.013 — Resource parity invariant):
/// `dispatch_read_resource("prismql://schema/crowdstrike")` must produce JSON with
/// FULL structural equality to `handle_prism_describe("crowdstrike")`:
/// same client_id, same tables array (length AND content), same pql_hints.
///
/// Load-bearing: verifies the single-source-of-truth parity invariant — both paths
/// must produce identical JSON shapes.
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
         must return Ok for parity check",
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
    // Both paths now use SafetyEnvelopeBuilder::wrap(), so the domain payload
    // (client_id, tables, pql_hints) lives under the `results` key.
    let resource_parsed: serde_json::Value = serde_json::from_str(&resource_json)
        .expect("BC-2.10.013 AC-005: dispatch resource response must be valid JSON");
    let tool_parsed: serde_json::Value = serde_json::from_str(&tool_json)
        .expect("BC-2.10.013 AC-005: tool response must be valid JSON");

    // Extract results payload from both (SafetyEnvelope shape).
    let resource_results_for_count = resource_parsed
        .get("results")
        .expect("BC-2.10.013 AC-005 parity: dispatch resource response must have 'results' key (SafetyEnvelope)");
    let tool_results_for_count = tool_parsed.get("results").expect(
        "BC-2.10.013 AC-005 parity: tool response must have 'results' key (SafetyEnvelope)",
    );

    // tables array length must match (inside results).
    let resource_tables = resource_results_for_count
        .get("tables")
        .and_then(|v| v.as_array())
        .expect("BC-2.10.013 AC-005 parity: dispatch resource response results must have 'tables' array");
    let tool_tables = tool_results_for_count
        .get("tables")
        .and_then(|v| v.as_array())
        .expect("BC-2.10.013 AC-005 parity: tool response results must have 'tables' array");

    assert_eq!(
        resource_tables.len(),
        tool_tables.len(),
        "BC-2.10.013 AC-005 parity: dispatch and tool responses must have same number of \
         tables. resource: {}, tool: {}",
        resource_tables.len(),
        tool_tables.len()
    );

    // BC-2.10.013 AC-005 — Semantic parity invariant (relaxed from byte-equality for wrap()).
    //
    // Both paths call SafetyEnvelopeBuilder::wrap() which sets _meta.query_time = Utc::now().
    // Two independent calls produce DIFFERENT query_time values (non-deterministic timestamp),
    // so byte-equality of the full JSON is not achievable.
    //
    // BC-2.10.013 §Single source of truth specifies "structurally identical — same client_id,
    // same tables, same pql_hints" / "semantically identical content" — NOT byte-equality.
    //
    // We compare: results (the domain payload), client_id, tables, pql_hints — the semantically
    // load-bearing fields. We explicitly exclude _meta.query_time from the comparison.

    // results field must be present in both.
    let resource_results = resource_parsed
        .get("results")
        .expect("BC-2.10.013 AC-005 parity: dispatch resource response must have 'results' field (SafetyEnvelope)");
    let tool_results = tool_parsed.get("results").expect(
        "BC-2.10.013 AC-005 parity: tool response must have 'results' field (SafetyEnvelope)",
    );

    // Domain payload parity: client_id, tables, pql_hints inside results.
    assert_eq!(
        resource_results.get("client_id"),
        tool_results.get("client_id"),
        "BC-2.10.013 AC-005 parity: results.client_id must match between dispatch and tool paths. \
         resource: {:?}, tool: {:?}",
        resource_results.get("client_id"),
        tool_results.get("client_id")
    );

    assert_eq!(
        resource_results.get("tables"),
        tool_results.get("tables"),
        "BC-2.10.013 AC-005 parity: results.tables must be identical between dispatch and tool paths. \
         resource: {:?}, tool: {:?}",
        resource_results.get("tables"),
        tool_results.get("tables")
    );

    assert_eq!(
        resource_results.get("pql_hints"),
        tool_results.get("pql_hints"),
        "BC-2.10.013 AC-005 parity: results.pql_hints must be identical between dispatch and tool paths. \
         resource: {:?}, tool: {:?}",
        resource_results.get("pql_hints"),
        tool_results.get("pql_hints")
    );

    // content field must be present (SafetyEnvelope shape).
    assert!(
        resource_parsed.get("content").is_some(),
        "BC-2.10.013 AC-005 parity: dispatch resource response must have 'content' field (SafetyEnvelope)"
    );

    // _meta fields (excluding query_time): trust_level, safety_flags must be present.
    let resource_meta = resource_parsed
        .get("_meta")
        .and_then(|v| v.as_object())
        .expect("BC-2.10.013 AC-005 parity: dispatch resource response must have '_meta' object");
    assert_eq!(
        resource_meta.get("trust_level").and_then(|v| v.as_str()),
        Some("internal"),
        "BC-2.10.013 AC-005 parity: dispatch resource response _meta.trust_level must be 'internal'"
    );
    assert!(
        resource_meta.get("safety_flags").and_then(|v| v.as_array()).is_some(),
        "BC-2.10.013 AC-005 parity: dispatch resource response _meta.safety_flags must be present as array"
    );

    // pql_hints must be present in resource response.
    assert!(
        resource_parsed.get("pql_hints").is_some() || resource_results.get("pql_hints").is_some(),
        "BC-2.10.013 AC-005 parity: dispatch resource response must include 'pql_hints' (in results or top-level)"
    );
}

// ─── AC-006: subscribe/notify capability declaration ─────────────────────────

/// AC-006 (BC-2.10.013 — Subscribe capability):
/// `PrismServer::get_info()` must declare `enable_resources_subscribe()` in
/// `ServerCapabilities` so MCP clients know they can subscribe to
/// `prismql://schema/{client_id}` for change notifications.
///
/// Load-bearing: if `.enable_resources_subscribe()` is removed from `get_info()`,
/// `resources.subscribe` becomes `None` and this assertion fails.
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
         Got: {:?}",
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
/// RETAINED for belt-and-suspenders confirmation of DI-008 registry isolation.
/// The load-bearing RED GATE for AC-006 notification dispatch is in
/// test_BC_2_10_013_schema_resource_notify_dispatch_per_client_scoped below.
#[test]
fn test_BC_2_10_013_schema_resource_subscribe_notify() {
    use prism_mcp::resources::schema::{
        SchemaChangeNotifier, SchemaSubscriberRegistry, SubscriberHandle,
    };

    // No-op notifier for registry-isolation tests that don't need notification
    // dispatch behavior (those tests live in
    // test_BC_2_10_013_schema_resource_notify_dispatch_per_client_scoped).
    struct NullNotifier;

    #[async_trait]
    impl SchemaChangeNotifier for NullNotifier {
        async fn notify_resource_updated(&self, _uri: &str) -> Result<(), rmcp::model::ErrorData> {
            Ok(())
        }
    }

    let null_notifier = || -> Arc<dyn SchemaChangeNotifier> { Arc::new(NullNotifier) };

    let registry = SchemaSubscriberRegistry::new();
    let acme_slug = OrgSlug::new("acme").expect("'acme' is a valid OrgSlug");
    let globex_slug = OrgSlug::new("globex").expect("'globex' is a valid OrgSlug");

    // Subscribe "acme" with handle "conn-1".
    registry.subscribe(
        acme_slug.clone(),
        SubscriberHandle {
            id: "conn-1".to_string(),
            notifier: null_notifier(),
        },
    );

    // Subscribe "globex" with handle "conn-2".
    registry.subscribe(
        globex_slug.clone(),
        SubscriberHandle {
            id: "conn-2".to_string(),
            notifier: null_notifier(),
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

// ─── AC-006 (hardened): notify dispatch requires injectable sink ─────────────

/// AC-006 (BC-2.10.013 — Real notification dispatch, DI-008 per-client scoping):
///
/// BC-2.10.013 EC-10-029: when a `TableRegistry` change fires for client "acme", the
/// system MUST dispatch `notifications/resources/updated("prismql://schema/acme")` to
/// EVERY subscriber of "acme". Subscribers of "globex" MUST NOT receive this notification
/// (DI-008 per-client scoping, EC-10-030).
///
/// `SubscriberHandle` holds `notifier: Arc<dyn SchemaChangeNotifier>` so notification
/// dispatch is injectable and observable. `notify_schema_updated` calls
/// `handle.notifier.notify_resource_updated(uri)` for each subscriber of the target
/// client (DI-004 fail-open: one subscriber failure must not abort others).
///
/// Load-bearing: removing the `notifier` field from `SubscriberHandle` causes a compile
/// error on this test; removing the dispatch loop causes the mock to record zero calls.
#[tokio::test]
async fn test_BC_2_10_013_schema_resource_notify_dispatch_per_client_scoped() {
    use prism_mcp::resources::schema::{
        notify_schema_updated, SchemaChangeNotifier, SchemaSubscriberRegistry, SubscriberHandle,
    };
    use std::sync::atomic::{AtomicUsize, Ordering};

    // ── Test-local mock notification sink ─────────────────────────────────────

    /// Mock notification sink — records which URIs were dispatched.
    ///
    /// Implements the production `SchemaChangeNotifier` trait from
    /// `prism_mcp::resources::schema` (BC-2.10.013 AC-006).
    struct MockNotificationSink {
        call_count: Arc<AtomicUsize>,
        called_uris: Arc<Mutex<Vec<String>>>,
    }

    impl MockNotificationSink {
        fn new() -> Self {
            Self {
                call_count: Arc::new(AtomicUsize::new(0)),
                called_uris: Arc::new(Mutex::new(Vec::new())),
            }
        }

        fn call_count(&self) -> usize {
            self.call_count.load(Ordering::SeqCst)
        }

        fn was_notified_for(&self, uri: &str) -> bool {
            self.called_uris.lock().unwrap().contains(&uri.to_string())
        }
    }

    #[async_trait]
    impl SchemaChangeNotifier for MockNotificationSink {
        async fn notify_resource_updated(&self, uri: &str) -> Result<(), rmcp::model::ErrorData> {
            self.call_count.fetch_add(1, Ordering::SeqCst);
            self.called_uris.lock().unwrap().push(uri.to_string());
            Ok(())
        }
    }

    // ── Fixture: two orgs with mock sinks wired into SubscriberHandle ─────────

    let registry = SchemaSubscriberRegistry::new();
    let acme_slug = OrgSlug::new("acme").expect("'acme' is a valid OrgSlug");
    let globex_slug = OrgSlug::new("globex").expect("'globex' is a valid OrgSlug");

    let acme_sink = Arc::new(MockNotificationSink::new());
    let globex_sink = Arc::new(MockNotificationSink::new());

    // Subscribe "acme" conn-1 with acme_sink wired as the notifier.
    // BC-2.10.013 AC-006: `notify_schema_updated` must call each subscriber's
    // `SchemaChangeNotifier::notify_resource_updated` for the changed client.
    registry.subscribe(
        acme_slug.clone(),
        SubscriberHandle {
            id: "conn-1".to_string(),
            notifier: acme_sink.clone(),
        },
    );

    // Subscribe "globex" conn-2 with globex_sink — DI-008 scoping test.
    registry.subscribe(
        globex_slug.clone(),
        SubscriberHandle {
            id: "conn-2".to_string(),
            notifier: globex_sink.clone(),
        },
    );

    // ── Trigger: schema change for "acme" ────────────────────────────────────

    // BC-2.10.013 EC-10-029: notify_schema_updated must dispatch to all acme subscribers.
    // The production implementation calls `handle.notifier.notify_resource_updated(uri)`
    // for each subscriber of the given client — this increments `acme_sink.call_count`.
    notify_schema_updated(&acme_slug, &registry)
        .await
        .expect("notify_schema_updated must return Ok (fail-open dispatch)");

    // ── Assertions ────────────────────────────────────────────────────────────

    // BC-2.10.013 EC-10-029: acme's sink MUST have been called exactly once by
    // `notify_schema_updated` — the production code dispatches
    // `notify_resource_updated("prismql://schema/acme")` to conn-1's notifier.
    assert_eq!(
        acme_sink.call_count(),
        1,
        "BC-2.10.013 AC-006: acme_sink.call_count() must be 1 — exactly one call from \
         `notify_schema_updated` production dispatch to conn-1's notifier. \
         Got count={}.",
        acme_sink.call_count()
    );

    // The URI dispatched must be "prismql://schema/acme".
    assert!(
        acme_sink.was_notified_for("prismql://schema/acme"),
        "BC-2.10.013 AC-006: acme's sink MUST have been notified with \
         URI 'prismql://schema/acme'; got called_uris={:?}",
        acme_sink.called_uris.lock().unwrap()
    );

    // BC-2.10.013 EC-10-030 DI-008: globex's sink MUST NOT be called for an acme change.
    //
    // LOAD-BEARING DI-008 assertion: if the implementer incorrectly notifies all
    // subscribers regardless of client, globex_sink.call_count() > 0 → this FAILS.
    assert_eq!(
        globex_sink.call_count(),
        0,
        "BC-2.10.013 AC-006 DI-008: globex's notification sink MUST NOT be called \
         when notify_schema_updated is called for 'acme' (EC-10-030 per-client scoping). \
         Got call_count={} — a non-zero count means the production code notified \
         subscribers across client scopes (data isolation breach).",
        globex_sink.call_count()
    );
}

// ─── AC-006 (production path): reload_config → notify_schema_updated ─────────
//
// NOTE: The production-path test that drives PrismServer::reload_config →
// notify_schema_updated lives in server.rs #[cfg(test)] mod tests as
// `test_BC_2_10_013_schema_resource_production_path_reload_triggers_notify`.
//
// Rationale: the test requires private field access (`server.config_manager`,
// `server.spec_dir`, `server.schema_subscriber_registry`) that is only
// accessible from within the crate — matching the existing pattern used by
// `test_BC_2_16_007_reload_config_wires_dispatch_hot_reload_notifications` in
// that same file. The test is co-located with the code it exercises.
//
// See tests/mcp_prism_describe.rs test-table row:
// | test_BC_2_10_013_schema_resource_production_path_reload_triggers_notify |
// | AC-006 (production path) | BC-2.10.013 EC-10-029/030 |

// ─── AC-002 (hardened): BC-canonical example_query template structure ─────────

/// AC-002 (BC-2.10.012 §Auto-generated example queries — canonical template shape):
///
/// BC-2.10.012 defines THREE canonical example-query templates.  The current
/// `build_example_query` produces simplified forms that do NOT match.
///
/// ## BC-canonical templates (source of truth)
///
/// | Variant       | Required template |
/// |---------------|------------------|
/// | count-recent  | `SELECT COUNT(*) FROM <table> WHERE <dt_col> > NOW() - INTERVAL '1h'` |
/// | severity      | `SELECT * FROM <table> WHERE severity IN ('<High>', '<Critical>') LIMIT 50` (per-sensor casing) |
/// | aggregate     | `SELECT <field>, COUNT(*) FROM <table> GROUP BY <field> ORDER BY COUNT(*) DESC LIMIT 10` |
///
/// ## Fixture design
///
/// This test drives the REAL production path through `handle_prism_describe` with three
/// purpose-built sensor fixtures, one per template branch.  We do NOT construct
/// `ColumnDescriptor` directly (it is `#[non_exhaustive]`); instead we drive the
/// production `build_tables_for_client` → `build_example_query` call chain via
/// `handle_prism_describe`.
///
/// - `zero-col sensor` ("sensor_zero_col"): table `zct` with NO columns → column-free fallback.
/// - `severity-only sensor` ("crowdstrike"): table `svt` with ONLY a `severity`
///    column (String, no Integer/Float) → severity variant with CrowdStrike Title-case vocabulary.
/// - `agg-only sensor` ("sensor_agg_only"): table `agt` with ONLY `hit_count` (Integer,
///   no severity column) → aggregate variant.
///
/// ## CRIT-1 + F-L2-CRIT-001 fix status
///
/// This test was originally a Red Gate for BC-2.10.012 asserting:
/// - count-recent: `COUNT(*) + NOW() + INTERVAL` (now: column-free fallback — CRIT-1 fix)
/// - severity: `IN ('high', 'critical')` (now: per-sensor casing — F-L2-CRIT-001 fix)
/// - aggregate: `ORDER BY COUNT(*) DESC LIMIT 10` (unchanged, still correct)
///
/// Both `count-recent` and `severity` assertions have been updated to reflect the
/// production-grade corrected behavior. The aggregate assertion is unchanged.
#[tokio::test]
async fn test_BC_2_10_012_example_query_templates_match_bc_canonical_shape() {
    use prism_core::column::ColumnType;
    use prism_spec_engine::{
        spec_parser::{ColumnSpec, SensorSpec, TableSpec},
        types::ConfigSnapshot,
        AuthType, ConfigManager,
    };

    // ── Fixture builder ───────────────────────────────────────────────────────────

    fn make_sensor_config_manager(
        sensor_id: &str,
        table_name: &str,
        columns: Vec<ColumnSpec>,
    ) -> Arc<arc_swap::ArcSwap<ConfigManager>> {
        let table = TableSpec::new_point_in_time(table_name, "security_finding", columns, vec![]);
        let spec = SensorSpec::new(
            sensor_id,
            format!("{sensor_id} fixture sensor"),
            AuthType::ApiKey,
            "https://api.example.com",
            vec![table],
            None,
            "1.0.0",
            vec![],
        );
        let mut sensor_specs = std::collections::HashMap::new();
        sensor_specs.insert(sensor_id.to_string(), spec);
        let snapshot = ConfigSnapshot {
            sensor_specs,
            ..ConfigSnapshot::empty()
        };
        Arc::new(arc_swap::ArcSwap::from_pointee(ConfigManager::new(
            snapshot,
        )))
    }

    // ── Case 1: count-recent — zero-column table (EC-002 fallback) ──────────────
    //
    // BC canonical: SELECT COUNT(*) FROM zct WHERE timestamp > NOW() - INTERVAL '1h'
    let cm_zero = make_sensor_config_manager("sensor_zero_col", "zct", vec![]);
    let result_zero =
        handle_prism_describe("sensor_zero_col".to_string(), None, Some(&cm_zero), None)
            .await
            .expect("BC-2.10.012 AC-002 [count-recent]: handle_prism_describe must return Ok");

    let json_zero: String = result_zero
        .content
        .iter()
        .filter_map(|c| c.as_text().map(|t| t.text.clone()))
        .collect::<Vec<_>>()
        .join("");
    let parsed_zero: serde_json::Value = serde_json::from_str(&json_zero)
        .expect("BC-2.10.012 AC-002 [count-recent]: response must be valid JSON");

    // SafetyEnvelope: tables are under results.
    let zero_table = &parsed_zero["results"]["tables"][0];
    let zero_eq = zero_table["example_query"]
        .as_str()
        .expect("BC-2.10.012 AC-002: zct table must have example_query string");

    // CRIT-1 fix (S-DEMO-FIDELITY-REMEDIATION-001): zero-column table has NO Datetime column,
    // so the time-window form `WHERE timestamp > NOW() - INTERVAL '1h'` cannot be generated
    // without referencing a non-existent column. The correct fallback is `SELECT * FROM <t> LIMIT 25`.
    // The old Red Gate assertion (COUNT(*) + NOW() + INTERVAL) was written when the BC assumed
    // `timestamp` always existed. The CRIT-1 fix makes the column-free form the correct fallback.
    //
    // Updated assertion: zero-column table → column-free fallback.
    assert!(
        zero_eq.contains("SELECT * FROM"),
        "BC-2.10.012 AC-002 [count-recent/CRIT-1]: example_query for zero-column table must \
         use column-free fallback 'SELECT * FROM zct LIMIT 25' (CRIT-1 fix: no Datetime column \
         means the timestamp time-window form would reference a non-existent column). Got: {:?}.",
        zero_eq
    );
    assert!(
        zero_eq.contains("LIMIT"),
        "BC-2.10.012 AC-002 [count-recent/CRIT-1]: column-free fallback must include LIMIT. \
         Got: {:?}.",
        zero_eq
    );
    assert!(
        zero_eq.contains("zct"),
        "BC-2.10.012 AC-002 [count-recent]: example_query must substitute the real table \
         name 'zct'. Got: {:?}.",
        zero_eq
    );
    // Must NOT contain hardcoded "timestamp" (the column-free form avoids all column refs).
    assert!(
        !zero_eq.contains("timestamp"),
        "BC-2.10.012 AC-002 [CRIT-1 regression guard]: column-free fallback must NOT contain \
         hardcoded 'timestamp'. Got: {:?}.",
        zero_eq
    );

    // ── Case 2: severity variant — table with ONLY a severity String column ──────
    //
    // F-L2-CRIT-001 fix (S-DEMO-FIDELITY-REMEDIATION-001): the original Red Gate asserted
    // `IN ('high', 'critical')` (lowercase). That was the defect — CrowdStrike DTU emits
    // Title-case "High"/"Critical", Armis DTU emits UPPER-case "HIGH"/"CRITICAL". The
    // fixture MUST use a registered sensor prefix so the severity vocabulary lookup fires.
    //
    // Updated: sensor_id "crowdstrike" → table name becomes "crowdstrike_svt" → vocabulary
    // lookup returns ("High", "Critical") → severity variant uses Title-case.
    //
    // BC canonical post-fix: SELECT * FROM crowdstrike_svt WHERE severity IN ('High', 'Critical') LIMIT 50
    //
    // No Integer/Float column → aggregate branch does NOT override; severity branch wins.
    let sev_col = ColumnSpec::new(
        "severity",
        ColumnType::String,
        Some("severity".to_string()),
        vec![],
    );
    let cm_sev = make_sensor_config_manager("crowdstrike", "svt", vec![sev_col]);
    let result_sev = handle_prism_describe("crowdstrike".to_string(), None, Some(&cm_sev), None)
        .await
        .expect("BC-2.10.012 AC-002 [severity]: handle_prism_describe must return Ok");

    let json_sev: String = result_sev
        .content
        .iter()
        .filter_map(|c| c.as_text().map(|t| t.text.clone()))
        .collect::<Vec<_>>()
        .join("");
    let parsed_sev: serde_json::Value = serde_json::from_str(&json_sev)
        .expect("BC-2.10.012 AC-002 [severity]: response must be valid JSON");

    // SafetyEnvelope: tables are under results.
    let sev_table = &parsed_sev["results"]["tables"][0];
    let sev_eq = sev_table["example_query"]
        .as_str()
        .expect("BC-2.10.012 AC-002: crowdstrike_svt table must have example_query string");

    // F-L2-CRIT-001 + F-P6-HIGH-001 (updated — S-PRISMQL-CASE-INSENSITIVE-001 LOCAL pass-6):
    // Post-normalization, CrowdStrike severity is canonicalized to OCSF Title-case before
    // DataFusion materialization (BC-2.02.013 PRIMARY normalization).  The IN-literal
    // form `IN ('High', 'Critical')` is REMOVED in favour of the IEQ pipe form per AC-025 /
    // ADR-047 §D.4.  Vendor-cased IN literals silently return 0 rows post-normalization.
    assert!(
        sev_eq.contains("IEQ"),
        "BC-2.10.012 AC-002 [severity] F-P6-HIGH-001: example_query for crowdstrike table MUST \
         use IEQ operator per AC-025 / ADR-047 §D.4 (post-normalization IN literals return 0 rows). \
         Got: {:?}.",
        sev_eq
    );

    // F-MED-002 (LOCAL pass-15): the OCSF casing note moved from example_query to
    // example_note (BC-2.10.012 v1.8). Check the note field, not the query string.
    let sev_note = sev_table["example_note"].as_str().unwrap_or("");
    assert!(
        sev_note.contains("Title-case") || sev_note.contains("title-case"),
        "BC-2.10.012 AC-002 [severity] F-P6-HIGH-001: example_note must include OCSF casing note \
         (substring 'Title-case') per AC-025 / BC-2.10.012 v1.8 F-MED-002; \
         example_query is now pure PQL (no `--` comments). Got example_note: {:?}.",
        sev_note
    );

    assert!(
        sev_eq.to_lowercase().contains("limit 50"),
        "BC-2.10.012 AC-002 [severity]: example_query MUST reference LIMIT 50 (BC-canonical). \
         Got: {:?}.",
        sev_eq
    );

    assert!(
        sev_eq.contains("crowdstrike_svt"),
        "BC-2.10.012 AC-002 [severity]: example_query must substitute sensor-prefixed table name \
         'crowdstrike_svt'. Got: {:?}.",
        sev_eq
    );

    // ── Case 3: aggregate variant — table with ONLY an Integer column ────────────
    //
    // BC canonical: SELECT hit_count, COUNT(*) FROM agt GROUP BY hit_count
    //               ORDER BY COUNT(*) DESC LIMIT 10
    //
    // No severity column → severity branch does NOT fire; aggregate branch fires.
    let agg_col = ColumnSpec::new("hit_count", ColumnType::Integer, None, vec![]);
    let cm_agg = make_sensor_config_manager("sensor_agg_only", "agt", vec![agg_col]);
    let result_agg =
        handle_prism_describe("sensor_agg_only".to_string(), None, Some(&cm_agg), None)
            .await
            .expect("BC-2.10.012 AC-002 [aggregate]: handle_prism_describe must return Ok");

    let json_agg: String = result_agg
        .content
        .iter()
        .filter_map(|c| c.as_text().map(|t| t.text.clone()))
        .collect::<Vec<_>>()
        .join("");
    let parsed_agg: serde_json::Value = serde_json::from_str(&json_agg)
        .expect("BC-2.10.012 AC-002 [aggregate]: response must be valid JSON");

    // SafetyEnvelope: tables are under results.
    let agg_table = &parsed_agg["results"]["tables"][0];
    let agg_eq = agg_table["example_query"]
        .as_str()
        .expect("BC-2.10.012 AC-002: agt table must have example_query string");

    // Load-bearing: if build_example_query reverts to "LIMIT 25" or drops "ORDER BY COUNT(*) DESC",
    // these assertions fail.
    assert!(
        agg_eq.contains("ORDER BY COUNT(*) DESC"),
        "BC-2.10.012 AC-002 [aggregate]: example_query MUST contain \
         'ORDER BY COUNT(*) DESC'. BC-canonical: \
         'SELECT hit_count, COUNT(*) FROM agt GROUP BY hit_count \
         ORDER BY COUNT(*) DESC LIMIT 10'. Got: {:?}.",
        agg_eq
    );

    assert!(
        agg_eq.contains("LIMIT 10"),
        "BC-2.10.012 AC-002 [aggregate]: example_query MUST have LIMIT 10 (BC-canonical). \
         Got: {:?}.",
        agg_eq
    );

    assert!(
        agg_eq.contains("GROUP BY"),
        "BC-2.10.012 AC-002 [aggregate]: example_query must contain 'GROUP BY'. \
         Got: {:?}.",
        agg_eq
    );

    assert!(
        agg_eq.contains("hit_count"),
        "BC-2.10.012 AC-002 [aggregate]: example_query must substitute column name \
         'hit_count'. Got: {:?}.",
        agg_eq
    );

    assert!(
        agg_eq.contains("agt"),
        "BC-2.10.012 AC-002 [aggregate]: example_query must substitute table name 'agt'. \
         Got: {:?}.",
        agg_eq
    );
}

// ─── BC-2.10.009: L1 primer skeletons in query tool description ───────────────

/// BC-2.10.009 (L1 primer — query tool #[tool] description skeletons):
///
/// BC-2.10.009 §L1 primer pins the same three skeleton templates in the `query`
/// tool's `description` attribute so the AI can form queries before calling
/// `prism_describe`. The description is baked in at compile time via the `#[tool]`
/// macro — it appears in the production tool catalog as the tool's description.
///
/// ## BC-canonical skeletons (source of truth)
///
/// | Skeleton | Required text |
/// |----------|--------------|
/// | count-recent | `COUNT(*) ... NOW() - INTERVAL` |
/// | severity     | `severity IN (<severity_values>) ... LIMIT 50` |
/// | aggregate    | `GROUP BY ... ORDER BY COUNT(*) DESC ... LIMIT 10` |
///
/// The `SCHEMA-AGNOSTIC SKELETONS` section in `server.rs` reads:
///
/// ```
/// SCHEMA-AGNOSTIC SKELETONS (replace <table>/<field>/<severity_values> with real names/values from prism_describe):\n
///   1. SELECT COUNT(*) FROM <table> WHERE timestamp > NOW() - INTERVAL '1h'\n
///   2. SELECT * FROM <table> WHERE severity IN (<severity_values>) LIMIT 50\n
///   3. SELECT <field>, COUNT(*) FROM <table> GROUP BY <field> ORDER BY COUNT(*) DESC LIMIT 10\n
/// ```
///
/// F-PML2-MED-001 fix: skeleton #2 now uses `<severity_values>` placeholder instead of
/// hardcoded `('high', 'critical')` literals. Severity literal casing is per-sensor
/// (crowdstrike: Title-case, armis: UPPER-case, cyberint: lowercase); hardcoding any
/// single casing breaks all other sensors silently (0 rows, no error).
/// The SEVERITY CASING WARNING added to the description directs agents to use
/// prism_describe's `example_query` field for correct per-sensor casing.
///
/// Load-bearing: reverting to the old skeletons (LIMIT 25, `= 'HIGH'`, no COUNT(*) / ORDER BY)
/// fails all three assertions.
#[test]
fn test_BC_2_10_009_query_tool_description_l1_primer_skeleton_shapes() {
    let catalog = PrismServer::production_tool_catalog();

    let query_tool = catalog.iter().find(|t| t.name.as_ref() == "query").expect(
        "BC-2.10.009: 'query' tool must be registered in the production tool catalog; \
             not found. Verify 'query' is in LIVE_TOOLS list.",
    );

    let description = query_tool
        .description
        .as_deref()
        .expect("BC-2.10.009: 'query' tool must have a non-empty description");

    // ── Skeleton 1: count-recent — must use COUNT(*) + time-window ──────────────
    //
    // BC-2.10.009 §L1 primer: first skeleton shows a time-windowed count query.
    assert!(
        description.contains("COUNT(*)"),
        "BC-2.10.009 AC-002 [L1 skeleton 1 — count-recent]: query tool description \
         MUST contain 'COUNT(*)'. \
         Got description (first 400 chars): {:?}",
        &description[..description.len().min(400)]
    );

    assert!(
        description.contains("NOW()") && description.contains("INTERVAL"),
        "BC-2.10.009 AC-002 [L1 skeleton 1 — count-recent]: query tool description \
         MUST contain time-window clause 'NOW() - INTERVAL'. \
         Got description (first 400 chars): {:?}",
        &description[..description.len().min(400)]
    );

    // ── Skeleton 2: severity filter — must use IN clause + LIMIT 50 ─────────────
    //
    // BC-2.10.009 §L1 primer: severity filter uses multi-value IN predicate and LIMIT 50.
    // F-PML2-MED-001 fix: severity values are now a placeholder `<severity_values>` —
    // hardcoding any single casing breaks sensors that use a different casing (0 rows,
    // no error). Agents must use prism_describe's example_query for correct per-sensor casing.
    assert!(
        description.contains("IN (<severity_values>)"),
        "BC-2.10.009 AC-002 [L1 skeleton 2 — severity]: query tool description \
         MUST contain `IN (<severity_values>)` (F-PML2-MED-001: placeholder, not hardcoded casing). \
         Got description (first 400 chars): {:?}",
        &description[..description.len().min(400)]
    );

    assert!(
        description.contains("LIMIT 50"),
        "BC-2.10.009 AC-002 [L1 skeleton 2 — severity]: query tool description \
         MUST contain 'LIMIT 50'. \
         Got description (first 400 chars): {:?}",
        &description[..description.len().min(400)]
    );

    // ── Skeleton 3: aggregate — must use ORDER BY COUNT(*) DESC + LIMIT 10 ──────
    //
    // BC-2.10.009 §L1 primer: aggregate skeleton shows descending-count sort and LIMIT 10.
    assert!(
        description.contains("ORDER BY COUNT(*) DESC"),
        "BC-2.10.009 AC-002 [L1 skeleton 3 — aggregate]: query tool description \
         MUST contain 'ORDER BY COUNT(*) DESC'. \
         Got description (first 400 chars): {:?}",
        &description[..description.len().min(400)]
    );

    assert!(
        description.contains("LIMIT 10"),
        "BC-2.10.009 AC-002 [L1 skeleton 3 — aggregate]: query tool description \
         MUST contain 'LIMIT 10'. \
         Got description (first 400 chars): {:?}",
        &description[..description.len().min(400)]
    );
}

// ─── Round-7: SafetyEnvelope on prism_describe (HIGH, BC-2.10.012 §Response envelope) ──

/// Round-7 HIGH (BC-2.10.012 §Response envelope):
/// `handle_prism_describe` MUST wrap its response in a `SafetyEnvelopeBuilder`
/// with `_meta.trust_level == "internal"`, consistent with all other Prism MCP
/// tools (query, check_sensor_health, etc.).
///
/// The tool MUST also declare `output_schema = schema_for_type::<ResponseEnvelopeSchema>()`
/// so MCP clients know the response carries `_meta` / `results` / `content` /
/// `structuredContent` fields. (The outputSchema declaration is tested separately via the
/// production tool catalog; this test focuses on the runtime response shape.)
///
/// Load-bearing: `handle_prism_describe` wraps its response with `SafetyEnvelopeBuilder::wrap`,
/// which injects `_meta.trust_level = "internal"`, `results`, `structuredContent`, and
/// `_meta.safety_flags`. Reverting to a bare `PrismDescribeResponse` serialization causes all
/// `_meta` assertions below to fail.
#[tokio::test]
async fn test_BC_2_10_012_prism_describe_response_uses_safety_envelope_with_trust_level_internal() {
    let config_manager = make_config_manager_acme_crowdstrike();

    // Call handle_prism_describe for a valid client — single-tenant config_manager path.
    let result = handle_prism_describe(
        "crowdstrike".to_string(),
        None,
        Some(&config_manager),
        None, // no audit_writer
    )
    .await
    .expect(
        "BC-2.10.012 SafetyEnvelope: handle_prism_describe must return Ok for valid client; \
         pre-condition: this test drives the envelope shape, not an error path",
    );

    // Extract the JSON body from the response content.
    let content_json: String = result
        .content
        .iter()
        .filter_map(|c| c.as_text().map(|t| t.text.clone()))
        .collect::<Vec<_>>()
        .join("");

    let parsed: serde_json::Value = serde_json::from_str(&content_json)
        .expect("BC-2.10.012 SafetyEnvelope: handle_prism_describe response must be valid JSON");

    // BC-2.10.012 §Response envelope assertions:
    // The response MUST carry `_meta.trust_level == "internal"`.
    // Load-bearing: reverting to a bare PrismDescribeResponse (no SafetyEnvelopeBuilder::wrap)
    // causes `parsed["_meta"]` to be null and the unwrap_or_else below to fire.
    let meta = parsed
        .get("_meta")
        .and_then(|v| v.as_object())
        .unwrap_or_else(|| {
            panic!(
                "BC-2.10.012 §Response envelope: prism_describe response MUST contain a \
             '_meta' object (SafetyEnvelope shape). \
             Got parsed keys: {:?}",
                parsed.as_object().map(|o| o.keys().collect::<Vec<_>>())
            )
        });

    assert_eq!(
        meta.get("trust_level").and_then(|v| v.as_str()),
        Some("internal"),
        "BC-2.10.012 §Response envelope: _meta.trust_level MUST be 'internal' \
         (schema-catalog data is Prism-generated, not sensor-sourced external content). \
         Got _meta: {:?}",
        meta
    );

    // Structural completeness: the envelope must also carry `results` and `structuredContent`
    // fields (BC-2.09.007 / BC-2.09.008 outputSchema contract).
    assert!(
        parsed.get("results").is_some(),
        "BC-2.10.012 §Response envelope: SafetyEnvelope must include 'results' field. \
         Got keys: {:?}",
        parsed.as_object().map(|o| o.keys().collect::<Vec<_>>())
    );

    assert!(
        parsed.get("structuredContent").is_some() || parsed.get("structured_content").is_some(),
        "BC-2.10.012 §Response envelope: SafetyEnvelope must include 'structuredContent' field. \
         Got keys: {:?}",
        parsed.as_object().map(|o| o.keys().collect::<Vec<_>>())
    );

    // BC-2.09.008 — load-bearing assertion (Finding 2 / TD-VSDD-059):
    // _meta.safety_flags MUST be present and be an Array.
    //
    // Load-bearing: a hand-rolled _meta that inserts only `trust_level` (not using
    // SafetyEnvelopeBuilder::wrap) would omit `safety_flags` and cause this to fail.
    assert!(
        meta.get("safety_flags").is_some(),
        "BC-2.09.008: _meta.safety_flags MUST be present (even as an empty array). \
         Got _meta keys: {:?}",
        meta.keys().collect::<Vec<_>>()
    );
    assert!(
        meta.get("safety_flags")
            .and_then(|v| v.as_array())
            .is_some(),
        "BC-2.09.008: _meta.safety_flags MUST be a JSON array. \
         Got: {:?}",
        meta.get("safety_flags")
    );
}

// ─── Round-7: dispatch invalid-client_id error message (MED, BC-2.10.013 EC-10-033) ──

/// Round-7 MED (BC-2.10.013 EC-10-033 — Invalid client_id in resource URI):
/// `dispatch_read_resource` called with URI `prismql://schema/acme/../etc` (path
/// traversal) or `prismql://schema/` (empty client_id) MUST return an MCP error
/// whose message contains "Invalid client_id in resource URI" — NOT the generic
/// "Unknown or unsupported resource URI" fallback.
///
/// BC-2.10.013 EC-10-033: "path-traversal client_id → resource error:
/// 'Invalid client_id in resource URI'"
///
/// Load-bearing: if `dispatch_read_resource` loses the explicit else-branch for invalid
/// client_id (replacing with a generic fallthrough), the error message reverts to
/// "Unknown or unsupported resource URI" and these assertions fail.
#[tokio::test]
async fn test_BC_2_10_013_dispatch_invalid_client_id_returns_specific_error_message() {
    use prism_mcp::context::PrismContext;

    let config_manager = make_config_manager_acme_crowdstrike();
    let context = Arc::new(PrismContext::new());

    // ── Case 1: path-traversal client_id ("acme/../etc") ─────────────────────────
    //
    // BC-2.10.013 EC-10-033: must return the BC-mandated "Invalid client_id in resource URI",
    // NOT the generic "Unknown or unsupported resource URI" fallthrough.
    let traversal_result = dispatch_read_resource(
        "prismql://schema/acme/../etc",
        &context,
        None,
        Some(&config_manager),
    )
    .await;

    assert!(
        traversal_result.is_err(),
        "BC-2.10.013 EC-10-033: dispatch_read_resource('prismql://schema/acme/../etc') must \
         return Err (path-traversal client_id is invalid); got Ok"
    );

    let traversal_err = traversal_result.unwrap_err();
    let traversal_msg = traversal_err.message.to_string();

    // Load-bearing: the BC-canonical EC-10-033 error string must be returned.
    assert!(
        traversal_msg.contains("Invalid client_id in resource URI"),
        "BC-2.10.013 EC-10-033: dispatch_read_resource with path-traversal URI \
         'prismql://schema/acme/../etc' MUST return error message containing \
         'Invalid client_id in resource URI' (BC-canonical EC-10-033 error string). \
         Got error message: {:?}",
        traversal_msg
    );

    // DI-006: the raw path-traversal payload "acme/../etc" must NOT appear in the error.
    assert!(
        !traversal_msg.contains("acme/../etc"),
        "BC-2.10.013 EC-10-033 DI-006: error message MUST NOT echo the raw path-traversal \
         client_id 'acme/../etc' (prompt-injection defense). Got error: {:?}",
        traversal_msg
    );

    // ── Case 2: empty client_id ("prismql://schema/") ────────────────────────────
    //
    // BC-2.10.013 EC-10-033: empty client_id (URI ends at schema/) is also invalid.
    let empty_result =
        dispatch_read_resource("prismql://schema/", &context, None, Some(&config_manager)).await;

    assert!(
        empty_result.is_err(),
        "BC-2.10.013 EC-10-033: dispatch_read_resource('prismql://schema/') with empty \
         client_id must return Err; got Ok"
    );

    let empty_err = empty_result.unwrap_err();
    let empty_msg = empty_err.message.to_string();

    // Load-bearing: must return the BC-canonical EC-10-033 error string.
    assert!(
        empty_msg.contains("Invalid client_id in resource URI"),
        "BC-2.10.013 EC-10-033: dispatch_read_resource with empty client_id \
         'prismql://schema/' MUST return error message containing \
         'Invalid client_id in resource URI'. \
         Got error message: {:?}",
        empty_msg
    );
}

// ─── Round-8: invalid-char client_id (LOW, BC-2.10.013 EC-10-033) ──────────────────

/// Round-8 LOW (BC-2.10.013 EC-10-033 — invalid-char client_id gets canonical error):
/// `dispatch_read_resource("prismql://schema/acme!")` MUST return the canonical
/// EC-10-033 error "Invalid client_id in resource URI", NOT a deeper different-string
/// rejection from inside `render_pql_schema_resource`.
///
/// Load-bearing: `dispatch_read_resource` validates `client_id` with `OrgSlug::new`
/// before calling `render_pql_schema_resource`. If that guard is replaced with manual
/// `is_empty`/`contains('/')`/`contains("..")` checks, `"acme!"` passes them and falls
/// through to a deeper render-path rejection with a different error string.
///
/// BC trace: BC-2.10.013 EC-10-033, DI-006.
#[tokio::test]
async fn test_BC_2_10_013_dispatch_invalid_char_client_id_returns_canonical_ec10033_error() {
    use prism_mcp::context::PrismContext;

    let config_manager = make_config_manager_acme_crowdstrike();
    let context = Arc::new(PrismContext::new());

    // ── Case: invalid-char client_id ("acme!") ──────────────────────────────
    //
    // "acme!" is rejected by OrgSlug::new in dispatch_read_resource, which returns
    // the canonical EC-10-033 error before reaching render_pql_schema_resource.
    let result = dispatch_read_resource(
        "prismql://schema/acme!",
        &context,
        None,
        Some(&config_manager),
    )
    .await;

    assert!(
        result.is_err(),
        "BC-2.10.013 EC-10-033: dispatch_read_resource('prismql://schema/acme!') must \
         return Err (invalid-char client_id is not a valid OrgSlug); got Ok"
    );

    let err = result.unwrap_err();
    let err_msg = err.message.to_string();

    // Load-bearing: must return the BC-canonical EC-10-033 string.
    assert!(
        err_msg.contains("Invalid client_id in resource URI"),
        "BC-2.10.013 EC-10-033: dispatch_read_resource('prismql://schema/acme!') \
         MUST return error message containing 'Invalid client_id in resource URI' \
         (canonical EC-10-033 string). \
         Got error message: {:?}",
        err_msg
    );

    // DI-006: do NOT echo the raw client_id — even 'acme!' is attacker-controlled input.
    assert!(
        !err_msg.contains("acme!"),
        "BC-2.10.013 EC-10-033 DI-006: error message MUST NOT echo the raw client_id 'acme!' \
         (prompt-injection defense). Got error: {:?}",
        err_msg
    );
}
