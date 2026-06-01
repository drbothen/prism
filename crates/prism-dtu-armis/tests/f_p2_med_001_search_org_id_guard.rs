//! F-P2-MED-001 — X-Org-Id tenant-isolation guard coverage for `GET /api/v1/search`.
//!
//! Finding: the dual-mode instance-identity guard added to `routes/search.rs` as part of
//! the TD-VSDD-060 sibling-sweep (devices.rs + alerts.rs → search.rs) had zero test
//! coverage. Sibling endpoints (`/api/v1/devices`, `/api/v1/alerts`) have this coverage
//! in `cr012_validate_org_id_consistency.rs`; the search endpoint did not.
//!
//! # Guard logic under test (search.rs lines 119-124)
//!
//! ```text
//! let is_real_org = state.instance_org_id != DTU_DEFAULT_INSTANCE_ORG_ID;
//! if is_real_org || headers.get("x-org-id").is_some() {
//!     if let Err((status, body)) = validate_org_id(&headers, state.instance_org_id) {
//!         return (status, body).into_response();
//!     }
//! }
//! ```
//!
//! When `is_real_org`:
//! - Absent `X-Org-Id` → 401 (BC-3.5.002 precondition 3)
//! - Matching `X-Org-Id` → 200 (BC-3.2.001 postcondition 1)
//! - Mismatched `X-Org-Id` → 401 (BC-3.5.002 precondition 3)
//!
//! # Behavioral contracts exercised
//!
//! - BC-3.5.002 precondition 3 — org-id header must match clone's instance_org_id.
//! - BC-3.2.001 postcondition 1 — same-org request returns the expected payload.
//!
//! # Acceptance criteria map
//!
//! | Test | Assertion | Finding |
//! |------|-----------|---------|
//! | `test_W3_FIX_SEC_001_search_real_org_absent_header_returns_401` | 401 on absent header | F-P2-MED-001 (a) |
//! | `test_W3_FIX_SEC_001_search_real_org_matching_header_returns_200` | 200 on matching header | F-P2-MED-001 (b) |
//! | `test_W3_FIX_SEC_001_search_real_org_mismatched_header_returns_401` | 401 on mismatch | F-P2-MED-001 (c) |

#![allow(clippy::expect_used, clippy::unwrap_used, non_snake_case)]
#![cfg(feature = "dtu")]

use prism_core::OrgId;
use prism_dtu_armis::ArmisClone;
use prism_dtu_common::BehavioralClone;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// A real (non-default) OrgId that triggers the instance-identity guard.
///
/// Must differ from `DTU_DEFAULT_INSTANCE_ORG_ID`
/// (`00000000-0000-7000-8000-0000000000AA`) to ensure `is_real_org == true`.
fn real_org_id() -> OrgId {
    OrgId(uuid::uuid!("00000000-0000-7000-8000-0000000000EE"))
}

/// A foreign OrgId — guaranteed to differ from `real_org_id()`.
fn foreign_org_uuid() -> &'static str {
    "00000000-0000-7000-8000-0000000000FF"
}

fn http_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .expect("reqwest::Client build")
}

// ---------------------------------------------------------------------------
// F-P2-MED-001 (a) — Real-org clone + absent X-Org-Id header → 401
//
// BC-3.5.002 precondition 3: missing X-Org-Id on a real-org clone must be rejected.
// ---------------------------------------------------------------------------

/// F-P2-MED-001 (a) / BC-3.5.002 precondition 3:
///
/// A real-org clone (`instance_org_id != DTU_DEFAULT_INSTANCE_ORG_ID`) processing
/// `GET /api/v1/search` with NO `X-Org-Id` header must return HTTP 401.
///
/// This exercises the `is_real_org` branch of the guard (search.rs ~119-124):
/// absent header is not skipped when `is_real_org` is true — the guard fires,
/// `validate_org_id` sees no header, and returns 401.
///
/// Traces to: BC-3.5.002 precondition 3; F-P2-MED-001 (a); CR-012/SEC-P2-001.
#[tokio::test]
async fn test_W3_FIX_SEC_001_search_real_org_absent_header_returns_401() {
    let org_id = real_org_id();
    let mut clone = ArmisClone::new_with_org(org_id).expect("ArmisClone::new_with_org failed");
    clone.start().await.expect("ArmisClone::start failed");
    let base_url = clone.base_url();

    let resp = http_client()
        .get(format!("{base_url}/api/v1/search"))
        .header("Authorization", "Bearer test-token")
        // No X-Org-Id header — instance-identity guard must reject this.
        .send()
        .await
        .expect("F-P2-MED-001 (a): request must not error at network level");

    assert_eq!(
        resp.status().as_u16(),
        401,
        "F-P2-MED-001 (a): GET /api/v1/search on real-org clone without X-Org-Id must \
         return HTTP 401; got {} (BC-3.5.002 precondition 3; CR-012/SEC-P2-001 guard on search.rs)",
        resp.status().as_u16()
    );
}

// ---------------------------------------------------------------------------
// F-P2-MED-001 (b) — Real-org clone + matching X-Org-Id header → 200
//
// BC-3.2.001 postcondition 1: same-org request passes the guard and returns data.
// ---------------------------------------------------------------------------

/// F-P2-MED-001 (b) / BC-3.2.001 postcondition 1:
///
/// A real-org clone processing `GET /api/v1/search` with the CORRECT `X-Org-Id`
/// must pass the guard and return HTTP 200 with a search result payload.
///
/// Verifies the guard does not over-reject: correct tenant identity succeeds.
///
/// Traces to: BC-3.2.001 postcondition 1; F-P2-MED-001 (b); CR-012/SEC-P2-001.
#[tokio::test]
async fn test_W3_FIX_SEC_001_search_real_org_matching_header_returns_200() {
    let org_id = real_org_id();
    let mut clone = ArmisClone::new_with_org(org_id).expect("ArmisClone::new_with_org failed");
    clone.start().await.expect("ArmisClone::start failed");
    let base_url = clone.base_url();

    let resp = http_client()
        .get(format!("{base_url}/api/v1/search"))
        .header("Authorization", "Bearer test-token")
        .header("X-Org-Id", org_id.as_uuid().to_string())
        .send()
        .await
        .expect("F-P2-MED-001 (b): request must not error");

    assert_eq!(
        resp.status().as_u16(),
        200,
        "F-P2-MED-001 (b): GET /api/v1/search on real-org clone with matching X-Org-Id must \
         return HTTP 200; got {} (BC-3.2.001 postcondition 1; CR-012/SEC-P2-001)",
        resp.status().as_u16()
    );

    // Verify the response body has the expected search envelope shape.
    let body: serde_json::Value = resp
        .json()
        .await
        .expect("F-P2-MED-001 (b): 200 response must be valid JSON");

    assert!(
        body.get("data").is_some(),
        "F-P2-MED-001 (b): search response must contain 'data' key; got: {body}"
    );
    assert!(
        body["data"].get("results").is_some(),
        "F-P2-MED-001 (b): search response data must contain 'results' key; got: {body}"
    );
}

// ---------------------------------------------------------------------------
// F-P2-MED-001 (c) — Real-org clone + mismatched X-Org-Id header → 401
//
// BC-3.5.002 precondition 3: cross-org credential is rejected.
// ---------------------------------------------------------------------------

/// F-P2-MED-001 (c) / BC-3.5.002 precondition 3:
///
/// A real-org clone processing `GET /api/v1/search` with a MISMATCHED `X-Org-Id`
/// (a different tenant's UUID) must return HTTP 401.
///
/// This exercises the mismatch branch of `validate_org_id` on the search endpoint.
///
/// Traces to: BC-3.5.002 precondition 3; F-P2-MED-001 (c); CR-012/SEC-P2-001.
#[tokio::test]
async fn test_W3_FIX_SEC_001_search_real_org_mismatched_header_returns_401() {
    let org_id = real_org_id();
    let mut clone = ArmisClone::new_with_org(org_id).expect("ArmisClone::new_with_org failed");
    clone.start().await.expect("ArmisClone::start failed");
    let base_url = clone.base_url();

    let resp = http_client()
        .get(format!("{base_url}/api/v1/search"))
        .header("Authorization", "Bearer test-token")
        .header("X-Org-Id", foreign_org_uuid())
        .send()
        .await
        .expect("F-P2-MED-001 (c): request must not error at network level");

    assert_eq!(
        resp.status().as_u16(),
        401,
        "F-P2-MED-001 (c): GET /api/v1/search on real-org clone with mismatched X-Org-Id must \
         return HTTP 401; got {} (BC-3.5.002 precondition 3; CR-012/SEC-P2-001)",
        resp.status().as_u16()
    );

    // Verify the 401 body is a JSON error object with 'error' key (not a silent rejection).
    let body: serde_json::Value = resp
        .json()
        .await
        .expect("F-P2-MED-001 (c): 401 response must be valid JSON");

    let error_msg = body["error"].as_str().unwrap_or("");
    assert!(
        error_msg.contains("org_id mismatch"),
        "F-P2-MED-001 (c): 401 body must contain 'org_id mismatch'; got: {error_msg:?}"
    );
}
