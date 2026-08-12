//! Route handler for the Claroty xDome device-alert relations endpoint.
//!
//! `POST /api/v1/device_alert_relations/` — device-alert pair list (Tier 3).
//!
//! Response envelope: `{"devices_alerts": [...], "count": N_or_null}`.
//! The key `devices_alerts` matches `response_path = "$.devices_alerts"` in
//! `claroty.sensor.toml`.  Requires valid `Authorization: Bearer` header (AC-002).
//!
//! The path is registered WITHOUT trailing slash in `clone.rs`.
//! `NormalizePathLayer::trim_trailing_slash()` (applied at the outer service level)
//! strips the trailing slash from inbound requests before routing, so both
//! `/api/v1/device_alert_relations/` and `/api/v1/device_alert_relations` reach
//! this handler (same pattern as `list_devices` and `list_alerts`).

use std::sync::Arc;

use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    Json,
};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::{
    routes::devices::{check_bearer_auth, validate_org_id},
    state::ClarotyState,
    types::GetDeviceAlertsBody,
};

/// `POST /api/v1/device_alert_relations/`
///
/// Returns synthetic device-alert relation entries from
/// `fixtures/device-alert-relations.json`.
/// Response: `{"devices_alerts": [...], "count": N}`.
/// Requires valid `Authorization: Bearer` header (AC-002).
///
/// When the clone has a real (non-nil) `instance_org_id`, the `X-Org-Id` header is
/// validated against `state.instance_org_id` after bearer auth — matching the guard
/// used by `list_devices` and `list_audit_logs`.  Nil-org clones (created via
/// `ClarotyClone::new()`) skip header validation for backward compatibility.
pub async fn list_device_alert_relations(
    State(state): State<Arc<ClarotyState>>,
    headers: HeaderMap,
    _body: Option<Json<GetDeviceAlertsBody>>,
) -> (StatusCode, Json<Value>) {
    if let Err(err) = check_bearer_auth(&headers) {
        return err;
    }

    // Org-isolation guard: validate X-Org-Id when clone has a real (non-nil) org.
    // Nil-org clones (ClarotyClone::new()) skip this check for backward compat —
    // nil UUID is the sentinel meaning "no org constraint".
    let nil_org = prism_core::OrgId::from_uuid(Uuid::nil());
    if state.instance_org_id != nil_org {
        if let Err(err) = validate_org_id(&headers, state.instance_org_id) {
            return err;
        }
    }

    // SAFETY: fixture files are bundled at build time; missing fixture is a build error.
    #[allow(clippy::expect_used)]
    let raw = prism_dtu_common::load_fixture(env!("CARGO_MANIFEST_DIR"), "device-alert-relations")
        .expect("fixtures/device-alert-relations.json must exist");
    // SAFETY: fixture content is a well-formed JSON array validated at CI time.
    #[allow(clippy::expect_used)]
    let entries = raw
        .as_array()
        .expect("device-alert-relations fixture must be a JSON array")
        .clone();
    let count = entries.len() as u32;

    (
        StatusCode::OK,
        Json(json!({"devices_alerts": entries, "count": count})),
    )
}

// ---------------------------------------------------------------------------
// Tests — Tier 3 device-alert relations DTU route
//
// AC traces:
//   test_claroty_tier3_device_alert_relations_dtu_route_returns_envelope → AC-001, AC-003, AC-004
//   test_claroty_tier3_device_alert_relations_dtu_auth_enforced          → AC-002
//   test_claroty_tier3_device_alert_relations_dtu_column_parity          → SAP-2
// ---------------------------------------------------------------------------
#[cfg(test)]
#[allow(non_snake_case, clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use std::time::Duration;

    use prism_dtu_common::BehavioralClone;
    use serde_json::json;

    use crate::clone::ClarotyClone;
    use crate::types::ClarotyDeviceAlertRelation;

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

    /// AC-001: POST /api/v1/device_alert_relations/ returns HTTP 200 with valid bearer.
    /// AC-003: Response envelope contains `devices_alerts` key (matches response_path = "$.devices_alerts").
    /// AC-004: `devices_alerts` array is non-empty (≥ 5 synthetic entries).
    ///
    /// Wire-shape assertion (SAP-2 probe rule 6): asserts on the SERIALIZED JSON envelope,
    /// not only Rust struct shape.  The key MUST be `devices_alerts` (not `device_alert_relations`
    /// or `alerts`) — this matches `response_path = "$.devices_alerts"` in the TOML and the
    /// `GetDeviceAlertsResponse.devices_alerts` key in the OpenAPI schema.
    #[tokio::test]
    async fn test_claroty_tier3_device_alert_relations_dtu_route_returns_envelope() {
        let (_clone, base_url) = start_clone().await;
        let client = test_client();

        let resp = client
            .post(format!("{base_url}/api/v1/device_alert_relations/"))
            .header("Authorization", "Bearer test-token")
            .json(&json!({"fields": ["device_uid", "alert_id"]}))
            .send()
            .await
            .expect("POST /api/v1/device_alert_relations/ must not fail at transport level");

        // AC-001: route registered and reachable (not 404).
        assert_eq!(
            resp.status().as_u16(),
            200,
            "POST /api/v1/device_alert_relations/ with valid bearer must return HTTP 200 (AC-001)"
        );

        let body: serde_json::Value = resp
            .json()
            .await
            .expect("response body must be valid JSON (AC-003)");

        // Wire-shape assertion: MUST be `devices_alerts` key, NOT `device_alert_relations`.
        // This is the SAP-2 probe — emission site (json! macro) is authoritative.
        let devices_alerts = body
            .get("devices_alerts")
            .and_then(|v| v.as_array())
            .expect(
                "response MUST contain `devices_alerts` JSON array key matching \
                 claroty.sensor.toml response_path=\"$.devices_alerts\" (AC-003); \
                 key must be 'devices_alerts' not 'device_alert_relations'",
            );

        // AC-004: at least 5 synthetic entries.
        assert!(
            devices_alerts.len() >= 5,
            "device-alert-relations fixture must contain at least 5 synthetic entries; \
             got {} (AC-004)",
            devices_alerts.len()
        );

        // AC-003: `count` field present and matches array length.
        let count = body.get("count").and_then(|v| v.as_u64()).expect(
            "response must contain `count` numeric field (AC-003); \
                     note: uses 'count' not 'total' per GetDeviceAlertsResponse schema",
        );
        assert_eq!(
            count as usize,
            devices_alerts.len(),
            "`count` must equal the length of the `devices_alerts` array (AC-003)"
        );
    }

    /// AC-002: POST /api/v1/device_alert_relations/ without bearer → HTTP 401.
    #[tokio::test]
    async fn test_claroty_tier3_device_alert_relations_dtu_auth_enforced() {
        let (_clone, base_url) = start_clone().await;
        let client = test_client();

        let resp = client
            .post(format!("{base_url}/api/v1/device_alert_relations/"))
            .json(&json!({"fields": ["device_uid"]}))
            .send()
            .await
            .expect("transport must not fail for auth test");

        assert_eq!(
            resp.status().as_u16(),
            401,
            "missing Authorization header must return HTTP 401 (AC-002)"
        );

        let body: serde_json::Value = resp.json().await.expect("401 response must have JSON body");
        assert_eq!(
            body.get("error").and_then(|v| v.as_str()),
            Some("missing or invalid Authorization header"),
            "401 body `error` must be the canonical auth error message (AC-002, POL-24); \
             got: {body}"
        );
        assert_eq!(
            body.get("code").and_then(|v| v.as_u64()),
            Some(401),
            "401 body must contain `code: 401` (AC-002); got: {body}"
        );
    }

    /// SAP-2 column parity: every column declared in claroty.sensor.toml
    /// `device_alert_relations` table maps 1:1 to a field in
    /// `ClarotyDeviceAlertRelation`.
    ///
    /// Deserializes the fixture into `Vec<ClarotyDeviceAlertRelation>` to exercise all
    /// field paths simultaneously.  A missing or mis-typed field causes a serde
    /// deserialization error → test fails with an informative message.
    #[tokio::test]
    async fn test_claroty_tier3_device_alert_relations_dtu_column_parity() {
        // Part 1: Direct fixture deserialization (SAP-2 compile-time parity check).
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let fixture_path = format!("{manifest_dir}/fixtures/device-alert-relations.json");
        let fixture_raw = std::fs::read_to_string(&fixture_path).unwrap_or_else(|e| {
            panic!(
                "fixtures/device-alert-relations.json must exist and be readable; \
                 error: {e}"
            )
        });

        let entries: Vec<ClarotyDeviceAlertRelation> = serde_json::from_str(&fixture_raw)
            .unwrap_or_else(|e| {
                panic!(
                    "fixtures/device-alert-relations.json must deserialize into \
                     Vec<ClarotyDeviceAlertRelation>; serde error: {e}\n\
                     This means a TOML column lacks a matching field in \
                     ClarotyDeviceAlertRelation — SAP-2 P1 CRITICAL"
                )
            });

        // At least 5 synthetic entries.
        assert!(
            entries.len() >= 5,
            "device-alert-relations.json must contain at least 5 synthetic entries; \
             got {} (AC-004)",
            entries.len()
        );

        // Spot-check required fields on the first entry.
        let first = &entries[0];
        assert!(
            !first.device_uid.is_empty(),
            "ClarotyDeviceAlertRelation.device_uid must be non-empty"
        );
        assert!(
            !first.device_alert_detected_time.is_empty(),
            "ClarotyDeviceAlertRelation.device_alert_detected_time must be non-empty"
        );
        assert!(
            !first.device_risk_score.is_empty(),
            "ClarotyDeviceAlertRelation.device_risk_score must be non-empty"
        );
        assert!(
            !first.device_alert_status.is_empty(),
            "ClarotyDeviceAlertRelation.device_alert_status must be non-empty"
        );

        // device_alert_detected_time must be ISO 8601 format.
        assert!(
            first.device_alert_detected_time.contains('T')
                && (first.device_alert_detected_time.ends_with('Z')
                    || first.device_alert_detected_time.contains('+')),
            "device_alert_detected_time must be ISO 8601; got {:?}",
            first.device_alert_detected_time
        );

        // Part 2: HTTP round-trip verifies full emission path.
        let (_clone, base_url) = start_clone().await;
        let client = test_client();

        let resp = client
            .post(format!("{base_url}/api/v1/device_alert_relations/"))
            .header("Authorization", "Bearer test-token")
            .json(&json!({"fields": ["device_uid", "alert_id"]}))
            .send()
            .await
            .expect("transport must not fail");

        assert_eq!(resp.status().as_u16(), 200, "HTTP 200 required (AC-001)");

        let body: serde_json::Value = resp.json().await.expect("body must be JSON");
        let devices_alerts_value = body
            .get("devices_alerts")
            .expect("response must have `devices_alerts` key (SAP-2 wire-emission check)");

        // Deserialize the HTTP response array into Vec<ClarotyDeviceAlertRelation>.
        // Fails if any column in the response doesn't match the struct (SAP-2).
        let response_entries: Vec<ClarotyDeviceAlertRelation> =
            serde_json::from_value(devices_alerts_value.clone()).unwrap_or_else(|e| {
                panic!(
                    "HTTP response `devices_alerts` array must deserialize into \
                     Vec<ClarotyDeviceAlertRelation>; serde error: {e} (SAP-2)"
                )
            });

        assert!(
            !response_entries.is_empty(),
            "HTTP response devices_alerts must be non-empty"
        );
    }
}
