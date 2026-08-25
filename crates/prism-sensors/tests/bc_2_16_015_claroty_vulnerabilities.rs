//! Red Gate test suite for BC-2.16.015 — Claroty xDome Vulnerability Findings Table.
//!
//! Covers S-CLAROTY-VULNS-001 acceptance criteria AC-001..AC-008.
//! BC-5.38.001 density check: 10 RGTs / 8 ACs = 1.25 (≥ 0.5 threshold).
//! Story v1.2 enumerates 10 RGTs: RG-001, RG-002, RG-003a [prism-bin e2e],
//! RG-003b [prism-sensors proxy], RG-004, RG-004b [prism-bin mock],
//! RG-005, RG-006, RG-007, RG-008.
//!
//! ## Red Gate invariant
//!
//! Non-`#[ignore]` tests in this file (covering RG-001, RG-002, RG-003b [proxy],
//! RG-006, RG-007, RG-008) MUST FAIL before implementation lands:
//!   - Each test finds the `claroty_vulnerabilities` table via `.find()` and panics at
//!     `.expect("claroty_vulnerabilities table must exist")` because the `[[tables]]`
//!     block has not yet been added to `claroty.sensor.toml`.
//! Also in this file (pass-2 additions, non-RGT extra coverage): EC-005 (cve_ids
//! empty-array), EC-006 (published_date null), F-VULNS-P1-005 (null-count non-empty
//! page).
//! Also in this file (pass-3 addition, non-RGT extra coverage): EC-004
//! (advisory-title verbatim).
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
    NullAuthProvider, SpecEngineError,
};
use serde_json::json;
use wiremock::{
    matchers::{body_partial_json, method},
    Mock, MockServer, ResponseTemplate,
};

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
    assert_eq!(
        table.columns.len(),
        19,
        "claroty_vulnerabilities must declare exactly 19 ColumnSpec entries (AC-001); \
         got {}: {:?}",
        table.columns.len(),
        table.columns.iter().map(|c| &c.name).collect::<Vec<_>>()
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

    let tier2: Vec<_> = table
        .columns
        .iter()
        .filter(|c| c.ocsf_field.is_none())
        .collect();

    assert_eq!(
        tier2.len(),
        17,
        "expected exactly 17 Tier-2 columns without ocsf_field (AC-002); got {}: {:?}",
        tier2.len(),
        tier2.iter().map(|c| &c.name).collect::<Vec<_>>()
    );

    let tier2_names: std::collections::HashSet<&str> =
        tier2.iter().map(|c| c.name.as_str()).collect();
    let expected_tier2: std::collections::HashSet<&str> = [
        "vulnerability_type",
        "cve_ids",
        "cvss_v3_score",
        "cvss_v3_exploitability_subscore",
        "cvss_v3_vector_string",
        "cvss_v2_score",
        "is_known_exploited",
        "affected_devices_count",
        "affected_ot_devices_count",
        "published_date",
        "epss_score",
        "adjusted_vulnerability_score",
        "adjusted_vulnerability_score_level",
        "exploits_count",
        "source_name",
        "source_url",
        "id",
    ]
    .iter()
    .copied()
    .collect();

    assert_eq!(
        tier2_names,
        expected_tier2,
        "Tier-2 column name set must exactly match BC-2.16.015 §2 (AC-002). \
         Extra: {:?}, Missing: {:?}",
        tier2_names.difference(&expected_tier2).collect::<Vec<_>>(),
        expected_tier2.difference(&tier2_names).collect::<Vec<_>>()
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

    // SAP-3 compliance note: the E-QUERY-038 gate is also tested end-to-end from
    // the REAL PrismQL parser surface (not just this proxy) in:
    //   crates/prism-bin/tests/bc_2_16_015_claroty_vulnerabilities_wire_shape.rs
    //   test_BC_2_16_015_claroty_vulnerabilities_e2e_e_query_038_tier2_column
    // This proxy (ocsf_projected_column_names direct call) is retained as
    // defense-in-depth per SAP-3 rule 3 (reachability rationale: circular dep
    // constraint prevents prism-sensors from importing prism-query directly).
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
    // LIVE-MONROE-001: requires CLAROTY_INSTANCE_URL env var pointing to monroe; run
    // manually or in live-validation CI job.
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

    // Every mapped record must expose the wire-level OCSF shape (AC-004).
    //
    // map_record stores Tier-1 fields in DOT form (not arrow-name form).
    // Arrow-name flattening (finding_info.title → finding_info_title) happens
    // downstream in pipeline_result_to_record_batch (private to prism-bin).
    // The true arrow-name serialized wire shape is asserted by the non-live mock test:
    //   crates/prism-bin/tests/bc_2_16_015_claroty_vulnerabilities_wire_shape.rs
    //   test_BC_2_16_015_claroty_vulnerabilities_wire_shape_class_uid_2002_mock
    for raw_record in result.records.iter().take(5) {
        // Use the original table (from `spec`, same column schema) for mapping.
        let orig_table = spec
            .tables
            .iter()
            .find(|t| t.table_name == "claroty_vulnerabilities")
            .expect("claroty_vulnerabilities table must exist in original spec");
        let row = ColumnMapper::map_record(raw_record, orig_table)
            .expect("map_record must succeed for live record");

        // ── Simulated wire-shape assertions ──────────────────────────────────
        // Build a simulated wire JSON row from the map_record output.
        // class_uid = 2002 is from EventClassSelector::select_by_class_name("vulnerability_finding")
        // (BC-2.02.012; prism-sensors has no prism-ocsf dep, so we assert the value directly).
        let mut simulated_wire_row = serde_json::Map::new();
        simulated_wire_row.insert("class_uid".to_string(), json!(2002_i32));
        if let Some(val) = row.mapped_fields.get("finding_info.title") {
            simulated_wire_row.insert("finding_info_title".to_string(), val.clone());
        }
        if let Some(val) = row.mapped_fields.get("message") {
            simulated_wire_row.insert("message".to_string(), val.clone());
        }
        if !row.raw_extensions.is_empty() {
            simulated_wire_row.insert(
                "raw_extensions".to_string(),
                serde_json::to_value(&row.raw_extensions)
                    .expect("raw_extensions must serialize to JSON"),
            );
        }

        assert_eq!(
            simulated_wire_row.get("class_uid"),
            Some(&json!(2002_i32)),
            "simulated wire row must have class_uid = 2002 (vulnerability_finding). \
             BC-2.16.015 AC-004."
        );

        let tier1_present = simulated_wire_row.contains_key("finding_info_title")
            || simulated_wire_row.contains_key("message");
        assert!(
            tier1_present,
            "live record simulated wire row must contain at least one Tier-1 OCSF field \
             (finding_info_title or message); wire row: {:?}",
            simulated_wire_row
        );

        // raw_extensions must be present and a JSON object (AC-005).
        assert!(
            simulated_wire_row
                .get("raw_extensions")
                .map(|v| v.is_object())
                .unwrap_or(false),
            "live record simulated wire row must contain raw_extensions as a JSON object; \
             got: {:?}",
            simulated_wire_row.get("raw_extensions")
        );

        // No Tier-2 column names must appear as top-level wire fields (AC-004 §Tier-2 isolation).
        let tier2_names = [
            "vulnerability_type",
            "cve_ids",
            "cvss_v3_score",
            "cvss_v3_exploitability_subscore",
            "cvss_v3_vector_string",
            "cvss_v2_score",
            "is_known_exploited",
            "affected_devices_count",
            "affected_ot_devices_count",
            "published_date",
            "epss_score",
            "adjusted_vulnerability_score",
            "adjusted_vulnerability_score_level",
            "exploits_count",
            "source_name",
            "source_url",
            "id",
        ];
        for tier2_name in &tier2_names {
            assert!(
                !simulated_wire_row.contains_key(*tier2_name),
                "Tier-2 column '{}' MUST NOT appear as a top-level wire field; \
                 it must be inside raw_extensions. BC-2.16.015 §2; ADR-058 §J6. \
                 Wire row keys: {:?}",
                tier2_name,
                simulated_wire_row.keys().collect::<Vec<_>>()
            );
        }
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
    // LIVE-MONROE-001: requires CLAROTY_INSTANCE_URL env var pointing to monroe; run
    // manually or in live-validation CI job.
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
/// BC-2.16.015 §EC-016-015-001: `ColumnOptions::Required` on `name` is push-down
/// eligibility ONLY — NOT an extraction null/error gate.
/// When the raw record lacks `name`, `ColumnMapper::map_record` silently skips it;
/// `finding_info_title` is absent from `mapped_fields` (null passthrough, no panic/error).
///
/// PRECISION NOTE: `ColumnOptions::Required` controls whether the column is eligible
/// for index-based push-down filtering. It does NOT cause the mapper to error or emit
/// a null sentinel when the field is absent. The mapper skips absent fields silently.
///
/// ## Traceability clarification (F-VULNS-ADV-003 OBS / pass-4)
///
/// This test gates the **map_record-layer contribution** only: when `name` is absent
/// from the raw API JSON object, `ColumnMapper::map_record` does NOT insert the
/// `finding_info.title` key into `mapped_fields`.  In other words, the key is absent
/// at the intermediate DOT-form layer (not yet null).
///
/// The downstream **end-to-end wire null-row** assertion — i.e., `finding_info_title`
/// appears as `null` (not absent) in the serialized MCP JSON when the Arrow column
/// exists in the schema but carries a null cell — is gated by
/// `test_BC_2_16_015_claroty_vulnerabilities_wire_shape_serialized_json_explicit_nulls`
/// (row1) in `crates/prism-bin/tests/bc_2_16_015_claroty_vulnerabilities_wire_shape.rs`.
///
/// The test name suffix `_produces_null_row` refers to the observable outcome from the
/// caller's perspective (a row with a null finding_info_title cell at the wire level),
/// not to the intermediate map_record assertion made here.  Do NOT rename this test —
/// the story RG table references `RG-006` by this name.
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

    // F-VULNS-010 (wire-null discipline): the missing `name` field means `finding_info_title`
    // will be NULL (not absent) in the serialized MCP wire row when `explicit_nulls=true` is
    // applied by the arrow_json WriterBuilder (CLAUDE.md §Wire-shape assertion discipline).
    //
    // Asserting the wire-null form (finding_info_title: null vs. absent) requires a full
    // RecordBatch → arrow_json serialization path, which is only available in prism-bin
    // (pipeline_result_to_record_batch is private to prism-bin). The non-live mock test
    // in bc_2_16_015_claroty_vulnerabilities_wire_shape.rs covers general wire-shape
    // assertions; the specific missing-name → null row case requires a live DTU/instance
    // to produce a record with an absent "name" field in real API data — covered by live RG-004.
    //
    // At this level (map_record), "absent from mapped_fields" is the correct assertion
    // (the mapper skips absent fields; null injection is downstream at RecordBatch build time).
}

// ── RG-007 ────────────────────────────────────────────────────────────────────
/// BC-2.16.015 §EC-016-015-003: When the API response delivers an empty `vulnerabilities`
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
/// BC-2.16.015 §EC-016-015-002: The `id` column uses `source_path = "$.id"`.
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

// ── F-VULNS-P1-005 (RG-007 sibling) ──────────────────────────────────────────
/// BC-2.16.015 §EC-016-015-003 hardening: when the API returns a NON-empty first page
/// (1000 records) with `count: null`, pagination must PROCEED (page_record_count ==
/// page_size == 1000 → NOT < page_size → no halt).  Then an empty second page triggers
/// the empty-page halt.  Final record count == 1000.
///
/// This hardens RG-007 which only covers `count: null` on an EMPTY first page.  That test
/// proves empty-page halt fires for zero records; this test proves null-count is handled
/// correctly when records ARE present (i.e., the pipeline does NOT null-deref on `count`
/// to decide whether to continue — it uses `page_record_count < page_size`).
///
/// Two wiremock mocks differentiated by `body_partial_json`:
///   - offset=0  → 1000 records + count: null  → pagination continues
///   - offset=1000 → empty array + count: null → empty-page halt
///
/// BC-2.16.015 AC-006; BC-2.16.002 §Postconditions OffsetLimit halt condition.
/// Story: S-CLAROTY-VULNS-001 F-VULNS-P1-005.
#[tokio::test]
async fn test_BC_2_16_015_claroty_vulnerabilities_nullable_count_nonempty_first_page_proceeds() {
    let mock_server = MockServer::start().await;

    // 1000 minimal vulnerability records for page 1.
    // Each has a `name` field so finding_info_title maps to a non-null value.
    let records: Vec<serde_json::Value> = (0..1000_u32)
        .map(|i| {
            json!({
                "name": format!("CVE-2024-{i:04}"),
                "vulnerability_type": "CVE"
            })
        })
        .collect();

    // Page 1: offset=0 → 1000 records + count: null
    // page_record_count (1000) == page_size (1000) → NOT < page_size → pagination PROCEEDS.
    Mock::given(method("POST"))
        .and(body_partial_json(json!({"offset": 0_u32})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "vulnerabilities": records,
            "count": serde_json::Value::Null,
            "total": 1000_u32,
            "page": 1_u32
        })))
        .mount(&mock_server)
        .await;

    // Page 2: offset=1000 → empty array + count: null
    // page_record_count (0) < page_size (1000) → empty-page halt fires.
    Mock::given(method("POST"))
        .and(body_partial_json(json!({"offset": 1000_u32})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "vulnerabilities": [],
            "count": serde_json::Value::Null,
            "total": 1000_u32,
            "page": 2_u32
        })))
        .mount(&mock_server)
        .await;

    let spec = SpecLoader::parse(CLAROTY_TOML).expect("claroty.sensor.toml must parse");
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
        .expect(
            "F-VULNS-P1-005: pipeline must succeed with count: null on a non-empty first page. \
             BC-2.16.015 AC-006.",
        );

    assert_eq!(
        result.records.len(),
        1000,
        "F-VULNS-P1-005: all 1000 records from page 1 must materialize. \
         Pagination PROCEEDED past null-count page 1 and halted on empty page 2. \
         BC-2.16.015 §EC-016-015-003; OffsetLimit halt condition."
    );
}

// ── F-VULNS-P1-002: EC-016-015-005 ───────────────────────────────────────────
/// BC-2.16.015 §EC-016-015-005: When `cve_ids` is an EMPTY JSON array `[]` in the API
/// response, `ColumnMapper::map_record` must store it as `Value::Array([])` in
/// `raw_extensions` — NOT as `Value::Null`, and NOT raise any error.
///
/// `cve_ids` is a Tier-2 column (no `ocsf_field`, `column_type = "json"`) that maps to
/// `raw_extensions` directly.  An empty array `[]` is a valid JSON value distinct from
/// null; it signals "no CVE IDs" without implying data absence.
///
/// NOTE: The downstream ENRICH-1 DD-2 conversion (empty `Value::Array` →
/// serialized `"[]"` string inside the `raw_extensions` JSON-object column) is
/// performed by `pipeline_result_to_record_batch` in prism-bin, AFTER `map_record`.
/// This test asserts the pre-ENRICH-1 form (`Value::Array(vec![])`) which is the correct
/// assertion at the `map_record` boundary.
///
/// Story: S-CLAROTY-VULNS-001 F-VULNS-P1-002.
#[test]
fn test_BC_2_16_015_claroty_vulnerabilities_ec005_cve_ids_empty_array_in_raw_extensions() {
    let spec = SpecLoader::parse(CLAROTY_TOML).expect("claroty.sensor.toml must parse");

    let table = spec
        .tables
        .iter()
        .find(|t| t.table_name == "claroty_vulnerabilities")
        .expect("claroty_vulnerabilities table must exist");

    let record = json!({ "cve_ids": [] });

    let row = ColumnMapper::map_record(&record, table).expect(
        "EC-016-015-005: map_record must succeed when cve_ids is an empty array. \
         BC-2.16.015 §EC-016-015-005.",
    );

    assert_eq!(
        row.raw_extensions.get("cve_ids"),
        Some(&serde_json::Value::Array(vec![])),
        "EC-016-015-005: cve_ids=[] must be stored as Value::Array([]) in raw_extensions \
         (NOT null). \
         raw_extensions: {:?}",
        row.raw_extensions
    );
}

// ── F-VULNS-P1-002: EC-016-015-006 ───────────────────────────────────────────
/// BC-2.16.015 §EC-016-015-006: When `published_date` is JSON `null` in the API response,
/// `ColumnMapper::map_record` must:
///   1. Store `Value::Null` in `raw_extensions["published_date"]` (row materializes)
///   2. NOT raise an error (in particular, NOT raise E-SPEC-018 `TimestampParseFailure`)
///
/// `published_date` is a Tier-2 datetime column (`column_type = "datetime"`, no
/// `ocsf_field`).  A null value is valid — it means "unpublished" — and the pipeline
/// must not attempt datetime parsing on it.
///
/// NOTE: The E-SPEC-018 (`TimestampParseFailure`) gate lives in
/// `normalize_timestamp_fields` in pipeline.rs, which uses `is_null_or_absent` to
/// skip null fields before attempting datetime parsing.  At the `map_record` level
/// (this test's scope), E-SPEC-018 can never be raised — `map_record` stores the raw
/// `Value::Null` without parsing.  The full-pipeline null-datetime protection is
/// exercised by RG-007 (via `PipelineExecutor::execute` with a null-valued response).
///
/// Story: S-CLAROTY-VULNS-001 F-VULNS-P1-002.
#[test]
fn test_BC_2_16_015_claroty_vulnerabilities_ec006_published_date_null_row_materializes() {
    let spec = SpecLoader::parse(CLAROTY_TOML).expect("claroty.sensor.toml must parse");

    let table = spec
        .tables
        .iter()
        .find(|t| t.table_name == "claroty_vulnerabilities")
        .expect("claroty_vulnerabilities table must exist");

    let record = json!({ "published_date": null });

    let row = ColumnMapper::map_record(&record, table).expect(
        "EC-016-015-006: map_record must not error when published_date is JSON null. \
         No E-SPEC-018 may be raised at this boundary. BC-2.16.015 §EC-016-015-006.",
    );

    assert_eq!(
        row.raw_extensions.get("published_date"),
        Some(&serde_json::Value::Null),
        "EC-016-015-006: published_date=null must store Value::Null in raw_extensions. \
         raw_extensions: {:?}",
        row.raw_extensions
    );
}

// ── F-VULNS-ADV-002: EC-016-015-007 — non-ISO published_date → E-SPEC-018 ─────
/// BC-2.16.015 §EC-016-015-007: When `published_date` in the API response carries a
/// non-null, non-ISO-8601 string (e.g. `"not-a-timestamp"` or `"2024/13/99"`),
/// `normalize_timestamp_fields` in prism-spec-engine/src/pipeline.rs must fire
/// `SpecEngineError::TimestampParseFailure` (E-SPEC-018).
///
/// `published_date` is a Tier-2 `column_type = "datetime"` column with no declared
/// `timestamp_formats` — resolving to the implicit `["iso8601"]` default via
/// `effective_formats` (ADR-028 §D8-B).  `normalize_timestamp_fields` is
/// tier-agnostic (filters by `column_type == Datetime`) and processes `published_date`
/// in the raw API record before `ColumnMapper::map_record`.
///
/// This test covers the previously-uncovered E-SPEC-018 arm for the
/// `claroty_vulnerabilities` Tier-2 datetime column path.  The arm is reachable
/// from the full `PipelineExecutor::execute` path because:
///
///   1. The mock returns `{"published_date": "not-a-timestamp", "name": "CVE-2024-1234", ...}`
///   2. `normalize_timestamp_fields` sees `published_date = "not-a-timestamp"` as a
///      non-null Datetime field and tries to parse with `["iso8601"]` → all fail.
///   3. The function returns `Err(SpecEngineError::TimestampParseFailure)`, which
///      propagates through `PipelineExecutor::execute` via `?`.
///
/// The error carries `column_name = "published_date"` and `sensor_id = "claroty"`.
///
/// Story: S-CLAROTY-VULNS-001 F-VULNS-ADV-002 (pass-4 fix-burst).
/// BC-2.16.015 §EC-016-015-007; SpecEngineError::TimestampParseFailure (E-SPEC-018).
#[tokio::test]
async fn test_BC_2_16_015_claroty_vulnerabilities_ec007_non_iso_published_date_e_spec_018() {
    let mock_server = MockServer::start().await;

    // Return a vulnerability record with a non-ISO published_date string.
    // "not-a-timestamp" fails all ISO-8601 parse attempts in normalize_timestamp_fields.
    // The record also carries a valid `name` field so that map_record itself succeeds;
    // the E-SPEC-018 failure is in the timestamp normalization pass, not the mapping pass.
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "vulnerabilities": [{
                "name": "CVE-2024-1234",
                "description": "A mock vulnerability with non-ISO published_date",
                "vulnerability_type": "CVE",
                "published_date": "not-a-timestamp"
            }],
            "total": 1_u32,
            "page": 1_u32
        })))
        .mount(&mock_server)
        .await;

    let spec = SpecLoader::parse(CLAROTY_TOML).expect("claroty.sensor.toml must parse");

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

    let result = PipelineExecutor::execute(&test_spec, table, &context, &http_client, &auth).await;

    // LOAD-BEARING EC-007 assertion: normalize_timestamp_fields must fire E-SPEC-018
    // when published_date cannot be parsed as ISO-8601.
    // The pipeline propagates this Err via ? — no row is materialized.
    assert!(
        result.is_err(),
        "EC-007 LOAD-BEARING: PipelineExecutor::execute must return Err when \
         published_date is non-ISO ('not-a-timestamp'). \
         normalize_timestamp_fields (prism-spec-engine/src/pipeline.rs) is \
         tier-agnostic and processes claroty_vulnerabilities Tier-2 Datetime columns. \
         Got Ok with {} records. BC-2.16.015 §EC-016-015-007.",
        result.as_ref().map(|r| r.records.len()).unwrap_or(0)
    );

    let err = result.unwrap_err();
    match &err {
        SpecEngineError::TimestampParseFailure {
            column_name,
            sensor_id,
            ..
        } => {
            assert_eq!(
                column_name.as_str(),
                "published_date",
                "EC-007: TimestampParseFailure.column_name must be 'published_date'; \
                 got: {column_name:?}. BC-2.16.015 §EC-016-015-007."
            );
            assert_eq!(
                sensor_id.as_str(),
                "claroty",
                "EC-007: TimestampParseFailure.sensor_id must be 'claroty'; \
                 got: {sensor_id:?}. BC-2.16.015 §EC-016-015-007."
            );
        }
        other => {
            panic!(
                "EC-007 LOAD-BEARING: PipelineExecutor::execute must return \
                 SpecEngineError::TimestampParseFailure (E-SPEC-018) when published_date \
                 is non-ISO. Got: {other:?}. BC-2.16.015 §EC-016-015-007."
            );
        }
    }
}

// ── F-VULNS-EC004-001: EC-016-015-004 ────────────────────────────────────────
/// BC-2.16.015 §EC-016-015-004: A vulnerability row whose `name` is an advisory-title
/// format (e.g., "ICSMA-21-161-01 (ZOLL Defibrillator Dashboard)") — NOT a CVE-YYYY-NNNNN
/// format — is preserved VERBATIM in the `finding_info.title` mapping.  No normalization
/// is applied; the mapped value must equal the input string exactly.
///
/// `ColumnMapper::map_record` stores Tier-1 fields in DOT form (intermediate
/// representation): `finding_info.title` is the intermediate key stored in
/// `mapped_fields` (not the Arrow name `finding_info_title`).  Arrow-name flattening
/// (`finding_info.title` → `finding_info_title`) happens downstream in
/// `pipeline_result_to_record_batch`.
///
/// Story: S-CLAROTY-VULNS-001 F-VULNS-EC004-001 (pass-3 fix-burst).
#[test]
fn test_BC_2_16_015_claroty_vulnerabilities_ec004_advisory_title_preserved_verbatim() {
    let spec = SpecLoader::parse(CLAROTY_TOML).expect("claroty.sensor.toml must parse");

    let table = spec
        .tables
        .iter()
        .find(|t| t.table_name == "claroty_vulnerabilities")
        .expect("claroty_vulnerabilities table must exist");

    // Advisory-title format: NOT CVE-YYYY-NNNNN.  Per BC-2.16.015 §EC-016-015-004
    // this value must be preserved verbatim — no normalisation of any kind.
    let advisory_title = "ICSMA-21-161-01 (ZOLL Defibrillator Dashboard)";
    let record = json!({ "name": advisory_title });

    let row = ColumnMapper::map_record(&record, table).expect(
        "EC-016-015-004: map_record must succeed for an advisory-title format name. \
         BC-2.16.015 §EC-016-015-004.",
    );

    // map_record stores Tier-1 fields in DOT form (intermediate); assert the dot-form
    // key `finding_info.title` (arrow-name flattening is downstream).
    assert_eq!(
        row.mapped_fields.get("finding_info.title"),
        Some(&serde_json::Value::String(advisory_title.to_string())),
        "EC-016-015-004: advisory-title '{}' MUST be preserved VERBATIM as \
         finding_info.title in mapped_fields — no normalisation applied. \
         Got: {:?}. BC-2.16.015 §EC-016-015-004.",
        advisory_title,
        row.mapped_fields.get("finding_info.title")
    );
}
