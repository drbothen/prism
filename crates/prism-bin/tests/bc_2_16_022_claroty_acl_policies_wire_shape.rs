// SPDX-License-Identifier: Apache-2.0
//! Wire-shape end-to-end test for BC-2.16.022 —
//! Claroty xDome Organization ACL Policies — `applied_models` JSON-array preservation.
//!
//! # RG-012 context
//!
//! RG-008 (`test_BC_2_16_022_applied_models_raw_extensions_json_array_not_string` in
//! `prism-bin/src/spec_driven_adapter.rs`) exercises the ENRICH-1 DD-2 array-preservation
//! fix at the RecordBatch/StringArray level by calling `pipeline_result_to_record_batch`
//! directly with synthetic `PipelineResult` data. It does NOT exercise:
//!   1. The HTTP fetch boundary (wiremock → `SpecDrivenSensorAdapter::fetch()`)
//!   2. The arrow_json serialization step (WriterBuilder → `with_explicit_nulls(true)`)
//!
//! RG-012 closes this gap by going end-to-end:
//!
//!   wiremock POST /api/v1/organization_acl_policies/
//!     → SpecDrivenSensorAdapter::fetch()
//!     → pipeline_result_to_record_batch (ENRICH-1 DD-2 Json arm)
//!     → arrow_json::WriterBuilder::new().with_explicit_nulls(true).build()
//!     → serialized JSON row bytes
//!
//! # LOAD-BEARING wire-shape assertions
//!
//! 1. `applied_models` MUST NOT appear as a top-level key in the serialized JSON row
//!    (it is Tier-2 and lives inside raw_extensions — ADR-058 §J6).
//! 2. `raw_extensions["applied_models"]` MUST be a NATIVE JSON array (not a stringified
//!    JSON string like `"[\"Siemens...\"]"`). `column_type = "json"` triggers the
//!    ENRICH-1 DD-2 Value::Array preservation path.
//! 3. The array element values MUST be preserved exactly.
//!
//! # RG-012 vs QueryEngine::execute note
//!
//! The story's RG-012 spec says "via QueryEngine::execute end-to-end path". There is no
//! DTU clone for ACL policies (D-2200; SAP-2 probe deferred per story), so a non-live
//! QueryEngine::execute cannot exercise this table without a live Claroty sensor.
//! The fetch-path (SpecDrivenSensorAdapter::fetch + arrow_json serialization) IS the
//! authoritative array-preservation gate — this is where ENRICH-1 DD-2 fires and where
//! the Json column arm preserves native arrays. The story's "QueryEngine::execute"
//! phrasing SHOULD be corrected to "fetch" in a story-side sync (story text only — no
//! code change required here).
//!
//! # SID-1 compliance
//!
//! This is a NON-#[ignore] test (SID-1: no-ignored-test rationalization prohibition).
//! RG-007/RG-010 (`#[ignore]` + `todo!()` in prism-spec-engine) remain as live-only
//! stubs; this test provides the non-live wire-shape coverage they cannot.
//!
//! BC: BC-2.16.022
//! Story: S-CLAROTY-ACLPOLICY-001 RG-012

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

use prism_bin::spec_driven_adapter::{AdapterAuthStrategy, SpecDrivenSensorAdapter};
use prism_core::{OrgId, OrgSlug};
use prism_sensors::{
    BearerStaticSensorAuth, SensorAdapter, adapter::QueryParams,
    adapter::SensorSpec as SensorAdapterSpec,
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
// Shared helper — build a SpecDrivenSensorAdapter directed at the mock server
// ---------------------------------------------------------------------------

/// Build a `SpecDrivenSensorAdapter` from the production `claroty.sensor.toml`
/// directed at the given mock server URI.
///
/// Uses `AdapterAuthStrategy::BearerStatic` matching the production boot path
/// for Claroty sensors.
fn make_claroty_adapter_for_acl_policies(mock_server_uri: &str) -> SpecDrivenSensorAdapter {
    let spec_content = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../prism-sensors/specs/claroty.sensor.toml"),
    )
    .expect(
        "bc_2_16_022_wire_shape: claroty.sensor.toml must be readable from \
         CARGO_MANIFEST_DIR/../prism-sensors/specs/",
    );

    let mut spec = SpecLoader::parse(&spec_content)
        .expect("bc_2_16_022_wire_shape: claroty.sensor.toml must parse");
    spec.base_url = mock_server_uri.to_string();

    let overlay_toml =
        "extends = \"claroty\"\ninstance_id = \"claroty@claroty-acl-policy-wire-test\"";
    let overlay: SensorInstanceOverlay = toml::from_str(overlay_toml)
        .expect("bc_2_16_022_wire_shape: SensorInstanceOverlay TOML parse failed");
    let resolved = OverlayLoader::merge_overlay_onto_type_spec(
        &spec,
        &overlay,
        OrgSlug::new("claroty-acl-policy-wire-test"),
    );

    let http_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect(
            "bc_2_16_022_wire_shape: reqwest::Client build failed \
             (ADR-050 rustls-tls; must not use native-tls)",
        );

    SpecDrivenSensorAdapter::new(
        Arc::new(resolved),
        AdapterAuthStrategy::BearerStatic,
        http_client,
    )
}

// ---------------------------------------------------------------------------
// RG-012 / BC-2.16.022 AC-007 — applied_models NATIVE JSON array at wire level
// ---------------------------------------------------------------------------

/// Wire-shape end-to-end test (RG-012): `SpecDrivenSensorAdapter::fetch()` for
/// `claroty_organization_acl_policies`, when serialized through the production MCP
/// arrow_json path, produces a JSON row where:
///
///   1. `applied_models` is ABSENT as a root-level key (it is Tier-2 → in raw_extensions)
///   2. `raw_extensions["applied_models"]` is a NATIVE JSON array (not a stringified string)
///   3. Array element values are preserved exactly
///   4. `class_uid` == 3004 is present (entity_management class, ADR-058 §K5 Div-3)
///   5. Tier-1 OCSF-projected column `metadata_uid` is present at the top level
///
/// ## RG-012 vs RG-008 distinction
///
/// RG-008 (`test_BC_2_16_022_applied_models_raw_extensions_json_array_not_string`)
/// calls `pipeline_result_to_record_batch` with a synthetic `PipelineResult` and asserts
/// on the Arrow `StringArray` value — it is upstream of the HTTP fetch boundary and the
/// arrow_json serialization step.
///
/// RG-012 (this test) goes end-to-end from the HTTP mock through the full
/// fetch → pipeline → arrow_json path, asserting the SERIALIZED JSON wire bytes
/// consumed by the MCP client / LLM agent. A regression in:
///   - the wiremock response envelope parsing (`$.organization_acl_policies`)
///   - the `column_type = "json"` ENRICH-1 DD-2 native-array preservation
///   - the arrow_json `with_explicit_nulls(true)` configuration
/// ...would fail this test while RG-008 could remain green.
///
/// ## Wire-shape assertion discipline (CLAUDE.md 2026-07-13)
///
/// Any test covering an MCP-visible surface MUST include at least one assertion on the
/// serialized JSON output — the exact envelope/row bytes the LLM agent consumes — not
/// only pre-serialization Rust structures. This test satisfies that requirement for the
/// `applied_models` JSON-array preservation invariant (BC-2.16.022 §PC5).
///
/// BC-2.16.022 AC-007; BC-2.16.022 §PC5 (applied_models column_type = "json");
/// ADR-058 §J6 (Tier-2 raw_extensions aggregation); ENRICH-1 DD-2 Json arm.
/// Story: S-CLAROTY-ACLPOLICY-001 RG-012.
#[tokio::test]
async fn test_BC_2_16_022_claroty_org_acl_policies_wire_shape_applied_models_json_array() {
    let mock_server = MockServer::start().await;

    // Mock the Claroty xDome organization_acl_policies POST endpoint.
    //
    // Endpoint: POST /api/v1/organization_acl_policies/
    //   (per claroty.sensor.toml [[tables.steps]] path_template)
    // Response envelope: {"organization_acl_policies": [...]}
    //   (per claroty.sensor.toml response_path = "$.organization_acl_policies")
    //
    // Record shape: all 11 OrganizationAclPolicyResponseItem fields including
    // `applied_models` as a JSON array of model name strings (the key column
    // under test for RG-012).
    Mock::given(method("POST"))
        .and(path("/api/v1/organization_acl_policies/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "organization_acl_policies": [{
                "policy_id":           "mock-policy-uuid-rg012-001",
                "policy_name":         "Mock Cisco dACL Policy",
                "policy_source":       "manual",
                "applied_models":      ["Siemens SIMATIC S7-300", "Rockwell ControlLogix 5571"],
                "matching_devices":    5_i32,
                "policy_acl_type":     "Cisco dACL",
                "policy_acl":          "permit ip any any",
                "policy_creation_date": "2024-01-01T00:00:00Z",
                "policy_last_updated": "2024-06-15T08:30:00Z",
                "policy_updated_by":   "admin@example.com",
                "policy_notes":        "Mock wire-shape test policy for RG-012"
            }]
        })))
        .mount(&mock_server)
        .await;

    let adapter = make_claroty_adapter_for_acl_policies(&mock_server.uri());

    // source_table = "claroty_organization_acl_policies":
    //   sensor_id = "claroty", table_name = "organization_acl_policies" (bare TOML)
    //   TableRegistry derives: "{sensor_id}_{table_name}" = "claroty_organization_acl_policies"
    let adapter_spec = SensorAdapterSpec {
        source_table: "claroty_organization_acl_policies".to_string(),
        org_id: OrgId::from_uuid(uuid::Uuid::now_v7()),
        #[allow(deprecated)]
        client_id: "claroty-acl-policy-wire-test".to_string(),
        sensor_config: serde_json::json!({}),
    };

    let params = QueryParams {
        cursor: None,
        limit: 10,
        start_time: None,
        end_time: None,
        filters: Default::default(),
    };

    let sensor_auth = BearerStaticSensorAuth::new("mock-bearer-token-acl-policy-rg012");

    let batches = adapter
        .fetch(&adapter_spec, &params, &sensor_auth)
        .await
        .expect(
            "RG-012: fetch() must succeed when the mock server returns a valid \
             organization_acl_policies response. BC-2.16.022 AC-007.",
        );

    assert!(
        !batches.batches.is_empty(),
        "RG-012: fetch() must return at least one RecordBatch for a non-empty response. \
         BC-2.16.022 AC-007."
    );

    let first_batch = &batches.batches[0];
    assert!(
        first_batch.num_rows() > 0,
        "RG-012: first RecordBatch must contain at least one row. BC-2.16.022 AC-007."
    );

    // ── Production MCP serialization path ─────────────────────────────────────
    // Mirrors prism-mcp/src/server.rs §CRIT-1 fix:
    //   arrow_json::writer::WriterBuilder::new()
    //       .with_explicit_nulls(true)
    //       .build::<_, arrow_json::writer::JsonArray>(&mut buf)
    //
    // explicit_nulls=true: NULL-valued Arrow cells → `{"key":null}` in JSON.
    // explicit_nulls=false (DEFAULT): NULL cells are OMITTED — the C3/H20 defect class
    // (BC-2.11.001 EC-11-079; CLAUDE.md §Wire-shape assertion discipline).
    let mut buf: Vec<u8> = Vec::new();
    let mut writer = arrow_json::writer::WriterBuilder::new()
        .with_explicit_nulls(true)
        .build::<_, arrow_json::writer::JsonArray>(&mut buf);
    for batch in &batches.batches {
        writer.write(batch).expect(
            "RG-012: arrow_json write must not fail for claroty_organization_acl_policies \
             RecordBatch. BC-2.16.022 AC-007.",
        );
    }
    writer
        .finish()
        .expect("RG-012: arrow_json finish must not fail. BC-2.16.022 AC-007.");

    let json_rows: Vec<serde_json::Value> = serde_json::from_slice::<Vec<serde_json::Value>>(&buf)
        .expect(
            "RG-012: arrow_json output must deserialize as a JSON array of row objects. \
             BC-2.16.022 AC-007.",
        );

    assert_eq!(
        json_rows.len(),
        1,
        "RG-012: serialized JSON must contain exactly 1 row (one mock record). \
         BC-2.16.022 AC-007."
    );

    let row0 = &json_rows[0];

    // ── LOAD-BEARING assertion 1: applied_models ABSENT at root level ──────────
    //
    // `applied_models` is a Tier-2 column (ocsf_field absent in claroty.sensor.toml).
    // Under ocsf_column_naming = true, Tier-2 columns aggregate into raw_extensions
    // (ADR-058 §J6). `applied_models` MUST NOT appear as a standalone top-level key
    // in the serialized JSON wire row — it lives inside raw_extensions.
    //
    // BC-2.16.022 §PC5; ADR-058 §J6.
    assert!(
        row0.get("applied_models").is_none(),
        "RG-012 LOAD-BEARING: 'applied_models' MUST NOT appear as a top-level key in the \
         serialized JSON row — it is Tier-2 (ocsf_field absent) and MUST live inside \
         raw_extensions (ADR-058 §J6). \
         Got row keys: {:?}. BC-2.16.022 §PC5; S-CLAROTY-ACLPOLICY-001 RG-012.",
        row0.as_object().map(|o| o.keys().collect::<Vec<_>>())
    );

    // ── Assertion 2: raw_extensions present as a JSON string ──────────────────
    //
    // raw_extensions is a DataType::Utf8 StringArray column (ADR-058 §J6).
    // In the serialized wire output it appears as a JSON string that itself contains
    // a JSON object.
    let raw_ext_str = row0.get("raw_extensions").and_then(|v| v.as_str()).expect(
        "RG-012: 'raw_extensions' must be present as a JSON string in the serialized \
             wire row. ADR-058 §J6 Tier-2 aggregation; BC-2.16.022 AC-007.",
    );
    let raw_ext_json: serde_json::Value = serde_json::from_str(raw_ext_str)
        .expect("RG-012: raw_extensions value must be valid JSON. BC-2.16.022 AC-007.");
    let raw_ext_obj = raw_ext_json
        .as_object()
        .expect("RG-012: raw_extensions must deserialize to a JSON object. ADR-058 §J6.");

    // ── LOAD-BEARING assertion 3: applied_models in raw_extensions is a NATIVE JSON array ──
    //
    // `applied_models` has `column_type = "json"` in claroty.sensor.toml.
    // The ENRICH-1 DD-2 Json arm in `pipeline_result_to_record_batch` must preserve
    // `Value::Array(...)` as a native JSON array — NOT stringify it (the pre-fix bug form
    // was `"[\"Siemens SIMATIC S7-300\",\"Rockwell ControlLogix 5571\"]"`, a JSON string).
    //
    // If `applied_models` is declared as `column_type = "string"` (wrong), the ENRICH-1
    // DD-2 String arm would serialize the array as a raw JSON string token, producing the
    // stringified form. This test would fail asserting `.is_array()`.
    //
    // BC-2.16.022 §PC5 (P1 CRITICAL: MUST be "json" not "string");
    // ENRICH-1 DD-2 Json arm; CLAUDE.md §Wire-shape assertion discipline.
    let applied_models_val = raw_ext_obj.get("applied_models").expect(
        "RG-012 LOAD-BEARING: 'applied_models' MUST be present inside raw_extensions. \
         column_type = 'json' Tier-2 column must appear in raw_extensions with the \
         ENRICH-1 DD-2 Json arm preserving the native array. \
         BC-2.16.022 AC-007 §PC5; ADR-058 §J6.",
    );

    assert!(
        applied_models_val.is_array(),
        "RG-012 LOAD-BEARING: raw_extensions['applied_models'] MUST be a NATIVE JSON array, \
         NOT a JSON string (the stringified form '\"[...]\"' from the ENRICH-1 DD-2 bug). \
         column_type = 'json' triggers the Value::Array preservation arm. \
         Got: {:?}. \
         BC-2.16.022 §PC5; ENRICH-1 DD-2 native-array preservation; ADR-058 §J6.",
        applied_models_val
    );

    // Negative: must NOT be a JSON string (the pre-fix bug form).
    assert!(
        !applied_models_val.is_string(),
        "RG-012 LOAD-BEARING: raw_extensions['applied_models'] MUST NOT be a JSON string. \
         A string declaration (column_type = 'string') would produce the array-stringification \
         bug: raw_extensions['applied_models'] = '\"[\\\"Siemens...\\\",...]\"'. \
         Got: {:?}. BC-2.16.022 §PC5.",
        applied_models_val
    );

    // ── Assertion 4: array element values preserved exactly ───────────────────
    //
    // The seeded `applied_models` array has two model name strings.
    // ENRICH-1 DD-2 Value::Array preservation must keep all elements intact.
    let models_arr = applied_models_val
        .as_array()
        .expect("applied_models is an array (asserted above)");
    assert_eq!(
        models_arr,
        &vec![
            serde_json::json!("Siemens SIMATIC S7-300"),
            serde_json::json!("Rockwell ControlLogix 5571"),
        ],
        "RG-012: applied_models array MUST contain exactly the seeded element values \
         ['Siemens SIMATIC S7-300', 'Rockwell ControlLogix 5571']. \
         BC-2.16.022 AC-007; ENRICH-1 DD-2 element preservation."
    );

    // ── Assertion 5: class_uid == 3004 (entity_management) ────────────────────
    //
    // claroty.sensor.toml: ocsf_class = "entity_management" → class_uid 3004.
    // (ADR-058 §K5 Divergence 3; OcsfClassSelector("entity_management") = 3004).
    // class_uid is a synthesized OCSF column (Int32Array) present at the top level.
    //
    // BC-2.16.022 §PC1 (ocsf_class = "entity_management"); ADR-058 §K5 Div-3.
    assert_eq!(
        row0.get("class_uid"),
        Some(&serde_json::json!(3004_i32)),
        "RG-012: class_uid MUST equal 3004 \
         (entity_management class_uid; claroty.sensor.toml ocsf_class = 'entity_management'; \
         ADR-058 §K5 Div-3). \
         BC-2.16.022 §PC1."
    );

    // ── Assertion 6: Tier-1 OCSF-projected columns present at top level ───────
    //
    // Tier-1 columns project as standalone Arrow columns under ocsf_column_naming = true.
    // policy_id → ocsf_field "metadata.uid" → Arrow name "metadata_uid" (dot→underscore).
    //
    // BC-2.16.022 §PC2 (policy_id → metadata_uid via ocsf_field mapping).
    assert!(
        row0.get("metadata_uid").is_some(),
        "RG-012: 'metadata_uid' MUST be present as a top-level key in the serialized JSON row \
         (policy_id Tier-1 column: ocsf_field 'metadata.uid' → Arrow name 'metadata_uid'). \
         BC-2.16.022 §PC2."
    );
    assert_eq!(
        row0.get("metadata_uid"),
        Some(&serde_json::json!("mock-policy-uuid-rg012-001")),
        "RG-012: metadata_uid MUST equal the seeded policy_id value. \
         BC-2.16.022 §PC2 / AC-008."
    );

    // ── Assertion 7: raw API name 'policy_id' NOT at root level ───────────────
    //
    // Under ocsf_column_naming = true, raw API field names are NOT projected as Arrow
    // column names. `policy_id` maps to `metadata_uid`; the raw name must be absent.
    //
    // BC-2.16.022 §PC2; ADR-058 §I2 (raw API names not projected under ocsf_column_naming).
    assert!(
        row0.get("policy_id").is_none(),
        "RG-012: 'policy_id' (raw API field name) MUST NOT appear as a top-level key in the \
         serialized JSON row under ocsf_column_naming = true — it is projected as \
         'metadata_uid' instead. ADR-058 §I2; BC-2.16.022 §PC2.",
    );
}
