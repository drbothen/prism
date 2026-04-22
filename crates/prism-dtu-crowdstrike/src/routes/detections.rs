//! Detection read routes for the CrowdStrike DTU.
//!
//! - `GET /detects/queries/detects/v1` — paginated detection ID list (Step 1)
//! - `POST /detects/entities/summaries/GET/v1` — batch detection detail fetch (Step 2)

use std::sync::Arc;

use axum::extract::{Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Json};
use serde::Deserialize;

use crate::state::{CrowdstrikeState, SessionData};

/// Query params for detection ID list.
#[derive(Debug, Deserialize, Default)]
pub struct DetectionListParams {
    /// FQL filter string — accepted but not parsed.
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
    serde_json::from_str::<Vec<String>>(raw)
        .expect("detections-ids.json must be a JSON array of strings")
}

/// Load detection detail objects from the embedded fixture, keyed by detection_id.
fn load_detection_details() -> std::collections::HashMap<String, serde_json::Value> {
    let raw = include_str!("../../fixtures/detections-detail.json");
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
fn check_auth(headers: &HeaderMap) -> Result<(), axum::response::Response> {
    let auth = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    // Strip "Bearer " prefix and check that a token follows.
    // Per spec: auth_required endpoints must 401 on missing or empty bearer.
    let token = auth.strip_prefix("Bearer ").unwrap_or("").trim();
    if token.is_empty() {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({
                "errors": [{"code": 401, "message": "access denied, authorization required"}]
            })),
        )
            .into_response());
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
        return e;
    }

    let all_ids = load_detection_ids();

    // Apply seed-based ordering for determinism.
    let seed = state
        .runtime_config
        .lock()
        .expect("runtime_config poisoned")
        .seed;
    let ordered_ids = shuffle_ids_by_seed(&all_ids, seed);

    let offset = params.offset.unwrap_or(0);
    let limit = params.limit.unwrap_or(100).min(all_ids.len());
    let total = ordered_ids.len();

    let page: Vec<String> = ordered_ids
        .into_iter()
        .skip(offset)
        .take(limit)
        .collect();

    // Register IDs in session registry if X-DTU-Session-Id header is present.
    if let Some(session_id) = headers.get("x-dtu-session-id").and_then(|v| v.to_str().ok()) {
        let mut registry = state
            .session_registry
            .lock()
            .expect("session_registry poisoned");
        let entry = registry
            .get_or_insert_mut(session_id.to_owned(), || SessionData {
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
        return e;
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
    let allowed_ids = if let Some(session_id) =
        headers.get("x-dtu-session-id").and_then(|v| v.to_str().ok())
    {
        let registry = state
            .session_registry
            .lock()
            .expect("session_registry poisoned");
        if let Some(session_data) = registry.peek(session_id) {
            let registered: std::collections::HashSet<&str> =
                session_data.detection_ids.iter().map(|s| s.as_str()).collect();
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
