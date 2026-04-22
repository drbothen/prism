//! OAuth2 token endpoint for the CrowdStrike DTU.
//!
//! `POST /oauth2/token` — simulates CrowdStrike's `client_credentials` grant.
//! Returns a static fake token unless `auth_mode == "reject"` is configured.
//!
//! # Spec decision: fidelity auth bypass
//!
//! `FidelityValidator` probes do not send an `Authorization` header.
//! The token endpoint itself is public (no auth required to call it), but when
//! `auth_mode="reject"` is active it returns 401 to simulate credential rejection.

use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Json};

use crate::state::CrowdstrikeState;

/// `POST /oauth2/token`
///
/// Accepts `client_credentials` grant; returns a static fake access token.
/// Returns HTTP 401 when `auth_mode == "reject"` is configured.
pub async fn token(
    State(state): State<Arc<CrowdstrikeState>>,
) -> impl IntoResponse {
    if state.is_auth_reject() {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({
                "errors": [{"code": 401, "message": "invalid_client"}]
            })),
        )
            .into_response();
    }

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "access_token": "dtu-fake-cs-token",
            "token_type": "bearer",
            "expires_in": 3600
        })),
    )
        .into_response()
}
