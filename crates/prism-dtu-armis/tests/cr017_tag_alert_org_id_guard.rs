//! CR-017 / M-50-001 — Armis `validate_org_id` dual-mode guard: tag and alert endpoints.
//!
//! Verifies that the `is_real_org` dual-mode guard (first applied to `devices.rs` by
//! W3-FIX-CODE-004) is now enforced on all remaining Armis route handlers that access
//! org-keyed state:
//!
//!   - `POST /api/v1/devices/{device_id}/tags/`  (`post_device_tag`)
//!   - `DELETE /api/v1/devices/{device_id}/tags/{tag_key}`  (`delete_device_tag`)
//!   - `GET /api/v1/alerts`  (`get_alerts`)
//!
//! Guard semantics:
//!   - Real-org clones (`instance_org_id != DTU_DEFAULT_INSTANCE_ORG_ID`): absent `X-Org-Id`
//!     → HTTP 401.  Matching `X-Org-Id` → HTTP 200/201.
//!   - Default-instance clones (`instance_org_id == DTU_DEFAULT_INSTANCE_ORG_ID`): absent
//!     `X-Org-Id` → HTTP 200/201 (backward compatibility).
//!
//! # Behavioral contracts exercised
//!
//!   BC-3.5.002 precondition 3 — `instance_org_id` guard on all org-keyed endpoints.
//!   BC-3.2.001 precondition 4 — HTTP layer enforces OrgId boundary before state access.
//!
//! # Naming convention
//!
//! Tests follow the story-mandated pattern:
//!   `test_<handler>_<condition>_<result>`
//! matching the existing `cr012_validate_org_id_consistency.rs` corpus.

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
// post_device_tag — BC-3.5.002 precondition 3; CR-017 AC-002
// ===========================================================================

/// CR-017 AC-002: `POST /api/v1/devices/{id}/tags/` on a real-org Armis clone must
/// reject a request that omits the `X-Org-Id` header with HTTP 401, before any
/// state mutation occurs.
///
/// Guard pattern: `is_real_org || headers.get("x-org-id").is_some()` (devices.rs:89-94).
/// This test FAILS before the guard is added to `tags.rs::post_device_tag`.
///
/// (BC-3.5.002 precondition 3; BC-3.2.001 precondition 4; CR-017 AC-002 EC-002)
#[tokio::test]
async fn test_post_device_tag_real_org_absent_header_returns_401() {
    let org_id = real_org_id();
    let mut clone = ArmisClone::new_with_org(org_id).expect("ArmisClone::new_with_org failed");
    clone.start().await.expect("ArmisClone::start failed");
    let base_url = clone.base_url();

    let resp = http_client()
        .post(format!("{base_url}/api/v1/devices/device-001/tags/"))
        .header("Authorization", "Bearer test-token")
        // No X-Org-Id header — instance-identity guard must reject this.
        .json(&serde_json::json!({"tag_key": "quarantine"}))
        .send()
        .await
        .expect("CR-017: request must not error at network level");

    assert_eq!(
        resp.status().as_u16(),
        401,
        "CR-017 AC-002: real-org clone must reject POST /api/v1/devices/{{id}}/tags/ \
         with absent X-Org-Id → HTTP 401; \
         expected 401 got {} (BC-3.5.002 precondition 3; CR-017 AC-002)",
        resp.status().as_u16()
    );
}

/// CR-017 AC-002 positive path: `POST /api/v1/devices/{id}/tags/` with the correct
/// `X-Org-Id` on a real-org clone must succeed with HTTP 201.
///
/// (BC-3.2.001 precondition 4; CR-017 AC-002)
#[tokio::test]
async fn test_post_device_tag_real_org_correct_header_returns_201() {
    let org_id = real_org_id();
    let mut clone = ArmisClone::new_with_org(org_id).expect("ArmisClone::new_with_org failed");
    clone.start().await.expect("ArmisClone::start failed");
    let base_url = clone.base_url();

    let resp = http_client()
        .post(format!("{base_url}/api/v1/devices/device-001/tags/"))
        .header("Authorization", "Bearer test-token")
        .header("X-Org-Id", org_id.as_uuid().to_string())
        .json(&serde_json::json!({"tag_key": "quarantine"}))
        .send()
        .await
        .expect("CR-017 positive: request must not error");

    assert_eq!(
        resp.status().as_u16(),
        201,
        "CR-017 AC-002 positive: real-org clone must accept POST /api/v1/devices/{{id}}/tags/ \
         with correct X-Org-Id → HTTP 201; \
         expected 201 got {} (BC-3.2.001 precondition 4; CR-017 AC-002)",
        resp.status().as_u16()
    );
}

/// CR-017 AC-002 backward compat: `POST /api/v1/devices/{id}/tags/` on a default-instance
/// clone (uses `DTU_DEFAULT_INSTANCE_ORG_ID`) must allow absent `X-Org-Id` → HTTP 201.
///
/// (BC-3.5.002 precondition 3; CR-017 EC-008)
#[tokio::test]
async fn test_post_device_tag_default_instance_absent_header_returns_201() {
    // ArmisClone::new() uses DTU_DEFAULT_INSTANCE_ORG_ID — guard is skipped.
    let mut clone = ArmisClone::new().expect("ArmisClone::new failed");
    clone.start().await.expect("ArmisClone::start failed");
    let base_url = clone.base_url();

    let resp = http_client()
        .post(format!("{base_url}/api/v1/devices/device-001/tags/"))
        .header("Authorization", "Bearer test-token")
        // No X-Org-Id header — default-instance clone must skip the guard.
        .json(&serde_json::json!({"tag_key": "quarantine"}))
        .send()
        .await
        .expect("CR-017 backward compat: request must not error");

    assert_eq!(
        resp.status().as_u16(),
        201,
        "CR-017 EC-008: default-instance clone must allow POST /api/v1/devices/{{id}}/tags/ \
         with absent X-Org-Id → HTTP 201 (backward compat); \
         expected 201 got {} (BC-3.5.002 precondition 3; CR-017 EC-008)",
        resp.status().as_u16()
    );
}

// ===========================================================================
// delete_device_tag — BC-3.5.002 precondition 3; CR-017 AC-002
// ===========================================================================

/// CR-017 AC-002: `DELETE /api/v1/devices/{id}/tags/{key}` on a real-org Armis clone must
/// reject a request that omits the `X-Org-Id` header with HTTP 401, before any state
/// mutation occurs.
///
/// This test FAILS before the guard is added to `tags.rs::delete_device_tag`.
///
/// (BC-3.5.002 precondition 3; BC-3.2.001 precondition 4; CR-017 AC-002 EC-003)
#[tokio::test]
async fn test_delete_device_tag_real_org_absent_header_returns_401() {
    let org_id = real_org_id();
    let mut clone = ArmisClone::new_with_org(org_id).expect("ArmisClone::new_with_org failed");
    clone.start().await.expect("ArmisClone::start failed");
    let base_url = clone.base_url();

    let resp = http_client()
        .delete(format!(
            "{base_url}/api/v1/devices/device-001/tags/quarantine"
        ))
        .header("Authorization", "Bearer test-token")
        // No X-Org-Id header — instance-identity guard must reject this.
        .send()
        .await
        .expect("CR-017 delete: request must not error at network level");

    assert_eq!(
        resp.status().as_u16(),
        401,
        "CR-017 AC-002: real-org clone must reject DELETE /api/v1/devices/{{id}}/tags/{{key}} \
         with absent X-Org-Id → HTTP 401; \
         expected 401 got {} (BC-3.5.002 precondition 3; CR-017 AC-002 EC-003)",
        resp.status().as_u16()
    );
}

/// CR-017 AC-002 backward compat: `DELETE /api/v1/devices/{id}/tags/{key}` on a
/// default-instance clone must allow absent `X-Org-Id` → non-401 response.
///
/// (BC-3.5.002 precondition 3; CR-017 EC-008)
#[tokio::test]
async fn test_delete_device_tag_default_instance_absent_header_allows_request() {
    // ArmisClone::new() uses DTU_DEFAULT_INSTANCE_ORG_ID — guard is skipped.
    let mut clone = ArmisClone::new().expect("ArmisClone::new failed");
    clone.start().await.expect("ArmisClone::start failed");
    let base_url = clone.base_url();

    let resp = http_client()
        .delete(format!(
            "{base_url}/api/v1/devices/device-001/tags/quarantine"
        ))
        .header("Authorization", "Bearer test-token")
        // No X-Org-Id header — default-instance clone must skip the guard.
        .send()
        .await
        .expect("CR-017 delete backward compat: request must not error");

    // Expect 404 (tag not found) or 200 — anything other than 401 confirms backward compat.
    let status = resp.status().as_u16();
    assert_ne!(
        status, 401,
        "CR-017 EC-008: default-instance clone must NOT return 401 for DELETE \
         /api/v1/devices/{{id}}/tags/{{key}} with absent X-Org-Id; \
         got 401 — guard must not fire on default-instance clones (BC-3.5.002 precondition 3)"
    );
}

// ===========================================================================
// get_alerts — BC-3.5.002 precondition 3; CR-017 AC-002
// ===========================================================================

/// CR-017 AC-002: `GET /api/v1/alerts` on a real-org Armis clone must
/// reject a request that omits the `X-Org-Id` header with HTTP 401, before any
/// alert data is exposed.
///
/// This test FAILS before the guard is added to `alerts.rs::get_alerts`.
///
/// (BC-3.5.002 precondition 3; BC-3.2.001 precondition 4; CR-017 AC-002 EC-004)
#[tokio::test]
async fn test_get_alerts_real_org_absent_header_returns_401() {
    let org_id = real_org_id();
    let mut clone = ArmisClone::new_with_org(org_id).expect("ArmisClone::new_with_org failed");
    clone.start().await.expect("ArmisClone::start failed");
    let base_url = clone.base_url();

    let resp = http_client()
        .get(format!("{base_url}/api/v1/alerts"))
        .header("Authorization", "Bearer test-token")
        // No X-Org-Id header — instance-identity guard must reject this.
        .send()
        .await
        .expect("CR-017 alerts: request must not error at network level");

    assert_eq!(
        resp.status().as_u16(),
        401,
        "CR-017 AC-002: real-org clone must reject GET /api/v1/alerts \
         with absent X-Org-Id → HTTP 401; \
         expected 401 got {} (BC-3.5.002 precondition 3; CR-017 AC-002 EC-004)",
        resp.status().as_u16()
    );
}

/// CR-017 AC-002 positive path: `GET /api/v1/alerts` with the correct `X-Org-Id`
/// on a real-org clone must succeed with HTTP 200.
///
/// (BC-3.2.001 precondition 4; CR-017 AC-002)
#[tokio::test]
async fn test_get_alerts_real_org_correct_header_returns_200() {
    let org_id = real_org_id();
    let mut clone = ArmisClone::new_with_org(org_id).expect("ArmisClone::new_with_org failed");
    clone.start().await.expect("ArmisClone::start failed");
    let base_url = clone.base_url();

    let resp = http_client()
        .get(format!("{base_url}/api/v1/alerts"))
        .header("Authorization", "Bearer test-token")
        .header("X-Org-Id", org_id.as_uuid().to_string())
        .send()
        .await
        .expect("CR-017 alerts positive: request must not error");

    assert_eq!(
        resp.status().as_u16(),
        200,
        "CR-017 AC-002 positive: real-org clone must accept GET /api/v1/alerts \
         with correct X-Org-Id → HTTP 200; \
         expected 200 got {} (BC-3.2.001 precondition 4; CR-017 AC-002)",
        resp.status().as_u16()
    );
}

/// CR-017 AC-002 backward compat: `GET /api/v1/alerts` on a default-instance clone
/// must allow absent `X-Org-Id` → HTTP 200 (backward compat).
///
/// (BC-3.5.002 precondition 3; CR-017 EC-008)
#[tokio::test]
async fn test_get_alerts_default_instance_absent_header_returns_200() {
    // ArmisClone::new() uses DTU_DEFAULT_INSTANCE_ORG_ID — guard is skipped.
    let mut clone = ArmisClone::new().expect("ArmisClone::new failed");
    clone.start().await.expect("ArmisClone::start failed");
    let base_url = clone.base_url();

    let resp = http_client()
        .get(format!("{base_url}/api/v1/alerts"))
        .header("Authorization", "Bearer test-token")
        // No X-Org-Id header — default-instance clone must skip the guard.
        .send()
        .await
        .expect("CR-017 alerts backward compat: request must not error");

    assert_eq!(
        resp.status().as_u16(),
        200,
        "CR-017 EC-008: default-instance clone must allow GET /api/v1/alerts \
         with absent X-Org-Id → HTTP 200 (backward compat); \
         expected 200 got {} (BC-3.5.002 precondition 3; CR-017 EC-008)",
        resp.status().as_u16()
    );
}
