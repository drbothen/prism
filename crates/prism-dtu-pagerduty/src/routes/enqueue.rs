//! `POST /v2/enqueue` — PagerDuty Events API v2 enqueue handler.
//!
//! Implements the full stateful incident lifecycle:
//! - `trigger`: create or re-open an incident; idempotent for active incidents
//! - `acknowledge`: transition an active incident to `Acknowledged`
//! - `resolve`: transition any incident to `Resolved` (idempotent)
//!
//! Validation order (per PagerDuty spec):
//! 1. Auth mode check — if `auth_reject` is active, return 403 immediately
//! 2. `routing_key` presence — return 400 if missing
//! 3. `event_action` validity — return 400 if not one of the three accepted values
//! 4. `payload.severity` validity — return 400 if not in accepted set (case-sensitive)
//! 5. Business logic per action

use std::sync::Arc;

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};

use crate::state::{IncidentRecord, IncidentStatus, PagerDutyState};
use crate::types::{ActionResponse, EnqueueRequest, PagerDutyError, TriggerResponse};

/// Accepted severity values (case-sensitive per PagerDuty spec).
const VALID_SEVERITIES: &[&str] = &["critical", "error", "warning", "info"];

/// `POST /v2/enqueue`
///
/// Accepts a JSON body and routes to the appropriate incident lifecycle action.
/// Returns HTTP 202 for `trigger` (both new and idempotent) and HTTP 200 for
/// `acknowledge` / `resolve`.
pub async fn post_enqueue(
    State(state): State<Arc<PagerDutyState>>,
    Json(body): Json<EnqueueRequest>,
) -> impl IntoResponse {
    // 1. Auth mode check — must precede all other validation.
    if state.is_auth_reject() {
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({
                "status": "invalid key",
                "message": "Forbidden"
            })),
        )
            .into_response();
    }

    // 2. Validate `routing_key` presence.
    if body
        .routing_key
        .as_deref()
        .map(str::is_empty)
        .unwrap_or(true)
    {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"status": "missing routing_key"})),
        )
            .into_response();
    }

    // 3. Validate `event_action`.
    let event_action = match body.event_action.as_deref() {
        Some("trigger") => "trigger",
        Some("acknowledge") => "acknowledge",
        Some("resolve") => "resolve",
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"status": "invalid event_action"})),
            )
                .into_response();
        }
    };

    // 4. Validate `payload.severity` (only required for `trigger`; skip for ack/resolve
    //    if payload is absent, but validate if payload IS present regardless of action).
    if let Some(payload) = body.payload.as_ref() {
        if let Some(severity) = payload.severity.as_deref() {
            if !VALID_SEVERITIES.contains(&severity) {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({"status": "invalid severity"})),
                )
                    .into_response();
            }
        }
    }

    // 5. Dispatch to action handler.
    match event_action {
        "trigger" => handle_trigger(&state, &body).await,
        "acknowledge" => handle_acknowledge(&state, &body).await,
        "resolve" => handle_resolve(&state, &body).await,
        // Safety: already validated above; this arm is unreachable.
        _ => unreachable!("event_action already validated"),
    }
}

/// Handle `event_action: "trigger"`.
///
/// - If `dedup_key` already exists in registry with status `Triggered` or `Acknowledged`:
///   return 202 without creating a new incident (idempotent per PagerDuty spec).
/// - If `dedup_key` is new OR previous incident was `Resolved`: create new incident
///   with status `Triggered`; return 202.
async fn handle_trigger(state: &Arc<PagerDutyState>, body: &EnqueueRequest) -> Response {
    // Extract or generate dedup_key.
    let dedup_key = body
        .dedup_key
        .clone()
        .filter(|k| !k.is_empty())
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

    let severity = body
        .payload
        .as_ref()
        .and_then(|p| p.severity.clone())
        .unwrap_or_else(|| "info".to_string());

    let summary = body
        .payload
        .as_ref()
        .and_then(|p| p.summary.clone())
        .unwrap_or_default();

    // SAFETY: mutex poison only occurs if a previous holder panicked — not possible in normal operation.
    #[allow(clippy::expect_used)]
    let mut registry = state
        .incident_registry
        .lock()
        .expect("incident_registry poisoned");

    // Idempotency check: if incident is active (Triggered or Acknowledged), no-op.
    if let Some(existing) = registry.get(&dedup_key) {
        if existing.status != IncidentStatus::Resolved {
            let resp = TriggerResponse {
                status: "success".to_string(),
                message: "Event processed".to_string(),
                dedup_key: dedup_key.clone(),
            };
            return (StatusCode::ACCEPTED, Json(resp)).into_response();
        }
    }

    // New incident or re-trigger after resolved — create fresh record.
    registry.insert(
        dedup_key.clone(),
        IncidentRecord {
            dedup_key: dedup_key.clone(),
            status: IncidentStatus::Triggered,
            severity,
            summary,
        },
    );

    let resp = TriggerResponse {
        status: "success".to_string(),
        message: "Event processed".to_string(),
        dedup_key,
    };
    (StatusCode::ACCEPTED, Json(resp)).into_response()
}

/// Handle `event_action: "acknowledge"`.
///
/// - If `dedup_key` not in registry: return 400 `{"status": "invalid dedup_key"}`.
/// - If incident status is `Resolved`: return 400 `{"status": "cannot acknowledge a resolved incident"}`.
/// - Otherwise: transition to `Acknowledged`; return 200 `{"status": "success"}`.
async fn handle_acknowledge(state: &Arc<PagerDutyState>, body: &EnqueueRequest) -> Response {
    let dedup_key = match body.dedup_key.as_deref().filter(|k| !k.is_empty()) {
        Some(k) => k.to_string(),
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"status": "invalid dedup_key"})),
            )
                .into_response();
        }
    };

    // SAFETY: mutex poison only occurs if a previous holder panicked — not possible in normal operation.
    #[allow(clippy::expect_used)]
    let mut registry = state
        .incident_registry
        .lock()
        .expect("incident_registry poisoned");

    match registry.get_mut(&dedup_key) {
        None => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"status": "invalid dedup_key"})),
        )
            .into_response(),
        Some(incident) if incident.status == IncidentStatus::Resolved => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"status": "cannot acknowledge a resolved incident"})),
        )
            .into_response(),
        Some(incident) => {
            incident.status = IncidentStatus::Acknowledged;
            let resp = ActionResponse {
                status: "success".to_string(),
            };
            (StatusCode::OK, Json(resp)).into_response()
        }
    }
}

/// Handle `event_action: "resolve"`.
///
/// - If `dedup_key` not in registry: return 400 `{"status": "invalid dedup_key"}`.
/// - Transition to `Resolved` regardless of current status (resolve is idempotent).
/// - Return 200 `{"status": "success"}`.
async fn handle_resolve(state: &Arc<PagerDutyState>, body: &EnqueueRequest) -> Response {
    let dedup_key = match body.dedup_key.as_deref().filter(|k| !k.is_empty()) {
        Some(k) => k.to_string(),
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"status": "invalid dedup_key"})),
            )
                .into_response();
        }
    };

    // SAFETY: mutex poison only occurs if a previous holder panicked — not possible in normal operation.
    #[allow(clippy::expect_used)]
    let mut registry = state
        .incident_registry
        .lock()
        .expect("incident_registry poisoned");

    match registry.get_mut(&dedup_key) {
        None => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"status": "invalid dedup_key"})),
        )
            .into_response(),
        Some(incident) => {
            incident.status = IncidentStatus::Resolved;
            let resp = ActionResponse {
                status: "success".to_string(),
            };
            (StatusCode::OK, Json(resp)).into_response()
        }
    }
}

// Suppress dead code warnings for error type used only in serialization.
#[allow(dead_code)]
fn _pagerduty_error_used(e: PagerDutyError) -> serde_json::Value {
    serde_json::to_value(e).unwrap_or_default()
}
