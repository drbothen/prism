//! Route handlers for vulnerability endpoints.
//!
//! `POST /api/v1/vulnerabilities` — vulnerability inventory.
//! `POST /api/v1/vulnerabilities/{vuln_id}/devices` — devices affected by a vulnerability.

use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use serde_json::Value;

use crate::state::ClarotyState;
use crate::types::{GetVulnerabilitiesBody, GetVulnerabilityDevicesBody};

/// `POST /api/v1/vulnerabilities`
///
/// Returns vulnerability inventory from `fixtures/vulnerabilities.json`.
/// Response: `{"vulnerabilities": [...], "total": N, "page": N}`.
/// Requires valid `Authorization: Bearer` header (AC-5).
pub async fn list_vulnerabilities(
    State(_state): State<Arc<ClarotyState>>,
    _headers: HeaderMap,
    _body: Option<Json<GetVulnerabilitiesBody>>,
) -> (StatusCode, Json<Value>) {
    unimplemented!("vulnerabilities::list_vulnerabilities")
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
    _headers: HeaderMap,
    _body: Option<Json<GetVulnerabilityDevicesBody>>,
) -> (StatusCode, Json<Value>) {
    unimplemented!("vulnerabilities::list_vulnerability_devices")
}
