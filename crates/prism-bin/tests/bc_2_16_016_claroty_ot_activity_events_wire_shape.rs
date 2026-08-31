// SPDX-License-Identifier: Apache-2.0
//! Wire-shape and SAP-3 end-to-end tests for BC-2.16.016 —
//! Claroty xDome OT Activity Events Table.
//!
//! # Tests in this file
//!
//! | ID           | Test name | Assertion |
//! |--------------|-----------|-----------|
//! | NEW-1        | test_BC_2_16_016_claroty_ot_activity_events_wire_shape_class_uid_2004_mock | class_uid=2004, finding_info_uid, time, activity_name, message, raw_extensions as JSON object, no Tier-2 top-level keys (RecordBatch-level assertions) |
//! | EC-002-WIRE  | test_BC_2_16_016_claroty_ot_activity_events_ec002_related_alert_ids_native_json_array | Wire-level: related_alert_ids=[1,2,3] survives as native JSON array (not stringified) inside raw_extensions JSON object |
//! | SAP-3        | test_BC_2_16_016_claroty_ot_activity_events_e2e_e_query_038_tier2_column | PrismError::ColumnNotFound (E-QUERY-038) when querying a Tier-2 column via QueryEngine::execute() |
//!
//! # SAP-2 status
//!
//! SAP-2 is NOT APPLICABLE for this table (no DTU exists).
//! See `crates/prism-sensors/tests/bc_2_16_016_claroty_ot_activity_events.rs` for the
//! SAP2_STATUS constant (AC-009 / RG-008).
//!
//! # F-OT-001 / SID-1 compliance (NEW-1)
//!
//! NEW-1 is the non-ignored wire-shape mock test required by SID-1.
//! RG-004 in `prism-sensors/tests/bc_2_16_016_claroty_ot_activity_events.rs` is
//! `#[ignore]`'d (live-only); NEW-1 provides the non-live wire-shape coverage via
//! wiremock + Arrow RecordBatch assertions on the actual serialized column values.
//!
//! # SAP-3 compliance
//!
//! SAP-3 rule 1: at least one test must reach each BC postcondition arm end-to-end
//! from the public surface (parser input), not just from a synthetic-AST proxy.
//! RG-003 in prism-sensors calls `ocsf_projected_column_names()` directly — valid
//! defense-in-depth, but SAP-3 requires this additional parser-surface test.
//!
//! The E-QUERY-038 plan-time column gate fires in `QueryEngine::execute_inner`
//! (engine.rs), BEFORE `run_materialization_pipeline` is called.
//!
//! BC: BC-2.16.016
//! Story: S-CLAROTY-OT-EVENTS-001

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
// Shared helpers — make_claroty_ot_activity_events_adapter
// ---------------------------------------------------------------------------

/// Build a `SpecDrivenSensorAdapter` from the production `claroty.sensor.toml`
/// directed at the given mock server URI.
///
/// This adapter targets the `claroty_ot_activity_events` table.
/// The sensor spec is loaded from the production TOML at test time
/// (not a synthetic spec) so that changes to claroty.sensor.toml propagate
/// through these wire-shape tests automatically.
fn make_claroty_ot_activity_events_adapter(mock_server_uri: &str) -> SpecDrivenSensorAdapter {
    let spec_content = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../prism-sensors/specs/claroty.sensor.toml"),
    )
    .expect(
        "bc_2_16_016_wire_shape: claroty.sensor.toml must be readable from \
         CARGO_MANIFEST_DIR/../prism-sensors/specs/",
    );

    let mut spec = SpecLoader::parse(&spec_content)
        .expect("bc_2_16_016_wire_shape: claroty.sensor.toml must parse");
    spec.base_url = mock_server_uri.to_string();

    let overlay_toml =
        "extends = \"claroty\"\ninstance_id = \"claroty@claroty-ot-events-wire-test\"";
    let overlay: SensorInstanceOverlay = toml::from_str(overlay_toml)
        .expect("bc_2_16_016_wire_shape: SensorInstanceOverlay TOML parse failed");
    let resolved = OverlayLoader::merge_overlay_onto_type_spec(
        &spec,
        &overlay,
        OrgSlug::new("claroty-ot-events-wire-test"),
    );

    let http_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect(
            "bc_2_16_016_wire_shape: reqwest::Client build failed (ADR-050 rustls-tls required)",
        );

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
/// Matches the `NoopCredentialStore` pattern from `bc_2_16_015_claroty_vulnerabilities_wire_shape.rs`.
/// The SAP-3 test fires at plan-time (E-QUERY-038) before any credential lookup occurs.
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
// NEW-1: Wire-shape mock test (SID-1 compliance)
// ---------------------------------------------------------------------------

/// Wire-shape mock test: `SpecDrivenSensorAdapter::fetch()` for `claroty_ot_activity_events`
/// returns RecordBatches with:
///   - `class_uid == 2004` (EventClassSelector("detection_finding") — BC-2.16.016 AC-004)
///   - `finding_info_uid` column present (ocsf_field "finding_info.uid" → arrow name)
///   - `time` column present (ocsf_field "time" → arrow name "time")
///   - `activity_name` column present (ocsf_field "activity_name")
///   - `message` column present (ocsf_field "message")
///   - `raw_extensions` present as StringArray holding a JSON object (Tier-2 aggregate)
///   - No Tier-2 column names at top level (Tier-2 isolation, ADR-058 §J6)
///
/// SID-1 compliance: non-ignored, uses wiremock. Provides non-live wire-shape coverage
/// for RG-004 (which is `#[ignore]`'d pending a live Claroty instance at CLAROTY_INSTANCE_URL).
///
/// The response envelope matches the BC-2.16.016 §Postconditions §response_path:
///   `response_path = "$.ot_activity_events"` → mock response key "ot_activity_events".
///
/// Note on class_uid: detection_finding = 2004
///   (EventClassSelector::select_by_class_name("detection_finding") = 2004).
///   BC-2.16.016 AC-004; ADR-058 §C2.
///
/// BC-2.16.016 AC-004 / AC-005; ADR-058 §C2 Option 4, §J6.
/// Story: S-CLAROTY-OT-EVENTS-001 RG-004 (non-live SID-1 coverage).
#[tokio::test]
async fn test_BC_2_16_016_claroty_ot_activity_events_wire_shape_class_uid_2004_mock() {
    let mock_server = MockServer::start().await;

    // Mock the OT activity events endpoint.
    // BC-2.16.016 §Postconditions §fetch_step: POST request, response_path = "$.ot_activity_events"
    // One record with all Tier-1 fields + representative Tier-2 fields + related_alert_ids array.
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "ot_activity_events": [{
                "event_id": 1001_i64,
                "detection_time": "2024-06-15T14:30:00Z",
                "event_type": "network_connection",
                "description": "A mock OT activity event for wire-shape testing",
                "source_ip": "192.168.10.1",
                "dest_ip": "192.168.20.1",
                "protocol": "TCP",
                "dest_port": 102_i64,
                "source_port": 49152_i64,
                "ip_protocol": "IPv4",
                "source_asset_id": "ot-asset-wire-001",
                "dest_asset_id": "ot-asset-wire-002",
                "source_device_name": "PLC-WIRE-01",
                "dest_device_name": "HMI-WIRE-01",
                "source_device_type": "PLC",
                "dest_device_type": "HMI",
                "source_site_name": "Wire-Test-Site-A",
                "dest_site_name": "Wire-Test-Site-B",
                "source_username": "wire-test-user",
                "related_alert_ids": [101_i64, 202_i64],
                "mode": "Protection"
            }],
            "total": 1_u32,
            "page": 1_u32
        })))
        .mount(&mock_server)
        .await;

    let adapter = make_claroty_ot_activity_events_adapter(&mock_server.uri());

    // source_table = "claroty_ot_activity_events":
    //   sensor_id = "claroty", table_name = "ot_activity_events"
    //   Registered in DataFusion as: format!("{sensor_id}_{table_name}") = "claroty_ot_activity_events"
    let adapter_spec = SensorAdapterSpec {
        source_table: "claroty_ot_activity_events".to_string(),
        org_id: OrgId::from_uuid(uuid::Uuid::now_v7()),
        #[allow(deprecated)]
        client_id: "claroty-ot-events-wire-test".to_string(),
        sensor_config: serde_json::json!({}),
    };

    let params = QueryParams {
        cursor: None,
        limit: 10,
        start_time: None,
        end_time: None,
        filters: Default::default(),
    };

    let sensor_auth = BearerStaticSensorAuth::new("mock-bearer-token-ot-events-wire-test");

    let result = adapter.fetch(&adapter_spec, &params, &sensor_auth).await;
    assert!(
        result.is_ok(),
        "NEW-1: fetch() must return Ok when mock returns a valid ot_activity_events response. \
         Got Err: {:?}. BC-2.16.016 AC-004.",
        result.err()
    );

    let batches = result.unwrap();
    assert!(
        !batches.batches.is_empty(),
        "NEW-1: fetch() must return at least one RecordBatch for a non-empty response. \
         BC-2.16.016 AC-004."
    );

    let first_batch = &batches.batches[0];
    assert!(
        first_batch.num_rows() > 0,
        "NEW-1: RecordBatch must contain at least one row. BC-2.16.016 AC-004."
    );

    let schema = first_batch.schema();
    let column_names: Vec<&str> = schema.fields().iter().map(|f| f.name().as_str()).collect();

    // ── Wire-shape assertion 1: class_uid == 2004 ──────────────────────────────
    // EventClassSelector::select_by_class_name("detection_finding") = 2004
    // (ADR-058 §C2; BC-2.16.016 §Postconditions §ocsf_class = "detection_finding").
    let class_uid_col_idx = schema.index_of("class_uid").expect(
        "NEW-1: RecordBatch must contain 'class_uid' column (OCSF synthesized field). \
         BC-2.16.016 AC-004.",
    );
    let class_uid_col = first_batch.column(class_uid_col_idx);
    let class_uid_array = class_uid_col
        .as_any()
        .downcast_ref::<arrow::array::Int32Array>()
        .expect("NEW-1: class_uid column must be Int32Array");
    let class_uid_val = class_uid_array.value(0);
    assert_eq!(
        class_uid_val, 2004,
        "NEW-1 LOAD-BEARING: class_uid MUST equal 2004 \
         (EventClassSelector::select_by_class_name('detection_finding') = 2004). \
         Got: {}. BC-2.16.016 AC-004; ADR-058 §C2.",
        class_uid_val
    );

    // ── Wire-shape assertion 2: finding_info_uid column present ───────────────
    // ocsf_field_to_arrow_name("finding_info.uid") = "finding_info_uid"
    // (ADR-058 §C2 Option 4: dot → underscore for nested OCSF fields).
    assert!(
        column_names.contains(&"finding_info_uid"),
        "NEW-1: RecordBatch must contain 'finding_info_uid' column \
         (event_id → ocsf_field=finding_info.uid → ADR-058 arrow name finding_info_uid). \
         BC-2.16.016 AC-004. Present columns: {:?}",
        column_names
    );

    // ── Wire-shape assertion 3: time column present ────────────────────────────
    // ocsf_field_to_arrow_name("time") = "time" (single segment, unchanged).
    assert!(
        column_names.contains(&"time"),
        "NEW-1: RecordBatch must contain 'time' column \
         (detection_time → ocsf_field=time → arrow name time). \
         BC-2.16.016 AC-004. Present columns: {:?}",
        column_names
    );

    // ── Wire-shape assertion 4: activity_name column present ──────────────────
    // ocsf_field_to_arrow_name("activity_name") = "activity_name" (single segment).
    assert!(
        column_names.contains(&"activity_name"),
        "NEW-1: RecordBatch must contain 'activity_name' column \
         (event_type → ocsf_field=activity_name → arrow name activity_name). \
         BC-2.16.016 AC-004. Present columns: {:?}",
        column_names
    );

    // ── Wire-shape assertion 5: message column present ────────────────────────
    assert!(
        column_names.contains(&"message"),
        "NEW-1: RecordBatch must contain 'message' column \
         (description → ocsf_field=message → arrow name message). \
         BC-2.16.016 AC-004. Present columns: {:?}",
        column_names
    );

    // ── Wire-shape assertion 6: raw_extensions present as StringArray ─────────
    // ADR-058 §J6: Tier-2 columns aggregate into raw_extensions (DataType::Utf8 StringArray).
    assert!(
        column_names.contains(&"raw_extensions"),
        "NEW-1: RecordBatch must contain 'raw_extensions' column (Tier-2 aggregation). \
         BC-2.16.016 AC-005. Present columns: {:?}",
        column_names
    );
    let raw_ext_col_idx = schema
        .index_of("raw_extensions")
        .expect("raw_extensions column must be present (asserted above)");
    let raw_ext_col = first_batch.column(raw_ext_col_idx);
    let raw_ext_array = raw_ext_col
        .as_any()
        .downcast_ref::<arrow::array::StringArray>()
        .expect("NEW-1: raw_extensions column must be StringArray (DataType::Utf8)");
    assert!(
        !raw_ext_array.is_null(0),
        "NEW-1: raw_extensions must not be null in the first row when Tier-2 data is present. \
         BC-2.16.016 AC-005."
    );
    let raw_ext_str = raw_ext_array.value(0);
    let raw_ext_json: serde_json::Value = serde_json::from_str(raw_ext_str)
        .expect("NEW-1: raw_extensions value must be valid JSON (DataType::Utf8 JSON blob)");
    assert!(
        raw_ext_json.is_object(),
        "NEW-1: raw_extensions must deserialize to a JSON object (not array, not scalar). \
         Got: {:?}. BC-2.16.016 AC-005; ADR-058 §J6.",
        raw_ext_json
    );

    // ── Wire-shape assertion 7: no Tier-2 column names at top level ──────────
    let tier2_names = [
        "source_ip",
        "dest_ip",
        "protocol",
        "dest_port",
        "source_port",
        "ip_protocol",
        "source_asset_id",
        "dest_asset_id",
        "source_device_name",
        "dest_device_name",
        "source_device_type",
        "dest_device_type",
        "source_site_name",
        "dest_site_name",
        "source_username",
        "related_alert_ids",
        "mode",
    ];
    for tier2_name in &tier2_names {
        assert!(
            !column_names.contains(tier2_name),
            "NEW-1: Tier-2 column '{}' MUST NOT appear as a top-level RecordBatch column — \
             it must be inside raw_extensions. BC-2.16.016 §2; ADR-058 §J6. \
             Top-level columns: {:?}",
            tier2_name,
            column_names
        );
    }

    // ── Wire-shape assertion 8: at least one Tier-2 field inside raw_extensions ──
    let raw_ext_obj = raw_ext_json
        .as_object()
        .expect("raw_extensions must be a JSON object (asserted above)");
    let has_tier2 = tier2_names
        .iter()
        .any(|name| raw_ext_obj.contains_key(*name));
    assert!(
        has_tier2,
        "NEW-1: raw_extensions object must contain at least one Tier-2 field. \
         BC-2.16.016 AC-005. raw_extensions keys: {:?}",
        raw_ext_obj.keys().collect::<Vec<_>>()
    );
}

// ---------------------------------------------------------------------------
// EC-002-WIRE: Wire-level related_alert_ids native JSON array assertion
// ---------------------------------------------------------------------------

/// Wire-level assertion for EC-016-016-002 (AC-006):
///   When `related_alert_ids` is a non-empty JSON array `[101, 202]` in the API response,
///   the downstream pipeline MUST preserve it as a NATIVE JSON ARRAY inside the
///   `raw_extensions` JSON blob — NOT as a stringified JSON string `"[101,202]"`.
///
///   At the wire level (serialized JSON output via production `arrow_json` path):
///   - `raw_extensions` (StringArray cell) deserializes to a JSON object
///   - The key `related_alert_ids` MUST be PRESENT in that object
///   - Its value MUST be the JSON array `[101, 202]` (not the string `"[101,202]"`)
///
/// Wire path: `SpecDrivenSensorAdapter::fetch()` →
///   `pipeline_result_to_record_batch` (OCSF mode, raw_extensions block) →
///   `arrow_json::writer::WriterBuilder::new().with_explicit_nulls(true)` →
///   serialized JSON row with `raw_extensions` as a JSON-string column.
///
/// Note: This assertion defines DESIRED behavior (BC-2.16.016 §Postconditions AC-006).
/// If the current pipeline stringifies json arrays (ENRICH-1 DD-2 arm), this test will
/// remain RED until the pipeline is updated to preserve json-type column values as
/// native JSON arrays in raw_extensions. The implementer is responsible for making
/// this test GREEN alongside adding the TOML block.
///
/// BC-2.16.016 §EC-016-016-002 (AC-006); CLAUDE.md §Wire-shape assertion discipline.
/// Story: S-CLAROTY-OT-EVENTS-001 RG-005 (non-live SID-1 wire-level coverage).
#[tokio::test]
async fn test_BC_2_16_016_claroty_ot_activity_events_ec002_related_alert_ids_native_json_array() {
    let mock_server = MockServer::start().await;

    // Record with related_alert_ids as a non-empty JSON integer array.
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "ot_activity_events": [{
                "event_id": 2002_i64,
                "detection_time": "2024-06-15T10:00:00Z",
                "event_type": "alert_triggered",
                "description": "OT event with multiple related alerts",
                "source_ip": "10.1.2.3",
                "dest_ip": "10.4.5.6",
                "protocol": "TCP",
                "dest_port": 502_i64,
                "source_port": 40000_i64,
                "ip_protocol": "IPv4",
                "source_asset_id": "ot-ec002-src",
                "dest_asset_id": "ot-ec002-dst",
                "source_device_name": "PLC-EC002",
                "dest_device_name": "Modbus-RTU-01",
                "source_device_type": "PLC",
                "dest_device_type": "RTU",
                "source_site_name": "Site-EC002-A",
                "dest_site_name": "Site-EC002-B",
                "source_username": "ot-operator",
                // related_alert_ids: non-empty integer array (EC-002)
                "related_alert_ids": [101_i64, 202_i64],
                "mode": "Protection"
            }],
            "total": 1_u32,
            "page": 1_u32
        })))
        .mount(&mock_server)
        .await;

    let adapter = make_claroty_ot_activity_events_adapter(&mock_server.uri());

    let adapter_spec = SensorAdapterSpec {
        source_table: "claroty_ot_activity_events".to_string(),
        org_id: OrgId::from_uuid(uuid::Uuid::now_v7()),
        #[allow(deprecated)]
        client_id: "claroty-ot-events-ec002-test".to_string(),
        sensor_config: serde_json::json!({}),
    };

    let params = QueryParams {
        cursor: None,
        limit: 10,
        start_time: None,
        end_time: None,
        filters: Default::default(),
    };

    let sensor_auth = BearerStaticSensorAuth::new("mock-bearer-token-ec002");

    let batches = adapter
        .fetch(&adapter_spec, &params, &sensor_auth)
        .await
        .expect(
            "EC-002-WIRE: fetch() must succeed for a valid response with related_alert_ids=[101,202]. \
             BC-2.16.016 §EC-016-016-002.",
        );

    assert!(
        !batches.batches.is_empty(),
        "EC-002-WIRE: fetch() must return at least one RecordBatch. BC-2.16.016 §EC-016-016-002."
    );

    // ── Production MCP serialization path (explicit_nulls=true) ──────────────
    // Mirrors server.rs (prism-mcp) production path with explicit_nulls=true.
    // CLAUDE.md §Wire-shape assertion discipline: tests covering MCP-visible surfaces
    // must assert on the serialized JSON output.
    let mut buf: Vec<u8> = Vec::new();
    let mut writer = arrow_json::writer::WriterBuilder::new()
        .with_explicit_nulls(true)
        .build::<_, arrow_json::writer::JsonArray>(&mut buf);
    for batch in &batches.batches {
        writer.write(batch).expect(
            "EC-002-WIRE: arrow_json write must not fail for ot_activity_events RecordBatch",
        );
    }
    writer
        .finish()
        .expect("EC-002-WIRE: arrow_json finish must not fail");

    let json_rows: Vec<serde_json::Value> = serde_json::from_slice::<Vec<serde_json::Value>>(&buf)
        .expect("EC-002-WIRE: arrow_json output must deserialize as a JSON array of row objects");

    assert_eq!(
        json_rows.len(),
        1,
        "EC-002-WIRE: serialized JSON must contain exactly 1 row. \
         BC-2.16.016 §EC-016-016-002."
    );

    let row0 = &json_rows[0];

    // ── class_uid == 2004 at top level ────────────────────────────────────────
    assert_eq!(
        row0.get("class_uid"),
        Some(&serde_json::json!(2004_i32)),
        "EC-002-WIRE: row0 class_uid must equal 2004 in serialized JSON. \
         BC-2.16.016 AC-004; ADR-058 §C2."
    );

    // ── raw_extensions must be a string containing a JSON object ─────────────
    let raw_ext_str = row0.get("raw_extensions").and_then(|v| v.as_str()).expect(
        "EC-002-WIRE: row0 raw_extensions must be present and a JSON string in \
             serialized output. BC-2.16.016 AC-005; ADR-058 §J6.",
    );
    let raw_ext_json: serde_json::Value = serde_json::from_str(raw_ext_str)
        .expect("EC-002-WIRE: row0 raw_extensions must be valid JSON");
    let raw_ext_obj = raw_ext_json
        .as_object()
        .expect("EC-002-WIRE: row0 raw_extensions must be a JSON object");

    // ── EC-002 LOAD-BEARING: related_alert_ids must be native JSON array ──────
    // BC-2.16.016 AC-006 / EC-016-016-002:
    //   "MUST contain `related_alert_ids` as a native JSON array (e.g., `[1, 2, 3]` or `[]`)
    //    — NOT as a stringified JSON string (e.g., `"[1,2,3]"`)"
    //
    // The desired wire form: raw_extensions["related_alert_ids"] = [101, 202] (JSON array)
    // NOT:                   raw_extensions["related_alert_ids"] = "[101,202]" (JSON string)
    let related_alert_ids_wire = raw_ext_obj.get("related_alert_ids").expect(
        "EC-002-WIRE LOAD-BEARING: 'related_alert_ids' key must be PRESENT in row0 \
         raw_extensions JSON object. BC-2.16.016 AC-006; EC-016-016-002.",
    );

    assert!(
        related_alert_ids_wire.is_array(),
        "EC-002-WIRE LOAD-BEARING (BC-2.16.016 AC-006 / EC-016-016-002): \
         'related_alert_ids' in raw_extensions MUST be a native JSON array, \
         NOT a stringified JSON string. \
         BC-2.16.016 §Postconditions requires native JSON array form. \
         Got value type: {:?}, value: {:?}. \
         If it is a string (ENRICH-1 DD-2 stringification), the implementer must \
         update pipeline_result_to_record_batch to preserve json-type column values \
         as native JSON arrays in raw_extensions.",
        related_alert_ids_wire
            .as_str()
            .map(|_| "String")
            .unwrap_or_else(|| "other"),
        related_alert_ids_wire
    );

    assert_eq!(
        related_alert_ids_wire,
        &serde_json::json!([101_i64, 202_i64]),
        "EC-002-WIRE LOAD-BEARING: 'related_alert_ids' must equal [101, 202] as a \
         native JSON array in row0 raw_extensions wire output. \
         BC-2.16.016 AC-006; EC-016-016-002."
    );
}

// ---------------------------------------------------------------------------
// SAP-3: End-to-end E-QUERY-038 test via QueryEngine::execute()
// ---------------------------------------------------------------------------

/// SAP-3 reachability test: querying a Tier-2 column directly via the REAL
/// `QueryEngine::execute()` surface raises `PrismError::ColumnNotFound`
/// (E-QUERY-038) with correct `available_columns`.
///
/// SAP-3 rule 1: at least one test must reach the BC-2.16.016 AC-003 arm
/// end-to-end from the public surface (PrismQL parser input), not merely via
/// the synthetic proxy in RG-003 (prism-sensors, `ocsf_projected_column_names`).
///
/// Architecture: the E-QUERY-038 gate fires inside `QueryEngine::execute_inner`
/// (engine.rs), BEFORE `run_materialization_pipeline` is called. This test uses:
///   1. `SpecLoader::parse(claroty.sensor.toml)` — production spec
///   2. `TableRegistry::register_sensor(&spec)` — populates OCSF-projected names
///      (S-ADR058-OCSF-ROUTING-001 fix: stores Arrow names, not raw col.names)
///   3. `QueryEngine::new_with_cache_config(...).with_table_registry(registry)` — wires the gate
///   4. `engine.execute("SELECT source_ip FROM claroty_ot_activity_events", ...)` — E-QUERY-038
///
/// `source_ip` is Tier-2 (no ocsf_field in claroty.sensor.toml):
///   - NOT in ocsf_projected_column_names → NOT in TableRegistry → E-QUERY-038
///   - available_columns contains "raw_extensions", "finding_info_uid", "time", etc.
///   - available_columns does NOT contain "source_ip"
///
/// Registered table name: `{sensor_id}_{table_name}` = `claroty_ot_activity_events`
///
/// No HTTP requests are issued — E-QUERY-038 fires at plan-time before any fan-out.
///
/// BC-2.16.016 AC-003; SAP-3; ADR-058 §I7; S-ADR058-OCSF-ROUTING-001.
/// Story: S-CLAROTY-OT-EVENTS-001 RG-003 (SAP-3 end-to-end gate).
#[tokio::test]
async fn test_BC_2_16_016_claroty_ot_activity_events_e2e_e_query_038_tier2_column() {
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

    // TableRegistry::register_sensor populates OCSF-projected column names for sensors
    // with ocsf_column_naming = true (S-ADR058-OCSF-ROUTING-001 fix).
    // For claroty_ot_activity_events:
    //   Tier-1 columns → Arrow names (finding_info_uid, time, activity_name, message)
    //   Tier-2 columns → aggregated as "raw_extensions"
    //   "source_ip" (Tier-2) → NOT in projected columns → E-QUERY-038
    let registry = Arc::new(TableRegistry::new());
    registry
        .register_sensor(&spec)
        .expect("SAP-3 test: register_sensor must not fail for production claroty.sensor.toml");

    // Build QueryEngine with the TableRegistry wired.
    // E-QUERY-038 fires before any HTTP requests; NoopCredentialStore satisfies
    // the Arc<dyn CredentialStore> constructor parameter.
    let engine = QueryEngine::new_with_cache_config(
        Arc::new(prism_sensors::AdapterRegistry::new()),
        Arc::new(NoopCredentialStore),
        Arc::new(OcsfNormalizer::new()),
        Arc::new(ClientRegistry::new(vec![])),
        QueryEngineConfig::default(),
        CacheConfig::default(),
    )
    .with_table_registry(registry);

    // Table registered: "claroty_ot_activity_events".
    // `source_ip` is Tier-2 (no ocsf_field) and MUST trigger E-QUERY-038 at plan time.
    let result = engine
        .execute(
            "SELECT source_ip FROM claroty_ot_activity_events",
            QueryOptions::default(),
        )
        .await;

    // ── LOAD-BEARING SAP-3 assertion: must fail at plan time with E-QUERY-038 ──
    assert!(
        result.is_err(),
        "SAP-3 LOAD-BEARING: QueryEngine::execute must return Err when a \
         Tier-2 column ('source_ip') is queried directly. \
         Got Ok. BC-2.16.016 AC-003; SAP-3."
    );

    let err = result.unwrap_err();

    match &err {
        PrismError::ColumnNotFound(details) => {
            // The queried Tier-2 column must be named in the error.
            assert_eq!(
                details.column, "source_ip",
                "SAP-3: ColumnNotFound.column must be 'source_ip'. \
                 Got: {:?}. BC-2.16.016 AC-003.",
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

            // Tier-1 OCSF projected Arrow-name columns must be available.
            assert!(
                avail.contains(&"finding_info_uid".to_string()),
                "SAP-3: available_columns must include 'finding_info_uid' \
                 (event_id → ocsf_field=finding_info.uid → ADR-058 arrow name). Got: {:?}",
                avail
            );
            assert!(
                avail.contains(&"time".to_string()),
                "SAP-3: available_columns must include 'time' \
                 (detection_time → ocsf_field=time). Got: {:?}",
                avail
            );
            assert!(
                avail.contains(&"activity_name".to_string()),
                "SAP-3: available_columns must include 'activity_name' \
                 (event_type → ocsf_field=activity_name). Got: {:?}",
                avail
            );
            assert!(
                avail.contains(&"message".to_string()),
                "SAP-3: available_columns must include 'message' \
                 (description → ocsf_field=message). Got: {:?}",
                avail
            );

            // source_ip is Tier-2 and MUST NOT appear in available_columns.
            assert!(
                !avail.contains(&"source_ip".to_string()),
                "SAP-3: 'source_ip' is Tier-2 and MUST NOT appear in \
                 available_columns (it belongs inside raw_extensions). Got: {:?}",
                avail
            );

            // class_uid is the OCSF event-class synthesized column (AC-003 + E-QUERY-001).
            assert!(
                avail.contains(&"class_uid".to_string()),
                "SAP-3: available_columns must include 'class_uid' \
                 (OCSF synthesized pseudo-column). Got: {:?}. \
                 BC-2.16.016 AC-003; BC Error Case E-QUERY-001.",
                avail
            );

            // _sensor is the per-row sensor-metadata synthesized pseudo-column.
            assert!(
                avail.contains(&"_sensor".to_string()),
                "SAP-3: available_columns must include '_sensor' \
                 (synthesized sensor-metadata pseudo-column). Got: {:?}. \
                 BC-2.16.016 AC-003; BC Error Case E-QUERY-001.",
                avail
            );
        }
        other => {
            panic!(
                "SAP-3 LOAD-BEARING: QueryEngine::execute must return \
                 PrismError::ColumnNotFound (E-QUERY-038) when a Tier-2 column is queried \
                 directly. Got: {:?}. BC-2.16.016 AC-003; SAP-3.",
                other
            );
        }
    }
}
