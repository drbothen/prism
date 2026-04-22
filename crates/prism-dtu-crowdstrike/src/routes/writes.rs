//! Write (mutation) endpoints for the CrowdStrike DTU.
//!
//! - `POST /devices/entities/devices-actions/v2` — contain / lift_containment
//! - `PATCH /detects/entities/detects/v2` — update_status / assign

use axum::http::StatusCode;

/// `POST /devices/entities/devices-actions/v2?action_name=contain`
///
/// Transitions the device to `containment_status: "contained"` in the containment store.
/// Returns HTTP 202 on success.
/// Returns HTTP 400 if `ids` is empty (EC-001).
/// Returns HTTP 400 if device is already contained (EC-002).
pub async fn contain_device() -> StatusCode {
    unimplemented!("writes::contain_device — not yet implemented")
}

/// `POST /devices/entities/devices-actions/v2?action_name=lift_containment`
///
/// Transitions the device to `containment_status: "normal"` in the containment store.
/// Returns HTTP 202 on success.
pub async fn lift_containment() -> StatusCode {
    unimplemented!("writes::lift_containment — not yet implemented")
}

/// `PATCH /detects/entities/detects/v2` — update_status path
///
/// Body: `{"ids": [...], "status": "..."}`.
/// Updates `detection_status_store` for each ID.
/// Returns HTTP 200 `{}` on success.
pub async fn update_detection_status() -> StatusCode {
    unimplemented!("writes::update_detection_status — not yet implemented")
}

/// `PATCH /detects/entities/detects/v2` — assign path
///
/// Body: `{"ids": [...], "assigned_to_uid": "..."}`.
/// Records the assignment. Dispatched when `assigned_to_uid` is present in body.
/// Returns HTTP 200 `{}` on success.
pub async fn assign_detection() -> StatusCode {
    unimplemented!("writes::assign_detection — not yet implemented")
}
