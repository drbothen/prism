//! BC-3.6.001 v0.5 — Ops-clone failure-mode coverage tests.
//!
//! Tests that all 6 `FailureMode` variants are fully honored by the three
//! MSSP-coordination clones (Jira, PagerDuty, Slack).
//!
//! Naming convention: `test_BC_3_6_001_<clone>_<mode>_honored`
//!
//! # Modes under test
//!
//! | Mode | Expected behavior |
//! |------|-------------------|
//! | AuthReject | HTTP 401 on every user-facing request |
//! | NetworkTimeout | Response delayed by configured `after_ms` |
//! | MalformedResponse | Response body is NOT valid JSON |
//! | Unprocessable | HTTP 422 at configured `at_request_n` |
//! | InternalError | HTTP 500 at configured `at_request_n` |
//! | RateLimit | HTTP 429 after N requests (unchanged — regression guard) |
//!
//! # User-facing routes tested
//!
//! - Jira: POST /rest/api/3/issue (create_issue)
//! - PagerDuty: POST /v2/enqueue
//! - Slack: POST /services/test-token
//!
//! # Phase-2 (400 guard) tests
//!
//! After Phase 2 all modes are representable, so the 400 path for CURRENT variants
//! is unreachable. A future-variant guard test documents the contract for the
//! `#[non_exhaustive]` future-variant arm (BC-3.6.001 Postcondition 5 / EC-008 / EC-009).
//!
//! BC anchors: BC-3.6.001 Postconditions 1+5, Invariant 5, VP-129, VP-157.

#![allow(clippy::expect_used, non_snake_case)]

use std::time::{Duration, Instant};

use prism_dtu_harness::{DtuType, IsolationMode};
use serde_json::json;

// ---------------------------------------------------------------------------
// Test helpers
// ---------------------------------------------------------------------------

fn test_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .build()
        .expect("test client build must succeed")
}

/// Configure a clone's failure mode via POST /dtu/configure.
async fn configure_failure(
    client: &reqwest::Client,
    addr: std::net::SocketAddr,
    admin_token: &str,
    payload: serde_json::Value,
) {
    let resp = client
        .post(format!("http://{addr}/dtu/configure"))
        .header("x-admin-token", admin_token)
        .json(&payload)
        .send()
        .await
        .expect("configure request must not fail at transport level");
    assert_eq!(
        resp.status().as_u16(),
        200,
        "configure must succeed: payload={payload}"
    );
}

/// Reset a clone via POST /dtu/reset.
async fn reset_clone(client: &reqwest::Client, addr: std::net::SocketAddr) {
    let resp = client
        .post(format!("http://{addr}/dtu/reset"))
        .send()
        .await
        .expect("reset request must not fail at transport level");
    assert_eq!(resp.status().as_u16(), 200, "reset must return 200");
}

/// Build valid Jira Basic auth header value.
///
/// Jira harness accepts any valid base64-encoded `user:token` pair.
fn jira_basic_auth() -> String {
    use base64::Engine as _;
    let encoded = base64::engine::general_purpose::STANDARD.encode("test-user:test-token");
    format!("Basic {encoded}")
}

/// Send a create-issue request to Jira and return the status code.
async fn jira_create_issue(client: &reqwest::Client, addr: std::net::SocketAddr) -> u16 {
    let resp = client
        .post(format!("http://{addr}/rest/api/3/issue"))
        .header("Authorization", jira_basic_auth())
        .json(&json!({
            "fields": {
                "project": {"key": "ACME-SEC"},
                "issuetype": {"name": "Task"},
                "summary": "Test issue"
            }
        }))
        .send()
        .await
        .expect("jira create_issue must not fail at transport level");
    resp.status().as_u16()
}

/// Send a PagerDuty enqueue request and return the status code.
async fn pd_enqueue(client: &reqwest::Client, addr: std::net::SocketAddr) -> u16 {
    let resp = client
        .post(format!("http://{addr}/v2/enqueue"))
        .json(&json!({
            "routing_key": "test-key",
            "event_action": "trigger",
            "payload": {"summary": "Test alert", "severity": "critical", "source": "test"}
        }))
        .send()
        .await
        .expect("pd enqueue must not fail at transport level");
    resp.status().as_u16()
}

/// Send a Slack webhook request and return the status code.
async fn slack_webhook(client: &reqwest::Client, addr: std::net::SocketAddr) -> u16 {
    let resp = client
        .post(format!("http://{addr}/services/test-token"))
        .json(&json!({"text": "Test message"}))
        .send()
        .await
        .expect("slack webhook must not fail at transport level");
    resp.status().as_u16()
}

/// Send a Slack webhook request and return the full response (for body inspection).
async fn slack_webhook_resp(
    client: &reqwest::Client,
    addr: std::net::SocketAddr,
) -> reqwest::Response {
    client
        .post(format!("http://{addr}/services/test-token"))
        .json(&json!({"text": "Test message"}))
        .send()
        .await
        .expect("slack webhook must not fail at transport level")
}

// ---------------------------------------------------------------------------
// JIRA — AuthReject
// ---------------------------------------------------------------------------

/// BC-3.6.001 Postcondition 1: AuthReject → HTTP 401 on create_issue.
#[tokio::test]
async fn test_BC_3_6_001_jira_auth_reject_honored() {
    let harness = prism_dtu_harness::Harness::builder()
        .isolation(IsolationMode::Logical)
        .with_customer_overrides("jira-tenant", |spec| {
            spec.dtu_types = vec![DtuType::Jira];
        })
        .build()
        .await
        .expect("harness build must succeed");

    let addr = harness
        .endpoint_for("jira-tenant", DtuType::Jira)
        .expect("jira endpoint must exist");
    let admin_token = harness
        .admin_token_for("jira-tenant", DtuType::Jira)
        .expect("jira admin token must exist")
        .to_owned();
    let client = test_client();

    configure_failure(&client, addr, &admin_token, json!({"auth_mode": "reject"})).await;

    let status = jira_create_issue(&client, addr).await;
    assert_eq!(
        status, 401,
        "Jira create_issue must return 401 under AuthReject \
         (BC-3.6.001 Postcondition 1 / Invariant 5)"
    );
}

// ---------------------------------------------------------------------------
// JIRA — NetworkTimeout
// ---------------------------------------------------------------------------

/// BC-3.6.001 Postcondition 1: NetworkTimeout → response delayed by after_ms on create_issue.
#[tokio::test]
async fn test_BC_3_6_001_jira_network_timeout_honored() {
    let harness = prism_dtu_harness::Harness::builder()
        .isolation(IsolationMode::Logical)
        .with_customer_overrides("jira-timeout", |spec| {
            spec.dtu_types = vec![DtuType::Jira];
        })
        .build()
        .await
        .expect("harness build must succeed");

    let addr = harness
        .endpoint_for("jira-timeout", DtuType::Jira)
        .expect("jira endpoint must exist");
    let admin_token = harness
        .admin_token_for("jira-timeout", DtuType::Jira)
        .expect("jira admin token must exist")
        .to_owned();

    // Use a longer timeout client for this test only.
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .expect("client build");

    configure_failure(
        &client,
        addr,
        &admin_token,
        json!({"network_timeout_ms": 200}),
    )
    .await;

    let start = Instant::now();
    let status = jira_create_issue(&client, addr).await;
    let elapsed = start.elapsed();

    assert!(
        elapsed >= Duration::from_millis(150),
        "Jira create_issue must be delayed by at least 150ms under NetworkTimeout(200ms); \
         elapsed={elapsed:?} (BC-3.6.001 Postcondition 1 / Invariant 5)"
    );
    // After the delay the request should still succeed (timeout injects delay, not error).
    assert!(
        status == 201 || status == 200,
        "Jira create_issue must succeed (201) after the timeout delay; got {status}"
    );
}

// ---------------------------------------------------------------------------
// JIRA — MalformedResponse
// ---------------------------------------------------------------------------

/// BC-3.6.001 Postcondition 1: MalformedResponse → body is NOT valid JSON on create_issue.
#[tokio::test]
async fn test_BC_3_6_001_jira_malformed_response_honored() {
    let harness = prism_dtu_harness::Harness::builder()
        .isolation(IsolationMode::Logical)
        .with_customer_overrides("jira-malformed", |spec| {
            spec.dtu_types = vec![DtuType::Jira];
        })
        .build()
        .await
        .expect("harness build must succeed");

    let addr = harness
        .endpoint_for("jira-malformed", DtuType::Jira)
        .expect("jira endpoint must exist");
    let admin_token = harness
        .admin_token_for("jira-malformed", DtuType::Jira)
        .expect("jira admin token must exist")
        .to_owned();
    let client = test_client();

    configure_failure(
        &client,
        addr,
        &admin_token,
        json!({"malformed_response": true}),
    )
    .await;

    let resp = client
        .post(format!("http://{addr}/rest/api/3/issue"))
        .header("Authorization", jira_basic_auth())
        .json(&json!({
            "fields": {
                "project": {"key": "ACME-SEC"},
                "issuetype": {"name": "Task"},
                "summary": "Test issue"
            }
        }))
        .send()
        .await
        .expect("request must not fail at transport level");

    let body_bytes = resp.bytes().await.expect("body must be readable");
    let parse_result = serde_json::from_slice::<serde_json::Value>(&body_bytes);
    assert!(
        parse_result.is_err(),
        "Jira create_issue body must be invalid JSON under MalformedResponse; \
         got valid JSON (BC-3.6.001 Postcondition 1 / Invariant 5)"
    );
}

// ---------------------------------------------------------------------------
// JIRA — Unprocessable
// ---------------------------------------------------------------------------

/// BC-3.6.001 Postcondition 1: Unprocessable → HTTP 422 at at_request_n on create_issue.
#[tokio::test]
async fn test_BC_3_6_001_jira_unprocessable_honored() {
    let harness = prism_dtu_harness::Harness::builder()
        .isolation(IsolationMode::Logical)
        .with_customer_overrides("jira-unprocessable", |spec| {
            spec.dtu_types = vec![DtuType::Jira];
        })
        .build()
        .await
        .expect("harness build must succeed");

    let addr = harness
        .endpoint_for("jira-unprocessable", DtuType::Jira)
        .expect("jira endpoint must exist");
    let admin_token = harness
        .admin_token_for("jira-unprocessable", DtuType::Jira)
        .expect("jira admin token must exist")
        .to_owned();
    let client = test_client();

    // Inject Unprocessable at request 2.
    configure_failure(&client, addr, &admin_token, json!({"unprocessable_at": 2})).await;

    // Request 1 must succeed.
    let s1 = jira_create_issue(&client, addr).await;
    assert!(
        s1 == 201 || s1 == 200,
        "Request 1 must succeed before unprocessable threshold; got {s1}"
    );

    // Request 2 must return 422.
    let s2 = jira_create_issue(&client, addr).await;
    assert_eq!(
        s2, 422,
        "Jira create_issue request 2 must return 422 under Unprocessable(at_n=2) \
         (BC-3.6.001 Postcondition 1 / Invariant 5)"
    );
}

// ---------------------------------------------------------------------------
// JIRA — InternalError
// ---------------------------------------------------------------------------

/// BC-3.6.001 Postcondition 1: InternalError → HTTP 500 at at_request_n on create_issue.
#[tokio::test]
async fn test_BC_3_6_001_jira_internal_error_honored() {
    let harness = prism_dtu_harness::Harness::builder()
        .isolation(IsolationMode::Logical)
        .with_customer_overrides("jira-internal-error", |spec| {
            spec.dtu_types = vec![DtuType::Jira];
        })
        .build()
        .await
        .expect("harness build must succeed");

    let addr = harness
        .endpoint_for("jira-internal-error", DtuType::Jira)
        .expect("jira endpoint must exist");
    let admin_token = harness
        .admin_token_for("jira-internal-error", DtuType::Jira)
        .expect("jira admin token must exist")
        .to_owned();
    let client = test_client();

    // Inject InternalError at request 1.
    configure_failure(&client, addr, &admin_token, json!({"internal_error_at": 1})).await;

    let status = jira_create_issue(&client, addr).await;
    assert_eq!(
        status, 500,
        "Jira create_issue must return 500 under InternalError(at_n=1) \
         (BC-3.6.001 Postcondition 1 / Invariant 5)"
    );
}

// ---------------------------------------------------------------------------
// JIRA — RateLimit (regression guard — was already supported)
// ---------------------------------------------------------------------------

/// BC-3.6.001 regression: RateLimit still honored after Phase 2 refactor.
#[tokio::test]
async fn test_BC_3_6_001_jira_rate_limit_honored() {
    let harness = prism_dtu_harness::Harness::builder()
        .isolation(IsolationMode::Logical)
        .with_customer_overrides("jira-rate-limit", |spec| {
            spec.dtu_types = vec![DtuType::Jira];
        })
        .build()
        .await
        .expect("harness build must succeed");

    let addr = harness
        .endpoint_for("jira-rate-limit", DtuType::Jira)
        .expect("jira endpoint must exist");
    let admin_token = harness
        .admin_token_for("jira-rate-limit", DtuType::Jira)
        .expect("jira admin token must exist")
        .to_owned();
    let client = test_client();

    configure_failure(
        &client,
        addr,
        &admin_token,
        json!({"rate_limit_after": 1, "retry_after_secs": 60}),
    )
    .await;

    // Request 1 must succeed.
    let s1 = jira_create_issue(&client, addr).await;
    assert!(
        s1 == 201 || s1 == 200,
        "Request 1 must succeed before rate-limit threshold; got {s1}"
    );

    // Request 2 must return 429.
    let s2 = jira_create_issue(&client, addr).await;
    assert_eq!(
        s2, 429,
        "Jira create_issue request 2 must return 429 under RateLimit(after_n=1) \
         (BC-3.6.001 Postcondition 1 regression guard)"
    );
}

// ---------------------------------------------------------------------------
// PAGERDUTY — AuthReject
// ---------------------------------------------------------------------------

/// BC-3.6.001 Postcondition 1 / PagerDuty-specific contract: AuthReject → HTTP 403
/// with body `{"status": "invalid key"}`.
///
/// BC-3.6.001 Postcondition 1 generically says "401" for AuthReject, but the PagerDuty
/// Events API returns HTTP 403 for an invalid routing key ("invalid key"), which is the
/// behavior the real `prism-dtu-pagerduty` clone has always served. The harness clone
/// must match the real clone's contract. BC-3.6.001 line 41 acknowledges "auth-reject
/// (401/403)" — per-clone variation is expected.
///
/// Authoritative anchor: `fidelity.rs::test_ac8_auth_reject_mode_returns_403` and
/// `harness_tests.rs::test_BC_3_5_001_pd_ac8_auth_reject_mode_returns_403` (BC-3.5.001).
#[tokio::test]
async fn test_BC_3_6_001_pagerduty_auth_reject_honored() {
    let harness = prism_dtu_harness::Harness::builder()
        .isolation(IsolationMode::Logical)
        .with_customer_overrides("pd-tenant", |spec| {
            spec.dtu_types = vec![DtuType::PagerDuty];
        })
        .build()
        .await
        .expect("harness build must succeed");

    let addr = harness
        .endpoint_for("pd-tenant", DtuType::PagerDuty)
        .expect("pagerduty endpoint must exist");
    let admin_token = harness
        .admin_token_for("pd-tenant", DtuType::PagerDuty)
        .expect("pagerduty admin token must exist")
        .to_owned();
    let client = test_client();

    configure_failure(&client, addr, &admin_token, json!({"auth_mode": "reject"})).await;

    let status = pd_enqueue(&client, addr).await;
    assert_eq!(
        status, 403,
        "PagerDuty post_enqueue must return 403 under AuthReject — \
         PagerDuty Events API uses 403 'invalid key' for bad routing_key; \
         harness clone must match real-clone contract (BC-3.5.001 / fidelity.rs AC-8)"
    );
}

// ---------------------------------------------------------------------------
// PAGERDUTY — NetworkTimeout
// ---------------------------------------------------------------------------

/// BC-3.6.001 Postcondition 1: NetworkTimeout → delayed response on post_enqueue.
#[tokio::test]
async fn test_BC_3_6_001_pagerduty_network_timeout_honored() {
    let harness = prism_dtu_harness::Harness::builder()
        .isolation(IsolationMode::Logical)
        .with_customer_overrides("pd-timeout", |spec| {
            spec.dtu_types = vec![DtuType::PagerDuty];
        })
        .build()
        .await
        .expect("harness build must succeed");

    let addr = harness
        .endpoint_for("pd-timeout", DtuType::PagerDuty)
        .expect("pagerduty endpoint must exist");
    let admin_token = harness
        .admin_token_for("pd-timeout", DtuType::PagerDuty)
        .expect("pagerduty admin token must exist")
        .to_owned();

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .expect("client build");

    configure_failure(
        &client,
        addr,
        &admin_token,
        json!({"network_timeout_ms": 200}),
    )
    .await;

    let start = Instant::now();
    let status = pd_enqueue(&client, addr).await;
    let elapsed = start.elapsed();

    assert!(
        elapsed >= Duration::from_millis(150),
        "PagerDuty post_enqueue must be delayed by at least 150ms under NetworkTimeout(200ms); \
         elapsed={elapsed:?} (BC-3.6.001 Postcondition 1 / Invariant 5)"
    );
    assert!(
        status == 202 || status == 200,
        "PagerDuty post_enqueue must succeed after delay; got {status}"
    );
}

// ---------------------------------------------------------------------------
// PAGERDUTY — MalformedResponse
// ---------------------------------------------------------------------------

/// BC-3.6.001 Postcondition 1: MalformedResponse → body is NOT valid JSON on post_enqueue.
#[tokio::test]
async fn test_BC_3_6_001_pagerduty_malformed_response_honored() {
    let harness = prism_dtu_harness::Harness::builder()
        .isolation(IsolationMode::Logical)
        .with_customer_overrides("pd-malformed", |spec| {
            spec.dtu_types = vec![DtuType::PagerDuty];
        })
        .build()
        .await
        .expect("harness build must succeed");

    let addr = harness
        .endpoint_for("pd-malformed", DtuType::PagerDuty)
        .expect("pagerduty endpoint must exist");
    let admin_token = harness
        .admin_token_for("pd-malformed", DtuType::PagerDuty)
        .expect("pagerduty admin token must exist")
        .to_owned();
    let client = test_client();

    configure_failure(
        &client,
        addr,
        &admin_token,
        json!({"malformed_response": true}),
    )
    .await;

    let resp = client
        .post(format!("http://{addr}/v2/enqueue"))
        .json(&json!({
            "routing_key": "test-key",
            "event_action": "trigger",
            "payload": {"summary": "Test alert", "severity": "critical", "source": "test"}
        }))
        .send()
        .await
        .expect("pd enqueue must not fail at transport level");

    let body_bytes = resp.bytes().await.expect("body must be readable");
    let parse_result = serde_json::from_slice::<serde_json::Value>(&body_bytes);
    assert!(
        parse_result.is_err(),
        "PagerDuty post_enqueue body must be invalid JSON under MalformedResponse; \
         got valid JSON (BC-3.6.001 Postcondition 1 / Invariant 5)"
    );
}

// ---------------------------------------------------------------------------
// PAGERDUTY — Unprocessable
// ---------------------------------------------------------------------------

/// BC-3.6.001 Postcondition 1: Unprocessable → HTTP 422 at at_request_n on post_enqueue.
#[tokio::test]
async fn test_BC_3_6_001_pagerduty_unprocessable_honored() {
    let harness = prism_dtu_harness::Harness::builder()
        .isolation(IsolationMode::Logical)
        .with_customer_overrides("pd-unprocessable", |spec| {
            spec.dtu_types = vec![DtuType::PagerDuty];
        })
        .build()
        .await
        .expect("harness build must succeed");

    let addr = harness
        .endpoint_for("pd-unprocessable", DtuType::PagerDuty)
        .expect("pagerduty endpoint must exist");
    let admin_token = harness
        .admin_token_for("pd-unprocessable", DtuType::PagerDuty)
        .expect("pagerduty admin token must exist")
        .to_owned();
    let client = test_client();

    configure_failure(&client, addr, &admin_token, json!({"unprocessable_at": 1})).await;

    let status = pd_enqueue(&client, addr).await;
    assert_eq!(
        status, 422,
        "PagerDuty post_enqueue request 1 must return 422 under Unprocessable(at_n=1) \
         (BC-3.6.001 Postcondition 1 / Invariant 5)"
    );
}

// ---------------------------------------------------------------------------
// PAGERDUTY — InternalError (regression guard)
// ---------------------------------------------------------------------------

/// BC-3.6.001 regression: InternalError still honored after Phase 2 refactor.
#[tokio::test]
async fn test_BC_3_6_001_pagerduty_internal_error_honored() {
    let harness = prism_dtu_harness::Harness::builder()
        .isolation(IsolationMode::Logical)
        .with_customer_overrides("pd-internal-error", |spec| {
            spec.dtu_types = vec![DtuType::PagerDuty];
        })
        .build()
        .await
        .expect("harness build must succeed");

    let addr = harness
        .endpoint_for("pd-internal-error", DtuType::PagerDuty)
        .expect("pagerduty endpoint must exist");
    let admin_token = harness
        .admin_token_for("pd-internal-error", DtuType::PagerDuty)
        .expect("pagerduty admin token must exist")
        .to_owned();
    let client = test_client();

    configure_failure(&client, addr, &admin_token, json!({"internal_error_at": 1})).await;

    let status = pd_enqueue(&client, addr).await;
    assert_eq!(
        status, 500,
        "PagerDuty post_enqueue request 1 must return 500 under InternalError(at_n=1) \
         (BC-3.6.001 Postcondition 1 regression guard)"
    );
}

// ---------------------------------------------------------------------------
// PAGERDUTY — RateLimit (regression guard)
// ---------------------------------------------------------------------------

/// BC-3.6.001 regression: RateLimit still honored after Phase 2 refactor.
#[tokio::test]
async fn test_BC_3_6_001_pagerduty_rate_limit_honored() {
    let harness = prism_dtu_harness::Harness::builder()
        .isolation(IsolationMode::Logical)
        .with_customer_overrides("pd-rate-limit", |spec| {
            spec.dtu_types = vec![DtuType::PagerDuty];
        })
        .build()
        .await
        .expect("harness build must succeed");

    let addr = harness
        .endpoint_for("pd-rate-limit", DtuType::PagerDuty)
        .expect("pagerduty endpoint must exist");
    let admin_token = harness
        .admin_token_for("pd-rate-limit", DtuType::PagerDuty)
        .expect("pagerduty admin token must exist")
        .to_owned();
    let client = test_client();

    configure_failure(
        &client,
        addr,
        &admin_token,
        json!({"rate_limit_after": 1, "retry_after_secs": 60}),
    )
    .await;

    let s1 = pd_enqueue(&client, addr).await;
    assert!(
        s1 == 202 || s1 == 200,
        "Request 1 must succeed before rate-limit threshold; got {s1}"
    );

    let s2 = pd_enqueue(&client, addr).await;
    assert_eq!(
        s2, 429,
        "PagerDuty post_enqueue request 2 must return 429 under RateLimit(after_n=1) \
         (BC-3.6.001 Postcondition 1 regression guard)"
    );
}

// ---------------------------------------------------------------------------
// SLACK — AuthReject
// ---------------------------------------------------------------------------

/// BC-3.6.001 Postcondition 1: AuthReject → HTTP 401 on post_webhook.
#[tokio::test]
async fn test_BC_3_6_001_slack_auth_reject_honored() {
    let harness = prism_dtu_harness::Harness::builder()
        .isolation(IsolationMode::Logical)
        .with_customer_overrides("slack-tenant", |spec| {
            spec.dtu_types = vec![DtuType::Slack];
        })
        .build()
        .await
        .expect("harness build must succeed");

    let addr = harness
        .endpoint_for("slack-tenant", DtuType::Slack)
        .expect("slack endpoint must exist");
    let admin_token = harness
        .admin_token_for("slack-tenant", DtuType::Slack)
        .expect("slack admin token must exist")
        .to_owned();
    let client = test_client();

    configure_failure(&client, addr, &admin_token, json!({"auth_mode": "reject"})).await;

    let status = slack_webhook(&client, addr).await;
    assert_eq!(
        status, 401,
        "Slack post_webhook must return 401 under AuthReject \
         (BC-3.6.001 Postcondition 1 / Invariant 5)"
    );
}

// ---------------------------------------------------------------------------
// SLACK — NetworkTimeout
// ---------------------------------------------------------------------------

/// BC-3.6.001 Postcondition 1: NetworkTimeout → delayed response on post_webhook.
#[tokio::test]
async fn test_BC_3_6_001_slack_network_timeout_honored() {
    let harness = prism_dtu_harness::Harness::builder()
        .isolation(IsolationMode::Logical)
        .with_customer_overrides("slack-timeout", |spec| {
            spec.dtu_types = vec![DtuType::Slack];
        })
        .build()
        .await
        .expect("harness build must succeed");

    let addr = harness
        .endpoint_for("slack-timeout", DtuType::Slack)
        .expect("slack endpoint must exist");
    let admin_token = harness
        .admin_token_for("slack-timeout", DtuType::Slack)
        .expect("slack admin token must exist")
        .to_owned();

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .expect("client build");

    configure_failure(
        &client,
        addr,
        &admin_token,
        json!({"network_timeout_ms": 200}),
    )
    .await;

    let start = Instant::now();
    let status = slack_webhook(&client, addr).await;
    let elapsed = start.elapsed();

    assert!(
        elapsed >= Duration::from_millis(150),
        "Slack post_webhook must be delayed by at least 150ms under NetworkTimeout(200ms); \
         elapsed={elapsed:?} (BC-3.6.001 Postcondition 1 / Invariant 5)"
    );
    assert!(
        status == 200,
        "Slack post_webhook must succeed after delay; got {status}"
    );
}

// ---------------------------------------------------------------------------
// SLACK — MalformedResponse
// ---------------------------------------------------------------------------

/// BC-3.6.001 Postcondition 1: MalformedResponse → body is NOT valid JSON on post_webhook.
#[tokio::test]
async fn test_BC_3_6_001_slack_malformed_response_honored() {
    let harness = prism_dtu_harness::Harness::builder()
        .isolation(IsolationMode::Logical)
        .with_customer_overrides("slack-malformed", |spec| {
            spec.dtu_types = vec![DtuType::Slack];
        })
        .build()
        .await
        .expect("harness build must succeed");

    let addr = harness
        .endpoint_for("slack-malformed", DtuType::Slack)
        .expect("slack endpoint must exist");
    let admin_token = harness
        .admin_token_for("slack-malformed", DtuType::Slack)
        .expect("slack admin token must exist")
        .to_owned();
    let client = test_client();

    configure_failure(
        &client,
        addr,
        &admin_token,
        json!({"malformed_response": true}),
    )
    .await;

    let resp = slack_webhook_resp(&client, addr).await;
    let body_bytes = resp.bytes().await.expect("body must be readable");
    let parse_result = serde_json::from_slice::<serde_json::Value>(&body_bytes);
    assert!(
        parse_result.is_err(),
        "Slack post_webhook body must be invalid JSON under MalformedResponse; \
         got valid JSON (BC-3.6.001 Postcondition 1 / Invariant 5)"
    );
}

// ---------------------------------------------------------------------------
// SLACK — Unprocessable
// ---------------------------------------------------------------------------

/// BC-3.6.001 Postcondition 1: Unprocessable → HTTP 422 at at_request_n on post_webhook.
#[tokio::test]
async fn test_BC_3_6_001_slack_unprocessable_honored() {
    let harness = prism_dtu_harness::Harness::builder()
        .isolation(IsolationMode::Logical)
        .with_customer_overrides("slack-unprocessable", |spec| {
            spec.dtu_types = vec![DtuType::Slack];
        })
        .build()
        .await
        .expect("harness build must succeed");

    let addr = harness
        .endpoint_for("slack-unprocessable", DtuType::Slack)
        .expect("slack endpoint must exist");
    let admin_token = harness
        .admin_token_for("slack-unprocessable", DtuType::Slack)
        .expect("slack admin token must exist")
        .to_owned();
    let client = test_client();

    configure_failure(&client, addr, &admin_token, json!({"unprocessable_at": 1})).await;

    let status = slack_webhook(&client, addr).await;
    assert_eq!(
        status, 422,
        "Slack post_webhook request 1 must return 422 under Unprocessable(at_n=1) \
         (BC-3.6.001 Postcondition 1 / Invariant 5)"
    );
}

// ---------------------------------------------------------------------------
// SLACK — InternalError (regression guard)
// ---------------------------------------------------------------------------

/// BC-3.6.001 regression: InternalError still honored after Phase 2 refactor.
#[tokio::test]
async fn test_BC_3_6_001_slack_internal_error_honored() {
    let harness = prism_dtu_harness::Harness::builder()
        .isolation(IsolationMode::Logical)
        .with_customer_overrides("slack-internal-error", |spec| {
            spec.dtu_types = vec![DtuType::Slack];
        })
        .build()
        .await
        .expect("harness build must succeed");

    let addr = harness
        .endpoint_for("slack-internal-error", DtuType::Slack)
        .expect("slack endpoint must exist");
    let admin_token = harness
        .admin_token_for("slack-internal-error", DtuType::Slack)
        .expect("slack admin token must exist")
        .to_owned();
    let client = test_client();

    configure_failure(&client, addr, &admin_token, json!({"internal_error_at": 1})).await;

    let status = slack_webhook(&client, addr).await;
    assert_eq!(
        status, 500,
        "Slack post_webhook request 1 must return 500 under InternalError(at_n=1) \
         (BC-3.6.001 Postcondition 1 regression guard)"
    );
}

// ---------------------------------------------------------------------------
// SLACK — RateLimit (regression guard)
// ---------------------------------------------------------------------------

/// BC-3.6.001 regression: RateLimit still honored after Phase 2 refactor.
#[tokio::test]
async fn test_BC_3_6_001_slack_rate_limit_honored() {
    let harness = prism_dtu_harness::Harness::builder()
        .isolation(IsolationMode::Logical)
        .with_customer_overrides("slack-rate-limit", |spec| {
            spec.dtu_types = vec![DtuType::Slack];
        })
        .build()
        .await
        .expect("harness build must succeed");

    let addr = harness
        .endpoint_for("slack-rate-limit", DtuType::Slack)
        .expect("slack endpoint must exist");
    let admin_token = harness
        .admin_token_for("slack-rate-limit", DtuType::Slack)
        .expect("slack admin token must exist")
        .to_owned();
    let client = test_client();

    configure_failure(
        &client,
        addr,
        &admin_token,
        json!({"rate_limit_after": 1, "retry_after_secs": 60}),
    )
    .await;

    let s1 = slack_webhook(&client, addr).await;
    assert_eq!(
        s1, 200,
        "Request 1 must succeed before rate-limit threshold"
    );

    let s2 = slack_webhook(&client, addr).await;
    assert_eq!(
        s2, 429,
        "Slack post_webhook request 2 must return 429 under RateLimit(after_n=1) \
         (BC-3.6.001 Postcondition 1 regression guard)"
    );
}

// ---------------------------------------------------------------------------
// VP-157: Postcondition-5 — HTTP 400 with exact unsupported_failure_mode body
//
// BC-3.6.001 v0.6 Postcondition 5: POST /dtu/configure with an unsupported
// failure mode returns HTTP 400 with body EXACTLY:
//   {"error":"unsupported_failure_mode","mode":"<variant-name>"}
//
// Trigger mechanism: `auth_mode` is a known field accepted by serde
// (so deny_unknown_fields does NOT fire), but the value "FutureVariant"
// is not recognized by the configure mapper. The mapper therefore returns
// an Err with the unrecognized mode name, and the handler returns the
// Postcondition-5 shape.
//
// This is distinct from the serde-level deny_unknown_fields rejection
// (tested in review_2026_06_10_deny_unknown.rs), which returns
// {"error":"invalid configure payload: ..."} for truly unknown field names.
//
// VP-157: POST /dtu/configure with unsupported mode returns HTTP 400 with
//         {"error":"unsupported_failure_mode","mode":"<variant>"} and leaves
//         clone state unchanged (BC-3.6.001 v0.6 Postcondition 5 / EC-008 / EC-009).
// ---------------------------------------------------------------------------

/// VP-157: Jira /dtu/configure returns exact Postcondition-5 body for unsupported auth_mode.
/// Trigger: structurally-valid payload (`auth_mode` is a known field) with unrecognized
/// value → mapper Err path → HTTP 400 + `{"error":"unsupported_failure_mode","mode":"FutureVariant"}`.
#[tokio::test]
async fn test_BC_3_6_001_jira_unsupported_mode_returns_400() {
    let harness = prism_dtu_harness::Harness::builder()
        .isolation(IsolationMode::Logical)
        .with_customer_overrides("jira-unsupported", |spec| {
            spec.dtu_types = vec![DtuType::Jira];
        })
        .build()
        .await
        .expect("harness build must succeed");

    let addr = harness
        .endpoint_for("jira-unsupported", DtuType::Jira)
        .expect("jira endpoint must exist");
    let admin_token = harness
        .admin_token_for("jira-unsupported", DtuType::Jira)
        .expect("jira admin token must exist")
        .to_owned();
    let client = test_client();

    // Structurally-valid payload: `auth_mode` is a known field, but value is
    // not in {"reject","none"} — triggers the Postcondition-5 unsupported-mode path.
    let resp = client
        .post(format!("http://{addr}/dtu/configure"))
        .header("x-admin-token", &admin_token)
        .json(&json!({"auth_mode": "FutureVariant"}))
        .send()
        .await
        .expect("configure request must not fail at transport level");

    let status = resp.status().as_u16();
    assert_eq!(
        status, 400,
        "Jira /dtu/configure must return 400 for unsupported auth_mode \
         (VP-157 / BC-3.6.001 Postcondition 5 / EC-008)"
    );

    let body: serde_json::Value = resp.json().await.expect("400 body must be JSON");
    assert_eq!(
        body,
        json!({"error": "unsupported_failure_mode", "mode": "FutureVariant"}),
        "400 response must match exact Postcondition-5 shape; body={body}"
    );
}

/// VP-157: PagerDuty /dtu/configure returns exact Postcondition-5 body for unsupported auth_mode.
#[tokio::test]
async fn test_BC_3_6_001_pagerduty_unsupported_mode_returns_400() {
    let harness = prism_dtu_harness::Harness::builder()
        .isolation(IsolationMode::Logical)
        .with_customer_overrides("pd-unsupported", |spec| {
            spec.dtu_types = vec![DtuType::PagerDuty];
        })
        .build()
        .await
        .expect("harness build must succeed");

    let addr = harness
        .endpoint_for("pd-unsupported", DtuType::PagerDuty)
        .expect("pagerduty endpoint must exist");
    let admin_token = harness
        .admin_token_for("pd-unsupported", DtuType::PagerDuty)
        .expect("pagerduty admin token must exist")
        .to_owned();
    let client = test_client();

    let resp = client
        .post(format!("http://{addr}/dtu/configure"))
        .header("x-admin-token", &admin_token)
        .json(&json!({"auth_mode": "FutureVariant"}))
        .send()
        .await
        .expect("configure request must not fail at transport level");

    let status = resp.status().as_u16();
    assert_eq!(
        status, 400,
        "PagerDuty /dtu/configure must return 400 for unsupported auth_mode \
         (VP-157 / BC-3.6.001 Postcondition 5 / EC-008)"
    );

    let body: serde_json::Value = resp.json().await.expect("400 body must be JSON");
    assert_eq!(
        body,
        json!({"error": "unsupported_failure_mode", "mode": "FutureVariant"}),
        "400 response must match exact Postcondition-5 shape; body={body}"
    );
}

/// VP-157: Slack /dtu/configure returns exact Postcondition-5 body for unsupported auth_mode.
#[tokio::test]
async fn test_BC_3_6_001_slack_unsupported_mode_returns_400() {
    let harness = prism_dtu_harness::Harness::builder()
        .isolation(IsolationMode::Logical)
        .with_customer_overrides("slack-unsupported", |spec| {
            spec.dtu_types = vec![DtuType::Slack];
        })
        .build()
        .await
        .expect("harness build must succeed");

    let addr = harness
        .endpoint_for("slack-unsupported", DtuType::Slack)
        .expect("slack endpoint must exist");
    let admin_token = harness
        .admin_token_for("slack-unsupported", DtuType::Slack)
        .expect("slack admin token must exist")
        .to_owned();
    let client = test_client();

    let resp = client
        .post(format!("http://{addr}/dtu/configure"))
        .header("x-admin-token", &admin_token)
        .json(&json!({"auth_mode": "FutureVariant"}))
        .send()
        .await
        .expect("configure request must not fail at transport level");

    let status = resp.status().as_u16();
    assert_eq!(
        status, 400,
        "Slack /dtu/configure must return 400 for unsupported auth_mode \
         (VP-157 / BC-3.6.001 Postcondition 5 / EC-008)"
    );

    let body: serde_json::Value = resp.json().await.expect("400 body must be JSON");
    assert_eq!(
        body,
        json!({"error": "unsupported_failure_mode", "mode": "FutureVariant"}),
        "400 response must match exact Postcondition-5 shape; body={body}"
    );
}
