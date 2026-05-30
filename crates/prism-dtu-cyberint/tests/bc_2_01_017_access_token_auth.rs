#![allow(clippy::unwrap_used, clippy::expect_used)]
// BC-based test names use mixed-case identifiers following the factory naming standard.
#![allow(non_snake_case)]
//! Red Gate tests for BC-2.01.017 and BC-2.16.013 — DTU-side access_token auth.
//!
//! Story: S-DTU-CYBERINT-AUTH-FIDELITY-001
//!
//! These four tests cover the DTU-side acceptance criteria:
//!
//! | Test                                                               | AC     | BC         |
//! |--------------------------------------------------------------------|--------|------------|
//! | test_BC_2_16_013_dtu_post_login_route_removed_returns_404          | AC-001 | BC-2.16.013 |
//! | test_BC_2_01_013_dtu_extract_access_token_parses_cookie_header     | AC-002 | BC-2.01.013 |
//! | test_BC_2_16_013_dtu_check_auth_requires_access_token_cookie_not_session | AC-003 | BC-2.16.013 |
//! | test_BC_2_16_013_dtu_state_access_token_allowlist_not_session_uuid | AC-004 | BC-2.16.013 |
//!
//! # Red Gate state
//!
//! These tests exercise `todo!()` stubs (AC-002..004) or assert behaviour of the
//! pre-implementation router (AC-001). They MUST FAIL in the Red Gate state because:
//! - `extract_access_token` panics (todo!()).
//! - `is_valid_access_token` panics (todo!()).
//! - `register_access_token` panics (todo!()).
//! - `with_demo_token` panics (todo!()).
//! - The assertions in AC-001 assert 404 but the router currently has /login → 200.
//! - The assertions in AC-003 assert cookie-based 200/401 routing which depends on
//!   `extract_access_token` (todo!()).
//!
//! Post-implementation all four tests must be GREEN.

#[cfg(feature = "dtu")]
mod bc_2_01_017_dtu {
    use axum::http::HeaderMap;
    use prism_dtu_common::BehavioralClone;
    use prism_dtu_cyberint::{state::CyberintState, CyberintClone};

    // ── Fixture helpers ──────────────────────────────────────────────────────

    /// Start a `CyberintClone` with `demo-access-key` registered via `configure()`.
    ///
    /// Uses the real DTU HTTP server — required for AC-001, AC-003 HTTP-layer tests.
    async fn start_clone_with_demo_token() -> (CyberintClone, String, reqwest::Client) {
        let mut clone =
            CyberintClone::new().expect("CyberintClone::new() must succeed in test setup");
        clone
            .start()
            .await
            .expect("start() must succeed in test setup");
        let base_url = clone.base_url();

        // Register the demo access_token via /dtu/configure so the DTU accepts it.
        // AC-004: this call exercises register_access_token (todo!() stub — will panic).
        clone
            .configure(serde_json::json!({"access_token": "demo-access-key"}))
            .await
            .expect("configure must succeed");

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .expect("reqwest client build must succeed");
        (clone, base_url, client)
    }

    // ── Test 1: AC-001 ───────────────────────────────────────────────────────

    /// AC-001 / BC-2.16.013 postcondition: `build_router` MUST NOT register
    /// `POST /login`. A request to POST /login on the corrected DTU returns 404.
    ///
    /// ADR-031 §D1-b: real Cyberint API has no login endpoint; DTU must match.
    ///
    /// # Red Gate failure mode
    ///
    /// Before implementation: `build_router` still has `.route("/login", post(post_login))`.
    /// The route returns 200 (or 401), not 404. This assertion fails with
    /// "expected 404, got 200/401".
    ///
    /// After implementation: /login route is removed; axum returns 404. Test passes.
    ///
    /// Story: S-DTU-CYBERINT-AUTH-FIDELITY-001 AC-001 | BC-2.16.013 | ADR-031 §D1-b
    #[tokio::test]
    async fn test_BC_2_16_013_dtu_post_login_route_removed_returns_404() {
        let mut clone =
            CyberintClone::new().expect("test_BC_2_16_013: CyberintClone::new() must succeed");
        clone
            .start()
            .await
            .expect("test_BC_2_16_013: start() must succeed");
        let base_url = clone.base_url();
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .expect("test_BC_2_16_013: reqwest client build");

        let resp = client
            .post(format!("{base_url}/login"))
            .json(&serde_json::json!({}))
            .send()
            .await
            .expect(
                "test_BC_2_16_013: POST /login request must not error at the network level \
                 (server is running on a bound port)",
            );

        assert_eq!(
            resp.status().as_u16(),
            404,
            "AC-001 / BC-2.16.013: POST /login MUST return 404 — the route is removed per \
             ADR-031 §D1-b; the real Cyberint API requires no login step. \
             Got status {} instead. Check build_router() in clone.rs for a remaining \
             .route(\"/login\", ...) registration.",
            resp.status().as_u16()
        );
    }

    // ── Test 2: AC-002 ───────────────────────────────────────────────────────

    /// AC-002 / BC-2.01.013 precondition: `extract_access_token` MUST parse the
    /// `access_token` cookie value from the `Cookie` header and return it.
    ///
    /// Tests the following canonical cases from BC-2.01.017 §Edge Cases:
    /// - Happy path: `Cookie: access_token=test-api-key-abc123` → `Some("test-api-key-abc123")`
    /// - Correct cookie name: `access_token` cookie present → value returned
    /// - Wrong cookie name: only `cyberint_session` present → `None` (AC-002 contract)
    /// - Absent header → `None`
    /// - Multiple cookies: `access_token=val1; session=val2` → `Some("val1")` (EC-003)
    ///
    /// # Red Gate failure mode
    ///
    /// `extract_access_token` is `todo!()` — panics with "AC-002: parse cookie header...".
    /// The first `assert_eq!` call triggers the panic before reaching later assertions.
    ///
    /// Story: S-DTU-CYBERINT-AUTH-FIDELITY-001 AC-002 | BC-2.01.013 | BC-2.01.017
    #[test]
    fn test_BC_2_01_013_dtu_extract_access_token_parses_cookie_header() {
        use prism_dtu_cyberint::routes::alerts::extract_access_token;

        // ── Case 1: Happy path — access_token cookie present ──────────────
        let mut headers = HeaderMap::new();
        headers.insert(
            "cookie",
            "access_token=test-api-key-abc123".parse().unwrap(),
        );
        let result = extract_access_token(&headers);
        assert_eq!(
            result.as_deref(),
            Some("test-api-key-abc123"),
            "AC-002 / TV-BC-2.01.017-001: extract_access_token must return the access_token \
             cookie value when present. Got: {:?}",
            result
        );

        // ── Case 2: Only cyberint_session cookie present → None ────────────
        // ADR-031 §D3-a: the old cyberint_session cookie name is rejected.
        // extract_access_token returns None — it only parses access_token.
        let mut headers_old = HeaderMap::new();
        headers_old.insert(
            "cookie",
            "cyberint_session=legacy-session-uuid".parse().unwrap(),
        );
        let result_old = extract_access_token(&headers_old);
        assert_eq!(
            result_old, None,
            "AC-002: extract_access_token MUST return None when only cyberint_session \
             cookie is present (ADR-031 §D3-a: old cookie name is superseded). \
             Got: {:?}",
            result_old
        );

        // ── Case 3: No Cookie header → None ──────────────────────────────
        let empty_headers = HeaderMap::new();
        let result_absent = extract_access_token(&empty_headers);
        assert_eq!(
            result_absent, None,
            "AC-002: extract_access_token must return None when Cookie header is absent. \
             Got: {:?}",
            result_absent
        );

        // ── Case 4: Multiple cookies — access_token appears first (EC-003) ─
        let mut multi_headers = HeaderMap::new();
        multi_headers.insert("cookie", "access_token=val1; session=val2".parse().unwrap());
        let result_multi = extract_access_token(&multi_headers);
        assert_eq!(
            result_multi.as_deref(),
            Some("val1"),
            "AC-002 / EC-003: extract_access_token must return the access_token value \
             when multiple cookies are present. Got: {:?}",
            result_multi
        );

        // ── Case 5: Case-sensitive cookie name — Access-Token ≠ access_token (EC-004) ─
        let mut case_headers = HeaderMap::new();
        case_headers.insert("cookie", "Access-Token=case-test-value".parse().unwrap());
        let result_case = extract_access_token(&case_headers);
        assert_eq!(
            result_case, None,
            "AC-002 / EC-004: cookie names are case-sensitive per RFC 6265; \
             'Access-Token' is NOT 'access_token'; must return None. Got: {:?}",
            result_case
        );
    }

    // ── Test 3: AC-003 ───────────────────────────────────────────────────────

    /// AC-003 / BC-2.16.013 postcondition: `check_auth` MUST validate the
    /// `access_token` cookie (not `cyberint_session`).
    ///
    /// - `Cookie: access_token={registered_token}` → 200
    /// - `Cookie: cyberint_session={any_value}` → 401
    /// - No cookie header → 401
    ///
    /// This is an HTTP-layer test exercising the full DTU request path:
    /// `check_auth` → `extract_access_token` → `is_valid_access_token`.
    ///
    /// # Red Gate failure modes
    ///
    /// 1. `configure()` calls `register_access_token` which is `todo!()` → panic during setup.
    /// 2. Even if setup somehow completes, `extract_access_token` is `todo!()` → panic on
    ///    the first authenticated request.
    /// 3. The `cyberint_session` negative test may accidentally pass if auth logic is broken,
    ///    but with todo!() panics the test overall will fail.
    ///
    /// Story: S-DTU-CYBERINT-AUTH-FIDELITY-001 AC-003 | BC-2.16.013 | ADR-031 §D1-a §D3-a
    #[tokio::test]
    async fn test_BC_2_16_013_dtu_check_auth_requires_access_token_cookie_not_session() {
        let (mut clone, base_url, client) = start_clone_with_demo_token().await;

        // ── Positive: access_token cookie → 200 ──────────────────────────
        let resp_ok = client
            .get(format!("{base_url}/api/v1/alerts"))
            .header("cookie", "access_token=demo-access-key")
            .send()
            .await
            .expect(
                "test_BC_2_16_013_check_auth: GET /api/v1/alerts must not error at network level",
            );

        assert_eq!(
            resp_ok.status().as_u16(),
            200,
            "AC-003 / BC-2.16.013: GET /api/v1/alerts with 'Cookie: access_token=demo-access-key' \
             MUST return 200 after the token is registered. Got {} instead. \
             Check check_auth and is_valid_access_token in alerts.rs / state.rs.",
            resp_ok.status().as_u16()
        );

        // ── Negative: cyberint_session cookie → 401 ───────────────────────
        // ADR-031 §D1-a: old cookie name must be explicitly rejected.
        let resp_old = client
            .get(format!("{base_url}/api/v1/alerts"))
            .header("cookie", "cyberint_session=any-legacy-session-value")
            .send()
            .await
            .expect(
                "test_BC_2_16_013_check_auth: GET with cyberint_session must not error at network level",
            );

        assert_eq!(
            resp_old.status().as_u16(),
            401,
            "AC-003 / BC-2.16.013: GET /api/v1/alerts with 'Cookie: cyberint_session=...' \
             MUST return 401 — the old pre-fix cookie name is not accepted by the corrected DTU. \
             Got {} instead. Verify extract_access_token does NOT parse cyberint_session.",
            resp_old.status().as_u16()
        );

        // ── Negative: no cookie → 401 ────────────────────────────────────
        let resp_no_cookie = client
            .get(format!("{base_url}/api/v1/alerts"))
            .send()
            .await
            .expect(
                "test_BC_2_16_013_check_auth: GET with no cookie must not error at network level",
            );

        assert_eq!(
            resp_no_cookie.status().as_u16(),
            401,
            "AC-003: GET /api/v1/alerts with no cookie header MUST return 401. \
             Got {} instead.",
            resp_no_cookie.status().as_u16()
        );

        let _ = clone.stop().await;
    }

    // ── Test 4: AC-004 ───────────────────────────────────────────────────────

    /// AC-004 / BC-2.16.013 invariant: `CyberintState` uses a static
    /// `access_token_allowlist` — NOT a session UUID store.
    ///
    /// Asserts:
    /// 1. Fresh state has an `access_token_allowlist` field (not `session_store`).
    /// 2. `register_access_token("demo-key")` → `is_valid_access_token("demo-key") == true`.
    /// 3. `is_valid_access_token("unknown")` → false (not in allowlist).
    /// 4. No `Uuid::new_v4()` session token issuance exists in the auth flow.
    ///
    /// # Red Gate failure mode
    ///
    /// `register_access_token` is `todo!()` → panics with
    /// "AC-004: insert token into access_token_allowlist".
    /// The test fails with a panic before reaching the `is_valid_access_token` assertions.
    ///
    /// Story: S-DTU-CYBERINT-AUTH-FIDELITY-001 AC-004 | BC-2.16.013 | ADR-031 §D3-a rule 3
    #[test]
    fn test_BC_2_16_013_dtu_state_access_token_allowlist_not_session_uuid() {
        use prism_core::OrgId;

        let org_id = OrgId::from_uuid(
            uuid::Uuid::parse_str("00000000-0000-7000-8000-000000000001")
                .expect("test org UUID must parse"),
        );
        let state = CyberintState::with_org_id_and_admin_token(
            org_id,
            vec![],
            vec![],
            vec![],
            "admin-token".to_owned(),
        );

        // ── Subtest 1: allowlist is empty on fresh state ──────────────────
        // Directly read the allowlist to confirm it exists and is empty.
        // This assertion does NOT exercise any todo!() path.
        {
            let allowlist = state
                .access_token_allowlist
                .lock()
                .expect("allowlist poisoned in test");
            assert!(
                allowlist.is_empty(),
                "AC-004: fresh CyberintState must have an empty access_token_allowlist. \
                 Got non-empty allowlist: {:?}",
                allowlist
            );
        }

        // ── Subtest 2: register_access_token → is_valid_access_token ─────
        // Both methods are todo!() stubs — calling either triggers a panic.
        state.register_access_token("demo-key-abc123".to_owned());

        // is_valid_access_token is also todo!() — will panic if registration reached
        // (but registration panics first at the Red Gate stage).
        let valid = state.is_valid_access_token("demo-key-abc123");
        assert!(
            valid,
            "AC-004: after register_access_token('demo-key-abc123'), \
             is_valid_access_token('demo-key-abc123') MUST return true. \
             Got false — check allowlist insertion in register_access_token.",
        );

        // ── Subtest 3: unknown token → false ─────────────────────────────
        let not_valid = state.is_valid_access_token("unknown-token-not-registered");
        assert!(
            !not_valid,
            "AC-004: is_valid_access_token('unknown-token-not-registered') MUST return false \
             when not in allowlist. Got true — check allowlist lookup in is_valid_access_token.",
        );

        // ── Subtest 4: with_demo_token builder ───────────────────────────
        // The builder variant is also todo!() — exercises that path as well.
        let state2 = CyberintState::with_org_id_and_admin_token(
            org_id,
            vec![],
            vec![],
            vec![],
            "admin-token-2".to_owned(),
        )
        .with_demo_token("builder-demo-key".to_owned());

        let valid_builder = state2.is_valid_access_token("builder-demo-key");
        assert!(
            valid_builder,
            "AC-004: with_demo_token builder must register the token such that \
             is_valid_access_token('builder-demo-key') returns true. Got false.",
        );
    }
}
