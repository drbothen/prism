//! OAuth2 token endpoint for the CrowdStrike DTU.
//!
//! `POST /oauth2/token` — simulates CrowdStrike's `client_credentials` grant.
//! Returns a static fake token unless `auth_mode == "reject"` is configured.

use axum::http::StatusCode;

/// `POST /oauth2/token`
///
/// Accepts `client_credentials` grant; returns a static fake access token.
/// Returns HTTP 401 when `auth_mode == "reject"` is configured on the state.
pub async fn token() -> StatusCode {
    unimplemented!("oauth::token — not yet implemented")
}
