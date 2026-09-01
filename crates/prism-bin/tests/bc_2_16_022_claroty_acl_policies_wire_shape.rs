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
//! # RG-012 vs fetch-path note
//!
//! The story's RG-012 spec originally said "via QueryEngine::execute end-to-end path". There is no
//! DTU clone for ACL policies (D-2200; SAP-2 probe deferred per story), so a non-live
//! QueryEngine::execute cannot exercise this table without a live Claroty sensor.
//! The fetch-path (SpecDrivenSensorAdapter::fetch + arrow_json serialization) IS the
//! authoritative array-preservation gate — this is where ENRICH-1 DD-2 fires and where
//! the Json column arm preserves native arrays. The story v1.3/v1.4 corrected the RG-012
//! row to say "fetch"; no outstanding story-side sync remains.
//!
//! # SID-1 compliance
//!
//! This is a NON-#[ignore] test (SID-1: no-ignored-test rationalization prohibition).
//! RG-007/RG-010 in prism-spec-engine are `#[ignore]` live-only tests; they are fully
//! implemented (~160 and ~90 lines of real assertions respectively) and are NOT stubs or
//! `todo!()` placeholders. This file provides the non-live wire-shape coverage they cannot.
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
// SAP-2 compliance marker — no DTU clone for claroty_organization_acl_policies
// ---------------------------------------------------------------------------

/// SAP-2 compliance: no DTU clone exists for `claroty_organization_acl_policies`.
/// The wire-shape tests in this file use wiremock instead of a DTU clone.
/// DTU clone work is deferred to D-2200 (S-CLAROTY-ACLPOLICY-DTU-001).
///
/// SAP-2 rule 4: a field deliberately excluded from TOML MUST have its exclusion
/// documented in the owning BC; here the *entire DTU clone* is absent, not a single field.
#[allow(dead_code)]
const SAP2_STATUS: &str = "N/A: no DTU clone exists for claroty_organization_acl_policies; \
    deferred to D-2200 (S-CLAROTY-ACLPOLICY-DTU-001)";

/// SAP-2 marker: verifies the const is present and carries the required N/A prefix +
/// D-2200 deferral anchor — so the absent DTU clone is never silently overlooked.
#[test]
fn test_BC_2_16_022_claroty_acl_policies_wire_shape_sap2_na_documented() {
    assert!(
        SAP2_STATUS.starts_with("N/A:"),
        "SAP-2 marker MUST start with 'N/A:' to document intentional DTU absence. \
         Got: {:?}",
        SAP2_STATUS
    );
    assert!(
        SAP2_STATUS.contains("D-2200"),
        "SAP-2 marker MUST contain the deferral anchor 'D-2200' \
         (S-CLAROTY-ACLPOLICY-DTU-001). Got: {:?}",
        SAP2_STATUS
    );
}

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

    // ── HIGH-1 fix: assert on the actual emitted POST body, not just the TOML template ──
    //
    // AC-002 story MUST: a test MUST assert that the serialized POST body does NOT contain
    // `offset` or `limit` keys. AC-003/EC-016-022-011/FIX-A: body MUST contain
    // `policy_acl_syntax == "Cisco dACL"` and `filter_by.field == "policy_id"`,
    // `filter_by.operation == "is_not_null"`. AC-010 non-live: exactly one request (single-fetch
    // no pagination loop; PaginationConfig::None).
    //
    // These assertions are LOAD-BEARING: they would fail if pipeline.rs::build_request
    // injected offset/limit on the PaginationConfig::None arm, or if filter_by were absent
    // from the outbound POST body. A template-text assertion (RG-002 / RG-013) cannot
    // catch injection that happens downstream of the template. This is the gate that
    // would have caught the FIX-A defect (HTTP 422 from missing filter_by) before live.
    //
    // BC-2.16.022 §Invariants; EC-016-022-007; AC-002; AC-003; AC-010 (non-live half).
    let reqs = mock_server.received_requests().await.unwrap_or_default();
    assert_eq!(
        reqs.len(),
        1,
        "RG-012 LOAD-BEARING (AC-010 non-live): exactly one POST request MUST be sent — \
         PaginationConfig::None means single-fetch with no pagination loop. \
         Got {} requests. BC-2.16.022 §PC4 / AC-010.",
        reqs.len()
    );
    let sent: serde_json::Value =
        serde_json::from_slice(&reqs[0].body).expect("RG-012: outbound POST body must be JSON");
    assert!(
        sent.get("offset").is_none() && sent.get("limit").is_none(),
        "RG-012 LOAD-BEARING (AC-002): PaginationConfig::None MUST NOT inject 'offset' or \
         'limit' into the POST body — GetOrganizationAclPoliciesRequest declares \
         additionalProperties:false with no offset/limit fields; injecting them causes HTTP 422. \
         BC-2.16.022 §Invariants; EC-016-022-007. Sent body: {}",
        sent
    );
    assert_eq!(
        sent["policy_acl_syntax"], "Cisco dACL",
        "RG-012 LOAD-BEARING (AC-003): POST body MUST contain \
         'policy_acl_syntax' == 'Cisco dACL' — REQUIRED field in \
         GetOrganizationAclPoliciesRequest (OpenAPI required:[\"policy_acl_syntax\"]). \
         BC-2.16.022 §PC1."
    );
    assert_eq!(
        sent["filter_by"]["field"], "policy_id",
        "RG-012 LOAD-BEARING (EC-016-022-011 / FIX-A): POST body filter_by.field MUST be \
         'policy_id' — the enumerate-all selector. Without filter_by the live API returns \
         HTTP 422. BC-2.16.022 §PC1 + EC-016-022-011."
    );
    assert_eq!(
        sent["filter_by"]["operation"], "is_not_null",
        "RG-012 LOAD-BEARING (EC-016-022-011 / FIX-A): POST body filter_by.operation MUST be \
         'is_not_null'. BC-2.16.022 §PC1 + EC-016-022-011."
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

    // ── CR-004: All Tier-1 OCSF-projected columns at top level with correct values ──
    //
    // Tier-1 columns project as standalone Arrow columns (dot→underscore wire names).
    // Each MUST carry the exact seeded value. This closes the gap where assertion 6
    // only checked metadata_uid but not the other 3 Tier-1 columns.
    //
    // BC-2.16.022 §PC2 (4 Tier-1 columns: policy_id→metadata_uid, policy_name→name,
    //   policy_updated_by→actor_user_name, policy_notes→comment).
    assert_eq!(
        row0.get("name"),
        Some(&serde_json::json!("Mock Cisco dACL Policy")),
        "RG-012 CR-004: 'name' MUST equal the seeded policy_name value \
         'Mock Cisco dACL Policy'. BC-2.16.022 §PC2 (policy_name → ocsf_field 'name')."
    );
    assert_eq!(
        row0.get("actor_user_name"),
        Some(&serde_json::json!("admin@example.com")),
        "RG-012 CR-004: 'actor_user_name' MUST equal the seeded policy_updated_by value \
         'admin@example.com'. BC-2.16.022 §PC2 \
         (policy_updated_by → ocsf_field 'actor.user.name' → Arrow 'actor_user_name')."
    );
    assert_eq!(
        row0.get("comment"),
        Some(&serde_json::json!("Mock wire-shape test policy for RG-012")),
        "RG-012 CR-004: 'comment' MUST equal the seeded policy_notes value \
         'Mock wire-shape test policy for RG-012'. BC-2.16.022 §PC2 \
         (policy_notes → ocsf_field 'comment')."
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

    // ── MEDIUM-1 fix: AC-007 item 7 — the 5 remaining Tier-2 field names ABSENT at root ──
    //
    // AC-007 §7: all 7 Tier-2 API field names MUST NOT appear as standalone top-level keys
    // in the serialized JSON row. `applied_models` is asserted absent above (assertion 1).
    // The remaining 5 Tier-2 field names are: policy_source, policy_acl, matching_devices,
    // policy_creation_date, policy_last_updated (ADR-058 §J6: Tier-2 → raw_extensions only).
    //
    // These assertions are LOAD-BEARING on the RG-012 path. BC-2.16.022 AC-007 §7.
    for tier2_raw_name in &[
        "policy_source",
        "policy_acl",
        "matching_devices",
        "policy_creation_date",
        "policy_last_updated",
    ] {
        assert!(
            row0.get(tier2_raw_name).is_none(),
            "RG-012 LOAD-BEARING (AC-007 §7 MEDIUM-1 fix): Tier-2 field '{}' MUST NOT \
             appear as a standalone top-level key in the serialized JSON row — it is \
             aggregated into raw_extensions (ADR-058 §J6). \
             Got row keys: {:?}. BC-2.16.022 AC-007.",
            tier2_raw_name,
            row0.as_object().map(|o| o.keys().collect::<Vec<_>>())
        );
    }
}

// ---------------------------------------------------------------------------
// MED-1 / AC-008 / Task-5 / EC-016-022-005 — applied_models empty-array wire-shape
// ---------------------------------------------------------------------------

/// Wire-shape empty-array test (MED-1 / AC-008 / EC-016-022-005):
/// `SpecDrivenSensorAdapter::fetch()` for `claroty_organization_acl_policies`, when
/// the response record has `applied_models: []` (empty JSON array), the serialized
/// wire output MUST have `raw_extensions["applied_models"]` as a NATIVE EMPTY JSON array.
///
/// ## Invariant under test (EC-016-022-005)
///
/// `applied_models: []` serializes as `[]` — not `null`, not the string `"[]"`.
/// This is the empty-array sub-case of BC-2.16.022 §PC5 (column_type = "json" preservation).
///
/// ## Why this is separate from RG-012
///
/// RG-012 seeds a 2-element array and asserts the non-empty case. Neither RG-012 nor
/// RG-008 exercise the empty array: production Claroty responses for newly created
/// policies with no model assignments return `applied_models: []`, and the ENRICH-1 DD-2
/// Json arm must treat an empty `Value::Array([])` identically to a non-empty one —
/// preserved as `[]`, not coerced to null or stringified.
///
/// ## Wire-shape assertion discipline (CLAUDE.md 2026-07-13)
///
/// Serializes through the production MCP arrow_json path (`with_explicit_nulls(true)`)
/// and asserts on the parsed JSON bytes — the exact envelope consumed by the LLM agent.
///
/// BC-2.16.022 AC-008 / §PC5; ADR-058 §J6; ENRICH-1 DD-2 Json arm.
/// Story: S-CLAROTY-ACLPOLICY-001 MED-1 / Task-5 / EC-016-022-005.
#[tokio::test]
async fn test_BC_2_16_022_applied_models_empty_array_wire_shape() {
    let mock_server = MockServer::start().await;

    // Mock the Claroty xDome organization_acl_policies POST endpoint with a record
    // whose applied_models is an EMPTY JSON array — the EC-016-022-005 empty-array case.
    //
    // Production scenario: a newly created policy with no model assignments returns [].
    Mock::given(method("POST"))
        .and(path("/api/v1/organization_acl_policies/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "organization_acl_policies": [{
                "policy_id":           "mock-policy-uuid-rg012-empty-001",
                "policy_name":         "Mock Empty Models Policy",
                "policy_source":       "auto",
                "applied_models":      [],
                "matching_devices":    0_i32,
                "policy_acl_type":     "Cisco dACL",
                "policy_acl":          "deny ip any any",
                "policy_creation_date": "2024-03-01T00:00:00Z",
                "policy_last_updated": "2024-03-01T00:00:00Z",
                "policy_updated_by":   "system@example.com",
                "policy_notes":        null
            }]
        })))
        .mount(&mock_server)
        .await;

    let adapter = make_claroty_adapter_for_acl_policies(&mock_server.uri());

    let adapter_spec = SensorAdapterSpec {
        source_table: "claroty_organization_acl_policies".to_string(),
        org_id: OrgId::from_uuid(uuid::Uuid::now_v7()),
        #[allow(deprecated)]
        client_id: "claroty-acl-policy-wire-test-empty".to_string(),
        sensor_config: serde_json::json!({}),
    };

    let params = QueryParams {
        cursor: None,
        limit: 10,
        start_time: None,
        end_time: None,
        filters: Default::default(),
    };

    let sensor_auth = BearerStaticSensorAuth::new("mock-bearer-token-acl-policy-empty");

    let batches = adapter
        .fetch(&adapter_spec, &params, &sensor_auth)
        .await
        .expect(
            "MED-1 / EC-016-022-005: fetch() must succeed when applied_models is []. \
             BC-2.16.022 AC-008.",
        );

    assert!(
        !batches.batches.is_empty(),
        "MED-1: fetch() must return at least one RecordBatch. BC-2.16.022 AC-008."
    );

    // ── Production MCP serialization path (mirrors RG-012) ────────────────────
    let mut buf: Vec<u8> = Vec::new();
    let mut writer = arrow_json::writer::WriterBuilder::new()
        .with_explicit_nulls(true)
        .build::<_, arrow_json::writer::JsonArray>(&mut buf);
    for batch in &batches.batches {
        writer.write(batch).expect(
            "MED-1 / EC-016-022-005: arrow_json write must not fail for empty applied_models. \
             BC-2.16.022 AC-008.",
        );
    }
    writer
        .finish()
        .expect("MED-1 / EC-016-022-005: arrow_json finish must not fail. BC-2.16.022 AC-008.");

    let json_rows: Vec<serde_json::Value> = serde_json::from_slice::<Vec<serde_json::Value>>(&buf)
        .expect(
            "MED-1 / EC-016-022-005: arrow_json output must deserialize as JSON array. \
             BC-2.16.022 AC-008.",
        );

    assert_eq!(
        json_rows.len(),
        1,
        "MED-1: serialized JSON must contain exactly 1 row. BC-2.16.022 AC-008."
    );

    let row0 = &json_rows[0];

    // ── Assertion 1: applied_models absent at root level ──────────────────────
    //
    // Tier-2 column; must live inside raw_extensions, not at top level.
    assert!(
        row0.get("applied_models").is_none(),
        "MED-1 / EC-016-022-005: 'applied_models' MUST NOT appear as a top-level key \
         (Tier-2 → raw_extensions). ADR-058 §J6. \
         Got row keys: {:?}.",
        row0.as_object().map(|o| o.keys().collect::<Vec<_>>())
    );

    // ── Assertion 2: raw_extensions is a JSON string ───────────────────────────
    let raw_ext_str = row0.get("raw_extensions").and_then(|v| v.as_str()).expect(
        "MED-1 / EC-016-022-005: 'raw_extensions' must be present as a JSON string. \
         ADR-058 §J6 Tier-2 aggregation; BC-2.16.022 AC-008.",
    );
    let raw_ext_json: serde_json::Value = serde_json::from_str(raw_ext_str)
        .expect("MED-1 / EC-016-022-005: raw_extensions value must be valid JSON.");
    let raw_ext_obj = raw_ext_json
        .as_object()
        .expect("MED-1 / EC-016-022-005: raw_extensions must be a JSON object. ADR-058 §J6.");

    // ── LOAD-BEARING assertion 3: applied_models is a NATIVE EMPTY JSON array ──
    //
    // EC-016-022-005: `applied_models: []` must round-trip as `[]` — a native empty array.
    // MUST NOT be null (null would indicate the column was dropped rather than preserved).
    // MUST NOT be the string `"[]"` (stringified form — the ENRICH-1 DD-2 pre-fix bug).
    // MUST NOT be absent (absent would indicate the Json arm skipped empty arrays).
    let applied_models_val = raw_ext_obj.get("applied_models").expect(
        "MED-1 / EC-016-022-005 LOAD-BEARING: 'applied_models' MUST be present inside \
         raw_extensions even when the array is empty. column_type = 'json' Tier-2 columns \
         with value [] must appear as an empty native JSON array — not be omitted. \
         BC-2.16.022 AC-008 / §PC5; ADR-058 §J6.",
    );

    // Must be a native JSON array (not null, not string).
    assert!(
        applied_models_val.is_array(),
        "MED-1 / EC-016-022-005 LOAD-BEARING: raw_extensions['applied_models'] MUST be a \
         NATIVE JSON array when applied_models is []. \
         Got: {:?}. BC-2.16.022 §PC5; ENRICH-1 DD-2 native-array preservation.",
        applied_models_val
    );

    // Must be EMPTY — this is the empty-array sub-case.
    let models_arr = applied_models_val
        .as_array()
        .expect("applied_models is an array (asserted above)");
    assert!(
        models_arr.is_empty(),
        "MED-1 / EC-016-022-005 LOAD-BEARING: raw_extensions['applied_models'] MUST be an \
         EMPTY array [] when the source value is []. Got {} elements: {:?}. \
         BC-2.16.022 AC-008 / EC-016-022-005.",
        models_arr.len(),
        models_arr
    );

    // Negative: must NOT be a JSON string (the stringified bug form).
    assert!(
        !applied_models_val.is_string(),
        "MED-1 / EC-016-022-005 LOAD-BEARING: raw_extensions['applied_models'] MUST NOT be \
         a JSON string (e.g. '\"[]\"'). The ENRICH-1 DD-2 stringification bug produces this form. \
         Got: {:?}. BC-2.16.022 §PC5.",
        applied_models_val
    );

    // Verify the serialized form is exactly `[]` (not `null`, not `"[]"`).
    let serialized = serde_json::to_string(applied_models_val)
        .expect("MED-1: applied_models_val must serialize");
    assert_eq!(
        serialized, "[]",
        "MED-1 / EC-016-022-005 LOAD-BEARING: raw_extensions['applied_models'] must serialize \
         as exactly '[]'. BC-2.16.022 AC-008 / EC-016-022-005.",
    );

    // ── CR-003 / null-not-absent assertion: 'comment' column is null in wire output ──
    //
    // The MED-1 mock seeds `policy_notes: null` (→ ocsf_field 'comment' → Arrow 'comment').
    // Wire-shape assertion discipline (2026-07-13): NULL cells MUST appear as `null`
    // in JSON, NOT be absent (BC-2.11.001 EC-11-079 row-shape null-not-absent invariant).
    // explicit_nulls=true in the WriterBuilder ensures this.
    //
    // BC-2.16.022 §PC2 (policy_notes → comment Tier-1 mapping);
    // BC-2.11.001 EC-11-079 (null-not-absent); CLAUDE.md §Wire-shape assertion discipline.
    assert_eq!(
        row0.get("comment"),
        Some(&serde_json::Value::Null),
        "MED-1 CR-003 LOAD-BEARING: 'comment' MUST be JSON null (not absent) when \
         policy_notes is null in the source record. explicit_nulls=true ensures null \
         cells appear as null, not as absent keys (BC-2.11.001 EC-11-079 null-not-absent). \
         BC-2.16.022 §PC2 / AC-008."
    );
}
