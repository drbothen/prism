//! Jira Cloud REST API v3 request/response types.
//!
//! Structs mirror the Jira Cloud REST API v3 JSON schema so that Prism's
//! Jira action delivery adapters can deserialize DTU responses identically
//! to live Jira Cloud responses.

use serde::{Deserialize, Serialize};
use serde_json::Value;

// ---------------------------------------------------------------------------
// Issue creation types
// ---------------------------------------------------------------------------

/// POST /rest/api/3/issue — create issue request body (top-level).
#[derive(Debug, Deserialize)]
pub struct CreateIssueRequest {
    pub fields: CreateIssueFields,
}

/// `fields` sub-object for create issue.
#[derive(Debug, Deserialize)]
pub struct CreateIssueFields {
    pub project: Option<ProjectRef>,
    pub issuetype: Option<IssueTypeRef>,
    pub summary: Option<String>,
    /// Any extra fields are captured here and ignored (EC-001: permissive).
    #[serde(flatten)]
    pub extra: std::collections::HashMap<String, Value>,
}

/// `fields.project` sub-object.
#[derive(Debug, Deserialize)]
pub struct ProjectRef {
    pub key: String,
}

/// `fields.issuetype` sub-object.
#[derive(Debug, Deserialize)]
pub struct IssueTypeRef {
    pub name: String,
}

/// Response for successful issue creation (201).
#[derive(Debug, Serialize)]
pub struct CreateIssueResponse {
    pub id: String,
    pub key: String,
    #[serde(rename = "self")]
    pub self_link: String,
}

// ---------------------------------------------------------------------------
// Issue get types
// ---------------------------------------------------------------------------

/// Full issue response for GET /rest/api/3/issue/{key}.
#[derive(Debug, Serialize)]
pub struct IssueResponse {
    pub id: String,
    pub key: String,
    #[serde(rename = "self")]
    pub self_link: String,
    pub fields: IssueFields,
}

/// `fields` sub-object returned in GET /rest/api/3/issue/{key}.
#[derive(Debug, Serialize)]
pub struct IssueFields {
    pub summary: String,
    pub status: IssueStatusField,
    pub issuetype: IssueTypeField,
    pub comment: CommentField,
    pub project: ProjectField,
}

#[derive(Debug, Serialize)]
pub struct IssueStatusField {
    pub name: String,
    pub id: String,
}

#[derive(Debug, Serialize)]
pub struct IssueTypeField {
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct CommentField {
    pub total: u32,
}

#[derive(Debug, Serialize)]
pub struct ProjectField {
    pub key: String,
}

// ---------------------------------------------------------------------------
// Comment types
// ---------------------------------------------------------------------------

/// POST /rest/api/3/issue/{key}/comment — request body.
#[derive(Debug, Deserialize)]
pub struct AddCommentRequest {
    /// Body content accepted as-is (not validated).
    pub body: Option<Value>,
}

/// Response for successful comment creation (201).
#[derive(Debug, Serialize)]
pub struct AddCommentResponse {
    pub id: String,
    #[serde(rename = "self")]
    pub self_link: String,
    pub created: String,
}

// ---------------------------------------------------------------------------
// Transitions types
// ---------------------------------------------------------------------------

/// A single transition entry in the transitions list.
#[derive(Debug, Serialize, Clone)]
pub struct TransitionEntry {
    pub id: String,
    pub name: String,
}

/// Response for GET /rest/api/3/issue/{key}/transitions.
#[derive(Debug, Serialize)]
pub struct TransitionsResponse {
    pub transitions: Vec<TransitionEntry>,
}

/// POST /rest/api/3/issue/{key}/transitions — request body.
#[derive(Debug, Deserialize)]
pub struct ExecuteTransitionRequest {
    pub transition: TransitionRef,
}

#[derive(Debug, Deserialize)]
pub struct TransitionRef {
    pub id: String,
}

// ---------------------------------------------------------------------------
// Error response types
// ---------------------------------------------------------------------------

/// Standard Jira error response body.
#[derive(Debug, Serialize)]
pub struct JiraError {
    #[serde(rename = "errorMessages")]
    pub error_messages: Vec<String>,
    pub errors: serde_json::Map<String, Value>,
}

impl JiraError {
    /// Construct a simple error with a single error message.
    pub fn messages(msgs: Vec<String>) -> Self {
        Self {
            error_messages: msgs,
            errors: serde_json::Map::new(),
        }
    }

    /// Construct an error with field-level errors (no top-level messages).
    pub fn field_errors(fields: Vec<(&str, &str)>) -> Self {
        let mut map = serde_json::Map::new();
        for (field, msg) in fields {
            map.insert(field.to_owned(), Value::String(msg.to_owned()));
        }
        Self {
            error_messages: Vec::new(),
            errors: map,
        }
    }
}

// ---------------------------------------------------------------------------
// DTU introspection types
// ---------------------------------------------------------------------------

/// Response for GET /dtu/issues — test API introspection.
#[derive(Debug, Serialize)]
pub struct DtuIssuesResponse {
    pub issues: Vec<DtuIssueSummary>,
}

/// Minimal issue summary for test assertion via GET /dtu/issues.
#[derive(Debug, Serialize)]
pub struct DtuIssueSummary {
    pub key: String,
    pub status: String,
    pub summary: String,
    pub comment_count: u32,
}

/// Known issue type names accepted by the DTU.
pub const VALID_ISSUE_TYPES: &[&str] = &["Task", "Bug", "Story", "Epic", "Incident"];
