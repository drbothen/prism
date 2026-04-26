#![allow(clippy::unwrap_used, clippy::expect_used)]
// AC-4: GET /v3/hash/{sha256} for a pre-registered malicious hash returns
// threat_sources containing "virustotal" and threat_score above 80.
//
// Was Red Gate at implementation start; hash registry now implemented.
// The test uses POST /dtu/configure to add a hash→malicious mapping. The configure
// endpoint only accepts an "ip" field (not a generic "lookup_value" field), so the
// hash registration will not be persisted to the fixture registry. The subsequent
// GET /v3/hash/{hash} will then return benign defaults (score 0), failing the
// assertion that threat_score > 80.
//
// Additionally, after a correct implementation, the response "hash" field must
// equal the queried hash — the test asserts this field by name.

use prism_dtu_common::{build_test_client, BehavioralClone};
use prism_dtu_threatintel::ThreatIntelClone;

const MALICIOUS_HASH: &str = "da39a3ee5e6b4b0d3255bfef95601890afd80709";
const API_KEY: &str = "test-key-valid";

#[tokio::test]
async fn ac_4_pre_registered_malicious_hash_returns_virustotal_source_and_score_above_80() {
    let mut clone = ThreatIntelClone::new();
    clone.start().await.expect("AC-4: start() must succeed");

    let base = clone.base_url();
    let admin_token = clone.admin_token().to_string();
    let client = build_test_client();

    // Pre-register the hash as malicious via the configure endpoint.
    // The AC requires a "pre-registered" malicious hash; we use runtime configure.
    let configure_resp = client
        .post(format!("{base}/dtu/configure"))
        .header("X-Admin-Token", &admin_token)
        .json(&serde_json::json!({
            "hash": MALICIOUS_HASH,
            "fixture": "malicious"
        }))
        .send()
        .await
        .expect("AC-4: configure request must reach server");

    assert_eq!(
        configure_resp.status().as_u16(),
        200,
        "AC-4: POST /dtu/configure must return 200 for hash registration"
    );

    // Now look up the hash.
    let resp = client
        .get(format!("{base}/v3/hash/{MALICIOUS_HASH}"))
        .query(&[("key", API_KEY)])
        .send()
        .await
        .expect("AC-4: hash lookup request must reach server");

    assert_eq!(
        resp.status().as_u16(),
        200,
        "AC-4: malicious hash lookup must return HTTP 200"
    );

    let body: serde_json::Value = resp
        .json()
        .await
        .expect("AC-4: response must be valid JSON");

    let threat_score = body
        .get("threat_score")
        .and_then(|v| v.as_u64())
        .expect("AC-4: response must contain 'threat_score' field");

    assert!(
        threat_score > 80,
        "AC-4: threat_score must be above 80 for pre-registered malicious hash, got {threat_score}"
    );

    let sources = body
        .get("threat_sources")
        .and_then(|v| v.as_array())
        .expect("AC-4: response must contain 'threat_sources' array");

    let source_strings: Vec<&str> = sources.iter().filter_map(|v| v.as_str()).collect();

    assert!(
        source_strings.contains(&"virustotal"),
        "AC-4: threat_sources must include 'virustotal' for hash lookups, got: {source_strings:?}"
    );

    // The response must identify the queried hash.
    let hash_field = body
        .get("hash")
        .and_then(|v| v.as_str())
        .expect("AC-4: response must contain 'hash' field identifying the queried hash");

    assert_eq!(
        hash_field, MALICIOUS_HASH,
        "AC-4: 'hash' field must match queried hash"
    );
}
