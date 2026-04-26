#![allow(clippy::unwrap_used, clippy::expect_used)]
// AC-3: GET /v3/ip/1.1.1.1 (not in registry) returns benign defaults:
// threat_score: 0, threat_is_known_malicious: false, threat_sources: [].
//
// Was Red Gate at implementation start; benign_default() now implemented.
// threat_score: 0 and threat_is_known_malicious: false and threat_sources: [].
// The test additionally checks that the endpoint key "ip" field must match the
// queried IP — the stub uses "lookup_value" as the field name, not "ip".

use prism_dtu_common::{build_test_client, BehavioralClone};
use prism_dtu_threatintel::ThreatIntelClone;

const UNKNOWN_IP: &str = "1.1.1.1";
const API_KEY: &str = "test-key-valid";

#[tokio::test]
async fn ac_3_unknown_ip_returns_benign_defaults() {
    let mut clone = ThreatIntelClone::new();
    clone.start().await.expect("AC-3: start() must succeed");

    let base = clone.base_url();
    let client = build_test_client();

    let resp = client
        .get(format!("{base}/v3/ip/{UNKNOWN_IP}"))
        .query(&[("key", API_KEY)])
        .send()
        .await
        .expect("AC-3: request must reach server");

    assert_eq!(
        resp.status().as_u16(),
        200,
        "AC-3: unknown IP lookup must return HTTP 200 (not 404)"
    );

    let body: serde_json::Value = resp
        .json()
        .await
        .expect("AC-3: response must be valid JSON");

    let threat_score = body
        .get("threat_score")
        .and_then(|v| v.as_u64())
        .expect("AC-3: response must contain 'threat_score' field");

    assert_eq!(
        threat_score, 0,
        "AC-3: benign-default threat_score must be 0 for unknown IP"
    );

    let is_malicious = body
        .get("threat_is_known_malicious")
        .and_then(|v| v.as_bool())
        .expect("AC-3: response must contain 'threat_is_known_malicious'");

    assert!(
        !is_malicious,
        "AC-3: threat_is_known_malicious must be false for unknown IP"
    );

    let sources = body
        .get("threat_sources")
        .and_then(|v| v.as_array())
        .expect("AC-3: response must contain 'threat_sources' array");

    assert!(
        sources.is_empty(),
        "AC-3: threat_sources must be empty for unknown IP, got: {sources:?}"
    );

    // The AC specifies the response identifies the queried IP.
    // The field must be present and equal to the queried value.
    let ip_field = body
        .get("ip")
        .and_then(|v| v.as_str())
        .expect("AC-3: response must contain 'ip' field identifying the queried address");

    assert_eq!(
        ip_field, UNKNOWN_IP,
        "AC-3: 'ip' field must match queried IP address"
    );
}
