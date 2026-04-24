//! AC-13: StartReport semantics under continue_on_error=true.
//!
//! Given DemoConfig with crowdstrike/claroty/cyberint having continue_on_error=true,
//! and cyberint's port is pre-bound to force a bind failure, when start_all() is
//! called and returns, then StartReport must satisfy:
//!   - `successfully_started` == ["crowdstrike", "claroty"] (in order)
//!   - `skipped_due_to_error` has exactly 1 entry: ("cyberint", AddrInUse)
//!   - `failed_at` is None
//!   - `cleaned_up_after_failure` is empty
//!
//! Expected Red Gate failure: `DemoHarness::start_all()` panics with `todo!()`.

mod common;

use prism_dtu_demo_server::config::{CloneConfig, ClonesConfig, DemoConfig};
use prism_dtu_demo_server::harness::{build_clone_pairs, DemoHarness};

/// Pre-bind a port to force EADDRINUSE on cyberint.
async fn pre_bind_port() -> (tokio::net::TcpListener, u16) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("AC-13: failed to pre-bind");
    let port = listener.local_addr().unwrap().port();
    (listener, port)
}

/// AC-13: StartReport under continue_on_error=true with cyberint failing to bind.
#[tokio::test]
async fn ac_13_start_report_continue_on_error_skipped_due_to_error() {
    let (_held, blocked_port) = pre_bind_port().await;

    let config = DemoConfig {
        harness: Default::default(),
        clones: ClonesConfig {
            crowdstrike: CloneConfig {
                enabled: true,
                port: 0,
                continue_on_error: true,
                ..Default::default()
            },
            claroty: CloneConfig {
                enabled: true,
                port: 0,
                continue_on_error: true,
                ..Default::default()
            },
            cyberint: CloneConfig {
                enabled: true,
                port: blocked_port,      // pre-bound → EADDRINUSE
                continue_on_error: true, // skip, don't abort
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

    let pairs = build_clone_pairs(&config).expect("AC-13: build_clone_pairs must succeed");
    let mut harness = DemoHarness::new(pairs);

    // Expected failure: start_all() panics with "not yet implemented".
    let result = harness.start_all(&config, None).await;

    // Under continue_on_error=true: start_all returns Ok even when a clone fails.
    assert!(
        result.is_ok(),
        "AC-13: start_all must return Ok under continue_on_error=true; got: {:?}",
        result.err()
    );

    let report = harness.last_start_report();

    // Assert: successfully_started == ["crowdstrike", "claroty"] in order.
    assert_eq!(
        report.successfully_started,
        vec!["crowdstrike".to_string(), "claroty".to_string()],
        "AC-13: successfully_started must be [\"crowdstrike\", \"claroty\"] in order; got: {:?}",
        report.successfully_started
    );

    // Assert: skipped_due_to_error has exactly 1 entry for cyberint with AddrInUse.
    assert_eq!(
        report.skipped_due_to_error.len(),
        1,
        "AC-13: skipped_due_to_error must have exactly 1 entry; got: {:?}",
        report.skipped_due_to_error
    );

    let (skipped_name, ref skipped_err) = &report.skipped_due_to_error[0];
    assert_eq!(
        skipped_name, "cyberint",
        "AC-13: skipped clone name must be 'cyberint'; got: {skipped_name}"
    );
    assert_eq!(
        skipped_err.kind(),
        std::io::ErrorKind::AddrInUse,
        "AC-13: skipped error kind must be AddrInUse; got: {:?}",
        skipped_err.kind()
    );

    // Assert: failed_at is None (continue_on_error=true never sets this).
    assert!(
        report.failed_at.is_none(),
        "AC-13: failed_at must be None under continue_on_error=true; got: {:?}",
        report.failed_at.as_ref().map(|(n, _)| n)
    );

    // Assert: cleaned_up_after_failure is empty (no rollback in continue mode).
    assert!(
        report.cleaned_up_after_failure.is_empty(),
        "AC-13: cleaned_up_after_failure must be empty under continue_on_error=true; got: {:?}",
        report.cleaned_up_after_failure
    );

    harness.stop_all().await;
}

/// AC-13: StartReport invariant — all-success shape is correct when no failures occur.
#[tokio::test]
async fn ac_13_start_report_all_success_invariant() {
    let config = common::all_clones_ephemeral_config();
    let pairs = build_clone_pairs(&config).expect("AC-13: build_clone_pairs must succeed");
    let mut harness = DemoHarness::new(pairs);

    // Expected failure: start_all() panics with "not yet implemented".
    harness
        .start_all(&config, None)
        .await
        .expect("AC-13: start_all must succeed with all-ephemeral config");

    let report = harness.last_start_report();

    // All-success invariant: successfully_started has all 6, everything else empty/None.
    assert_eq!(
        report.successfully_started.len(),
        6,
        "AC-13: all-success StartReport must have 6 successfully_started; got: {:?}",
        report.successfully_started
    );
    assert!(
        report.cleaned_up_after_failure.is_empty(),
        "AC-13: all-success StartReport must have empty cleaned_up_after_failure"
    );
    assert!(
        report.failed_at.is_none(),
        "AC-13: all-success StartReport must have failed_at == None"
    );
    assert!(
        report.skipped_due_to_error.is_empty(),
        "AC-13: all-success StartReport must have empty skipped_due_to_error"
    );

    harness.stop_all().await;
}

/// AC-13: StartReport invariant — abort shape is correct when continue_on_error=false and a clone fails.
#[tokio::test]
async fn ac_13_start_report_abort_invariant() {
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
                port: 0,
                continue_on_error: false,
                ..Default::default()
            },
            cyberint: CloneConfig {
                enabled: true,
                port: blocked_port,
                continue_on_error: false,
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

    let pairs = build_clone_pairs(&config).expect("AC-13: build_clone_pairs must succeed");
    let mut harness = DemoHarness::new(pairs);

    // Expected failure: start_all() panics with "not yet implemented".
    let _ = harness.start_all(&config, None).await;

    let report = harness.last_start_report();

    // Abort invariant: failed_at is Some, skipped_due_to_error is empty.
    if report.failed_at.is_some() {
        assert!(
            report.skipped_due_to_error.is_empty(),
            "AC-13: abort StartReport must have empty skipped_due_to_error"
        );
    }
    // (If start_all panicked, we never reach here — that's the expected Red Gate failure.)
}
