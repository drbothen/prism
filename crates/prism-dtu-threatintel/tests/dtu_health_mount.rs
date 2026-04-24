#![allow(clippy::unwrap_used, clippy::expect_used)]
// TD-WV0-05: GET /dtu/health must be mounted on ThreatIntelClone.
//
// The L1 canonical reference (prism-dtu-crowdstrike) mounts GET /dtu/health as a
// no-auth DTU introspection endpoint returning HTTP 200 `{"status": "ok"}`.
// ThreatIntelClone currently mounts /dtu/configure but is MISSING /dtu/health —
// this test will fail with HTTP 404 at Red Gate.
//
// Expected Red Gate failure: GET /dtu/health returns 404 (route not mounted).

use prism_dtu_common::{build_test_client, BehavioralClone};
use prism_dtu_threatintel::ThreatIntelClone;

#[tokio::test]
async fn test_dtu_health_mount_threatintel_returns_200_status_ok() {
    let mut clone = ThreatIntelClone::new();
    clone
        .start()
        .await
        .expect("TD-WV0-05: ThreatIntelClone::start() must succeed");

    let base_url = clone.base_url();
    let client = build_test_client();

    let resp = client
        .get(format!("{base_url}/dtu/health"))
        .send()
        .await
        .expect("TD-WV0-05: GET /dtu/health must reach ThreatIntelClone server");

    assert_eq!(
        resp.status().as_u16(),
        200,
        "TD-WV0-05: GET /dtu/health must return HTTP 200 (no auth required)"
    );

    let body: serde_json::Value = resp
        .json()
        .await
        .expect("TD-WV0-05: GET /dtu/health response must be valid JSON");

    assert_eq!(
        body.get("status").and_then(|v| v.as_str()).unwrap_or(""),
        "ok",
        "TD-WV0-05: GET /dtu/health body must be {{\"status\": \"ok\"}}"
    );
}
