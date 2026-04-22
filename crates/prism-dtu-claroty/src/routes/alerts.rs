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
use serde_json::Value;

use crate::state::ClarotyState;
use crate::types::{GetAlertedDevicesBody, GetAlertsBody};

/// `POST /api/v1/alerts`
///
/// Returns alert list from `fixtures/alerts.json`.
/// Response: `{"alerts": [...], "total": N, "page": N}`.
/// Requires valid `Authorization: Bearer` header (AC-5).
pub async fn list_alerts(
    State(_state): State<Arc<ClarotyState>>,
    _headers: HeaderMap,
    _body: Option<Json<GetAlertsBody>>,
) -> (StatusCode, Json<Value>) {
    unimplemented!("alerts::list_alerts")
}

/// `POST /api/v1/alerts/{alert_id}/devices`
///
/// Returns devices associated with the specified alert from
/// `fixtures/alerted-devices.json`.
/// Response: `{"devices": [...], "total": N}`.
/// Requires valid `Authorization: Bearer` header (AC-5).
pub async fn list_alerted_devices(
    State(_state): State<Arc<ClarotyState>>,
    Path(_alert_id): Path<String>,
    _headers: HeaderMap,
    _body: Option<Json<GetAlertedDevicesBody>>,
) -> (StatusCode, Json<Value>) {
    unimplemented!("alerts::list_alerted_devices")
}
