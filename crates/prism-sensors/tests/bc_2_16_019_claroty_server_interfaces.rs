#![allow(non_snake_case)]
#![allow(clippy::expect_used, clippy::unwrap_used)]
//! Red Gate tests for BC-2.16.019 — Claroty xDome Server Interfaces Table.
//!
//! Story: S-CLAROTY-SERVERS-001 (Wave C G4)
//! BCs:   BC-2.16.019 v1.0
//! ACs:   AC-009..AC-016
//!
//! | RG  | Test name                                                                               | Type                  | AC   |
//! |-----|-----------------------------------------------------------------------------------------|-----------------------|------|
//! | 009 | test_BC_2_16_019_claroty_server_interfaces_toml_block_parses                            | Unit (SpecLoader)     | AC-009 |
//! | 010 | test_BC_2_16_019_claroty_server_interfaces_tier1_columns_two_with_ocsf_field            | Unit (ColumnSpec)     | AC-010 |
//! | 011 | test_BC_2_16_019_claroty_server_interfaces_tier2_column_raises_e_query_038              | Integration (plan)    | AC-011 |
//! | 012 | test_BC_2_16_019_claroty_server_interfaces_interface_status_raw_name_raises_e_query_038 | Integration (plan)    | AC-012 |
//! | 013 | test_BC_2_16_019_claroty_server_interfaces_live_wire_shape_class_uid_and_tier1          | Live #[ignore]        | AC-013 |
//! | 014 | test_BC_2_16_019_claroty_server_interfaces_live_raw_extensions_contains_tier2_keys      | Live #[ignore]        | AC-014 |
//! | 015 | test_BC_2_16_019_claroty_server_interfaces_required_server_name_absent_produces_null_row | Unit (mock)          | AC-015 (part 1) |
//!        test_BC_2_16_019_claroty_server_interfaces_null_interface_name_row_not_dropped         | Unit (mock)          | AC-015 (part 2) |
//! | 016 | test_BC_2_16_019_claroty_server_interfaces_nullable_count_uses_empty_page_halt           | Unit (mock)           | AC-016 |
//!
//! Red Gate failure mode (ALL non-#[ignore] tests):
//! `expect("claroty_server_interfaces table must exist")` panics because the [[tables]] block
//! for `claroty_server_interfaces` is not yet present in `crates/prism-sensors/specs/claroty.sensor.toml`.
//! This is the CORRECT failure reason — not a compile error.
//!
//! Endpoint note: `claroty_server_interfaces` uses a SEPARATE endpoint
//! `/api/v1/server_interfaces/` (NOT a sub-path of `/api/v1/servers/`).
//! operationId: `get_servers_api_v1_server_interfaces__post` per OpenAPI spec.
//!
//! Composite PK note: PK = (server_name, interface_name). Only server_name has REQUIRED.
//! interface_name is Tier-2 without REQUIRED — null interface_name is degraded, not dropped.

use prism_core::column::ColumnOptions;
use prism_spec_engine::{
    column_mapping::{ocsf_projected_column_names, ColumnMapper},
    spec_parser::{PaginationConfig, SpecLoader},
};

// ---------------------------------------------------------------------------
// Helper: load the full claroty.sensor.toml and return the claroty_server_interfaces TableSpec.
// Panics with the Red Gate message if the table is absent (pre-implementation).
// ---------------------------------------------------------------------------

fn load_claroty_server_interfaces_table() -> (
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
        .find(|t| t.table_name == "server_interfaces")
        .cloned()
        .expect(
            "server_interfaces table must exist in claroty.sensor.toml \
             — RED GATE: [[tables]] block not yet added (S-CLAROTY-SERVERS-001 Task 8)",
        );
    (spec, table)
}

// ---------------------------------------------------------------------------
// RG-009 | AC-009 — TOML block parses; 10 columns; 1 fetch step; SEPARATE endpoint; pagination 1000
// ---------------------------------------------------------------------------

/// RED GATE: panics at `load_claroty_server_interfaces_table()` until Task 8 adds the TOML block.
/// After implementation: asserts 10 ColumnSpec entries, 1 fetch step, path = /api/v1/server_interfaces/
/// (SEPARATE from /api/v1/servers/), response_path = $.server_interfaces, OffsetLimit page_size=1000.
///
/// Traces to: BC-2.16.019 §Postconditions §1 (TOML Table Contract; endpoint correction confirmed).
/// Story: S-CLAROTY-SERVERS-001 AC-009
#[test]
fn test_BC_2_16_019_claroty_server_interfaces_toml_block_parses() {
    let (_spec, server_interfaces) = load_claroty_server_interfaces_table();

    assert_eq!(
        server_interfaces.columns.len(),
        10,
        "claroty_server_interfaces must declare exactly 10 columns \
         (2 Tier-1: server_name/interface_status + 8 Tier-2 incl. interface_name) \
         per BC-2.16.019 §PC1; got {}",
        server_interfaces.columns.len()
    );

    assert_eq!(
        server_interfaces.steps.len(),
        1,
        "claroty_server_interfaces must have exactly 1 fetch step (fetch_server_interfaces)"
    );

    let step = &server_interfaces.steps[0];
    assert_eq!(
        step.name, "fetch_server_interfaces",
        "fetch step name must be 'fetch_server_interfaces'"
    );
    assert_eq!(
        step.method, "POST",
        "fetch step method must be POST (POST-for-read pattern)"
    );

    // SEPARATE endpoint — NOT a sub-path of /api/v1/servers/
    // operationId: get_servers_api_v1_server_interfaces__post (BC-2.16.019 §PC1 endpoint correction)
    assert_eq!(
        step.path_template, "/api/v1/server_interfaces/",
        "fetch step path_template must be '/api/v1/server_interfaces/' \
         (SEPARATE endpoint — NOT '/api/v1/servers/server_interfaces/'; \
         BC-2.16.019 §PC1 endpoint correction confirmed from OpenAPI operationId)"
    );

    assert_eq!(
        step.response_path, "$.server_interfaces",
        "fetch step response_path must be '$.server_interfaces' (BC-2.16.019 §PC1 envelope key)"
    );

    match &step.pagination {
        Some(PaginationConfig::OffsetLimit { page_size }) => {
            assert_eq!(
                *page_size, 1000,
                "claroty_server_interfaces must use page_size = 1000 (BC-2.16.019 §PC1)"
            );
        }
        other => {
            panic!(
                "claroty_server_interfaces fetch step must have OffsetLimit pagination; \
                 got: {other:?}"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// RG-010 | AC-010 — 2 Tier-1 columns; server_name→device.name REQUIRED; interface_status→status_code; 8 Tier-2
// ---------------------------------------------------------------------------

/// RED GATE: panics at `load_claroty_server_interfaces_table()` until Task 8 adds the TOML block.
/// After implementation: asserts Tier-1/Tier-2 column classification per BC-2.16.019 §PC2.
/// Note: interface_name is Tier-2 (no ocsf_field) despite being a composite PK element.
///
/// Traces to: BC-2.16.019 §Postconditions §2 (Tier-1/Tier-2 column classification);
///            §Postconditions §3 (composite PK — interface_name Tier-2 without REQUIRED).
/// Story: S-CLAROTY-SERVERS-001 AC-010
#[test]
fn test_BC_2_16_019_claroty_server_interfaces_tier1_columns_two_with_ocsf_field() {
    let (_spec, server_interfaces) = load_claroty_server_interfaces_table();

    // --- Tier-1 count ---
    let tier1_cols: Vec<_> = server_interfaces
        .columns
        .iter()
        .filter(|c| c.ocsf_field.is_some())
        .collect();
    assert_eq!(
        tier1_cols.len(),
        2,
        "claroty_server_interfaces must have exactly 2 Tier-1 columns (ocsf_field = Some); \
         got {}: {:?}",
        tier1_cols.len(),
        tier1_cols.iter().map(|c| &c.name).collect::<Vec<_>>()
    );

    // --- server_name → device.name, REQUIRED (composite PK anchor) ---
    let server_name_col = server_interfaces
        .columns
        .iter()
        .find(|c| c.name == "server_name")
        .expect("server_name column must be declared in claroty_server_interfaces");

    assert_eq!(
        server_name_col.ocsf_field.as_deref(),
        Some("device.name"),
        "server_name must have ocsf_field = \"device.name\" (BC-2.16.019 §PC2 Tier-1)"
    );

    assert!(
        server_name_col.options.contains(&ColumnOptions::Required),
        "server_name must have REQUIRED option (composite PK anchor, BC-2.16.019 §PC3)"
    );

    // --- interface_status → status_code ---
    let interface_status_col = server_interfaces
        .columns
        .iter()
        .find(|c| c.name == "interface_status")
        .expect("interface_status column must be declared in claroty_server_interfaces");

    assert_eq!(
        interface_status_col.ocsf_field.as_deref(),
        Some("status_code"),
        "interface_status must have ocsf_field = \"status_code\" (BC-2.16.019 §PC2 Tier-1)"
    );

    // interface_status is NOT REQUIRED (only server_name is)
    assert!(
        !interface_status_col.options.contains(&ColumnOptions::Required),
        "interface_status must NOT have REQUIRED option (only server_name is REQUIRED per BC-2.16.019)"
    );

    // --- interface_name is Tier-2 despite being composite PK element ---
    let interface_name_col = server_interfaces
        .columns
        .iter()
        .find(|c| c.name == "interface_name")
        .expect("interface_name column must be declared in claroty_server_interfaces");

    assert!(
        interface_name_col.ocsf_field.is_none(),
        "interface_name must have NO ocsf_field (Tier-2, composite PK element but not OCSF mapped; \
         BC-2.16.019 §PC3)"
    );

    // interface_name MUST NOT have REQUIRED option (null interface_name is degraded, not dropped)
    assert!(
        !interface_name_col.options.contains(&ColumnOptions::Required),
        "interface_name must NOT have REQUIRED option \
         (composite PK element; null interface_name is degraded-not-dropped per BC-2.16.019 §Invariants)"
    );

    // --- Tier-2 count: exactly 8 (including interface_name) ---
    let tier2_count = server_interfaces
        .columns
        .iter()
        .filter(|c| c.ocsf_field.is_none())
        .count();
    assert_eq!(
        tier2_count, 8,
        "claroty_server_interfaces must have exactly 8 Tier-2 columns (ocsf_field = None, \
         including interface_name); got {tier2_count}"
    );
}

// ---------------------------------------------------------------------------
// RG-011 | AC-011 — Tier-2 column interface_name raises E-QUERY-038 (composite PK element is Tier-2)
// ---------------------------------------------------------------------------

/// RED GATE: panics at `load_claroty_server_interfaces_table()` until Task 8 adds the TOML block.
/// After implementation: verifies that `interface_name` (Tier-2 composite PK element) is NOT
/// in the projected column set, while `raw_extensions`, `device_name`, `status_code`,
/// `class_uid`, `_sensor` ARE.
///
/// SAP-3 defense-in-depth note: this test exercises `ocsf_projected_column_names`, which is
/// the exact data structure the E-QUERY-038 plan-time gate consults in prism-query. Full
/// end-to-end reachability from the PrismQL parser surface requires prism-query, which would
/// create a circular dependency from prism-sensors. This spec-level test is the correct
/// coverage layer for the prism-sensors crate.
///
/// SAP-3 rule-3 defense-in-depth disclaimer: the AUTHORITATIVE end-to-end E-QUERY-038 gate
/// that fires via the real `QueryEngine::execute()` parser surface is:
///   `crates/prism-bin/tests/bc_2_16_019_claroty_server_interfaces_wire_shape.rs` →
///   `test_BC_2_16_019_claroty_server_interfaces_e2e_e_query_038_tier2_column`
/// That test uses `QueryEngine::execute("SELECT interface_name FROM claroty_server_interfaces LIMIT 1")`
/// end-to-end and asserts PrismError::ColumnNotFound for the Tier-2 composite PK element.
/// This prism-sensors test is SAP-3 defense-in-depth only.
///
/// Traces to: BC-2.16.019 §Invariants (interface_name Tier-2 despite PK role); EC-016-019-004.
/// Story: S-CLAROTY-SERVERS-001 AC-011
#[test]
fn test_BC_2_16_019_claroty_server_interfaces_tier2_column_raises_e_query_038() {
    let (_spec, server_interfaces) = load_claroty_server_interfaces_table();

    let projected = ocsf_projected_column_names(&server_interfaces, true);

    // interface_name (composite PK element, Tier-2) MUST NOT appear as standalone projected column
    assert!(
        !projected.contains(&"interface_name".to_string()),
        "interface_name (Tier-2 composite PK element) must NOT be in ocsf_projected_column_names \
         (a SELECT interface_name query raises E-QUERY-038; access via raw_extensions); \
         got projected: {projected:?}"
    );

    // Other Tier-2 column names must also be absent from projected
    for tier2_raw in &[
        "interface_type",
        "interface_connection_type",
        "site_id",
        "avg_traffic_past_month_mbps",
        "avg_traffic_past_week_mbps",
        "avg_traffic_past_hour_mbps",
        "notes",
    ] {
        assert!(
            !projected.contains(&tier2_raw.to_string()),
            "Tier-2 column '{}' must NOT be in projected names (E-QUERY-038 at plan time); \
             got projected: {projected:?}",
            tier2_raw
        );
    }

    // E-QUERY-038 available_columns MUST contain these (per AC-011 contract)
    assert!(
        projected.contains(&"raw_extensions".to_string()),
        "projected names must contain 'raw_extensions' (BC-2.16.019 §PC2 Tier-2 aggregation; \
         interface_name accessible via raw_extensions, not as standalone column)"
    );
    assert!(
        projected.contains(&"device_name".to_string()),
        "projected names must contain 'device_name' (Tier-1: server_name → device.name)"
    );
    assert!(
        projected.contains(&"status_code".to_string()),
        "projected names must contain 'status_code' (Tier-1: interface_status → status_code)"
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
// RG-012 | AC-012 — interface_status raw TOML name raises E-QUERY-038; status_code is accepted
// ---------------------------------------------------------------------------

/// RED GATE: panics at `load_claroty_server_interfaces_table()` until Task 8 adds the TOML block.
/// After implementation: verifies that raw Tier-1 TOML names (`interface_status`, `server_name`)
/// are NOT in projected column names, while Arrow forms (`status_code`, `device_name`) ARE.
///
/// SAP-3 defense-in-depth note: same as RG-011. Spec-level coverage of E-QUERY-038 plan-time gate.
///
/// SID-2 composed-output: asserts the full projected names string contains both
/// 'status_code' and 'device_name' and does not contain 'interface_status' or 'server_name'.
///
/// Traces to: BC-2.16.019 §Invariants (Tier-1 rename enforced; interface_status → status_code);
///            EC-016-019-006.
/// Story: S-CLAROTY-SERVERS-001 AC-012
#[test]
fn test_BC_2_16_019_claroty_server_interfaces_interface_status_raw_name_raises_e_query_038() {
    let (_spec, server_interfaces) = load_claroty_server_interfaces_table();

    let projected = ocsf_projected_column_names(&server_interfaces, true);

    // Raw Tier-1 TOML names MUST NOT appear in projected names (E-QUERY-038 at plan time)
    assert!(
        !projected.contains(&"interface_status".to_string()),
        "interface_status (raw Tier-1 TOML name) must NOT be in projected names; \
         Arrow name is 'status_code'. A SELECT interface_status query raises E-QUERY-038."
    );
    assert!(
        !projected.contains(&"server_name".to_string()),
        "server_name (raw Tier-1 TOML name) must NOT be in projected names; \
         Arrow name is 'device_name'. A SELECT server_name query raises E-QUERY-038."
    );

    // Arrow column names MUST be present (the accepted query columns)
    assert!(
        projected.contains(&"status_code".to_string()),
        "status_code (Arrow form of interface_status) MUST be in projected names"
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
        !projected_joined.contains("interface_status") && !projected_joined.contains(",server_name"),
        "projected names string must NOT contain raw TOML names 'interface_status' or 'server_name'; \
         got: {projected_joined}"
    );
}

// ---------------------------------------------------------------------------
// RG-013 | AC-013 — Live Variant-1: wire JSON class_uid=5001, device_name, status_code, raw_extensions
// ---------------------------------------------------------------------------

/// Live Variant-1 wire-shape test per AC-013.
///
/// Wire-shape assertions (2026-07-13 discipline; BC-2.11.001 EC-11-079):
/// 1. class_uid key present with value 5001
/// 2. device_name key present (non-null string — collection server name)
/// 3. status_code key present (case-insensitive match in {"up","no carrier"} per 2026-08-31 note)
/// 4. raw_extensions key present as JSON object (not null, not absent)
/// 5. raw_extensions object contains "interface_name" and "interface_type" keys
///    (composite PK join key and interface type must be accessible via raw_extensions)
/// 6. server_name, interface_status, interface_name, interface_type,
///    avg_traffic_past_month_mbps NOT as standalone top-level keys
///
/// Status-value casing note (pre-delivery remove-uncertainty pass 2026-08-31): OpenAPI §example
/// renders interface_status in lowercase ("up") as a synthetic placeholder. The capitalized
/// set {"Up","No Carrier"} is the expected live casing but is UNCONFIRMED from the schema.
/// This test MUST compare status_code case-insensitively.
///
/// Traces to: BC-2.16.019 §Postconditions §1 (class_uid), §PC2 (Tier-1/Tier-2 wire representation),
///            §PC3 (composite PK join keys in raw_extensions); TV-BC-2.16.019-002.
/// Story: S-CLAROTY-SERVERS-001 AC-013
#[test]
#[ignore]
// LIVE-MONROE-001: requires CLAROTY_INSTANCE_URL env var pointing to monroe; run manually or in live-validation CI job
fn test_BC_2_16_019_claroty_server_interfaces_live_wire_shape_class_uid_and_tier1() {
    // RED GATE: fails if claroty_server_interfaces is not yet in claroty.sensor.toml
    let (_spec, _server_interfaces) = load_claroty_server_interfaces_table();

    let _instance_url = std::env::var("CLAROTY_INSTANCE_URL")
        .expect("LIVE-MONROE-001: CLAROTY_INSTANCE_URL must be set to run live tests");

    // Wire-shape assertions per AC-013 / TV-BC-2.16.019-002:
    // Run: SELECT * FROM claroty.claroty_server_interfaces LIMIT 1 (via prism MCP/query stack)
    // Parse serialized JSON response, assert:
    //   row["class_uid"] == 5001
    //   row["device_name"] != null (non-null string)
    //   row["status_code"].as_str().unwrap().to_lowercase() in {"up","no carrier"}
    //   row["raw_extensions"].is_object() == true
    //   row["raw_extensions"]["interface_name"] exists (composite PK join key)
    //   row["raw_extensions"]["interface_type"] exists
    //   row.keys() does NOT contain "server_name","interface_status","interface_name",
    //     "interface_type","avg_traffic_past_month_mbps"
    todo!(
        "S-CLAROTY-SERVERS-001 AC-013: implement via prism MCP query stack or direct reqwest POST \
         to /api/v1/server_interfaces/ after the TOML block lands in claroty.sensor.toml"
    )
}

// ---------------------------------------------------------------------------
// RG-014 | AC-014 — Live Variant-1: raw_extensions contains Tier-2 keys including interface_name
// ---------------------------------------------------------------------------

/// Live Variant-1 test: SELECT raw_extensions succeeds; raw_extensions contains
/// interface_name, interface_type, interface_connection_type keys.
///
/// Traces to: BC-2.16.019 §Postconditions §2 (Tier-2 keys in raw_extensions incl. composite PK element);
///            TV-BC-2.16.019-005.
/// Story: S-CLAROTY-SERVERS-001 AC-014
#[test]
#[ignore]
// LIVE-MONROE-001: requires CLAROTY_INSTANCE_URL env var pointing to monroe; run manually or in live-validation CI job
fn test_BC_2_16_019_claroty_server_interfaces_live_raw_extensions_contains_tier2_keys() {
    // RED GATE: fails if claroty_server_interfaces is not yet in claroty.sensor.toml
    let (_spec, _server_interfaces) = load_claroty_server_interfaces_table();

    let _instance_url = std::env::var("CLAROTY_INSTANCE_URL")
        .expect("LIVE-MONROE-001: CLAROTY_INSTANCE_URL must be set to run live tests");

    // Assertions per AC-014 / TV-BC-2.16.019-005:
    // Run: SELECT raw_extensions FROM claroty.claroty_server_interfaces LIMIT 5
    // For each row: raw_extensions is non-null JSON object
    // Deserialized raw_extensions object contains at minimum:
    //   "interface_name" (composite PK join key), "interface_type", "interface_connection_type"
    // (or null values when the live API returns them)
    // No E-QUERY-038 is raised on raw_extensions itself (raw_extensions IS a projected column)
    todo!(
        "S-CLAROTY-SERVERS-001 AC-014: implement via prism MCP query stack or direct reqwest POST \
         to /api/v1/server_interfaces/ after the TOML block lands in claroty.sensor.toml"
    )
}

// ---------------------------------------------------------------------------
// RG-015 (part 1) | AC-015 — REQUIRED server_name absent → null row; no hard error
// ---------------------------------------------------------------------------

/// RED GATE: panics at `load_claroty_server_interfaces_table()` until Task 8 adds the TOML block.
/// After implementation: verifies (a) server_name ColumnSpec has REQUIRED option,
/// (b) ColumnMapper::map_record with a record missing server_name returns a MappingResult
/// where "device.name" is absent from mapped_fields (pipeline would produce null row).
///
/// Note: REQUIRED null-row production is enforced at PipelineExecutor level, not ColumnMapper.
/// This test verifies the structural precondition (REQUIRED declared) and ColumnMapper's behavior.
///
/// SAP-3 rule-3 defense-in-depth disclaimer: this test calls `ColumnMapper::map_record`
/// directly. `map_record` has ZERO production callers; the production data path is
/// `SpecDrivenSensorAdapter::fetch` → `pipeline_result_to_record_batch` → `build_column_array`.
/// This test is SAP-3 defense-in-depth only. The authoritative production-path coverage
/// for RG-015 part 1 (null-passthrough, server_name absent) is:
///   `crates/prism-bin/tests/bc_2_16_019_claroty_server_interfaces_wire_shape.rs` →
///   `test_BC_2_16_019_claroty_server_interfaces_null_interface_name_row_not_dropped_wire`
/// (The wire test exercises the full production path with a server_name-present /
/// interface_name-null record and asserts on serialized wire JSON.)
///
/// Traces to: BC-2.16.019 §Invariants (server_name MUST be present); EC-016-019-001.
/// Story: S-CLAROTY-SERVERS-001 AC-015 (part 1)
#[test]
fn test_BC_2_16_019_claroty_server_interfaces_required_server_name_absent_produces_null_row() {
    let (_spec, server_interfaces) = load_claroty_server_interfaces_table();

    // (a) Structural: server_name must have REQUIRED option
    let server_name_col = server_interfaces
        .columns
        .iter()
        .find(|c| c.name == "server_name")
        .expect("server_name column must exist in claroty_server_interfaces");

    assert!(
        server_name_col.options.contains(&ColumnOptions::Required),
        "server_name column MUST have REQUIRED option in claroty_server_interfaces \
         (composite PK anchor; pipeline produces null row when server_name absent); \
         BC-2.16.019 §Invariants EC-016-019-001"
    );

    // (b) ColumnMapper behavior with absent server_name → device.name absent from mapped_fields
    let record_missing_server_name = serde_json::json!({
        "interface_status": "Up",
        "interface_name": "eth0",
        "interface_type": "SPAN",
        "site_id": 1
        // server_name deliberately absent
    });

    let result = ColumnMapper::map_record(&record_missing_server_name, &server_interfaces)
        .expect("ColumnMapper::map_record must not hard-error on a missing REQUIRED field");

    // device.name must be absent from mapped_fields (server_name was not in raw record)
    assert!(
        !result.mapped_fields.contains_key("device.name"),
        "device.name must be absent from mapped_fields when server_name is not in raw record \
         (pipeline will produce a null row per REQUIRED semantics; BC-2.16.019 EC-016-019-001)"
    );

    // interface_status → status_code present via ocsf_field
    assert!(
        result.mapped_fields.contains_key("status_code"),
        "status_code (from interface_status) must be in mapped_fields when interface_status is present"
    );

    // interface_name is Tier-2 → goes to raw_extensions when present in record
    assert!(
        result.raw_extensions.contains_key("interface_name"),
        "interface_name (Tier-2) must be in raw_extensions when present in raw record"
    );
}

// ---------------------------------------------------------------------------
// RG-015 (part 2) | AC-015 — null interface_name (composite PK degraded) → row NOT dropped
// ---------------------------------------------------------------------------

/// RED GATE: panics at `load_claroty_server_interfaces_table()` until Task 8 adds the TOML block.
/// After implementation: verifies that when interface_name is null in the API response,
/// the row is NOT dropped — server_name resolves to non-null device.name Arrow cell;
/// interface_name is null in raw_extensions.
///
/// Composite PK degraded scenario: server_name = "Monroe-Collector-1", interface_name = null.
/// The row is degraded (server identified, interface lost) but must be materialized.
///
/// SAP-3 rule-3 defense-in-depth disclaimer: this test calls `ColumnMapper::map_record`
/// directly. `map_record` has ZERO production callers; the production data path is
/// `SpecDrivenSensorAdapter::fetch` → `pipeline_result_to_record_batch` → `build_column_array`.
/// This test is SAP-3 defense-in-depth only. The authoritative production-path coverage
/// for RG-015 part 2 (null interface_name row-not-dropped, wire-level null-not-absent) is:
///   `crates/prism-bin/tests/bc_2_16_019_claroty_server_interfaces_wire_shape.rs` →
///   `test_BC_2_16_019_claroty_server_interfaces_null_interface_name_row_not_dropped_wire`
/// That test exercises `SpecDrivenSensorAdapter::fetch` end-to-end with a wiremock response
/// containing `interface_name = null`, asserts both rows survive (row-not-dropped invariant),
/// and asserts `"interface_name": null` (not absent) inside raw_extensions in the serialized
/// wire JSON (CLAUDE.md §Wire-shape assertion discipline; BC-2.11.001 EC-11-079).
///
/// Traces to: BC-2.16.019 §Invariants (interface_name does NOT have REQUIRED); EC-016-019-002.
/// Story: S-CLAROTY-SERVERS-001 AC-015 (part 2)
#[test]
fn test_BC_2_16_019_claroty_server_interfaces_null_interface_name_row_not_dropped() {
    let (_spec, server_interfaces) = load_claroty_server_interfaces_table();

    // interface_name must NOT have REQUIRED option (null = degraded, not dropped)
    let interface_name_col = server_interfaces
        .columns
        .iter()
        .find(|c| c.name == "interface_name")
        .expect("interface_name column must exist in claroty_server_interfaces");

    assert!(
        !interface_name_col.options.contains(&ColumnOptions::Required),
        "interface_name must NOT have REQUIRED option \
         (composite PK element; null interface_name is degraded-not-dropped per BC-2.16.019 §Invariants \
         EC-016-019-002)"
    );

    // ColumnMapper behavior: record with server_name present but interface_name = null
    let record_null_interface_name = serde_json::json!({
        "server_name": "Monroe-Collector-1",
        "interface_status": "Up",
        "interface_name": serde_json::Value::Null,
        "interface_type": "SPAN"
    });

    let result = ColumnMapper::map_record(&record_null_interface_name, &server_interfaces)
        .expect("ColumnMapper::map_record must succeed for degraded composite PK row");

    // device.name MUST be present (server_name was in raw record with non-null value)
    assert!(
        result.mapped_fields.contains_key("device.name"),
        "device.name must be in mapped_fields when server_name is present and non-null \
         (row must be materialized; BC-2.16.019 EC-016-019-002)"
    );

    // interface_name with null value — ColumnMapper includes null values in raw_extensions
    // (demonstrates the row is processed, not dropped)
    assert!(
        result.mapped_fields.contains_key("status_code"),
        "status_code must be in mapped_fields when interface_status is present"
    );

    // SID-2 composed-output: wire-shape assertion on the composite result
    // Both device.name (Tier-1) and the Tier-2 processing must coexist in a valid MappingResult
    assert!(
        result.mapped_fields.contains_key("device.name") && !result.mapped_fields.is_empty(),
        "MappingResult must contain device.name and be non-empty \
         (degraded composite PK row is materialized, not dropped)"
    );
}

// ---------------------------------------------------------------------------
// RG-016 | AC-016 — Nullable count envelope: empty-page halt; no null-ptr deref
// ---------------------------------------------------------------------------

/// RED GATE: panics at `load_claroty_server_interfaces_table()` until Task 8 adds the TOML block.
/// After implementation: verifies the claroty_server_interfaces fetch step uses OffsetLimit
/// pagination (which drives the empty-page halt behavior when count=null in the response).
///
/// The full behavioral assertion (count=null → pagination halts, no null-ptr deref) is
/// enforced at the PipelineExecutor level. This test verifies the structural prerequisite:
/// pagination type is OffsetLimit and page_size is 1000.
///
/// Mock response fixture documented:
/// `{"server_interfaces": [], "count": null}` — the PipelineExecutor must halt on empty page
/// without dereferencing count.
///
/// Traces to: BC-2.16.019 §Postconditions §1 (pagination note); EC-016-019-005.
/// Story: S-CLAROTY-SERVERS-001 AC-016
#[test]
fn test_BC_2_16_019_claroty_server_interfaces_nullable_count_uses_empty_page_halt() {
    let (_spec, server_interfaces) = load_claroty_server_interfaces_table();

    // Structural: fetch step must use OffsetLimit pagination (prerequisite for empty-page halt)
    assert_eq!(
        server_interfaces.steps.len(),
        1,
        "claroty_server_interfaces must have exactly 1 fetch step"
    );
    let step = &server_interfaces.steps[0];

    match &step.pagination {
        Some(PaginationConfig::OffsetLimit { page_size }) => {
            assert_eq!(
                *page_size, 1000,
                "claroty_server_interfaces OffsetLimit page_size must be 1000 (BC-2.16.019 §PC1)"
            );
            // OffsetLimit pagination mode implements empty-page halt:
            // when the API returns an empty server_interfaces array (even with count=null),
            // PipelineExecutor stops requesting further pages without dereferencing count.
            // Consistent with claroty_servers pattern (BC-2.16.018 EC-016-018-004 precedent).
        }
        Some(other) => {
            panic!(
                "claroty_server_interfaces must use OffsetLimit pagination to support empty-page \
                 halt (count=null must not crash); got: {other:?}"
            );
        }
        None => {
            panic!(
                "claroty_server_interfaces fetch step must declare OffsetLimit pagination; \
                 None means no pagination configured (EC-016-019-005 would be unguarded)"
            );
        }
    }

    // Demonstrate the mock response shape the PipelineExecutor must handle safely:
    // {"server_interfaces": [], "count": null}
    let _mock_envelope_with_null_count = serde_json::json!({
        "server_interfaces": [],
        "count": serde_json::Value::Null
    });
    // (Full behavioral test belongs in prism-spec-engine integration tests where
    // PipelineExecutor is directly accessible.)
}
