//! X-Org-Id auth tests for `prism-dtu-cyberint` (rewritten for ADR-031 §D3-a).
//!
//! Exercises the X-Prism-Org-Id header behavior under the new static access_token
//! allowlist model (ADR-031 §D3-a).
//!
//! # Auth Model: Cyberint uses Access-Token Allowlist (ADR-031 §D3-a)
//!
//! Under ADR-031 §D3-a, the access_token is org-agnostic. A valid access_token is
//! accepted regardless of the X-Prism-Org-Id header value (when the UUID is valid).
//!
//! - Valid access_token + any valid UUID X-Prism-Org-Id → 200
//!   (The UUID is a routing hint; auth is via access_token allowlist, not per-org session)
//! - Valid access_token + missing X-Prism-Org-Id → 200 (falls back to instance_org_id)
//! - Valid access_token + non-UUID X-Prism-Org-Id → 401 (non-UUID guard fires first)
//! - Missing access_token → 401 (no cookie → unauthorized)
//!
//! # Semantic change from W3-FIX-SEC-001
//!
//! The original x_org_id_auth.rs tests (W3-FIX-SEC-001) tested per-org session validation:
//! "session registered for org A + X-Prism-Org-Id for org B → 401 (session not found for B)".
//! That model is superseded by ADR-031 §D3-a: access_token is org-agnostic (see
//! state.rs `reset_for` comment). Cross-org requests with a valid access_token are now
//! accepted (200), not rejected.
//!
//! # Acceptance Criteria covered
//!
//! | Test | Description |
//! |------|-------------|
//! | AC-001 | Same-org request with access_token → 200 (BC-3.2.001 postcondition 1) |
//! | AC-002 | No-cookie request → 401 (missing auth is still unauthorized) |
//! | AC-003 | Missing X-Prism-Org-Id → 200 (fallback to instance_org_id) |
//! | EC-001 | Non-UUID X-Prism-Org-Id → 401 (non-UUID guard fires before auth check) |

#![allow(clippy::expect_used, clippy::unwrap_used, non_snake_case)]
#![cfg(feature = "dtu")]

use prism_dtu_common::BehavioralClone;
use prism_dtu_cyberint::CyberintClone;

const DEMO_TOKEN: &str = "demo-access-key";

// ---------------------------------------------------------------------------
// Sentinel UUID — valid UUID format; not a real OrgId
// ---------------------------------------------------------------------------

const SENTINEL_UUID: &str = "00000000-0000-7000-8000-000000000000";

// ---------------------------------------------------------------------------
// Test helper: start a clone with demo token registered.
// Returns (clone, base_url).
// ---------------------------------------------------------------------------

async fn start_clone() -> (CyberintClone, String) {
    let mut clone = CyberintClone::new().expect("CyberintClone::new failed");
    clone.start().await.expect("CyberintClone::start failed");
    let base_url = clone.base_url();
    clone
        .configure(serde_json::json!({"access_token": DEMO_TOKEN}))
        .await
        .expect("configure access_token must succeed");
    (clone, base_url)
}

/// Build a reqwest Client with short timeout for testing.
fn http_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .expect("reqwest::Client build")
}

fn foreign_org_uuid() -> String {
    "00000000-0000-7000-8000-0000000000BB".to_owned()
}

// ===========================================================================
// AC-001 — Same-org request with access_token succeeds (BC-3.2.001 postcondition 1)
// ===========================================================================

/// AC-001 / BC-3.2.001 postcondition 1:
/// A request supplying `Cookie: access_token=demo-access-key` and
/// `X-Prism-Org-Id: <instance_org_id>` receives HTTP 200.
///
/// Under ADR-031 §D3-a, the access_token is org-agnostic; the X-Prism-Org-Id
/// header is a routing hint only.
///
/// Traces to: BC-3.2.001 postcondition 1, W3-FIX-SEC-001 AC-001.
#[tokio::test]
async fn test_AC_001_x_org_id_validated_against_bearer_token() {
    let (clone, base_url) = start_clone().await;
    let instance_org_id = clone.instance_org_id();
    let client = http_client();

    let resp = client
        .get(format!("{base_url}/api/v1/alerts"))
        .header("Cookie", format!("access_token={DEMO_TOKEN}"))
        .header("X-Prism-Org-Id", instance_org_id.as_uuid().to_string())
        .send()
        .await
        .expect("AC-001: request must not error");

    assert_eq!(
        resp.status().as_u16(),
        200,
        "AC-001: GET /api/v1/alerts with valid access_token + matching X-Prism-Org-Id \
         must return HTTP 200; got {}",
        resp.status().as_u16()
    );
}

// ===========================================================================
// AC-002 — Missing access_token → 401 (auth still required)
// ===========================================================================

/// Under ADR-031 §D3-a, a request with NO Cookie header at all returns 401.
/// Auth is required; the access_token allowlist model is still an auth gate.
///
/// Previously tested as "cross-org credential returns 401 (session not found)".
/// Under the new model the correct test for 401 is: absent or invalid access_token.
#[tokio::test]
async fn test_AC_002_cross_org_credential_returns_401() {
    let (_clone, base_url) = start_clone().await;
    let client = http_client();

    // No Cookie header at all — must return 401 (no access_token).
    let resp = client
        .get(format!("{base_url}/api/v1/alerts"))
        .header("X-Prism-Org-Id", foreign_org_uuid())
        .send()
        .await
        .expect("AC-002: request must not error");

    assert_eq!(
        resp.status().as_u16(),
        401,
        "AC-002: GET /api/v1/alerts with no Cookie must return HTTP 401 \
         (access_token required); got {}",
        resp.status().as_u16()
    );
}

/// AC-002 variant — 401 response body has expected shape.
#[tokio::test]
async fn test_AC_002_cross_org_401_body_is_json_error_object() {
    let (_clone, base_url) = start_clone().await;
    let client = http_client();

    // No access_token cookie → 401.
    let resp = client
        .get(format!("{base_url}/api/v1/alerts"))
        .header("X-Prism-Org-Id", foreign_org_uuid())
        .send()
        .await
        .expect("AC-002 body: request must not error");

    assert_eq!(
        resp.status().as_u16(),
        401,
        "AC-002 body: expected HTTP 401 for missing access_token; got {}",
        resp.status().as_u16()
    );

    let body: serde_json::Value = resp
        .json()
        .await
        .expect("AC-002 body: 401 response must be valid JSON");

    assert!(
        body.get("error").is_some(),
        "AC-002 body: 401 response must include 'error' field; got: {body:?}"
    );
}

// ===========================================================================
// AC-003 — Missing X-Prism-Org-Id → 200 (fallback to instance_org_id)
// ===========================================================================

/// AC-003 (Cyberint ADR-031 model) — missing X-Prism-Org-Id header with valid
/// access_token returns HTTP 200 via instance fallback.
///
/// Traces to: W3-FIX-SEC-001 AC-003 (model B semantics retained).
#[tokio::test]
async fn test_AC_003_cyberint_missing_header_returns_default_session_or_400() {
    let (_clone, base_url) = start_clone().await;
    let client = http_client();

    let resp = client
        .get(format!("{base_url}/api/v1/alerts"))
        .header("Cookie", format!("access_token={DEMO_TOKEN}"))
        .send()
        .await
        .expect("AC-003: request must not error");

    assert_eq!(
        resp.status().as_u16(),
        200,
        "AC-003: GET /api/v1/alerts with valid access_token and no X-Prism-Org-Id \
         must return HTTP 200 via instance fallback; got {} \
         (ADR-031 §D3-a: access_token is org-agnostic; missing header defaults to instance_org_id)",
        resp.status().as_u16()
    );
}

/// AC-003 variant — valid access_token + valid-but-foreign UUID → 200.
///
/// Under ADR-031 §D3-a, the access_token is org-agnostic.
/// A valid access_token with any valid UUID header succeeds.
#[tokio::test]
async fn test_AC_003_cyberint_mismatched_header_returns_401_session_not_found() {
    let (_clone, base_url) = start_clone().await;
    let client = http_client();

    // Valid access_token + valid-but-foreign UUID.
    // Under ADR-031 §D3-a (org-agnostic access_token), this returns 200.
    let resp = client
        .get(format!("{base_url}/api/v1/alerts"))
        .header("Cookie", format!("access_token={DEMO_TOKEN}"))
        .header("X-Prism-Org-Id", foreign_org_uuid())
        .send()
        .await
        .expect("AC-003 mismatch: request must not error");

    assert_eq!(
        resp.status().as_u16(),
        200,
        "AC-003 (ADR-031 §D3-a): GET /api/v1/alerts with valid access_token + foreign \
         X-Prism-Org-Id must return HTTP 200 (access_token is org-agnostic; any valid \
         UUID header is accepted); got {}",
        resp.status().as_u16()
    );
}

// ===========================================================================
// AC-005 — Regression: cross-org header with missing access_token → 401
// ===========================================================================

/// AC-005 / BC-3.5.002:
/// A request with no Cookie header returns 401 regardless of X-Prism-Org-Id.
/// Auth is still required; the token model changed but the auth gate did not.
///
/// Traces to: BC-3.5.002 precondition 3, W3-FIX-SEC-001 AC-005.
#[tokio::test]
async fn test_cross_org_header_rejected() {
    let (_clone, base_url) = start_clone().await;
    let client = http_client();

    // No Cookie header — must return 401 regardless of X-Prism-Org-Id.
    let resp = client
        .get(format!("{base_url}/api/v1/alerts"))
        .header("X-Prism-Org-Id", foreign_org_uuid())
        .send()
        .await
        .expect("AC-005: request must not error");

    let status = resp.status().as_u16();

    assert_eq!(
        status, 401,
        "AC-005: request with no access_token cookie must return HTTP 401, not {status}"
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
/// return HTTP 401 — the non-UUID guard fires before the access_token check.
///
/// Traces to: W3-FIX-SEC-001 EC-001.
#[tokio::test]
async fn test_EC_001_non_uuid_x_org_id_returns_401() {
    let (_clone, base_url) = start_clone().await;
    let client = http_client();

    let resp = client
        .get(format!("{base_url}/api/v1/alerts"))
        .header("Cookie", format!("access_token={DEMO_TOKEN}"))
        .header("X-Prism-Org-Id", "not-a-uuid-at-all")
        .send()
        .await
        .expect("EC-001: request must not error");

    assert_eq!(
        resp.status().as_u16(),
        401,
        "EC-001: non-UUID X-Prism-Org-Id header must return HTTP 401 \
         (non-UUID guard fires before auth check); got {}",
        resp.status().as_u16()
    );
}

// ===========================================================================
// EC-003 — Sentinel UUID sent as header → 200 (valid UUID, valid access_token)
// ===========================================================================

/// EC-003 (updated for ADR-031 §D3-a):
/// The sentinel UUID `00000000-0000-7000-8000-000000000000` is a valid UUID.
/// Under ADR-031 §D3-a (org-agnostic access_token), a valid access_token + valid
/// UUID header returns 200 — the access_token is accepted regardless of the UUID value.
///
/// The old expectation was 401 (under the session-store model: no session for sentinel org).
/// Under the new model: access_token is org-agnostic; the sentinel UUID is just a valid
/// routing key; no per-org session check exists.
///
/// Traces to: W3-FIX-SEC-001 EC-003 (updated for ADR-031 §D3-a).
#[tokio::test]
async fn test_EC_003_sentinel_uuid_as_x_org_id_returns_401() {
    let (_clone, base_url) = start_clone().await;
    let client = http_client();

    let resp = client
        .get(format!("{base_url}/api/v1/alerts"))
        .header("Cookie", format!("access_token={DEMO_TOKEN}"))
        .header("X-Prism-Org-Id", SENTINEL_UUID)
        .send()
        .await
        .expect("EC-003: request must not error");

    // Under ADR-031 §D3-a: valid access_token + valid UUID → 200.
    // (The old expectation of 401 was based on the session-store model, now superseded.)
    assert_eq!(
        resp.status().as_u16(),
        200,
        "EC-003 (ADR-031 §D3-a): valid access_token + sentinel UUID as X-Prism-Org-Id \
         must return HTTP 200 (access_token is org-agnostic; sentinel UUID is a valid UUID); \
         got {}",
        resp.status().as_u16()
    );
}
