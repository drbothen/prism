# AC-2: Benign IP Lookup Returns Non-Malicious Score

## AC Statement

Given `GET /v3/ip/8.8.8.8` with a valid API key, Then the response contains
`threat_is_known_malicious: false` and `threat_score` below 20.

## Test File and Function

- File: `crates/prism-dtu-threatintel/tests/ac_2_benign_ip_lookup.rs`
- Function: `ac_2_benign_ip_returns_not_malicious_with_score_below_20`

## Implementation Excerpt

From `crates/prism-dtu-threatintel/src/state.rs` (default registry):

```rust
m.insert("8.8.8.8".to_string(), FixtureKey::Benign);
```

From `crates/prism-dtu-threatintel/src/routes/lookup.rs`:

```rust
FixtureKey::Benign => json!({
    "ip": ip,
    "threat_score": 5,
    "threat_is_known_malicious": false,
    "threat_sources": ["greynoise"]
}),
```

## Test Output

```
running 1 test
test ac_2_benign_ip_returns_not_malicious_with_score_below_20 ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
```

## AC Mapping

`8.8.8.8` maps to `FixtureKey::Benign` in the default registry; the handler returns `threat_score: 5` (< 20) and `threat_is_known_malicious: false`, satisfying the AC.
