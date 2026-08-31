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
//! | F-VULNS-EC009-001 | test_BC_2_16_015_claroty_vulnerabilities_e2e_e_query_038_id_column | PrismError::ColumnNotFound (E-QUERY-038) when querying Tier-2 `id` column (source_path="$.id") via QueryEngine::execute() |
//! | F-VULNS-ADV-003 | test_BC_2_16_015_claroty_vulnerabilities_production_mcp_serializer_uses_explicit_nulls_true | Source-scan guard: prism-mcp/src/server.rs MUST use with_explicit_nulls(true) in RecordBatch→JSON path |
//! | F-L2-001     | test_BC_2_16_015_claroty_vulnerabilities_ec005_empty_cve_ids_wire_serialized_json | Wire-level: cve_ids=[] serializes as a native JSON array `[]` in raw_extensions (ENRICH-1 DD-2 Json arm; not null, not absent, not stringified) |
//! | F-L2-003     | test_BC_2_16_015_claroty_vulnerabilities_ec004_advisory_title_verbatim_wire | Wire-level: advisory-title name preserved verbatim in finding_info_title serialized JSON (no normalization) |
//! | F-VULNS-ADV-002 | test_BC_2_16_015_claroty_vulnerabilities_ec008_non_200_e_sensor_001 | SensorError::HttpError(E-SENSOR-001) + body excerpt in rendered error on HTTP 500 |
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
    BearerStaticSensorAuth, SensorAdapter, SensorError, adapter::QueryParams,
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
/// for RG-004b (which is `#[ignore]`'d pending a live Claroty instance at CLAROTY_INSTANCE_URL).
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
/// Story: S-CLAROTY-VULNS-001 RG-004b (non-live SID-1 coverage).
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

    // source_table = "claroty_vulnerabilities":
    //   sensor_id = "claroty", table_name = "vulnerabilities" (F-VULNS-P5-001 fix: bare name).
    //   Registered in DataFusion as: format!("{sensor_id}_{table_name}") = "claroty_vulnerabilities".
    let adapter_spec = SensorAdapterSpec {
        source_table: "claroty_vulnerabilities".to_string(),
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
        !batches.batches.is_empty(),
        "fetch() must return at least one RecordBatch for a non-empty response. \
         BC-2.16.015 AC-004."
    );

    let first_batch = &batches.batches[0];
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

    // ── Wire-shape assertion 7: `id` source_path extraction lands with exact value ──
    // LOAD-BEARING AC-008 positive assertion: the `id` column uses `source_path = "$.id"`
    // (root-level scalar JSONPath). The seeded record carries `"id": "mock-vuln-wire-001"`.
    // This assertion verifies the non-wildcard scalar extraction arm fires and lands the
    // EXACT value in raw_extensions. A regression breaking this arm silently drops `id`
    // from raw_extensions with no other test going red — the false-green risk identified
    // as MED finding FINDING-1 in the LOCAL adversary cascade (passes 1 and 3).
    assert_eq!(
        raw_ext_obj.get("id"),
        Some(&serde_json::json!("mock-vuln-wire-001")),
        "Wire-shape assertion 7 LOAD-BEARING (AC-008): 'id' MUST be present in \
         raw_extensions with the exact seeded value 'mock-vuln-wire-001'. \
         Column 'id' uses source_path = '$.id' (root-level scalar JSONPath extraction). \
         S-CLAROTY-VULNS-001 §Edge Cases EC-002; BC-2.16.015 AC-008; ADR-058 §J6.",
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
        source_table: "claroty_vulnerabilities".to_string(),
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
        !batches.batches.is_empty(),
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
    for batch in &batches.batches {
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

    // LOAD-BEARING AC-008 positive assertion: verify `id` is present in row0 raw_extensions
    // with the EXACT seeded value. `id` uses source_path = "$.id" (scalar JSONPath extraction).
    // A regression dropping `id` would pass the `has_tier2` check above (any-match) silently.
    assert_eq!(
        raw_ext_obj0.get("id"),
        Some(&serde_json::json!("mock-wire-json-001")),
        "F-VULNS-P1-001 LOAD-BEARING (AC-008): 'id' MUST be present in row0 raw_extensions \
         with exact seeded value 'mock-wire-json-001'. source_path = '$.id' scalar extraction. \
         S-CLAROTY-VULNS-001 §Edge Cases EC-002; BC-2.16.015 AC-008.",
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
///   4. `engine.execute("SELECT vulnerability_type FROM claroty_vulnerabilities", ...)`
///      — enters via the public query surface, fires E-QUERY-038
///
/// `vulnerability_type` is Tier-2 (no ocsf_field in claroty.sensor.toml):
///   - NOT in ocsf_projected_column_names → NOT in TableRegistry → E-QUERY-038
///   - available_columns contains "raw_extensions", "finding_info_title", "message"
///   - available_columns does NOT contain "vulnerability_type"
///
/// Registered table name: `{sensor_id}_{table_name}` = `claroty_vulnerabilities`
/// (sensor_id="claroty", table_name="vulnerabilities" per TOML, F-VULNS-P5-001 fix).
///
/// No HTTP requests are issued — E-QUERY-038 fires at plan-time before any fan-out.
///
/// BC-2.16.015 AC-003; SAP-3; ADR-058 §I7; S-ADR058-OCSF-ROUTING-001.
/// Story: S-CLAROTY-VULNS-001 RG-003a (SAP-3 end-to-end gate).
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
    // For claroty_vulnerabilities:
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

    // Table registered in DataFusion: "{sensor_id}_{table_name}" = "claroty_vulnerabilities".
    // `vulnerability_type` is a Tier-2 column (no ocsf_field) and is NOT in
    // ocsf_projected_column_names. The plan-time check_query_column_availability gate
    // MUST raise E-QUERY-038 (PrismError::ColumnNotFound).
    let result = engine
        .execute(
            "SELECT vulnerability_type FROM claroty_vulnerabilities",
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
            // F-VULNS-AC003-001: synthesized pseudo-columns must also be listed.
            // class_uid is the OCSF event-class synthesized column (AC-003 + E-QUERY-001).
            assert!(
                avail.contains(&"class_uid".to_string()),
                "F-VULNS-AC003-001: available_columns must include 'class_uid' \
                 (OCSF synthesized pseudo-column). Got: {:?}. \
                 BC-2.16.015 AC-003; BC Error Case E-QUERY-001.",
                avail
            );
            // _sensor is the per-row sensor-metadata synthesized pseudo-column.
            assert!(
                avail.contains(&"_sensor".to_string()),
                "F-VULNS-AC003-001: available_columns must include '_sensor' \
                 (synthesized sensor-metadata pseudo-column). Got: {:?}. \
                 BC-2.16.015 AC-003; BC Error Case E-QUERY-001.",
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

// ---------------------------------------------------------------------------
// F-VULNS-EC009-001: EC-009 SELECT id → E-QUERY-038 (id-specific reachability)
// ---------------------------------------------------------------------------

/// SAP-3 reachability test for EC-009: querying the `id` column directly via the
/// REAL `QueryEngine::execute()` surface raises `PrismError::ColumnNotFound`
/// (E-QUERY-038), with `id` NOT in `available_columns`.
///
/// `id` is a Tier-2 column declared with `source_path = "$.id"` (no `ocsf_field`).
/// It lives inside `raw_extensions` and is NOT a queryable Arrow column.
/// Story §Edge Cases EC-009 asserts that `SELECT id FROM claroty_vulnerabilities` fires
/// E-QUERY-038 at plan-time, exactly as any other Tier-2 column would.
///
/// This test closes the id-specific SAP-3 reachability gap identified in
/// F-VULNS-EC009-001 (pass-3 adversary finding): the existing SAP-3 test only
/// exercised `vulnerability_type`; `id` has a different semantic shape (source_path
/// extraction vs. simple field mapping) and its E-QUERY-038 arm was not end-to-end
/// covered from the public query surface.
///
/// No HTTP requests are issued — E-QUERY-038 fires at plan-time before any fan-out.
///
/// S-CLAROTY-VULNS-001 §Edge Cases EC-009; BC-2.16.015 §Error Cases E-QUERY-001/E-QUERY-038;
/// SAP-3; ADR-058 §I7.
/// Story: S-CLAROTY-VULNS-001 F-VULNS-EC009-001 (pass-3 fix-burst).
#[tokio::test]
async fn test_BC_2_16_015_claroty_vulnerabilities_e2e_e_query_038_id_column() {
    let spec_content = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../prism-sensors/specs/claroty.sensor.toml"),
    )
    .expect(
        "EC-009 test: claroty.sensor.toml must be readable from \
         CARGO_MANIFEST_DIR/../prism-sensors/specs/",
    );
    let spec =
        SpecLoader::parse(&spec_content).expect("EC-009 test: claroty.sensor.toml must parse");

    let registry = Arc::new(TableRegistry::new());
    registry
        .register_sensor(&spec)
        .expect("EC-009 test: register_sensor must not fail for production claroty.sensor.toml");

    let engine = QueryEngine::new_with_cache_config(
        Arc::new(prism_sensors::AdapterRegistry::new()),
        Arc::new(NoopCredentialStore),
        Arc::new(OcsfNormalizer::new()),
        Arc::new(ClientRegistry::new(vec![])),
        QueryEngineConfig::default(),
        CacheConfig::default(),
    )
    .with_table_registry(registry);

    // `id` is a Tier-2 column (source_path = "$.id", no ocsf_field).
    // It lives inside raw_extensions and MUST NOT be queryable as a top-level Arrow column.
    // E-QUERY-038 (PrismError::ColumnNotFound) must fire at plan-time.
    let result = engine
        .execute(
            "SELECT id FROM claroty_vulnerabilities LIMIT 1",
            QueryOptions::default(),
        )
        .await;

    // LOAD-BEARING EC-009 assertion: must fail at plan-time with E-QUERY-038.
    assert!(
        result.is_err(),
        "F-VULNS-EC009-001 LOAD-BEARING: QueryEngine::execute must return Err when \
         Tier-2 column 'id' is queried directly (E-QUERY-038). \
         Got Ok. BC-2.16.015 EC-009; SAP-3.",
    );

    let err = result.unwrap_err();

    match &err {
        PrismError::ColumnNotFound(details) => {
            assert_eq!(
                details.column, "id",
                "F-VULNS-EC009-001: ColumnNotFound.column must be 'id'. \
                 Got: {:?}. BC-2.16.015 EC-009.",
                details.column
            );
            let avail = &details.available_columns;
            // `id` lives inside raw_extensions — MUST NOT appear as a queryable column.
            assert!(
                !avail.contains(&"id".to_string()),
                "F-VULNS-EC009-001: 'id' MUST NOT appear in available_columns — \
                 it is Tier-2 and lives inside raw_extensions. Got: {:?}. \
                 BC-2.16.015 EC-009.",
                avail
            );
            // raw_extensions must be listed as the container that holds `id`.
            assert!(
                avail.contains(&"raw_extensions".to_string()),
                "F-VULNS-EC009-001: available_columns must include 'raw_extensions' \
                 (the Tier-2 container that holds 'id' and other Tier-2 fields). \
                 Got: {:?}.",
                avail
            );
        }
        other => {
            panic!(
                "F-VULNS-EC009-001 LOAD-BEARING: QueryEngine::execute must return \
                 PrismError::ColumnNotFound (E-QUERY-038) when 'id' (Tier-2, \
                 source_path='$.id') is queried directly. \
                 Got: {:?}. BC-2.16.015 EC-009; SAP-3.",
                other
            );
        }
    }
}

// ---------------------------------------------------------------------------
// F-VULNS-ADV-003: Production MCP serialization config guard
// ---------------------------------------------------------------------------

/// Source-scan guard: the production MCP serialization path in
/// `crates/prism-mcp/src/server.rs` MUST use `with_explicit_nulls(true)` when
/// serializing RecordBatches to JSON for tool call responses.
///
/// ## Why source-scan, not call-level
///
/// The production serialization is embedded inside `PrismServer::call_tool` —
/// a method that requires a wired `QueryEngine`, RocksDB, and MCP server
/// infrastructure not available in unit tests.  A call-level test would need
/// the full binary (`prism start`).
///
/// A source-scan guard provides a lighter-weight but equally effective ratchet:
/// it fails immediately if `with_explicit_nulls(true)` is changed to `false`,
/// removed, or the writer is replaced with one that omits the call — catching
/// the C3/H20 defect class (BC-2.11.001 EC-11-079) before CI runs.
///
/// This test strengthens `test_BC_2_16_015_claroty_vulnerabilities_wire_shape_serialized_json_explicit_nulls`,
/// which builds its own `WriterBuilder::with_explicit_nulls(true)` and would NOT
/// detect a regression in the PRODUCTION server.rs configuration (the test
/// re-implements the builder locally instead of calling the server path).
///
/// ## What this guards
///
/// - `arrow_json::writer::WriterBuilder::new().with_explicit_nulls(true)` must
///   appear in the production `server.rs` serialization block.
/// - If this setting is changed to `false` (or removed), the `finding_info_title`
///   key would be **absent** from JSON rows where `name` was missing in the API
///   response — the C3/H20 null-not-absent defect class.
///
/// BC-2.16.015 AC-004; BC-2.11.001 EC-11-079 (null-not-absent);
/// CLAUDE.md §Wire-shape assertion discipline.
/// Story: S-CLAROTY-VULNS-001 F-VULNS-ADV-003 (pass-4 fix-burst).
#[test]
fn test_BC_2_16_015_claroty_vulnerabilities_production_mcp_serializer_uses_explicit_nulls_true() {
    // Read the production server.rs source.
    // Path: crates/prism-mcp/src/server.rs relative to the prism-bin CARGO_MANIFEST_DIR.
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR must be set by cargo during test execution");
    let server_rs_path = std::path::Path::new(&manifest_dir)
        .join("..")
        .join("prism-mcp")
        .join("src")
        .join("server.rs");

    let server_src = std::fs::read_to_string(&server_rs_path).unwrap_or_else(|e| {
        panic!(
            "F-VULNS-ADV-003: Could not read prism-mcp/src/server.rs at {:?}: {}. \
             Ensure the test runs from the prism-bin crate directory.",
            server_rs_path, e
        )
    });

    // Find the explicit_nulls configuration site.
    // Production form (BC-2.11.001 EC-11-079 CRIT-1 fix):
    //   arrow_json::writer::WriterBuilder::new()
    //       .with_explicit_nulls(true)
    //       .build::<_, arrow_json::writer::JsonArray>(&mut buf)
    assert!(
        server_src.contains("with_explicit_nulls(true)"),
        "F-VULNS-ADV-003 LOAD-BEARING: prism-mcp/src/server.rs MUST contain \
         'with_explicit_nulls(true)' in the production RecordBatch → JSON serialization \
         path (BC-2.11.001 EC-11-079 CRIT-1 fix). \
         Changing this to 'false' or removing it causes NULL-valued Arrow cells to be \
         ABSENT from JSON row objects — the C3/H20 null-not-absent defect class. \
         Source path checked: {server_rs_path:?}"
    );

    // Verify the setting is NOT `with_explicit_nulls(false)`.
    // This catches a regression where someone adds a disabled form alongside the enabled one.
    // NOTE: a comment containing `with_explicit_nulls(false)` as documentation text would
    // also match this assertion — acceptable false-positive risk given this codebase's
    // conventions (doc comments use backtick form, not the raw API call form).
    let false_count = server_src.matches("with_explicit_nulls(false)").count();
    assert_eq!(
        false_count, 0,
        "F-VULNS-ADV-003: prism-mcp/src/server.rs MUST NOT contain \
         'with_explicit_nulls(false)' — that would disable null-not-absent protection. \
         Found {} occurrence(s). BC-2.11.001 EC-11-079.",
        false_count
    );
}

// ---------------------------------------------------------------------------
// F-L2-001: EC-016-015-005 — empty cve_ids wire assertion
// ---------------------------------------------------------------------------

/// Wire-level assertion for EC-016-015-005: when `cve_ids` is an EMPTY JSON array `[]`
/// in the Claroty xDome API response, the downstream ENRICH-1 DD-2 transformation in
/// `pipeline_result_to_record_batch` (`serde_json::Value::Array(arr)` arm) preserves it
/// as a NATIVE JSON array `[]` because `cve_ids` has `column_type = "json"` (BC-2.16.015 v1.9).
///
/// At the wire level (MCP serialized JSON output):
///   - `raw_extensions` (StringArray cell) deserializes to a JSON object
///   - The key `cve_ids` MUST be PRESENT in that object (not absent)
///   - Its value MUST be a NATIVE JSON array `[]` (not JSON null, not the string "[]",
///     not the string "null")
///
/// This closes the empty-vs-null regression gap identified in F-L2-001:
/// the existing `test_BC_2_16_015_claroty_vulnerabilities_ec005_cve_ids_empty_array_in_raw_extensions`
/// (prism-sensors) only asserts the pre-conversion form (`Value::Array(vec![])` in
/// `map_record`). The DD-2 arm native-array preservation path (Json branch) was not
/// asserted at the wire level before this test.
///
/// Wire path: `SpecDrivenSensorAdapter::fetch()` →
///   `pipeline_result_to_record_batch` (OCSF mode, raw_extensions block, ENRICH-1 DD-2) →
///   `arrow_json::writer::WriterBuilder::new().with_explicit_nulls(true)` →
///   serialized JSON row with `raw_extensions` as a JSON-string column.
///
/// BC-2.16.015 §EC-016-015-005; ENRICH-1 DD-2; CLAUDE.md §Wire-shape assertion discipline.
/// Story: S-CLAROTY-VULNS-001 F-L2-001 (diverse-lens batch fix-burst).
#[tokio::test]
async fn test_BC_2_16_015_claroty_vulnerabilities_ec005_empty_cve_ids_wire_serialized_json() {
    let mock_server = MockServer::start().await;

    // Record with cve_ids as an EMPTY JSON array [].
    // ENRICH-1 DD-2 arm (Json branch): Value::Array(vec![]) → Value::Array(vec![]) preserved
    // as a native JSON array because cve_ids has column_type = "json" (BC-2.16.015 v1.9).
    Mock::given(method("POST"))
        .and(path("/api/v1/vulnerabilities/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "vulnerabilities": [{
                "name": "CVE-2024-EMPTY-CVEIDS",
                "description": "A mock vulnerability with empty cve_ids",
                "vulnerability_type": "CVE",
                "cve_ids": [],
                "cvss_v3_score": 7.5_f64,
                "id": "mock-empty-cveids-001"
            }],
            "total": 1_u32,
            "page": 1_u32
        })))
        .mount(&mock_server)
        .await;

    let adapter = make_claroty_adapter(&mock_server.uri());

    let adapter_spec = SensorAdapterSpec {
        source_table: "claroty_vulnerabilities".to_string(),
        org_id: OrgId::from_uuid(uuid::Uuid::now_v7()),
        #[allow(deprecated)]
        client_id: "claroty-vulns-ec005-wire-test".to_string(),
        sensor_config: serde_json::json!({}),
    };

    let params = QueryParams {
        cursor: None,
        limit: 10,
        start_time: None,
        end_time: None,
        filters: Default::default(),
    };

    let sensor_auth = BearerStaticSensorAuth::new("mock-bearer-token-ec005");

    let batches = adapter
        .fetch(&adapter_spec, &params, &sensor_auth)
        .await
        .expect(
            "F-L2-001: fetch() must succeed for a valid response with empty cve_ids array. \
             BC-2.16.015 §EC-016-015-005.",
        );

    assert!(
        !batches.batches.is_empty(),
        "F-L2-001: fetch() must return at least one RecordBatch. BC-2.16.015 §EC-016-015-005."
    );

    // Serialize through the production MCP path (explicit_nulls=true).
    let mut buf: Vec<u8> = Vec::new();
    let mut writer = arrow_json::writer::WriterBuilder::new()
        .with_explicit_nulls(true)
        .build::<_, arrow_json::writer::JsonArray>(&mut buf);
    for batch in &batches.batches {
        writer
            .write(batch)
            .expect("F-L2-001: arrow_json write must not fail for cve_ids=[] RecordBatch");
    }
    writer
        .finish()
        .expect("F-L2-001: arrow_json finish must not fail");

    let json_rows: Vec<serde_json::Value> = serde_json::from_slice::<Vec<serde_json::Value>>(&buf)
        .expect("F-L2-001: arrow_json output must deserialize as a JSON array of row objects");

    assert_eq!(
        json_rows.len(),
        1,
        "F-L2-001: serialized JSON must contain exactly 1 row. BC-2.16.015 §EC-016-015-005."
    );

    let row = &json_rows[0];

    // raw_extensions must be present as a JSON string.
    let raw_ext_str = row.get("raw_extensions").and_then(|v| v.as_str()).expect(
        "F-L2-001: raw_extensions must be present and a JSON string in the wire row. \
             BC-2.16.015 §EC-016-015-005; ADR-058 §J6.",
    );

    let raw_ext_obj: serde_json::Value = serde_json::from_str(raw_ext_str)
        .expect("F-L2-001: raw_extensions string must be valid JSON (deserializable as object)");
    assert!(
        raw_ext_obj.is_object(),
        "F-L2-001: raw_extensions must deserialize to a JSON object. Got: {:?}.",
        raw_ext_obj
    );

    // LOAD-BEARING EC-005 wire assertion: cve_ids must be PRESENT in raw_extensions
    // (not absent) and its value must be a NATIVE JSON array [] (not JSON null, not the
    // string "[]"). BC-2.16.015 v1.9: cve_ids column_type = "json" → DD-2 Json branch
    // preserves Value::Array(vec![]) as-is in raw_extensions.
    let cve_ids_wire = raw_ext_obj.get("cve_ids");
    assert!(
        cve_ids_wire.is_some(),
        "F-L2-001 LOAD-BEARING: 'cve_ids' key MUST be PRESENT in raw_extensions JSON object \
         even when the API returned an empty array. \
         Got absent (raw_extensions keys: {:?}). \
         BC-2.16.015 §EC-016-015-005.",
        raw_ext_obj
            .as_object()
            .map(|o| o.keys().collect::<Vec<_>>())
    );
    assert_ne!(
        cve_ids_wire,
        Some(&serde_json::Value::Null),
        "F-L2-001 LOAD-BEARING: 'cve_ids' MUST NOT be JSON null when API returned []. \
         ENRICH-1 DD-2 Json branch preserves empty arrays as native [], not null. \
         BC-2.16.015 §EC-016-015-005."
    );
    assert_eq!(
        cve_ids_wire,
        Some(&serde_json::Value::Array(vec![])),
        "F-L2-001 LOAD-BEARING: 'cve_ids' MUST equal a native JSON array [] in raw_extensions. \
         ENRICH-1 DD-2 Json branch: column_type=json preserves Value::Array as-is (BC-2.16.015 v1.9). \
         Got: {:?}. BC-2.16.015 §EC-016-015-005.",
        cve_ids_wire
    );
}

// ---------------------------------------------------------------------------
// F-L2-003: EC-016-015-004 — advisory-title verbatim at wire level
// ---------------------------------------------------------------------------

/// Wire-level assertion for EC-016-015-004: a vulnerability record whose `name` is an
/// advisory-title format (e.g., "ICSMA-21-161-01 (ZOLL Defibrillator Dashboard)") — NOT
/// a CVE-YYYY-NNNNN format — is preserved VERBATIM in the serialized `finding_info_title`
/// key of the wire JSON row.  No normalization is applied at any stage of the pipeline.
///
/// The existing EC-004 test in prism-sensors
/// (`test_BC_2_16_015_claroty_vulnerabilities_ec004_advisory_title_preserved_verbatim`)
/// asserts verbatim preservation at the `ColumnMapper::map_record` boundary (dot-form
/// intermediate: `finding_info.title`).  This test closes the remaining gap identified in
/// F-L2-003: EC-004 was NOT asserted at the wire level (the serialized MCP JSON that an
/// LLM agent actually consumes).
///
/// Wire path: `SpecDrivenSensorAdapter::fetch()` → `pipeline_result_to_record_batch`
/// (OCSF mode, finding_info_title = dot→underscore flattening of finding_info.title) →
/// `arrow_json::writer::WriterBuilder::new().with_explicit_nulls(true)` →
/// serialized JSON row with `finding_info_title` as a top-level string key.
///
/// Assertion: the serialized `finding_info_title` value MUST equal the advisory-title
/// string VERBATIM — no uppercase normalization, no parenthesis stripping, no CVE
/// re-formatting applied.
///
/// BC-2.16.015 §EC-016-015-004; CLAUDE.md §Wire-shape assertion discipline.
/// Story: S-CLAROTY-VULNS-001 F-L2-003 (diverse-lens batch fix-burst).
#[tokio::test]
async fn test_BC_2_16_015_claroty_vulnerabilities_ec004_advisory_title_verbatim_wire() {
    let mock_server = MockServer::start().await;

    // Advisory-title format per BC-2.16.015 §EC-016-015-004 exemplar.
    // NOT a CVE-YYYY-NNNNN identifier — pipeline must not apply any transformation.
    let advisory_title = "ICSMA-21-161-01 (ZOLL Defibrillator Dashboard)";

    Mock::given(method("POST"))
        .and(path("/api/v1/vulnerabilities/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "vulnerabilities": [{
                "name": advisory_title,
                "description": "ICS Medical Advisory for ZOLL Defibrillator Dashboard",
                "vulnerability_type": "ICS-Advisory",
                "cvss_v3_score": 9.4_f64
            }],
            "total": 1_u32,
            "page": 1_u32
        })))
        .mount(&mock_server)
        .await;

    let adapter = make_claroty_adapter(&mock_server.uri());

    let adapter_spec = SensorAdapterSpec {
        source_table: "claroty_vulnerabilities".to_string(),
        org_id: OrgId::from_uuid(uuid::Uuid::now_v7()),
        #[allow(deprecated)]
        client_id: "claroty-vulns-ec004-wire-test".to_string(),
        sensor_config: serde_json::json!({}),
    };

    let params = QueryParams {
        cursor: None,
        limit: 10,
        start_time: None,
        end_time: None,
        filters: Default::default(),
    };

    let sensor_auth = BearerStaticSensorAuth::new("mock-bearer-token-ec004-wire");

    let batches = adapter
        .fetch(&adapter_spec, &params, &sensor_auth)
        .await
        .expect(
            "F-L2-003: fetch() must succeed for a record with an advisory-title name. \
             BC-2.16.015 §EC-016-015-004.",
        );

    assert!(
        !batches.batches.is_empty(),
        "F-L2-003: fetch() must return at least one RecordBatch. BC-2.16.015 §EC-016-015-004."
    );

    // Serialize through the production MCP path.
    let mut buf: Vec<u8> = Vec::new();
    let mut writer = arrow_json::writer::WriterBuilder::new()
        .with_explicit_nulls(true)
        .build::<_, arrow_json::writer::JsonArray>(&mut buf);
    for batch in &batches.batches {
        writer
            .write(batch)
            .expect("F-L2-003: arrow_json write must not fail for advisory-title RecordBatch");
    }
    writer
        .finish()
        .expect("F-L2-003: arrow_json finish must not fail");

    let json_rows: Vec<serde_json::Value> = serde_json::from_slice::<Vec<serde_json::Value>>(&buf)
        .expect("F-L2-003: arrow_json output must deserialize as a JSON array of row objects");

    assert_eq!(
        json_rows.len(),
        1,
        "F-L2-003: serialized JSON must contain exactly 1 row. BC-2.16.015 §EC-016-015-004."
    );

    let row = &json_rows[0];

    // LOAD-BEARING EC-004 wire assertion: finding_info_title MUST equal advisory_title VERBATIM.
    // No normalization may be applied at any stage: not in ColumnMapper::map_record
    // (dot-form intermediate), not in pipeline_result_to_record_batch (Arrow StringArray),
    // and not in arrow_json serialization (JSON string).
    let wire_title = row
        .get("finding_info_title")
        .and_then(|v| v.as_str())
        .expect(
            "F-L2-003 LOAD-BEARING: 'finding_info_title' key must be present and a string \
             in the wire JSON row (advisory-title → ocsf_field=finding_info.title → \
             ADR-058 arrow name finding_info_title). \
             BC-2.16.015 §EC-016-015-004.",
        );

    assert_eq!(
        wire_title, advisory_title,
        "F-L2-003 LOAD-BEARING: 'finding_info_title' MUST equal the advisory-title string \
         VERBATIM in the serialized wire JSON. No normalization (uppercase, CVE re-format, \
         parenthesis stripping) may be applied. \
         Expected: {:?}. Got: {:?}. BC-2.16.015 §EC-016-015-004.",
        advisory_title, wire_title
    );
}

// ---------------------------------------------------------------------------
// F-VULNS-ADV-002: EC-008 — non-200 HTTP → E-SENSOR-001 structured error
// ---------------------------------------------------------------------------

/// BC-2.16.015 §Error Cases EC-008: when the Claroty xDome API returns a non-200
/// HTTP status for POST /api/v1/vulnerabilities/, `SpecDrivenSensorAdapter::fetch()`
/// MUST surface `SensorError::HttpError { status }` with `error_code() == "E-SENSOR-001"`,
/// AND the rendered error string MUST contain the response body excerpt so that
/// downstream callers (sensor=claroty, status, body) have full context.
///
/// Pattern mirrors the sibling audit_logs EC-008 assertion in
/// `bc_2_01_013_claroty_audit_logs_layer2.rs` (RG-005), which asserts
/// `rendered.contains("invalid filter")` and a single-`HTTP`-prefix regression guard.
///
/// Assertions:
///   1. `fetch()` returns `Err` (not Ok)
///   2. The error is `SensorError::HttpError { status: 500 }`
///   3. `err.error_code()` == "E-SENSOR-001"
///   4. `rendered` CONTAINS the body excerpt "vulnerability service temporarily unavailable"
///      (BC-2.16.015 §Error Cases EC-008 "sensor=claroty, status, BODY")
///   5. `rendered.matches("HTTP").count() == 1` (single-HTTP-prefix regression guard,
///      mirrors sibling F-P37-HIGH-001 double-prefix guard in bc_2_01_013)
///
/// No HTTP requests are retried — a non-200 response causes immediate Err.
/// No pagination halts — the error surfaces before any pagination logic.
///
/// SID-1 compliance: no `#[ignore]`; wiremock mock, no live Claroty DTU needed.
///
/// BC-2.16.015 EC-008; prism-sensors::SensorError::HttpError (E-SENSOR-001).
/// Story: S-CLAROTY-VULNS-001 F-VULNS-P5-003 (pass-5 fix-burst).
#[tokio::test]
async fn test_BC_2_16_015_claroty_vulnerabilities_ec008_non_200_e_sensor_001() {
    let mock_server = MockServer::start().await;

    // Return HTTP 500 Internal Server Error for the vulnerabilities POST.
    // The recognizable body string "vulnerability service temporarily unavailable" is
    // the excerpt asserted below — proving sensor=claroty + status + BODY are all
    // threaded through the error chain per BC-2.16.015 §Error Cases EC-008.
    Mock::given(method("POST"))
        .and(path("/api/v1/vulnerabilities/"))
        .respond_with(ResponseTemplate::new(500).set_body_string(
            r#"{"error": "Internal Server Error", "message": "vulnerability service temporarily unavailable"}"#,
        ))
        .mount(&mock_server)
        .await;

    let adapter = make_claroty_adapter(&mock_server.uri());

    let adapter_spec = SensorAdapterSpec {
        source_table: "claroty_vulnerabilities".to_string(),
        org_id: OrgId::from_uuid(uuid::Uuid::now_v7()),
        #[allow(deprecated)]
        client_id: "claroty-vulns-ec008-test".to_string(),
        sensor_config: serde_json::json!({}),
    };

    let params = QueryParams {
        cursor: None,
        limit: 10,
        start_time: None,
        end_time: None,
        filters: Default::default(),
    };

    let sensor_auth = BearerStaticSensorAuth::new("mock-bearer-token-ec008");

    let result = adapter.fetch(&adapter_spec, &params, &sensor_auth).await;

    // LOAD-BEARING EC-008 assertion: non-200 must surface as E-SENSOR-001.
    assert!(
        result.is_err(),
        "EC-008 LOAD-BEARING: fetch() must return Err when xDome returns HTTP 500. \
         Got Ok. BC-2.16.015 EC-008."
    );

    let err = result.unwrap_err();

    match &err {
        SensorError::HttpError { status, .. } => {
            assert_eq!(
                *status, 500u16,
                "EC-008: SensorError::HttpError status must be 500. \
                 Got: {}. BC-2.16.015 EC-008.",
                status
            );
        }
        other => {
            panic!(
                "EC-008 LOAD-BEARING: fetch() must return SensorError::HttpError{{500}} \
                 for a 500 xDome /api/v1/vulnerabilities/ response. Got: {other:?}. \
                 BC-2.16.015 EC-008."
            );
        }
    }

    assert_eq!(
        err.error_code(),
        "E-SENSOR-001",
        "EC-008: error code must be E-SENSOR-001. Got: {:?}. BC-2.16.015 EC-008.",
        err.error_code()
    );

    let rendered = err.to_string();
    assert!(
        rendered.contains("500"),
        "EC-008: rendered error MUST contain the HTTP status '500'. \
         Got: {rendered}. BC-2.16.015 EC-008."
    );
    // LOAD-BEARING body-excerpt assertion (F-VULNS-P5-003):
    // BC-2.16.015 §Error Cases EC-008 requires "sensor=claroty, status, BODY" —
    // the response body must be threaded into the rendered error so callers have full context.
    // Mirrors sibling bc_2_01_013_claroty_audit_logs_layer2.rs RG-005 assertion on
    // rendered.contains("invalid filter").
    assert!(
        rendered.contains("vulnerability service temporarily unavailable"),
        "EC-008 LOAD-BEARING (body-excerpt): rendered error MUST contain the response \
         body excerpt 'vulnerability service temporarily unavailable'. \
         map_spec_engine_error_to_sensor_error must thread the HttpError body through. \
         Got: {rendered}. BC-2.16.015 EC-008."
    );
    // Single-HTTP-prefix regression guard (mirrors sibling F-P37-HIGH-001 guard):
    // The rendered error must contain exactly ONE 'HTTP' occurrence — no double-prefix.
    assert_eq!(
        rendered.matches("HTTP").count(),
        1,
        "EC-008 double-prefix regression guard: rendered error must contain exactly 1 \
         'HTTP' occurrence (mirrors sibling RG-005 F-P37-HIGH-001 guard). \
         Got {} occurrences in: {rendered}",
        rendered.matches("HTTP").count()
    );
}
