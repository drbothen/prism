// AC-7: WebhookReceiver captures POST body + path.
//
// Test starts a WebhookReceiver, sends a POST with JSON body to a specific path,
// then asserts received_payloads() contains a CapturedRequest with matching path and body.
//
// Expected failure mode: WebhookReceiver::start is todo!() — panics at runtime.

use prism_dtu_common::WebhookReceiver;

#[tokio::test]
async fn ac_7_webhook_receiver_captures_post_body_and_path() {
    let receiver: WebhookReceiver = WebhookReceiver::start()
        .await
        .expect("AC-7: WebhookReceiver::start must succeed");

    let base_url = format!("http://{}", receiver.bound_addr());
    let target_path = "/events/ingest";
    let payload = serde_json::json!({"event": "login", "user": "alice"});

    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{base_url}{target_path}"))
        .json(&payload)
        .send()
        .await
        .expect("AC-7: POST request must reach the receiver");

    // Allow any 2xx — the receiver must accept the request.
    assert!(
        resp.status().is_success(),
        "AC-7: WebhookReceiver must return 2xx for POST requests; got {}",
        resp.status()
    );

    // Wait briefly for the server task to store the request.
    tokio::time::sleep(std::time::Duration::from_millis(20)).await;

    let payloads = receiver.received_payloads();
    assert!(
        !payloads.is_empty(),
        "AC-7: received_payloads() must be non-empty after POST"
    );

    let captured = &payloads[0];
    assert_eq!(
        captured.path, target_path,
        "AC-7: captured path must match the request path"
    );

    let body_str = std::str::from_utf8(&captured.body).expect("AC-7: body must be valid UTF-8");
    assert!(
        body_str.contains("alice"),
        "AC-7: captured body must contain the posted JSON payload"
    );
}
