//! AC-1: All 6 clones bind their configured ports and a URL table is printed.
//!
//! Given a valid `demo.toml` enabling all 6 merged clones with ephemeral ports,
//! when `DemoHarness::start_all()` is called, then all 6 clones are bound and
//! serving, and `print_url_table()` runs without panic.
//!
//! Expected Red Gate failure: `DemoHarness::start_all()` panics with
//! `"not yet implemented"` (the `todo!()` in harness.rs).
#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use prism_dtu_demo_server::harness::{build_clone_pairs, DemoHarness};

/// AC-1: start_all() starts all 6 clones and each is bound to a non-zero port.
#[tokio::test]
async fn ac_1_all_six_clones_bind_ephemeral_ports() {
    let config = common::all_clones_ephemeral_config();

    // build_clone_pairs is also todo!() — either this or start_all will panic first.
    let pairs = build_clone_pairs(&config).expect("AC-1: build_clone_pairs must succeed");
    assert_eq!(pairs.len(), 6, "AC-1: exactly 6 clone pairs must be built");

    let mut harness = DemoHarness::new(pairs);

    // Expected failure: start_all() panics with "not yet implemented".
    harness
        .start_all(&config, None)
        .await
        .expect("AC-1: start_all() must succeed");

    // Every pair must have a bound address after start_all returns.
    for pair in &harness.pairs {
        let addr = pair
            .bound_addr
            .expect("AC-1: each ClonePair must have bound_addr after start_all");
        assert_ne!(
            addr.port(),
            0,
            "AC-1: {} bound port must be non-zero",
            pair.name
        );
        assert!(
            addr.ip().is_loopback(),
            "AC-1: {} must bind on loopback, got {}",
            pair.name,
            addr
        );
    }

    harness.stop_all().await;
}

/// AC-1: print_url_table() runs without panic after start_all() succeeds.
#[tokio::test]
async fn ac_1_print_url_table_runs_without_panic() {
    let config = common::all_clones_ephemeral_config();
    let pairs = build_clone_pairs(&config).expect("AC-1: build_clone_pairs must succeed");
    let mut harness = DemoHarness::new(pairs);

    // Expected failure: start_all() todo!() panics.
    harness
        .start_all(&config, None)
        .await
        .expect("AC-1: start_all must succeed before printing URL table");

    // print_url_table() must not panic (it is also todo!()).
    harness.print_url_table();

    harness.stop_all().await;
}
