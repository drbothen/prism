//! AC-5: Graceful shutdown — all clones complete graceful drain or are force-aborted.
//!
//! Given the harness is running, when `stop_all()` is called, then:
//! - All clones stop within 5 seconds (via graceful broadcast drain).
//! - `reset()` is called on each clone.
//! - The process would exit with code 0 (tested via task completion).
//!
//! Expected Red Gate failure: `DemoHarness::start_all()` panics with `todo!()`.
//!
//! # TD-WV1-04-FU-002
//!
//! An additional TLS-variant test (`ac_5_tls_graceful_shutdown_releases_port`) is
//! included to exercise the TLS path through `stop()` and verify port release.

#![allow(clippy::unwrap_used, clippy::expect_used)]
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
        .start_all(&config, None)
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
        .start_all(&config, None)
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

/// AC-5 / TD-WV1-04-FU-002: TLS variant — `stop()` on a TLS clone releases the port.
///
/// Exercises the TLS shutdown path (axum_server::Handle::graceful_shutdown → JoinHandle await)
/// and verifies that after `clone.stop()` the port is freed and can be rebound.
///
/// This test only compiles and runs with the `tls` feature enabled.
#[cfg(feature = "tls")]
#[tokio::test]
async fn ac_5_tls_graceful_shutdown_releases_port() {
    use std::sync::Arc;

    use prism_dtu_common::BehavioralClone;
    use prism_dtu_crowdstrike::CrowdstrikeClone;
    use prism_dtu_demo_server::tls::inner;

    // Install rustls crypto provider (idempotent — safe to call multiple times).
    let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();

    // 1. Generate a self-signed cert and build RustlsConfig.
    let (cert_pem, key_pem, _cert_der) =
        inner::generate_self_signed_cert().expect("AC-5 TLS: cert generation must succeed");
    let rustls_cfg = inner::build_rustls_config(&cert_pem, &key_pem)
        .await
        .expect("AC-5 TLS: RustlsConfig must build");

    // 2. Start CrowdstrikeClone over TLS on an ephemeral port.
    let mut clone = CrowdstrikeClone::new();
    let bind_addr: std::net::SocketAddr = "127.0.0.1:0".parse().unwrap();
    let bound_addr = clone
        .start_on(bind_addr, None, Some(Arc::new(rustls_cfg)))
        .await
        .expect("AC-5 TLS: start_on with TLS must succeed");

    assert!(
        clone.is_tls_active(),
        "AC-5 TLS: is_tls_active() must return true after TLS start"
    );

    // 3. Verify the clone is up by doing an HTTPS GET to /dtu/health.
    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .timeout(Duration::from_secs(5))
        .build()
        .expect("AC-5 TLS: reqwest client must build");

    let health_url = format!("https://{}/dtu/health", bound_addr);
    let resp = client
        .get(&health_url)
        .send()
        .await
        .expect("AC-5 TLS: HTTPS GET to /dtu/health must succeed while clone is running");
    assert_eq!(
        resp.status(),
        200,
        "AC-5 TLS: /dtu/health must return 200 while TLS clone is running"
    );

    // 4. Stop the clone — must complete within 10s (5s drain + 5s overhead).
    timeout(Duration::from_secs(10), clone.stop())
        .await
        .expect("AC-5 TLS: stop() must complete within 10 seconds")
        .expect("AC-5 TLS: stop() must return Ok");

    // 5. Verify the port is released — a new TcpListener must bind successfully.
    prism_dtu_demo_server::harness::test_utils::assert_port_released(
        bound_addr,
        Duration::from_millis(500),
    )
    .await;
}
