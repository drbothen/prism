//! Red Gate tests for BC-2.16.021 — Claroty xDome Organization Firewall Domain.
//!
//! Covers claroty_organization_firewall_groups and claroty_organization_firewall_policies
//! TOML tables. All non-#[ignore] tests MUST fail before Task 9 adds the [[tables]] blocks
//! to crates/prism-sensors/specs/claroty.sensor.toml — failure mode is `.expect()` panic
//! when the table is absent from the parsed SensorSpec.
//!
//! Key asymmetry under test (EC-016-021-006):
//!   path_template = "/api/v1/organization_fw_groups/"  (abbreviated _fw_groups in URL)
//!   response_path = "$.organization_firewall_groups"   (full spelling in envelope key)
//!
//! Red Gate test list (RG-015..RG-026):
//!   RG-015  test_BC_2_16_021_claroty_organization_firewall_groups_toml_block_parses
//!   RG-016  test_BC_2_16_021_claroty_organization_firewall_groups_tier1_columns_four_with_ocsf_field
//!   RG-017  test_BC_2_16_021_claroty_organization_firewall_groups_live_fw_asymmetry_nonempty_result (#[ignore])
//!   RG-018  test_BC_2_16_021_claroty_organization_firewall_groups_live_wire_shape_class_uid_and_tier1 (#[ignore])
//!   RG-019  test_BC_2_16_021_claroty_organization_firewall_groups_required_fwgroupname_absent_produces_null_row
//!   RG-020  test_BC_2_16_021_claroty_organization_firewall_groups_tier2_column_raises_e_query_038
//!   RG-021  test_BC_2_16_021_claroty_organization_firewall_policies_toml_block_parses
//!   RG-022  test_BC_2_16_021_claroty_organization_firewall_policies_tier1_columns_four_with_ocsf_field
//!   RG-023  test_BC_2_16_021_claroty_organization_firewall_policies_live_wire_shape_class_uid_and_tier1 (#[ignore])
//!   RG-024  test_BC_2_16_021_claroty_organization_firewall_policies_applied_group_pairs_raises_e_query_038
//!   RG-025a test_BC_2_16_021_claroty_organization_firewall_policies_required_policy_name_absent_produces_null_row
//!   RG-025b test_BC_2_16_021_claroty_organization_firewall_policies_nullable_count_uses_empty_page_halt
//!   RG-026  test_BC_2_16_021_claroty_organization_firewall_policies_json_columns_not_stringified
//!
//! BC-5.38.001 density check: 12 test functions (RG-025 split into 2 sub-tests) gate 12 ACs
//! (RG-015..026, with RG-025 counted as 1 RGT per 1 AC). Ratio = 1.0 ≥ 0.5. PASS.
//!
//! Story: S-CLAROTY-ORGPOLICY-001 | BC: BC-2.16.021
#![allow(clippy::expect_used, clippy::unwrap_used)]

use prism_core::column::ColumnOptions;
use prism_spec_engine::column_mapping::{ocsf_projected_column_names, ColumnMapper};
use prism_spec_engine::spec_parser::{PaginationConfig, SpecLoader};
use serde_json::json;

// ---------------------------------------------------------------------------
// Shared helper
// ---------------------------------------------------------------------------

fn load_claroty_spec() -> prism_spec_engine::spec_parser::SensorSpec {
    let toml_path = concat!(env!("CARGO_MANIFEST_DIR"), "/specs/claroty.sensor.toml");
    let content = std::fs::read_to_string(toml_path)
        .unwrap_or_else(|e| panic!("Failed to read claroty.sensor.toml: {e}"));
    SpecLoader::parse(&content)
        .unwrap_or_else(|e| panic!("SpecLoader::parse failed for claroty.sensor.toml: {e:?}"))
}

// ===========================================================================
// RG-015 — claroty_organization_firewall_groups TOML block parses
//          (URL↔envelope-key asymmetry assertion is load-bearing)
// ===========================================================================

/// RG-015 (RED): claroty.sensor.toml must declare a [[tables]] block with
/// table_name = "claroty_organization_firewall_groups", ocsf_class = "entity_management",
/// step "fetch_organization_firewall_groups", method = "POST",
/// path_template = "/api/v1/organization_fw_groups/"  (abbreviated _fw_groups in URL),
/// response_path = "$.organization_firewall_groups"   (FULL spelling in envelope key — NOT $.organization_fw_groups),
/// offset_limit pagination page_size = 1000, and exactly 11 ColumnSpec entries.
///
/// CRITICAL: Both the abbreviated path_template AND the full-spelling response_path
/// must be present in the same TOML block. This is the documented URL↔envelope-key
/// asymmetry (EC-016-021-006). Using $.organization_fw_groups in response_path
/// causes silent empty results at runtime.
///
/// FAILS before implementation (TOML blocks absent → .expect() panics).
///
/// Traces to: BC-2.16.021 §Postconditions §1 — TOML Table Contract (fw_groups);
///            EC-016-021-005/006. Story: S-CLAROTY-ORGPOLICY-001 AC-015
#[test]
fn test_BC_2_16_021_claroty_organization_firewall_groups_toml_block_parses() {
    let spec = load_claroty_spec();

    let fw_groups = spec
        .tables
        .iter()
        .find(|t| t.table_name == "claroty_organization_firewall_groups")
        .expect(
            "claroty.sensor.toml must contain a [[tables]] block with \
             table_name = \"claroty_organization_firewall_groups\" — add per S-CLAROTY-ORGPOLICY-001 AC-015",
        );

    assert_eq!(
        fw_groups.columns.len(),
        11,
        "claroty_organization_firewall_groups must declare exactly 11 columns (BC-2.16.021 §PC1); \
         got {}",
        fw_groups.columns.len()
    );

    assert_eq!(fw_groups.steps.len(), 1, "must have exactly 1 fetch step");
    let step = &fw_groups.steps[0];

    assert_eq!(
        step.name, "fetch_organization_firewall_groups",
        "step name must be 'fetch_organization_firewall_groups'; got '{}'",
        step.name
    );
    assert_eq!(step.method, "POST", "step must use POST");

    // CRITICAL asymmetry: abbreviated URL path, full-spelling envelope key
    assert_eq!(
        step.path_template, "/api/v1/organization_fw_groups/",
        "path_template must be '/api/v1/organization_fw_groups/' (abbreviated _fw_groups in URL); \
         got '{}' (EC-016-021-006)",
        step.path_template
    );
    assert_eq!(
        step.response_path, "$.organization_firewall_groups",
        "response_path must be '$.organization_firewall_groups' (FULL spelling — NOT \
         $.organization_fw_groups); using the abbreviated form causes silent empty results; \
         got '{}' (EC-016-021-006)",
        step.response_path
    );

    // Both strings must be present and different — confirming the asymmetry is explicit
    assert_ne!(
        step.path_template, step.response_path,
        "path_template and response_path must differ \
         (documented URL↔envelope-key asymmetry — EC-016-021-006)"
    );
    assert!(
        step.path_template.contains("_fw_groups"),
        "path_template must contain abbreviated '_fw_groups'; got '{}'",
        step.path_template
    );
    assert!(
        step.response_path.contains("organization_firewall_groups"),
        "response_path must contain full 'organization_firewall_groups' (not fw_groups); \
         got '{}'",
        step.response_path
    );

    match &step.pagination {
        Some(PaginationConfig::OffsetLimit { page_size }) => {
            assert_eq!(*page_size, 1000, "page_size must be 1000; got {page_size}");
        }
        other => panic!("must use OffsetLimit pagination; got: {other:?}"),
    }

    assert_eq!(
        fw_groups.ocsf_class, "entity_management",
        "ocsf_class must be 'entity_management' (class_uid 3004); got '{}'",
        fw_groups.ocsf_class
    );
}

// ===========================================================================
// RG-016 — fw_groups Tier-1 column inspection
// ===========================================================================

/// RG-016 (RED): claroty_organization_firewall_groups must declare exactly 4 Tier-1
/// columns (ocsf_field set) and exactly 7 Tier-2 columns (ocsf_field None):
///   firewall_group_name → "name" (REQUIRED), firewall_group_description → "comment",
///   enabled → "status_code", updated_by → "actor.user.name".
///
/// FAILS before implementation.
///
/// Traces to: BC-2.16.021 §PC3 — Tier-1/Tier-2 classification (fw_groups).
/// Story: S-CLAROTY-ORGPOLICY-001 AC-016
#[test]
fn test_BC_2_16_021_claroty_organization_firewall_groups_tier1_columns_four_with_ocsf_field() {
    let spec = load_claroty_spec();

    let fw_groups = spec
        .tables
        .iter()
        .find(|t| t.table_name == "claroty_organization_firewall_groups")
        .expect(
            "claroty.sensor.toml must contain 'claroty_organization_firewall_groups' — add per AC-016",
        );

    let tier1_count = fw_groups
        .columns
        .iter()
        .filter(|c| c.ocsf_field.is_some())
        .count();
    let tier2_count = fw_groups
        .columns
        .iter()
        .filter(|c| c.ocsf_field.is_none())
        .count();

    assert_eq!(
        tier1_count, 4,
        "fw_groups must have exactly 4 Tier-1 columns; got {tier1_count}"
    );
    assert_eq!(
        tier2_count, 7,
        "fw_groups must have exactly 7 Tier-2 columns; got {tier2_count}"
    );

    // firewall_group_name → "name", REQUIRED
    let fw_name_col = fw_groups
        .columns
        .iter()
        .find(|c| c.name == "firewall_group_name")
        .expect("column 'firewall_group_name' must exist");
    assert_eq!(
        fw_name_col.ocsf_field.as_deref(),
        Some("name"),
        "firewall_group_name must map ocsf_field = \"name\""
    );
    assert!(
        fw_name_col.options.contains(&ColumnOptions::Required),
        "firewall_group_name must carry REQUIRED option (PK discipline)"
    );

    // firewall_group_description → "comment"
    let fw_desc_col = fw_groups
        .columns
        .iter()
        .find(|c| c.name == "firewall_group_description")
        .expect("column 'firewall_group_description' must exist");
    assert_eq!(
        fw_desc_col.ocsf_field.as_deref(),
        Some("comment"),
        "firewall_group_description must map ocsf_field = \"comment\""
    );

    // enabled → "status_code"
    let enabled_col = fw_groups
        .columns
        .iter()
        .find(|c| c.name == "enabled")
        .expect("column 'enabled' must exist");
    assert_eq!(
        enabled_col.ocsf_field.as_deref(),
        Some("status_code"),
        "enabled must map ocsf_field = \"status_code\""
    );

    // updated_by → "actor.user.name"
    let updated_by_col = fw_groups
        .columns
        .iter()
        .find(|c| c.name == "updated_by")
        .expect("column 'updated_by' must exist");
    assert_eq!(
        updated_by_col.ocsf_field.as_deref(),
        Some("actor.user.name"),
        "updated_by must map ocsf_field = \"actor.user.name\""
    );
}

// ===========================================================================
// RG-017 — Live fw URL↔envelope asymmetry confirmation (#[ignore])
// ===========================================================================

/// RG-017 (LIVE — #[ignore]): SELECT name FROM claroty.claroty_organization_firewall_groups
/// LIMIT 5 returns at least 1 row with non-null name. Non-empty result confirms
/// response_path = "$.organization_firewall_groups" correctly extracts from the xDome
/// response (which uses full spelling despite abbreviated URL).
/// Using "$.organization_fw_groups" in response_path causes silent empty result.
///
/// Traces to: BC-2.16.021 invariant; AC-017; EC-016-021-005/006.
/// Story: S-CLAROTY-ORGPOLICY-001 AC-017
#[tokio::test]
#[ignore = "LIVE-MONROE-001: requires CLAROTY_INSTANCE_URL env var pointing to monroe; run manually or in live-validation CI job"]
async fn test_BC_2_16_021_claroty_organization_firewall_groups_live_fw_asymmetry_nonempty_result() {
    todo!("LIVE-MONROE-001: implement when CLAROTY_INSTANCE_URL is available in test environment")
}

// ===========================================================================
// RG-018 — Live wire-shape for fw_groups (#[ignore])
// ===========================================================================

/// RG-018 (LIVE — #[ignore]): SELECT * FROM claroty.claroty_organization_firewall_groups
/// LIMIT 1 serialized JSON: class_uid=3004, name present, raw_extensions.device_conditions
/// is JSON array (NOT stringified). No Tier-2 column names as standalone root keys.
///
/// Traces to: BC-2.16.021 §PC1/§PC3/§PC6; TV-BC-2.16.021-002.
/// Story: S-CLAROTY-ORGPOLICY-001 AC-018
#[tokio::test]
#[ignore = "LIVE-MONROE-001: requires CLAROTY_INSTANCE_URL env var pointing to monroe; run manually or in live-validation CI job"]
async fn test_BC_2_16_021_claroty_organization_firewall_groups_live_wire_shape_class_uid_and_tier1()
{
    todo!("LIVE-MONROE-001: implement when CLAROTY_INSTANCE_URL is available in test environment")
}

// ===========================================================================
// RG-019 — REQUIRED firewall_group_name absent → null row, no hard error
// ===========================================================================

/// RG-019 (RED): When a fw_groups row has 'firewall_group_name' absent,
/// ColumnMapper::map_record returns Ok (no hard error). mapped_fields must not
/// contain 'name'. The column must carry REQUIRED option.
///
/// FAILS before implementation.
///
/// Traces to: BC-2.16.021 invariant; AC-019; EC-016-021-001.
/// Story: S-CLAROTY-ORGPOLICY-001 AC-019
///
/// # SAP-3 Rule-3 Defense-in-Depth Disclaimer
///
/// This test invokes `ColumnMapper::map_record` directly (pre-serialization path).
/// `map_record` has ZERO production callers — it is not on the production data path.
/// This test is **defense-in-depth only** (SAP-3 rule-3).
///
/// The authoritative production-path gate (SAP-4) is:
/// `test_BC_2_16_021_claroty_organization_firewall_groups_wire_shape_serialized_json_null_not_absent`
/// in `crates/prism-bin/tests/bc_2_16_021_claroty_org_fw_policy_wire_shape.rs`,
/// which asserts `"name": null` at the serialized JSON wire level (not absent).
#[test]
fn test_BC_2_16_021_claroty_organization_firewall_groups_required_fwgroupname_absent_produces_null_row(
) {
    let spec = load_claroty_spec();

    let fw_groups = spec
        .tables
        .iter()
        .find(|t| t.table_name == "claroty_organization_firewall_groups")
        .expect(
            "claroty.sensor.toml must contain 'claroty_organization_firewall_groups' — add per AC-019",
        );

    // Confirm firewall_group_name carries REQUIRED
    let fw_name_col = fw_groups
        .columns
        .iter()
        .find(|c| c.name == "firewall_group_name")
        .expect("column 'firewall_group_name' must exist");
    assert!(
        fw_name_col.options.contains(&ColumnOptions::Required),
        "firewall_group_name must carry REQUIRED option (PK discipline, BC-2.16.021 §PC1)"
    );

    // Row with firewall_group_name absent
    let raw_missing = json!({
        "firewall_group_description": "Group without a name",
        "priority": 3
    });

    let result = ColumnMapper::map_record(&raw_missing, fw_groups).expect(
        "ColumnMapper::map_record must return Ok even when REQUIRED field is absent; \
             BC-2.16.003: records are never dropped via hard error",
    );

    assert!(
        !result.mapped_fields.contains_key("name"),
        "mapped_fields must not contain 'name' when firewall_group_name is absent \
         (REQUIRED absent → null row semantics)"
    );

    // Subsequent row unaffected
    let raw_valid = json!({
        "firewall_group_name": "Production FW Group",
        "firewall_group_description": "Valid group"
    });
    let result_valid = ColumnMapper::map_record(&raw_valid, fw_groups)
        .expect("subsequent valid record must succeed");
    assert!(
        result_valid.mapped_fields.contains_key("name"),
        "subsequent valid record must produce mapped_fields['name']"
    );
}

// ===========================================================================
// RG-020 — Tier-2 column (firewall_group_source) raises E-QUERY-038 at plan time
// ===========================================================================

/// RG-020 (RED): ocsf_projected_column_names for claroty_organization_firewall_groups
/// with ocsf_column_naming=true must NOT include 'firewall_group_source' (Tier-2).
/// 'raw_extensions' must be present. Tier-1 Arrow names must all be present.
///
/// FAILS before implementation.
///
/// Traces to: BC-2.16.021 invariant — Tier-2 not exposed standalone; AC-020.
/// Story: S-CLAROTY-ORGPOLICY-001 AC-020
#[test]
fn test_BC_2_16_021_claroty_organization_firewall_groups_tier2_column_raises_e_query_038() {
    let spec = load_claroty_spec();

    let fw_groups = spec
        .tables
        .iter()
        .find(|t| t.table_name == "claroty_organization_firewall_groups")
        .expect(
            "claroty.sensor.toml must contain 'claroty_organization_firewall_groups' — add per AC-020",
        );

    let available = ocsf_projected_column_names(fw_groups, true);

    // firewall_group_source (Tier-2) must NOT appear as standalone Arrow column
    assert!(
        !available.contains(&"firewall_group_source".to_string()),
        "firewall_group_source (Tier-2) must NOT be in ocsf_projected_column_names; \
         selecting it raises E-QUERY-038; available: {available:?}"
    );

    // raw_extensions must be present
    assert!(
        available.contains(&"raw_extensions".to_string()),
        "raw_extensions must appear in available columns; available: {available:?}"
    );

    // Tier-1 Arrow names must be present
    for expected in [
        "name",
        "comment",
        "status_code",
        "actor_user_name",
        "class_uid",
        "_sensor",
    ] {
        assert!(
            available.contains(&expected.to_string()),
            "'{expected}' must be in ocsf_projected_column_names; available: {available:?}"
        );
    }
}

// ===========================================================================
// RG-021 — claroty_organization_firewall_policies TOML block parses
// ===========================================================================

/// RG-021 (RED): claroty.sensor.toml must declare a [[tables]] block with
/// table_name = "claroty_organization_firewall_policies", ocsf_class = "entity_management",
/// step "fetch_organization_firewall_policies", method = "POST",
/// path_template = "/api/v1/organization_fw_group_policies/"  (abbreviated URL),
/// response_path = "$.organization_firewall_policies"  (full spelling envelope key),
/// offset_limit pagination page_size = 1000,
/// body_template containing "applied_group_pairs" (NOT "applied_zone_pairs"),
/// and exactly 13 ColumnSpec entries.
///
/// FAILS before implementation.
///
/// Traces to: BC-2.16.021 §PC2 — TOML Table Contract (fw_policies).
/// Story: S-CLAROTY-ORGPOLICY-001 AC-021
#[test]
fn test_BC_2_16_021_claroty_organization_firewall_policies_toml_block_parses() {
    let spec = load_claroty_spec();

    let fw_policies = spec
        .tables
        .iter()
        .find(|t| t.table_name == "claroty_organization_firewall_policies")
        .expect(
            "claroty.sensor.toml must contain a [[tables]] block with \
             table_name = \"claroty_organization_firewall_policies\" — add per AC-021",
        );

    assert_eq!(
        fw_policies.columns.len(),
        13,
        "claroty_organization_firewall_policies must declare exactly 13 columns; \
         got {}",
        fw_policies.columns.len()
    );

    assert_eq!(fw_policies.steps.len(), 1, "must have exactly 1 fetch step");
    let step = &fw_policies.steps[0];

    assert_eq!(
        step.name, "fetch_organization_firewall_policies",
        "step name must be 'fetch_organization_firewall_policies'; got '{}'",
        step.name
    );
    assert_eq!(step.method, "POST", "step must use POST");

    // Abbreviated URL, full-spelling envelope key (same asymmetry as fw_groups)
    assert_eq!(
        step.path_template, "/api/v1/organization_fw_group_policies/",
        "path_template must be '/api/v1/organization_fw_group_policies/' (abbreviated); \
         got '{}'",
        step.path_template
    );
    assert_eq!(
        step.response_path, "$.organization_firewall_policies",
        "response_path must be '$.organization_firewall_policies' (full spelling); \
         got '{}'",
        step.response_path
    );

    match &step.pagination {
        Some(PaginationConfig::OffsetLimit { page_size }) => {
            assert_eq!(*page_size, 1000, "page_size must be 1000; got {page_size}");
        }
        other => panic!("must use OffsetLimit pagination; got: {other:?}"),
    }

    assert_eq!(
        fw_policies.ocsf_class, "entity_management",
        "ocsf_class must be 'entity_management'; got '{}'",
        fw_policies.ocsf_class
    );

    // body_template must contain "applied_group_pairs" (NOT "applied_zone_pairs")
    // This is the most critical cross-table distinction:
    //   zone_policies use "applied_zone_pairs" ({src_zone, dst_zone} pair objects)
    //   fw_policies use "applied_group_pairs"  ({src_group, dst_group} pair objects)
    let body_template = step.body_template.as_deref().unwrap_or("");
    assert!(
        body_template.contains("applied_group_pairs"),
        "body_template must contain 'applied_group_pairs' (firewall domain); \
         EC-016-021-010: using 'applied_zone_pairs' (zone domain) silently requests \
         the wrong field from xDome. body_template: '{body_template}'"
    );
    assert!(
        !body_template.contains("applied_zone_pairs"),
        "body_template must NOT contain 'applied_zone_pairs' (that belongs to zone_policies); \
         EC-016-021-010. body_template: '{body_template}'"
    );
}

// ===========================================================================
// RG-022 — fw_policies Tier-1 column inspection
// ===========================================================================

/// RG-022 (RED): claroty_organization_firewall_policies must declare exactly 4 Tier-1
/// columns (ocsf_field set) and exactly 9 Tier-2 columns (ocsf_field None):
///   policy_name → "name" (REQUIRED), policy_action → "activity_name",
///   policy_notes → "comment", updated_by → "actor.user.name".
///
/// FAILS before implementation.
///
/// Traces to: BC-2.16.021 §PC4 — Tier-1/Tier-2 classification (fw_policies).
/// Story: S-CLAROTY-ORGPOLICY-001 AC-022
#[test]
fn test_BC_2_16_021_claroty_organization_firewall_policies_tier1_columns_four_with_ocsf_field() {
    let spec = load_claroty_spec();

    let fw_policies = spec
        .tables
        .iter()
        .find(|t| t.table_name == "claroty_organization_firewall_policies")
        .expect(
            "claroty.sensor.toml must contain 'claroty_organization_firewall_policies' — add per AC-022",
        );

    let tier1_count = fw_policies
        .columns
        .iter()
        .filter(|c| c.ocsf_field.is_some())
        .count();
    let tier2_count = fw_policies
        .columns
        .iter()
        .filter(|c| c.ocsf_field.is_none())
        .count();

    assert_eq!(
        tier1_count, 4,
        "fw_policies must have exactly 4 Tier-1 columns; got {tier1_count}"
    );
    assert_eq!(
        tier2_count, 9,
        "fw_policies must have exactly 9 Tier-2 columns (incl. 3 Json); got {tier2_count}"
    );

    // policy_name → "name", REQUIRED
    let policy_name_col = fw_policies
        .columns
        .iter()
        .find(|c| c.name == "policy_name")
        .expect("column 'policy_name' must exist");
    assert_eq!(
        policy_name_col.ocsf_field.as_deref(),
        Some("name"),
        "policy_name must map ocsf_field = \"name\""
    );
    assert!(
        policy_name_col.options.contains(&ColumnOptions::Required),
        "policy_name must carry REQUIRED option"
    );

    // policy_action → "activity_name"
    let action_col = fw_policies
        .columns
        .iter()
        .find(|c| c.name == "policy_action")
        .expect("column 'policy_action' must exist");
    assert_eq!(
        action_col.ocsf_field.as_deref(),
        Some("activity_name"),
        "policy_action must map ocsf_field = \"activity_name\""
    );

    // policy_notes → "comment"
    let notes_col = fw_policies
        .columns
        .iter()
        .find(|c| c.name == "policy_notes")
        .expect("column 'policy_notes' must exist");
    assert_eq!(
        notes_col.ocsf_field.as_deref(),
        Some("comment"),
        "policy_notes must map ocsf_field = \"comment\""
    );

    // updated_by → "actor.user.name"
    let updated_by_col = fw_policies
        .columns
        .iter()
        .find(|c| c.name == "updated_by")
        .expect("column 'updated_by' must exist");
    assert_eq!(
        updated_by_col.ocsf_field.as_deref(),
        Some("actor.user.name"),
        "updated_by must map ocsf_field = \"actor.user.name\""
    );
}

// ===========================================================================
// RG-023 — Live wire-shape for fw_policies (#[ignore])
// ===========================================================================

/// RG-023 (LIVE — #[ignore]): SELECT * FROM claroty.claroty_organization_firewall_policies
/// LIMIT 1 serialized JSON: class_uid=3004, name present, activity_name present,
/// raw_extensions has communication_conditions, related_alerts_ids, applied_group_pairs
/// (NOT applied_zone_pairs) as JSON arrays. No Tier-2 column names as standalone root keys.
///
/// Traces to: BC-2.16.021 §PC2/§PC4/§PC6; TV-BC-2.16.021-007.
/// Story: S-CLAROTY-ORGPOLICY-001 AC-023
#[tokio::test]
#[ignore = "LIVE-MONROE-001: requires CLAROTY_INSTANCE_URL env var pointing to monroe; run manually or in live-validation CI job"]
async fn test_BC_2_16_021_claroty_organization_firewall_policies_live_wire_shape_class_uid_and_tier1(
) {
    todo!("LIVE-MONROE-001: implement when CLAROTY_INSTANCE_URL is available in test environment")
}

// ===========================================================================
// RG-024 — applied_group_pairs (Tier-2 Json) raises E-QUERY-038;
//           TOML column block uses applied_group_pairs NOT applied_zone_pairs
// ===========================================================================

/// RG-024 (RED): ocsf_projected_column_names for claroty_organization_firewall_policies
/// must NOT include 'applied_group_pairs' as a standalone Arrow column.
/// Also: the TOML column block must declare a column named 'applied_group_pairs'
/// (NOT 'applied_zone_pairs') — the zone-domain column name must not appear in
/// the fw_policies block (EC-016-021-010).
///
/// FAILS before implementation.
///
/// Traces to: BC-2.16.021 invariant; AC-024; EC-016-021-007/010.
/// Story: S-CLAROTY-ORGPOLICY-001 AC-024
#[test]
fn test_BC_2_16_021_claroty_organization_firewall_policies_applied_group_pairs_raises_e_query_038()
{
    let spec = load_claroty_spec();

    let fw_policies = spec
        .tables
        .iter()
        .find(|t| t.table_name == "claroty_organization_firewall_policies")
        .expect(
            "claroty.sensor.toml must contain 'claroty_organization_firewall_policies' — add per AC-024",
        );

    // Part 1: plan-time column availability (E-QUERY-038 proxy)
    let available = ocsf_projected_column_names(fw_policies, true);

    assert!(
        !available.contains(&"applied_group_pairs".to_string()),
        "applied_group_pairs (Tier-2 Json) must NOT be in ocsf_projected_column_names; \
         selecting it raises E-QUERY-038; available: {available:?}"
    );

    assert!(
        available.contains(&"raw_extensions".to_string()),
        "raw_extensions must appear in available columns; available: {available:?}"
    );

    for expected in [
        "name",
        "activity_name",
        "comment",
        "actor_user_name",
        "class_uid",
        "_sensor",
    ] {
        assert!(
            available.contains(&expected.to_string()),
            "'{expected}' must be in ocsf_projected_column_names; available: {available:?}"
        );
    }

    // Part 2: TOML column block uses 'applied_group_pairs' (firewall domain)
    //         NOT 'applied_zone_pairs' (zone domain) — EC-016-021-010
    let has_group_pairs = fw_policies
        .columns
        .iter()
        .any(|c| c.name == "applied_group_pairs");
    assert!(
        has_group_pairs,
        "claroty_organization_firewall_policies must declare a column named 'applied_group_pairs' \
         (firewall domain); EC-016-021-010: using 'applied_zone_pairs' (zone domain) silently \
         requests the wrong xDome field"
    );

    let has_zone_pairs = fw_policies
        .columns
        .iter()
        .any(|c| c.name == "applied_zone_pairs");
    assert!(
        !has_zone_pairs,
        "claroty_organization_firewall_policies must NOT declare 'applied_zone_pairs' \
         (that belongs to claroty_organization_zone_policies); \
         EC-016-021-010: column name cross-contamination between zone and firewall domains"
    );
}

// ===========================================================================
// RG-025a — REQUIRED policy_name absent → null row, no hard error
// ===========================================================================

/// RG-025a (RED): When a fw_policies row has 'policy_name' absent,
/// ColumnMapper::map_record returns Ok (no hard error). mapped_fields must not
/// contain 'name'.
///
/// FAILS before implementation.
///
/// Traces to: BC-2.16.021 invariant; AC-025 (first sub-test); EC-016-021-002.
/// Story: S-CLAROTY-ORGPOLICY-001 AC-025
///
/// # SAP-3 Rule-3 Defense-in-Depth Disclaimer
///
/// This test invokes `ColumnMapper::map_record` directly (pre-serialization path).
/// `map_record` has ZERO production callers — it is not on the production data path.
/// This test is **defense-in-depth only** (SAP-3 rule-3).
///
/// The authoritative production-path gate (SAP-4) is:
/// `test_BC_2_16_021_claroty_organization_firewall_policies_wire_shape_serialized_json_null_not_absent`
/// in `crates/prism-bin/tests/bc_2_16_021_claroty_org_fw_policy_wire_shape.rs`,
/// which asserts `"name": null` at the serialized JSON wire level (not absent).
#[test]
fn test_BC_2_16_021_claroty_organization_firewall_policies_required_policy_name_absent_produces_null_row(
) {
    let spec = load_claroty_spec();

    let fw_policies = spec
        .tables
        .iter()
        .find(|t| t.table_name == "claroty_organization_firewall_policies")
        .expect(
            "claroty.sensor.toml must contain 'claroty_organization_firewall_policies' — add per AC-025a",
        );

    // Confirm policy_name carries REQUIRED
    let policy_name_col = fw_policies
        .columns
        .iter()
        .find(|c| c.name == "policy_name")
        .expect("column 'policy_name' must exist");
    assert!(
        policy_name_col.options.contains(&ColumnOptions::Required),
        "policy_name must carry REQUIRED option"
    );

    // Row with policy_name absent
    let raw_missing = json!({
        "policy_action": "Deny",
        "matching_devices": 10
    });

    let result = ColumnMapper::map_record(&raw_missing, fw_policies).expect(
        "ColumnMapper::map_record must return Ok even when REQUIRED field is absent; \
             BC-2.16.003: records are never dropped via hard error",
    );

    assert!(
        !result.mapped_fields.contains_key("name"),
        "mapped_fields must not contain 'name' when policy_name is absent (null row)"
    );
}

// ===========================================================================
// RG-025b — count: null in fw_policies envelope → empty-page halt, no error
// ===========================================================================

/// RG-025b (RED): The claroty_organization_firewall_policies table uses offset_limit
/// pagination. When the envelope contains count: null, the empty-page halt mechanism
/// applies — no error raised.
///
/// Structural test: verifies OffsetLimit config is in place (EC-016-021-004).
///
/// FAILS before implementation (table absent → .expect() panics).
///
/// Traces to: BC-2.16.021 §PC2 pagination; AC-025 (second sub-test); EC-016-021-004.
/// Story: S-CLAROTY-ORGPOLICY-001 AC-025
#[test]
fn test_BC_2_16_021_claroty_organization_firewall_policies_nullable_count_uses_empty_page_halt() {
    let spec = load_claroty_spec();

    let fw_policies = spec
        .tables
        .iter()
        .find(|t| t.table_name == "claroty_organization_firewall_policies")
        .expect(
            "claroty.sensor.toml must contain 'claroty_organization_firewall_policies' — add per AC-025b",
        );

    assert_eq!(fw_policies.steps.len(), 1, "must have exactly 1 step");

    match &fw_policies.steps[0].pagination {
        Some(PaginationConfig::OffsetLimit { page_size }) => {
            assert_eq!(
                *page_size, 1000,
                "page_size must be 1000; EC-016-021-004 empty-page halt requires OffsetLimit"
            );
        }
        other => panic!(
            "claroty_organization_firewall_policies must use OffsetLimit pagination; \
             count: null safety depends on empty-page halt; got: {other:?}"
        ),
    }

    assert_eq!(
        fw_policies.steps[0].response_path, "$.organization_firewall_policies",
        "response_path must be '$.organization_firewall_policies'; got '{}'",
        fw_policies.steps[0].response_path
    );

    // Simulate: envelope with count: null and empty data array
    let empty_envelope = json!({
        "organization_firewall_policies": [],
        "count": null
    });
    let data_array = empty_envelope
        .get("organization_firewall_policies")
        .expect("organization_firewall_policies key must be present");
    assert!(
        data_array.is_array() && data_array.as_array().unwrap().is_empty(),
        "empty page ([] data) must halt pagination regardless of count: null; \
         got: {data_array:?} (EC-016-021-004)"
    );
}

// ===========================================================================
// RG-026 — Json columns (comm_conditions, related_alerts_ids, applied_group_pairs)
//           NOT stringified; applied_group_pairs ≠ applied_zone_pairs
// ===========================================================================

/// RG-026 (RED): All three Json columns of fw_policies must appear in raw_extensions
/// as JSON-typed values (arrays), NOT as JSON string encodings.
///   - communication_conditions: array of condition objects
///   - related_alerts_ids: array of triggered alert IDs
///   - applied_group_pairs: array of {src_group, dst_group} pair objects (firewall domain)
///     NOTE: applied_GROUP_pairs for firewall_policies — NOT applied_ZONE_pairs
///
/// Wire-shape assertion (2026-07-13 discipline): each value must be Value::Array.
/// SID-2 composed-output: assert all three keys present and correct independently.
/// Also: raw_extensions must NOT contain 'applied_zone_pairs' key.
///
/// FAILS before implementation.
///
/// Traces to: BC-2.16.021 §PC6; AC-026; spike-findings §Spike 3 §Table D.
/// Story: S-CLAROTY-ORGPOLICY-001 AC-026
///
/// # SAP-3 Rule-3 Defense-in-Depth Disclaimer
///
/// This test invokes `ColumnMapper::map_record` directly (pre-serialization path).
/// `map_record` has ZERO production callers — it is not on the production data path.
/// This test is **defense-in-depth only** (SAP-3 rule-3).
///
/// The authoritative production-path gate (SAP-4) is:
/// `test_BC_2_16_021_claroty_organization_firewall_policies_wire_shape_class_uid_3004_mock`
/// in `crates/prism-bin/tests/bc_2_16_021_claroty_org_fw_policy_wire_shape.rs`,
/// which asserts communication_conditions/related_alerts_ids/applied_group_pairs are
/// NATIVE JSON arrays in raw_extensions through `SpecDrivenSensorAdapter::fetch`.
#[test]
fn test_BC_2_16_021_claroty_organization_firewall_policies_json_columns_not_stringified() {
    let spec = load_claroty_spec();

    let fw_policies = spec
        .tables
        .iter()
        .find(|t| t.table_name == "claroty_organization_firewall_policies")
        .expect(
            "claroty.sensor.toml must contain 'claroty_organization_firewall_policies' — add per AC-026",
        );

    // Verify all three Json columns are declared column_type = "json"
    for col_name in [
        "communication_conditions",
        "related_alerts_ids",
        "applied_group_pairs",
    ] {
        let col = fw_policies
            .columns
            .iter()
            .find(|c| c.name == col_name)
            .unwrap_or_else(|| {
                panic!("column '{col_name}' must exist in claroty_organization_firewall_policies")
            });
        assert_eq!(
            col.column_type,
            prism_core::column::ColumnType::Json,
            "column '{col_name}' must be column_type = \"json\" (not String); \
             declaring as String causes stringification defect (AC-026)"
        );
        assert!(
            col.ocsf_field.is_none(),
            "'{col_name}' must be Tier-2 (no ocsf_field) → aggregates into raw_extensions"
        );
    }

    // Confirm 'applied_zone_pairs' does NOT exist in fw_policies
    let has_zone_pairs = fw_policies
        .columns
        .iter()
        .any(|c| c.name == "applied_zone_pairs");
    assert!(
        !has_zone_pairs,
        "claroty_organization_firewall_policies must NOT declare 'applied_zone_pairs'; \
         that column belongs to zone_policies (EC-016-021-010)"
    );

    // Wire-shape test: simulate a raw record with all three Json columns
    // applied_group_pairs contains {src_group, dst_group} (firewall domain)
    let raw = json!({
        "policy_name": "FW Policy Alpha",
        "communication_conditions": [{"protocol": "TCP", "port": 443}],
        "related_alerts_ids": [201, 302],
        "applied_group_pairs": [{"src_group": "DMZ", "dst_group": "Internal"}]
    });

    let result =
        ColumnMapper::map_record(&raw, fw_policies).expect("ColumnMapper::map_record must succeed");

    // SID-2 composed-output: assert each key independently
    for json_col in [
        "communication_conditions",
        "related_alerts_ids",
        "applied_group_pairs",
    ] {
        let val = result
            .raw_extensions
            .get(json_col)
            .unwrap_or_else(|| panic!("'{json_col}' must appear in raw_extensions"));

        assert!(
            val.is_array(),
            "'{json_col}' in raw_extensions must be a JSON array (not stringified); \
             wire-shape discipline 2026-07-13; got: {val:?}"
        );

        let wire = serde_json::to_string(val).expect("raw_extensions value must be serializable");
        assert!(
            wire.starts_with('['),
            "'{json_col}' wire representation must start with '['; got: {wire}"
        );
    }

    // Specific: applied_group_pairs contains {src_group, dst_group} (firewall domain)
    let applied_group = result
        .raw_extensions
        .get("applied_group_pairs")
        .unwrap()
        .as_array()
        .unwrap();
    let first_pair = &applied_group[0];
    assert!(
        first_pair.get("src_group").is_some(),
        "applied_group_pairs elements must contain 'src_group' (firewall domain)"
    );
    assert!(
        first_pair.get("dst_group").is_some(),
        "applied_group_pairs elements must contain 'dst_group' (firewall domain)"
    );

    // SID-2 no-duplication: applied_group_pairs must NOT contain zone-domain keys
    assert!(
        first_pair.get("src_zone").is_none(),
        "applied_group_pairs must NOT contain 'src_zone' — that belongs to applied_zone_pairs \
         in the zone domain (BC-2.16.020 §PC4)"
    );

    // raw_extensions must NOT have 'applied_zone_pairs' key (wrong column for this table)
    assert!(
        !result.raw_extensions.contains_key("applied_zone_pairs"),
        "raw_extensions must NOT contain 'applied_zone_pairs' for fw_policies \
         (that key belongs to zone_policies — EC-016-021-010)"
    );
}
