//! CR-018 — CrowdStrike `validate_org_id` nil-instance guard: detection endpoints.
//!
//! Verifies that the nil-instance guard (applied to `hosts.rs` and `writes.rs` by
//! W3-FIX-SEC-001) is now enforced on the two detection route handlers:
//!
//!   - `GET /detects/queries/detects/v1`  (`list_detection_ids`)
//!   - `POST /detects/entities/summaries/GET/v1`  (`get_detection_summaries`)
//!
//! Guard semantics (identical to `hosts.rs:146-150`):
//!   - Real-org clones (`instance_org_id != OrgId::from_uuid(Uuid::nil())`): absent or
//!     mismatched `X-Org-Id` → HTTP 401.  Matching `X-Org-Id` → HTTP 200.
//!   - Nil-instance clones (`instance_org_id == Uuid::nil()`): guard is a no-op;
//!     requests without `X-Org-Id` proceed normally (backward compat, EC-007).
//!
//! # Behavioral contracts exercised
//!
//!   BC-3.5.002 precondition 3 — `instance_org_id` guard on all org-keyed endpoints.
//!   BC-3.2.001 precondition 4 — HTTP layer enforces OrgId boundary before state access.
//!
//! # Architecture compliance note
//!
//! CrowdStrike MUST use `OrgId::from_uuid(uuid::Uuid::nil())` as the nil sentinel —
//! NOT `crate::state::DTU_DEFAULT_INSTANCE_ORG_ID` (Armis-specific non-nil sentinel).
//! Mixing the two sentinel constants is a correctness error per the story architecture
//! compliance rules.
//!
//! # Naming convention
//!
//! Tests follow `test_<handler>_<condition>_<result>` consistent with
//! `x_org_id_auth.rs` and `cr012_validate_org_id_consistency.rs`.

#![allow(clippy::expect_used, clippy::unwrap_used, non_snake_case)]
#![cfg(feature = "dtu")]

use std::sync::Arc;

use prism_core::OrgId;
use prism_dtu_common::BehavioralClone;
use prism_dtu_crowdstrike::{CrowdstrikeClone, CrowdstrikeState};

// ---------------------------------------------------------------------------
// Test helpers (mirrors x_org_id_auth.rs structure)
// ---------------------------------------------------------------------------

fn http_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .expect("reqwest::Client build")
}

/// A deterministic non-nil OrgId for real-org clone tests.
fn real_org_id() -> OrgId {
    OrgId::from_uuid(
        uuid::Uuid::parse_str("00000000-0000-7000-8000-0000000000CC").expect("valid uuid"),
    )
}

/// Start a CrowdStrike clone whose `instance_org_id` is bound to `org_id`.
///
/// Replicates the `start_clone_with_org` helper from `x_org_id_auth.rs` so this file
/// is self-contained and mirrors the established pattern.
async fn start_clone_with_org(org_id: OrgId) -> (CrowdstrikeClone, String) {
    let mut clone = CrowdstrikeClone::new();
    let token = clone.admin_token().to_string();
    clone.state = Arc::new(CrowdstrikeState::with_admin_token_and_org(token, org_id));
    clone.start().await.expect("CrowdstrikeClone::start failed");
    let base_url = clone.base_url();
    (clone, base_url)
}

// ===========================================================================
// list_detection_ids — BC-3.5.002 precondition 3; CR-018 AC-003
// ===========================================================================

/// CR-018 AC-003: `GET /detects/queries/detects/v1` on a real-org CrowdStrike clone must
/// reject a request that omits the `X-Org-Id` header with HTTP 401, before the
/// detection store is queried.
///
/// This test FAILS before the nil-instance guard is added to
/// `detections.rs::list_detection_ids`.
///
/// (BC-3.5.002 precondition 3; BC-3.2.001 precondition 4; CR-018 AC-003 EC-005)
#[tokio::test]
async fn test_list_detection_ids_real_org_absent_header_returns_401() {
    let (_clone, base_url) = start_clone_with_org(real_org_id()).await;

    let resp = http_client()
        .get(format!("{base_url}/detects/queries/detects/v1"))
        .header("Authorization", "Bearer test-token")
        // No X-Org-Id header — nil-instance guard must reject this.
        .send()
        .await
        .expect("CR-018: request must not error at network level");

    assert_eq!(
        resp.status().as_u16(),
        401,
        "CR-018 AC-003: real-org clone must reject GET /detects/queries/detects/v1 \
         with absent X-Org-Id → HTTP 401; \
         expected 401 got {} (BC-3.5.002 precondition 3; CR-018 AC-003 EC-005)",
        resp.status().as_u16()
    );
}

/// CR-018 AC-003 positive path: `GET /detects/queries/detects/v1` with the correct
/// `X-Org-Id` on a real-org clone must succeed with HTTP 200.
///
/// (BC-3.2.001 precondition 4; CR-018 AC-003)
#[tokio::test]
async fn test_list_detection_ids_real_org_correct_header_returns_200() {
    let org_id = real_org_id();
    let (_clone, base_url) = start_clone_with_org(org_id).await;

    let resp = http_client()
        .get(format!("{base_url}/detects/queries/detects/v1"))
        .header("Authorization", "Bearer test-token")
        .header("X-Org-Id", org_id.as_uuid().to_string())
        .send()
        .await
        .expect("CR-018 positive: request must not error");

    assert_eq!(
        resp.status().as_u16(),
        200,
        "CR-018 AC-003 positive: real-org clone must accept GET /detects/queries/detects/v1 \
         with correct X-Org-Id → HTTP 200; \
         expected 200 got {} (BC-3.2.001 precondition 4; CR-018 AC-003)",
        resp.status().as_u16()
    );
}

/// CR-018 EC-007 backward compat: `GET /detects/queries/detects/v1` on a nil-instance
/// clone (`CrowdstrikeClone::new()`) must allow absent `X-Org-Id` → HTTP 200.
///
/// The guard condition `instance_org_id != Uuid::nil()` is false for nil-instance
/// clones, so the guard is skipped entirely.
///
/// (CR-018 EC-007; BC-3.5.002 precondition 3)
#[tokio::test]
async fn test_list_detection_ids_nil_instance_absent_header_returns_200() {
    // CrowdstrikeClone::new() has a nil instance_org_id — guard must be a no-op.
    let mut clone = CrowdstrikeClone::new();
    clone.start().await.expect("CrowdstrikeClone::start failed");
    let base_url = clone.base_url();

    let resp = http_client()
        .get(format!("{base_url}/detects/queries/detects/v1"))
        .header("Authorization", "Bearer test-token")
        // No X-Org-Id header — nil-instance clone must skip the guard.
        .send()
        .await
        .expect("CR-018 nil backward compat: request must not error");

    assert_eq!(
        resp.status().as_u16(),
        200,
        "CR-018 EC-007: nil-instance clone must allow GET /detects/queries/detects/v1 \
         with absent X-Org-Id → HTTP 200 (backward compat); \
         expected 200 got {} (BC-3.5.002 precondition 3; CR-018 EC-007)",
        resp.status().as_u16()
    );
}

// ===========================================================================
// get_detection_summaries — BC-3.5.002 precondition 3; CR-018 AC-003
// ===========================================================================

/// CR-018 AC-003: `POST /detects/entities/summaries/GET/v1` on a real-org CrowdStrike
/// clone must reject a request that omits the `X-Org-Id` header with HTTP 401, before
/// the detection store is queried.
///
/// This test FAILS before the nil-instance guard is added to
/// `detections.rs::get_detection_summaries`.
///
/// (BC-3.5.002 precondition 3; BC-3.2.001 precondition 4; CR-018 AC-003 EC-006)
#[tokio::test]
async fn test_get_detection_summaries_real_org_absent_header_returns_401() {
    let (_clone, base_url) = start_clone_with_org(real_org_id()).await;

    let resp = http_client()
        .post(format!("{base_url}/detects/entities/summaries/GET/v1"))
        .header("Authorization", "Bearer test-token")
        // No X-Org-Id header — nil-instance guard must reject this.
        .json(&serde_json::json!({"ids": ["det-001"]}))
        .send()
        .await
        .expect("CR-018 summaries: request must not error at network level");

    assert_eq!(
        resp.status().as_u16(),
        401,
        "CR-018 AC-003: real-org clone must reject \
         POST /detects/entities/summaries/GET/v1 \
         with absent X-Org-Id → HTTP 401; \
         expected 401 got {} (BC-3.5.002 precondition 3; CR-018 AC-003 EC-006)",
        resp.status().as_u16()
    );
}

/// CR-018 AC-003 positive path: `POST /detects/entities/summaries/GET/v1` with the
/// correct `X-Org-Id` on a real-org clone must succeed with HTTP 200.
///
/// (BC-3.2.001 precondition 4; CR-018 AC-003)
#[tokio::test]
async fn test_get_detection_summaries_real_org_correct_header_returns_200() {
    let org_id = real_org_id();
    let (_clone, base_url) = start_clone_with_org(org_id).await;

    let resp = http_client()
        .post(format!("{base_url}/detects/entities/summaries/GET/v1"))
        .header("Authorization", "Bearer test-token")
        .header("X-Org-Id", org_id.as_uuid().to_string())
        .json(&serde_json::json!({"ids": ["det-001"]}))
        .send()
        .await
        .expect("CR-018 summaries positive: request must not error");

    assert_eq!(
        resp.status().as_u16(),
        200,
        "CR-018 AC-003 positive: real-org clone must accept \
         POST /detects/entities/summaries/GET/v1 \
         with correct X-Org-Id → HTTP 200; \
         expected 200 got {} (BC-3.2.001 precondition 4; CR-018 AC-003)",
        resp.status().as_u16()
    );
}

/// CR-018 EC-007 backward compat: `POST /detects/entities/summaries/GET/v1` on a
/// nil-instance clone must allow absent `X-Org-Id` → HTTP 200.
///
/// (CR-018 EC-007; BC-3.5.002 precondition 3)
#[tokio::test]
async fn test_get_detection_summaries_nil_instance_absent_header_returns_200() {
    // CrowdstrikeClone::new() has a nil instance_org_id — guard must be a no-op.
    let mut clone = CrowdstrikeClone::new();
    clone.start().await.expect("CrowdstrikeClone::start failed");
    let base_url = clone.base_url();

    let resp = http_client()
        .post(format!("{base_url}/detects/entities/summaries/GET/v1"))
        .header("Authorization", "Bearer test-token")
        // No X-Org-Id header — nil-instance clone must skip the guard.
        .json(&serde_json::json!({"ids": ["det-001"]}))
        .send()
        .await
        .expect("CR-018 summaries nil backward compat: request must not error");

    assert_eq!(
        resp.status().as_u16(),
        200,
        "CR-018 EC-007: nil-instance clone must allow \
         POST /detects/entities/summaries/GET/v1 \
         with absent X-Org-Id → HTTP 200 (backward compat); \
         expected 200 got {} (BC-3.5.002 precondition 3; CR-018 EC-007)",
        resp.status().as_u16()
    );
}
