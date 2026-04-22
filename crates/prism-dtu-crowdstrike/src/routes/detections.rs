//! Detection read routes for the CrowdStrike DTU.
//!
//! - `GET /detects/queries/detects/v1` — paginated detection ID list (Step 1)
//! - `POST /detects/entities/summaries/GET/v1` — batch detection detail fetch (Step 2)

use axum::http::StatusCode;

/// `GET /detects/queries/detects/v1`
///
/// Paginated detection ID list. Loads IDs from `fixtures/detections-ids.json`.
/// Registers returned IDs in session registry under `X-DTU-Session-Id`.
/// Returns HTTP 401 if `Authorization` header is absent.
pub async fn list_detection_ids() -> StatusCode {
    unimplemented!("detections::list_detection_ids — not yet implemented")
}

/// `POST /detects/entities/summaries/GET/v1`
///
/// Batch detection detail fetch. Body: `{"ids": ["det-001", ...]}`.
/// Looks up IDs in session registry; returns matching records from
/// `fixtures/detections-detail.json`. Returns HTTP 400 if `ids` is empty.
pub async fn get_detection_summaries() -> StatusCode {
    unimplemented!("detections::get_detection_summaries — not yet implemented")
}
