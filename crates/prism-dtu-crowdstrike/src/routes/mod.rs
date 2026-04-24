//! Route modules for the CrowdStrike DTU.

pub mod detections;
pub mod hosts;
pub mod oauth;
pub mod writes;

use std::sync::Arc;

use axum::extract::State as AxumState;
use axum::extract::State;
use axum::http::{HeaderMap, Request, StatusCode};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Json, Response};
use axum::routing::{get, patch, post};
use axum::Router;
use prism_dtu_common::{FailureMode, LatencyLayer};

use crate::state::CrowdstrikeState;

/// `GET /dtu/health` — DTU introspection endpoint. No auth required.
///
/// Returns HTTP 200 with `{"status": "ok"}`. Used by `FidelityValidator` as a
/// no-auth probe per ADR-003 §Decision Conflict #2 Option C.
async fn dtu_health() -> impl IntoResponse {
    (StatusCode::OK, Json(serde_json::json!({"status": "ok"}))).into_response()
}

/// `POST /dtu/reset` — DTU introspection endpoint. No auth required.
///
/// Clears all mutable state (containment store, detection status store, session
/// registry) and returns HTTP 200 with `{"status": "ok"}`. Used by
/// `FidelityValidator` and test harnesses per ADR-003 §Decision Conflict #2.
async fn dtu_reset(State(state): State<Arc<CrowdstrikeState>>) -> impl IntoResponse {
    state.reset();
    (StatusCode::OK, Json(serde_json::json!({"status": "ok"}))).into_response()
}

/// `POST /dtu/configure` — DTU introspection endpoint.
///
/// Applies runtime configuration from the JSON body (e.g. `{"auth_mode": "reject"}`).
///
/// # ADR-003 Amendment #5 (TD-WV0-07)
///
/// Requires a valid `X-Admin-Token` header matching `state.admin_token`.
/// Missing or incorrect token → HTTP 401 with `{"error": "..."}`.
async fn dtu_configure(
    State(state): State<Arc<CrowdstrikeState>>,
    headers: HeaderMap,
    Json(body): Json<serde_json::Value>,
) -> impl IntoResponse {
    let provided = headers
        .get("x-admin-token")
        .and_then(|v| v.to_str().ok());
    if provided != Some(state.admin_token.as_str()) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"error": "missing or invalid X-Admin-Token"})),
        )
            .into_response();
    }
    match state.apply_config(&body) {
        Ok(()) => (StatusCode::OK, Json(serde_json::json!({"status": "ok"}))).into_response(),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// Axum middleware that applies `FailureMode` injection, using the shared
/// request counter from `CrowdstrikeState`.
///
/// Unlike using `FailureLayer` from `prism-dtu-common` directly, this approach
/// stores the counter in the shared state so all route groups share the same count.
/// (axum's `Router::layer()` clones the layer per route group, which would create
/// independent counters in `FailureLayer`'s `layer()` implementation.)
async fn failure_injection_middleware(
    AxumState((state, mode)): AxumState<(Arc<CrowdstrikeState>, Arc<FailureMode>)>,
    req: Request<axum::body::Body>,
    next: Next,
) -> Response {
    let count = state.next_request_count();

    match mode.as_ref() {
        FailureMode::AuthReject => (
            StatusCode::UNAUTHORIZED,
            axum::Json(serde_json::json!({
                "errors": [{"code": 401, "message": "invalid_client"}]
            })),
        )
            .into_response(),
        FailureMode::RateLimit {
            after_n_requests,
            retry_after_secs,
        } => {
            if count > *after_n_requests {
                (
                    StatusCode::TOO_MANY_REQUESTS,
                    [(
                        axum::http::header::RETRY_AFTER,
                        retry_after_secs.to_string(),
                    )],
                    axum::body::Body::empty(),
                )
                    .into_response()
            } else {
                next.run(req).await
            }
        }
        FailureMode::InternalError { at_request_n } => {
            if count == *at_request_n {
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            } else {
                next.run(req).await
            }
        }
        FailureMode::NetworkTimeout { after_ms } => {
            tokio::time::sleep(std::time::Duration::from_millis(after_ms + 1)).await;
            next.run(req).await
        }
        FailureMode::None => next.run(req).await,
        FailureMode::Unprocessable { at_request_n } => {
            if count == *at_request_n {
                StatusCode::UNPROCESSABLE_ENTITY.into_response()
            } else {
                next.run(req).await
            }
        }
        FailureMode::MalformedResponse => {
            // Return a non-JSON body to exercise Prism's parse-error path (EC-006).
            axum::response::Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(
                    b"\xff\xfe{not valid json!@#$%^&*(" as &[u8],
                ))
                .expect("build malformed response")
        }
    }
}

/// Build the full axum router for the CrowdStrike DTU.
///
/// Wires all 8 in-scope endpoints (4 read, 4 write) plus the OAuth token endpoint.
/// Wraps with `LatencyLayer` (from prism-dtu-common) and a custom axum middleware
/// for `FailureMode` injection that uses the shared counter in `CrowdstrikeState`.
pub fn build_router(
    state: Arc<CrowdstrikeState>,
    failure_mode: FailureMode,
    latency_ms: u64,
) -> Router {
    let failure_mode = Arc::new(failure_mode);

    let router = Router::new()
        // DTU introspection endpoints (no auth required — fidelity probe targets per ADR-003).
        .route("/dtu/health", get(dtu_health))
        .route("/dtu/reset", post(dtu_reset))
        .route("/dtu/configure", post(dtu_configure))
        // OAuth2 token endpoint (no auth required to call).
        .route("/oauth2/token", post(oauth::token))
        // Detection read endpoints.
        .route(
            "/detects/queries/detects/v1",
            get(detections::list_detection_ids),
        )
        .route(
            "/detects/entities/summaries/GET/v1",
            post(detections::get_detection_summaries),
        )
        // Host read endpoints.
        .route("/devices/queries/devices/v1", get(hosts::list_host_ids))
        .route("/devices/entities/devices/v2", get(hosts::get_host_details))
        // Write endpoints.
        .route(
            "/devices/entities/devices-actions/v2",
            post(writes::device_actions),
        )
        .route(
            "/detects/entities/detects/v2",
            patch(writes::patch_detections),
        )
        .with_state(Arc::clone(&state))
        // Axum middleware for failure injection: uses state-held counter
        // so the count is shared across all routes.
        .route_layer(middleware::from_fn_with_state(
            (Arc::clone(&state), Arc::clone(&failure_mode)),
            failure_injection_middleware,
        ));

    // Wrap with LatencyLayer for optional artificial delay.
    router.layer(LatencyLayer { latency_ms })
}
