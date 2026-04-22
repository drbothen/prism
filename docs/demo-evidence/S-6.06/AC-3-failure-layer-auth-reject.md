# AC-3: FailureLayer — AuthReject returns HTTP 401 unconditionally

## Acceptance Criterion

Given `FailureLayer` is configured with `FailureMode::AuthReject`,
When any HTTP request arrives, Then the layer returns HTTP 401 regardless of the
`Authorization` header value.

## Test

- File: `crates/prism-dtu-common/tests/ac_3_failure_layer_auth_reject.rs`
- Function: `ac_3_failure_layer_auth_reject_returns_401_unconditionally`
- Test command: `cargo test --features prism-dtu-common/dtu --test ac_3_failure_layer_auth_reject`

## Implementation (excerpt)

File: `crates/prism-dtu-common/src/layers/failure.rs`

```rust
match mode {
    FailureMode::AuthReject => Ok(Response::builder()
        .status(401)
        .body(Body::empty())
        .expect("build 401 response")),
    // ... other modes
}
```

## Test output

```
running 1 test
test ac_3_failure_layer_auth_reject_returns_401_unconditionally ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

## Mapping

`AuthReject` is the first arm of the mode match and returns a 401 response immediately without inspecting headers or calling the inner service; the test verifies this for both an unauthenticated request and one bearing `Authorization: Bearer valid-token-123`.
