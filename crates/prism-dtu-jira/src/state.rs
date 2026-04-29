//! `JiraState` — in-memory state for the Jira Cloud REST API v3 DTU behavioral clone.
//!
//! Maintains:
//! - `issue_registry`: `issueKey → IssueRecord` — stateful issue store
//! - `next_issue_num`: monotonically increasing issue number (starts at 1000, resets to 1000)
//! - Status machine: Open → InProgress → Done (invalid transitions return 400)
//!
//! No HTTP-layer types (`axum::Json`, `axum::extract::*`) appear here.
//! `JiraState` is pure Rust — no Axum dependency for its public methods.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};

#[cfg(feature = "dtu")]
use prism_core::OrgId;
use prism_dtu_common::FailureMode;

use serde::{Deserialize, Serialize};

/// Issue status state machine values.
///
/// Serialize/Deserialize are implemented manually to emit the Jira display names
/// ("Open", "In Progress", "Done") rather than Rust variant names.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IssueStatus {
    Open,
    InProgress,
    Done,
}

impl IssueStatus {
    /// Human-readable display name as returned by the Jira REST API.
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Open => "Open",
            Self::InProgress => "In Progress",
            Self::Done => "Done",
        }
    }

    /// Numeric string ID used by the Jira REST API status field.
    pub fn status_id(&self) -> &'static str {
        match self {
            Self::Open => "1",
            Self::InProgress => "3",
            Self::Done => "6",
        }
    }
}

impl Serialize for IssueStatus {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.display_name())
    }
}

impl<'de> Deserialize<'de> for IssueStatus {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "Open" => Ok(Self::Open),
            "In Progress" => Ok(Self::InProgress),
            "Done" => Ok(Self::Done),
            other => Err(serde::de::Error::unknown_variant(
                other,
                &["Open", "In Progress", "Done"],
            )),
        }
    }
}

/// A single issue record stored in the registry.
///
/// `org_id` carries the `OrgId` UUID string of the originating organisation for
/// shared-mode ingress tagging (S-3.2.07 / BC-3.2.004).
///
/// # Constraints (BC-3.2.004)
/// - `org_id` MUST be stored as a UUID string (not OrgSlug) — AI-opacity principle.
/// - `key` (`"PROJ-NNN"`) is MSSP-scoped and MUST NOT contain the OrgId UUID.
/// - The `issue_registry` is NOT re-keyed by OrgId — keyed by bare issue key per ADR-008 §1.2.
/// - `org_id` must NOT appear in query results under `"mode"`, `"shared"`, or `"dtu_mode"` keys.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueRecord {
    pub id: String,
    pub key: String,
    pub summary: String,
    pub issue_type: String,
    pub project_key: String,
    pub status: IssueStatus,
    pub comment_count: u32,
    /// OrgId UUID string of the originating organisation (S-3.2.07 / BC-3.2.004).
    ///
    /// Empty string when OrgId is not available (e.g. unauthenticated test calls).
    /// Implementation note: updated by `capture_issue` when called from the shared-mode
    /// ingress path (route handlers that resolve OrgId from `X-Prism-Org-Id` header).
    pub org_id: String,
    /// Raw fields payload from create request (extra fields preserved verbatim).
    pub fields: serde_json::Value,
}

/// Shared mutable state for the Jira Cloud REST API v3 DTU clone.
///
/// `Arc<JiraState>` is passed to every axum route handler via `axum::extract::State`.
pub struct JiraState {
    /// Issue registry: `issueKey → IssueRecord`.
    pub issue_registry: Mutex<HashMap<String, IssueRecord>>,

    /// Monotonically increasing issue number. Resets to 1000 on `reset()`.
    pub next_issue_num: AtomicU32,

    /// Shared failure mode, read by `FailureLayerShared` on every request.
    pub failure_mode: Arc<Mutex<FailureMode>>,

    /// Admin shared-secret token for `POST /dtu/configure`.
    pub admin_token: String,
}

/// Validated configuration payload for `POST /dtu/configure`.
#[derive(Debug, serde::Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct ConfigPayload {
    #[serde(default)]
    failure_mode: Option<String>,
    #[serde(default)]
    after_n_requests: Option<u32>,
    #[serde(default)]
    retry_after_secs: Option<u32>,
    #[serde(default)]
    at_request_n: Option<u32>,
    #[serde(default)]
    after_ms: Option<u64>,
}

impl Default for JiraState {
    fn default() -> Self {
        Self::new()
    }
}

impl JiraState {
    /// Construct a fresh `JiraState` with an empty issue registry.
    pub fn new() -> Self {
        Self::with_admin_token(uuid::Uuid::new_v4().to_string())
    }

    /// Construct with a specific admin token (used by clone to share between
    /// the route handler and BehavioralClone::admin_token()).
    pub fn with_admin_token(admin_token: String) -> Self {
        Self {
            issue_registry: Mutex::new(HashMap::new()),
            next_issue_num: AtomicU32::new(1000),
            failure_mode: Arc::new(Mutex::new(FailureMode::None)),
            admin_token,
        }
    }

    /// Reset all mutable state to initial values.
    ///
    /// - Clears the issue registry (all issues removed).
    /// - Resets `next_issue_num` to 1000 (predictable issue keys: PROJ-1000, PROJ-1001, ...).
    /// - Resets the failure mode to `FailureMode::None`.
    pub fn reset(&self) {
        // SAFETY: mutex poison only occurs if a previous holder panicked — not possible in normal operation.
        #[allow(clippy::expect_used)]
        let mut registry = self.issue_registry.lock().expect("issue_registry poisoned");
        registry.clear();

        self.next_issue_num.store(1000, Ordering::SeqCst);

        // SAFETY: same as above.
        #[allow(clippy::expect_used)]
        let mut mode = self.failure_mode.lock().expect("failure_mode poisoned");
        *mode = FailureMode::None;
    }

    /// Allocate the next issue number and return it, atomically incrementing the counter.
    pub fn next_issue_num(&self) -> u32 {
        self.next_issue_num.fetch_add(1, Ordering::SeqCst)
    }

    /// Insert an issue into the registry.
    pub fn insert_issue(&self, record: IssueRecord) {
        // SAFETY: mutex poison only occurs if a previous holder panicked — not possible in normal operation.
        #[allow(clippy::expect_used)]
        let mut registry = self.issue_registry.lock().expect("issue_registry poisoned");
        registry.insert(record.key.clone(), record);
    }

    /// Capture a Jira issue tagged with the originating `OrgId` (S-3.2.07 / BC-3.2.004).
    ///
    /// This is the shared-mode variant of `insert_issue`. The `OrgId` UUID string is
    /// embedded in `IssueRecord.org_id` per ADR-007 §2.6 Step 3 and BC-3.2.004 invariant 1.
    ///
    /// # Constraints (BC-3.2.004)
    /// - `OrgId` MUST appear in `IssueRecord.org_id` — never in `issue_key`, URL path, or headers.
    /// - UUID string form (not OrgSlug) MUST be used (AI-opacity principle).
    /// - The `issue_registry` is NOT re-keyed by OrgId — remains keyed by bare issue key (ADR-008 §1.2).
    ///
    /// # Implementation Status
    /// Stub added in chore(S-3.2.07). Full implementation (route handler wiring) is in S-3.2.07.
    #[cfg(feature = "dtu")]
    pub fn capture_issue(&self, org_id: OrgId, issue_key: String, record: IssueRecord) {
        let tagged = IssueRecord {
            org_id: org_id.to_string(),
            ..record
        };
        // SAFETY: mutex poison only occurs if a previous holder panicked — not possible in normal operation.
        #[allow(clippy::expect_used)]
        let mut registry = self.issue_registry.lock().expect("issue_registry poisoned");
        registry.insert(issue_key, tagged);
    }

    /// Look up an issue by key. Returns `None` if not found.
    pub fn get_issue(&self, key: &str) -> Option<IssueRecord> {
        // SAFETY: mutex poison only occurs if a previous holder panicked — not possible in normal operation.
        #[allow(clippy::expect_used)]
        let registry = self.issue_registry.lock().expect("issue_registry poisoned");
        registry.get(key).cloned()
    }

    /// Increment comment_count on an issue. Returns `false` if issue not found.
    pub fn increment_comment_count(&self, key: &str) -> bool {
        // SAFETY: mutex poison only occurs if a previous holder panicked — not possible in normal operation.
        #[allow(clippy::expect_used)]
        let mut registry = self.issue_registry.lock().expect("issue_registry poisoned");
        if let Some(issue) = registry.get_mut(key) {
            issue.comment_count += 1;
            true
        } else {
            false
        }
    }

    /// Apply a status transition to an issue.
    ///
    /// Returns `true` on success, `false` if the transition is invalid for
    /// the current status or the issue is not found.
    pub fn apply_transition(&self, key: &str, transition_id: &str) -> bool {
        // SAFETY: mutex poison only occurs if a previous holder panicked — not possible in normal operation.
        #[allow(clippy::expect_used)]
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

    /// Return available transition entries for the current issue status.
    ///
    /// Returns `None` if the issue is not found.
    pub fn available_transitions(&self, key: &str) -> Option<Vec<crate::types::TransitionEntry>> {
        use crate::types::TransitionEntry;

        // SAFETY: mutex poison only occurs if a previous holder panicked — not possible in normal operation.
        #[allow(clippy::expect_used)]
        let registry = self.issue_registry.lock().expect("issue_registry poisoned");
        let issue = registry.get(key)?;
        let transitions = match issue.status {
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
        };
        Some(transitions)
    }

    /// Return all issues in the registry (for GET /dtu/issues).
    pub fn all_issues(&self) -> Vec<IssueRecord> {
        // SAFETY: mutex poison only occurs if a previous holder panicked — not possible in normal operation.
        #[allow(clippy::expect_used)]
        let registry = self.issue_registry.lock().expect("issue_registry poisoned");
        registry.values().cloned().collect()
    }

    /// Apply a JSON configuration patch (from `POST /dtu/configure`).
    pub fn apply_config(&self, config: &serde_json::Value) -> anyhow::Result<()> {
        let payload: ConfigPayload = serde_json::from_value(config.clone())
            .map_err(|e| anyhow::anyhow!("invalid /dtu/configure payload: {e}"))?;

        if let Some(mode_str) = payload.failure_mode.as_deref() {
            let new_mode = match mode_str {
                "none" => FailureMode::None,
                "rate_limit" => {
                    let after_n = payload.after_n_requests.unwrap_or(0);
                    let retry_after = payload.retry_after_secs.unwrap_or(30);
                    FailureMode::RateLimit {
                        after_n_requests: after_n,
                        retry_after_secs: retry_after,
                    }
                }
                "malformed_response" => FailureMode::MalformedResponse,
                "auth_reject" => FailureMode::AuthReject,
                "internal_error" => {
                    let at_n = payload.at_request_n.unwrap_or(1);
                    FailureMode::InternalError { at_request_n: at_n }
                }
                "network_timeout" => {
                    let after_ms = payload.after_ms.unwrap_or(5000);
                    FailureMode::NetworkTimeout { after_ms }
                }
                other => {
                    anyhow::bail!("unknown failure_mode: {other}");
                }
            };
            // SAFETY: mutex poison only occurs if a previous holder panicked — not possible in normal operation.
            #[allow(clippy::expect_used)]
            let mut guard = self
                .failure_mode
                .lock()
                .expect("JiraState: failure_mode lock poisoned");
            *guard = new_mode;
        }
        Ok(())
    }
}
