# AC-5: Authenticated Rate Limit — 51st Request Returns HTTP 429

## AC Statement

Given `apiKey: "valid-key"` and 50 prior authenticated requests in a 30-second window,
when the 51st request arrives, then the response is HTTP 429 — maps to `E-INFUSION-RATE-001`.

## Test File

`crates/prism-dtu-nvd/tests/ac_5_authenticated_rate_limit.rs`

## Test Function

`ac_5_authenticated_rate_limit_429_after_50_requests`

## Implementation Excerpt

`crates/prism-dtu-nvd/src/state.rs` — authenticated bucket check and `apply_config` exhaust:

```rust
if let Some(key) = api_key {
    let bucket_key = Some(key.to_owned());
    let bucket = buckets
        .entry(bucket_key)
        .or_insert_with(RateLimitBucket::authenticated);

    if bucket.window_start.elapsed().as_secs() >= 30 {
        bucket.count = 0;
        bucket.window_start = std::time::Instant::now();
    }

    if bucket.count >= bucket.limit {
        return Err(RateLimitError::AuthenticatedExceeded);
    }
    bucket.count += 1;
}
```

`RateLimitBucket::authenticated()` sets `limit: 50`. The test pre-exhausts via
`POST /dtu/configure {"exhaust_authenticated_bucket": true}` which sets `bucket.count = bucket.limit`.

## Test Run Output

```
Running tests/ac_5_authenticated_rate_limit.rs

running 1 test
test ac_5_authenticated_rate_limit_429_after_50_requests ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
```

## Mapping

AC-5 is satisfied: the authenticated bucket enforces 50 requests/30s; `POST /dtu/configure`
allows tests to pre-exhaust the bucket without firing 50 real requests; the 51st request
returns HTTP 429, enabling Prism's infusion to handle `E-INFUSION-RATE-001`.
