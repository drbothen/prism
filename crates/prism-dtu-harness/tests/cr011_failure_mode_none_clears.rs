//! CR-011: `with_failure(slug, dtu_type, FailureMode::None)` removes the entry instead
//! of inserting a sentinel.
//!
//! Covers:
//!   BC-3.6.001 invariant 4 — `FailureMode::None` clears failure; no sentinel entry in
//!                            `initial_failure` HashMap.
//!   EC-001 — `with_failure(None)` on org with no prior failure is a no-op (no entry).
//!   EC-002 — set a failure, then clear with `with_failure(None)`, then build:
//!            no configure call issued; clone returns HTTP 200 for all requests.
//!
//! Verification properties:
//!   VP-128 (failure injection isolation)
//!
//! # Red Gate (pre-fix)
//!
//! Before the CR-011 fix, `with_failure(_, _, FailureMode::None)` called
//! `existing.initial_failure.insert(dtu_type, FailureMode::None)`, inserting a
//! sentinel entry.  With the fix, `FailureMode::None` triggers
//! `existing.initial_failure.remove(&dtu_type)`.
//!
//! These tests exercise the build + HTTP-round-trip to confirm no spurious
//! configure call was issued.  They pass once the CR-011 fix is applied (Green Gate).

#![allow(clippy::expect_used, non_snake_case)]

use prism_dtu_common::FailureMode;
use prism_dtu_harness::{DtuType, IsolationMode};

// ============================================================================
// EC-001: with_failure(None) on org with no prior failure set — no-op
//
// BC-3.6.001 invariant 4; EC-001
// ============================================================================

/// EC-001: `with_failure(slug, dtu_type, FailureMode::None)` when no failure has been
/// set is a no-op — no sentinel is inserted and the clone returns HTTP 200.
///
/// Build a single-Claroty harness for "acme". Call `with_failure(FailureMode::None)`
/// before any other `with_failure` call. Assert the Claroty clone returns HTTP 200.
///
/// (BC-3.6.001 invariant 4; CR-011 EC-001)
#[tokio::test]
async fn test_BC_3_6_001_invariant4_with_failure_none_on_empty_spec_is_noop() {
    let harness = prism_dtu_harness::Harness::builder()
        .isolation(IsolationMode::Logical)
        .with_customer("acme")
        // FailureMode::None on a spec with no existing failure — must not insert anything.
        .with_failure("acme", DtuType::Claroty, FailureMode::None)
        .build()
        .await
        .expect("harness build must succeed after with_failure(None) on empty spec");

    let addr = harness
        .endpoint_for("acme", DtuType::Claroty)
        .expect("acme Claroty endpoint must be present");

    let status = reqwest::get(format!("http://{addr}/assets/v1/assets"))
        .await
        .expect("HTTP GET must succeed")
        .status()
        .as_u16();

    // No failure injected — clone must return 200, not 401/429/500.
    assert_eq!(
        status, 200,
        "with_failure(None) on empty spec must not inject any failure mode; \
         expected HTTP 200 but got {status} (BC-3.6.001 invariant 4; CR-011 EC-001)"
    );
}

// ============================================================================
// EC-002: set failure then clear with with_failure(None) — build returns 200
//
// BC-3.6.001 invariant 4; EC-002
// ============================================================================

/// EC-002: Set `FailureMode::AuthReject`, then clear with `FailureMode::None`.
/// After `build()`, the Claroty clone must return HTTP 200, not 401.
///
/// This verifies that `FailureMode::None` on the immediate-resolution path
/// actually calls `remove(&dtu_type)` rather than inserting a sentinel.
/// If the sentinel were inserted, the configure endpoint would be called with
/// `FailureMode::None`, resulting in undefined behavior or a silent 401.
///
/// (BC-3.6.001 invariant 4; CR-011 EC-002; AC-002)
#[tokio::test]
async fn test_BC_3_6_001_invariant4_with_failure_none_after_set_clears_entry() {
    let harness = prism_dtu_harness::Harness::builder()
        .isolation(IsolationMode::Logical)
        .with_customer("acme")
        // Set a failure first.
        .with_failure("acme", DtuType::Claroty, FailureMode::AuthReject)
        // Then clear it with FailureMode::None — this should remove the entry.
        .with_failure("acme", DtuType::Claroty, FailureMode::None)
        .build()
        .await
        .expect("harness build must succeed after clearing failure with FailureMode::None");

    let addr = harness
        .endpoint_for("acme", DtuType::Claroty)
        .expect("acme Claroty endpoint must be present");

    let status = reqwest::get(format!("http://{addr}/assets/v1/assets"))
        .await
        .expect("HTTP GET must succeed")
        .status()
        .as_u16();

    // Failure was cleared — clone must return 200, not 401.
    assert_eq!(
        status, 200,
        "with_failure(None) after AuthReject must clear the failure mode; \
         expected HTTP 200 but got {status} — a 401 means the sentinel was \
         inserted instead of removed (BC-3.6.001 invariant 4; CR-011 EC-002)"
    );
}

// ============================================================================
// EC-002 (deferred path): pending_failures drain also removes on None
//
// BC-3.6.001 invariant 4; EC-002 deferred variant
// ============================================================================

/// EC-002 (deferred): `with_failure(None)` called BEFORE `with_customer` uses the
/// deferred `pending_failures` path. After `build()`, the clone must return HTTP 200.
///
/// This verifies that the deferred drain in `build()` also applies the
/// `remove`-on-`None` logic.
///
/// (BC-3.6.001 invariant 4; CR-011 EC-002 deferred; story EC-002)
#[tokio::test]
async fn test_BC_3_6_001_invariant4_with_failure_none_on_deferred_path_clears_entry() {
    let harness = prism_dtu_harness::Harness::builder()
        .isolation(IsolationMode::Logical)
        // Note: with_failure called BEFORE with_customer — deferred path.
        // First set a deferred failure, then clear it via a second with_failure(None).
        .with_failure("acme", DtuType::Claroty, FailureMode::AuthReject)
        .with_failure("acme", DtuType::Claroty, FailureMode::None)
        .with_customer("acme")
        .build()
        .await
        .expect("harness build must succeed: deferred FailureMode::None clears entry");

    let addr = harness
        .endpoint_for("acme", DtuType::Claroty)
        .expect("acme Claroty endpoint must be present");

    let status = reqwest::get(format!("http://{addr}/assets/v1/assets"))
        .await
        .expect("HTTP GET must succeed")
        .status()
        .as_u16();

    assert_eq!(
        status, 200,
        "deferred with_failure(None) must clear failure entry during build(); \
         expected HTTP 200 but got {status} (BC-3.6.001 invariant 4; CR-011 deferred)"
    );
}

// ============================================================================
// Invariant: with_failure(None) does not affect other DtuTypes for same org
//
// BC-3.6.001 invariant 1; postcondition 2
// ============================================================================

/// Invariant: Clearing Claroty failure via `FailureMode::None` must NOT affect
/// the Armis failure mode for the same org.
///
/// (BC-3.6.001 invariant 1; postcondition 2; CR-011)
#[tokio::test]
async fn test_BC_3_6_001_invariant4_clearing_one_dtu_does_not_affect_others() {
    let harness = prism_dtu_harness::Harness::builder()
        .isolation(IsolationMode::Logical)
        .with_customer_overrides("acme", |spec| {
            spec.dtu_types = vec![DtuType::Claroty, DtuType::Armis];
        })
        // Set AuthReject on both.
        .with_failure("acme", DtuType::Claroty, FailureMode::AuthReject)
        .with_failure("acme", DtuType::Armis, FailureMode::AuthReject)
        // Clear only Claroty.
        .with_failure("acme", DtuType::Claroty, FailureMode::None)
        .build()
        .await
        .expect("harness build must succeed");

    let claroty_addr = harness
        .endpoint_for("acme", DtuType::Claroty)
        .expect("acme Claroty endpoint must be present");
    let armis_addr = harness
        .endpoint_for("acme", DtuType::Armis)
        .expect("acme Armis endpoint must be present");

    let claroty_status = reqwest::get(format!("http://{claroty_addr}/assets/v1/assets"))
        .await
        .expect("HTTP GET to Claroty must succeed")
        .status()
        .as_u16();

    let armis_status = reqwest::get(format!("http://{armis_addr}/api/v1/devices"))
        .await
        .expect("HTTP GET to Armis must succeed")
        .status()
        .as_u16();

    // Claroty was cleared → 200.
    assert_eq!(
        claroty_status, 200,
        "Claroty failure cleared via FailureMode::None; expected 200 got {claroty_status} \
         (BC-3.6.001 invariant 4; CR-011)"
    );

    // Armis was NOT cleared → still 401.
    assert_eq!(
        armis_status, 401,
        "Armis failure NOT cleared; expected 401 (AuthReject still set) got {armis_status} \
         (BC-3.6.001 invariant 1; CR-011)"
    );
}
