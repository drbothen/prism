# AC-4: Unauthenticated Rate Limit — 6th Request Returns HTTP 403

## AC Statement

Given no `apiKey` parameter and 5 prior unauthenticated requests in a 30-second window,
when the 6th request arrives, then the response is HTTP 403
`{"error": "Forbidden. Too many requests. (Unauthenticated rate limit exceeded.)"}`.

## Test File

`crates/prism-dtu-nvd/tests/ac_4_unauthenticated_rate_limit.rs`

## Test Function

`ac_4_unauthenticated_rate_limit_403_on_sixth_request`

## Implementation Excerpt

`crates/prism-dtu-nvd/src/state.rs` — unauthenticated bucket check in `check_rate_limit`:

```rust
} else {
    // Unauthenticated bucket.
    let bucket = buckets
        .entry(None)
        .or_insert_with(RateLimitBucket::unauthenticated);

    if bucket.window_start.elapsed().as_secs() >= 30 {
        bucket.count = 0;
        bucket.window_start = std::time::Instant::now();
    }

    if bucket.count >= bucket.limit {
        return Err(RateLimitError::UnauthenticatedExceeded);
    }
    bucket.count += 1;
}
```

`RateLimitBucket::unauthenticated()` sets `limit: 5`.

## Test Run Output

```
Running tests/ac_4_unauthenticated_rate_limit.rs

running 1 test
test ac_4_unauthenticated_rate_limit_403_on_sixth_request ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
```

## Mapping

AC-4 is satisfied: the unauthenticated bucket enforces a 5-request/30s limit; the 6th
request triggers HTTP 403 with the canonical error string, mirroring NVD's published
unauthenticated rate limit.
