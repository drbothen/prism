# AC-4: Malicious Hash Lookup Returns VirusTotal Shape

## AC Statement

Given `GET /v3/hash/{sha256}` for a pre-registered malicious hash, Then the
response includes `threat_sources` containing `"virustotal"` and `threat_score` above 80.

## Test File and Function

- File: `crates/prism-dtu-threatintel/tests/ac_4_malicious_hash_lookup.rs`
- Function: `ac_4_pre_registered_malicious_hash_returns_virustotal_source_and_score_above_80`

## Implementation Excerpt

From `crates/prism-dtu-threatintel/src/routes/lookup.rs`:

```rust
pub async fn hash_lookup(/* ... */) -> impl IntoResponse {
    // auth + rate-limit checks elided
    let body = match state.lookup_fixture(&hash) {
        Some(FixtureKey::Malicious) => json!({
            "hash": hash,
            "threat_score": 95,
            "threat_is_known_malicious": true,
            "threat_sources": ["virustotal"]
        }),
        Some(FixtureKey::Benign) => json!({
            "hash": hash,
            "threat_score": 2,
            "threat_is_known_malicious": false,
            "threat_sources": ["virustotal"]
        }),
        _ => json!({
            "hash": hash,
            "threat_score": 0,
            "threat_is_known_malicious": false,
            "threat_sources": []
        }),
    };
    (StatusCode::OK, Json(body)).into_response()
}
```

The test pre-registers the hash via `POST /dtu/configure {"hash": "<sha256>", "fixture": "malicious"}`.

## Test Output

```
running 1 test
test ac_4_pre_registered_malicious_hash_returns_virustotal_source_and_score_above_80 ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
```

## AC Mapping

The `configure` endpoint accepts a `"hash"` key alongside `"fixture"`, inserting the mapping into the fixture registry. On subsequent `GET /v3/hash/{hash}`, the `Malicious` branch returns `threat_score: 95` (> 80) with `threat_sources: ["virustotal"]`.
