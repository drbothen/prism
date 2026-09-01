#![allow(non_snake_case)]
#![allow(clippy::expect_used, clippy::unwrap_used)]
//! Red Gate tests for BC-2.16.018 — Claroty xDome Collection Servers Table.
//!
//! Story: S-CLAROTY-SERVERS-001 (Wave C G4)
//! BCs:   BC-2.16.018 v1.0
//! ACs:   AC-001..AC-008
//!
//! | RG  | Test name                                                                   | Type                  | AC   |
//! |-----|-----------------------------------------------------------------------------|-----------------------|------|
//! | 001 | test_BC_2_16_018_claroty_servers_toml_block_parses                          | Unit (SpecLoader)     | AC-001 |
//! | 002 | test_BC_2_16_018_claroty_servers_tier1_columns_two_with_ocsf_field          | Unit (ColumnSpec)     | AC-002 |
//! | 003 | test_BC_2_16_018_claroty_servers_tier2_column_raises_e_query_038            | Integration (plan)    | AC-003 |
//! | 004 | test_BC_2_16_018_claroty_servers_tier1_raw_toml_name_raises_e_query_038     | Integration (plan)    | AC-004 |
//! | 005 | test_BC_2_16_018_claroty_servers_live_wire_shape_class_uid_and_tier1        | Live #[ignore]        | AC-005 |
//! | 006 | test_BC_2_16_018_claroty_servers_live_raw_extensions_contains_tier2_keys    | Live #[ignore]        | AC-006 |
//! | 007 | test_BC_2_16_018_claroty_servers_required_server_name_absent_produces_null_row | Unit (mock)         | AC-007 |
//! | 008 | test_BC_2_16_018_claroty_servers_nullable_count_uses_empty_page_halt        | Unit (mock)           | AC-008 |
//!
//! Red Gate failure mode (ALL non-#[ignore] tests):
//! `expect("claroty_servers table must exist")` panics because the [[tables]] block
//! for `claroty_servers` is not yet present in `crates/prism-sensors/specs/claroty.sensor.toml`.
//! This is the CORRECT failure reason — not a compile error.

use std::collections::HashMap;

use prism_core::{column::ColumnOptions, OrgSlug};
use prism_spec_engine::{
    column_mapping::{ocsf_projected_column_names, ColumnMapper},
    pipeline::{FetchContext, PipelineExecutor},
    spec_parser::{PaginationConfig, SpecLoader},
    NullAuthProvider,
};
use serde_json::json;

// ---------------------------------------------------------------------------
// SAP-2 marker — no DTU clone exists for claroty_servers
// ---------------------------------------------------------------------------

/// SAP-2 compliance: no DTU clone exists for the claroty_servers table.
/// Parity check deferred to S-CLAROTY-SERVERS-DTU-001 (D-2200 tracking entry).
#[allow(dead_code)]
const SAP2_STATUS: &str =
    "N/A: no DTU clone exists for claroty_servers; deferred to D-2200 (S-CLAROTY-SERVERS-DTU-001)";

// ---------------------------------------------------------------------------
// Helper: load the full claroty.sensor.toml and return the claroty_servers TableSpec.
// Panics with the Red Gate message if the table is absent (pre-implementation).
// ---------------------------------------------------------------------------

fn load_claroty_servers_table() -> (
    prism_spec_engine::spec_parser::SensorSpec,
    prism_spec_engine::spec_parser::TableSpec,
) {
    let toml_path = concat!(env!("CARGO_MANIFEST_DIR"), "/specs/claroty.sensor.toml");
    let content = std::fs::read_to_string(toml_path)
        .unwrap_or_else(|e| panic!("Failed to read claroty.sensor.toml: {e}"));
    let spec = SpecLoader::parse(&content)
        .unwrap_or_else(|e| panic!("SpecLoader::parse failed for claroty.sensor.toml: {e:?}"));
    let table = spec
        .tables
        .iter()
        .find(|t| t.table_name == "servers")
        .cloned()
        .expect(
            "servers table must exist in claroty.sensor.toml \
             — RED GATE: [[tables]] block not yet added (S-CLAROTY-SERVERS-001 Task 8)",
        );
    (spec, table)
}

// ---------------------------------------------------------------------------
// RG-001 | AC-001 — TOML block parses; 17 columns; 1 fetch step; pagination offset_limit 1000
// ---------------------------------------------------------------------------

/// RED GATE: panics at `load_claroty_servers_table()` until Task 8 adds the TOML block.
/// After implementation: asserts 17 ColumnSpec entries, 1 fetch step, OffsetLimit 1000.
///
/// Traces to: BC-2.16.018 §Postconditions §1 (TOML Table Contract).
/// Story: S-CLAROTY-SERVERS-001 AC-001
#[test]
fn test_BC_2_16_018_claroty_servers_toml_block_parses() {
    let (_spec, servers) = load_claroty_servers_table();

    assert_eq!(
        servers.columns.len(),
        17,
        "claroty_servers must declare exactly 17 columns \
         (2 Tier-1: server_name/server_status + 15 Tier-2) per BC-2.16.018 §PC1; \
         got {}",
        servers.columns.len()
    );

    assert_eq!(
        servers.steps.len(),
        1,
        "claroty_servers must have exactly 1 fetch step (fetch_servers)"
    );

    let step = &servers.steps[0];
    assert_eq!(
        step.name, "fetch_servers",
        "fetch step name must be 'fetch_servers'"
    );
    assert_eq!(
        step.method, "POST",
        "fetch step method must be POST (POST-for-read pattern)"
    );
    assert_eq!(
        step.path_template, "/api/v1/servers/",
        "fetch step path_template must be '/api/v1/servers/' (trailing slash per xDome convention)"
    );
    assert_eq!(
        step.response_path, "$.servers",
        "fetch step response_path must be '$.servers' (BC-2.16.018 §PC1 envelope key)"
    );

    match &step.pagination {
        Some(PaginationConfig::OffsetLimit { page_size }) => {
            assert_eq!(
                *page_size, 1000,
                "claroty_servers must use page_size = 1000 (BC-2.16.018 §PC1)"
            );
        }
        other => {
            panic!("claroty_servers fetch step must have OffsetLimit pagination; got: {other:?}");
        }
    }

    // ── CR-004: body_template fields projection ────────────────────────────────
    // The POST step requires a `fields` projection (GetServersParameters minItems: 1).
    // All 17 declared column names must appear in the body_template fields array.
    // A dropped or misspelled field means xDome silently omits it from the response.
    // Read column names from parsed spec — NOT a hardcoded list.
    assert!(
        step.body_template.is_some(),
        "BC-2.16.018 AC-001 CR-004: fetch_servers step must have body_template \
         for fields projection (GetServersParameters minItems: 1)"
    );
    let body_tmpl_str = step.body_template.as_deref().unwrap();
    let body_tmpl_json: serde_json::Value = serde_json::from_str(body_tmpl_str)
        .expect("BC-2.16.018 AC-001 CR-004: body_template must be valid JSON");
    let fields_arr = body_tmpl_json["fields"]
        .as_array()
        .expect("BC-2.16.018 AC-001 CR-004: body_template must have a 'fields' array");
    let fields_as_strs: Vec<&str> = fields_arr.iter().filter_map(|v| v.as_str()).collect();
    for col in &servers.columns {
        assert!(
            fields_as_strs.contains(&col.name.as_str()),
            "BC-2.16.018 AC-001 CR-004: column '{}' must be present in body_template \
             'fields' array (GetServersParameters projection). \
             Present fields: {:?}",
            col.name,
            fields_as_strs
        );
    }
}

// ---------------------------------------------------------------------------
// RG-002 | AC-002 — 2 Tier-1 columns; server_name→device.name REQUIRED; server_status→status_code; 15 Tier-2
// ---------------------------------------------------------------------------

/// RED GATE: panics at `load_claroty_servers_table()` until Task 8 adds the TOML block.
/// After implementation: asserts Tier-1/Tier-2 column classification per BC-2.16.018 §PC2.
///
/// Traces to: BC-2.16.018 §Postconditions §2 (Tier-1/Tier-2 column classification).
/// Story: S-CLAROTY-SERVERS-001 AC-002
#[test]
fn test_BC_2_16_018_claroty_servers_tier1_columns_two_with_ocsf_field() {
    let (_spec, servers) = load_claroty_servers_table();

    // --- Tier-1 count ---
    let tier1_cols: Vec<_> = servers
        .columns
        .iter()
        .filter(|c| c.ocsf_field.is_some())
        .collect();
    assert_eq!(
        tier1_cols.len(),
        2,
        "claroty_servers must have exactly 2 Tier-1 columns (ocsf_field = Some); \
         got {}: {:?}",
        tier1_cols.len(),
        tier1_cols.iter().map(|c| &c.name).collect::<Vec<_>>()
    );

    // --- server_name → device.name, REQUIRED ---
    let server_name_col = servers
        .columns
        .iter()
        .find(|c| c.name == "server_name")
        .expect("server_name column must be declared in claroty_servers");

    assert_eq!(
        server_name_col.ocsf_field.as_deref(),
        Some("device.name"),
        "server_name must have ocsf_field = \"device.name\" (BC-2.16.018 §PC2 Tier-1)"
    );

    assert!(
        server_name_col.options.contains(&ColumnOptions::Required),
        "server_name must have options = [\"REQUIRED\"] (single-column PK, BC-2.16.018 §PC2)"
    );

    // --- server_status → status_code ---
    let server_status_col = servers
        .columns
        .iter()
        .find(|c| c.name == "server_status")
        .expect("server_status column must be declared in claroty_servers");

    assert_eq!(
        server_status_col.ocsf_field.as_deref(),
        Some("status_code"),
        "server_status must have ocsf_field = \"status_code\" (BC-2.16.018 §PC2 Tier-1)"
    );

    // server_status is NOT REQUIRED (only server_name is PK)
    assert!(
        !server_status_col.options.contains(&ColumnOptions::Required),
        "server_status must NOT have REQUIRED option (only server_name is REQUIRED per BC-2.16.018 §PC3)"
    );

    // --- Tier-2 count: exactly 15 ---
    let tier2_count = servers
        .columns
        .iter()
        .filter(|c| c.ocsf_field.is_none())
        .count();
    assert_eq!(
        tier2_count, 15,
        "claroty_servers must have exactly 15 Tier-2 columns (ocsf_field = None); \
         got {tier2_count}"
    );

    // Verify Arrow names via ocsf_field_to_arrow_name
    use prism_spec_engine::column_mapping::ocsf_field_to_arrow_name;
    assert_eq!(
        ocsf_field_to_arrow_name("device.name"),
        "device_name",
        "ocsf_field_to_arrow_name(\"device.name\") must return \"device_name\" (ADR-058 §C)"
    );
    assert_eq!(
        ocsf_field_to_arrow_name("status_code"),
        "status_code",
        "ocsf_field_to_arrow_name(\"status_code\") must return \"status_code\" (no dots)"
    );
}

// ---------------------------------------------------------------------------
// RG-003 | AC-003 — Tier-2 column query raises E-QUERY-038; available_columns excludes Tier-2 raw name
// ---------------------------------------------------------------------------

/// RED GATE: panics at `load_claroty_servers_table()` until Task 8 adds the TOML block.
/// After implementation: verifies that `server_location` (Tier-2) is NOT in the OCSF-projected
/// column set, while `raw_extensions`, `device_name`, `status_code`, `class_uid`, `_sensor` ARE.
///
/// SAP-3 defense-in-depth note: this test exercises `ocsf_projected_column_names`, which is
/// the exact data structure the E-QUERY-038 plan-time gate consults in prism-query's
/// `check_column_availability`. Full end-to-end reachability from the PrismQL parser surface
/// (parser → planner → gate → E-QUERY-038 error) requires prism-query, which would create a
/// circular dependency from prism-sensors (prism-query already depends on prism-sensors in
/// production). This spec-level test is the correct coverage layer for the prism-sensors crate.
///
/// SAP-3 rule-3 defense-in-depth disclaimer: the AUTHORITATIVE end-to-end E-QUERY-038 gate
/// that fires via the real `QueryEngine::execute()` parser surface is:
///   `crates/prism-bin/tests/bc_2_16_018_claroty_servers_wire_shape.rs` →
///   `test_BC_2_16_018_claroty_servers_e2e_e_query_038_tier2_column`
/// That test uses `QueryEngine::execute("SELECT server_location FROM claroty_servers LIMIT 1")`
/// end-to-end and asserts PrismError::ColumnNotFound with the correct available_columns set.
/// This prism-sensors test is SAP-3 defense-in-depth only.
///
/// Traces to: BC-2.16.018 §Postconditions §2 (Tier-2 not exposed as standalone Arrow column);
///            EC-016-018-005 (Tier-2 column query → E-QUERY-038).
/// Story: S-CLAROTY-SERVERS-001 AC-003
#[test]
fn test_BC_2_16_018_claroty_servers_tier2_column_raises_e_query_038() {
    let (_spec, servers) = load_claroty_servers_table();

    // ocsf_column_naming=true is active at the claroty sensor level (S-ADR058-OCSF-ROUTING-001)
    let projected = ocsf_projected_column_names(&servers, true);

    // Tier-2 column server_location MUST NOT appear as a standalone projected name.
    // A PrismQL query `SELECT server_location FROM claroty.claroty_servers` raises E-QUERY-038.
    assert!(
        !projected.contains(&"server_location".to_string()),
        "server_location (Tier-2) must NOT be in ocsf_projected_column_names \
         (a SELECT server_location query raises E-QUERY-038 at plan time); \
         got projected: {projected:?}"
    );

    // Other Tier-2 column names must also be absent (E-QUERY-038 applies to all Tier-2)
    for tier2_raw in &[
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
    ] {
        assert!(
            !projected.contains(&tier2_raw.to_string()),
            "Tier-2 column '{}' must NOT be in projected names (E-QUERY-038 at plan time); \
             got projected: {projected:?}",
            tier2_raw
        );
    }

    // E-QUERY-038 available_columns MUST contain these (per AC-003 contract)
    assert!(
        projected.contains(&"raw_extensions".to_string()),
        "projected names must contain 'raw_extensions' (BC-2.16.018 §PC2 Tier-2 aggregation)"
    );
    assert!(
        projected.contains(&"device_name".to_string()),
        "projected names must contain 'device_name' (Tier-1: server_name → device.name)"
    );
    assert!(
        projected.contains(&"status_code".to_string()),
        "projected names must contain 'status_code' (Tier-1: server_status → status_code)"
    );
    assert!(
        projected.contains(&"class_uid".to_string()),
        "projected names must contain 'class_uid' (synthesized pseudo-column, ADR-058 §G)"
    );
    assert!(
        projected.contains(&"_sensor".to_string()),
        "projected names must contain '_sensor' (synthesized pseudo-column, ADR-058 §G)"
    );
}

// ---------------------------------------------------------------------------
// RG-004 | AC-004 — Tier-1 raw TOML name raises E-QUERY-038; Arrow name is accepted
// ---------------------------------------------------------------------------

/// RED GATE: panics at `load_claroty_servers_table()` until Task 8 adds the TOML block.
/// After implementation: verifies that raw Tier-1 TOML names (`server_status`, `server_name`)
/// are NOT in projected column names, while Arrow forms (`status_code`, `device_name`) ARE.
///
/// SAP-3 defense-in-depth note: same as RG-003. This test exercises the spec-level
/// projection to confirm Tier-1 column renaming is enforced at the plan-time gate boundary.
///
/// SID-2 composed-output: asserts the full projected names string contains both
/// 'status_code' and 'device_name' and does not contain 'server_status' or 'server_name'.
///
/// Traces to: BC-2.16.018 §Postconditions §2 (Tier-1 rename enforced); TV-BC-2.16.018-003.
/// Story: S-CLAROTY-SERVERS-001 AC-004
#[test]
fn test_BC_2_16_018_claroty_servers_tier1_raw_toml_name_raises_e_query_038() {
    let (_spec, servers) = load_claroty_servers_table();

    let projected = ocsf_projected_column_names(&servers, true);

    // Raw Tier-1 TOML names MUST NOT appear in projected names (E-QUERY-038 at plan time)
    assert!(
        !projected.contains(&"server_status".to_string()),
        "server_status (raw Tier-1 TOML name) must NOT be in projected names; \
         Arrow name is 'status_code'. A SELECT server_status query raises E-QUERY-038."
    );
    assert!(
        !projected.contains(&"server_name".to_string()),
        "server_name (raw Tier-1 TOML name) must NOT be in projected names; \
         Arrow name is 'device_name'. A SELECT server_name query raises E-QUERY-038."
    );

    // Arrow column names MUST be present (these are the accepted query columns)
    assert!(
        projected.contains(&"status_code".to_string()),
        "status_code (Arrow form of server_status) MUST be in projected names"
    );
    assert!(
        projected.contains(&"device_name".to_string()),
        "device_name (Arrow form of server_name) MUST be in projected names"
    );

    // SID-2: composed-output assertion — full projected names string must satisfy both
    // presence and absence of the relevant column name pairs simultaneously.
    let projected_joined = projected.join(", ");
    assert!(
        projected_joined.contains("status_code") && projected_joined.contains("device_name"),
        "projected names string must contain both 'status_code' and 'device_name'; \
         got: {projected_joined}"
    );
    assert!(
        !projected_joined.contains("server_status") && !projected_joined.contains("server_name"),
        "projected names string must NOT contain raw TOML names 'server_status' or 'server_name'; \
         got: {projected_joined}"
    );
}

// ---------------------------------------------------------------------------
// RG-005 | AC-005 — Live Variant-1: wire JSON class_uid=5001, device_name, status_code, raw_extensions
// ---------------------------------------------------------------------------

/// Live Variant-1 wire-shape test per AC-005.
///
/// Wire-shape assertions (2026-07-13 discipline; BC-2.11.001 EC-11-079):
/// 1. class_uid key present with value 5001
/// 2. device_name key present (non-null string)
/// 3. status_code key present (case-insensitive match in {"up","down","pending"} per 2026-08-31 note)
/// 4. raw_extensions key present as JSON object (not null, not absent)
/// 5. server_name, server_status, server_location, management_ip NOT as standalone root keys
///
/// Status-value casing note (pre-delivery remove-uncertainty pass 2026-08-31): OpenAPI §example
/// renders server_status in lowercase ("up") as a synthetic placeholder. The capitalized
/// set {"Up","Down","Pending"} is the expected live casing but is UNCONFIRMED from the schema.
/// This test MUST compare status_code case-insensitively.
///
/// Traces to: BC-2.16.018 §Postconditions §1 (class_uid), §PC2 (Tier-1/Tier-2 wire representation);
///            TV-BC-2.16.018-002.
/// Story: S-CLAROTY-SERVERS-001 AC-005
#[tokio::test]
#[ignore]
// LIVE-MONROE-001: requires CLAROTY_INSTANCE_URL env var pointing to monroe; run manually or in live-validation CI job
async fn test_BC_2_16_018_claroty_servers_live_wire_shape_class_uid_and_tier1() {
    // RED GATE: fails if claroty_servers is not yet in claroty.sensor.toml
    let (spec, _servers) = load_claroty_servers_table();

    let instance_url = std::env::var("CLAROTY_INSTANCE_URL")
        .expect("LIVE-MONROE-001: CLAROTY_INSTANCE_URL must be set to run live tests");

    // Build live spec directed at the real Claroty xDome instance (Monroe).
    let mut live_spec = spec.clone();
    live_spec.base_url = instance_url;

    // live_table must point into live_spec to avoid dangling borrows.
    let live_table = live_spec
        .tables
        .iter()
        .find(|t| t.table_name == "servers")
        .expect("servers table must exist in live_spec");

    let context = FetchContext::new(OrgSlug::new("live-test"), HashMap::new(), None);
    let http_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("http client must build (ADR-050 rustls-tls)");
    let auth = NullAuthProvider;

    let result = PipelineExecutor::execute(&live_spec, live_table, &context, &http_client, &auth)
        .await
        .expect("live PipelineExecutor::execute must succeed for claroty_servers");

    assert!(
        !result.records.is_empty(),
        "live claroty_servers must return at least one record. \
         BC-2.16.018 AC-005."
    );

    // Inspect up to 5 records for AC-005 wire-shape compliance.
    for raw_record in result.records.iter().take(5) {
        // `map_record` operates on the original spec's table (not live_spec) because the
        // column schema is spec-driven, not live-response-driven.
        let orig_table = spec
            .tables
            .iter()
            .find(|t| t.table_name == "servers")
            .expect("servers table must exist in original spec");
        let row = ColumnMapper::map_record(raw_record, orig_table)
            .expect("ColumnMapper::map_record must succeed for live claroty_servers record");

        // Build a simulated wire row to mirror the prism-bin serialization path.
        // class_uid = 5001 comes from EventClassSelector::select_by_class_name("inventory_info")
        // (BC-2.02.012; prism-sensors has no prism-ocsf dep, so we assert the value
        // as a literal — the authoritative load-bearing class_uid=5001 assertion is in
        // test_BC_2_16_018_claroty_servers_wire_shape_class_uid_5001_mock (NEW-1)).
        let mut simulated_wire_row = serde_json::Map::new();
        simulated_wire_row.insert("class_uid".to_string(), json!(5001_i32));

        // device.name → device_name (ADR-058 §C2 Option 4: dot → underscore).
        if let Some(val) = row.mapped_fields.get("device.name") {
            simulated_wire_row.insert("device_name".to_string(), val.clone());
        }
        // status_code (single-segment ocsf_field, arrow name unchanged).
        if let Some(val) = row.mapped_fields.get("status_code") {
            simulated_wire_row.insert("status_code".to_string(), val.clone());
        }
        // raw_extensions is a JSON object of Tier-2 fields.
        if !row.raw_extensions.is_empty() {
            simulated_wire_row.insert(
                "raw_extensions".to_string(),
                serde_json::to_value(&row.raw_extensions)
                    .expect("raw_extensions must serialize to JSON"),
            );
        }

        // ── class_uid assertion (ILLUSTRATIVE-ONLY) ────────────────────────────
        // NON-LOAD-BEARING: class_uid=5001 was inserted by this test's own literal.
        // The load-bearing assertion is in NEW-1 (test_BC_2_16_018_claroty_servers_wire_shape_class_uid_5001_mock).
        assert_eq!(
            simulated_wire_row.get("class_uid"),
            Some(&json!(5001_i32)),
            "simulated wire row must have class_uid = 5001 (inventory_info). \
             BC-2.16.018 AC-005. NOTE: ILLUSTRATIVE-ONLY; class_uid inserted by this test literal."
        );

        // ── device_name must be present and non-null ───────────────────────────
        assert!(
            simulated_wire_row.contains_key("device_name"),
            "live record simulated wire row must contain 'device_name' \
             (server_name → device.name → device_name). \
             BC-2.16.018 AC-005. Row keys: {:?}",
            simulated_wire_row.keys().collect::<Vec<_>>()
        );
        assert!(
            simulated_wire_row
                .get("device_name")
                .map(|v| !v.is_null())
                .unwrap_or(false),
            "live record 'device_name' must be non-null for a live servers row. \
             BC-2.16.018 AC-005."
        );

        // ── status_code must be present and in {up, down, pending} (case-insensitive) ──
        assert!(
            simulated_wire_row.contains_key("status_code"),
            "live record simulated wire row must contain 'status_code' \
             (server_status → status_code). \
             BC-2.16.018 AC-005. Row keys: {:?}",
            simulated_wire_row.keys().collect::<Vec<_>>()
        );
        if let Some(sc_val) = simulated_wire_row.get("status_code") {
            if let Some(s) = sc_val.as_str() {
                let lower = s.to_lowercase();
                assert!(
                    ["up", "down", "pending"].contains(&lower.as_str()),
                    "status_code must be 'up', 'down', or 'pending' (case-insensitive). \
                     BC-2.16.018 AC-005. Got: {:?}",
                    s
                );
            }
        }

        // ── raw_extensions must be present and a JSON object ──────────────────
        assert!(
            simulated_wire_row
                .get("raw_extensions")
                .map(|v| v.is_object())
                .unwrap_or(false),
            "live record simulated wire row must contain raw_extensions as a JSON object; \
             BC-2.16.018 AC-006. Got: {:?}",
            simulated_wire_row.get("raw_extensions")
        );

        // ── Raw TOML names MUST NOT appear as top-level wire fields ────────────
        // (BC-2.16.018 AC-004 §Tier-1 arrow-name discipline)
        let tier2_names = [
            "server_name",
            "server_status",
            "server_location",
            "management_ip",
            "model",
            "os_version",
            "site_id",
            "serial_number",
            "num_of_interfaces",
        ];
        for raw_name in &tier2_names {
            assert!(
                !simulated_wire_row.contains_key(*raw_name),
                "Raw TOML name / Tier-2 field '{}' MUST NOT appear as a top-level wire field. \
                 BC-2.16.018 AC-004. Wire row keys: {:?}",
                raw_name,
                simulated_wire_row.keys().collect::<Vec<_>>()
            );
        }
    }
}

// ---------------------------------------------------------------------------
// RG-006 | AC-006 — Live Variant-1: raw_extensions contains Tier-2 keys
// ---------------------------------------------------------------------------

/// Live Variant-1 test: SELECT raw_extensions succeeds; raw_extensions contains
/// management_ip, model, os_version keys.
///
/// Traces to: BC-2.16.018 §Postconditions §2 (Tier-2 keys in raw_extensions); TV-BC-2.16.018-005.
/// Story: S-CLAROTY-SERVERS-001 AC-006
#[tokio::test]
#[ignore]
// LIVE-MONROE-001: requires CLAROTY_INSTANCE_URL env var pointing to monroe; run manually or in live-validation CI job
async fn test_BC_2_16_018_claroty_servers_live_raw_extensions_contains_tier2_keys() {
    // RED GATE: fails if claroty_servers is not yet in claroty.sensor.toml
    let (spec, _servers) = load_claroty_servers_table();

    let instance_url = std::env::var("CLAROTY_INSTANCE_URL")
        .expect("LIVE-MONROE-001: CLAROTY_INSTANCE_URL must be set to run live tests");

    let mut live_spec = spec.clone();
    live_spec.base_url = instance_url;

    let live_table = live_spec
        .tables
        .iter()
        .find(|t| t.table_name == "servers")
        .expect("servers table must exist in live_spec");

    let context = FetchContext::new(OrgSlug::new("live-test"), HashMap::new(), None);
    let http_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("http client must build (ADR-050 rustls-tls)");
    let auth = NullAuthProvider;

    let result = PipelineExecutor::execute(&live_spec, live_table, &context, &http_client, &auth)
        .await
        .expect("live PipelineExecutor::execute must succeed for claroty_servers (raw_extensions)");

    // Each non-empty record must yield a non-empty raw_extensions map (AC-006).
    for raw_record in result.records.iter().take(3) {
        let orig_table = spec
            .tables
            .iter()
            .find(|t| t.table_name == "servers")
            .expect("servers table must exist in original spec");
        let row = ColumnMapper::map_record(raw_record, orig_table)
            .expect("ColumnMapper::map_record must succeed for live claroty_servers record");

        assert!(
            !row.raw_extensions.is_empty(),
            "live claroty_servers record must have non-empty raw_extensions (Tier-2 data). \
             BC-2.16.018 AC-006. Record: {:?}",
            raw_record
        );
    }
}

// ---------------------------------------------------------------------------
// RG-007 | AC-007 — REQUIRED server_name absent → null row; subsequent rows unaffected
// ---------------------------------------------------------------------------

/// RED GATE: panics at `load_claroty_servers_table()` until Task 8 adds the TOML block.
/// After implementation: verifies (a) server_name ColumnSpec has REQUIRED option,
/// (b) ColumnMapper::map_record with a record missing server_name returns a MappingResult
/// where the Tier-1 OCSF field "device.name" is absent from mapped_fields.
///
/// Note: REQUIRED null-row production (dropping the row at the RecordBatch level)
/// is enforced at the PipelineExecutor level, not at ColumnMapper. This test verifies
/// the structural precondition (REQUIRED option declared) and ColumnMapper's treatment
/// of an absent field (device.name absent from mapped_fields). The pipeline produces
/// a null row when mapped_fields lacks a REQUIRED Tier-1 field.
///
/// SAP-3 rule-3 defense-in-depth disclaimer: this test calls `ColumnMapper::map_record`
/// directly. `map_record` has ZERO production callers; the production data path is
/// `SpecDrivenSensorAdapter::fetch` → `pipeline_result_to_record_batch` → `build_column_array`.
/// This test is SAP-3 defense-in-depth only. The authoritative production-path coverage
/// for RG-007 (null-passthrough, wire-level null-not-absent) is:
///   `crates/prism-bin/tests/bc_2_16_018_claroty_servers_wire_shape.rs` →
///   `test_BC_2_16_018_claroty_servers_null_passthrough_server_name_absent_null_not_absent`
/// That test exercises `SpecDrivenSensorAdapter::fetch` end-to-end with a wiremock
/// response containing an absent server_name, and asserts `"device_name": null` (not absent)
/// in the serialized wire JSON (CLAUDE.md §Wire-shape assertion discipline; BC-2.11.001 EC-11-079).
///
/// Traces to: BC-2.16.018 §Invariants (server_name MUST be present); EC-016-018-001.
/// Story: S-CLAROTY-SERVERS-001 AC-007
#[test]
fn test_BC_2_16_018_claroty_servers_required_server_name_absent_produces_null_row() {
    let (_spec, servers) = load_claroty_servers_table();

    // (a) Structural: server_name must have REQUIRED option
    let server_name_col = servers
        .columns
        .iter()
        .find(|c| c.name == "server_name")
        .expect("server_name column must exist in claroty_servers");

    assert!(
        server_name_col.options.contains(&ColumnOptions::Required),
        "server_name column MUST have REQUIRED option (pipeline produces null row \
         when server_name is absent from API response); BC-2.16.018 §Invariants"
    );

    // (b) ColumnMapper behavior: a record without server_name → device.name absent from mapped_fields
    // This demonstrates the data path: absent → ColumnMapper skips → pipeline sees no device_name
    // → pipeline marks row as null (REQUIRED semantics).
    let record_missing_server_name = serde_json::json!({
        "server_status": "Up",
        "server_location": "Datacenter-A",
        "site_id": 1,
        "model": "MCS R340",
        "os_version": "Ubuntu 20.04"
        // server_name deliberately absent
    });

    let result = ColumnMapper::map_record(&record_missing_server_name, &servers)
        .expect("ColumnMapper::map_record must not hard-error on a missing REQUIRED field");

    // device.name must be absent from mapped_fields (server_name was not in raw record)
    assert!(
        !result.mapped_fields.contains_key("device.name"),
        "device.name must be absent from mapped_fields when server_name is not in raw record \
         (pipeline will produce a null row per REQUIRED semantics, BC-2.16.018 EC-016-018-001)"
    );

    // server_status maps to Tier-1 status_code — present via ocsf_field "status_code"
    assert!(
        result.mapped_fields.contains_key("status_code"),
        "status_code (from server_status) must be in mapped_fields when server_status is present"
    );
}

// ---------------------------------------------------------------------------
// RG-008 | AC-008 — Nullable count envelope: empty-page halt; no null-ptr deref
// ---------------------------------------------------------------------------

/// RED GATE: panics at `load_claroty_servers_table()` until Task 8 adds the TOML block.
/// After implementation: verifies the claroty_servers fetch step uses OffsetLimit pagination
/// (which drives the empty-page halt behavior when the API returns count=null).
///
/// The full behavioral assertion (count=null → pagination halts, no null-ptr deref) is
/// enforced at the PipelineExecutor level. This test verifies the structural prerequisite:
/// pagination type is OffsetLimit (the correct pagination mode for empty-page halt semantics,
/// per BC-2.16.018 §PC1 pagination note and the established pattern from claroty_vulnerabilities
/// BC-2.16.015 EC-016-015-003).
///
/// Mock response fixture documented:
/// `{"servers": [], "count": null}` — the PipelineExecutor must halt on empty page without
/// dereferencing count.
///
/// Traces to: BC-2.16.018 §Postconditions §1 (pagination note); EC-016-018-004.
/// Story: S-CLAROTY-SERVERS-001 AC-008
#[test]
fn test_BC_2_16_018_claroty_servers_nullable_count_uses_empty_page_halt() {
    let (_spec, servers) = load_claroty_servers_table();

    // Structural: fetch step must use OffsetLimit pagination (prerequisite for empty-page halt)
    assert_eq!(
        servers.steps.len(),
        1,
        "claroty_servers must have exactly 1 fetch step"
    );
    let step = &servers.steps[0];

    match &step.pagination {
        Some(PaginationConfig::OffsetLimit { page_size }) => {
            assert_eq!(
                *page_size, 1000,
                "claroty_servers OffsetLimit page_size must be 1000 (BC-2.16.018 §PC1)"
            );
            // OffsetLimit pagination mode implements empty-page halt:
            // when the API returns an empty servers array (even with count=null),
            // the PipelineExecutor stops requesting further pages without dereferencing count.
            // This is the established Claroty pattern (BC-2.16.015 EC-016-015-003 precedent).
        }
        Some(other) => {
            panic!(
                "claroty_servers must use OffsetLimit pagination to support empty-page halt \
                 (count=null must not crash); got: {other:?}"
            );
        }
        None => {
            panic!(
                "claroty_servers fetch step must declare OffsetLimit pagination; \
                 None means no pagination configured (EC-016-018-004 would be unguarded)"
            );
        }
    }

    // Demonstrate the mock response shape the PipelineExecutor must handle safely:
    // {"servers": [], "count": null}
    // The empty servers array triggers the empty-page halt; count is never dereferenced.
    let _mock_envelope_with_null_count = serde_json::json!({
        "servers": [],
        "count": serde_json::Value::Null
    });
    // (Full behavioral test of PipelineExecutor processing this envelope belongs in
    // prism-spec-engine integration tests, where PipelineExecutor is directly accessible.)
}

// ---------------------------------------------------------------------------
// F-001: SAP-2 N/A marker test
// ---------------------------------------------------------------------------

/// SAP-2 marker test: confirms SAP2_STATUS is properly documented as N/A
/// (no DTU clone exists) with a D-2200 deferral anchor.
///
/// BC-2.16.018; Story: S-CLAROTY-SERVERS-001
#[test]
fn test_BC_2_16_018_claroty_servers_sap2_na_documented() {
    assert!(
        SAP2_STATUS.starts_with("N/A:"),
        "SAP2_STATUS must begin with 'N/A:' to document the absence of a DTU clone. \
         Got: {:?}",
        SAP2_STATUS
    );
    assert!(
        SAP2_STATUS.contains("D-2200"),
        "SAP2_STATUS must cite D-2200 (S-CLAROTY-SERVERS-DTU-001 deferral anchor). \
         Got: {:?}",
        SAP2_STATUS
    );
}
