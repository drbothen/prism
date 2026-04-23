//! AC-10: All 6 clone /dtu/health endpoints return HTTP 200 with `{"status":"ok"}`.
//!
//! Given the harness is running, when `GET /dtu/health` is issued on each of the 6
//! clone ports, then all 6 return HTTP 200 with `{"status":"ok"}`.
//!
//! Expected Red Gate failure: `DemoHarness::start_all()` panics with `todo!()`.

mod common;

use prism_dtu_demo_server::harness::{build_clone_pairs, DemoHarness};

/// AC-10: /dtu/health returns 200 + {"status":"ok"} on all 6 clone ports.
#[tokio::test]
async fn ac_10_all_six_health_endpoints_return_200() {
    let config = common::all_clones_ephemeral_config();
    let pairs = build_clone_pairs(&config).expect("AC-10: build_clone_pairs must succeed");
    let mut harness = DemoHarness::new(pairs);

    // Expected failure: start_all() panics with "not yet implemented".
    harness
        .start_all(&config)
        .await
        .expect("AC-10: start_all must succeed");

    let client = common::http_client();

    let clone_names = ["crowdstrike", "claroty", "cyberint", "armis", "threatintel", "nvd"];

    for name in clone_names {
        let addr = harness
            .pairs
            .iter()
            .find(|p| p.name == name)
            .and_then(|p| p.bound_addr)
            .unwrap_or_else(|| panic!("AC-10: {name} must have a bound address"));

        let url = format!("http://{addr}/dtu/health");

        let resp = client
            .get(&url)
            .send()
            .await
            .unwrap_or_else(|e| panic!("AC-10: GET {url} failed: {e}"));

        assert_eq!(
            resp.status(),
            200,
            "AC-10: {name} /dtu/health must return HTTP 200, got {}",
            resp.status()
        );

        let body: serde_json::Value = resp
            .json()
            .await
            .unwrap_or_else(|e| panic!("AC-10: {name} /dtu/health must return JSON: {e}"));

        assert_eq!(
            body.get("status").and_then(|v| v.as_str()),
            Some("ok"),
            "AC-10: {name} /dtu/health must return {{\"status\":\"ok\"}}; got: {body}"
        );
    }

    harness.stop_all().await;
}

/// AC-10: /dtu/health is served directly on each clone's own port (no harness proxy).
#[tokio::test]
async fn ac_10_health_served_on_clone_own_port_not_harness_proxy() {
    let config = common::all_clones_ephemeral_config();
    let pairs = build_clone_pairs(&config).expect("AC-10: build_clone_pairs must succeed");
    let mut harness = DemoHarness::new(pairs);

    // Expected failure: start_all() panics with "not yet implemented".
    harness
        .start_all(&config)
        .await
        .expect("AC-10: start_all must succeed");

    // All 6 bound addresses must be DISTINCT (no clone sharing a port).
    let addrs: Vec<_> = harness
        .pairs
        .iter()
        .filter_map(|p| p.bound_addr)
        .collect();

    let mut deduped = addrs.clone();
    deduped.dedup();
    assert_eq!(
        addrs.len(),
        deduped.len(),
        "AC-10: all 6 clones must bind on distinct addresses; found duplicate in: {addrs:?}"
    );

    harness.stop_all().await;
}
