//! W3-FIX-SEC-001 — X-Org-Id auth enforcement tests for `prism-dtu-cyberint`.
//!
//! Exercises BC-3.5.001, BC-3.5.002, and BC-3.2.001 per the story acceptance criteria.
//! Cyberint uses `X-Prism-Org-Id` (not `X-Org-Id`) for its org header.
//!
//! # Auth Model: Cyberint uses Auth Model B (multi-org-per-instance routing)
//!
//! Cyberint operates on **auth model B**: the clone supports multiple concurrent orgs.
//! `X-Prism-Org-Id` is a **routing header**, not a security gate.
//!
//! - Missing `X-Prism-Org-Id` → 200 (defaults to `instance_org_id` via fallback).
//!   The session was registered under `(instance_org_id, token)` on login; the request
//!   is also resolved to `instance_org_id` via the same fallback; lookup succeeds.
//! - Mismatched `X-Prism-Org-Id` (valid UUID but no session for that org) → 401
//!   with `{"error": "org_id mismatch: ..."}` body.
//!
//! **AC-003 SUPERSEDED for Cyberint by ARCH-MODEL-B.**
//! Auth model A (single-org-per-instance, `X-Org-Id` is a strict security gate with
//! missing-header → 401) applies to Claroty and CrowdStrike only.  AC-003's original
//! wording ("missing header → 401") does NOT apply here.  The two replacement tests
//! below (`test_AC_003_cyberint_missing_header_returns_default_session_or_400` and
//! `test_AC_003_cyberint_mismatched_header_returns_401_session_not_found`) document the
//! correct model-B semantics.
//!
//! # Acceptance Criteria covered
//!
//! | AC | Description |
//! |----|-------------|
//! | AC-001 | Same-org request succeeds (BC-3.2.001 postcondition 1) |
//! | AC-002 | Cross-org spoofing returns HTTP 401 (BC-3.5.002 precondition 3) |
//! | AC-003 | Auth model B: missing header → 200 (default session); mismatch → 401 |
//! | AC-004 | All four DTU clones covered (BC-3.2.001 invariant 1) |
//! | AC-005 | Regression: `test_cross_org_header_rejected` (BC-3.5.002 precondition 3) |
//! | AC-006 | Positive paths in existing tests still pass (BC-3.5.001 postcondition 1) |
//!
//! # Edge cases covered
//!
//! | EC | Description |
//! |----|-------------|
//! | EC-001 | Non-UUID value in X-Prism-Org-Id header → HTTP 401 |
//! | EC-003 | Sentinel UUID sent as header → HTTP 401 |

#![allow(clippy::expect_used, clippy::unwrap_used, non_snake_case)]
#![cfg(feature = "dtu")]

use prism_dtu_common::BehavioralClone;
use prism_dtu_cyberint::CyberintClone;

// ---------------------------------------------------------------------------
// Sentinel UUID — NEVER a valid instance_org_id
// ---------------------------------------------------------------------------

const SENTINEL_UUID: &str = "00000000-0000-7000-8000-000000000000";

// ---------------------------------------------------------------------------
// Test helper: start a clone and return (clone, base_url).
//
// The returned `CyberintClone` exposes `instance_org_id()` so tests can
// build correctly-keyed org headers without relying on hardcoded UUIDs.
// ---------------------------------------------------------------------------

async fn start_clone() -> (CyberintClone, String) {
    let mut clone = CyberintClone::new().expect("CyberintClone::new failed");
    clone.start().await.expect("CyberintClone::start failed");
    let base_url = clone.base_url();
    (clone, base_url)
}

/// Build a reqwest Client with cookie store and short timeout for testing.
fn http_client() -> reqwest::Client {
    reqwest::Client::builder()
        .cookie_store(true)
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .expect("reqwest::Client build")
}

/// Log in to a Cyberint clone and return the session cookie string.
///
/// The login endpoint uses `extract_org_id` with instance fallback, so it
/// succeeds without an explicit X-Prism-Org-Id header.  The session is
/// registered under `(instance_org_id, token)`.
async fn login(base_url: &str, client: &reqwest::Client) -> String {
    let resp = client
        .post(format!("{base_url}/login"))
        .json(&serde_json::json!({}))
        .send()
        .await
        .expect("login: request must not error");

    assert_eq!(
        resp.status().as_u16(),
        200,
        "login: POST /login must return HTTP 200; got {}",
        resp.status().as_u16()
    );

    // Extract Set-Cookie header value.
    let cookie_header = resp
        .headers()
        .get("set-cookie")
        .expect("login: Set-Cookie header must be present")
        .to_str()
        .expect("login: Set-Cookie must be ASCII")
        .to_owned();

    // Extract just the cookie name=value pair.
    cookie_header
        .split(';')
        .next()
        .expect("login: Set-Cookie must have at least one segment")
        .trim()
        .to_owned()
}

// ---------------------------------------------------------------------------
// Deterministic "wrong" OrgId — guaranteed different from any freshly minted
// instance_org_id since UUIDs are unique.
// ---------------------------------------------------------------------------

fn foreign_org_uuid() -> String {
    "00000000-0000-7000-8000-0000000000BB".to_owned()
}

// ===========================================================================
// AC-001 — Same-org request succeeds (BC-3.2.001 postcondition 1)
// ===========================================================================

/// AC-001 / BC-3.2.001 postcondition 1:
/// A request supplying `X-Prism-Org-Id: <instance_org_id>` receives HTTP 200
/// from `GET /api/v1/alerts`.
///
/// Uses `clone.instance_org_id()` to obtain the actual UUID — no hardcoded
/// placeholder needed (W3-FIX-SEC-001 Round 3 fix).
///
/// Traces to: BC-3.2.001 postcondition 1, W3-FIX-SEC-001 AC-001.
#[tokio::test]
async fn test_AC_001_x_org_id_validated_against_bearer_token() {
    let (clone, base_url) = start_clone().await;
    let instance_org_id = clone.instance_org_id();
    let client = http_client();

    // Login without an explicit org header — session is registered under instance_org_id.
    let cookie = login(&base_url, &client).await;

    // Supply the actual instance_org_id in the header.
    let resp = client
        .get(format!("{base_url}/api/v1/alerts"))
        .header("Cookie", &cookie)
        .header("X-Prism-Org-Id", instance_org_id.as_uuid().to_string())
        .send()
        .await
        .expect("AC-001: request must not error");

    assert_eq!(
        resp.status().as_u16(),
        200,
        "AC-001: GET /api/v1/alerts with matching X-Prism-Org-Id must return HTTP 200; \
         got {}",
        resp.status().as_u16()
    );
}

// ===========================================================================
// AC-002 — Cross-org spoofing returns 401 (BC-3.5.002 precondition 3)
// ===========================================================================

/// AC-002 / BC-3.5.002 precondition 3:
/// A request supplying a different org's UUID in `X-Prism-Org-Id` receives HTTP 401
/// with JSON body `{"error": "org_id mismatch: request does not match this clone instance"}`.
///
/// Traces to: BC-3.5.002 precondition 3, W3-FIX-SEC-001 AC-002.
#[tokio::test]
async fn test_AC_002_cross_org_credential_returns_401() {
    let (_clone, base_url) = start_clone().await;
    let client = http_client();
    let cookie = login(&base_url, &client).await;

    // Send a foreign org UUID in X-Prism-Org-Id — session was registered under
    // instance_org_id, so lookup under this foreign org returns "not found" → 401.
    let resp = client
        .get(format!("{base_url}/api/v1/alerts"))
        .header("Cookie", &cookie)
        .header("X-Prism-Org-Id", foreign_org_uuid())
        .send()
        .await
        .expect("AC-002: request must not error");

    assert_eq!(
        resp.status().as_u16(),
        401,
        "AC-002: GET /api/v1/alerts with mismatched X-Prism-Org-Id must return HTTP 401; \
         got {}",
        resp.status().as_u16()
    );
}

/// AC-002 variant — JSON error body has expected shape.
///
/// The 401 response body MUST be `{"error": "org_id mismatch: ..."}` (not plain text).
/// Traces to: W3-FIX-SEC-001 AC-002, Architecture Compliance Rule §3.
#[tokio::test]
async fn test_AC_002_cross_org_401_body_is_json_error_object() {
    let (_clone, base_url) = start_clone().await;
    let client = http_client();
    let cookie = login(&base_url, &client).await;

    let resp = client
        .get(format!("{base_url}/api/v1/alerts"))
        .header("Cookie", &cookie)
        .header("X-Prism-Org-Id", foreign_org_uuid())
        .send()
        .await
        .expect("AC-002 body: request must not error");

    assert_eq!(
        resp.status().as_u16(),
        401,
        "AC-002 body: expected HTTP 401 for cross-org header; got {}",
        resp.status().as_u16()
    );

    let body: serde_json::Value = resp
        .json()
        .await
        .expect("AC-002 body: 401 response must be valid JSON");

    let error_msg = body["error"].as_str().unwrap_or("");
    assert!(
        error_msg.contains("org_id mismatch"),
        "AC-002 body: error field must contain 'org_id mismatch'; got: {error_msg:?}"
    );
}

// ===========================================================================
// AC-003 — Auth model B: missing header uses default session (not 401)
//
// AC-003 SUPERSEDED for Cyberint by ARCH-MODEL-B.
//
// Cyberint operates on auth model B (multi-org-per-instance, X-Prism-Org-Id is
// a routing header, not a security gate).  Auth model A (single-org-per-instance,
// X-Org-Id is a strict security gate, missing header → 401) applies to Claroty
// and CrowdStrike ONLY.
//
// When X-Prism-Org-Id is absent:
// - extract_org_id falls back to state.instance_org_id
// - Session was registered under (instance_org_id, token) at login
// - is_valid_session(instance_org_id, token) → true
// - Returns HTTP 200 (default session path)
//
// When X-Prism-Org-Id is present but references a foreign org with no session:
// - extract_org_id returns the foreign OrgId
// - is_valid_session(foreign_org_id, token) → false
// - x-prism-org-id header IS present → returns 401 with "org_id mismatch"
// ===========================================================================

/// AC-003 (Cyberint model B) — missing X-Prism-Org-Id header defaults to
/// the instance's own session and returns HTTP 200.
///
/// Cyberint's `extract_org_id` falls back to `state.instance_org_id` when no
/// header is present. The session was registered under `(instance_org_id, token)`
/// at login, so the lookup succeeds.
///
/// This replaces the original AC-003 test that expected 401 for missing header.
/// That expectation applies to auth model A (Claroty/CrowdStrike) only.
///
/// Traces to: W3-FIX-SEC-001 AC-003 (Cyberint model B supersession), ARCH-MODEL-B.
#[tokio::test]
async fn test_AC_003_cyberint_missing_header_returns_default_session_or_400() {
    let (_clone, base_url) = start_clone().await;
    let client = http_client();

    // Login without any org header — session registered under instance_org_id.
    let cookie = login(&base_url, &client).await;

    // Request also without any X-Prism-Org-Id header.
    // extract_org_id falls back to instance_org_id; session lookup succeeds → 200.
    let resp = client
        .get(format!("{base_url}/api/v1/alerts"))
        .header("Cookie", &cookie)
        .send()
        .await
        .expect("AC-003: request must not error");

    assert_eq!(
        resp.status().as_u16(),
        200,
        "AC-003 (model B): GET /api/v1/alerts without X-Prism-Org-Id must return HTTP 200 \
         via instance fallback; got {} — Cyberint uses auth model B (routing header, not \
         security gate); missing header defaults to instance_org_id session",
        resp.status().as_u16()
    );
}

/// AC-003 (Cyberint model B) — mismatched X-Prism-Org-Id (session not found for
/// that org) returns HTTP 401 with `{"error": "org_id mismatch: ..."}`.
///
/// When X-Prism-Org-Id is present but references an org for which no session
/// exists on this clone, `is_valid_session` returns false and the response is
/// 401 with the "org_id mismatch" body (because the header IS present).
///
/// Traces to: W3-FIX-SEC-001 AC-003 (Cyberint model B), BC-3.5.002 precondition 3.
#[tokio::test]
async fn test_AC_003_cyberint_mismatched_header_returns_401_session_not_found() {
    let (_clone, base_url) = start_clone().await;
    let client = http_client();

    // Login without org header — session registered under instance_org_id.
    let cookie = login(&base_url, &client).await;

    // Now send a valid UUID belonging to a DIFFERENT org.
    // No session exists for this org → is_valid_session returns false.
    // Because x-prism-org-id IS present, check_auth returns "org_id mismatch" 401.
    let resp = client
        .get(format!("{base_url}/api/v1/alerts"))
        .header("Cookie", &cookie)
        .header("X-Prism-Org-Id", foreign_org_uuid())
        .send()
        .await
        .expect("AC-003 mismatch: request must not error");

    assert_eq!(
        resp.status().as_u16(),
        401,
        "AC-003 (model B mismatch): GET /api/v1/alerts with foreign X-Prism-Org-Id \
         must return HTTP 401; got {} — no session exists for the foreign org",
        resp.status().as_u16()
    );

    let body: serde_json::Value = resp
        .json()
        .await
        .expect("AC-003 mismatch: 401 response must be valid JSON");

    let error_msg = body["error"].as_str().unwrap_or("");
    assert!(
        error_msg.contains("org_id mismatch"),
        "AC-003 mismatch: error field must contain 'org_id mismatch'; got: {error_msg:?}"
    );
}

// ===========================================================================
// AC-005 — Regression: test_cross_org_header_rejected (BC-3.5.002 precondition 3)
// ===========================================================================

/// AC-005 / BC-3.5.002 precondition 3:
/// Integration test demonstrating credential-mismatch returns HTTP 401.
/// Verifies it is NOT HTTP 200 and NOT a silent empty response.
///
/// Traces to: BC-3.5.002 precondition 3, W3-FIX-SEC-001 AC-005.
#[tokio::test]
async fn test_cross_org_header_rejected() {
    let (_clone, base_url) = start_clone().await;
    let client = http_client();
    let cookie = login(&base_url, &client).await;

    let resp = client
        .get(format!("{base_url}/api/v1/alerts"))
        .header("Cookie", &cookie)
        .header("X-Prism-Org-Id", foreign_org_uuid())
        .send()
        .await
        .expect("AC-005: request must not error");

    let status = resp.status().as_u16();

    assert_eq!(
        status, 401,
        "AC-005: cross-org credential mismatch must return HTTP 401, not {status}"
    );

    let body_bytes = resp
        .bytes()
        .await
        .expect("AC-005: response body must be readable");
    assert!(
        !body_bytes.is_empty(),
        "AC-005: 401 response body must not be empty (silent rejection is not allowed)"
    );
}

// ===========================================================================
// EC-001 — Non-UUID value in X-Prism-Org-Id → 401
// ===========================================================================

/// EC-001:
/// When `X-Prism-Org-Id` is present but not a valid UUID string, the handler must
/// return HTTP 401 with `{"error": "org_id mismatch: ..."}`.
///
/// Traces to: W3-FIX-SEC-001 EC-001.
#[tokio::test]
async fn test_EC_001_non_uuid_x_org_id_returns_401() {
    let (_clone, base_url) = start_clone().await;
    let client = http_client();
    let cookie = login(&base_url, &client).await;

    let resp = client
        .get(format!("{base_url}/api/v1/alerts"))
        .header("Cookie", &cookie)
        .header("X-Prism-Org-Id", "not-a-uuid-at-all")
        .send()
        .await
        .expect("EC-001: request must not error");

    assert_eq!(
        resp.status().as_u16(),
        401,
        "EC-001: non-UUID X-Prism-Org-Id header must return HTTP 401; \
         got {} — get_alerts rejects non-UUID values before check_auth",
        resp.status().as_u16()
    );
}

// ===========================================================================
// EC-003 — Sentinel UUID sent as header → 401
// ===========================================================================

/// EC-003:
/// Sending the sentinel UUID `00000000-0000-7000-8000-000000000000` as the
/// `X-Prism-Org-Id` header must return HTTP 401.
///
/// The sentinel is a valid UUID but cannot match any real `instance_org_id`.
/// The session was registered under `(instance_org_id, token)`, not under the
/// sentinel OrgId, so `is_valid_session` returns false → 401.
///
/// Traces to: W3-FIX-SEC-001 EC-003.
#[tokio::test]
async fn test_EC_003_sentinel_uuid_as_x_org_id_returns_401() {
    let (_clone, base_url) = start_clone().await;
    let client = http_client();
    let cookie = login(&base_url, &client).await;

    let resp = client
        .get(format!("{base_url}/api/v1/alerts"))
        .header("Cookie", &cookie)
        .header("X-Prism-Org-Id", SENTINEL_UUID)
        .send()
        .await
        .expect("EC-003: request must not error");

    assert_eq!(
        resp.status().as_u16(),
        401,
        "EC-003: sentinel UUID in X-Prism-Org-Id must return HTTP 401; \
         got {} — no session exists under the sentinel OrgId",
        resp.status().as_u16()
    );
}
