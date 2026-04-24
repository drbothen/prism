//! Route handlers for alert endpoints.
//!
//! `POST /api/v1/alerts` — alert list with optional filter params.
//! `POST /api/v1/alerts/{alert_id}/devices` — devices associated with a specific alert.

#![allow(clippy::expect_used)]
use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use serde_json::{json, Value};

use crate::routes::devices::check_bearer_auth;
use crate::state::ClarotyState;
use crate::types::{GetAlertedDevicesBody, GetAlertsBody};

/// `POST /api/v1/alerts`
///
/// Returns alert list from `fixtures/alerts.json`.
/// Response: `{"alerts": [...], "total": N, "page": N}`.
/// Requires valid `Authorization: Bearer` header (AC-5).
pub async fn list_alerts(
    State(_state): State<Arc<ClarotyState>>,
    headers: HeaderMap,
    _body: Option<Json<GetAlertsBody>>,
) -> (StatusCode, Json<Value>) {
    if let Err(err) = check_bearer_auth(&headers) {
        return err;
    }

    let raw = prism_dtu_common::load_fixture(env!("CARGO_MANIFEST_DIR"), "alerts")
        .expect("fixtures/alerts.json must exist");
    let alerts = raw
        .as_array()
        .expect("alerts fixture must be a JSON array")
        .clone();
    let total = alerts.len() as u32;

    (
        StatusCode::OK,
        Json(json!({"alerts": alerts, "total": total, "page": 1u32})),
    )
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
    headers: HeaderMap,
    _body: Option<Json<GetAlertedDevicesBody>>,
) -> (StatusCode, Json<Value>) {
    if let Err(err) = check_bearer_auth(&headers) {
        return err;
    }

    let raw = prism_dtu_common::load_fixture(env!("CARGO_MANIFEST_DIR"), "alerted-devices")
        .expect("fixtures/alerted-devices.json must exist");
    let devices = raw
        .as_array()
        .expect("alerted-devices fixture must be a JSON array")
        .clone();
    let total = devices.len() as u32;

    (
        StatusCode::OK,
        Json(json!({"devices": devices, "total": total})),
    )
}
