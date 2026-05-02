#![allow(clippy::unwrap_used, clippy::expect_used)]
// TD-WV0-05: POST /dtu/reset must be mounted on ThreatIntelClone.
//
// The L1 canonical reference (prism-dtu-crowdstrike) mounts POST /dtu/reset as a
// no-auth DTU introspection endpoint that clears all mutable state and returns
// HTTP 200 `{"status": "ok"}`.
//
// ThreatIntelClone currently mounts /dtu/configure but is MISSING /dtu/reset —
// the HTTP layer of reset is unimplemented (BehavioralClone::reset() exists but
// Was Red Gate at implementation start; reset endpoint now mounted.
// POST /dtu/reset returns 404.
//
// Acceptance criteria tested here:
//   1. POST /dtu/reset returns HTTP 200 `{"status": "ok"}` with X-Admin-Token.
//   2. After configuring a custom fixture via POST /dtu/configure and performing
//      one lookup (incrementing request_counter to 1), POST /dtu/reset clears
//      both the fixture_registry (custom entry removed) and request_counter (reset
//      to 0). Clearing is verified by:
//      a. The previously-configured IP now returns benign defaults (score 0).
//      b. A subsequent GET /dtu/request-count-style check is not applicable to
//         ThreatIntelClone, so we verify counter indirectly: configure
//         rate_limit_after=0 before reset; after reset that threshold is also
//         gone so requests succeed without 429.
//
// Was Red Gate at implementation start; POST /dtu/reset now returns 200.

use prism_dtu_common::{build_test_client, BehavioralClone};
use prism_dtu_threatintel::ThreatIntelClone;

const CUSTOM_IP: &str = "10.0.0.99";
const API_KEY: &str = "test-key-reset";

#[tokio::test]
async fn test_dtu_reset_mount_threatintel_returns_200_status_ok() {
    let mut clone = ThreatIntelClone::new();
    clone
        .start()
        .await
        .expect("TD-WV0-05: ThreatIntelClone::start() must succeed");

    let base_url = clone.base_url();
    let admin_token = clone.admin_token().to_string();
    let client = build_test_client();

    // Step 1: configure a custom fixture entry (adds to fixture_registry).
    let cfg_resp = client
        .post(format!("{base_url}/dtu/configure"))
        .header("X-Admin-Token", &admin_token)
        .json(&serde_json::json!({
            "ip": CUSTOM_IP,
            "fixture": "malicious"
        }))
        .send()
        .await
        .expect("TD-WV0-05: POST /dtu/configure must reach server");

    assert_eq!(
        cfg_resp.status().as_u16(),
        200,
        "TD-WV0-05: POST /dtu/configure must return 200 (pre-condition for reset test)"
    );

    // Step 2: perform one lookup to increment the request_counter.
    let lookup_resp = client
        .get(format!("{base_url}/v3/ip/{CUSTOM_IP}"))
        .query(&[("key", API_KEY)])
        .send()
        .await
        .expect("TD-WV0-05: GET /v3/ip lookup must reach server");

    assert_eq!(
        lookup_resp.status().as_u16(),
        200,
        "TD-WV0-05: lookup of configured malicious IP must return 200"
    );

    let lookup_body: serde_json::Value = lookup_resp
        .json()
        .await
        .expect("TD-WV0-05: lookup response must be valid JSON");

    assert_eq!(
        lookup_body
            .get("threat_score")
            .and_then(|v| v.as_u64())
            .unwrap_or(0),
        85,
        "TD-WV0-05: configured malicious IP must return threat_score 85 (pre-reset)"
    );

    // Step 3: POST /dtu/reset — this is the route under test.
    let reset_resp = client
        .post(format!("{base_url}/dtu/reset"))
        .header("X-Admin-Token", &admin_token)
        .send()
        .await
        .expect("TD-WV0-05: POST /dtu/reset must reach ThreatIntelClone server");

    assert_eq!(
        reset_resp.status().as_u16(),
        200,
        "TD-WV0-05: POST /dtu/reset must return HTTP 200"
    );

    let reset_body: serde_json::Value = reset_resp
        .json()
        .await
        .expect("TD-WV0-05: POST /dtu/reset response must be valid JSON");

    assert_eq!(
        reset_body
            .get("status")
            .and_then(|v| v.as_str())
            .unwrap_or(""),
        "ok",
        "TD-WV0-05: POST /dtu/reset body must be {{\"status\": \"ok\"}}"
    );

    // Step 4: verify fixture_registry is cleared — CUSTOM_IP must no longer be malicious.
    let post_reset_lookup = client
        .get(format!("{base_url}/v3/ip/{CUSTOM_IP}"))
        .query(&[("key", API_KEY)])
        .send()
        .await
        .expect("TD-WV0-05: post-reset lookup must reach server");

    assert_eq!(
        post_reset_lookup.status().as_u16(),
        200,
        "TD-WV0-05: post-reset lookup must return 200"
    );

    let post_reset_body: serde_json::Value = post_reset_lookup
        .json()
        .await
        .expect("TD-WV0-05: post-reset lookup response must be valid JSON");

    let post_reset_score = post_reset_body
        .get("threat_score")
        .and_then(|v| v.as_u64())
        .unwrap_or(999);

    assert_eq!(
        post_reset_score, 0,
        "TD-WV0-05: fixture_registry must be cleared by reset — CUSTOM_IP must return \
         benign default score 0, got {post_reset_score}"
    );

    // Step 5: verify request_counter is reset to 0.
    //
    // Approach: call POST /dtu/reset again to establish a clean baseline (counter=0,
    // rate_limit_after=None), then configure rate_limit_after=1, then verify that the
    // first request succeeds (counter 0→1; 1 > 1 is false → 200) and the second returns
    // 429 (counter 1→2; 2 > 1 is true → 429). This proves the counter starts from 0
    // after reset without relying on configure() having any counter-reset side effect.
    //
    // Note: Step 4 above performed one additional lookup (incrementing the counter to 1
    // after the Step 3 reset), so a second reset is required here for isolation.
    let reset2_resp = client
        .post(format!("{base_url}/dtu/reset"))
        .header("X-Admin-Token", &admin_token)
        .send()
        .await
        .expect("TD-WV0-05: second POST /dtu/reset (step-5 baseline) must reach server");

    assert_eq!(
        reset2_resp.status().as_u16(),
        200,
        "TD-WV0-05: second POST /dtu/reset must return 200"
    );

    let rl_cfg = client
        .post(format!("{base_url}/dtu/configure"))
        .header("X-Admin-Token", &admin_token)
        .json(&serde_json::json!({"rate_limit_after": 1}))
        .send()
        .await
        .expect("TD-WV0-05: rate_limit configure must reach server");

    assert_eq!(
        rl_cfg.status().as_u16(),
        200,
        "TD-WV0-05: rate_limit configure must return 200"
    );

    // First request: counter goes from 0 → 1; threshold is 1; 1 > 1 is false → 200.
    let first_resp = client
        .get(format!("{base_url}/v3/ip/8.8.8.8"))
        .query(&[("key", API_KEY)])
        .send()
        .await
        .expect("TD-WV0-05: first post-reset request must reach server");

    assert_eq!(
        first_resp.status().as_u16(),
        200,
        "TD-WV0-05: first post-reset request must return 200 — counter reset to 0 confirmed"
    );

    // Second request: counter goes from 1 → 2; 2 > 1 is true → 429.
    let second_resp = client
        .get(format!("{base_url}/v3/ip/8.8.8.8"))
        .query(&[("key", API_KEY)])
        .send()
        .await
        .expect("TD-WV0-05: second post-reset request must reach server");

    assert_eq!(
        second_resp.status().as_u16(),
        429,
        "TD-WV0-05: second post-reset request must return 429 — counter correctly \
         incremented from 0 after reset"
    );
}
