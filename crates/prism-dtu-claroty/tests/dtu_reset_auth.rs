//! W3-FIX-SEC-002: `POST /dtu/reset` admin token auth — prism-dtu-claroty
//!
//! Acceptance criteria:
//!   AC-001 (BC-3.2.001 inv-1): reset without token → 401
//!   AC-002 (BC-3.5.001 post-3): reset with correct token → 200
//!   AC-003 (BC-3.5.002 pre-3): cross-clone token → 401
//!
//! RED GATE: all three tests must panic with `todo!` until the handler
//! is implemented.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use prism_dtu_claroty::ClarotyClone;
use prism_dtu_common::BehavioralClone;

// ---------------------------------------------------------------------------
// AC-001: missing token → HTTP 401
// ---------------------------------------------------------------------------

/// BC-3.2.001 invariant 1: unauthenticated reset must be rejected.
///
/// `POST /dtu/reset` with no `X-Admin-Token` header must return HTTP 401
/// and must not clear state.
#[tokio::test]
async fn test_AC_001_dtu_reset_without_admin_token_returns_401() {
    let mut clone = ClarotyClone::new();
    clone
        .start()
        .await
        .expect("W3-FIX-SEC-002 claroty: start() must succeed");

    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{}/dtu/reset", clone.base_url()))
        .send()
        .await
        .expect("request must complete");

    assert_eq!(
        resp.status().as_u16(),
        401,
        "AC-001: POST /dtu/reset without X-Admin-Token must return 401"
    );

    let body: serde_json::Value = resp.json().await.expect("body must be JSON");
    assert_eq!(
        body["error"].as_str().unwrap_or(""),
        "missing or invalid admin token",
        "AC-001: 401 body must carry error field"
    );
}

// ---------------------------------------------------------------------------
// AC-002: correct token → HTTP 200
// ---------------------------------------------------------------------------

/// BC-3.5.001 postcondition 3: authorised reset must succeed and clear state.
///
/// `POST /dtu/reset` with the clone's own `X-Admin-Token` must return HTTP 200
/// `{"status": "ok"}` and clear state.
#[tokio::test]
async fn test_AC_002_dtu_reset_with_admin_token_returns_200() {
    let mut clone = ClarotyClone::new();
    clone
        .start()
        .await
        .expect("W3-FIX-SEC-002 claroty: start() must succeed");
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
        "AC-002: POST /dtu/reset with correct X-Admin-Token must return 200"
    );
}

// ---------------------------------------------------------------------------
// AC-003: cross-clone (wrong) token → HTTP 401
// ---------------------------------------------------------------------------

/// BC-3.5.002 precondition 3: admin token is per-clone; another clone's token
/// must be rejected.
///
/// Spin up two independent `ClarotyClone` instances.  Token from clone B
/// presented to clone A must return HTTP 401.  Clone A state must not be
/// cleared.
#[tokio::test]
async fn test_AC_003_cross_clone_admin_token_returns_401() {
    let mut clone_a = ClarotyClone::new();
    clone_a
        .start()
        .await
        .expect("W3-FIX-SEC-002 claroty clone-A: start() must succeed");

    let mut clone_b = ClarotyClone::new();
    clone_b
        .start()
        .await
        .expect("W3-FIX-SEC-002 claroty clone-B: start() must succeed");

    // Tokens from distinct instances must differ (UUID v4).
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
        "AC-003: clone-B token on clone-A /dtu/reset must return 401"
    );
}
