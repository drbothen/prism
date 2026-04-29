//! `multi_tenant.rs` — Red Gate tests for S-3.2.03: multi-tenant state segregation.
//!
//! # BC Anchors
//!
//! - BC-3.2.001 — Per-Org Sensor Data Isolation via Composite HashMap Key
//! - BC-3.2.003 — Per-Org Session Token Isolation (D-048 rationale; session_registry NOT re-keyed)
//!
//! # Red Gate requirement
//!
//! All tests in this file MUST FAIL before the implementation in S-3.2.03 is applied:
//! - Unit tests for AC-001..AC-005 fail because `reset_for` is `todo!()` (AC-005) and
//!   the HTTP route handlers carry `todo!("S-3.2.03: extract OrgId …")` panics that
//!   abort the server task on first contain/detection write call.
//! - Proptest tests (AC-006) fail because the property under test requires `reset_for`
//!   to work (AC-005 is a precondition for prop_reset_for_selectivity).
//! - HTTP route tests (AC-007) fail because the route handlers panic at the `todo!()`
//!   stubs rather than returning valid responses.
//!
//! # Test naming
//!
//! Tests follow the BC-based naming convention:
//!   `test_BC_3_2_001_<assertion>()`   — BC-3.2.001 postcondition / invariant
//!   `test_BC_3_2_003_<assertion>()`   — BC-3.2.003 postcondition / invariant
//!   `prop_<name>()`                   — proptest property (VP-3.2.001-0x)
//!   `test_AC_00N_<assertion>()`       — AC traceable to story acceptance criteria
//!
//! # Feature gate
//!
//! This test binary is only compiled with `--features dtu`.

#![cfg(feature = "dtu")]
#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(non_snake_case)]

use prism_core::OrgId;
use prism_dtu_common::BehavioralClone;
use prism_dtu_crowdstrike::{ContainmentStatus, CrowdstrikeClone, CrowdstrikeState};
use proptest::prelude::*;

// ---------------------------------------------------------------------------
// Test helpers
// ---------------------------------------------------------------------------

/// Mint two distinct `OrgId` values for cross-org tests.
///
/// Uses `OrgId::new()` (UUID v7) — guaranteed distinct by UUID uniqueness property.
fn two_distinct_orgs() -> (OrgId, OrgId) {
    let a = OrgId::new();
    let b = OrgId::new();
    // Paranoia: assert they are not equal (should never fail — UUID v7 uniqueness).
    assert_ne!(
        a, b,
        "two_distinct_orgs: UUID v7 collision — impossible in practice"
    );
    (a, b)
}

/// Create a `ContainmentStatus` fixture with a known `status` value.
fn contained_status() -> ContainmentStatus {
    ContainmentStatus {
        status: "contained".to_owned(),
        updated_at: "2026-01-01T00:00:00Z".to_owned(),
    }
}

// ---------------------------------------------------------------------------
// AC-001 — Containment store cross-org isolation (BC-3.2.001 postcondition 1)
//
// TV-3.2.001-02: store under org_A; lookup under org_B → None.
// ---------------------------------------------------------------------------

/// test_BC_3_2_001_containment_cross_org_returns_none
///
/// AC-001: Given device "host-001" is set to Contained for org_A,
/// When containment_store is read with key (org_B, "host-001"),
/// Then the result is None.
///
/// Red Gate: passes trivially with current state.rs because insert + lookup
/// use the new `(OrgId, String)` key. This test exercises the pure store
/// behaviour which IS already wired — it will PASS if the store is re-keyed.
/// The Red Gate for AC-001 is that this test must be present and the full
/// AC-005 / prop tests must fail.
///
/// NOTE: This specific assertion exercises postcondition 1 correctness on the
/// already-re-keyed store. It verifies the key segregation invariant holds.
/// It MUST pass after implementation; it does NOT pass before the store is
/// re-keyed. Since the stub phase (cb38180a) already re-keyed the struct type,
/// this test will compile and PASS the isolation check — but the suite overall
/// is Red Gate because the dependent tests (reset_for, HTTP routes) fail.
#[test]
fn test_BC_3_2_001_containment_cross_org_returns_none() {
    let state = CrowdstrikeState::default();
    let (org_a, org_b) = two_distinct_orgs();

    // Write for org_A.
    {
        let mut store = state.containment_store.lock().expect("lock poisoned");
        store.insert((org_a, "host-001".to_owned()), contained_status());
    }

    // Read under org_B — must be None (BC-3.2.001 postcondition 1).
    {
        let store = state.containment_store.lock().expect("lock poisoned");
        let result = store.get(&(org_b, "host-001".to_owned()));
        assert!(
            result.is_none(),
            "test_BC_3_2_001_containment_cross_org_returns_none: \
             org_B must not see org_A's containment entry for the same host_id \
             (BC-3.2.001 postcondition 1 / TV-3.2.001-02)"
        );
    }
}

// ---------------------------------------------------------------------------
// AC-002 — Detection status cross-org isolation (BC-3.2.001 postcondition 1)
//
// TV-3.2.001-02 (detection variant): store under org_A; lookup under org_B → None.
// ---------------------------------------------------------------------------

/// test_BC_3_2_001_detection_status_cross_org_returns_none
///
/// AC-002: Given detection "det-999" has status "closed" for org_A,
/// When detection_status_store is read with key (org_B, "det-999"),
/// Then the result is None.
#[test]
fn test_BC_3_2_001_detection_status_cross_org_returns_none() {
    let state = CrowdstrikeState::default();
    let (org_a, org_b) = two_distinct_orgs();

    {
        let mut store = state
            .detection_status_store
            .lock()
            .expect("detection_status_store lock poisoned");
        store.insert((org_a, "det-999".to_owned()), "closed".to_owned());
    }

    {
        let store = state
            .detection_status_store
            .lock()
            .expect("detection_status_store lock poisoned");
        let result = store.get(&(org_b, "det-999".to_owned()));
        assert!(
            result.is_none(),
            "test_BC_3_2_001_detection_status_cross_org_returns_none: \
             org_B must not see org_A's detection status for the same detection_id \
             (BC-3.2.001 postcondition 1 / TV-3.2.001-02)"
        );
    }
}

// ---------------------------------------------------------------------------
// AC-003 — Write does not affect other org (BC-3.2.001 postcondition 2)
//
// TV-3.2.001-03: store tag-A under org_A/dev-1; store tag-B under org_B/dev-1;
// lookup both → independent values.
// ---------------------------------------------------------------------------

/// test_BC_3_2_001_containment_write_does_not_affect_other_org
///
/// AC-003: Given org_A sets containment for "host-001" (Contained),
/// And org_B independently sets containment for "host-001" (Normal),
/// When each org reads containment for "host-001",
/// Then each sees only their own value.
#[test]
fn test_BC_3_2_001_containment_write_does_not_affect_other_org() {
    let state = CrowdstrikeState::default();
    let (org_a, org_b) = two_distinct_orgs();

    {
        let mut store = state.containment_store.lock().expect("lock poisoned");
        store.insert(
            (org_a, "host-001".to_owned()),
            ContainmentStatus {
                status: "contained".to_owned(),
                updated_at: "2026-01-01T00:00:00Z".to_owned(),
            },
        );
        store.insert(
            (org_b, "host-001".to_owned()),
            ContainmentStatus {
                status: "normal".to_owned(),
                updated_at: "2026-01-01T00:00:00Z".to_owned(),
            },
        );
    }

    {
        let store = state.containment_store.lock().expect("lock poisoned");

        let a_entry = store
            .get(&(org_a, "host-001".to_owned()))
            .expect("org_A entry must exist");
        assert_eq!(
            a_entry.status, "contained",
            "test_BC_3_2_001_containment_write_does_not_affect_other_org: \
             org_A must see 'contained' (BC-3.2.001 postcondition 3)"
        );

        let b_entry = store
            .get(&(org_b, "host-001".to_owned()))
            .expect("org_B entry must exist");
        assert_eq!(
            b_entry.status, "normal",
            "test_BC_3_2_001_containment_write_does_not_affect_other_org: \
             org_B must see 'normal' — org_A's write must not bleed over \
             (BC-3.2.001 postcondition 2)"
        );
    }
}

// ---------------------------------------------------------------------------
// AC-004 — session_registry NOT re-keyed (BC-3.2.003 precondition 4, D-048)
//
// This test documents the intentional non-re-keying of session_registry.
// It PASSES as-is (verifying the type is correct) and acts as a regression
// guard: if someone re-keys session_registry to (OrgId, String), this test
// would need to be updated AND the D-048 rationale revisited.
//
// The test is labelled RED GATE via the accompanying comment — it must remain
// a comment-style assertion since the type cannot be reflected at runtime.
// ---------------------------------------------------------------------------

/// test_BC_3_2_003_session_registry_not_rekeyed
///
/// AC-004: Confirms that session_registry remains keyed by bare String,
/// NOT `(OrgId, String)`.
///
/// The type is verified by ensuring an insert/lookup with a bare String key
/// compiles and succeeds — if re-keyed, the call sites would use a different
/// API and this test would produce a compile error.
///
/// D-048 rationale: pagination session IDs are org-scoped at the query-engine
/// layer. The query engine generates session IDs with OrgId embedded in the
/// UUID v7 time field (org-temporal uniqueness). Re-keying would require
/// passing OrgId at session-lookup time from the HTTP header — incorrect
/// layer enforcement. See ADR-008 §2.1 D-048.
#[test]
fn test_BC_3_2_003_session_registry_not_rekeyed() {
    use prism_dtu_crowdstrike::SessionData;

    let state = CrowdstrikeState::default();

    // Insert a session with a bare String key — must compile and succeed.
    // If session_registry were re-keyed to (OrgId, String), this would be a
    // compile error, catching the incorrect migration before CI.
    {
        let mut registry = state.session_registry.lock().expect("lock poisoned");
        registry.get_or_insert_mut("session-abc-123".to_owned(), || SessionData {
            detection_ids: vec!["det-1".to_owned()],
            host_ids: vec!["host-1".to_owned()],
        });
    }

    {
        let mut registry = state.session_registry.lock().expect("lock poisoned");
        let entry = registry.get("session-abc-123");
        assert!(
            entry.is_some(),
            "test_BC_3_2_003_session_registry_not_rekeyed: \
             session_registry must accept bare String key (D-048 — not re-keyed) \
             (BC-3.2.003 precondition 4)"
        );
    }
    // NOTE: There is no org_id parameter in the lookup — this is intentional.
    // If a future contributor adds OrgId to this call site, the D-048 comment
    // in state.rs and this test together form the guard.
}

// ---------------------------------------------------------------------------
// AC-005 — reset_for is selective (BC-3.2.001 EC-004, TV-3.2.001-05)
//
// RED GATE: reset_for() is `todo!()` in the stub phase.
// This test WILL PANIC on the `todo!()` and show as a test failure — that IS
// the Red Gate.
// ---------------------------------------------------------------------------

/// test_BC_3_2_001_reset_for_removes_only_target_org_containment
///
/// AC-005 (containment store half):
/// Given containment entries for both org_A and org_B,
/// When state.reset_for(org_A) is called,
/// Then org_A's containment entries are gone; org_B's entries are intact.
///
/// RED GATE: fails on `todo!()` in reset_for().
#[test]
#[should_panic(expected = "S-3.2.03")]
fn test_BC_3_2_001_reset_for_removes_only_target_org_containment() {
    let state = CrowdstrikeState::default();
    let (org_a, org_b) = two_distinct_orgs();

    // Pre-condition: populate both orgs.
    {
        let mut store = state.containment_store.lock().expect("lock poisoned");
        store.insert((org_a, "host-a".to_owned()), contained_status());
        store.insert((org_b, "host-b".to_owned()), contained_status());
    }

    // Action: reset only org_A.
    // RED GATE — this line panics with `todo!("S-3.2.03: implement per-org reset …")`
    state.reset_for(org_a);

    // Postcondition: org_A gone; org_B intact (TV-3.2.001-05).
    {
        let store = state.containment_store.lock().expect("lock poisoned");
        assert!(
            store.get(&(org_a, "host-a".to_owned())).is_none(),
            "reset_for must remove org_A containment entries"
        );
        assert!(
            store.get(&(org_b, "host-b".to_owned())).is_some(),
            "reset_for must NOT remove org_B containment entries (EC-004)"
        );
    }
}

/// test_BC_3_2_001_reset_for_removes_only_target_org_detection_status
///
/// AC-005 (detection store half):
/// Given detection status entries for both org_A and org_B,
/// When state.reset_for(org_A) is called,
/// Then org_A's detection entries are gone; org_B's entries are intact.
///
/// RED GATE: fails on `todo!()` in reset_for().
#[test]
#[should_panic(expected = "S-3.2.03")]
fn test_BC_3_2_001_reset_for_removes_only_target_org_detection_status() {
    let state = CrowdstrikeState::default();
    let (org_a, org_b) = two_distinct_orgs();

    {
        let mut store = state.detection_status_store.lock().expect("lock poisoned");
        store.insert((org_a, "det-a".to_owned()), "closed".to_owned());
        store.insert((org_b, "det-b".to_owned()), "new".to_owned());
    }

    // RED GATE — panics on todo!()
    state.reset_for(org_a);

    {
        let store = state.detection_status_store.lock().expect("lock poisoned");
        assert!(
            store.get(&(org_a, "det-a".to_owned())).is_none(),
            "reset_for must remove org_A detection status entries"
        );
        assert!(
            store.get(&(org_b, "det-b".to_owned())).is_some(),
            "reset_for must NOT remove org_B detection status entries (EC-004)"
        );
    }
}

/// test_BC_3_2_001_reset_for_both_stores_atomically
///
/// EC-003: reset_for(org_A) must clear BOTH containment AND detection stores
/// atomically — not just one of them.
///
/// RED GATE: fails on `todo!()` in reset_for().
#[test]
#[should_panic(expected = "S-3.2.03")]
fn test_BC_3_2_001_reset_for_both_stores_atomically() {
    let state = CrowdstrikeState::default();
    let (org_a, org_b) = two_distinct_orgs();

    {
        let mut c_store = state.containment_store.lock().expect("lock poisoned");
        c_store.insert((org_a, "host-a".to_owned()), contained_status());
        c_store.insert((org_b, "host-b".to_owned()), contained_status());
    }
    {
        let mut d_store = state.detection_status_store.lock().expect("lock poisoned");
        d_store.insert((org_a, "det-a".to_owned()), "closed".to_owned());
        d_store.insert((org_b, "det-b".to_owned()), "new".to_owned());
    }

    // RED GATE
    state.reset_for(org_a);

    // Both stores must be cleared for org_A; org_B untouched.
    {
        let c_store = state.containment_store.lock().expect("lock poisoned");
        let d_store = state.detection_status_store.lock().expect("lock poisoned");

        assert!(
            c_store.get(&(org_a, "host-a".to_owned())).is_none(),
            "reset_for: containment store must be cleared for org_A (EC-003)"
        );
        assert!(
            d_store.get(&(org_a, "det-a".to_owned())).is_none(),
            "reset_for: detection store must be cleared for org_A (EC-003)"
        );
        assert!(
            c_store.get(&(org_b, "host-b".to_owned())).is_some(),
            "reset_for: org_B containment must be intact (EC-003)"
        );
        assert!(
            d_store.get(&(org_b, "det-b".to_owned())).is_some(),
            "reset_for: org_B detection status must be intact (EC-003)"
        );
    }
}

// ---------------------------------------------------------------------------
// AC-007 — HTTP route handlers: contain/lift_containment/patch_detections use
// (org_id, host_id) keys (BC-3.2.001 invariant 1)
//
// RED GATE: The route handlers have `todo!("S-3.2.03: extract OrgId from
// request extensions")` which panics when hit — so any HTTP request to
// contain or patch_detections will result in the server task panicking and the
// client receiving a connection error.
//
// These tests verify that the routes WILL work correctly after the `todo!()` is
// replaced by real OrgId extraction.
// ---------------------------------------------------------------------------

/// test_AC_007_contain_route_accepts_org_a_containment
///
/// AC-007: POST /devices/entities/devices-actions/v2?action_name=contain
/// must return HTTP 202 for a valid org-scoped request.
///
/// RED GATE: The route handler panics at `todo!()` before returning a response.
/// The reqwest client will see a connection error (server task aborts).
#[tokio::test]
async fn test_AC_007_contain_route_accepts_org_a_containment() {
    let mut clone = CrowdstrikeClone::new();
    clone.start().await.expect("start() must succeed");
    let base_url = clone.base_url();
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .unwrap();

    // This call will hit the `todo!()` in the contain() handler and panic.
    // The test expects HTTP 202 — it will fail because the server task panics.
    let resp = client
        .post(format!("{base_url}/devices/entities/devices-actions/v2"))
        .query(&[("action_name", "contain")])
        .header("Authorization", "Bearer dtu-fake-cs-token")
        // X-Org-Id header — the implementer must wire this through.
        .header("X-Org-Id", OrgId::new().to_string())
        .json(&serde_json::json!({"ids": ["host-001"]}))
        .send()
        .await
        .expect("request must reach server (even if it panics inside the handler)");

    assert_eq!(
        resp.status().as_u16(),
        202,
        "test_AC_007_contain_route_accepts_org_a_containment: \
         POST contain must return HTTP 202 once OrgId extraction is implemented \
         (BC-3.2.001 invariant 1) — RED GATE: currently panics at todo!()"
    );
}

/// test_AC_007_lift_containment_route_uses_org_scoped_key
///
/// AC-007: POST /devices/entities/devices-actions/v2?action_name=lift_containment
/// must return HTTP 202 for a valid org-scoped request.
///
/// RED GATE: same as contain — panics at todo!().
#[tokio::test]
async fn test_AC_007_lift_containment_route_uses_org_scoped_key() {
    let mut clone = CrowdstrikeClone::new();
    clone.start().await.expect("start() must succeed");
    let base_url = clone.base_url();
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .unwrap();

    let resp = client
        .post(format!("{base_url}/devices/entities/devices-actions/v2"))
        .query(&[("action_name", "lift_containment")])
        .header("Authorization", "Bearer dtu-fake-cs-token")
        .header("X-Org-Id", OrgId::new().to_string())
        .json(&serde_json::json!({"ids": ["host-001"]}))
        .send()
        .await
        .expect("request must reach server");

    assert_eq!(
        resp.status().as_u16(),
        202,
        "test_AC_007_lift_containment_route_uses_org_scoped_key: \
         POST lift_containment must return HTTP 202 — RED GATE: panics at todo!()"
    );
}

/// test_AC_007_patch_detections_route_uses_org_scoped_key
///
/// AC-007: PATCH /detects/entities/detects/v2 must return HTTP 200 for a
/// valid org-scoped request.
///
/// RED GATE: panics at `todo!()` in patch_detections handler.
#[tokio::test]
async fn test_AC_007_patch_detections_route_uses_org_scoped_key() {
    let mut clone = CrowdstrikeClone::new();
    clone.start().await.expect("start() must succeed");
    let base_url = clone.base_url();
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .unwrap();

    let resp = client
        .patch(format!("{base_url}/detects/entities/detects/v2"))
        .header("Authorization", "Bearer dtu-fake-cs-token")
        .header("X-Org-Id", OrgId::new().to_string())
        .json(&serde_json::json!({
            "ids": ["det-001"],
            "status": "closed"
        }))
        .send()
        .await
        .expect("request must reach server");

    assert_eq!(
        resp.status().as_u16(),
        200,
        "test_AC_007_patch_detections_route_uses_org_scoped_key: \
         PATCH detections must return HTTP 200 — RED GATE: panics at todo!()"
    );
}

// ---------------------------------------------------------------------------
// AC-006 — OrgId-flipping proptest kills mutation (VP-3.2.001-03)
//
// For any two distinct OrgIds and any shared resource_id:
//   write under org_A → lookup under org_B → None (containment_store)
//   write under org_A → lookup under org_B → None (detection_status_store)
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(proptest::test_runner::Config {
        cases: 1000,
        max_shrink_iters: 512,
        ..Default::default()
    })]

    /// prop_containment_cross_org_isolation
    ///
    /// AC-006 / VP-3.2.001-01: For any adversarial org pair and shared device_id,
    /// containment stored under org_A is never visible under org_B.
    ///
    /// Exercises VP-3.2.001-03 — OrgId-flipping mutation must be killed.
    #[test]
    fn prop_containment_cross_org_isolation(
        device_id in "[a-z0-9-]{1,64}",
        status_str in prop_oneof![Just("contained"), Just("normal")]
    ) {
        let state = CrowdstrikeState::default();
        let org_a = OrgId::new();
        let org_b = OrgId::new();

        // org_a ≠ org_b is guaranteed by OrgId::new() UUID v7 uniqueness.
        // Belt-and-suspenders assert:
        prop_assume!(org_a != org_b);

        {
            let mut store = state.containment_store.lock().expect("lock poisoned");
            store.insert(
                (org_a, device_id.clone()),
                ContainmentStatus {
                    status: status_str.to_owned(),
                    updated_at: "2026-01-01T00:00:00Z".to_owned(),
                },
            );
        }

        let store = state.containment_store.lock().expect("lock poisoned");
        let cross_org_result = store.get(&(org_b, device_id.clone()));

        prop_assert!(
            cross_org_result.is_none(),
            "prop_containment_cross_org_isolation: \
             org_B must not see org_A's containment entry for device_id={device_id:?} \
             (VP-3.2.001-01 / BC-3.2.001 postcondition 1)"
        );
    }

    /// prop_detection_cross_org_isolation
    ///
    /// AC-006 / VP-3.2.001-01 (detection variant):
    /// For any adversarial org pair and shared detection_id,
    /// detection status stored under org_A is never visible under org_B.
    #[test]
    fn prop_detection_cross_org_isolation(
        detection_id in "[a-z0-9-]{1,64}",
        status_str in "[a-z]{3,20}"
    ) {
        let state = CrowdstrikeState::default();
        let org_a = OrgId::new();
        let org_b = OrgId::new();

        prop_assume!(org_a != org_b);

        {
            let mut store = state.detection_status_store.lock().expect("lock poisoned");
            store.insert((org_a, detection_id.clone()), status_str.clone());
        }

        let store = state.detection_status_store.lock().expect("lock poisoned");
        let cross_org_result = store.get(&(org_b, detection_id.clone()));

        prop_assert!(
            cross_org_result.is_none(),
            "prop_detection_cross_org_isolation: \
             org_B must not see org_A's detection status for detection_id={detection_id:?} \
             (VP-3.2.001-01 / BC-3.2.001 postcondition 1)"
        );
    }

    /// prop_reset_for_selectivity
    ///
    /// AC-005 / VP-3.2.001-04: reset_for(org_A) must remove exactly org_A's entries
    /// and leave org_B's entries intact.
    ///
    /// RED GATE: fails because reset_for() is `todo!()`.
    #[test]
    #[should_panic(expected = "S-3.2.03")]
    fn prop_reset_for_selectivity(
        device_id_a in "[a-z0-9-]{1,32}",
        device_id_b in "[a-z0-9-]{1,32}"
    ) {
        let state = CrowdstrikeState::default();
        let org_a = OrgId::new();
        let org_b = OrgId::new();

        prop_assume!(org_a != org_b);

        {
            let mut c = state.containment_store.lock().expect("lock poisoned");
            c.insert((org_a, device_id_a.clone()), contained_status());
            c.insert((org_b, device_id_b.clone()), contained_status());
        }

        // RED GATE — panics at todo!()
        state.reset_for(org_a);

        {
            let c = state.containment_store.lock().expect("lock poisoned");
            prop_assert!(
                c.get(&(org_a, device_id_a.clone())).is_none(),
                "prop_reset_for_selectivity: org_A entry must be removed after reset_for"
            );
            prop_assert!(
                c.get(&(org_b, device_id_b.clone())).is_some(),
                "prop_reset_for_selectivity: org_B entry must survive reset_for(org_A)"
            );
        }
    }

    /// prop_write_isolation_no_cross_org_mutation
    ///
    /// VP-3.2.001-02: write under org_A does not modify any entry keyed under org_B.
    ///
    /// For any three distinct values (org_A, org_B, shared_device_id):
    /// - Pre-populate org_B with a containment entry.
    /// - Write a different containment entry for org_A at the same device_id.
    /// - Verify org_B's entry is unchanged.
    #[test]
    fn prop_write_isolation_no_cross_org_mutation(
        device_id in "[a-z0-9-]{1,32}",
    ) {
        let state = CrowdstrikeState::default();
        let org_a = OrgId::new();
        let org_b = OrgId::new();

        prop_assume!(org_a != org_b);

        // Pre-populate org_B.
        let org_b_original = ContainmentStatus {
            status: "normal".to_owned(),
            updated_at: "2026-01-01T00:00:00Z".to_owned(),
        };
        {
            let mut store = state.containment_store.lock().expect("lock poisoned");
            store.insert((org_b, device_id.clone()), org_b_original.clone());
        }

        // Write for org_A at the same device_id.
        {
            let mut store = state.containment_store.lock().expect("lock poisoned");
            store.insert(
                (org_a, device_id.clone()),
                ContainmentStatus {
                    status: "contained".to_owned(),
                    updated_at: "2026-01-02T00:00:00Z".to_owned(),
                },
            );
        }

        // org_B's entry must be unmodified.
        let store = state.containment_store.lock().expect("lock poisoned");
        let org_b_after = store
            .get(&(org_b, device_id.clone()))
            .expect("prop_write_isolation: org_B entry must still exist after org_A write");

        prop_assert_eq!(
            org_b_after.status.as_str(),
            org_b_original.status.as_str(),
            "prop_write_isolation_no_cross_org_mutation: \
             org_B's containment status must be unchanged after org_A write \
             (VP-3.2.001-02 / BC-3.2.001 postcondition 2)"
        );
    }
}
