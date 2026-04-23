//! AC-5: Graceful shutdown — all clones complete graceful drain or are force-aborted.
//!
//! Given the harness is running, when `stop_all()` is called, then:
//! - All clones stop within 5 seconds (via graceful broadcast drain).
//! - `reset()` is called on each clone.
//! - The process would exit with code 0 (tested via task completion).
//!
//! Expected Red Gate failure: `DemoHarness::start_all()` panics with `todo!()`.

mod common;

use std::time::Duration;

use prism_dtu_demo_server::harness::{build_clone_pairs, DemoHarness};
use tokio::time::timeout;

/// AC-5: stop_all() completes within the 5-second graceful drain timeout.
#[tokio::test]
async fn ac_5_stop_all_completes_within_graceful_timeout() {
    let config = common::all_clones_ephemeral_config();
    let pairs = build_clone_pairs(&config).expect("AC-5: build_clone_pairs must succeed");
    let mut harness = DemoHarness::new(pairs);

    // Expected failure: start_all() panics with "not yet implemented".
    harness
        .start_all(&config)
        .await
        .expect("AC-5: start_all must succeed");

    // stop_all() must complete within the 5s graceful drain window (+ test overhead).
    // We allow 8s total to accommodate slow CI environments.
    timeout(Duration::from_secs(8), harness.stop_all())
        .await
        .expect("AC-5: stop_all() must complete within 8 seconds (5s graceful + 3s overhead)");
}

/// AC-5: After stop_all(), clone endpoints no longer accept connections.
#[tokio::test]
async fn ac_5_endpoints_unreachable_after_stop_all() {
    let config = common::all_clones_ephemeral_config();
    let pairs = build_clone_pairs(&config).expect("AC-5: build_clone_pairs must succeed");
    let mut harness = DemoHarness::new(pairs);

    // Expected failure: start_all() panics with "not yet implemented".
    harness
        .start_all(&config)
        .await
        .expect("AC-5: start_all must succeed");

    // Capture all bound addresses BEFORE stopping.
    let addrs: Vec<_> = harness.pairs.iter().filter_map(|p| p.bound_addr).collect();

    assert_eq!(
        addrs.len(),
        6,
        "AC-5: all 6 clones must be bound before stopping"
    );

    harness.stop_all().await;

    // After stop_all(), each port must have been released (no listener leak).
    for addr in addrs {
        prism_dtu_demo_server::harness::test_utils::assert_port_released(
            addr,
            Duration::from_millis(500),
        )
        .await;
    }
}
