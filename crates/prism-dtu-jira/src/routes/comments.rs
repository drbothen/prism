//! Add comment route handler.
//!
//! Endpoint:
//! - `POST /rest/api/3/issue/{issueKey}/comment` — add comment to an issue
//!
//! Auth: requires `Authorization: Basic {base64}` header.
//! Missing/invalid auth → HTTP 401.
//!
//! Behavior:
//! - 404 if issue not found
//! - Body `{"body": {...}}` accepted as-is (not validated, per story spec)
//! - Increments `comment_count` on the issue record
//! - Returns 201 with comment id, self link, and created timestamp
//!
//! Edge case EC-004: Jira permits comments on closed (Done) issues — no status check.

use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};

use crate::routes::issues::check_basic_auth;
use crate::state::JiraState;
use crate::types::{AddCommentRequest, AddCommentResponse, JiraError};

/// `POST /rest/api/3/issue/{issueKey}/comment` — add a comment to an issue.
pub async fn add_comment(
    State(state): State<Arc<JiraState>>,
    headers: HeaderMap,
    Path(issue_key): Path<String>,
    Json(_body): Json<AddCommentRequest>,
) -> impl IntoResponse {
    if let Some(err) = check_basic_auth(&headers) {
        return err;
    }

    if !state.increment_comment_count(&issue_key) {
        return (
            StatusCode::NOT_FOUND,
            Json(JiraError::messages(vec!["Issue does not exist".to_owned()])),
        )
            .into_response();
    }

    let comment_id = uuid::Uuid::new_v4()
        .to_string()
        .replace('-', "")
        .chars()
        .take(8)
        .collect::<String>();
    let self_link = format!("http://localhost/rest/api/3/issue/{issue_key}/comment/{comment_id}");
    let created = chrono_now_iso8601();

    (
        StatusCode::CREATED,
        Json(AddCommentResponse {
            id: comment_id,
            self_link,
            created,
        }),
    )
        .into_response()
}

/// Return a fixed ISO-8601 timestamp string for deterministic test output.
///
/// In a real implementation this would use `chrono` or `time`. For stub
/// determinism, we return a hardcoded placeholder — tests assert only that the
/// field is present, not its specific value.
fn chrono_now_iso8601() -> String {
    // Stub timestamp — deterministic for tests.
    "2026-04-16T00:00:00.000+0000".to_owned()
}
