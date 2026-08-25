//! Red Gate test suite for BC-2.16.015 — Claroty xDome Vulnerability Findings Table.
//!
//! Covers S-CLAROTY-VULNS-001 acceptance criteria AC-001..AC-008.
//! BC-5.38.001 density check: 8 RGTs / 8 ACs = 1.0 (≥ 0.5 threshold).
//!
//! ## Red Gate invariant
//!
//! ALL non-`#[ignore]` tests (RG-001..003, RG-006..008) MUST FAIL before implementation
//! lands:
//!   - Each test finds the `claroty_vulnerabilities` table via `.find()` and panics at
//!     `.expect("claroty_vulnerabilities table must exist")` because the `[[tables]]`
//!     block has not yet been added to `claroty.sensor.toml`.
//!
//! ## SAP-3 compliance (RG-003)
//!
//! The E-QUERY-038 gate is enforced by `check_column_availability` in prism-query, which
//! delegates to `ocsf_projected_column_names`. Using `ocsf_projected_column_names` directly
//! from prism-spec-engine is the architecturally correct proxy: prism-sensors CANNOT depend
//! on prism-query (prism-query depends on prism-sensors in production — circular dependency).
//! Both `TableRegistry::register_sensor` and `check_column_availability` delegate to the same
//! canonical function per ADR-058 §I7, so this test is architecturally equivalent to an
//! end-to-end E-QUERY-038 assertion.
//!
//! ## SID-1 compliance (RG-004, RG-005)
//!
//! RG-004 and RG-005 are `#[ignore]`'d because they require a live Claroty DTU instance.
//! Non-ignored coverage (RG-001..003, RG-006..008) satisfies SID-1.

#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::collections::HashMap;

use prism_core::OrgSlug;
use prism_spec_engine::{
    column_mapping::{ocsf_projected_column_names, ColumnMapper},
    pipeline::{FetchContext, PipelineExecutor},
    spec_parser::SpecLoader,
    NullAuthProvider,
};
use serde_json::json;
use wiremock::{matchers::method, Mock, MockServer, ResponseTemplate};

const CLAROTY_TOML: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/specs/claroty.sensor.toml"
));

// ── RG-001 ────────────────────────────────────────────────────────────────────
/// BC-2.16.015 §Precondition P1: `[[tables]]` block with `table_name = "claroty_vulnerabilities"`
/// must parse without error and appear in the SensorSpec tables list.
/// Also asserts `ocsf_column_naming = true` on the claroty sensor (AC-001 / ADR-058 §D2).
///
/// RED: panics at the `.expect` because the TOML block has not been added yet.
#[test]
fn test_BC_2_16_015_claroty_vulnerabilities_toml_block_parses() {
    let spec = SpecLoader::parse(CLAROTY_TOML).expect("claroty.sensor.toml must parse");

    let table = spec
        .tables
        .iter()
        .find(|t| t.table_name == "claroty_vulnerabilities")
        .expect("claroty_vulnerabilities table must exist");

    assert_eq!(
        table.table_name, "claroty_vulnerabilities",
        "table_name must be 'claroty_vulnerabilities'"
    );
    assert!(
        spec.ocsf_column_naming,
        "claroty sensor must carry ocsf_column_naming = true (ADR-058 §D2)"
    );
}

// ── RG-002 ────────────────────────────────────────────────────────────────────
/// BC-2.16.015 §Postcondition: exactly 2 Tier-1 columns (`ocsf_field` present):
///   - `name`        → `ocsf_field = "finding_info.title"` (REQUIRED)
///   - `description` → `ocsf_field = "message"`
///
/// RED: panics at the `.expect` because the TOML block has not been added yet.
#[test]
fn test_BC_2_16_015_claroty_vulnerabilities_tier1_columns_two_with_ocsf_field() {
    let spec = SpecLoader::parse(CLAROTY_TOML).expect("claroty.sensor.toml must parse");

    let table = spec
        .tables
        .iter()
        .find(|t| t.table_name == "claroty_vulnerabilities")
        .expect("claroty_vulnerabilities table must exist");

    let tier1: Vec<_> = table
        .columns
        .iter()
        .filter(|c| c.ocsf_field.is_some())
        .collect();

    assert_eq!(
        tier1.len(),
        2,
        "expected exactly 2 Tier-1 columns with ocsf_field; got {}: {:?}",
        tier1.len(),
        tier1.iter().map(|c| &c.name).collect::<Vec<_>>()
    );

    let name_col = tier1
        .iter()
        .find(|c| c.name == "name")
        .expect("Tier-1 column 'name' must exist");
    assert_eq!(
        name_col.ocsf_field.as_deref(),
        Some("finding_info.title"),
        "column 'name' must declare ocsf_field = \"finding_info.title\""
    );

    let desc_col = tier1
        .iter()
        .find(|c| c.name == "description")
        .expect("Tier-1 column 'description' must exist");
    assert_eq!(
        desc_col.ocsf_field.as_deref(),
        Some("message"),
        "column 'description' must declare ocsf_field = \"message\""
    );
}

// ── RG-003 ────────────────────────────────────────────────────────────────────
/// BC-2.16.015 §Postcondition: Tier-2 column raw names are NOT in the OCSF-projected
/// column set — querying them directly would raise E-QUERY-038 (`PrismError::ColumnNotFound`).
/// `raw_extensions` MUST appear in the projected set (ADR-058 §J6: emitted when ≥1 Tier-2
/// column exists).
///
/// SAP-3: `ocsf_projected_column_names` is the canonical function used by both
/// `check_column_availability` (E-QUERY-038 gate) and `TableRegistry::register_sensor`
/// per ADR-058 §I7.  Direct use here is the only architecturally valid proxy for the
/// E-QUERY-038 assertion from prism-sensors tests (circular dep constraint).
///
/// RED: panics at the `.expect` because the TOML block has not been added yet.
#[test]
fn test_BC_2_16_015_claroty_vulnerabilities_tier2_column_raises_e_query_038() {
    let spec = SpecLoader::parse(CLAROTY_TOML).expect("claroty.sensor.toml must parse");

    let table = spec
        .tables
        .iter()
        .find(|t| t.table_name == "claroty_vulnerabilities")
        .expect("claroty_vulnerabilities table must exist");

    let projected = ocsf_projected_column_names(table, spec.ocsf_column_naming);

    // Representative Tier-2 column names must NOT appear in the projected set.
    let tier2_samples = [
        "vulnerability_type",
        "cve_ids",
        "cvss_v3_score",
        "source_url",
        "epss_score",
    ];
    for col_name in &tier2_samples {
        assert!(
            !projected.contains(&col_name.to_string()),
            "Tier-2 column '{}' must not be in projected columns \
             (E-QUERY-038 would fire if queried directly); projected set: {:?}",
            col_name,
            projected
        );
    }

    // raw_extensions must be present (ADR-058 §J6: emitted for tables with ≥1 Tier-2 column).
    assert!(
        projected.contains(&"raw_extensions".to_string()),
        "projected columns must include 'raw_extensions' for Tier-2 aggregation; got: {:?}",
        projected
    );

    // Tier-1 OCSF Arrow names must be present.
    assert!(
        projected.contains(&"finding_info_title".to_string()),
        "projected columns must include 'finding_info_title' \
         (name → ocsf_field finding_info.title → Arrow name finding_info_title)"
    );
    assert!(
        projected.contains(&"message".to_string()),
        "projected columns must include 'message' (description → ocsf_field message)"
    );
}

// ── RG-004 ────────────────────────────────────────────────────────────────────
/// LIVE: wire shape contains OCSF Tier-1 columns (`finding_info_title` / `message`)
/// in every mapped record.
///
/// Requires a live Claroty xDome instance reachable via `CLAROTY_INSTANCE_URL`.
///
/// SID-1 compliance: live dependency; non-ignored coverage for this AC is provided
/// by RG-001..003 and RG-006..008. This test is defence-in-depth only.
/// Blocked by: DTU-VULNS-001 — requires `CLAROTY_INSTANCE_URL` env var and the
/// `claroty_vulnerabilities` DTU route; ungated in CI after S-CLAROTY-VULNS-001 merges.
#[ignore]
#[tokio::test]
async fn test_BC_2_16_015_claroty_vulnerabilities_live_wire_shape_class_uid_and_tier1() {
    // DTU-VULNS-001: requires CLAROTY_INSTANCE_URL env var pointing to a live Claroty
    // xDome instance or DTU clone; ungated in CI after S-CLAROTY-VULNS-001 merges.
    let instance_url = std::env::var("CLAROTY_INSTANCE_URL")
        .expect("CLAROTY_INSTANCE_URL must be set for this live test");

    let spec = SpecLoader::parse(CLAROTY_TOML).expect("claroty.sensor.toml must parse");

    let mut live_spec = spec.clone();
    live_spec.base_url = instance_url;

    let live_table = live_spec
        .tables
        .iter()
        .find(|t| t.table_name == "claroty_vulnerabilities")
        .expect("claroty_vulnerabilities table must exist");

    let context = FetchContext::new(OrgSlug::new("live-test"), HashMap::new());
    let http_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("http client must build");
    let auth = NullAuthProvider;

    let result = PipelineExecutor::execute(&live_spec, live_table, &context, &http_client, &auth)
        .await
        .expect("live pipeline execution must succeed");

    // Every mapped record must expose at least one Tier-1 OCSF Arrow field.
    for raw_record in result.records.iter().take(5) {
        // Use the original table (from `spec`, same column schema) for mapping.
        let orig_table = spec
            .tables
            .iter()
            .find(|t| t.table_name == "claroty_vulnerabilities")
            .expect("claroty_vulnerabilities table must exist in original spec");
        let row = ColumnMapper::map_record(raw_record, orig_table)
            .expect("map_record must succeed for live record");
        assert!(
            row.mapped_fields.contains_key("finding_info_title")
                || row.mapped_fields.contains_key("message"),
            "live record must contain at least one Tier-1 OCSF field; \
             mapped_fields keys: {:?}",
            row.mapped_fields.keys().collect::<Vec<_>>()
        );
    }
}

// ── RG-005 ────────────────────────────────────────────────────────────────────
/// LIVE: `raw_extensions` is populated with Tier-2 field data in each mapped record.
///
/// Requires a live Claroty xDome instance reachable via `CLAROTY_INSTANCE_URL`.
///
/// SID-1 compliance: live dependency; non-ignored coverage for this AC is provided
/// by RG-001..003 and RG-006..008. This test is defence-in-depth only.
/// Blocked by: DTU-VULNS-001 — requires `CLAROTY_INSTANCE_URL` env var and the
/// `claroty_vulnerabilities` DTU route; ungated in CI after S-CLAROTY-VULNS-001 merges.
#[ignore]
#[tokio::test]
async fn test_BC_2_16_015_claroty_vulnerabilities_live_raw_extensions_contains_tier2_keys() {
    // DTU-VULNS-001: requires CLAROTY_INSTANCE_URL env var pointing to a live Claroty
    // xDome instance or DTU clone; ungated in CI after S-CLAROTY-VULNS-001 merges.
    let instance_url = std::env::var("CLAROTY_INSTANCE_URL")
        .expect("CLAROTY_INSTANCE_URL must be set for this live test");

    let spec = SpecLoader::parse(CLAROTY_TOML).expect("claroty.sensor.toml must parse");

    let mut live_spec = spec.clone();
    live_spec.base_url = instance_url;

    let live_table = live_spec
        .tables
        .iter()
        .find(|t| t.table_name == "claroty_vulnerabilities")
        .expect("claroty_vulnerabilities table must exist");

    let context = FetchContext::new(OrgSlug::new("live-test"), HashMap::new());
    let http_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("http client must build");
    let auth = NullAuthProvider;

    let result = PipelineExecutor::execute(&live_spec, live_table, &context, &http_client, &auth)
        .await
        .expect("live pipeline execution must succeed");

    // Each non-empty record should yield a non-empty raw_extensions map.
    for raw_record in result.records.iter().take(3) {
        let orig_table = spec
            .tables
            .iter()
            .find(|t| t.table_name == "claroty_vulnerabilities")
            .expect("claroty_vulnerabilities table must exist in original spec");
        let row = ColumnMapper::map_record(raw_record, orig_table)
            .expect("map_record must succeed for live record");
        assert!(
            !row.raw_extensions.is_empty(),
            "live record must have non-empty raw_extensions (Tier-2 data); \
             record: {:?}",
            raw_record
        );
    }
}

// ── RG-006 ────────────────────────────────────────────────────────────────────
/// BC-2.16.015 §EC-016-015-002: `ColumnOptions::Required` on `name` is push-down
/// eligibility ONLY — NOT an extraction null/error gate.
/// When the raw record lacks `name`, `ColumnMapper::map_record` silently skips it;
/// `finding_info_title` is absent from `mapped_fields` (null passthrough, no panic/error).
///
/// PRECISION NOTE: `ColumnOptions::Required` controls whether the column is eligible
/// for index-based push-down filtering. It does NOT cause the mapper to error or emit
/// a null sentinel when the field is absent. The mapper skips absent fields silently.
///
/// RED: panics at the `.expect` because the TOML block has not been added yet.
#[test]
fn test_BC_2_16_015_claroty_vulnerabilities_required_name_absent_produces_null_row() {
    let spec = SpecLoader::parse(CLAROTY_TOML).expect("claroty.sensor.toml must parse");

    let table = spec
        .tables
        .iter()
        .find(|t| t.table_name == "claroty_vulnerabilities")
        .expect("claroty_vulnerabilities table must exist");

    // Record that intentionally omits the REQUIRED 'name' field.
    let record_without_name = json!({
        "description": "A vulnerability with no name field",
        "vulnerability_type": "CVE",
        "cvss_v3_score": 7.5_f64
    });

    let row = ColumnMapper::map_record(&record_without_name, table).expect(
        "map_record must not return Err when a REQUIRED column is absent \
                 (Required = push-down eligibility, not extraction gate)",
    );

    // PRECISION: absent 'name' → 'finding_info_title' is simply absent from mapped_fields.
    assert!(
        !row.mapped_fields.contains_key("finding_info_title"),
        "finding_info_title must be absent (not errored) when 'name' is missing; \
         mapped_fields: {:?}",
        row.mapped_fields
    );

    // 'description' → 'message' must still map correctly (independent Tier-1 column).
    assert!(
        row.mapped_fields.contains_key("message"),
        "'message' (from 'description') must map correctly even when 'name' is absent"
    );
}

// ── RG-007 ────────────────────────────────────────────────────────────────────
/// BC-2.16.015 §EC-016-015-005: When the API response delivers an empty `vulnerabilities`
/// array (and `count: null`), the pipeline halts via the empty-page mechanism and returns
/// 0 records.  A null `count` field must not cause a panic or error.
///
/// Uses wiremock to simulate the Claroty DTU response shape
/// `{"vulnerabilities": [], "count": null, "total": 0, "page": 1}`.
///
/// RED: panics at the `.expect` because the TOML block has not been added yet.
#[tokio::test]
async fn test_BC_2_16_015_claroty_vulnerabilities_nullable_count_uses_empty_page_halt() {
    let mock_server = MockServer::start().await;

    // Mock any POST to the mock server with an empty vulnerabilities payload.
    // Shape mirrors DTU list_vulnerabilities handler (vulnerabilities.rs):
    //   {"vulnerabilities": [...], "total": N, "page": N}
    // "count": null is added to exercise the nullable-count halt path.
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "vulnerabilities": [],
            "count": serde_json::Value::Null,
            "total": 0_u32,
            "page": 1_u32
        })))
        .mount(&mock_server)
        .await;

    let spec = SpecLoader::parse(CLAROTY_TOML).expect("claroty.sensor.toml must parse");

    // Clone and redirect base_url to mock server so pipeline does not call real Claroty.
    let mut test_spec = spec.clone();
    test_spec.base_url = mock_server.uri();

    let table = test_spec
        .tables
        .iter()
        .find(|t| t.table_name == "claroty_vulnerabilities")
        .expect("claroty_vulnerabilities table must exist");

    let context = FetchContext::new(OrgSlug::new("test-org"), HashMap::new());
    let http_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("http client must build");
    let auth = NullAuthProvider;

    let result = PipelineExecutor::execute(&test_spec, table, &context, &http_client, &auth)
        .await
        .expect("pipeline must succeed on empty-page / null-count response");

    assert_eq!(
        result.records.len(),
        0,
        "empty vulnerabilities page must produce 0 records (empty-page halt); \
         got {} records",
        result.records.len()
    );
    assert_eq!(
        result.table_name, "claroty_vulnerabilities",
        "PipelineResult.table_name must match the table spec table_name"
    );
}

// ── RG-008 ────────────────────────────────────────────────────────────────────
/// BC-2.16.015 §EC-016-015-003: The `id` column uses `source_path = "$.id"`.
/// When the raw record lacks a root-level `id` key, `ColumnMapper::map_record`
/// skips the extraction silently (no error); `id` is absent from `raw_extensions`.
///
/// RED: panics at the `.expect` because the TOML block has not been added yet.
#[test]
fn test_BC_2_16_015_claroty_vulnerabilities_source_path_id_null_when_absent() {
    let spec = SpecLoader::parse(CLAROTY_TOML).expect("claroty.sensor.toml must parse");

    let table = spec
        .tables
        .iter()
        .find(|t| t.table_name == "claroty_vulnerabilities")
        .expect("claroty_vulnerabilities table must exist");

    // Record that omits the 'id' field (source_path = "$.id" finds nothing).
    let record_without_id = json!({
        "name": "CVE-2024-1234",
        "description": "Test vulnerability with no id field",
        "vulnerability_type": "CVE",
        "cvss_v3_score": 5.5_f64
    });

    let row = ColumnMapper::map_record(&record_without_id, table)
        .expect("map_record must not error when source_path '$.id' finds no match");

    // source_path extraction miss → column skipped; 'id' absent from raw_extensions.
    assert!(
        !row.raw_extensions.contains_key("id"),
        "'id' must be absent from raw_extensions when source_path '$.id' finds no match; \
         raw_extensions: {:?}",
        row.raw_extensions
    );

    // Tier-1 columns that ARE present must still map correctly.
    // map_record stores OCSF paths in DOT form (intermediate); arrow-name flattening
    // (finding_info_title) is downstream at pipeline_result_to_record_batch. Assert dot-form
    // here (cf. bc_2_16_003_test).
    assert!(
        row.mapped_fields.contains_key("finding_info.title"),
        "'name' (ocsf_field=finding_info.title) present in the map_record intermediate \
         when 'name' exists in the record; mapped_fields: {:?}",
        row.mapped_fields
    );
}
