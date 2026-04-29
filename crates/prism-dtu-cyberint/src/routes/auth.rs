//! Auth route handler for the Cyberint DTU clone.
//!
//! Implements the CookieRoundtrip auth pattern:
//! - `POST /login` — accepts any body; issues a UUID session token via Set-Cookie.

use std::sync::Arc;

use axum::{
    extract::State,
    http::{header, HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use uuid::Uuid;

use crate::routes::alerts::extract_org_id;
use crate::state::CyberintState;

/// `POST /login`
///
/// Accepts any JSON body. Generates a new UUID session token, registers it in the
/// session store scoped to the request's `OrgId`, and returns it as a `Set-Cookie` header.
///
/// Response: 200 `{"message": "Login successful"}`
///
/// # S-3.2.04 stub
///
/// `register_session` now requires `(OrgId, token)` per BC-3.2.003.  OrgId is
/// extracted via `extract_org_id` (stub: `todo!()` pending request-context wiring
/// in the implementation phase per ADR-008 §2.1).
pub async fn post_login(
    State(state): State<Arc<CyberintState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let token = Uuid::new_v4().to_string();
    let org_id = extract_org_id(&headers, state.instance_org_id);
    state.register_session(org_id, token.clone());

    let cookie_value = format!("cyberint_session={token}; Path=/; HttpOnly");

    (
        StatusCode::OK,
        [(header::SET_COOKIE, cookie_value)],
        Json(serde_json::json!({"message": "Login successful"})),
    )
        .into_response()
}
