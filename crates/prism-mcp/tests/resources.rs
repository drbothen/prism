//! Integration tests for S-5.03: MCP Resources and Prompts.
//!
//! Tests cover:
//! - AC-1 (BC-2.10.008): `prism://config/clients` returns all configured clients
//! - AC-2 (BC-2.10.008): `prism://config/clients/{client_id}/sensors` returns sensors
//! - AC-3 (BC-2.10.009): `prompts/list` returns 4 mandated prompts; triage_alerts includes DI-006 reminder
//! - AC-4 (BC-2.08.005): `check_sensor_health` returns structured per-sensor result
//! - AC-5 (BC-2.08.006): `prism://sensors/health` returns cached data after health check
//! - AC-6 (BC-2.08.006): `prism://sensors/health` returns "unknown" before any health check
//! - AC-8 (BC-2.10.008): `prism://config/clients` lists only sensors in TableRegistry
//! - AC-9 (BC-2.16.007): hot-reload notifications dispatched on table-set change only
//!
//! Red Gate test names (must fail against stubs, pass after implementation):
//! - test_BC_2_10_008_config_clients_returns_all_clients (AC-1)
//! - test_BC_2_10_008_client_sensors_invalid_id_returns_error (AC-2)
//! - test_BC_2_10_009_prompts_list_includes_four_mandated_prompts (AC-3)
//! - test_BC_2_10_009_triage_alerts_includes_security_reminder (AC-3)
//! - test_BC_2_08_005_check_sensor_health_returns_structured_result (AC-4)
//! - test_BC_2_08_006_sensors_health_resource_returns_cached_data (AC-5)
//! - test_BC_2_08_006_sensors_health_resource_returns_unknown_before_check (AC-6)
//! - test_BC_2_10_008_config_clients_resource_reflects_registered_tables (AC-8)
//! - test_BC_2_16_007_hot_reload_sends_mcp_list_changed_notification (AC-9)

use chrono;
use prism_mcp::{
    context::PrismContext,
    prompts::{
        build_prompt_router, render_triage_alerts, PROMPT_CLIENT_OVERVIEW,
        PROMPT_CROSS_CLIENT_STATUS, PROMPT_INVESTIGATE_HOST, PROMPT_TRIAGE_ALERTS,
    },
    resources::{
        dispatch_hot_reload_notifications, render_sensors_health_resource, SensorHealthResult,
    },
    server::PrismServer,
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
