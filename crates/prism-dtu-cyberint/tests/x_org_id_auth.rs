//! W3-FIX-SEC-001 — X-Org-Id auth enforcement tests for `prism-dtu-cyberint`.
//!
//! Exercises BC-3.5.001, BC-3.5.002, and BC-3.2.001 per the story acceptance criteria.
//! Cyberint uses `X-Prism-Org-Id` (not `X-Org-Id`) for its org header.
//!
//! # Red Gate (Phase 2)
//!
//! Test bodies replaced with real assertion-driven logic. Tests for AC-002, AC-003,
//! EC-001, EC-003, and AC-005 assert HTTP 401 on mismatch/missing/malformed org
//! headers. These tests currently FAIL with "expected 401, got 200" because
//! `validate_org_id` is `todo!()` and not yet wired into Cyberint route handlers.
//!
//! # Implementation note: instance_org_id accessibility
//!
//! `CyberintClone::state` is private. There is no public accessor for `instance_org_id`.
//! The implementation phase must add either a `new_with_org(org_id)` constructor or a
//! public `instance_org_id()` method to `CyberintClone` before `test_AC_001` can make a
//! structurally correct same-org assertion. Until then, `test_AC_001` logs in without an
//! org header (which uses the instance fallback) and verifies the 200 path — an acceptable
//! Red Gate placeholder that becomes strengthened once the accessor lands.
//!
//! # Acceptance Criteria covered
//!
//! | AC | Description |
//! |----|-------------|
//! | AC-001 | Same-org request succeeds (BC-3.2.001 postcondition 1) |
//! | AC-002 | Cross-org spoofing returns HTTP 401 (BC-3.5.002 precondition 3) |
//! | AC-003 | Missing header returns HTTP 401 (BC-3.5.001 postcondition 1) |
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

use prism_core::OrgId;
use prism_dtu_common::BehavioralClone;
use prism_dtu_cyberint::CyberintClone;

// ---------------------------------------------------------------------------
// Sentinel UUID — NEVER a valid instance_org_id
// ---------------------------------------------------------------------------

const SENTINEL_UUID: &str = "00000000-0000-7000-8000-000000000000";

// ---------------------------------------------------------------------------
// Test helper: start a clone and return (clone, base_url).
//
// NOTE: `CyberintClone::state` is private; there is no public accessor for
// `instance_org_id`. The `org_id` parameter is accepted for API symmetry with
// other crate helpers but CANNOT be used to set the clone's internal org.
// The implementation phase must add `CyberintClone::new_with_org(OrgId)` or an
// accessor before this helper can be made fully correct.
// ---------------------------------------------------------------------------

async fn start_clone_with_org(_org_id: OrgId) -> (CyberintClone, String) {
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
/// succeeds without an explicit X-Prism-Org-Id header.
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
// Deterministic "wrong" OrgIds — guaranteed different from any freshly minted
// instance_org_id since UUIDs are unique.
// ---------------------------------------------------------------------------

fn org_wrong() -> OrgId {
    OrgId::from_uuid(
        uuid::Uuid::parse_str("00000000-0000-7000-8000-0000000000BB").expect("valid uuid"),
    )
}

// ===========================================================================
// AC-001 — Same-org request succeeds (BC-3.2.001 postcondition 1)
// ===========================================================================

/// AC-001 / BC-3.2.001 postcondition 1:
/// A request supplying `X-Prism-Org-Id: <instance_org_id>` receives HTTP 200
/// from `GET /api/v1/alerts`.
///
/// Implementation note: `CyberintClone::state` is private; `instance_org_id`
/// cannot be read from the outside. This test logs in WITHOUT an org header
/// (which registers the session under the instance's implicit org via fallback)
/// then makes a request without an org header (same fallback path). This verifies
/// the positive path continues to work. Once `CyberintClone::instance_org_id()`
/// is exposed, this test should be strengthened to use the actual UUID.
///
/// Traces to: BC-3.2.001 postcondition 1, W3-FIX-SEC-001 AC-001.
#[tokio::test]
async fn test_AC_001_x_org_id_validated_against_bearer_token() {
    // Note: org_id parameter is unused because CyberintClone doesn't expose the
    // instance_org_id through its public API yet.
    let instance_org = OrgId::from_uuid(
        uuid::Uuid::parse_str("00000000-0000-7000-8000-0000000000AA").expect("valid uuid"),
    );
    let (_clone, base_url) = start_clone_with_org(instance_org).await;
    let client = http_client();

    // Login without org header — registers session under the instance's implicit org.
    let cookie = login(&base_url, &client).await;

    // Make a request also without an org header — current extract_org_id fallback
    // uses the instance org, so alert store lookup uses the same key. Returns 200.
    let resp = client
        .get(format!("{base_url}/api/v1/alerts"))
        .header("Cookie", &cookie)
        .send()
        .await
        .expect("AC-001: request must not error");

    assert_eq!(
        resp.status().as_u16(),
        200,
        "AC-001: GET /api/v1/alerts with matching org context must return HTTP 200; \
         got {} — this test uses the instance fallback path since instance_org_id \
         is not publicly accessible yet",
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
    let (_clone, base_url) = start_clone_with_org(org_wrong()).await;
    let client = http_client();
    let cookie = login(&base_url, &client).await;

    // Send a wrong org UUID in X-Prism-Org-Id.
    let resp = client
        .get(format!("{base_url}/api/v1/alerts"))
        .header("Cookie", &cookie)
        .header("X-Prism-Org-Id", org_wrong().as_uuid().to_string())
        .send()
        .await
        .expect("AC-002: request must not error");

    assert_eq!(
        resp.status().as_u16(),
        401,
        "AC-002: GET /api/v1/alerts with mismatched X-Prism-Org-Id must return HTTP 401; \
         got {} — validate_org_id is not yet wired into Cyberint get_alerts handler",
        resp.status().as_u16()
    );
}

/// AC-002 variant — JSON error body has expected shape.
///
/// The 401 response body MUST be `{"error": "org_id mismatch: ..."}` (not plain text).
/// Traces to: W3-FIX-SEC-001 AC-002, Architecture Compliance Rule §3.
#[tokio::test]
async fn test_AC_002_cross_org_401_body_is_json_error_object() {
    let (_clone, base_url) = start_clone_with_org(org_wrong()).await;
    let client = http_client();
    let cookie = login(&base_url, &client).await;

    let resp = client
        .get(format!("{base_url}/api/v1/alerts"))
        .header("Cookie", &cookie)
        .header("X-Prism-Org-Id", org_wrong().as_uuid().to_string())
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
// AC-003 — Missing header returns 401 (BC-3.5.001 postcondition 1)
// ===========================================================================

/// AC-003 / BC-3.5.001 postcondition 1:
/// A request that omits the `X-Prism-Org-Id` header entirely receives HTTP 401.
/// The `instance_org_id` fallback MUST NOT be accepted as a substitute for a
/// missing header once `validate_org_id` is wired in.
///
/// Traces to: BC-3.5.001 postcondition 1, W3-FIX-SEC-001 AC-003.
#[tokio::test]
async fn test_AC_003_missing_x_org_id_header_returns_401() {
    let (_clone, base_url) = start_clone_with_org(org_wrong()).await;
    let client = http_client();
    let cookie = login(&base_url, &client).await;

    // No X-Prism-Org-Id header at all.
    let resp = client
        .get(format!("{base_url}/api/v1/alerts"))
        .header("Cookie", &cookie)
        .send()
        .await
        .expect("AC-003: request must not error");

    assert_eq!(
        resp.status().as_u16(),
        401,
        "AC-003: GET /api/v1/alerts without X-Prism-Org-Id header must return HTTP 401; \
         got {} — validate_org_id must be wired in before extract_org_id fallback",
        resp.status().as_u16()
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
    let (_clone, base_url) = start_clone_with_org(org_wrong()).await;
    let client = http_client();
    let cookie = login(&base_url, &client).await;

    let resp = client
        .get(format!("{base_url}/api/v1/alerts"))
        .header("Cookie", &cookie)
        .header("X-Prism-Org-Id", org_wrong().as_uuid().to_string())
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
    let (_clone, base_url) = start_clone_with_org(org_wrong()).await;
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
         got {} — validate_org_id must treat unparseable headers as mismatch",
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
/// Traces to: W3-FIX-SEC-001 EC-003.
#[tokio::test]
async fn test_EC_003_sentinel_uuid_as_x_org_id_returns_401() {
    let (_clone, base_url) = start_clone_with_org(org_wrong()).await;
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
         got {} — sentinel must not be accepted as a valid org identity",
        resp.status().as_u16()
    );
}
