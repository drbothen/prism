//! Green Gate tests for S-3.2.07 — prism-dtu-jira shared-mode OrgId ingress tagging.
//!
//! # Behavioral contracts exercised
//!
//! - BC-3.2.004: Shared-Mode DTU Tags OrgId in Payload Body Not in Routing Headers
//! - BC-3.2.005: DTU Mode is Deployment-Time Config — No Runtime API to Change It
//!
//! # Acceptance criteria
//!
//! - AC-001 / BC-3.2.004 postcondition 1: OrgId UUID in captured IssueRecord.org_id
//! - AC-002 / BC-3.2.004 postcondition 2: OrgId absent from issue_key and HTTP routing fields
//! - AC-003 / BC-3.2.004 postcondition 4: Concurrent issues from distinct orgs distinguished
//! - AC-004 / BC-3.2.004 postcondition 5: No mode metadata in issue query results
//! - AC-005 / BC-3.2.005 postcondition 1: DtuMode::Shared set at startup and immutable
//! - AC-006 / BC-3.2.005 postcondition 3: Invalid mode string rejected with deserialisation error
//!
//! # Naming convention
//!
//! `test_BC_S_SS_NNN_xxx` per TDD discipline.

#![allow(clippy::unwrap_used, clippy::expect_used, non_snake_case)]
#![cfg(feature = "dtu")]

use prism_dtu_common::{BehavioralClone, DtuMode};
use prism_dtu_jira::clone::JIRA_DTU_MODE;
use prism_dtu_jira::JiraClone;

// ---------------------------------------------------------------------------
// Test helpers
// ---------------------------------------------------------------------------

/// Start a fresh `JiraClone` and return (clone, base_url, reqwest::Client).
async fn start_clone() -> (JiraClone, String, reqwest::Client) {
    let mut clone = JiraClone::new().expect("JiraClone::new");
    clone.start().await.expect("JiraClone::start");
    let base_url = clone.base_url();
    let client = reqwest::Client::new();
    (clone, base_url, client)
}

/// Canonical test OrgId UUIDs (AI-opaque form, BC-3.2.004 invariant 1).
const ORG_UUID_A: &str = "00000000-0000-7000-8000-000000000001";
const ORG_UUID_B: &str = "00000000-0000-7000-8000-000000000002";

/// Build the Basic-auth header used by all Jira fidelity requests.
fn basic_auth(user: &str, pass: &str) -> String {
    use base64::Engine as _;
    let encoded = base64::engine::general_purpose::STANDARD.encode(format!("{user}:{pass}"));
    format!("Basic {encoded}")
}

/// POST a create-issue request and return the parsed JSON response body.
///
/// Sends `Authorization: Basic` + optional `X-Prism-Org-Id` header.
async fn create_issue(
    client: &reqwest::Client,
    base_url: &str,
    org_uuid: Option<&str>,
    project_key: &str,
    summary: &str,
) -> (u16, serde_json::Value) {
    let mut req = client
        .post(format!("{base_url}/rest/api/3/issue"))
        .header("Authorization", basic_auth("user", "token"))
        .json(&serde_json::json!({
            "fields": {
                "project": {"key": project_key},
                "issuetype": {"name": "Task"},
                "summary": summary,
            }
        }));

    if let Some(uuid) = org_uuid {
        req = req.header("X-Prism-Org-Id", uuid);
    }

    let resp = req.send().await.expect("POST /rest/api/3/issue");
    let status = resp.status().as_u16();
    let body: serde_json::Value = resp.json().await.expect("JSON body");
    (status, body)
}

// ---------------------------------------------------------------------------
// AC-001 / BC-3.2.004 postcondition 1
// OrgId UUID in captured IssueRecord.org_id
// ---------------------------------------------------------------------------

/// AC-001: OrgId UUID appears in IssueRecord.org_id after capture_issue.
///
/// Exercises TV-3.2.004-01 and VP-3.2.004-01.
#[tokio::test]
async fn test_BC_3_2_004_ac001_org_id_in_issue_record() {
    let (mut clone, base_url, client) = start_clone().await;

    let (status, body) =
        create_issue(&client, &base_url, Some(ORG_UUID_A), "PROJ", "Test issue").await;
    assert_eq!(status, 201, "create issue must return 201; body: {body}");
    let issue_key = body["key"].as_str().expect("response.key");

    // Inspect the captured IssueRecord via the internal state.
    // The route handler must have called capture_issue so org_id == ORG_UUID_A.
    let state = clone.state();
    let record = state
        .get_issue(issue_key)
        .expect("issue must be in registry");

    assert_eq!(
        record.org_id, ORG_UUID_A,
        "BC-3.2.004 postcondition 1: IssueRecord.org_id must equal the sender's UUID"
    );

    clone.stop().await.expect("stop");
}

// ---------------------------------------------------------------------------
// AC-002 / BC-3.2.004 postcondition 2
// OrgId absent from issue_key and HTTP routing fields
// ---------------------------------------------------------------------------

/// AC-002: issue_key does not contain org_id UUID (MSSP-scoped per ADR-008 §1.2).
///
/// Exercises TV-3.2.004-01 (issue_key side) and VP-3.2.004-02.
#[tokio::test]
async fn test_BC_3_2_004_ac002_issue_key_not_org_scoped() {
    let (mut clone, base_url, client) = start_clone().await;

    let (status, body) = create_issue(
        &client,
        &base_url,
        Some(ORG_UUID_A),
        "PROJ",
        "Key isolation test",
    )
    .await;
    assert_eq!(status, 201, "create issue must return 201; body: {body}");
    let issue_key = body["key"].as_str().expect("response.key");

    // BC-3.2.004 postcondition 2: issue_key must NOT contain the OrgId UUID.
    // Jira issue keys are MSSP-scoped ("PROJ-1000") — never org-scoped.
    assert!(
        !issue_key.contains(ORG_UUID_A),
        "BC-3.2.004 postcondition 2 / ADR-008 §1.2: issue_key '{issue_key}' must not \
         contain the OrgId UUID '{ORG_UUID_A}'"
    );

    clone.stop().await.expect("stop");
}

/// AC-002: org_id does not appear in the POST /rest/api/3/issue response URL or headers.
///
/// Exercises VP-3.2.004-02 — HTTP routing fields must carry no OrgId.
#[tokio::test]
async fn test_BC_3_2_004_ac002_org_id_absent_from_routing() {
    let (mut clone, base_url, client) = start_clone().await;

    let resp = client
        .post(format!("{base_url}/rest/api/3/issue"))
        .header("Authorization", basic_auth("user", "token"))
        .header("X-Prism-Org-Id", ORG_UUID_A)
        .json(&serde_json::json!({
            "fields": {
                "project": {"key": "PROJ"},
                "issuetype": {"name": "Task"},
                "summary": "Routing isolation test",
            }
        }))
        .send()
        .await
        .expect("POST /rest/api/3/issue");

    assert_eq!(resp.status().as_u16(), 201);

    // Check that the OrgId UUID does not appear in response headers.
    for (name, value) in resp.headers() {
        let val_str = value.to_str().unwrap_or("");
        assert!(
            !val_str.contains(ORG_UUID_A),
            "BC-3.2.004 postcondition 2: response header '{}' must not contain OrgId UUID; \
             found '{val_str}'",
            name
        );
    }

    // Check that the response URL does not contain the OrgId UUID.
    let url_str = resp.url().as_str();
    assert!(
        !url_str.contains(ORG_UUID_A),
        "BC-3.2.004 postcondition 2: response URL '{url_str}' must not contain OrgId UUID"
    );

    // The response body (JSON) is the create-issue response — it should only contain
    // id, key, and self fields. None of these should contain the OrgId UUID.
    let body: serde_json::Value = resp.json().await.expect("JSON body");
    let body_str = body.to_string();
    assert!(
        !body_str.contains(ORG_UUID_A),
        "BC-3.2.004 postcondition 2: response body must not contain OrgId UUID; \
         body: {body_str}"
    );

    clone.stop().await.expect("stop");
}

// ---------------------------------------------------------------------------
// AC-003 / BC-3.2.004 postcondition 4
// Concurrent issues from different orgs are independently attributed
// ---------------------------------------------------------------------------

/// AC-003: concurrent issues from org_A and org_B each carry their sender's OrgId.
///
/// Exercises TV-3.2.004-02 and VP-3.2.004-03.
#[tokio::test]
async fn test_BC_3_2_004_ac003_concurrent_issues_distinguished() {
    let (mut clone, base_url, _client) = start_clone().await;

    let base_url_a = base_url.clone();
    let base_url_b = base_url.clone();

    // Spawn concurrent create-issue requests for org_A and org_B.
    let task_a = tokio::spawn(async move {
        let c = reqwest::Client::new();
        c.post(format!("{base_url_a}/rest/api/3/issue"))
            .header("Authorization", basic_auth("user", "token"))
            .header("X-Prism-Org-Id", ORG_UUID_A)
            .json(&serde_json::json!({
                "fields": {
                    "project": {"key": "PROJA"},
                    "issuetype": {"name": "Task"},
                    "summary": "Issue from org A",
                }
            }))
            .send()
            .await
            .expect("task A POST")
    });

    let task_b = tokio::spawn(async move {
        let c = reqwest::Client::new();
        c.post(format!("{base_url_b}/rest/api/3/issue"))
            .header("Authorization", basic_auth("user", "token"))
            .header("X-Prism-Org-Id", ORG_UUID_B)
            .json(&serde_json::json!({
                "fields": {
                    "project": {"key": "PROJB"},
                    "issuetype": {"name": "Task"},
                    "summary": "Issue from org B",
                }
            }))
            .send()
            .await
            .expect("task B POST")
    });

    let (r_a, r_b) = tokio::join!(task_a, task_b);
    let resp_a: serde_json::Value = r_a.expect("task A").json().await.expect("body A");
    let resp_b: serde_json::Value = r_b.expect("task B").json().await.expect("body B");

    let key_a = resp_a["key"].as_str().expect("key_a");
    let key_b = resp_b["key"].as_str().expect("key_b");

    let state = clone.state();

    let record_a = state.get_issue(key_a).expect("issue A must be in registry");
    let record_b = state.get_issue(key_b).expect("issue B must be in registry");

    assert_eq!(
        record_a.org_id, ORG_UUID_A,
        "BC-3.2.004 postcondition 4: issue A must carry org_A's UUID"
    );
    assert_eq!(
        record_b.org_id, ORG_UUID_B,
        "BC-3.2.004 postcondition 4: issue B must carry org_B's UUID"
    );

    // Cross-check: neither record carries the other org's UUID.
    assert_ne!(
        record_a.org_id, ORG_UUID_B,
        "BC-3.2.004 postcondition 4: issue A must NOT carry org_B's UUID"
    );
    assert_ne!(
        record_b.org_id, ORG_UUID_A,
        "BC-3.2.004 postcondition 4: issue B must NOT carry org_A's UUID"
    );

    clone.stop().await.expect("stop");
}

// ---------------------------------------------------------------------------
// AC-004 / BC-3.2.004 postcondition 5
// No mode metadata in issue query results (GET /dtu/issues)
// ---------------------------------------------------------------------------

/// AC-004: GET /dtu/issues response rows contain no "mode", "shared", or "dtu_mode" fields.
///
/// Exercises TV-3.2.004-05 and VP-3.2.004-04.
#[tokio::test]
async fn test_BC_3_2_004_ac004_mode_metadata_absent_from_query_results() {
    let (mut clone, base_url, client) = start_clone().await;

    // Create an issue first so there is something in the registry.
    let (status, _) = create_issue(
        &client,
        &base_url,
        Some(ORG_UUID_A),
        "PROJ",
        "Mode metadata test",
    )
    .await;
    assert_eq!(status, 201);

    // GET /dtu/issues — the response rows must not carry mode metadata.
    let resp = client
        .get(format!("{base_url}/dtu/issues"))
        .send()
        .await
        .expect("GET /dtu/issues");
    assert_eq!(resp.status().as_u16(), 200);

    let body: serde_json::Value = resp.json().await.expect("JSON body");
    let issues = body["issues"].as_array().expect("issues array");
    assert!(
        !issues.is_empty(),
        "expected at least one issue in GET /dtu/issues response"
    );

    // BC-3.2.004 postcondition 5: no mode metadata in result rows.
    for (i, issue) in issues.iter().enumerate() {
        let row_str = issue.to_string();
        assert!(
            issue.get("mode").is_none(),
            "BC-3.2.004 postcondition 5 (issue {i}): 'mode' must not appear in query results; \
             row: {row_str}"
        );
        assert!(
            issue.get("shared").is_none(),
            "BC-3.2.004 postcondition 5 (issue {i}): 'shared' must not appear in query results; \
             row: {row_str}"
        );
        assert!(
            issue.get("dtu_mode").is_none(),
            "BC-3.2.004 postcondition 5 (issue {i}): 'dtu_mode' must not appear in query results; \
             row: {row_str}"
        );
    }

    clone.stop().await.expect("stop");
}

// ---------------------------------------------------------------------------
// AC-005 / BC-3.2.005 postcondition 1 + invariant 1
// DtuMode::Shared set at startup and immutable for process lifetime
// ---------------------------------------------------------------------------

/// AC-005: JIRA_DTU_MODE constant is DtuMode::Shared (compile-time assertion).
///
/// Exercises BC-3.2.005 postcondition 1 and TV-3.2.005-01.
/// This test is expected to PASS immediately — the constant is already set.
#[test]
fn test_BC_3_2_005_ac005_jira_dtu_mode_is_shared() {
    // This test exercises the compile-time constant declared in clone.rs.
    // It will pass once JIRA_DTU_MODE is set to DtuMode::Shared (stub is already there).
    assert_eq!(
        JIRA_DTU_MODE,
        DtuMode::Shared,
        "JIRA_DTU_MODE must be DtuMode::Shared per BC-3.2.005 postcondition 1"
    );
}

/// AC-005: DtuMode::Shared cannot be changed after startup via any in-process API.
///
/// Exercises BC-3.2.005 postcondition 4 and TV-3.2.005-05.
#[tokio::test]
async fn test_BC_3_2_005_ac005_mode_immutable_after_startup() {
    let mut clone = JiraClone::new().expect("JiraClone::new");
    clone.start().await.expect("start");
    let base_url = clone.base_url();
    let client = reqwest::Client::new();

    // Attempt to change mode via POST /dtu/configure (must be rejected as an unknown field).
    // BC-3.2.005 postcondition 4: whether the call returns Ok or Err, DtuMode MUST NOT change.
    let configure_result = clone.configure(serde_json::json!({"mode": "client"})).await;
    // Result is irrelevant — what matters is that JIRA_DTU_MODE is still DtuMode::Shared.
    let _ = configure_result;

    // Static check: compile-time constant cannot be mutated.
    assert_eq!(
        JIRA_DTU_MODE,
        DtuMode::Shared,
        "BC-3.2.005 postcondition 4: JIRA_DTU_MODE must remain DtuMode::Shared \
         after any attempted runtime mode change"
    );

    // Dynamic check: the shared-mode dispatch path is still active — newly created
    // issues must still carry an org_id.
    let (status, body) = create_issue(
        &client,
        &base_url,
        Some(ORG_UUID_A),
        "PROJ",
        "Mode immutable check",
    )
    .await;
    assert_eq!(
        status, 201,
        "create issue must succeed after configure attempt; body: {body}"
    );
    let issue_key = body["key"].as_str().expect("response.key");

    let state = clone.state();
    let record = state
        .get_issue(issue_key)
        .expect("issue must be in registry");
    assert_eq!(
        record.org_id, ORG_UUID_A,
        "BC-3.2.005 postcondition 4: shared-mode dispatch must still tag org_id after \
         a (rejected) configure attempt"
    );

    clone.stop().await.expect("stop");
}

// ---------------------------------------------------------------------------
// AC-006 / BC-3.2.005 postcondition 3
// Invalid mode string rejected by serde deserialization
// ---------------------------------------------------------------------------

/// AC-006: mode = "SHared" (wrong case) fails serde deserialization.
///
/// Exercises BC-3.2.005 postcondition 3 and TV-3.2.005-03 (invalid mode string).
#[test]
fn test_BC_3_2_005_ac006_invalid_mode_string_rejected() {
    // BC-3.2.005 postcondition 3: serde must reject any variant that is not "shared" or "client".
    // The prism_core::DtuMode has #[serde(rename_all = "lowercase")] so "SHared" is invalid.
    let result: Result<DtuMode, _> = serde_json::from_str("\"SHared\"");
    assert!(
        result.is_err(),
        "BC-3.2.005 postcondition 3: DtuMode must reject unknown variant 'SHared'; got Ok({:?})",
        result.ok()
    );
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("SHared") || err_msg.contains("variant") || err_msg.contains("unknown"),
        "BC-3.2.005 postcondition 3: error message must identify the offending value or be \
         a variant error; got: {err_msg}"
    );
}
