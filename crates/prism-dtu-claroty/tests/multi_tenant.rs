#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Multi-tenant state segregation tests for `prism-dtu-claroty`.
//!
//! Exercises BC-3.2.001 (Per-Org Sensor Data Isolation via Composite HashMap Key)
//! and BC-3.2.003 (Per-Org Session Token Isolation via (OrgId, token) Composite Key).
//!
//! # Red Gate scope (S-3.2.01 Step 2)
//!
//! Unit/proptest tests on `ClarotyState` exercise the state layer, which is already
//! implemented in the stub commit.  The HTTP-level `POST /dtu/reset_for/{org_id}`
//! integration tests are RED — the route does not exist yet and return 404, which
//! triggers assertion failures and satisfies the Red Gate requirement.
//!
//! # Test naming
//!
//! All tests follow the `test_bc_3_2_NNN_<short>` pattern (BC-based traceability).

use prism_core::OrgId;
use prism_dtu_claroty::{ClarotyClone, ClarotyState};
use prism_dtu_common::BehavioralClone;
use proptest::prelude::*;
use serde_json::json;

// ---------------------------------------------------------------------------
// Deterministic test OrgIds (BC-3.2.001 invariant 3 — test-only)
// ---------------------------------------------------------------------------

/// Org A sentinel for deterministic unit tests.
#[cfg(test)]
const ORG_A: OrgId = OrgId(uuid::uuid!("00000000-0000-7000-8000-0000000000AA"));

/// Org B sentinel for deterministic unit tests.
#[cfg(test)]
const ORG_B: OrgId = OrgId(uuid::uuid!("00000000-0000-7000-8000-0000000000BB"));

/// Org C sentinel (never written to) for missing-org tests.
#[cfg(test)]
const ORG_C: OrgId = OrgId(uuid::uuid!("00000000-0000-7000-8000-0000000000CC"));

// ---------------------------------------------------------------------------
// Helper: start a clone and return (clone, base_url, admin_token)
// ---------------------------------------------------------------------------

async fn start_clone() -> (ClarotyClone, String, String) {
    let mut clone = ClarotyClone::new();
    clone.start().await.expect("ClarotyClone::start failed");
    let base_url = clone.base_url();
    let admin_token = clone.admin_token().to_string();
    (clone, base_url, admin_token)
}

// ===========================================================================
// BC-3.2.001 — State-layer unit tests (TV-3.2.001-01 through TV-3.2.001-05)
// ===========================================================================

/// BC-3.2.001 TV-3.2.001-01 — Same-org lookup returns the stored tag.
///
/// Traces to: BC-3.2.001 postcondition 1, AC-003 (same-org path).
#[test]
fn test_bc_3_2_001_same_org_lookup_returns_stored_tag() {
    let state = ClarotyState::default();
    state.add_tag(ORG_A, "dev-001", "malware");

    let tags = state.get_tags(ORG_A, "dev-001");
    assert!(
        tags.contains("malware"),
        "same-org lookup must return the stored tag; got: {tags:?}"
    );
}

/// BC-3.2.001 TV-3.2.001-02 — Cross-org lookup returns empty set (AC-001).
///
/// Traces to: BC-3.2.001 postcondition 1, AC-001.
#[test]
fn test_bc_3_2_001_cross_org_lookup_returns_empty() {
    let state = ClarotyState::default();
    state.add_tag(ORG_A, "dev-001", "malware");

    let tags = state.get_tags(ORG_B, "dev-001");
    assert!(
        tags.is_empty(),
        "cross-org lookup must return empty set; got: {tags:?}"
    );
}

/// BC-3.2.001 TV-3.2.001-03 — Independent per-org state (AC-003).
///
/// Both orgs store different tags for the same device_id. Each lookup returns
/// only its own org's content.
///
/// Traces to: BC-3.2.001 postcondition 3, AC-003.
#[test]
fn test_bc_3_2_001_independent_per_org_state() {
    let state = ClarotyState::default();
    state.add_tag(ORG_A, "dev-001", "tag-A");
    state.add_tag(ORG_B, "dev-001", "tag-B");

    let tags_a = state.get_tags(ORG_A, "dev-001");
    let tags_b = state.get_tags(ORG_B, "dev-001");

    assert!(
        tags_a.contains("tag-A") && !tags_a.contains("tag-B"),
        "org_A lookup must contain only tag-A; got: {tags_a:?}"
    );
    assert!(
        tags_b.contains("tag-B") && !tags_b.contains("tag-A"),
        "org_B lookup must contain only tag-B; got: {tags_b:?}"
    );
}

/// BC-3.2.001 TV-3.2.001-04 — Missing org returns default (AC-004).
///
/// Org C has never had any tags stored. Lookup must return empty set, not panic.
///
/// Traces to: BC-3.2.001 postcondition 4, AC-004.
#[test]
fn test_bc_3_2_001_missing_org_returns_default() {
    let state = ClarotyState::default();
    // Write to other orgs — org_C never touched.
    state.add_tag(ORG_A, "dev-001", "some-tag");
    state.add_tag(ORG_B, "dev-001", "other-tag");

    let tags = state.get_tags(ORG_C, "dev-001");
    assert!(
        tags.is_empty(),
        "lookup for org with no entries must return empty set; got: {tags:?}"
    );
}

/// BC-3.2.001 TV-3.2.001-05 — `reset_for` selectivity (AC-005, EC-003, EC-004).
///
/// After `reset_for(ORG_A)`:
/// - `get_tags(ORG_A, *)` returns empty
/// - `get_tags(ORG_B, *)` returns ORG_B's original tags unmodified
///
/// Traces to: BC-3.2.001 invariant 1, AC-005.
#[test]
fn test_bc_3_2_001_reset_for_is_selective() {
    let state = ClarotyState::default();
    state.add_tag(ORG_A, "dev-001", "tag-A");
    state.add_tag(ORG_B, "dev-001", "tag-B");

    state.reset_for(ORG_A);

    let tags_a = state.get_tags(ORG_A, "dev-001");
    let tags_b = state.get_tags(ORG_B, "dev-001");

    assert!(
        tags_a.is_empty(),
        "after reset_for(ORG_A), org_A tags must be empty; got: {tags_a:?}"
    );
    assert!(
        tags_b.contains("tag-B"),
        "after reset_for(ORG_A), org_B tags must be intact; got: {tags_b:?}"
    );
}

/// BC-3.2.001 — Write isolation (AC-002).
///
/// Writing a tag under org_A does not modify any entry under org_B for the
/// same device_id.
///
/// Traces to: BC-3.2.001 postcondition 2, AC-002.
#[test]
fn test_bc_3_2_001_write_isolation() {
    let state = ClarotyState::default();
    state.add_tag(ORG_A, "dev-001", "tag-original");
    state.add_tag(ORG_B, "dev-001", "org-b-original");

    // Write a new tag under org_A.
    state.add_tag(ORG_A, "dev-001", "new-tag");

    // Org B must be unaffected.
    let tags_b = state.get_tags(ORG_B, "dev-001");
    assert!(
        !tags_b.contains("new-tag"),
        "write under org_A must not modify org_B entries; got: {tags_b:?}"
    );
    assert!(
        tags_b.contains("org-b-original"),
        "org_B original tag must survive org_A write; got: {tags_b:?}"
    );
}

/// BC-3.2.001 — `reset_all` removes entries across all orgs.
///
/// Verifies the full-store clear does not leave partial state for any org.
/// Traces to: AC-008 (`reset_all` completeness).
#[test]
fn test_bc_3_2_001_reset_all_clears_all_orgs() {
    let state = ClarotyState::default();
    state.add_tag(ORG_A, "dev-001", "tag-A");
    state.add_tag(ORG_B, "dev-001", "tag-B");
    state.add_tag(ORG_C, "dev-002", "tag-C");

    state.reset_all();

    assert!(
        state.get_tags(ORG_A, "dev-001").is_empty(),
        "reset_all must clear ORG_A entries"
    );
    assert!(
        state.get_tags(ORG_B, "dev-001").is_empty(),
        "reset_all must clear ORG_B entries"
    );
    assert!(
        state.get_tags(ORG_C, "dev-002").is_empty(),
        "reset_all must clear ORG_C entries"
    );
}

/// BC-3.2.001 — `reset_for` with multiple device_ids per org.
///
/// Ensures ALL `(ORG_A, *)` entries are removed, not just the first encountered.
/// Traces to: BC-3.2.001 invariant 1, EC-004.
#[test]
fn test_bc_3_2_001_reset_for_clears_all_devices_for_org() {
    let state = ClarotyState::default();
    state.add_tag(ORG_A, "dev-001", "tag-A");
    state.add_tag(ORG_A, "dev-002", "tag-A2");
    state.add_tag(ORG_A, "dev-003", "tag-A3");
    state.add_tag(ORG_B, "dev-001", "tag-B");

    state.reset_for(ORG_A);

    assert!(
        state.get_tags(ORG_A, "dev-001").is_empty(),
        "reset_for must clear dev-001 for ORG_A"
    );
    assert!(
        state.get_tags(ORG_A, "dev-002").is_empty(),
        "reset_for must clear dev-002 for ORG_A"
    );
    assert!(
        state.get_tags(ORG_A, "dev-003").is_empty(),
        "reset_for must clear dev-003 for ORG_A"
    );
    assert!(
        state.get_tags(ORG_B, "dev-001").contains("tag-B"),
        "reset_for(ORG_A) must not affect ORG_B; got: {:?}",
        state.get_tags(ORG_B, "dev-001")
    );
}

// ===========================================================================
// BC-3.2.001 — HTTP integration tests: cross-org isolation via X-Org-Id header
// ===========================================================================

/// BC-3.2.001 — HTTP: tag added under org_A does not appear in org_B device list (AC-001).
///
/// Sends `X-Org-Id: ORG_A` to add a tag, then `X-Org-Id: ORG_B` to list devices.
/// The device list for org_B must have empty tags for that device.
///
/// Traces to: BC-3.2.001 postcondition 1, AC-001.
#[tokio::test]
async fn test_bc_3_2_001_http_cross_org_tag_not_visible_to_other_org() {
    let (_clone, base_url, _admin_token) = start_clone().await;
    let client = reqwest::Client::new();

    // Add tag for ORG_A.
    let add_resp = client
        .post(format!("{base_url}/api/v1/devices/asset-001/tags/"))
        .header("Authorization", "Bearer test-token")
        .header("X-Org-Id", ORG_A.to_string())
        .json(&json!({"tag_key": "malware", "tag_value": "true"}))
        .send()
        .await
        .expect("add tag request failed");
    assert_eq!(add_resp.status().as_u16(), 201, "add tag must return 201");

    // List devices under ORG_B — must not see ORG_A's tag.
    let list_resp = client
        .post(format!("{base_url}/api/v1/devices"))
        .header("Authorization", "Bearer test-token")
        .header("X-Org-Id", ORG_B.to_string())
        .json(&json!({}))
        .send()
        .await
        .expect("device list request failed");

    let body: serde_json::Value = list_resp.json().await.expect("body is JSON");
    let devices = body["devices"].as_array().expect("`devices` array");
    let device = devices
        .iter()
        .find(|d| d["asset_id"] == "asset-001")
        .expect("asset-001 must exist in device list");

    let tags = device["tags"].as_array().expect("`tags` must be array");
    assert!(
        tags.is_empty(),
        "org_B device list must not contain org_A tag `malware`; got tags: {tags:?}"
    );
}

/// BC-3.2.001 — HTTP: org_A and org_B maintain independent tag state (AC-003).
///
/// Traces to: BC-3.2.001 postcondition 3, AC-003.
#[tokio::test]
async fn test_bc_3_2_001_http_independent_per_org_tag_state() {
    let (_clone, base_url, _admin_token) = start_clone().await;
    let client = reqwest::Client::new();

    // Add tag for ORG_A.
    client
        .post(format!("{base_url}/api/v1/devices/asset-001/tags/"))
        .header("Authorization", "Bearer test-token")
        .header("X-Org-Id", ORG_A.to_string())
        .json(&json!({"tag_key": "tag-for-org-a", "tag_value": "true"}))
        .send()
        .await
        .expect("add tag A failed");

    // Add tag for ORG_B.
    client
        .post(format!("{base_url}/api/v1/devices/asset-001/tags/"))
        .header("Authorization", "Bearer test-token")
        .header("X-Org-Id", ORG_B.to_string())
        .json(&json!({"tag_key": "tag-for-org-b", "tag_value": "true"}))
        .send()
        .await
        .expect("add tag B failed");

    // List devices under ORG_A — must see only its own tag.
    let body_a: serde_json::Value = client
        .post(format!("{base_url}/api/v1/devices"))
        .header("Authorization", "Bearer test-token")
        .header("X-Org-Id", ORG_A.to_string())
        .json(&json!({}))
        .send()
        .await
        .expect("list A failed")
        .json()
        .await
        .expect("body A is JSON");

    let devices_a = body_a["devices"].as_array().expect("`devices` array A");
    let device_a = devices_a
        .iter()
        .find(|d| d["asset_id"] == "asset-001")
        .expect("asset-001 in A list");
    let tags_a = device_a["tags"].as_array().expect("tags A");

    assert!(
        tags_a.iter().any(|t| t == "tag-for-org-a"),
        "org_A device list must contain tag-for-org-a; got: {tags_a:?}"
    );
    assert!(
        !tags_a.iter().any(|t| t == "tag-for-org-b"),
        "org_A device list must NOT contain tag-for-org-b; got: {tags_a:?}"
    );

    // List devices under ORG_B — must see only its own tag.
    let body_b: serde_json::Value = client
        .post(format!("{base_url}/api/v1/devices"))
        .header("Authorization", "Bearer test-token")
        .header("X-Org-Id", ORG_B.to_string())
        .json(&json!({}))
        .send()
        .await
        .expect("list B failed")
        .json()
        .await
        .expect("body B is JSON");

    let devices_b = body_b["devices"].as_array().expect("`devices` array B");
    let device_b = devices_b
        .iter()
        .find(|d| d["asset_id"] == "asset-001")
        .expect("asset-001 in B list");
    let tags_b = device_b["tags"].as_array().expect("tags B");

    assert!(
        tags_b.iter().any(|t| t == "tag-for-org-b"),
        "org_B device list must contain tag-for-org-b; got: {tags_b:?}"
    );
    assert!(
        !tags_b.iter().any(|t| t == "tag-for-org-a"),
        "org_B device list must NOT contain tag-for-org-a; got: {tags_b:?}"
    );
}

// ===========================================================================
// BC-3.2.001 — HTTP integration tests: POST /dtu/reset_for/{org_id}
//
// RED GATE: These tests FAIL because the `/dtu/reset_for/{org_id}` route has
// not been added to `ClarotyClone::build_router()` yet.  The server returns
// 404 Method Not Allowed; the assertions below expect 200 OK.
// ===========================================================================

/// BC-3.2.001 (AC-005) — HTTP `POST /dtu/reset_for/{org_id}` returns 200.
///
/// RED GATE: Route does not exist yet → server returns 404/405 → test FAILS.
///
/// Traces to: BC-3.2.001 invariant 1, AC-005.
#[tokio::test]
async fn test_bc_3_2_001_http_reset_for_returns_200() {
    let (_clone, base_url, _admin_token) = start_clone().await;
    let client = reqwest::Client::new();

    let resp = client
        .post(format!("{base_url}/dtu/reset_for/{ORG_A}"))
        .send()
        .await
        .expect("reset_for request failed");

    assert_eq!(
        resp.status().as_u16(),
        200,
        "POST /dtu/reset_for/{{org_id}} must return 200; got {} — route not yet wired (Red Gate)",
        resp.status()
    );
}

/// BC-3.2.001 (AC-005) — HTTP `POST /dtu/reset_for/{org_id}` clears only org_A tags.
///
/// After `POST /dtu/reset_for/ORG_A`:
/// - org_A's device list returns empty tags for all devices
/// - org_B's device list still returns its original tags
///
/// RED GATE: Route does not exist yet → first request returns 404/405 → assertion fails.
///
/// Traces to: BC-3.2.001 invariant 1, AC-005, EC-003, EC-004.
#[tokio::test]
async fn test_bc_3_2_001_http_reset_for_clears_org_a_preserves_org_b() {
    let (_clone, base_url, _admin_token) = start_clone().await;
    let client = reqwest::Client::new();

    // Add tags for both orgs.
    for (org, tag) in [(ORG_A.to_string(), "tag-a"), (ORG_B.to_string(), "tag-b")] {
        client
            .post(format!("{base_url}/api/v1/devices/asset-001/tags/"))
            .header("Authorization", "Bearer test-token")
            .header("X-Org-Id", &org)
            .json(&json!({"tag_key": tag, "tag_value": "true"}))
            .send()
            .await
            .expect("add tag failed");
    }

    // Issue selective reset for ORG_A.
    let reset_resp = client
        .post(format!("{base_url}/dtu/reset_for/{ORG_A}"))
        .send()
        .await
        .expect("reset_for request failed");

    assert_eq!(
        reset_resp.status().as_u16(),
        200,
        "POST /dtu/reset_for/{{org_id}} must return 200; got {} — route not yet wired (Red Gate)",
        reset_resp.status()
    );

    // ORG_A device list must have empty tags.
    let body_a: serde_json::Value = client
        .post(format!("{base_url}/api/v1/devices"))
        .header("Authorization", "Bearer test-token")
        .header("X-Org-Id", ORG_A.to_string())
        .json(&json!({}))
        .send()
        .await
        .expect("list A failed")
        .json()
        .await
        .expect("body A JSON");
    let devices_a = body_a["devices"].as_array().expect("devices A");
    let dev_a = devices_a
        .iter()
        .find(|d| d["asset_id"] == "asset-001")
        .expect("asset-001 in A");
    let tags_a = dev_a["tags"].as_array().expect("tags A");
    assert!(
        tags_a.is_empty(),
        "after reset_for(ORG_A), org_A tags must be empty; got: {tags_a:?}"
    );

    // ORG_B device list must still have its tags.
    let body_b: serde_json::Value = client
        .post(format!("{base_url}/api/v1/devices"))
        .header("Authorization", "Bearer test-token")
        .header("X-Org-Id", ORG_B.to_string())
        .json(&json!({}))
        .send()
        .await
        .expect("list B failed")
        .json()
        .await
        .expect("body B JSON");
    let devices_b = body_b["devices"].as_array().expect("devices B");
    let dev_b = devices_b
        .iter()
        .find(|d| d["asset_id"] == "asset-001")
        .expect("asset-001 in B");
    let tags_b = dev_b["tags"].as_array().expect("tags B");
    assert!(
        tags_b.iter().any(|t| t == "tag-b"),
        "org_B tags must survive reset_for(ORG_A); got: {tags_b:?}"
    );
}

/// BC-3.2.001 (AC-005) — HTTP `POST /dtu/reset_for/{org_id}` with invalid UUID returns 400.
///
/// If the org_id path parameter is not a valid UUID, the server must return 400
/// rather than 500 or 404.
///
/// RED GATE: Route does not exist → returns 404/405 → assertion fails.
///
/// Traces to: BC-3.2.001 precondition 3 (org_id must be valid UUID).
#[tokio::test]
async fn test_bc_3_2_001_http_reset_for_invalid_org_id_returns_400() {
    let (_clone, base_url, _admin_token) = start_clone().await;
    let client = reqwest::Client::new();

    let resp = client
        .post(format!("{base_url}/dtu/reset_for/not-a-valid-uuid"))
        .send()
        .await
        .expect("reset_for request failed");

    assert_eq!(
        resp.status().as_u16(),
        400,
        "POST /dtu/reset_for/{{invalid}} must return 400; got {} — route not yet wired (Red Gate)",
        resp.status()
    );
}

// ===========================================================================
// BC-3.2.001 VP-3.2.001-01 / VP-3.2.001-02 — Proptest: adversarial org pairs
//
// These tests exercise the state layer directly (state methods are implemented).
// They are NOT expected to be Red Gate failures; they validate structural
// isolation properties that kill OrgId-flipping mutations (TD-DTU-MUTATE-COVERAGE-001).
// ===========================================================================

proptest! {
    /// VP-3.2.001-01 — Cross-org lookup always returns default (AC-007).
    ///
    /// For any two distinct org_ids and any device_id, writing under org_a and
    /// reading under org_b returns empty.  Kills the OrgId-flipping mutation
    /// (TD-DTU-MUTATE-COVERAGE-001).
    ///
    /// Traces to: BC-3.2.001 VP-3.2.001-01, AC-007.
    #[test]
    fn test_bc_3_2_001_prop_cross_org_lookup_returns_default(
        // Generate two distinct 128-bit values for use as UUID bytes.
        bytes_a in prop::array::uniform16(0u8..),
        bytes_b in prop::array::uniform16(0u8..),
        device_id in "[a-z]{1,16}",
        tag_key in "[a-z]{1,16}",
    ) {
        // Force version bits to v7 to satisfy OrgId::from_uuid_v7 precondition.
        let mut ba = bytes_a;
        ba[6] = (ba[6] & 0x0f) | 0x70;  // version = 7
        ba[8] = (ba[8] & 0x3f) | 0x80;  // variant = RFC 4122

        let mut bb = bytes_b;
        bb[6] = (bb[6] & 0x0f) | 0x70;
        bb[8] = (bb[8] & 0x3f) | 0x80;

        let uuid_a = uuid::Uuid::from_bytes(ba);
        let uuid_b = uuid::Uuid::from_bytes(bb);

        // prop_assume guards against the negligible collision case.
        prop_assume!(uuid_a != uuid_b);

        let org_a = OrgId::from_uuid(uuid_a);
        let org_b = OrgId::from_uuid(uuid_b);

        let state = ClarotyState::default();
        state.add_tag(org_a, &device_id, &tag_key);

        let result = state.get_tags(org_b, &device_id);
        prop_assert!(
            result.is_empty(),
            "cross-org lookup must return empty; tag_key={tag_key:?} org_a={org_a} org_b={org_b} result={result:?}"
        );
    }

    /// VP-3.2.001-02 — Write under org_a does not modify any entry under org_b.
    ///
    /// Verifies write isolation: a write to org_a's store cannot change org_b's
    /// pre-existing tags (adversarial mutation check).
    ///
    /// Traces to: BC-3.2.001 VP-3.2.001-02, AC-002.
    #[test]
    fn test_bc_3_2_001_prop_write_isolation(
        bytes_a in prop::array::uniform16(0u8..),
        bytes_b in prop::array::uniform16(0u8..),
        device_id in "[a-z]{1,16}",
        tag_a in "[a-z]{1,12}",
        tag_b in "[a-z]{1,12}",
        new_tag in "[a-z]{1,12}",
    ) {
        let mut ba = bytes_a;
        ba[6] = (ba[6] & 0x0f) | 0x70;
        ba[8] = (ba[8] & 0x3f) | 0x80;

        let mut bb = bytes_b;
        bb[6] = (bb[6] & 0x0f) | 0x70;
        bb[8] = (bb[8] & 0x3f) | 0x80;

        let uuid_a = uuid::Uuid::from_bytes(ba);
        let uuid_b = uuid::Uuid::from_bytes(bb);
        prop_assume!(uuid_a != uuid_b);

        let org_a = OrgId::from_uuid(uuid_a);
        let org_b = OrgId::from_uuid(uuid_b);

        let state = ClarotyState::default();
        // Seed org_b with a known tag.
        state.add_tag(org_b, &device_id, &tag_b);
        let before = state.get_tags(org_b, &device_id).clone();

        // Write under org_a (including its own tag and the new tag).
        state.add_tag(org_a, &device_id, &tag_a);
        state.add_tag(org_a, &device_id, &new_tag);

        // org_b must be unchanged.
        let after = state.get_tags(org_b, &device_id);
        prop_assert_eq!(
            before,
            after,
            "write under org_a must not modify org_b's tags"
        );
    }

    /// VP-3.2.001-04 — `reset_for(org_a)` removes exactly org_a's entries.
    ///
    /// After reset_for, org_a is empty; org_b is unaffected.
    ///
    /// Traces to: BC-3.2.001 VP-3.2.001-04, AC-005.
    #[test]
    fn test_bc_3_2_001_prop_reset_for_selectivity(
        bytes_a in prop::array::uniform16(0u8..),
        bytes_b in prop::array::uniform16(0u8..),
        device_id in "[a-z]{1,16}",
        tag_a in "[a-z]{1,12}",
        tag_b in "[a-z]{1,12}",
    ) {
        let mut ba = bytes_a;
        ba[6] = (ba[6] & 0x0f) | 0x70;
        ba[8] = (ba[8] & 0x3f) | 0x80;

        let mut bb = bytes_b;
        bb[6] = (bb[6] & 0x0f) | 0x70;
        bb[8] = (bb[8] & 0x3f) | 0x80;

        let uuid_a = uuid::Uuid::from_bytes(ba);
        let uuid_b = uuid::Uuid::from_bytes(bb);
        prop_assume!(uuid_a != uuid_b);

        let org_a = OrgId::from_uuid(uuid_a);
        let org_b = OrgId::from_uuid(uuid_b);

        let state = ClarotyState::default();
        state.add_tag(org_a, &device_id, &tag_a);
        state.add_tag(org_b, &device_id, &tag_b);

        state.reset_for(org_a);

        let tags_a = state.get_tags(org_a, &device_id);
        let tags_b = state.get_tags(org_b, &device_id);

        prop_assert!(
            tags_a.is_empty(),
            "after reset_for(org_a), org_a tags must be empty; got: {tags_a:?}"
        );
        prop_assert!(
            tags_b.contains(&tag_b),
            "after reset_for(org_a), org_b tags must be intact; got: {tags_b:?}"
        );
    }
}

// ===========================================================================
// BC-3.2.001 invariant 3 / AC-006 — DEFAULT_ORG_ID compile-time gate
//
// DEFAULT_ORG_ID is #[cfg(test)] only in the production crate's src/state.rs.
// In integration tests (a separate crate context) it is not importable —
// that is the enforcement mechanism.  The in-crate unit test below lives in
// src/state.rs and verifies the constant is reachable within the crate's own
// test harness; it is the canonical AC-006 test.
//
// No integration test is written here because attempting to import
// `prism_dtu_claroty::state::DEFAULT_ORG_ID` in an integration test would
// be a compile error, which is the very property being tested (invariant 3).
// ===========================================================================
