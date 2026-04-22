# AC-5: Missing API Key Returns HTTP 401

## AC Statement

Given both `key` query param and `Authorization` header absent, Then the
response is HTTP 401 `{"error": "missing API key", "code": 401}` — maps to
`E-INFUSION-AUTH-001`.

## Test File and Function

- File: `crates/prism-dtu-threatintel/tests/ac_5_auth_reject_missing_key.rs`
- Function: `ac_5_missing_api_key_returns_401_with_error_body`

## Implementation Excerpt

From `crates/prism-dtu-threatintel/src/routes/lookup.rs`:

```rust
fn check_auth(params: &LookupParams, headers: &HeaderMap) -> Result<(), (StatusCode, Json<Value>)> {
    let has_query_key = params
        .key
        .as_deref()
        .map(|k| !k.is_empty())
        .unwrap_or(false);

    // Bearer token: require non-empty token after the "Bearer " prefix (7 chars).
    let has_bearer = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .map(|v| v.starts_with("Bearer ") && !v[7..].trim().is_empty())
        .unwrap_or(false);

    if has_query_key || has_bearer {
        Ok(())
    } else {
        Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "missing API key", "code": 401})),
        ))
    }
}
```

## Test Output

```
running 1 test
test ac_5_missing_api_key_returns_401_with_error_body ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
```

## AC Mapping

`check_auth` rejects requests with no query key and no Bearer header (including `"Bearer "` with only whitespace after the prefix) with HTTP 401 and the exact body `{"error": "missing API key", "code": 401}` required by `E-INFUSION-AUTH-001`.
