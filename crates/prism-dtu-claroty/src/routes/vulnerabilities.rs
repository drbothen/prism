//! Route handlers for vulnerability endpoints.
//!
//! `POST /api/v1/vulnerabilities` — vulnerability inventory.
//! `POST /api/v1/vulnerabilities/{vuln_id}/devices` — devices affected by a vulnerability.
//!
//! # W3-FIX-SEC-001 (AC-001..AC-003)
//!
//! Both endpoints enforce the shared `validate_org_id` guard (from `routes::devices`)
//! when the clone was created with a real (non-nil) `instance_org_id`. Nil-org clones
//! (`ClarotyClone::new()`) skip header validation for backward compatibility.
//! This closes the org-isolation gap identified in O-PR3R2-001.

use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use prism_core::OrgId;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::{
    routes::devices::{check_bearer_auth, validate_org_id},
    state::ClarotyState,
    types::{GetVulnerabilitiesBody, GetVulnerabilityDevicesBody},
};

/// `POST /api/v1/vulnerabilities`
///
/// Returns vulnerability inventory from `fixtures/vulnerabilities.json`.
/// Response: `{"vulnerabilities": [...], "total": N, "page": N}`.
/// Requires valid `Authorization: Bearer` header (AC-5).
///
/// # W3-FIX-SEC-001 (AC-001..AC-003)
///
/// When the clone was created with a real (non-nil) `instance_org_id`, the
/// `X-Org-Id` header is validated against `state.instance_org_id` after bearer
/// auth — matching the guard used by `list_devices`, `list_alerts`, and
/// `list_audit_logs`. Nil-org clones skip header validation for backward
/// compatibility.
pub async fn list_vulnerabilities(
    State(state): State<Arc<ClarotyState>>,
    headers: HeaderMap,
    _body: Option<Json<GetVulnerabilitiesBody>>,
) -> (StatusCode, Json<Value>) {
    if let Err(err) = check_bearer_auth(&headers) {
        return err;
    }

    // W3-FIX-SEC-001: org-isolation guard (O-PR3R2-001 closure).
    //
    // When instance_org_id is non-nil (real org clone), validate the X-Org-Id
    // header matches. Nil-org clones (ClarotyClone::new()) skip this check for
    // backward compat — nil UUID is the sentinel meaning "no org constraint".
    let nil_org = OrgId::from_uuid(Uuid::nil());
    if state.instance_org_id != nil_org {
        if let Err(err) = validate_org_id(&headers, state.instance_org_id) {
            return err;
        }
    }

    // SAFETY: fixture files are bundled at build time; missing fixture is a build error, not runtime condition.
    #[allow(clippy::expect_used)]
    let raw = prism_dtu_common::load_fixture(env!("CARGO_MANIFEST_DIR"), "vulnerabilities")
        .expect("fixtures/vulnerabilities.json must exist");
    // SAFETY: fixture content is a well-formed JSON array validated at CI time.
    #[allow(clippy::expect_used)]
    let vulns = raw
        .as_array()
        .expect("vulnerabilities fixture must be a JSON array")
        .clone();
    let total = vulns.len() as u32;

    (
        StatusCode::OK,
        Json(json!({"vulnerabilities": vulns, "total": total, "page": 1u32})),
    )
}

/// `POST /api/v1/vulnerabilities/{vuln_id}/devices`
///
/// Returns devices affected by the specified vulnerability from
/// `fixtures/vulnerability-devices.json`.
/// Response: `{"devices": [...], "total": N}`.
/// Requires valid `Authorization: Bearer` header (AC-5).
///
/// # W3-FIX-SEC-001 (AC-001..AC-003)
///
/// When the clone was created with a real (non-nil) `instance_org_id`, the
/// `X-Org-Id` header is validated against `state.instance_org_id` after bearer
/// auth — matching the guard used by all other org-scoped fixture-list endpoints.
/// Nil-org clones skip header validation for backward compatibility.
pub async fn list_vulnerability_devices(
    State(state): State<Arc<ClarotyState>>,
    Path(_vuln_id): Path<String>,
    headers: HeaderMap,
    _body: Option<Json<GetVulnerabilityDevicesBody>>,
) -> (StatusCode, Json<Value>) {
    if let Err(err) = check_bearer_auth(&headers) {
        return err;
    }

    // W3-FIX-SEC-001: org-isolation guard (O-PR3R2-001 closure).
    //
    // When instance_org_id is non-nil (real org clone), validate the X-Org-Id
    // header matches. Nil-org clones (ClarotyClone::new()) skip this check for
    // backward compat — nil UUID is the sentinel meaning "no org constraint".
    let nil_org = OrgId::from_uuid(Uuid::nil());
    if state.instance_org_id != nil_org {
        if let Err(err) = validate_org_id(&headers, state.instance_org_id) {
            return err;
        }
    }

    // SAFETY: fixture files are bundled at build time; missing fixture is a build error, not runtime condition.
    #[allow(clippy::expect_used)]
    let raw = prism_dtu_common::load_fixture(env!("CARGO_MANIFEST_DIR"), "vulnerability-devices")
        .expect("fixtures/vulnerability-devices.json must exist");
    // SAFETY: fixture content is a well-formed JSON array validated at CI time.
    #[allow(clippy::expect_used)]
    let devices = raw
        .as_array()
        .expect("vulnerability-devices fixture must be a JSON array")
        .clone();
    let total = devices.len() as u32;

    (
        StatusCode::OK,
        Json(json!({"devices": devices, "total": total})),
    )
}

// ---------------------------------------------------------------------------
// Tests — W3-FIX-SEC-001 org-isolation guard for vulnerabilities endpoints
//
// O-PR3R2-001: list_vulnerabilities and list_vulnerability_devices were missing
// the X-Org-Id guard entirely. This closes the gap with a complete 3-cell matrix
// per endpoint.
//
// AC traces:
//   test_W3_FIX_SEC_001_claroty_vulnerabilities_org_mismatch_returns_401
//     → Cell A: non-nil clone + MISMATCHED X-Org-Id → 401
//   test_W3_FIX_SEC_001_claroty_vulnerabilities_missing_org_header_on_real_org_returns_401
//     → Cell B: non-nil clone + ABSENT X-Org-Id → 401
//   test_W3_FIX_SEC_001_claroty_vulnerabilities_nil_org_no_header_returns_200
//     → Cell C: nil-org clone + ABSENT header → 200
//   test_W3_FIX_SEC_001_claroty_vulnerability_devices_org_mismatch_returns_401
//     → Cell A: non-nil clone + MISMATCHED X-Org-Id → 401
//   test_W3_FIX_SEC_001_claroty_vulnerability_devices_missing_org_header_on_real_org_returns_401
//     → Cell B: non-nil clone + ABSENT X-Org-Id → 401
//   test_W3_FIX_SEC_001_claroty_vulnerability_devices_nil_org_no_header_returns_200
//     → Cell C: nil-org clone + ABSENT header → 200
// ---------------------------------------------------------------------------
#[cfg(test)]
#[allow(non_snake_case, clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use std::time::Duration;

    use prism_core::OrgId;
    use prism_dtu_common::BehavioralClone;
    use serde_json::json;
    use uuid::Uuid;

    use crate::clone::ClarotyClone;

    fn test_client() -> reqwest::Client {
        reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("test client build must succeed")
    }

    async fn start_clone() -> (ClarotyClone, String) {
        let mut clone = ClarotyClone::new();
        clone
            .start()
            .await
            .expect("ClarotyClone::start must succeed in test environment");
        let base_url = clone.base_url();
        (clone, base_url)
    }

    async fn start_clone_with_org(org_id: OrgId) -> (ClarotyClone, String) {
        let mut clone = ClarotyClone::with_org(org_id);
        clone
            .start()
            .await
            .expect("ClarotyClone::start must succeed in test environment");
        let base_url = clone.base_url();
        (clone, base_url)
    }

    // -----------------------------------------------------------------------
    // list_vulnerabilities — 3-cell matrix
    // -----------------------------------------------------------------------

    /// W3-FIX-SEC-001 — Cell A: non-nil clone + mismatched X-Org-Id → 401.
    ///
    /// `list_vulnerabilities` now enforces the shared `validate_org_id` guard
    /// (O-PR3R2-001 closure). A request whose X-Org-Id does not match the clone's
    /// `instance_org_id` must be rejected with HTTP 401.
    #[tokio::test]
    async fn test_W3_FIX_SEC_001_claroty_vulnerabilities_org_mismatch_returns_401() {
        let instance_org = OrgId::from_uuid(
            Uuid::parse_str("11111111-1111-7000-8000-000000000001").expect("valid test UUID"),
        );
        let caller_org =
            Uuid::parse_str("22222222-2222-7000-8000-000000000002").expect("valid test UUID");

        let (_clone, base_url) = start_clone_with_org(instance_org).await;
        let client = test_client();

        let resp = client
            .post(format!("{base_url}/api/v1/vulnerabilities"))
            .header("Authorization", "Bearer test-token")
            .header("X-Org-Id", caller_org.to_string())
            .json(&json!({}))
            .send()
            .await
            .expect("transport must not fail for org-mismatch test");

        assert_eq!(
            resp.status().as_u16(),
            401,
            "non-nil-org clone + mismatched X-Org-Id must return HTTP 401 (W3-FIX-SEC-001)"
        );

        let body: serde_json::Value = resp
            .json()
            .await
            .expect("401 response must have JSON body (W3-FIX-SEC-001)");
        assert!(
            body.get("error")
                .and_then(|v| v.as_str())
                .map(|s| s.contains("org_id mismatch"))
                .unwrap_or(false),
            "W3-FIX-SEC-001: 401 body must contain 'org_id mismatch'; got: {body}"
        );
    }

    /// W3-FIX-SEC-001 — Cell B: non-nil clone + absent X-Org-Id header → 401.
    ///
    /// `validate_org_id` treats an absent header as a mismatch (AC-003: missing
    /// header → 401). This cell was entirely absent before O-PR3R2-001 closure.
    #[tokio::test]
    async fn test_W3_FIX_SEC_001_claroty_vulnerabilities_missing_org_header_on_real_org_returns_401(
    ) {
        let instance_org = OrgId::from_uuid(
            Uuid::parse_str("11111111-1111-7000-8000-000000000001").expect("valid test UUID"),
        );

        let (_clone, base_url) = start_clone_with_org(instance_org).await;
        let client = test_client();

        let resp = client
            .post(format!("{base_url}/api/v1/vulnerabilities"))
            .header("Authorization", "Bearer test-token")
            // Intentionally NO X-Org-Id header.
            .json(&json!({}))
            .send()
            .await
            .expect("transport must not fail for missing-header test");

        assert_eq!(
            resp.status().as_u16(),
            401,
            "non-nil-org clone + absent X-Org-Id header must return HTTP 401 (W3-FIX-SEC-001 AC-003)"
        );

        let body: serde_json::Value = resp
            .json()
            .await
            .expect("401 response must have JSON body (W3-FIX-SEC-001)");
        assert!(
            body.get("error")
                .and_then(|v| v.as_str())
                .map(|s| s.contains("org_id mismatch"))
                .unwrap_or(false),
            "W3-FIX-SEC-001: 401 body must contain 'org_id mismatch'; got: {body}"
        );
    }

    /// W3-FIX-SEC-001 — Cell C: nil-org clone + absent header → 200 (backward-compat).
    ///
    /// `ClarotyClone::new()` sets `instance_org_id` to the nil UUID — the sentinel
    /// meaning "no org constraint". Callers that do not supply `X-Org-Id` must not
    /// be rejected.
    #[tokio::test]
    async fn test_W3_FIX_SEC_001_claroty_vulnerabilities_nil_org_no_header_returns_200() {
        let (_clone, base_url) = start_clone().await;
        let client = test_client();

        let resp = client
            .post(format!("{base_url}/api/v1/vulnerabilities"))
            .header("Authorization", "Bearer test-token")
            // Intentionally NO X-Org-Id header.
            .json(&json!({}))
            .send()
            .await
            .expect("transport must not fail for nil-org backward-compat test");

        assert_eq!(
            resp.status().as_u16(),
            200,
            "nil-org clone without X-Org-Id header must return HTTP 200 (W3-FIX-SEC-001 backward-compat)"
        );
    }

    // -----------------------------------------------------------------------
    // list_vulnerability_devices — 3-cell matrix
    // -----------------------------------------------------------------------

    /// W3-FIX-SEC-001 — Cell A: non-nil clone + mismatched X-Org-Id → 401.
    ///
    /// `list_vulnerability_devices` now enforces the shared `validate_org_id` guard
    /// (O-PR3R2-001 closure). A request whose X-Org-Id does not match the clone's
    /// `instance_org_id` must be rejected with HTTP 401.
    #[tokio::test]
    async fn test_W3_FIX_SEC_001_claroty_vulnerability_devices_org_mismatch_returns_401() {
        let instance_org = OrgId::from_uuid(
            Uuid::parse_str("11111111-1111-7000-8000-000000000001").expect("valid test UUID"),
        );
        let caller_org =
            Uuid::parse_str("22222222-2222-7000-8000-000000000002").expect("valid test UUID");

        let (_clone, base_url) = start_clone_with_org(instance_org).await;
        let client = test_client();

        let resp = client
            .post(format!(
                "{base_url}/api/v1/vulnerabilities/vuln-001/devices"
            ))
            .header("Authorization", "Bearer test-token")
            .header("X-Org-Id", caller_org.to_string())
            .json(&json!({}))
            .send()
            .await
            .expect("transport must not fail for org-mismatch test");

        assert_eq!(
            resp.status().as_u16(),
            401,
            "non-nil-org clone + mismatched X-Org-Id must return HTTP 401 (W3-FIX-SEC-001)"
        );

        let body: serde_json::Value = resp
            .json()
            .await
            .expect("401 response must have JSON body (W3-FIX-SEC-001)");
        assert!(
            body.get("error")
                .and_then(|v| v.as_str())
                .map(|s| s.contains("org_id mismatch"))
                .unwrap_or(false),
            "W3-FIX-SEC-001: 401 body must contain 'org_id mismatch'; got: {body}"
        );
    }

    /// W3-FIX-SEC-001 — Cell B: non-nil clone + absent X-Org-Id header → 401.
    ///
    /// `validate_org_id` treats an absent header as a mismatch (AC-003: missing
    /// header → 401). This cell was entirely absent before O-PR3R2-001 closure.
    #[tokio::test]
    async fn test_W3_FIX_SEC_001_claroty_vulnerability_devices_missing_org_header_on_real_org_returns_401(
    ) {
        let instance_org = OrgId::from_uuid(
            Uuid::parse_str("11111111-1111-7000-8000-000000000001").expect("valid test UUID"),
        );

        let (_clone, base_url) = start_clone_with_org(instance_org).await;
        let client = test_client();

        let resp = client
            .post(format!(
                "{base_url}/api/v1/vulnerabilities/vuln-001/devices"
            ))
            .header("Authorization", "Bearer test-token")
            // Intentionally NO X-Org-Id header.
            .json(&json!({}))
            .send()
            .await
            .expect("transport must not fail for missing-header test");

        assert_eq!(
            resp.status().as_u16(),
            401,
            "non-nil-org clone + absent X-Org-Id header must return HTTP 401 (W3-FIX-SEC-001 AC-003)"
        );

        let body: serde_json::Value = resp
            .json()
            .await
            .expect("401 response must have JSON body (W3-FIX-SEC-001)");
        assert!(
            body.get("error")
                .and_then(|v| v.as_str())
                .map(|s| s.contains("org_id mismatch"))
                .unwrap_or(false),
            "W3-FIX-SEC-001: 401 body must contain 'org_id mismatch'; got: {body}"
        );
    }

    /// W3-FIX-SEC-001 — Cell C: nil-org clone + absent header → 200 (backward-compat).
    ///
    /// `ClarotyClone::new()` sets `instance_org_id` to the nil UUID — the sentinel
    /// meaning "no org constraint". Callers that do not supply `X-Org-Id` must not
    /// be rejected.
    #[tokio::test]
    async fn test_W3_FIX_SEC_001_claroty_vulnerability_devices_nil_org_no_header_returns_200() {
        let (_clone, base_url) = start_clone().await;
        let client = test_client();

        let resp = client
            .post(format!(
                "{base_url}/api/v1/vulnerabilities/vuln-001/devices"
            ))
            .header("Authorization", "Bearer test-token")
            // Intentionally NO X-Org-Id header.
            .json(&json!({}))
            .send()
            .await
            .expect("transport must not fail for nil-org backward-compat test");

        assert_eq!(
            resp.status().as_u16(),
            200,
            "nil-org clone without X-Org-Id header must return HTTP 200 (W3-FIX-SEC-001 backward-compat)"
        );
    }
}
