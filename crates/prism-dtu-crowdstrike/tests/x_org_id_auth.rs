//! W3-FIX-SEC-001 — X-Org-Id auth enforcement tests for `prism-dtu-crowdstrike`.
//!
//! Exercises BC-3.5.001, BC-3.5.002, and BC-3.2.001 per the story acceptance criteria.
//!
//! # Red Gate (Phase 1)
//!
//! All test bodies are `todo!()`. They compile but panic at runtime, satisfying the
//! Red Gate requirement: every test MUST FAIL before the implementation lands.
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

use prism_core::OrgId;
use prism_dtu_common::BehavioralClone;
use prism_dtu_crowdstrike::CrowdstrikeClone;

// ---------------------------------------------------------------------------
// Test helper: start a clone and return (clone, base_url)
// ---------------------------------------------------------------------------

async fn start_clone_with_org(_org_id: OrgId) -> (CrowdstrikeClone, String) {
    todo!("AC-001: start_clone_with_org helper — create CrowdstrikeClone whose instance_org_id == org_id")
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
    todo!("AC-001: same-org X-Org-Id header returns HTTP 200 from CrowdStrike /devices/queries/devices/v1")
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
    todo!("AC-002: cross-org X-Org-Id header returns HTTP 401 from CrowdStrike host routes")
}

/// AC-002 variant — JSON error body has expected shape.
///
/// The 401 response body MUST be `{"error": "org_id mismatch: ..."}` (not plain text).
/// Traces to: W3-FIX-SEC-001 AC-002, Architecture Compliance Rule §3.
#[tokio::test]
async fn test_AC_002_cross_org_401_body_is_json_error_object() {
    todo!("AC-002: verify 401 response body is JSON object with 'error' key on cross-org CrowdStrike request")
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
    todo!("AC-003: absent X-Org-Id header returns HTTP 401 from CrowdStrike host routes")
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
    todo!("AC-005: cross-org credential mismatch returns 401, not 200, not silent empty (CrowdStrike)")
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
    todo!("EC-001: non-UUID X-Org-Id header returns HTTP 401 from CrowdStrike host routes")
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
    todo!("EC-003: sentinel UUID 00000000-0000-7000-8000-000000000000 in X-Org-Id returns HTTP 401 (CrowdStrike)")
}
