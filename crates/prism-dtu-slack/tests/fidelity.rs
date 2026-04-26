#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Fidelity test: verifies the Slack Incoming Webhook DTU can start and respond to
//! a valid Block Kit payload with HTTP 200 `{"ok": true, "message_ts": "..."}`.
//!
//! Per story Task 9: starts `SlackClone`, POSTs the valid Block Kit fixture,
//! asserts HTTP 200 with `ok: true` and `message_ts` present (AC-1).

#![cfg(feature = "dtu")]

use prism_dtu_common::BehavioralClone;
use prism_dtu_slack::SlackClone;

#[tokio::test]
async fn slack_dtu_fidelity() {
    let mut clone = SlackClone::new().expect("SlackClone::new failed");
    clone.start().await.expect("SlackClone::start failed");
    let base_url = clone.base_url();

    // Load the valid Block Kit fixture embedded at compile time.
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("../fixtures/valid-block-kit.json"))
            .expect("valid-block-kit.json is valid JSON");

    let client = reqwest::Client::new();

    // AC-1: POST valid Block Kit payload → HTTP 200 with ok=true and message_ts.
    let resp = client
        .post(format!(
            "{base_url}/services/T00000000/B00000000/XXXXXXXXXXXX"
        ))
        .json(&fixture)
        .send()
        .await
        .expect("POST /services/token failed");

    assert_eq!(
        resp.status().as_u16(),
        200,
        "expected HTTP 200 for valid Block Kit payload"
    );

    let body: serde_json::Value = resp.json().await.expect("response body is JSON");
    assert_eq!(body["ok"], true, "expected ok=true in response");
    assert!(
        body["message_ts"].is_string(),
        "expected message_ts string in response, got: {body:?}"
    );
    assert_eq!(
        body["message_ts"].as_str().unwrap(),
        "1234567890.123456",
        "expected stable fake message_ts value"
    );

    clone.stop().await.expect("SlackClone::stop failed");
}
