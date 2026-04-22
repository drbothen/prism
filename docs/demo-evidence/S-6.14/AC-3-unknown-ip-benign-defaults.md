# AC-3: Unknown IP Returns Benign Defaults

## AC Statement

Given `GET /v3/ip/1.1.1.1` (not in registry), Then the response contains the
benign defaults: `threat_score: 0`, `threat_is_known_malicious: false`,
`threat_sources: []`.

## Test File and Function

- File: `crates/prism-dtu-threatintel/tests/ac_3_unknown_ip_benign_defaults.rs`
- Function: `ac_3_unknown_ip_returns_benign_defaults`

## Implementation Excerpt

From `crates/prism-dtu-threatintel/src/routes/lookup.rs`:

```rust
/// Build the benign-default response for an unknown IP address.
fn ip_benign_default(ip: &str) -> Value {
    json!({
        "ip": ip,
        "threat_score": 0,
        "threat_is_known_malicious": false,
        "threat_sources": []
    })
}

// In the ip_lookup handler:
let body = state
    .lookup_fixture(&ip)
    .as_ref()
    .map(|k| ip_fixture_response(k, &ip))
    .unwrap_or_else(|| ip_benign_default(&ip));
```

## Test Output

```
running 1 test
test ac_3_unknown_ip_returns_benign_defaults ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
```

## AC Mapping

When `lookup_fixture` returns `None` (IP not in registry), the handler falls back to `ip_benign_default` which returns `threat_score: 0`, `threat_is_known_malicious: false`, and `threat_sources: []` — exactly the AC contract.
