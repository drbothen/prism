//! Host read routes for the CrowdStrike DTU.
//!
//! - `GET /devices/queries/devices/v1` — paginated host ID list (Step 1)
//! - `GET /devices/entities/devices/v2` — batch host detail fetch (Step 2)

use axum::http::StatusCode;

/// `GET /devices/queries/devices/v1`
///
/// Paginated host ID list. Loads IDs from `fixtures/hosts-ids.json`.
/// Registers returned IDs in session registry under `X-DTU-Session-Id`.
/// Supports `filter` (FQL string, accepted but not parsed), `limit`, `offset` query params.
pub async fn list_host_ids() -> StatusCode {
    unimplemented!("hosts::list_host_ids — not yet implemented")
}

/// `GET /devices/entities/devices/v2`
///
/// Batch host detail fetch. Query param: `ids` (repeated, e.g., `?ids=h-001&ids=h-002`).
/// Loads base records from `fixtures/hosts-detail.json` and merges `containment_status`
/// from the `containment_store` for each device.
pub async fn get_host_details() -> StatusCode {
    unimplemented!("hosts::get_host_details — not yet implemented")
}
