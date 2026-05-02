//! CR-023 — Armis `validate_org_id` dual-mode guard: activity and risk endpoints.
//!
//! Verifies that the `is_real_org` dual-mode guard (extended to `devices.rs` by
//! W3-FIX-CODE-005) is enforced on the two remaining Armis route handlers:
//!
//!   - `GET /api/v1/devices/{device_id}/activity`  (`get_device_activity`)
//!   - `GET /api/v1/devices/{device_id}/risk`       (`get_device_risk`)
//!
//! Guard semantics:
//!   - Real-org clones (`instance_org_id != DTU_DEFAULT_INSTANCE_ORG_ID`): absent `X-Org-Id`
//!     → HTTP 401.  Matching `X-Org-Id` → HTTP 200.
//!   - Default-instance clones (`instance_org_id == DTU_DEFAULT_INSTANCE_ORG_ID`): absent
//!     `X-Org-Id` → HTTP 200 (backward compatibility).
//!
//! # Behavioral contracts exercised
//!
//!   BC-3.5.001 invariant 3 — failure injection state scoped to target clone; the
//!   dual-mode `is_real_org` guard enforces the per-clone `instance_org_id` boundary
//!   that supports clone isolation.
//!
//! # Acceptance criteria
//!
//!   AC-001 — `get_device_activity` real-org: absent `X-Org-Id` → HTTP 401.
//!   AC-002 — `get_device_activity` real-org: correct `X-Org-Id` → HTTP 200.
//!   AC-003 — `get_device_activity` default-instance: absent `X-Org-Id` → HTTP 200.
//!   AC-004 — `get_device_risk` real-org: absent `X-Org-Id` → HTTP 401.
//!   AC-005 — `get_device_risk` real-org: correct `X-Org-Id` → HTTP 200.
//!   AC-006 — `get_device_risk` default-instance: absent `X-Org-Id` → HTTP 200.
//!
//! # Naming convention
//!
//! Tests follow `test_<handler>_<mode>_<condition>_<result>` per the story mandate and
//! the existing `cr017_tag_alert_org_id_guard.rs` corpus.

#![allow(clippy::expect_used, clippy::unwrap_used, non_snake_case)]
#![cfg(feature = "dtu")]

use prism_core::OrgId;
use prism_dtu_armis::ArmisClone;
use prism_dtu_common::BehavioralClone;

// ---------------------------------------------------------------------------
// Test helpers
// ---------------------------------------------------------------------------

fn http_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .expect("reqwest::Client build")
}

/// A non-default, non-nil OrgId that triggers the `is_real_org` guard.
///
/// Must differ from `DTU_DEFAULT_INSTANCE_ORG_ID`
/// (`00000000-0000-7000-8000-0000000000AA`) to activate the instance-identity check.
fn real_org_id() -> OrgId {
    OrgId(uuid::uuid!("00000000-0000-7000-8000-0000000000CC"))
}

// ===========================================================================
// get_device_activity — BC-3.5.001 invariant 3; CR-023 AC-001..003
// ===========================================================================

/// CR-023 AC-001: `GET /api/v1/devices/{id}/activity` on a real-org Armis clone must
/// reject a request that omits the `X-Org-Id` header with HTTP 401, before any
/// activity data is exposed.
///
/// Guard code: `devices.rs:205-209` — `is_real_org || headers.get("x-org-id").is_some()`.
/// This test FAILS if the guard at `devices.rs:205-209` is removed.
///
/// (BC-3.5.001 invariant 3; CR-023 AC-001)
#[tokio::test]
async fn test_get_device_activity_real_org_absent_header_returns_401() {
    let org_id = real_org_id();
    let mut clone = ArmisClone::new_with_org(org_id).expect("ArmisClone::new_with_org failed");
    clone.start().await.expect("ArmisClone::start failed");
    let base_url = clone.base_url();

    let resp = http_client()
        .get(format!("{base_url}/api/v1/devices/d-001/activity"))
        .header("Authorization", "Bearer test-token")
        // No X-Org-Id header — instance-identity guard must reject this.
        .send()
        .await
        .expect("CR-023: request must not error at network level");

    assert_eq!(
        resp.status().as_u16(),
        401,
        "CR-023 AC-001: real-org clone must reject GET /api/v1/devices/{{id}}/activity \
         with absent X-Org-Id → HTTP 401; \
         expected 401 got {} (BC-3.5.001 invariant 3; CR-023 AC-001)",
        resp.status().as_u16()
    );
}

/// CR-023 AC-002: `GET /api/v1/devices/{id}/activity` with the correct `X-Org-Id`
/// on a real-org clone must succeed with HTTP 200.
///
/// (BC-3.5.001 postcondition 1; CR-023 AC-002)
#[tokio::test]
async fn test_get_device_activity_real_org_correct_header_returns_200() {
    let org_id = real_org_id();
    let mut clone = ArmisClone::new_with_org(org_id).expect("ArmisClone::new_with_org failed");
    clone.start().await.expect("ArmisClone::start failed");
    let base_url = clone.base_url();

    let resp = http_client()
        .get(format!("{base_url}/api/v1/devices/d-001/activity"))
        .header("Authorization", "Bearer test-token")
        .header("X-Org-Id", org_id.as_uuid().to_string())
        .send()
        .await
        .expect("CR-023 positive: request must not error");

    assert_eq!(
        resp.status().as_u16(),
        200,
        "CR-023 AC-002: real-org clone must accept GET /api/v1/devices/{{id}}/activity \
         with correct X-Org-Id → HTTP 200; \
         expected 200 got {} (BC-3.5.001 postcondition 1; CR-023 AC-002)",
        resp.status().as_u16()
    );
}

/// CR-023 AC-003: `GET /api/v1/devices/{id}/activity` on a default-instance clone
/// must allow absent `X-Org-Id` → HTTP 200 (backward compatibility; guard is a no-op
/// for `DTU_DEFAULT_INSTANCE_ORG_ID`).
///
/// (BC-3.5.001 postcondition 2; CR-023 AC-003)
#[tokio::test]
async fn test_get_device_activity_default_instance_absent_header_returns_200() {
    // ArmisClone::new() uses DTU_DEFAULT_INSTANCE_ORG_ID — guard is skipped.
    let mut clone = ArmisClone::new().expect("ArmisClone::new failed");
    clone.start().await.expect("ArmisClone::start failed");
    let base_url = clone.base_url();

    let resp = http_client()
        .get(format!("{base_url}/api/v1/devices/d-001/activity"))
        .header("Authorization", "Bearer test-token")
        // No X-Org-Id header — default-instance clone must skip the guard.
        .send()
        .await
        .expect("CR-023 backward compat: request must not error");

    assert_eq!(
        resp.status().as_u16(),
        200,
        "CR-023 AC-003: default-instance clone must allow GET /api/v1/devices/{{id}}/activity \
         with absent X-Org-Id → HTTP 200 (backward compat); \
         expected 200 got {} (BC-3.5.001 postcondition 2; CR-023 AC-003)",
        resp.status().as_u16()
    );
}

// ===========================================================================
// get_device_risk — BC-3.5.001 invariant 3; CR-023 AC-004..006
// ===========================================================================

/// CR-023 AC-004: `GET /api/v1/devices/{id}/risk` on a real-org Armis clone must
/// reject a request that omits the `X-Org-Id` header with HTTP 401, before any
/// risk data is exposed.
///
/// Guard code: `devices.rs:241-245` — mirrors the activity guard.
/// This test FAILS if the guard at `devices.rs:241-245` is removed.
///
/// (BC-3.5.001 invariant 3; CR-023 AC-004)
#[tokio::test]
async fn test_get_device_risk_real_org_absent_header_returns_401() {
    let org_id = real_org_id();
    let mut clone = ArmisClone::new_with_org(org_id).expect("ArmisClone::new_with_org failed");
    clone.start().await.expect("ArmisClone::start failed");
    let base_url = clone.base_url();

    let resp = http_client()
        .get(format!("{base_url}/api/v1/devices/d-001/risk"))
        .header("Authorization", "Bearer test-token")
        // No X-Org-Id header — instance-identity guard must reject this.
        .send()
        .await
        .expect("CR-023: request must not error at network level");

    assert_eq!(
        resp.status().as_u16(),
        401,
        "CR-023 AC-004: real-org clone must reject GET /api/v1/devices/{{id}}/risk \
         with absent X-Org-Id → HTTP 401; \
         expected 401 got {} (BC-3.5.001 invariant 3; CR-023 AC-004)",
        resp.status().as_u16()
    );
}

/// CR-023 AC-005: `GET /api/v1/devices/{id}/risk` with the correct `X-Org-Id`
/// on a real-org clone must succeed with HTTP 200.
///
/// (BC-3.5.001 postcondition 1; CR-023 AC-005)
#[tokio::test]
async fn test_get_device_risk_real_org_correct_header_returns_200() {
    let org_id = real_org_id();
    let mut clone = ArmisClone::new_with_org(org_id).expect("ArmisClone::new_with_org failed");
    clone.start().await.expect("ArmisClone::start failed");
    let base_url = clone.base_url();

    let resp = http_client()
        .get(format!("{base_url}/api/v1/devices/d-001/risk"))
        .header("Authorization", "Bearer test-token")
        .header("X-Org-Id", org_id.as_uuid().to_string())
        .send()
        .await
        .expect("CR-023 positive: request must not error");

    assert_eq!(
        resp.status().as_u16(),
        200,
        "CR-023 AC-005: real-org clone must accept GET /api/v1/devices/{{id}}/risk \
         with correct X-Org-Id → HTTP 200; \
         expected 200 got {} (BC-3.5.001 postcondition 1; CR-023 AC-005)",
        resp.status().as_u16()
    );
}

/// CR-023 AC-006: `GET /api/v1/devices/{id}/risk` on a default-instance clone
/// must allow absent `X-Org-Id` → HTTP 200 (backward compatibility; guard is a no-op
/// for `DTU_DEFAULT_INSTANCE_ORG_ID`).
///
/// (BC-3.5.001 postcondition 2; CR-023 AC-006)
#[tokio::test]
async fn test_get_device_risk_default_instance_absent_header_returns_200() {
    // ArmisClone::new() uses DTU_DEFAULT_INSTANCE_ORG_ID — guard is skipped.
    let mut clone = ArmisClone::new().expect("ArmisClone::new failed");
    clone.start().await.expect("ArmisClone::start failed");
    let base_url = clone.base_url();

    let resp = http_client()
        .get(format!("{base_url}/api/v1/devices/d-001/risk"))
        .header("Authorization", "Bearer test-token")
        // No X-Org-Id header — default-instance clone must skip the guard.
        .send()
        .await
        .expect("CR-023 backward compat: request must not error");

    assert_eq!(
        resp.status().as_u16(),
        200,
        "CR-023 AC-006: default-instance clone must allow GET /api/v1/devices/{{id}}/risk \
         with absent X-Org-Id → HTTP 200 (backward compat); \
         expected 200 got {} (BC-3.5.001 postcondition 2; CR-023 AC-006)",
        resp.status().as_u16()
    );
}
