# AC-6: auth_mode=reject — Any apiKey Request Returns HTTP 403

## AC Statement

Given `auth_mode=reject` configured via `POST /dtu/configure`, when any request with
`apiKey` arrives, then the response is HTTP 403
`{"error": "Forbidden. apiKey not verified."}` — maps to `E-INFUSION-AUTH-002`.

## Test File

`crates/prism-dtu-nvd/tests/ac_6_auth_mode_reject.rs`

## Test Functions

- `ac_6_auth_mode_reject_returns_403_for_any_api_key` (AC-6a: rejection path)
- `ac_6_auth_mode_reject_does_not_affect_unauthenticated_requests` (AC-6b: unauthenticated unaffected)

## Implementation Excerpt

`crates/prism-dtu-nvd/src/state.rs` — auth_mode guard at top of `check_rate_limit`:

```rust
pub fn check_rate_limit(&self, api_key: Option<&str>) -> Result<(), RateLimitError> {
    // Check auth_mode first — if reject and api_key is Some, reject immediately.
    {
        let mode = self.auth_mode.lock().expect("auth_mode poisoned");
        if *mode == AuthMode::Reject && api_key.is_some() {
            return Err(RateLimitError::ApiKeyRejected);
        }
    }
    // ... rate bucket logic follows
}
```

`apply_config` toggles `auth_mode` based on `"auth_mode": "reject"` in the configure payload.

## Test Run Output

```
Running tests/ac_6_auth_mode_reject.rs

running 2 tests
test ac_6_auth_mode_reject_returns_403_for_any_api_key ... ok
test ac_6_auth_mode_reject_does_not_affect_unauthenticated_requests ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
```

## Mapping

AC-6 (both sub-tests) is satisfied: `auth_mode=reject` causes HTTP 403 with
`"Forbidden. apiKey not verified."` for any request bearing `apiKey`; unauthenticated
requests pass through unaffected, verifying the two buckets remain independent.
