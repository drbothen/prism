//! BC-3.2.001 — Per-Org Sensor Data Isolation via Composite HashMap Key
//!
//! Red Gate tests for S-3.2.02: all tests exercise `ArmisState` tag-store
//! operations through the composite `(OrgId, device_id)` key.  Every test
//! MUST FAIL until the implementer fills in `add_tag`, `remove_tag`,
//! `tags_for`, and `reset_for` in `src/state.rs`.
//!
//! ## Traceability
//!
//! | Test | AC / TV | VP |
//! |------|---------|----|
//! | `test_BC_3_2_001_same_org_lookup_returns_stored_tags` | AC-001, TV-3.2.001-01 | — |
//! | `test_BC_3_2_001_cross_org_lookup_returns_empty` | AC-001, TV-3.2.001-02 | VP-3.2.001-01 |
//! | `test_BC_3_2_001_write_isolation_org_a_does_not_affect_org_b` | AC-002, TV-3.2.001-03 | VP-3.2.001-02 |
//! | `test_BC_3_2_001_independent_state_per_org_same_device_id` | AC-003, TV-3.2.001-03 | — |
//! | `test_BC_3_2_001_lookup_unknown_org_returns_empty` | AC-001 EC-002, TV-3.2.001-04 | — |
//! | `test_BC_3_2_001_reset_for_is_selective` | AC-005, TV-3.2.001-05 | VP-3.2.001-04 |
//! | `test_BC_3_2_001_fixture_registries_bare_string_keyed` | AC-004 | — |
//! | `prop_cross_org_tag_isolation` | AC-006, VP-3.2.001-01 | VP-3.2.001-03 |
//! | `prop_write_does_not_affect_other_org` | AC-002, AC-006 | VP-3.2.001-02, VP-3.2.001-03 |
//! | `prop_reset_for_selectivity` | AC-005 | VP-3.2.001-04 |

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![cfg(feature = "dtu")]

use prism_core::OrgId;
use prism_dtu_armis::state::ArmisState;
use proptest::prelude::*;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Test fixture helpers
// ---------------------------------------------------------------------------

/// OrgId for the primary test organisation (mirrors `state::DEFAULT_ORG_ID`).
///
/// Redeclared here because `DEFAULT_ORG_ID` in state.rs is `#[cfg(test)]`
/// and is therefore not visible to integration test binaries (which are
/// compiled as separate crates).
///
/// BC-3.2.001 invariant 3: this constant must never appear in production code.
const ORG_ID_A: OrgId = OrgId(uuid::uuid!("00000000-0000-7000-8000-000000000001"));

/// Second org distinct from `ORG_ID_A` — used as `org_id_B` throughout.
const ORG_ID_B: OrgId = OrgId(uuid::uuid!("00000000-0000-7000-8000-000000000002"));

/// Third org that has never written any state — exercises EC-002.
const ORG_ID_C: OrgId = OrgId(uuid::uuid!("00000000-0000-7000-8000-000000000003"));

/// Build an `ArmisState` with empty fixture lists suitable for unit tests that
/// do not exercise device-list or alert/activity routes.
fn empty_state() -> ArmisState {
    ArmisState::new(vec![], vec![], vec![])
}

// ---------------------------------------------------------------------------
// AC-001 / TV-3.2.001-01 — same-org retrieval
// ---------------------------------------------------------------------------

/// BC-3.2.001 postcondition 1 (same-org path):
/// After writing `{"malware"}` for `(org_id_A, "dev-1")`, `tags_for(org_id_A, "dev-1")`
/// returns a list that contains `"malware"`.
///
/// This test will FAIL until `add_tag` and `tags_for` are implemented.
#[test]
fn test_BC_3_2_001_same_org_lookup_returns_stored_tags() {
    let state = empty_state();
    let org_a = ORG_ID_A;

    state.add_tag(org_a, "dev-1", "malware");

    let tags = state.tags_for(org_a, "dev-1", &[]);
    assert!(
        tags.contains(&"malware".to_string()),
        "BC-3.2.001 TV-3.2.001-01: tags_for(org_a, 'dev-1') must contain 'malware'; got {tags:?}"
    );
}

// ---------------------------------------------------------------------------
// AC-001 / TV-3.2.001-02 — cross-org lookup returns empty
// ---------------------------------------------------------------------------

/// BC-3.2.001 postcondition 1 (cross-org path):
/// After writing `{"malware"}` for `(org_id_A, "dev-1")`, `tags_for(org_id_B, "dev-1")`
/// must return an empty list.
///
/// This test will FAIL until `add_tag` and `tags_for` are implemented.
///
/// Exercises VP-3.2.001-01.
#[test]
fn test_BC_3_2_001_cross_org_lookup_returns_empty() {
    let state = empty_state();
    let org_a = ORG_ID_A;
    let org_b = ORG_ID_B;

    state.add_tag(org_a, "dev-1", "malware");

    let tags = state.tags_for(org_b, "dev-1", &[]);
    assert!(
        tags.is_empty(),
        "BC-3.2.001 TV-3.2.001-02: tags_for(org_b, 'dev-1') must be empty when tag was stored under org_a; got {tags:?}"
    );
}

// ---------------------------------------------------------------------------
// AC-002 / TV-3.2.001-03 — write isolation
// ---------------------------------------------------------------------------

/// BC-3.2.001 postcondition 2:
/// Writing `"compromised"` under `org_id_A` must NOT appear in `org_id_B`'s view.
///
/// This test will FAIL until `add_tag` and `tags_for` are implemented.
///
/// Exercises VP-3.2.001-02.
#[test]
fn test_BC_3_2_001_write_isolation_org_a_does_not_affect_org_b() {
    let state = empty_state();
    let org_a = ORG_ID_A;
    let org_b = ORG_ID_B;

    // org_b starts with its own tag.
    state.add_tag(org_b, "dev-42", "benign");
    // org_a writes "compromised".
    state.add_tag(org_a, "dev-42", "compromised");

    let org_b_tags = state.tags_for(org_b, "dev-42", &[]);
    assert!(
        !org_b_tags.contains(&"compromised".to_string()),
        "BC-3.2.001 AC-002: org_b must NOT see 'compromised' written by org_a; got {org_b_tags:?}"
    );
    assert!(
        org_b_tags.contains(&"benign".to_string()),
        "BC-3.2.001 AC-002: org_b must still have its own 'benign' tag; got {org_b_tags:?}"
    );
}

// ---------------------------------------------------------------------------
// AC-003 / TV-3.2.001-03 — independent state per org
// ---------------------------------------------------------------------------

/// BC-3.2.001 postcondition 3:
/// Storing `{"tag-A"}` for org_A and `{"tag-B"}` for org_B under the same device ID
/// must produce independent, correctly isolated results for both orgs.
///
/// This test will FAIL until `add_tag` and `tags_for` are implemented.
#[test]
fn test_BC_3_2_001_independent_state_per_org_same_device_id() {
    let state = empty_state();
    let org_a = ORG_ID_A;
    let org_b = ORG_ID_B;

    state.add_tag(org_a, "dev-1", "tag-A");
    state.add_tag(org_b, "dev-1", "tag-B");

    let tags_a = state.tags_for(org_a, "dev-1", &[]);
    let tags_b = state.tags_for(org_b, "dev-1", &[]);

    assert!(
        tags_a.contains(&"tag-A".to_string()),
        "BC-3.2.001 TV-3.2.001-03: org_a must have 'tag-A'; got {tags_a:?}"
    );
    assert!(
        !tags_a.contains(&"tag-B".to_string()),
        "BC-3.2.001 TV-3.2.001-03: org_a must NOT have 'tag-B'; got {tags_a:?}"
    );
    assert!(
        tags_b.contains(&"tag-B".to_string()),
        "BC-3.2.001 TV-3.2.001-03: org_b must have 'tag-B'; got {tags_b:?}"
    );
    assert!(
        !tags_b.contains(&"tag-A".to_string()),
        "BC-3.2.001 TV-3.2.001-03: org_b must NOT have 'tag-A'; got {tags_b:?}"
    );
}

// ---------------------------------------------------------------------------
// EC-002 / TV-3.2.001-04 — lookup for org with no state
// ---------------------------------------------------------------------------

/// BC-3.2.001 postcondition 4 / EC-002:
/// `tags_for` for an OrgId that has never written any state must return an
/// empty list — not a panic or error.
///
/// This test will FAIL until `tags_for` is implemented.
#[test]
fn test_BC_3_2_001_lookup_unknown_org_returns_empty() {
    let state = empty_state();
    // org_c has never written anything.
    let tags = state.tags_for(ORG_ID_C, "dev-1", &[]);
    assert!(
        tags.is_empty(),
        "BC-3.2.001 TV-3.2.001-04 EC-002: tags_for an org with no history must be empty; got {tags:?}"
    );
}

// ---------------------------------------------------------------------------
// AC-005 / TV-3.2.001-05 — reset_for is selective
// ---------------------------------------------------------------------------

/// BC-3.2.001 edge case EC-004 / TV-3.2.001-05:
/// `reset_for(org_id_A)` must remove all `(org_id_A, *)` entries and leave
/// org_id_B entries untouched.
///
/// This test will FAIL until `reset_for` is implemented.
///
/// Exercises VP-3.2.001-04.
#[test]
fn test_BC_3_2_001_reset_for_is_selective() {
    let state = empty_state();
    let org_a = ORG_ID_A;
    let org_b = ORG_ID_B;

    state.add_tag(org_a, "dev-1", "tag-A");
    state.add_tag(org_a, "dev-2", "tag-A2");
    state.add_tag(org_b, "dev-1", "tag-B");

    // Reset org_a only.
    state.reset_for(org_a);

    // org_a entries must be gone.
    let tags_a_dev1 = state.tags_for(org_a, "dev-1", &[]);
    let tags_a_dev2 = state.tags_for(org_a, "dev-2", &[]);
    assert!(
        tags_a_dev1.is_empty(),
        "BC-3.2.001 TV-3.2.001-05: after reset_for(org_a), org_a dev-1 tags must be empty; got {tags_a_dev1:?}"
    );
    assert!(
        tags_a_dev2.is_empty(),
        "BC-3.2.001 TV-3.2.001-05: after reset_for(org_a), org_a dev-2 tags must be empty; got {tags_a_dev2:?}"
    );

    // org_b entries must be intact.
    let tags_b = state.tags_for(org_b, "dev-1", &[]);
    assert!(
        tags_b.contains(&"tag-B".to_string()),
        "BC-3.2.001 TV-3.2.001-05: reset_for(org_a) must NOT remove org_b tags; got {tags_b:?}"
    );
}

// ---------------------------------------------------------------------------
// AC-004 — fixture registries remain bare-String keyed
// ---------------------------------------------------------------------------

/// BC-3.2.001 invariant 1:
/// `device_registry` and `devices_ordered` must use `HashMap<String, _>` keys
/// (not `(OrgId, String)`) — they are read-only fixture data per ADR-008 §2.1
/// Step 6b. This test verifies the _type_ by constructing ArmisState and
/// accessing the fields — if the migration wrongly re-keys the read-only
/// registries this will fail to compile (and therefore fail the Red Gate).
///
/// This test PASSES at compile time if the type is correct.  It will FAIL
/// at runtime only if the state.rs migration incorrectly re-keys these fields.
#[test]
fn test_BC_3_2_001_fixture_registries_bare_string_keyed() {
    use prism_dtu_armis::types::DeviceRecord;

    // Construct a state with a minimal DeviceRecord so device_registry is non-empty.
    let device = DeviceRecord {
        device_id: "d-fixture-001".to_string(),
        name: "test-device".to_string(),
        ip_address: None,
        mac_address: None,
        device_type: None,
        manufacturer: None,
        os_name: None,
        os_version: None,
        risk_score: None,
        risk_factors: vec![],
        last_seen: None,
        first_seen: None,
        network_id: None,
        site: None,
        tags: vec![],
    };

    let state = ArmisState::new(vec![device], vec![], vec![]);

    // device_registry must be keyed by String, not (OrgId, String).
    // This lookup compiles only if device_registry: HashMap<String, _>.
    let lookup: Option<&prism_dtu_armis::types::DeviceRecord> =
        state.device_registry.get("d-fixture-001");
    assert!(
        lookup.is_some(),
        "BC-3.2.001 AC-004: device_registry must be bare-String keyed and contain 'd-fixture-001'"
    );

    // devices_ordered must remain a Vec<DeviceRecord> (no OrgId needed).
    assert_eq!(
        state.devices_ordered.len(),
        1,
        "BC-3.2.001 AC-004: devices_ordered must reflect loaded fixture count"
    );
}

// ---------------------------------------------------------------------------
// remove_tag — composite key wiring
// ---------------------------------------------------------------------------

/// BC-3.2.001 postcondition 2 (remove path):
/// `remove_tag(org_a, device_id, tag)` must only remove the tag from org_a's
/// bucket; org_b's identical tag must be unaffected.
///
/// This test will FAIL until `remove_tag` and `tags_for` are implemented.
#[test]
fn test_BC_3_2_001_remove_tag_is_org_scoped() {
    let state = empty_state();
    let org_a = ORG_ID_A;
    let org_b = ORG_ID_B;

    state.add_tag(org_a, "dev-5", "shared-label");
    state.add_tag(org_b, "dev-5", "shared-label");

    // Remove from org_a only.
    let removed = state.remove_tag(org_a, "dev-5", "shared-label");
    assert!(
        removed,
        "BC-3.2.001: remove_tag(org_a, 'dev-5', 'shared-label') must return true"
    );

    // org_a bucket must be empty.
    let tags_a = state.tags_for(org_a, "dev-5", &[]);
    assert!(
        !tags_a.contains(&"shared-label".to_string()),
        "BC-3.2.001: org_a must not have 'shared-label' after remove; got {tags_a:?}"
    );

    // org_b bucket must be intact.
    let tags_b = state.tags_for(org_b, "dev-5", &[]);
    assert!(
        tags_b.contains(&"shared-label".to_string()),
        "BC-3.2.001: org_b must still have 'shared-label' after org_a removes it; got {tags_b:?}"
    );
}

// ---------------------------------------------------------------------------
// VP-3.2.001-01 / AC-006 — proptest: cross-org lookup always returns empty
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(proptest::test_runner::Config {
        cases: 1000,
        max_shrink_iters: 512,
        ..Default::default()
    })]

    /// VP-3.2.001-01 / VP-3.2.001-03 (OrgId-flipping mutation killed):
    ///
    /// For any pair of distinct UUIDs (org_a, org_b) and any device_id string,
    /// writing a tag under org_a and reading under org_b must always return empty.
    ///
    /// Kills the OrgId-flipping mutation (TD-DTU-MUTATE-COVERAGE-001): an
    /// implementation that ignores the org component of the composite key will
    /// fail this property.
    #[test]
    fn prop_cross_org_tag_isolation(
        org_a_bytes in prop::array::uniform16(0u8..),
        org_b_bytes in prop::array::uniform16(0u8..),
        device_id in "[a-zA-Z0-9_-]{1,64}",
        tag in "[a-zA-Z0-9_-]{1,32}",
    ) {
        // Construct two OrgIds from raw bytes; they may or may not collide —
        // when they do collide, the lookup is legitimately non-empty, so we
        // filter that case out.
        let uuid_a = Uuid::from_bytes(org_a_bytes);
        let uuid_b = Uuid::from_bytes(org_b_bytes);
        prop_assume!(uuid_a != uuid_b);

        let org_a = OrgId::from_uuid(uuid_a);
        let org_b = OrgId::from_uuid(uuid_b);

        let state = empty_state();
        state.add_tag(org_a, &device_id, &tag);

        let tags_b = state.tags_for(org_b, &device_id, &[]);
        prop_assert!(
            tags_b.is_empty(),
            "VP-3.2.001-01: cross-org lookup must be empty; org_a={org_a}, org_b={org_b}, device_id={device_id:?}, got {tags_b:?}"
        );
    }

    /// VP-3.2.001-02 / AC-002:
    ///
    /// For any org_a ≠ org_b, any sequence of add_tag calls under org_a must
    /// not change the tag set for org_b.
    ///
    /// Pre-populates org_b with a known tag, then writes under org_a, and
    /// verifies org_b's content is unchanged.
    #[test]
    fn prop_write_does_not_affect_other_org(
        org_a_bytes in prop::array::uniform16(0u8..),
        org_b_bytes in prop::array::uniform16(0u8..),
        device_id in "[a-zA-Z0-9_-]{1,64}",
        tag_a in "[a-zA-Z0-9_-]{1,32}",
        tag_b in "[a-zA-Z0-9_-]{1,32}",
    ) {
        let uuid_a = Uuid::from_bytes(org_a_bytes);
        let uuid_b = Uuid::from_bytes(org_b_bytes);
        prop_assume!(uuid_a != uuid_b);

        let org_a = OrgId::from_uuid(uuid_a);
        let org_b = OrgId::from_uuid(uuid_b);

        let state = empty_state();
        // Pre-populate org_b.
        state.add_tag(org_b, &device_id, &tag_b);

        // Write under org_a.
        state.add_tag(org_a, &device_id, &tag_a);

        // org_b must still have exactly tag_b (and not tag_a unless they happen
        // to be the same string value).
        let tags_b = state.tags_for(org_b, &device_id, &[]);
        prop_assert!(
            tags_b.contains(&tag_b),
            "VP-3.2.001-02: org_b must retain its own tag after org_a writes; got {tags_b:?}"
        );
        if tag_a != tag_b {
            prop_assert!(
                !tags_b.contains(&tag_a),
                "VP-3.2.001-02: org_b must NOT contain org_a's tag; got {tags_b:?}"
            );
        }
    }

    /// VP-3.2.001-04 / AC-005:
    ///
    /// `reset_for(org_a)` removes all org_a entries and leaves all org_b entries intact,
    /// for any pair of distinct OrgIds and any device IDs.
    #[test]
    fn prop_reset_for_selectivity(
        org_a_bytes in prop::array::uniform16(0u8..),
        org_b_bytes in prop::array::uniform16(0u8..),
        device_id in "[a-zA-Z0-9_-]{1,64}",
        tag_a in "[a-zA-Z0-9_-]{1,32}",
        tag_b in "[a-zA-Z0-9_-]{1,32}",
    ) {
        let uuid_a = Uuid::from_bytes(org_a_bytes);
        let uuid_b = Uuid::from_bytes(org_b_bytes);
        prop_assume!(uuid_a != uuid_b);

        let org_a = OrgId::from_uuid(uuid_a);
        let org_b = OrgId::from_uuid(uuid_b);

        let state = empty_state();
        state.add_tag(org_a, &device_id, &tag_a);
        state.add_tag(org_b, &device_id, &tag_b);

        state.reset_for(org_a);

        // org_a must be empty.
        let tags_a = state.tags_for(org_a, &device_id, &[]);
        prop_assert!(
            tags_a.is_empty(),
            "VP-3.2.001-04: after reset_for(org_a), org_a tags must be empty; got {tags_a:?}"
        );

        // org_b must retain its tag.
        let tags_b = state.tags_for(org_b, &device_id, &[]);
        prop_assert!(
            tags_b.contains(&tag_b),
            "VP-3.2.001-04: reset_for(org_a) must not affect org_b; got {tags_b:?}"
        );
    }
}
