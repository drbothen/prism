# AC-7: WebhookReceiver captures POST path and body

## Acceptance Criterion

Given `WebhookReceiver` is started, When a POST request with JSON body
is sent to its bound address, Then `received_payloads()` returns a `CapturedRequest`
with the correct path and body bytes.

## Test

- File: `crates/prism-dtu-common/tests/ac_7_webhook_receiver.rs`
- Function: `ac_7_webhook_receiver_captures_post_body_and_path`
- Test command: `cargo test --features prism-dtu-common/dtu --test ac_7_webhook_receiver`

## Implementation (excerpt)

File: `crates/prism-dtu-common/src/webhook.rs`

```rust
async fn capture_handler(
    State(captured): State<Arc<Mutex<Vec<CapturedRequest>>>>,
    req: Request<Body>,
) -> StatusCode {
    let path = req.uri().path().to_owned();
    let headers = req.headers().clone();
    let body_bytes = axum::body::to_bytes(req.into_body(), usize::MAX)
        .await
        .unwrap_or_default();

    captured.lock().expect("captured lock poisoned").push(CapturedRequest {
        path,
        body: body_bytes,
        headers,
    });

    StatusCode::OK
}
```

## Test output

```
running 1 test
test ac_7_webhook_receiver_captures_post_body_and_path ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
```

## Mapping

`WebhookReceiver` starts an axum server on an OS-assigned port with a fallback `any` route that reads the request path, headers, and body bytes into a `CapturedRequest` stored in a shared `Arc<Mutex<Vec<...>>>`; the test POSTs `{"event":"login","user":"alice"}` to `/events/ingest` and asserts the captured path and body match.
