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

/// Load the device-alert-relations fixture as a `Vec<serde_json::Value>`.
fn load_device_alert_relations_fixture() -> Vec<Value> {
    // SAFETY: fixture files are bundled at build time; missing fixture is a build error.
    #[allow(clippy::expect_used)]
    let raw = prism_dtu_common::load_fixture(env!("CARGO_MANIFEST_DIR"), "device-alert-relations")
        .expect("fixtures/device-alert-relations.json must exist");
    // SAFETY: fixture content is a well-formed JSON array validated at CI time.
    #[allow(clippy::expect_used)]
    raw.as_array()
        .expect("device-alert-relations fixture must be a JSON array")
        .clone()
}

/// `POST /api/v1/device_alert_relations/`
///
/// Returns device-alert relation entries.  Response: `{"devices_alerts": [...], "count": N}`.
/// Requires valid `Authorization: Bearer` header (AC-002).
///
/// # Three-way serving composition (F-CLARO-P2-MED-005 / ADR-036 v2.3 §2.4)
///
/// Mirrors `list_devices` composition exactly so the two routes stay structurally
/// parallel (parallel structure is itself a future-drift guard):
///
/// - **Scenario path** (`fixture_gen_seeded=true` AND `timeline.is_some()`): applies
///   `StageMask` to filter relations whose referenced device is not yet visible at the
///   current stage (INV-CROSS-DTU-ENTITY-COHERENCE-001).  Uses `_device_id` (the stage-
///   gating key stamped by the generator) to match the primary/lateral device gate.
/// - **Seeded path** (`fixture_gen_seeded=true` AND `timeline.is_none()`): returns all
///   `device_alert_relation`-surface records from `generated_records`.  `DormantTenant`
///   (seeded=true, 0 records) correctly returns empty — NOT the static fixture.
/// - **Static path** (`fixture_gen_seeded=false`): loads from
///   `fixtures/device-alert-relations.json` (backward-compatible default).
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

    // Three-way composition (ADR-036 v2.3 §2.4, F-CLARO-P2-MED-005):
    //
    // F3 / DTU-05: filter on the authoritative `_surface` discriminator stamped by
    // the generator — NOT key-presence heuristics.
    //
    // Sentinel: use `fixture_gen_seeded` (not generated_records.is_empty()) so that
    // DormantTenant (seeded=true, 0 records) serves empty — not the static fixture.
    // F-P6-HIGH-001 fix / ADR-036 v2.2 precedent.
    #[cfg(feature = "fixture-gen")]
    let entries: Vec<Value> = if state.fixture_gen_seeded {
        if let Some(ref timeline) = state.timeline {
            // Scenario path: apply StageMask — filter out relations referencing devices
            // that are hidden at the current stage (INV-CROSS-DTU-ENTITY-COHERENCE-001).
            // Uses `_device_id` (stage-gating key) stamped by make_device_alert_relation,
            // mirroring the `device_id` check in list_devices exactly.
            use prism_dtu_common::current_stage_index;
            let now = chrono::Utc::now().timestamp();
            let stage_idx = current_stage_index(timeline, now);
            let mask = &timeline.stages[stage_idx].visible_entity_mask;
            let primary_id = &timeline.entities.primary_device_id_cs;
            let lateral_ids: std::collections::HashSet<&str> = timeline
                .entities
                .lateral_device_ids_cs
                .iter()
                .map(|s| s.as_str())
                .collect();

            state
                .generated_records
                .iter()
                .filter(|rec| {
                    rec.get("_surface").and_then(|v| v.as_str()) == Some("device_alert_relation")
                })
                .filter(|rec| {
                    // Same gate as list_devices: primary/lateral/non-catalog device_id check.
                    let device_id = rec.get("_device_id").and_then(|v| v.as_str()).unwrap_or("");
                    if device_id == primary_id {
                        // Stage 0: primary device (and its relations) not yet surfaced.
                        // stage_idx > 0 guard per BC-2.06.019 PC-4.
                        mask.primary_device && stage_idx > 0
                    } else if lateral_ids.contains(device_id) {
                        mask.lateral_devices
                    } else {
                        true
                    }
                })
                .cloned()
                .collect()
        } else {
            // Seeded path (no scenario): all device_alert_relation-surface records.
            // DormantTenant (seeded=true, 0 records) → empty — NOT static fixture.
            state
                .generated_records
                .iter()
                .filter(|rec| {
                    rec.get("_surface").and_then(|v| v.as_str()) == Some("device_alert_relation")
                })
                .cloned()
                .collect()
        }
    } else {
        // Static path: load from fixture (fixture_gen_seeded=false).
        load_device_alert_relations_fixture()
    };
    #[cfg(not(feature = "fixture-gen"))]
    let entries = load_device_alert_relations_fixture();

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
//   test_claroty_tier3_device_alert_relations_dtu_route_returns_envelope → AC-001, AC-003
//     NOTE: AC-004 (TOML device_alert_relations table block with 10 contracted columns) is covered
//     by claroty_spec_prose_fidelity.rs::test_claroty_tier3_device_alert_relations_table_declared.
//     The ≥5-entries assertion in this test traces to Task 5 (fixture density requirement), NOT AC-004.
//     RG-002 traceability: the absent-key assertion (AC-003 second bullet — `device_alert_relations`
//     MUST NOT appear as a top-level key) is embedded in this test as the `is_none()` assertion.
//   test_claroty_tier3_device_alert_relations_dtu_auth_enforced          → AC-002
//   test_claroty_tier3_device_alert_relations_dtu_column_parity          → SAP-2
//   test_device_alert_relations_fixture_referential_integrity             → HIGH-4 (device_uid/alert_id FK integrity)
//   test_W3_FIX_SEC_001_claroty_device_alert_relations_org_mismatch_returns_401              → SEC-001
//   test_W3_FIX_SEC_001_claroty_device_alert_relations_missing_org_header_returns_401        → SEC-001
//   test_W3_FIX_SEC_001_claroty_device_alert_relations_nil_org_no_header_returns_200         → SEC-001
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

    /// Start a `ClarotyClone` with a specific non-nil org identity.
    ///
    /// Used by SEC-001 org-isolation tests. The clone enforces X-Org-Id validation
    /// because `instance_org_id` is non-nil.
    async fn start_clone_with_org(org_id: OrgId) -> (ClarotyClone, String) {
        let mut clone = ClarotyClone::with_org(org_id);
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

        // AC-003 (second bullet) — RG-002 absent-key assertion:
        // The path-stem key `device_alert_relations` MUST NOT appear as a top-level key.
        // Using the path stem silently discards every row at pipeline normalization time
        // (BC-2.16.013 EC-016-013-009 stem ambiguity). Both the presence of `devices_alerts`
        // AND the absence of `device_alert_relations` are required; asserting only one is
        // insufficient (S-DEMO-CLAROTY-DAR-001 AC-003 second bullet; F-CLARO-P2-MED-002).
        assert!(
            body.get("device_alert_relations").is_none(),
            "response MUST NOT contain `device_alert_relations` top-level key — using the \
             path stem as the response key silently discards all rows at pipeline normalization \
             time (BC-2.16.013 EC-016-013-009 stem ambiguity); AC-003 mandates BOTH: \
             'devices_alerts' present AND 'device_alert_relations' absent"
        );

        // Task 5 (fixture ≥5 entries): fixture must contain at least 5 synthetic entries.
        assert!(
            devices_alerts.len() >= 5,
            "device-alert-relations fixture must contain at least 5 synthetic entries; \
             got {} (Task 5 — fixture density requirement)",
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

        // Task 5 (fixture ≥5 entries): fixture must contain at least 5 synthetic entries.
        assert!(
            entries.len() >= 5,
            "device-alert-relations.json must contain at least 5 synthetic entries; \
             got {} (Task 5 — fixture density requirement)",
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

    /// Referential integrity: every `device_uid` in the relations fixture must exist
    /// as a `uid` in `devices.json`, and every `alert_id` must exist as an `id` in
    /// `alerts.json`.  This test is the canonical guard against dangling FK references
    /// (HIGH-4 resolution: original fixture had UUIDs that matched no device uid).
    ///
    /// Not wired to an AC by number — this is a data-integrity invariant that applies
    /// at all times regardless of story scope.
    #[test]
    fn test_device_alert_relations_fixture_referential_integrity() {
        let manifest_dir = env!("CARGO_MANIFEST_DIR");

        // Load all three fixtures.
        let relations_raw = std::fs::read_to_string(format!(
            "{manifest_dir}/fixtures/device-alert-relations.json"
        ))
        .expect("device-alert-relations.json must exist");
        let devices_raw = std::fs::read_to_string(format!("{manifest_dir}/fixtures/devices.json"))
            .expect("devices.json must exist");
        let alerts_raw = std::fs::read_to_string(format!("{manifest_dir}/fixtures/alerts.json"))
            .expect("alerts.json must exist");

        let relations: Vec<serde_json::Value> = serde_json::from_str(&relations_raw)
            .expect("relations fixture must parse as JSON array");
        let devices: Vec<serde_json::Value> =
            serde_json::from_str(&devices_raw).expect("devices fixture must parse as JSON array");
        let alerts: Vec<serde_json::Value> =
            serde_json::from_str(&alerts_raw).expect("alerts fixture must parse as JSON array");

        // Build lookup sets.
        let device_uids: std::collections::HashSet<String> = devices
            .iter()
            .filter_map(|d| d.get("uid").and_then(|v| v.as_str()).map(String::from))
            .collect();

        // alert ids may be integer or string in the fixture; normalise to String for comparison.
        let alert_ids: std::collections::HashSet<String> = alerts
            .iter()
            .filter_map(|a| {
                a.get("id").map(|v| match v {
                    serde_json::Value::Number(n) => n.to_string(),
                    serde_json::Value::String(s) => s.clone(),
                    other => other.to_string(),
                })
            })
            .collect();

        for (i, rel) in relations.iter().enumerate() {
            let device_uid = rel
                .get("device_uid")
                .and_then(|v| v.as_str())
                .unwrap_or_else(|| panic!("relations[{i}] missing device_uid"));

            assert!(
                device_uids.contains(device_uid),
                "relations[{i}].device_uid = {:?} does not exist in devices.json uids.\n\
                 Known uids: {device_uids:?}",
                device_uid
            );

            let alert_id_val = rel
                .get("alert_id")
                .unwrap_or_else(|| panic!("relations[{i}] missing alert_id"));
            let alert_id_str = match alert_id_val {
                serde_json::Value::Number(n) => n.to_string(),
                serde_json::Value::String(s) => s.clone(),
                other => other.to_string(),
            };

            assert!(
                alert_ids.contains(&alert_id_str),
                "relations[{i}].alert_id = {:?} does not exist in alerts.json ids.\n\
                 Known ids: {alert_ids:?}",
                alert_id_str
            );
        }
    }

    // -----------------------------------------------------------------------
    // F-CLARO-P2-MED-005: seeded-path FK integrity test
    //
    // Parameterizes over the SEEDED serving path (fixture_gen_seeded=true).
    // The existing static-fixture test (`test_device_alert_relations_fixture_referential_integrity`)
    // only covered the static path where FK integrity already held.
    //
    // RED before fix: `list_device_alert_relations` always loads the static fixture
    // (uid-001..015) regardless of seeded state. Generated devices have uids of the
    // form {slug}-{seed}-device-{hex}, so the intersection is empty — uid-001 is not
    // in the generated device uid set, causing the FK assertion to fail.
    //
    // GREEN after fix: the seeded path serves generated device_alert_relation records
    // whose `device_uid` values reference generated device uids.
    //
    // Gated behind `#[cfg(feature = "fixture-gen")]` because:
    //   - `ClarotyClone::new_with_seed` requires `fixture-gen`
    //   - `prism_dtu_common::Archetype` requires `fixture-gen`
    // Runs under `just check` (--all-features) and any invocation with `--features fixture-gen`.
    // -----------------------------------------------------------------------

    /// F-CLARO-P2-MED-005 — referential integrity of `device_alert_relations` on the
    /// seeded path.
    ///
    /// Constructs a `CompromisedEndpoint` seeded clone (50 generated devices, 20 generated
    /// alerts) then asserts:
    /// 1. `device_alert_relations` returns non-empty results (seeded path serves data).
    /// 2. Every `device_uid` in the relations set exists as a `uid` in the generated
    ///    devices endpoint response.
    /// 3. Every `alert_id` in the relations set exists as an `id` in the generated
    ///    alerts endpoint response.
    ///
    /// TD-VSDD-059: this test MUST fail on the seeded path before the fix.
    /// The static fixture (uid-001..015) never overlaps with generated device uids
    /// ({slug}-42-device-{hex}), so point 2 catches the defect.
    #[cfg(feature = "fixture-gen")]
    #[tokio::test]
    async fn test_device_alert_relations_seeded_fk_integrity() {
        // NOTE: prism_dtu_common::OrgId is a [u8; 16]-backed newtype — different from
        // prism_core::OrgId (which wraps uuid::Uuid). Use the former for new_with_seed.
        let org_id = prism_dtu_common::OrgId([
            0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x70, 0x00, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x42,
        ]);

        let mut clone = ClarotyClone::new_with_seed(
            42,
            prism_dtu_common::Archetype::CompromisedEndpoint,
            org_id,
        );
        clone
            .start()
            .await
            .expect("seeded ClarotyClone must start in test environment");
        let base_url = clone.base_url();
        let client = test_client();

        // Fetch generated devices → build uid set.
        let devices_resp: serde_json::Value = client
            .post(format!("{base_url}/api/v1/devices"))
            .header("Authorization", "Bearer test-token")
            .json(&json!({}))
            .send()
            .await
            .expect("devices request must succeed")
            .json()
            .await
            .expect("devices response must be JSON");

        // Fetch generated alerts → build id set.
        let alerts_resp: serde_json::Value = client
            .post(format!("{base_url}/api/v1/alerts"))
            .header("Authorization", "Bearer test-token")
            .json(&json!({"fields": ["id"]}))
            .send()
            .await
            .expect("alerts request must succeed")
            .json()
            .await
            .expect("alerts response must be JSON");

        // Fetch device_alert_relations — this is what we are testing.
        let relations_resp: serde_json::Value = client
            .post(format!("{base_url}/api/v1/device_alert_relations/"))
            .header("Authorization", "Bearer test-token")
            .json(&json!({"fields": ["device_uid", "alert_id"]}))
            .send()
            .await
            .expect("relations request must succeed")
            .json()
            .await
            .expect("relations response must be JSON");

        // Build device uid set from the seeded devices route.
        let device_uids: std::collections::HashSet<String> = devices_resp
            .get("devices")
            .and_then(|v| v.as_array())
            .expect("devices response must have 'devices' array")
            .iter()
            .filter_map(|d| d.get("uid").and_then(|v| v.as_str()).map(String::from))
            .collect();

        // Build alert id set (normalised to String) from the seeded alerts route.
        let alert_ids: std::collections::HashSet<String> = alerts_resp
            .get("alerts")
            .and_then(|v| v.as_array())
            .expect("alerts response must have 'alerts' array")
            .iter()
            .filter_map(|a| {
                a.get("id").map(|v| match v {
                    serde_json::Value::Number(n) => n.to_string(),
                    serde_json::Value::String(s) => s.clone(),
                    other => other.to_string(),
                })
            })
            .collect();

        assert!(
            !device_uids.is_empty(),
            "seeded devices must be non-empty (CompromisedEndpoint produces 50 devices)"
        );
        assert!(
            !alert_ids.is_empty(),
            "seeded alerts must be non-empty (CompromisedEndpoint produces 20 alerts)"
        );

        let relations = relations_resp
            .get("devices_alerts")
            .and_then(|v| v.as_array())
            .expect("relations response must have 'devices_alerts' key (AC-003)");

        // Point 1: seeded path must produce non-empty relations.
        // After fix: CompromisedEndpoint emits 20 device_alert_relation records.
        // Before fix: static fixture is returned but uid-001 fails FK check below.
        assert!(
            !relations.is_empty(),
            "F-CLARO-P2-MED-005: seeded device_alert_relations must be non-empty for \
             CompromisedEndpoint (20 generated alerts → 20 relations). \
             DormantTenant would correctly return empty — but this archetype has data."
        );

        // Points 2 + 3: FK integrity over ALL returned relations.
        for (i, rel) in relations.iter().enumerate() {
            let device_uid = rel
                .get("device_uid")
                .and_then(|v| v.as_str())
                .unwrap_or_else(|| panic!("relations[{i}] missing device_uid"));

            assert!(
                device_uids.contains(device_uid),
                "F-CLARO-P2-MED-005 FK violation: relations[{i}].device_uid = {device_uid:?} \
                 not found in generated device uid set.\n\
                 EXPECTED: uids of the form '{{slug}}-42-device-{{hex}}' (generated path).\n\
                 ACTUAL: '{device_uid}' (static fixture uid like 'uid-001' means the route \
                 is serving the static fixture instead of the seeded path).\n\
                 Generated device uids (first 5): {:?}",
                device_uids.iter().take(5).collect::<Vec<_>>()
            );

            let alert_id_val = rel
                .get("alert_id")
                .unwrap_or_else(|| panic!("relations[{i}] missing alert_id"));
            let alert_id_str = match alert_id_val {
                serde_json::Value::Number(n) => n.to_string(),
                serde_json::Value::String(s) => s.clone(),
                other => other.to_string(),
            };

            assert!(
                alert_ids.contains(&alert_id_str),
                "F-CLARO-P2-MED-005 FK violation: relations[{i}].alert_id = {alert_id_str:?} \
                 not found in generated alert id set.\n\
                 Generated alert ids (first 5): {:?}",
                alert_ids.iter().take(5).collect::<Vec<_>>()
            );
        }
    }

    // -----------------------------------------------------------------------
    // W3-FIX-SEC-001 org-isolation guard tests (SEC-001 closure)
    //
    // These three tests close the org-isolation test matrix for
    // `list_device_alert_relations`, matching the pattern established for
    // `list_audit_logs` and `list_alerts` by W3-FIX-SEC-001.
    //
    // The code guard (nil_org sentinel + validate_org_id call) was present in
    // the original implementation; these tests verify it is active.
    // -----------------------------------------------------------------------

    /// W3-FIX-SEC-001 — org-isolation guard: non-nil clone + mismatched X-Org-Id → 401.
    ///
    /// When the clone has a real (non-nil) `instance_org_id`, a request bearing a
    /// different UUID must be rejected with HTTP 401.
    ///
    /// Org-isolation test matrix for `list_device_alert_relations`:
    ///   this test → non-nil + MISMATCH → 401
    ///   test_W3_FIX_SEC_001_claroty_device_alert_relations_missing_org_header_returns_401 → non-nil + ABSENT → 401
    ///   test_W3_FIX_SEC_001_claroty_device_alert_relations_nil_org_no_header_returns_200  → nil + ABSENT → 200
    #[tokio::test]
    async fn test_W3_FIX_SEC_001_claroty_device_alert_relations_org_mismatch_returns_401() {
        // Instance org: a real, non-nil UUID.
        let instance_org = OrgId::from_uuid(
            Uuid::parse_str("11111111-1111-7000-8000-000000000001").expect("valid test UUID"),
        );
        // A different org UUID sent by the caller.
        let caller_org =
            Uuid::parse_str("22222222-2222-7000-8000-000000000002").expect("valid test UUID");

        let (_clone, base_url) = start_clone_with_org(instance_org).await;
        let client = test_client();

        let resp = client
            .post(format!("{base_url}/api/v1/device_alert_relations/"))
            .header("Authorization", "Bearer test-token")
            .header("X-Org-Id", caller_org.to_string())
            .json(&json!({"fields": ["device_uid", "alert_id"]}))
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
            .expect("401 response must have JSON body (SEC-001)");
        assert!(
            body.get("error")
                .and_then(|v| v.as_str())
                .map(|s| s.contains("org_id mismatch"))
                .unwrap_or(false),
            "SEC-001: 401 body must contain 'org_id mismatch'; got: {body}"
        );
    }

    /// W3-FIX-SEC-001 — org-isolation guard: non-nil clone + absent X-Org-Id header → 401.
    ///
    /// When the clone has a real (non-nil) `instance_org_id`, a request that omits the
    /// `X-Org-Id` header entirely must return HTTP 401. `validate_org_id` treats an
    /// absent header as a mismatch (AC-003: missing header → 401).
    ///
    /// Org-isolation test matrix for `list_device_alert_relations`:
    ///   test_W3_FIX_SEC_001_claroty_device_alert_relations_org_mismatch_returns_401 → non-nil + MISMATCH → 401
    ///   this test → non-nil + ABSENT → 401
    ///   test_W3_FIX_SEC_001_claroty_device_alert_relations_nil_org_no_header_returns_200 → nil + ABSENT → 200
    #[tokio::test]
    async fn test_W3_FIX_SEC_001_claroty_device_alert_relations_missing_org_header_returns_401() {
        let instance_org = OrgId::from_uuid(
            Uuid::parse_str("11111111-1111-7000-8000-000000000001").expect("valid test UUID"),
        );

        let (_clone, base_url) = start_clone_with_org(instance_org).await;
        let client = test_client();

        let resp = client
            .post(format!("{base_url}/api/v1/device_alert_relations/"))
            .header("Authorization", "Bearer test-token")
            // Intentionally NO X-Org-Id header.
            .json(&json!({"fields": ["device_uid", "alert_id"]}))
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

    /// W3-FIX-SEC-001 backward-compat: nil-org clone without X-Org-Id header must return 200.
    ///
    /// `ClarotyClone::new()` sets `instance_org_id` to the nil UUID — the sentinel meaning
    /// "no org constraint". Callers that do not supply `X-Org-Id` must not be rejected.
    ///
    /// Org-isolation test matrix for `list_device_alert_relations`:
    ///   test_W3_FIX_SEC_001_claroty_device_alert_relations_org_mismatch_returns_401        → non-nil + MISMATCH → 401
    ///   test_W3_FIX_SEC_001_claroty_device_alert_relations_missing_org_header_returns_401  → non-nil + ABSENT → 401
    ///   this test → nil + ABSENT → 200
    #[tokio::test]
    async fn test_W3_FIX_SEC_001_claroty_device_alert_relations_nil_org_no_header_returns_200() {
        // Nil-org clone — no org enforcement.
        let (_clone, base_url) = start_clone().await;
        let client = test_client();

        let resp = client
            .post(format!("{base_url}/api/v1/device_alert_relations/"))
            .header("Authorization", "Bearer test-token")
            // Intentionally NO X-Org-Id header.
            .json(&json!({"fields": ["device_uid", "alert_id"]}))
            .send()
            .await
            .expect("transport must not fail for nil-org backward-compat test");

        assert_eq!(
            resp.status().as_u16(),
            200,
            "nil-org clone without X-Org-Id header must return HTTP 200 (SEC-001 backward-compat)"
        );
    }
}
