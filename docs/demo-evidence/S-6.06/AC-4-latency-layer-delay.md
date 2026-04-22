# AC-4: LatencyLayer delays response by at least 80ms when configured 100ms

## Acceptance Criterion

Given `LatencyLayer` is configured with `latency_ms: 100`, When a request
is processed, Then the response is delayed by at least 80ms (allowing for timer
resolution) before being returned.

## Test

- File: `crates/prism-dtu-common/tests/ac_4_latency_layer_delay.rs`
- Function: `ac_4_latency_layer_delays_response_by_configured_ms`
- Test command: `cargo test --features prism-dtu-common/dtu --test ac_4_latency_layer_delay`

## Implementation (excerpt)

File: `crates/prism-dtu-common/src/layers/latency.rs`

```rust
fn call(&mut self, req: Req) -> Self::Future {
    let latency_ms = self.latency_ms;
    let fut = self.inner.call(req);
    Box::pin(async move {
        if latency_ms > 0 {
            tokio::time::sleep(std::time::Duration::from_millis(latency_ms)).await;
        }
        fut.await
    })
}
```

## Test output

```
running 1 test
test ac_4_latency_layer_delays_response_by_configured_ms ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s
```

## Mapping

`LatencyMiddleware::call` sleeps for `latency_ms` milliseconds using `tokio::time::sleep` before forwarding to the inner service; the test measures wall-clock elapsed time with `std::time::Instant` and asserts `>= 80ms`, providing 20ms of timer-resolution tolerance for a configured 100ms delay.
