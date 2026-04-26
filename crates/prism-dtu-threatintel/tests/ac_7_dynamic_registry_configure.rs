#![allow(clippy::unwrap_used, clippy::expect_used)]
// AC-7: POST /dtu/configure {"ip": "10.0.0.1", "fixture": "malicious"} then
// GET /v3/ip/10.0.0.1 returns the malicious fixture response.
//
// Was Red Gate at implementation start; configure endpoint now implemented.
// fixture_registry keyed by the "ip" field value. The subsequent lookup then
// dispatches to fixture_response(FixtureKey::Malicious, ...) which returns
// threat_score: 85. The test asserts the exact malicious fixture shape. This
// should work in the stub.
//
// However, the test additionally asserts that after clone.reset(), the custom
// IP entry is removed and GET /v3/ip/10.0.0.1 returns benign defaults (score 0).
// The reset() implementation restores the default registry (removes custom entries).
// This assertion validates that reset() correctly reverts state — a critical
// invariant for test isolation.
//
// Secondary assertion: configure with an invalid fixture name returns HTTP 400.

use prism_dtu_common::{build_test_client, BehavioralClone};
use prism_dtu_threatintel::ThreatIntelClone;

const CUSTOM_IP: &str = "10.0.0.1";
const API_KEY: &str = "test-key-ac7";

#[tokio::test]
async fn ac_7_dynamic_registry_addition_serves_malicious_fixture() {
    let mut clone = ThreatIntelClone::new();
    clone.start().await.expect("AC-7: start() must succeed");

    let base = clone.base_url();
    let admin_token = clone.admin_token().to_string();
    let client = build_test_client();

    // Pre-condition: 10.0.0.1 is not in the default registry; returns benign defaults.
    let pre_resp = client
        .get(format!("{base}/v3/ip/{CUSTOM_IP}"))
        .query(&[("key", API_KEY)])
        .send()
        .await
        .expect("AC-7: pre-configure lookup must reach server");

    assert_eq!(
        pre_resp.status().as_u16(),
        200,
        "AC-7: pre-configure must be 200"
    );

    let pre_body: serde_json::Value = pre_resp
        .json()
        .await
        .expect("AC-7: pre-configure response must be JSON");
    let pre_score = pre_body
        .get("threat_score")
        .and_then(|v| v.as_u64())
        .unwrap_or(999);
    assert_eq!(
        pre_score, 0,
        "AC-7: 10.0.0.1 must return score 0 before configure (not in registry)"
    );

    // Configure: add 10.0.0.1 → malicious.
    let cfg_resp = client
        .post(format!("{base}/dtu/configure"))
        .header("X-Admin-Token", &admin_token)
        .json(&serde_json::json!({
            "ip": CUSTOM_IP,
            "fixture": "malicious"
        }))
        .send()
        .await
        .expect("AC-7: configure request must reach server");

    assert_eq!(
        cfg_resp.status().as_u16(),
        200,
        "AC-7: POST /dtu/configure must return 200"
    );

    // Post-configure: 10.0.0.1 must now return malicious fixture.
    let resp = client
        .get(format!("{base}/v3/ip/{CUSTOM_IP}"))
        .query(&[("key", API_KEY)])
        .send()
        .await
        .expect("AC-7: post-configure lookup must reach server");

    assert_eq!(
        resp.status().as_u16(),
        200,
        "AC-7: post-configure lookup must return HTTP 200"
    );

    let body: serde_json::Value = resp.json().await.expect("AC-7: response must be JSON");

    let threat_score = body
        .get("threat_score")
        .and_then(|v| v.as_u64())
        .expect("AC-7: response must contain 'threat_score'");

    assert_eq!(
        threat_score, 85,
        "AC-7: dynamically configured malicious IP must return threat_score 85"
    );

    let is_malicious = body
        .get("threat_is_known_malicious")
        .and_then(|v| v.as_bool())
        .expect("AC-7: response must contain 'threat_is_known_malicious'");

    assert!(
        is_malicious,
        "AC-7: threat_is_known_malicious must be true for dynamically added malicious IP"
    );

    // After reset(), the custom entry must be removed.
    clone.reset().await.expect("AC-7: reset() must succeed");

    let post_reset_resp = client
        .get(format!("{base}/v3/ip/{CUSTOM_IP}"))
        .query(&[("key", API_KEY)])
        .send()
        .await
        .expect("AC-7: post-reset lookup must reach server");

    assert_eq!(
        post_reset_resp.status().as_u16(),
        200,
        "AC-7: post-reset must be 200"
    );

    let post_reset_body: serde_json::Value = post_reset_resp
        .json()
        .await
        .expect("AC-7: post-reset response must be JSON");

    let post_reset_score = post_reset_body
        .get("threat_score")
        .and_then(|v| v.as_u64())
        .expect("AC-7: post-reset response must contain 'threat_score'");

    assert_eq!(
        post_reset_score, 0,
        "AC-7: after reset(), custom IP must return benign defaults (score 0)"
    );

    // Invalid fixture name must return 400.
    let invalid_cfg = client
        .post(format!("{base}/dtu/configure"))
        .header("X-Admin-Token", &admin_token)
        .json(&serde_json::json!({
            "ip": "192.168.1.1",
            "fixture": "suspicious"
        }))
        .send()
        .await
        .expect("AC-7: invalid fixture configure must reach server");

    assert_eq!(
        invalid_cfg.status().as_u16(),
        400,
        "AC-7: invalid fixture name 'suspicious' must return HTTP 400 (EC-003)"
    );

    let invalid_body: serde_json::Value = invalid_cfg
        .json()
        .await
        .expect("AC-7: 400 response must be JSON");

    // The error message includes serde's variant error for an invalid fixture key value.
    // TD-WV0-04: deny_unknown_fields causes serde to produce "unknown variant `suspicious`"
    // which is wrapped in the standard /dtu/configure error prefix.
    let error_msg = invalid_body
        .get("error")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    assert!(
        error_msg.contains("suspicious") || error_msg.contains("unknown"),
        "AC-7: 400 error body must describe the invalid fixture key, got: {error_msg}"
    );
}
