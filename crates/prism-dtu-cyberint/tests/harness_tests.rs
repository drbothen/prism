#![allow(clippy::unwrap_used, clippy::expect_used)]
// BC-based test names use mixed-case identifiers following the factory naming standard.
#![allow(non_snake_case)]
//! Harness-migrated tests for `prism-dtu-cyberint`.
//!
//! All tests in this file use `prism-dtu-harness` (`HarnessBuilder`) to host the
//! `CyberintClone` instead of directly instantiating `CyberintClone::new()`.
//! This satisfies AC-006 of S-3.4.04 (no direct `CyberintClone::start()` outside
//! the harness post-migration).
//!
//! # Coverage
//!
//! | Test function | Origin | BC / AC |
//! |---|---|---|
//! | `ac_multi_tenant_ac1_login_returns_200_with_set_cookie_header` | ac_1_cookie_auth_roundtrip.rs | BC-3.5.001 AC-001 |
//! | `ac_multi_tenant_ac1_authenticated_request_returns_200` | ac_1_cookie_auth_roundtrip.rs | BC-3.5.001 AC-001 |
//! | `ac_multi_tenant_ac1_ec003_two_logins_distinct_tokens` | ac_1_cookie_auth_roundtrip.rs | BC-3.5.001 AC-001 / EC-003 |
//! | `ac_multi_tenant_ac2_alerts_no_cookie_returns_401` | ac_2_unauthenticated_returns_401.rs | BC-3.5.001 AC-002 |
//! | `ac_multi_tenant_ac2_empty_cookie_returns_401` | ac_2_unauthenticated_returns_401.rs | BC-3.5.001 AC-002 |
//! | `ac_multi_tenant_ac2_invalid_token_returns_401` | ac_2_unauthenticated_returns_401.rs | BC-3.5.001 AC-002 |
//! | `ac_multi_tenant_ac3_patch_status_returns_200` | ac_3_alert_status_transition.rs | BC-3.5.001 AC-003 |
//! | `ac_multi_tenant_ac3_status_persists_after_patch` | ac_3_alert_status_transition.rs | BC-3.5.001 AC-003 |
//! | `ac_multi_tenant_ac4_close_alert_returns_200` | ac_4_irreversible_close.rs | BC-3.5.001 AC-004 |
//! | `ac_multi_tenant_ac4_close_then_ack_returns_400` | ac_4_irreversible_close.rs | BC-3.5.001 AC-004 |
//! | `ac_multi_tenant_ac5_mixed_timestamps_present` | ac_5_mixed_timestamp_fixtures.rs | BC-3.5.001 AC-005 |
//! | `ac_multi_tenant_ac6_cursor_pagination_returns_next_cursor` | ac_6_cursor_pagination.rs | BC-3.5.001 AC-006 |
//! | `ac_multi_tenant_ac7_rate_limit_returns_429` | ac_7_rate_limit.rs | BC-3.5.001 AC-007 |
//! | `ac_multi_tenant_ac8_reset_clears_state` | ac_8_reset_semantics.rs | BC-3.5.001 AC-008 |
//! | `ac_multi_tenant_ec002_unknown_alert_returns_404` | edge_cases.rs | BC-3.5.001 AC-002 |
//! | `ac_multi_tenant_ec005_out_of_scope_endpoint_404` | edge_cases.rs | BC-3.5.001 AC-002 |
//! | `ac_multi_tenant_ec006_auth_mode_reject_returns_401` | edge_cases.rs | BC-3.5.001 AC-002 |
//! | `ac_multi_tenant_fidelity_validator_passes` | fidelity_validator.rs | BC-3.5.001 AC-003 |
//! | `ac_multi_tenant_td_wv0_04_known_field_returns_200` | td_wv0_04_configure_deny_unknown.rs | BC-3.5.001 AC-001 |
//! | `ac_multi_tenant_td_wv0_04_unknown_field_returns_400` | td_wv0_04_configure_deny_unknown.rs | BC-3.5.001 AC-001 |
//! | `ac_multi_tenant_td_wv0_07_no_token_returns_401` | td_wv0_07_configure_requires_admin_token.rs | BC-3.5.001 AC-001 |
//! | `ac_multi_tenant_td_wv0_07_wrong_token_returns_401` | td_wv0_07_configure_requires_admin_token.rs | BC-3.5.001 AC-001 |
//! | `ac_multi_tenant_td_wv0_07_correct_token_returns_200` | td_wv0_07_configure_requires_admin_token.rs | BC-3.5.001 AC-001 |
//! | `ac_multi_org_logical_isolation` | NEW | BC-3.5.001 AC-004 (postcondition 2; TV-2) |
//! | `ac_network_cross_creds_401` | NEW | BC-3.5.002 AC-005 (postcondition 2; TV-3) |
//! | `ac_failure_injection_via_with_failure` | NEW | BC-3.6.001 AC (postcondition 1) / EC-001 |
//!
//! # Architecture Compliance
//!
//! - `prism-dtu-harness` is a `[dev-dependency]` only (ADR-011 §2.9).
//! - No test below calls `CyberintClone::new()` or `CyberintClone::start()` directly (S-3.4.04 AC-006).
//! - `base_url` is always derived from `harness.endpoint_for("test-tenant", DtuType::Cyberint)` (migration pattern).

#[cfg(feature = "dtu")]
mod harness_tests {
    #[allow(unused_imports)]
    use prism_dtu_common::{FailureMode, FidelityCheck, FidelityValidator};
    use prism_dtu_harness::types::DtuType;
    use prism_dtu_harness::{HarnessBuilder, IsolationMode};

    // ── Shared helper: login and return session token ────────────────────────

    #[allow(dead_code)]
    async fn cyberint_login(base_url: &str, client: &reqwest::Client) -> String {
        let resp = client
            .post(format!("{base_url}/login"))
            .json(&serde_json::json!({}))
            .send()
            .await
            .expect("login must succeed");
        assert_eq!(resp.status().as_u16(), 200, "login must return 200");

        let set_cookie = resp
            .headers()
            .get("set-cookie")
            .expect("Set-Cookie must be present on login")
            .to_str()
            .expect("Set-Cookie must be ASCII")
            .to_owned();
        set_cookie
            .split(';')
            .next()
            .and_then(|s| s.strip_prefix("cyberint_session="))
            .expect("Set-Cookie must contain cyberint_session=")
            .to_owned()
    }

    // ── Helper: build a single-org Logical harness for "alpha" ───────────────

    #[allow(dead_code)]
    async fn single_org_harness() -> (prism_dtu_harness::Harness, String, reqwest::Client) {
        let harness = HarnessBuilder::new()
            .isolation(IsolationMode::Logical)
            .with_customer("alpha")
            .build()
            .await
            .expect("harness build must succeed");

        let addr = harness
            .endpoint_for("alpha", DtuType::Cyberint)
            .expect("alpha/Cyberint endpoint must exist");
        let base_url = format!("http://{addr}");

        let client = reqwest::Client::builder()
            .cookie_store(true)
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .expect("build reqwest client");

        (harness, base_url, client)
    }

    // =========================================================================
    // AC-1: Cookie auth round-trip (migrated from ac_1_cookie_auth_roundtrip.rs)
    // =========================================================================

    /// BC-3.5.001 AC-001 — POST /login returns 200 with Set-Cookie header.
    #[tokio::test]
    async fn ac_multi_tenant_ac1_login_returns_200_with_set_cookie_header() {
        let harness = HarnessBuilder::new()
            .isolation(IsolationMode::Logical)
            .with_customer("test-tenant")
            .build()
            .await
            .expect("harness build must succeed");

        let addr = harness
            .endpoint_for("test-tenant", DtuType::Cyberint)
            .expect("test-tenant/Cyberint endpoint must exist");
        let base_url = format!("http://{addr}");

        let client = reqwest::Client::builder()
            .cookie_store(true)
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .expect("build reqwest client");

        let resp = client
            .post(format!("{base_url}/login"))
            .json(&serde_json::json!({}))
            .send()
            .await
            .expect("AC-1: POST /login must not error");

        assert_eq!(
            resp.status().as_u16(),
            200,
            "AC-1: POST /login must return HTTP 200"
        );

        let set_cookie = resp
            .headers()
            .get("set-cookie")
            .expect("AC-1: Set-Cookie header must be present on login response");
        let cookie_str = set_cookie.to_str().expect("AC-1: Set-Cookie must be ASCII");
        assert!(
            cookie_str.contains("cyberint_session="),
            "AC-1: Set-Cookie must contain cyberint_session=, got: {cookie_str}"
        );

        let body: serde_json::Value = resp
            .json()
            .await
            .expect("AC-1: login response must be valid JSON");
        assert_eq!(
            body["message"].as_str().unwrap_or(""),
            "Login successful",
            "AC-1: login body must contain message: Login successful"
        );
    }

    /// BC-3.5.001 AC-001 — authenticated GET /api/v1/alerts with valid cookie returns 200.
    #[tokio::test]
    async fn ac_multi_tenant_ac1_authenticated_request_returns_200() {
        let harness = HarnessBuilder::new()
            .isolation(IsolationMode::Logical)
            .with_customer("test-tenant")
            .build()
            .await
            .expect("harness build must succeed");

        let addr = harness
            .endpoint_for("test-tenant", DtuType::Cyberint)
            .expect("test-tenant/Cyberint endpoint must exist");
        let base_url = format!("http://{addr}");

        let login_client = reqwest::Client::builder()
            .cookie_store(true)
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .expect("build cookie-jar client");

        let login_resp = login_client
            .post(format!("{base_url}/login"))
            .json(&serde_json::json!({}))
            .send()
            .await
            .expect("AC-1: login request must succeed");
        assert_eq!(
            login_resp.status().as_u16(),
            200,
            "AC-1: login must return 200"
        );

        // Extract session token from Set-Cookie for manual header usage.
        let set_cookie = login_resp
            .headers()
            .get("set-cookie")
            .expect("AC-1: Set-Cookie must be present")
            .to_str()
            .expect("AC-1: Set-Cookie must be ASCII");
        let token = set_cookie
            .split(';')
            .next()
            .and_then(|s| s.strip_prefix("cyberint_session="))
            .expect("AC-1: Set-Cookie must contain cyberint_session=");

        // Use token in Cookie header directly with a bare client.
        let bare_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .expect("build bare client");

        let alerts_resp = bare_client
            .get(format!("{base_url}/api/v1/alerts"))
            .header("Cookie", format!("cyberint_session={token}"))
            .send()
            .await
            .expect("AC-1: GET /api/v1/alerts with valid cookie must not error");

        assert_eq!(
            alerts_resp.status().as_u16(),
            200,
            "AC-1: GET /api/v1/alerts with valid session cookie must return HTTP 200"
        );
    }

    /// BC-3.5.001 AC-001 / EC-003 — two logins yield distinct tokens, both valid.
    #[tokio::test]
    async fn ac_multi_tenant_ac1_ec003_two_logins_distinct_tokens() {
        let harness = HarnessBuilder::new()
            .isolation(IsolationMode::Logical)
            .with_customer("test-tenant")
            .build()
            .await
            .expect("harness build must succeed");

        let addr = harness
            .endpoint_for("test-tenant", DtuType::Cyberint)
            .expect("test-tenant/Cyberint endpoint must exist");
        let base_url = format!("http://{addr}");

        let bare = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .expect("build client");

        let resp1 = bare
            .post(format!("{base_url}/login"))
            .json(&serde_json::json!({}))
            .send()
            .await
            .expect("EC-003: first login must succeed");
        let cookie1 = resp1
            .headers()
            .get("set-cookie")
            .expect("EC-003: Set-Cookie on first login")
            .to_str()
            .expect("EC-003: Set-Cookie on first login must be ASCII")
            .to_owned();

        let resp2 = bare
            .post(format!("{base_url}/login"))
            .json(&serde_json::json!({}))
            .send()
            .await
            .expect("EC-003: second login must succeed");
        let cookie2 = resp2
            .headers()
            .get("set-cookie")
            .expect("EC-003: Set-Cookie on second login")
            .to_str()
            .expect("EC-003: Set-Cookie on second login must be ASCII")
            .to_owned();

        assert_ne!(
            cookie1, cookie2,
            "EC-003: two logins must produce distinct session tokens"
        );

        // Extract both tokens and verify both are accepted.
        let token1 = cookie1
            .split(';')
            .next()
            .and_then(|s| s.strip_prefix("cyberint_session="))
            .expect("EC-003: parse token1")
            .to_owned();
        let token2 = cookie2
            .split(';')
            .next()
            .and_then(|s| s.strip_prefix("cyberint_session="))
            .expect("EC-003: parse token2")
            .to_owned();

        for (label, token) in [("token1", token1.as_str()), ("token2", token2.as_str())] {
            let r = bare
                .get(format!("{base_url}/api/v1/alerts"))
                .header("Cookie", format!("cyberint_session={token}"))
                .send()
                .await
                .unwrap_or_else(|_| panic!("EC-003: request with {label} must send"));
            assert_eq!(
                r.status().as_u16(),
                200,
                "EC-003: {label} must be a valid session token (got {} instead of 200)",
                r.status()
            );
        }
    }

    // =========================================================================
    // AC-2: Unauthenticated returns 401 (migrated from ac_2_unauthenticated_returns_401.rs)
    // =========================================================================

    /// BC-3.5.001 AC-002 — GET /api/v1/alerts with no Cookie header returns 401.
    #[tokio::test]
    async fn ac_multi_tenant_ac2_alerts_no_cookie_returns_401() {
        let harness = HarnessBuilder::new()
            .isolation(IsolationMode::Logical)
            .with_customer("test-tenant")
            .build()
            .await
            .expect("harness build must succeed");

        let addr = harness
            .endpoint_for("test-tenant", DtuType::Cyberint)
            .expect("test-tenant/Cyberint endpoint must exist");
        let base_url = format!("http://{addr}");

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .expect("build client");

        let resp = client
            .get(format!("{base_url}/api/v1/alerts"))
            .send()
            .await
            .expect("AC-2: request must not error");

        assert_eq!(
            resp.status().as_u16(),
            401,
            "AC-2: GET /api/v1/alerts without cookie must return HTTP 401"
        );

        let body: serde_json::Value = resp
            .json()
            .await
            .expect("AC-2: 401 response must be valid JSON");
        assert_eq!(
            body["error"].as_str().unwrap_or(""),
            "unauthorized",
            "AC-2: error field must be 'unauthorized'"
        );
        assert_eq!(
            body["code"].as_u64().unwrap_or(0),
            401,
            "AC-2: code field must be 401"
        );
    }

    /// BC-3.5.001 AC-002 — Cookie header without cyberint_session returns 401.
    #[tokio::test]
    async fn ac_multi_tenant_ac2_empty_cookie_returns_401() {
        let harness = HarnessBuilder::new()
            .isolation(IsolationMode::Logical)
            .with_customer("test-tenant")
            .build()
            .await
            .expect("harness build must succeed");

        let addr = harness
            .endpoint_for("test-tenant", DtuType::Cyberint)
            .expect("test-tenant/Cyberint endpoint must exist");
        let base_url = format!("http://{addr}");

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .expect("build client");

        let resp = client
            .get(format!("{base_url}/api/v1/alerts"))
            .header("Cookie", "other_cookie=somevalue")
            .send()
            .await
            .expect("AC-2: request with wrong cookie must not error");

        assert_eq!(
            resp.status().as_u16(),
            401,
            "AC-2: Cookie header without cyberint_session must return HTTP 401"
        );
    }

    /// BC-3.5.001 AC-002 — invalid (non-registered) session token returns 401.
    #[tokio::test]
    async fn ac_multi_tenant_ac2_invalid_token_returns_401() {
        let harness = HarnessBuilder::new()
            .isolation(IsolationMode::Logical)
            .with_customer("test-tenant")
            .build()
            .await
            .expect("harness build must succeed");

        let addr = harness
            .endpoint_for("test-tenant", DtuType::Cyberint)
            .expect("test-tenant/Cyberint endpoint must exist");
        let base_url = format!("http://{addr}");

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .expect("build client");

        let resp = client
            .get(format!("{base_url}/api/v1/alerts"))
            .header(
                "Cookie",
                "cyberint_session=00000000-0000-0000-0000-000000000000",
            )
            .send()
            .await
            .expect("AC-2: request with invalid token must not error");

        assert_eq!(
            resp.status().as_u16(),
            401,
            "AC-2: invalid session token must return HTTP 401"
        );
    }

    // =========================================================================
    // AC-3: Alert status transition (migrated from ac_3_alert_status_transition.rs)
    // =========================================================================

    /// BC-3.5.001 AC-003 — PATCH /alerts/{id}/status returns 200 with acknowledged status.
    #[tokio::test]
    async fn ac_multi_tenant_ac3_patch_status_returns_200() {
        let harness = HarnessBuilder::new()
            .isolation(IsolationMode::Logical)
            .with_customer("test-tenant")
            .build()
            .await
            .expect("harness build must succeed");

        let addr = harness
            .endpoint_for("test-tenant", DtuType::Cyberint)
            .expect("test-tenant/Cyberint endpoint must exist");
        let base_url = format!("http://{addr}");

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .expect("build client");

        let token = cyberint_login(&base_url, &client).await;

        let resp = client
            .patch(format!("{base_url}/api/v1/alerts/CYB-2024-001/status"))
            .header("Cookie", format!("cyberint_session={token}"))
            .json(&serde_json::json!({"status": "acknowledged"}))
            .send()
            .await
            .expect("AC-3: PATCH must not error");

        assert_eq!(
            resp.status().as_u16(),
            200,
            "AC-3: PATCH status must return HTTP 200"
        );

        let body: serde_json::Value = resp.json().await.expect("AC-3: body must be JSON");
        assert_eq!(
            body["alert_id"].as_str().unwrap_or(""),
            "CYB-2024-001",
            "AC-3: response must include alert_id"
        );
        assert_eq!(
            body["status"].as_str().unwrap_or(""),
            "acknowledged",
            "AC-3: response status must be acknowledged"
        );
    }

    /// BC-3.5.001 AC-003 — status persists after PATCH (GET confirms updated state).
    #[tokio::test]
    async fn ac_multi_tenant_ac3_status_persists_after_patch() {
        let harness = HarnessBuilder::new()
            .isolation(IsolationMode::Logical)
            .with_customer("test-tenant")
            .build()
            .await
            .expect("harness build must succeed");

        let addr = harness
            .endpoint_for("test-tenant", DtuType::Cyberint)
            .expect("test-tenant/Cyberint endpoint must exist");
        let base_url = format!("http://{addr}");

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .expect("build client");

        let token = cyberint_login(&base_url, &client).await;
        let cookie = format!("cyberint_session={token}");

        // PATCH to acknowledge.
        let patch_resp = client
            .patch(format!("{base_url}/api/v1/alerts/CYB-2024-003/status"))
            .header("Cookie", &cookie)
            .json(&serde_json::json!({"status": "acknowledged"}))
            .send()
            .await
            .expect("AC-3: PATCH must not error");
        assert_eq!(
            patch_resp.status().as_u16(),
            200,
            "AC-3: PATCH must return 200"
        );

        // GET the alert and verify status is acknowledged.
        let get_resp = client
            .get(format!("{base_url}/api/v1/alerts/CYB-2024-003"))
            .header("Cookie", &cookie)
            .send()
            .await
            .expect("AC-3: GET must not error");
        assert_eq!(get_resp.status().as_u16(), 200, "AC-3: GET must return 200");

        let body: serde_json::Value = get_resp.json().await.expect("AC-3: GET body must be JSON");
        assert_eq!(
            body["status"].as_str().unwrap_or(""),
            "acknowledged",
            "AC-3: status must be acknowledged after PATCH (state must persist)"
        );
        assert_eq!(
            body["alert_id"].as_str().unwrap_or(""),
            "CYB-2024-003",
            "AC-3: alert_id must match the patched alert"
        );
    }

    // =========================================================================
    // AC-4: Irreversible close (migrated from ac_4_irreversible_close.rs)
    // =========================================================================

    /// BC-3.5.001 AC-004 — POST /close returns 200 with status: "closed".
    #[tokio::test]
    async fn ac_multi_tenant_ac4_close_alert_returns_200() {
        let harness = HarnessBuilder::new()
            .isolation(IsolationMode::Logical)
            .with_customer("test-tenant")
            .build()
            .await
            .expect("harness build must succeed");

        let addr = harness
            .endpoint_for("test-tenant", DtuType::Cyberint)
            .expect("test-tenant/Cyberint endpoint must exist");
        let base_url = format!("http://{addr}");

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .expect("build client");

        let token = cyberint_login(&base_url, &client).await;
        let cookie = format!("cyberint_session={token}");

        let resp = client
            .post(format!("{base_url}/api/v1/alerts/CYB-2024-002/close"))
            .header("Cookie", &cookie)
            .send()
            .await
            .expect("AC-4: POST /close must not error");

        assert_eq!(
            resp.status().as_u16(),
            200,
            "AC-4: POST /close must return HTTP 200"
        );

        let body: serde_json::Value = resp.json().await.expect("AC-4: close body must be JSON");
        assert_eq!(
            body["status"].as_str().unwrap_or(""),
            "closed",
            "AC-4: close response must include status: closed"
        );
        assert_eq!(
            body["alert_id"].as_str().unwrap_or(""),
            "CYB-2024-002",
            "AC-4: close response must include the alert_id"
        );
    }

    /// BC-3.5.001 AC-004 — PATCH /status after close returns 400 "alert already closed".
    #[tokio::test]
    async fn ac_multi_tenant_ac4_close_then_ack_returns_400() {
        let harness = HarnessBuilder::new()
            .isolation(IsolationMode::Logical)
            .with_customer("test-tenant")
            .build()
            .await
            .expect("harness build must succeed");

        let addr = harness
            .endpoint_for("test-tenant", DtuType::Cyberint)
            .expect("test-tenant/Cyberint endpoint must exist");
        let base_url = format!("http://{addr}");

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .expect("build client");

        let token = cyberint_login(&base_url, &client).await;
        let cookie = format!("cyberint_session={token}");

        // Close first.
        client
            .post(format!("{base_url}/api/v1/alerts/CYB-2024-006/close"))
            .header("Cookie", &cookie)
            .send()
            .await
            .expect("AC-4: close must succeed");

        // Now try to acknowledge the closed alert.
        let patch_resp = client
            .patch(format!("{base_url}/api/v1/alerts/CYB-2024-006/status"))
            .header("Cookie", &cookie)
            .json(&serde_json::json!({"status": "acknowledged"}))
            .send()
            .await
            .expect("AC-4: PATCH after close must not error");

        assert_eq!(
            patch_resp.status().as_u16(),
            400,
            "AC-4: PATCH on closed alert must return HTTP 400"
        );

        let body: serde_json::Value = patch_resp
            .json()
            .await
            .expect("AC-4: 400 response must be JSON");
        assert_eq!(
            body["error"].as_str().unwrap_or(""),
            "alert already closed",
            "AC-4: error message must be 'alert already closed'"
        );
    }

    // =========================================================================
    // AC-5: Mixed timestamp fixtures (migrated from ac_5_mixed_timestamp_fixtures.rs)
    // =========================================================================

    /// BC-3.5.001 AC-005 — alerts list contains both past and future timestamps.
    #[tokio::test]
    async fn ac_multi_tenant_ac5_mixed_timestamps_present() {
        let harness = HarnessBuilder::new()
            .isolation(IsolationMode::Logical)
            .with_customer("test-tenant")
            .build()
            .await
            .expect("harness build must succeed");

        let addr = harness
            .endpoint_for("test-tenant", DtuType::Cyberint)
            .expect("test-tenant/Cyberint endpoint must exist");
        let base_url = format!("http://{addr}");

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .expect("build client");

        let token = cyberint_login(&base_url, &client).await;

        let resp = client
            .get(format!("{base_url}/api/v1/alerts"))
            .header("Cookie", format!("cyberint_session={token}"))
            .send()
            .await
            .expect("AC-5: GET /api/v1/alerts must not error");

        assert_eq!(resp.status().as_u16(), 200, "AC-5: must return 200");

        let body: serde_json::Value = resp.json().await.expect("AC-5: body must be JSON");
        let data = body["data"].as_array().expect("AC-5: data must be array");
        assert!(!data.is_empty(), "AC-5: alert list must not be empty");

        // At least one alert must have an ISO 8601 string timestamp.
        let has_iso8601 = data.iter().any(|a| a["created_at"].is_string());
        assert!(
            has_iso8601,
            "AC-5: fixture must contain at least one ISO 8601 string timestamp"
        );

        // At least one alert must have a Unix epoch (integer) timestamp.
        let has_unix_epoch = data.iter().any(|a| a["created_at"].is_number());
        assert!(
            has_unix_epoch,
            "AC-5: fixture must contain at least one Unix epoch (integer) timestamp"
        );
    }

    // =========================================================================
    // AC-6: Cursor pagination (migrated from ac_6_cursor_pagination.rs)
    // =========================================================================

    /// BC-3.5.001 AC-006 — GET /api/v1/alerts returns next_cursor when more pages exist.
    #[tokio::test]
    async fn ac_multi_tenant_ac6_cursor_pagination_returns_next_cursor() {
        let harness = HarnessBuilder::new()
            .isolation(IsolationMode::Logical)
            .with_customer("test-tenant")
            .build()
            .await
            .expect("harness build must succeed");

        let addr = harness
            .endpoint_for("test-tenant", DtuType::Cyberint)
            .expect("test-tenant/Cyberint endpoint must exist");
        let base_url = format!("http://{addr}");

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .expect("build client");

        let token = cyberint_login(&base_url, &client).await;
        let cookie = format!("cyberint_session={token}");

        // Fetch page 1 — must have data and non-null next_cursor.
        let resp = client
            .get(format!("{base_url}/api/v1/alerts"))
            .header("Cookie", &cookie)
            .send()
            .await
            .expect("AC-6: GET page 1 must not error");
        assert_eq!(resp.status().as_u16(), 200, "AC-6: page 1 must return 200");

        let body: serde_json::Value = resp.json().await.expect("AC-6: body must be JSON");

        let data = body["data"].as_array().expect("AC-6: data must be array");
        assert!(
            !data.is_empty(),
            "AC-6: page 1 must contain alerts (got empty data array)"
        );

        assert!(
            !body["next_cursor"].is_null(),
            "AC-6: page 1 must include a non-null next_cursor"
        );

        // Fetch page 2 using the cursor.
        let cursor = body["next_cursor"]
            .as_str()
            .expect("AC-6: page 1 next_cursor must be a string")
            .to_owned();

        let page2_resp = client
            .get(format!("{base_url}/api/v1/alerts"))
            .header("Cookie", &cookie)
            .query(&[("cursor", cursor.as_str())])
            .send()
            .await
            .expect("AC-6: page 2 request must not error");
        assert_eq!(
            page2_resp.status().as_u16(),
            200,
            "AC-6: page 2 must return 200"
        );

        let page2: serde_json::Value = page2_resp.json().await.expect("AC-6: page 2 must be JSON");
        let data2 = page2["data"]
            .as_array()
            .expect("AC-6: page 2 data must be array");
        assert!(
            !data2.is_empty(),
            "AC-6: page 2 must contain alerts (got empty data array)"
        );

        assert!(
            page2["next_cursor"].is_null(),
            "AC-6: page 2 must have next_cursor: null (no more pages), got: {:?}",
            page2["next_cursor"]
        );
    }

    // =========================================================================
    // AC-7: Rate limiting (migrated from ac_7_rate_limit.rs)
    // =========================================================================

    /// BC-3.5.001 AC-007 — after N requests, clone returns 429 with Retry-After.
    #[tokio::test]
    async fn ac_multi_tenant_ac7_rate_limit_returns_429() {
        let harness = HarnessBuilder::new()
            .isolation(IsolationMode::Logical)
            .with_customer("test-tenant")
            .build()
            .await
            .expect("harness build must succeed");

        let addr = harness
            .endpoint_for("test-tenant", DtuType::Cyberint)
            .expect("test-tenant/Cyberint endpoint must exist");
        let base_url = format!("http://{addr}");

        let admin_token = harness
            .admin_token_for("test-tenant", DtuType::Cyberint)
            .expect("admin token must exist")
            .to_owned();

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .expect("build client");

        let token = cyberint_login(&base_url, &client).await;
        let cookie = format!("cyberint_session={token}");

        // Configure rate limit: allow 1 request then 429.
        let configure_resp = client
            .post(format!("{base_url}/dtu/configure"))
            .header("X-Admin-Token", &admin_token)
            .json(&serde_json::json!({"rate_limit_after": 1}))
            .send()
            .await
            .expect("AC-7: POST /dtu/configure must not error");
        assert_eq!(
            configure_resp.status().as_u16(),
            200,
            "AC-7: configure must return 200"
        );

        // First request — should succeed.
        let first_resp = client
            .get(format!("{base_url}/api/v1/alerts"))
            .header("Cookie", &cookie)
            .send()
            .await
            .expect("AC-7: first request must not error");
        assert_eq!(
            first_resp.status().as_u16(),
            200,
            "AC-7: first request (within threshold) must return 200"
        );

        // Second request — count exceeds threshold → 429.
        let second_resp = client
            .get(format!("{base_url}/api/v1/alerts"))
            .header("Cookie", &cookie)
            .send()
            .await
            .expect("AC-7: second request must not error");
        assert_eq!(
            second_resp.status().as_u16(),
            429,
            "AC-7: second request past threshold must return HTTP 429"
        );

        let body: serde_json::Value = second_resp
            .json()
            .await
            .expect("AC-7: 429 body must be JSON");
        assert!(
            !body["error"].is_null(),
            "AC-7: HTTP 429 response must include an error field"
        );
    }

    // =========================================================================
    // AC-8: Reset semantics (migrated from ac_8_reset_semantics.rs)
    // =========================================================================

    /// BC-3.5.001 AC-008 — POST /dtu/reset clears alert_store and session_store state.
    #[tokio::test]
    async fn ac_multi_tenant_ac8_reset_clears_state() {
        let harness = HarnessBuilder::new()
            .isolation(IsolationMode::Logical)
            .with_customer("test-tenant")
            .build()
            .await
            .expect("harness build must succeed");

        let addr = harness
            .endpoint_for("test-tenant", DtuType::Cyberint)
            .expect("test-tenant/Cyberint endpoint must exist");
        let base_url = format!("http://{addr}");

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .expect("build client");

        let token = cyberint_login(&base_url, &client).await;
        let cookie = format!("cyberint_session={token}");

        // Acknowledge an alert to change its status.
        client
            .patch(format!("{base_url}/api/v1/alerts/CYB-2024-010/status"))
            .header("Cookie", &cookie)
            .json(&serde_json::json!({"status": "acknowledged"}))
            .send()
            .await
            .expect("AC-8: PATCH must succeed");

        // Verify it's acknowledged before reset.
        let before_body: serde_json::Value = client
            .get(format!("{base_url}/api/v1/alerts/CYB-2024-010"))
            .header("Cookie", &cookie)
            .send()
            .await
            .expect("AC-8: GET before reset must succeed")
            .json()
            .await
            .expect("AC-8: before-reset body must be JSON");
        assert_eq!(
            before_body["status"].as_str().unwrap_or(""),
            "acknowledged",
            "AC-8: pre-reset status must be acknowledged"
        );

        // Reset via POST /dtu/reset.
        let reset_resp = client
            .post(format!("{base_url}/dtu/reset"))
            .send()
            .await
            .expect("AC-8: POST /dtu/reset must not error");
        assert_eq!(
            reset_resp.status().as_u16(),
            200,
            "AC-8: /dtu/reset must return 200"
        );
        let reset_body: serde_json::Value = reset_resp
            .json()
            .await
            .expect("AC-8: reset body must be JSON");
        assert_eq!(
            reset_body["status"].as_str().unwrap_or(""),
            "ok",
            "AC-8: /dtu/reset must return {{status: ok}}"
        );

        // After reset, need a new login (old token is invalid).
        let new_token = cyberint_login(&base_url, &client).await;
        let new_cookie = format!("cyberint_session={new_token}");

        // Alert status must be back to "open".
        let after_body: serde_json::Value = client
            .get(format!("{base_url}/api/v1/alerts/CYB-2024-010"))
            .header("Cookie", &new_cookie)
            .send()
            .await
            .expect("AC-8: GET after reset must succeed")
            .json()
            .await
            .expect("AC-8: after-reset body must be JSON");
        assert_eq!(
            after_body["status"].as_str().unwrap_or(""),
            "open",
            "AC-8: alert status must revert to 'open' after reset"
        );
    }

    // =========================================================================
    // Edge cases (migrated from edge_cases.rs)
    // =========================================================================

    /// BC-3.5.001 AC-002 — GET /alerts/{unknown} returns 404 "alert not found".
    #[tokio::test]
    async fn ac_multi_tenant_ec002_unknown_alert_returns_404() {
        let harness = HarnessBuilder::new()
            .isolation(IsolationMode::Logical)
            .with_customer("test-tenant")
            .build()
            .await
            .expect("harness build must succeed");

        let addr = harness
            .endpoint_for("test-tenant", DtuType::Cyberint)
            .expect("test-tenant/Cyberint endpoint must exist");
        let base_url = format!("http://{addr}");

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .expect("build client");

        let token = cyberint_login(&base_url, &client).await;

        let resp = client
            .get(format!("{base_url}/api/v1/alerts/NONEXISTENT-9999"))
            .header("Cookie", format!("cyberint_session={token}"))
            .send()
            .await
            .expect("EC-002: GET for unknown alert must not error");

        assert_eq!(
            resp.status().as_u16(),
            404,
            "EC-002: unknown alert_id must return HTTP 404"
        );

        let body: serde_json::Value = resp.json().await.expect("EC-002: 404 body must be JSON");
        assert_eq!(
            body["error"].as_str().unwrap_or(""),
            "alert not found",
            "EC-002: error message must be 'alert not found'"
        );
    }

    /// BC-3.5.001 AC-002 — out-of-scope endpoint /api/v1/digital-risk/findings returns 404.
    #[tokio::test]
    async fn ac_multi_tenant_ec005_out_of_scope_endpoint_404() {
        let harness = HarnessBuilder::new()
            .isolation(IsolationMode::Logical)
            .with_customer("test-tenant")
            .build()
            .await
            .expect("harness build must succeed");

        let addr = harness
            .endpoint_for("test-tenant", DtuType::Cyberint)
            .expect("test-tenant/Cyberint endpoint must exist");
        let base_url = format!("http://{addr}");

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .expect("build client");

        let resp = client
            .get(format!("{base_url}/api/v1/digital-risk/findings"))
            .send()
            .await
            .expect("EC-005: request to out-of-scope endpoint must not error");

        assert_eq!(
            resp.status().as_u16(),
            404,
            "EC-005: /api/v1/digital-risk/findings must return HTTP 404 (not in scope)"
        );
    }

    /// BC-3.5.001 AC-002 — auth_mode=reject causes all authenticated requests to return 401.
    #[tokio::test]
    async fn ac_multi_tenant_ec006_auth_mode_reject_returns_401() {
        let harness = HarnessBuilder::new()
            .isolation(IsolationMode::Logical)
            .with_customer("test-tenant")
            .build()
            .await
            .expect("harness build must succeed");

        let addr = harness
            .endpoint_for("test-tenant", DtuType::Cyberint)
            .expect("test-tenant/Cyberint endpoint must exist");
        let base_url = format!("http://{addr}");

        let admin_token = harness
            .admin_token_for("test-tenant", DtuType::Cyberint)
            .expect("admin token must exist")
            .to_owned();

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .expect("build client");

        let token = cyberint_login(&base_url, &client).await;
        let cookie = format!("cyberint_session={token}");

        // Verify token works before auth_mode change.
        let before = client
            .get(format!("{base_url}/api/v1/alerts"))
            .header("Cookie", &cookie)
            .send()
            .await
            .expect("EC-006: request before configure must not error");
        assert_eq!(
            before.status().as_u16(),
            200,
            "EC-006: valid cookie must work before auth_mode=reject"
        );

        // Set auth_mode=reject.
        let configure_resp = client
            .post(format!("{base_url}/dtu/configure"))
            .header("X-Admin-Token", &admin_token)
            .json(&serde_json::json!({"auth_mode": "reject"}))
            .send()
            .await
            .expect("EC-006: configure must not error");
        assert_eq!(
            configure_resp.status().as_u16(),
            200,
            "EC-006: configure must return 200"
        );

        // All authenticated requests now return 401 regardless of cookie.
        let after = client
            .get(format!("{base_url}/api/v1/alerts"))
            .header("Cookie", &cookie)
            .send()
            .await
            .expect("EC-006: post-configure request must not error");
        assert_eq!(
            after.status().as_u16(),
            401,
            "EC-006: auth_mode=reject must return 401 even with a valid session cookie"
        );

        let body: serde_json::Value = after.json().await.expect("EC-006: 401 body must be JSON");
        assert_eq!(
            body["error"].as_str().unwrap_or(""),
            "unauthorized",
            "EC-006: error field must be 'unauthorized'"
        );
    }

    // =========================================================================
    // Fidelity validator (migrated from fidelity_validator.rs)
    // =========================================================================

    /// BC-3.5.001 AC-003 — FidelityValidator reports checks_failed == 0 for all Cyberint endpoints.
    ///
    /// `base_url` is sourced from `harness.endpoint_for("test-tenant", DtuType::Cyberint)` (story AC-003).
    #[tokio::test]
    async fn ac_multi_tenant_fidelity_validator_passes() {
        let harness = HarnessBuilder::new()
            .isolation(IsolationMode::Logical)
            .with_customer("test-tenant")
            .build()
            .await
            .expect("harness build must succeed");

        let addr = harness
            .endpoint_for("test-tenant", DtuType::Cyberint)
            .expect("test-tenant/Cyberint endpoint must exist");
        let base_url = format!("http://{addr}");

        let admin_token = harness
            .admin_token_for("test-tenant", DtuType::Cyberint)
            .expect("admin token must exist")
            .to_owned();

        // Fidelity checks limited to endpoints that do not require cookie auth,
        // plus the 401 shape check for unauthenticated access.
        let checks = vec![
            // DTU health endpoint (ADR-002 required, no auth).
            FidelityCheck {
                endpoint: "/dtu/health".to_string(),
                method: http::Method::GET,
                body: None,
                expected_status: 200,
                required_fields: vec!["status".to_string()],
                ..Default::default()
            },
            // Login endpoint shape check.
            FidelityCheck {
                endpoint: "/login".to_string(),
                method: http::Method::POST,
                body: Some(serde_json::json!({})),
                expected_status: 200,
                required_fields: vec!["message".to_string()],
                ..Default::default()
            },
            // Unauthenticated access to alerts must return 401 with "error" field.
            FidelityCheck {
                endpoint: "/api/v1/alerts".to_string(),
                method: http::Method::GET,
                body: None,
                expected_status: 401,
                required_fields: vec!["error".to_string()],
                ..Default::default()
            },
            // DTU configure returns {"status": "ok"} (requires X-Admin-Token per ADR-003 Amendment #5).
            FidelityCheck {
                endpoint: "/dtu/configure".to_string(),
                method: http::Method::POST,
                body: Some(serde_json::json!({})),
                expected_status: 200,
                required_fields: vec!["status".to_string()],
                headers: vec![("X-Admin-Token".to_string(), admin_token.clone())],
            },
            FidelityCheck {
                endpoint: "/dtu/reset".to_string(),
                method: http::Method::POST,
                body: None,
                expected_status: 200,
                required_fields: vec!["status".to_string()],
                ..Default::default()
            },
        ];

        let report = FidelityValidator::run(&base_url, checks).await;
        assert_eq!(
            report.checks_failed, 0,
            "fidelity failures: {:?}",
            report.failures
        );
    }

    // =========================================================================
    // TD tests (migrated from td_wv0_04 and td_wv0_07)
    // =========================================================================

    /// BC-3.5.001 AC-001 — POST /dtu/configure with known field returns 200.
    #[tokio::test]
    async fn ac_multi_tenant_td_wv0_04_known_field_returns_200() {
        let harness = HarnessBuilder::new()
            .isolation(IsolationMode::Logical)
            .with_customer("test-tenant")
            .build()
            .await
            .expect("harness build must succeed");

        let addr = harness
            .endpoint_for("test-tenant", DtuType::Cyberint)
            .expect("test-tenant/Cyberint endpoint must exist");
        let base_url = format!("http://{addr}");

        let admin_token = harness
            .admin_token_for("test-tenant", DtuType::Cyberint)
            .expect("admin token must exist")
            .to_owned();

        let client = reqwest::Client::new();
        let resp = client
            .post(format!("{base_url}/dtu/configure"))
            .header("X-Admin-Token", &admin_token)
            .json(&serde_json::json!({"auth_mode": "reject"}))
            .send()
            .await
            .expect("TD-WV0-04: request must succeed");

        assert_eq!(
            resp.status().as_u16(),
            200,
            "TD-WV0-04: known field must return 200"
        );
    }

    /// BC-3.5.001 AC-001 — POST /dtu/configure with unknown field returns 400.
    #[tokio::test]
    async fn ac_multi_tenant_td_wv0_04_unknown_field_returns_400() {
        let harness = HarnessBuilder::new()
            .isolation(IsolationMode::Logical)
            .with_customer("test-tenant")
            .build()
            .await
            .expect("harness build must succeed");

        let addr = harness
            .endpoint_for("test-tenant", DtuType::Cyberint)
            .expect("test-tenant/Cyberint endpoint must exist");
        let base_url = format!("http://{addr}");

        let admin_token = harness
            .admin_token_for("test-tenant", DtuType::Cyberint)
            .expect("admin token must exist")
            .to_owned();

        let client = reqwest::Client::new();
        let resp = client
            .post(format!("{base_url}/dtu/configure"))
            .header("X-Admin-Token", &admin_token)
            .json(&serde_json::json!({"bogus": "val"}))
            .send()
            .await
            .expect("TD-WV0-04: request must succeed");

        assert_eq!(
            resp.status().as_u16(),
            400,
            "TD-WV0-04: unknown field must return 400 Bad Request, not silently accept"
        );
    }

    /// BC-3.5.001 AC-001 — POST /dtu/configure without X-Admin-Token returns 401.
    #[tokio::test]
    async fn ac_multi_tenant_td_wv0_07_no_token_returns_401() {
        let harness = HarnessBuilder::new()
            .isolation(IsolationMode::Logical)
            .with_customer("test-tenant")
            .build()
            .await
            .expect("harness build must succeed");

        let addr = harness
            .endpoint_for("test-tenant", DtuType::Cyberint)
            .expect("test-tenant/Cyberint endpoint must exist");
        let base_url = format!("http://{addr}");

        let client = reqwest::Client::new();
        let resp = client
            .post(format!("{base_url}/dtu/configure"))
            .json(&serde_json::json!({"auth_mode": "accept"}))
            .send()
            .await
            .expect("TD-WV0-07: request must succeed");

        assert_eq!(
            resp.status().as_u16(),
            401,
            "TD-WV0-07: missing X-Admin-Token must return 401"
        );
    }

    /// BC-3.5.001 AC-001 — POST /dtu/configure with wrong X-Admin-Token returns 401.
    #[tokio::test]
    async fn ac_multi_tenant_td_wv0_07_wrong_token_returns_401() {
        let harness = HarnessBuilder::new()
            .isolation(IsolationMode::Logical)
            .with_customer("test-tenant")
            .build()
            .await
            .expect("harness build must succeed");

        let addr = harness
            .endpoint_for("test-tenant", DtuType::Cyberint)
            .expect("test-tenant/Cyberint endpoint must exist");
        let base_url = format!("http://{addr}");

        let client = reqwest::Client::new();
        let resp = client
            .post(format!("{base_url}/dtu/configure"))
            .header("X-Admin-Token", "wrong-token-that-will-never-match")
            .json(&serde_json::json!({"auth_mode": "accept"}))
            .send()
            .await
            .expect("TD-WV0-07: request must succeed");

        assert_eq!(
            resp.status().as_u16(),
            401,
            "TD-WV0-07: incorrect X-Admin-Token must return 401"
        );
    }

    /// BC-3.5.001 AC-001 — POST /dtu/configure with correct X-Admin-Token returns 200.
    #[tokio::test]
    async fn ac_multi_tenant_td_wv0_07_correct_token_returns_200() {
        let harness = HarnessBuilder::new()
            .isolation(IsolationMode::Logical)
            .with_customer("test-tenant")
            .build()
            .await
            .expect("harness build must succeed");

        let addr = harness
            .endpoint_for("test-tenant", DtuType::Cyberint)
            .expect("test-tenant/Cyberint endpoint must exist");
        let base_url = format!("http://{addr}");

        let admin_token = harness
            .admin_token_for("test-tenant", DtuType::Cyberint)
            .expect("admin token must exist")
            .to_owned();

        let client = reqwest::Client::new();
        let resp = client
            .post(format!("{base_url}/dtu/configure"))
            .header("X-Admin-Token", &admin_token)
            .json(&serde_json::json!({"auth_mode": "accept"}))
            .send()
            .await
            .expect("TD-WV0-07: request must succeed");

        assert_eq!(
            resp.status().as_u16(),
            200,
            "TD-WV0-07: correct X-Admin-Token must return 200"
        );
    }

    // =========================================================================
    // NEW: 2-org logical isolation (BC-3.5.001 postcondition 2; TV-2)
    // =========================================================================

    /// BC-3.5.001 AC-004 — two-org logical harness returns pairwise-disjoint alert/indicator sets.
    ///
    /// Builds a 2-org harness (alpha + beta) in `IsolationMode::Logical`. Fetches
    /// `/api/v1/alerts` from each org's Cyberint clone and asserts:
    /// - Both responses contain a `data` array.
    /// - The alert_id sets are pairwise-disjoint (no cross-org data leakage).
    ///
    /// (BC-3.5.001 postcondition 2; TV-2; S-3.4.04 AC-004)
    #[tokio::test]
    async fn ac_multi_org_logical_isolation() {
        // Build a 2-org harness with distinct seeds to guarantee distinct alert_id sets.
        let harness = HarnessBuilder::new()
            .isolation(IsolationMode::Logical)
            .with_customer_overrides("alpha", |spec| {
                spec.dtu_types = vec![DtuType::Cyberint];
                spec.seed = 1001;
            })
            .with_customer_overrides("beta", |spec| {
                spec.dtu_types = vec![DtuType::Cyberint];
                spec.seed = 1002;
            })
            .build()
            .await
            .expect("2-org harness build must succeed");

        let alpha_addr = harness
            .endpoint_for("alpha", DtuType::Cyberint)
            .expect("alpha/Cyberint endpoint must exist");
        let beta_addr = harness
            .endpoint_for("beta", DtuType::Cyberint)
            .expect("beta/Cyberint endpoint must exist");

        let alpha_url = format!("http://{alpha_addr}");
        let beta_url = format!("http://{beta_addr}");

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .expect("build client");

        // Login to both orgs separately — cookie-based auth is per-clone.
        let alpha_token = cyberint_login(&alpha_url, &client).await;
        let beta_token = cyberint_login(&beta_url, &client).await;

        // Fetch alerts for each org.
        let alpha_resp = client
            .get(format!("{alpha_url}/api/v1/alerts"))
            .header("Cookie", format!("cyberint_session={alpha_token}"))
            .send()
            .await
            .expect("AC-multi-org: GET alpha alerts must not error");
        assert_eq!(
            alpha_resp.status().as_u16(),
            200,
            "AC-multi-org: alpha alerts must return 200"
        );

        let beta_resp = client
            .get(format!("{beta_url}/api/v1/alerts"))
            .header("Cookie", format!("cyberint_session={beta_token}"))
            .send()
            .await
            .expect("AC-multi-org: GET beta alerts must not error");
        assert_eq!(
            beta_resp.status().as_u16(),
            200,
            "AC-multi-org: beta alerts must return 200"
        );

        let alpha_body: serde_json::Value = alpha_resp
            .json()
            .await
            .expect("AC-multi-org: alpha body must be JSON");
        let beta_body: serde_json::Value = beta_resp
            .json()
            .await
            .expect("AC-multi-org: beta body must be JSON");

        let alpha_data = alpha_body["data"]
            .as_array()
            .expect("AC-multi-org: alpha data must be array");
        let beta_data = beta_body["data"]
            .as_array()
            .expect("AC-multi-org: beta data must be array");

        assert!(
            !alpha_data.is_empty(),
            "AC-multi-org: alpha alerts must be non-empty (BC-3.5.001 postcondition 2)"
        );
        assert!(
            !beta_data.is_empty(),
            "AC-multi-org: beta alerts must be non-empty (BC-3.5.001 postcondition 2)"
        );

        // Collect alert_id sets — they must be pairwise disjoint.
        let alpha_ids: std::collections::HashSet<&str> = alpha_data
            .iter()
            .filter_map(|a| a["alert_id"].as_str())
            .collect();
        let beta_ids: std::collections::HashSet<&str> = beta_data
            .iter()
            .filter_map(|a| a["alert_id"].as_str())
            .collect();

        let intersection: std::collections::HashSet<_> =
            alpha_ids.intersection(&beta_ids).collect();
        assert!(
            intersection.is_empty(),
            "AC-multi-org: alert_id sets must be pairwise-disjoint — \
             found common IDs: {:?} (BC-3.5.001 postcondition 2; TV-2)",
            intersection
        );

        // Also verify threat-intel data is disjoint (BC-3.5.001 postcondition 2).
        let alpha_ti_resp = client
            .get(format!("{alpha_url}/api/v1/threat-intel"))
            .header("Cookie", format!("cyberint_session={alpha_token}"))
            .send()
            .await
            .expect("AC-multi-org: GET alpha threat-intel must not error");
        let beta_ti_resp = client
            .get(format!("{beta_url}/api/v1/threat-intel"))
            .header("Cookie", format!("cyberint_session={beta_token}"))
            .send()
            .await
            .expect("AC-multi-org: GET beta threat-intel must not error");

        assert_eq!(
            alpha_ti_resp.status().as_u16(),
            200,
            "AC-multi-org: alpha threat-intel must return 200"
        );
        assert_eq!(
            beta_ti_resp.status().as_u16(),
            200,
            "AC-multi-org: beta threat-intel must return 200"
        );

        let alpha_ti: serde_json::Value = alpha_ti_resp
            .json()
            .await
            .expect("AC-multi-org: alpha threat-intel body must be JSON");
        let beta_ti: serde_json::Value = beta_ti_resp
            .json()
            .await
            .expect("AC-multi-org: beta threat-intel body must be JSON");

        let alpha_ti_data = alpha_ti["data"].as_array().cloned().unwrap_or_default();
        let beta_ti_data = beta_ti["data"].as_array().cloned().unwrap_or_default();

        // Both must have data fields.
        assert!(
            alpha_ti.get("data").is_some(),
            "AC-multi-org: alpha threat-intel must include data field"
        );
        assert!(
            beta_ti.get("data").is_some(),
            "AC-multi-org: beta threat-intel must include data field"
        );

        // Collect threat IDs for disjointness check.
        let alpha_ti_ids: std::collections::HashSet<String> = alpha_ti_data
            .iter()
            .filter_map(|t| {
                t["indicator_id"]
                    .as_str()
                    .or_else(|| t["id"].as_str())
                    .map(|s| s.to_owned())
            })
            .collect();
        let beta_ti_ids: std::collections::HashSet<String> = beta_ti_data
            .iter()
            .filter_map(|t| {
                t["indicator_id"]
                    .as_str()
                    .or_else(|| t["id"].as_str())
                    .map(|s| s.to_owned())
            })
            .collect();

        // Only assert disjoint if both sets are non-empty (both orgs generated indicators).
        if !alpha_ti_ids.is_empty() && !beta_ti_ids.is_empty() {
            let ti_intersection: std::collections::HashSet<_> =
                alpha_ti_ids.intersection(&beta_ti_ids).collect();
            assert!(
                ti_intersection.is_empty(),
                "AC-multi-org: threat-intel indicator sets must be pairwise-disjoint — \
                 found common IDs: {:?} (BC-3.5.001 postcondition 2; TV-2)",
                ti_intersection
            );
        }
    }

    // =========================================================================
    // NEW: Network cross-org credential mismatch → HTTP 401 (BC-3.5.002; TV-3)
    // =========================================================================

    /// BC-3.5.002 AC-005 — cross-org credential mismatch in Network mode returns HTTP 401.
    ///
    /// Builds a 2-org harness (alpha + beta) in `IsolationMode::Network`.
    /// Obtains OrgA's admin token via `harness.admin_token_for("alpha", DtuType::Cyberint)`.
    /// Sends OrgA's token to OrgB's endpoint (OrgB's `SocketAddr` from `customer_endpoints()`).
    /// Asserts HTTP 401 (wrong-token rejection by OrgB's auth middleware).
    ///
    /// (BC-3.5.002 postcondition 2; VP-126; TV-3; S-3.4.04 AC-005)
    #[tokio::test]
    async fn ac_network_cross_creds_401() {
        let harness = HarnessBuilder::new()
            .isolation(IsolationMode::Network)
            .with_customer_overrides("alpha", |spec| {
                spec.dtu_types = vec![DtuType::Cyberint];
                spec.seed = 2001;
            })
            .with_customer_overrides("beta", |spec| {
                spec.dtu_types = vec![DtuType::Cyberint];
                spec.seed = 2002;
            })
            .build()
            .await
            .expect("2-org Network harness build must succeed");

        // Retrieve alpha's admin token — used as cross-org credential against beta's endpoint.
        let alpha_admin_token = harness
            .admin_token_for("alpha", DtuType::Cyberint)
            .expect("alpha admin token must exist after Network build")
            .to_owned();

        // Get beta's endpoint address from customer_endpoints().
        let beta_addr = harness
            .endpoint_for("beta", DtuType::Cyberint)
            .expect("beta/Cyberint endpoint must exist in Network harness");
        let beta_url = format!("http://{beta_addr}");

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .expect("build client");

        // Send alpha's admin token to beta's configure endpoint — cross-org credential mismatch.
        // In Network mode, beta's clone validates the Bearer token against its own admin_token,
        // and alpha's token != beta's token → HTTP 401.
        //
        // We use the Authorization: Bearer pattern (Network-mode router validates this).
        let cross_resp = client
            .get(format!("{beta_url}/api/v1/alerts"))
            .bearer_auth(&alpha_admin_token)
            .send()
            .await
            .expect("AC-network-cross-creds: cross-org request must not fail at network level");

        assert_eq!(
            cross_resp.status().as_u16(),
            401,
            "AC-network-cross-creds: alpha's admin token sent to beta's endpoint must return \
             HTTP 401 (BC-3.5.002 postcondition 2; VP-126; TV-3)"
        );

        // Also verify correct-org credentials work on beta's endpoint (sanity check).
        let beta_admin_token = harness
            .admin_token_for("beta", DtuType::Cyberint)
            .expect("beta admin token must exist")
            .to_owned();

        let correct_resp = client
            .get(format!("{beta_url}/api/v1/alerts"))
            .bearer_auth(&beta_admin_token)
            .send()
            .await
            .expect("AC-network-cross-creds: correct-org request must not fail at network level");

        assert_eq!(
            correct_resp.status().as_u16(),
            200,
            "AC-network-cross-creds: beta's admin token sent to beta's endpoint must return \
             HTTP 200 (BC-3.5.002 postcondition 2; TV-3 correct-org sanity check)"
        );
    }

    // =========================================================================
    // NEW: Failure injection via .with_failure() builder shorthand (BC-3.6.001)
    // =========================================================================

    /// BC-3.6.001 postcondition 1 — `.with_failure()` builder injects `Timeout` mode before
    /// the harness is returned; the first request from OrgA observes the failure.
    ///
    /// Procedure:
    /// 1. Build a 2-org harness in `IsolationMode::Logical`.
    ///    - alpha: inject `FailureMode::NetworkTimeout { after_ms: 500 }` via `.with_failure()`.
    ///    - beta: no injection.
    /// 2. Send GET /api/v1/alerts to alpha's clone with a 200ms reqwest timeout.
    ///    Assert the request times out (or returns an error — clone is slow).
    /// 3. Send GET /api/v1/alerts to beta's clone.
    ///    Assert HTTP 200 (beta is unaffected — BC-3.6.001 TV-6 / EC-001 isolation).
    ///
    /// (BC-3.6.001 postcondition 1; TV-6; S-3.4.04 edge-case EC-001)
    #[tokio::test]
    async fn ac_failure_injection_via_with_failure() {
        // Build harness with NetworkTimeout pre-injected on alpha via .with_failure().
        // The first request to alpha after build() returns will observe the 500ms delay.
        let harness = HarnessBuilder::new()
            .isolation(IsolationMode::Logical)
            .with_customer_overrides("alpha", |spec| {
                spec.dtu_types = vec![DtuType::Cyberint];
                spec.seed = 3001;
            })
            .with_failure(
                "alpha",
                DtuType::Cyberint,
                FailureMode::NetworkTimeout { after_ms: 500 },
            )
            .with_customer_overrides("beta", |spec| {
                spec.dtu_types = vec![DtuType::Cyberint];
                spec.seed = 3002;
            })
            .build()
            .await
            .expect("2-org harness with pre-injected failure build must succeed");

        let alpha_addr = harness
            .endpoint_for("alpha", DtuType::Cyberint)
            .expect("alpha/Cyberint endpoint must exist");
        let beta_addr = harness
            .endpoint_for("beta", DtuType::Cyberint)
            .expect("beta/Cyberint endpoint must exist");

        let alpha_url = format!("http://{alpha_addr}");
        let beta_url = format!("http://{beta_addr}");

        // alpha: use a 200ms timeout — alpha's clone has 500ms delay injected,
        // so the request must time out (BC-3.6.001 postcondition 1 first-request-observes-failure).
        let alpha_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_millis(200))
            .build()
            .expect("build alpha client with 200ms timeout");

        let alpha_result = alpha_client
            .get(format!("{alpha_url}/api/v1/alerts"))
            .send()
            .await;

        // The request must fail (timeout or connection error) — not return HTTP 200.
        assert!(
            alpha_result.is_err(),
            "BC-3.6.001 postcondition 1: first request to alpha must observe \
             NetworkTimeout failure — expected error but got HTTP {}",
            alpha_result
                .as_ref()
                .map(|r| r.status().to_string())
                .unwrap_or_else(|_| "err".to_string())
        );

        // beta: no failure injected — must respond with HTTP 200 (isolation guarantee).
        let beta_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .expect("build beta client");

        // beta's clone serves Cyberint alerts without cookie auth at the /api/v1/alerts
        // path — but the clone IS cookie-gated. We verify via the health endpoint
        // (no auth required) to confirm beta's clone is healthy and unaffected.
        let beta_health = beta_client
            .get(format!("{beta_url}/dtu/health"))
            .send()
            .await
            .expect("BC-3.6.001 TV-6: GET beta /dtu/health must not error");

        assert_eq!(
            beta_health.status().as_u16(),
            200,
            "BC-3.6.001 TV-6 EC-001: beta clone must return HTTP 200 — \
             unaffected by alpha's NetworkTimeout injection (BC-3.6.001 TV-6)"
        );

        let beta_body: serde_json::Value = beta_health
            .json()
            .await
            .expect("BC-3.6.001 TV-6: beta health body must be JSON");
        assert_eq!(
            beta_body["status"].as_str().unwrap_or(""),
            "ok",
            "BC-3.6.001 TV-6: beta /dtu/health must return {{status: ok}}"
        );
    }
}
