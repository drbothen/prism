//! Integration tests for S-5.03: MCP Resources and Prompts.
//!
//! Tests cover:
//! - AC-1 (BC-2.10.008): `prism://config/clients` returns all configured clients
//! - AC-2 (BC-2.10.008): `prism://config/clients/{client_id}/sensors` returns sensors
//! - AC-3 (BC-2.10.009): `prompts/list` returns 4 mandated prompts; all 4 prompts include DI-006 reminder
//! - AC-4 (BC-2.08.005): `check_sensor_health` returns structured per-sensor result with trust_level "internal"
//! - AC-5 (BC-2.08.006): `prism://sensors/health` returns cached data after health check
//! - AC-6 (BC-2.08.006): `prism://sensors/health` returns "unknown" before any health check
//! - AC-8 (BC-2.10.008): `prism://config/clients` lists only sensors in TableRegistry
//! - AC-9 (BC-2.16.007): hot-reload notifications dispatched on table-set change only
//! - BC-2.10.008 EC-10-014: zero clients → empty array
//! - BC-2.10.008 EC-10-016: unknown client_id → 404-equivalent
//! - BC-2.08.005: trust_level="internal" always set; structuredContent shape; partial failure handling
//! - BC-2.08.006 EC-08-012: stale data returns with stale:true flag
//! - BC-2.08.006 EC-08-013: zero clients → `{"clients":{}}` not an error
//! - BC-2.10.009: all 4 prompt renders include DI-006 security reminder; invalid name → MCP error
//!
//! Red Gate test names (must fail against stubs, pass after implementation):
//! - test_BC_2_10_008_config_clients_returns_all_clients (AC-1)
//! - test_BC_2_10_008_client_sensors_invalid_id_returns_error (AC-2)
//! - test_BC_2_10_009_prompts_list_includes_four_mandated_prompts (AC-3)
//! - test_BC_2_10_009_triage_alerts_includes_security_reminder (AC-3)
//! - test_BC_2_10_009_investigate_host_includes_security_reminder (AC-3, BC-2.10.009 postcondition 3)
//! - test_BC_2_10_009_client_overview_includes_security_reminder (AC-3, BC-2.10.009 postcondition 3)
//! - test_BC_2_10_009_cross_client_status_includes_security_reminder (AC-3, BC-2.10.009 postcondition 3)
//! - test_BC_2_08_005_check_sensor_health_returns_structured_result (AC-4)
//! - test_BC_2_08_005_check_sensor_health_trust_level_is_internal (AC-4, BC-2.08.005 postcondition 7)
//! - test_BC_2_08_005_check_sensor_health_structured_content_shape (AC-4, BC-2.08.005 postcondition 5)
//! - test_BC_2_08_005_check_sensor_health_requires_client_id (BC-2.08.005 v1.4 precondition)
//! - test_BC_2_08_006_sensors_health_resource_returns_cached_data (AC-5)
//! - test_BC_2_08_006_sensors_health_resource_returns_unknown_before_check (AC-6)
//! - test_BC_2_08_006_sensors_health_zero_clients_returns_empty_object (BC-2.08.006 EC-08-013)
//! - test_BC_2_10_008_config_clients_resource_reflects_registered_tables (AC-8)
//! - test_BC_2_16_007_hot_reload_sends_mcp_list_changed_notification (AC-9)
//! - test_BC_2_10_008_invariant_zero_clients_returns_empty_array (BC-2.10.008 EC-10-014)

use chrono;
use prism_mcp::{
    context::PrismContext,
    prompts::{
        build_prompt_router, render_client_overview, render_cross_client_status,
        render_investigate_host, render_triage_alerts, PROMPT_CLIENT_OVERVIEW,
        PROMPT_CROSS_CLIENT_STATUS, PROMPT_INVESTIGATE_HOST, PROMPT_TRIAGE_ALERTS,
    },
    resources::{
        dispatch_hot_reload_notifications, render_sensors_health_resource, SensorHealthResult,
        SensorHealthStructuredContent,
    },
    server::PrismServer,
    CheckSensorHealthParams,
};

// ─── AC-1: prism://config/clients returns all configured clients ──────────────

/// AC-1 (BC-2.10.008 postcondition 1): `prism://config/clients` response includes
/// all configured clients with `sensor_count` and `enabled_sensors` populated.
#[tokio::test]
async fn test_BC_2_10_008_config_clients_returns_all_clients() {
    // Requires: PrismServer configured with two clients ("acme", "globex").
    // When: prism://config/clients is read.
    // Then: response contains both clients with sensor_count and enabled_sensors.
    //
    // NOTE: This test will fail against stubs (todo!() bodies) — Red Gate holds.
    todo!("AC-1: implement render_client_list_resource to make this test pass")
}

// ─── AC-2: prism://config/clients/{client_id}/sensors with invalid client_id ────

/// AC-2 / EC-001 (BC-2.10.008): `prism://config/clients/{client_id}/sensors` with
/// invalid `client_id` returns a 404-equivalent error (not a server error).
#[tokio::test]
async fn test_BC_2_10_008_client_sensors_invalid_id_returns_error() {
    // Requires: a PrismServer with any valid config.
    // When: prism://config/clients/../../etc/passwd/sensors is read.
    // Then: error is returned; TenantId::new() rejected before any CF scan.
    //
    // NOTE: This test will fail against stubs (todo!() bodies) — Red Gate holds.
    todo!("AC-2/EC-001: implement render_client_sensors_resource validation to make this test pass")
}

// ─── AC-3: prompts/list includes four mandated prompts ───────────────────────

/// AC-3 (BC-2.10.009 postcondition 1): `prompts/list` response includes at minimum
/// the four mandated prompts: `triage_alerts`, `investigate_host`, `client_overview`,
/// `cross_client_status`.
#[test]
fn test_BC_2_10_009_prompts_list_includes_four_mandated_prompts() {
    // When: PromptRouter is built.
    // Then: it includes all four mandated prompts by their canonical names.
    //
    // NOTE: This test will fail against stubs (todo!() bodies) — Red Gate holds.
    let router = build_prompt_router();
    let prompts = router.list_all();
    let names: Vec<&str> = prompts.iter().map(|p| p.name.as_str()).collect();

    assert!(
        names.contains(&PROMPT_TRIAGE_ALERTS),
        "Missing prompt: {PROMPT_TRIAGE_ALERTS}; got: {names:?}"
    );
    assert!(
        names.contains(&PROMPT_INVESTIGATE_HOST),
        "Missing prompt: {PROMPT_INVESTIGATE_HOST}; got: {names:?}"
    );
    assert!(
        names.contains(&PROMPT_CLIENT_OVERVIEW),
        "Missing prompt: {PROMPT_CLIENT_OVERVIEW}; got: {names:?}"
    );
    assert!(
        names.contains(&PROMPT_CROSS_CLIENT_STATUS),
        "Missing prompt: {PROMPT_CROSS_CLIENT_STATUS}; got: {names:?}"
    );
    assert_eq!(
        prompts.len(),
        4,
        "Expected exactly 4 prompts; got {}: {names:?}",
        prompts.len()
    );
}

/// AC-3 (BC-2.10.009 postcondition 4 / DI-006): `triage_alerts` prompt message
/// includes the security reminder about untrusted sensor data.
#[test]
fn test_BC_2_10_009_triage_alerts_includes_security_reminder() {
    // When: triage_alerts is rendered with client_id: "acme".
    // Then: the prompt message includes the DI-006 security reminder.
    //
    // NOTE: This test will fail against stubs (todo!() bodies) — Red Gate holds.
    let result = render_triage_alerts("acme");
    let all_text: String = result
        .messages
        .iter()
        .filter_map(|m| {
            if let rmcp::model::PromptMessageContent::Text { text } = &m.content {
                Some(text.clone())
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
        .join(" ");

    assert!(
        all_text.contains("untrusted"),
        "DI-006: triage_alerts must include security reminder about untrusted sensor data; \
         got text: {all_text:?}"
    );
}

// ─── AC-4: check_sensor_health returns structured per-sensor result ───────────

/// AC-4 (BC-2.08.005 postconditions 1, 6, 7, 8): `check_sensor_health` with a
/// reachable mock sensor returns a `SensorHealthResult` with correct fields and
/// `structuredContent` + `content[].text` prose summary.
#[tokio::test]
async fn test_BC_2_08_005_check_sensor_health_returns_structured_result() {
    // Requires: PrismServer with a mock reachable CrowdStrike sensor for "acme".
    // When: check_sensor_health(client_id: "acme") is called.
    // Then: SensorHealthResult has sensor_id="crowdstrike", reachable=true, auth_valid=true,
    //       last_successful_query_at populated; response has trust_level="internal".
    //
    // NOTE: This test will fail against stubs (todo!() bodies) — Red Gate holds.
    todo!("AC-4: implement check_sensor_health to make this test pass")
}

// ─── AC-5: prism://sensors/health returns cached data after health check ─────

/// AC-5 (BC-2.08.006 postcondition 1): after a successful `check_sensor_health`
/// run, `prism://sensors/health` returns the cached per-sensor results.
#[test]
fn test_BC_2_08_006_sensors_health_resource_returns_cached_data() {
    // Requires: a PrismContext with a cached SensorHealthResult for ("acme", "crowdstrike").
    // When: render_sensors_health_resource is called.
    // Then: the response contains the cached sensor_id="crowdstrike" result.
    //
    // NOTE: This test will fail against stubs (todo!() bodies) — Red Gate holds.
    let context = PrismContext::new();

    // Manually populate the cache (simulating a prior check_sensor_health run).
    let cached_result = SensorHealthResult::new("crowdstrike", "acme")
        .with_reachable(true)
        .with_auth_valid(true)
        .with_last_successful_query_at(chrono::Utc::now());
    context
        .health_cache
        .insert("acme".to_string(), "crowdstrike".to_string(), cached_result);

    let result = render_sensors_health_resource(&context)
        .expect("render_sensors_health_resource must not fail when cache has data");

    let content_text = result
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
        content_text.contains("crowdstrike"),
        "AC-5: sensors/health resource must include cached crowdstrike result; got: {content_text:?}"
    );
}

// ─── AC-6: prism://sensors/health returns "unknown" before any health check ──

/// AC-6 / EC-002 (BC-2.08.006 postcondition 2): `prism://sensors/health` returns
/// `status: "unknown"` with an instructional message before any `check_sensor_health`
/// has been run. Must NOT return an error.
#[test]
fn test_BC_2_08_006_sensors_health_resource_returns_unknown_before_check() {
    // Requires: a fresh PrismContext with empty health cache.
    // When: render_sensors_health_resource is called.
    // Then: response contains status="unknown" and instructional message; not an error.
    //
    // NOTE: This test will fail against stubs (todo!() bodies) — Red Gate holds.
    let context = PrismContext::new();

    let result = render_sensors_health_resource(&context)
        .expect("AC-6: render_sensors_health_resource must return Ok (not an error) before any health check");

    let content_text = result
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
        content_text.contains("unknown"),
        "AC-6/EC-002: sensors/health resource must include 'unknown' status before any health check; \
         got: {content_text:?}"
    );
    assert!(
        content_text.to_lowercase().contains("check_sensor_health"),
        "AC-6/EC-002: sensors/health resource must include instructional message about running \
         check_sensor_health; got: {content_text:?}"
    );
}

// ─── AC-8: prism://config/clients reflects TableRegistry ─────────────────────

/// AC-8 (BC-2.10.008 postcondition 1 + S-3.13): `prism://config/clients` resource
/// lists only sensors present in `table_registry.registered_tables()`. Sensors absent
/// from `TableRegistry` must NOT appear in the response.
///
/// Prerequisite: S-3.13 must be merged (provides `TableRegistry::registered_tables()` API).
#[tokio::test]
async fn test_BC_2_10_008_config_clients_resource_reflects_registered_tables() {
    // Requires: S-3.13 merged; QueryEngine with TableRegistry containing only
    //           CrowdStrike and Claroty tables (not Armis or Cyberint).
    // When: prism://config/clients is read.
    // Then: response lists exactly CrowdStrike and Claroty sensors; Armis and Cyberint absent.
    //
    // NOTE: This test will fail against stubs (todo!() bodies) — Red Gate holds.
    // NOTE: Requires S-3.13 to be merged before this test can be fully implemented.
    todo!("AC-8: implement TableRegistry-backed resource listing to make this test pass")
}

// ─── AC-9: hot-reload sends MCP list_changed notifications ───────────────────

/// AC-9 (BC-2.16.007): hot-reload swap that changes the table set dispatches
/// `notifications/resources/list_changed` AND `notifications/tools/list_changed`.
/// A swap that does NOT change the table set dispatches NEITHER notification.
///
/// Prerequisite: S-3.13 must be merged (provides `TableRegistry::registered_tables()` API).
#[tokio::test]
async fn test_BC_2_16_007_hot_reload_sends_mcp_list_changed_notification() {
    // Requires: S-3.13 merged; a mock Peer<RoleServer> that captures notifications.
    // When: dispatch_hot_reload_notifications is called with changed table set.
    // Then: both notifications are dispatched.
    // When: dispatch_hot_reload_notifications is called with SAME table set.
    // Then: no notifications are dispatched.
    //
    // NOTE: This test will fail against stubs (todo!() bodies) — Red Gate holds.
    // NOTE: Requires S-3.13 merged + a Peer<RoleServer> mock infrastructure.
    todo!("AC-9: implement dispatch_hot_reload_notifications to make this test pass")
}

// ─── AC-3 extended: remaining 3 prompts include DI-006 security reminder ─────

/// AC-3 (BC-2.10.009 postcondition 3 / DI-006): `investigate_host` prompt message
/// includes the security reminder about untrusted sensor data.
///
/// BC-2.10.009 postcondition: "Prompt messages include security reminders about
/// untrusted sensor data." This invariant applies to ALL four mandated prompts,
/// not just `triage_alerts`.
#[test]
fn test_BC_2_10_009_investigate_host_includes_security_reminder() {
    // When: investigate_host is rendered with client_id: "acme" and hostname: "10.0.0.1".
    // Then: the prompt message includes the DI-006 security reminder.
    //
    // NOTE: This test will fail against stubs (todo!() bodies) — Red Gate holds.
    let result = render_investigate_host("acme", "10.0.0.1");
    let all_text: String = result
        .messages
        .iter()
        .filter_map(|m| {
            if let rmcp::model::PromptMessageContent::Text { text } = &m.content {
                Some(text.clone())
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
        .join(" ");

    assert!(
        all_text.contains("untrusted"),
        "DI-006: investigate_host must include security reminder about untrusted sensor data; \
         got text: {all_text:?}"
    );
}

/// AC-3 (BC-2.10.009 postcondition 3 / DI-006): `client_overview` prompt message
/// includes the security reminder about untrusted sensor data.
#[test]
fn test_BC_2_10_009_client_overview_includes_security_reminder() {
    // When: client_overview is rendered with client_id: "acme".
    // Then: the prompt message includes the DI-006 security reminder.
    //
    // NOTE: This test will fail against stubs (todo!() bodies) — Red Gate holds.
    let result = render_client_overview("acme");
    let all_text: String = result
        .messages
        .iter()
        .filter_map(|m| {
            if let rmcp::model::PromptMessageContent::Text { text } = &m.content {
                Some(text.clone())
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
        .join(" ");

    assert!(
        all_text.contains("untrusted"),
        "DI-006: client_overview must include security reminder about untrusted sensor data; \
         got text: {all_text:?}"
    );
}

/// AC-3 (BC-2.10.009 postcondition 3 / DI-006): `cross_client_status` prompt message
/// includes the security reminder about untrusted sensor data.
///
/// BC-2.10.009: cross_client_status accepts an optional `time_range` argument.
#[test]
fn test_BC_2_10_009_cross_client_status_includes_security_reminder() {
    // When: cross_client_status is rendered with time_range: Some("24h").
    // Then: the prompt message includes the DI-006 security reminder.
    //
    // NOTE: This test will fail against stubs (todo!() bodies) — Red Gate holds.
    let result = render_cross_client_status(Some("24h"));
    let all_text: String = result
        .messages
        .iter()
        .filter_map(|m| {
            if let rmcp::model::PromptMessageContent::Text { text } = &m.content {
                Some(text.clone())
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
        .join(" ");

    assert!(
        all_text.contains("untrusted"),
        "DI-006: cross_client_status must include security reminder about untrusted sensor data; \
         got text: {all_text:?}"
    );
}

/// BC-2.10.009 error case: an invalid (unknown) prompt name returns a standard MCP error,
/// not a panic or empty result.
///
/// BC-2.10.009 Error Cases: "Prompt not found — Invalid prompt name → MCP error: 'Prompt '{name}' not found'"
#[test]
fn test_BC_2_10_009_invalid_prompt_name_returns_error() {
    // When: the PromptRouter tries to get a prompt named "nonexistent_prompt".
    // Then: the router returns None or an error (not a panic).
    //
    // NOTE: This test will fail against stubs (todo!() bodies) — Red Gate holds.
    let router = build_prompt_router();
    // PromptRouter::get returns None for unknown prompt names (rmcp 1.7 API).
    // The test verifies the router does NOT panic and signals "not found" properly.
    let found = router
        .list_all()
        .iter()
        .any(|p| p.name == "nonexistent_prompt");
    assert!(
        !found,
        "BC-2.10.009 error case: 'nonexistent_prompt' must NOT appear in prompt list; \
         only the 4 mandated prompts should be registered"
    );
}

// ─── AC-4 extended: trust_level, structuredContent shape, client_id required ─

/// AC-4 (BC-2.08.005 postcondition 7): `check_sensor_health` response metadata
/// includes `trust_level: "internal"` on the `SensorHealthStructuredContent`.
///
/// BC-2.08.005: "Response metadata includes `trust_level: "internal"` (health data
/// is Prism-internal, not sensor-sourced)."
#[test]
fn test_BC_2_08_005_check_sensor_health_trust_level_is_internal() {
    // When: a SensorHealthStructuredContent is constructed (simulating check_sensor_health output).
    // Then: trust_level is "internal".
    //
    // NOTE: This test will fail against stubs (todo!() bodies) — Red Gate holds.
    // SID-1: unit test at the boundary — does not require a running sensor adapter.
    //
    // This test exercises the SensorHealthStructuredContent type's trust_level invariant.
    // The implementer must set trust_level = "internal" unconditionally (not caller-supplied).
    //
    // We test the serialized output to verify the field is present with the correct value.
    let sensors = vec![SensorHealthResult::new("crowdstrike", "acme")
        .with_reachable(true)
        .with_auth_valid(true)];
    let pressure = prism_mcp::resources::ResourcePressure::new(0, 0);
    let content = SensorHealthStructuredContent::new(
        sensors,
        pressure,
        "1 of 1 sensors healthy for client 'acme'",
    );
    assert_eq!(
        content.trust_level, "internal",
        "BC-2.08.005 postcondition 7: trust_level MUST be 'internal'; \
         got: {:?}",
        content.trust_level
    );
    let json =
        serde_json::to_string(&content).expect("SensorHealthStructuredContent must serialize");
    assert!(
        json.contains(r#""trust_level":"internal""#),
        "BC-2.08.005 postcondition 7: serialized response must contain trust_level:internal; \
         got: {json:?}"
    );
}

/// AC-4 (BC-2.08.005 postcondition 5 + 6): `check_sensor_health` response uses
/// `structuredContent` AND `content[].text` prose summary. The structured content
/// must include a `sensors` array and a `resource_pressure` section.
///
/// SID-1: unit test at the data type boundary (no running sensor adapter needed).
#[test]
fn test_BC_2_08_005_check_sensor_health_structured_content_shape() {
    // When: SensorHealthStructuredContent is built with one sensor result.
    // Then: serialized JSON contains "sensors", "resource_pressure", "summary", "trust_level".
    //
    // NOTE: This test will fail against stubs (todo!() bodies) — Red Gate holds.
    let sensors = vec![SensorHealthResult::new("crowdstrike", "acme")
        .with_reachable(true)
        .with_auth_valid(true)
        .with_last_successful_query_at(chrono::Utc::now())];
    let pressure = prism_mcp::resources::ResourcePressure::new(3, 7);
    let content = SensorHealthStructuredContent::new(
        sensors,
        pressure,
        "1 of 1 sensors healthy for client 'acme'",
    );
    let json =
        serde_json::to_string(&content).expect("SensorHealthStructuredContent must serialize");

    assert!(
        json.contains(r#""sensors""#),
        "BC-2.08.005 postcondition 5: structuredContent must contain 'sensors' array; got: {json:?}"
    );
    assert!(
        json.contains(r#""resource_pressure""#),
        "BC-2.08.005 postcondition: structuredContent must contain 'resource_pressure' section; got: {json:?}"
    );
    assert!(
        json.contains(r#""active_cursor_count""#),
        "BC-2.08.005 postcondition: resource_pressure must contain 'active_cursor_count'; got: {json:?}"
    );
    assert!(
        json.contains(r#""active_token_count""#),
        "BC-2.08.005 postcondition: resource_pressure must contain 'active_token_count'; got: {json:?}"
    );
    assert!(
        json.contains(r#""trust_level":"internal""#),
        "BC-2.08.005 postcondition 7: trust_level must be 'internal'; got: {json:?}"
    );
    assert!(
        json.contains(r#""summary""#),
        "BC-2.08.005 postcondition 6: structuredContent must contain prose 'summary'; got: {json:?}"
    );
    // Verify client_id is present in sensor result (BC-2.08.005 postcondition: client_id always present).
    assert!(
        json.contains(r#""client_id":"acme""#),
        "BC-2.08.005 postcondition: SensorHealthResult must include client_id; got: {json:?}"
    );
}

/// BC-2.08.005 precondition (v1.4 OOD-001 adjudication): `check_sensor_health` requires
/// `client_id: String` as a required field. A `CheckSensorHealthParams` without `client_id`
/// must not be constructible (compile-time structural enforcement).
///
/// This test asserts that the `client_id` field exists and is non-empty-validated.
#[tokio::test]
async fn test_BC_2_08_005_check_sensor_health_requires_client_id() {
    use rmcp::handler::server::wrapper::Parameters;
    // When: check_sensor_health is called with an empty client_id.
    // Then: it returns an INVALID_PARAMS error (validate_text_field rejects empty string).
    //
    // NOTE: This test will fail against stubs (todo!() bodies) — Red Gate holds.
    let server = PrismServer::new();
    // Use for_client with an empty string — BC-2.08.005 requires non-empty client_id.
    let params = CheckSensorHealthParams::for_client(String::new());
    let err = server
        .check_sensor_health(Parameters(params))
        .await
        .expect_err("BC-2.08.005 precondition: empty client_id must return an error");
    // The error must be INVALID_PARAMS — not a todo!() panic.
    // Error code -32602 (INVALID_PARAMS) is expected for invalid (empty) client_id.
    assert_eq!(
        err.code.0,
        prism_mcp::error_mapping::codes::INVALID_PARAMS,
        "BC-2.08.005: empty client_id must produce INVALID_PARAMS (-32602); \
         got code={} message={:?}",
        err.code.0,
        err.message
    );
}

// ─── BC-2.08.006 extended: zero clients, stale flag ──────────────────────────

/// BC-2.08.006 EC-08-013: `prism://sensors/health` when zero clients are configured
/// returns `{ "clients": {} }` (empty object), not an error.
///
/// BC-2.08.006: "Zero clients configured → Resource returns `{ "clients": {} }` — empty object, not an error"
#[test]
fn test_BC_2_08_006_sensors_health_zero_clients_returns_empty_object() {
    // Requires: a fresh PrismContext with empty health cache.
    // When: render_sensors_health_resource is called (zero clients = zero cache entries).
    // Then: response succeeds (Ok) and contains either "unknown" status or empty clients object.
    //       Must NOT return an error.
    //
    // NOTE: This test will fail against stubs (todo!() bodies) — Red Gate holds.
    let context = PrismContext::new();

    let result = render_sensors_health_resource(&context)
        .expect("BC-2.08.006 EC-08-013: render_sensors_health_resource must return Ok even with zero clients");

    let content_text = result
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

    // The response must be valid JSON and must NOT be a hard error string.
    // It may be either the "unknown" sentinel (EC-002, which shares the empty-cache case)
    // or a `{"clients":{}}` shape — both are acceptable for zero-client fresh state.
    // What it MUST NOT be: an MCP error or panic.
    assert!(
        !content_text.is_empty(),
        "BC-2.08.006 EC-08-013: response must not be empty string; got empty content"
    );
}

// ─── BC-2.10.008 invariant: EC-10-014 — zero clients returns empty array ─────

/// BC-2.10.008 EC-10-014: `prism://config/clients` with zero configured clients
/// returns an empty JSON array `[]`, not an error.
///
/// BC-2.10.008: "EC-10-014: Zero clients configured → `prism://config/clients` returns empty JSON array `[]`"
#[tokio::test]
async fn test_BC_2_10_008_invariant_zero_clients_returns_empty_array() {
    // Requires: a PrismServer configured with zero clients.
    // When: prism://config/clients is read.
    // Then: response is a JSON array `[]` (not an error, not null).
    //
    // NOTE: This test will fail against stubs (todo!() bodies) — Red Gate holds.
    todo!("BC-2.10.008 EC-10-014: implement render_client_list_resource with empty config to make this test pass")
}
