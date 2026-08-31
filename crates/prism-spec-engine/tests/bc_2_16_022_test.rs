#![allow(non_snake_case, clippy::expect_used, clippy::unwrap_used)]
//! Red Gate tests for BC-2.16.022 — Claroty xDome Organization ACL Policies TOML table.
//!
//! Story: S-CLAROTY-ACLPOLICY-001
//! Tests: RG-001, RG-002, RG-003, RG-004 (unit); RG-007, RG-010 (#[ignore] live)
//!
//! # Red Gate mechanism
//!
//! Every non-#[ignore] test calls
//! `.find(|t| t.table_name == "claroty_organization_acl_policies").expect(...)`
//! on the loaded `claroty.sensor.toml`. Pre-implementation the table is absent →
//! `.expect()` panics → test FAILS with a descriptive message. Post-implementation
//! the table is present → tests proceed to behavioral assertions.
//!
//! CONTAMINATION CONTROL: this file MUST NOT read holdout scenario files (HS-029).

use std::fs;
use std::path::Path;

use prism_core::{ColumnOptions, ColumnType};
use prism_spec_engine::spec_parser::{PaginationConfig, SpecLoader};

// ---------------------------------------------------------------------------
// Fixture helpers
// ---------------------------------------------------------------------------

/// Absolute path to `claroty.sensor.toml` relative to this crate.
fn claroty_spec_path() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../prism-sensors/specs/claroty.sensor.toml")
}

/// Load and parse `claroty.sensor.toml` via pure TOML deserialization.
/// Uses `SpecLoader::parse` (no env var resolution needed for unit tests).
fn load_claroty_spec() -> prism_spec_engine::spec_parser::SensorSpec {
    let content = fs::read_to_string(claroty_spec_path())
        .expect("claroty.sensor.toml must be readable from prism-sensors/specs/");
    SpecLoader::parse(&content).expect("claroty.sensor.toml must be a valid SensorSpec TOML")
}

// ---------------------------------------------------------------------------
// RG-001 / AC-001 — TOML block structure, ocsf_class, 11 columns, step shape
// ---------------------------------------------------------------------------

/// BC-2.16.022 §PC1 — TOML block parses Ok; ocsf_class = "entity_management";
/// 11 columns; exactly 1 step with response_path = "$.organization_acl_policies";
/// body_template present (POST endpoint).
///
/// Red Gate: `.expect()` panics when table absent from claroty.sensor.toml.
#[test]
fn test_BC_2_16_022_claroty_org_acl_policies_toml_block_parses() {
    let spec = load_claroty_spec();

    // Red Gate: panics if `organization_acl_policies` bare-name entry is absent.
    // MED-1 correction: TOML uses bare table_name = "organization_acl_policies" (consistent
    // with sibling tables alerts/audit_logs/devices/device_alert_relations/vulnerabilities).
    // TableRegistry derives the registered name as claroty_organization_acl_policies.
    let table = spec
        .tables
        .iter()
        .find(|t| t.table_name == "organization_acl_policies")
        .expect(
            "BC-2.16.022 AC-001 RED GATE: claroty.sensor.toml MUST contain a [[tables]] block \
             with table_name = \"organization_acl_policies\" (bare, consistent with sibling \
             tables). TableRegistry derives claroty_organization_acl_policies. \
             Implementer: add the TOML block from S-CLAROTY-ACLPOLICY-001 \
             §TOML Column-Block Specification.",
        );

    // BC-2.16.022 §PC1: ocsf_class must be "entity_management" (class_uid 3004).
    assert_eq!(
        table.ocsf_class, "entity_management",
        "BC-2.16.022 AC-001: ocsf_class MUST be 'entity_management' (class_uid 3004); \
         got: '{}'",
        table.ocsf_class
    );

    // BC-2.16.022 §PC2: 11 declared columns (4 Tier-1 + 7 Tier-2).
    assert_eq!(
        table.columns.len(),
        11,
        "BC-2.16.022 AC-001: claroty_organization_acl_policies MUST have exactly 11 columns \
         (4 Tier-1 with ocsf_field + 7 Tier-2 without). Got: {}",
        table.columns.len()
    );

    // BC-2.16.022 §PC1: exactly one step.
    assert_eq!(
        table.steps.len(),
        1,
        "BC-2.16.022 AC-001: claroty_organization_acl_policies MUST have exactly 1 step; \
         got: {}",
        table.steps.len()
    );

    let step = &table.steps[0];

    // BC-2.16.022 §PC1: response_path must be "$.organization_acl_policies".
    assert_eq!(
        step.response_path, "$.organization_acl_policies",
        "BC-2.16.022 AC-001: response_path MUST be '$.organization_acl_policies' \
         (xDome API response envelope contract). Got: '{}'",
        step.response_path
    );

    // BC-2.16.022 §PC1: body_template must be Some (POST endpoint requires a JSON body).
    assert!(
        step.body_template.is_some(),
        "BC-2.16.022 AC-001: body_template MUST be Some(...) for the POST \
         /api/v1/organization_acl_policies/ endpoint — 'policy_acl_syntax' is REQUIRED \
         in GetOrganizationAclPoliciesRequest. Got: None"
    );
}

// ---------------------------------------------------------------------------
// RG-002 / AC-002 — PaginationConfig::None; body_template has no offset/limit
// ---------------------------------------------------------------------------

/// BC-2.16.022 §PC4 — step.pagination == Some(PaginationConfig::None);
/// body_template has no "offset" or "limit" keys.
///
/// This is the ONLY Claroty table with PaginationConfig::None (single-fetch).
/// Injecting offset/limit would cause the Claroty API to return 422
/// (GetOrganizationAclPoliciesRequest has no offset/limit fields).
///
/// Red Gate: `.expect()` panics when table absent from claroty.sensor.toml.
#[test]
fn test_BC_2_16_022_claroty_org_acl_policies_pagination_none_no_offset_limit() {
    let spec = load_claroty_spec();

    let table = spec
        .tables
        .iter()
        .find(|t| t.table_name == "organization_acl_policies")
        .expect(
            "BC-2.16.022 AC-002 RED GATE: claroty_organization_acl_policies must exist in \
             claroty.sensor.toml to test PaginationConfig::None contract.",
        );

    let step = table
        .steps
        .first()
        .expect("BC-2.16.022 AC-002: table must have at least one step");

    // BC-2.16.022 §PC4: pagination MUST be PaginationConfig::None.
    assert_eq!(
        step.pagination,
        Some(PaginationConfig::None),
        "BC-2.16.022 AC-002: step pagination MUST be Some(PaginationConfig::None) \
         (TOML: type = \"none\"). This is the ONLY Claroty table with non-paginated \
         single-fetch. Injecting offset/limit causes 422 from the Claroty API. \
         Got: {:?}",
        step.pagination
    );

    // BC-2.16.022 §PC4: body_template must NOT contain "offset" or "limit" keys.
    // PaginationConfig::None pipeline path does NOT inject these fields.
    let body_template_str = step
        .body_template
        .as_deref()
        .expect("BC-2.16.022 AC-002: body_template must be present for POST step");

    let body: serde_json::Value = serde_json::from_str(body_template_str)
        .expect("BC-2.16.022 AC-002: body_template must be valid JSON");

    assert!(
        body.get("offset").is_none(),
        "BC-2.16.022 AC-002: body_template MUST NOT contain 'offset' key — \
         PaginationConfig::None; injecting offset causes 422 from Claroty API. \
         body_template: {}",
        body_template_str
    );
    assert!(
        body.get("limit").is_none(),
        "BC-2.16.022 AC-002: body_template MUST NOT contain 'limit' key — \
         PaginationConfig::None; injecting limit causes 422 from Claroty API. \
         body_template: {}",
        body_template_str
    );
}

// ---------------------------------------------------------------------------
// RG-003 / AC-003 — body_template has policy_acl_syntax = "Cisco dACL" and
//                    fields array with all 11 API column names
// ---------------------------------------------------------------------------

/// BC-2.16.022 §PC1 — body_template JSON has "policy_acl_syntax" = "Cisco dACL"
/// and a "fields" array containing all 11 OrganizationAclPolicyResponseItem field names.
///
/// "policy_acl_syntax" is a REQUIRED field in GetOrganizationAclPoliciesRequest
/// (OpenAPI schema: required: ["policy_acl_syntax"]). Omitting it causes 422.
///
/// Red Gate: `.expect()` panics when table absent from claroty.sensor.toml.
#[test]
fn test_BC_2_16_022_claroty_org_acl_policies_body_template_has_policy_acl_syntax() {
    let spec = load_claroty_spec();

    let table = spec
        .tables
        .iter()
        .find(|t| t.table_name == "organization_acl_policies")
        .expect(
            "BC-2.16.022 AC-003 RED GATE: claroty_organization_acl_policies must exist in \
             claroty.sensor.toml to test body_template policy_acl_syntax contract.",
        );

    let step = table
        .steps
        .first()
        .expect("BC-2.16.022 AC-003: table must have at least one step");

    let body_template_str = step
        .body_template
        .as_deref()
        .expect("BC-2.16.022 AC-003: body_template must be present");

    let body: serde_json::Value = serde_json::from_str(body_template_str)
        .expect("BC-2.16.022 AC-003: body_template must be valid JSON");

    // BC-2.16.022 §PC1: "policy_acl_syntax" key must be present with value "Cisco dACL".
    let syntax_val = body.get("policy_acl_syntax").expect(
        "BC-2.16.022 AC-003: body_template MUST contain 'policy_acl_syntax' key. \
         It is a REQUIRED parameter in GetOrganizationAclPoliciesRequest \
         (OpenAPI: required: [\"policy_acl_syntax\"]). Omitting it causes 422.",
    );
    assert_eq!(
        syntax_val, "Cisco dACL",
        "BC-2.16.022 AC-003: body_template 'policy_acl_syntax' MUST be 'Cisco dACL' \
         (hardcoded v1 default per spike findings §Spike 4). Got: '{}'",
        syntax_val
    );

    // BC-2.16.022 §PC1: "fields" array must contain all 11 API field names.
    let fields_val = body
        .get("fields")
        .expect("BC-2.16.022 AC-003: body_template MUST contain 'fields' array");
    let fields_arr = fields_val
        .as_array()
        .expect("BC-2.16.022 AC-003: body_template 'fields' must be a JSON array");
    let field_strings: Vec<&str> = fields_arr.iter().filter_map(|v| v.as_str()).collect();

    // All 11 API field names from OrganizationAclPolicyResponseItem.
    const EXPECTED_API_FIELDS: &[&str] = &[
        "policy_id",
        "policy_name",
        "policy_source",
        "applied_models",
        "matching_devices",
        "policy_acl_type",
        "policy_acl",
        "policy_creation_date",
        "policy_last_updated",
        "policy_updated_by",
        "policy_notes",
    ];
    for &expected in EXPECTED_API_FIELDS {
        assert!(
            field_strings.contains(&expected),
            "BC-2.16.022 AC-003: body_template 'fields' array MUST contain '{}'; \
             got: {:?}",
            expected,
            field_strings
        );
    }
    assert_eq!(
        fields_arr.len(),
        11,
        "BC-2.16.022 AC-003: body_template 'fields' array MUST have exactly 11 entries; \
         got: {}. fields: {:?}",
        fields_arr.len(),
        field_strings
    );
}

// ---------------------------------------------------------------------------
// RG-004 / AC-004 — Tier-1 (4) and Tier-2 (7) classification; column types
// ---------------------------------------------------------------------------

/// BC-2.16.022 §PC2/§PC3 — 4 Tier-1 columns (ocsf_field set), 7 Tier-2 (ocsf_field absent).
///
/// Tier-1: policy_id→metadata.uid (REQUIRED, String), policy_name→name (String),
///         policy_updated_by→actor.user.name (String), policy_notes→comment (String).
/// Tier-2: policy_source (String), policy_acl_type (String), policy_acl (String),
///         applied_models (Json — P1 CRITICAL if String), matching_devices (Integer),
///         policy_creation_date (Datetime, no timestamp_formats → implicit iso8601),
///         policy_last_updated (Datetime, no timestamp_formats → implicit iso8601).
///
/// Red Gate: `.expect()` panics when table absent from claroty.sensor.toml.
#[test]
fn test_BC_2_16_022_claroty_org_acl_policies_tier1_four_tier2_seven_correct_types() {
    let spec = load_claroty_spec();

    let table = spec
        .tables
        .iter()
        .find(|t| t.table_name == "organization_acl_policies")
        .expect(
            "BC-2.16.022 AC-004 RED GATE: claroty_organization_acl_policies must exist in \
             claroty.sensor.toml to test Tier-1/Tier-2 column classification.",
        );

    let tier1_cols: Vec<_> = table
        .columns
        .iter()
        .filter(|col| col.ocsf_field.is_some())
        .collect();
    let tier2_cols: Vec<_> = table
        .columns
        .iter()
        .filter(|col| col.ocsf_field.is_none())
        .collect();

    // BC-2.16.022 §PC2: exactly 4 Tier-1 and 7 Tier-2 columns.
    assert_eq!(
        tier1_cols.len(),
        4,
        "BC-2.16.022 AC-004: MUST have exactly 4 Tier-1 columns (ocsf_field set). \
         Got: {}. Tier-1 names: {:?}",
        tier1_cols.len(),
        tier1_cols.iter().map(|c| &c.name).collect::<Vec<_>>()
    );
    assert_eq!(
        tier2_cols.len(),
        7,
        "BC-2.16.022 AC-004: MUST have exactly 7 Tier-2 columns (ocsf_field absent). \
         Got: {}. Tier-2 names: {:?}",
        tier2_cols.len(),
        tier2_cols.iter().map(|c| &c.name).collect::<Vec<_>>()
    );

    // BC-2.16.022 §PC3: policy_id → ocsf_field = "metadata.uid", REQUIRED, String.
    let policy_id_col = table
        .columns
        .iter()
        .find(|col| col.name == "policy_id")
        .expect("BC-2.16.022 AC-004: 'policy_id' column must exist");
    assert_eq!(
        policy_id_col.ocsf_field.as_deref(),
        Some("metadata.uid"),
        "BC-2.16.022 AC-004: policy_id MUST map to ocsf_field 'metadata.uid' \
         (Arrow projection: metadata_uid via dot→underscore flattening). Got: {:?}",
        policy_id_col.ocsf_field
    );
    assert_eq!(
        policy_id_col.column_type,
        ColumnType::String,
        "BC-2.16.022 AC-004: policy_id MUST be column_type String. Got: {:?}",
        policy_id_col.column_type
    );
    assert!(
        policy_id_col.options.contains(&ColumnOptions::Required),
        "BC-2.16.022 AC-004: policy_id MUST have ColumnOptions::Required \
         (TOML: options = [\"REQUIRED\"]). Got: {:?}",
        policy_id_col.options
    );

    // BC-2.16.022 §PC2: policy_name → ocsf_field = "name".
    let policy_name_col = table
        .columns
        .iter()
        .find(|col| col.name == "policy_name")
        .expect("BC-2.16.022 AC-004: 'policy_name' column must exist");
    assert_eq!(
        policy_name_col.ocsf_field.as_deref(),
        Some("name"),
        "BC-2.16.022 AC-004: policy_name MUST map to ocsf_field 'name'. Got: {:?}",
        policy_name_col.ocsf_field
    );

    // BC-2.16.022 §PC2: policy_updated_by → ocsf_field = "actor.user.name".
    let policy_updated_by_col = table
        .columns
        .iter()
        .find(|col| col.name == "policy_updated_by")
        .expect("BC-2.16.022 AC-004: 'policy_updated_by' column must exist");
    assert_eq!(
        policy_updated_by_col.ocsf_field.as_deref(),
        Some("actor.user.name"),
        "BC-2.16.022 AC-004: policy_updated_by MUST map to ocsf_field 'actor.user.name'. \
         Got: {:?}",
        policy_updated_by_col.ocsf_field
    );

    // BC-2.16.022 §PC2: policy_notes → ocsf_field = "comment".
    let policy_notes_col = table
        .columns
        .iter()
        .find(|col| col.name == "policy_notes")
        .expect("BC-2.16.022 AC-004: 'policy_notes' column must exist");
    assert_eq!(
        policy_notes_col.ocsf_field.as_deref(),
        Some("comment"),
        "BC-2.16.022 AC-004: policy_notes MUST map to ocsf_field 'comment'. Got: {:?}",
        policy_notes_col.ocsf_field
    );

    // BC-2.16.022 §PC5: applied_models MUST be column_type Json (P1 CRITICAL if String).
    // Declaring it as String causes the array to be serialized as a raw JSON-string token
    // in raw_extensions — not a native JSON array (raw_extensions array-stringification bug).
    let applied_models_col = table
        .columns
        .iter()
        .find(|col| col.name == "applied_models")
        .expect("BC-2.16.022 AC-004: 'applied_models' column must exist");
    assert_eq!(
        applied_models_col.column_type,
        ColumnType::Json,
        "BC-2.16.022 AC-004 P1: applied_models MUST be column_type Json (NOT String). \
         A String declaration causes the array to be stringified in raw_extensions \
         (ENRICH-1 DD-2 bug). Got: {:?}",
        applied_models_col.column_type
    );
    assert!(
        applied_models_col.ocsf_field.is_none(),
        "BC-2.16.022 AC-004: applied_models MUST be Tier-2 (ocsf_field == None). \
         Got: {:?}",
        applied_models_col.ocsf_field
    );

    // BC-2.16.022 §PC2 / ADR-028 §D8-B: datetime Tier-2 columns with NO timestamp_formats
    // (implicit iso8601 default via effective_formats returning [\"iso8601\"]).
    for dt_col_name in &["policy_creation_date", "policy_last_updated"] {
        let dt_col = table
            .columns
            .iter()
            .find(|col| col.name == *dt_col_name)
            .expect(&format!(
                "BC-2.16.022 AC-004: '{}' column must exist",
                dt_col_name
            ));
        assert_eq!(
            dt_col.column_type,
            ColumnType::Datetime,
            "BC-2.16.022 AC-004: '{}' MUST be column_type Datetime. Got: {:?}",
            dt_col_name,
            dt_col.column_type
        );
        assert!(
            dt_col.ocsf_field.is_none(),
            "BC-2.16.022 AC-004: '{}' MUST be Tier-2 (ocsf_field == None). Got: {:?}",
            dt_col_name,
            dt_col.ocsf_field
        );
        assert!(
            dt_col.timestamp_formats.is_empty(),
            "BC-2.16.022 AC-004 (ADR-028 §D8-B): '{}' MUST have empty timestamp_formats \
             (implicit iso8601 default — effective_formats returns [\"iso8601\"]). \
             Got: {:?}",
            dt_col_name,
            dt_col.timestamp_formats
        );
    }
}

// ---------------------------------------------------------------------------
// RG-007 / AC-007 — Live Variant-1 wire-shape test (#[ignore])
// ---------------------------------------------------------------------------

/// BC-2.16.022 §TV-BC-2.16.022-001 — SELECT * LIMIT 1 wire JSON must contain:
/// class_uid=3004, metadata_uid present non-null (UUID), raw_extensions present,
/// raw_extensions["applied_models"] is a JSON array (NOT a "\"[...]\"" string),
/// "policy_id" NOT a standalone root key.
///
/// Wire-shape assertion discipline (2026-07-13): assert on serialized JSON output.
///
/// #[ignore] — LIVE-MONROE-001: requires CLAROTY_INSTANCE_URL env var pointing to
/// monroe live sensor. Run manually or in live-validation CI job.
#[tokio::test]
#[ignore = "LIVE-MONROE-001: requires CLAROTY_INSTANCE_URL env var pointing to live \
             monroe sensor; run manually or in live-validation CI job"]
async fn test_BC_2_16_022_claroty_org_acl_policies_live_wire_shape_class_uid_and_metadata_uid() {
    // Structural gate: table must exist even for live tests.
    let spec = load_claroty_spec();
    let _table = spec
        .tables
        .iter()
        .find(|t| t.table_name == "organization_acl_policies")
        .expect(
            "BC-2.16.022 RG-007: claroty_organization_acl_policies must be in \
             claroty.sensor.toml before running live wire-shape test",
        );

    // TODO(LIVE-MONROE-001): implement live QueryEngine call against monroe sensor.
    // The live test MUST assert (wire-shape discipline, 2026-07-13):
    //   1. class_uid == 3004 in serialized JSON row
    //   2. metadata_uid present and non-null UUID string
    //   3. raw_extensions present as JSON object
    //   4. raw_extensions["applied_models"] is a JSON array (NOT "\"[...]\"" string)
    //   5. "policy_id" NOT a standalone top-level key in row JSON
    todo!(
        "LIVE-MONROE-001: live wire-shape test; \
         ungated after monroe sensor accessible and S-CLAROTY-ACLPOLICY-001 merged"
    );
}

// ---------------------------------------------------------------------------
// RG-010 / AC-010 — Live unbounded SELECT * (no LIMIT), no pagination loop (#[ignore])
// ---------------------------------------------------------------------------

/// BC-2.16.022 §TV-BC-2.16.022-006 — SELECT * FROM claroty.claroty_organization_acl_policies
/// (no LIMIT) must succeed without triggering a second-page loop (PaginationConfig::None).
/// Wire output must NOT contain a "count" key (no count field in xDome response envelope).
///
/// #[ignore] — LIVE-MONROE-001: requires CLAROTY_INSTANCE_URL env var.
#[tokio::test]
#[ignore = "LIVE-MONROE-001: requires CLAROTY_INSTANCE_URL env var pointing to live \
             monroe sensor; run manually or in live-validation CI job"]
async fn test_BC_2_16_022_claroty_org_acl_policies_live_unbounded_select_no_pagination() {
    let spec = load_claroty_spec();
    let _table = spec
        .tables
        .iter()
        .find(|t| t.table_name == "organization_acl_policies")
        .expect(
            "BC-2.16.022 RG-010: claroty_organization_acl_policies must be in \
             claroty.sensor.toml before running live unbounded-select test",
        );

    // TODO(LIVE-MONROE-001): implement live QueryEngine call — SELECT * without LIMIT.
    // The live test MUST assert (wire-shape discipline, 2026-07-13):
    //   1. Query succeeds (no E-SENSOR-001)
    //   2. Wire JSON rows do NOT have a "count" key
    //   3. Running the same query twice returns the same row count (no second-page loop)
    todo!(
        "LIVE-MONROE-001: live unbounded-select test; \
         ungated after monroe sensor accessible and S-CLAROTY-ACLPOLICY-001 merged"
    );
}
