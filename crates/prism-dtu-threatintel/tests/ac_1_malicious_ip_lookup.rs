// AC-1: GET /v3/ip/45.55.100.1 with valid API key returns threat_score: 85,
// threat_is_known_malicious: true, and threat_sources including "greynoise".
//
// Expected Red Gate failure: the stub response uses "lookup_value" as the
// response key; the test additionally asserts that threat_sources is an array
// containing "greynoise" as a string value, and checks exact threat_score.
// The test is written to the AC contract; if the stub shape is correct these
// pass, but any deviation in field values causes failure.
//
// Note: this test also verifies the server binds and is reachable (pre-condition
// for all subsequent AC tests).

use prism_dtu_common::{build_test_client, BehavioralClone};
use prism_dtu_threatintel::ThreatIntelClone;

const MALICIOUS_IP: &str = "45.55.100.1";
const API_KEY: &str = "test-key-valid";

#[tokio::test]
async fn ac_1_malicious_ip_returns_threat_score_85_and_greynoise_source() {
    let mut clone = ThreatIntelClone::new();
    clone.start().await.expect("AC-1: ThreatIntelClone::start() must succeed");

    let base = clone.base_url();
    assert!(clone.bound_addr().port() > 0, "AC-1: bound_addr must have non-zero port");

    let client = build_test_client();

    let resp = client
        .get(format!("{base}/v3/ip/{MALICIOUS_IP}"))
        .query(&[("key", API_KEY)])
        .send()
        .await
        .expect("AC-1: request must reach ThreatIntelClone server");

    assert_eq!(
        resp.status().as_u16(),
        200,
        "AC-1: malicious IP lookup must return HTTP 200"
    );

    let body: serde_json::Value = resp
        .json()
        .await
        .expect("AC-1: response must be valid JSON");

    let threat_score = body
        .get("threat_score")
        .and_then(|v| v.as_u64())
        .expect("AC-1: response must contain 'threat_score' field");

    assert_eq!(
        threat_score, 85,
        "AC-1: threat_score must be 85 for known malicious IP"
    );

    let is_malicious = body
        .get("threat_is_known_malicious")
        .and_then(|v| v.as_bool())
        .expect("AC-1: response must contain 'threat_is_known_malicious' field");

    assert!(
        is_malicious,
        "AC-1: threat_is_known_malicious must be true for 45.55.100.1"
    );

    let sources = body
        .get("threat_sources")
        .and_then(|v| v.as_array())
        .expect("AC-1: response must contain 'threat_sources' array");

    let source_strings: Vec<&str> = sources
        .iter()
        .filter_map(|v| v.as_str())
        .collect();

    assert!(
        source_strings.contains(&"greynoise"),
        "AC-1: threat_sources must include 'greynoise', got: {source_strings:?}"
    );
}
