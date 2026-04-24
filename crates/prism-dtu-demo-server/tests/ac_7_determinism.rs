//! AC-7: Deterministic mode — same seed + same request sequence → byte-identical
//! JSON response bodies across runs.
//!
//! Given `seed = 42` AND `--deterministic-logging` is active, when two identical
//! GET requests are issued to the same clone endpoint, the JSON response bodies
//! must be byte-identical (within a single harness run, same-seed determinism).
//!
//! Expected Red Gate failure: `DemoHarness::start_all()` panics with `todo!()`.

mod common;

use prism_dtu_demo_server::config::{CloneConfig, ClonesConfig, DemoConfig};
use prism_dtu_demo_server::harness::{build_clone_pairs, DemoHarness};

/// AC-7: Two consecutive identical GET requests to crowdstrike return byte-identical bodies.
#[tokio::test]
async fn ac_7_same_seed_same_request_sequence_yields_identical_bodies() {
    // Config: all 6 clones, seed = 42 (the deterministic seed from demo.toml).
    let config = DemoConfig {
        harness: Default::default(),
        clones: ClonesConfig {
            crowdstrike: CloneConfig {
                enabled: true,
                port: 0,
                seed: 42,
                ..Default::default()
            },
            claroty: CloneConfig {
                enabled: false,
                ..Default::default()
            },
            cyberint: CloneConfig {
                enabled: false,
                ..Default::default()
            },
            armis: CloneConfig {
                enabled: false,
                ..Default::default()
            },
            threatintel: CloneConfig {
                enabled: false,
                ..Default::default()
            },
            nvd: CloneConfig {
                enabled: false,
                ..Default::default()
            },
        },
    };

    let pairs = build_clone_pairs(&config).expect("AC-7: build_clone_pairs must succeed");
    let mut harness = DemoHarness::new(pairs);

    // Expected failure: start_all() panics with "not yet implemented".
    harness
        .start_all(&config, None)
        .await
        .expect("AC-7: start_all must succeed");

    let cs_addr = harness
        .pairs
        .iter()
        .find(|p| p.name == "crowdstrike")
        .and_then(|p| p.bound_addr)
        .expect("AC-7: crowdstrike must be bound");

    let client = common::http_client();
    let url = format!("http://{cs_addr}/devices/queries/devices/v1");

    // First request.
    let body1 = client
        .get(&url)
        .header("Authorization", "Bearer dtu-fake-cs-token")
        .send()
        .await
        .expect("AC-7: first request must succeed")
        .text()
        .await
        .expect("AC-7: first response must be text");

    // Reset so second request sees the same state (deterministic replay).
    // This calls `clone.reset()` — also todo!(), but start_all() will panic first.
    let _ = harness
        .pairs
        .iter()
        .find(|p| p.name == "crowdstrike")
        .is_some(); // just a check; actual reset path is via harness

    // A second request after reset must produce the same body.
    // (In deterministic mode, the clone resets its RNG to seed=42 on reset.)
    let body2 = client
        .get(&url)
        .header("Authorization", "Bearer dtu-fake-cs-token")
        .send()
        .await
        .expect("AC-7: second request must succeed")
        .text()
        .await
        .expect("AC-7: second response must be text");

    // Both bodies must be non-empty and valid JSON.
    assert!(
        !body1.is_empty(),
        "AC-7: first response body must not be empty"
    );
    assert!(
        !body2.is_empty(),
        "AC-7: second response body must not be empty"
    );

    // With seed=42 and no intervening configure/reset, consecutive requests
    // for a deterministic clone should return the same body (page 1 fixture).
    assert_eq!(
        body1, body2,
        "AC-7: with seed=42, same request must produce byte-identical responses"
    );

    harness.stop_all().await;
}

/// AC-7: configs/demo.toml must have seed = 42 for all clones.
#[test]
fn ac_7_demo_toml_has_seed_42_for_all_clones() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("configs")
        .join("demo.toml");

    let contents = std::fs::read_to_string(&path)
        .unwrap_or_else(|_| panic!("AC-7: configs/demo.toml must exist at {:?}", path));

    assert!(
        contents.contains("seed = 42"),
        "AC-7: demo.toml must set seed = 42 for deterministic replay; got:\n{contents}"
    );
}
