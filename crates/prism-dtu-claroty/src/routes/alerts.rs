//! Route handlers for alert endpoints.
//!
//! `POST /api/v1/alerts` — alert list with optional filter params.
//! `POST /api/v1/alerts/{alert_id}/devices` — devices associated with a specific alert.

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
    types::{GetAlertedDevicesBody, GetAlertsBody},
};

/// `POST /api/v1/alerts`
///
/// Returns alert list from `fixtures/alerts.json`.
/// Response: `{"alerts": [...], "total": N, "page": N}`.
/// Requires valid `Authorization: Bearer` header (AC-5).
///
/// # W3-FIX-SEC-001 (AC-001..AC-003)
///
/// When the clone was created with a real (non-nil) `instance_org_id`, the
/// `X-Org-Id` header is validated against `state.instance_org_id` after bearer
/// auth — matching the guard used by `list_devices` and `list_audit_logs`.
/// Nil-org clones skip header validation for backward compatibility.
pub async fn list_alerts(
    State(state): State<Arc<ClarotyState>>,
    headers: HeaderMap,
    _body: Option<Json<GetAlertsBody>>,
) -> (StatusCode, Json<Value>) {
    if let Err(err) = check_bearer_auth(&headers) {
        return err;
    }

    // W3-FIX-SEC-001: org-isolation guard.
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

    // F-P2-MED-001: dual-path — when the clone was built via new_with_seed (fixture_gen_seeded),
    // serve generated alert records as raw serde_json::Value.
    // Use fixture_gen_seeded (not generated_records.is_empty()) so DormantTenant (seeded=true,
    // 0 records) serves empty alerts — not static fixture. F-P6-HIGH-001 fix.
    // The adapter reads "$.alerts" response_path — the envelope structure is preserved.
    //
    // F3 / DTU-05 (review 2026-06-10): filter on the authoritative `_surface`
    // discriminator stamped by the generator — NOT key-presence heuristics like
    // `rec.get("alert_id").is_some()`, the exact fragile pattern behind Cyberint's
    // F-P3-CRIT-001 cross-surface leak. Mirrors prism-dtu-cyberint routes/alerts.rs
    // exactly (the tag is served as-is; Cyberint does not strip it either).
    #[cfg(feature = "fixture-gen")]
    if state.fixture_gen_seeded {
        let generated_alerts: Vec<serde_json::Value> = state
            .generated_records
            .iter()
            .filter(|rec| rec.get("_surface").and_then(|v| v.as_str()) == Some("alert"))
            .cloned()
            .collect();
        let total = generated_alerts.len() as u32;
        return (
            StatusCode::OK,
            Json(json!({"alerts": generated_alerts, "total": total, "page": 1u32})),
        );
    }

    // SAFETY: fixture files are bundled at build time; missing fixture is a build error, not runtime condition.
    #[allow(clippy::expect_used)]
    let raw = prism_dtu_common::load_fixture(env!("CARGO_MANIFEST_DIR"), "alerts")
        .expect("fixtures/alerts.json must exist");
    // SAFETY: fixture content is a well-formed JSON array validated at CI time.
    #[allow(clippy::expect_used)]
    let alerts = raw
        .as_array()
        .expect("alerts fixture must be a JSON array")
        .clone();
    let total = alerts.len() as u32;

    (
        StatusCode::OK,
        Json(json!({"alerts": alerts, "total": total, "page": 1u32})),
    )
}

/// `POST /api/v1/alerts/{alert_id}/devices`
///
/// Returns devices associated with the specified alert from
/// `fixtures/alerted-devices.json`.
/// Response: `{"devices": [...], "total": N}`.
/// Requires valid `Authorization: Bearer` header (AC-5).
///
/// # W3-FIX-SEC-001 (AC-001..AC-003)
///
/// When the clone was created with a real (non-nil) `instance_org_id`, the
/// `X-Org-Id` header is validated against `state.instance_org_id` after bearer
/// auth — matching the guard used by `list_devices` and `list_audit_logs`.
/// Nil-org clones skip header validation for backward compatibility.
pub async fn list_alerted_devices(
    State(state): State<Arc<ClarotyState>>,
    Path(_alert_id): Path<String>,
    headers: HeaderMap,
    _body: Option<Json<GetAlertedDevicesBody>>,
) -> (StatusCode, Json<Value>) {
    if let Err(err) = check_bearer_auth(&headers) {
        return err;
    }

    // W3-FIX-SEC-001: org-isolation guard.
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
    let raw = prism_dtu_common::load_fixture(env!("CARGO_MANIFEST_DIR"), "alerted-devices")
        .expect("fixtures/alerted-devices.json must exist");
    // SAFETY: fixture content is a well-formed JSON array validated at CI time.
    #[allow(clippy::expect_used)]
    let devices = raw
        .as_array()
        .expect("alerted-devices fixture must be a JSON array")
        .clone();
    let total = devices.len() as u32;

    (
        StatusCode::OK,
        Json(json!({"devices": devices, "total": total})),
    )
}

// ---------------------------------------------------------------------------
// Tests — W3-FIX-SEC-001 org-isolation guard for alerts endpoints
//
// O-PR3-001: list_alerts and list_alerted_devices enforce the same
// X-Org-Id org-isolation guard as list_devices and list_audit_logs.
//
// Complete 3-cell matrix per endpoint (F-PR3R2-MED-002 closure):
//
// AC traces (list_alerts):
//   test_W3_FIX_SEC_001_claroty_alerts_org_mismatch_returns_401
//     → Cell A: non-nil clone + MISMATCHED X-Org-Id → 401
//   test_W3_FIX_SEC_001_claroty_alerts_missing_org_header_on_real_org_returns_401
//     → Cell B: non-nil clone + ABSENT X-Org-Id → 401
//   test_W3_FIX_SEC_001_claroty_alerts_nil_org_no_header_returns_200
//     → Cell C: nil-org clone + ABSENT header → 200
//
// AC traces (list_alerted_devices):
//   test_W3_FIX_SEC_001_claroty_alerted_devices_org_mismatch_returns_401
//     → Cell A: non-nil clone + MISMATCHED X-Org-Id → 401
//   test_W3_FIX_SEC_001_claroty_alerted_devices_missing_org_header_on_real_org_returns_401
//     → Cell B: non-nil clone + ABSENT X-Org-Id → 401
//   test_W3_FIX_SEC_001_claroty_alerted_devices_nil_org_no_header_returns_200
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

    /// W3-FIX-SEC-001 — org-isolation guard for list_alerts (non-nil clone, mismatched org).
    ///
    /// When the clone has a real (non-nil) `instance_org_id`, a request whose
    /// `X-Org-Id` header does not match `instance_org_id` must return HTTP 401.
    ///
    /// Verifies:
    /// - Valid bearer + mismatched X-Org-Id → 401 `{"error": "org_id mismatch: ..."}`
    #[tokio::test]
    async fn test_W3_FIX_SEC_001_claroty_alerts_org_mismatch_returns_401() {
        let instance_org = OrgId::from_uuid(
            Uuid::parse_str("11111111-1111-7000-8000-000000000001").expect("valid test UUID"),
        );
        let caller_org =
            Uuid::parse_str("22222222-2222-7000-8000-000000000002").expect("valid test UUID");

        let (_clone, base_url) = start_clone_with_org(instance_org).await;
        let client = test_client();

        let resp = client
            .post(format!("{base_url}/api/v1/alerts"))
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

    /// W3-FIX-SEC-001 — Cell B: non-nil clone + absent X-Org-Id header → 401 for list_alerts.
    ///
    /// When the clone has a real (non-nil) `instance_org_id`, a request that omits
    /// the `X-Org-Id` header entirely must return HTTP 401. `validate_org_id` treats
    /// an absent header as a mismatch (AC-003: missing header → 401).
    ///
    /// This closes the org-isolation test matrix for `list_alerts`:
    ///   test_W3_FIX_SEC_001_claroty_alerts_org_mismatch_returns_401              → Cell A: non-nil + MISMATCH → 401
    ///   test_W3_FIX_SEC_001_claroty_alerts_missing_org_header_on_real_org_returns_401 → Cell B: non-nil + ABSENT → 401 (this test)
    ///   test_W3_FIX_SEC_001_claroty_alerts_nil_org_no_header_returns_200         → Cell C: nil + ABSENT → 200
    #[tokio::test]
    async fn test_W3_FIX_SEC_001_claroty_alerts_missing_org_header_on_real_org_returns_401() {
        let instance_org = OrgId::from_uuid(
            Uuid::parse_str("11111111-1111-7000-8000-000000000001").expect("valid test UUID"),
        );

        let (_clone, base_url) = start_clone_with_org(instance_org).await;
        let client = test_client();

        let resp = client
            .post(format!("{base_url}/api/v1/alerts"))
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

    /// W3-FIX-SEC-001 — Cell C: nil-org clone + absent header → 200 (backward-compat) for list_alerts.
    ///
    /// `ClarotyClone::new()` sets `instance_org_id` to the nil UUID — the sentinel meaning
    /// "no org constraint". Callers that do not supply `X-Org-Id` must not be rejected.
    #[tokio::test]
    async fn test_W3_FIX_SEC_001_claroty_alerts_nil_org_no_header_returns_200() {
        let (_clone, base_url) = start_clone().await;
        let client = test_client();

        let resp = client
            .post(format!("{base_url}/api/v1/alerts"))
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

    /// W3-FIX-SEC-001 — org-isolation guard for list_alerted_devices (non-nil clone, mismatched org).
    ///
    /// When the clone has a real (non-nil) `instance_org_id`, a request whose
    /// `X-Org-Id` header does not match `instance_org_id` must return HTTP 401.
    ///
    /// Verifies:
    /// - Valid bearer + mismatched X-Org-Id → 401 `{"error": "org_id mismatch: ..."}`
    #[tokio::test]
    async fn test_W3_FIX_SEC_001_claroty_alerted_devices_org_mismatch_returns_401() {
        let instance_org = OrgId::from_uuid(
            Uuid::parse_str("11111111-1111-7000-8000-000000000001").expect("valid test UUID"),
        );
        let caller_org =
            Uuid::parse_str("22222222-2222-7000-8000-000000000002").expect("valid test UUID");

        let (_clone, base_url) = start_clone_with_org(instance_org).await;
        let client = test_client();

        let resp = client
            .post(format!("{base_url}/api/v1/alerts/alert-123/devices"))
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

    /// W3-FIX-SEC-001 — Cell B: non-nil clone + absent X-Org-Id header → 401 for list_alerted_devices.
    ///
    /// When the clone has a real (non-nil) `instance_org_id`, a request that omits
    /// the `X-Org-Id` header entirely must return HTTP 401. `validate_org_id` treats
    /// an absent header as a mismatch (AC-003: missing header → 401).
    ///
    /// This closes the org-isolation test matrix for `list_alerted_devices`:
    ///   test_W3_FIX_SEC_001_claroty_alerted_devices_org_mismatch_returns_401              → Cell A: non-nil + MISMATCH → 401
    ///   test_W3_FIX_SEC_001_claroty_alerted_devices_missing_org_header_on_real_org_returns_401 → Cell B: non-nil + ABSENT → 401 (this test)
    ///   test_W3_FIX_SEC_001_claroty_alerted_devices_nil_org_no_header_returns_200         → Cell C: nil + ABSENT → 200
    #[tokio::test]
    async fn test_W3_FIX_SEC_001_claroty_alerted_devices_missing_org_header_on_real_org_returns_401(
    ) {
        let instance_org = OrgId::from_uuid(
            Uuid::parse_str("11111111-1111-7000-8000-000000000001").expect("valid test UUID"),
        );

        let (_clone, base_url) = start_clone_with_org(instance_org).await;
        let client = test_client();

        let resp = client
            .post(format!("{base_url}/api/v1/alerts/alert-123/devices"))
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

    /// W3-FIX-SEC-001 — Cell C: nil-org clone + absent header → 200 (backward-compat) for list_alerted_devices.
    ///
    /// `ClarotyClone::new()` sets `instance_org_id` to the nil UUID — the sentinel meaning
    /// "no org constraint". Callers that do not supply `X-Org-Id` must not be rejected.
    #[tokio::test]
    async fn test_W3_FIX_SEC_001_claroty_alerted_devices_nil_org_no_header_returns_200() {
        let (_clone, base_url) = start_clone().await;
        let client = test_client();

        let resp = client
            .post(format!("{base_url}/api/v1/alerts/alert-123/devices"))
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
