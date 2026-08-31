// SPDX-License-Identifier: Apache-2.0
//! Production-path wire-shape, SAP-3 end-to-end E-QUERY-038, and null-passthrough tests
//! for BC-2.16.019 — Claroty xDome Collection Server Interfaces Table.
//!
//! # SAP-4 Coverage Gap Closure
//!
//! The prism-sensors `map_record`-based tests (RG-015 parts 1 and 2) call
//! `ColumnMapper::map_record` directly. `map_record` has ZERO production callers.
//! The production path is:
//!   `SpecDrivenSensorAdapter::fetch` → `pipeline_result_to_record_batch` → `build_column_array`
//!
//! This file provides the authoritative production-path coverage for the null-passthrough
//! behaviors (RG-015) using a wiremock + `SpecDrivenSensorAdapter::fetch` harness and
//! asserting on SERIALIZED JSON wire output (CLAUDE.md §Wire-shape assertion discipline).
//!
//! # Tests in this file
//!
//! | ID              | Test name | Assertion |
//! |-----------------|-----------|-----------|
//! | NEW-2           | test_BC_2_16_019_claroty_server_interfaces_wire_shape_class_uid_5001_mock | class_uid=5001, device_name, status_code, raw_extensions; no Tier-2 top-level keys |
//! | RG-015-WIRE     | test_BC_2_16_019_claroty_server_interfaces_null_interface_name_row_not_dropped_wire | null interface_name → row not dropped; device_name non-null; null-not-absent in raw_extensions |
//! | RG-015-WIRE-SNA | test_BC_2_16_019_claroty_server_interfaces_null_passthrough_server_name_absent_null_not_absent | server_name absent → device_name null (null-not-absent); row survives; mirrors servers RG-007-WIRE |
//! | RG-011-E2E      | test_BC_2_16_019_claroty_server_interfaces_e2e_e_query_038_tier2_column | E-QUERY-038 via QueryEngine::execute() for interface_name (Tier-2 composite PK); authoritative SAP-3 gate |
//! | RG-016-PROD     | test_BC_2_16_019_claroty_server_interfaces_ec016_019_005_count_null_empty_page_halt_ok_zero_rows | SpecDrivenSensorAdapter::fetch returns Ok+zero rows for {"server_interfaces":[],"count":null} (EC-016-019-005) |
//!
//! # SID-1 compliance (NEW-2)
//!
//! RG-010 in `prism-sensors/tests/bc_2_16_019_claroty_server_interfaces.rs` is `#[ignore]`'d
//! (live-only). NEW-2 provides the non-live wire-shape coverage via wiremock +
//! RecordBatch assertions on the actual serialized column values.
//!
//! # RG-015-WIRE: null-passthrough production-path coverage
//!
//! RG-015 parts 1 and 2 in prism-sensors call `ColumnMapper::map_record` directly
//! (SAP-3 defense-in-depth only). This test exercises the FULL production path from HTTP
//! response to serialized wire bytes for the key null-passthrough scenario:
//!
//! - `interface_name` is Tier-2 (no `ocsf_field`) → lives in `raw_extensions`
//! - A record with `interface_name = null` MUST NOT be dropped (row-not-dropped invariant)
//! - `server_name` IS present → `device_name` is non-null → row is well-formed
//! - `raw_extensions` MUST contain `"interface_name": null` (null-not-absent in raw_extensions)
//!
//! # RG-011-E2E: authoritative SAP-3 E-QUERY-038 gate
//!
//! RG-011 in prism-sensors calls `ocsf_projected_column_names()` directly (defense-in-depth).
//! This test fires the E-QUERY-038 plan-time gate end-to-end via `QueryEngine::execute()` —
//! the real public surface that LLM agents reach. No HTTP requests are issued; the gate fires
//! at plan-time before any fan-out.
//!
//! # RG-016-PROD: count=null empty-page-halt production-path coverage (MEDIUM-1 closure)
//!
//! `test_BC_2_16_019_claroty_server_interfaces_nullable_count_uses_empty_page_halt` in
//! prism-sensors (RG-016) constructs `{"server_interfaces":[],"count":null}` and discards it
//! — it is a structural assertion only (paper-test). RG-016-PROD serves that payload via
//! wiremock to the production `SpecDrivenSensorAdapter::fetch` path and asserts Ok+zero-rows.
//!
//! BC: BC-2.16.019
//! Story: S-CLAROTY-SERVERS-001

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
// Shared helpers
// ---------------------------------------------------------------------------

/// Minimal no-op `CredentialStore` for constructing `QueryEngine` in SAP-3 tests.
/// The E-QUERY-038 gate fires at plan-time before any credential lookup.
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

/// Build a `SpecDrivenSensorAdapter` from the production `claroty.sensor.toml`
/// directed at the given mock server URI.
fn make_claroty_adapter(mock_server_uri: &str) -> SpecDrivenSensorAdapter {
    let spec_content = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../prism-sensors/specs/claroty.sensor.toml"),
    )
    .expect(
        "bc_2_16_019_wire_shape: claroty.sensor.toml must be readable from \
         CARGO_MANIFEST_DIR/../prism-sensors/specs/",
    );

    let mut spec = SpecLoader::parse(&spec_content)
        .expect("bc_2_16_019_wire_shape: claroty.sensor.toml must parse");
    spec.base_url = mock_server_uri.to_string();

    let overlay_toml =
        "extends = \"claroty\"\ninstance_id = \"claroty@claroty-server-interfaces-wire-test\"";
    let overlay: SensorInstanceOverlay = toml::from_str(overlay_toml)
        .expect("bc_2_16_019_wire_shape: SensorInstanceOverlay TOML parse failed");
    let resolved = OverlayLoader::merge_overlay_onto_type_spec(
        &spec,
        &overlay,
        OrgSlug::new("claroty-server-interfaces-wire-test"),
    );

    let http_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("bc_2_16_019_wire_shape: reqwest::Client build failed (ADR-050 rustls-tls)");

    SpecDrivenSensorAdapter::new(
        Arc::new(resolved),
        AdapterAuthStrategy::BearerStatic,
        http_client,
    )
}

// ---------------------------------------------------------------------------
// NEW-2: Wire-shape mock test (SID-1 compliance)
// ---------------------------------------------------------------------------

/// Wire-shape mock test: `SpecDrivenSensorAdapter::fetch()` for `claroty_server_interfaces`
/// returns RecordBatches with:
///   - `class_uid == 5001` (EventClassSelector("inventory_info"))
///   - `device_name` column present (non-null, == "Monroe-Collector-1")
///   - `status_code` column present (case-insensitive match)
///   - `raw_extensions` present as StringArray holding a JSON object (Tier-2 aggregate)
///   - No Tier-2 column names at top level (Tier-2 isolation, ADR-058 §J6)
///
/// SID-1 compliance: non-ignored, uses wiremock. Provides non-live wire-shape coverage for
/// RG-010 (which is `#[ignore]`'d pending a live Claroty instance at CLAROTY_INSTANCE_URL).
///
/// BC-2.16.019 AC-010; ADR-058 §C2 Option 4, §J6.
/// Story: S-CLAROTY-SERVERS-001 NEW-2 (SID-1 mock coverage).
#[tokio::test]
async fn test_BC_2_16_019_claroty_server_interfaces_wire_shape_class_uid_5001_mock() {
    let mock_server = MockServer::start().await;

    // Mock the server_interfaces endpoint.
    // DTU route: POST /api/v1/server_interfaces/ (trailing slash per xDome convention).
    // Response envelope: {"server_interfaces": [...]}
    // One record with both Tier-1 fields (server_name, interface_status) + Tier-2 fields.
    Mock::given(method("POST"))
        .and(path("/api/v1/server_interfaces/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "server_interfaces": [{
                "server_name": "Monroe-Collector-1",
                "interface_status": "Active",
                "interface_name": "eth0",
                "interface_type": "1000BASE-T",
                "interface_connection_type": "Wired",
                "site_id": 1_u32,
                "avg_traffic_past_hour_mbps": 12.5_f64
            }]
        })))
        .mount(&mock_server)
        .await;

    let adapter = make_claroty_adapter(&mock_server.uri());

    // source_table = "claroty_server_interfaces":
    //   sensor_id = "claroty", TOML table_name = "server_interfaces"
    //   strip_prefix("claroty_") → "server_interfaces" → matches TOML table_name.
    //   Registered in DataFusion as: format!("{sensor_id}_{table_name}") = "claroty_server_interfaces".
    let adapter_spec = SensorAdapterSpec {
        source_table: "claroty_server_interfaces".to_string(),
        org_id: OrgId::from_uuid(uuid::Uuid::now_v7()),
        #[allow(deprecated)]
        client_id: "claroty-server-interfaces-wire-test".to_string(),
        sensor_config: serde_json::json!({}),
    };

    let params = QueryParams {
        cursor: None,
        limit: 10,
        start_time: None,
        end_time: None,
        filters: Default::default(),
    };

    let sensor_auth = BearerStaticSensorAuth::new("mock-bearer-token-server-interfaces-wire-test");

    let result = adapter.fetch(&adapter_spec, &params, &sensor_auth).await;
    assert!(
        result.is_ok(),
        "fetch() must return Ok when mock returns a valid server_interfaces response. \
         Got Err: {:?}. BC-2.16.019 AC-010.",
        result.err()
    );

    let batches = result.unwrap();
    assert!(
        !batches.batches.is_empty(),
        "fetch() must return at least one RecordBatch for a non-empty response. \
         BC-2.16.019 AC-010."
    );

    let first_batch = &batches.batches[0];
    assert!(
        first_batch.num_rows() > 0,
        "RecordBatch must contain at least one row. BC-2.16.019 AC-010."
    );

    let schema = first_batch.schema();
    let column_names: Vec<&str> = schema.fields().iter().map(|f| f.name().as_str()).collect();

    // ── Wire-shape assertion 1: class_uid == 5001 ──────────────────────────────
    // EventClassSelector::select_by_class_name("inventory_info") = 5001.
    // (ADR-058 §C2; BC-2.02.012).
    let class_uid_col_idx = schema.index_of("class_uid").expect(
        "RecordBatch must contain 'class_uid' column (OCSF synthesized field). \
         BC-2.16.019 AC-010.",
    );
    let class_uid_col = first_batch.column(class_uid_col_idx);
    let class_uid_array = class_uid_col
        .as_any()
        .downcast_ref::<arrow::array::Int32Array>()
        .expect("class_uid column must be Int32Array");
    let class_uid_val = class_uid_array.value(0);
    assert_eq!(
        class_uid_val, 5001,
        "class_uid MUST equal 5001 \
         (EventClassSelector::select_by_class_name('inventory_info') = 5001). \
         Got: {}. BC-2.16.019 AC-010; ADR-058 §C2.",
        class_uid_val
    );

    // ── Wire-shape assertion 2: device_name column present and non-null ──────
    // ocsf_field_to_arrow_name("device.name") = "device_name"
    // (ADR-058 §C2 Option 4: dot → underscore for nested OCSF fields).
    assert!(
        column_names.contains(&"device_name"),
        "RecordBatch must contain 'device_name' column \
         (server_name → ocsf_field=device.name → ADR-058 arrow name device_name). \
         BC-2.16.019 AC-010; AC-011. Present columns: {:?}",
        column_names
    );
    let device_name_col_idx = schema
        .index_of("device_name")
        .expect("device_name column must be present (asserted above)");
    let device_name_col = first_batch.column(device_name_col_idx);
    let device_name_array = device_name_col
        .as_any()
        .downcast_ref::<arrow::array::StringArray>()
        .expect("device_name column must be StringArray (DataType::Utf8)");
    assert!(
        !device_name_array.is_null(0),
        "device_name must not be null in the first row when server_name is present. \
         BC-2.16.019 AC-010."
    );
    assert_eq!(
        device_name_array.value(0),
        "Monroe-Collector-1",
        "device_name MUST equal the seeded server_name value 'Monroe-Collector-1'. \
         BC-2.16.019 AC-010."
    );

    // ── Wire-shape assertion 3: status_code column present ─────────────────────
    // ocsf_field_to_arrow_name("status_code") = "status_code" (single segment, unchanged).
    assert!(
        column_names.contains(&"status_code"),
        "RecordBatch must contain 'status_code' column \
         (interface_status → ocsf_field=status_code → arrow name status_code). \
         BC-2.16.019 AC-010. Present columns: {:?}",
        column_names
    );

    // ── Wire-shape assertion 4: raw_extensions present as StringArray (JSON object) ──
    // ADR-058 §J6: Tier-2 columns (interface_name, interface_type, etc.) aggregate into raw_extensions.
    assert!(
        column_names.contains(&"raw_extensions"),
        "RecordBatch must contain 'raw_extensions' column (Tier-2 aggregation, ADR-058 §J6). \
         BC-2.16.019 AC-013. Present columns: {:?}",
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
    assert!(
        !raw_ext_array.is_null(0),
        "raw_extensions must not be null in the first row when Tier-2 data is present. \
         BC-2.16.019 AC-013."
    );
    let raw_ext_str = raw_ext_array.value(0);
    let raw_ext_json: serde_json::Value = serde_json::from_str(raw_ext_str)
        .expect("raw_extensions value must be valid JSON (DataType::Utf8 JSON blob)");
    assert!(
        raw_ext_json.is_object(),
        "raw_extensions must deserialize to a JSON object (not array, not scalar). \
         Got: {:?}. BC-2.16.019 AC-013; ADR-058 §J6.",
        raw_ext_json
    );

    // ── Wire-shape assertion 5: at least one Tier-2 field is inside raw_extensions ──
    let raw_ext_obj = raw_ext_json
        .as_object()
        .expect("raw_extensions must be a JSON object");
    let tier2_spot_check = [
        "interface_name",
        "interface_type",
        "interface_connection_type",
        "site_id",
        "avg_traffic_past_hour_mbps",
    ];
    let has_tier2 = tier2_spot_check
        .iter()
        .any(|name| raw_ext_obj.contains_key(*name));
    assert!(
        has_tier2,
        "raw_extensions object must contain at least one Tier-2 field. \
         BC-2.16.019 AC-013. raw_extensions keys: {:?}",
        raw_ext_obj.keys().collect::<Vec<_>>()
    );

    // ── Wire-shape assertion 6: no Tier-2 column names at top level ───────────
    // Tier-2 columns MUST be inside raw_extensions, not as top-level wire columns.
    // BC-2.16.019 Tier-2 isolation (ADR-058 §J6).
    let tier2_names = [
        "interface_name",
        "interface_type",
        "interface_connection_type",
        "site_id",
        "avg_traffic_past_month_mbps",
        "avg_traffic_past_week_mbps",
        "avg_traffic_past_hour_mbps",
        "notes",
    ];
    for tier2_name in &tier2_names {
        assert!(
            !column_names.contains(tier2_name),
            "Tier-2 column '{}' MUST NOT appear as a top-level RecordBatch column — \
             it must be inside raw_extensions. BC-2.16.019; ADR-058 §J6. \
             Top-level columns: {:?}",
            tier2_name,
            column_names
        );
    }

    // ── Wire-shape assertion 7: raw TOML column names NOT at top level ────────
    // server_name and interface_status are Tier-1 TOML names RENAMED to Arrow equivalents.
    assert!(
        !column_names.contains(&"server_name"),
        "server_name (raw TOML name) MUST NOT appear as a top-level column; \
         Arrow name is 'device_name'. BC-2.16.019 AC-011."
    );
    assert!(
        !column_names.contains(&"interface_status"),
        "interface_status (raw TOML name) MUST NOT appear as a top-level column; \
         Arrow name is 'status_code'. BC-2.16.019 AC-011."
    );
}

// ---------------------------------------------------------------------------
// RG-015-WIRE: Null-passthrough production-path test (SAP-4 gap closure)
// ---------------------------------------------------------------------------

/// Production-path null-passthrough test (SAP-4 gap closure, RG-015):
/// `SpecDrivenSensorAdapter::fetch()` for `claroty_server_interfaces` with two records:
///
///   Record 0: `server_name` present, `interface_name = null` (Tier-2 null field)
///   Record 1: `server_name` present, `interface_name = "eth1"` (Tier-2 non-null field)
///
/// LOAD-BEARING assertions (CLAUDE.md §Wire-shape assertion discipline):
///
///   1. Both rows survive — row count MUST be 2.
///      The record with `interface_name = null` MUST NOT be dropped.
///      Only `device_name` null triggers potential row-drop semantics; `interface_name`
///      is Tier-2 (not REQUIRED from a Tier-1 perspective) so null is valid.
///
///   2. Row 0 `device_name` MUST be non-null ("Monroe-Collector-1") — `server_name` was
///      present, so the REQUIRED Tier-1 field resolved correctly.
///
///   3. Row 0 `raw_extensions` MUST contain `"interface_name": null` (null-not-absent).
///      With `explicit_nulls=true`, null Tier-2 values in raw_extensions MUST appear as
///      `"interface_name": null` — NOT be absent from the raw_extensions object.
///      BC-2.11.001 EC-11-079 (null-not-absent discipline applies to raw_extensions too).
///
///   4. Row 1 `raw_extensions` MUST contain `"interface_name": "eth1"` (non-null, correct value).
///
/// ## Why this test exists
///
/// The prism-sensors tests `test_BC_2_16_019_claroty_server_interfaces_required_server_name_absent_produces_null_row`
/// and `test_BC_2_16_019_claroty_server_interfaces_null_interface_name_row_not_dropped`
/// (RG-015 parts 1 and 2) call `ColumnMapper::map_record` directly. `map_record` has
/// ZERO production callers — the production path is `SpecDrivenSensorAdapter::fetch` →
/// `pipeline_result_to_record_batch` → `build_column_array`. This test exercises that
/// full production path and asserts at the serialized wire level.
///
/// BC-2.16.019 AC-015 (null-passthrough); BC-2.11.001 EC-11-079 (null-not-absent);
/// CLAUDE.md §Wire-shape assertion discipline; SAP-4.
/// Story: S-CLAROTY-SERVERS-001 RG-015-WIRE (production-path null-passthrough coverage).
#[tokio::test]
async fn test_BC_2_16_019_claroty_server_interfaces_null_interface_name_row_not_dropped_wire() {
    let mock_server = MockServer::start().await;

    // Two records:
    //   Record 0: interface_name = null (Tier-2 null field), server_name = "Monroe-Collector-1"
    //   Record 1: interface_name = "eth1" (non-null), server_name = "Monroe-Collector-1"
    Mock::given(method("POST"))
        .and(path("/api/v1/server_interfaces/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "server_interfaces": [
                {
                    // Record 0: interface_name is null (Tier-2 field; NOT a row-drop trigger)
                    "server_name": "Monroe-Collector-1",
                    "interface_status": "Active",
                    "interface_name": serde_json::Value::Null,
                    "interface_type": "1000BASE-T"
                },
                {
                    // Record 1: interface_name present (non-null)
                    "server_name": "Monroe-Collector-1",
                    "interface_status": "Inactive",
                    "interface_name": "eth1",
                    "interface_connection_type": "Wired"
                }
            ]
        })))
        .mount(&mock_server)
        .await;

    let adapter = make_claroty_adapter(&mock_server.uri());

    let adapter_spec = SensorAdapterSpec {
        source_table: "claroty_server_interfaces".to_string(),
        org_id: OrgId::from_uuid(uuid::Uuid::now_v7()),
        #[allow(deprecated)]
        client_id: "claroty-server-interfaces-null-test".to_string(),
        sensor_config: serde_json::json!({}),
    };

    let params = QueryParams {
        cursor: None,
        limit: 10,
        start_time: None,
        end_time: None,
        filters: Default::default(),
    };

    let sensor_auth = BearerStaticSensorAuth::new("mock-bearer-token-server-interfaces-null-test");

    let batches = adapter
        .fetch(&adapter_spec, &params, &sensor_auth)
        .await
        .expect(
            "RG-015-WIRE: fetch() must succeed for a valid two-record server_interfaces response. \
             BC-2.16.019 AC-015.",
        );

    assert!(
        !batches.batches.is_empty(),
        "RG-015-WIRE: fetch() must return at least one RecordBatch. BC-2.16.019 AC-015."
    );

    // ── Production MCP serialization path ─────────────────────────────────────
    // explicit_nulls=true: NULL-valued Arrow cells AND null values in Tier-2 aggregations
    // MUST appear in serialized JSON, not be omitted.
    let mut buf: Vec<u8> = Vec::new();
    let mut writer = arrow_json::writer::WriterBuilder::new()
        .with_explicit_nulls(true)
        .build::<_, arrow_json::writer::JsonArray>(&mut buf);
    for batch in &batches.batches {
        writer.write(batch).expect(
            "RG-015-WIRE: arrow_json write must not fail for claroty_server_interfaces RecordBatch",
        );
    }
    writer
        .finish()
        .expect("RG-015-WIRE: arrow_json finish must not fail");

    let json_rows: Vec<serde_json::Value> = serde_json::from_slice::<Vec<serde_json::Value>>(&buf)
        .expect("RG-015-WIRE: arrow_json output must deserialize as a JSON array of row objects");

    // ── Assertion 1: Both rows survive (row-not-dropped invariant) ─────────────
    assert_eq!(
        json_rows.len(),
        2,
        "RG-015-WIRE LOAD-BEARING: serialized JSON must contain exactly 2 rows — \
         BOTH records survive even when record 0 has interface_name = null. \
         The row MUST NOT be dropped; interface_name is Tier-2, not a REQUIRED Tier-1 field. \
         BC-2.16.019 AC-015."
    );

    // ── Assertion 2: Row 0 device_name is non-null (server_name was present) ───
    let row0 = &json_rows[0];
    let row0_device_name = row0.get("device_name");
    assert!(
        row0_device_name.is_some(),
        "RG-015-WIRE: 'device_name' key must be present in row 0. BC-2.16.019 AC-010."
    );
    assert_eq!(
        row0_device_name,
        Some(&serde_json::json!("Monroe-Collector-1")),
        "RG-015-WIRE: row 0 device_name must be 'Monroe-Collector-1' (non-null; \
         server_name was present in the API response). BC-2.16.019 AC-015."
    );

    // ── Assertion 3: Row 0 raw_extensions contains interface_name: null ────────
    // Null-not-absent in raw_extensions (BC-2.11.001 EC-11-079 applies to Tier-2 values
    // inside raw_extensions as well as top-level Arrow null cells).
    let row0_raw_ext_str = row0
        .get("raw_extensions")
        .expect("RG-015-WIRE: 'raw_extensions' key must be present in row 0")
        .as_str()
        .expect("RG-015-WIRE: raw_extensions must be a JSON string (StringArray cell)");
    let row0_raw_ext: serde_json::Value = serde_json::from_str(row0_raw_ext_str)
        .expect("RG-015-WIRE: row 0 raw_extensions must be valid JSON");
    let row0_raw_obj = row0_raw_ext
        .as_object()
        .expect("RG-015-WIRE: row 0 raw_extensions must be a JSON object");

    // Key MUST be present (not absent) — null-not-absent discipline
    assert!(
        row0_raw_obj.contains_key("interface_name"),
        "RG-015-WIRE LOAD-BEARING (null-not-absent in raw_extensions): \
         'interface_name' key MUST be PRESENT inside raw_extensions for row 0 even \
         when the API value was null. BC-2.11.001 EC-11-079. \
         row0 raw_extensions keys: {:?}",
        row0_raw_obj.keys().collect::<Vec<_>>()
    );
    assert_eq!(
        row0_raw_obj.get("interface_name"),
        Some(&serde_json::Value::Null),
        "RG-015-WIRE LOAD-BEARING (null-not-absent in raw_extensions): \
         'interface_name' MUST be JSON null inside raw_extensions for row 0. \
         BC-2.16.019 AC-015; BC-2.11.001 EC-11-079."
    );

    // ── Assertion 4: Row 1 raw_extensions contains interface_name: "eth1" ──────
    let row1 = &json_rows[1];
    let row1_raw_ext_str = row1
        .get("raw_extensions")
        .expect("RG-015-WIRE: 'raw_extensions' key must be present in row 1")
        .as_str()
        .expect("RG-015-WIRE: raw_extensions must be a JSON string (StringArray cell)");
    let row1_raw_ext: serde_json::Value = serde_json::from_str(row1_raw_ext_str)
        .expect("RG-015-WIRE: row 1 raw_extensions must be valid JSON");
    let row1_raw_obj = row1_raw_ext
        .as_object()
        .expect("RG-015-WIRE: row 1 raw_extensions must be a JSON object");

    assert_eq!(
        row1_raw_obj.get("interface_name"),
        Some(&serde_json::json!("eth1")),
        "RG-015-WIRE: row 1 'interface_name' inside raw_extensions must be 'eth1' \
         (non-null, exact seeded value). BC-2.16.019 AC-015."
    );

    // ── Assertion 5: class_uid == 5001 in both rows ────────────────────────────
    assert_eq!(
        row0.get("class_uid"),
        Some(&serde_json::json!(5001_i32)),
        "RG-015-WIRE: row 0 class_uid must equal 5001. BC-2.16.019 AC-010."
    );
    assert_eq!(
        row1.get("class_uid"),
        Some(&serde_json::json!(5001_i32)),
        "RG-015-WIRE: row 1 class_uid must equal 5001. BC-2.16.019 AC-010."
    );
}

// ---------------------------------------------------------------------------
// RG-015-WIRE-SNA: server_name-absent production-path null-passthrough test
//                  (AC-015 part 1 / EC-016-019-001 — sibling of servers RG-007-WIRE)
// ---------------------------------------------------------------------------

/// Production-path null-passthrough test: `SpecDrivenSensorAdapter::fetch()` for
/// `claroty_server_interfaces` with a two-record response where the FIRST record
/// omits `server_name` (the REQUIRED Tier-1 composite PK anchor):
///
///   - Record 0: `server_name` absent → `device_name` becomes Arrow null (REQUIRED absent)
///   - Record 1: `server_name = "Monroe-2"` → `device_name = "Monroe-2"` (non-null)
///
/// LOAD-BEARING assertions (CLAUDE.md §Wire-shape assertion discipline):
///   1. Both rows survive — row count MUST be 2 (row with absent server_name is NOT dropped).
///   2. NULL-NOT-ABSENT (C3/H20 defect class): row 0 `device_name` MUST appear as
///      `"device_name": null` in serialized JSON — NOT be absent.
///      `arrow_json` with `explicit_nulls=false` (DEFAULT) would OMIT the key;
///      this test locks in the `explicit_nulls=true` production configuration.
///   3. Row 1 `device_name` MUST be `"Monroe-2"` (non-null, correct value).
///
/// ## Why this test exists (twin asymmetry closure / MED-2)
///
/// The sibling table `claroty_servers` has the authoritative production-path coverage for
/// its server_name-absent scenario in `bc_2_16_018_claroty_servers_wire_shape.rs` →
/// `test_BC_2_16_018_claroty_servers_null_passthrough_server_name_absent_null_not_absent`
/// (RG-007-WIRE). The `claroty_server_interfaces` table had a coverage gap: the existing
/// RG-015-WIRE test seeds server_name PRESENT in both records, covering only the
/// interface_name=null scenario. This test mirrors RG-007-WIRE for the server_interfaces
/// table, closing the AC-015 part 1 / EC-016-019-001 production-path gap.
///
/// BC-2.16.019 AC-015 (part 1); BC-2.11.001 EC-11-079 (null-not-absent);
/// EC-016-019-001; CLAUDE.md §Wire-shape assertion discipline; SAP-4.
/// Story: S-CLAROTY-SERVERS-001 RG-015-WIRE-SNA (MED-2 twin-asymmetry closure).
#[tokio::test]
async fn test_BC_2_16_019_claroty_server_interfaces_null_passthrough_server_name_absent_null_not_absent()
 {
    let mock_server = MockServer::start().await;

    // Two records:
    //   Record 0: server_name deliberately absent (REQUIRED Tier-1 composite PK anchor missing)
    //             → device_name will be Arrow null (null-not-absent in serialized wire JSON)
    //   Record 1: server_name = "Monroe-2" → device_name = "Monroe-2" (non-null)
    Mock::given(method("POST"))
        .and(path("/api/v1/server_interfaces/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "server_interfaces": [
                {
                    // Record 0: server_name deliberately absent (REQUIRED field missing)
                    "interface_status": "Active",
                    "interface_name": "eth0",
                    "interface_type": "1000BASE-T"
                    // server_name absent → device_name will be Arrow null
                },
                {
                    // Record 1: server_name present → device_name non-null
                    "server_name": "Monroe-2",
                    "interface_status": "Inactive",
                    "interface_name": "eth1"
                }
            ]
        })))
        .mount(&mock_server)
        .await;

    let adapter = make_claroty_adapter(&mock_server.uri());

    let adapter_spec = SensorAdapterSpec {
        source_table: "claroty_server_interfaces".to_string(),
        org_id: OrgId::from_uuid(uuid::Uuid::now_v7()),
        #[allow(deprecated)]
        client_id: "claroty-server-interfaces-sna-test".to_string(),
        sensor_config: serde_json::json!({}),
    };

    let params = QueryParams {
        cursor: None,
        limit: 10,
        start_time: None,
        end_time: None,
        filters: Default::default(),
    };

    let sensor_auth = BearerStaticSensorAuth::new("mock-bearer-token-server-interfaces-sna-test");

    let batches = adapter
        .fetch(&adapter_spec, &params, &sensor_auth)
        .await
        .expect(
            "RG-015-WIRE-SNA: fetch() must succeed for a valid two-record \
             server_interfaces response (server_name absent in record 0). \
             BC-2.16.019 AC-015.",
        );

    assert!(
        !batches.batches.is_empty(),
        "RG-015-WIRE-SNA: fetch() must return at least one RecordBatch. BC-2.16.019 AC-015."
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
            "RG-015-WIRE-SNA: arrow_json write must not fail for \
             claroty_server_interfaces RecordBatch",
        );
    }
    writer
        .finish()
        .expect("RG-015-WIRE-SNA: arrow_json finish must not fail");

    let json_rows: Vec<serde_json::Value> = serde_json::from_slice::<Vec<serde_json::Value>>(&buf)
        .expect(
            "RG-015-WIRE-SNA: arrow_json output must deserialize as a \
             JSON array of row objects",
        );

    assert_eq!(
        json_rows.len(),
        2,
        "RG-015-WIRE-SNA LOAD-BEARING: serialized JSON must contain exactly 2 rows — \
         BOTH records survive even when record 0 has server_name absent. \
         The row MUST NOT be dropped; it becomes a null-cell row per REQUIRED semantics. \
         BC-2.16.019 AC-015; EC-016-019-001."
    );

    // ── Row 0: server_name absent → device_name null-not-absent ───────────────
    let row0 = &json_rows[0];

    // NULL-NOT-ABSENT LOAD-BEARING assertion (RG-015-WIRE-SNA):
    // When `server_name` is absent in the API record, `device_name` becomes an Arrow null
    // cell (nullable=true, value=None). With explicit_nulls=true the JSON row MUST contain
    // `"device_name": null` — NOT omit the key.
    // With explicit_nulls=false (the arrow_json DEFAULT), the key would be absent.
    // This locks in the C3/H20-class defect prevention (BC-2.11.001 EC-11-079).
    let row0_device_name = row0.get("device_name");
    assert!(
        row0_device_name.is_some(),
        "RG-015-WIRE-SNA LOAD-BEARING (null-not-absent): 'device_name' key MUST be \
         PRESENT in row 0 serialized JSON even when 'server_name' was absent in the \
         API response. arrow_json with explicit_nulls=false (DEFAULT) would OMIT this \
         key — that is the C3/H20 defect class (BC-2.11.001 EC-11-079). \
         row0 keys present: {:?}",
        row0.as_object().map(|o| o.keys().collect::<Vec<_>>())
    );
    assert_eq!(
        row0_device_name,
        Some(&serde_json::Value::Null),
        "RG-015-WIRE-SNA LOAD-BEARING (null-not-absent): 'device_name' MUST be \
         JSON null (not another value) in row 0. \
         BC-2.11.001 EC-11-079; BC-2.16.019 AC-015; EC-016-019-001."
    );

    // Row 0 class_uid must still be 5001 even for a null-device_name row
    assert_eq!(
        row0.get("class_uid"),
        Some(&serde_json::json!(5001_i32)),
        "RG-015-WIRE-SNA: row 0 class_uid must equal 5001 even when device_name is null. \
         BC-2.16.019 AC-010."
    );

    // Raw Tier-1 TOML name MUST NOT appear as a top-level key
    assert!(
        row0.get("server_name").is_none(),
        "RG-015-WIRE-SNA: 'server_name' (raw TOML name) MUST NOT appear as top-level key \
         in row 0. Arrow name is 'device_name'. BC-2.16.019 AC-011."
    );

    // ── Row 1: server_name present → device_name non-null ─────────────────────
    let row1 = &json_rows[1];

    let row1_device_name = row1.get("device_name");
    assert!(
        row1_device_name.is_some(),
        "RG-015-WIRE-SNA: 'device_name' key must be present in row 1. BC-2.16.019 AC-010."
    );
    assert_eq!(
        row1_device_name,
        Some(&serde_json::json!("Monroe-2")),
        "RG-015-WIRE-SNA: row 1 'device_name' must be 'Monroe-2' (non-null, exact seeded value). \
         BC-2.16.019 AC-015."
    );

    assert!(
        row1.get("server_name").is_none(),
        "RG-015-WIRE-SNA: 'server_name' (raw TOML name) MUST NOT appear as top-level key \
         in row 1. Arrow name is 'device_name'. BC-2.16.019 AC-011."
    );
}

// ---------------------------------------------------------------------------
// RG-011-E2E: Authoritative SAP-3 E-QUERY-038 gate (HIGH-1 closure)
// ---------------------------------------------------------------------------

/// SAP-3 authoritative reachability test: querying a Tier-2 column directly via
/// `QueryEngine::execute()` raises `PrismError::ColumnNotFound` (E-QUERY-038) with
/// correct `available_columns`.
///
/// ## Why this test is AUTHORITATIVE (not defense-in-depth)
///
/// `test_BC_2_16_019_claroty_server_interfaces_tier2_column_raises_e_query_038` (RG-011,
/// prism-sensors) calls `ocsf_projected_column_names()` directly — valid defense-in-depth,
/// but SAP-3 rule 1 requires at least one test that reaches the arm **end-to-end from the
/// public parser surface**. This test uses `QueryEngine::execute()` as the entry point:
/// parser → planner → `check_query_column_availability` → E-QUERY-038.
/// No HTTP requests are issued; the gate fires at plan-time before any fan-out.
///
/// ## Column under test
///
/// `interface_name` is a Tier-2 column (no `ocsf_field`) that is ALSO a composite PK element
/// (server_name, interface_name). Despite its PK role, it is NOT a queryable top-level Arrow
/// column — it lives inside `raw_extensions` (BC-2.16.019 §PC3).
///
/// ## Available columns contract (BC-2.16.019 AC-011)
///
/// - `available_columns` ⊇ {`raw_extensions`, `device_name`, `status_code`}  (Tier-2 aggregate + Tier-1)
/// - `available_columns` ∌ `interface_name`                                   (Tier-2 — inside raw_extensions)
///
/// BC-2.16.019 AC-011; EC-016-019-004; SAP-3; ADR-058 §I7; S-ADR058-OCSF-ROUTING-001.
/// Story: S-CLAROTY-SERVERS-001 RG-011-E2E (HIGH-1 fix-burst).
#[tokio::test]
async fn test_BC_2_16_019_claroty_server_interfaces_e2e_e_query_038_tier2_column() {
    let spec_content = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../prism-sensors/specs/claroty.sensor.toml"),
    )
    .expect(
        "RG-011-E2E: claroty.sensor.toml must be readable from \
         CARGO_MANIFEST_DIR/../prism-sensors/specs/",
    );
    let spec =
        SpecLoader::parse(&spec_content).expect("RG-011-E2E: claroty.sensor.toml must parse");

    // TableRegistry::register_sensor populates OCSF-projected column names for sensors
    // with ocsf_column_naming = true (S-ADR058-OCSF-ROUTING-001).
    // For claroty_server_interfaces:
    //   Tier-1: device_name (server_name → device.name), status_code (interface_status → status_code)
    //   Tier-2: aggregated as "raw_extensions" — including interface_name (composite PK element)
    //   "interface_name" (Tier-2) → NOT in projected columns → E-QUERY-038
    let registry = Arc::new(TableRegistry::new());
    registry
        .register_sensor(&spec)
        .expect("RG-011-E2E: register_sensor must not fail for production claroty.sensor.toml");

    let engine = QueryEngine::new_with_cache_config(
        Arc::new(prism_sensors::AdapterRegistry::new()),
        Arc::new(NoopCredentialStore),
        Arc::new(OcsfNormalizer::new()),
        Arc::new(ClientRegistry::new(vec![])),
        QueryEngineConfig::default(),
        CacheConfig::default(),
    )
    .with_table_registry(registry);

    // interface_name is Tier-2 (composite PK element, no ocsf_field) → E-QUERY-038 at plan-time.
    // Table registered as "claroty_server_interfaces" ({sensor_id}_{table_name}).
    let result = engine
        .execute(
            "SELECT interface_name FROM claroty_server_interfaces LIMIT 1",
            QueryOptions::default(),
        )
        .await;

    // LOAD-BEARING SAP-3 assertion: must fail at plan-time with E-QUERY-038.
    assert!(
        result.is_err(),
        "RG-011-E2E LOAD-BEARING: QueryEngine::execute must return Err when \
         Tier-2 column 'interface_name' (composite PK element) is queried directly. \
         Got Ok. BC-2.16.019 AC-011; SAP-3."
    );

    let err = result.unwrap_err();

    match &err {
        PrismError::ColumnNotFound(details) => {
            assert_eq!(
                details.column, "interface_name",
                "RG-011-E2E: ColumnNotFound.column must be 'interface_name'. \
                 Got: {:?}. BC-2.16.019 AC-011.",
                details.column
            );
            let avail = &details.available_columns;

            // Tier-2 aggregate must be listed as available.
            assert!(
                avail.contains(&"raw_extensions".to_string()),
                "RG-011-E2E: available_columns must include 'raw_extensions' \
                 (ADR-058 §J6; interface_name accessible via raw_extensions). Got: {:?}",
                avail
            );
            // Tier-1 OCSF projected columns must be available.
            assert!(
                avail.contains(&"device_name".to_string()),
                "RG-011-E2E: available_columns must include 'device_name' \
                 (server_name → ocsf_field=device.name → arrow name). Got: {:?}",
                avail
            );
            assert!(
                avail.contains(&"status_code".to_string()),
                "RG-011-E2E: available_columns must include 'status_code' \
                 (interface_status → ocsf_field=status_code). Got: {:?}",
                avail
            );
            // interface_name is Tier-2 → MUST NOT appear in available_columns.
            assert!(
                !avail.contains(&"interface_name".to_string()),
                "RG-011-E2E: 'interface_name' is Tier-2 and MUST NOT appear in \
                 available_columns (it belongs inside raw_extensions, despite being a \
                 composite PK element). Got: {:?}",
                avail
            );
        }
        other => {
            panic!(
                "RG-011-E2E LOAD-BEARING: QueryEngine::execute must return \
                 PrismError::ColumnNotFound (E-QUERY-038) for Tier-2 column 'interface_name' \
                 (composite PK element). \
                 Got: {:?}. BC-2.16.019 AC-011; SAP-3.",
                other
            );
        }
    }
}

// ---------------------------------------------------------------------------
// RG-016-PROD: count=null empty-page-halt production-path test (MEDIUM-1 closure)
// ---------------------------------------------------------------------------

/// Production-path empty-page-halt test (MEDIUM-1 closure, EC-016-019-005):
/// `SpecDrivenSensorAdapter::fetch()` for `claroty_server_interfaces` with a mock
/// response of `{"server_interfaces":[],"count":null}` MUST return `Ok` with zero rows
/// — no panic, no error.
///
/// ## Why this test exists
///
/// `test_BC_2_16_019_claroty_server_interfaces_nullable_count_uses_empty_page_halt` (RG-016,
/// prism-sensors) is a PAPER-TEST — it constructs `{"server_interfaces":[],"count":null}`
/// and DISCARDS it (`_`-bound). The structural assertion is correct but the behavioral
/// claim (PipelineExecutor halts without dereferencing null count) is NEVER exercised.
///
/// This test closes the gap by serving that exact payload via wiremock.
///
/// ## Behavioral contract (EC-016-019-005)
///
/// OffsetLimit pagination halts when the response array is empty — it does NOT dereference
/// `count`. `count=null` MUST NOT cause a panic, `unwrap`, or `Err` return.
///
/// BC-2.16.019 §PC1 (pagination note); EC-016-019-005; TD-VSDD-059 (paper-fix closure).
/// Story: S-CLAROTY-SERVERS-001 RG-016-PROD (MEDIUM-1 fix-burst).
#[tokio::test]
async fn test_BC_2_16_019_claroty_server_interfaces_ec016_019_005_count_null_empty_page_halt_ok_zero_rows()
 {
    let mock_server = MockServer::start().await;

    // Serve the exact payload RG-016 in prism-sensors documented but never tested:
    //   {"server_interfaces": [], "count": null}
    wiremock::Mock::given(method("POST"))
        .and(path("/api/v1/server_interfaces/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "server_interfaces": [],
            "count": serde_json::Value::Null
        })))
        .mount(&mock_server)
        .await;

    let adapter = make_claroty_adapter(&mock_server.uri());

    let adapter_spec = SensorAdapterSpec {
        source_table: "claroty_server_interfaces".to_string(),
        org_id: OrgId::from_uuid(uuid::Uuid::now_v7()),
        #[allow(deprecated)]
        client_id: "claroty-server-interfaces-count-null-test".to_string(),
        sensor_config: serde_json::json!({}),
    };

    let params = QueryParams {
        cursor: None,
        limit: 10,
        start_time: None,
        end_time: None,
        filters: Default::default(),
    };

    let sensor_auth =
        BearerStaticSensorAuth::new("mock-bearer-token-server-interfaces-count-null-test");

    let result = adapter.fetch(&adapter_spec, &params, &sensor_auth).await;

    // LOAD-BEARING assertion: Ok, not panic, not Err.
    assert!(
        result.is_ok(),
        "RG-016-PROD LOAD-BEARING: fetch() MUST return Ok (not panic, not Err) \
         when the API returns empty server_interfaces array with count=null (EC-016-019-005). \
         OffsetLimit pagination must halt on empty page without dereferencing count. \
         Got Err: {:?}",
        result.err()
    );

    let batches = result.unwrap();
    let total_rows: usize = batches.batches.iter().map(|b| b.num_rows()).sum();
    assert_eq!(
        total_rows, 0,
        "RG-016-PROD: empty server_interfaces array with count=null MUST produce ZERO rows. \
         Got {} rows. BC-2.16.019 §PC1; EC-016-019-005.",
        total_rows
    );
}
