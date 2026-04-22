# AC-6: Rate Limit Enforced After Configured Threshold

## AC Statement

Given `POST /dtu/configure {"rate_limit_after": 3}`, When the 4th lookup
request arrives, Then the response is HTTP 429 with `Retry-After: 30`.

## Test File and Function

- File: `crates/prism-dtu-threatintel/tests/ac_6_rate_limit_429.rs`
- Function: `ac_6_rate_limit_after_3_returns_429_on_4th_request_with_retry_after_30`

## Implementation Excerpt

From `crates/prism-dtu-threatintel/src/routes/lookup.rs`:

```rust
fn check_rate_limit(state: &ThreatIntelState) -> Result<u32, (StatusCode, Json<Value>)> {
    let count = state.increment_counter();
    if state.is_rate_limited(count) {
        Err((
            StatusCode::TOO_MANY_REQUESTS,
            Json(json!({"error": "rate limit exceeded", "code": 429})),
        ))
    } else {
        Ok(count)
    }
}

// In ip_lookup handler, on rate-limit hit:
if let Err(resp) = check_rate_limit(&state) {
    let mut r = resp.into_response();
    r.headers_mut()
        .insert("retry-after", "30".parse().expect("static header value"));
    return r;
}
```

From `crates/prism-dtu-threatintel/src/state.rs`:

```rust
pub fn is_rate_limited(&self, current_count: u32) -> bool {
    let threshold = self.rate_limit_after.lock().expect("rate_limit_after poisoned");
    match *threshold {
        Some(n) => current_count > n,
        None => false,
    }
}
```

The test also verifies that `reset()` clears both the counter and the threshold.

## Test Output

```
running 1 test
test ac_6_rate_limit_after_3_returns_429_on_4th_request_with_retry_after_30 ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
```

## AC Mapping

After `POST /dtu/configure {"rate_limit_after": 3}`, requests 1-3 succeed (count <= 3); request 4 increments counter to 4 (4 > 3), triggering HTTP 429 with `Retry-After: 30`. `reset()` restores counter to 0 and clears the threshold.
