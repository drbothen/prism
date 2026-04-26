//! List and execute transitions route handlers.
//!
//! Endpoints:
//! - `GET /rest/api/3/issue/{issueKey}/transitions` — list available transitions
//! - `POST /rest/api/3/issue/{issueKey}/transitions` — execute a transition
//!
//! Auth: both endpoints require `Authorization: Basic {base64}` header.
//!
//! Status machine (L3 behavioral fidelity core):
//! - Open → InProgress (transition id "11" — "Start Progress")
//! - Open → Done       (transition id "31" — "Close")
//! - InProgress → Done (transition id "21" — "Done")
//! - All other transition attempts → 400 `{"errorMessages": ["Invalid transition id"], "errors": {}}`
//!
//! Edge case EC-002: GET transitions when status is Done returns `{"transitions": []}`.

use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};

use crate::routes::issues::check_basic_auth;
use crate::state::JiraState;
use crate::types::{ExecuteTransitionRequest, JiraError, TransitionsResponse};

/// `GET /rest/api/3/issue/{issueKey}/transitions` — list available transitions.
pub async fn list_transitions(
    State(state): State<Arc<JiraState>>,
    headers: HeaderMap,
    Path(issue_key): Path<String>,
) -> impl IntoResponse {
    if let Some(err) = check_basic_auth(&headers) {
        return err;
    }

    match state.available_transitions(&issue_key) {
        None => (
            StatusCode::NOT_FOUND,
            Json(JiraError::messages(vec!["Issue does not exist".to_owned()])),
        )
            .into_response(),
        Some(transitions) => {
            (StatusCode::OK, Json(TransitionsResponse { transitions })).into_response()
        }
    }
}

/// `POST /rest/api/3/issue/{issueKey}/transitions` — execute a transition.
pub async fn execute_transition(
    State(state): State<Arc<JiraState>>,
    headers: HeaderMap,
    Path(issue_key): Path<String>,
    Json(body): Json<ExecuteTransitionRequest>,
) -> impl IntoResponse {
    if let Some(err) = check_basic_auth(&headers) {
        return err;
    }

    // Check issue exists first.
    if state.get_issue(&issue_key).is_none() {
        return (
            StatusCode::NOT_FOUND,
            Json(JiraError::messages(vec!["Issue does not exist".to_owned()])),
        )
            .into_response();
    }

    if state.apply_transition(&issue_key, &body.transition.id) {
        StatusCode::NO_CONTENT.into_response()
    } else {
        (
            StatusCode::BAD_REQUEST,
            Json(JiraError::messages(
                vec!["Invalid transition id".to_owned()],
            )),
        )
            .into_response()
    }
}
