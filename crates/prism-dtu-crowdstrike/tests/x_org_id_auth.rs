//! W3-FIX-SEC-001 — X-Org-Id auth enforcement tests for `prism-dtu-crowdstrike`.
//!
//! Exercises BC-3.5.001, BC-3.5.002, and BC-3.2.001 per the story acceptance criteria.
//!
//! # Red Gate (Phase 2)
//!
//! Test bodies replaced with real assertion-driven logic. Tests for AC-002, AC-003,
//! EC-001, EC-003, and AC-005 assert HTTP 401 on mismatch/missing/malformed org
//! headers. These tests currently FAIL with "expected 401, got 200" because
//! `validate_org_id` is `todo!()` and not yet wired into CrowdStrike route handlers.
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
//! | EC-001 | Non-UUID value in X-Org-Id header → HTTP 401 |
//! | EC-003 | Sentinel UUID sent as header → HTTP 401 |

#![allow(clippy::expect_used, clippy::unwrap_used, non_snake_case)]
#![cfg(feature = "dtu")]

use std::sync::Arc;

use prism_core::OrgId;
use prism_dtu_common::BehavioralClone;
use prism_dtu_crowdstrike::{CrowdstrikeClone, CrowdstrikeState};

// ---------------------------------------------------------------------------
// Sentinel UUID — NEVER a valid instance_org_id
// ---------------------------------------------------------------------------

const SENTINEL_UUID: &str = "00000000-0000-7000-8000-000000000000";

// ---------------------------------------------------------------------------
// Test helper: start a clone whose instance_org_id is set to `org_id`.
//
// CrowdstrikeClone exposes `pub state: Arc<CrowdstrikeState>`, so we can swap
// in a state with the desired instance_org_id before starting.
// ---------------------------------------------------------------------------

async fn start_clone_with_org(org_id: OrgId) -> (CrowdstrikeClone, String) {
    let mut clone = CrowdstrikeClone::new();
    let token = clone.admin_token().to_string();
    clone.state = Arc::new(CrowdstrikeState::with_admin_token_and_org(token, org_id));
    clone.start().await.expect("CrowdstrikeClone::start failed");
    let base_url = clone.base_url();
    (clone, base_url)
}

/// Build a reqwest Client with a short timeout for testing.
fn http_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .expect("reqwest::Client build")
}

// ---------------------------------------------------------------------------
// Deterministic test OrgIds
// ---------------------------------------------------------------------------

fn org_a() -> OrgId {
    OrgId::from_uuid(
        uuid::Uuid::parse_str("00000000-0000-7000-8000-0000000000AA").expect("valid uuid"),
    )
}

fn org_b() -> OrgId {
    OrgId::from_uuid(
        uuid::Uuid::parse_str("00000000-0000-7000-8000-0000000000BB").expect("valid uuid"),
    )
}

// ===========================================================================
// AC-001 — Same-org request succeeds (BC-3.2.001 postcondition 1)
// ===========================================================================

/// AC-001 / BC-3.2.001 postcondition 1:
/// A request supplying `X-Org-Id: <instance_org_id>` receives HTTP 200
/// from `GET /devices/queries/devices/v1`.
///
/// Traces to: BC-3.2.001 postcondition 1, W3-FIX-SEC-001 AC-001.
#[tokio::test]
async fn test_AC_001_x_org_id_validated_against_bearer_token() {
    let instance_org = org_a();
    let (_clone, base_url) = start_clone_with_org(instance_org).await;
    let client = http_client();

    let resp = client
        .get(format!("{base_url}/devices/queries/devices/v1"))
        .header("Authorization", "Bearer test-token")
        .header("X-Org-Id", instance_org.as_uuid().to_string())
        .send()
        .await
        .expect("AC-001: request must not error");

    assert_eq!(
        resp.status().as_u16(),
        200,
        "AC-001: GET /devices/queries/devices/v1 with matching X-Org-Id must return HTTP 200; \
         got {} — validate_org_id is not yet wired into CrowdStrike route handlers",
        resp.status().as_u16()
    );
}

// ===========================================================================
// AC-002 — Cross-org spoofing returns 401 (BC-3.5.002 precondition 3)
// ===========================================================================

/// AC-002 / BC-3.5.002 precondition 3:
/// A request supplying a different org's UUID in `X-Org-Id` receives HTTP 401
/// with JSON body `{"error": "org_id mismatch: request does not match this clone instance"}`.
///
/// Traces to: BC-3.5.002 precondition 3, W3-FIX-SEC-001 AC-002.
#[tokio::test]
async fn test_AC_002_cross_org_credential_returns_401() {
    // Clone is bound to org_a; caller supplies org_b UUID.
    let (_clone, base_url) = start_clone_with_org(org_a()).await;
    let client = http_client();

    let resp = client
        .get(format!("{base_url}/devices/queries/devices/v1"))
        .header("Authorization", "Bearer test-token")
        .header("X-Org-Id", org_b().as_uuid().to_string())
        .send()
        .await
        .expect("AC-002: request must not error");

    assert_eq!(
        resp.status().as_u16(),
        401,
        "AC-002: GET /devices/queries/devices/v1 with mismatched X-Org-Id must return HTTP 401; \
         got {} — validate_org_id is not yet wired into hosts handler",
        resp.status().as_u16()
    );
}

/// AC-002 variant — JSON error body has expected shape.
///
/// The 401 response body MUST be `{"error": "org_id mismatch: ..."}` (not plain text).
/// Traces to: W3-FIX-SEC-001 AC-002, Architecture Compliance Rule §3.
#[tokio::test]
async fn test_AC_002_cross_org_401_body_is_json_error_object() {
    let (_clone, base_url) = start_clone_with_org(org_a()).await;
    let client = http_client();

    let resp = client
        .get(format!("{base_url}/devices/queries/devices/v1"))
        .header("Authorization", "Bearer test-token")
        .header("X-Org-Id", org_b().as_uuid().to_string())
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
/// A request that omits the `X-Org-Id` header entirely receives HTTP 401.
///
/// Traces to: BC-3.5.001 postcondition 1, W3-FIX-SEC-001 AC-003.
#[tokio::test]
async fn test_AC_003_missing_x_org_id_header_returns_401() {
    let (_clone, base_url) = start_clone_with_org(org_a()).await;
    let client = http_client();

    // No X-Org-Id header at all.
    let resp = client
        .get(format!("{base_url}/devices/queries/devices/v1"))
        .header("Authorization", "Bearer test-token")
        .send()
        .await
        .expect("AC-003: request must not error");

    assert_eq!(
        resp.status().as_u16(),
        401,
        "AC-003: GET /devices/queries/devices/v1 without X-Org-Id header must return HTTP 401; \
         got {} — sentinel fallback must be rejected by validate_org_id",
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
    let (_clone, base_url) = start_clone_with_org(org_a()).await;
    let client = http_client();

    let resp = client
        .get(format!("{base_url}/devices/queries/devices/v1"))
        .header("Authorization", "Bearer test-token")
        .header("X-Org-Id", org_b().as_uuid().to_string())
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
// EC-001 — Non-UUID value in X-Org-Id → 401
// ===========================================================================

/// EC-001:
/// When `X-Org-Id` is present but not a valid UUID string, the handler must
/// return HTTP 401 with `{"error": "org_id mismatch: ..."}`.
///
/// Traces to: W3-FIX-SEC-001 EC-001.
#[tokio::test]
async fn test_EC_001_non_uuid_x_org_id_returns_401() {
    let (_clone, base_url) = start_clone_with_org(org_a()).await;
    let client = http_client();

    let resp = client
        .get(format!("{base_url}/devices/queries/devices/v1"))
        .header("Authorization", "Bearer test-token")
        .header("X-Org-Id", "not-a-uuid-at-all")
        .send()
        .await
        .expect("EC-001: request must not error");

    assert_eq!(
        resp.status().as_u16(),
        401,
        "EC-001: non-UUID X-Org-Id header must return HTTP 401; \
         got {} — validate_org_id must treat unparseable headers as mismatch",
        resp.status().as_u16()
    );
}

// ===========================================================================
// EC-003 — Sentinel UUID sent as header → 401
// ===========================================================================

/// EC-003:
/// Sending the sentinel UUID `00000000-0000-7000-8000-000000000000` as the
/// `X-Org-Id` header must return HTTP 401.
///
/// Traces to: W3-FIX-SEC-001 EC-003.
#[tokio::test]
async fn test_EC_003_sentinel_uuid_as_x_org_id_returns_401() {
    let (_clone, base_url) = start_clone_with_org(org_a()).await;
    let client = http_client();

    let resp = client
        .get(format!("{base_url}/devices/queries/devices/v1"))
        .header("Authorization", "Bearer test-token")
        .header("X-Org-Id", SENTINEL_UUID)
        .send()
        .await
        .expect("EC-003: request must not error");

    assert_eq!(
        resp.status().as_u16(),
        401,
        "EC-003: sentinel UUID in X-Org-Id must return HTTP 401; \
         got {} — sentinel must not be accepted as a valid org identity",
        resp.status().as_u16()
    );
}
