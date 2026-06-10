//! F10 / finding ⑫ (2026-06-10 review): `/dtu/configure` strict payload schema.
//!
//! The jira / pagerduty / slack harness clones' `HarnessConfigure` structs must
//! carry `#[serde(deny_unknown_fields)]` — an unknown field in the configure
//! payload is a harness-side bug (typo'd failure-mode key silently ignored →
//! test exercises the wrong behavior). Human overrode the prior "deliberately
//! permissive" stance on 2026-06-10.
//!
//! Each test posts:
//! 1. a payload with an unknown field → HTTP 400 (strict rejection)
//! 2. a known-good payload → HTTP 200 (non-regression: every field
//!    `failure_mode_to_json` can emit remains accepted)
//!
//! Idiom: reqwest-over-TcpListener per tests/logical_isolation_test.rs.

#![allow(clippy::expect_used, non_snake_case)]

use std::time::Duration;

use prism_dtu_harness::{DtuType, IsolationMode};

fn test_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .expect("test client build must succeed")
}

/// Drive the unknown-field rejection + known-field acceptance pair for one clone type.
async fn assert_configure_strict(dtu_type: DtuType) {
    let harness = prism_dtu_harness::Harness::builder()
        .isolation(IsolationMode::Logical)
        .with_customer_overrides("test-tenant", |spec| {
            spec.dtu_types = vec![dtu_type];
        })
        .build()
        .await
        .expect("harness build must succeed");

    let addr = harness
        .endpoint_for("test-tenant", dtu_type)
        .unwrap_or_else(|| panic!("no endpoint for {dtu_type:?}"));
    let admin_token = harness
        .admin_token_for("test-tenant", dtu_type)
        .unwrap_or_else(|| panic!("no admin token for {dtu_type:?}"))
        .to_owned();
    let client = test_client();

    // 1. Unknown field → 400 (deny_unknown_fields).
    let resp = client
        .post(format!("http://{addr}/dtu/configure"))
        .header("x-admin-token", &admin_token)
        .json(&serde_json::json!({ "rate_limit_atfer": 3 })) // typo'd key
        .send()
        .await
        .expect("configure request must not fail at transport level");
    assert_eq!(
        resp.status().as_u16(),
        400,
        "{dtu_type:?} /dtu/configure must reject unknown payload fields with 400 \
         (deny_unknown_fields — finding ⑫, 2026-06-10 review)"
    );

    // 2. Known payload → 200 (non-regression for the failure_mode_to_json surface).
    let resp = client
        .post(format!("http://{addr}/dtu/configure"))
        .header("x-admin-token", &admin_token)
        .json(&serde_json::json!({ "rate_limit_after": 3, "retry_after_secs": 60 }))
        .send()
        .await
        .expect("configure request must not fail at transport level");
    assert_eq!(
        resp.status().as_u16(),
        200,
        "{dtu_type:?} /dtu/configure must still accept known fields"
    );

    // 3. clear → 200 (restore clean state; also covers the "clear" field).
    let resp = client
        .post(format!("http://{addr}/dtu/configure"))
        .header("x-admin-token", &admin_token)
        .json(&serde_json::json!({ "clear": true }))
        .send()
        .await
        .expect("configure request must not fail at transport level");
    assert_eq!(
        resp.status().as_u16(),
        200,
        "{dtu_type:?} clear must succeed"
    );
}

#[tokio::test]
async fn test_f10_jira_configure_rejects_unknown_fields() {
    assert_configure_strict(DtuType::Jira).await;
}

#[tokio::test]
async fn test_f10_pagerduty_configure_rejects_unknown_fields() {
    assert_configure_strict(DtuType::PagerDuty).await;
}

#[tokio::test]
async fn test_f10_slack_configure_rejects_unknown_fields() {
    assert_configure_strict(DtuType::Slack).await;
}
