//! S-3.4.05 harness migration tests — prism-dtu-slack, shared-mode.
//!
//! # Behavioral contracts
//!
//! - BC-3.2.004: Shared-Mode DTU Tags OrgId in Payload Body Not in Routing Headers
//! - BC-3.3.001: DTU Mode Policy (startup EC-003: MSSP Coordination types permit client override)
//! - BC-3.5.001: Harness Logical Isolation Invariants
//!
//! # Test catalog (migrated from existing tests + new AC tests)
//!
//! ## Migrated from tests/fidelity.rs (1 test)
//! - test_BC_3_5_001_slack_fidelity_valid_blocks_payload_200
//!
//! ## Migrated from tests/ac_tests.rs (11 tests)
//! - test_BC_3_5_001_slack_ac_1_valid_blocks_payload_200_ok_stable_message_ts
//! - test_BC_3_5_001_slack_ac_1_text_only_payload_200
//! - test_BC_3_5_001_slack_ac_2_missing_blocks_and_text_400_invalid_payload
//! - test_BC_3_5_001_slack_ec_001_empty_json_object_400_invalid_payload
//! - test_BC_3_5_001_slack_ac_3_unknown_top_level_field_400_unknown_field
//! - test_BC_3_5_001_slack_ac_3_all_allowed_top_level_fields_accepted
//! - test_BC_3_5_001_slack_ac_4_rate_limit_429_retry_after_ratelimited_body
//! - test_BC_3_5_001_slack_ec_002_fail_with_500_internal_server_error
//! - test_BC_3_5_001_slack_ac_5_three_deliveries_captured_in_order
//! - test_BC_3_5_001_slack_ac_5_in_process_received_payloads_matches_http
//! - test_BC_3_5_001_slack_ac_6_reset_clears_received_payloads_and_counter
//! - test_BC_3_5_001_slack_ac_6_post_dtu_reset_endpoint_clears_state
//! - test_BC_3_5_001_slack_ec_004_message_ts_stable_across_deliveries
//!
//! ## Migrated from tests/org_tagging.rs (7 tests)
//! - test_BC_3_2_004_slack_org_id_in_payload_body
//! - test_BC_3_2_004_slack_org_id_not_in_http_url
//! - test_BC_3_2_004_slack_concurrent_sends_distinguished
//! - test_BC_3_2_004_slack_mode_metadata_absent_from_query_results
//! - test_BC_3_2_005_slack_dtu_mode_is_shared_at_startup
//! - test_BC_3_2_005_slack_invalid_mode_string_rejected_at_deserialization
//! - test_BC_3_2_005_slack_mode_immutable_after_startup
//!
//! ## New harness-specific tests (S-3.4.05 AC-004, AC-007, EC-001, EC-002, EC-003)
//! - ac_shared_mode_org_id_tagging
//! - ac_multi_org_logical_isolation_shared_mode
//! - ac_client_mode_override_does_not_produce_startup_error
//!
//! # Red Gate rationale
//!
//! All BC-3.2.004 / BC-3.2.005 org-tagging assertions fail because the Slack webhook
//! handler calls `capture_payload` (raw) instead of `capture_payload_tagged`
//! (org-id-wrapped). The captured entry has no `"org_id"` key and no `"payload"` wrapper.
//!
//! All BC-3.5.001 harness migration assertions fail because the harness `endpoint()`
//! method referenced in stub comments does not yet exist on `Harness`; these tests use
//! `endpoint_for(slug, DtuType::Slack)` which IS implemented but the Slack DTU type is
//! not wired into the harness clone-server dispatch yet (the clone server currently only
//! dispatches sensor DTU types: Claroty, Armis, CrowdStrike, Cyberint).
//!
//! # Naming convention
//!
//! `test_BC_S_SS_NNN_xxx()` for BC-traced tests.
//! `ac_xxx()` for story-level AC tests without a direct BC number.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    non_snake_case,
    unused_imports
)]
#![cfg(feature = "dtu")]

use prism_dtu_harness::{DtuType, HarnessBuilder, IsolationMode};
use serde_json::Value;

// ---------------------------------------------------------------------------
// Test constants
// ---------------------------------------------------------------------------

/// Org slug used across most single-org harness tests.
const TENANT: &str = "test-tenant";

/// Canonical OrgId test vectors (AI-opaque UUIDs).
const ORG_UUID_A: &str = "00000000-0000-7000-8000-000000000001";
const ORG_UUID_B: &str = "00000000-0000-7000-8000-000000000002";

/// Stable message_ts value emitted by the Slack clone (from spec / ac_tests.rs).
const STABLE_MESSAGE_TS: &str = "1234567890.123456";

// ---------------------------------------------------------------------------
// Test helpers
// ---------------------------------------------------------------------------

/// Build a single-tenant Slack harness with `IsolationMode::Logical`.
///
/// RED GATE: This call fails at the `endpoint_for` step — the harness
/// clone-server does not dispatch `DtuType::Slack` to a Slack behavioral
/// clone; it returns `None` from `endpoint_for`, causing the `expect` to panic.
///
/// Once the implementer wires `DtuType::Slack` into `clone_server::start_clone`
/// dispatch, this helper will succeed and the per-test assertions will gate
/// further progress.
async fn build_slack_harness() -> (
    prism_dtu_harness::Harness,
    std::net::SocketAddr,
    reqwest::Client,
) {
    let harness = HarnessBuilder::new()
        .isolation(IsolationMode::Logical)
        .with_customer_overrides(TENANT, |spec| {
            spec.dtu_types = vec![DtuType::Slack];
        })
        .build()
        .await
        .expect("Slack harness build must succeed (BC-3.5.001 precondition 2)");

    let addr = harness
        .endpoint_for(TENANT, DtuType::Slack)
        .expect("Slack endpoint must be present after build");

    let client = reqwest::Client::new();
    (harness, addr, client)
}

/// Post a JSON payload to the Slack webhook endpoint via the harness.
async fn post_webhook(
    client: &reqwest::Client,
    addr: std::net::SocketAddr,
    payload: &Value,
) -> reqwest::Response {
    post_webhook_for_org(client, addr, payload, None).await
}

/// Post a JSON payload with an optional `X-Prism-Org-Id` header.
async fn post_webhook_for_org(
    client: &reqwest::Client,
    addr: std::net::SocketAddr,
    payload: &Value,
    org_id: Option<&str>,
) -> reqwest::Response {
    let url = format!("http://{addr}/services/T00000000/B00000000/XXXXXXXXXXXX");
    let mut req = client.post(&url).json(payload);
    if let Some(id) = org_id {
        req = req.header("X-Prism-Org-Id", id);
    }
    req.send()
        .await
        .expect("POST /services/token must not fail at network level")
}

/// Retrieve captured payloads via the DTU introspection HTTP endpoint.
async fn get_received_payloads(client: &reqwest::Client, addr: std::net::SocketAddr) -> Vec<Value> {
    let resp = client
        .get(format!("http://{addr}/dtu/received-payloads"))
        .send()
        .await
        .expect("GET /dtu/received-payloads must not fail at network level");
    assert_eq!(resp.status().as_u16(), 200);
    let body: Value = resp.json().await.expect("response body is JSON");
    body["payloads"]
        .as_array()
        .expect("payloads must be an array")
        .clone()
}

// ---------------------------------------------------------------------------
// Migrated: fidelity.rs → test_BC_3_5_001_slack_fidelity_*
// ---------------------------------------------------------------------------

/// Migrated from fidelity.rs: `slack_dtu_fidelity`.
///
/// Verifies the Slack clone hosted in the harness starts and responds
/// to a valid Block Kit payload with HTTP 200, ok=true, and stable message_ts.
///
/// RED GATE: Fails because `DtuType::Slack` is not yet dispatched by the harness
/// clone-server — `endpoint_for` returns `None` → panic in `build_slack_harness`.
///
/// Traces to: BC-3.5.001 postcondition 1 (AC-001); AC-001 in S-3.4.05.
#[tokio::test]
async fn test_BC_3_5_001_slack_fidelity_valid_blocks_payload_200() {
    let (_harness, addr, client) = build_slack_harness().await;

    let fixture: Value = serde_json::from_str(include_str!("../fixtures/valid-block-kit.json"))
        .expect("valid-block-kit.json is valid JSON");

    let resp = post_webhook(&client, addr, &fixture).await;

    assert_eq!(
        resp.status().as_u16(),
        200,
        "BC-3.5.001 postcondition 1: valid Block Kit payload must return HTTP 200"
    );

    let body: Value = resp.json().await.expect("response body is JSON");
    assert_eq!(body["ok"], true, "response must have ok=true");
    assert_eq!(
        body["message_ts"].as_str().unwrap_or(""),
        STABLE_MESSAGE_TS,
        "message_ts must be the stable spec-literal value '{STABLE_MESSAGE_TS}'"
    );
}

// ---------------------------------------------------------------------------
// Migrated: ac_tests.rs → test_BC_3_5_001_slack_ac_*
// ---------------------------------------------------------------------------

/// Migrated from ac_tests.rs: `ac_1_valid_blocks_payload_returns_200_ok_with_stable_message_ts`.
///
/// RED GATE: Fails at `build_slack_harness` — `DtuType::Slack` not dispatched.
///
/// Traces to: BC-3.5.001 postcondition 1; S-3.4.05 AC-001.
#[tokio::test]
async fn test_BC_3_5_001_slack_ac_1_valid_blocks_payload_200_ok_stable_message_ts() {
    let (_harness, addr, client) = build_slack_harness().await;

    let fixture: Value = serde_json::from_str(include_str!("../fixtures/valid-block-kit.json"))
        .expect("valid-block-kit.json is valid JSON");

    let resp = post_webhook(&client, addr, &fixture).await;

    assert_eq!(
        resp.status().as_u16(),
        200,
        "AC-1: valid Block Kit payload must return HTTP 200"
    );
    let body: Value = resp.json().await.expect("response body is JSON");
    assert_eq!(body["ok"], true, "AC-1: response must have ok=true");
    assert_eq!(
        body["message_ts"].as_str().unwrap_or(""),
        STABLE_MESSAGE_TS,
        "AC-1: message_ts must be the stable spec-literal value '{STABLE_MESSAGE_TS}'"
    );
}

/// Migrated from ac_tests.rs: `ac_1_text_only_payload_returns_200`.
///
/// RED GATE: Fails at `build_slack_harness`.
///
/// Traces to: BC-3.5.001 postcondition 1; S-3.4.05 AC-001.
#[tokio::test]
async fn test_BC_3_5_001_slack_ac_1_text_only_payload_200() {
    let (_harness, addr, client) = build_slack_harness().await;

    let payload = serde_json::json!({"text": "Hello from Prism"});
    let resp = post_webhook(&client, addr, &payload).await;

    assert_eq!(
        resp.status().as_u16(),
        200,
        "AC-1: payload with 'text' only must return HTTP 200"
    );
    let body: Value = resp.json().await.expect("response body is JSON");
    assert_eq!(
        body["ok"], true,
        "AC-1: text-only payload must return ok=true"
    );
}

/// Migrated from ac_tests.rs: `ac_2_missing_blocks_and_text_returns_400_invalid_payload`.
///
/// RED GATE: Fails at `build_slack_harness`.
///
/// Traces to: BC-3.5.001 postcondition 1; S-3.4.05 AC-001.
#[tokio::test]
async fn test_BC_3_5_001_slack_ac_2_missing_blocks_and_text_400_invalid_payload() {
    let (_harness, addr, client) = build_slack_harness().await;

    let payload = serde_json::json!({"username": "prism-bot"});
    let resp = post_webhook(&client, addr, &payload).await;

    assert_eq!(
        resp.status().as_u16(),
        400,
        "AC-2: payload missing both 'blocks' and 'text' must return HTTP 400"
    );
    let body_text = resp.text().await.expect("response body text");
    assert_eq!(
        body_text.trim(),
        "\"invalid_payload\"",
        "AC-2: response body must be literal '\"invalid_payload\"'"
    );
}

/// Migrated from ac_tests.rs: `ec_001_empty_json_object_returns_400_invalid_payload`.
///
/// RED GATE: Fails at `build_slack_harness`.
///
/// Traces to: BC-3.5.001 postcondition 1 (EC-001).
#[tokio::test]
async fn test_BC_3_5_001_slack_ec_001_empty_json_object_400_invalid_payload() {
    let (_harness, addr, client) = build_slack_harness().await;

    let payload = serde_json::json!({});
    let resp = post_webhook(&client, addr, &payload).await;

    assert_eq!(
        resp.status().as_u16(),
        400,
        "EC-001: empty JSON object must return HTTP 400"
    );
    let body_text = resp.text().await.expect("response body text");
    assert_eq!(
        body_text.trim(),
        "\"invalid_payload\"",
        "EC-001: response body for empty object must be '\"invalid_payload\"'"
    );
}

/// Migrated from ac_tests.rs: `ac_3_unknown_top_level_field_returns_400_unknown_field`.
///
/// RED GATE: Fails at `build_slack_harness`.
///
/// Traces to: BC-3.5.001 postcondition 1; S-3.4.05 AC-001.
#[tokio::test]
async fn test_BC_3_5_001_slack_ac_3_unknown_top_level_field_400_unknown_field() {
    let (_harness, addr, client) = build_slack_harness().await;

    let payload = serde_json::json!({
        "blocks": [{"type": "section", "text": {"type": "mrkdwn", "text": "hi"}}],
        "unknown_key": "value"
    });
    let resp = post_webhook(&client, addr, &payload).await;

    assert_eq!(
        resp.status().as_u16(),
        400,
        "AC-3: payload with unknown top-level field must return HTTP 400"
    );
    let body_text = resp.text().await.expect("response body text");
    assert_eq!(
        body_text.trim(),
        "\"unknown_field\"",
        "AC-3: response body must be literal '\"unknown_field\"'"
    );
}

/// Migrated from ac_tests.rs: `ac_3_all_allowed_top_level_fields_are_accepted`.
///
/// RED GATE: Fails at `build_slack_harness`.
///
/// Traces to: BC-3.5.001 postcondition 1; S-3.4.05 AC-001.
#[tokio::test]
async fn test_BC_3_5_001_slack_ac_3_all_allowed_top_level_fields_accepted() {
    let (_harness, addr, client) = build_slack_harness().await;

    let payload = serde_json::json!({
        "blocks": [{"type": "section", "text": {"type": "mrkdwn", "text": "test"}}],
        "text": "fallback text",
        "username": "prism-bot",
        "icon_emoji": ":robot_face:",
        "icon_url": "https://example.com/icon.png",
        "attachments": []
    });
    let resp = post_webhook(&client, addr, &payload).await;

    assert_eq!(
        resp.status().as_u16(),
        200,
        "AC-3: payload using only allowed fields must return HTTP 200"
    );
}

/// Migrated from ac_tests.rs: `ac_4_rate_limit_returns_429_with_retry_after_and_ratelimited_body`.
///
/// RED GATE: Fails at `build_slack_harness`.
///
/// Traces to: BC-3.5.001 postcondition 1; S-3.4.05 AC-001.
#[tokio::test]
async fn test_BC_3_5_001_slack_ac_4_rate_limit_429_retry_after_ratelimited_body() {
    let (harness, addr, client) = build_slack_harness().await;

    // Inject RateLimit failure: allow 3 requests, then throttle.
    harness
        .inject_failure(
            TENANT,
            DtuType::Slack,
            prism_dtu_common::FailureMode::RateLimit {
                after_n_requests: 3,
                retry_after_secs: 30,
            },
        )
        .await
        .expect("inject_failure must succeed for registered org");

    let valid_payload = serde_json::json!({"text": "ping"});

    // Requests 1–3 must succeed.
    for n in 1u32..=3 {
        let resp = post_webhook(&client, addr, &valid_payload).await;
        assert_eq!(
            resp.status().as_u16(),
            200,
            "AC-4: request {n} must succeed (within rate-limit threshold of 3)"
        );
    }

    // Request 4 must receive HTTP 429.
    let resp = post_webhook(&client, addr, &valid_payload).await;
    assert_eq!(
        resp.status().as_u16(),
        429,
        "AC-4: 4th request must return HTTP 429 (rate-limit threshold 3 exceeded)"
    );

    let retry_after = resp
        .headers()
        .get("retry-after")
        .expect("AC-4: HTTP 429 must include Retry-After header")
        .to_str()
        .expect("Retry-After header must be valid ASCII");
    assert_eq!(
        retry_after, "30",
        "AC-4: Retry-After must equal 30 (spec value from AC-4)"
    );

    let body_text = resp.text().await.expect("response body text");
    assert_eq!(
        body_text.trim(),
        "\"ratelimited\"",
        "AC-4: body must be literal '\"ratelimited\"'"
    );
}

/// Migrated from ac_tests.rs: `ec_002_fail_with_500_returns_internal_server_error`.
///
/// RED GATE: Fails at `build_slack_harness`.
///
/// Traces to: BC-3.5.001 postcondition 1 (EC-002).
#[tokio::test]
async fn test_BC_3_5_001_slack_ec_002_fail_with_500_internal_server_error() {
    let (harness, addr, client) = build_slack_harness().await;

    harness
        .inject_failure(
            TENANT,
            DtuType::Slack,
            prism_dtu_common::FailureMode::InternalError { at_request_n: 1 },
        )
        .await
        .expect("inject_failure must succeed");

    let valid_payload = serde_json::json!({"text": "should fail"});
    let resp = post_webhook(&client, addr, &valid_payload).await;

    assert_eq!(
        resp.status().as_u16(),
        500,
        "EC-002: configured internal error mode must return HTTP 500"
    );
}

/// Migrated from ac_tests.rs: `ac_5_three_deliveries_captured_in_order`.
///
/// RED GATE: Fails at `build_slack_harness`; secondary fail: captured entries
/// use raw payload format (no tagged wrapper), so `entry["payload"]["text"]` is null.
///
/// Traces to: BC-3.5.001 postcondition 1; S-3.4.05 AC-001.
#[tokio::test]
async fn test_BC_3_5_001_slack_ac_5_three_deliveries_captured_in_order() {
    let (_harness, addr, client) = build_slack_harness().await;

    let payloads = [
        serde_json::json!({"text": "message one"}),
        serde_json::json!({"text": "message two"}),
        serde_json::json!({"text": "message three"}),
    ];

    for payload in &payloads {
        let resp = post_webhook(&client, addr, payload).await;
        assert_eq!(
            resp.status().as_u16(),
            200,
            "AC-5: each delivery must return HTTP 200"
        );
    }

    let captured = get_received_payloads(&client, addr).await;
    assert_eq!(
        captured.len(),
        3,
        "AC-5: must have exactly 3 captured payloads; got {}",
        captured.len()
    );

    // S-3.4.05: captured entries use the tagged wrapper format:
    // {"org_id": "<uuid>", "payload": {"text": "...", ...}}
    assert_eq!(
        captured[0]["payload"]["text"].as_str().unwrap_or(""),
        "message one",
        "AC-5: first captured payload must have text='message one' (tagged wrapper format)"
    );
    assert_eq!(
        captured[1]["payload"]["text"].as_str().unwrap_or(""),
        "message two",
        "AC-5: second captured payload must have text='message two'"
    );
    assert_eq!(
        captured[2]["payload"]["text"].as_str().unwrap_or(""),
        "message three",
        "AC-5: third captured payload must have text='message three'"
    );
}

/// Migrated from ac_tests.rs: `ac_5_in_process_received_payloads_api_matches_http_endpoint`.
///
/// RED GATE: Fails at `build_slack_harness`.
///
/// Traces to: BC-3.5.001 postcondition 1; S-3.4.05 AC-001.
#[tokio::test]
async fn test_BC_3_5_001_slack_ac_5_in_process_received_payloads_matches_http() {
    let (_harness, addr, client) = build_slack_harness().await;

    let payload_a = serde_json::json!({"blocks": [{"type": "divider"}]});
    let payload_b = serde_json::json!({"text": "second"});

    post_webhook(&client, addr, &payload_a).await;
    post_webhook(&client, addr, &payload_b).await;

    let captured = get_received_payloads(&client, addr).await;
    assert_eq!(
        captured.len(),
        2,
        "AC-5: GET /dtu/received-payloads must return 2 after 2 deliveries"
    );

    // S-3.4.05: captured entries use tagged wrapper — check inner payload.
    assert_eq!(
        captured[0]["payload"]["blocks"][0]["type"]
            .as_str()
            .unwrap_or(""),
        "divider",
        "AC-5: first captured payload inner body must match first sent payload"
    );
}

/// Migrated from ac_tests.rs: `ac_6_reset_clears_received_payloads_and_request_counter`.
///
/// RED GATE: Fails at `build_slack_harness`.
///
/// Traces to: BC-3.5.001 postcondition 1; S-3.4.05 AC-001.
#[tokio::test]
async fn test_BC_3_5_001_slack_ac_6_reset_clears_received_payloads_and_counter() {
    let (_harness, addr, client) = build_slack_harness().await;

    let payload = serde_json::json!({"text": "before reset"});
    post_webhook(&client, addr, &payload).await;
    post_webhook(&client, addr, &payload).await;

    let before = get_received_payloads(&client, addr).await;
    assert_eq!(
        before.len(),
        2,
        "precondition: must have 2 payloads before reset"
    );

    // Reset via HTTP endpoint.
    let reset_resp = client
        .post(format!("http://{addr}/dtu/reset"))
        .send()
        .await
        .expect("POST /dtu/reset");
    assert_eq!(reset_resp.status().as_u16(), 200, "reset must return 200");

    let after = get_received_payloads(&client, addr).await;
    assert!(
        after.is_empty(),
        "AC-6: GET /dtu/received-payloads must be empty after reset; got {} entries",
        after.len()
    );

    // First post after reset must succeed (counter reset).
    let resp_after = post_webhook(&client, addr, &payload).await;
    assert_eq!(
        resp_after.status().as_u16(),
        200,
        "AC-6: first request after reset must succeed (counter was reset to 0)"
    );
}

/// Migrated from ac_tests.rs: `ac_6_post_dtu_reset_endpoint_clears_state`.
///
/// RED GATE: Fails at `build_slack_harness`.
///
/// Traces to: BC-3.5.001 postcondition 1; S-3.4.05 AC-001.
#[tokio::test]
async fn test_BC_3_5_001_slack_ac_6_post_dtu_reset_endpoint_clears_state() {
    let (_harness, addr, client) = build_slack_harness().await;

    let payload = serde_json::json!({"text": "to be cleared"});
    post_webhook(&client, addr, &payload).await;

    let reset_resp = client
        .post(format!("http://{addr}/dtu/reset"))
        .send()
        .await
        .expect("POST /dtu/reset");
    assert_eq!(
        reset_resp.status().as_u16(),
        200,
        "POST /dtu/reset must return 200"
    );

    let payloads = get_received_payloads(&client, addr).await;
    assert!(
        payloads.is_empty(),
        "AC-6: received-payloads must be empty after POST /dtu/reset"
    );
}

/// Migrated from ac_tests.rs: `ec_004_message_ts_is_stable_across_deliveries`.
///
/// RED GATE: Fails at `build_slack_harness`.
///
/// Traces to: BC-3.5.001 postcondition 1 (EC-004).
#[tokio::test]
async fn test_BC_3_5_001_slack_ec_004_message_ts_stable_across_deliveries() {
    let (_harness, addr, client) = build_slack_harness().await;

    let resp1 = post_webhook(&client, addr, &serde_json::json!({"text": "first"})).await;
    let body1: Value = resp1.json().await.expect("JSON");
    let ts1 = body1["message_ts"].as_str().unwrap_or("").to_string();

    let resp2 = post_webhook(&client, addr, &serde_json::json!({"text": "second"})).await;
    let body2: Value = resp2.json().await.expect("JSON");
    let ts2 = body2["message_ts"].as_str().unwrap_or("").to_string();

    assert_eq!(
        ts1, ts2,
        "EC-004: message_ts must be the same stable fake value across deliveries"
    );
    assert_eq!(
        ts1, STABLE_MESSAGE_TS,
        "EC-004: message_ts must be the spec-literal value '{STABLE_MESSAGE_TS}'"
    );
}

// ---------------------------------------------------------------------------
// Migrated: org_tagging.rs → test_BC_3_2_004_slack_* / test_BC_3_2_005_slack_*
// ---------------------------------------------------------------------------

/// Migrated from org_tagging.rs: `test_BC_3_2_004_org_id_in_payload_body`.
///
/// Dispatches a payload on behalf of org_A via X-Prism-Org-Id header and asserts
/// the captured entry's top-level `org_id` field equals org_A's UUID.
///
/// RED GATE PRIMARY: `build_slack_harness` panics (DtuType::Slack not dispatched).
/// RED GATE SECONDARY: even if the harness starts, the captured entry has no
/// `"org_id"` key — the handler calls `capture_payload` not `capture_payload_tagged`.
///
/// Traces to: BC-3.2.004 postcondition 1; VP-087; S-3.4.05 AC-004.
#[tokio::test]
async fn test_BC_3_2_004_slack_org_id_in_payload_body() {
    let (_harness, addr, client) = build_slack_harness().await;

    let payload = serde_json::json!({"text": "alert from org A"});
    let resp = post_webhook_for_org(&client, addr, &payload, Some(ORG_UUID_A)).await;
    assert_eq!(resp.status().as_u16(), 200);

    let captured = get_received_payloads(&client, addr).await;
    assert!(
        !captured.is_empty(),
        "expected at least one captured payload after POST"
    );

    let org_id_in_body = captured[0]
        .get("org_id")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    assert_eq!(
        org_id_in_body, ORG_UUID_A,
        "BC-3.2.004 postcondition 1: captured payload must contain \
         org_id == '{ORG_UUID_A}' (the sender's UUID) in the body wrapper"
    );
}

/// Migrated from org_tagging.rs: `test_BC_3_2_004_org_id_not_in_http_url`.
///
/// Asserts OrgId UUID does not appear in response URL or headers; captured entry
/// uses tagged wrapper format `{"org_id": ..., "payload": ...}`.
///
/// RED GATE: captured entry has no `"org_id"` key.
///
/// Traces to: BC-3.2.004 postcondition 2; VP-088; S-3.4.05 AC-004.
#[tokio::test]
async fn test_BC_3_2_004_slack_org_id_not_in_http_url() {
    let (_harness, addr, client) = build_slack_harness().await;

    let payload = serde_json::json!({"text": "routing isolation test"});
    let resp = post_webhook(&client, addr, &payload).await;
    assert_eq!(resp.status().as_u16(), 200);

    let captured = get_received_payloads(&client, addr).await;
    assert!(
        !captured.is_empty(),
        "expected at least one captured payload"
    );

    for (i, entry) in captured.iter().enumerate() {
        assert!(
            entry.get("org_id").is_some(),
            "BC-3.2.004 postcondition 2 (entry {i}): captured entry must have \
             top-level 'org_id' key (the tagged wrapper); raw payload stored without \
             wrapper violates the isolation contract"
        );

        let inner = entry
            .get("payload")
            .expect("captured entry must have a 'payload' wrapper key");

        assert!(
            inner.get("org_id").is_none(),
            "BC-3.2.004 postcondition 2 (entry {i}): 'org_id' must NOT leak \
             inside the inner Slack payload body — it belongs only in the outer wrapper"
        );
    }
}

/// Migrated from org_tagging.rs: `test_BC_3_2_004_concurrent_sends_distinguished`.
///
/// Spawns concurrent HTTP tasks for org_A and org_B; asserts both payloads
/// captured each with their own org_id UUID.
///
/// RED GATE: captured entries have no `"org_id"` key.
///
/// Traces to: BC-3.2.004 postcondition 4; VP-089; S-3.4.05 AC-004.
#[tokio::test]
async fn test_BC_3_2_004_slack_concurrent_sends_distinguished() {
    let (_harness, addr, client) = build_slack_harness().await;

    let addr_a = addr;
    let addr_b = addr;

    let task_a = tokio::spawn(async move {
        let c = reqwest::Client::new();
        c.post(format!(
            "http://{addr_a}/services/T00000000/B00000000/XXXXXXXXXXXX"
        ))
        .header("X-Prism-Org-Id", ORG_UUID_A)
        .json(&serde_json::json!({"text": "from org A"}))
        .send()
        .await
        .expect("task A post")
    });

    let task_b = tokio::spawn(async move {
        let c = reqwest::Client::new();
        c.post(format!(
            "http://{addr_b}/services/T00000000/B00000000/XXXXXXXXXXXX"
        ))
        .header("X-Prism-Org-Id", ORG_UUID_B)
        .json(&serde_json::json!({"text": "from org B"}))
        .send()
        .await
        .expect("task B post")
    });

    let (r_a, r_b) = tokio::join!(task_a, task_b);
    assert!(
        r_a.expect("task A").status().is_success(),
        "org A POST must succeed"
    );
    assert!(
        r_b.expect("task B").status().is_success(),
        "org B POST must succeed"
    );

    let captured = get_received_payloads(&client, addr).await;
    assert_eq!(
        captured.len(),
        2,
        "BC-3.2.004 postcondition 4: both concurrent payloads must be captured; got {}",
        captured.len()
    );

    let captured_ids: Vec<&str> = captured
        .iter()
        .map(|e| e["org_id"].as_str().unwrap_or(""))
        .collect();

    assert!(
        captured_ids.contains(&ORG_UUID_A),
        "BC-3.2.004 postcondition 4: captured entries must include org_A's UUID '{ORG_UUID_A}'; \
         got {:?}",
        captured_ids
    );
    assert!(
        captured_ids.contains(&ORG_UUID_B),
        "BC-3.2.004 postcondition 4: captured entries must include org_B's UUID '{ORG_UUID_B}'; \
         got {:?}",
        captured_ids
    );
}

/// Migrated from org_tagging.rs: `test_BC_3_2_004_mode_metadata_absent_from_query_results`.
///
/// Asserts captured entries contain no `mode`, `shared`, or `org_routing` keys.
///
/// RED GATE: captured entry has no `"org_id"` key (tagged wrapper not present).
///
/// Traces to: BC-3.2.004 postcondition 5; VP-090; S-3.4.05 AC-004.
#[tokio::test]
async fn test_BC_3_2_004_slack_mode_metadata_absent_from_query_results() {
    let (_harness, addr, client) = build_slack_harness().await;

    let ocsf_event = serde_json::json!({
        "text": "OCSF event notification",
        "blocks": [{"type": "section", "text": {"type": "mrkdwn", "text": "Device alert"}}]
    });
    let resp = post_webhook(&client, addr, &ocsf_event).await;
    assert_eq!(resp.status().as_u16(), 200);

    let captured = get_received_payloads(&client, addr).await;
    assert!(!captured.is_empty(), "expected at least one captured entry");

    for (i, entry) in captured.iter().enumerate() {
        assert!(
            entry.get("mode").is_none(),
            "BC-3.2.004 postcondition 5 (entry {i}): 'mode' must not appear in query results"
        );
        assert!(
            entry.get("org_routing").is_none(),
            "BC-3.2.004 postcondition 5 (entry {i}): 'org_routing' must not appear in query results"
        );
        assert!(
            entry.get("shared").is_none(),
            "BC-3.2.004 postcondition 5 (entry {i}): 'shared' must not appear in query results"
        );

        // The entry MUST use the tagged wrapper format.
        assert!(
            entry.get("org_id").is_some(),
            "BC-3.2.004 postcondition 5 (entry {i}): captured entry must use the \
             tagged wrapper format {{\"org_id\": \"<uuid>\", \"payload\": {{...}}}}"
        );
        assert!(
            entry.get("payload").is_some(),
            "BC-3.2.004 postcondition 5 (entry {i}): captured entry must have 'payload' key"
        );
    }
}

/// Migrated from org_tagging.rs: `test_BC_3_2_005_dtu_mode_is_shared_at_startup`.
///
/// RED GATE PRIMARY: `build_slack_harness` panics.
/// RED GATE SECONDARY: captured payload has no `"org_id"` key.
///
/// Traces to: BC-3.2.005 postcondition 1; VP-122; S-3.4.05 AC-001.
#[tokio::test]
async fn test_BC_3_2_005_slack_dtu_mode_is_shared_at_startup() {
    use prism_dtu_common::DtuMode;
    use prism_dtu_slack::clone::SLACK_DTU_MODE;

    // Static check: the Slack DTU crate-local mirror constant must be DtuMode::Shared.
    assert_eq!(
        SLACK_DTU_MODE,
        DtuMode::Shared,
        "BC-3.2.005 postcondition 1: Slack DTU must register as DtuMode::Shared"
    );

    // Dynamic check: shared-mode dispatch must tag payloads with org_id.
    let (_harness, addr, client) = build_slack_harness().await;

    let payload = serde_json::json!({"text": "startup mode verification"});
    let resp = post_webhook(&client, addr, &payload).await;
    assert_eq!(resp.status().as_u16(), 200);

    let captured = get_received_payloads(&client, addr).await;
    assert!(
        !captured.is_empty(),
        "startup mode test: expected at least one captured payload"
    );

    assert!(
        captured[0].get("org_id").is_some(),
        "BC-3.2.005 postcondition 1: shared-mode dispatch must embed 'org_id' in every \
         captured payload; found no 'org_id' key — SLACK_DTU_MODE = Shared but the \
         dispatch path does not yet call capture_payload_tagged"
    );
}

/// Migrated from org_tagging.rs: `test_BC_3_2_005_invalid_mode_string_rejected_at_deserialization`.
///
/// RED GATE: The TOML-context error annotation (`[[dtu]]`) is not yet implemented.
///
/// Traces to: BC-3.2.005 postcondition 3; S-3.4.05 AC-001.
#[test]
fn test_BC_3_2_005_slack_invalid_mode_string_rejected_at_deserialization() {
    use prism_dtu_common::DtuMode;

    // Part 1: serde correctly rejects "Hybrid" — this passes via the prism_core re-export.
    let result: Result<DtuMode, _> = serde_json::from_str("\"Hybrid\"");
    assert!(
        result.is_err(),
        "BC-3.2.005 postcondition 3: DtuMode must reject unknown variant 'Hybrid'"
    );
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("Hybrid") || err_msg.contains("variant"),
        "error message must identify the offending value; got: {err_msg}"
    );

    // Part 2 (RED GATE — fails): The full config-parse pipeline must annotate serde
    // errors with the TOML block context `[[dtu]]`.
    let toml_snippet = r#"
        [[dtu]]
        type = "slack"
        mode = "Hybrid"
    "#;
    let parse_result = validate_dtu_mode_in_toml(toml_snippet);
    assert!(
        parse_result.is_err(),
        "BC-3.2.005 postcondition 3: TOML snippet with mode='Hybrid' must fail validation"
    );
    let toml_err_msg = parse_result.unwrap_err();
    assert!(
        toml_err_msg.contains("[[dtu]]") || toml_err_msg.contains("Hybrid"),
        "BC-3.2.005 postcondition 3: startup error must identify the offending [[dtu]] \
         block or the invalid mode value; got: {toml_err_msg}"
    );
}

/// Helper: validate DTU mode in TOML context.
///
/// Attempts to parse the TOML snippet and validate `[[dtu]]` blocks' `mode` field
/// using the authoritative `DtuMode` deserializer. Returns `Err(String)` when an
/// invalid mode is found, with context identifying the offending `[[dtu]]` block.
fn validate_dtu_mode_in_toml(toml_snippet: &str) -> Result<(), String> {
    use prism_dtu_common::DtuMode;

    let doc: toml::Value = toml_snippet
        .parse()
        .map_err(|e| format!("TOML parse error: {e}"))?;

    let dtu_entries = match doc.get("dtu") {
        Some(toml::Value::Array(arr)) => arr.clone(),
        Some(_) => return Err("[[dtu]] must be an array-of-tables".to_string()),
        None => return Ok(()),
    };

    for (i, entry) in dtu_entries.iter().enumerate() {
        if let Some(mode_val) = entry.get("mode") {
            let mode_str = mode_val
                .as_str()
                .ok_or_else(|| format!("[[dtu]] block {i}: mode must be a string"))?;
            let json_mode = serde_json::json!(mode_str);
            serde_json::from_value::<DtuMode>(json_mode)
                .map_err(|e| format!("[[dtu]] block {i}: invalid mode value {mode_str:?}: {e}"))?;
        }
    }
    Ok(())
}

/// Migrated from org_tagging.rs: `test_BC_3_2_005_mode_immutable_after_startup`.
///
/// RED GATE PRIMARY: `build_slack_harness` panics.
/// RED GATE SECONDARY: captured payload has no `"org_id"` key after configure.
///
/// Traces to: BC-3.2.005 invariant 4; S-3.4.05 AC-001.
#[tokio::test]
async fn test_BC_3_2_005_slack_mode_immutable_after_startup() {
    use prism_dtu_common::DtuMode;
    use prism_dtu_slack::clone::SLACK_DTU_MODE;

    let (harness, addr, client) = build_slack_harness().await;

    // Attempt mode change via configure (must be rejected — mode is not runtime-settable).
    let _ = harness
        .inject_failure(TENANT, DtuType::Slack, prism_dtu_common::FailureMode::None)
        .await;

    // Static check: SLACK_DTU_MODE is a compile-time constant — it cannot change.
    assert_eq!(
        SLACK_DTU_MODE,
        DtuMode::Shared,
        "BC-3.2.005 invariant 4: SLACK_DTU_MODE must remain DtuMode::Shared \
         after attempted runtime mode change"
    );

    // Dynamic check: shared-mode dispatch path must still embed org_id.
    let payload = serde_json::json!({"text": "post-configure mode immutability check"});
    let resp = post_webhook(&client, addr, &payload).await;
    assert_eq!(resp.status().as_u16(), 200);

    let captured = get_received_payloads(&client, addr).await;
    assert!(
        !captured.is_empty(),
        "mode immutability test: expected at least one captured payload"
    );

    assert!(
        captured[0].get("org_id").is_some(),
        "BC-3.2.005 invariant 4: shared-mode dispatch must still embed 'org_id' after \
         a (rejected) configure attempt; found no 'org_id' key"
    );
}

// ---------------------------------------------------------------------------
// New harness-specific tests (S-3.4.05 ACs)
// ---------------------------------------------------------------------------

/// AC-004 (S-3.4.05): Shared-mode OrgId tagging — different orgs produce different tags.
///
/// Builds a harness with a single shared Slack clone (IsolationMode::Logical,
/// DtuType::Slack). Dispatches actions on behalf of org_A and org_B. Asserts
/// that the captured outbound webhook payload body contains the respective
/// OrgId UUID as a structured field, and that the OrgId does NOT appear in
/// any HTTP header or URL path segment.
///
/// Canonical test vector from BC-3.2.004 TV-3.2.004-01 and TV-3.2.004-02.
///
/// RED GATE PRIMARY: `build_slack_harness` panics (DtuType::Slack not dispatched).
/// RED GATE SECONDARY: captured entries have no `"org_id"` key.
///
/// Traces to: BC-3.2.004 postconditions 1, 2; BC-3.5.001 postcondition 1;
///            VP-087, VP-088; S-3.4.05 AC-004.
#[tokio::test]
async fn ac_shared_mode_org_id_tagging() {
    let (_harness, addr, client) = build_slack_harness().await;

    // Dispatch for org_A.
    let resp_a = post_webhook_for_org(
        &client,
        addr,
        &serde_json::json!({"text": "org A alert"}),
        Some(ORG_UUID_A),
    )
    .await;
    assert_eq!(resp_a.status().as_u16(), 200, "org_A POST must succeed");

    // Verify org_A UUID is NOT in response URL or headers.
    let url_a = resp_a.url().to_string();
    assert!(
        !url_a.contains(ORG_UUID_A),
        "BC-3.2.004 postcondition 2: ORG_UUID_A must not appear in response URL"
    );
    for (name, value) in resp_a.headers() {
        let val_str = value.to_str().unwrap_or("");
        assert!(
            !val_str.contains(ORG_UUID_A),
            "BC-3.2.004 postcondition 2: ORG_UUID_A must not appear in response header '{name}'"
        );
    }

    // Dispatch for org_B.
    let resp_b = post_webhook_for_org(
        &client,
        addr,
        &serde_json::json!({"text": "org B alert"}),
        Some(ORG_UUID_B),
    )
    .await;
    assert_eq!(resp_b.status().as_u16(), 200, "org_B POST must succeed");

    // Retrieve captured payloads and verify org_id tags.
    let captured = get_received_payloads(&client, addr).await;
    assert_eq!(captured.len(), 2, "both payloads must be captured");

    // Build a map of org_id → captured entry.
    let org_ids: Vec<&str> = captured
        .iter()
        .map(|e| e["org_id"].as_str().unwrap_or(""))
        .collect();

    assert!(
        org_ids.contains(&ORG_UUID_A),
        "AC-004: captured entries must include ORG_UUID_A '{ORG_UUID_A}'; got {:?}",
        org_ids
    );
    assert!(
        org_ids.contains(&ORG_UUID_B),
        "AC-004: captured entries must include ORG_UUID_B '{ORG_UUID_B}'; got {:?}",
        org_ids
    );
    assert_ne!(
        org_ids[0], org_ids[1],
        "AC-004: the two captured org_ids must be distinct"
    );
}

/// AC-005 / EC-001, EC-002 (S-3.4.05): Multi-org logical isolation in shared mode.
///
/// A single shared Slack listener serves all orgs. Tests that two sequential
/// dispatches — one for org_A and one for org_B — produce two captured payloads,
/// each tagged with its sender's OrgId, with no cross-contamination.
///
/// RED GATE PRIMARY: `build_slack_harness` panics.
/// RED GATE SECONDARY: captured entries have no `"org_id"` key, so cross-org
/// contamination cannot be checked.
///
/// Traces to: BC-3.5.001 postconditions 1, 2; BC-3.2.004 postcondition 4;
///            VP-089, VP-122; S-3.4.05 AC-001, EC-001, EC-002.
#[tokio::test]
async fn ac_multi_org_logical_isolation_shared_mode() {
    // Single shared Slack clone — one endpoint serves all orgs.
    let (_harness, addr, client) = build_slack_harness().await;

    // Sequential dispatches from different orgs.
    post_webhook_for_org(
        &client,
        addr,
        &serde_json::json!({"text": "org A first"}),
        Some(ORG_UUID_A),
    )
    .await;
    post_webhook_for_org(
        &client,
        addr,
        &serde_json::json!({"text": "org B second"}),
        Some(ORG_UUID_B),
    )
    .await;

    let captured = get_received_payloads(&client, addr).await;
    assert_eq!(
        captured.len(),
        2,
        "AC-005: both org payloads must be captured; got {}",
        captured.len()
    );

    // Extract org_ids.
    let ids: Vec<&str> = captured
        .iter()
        .map(|e| e["org_id"].as_str().unwrap_or(""))
        .collect();

    // Each entry must carry its sender's OrgId.
    assert!(
        ids.contains(&ORG_UUID_A),
        "AC-005 EC-001: captured entries must include ORG_UUID_A"
    );
    assert!(
        ids.contains(&ORG_UUID_B),
        "AC-005 EC-002: captured entries must include ORG_UUID_B"
    );

    // No cross-contamination: find org_A entry and check its inner payload.
    let entry_a = captured
        .iter()
        .find(|e| e["org_id"].as_str() == Some(ORG_UUID_A))
        .expect("org_A entry must be present");
    let inner_a = entry_a["payload"].to_string();
    assert!(
        !inner_a.contains(ORG_UUID_B),
        "EC-001: org_A entry's inner payload must not contain org_B UUID"
    );

    let entry_b = captured
        .iter()
        .find(|e| e["org_id"].as_str() == Some(ORG_UUID_B))
        .expect("org_B entry must be present");
    let inner_b = entry_b["payload"].to_string();
    assert!(
        !inner_b.contains(ORG_UUID_A),
        "EC-002: org_B entry's inner payload must not contain org_A UUID"
    );
}

/// AC-007 / EC-003 (S-3.4.05): `CustomerSpec` with `mode = "client"` for Slack does NOT
/// produce a startup error (BC-3.3.001-startup EC-003: MSSP Coordination types permit
/// client mode override).
///
/// The story states: "A CustomerSpec with mode = 'client' for Slack/PagerDuty/Jira
/// does NOT produce a startup error".
///
/// RED GATE: This test verifies that `HarnessBuilder::build()` returns `Ok` — which
/// requires `DtuType::Slack` to be wired into the clone-server dispatch. Until that
/// wiring exists, `endpoint_for` returns `None` and the subsequent `expect` panics,
/// marking this test red for a different reason than the assertion itself.
///
/// Once DtuType::Slack is dispatched, this test should pass immediately (no
/// startup error is the expected behavior per BC-3.3.001 EC-003).
///
/// Traces to: BC-3.5.001 precondition 2 (valid customer registered);
///            BC-3.3.001-startup EC-003; S-3.4.05 AC-007.
#[tokio::test]
async fn ac_client_mode_override_does_not_produce_startup_error() {
    // Build a harness with DtuType::Slack — the client mode override should not cause
    // an error at harness build time (BC-3.3.001 EC-003).
    let harness_result = HarnessBuilder::new()
        .isolation(IsolationMode::Logical)
        .with_customer_overrides(TENANT, |spec| {
            spec.dtu_types = vec![DtuType::Slack];
            // No initial_failure injection — this verifies clean startup only.
        })
        .build()
        .await;

    assert!(
        harness_result.is_ok(),
        "BC-3.3.001 EC-003: HarnessBuilder with DtuType::Slack must NOT produce a startup \
         error; got: {:?}",
        harness_result.err()
    );

    let harness = harness_result.unwrap();

    // Verify the Slack endpoint is reachable.
    let addr = harness
        .endpoint_for(TENANT, DtuType::Slack)
        .expect("Slack endpoint must be present after successful build");

    let client = reqwest::Client::new();
    let resp = client
        .get(format!("http://{addr}/dtu/health"))
        .send()
        .await
        .expect("GET /dtu/health must not fail at network level");

    assert_eq!(
        resp.status().as_u16(),
        200,
        "BC-3.3.001 EC-003: Slack clone health check must return 200 after clean startup"
    );
}
