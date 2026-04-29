//! Issue creation and retrieval route handlers.
//!
//! Endpoints:
//! - `POST /rest/api/3/issue` — create issue (field validation + registry insert)
//! - `GET /rest/api/3/issue/{issueKey}` — get issue by key
//!
//! Auth: both endpoints require `Authorization: Basic {base64}` header.
//! Missing/invalid auth → HTTP 401 per Jira Cloud API spec (AC-8).
//!
//! Field validation:
//! - Missing `fields.project.key` → 400 `{"errorMessages": [], "errors": {"project": "required"}}`
//! - Unknown `issuetype.name` → 400 `{"errorMessages": [], "errors": {"issuetype": "unknown"}}`
//!
//! Issue key format: `"{project_key}-{next_issue_num}"` e.g. `"ACME-SEC-1000"`.

use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};

#[cfg(feature = "dtu")]
use prism_core::OrgId;

use crate::state::{IssueRecord, IssueStatus, JiraState};
use crate::types::{
    CommentField, CreateIssueRequest, CreateIssueResponse, IssueFields, IssueResponse,
    IssueStatusField, IssueTypeField, JiraError, ProjectField, VALID_ISSUE_TYPES,
};

/// Check that the request carries a valid `Authorization: Basic {base64}` header.
///
/// Returns `Some(response)` if auth is missing/invalid, `None` if valid.
/// The DTU accepts any valid Base64-encoded `user:token` string — credentials are
/// not validated. Only the header format matters (R-DTU-008 mitigation).
pub(crate) fn check_basic_auth(headers: &HeaderMap) -> Option<axum::response::Response> {
    use base64::Engine as _;

    let auth_value = match headers.get("authorization") {
        Some(v) => v,
        None => {
            return Some(
                (
                    StatusCode::UNAUTHORIZED,
                    Json(JiraError::messages(vec![
                        "Basic authentication required".to_owned()
                    ])),
                )
                    .into_response(),
            );
        }
    };

    let auth_str = match auth_value.to_str() {
        Ok(s) => s,
        Err(_) => {
            return Some(
                (
                    StatusCode::UNAUTHORIZED,
                    Json(JiraError::messages(vec![
                        "Basic authentication required".to_owned()
                    ])),
                )
                    .into_response(),
            );
        }
    };

    // Must start with "Basic " (case-sensitive per RFC 7617).
    let encoded = match auth_str.strip_prefix("Basic ") {
        Some(e) => e,
        None => {
            return Some(
                (
                    StatusCode::UNAUTHORIZED,
                    Json(JiraError::messages(vec![
                        "Basic authentication required".to_owned()
                    ])),
                )
                    .into_response(),
            );
        }
    };

    // Must be valid Base64.
    if base64::engine::general_purpose::STANDARD
        .decode(encoded)
        .is_err()
    {
        return Some(
            (
                StatusCode::UNAUTHORIZED,
                Json(JiraError::messages(vec![
                    "Basic authentication required".to_owned()
                ])),
            )
                .into_response(),
        );
    }

    None
}

/// Infer the originating `OrgId` from the `X-Prism-Org-Id` request header.
///
/// Resolution order (BC-3.2.004 invariant 1):
/// 1. `X-Prism-Org-Id` request header — present in test harness and production
///    multi-org routing. Value must be a valid UUID string.
/// 2. Fallback: generate a new `OrgId` (anonymous ingress — ensures every captured
///    issue record has an `org_id` field even when the caller does not supply one).
///
/// The resolved UUID is embedded in `IssueRecord.org_id` only — NEVER placed in
/// a URL path segment, URL query parameter, forwarded `X-` header, or `issue_key`.
#[cfg(feature = "dtu")]
fn resolve_org_id_from_headers(headers: &HeaderMap) -> OrgId {
    headers
        .get("X-Prism-Org-Id")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| uuid::Uuid::parse_str(s).ok())
        .map(OrgId::from_uuid)
        .unwrap_or_else(OrgId::new)
}

/// `POST /rest/api/3/issue` — create a new Jira issue.
pub async fn create_issue(
    State(state): State<Arc<JiraState>>,
    headers: HeaderMap,
    Json(body): Json<CreateIssueRequest>,
) -> impl IntoResponse {
    if let Some(err) = check_basic_auth(&headers) {
        return err;
    }

    // Validate required field: project.key
    let project_key = match body.fields.project {
        Some(ref p) => p.key.clone(),
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(JiraError::field_errors(vec![("project", "required")])),
            )
                .into_response();
        }
    };

    // Validate required field: issuetype.name
    let issue_type_name = match body.fields.issuetype {
        Some(ref it) => {
            if !VALID_ISSUE_TYPES.contains(&it.name.as_str()) {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(JiraError::field_errors(vec![("issuetype", "unknown")])),
                )
                    .into_response();
            }
            it.name.clone()
        }
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(JiraError::field_errors(vec![("issuetype", "required")])),
            )
                .into_response();
        }
    };

    // Validate required field: summary
    let summary = match body.fields.summary {
        Some(ref s) => s.clone(),
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(JiraError::field_errors(vec![("summary", "required")])),
            )
                .into_response();
        }
    };

    // Assign issue number and key.
    let issue_num = state.next_issue_num();
    let issue_id = format!("1{:04}", issue_num);
    let issue_key = format!("{project_key}-{issue_num}");
    let self_link = format!("http://localhost/rest/api/3/issue/{issue_key}");

    let record = IssueRecord {
        id: issue_id.clone(),
        key: issue_key.clone(),
        summary,
        issue_type: issue_type_name,
        project_key,
        status: IssueStatus::Open,
        comment_count: 0,
        // org_id is populated by capture_issue (shared-mode path, S-3.2.07 / BC-3.2.004).
        // The direct insert_issue path leaves org_id empty (non-shared fallback).
        org_id: String::new(),
        fields: serde_json::Value::Null,
    };

    // Shared-mode ingress tagging (BC-3.2.004 / S-3.2.07):
    // Resolve OrgId from X-Prism-Org-Id header and embed it in IssueRecord.org_id.
    // OrgId MUST appear only in IssueRecord.org_id — never in issue_key, URL path, or headers.
    #[cfg(feature = "dtu")]
    {
        let org_id = resolve_org_id_from_headers(&headers);
        state.capture_issue(org_id, issue_key.clone(), record);
    }
    #[cfg(not(feature = "dtu"))]
    {
        state.insert_issue(record);
    }

    (
        StatusCode::CREATED,
        Json(CreateIssueResponse {
            id: issue_id,
            key: issue_key,
            self_link,
        }),
    )
        .into_response()
}

/// `GET /rest/api/3/issue/{issueKey}` — get issue by key.
pub async fn get_issue(
    State(state): State<Arc<JiraState>>,
    headers: HeaderMap,
    Path(issue_key): Path<String>,
) -> impl IntoResponse {
    if let Some(err) = check_basic_auth(&headers) {
        return err;
    }

    match state.get_issue(&issue_key) {
        None => (
            StatusCode::NOT_FOUND,
            Json(JiraError::messages(vec!["Issue does not exist".to_owned()])),
        )
            .into_response(),
        Some(issue) => {
            let self_link = format!("http://localhost/rest/api/3/issue/{issue_key}");
            let response = IssueResponse {
                id: issue.id.clone(),
                key: issue.key.clone(),
                self_link,
                fields: IssueFields {
                    summary: issue.summary.clone(),
                    status: IssueStatusField {
                        name: issue.status.display_name().to_owned(),
                        id: issue.status.status_id().to_owned(),
                    },
                    issuetype: IssueTypeField {
                        name: issue.issue_type.clone(),
                    },
                    comment: CommentField {
                        total: issue.comment_count,
                    },
                    project: ProjectField {
                        key: issue.project_key.clone(),
                    },
                },
            };
            (StatusCode::OK, Json(response)).into_response()
        }
    }
}
