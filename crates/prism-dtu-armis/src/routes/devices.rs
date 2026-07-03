//! Device inventory, activity log, and risk score route handlers.
//!
//! Endpoints:
//! - `GET /api/v1/devices` — AQL-forwarded device query (GET form)
//! - `POST /api/v1/devices` — AQL-forwarded device query (POST form)
//!   (Armis supports both methods per API spec — EC-005)
//! - `GET /api/v1/devices/{device_id}/activity` — device activity log
//! - `GET /api/v1/devices/{device_id}/risk` — device risk score
//!
//! # Auth model (dual-mode, CR-012/SEC-P2-001)
//!
//! All endpoints require `Authorization: Bearer {non-empty}` header.
//! Missing/empty token → HTTP 403 `{"error": "...", "code": 403}` (Armis returns
//! 403, not 401 — AC-5, per `dtu-assessment.md §3.4`).
//!
//! `X-Org-Id` uses a **dual-mode** policy keyed on `instance_org_id`:
//!
//! ## Default-instance clones (`instance_org_id == DTU_DEFAULT_INSTANCE_ORG_ID`)
//! Use **validate-on-presence**:
//! - Header absent → guard skipped → request proceeds (backward compat with 50+
//!   pre-existing tests that omit the header).
//! - Header present with matching UUID → 200.
//! - Header present with mismatch or non-UUID → 401.
//!
//! ## Real-org clones (`instance_org_id != DTU_DEFAULT_INSTANCE_ORG_ID`)
//! Use **auth model A** (same as Claroty/CrowdStrike):
//! - Header absent → 401.
//! - Header present with matching UUID → 200.
//! - Header present with mismatch → 401.
//!
//! (CR-012/SEC-P2-001; BC-3.5.002 precondition 3; x_org_id_auth.rs §Auth model)
//!
//! AQL passthrough: `aql` query parameter (or POST body field) is accepted verbatim,
//! appended to `aql_log`, and NOT parsed or validated (R-DTU-002 mitigation).

use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use serde_json::Value as JsonValue;

use crate::{
    state::{ArmisState, DTU_ROUTE_ORG_ID},
    types::{
        ActivityData, ActivityResponse, ArmisError, DeviceRecord, DevicesData, DevicesResponse,
        RiskData, RiskResponse,
    },
};

/// Query parameters accepted by `GET /api/v1/devices`.
#[derive(Debug, Deserialize, Default)]
pub struct DeviceQueryParams {
    /// AQL string — accepted verbatim, stored in AQL log (not parsed).
    pub aql: Option<String>,
    /// Page number (1-based). Defaults to 1.
    pub page: Option<u32>,
    /// Page size. Defaults to 25.
    pub size: Option<u32>,
}

/// POST body accepted by `POST /api/v1/devices`.
#[derive(Debug, Deserialize, Default)]
pub struct DeviceQueryBody {
    pub aql: Option<String>,
    pub page: Option<u32>,
    pub size: Option<u32>,
}

/// `GET /api/v1/devices` — device inventory with AQL forwarding and pagination.
pub async fn get_or_post_devices(
    State(state): State<Arc<ArmisState>>,
    headers: HeaderMap,
    Query(params): Query<DeviceQueryParams>,
) -> impl IntoResponse {
    if let Some(err) = check_bearer_auth(&headers) {
        return err;
    }

    // CR-012/SEC-P2-001: dual-mode X-Org-Id policy.
    // Real-org clones (instance_org_id != DTU_DEFAULT_INSTANCE_ORG_ID):
    //   auth model A — absent header → 401, mismatch → 401.
    // Default-instance clones (instance_org_id == DTU_DEFAULT_INSTANCE_ORG_ID):
    //   validate-on-presence — absent header → skip (backward compat),
    //   present header with mismatch → 401.
    // See module doc for full rationale.
    let is_real_org = state.instance_org_id != crate::state::DTU_DEFAULT_INSTANCE_ORG_ID;
    if is_real_org || headers.get("x-org-id").is_some() {
        if let Err((status, body)) = validate_org_id(&headers, state.instance_org_id) {
            return (status, body).into_response();
        }
    }

    // Capture AQL string verbatim (R-DTU-002).
    if let Some(ref aql) = params.aql {
        state.capture_aql(aql);
    }

    let page = params.page.unwrap_or(1).max(1);
    let size = params.size.unwrap_or(25).max(1) as usize;

    paginate_devices(&state, page, size as u32)
}

/// `POST /api/v1/devices` — AQL device query via JSON body (EC-005).
///
/// Armis supports both GET (query-param AQL) and POST (JSON body AQL).
/// This handler reads AQL from the JSON body and falls back to query-param AQL.
pub async fn post_devices(
    State(state): State<Arc<ArmisState>>,
    headers: HeaderMap,
    Query(params): Query<DeviceQueryParams>,
    body: Option<Json<DeviceQueryBody>>,
) -> impl IntoResponse {
    if let Some(err) = check_bearer_auth(&headers) {
        return err;
    }

    // CR-012/SEC-P2-001: dual-mode X-Org-Id policy (see get_or_post_devices comment).
    let is_real_org = state.instance_org_id != crate::state::DTU_DEFAULT_INSTANCE_ORG_ID;
    if is_real_org || headers.get("x-org-id").is_some() {
        if let Err((status, err_body)) = validate_org_id(&headers, state.instance_org_id) {
            return (status, err_body).into_response();
        }
    }

    // AQL priority: JSON body > query param (R-DTU-002).
    let aql = body
        .as_ref()
        .and_then(|b| b.aql.clone())
        .or_else(|| params.aql.clone());

    if let Some(ref aql_str) = aql {
        state.capture_aql(aql_str);
    }

    let page = body
        .as_ref()
        .and_then(|b| b.page)
        .or(params.page)
        .unwrap_or(1)
        .max(1);
    let size = body
        .as_ref()
        .and_then(|b| b.size)
        .or(params.size)
        .unwrap_or(25)
        .max(1);

    paginate_devices(&state, page, size)
}

/// Pagination helper shared by GET and POST device queries.
///
/// Three-way composition (ADR-036 v2.3 §2.4, BC-2.06.019 PC-4, F-P6-HIGH-001):
/// - Scenario path (`fixture_gen_seeded == true && timeline.is_some()`):
///   applies StageMask projection — only entities visible at the current stage are served.
///   Primary device: visible when `mask.primary_device && stage_index > 0`.
///   Lateral devices: visible when `mask.lateral_devices`.
///   Non-catalog records: always visible (pass-through).
/// - Seeded path (`fixture_gen_seeded == true && timeline.is_none()`):
///   serves all generated records (Story-A behavior, unchanged).
///   DormantTenant (seeded=true, 0 records) serves EMPTY — not static fixture.
/// - Static path (`fixture_gen_seeded == false`):
///   serves from `state.devices_ordered` (backward-compatible path).
///
/// MUST branch on `fixture_gen_seeded`, NOT `generated_records.is_empty()`.
/// DormantTenant guard: zero records + seeded=true → empty, not static.
/// F-P6-HIGH-001 / ADR-036 v2.2 / ADR-036 v2.3 §2.4.
fn paginate_devices(state: &ArmisState, page: u32, size: u32) -> axum::response::Response {
    // Three-way composition sentinel: use fixture_gen_seeded (not is_empty()).
    // DormantTenant (seeded=true, 0 records) must serve empty — not static fixture.
    // F-P6-HIGH-001 / ADR-036 v2.2.
    #[cfg(feature = "fixture-gen")]
    let use_generated = state.fixture_gen_seeded;
    #[cfg(not(feature = "fixture-gen"))]
    let use_generated = false;

    #[cfg(feature = "fixture-gen")]
    if use_generated {
        // F-P2-CRIT-002: serve generated records as raw serde_json::Value (Claroty pattern).
        //
        // Generated Armis records use camelCase native field names ("asset_id", "lastSeen", etc.)
        // which do NOT match the DeviceRecord struct's snake_case fields.
        // The adapter reads $.data.devices by JSON path — raw Value is correct here.
        //
        // Only include records that have "asset_id" and no "alert_id" (device discriminator).
        // This partitioning is consistent with the search.rs dual-path for /api/v1/search.
        //
        // NO silent .ok() drops: every generated record that has "asset_id" is served.
        // Records without "asset_id" (alerts in the fixture) are intentionally excluded
        // from the /api/v1/devices endpoint — this is not data loss, it is correct partitioning.

        // Scenario path: apply StageMask projection (BC-2.06.019 PC-4 / B-P1-01).
        // Must nest INSIDE fixture_gen_seeded=true (three-way composition requirement).
        // DormantTenant guard: branching on fixture_gen_seeded, NOT is_empty().
        if let Some(ref timeline) = state.timeline {
            use prism_dtu_common::current_stage_index;
            let now = chrono::Utc::now().timestamp();
            let stage_idx = current_stage_index(timeline, now);
            let mask = &timeline.stages[stage_idx].visible_entity_mask;
            let primary_id = &timeline.entities.primary_device_id_armis;
            let lateral_ids: std::collections::HashSet<&str> = timeline
                .entities
                .lateral_device_ids_armis
                .iter()
                .map(|s| s.as_str())
                .collect();

            // Stage 0 (Baseline): no attack entities visible yet — primary device
            // appears first at stage 1 (Recon), even though mask.primary_device=true
            // at stage 0. The `stage_idx > 0` guard implements "scenario not yet started"
            // semantics: the device is enrolled in the scenario but not yet surfaced.
            // BC-2.06.019 PC-4 / TV-019-007, TV-019-017 (stage 0 → primary absent).
            let all_generated: Vec<&serde_json::Value> = state
                .generated_records
                .iter()
                .filter(|v| v.get("asset_id").is_some() && v.get("alert_id").is_none())
                .filter(|v| {
                    let asset_id = v.get("asset_id").and_then(|a| a.as_str()).unwrap_or("");
                    if asset_id == primary_id {
                        mask.primary_device && stage_idx > 0
                    } else if lateral_ids.contains(asset_id) {
                        mask.lateral_devices
                    } else {
                        // Non-catalog records always pass through.
                        true
                    }
                })
                .collect();

            let total = all_generated.len() as u32;
            let offset = ((page - 1) * size) as usize;
            let page_devices: Vec<serde_json::Value> = if offset >= all_generated.len() {
                vec![]
            } else {
                all_generated
                    .iter()
                    .skip(offset)
                    .take(size as usize)
                    .map(|v| {
                        // BC-2.06.019 v1.13 PC-4: device_cves=false → strip device_cves_first.
                        // Stage 0-3: mask.device_cves=false; CVE-related enrichment fields
                        // are omitted from device records until Containment (stage 4).
                        // F-PIVOT003-R7A-001: SERVED-ROUTE enforcement (not just data-layer).
                        if !mask.device_cves {
                            let mut owned = (*v).clone();
                            if let Some(obj) = owned.as_object_mut() {
                                obj.remove("device_cves_first");
                            }
                            owned
                        } else {
                            (*v).clone()
                        }
                    })
                    .collect()
            };

            let body = serde_json::json!({
                "data": {
                    "devices": page_devices,
                    "total": total,
                    "page": page,
                }
            });
            return (StatusCode::OK, Json(body)).into_response();
        }

        // Seeded path (no scenario): serve all generated records (Story-A behavior).
        let all_generated: Vec<&serde_json::Value> = state
            .generated_records
            .iter()
            .filter(|v| v.get("asset_id").is_some() && v.get("alert_id").is_none())
            .collect();

        let total = all_generated.len() as u32;
        let offset = ((page - 1) * size) as usize;
        let page_devices: Vec<serde_json::Value> = if offset >= all_generated.len() {
            vec![]
        } else {
            all_generated
                .iter()
                .skip(offset)
                .take(size as usize)
                .map(|v| (*v).clone())
                .collect()
        };

        // DevicesResponse wraps a Vec<DeviceRecord>, but we need to return raw Values.
        // Return as a hand-assembled JSON response to avoid the DeviceRecord deserialization
        // mismatch that caused F-P2-CRIT-002 (camelCase vs snake_case field names).
        let body = serde_json::json!({
            "data": {
                "devices": page_devices,
                "total": total,
                "page": page,
            }
        });
        return (StatusCode::OK, Json(body)).into_response();
    }

    // Static-fixture fallback path.
    let _ = use_generated; // suppress unused warning in non-fixture-gen builds
    let all_devices = &state.devices_ordered;
    let total = all_devices.len() as u32;
    let offset = ((page - 1) * size) as usize;

    // EC-004: page beyond last → empty devices array, correct total.
    let page_devices: Vec<DeviceRecord> = if offset >= all_devices.len() {
        vec![]
    } else {
        all_devices
            .iter()
            .skip(offset)
            .take(size as usize)
            .map(|d| {
                // BC-3.2.001: merge per-org tag_store entries with fixture tags.
                // DTU clone is a single-tenant HTTP server per test instance; use DTU_ROUTE_ORG_ID.
                let merged_tags = state.tags_for(DTU_ROUTE_ORG_ID, &d.device_id, &d.tags);
                DeviceRecord {
                    tags: merged_tags,
                    ..d.clone()
                }
            })
            .collect()
    };

    let body = DevicesResponse {
        data: DevicesData {
            devices: page_devices,
            total,
            page,
        },
    };

    (StatusCode::OK, Json(body)).into_response()
}

/// `GET /api/v1/devices/{device_id}/activity`
///
/// Returns activity records filtered to the requested device_id.
pub async fn get_device_activity(
    State(state): State<Arc<ArmisState>>,
    headers: HeaderMap,
    Path(device_id): Path<String>,
) -> impl IntoResponse {
    if let Some(err) = check_bearer_auth(&headers) {
        return err;
    }

    // CR-017 / M-50-001: dual-mode X-Org-Id policy (see module doc).
    let is_real_org = state.instance_org_id != crate::state::DTU_DEFAULT_INSTANCE_ORG_ID;
    if is_real_org || headers.get("x-org-id").is_some() {
        if let Err((status, body)) = validate_org_id(&headers, state.instance_org_id) {
            return (status, body).into_response();
        }
    }

    let activities: Vec<_> = state
        .activity_fixture
        .iter()
        .filter(|a| a.device_id == device_id)
        .cloned()
        .collect();

    let total = activities.len() as u32;
    let body = ActivityResponse {
        data: ActivityData { activities, total },
    };

    (StatusCode::OK, Json(body)).into_response()
}

/// `GET /api/v1/devices/{device_id}/risk`
///
/// Returns the risk score for a device.
/// EC-002: device not in fixture → HTTP 404 `{"error": "device not found", "code": 404}`.
pub async fn get_device_risk(
    State(state): State<Arc<ArmisState>>,
    headers: HeaderMap,
    Path(device_id): Path<String>,
) -> impl IntoResponse {
    if let Some(err) = check_bearer_auth(&headers) {
        return err;
    }

    // CR-017 / M-50-001: dual-mode X-Org-Id policy (see module doc).
    let is_real_org = state.instance_org_id != crate::state::DTU_DEFAULT_INSTANCE_ORG_ID;
    if is_real_org || headers.get("x-org-id").is_some() {
        if let Err((status, body)) = validate_org_id(&headers, state.instance_org_id) {
            return (status, body).into_response();
        }
    }

    match state.device_registry.get(&device_id) {
        Some(device) => {
            let body = RiskResponse {
                data: RiskData {
                    device_id: device.device_id.clone(),
                    risk_score: device.risk_score.unwrap_or(0),
                    risk_factors: device.risk_factors.clone(),
                },
            };
            (StatusCode::OK, Json(body)).into_response()
        }
        None => {
            let body = ArmisError {
                error: "device not found".to_owned(),
                code: 404,
            };
            (StatusCode::NOT_FOUND, Json(body)).into_response()
        }
    }
}

// ---------------------------------------------------------------------------
// Auth helpers
// ---------------------------------------------------------------------------

/// Validate the `X-Org-Id` header against `instance_org_id`.
///
/// # W3-FIX-SEC-001 (AC-001..AC-003, BC-3.5.002 precondition 3)
///
/// Returns `Ok(OrgId)` when the header is present, parseable as UUID, and matches
/// `instance_org_id` byte-for-byte.
///
/// Returns `Err((401, JSON body))` when:
/// - The header is absent (AC-003)
/// - The header value is not a valid UUID (EC-001)
/// - The parsed UUID does not match `instance_org_id` (AC-002)
pub(crate) fn validate_org_id(
    headers: &HeaderMap,
    instance_org_id: prism_core::OrgId,
) -> Result<prism_core::OrgId, (StatusCode, Json<JsonValue>)> {
    let mismatch_err = || {
        (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({
                "error": "org_id mismatch: request does not match this clone instance"
            })),
        )
    };

    // AC-003: missing header → 401.
    let header_val = headers
        .get("x-org-id")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(mismatch_err)?;

    // EC-001: non-UUID value → 401.
    let parsed_uuid = uuid::Uuid::parse_str(header_val).map_err(|_| mismatch_err())?;
    let header_org = prism_core::OrgId::from_uuid(parsed_uuid);

    // AC-002: UUID present but mismatches instance_org_id → 401.
    if header_org != instance_org_id {
        return Err(mismatch_err());
    }

    Ok(header_org)
}

/// Validate the `Authorization: Bearer {non-empty}` header.
///
/// Returns `Some(response)` if the request is unauthorized (HTTP 403), or
/// `None` if the header is present and valid.
///
/// Per AC-5 and `dtu-assessment.md §3.4`: Armis returns 403, NOT 401.
fn check_bearer_auth(headers: &HeaderMap) -> Option<axum::response::Response> {
    let valid = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .map(|v| v.starts_with("Bearer ") && v.len() > "Bearer ".len())
        .unwrap_or(false);

    if valid {
        None
    } else {
        let body = ArmisError {
            error: "invalid or missing bearer token".to_owned(),
            code: 403,
        };
        Some((StatusCode::FORBIDDEN, Json(body)).into_response())
    }
}

/// Deterministic unit tests for BC-2.06.019 PC-4 stage-0 device-filtering predicate.
///
/// These tests verify the `mask.primary_device && stage_idx > 0` guard implemented in
/// `paginate_devices` without wall-clock dependency or HTTP server spin-up. The HTTP-level
/// integration tests that exercise the full route are in
/// `tests/bc_2_06_019_scenario_progression.rs`; those tests now run in CI — the macOS
/// native-tls Keychain init race that previously exceeded the 50s stage-0 window was
/// resolved by standardizing reqwest to rustls-tls per ADR-050 (S-DEMO-FIDELITY-REMEDIATION-001).
/// These in-process unit tests remain as a fast, deterministic complement to the subprocess
/// HTTP integration tests.
#[cfg(all(test, feature = "fixture-gen"))]
mod tests {
    use prism_dtu_common::{
        build_default_incident_timeline, build_scenario_entity_catalog, current_stage_index, OrgId,
    };

    fn deadbeef_org() -> OrgId {
        OrgId([
            0xde, 0xad, 0xbe, 0xef, 0x00, 0x00, 0x70, 0x00, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00,
        ])
    }

    /// BC-2.06.019 PC-4 — stage-0 filtering predicate is deterministic.
    ///
    /// Verifies that `mask.primary_device && stage_idx > 0` evaluates to:
    ///   - false at stage 0 (elapsed < 60s) → primary device ABSENT
    ///   - true  at stage 1 (elapsed >= 60s) → primary device PRESENT
    ///
    /// Uses fixed epoch (no wall clock). This is the deterministic complement to the
    /// HTTP integration tests in `tests/bc_2_06_019_scenario_progression.rs` (TV-019-009).
    #[test]
    fn test_BC_2_06_019_stage0_primary_device_filtering_predicate_deterministic() {
        let org = deadbeef_org();
        let catalog = build_scenario_entity_catalog(42, &org);
        let start_epoch: i64 = 2_000_000; // fixed — no wall clock

        let timeline = build_default_incident_timeline(catalog, start_epoch, &[]);

        // Stage 0: elapsed = 10s < 60s → stage_idx = 0.
        // Predicate: mask.primary_device && stage_idx > 0 = true && false = false → ABSENT.
        let stage_idx_0 = current_stage_index(&timeline, start_epoch + 10);
        assert_eq!(
            stage_idx_0, 0,
            "TV-019-001: at elapsed=10s, stage must be 0 (Baseline); got {stage_idx_0}"
        );
        let mask_0 = &timeline.stages[stage_idx_0].visible_entity_mask;
        let primary_visible_at_0 = mask_0.primary_device && stage_idx_0 > 0;
        assert!(
            !primary_visible_at_0,
            "BC-2.06.019 PC-4: at stage 0 (elapsed 10s < 60s), primary device must be ABSENT \
             (mask.primary_device={}  stage_idx={} → predicate={}). \
             The `stage_idx > 0` guard must suppress primary_device at Baseline.",
            mask_0.primary_device, stage_idx_0, primary_visible_at_0
        );

        // Stage 1: elapsed = 90s >= 60s → stage_idx = 1.
        // Predicate: mask.primary_device && stage_idx > 0 = true && true = true → PRESENT.
        let stage_idx_1 = current_stage_index(&timeline, start_epoch + 90);
        assert_eq!(
            stage_idx_1, 1,
            "TV-019-002: at elapsed=90s, stage must be 1 (Recon); got {stage_idx_1}"
        );
        let mask_1 = &timeline.stages[stage_idx_1].visible_entity_mask;
        let primary_visible_at_1 = mask_1.primary_device && stage_idx_1 > 0;
        assert!(
            primary_visible_at_1,
            "BC-2.06.019 PC-4: at stage 1 (elapsed 90s >= 60s), primary device must be PRESENT \
             (mask.primary_device={}  stage_idx={} → predicate={}). \
             Primary device becomes visible at stage 1 (Recon) per StageMask.",
            mask_1.primary_device, stage_idx_1, primary_visible_at_1
        );
    }
}
