//! Jira Cloud REST API v3 clone router for the DTU harness (S-3.4.05).
//!
//! Provides Jira-specific route handlers running inside the harness
//! for `DtuType::Jira` clones. Uses `CloneState` for failure mode injection.
//!
//! # Routes served
//!
//! - `POST /rest/api/3/issue`                       — create issue
//! - `GET  /rest/api/3/issue/{key}`                 — get issue
//! - `POST /rest/api/3/issue/{key}/comment`         — add comment
//! - `GET  /rest/api/3/issue/{key}/transitions`     — list transitions
//! - `POST /rest/api/3/issue/{key}/transitions`     — execute transition
//! - `GET  /dtu/issues`                             — test API: issue registry
//! - `POST /dtu/reset`                              — clear all state
//! - `GET  /dtu/health`                             — liveness check
//! - `POST /dtu/configure`                          — harness failure injection
//!
//! # OrgId tagging (BC-3.2.004)
//!
//! Every created issue stores the originating OrgId UUID in `IssueRecord.org_id`,
//! resolved from the `X-Prism-Org-Id` header.
//!
//! # Status machine
//!
//! Open → InProgress (id "11" — "Start Progress")
//! Open → Done       (id "31" — "Close")
//! InProgress → Done (id "21" — "Done")
//!
//! # BC anchors
//!
//! - BC-3.2.004 — shared-mode org-id tagging
//! - BC-3.5.001 — harness logical isolation

use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicU32, Ordering},
    Arc, Mutex,
};

use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use prism_dtu_common::FailureMode;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::clone_server::CloneState;

// ---------------------------------------------------------------------------
// Issue state types (mirrors prism-dtu-jira, self-contained)
// ---------------------------------------------------------------------------

/// Issue status state machine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IssueStatus {
    Open,
    InProgress,
    Done,
}

impl IssueStatus {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Open => "Open",
            Self::InProgress => "In Progress",
            Self::Done => "Done",
        }
    }

    pub fn status_id(&self) -> &'static str {
        match self {
            Self::Open => "1",
            Self::InProgress => "3",
            Self::Done => "6",
        }
    }
}

/// A single issue record in the Jira harness registry.
#[derive(Debug, Clone)]
pub struct IssueRecord {
    pub id: String,
    pub key: String,
    pub summary: String,
    pub issue_type: String,
    pub project_key: String,
    pub status: IssueStatus,
    pub comment_count: u32,
    /// OrgId UUID string (BC-3.2.004).
    pub org_id: String,
}

/// Accepted issue type names.
const VALID_ISSUE_TYPES: &[&str] = &["Task", "Bug", "Story", "Epic", "Incident"];

// ---------------------------------------------------------------------------
// Jira-specific state
// ---------------------------------------------------------------------------

/// In-memory state for harness-hosted Jira clone.
pub struct JiraHarnessState {
    /// Issue registry keyed by issue key (e.g. "ACME-SEC-1000").
    pub issue_registry: Mutex<HashMap<String, IssueRecord>>,
    /// Monotonically increasing issue number; resets to 1000 on reset.
    pub next_issue_num: AtomicU32,
}

impl JiraHarnessState {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            issue_registry: Mutex::new(HashMap::new()),
            next_issue_num: AtomicU32::new(1000),
        })
    }

    pub fn next_num(&self) -> u32 {
        self.next_issue_num.fetch_add(1, Ordering::SeqCst)
    }

    #[allow(clippy::expect_used)]
    pub fn insert(&self, record: IssueRecord) {
        self.issue_registry
            .lock()
            .expect("issue_registry poisoned")
            .insert(record.key.clone(), record);
    }

    #[allow(clippy::expect_used)]
    pub fn get(&self, key: &str) -> Option<IssueRecord> {
        self.issue_registry
            .lock()
            .expect("issue_registry poisoned")
            .get(key)
            .cloned()
    }

    #[allow(clippy::expect_used)]
    pub fn increment_comment_count(&self, key: &str) -> bool {
        let mut registry = self.issue_registry.lock().expect("issue_registry poisoned");
        if let Some(issue) = registry.get_mut(key) {
            issue.comment_count += 1;
            true
        } else {
            false
        }
    }

    #[allow(clippy::expect_used)]
    pub fn apply_transition(&self, key: &str, transition_id: &str) -> bool {
        let mut registry = self.issue_registry.lock().expect("issue_registry poisoned");
        let Some(issue) = registry.get_mut(key) else {
            return false;
        };
        let new_status = match (&issue.status, transition_id) {
            (IssueStatus::Open, "11") => IssueStatus::InProgress,
            (IssueStatus::Open, "31") => IssueStatus::Done,
            (IssueStatus::InProgress, "21") => IssueStatus::Done,
            _ => return false,
        };
        issue.status = new_status;
        true
    }

    #[allow(clippy::expect_used)]
    pub(crate) fn available_transitions(&self, key: &str) -> Option<Vec<TransitionEntry>> {
        let registry = self.issue_registry.lock().expect("issue_registry poisoned");
        let issue = registry.get(key)?;
        Some(match issue.status {
            IssueStatus::Open => vec![
                TransitionEntry {
                    id: "11".to_owned(),
                    name: "Start Progress".to_owned(),
                },
                TransitionEntry {
                    id: "31".to_owned(),
                    name: "Close".to_owned(),
                },
            ],
            IssueStatus::InProgress => vec![TransitionEntry {
                id: "21".to_owned(),
                name: "Done".to_owned(),
            }],
            IssueStatus::Done => vec![],
        })
    }

    #[allow(clippy::expect_used)]
    pub fn all_issues(&self) -> Vec<IssueRecord> {
        self.issue_registry
            .lock()
            .expect("issue_registry poisoned")
            .values()
            .cloned()
            .collect()
    }

    #[allow(clippy::expect_used)]
    pub fn reset(&self) {
        self.issue_registry
            .lock()
            .expect("issue_registry poisoned")
            .clear();
        self.next_issue_num.store(1000, Ordering::SeqCst);
    }
}

// ---------------------------------------------------------------------------
// Combined state
// ---------------------------------------------------------------------------

/// Combined state for harness-hosted Jira clone.
pub struct JiraCloneCtx {
    pub clone_state: Arc<CloneState>,
    pub jira_state: Arc<JiraHarnessState>,
}

// ---------------------------------------------------------------------------
// Request/response types (self-contained, mirrors prism-dtu-jira/types.rs)
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct CreateIssueRequest {
    fields: CreateIssueFields,
}

#[derive(Debug, Deserialize)]
struct CreateIssueFields {
    project: Option<ProjectRef>,
    issuetype: Option<IssueTypeRef>,
    summary: Option<String>,
    #[serde(flatten)]
    #[allow(dead_code)]
    extra: HashMap<String, Value>,
}

#[derive(Debug, Deserialize)]
struct ProjectRef {
    key: String,
}

#[derive(Debug, Deserialize)]
struct IssueTypeRef {
    name: String,
}

#[derive(Debug, Serialize)]
struct CreateIssueResponse {
    id: String,
    key: String,
    #[serde(rename = "self")]
    self_link: String,
}

#[derive(Debug, Serialize)]
struct IssueResponse {
    id: String,
    key: String,
    #[serde(rename = "self")]
    self_link: String,
    fields: IssueFields,
}

#[derive(Debug, Serialize)]
struct IssueFields {
    summary: String,
    status: IssueStatusField,
    issuetype: IssueTypeField,
    comment: CommentField,
    project: ProjectField,
}

#[derive(Debug, Serialize)]
struct IssueStatusField {
    name: String,
    id: String,
}

#[derive(Debug, Serialize)]
struct IssueTypeField {
    name: String,
}

#[derive(Debug, Serialize)]
struct CommentField {
    total: u32,
}

#[derive(Debug, Serialize)]
struct ProjectField {
    key: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct AddCommentRequest {
    body: Option<Value>,
}

#[derive(Debug, Serialize)]
struct AddCommentResponse {
    id: String,
    #[serde(rename = "self")]
    self_link: String,
    created: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct TransitionEntry {
    id: String,
    name: String,
}

#[derive(Debug, Serialize)]
struct TransitionsResponse {
    transitions: Vec<TransitionEntry>,
}

#[derive(Debug, Deserialize)]
struct ExecuteTransitionRequest {
    transition: TransitionRef,
}

#[derive(Debug, Deserialize)]
struct TransitionRef {
    id: String,
}

#[derive(Debug, Serialize)]
struct JiraError {
    #[serde(rename = "errorMessages")]
    error_messages: Vec<String>,
    errors: serde_json::Map<String, Value>,
}

impl JiraError {
    fn messages(msgs: Vec<String>) -> Self {
        Self {
            error_messages: msgs,
            errors: serde_json::Map::new(),
        }
    }

    fn field_errors(fields: Vec<(&str, &str)>) -> Self {
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
// Basic auth check (mirrors prism-dtu-jira)
// ---------------------------------------------------------------------------

fn check_basic_auth(headers: &HeaderMap) -> Option<axum::response::Response> {
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

// ---------------------------------------------------------------------------
// Helper: resolve OrgId
// ---------------------------------------------------------------------------

fn resolve_org_id(headers: &HeaderMap) -> String {
    headers
        .get("X-Prism-Org-Id")
        .and_then(|v| v.to_str().ok())
        .filter(|s| uuid::Uuid::parse_str(s).is_ok())
        .map(|s| s.to_owned())
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string())
}

// ---------------------------------------------------------------------------
// Route handlers
// ---------------------------------------------------------------------------

/// `POST /rest/api/3/issue` — create a Jira issue.
///
/// Applies rate-limit failure mode from CloneState before processing.
async fn create_issue(
    State(ctx): State<Arc<JiraCloneCtx>>,
    headers: HeaderMap,
    Json(body): Json<CreateIssueRequest>,
) -> impl IntoResponse {
    if let Some(err) = check_basic_auth(&headers) {
        return err;
    }

    // Apply failure mode: rate-limit returns 429 without creating.
    let count = ctx.clone_state.increment_request();
    let mode = ctx.clone_state.current_failure_mode();
    match &mode {
        FailureMode::RateLimit {
            after_n_requests,
            retry_after_secs,
        } if count > *after_n_requests => {
            let retry_str = retry_after_secs.to_string();
            let mut resp = (
                StatusCode::TOO_MANY_REQUESTS,
                Json(json!({"error": "rate limited"})),
            )
                .into_response();
            #[allow(clippy::expect_used)]
            resp.headers_mut().insert(
                "retry-after",
                retry_str
                    .parse()
                    .expect("retry_after_secs is a valid header value"),
            );
            return resp;
        }
        _ => {}
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

    // Allocate issue number and key.
    let issue_num = ctx.jira_state.next_num();
    let issue_id = format!("1{:04}", issue_num);
    let issue_key = format!("{project_key}-{issue_num}");
    let self_link = format!("http://localhost/rest/api/3/issue/{issue_key}");

    // Resolve OrgId for shared-mode tagging (BC-3.2.004).
    let org_id = resolve_org_id(&headers);

    let record = IssueRecord {
        id: issue_id.clone(),
        key: issue_key.clone(),
        summary,
        issue_type: issue_type_name,
        project_key,
        status: IssueStatus::Open,
        comment_count: 0,
        org_id,
    };
    ctx.jira_state.insert(record);

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
async fn get_issue(
    State(ctx): State<Arc<JiraCloneCtx>>,
    headers: HeaderMap,
    Path(issue_key): Path<String>,
) -> impl IntoResponse {
    if let Some(err) = check_basic_auth(&headers) {
        return err;
    }

    match ctx.jira_state.get(&issue_key) {
        None => (
            StatusCode::NOT_FOUND,
            Json(JiraError::messages(vec!["Issue does not exist".to_owned()])),
        )
            .into_response(),
        Some(issue) => {
            let self_link = format!("http://localhost/rest/api/3/issue/{issue_key}");
            (
                StatusCode::OK,
                Json(IssueResponse {
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
                }),
            )
                .into_response()
        }
    }
}

/// `POST /rest/api/3/issue/{issueKey}/comment` — add a comment to an issue.
async fn add_comment(
    State(ctx): State<Arc<JiraCloneCtx>>,
    headers: HeaderMap,
    Path(issue_key): Path<String>,
    Json(_body): Json<AddCommentRequest>,
) -> impl IntoResponse {
    if let Some(err) = check_basic_auth(&headers) {
        return err;
    }

    if !ctx.jira_state.increment_comment_count(&issue_key) {
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

    (
        StatusCode::CREATED,
        Json(AddCommentResponse {
            id: comment_id,
            self_link,
            created: "2026-04-16T00:00:00.000+0000".to_owned(),
        }),
    )
        .into_response()
}

/// `GET /rest/api/3/issue/{issueKey}/transitions` — list available transitions.
async fn list_transitions(
    State(ctx): State<Arc<JiraCloneCtx>>,
    headers: HeaderMap,
    Path(issue_key): Path<String>,
) -> impl IntoResponse {
    if let Some(err) = check_basic_auth(&headers) {
        return err;
    }

    match ctx.jira_state.available_transitions(&issue_key) {
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
async fn execute_transition(
    State(ctx): State<Arc<JiraCloneCtx>>,
    headers: HeaderMap,
    Path(issue_key): Path<String>,
    Json(body): Json<ExecuteTransitionRequest>,
) -> impl IntoResponse {
    if let Some(err) = check_basic_auth(&headers) {
        return err;
    }

    if ctx.jira_state.get(&issue_key).is_none() {
        return (
            StatusCode::NOT_FOUND,
            Json(JiraError::messages(vec!["Issue does not exist".to_owned()])),
        )
            .into_response();
    }

    if ctx
        .jira_state
        .apply_transition(&issue_key, &body.transition.id)
    {
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

/// `GET /dtu/issues` — return all issues in the registry.
async fn get_dtu_issues(State(ctx): State<Arc<JiraCloneCtx>>) -> impl IntoResponse {
    let issues: Vec<Value> = ctx
        .jira_state
        .all_issues()
        .into_iter()
        .map(|i| {
            json!({
                "key": i.key,
                "status": i.status.display_name(),
                "summary": i.summary,
                "comment_count": i.comment_count,
                "org_id": i.org_id,
            })
        })
        .collect();
    (StatusCode::OK, Json(json!({"issues": issues}))).into_response()
}

/// `POST /dtu/reset` — clear all state.
async fn post_reset(State(ctx): State<Arc<JiraCloneCtx>>) -> impl IntoResponse {
    ctx.jira_state.reset();
    ctx.clone_state.request_count.store(0, Ordering::SeqCst);
    ctx.clone_state.set_failure_mode(FailureMode::None);
    (StatusCode::OK, Json(json!({"status": "ok"}))).into_response()
}

/// `GET /dtu/health` — liveness check.
async fn get_health() -> impl IntoResponse {
    (StatusCode::OK, Json(json!({"status": "ok"}))).into_response()
}

// ---------------------------------------------------------------------------
// Configure handler (harness format)
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, Default)]
struct HarnessConfigure {
    #[serde(default)]
    auth_mode: Option<String>,
    #[serde(default)]
    rate_limit_after: Option<u32>,
    #[serde(default)]
    retry_after_secs: Option<u32>,
    #[serde(default)]
    internal_error_at: Option<u32>,
    #[serde(default)]
    network_timeout_ms: Option<u64>,
    #[serde(default)]
    malformed_response: Option<bool>,
    #[serde(default)]
    unprocessable_at: Option<u32>,
    #[serde(default)]
    clear: Option<bool>,
}

/// `POST /dtu/configure` — harness configure endpoint.
async fn post_configure(
    State(ctx): State<Arc<JiraCloneCtx>>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> (StatusCode, Json<Value>) {
    let provided = headers.get("x-admin-token").and_then(|v| v.to_str().ok());
    if provided != Some(ctx.clone_state.admin_token.as_str()) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "missing or invalid X-Admin-Token"})),
        );
    }

    let cfg: HarnessConfigure = match serde_json::from_value(body) {
        Ok(c) => c,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({"error": format!("invalid configure payload: {e}")})),
            );
        }
    };

    let mode = harness_configure_to_failure_mode(&cfg);
    ctx.clone_state.request_count.store(0, Ordering::SeqCst);
    ctx.clone_state.set_failure_mode(mode);
    (StatusCode::OK, Json(json!({"status": "ok"})))
}

fn harness_configure_to_failure_mode(cfg: &HarnessConfigure) -> FailureMode {
    if cfg.clear == Some(true) {
        return FailureMode::None;
    }
    if cfg.auth_mode.as_deref() == Some("reject") {
        return FailureMode::AuthReject;
    }
    if let Some(n) = cfg.rate_limit_after {
        return FailureMode::RateLimit {
            after_n_requests: n,
            retry_after_secs: cfg.retry_after_secs.unwrap_or(60),
        };
    }
    if let Some(n) = cfg.internal_error_at {
        return FailureMode::InternalError { at_request_n: n };
    }
    if let Some(ms) = cfg.network_timeout_ms {
        return FailureMode::NetworkTimeout { after_ms: ms };
    }
    if cfg.malformed_response == Some(true) {
        return FailureMode::MalformedResponse;
    }
    if let Some(n) = cfg.unprocessable_at {
        return FailureMode::Unprocessable { at_request_n: n };
    }
    FailureMode::None
}

// ---------------------------------------------------------------------------
// Router construction
// ---------------------------------------------------------------------------

/// Build the axum router for a harness-hosted Jira clone.
pub fn build_jira_router(
    clone_state: Arc<CloneState>,
    jira_state: Arc<JiraHarnessState>,
) -> Router {
    let ctx = Arc::new(JiraCloneCtx {
        clone_state,
        jira_state,
    });

    Router::new()
        // Jira Cloud REST API v3
        .route("/rest/api/3/issue", post(create_issue))
        .route("/rest/api/3/issue/:key", get(get_issue))
        .route("/rest/api/3/issue/:key/comment", post(add_comment))
        .route(
            "/rest/api/3/issue/:key/transitions",
            get(list_transitions).post(execute_transition),
        )
        // DTU test API
        .route("/dtu/issues", get(get_dtu_issues))
        .route("/dtu/reset", post(post_reset))
        .route("/dtu/health", get(get_health))
        // Harness configure
        .route("/dtu/configure", post(post_configure))
        .with_state(ctx)
}
