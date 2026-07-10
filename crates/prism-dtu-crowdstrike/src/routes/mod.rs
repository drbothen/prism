//! Route modules for the CrowdStrike DTU.

pub mod detections;
pub mod hosts;
pub mod oauth;
pub mod writes;

use std::sync::Arc;

use axum::{
    extract::{State as AxumState, State},
    http::{HeaderMap, Request, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Json, Response},
    routing::{get, patch, post},
    Router,
};
use prism_dtu_common::{FailureMode, LatencyLayer};
use subtle::ConstantTimeEq;

use crate::state::CrowdstrikeState;

/// `GET /dtu/health` — DTU introspection endpoint. No auth required.
///
/// Returns HTTP 200 with `{"status": "ok"}`. Used by `FidelityValidator` as a
/// no-auth probe per ADR-003 §Decision Conflict #2 Option C.
async fn dtu_health() -> impl IntoResponse {
    (StatusCode::OK, Json(serde_json::json!({"status": "ok"}))).into_response()
}

/// `GET /dtu/filter-log` — DTU introspection endpoint. No auth required.
///
/// Returns all FQL filter strings received by `GET /detects/queries/detects/v1`
/// since the last reset. Format: `{"filter_strings": ["..."]}`.
///
/// Used by wire-level tests (F-P1-HIGH-003 / AC-CWS-002) to assert the DTU
/// received the correct FQL filter string from the production push-down path.
/// Parallel to the Armis `GET /dtu/aql-log` endpoint (R-DTU-002 pattern).
async fn dtu_filter_log(State(state): State<Arc<CrowdstrikeState>>) -> impl IntoResponse {
    let filter_strings = state.get_filter_log();
    (
        StatusCode::OK,
        Json(serde_json::json!({ "filter_strings": filter_strings })),
    )
        .into_response()
}

/// `POST /dtu/reset` — DTU introspection endpoint.
///
/// Clears all mutable state (containment store, detection status store, session
/// registry) and returns HTTP 200 with `{"status": "ok"}`.
///
/// # ADR-003 Amendment #5 (W3-FIX-SEC-002)
///
/// Requires `X-Admin-Token` header matching `state.admin_token`. Returns 401 with
/// `{"error": "missing or invalid admin token"}` if the header is absent or wrong.
async fn dtu_reset(
    State(state): State<Arc<CrowdstrikeState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let provided = headers.get("x-admin-token").and_then(|v| v.to_str().ok());
    // SEC-P3-003: constant-time comparison to prevent timing oracle attacks (CWE-208).
    let provided_bytes = provided.unwrap_or("").as_bytes();
    let expected_bytes = state.admin_token.as_bytes();
    let valid: bool = provided_bytes.ct_eq(expected_bytes).into();
    if !valid {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"error": "missing or invalid admin token"})),
        )
            .into_response();
    }
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
    let provided = headers.get("x-admin-token").and_then(|v| v.to_str().ok());
    // SEC-P3-003: constant-time comparison to prevent timing oracle attacks (CWE-208).
    let provided_bytes = provided.unwrap_or("").as_bytes();
    let expected_bytes = state.admin_token.as_bytes();
    let valid: bool = provided_bytes.ct_eq(expected_bytes).into();
    if !valid {
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
            // SAFETY: the builder args are all constants; failure is impossible at runtime.
            #[allow(clippy::expect_used)]
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
/// Wires all 9 in-scope endpoints (5 read, 4 write) plus the OAuth token endpoint.
///
/// Counting method for write endpoints: writes are counted as SEMANTIC OPERATIONS
/// (4 total), not routes (2 total). Two write routes dispatch to two operations each:
///   - `POST /devices/entities/devices-actions/v2` → `contain` + `lift_containment` (2 ops)
///   - `PATCH /detects/entities/detects/v2` → `assign` + `update_status` (2 ops)
///
/// Total: 4 semantic write operations, 2 write routes.
///
/// Wraps with `LatencyLayer` (from prism-dtu-common) and a custom axum middleware
/// for `FailureMode` injection that uses the shared counter in `CrowdstrikeState`.
///
/// The 5th read endpoint is `POST /devices/entities/devices/v2` (`post_host_details`),
/// added by DEFECT-CSDEVICES-EMPTY-PIPELINE-001 (D-1650 ratification §Contract Part 2)
/// to satisfy the updated `crowdstrike.sensor.toml` `fetch_devices` step (POST + body_template).
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
        .route("/dtu/filter-log", get(dtu_filter_log))
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
        .route(
            "/devices/entities/devices/v2",
            get(hosts::get_host_details).post(hosts::post_host_details),
        )
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
