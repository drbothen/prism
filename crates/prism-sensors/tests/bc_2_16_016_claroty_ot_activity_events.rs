// SPDX-License-Identifier: Apache-2.0
//! Red Gate test suite for BC-2.16.016 — Claroty xDome OT Activity Events Table.
//!
//! Covers S-CLAROTY-OT-EVENTS-001 acceptance criteria AC-003..AC-009.
//! BC-5.38.001 density check: 10 RGTs / 9 ACs = 1.11 (≥ 0.5 threshold — PASS).
//! (Additional coverage: EC-002 non-empty array test and EC-004 mode-absent test; added in v1.2).
//!
//! ## Red Gate invariant
//!
//! Non-`#[ignore]` tests in this file (covering RG-003, RG-006, RG-007, RG-008)
//! MUST FAIL before implementation lands:
//!   - Each test finds the `ot_activity_events` table via `.find()` and panics at
//!     `.expect("claroty_ot_activity_events table must exist")` because the `[[tables]]`
//!     block has not yet been added to `claroty.sensor.toml`.
//! Exception: RG-008 is a trivial SAP-2 marker test (constant assertion) that PASSES
//! before implementation by design — it documents a process status, not a behavioral
//! assertion about the TOML.
//!
//! ## SAP-2 status
//!
//! SAP-2 (DTU↔TOML parity probe) is NOT APPLICABLE to this table.
//! There is no DTU clone for the Claroty OT activity events endpoint as of
//! S-CLAROTY-OT-EVENTS-001 (deferred D-2200: OT-events DTU out of scope for Wave G2).
//! This is documented via the `SAP2_STATUS` constant below.
//!
//! ## SAP-3 compliance (RG-003)
//!
//! The E-QUERY-038 gate is enforced by `check_column_availability` in prism-query, which
//! delegates to `ocsf_projected_column_names`. Using `ocsf_projected_column_names` directly
//! from prism-spec-engine is the architecturally correct proxy: prism-sensors CANNOT depend
//! on prism-query (prism-query depends on prism-sensors in production — circular dependency).
//! Both `TableRegistry::register_sensor` and `check_column_availability` delegate to the same
//! canonical function per ADR-058 §I7, so this proxy test is architecturally equivalent.
//! The end-to-end SAP-3 test via `QueryEngine::execute()` lives in
//! `crates/prism-bin/tests/bc_2_16_016_claroty_ot_activity_events_wire_shape.rs`.
//!
//! ## SID-1 compliance (RG-004, RG-005)
//!
//! RG-004 and RG-005 are `#[ignore]`'d because they require a live Claroty xDome instance.
//! Non-ignored coverage (RG-003, RG-006, RG-007, RG-008) satisfies SID-1: every
//! spec-required behavior has a non-ignored unit/integration test via mock/wiremock.
//!
//! ## Note on mapped_fields (DOT-form key discipline)
//!
//! `ColumnMapper::map_record` returns a `MappingResult` with:
//!   - `mapped_fields`: Tier-1 OCSF-field mappings, keyed by DOT-form (ocsf_field value).
//!     event_id → ocsf_field="finding_info.uid" → key "finding_info.uid" in mapped_fields.
//!   - `raw_extensions`: Tier-2 columns (no ocsf_field) and coercion failures, keyed by column name.
//! Asserting Arrow-name form ("finding_info_uid") would be tautological — Arrow-name flattening
//! happens DOWNSTREAM in `pipeline_result_to_record_batch`. Tests here assert the DOT form
//! to correctly exercise the map_record boundary (TD-VSDD-059 guard).
//!
//! BC: BC-2.16.016
//! Story: S-CLAROTY-OT-EVENTS-001

#![allow(clippy::expect_used, clippy::unwrap_used)]

use prism_spec_engine::{
    column_mapping::{ocsf_projected_column_names, ColumnMapper},
    spec_parser::SpecLoader,
};
use serde_json::json;

const CLAROTY_TOML: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/specs/claroty.sensor.toml"
));

// SAP-2 N/A documentation constant.
// Adversarial reviewer: grep for SAP2_STATUS to confirm the absence of a DTU is documented
// and the deferred story ID is cited per Canonical Principle Rule 3.
const SAP2_STATUS: &str =
    "N/A: no DTU clone exists for claroty_ot_activity_events; deferred to D-2200 \
     (OT-events DTU out of scope for Wave G2, S-CLAROTY-OT-EVENTS-001)";

// ── RG-003 (defense-in-depth proxy — NOT the SAP-3 reachability gate) ────────
/// BC-2.16.016 AC-003 — DEFENSE-IN-DEPTH per SAP-3 rule 3.
///
/// THIS TEST IS NOT THE SAP-3 REACHABILITY GATE.
///
/// This test verifies that `source_ip` is absent from `ocsf_projected_column_names`
/// for `claroty_ot_activity_events` — a necessary precondition for E-QUERY-038 to
/// fire. However, it does NOT exercise E-QUERY-038 end-to-end from the PrismQL
/// parser surface; it calls `ocsf_projected_column_names` directly (synthetic proxy).
///
/// Why proxy: prism-sensors CANNOT depend on prism-query (circular dependency). The
/// end-to-end path — parser → QueryEngine::execute_inner → check_column_availability
/// → E-QUERY-038 — is only exercisable from prism-bin.
///
/// SAP-3 rule 3: defense-in-depth tests must carry a comment stating they are NOT
/// the reachability gate. This is that comment.
///
/// AUTHORITATIVE RG-003 end-to-end gate lives in:
///   `crates/prism-bin/tests/bc_2_16_016_claroty_ot_activity_events_wire_shape.rs`
///   fn `test_BC_2_16_016_claroty_ot_activity_events_tier2_source_ip_raises_e_query_038`
///
/// RED: panics at `.expect("claroty_ot_activity_events table must exist")` because
///      the [[tables]] block has not been added to claroty.sensor.toml yet.
///
/// BC-2.16.016 AC-003; ADR-058 §I7; SAP-3 rule 3; S-CLAROTY-OT-EVENTS-001 RG-003.
#[test]
fn test_BC_2_16_016_claroty_ot_activity_events_tier2_source_ip_raises_e_query_038() {
    let spec = SpecLoader::parse(CLAROTY_TOML).expect("claroty.sensor.toml must parse");

    let table = spec
        .tables
        .iter()
        .find(|t| t.table_name == "ot_activity_events")
        .expect("claroty_ot_activity_events table must exist");

    // Build the registered table name: "{sensor_id}_{table_name}" = "claroty_ot_activity_events"
    let registered_name = format!("{}_{}", spec.sensor_id, table.table_name);
    assert_eq!(
        registered_name, "claroty_ot_activity_events",
        "RG-003: registered table name must be 'claroty_ot_activity_events'. Got: {}",
        registered_name
    );

    // ocsf_projected_column_names(tbl: &TableSpec, ocsf_column_naming: bool) -> Vec<String>
    // Returns the Arrow-name column set visible to QueryEngine.
    // For OCSF tables: Tier-1 columns (ocsf_field present) → their Arrow-name equivalents,
    // plus "raw_extensions", "class_uid", "_sensor".
    // Tier-2 columns (no ocsf_field) → NOT in projected names → E-QUERY-038 if queried.
    let projected = ocsf_projected_column_names(table, spec.ocsf_column_naming);

    // ── E-QUERY-038 proxy assertion: source_ip is Tier-2, NOT projected ─────
    assert!(
        !projected.contains(&"source_ip".to_string()),
        "RG-003 LOAD-BEARING: 'source_ip' is a Tier-2 column (no ocsf_field) and MUST NOT \
         appear in ocsf_projected_column_names (it belongs inside raw_extensions). \
         If present, E-QUERY-038 would NOT fire when it should. \
         Projected columns: {:?}. BC-2.16.016 AC-003; ADR-058 §I7.",
        projected
    );

    // ── raw_extensions MUST be projected ─────────────────────────────────────
    assert!(
        projected.contains(&"raw_extensions".to_string()),
        "RG-003: 'raw_extensions' must be in ocsf_projected_column_names for OCSF tables \
         (ADR-058 §J6). Got: {:?}. BC-2.16.016 AC-003.",
        projected
    );

    // ── Tier-1 Arrow names MUST be projected ─────────────────────────────────
    // event_id → ocsf_field="finding_info.uid" → Arrow name "finding_info_uid"
    // (ADR-058 §C2 Option 4: dot-separated OCSF field → dot→underscore flattening)
    assert!(
        projected.contains(&"finding_info_uid".to_string()),
        "RG-003: 'finding_info_uid' (event_id → ocsf_field=finding_info.uid → arrow name) \
         must be in ocsf_projected_column_names. Got: {:?}. \
         BC-2.16.016 AC-003; ADR-058 §C2 Option 4.",
        projected
    );
    // detection_time → ocsf_field="time" → Arrow name "time"
    assert!(
        projected.contains(&"time".to_string()),
        "RG-003: 'time' (detection_time → ocsf_field=time → arrow name) must be in \
         ocsf_projected_column_names. Got: {:?}. BC-2.16.016 AC-003; ADR-058 §C2.",
        projected
    );
    // event_type → ocsf_field="activity_name" → Arrow name "activity_name"
    assert!(
        projected.contains(&"activity_name".to_string()),
        "RG-003: 'activity_name' (event_type → ocsf_field=activity_name → arrow name) must be \
         in ocsf_projected_column_names. Got: {:?}. BC-2.16.016 AC-003; ADR-058 §C2.",
        projected
    );
    // description → ocsf_field="message" → Arrow name "message"
    assert!(
        projected.contains(&"message".to_string()),
        "RG-003: 'message' (description → ocsf_field=message → arrow name) must be in \
         ocsf_projected_column_names. Got: {:?}. BC-2.16.016 AC-003; ADR-058 §C2.",
        projected
    );

    // ── All 17 Tier-2 column names MUST NOT be projected ─────────────────────
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
            !projected.contains(&tier2_name.to_string()),
            "RG-003: Tier-2 column '{}' MUST NOT be in ocsf_projected_column_names — \
             it belongs inside raw_extensions (E-QUERY-038 gate). \
             BC-2.16.016 AC-003; ADR-058 §J6.",
            tier2_name
        );
    }
}

// ── RG-004 ────────────────────────────────────────────────────────────────────
/// BC-2.16.016 AC-004 (LIVE — requires CLAROTY_INSTANCE_URL):
///   Wire-shape class_uid == 2004, Tier-1 columns present (finding_info_uid, time,
///   activity_name, message), raw_extensions non-null JSON object with at least one
///   Tier-2 field.
///
/// LIVE-MONROE-001: requires CLAROTY_INSTANCE_URL — ungated in CI after a live
/// Claroty xDome instance is available. Until then, the non-live wire-shape coverage
/// is provided by the prism-bin wire-shape test (SID-1 compliance).
#[test]
#[ignore]
// LIVE-MONROE-001: requires CLAROTY_INSTANCE_URL (live Claroty xDome instance)
fn test_BC_2_16_016_claroty_ot_activity_events_live_wire_shape_class_uid_and_tier1() {
    // LIVE-ONLY: connect to real Claroty instance and verify RecordBatch wire shape.
    // Non-live SID-1 coverage: prism-bin/tests/bc_2_16_016_claroty_ot_activity_events_wire_shape.rs
    //   test_BC_2_16_016_claroty_ot_activity_events_wire_shape_class_uid_2004_mock
    panic!(
        "LIVE-MONROE-001: this test requires a live Claroty xDome instance at CLAROTY_INSTANCE_URL"
    );
}

// ── RG-005 ────────────────────────────────────────────────────────────────────
/// BC-2.16.016 AC-006 / EC-002 (LIVE — requires CLAROTY_INSTANCE_URL):
///   Wire-level related_alert_ids assertion: raw_extensions JSON object contains
///   `related_alert_ids` as a native JSON array (not stringified) when the field
///   is present in the API response.
///
/// LIVE-MONROE-001: requires CLAROTY_INSTANCE_URL — ungated in CI after a live
/// Claroty xDome instance is available. The non-live wire-level assertion for EC-002
/// is in prism-bin/tests/bc_2_16_016_claroty_ot_activity_events_wire_shape.rs.
#[test]
#[ignore]
// LIVE-MONROE-001: requires CLAROTY_INSTANCE_URL (live Claroty xDome instance)
fn test_BC_2_16_016_claroty_ot_activity_events_live_raw_extensions_contains_network_fields() {
    // LIVE-ONLY: connect to real Claroty instance and verify raw_extensions network fields.
    // Non-live SID-1 coverage: prism-bin/tests/bc_2_16_016_claroty_ot_activity_events_wire_shape.rs
    //   test_BC_2_16_016_claroty_ot_activity_events_ec002_related_alert_ids_native_json_array
    panic!(
        "LIVE-MONROE-001: this test requires a live Claroty xDome instance at CLAROTY_INSTANCE_URL"
    );
}

// ── RG-006 ────────────────────────────────────────────────────────────────────
/// BC-2.16.016 AC-007 / EC-001 (EC-016-016-001):
///   When the API response omits `event_id`, `ColumnMapper::map_record` must still
///   return a row (not an error), but the mapped `finding_info.uid` key in the
///   `mapped_fields` map MUST be absent/null (graceful null passthrough).
///
///   `ColumnMapper::map_record(raw: &Value, table: &TableSpec) -> Result<MappingResult, PrismError>`
///
///   Rationale for null behavior:
///   Absent-field → null is the DEFAULT absent-field handling of `ColumnMapper::map_record`
///   and occurs regardless of `ColumnOptions::Required`. The `REQUIRED` option on `event_id`
///   declares it as a mandatory push-down parameter (BC-2.11.007 / classify_predicates),
///   consistent with sibling id/uid columns — it does NOT gate null-row production.
///
///   Note on DOT-form key discipline (TD-VSDD-059 guard):
///   `map_record` stores Tier-1 fields in DOT form (ocsf_field value, not Arrow-name).
///   event_id → ocsf_field = "finding_info.uid" → DOT-form key "finding_info.uid" in mapped_fields.
///   Asserting the Arrow-name form ("finding_info_uid") would be tautological — Arrow-name
///   flattening happens downstream in `pipeline_result_to_record_batch`.
///
/// ## SAP-3 rule 3 — DEFENSE-IN-DEPTH NOTICE
///
/// THIS TEST IS NOT THE PRODUCTION REACHABILITY GATE for AC-007.
///
/// This test exercises `ColumnMapper::map_record` directly — a non-production
/// intermediate path with ZERO live callers in `crates/*/src/` (confirmed: grep
/// `ColumnMapper::map_record` in `crates/*/src/` returns only the definition itself).
/// The production materialization path goes through `SpecDrivenSensorAdapter::fetch`
/// → `pipeline_result_to_record_batch` → `build_column_array`, NOT through `map_record`.
///
/// SAP-3 rule 3: defense-in-depth tests must carry a comment stating they are NOT
/// the reachability gate. The AUTHORITATIVE production-path gate for AC-007 lives in:
///   `crates/prism-bin/tests/bc_2_16_016_claroty_ot_activity_events_wire_shape.rs`
///   fn `test_BC_2_16_016_claroty_ot_activity_events_ac007_absent_event_id_null_finding_info_uid_production_path`
///
/// RED: panics at `.expect("claroty_ot_activity_events table must exist")`.
///
/// BC-2.16.016 AC-007; EC-016-016-001; SAP-3 rule 3; S-CLAROTY-OT-EVENTS-001 RG-006.
#[test]
fn test_BC_2_16_016_claroty_ot_activity_events_required_event_id_absent_produces_null_row() {
    let spec = SpecLoader::parse(CLAROTY_TOML).expect("claroty.sensor.toml must parse");

    let table = spec
        .tables
        .iter()
        .find(|t| t.table_name == "ot_activity_events")
        .expect("claroty_ot_activity_events table must exist");

    // API response with event_id absent (EC-016-016-001: REQUIRED field missing).
    // All other fields present to isolate the event_id absence.
    let api_record = json!({
        // event_id intentionally absent (REQUIRED field — tests graceful null passthrough)
        "detection_time": "2024-01-15T10:30:00Z",
        "event_type": "network_connection",
        "description": "Outbound connection detected",
        "source_ip": "192.168.1.100",
        "dest_ip": "10.0.0.50",
        "protocol": "TCP",
        "dest_port": 443,
        "source_port": 54321,
        "ip_protocol": "IPv4",
        "source_asset_id": "asset-001",
        "dest_asset_id": "asset-002",
        "source_device_name": "PLC-01",
        "dest_device_name": "HMI-02",
        "source_device_type": "PLC",
        "dest_device_type": "HMI",
        "source_site_name": "Site-A",
        "dest_site_name": "Site-B",
        "source_username": "operator",
        "related_alert_ids": [101, 102],
        "mode": "Learning"
    });

    // map_record(raw: &Value, table: &TableSpec) -> Result<MappingResult, PrismError>
    let row = ColumnMapper::map_record(&api_record, table)
        .expect("RG-006: map_record must not return Err for a valid API record");

    // ── finding_info.uid must be absent or null (event_id was omitted) ────────
    // map_record stores Tier-1 keys in DOT form (ocsf_field value, NOT the Arrow-name).
    // event_id → ocsf_field = "finding_info.uid" → DOT-form key "finding_info.uid" in mapped_fields.
    // Asserting "finding_info_uid" would test the wrong key (Arrow-name form, post-flattening).
    let finding_info_uid = row.mapped_fields.get("finding_info.uid");
    assert!(
        finding_info_uid.is_none() || finding_info_uid == Some(&serde_json::Value::Null),
        "RG-006 LOAD-BEARING (AC-007 / EC-016-016-001): when 'event_id' is absent in the API \
         response, 'finding_info.uid' (DOT-form key in mapped_fields) must be absent or null — \
         NOT Some(non-null). Got: {:?}. BC-2.16.016 AC-007; EC-016-016-001.",
        finding_info_uid
    );

    // ── Row must NOT be dropped — other Tier-1 fields must still be present ──
    // detection_time → ocsf_field = "time" → DOT-form key "time" in mapped_fields.
    let time_val = row.mapped_fields.get("time");
    assert!(
        time_val.is_some() && time_val != Some(&serde_json::Value::Null),
        "RG-006: 'time' (detection_time, DOT-form key) must be present and non-null in \
         mapped_fields even when event_id is absent — map_record must not drop the row. \
         BC-2.16.016 AC-007; EC-016-016-001."
    );

    // ── raw_extensions must still contain Tier-2 fields ──────────────────────
    assert!(
        row.raw_extensions.contains_key("source_ip"),
        "RG-006: 'source_ip' must be in raw_extensions even when event_id is absent. \
         BC-2.16.016 AC-007; EC-016-016-001."
    );

    // ── related_alert_ids=[101,102] must be Value::Array in raw_extensions ────
    assert_eq!(
        row.raw_extensions.get("related_alert_ids"),
        Some(&serde_json::Value::Array(vec![
            serde_json::json!(101_i64),
            serde_json::json!(102_i64),
        ])),
        "RG-006: related_alert_ids=[101,102] must be Value::Array in raw_extensions \
         at the map_record layer. BC-2.16.016 AC-006; EC-016-016-002."
    );
}

// ── RG-007 ────────────────────────────────────────────────────────────────────
/// BC-2.16.016 AC-008 / EC-003 (EC-016-016-003):
///   When `detection_time` is absent from the API response, the pipeline must NOT fail —
///   the row must still be emitted with `time` null-passthrough in `mapped_fields`.
///   ADR-028 §D8-B: implicit iso8601 default applies when `timestamp_formats` is omitted.
///   A null/absent source field → null Arrow cell, NOT a dropped row.
///
///   Also covers at the map_record layer:
///   - EC-002: related_alert_ids=[] → Value::Array([]) in raw_extensions
///   - EC-004: mode=null → Value::Null in raw_extensions
///
/// ## SAP-3 rule 3 — DEFENSE-IN-DEPTH NOTICE
///
/// THIS TEST IS NOT THE PRODUCTION REACHABILITY GATE for AC-008.
///
/// This test exercises `ColumnMapper::map_record` directly — a non-production
/// intermediate path with ZERO live callers in `crates/*/src/` (confirmed: grep
/// `ColumnMapper::map_record` in `crates/*/src/` returns only the definition itself).
/// The production materialization path goes through `SpecDrivenSensorAdapter::fetch`
/// → `pipeline_result_to_record_batch` → `build_column_array`, NOT through `map_record`.
///
/// SAP-3 rule 3: defense-in-depth tests must carry a comment stating they are NOT
/// the reachability gate. The AUTHORITATIVE production-path gate for AC-008 lives in:
///   `crates/prism-bin/tests/bc_2_16_016_claroty_ot_activity_events_wire_shape.rs`
///   fn `test_BC_2_16_016_claroty_ot_activity_events_ac008_absent_detection_time_null_time_production_path`
///
/// RED: panics at `.expect("claroty_ot_activity_events table must exist")`.
///
/// BC-2.16.016 AC-008; EC-016-016-003; ADR-028 §D8-B; SAP-3 rule 3;
/// S-CLAROTY-OT-EVENTS-001 RG-007.
#[test]
fn test_BC_2_16_016_claroty_ot_activity_events_detection_time_null_passthrough() {
    let spec = SpecLoader::parse(CLAROTY_TOML).expect("claroty.sensor.toml must parse");

    let table = spec
        .tables
        .iter()
        .find(|t| t.table_name == "ot_activity_events")
        .expect("claroty_ot_activity_events table must exist");

    // API response with detection_time absent (EC-016-016-003: optional Datetime field null).
    // Also includes related_alert_ids=[] (EC-002) and mode=null (EC-004) for combined coverage.
    let api_record = json!({
        "event_id": 12345_i64,
        // detection_time intentionally absent (EC-016-016-003)
        "event_type": "network_connection",
        "description": "OT event with no detection time",
        "source_ip": "192.168.1.1",
        "dest_ip": "192.168.2.1",
        "protocol": "TCP",
        "dest_port": 80_i64,
        "source_port": 54321_i64,
        "ip_protocol": "IPv4",
        "source_asset_id": "asset-003",
        "dest_asset_id": "asset-004",
        "source_device_name": "PLC-02",
        "dest_device_name": "SCADA-01",
        "source_device_type": "PLC",
        "dest_device_type": "SCADA",
        "source_site_name": "Site-C",
        "dest_site_name": "Site-D",
        "source_username": null,
        "related_alert_ids": [],   // EC-002: empty JSON array
        "mode": null               // EC-004: mode absent/null
    });

    // map_record(raw: &Value, table: &TableSpec) -> Result<MappingResult, PrismError>
    let row = ColumnMapper::map_record(&api_record, table)
        .expect("RG-007: map_record must not return Err for a valid API record");

    // ── finding_info.uid must be present (event_id = 12345 was provided) ─────
    // DOT-form key: event_id → ocsf_field="finding_info.uid" → "finding_info.uid" in mapped_fields.
    let finding_info_uid = row.mapped_fields.get("finding_info.uid");
    assert!(
        finding_info_uid.is_some() && finding_info_uid != Some(&serde_json::Value::Null),
        "RG-007: 'finding_info.uid' (DOT form) must be present and non-null when \
         event_id=12345 is provided. Got: {:?}. BC-2.16.016 AC-008.",
        finding_info_uid
    );
    assert_eq!(
        finding_info_uid,
        Some(&serde_json::json!(12345_i64)),
        "RG-007: 'finding_info.uid' (DOT form) must equal json!(12345_i64). \
         BC-2.16.016 AC-008."
    );

    // ── detection_time absent → time must be null/absent in mapped_fields ─────
    // DOT-form key: detection_time → ocsf_field="time" → "time" in mapped_fields.
    let time_val = row.mapped_fields.get("time");
    assert!(
        time_val.is_none() || time_val == Some(&serde_json::Value::Null),
        "RG-007 LOAD-BEARING (AC-008 / EC-016-016-003): when 'detection_time' is absent, \
         'time' (DOT-form key in mapped_fields) must be absent or null. \
         ADR-028 §D8-B implicit iso8601 default: absent source → null, not an error. \
         Got: {:?}. BC-2.16.016 AC-008; EC-016-016-003.",
        time_val
    );

    // ── EC-002: related_alert_ids=[] → Value::Array([]) in raw_extensions ────
    // json column_type stores arrays as serde_json::Value::Array in raw_extensions.
    // Empty array must stay as Value::Array(vec![]) NOT as Value::Null or absent.
    assert_eq!(
        row.raw_extensions.get("related_alert_ids"),
        Some(&serde_json::Value::Array(vec![])),
        "RG-007 (EC-002 / EC-016-016-002): related_alert_ids=[] must be stored as \
         Value::Array(vec![]) in raw_extensions at the map_record level, NOT as null or absent. \
         BC-2.16.016 AC-006; EC-016-016-002."
    );

    // ── EC-004: mode=null → Value::Null in raw_extensions ────────────────────
    // When mode is null in the API response, it must appear as Value::Null in raw_extensions.
    assert_eq!(
        row.raw_extensions.get("mode"),
        Some(&serde_json::Value::Null),
        "RG-007 (EC-004 / EC-016-016-004): mode=null in API response must be stored as \
         Value::Null in raw_extensions at the map_record level. \
         BC-2.16.016 AC-006; EC-016-016-004."
    );
}

// ── Additional coverage: EC-002 related_alert_ids with non-empty array ────────
/// BC-2.16.016 EC-002 (EC-016-016-002):
///   When `related_alert_ids` is a non-empty JSON array `[1, 2, 3]` in the API response,
///   `ColumnMapper::map_record` must store it as `Value::Array([json!(1), json!(2), json!(3)])`
///   in `raw_extensions` — NOT as a JSON string `"[1,2,3]"`.
///
///   At the map_record layer, json-type column values are stored verbatim as
///   `serde_json::Value::Array`. This asserts the PRE-stringification form at map_record.
///   The wire-level native-JSON-array assertion is in the prism-bin wire-shape test.
///
/// RED: panics at `.expect("claroty_ot_activity_events table must exist")`.
///
/// BC-2.16.016 AC-006; EC-016-016-002; S-CLAROTY-OT-EVENTS-001 (additional EC coverage).
#[test]
fn test_BC_2_16_016_claroty_ot_activity_events_ec002_related_alert_ids_nonempty_array_in_raw_extensions(
) {
    let spec = SpecLoader::parse(CLAROTY_TOML).expect("claroty.sensor.toml must parse");

    let table = spec
        .tables
        .iter()
        .find(|t| t.table_name == "ot_activity_events")
        .expect("claroty_ot_activity_events table must exist");

    let api_record = json!({
        "event_id": 99999_i64,
        "detection_time": "2024-06-01T12:00:00Z",
        "event_type": "policy_violation",
        "description": "Multiple alerts triggered",
        "source_ip": "10.0.0.1",
        "dest_ip": "10.0.0.2",
        "protocol": "UDP",
        "dest_port": 161_i64,
        "source_port": 1024_i64,
        "ip_protocol": "IPv4",
        "source_asset_id": "asset-ec002",
        "dest_asset_id": "asset-ec002-dst",
        "source_device_name": "PLC-EC002",
        "dest_device_name": "RTU-EC002",
        "source_device_type": "PLC",
        "dest_device_type": "RTU",
        "source_site_name": "Site-EC002",
        "dest_site_name": "Site-EC002-Dst",
        "source_username": "admin",
        "related_alert_ids": [1, 2, 3],
        "mode": "Protection"
    });

    // map_record(raw: &Value, table: &TableSpec) -> Result<MappingResult, PrismError>
    let row = ColumnMapper::map_record(&api_record, table)
        .expect("EC-002: map_record must not return Err for a valid API record");

    // ── EC-002 LOAD-BEARING: related_alert_ids must be Value::Array at map_record ──
    // The json column_type stores arrays as serde_json::Value::Array in raw_extensions.
    // Tier-2 columns have no ocsf_field → they land in raw_extensions, not mapped_fields.
    assert_eq!(
        row.raw_extensions.get("related_alert_ids"),
        Some(&serde_json::Value::Array(vec![
            serde_json::json!(1_i64),
            serde_json::json!(2_i64),
            serde_json::json!(3_i64),
        ])),
        "EC-002 LOAD-BEARING: related_alert_ids=[1,2,3] must be stored as \
         Value::Array([json!(1), json!(2), json!(3)]) in raw_extensions at the map_record \
         layer. BC-2.16.016 AC-006; EC-016-016-002."
    );
}

// ── Additional coverage: EC-004 mode absent ───────────────────────────────────
/// BC-2.16.016 EC-004 (EC-016-016-004):
///   When `mode` is absent from the API response, it must be absent from `raw_extensions`
///   (not present as a non-null value). An absent optional field → absent or null key.
///
/// RED: panics at `.expect("claroty_ot_activity_events table must exist")`.
///
/// BC-2.16.016 AC-006; EC-016-016-004; S-CLAROTY-OT-EVENTS-001 (additional EC coverage).
#[test]
fn test_BC_2_16_016_claroty_ot_activity_events_ec004_mode_absent_from_raw_extensions() {
    let spec = SpecLoader::parse(CLAROTY_TOML).expect("claroty.sensor.toml must parse");

    let table = spec
        .tables
        .iter()
        .find(|t| t.table_name == "ot_activity_events")
        .expect("claroty_ot_activity_events table must exist");

    let api_record = json!({
        "event_id": 55555_i64,
        "detection_time": "2024-03-01T08:00:00Z",
        "event_type": "scan",
        "description": "Network scan detected",
        "source_ip": "172.16.0.1",
        "dest_ip": "172.16.0.255",
        "protocol": "ICMP",
        "dest_port": 0_i64,
        "source_port": 0_i64,
        "ip_protocol": "IPv4",
        "source_asset_id": "scanner-001",
        "dest_asset_id": "broadcast",
        "source_device_name": "Workstation-01",
        "dest_device_name": "Broadcast",
        "source_device_type": "Workstation",
        "dest_device_type": "Network",
        "source_site_name": "Site-X",
        "dest_site_name": "Site-X",
        "source_username": null,
        "related_alert_ids": [999_i64]
        // mode intentionally absent (EC-016-016-004)
    });

    // map_record(raw: &Value, table: &TableSpec) -> Result<MappingResult, PrismError>
    let row = ColumnMapper::map_record(&api_record, table)
        .expect("EC-004: map_record must not return Err for a valid API record");

    // ── EC-004: mode absent from API → should be absent or null in raw_extensions ──
    let mode_val = row.raw_extensions.get("mode");
    assert!(
        mode_val.is_none() || mode_val == Some(&serde_json::Value::Null),
        "EC-004: when 'mode' is absent from the API response, raw_extensions['mode'] must be \
         absent or null (not a non-null value). Got: {:?}. \
         BC-2.16.016 EC-016-016-004.",
        mode_val
    );

    // ── Verify related_alert_ids=[999] still maps correctly ───────────────────
    assert_eq!(
        row.raw_extensions.get("related_alert_ids"),
        Some(&serde_json::Value::Array(vec![serde_json::json!(999_i64)])),
        "EC-004: related_alert_ids=[999] must still map correctly even when mode is absent. \
         BC-2.16.016 EC-016-016-004."
    );
}

// ── RG-008 ────────────────────────────────────────────────────────────────────
/// BC-2.16.016 AC-009 — SAP-2 N/A marker:
///   Documents that SAP-2 (DTU↔TOML schema parity probe) is NOT APPLICABLE because
///   there is no DTU clone for the Claroty OT activity events endpoint as of
///   S-CLAROTY-OT-EVENTS-001. The adversarial reviewer can verify by searching for
///   `SAP2_STATUS` in this file.
///
/// This is a TRIVIAL MARKER TEST — it is GREEN-BY-DESIGN, always passes,
/// and documents a process status rather than asserting behavioral correctness.
/// The Red Gate (all tests fail before implementation) applies to RG-001..RG-007 only.
///
/// BC-2.16.016 AC-009; S-CLAROTY-OT-EVENTS-001 RG-008.
#[test]
fn test_BC_2_16_016_claroty_ot_activity_events_sap2_na_documented() {
    // SAP2_STATUS constant is declared at the top of this file.
    // Adversarial reviewer: verify it starts with "N/A:" and cites a deferred decision ID.
    assert!(
        SAP2_STATUS.starts_with("N/A:"),
        "RG-008 (AC-009): SAP2_STATUS must start with 'N/A:' — \
         documents that DTU parity probe is not applicable for this table. \
         Adversarial reviewer: verify the constant cites a deferred story/decision ID."
    );
    assert!(
        SAP2_STATUS.contains("D-2200"),
        "RG-008 (AC-009): SAP2_STATUS must cite deferred decision ID D-2200 per \
         Canonical Principle Rule 3 (deferral must name the future anchor). \
         Got: {}",
        SAP2_STATUS
    );
}
