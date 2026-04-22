# AC-2: FailureLayer — RateLimit returns HTTP 429 + Retry-After

## Acceptance Criterion

Given `FailureLayer` is configured with `FailureMode::RateLimit { after_n_requests: 5, retry_after_secs: 60 }`,
When the 6th HTTP request arrives, Then the layer returns HTTP 429 with header
`Retry-After: 60` without forwarding to the route handler.

## Test

- File: `crates/prism-dtu-common/tests/ac_2_failure_layer_rate_limit.rs`
- Function: `ac_2_failure_layer_rate_limit_returns_429_after_threshold`
- Test command: `cargo test --features prism-dtu-common/dtu --test ac_2_failure_layer_rate_limit`

## Implementation (excerpt)

File: `crates/prism-dtu-common/src/layers/failure.rs`

```rust
FailureMode::RateLimit {
    after_n_requests,
    retry_after_secs,
} => {
    if count > after_n_requests {
        Ok(Response::builder()
            .status(429)
            .header("Retry-After", retry_after_secs.to_string())
            .body(Body::empty())
            .expect("build 429 response"))
    } else {
        fut.await
    }
}
```

## Test output

```
running 1 test
test ac_2_failure_layer_rate_limit_returns_429_after_threshold ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

## Mapping

`FailureMiddleware` uses an `AtomicU32` request counter; on each call it increments the counter before branching, so once `count > after_n_requests` the layer short-circuits with 429 and the configured `Retry-After` value, never reaching the route handler — exactly what the AC requires.
