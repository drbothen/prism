#![allow(clippy::unwrap_used, clippy::expect_used)]
// BC-based test names use mixed-case identifiers following the factory naming standard.
// See prism-core/src/tests/capability_tests.rs for precedent.
#![allow(non_snake_case)]
//! Multi-tenant state segregation tests for `prism-dtu-cyberint`.
//!
//! Covers:
//! - BC-3.2.001: Per-Org Sensor Data Isolation via Composite HashMap Key
//!
//! NOTE (ADR-031 §D3-a): BC-3.2.003 (per-org session token isolation) has been
//! superseded by the static access-token allowlist model.  The access_token is an
//! account-level API key — it is org-agnostic.  Per-org token isolation no longer
//! applies.  Tests below cover the new allowlist semantics.
//!
//! Acceptance criteria tested:
//! - AC-001: Alert store cross-org isolation
//! - AC-004: access_token_allowlist is org-agnostic (global)
//! - AC-005: build_alert_store accepts OrgId parameter
//! - AC-006: reset_for clears alert_store for one org; access_token_allowlist is unaffected
//! - AC-007: OrgId-flipping proptest kills mutation (VP-3.2.001-03)
//!
//! HTTP-layer tests verify that the `access_token` cookie is used for authentication
//! (no POST /login; AC-001 + ADR-031 §D3-a).

#[cfg(feature = "dtu")]
mod multi_tenant {
    use prism_core::OrgId;
    use prism_dtu_common::BehavioralClone;
    use prism_dtu_cyberint::{
        state::CyberintState,
        types::{Alert, AlertStatus},
        CyberintClone,
    };
    use uuid::Uuid;

    // ── Test fixture helpers ─────────────────────────────────────────────────

    /// Construct two distinct `OrgId` values for use in isolation tests.
    ///
    /// Uses `OrgId::from_uuid` (bypasses v7 assertion) because test IDs are
    /// deterministic constants, not freshly minted v7 timestamps.
    fn org_pair() -> (OrgId, OrgId) {
        // Deterministic but distinct UUIDs for org_A and org_B.
        let a = OrgId::from_uuid(
            Uuid::parse_str("00000000-0000-7000-8000-000000000001").expect("valid uuid"),
        );
        let b = OrgId::from_uuid(
            Uuid::parse_str("00000000-0000-7000-8000-000000000002").expect("valid uuid"),
        );
        (a, b)
    }

    /// Build a minimal `CyberintState` with a single fixture alert for `org_id`.
    fn state_with_one_alert(org_id: OrgId, alert_id: &str) -> CyberintState {
        let fixture = vec![Alert {
            alert_id: alert_id.to_owned(),
            title: "Test alert".to_owned(),
            severity: "high".to_owned(),
            status: "open".to_owned(),
            created_at: serde_json::json!("2024-01-01T00:00:00Z"),
            source: "test".to_owned(),
            alert_type: "test".to_owned(),
            affected_assets: vec![],
        }];
        CyberintState::with_org_id_and_admin_token(
            org_id,
            fixture,
            vec![],
            vec![],
            "admin-token".to_owned(),
        )
    }

    /// Build a `CyberintState` pre-seeded with alerts for `org_id` and then
    /// manually insert entries for a second org to simulate multi-tenant state.
    fn state_with_two_orgs(org_a: OrgId, org_b: OrgId, alert_id: &str) -> CyberintState {
        let state = state_with_one_alert(org_a, alert_id);
        // Manually insert an entry for org_b into alert_store.
        {
            let mut store = state
                .alert_store
                .lock()
                .expect("alert_store poisoned in fixture setup");
            store.insert(
                (org_b, alert_id.to_owned()),
                AlertStatus {
                    alert_id: alert_id.to_owned(),
                    status: "open".to_owned(),
                    closed: false,
                },
            );
        }
        state
    }

    // ── HTTP helper ──────────────────────────────────────────────────────────

    /// Start a clone and return `(clone, base_url, admin_token, client)`.
    async fn start_clone() -> (CyberintClone, String, String, reqwest::Client) {
        let mut clone = CyberintClone::new().expect("multi_tenant: new must succeed");
        clone
            .start()
            .await
            .expect("multi_tenant: start must succeed");
        let base_url = clone.base_url();
        let admin_token = clone.admin_token().to_string();
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .expect("build client");
        (clone, base_url, admin_token, client)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // AC-001 — Alert store cross-org isolation (BC-3.2.001 postcondition 1)
    // TV-3.2.001-02: Store tag for org_A; lookup org_B returns None.
    // ═══════════════════════════════════════════════════════════════════════════

    /// BC-3.2.001 post-condition 1: alert written under org_A is not visible to org_B.
    ///
    /// TV-3.2.001-02: Store AlertStatus for (org_id_A, "alert-007");
    /// lookup (org_id_B, "alert-007") must return None.
    #[test]
    fn test_BC_3_2_001_alert_cross_org_isolation_write_a_read_b_returns_none() {
        let (org_a, org_b) = org_pair();
        let state = state_with_one_alert(org_a, "alert-007");

        let store = state.alert_store.lock().expect("alert_store poisoned");

        // org_A entry must exist.
        assert!(
            store.get(&(org_a, "alert-007".to_owned())).is_some(),
            "AC-001: (org_A, alert-007) must be in alert_store"
        );
        // org_B must not see org_A's entry.
        assert!(
            store.get(&(org_b, "alert-007".to_owned())).is_none(),
            "AC-001: (org_B, alert-007) must be None — cross-org leak detected"
        );
    }

    /// BC-3.2.001 post-condition 3: both orgs can store the same alert_id with
    /// independent content and each retrieval returns the correct org's content.
    ///
    /// TV-3.2.001-03: Store (org_A, "dev-1") and (org_B, "dev-1"); each lookup
    /// returns independent content.
    #[test]
    fn test_BC_3_2_001_alert_independent_per_org_state_same_key() {
        let (org_a, org_b) = org_pair();
        let state = state_with_two_orgs(org_a, org_b, "alert-shared");

        // Mutate org_A's entry to distinguish it.
        {
            let mut store = state.alert_store.lock().expect("lock");
            if let Some(entry) = store.get_mut(&(org_a, "alert-shared".to_owned())) {
                entry.status = "acknowledged".to_owned();
            }
        }

        let store = state.alert_store.lock().expect("lock");
        let a_status = store
            .get(&(org_a, "alert-shared".to_owned()))
            .expect("org_A entry must exist")
            .status
            .clone();
        let b_status = store
            .get(&(org_b, "alert-shared".to_owned()))
            .expect("org_B entry must exist")
            .status
            .clone();

        assert_eq!(
            a_status, "acknowledged",
            "AC-001/TV-3.2.001-03: org_A status must reflect its own mutation"
        );
        assert_eq!(
            b_status, "open",
            "AC-001/TV-3.2.001-03: org_B status must be unaffected by org_A mutation"
        );
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // AC-004 — access_token_allowlist is org-agnostic (ADR-031 §D3-a rule 3)
    //
    // The real Cyberint API issues API keys at account level (not per-org).
    // register_access_token / is_valid_access_token have no OrgId parameter.
    // A token registered once is valid for all orgs on this clone instance.
    // ═══════════════════════════════════════════════════════════════════════════

    /// AC-004: register_access_token registers a token in the global allowlist;
    /// is_valid_access_token confirms validity for any org context.
    ///
    /// ADR-031 §D3-a rule 3: session-store becomes static-auth registry.
    #[test]
    fn test_BC_2_01_017_access_token_registered_is_valid_globally() {
        let (org_a, _org_b) = org_pair();
        let state = CyberintState::with_org_id_and_admin_token(
            org_a,
            vec![],
            vec![],
            vec![],
            "admin".into(),
        );

        state.register_access_token("test-api-key-abc123".to_owned());

        assert!(
            state.is_valid_access_token("test-api-key-abc123"),
            "AC-004: registered access_token must be valid"
        );
    }

    /// AC-004: token NOT registered must not validate.
    #[test]
    fn test_BC_2_01_017_unregistered_access_token_is_invalid() {
        let (org_a, _org_b) = org_pair();
        let state = CyberintState::with_org_id_and_admin_token(
            org_a,
            vec![],
            vec![],
            vec![],
            "admin".into(),
        );

        assert!(
            !state.is_valid_access_token("not-registered-token"),
            "AC-004: unregistered token must not validate"
        );
    }

    /// AC-004: multiple tokens can be registered; each validates independently.
    #[test]
    fn test_BC_2_01_017_multiple_tokens_each_validate_independently() {
        let (org_a, _org_b) = org_pair();
        let state = CyberintState::with_org_id_and_admin_token(
            org_a,
            vec![],
            vec![],
            vec![],
            "admin".into(),
        );

        state.register_access_token("token-one".to_owned());
        state.register_access_token("token-two".to_owned());

        assert!(
            state.is_valid_access_token("token-one"),
            "AC-004: first registered token must be valid"
        );
        assert!(
            state.is_valid_access_token("token-two"),
            "AC-004: second registered token must be valid"
        );
        assert!(
            !state.is_valid_access_token("token-three"),
            "AC-004: unregistered third token must be invalid"
        );
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // AC-005 — build_alert_store accepts OrgId parameter (BC-3.2.001 invariant 1)
    // ═══════════════════════════════════════════════════════════════════════════

    /// BC-3.2.001 invariant 1: composite key (OrgId, String) is the exclusive keying
    /// scheme. Verify that build_alert_store (exercised via with_org_id_and_admin_token)
    /// produces keys of the form (org_id, alert_id) for all fixture alerts.
    #[test]
    fn test_BC_3_2_001_build_alert_store_keys_are_org_composite() {
        let (org_a, org_b) = org_pair();

        let alert_ids = ["CYB-AC005-001", "CYB-AC005-002", "CYB-AC005-003"];
        let fixtures: Vec<Alert> = alert_ids
            .iter()
            .map(|id| Alert {
                alert_id: id.to_string(),
                title: format!("Alert {id}"),
                severity: "medium".to_owned(),
                status: "open".to_owned(),
                created_at: serde_json::json!("2024-01-01T00:00:00Z"),
                source: "test".to_owned(),
                alert_type: "test".to_owned(),
                affected_assets: vec![],
            })
            .collect();

        let state = CyberintState::with_org_id_and_admin_token(
            org_a,
            fixtures.clone(),
            vec![],
            vec![],
            "admin".into(),
        );

        let store = state.alert_store.lock().expect("lock");

        for id in &alert_ids {
            // Key under org_A must exist.
            assert!(
                store.get(&(org_a, id.to_string())).is_some(),
                "AC-005: (org_A, {id}) must be present after build_alert_store with org_A"
            );
            // Key under org_B must NOT exist (wrong org).
            assert!(
                store.get(&(org_b, id.to_string())).is_none(),
                "AC-005: (org_B, {id}) must be absent — build_alert_store must key under the supplied org_id"
            );
        }
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // AC-006 — reset_for clears alert_store for one org; allowlist is unaffected
    //
    // NOTE (ADR-031 §D3-a rule 3): access_token_allowlist is org-agnostic.
    // reset_for(org_A) clears alert_store entries for org_A but does NOT remove
    // access tokens from the allowlist (tokens are account-level, not per-org).
    // ═══════════════════════════════════════════════════════════════════════════

    /// BC-3.2.001 edge case EC-004: reset_for(org_A) removes all (org_A, *) alert entries;
    /// (org_B, *) entries remain intact.
    ///
    /// TV-3.2.001-05: after reset_for(org_A), org_A alert = empty; org_B alert = intact.
    #[test]
    fn test_BC_3_2_001_reset_for_removes_org_a_alert_entries_preserves_org_b() {
        let (org_a, org_b) = org_pair();
        let state = state_with_two_orgs(org_a, org_b, "alert-reset-test");

        // Pre-condition: both orgs have entries.
        {
            let store = state.alert_store.lock().expect("lock");
            assert!(
                store.get(&(org_a, "alert-reset-test".to_owned())).is_some(),
                "AC-006 pre-condition: org_A entry must exist before reset_for"
            );
            assert!(
                store.get(&(org_b, "alert-reset-test".to_owned())).is_some(),
                "AC-006 pre-condition: org_B entry must exist before reset_for"
            );
        }

        // Reset only org_A.
        state.reset_for(org_a);

        // Post-condition: org_A entries are gone; org_B entries survive.
        let store = state.alert_store.lock().expect("lock");
        assert!(
            store.get(&(org_a, "alert-reset-test".to_owned())).is_none(),
            "AC-006: org_A alert entry must be removed by reset_for(org_A)"
        );
        assert!(
            store.get(&(org_b, "alert-reset-test".to_owned())).is_some(),
            "AC-006: org_B alert entry must survive reset_for(org_A)"
        );
    }

    /// AC-006 + ADR-031 §D3-a rule 3: reset_for(org_A) does NOT remove tokens from
    /// the access_token_allowlist (the allowlist is org-agnostic).
    ///
    /// This verifies the architectural invariant that tokens persist across per-org
    /// resets — matching the real Cyberint API where API keys are account-level.
    #[test]
    fn test_BC_3_2_001_reset_for_does_not_clear_access_token_allowlist() {
        let (org_a, _org_b) = org_pair();
        let state = CyberintState::with_org_id_and_admin_token(
            org_a,
            vec![],
            vec![],
            vec![],
            "admin".into(),
        );

        state.register_access_token("persistent-api-key".to_owned());

        // reset_for(org_A) must NOT clear the allowlist.
        state.reset_for(org_a);

        assert!(
            state.is_valid_access_token("persistent-api-key"),
            "AC-006/ADR-031 §D3-a: access_token must persist across reset_for — allowlist is org-agnostic"
        );
    }

    /// BC-3.2.001 EC-004: reset_for clears alert_store for org_A;
    /// org_B alert entries survive; allowlist is unaffected.
    #[test]
    fn test_BC_3_2_001_reset_for_clears_alert_store_for_org_a_allowlist_unaffected() {
        let (org_a, org_b) = org_pair();
        let state = state_with_two_orgs(org_a, org_b, "alert-atomic");

        state.register_access_token("shared-api-key".to_owned());

        state.reset_for(org_a);

        // alert_store: org_A gone, org_B intact.
        {
            let store = state.alert_store.lock().expect("lock");
            assert!(
                store.get(&(org_a, "alert-atomic".to_owned())).is_none(),
                "AC-006: alert_store org_A entry must be absent after reset_for"
            );
            assert!(
                store.get(&(org_b, "alert-atomic".to_owned())).is_some(),
                "AC-006: alert_store org_B entry must survive reset_for"
            );
        }

        // access_token_allowlist: org-agnostic — not cleared by reset_for.
        assert!(
            state.is_valid_access_token("shared-api-key"),
            "ADR-031 §D3-a: access_token_allowlist must survive reset_for — account-level token"
        );
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // AC-007 — OrgId-flipping proptest (BC-3.2.001 VP-3.2.001-03)
    //
    // Covers VP-077 through VP-086.
    // ═══════════════════════════════════════════════════════════════════════════

    use proptest::prelude::*;

    /// Arbitrary `OrgId` strategy: generate a random UUID v4 and wrap via `from_uuid`
    /// (bypasses v7 assertion intentionally for proptest adversarial inputs per ADR-008 §8 Q2).
    fn arb_org_id() -> impl Strategy<Value = OrgId> {
        any::<[u8; 16]>().prop_map(|bytes| OrgId::from_uuid(Uuid::from_bytes(bytes)))
    }

    /// Arbitrary distinct org pair: generate two OrgIds and discard cases where they
    /// are equal (negligible probability for random 128-bit values).
    fn arb_distinct_org_pair() -> impl Strategy<Value = (OrgId, OrgId)> {
        (arb_org_id(), arb_org_id()).prop_filter("orgs must differ", |(a, b)| a != b)
    }

    /// Arbitrary token string (non-empty, up to 64 chars).
    fn arb_token() -> impl Strategy<Value = String> {
        "[a-z0-9-]{1,64}".prop_map(|s| s)
    }

    /// VP-3.2.001-01: Cross-org alert lookup always returns None.
    ///
    /// Given any two distinct orgs and any alert_id:
    /// - Store an AlertStatus under org_A.
    /// - alert_store lookup under org_B must return None.
    proptest! {
        #[test]
        fn test_BC_3_2_001_invariant_cross_org_alert_lookup_always_none(
            (org_a, org_b) in arb_distinct_org_pair(),
            alert_id in "[a-z0-9-]{1,32}",
        ) {
            let fixture = vec![Alert {
                alert_id: alert_id.clone(),
                title: "prop alert".to_owned(),
                severity: "low".to_owned(),
                status: "open".to_owned(),
                created_at: serde_json::json!("2024-01-01T00:00:00Z"),
                source: "prop".to_owned(),
                alert_type: "prop".to_owned(),
                affected_assets: vec![],
            }];
            let state = CyberintState::with_org_id_and_admin_token(
                org_a, fixture, vec![], vec![], "admin".into(),
            );

            let store = state.alert_store.lock().expect("lock");
            prop_assert!(
                store.get(&(org_b, alert_id.clone())).is_none(),
                "VP-3.2.001-01: alert written under org_A must never appear under org_B"
            );
        }
    }

    /// VP-3.2.001-03 (mutation kill): OrgId-flipping — replacing lookup org with a
    /// different org must always return None, for any shared alert_id.
    ///
    /// This test directly targets the mutation `org_id_A → org_id_B` in the lookup
    /// key construction (TD-DTU-MUTATE-COVERAGE-001).
    proptest! {
        #[test]
        fn test_BC_3_2_001_invariant_org_id_flip_kills_mutation(
            (org_a, org_b) in arb_distinct_org_pair(),
            alert_id in "[a-z0-9-]{1,32}",
        ) {
            let fixture = vec![Alert {
                alert_id: alert_id.clone(),
                title: "mutation test".to_owned(),
                severity: "critical".to_owned(),
                status: "open".to_owned(),
                created_at: serde_json::json!("2024-06-01T00:00:00Z"),
                source: "prop".to_owned(),
                alert_type: "prop".to_owned(),
                affected_assets: vec![],
            }];
            let state = CyberintState::with_org_id_and_admin_token(
                org_a, fixture, vec![], vec![], "admin".into(),
            );

            let store = state.alert_store.lock().expect("lock");
            // If a mutant replaced org_a with org_b in the lookup, it would still return None
            // (because org_b's entry doesn't exist).  We assert from the write side:
            // the entry MUST exist under the correct org.
            prop_assert!(
                store.get(&(org_a, alert_id.clone())).is_some(),
                "VP-3.2.001-03: entry must be stored under the correct org_A key"
            );
            prop_assert!(
                store.get(&(org_b, alert_id.clone())).is_none(),
                "VP-3.2.001-03: OrgId-flipped lookup must return None (mutation killed)"
            );
        }
    }

    /// VP-3.2.001-04: reset_for(org_A) selectivity for alert_store.
    ///
    /// For any two distinct orgs and any alert_id + token pair:
    /// - Write entries for both orgs.
    /// - Call reset_for(org_A).
    /// - org_A alert entries must be absent; org_B alert entries must be intact.
    /// - access_token registered before reset_for must still be valid (org-agnostic).
    proptest! {
        #[test]
        fn test_BC_3_2_001_invariant_reset_for_selectivity(
            (org_a, org_b) in arb_distinct_org_pair(),
            alert_id in "[a-z0-9-]{1,32}",
            token in arb_token(),
        ) {
            let fixture = vec![Alert {
                alert_id: alert_id.clone(),
                title: "reset selectivity".to_owned(),
                severity: "medium".to_owned(),
                status: "open".to_owned(),
                created_at: serde_json::json!("2024-01-01T00:00:00Z"),
                source: "prop".to_owned(),
                alert_type: "prop".to_owned(),
                affected_assets: vec![],
            }];
            let state = CyberintState::with_org_id_and_admin_token(
                org_a, fixture, vec![], vec![], "admin".into(),
            );

            // Add org_B entries manually.
            {
                let mut store = state.alert_store.lock().expect("lock");
                store.insert(
                    (org_b, alert_id.clone()),
                    AlertStatus {
                        alert_id: alert_id.clone(),
                        status: "open".to_owned(),
                        closed: false,
                    },
                );
            }

            // Register token (org-agnostic).
            state.register_access_token(token.clone());

            state.reset_for(org_a);

            {
                let store = state.alert_store.lock().expect("lock");
                prop_assert!(
                    store.get(&(org_a, alert_id.clone())).is_none(),
                    "VP-3.2.001-04: alert_store org_A entries must be absent after reset_for"
                );
                prop_assert!(
                    store.get(&(org_b, alert_id.clone())).is_some(),
                    "VP-3.2.001-04: alert_store org_B entries must survive reset_for"
                );
            }
            // access_token_allowlist is org-agnostic — must survive reset_for.
            prop_assert!(
                state.is_valid_access_token(&token),
                "ADR-031 §D3-a: access_token must survive reset_for(org_A) — account-level key"
            );
        }
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // HTTP-layer multi-tenant tests
    //
    // These exercise the full route stack via reqwest against a running clone
    // instance.  The new auth model (ADR-031 §D3-a) uses `access_token` cookie;
    // there is no POST /login endpoint (AC-001).
    //
    // ═══════════════════════════════════════════════════════════════════════════

    /// HTTP AC-001: POST /login must not exist — the DTU has no login endpoint.
    ///
    /// This test exercises ADR-031 §D3-a rule 1: the DTU clone must NOT implement
    /// a fake login step; the real Cyberint API has no such endpoint.
    #[tokio::test]
    async fn test_BC_2_01_017_http_post_login_returns_404() {
        let (_clone, base_url, _admin_token, client) = start_clone().await;

        let resp = client
            .post(format!("{base_url}/login"))
            .json(&serde_json::json!({}))
            .send()
            .await
            .expect("HTTP request must not produce a network error");

        assert_eq!(
            resp.status().as_u16(),
            404,
            "AC-001/ADR-031 §D3-a: POST /login must return 404 — endpoint removed"
        );
    }

    /// HTTP AC-003: GET /api/v1/alerts requires valid access_token cookie (AC-002 + AC-003).
    ///
    /// - No cookie → 401
    /// - Valid access_token cookie → 200
    #[tokio::test]
    async fn test_BC_2_01_017_http_access_token_cookie_auth_required_for_alerts() {
        let (_clone, base_url, admin_token, client) = start_clone().await;

        // Provision a demo access_token via /dtu/configure.
        let demo_token = "integration-test-api-key-12345";
        let configure_resp = client
            .post(format!("{base_url}/dtu/configure"))
            .header("X-Admin-Token", &admin_token)
            .json(&serde_json::json!({ "access_token": demo_token }))
            .send()
            .await
            .expect("configure must not produce a network error");

        assert_eq!(
            configure_resp.status().as_u16(),
            200,
            "HTTP test setup: POST /dtu/configure must return 200"
        );

        // Without cookie → 401.
        let no_cookie_resp = client
            .get(format!("{base_url}/api/v1/alerts"))
            .send()
            .await
            .expect("GET alerts without cookie must not produce a network error");

        assert_eq!(
            no_cookie_resp.status().as_u16(),
            401,
            "AC-002: GET /api/v1/alerts without cookie must return 401"
        );

        // With valid access_token cookie → 200.
        let with_cookie_resp = client
            .get(format!("{base_url}/api/v1/alerts"))
            .header("Cookie", format!("access_token={demo_token}"))
            .send()
            .await
            .expect("GET alerts with valid cookie must not produce a network error");

        assert_eq!(
            with_cookie_resp.status().as_u16(),
            200,
            "AC-003: GET /api/v1/alerts with valid access_token cookie must return 200"
        );
    }

    /// HTTP AC-003: GET /api/v1/threat-intel requires valid access_token cookie.
    #[tokio::test]
    async fn test_BC_2_01_017_http_access_token_cookie_auth_required_for_threats() {
        let (_clone, base_url, admin_token, client) = start_clone().await;

        let demo_token = "integration-test-api-key-threats-67890";
        let configure_resp = client
            .post(format!("{base_url}/dtu/configure"))
            .header("X-Admin-Token", &admin_token)
            .json(&serde_json::json!({ "access_token": demo_token }))
            .send()
            .await
            .expect("configure must not produce a network error");

        assert_eq!(
            configure_resp.status().as_u16(),
            200,
            "HTTP test setup: POST /dtu/configure must return 200"
        );

        // Without cookie → 401.
        let no_cookie_resp = client
            .get(format!("{base_url}/api/v1/threat-intel"))
            .send()
            .await
            .expect("GET threat-intel without cookie must not produce a network error");

        assert_eq!(
            no_cookie_resp.status().as_u16(),
            401,
            "AC-003 (threats): GET /api/v1/threat-intel without cookie must return 401"
        );

        // With valid access_token cookie → 200.
        let with_cookie_resp = client
            .get(format!("{base_url}/api/v1/threat-intel"))
            .header("Cookie", format!("access_token={demo_token}"))
            .send()
            .await
            .expect("GET threat-intel with valid cookie must not produce a network error");

        assert_eq!(
            with_cookie_resp.status().as_u16(),
            200,
            "AC-003 (threats): GET /api/v1/threat-intel with valid access_token cookie must return 200"
        );
    }

    /// HTTP AC-006: reset_all via POST /dtu/reset invalidates registered tokens.
    ///
    /// After /dtu/reset, previously valid access_token must be rejected.
    #[tokio::test]
    async fn test_BC_2_01_017_http_reset_all_clears_access_token_allowlist() {
        let (_clone, base_url, admin_token, client) = start_clone().await;

        let demo_token = "reset-test-token-abc";

        // Provision token.
        client
            .post(format!("{base_url}/dtu/configure"))
            .header("X-Admin-Token", &admin_token)
            .json(&serde_json::json!({ "access_token": demo_token }))
            .send()
            .await
            .expect("configure must succeed");

        // Verify token is valid.
        let before_reset = client
            .get(format!("{base_url}/api/v1/alerts"))
            .header("Cookie", format!("access_token={demo_token}"))
            .send()
            .await
            .expect("GET before reset must succeed");

        assert_eq!(
            before_reset.status().as_u16(),
            200,
            "HTTP setup: token must be valid before reset"
        );

        // Reset all state.
        let reset_resp = client
            .post(format!("{base_url}/dtu/reset"))
            .header("X-Admin-Token", &admin_token)
            .send()
            .await
            .expect("reset must not produce a network error");

        assert_eq!(
            reset_resp.status().as_u16(),
            200,
            "HTTP AC-006: /dtu/reset must return 200"
        );

        // Token must now be invalid — reset cleared the allowlist.
        let after_reset = client
            .get(format!("{base_url}/api/v1/alerts"))
            .header("Cookie", format!("access_token={demo_token}"))
            .send()
            .await
            .expect("GET after reset must not produce a network error");

        assert_eq!(
            after_reset.status().as_u16(),
            401,
            "AC-006: access_token must be invalid after reset_all"
        );
    }
}
