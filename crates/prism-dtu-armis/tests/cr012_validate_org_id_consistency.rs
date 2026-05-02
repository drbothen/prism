//! CR-012/SEC-P2-001: Armis `validate_org_id` guard — instance-identity model.
//!
//! Verifies that after the CR-012 fix, the Armis DTU clone uses the same
//! instance-identity guard as Claroty and CrowdStrike:
//!
//!   - Real-org clones (instance_org_id != DTU_DEFAULT_INSTANCE_ORG_ID) MUST reject
//!     requests with a missing `X-Org-Id` header with HTTP 401 (AC-003).
//!   - Legacy single-tenant clones (instance_org_id == DTU_DEFAULT_INSTANCE_ORG_ID)
//!     skip the guard for backward compatibility (EC-003).
//!   - Mismatched `X-Org-Id` always returns 401 regardless of instance type (AC-002).
//!
//! # Behavioral contracts exercised
//!
//!   BC-3.5.002 precondition 3 — org-id header must match clone's instance_org_id.
//!
//! # Previous behavior (pre-fix)
//!
//! The old code used `if headers.get("x-org-id").is_some()` — a header-presence guard.
//! This meant that a real-org clone would silently allow requests with no `X-Org-Id`
//! header, weaker than the Claroty/CrowdStrike model. CR-012 replaces this with the
//! `if state.instance_org_id != DTU_DEFAULT_INSTANCE_ORG_ID` instance-identity guard.

#![allow(clippy::expect_used, clippy::unwrap_used, non_snake_case)]
#![cfg(feature = "dtu")]

use prism_core::OrgId;
use prism_dtu_armis::state::DTU_DEFAULT_INSTANCE_ORG_ID;
use prism_dtu_armis::ArmisClone;
use prism_dtu_common::BehavioralClone;

fn http_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .expect("reqwest::Client build")
}

/// Build a real-org (non-default) OrgId for use in tests.
///
/// Must differ from DTU_DEFAULT_INSTANCE_ORG_ID to trigger the instance-identity guard.
fn real_org_id() -> OrgId {
    OrgId(uuid::uuid!("00000000-0000-7000-8000-0000000000CC"))
}

// ============================================================================
// EC-003: Legacy clone (default instance_org_id) skips guard — 200 without header
//
// BC-3.5.002 precondition 3; CR-012 EC-003 backward compat
// ============================================================================

/// EC-003: A clone constructed with `DTU_DEFAULT_INSTANCE_ORG_ID` (legacy single-tenant)
/// skips the instance-identity guard. Requests without `X-Org-Id` still return 200.
///
/// This preserves backward compatibility with pre-existing tests that call Armis
/// without any `X-Org-Id` header.
///
/// (BC-3.5.002 precondition 3; CR-012 EC-003)
#[tokio::test]
async fn test_BC_3_5_002_precon3_legacy_clone_allows_missing_org_header() {
    // ArmisClone::new() uses DTU_DEFAULT_INSTANCE_ORG_ID — guard is skipped.
    let mut clone = ArmisClone::new().expect("ArmisClone::new failed");
    clone.start().await.expect("ArmisClone::start failed");
    let base_url = clone.base_url();

    let resp = http_client()
        .get(format!("{base_url}/api/v1/devices"))
        .header("Authorization", "Bearer test-token")
        // No X-Org-Id header.
        .send()
        .await
        .expect("EC-003: request must not error");

    assert_eq!(
        resp.status().as_u16(),
        200,
        "EC-003: legacy clone (default instance_org_id) must allow missing X-Org-Id; \
         expected 200 got {} (BC-3.5.002 precondition 3; CR-012 EC-003)",
        resp.status().as_u16()
    );
}

// ============================================================================
// AC-003: Real-org clone rejects missing X-Org-Id header with 401
//
// BC-3.5.002 precondition 3; CR-012 AC-003 (instance-identity guard)
// ============================================================================

/// AC-003: A real-org Armis clone (instance_org_id != DTU_DEFAULT_INSTANCE_ORG_ID)
/// must reject requests that omit the `X-Org-Id` header with HTTP 401.
///
/// This verifies the CR-012 fix: the old header-presence guard (`is_some()`) has been
/// replaced with the instance-identity guard (`!= DTU_DEFAULT_INSTANCE_ORG_ID`).
///
/// (BC-3.5.002 precondition 3; CR-012 AC-003)
#[tokio::test]
async fn test_BC_3_5_002_precon3_real_org_clone_rejects_missing_org_header() {
    let org_id = real_org_id();
    let mut clone = ArmisClone::new_with_org(org_id).expect("ArmisClone::new_with_org failed");
    clone.start().await.expect("ArmisClone::start failed");
    let base_url = clone.base_url();

    let resp = http_client()
        .get(format!("{base_url}/api/v1/devices"))
        .header("Authorization", "Bearer test-token")
        // No X-Org-Id header — instance-identity guard must reject this.
        .send()
        .await
        .expect("AC-003: request must not error at network level");

    assert_eq!(
        resp.status().as_u16(),
        401,
        "AC-003: real-org clone must reject missing X-Org-Id with HTTP 401; \
         expected 401 got {} (BC-3.5.002 precondition 3; CR-012 AC-003)",
        resp.status().as_u16()
    );
}

// ============================================================================
// AC-001: Real-org clone accepts matching X-Org-Id — 200
//
// BC-3.2.001 postcondition 1; CR-012 AC-001
// ============================================================================

/// AC-001: A real-org clone accepts a request bearing the correct `X-Org-Id`.
///
/// (BC-3.2.001 postcondition 1; CR-012 AC-001)
#[tokio::test]
async fn test_BC_3_5_002_precon3_real_org_clone_accepts_matching_org_header() {
    let org_id = real_org_id();
    let mut clone = ArmisClone::new_with_org(org_id).expect("ArmisClone::new_with_org failed");
    clone.start().await.expect("ArmisClone::start failed");
    let base_url = clone.base_url();

    let resp = http_client()
        .get(format!("{base_url}/api/v1/devices"))
        .header("Authorization", "Bearer test-token")
        .header("X-Org-Id", org_id.as_uuid().to_string())
        .send()
        .await
        .expect("AC-001: request must not error");

    assert_eq!(
        resp.status().as_u16(),
        200,
        "AC-001: real-org clone must accept matching X-Org-Id with HTTP 200; \
         expected 200 got {} (BC-3.2.001 postcondition 1; CR-012 AC-001)",
        resp.status().as_u16()
    );
}

// ============================================================================
// AC-002: Real-org clone rejects mismatched X-Org-Id — 401
//
// BC-3.5.002 precondition 3; CR-012 AC-002
// ============================================================================

/// AC-002: A real-org clone rejects a request bearing a different org's UUID.
///
/// (BC-3.5.002 precondition 3; CR-012 AC-002)
#[tokio::test]
async fn test_BC_3_5_002_precon3_real_org_clone_rejects_mismatched_org_header() {
    let org_id = real_org_id();
    let mut clone = ArmisClone::new_with_org(org_id).expect("ArmisClone::new_with_org failed");
    clone.start().await.expect("ArmisClone::start failed");
    let base_url = clone.base_url();

    // Send a different org's UUID.
    let foreign_uuid = "00000000-0000-7000-8000-0000000000DD";

    let resp = http_client()
        .get(format!("{base_url}/api/v1/devices"))
        .header("Authorization", "Bearer test-token")
        .header("X-Org-Id", foreign_uuid)
        .send()
        .await
        .expect("AC-002: request must not error at network level");

    assert_eq!(
        resp.status().as_u16(),
        401,
        "AC-002: real-org clone must reject mismatched X-Org-Id with HTTP 401; \
         expected 401 got {} (BC-3.5.002 precondition 3; CR-012 AC-002)",
        resp.status().as_u16()
    );
}

// ============================================================================
// EC-003: DEFAULT_INSTANCE_ORG_ID is not treated as a valid "real" org
//
// BC-3.5.002 precondition 3; CR-012 EC-003
// ============================================================================

/// Regression: the `DTU_DEFAULT_INSTANCE_ORG_ID` sentinel must NOT be treated
/// as a valid real-org identity. Sending it as an `X-Org-Id` header to a legacy
/// clone (which uses `DTU_DEFAULT_INSTANCE_ORG_ID`) should return 200 (match),
/// not 401.
///
/// This documents that the sentinel UUID is a valid match for legacy clones only.
///
/// (BC-3.5.002 precondition 3; CR-012 EC-003)
#[tokio::test]
async fn test_BC_3_5_002_precon3_sentinel_org_matches_legacy_clone() {
    let mut clone = ArmisClone::new().expect("ArmisClone::new failed");
    clone.start().await.expect("ArmisClone::start failed");
    let base_url = clone.base_url();

    // Send the sentinel as X-Org-Id — legacy clone must accept it (it matches).
    let sentinel_str = DTU_DEFAULT_INSTANCE_ORG_ID.as_uuid().to_string();

    let resp = http_client()
        .get(format!("{base_url}/api/v1/devices"))
        .header("Authorization", "Bearer test-token")
        .header("X-Org-Id", sentinel_str)
        .send()
        .await
        .expect("EC-003 sentinel match: request must not error");

    assert_eq!(
        resp.status().as_u16(),
        200,
        "EC-003: legacy clone accepts DTU_DEFAULT_INSTANCE_ORG_ID as X-Org-Id; \
         expected 200 got {} (CR-012 EC-003)",
        resp.status().as_u16()
    );
}
