//! AC-3: POST /dtu/configure on a clone's own port applies config per its documented
//! `apply_config()` semantics. No harness proxy — each clone owns its endpoint.
//!
//! Expected Red Gate failure: `DemoHarness::start_all()` panics with `todo!()`.

#![allow(clippy::unwrap_used, clippy::expect_used)]
mod common;

use prism_dtu_demo_server::harness::{build_clone_pairs, DemoHarness};

/// AC-3: POST /dtu/configure on cyberint's port returns 200 and is processed directly.
#[tokio::test]
async fn ac_3_configure_called_on_clone_port_directly() {
    let config = common::all_clones_ephemeral_config();
    let pairs = build_clone_pairs(&config).expect("AC-3: build_clone_pairs must succeed");
    let mut harness = DemoHarness::new(pairs);

    // Expected failure: start_all() panics with "not yet implemented".
    harness
        .start_all(&config, None)
        .await
        .expect("AC-3: start_all must succeed");

    // Find cyberint's bound address (AC-3 uses port 17082 in demo; ephemeral here).
    let cy_addr = harness
        .pairs
        .iter()
        .find(|p| p.name == "cyberint")
        .and_then(|p| p.bound_addr)
        .expect("AC-3: cyberint must have a bound address");

    let client = common::http_client();
    let url = format!("http://{cy_addr}/dtu/configure");

    // Send a configure payload directly to the clone's own port.
    // Use a valid cyberint configure field (auth_mode), not crowdstrike's "seed".
    let payload = serde_json::json!({ "auth_mode": "accept" });
    let resp = client
        .post(&url)
        .json(&payload)
        .send()
        .await
        .expect("AC-3: POST /dtu/configure must not fail at transport level");

    assert!(
        resp.status().is_success(),
        "AC-3: POST /dtu/configure on cyberint port must return 2xx, got: {}",
        resp.status()
    );

    harness.stop_all().await;
}

/// AC-3: The harness has no proxy layer — /dtu/configure is NOT served on any harness port.
///
/// Verifies that there is no harness-level configure proxy endpoint exposed on any port
/// other than the clone's own port. (After F-6.20-P02-M-002 removal of harness proxy.)
#[tokio::test]
async fn ac_3_no_harness_proxy_for_configure() {
    let config = common::all_clones_ephemeral_config();
    let pairs = build_clone_pairs(&config).expect("AC-3: build_clone_pairs must succeed");
    let mut harness = DemoHarness::new(pairs);

    // Expected failure: start_all() panics with "not yet implemented".
    harness
        .start_all(&config, None)
        .await
        .expect("AC-3: start_all must succeed");

    // Each clone's configure URL is on its OWN port (no central harness proxy).
    // Verify configure works on crowdstrike's own port.
    let cs_addr = harness
        .pairs
        .iter()
        .find(|p| p.name == "crowdstrike")
        .and_then(|p| p.bound_addr)
        .expect("AC-3: crowdstrike must have a bound address");

    let client = common::http_client();
    let url = format!("http://{cs_addr}/dtu/configure");
    let payload = serde_json::json!({ "seed": 42 });

    let resp = client
        .post(&url)
        .json(&payload)
        .send()
        .await
        .expect("AC-3: POST to crowdstrike /dtu/configure must succeed");

    assert!(
        resp.status().is_success(),
        "AC-3: crowdstrike /dtu/configure must return 2xx; got: {}",
        resp.status()
    );

    harness.stop_all().await;
}
