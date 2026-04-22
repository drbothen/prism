//! Auth route handler for the Cyberint DTU clone.
//!
//! Implements the CookieRoundtrip auth pattern:
//! - `POST /login` — accepts any body; issues a UUID session token via Set-Cookie.

use std::sync::Arc;

use axum::{
    extract::State,
    http::{header, StatusCode},
    response::IntoResponse,
    Json,
};
use uuid::Uuid;

use crate::state::CyberintState;

/// `POST /login`
///
/// Accepts any JSON body. Generates a new UUID session token, registers it in the
/// session store, and returns it as a `Set-Cookie` header.
///
/// Response: 200 `{"message": "Login successful"}`
pub async fn post_login(
    State(state): State<Arc<CyberintState>>,
) -> impl IntoResponse {
    let token = Uuid::new_v4().to_string();
    state.register_session(token.clone());

    let cookie_value = format!("cyberint_session={token}; Path=/; HttpOnly");

    (
        StatusCode::OK,
        [(header::SET_COOKIE, cookie_value)],
        Json(serde_json::json!({"message": "Login successful"})),
    )
        .into_response()
}
