//! AC-11: Partial-startup cleanup — if clone N fails to bind (EADDRINUSE) and
//! `continue_on_error = false`, the N-1 already-started clones have `stop()` called
//! on them (no listener leak) and `start_all()` returns `Err`.
//!
//! `StartReport` must have:
//!   - `cleaned_up_after_failure` == ["crowdstrike", "claroty", "cyberint"]
//!   - `failed_at` set to ("armis", AddrInUse-kind error)
//!   - `successfully_started` empty (cleanup rolled back the 3 started clones)
//!   - `skipped_due_to_error` empty
//!
//! Expected Red Gate failure: `DemoHarness::start_all()` panics with `todo!()`.

#![allow(clippy::unwrap_used, clippy::expect_used)]
mod common;

use std::net::SocketAddr;
use std::time::Duration;

use prism_dtu_demo_server::config::{CloneConfig, ClonesConfig, DemoConfig};
use prism_dtu_demo_server::harness::{build_clone_pairs, DemoHarness};

/// Pre-bind a port to force EADDRINUSE on the 4th clone (armis).
///
/// Returns the bound listener (keep alive for the duration of the test) and the port.
async fn pre_bind_port() -> (tokio::net::TcpListener, u16) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("AC-11: failed to pre-bind a port");
    let port = listener.local_addr().expect("AC-11: no local_addr").port();
    (listener, port)
}

/// AC-11: start_all aborts and calls stop() on N-1 clones when clone 4 fails to bind.
#[tokio::test]
async fn ac_11_partial_startup_cleanup_on_bind_failure() {
    // Pre-bind a port that armis will try to bind to.
    let (_held_listener, blocked_port) = pre_bind_port().await;

    let config = DemoConfig {
        harness: Default::default(),
        clones: ClonesConfig {
            crowdstrike: CloneConfig {
                enabled: true,
                port: 0, // ephemeral — will succeed
                continue_on_error: false,
                ..Default::default()
            },
            claroty: CloneConfig {
                enabled: true,
                port: 0,
                continue_on_error: false,
                ..Default::default()
            },
            cyberint: CloneConfig {
                enabled: true,
                port: 0,
                continue_on_error: false,
                ..Default::default()
            },
            armis: CloneConfig {
                enabled: true,
                port: blocked_port,       // EADDRINUSE — this clone will fail
                continue_on_error: false, // abort path
                ..Default::default()
            },
            threatintel: CloneConfig {
                enabled: true,
                port: 0,
                continue_on_error: false,
                ..Default::default()
            },
            nvd: CloneConfig {
                enabled: true,
                port: 0,
                continue_on_error: false,
                ..Default::default()
            },
        },
    };

    let pairs = build_clone_pairs(&config).expect("AC-11: build_clone_pairs must succeed");
    let mut harness = DemoHarness::new(pairs);

    // Expected failure: start_all() panics with "not yet implemented".
    // When implemented: must return Err because armis fails to bind.
    let result = harness.start_all(&config, None).await;

    assert!(
        result.is_err(),
        "AC-11: start_all() must return Err when clone 4 (armis) fails to bind (EADDRINUSE)"
    );

    let report = harness.last_start_report();

    // StartReport invariant (abort path): failed_at is set, cleaned_up_after_failure
    // has the 3 already-started clones, skipped_due_to_error is empty.
    assert!(
        report.failed_at.is_some(),
        "AC-11: StartReport.failed_at must be Some when continue_on_error=false and bind fails"
    );

    let (failed_name, ref failed_err) =
        report.failed_at.as_ref().expect("AC-11: failed_at is Some");
    assert_eq!(
        failed_name, "armis",
        "AC-11: failed_at name must be 'armis' (4th clone); got: {failed_name}"
    );
    assert_eq!(
        failed_err.kind(),
        std::io::ErrorKind::AddrInUse,
        "AC-11: failed_at error kind must be AddrInUse; got: {:?}",
        failed_err.kind()
    );

    assert_eq!(
        report.cleaned_up_after_failure,
        vec!["crowdstrike", "claroty", "cyberint"],
        "AC-11: cleaned_up_after_failure must list the 3 rolled-back clones in order"
    );

    assert!(
        report.successfully_started.is_empty(),
        "AC-11: successfully_started must be empty after abort cleanup; got: {:?}",
        report.successfully_started
    );

    assert!(
        report.skipped_due_to_error.is_empty(),
        "AC-11: skipped_due_to_error must be empty under continue_on_error=false"
    );
}

/// AC-11: No listener leak — all 3 already-started clones release their ports after cleanup.
#[tokio::test]
async fn ac_11_no_listener_leak_after_partial_startup_failure() {
    let (_held_listener, blocked_port) = pre_bind_port().await;

    let config = DemoConfig {
        harness: Default::default(),
        clones: ClonesConfig {
            crowdstrike: CloneConfig {
                enabled: true,
                port: 0,
                continue_on_error: false,
                ..Default::default()
            },
            claroty: CloneConfig {
                enabled: true,
                port: 0,
                continue_on_error: false,
                ..Default::default()
            },
            cyberint: CloneConfig {
                enabled: true,
                port: 0,
                continue_on_error: false,
                ..Default::default()
            },
            armis: CloneConfig {
                enabled: true,
                port: blocked_port,
                continue_on_error: false,
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

    let pairs = build_clone_pairs(&config).expect("AC-11: build_clone_pairs must succeed");
    let mut harness = DemoHarness::new(pairs);

    // Capture None bound_addrs before start (all should be None).
    // After the expected failure start_all returns Err, any previously bound
    // addresses in pairs should have been cleaned up.

    // Expected failure: start_all() panics with "not yet implemented".
    let _ = harness.start_all(&config, None).await;

    // After partial-startup cleanup, the 3 started clones must have released ports.
    // We check the pairs that DID start (crowdstrike, claroty, cyberint).
    let started_addrs: Vec<SocketAddr> = harness
        .pairs
        .iter()
        .filter(|p| ["crowdstrike", "claroty", "cyberint"].contains(&p.name.as_str()))
        .filter_map(|p| p.bound_addr)
        .collect();

    for addr in started_addrs {
        prism_dtu_demo_server::harness::test_utils::assert_port_released(
            addr,
            Duration::from_millis(200),
        )
        .await;
    }
}
