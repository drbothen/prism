// SPDX-License-Identifier: Apache-2.0
//! SAP-4 production-path wire-shape tests for BC-2.16.021 —
//! Claroty xDome Organization Firewall Domain (firewall groups + firewall policies).
//!
//! # SAP-4 Gap Closed
//!
//! `crates/prism-sensors/tests/bc_2_16_021_claroty_org_fw_policy.rs` tests RG-019,
//! RG-025, and RG-026 via `ColumnMapper::map_record` directly (pre-serialization).
//! `map_record` has ZERO production callers. The production path is:
//!
//! ```text
//! SpecDrivenSensorAdapter::fetch
//!   → PipelineExecutor::execute
//!     → pipeline_result_to_record_batch
//!       → build_column_array (generic ColumnType::Json arm)
//!         → RecordBatch serialized via arrow_json::writer::WriterBuilder::new()
//!             .with_explicit_nulls(true)
//! ```
//!
//! This file provides the authoritative production-path gate for:
//!   - JSON columns (`device_conditions`, `communication_conditions`, `related_alerts_ids`,
//!     `applied_group_pairs`) materializing as NATIVE JSON array/object in `raw_extensions`
//!     (not stringified) through the generic `ColumnType::Json` arm in `build_column_array`.
//!   - Required-field-absent → null-cell → row-survives → `"name": null` at wire level
//!     (not absent — BC-2.11.001 EC-11-079 null-not-absent discipline).
//!
//! # URL↔Envelope Asymmetry (EC-016-021-006)
//!
//! Firewall domain tables use abbreviated URLs but full-spelling envelope keys:
//!   - `claroty_organization_firewall_groups`:
//!       path_template = "/api/v1/organization_fw_groups/"  (abbreviated _fw_groups)
//!       response_path = "$.organization_firewall_groups"   (FULL spelling in envelope)
//!   - `claroty_organization_firewall_policies`:
//!       path_template = "/api/v1/organization_fw_group_policies/"  (abbreviated)
//!       response_path = "$.organization_firewall_policies"          (full spelling)
//!
//! These asymmetries are asserted in RG-015 (prism-sensors). The mock server in this file
//! uses the ABBREVIATED paths and the FULL-SPELLING envelope keys accordingly.
//!
//! # Tests in this file
//!
//! | ID              | Test name | Assertion |
//! |-----------------|-----------|-----------|
//! | SAP4-021-FG-1   | test_BC_2_16_021_claroty_organization_firewall_groups_wire_shape_class_uid_3004_mock | class_uid=3004, Tier-1 cols, raw_extensions JSON object, device_conditions native array |
//! | SAP4-021-FG-2   | test_BC_2_16_021_claroty_organization_firewall_groups_wire_shape_serialized_json_null_not_absent | null-not-absent for firewall_group_name absent → "name":null in serialized JSON |
//! | SAP4-021-FP-3   | test_BC_2_16_021_claroty_organization_firewall_policies_wire_shape_class_uid_3004_mock | class_uid=3004, Tier-1 cols, raw_extensions JSON object, communication_conditions/related_alerts_ids/applied_group_pairs native arrays |
//! | SAP4-021-FP-4   | test_BC_2_16_021_claroty_organization_firewall_policies_wire_shape_serialized_json_null_not_absent | null-not-absent for policy_name absent → "name":null in serialized JSON |
//!
//! # SID-1 Compliance
//!
//! Tests RG-017, RG-018, and RG-023 in prism-sensors are `#[ignore]`'d (live-only). This file
//! provides non-live SID-1-compliant coverage via wiremock + mock server, exercising
//! the real production `SpecDrivenSensorAdapter::fetch` code path.
//!
//! # Source Table Name Note
//!
//! TOML entries for org policy tables carry the sensor prefix inside `table_name`:
//!   e.g. `table_name = "claroty_organization_firewall_groups"`. `SpecDrivenSensorAdapter::fetch`
//!   strips the `sensor_id + "_"` prefix from `spec.source_table` to resolve the matching table.
//!   Therefore `source_table = "claroty_claroty_organization_firewall_groups"`.
//!
//! BC: BC-2.16.021
//! Story: S-CLAROTY-ORGPOLICY-001

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

/// Build a `SpecDrivenSensorAdapter` from the production `claroty.sensor.toml`
/// directed at the given mock server URI.
fn make_claroty_adapter(mock_server_uri: &str) -> SpecDrivenSensorAdapter {
    let spec_content = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../prism-sensors/specs/claroty.sensor.toml"),
    )
    .expect(
        "bc_2_16_021_wire_shape: claroty.sensor.toml must be readable from \
         CARGO_MANIFEST_DIR/../prism-sensors/specs/",
    );

    let mut spec = SpecLoader::parse(&spec_content)
        .expect("bc_2_16_021_wire_shape: claroty.sensor.toml must parse");
    spec.base_url = mock_server_uri.to_string();

    let overlay_toml =
        "extends = \"claroty\"\ninstance_id = \"claroty@claroty-fwpolicy-wire-test\"";
    let overlay: SensorInstanceOverlay = toml::from_str(overlay_toml)
        .expect("bc_2_16_021_wire_shape: SensorInstanceOverlay TOML parse failed");
    let resolved = OverlayLoader::merge_overlay_onto_type_spec(
        &spec,
        &overlay,
        OrgSlug::new("claroty-fwpolicy-wire-test"),
    );

    let http_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("bc_2_16_021_wire_shape: reqwest::Client build failed (ADR-050 rustls-tls)");

    SpecDrivenSensorAdapter::new(
        Arc::new(resolved),
        AdapterAuthStrategy::BearerStatic,
        http_client,
    )
}

/// Minimal no-op `CredentialStore` for constructing `QueryEngine` in SAP-3 tests.
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
// SAP4-021-FG-1: firewall_groups wire-shape mock test
// ---------------------------------------------------------------------------

/// SAP-4 production-path wire-shape test for `claroty_organization_firewall_groups`:
/// `SpecDrivenSensorAdapter::fetch()` returns RecordBatches with:
///   - `class_uid == 3004` (entity_management; BC-2.16.021)
///   - Tier-1 Arrow columns: `name`, `comment`, `status_code`, `actor_user_name`
///   - `raw_extensions` as StringArray holding a JSON object (Tier-2 aggregate)
///   - `device_conditions` in `raw_extensions` as a NATIVE JSON array (NOT stringified)
///   - No Tier-2 column names at top level
///
/// # URL↔Envelope Asymmetry (EC-016-021-006)
///
/// The mock server uses:
///   - Mock path: `/api/v1/organization_fw_groups/` (abbreviated URL, as in TOML path_template)
///   - Response envelope key: `"organization_firewall_groups"` (full spelling, as in response_path)
///
/// This asymmetry is critical: using `"organization_fw_groups"` as the envelope key would cause
/// silent empty results (EC-016-021-006). The production TOML has:
///   `response_path = "$.organization_firewall_groups"` (full spelling).
///
/// SAP-4 gap closure: `test_BC_2_16_021_claroty_organization_firewall_groups_required_fwgroupname_absent_produces_null_row`
/// (prism-sensors) uses `ColumnMapper::map_record` only. This test exercises the REAL production path.
///
/// BC-2.16.021 AC-015/AC-016/AC-017/AC-019; ADR-058 §J6; EC-016-021-006.
/// Story: S-CLAROTY-ORGPOLICY-001 (SAP-4 production-path coverage).
#[tokio::test]
async fn test_BC_2_16_021_claroty_organization_firewall_groups_wire_shape_class_uid_3004_mock() {
    let mock_server = MockServer::start().await;

    // CRITICAL: path uses abbreviated "_fw_groups"; envelope key uses FULL "organization_firewall_groups".
    // EC-016-021-006: using "$.organization_fw_groups" would cause silent empty results.
    Mock::given(method("POST"))
        .and(path("/api/v1/organization_fw_groups/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            // FULL spelling envelope key — NOT "organization_fw_groups"
            "organization_firewall_groups": [{
                "firewall_group_name": "FW-Group-Production",
                "firewall_group_description": "Production OT firewall group",
                "firewall_group_source": "Custom",
                "priority": 2,
                "enabled": true,
                "device_conditions": [
                    {"type": "network_segment", "value": "ot-segment"},
                    {"type": "vendor", "value": "siemens"}
                ],
                "attributed_devices": 15,
                "exportable_attributed_devices": 15,
                "created_time": "2024-02-01T00:00:00Z",
                "last_update": "2024-07-01T00:00:00Z",
                "updated_by": "fw-admin@example.com"
            }]
        })))
        .mount(&mock_server)
        .await;

    let adapter = make_claroty_adapter(&mock_server.uri());

    // TOML table_name = "claroty_organization_firewall_groups" (WITH sensor prefix).
    // source_table = "claroty_" + "claroty_organization_firewall_groups".
    let adapter_spec = SensorAdapterSpec {
        source_table: "claroty_claroty_organization_firewall_groups".to_string(),
        org_id: OrgId::from_uuid(uuid::Uuid::now_v7()),
        #[allow(deprecated)]
        client_id: "claroty-fw-groups-wire-test".to_string(),
        sensor_config: serde_json::json!({}),
    };

    let params = QueryParams {
        cursor: None,
        limit: 10,
        start_time: None,
        end_time: None,
        filters: Default::default(),
    };

    let sensor_auth = BearerStaticSensorAuth::new("mock-bearer-token-fw-groups-wire-test");

    let result = adapter.fetch(&adapter_spec, &params, &sensor_auth).await;
    assert!(
        result.is_ok(),
        "SAP4-021-FG-1: fetch() must return Ok for a valid organization_firewall_groups \
         response. Got Err: {:?}. BC-2.16.021 AC-015.",
        result.err()
    );

    let batches = result.unwrap();
    assert!(
        !batches.batches.is_empty(),
        "SAP4-021-FG-1: fetch() must return at least one RecordBatch. BC-2.16.021 AC-015."
    );

    let first_batch = &batches.batches[0];
    assert!(
        first_batch.num_rows() > 0,
        "SAP4-021-FG-1: RecordBatch must contain at least one row. BC-2.16.021 AC-015."
    );

    let schema = first_batch.schema();
    let column_names: Vec<&str> = schema.fields().iter().map(|f| f.name().as_str()).collect();

    // ── Wire-shape assertion 1: class_uid == 3004 ────────────────────────────
    let class_uid_col_idx = schema
        .index_of("class_uid")
        .expect("SAP4-021-FG-1: RecordBatch must contain 'class_uid'. BC-2.16.021 AC-015.");
    let class_uid_array = first_batch
        .column(class_uid_col_idx)
        .as_any()
        .downcast_ref::<arrow::array::Int32Array>()
        .expect("class_uid must be Int32Array");
    assert_eq!(
        class_uid_array.value(0),
        3004,
        "SAP4-021-FG-1: class_uid MUST equal 3004 (entity_management). BC-2.16.021."
    );

    // ── Wire-shape assertion 2: Tier-1 Arrow columns present ─────────────────
    // firewall_group_name → "name", firewall_group_description → "comment",
    // enabled → "status_code", updated_by → "actor_user_name"
    for expected_col in ["name", "comment", "status_code", "actor_user_name"] {
        assert!(
            column_names.contains(&expected_col),
            "SAP4-021-FG-1: RecordBatch must contain '{}' (Tier-1 Arrow column). \
             BC-2.16.021 AC-016. Present columns: {:?}",
            expected_col,
            column_names
        );
    }

    // ── Wire-shape assertion 3: raw_extensions present as StringArray ─────────
    assert!(
        column_names.contains(&"raw_extensions"),
        "SAP4-021-FG-1: raw_extensions must be present (Tier-2 aggregate, ADR-058 §J6). \
         BC-2.16.021 AC-017. Present columns: {:?}",
        column_names
    );
    let raw_ext_col_idx = schema.index_of("raw_extensions").unwrap();
    let raw_ext_array = first_batch
        .column(raw_ext_col_idx)
        .as_any()
        .downcast_ref::<arrow::array::StringArray>()
        .expect("raw_extensions must be StringArray");
    assert!(
        !raw_ext_array.is_null(0),
        "SAP4-021-FG-1: raw_extensions must not be null in row 0. BC-2.16.021 AC-017."
    );
    let raw_ext_json: serde_json::Value = serde_json::from_str(raw_ext_array.value(0))
        .expect("SAP4-021-FG-1: raw_extensions must be valid JSON");
    assert!(
        raw_ext_json.is_object(),
        "SAP4-021-FG-1: raw_extensions must be a JSON object. Got: {:?}.",
        raw_ext_json
    );
    let raw_ext_obj = raw_ext_json.as_object().unwrap();

    // ── Wire-shape assertion 4 (LOAD-BEARING SAP-4): device_conditions as NATIVE JSON array ──
    // Authoritative production-path gate for device_conditions Json arm.
    // `test_BC_2_16_021_claroty_organization_firewall_groups_required_fwgroupname_absent_produces_null_row`
    // (prism-sensors) tests `ColumnMapper::map_record` only; this test exercises the full
    // production path via `SpecDrivenSensorAdapter::fetch`.
    let device_cond = raw_ext_obj.get("device_conditions").expect(
        "SAP4-021-FG-1 LOAD-BEARING: 'device_conditions' MUST be present in raw_extensions. \
             column_type = \"json\" → ColumnType::Json arm in build_column_array. \
             BC-2.16.021 AC-019; §PC6.",
    );
    assert!(
        device_cond.is_array(),
        "SAP4-021-FG-1 LOAD-BEARING: 'device_conditions' in raw_extensions MUST be a NATIVE \
         JSON array (not stringified). Production path: build_column_array ColumnType::Json arm. \
         Got: {:?}. BC-2.16.021 §PC6.",
        device_cond
    );
    let device_cond_arr = device_cond.as_array().unwrap();
    assert!(
        !device_cond_arr.is_empty(),
        "SAP4-021-FG-1: device_conditions must be non-empty array for this mock."
    );
    assert!(
        device_cond_arr[0].is_object(),
        "SAP4-021-FG-1: device_conditions[0] must be a JSON object. Got: {:?}.",
        device_cond_arr[0]
    );

    // ── Wire-shape assertion 5: Tier-2 columns NOT at top level ──────────────
    let tier2_names = [
        "firewall_group_source",
        "priority",
        "device_conditions",
        "attributed_devices",
        "exportable_attributed_devices",
        "created_time",
        "last_update",
    ];
    for tier2_name in &tier2_names {
        assert!(
            !column_names.contains(tier2_name),
            "SAP4-021-FG-1: Tier-2 column '{}' MUST NOT appear as top-level RecordBatch \
             column (ADR-058 §J6). Top-level columns: {:?}",
            tier2_name,
            column_names
        );
    }
}

// ---------------------------------------------------------------------------
// SAP4-021-FG-2: firewall_groups null-not-absent
// ---------------------------------------------------------------------------

/// SAP-4 production-path null-not-absent test for `claroty_organization_firewall_groups`:
/// When `firewall_group_name` (REQUIRED) is absent, the row must survive and
/// `"name": null` must appear in serialized JSON (not absent).
///
/// BC-2.16.021 AC-019; BC-2.11.001 EC-11-079.
/// Story: S-CLAROTY-ORGPOLICY-001 (SAP-4 production-path coverage).
#[tokio::test]
async fn test_BC_2_16_021_claroty_organization_firewall_groups_wire_shape_serialized_json_null_not_absent()
 {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/v1/organization_fw_groups/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "organization_firewall_groups": [
                {
                    "firewall_group_name": "Named-FW-Group",
                    "firewall_group_description": "Has all required fields",
                    "firewall_group_source": "Custom",
                    "priority": 1,
                    "enabled": true,
                    "device_conditions": [{"type": "vendor", "value": "ge"}],
                    "attributed_devices": 10,
                    "exportable_attributed_devices": 10,
                    "updated_by": "admin@example.com"
                },
                {
                    // firewall_group_name ABSENT — REQUIRED → Arrow null for "name"
                    "firewall_group_description": "No firewall_group_name provided",
                    "firewall_group_source": "Recommended",
                    "priority": 99,
                    "enabled": false,
                    "device_conditions": []
                }
            ]
        })))
        .mount(&mock_server)
        .await;

    let adapter = make_claroty_adapter(&mock_server.uri());

    let adapter_spec = SensorAdapterSpec {
        source_table: "claroty_claroty_organization_firewall_groups".to_string(),
        org_id: OrgId::from_uuid(uuid::Uuid::now_v7()),
        #[allow(deprecated)]
        client_id: "claroty-fw-groups-null-test".to_string(),
        sensor_config: serde_json::json!({}),
    };

    let params = QueryParams {
        cursor: None,
        limit: 10,
        start_time: None,
        end_time: None,
        filters: Default::default(),
    };

    let sensor_auth = BearerStaticSensorAuth::new("mock-bearer-token-fw-groups-null-test");

    let batches = adapter
        .fetch(&adapter_spec, &params, &sensor_auth)
        .await
        .expect(
            "SAP4-021-FG-2: fetch() must return Ok for a valid two-record fw_groups mock. \
             BC-2.16.021 AC-019.",
        );

    assert!(
        !batches.batches.is_empty(),
        "SAP4-021-FG-2: fetch() must return at least one RecordBatch. BC-2.16.021 AC-019."
    );

    let mut buf: Vec<u8> = Vec::new();
    let mut writer = arrow_json::writer::WriterBuilder::new()
        .with_explicit_nulls(true)
        .build::<_, arrow_json::writer::JsonArray>(&mut buf);
    for batch in &batches.batches {
        writer
            .write(batch)
            .expect("SAP4-021-FG-2: arrow_json write must not fail");
    }
    writer
        .finish()
        .expect("SAP4-021-FG-2: arrow_json finish must not fail");

    let json_rows: Vec<serde_json::Value> = serde_json::from_slice::<Vec<serde_json::Value>>(&buf)
        .expect("SAP4-021-FG-2: arrow_json output must deserialize as JSON array");

    assert_eq!(
        json_rows.len(),
        2,
        "SAP4-021-FG-2: must have 2 rows — REQUIRED absent must NOT drop row. \
         BC-2.16.021 AC-019."
    );

    assert_eq!(
        json_rows[0].get("name"),
        Some(&serde_json::json!("Named-FW-Group")),
        "SAP4-021-FG-2: row0 'name' must be 'Named-FW-Group'. BC-2.16.021 AC-019."
    );

    let row1_name = json_rows[1].get("name");
    assert!(
        row1_name.is_some(),
        "SAP4-021-FG-2 LOAD-BEARING (null-not-absent): 'name' key MUST be PRESENT in row1 \
         even when 'firewall_group_name' was absent. explicit_nulls=false (DEFAULT) would \
         OMIT this key (C3/H20 defect class). BC-2.11.001 EC-11-079; BC-2.16.021 AC-019. \
         row1 keys: {:?}",
        json_rows[1]
            .as_object()
            .map(|o| o.keys().collect::<Vec<_>>())
    );
    assert_eq!(
        row1_name,
        Some(&serde_json::Value::Null),
        "SAP4-021-FG-2 LOAD-BEARING: 'name' MUST be JSON null in row1. \
         BC-2.11.001 EC-11-079; BC-2.16.021 AC-019."
    );
}

// ---------------------------------------------------------------------------
// SAP4-021-FP-3: firewall_policies wire-shape mock test
// ---------------------------------------------------------------------------

/// SAP-4 production-path wire-shape test for `claroty_organization_firewall_policies`:
/// `SpecDrivenSensorAdapter::fetch()` returns RecordBatches with:
///   - `class_uid == 3004` (entity_management; BC-2.16.021)
///   - Tier-1 Arrow columns: `name`, `activity_name`, `comment`, `actor_user_name`
///   - `raw_extensions` with all three JSON columns as NATIVE arrays (not stringified):
///     - `communication_conditions` (array of src/dst firewall-group condition objects)
///     - `related_alerts_ids` (array of triggered alert ID strings)
///     - `applied_group_pairs` (array of {src_group, dst_group} pair objects — NOT applied_zone_pairs)
///   - No Tier-2 column names at top level
///
/// NOTE: `applied_group_pairs` (firewall domain) vs `applied_zone_pairs` (zone domain) —
/// these are distinct columns in distinct tables; do not conflate them (BC-2.16.021 vs BC-2.16.020).
///
/// # URL↔Envelope Asymmetry (EC-016-021-006)
///
/// Mock path: `/api/v1/organization_fw_group_policies/` (abbreviated)
/// Envelope key: `"organization_firewall_policies"` (full spelling; NOT "organization_fw_group_policies")
///
/// SAP-4 gap closure: `test_BC_2_16_021_claroty_organization_firewall_policies_json_columns_not_stringified`
/// (prism-sensors) uses `ColumnMapper::map_record` only. This test exercises the REAL production path.
///
/// BC-2.16.021 AC-021/AC-022/AC-023/AC-026; ADR-058 §J6; EC-016-021-006.
/// Story: S-CLAROTY-ORGPOLICY-001 (SAP-4 production-path coverage).
#[tokio::test]
async fn test_BC_2_16_021_claroty_organization_firewall_policies_wire_shape_class_uid_3004_mock() {
    let mock_server = MockServer::start().await;

    // CRITICAL: abbreviated URL path; FULL-SPELLING envelope key (EC-016-021-006).
    Mock::given(method("POST"))
        .and(path("/api/v1/organization_fw_group_policies/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            // FULL spelling — NOT "organization_fw_group_policies"
            "organization_firewall_policies": [{
                "policy_name": "FW-Allow-OT-to-DMZ",
                "policy_source": "Custom",
                "policy_action": "Allow",
                "communication_conditions": [
                    {"src_group_id": 10, "dst_group_id": 20, "protocol": "tcp", "port": 502}
                ],
                "matching_devices": 6,
                "should_generate_alerts": true,
                "alert_use_case": "Unauthorized Connection",
                "policy_notes": "OT-to-DMZ Modbus/TCP policy",
                "related_alerts_ids": ["fw-alert-001"],
                "applied_group_pairs": [
                    {"src_group": 10, "dst_group": 20},
                    {"src_group": 11, "dst_group": 21}
                ],
                "created_time": "2024-03-01T00:00:00Z",
                "last_updated": "2024-08-01T00:00:00Z",
                "updated_by": "fw-policy-admin@example.com"
            }]
        })))
        .mount(&mock_server)
        .await;

    let adapter = make_claroty_adapter(&mock_server.uri());

    // TOML table_name = "claroty_organization_firewall_policies" (WITH sensor prefix).
    // source_table = "claroty_" + "claroty_organization_firewall_policies".
    let adapter_spec = SensorAdapterSpec {
        source_table: "claroty_claroty_organization_firewall_policies".to_string(),
        org_id: OrgId::from_uuid(uuid::Uuid::now_v7()),
        #[allow(deprecated)]
        client_id: "claroty-fw-policies-wire-test".to_string(),
        sensor_config: serde_json::json!({}),
    };

    let params = QueryParams {
        cursor: None,
        limit: 10,
        start_time: None,
        end_time: None,
        filters: Default::default(),
    };

    let sensor_auth = BearerStaticSensorAuth::new("mock-bearer-token-fw-policies-wire-test");

    let result = adapter.fetch(&adapter_spec, &params, &sensor_auth).await;
    assert!(
        result.is_ok(),
        "SAP4-021-FP-3: fetch() must return Ok for a valid organization_firewall_policies \
         response. Got Err: {:?}. BC-2.16.021 AC-021.",
        result.err()
    );

    let batches = result.unwrap();
    assert!(
        !batches.batches.is_empty(),
        "SAP4-021-FP-3: fetch() must return at least one RecordBatch. BC-2.16.021 AC-021."
    );

    let first_batch = &batches.batches[0];
    assert!(
        first_batch.num_rows() > 0,
        "SAP4-021-FP-3: RecordBatch must contain at least one row. BC-2.16.021 AC-021."
    );

    let schema = first_batch.schema();
    let column_names: Vec<&str> = schema.fields().iter().map(|f| f.name().as_str()).collect();

    // ── Wire-shape assertion 1: class_uid == 3004 ────────────────────────────
    let class_uid_col_idx = schema
        .index_of("class_uid")
        .expect("SAP4-021-FP-3: RecordBatch must contain 'class_uid'. BC-2.16.021 AC-021.");
    let class_uid_array = first_batch
        .column(class_uid_col_idx)
        .as_any()
        .downcast_ref::<arrow::array::Int32Array>()
        .expect("class_uid must be Int32Array");
    assert_eq!(
        class_uid_array.value(0),
        3004,
        "SAP4-021-FP-3: class_uid MUST equal 3004 (entity_management). BC-2.16.021."
    );

    // ── Wire-shape assertion 2: Tier-1 Arrow columns ─────────────────────────
    // policy_name → "name", policy_action → "activity_name",
    // policy_notes → "comment", updated_by → "actor_user_name"
    for expected_col in ["name", "activity_name", "comment", "actor_user_name"] {
        assert!(
            column_names.contains(&expected_col),
            "SAP4-021-FP-3: RecordBatch must contain '{}' (Tier-1 Arrow column). \
             BC-2.16.021 AC-022. Present columns: {:?}",
            expected_col,
            column_names
        );
    }

    // ── Wire-shape assertion 3: raw_extensions present ───────────────────────
    assert!(
        column_names.contains(&"raw_extensions"),
        "SAP4-021-FP-3: raw_extensions must be present (Tier-2 aggregate, ADR-058 §J6). \
         BC-2.16.021 AC-023. Present columns: {:?}",
        column_names
    );
    let raw_ext_col_idx = schema.index_of("raw_extensions").unwrap();
    let raw_ext_array = first_batch
        .column(raw_ext_col_idx)
        .as_any()
        .downcast_ref::<arrow::array::StringArray>()
        .expect("raw_extensions must be StringArray");
    assert!(
        !raw_ext_array.is_null(0),
        "SAP4-021-FP-3: raw_extensions must not be null in row 0. BC-2.16.021 AC-023."
    );
    let raw_ext_json: serde_json::Value = serde_json::from_str(raw_ext_array.value(0))
        .expect("SAP4-021-FP-3: raw_extensions must be valid JSON");
    assert!(
        raw_ext_json.is_object(),
        "SAP4-021-FP-3: raw_extensions must be a JSON object. Got: {:?}.",
        raw_ext_json
    );
    let raw_ext_obj = raw_ext_json.as_object().unwrap();

    // ── Wire-shape assertion 4 (LOAD-BEARING SAP-4): three JSON columns as NATIVE arrays ──
    // communication_conditions, related_alerts_ids, applied_group_pairs must each be
    // a NATIVE JSON array in raw_extensions — NOT stringified.
    // NOTE: applied_group_pairs is the firewall-domain column; zone-domain uses applied_zone_pairs.
    let json_col_checks = [
        (
            "communication_conditions",
            "array of src/dst firewall-group condition objects",
        ),
        ("related_alerts_ids", "array of triggered alert ID strings"),
        (
            "applied_group_pairs",
            "array of {src_group, dst_group} pair objects (FW domain — NOT applied_zone_pairs)",
        ),
    ];
    for (col_name, description) in &json_col_checks {
        let col_val = raw_ext_obj.get(*col_name).unwrap_or_else(|| {
            panic!(
                "SAP4-021-FP-3 LOAD-BEARING: '{}' ({}) MUST be present in raw_extensions. \
                 column_type = \"json\" → ColumnType::Json arm in build_column_array. \
                 BC-2.16.021 AC-026; §PC6.",
                col_name, description
            )
        });
        assert!(
            col_val.is_array(),
            "SAP4-021-FP-3 LOAD-BEARING: '{}' in raw_extensions MUST be a NATIVE JSON array \
             (not stringified). Production path: build_column_array ColumnType::Json arm. \
             Got: {:?}. BC-2.16.021 §PC6.",
            col_name,
            col_val
        );
    }

    // ── Wire-shape assertion 5: Tier-2 columns NOT at top level ──────────────
    let tier2_names = [
        "policy_source",
        "communication_conditions",
        "matching_devices",
        "should_generate_alerts",
        "alert_use_case",
        "related_alerts_ids",
        "applied_group_pairs",
        "created_time",
        "last_updated",
    ];
    for tier2_name in &tier2_names {
        assert!(
            !column_names.contains(tier2_name),
            "SAP4-021-FP-3: Tier-2 column '{}' MUST NOT appear as top-level RecordBatch \
             column (ADR-058 §J6). Top-level columns: {:?}",
            tier2_name,
            column_names
        );
    }
}

// ---------------------------------------------------------------------------
// SAP4-021-FP-4: firewall_policies null-not-absent
// ---------------------------------------------------------------------------

/// SAP-4 production-path null-not-absent test for `claroty_organization_firewall_policies`:
/// When `policy_name` (REQUIRED) is absent, the row must survive and
/// `"name": null` must appear in serialized JSON (not absent).
///
/// BC-2.16.021 AC-025; BC-2.11.001 EC-11-079.
/// Story: S-CLAROTY-ORGPOLICY-001 (SAP-4 production-path coverage).
#[tokio::test]
async fn test_BC_2_16_021_claroty_organization_firewall_policies_wire_shape_serialized_json_null_not_absent()
 {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/v1/organization_fw_group_policies/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "organization_firewall_policies": [
                {
                    "policy_name": "Named-FW-Policy-001",
                    "policy_action": "Allow",
                    "policy_notes": "Fully specified FW policy",
                    "policy_source": "Custom",
                    "communication_conditions": [{"src_group_id": 1, "dst_group_id": 2}],
                    "matching_devices": 5,
                    "should_generate_alerts": false,
                    "related_alerts_ids": [],
                    "applied_group_pairs": [{"src_group": 1, "dst_group": 2}],
                    "updated_by": "admin@example.com"
                },
                {
                    // policy_name ABSENT — REQUIRED → Arrow null for "name"
                    "policy_action": "Deny",
                    "policy_notes": "No policy_name — REQUIRED field absent",
                    "policy_source": "Recommended",
                    "communication_conditions": [],
                    "related_alerts_ids": ["orphan-fw-001"],
                    "applied_group_pairs": []
                }
            ]
        })))
        .mount(&mock_server)
        .await;

    let adapter = make_claroty_adapter(&mock_server.uri());

    let adapter_spec = SensorAdapterSpec {
        source_table: "claroty_claroty_organization_firewall_policies".to_string(),
        org_id: OrgId::from_uuid(uuid::Uuid::now_v7()),
        #[allow(deprecated)]
        client_id: "claroty-fw-policies-null-test".to_string(),
        sensor_config: serde_json::json!({}),
    };

    let params = QueryParams {
        cursor: None,
        limit: 10,
        start_time: None,
        end_time: None,
        filters: Default::default(),
    };

    let sensor_auth = BearerStaticSensorAuth::new("mock-bearer-token-fw-policies-null-test");

    let batches = adapter
        .fetch(&adapter_spec, &params, &sensor_auth)
        .await
        .expect(
            "SAP4-021-FP-4: fetch() must return Ok for a valid two-record fw_policies mock. \
             BC-2.16.021 AC-025.",
        );

    assert!(
        !batches.batches.is_empty(),
        "SAP4-021-FP-4: fetch() must return at least one RecordBatch. BC-2.16.021 AC-025."
    );

    let mut buf: Vec<u8> = Vec::new();
    let mut writer = arrow_json::writer::WriterBuilder::new()
        .with_explicit_nulls(true)
        .build::<_, arrow_json::writer::JsonArray>(&mut buf);
    for batch in &batches.batches {
        writer
            .write(batch)
            .expect("SAP4-021-FP-4: arrow_json write must not fail");
    }
    writer
        .finish()
        .expect("SAP4-021-FP-4: arrow_json finish must not fail");

    let json_rows: Vec<serde_json::Value> = serde_json::from_slice::<Vec<serde_json::Value>>(&buf)
        .expect("SAP4-021-FP-4: arrow_json output must deserialize as JSON array");

    assert_eq!(
        json_rows.len(),
        2,
        "SAP4-021-FP-4: must have 2 rows — REQUIRED absent must NOT drop row. \
         BC-2.16.021 AC-025."
    );

    assert_eq!(
        json_rows[0].get("name"),
        Some(&serde_json::json!("Named-FW-Policy-001")),
        "SAP4-021-FP-4: row0 'name' must be 'Named-FW-Policy-001'. BC-2.16.021 AC-025."
    );

    let row1_name = json_rows[1].get("name");
    assert!(
        row1_name.is_some(),
        "SAP4-021-FP-4 LOAD-BEARING (null-not-absent): 'name' key MUST be PRESENT in row1 \
         even when 'policy_name' was absent. explicit_nulls=false (DEFAULT) would OMIT this key \
         (C3/H20 defect class). BC-2.11.001 EC-11-079; BC-2.16.021 AC-025. \
         row1 keys: {:?}",
        json_rows[1]
            .as_object()
            .map(|o| o.keys().collect::<Vec<_>>())
    );
    assert_eq!(
        row1_name,
        Some(&serde_json::Value::Null),
        "SAP4-021-FP-4 LOAD-BEARING: 'name' MUST be JSON null in row1. \
         BC-2.11.001 EC-11-079; BC-2.16.021 AC-025."
    );
}
