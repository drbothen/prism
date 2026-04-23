//! AC-12: continue_on_error=true — when a clone fails to bind, the harness logs WARN,
//! skips that clone (excludes it from the URL table), and continues starting the rest.
//!
//! Contrasts with AC-11: when continue_on_error=false (default), the same failure
//! triggers abort-and-cleanup.
//!
//! Expected Red Gate failure: `DemoHarness::start_all()` panics with `todo!()`.

mod common;

use prism_dtu_demo_server::config::{CloneConfig, ClonesConfig, DemoConfig};
use prism_dtu_demo_server::harness::{build_clone_pairs, DemoHarness};

/// Pre-bind a port for forcing EADDRINUSE.
async fn pre_bind_port() -> (tokio::net::TcpListener, u16) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("AC-12: failed to pre-bind");
    let port = listener.local_addr().unwrap().port();
    (listener, port)
}

/// AC-12: continue_on_error=true allows harness to skip the failed clone and proceed.
#[tokio::test]
async fn ac_12_continue_on_error_skips_failed_clone_and_starts_rest() {
    let (_held, blocked_port) = pre_bind_port().await;

    let config = DemoConfig {
        harness: Default::default(),
        clones: ClonesConfig {
            crowdstrike: CloneConfig {
                enabled: true,
                port: 0,
                continue_on_error: true, // continue mode
                ..Default::default()
            },
            claroty: CloneConfig {
                enabled: true,
                port: blocked_port,      // will fail to bind
                continue_on_error: true, // skip, not abort
                ..Default::default()
            },
            cyberint: CloneConfig {
                enabled: true,
                port: 0,
                continue_on_error: true,
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

    let pairs = build_clone_pairs(&config).expect("AC-12: build_clone_pairs must succeed");
    let mut harness = DemoHarness::new(pairs);

    // Expected failure: start_all() panics with "not yet implemented".
    // When implemented: must return Ok (continue_on_error=true means no abort).
    let result = harness.start_all(&config).await;

    assert!(
        result.is_ok(),
        "AC-12: start_all with continue_on_error=true must return Ok even when a clone fails; got: {:?}",
        result.err()
    );

    let report = harness.last_start_report();

    // Under continue_on_error=true: failed_at is None, no rollback.
    assert!(
        report.failed_at.is_none(),
        "AC-12: failed_at must be None under continue_on_error=true"
    );

    assert!(
        report.cleaned_up_after_failure.is_empty(),
        "AC-12: cleaned_up_after_failure must be empty (no rollback in continue mode)"
    );

    // Crowdstrike and cyberint should have started; claroty should be in skipped list.
    assert!(
        report
            .successfully_started
            .contains(&"crowdstrike".to_string()),
        "AC-12: crowdstrike must be in successfully_started"
    );
    assert!(
        report
            .successfully_started
            .contains(&"cyberint".to_string()),
        "AC-12: cyberint must be in successfully_started"
    );

    // Claroty failed and was skipped.
    assert!(
        !report.successfully_started.contains(&"claroty".to_string()),
        "AC-12: claroty must NOT be in successfully_started (it failed)"
    );

    harness.stop_all().await;
}

/// AC-12: With continue_on_error=false (default), the same failure aborts (AC-11 path).
#[tokio::test]
async fn ac_12_default_continue_on_error_false_triggers_abort() {
    let (_held, blocked_port) = pre_bind_port().await;

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
                port: blocked_port,
                continue_on_error: false, // abort on failure
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

    let pairs = build_clone_pairs(&config).expect("AC-12: build_clone_pairs must succeed");
    let mut harness = DemoHarness::new(pairs);

    // Expected failure: start_all() panics with "not yet implemented".
    // When implemented: must return Err (abort path with continue_on_error=false).
    let result = harness.start_all(&config).await;

    assert!(
        result.is_err(),
        "AC-12: start_all with continue_on_error=false must return Err when a clone fails to bind"
    );
}
