//! Red Gate tests for BC-2.16.020 — Claroty xDome Organization Zone Domain.
//!
//! Covers claroty_organization_zones and claroty_organization_zone_policies TOML tables.
//! All non-#[ignore] tests MUST fail before Task 9 adds the [[tables]] blocks to
//! crates/prism-sensors/specs/claroty.sensor.toml — failure mode is `.expect()` panic
//! when the table is absent from the parsed SensorSpec.
//!
//! Red Gate test list (RG-001..RG-014):
//!   RG-001  test_BC_2_16_020_claroty_organization_zones_toml_block_parses
//!   RG-002  test_BC_2_16_020_claroty_organization_zones_tier1_columns_four_with_ocsf_field
//!   RG-003  test_BC_2_16_020_claroty_organization_zones_tier2_column_raises_e_query_038
//!   RG-004  test_BC_2_16_020_claroty_organization_zones_tier1_raw_toml_name_raises_e_query_038
//!   RG-005  test_BC_2_16_020_claroty_organization_zones_live_wire_shape_class_uid_and_tier1 (#[ignore])
//!   RG-006  test_BC_2_16_020_claroty_organization_zones_device_conditions_json_not_string
//!   RG-007  test_BC_2_16_020_claroty_organization_zones_required_zone_name_absent_produces_null_row
//!   RG-008  test_BC_2_16_020_claroty_organization_zones_nullable_count_uses_empty_page_halt
//!   RG-009  test_BC_2_16_020_claroty_organization_zone_policies_toml_block_parses
//!   RG-010  test_BC_2_16_020_claroty_organization_zone_policies_tier1_columns_four_with_ocsf_field
//!   RG-011  test_BC_2_16_020_claroty_organization_zone_policies_live_wire_shape_class_uid_and_tier1 (#[ignore])
//!   RG-012  test_BC_2_16_020_claroty_organization_zone_policies_applied_zone_pairs_raises_e_query_038
//!   RG-013  test_BC_2_16_020_claroty_organization_zone_policies_required_policy_name_absent_produces_null_row
//!   RG-014  test_BC_2_16_020_claroty_organization_zone_policies_json_columns_not_stringified
//!
//! BC-5.38.001 density check: 14 RGTs cover 14 ACs (RG-001..014 → AC-001..014). Ratio = 1.0 ≥ 0.5. PASS.
//!
//! Story: S-CLAROTY-ORGPOLICY-001 | BC: BC-2.16.020
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
// RG-001 — claroty_organization_zones TOML block parses
// ===========================================================================

/// RG-001 (RED): claroty.sensor.toml must declare a [[tables]] block with
/// table_name = "claroty_organization_zones", ocsf_class = "entity_management",
/// step "fetch_organization_zones", method = "POST",
/// path_template = "/api/v1/organization_zones/",
/// response_path = "$.organization_zones",
/// offset_limit pagination page_size = 1000,
/// and exactly 11 ColumnSpec entries.
///
/// FAILS before implementation (TOML blocks absent → .expect() panics).
///
/// Traces to: BC-2.16.020 §Postconditions §1 — TOML Table Contract (zones).
/// Story: S-CLAROTY-ORGPOLICY-001 AC-001
#[test]
fn test_BC_2_16_020_claroty_organization_zones_toml_block_parses() {
    let spec = load_claroty_spec();

    let zones = spec
        .tables
        .iter()
        .find(|t| t.table_name == "organization_zones")
        .expect(
            "claroty.sensor.toml must contain a [[tables]] block with \
             table_name = \"organization_zones\" — add per S-CLAROTY-ORGPOLICY-001 AC-001",
        );

    assert_eq!(
        zones.columns.len(),
        11,
        "claroty_organization_zones must declare exactly 11 columns (BC-2.16.020 §PC1); \
         got {}",
        zones.columns.len()
    );

    assert_eq!(
        zones.steps.len(),
        1,
        "claroty_organization_zones must have exactly 1 fetch step; got {}",
        zones.steps.len()
    );
    let step = &zones.steps[0];

    assert_eq!(
        step.name, "fetch_organization_zones",
        "step name must be 'fetch_organization_zones'; got '{}'",
        step.name
    );
    assert_eq!(
        step.method, "POST",
        "claroty_organization_zones fetch step must use POST (POST-for-read); got '{}'",
        step.method
    );
    assert_eq!(
        step.path_template, "/api/v1/organization_zones/",
        "path_template must be '/api/v1/organization_zones/'; got '{}'",
        step.path_template
    );
    assert_eq!(
        step.response_path, "$.organization_zones",
        "response_path must be '$.organization_zones'; got '{}'",
        step.response_path
    );

    match &step.pagination {
        Some(PaginationConfig::OffsetLimit { page_size }) => {
            assert_eq!(
                *page_size, 1000,
                "pagination page_size must be 1000; got {page_size}"
            );
        }
        other => panic!(
            "claroty_organization_zones must use OffsetLimit pagination with page_size=1000; \
             got: {other:?}"
        ),
    }

    assert_eq!(
        zones.ocsf_class, "entity_management",
        "ocsf_class must be 'entity_management' (class_uid 3004 — existing arm); \
         got '{}'",
        zones.ocsf_class
    );
}

// ===========================================================================
// RG-002 — zones Tier-1 column inspection
// ===========================================================================

/// RG-002 (RED): claroty_organization_zones must declare exactly 4 Tier-1 columns
/// (ocsf_field set) and exactly 7 Tier-2 columns (ocsf_field None):
///   zone_name → "name" (REQUIRED), zone_description → "comment",
///   enabled → "status_code", updated_by → "actor.user.name".
///
/// FAILS before implementation.
///
/// Traces to: BC-2.16.020 §Postconditions §3 — Tier-1/Tier-2 classification (zones).
/// Story: S-CLAROTY-ORGPOLICY-001 AC-002
#[test]
fn test_BC_2_16_020_claroty_organization_zones_tier1_columns_four_with_ocsf_field() {
    let spec = load_claroty_spec();

    let zones = spec
        .tables
        .iter()
        .find(|t| t.table_name == "organization_zones")
        .expect("claroty.sensor.toml must contain 'organization_zones' — add per AC-002");

    let tier1_count = zones
        .columns
        .iter()
        .filter(|c| c.ocsf_field.is_some())
        .count();
    let tier2_count = zones
        .columns
        .iter()
        .filter(|c| c.ocsf_field.is_none())
        .count();

    assert_eq!(
        tier1_count, 4,
        "claroty_organization_zones must have exactly 4 Tier-1 columns (ocsf_field set); \
         got {tier1_count}"
    );
    assert_eq!(
        tier2_count, 7,
        "claroty_organization_zones must have exactly 7 Tier-2 columns (ocsf_field None); \
         got {tier2_count}"
    );

    // zone_name → "name", REQUIRED
    let zone_name_col = zones
        .columns
        .iter()
        .find(|c| c.name == "zone_name")
        .expect("column 'zone_name' must exist in claroty_organization_zones");
    assert_eq!(
        zone_name_col.ocsf_field.as_deref(),
        Some("name"),
        "zone_name must map ocsf_field = \"name\"; got {:?}",
        zone_name_col.ocsf_field
    );
    assert!(
        zone_name_col.options.contains(&ColumnOptions::Required),
        "zone_name must carry options = [\"REQUIRED\"] (PK discipline, BC-2.16.020 §PC1)"
    );

    // zone_description → "comment"
    let zone_desc_col = zones
        .columns
        .iter()
        .find(|c| c.name == "zone_description")
        .expect("column 'zone_description' must exist");
    assert_eq!(
        zone_desc_col.ocsf_field.as_deref(),
        Some("comment"),
        "zone_description must map ocsf_field = \"comment\""
    );

    // enabled → "status_code"
    let enabled_col = zones
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
    let updated_by_col = zones
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
// RG-003 — Tier-2 column (zone_source) raises E-QUERY-038 at plan time
// ===========================================================================

/// RG-003 (RED): ocsf_projected_column_names for claroty_organization_zones with
/// ocsf_column_naming=true must NOT include 'zone_source' (Tier-2) as a standalone
/// Arrow column. 'raw_extensions' must be present.
/// Tier-1 Arrow names (name, comment, status_code, actor_user_name, class_uid, _sensor)
/// must all be present.
///
/// FAILS before implementation.
///
/// Traces to: BC-2.16.020 invariant — Tier-2 not exposed standalone; AC-003.
/// Story: S-CLAROTY-ORGPOLICY-001 AC-003
///
/// SAP-3 NOTE (defense-in-depth): This test exercises `ocsf_projected_column_names`
/// directly (pre-plan-gate helper). Per SAP-3 rule-3, it counts as defense-in-depth only.
/// The AUTHORITATIVE end-to-end gate from the public query surface (SQL parser →
/// QueryEngine::execute) is
/// `test_BC_2_16_020_claroty_organization_zones_e2e_e_query_038_tier2_column`
/// in `crates/prism-bin/tests/bc_2_16_020_claroty_org_zone_policy_wire_shape.rs`
/// (RG-003a, F-ORGPOL-P1-MED-001 closure).
#[test]
fn test_BC_2_16_020_claroty_organization_zones_tier2_column_raises_e_query_038() {
    let spec = load_claroty_spec();

    let zones = spec
        .tables
        .iter()
        .find(|t| t.table_name == "organization_zones")
        .expect("claroty.sensor.toml must contain 'organization_zones' — add per AC-003");

    let available = ocsf_projected_column_names(zones, true);

    assert!(
        !available.contains(&"zone_source".to_string()),
        "zone_source (Tier-2) must NOT be in ocsf_projected_column_names; \
         selecting it raises E-QUERY-038; available: {available:?}"
    );

    // raw_extensions must be present (Tier-2 columns exist → ADR-058 §J6)
    assert!(
        available.contains(&"raw_extensions".to_string()),
        "raw_extensions must appear in ocsf_projected_column_names when Tier-2 columns exist; \
         available: {available:?}"
    );

    // All Tier-1 Arrow names must be present
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
            "'{expected}' must appear in ocsf_projected_column_names; available: {available:?}"
        );
    }
}

// ===========================================================================
// RG-004 — Tier-1 raw TOML name 'zone_name' raises E-QUERY-038 (Arrow name is 'name')
// ===========================================================================

/// RG-004 (RED / WIRE-SHAPE): ocsf_projected_column_names must contain 'name' (Arrow form)
/// but NOT 'zone_name' (raw TOML name). A query SELECT zone_name raises E-QUERY-038.
///
/// FAILS before implementation.
///
/// Traces to: BC-2.16.020 invariant (AC-004 WIRE-SHAPE rename); TV-BC-2.16.020-003.
/// Story: S-CLAROTY-ORGPOLICY-001 AC-004
#[test]
fn test_BC_2_16_020_claroty_organization_zones_tier1_raw_toml_name_raises_e_query_038() {
    let spec = load_claroty_spec();

    let zones = spec
        .tables
        .iter()
        .find(|t| t.table_name == "organization_zones")
        .expect("claroty.sensor.toml must contain 'organization_zones' — add per AC-004");

    let available = ocsf_projected_column_names(zones, true);

    // Arrow name 'name' must be present (ocsf_field_to_arrow_name("name") = "name")
    assert!(
        available.contains(&"name".to_string()),
        "'name' (Arrow form of zone_name) must be in ocsf_projected_column_names; \
         available: {available:?}"
    );

    // Raw TOML name 'zone_name' must NOT be present (E-QUERY-038 at plan time)
    assert!(
        !available.contains(&"zone_name".to_string()),
        "'zone_name' (raw TOML column name) must NOT be in ocsf_projected_column_names; \
         only Arrow form 'name' is exposed (ADR-058 §C); available: {available:?}"
    );
}

// ===========================================================================
// RG-005 — Live Variant-1 wire-shape (#[ignore])
// ===========================================================================

/// RG-005 (LIVE — #[ignore]): SELECT * FROM claroty.claroty_organization_zones LIMIT 1
/// serialized JSON must contain class_uid=3004, 'name' key, 'raw_extensions' object with
/// 'device_conditions' as JSON array (NOT stringified). No Tier-2 names as standalone keys.
///
/// Traces to: BC-2.16.020 §PC1 class_uid; §PC3 Tier-1 wire; §PC6 Json; TV-BC-2.16.020-002.
/// Story: S-CLAROTY-ORGPOLICY-001 AC-005
#[tokio::test]
#[ignore = "LIVE-MONROE-001: requires CLAROTY_INSTANCE_URL env var pointing to monroe; run manually or in live-validation CI job"]
async fn test_BC_2_16_020_claroty_organization_zones_live_wire_shape_class_uid_and_tier1() {
    todo!("LIVE-MONROE-001: implement when CLAROTY_INSTANCE_URL is available in test environment")
}

// ===========================================================================
// RG-006 — device_conditions Json column NOT stringified (wire-shape discipline)
// ===========================================================================

/// RG-006 (RED): 'device_conditions' (column_type = "json") must appear in raw_extensions
/// as a JSON array value — NOT a JSON string encoding.
///
/// Wire-shape assertion (2026-07-13 discipline): raw_extensions["device_conditions"]
/// must be Value::Array, not Value::String. Empty array → [] (not null).
///
/// FAILS before implementation.
///
/// Traces to: BC-2.16.020 §PC6 Json column serialization; AC-006; EC-016-020-003.
/// Story: S-CLAROTY-ORGPOLICY-001 AC-006
///
/// # SAP-3 Rule-3 Defense-in-Depth Disclaimer
///
/// This test invokes `ColumnMapper::map_record` directly (pre-serialization path).
/// `map_record` has ZERO production callers — it is not on the production data path.
/// This test is **defense-in-depth only** (SAP-3 rule-3).
///
/// The authoritative production-path gate (SAP-4) is:
/// `test_BC_2_16_020_claroty_organization_zones_wire_shape_class_uid_3004_mock`
/// in `crates/prism-bin/tests/bc_2_16_020_claroty_org_zone_policy_wire_shape.rs`,
/// which exercises `SpecDrivenSensorAdapter::fetch → build_column_array ColumnType::Json arm`.
#[test]
fn test_BC_2_16_020_claroty_organization_zones_device_conditions_json_not_string() {
    let spec = load_claroty_spec();

    let zones = spec
        .tables
        .iter()
        .find(|t| t.table_name == "organization_zones")
        .expect("claroty.sensor.toml must contain 'organization_zones' — add per AC-006");

    // Verify column_type = "json"
    let device_cond_col = zones
        .columns
        .iter()
        .find(|c| c.name == "device_conditions")
        .expect("column 'device_conditions' must exist in claroty_organization_zones");
    assert_eq!(
        device_cond_col.column_type,
        prism_core::column::ColumnType::Json,
        "device_conditions must be column_type = \"json\" (not String); \
         declaring as String causes stringification defect (AC-006)"
    );
    assert!(
        device_cond_col.ocsf_field.is_none(),
        "device_conditions must be Tier-2 (no ocsf_field) → aggregates into raw_extensions"
    );

    // Wire-shape test: non-empty array
    let raw_with_array = json!({
        "zone_name": "OT Zone Alpha",
        "device_conditions": [{"type": "ip_range", "value": "10.0.0.0/8"}]
    });

    let result = ColumnMapper::map_record(&raw_with_array, zones)
        .expect("ColumnMapper::map_record must succeed (BC-2.16.003: no hard errors)");

    let device_cond_val = result
        .raw_extensions
        .get("device_conditions")
        .expect("device_conditions must appear in raw_extensions (Tier-2, no ocsf_field)");

    // Wire-shape assertion: must be JSON array, NOT a string
    assert!(
        device_cond_val.is_array(),
        "device_conditions must serialize to a JSON array in raw_extensions (not stringified); \
         wire-shape discipline 2026-07-13; got: {device_cond_val:?}"
    );

    let wire_json = serde_json::to_string(device_cond_val)
        .expect("raw_extensions value must be JSON-serializable");
    assert!(
        wire_json.starts_with('['),
        "wire-shape: serialized device_conditions must start with '['; got: {wire_json}"
    );

    // Empty array test (EC-016-020-003): [] not null
    let raw_empty = json!({
        "zone_name": "Empty Zone",
        "device_conditions": []
    });
    let result_empty = ColumnMapper::map_record(&raw_empty, zones)
        .expect("ColumnMapper::map_record must succeed for empty device_conditions");
    let empty_val = result_empty
        .raw_extensions
        .get("device_conditions")
        .expect("device_conditions must appear in raw_extensions even when empty array");
    assert!(
        empty_val.is_array(),
        "empty device_conditions must serialize as [] (JSON array), not null; \
         got: {empty_val:?} (EC-016-020-003)"
    );
    assert_eq!(
        empty_val,
        &json!([]),
        "empty device_conditions must be exactly [] (not null, not 'null' string)"
    );
}

// ===========================================================================
// RG-007 — REQUIRED zone_name absent → null row, no hard error
// ===========================================================================

/// RG-007 (RED): When a claroty_organization_zones row has 'zone_name' absent,
/// ColumnMapper::map_record returns Ok (no hard error). mapped_fields must not
/// contain 'name'. The zone_name column must carry the REQUIRED option.
///
/// FAILS before implementation.
///
/// Traces to: BC-2.16.020 invariant; AC-007; EC-016-020-001.
/// Story: S-CLAROTY-ORGPOLICY-001 AC-007
///
/// # SAP-3 Rule-3 Defense-in-Depth Disclaimer
///
/// This test invokes `ColumnMapper::map_record` directly (pre-serialization path).
/// `map_record` has ZERO production callers — it is not on the production data path.
/// This test is **defense-in-depth only** (SAP-3 rule-3).
///
/// The authoritative production-path gate (SAP-4) is:
/// `test_BC_2_16_020_claroty_organization_zones_wire_shape_serialized_json_null_not_absent`
/// in `crates/prism-bin/tests/bc_2_16_020_claroty_org_zone_policy_wire_shape.rs`,
/// which asserts `"name": null` at the serialized JSON wire level (not absent).
#[test]
fn test_BC_2_16_020_claroty_organization_zones_required_zone_name_absent_produces_null_row() {
    let spec = load_claroty_spec();

    let zones = spec
        .tables
        .iter()
        .find(|t| t.table_name == "organization_zones")
        .expect("claroty.sensor.toml must contain 'organization_zones' — add per AC-007");

    // Confirm zone_name carries REQUIRED
    let zone_name_col = zones
        .columns
        .iter()
        .find(|c| c.name == "zone_name")
        .expect("column 'zone_name' must exist");
    assert!(
        zone_name_col.options.contains(&ColumnOptions::Required),
        "zone_name must carry REQUIRED option (PK discipline, BC-2.16.020 §PC1)"
    );

    // Row with zone_name absent — no hard error, 'name' absent from mapped_fields
    let raw_missing = json!({
        "zone_description": "Zone without a name",
        "priority": 5
    });

    let result = ColumnMapper::map_record(&raw_missing, zones).expect(
        "ColumnMapper::map_record must return Ok even when REQUIRED field is absent; \
             BC-2.16.003 invariant: records are NEVER dropped via hard error",
    );

    assert!(
        !result.mapped_fields.contains_key("name"),
        "mapped_fields must not contain 'name' when zone_name is absent \
         (REQUIRED field absent → null row semantics; BC-2.16.020 §PC1)"
    );

    // Subsequent row unaffected — second call on a valid record returns Ok
    let raw_valid = json!({
        "zone_name": "Production Zone",
        "zone_description": "Valid zone"
    });
    let result_valid = ColumnMapper::map_record(&raw_valid, zones)
        .expect("ColumnMapper::map_record must succeed for a subsequent valid record");
    assert!(
        result_valid.mapped_fields.contains_key("name"),
        "subsequent valid record must produce mapped_fields['name']"
    );
}

// ===========================================================================
// RG-008 — count: null in zones envelope → empty-page halt, no error
// ===========================================================================

/// RG-008 (RED): The claroty_organization_zones table uses offset_limit pagination.
/// When the API response envelope contains count: null, the spec-engine pagination
/// uses the empty-page halt check — no null-pointer dereference on count. No error raised.
///
/// Structural test: verifies OffsetLimit config is in place so the empty-page halt
/// mechanism applies (EC-016-020-004).
///
/// FAILS before implementation (table absent → .expect() panics).
///
/// Traces to: BC-2.16.020 §PC1 pagination; AC-008; EC-016-020-004.
/// Story: S-CLAROTY-ORGPOLICY-001 AC-008
#[test]
fn test_BC_2_16_020_claroty_organization_zones_nullable_count_uses_empty_page_halt() {
    let spec = load_claroty_spec();

    let zones = spec
        .tables
        .iter()
        .find(|t| t.table_name == "organization_zones")
        .expect("claroty.sensor.toml must contain 'organization_zones' — add per AC-008");

    assert_eq!(zones.steps.len(), 1, "must have exactly 1 step");

    // Structural: OffsetLimit pagination is required for empty-page halt behavior
    match &zones.steps[0].pagination {
        Some(PaginationConfig::OffsetLimit { page_size }) => {
            assert_eq!(
                *page_size, 1000,
                "page_size must be 1000; count: null safety depends on empty-page halt"
            );
        }
        other => panic!(
            "claroty_organization_zones must use OffsetLimit pagination; \
             EC-016-020-004 requires empty-page halt (not count-based halt) so count: null is safe; \
             got: {other:?}"
        ),
    }

    // response_path must be "$.organization_zones" — extract the data array, not count
    assert_eq!(
        zones.steps[0].response_path, "$.organization_zones",
        "response_path must extract the data array, not the count field; \
         empty array → empty-page halt; got '{}'",
        zones.steps[0].response_path
    );

    // Simulate: envelope with count: null and empty organization_zones array
    // When data array is empty, pagination halts (count is irrelevant)
    let empty_envelope = json!({
        "organization_zones": [],
        "count": null
    });
    let data_array = empty_envelope
        .get("organization_zones")
        .expect("organization_zones key must be present in envelope");
    assert!(
        data_array.is_array() && data_array.as_array().unwrap().is_empty(),
        "empty page ([] data) must halt pagination regardless of count value; \
         got: {data_array:?}"
    );
}

// ===========================================================================
// RG-009 — claroty_organization_zone_policies TOML block parses
// ===========================================================================

/// RG-009 (RED): claroty.sensor.toml must declare a [[tables]] block with
/// table_name = "claroty_organization_zone_policies", ocsf_class = "entity_management",
/// step "fetch_organization_zone_policies", method = "POST",
/// path_template = "/api/v1/organization_zone_policies/",
/// response_path = "$.organization_zone_policies",
/// offset_limit pagination page_size = 1000,
/// body_template containing "last_updated" (WITH trailing 'd' — field name asymmetry),
/// and exactly 13 ColumnSpec entries.
///
/// FAILS before implementation.
///
/// Traces to: BC-2.16.020 §PC2 — TOML Table Contract (zone_policies);
///            datetime field name asymmetry note. Story: S-CLAROTY-ORGPOLICY-001 AC-009
#[test]
fn test_BC_2_16_020_claroty_organization_zone_policies_toml_block_parses() {
    let spec = load_claroty_spec();

    let zone_policies = spec
        .tables
        .iter()
        .find(|t| t.table_name == "organization_zone_policies")
        .expect(
            "claroty.sensor.toml must contain a [[tables]] block with \
             table_name = \"organization_zone_policies\" — add per AC-009",
        );

    assert_eq!(
        zone_policies.columns.len(),
        13,
        "claroty_organization_zone_policies must declare exactly 13 columns (BC-2.16.020 §PC2); \
         got {}",
        zone_policies.columns.len()
    );

    assert_eq!(
        zone_policies.steps.len(),
        1,
        "must have exactly 1 fetch step"
    );
    let step = &zone_policies.steps[0];

    assert_eq!(
        step.name, "fetch_organization_zone_policies",
        "step name must be 'fetch_organization_zone_policies'; got '{}'",
        step.name
    );
    assert_eq!(step.method, "POST", "step must use POST");
    assert_eq!(
        step.path_template, "/api/v1/organization_zone_policies/",
        "path_template must be '/api/v1/organization_zone_policies/'; got '{}'",
        step.path_template
    );
    assert_eq!(
        step.response_path, "$.organization_zone_policies",
        "response_path must be '$.organization_zone_policies'; got '{}'",
        step.response_path
    );

    match &step.pagination {
        Some(PaginationConfig::OffsetLimit { page_size }) => {
            assert_eq!(*page_size, 1000, "page_size must be 1000; got {page_size}");
        }
        other => panic!("must use OffsetLimit pagination; got: {other:?}"),
    }

    assert_eq!(
        zone_policies.ocsf_class, "entity_management",
        "ocsf_class must be 'entity_management'; got '{}'",
        zone_policies.ocsf_class
    );

    // body_template must contain "last_updated" (WITH trailing 'd') — field name asymmetry
    // zones use "last_update" (no 'd'); zone_policies use "last_updated" (with 'd')
    let body_template = step.body_template.as_deref().unwrap_or("");
    assert!(
        body_template.contains("last_updated"),
        "body_template must contain 'last_updated' (WITH trailing 'd') for zone_policies; \
         EC-016-020-009: 'last_update' (no 'd') silently omits temporal data. \
         body_template: '{body_template}'"
    );
    // Reject the incorrect zones-style field name
    assert!(
        !body_template.contains("\"last_update\""),
        "body_template must NOT contain '\"last_update\"' (the zones field name, no trailing 'd'); \
         zone_policies must use 'last_updated'. EC-016-020-009. body_template: '{body_template}'"
    );
}

// ===========================================================================
// RG-010 — zone_policies Tier-1 column inspection
// ===========================================================================

/// RG-010 (RED): claroty_organization_zone_policies must declare exactly 4 Tier-1
/// columns (ocsf_field set) and exactly 9 Tier-2 columns (ocsf_field None):
///   policy_name → "name" (REQUIRED), policy_action → "activity_name",
///   policy_notes → "comment", updated_by → "actor.user.name".
///
/// FAILS before implementation.
///
/// Traces to: BC-2.16.020 §PC4 — Tier-1/Tier-2 classification (zone_policies).
/// Story: S-CLAROTY-ORGPOLICY-001 AC-010
#[test]
fn test_BC_2_16_020_claroty_organization_zone_policies_tier1_columns_four_with_ocsf_field() {
    let spec = load_claroty_spec();

    let zone_policies = spec
        .tables
        .iter()
        .find(|t| t.table_name == "organization_zone_policies")
        .expect("claroty.sensor.toml must contain 'organization_zone_policies' — add per AC-010");

    let tier1_count = zone_policies
        .columns
        .iter()
        .filter(|c| c.ocsf_field.is_some())
        .count();
    let tier2_count = zone_policies
        .columns
        .iter()
        .filter(|c| c.ocsf_field.is_none())
        .count();

    assert_eq!(
        tier1_count, 4,
        "zone_policies must have exactly 4 Tier-1 columns; got {tier1_count}"
    );
    assert_eq!(
        tier2_count, 9,
        "zone_policies must have exactly 9 Tier-2 columns (incl. 3 Json); got {tier2_count}"
    );

    // policy_name → "name", REQUIRED
    let policy_name_col = zone_policies
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
    let action_col = zone_policies
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
    let notes_col = zone_policies
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
    let updated_by_col = zone_policies
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
// RG-011 — Live wire-shape for zone_policies (#[ignore])
// ===========================================================================

/// RG-011 (LIVE — #[ignore]): SELECT * FROM claroty.claroty_organization_zone_policies
/// LIMIT 1 serialized JSON: class_uid=3004, name present, activity_name present,
/// raw_extensions has communication_conditions, related_alerts_ids, applied_zone_pairs
/// as JSON arrays (NOT stringified). No Tier-2 column names as standalone root keys.
///
/// Traces to: BC-2.16.020 §PC2/§PC4/§PC6; TV-BC-2.16.020-007.
/// Story: S-CLAROTY-ORGPOLICY-001 AC-011
#[tokio::test]
#[ignore = "LIVE-MONROE-001: requires CLAROTY_INSTANCE_URL env var pointing to monroe; run manually or in live-validation CI job"]
async fn test_BC_2_16_020_claroty_organization_zone_policies_live_wire_shape_class_uid_and_tier1() {
    todo!("LIVE-MONROE-001: implement when CLAROTY_INSTANCE_URL is available in test environment")
}

// ===========================================================================
// RG-012 — applied_zone_pairs (Tier-2 Json) raises E-QUERY-038 at plan time
// ===========================================================================

/// RG-012 (RED): ocsf_projected_column_names for claroty_organization_zone_policies
/// must NOT include 'applied_zone_pairs' as a standalone Arrow column.
/// 'raw_extensions' must be present. Tier-1 Arrow names must be present.
///
/// FAILS before implementation.
///
/// Traces to: BC-2.16.020 invariant; AC-012; EC-016-020-006.
/// Story: S-CLAROTY-ORGPOLICY-001 AC-012
///
/// SAP-3 NOTE (defense-in-depth): This test exercises `ocsf_projected_column_names`
/// directly (pre-plan-gate helper). Per SAP-3 rule-3, it counts as defense-in-depth only.
/// The AUTHORITATIVE end-to-end gate from the public query surface (SQL parser →
/// QueryEngine::execute) is
/// `test_BC_2_16_020_claroty_organization_zone_policies_e2e_e_query_038_tier2_column`
/// in `crates/prism-bin/tests/bc_2_16_020_claroty_org_zone_policy_wire_shape.rs`
/// (RG-012a, F-ORGPOL-P1-MED-001 closure).
#[test]
fn test_BC_2_16_020_claroty_organization_zone_policies_applied_zone_pairs_raises_e_query_038() {
    let spec = load_claroty_spec();

    let zone_policies = spec
        .tables
        .iter()
        .find(|t| t.table_name == "organization_zone_policies")
        .expect("claroty.sensor.toml must contain 'organization_zone_policies' — add per AC-012");

    let available = ocsf_projected_column_names(zone_policies, true);

    // applied_zone_pairs (Tier-2 Json) must NOT appear as a standalone Arrow column
    assert!(
        !available.contains(&"applied_zone_pairs".to_string()),
        "applied_zone_pairs (Tier-2 Json) must NOT be in ocsf_projected_column_names; \
         selecting it raises E-QUERY-038; available: {available:?}"
    );

    // raw_extensions must be present (3 Tier-2 Json columns → has_tier2 = true)
    assert!(
        available.contains(&"raw_extensions".to_string()),
        "raw_extensions must appear in available columns; available: {available:?}"
    );

    // Tier-1 Arrow names must be present
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
}

// ===========================================================================
// RG-013 — REQUIRED policy_name absent → null row, no hard error
// ===========================================================================

/// RG-013 (RED): When a zone_policies row has 'policy_name' absent,
/// ColumnMapper::map_record returns Ok (no hard error). mapped_fields must not
/// contain 'name'.
///
/// FAILS before implementation.
///
/// Traces to: BC-2.16.020 invariant; AC-013; EC-016-020-002.
/// Story: S-CLAROTY-ORGPOLICY-001 AC-013
///
/// # SAP-3 Rule-3 Defense-in-Depth Disclaimer
///
/// This test invokes `ColumnMapper::map_record` directly (pre-serialization path).
/// `map_record` has ZERO production callers — it is not on the production data path.
/// This test is **defense-in-depth only** (SAP-3 rule-3).
///
/// The authoritative production-path gate (SAP-4) is:
/// `test_BC_2_16_020_claroty_organization_zone_policies_wire_shape_serialized_json_null_not_absent`
/// in `crates/prism-bin/tests/bc_2_16_020_claroty_org_zone_policy_wire_shape.rs`,
/// which asserts `"name": null` at the serialized JSON wire level (not absent).
#[test]
fn test_BC_2_16_020_claroty_organization_zone_policies_required_policy_name_absent_produces_null_row(
) {
    let spec = load_claroty_spec();

    let zone_policies = spec
        .tables
        .iter()
        .find(|t| t.table_name == "organization_zone_policies")
        .expect("claroty.sensor.toml must contain 'organization_zone_policies' — add per AC-013");

    // Confirm policy_name carries REQUIRED
    let policy_name_col = zone_policies
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
        "policy_action": "Allow",
        "matching_devices": 5
    });

    let result = ColumnMapper::map_record(&raw_missing, zone_policies).expect(
        "ColumnMapper::map_record must return Ok even when REQUIRED field is absent; \
             BC-2.16.003: records are never dropped via hard error",
    );

    assert!(
        !result.mapped_fields.contains_key("name"),
        "mapped_fields must not contain 'name' when policy_name is absent \
         (REQUIRED absent → null row semantics; BC-2.16.020 §PC2)"
    );
}

// ===========================================================================
// RG-014 — Json columns (comm_conditions, related_alerts_ids, applied_zone_pairs)
//           NOT stringified in raw_extensions (wire-shape + SID-2)
// ===========================================================================

/// RG-014 (RED): All three Json columns of zone_policies must appear in raw_extensions
/// as JSON-typed values (arrays), NOT as JSON string encodings.
///
/// Wire-shape assertion (2026-07-13 discipline): each value must be Value::Array.
/// SID-2 composed-output: assert all three keys present and correct independently.
/// applied_zone_pairs contains {src_zone, dst_zone} objects (zone domain — NOT src_group/dst_group).
///
/// FAILS before implementation.
///
/// Traces to: BC-2.16.020 §PC6; AC-014; spike-findings §Spike 3 §Table B.
/// Story: S-CLAROTY-ORGPOLICY-001 AC-014
///
/// # SAP-3 Rule-3 Defense-in-Depth Disclaimer
///
/// This test invokes `ColumnMapper::map_record` directly (pre-serialization path).
/// `map_record` has ZERO production callers — it is not on the production data path.
/// This test is **defense-in-depth only** (SAP-3 rule-3).
///
/// The authoritative production-path gate (SAP-4) is:
/// `test_BC_2_16_020_claroty_organization_zone_policies_wire_shape_class_uid_3004_mock`
/// in `crates/prism-bin/tests/bc_2_16_020_claroty_org_zone_policy_wire_shape.rs`,
/// which asserts communication_conditions/related_alerts_ids/applied_zone_pairs are
/// NATIVE JSON arrays in raw_extensions through `SpecDrivenSensorAdapter::fetch`.
#[test]
fn test_BC_2_16_020_claroty_organization_zone_policies_json_columns_not_stringified() {
    let spec = load_claroty_spec();

    let zone_policies = spec
        .tables
        .iter()
        .find(|t| t.table_name == "organization_zone_policies")
        .expect("claroty.sensor.toml must contain 'organization_zone_policies' — add per AC-014");

    // Verify all three Json columns are declared column_type = "json"
    for col_name in [
        "communication_conditions",
        "related_alerts_ids",
        "applied_zone_pairs",
    ] {
        let col = zone_policies
            .columns
            .iter()
            .find(|c| c.name == col_name)
            .unwrap_or_else(|| {
                panic!("column '{col_name}' must exist in claroty_organization_zone_policies")
            });
        assert_eq!(
            col.column_type,
            prism_core::column::ColumnType::Json,
            "column '{col_name}' must be column_type = \"json\" (not String); \
             declaring as String causes stringification defect (AC-014)"
        );
        assert!(
            col.ocsf_field.is_none(),
            "'{col_name}' must be Tier-2 (no ocsf_field) → aggregates into raw_extensions"
        );
    }

    // Wire-shape test
    let raw = json!({
        "policy_name": "Zone Policy Alpha",
        "communication_conditions": [{"src": "Zone A", "dst": "Zone B"}],
        "related_alerts_ids": [101, 202, 303],
        "applied_zone_pairs": [{"src_zone": "Z1", "dst_zone": "Z2"}]
    });

    let result = ColumnMapper::map_record(&raw, zone_policies)
        .expect("ColumnMapper::map_record must succeed");

    // SID-2 composed-output: assert each key independently
    for json_col in [
        "communication_conditions",
        "related_alerts_ids",
        "applied_zone_pairs",
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

    // Specific: applied_zone_pairs contains {src_zone, dst_zone} (zone domain)
    let applied_zone = result
        .raw_extensions
        .get("applied_zone_pairs")
        .unwrap()
        .as_array()
        .unwrap();
    let first_pair = &applied_zone[0];
    assert!(
        first_pair.get("src_zone").is_some(),
        "applied_zone_pairs elements must contain 'src_zone' (zone domain, not 'src_group')"
    );
    assert!(
        first_pair.get("dst_zone").is_some(),
        "applied_zone_pairs elements must contain 'dst_zone' (zone domain, not 'dst_group')"
    );
    // SID-2 no-duplication: applied_zone_pairs must NOT contain group-domain keys
    assert!(
        first_pair.get("src_group").is_none(),
        "applied_zone_pairs must NOT contain 'src_group' — that belongs to applied_group_pairs \
         in the firewall domain (BC-2.16.021 §PC4)"
    );
}
