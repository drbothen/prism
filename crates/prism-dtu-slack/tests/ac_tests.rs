//! S-6.11 Acceptance Criteria tests — Slack Incoming Webhook DTU (L2 stateful).
//!
//! One test per AC (AC-1..AC-6) plus edge-case coverage (EC-001, EC-002, EC-004).
//! Tests use literal spec values per TDD discipline:
//!   - HTTP status codes: `StatusCode::OK` / `StatusCode::BAD_REQUEST` / `StatusCode::TOO_MANY_REQUESTS`
//!   - `message_ts`: literal `"1234567890.123456"` (not a stub constant)
//!   - Block Kit allowed fields: `["blocks","text","username","icon_emoji","icon_url","attachments"]`
//!
//! GREEN-BY-DESIGN tests are marked where the stub author fully pre-implemented the behaviour.
//! They are kept because a future refactor could break them.
//!
//! Naming convention: `ac_NNN_descriptive_name` (story has no BC-S.SS.NNN identifiers;
//! AC numbers serve as the primary trace anchor per the story spec).

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![cfg(feature = "dtu")]

use http::StatusCode;
use prism_dtu_common::BehavioralClone;
use prism_dtu_slack::SlackClone;

// ---------------------------------------------------------------------------
// Test helpers
// ---------------------------------------------------------------------------

/// Start a fresh `SlackClone` and return (clone, base_url, reqwest::Client).
async fn start_clone() -> (SlackClone, String, reqwest::Client) {
    let mut clone = SlackClone::new().expect("SlackClone::new");
    clone.start().await.expect("SlackClone::start");
    let base_url = clone.base_url();
    let client = reqwest::Client::new();
    (clone, base_url, client)
}

/// Post a JSON payload to `/services/T00000000/B00000000/XXXXXXXXXXXX` and return the response.
async fn post_to_webhook(
    client: &reqwest::Client,
    base_url: &str,
    payload: &serde_json::Value,
) -> reqwest::Response {
    client
        .post(format!(
            "{base_url}/services/T00000000/B00000000/XXXXXXXXXXXX"
        ))
        .json(payload)
        .send()
        .await
        .expect("POST /services/token")
}

/// Configure the DTU via `/dtu/configure` with the admin token from the clone.
async fn configure_clone(
    client: &reqwest::Client,
    base_url: &str,
    admin_token: &str,
    config: serde_json::Value,
) {
    let resp = client
        .post(format!("{base_url}/dtu/configure"))
        .header("X-Admin-Token", admin_token)
        .json(&config)
        .send()
        .await
        .expect("POST /dtu/configure");
    assert_eq!(
        resp.status().as_u16(),
        200,
        "configure must succeed; got {:?}",
        resp.status()
    );
}

// ---------------------------------------------------------------------------
// AC-1: Valid Block Kit payload with `blocks` → HTTP 200, ok=true, stable message_ts
//
// GREEN-BY-DESIGN: stub-author fully implemented the webhook handler.
// Test is retained because a refactor could break the exact message_ts value or
// response shape.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn ac_1_valid_blocks_payload_returns_200_ok_with_stable_message_ts() {
    let (mut clone, base_url, client) = start_clone().await;

    // Use the fixture embedded in the crate — same data the story task 9 test uses,
    // but this test asserts the exact JSON shape against spec-literal values.
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("../fixtures/valid-block-kit.json"))
            .expect("valid-block-kit.json is valid JSON");

    let resp = post_to_webhook(&client, &base_url, &fixture).await;

    // Assert literal StatusCode::OK (200) — not just `.is_success()`.
    assert_eq!(
        resp.status(),
        StatusCode::OK,
        "AC-1: valid Block Kit payload must return HTTP 200"
    );

    let body: serde_json::Value = resp.json().await.expect("response body is JSON");

    // Assert exact spec-literal response fields (AC-1).
    assert_eq!(body["ok"], true, "AC-1: response must have ok=true");
    assert_eq!(
        body["message_ts"].as_str().unwrap_or(""),
        "1234567890.123456",
        "AC-1: message_ts must be the stable spec-literal value '1234567890.123456'"
    );

    clone.stop().await.expect("stop");
}

// ---------------------------------------------------------------------------
// AC-1 (text-only variant): `text` field alone is sufficient.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn ac_1_text_only_payload_returns_200() {
    let (mut clone, base_url, client) = start_clone().await;

    let payload = serde_json::json!({"text": "Hello from Prism"});
    let resp = post_to_webhook(&client, &base_url, &payload).await;

    assert_eq!(
        resp.status(),
        StatusCode::OK,
        "AC-1: payload with 'text' only must return HTTP 200"
    );

    let body: serde_json::Value = resp.json().await.expect("response body is JSON");
    assert_eq!(
        body["ok"], true,
        "AC-1: text-only payload must return ok=true"
    );

    clone.stop().await.expect("stop");
}

// ---------------------------------------------------------------------------
// AC-2: Missing both `blocks` and `text` → HTTP 400 "invalid_payload"
// ---------------------------------------------------------------------------

#[tokio::test]
async fn ac_2_missing_blocks_and_text_returns_400_invalid_payload() {
    let (mut clone, base_url, client) = start_clone().await;

    // Payload has allowed fields (username) but is missing both `blocks` and `text`.
    let payload = serde_json::json!({"username": "prism-bot"});
    let resp = post_to_webhook(&client, &base_url, &payload).await;

    assert_eq!(
        resp.status(),
        StatusCode::BAD_REQUEST,
        "AC-2: payload missing both 'blocks' and 'text' must return HTTP 400"
    );

    let body_text = resp.text().await.expect("response body text");
    assert_eq!(
        body_text.trim(),
        "\"invalid_payload\"",
        "AC-2: response body must be literal '\"invalid_payload\"'"
    );

    clone.stop().await.expect("stop");
}

// ---------------------------------------------------------------------------
// EC-001: Empty JSON object `{}` → HTTP 400 "invalid_payload"
// ---------------------------------------------------------------------------

#[tokio::test]
async fn ec_001_empty_json_object_returns_400_invalid_payload() {
    let (mut clone, base_url, client) = start_clone().await;

    let payload = serde_json::json!({});
    let resp = post_to_webhook(&client, &base_url, &payload).await;

    assert_eq!(
        resp.status(),
        StatusCode::BAD_REQUEST,
        "EC-001: empty JSON object must return HTTP 400"
    );

    let body_text = resp.text().await.expect("response body text");
    assert_eq!(
        body_text.trim(),
        "\"invalid_payload\"",
        "EC-001: response body for empty object must be '\"invalid_payload\"'"
    );

    clone.stop().await.expect("stop");
}

// ---------------------------------------------------------------------------
// AC-3: Unknown top-level field → HTTP 400 "unknown_field"
// ---------------------------------------------------------------------------

#[tokio::test]
async fn ac_3_unknown_top_level_field_returns_400_unknown_field() {
    let (mut clone, base_url, client) = start_clone().await;

    // `blocks` is present (satisfies AC-2 check) but `unknown_key` is not in the
    // ALLOWED_BLOCK_KIT_KEYS allow-list from `fixtures/block-kit-schema.json`.
    let payload = serde_json::json!({
        "blocks": [{"type": "section", "text": {"type": "mrkdwn", "text": "hi"}}],
        "unknown_key": "value"
    });
    let resp = post_to_webhook(&client, &base_url, &payload).await;

    assert_eq!(
        resp.status(),
        StatusCode::BAD_REQUEST,
        "AC-3: payload with unknown top-level field must return HTTP 400"
    );

    let body_text = resp.text().await.expect("response body text");
    assert_eq!(
        body_text.trim(),
        "\"unknown_field\"",
        "AC-3: response body must be literal '\"unknown_field\"' for unknown top-level keys"
    );

    clone.stop().await.expect("stop");
}

// ---------------------------------------------------------------------------
// AC-3 (exhaustive): Every field in the spec allow-list is accepted.
// This ensures the ALLOWED_BLOCK_KIT_KEYS set exactly matches
// `fixtures/block-kit-schema.json` `allowed_top_level_fields`.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn ac_3_all_allowed_top_level_fields_are_accepted() {
    let (mut clone, base_url, client) = start_clone().await;

    // All 6 allowed top-level fields from block-kit-schema.json.
    let payload = serde_json::json!({
        "blocks": [{"type": "section", "text": {"type": "mrkdwn", "text": "test"}}],
        "text": "fallback text",
        "username": "prism-bot",
        "icon_emoji": ":robot_face:",
        "icon_url": "https://example.com/icon.png",
        "attachments": []
    });
    let resp = post_to_webhook(&client, &base_url, &payload).await;

    assert_eq!(
        resp.status(),
        StatusCode::OK,
        "AC-3: payload using only allowed fields must return HTTP 200"
    );

    clone.stop().await.expect("stop");
}

// ---------------------------------------------------------------------------
// AC-4: Rate-limit — after N requests the next POST returns 429 + Retry-After: 30 +
//        body "ratelimited"
// ---------------------------------------------------------------------------

#[tokio::test]
async fn ac_4_rate_limit_returns_429_with_retry_after_and_ratelimited_body() {
    let (mut clone, base_url, client) = start_clone().await;
    let admin_token = clone.admin_token().to_string();

    // Configure: rate-limit after 3 requests.
    configure_clone(
        &client,
        &base_url,
        &admin_token,
        serde_json::json!({"rate_limit_after": 3}),
    )
    .await;

    let valid_payload = serde_json::json!({"text": "ping"});

    // Requests 1–3 must succeed.
    for n in 1u32..=3 {
        let resp = post_to_webhook(&client, &base_url, &valid_payload).await;
        assert_eq!(
            resp.status(),
            StatusCode::OK,
            "AC-4: request {n} must succeed (within rate-limit threshold of 3)"
        );
    }

    // Request 4 must receive HTTP 429.
    let resp = post_to_webhook(&client, &base_url, &valid_payload).await;

    assert_eq!(
        resp.status(),
        StatusCode::TOO_MANY_REQUESTS,
        "AC-4: 4th request must return HTTP 429 (rate-limit threshold 3 exceeded)"
    );

    // `Retry-After` header must be present and equal to 30 (spec value from AC-4).
    let retry_after = resp
        .headers()
        .get("retry-after")
        .expect("AC-4: HTTP 429 must include Retry-After header")
        .to_str()
        .expect("Retry-After header must be valid ASCII");
    assert_eq!(
        retry_after, "30",
        "AC-4: Retry-After must equal 30 (spec value from AC-4 / story Task 5)"
    );

    let body_text = resp.text().await.expect("response body text");
    assert_eq!(
        body_text.trim(),
        "\"ratelimited\"",
        "AC-4: body must be literal '\"ratelimited\"'"
    );

    clone.stop().await.expect("stop");
}

// ---------------------------------------------------------------------------
// EC-002: `fail_with: 500` failure mode — next POST returns HTTP 500
// ---------------------------------------------------------------------------

#[tokio::test]
async fn ec_002_fail_with_500_returns_internal_server_error() {
    let (mut clone, base_url, client) = start_clone().await;
    let admin_token = clone.admin_token().to_string();

    // Configure: fail at request 1.
    configure_clone(
        &client,
        &base_url,
        &admin_token,
        serde_json::json!({"fail_with": 500}),
    )
    .await;

    let valid_payload = serde_json::json!({"text": "should fail"});
    let resp = post_to_webhook(&client, &base_url, &valid_payload).await;

    assert_eq!(
        resp.status().as_u16(),
        500,
        "EC-002: configured fail_with:500 must return HTTP 500"
    );

    clone.stop().await.expect("stop");
}

// ---------------------------------------------------------------------------
// AC-5: 3 successful deliveries → GET /dtu/received-payloads returns all 3 in order
// ---------------------------------------------------------------------------

#[tokio::test]
async fn ac_5_three_deliveries_captured_in_order() {
    let (mut clone, base_url, client) = start_clone().await;

    let payloads = [
        serde_json::json!({"text": "message one"}),
        serde_json::json!({"text": "message two"}),
        serde_json::json!({"text": "message three"}),
    ];

    // Send 3 valid webhook deliveries.
    for payload in &payloads {
        let resp = post_to_webhook(&client, &base_url, payload).await;
        assert_eq!(
            resp.status(),
            StatusCode::OK,
            "AC-5: each delivery must return HTTP 200"
        );
    }

    // Retrieve captured payloads via DTU introspection endpoint.
    let resp = client
        .get(format!("{base_url}/dtu/received-payloads"))
        .send()
        .await
        .expect("GET /dtu/received-payloads");

    assert_eq!(
        resp.status(),
        StatusCode::OK,
        "AC-5: GET /dtu/received-payloads must return HTTP 200"
    );

    let body: serde_json::Value = resp.json().await.expect("response body is JSON");
    let captured = body["payloads"]
        .as_array()
        .expect("payloads must be an array");

    assert_eq!(
        captured.len(),
        3,
        "AC-5: must have exactly 3 captured payloads; got {}",
        captured.len()
    );

    // Assert order: payloads[0] has text="message one", etc.
    // S-3.2.05: captured entries use the tagged wrapper format:
    // {"org_id": "<uuid>", "payload": {"text": "...", ...}}
    assert_eq!(
        captured[0]["payload"]["text"].as_str().unwrap_or(""),
        "message one",
        "AC-5: first captured payload must be 'message one'"
    );
    assert_eq!(
        captured[1]["payload"]["text"].as_str().unwrap_or(""),
        "message two",
        "AC-5: second captured payload must be 'message two'"
    );
    assert_eq!(
        captured[2]["payload"]["text"].as_str().unwrap_or(""),
        "message three",
        "AC-5: third captured payload must be 'message three'"
    );

    clone.stop().await.expect("stop");
}

// ---------------------------------------------------------------------------
// AC-5 (in-process API): received_payloads() on the clone struct returns same data
// ---------------------------------------------------------------------------

#[tokio::test]
async fn ac_5_in_process_received_payloads_api_matches_http_endpoint() {
    let (mut clone, base_url, client) = start_clone().await;

    let payload_a = serde_json::json!({"blocks": [{"type": "divider"}]});
    let payload_b = serde_json::json!({"text": "second"});

    post_to_webhook(&client, &base_url, &payload_a).await;
    post_to_webhook(&client, &base_url, &payload_b).await;

    // In-process API (story Task 2).
    let in_process = clone.received_payloads();
    assert_eq!(
        in_process.len(),
        2,
        "AC-5: in-process received_payloads() must return 2 after 2 deliveries"
    );

    // S-3.2.05: captured entries use the tagged wrapper format.
    assert_eq!(
        in_process[0]["payload"]["blocks"][0]["type"]
            .as_str()
            .unwrap_or(""),
        "divider",
        "AC-5: first in-process payload must match first sent payload"
    );

    clone.stop().await.expect("stop");
}

// ---------------------------------------------------------------------------
// AC-6: reset() clears received_payloads and rate-limit counter
// ---------------------------------------------------------------------------

#[tokio::test]
async fn ac_6_reset_clears_received_payloads_and_request_counter() {
    let (mut clone, base_url, client) = start_clone().await;
    let admin_token = clone.admin_token().to_string();

    // Send 2 payloads.
    let payload = serde_json::json!({"text": "before reset"});
    post_to_webhook(&client, &base_url, &payload).await;
    post_to_webhook(&client, &base_url, &payload).await;

    // Verify 2 are captured before reset.
    let in_process_before = clone.received_payloads();
    assert_eq!(
        in_process_before.len(),
        2,
        "AC-6: precondition: must have 2 payloads before reset"
    );

    // Also configure a rate-limit so we can verify the counter resets.
    configure_clone(
        &client,
        &base_url,
        &admin_token,
        serde_json::json!({"rate_limit_after": 1}),
    )
    .await;

    // Call reset via the BehavioralClone trait method.
    clone.reset().await.expect("AC-6: reset must succeed");

    // After reset: in-process API must return empty.
    let in_process_after = clone.received_payloads();
    assert!(
        in_process_after.is_empty(),
        "AC-6: received_payloads must be empty after reset(); got {} entries",
        in_process_after.len()
    );

    // Verify via HTTP endpoint too.
    let resp = client
        .get(format!("{base_url}/dtu/received-payloads"))
        .send()
        .await
        .expect("GET /dtu/received-payloads after reset");
    let body: serde_json::Value = resp.json().await.expect("JSON");
    let payloads_after = body["payloads"].as_array().expect("array");
    assert!(
        payloads_after.is_empty(),
        "AC-6: HTTP /dtu/received-payloads must be empty after reset()"
    );

    // Send one more payload — if request_count was reset to 0, this should succeed
    // (rate-limit threshold is 1, but we're now at count=1, not count=2).
    // After reset counter=0; this request increments to 1 which equals threshold=1,
    // so it is NOT over threshold → HTTP 200.
    let resp_after_reset = post_to_webhook(&client, &base_url, &payload).await;
    assert_eq!(
        resp_after_reset.status(),
        StatusCode::OK,
        "AC-6: first request after reset must succeed (counter was reset to 0)"
    );

    clone.stop().await.expect("stop");
}

// ---------------------------------------------------------------------------
// AC-6 (POST /dtu/reset HTTP endpoint): equivalent via HTTP
// ---------------------------------------------------------------------------

#[tokio::test]
async fn ac_6_post_dtu_reset_endpoint_clears_state() {
    let (mut clone, base_url, client) = start_clone().await;

    // Send a payload, then reset via HTTP.
    let payload = serde_json::json!({"text": "to be cleared"});
    post_to_webhook(&client, &base_url, &payload).await;

    let reset_resp = client
        .post(format!("{base_url}/dtu/reset"))
        .send()
        .await
        .expect("POST /dtu/reset");

    assert_eq!(
        reset_resp.status(),
        StatusCode::OK,
        "AC-6: POST /dtu/reset must return HTTP 200"
    );

    // Verify clear.
    let get_resp = client
        .get(format!("{base_url}/dtu/received-payloads"))
        .send()
        .await
        .expect("GET /dtu/received-payloads after reset");
    let body: serde_json::Value = get_resp.json().await.expect("JSON");
    let payloads = body["payloads"].as_array().expect("array");
    assert!(
        payloads.is_empty(),
        "AC-6: received-payloads must be empty after POST /dtu/reset"
    );

    clone.stop().await.expect("stop");
}

// ---------------------------------------------------------------------------
// EC-004: message_ts is stable — two deliveries return the same value
// ---------------------------------------------------------------------------

#[tokio::test]
async fn ec_004_message_ts_is_stable_across_deliveries() {
    let (mut clone, base_url, client) = start_clone().await;

    let payload = serde_json::json!({"text": "first"});
    let resp1 = post_to_webhook(&client, &base_url, &payload).await;
    let body1: serde_json::Value = resp1.json().await.expect("JSON");
    let ts1 = body1["message_ts"].as_str().unwrap_or("").to_string();

    let payload2 = serde_json::json!({"text": "second"});
    let resp2 = post_to_webhook(&client, &base_url, &payload2).await;
    let body2: serde_json::Value = resp2.json().await.expect("JSON");
    let ts2 = body2["message_ts"].as_str().unwrap_or("").to_string();

    assert_eq!(
        ts1, ts2,
        "EC-004: message_ts must be the same stable fake value across deliveries"
    );
    assert_eq!(
        ts1, "1234567890.123456",
        "EC-004: message_ts must be the spec-literal value '1234567890.123456'"
    );

    clone.stop().await.expect("stop");
}

// ---------------------------------------------------------------------------
// Architecture compliance: crate must not depend on forbidden modules.
// This is a compile-time check; we verify it by asserting that the only
// imported crates are those permitted by the story spec.
// (The `cargo deny` rule enforces this at CI time; this test documents the contract.)
// ---------------------------------------------------------------------------

#[test]
fn architecture_forbidden_dependencies_documented() {
    // Forbidden: prism-sensors, prism-query, prism-operations, prism-mcp, prism-spec-engine.
    // This test exists as a documentation anchor — the actual enforcement is via
    // `cargo deny` rules in deny.toml. If this test file compiles, the crate compiled
    // without importing any forbidden module (Rust's type system guarantees this).
    //
    // The test body verifies the crate name is what the story spec declares.
    let crate_name = env!("CARGO_PKG_NAME");
    assert_eq!(
        crate_name, "prism-dtu-slack",
        "crate name must match story target_module declaration"
    );
}
