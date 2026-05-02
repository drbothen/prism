//! SEC-P3-003 test suite — constant-time `X-Admin-Token` comparison.
//!
//! Representative test crate: prism-dtu-claroty.
//! The same pattern applies to all four DTU clone crates:
//!   prism-dtu-armis, prism-dtu-claroty, prism-dtu-crowdstrike, prism-dtu-slack.
//!
//! Covers:
//!   BC-3.5.002 precondition 6: each clone's auth middleware initialized with
//!     own `admin_token`; comparison must not leak timing information.
//!   BC-3.5.001 invariant 3: failure injection state scoped to target clone;
//!     the admin-token gate enforces per-clone identity for this invariant.
//!   AC-001 (SEC-P3-003): correct token → HTTP 200 (behavioral contract
//!     unchanged after constant-time refactor).
//!   AC-002 (SEC-P3-003): wrong token → HTTP 401 (behavioral contract
//!     unchanged after constant-time refactor).
//!
//! ## What this story hardens (not what it changes)
//!
//! The OBSERVABLE behavior of the admin-token gate is IDENTICAL before and
//! after the SEC-P3-003 fix: correct token → 200, wrong/absent token → 401.
//! What changes is the IMPLEMENTATION: the short-circuit `!=` comparison is
//! replaced with `subtle::ConstantTimeEq::ct_eq`, which does not branch early
//! on the first differing byte.
//!
//! ## Timing test rationale
//!
//! A deterministic timing test is intentionally omitted.  The `subtle` crate's
//! `ct_eq` primitive provides the constant-time guarantee at the assembly level
//! via a `black_box` barrier; it is not measurable with wall-clock timing in a
//! multi-process test environment (OS scheduler jitter dominates).  The correct
//! verification method is code inspection: confirm `subtle::ConstantTimeEq` is
//! used in the comparison (rather than `!=`).  The functional tests below
//! verify that the observable HTTP contract is preserved after the refactor.
//!
//! ## Red Gate
//!
//! Both `test_AC_001_*` and `test_AC_002_*` test the SAME HTTP behavior that
//! was already implemented in W3-FIX-SEC-002.  They will PASS at the Red Gate
//! because the behavior is already correct.
//!
//! They serve as a binding specification: after the constant-time refactor, the
//! same tests must continue to pass.  If the implementer accidentally breaks the
//! behavioral contract (e.g., inverts the comparison or uses a wrong method
//! call), these tests will catch it.
//!
//! A compile-time check (presence of `use subtle::ConstantTimeEq` in the
//! production code) is the primary gate for this story's implementation
//! requirement; the behavioral tests below are the runtime regression guard.
//!
//! Test naming: `test_AC_001_*`, `test_AC_002_*` per story AC IDs.
#![allow(non_snake_case, clippy::expect_used, clippy::unwrap_used)]
#![cfg(feature = "dtu")]

use prism_dtu_claroty::ClarotyClone;
use prism_dtu_common::BehavioralClone;

// ===========================================================================
// AC-001: correct admin token → HTTP 200 (constant-time path)
// ===========================================================================

/// BC-3.5.002 precondition 6 / BC-3.5.001 invariant 3 / AC-001 (SEC-P3-003):
///
/// `POST /dtu/reset` with the clone's own `X-Admin-Token` must return HTTP 200
/// `{"status": "ok"}` — the observable behavior is unchanged after replacing
/// the short-circuit `!=` with `subtle::ct_eq`.
///
/// ## Why this is a Red Gate pass (not a fail)
///
/// The behavioral contract for a correct token has been fulfilled since
/// W3-FIX-SEC-002.  This test documents the contract explicitly in terms of
/// SEC-P3-003 so that if the constant-time refactor accidentally inverts the
/// comparison polarity or introduces a new code path, this assertion catches it.
#[tokio::test]
async fn test_AC_001_constant_time_compare_correct_token_returns_200() {
    let mut clone = ClarotyClone::new();
    clone
        .start()
        .await
        .expect("SEC-P3-003 claroty: start() must succeed");

    let token = clone.admin_token().to_string();

    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{}/dtu/reset", clone.base_url()))
        .header("X-Admin-Token", &token)
        .send()
        .await
        .expect("request must complete");

    assert_eq!(
        resp.status().as_u16(),
        200,
        "AC-001 (SEC-P3-003): POST /dtu/reset with correct X-Admin-Token must \
         return 200 after constant-time refactor"
    );

    let body: serde_json::Value = resp.json().await.expect("body must be valid JSON");
    assert_eq!(
        body["status"].as_str().unwrap_or(""),
        "ok",
        "AC-001 (SEC-P3-003): 200 response body must contain {{\"status\": \"ok\"}}"
    );
}

/// BC-3.5.002 precondition 6 / AC-001 (SEC-P3-003):
///
/// `POST /dtu/configure` with the clone's own `X-Admin-Token` must return HTTP 200.
///
/// Per the story's Architecture Compliance Rules, the constant-time refactor
/// MUST be applied to BOTH `dtu_reset` AND `dtu_configure` handlers.  This
/// test verifies the configure endpoint is also covered.
#[tokio::test]
async fn test_AC_001_configure_correct_token_returns_200() {
    let mut clone = ClarotyClone::new();
    clone
        .start()
        .await
        .expect("SEC-P3-003 claroty configure: start() must succeed");

    let token = clone.admin_token().to_string();

    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{}/dtu/configure", clone.base_url()))
        .header("X-Admin-Token", &token)
        .header("Content-Type", "application/json")
        // Send a no-op latency-only configure payload.
        .body(r#"{"latency_ms": 0}"#)
        .send()
        .await
        .expect("configure request must complete");

    assert_eq!(
        resp.status().as_u16(),
        200,
        "AC-001 (SEC-P3-003): POST /dtu/configure with correct X-Admin-Token must \
         return 200 after constant-time refactor"
    );
}

// ===========================================================================
// AC-002: wrong admin token → HTTP 401 (constant-time path)
// ===========================================================================

/// BC-3.5.002 precondition 6 / BC-3.5.001 invariant 3 / AC-002 (SEC-P3-003):
///
/// `POST /dtu/reset` with a wrong `X-Admin-Token` must return HTTP 401 with
/// `{"error": "missing or invalid admin token"}` — behavioral contract
/// unchanged after constant-time refactor.
///
/// ## Production concern (constant-time property)
///
/// With short-circuit `!=` (pre-fix): the comparison branches on the first
/// differing byte, giving an attacker with sub-microsecond timing resolution
/// the ability to enumerate the token byte-by-byte.  The admin token is a
/// UUID v4 (122 bits of entropy) so practical exploitation requires ~122 ×
/// average_time_per_probe.  This is a LOCAL attack vector (the test server
/// is localhost-only) which is why the severity is LOW rather than HIGH.
///
/// With `subtle::ct_eq` (post-fix): the comparison takes the same time
/// regardless of where the first differing byte appears, closing the oracle.
///
/// The timing property cannot be asserted deterministically in a unit test;
/// the behavioral tests below verify that the refactor does not change the
/// observable HTTP responses.
#[tokio::test]
async fn test_AC_002_constant_time_compare_wrong_token_returns_401() {
    let mut clone = ClarotyClone::new();
    clone
        .start()
        .await
        .expect("SEC-P3-003 claroty: start() must succeed");

    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{}/dtu/reset", clone.base_url()))
        .header("X-Admin-Token", "completely-wrong-token-value")
        .send()
        .await
        .expect("request must complete");

    assert_eq!(
        resp.status().as_u16(),
        401,
        "AC-002 (SEC-P3-003): POST /dtu/reset with wrong X-Admin-Token must \
         return 401 after constant-time refactor"
    );

    let body: serde_json::Value = resp.json().await.expect("body must be valid JSON");
    assert_eq!(
        body["error"].as_str().unwrap_or(""),
        "missing or invalid admin token",
        "AC-002 (SEC-P3-003): 401 body must carry {{\"error\": \"missing or invalid admin token\"}}"
    );
}

/// BC-3.5.002 precondition 6 / AC-002 (SEC-P3-003):
///
/// `POST /dtu/reset` with NO `X-Admin-Token` header must return HTTP 401.
///
/// `provided_bytes` is `"".as_bytes()` (empty slice) when the header is absent.
/// `ct_eq` on slices of different lengths returns `Choice(0)` (false) without
/// leaking timing information (EC-007).
#[tokio::test]
async fn test_AC_002_absent_token_returns_401() {
    let mut clone = ClarotyClone::new();
    clone
        .start()
        .await
        .expect("SEC-P3-003 claroty: start() must succeed");

    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{}/dtu/reset", clone.base_url()))
        // No X-Admin-Token header.
        .send()
        .await
        .expect("request must complete");

    assert_eq!(
        resp.status().as_u16(),
        401,
        "AC-002 (SEC-P3-003 / EC-007): POST /dtu/reset with absent X-Admin-Token \
         must return 401; ct_eq on empty vs non-empty slice must return false"
    );
}

/// BC-3.5.002 precondition 6 / AC-002 (SEC-P3-003) / EC-006:
///
/// A token with the correct string content but WRONG BYTE LENGTH must return
/// HTTP 401.  `subtle::ct_eq` on slices of different lengths returns `Choice(0)`
/// without leaking which byte first differed (EC-006).
#[tokio::test]
async fn test_AC_002_wrong_length_token_returns_401() {
    let mut clone = ClarotyClone::new();
    clone
        .start()
        .await
        .expect("SEC-P3-003 claroty wrong-length: start() must succeed");

    let correct_token = clone.admin_token().to_string();
    // Truncate to one less byte: same prefix, wrong length.
    let truncated = &correct_token[..correct_token.len().saturating_sub(1)];

    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{}/dtu/reset", clone.base_url()))
        .header("X-Admin-Token", truncated)
        .send()
        .await
        .expect("request must complete");

    assert_eq!(
        resp.status().as_u16(),
        401,
        "AC-002 (SEC-P3-003 / EC-006): POST /dtu/reset with a truncated token \
         (correct prefix, wrong length) must return 401"
    );
}

/// BC-3.5.002 precondition 6 / AC-002 (SEC-P3-003):
///
/// `POST /dtu/configure` with a wrong token must return HTTP 401.
///
/// Mirrors AC-002 for `dtu_reset` and verifies the configure handler is also
/// hardened by the constant-time refactor (story Architecture Compliance Rule).
#[tokio::test]
async fn test_AC_002_configure_wrong_token_returns_401() {
    let mut clone = ClarotyClone::new();
    clone
        .start()
        .await
        .expect("SEC-P3-003 claroty configure: start() must succeed");

    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{}/dtu/configure", clone.base_url()))
        .header("X-Admin-Token", "wrong-token-configure")
        .header("Content-Type", "application/json")
        .body(r#"{"latency_ms": 0}"#)
        .send()
        .await
        .expect("configure request must complete");

    assert_eq!(
        resp.status().as_u16(),
        401,
        "AC-002 (SEC-P3-003): POST /dtu/configure with wrong X-Admin-Token must \
         return 401 after constant-time refactor"
    );
}

// ===========================================================================
// EC-008: cross-clone token isolation preserved after constant-time refactor
// ===========================================================================

/// BC-3.5.002 precondition 6 / EC-008 (SEC-P3-003):
///
/// Token from clone B presented to clone A must return HTTP 401 after the
/// constant-time refactor.  This ensures per-clone token isolation (BC-3.5.001
/// invariant 3) is not weakened by the implementation change.
///
/// This is a behavioral regression test — the cross-clone rejection was
/// enforced before the refactor and must continue to be enforced after.
#[tokio::test]
async fn test_AC_002_cross_clone_constant_time_token_returns_401() {
    let mut clone_a = ClarotyClone::new();
    clone_a
        .start()
        .await
        .expect("SEC-P3-003 claroty clone-A: start() must succeed");

    let mut clone_b = ClarotyClone::new();
    clone_b
        .start()
        .await
        .expect("SEC-P3-003 claroty clone-B: start() must succeed");

    let token_b = clone_b.admin_token().to_string();

    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{}/dtu/reset", clone_a.base_url()))
        .header("X-Admin-Token", &token_b)
        .send()
        .await
        .expect("request must complete");

    assert_eq!(
        resp.status().as_u16(),
        401,
        "EC-008 (SEC-P3-003): clone-B token on clone-A /dtu/reset must return 401 \
         after constant-time refactor; per-clone token isolation must be preserved"
    );
}
