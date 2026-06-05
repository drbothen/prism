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
    // falls within the range are included. Open intervals: Gt=exclusive lower, Lt=exclusive upper.
    let (fql_after, fql_before) = params
        .filter
        .as_deref()
        .filter(|s| !s.is_empty())
        .map(crate::state::CrowdstrikeState::parse_fql_time_bounds)
        .unwrap_or((None, None));

    let all_ids = load_detection_ids();

    // When FQL time bounds are present, load the detail fixture to filter by created_timestamp.
    // When no bounds, all IDs pass (no filtering — verbatim passthrough).
    let filtered_ids: Vec<String> = if fql_after.is_some() || fql_before.is_some() {
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
                // Apply bounds: exclusive intervals matching CrowdStrike FQL :> and :< semantics.
                if let Some(after) = fql_after {
                    if ts <= after {
                        return false;
                    }
                }
                if let Some(before) = fql_before {
                    if ts >= before {
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
    let limit = params.limit.unwrap_or(100).min(ordered_ids.len().max(1));
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
