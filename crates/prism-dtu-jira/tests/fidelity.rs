#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Fidelity test: full create → add comment → transition to InProgress → transition to Done lifecycle.
//!
//! Tests:
//! - Full lifecycle: create issue → comment → Start Progress → Done
//! - Invalid transition (Done → InProgress) returns 400
//! - Missing auth returns 401
//! - Unknown issue key returns 404
//! - Missing project.key returns 400
//! - Unknown issuetype.name returns 400
//! - transitions list for Open status returns ids "11" and "31"
//! - transitions list for Done status returns empty list

#![cfg(feature = "dtu")]

use prism_dtu_common::BehavioralClone;
use prism_dtu_jira::JiraClone;

fn basic_auth_header() -> String {
    use base64::Engine as _;
    let encoded = base64::engine::general_purpose::STANDARD.encode("testuser:testtoken");
    format!("Basic {encoded}")
}

#[tokio::test]
async fn test_full_lifecycle_create_comment_transition() {
    let mut clone = JiraClone::new().expect("JiraClone::new failed");
    clone.start().await.expect("start failed");
    let base = clone.base_url();
    let client = reqwest::Client::new();

    // AC-1: Create issue
    let create_resp = client
        .post(format!("{base}/rest/api/3/issue"))
        .header("Authorization", basic_auth_header())
        .json(&serde_json::json!({
            "fields": {
                "project": {"key": "ACME-SEC"},
                "issuetype": {"name": "Task"},
                "summary": "Test incident for lifecycle"
            }
        }))
        .send()
        .await
        .expect("create request failed");
    assert_eq!(create_resp.status(), 201, "Expected 201 on create");
    let create_body: serde_json::Value = create_resp.json().await.expect("create body failed");
    let issue_key = create_body["key"].as_str().expect("key missing").to_owned();
    assert!(
        issue_key.starts_with("ACME-SEC-"),
        "key must start with ACME-SEC-"
    );

    // AC-1: GET /dtu/issues confirms status: Open
    let dtu_resp = client
        .get(format!("{base}/dtu/issues"))
        .send()
        .await
        .expect("dtu/issues request failed");
    assert_eq!(dtu_resp.status(), 200);
    let dtu_body: serde_json::Value = dtu_resp.json().await.expect("dtu body failed");
    let issues = dtu_body["issues"].as_array().expect("issues array missing");
    let found = issues
        .iter()
        .find(|i| i["key"] == issue_key)
        .expect("issue not found in /dtu/issues");
    assert_eq!(found["status"], "Open", "Newly created issue must be Open");

    // AC-2: Add comment
    let comment_resp = client
        .post(format!("{base}/rest/api/3/issue/{issue_key}/comment"))
        .header("Authorization", basic_auth_header())
        .json(&serde_json::json!({"body": {"type": "doc", "content": []}}))
        .send()
        .await
        .expect("comment request failed");
    assert_eq!(comment_resp.status(), 201, "Expected 201 on add comment");

    // Verify comment_count incremented
    let get_resp = client
        .get(format!("{base}/rest/api/3/issue/{issue_key}"))
        .header("Authorization", basic_auth_header())
        .send()
        .await
        .expect("get issue failed");
    assert_eq!(get_resp.status(), 200);
    let get_body: serde_json::Value = get_resp.json().await.expect("get body failed");
    assert_eq!(
        get_body["fields"]["comment"]["total"], 1,
        "AC-2: comment.total must be 1 after adding a comment"
    );

    // AC-3: List transitions when Open — must include "11" and "31"
    let trans_resp = client
        .get(format!("{base}/rest/api/3/issue/{issue_key}/transitions"))
        .header("Authorization", basic_auth_header())
        .send()
        .await
        .expect("list transitions failed");
    assert_eq!(trans_resp.status(), 200);
    let trans_body: serde_json::Value = trans_resp.json().await.expect("trans body failed");
    let transitions = trans_body["transitions"]
        .as_array()
        .expect("transitions array missing");
    let ids: Vec<&str> = transitions
        .iter()
        .filter_map(|t| t["id"].as_str())
        .collect();
    assert!(
        ids.contains(&"11"),
        "AC-3: transition 11 must be available from Open"
    );
    assert!(
        ids.contains(&"31"),
        "AC-3: transition 31 must be available from Open"
    );

    // AC-4: Execute transition 11 (Start Progress)
    let exec_resp = client
        .post(format!("{base}/rest/api/3/issue/{issue_key}/transitions"))
        .header("Authorization", basic_auth_header())
        .json(&serde_json::json!({"transition": {"id": "11"}}))
        .send()
        .await
        .expect("execute transition failed");
    assert_eq!(
        exec_resp.status(),
        204,
        "AC-4: execute transition must return 204"
    );

    // Verify status is now In Progress
    let get_resp2 = client
        .get(format!("{base}/rest/api/3/issue/{issue_key}"))
        .header("Authorization", basic_auth_header())
        .send()
        .await
        .expect("get issue after transition failed");
    let get_body2: serde_json::Value = get_resp2.json().await.expect("get body2 failed");
    assert_eq!(
        get_body2["fields"]["status"]["name"], "In Progress",
        "AC-4: status must be In Progress after transition 11"
    );

    // Transition to Done (id "21")
    let done_resp = client
        .post(format!("{base}/rest/api/3/issue/{issue_key}/transitions"))
        .header("Authorization", basic_auth_header())
        .json(&serde_json::json!({"transition": {"id": "21"}}))
        .send()
        .await
        .expect("execute transition to done failed");
    assert_eq!(
        done_resp.status(),
        204,
        "Transition to Done must return 204"
    );

    // Verify status is now Done
    let get_resp3 = client
        .get(format!("{base}/rest/api/3/issue/{issue_key}"))
        .header("Authorization", basic_auth_header())
        .send()
        .await
        .expect("get issue after done failed");
    let get_body3: serde_json::Value = get_resp3.json().await.expect("get body3 failed");
    assert_eq!(
        get_body3["fields"]["status"]["name"], "Done",
        "Status must be Done after transition 21"
    );

    // EC-002: transitions list for Done status returns empty
    let done_trans_resp = client
        .get(format!("{base}/rest/api/3/issue/{issue_key}/transitions"))
        .header("Authorization", basic_auth_header())
        .send()
        .await
        .expect("list transitions Done failed");
    let done_trans_body: serde_json::Value = done_trans_resp
        .json()
        .await
        .expect("done trans body failed");
    let done_transitions = done_trans_body["transitions"]
        .as_array()
        .expect("array missing");
    assert!(
        done_transitions.is_empty(),
        "EC-002: transitions must be empty when status is Done"
    );

    // AC-5: Invalid transition from Done returns 400
    let invalid_resp = client
        .post(format!("{base}/rest/api/3/issue/{issue_key}/transitions"))
        .header("Authorization", basic_auth_header())
        .json(&serde_json::json!({"transition": {"id": "11"}}))
        .send()
        .await
        .expect("invalid transition request failed");
    assert_eq!(
        invalid_resp.status(),
        400,
        "AC-5: invalid transition must return 400"
    );
    let invalid_body: serde_json::Value = invalid_resp.json().await.expect("invalid body failed");
    assert_eq!(
        invalid_body["errorMessages"][0], "Invalid transition id",
        "AC-5: error message must match exactly"
    );

    clone.stop().await.expect("stop failed");
}

#[tokio::test]
async fn test_missing_auth_returns_401() {
    let mut clone = JiraClone::new().expect("JiraClone::new failed");
    clone.start().await.expect("start failed");
    let base = clone.base_url();
    let client = reqwest::Client::new();

    // AC-8: No auth header on create
    let resp = client
        .post(format!("{base}/rest/api/3/issue"))
        .json(&serde_json::json!({
            "fields": {
                "project": {"key": "ACME-SEC"},
                "issuetype": {"name": "Task"},
                "summary": "no auth test"
            }
        }))
        .send()
        .await
        .expect("request failed");
    assert_eq!(resp.status(), 401, "AC-8: missing auth must return 401");
    let body: serde_json::Value = resp.json().await.expect("body failed");
    assert_eq!(
        body["errorMessages"][0], "Basic authentication required",
        "AC-8: error message must match exactly"
    );

    clone.stop().await.expect("stop failed");
}

#[tokio::test]
async fn test_missing_project_key_returns_400() {
    let mut clone = JiraClone::new().expect("JiraClone::new failed");
    clone.start().await.expect("start failed");
    let base = clone.base_url();
    let client = reqwest::Client::new();

    // AC-6: Missing fields.project.key
    let resp = client
        .post(format!("{base}/rest/api/3/issue"))
        .header("Authorization", basic_auth_header())
        .json(&serde_json::json!({
            "fields": {
                "issuetype": {"name": "Task"},
                "summary": "missing project key"
            }
        }))
        .send()
        .await
        .expect("request failed");
    assert_eq!(
        resp.status(),
        400,
        "AC-6: missing project.key must return 400"
    );
    let body: serde_json::Value = resp.json().await.expect("body failed");
    assert_eq!(
        body["errors"]["project"], "required",
        "AC-6: errors.project must be 'required'"
    );

    clone.stop().await.expect("stop failed");
}

#[tokio::test]
async fn test_unknown_issuetype_returns_400() {
    let mut clone = JiraClone::new().expect("JiraClone::new failed");
    clone.start().await.expect("start failed");
    let base = clone.base_url();
    let client = reqwest::Client::new();

    // AC-7: Unknown issuetype.name "Feature"
    let resp = client
        .post(format!("{base}/rest/api/3/issue"))
        .header("Authorization", basic_auth_header())
        .json(&serde_json::json!({
            "fields": {
                "project": {"key": "ACME-SEC"},
                "issuetype": {"name": "Feature"},
                "summary": "unknown type"
            }
        }))
        .send()
        .await
        .expect("request failed");
    assert_eq!(
        resp.status(),
        400,
        "AC-7: unknown issuetype must return 400"
    );
    let body: serde_json::Value = resp.json().await.expect("body failed");
    assert_eq!(
        body["errors"]["issuetype"], "unknown",
        "AC-7: errors.issuetype must be 'unknown'"
    );

    clone.stop().await.expect("stop failed");
}

#[tokio::test]
async fn test_unknown_issue_key_returns_404() {
    let mut clone = JiraClone::new().expect("JiraClone::new failed");
    clone.start().await.expect("start failed");
    let base = clone.base_url();
    let client = reqwest::Client::new();

    // AC-9: GET unknown issue
    let resp = client
        .get(format!("{base}/rest/api/3/issue/UNKNOWN-999"))
        .header("Authorization", basic_auth_header())
        .send()
        .await
        .expect("request failed");
    assert_eq!(resp.status(), 404, "AC-9: unknown issue must return 404");
    let body: serde_json::Value = resp.json().await.expect("body failed");
    assert_eq!(
        body["errorMessages"][0], "Issue does not exist",
        "AC-9: error message must match exactly"
    );

    clone.stop().await.expect("stop failed");
}

#[tokio::test]
async fn test_reset_clears_all_issues() {
    let mut clone = JiraClone::new().expect("JiraClone::new failed");
    clone.start().await.expect("start failed");
    let base = clone.base_url();
    let client = reqwest::Client::new();

    // Create an issue
    let create_resp = client
        .post(format!("{base}/rest/api/3/issue"))
        .header("Authorization", basic_auth_header())
        .json(&serde_json::json!({
            "fields": {
                "project": {"key": "TEST"},
                "issuetype": {"name": "Bug"},
                "summary": "issue to be cleared"
            }
        }))
        .send()
        .await
        .expect("create failed");
    assert_eq!(create_resp.status(), 201);
    let create_body: serde_json::Value = create_resp.json().await.expect("body failed");
    let key = create_body["key"].as_str().expect("key missing").to_owned();

    // Verify it exists
    let get_resp = client
        .get(format!("{base}/rest/api/3/issue/{key}"))
        .header("Authorization", basic_auth_header())
        .send()
        .await
        .expect("get failed");
    assert_eq!(get_resp.status(), 200);

    // EC-005: Reset
    clone.reset().await.expect("reset failed");

    // Verify it no longer exists
    let after_reset = client
        .get(format!("{base}/rest/api/3/issue/{key}"))
        .header("Authorization", basic_auth_header())
        .send()
        .await
        .expect("get after reset failed");
    assert_eq!(
        after_reset.status(),
        404,
        "EC-005: issue must be gone after reset"
    );

    // Verify next key starts from PROJ-1000 again
    let create2_resp = client
        .post(format!("{base}/rest/api/3/issue"))
        .header("Authorization", basic_auth_header())
        .json(&serde_json::json!({
            "fields": {
                "project": {"key": "PROJ"},
                "issuetype": {"name": "Task"},
                "summary": "after reset issue"
            }
        }))
        .send()
        .await
        .expect("create2 failed");
    assert_eq!(create2_resp.status(), 201);
    let create2_body: serde_json::Value = create2_resp.json().await.expect("body2 failed");
    let key2 = create2_body["key"]
        .as_str()
        .expect("key2 missing")
        .to_owned();
    assert_eq!(
        key2, "PROJ-1000",
        "After reset, first new issue must be PROJ-1000"
    );

    clone.stop().await.expect("stop failed");
}
