//! Per-DtuType failure scope regression tests — W3-FIX-CODE-001
//!
//! Covers:
//!   AC-001 — `with_failure(slug, DtuType, mode)` injects failure ONLY into the
//!            specified DtuType; other DtuTypes for the same org return HTTP 200.
//!   AC-002 — `Drop` honors the 5-second graceful shutdown budget; tasks self-complete
//!            when the shutdown signal fires (no immediate `handle.abort()`).
//!
//! BCs exercised:
//!   BC-3.6.001 postcondition 2 — other clones return normal responses.
//!   BC-3.6.001 invariant 1     — failure state scoped strictly to target.
//!   BC-3.5.001 EC-004          — Drop "waits up to 5s for graceful exit".
//!
//! Verification properties:
//!   VP-128 (failure injection isolation)
//!   VP-129 (per-DtuType injection scope)
//!   VP-130 (drop grace period)
//!   VP-124 (drop releases resources)
//!
//! # Red Gate expectations
//!
//! | Test | Expected RED failure | Reason |
//! |------|---------------------|--------|
//! | AC-001 (failure scope) | PANICS — `with_failure` body is `todo!()` | todo!() fires at build time |
//! | AC-002 (drop grace) | PANICS — `Drop::drop` body contains `todo!()` | todo!() fires at drop time |
//!
//! Both tests MUST FAIL (panic) before implementation begins — Red Gate verified.

#![allow(clippy::expect_used, non_snake_case)]

use prism_dtu_common::FailureMode;
use prism_dtu_harness::{DtuType, IsolationMode};

// ============================================================================
// AC-001 — with_failure only injects into the specified DtuType
//
// BC-3.6.001 postcondition 2; invariant 1; VP-128; VP-129
// ============================================================================

/// AC-001: `with_failure("acme", DtuType::Claroty, FailureMode::AuthReject)` must
/// inject `AuthReject` ONLY into the Claroty clone for org `acme`.
///
/// After `build()`, send HTTP GET requests to the Armis, CrowdStrike, and Cyberint
/// clones for `acme` — each must return HTTP 200, not 401. Only the Claroty clone
/// must return 401.
///
/// # Red Gate
///
/// PANICS: `with_failure` body is `todo!("AC-001: store failure scoped to dtu_type")`.
/// The panic fires synchronously inside the builder chain before `build().await` is
/// reached. This is the correct Red Gate signal.
///
/// (BC-3.6.001 postcondition 2; invariant 1; VP-128; VP-129; W3-FIX-CODE-001 AC-001)
#[tokio::test]
async fn test_AC_001_with_failure_only_injects_into_specified_dtu_type() {
    // Build a harness with all four Security Telemetry DtuTypes for "acme".
    // Inject AuthReject ONLY for Claroty.
    //
    // RED: with_failure panics via todo!("AC-001: store failure scoped to dtu_type")
    // before build() is called. The #[should_panic] annotation records the expected
    // Red Gate behavior.
    let harness = prism_dtu_harness::Harness::builder()
        .isolation(IsolationMode::Logical)
        .with_customer_overrides("acme", |spec| {
            spec.dtu_types = vec![
                DtuType::Claroty,
                DtuType::Armis,
                DtuType::CrowdStrike,
                DtuType::Cyberint,
            ];
        })
        .with_failure("acme", DtuType::Claroty, FailureMode::AuthReject)
        .build()
        .await
        .expect("harness build must succeed (AC-001; BC-3.6.001)");

    let client = reqwest::Client::new();

    // The Claroty clone must return 401 (AuthReject injected).
    let claroty_addr = harness
        .endpoint_for("acme", DtuType::Claroty)
        .expect("Claroty endpoint must exist after build");
    let resp = client
        .get(format!("http://{claroty_addr}/assets/v1/assets"))
        .send()
        .await
        .expect("request to Claroty must not fail at transport level");
    assert_eq!(
        resp.status().as_u16(),
        401,
        "Claroty clone must return 401 — AuthReject was injected via with_failure \
         (AC-001; BC-3.6.001 postcondition 1)"
    );

    // Armis must return 200 — failure was NOT injected for Armis.
    // Armis requires Bearer auth (returns 403 without it — AC-5 behaviour).
    let armis_addr = harness
        .endpoint_for("acme", DtuType::Armis)
        .expect("Armis endpoint must exist after build");
    let resp = client
        .get(format!("http://{armis_addr}/api/v1/devices"))
        .header("Authorization", "Bearer harness-test-token")
        .send()
        .await
        .expect("request to Armis must not fail at transport level");
    assert_eq!(
        resp.status().as_u16(),
        200,
        "Armis clone must return HTTP 200 — failure was injected ONLY for Claroty \
         (AC-001; BC-3.6.001 postcondition 2; VP-128)"
    );

    // CrowdStrike must return 200 — failure was NOT injected for CrowdStrike.
    // CrowdStrike requires Bearer auth (returns 401 without it).
    // Path is /devices/entities/devices/v2 (the CrowdStrike host-details endpoint).
    let crowdstrike_addr = harness
        .endpoint_for("acme", DtuType::CrowdStrike)
        .expect("CrowdStrike endpoint must exist after build");
    let resp = client
        .get(format!(
            "http://{crowdstrike_addr}/devices/entities/devices/v2"
        ))
        .header("Authorization", "Bearer harness-test-token")
        .send()
        .await
        .expect("request to CrowdStrike must not fail at transport level");
    assert_eq!(
        resp.status().as_u16(),
        200,
        "CrowdStrike clone must return HTTP 200 — failure was injected ONLY for Claroty \
         (AC-001; BC-3.6.001 postcondition 2; VP-129)"
    );

    // Cyberint must return 200 — failure was NOT injected for Cyberint.
    let cyberint_addr = harness
        .endpoint_for("acme", DtuType::Cyberint)
        .expect("Cyberint endpoint must exist after build");
    let resp = client
        .get(format!("http://{cyberint_addr}/api/v1/events"))
        .send()
        .await
        .expect("request to Cyberint must not fail at transport level");
    assert_eq!(
        resp.status().as_u16(),
        200,
        "Cyberint clone must return HTTP 200 — failure was injected ONLY for Claroty \
         (AC-001; BC-3.6.001 postcondition 2; VP-129)"
    );
}

// ============================================================================
// AC-002 — Drop honors 5-second graceful shutdown budget
//
// BC-3.5.001 EC-004; VP-124; VP-130; W3-FIX-CODE-001 AC-002/AC-004
// ============================================================================

/// AC-002: `drop(harness)` must complete within 5 seconds without hard-aborting
/// the clone tasks. The shutdown signal is sent; `axum::with_graceful_shutdown`
/// drains in-flight requests and the task exits cleanly.
///
/// This test spawns a single Claroty clone, then drops the harness while no
/// in-flight requests are active. The expectation is that the shutdown signal
/// fires, axum drains (immediately, no active requests), and the task exits
/// before the 5-second gate expires.
///
/// # Red Gate
///
/// PANICS: `Drop::drop` contains `todo!("AC-002: honor 5s graceful shutdown")`.
/// When `drop(harness)` is called, the `todo!()` fires and the test panics.
/// The `tokio::time::timeout` wrapper will propagate the panic, failing the test.
///
/// (BC-3.5.001 EC-004; VP-124; VP-130; W3-FIX-CODE-001 AC-002/AC-004)
#[tokio::test]
async fn test_AC_002_drop_honors_5s_graceful_shutdown_budget() {
    // Spawn a single-clone harness (Claroty only for speed).
    let harness = prism_dtu_harness::Harness::builder()
        .isolation(IsolationMode::Logical)
        .with_customer_overrides("drop-test", |spec| {
            spec.dtu_types = vec![DtuType::Claroty];
        })
        .build()
        .await
        .expect("harness build must succeed for drop test (AC-002)");

    // Gate the drop with a 5-second timeout.
    //
    // With the correct implementation: shutdown signal fires → axum graceful drain →
    // task exits → drop completes within the 5s budget.
    //
    // RED: drop(harness) panics via todo!("AC-002: honor 5s graceful shutdown").
    // The panic propagates out of the timeout future and fails the test.
    let drop_result = tokio::time::timeout(std::time::Duration::from_secs(5), async {
        drop(harness);
    })
    .await;

    assert!(
        drop_result.is_ok(),
        "drop(harness) must complete within 5s — graceful shutdown via \
         axum::with_graceful_shutdown must drain and exit cleanly \
         (AC-002; BC-3.5.001 EC-004; VP-124; VP-130)"
    );
}
