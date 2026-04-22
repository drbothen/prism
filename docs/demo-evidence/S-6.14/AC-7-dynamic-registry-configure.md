# AC-7: Dynamic Registry Addition via POST /dtu/configure

## AC Statement

Given `POST /dtu/configure {"ip": "10.0.0.1", "fixture": "malicious"}`, Then
`GET /v3/ip/10.0.0.1` returns the malicious fixture response (dynamic registry addition).

## Test File and Function

- File: `crates/prism-dtu-threatintel/tests/ac_7_dynamic_registry_configure.rs`
- Function: `ac_7_dynamic_registry_addition_serves_malicious_fixture`

## Implementation Excerpt

From `crates/prism-dtu-threatintel/src/routes/lookup.rs`:

```rust
pub async fn configure(
    State(state): State<Arc<ThreatIntelState>>,
    Json(body): Json<Value>,
) -> impl IntoResponse {
    let lookup_value = body
        .get("ip")
        .or_else(|| body.get("hash"))
        .or_else(|| body.get("domain"))
        .and_then(|v| v.as_str());

    if let Some(fixture_str) = body.get("fixture").and_then(|v| v.as_str()) {
        let fixture_key = match fixture_str {
            "malicious" => FixtureKey::Malicious,
            "benign"    => FixtureKey::Benign,
            "unknown"   => FixtureKey::Unknown,
            _ => return (StatusCode::BAD_REQUEST,
                         Json(json!({"error": "unknown fixture key"}))).into_response(),
        };
        if let Some(value) = lookup_value {
            let mut registry = state.fixture_registry.lock().expect("poisoned");
            registry.insert(value.to_string(), fixture_key);
            return (StatusCode::OK, Json(json!({"status": "ok"}))).into_response();
        }
    }
    (StatusCode::OK, Json(json!({"status": "ok"}))).into_response()
}
```

The test also verifies: (a) pre-condition score is 0; (b) post-configure score is 85; (c) after `reset()`, score returns to 0 (custom entry purged); (d) invalid fixture name returns HTTP 400.

## Test Output

```
running 1 test
test ac_7_dynamic_registry_addition_serves_malicious_fixture ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
```

## AC Mapping

`POST /dtu/configure` inserts the IP into the Mutex-guarded `fixture_registry`; subsequent lookups dispatch through `ip_fixture_response(FixtureKey::Malicious, ...)` returning `threat_score: 85`. `reset()` restores the default registry removing the custom entry.
