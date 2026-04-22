// AC-2: GET /v3/ip/8.8.8.8 with valid API key returns threat_is_known_malicious: false
// and threat_score below 20.
//
// Expected Red Gate failure: test asserts exact contract from AC-2. If the stub
// returns a score of 5 and false for is_known_malicious, those assertions pass.
// The test enforces the < 20 bound and the false value as strict assertions.

use prism_dtu_common::{build_test_client, BehavioralClone};
use prism_dtu_threatintel::ThreatIntelClone;

const BENIGN_IP: &str = "8.8.8.8";
const API_KEY: &str = "test-key-valid";

#[tokio::test]
async fn ac_2_benign_ip_returns_not_malicious_with_score_below_20() {
    let mut clone = ThreatIntelClone::new();
    clone.start().await.expect("AC-2: start() must succeed");

    let base = clone.base_url();
    let client = build_test_client();

    let resp = client
        .get(format!("{base}/v3/ip/{BENIGN_IP}"))
        .query(&[("key", API_KEY)])
        .send()
        .await
        .expect("AC-2: request must reach server");

    assert_eq!(
        resp.status().as_u16(),
        200,
        "AC-2: benign IP lookup must return HTTP 200"
    );

    let body: serde_json::Value = resp
        .json()
        .await
        .expect("AC-2: response must be valid JSON");

    let is_malicious = body
        .get("threat_is_known_malicious")
        .and_then(|v| v.as_bool())
        .expect("AC-2: response must contain 'threat_is_known_malicious'");

    assert!(
        !is_malicious,
        "AC-2: threat_is_known_malicious must be false for 8.8.8.8 (benign)"
    );

    let threat_score = body
        .get("threat_score")
        .and_then(|v| v.as_u64())
        .expect("AC-2: response must contain 'threat_score'");

    assert!(
        threat_score < 20,
        "AC-2: threat_score must be below 20 for benign IP, got {threat_score}"
    );
}
