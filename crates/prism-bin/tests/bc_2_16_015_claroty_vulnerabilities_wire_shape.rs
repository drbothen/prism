// SPDX-License-Identifier: Apache-2.0
//! Wire-shape and SAP-3 end-to-end tests for BC-2.16.015 —
//! Claroty xDome Vulnerability Findings Table.
//!
//! # Tests in this file
//!
//! | ID           | Test name | Assertion |
//! |--------------|-----------|-----------|
//! | NEW-1        | test_BC_2_16_015_claroty_vulnerabilities_wire_shape_class_uid_2002_mock | class_uid=2002, finding_info_title, message, raw_extensions as JSON object, no Tier-2 top-level keys (RecordBatch-level assertions) |
//! | F-VULNS-P1-001 | test_BC_2_16_015_claroty_vulnerabilities_wire_shape_serialized_json_explicit_nulls | Serialized-JSON wire assertions via production arrow_json path; null-not-absent for missing name |
//! | SAP-3        | test_BC_2_16_015_claroty_vulnerabilities_e2e_e_query_038_tier2_column | PrismError::ColumnNotFound (E-QUERY-038) when querying a Tier-2 column via QueryEngine::execute() |
//!
//! # F-VULNS-004 / SID-1 compliance (NEW-1)
//!
//! NEW-1 is the non-ignored wire-shape mock test required by SID-1.
//! RG-004 in `prism-sensors/tests/bc_2_16_015_claroty_vulnerabilities.rs` is
//! `#[ignore]`'d (live-only); NEW-1 provides the non-live wire-shape coverage via
//! wiremock + Arrow RecordBatch assertions on the actual serialized column values.
//!
//! # F-VULNS-005 / SAP-3 compliance (SAP-3 test)
//!
//! SAP-3 rule 1: at least one test must reach each BC postcondition arm end-to-end
//! from the public surface (parser input), not just from a synthetic-AST proxy.
//! RG-003 in prism-sensors calls `ocsf_projected_column_names()` directly — valid
//! defense-in-depth, but SAP-3 requires this additional parser-surface test.
//!
//! The E-QUERY-038 plan-time column gate fires in `QueryEngine::execute_inner`
//! (engine.rs), BEFORE `run_materialization_pipeline` is called.  Tests that call
//! `run_materialization_pipeline` directly bypass the gate; this test uses
//! `QueryEngine::execute()` as the SAP-3 entry point.
//!
//! BC: BC-2.16.015
//! Story: S-CLAROTY-VULNS-001

#![allow(
    dead_code,
    unused_imports,
    non_snake_case,
    clippy::unwrap_used,
    clippy::expect_used
)]
extern crate toml;

use arrow_json;
use std::sync::Arc;

use arrow::array::Array;
use prism_bin::spec_driven_adapter::{AdapterAuthStrategy, SpecDrivenSensorAdapter};
use prism_core::{OrgId, OrgSlug, PrismError, SensorId};
use prism_ocsf::OcsfNormalizer;
use prism_query::{
    cache::CacheConfig,
    engine::{QueryEngine, QueryEngineConfig, QueryOptions},
    scoping::ClientRegistry,
    table_registry::TableRegistry,
};
use prism_sensors::{
    BearerStaticSensorAuth, SensorAdapter, adapter::QueryParams,
    adapter::SensorSpec as SensorAdapterSpec, auth::SensorAuth,
};
use prism_spec_engine::{
    overlay::{OverlayLoader, SensorInstanceOverlay},
    spec_parser::SpecLoader,
};
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{method, path},
};

// ---------------------------------------------------------------------------
// Shared helpers (NEW-1 mock test)
// ---------------------------------------------------------------------------

/// Build a `SpecDrivenSensorAdapter` from the production `claroty.sensor.toml`
/// directed at the given mock server URI.
fn make_claroty_adapter(mock_server_uri: &str) -> SpecDrivenSensorAdapter {
    let spec_content = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../prism-sensors/specs/claroty.sensor.toml"),
    )
    .expect(
        "bc_2_16_015_wire_shape: claroty.sensor.toml must be readable from \
         CARGO_MANIFEST_DIR/../prism-sensors/specs/",
    );

    let mut spec = SpecLoader::parse(&spec_content)
        .expect("bc_2_16_015_wire_shape: claroty.sensor.toml must parse");
    spec.base_url = mock_server_uri.to_string();

    let overlay_toml = "extends = \"claroty\"\ninstance_id = \"claroty@claroty-vulns-wire-test\"";
    let overlay: SensorInstanceOverlay = toml::from_str(overlay_toml)
        .expect("bc_2_16_015_wire_shape: SensorInstanceOverlay TOML parse failed");
    let resolved = OverlayLoader::merge_overlay_onto_type_spec(
        &spec,
        &overlay,
        OrgSlug::new("claroty-vulns-wire-test"),
    );

    let http_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("bc_2_16_015_wire_shape: reqwest::Client build failed (ADR-050 rustls-tls)");

    SpecDrivenSensorAdapter::new(
        Arc::new(resolved),
        AdapterAuthStrategy::BearerStatic,
        http_client,
    )
}

// ---------------------------------------------------------------------------
// SAP-3 helper: no-op CredentialStore for QueryEngine construction
// ---------------------------------------------------------------------------

/// Minimal no-op `CredentialStore` for constructing `QueryEngine` in SAP-3 tests.
/// Matches the `NoopCs` pattern from `ocsf_column_routing_tests.rs` in prism-query.
/// The SAP-3 test fires at plan-time (E-QUERY-038) before any credential lookup.
struct NoopCredentialStore;

#[async_trait::async_trait]
impl prism_credentials::CredentialStore for NoopCredentialStore {
    async fn get(
        &self,
        _tenant: &prism_core::OrgSlug,
        _sensor_id: &str,
        _name: &prism_credentials::namespace::CredentialName,
    ) -> Result<Option<secrecy::SecretString>, PrismError> {
        Ok(None)
    }

    async fn set(
        &self,
        _tenant: &prism_core::OrgSlug,
        _sensor_id: &str,
        _name: &prism_credentials::namespace::CredentialName,
        _value: secrecy::SecretString,
    ) -> Result<(), PrismError> {
        Ok(())
    }

    async fn delete(
        &self,
        _tenant: &prism_core::OrgSlug,
        _sensor_id: &str,
        _name: &prism_credentials::namespace::CredentialName,
    ) -> Result<bool, PrismError> {
        Ok(false)
    }

    async fn list(
        &self,
        _tenant: &prism_core::OrgSlug,
    ) -> Result<Vec<(String, prism_credentials::namespace::CredentialName)>, PrismError> {
        Ok(vec![])
    }

    async fn exists(
        &self,
        _tenant: &prism_core::OrgSlug,
        _sensor_id: &str,
        _name: &prism_credentials::namespace::CredentialName,
    ) -> Result<bool, PrismError> {
        Ok(false)
    }
}

// ---------------------------------------------------------------------------
// NEW-1: Wire-shape mock test (SID-1 compliance, F-VULNS-004)
// ---------------------------------------------------------------------------

/// Wire-shape mock test: `SpecDrivenSensorAdapter::fetch()` for `claroty_vulnerabilities`
/// returns RecordBatches with:
///   - `class_uid == 2002` (EventClassSelector("vulnerability_finding"))
///   - `finding_info_title` column present (ocsf_field "finding_info.title" → arrow name)
///   - `message` column present (ocsf_field "message")
///   - `raw_extensions` present as StringArray holding a JSON object (Tier-2 aggregate)
///   - No Tier-2 column names at top level (Tier-2 isolation, ADR-058 §J6)
///
/// SID-1 compliance: non-ignored, uses wiremock. Provides non-live wire-shape coverage
/// for RG-004 (which is `#[ignore]`'d pending a live Claroty instance at CLAROTY_INSTANCE_URL).
///
/// F-VULNS-010 note: the wire-null case for a missing `name` field (finding_info_title
/// serialized as JSON null, not absent) requires the full RecordBatch → arrow_json
/// serialization path with `explicit_nulls(true)`. This is asserted by the live RG-004
/// path which builds a simulated wire row from the map_record output. At the RecordBatch
/// level, the column exists in the schema even when a row has a null value — the null
/// is present-not-absent per the `explicit_nulls(true)` wire discipline (CLAUDE.md
/// §Wire-shape assertion discipline).
///
/// BC-2.16.015 AC-004 / AC-005; ADR-058 §C2 Option 4, §J6.
/// Story: S-CLAROTY-VULNS-001 RG-004 (non-live SID-1 coverage).
#[tokio::test]
async fn test_BC_2_16_015_claroty_vulnerabilities_wire_shape_class_uid_2002_mock() {
    let mock_server = MockServer::start().await;

    // Mock the DTU vulnerabilities endpoint.
    // DTU route: POST /api/v1/vulnerabilities/ (trailing slash per claroty.sensor.toml).
    // Response envelope: {"vulnerabilities": [...], "total": N, "page": N}
    // One record with both Tier-1 fields (name, description) + representative Tier-2 fields.
    Mock::given(method("POST"))
        .and(path("/api/v1/vulnerabilities/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "vulnerabilities": [{
                "name": "CVE-2024-9999",
                "description": "A mock vulnerability for wire-shape testing",
                "vulnerability_type": "CVE",
                "cve_ids": ["CVE-2024-9999"],
                "cvss_v3_score": 9.8_f64,
                "id": "mock-vuln-wire-001",
                "is_known_exploited": true,
                "epss_score": 0.75_f64
            }],
            "total": 1_u32,
            "page": 1_u32
        })))
        .mount(&mock_server)
        .await;

    let adapter = make_claroty_adapter(&mock_server.uri());

    // source_table = "claroty_claroty_vulnerabilities":
    //   strip "claroty_" prefix → "claroty_vulnerabilities" = table.table_name in the spec.
    //   (Registered in DataFusion as: format!("{sensor_id}_{table_name}") = "claroty_claroty_vulnerabilities")
    let adapter_spec = SensorAdapterSpec {
        source_table: "claroty_claroty_vulnerabilities".to_string(),
        org_id: OrgId::from_uuid(uuid::Uuid::now_v7()),
        #[allow(deprecated)]
        client_id: "claroty-vulns-wire-test".to_string(),
        sensor_config: serde_json::json!({}),
    };

    let params = QueryParams {
        cursor: None,
        limit: 10,
        start_time: None,
        end_time: None,
        filters: Default::default(),
    };

    let sensor_auth = BearerStaticSensorAuth::new("mock-bearer-token-vulns-wire-test");

    let result = adapter.fetch(&adapter_spec, &params, &sensor_auth).await;
    assert!(
        result.is_ok(),
        "fetch() must return Ok when mock returns a valid vulnerabilities response. \
         Got Err: {:?}. BC-2.16.015 AC-004.",
        result.err()
    );

    let batches = result.unwrap();
    assert!(
        !batches.is_empty(),
        "fetch() must return at least one RecordBatch for a non-empty response. \
         BC-2.16.015 AC-004."
    );

    let first_batch = &batches[0];
    assert!(
        first_batch.num_rows() > 0,
        "RecordBatch must contain at least one row. BC-2.16.015 AC-004."
    );

    let schema = first_batch.schema();
    let column_names: Vec<&str> = schema.fields().iter().map(|f| f.name().as_str()).collect();

    // ── Wire-shape assertion 1: class_uid == 2002 ──────────────────────────────
    // EventClassSelector::select_by_class_name("vulnerability_finding") = 2002
    // (ADR-058 §C2; BC-2.02.012 Vulnerability Finding class mapping).
    let class_uid_col_idx = schema.index_of("class_uid").expect(
        "RecordBatch must contain 'class_uid' column (OCSF synthesized field). \
             BC-2.16.015 AC-004.",
    );
    let class_uid_col = first_batch.column(class_uid_col_idx);
    let class_uid_array = class_uid_col
        .as_any()
        .downcast_ref::<arrow::array::Int32Array>()
        .expect("class_uid column must be Int32Array");
    let class_uid_val = class_uid_array.value(0);
    assert_eq!(
        class_uid_val, 2002,
        "class_uid MUST equal 2002 \
         (EventClassSelector::select_by_class_name('vulnerability_finding') = 2002). \
         Got: {}. BC-2.16.015 AC-004; ADR-058 §C2.",
        class_uid_val
    );

    // ── Wire-shape assertion 2: finding_info_title column present ──────────────
    // ocsf_field_to_arrow_name("finding_info.title") = "finding_info_title"
    // (ADR-058 §C2 Option 4: dot → underscore for nested OCSF fields).
    assert!(
        column_names.contains(&"finding_info_title"),
        "RecordBatch must contain 'finding_info_title' column \
         (name → ocsf_field=finding_info.title → ADR-058 arrow name finding_info_title). \
         BC-2.16.015 AC-004. Present columns: {:?}",
        column_names
    );

    // ── Wire-shape assertion 3: message column present ─────────────────────────
    // ocsf_field_to_arrow_name("message") = "message" (single segment, unchanged).
    assert!(
        column_names.contains(&"message"),
        "RecordBatch must contain 'message' column \
         (description → ocsf_field=message → arrow name message). \
         BC-2.16.015 AC-004. Present columns: {:?}",
        column_names
    );

    // ── Wire-shape assertion 4: raw_extensions present as StringArray (JSON object) ──
    // ADR-058 §J6: Tier-2 columns aggregate into raw_extensions (DataType::Utf8 StringArray).
    assert!(
        column_names.contains(&"raw_extensions"),
        "RecordBatch must contain 'raw_extensions' column (Tier-2 aggregation, ADR-058 §J6). \
         BC-2.16.015 AC-005. Present columns: {:?}",
        column_names
    );
    let raw_ext_col_idx = schema
        .index_of("raw_extensions")
        .expect("raw_extensions column must be present (asserted above)");
    let raw_ext_col = first_batch.column(raw_ext_col_idx);
    let raw_ext_array = raw_ext_col
        .as_any()
        .downcast_ref::<arrow::array::StringArray>()
        .expect("raw_extensions column must be StringArray (DataType::Utf8)");
    // The first row must have a non-null raw_extensions value (we sent Tier-2 data).
    assert!(
        !raw_ext_array.is_null(0),
        "raw_extensions must not be null in the first row when Tier-2 data is present. \
         BC-2.16.015 AC-005."
    );
    let raw_ext_str = raw_ext_array.value(0);
    let raw_ext_json: serde_json::Value = serde_json::from_str(raw_ext_str)
        .expect("raw_extensions value must be valid JSON (DataType::Utf8 JSON blob)");
    assert!(
        raw_ext_json.is_object(),
        "raw_extensions must deserialize to a JSON object (not array, not scalar). \
         Got: {:?}. BC-2.16.015 AC-005; ADR-058 §J6.",
        raw_ext_json
    );

    // ── Wire-shape assertion 5: no Tier-2 column names at top level ───────────
    // Tier-2 columns MUST be inside raw_extensions, not as top-level wire columns.
    // BC-2.16.015 §2 Tier-2 isolation (ADR-058 §J6).
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
            !column_names.contains(tier2_name),
            "Tier-2 column '{}' MUST NOT appear as a top-level RecordBatch column — \
             it must be inside raw_extensions. BC-2.16.015 §2; ADR-058 §J6. \
             Top-level columns: {:?}",
            tier2_name,
            column_names
        );
    }

    // ── Wire-shape assertion 6: at least one Tier-2 field is inside raw_extensions ──
    let raw_ext_obj = raw_ext_json
        .as_object()
        .expect("raw_extensions must be a JSON object");
    let tier2_in_raw_ext = tier2_names
        .iter()
        .any(|name| raw_ext_obj.contains_key(*name));
    assert!(
        tier2_in_raw_ext,
        "raw_extensions object must contain at least one Tier-2 field. \
         BC-2.16.015 AC-005. raw_extensions keys: {:?}",
        raw_ext_obj.keys().collect::<Vec<_>>()
    );
}

// ---------------------------------------------------------------------------
// F-VULNS-P1-001: Serialized-JSON wire-shape + null-not-absent assertion
// ---------------------------------------------------------------------------

/// Serialized-JSON wire-shape test: verifies that RecordBatches produced by
/// `SpecDrivenSensorAdapter::fetch()` for `claroty_vulnerabilities`, when serialized
/// through the production MCP path (`arrow_json::writer::WriterBuilder::new()
/// .with_explicit_nulls(true).build::<_, arrow_json::writer::JsonArray>(&mut buf)`),
/// produce correctly-shaped JSON row objects satisfying the wire-shape discipline.
///
/// LOAD-BEARING assertions (CLAUDE.md §Wire-shape assertion discipline):
///   1. `class_uid` == 2002 present at top level (integer)
///   2. `finding_info_title` and `message` present at top level
///   3. `raw_extensions` present as a JSON object containing at least one Tier-2 field
///   4. No Tier-2 column name appears as a standalone top-level key
///   5. NULL-NOT-ABSENT (C3/H20 defect class): for a row where API `name` is absent,
///      `finding_info_title` MUST appear as `null` in the JSON row — NOT be omitted.
///      `arrow_json` with `explicit_nulls=false` (the DEFAULT) would omit the key;
///      this test locks in the `explicit_nulls=true` production configuration.
///
/// Two mock records:
///   - Record 0: `name = "CVE-2024-9999"` (Tier-1 present) + Tier-2 fields
///   - Record 1: no `name` field → `finding_info_title` becomes null in RecordBatch
///
/// BC-2.16.015 AC-004 / AC-005; BC-2.11.001 EC-11-079 (null-not-absent);
/// CLAUDE.md §Wire-shape assertion discipline. Story: S-CLAROTY-VULNS-001 F-VULNS-P1-001.
#[tokio::test]
async fn test_BC_2_16_015_claroty_vulnerabilities_wire_shape_serialized_json_explicit_nulls() {
    let mock_server = MockServer::start().await;

    // Two records: first has `name`, second lacks it.
    // This exercises:
    //   - Row 0: all top-level Tier-1 fields present + raw_extensions non-null
    //   - Row 1: finding_info_title = null (absent source field → Arrow null cell)
    Mock::given(method("POST"))
        .and(path("/api/v1/vulnerabilities/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "vulnerabilities": [
                {
                    "name": "CVE-2024-9999",
                    "description": "A mock vulnerability for serialized JSON wire testing",
                    "vulnerability_type": "CVE",
                    "cve_ids": ["CVE-2024-9999"],
                    "cvss_v3_score": 9.8_f64,
                    "id": "mock-wire-json-001",
                    "is_known_exploited": true
                },
                {
                    // No `name` field — finding_info_title becomes Arrow null → "finding_info_title":null in JSON
                    "description": "No-name vulnerability for null-not-absent test",
                    "vulnerability_type": "CWE",
                    "cvss_v3_score": 5.0_f64
                }
            ],
            "total": 2_u32,
            "page": 1_u32
        })))
        .mount(&mock_server)
        .await;

    let adapter = make_claroty_adapter(&mock_server.uri());

    let adapter_spec = SensorAdapterSpec {
        source_table: "claroty_claroty_vulnerabilities".to_string(),
        org_id: OrgId::from_uuid(uuid::Uuid::now_v7()),
        #[allow(deprecated)]
        client_id: "claroty-vulns-wire-json-test".to_string(),
        sensor_config: serde_json::json!({}),
    };

    let params = QueryParams {
        cursor: None,
        limit: 10,
        start_time: None,
        end_time: None,
        filters: Default::default(),
    };

    let sensor_auth = BearerStaticSensorAuth::new("mock-bearer-token-wire-json-test");

    let batches = adapter
        .fetch(&adapter_spec, &params, &sensor_auth)
        .await
        .expect("fetch() must succeed for a valid two-record mock response. BC-2.16.015 AC-004.");

    assert!(
        !batches.is_empty(),
        "fetch() must return at least one RecordBatch for a non-empty response. \
         BC-2.16.015 AC-004."
    );

    // ── Production MCP serialization path ─────────────────────────────────────
    // Mirrors server.rs (prism-mcp) §CRIT-1 fix:
    //   arrow_json::writer::WriterBuilder::new()
    //       .with_explicit_nulls(true)
    //       .build::<_, arrow_json::writer::JsonArray>(&mut buf)
    //
    // explicit_nulls=true: NULL-valued Arrow cells → `{"key":null}` in JSON output.
    // explicit_nulls=false (DEFAULT): NULL cells are OMITTED — the C3/H20 defect class
    // (BC-2.11.001 EC-11-079; CLAUDE.md §Wire-shape assertion discipline).
    let mut buf: Vec<u8> = Vec::new();
    let mut writer = arrow_json::writer::WriterBuilder::new()
        .with_explicit_nulls(true)
        .build::<_, arrow_json::writer::JsonArray>(&mut buf);
    for batch in &batches {
        writer.write(batch).expect(
            "F-VULNS-P1-001: arrow_json write must not fail for claroty_vulnerabilities RecordBatch",
        );
    }
    writer
        .finish()
        .expect("F-VULNS-P1-001: arrow_json finish must not fail");

    let json_rows: Vec<serde_json::Value> = serde_json::from_slice::<Vec<serde_json::Value>>(&buf)
        .expect(
            "F-VULNS-P1-001: arrow_json output must deserialize as a JSON array of row objects",
        );

    assert_eq!(
        json_rows.len(),
        2,
        "F-VULNS-P1-001: serialized JSON must contain exactly 2 rows (one per mock record). \
         BC-2.16.015 AC-004."
    );

    // ── Row 0: name present → finding_info_title non-null ─────────────────────
    let row0 = &json_rows[0];

    assert_eq!(
        row0.get("class_uid"),
        Some(&serde_json::json!(2002_i32)),
        "F-VULNS-P1-001 LOAD-BEARING: row0 class_uid must equal 2002 in serialized JSON. \
         BC-2.16.015 AC-004; ADR-058 §C2."
    );

    assert_eq!(
        row0.get("finding_info_title"),
        Some(&serde_json::json!("CVE-2024-9999")),
        "F-VULNS-P1-001: row0 finding_info_title must be 'CVE-2024-9999' in serialized JSON. \
         BC-2.16.015 AC-004."
    );

    assert!(
        row0.get("message").is_some(),
        "F-VULNS-P1-001: row0 message must be present in serialized JSON. BC-2.16.015 AC-004."
    );

    // raw_extensions must be a StringArray cell containing a JSON object string
    let raw_ext_str0 = row0.get("raw_extensions").and_then(|v| v.as_str()).expect(
        "F-VULNS-P1-001: row0 raw_extensions must be present and a JSON string. \
             BC-2.16.015 AC-005; ADR-058 §J6.",
    );
    let raw_ext_json0: serde_json::Value = serde_json::from_str(raw_ext_str0)
        .expect("F-VULNS-P1-001: row0 raw_extensions must be valid JSON");
    assert!(
        raw_ext_json0.is_object(),
        "F-VULNS-P1-001: row0 raw_extensions must be a JSON object. \
         Got: {:?}. BC-2.16.015 AC-005; ADR-058 §J6.",
        raw_ext_json0
    );

    let raw_ext_obj0 = raw_ext_json0
        .as_object()
        .expect("raw_extensions is an object (asserted above)");
    let tier2_spot_check = ["vulnerability_type", "cve_ids", "cvss_v3_score", "id"];
    let has_tier2 = tier2_spot_check
        .iter()
        .any(|name| raw_ext_obj0.contains_key(*name));
    assert!(
        has_tier2,
        "F-VULNS-P1-001: row0 raw_extensions must contain at least one Tier-2 field. \
         raw_extensions keys: {:?}. BC-2.16.015 AC-005.",
        raw_ext_obj0.keys().collect::<Vec<_>>()
    );

    // No Tier-2 column name must appear as a top-level key in row0
    let all_tier2 = [
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
    for tier2_name in &all_tier2 {
        assert!(
            row0.get(*tier2_name).is_none(),
            "F-VULNS-P1-001: Tier-2 column '{}' MUST NOT appear as a top-level key in \
             serialized JSON row0. BC-2.16.015 §2; ADR-058 §J6.",
            tier2_name
        );
    }

    // ── Row 1: name absent → finding_info_title null-not-absent ───────────────
    let row1 = &json_rows[1];

    // NULL-NOT-ABSENT LOAD-BEARING assertion:
    // When `name` is absent in the API record, `finding_info_title` becomes an Arrow null
    // cell (nullable=true, value=None). With explicit_nulls=true the JSON row MUST contain
    // `"finding_info_title": null` — NOT omit the key.
    // With explicit_nulls=false (the arrow_json DEFAULT), the key would be absent.
    // This locks in the C3/H20-class defect prevention (BC-2.11.001 EC-11-079).
    let row1_fit = row1.get("finding_info_title");
    assert!(
        row1_fit.is_some(),
        "F-VULNS-P1-001 LOAD-BEARING (null-not-absent): 'finding_info_title' key MUST be \
         PRESENT in row1 serialized JSON even when 'name' was absent in the API response. \
         arrow_json with explicit_nulls=false (DEFAULT) would OMIT this key — \
         that is the C3/H20 defect class (BC-2.11.001 EC-11-079). \
         row1 keys present: {:?}",
        row1.as_object().map(|o| o.keys().collect::<Vec<_>>())
    );
    assert_eq!(
        row1_fit,
        Some(&serde_json::Value::Null),
        "F-VULNS-P1-001 LOAD-BEARING (null-not-absent): 'finding_info_title' MUST be \
         JSON null (not another value) in row1 serialized output. \
         BC-2.11.001 EC-11-079; BC-2.16.015 AC-004."
    );
}

// ---------------------------------------------------------------------------
// SAP-3: End-to-end E-QUERY-038 test via QueryEngine::execute() (F-VULNS-005)
// ---------------------------------------------------------------------------

/// SAP-3 reachability test: querying a Tier-2 column directly via the REAL
/// `QueryEngine::execute()` surface raises `PrismError::ColumnNotFound`
/// (E-QUERY-038) with correct `available_columns`.
///
/// SAP-3 rule 1: at least one test must reach the BC-2.16.015 AC-003 arm
/// end-to-end from the public surface (PrismQL parser input), not merely via
/// the synthetic proxy in RG-003 (prism-sensors, `ocsf_projected_column_names`).
///
/// Architecture: the E-QUERY-038 gate (`check_query_column_availability`) fires
/// inside `QueryEngine::execute_inner` (engine.rs §S-DEMO-PRISMQL-ONBOARDING-001-B),
/// BEFORE `run_materialization_pipeline` is called. This test uses:
///   1. `SpecLoader::parse(claroty.sensor.toml)` — production spec
///   2. `TableRegistry::register_sensor(&spec)` — populates OCSF-projected names
///      (S-ADR058-OCSF-ROUTING-001 fix: stores Arrow names, not raw col.names)
///   3. `QueryEngine::new_with_cache_config(...).with_table_registry(registry)` — wires the gate
///   4. `engine.execute("SELECT vulnerability_type FROM claroty_claroty_vulnerabilities", ...)`
///      — enters via the public query surface, fires E-QUERY-038
///
/// `vulnerability_type` is Tier-2 (no ocsf_field in claroty.sensor.toml):
///   - NOT in ocsf_projected_column_names → NOT in TableRegistry → E-QUERY-038
///   - available_columns contains "raw_extensions", "finding_info_title", "message"
///   - available_columns does NOT contain "vulnerability_type"
///
/// Registered table name: `{sensor_id}_{table_name}` = `claroty_claroty_vulnerabilities`
/// (sensor_id="claroty", table_name="claroty_vulnerabilities" per TOML).
///
/// No HTTP requests are issued — E-QUERY-038 fires at plan-time before any fan-out.
///
/// BC-2.16.015 AC-003; SAP-3; ADR-058 §I7; S-ADR058-OCSF-ROUTING-001.
/// Story: S-CLAROTY-VULNS-001 RG-003 (SAP-3 end-to-end gate).
#[tokio::test]
async fn test_BC_2_16_015_claroty_vulnerabilities_e2e_e_query_038_tier2_column() {
    // Load the production claroty.sensor.toml (ocsf_column_naming = true at sensor level).
    let spec_content = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../prism-sensors/specs/claroty.sensor.toml"),
    )
    .expect(
        "SAP-3 test: claroty.sensor.toml must be readable from \
         CARGO_MANIFEST_DIR/../prism-sensors/specs/",
    );
    let spec =
        SpecLoader::parse(&spec_content).expect("SAP-3 test: claroty.sensor.toml must parse");

    // TableRegistry::register_sensor populates OCSF-projected column names for
    // sensors with ocsf_column_naming = true (S-ADR058-OCSF-ROUTING-001 fix).
    // For claroty_claroty_vulnerabilities:
    //   Tier-1 columns → Arrow names (finding_info_title, message, ...)
    //   Tier-2 columns → aggregated as "raw_extensions"
    //   "vulnerability_type" (Tier-2) → NOT in projected columns → E-QUERY-038
    let registry = Arc::new(TableRegistry::new());
    registry
        .register_sensor(&spec)
        .expect("SAP-3 test: register_sensor must not fail for production claroty.sensor.toml");

    // Build QueryEngine with the TableRegistry wired.
    // new_with_cache_config uses a NullCredentialResolver by default (no fan-out occurs
    // because E-QUERY-038 fires before any HTTP requests).
    // The NoopCredentialStore satisfies the Arc<dyn CredentialStore> constructor parameter.
    let engine = QueryEngine::new_with_cache_config(
        Arc::new(prism_sensors::AdapterRegistry::new()),
        Arc::new(NoopCredentialStore),
        Arc::new(OcsfNormalizer::new()),
        Arc::new(ClientRegistry::new(vec![])),
        QueryEngineConfig::default(),
        CacheConfig::default(),
    )
    .with_table_registry(registry);

    // Table registered in DataFusion: "{sensor_id}_{table_name}" = "claroty_claroty_vulnerabilities".
    // `vulnerability_type` is a Tier-2 column (no ocsf_field) and is NOT in
    // ocsf_projected_column_names. The plan-time check_query_column_availability gate
    // MUST raise E-QUERY-038 (PrismError::ColumnNotFound).
    let result = engine
        .execute(
            "SELECT vulnerability_type FROM claroty_claroty_vulnerabilities",
            QueryOptions::default(),
        )
        .await;

    // LOAD-BEARING SAP-3 assertion: must fail at plan time with E-QUERY-038.
    assert!(
        result.is_err(),
        "SAP-3 LOAD-BEARING: QueryEngine::execute must return Err when a \
         Tier-2 column ('vulnerability_type') is queried directly. \
         Got Ok. BC-2.16.015 AC-003; SAP-3."
    );

    let err = result.unwrap_err();

    match &err {
        PrismError::ColumnNotFound(details) => {
            // The queried Tier-2 column must be named in the error.
            assert_eq!(
                details.column, "vulnerability_type",
                "SAP-3: ColumnNotFound.column must be 'vulnerability_type'. \
                 Got: {:?}. BC-2.16.015 AC-003.",
                details.column
            );
            let avail = &details.available_columns;
            // raw_extensions (Tier-2 aggregate) must be listed as available.
            assert!(
                avail.contains(&"raw_extensions".to_string()),
                "SAP-3: available_columns must include 'raw_extensions' \
                 (ADR-058 §J6 Tier-2 aggregate). Got: {:?}",
                avail
            );
            // Tier-1 OCSF projected column names must be available.
            assert!(
                avail.contains(&"finding_info_title".to_string()),
                "SAP-3: available_columns must include 'finding_info_title' \
                 (name → ocsf_field=finding_info.title → arrow name). Got: {:?}",
                avail
            );
            assert!(
                avail.contains(&"message".to_string()),
                "SAP-3: available_columns must include 'message' \
                 (description → ocsf_field=message). Got: {:?}",
                avail
            );
            // vulnerability_type is Tier-2 and MUST NOT appear in available_columns.
            assert!(
                !avail.contains(&"vulnerability_type".to_string()),
                "SAP-3: 'vulnerability_type' is Tier-2 and MUST NOT appear in \
                 available_columns (it belongs inside raw_extensions). Got: {:?}",
                avail
            );
        }
        other => {
            panic!(
                "SAP-3 LOAD-BEARING: QueryEngine::execute must return \
                 PrismError::ColumnNotFound (E-QUERY-038) when a Tier-2 column is queried \
                 directly. Got: {:?}. BC-2.16.015 AC-003; SAP-3.",
                other
            );
        }
    }
}
