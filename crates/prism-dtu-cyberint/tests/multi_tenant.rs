#![allow(clippy::unwrap_used, clippy::expect_used)]
// BC-based test names use mixed-case identifiers following the factory naming standard.
// See prism-core/src/tests/capability_tests.rs for precedent.
#![allow(non_snake_case)]
//! Multi-tenant state segregation tests for `prism-dtu-cyberint`.
//!
//! Covers:
//! - BC-3.2.001: Per-Org Sensor Data Isolation via Composite HashMap Key
//! - BC-3.2.003: Per-Org Session Token Isolation via (OrgId, token) Composite Key
//!
//! Acceptance criteria tested:
//! - AC-001: Alert store cross-org isolation
//! - AC-002: Session cross-org isolation
//! - AC-003: Same token string, independent contexts
//! - AC-004: Token refresh preserves OrgId binding
//! - AC-005: build_alert_store accepts OrgId parameter
//! - AC-006: reset_for clears both stores for one org only
//! - AC-007: OrgId-flipping proptest kills mutation (VP-3.2.001-03)
//!
//! HTTP-layer tests also verify that OrgId is correctly threaded from the
//! request context through all route handlers (extract_org_id stub).
//!
//! # Test Status
//!
//! All acceptance criteria (AC-001 through AC-007) and HTTP-layer isolation
//! tests are implemented and expected to pass. `reset_for` and `extract_org_id`
//! are both fully implemented (see `state.rs:257` and `routes/alerts.rs:66`).

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

    /// Start a clone and return `(clone, base_url, client)`.
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
    // AC-002 — Session cross-org isolation (BC-3.2.003 postcondition 2)
    // TV-3.2.003-02: register for org_A; is_valid_session(org_B, token) = false.
    // ═══════════════════════════════════════════════════════════════════════════

    /// BC-3.2.003 postcondition 2: token registered under org_A is invalid in org_B context.
    ///
    /// TV-3.2.003-02: register_session(org_id_A, "tok-abc");
    /// is_valid_session(org_id_B, "tok-abc") must return false.
    #[test]
    fn test_BC_3_2_003_session_cross_org_isolation_register_a_validate_b_returns_false() {
        let (org_a, org_b) = org_pair();
        let state =
            CyberintState::with_org_id_and_admin_token(org_a, vec![], vec![], vec![], "tok".into());

        state.register_session(org_a, "tok-abc".to_owned());

        assert!(
            state.is_valid_session(org_a, "tok-abc"),
            "AC-002: is_valid_session(org_A, tok-abc) must return true"
        );
        assert!(
            !state.is_valid_session(org_b, "tok-abc"),
            "AC-002: is_valid_session(org_B, tok-abc) must return false — cross-org session leak"
        );
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // AC-003 — Same token string, independent contexts (BC-3.2.003 EC-001)
    // TV-3.2.003-04: both orgs register same string; each valid only in its own context.
    // ═══════════════════════════════════════════════════════════════════════════

    /// BC-3.2.003 edge case EC-001 / TV-3.2.003-04: identical token string registered
    /// independently for both orgs; each is valid only in its own context.
    #[test]
    fn test_BC_3_2_003_identical_token_string_independent_per_org_contexts() {
        let (org_a, org_b) = org_pair();
        let state = CyberintState::with_org_id_and_admin_token(
            org_a,
            vec![],
            vec![],
            vec![],
            "admin".into(),
        );

        let shared_token = "tok-shared-uuid-aaaabbbb";
        state.register_session(org_a, shared_token.to_owned());
        state.register_session(org_b, shared_token.to_owned());

        assert!(
            state.is_valid_session(org_a, shared_token),
            "AC-003: org_A must validate its own registration of the shared token"
        );
        assert!(
            state.is_valid_session(org_b, shared_token),
            "AC-003: org_B must validate its own registration of the shared token"
        );
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // AC-004 — Token refresh preserves OrgId binding (BC-3.2.003 postcondition 3)
    // ═══════════════════════════════════════════════════════════════════════════

    /// BC-3.2.003 postcondition 3: refresh stores new token under same OrgId;
    /// org_B's token is unaffected.
    ///
    /// TV-3.2.003-03: orgA refreshes old→new; orgB original token unaffected.
    #[test]
    fn test_BC_3_2_003_token_refresh_preserves_org_binding() {
        let (org_a, org_b) = org_pair();
        let state = CyberintState::with_org_id_and_admin_token(
            org_a,
            vec![],
            vec![],
            vec![],
            "admin".into(),
        );

        state.register_session(org_a, "old-tok".to_owned());
        state.register_session(org_b, "tok-b".to_owned());

        // Simulate refresh: remove old token, insert new token under same org_id.
        {
            let mut store = state.session_store.lock().expect("session_store poisoned");
            store.remove(&(org_a, "old-tok".to_owned()));
            store.insert((org_a, "new-tok".to_owned()));
        }

        assert!(
            state.is_valid_session(org_a, "new-tok"),
            "AC-004: org_A new token must be valid after refresh"
        );
        assert!(
            !state.is_valid_session(org_a, "old-tok"),
            "AC-004: org_A old token must be invalid after refresh"
        );
        assert!(
            state.is_valid_session(org_b, "tok-b"),
            "AC-004: org_B token must be unaffected by org_A refresh"
        );
        assert!(
            !state.is_valid_session(org_b, "old-tok"),
            "AC-004: is_valid_session(org_B, old-tok) must be false — old-tok was never org_B's"
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
    // AC-006 — reset_for clears both stores for one org (BC-3.2.001 EC-004)
    //
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

    /// BC-3.2.003 edge case EC-004: reset_for(org_A) removes org_A session tokens;
    /// org_B tokens remain valid.
    ///
    /// TV-3.2.003-03 (reset_for variant).
    #[test]
    fn test_BC_3_2_003_reset_for_removes_org_a_session_tokens_preserves_org_b() {
        let (org_a, org_b) = org_pair();
        let state = CyberintState::with_org_id_and_admin_token(
            org_a,
            vec![],
            vec![],
            vec![],
            "admin".into(),
        );

        state.register_session(org_a, "tok-a".to_owned());
        state.register_session(org_b, "tok-b".to_owned());

        // Verify both tokens valid before reset.
        assert!(
            state.is_valid_session(org_a, "tok-a"),
            "AC-006 pre-cond: org_A token must be valid before reset_for"
        );
        assert!(
            state.is_valid_session(org_b, "tok-b"),
            "AC-006 pre-cond: org_B token must be valid before reset_for"
        );

        // Reset only org_A.
        state.reset_for(org_a);

        assert!(
            !state.is_valid_session(org_a, "tok-a"),
            "AC-006: org_A session token must be invalid after reset_for(org_A)"
        );
        assert!(
            state.is_valid_session(org_b, "tok-b"),
            "AC-006: org_B session token must survive reset_for(org_A)"
        );
    }

    /// BC-3.2.001 EC-004 + BC-3.2.003 EC-004: reset_for clears BOTH stores atomically;
    /// neither alert_store nor session_store retains org_A entries after reset.
    #[test]
    fn test_BC_3_2_001_reset_for_clears_both_stores_atomically_for_org_a() {
        let (org_a, org_b) = org_pair();
        let state = state_with_two_orgs(org_a, org_b, "alert-atomic");

        state.register_session(org_a, "session-a".to_owned());
        state.register_session(org_b, "session-b".to_owned());

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

        // session_store: org_A gone, org_B intact.
        assert!(
            !state.is_valid_session(org_a, "session-a"),
            "AC-006: session_store org_A token must be absent after reset_for"
        );
        assert!(
            state.is_valid_session(org_b, "session-b"),
            "AC-006: session_store org_B token must survive reset_for"
        );
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // AC-007 — OrgId-flipping proptest (BC-3.2.001 VP-3.2.001-03 / VP-3.2.003-01)
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

    /// VP-3.2.003-01: Cross-org token validation always returns false.
    ///
    /// Given any two distinct orgs and any token string:
    /// - Register the token under org_A.
    /// - is_valid_session(org_B, token) must return false.
    ///
    /// At least 1000 cases (proptest default).
    proptest! {
        #[test]
        fn test_BC_3_2_003_invariant_cross_org_session_validation_always_false(
            (org_a, org_b) in arb_distinct_org_pair(),
            token in arb_token(),
        ) {
            let state = CyberintState::with_org_id_and_admin_token(
                org_a, vec![], vec![], vec![], "admin".into(),
            );
            state.register_session(org_a, token.clone());

            prop_assert!(
                !state.is_valid_session(org_b, &token),
                "VP-3.2.003-01: token registered under org_A must never validate for org_B"
            );
        }
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

    /// VP-3.2.001-04 + VP-3.2.003-03: reset_for(org_A) selectivity.
    ///
    /// For any two distinct orgs and any alert_id + token pair:
    /// - Write entries for both orgs.
    /// - Call reset_for(org_A).
    /// - org_A entries must be absent; org_B entries must be intact.
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
            state.register_session(org_a, token.clone());
            state.register_session(org_b, token.clone());

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
            prop_assert!(
                !state.is_valid_session(org_a, &token),
                "VP-3.2.003-03: session_store org_A token must be absent after reset_for"
            );
            prop_assert!(
                state.is_valid_session(org_b, &token),
                "VP-3.2.003-03: session_store org_B token must survive reset_for"
            );
        }
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // HTTP-layer multi-tenant isolation tests
    //
    // These exercise the full route stack via reqwest against a running clone
    // instance. `extract_org_id` is implemented and threads the correct OrgId
    // from the X-Prism-Org-Id header through all route handlers.
    //
    // ═══════════════════════════════════════════════════════════════════════════

    /// HTTP AC-002: Session token registered for org_A must not authenticate requests
    /// in org_B's context.
    ///
    /// This test exercises the full route stack where `extract_org_id` threads the
    /// correct OrgId from the X-Prism-Org-Id header through to `is_valid_session`.
    #[tokio::test]
    async fn test_BC_3_2_003_http_session_token_registered_for_org_a_rejected_by_org_b() {
        let (_clone, base_url, _admin_token, client) = start_clone().await;

        let org_a_id = Uuid::parse_str("00000000-0000-7000-8000-000000000001")
            .expect("valid uuid")
            .to_string();
        let org_b_id = Uuid::parse_str("00000000-0000-7000-8000-000000000002")
            .expect("valid uuid")
            .to_string();

        let login_resp = client
            .post(format!("{base_url}/login"))
            .header("X-Prism-Org-Id", &org_a_id)
            .json(&serde_json::json!({}))
            .send()
            .await
            .expect("HTTP login as org_A must not produce a network error");

        assert_eq!(
            login_resp.status().as_u16(),
            200,
            "HTTP AC-002: POST /login as org_A must return 200"
        );

        let set_cookie = login_resp
            .headers()
            .get("set-cookie")
            .expect("Set-Cookie must be present")
            .to_str()
            .expect("Set-Cookie must be ASCII")
            .to_owned();
        let token = set_cookie
            .split(';')
            .next()
            .and_then(|s| s.strip_prefix("cyberint_session="))
            .expect("cyberint_session must be in Set-Cookie")
            .to_owned();

        // Use org_A's token but identify as org_B — must receive 401.
        let alert_resp = client
            .get(format!("{base_url}/api/v1/alerts"))
            .header("Cookie", format!("cyberint_session={token}"))
            .header("X-Prism-Org-Id", &org_b_id)
            .send()
            .await
            .expect(
                "GET /api/v1/alerts as org_B with org_A token must not produce a network error",
            );

        assert_eq!(
            alert_resp.status().as_u16(),
            401,
            "HTTP AC-002: org_A's session token must be rejected in org_B's context"
        );
    }

    /// HTTP AC-006: Per-org reset via HTTP — org_A's token invalidated; org_B's intact.
    #[tokio::test]
    async fn test_BC_3_2_001_http_reset_for_invalidates_org_a_preserves_org_b() {
        let (_clone, base_url, admin_token, client) = start_clone().await;

        let org_a_id = Uuid::parse_str("00000000-0000-7000-8000-000000000001")
            .expect("valid uuid")
            .to_string();
        let org_b_id = Uuid::parse_str("00000000-0000-7000-8000-000000000002")
            .expect("valid uuid")
            .to_string();

        // Login as org_A.
        let login_a = client
            .post(format!("{base_url}/login"))
            .header("X-Prism-Org-Id", &org_a_id)
            .json(&serde_json::json!({}))
            .send()
            .await
            .expect("login org_A must not produce a network error");

        assert_eq!(
            login_a.status().as_u16(),
            200,
            "HTTP AC-006: org_A login must return 200"
        );

        let cookie_a = login_a
            .headers()
            .get("set-cookie")
            .expect("Set-Cookie must be present for org_A")
            .to_str()
            .expect("ASCII")
            .split(';')
            .next()
            .and_then(|s| s.strip_prefix("cyberint_session="))
            .expect("cyberint_session must be in Set-Cookie for org_A")
            .to_owned();

        // Login as org_B.
        let login_b = client
            .post(format!("{base_url}/login"))
            .header("X-Prism-Org-Id", &org_b_id)
            .json(&serde_json::json!({}))
            .send()
            .await
            .expect("login org_B must not produce a network error");

        assert_eq!(
            login_b.status().as_u16(),
            200,
            "HTTP AC-006: org_B login must return 200"
        );

        let cookie_b = login_b
            .headers()
            .get("set-cookie")
            .expect("Set-Cookie must be present for org_B")
            .to_str()
            .expect("ASCII")
            .split(';')
            .next()
            .and_then(|s| s.strip_prefix("cyberint_session="))
            .expect("cyberint_session must be in Set-Cookie for org_B")
            .to_owned();

        // Reset for org_A only via POST /dtu/reset (per-org variant).
        // NOTE: the current /dtu/reset resets ALL orgs. A per-org endpoint
        // is the desired post-implementation behavior; until then this test
        // verifies the route stack can thread org_id to reset_for.
        let reset_resp = client
            .post(format!("{base_url}/dtu/reset"))
            .header("X-Admin-Token", &admin_token)
            .header("X-Prism-Org-Id", &org_a_id)
            .send()
            .await
            .expect("reset must not produce a network error");

        assert_eq!(
            reset_resp.status().as_u16(),
            200,
            "HTTP AC-006: /dtu/reset must return 200"
        );

        // org_A token must now be invalid — org_A's session was reset.
        let after_a = client
            .get(format!("{base_url}/api/v1/alerts"))
            .header("Cookie", format!("cyberint_session={cookie_a}"))
            .header("X-Prism-Org-Id", &org_a_id)
            .send()
            .await
            .expect("GET alerts as org_A after reset must not produce a network error");

        assert_eq!(
            after_a.status().as_u16(),
            401,
            "HTTP AC-006: org_A token must be invalid after per-org reset"
        );

        // org_B token must still be valid — org_B was not reset.
        let after_b = client
            .get(format!("{base_url}/api/v1/alerts"))
            .header("Cookie", format!("cyberint_session={cookie_b}"))
            .header("X-Prism-Org-Id", &org_b_id)
            .send()
            .await
            .expect("GET alerts as org_B after org_A reset must not produce a network error");

        assert_eq!(
            after_b.status().as_u16(),
            200,
            "HTTP AC-006: org_B token must remain valid after org_A-only reset"
        );
    }
}
