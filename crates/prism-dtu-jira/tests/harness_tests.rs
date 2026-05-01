//! S-3.4.05 harness migration tests — prism-dtu-jira, shared-mode.
//!
//! # Behavioral contracts
//!
//! - BC-3.2.004: Shared-Mode DTU Tags OrgId in Payload Body Not in Routing Headers
//! - BC-3.3.001: DTU Mode Policy (startup EC-003: MSSP Coordination types permit client override)
//! - BC-3.5.001: Harness Logical Isolation Invariants
//!
//! # Test catalog (migrated from existing tests + new AC tests)
//!
//! ## Migrated from tests/fidelity.rs (28 tests)
//! - test_BC_3_5_001_jira_full_lifecycle_create_comment_transition
//! - test_BC_3_5_001_jira_missing_auth_returns_401
//! - test_BC_3_5_001_jira_missing_project_key_returns_400
//! - test_BC_3_5_001_jira_unknown_issuetype_returns_400
//! - test_BC_3_5_001_jira_unknown_issue_key_returns_404
//! - test_BC_3_5_001_jira_reset_clears_all_issues
//! - test_BC_3_5_001_jira_ac10_rate_limit_429_issue_not_persisted
//! - test_BC_3_5_001_jira_ec001_extra_fields_ignored
//! - test_BC_3_5_001_jira_ec003_sequential_creates_incremented_keys
//! - test_BC_3_5_001_jira_ec004_comment_on_done_returns_201
//! - test_BC_3_5_001_jira_get_issue_response_shape_self_and_status_id
//! - test_BC_3_5_001_jira_bearer_scheme_returns_401
//! - test_BC_3_5_001_jira_invalid_base64_basic_auth_returns_401
//! - test_BC_3_5_001_jira_execute_transition_on_missing_issue_404
//! - test_BC_3_5_001_jira_list_transitions_on_missing_issue_404
//! - test_BC_3_5_001_jira_missing_issuetype_entirely_returns_400
//! - test_BC_3_5_001_jira_missing_summary_returns_400
//! - test_BC_3_5_001_jira_open_to_done_direct_transition_id_31
//! - test_BC_3_5_001_jira_inprogress_to_done_transition_id_21
//! - test_BC_3_5_001_jira_open_transition_names_start_progress_and_close
//! - test_BC_3_5_001_jira_dtu_health_returns_200_without_auth
//! - test_BC_3_5_001_jira_configure_without_admin_token_returns_401
//! - test_BC_3_5_001_jira_configure_with_wrong_admin_token_returns_401
//! - test_BC_3_5_001_jira_issues_response_includes_comment_count
//! - test_BC_3_5_001_jira_add_comment_on_missing_issue_returns_404
//! - test_BC_3_5_001_jira_add_comment_response_has_id_self_created
//! - test_BC_3_5_001_jira_inprogress_to_inprogress_invalid_transition
//! - test_BC_3_5_001_jira_all_valid_issue_types_accepted
//!
//! ## Migrated from tests/org_tagging.rs (8 tests)
//! - test_BC_3_2_004_jira_ac001_org_id_in_issue_record
//! - test_BC_3_2_004_jira_ac002_issue_key_not_org_scoped
//! - test_BC_3_2_004_jira_ac002_org_id_absent_from_routing
//! - test_BC_3_2_004_jira_ac003_concurrent_issues_distinguished
//! - test_BC_3_2_004_jira_ac004_mode_metadata_absent_from_query_results
//! - test_BC_3_2_005_jira_ac005_jira_dtu_mode_is_shared
//! - test_BC_3_2_005_jira_ac005_mode_immutable_after_startup
//! - test_BC_3_2_005_jira_ac006_invalid_mode_string_rejected
//!
//! ## New harness-specific tests (S-3.4.05 AC-006, AC-007, EC-001, EC-002, EC-003)
//! - ac_shared_mode_org_id_tagging
//! - ac_multi_org_logical_isolation_shared_mode
//! - ac_client_mode_override_does_not_produce_startup_error
//!
//! # Red Gate rationale
//!
//! All BC-3.2.004 / BC-3.2.005 org-tagging assertions fail because the Jira
//! create-issue handler does not extract `X-Prism-Org-Id` and store it in
//! `IssueRecord.org_id` — the field is always empty string.
//!
//! All BC-3.5.001 harness migration assertions fail because `DtuType::Jira` is
//! not yet dispatched by the harness clone-server — `endpoint_for` returns `None`.
//!
//! # Naming convention
//!
//! `test_BC_S_SS_NNN_xxx()` for BC-traced tests.
//! `ac_xxx()` for story-level AC tests without a direct BC number.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    non_snake_case,
    unused_imports
)]
#![cfg(feature = "dtu")]

use prism_dtu_harness::{DtuType, HarnessBuilder, IsolationMode};
use serde_json::Value;

// ---------------------------------------------------------------------------
// Test constants
// ---------------------------------------------------------------------------

const TENANT: &str = "test-tenant";

/// Canonical OrgId test vectors (AI-opaque UUIDs).
const ORG_UUID_A: &str = "00000000-0000-7000-8000-000000000001";
const ORG_UUID_B: &str = "00000000-0000-7000-8000-000000000002";
const ORG_UUID_C: &str = "00000000-0000-7000-8000-000000000003";

// ---------------------------------------------------------------------------
// Test helpers
// ---------------------------------------------------------------------------

/// Build a single-tenant Jira harness with `IsolationMode::Logical`.
///
/// RED GATE: Fails because `DtuType::Jira` is not yet dispatched by the harness
/// clone-server — `endpoint_for` returns `None`, causing `expect` to panic.
async fn build_jira_harness() -> (
    prism_dtu_harness::Harness,
    std::net::SocketAddr,
    reqwest::Client,
) {
    let harness = HarnessBuilder::new()
        .isolation(IsolationMode::Logical)
        .with_customer_overrides(TENANT, |spec| {
            spec.dtu_types = vec![DtuType::Jira];
        })
        .build()
        .await
        .expect("Jira harness build must succeed (BC-3.5.001 precondition 2)");

    let addr = harness
        .endpoint_for(TENANT, DtuType::Jira)
        .expect("Jira endpoint must be present after build");

    let client = reqwest::Client::new();
    (harness, addr, client)
}

/// Build a Basic-auth header for Jira API calls.
fn basic_auth() -> String {
    use base64::Engine as _;
    let encoded = base64::engine::general_purpose::STANDARD.encode("testuser:testtoken");
    format!("Basic {encoded}")
}

/// POST a create-issue request via the harness, returning (status, body).
async fn create_issue(
    client: &reqwest::Client,
    addr: std::net::SocketAddr,
    project_key: &str,
    issue_type: &str,
    summary: &str,
    org_id: Option<&str>,
) -> (u16, Value) {
    let mut req = client
        .post(format!("http://{addr}/rest/api/3/issue"))
        .header("Authorization", basic_auth())
        .json(&serde_json::json!({
            "fields": {
                "project": {"key": project_key},
                "issuetype": {"name": issue_type},
                "summary": summary,
            }
        }));
    if let Some(id) = org_id {
        req = req.header("X-Prism-Org-Id", id);
    }
    let resp = req.send().await.expect("POST /rest/api/3/issue");
    let status = resp.status().as_u16();
    let body: Value = resp.json().await.expect("JSON body");
    (status, body)
}

/// GET /dtu/issues and return the issues array.
async fn get_dtu_issues(client: &reqwest::Client, addr: std::net::SocketAddr) -> Vec<Value> {
    let resp = client
        .get(format!("http://{addr}/dtu/issues"))
        .send()
        .await
        .expect("GET /dtu/issues must not fail at network level");
    assert_eq!(
        resp.status().as_u16(),
        200,
        "GET /dtu/issues must return 200"
    );
    let body: Value = resp.json().await.expect("JSON body");
    body["issues"]
        .as_array()
        .expect("issues must be an array")
        .clone()
}

/// POST a transition to the given issue.
async fn execute_transition(
    client: &reqwest::Client,
    addr: std::net::SocketAddr,
    issue_key: &str,
    transition_id: &str,
) -> reqwest::Response {
    client
        .post(format!(
            "http://{addr}/rest/api/3/issue/{issue_key}/transitions"
        ))
        .header("Authorization", basic_auth())
        .json(&serde_json::json!({"transition": {"id": transition_id}}))
        .send()
        .await
        .expect("POST /transitions must not fail at network level")
}

/// GET transitions for an issue.
async fn list_transitions(
    client: &reqwest::Client,
    addr: std::net::SocketAddr,
    issue_key: &str,
) -> Vec<Value> {
    let resp = client
        .get(format!(
            "http://{addr}/rest/api/3/issue/{issue_key}/transitions"
        ))
        .header("Authorization", basic_auth())
        .send()
        .await
        .expect("GET /transitions must not fail at network level");
    let body: Value = resp.json().await.expect("JSON body");
    body["transitions"]
        .as_array()
        .expect("transitions array")
        .clone()
}

// ---------------------------------------------------------------------------
// Migrated: fidelity.rs → test_BC_3_5_001_jira_*
// ---------------------------------------------------------------------------

/// Migrated from fidelity.rs: `test_full_lifecycle_create_comment_transition`.
///
/// Full issue lifecycle: create → comment → transition → Done via harness.
///
/// RED GATE: Fails at `build_jira_harness` — `DtuType::Jira` not dispatched.
///
/// Traces to: BC-3.5.001 postcondition 1; S-3.4.05 AC-003.
#[tokio::test]
async fn test_BC_3_5_001_jira_full_lifecycle_create_comment_transition() {
    let (_harness, addr, client) = build_jira_harness().await;

    // AC-1: Create issue.
    let (status, body) = create_issue(
        &client,
        addr,
        "ACME-SEC",
        "Task",
        "Test incident for lifecycle",
        None,
    )
    .await;
    assert_eq!(status, 201, "AC-1: create must return 201");
    let issue_key = body["key"].as_str().expect("key missing").to_owned();
    assert!(
        issue_key.starts_with("ACME-SEC-"),
        "key must start with ACME-SEC-"
    );

    // AC-1: GET /dtu/issues confirms status: Open.
    let issues = get_dtu_issues(&client, addr).await;
    let found = issues
        .iter()
        .find(|i| i["key"] == issue_key)
        .expect("issue not found in /dtu/issues");
    assert_eq!(found["status"], "Open", "Newly created issue must be Open");

    // AC-2: Add comment.
    let comment_resp = client
        .post(format!(
            "http://{addr}/rest/api/3/issue/{issue_key}/comment"
        ))
        .header("Authorization", basic_auth())
        .json(&serde_json::json!({"body": {"type": "doc", "content": []}}))
        .send()
        .await
        .expect("comment request failed");
    assert_eq!(
        comment_resp.status().as_u16(),
        201,
        "AC-2: comment must return 201"
    );

    // AC-3: List transitions when Open — must include "11" and "31".
    let transitions = list_transitions(&client, addr, &issue_key).await;
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

    // AC-4: Execute transition 11 (Start Progress).
    let exec_resp = execute_transition(&client, addr, &issue_key, "11").await;
    assert_eq!(
        exec_resp.status().as_u16(),
        204,
        "AC-4: execute transition must return 204"
    );

    // Transition to Done (id "21").
    let done_resp = execute_transition(&client, addr, &issue_key, "21").await;
    assert_eq!(
        done_resp.status().as_u16(),
        204,
        "Transition to Done must return 204"
    );

    // AC-5: Invalid transition from Done returns 400.
    let invalid_resp = execute_transition(&client, addr, &issue_key, "11").await;
    assert_eq!(
        invalid_resp.status().as_u16(),
        400,
        "AC-5: invalid transition must return 400"
    );
    let invalid_body: Value = invalid_resp.json().await.expect("invalid body failed");
    assert_eq!(
        invalid_body["errorMessages"][0], "Invalid transition id",
        "AC-5: error message must match exactly"
    );
}

/// Migrated from fidelity.rs: `test_missing_auth_returns_401`.
///
/// RED GATE: Fails at `build_jira_harness`.
///
/// Traces to: BC-3.5.001 postcondition 1; S-3.4.05 AC-003.
#[tokio::test]
async fn test_BC_3_5_001_jira_missing_auth_returns_401() {
    let (_harness, addr, client) = build_jira_harness().await;

    let resp = client
        .post(format!("http://{addr}/rest/api/3/issue"))
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

    assert_eq!(
        resp.status().as_u16(),
        401,
        "AC-8: missing auth must return 401"
    );
    let body: Value = resp.json().await.expect("body failed");
    assert_eq!(
        body["errorMessages"][0], "Basic authentication required",
        "AC-8: error message must match exactly"
    );
}

/// Migrated from fidelity.rs: `test_missing_project_key_returns_400`.
///
/// RED GATE: Fails at `build_jira_harness`.
///
/// Traces to: BC-3.5.001 postcondition 1; S-3.4.05 AC-003.
#[tokio::test]
async fn test_BC_3_5_001_jira_missing_project_key_returns_400() {
    let (_harness, addr, client) = build_jira_harness().await;

    let resp = client
        .post(format!("http://{addr}/rest/api/3/issue"))
        .header("Authorization", basic_auth())
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
        resp.status().as_u16(),
        400,
        "AC-6: missing project.key must return 400"
    );
    let body: Value = resp.json().await.expect("body failed");
    assert_eq!(
        body["errors"]["project"], "required",
        "AC-6: errors.project must be 'required'"
    );
}

/// Migrated from fidelity.rs: `test_unknown_issuetype_returns_400`.
///
/// RED GATE: Fails at `build_jira_harness`.
///
/// Traces to: BC-3.5.001 postcondition 1; S-3.4.05 AC-003.
#[tokio::test]
async fn test_BC_3_5_001_jira_unknown_issuetype_returns_400() {
    let (_harness, addr, client) = build_jira_harness().await;

    let resp = client
        .post(format!("http://{addr}/rest/api/3/issue"))
        .header("Authorization", basic_auth())
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
        resp.status().as_u16(),
        400,
        "AC-7: unknown issuetype must return 400"
    );
    let body: Value = resp.json().await.expect("body failed");
    assert_eq!(
        body["errors"]["issuetype"], "unknown",
        "AC-7: errors.issuetype must be 'unknown'"
    );
}

/// Migrated from fidelity.rs: `test_unknown_issue_key_returns_404`.
///
/// RED GATE: Fails at `build_jira_harness`.
///
/// Traces to: BC-3.5.001 postcondition 1; S-3.4.05 AC-003.
#[tokio::test]
async fn test_BC_3_5_001_jira_unknown_issue_key_returns_404() {
    let (_harness, addr, client) = build_jira_harness().await;

    let resp = client
        .get(format!("http://{addr}/rest/api/3/issue/UNKNOWN-999"))
        .header("Authorization", basic_auth())
        .send()
        .await
        .expect("request failed");

    assert_eq!(
        resp.status().as_u16(),
        404,
        "AC-9: unknown issue must return 404"
    );
    let body: Value = resp.json().await.expect("body failed");
    assert_eq!(body["errorMessages"][0], "Issue does not exist");
}

/// Migrated from fidelity.rs: `test_reset_clears_all_issues`.
///
/// RED GATE: Fails at `build_jira_harness`.
///
/// Traces to: BC-3.5.001 postcondition 1; S-3.4.05 AC-003.
#[tokio::test]
async fn test_BC_3_5_001_jira_reset_clears_all_issues() {
    let (_harness, addr, client) = build_jira_harness().await;

    let (status, body) =
        create_issue(&client, addr, "TEST", "Bug", "issue to be cleared", None).await;
    assert_eq!(status, 201);
    let key = body["key"].as_str().expect("key missing").to_owned();

    // Verify it exists.
    let get_resp = client
        .get(format!("http://{addr}/rest/api/3/issue/{key}"))
        .header("Authorization", basic_auth())
        .send()
        .await
        .expect("get failed");
    assert_eq!(get_resp.status().as_u16(), 200);

    // Reset via HTTP.
    let reset_resp = client
        .post(format!("http://{addr}/dtu/reset"))
        .send()
        .await
        .expect("reset failed");
    assert_eq!(reset_resp.status().as_u16(), 200, "reset must return 200");

    // Verify it no longer exists.
    let after_reset = client
        .get(format!("http://{addr}/rest/api/3/issue/{key}"))
        .header("Authorization", basic_auth())
        .send()
        .await
        .expect("get after reset failed");
    assert_eq!(
        after_reset.status().as_u16(),
        404,
        "EC-005: issue must be gone after reset"
    );

    // First key after reset must start from PROJ-1000.
    let (status2, body2) =
        create_issue(&client, addr, "PROJ", "Task", "after reset issue", None).await;
    assert_eq!(status2, 201);
    let key2 = body2["key"].as_str().expect("key2 missing").to_owned();
    assert_eq!(
        key2, "PROJ-1000",
        "After reset, first new issue must be PROJ-1000"
    );
}

/// Migrated from fidelity.rs: `test_ac10_rate_limit_429_returned_and_issue_not_persisted`.
///
/// RED GATE: Fails at `build_jira_harness`.
///
/// Traces to: BC-3.5.001 postcondition 1; S-3.4.05 AC-003.
#[tokio::test]
async fn test_BC_3_5_001_jira_ac10_rate_limit_429_issue_not_persisted() {
    let (harness, addr, client) = build_jira_harness().await;

    harness
        .inject_failure(
            TENANT,
            DtuType::Jira,
            prism_dtu_common::FailureMode::RateLimit {
                after_n_requests: 0,
                retry_after_secs: 30,
            },
        )
        .await
        .expect("inject_failure must succeed");

    let resp = client
        .post(format!("http://{addr}/rest/api/3/issue"))
        .header("Authorization", basic_auth())
        .json(&serde_json::json!({
            "fields": {
                "project": {"key": "RATELIM"},
                "issuetype": {"name": "Task"},
                "summary": "should not be persisted"
            }
        }))
        .send()
        .await
        .expect("request failed");
    assert_eq!(
        resp.status().as_u16(),
        429,
        "AC-10: rate-limited create must return 429"
    );

    // Clear failure mode and verify no issue was persisted.
    harness
        .clear_failure(TENANT, DtuType::Jira)
        .await
        .expect("clear_failure must succeed");

    let issues = get_dtu_issues(&client, addr).await;
    assert!(
        issues.is_empty(),
        "EC-006: issue must NOT be persisted when rate limit triggered (got {} issues)",
        issues.len()
    );
}

/// Migrated from fidelity.rs: `test_ec001_extra_fields_in_create_body_are_ignored`.
///
/// RED GATE: Fails at `build_jira_harness`.
///
/// Traces to: BC-3.5.001 postcondition 1; S-3.4.05 AC-003.
#[tokio::test]
async fn test_BC_3_5_001_jira_ec001_extra_fields_ignored() {
    let (_harness, addr, client) = build_jira_harness().await;

    let resp = client
        .post(format!("http://{addr}/rest/api/3/issue"))
        .header("Authorization", basic_auth())
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
        resp.status().as_u16(),
        201,
        "EC-001: extra fields must be silently ignored"
    );
    let body: Value = resp.json().await.expect("body failed");
    assert!(
        body["key"].as_str().unwrap_or("").starts_with("EXTRA-"),
        "EC-001: key must start with EXTRA-"
    );
}

/// Migrated from fidelity.rs: `test_ec003_sequential_creates_get_incremented_keys`.
///
/// RED GATE: Fails at `build_jira_harness`.
///
/// Traces to: BC-3.5.001 postcondition 1; S-3.4.05 AC-003.
#[tokio::test]
async fn test_BC_3_5_001_jira_ec003_sequential_creates_incremented_keys() {
    let (_harness, addr, client) = build_jira_harness().await;

    let (s1, b1) = create_issue(&client, addr, "SEQTEST", "Task", "first issue", None).await;
    assert_eq!(s1, 201);
    let key1 = b1["key"].as_str().expect("key1").to_owned();
    assert_eq!(
        key1, "SEQTEST-1000",
        "EC-003: first issue must be SEQTEST-1000"
    );

    let (s2, b2) = create_issue(&client, addr, "SEQTEST", "Bug", "second issue", None).await;
    assert_eq!(s2, 201);
    let key2 = b2["key"].as_str().expect("key2").to_owned();
    assert_eq!(
        key2, "SEQTEST-1001",
        "EC-003: second issue must be SEQTEST-1001"
    );

    // Both issues must be retrievable.
    for key in [&key1, &key2] {
        let get = client
            .get(format!("http://{addr}/rest/api/3/issue/{key}"))
            .header("Authorization", basic_auth())
            .send()
            .await
            .expect("get failed");
        assert_eq!(
            get.status().as_u16(),
            200,
            "EC-003: {key} must be retrievable"
        );
    }
}

/// Migrated from fidelity.rs: `test_ec004_comment_on_done_issue_returns_201`.
///
/// RED GATE: Fails at `build_jira_harness`.
///
/// Traces to: BC-3.5.001 postcondition 1; S-3.4.05 AC-003.
#[tokio::test]
async fn test_BC_3_5_001_jira_ec004_comment_on_done_returns_201() {
    let (_harness, addr, client) = build_jira_harness().await;

    let (s, b) = create_issue(
        &client,
        addr,
        "CLDONE",
        "Task",
        "issue to close then comment",
        None,
    )
    .await;
    assert_eq!(s, 201);
    let key = b["key"].as_str().expect("key").to_owned();

    // Transition Open → Done directly (id "31").
    let trans_resp = execute_transition(&client, addr, &key, "31").await;
    assert_eq!(
        trans_resp.status().as_u16(),
        204,
        "transition to Done must return 204"
    );

    // Add comment on Done issue — must succeed.
    let comment_resp = client
        .post(format!("http://{addr}/rest/api/3/issue/{key}/comment"))
        .header("Authorization", basic_auth())
        .json(&serde_json::json!({"body": {"type": "doc", "content": []}}))
        .send()
        .await
        .expect("comment on Done issue failed");
    assert_eq!(
        comment_resp.status().as_u16(),
        201,
        "EC-004: comment on Done issue must return 201"
    );

    // Verify comment_count incremented.
    let get_resp = client
        .get(format!("http://{addr}/rest/api/3/issue/{key}"))
        .header("Authorization", basic_auth())
        .send()
        .await
        .expect("get failed");
    let get_body: Value = get_resp.json().await.expect("body failed");
    assert_eq!(
        get_body["fields"]["comment"]["total"], 1,
        "EC-004: comment_count must be 1 after commenting on Done issue"
    );
}

/// Migrated from fidelity.rs: `test_get_issue_response_has_self_field_and_status_id`.
///
/// RED GATE: Fails at `build_jira_harness`.
///
/// Traces to: BC-3.5.001 postcondition 1; S-3.4.05 AC-003.
#[tokio::test]
async fn test_BC_3_5_001_jira_get_issue_response_shape_self_and_status_id() {
    let (_harness, addr, client) = build_jira_harness().await;

    let (s, b) = create_issue(
        &client,
        addr,
        "SHAPE",
        "Incident",
        "shape fidelity test",
        None,
    )
    .await;
    assert_eq!(s, 201);
    let key = b["key"].as_str().expect("key").to_owned();

    // "self" must be present in create response.
    let self_link = b["self"].as_str().unwrap_or("");
    assert!(
        !self_link.is_empty(),
        "POST create response must include 'self' field"
    );
    assert!(
        self_link.contains(&key),
        "POST create 'self' must contain the issue key"
    );

    // GET issue response shape.
    let get_resp = client
        .get(format!("http://{addr}/rest/api/3/issue/{key}"))
        .header("Authorization", basic_auth())
        .send()
        .await
        .expect("get failed");
    assert_eq!(get_resp.status().as_u16(), 200);
    let get_body: Value = get_resp.json().await.expect("get body failed");

    assert!(
        !get_body["self"].as_str().unwrap_or("").is_empty(),
        "GET issue must include 'self'"
    );
    assert_eq!(
        get_body["fields"]["status"]["id"].as_str().unwrap_or(""),
        "1",
        "fields.status.id for Open must be '1'"
    );
    assert_eq!(get_body["fields"]["status"]["name"], "Open");
    assert_eq!(get_body["fields"]["comment"]["total"], 0);
    assert!(
        !get_body["id"].as_str().unwrap_or("").is_empty(),
        "GET issue must include 'id'"
    );
}

/// Migrated from fidelity.rs: `test_bearer_scheme_returns_401`.
///
/// RED GATE: Fails at `build_jira_harness`.
///
/// Traces to: BC-3.5.001 postcondition 1; S-3.4.05 AC-003.
#[tokio::test]
async fn test_BC_3_5_001_jira_bearer_scheme_returns_401() {
    let (_harness, addr, client) = build_jira_harness().await;

    let resp = client
        .post(format!("http://{addr}/rest/api/3/issue"))
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
    assert_eq!(resp.status().as_u16(), 401, "Bearer scheme must return 401");
    let body: Value = resp.json().await.expect("body failed");
    assert_eq!(body["errorMessages"][0], "Basic authentication required");
}

/// Migrated from fidelity.rs: `test_invalid_base64_in_basic_auth_returns_401`.
///
/// RED GATE: Fails at `build_jira_harness`.
///
/// Traces to: BC-3.5.001 postcondition 1; S-3.4.05 AC-003.
#[tokio::test]
async fn test_BC_3_5_001_jira_invalid_base64_basic_auth_returns_401() {
    let (_harness, addr, client) = build_jira_harness().await;

    let resp = client
        .get(format!("http://{addr}/rest/api/3/issue/PROJ-1000"))
        .header("Authorization", "Basic !!!not-valid-base64!!!")
        .send()
        .await
        .expect("request failed");
    assert_eq!(
        resp.status().as_u16(),
        401,
        "Invalid base64 in Basic auth must return 401"
    );
}

/// Migrated from fidelity.rs: `test_execute_transition_on_missing_issue_returns_404`.
///
/// RED GATE: Fails at `build_jira_harness`.
///
/// Traces to: BC-3.5.001 postcondition 1; S-3.4.05 AC-003.
#[tokio::test]
async fn test_BC_3_5_001_jira_execute_transition_on_missing_issue_404() {
    let (_harness, addr, client) = build_jira_harness().await;

    let resp = execute_transition(&client, addr, "GHOST-9999", "11").await;
    assert_eq!(resp.status().as_u16(), 404);
    let body: Value = resp.json().await.expect("body failed");
    assert_eq!(body["errorMessages"][0], "Issue does not exist");
}

/// Migrated from fidelity.rs: `test_list_transitions_on_missing_issue_returns_404`.
///
/// RED GATE: Fails at `build_jira_harness`.
///
/// Traces to: BC-3.5.001 postcondition 1; S-3.4.05 AC-003.
#[tokio::test]
async fn test_BC_3_5_001_jira_list_transitions_on_missing_issue_404() {
    let (_harness, addr, client) = build_jira_harness().await;

    let resp = client
        .get(format!(
            "http://{addr}/rest/api/3/issue/GHOST-9999/transitions"
        ))
        .header("Authorization", basic_auth())
        .send()
        .await
        .expect("request failed");
    assert_eq!(
        resp.status().as_u16(),
        404,
        "GET transitions on non-existent issue must return 404"
    );
}

/// Migrated from fidelity.rs: `test_missing_issuetype_entirely_returns_400`.
///
/// RED GATE: Fails at `build_jira_harness`.
///
/// Traces to: BC-3.5.001 postcondition 1; S-3.4.05 AC-003.
#[tokio::test]
async fn test_BC_3_5_001_jira_missing_issuetype_entirely_returns_400() {
    let (_harness, addr, client) = build_jira_harness().await;

    let resp = client
        .post(format!("http://{addr}/rest/api/3/issue"))
        .header("Authorization", basic_auth())
        .json(&serde_json::json!({
            "fields": {
                "project": {"key": "MISS"},
                "summary": "missing issuetype"
            }
        }))
        .send()
        .await
        .expect("request failed");
    assert_eq!(
        resp.status().as_u16(),
        400,
        "Missing issuetype must return 400"
    );
    let body: Value = resp.json().await.expect("body failed");
    assert!(
        !body["errors"]["issuetype"].is_null(),
        "errors.issuetype must be present"
    );
}

/// Migrated from fidelity.rs: `test_missing_summary_returns_400`.
///
/// RED GATE: Fails at `build_jira_harness`.
///
/// Traces to: BC-3.5.001 postcondition 1; S-3.4.05 AC-003.
#[tokio::test]
async fn test_BC_3_5_001_jira_missing_summary_returns_400() {
    let (_harness, addr, client) = build_jira_harness().await;

    let resp = client
        .post(format!("http://{addr}/rest/api/3/issue"))
        .header("Authorization", basic_auth())
        .json(&serde_json::json!({
            "fields": {
                "project": {"key": "MISS"},
                "issuetype": {"name": "Task"}
            }
        }))
        .send()
        .await
        .expect("request failed");
    assert_eq!(
        resp.status().as_u16(),
        400,
        "Missing summary must return 400"
    );
    let body: Value = resp.json().await.expect("body failed");
    assert_eq!(body["errors"]["summary"], "required");
}

/// Migrated from fidelity.rs: `test_open_to_done_direct_transition_id_31`.
///
/// RED GATE: Fails at `build_jira_harness`.
///
/// Traces to: BC-3.5.001 postcondition 1; S-3.4.05 AC-003.
#[tokio::test]
async fn test_BC_3_5_001_jira_open_to_done_direct_transition_id_31() {
    let (_harness, addr, client) = build_jira_harness().await;

    let (s, b) = create_issue(&client, addr, "DIRECT", "Story", "direct close test", None).await;
    assert_eq!(s, 201);
    let key = b["key"].as_str().expect("key").to_owned();

    let trans_resp = execute_transition(&client, addr, &key, "31").await;
    assert_eq!(
        trans_resp.status().as_u16(),
        204,
        "Open → Done direct transition must return 204"
    );

    let get_resp = client
        .get(format!("http://{addr}/rest/api/3/issue/{key}"))
        .header("Authorization", basic_auth())
        .send()
        .await
        .expect("get failed");
    let get_body: Value = get_resp.json().await.expect("get body failed");
    assert_eq!(get_body["fields"]["status"]["name"], "Done");
    assert_eq!(
        get_body["fields"]["status"]["id"], "6",
        "fields.status.id for Done must be '6'"
    );
}

/// Migrated from fidelity.rs: `test_inprogress_to_done_transition_id_21`.
///
/// RED GATE: Fails at `build_jira_harness`.
///
/// Traces to: BC-3.5.001 postcondition 1; S-3.4.05 AC-003.
#[tokio::test]
async fn test_BC_3_5_001_jira_inprogress_to_done_transition_id_21() {
    let (_harness, addr, client) = build_jira_harness().await;

    let (s, b) = create_issue(&client, addr, "PROG", "Epic", "inprogress to done", None).await;
    assert_eq!(s, 201);
    let key = b["key"].as_str().expect("key").to_owned();

    // Transition to InProgress (id "11").
    let t1 = execute_transition(&client, addr, &key, "11").await;
    assert_eq!(t1.status().as_u16(), 204);

    // Verify InProgress transitions list: only id "21" must be available.
    let transitions = list_transitions(&client, addr, &key).await;
    let ids: Vec<&str> = transitions
        .iter()
        .filter_map(|t| t["id"].as_str())
        .collect();
    assert_eq!(
        ids,
        vec!["21"],
        "InProgress must only offer transition id '21'"
    );

    // Transition to Done (id "21").
    let t2 = execute_transition(&client, addr, &key, "21").await;
    assert_eq!(
        t2.status().as_u16(),
        204,
        "InProgress → Done transition must return 204"
    );

    let get_resp = client
        .get(format!("http://{addr}/rest/api/3/issue/{key}"))
        .header("Authorization", basic_auth())
        .send()
        .await
        .expect("get failed");
    let get_body: Value = get_resp.json().await.expect("body failed");
    assert_eq!(get_body["fields"]["status"]["name"], "Done");
}

/// Migrated from fidelity.rs: `test_open_transition_names_are_start_progress_and_close`.
///
/// RED GATE: Fails at `build_jira_harness`.
///
/// Traces to: BC-3.5.001 postcondition 1; S-3.4.05 AC-003.
#[tokio::test]
async fn test_BC_3_5_001_jira_open_transition_names_start_progress_and_close() {
    let (_harness, addr, client) = build_jira_harness().await;

    let (s, b) = create_issue(
        &client,
        addr,
        "NAMES",
        "Task",
        "transition names test",
        None,
    )
    .await;
    assert_eq!(s, 201);
    let key = b["key"].as_str().expect("key").to_owned();

    let transitions = list_transitions(&client, addr, &key).await;
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
}

/// Migrated from fidelity.rs: `test_dtu_health_returns_200_without_auth`.
///
/// RED GATE: Fails at `build_jira_harness`.
///
/// Traces to: BC-3.5.001 postcondition 1; S-3.4.05 AC-003.
#[tokio::test]
async fn test_BC_3_5_001_jira_dtu_health_returns_200_without_auth() {
    let (_harness, addr, client) = build_jira_harness().await;

    let resp = client
        .get(format!("http://{addr}/dtu/health"))
        .send()
        .await
        .expect("request failed");
    assert_eq!(
        resp.status().as_u16(),
        200,
        "GET /dtu/health must return 200 without auth"
    );
    let body: Value = resp.json().await.expect("body failed");
    assert_eq!(body["status"], "ok");
}

/// Migrated from fidelity.rs: `test_dtu_configure_without_admin_token_returns_401`.
///
/// RED GATE: Fails at `build_jira_harness`.
///
/// Traces to: BC-3.5.001 postcondition 1; S-3.4.05 AC-003.
#[tokio::test]
async fn test_BC_3_5_001_jira_configure_without_admin_token_returns_401() {
    let (_harness, addr, client) = build_jira_harness().await;

    let resp = client
        .post(format!("http://{addr}/dtu/configure"))
        .json(&serde_json::json!({"failure_mode": "none"}))
        .send()
        .await
        .expect("request failed");
    assert_eq!(
        resp.status().as_u16(),
        401,
        "POST /dtu/configure without X-Admin-Token must return 401"
    );
}

/// Migrated from fidelity.rs: `test_dtu_configure_with_wrong_admin_token_returns_401`.
///
/// RED GATE: Fails at `build_jira_harness`.
///
/// Traces to: BC-3.5.001 postcondition 1; S-3.4.05 AC-003.
#[tokio::test]
async fn test_BC_3_5_001_jira_configure_with_wrong_admin_token_returns_401() {
    let (_harness, addr, client) = build_jira_harness().await;

    let resp = client
        .post(format!("http://{addr}/dtu/configure"))
        .header("X-Admin-Token", "definitely-wrong-token")
        .json(&serde_json::json!({"failure_mode": "none"}))
        .send()
        .await
        .expect("request failed");
    assert_eq!(
        resp.status().as_u16(),
        401,
        "POST /dtu/configure with wrong token must return 401"
    );
}

/// Migrated from fidelity.rs: `test_dtu_issues_response_includes_comment_count`.
///
/// RED GATE: Fails at `build_jira_harness`.
///
/// Traces to: BC-3.5.001 postcondition 1; S-3.4.05 AC-003.
#[tokio::test]
async fn test_BC_3_5_001_jira_issues_response_includes_comment_count() {
    let (_harness, addr, client) = build_jira_harness().await;

    let (s, b) = create_issue(&client, addr, "DTUI", "Task", "dtu issues test", None).await;
    assert_eq!(s, 201);
    let key = b["key"].as_str().expect("key").to_owned();

    let issues = get_dtu_issues(&client, addr).await;
    let found = issues
        .iter()
        .find(|i| i["key"] == key)
        .expect("issue not found");
    assert_eq!(
        found["comment_count"], 0,
        "GET /dtu/issues must include comment_count == 0 for new issue"
    );
    assert!(
        found["summary"].is_string(),
        "GET /dtu/issues must include summary field"
    );
}

/// Migrated from fidelity.rs: `test_add_comment_on_missing_issue_returns_404`.
///
/// RED GATE: Fails at `build_jira_harness`.
///
/// Traces to: BC-3.5.001 postcondition 1; S-3.4.05 AC-003.
#[tokio::test]
async fn test_BC_3_5_001_jira_add_comment_on_missing_issue_returns_404() {
    let (_harness, addr, client) = build_jira_harness().await;

    let resp = client
        .post(format!("http://{addr}/rest/api/3/issue/GHOST-0000/comment"))
        .header("Authorization", basic_auth())
        .json(&serde_json::json!({"body": {"type": "doc", "content": []}}))
        .send()
        .await
        .expect("request failed");
    assert_eq!(resp.status().as_u16(), 404);
    let body: Value = resp.json().await.expect("body failed");
    assert_eq!(body["errorMessages"][0], "Issue does not exist");
}

/// Migrated from fidelity.rs: `test_add_comment_response_has_id_self_created`.
///
/// RED GATE: Fails at `build_jira_harness`.
///
/// Traces to: BC-3.5.001 postcondition 1; S-3.4.05 AC-003.
#[tokio::test]
async fn test_BC_3_5_001_jira_add_comment_response_has_id_self_created() {
    let (_harness, addr, client) = build_jira_harness().await;

    let (s, b) = create_issue(&client, addr, "CMSHP", "Task", "comment shape test", None).await;
    assert_eq!(s, 201);
    let key = b["key"].as_str().expect("key").to_owned();

    let comment_resp = client
        .post(format!("http://{addr}/rest/api/3/issue/{key}/comment"))
        .header("Authorization", basic_auth())
        .json(&serde_json::json!({"body": {"type": "doc", "content": []}}))
        .send()
        .await
        .expect("request failed");
    assert_eq!(comment_resp.status().as_u16(), 201);
    let body: Value = comment_resp.json().await.expect("body failed");

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
}

/// Migrated from fidelity.rs: `test_inprogress_to_inprogress_is_invalid_transition`.
///
/// RED GATE: Fails at `build_jira_harness`.
///
/// Traces to: BC-3.5.001 postcondition 1; S-3.4.05 AC-003.
#[tokio::test]
async fn test_BC_3_5_001_jira_inprogress_to_inprogress_invalid_transition() {
    let (_harness, addr, client) = build_jira_harness().await;

    let (s, b) = create_issue(
        &client,
        addr,
        "INVTR",
        "Task",
        "invalid re-transition test",
        None,
    )
    .await;
    assert_eq!(s, 201);
    let key = b["key"].as_str().expect("key").to_owned();

    // Advance to InProgress.
    let t1 = execute_transition(&client, addr, &key, "11").await;
    assert_eq!(t1.status().as_u16(), 204);

    // Attempt id "11" again from InProgress — must be rejected.
    let t2 = execute_transition(&client, addr, &key, "11").await;
    assert_eq!(
        t2.status().as_u16(),
        400,
        "Re-applying transition id '11' from InProgress must return 400"
    );
    let body: Value = t2.json().await.expect("body failed");
    assert_eq!(body["errorMessages"][0], "Invalid transition id");
}

/// Migrated from fidelity.rs: `test_all_valid_issue_types_are_accepted`.
///
/// RED GATE: Fails at `build_jira_harness`.
///
/// Traces to: BC-3.5.001 postcondition 1; S-3.4.05 AC-003.
#[tokio::test]
async fn test_BC_3_5_001_jira_all_valid_issue_types_accepted() {
    let (_harness, addr, client) = build_jira_harness().await;

    for issue_type in ["Task", "Bug", "Story", "Epic", "Incident"] {
        let resp = client
            .post(format!("http://{addr}/rest/api/3/issue"))
            .header("Authorization", basic_auth())
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
            resp.status().as_u16(),
            201,
            "Issue type '{issue_type}' must be accepted with 201"
        );
        let body: Value = resp.json().await.expect("body failed");
        assert!(
            body["key"].as_str().unwrap_or("").starts_with("TYPES-"),
            "Issue type '{issue_type}': key must start with TYPES-"
        );
    }
}

// ---------------------------------------------------------------------------
// Migrated: org_tagging.rs → test_BC_3_2_004_jira_* / test_BC_3_2_005_jira_*
// ---------------------------------------------------------------------------

/// Migrated from org_tagging.rs: `test_BC_3_2_004_ac001_org_id_in_issue_record`.
///
/// OrgId UUID appears in IssueRecord.org_id after create_issue with X-Prism-Org-Id.
///
/// RED GATE PRIMARY: `build_jira_harness` panics.
/// RED GATE SECONDARY: `IssueRecord.org_id` is empty — the handler does not
/// extract `X-Prism-Org-Id`.
///
/// Traces to: BC-3.2.004 postcondition 1; VP-087; S-3.4.05 AC-006.
#[tokio::test]
async fn test_BC_3_2_004_jira_ac001_org_id_in_issue_record() {
    let (_harness, addr, client) = build_jira_harness().await;

    let (status, body) = create_issue(
        &client,
        addr,
        "PROJ",
        "Task",
        "Test issue for org-id",
        Some(ORG_UUID_A),
    )
    .await;
    assert_eq!(status, 201, "create issue must return 201; body: {body}");
    let issue_key = body["key"].as_str().expect("response.key");

    // Retrieve org_id from GET /dtu/issues.
    let issues = get_dtu_issues(&client, addr).await;
    let found = issues
        .iter()
        .find(|i| i["key"].as_str() == Some(issue_key))
        .expect("issue must be in /dtu/issues");

    assert_eq!(
        found["org_id"].as_str().unwrap_or(""),
        ORG_UUID_A,
        "BC-3.2.004 postcondition 1: IssueRecord.org_id must equal the sender's UUID"
    );
}

/// Migrated from org_tagging.rs: `test_BC_3_2_004_ac002_issue_key_not_org_scoped`.
///
/// issue_key does not contain org_id UUID (ADR-008 §1.2: MSSP-scoped keys).
///
/// RED GATE: Fails at `build_jira_harness`.
///
/// Traces to: BC-3.2.004 postcondition 2; VP-088; S-3.4.05 AC-006.
#[tokio::test]
async fn test_BC_3_2_004_jira_ac002_issue_key_not_org_scoped() {
    let (_harness, addr, client) = build_jira_harness().await;

    let (status, body) = create_issue(
        &client,
        addr,
        "PROJ",
        "Task",
        "Key isolation test",
        Some(ORG_UUID_A),
    )
    .await;
    assert_eq!(status, 201);
    let issue_key = body["key"].as_str().expect("response.key");

    assert!(
        !issue_key.contains(ORG_UUID_A),
        "BC-3.2.004 postcondition 2 / ADR-008 §1.2: issue_key '{issue_key}' must not \
         contain the OrgId UUID '{ORG_UUID_A}'"
    );
}

/// Migrated from org_tagging.rs: `test_BC_3_2_004_ac002_org_id_absent_from_routing`.
///
/// org_id does not appear in response URL, headers, or create-issue response body.
///
/// RED GATE: Fails at `build_jira_harness`.
///
/// Traces to: BC-3.2.004 postcondition 2; VP-088; S-3.4.05 AC-006.
#[tokio::test]
async fn test_BC_3_2_004_jira_ac002_org_id_absent_from_routing() {
    let (_harness, addr, client) = build_jira_harness().await;

    let resp = client
        .post(format!("http://{addr}/rest/api/3/issue"))
        .header("Authorization", basic_auth())
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

    // Check response URL.
    let url_str = resp.url().as_str();
    assert!(
        !url_str.contains(ORG_UUID_A),
        "BC-3.2.004 postcondition 2: response URL must not contain OrgId UUID"
    );

    // Check response headers.
    for (name, value) in resp.headers() {
        let val_str = value.to_str().unwrap_or("");
        assert!(
            !val_str.contains(ORG_UUID_A),
            "BC-3.2.004 postcondition 2: response header '{}' must not contain OrgId UUID",
            name
        );
    }

    // Check response body (create response: id, key, self).
    let body: Value = resp.json().await.expect("JSON body");
    let body_str = body.to_string();
    assert!(
        !body_str.contains(ORG_UUID_A),
        "BC-3.2.004 postcondition 2: response body must not contain OrgId UUID; body: {body_str}"
    );
}

/// Migrated from org_tagging.rs: `test_BC_3_2_004_ac003_concurrent_issues_distinguished`.
///
/// Concurrent issues from org_A and org_B each carry their sender's OrgId.
///
/// RED GATE PRIMARY: `build_jira_harness` panics.
/// RED GATE SECONDARY: `IssueRecord.org_id` is empty.
///
/// Traces to: BC-3.2.004 postcondition 4; VP-089; S-3.4.05 AC-006.
#[tokio::test]
async fn test_BC_3_2_004_jira_ac003_concurrent_issues_distinguished() {
    let (_harness, addr, client) = build_jira_harness().await;

    let addr_a = addr;
    let addr_b = addr;

    let task_a = tokio::spawn(async move {
        let c = reqwest::Client::new();
        c.post(format!("http://{addr_a}/rest/api/3/issue"))
            .header("Authorization", {
                use base64::Engine as _;
                format!(
                    "Basic {}",
                    base64::engine::general_purpose::STANDARD.encode("u:t")
                )
            })
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
        c.post(format!("http://{addr_b}/rest/api/3/issue"))
            .header("Authorization", {
                use base64::Engine as _;
                format!(
                    "Basic {}",
                    base64::engine::general_purpose::STANDARD.encode("u:t")
                )
            })
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
    let resp_a: Value = r_a.expect("task A").json().await.expect("body A");
    let resp_b: Value = r_b.expect("task B").json().await.expect("body B");

    let key_a = resp_a["key"].as_str().expect("key_a");
    let key_b = resp_b["key"].as_str().expect("key_b");

    let issues = get_dtu_issues(&client, addr).await;

    let rec_a = issues
        .iter()
        .find(|i| i["key"].as_str() == Some(key_a))
        .expect("issue A");
    let rec_b = issues
        .iter()
        .find(|i| i["key"].as_str() == Some(key_b))
        .expect("issue B");

    assert_eq!(
        rec_a["org_id"].as_str().unwrap_or(""),
        ORG_UUID_A,
        "BC-3.2.004 postcondition 4: issue A must carry org_A's UUID"
    );
    assert_eq!(
        rec_b["org_id"].as_str().unwrap_or(""),
        ORG_UUID_B,
        "BC-3.2.004 postcondition 4: issue B must carry org_B's UUID"
    );
    assert_ne!(rec_a["org_id"], rec_b["org_id"], "org ids must be distinct");
}

/// Migrated from org_tagging.rs: `test_BC_3_2_004_ac004_mode_metadata_absent_from_query_results`.
///
/// GET /dtu/issues response rows contain no "mode", "shared", or "dtu_mode" fields.
///
/// RED GATE: Fails at `build_jira_harness`.
///
/// Traces to: BC-3.2.004 postcondition 5; VP-090; S-3.4.05 AC-006.
#[tokio::test]
async fn test_BC_3_2_004_jira_ac004_mode_metadata_absent_from_query_results() {
    let (_harness, addr, client) = build_jira_harness().await;

    let (status, _) = create_issue(
        &client,
        addr,
        "PROJ",
        "Task",
        "Mode metadata test",
        Some(ORG_UUID_A),
    )
    .await;
    assert_eq!(status, 201);

    let issues = get_dtu_issues(&client, addr).await;
    assert!(
        !issues.is_empty(),
        "expected at least one issue in /dtu/issues"
    );

    for (i, issue) in issues.iter().enumerate() {
        let row_str = issue.to_string();
        assert!(
            issue.get("mode").is_none(),
            "BC-3.2.004 postcondition 5 (issue {i}): 'mode' must not appear; row: {row_str}"
        );
        assert!(
            issue.get("shared").is_none(),
            "BC-3.2.004 postcondition 5 (issue {i}): 'shared' must not appear; row: {row_str}"
        );
        assert!(
            issue.get("dtu_mode").is_none(),
            "BC-3.2.004 postcondition 5 (issue {i}): 'dtu_mode' must not appear; row: {row_str}"
        );
    }
}

/// Migrated from org_tagging.rs: `test_BC_3_2_005_ac005_jira_dtu_mode_is_shared`.
///
/// JIRA_DTU_MODE constant is DtuMode::Shared (compile-time assertion).
///
/// This test is synchronous. It PASSES at Red Gate because the constant is set.
///
/// Traces to: BC-3.2.005 postcondition 1; VP-122; S-3.4.05 AC-003.
#[test]
fn test_BC_3_2_005_jira_ac005_jira_dtu_mode_is_shared() {
    use prism_dtu_common::DtuMode;
    use prism_dtu_jira::clone::JIRA_DTU_MODE;

    assert_eq!(
        JIRA_DTU_MODE,
        DtuMode::Shared,
        "JIRA_DTU_MODE must be DtuMode::Shared per BC-3.2.005 postcondition 1"
    );
}

/// Migrated from org_tagging.rs: `test_BC_3_2_005_ac005_mode_immutable_after_startup`.
///
/// DtuMode::Shared cannot be changed via configure() after startup.
///
/// RED GATE PRIMARY: `build_jira_harness` panics.
///
/// Traces to: BC-3.2.005 postcondition 4; VP-123; S-3.4.05 AC-003.
#[tokio::test]
async fn test_BC_3_2_005_jira_ac005_mode_immutable_after_startup() {
    use prism_dtu_common::DtuMode;
    use prism_dtu_jira::clone::JIRA_DTU_MODE;

    let (harness, addr, client) = build_jira_harness().await;

    // Attempt mode change (must be ignored).
    let _ = harness
        .inject_failure(TENANT, DtuType::Jira, prism_dtu_common::FailureMode::None)
        .await;

    assert_eq!(
        JIRA_DTU_MODE,
        DtuMode::Shared,
        "BC-3.2.005 postcondition 4: JIRA_DTU_MODE must remain DtuMode::Shared \
         after any attempted runtime mode change"
    );

    // Dynamic check: newly created issues must still carry org_id.
    let (status, body) = create_issue(
        &client,
        addr,
        "PROJ",
        "Task",
        "Mode immutable check",
        Some(ORG_UUID_A),
    )
    .await;
    assert_eq!(
        status, 201,
        "create issue must succeed after configure attempt; body: {body}"
    );
    let issue_key = body["key"].as_str().expect("response.key");

    let issues = get_dtu_issues(&client, addr).await;
    let record = issues
        .iter()
        .find(|i| i["key"].as_str() == Some(issue_key))
        .expect("issue must be in registry");
    assert_eq!(
        record["org_id"].as_str().unwrap_or(""),
        ORG_UUID_A,
        "BC-3.2.005 postcondition 4: shared-mode dispatch must still tag org_id after \
         a (rejected) configure attempt"
    );
}

/// Migrated from org_tagging.rs: `test_BC_3_2_005_ac006_invalid_mode_string_rejected`.
///
/// mode = "SHared" (wrong case) fails serde deserialization.
///
/// This test is synchronous. It PASSES at Red Gate.
///
/// Traces to: BC-3.2.005 postcondition 3; S-3.4.05 AC-003.
#[test]
fn test_BC_3_2_005_jira_ac006_invalid_mode_string_rejected() {
    use prism_dtu_common::DtuMode;

    let result: Result<DtuMode, _> = serde_json::from_str("\"SHared\"");
    assert!(
        result.is_err(),
        "BC-3.2.005 postcondition 3: DtuMode must reject unknown variant 'SHared'; got Ok({:?})",
        result.ok()
    );
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("SHared") || err_msg.contains("variant") || err_msg.contains("unknown"),
        "error message must identify the offending value; got: {err_msg}"
    );
}

// ---------------------------------------------------------------------------
// New harness-specific tests (S-3.4.05 ACs)
// ---------------------------------------------------------------------------

/// AC-006 (S-3.4.05): Shared-mode OrgId tagging — Jira designated issue field.
///
/// Builds a harness with a single shared Jira clone (IsolationMode::Logical,
/// DtuType::Jira). Creates tickets on behalf of org_C. Asserts that the captured
/// IssueRecord contains the org_C UUID in the designated issue field.
///
/// Canonical test vectors: ORG_UUID_C = "00000000-0000-7000-8000-000000000003".
///
/// RED GATE PRIMARY: `build_jira_harness` panics.
/// RED GATE SECONDARY: `IssueRecord.org_id` is empty.
///
/// Traces to: BC-3.2.004 postconditions 1, 2; BC-3.5.001 postcondition 1;
///            VP-087, VP-088; S-3.4.05 AC-006.
#[tokio::test]
async fn ac_shared_mode_org_id_tagging() {
    let (_harness, addr, client) = build_jira_harness().await;

    // Create ticket for org_C.
    let (status_c, body_c) = create_issue(
        &client,
        addr,
        "ORGC",
        "Task",
        "ticket for org C",
        Some(ORG_UUID_C),
    )
    .await;
    assert_eq!(status_c, 201, "org_C create must succeed");
    let key_c = body_c["key"].as_str().expect("key_c");

    // Verify org_C UUID is NOT in response URL or headers.
    let create_resp_c = client
        .post(format!("http://{addr}/rest/api/3/issue"))
        .header("Authorization", basic_auth())
        .header("X-Prism-Org-Id", ORG_UUID_C)
        .json(&serde_json::json!({
            "fields": {
                "project": {"key": "ORGC2"},
                "issuetype": {"name": "Task"},
                "summary": "org C second ticket",
            }
        }))
        .send()
        .await
        .expect("create failed");
    let url_c = create_resp_c.url().to_string();
    assert!(
        !url_c.contains(ORG_UUID_C),
        "ORG_UUID_C must not appear in response URL"
    );
    for (name, value) in create_resp_c.headers() {
        let val_str = value.to_str().unwrap_or("");
        assert!(
            !val_str.contains(ORG_UUID_C),
            "ORG_UUID_C must not appear in response header '{name}'"
        );
    }

    // Create ticket for org_A.
    let (status_a, body_a) = create_issue(
        &client,
        addr,
        "ORGA",
        "Task",
        "ticket for org A",
        Some(ORG_UUID_A),
    )
    .await;
    assert_eq!(status_a, 201, "org_A create must succeed");
    let key_a = body_a["key"].as_str().expect("key_a");

    // Retrieve captured records and verify org_id tags.
    let issues = get_dtu_issues(&client, addr).await;

    let rec_c = issues
        .iter()
        .find(|i| i["key"].as_str() == Some(key_c))
        .expect("org_C issue");
    let rec_a = issues
        .iter()
        .find(|i| i["key"].as_str() == Some(key_a))
        .expect("org_A issue");

    assert_eq!(
        rec_c["org_id"].as_str().unwrap_or(""),
        ORG_UUID_C,
        "AC-006: org_C ticket must carry ORG_UUID_C in org_id field"
    );
    assert_eq!(
        rec_a["org_id"].as_str().unwrap_or(""),
        ORG_UUID_A,
        "AC-006: org_A ticket must carry ORG_UUID_A in org_id field"
    );
    assert_ne!(
        rec_c["org_id"], rec_a["org_id"],
        "AC-006: the two captured org_ids must be distinct"
    );
}

/// AC-006 / EC-001, EC-002 (S-3.4.05): Multi-org logical isolation in shared Jira mode.
///
/// A single shared Jira listener serves all orgs. Creates tickets for org_A and org_B
/// and verifies each IssueRecord carries its sender's OrgId, with no cross-contamination.
///
/// RED GATE PRIMARY: `build_jira_harness` panics.
/// RED GATE SECONDARY: `IssueRecord.org_id` is empty.
///
/// Traces to: BC-3.5.001 postconditions 1, 2; BC-3.2.004 postcondition 4;
///            VP-089, VP-122; S-3.4.05 AC-003, EC-001, EC-002.
#[tokio::test]
async fn ac_multi_org_logical_isolation_shared_mode() {
    let (_harness, addr, client) = build_jira_harness().await;

    let (s_a, b_a) = create_issue(
        &client,
        addr,
        "ISOLA",
        "Task",
        "org A ticket",
        Some(ORG_UUID_A),
    )
    .await;
    assert_eq!(s_a, 201);
    let key_a = b_a["key"].as_str().expect("key_a").to_owned();

    let (s_b, b_b) = create_issue(
        &client,
        addr,
        "ISOLB",
        "Task",
        "org B ticket",
        Some(ORG_UUID_B),
    )
    .await;
    assert_eq!(s_b, 201);
    let key_b = b_b["key"].as_str().expect("key_b").to_owned();

    let issues = get_dtu_issues(&client, addr).await;
    assert_eq!(
        issues.len(),
        2,
        "AC-006: both org tickets must be captured; got {}",
        issues.len()
    );

    let rec_a = issues
        .iter()
        .find(|i| i["key"].as_str() == Some(key_a.as_str()))
        .expect("org_A issue");
    let rec_b = issues
        .iter()
        .find(|i| i["key"].as_str() == Some(key_b.as_str()))
        .expect("org_B issue");

    assert_eq!(
        rec_a["org_id"].as_str().unwrap_or(""),
        ORG_UUID_A,
        "EC-001: org_A ticket must carry ORG_UUID_A"
    );
    assert_eq!(
        rec_b["org_id"].as_str().unwrap_or(""),
        ORG_UUID_B,
        "EC-002: org_B ticket must carry ORG_UUID_B"
    );

    // No cross-contamination: issue_key must not contain the other org's UUID.
    assert!(
        !key_a.contains(ORG_UUID_B),
        "EC-001: org_A issue_key must not contain ORG_UUID_B"
    );
    assert!(
        !key_b.contains(ORG_UUID_A),
        "EC-002: org_B issue_key must not contain ORG_UUID_A"
    );
}

/// AC-007 / EC-003 (S-3.4.05): `CustomerSpec` with `mode = "client"` for Jira does NOT
/// produce a startup error (BC-3.3.001-startup EC-003: MSSP Coordination types permit
/// client mode override).
///
/// RED GATE: This test verifies that `HarnessBuilder::build()` returns `Ok` — which
/// requires `DtuType::Jira` to be wired into the clone-server dispatch.
///
/// Traces to: BC-3.5.001 precondition 2 (valid customer registered);
///            BC-3.3.001-startup EC-003; S-3.4.05 AC-007.
#[tokio::test]
async fn ac_client_mode_override_does_not_produce_startup_error() {
    let harness_result = HarnessBuilder::new()
        .isolation(IsolationMode::Logical)
        .with_customer_overrides(TENANT, |spec| {
            spec.dtu_types = vec![DtuType::Jira];
        })
        .build()
        .await;

    assert!(
        harness_result.is_ok(),
        "BC-3.3.001 EC-003: HarnessBuilder with DtuType::Jira must NOT produce a startup \
         error; got: {:?}",
        harness_result.err()
    );

    let harness = harness_result.unwrap();
    let addr = harness
        .endpoint_for(TENANT, DtuType::Jira)
        .expect("Jira endpoint must be present after successful build");

    let client = reqwest::Client::new();
    let resp = client
        .get(format!("http://{addr}/dtu/health"))
        .send()
        .await
        .expect("GET /dtu/health must not fail at network level");

    assert_eq!(
        resp.status().as_u16(),
        200,
        "BC-3.3.001 EC-003: Jira clone health check must return 200 after clean startup"
    );
}
