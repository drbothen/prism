// SPDX-License-Identifier: Apache-2.0
//! SAP-4 production-path wire-shape tests for BC-2.16.020 —
//! Claroty xDome Organization Zone Domain (zones + zone policies).
//!
//! # SAP-4 Gap Closed
//!
//! `crates/prism-sensors/tests/bc_2_16_020_claroty_org_zone_policy.rs` tests RG-006,
//! RG-007, RG-013, and RG-014 via `ColumnMapper::map_record` directly (pre-serialization).
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
//!     `applied_zone_pairs`) materializing as NATIVE JSON array/object in `raw_extensions`
//!     (not stringified) through the generic `ColumnType::Json` arm in `build_column_array`.
//!   - Required-field-absent → null-cell → row-survives → `"name": null` at wire level
//!     (not absent — BC-2.11.001 EC-11-079 null-not-absent discipline).
//!
//! # Tests in this file
//!
//! | ID              | Test name | Assertion |
//! |-----------------|-----------|-----------|
//! | SAP4-020-Z-1    | test_BC_2_16_020_claroty_organization_zones_wire_shape_class_uid_3004_mock | class_uid=3004, Tier-1 cols, raw_extensions JSON object, device_conditions native array |
//! | SAP4-020-Z-2    | test_BC_2_16_020_claroty_organization_zones_wire_shape_serialized_json_null_not_absent | null-not-absent for zone_name absent → "name":null in serialized JSON |
//! | SAP4-020-ZP-3   | test_BC_2_16_020_claroty_organization_zone_policies_wire_shape_class_uid_3004_mock | class_uid=3004, Tier-1 cols, raw_extensions JSON object, communication_conditions/related_alerts_ids/applied_zone_pairs native arrays |
//! | SAP4-020-ZP-4   | test_BC_2_16_020_claroty_organization_zone_policies_wire_shape_serialized_json_null_not_absent | null-not-absent for policy_name absent → "name":null in serialized JSON |
//!
//! # SID-1 Compliance
//!
//! Tests RG-005 and RG-011 in prism-sensors are `#[ignore]`'d (live-only). This file
//! provides non-live SID-1-compliant coverage via wiremock + mock server, exercising
//! the real production `SpecDrivenSensorAdapter::fetch` code path.
//!
//! # Source Table Name Note
//!
//! TOML entries for org policy tables use bare names (e.g. `table_name = "organization_zones"`),
//! matching the convention of sibling tables. `SpecDrivenSensorAdapter::fetch` prepends
//! `sensor_id + "_"` to produce `source_table`, yielding the single-prefixed queryable name
//! `"claroty_organization_zones"`. Tests pass `source_table = "claroty_organization_zones"`
//! (single prefix — no double-prefix).
//!
//! BC: BC-2.16.020
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
use prism_core::{OrgId, OrgSlug, PrismError};
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
// SAP-2 marker — no DTU clone exists for these org zone policy tables
// ---------------------------------------------------------------------------

/// SAP-2 compliance: no DTU clone exists for claroty_organization_zones
/// or claroty_organization_zone_policies. Parity check deferred to
/// S-CLAROTY-ORGPOLICY-DTU-001 (D-2200 tracking entry).
#[allow(dead_code)]
const SAP2_STATUS: &str = "N/A: no DTU clone exists for claroty_organization_zones and \
     claroty_organization_zone_policies; deferred to D-2200 (S-CLAROTY-ORGPOLICY-DTU-001)";

/// SAP-2 marker test: confirms SAP2_STATUS is properly documented as N/A
/// (no DTU clone exists) with a D-2200 deferral anchor.
///
/// BC-2.16.020; Story: S-CLAROTY-ORGPOLICY-001
#[test]
fn test_BC_2_16_020_claroty_org_zone_policy_wire_shape_sap2_na_documented() {
    assert!(
        SAP2_STATUS.starts_with("N/A:"),
        "SAP2_STATUS must begin with 'N/A:' to document the absence of a DTU clone. \
         BC-2.16.020 CR-001. Got: {:?}",
        SAP2_STATUS
    );
    assert!(
        SAP2_STATUS.contains("D-2200"),
        "SAP2_STATUS must cite D-2200 (S-CLAROTY-ORGPOLICY-DTU-001 deferral anchor). \
         BC-2.16.020 CR-001. Got: {:?}",
        SAP2_STATUS
    );
}

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
        "bc_2_16_020_wire_shape: claroty.sensor.toml must be readable from \
         CARGO_MANIFEST_DIR/../prism-sensors/specs/",
    );

    let mut spec = SpecLoader::parse(&spec_content)
        .expect("bc_2_16_020_wire_shape: claroty.sensor.toml must parse");
    spec.base_url = mock_server_uri.to_string();

    let overlay_toml =
        "extends = \"claroty\"\ninstance_id = \"claroty@claroty-orgpolicy-wire-test\"";
    let overlay: SensorInstanceOverlay = toml::from_str(overlay_toml)
        .expect("bc_2_16_020_wire_shape: SensorInstanceOverlay TOML parse failed");
    let resolved = OverlayLoader::merge_overlay_onto_type_spec(
        &spec,
        &overlay,
        OrgSlug::new("claroty-orgpolicy-wire-test"),
    );

    let http_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("bc_2_16_020_wire_shape: reqwest::Client build failed (ADR-050 rustls-tls)");

    SpecDrivenSensorAdapter::new(
        Arc::new(resolved),
        AdapterAuthStrategy::BearerStatic,
        http_client,
    )
}

/// Minimal no-op `CredentialStore` for constructing `QueryEngine` in SAP-3 tests.
/// Matches the `NoopCs` pattern from `bc_2_16_015_claroty_vulnerabilities_wire_shape.rs`.
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
// SAP4-020-Z-1: zones wire-shape mock test
// ---------------------------------------------------------------------------

/// SAP-4 production-path wire-shape test for `claroty_organization_zones`:
/// `SpecDrivenSensorAdapter::fetch()` returns RecordBatches with:
///   - `class_uid == 3004` (entity_management; ADR-058 §C2; BC-2.16.020 §PC1)
///   - `name` column present (zone_name → ocsf_field "name" → Arrow name "name")
///   - `comment` column present (zone_description → ocsf_field "comment")
///   - `status_code` column present (enabled → ocsf_field "status_code")
///   - `actor_user_name` column present (updated_by → ocsf_field "actor.user.name" → Arrow "actor_user_name")
///   - `raw_extensions` present as StringArray holding a JSON object (Tier-2 aggregate)
///   - `device_conditions` present in `raw_extensions` as a NATIVE JSON array (NOT stringified)
///     — this asserts the generic `ColumnType::Json` arm in `build_column_array` fires correctly
///   - No Tier-2 column names at top level (ADR-058 §J6 Tier-2 isolation)
///
/// SAP-4 gap closure: `test_BC_2_16_020_claroty_organization_zones_device_conditions_json_not_string`
/// (prism-sensors) uses `ColumnMapper::map_record` which is NEVER called in production.
/// This test exercises the REAL production path via `SpecDrivenSensorAdapter::fetch`.
///
/// BC-2.16.020 AC-001/AC-002/AC-003/AC-006; ADR-058 §J6.
/// Story: S-CLAROTY-ORGPOLICY-001 (SAP-4 production-path coverage).
#[tokio::test]
async fn test_BC_2_16_020_claroty_organization_zones_wire_shape_class_uid_3004_mock() {
    let mock_server = MockServer::start().await;

    // Mock the organization_zones POST endpoint.
    // path_template = "/api/v1/organization_zones/" → envelope key = "organization_zones"
    Mock::given(method("POST"))
        .and(path("/api/v1/organization_zones/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "organization_zones": [{
                "zone_name": "Production OT Zone",
                "zone_description": "Primary production OT zone",
                "zone_source": "Custom",
                "priority": 1,
                "enabled": true,
                "device_conditions": [
                    {"type": "ip_range", "value": "10.0.0.0/8"},
                    {"type": "asset_type", "value": "plc"}
                ],
                "attributed_devices": 42,
                "exportable_attributed_devices": 42,
                "created_time": "2024-01-01T00:00:00Z",
                "last_update": "2024-06-01T00:00:00Z",
                "updated_by": "analyst@example.com"
            }]
        })))
        .mount(&mock_server)
        .await;

    let adapter = make_claroty_adapter(&mock_server.uri());

    // source_table: strip sensor_id prefix "claroty_" from source_table to get raw table name.
    // TOML table_name = "organization_zones" (bare, no sensor prefix — same as sibling tables).
    // source_table = "claroty_" + "organization_zones" = "claroty_organization_zones" (single prefix).
    // See: spec_driven_adapter.rs §source_table resolution logic.
    let adapter_spec = SensorAdapterSpec {
        source_table: "claroty_organization_zones".to_string(),
        org_id: OrgId::from_uuid(uuid::Uuid::now_v7()),
        #[allow(deprecated)]
        client_id: "claroty-orgpolicy-zones-wire-test".to_string(),
        sensor_config: serde_json::json!({}),
    };

    let params = QueryParams {
        cursor: None,
        limit: 10,
        start_time: None,
        end_time: None,
        filters: Default::default(),
    };

    let sensor_auth = BearerStaticSensorAuth::new("mock-bearer-token-zones-wire-test");

    let result = adapter.fetch(&adapter_spec, &params, &sensor_auth).await;
    assert!(
        result.is_ok(),
        "SAP4-020-Z-1: fetch() must return Ok for a valid organization_zones response. \
         Got Err: {:?}. BC-2.16.020 AC-001.",
        result.err()
    );

    let batches = result.unwrap();
    assert!(
        !batches.batches.is_empty(),
        "SAP4-020-Z-1: fetch() must return at least one RecordBatch. BC-2.16.020 AC-001."
    );

    let first_batch = &batches.batches[0];
    assert!(
        first_batch.num_rows() > 0,
        "SAP4-020-Z-1: RecordBatch must contain at least one row. BC-2.16.020 AC-001."
    );

    let schema = first_batch.schema();
    let column_names: Vec<&str> = schema.fields().iter().map(|f| f.name().as_str()).collect();

    // ── Wire-shape assertion 1: class_uid == 3004 ────────────────────────────
    // entity_management class_uid = 3004 (BC-2.16.020 §PC1; ADR-058 §C2).
    let class_uid_col_idx = schema.index_of("class_uid").expect(
        "SAP4-020-Z-1: RecordBatch must contain 'class_uid' column (OCSF synthesized field). \
         BC-2.16.020 AC-001.",
    );
    let class_uid_col = first_batch.column(class_uid_col_idx);
    let class_uid_array = class_uid_col
        .as_any()
        .downcast_ref::<arrow::array::Int32Array>()
        .expect("SAP4-020-Z-1: class_uid column must be Int32Array");
    let class_uid_val = class_uid_array.value(0);
    assert_eq!(
        class_uid_val, 3004,
        "SAP4-020-Z-1: class_uid MUST equal 3004 (entity_management; BC-2.16.020 §PC1). \
         Got: {}.",
        class_uid_val
    );

    // ── Wire-shape assertion 2: Tier-1 Arrow columns present ─────────────────
    // zone_name → ocsf_field "name" → Arrow "name"
    assert!(
        column_names.contains(&"name"),
        "SAP4-020-Z-1: RecordBatch must contain 'name' column \
         (zone_name → ocsf_field=name → Arrow name). BC-2.16.020 AC-002. \
         Present columns: {:?}",
        column_names
    );
    // zone_description → ocsf_field "comment" → Arrow "comment"
    assert!(
        column_names.contains(&"comment"),
        "SAP4-020-Z-1: RecordBatch must contain 'comment' column \
         (zone_description → ocsf_field=comment). BC-2.16.020 AC-002. \
         Present columns: {:?}",
        column_names
    );
    // enabled → ocsf_field "status_code" → Arrow "status_code"
    assert!(
        column_names.contains(&"status_code"),
        "SAP4-020-Z-1: RecordBatch must contain 'status_code' column \
         (enabled → ocsf_field=status_code). BC-2.16.020 AC-002. \
         Present columns: {:?}",
        column_names
    );
    // updated_by → ocsf_field "actor.user.name" → Arrow "actor_user_name" (dot → underscore)
    assert!(
        column_names.contains(&"actor_user_name"),
        "SAP4-020-Z-1: RecordBatch must contain 'actor_user_name' column \
         (updated_by → ocsf_field=actor.user.name → Arrow actor_user_name). \
         BC-2.16.020 AC-002. Present columns: {:?}",
        column_names
    );

    // ── Wire-shape assertion 3: raw_extensions present as StringArray ─────────
    assert!(
        column_names.contains(&"raw_extensions"),
        "SAP4-020-Z-1: RecordBatch must contain 'raw_extensions' column \
         (Tier-2 aggregation, ADR-058 §J6). BC-2.16.020 AC-003. \
         Present columns: {:?}",
        column_names
    );
    let raw_ext_col_idx = schema
        .index_of("raw_extensions")
        .expect("raw_extensions column must be present (asserted above)");
    let raw_ext_col = first_batch.column(raw_ext_col_idx);
    let raw_ext_array = raw_ext_col
        .as_any()
        .downcast_ref::<arrow::array::StringArray>()
        .expect("SAP4-020-Z-1: raw_extensions column must be StringArray (DataType::Utf8)");
    assert!(
        !raw_ext_array.is_null(0),
        "SAP4-020-Z-1: raw_extensions must not be null in row 0 when Tier-2 data is present. \
         BC-2.16.020 AC-003."
    );
    let raw_ext_str = raw_ext_array.value(0);
    let raw_ext_json: serde_json::Value = serde_json::from_str(raw_ext_str)
        .expect("SAP4-020-Z-1: raw_extensions value must be valid JSON (DataType::Utf8 JSON blob)");
    assert!(
        raw_ext_json.is_object(),
        "SAP4-020-Z-1: raw_extensions must deserialize to a JSON object. \
         Got: {:?}. BC-2.16.020 AC-003; ADR-058 §J6.",
        raw_ext_json
    );

    // ── Wire-shape assertion 4 (LOAD-BEARING SAP-4): device_conditions as NATIVE JSON array ──
    // This is the authoritative production-path gate for BC-2.16.020 §PC6 Json arm.
    // `test_BC_2_16_020_claroty_organization_zones_device_conditions_json_not_string` (prism-sensors)
    // tests only `ColumnMapper::map_record` — it never calls `build_column_array`.
    // This assertion verifies the generic `ColumnType::Json` arm in `build_column_array`
    // (spec_driven_adapter.rs `pipeline_result_to_record_batch`) correctly:
    //   - Value::Array → preserved as native JSON array (not stringified)
    // Per BC-2.16.020 §PC6 and EC-016-020-003 (AC-006).
    let raw_ext_obj = raw_ext_json
        .as_object()
        .expect("SAP4-020-Z-1: raw_extensions must be a JSON object (asserted above)");
    let device_cond = raw_ext_obj.get("device_conditions").expect(
        "SAP4-020-Z-1 LOAD-BEARING: 'device_conditions' MUST be present in raw_extensions. \
             column_type = \"json\" → ColumnType::Json arm in build_column_array. \
             BC-2.16.020 AC-006; §PC6.",
    );
    assert!(
        device_cond.is_array(),
        "SAP4-020-Z-1 LOAD-BEARING: 'device_conditions' in raw_extensions MUST be a NATIVE \
         JSON array (not a JSON string containing an encoded array). \
         This exercises the generic ColumnType::Json arm in build_column_array. \
         Got: {:?}. BC-2.16.020 §PC6; EC-016-020-003.",
        device_cond
    );
    let device_cond_arr = device_cond.as_array().unwrap();
    assert!(
        !device_cond_arr.is_empty(),
        "SAP4-020-Z-1: device_conditions must be a non-empty array for this mock. \
         Got: [] (empty). BC-2.16.020 §PC6."
    );
    // Verify array elements are objects (not scalars or strings)
    assert!(
        device_cond_arr[0].is_object(),
        "SAP4-020-Z-1: device_conditions[0] must be a JSON object \
         (structured condition). Got: {:?}. BC-2.16.020 §PC6.",
        device_cond_arr[0]
    );

    // ── Wire-shape assertion 5: Tier-2 columns NOT at top level ──────────────
    let tier2_names = [
        "zone_source",
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
            "SAP4-020-Z-1: Tier-2 column '{}' MUST NOT appear as a top-level RecordBatch \
             column (ADR-058 §J6). It must be inside raw_extensions. \
             Top-level columns: {:?}",
            tier2_name,
            column_names
        );
    }
}

// ---------------------------------------------------------------------------
// SAP4-020-Z-2: zones null-not-absent (zone_name absent → "name":null at wire)
// ---------------------------------------------------------------------------

/// SAP-4 production-path null-not-absent test for `claroty_organization_zones`:
/// When a zone record has `zone_name` absent (REQUIRED field), the row must:
///   - Survive (not be dropped): RecordBatch row count = 2 (for 2-record mock)
///   - Produce Arrow null for the `name` column
///   - Emit `"name": null` (NOT absent) in serialized JSON via explicit_nulls=true
///
/// LOAD-BEARING: verifies BC-2.11.001 EC-11-079 null-not-absent discipline on the
/// production path (`SpecDrivenSensorAdapter::fetch` → `arrow_json::WriterBuilder`
/// `.with_explicit_nulls(true)`), not just at the `ColumnMapper::map_record` level.
///
/// BC-2.16.020 AC-007; BC-2.11.001 EC-11-079; CLAUDE.md §Wire-shape assertion discipline.
/// Story: S-CLAROTY-ORGPOLICY-001 (SAP-4 production-path coverage).
#[tokio::test]
async fn test_BC_2_16_020_claroty_organization_zones_wire_shape_serialized_json_null_not_absent() {
    let mock_server = MockServer::start().await;

    // Two records: first has zone_name (REQUIRED), second does NOT.
    Mock::given(method("POST"))
        .and(path("/api/v1/organization_zones/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "organization_zones": [
                {
                    "zone_name": "Zone With Name",
                    "zone_description": "Has all required fields",
                    "zone_source": "Custom",
                    "priority": 1,
                    "enabled": true,
                    "device_conditions": [{"type": "ip_range", "value": "192.168.0.0/16"}],
                    "attributed_devices": 5,
                    "exportable_attributed_devices": 5,
                    "updated_by": "analyst@example.com"
                },
                {
                    // zone_name ABSENT — REQUIRED field missing → Arrow null for "name"
                    "zone_description": "Zone without a name (REQUIRED field absent)",
                    "zone_source": "Recommended",
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
        source_table: "claroty_organization_zones".to_string(),
        org_id: OrgId::from_uuid(uuid::Uuid::now_v7()),
        #[allow(deprecated)]
        client_id: "claroty-zones-null-test".to_string(),
        sensor_config: serde_json::json!({}),
    };

    let params = QueryParams {
        cursor: None,
        limit: 10,
        start_time: None,
        end_time: None,
        filters: Default::default(),
    };

    let sensor_auth = BearerStaticSensorAuth::new("mock-bearer-token-zones-null-test");

    let batches = adapter
        .fetch(&adapter_spec, &params, &sensor_auth)
        .await
        .expect(
            "SAP4-020-Z-2: fetch() must return Ok for a valid two-record org zones mock. \
             BC-2.16.020 AC-007.",
        );

    assert!(
        !batches.batches.is_empty(),
        "SAP4-020-Z-2: fetch() must return at least one RecordBatch. BC-2.16.020 AC-007."
    );

    // Serialize through the production MCP path (explicit_nulls=true).
    let mut buf: Vec<u8> = Vec::new();
    let mut writer = arrow_json::writer::WriterBuilder::new()
        .with_explicit_nulls(true)
        .build::<_, arrow_json::writer::JsonArray>(&mut buf);
    for batch in &batches.batches {
        writer
            .write(batch)
            .expect("SAP4-020-Z-2: arrow_json write must not fail for org zones RecordBatch");
    }
    writer
        .finish()
        .expect("SAP4-020-Z-2: arrow_json finish must not fail");

    let json_rows: Vec<serde_json::Value> = serde_json::from_slice::<Vec<serde_json::Value>>(&buf)
        .expect("SAP4-020-Z-2: arrow_json output must deserialize as a JSON array of row objects");

    assert_eq!(
        json_rows.len(),
        2,
        "SAP4-020-Z-2: serialized JSON must contain exactly 2 rows (one per mock record). \
         REQUIRED field absent must NOT drop the row. BC-2.16.020 AC-007."
    );

    // ── Row 0: zone_name present → "name" non-null ───────────────────────────
    let row0 = &json_rows[0];
    assert_eq!(
        row0.get("class_uid"),
        Some(&serde_json::json!(3004_i32)),
        "SAP4-020-Z-2: row0 class_uid must equal 3004 (entity_management). \
         BC-2.16.020 AC-001; ADR-058 §C2."
    );
    assert_eq!(
        row0.get("name"),
        Some(&serde_json::json!("Zone With Name")),
        "SAP4-020-Z-2: row0 'name' must be 'Zone With Name' (zone_name → Arrow name). \
         BC-2.16.020 AC-007."
    );
    // ── CR-004: all Tier-1 column values for row0 ────────────────────────────
    assert_eq!(
        row0.get("comment"),
        Some(&serde_json::json!("Has all required fields")),
        "SAP4-020-Z-2 CR-004: row0 'comment' must match mock fixture value \
         (zone_description='Has all required fields' → comment). \
         BC-2.16.020 AC-007; catches zone_description→comment Arrow-name rename."
    );
    // enabled=true → Boolean column_type → Arrow BooleanArray → JSON boolean true
    assert_eq!(
        row0.get("status_code"),
        Some(&serde_json::json!(true)),
        "SAP4-020-Z-2 CR-004: row0 'status_code' must be JSON boolean true \
         (enabled=true, column_type=boolean → BooleanArray serializes as JSON true). \
         BC-2.16.020 AC-007; catches enabled→status_code Arrow-name rename."
    );
    assert_eq!(
        row0.get("actor_user_name"),
        Some(&serde_json::json!("analyst@example.com")),
        "SAP4-020-Z-2 CR-004: row0 'actor_user_name' must match mock fixture value \
         (updated_by='analyst@example.com' → actor.user.name → Arrow actor_user_name). \
         BC-2.16.020 AC-007; catches updated_by→actor.user.name→actor_user_name rename."
    );

    // ── Row 1: zone_name absent → "name": null (NOT absent) ──────────────────
    // NULL-NOT-ABSENT LOAD-BEARING: with explicit_nulls=true, Arrow null cell MUST
    // appear as `"name": null` in JSON — NOT be omitted. explicit_nulls=false (DEFAULT)
    // would omit the key (C3/H20 defect class; BC-2.11.001 EC-11-079).
    let row1 = &json_rows[1];
    let row1_name = row1.get("name");
    assert!(
        row1_name.is_some(),
        "SAP4-020-Z-2 LOAD-BEARING (null-not-absent): 'name' key MUST be PRESENT in row1 \
         serialized JSON even when 'zone_name' was absent in the API response. \
         arrow_json with explicit_nulls=false (DEFAULT) would OMIT this key. \
         BC-2.11.001 EC-11-079; BC-2.16.020 AC-007. \
         row1 keys: {:?}",
        row1.as_object().map(|o| o.keys().collect::<Vec<_>>())
    );
    assert_eq!(
        row1_name,
        Some(&serde_json::Value::Null),
        "SAP4-020-Z-2 LOAD-BEARING (null-not-absent): 'name' MUST be JSON null in row1. \
         BC-2.11.001 EC-11-079; BC-2.16.020 AC-007."
    );
}

// ---------------------------------------------------------------------------
// SAP4-020-ZP-3: zone_policies wire-shape mock test
// ---------------------------------------------------------------------------

/// SAP-4 production-path wire-shape test for `claroty_organization_zone_policies`:
/// `SpecDrivenSensorAdapter::fetch()` returns RecordBatches with:
///   - `class_uid == 3004` (entity_management; BC-2.16.020 §PC2)
///   - `name` column (policy_name → "name"), `activity_name` (policy_action → "activity_name"),
///     `comment` (policy_notes → "comment"), `actor_user_name` (updated_by → "actor.user.name")
///   - `raw_extensions` present with all three JSON columns as NATIVE arrays (not stringified):
///     - `communication_conditions` (array of src/dst zone condition objects)
///     - `related_alerts_ids` (array of alert ID strings)
///     - `applied_zone_pairs` (array of {src_zone, dst_zone} pair objects)
///   - No Tier-2 column names at top level (ADR-058 §J6)
///
/// SAP-4 gap closure: `test_BC_2_16_020_claroty_organization_zone_policies_json_columns_not_stringified`
/// (prism-sensors) uses `ColumnMapper::map_record` only. This test exercises the REAL
/// production path via `SpecDrivenSensorAdapter::fetch`.
///
/// BC-2.16.020 AC-009/AC-010/AC-011/AC-014; ADR-058 §J6.
/// Story: S-CLAROTY-ORGPOLICY-001 (SAP-4 production-path coverage).
#[tokio::test]
async fn test_BC_2_16_020_claroty_organization_zone_policies_wire_shape_class_uid_3004_mock() {
    let mock_server = MockServer::start().await;

    // Mock the organization_zone_policies POST endpoint.
    // path_template = "/api/v1/organization_zone_policies/" → envelope key = "organization_zone_policies"
    Mock::given(method("POST"))
        .and(path("/api/v1/organization_zone_policies/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "organization_zone_policies": [{
                "policy_name": "Allow-Zone-A-to-Zone-B",
                "policy_source": "Custom",
                "policy_action": "Allow",
                "communication_conditions": [
                    {"src_zone_id": 1, "dst_zone_id": 2, "protocol": "tcp", "port": 443}
                ],
                "matching_devices": 8,
                "should_generate_alerts": false,
                "alert_use_case": "Unknown Communication",
                "policy_notes": "Permit TLS traffic between production zones",
                "related_alerts_ids": ["alert-001", "alert-002", "alert-003"],
                "applied_zone_pairs": [
                    {"src_zone": 1, "dst_zone": 2},
                    {"src_zone": 3, "dst_zone": 4}
                ],
                "created_time": "2024-01-01T00:00:00Z",
                "last_updated": "2024-06-01T00:00:00Z",
                "updated_by": "policy-admin@example.com"
            }]
        })))
        .mount(&mock_server)
        .await;

    let adapter = make_claroty_adapter(&mock_server.uri());

    // TOML table_name = "organization_zone_policies" (bare, no sensor prefix).
    // source_table = "claroty_" + "organization_zone_policies" = "claroty_organization_zone_policies".
    let adapter_spec = SensorAdapterSpec {
        source_table: "claroty_organization_zone_policies".to_string(),
        org_id: OrgId::from_uuid(uuid::Uuid::now_v7()),
        #[allow(deprecated)]
        client_id: "claroty-zone-policies-wire-test".to_string(),
        sensor_config: serde_json::json!({}),
    };

    let params = QueryParams {
        cursor: None,
        limit: 10,
        start_time: None,
        end_time: None,
        filters: Default::default(),
    };

    let sensor_auth = BearerStaticSensorAuth::new("mock-bearer-token-zone-policies-wire-test");

    let result = adapter.fetch(&adapter_spec, &params, &sensor_auth).await;
    assert!(
        result.is_ok(),
        "SAP4-020-ZP-3: fetch() must return Ok for a valid organization_zone_policies response. \
         Got Err: {:?}. BC-2.16.020 AC-009.",
        result.err()
    );

    let batches = result.unwrap();
    assert!(
        !batches.batches.is_empty(),
        "SAP4-020-ZP-3: fetch() must return at least one RecordBatch. BC-2.16.020 AC-009."
    );

    let first_batch = &batches.batches[0];
    assert!(
        first_batch.num_rows() > 0,
        "SAP4-020-ZP-3: RecordBatch must contain at least one row. BC-2.16.020 AC-009."
    );

    let schema = first_batch.schema();
    let column_names: Vec<&str> = schema.fields().iter().map(|f| f.name().as_str()).collect();

    // ── Wire-shape assertion 1: class_uid == 3004 ────────────────────────────
    let class_uid_col_idx = schema
        .index_of("class_uid")
        .expect("SAP4-020-ZP-3: RecordBatch must contain 'class_uid'. BC-2.16.020 AC-009.");
    let class_uid_array = first_batch
        .column(class_uid_col_idx)
        .as_any()
        .downcast_ref::<arrow::array::Int32Array>()
        .expect("class_uid must be Int32Array");
    assert_eq!(
        class_uid_array.value(0),
        3004,
        "SAP4-020-ZP-3: class_uid MUST equal 3004 (entity_management). BC-2.16.020 §PC2."
    );

    // ── Wire-shape assertion 2: Tier-1 Arrow columns ─────────────────────────
    // policy_name → "name", policy_action → "activity_name",
    // policy_notes → "comment", updated_by → "actor_user_name"
    for expected_col in ["name", "activity_name", "comment", "actor_user_name"] {
        assert!(
            column_names.contains(&expected_col),
            "SAP4-020-ZP-3: RecordBatch must contain '{}' (Tier-1 Arrow column). \
             BC-2.16.020 AC-010. Present columns: {:?}",
            expected_col,
            column_names
        );
    }

    // ── Wire-shape assertion 3: raw_extensions present ───────────────────────
    assert!(
        column_names.contains(&"raw_extensions"),
        "SAP4-020-ZP-3: raw_extensions must be present (Tier-2 aggregate, ADR-058 §J6). \
         BC-2.16.020 AC-011. Present columns: {:?}",
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
        "SAP4-020-ZP-3: raw_extensions must not be null in row 0. BC-2.16.020 AC-011."
    );
    let raw_ext_json: serde_json::Value = serde_json::from_str(raw_ext_array.value(0))
        .expect("SAP4-020-ZP-3: raw_extensions must be valid JSON");
    assert!(
        raw_ext_json.is_object(),
        "SAP4-020-ZP-3: raw_extensions must be a JSON object. Got: {:?}.",
        raw_ext_json
    );
    let raw_ext_obj = raw_ext_json.as_object().unwrap();

    // ── Wire-shape assertion 4 (LOAD-BEARING SAP-4): three JSON columns as NATIVE arrays ──
    // communication_conditions, related_alerts_ids, applied_zone_pairs must each be
    // a NATIVE JSON array in raw_extensions — NOT stringified. This exercises the generic
    // ColumnType::Json arm in build_column_array on the production fetch path.
    let json_col_checks = [
        (
            "communication_conditions",
            "array of src/dst zone condition objects",
        ),
        ("related_alerts_ids", "array of triggered alert ID strings"),
        (
            "applied_zone_pairs",
            "array of {src_zone, dst_zone} pair objects",
        ),
    ];
    for (col_name, description) in &json_col_checks {
        let col_val = raw_ext_obj.get(*col_name).unwrap_or_else(|| {
            panic!(
                "SAP4-020-ZP-3 LOAD-BEARING: '{}' ({}) MUST be present in raw_extensions. \
                 column_type = \"json\" → ColumnType::Json arm in build_column_array. \
                 BC-2.16.020 AC-014; §PC6.",
                col_name, description
            )
        });
        assert!(
            col_val.is_array(),
            "SAP4-020-ZP-3 LOAD-BEARING: '{}' in raw_extensions MUST be a NATIVE JSON array \
             (not stringified). This asserts the generic ColumnType::Json arm in \
             build_column_array (production path). Got: {:?}. BC-2.16.020 §PC6.",
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
        "applied_zone_pairs",
        "created_time",
        "last_updated",
    ];
    for tier2_name in &tier2_names {
        assert!(
            !column_names.contains(tier2_name),
            "SAP4-020-ZP-3: Tier-2 column '{}' MUST NOT appear as top-level RecordBatch \
             column (ADR-058 §J6). Top-level columns: {:?}",
            tier2_name,
            column_names
        );
    }
}

// ---------------------------------------------------------------------------
// SAP4-020-ZP-4: zone_policies null-not-absent (policy_name absent → "name":null)
// ---------------------------------------------------------------------------

/// SAP-4 production-path null-not-absent test for `claroty_organization_zone_policies`:
/// When `policy_name` (REQUIRED) is absent, the row must survive and
/// `"name": null` must appear in serialized JSON (not absent).
///
/// BC-2.16.020 AC-013; BC-2.11.001 EC-11-079.
/// Story: S-CLAROTY-ORGPOLICY-001 (SAP-4 production-path coverage).
#[tokio::test]
async fn test_BC_2_16_020_claroty_organization_zone_policies_wire_shape_serialized_json_null_not_absent()
 {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/v1/organization_zone_policies/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "organization_zone_policies": [
                {
                    "policy_name": "Named-Policy-001",
                    "policy_action": "Allow",
                    "policy_notes": "A fully-specified policy",
                    "policy_source": "Custom",
                    "communication_conditions": [{"src": 1, "dst": 2}],
                    "matching_devices": 3,
                    "should_generate_alerts": false,
                    "related_alerts_ids": [],
                    "applied_zone_pairs": [{"src_zone": 1, "dst_zone": 2}],
                    "updated_by": "admin@example.com"
                },
                {
                    // policy_name ABSENT — REQUIRED field → Arrow null for "name"
                    "policy_action": "Deny",
                    "policy_notes": "No policy_name provided",
                    "policy_source": "Recommended",
                    "communication_conditions": [],
                    "related_alerts_ids": ["orphan-001"],
                    "applied_zone_pairs": []
                }
            ]
        })))
        .mount(&mock_server)
        .await;

    let adapter = make_claroty_adapter(&mock_server.uri());

    let adapter_spec = SensorAdapterSpec {
        source_table: "claroty_organization_zone_policies".to_string(),
        org_id: OrgId::from_uuid(uuid::Uuid::now_v7()),
        #[allow(deprecated)]
        client_id: "claroty-zone-policies-null-test".to_string(),
        sensor_config: serde_json::json!({}),
    };

    let params = QueryParams {
        cursor: None,
        limit: 10,
        start_time: None,
        end_time: None,
        filters: Default::default(),
    };

    let sensor_auth = BearerStaticSensorAuth::new("mock-bearer-token-zone-policies-null-test");

    let batches = adapter
        .fetch(&adapter_spec, &params, &sensor_auth)
        .await
        .expect(
            "SAP4-020-ZP-4: fetch() must return Ok for a valid two-record mock. \
             BC-2.16.020 AC-013.",
        );

    assert!(
        !batches.batches.is_empty(),
        "SAP4-020-ZP-4: fetch() must return at least one RecordBatch. BC-2.16.020 AC-013."
    );

    // Serialize through the production path with explicit_nulls=true.
    let mut buf: Vec<u8> = Vec::new();
    let mut writer = arrow_json::writer::WriterBuilder::new()
        .with_explicit_nulls(true)
        .build::<_, arrow_json::writer::JsonArray>(&mut buf);
    for batch in &batches.batches {
        writer
            .write(batch)
            .expect("SAP4-020-ZP-4: arrow_json write must not fail");
    }
    writer
        .finish()
        .expect("SAP4-020-ZP-4: arrow_json finish must not fail");

    let json_rows: Vec<serde_json::Value> = serde_json::from_slice::<Vec<serde_json::Value>>(&buf)
        .expect("SAP4-020-ZP-4: arrow_json output must deserialize as JSON array");

    assert_eq!(
        json_rows.len(),
        2,
        "SAP4-020-ZP-4: serialized JSON must contain 2 rows. \
         REQUIRED policy_name absent MUST NOT drop the row. BC-2.16.020 AC-013."
    );

    // Row 0: policy_name present → "name" non-null
    assert_eq!(
        json_rows[0].get("name"),
        Some(&serde_json::json!("Named-Policy-001")),
        "SAP4-020-ZP-4: row0 'name' must be 'Named-Policy-001'. BC-2.16.020 AC-013."
    );
    // ── CR-004: all Tier-1 column values for row0 ────────────────────────────
    assert_eq!(
        json_rows[0].get("activity_name"),
        Some(&serde_json::json!("Allow")),
        "SAP4-020-ZP-4 CR-004: row0 'activity_name' must be 'Allow' \
         (policy_action='Allow' → activity_name). \
         BC-2.16.020 AC-013; catches policy_action→activity_name Arrow-name rename."
    );
    assert_eq!(
        json_rows[0].get("comment"),
        Some(&serde_json::json!("A fully-specified policy")),
        "SAP4-020-ZP-4 CR-004: row0 'comment' must be 'A fully-specified policy' \
         (policy_notes → comment). \
         BC-2.16.020 AC-013; catches policy_notes→comment Arrow-name rename."
    );
    assert_eq!(
        json_rows[0].get("actor_user_name"),
        Some(&serde_json::json!("admin@example.com")),
        "SAP4-020-ZP-4 CR-004: row0 'actor_user_name' must be 'admin@example.com' \
         (updated_by → actor.user.name → Arrow actor_user_name). \
         BC-2.16.020 AC-013; catches updated_by→actor.user.name→actor_user_name rename."
    );

    // Row 1: policy_name absent → "name": null (LOAD-BEARING null-not-absent)
    let row1_name = json_rows[1].get("name");
    assert!(
        row1_name.is_some(),
        "SAP4-020-ZP-4 LOAD-BEARING (null-not-absent): 'name' key MUST be PRESENT \
         in row1 even when 'policy_name' was absent. explicit_nulls=false (DEFAULT) \
         would omit this key (C3/H20 defect class). BC-2.11.001 EC-11-079; BC-2.16.020 AC-013. \
         row1 keys: {:?}",
        json_rows[1]
            .as_object()
            .map(|o| o.keys().collect::<Vec<_>>())
    );
    assert_eq!(
        row1_name,
        Some(&serde_json::Value::Null),
        "SAP4-020-ZP-4 LOAD-BEARING: 'name' MUST be JSON null in row1. \
         BC-2.11.001 EC-11-079; BC-2.16.020 AC-013."
    );
}

// ---------------------------------------------------------------------------
// SAP-3: End-to-end E-QUERY-038 tests via QueryEngine::execute()
// Authoritative RG-003/RG-012 coverage from the public query surface.
// (F-ORGPOL-P1-MED-001 closure)
// ---------------------------------------------------------------------------

/// SAP-3 reachability test (authoritative, F-ORGPOL-P1-MED-001):
/// `SELECT zone_source FROM claroty_organization_zones LIMIT 1` via
/// `QueryEngine::execute()` must raise `PrismError::ColumnNotFound` (E-QUERY-038).
///
/// This is the AUTHORITATIVE SAP-3 test for BC-2.16.020 AC-003.
///
/// `test_BC_2_16_020_claroty_organization_zones_tier2_column_raises_e_query_038`
/// (RG-003 in prism-sensors) calls `ocsf_projected_column_names` directly —
/// valid defense-in-depth per SAP-3 rule-3, but NOT an end-to-end gate from the
/// public query surface (SQL parser → QueryEngine::execute).
/// This test uses `QueryEngine::execute()` as the SAP-3 entry point.
///
/// Architecture: the E-QUERY-038 gate (`check_query_column_availability`) fires in
/// `QueryEngine::execute_inner` BEFORE `run_materialization_pipeline`. No HTTP requests.
///
/// `zone_source` is Tier-2 (no ocsf_field in claroty.sensor.toml):
///   - NOT in ocsf_projected_column_names → NOT in TableRegistry → E-QUERY-038
///   - available_columns ⊇ {name, comment, status_code, actor_user_name,
///                           raw_extensions, class_uid, _sensor}
///   - available_columns ∌ "zone_source"
///
/// BC-2.16.020 AC-003; SAP-3; ADR-058 §I7; F-ORGPOL-P1-MED-001.
/// Story: S-CLAROTY-ORGPOLICY-001 RG-003a (SAP-3 end-to-end gate).
#[tokio::test]
async fn test_BC_2_16_020_claroty_organization_zones_e2e_e_query_038_tier2_column() {
    let spec_content = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../prism-sensors/specs/claroty.sensor.toml"),
    )
    .expect(
        "RG-003a: claroty.sensor.toml must be readable from \
         CARGO_MANIFEST_DIR/../prism-sensors/specs/",
    );
    let spec = SpecLoader::parse(&spec_content).expect("RG-003a: claroty.sensor.toml must parse");

    let registry = Arc::new(TableRegistry::new());
    registry
        .register_sensor(&spec)
        .expect("RG-003a: register_sensor must not fail for production claroty.sensor.toml");

    let engine = QueryEngine::new_with_cache_config(
        Arc::new(prism_sensors::AdapterRegistry::new()),
        Arc::new(NoopCredentialStore),
        Arc::new(OcsfNormalizer::new()),
        Arc::new(ClientRegistry::new(vec![])),
        QueryEngineConfig::default(),
        CacheConfig::default(),
    )
    .with_table_registry(registry);

    // `zone_source` is Tier-2 (no ocsf_field). Lives inside raw_extensions.
    // E-QUERY-038 (PrismError::ColumnNotFound) must fire at plan-time.
    let result = engine
        .execute(
            "SELECT zone_source FROM claroty_organization_zones LIMIT 1",
            QueryOptions::default(),
        )
        .await;

    assert!(
        result.is_err(),
        "RG-003a LOAD-BEARING (SAP-3): QueryEngine::execute must return Err when \
         Tier-2 column 'zone_source' is queried directly (E-QUERY-038). \
         Got Ok. BC-2.16.020 AC-003; SAP-3.",
    );

    let err = result.unwrap_err();

    match &err {
        PrismError::ColumnNotFound(details) => {
            assert_eq!(
                details.column, "zone_source",
                "RG-003a: ColumnNotFound.column must be 'zone_source'. \
                 Got: {:?}. BC-2.16.020 AC-003.",
                details.column
            );
            let avail = &details.available_columns;
            // raw_extensions (Tier-2 aggregate) must be available.
            assert!(
                avail.contains(&"raw_extensions".to_string()),
                "RG-003a: available_columns must include 'raw_extensions' \
                 (ADR-058 §J6 Tier-2 aggregate). Got: {:?}",
                avail
            );
            // All four Tier-1 OCSF projected Arrow names must be available.
            for expected in ["name", "comment", "status_code", "actor_user_name"] {
                assert!(
                    avail.contains(&expected.to_string()),
                    "RG-003a: available_columns must include '{}' \
                     (Tier-1 Arrow column, claroty_organization_zones). Got: {:?}",
                    expected,
                    avail
                );
            }
            // zone_source is Tier-2 → MUST NOT appear in available_columns.
            assert!(
                !avail.contains(&"zone_source".to_string()),
                "RG-003a: 'zone_source' is Tier-2 and MUST NOT appear in \
                 available_columns (lives inside raw_extensions). Got: {:?}",
                avail
            );
            // Synthesized pseudo-columns must also be listed.
            assert!(
                avail.contains(&"class_uid".to_string()),
                "RG-003a: available_columns must include 'class_uid' \
                 (OCSF synthesized pseudo-column). Got: {:?}.",
                avail
            );
            assert!(
                avail.contains(&"_sensor".to_string()),
                "RG-003a: available_columns must include '_sensor' \
                 (synthesized sensor-metadata pseudo-column). Got: {:?}.",
                avail
            );
        }
        other => {
            panic!(
                "RG-003a LOAD-BEARING (SAP-3): QueryEngine::execute must return \
                 PrismError::ColumnNotFound (E-QUERY-038) when Tier-2 column \
                 'zone_source' is queried directly. Got: {:?}. \
                 BC-2.16.020 AC-003; SAP-3.",
                other
            );
        }
    }
}

/// SAP-3 reachability test (authoritative, F-ORGPOL-P1-MED-001):
/// `SELECT applied_zone_pairs FROM claroty_organization_zone_policies LIMIT 1` via
/// `QueryEngine::execute()` must raise `PrismError::ColumnNotFound` (E-QUERY-038).
///
/// This is the AUTHORITATIVE SAP-3 test for BC-2.16.020 AC-012.
///
/// `test_BC_2_16_020_claroty_organization_zone_policies_applied_zone_pairs_raises_e_query_038`
/// (RG-012 in prism-sensors) calls `ocsf_projected_column_names` directly —
/// valid defense-in-depth per SAP-3 rule-3, but NOT an end-to-end gate from the
/// public query surface (SQL parser → QueryEngine::execute).
///
/// `applied_zone_pairs` is Tier-2 Json (no ocsf_field in claroty.sensor.toml):
///   - NOT in ocsf_projected_column_names → NOT in TableRegistry → E-QUERY-038
///   - available_columns ⊇ {name, activity_name, comment, actor_user_name,
///                           raw_extensions, class_uid, _sensor}
///   - available_columns ∌ "applied_zone_pairs"
///
/// BC-2.16.020 AC-012; SAP-3; ADR-058 §I7; F-ORGPOL-P1-MED-001.
/// Story: S-CLAROTY-ORGPOLICY-001 RG-012a (SAP-3 end-to-end gate).
#[tokio::test]
async fn test_BC_2_16_020_claroty_organization_zone_policies_e2e_e_query_038_tier2_column() {
    let spec_content = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../prism-sensors/specs/claroty.sensor.toml"),
    )
    .expect(
        "RG-012a: claroty.sensor.toml must be readable from \
         CARGO_MANIFEST_DIR/../prism-sensors/specs/",
    );
    let spec = SpecLoader::parse(&spec_content).expect("RG-012a: claroty.sensor.toml must parse");

    let registry = Arc::new(TableRegistry::new());
    registry
        .register_sensor(&spec)
        .expect("RG-012a: register_sensor must not fail for production claroty.sensor.toml");

    let engine = QueryEngine::new_with_cache_config(
        Arc::new(prism_sensors::AdapterRegistry::new()),
        Arc::new(NoopCredentialStore),
        Arc::new(OcsfNormalizer::new()),
        Arc::new(ClientRegistry::new(vec![])),
        QueryEngineConfig::default(),
        CacheConfig::default(),
    )
    .with_table_registry(registry);

    // `applied_zone_pairs` is Tier-2 Json (no ocsf_field). Lives inside raw_extensions.
    // E-QUERY-038 (PrismError::ColumnNotFound) must fire at plan-time.
    let result = engine
        .execute(
            "SELECT applied_zone_pairs FROM claroty_organization_zone_policies LIMIT 1",
            QueryOptions::default(),
        )
        .await;

    assert!(
        result.is_err(),
        "RG-012a LOAD-BEARING (SAP-3): QueryEngine::execute must return Err when \
         Tier-2 column 'applied_zone_pairs' is queried directly (E-QUERY-038). \
         Got Ok. BC-2.16.020 AC-012; SAP-3.",
    );

    let err = result.unwrap_err();

    match &err {
        PrismError::ColumnNotFound(details) => {
            assert_eq!(
                details.column, "applied_zone_pairs",
                "RG-012a: ColumnNotFound.column must be 'applied_zone_pairs'. \
                 Got: {:?}. BC-2.16.020 AC-012.",
                details.column
            );
            let avail = &details.available_columns;
            assert!(
                avail.contains(&"raw_extensions".to_string()),
                "RG-012a: available_columns must include 'raw_extensions'. Got: {:?}",
                avail
            );
            // Tier-1 Arrow names for zone_policies (4 Tier-1 columns).
            for expected in ["name", "activity_name", "comment", "actor_user_name"] {
                assert!(
                    avail.contains(&expected.to_string()),
                    "RG-012a: available_columns must include '{}' \
                     (Tier-1 Arrow column, claroty_organization_zone_policies). Got: {:?}",
                    expected,
                    avail
                );
            }
            // applied_zone_pairs is Tier-2 → MUST NOT appear in available_columns.
            assert!(
                !avail.contains(&"applied_zone_pairs".to_string()),
                "RG-012a: 'applied_zone_pairs' is Tier-2 and MUST NOT appear in \
                 available_columns (lives inside raw_extensions). Got: {:?}",
                avail
            );
            assert!(
                avail.contains(&"class_uid".to_string()),
                "RG-012a: available_columns must include 'class_uid'. Got: {:?}.",
                avail
            );
            assert!(
                avail.contains(&"_sensor".to_string()),
                "RG-012a: available_columns must include '_sensor'. Got: {:?}.",
                avail
            );
        }
        other => {
            panic!(
                "RG-012a LOAD-BEARING (SAP-3): QueryEngine::execute must return \
                 PrismError::ColumnNotFound (E-QUERY-038) when Tier-2 column \
                 'applied_zone_pairs' is queried directly. Got: {:?}. \
                 BC-2.16.020 AC-012; SAP-3.",
                other
            );
        }
    }
}
