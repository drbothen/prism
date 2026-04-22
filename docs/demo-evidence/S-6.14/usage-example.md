# Usage Example — prism-dtu-threatintel

The following snippet shows how an integration test consumer spins up a
`ThreatIntelClone`, issues lookups, and asserts responses. This is the
pattern used by all 7 AC tests.

```rust
use prism_dtu_common::{build_test_client, BehavioralClone};
use prism_dtu_threatintel::ThreatIntelClone;

#[tokio::test]
async fn example_threatintel_dtu_usage() {
    // 1. Instantiate and start the DTU (binds to 127.0.0.1:0).
    let mut clone = ThreatIntelClone::new();
    clone.start().await.expect("DTU must start");

    let base = clone.base_url(); // e.g. "http://127.0.0.1:54321"
    let client = build_test_client();

    // 2. Malicious IP lookup (pre-loaded default).
    let resp = client
        .get(format!("{base}/v3/ip/45.55.100.1"))
        .query(&[("key", "any-non-empty-value")])
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status().as_u16(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["threat_score"], 85);
    assert_eq!(body["threat_is_known_malicious"], true);

    // 3. Dynamically register a new IP → fixture mapping.
    client
        .post(format!("{base}/dtu/configure"))
        .json(&serde_json::json!({"ip": "10.0.0.99", "fixture": "malicious"}))
        .send()
        .await
        .unwrap();

    let resp2 = client
        .get(format!("{base}/v3/ip/10.0.0.99"))
        .query(&[("key", "any-non-empty-value")])
        .send()
        .await
        .unwrap();
    assert_eq!(resp2.json::<serde_json::Value>().await.unwrap()["threat_score"], 85);

    // 4. Reset clears counters and custom entries.
    clone.reset().await.expect("reset must succeed");
    // After reset, 10.0.0.99 reverts to benign defaults (score 0).
}
```

Key points:
- Any non-empty string is accepted as `key` query param or `Authorization: Bearer <token>`.
- `reset()` must be called between test cases for counter and registry isolation.
- Hash lookups use the same pattern with `/v3/hash/{sha256}` and require the hash to be pre-registered via `POST /dtu/configure {"hash": "...", "fixture": "malicious"}`.
