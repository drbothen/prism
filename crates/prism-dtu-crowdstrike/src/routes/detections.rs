//! Detection read routes for the CrowdStrike DTU.
//!
//! - `GET /detects/queries/detects/v1` — paginated detection ID list (Step 1)
//! - `POST /detects/entities/summaries/GET/v1` — batch detection detail fetch (Step 2)
//!
//! # FQL filter honoring (F-P1-OBS-001 / AC-CWS-002)
//!
//! The `filter` query param is parsed for `created_timestamp:>'T'` / `created_timestamp:<'T'`
//! CrowdStrike FQL clauses. When present, only IDs whose `created_timestamp` in the
//! detail fixture falls within the range are returned from Step 1.
//! The verbatim filter string is captured in `state.filter_log` before any parsing
//! (parallel to Armis R-DTU-002 opaque-capture pattern). Accessible via GET /dtu/filter-log.

use std::sync::Arc;

use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Json},
};
use prism_core::OrgId;
use serde::Deserialize;

use crate::{
    routes::hosts::validate_org_id,
    state::{CrowdstrikeState, SessionData},
};

/// Query params for detection ID list.
#[derive(Debug, Deserialize, Default)]
pub struct DetectionListParams {
    /// FQL filter string — captured verbatim in filter_log, then parsed for
    /// `created_timestamp:>'T'` / `created_timestamp:<'T'` time bounds (F-P1-OBS-001).
    pub filter: Option<String>,
    /// Maximum results to return (default 100).
    pub limit: Option<usize>,
    /// Offset into the result set.
    pub offset: Option<usize>,
}

/// Body for batch detection detail fetch.
#[derive(Debug, Deserialize)]
pub struct GetDetectionSummariesBody {
    pub ids: Vec<String>,
}

/// Load detection IDs from the embedded fixture.
fn load_detection_ids() -> Vec<String> {
    let raw = include_str!("../../fixtures/detections-ids.json");
    // SAFETY: fixture is compiled in via include_str!; parse failure means a corrupt
    // build artifact — panicking at startup is correct behaviour.
    #[allow(clippy::expect_used)]
    serde_json::from_str::<Vec<String>>(raw)
        .expect("detections-ids.json must be a JSON array of strings")
}

/// Load detection detail objects from the embedded fixture, keyed by detection_id.
fn load_detection_details() -> std::collections::HashMap<String, serde_json::Value> {
    let raw = include_str!("../../fixtures/detections-detail.json");
    // SAFETY: fixture is compiled in via include_str!; parse failure means a corrupt
    // build artifact — panicking at startup is correct behaviour.
    #[allow(clippy::expect_used)]
    let records: Vec<serde_json::Value> =
        serde_json::from_str(raw).expect("detections-detail.json must be a JSON array");
    let mut map = std::collections::HashMap::new();
    for record in records {
        if let Some(id) = record.get("detection_id").and_then(|v| v.as_str()) {
            map.insert(id.to_owned(), record);
        }
    }
    map
}

/// Validate the `Authorization` header.
///
/// Returns `Ok(())` if the header is present and non-empty.
/// Returns an error response if missing or empty.
fn check_auth(headers: &HeaderMap) -> Result<(), Box<axum::response::Response>> {
    let auth = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    // Strip "Bearer " prefix and check that a token follows.
    // Per spec: auth_required endpoints must 401 on missing or empty bearer.
    let token = auth.strip_prefix("Bearer ").unwrap_or("").trim();
    if token.is_empty() {
        return Err(Box::new(
            (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({
                    "errors": [{"code": 401, "message": "access denied, authorization required"}]
                })),
            )
                .into_response(),
        ));
    }
    Ok(())
}

/// Shuffle IDs deterministically using the seed from the state.
///
/// AC-6 seed scope: the seed influences ordering of the IDs in the `resources`
/// array, not fixture content. This makes two calls with the same seed return
/// identical responses (deterministic), while different seeds produce different
/// orderings (different responses).
fn shuffle_ids_by_seed(ids: &[String], seed: u64) -> Vec<String> {
    use rand::seq::SliceRandom;
    let mut rng = prism_dtu_common::seeded_rng(seed);
    let mut shuffled = ids.to_vec();
    shuffled.shuffle(&mut rng);
    shuffled
}

/// `GET /detects/queries/detects/v1`
///
/// Paginated detection ID list. Loads IDs from `fixtures/detections-ids.json`.
/// Registers returned IDs in session registry under `X-DTU-Session-Id`.
/// Returns HTTP 401 if `Authorization` header is absent or empty.
///
/// Three-way composition (ADR-036 v2.3 §2.4, BC-2.06.019 PC-4, BPRL-P4-02):
/// - Scenario path (`fixture_gen_seeded=true && timeline.is_some()`): apply StageMask.
///   Detections whose `device_id` equals the primary CrowdStrike device are withheld
///   while `stage_idx == 0` (mirror of hosts.rs `stage_idx > 0` guard).
///   Detections referencing lateral devices are withheld when `mask.lateral_devices=false`.
///   Non-catalog detections are always visible.
/// - Seeded path (`fixture_gen_seeded=true && timeline.is_none()`): all generated IDs.
/// - Static path (`fixture_gen_seeded=false`): embedded fixture.
///
/// Detections route added to BC-2.06.019 PC-4 coverage matrix per D-1109.
/// Use fixture_gen_seeded (not generated_detections.is_empty()) — DormantTenant guard.
pub async fn list_detection_ids(
    State(state): State<Arc<CrowdstrikeState>>,
    Query(params): Query<DetectionListParams>,
    headers: HeaderMap,
) -> impl IntoResponse {
    if let Err(e) = check_auth(&headers) {
        return *e;
    }

    // CR-018: validate X-Org-Id against instance_org_id (W3-FIX-SEC-001).
    // Guard is active only for real-org clones (instance_org_id != nil).
    // Nil-instance clones (CrowdstrikeClone::new()) skip the guard for backward compat (EC-007).
    // CrowdStrike sentinel: OrgId::from_uuid(Uuid::nil()) — NOT DTU_DEFAULT_INSTANCE_ORG_ID.
    if state.instance_org_id != OrgId::from_uuid(uuid::Uuid::nil()) {
        if let Err((status, body)) = validate_org_id(&headers, state.instance_org_id) {
            return (status, body).into_response();
        }
    }

    // F-P1-OBS-001: capture filter verbatim in filter_log before any parsing
    // (parallel to Armis R-DTU-002 opaque-capture pattern).
    if let Some(ref fql) = params.filter {
        if !fql.is_empty() {
            state.capture_filter(fql);
        }
    }

    // F-P1-OBS-001: parse FQL time bounds for fixture filtering.
    // CrowdStrike FQL syntax: `created_timestamp:>'YYYY-MM-DDTHH:MM:SSZ'` (lower)
    //                          `created_timestamp:<'YYYY-MM-DDTHH:MM:SSZ'` (upper)
    // When present, only IDs whose `created_timestamp` in the detail fixture
    // falls within the range are included. Inclusive boundary semantics: records with
    // ts == bound are KEPT here so push-down result ⊇ exact DataFusion result (ADV-P08-MED-001).
    let (fql_after, fql_before) = params
        .filter
        .as_deref()
        .filter(|s| !s.is_empty())
        .map(crate::state::CrowdstrikeState::parse_fql_time_bounds)
        .unwrap_or((None, None));

    // Three-way composition (ADR-036 v2.3 §2.4, BC-2.06.019 PC-4, BPRL-P4-02):
    // - Scenario path (fixture_gen_seeded=true && timeline.is_some()): apply StageMask.
    // - Seeded path (fixture_gen_seeded=true && timeline.is_none()): all generated IDs.
    // - Static path (fixture_gen_seeded=false): embedded fixture.
    // Use fixture_gen_seeded (not generated_detections.is_empty()) — DormantTenant guard.
    // F-P10-HIGH-001 / F-P6-HIGH-001 / ADR-036 v2.2.
    // Generated records are immutable after construction — no lock needed.
    #[cfg(feature = "fixture-gen")]
    let all_ids: Vec<String> = if state.fixture_gen_seeded {
        if let Some(ref timeline) = state.timeline {
            // Scenario path: apply StageMask projection (BC-2.06.019 PC-4 / BPRL-P4-02).
            // Mirror hosts.rs list_host_ids scenario logic exactly.
            // Detections referencing the primary device are withheld at stage 0 (Baseline)
            // because the primary device itself is withheld from hosts.rs at stage 0.
            // Narrative coherence: a detection cannot reference a device that doesn't exist yet.
            // Detections route added to PC-4 coverage matrix per D-1109.
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

            // Build a lookup map from detection_id → device_id for the stage filter.
            let det_device_map: std::collections::HashMap<&str, &str> = state
                .generated_detections
                .iter()
                .filter_map(|rec| {
                    let det_id = rec.get("detection_id").and_then(|v| v.as_str())?;
                    let dev_id = rec.get("device_id").and_then(|v| v.as_str())?;
                    Some((det_id, dev_id))
                })
                .collect();

            // Pre-compute catalog IOC hash set for O(1) ioc_hashes gate.
            // BC-2.06.019 PC-4 / F-PIVOT003-R7A-002: ioc_hashes=false → withhold
            // detection records whose behaviors[].ioc_value matches a catalog IOC hash.
            let catalog_ioc_hashes: std::collections::HashSet<&str> = timeline
                .entities
                .ioc_hashes
                .iter()
                .map(|s| s.as_str())
                .collect();

            state
                .generated_detections
                .iter()
                .filter_map(|rec| {
                    let det_id = rec.get("detection_id").and_then(|v| v.as_str())?;
                    let dev_id = det_device_map.get(det_id).copied().unwrap_or("");
                    // Apply StageMask: mirror hosts.rs primary_device guard.
                    // stage_idx > 0 guard: BC-2.06.019 PC-4 / BPRL-P4-02.
                    let visible = if dev_id == primary_id {
                        mask.primary_device && stage_idx > 0
                    } else if lateral_ids.contains(dev_id) {
                        mask.lateral_devices
                    } else {
                        true
                    };
                    if !visible {
                        return None;
                    }
                    // BC-2.06.019 PC-4 / F-PIVOT003-R7A-002: ioc_hashes gate.
                    // When mask.ioc_hashes=false, withhold detections whose
                    // behaviors[].ioc_value is in the catalog IOC hash set.
                    // Mirrors Cyberint's ioc_hashes=false filter on alerts.rs.
                    if !mask.ioc_hashes {
                        if let Some(behaviors) = rec.get("behaviors").and_then(|v| v.as_array()) {
                            for behavior in behaviors {
                                if let Some(ioc_val) =
                                    behavior.get("ioc_value").and_then(|v| v.as_str())
                                {
                                    if catalog_ioc_hashes.contains(ioc_val) {
                                        return None; // withhold IOC-bearing detection
                                    }
                                }
                            }
                        }
                    }
                    Some(det_id.to_owned())
                })
                .collect()
        } else {
            // Seeded path (no scenario): all generated detection IDs (Story-A behavior).
            state
                .generated_detections
                .iter()
                .filter_map(|rec| {
                    rec.get("detection_id")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_owned())
                })
                .collect()
        }
    } else {
        load_detection_ids()
    };
    #[cfg(not(feature = "fixture-gen"))]
    let all_ids = load_detection_ids();

    // When FQL time bounds are present, filter by created_timestamp against the
    // SAME source the IDs came from. When no bounds, all IDs pass (verbatim passthrough).
    //
    // F7 / CS-04 (review 2026-06-10): on the seeded path the time-window filter
    // previously consulted the STATIC load_detection_details() map — generated
    // detection IDs never match static IDs, so seeded clones returned ZERO rows
    // for ANY created_timestamp-bounded query. The filter source must follow the
    // dual-path sentinel: generated_detections when fixture_gen_seeded, static
    // detail fixture otherwise. Same inclusive FQL boundary semantics either way.
    let filtered_ids: Vec<String> = if fql_after.is_some() || fql_before.is_some() {
        #[cfg(feature = "fixture-gen")]
        let details: std::collections::HashMap<String, serde_json::Value> =
            if state.fixture_gen_seeded {
                state
                    .generated_detections
                    .iter()
                    .filter_map(|rec| {
                        rec.get("detection_id")
                            .and_then(|v| v.as_str())
                            .map(|id| (id.to_owned(), rec.clone()))
                    })
                    .collect()
            } else {
                load_detection_details()
            };
        #[cfg(not(feature = "fixture-gen"))]
        let details = load_detection_details();
        all_ids
            .into_iter()
            .filter(|id| {
                // Include ID only if its created_timestamp falls within the FQL bounds.
                // IDs without a detail record are excluded (conservative).
                let Some(record) = details.get(id) else {
                    return false;
                };
                let Some(ts_str) = record.get("created_timestamp").and_then(|v| v.as_str()) else {
                    return false;
                };
                let Some(ts) = ts_str
                    .parse::<chrono::DateTime<chrono::Utc>>()
                    .ok()
                    .or_else(|| {
                        chrono::NaiveDateTime::parse_from_str(ts_str, "%Y-%m-%dT%H:%M:%S")
                            .ok()
                            .map(|n| n.and_utc())
                    })
                else {
                    return false;
                };
                // Apply bounds: inclusive boundary semantics so push-down result ⊇ exact result
                // (BC-2.11.007 result-equivalence invariant). For a strict `>` predicate the
                // FQL emits `:>` but we keep the boundary record here — DataFusion's exact
                // post-filter removes it if the PrismQL predicate was strict (ts > bound).
                // For an inclusive `>=` predicate the boundary record must pass both push-down
                // and DataFusion → result-equivalence holds (ADV-P08-MED-001 fix).
                if let Some(after) = fql_after {
                    if ts < after {
                        return false;
                    }
                }
                if let Some(before) = fql_before {
                    if ts > before {
                        return false;
                    }
                }
                true
            })
            .collect()
    } else {
        all_ids
    };

    // Apply seed-based ordering for determinism.
    // SAFETY: mutex poison only occurs if a previous holder panicked — not possible in normal operation.
    #[allow(clippy::expect_used)]
    let seed = state
        .runtime_config
        .lock()
        .expect("runtime_config poisoned")
        .seed;
    let ordered_ids = shuffle_ids_by_seed(&filtered_ids, seed);

    let offset = params.offset.unwrap_or(0);
    // Limit clamp mirrors sibling hosts.rs:166 form (`unwrap_or(100).min(len)`).
    // Production push-down never sends limit=0 (EC-008 maps limit=0 → empty row →
    // stripped before the AQL/FQL call), so `?limit=0` correctly yields 0 records.
    let limit = params.limit.unwrap_or(100).min(ordered_ids.len());
    let total = ordered_ids.len();

    let page: Vec<String> = ordered_ids.into_iter().skip(offset).take(limit).collect();

    // Register IDs in session registry if X-DTU-Session-Id header is present.
    if let Some(session_id) = headers
        .get("x-dtu-session-id")
        .and_then(|v| v.to_str().ok())
    {
        // SAFETY: mutex poison only occurs if a previous holder panicked — not possible in normal operation.
        #[allow(clippy::expect_used)]
        let mut registry = state
            .session_registry
            .lock()
            .expect("session_registry poisoned");
        let entry = registry.get_or_insert_mut(session_id.to_owned(), || SessionData {
            detection_ids: Vec::new(),
            host_ids: Vec::new(),
        });
        // Accumulate all IDs returned so far for this session.
        for id in &page {
            if !entry.detection_ids.contains(id) {
                entry.detection_ids.push(id.clone());
            }
        }
    }

    let next_token = if offset + limit < total {
        Some(format!("offset={}", offset + limit))
    } else {
        None
    };

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "resources": page,
            "meta": {
                "pagination": {
                    "offset": offset,
                    "limit": limit,
                    "total": total
                }
            },
            "next_token": next_token
        })),
    )
        .into_response()
}

/// `POST /detects/entities/summaries/GET/v1`
///
/// Batch detection detail fetch. Body: `{"ids": ["det-001", ...]}`.
/// Looks up IDs in session registry; returns matching records from
/// `fixtures/detections-detail.json`. Returns HTTP 400 if `ids` is empty.
///
/// Three-way composition (ADR-036 v2.3 §2.4, BC-2.06.019 PC-4, BPRL-P4-02):
/// In scenario mode (`timeline.is_some()`), the details lookup map is filtered
/// by StageMask before assembly — mirroring hosts.rs `get_host_details` logic.
/// Detections referencing the primary device are withheld at stage 0 (Baseline).
/// Detections route added to BC-2.06.019 PC-4 coverage matrix per D-1109.
pub async fn get_detection_summaries(
    State(state): State<Arc<CrowdstrikeState>>,
    headers: HeaderMap,
    Json(body): Json<GetDetectionSummariesBody>,
) -> impl IntoResponse {
    if let Err(e) = check_auth(&headers) {
        return *e;
    }

    // CR-018: validate X-Org-Id against instance_org_id (W3-FIX-SEC-001).
    // Guard is active only for real-org clones (instance_org_id != nil).
    // Nil-instance clones skip the guard for backward compat (EC-007).
    // CrowdStrike sentinel: OrgId::from_uuid(Uuid::nil()) — NOT DTU_DEFAULT_INSTANCE_ORG_ID.
    if state.instance_org_id != OrgId::from_uuid(uuid::Uuid::nil()) {
        if let Err((status, body_err)) = validate_org_id(&headers, state.instance_org_id) {
            return (status, body_err).into_response();
        }
    }

    if body.ids.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "errors": [{"code": 400, "message": "ids array must not be empty"}]
            })),
        )
            .into_response();
    }

    // Three-way composition (ADR-036 v2.3 §2.4, BC-2.06.019 PC-4, BPRL-P4-02):
    // - Scenario path (fixture_gen_seeded=true && timeline.is_some()): filter by StageMask.
    //   Detections referencing the primary device are withheld at stage 0 (Baseline).
    //   Mirrors hosts.rs get_host_details scenario_stage_ctx logic exactly.
    //   Detections route added to PC-4 coverage matrix per D-1109.
    // - Seeded path (fixture_gen_seeded=true && timeline.is_none()): all generated records.
    // - Static path (fixture_gen_seeded=false): embedded fixture.
    // Use fixture_gen_seeded (not generated_detections.is_empty()) — DormantTenant guard.
    // F-P10-HIGH-001 / F-P6-HIGH-001 / ADR-036 v2.2.
    #[cfg(feature = "fixture-gen")]
    let details: std::collections::HashMap<String, serde_json::Value> = if state.fixture_gen_seeded
    {
        if let Some(ref timeline) = state.timeline {
            // Scenario path: filter by StageMask projection (BC-2.06.019 PC-4 / BPRL-P4-02).
            // Mirror hosts.rs get_host_details fixture-build filtering exactly.
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

            // Pre-compute catalog IOC hash set for O(1) ioc_hashes gate.
            // BC-2.06.019 PC-4 / F-PIVOT003-R7A-002: ioc_hashes=false → withhold
            // detection records whose behaviors[].ioc_value matches a catalog IOC hash.
            let catalog_ioc_hashes_summaries: std::collections::HashSet<&str> = timeline
                .entities
                .ioc_hashes
                .iter()
                .map(|s| s.as_str())
                .collect();

            state
                .generated_detections
                .iter()
                .filter_map(|rec| {
                    let det_id = rec.get("detection_id").and_then(|v| v.as_str())?;
                    let dev_id = rec.get("device_id").and_then(|v| v.as_str()).unwrap_or("");
                    // Apply StageMask based on the referenced device_id.
                    // stage_idx > 0 guard: BC-2.06.019 PC-4 / BPRL-P4-02.
                    let visible = if dev_id == primary_id {
                        mask.primary_device && stage_idx > 0
                    } else if lateral_ids.contains(dev_id) {
                        mask.lateral_devices
                    } else {
                        true
                    };
                    if !visible {
                        return None;
                    }
                    // BC-2.06.019 PC-4 / F-PIVOT003-R7A-002: ioc_hashes gate.
                    // When mask.ioc_hashes=false, withhold detections whose
                    // behaviors[].ioc_value is in the catalog IOC hash set.
                    // Mirrors list_detection_ids scenario path and Cyberint alerts.rs.
                    if !mask.ioc_hashes {
                        if let Some(behaviors) = rec.get("behaviors").and_then(|v| v.as_array()) {
                            for behavior in behaviors {
                                if let Some(ioc_val) =
                                    behavior.get("ioc_value").and_then(|v| v.as_str())
                                {
                                    if catalog_ioc_hashes_summaries.contains(ioc_val) {
                                        return None; // withhold IOC-bearing detection
                                    }
                                }
                            }
                        }
                    }
                    Some((det_id.to_owned(), rec.clone()))
                })
                .collect()
        } else {
            // Seeded path (no scenario): all generated detection records.
            state
                .generated_detections
                .iter()
                .filter_map(|rec| {
                    rec.get("detection_id")
                        .and_then(|v| v.as_str())
                        .map(|id| (id.to_owned(), rec.clone()))
                })
                .collect()
        }
    } else {
        load_detection_details()
    };
    #[cfg(not(feature = "fixture-gen"))]
    let details = load_detection_details();

    // Filter requested IDs against session registry if session header present.
    let allowed_ids = if let Some(session_id) = headers
        .get("x-dtu-session-id")
        .and_then(|v| v.to_str().ok())
    {
        // SAFETY: mutex poison only occurs if a previous holder panicked — not possible in normal operation.
        #[allow(clippy::expect_used)]
        let registry = state
            .session_registry
            .lock()
            .expect("session_registry poisoned");
        if let Some(session_data) = registry.peek(session_id) {
            let registered: std::collections::HashSet<&str> = session_data
                .detection_ids
                .iter()
                .map(|s| s.as_str())
                .collect();
            body.ids
                .iter()
                .filter(|id| registered.contains(id.as_str()))
                .cloned()
                .collect::<Vec<_>>()
        } else {
            // Session not in registry → return empty (EC-003).
            Vec::new()
        }
    } else {
        // No session header → use all requested IDs directly.
        body.ids.clone()
    };

    let resources: Vec<serde_json::Value> = allowed_ids
        .into_iter()
        .filter_map(|id| details.get(&id).cloned())
        .collect();

    (
        StatusCode::OK,
        Json(serde_json::json!({ "resources": resources })),
    )
        .into_response()
}
