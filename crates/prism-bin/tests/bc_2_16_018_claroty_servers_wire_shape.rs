// SPDX-License-Identifier: Apache-2.0
//! Production-path wire-shape and SAP-4 null-passthrough tests for BC-2.16.018 —
//! Claroty xDome Collection Servers Table.
//!
//! # SAP-4 Coverage Gap Closure
//!
//! The prism-sensors `map_record`-based tests (RG-007) call
//! `ColumnMapper::map_record` directly. `map_record` has ZERO production callers.
//! The production path is:
//!   `SpecDrivenSensorAdapter::fetch` → `pipeline_result_to_record_batch` → `build_column_array`
//!
//! This file provides the authoritative production-path coverage for the null-passthrough
//! behaviors (RG-007) using a wiremock + `SpecDrivenSensorAdapter::fetch` harness and
//! asserting on SERIALIZED JSON wire output (CLAUDE.md §Wire-shape assertion discipline).
//!
//! # Tests in this file
//!
//! | ID    | Test name | Assertion |
//! |-------|-----------|-----------|
//! | NEW-1 | test_BC_2_16_018_claroty_servers_wire_shape_class_uid_5001_mock | class_uid=5001, device_name, status_code, raw_extensions; no Tier-2 top-level keys |
//! | RG-007-WIRE | test_BC_2_16_018_claroty_servers_null_passthrough_server_name_absent_null_not_absent | server_name absent → device_name null in serialized JSON (null-not-absent discipline) |
//!
//! # SID-1 compliance (NEW-1)
//!
//! RG-005 in `prism-sensors/tests/bc_2_16_018_claroty_servers.rs` is `#[ignore]`'d
//! (live-only). NEW-1 provides the non-live wire-shape coverage via wiremock +
//! RecordBatch assertions on the actual serialized column values.
//!
//! # RG-007-WIRE: null-passthrough production-path coverage
//!
//! RG-007 in prism-sensors calls `ColumnMapper::map_record` directly (SAP-3 defense-in-depth
//! only). This test exercises the FULL production path from HTTP response to serialized
//! wire bytes. The key assertion: with `explicit_nulls=true`, a row where `server_name`
//! was absent MUST appear as `"device_name": null` in JSON — NOT have the key absent.
//! This prevents the C3/H20 null-not-absent defect class (BC-2.11.001 EC-11-079).
//!
//! BC: BC-2.16.018
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

/// Build a `SpecDrivenSensorAdapter` from the production `claroty.sensor.toml`
/// directed at the given mock server URI.
fn make_claroty_adapter(mock_server_uri: &str) -> SpecDrivenSensorAdapter {
    let spec_content = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../prism-sensors/specs/claroty.sensor.toml"),
    )
    .expect(
        "bc_2_16_018_wire_shape: claroty.sensor.toml must be readable from \
         CARGO_MANIFEST_DIR/../prism-sensors/specs/",
    );

    let mut spec = SpecLoader::parse(&spec_content)
        .expect("bc_2_16_018_wire_shape: claroty.sensor.toml must parse");
    spec.base_url = mock_server_uri.to_string();

    let overlay_toml = "extends = \"claroty\"\ninstance_id = \"claroty@claroty-servers-wire-test\"";
    let overlay: SensorInstanceOverlay = toml::from_str(overlay_toml)
        .expect("bc_2_16_018_wire_shape: SensorInstanceOverlay TOML parse failed");
    let resolved = OverlayLoader::merge_overlay_onto_type_spec(
        &spec,
        &overlay,
        OrgSlug::new("claroty-servers-wire-test"),
    );

    let http_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("bc_2_16_018_wire_shape: reqwest::Client build failed (ADR-050 rustls-tls)");

    SpecDrivenSensorAdapter::new(
        Arc::new(resolved),
        AdapterAuthStrategy::BearerStatic,
        http_client,
    )
}

// ---------------------------------------------------------------------------
// NEW-1: Wire-shape mock test (SID-1 compliance)
// ---------------------------------------------------------------------------

/// Wire-shape mock test: `SpecDrivenSensorAdapter::fetch()` for `claroty_servers`
/// returns RecordBatches with:
///   - `class_uid == 5001` (EventClassSelector("inventory_info"))
///   - `device_name` column present (non-null, == "Monroe-Collector-1")
///   - `status_code` column present (case-insensitive match in {"up","down","pending"})
///   - `raw_extensions` present as StringArray holding a JSON object (Tier-2 aggregate)
///   - No Tier-2 column names at top level (Tier-2 isolation, ADR-058 §J6)
///
/// SID-1 compliance: non-ignored, uses wiremock. Provides non-live wire-shape coverage
/// for RG-005 (which is `#[ignore]`'d pending a live Claroty instance at CLAROTY_INSTANCE_URL).
///
/// BC-2.16.018 AC-005; ADR-058 §C2 Option 4, §J6.
/// Story: S-CLAROTY-SERVERS-001 NEW-1 (SID-1 mock coverage).
#[tokio::test]
async fn test_BC_2_16_018_claroty_servers_wire_shape_class_uid_5001_mock() {
    let mock_server = MockServer::start().await;

    // Mock the servers endpoint.
    // DTU route: POST /api/v1/servers/ (trailing slash per xDome convention).
    // Response envelope: {"servers": [...]}
    // One record with both Tier-1 fields (server_name, server_status) + representative Tier-2 fields.
    Mock::given(method("POST"))
        .and(path("/api/v1/servers/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "servers": [{
                "server_name": "Monroe-Collector-1",
                "server_status": "Up",
                "server_location": "Datacenter-A",
                "site_id": 1_u32,
                "model": "MCS R340",
                "os_version": "Ubuntu 20.04",
                "serial_number": "SN-WIRE-001",
                "num_of_interfaces": 4_u32,
                "management_ip": "10.0.0.10",
                "uptime_days": 667.23_f64
            }]
        })))
        .mount(&mock_server)
        .await;

    let adapter = make_claroty_adapter(&mock_server.uri());

    // source_table = "claroty_servers":
    //   sensor_id = "claroty", TOML table_name = "servers"
    //   strip_prefix("claroty_") → "servers" → matches TOML table_name.
    //   Registered in DataFusion as: format!("{sensor_id}_{table_name}") = "claroty_servers".
    let adapter_spec = SensorAdapterSpec {
        source_table: "claroty_servers".to_string(),
        org_id: OrgId::from_uuid(uuid::Uuid::now_v7()),
        #[allow(deprecated)]
        client_id: "claroty-servers-wire-test".to_string(),
        sensor_config: serde_json::json!({}),
    };

    let params = QueryParams {
        cursor: None,
        limit: 10,
        start_time: None,
        end_time: None,
        filters: Default::default(),
    };

    let sensor_auth = BearerStaticSensorAuth::new("mock-bearer-token-servers-wire-test");

    let result = adapter.fetch(&adapter_spec, &params, &sensor_auth).await;
    assert!(
        result.is_ok(),
        "fetch() must return Ok when mock returns a valid servers response. \
         Got Err: {:?}. BC-2.16.018 AC-005.",
        result.err()
    );

    let batches = result.unwrap();
    assert!(
        !batches.batches.is_empty(),
        "fetch() must return at least one RecordBatch for a non-empty response. \
         BC-2.16.018 AC-005."
    );

    let first_batch = &batches.batches[0];
    assert!(
        first_batch.num_rows() > 0,
        "RecordBatch must contain at least one row. BC-2.16.018 AC-005."
    );

    let schema = first_batch.schema();
    let column_names: Vec<&str> = schema.fields().iter().map(|f| f.name().as_str()).collect();

    // ── Wire-shape assertion 1: class_uid == 5001 ──────────────────────────────
    // EventClassSelector::select_by_class_name("inventory_info") = 5001
    // (ADR-058 §C2; BC-2.02.012; same arm as claroty_devices).
    let class_uid_col_idx = schema.index_of("class_uid").expect(
        "RecordBatch must contain 'class_uid' column (OCSF synthesized field). \
         BC-2.16.018 AC-005.",
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
         Got: {}. BC-2.16.018 AC-005; ADR-058 §C2.",
        class_uid_val
    );

    // ── Wire-shape assertion 2: device_name column present and non-null ──────
    // ocsf_field_to_arrow_name("device.name") = "device_name"
    // (ADR-058 §C2 Option 4: dot → underscore for nested OCSF fields).
    assert!(
        column_names.contains(&"device_name"),
        "RecordBatch must contain 'device_name' column \
         (server_name → ocsf_field=device.name → ADR-058 arrow name device_name). \
         BC-2.16.018 AC-005; AC-002. Present columns: {:?}",
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
         BC-2.16.018 AC-005."
    );
    assert_eq!(
        device_name_array.value(0),
        "Monroe-Collector-1",
        "device_name MUST equal the seeded server_name value 'Monroe-Collector-1'. \
         BC-2.16.018 AC-005."
    );

    // ── Wire-shape assertion 3: status_code column present ─────────────────────
    // ocsf_field_to_arrow_name("status_code") = "status_code" (single segment, unchanged).
    assert!(
        column_names.contains(&"status_code"),
        "RecordBatch must contain 'status_code' column \
         (server_status → ocsf_field=status_code → arrow name status_code). \
         BC-2.16.018 AC-005. Present columns: {:?}",
        column_names
    );

    // ── Wire-shape assertion 4: raw_extensions present as StringArray (JSON object) ──
    // ADR-058 §J6: Tier-2 columns aggregate into raw_extensions (DataType::Utf8 StringArray).
    assert!(
        column_names.contains(&"raw_extensions"),
        "RecordBatch must contain 'raw_extensions' column (Tier-2 aggregation, ADR-058 §J6). \
         BC-2.16.018 AC-006. Present columns: {:?}",
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
         BC-2.16.018 AC-006."
    );
    let raw_ext_str = raw_ext_array.value(0);
    let raw_ext_json: serde_json::Value = serde_json::from_str(raw_ext_str)
        .expect("raw_extensions value must be valid JSON (DataType::Utf8 JSON blob)");
    assert!(
        raw_ext_json.is_object(),
        "raw_extensions must deserialize to a JSON object (not array, not scalar). \
         Got: {:?}. BC-2.16.018 AC-006; ADR-058 §J6.",
        raw_ext_json
    );

    // ── Wire-shape assertion 5: at least one Tier-2 field is inside raw_extensions ──
    let raw_ext_obj = raw_ext_json
        .as_object()
        .expect("raw_extensions must be a JSON object");
    let tier2_spot_check = [
        "server_location",
        "model",
        "os_version",
        "management_ip",
        "serial_number",
    ];
    let has_tier2 = tier2_spot_check
        .iter()
        .any(|name| raw_ext_obj.contains_key(*name));
    assert!(
        has_tier2,
        "raw_extensions object must contain at least one Tier-2 field. \
         BC-2.16.018 AC-006. raw_extensions keys: {:?}",
        raw_ext_obj.keys().collect::<Vec<_>>()
    );

    // ── Wire-shape assertion 6: no Tier-2 column names at top level ───────────
    // Tier-2 columns MUST be inside raw_extensions, not as top-level wire columns.
    // BC-2.16.018 §2 Tier-2 isolation (ADR-058 §J6).
    let tier2_names = [
        "server_location",
        "site_id",
        "model",
        "os_version",
        "serial_number",
        "num_of_interfaces",
        "management_ip",
        "idrac_ip",
        "management_mac",
        "uptime_days",
        "avg_traffic_past_month_mbps",
        "avg_traffic_past_week_mbps",
        "avg_traffic_past_hour_mbps",
        "num_of_open_incidents",
        "notes",
    ];
    for tier2_name in &tier2_names {
        assert!(
            !column_names.contains(tier2_name),
            "Tier-2 column '{}' MUST NOT appear as a top-level RecordBatch column — \
             it must be inside raw_extensions. BC-2.16.018 §2; ADR-058 §J6. \
             Top-level columns: {:?}",
            tier2_name,
            column_names
        );
    }

    // ── Wire-shape assertion 7: raw TOML column names NOT at top level ────────
    // server_name and server_status are Tier-1 TOML names that are RENAMED to their
    // Arrow equivalents (device_name, status_code). The raw TOML names MUST NOT appear.
    assert!(
        !column_names.contains(&"server_name"),
        "server_name (raw TOML name) MUST NOT appear as a top-level column; \
         Arrow name is 'device_name'. BC-2.16.018 AC-004."
    );
    assert!(
        !column_names.contains(&"server_status"),
        "server_status (raw TOML name) MUST NOT appear as a top-level column; \
         Arrow name is 'status_code'. BC-2.16.018 AC-004."
    );
}

// ---------------------------------------------------------------------------
// RG-007-WIRE: Null-passthrough production-path test (SAP-4 gap closure)
// ---------------------------------------------------------------------------

/// Production-path null-passthrough test (SAP-4 gap closure, RG-007):
/// `SpecDrivenSensorAdapter::fetch()` for `claroty_servers` with a two-record
/// response where the FIRST record omits `server_name`:
///
///   - Record 0: no `server_name` → `device_name` becomes Arrow null (REQUIRED absent)
///   - Record 1: `server_name = "Monroe-2"` → `device_name = "Monroe-2"` (non-null)
///
/// LOAD-BEARING assertions (CLAUDE.md §Wire-shape assertion discipline):
///   1. Both rows survive — row count MUST be 2 (row with absent server_name is NOT dropped)
///   2. NULL-NOT-ABSENT (C3/H20 defect class): row 0 `device_name` MUST appear as
///      `"device_name": null` in serialized JSON — NOT be absent.
///      `arrow_json` with `explicit_nulls=false` (the DEFAULT) would OMIT the key;
///      this test locks in the `explicit_nulls=true` production configuration.
///   3. Row 1 `device_name` MUST be `"Monroe-2"` (non-null, correct value).
///
/// ## Why this test exists
///
/// The prism-sensors test `test_BC_2_16_018_claroty_servers_required_server_name_absent_produces_null_row`
/// (RG-007) calls `ColumnMapper::map_record` directly. `map_record` has ZERO production
/// callers — the production path is `SpecDrivenSensorAdapter::fetch` →
/// `pipeline_result_to_record_batch` → `build_column_array`. This test exercises that
/// full production path and asserts at the serialized wire level (not just at the
/// RecordBatch struct level).
///
/// BC-2.16.018 AC-007; BC-2.11.001 EC-11-079 (null-not-absent);
/// CLAUDE.md §Wire-shape assertion discipline; SAP-4.
/// Story: S-CLAROTY-SERVERS-001 RG-007-WIRE (production-path null-passthrough coverage).
#[tokio::test]
async fn test_BC_2_16_018_claroty_servers_null_passthrough_server_name_absent_null_not_absent() {
    let mock_server = MockServer::start().await;

    // Two records:
    //   Record 0: server_name absent → device_name = Arrow null (REQUIRED field missing)
    //   Record 1: server_name = "Monroe-2" → device_name = "Monroe-2" (non-null)
    Mock::given(method("POST"))
        .and(path("/api/v1/servers/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "servers": [
                {
                    // Record 0: server_name deliberately absent (REQUIRED field missing)
                    "server_status": "Up",
                    "server_location": "Datacenter-A",
                    "model": "MCS R340"
                    // server_name absent → device_name will be Arrow null
                },
                {
                    // Record 1: server_name present → device_name non-null
                    "server_name": "Monroe-2",
                    "server_status": "Down",
                    "model": "MCS R640"
                }
            ]
        })))
        .mount(&mock_server)
        .await;

    let adapter = make_claroty_adapter(&mock_server.uri());

    let adapter_spec = SensorAdapterSpec {
        source_table: "claroty_servers".to_string(),
        org_id: OrgId::from_uuid(uuid::Uuid::now_v7()),
        #[allow(deprecated)]
        client_id: "claroty-servers-null-passthrough-test".to_string(),
        sensor_config: serde_json::json!({}),
    };

    let params = QueryParams {
        cursor: None,
        limit: 10,
        start_time: None,
        end_time: None,
        filters: Default::default(),
    };

    let sensor_auth = BearerStaticSensorAuth::new("mock-bearer-token-servers-null-test");

    let batches = adapter
        .fetch(&adapter_spec, &params, &sensor_auth)
        .await
        .expect(
            "RG-007-WIRE: fetch() must succeed for a valid two-record servers response. \
             BC-2.16.018 AC-007.",
        );

    assert!(
        !batches.batches.is_empty(),
        "RG-007-WIRE: fetch() must return at least one RecordBatch. BC-2.16.018 AC-007."
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
        writer
            .write(batch)
            .expect("RG-007-WIRE: arrow_json write must not fail for claroty_servers RecordBatch");
    }
    writer
        .finish()
        .expect("RG-007-WIRE: arrow_json finish must not fail");

    let json_rows: Vec<serde_json::Value> = serde_json::from_slice::<Vec<serde_json::Value>>(&buf)
        .expect("RG-007-WIRE: arrow_json output must deserialize as a JSON array of row objects");

    assert_eq!(
        json_rows.len(),
        2,
        "RG-007-WIRE LOAD-BEARING: serialized JSON must contain exactly 2 rows — \
         BOTH records survive even when record 0 has server_name absent. \
         The row MUST NOT be dropped; it becomes a null-cell row per REQUIRED semantics. \
         BC-2.16.018 AC-007."
    );

    // ── Row 0: server_name absent → device_name null-not-absent ───────────────
    let row0 = &json_rows[0];

    // NULL-NOT-ABSENT LOAD-BEARING assertion (RG-007-WIRE):
    // When `server_name` is absent in the API record, `device_name` becomes an Arrow null
    // cell (nullable=true, value=None). With explicit_nulls=true the JSON row MUST contain
    // `"device_name": null` — NOT omit the key.
    // With explicit_nulls=false (the arrow_json DEFAULT), the key would be absent.
    // This locks in the C3/H20-class defect prevention (BC-2.11.001 EC-11-079).
    let row0_device_name = row0.get("device_name");
    assert!(
        row0_device_name.is_some(),
        "RG-007-WIRE LOAD-BEARING (null-not-absent): 'device_name' key MUST be \
         PRESENT in row 0 serialized JSON even when 'server_name' was absent in the \
         API response. arrow_json with explicit_nulls=false (DEFAULT) would OMIT this \
         key — that is the C3/H20 defect class (BC-2.11.001 EC-11-079). \
         row0 keys present: {:?}",
        row0.as_object().map(|o| o.keys().collect::<Vec<_>>())
    );
    assert_eq!(
        row0_device_name,
        Some(&serde_json::Value::Null),
        "RG-007-WIRE LOAD-BEARING (null-not-absent): 'device_name' MUST be \
         JSON null (not another value) in row 0. \
         BC-2.11.001 EC-11-079; BC-2.16.018 AC-007."
    );

    // Row 0 class_uid must still be 5001 even for a null-device_name row
    assert_eq!(
        row0.get("class_uid"),
        Some(&serde_json::json!(5001_i32)),
        "RG-007-WIRE: row 0 class_uid must equal 5001 even when device_name is null. \
         BC-2.16.018 AC-005."
    );

    // ── Row 1: server_name present → device_name non-null ─────────────────────
    let row1 = &json_rows[1];

    let row1_device_name = row1.get("device_name");
    assert!(
        row1_device_name.is_some(),
        "RG-007-WIRE: 'device_name' key must be present in row 1. BC-2.16.018 AC-005."
    );
    assert_eq!(
        row1_device_name,
        Some(&serde_json::json!("Monroe-2")),
        "RG-007-WIRE: row 1 'device_name' must be 'Monroe-2' (non-null, exact seeded value). \
         BC-2.16.018 AC-007."
    );

    // Raw Tier-1 TOML names MUST NOT appear as top-level keys in either row
    assert!(
        row0.get("server_name").is_none(),
        "RG-007-WIRE: 'server_name' (raw TOML name) MUST NOT appear as top-level key in row 0. \
         Arrow name is 'device_name'. BC-2.16.018 AC-004."
    );
    assert!(
        row1.get("server_name").is_none(),
        "RG-007-WIRE: 'server_name' (raw TOML name) MUST NOT appear as top-level key in row 1. \
         Arrow name is 'device_name'. BC-2.16.018 AC-004."
    );
}
