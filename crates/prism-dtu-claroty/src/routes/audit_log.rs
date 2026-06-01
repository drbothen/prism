//! Route handler for the Claroty xDome audit log endpoint.
//!
//! `POST /api/v1/audit_log/get` — audit log list (Gap-CL-006 closure).
//!
//! Response envelope: `{"audit_log": [...], "total": N}`.
//! The key `audit_log` matches `response_path = "$.audit_log"` in
//! `claroty.sensor.toml`. Requires valid `Authorization: Bearer` header (AC-002).

use std::sync::Arc;

use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    Json,
};
use serde_json::Value;

use crate::{routes::devices::check_bearer_auth, state::ClarotyState, types::GetAuditLogBody};

/// `POST /api/v1/audit_log/get`
///
/// Returns synthetic audit log entries from `fixtures/audit-log.json`.
/// Response: `{"audit_log": [...], "total": N}`.
/// Requires valid `Authorization: Bearer` header (AC-002).
///
/// Gap-CL-006: DTU-side closure. `claroty.sensor.toml` `response_path = "$.audit_log"`.
pub async fn list_audit_logs(
    State(_state): State<Arc<ClarotyState>>,
    headers: HeaderMap,
    _body: Option<Json<GetAuditLogBody>>,
) -> (StatusCode, Json<Value>) {
    let _ = check_bearer_auth(&headers);
    todo!("S-DEMO-CLAROTY-AUDIT-DTU-001: implement list_audit_logs — load fixture, check bearer, return JSON envelope")
}
