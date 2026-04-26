#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Fidelity tests: Jira Cloud REST API v3 L3 behavioral clone.
//!
//! Coverage map (all ACs + ECs from S-6.13):
//!
//! AC-1  — create issue 201, key format, /dtu/issues shows Open status
//! AC-2  — add comment 201, GET issue shows comment.total incremented
//! AC-3  — GET transitions when Open: ids "11" + "31" present
//! AC-4  — POST transition id "11": 204 + status becomes "In Progress"
//! AC-5  — invalid transition from Done: 400 + exact error message
//! AC-6  — missing project.key: 400 + errors.project = "required"
//! AC-7  — unknown issuetype.name: 400 + errors.issuetype = "unknown"
//! AC-8  — missing auth header: 401 + exact error message
//! AC-9  — GET unknown issue key: 404 + exact error message
//! AC-10 — RateLimit failure mode: 429 returned; issue NOT persisted (EC-006)
//! EC-001 — extra unknown fields in create body are silently ignored
//! EC-002 — GET transitions when Done: empty list
//! EC-003 — two creates same project: second key incremented (PROJ-1001)
//! EC-004 — comment on Done issue: 201 (Jira permits comments on closed issues)
//! EC-005 — reset() clears registry; subsequent GET returns 404; counter resets to 1000
//!
//! Additional subtle fidelity assertions:
//! - GET issue response has "self" field (Jira REST v3 contract)
//! - GET issue response has fields.status.id as numeric string
//! - POST create response has "id" + "key" + "self" fields
//! - Auth: wrong scheme (Bearer) returns 401
//! - Auth: invalid base64 returns 401
//! - POST transitions on non-existent issue: 404
//! - GET transitions on non-existent issue: 404
//! - Missing issuetype entirely: 400 (different from unknown issuetype)
//! - Missing summary: 400
//! - Open → Done direct transition (id "31"): 204 + status "Done"
//! - GET /dtu/health returns 200 without auth
//! - POST /dtu/configure without X-Admin-Token returns 401
//! - Transition name "Start Progress" and "Close" correct in Open transitions list
//! - InProgress → Done transition (id "21") via dedicated single-transition test

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

// ---------------------------------------------------------------------------
// AC-10 / EC-006 — RateLimit failure mode: 429 returned; issue NOT persisted
// ---------------------------------------------------------------------------

/// AC-10: Given `FailureMode::RateLimit` configured with threshold 0,
/// When POST /rest/api/3/issue is called, Then HTTP 429 is returned AND
/// the issue is NOT persisted in the registry (EC-006 atomicity).
#[tokio::test]
async fn test_ac10_rate_limit_429_returned_and_issue_not_persisted() {
    let mut clone = JiraClone::new().expect("JiraClone::new failed");
    clone.start().await.expect("start failed");
    let base = clone.base_url();
    let admin_token = clone.admin_token().to_string();
    let client = reqwest::Client::new();

    // Configure rate limit: trigger immediately (after_n=0).
    let configure_resp = client
        .post(format!("{base}/dtu/configure"))
        .header("X-Admin-Token", &admin_token)
        .json(&serde_json::json!({
            "failure_mode": "rate_limit",
            "after_n_requests": 0,
            "retry_after_secs": 30
        }))
        .send()
        .await
        .expect("configure request failed");
    assert_eq!(
        configure_resp.status(),
        200,
        "AC-10: POST /dtu/configure must return 200"
    );

    // Next create request must return 429 (rate-limited).
    let create_resp = client
        .post(format!("{base}/rest/api/3/issue"))
        .header("Authorization", basic_auth_header())
        .json(&serde_json::json!({
            "fields": {
                "project": {"key": "RATELIM"},
                "issuetype": {"name": "Task"},
                "summary": "should not be persisted"
            }
        }))
        .send()
        .await
        .expect("rate-limited create request failed");
    assert_eq!(
        create_resp.status(),
        429,
        "AC-10: create issue when rate-limited must return 429"
    );

    // EC-006: The issue must NOT be in the registry (atomicity: fail before state write).
    // Reset failure mode so /dtu/issues is reachable normally.
    let reset_mode_resp = client
        .post(format!("{base}/dtu/configure"))
        .header("X-Admin-Token", &admin_token)
        .json(&serde_json::json!({
            "failure_mode": "none"
        }))
        .send()
        .await
        .expect("reset configure failed");
    assert_eq!(reset_mode_resp.status(), 200);

    let dtu_resp = client
        .get(format!("{base}/dtu/issues"))
        .send()
        .await
        .expect("dtu/issues failed");
    let dtu_body: serde_json::Value = dtu_resp.json().await.expect("dtu body failed");
    let issues = dtu_body["issues"].as_array().expect("issues array missing");
    assert!(
        issues.is_empty(),
        "EC-006: issue must NOT be persisted when rate limit triggered (got {} issues)",
        issues.len()
    );

    clone.stop().await.expect("stop failed");
}

// ---------------------------------------------------------------------------
// EC-001 — Extra unknown fields in create body are silently ignored
// ---------------------------------------------------------------------------

/// EC-001: Given POST /rest/api/3/issue with all required fields plus extra unknown
/// fields, Then the response is HTTP 201 and the issue is created normally.
#[tokio::test]
async fn test_ec001_extra_fields_in_create_body_are_ignored() {
    let mut clone = JiraClone::new().expect("JiraClone::new failed");
    clone.start().await.expect("start failed");
    let base = clone.base_url();
    let client = reqwest::Client::new();

    let create_resp = client
        .post(format!("{base}/rest/api/3/issue"))
        .header("Authorization", basic_auth_header())
        .json(&serde_json::json!({
            "fields": {
                "project": {"key": "EXTRA"},
                "issuetype": {"name": "Bug"},
                "summary": "extra fields test",
                "priority": {"name": "High"},
                "labels": ["security", "critical"],
                "customfield_10000": "some custom value"
            }
        }))
        .send()
        .await
        .expect("create with extra fields failed");

    assert_eq!(
        create_resp.status(),
        201,
        "EC-001: extra fields must be silently ignored; create must succeed with 201"
    );
    let body: serde_json::Value = create_resp.json().await.expect("body failed");
    assert!(
        body["key"].as_str().unwrap_or("").starts_with("EXTRA-"),
        "EC-001: key must start with EXTRA-"
    );

    clone.stop().await.expect("stop failed");
}

// ---------------------------------------------------------------------------
// EC-003 — Two creates for same project get incremented keys
// ---------------------------------------------------------------------------

/// EC-003: Given two POST /rest/api/3/issue calls with the same project key,
/// Then the second call gets issueKey with the next counter (no conflict).
#[tokio::test]
async fn test_ec003_sequential_creates_get_incremented_keys() {
    let mut clone = JiraClone::new().expect("JiraClone::new failed");
    clone.start().await.expect("start failed");
    let base = clone.base_url();
    let client = reqwest::Client::new();

    let resp1 = client
        .post(format!("{base}/rest/api/3/issue"))
        .header("Authorization", basic_auth_header())
        .json(&serde_json::json!({
            "fields": {
                "project": {"key": "SEQTEST"},
                "issuetype": {"name": "Task"},
                "summary": "first issue"
            }
        }))
        .send()
        .await
        .expect("first create failed");
    assert_eq!(resp1.status(), 201);
    let body1: serde_json::Value = resp1.json().await.expect("body1 failed");
    let key1 = body1["key"].as_str().expect("key1 missing").to_owned();
    assert_eq!(
        key1, "SEQTEST-1000",
        "EC-003: first issue must be SEQTEST-1000"
    );

    let resp2 = client
        .post(format!("{base}/rest/api/3/issue"))
        .header("Authorization", basic_auth_header())
        .json(&serde_json::json!({
            "fields": {
                "project": {"key": "SEQTEST"},
                "issuetype": {"name": "Bug"},
                "summary": "second issue"
            }
        }))
        .send()
        .await
        .expect("second create failed");
    assert_eq!(resp2.status(), 201);
    let body2: serde_json::Value = resp2.json().await.expect("body2 failed");
    let key2 = body2["key"].as_str().expect("key2 missing").to_owned();
    assert_eq!(
        key2, "SEQTEST-1001",
        "EC-003: second issue must be SEQTEST-1001"
    );

    // Both issues must be retrievable independently.
    let get1 = client
        .get(format!("{base}/rest/api/3/issue/{key1}"))
        .header("Authorization", basic_auth_header())
        .send()
        .await
        .expect("get issue1 failed");
    assert_eq!(
        get1.status(),
        200,
        "EC-003: first issue must still be retrievable"
    );

    let get2 = client
        .get(format!("{base}/rest/api/3/issue/{key2}"))
        .header("Authorization", basic_auth_header())
        .send()
        .await
        .expect("get issue2 failed");
    assert_eq!(
        get2.status(),
        200,
        "EC-003: second issue must be retrievable"
    );

    clone.stop().await.expect("stop failed");
}

// ---------------------------------------------------------------------------
// EC-004 — Comment on Done issue: 201 (Jira permits comments on closed issues)
// ---------------------------------------------------------------------------

/// EC-004: Given a Done issue, When POST /rest/api/3/issue/{key}/comment is called,
/// Then the response is 201 (Jira permits comments on closed issues).
#[tokio::test]
async fn test_ec004_comment_on_done_issue_returns_201() {
    let mut clone = JiraClone::new().expect("JiraClone::new failed");
    clone.start().await.expect("start failed");
    let base = clone.base_url();
    let client = reqwest::Client::new();

    // Create issue and transition it to Done via Open → Done (id "31").
    let create_resp = client
        .post(format!("{base}/rest/api/3/issue"))
        .header("Authorization", basic_auth_header())
        .json(&serde_json::json!({
            "fields": {
                "project": {"key": "CLDONE"},
                "issuetype": {"name": "Task"},
                "summary": "issue to be closed then commented"
            }
        }))
        .send()
        .await
        .expect("create failed");
    assert_eq!(create_resp.status(), 201);
    let create_body: serde_json::Value = create_resp.json().await.expect("body failed");
    let key = create_body["key"].as_str().expect("key missing").to_owned();

    // Transition to Done directly via id "31".
    let trans_resp = client
        .post(format!("{base}/rest/api/3/issue/{key}/transitions"))
        .header("Authorization", basic_auth_header())
        .json(&serde_json::json!({"transition": {"id": "31"}}))
        .send()
        .await
        .expect("transition failed");
    assert_eq!(
        trans_resp.status(),
        204,
        "EC-004: transition to Done must return 204"
    );

    // Verify it's Done.
    let get_resp = client
        .get(format!("{base}/rest/api/3/issue/{key}"))
        .header("Authorization", basic_auth_header())
        .send()
        .await
        .expect("get failed");
    let get_body: serde_json::Value = get_resp.json().await.expect("get body failed");
    assert_eq!(
        get_body["fields"]["status"]["name"], "Done",
        "EC-004: issue must be Done"
    );

    // Now add a comment to the Done issue — must succeed with 201.
    let comment_resp = client
        .post(format!("{base}/rest/api/3/issue/{key}/comment"))
        .header("Authorization", basic_auth_header())
        .json(&serde_json::json!({"body": {"type": "doc", "content": [{"type": "text", "text": "comment on closed issue"}]}}))
        .send()
        .await
        .expect("comment on Done issue failed");
    assert_eq!(
        comment_resp.status(),
        201,
        "EC-004: comment on Done issue must return 201 (Jira permits comments on closed issues)"
    );

    // comment_count must be incremented.
    let get_resp2 = client
        .get(format!("{base}/rest/api/3/issue/{key}"))
        .header("Authorization", basic_auth_header())
        .send()
        .await
        .expect("get after comment failed");
    let get_body2: serde_json::Value = get_resp2.json().await.expect("body2 failed");
    assert_eq!(
        get_body2["fields"]["comment"]["total"], 1,
        "EC-004: comment_count must be 1 after commenting on Done issue"
    );

    clone.stop().await.expect("stop failed");
}

// ---------------------------------------------------------------------------
// Subtle response shape fidelity — GET issue "self" field + status.id
// ---------------------------------------------------------------------------

/// Jira REST API v3 contract: GET /rest/api/3/issue/{key} must return
/// a "self" field (absolute URL to the resource) AND fields.status.id
/// as a numeric string.
#[tokio::test]
async fn test_get_issue_response_has_self_field_and_status_id() {
    let mut clone = JiraClone::new().expect("JiraClone::new failed");
    clone.start().await.expect("start failed");
    let base = clone.base_url();
    let client = reqwest::Client::new();

    let create_resp = client
        .post(format!("{base}/rest/api/3/issue"))
        .header("Authorization", basic_auth_header())
        .json(&serde_json::json!({
            "fields": {
                "project": {"key": "SHAPE"},
                "issuetype": {"name": "Incident"},
                "summary": "shape fidelity test"
            }
        }))
        .send()
        .await
        .expect("create failed");
    assert_eq!(create_resp.status(), 201);
    let create_body: serde_json::Value = create_resp.json().await.expect("create body failed");
    let key = create_body["key"].as_str().expect("key missing").to_owned();

    // "self" must be present in create response too.
    let self_link = create_body["self"].as_str().unwrap_or("");
    assert!(
        !self_link.is_empty(),
        "POST create response must include 'self' field"
    );
    assert!(
        self_link.contains(&key),
        "POST create response 'self' must contain the issue key"
    );

    // GET issue response shape.
    let get_resp = client
        .get(format!("{base}/rest/api/3/issue/{key}"))
        .header("Authorization", basic_auth_header())
        .send()
        .await
        .expect("get failed");
    assert_eq!(get_resp.status(), 200);
    let get_body: serde_json::Value = get_resp.json().await.expect("get body failed");

    // "self" must be present.
    let get_self = get_body["self"].as_str().unwrap_or("");
    assert!(
        !get_self.is_empty(),
        "GET issue response must include 'self' field per Jira REST v3 contract"
    );

    // fields.status.id must be a numeric string ("1" for Open).
    let status_id = get_body["fields"]["status"]["id"].as_str().unwrap_or("");
    assert_eq!(
        status_id, "1",
        "fields.status.id for Open must be '1' (numeric string, not integer)"
    );

    // fields.status.name must be "Open".
    let status_name = get_body["fields"]["status"]["name"].as_str().unwrap_or("");
    assert_eq!(
        status_name, "Open",
        "fields.status.name must be 'Open' for a new issue"
    );

    // fields.comment.total must be 0.
    assert_eq!(
        get_body["fields"]["comment"]["total"], 0,
        "fields.comment.total must be 0 for a new issue"
    );

    // "id" field must be present and non-empty.
    let id_field = get_body["id"].as_str().unwrap_or("");
    assert!(
        !id_field.is_empty(),
        "GET issue response must include 'id' field"
    );

    clone.stop().await.expect("stop failed");
}

// ---------------------------------------------------------------------------
// Auth edge cases: wrong scheme (Bearer) and invalid base64
// ---------------------------------------------------------------------------

/// Auth: Authorization: Bearer <token> returns 401 (not 403 — Jira uses 401).
#[tokio::test]
async fn test_bearer_scheme_returns_401() {
    let mut clone = JiraClone::new().expect("JiraClone::new failed");
    clone.start().await.expect("start failed");
    let base = clone.base_url();
    let client = reqwest::Client::new();

    let resp = client
        .post(format!("{base}/rest/api/3/issue"))
        .header("Authorization", "Bearer some-token-value")
        .json(&serde_json::json!({
            "fields": {
                "project": {"key": "AUTH"},
                "issuetype": {"name": "Task"},
                "summary": "bearer auth test"
            }
        }))
        .send()
        .await
        .expect("request failed");
    assert_eq!(
        resp.status(),
        401,
        "Bearer scheme must return 401 (Jira requires Basic auth)"
    );
    let body: serde_json::Value = resp.json().await.expect("body failed");
    assert_eq!(
        body["errorMessages"][0], "Basic authentication required",
        "Error message must be 'Basic authentication required'"
    );

    clone.stop().await.expect("stop failed");
}

/// Auth: Authorization: Basic <invalid-base64> returns 401.
#[tokio::test]
async fn test_invalid_base64_in_basic_auth_returns_401() {
    let mut clone = JiraClone::new().expect("JiraClone::new failed");
    clone.start().await.expect("start failed");
    let base = clone.base_url();
    let client = reqwest::Client::new();

    let resp = client
        .get(format!("{base}/rest/api/3/issue/PROJ-1000"))
        .header("Authorization", "Basic !!!not-valid-base64!!!")
        .send()
        .await
        .expect("request failed");
    assert_eq!(
        resp.status(),
        401,
        "Invalid base64 in Basic auth must return 401"
    );
    let body: serde_json::Value = resp.json().await.expect("body failed");
    assert_eq!(
        body["errorMessages"][0], "Basic authentication required",
        "Error message must match exactly"
    );

    clone.stop().await.expect("stop failed");
}

// ---------------------------------------------------------------------------
// Transition 404 when issue doesn't exist
// ---------------------------------------------------------------------------

/// POST /rest/api/3/issue/{key}/transitions on non-existent issue returns 404.
#[tokio::test]
async fn test_execute_transition_on_missing_issue_returns_404() {
    let mut clone = JiraClone::new().expect("JiraClone::new failed");
    clone.start().await.expect("start failed");
    let base = clone.base_url();
    let client = reqwest::Client::new();

    let resp = client
        .post(format!("{base}/rest/api/3/issue/GHOST-9999/transitions"))
        .header("Authorization", basic_auth_header())
        .json(&serde_json::json!({"transition": {"id": "11"}}))
        .send()
        .await
        .expect("request failed");
    assert_eq!(
        resp.status(),
        404,
        "POST transitions on non-existent issue must return 404"
    );
    let body: serde_json::Value = resp.json().await.expect("body failed");
    assert_eq!(
        body["errorMessages"][0], "Issue does not exist",
        "Error message must be 'Issue does not exist'"
    );

    clone.stop().await.expect("stop failed");
}

/// GET /rest/api/3/issue/{key}/transitions on non-existent issue returns 404.
#[tokio::test]
async fn test_list_transitions_on_missing_issue_returns_404() {
    let mut clone = JiraClone::new().expect("JiraClone::new failed");
    clone.start().await.expect("start failed");
    let base = clone.base_url();
    let client = reqwest::Client::new();

    let resp = client
        .get(format!("{base}/rest/api/3/issue/GHOST-9999/transitions"))
        .header("Authorization", basic_auth_header())
        .send()
        .await
        .expect("request failed");
    assert_eq!(
        resp.status(),
        404,
        "GET transitions on non-existent issue must return 404"
    );
    let body: serde_json::Value = resp.json().await.expect("body failed");
    assert_eq!(
        body["errorMessages"][0], "Issue does not exist",
        "Error message must be 'Issue does not exist'"
    );

    clone.stop().await.expect("stop failed");
}

// ---------------------------------------------------------------------------
// Missing required fields: issuetype absent + summary absent
// ---------------------------------------------------------------------------

/// Missing issuetype entirely (vs. unknown issuetype) returns 400 with
/// errors.issuetype present.
#[tokio::test]
async fn test_missing_issuetype_entirely_returns_400() {
    let mut clone = JiraClone::new().expect("JiraClone::new failed");
    clone.start().await.expect("start failed");
    let base = clone.base_url();
    let client = reqwest::Client::new();

    let resp = client
        .post(format!("{base}/rest/api/3/issue"))
        .header("Authorization", basic_auth_header())
        .json(&serde_json::json!({
            "fields": {
                "project": {"key": "MISS"},
                "summary": "missing issuetype"
            }
        }))
        .send()
        .await
        .expect("request failed");
    assert_eq!(resp.status(), 400, "Missing issuetype must return 400");
    let body: serde_json::Value = resp.json().await.expect("body failed");
    // errors.issuetype must be present (either "required" or "unknown")
    assert!(
        !body["errors"]["issuetype"].is_null(),
        "errors.issuetype must be present when issuetype is missing"
    );

    clone.stop().await.expect("stop failed");
}

/// Missing summary returns 400.
#[tokio::test]
async fn test_missing_summary_returns_400() {
    let mut clone = JiraClone::new().expect("JiraClone::new failed");
    clone.start().await.expect("start failed");
    let base = clone.base_url();
    let client = reqwest::Client::new();

    let resp = client
        .post(format!("{base}/rest/api/3/issue"))
        .header("Authorization", basic_auth_header())
        .json(&serde_json::json!({
            "fields": {
                "project": {"key": "MISS"},
                "issuetype": {"name": "Task"}
            }
        }))
        .send()
        .await
        .expect("request failed");
    assert_eq!(resp.status(), 400, "Missing summary must return 400");
    let body: serde_json::Value = resp.json().await.expect("body failed");
    assert_eq!(
        body["errors"]["summary"], "required",
        "errors.summary must be 'required'"
    );

    clone.stop().await.expect("stop failed");
}

// ---------------------------------------------------------------------------
// Open → Done direct transition (id "31")
// ---------------------------------------------------------------------------

/// Open → Done direct transition via id "31" returns 204 and sets status "Done".
#[tokio::test]
async fn test_open_to_done_direct_transition_id_31() {
    let mut clone = JiraClone::new().expect("JiraClone::new failed");
    clone.start().await.expect("start failed");
    let base = clone.base_url();
    let client = reqwest::Client::new();

    let create_resp = client
        .post(format!("{base}/rest/api/3/issue"))
        .header("Authorization", basic_auth_header())
        .json(&serde_json::json!({
            "fields": {
                "project": {"key": "DIRECT"},
                "issuetype": {"name": "Story"},
                "summary": "direct close test"
            }
        }))
        .send()
        .await
        .expect("create failed");
    assert_eq!(create_resp.status(), 201);
    let create_body: serde_json::Value = create_resp.json().await.expect("body failed");
    let key = create_body["key"].as_str().expect("key missing").to_owned();

    // Transition to Done directly via id "31" (Open → Done, no InProgress stop).
    let trans_resp = client
        .post(format!("{base}/rest/api/3/issue/{key}/transitions"))
        .header("Authorization", basic_auth_header())
        .json(&serde_json::json!({"transition": {"id": "31"}}))
        .send()
        .await
        .expect("transition failed");
    assert_eq!(
        trans_resp.status(),
        204,
        "Open → Done direct transition (id 31) must return 204"
    );

    // Verify status.
    let get_resp = client
        .get(format!("{base}/rest/api/3/issue/{key}"))
        .header("Authorization", basic_auth_header())
        .send()
        .await
        .expect("get failed");
    let get_body: serde_json::Value = get_resp.json().await.expect("get body failed");
    assert_eq!(
        get_body["fields"]["status"]["name"], "Done",
        "Status must be 'Done' after direct Open → Done transition"
    );
    assert_eq!(
        get_body["fields"]["status"]["id"], "6",
        "fields.status.id for Done must be '6'"
    );

    clone.stop().await.expect("stop failed");
}

// ---------------------------------------------------------------------------
// InProgress → Done transition (id "21") — dedicated focused test
// ---------------------------------------------------------------------------

/// InProgress → Done transition via id "21": returns 204, sets status "Done",
/// and GET transitions returns empty list (EC-002 corollary).
#[tokio::test]
async fn test_inprogress_to_done_transition_id_21() {
    let mut clone = JiraClone::new().expect("JiraClone::new failed");
    clone.start().await.expect("start failed");
    let base = clone.base_url();
    let client = reqwest::Client::new();

    // Create and advance to InProgress.
    let create_resp = client
        .post(format!("{base}/rest/api/3/issue"))
        .header("Authorization", basic_auth_header())
        .json(&serde_json::json!({
            "fields": {
                "project": {"key": "PROG"},
                "issuetype": {"name": "Epic"},
                "summary": "inprogress to done"
            }
        }))
        .send()
        .await
        .expect("create failed");
    assert_eq!(create_resp.status(), 201);
    let create_body: serde_json::Value = create_resp.json().await.expect("body failed");
    let key = create_body["key"].as_str().expect("key missing").to_owned();

    // Transition to InProgress (id "11").
    let t1 = client
        .post(format!("{base}/rest/api/3/issue/{key}/transitions"))
        .header("Authorization", basic_auth_header())
        .json(&serde_json::json!({"transition": {"id": "11"}}))
        .send()
        .await
        .expect("transition 11 failed");
    assert_eq!(t1.status(), 204);

    // Verify InProgress transitions list: only id "21" must be available.
    let trans_list = client
        .get(format!("{base}/rest/api/3/issue/{key}/transitions"))
        .header("Authorization", basic_auth_header())
        .send()
        .await
        .expect("list transitions failed");
    assert_eq!(trans_list.status(), 200);
    let trans_body: serde_json::Value = trans_list.json().await.expect("trans body failed");
    let ids: Vec<&str> = trans_body["transitions"]
        .as_array()
        .expect("transitions array missing")
        .iter()
        .filter_map(|t| t["id"].as_str())
        .collect();
    assert_eq!(
        ids,
        vec!["21"],
        "InProgress must only offer transition id '21'"
    );
    let names: Vec<&str> = trans_body["transitions"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|t| t["name"].as_str())
        .collect();
    assert_eq!(
        names,
        vec!["Done"],
        "Transition id '21' must be named 'Done'"
    );

    // Transition to Done (id "21").
    let t2 = client
        .post(format!("{base}/rest/api/3/issue/{key}/transitions"))
        .header("Authorization", basic_auth_header())
        .json(&serde_json::json!({"transition": {"id": "21"}}))
        .send()
        .await
        .expect("transition 21 failed");
    assert_eq!(
        t2.status(),
        204,
        "InProgress → Done transition (id 21) must return 204"
    );

    // Verify status.
    let get_resp = client
        .get(format!("{base}/rest/api/3/issue/{key}"))
        .header("Authorization", basic_auth_header())
        .send()
        .await
        .expect("get failed");
    let get_body: serde_json::Value = get_resp.json().await.expect("get body failed");
    assert_eq!(
        get_body["fields"]["status"]["name"], "Done",
        "Status must be 'Done' after transition 21"
    );

    clone.stop().await.expect("stop failed");
}

// ---------------------------------------------------------------------------
// Transition names in Open transitions list
// ---------------------------------------------------------------------------

/// AC-3 corollary: Open transitions list names must be exactly
/// "Start Progress" (id "11") and "Close" (id "31") per story spec.
#[tokio::test]
async fn test_open_transition_names_are_start_progress_and_close() {
    let mut clone = JiraClone::new().expect("JiraClone::new failed");
    clone.start().await.expect("start failed");
    let base = clone.base_url();
    let client = reqwest::Client::new();

    let create_resp = client
        .post(format!("{base}/rest/api/3/issue"))
        .header("Authorization", basic_auth_header())
        .json(&serde_json::json!({
            "fields": {
                "project": {"key": "NAMES"},
                "issuetype": {"name": "Task"},
                "summary": "transition names test"
            }
        }))
        .send()
        .await
        .expect("create failed");
    assert_eq!(create_resp.status(), 201);
    let key = create_resp
        .json::<serde_json::Value>()
        .await
        .expect("body failed")["key"]
        .as_str()
        .expect("key missing")
        .to_owned();

    let trans_resp = client
        .get(format!("{base}/rest/api/3/issue/{key}/transitions"))
        .header("Authorization", basic_auth_header())
        .send()
        .await
        .expect("list transitions failed");
    assert_eq!(trans_resp.status(), 200);
    let trans_body: serde_json::Value = trans_resp.json().await.expect("trans body failed");
    let transitions = trans_body["transitions"].as_array().expect("array missing");

    // Build id→name map.
    let name_for: std::collections::HashMap<&str, &str> = transitions
        .iter()
        .filter_map(|t| {
            let id = t["id"].as_str()?;
            let name = t["name"].as_str()?;
            Some((id, name))
        })
        .collect();

    assert_eq!(
        name_for.get("11").copied(),
        Some("Start Progress"),
        "Transition id '11' must be named 'Start Progress'"
    );
    assert_eq!(
        name_for.get("31").copied(),
        Some("Close"),
        "Transition id '31' must be named 'Close'"
    );

    clone.stop().await.expect("stop failed");
}

// ---------------------------------------------------------------------------
// DTU internal endpoints accessible without auth + configure token guard
// ---------------------------------------------------------------------------

/// GET /dtu/health returns 200 without any auth header.
#[tokio::test]
async fn test_dtu_health_returns_200_without_auth() {
    let mut clone = JiraClone::new().expect("JiraClone::new failed");
    clone.start().await.expect("start failed");
    let base = clone.base_url();
    let client = reqwest::Client::new();

    let resp = client
        .get(format!("{base}/dtu/health"))
        .send()
        .await
        .expect("request failed");
    assert_eq!(
        resp.status(),
        200,
        "GET /dtu/health must return 200 without any auth header"
    );
    let body: serde_json::Value = resp.json().await.expect("body failed");
    assert_eq!(
        body["status"], "ok",
        "Health body must be {{\"status\": \"ok\"}}"
    );

    clone.stop().await.expect("stop failed");
}

/// POST /dtu/configure without X-Admin-Token header returns 401.
#[tokio::test]
async fn test_dtu_configure_without_admin_token_returns_401() {
    let mut clone = JiraClone::new().expect("JiraClone::new failed");
    clone.start().await.expect("start failed");
    let base = clone.base_url();
    let client = reqwest::Client::new();

    let resp = client
        .post(format!("{base}/dtu/configure"))
        .json(&serde_json::json!({"failure_mode": "none"}))
        .send()
        .await
        .expect("request failed");
    assert_eq!(
        resp.status(),
        401,
        "POST /dtu/configure without X-Admin-Token must return 401"
    );

    clone.stop().await.expect("stop failed");
}

/// POST /dtu/configure with wrong X-Admin-Token returns 401.
#[tokio::test]
async fn test_dtu_configure_with_wrong_admin_token_returns_401() {
    let mut clone = JiraClone::new().expect("JiraClone::new failed");
    clone.start().await.expect("start failed");
    let base = clone.base_url();
    let client = reqwest::Client::new();

    let resp = client
        .post(format!("{base}/dtu/configure"))
        .header("X-Admin-Token", "definitely-wrong-token")
        .json(&serde_json::json!({"failure_mode": "none"}))
        .send()
        .await
        .expect("request failed");
    assert_eq!(
        resp.status(),
        401,
        "POST /dtu/configure with wrong token must return 401"
    );

    clone.stop().await.expect("stop failed");
}

// ---------------------------------------------------------------------------
// GET /dtu/issues returns comment_count field
// ---------------------------------------------------------------------------

/// GET /dtu/issues response includes comment_count field for each issue.
#[tokio::test]
async fn test_dtu_issues_response_includes_comment_count() {
    let mut clone = JiraClone::new().expect("JiraClone::new failed");
    clone.start().await.expect("start failed");
    let base = clone.base_url();
    let client = reqwest::Client::new();

    let create_resp = client
        .post(format!("{base}/rest/api/3/issue"))
        .header("Authorization", basic_auth_header())
        .json(&serde_json::json!({
            "fields": {
                "project": {"key": "DTUI"},
                "issuetype": {"name": "Task"},
                "summary": "dtu issues test"
            }
        }))
        .send()
        .await
        .expect("create failed");
    assert_eq!(create_resp.status(), 201);
    let key = create_resp
        .json::<serde_json::Value>()
        .await
        .expect("body failed")["key"]
        .as_str()
        .expect("key missing")
        .to_owned();

    let dtu_resp = client
        .get(format!("{base}/dtu/issues"))
        .send()
        .await
        .expect("request failed");
    assert_eq!(dtu_resp.status(), 200);
    let dtu_body: serde_json::Value = dtu_resp.json().await.expect("body failed");
    let issues = dtu_body["issues"].as_array().expect("issues array missing");
    let found = issues
        .iter()
        .find(|i| i["key"] == key)
        .expect("issue not found");
    assert_eq!(
        found["comment_count"], 0,
        "GET /dtu/issues must include comment_count field (0 for new issue)"
    );
    assert!(
        found["summary"].is_string(),
        "GET /dtu/issues must include summary field"
    );

    clone.stop().await.expect("stop failed");
}

// ---------------------------------------------------------------------------
// add_comment 404 on missing issue
// ---------------------------------------------------------------------------

/// POST /rest/api/3/issue/{key}/comment on non-existent issue returns 404.
#[tokio::test]
async fn test_add_comment_on_missing_issue_returns_404() {
    let mut clone = JiraClone::new().expect("JiraClone::new failed");
    clone.start().await.expect("start failed");
    let base = clone.base_url();
    let client = reqwest::Client::new();

    let resp = client
        .post(format!("{base}/rest/api/3/issue/GHOST-0000/comment"))
        .header("Authorization", basic_auth_header())
        .json(&serde_json::json!({"body": {"type": "doc", "content": []}}))
        .send()
        .await
        .expect("request failed");
    assert_eq!(
        resp.status(),
        404,
        "POST comment on non-existent issue must return 404"
    );
    let body: serde_json::Value = resp.json().await.expect("body failed");
    assert_eq!(
        body["errorMessages"][0], "Issue does not exist",
        "Error message must be 'Issue does not exist'"
    );

    clone.stop().await.expect("stop failed");
}

// ---------------------------------------------------------------------------
// add_comment response shape
// ---------------------------------------------------------------------------

/// POST /rest/api/3/issue/{key}/comment response must have id, self, and created fields.
#[tokio::test]
async fn test_add_comment_response_has_id_self_created() {
    let mut clone = JiraClone::new().expect("JiraClone::new failed");
    clone.start().await.expect("start failed");
    let base = clone.base_url();
    let client = reqwest::Client::new();

    let create_resp = client
        .post(format!("{base}/rest/api/3/issue"))
        .header("Authorization", basic_auth_header())
        .json(&serde_json::json!({
            "fields": {
                "project": {"key": "CMSHP"},
                "issuetype": {"name": "Task"},
                "summary": "comment shape test"
            }
        }))
        .send()
        .await
        .expect("create failed");
    assert_eq!(create_resp.status(), 201);
    let key = create_resp
        .json::<serde_json::Value>()
        .await
        .expect("body failed")["key"]
        .as_str()
        .expect("key missing")
        .to_owned();

    let comment_resp = client
        .post(format!("{base}/rest/api/3/issue/{key}/comment"))
        .header("Authorization", basic_auth_header())
        .json(&serde_json::json!({"body": {"type": "doc", "content": []}}))
        .send()
        .await
        .expect("request failed");
    assert_eq!(comment_resp.status(), 201);
    let body: serde_json::Value = comment_resp.json().await.expect("body failed");

    assert!(
        body["id"].is_string() && !body["id"].as_str().unwrap_or("").is_empty(),
        "Comment response must have non-empty 'id' field"
    );
    assert!(
        body["self"].is_string() && !body["self"].as_str().unwrap_or("").is_empty(),
        "Comment response must have non-empty 'self' field"
    );
    assert!(
        body["created"].is_string() && !body["created"].as_str().unwrap_or("").is_empty(),
        "Comment response must have non-empty 'created' field"
    );

    clone.stop().await.expect("stop failed");
}

// ---------------------------------------------------------------------------
// Invalid transition from InProgress (attempting id "11" again after Start Progress)
// ---------------------------------------------------------------------------

/// InProgress → InProgress (transition id "11") is invalid: must return 400.
#[tokio::test]
async fn test_inprogress_to_inprogress_is_invalid_transition() {
    let mut clone = JiraClone::new().expect("JiraClone::new failed");
    clone.start().await.expect("start failed");
    let base = clone.base_url();
    let client = reqwest::Client::new();

    let create_resp = client
        .post(format!("{base}/rest/api/3/issue"))
        .header("Authorization", basic_auth_header())
        .json(&serde_json::json!({
            "fields": {
                "project": {"key": "INVTR"},
                "issuetype": {"name": "Task"},
                "summary": "invalid re-transition test"
            }
        }))
        .send()
        .await
        .expect("create failed");
    assert_eq!(create_resp.status(), 201);
    let key = create_resp
        .json::<serde_json::Value>()
        .await
        .expect("body failed")["key"]
        .as_str()
        .expect("key missing")
        .to_owned();

    // First, advance to InProgress via id "11".
    let t1 = client
        .post(format!("{base}/rest/api/3/issue/{key}/transitions"))
        .header("Authorization", basic_auth_header())
        .json(&serde_json::json!({"transition": {"id": "11"}}))
        .send()
        .await
        .expect("first transition failed");
    assert_eq!(t1.status(), 204);

    // Now try id "11" again from InProgress — must be rejected.
    let t2 = client
        .post(format!("{base}/rest/api/3/issue/{key}/transitions"))
        .header("Authorization", basic_auth_header())
        .json(&serde_json::json!({"transition": {"id": "11"}}))
        .send()
        .await
        .expect("second transition failed");
    assert_eq!(
        t2.status(),
        400,
        "Re-applying transition id '11' from InProgress must return 400"
    );
    let body: serde_json::Value = t2.json().await.expect("body failed");
    assert_eq!(
        body["errorMessages"][0], "Invalid transition id",
        "Error message must be 'Invalid transition id'"
    );

    clone.stop().await.expect("stop failed");
}

// ---------------------------------------------------------------------------
// All valid issue types accepted (Epic, Story, Bug, Incident)
// ---------------------------------------------------------------------------

/// All 5 valid issue types must be accepted: Task, Bug, Story, Epic, Incident.
#[tokio::test]
async fn test_all_valid_issue_types_are_accepted() {
    let mut clone = JiraClone::new().expect("JiraClone::new failed");
    clone.start().await.expect("start failed");
    let base = clone.base_url();
    let client = reqwest::Client::new();

    for issue_type in ["Task", "Bug", "Story", "Epic", "Incident"] {
        let resp = client
            .post(format!("{base}/rest/api/3/issue"))
            .header("Authorization", basic_auth_header())
            .json(&serde_json::json!({
                "fields": {
                    "project": {"key": "TYPES"},
                    "issuetype": {"name": issue_type},
                    "summary": format!("{issue_type} creation test")
                }
            }))
            .send()
            .await
            .expect("create failed");
        assert_eq!(
            resp.status(),
            201,
            "Issue type '{issue_type}' must be accepted with 201"
        );
        let body: serde_json::Value = resp.json().await.expect("body failed");
        let key = body["key"].as_str().unwrap_or("");
        assert!(
            key.starts_with("TYPES-"),
            "Issue type '{issue_type}': key must start with TYPES-"
        );
    }

    clone.stop().await.expect("stop failed");
}
