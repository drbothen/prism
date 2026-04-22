# AC-1: Malicious IP Lookup Returns Correct Threat Shape

## AC Statement

Given `GET /v3/ip/45.55.100.1` with a valid API key, Then the response contains
`threat_score: 85`, `threat_is_known_malicious: true`, and `threat_sources` including
`"greynoise"`.

## Test File and Function

- File: `crates/prism-dtu-threatintel/tests/ac_1_malicious_ip_lookup.rs`
- Function: `ac_1_malicious_ip_returns_threat_score_85_and_greynoise_source`

## Implementation Excerpt

From `crates/prism-dtu-threatintel/src/routes/lookup.rs`:

```rust
fn ip_fixture_response(key: &FixtureKey, ip: &str) -> Value {
    match key {
        FixtureKey::Malicious => json!({
            "ip": ip,
            "threat_score": 85,
            "threat_is_known_malicious": true,
            "threat_sources": ["greynoise", "abuseipdb"]
        }),
        FixtureKey::Benign => json!({
            "ip": ip,
            "threat_score": 5,
            "threat_is_known_malicious": false,
            "threat_sources": ["greynoise"]
        }),
        FixtureKey::Unknown => json!({
            "ip": ip,
            "threat_score": 0,
            "threat_is_known_malicious": false,
            "threat_sources": []
        }),
    }
}
```

From `crates/prism-dtu-threatintel/src/state.rs` (default registry):

```rust
m.insert("45.55.100.1".to_string(), FixtureKey::Malicious);
```

## Test Output

```
running 1 test
test ac_1_malicious_ip_returns_threat_score_85_and_greynoise_source ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
```

## AC Mapping

`45.55.100.1` is pre-loaded in the default fixture registry as `FixtureKey::Malicious`; the `ip_fixture_response` handler returns `threat_score: 85` and `threat_sources: ["greynoise", "abuseipdb"]` which satisfies the AC exactly.
