//! AC-6: seed=42 produces identical responses on repeated calls (S-6.07).
//!
//! Given `CrowdstrikeClone` is started with `seed: 42`, When
//! `GET /detects/queries/detects/v1` is called twice with the same query params,
//! Then both responses are identical (deterministic seeding).
//!
//! Expected Red Gate failure: `CrowdstrikeClone::start()` panics with
//! "not yet implemented".

use prism_dtu_common::{BehavioralClone, StubConfig};
use prism_dtu_crowdstrike::CrowdstrikeClone;

/// AC-6: Two calls to GET /detects/queries/detects/v1 with seed=42 return identical bodies.
#[tokio::test]
async fn ac_6_seed_42_detection_query_is_deterministic() {
    // Expected failure: CrowdstrikeClone::start() panics with "not yet implemented".
    let mut clone = CrowdstrikeClone::with_config(StubConfig {
        seed: 42,
        latency_ms: 0,
        failure_mode: prism_dtu_common::FailureMode::None,
        bind: None,
    });
    clone.start().await.expect("AC-6: start() must succeed");

    let base_url = clone.base_url();
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .unwrap();

    // First call.
    let resp1 = client
        .get(format!("{base_url}/detects/queries/detects/v1"))
        .query(&[("limit", "10"), ("offset", "0")])
        .header("Authorization", "Bearer dtu-fake-cs-token")
        .send()
        .await
        .expect("AC-6: first request must reach server");

    assert_eq!(
        resp1.status().as_u16(),
        200,
        "AC-6: first call must return 200"
    );

    let body1: serde_json::Value = resp1.json().await.expect("AC-6: first body must be JSON");

    // Second call with same params.
    let resp2 = client
        .get(format!("{base_url}/detects/queries/detects/v1"))
        .query(&[("limit", "10"), ("offset", "0")])
        .header("Authorization", "Bearer dtu-fake-cs-token")
        .send()
        .await
        .expect("AC-6: second request must reach server");

    assert_eq!(
        resp2.status().as_u16(),
        200,
        "AC-6: second call must return 200"
    );

    let body2: serde_json::Value = resp2.json().await.expect("AC-6: second body must be JSON");

    assert_eq!(
        body1, body2,
        "AC-6: two calls with seed=42 and identical params must return identical responses"
    );
}

/// AC-6: Determinism holds for host ID list as well.
#[tokio::test]
async fn ac_6_seed_42_host_query_is_deterministic() {
    let mut clone = CrowdstrikeClone::with_config(StubConfig {
        seed: 42,
        latency_ms: 0,
        failure_mode: prism_dtu_common::FailureMode::None,
        bind: None,
    });
    clone
        .start()
        .await
        .expect("AC-6 hosts: start() must succeed");

    let base_url = clone.base_url();
    let client = reqwest::Client::new();

    let resp1 = client
        .get(format!("{base_url}/devices/queries/devices/v1"))
        .query(&[("limit", "5"), ("offset", "0")])
        .header("Authorization", "Bearer dtu-fake-cs-token")
        .send()
        .await
        .expect("AC-6 hosts: first request must reach server");

    let body1: serde_json::Value = resp1
        .json()
        .await
        .expect("AC-6 hosts: first body must be JSON");

    let resp2 = client
        .get(format!("{base_url}/devices/queries/devices/v1"))
        .query(&[("limit", "5"), ("offset", "0")])
        .header("Authorization", "Bearer dtu-fake-cs-token")
        .send()
        .await
        .expect("AC-6 hosts: second request must reach server");

    let body2: serde_json::Value = resp2
        .json()
        .await
        .expect("AC-6 hosts: second body must be JSON");

    assert_eq!(
        body1, body2,
        "AC-6 hosts: two calls with seed=42 and identical params must return identical responses"
    );
}

/// AC-6: Different seeds produce different responses (non-trivial seeding check).
#[tokio::test]
async fn ac_6_different_seeds_produce_different_responses() {
    // Start two clones with different seeds.
    let mut clone_42 = CrowdstrikeClone::with_config(StubConfig {
        seed: 42,
        ..StubConfig::default()
    });
    let mut clone_99 = CrowdstrikeClone::with_config(StubConfig {
        seed: 99,
        ..StubConfig::default()
    });

    clone_42
        .start()
        .await
        .expect("AC-6 seeds: clone_42 start() must succeed");
    clone_99
        .start()
        .await
        .expect("AC-6 seeds: clone_99 start() must succeed");

    let client = reqwest::Client::new();

    let resp_42 = client
        .get(format!(
            "{}/detects/queries/detects/v1",
            clone_42.base_url()
        ))
        .header("Authorization", "Bearer dtu-fake-cs-token")
        .send()
        .await
        .expect("AC-6 seeds: clone_42 request must reach server");

    let resp_99 = client
        .get(format!(
            "{}/detects/queries/detects/v1",
            clone_99.base_url()
        ))
        .header("Authorization", "Bearer dtu-fake-cs-token")
        .send()
        .await
        .expect("AC-6 seeds: clone_99 request must reach server");

    let body_42: serde_json::Value = resp_42
        .json()
        .await
        .expect("AC-6 seeds: body_42 must be JSON");
    let body_99: serde_json::Value = resp_99
        .json()
        .await
        .expect("AC-6 seeds: body_99 must be JSON");

    // NOTE: This assertion is "best effort" — if the fixture is purely static (not
    // seed-influenced), both will be equal and this test will fail at assertion rather
    // than at start(). That is still a Red Gate failure. If the fixture is static, this
    // assertion should be removed and replaced with a note that seeds affect ordering,
    // not fixture content. Spec item is noted in the report.
    assert_ne!(
        body_42, body_99,
        "AC-6 seeds: seed=42 and seed=99 must produce different responses (or fixture is not seed-influenced — see spec underspecification note)"
    );
}
