//! Route handlers for vulnerability endpoints.
//!
//! `POST /api/v1/vulnerabilities` — vulnerability inventory.
//! `POST /api/v1/vulnerabilities/{vuln_id}/devices` — devices affected by a vulnerability.

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
use crate::types::{GetVulnerabilitiesBody, GetVulnerabilityDevicesBody};

/// `POST /api/v1/vulnerabilities`
///
/// Returns vulnerability inventory from `fixtures/vulnerabilities.json`.
/// Response: `{"vulnerabilities": [...], "total": N, "page": N}`.
/// Requires valid `Authorization: Bearer` header (AC-5).
pub async fn list_vulnerabilities(
    State(_state): State<Arc<ClarotyState>>,
    headers: HeaderMap,
    _body: Option<Json<GetVulnerabilitiesBody>>,
) -> (StatusCode, Json<Value>) {
    if let Err(err) = check_bearer_auth(&headers) {
        return err;
    }

    let raw = prism_dtu_common::load_fixture(env!("CARGO_MANIFEST_DIR"), "vulnerabilities")
        .expect("fixtures/vulnerabilities.json must exist");
    let vulns = raw
        .as_array()
        .expect("vulnerabilities fixture must be a JSON array")
        .clone();
    let total = vulns.len() as u32;

    (
        StatusCode::OK,
        Json(json!({"vulnerabilities": vulns, "total": total, "page": 1u32})),
    )
}

/// `POST /api/v1/vulnerabilities/{vuln_id}/devices`
///
/// Returns devices affected by the specified vulnerability from
/// `fixtures/vulnerability-devices.json`.
/// Response: `{"devices": [...], "total": N}`.
/// Requires valid `Authorization: Bearer` header (AC-5).
pub async fn list_vulnerability_devices(
    State(_state): State<Arc<ClarotyState>>,
    Path(_vuln_id): Path<String>,
    headers: HeaderMap,
    _body: Option<Json<GetVulnerabilityDevicesBody>>,
) -> (StatusCode, Json<Value>) {
    if let Err(err) = check_bearer_auth(&headers) {
        return err;
    }

    let raw = prism_dtu_common::load_fixture(env!("CARGO_MANIFEST_DIR"), "vulnerability-devices")
        .expect("fixtures/vulnerability-devices.json must exist");
    let devices = raw
        .as_array()
        .expect("vulnerability-devices fixture must be a JSON array")
        .clone();
    let total = devices.len() as u32;

    (
        StatusCode::OK,
        Json(json!({"devices": devices, "total": total})),
    )
}
