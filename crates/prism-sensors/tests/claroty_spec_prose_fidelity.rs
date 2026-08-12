//! Spec-prose fidelity tests for `claroty.sensor.toml` audit_logs table comments.
//!
//! These tests enforce the TOML comment corrections defined by
//! S-DEMO-CLAROTY-SPEC-PROSE-FIX-001 (wave-5-e-demo-fidelity).
//!
//! Red Gate layout:
//!
//! | Test | Red Gate? | Reason for initial state |
//! |------|-----------|--------------------------|
//! | `test_BC_2_16_013_AC001_audit_logs_no_stale_dtu_gap_comments` | GREEN pre-impl | Stale comments already removed by the `docs(S-DEMO-CLAROTY-AUDIT-DTU-001)` commit (9e4e17bf); AC-001 is now a no-regression guard verifying they stay absent. |
//! | `test_BC_2_16_013_AC002_audit_logs_gap_cl_006_closed_comment_present` | RED pre-impl | `"Gap-CL-006 CLOSED"` comment line not yet written; this is the genuine Red Gate assertion. |
//! | `test_BC_2_16_013_AC004_audit_logs_functional_fields_unchanged` | GREEN pre-impl | No-regression guard — asserts functional TOML content is intact before and after. |
//!
//! Traces to: BC-2.16.013 §Postconditions §1 (audit_logs clause).
//!
//! Story: S-DEMO-CLAROTY-SPEC-PROSE-FIX-001
#![allow(clippy::expect_used, clippy::unwrap_used)]

use prism_spec_engine::spec_parser::SpecLoader;

// ---------------------------------------------------------------------------
// Helper: load and scope the audit_logs block from the TOML file.
// ---------------------------------------------------------------------------

/// Extract the text content of the `audit_logs` table block from `claroty.sensor.toml`.
///
/// Strategy: scan from the `[[tables]]` line that immediately precedes
/// `table_name = "audit_logs"` through to the next `[[tables]]` separator (exclusive)
/// or end-of-file, whichever comes first.
///
/// This scoping prevents false-positive or false-negative matches from lines in
/// the `alerts` or `devices` table blocks.
fn audit_logs_block() -> String {
    let toml_path = concat!(env!("CARGO_MANIFEST_DIR"), "/specs/claroty.sensor.toml");
    let full_content = std::fs::read_to_string(toml_path)
        .unwrap_or_else(|e| panic!("Failed to read claroty.sensor.toml: {e}"));

    // Find the `[[tables]]` line that owns `table_name = "audit_logs"`.
    // We walk line-by-line and locate the boundary of the block.
    let lines: Vec<&str> = full_content.lines().collect();

    // Find the line index of `table_name = "audit_logs"`
    let audit_logs_name_idx = lines
        .iter()
        .position(|l| l.trim() == r#"table_name = "audit_logs""#)
        .expect("claroty.sensor.toml must contain 'table_name = \"audit_logs\"'");

    // Walk backwards from that index to find the opening `[[tables]]` line.
    let block_start = (0..=audit_logs_name_idx)
        .rev()
        .find(|&i| lines[i].trim() == "[[tables]]")
        .expect("audit_logs table must be preceded by a [[tables]] header");

    // Walk forwards from `block_start + 1` to find the next `[[tables]]` or end-of-file.
    let block_end = lines
        .iter()
        .enumerate()
        .skip(block_start + 1)
        .find(|(_, l)| l.trim() == "[[tables]]")
        .map(|(i, _)| i)
        .unwrap_or(lines.len());

    lines[block_start..block_end].join("\n")
}

// ---------------------------------------------------------------------------
// AC-001 — No stale "DTU gap" comments in the audit_logs block
//
// GREEN-BY-DESIGN (pre-implementation): the stale "DTU gap" / "no route" /
// "404 until DTU route lands" comments were removed by the earlier
// `docs(S-DEMO-CLAROTY-AUDIT-DTU-001)` commit (9e4e17bf) — before this story
// was actioned. This test is therefore a no-regression guard: it asserts those
// stale strings remain absent after the implementer touches the comment lines.
//
// If any stale comment string re-appears in the audit_logs block (e.g., from an
// incorrect revert or incorrect patch), this test becomes a red gate.
//
// Traces to: BC-2.16.013 §Postconditions §1 — audit_logs clause.
// ---------------------------------------------------------------------------

/// GREEN-BY-DESIGN (no-regression guard): audit_logs block must not contain
/// `"DTU gap"`, `"no /api/v1/audit_log/get route"`, or `"404 until DTU route lands"`.
///
/// These comment substrings described the pre-S-DEMO-CLAROTY-AUDIT-DTU-001 state
/// of the Claroty audit_log route gap. After Gap-CL-006 was closed by
/// S-DEMO-CLAROTY-AUDIT-DTU-001, these comments became stale misinformation
/// (F-P2-DEFER-001). They were removed in the `docs(S-DEMO-CLAROTY-AUDIT-DTU-001)`
/// commit and must remain absent.
///
/// Story: S-DEMO-CLAROTY-SPEC-PROSE-FIX-001 AC-001
/// BC: BC-2.16.013 §Postconditions §1
#[test]
fn test_BC_2_16_013_AC001_audit_logs_no_stale_dtu_gap_comments() {
    let block = audit_logs_block();

    assert!(
        !block.contains("DTU gap"),
        "audit_logs block must not contain stale 'DTU gap' comment; found it in:\n{block}"
    );
    assert!(
        !block.contains("no /api/v1/audit_log/get route"),
        "audit_logs block must not contain stale 'no /api/v1/audit_log/get route' comment; found it in:\n{block}"
    );
    assert!(
        !block.contains("404 until DTU route lands"),
        "audit_logs block must not contain stale '404 until DTU route lands' comment; found it in:\n{block}"
    );
}

// ---------------------------------------------------------------------------
// AC-002 — Gap-CL-006 closure comment MUST be present in the audit_logs block
//
// RED gate (pre-implementation): the `# Gap-CL-006 CLOSED by S-DEMO-CLAROTY-AUDIT-DTU-001.`
// comment line has not yet been written into the audit_logs block.
// This test FAILS before the implementer adds the closure comment.
// It PASSES only after the implementer adds a line containing both
// "Gap-CL-006 CLOSED" and "S-DEMO-CLAROTY-AUDIT-DTU-001" to the block.
//
// Traces to: BC-2.16.013 §Postconditions §1 — prose accurately reflects
// closed gap (Gap-CL-006 CLOSED by S-DEMO-CLAROTY-AUDIT-DTU-001).
// ---------------------------------------------------------------------------

/// RED gate: audit_logs block must contain a comment line with `"Gap-CL-006 CLOSED"`.
///
/// S-DEMO-CLAROTY-SPEC-PROSE-FIX-001 §Change 1 requires replacing all stale "DTU gap"
/// comments with:
///
///   # Gap-CL-006 CLOSED by S-DEMO-CLAROTY-AUDIT-DTU-001.
///   # POST /api/v1/audit_log/get route registered in prism-dtu-claroty.
///
/// Until the implementer adds that comment, this assertion fails.
///
/// Story: S-DEMO-CLAROTY-SPEC-PROSE-FIX-001 AC-002
/// BC: BC-2.16.013 §Postconditions §1
#[test]
fn test_BC_2_16_013_AC002_audit_logs_gap_cl_006_closed_comment_present() {
    let block = audit_logs_block();

    // Both "Gap-CL-006 CLOSED" and "S-DEMO-CLAROTY-AUDIT-DTU-001" must appear on the
    // same comment line (e.g. `# Gap-CL-006 CLOSED by S-DEMO-CLAROTY-AUDIT-DTU-001.`).
    // Two independent contains() checks would false-pass if the strings landed on
    // unrelated lines — co-occurrence on one line is the correct assertion.
    assert!(
        block.lines().any(|line| line.contains("Gap-CL-006 CLOSED")
            && line.contains("S-DEMO-CLAROTY-AUDIT-DTU-001")),
        "audit_logs block must contain a single comment line with both \
         'Gap-CL-006 CLOSED' and 'S-DEMO-CLAROTY-AUDIT-DTU-001'; not found in:\n{block}"
    );
}

// ---------------------------------------------------------------------------
// AC-004 — No functional TOML content changed (no-regression guard)
//
// GREEN-BY-DESIGN (both before and after implementation): the implementer only
// modifies comment lines. The functional TOML fields (path_template, method,
// response_path, columns) must remain identical before and after this story.
//
// This test parses the full claroty.sensor.toml via the canonical SpecLoader::parse
// API, finds the audit_logs TableSpec, and asserts every functional field matches
// the expected values grounded from BC-2.16.013 §Postconditions §1 and
// the Gap-CL-002 fix (path corrected to /api/v1/audit_log/get in 72baf413).
//
// Traces to: BC-2.16.013 §Postconditions §1 — TOML functional content
// already correct per earlier Gap-CL-002 fix; only comment lines are in scope.
// ---------------------------------------------------------------------------

/// RED gate (items 3 + 4 of CLAROTY-LIVE-API-FIDELITY): audit_logs functional fields
/// must match the real xDome API.
///
/// Parses `claroty.sensor.toml` via `SpecLoader::parse` (canonical TOML loader)
/// and asserts:
/// - `path_template` == `"/api/v1/audit_log/get"` (NO trailing slash — OpenAPI declares
///   the path without one; alerts/devices use trailing slash by API convention but
///   audit_log/get does not).
/// - `method` == `"POST"` (POST-for-read pattern per BC-2.16.013 §Postconditions §1)
/// - `response_path` == `"$.audit_log"` (per DTU GetAuditLogResponse struct)
/// - Expected columns: `id`, `action`, `category`, `details`, `timestamp`,
///   `user_display_name`, `username`, `note` (8 columns total).
///   `actor` and `resource` MUST NOT be present — they do not exist in the xDome API
///   (LIVE-DRIFT-003, confirmed against xDome OpenAPI §GetAuditLogResponse example).
///
/// Before fix: fails because spec has trailing slash and old column names (actor/resource).
/// After fix: passes with corrected path and real API field names.
///
/// Story: CLAROTY-LIVE-API-FIDELITY items 3 and 4
/// BC: BC-2.16.013 §Postconditions §1
#[test]
fn test_BC_2_16_013_AC004_audit_logs_functional_fields_unchanged() {
    let toml_path = concat!(env!("CARGO_MANIFEST_DIR"), "/specs/claroty.sensor.toml");
    let content = std::fs::read_to_string(toml_path)
        .unwrap_or_else(|e| panic!("Failed to read claroty.sensor.toml: {e}"));

    let spec = SpecLoader::parse(&content)
        .unwrap_or_else(|e| panic!("SpecLoader::parse failed for claroty.sensor.toml: {e:?}"));

    // Locate the audit_logs TableSpec.
    let audit_logs = spec
        .tables
        .iter()
        .find(|t| t.table_name == "audit_logs")
        .expect("claroty.sensor.toml must declare a table named 'audit_logs'");

    // --- Step-level assertions ---
    assert_eq!(
        audit_logs.steps.len(),
        1,
        "audit_logs table must have exactly 1 fetch step"
    );
    let step = &audit_logs.steps[0];

    assert_eq!(
        step.method, "POST",
        "audit_logs fetch step must use POST (POST-for-read pattern per BC-2.16.013 §PC §1)"
    );

    // Item 3 (LIVE-DRIFT-002): path must NOT have trailing slash.
    // OpenAPI declares /api/v1/audit_log/get (without slash).
    // alerts and devices use trailing slash by xDome API convention; audit_log/get does not.
    assert_eq!(
        step.path_template, "/api/v1/audit_log/get",
        "audit_logs fetch step path_template must be '/api/v1/audit_log/get' (no trailing slash). \
         OpenAPI declares the audit_log path without trailing slash while alerts/devices paths \
         include one — the convention is per-path (LIVE-DRIFT-002)"
    );

    assert_eq!(
        step.response_path, "$.audit_log",
        "audit_logs fetch step response_path must be '$.audit_log' \
         (DTU GetAuditLogResponse struct field name)"
    );

    // --- Column-set assertions (Item 4 — LIVE-DRIFT-003) ---
    // Real xDome audit_log fields: id, action, category, details, timestamp,
    // user_display_name, username, note.
    // actor and resource DO NOT EXIST in the xDome API and must not appear.
    let column_names: Vec<&str> = audit_logs.columns.iter().map(|c| c.name.as_str()).collect();

    let expected_cols = [
        "id",
        "action",
        "category",
        "details",
        "timestamp",
        "user_display_name",
        "username",
        "note",
    ];
    for expected_col in &expected_cols {
        assert!(
            column_names.contains(expected_col),
            "audit_logs columns must include '{expected_col}' (real xDome API field); \
             got: {column_names:?}"
        );
    }

    // Nonexistent fields MUST NOT appear — they silently produce empty values at runtime.
    for ghost_col in &["actor", "resource"] {
        assert!(
            !column_names.contains(ghost_col),
            "audit_logs columns must NOT include '{ghost_col}' — this field does not exist \
             in the xDome API and silently returns nothing (LIVE-DRIFT-003); \
             got columns: {column_names:?}"
        );
    }

    assert_eq!(
        column_names.len(),
        8,
        "audit_logs must have exactly 8 columns \
         (id, action, category, details, timestamp, user_display_name, username, note); \
         got {}: {column_names:?}",
        column_names.len()
    );
}

// ---------------------------------------------------------------------------
// Tier 2 tests — array columns, scalar columns, alert expansion
// ---------------------------------------------------------------------------

/// Column-mapping layer assertion (Tier 2): `ip_list` column with `source_path = "$.ip_list[*]"`
/// serializes an array of IP strings to a compact JSON-list string at the `ColumnMapper::map_record`
/// layer (the intermediate pre-Arrow mapping step).
///
/// This test covers the `ColumnMapper::map_record` path (prism-spec-engine).  The production
/// query path uses `build_column_array` in prism-bin, which is covered by:
/// - `test_build_column_array_claroty_ip_list_string_elements_serialize_to_json_list_string`
/// - `test_build_column_array_claroty_vlan_list_integer_elements_stringify_to_json_list_string`
/// (both in `crates/prism-bin/src/spec_driven_adapter.rs` — MEDIUM-6 fix).
///
/// Together these three tests form the full coverage chain for ENRICH-1 array columns.
#[test]
fn test_claroty_tier2_ip_list_array_column_serializes_to_json_list_string() {
    use prism_spec_engine::column_mapping::ColumnMapper;

    let toml_path = concat!(env!("CARGO_MANIFEST_DIR"), "/specs/claroty.sensor.toml");
    let content = std::fs::read_to_string(toml_path)
        .unwrap_or_else(|e| panic!("Failed to read claroty.sensor.toml: {e}"));

    let spec =
        SpecLoader::parse(&content).unwrap_or_else(|e| panic!("SpecLoader::parse failed: {e:?}"));

    let devices = spec
        .tables
        .iter()
        .find(|t| t.table_name == "devices")
        .expect("devices table must exist");

    // Verify ip_list column is declared with source_path
    let ip_col = devices
        .columns
        .iter()
        .find(|c| c.name == "ip_list")
        .expect("devices table must declare ip_list column (Tier 2)");
    assert_eq!(
        ip_col.source_path.as_deref(),
        Some("$.ip_list[*]"),
        "ip_list column must use source_path = '$.ip_list[*]' for ENRICH-1 array extraction"
    );

    // Simulate a raw JSON record from the xDome API containing ip_list
    let raw = serde_json::json!({
        "uid": "uid-wire-test",
        "ip_list": ["10.0.1.1", "10.0.1.2", "10.0.1.3"]
    });

    let result = ColumnMapper::map_record(&raw, devices)
        .expect("ColumnMapper::map_record must succeed for devices with ip_list");

    // ip_list has no ocsf_field → goes to raw_extensions as a JSON-list string
    let ip_list_val = result
        .raw_extensions
        .get("ip_list")
        .expect("ip_list must appear in raw_extensions (no ocsf_field declared)");

    // Wire-shape assertion: the array must serialize to a compact JSON-list string
    // This is the exact string the DataFusion engine sees as a string column value.
    assert_eq!(
        ip_list_val,
        &serde_json::Value::String("[\"10.0.1.1\",\"10.0.1.2\",\"10.0.1.3\"]".to_string()),
        "ip_list with source_path='$.ip_list[*]' must serialize array to compact JSON-list \
         string at wire level; got: {ip_list_val:?} (ENRICH-1 wildcard serialization)"
    );
}

/// Tier 2 structural test: devices table must declare all Tier 2 array columns
/// with correct source_path values (ENRICH-1 pattern).
#[test]
fn test_claroty_tier2_device_array_columns_declared_with_source_paths() {
    let toml_path = concat!(env!("CARGO_MANIFEST_DIR"), "/specs/claroty.sensor.toml");
    let content = std::fs::read_to_string(toml_path)
        .unwrap_or_else(|e| panic!("Failed to read claroty.sensor.toml: {e}"));

    let spec =
        SpecLoader::parse(&content).unwrap_or_else(|e| panic!("SpecLoader::parse failed: {e:?}"));

    let devices = spec
        .tables
        .iter()
        .find(|t| t.table_name == "devices")
        .expect("devices table must exist");

    // Each array column → expected source_path
    let array_cols = [
        ("ip_list", "$.ip_list[*]"),
        ("mac_list", "$.mac_list[*]"),
        ("network_list", "$.network_list[*]"),
        ("vlan_list", "$.vlan_list[*]"),
    ];

    for (col_name, expected_path) in &array_cols {
        let col = devices
            .columns
            .iter()
            .find(|c| c.name == *col_name)
            .unwrap_or_else(|| {
                panic!("devices table must declare '{col_name}' column (Tier 2 array column)")
            });
        assert_eq!(
            col.source_path.as_deref(),
            Some(*expected_path),
            "devices.{col_name} must have source_path = '{expected_path}'"
        );
    }
}

/// Tier 2 structural test: devices table must declare all Tier 2 scalar columns.
#[test]
fn test_claroty_tier2_device_scalar_columns_declared() {
    let toml_path = concat!(env!("CARGO_MANIFEST_DIR"), "/specs/claroty.sensor.toml");
    let content = std::fs::read_to_string(toml_path)
        .unwrap_or_else(|e| panic!("Failed to read claroty.sensor.toml: {e}"));

    let spec =
        SpecLoader::parse(&content).unwrap_or_else(|e| panic!("SpecLoader::parse failed: {e:?}"));

    let devices = spec
        .tables
        .iter()
        .find(|t| t.table_name == "devices")
        .expect("devices table must exist");

    let col_names: Vec<&str> = devices.columns.iter().map(|c| c.name.as_str()).collect();

    let expected_scalar_cols = [
        "purdue_level",
        "site_name",
        "device_subcategory",
        "device_type_family",
        "criticality",
        "is_online",
        "device_name",
        "manufacturer",
        "model",
        "os_category",
    ];

    for col_name in &expected_scalar_cols {
        assert!(
            col_names.contains(col_name),
            "devices table must declare scalar column '{col_name}' (Tier 2); \
             got columns: {col_names:?}"
        );
    }
}

/// Tier 2 structural test: alerts table must declare `alert_class`, `ot_devices_count`,
/// and `alert_name` (all verified in xDome Alert fields_enum, OpenAPI 2026-06-20).
#[test]
fn test_claroty_tier2_alert_columns_declared() {
    let toml_path = concat!(env!("CARGO_MANIFEST_DIR"), "/specs/claroty.sensor.toml");
    let content = std::fs::read_to_string(toml_path)
        .unwrap_or_else(|e| panic!("Failed to read claroty.sensor.toml: {e}"));

    let spec =
        SpecLoader::parse(&content).unwrap_or_else(|e| panic!("SpecLoader::parse failed: {e:?}"));

    let alerts = spec
        .tables
        .iter()
        .find(|t| t.table_name == "alerts")
        .expect("alerts table must exist");

    let col_names: Vec<&str> = alerts.columns.iter().map(|c| c.name.as_str()).collect();

    for col_name in &["alert_class", "ot_devices_count", "alert_name"] {
        assert!(
            col_names.contains(col_name),
            "alerts table must declare '{col_name}' column (Tier 2, xDome OpenAPI verified); \
             got: {col_names:?}"
        );
    }
}

/// Tier 2 structural test: devices body_template must include all declared column names
/// so the xDome API returns them in each response (GetDevicesParameters.fields is REQUIRED).
#[test]
fn test_claroty_tier2_devices_body_template_covers_all_declared_columns() {
    let toml_path = concat!(env!("CARGO_MANIFEST_DIR"), "/specs/claroty.sensor.toml");
    let content = std::fs::read_to_string(toml_path)
        .unwrap_or_else(|e| panic!("Failed to read claroty.sensor.toml: {e}"));

    let spec =
        SpecLoader::parse(&content).unwrap_or_else(|e| panic!("SpecLoader::parse failed: {e:?}"));

    let devices = spec
        .tables
        .iter()
        .find(|t| t.table_name == "devices")
        .expect("devices table must exist");

    let step = devices
        .steps
        .first()
        .expect("devices table must have a fetch step");

    let body_tmpl = step
        .body_template
        .as_deref()
        .expect("devices fetch step must have a body_template");

    // All declared column names (including array columns) must appear in the fields array
    // so the xDome API returns them. Array columns use source_path but the API field name
    // (e.g., "ip_list") must still be requested via the fields projection.
    for col in &devices.columns {
        assert!(
            body_tmpl.contains(&format!("\"{}\"", col.name)),
            "devices body_template must include '\"{}\"' so the xDome API returns this field; \
             body_template = '{body_tmpl}'",
            col.name
        );
    }
}

/// Tier 2 structural test: alerts body_template must include all declared column names.
#[test]
fn test_claroty_tier2_alerts_body_template_covers_all_declared_columns() {
    let toml_path = concat!(env!("CARGO_MANIFEST_DIR"), "/specs/claroty.sensor.toml");
    let content = std::fs::read_to_string(toml_path)
        .unwrap_or_else(|e| panic!("Failed to read claroty.sensor.toml: {e}"));

    let spec =
        SpecLoader::parse(&content).unwrap_or_else(|e| panic!("SpecLoader::parse failed: {e:?}"));

    let alerts = spec
        .tables
        .iter()
        .find(|t| t.table_name == "alerts")
        .expect("alerts table must exist");

    let step = alerts
        .steps
        .first()
        .expect("alerts table must have a fetch step");

    let body_tmpl = step
        .body_template
        .as_deref()
        .expect("alerts fetch step must have a body_template");

    for col in &alerts.columns {
        assert!(
            body_tmpl.contains(&format!("\"{}\"", col.name)),
            "alerts body_template must include '\"{}\"' so the xDome API returns this field; \
             body_template = '{body_tmpl}'",
            col.name
        );
    }
}

// ---------------------------------------------------------------------------
// Tier 3 tests — device_alert_relations table
// ---------------------------------------------------------------------------

/// Tier 3 structural test: `device_alert_relations` table must be declared in
/// `claroty.sensor.toml` with the correct step configuration.
///
/// Asserts:
/// - `table_name = "device_alert_relations"` present
/// - `method = "POST"` (POST-for-read pattern)
/// - `path_template = "/api/v1/device_alert_relations/"` (trailing slash, xDome convention)
/// - `response_path = "$.devices_alerts"` (NOT "$.device_alert_relations" — envelope key
///   is `devices_alerts` per GetDeviceAlertsResponse OpenAPI schema)
/// - All 10 declared column names present (verified in AlertedDevicesPairs__fields_enum)
#[test]
fn test_claroty_tier3_device_alert_relations_table_declared() {
    let toml_path = concat!(env!("CARGO_MANIFEST_DIR"), "/specs/claroty.sensor.toml");
    let content = std::fs::read_to_string(toml_path)
        .unwrap_or_else(|e| panic!("Failed to read claroty.sensor.toml: {e}"));

    let spec =
        SpecLoader::parse(&content).unwrap_or_else(|e| panic!("SpecLoader::parse failed: {e:?}"));

    let dar = spec
        .tables
        .iter()
        .find(|t| t.table_name == "device_alert_relations")
        .expect("claroty.sensor.toml must declare a table named 'device_alert_relations' (Tier 3)");

    // Step assertions
    assert_eq!(
        dar.steps.len(),
        1,
        "device_alert_relations table must have exactly 1 fetch step"
    );
    let step = &dar.steps[0];

    assert_eq!(
        step.method, "POST",
        "device_alert_relations fetch step must use POST (POST-for-read pattern)"
    );
    assert_eq!(
        step.path_template, "/api/v1/device_alert_relations/",
        "device_alert_relations path_template must be '/api/v1/device_alert_relations/' \
         (trailing slash per xDome API convention; NormalizePathLayer strips on inbound)"
    );

    // Wire-shape assertion: response_path MUST be "$.devices_alerts", NOT "$.device_alert_relations".
    // The xDome GetDeviceAlertsResponse envelope key is `devices_alerts`.
    assert_eq!(
        step.response_path, "$.devices_alerts",
        "device_alert_relations response_path must be '$.devices_alerts' \
         (GetDeviceAlertsResponse envelope key per OpenAPI schema); \
         NOT '$.device_alert_relations' — SAP-2 wire-shape check"
    );

    // Column set assertions (all 10 verified in AlertedDevicesPairs__fields_enum)
    let col_names: Vec<&str> = dar.columns.iter().map(|c| c.name.as_str()).collect();

    let expected_cols = [
        "device_uid",
        "alert_id",
        "device_alert_detected_time",
        "device_risk_score",
        "network_signature_severity",
        "network_signature_confidence",
        "malicious_ip_severity",
        "alert_note",
        "external_ip",
        "device_alert_status",
    ];
    for col_name in &expected_cols {
        assert!(
            col_names.contains(col_name),
            "device_alert_relations table must declare column '{col_name}' \
             (verified in 92-value AlertedDevicesPairs__fields_enum, xDome OpenAPI 2026-06-20); \
             got: {col_names:?}"
        );
    }

    assert_eq!(
        col_names.len(),
        10,
        "device_alert_relations must have exactly 10 columns; got {}: {col_names:?}",
        col_names.len()
    );
}

/// Tier 3 body_template coverage: `device_alert_relations` body_template must include
/// all 10 declared column names so the xDome API returns them in each response
/// (GetDeviceAlertsParameters.fields is REQUIRED, minItems: 1).
#[test]
fn test_claroty_tier3_device_alert_relations_body_template_covers_all_declared_columns() {
    let toml_path = concat!(env!("CARGO_MANIFEST_DIR"), "/specs/claroty.sensor.toml");
    let content = std::fs::read_to_string(toml_path)
        .unwrap_or_else(|e| panic!("Failed to read claroty.sensor.toml: {e}"));

    let spec =
        SpecLoader::parse(&content).unwrap_or_else(|e| panic!("SpecLoader::parse failed: {e:?}"));

    let dar = spec
        .tables
        .iter()
        .find(|t| t.table_name == "device_alert_relations")
        .expect("device_alert_relations table must exist");

    let step = dar.steps.first().expect("must have a fetch step");

    let body_tmpl = step
        .body_template
        .as_deref()
        .expect("device_alert_relations fetch step must have a body_template");

    for col in &dar.columns {
        assert!(
            body_tmpl.contains(&format!("\"{}\"", col.name)),
            "device_alert_relations body_template must include '\"{}\"' so the xDome API \
             returns this field (GetDeviceAlertsParameters.fields REQUIRED, minItems: 1); \
             body_template = '{body_tmpl}'",
            col.name
        );
    }
}

/// RED gate (item 8 of CLAROTY-LIVE-API-FIDELITY): all Claroty tables (alerts,
/// audit_logs, devices, device_alert_relations) must use page_size = 1000.
///
/// Live measurement: audit_log sweep burns 200 requests/cycle with page_size=100 against
/// a 2000/min API ceiling. page_size=1000 provides 10x headroom. 5000 is excluded because
/// a full devices page would be ~3.5MB held in memory without partial-page resume.
///
/// Before fix: fails because spec has page_size = 100.
/// After fix: passes with page_size = 1000 on all four tables.
///
/// Story: CLAROTY-LIVE-API-FIDELITY item 8
#[test]
fn test_claroty_live_all_tables_page_size_1000() {
    let toml_path = concat!(env!("CARGO_MANIFEST_DIR"), "/specs/claroty.sensor.toml");
    let content = std::fs::read_to_string(toml_path)
        .unwrap_or_else(|e| panic!("Failed to read claroty.sensor.toml: {e}"));

    let spec = SpecLoader::parse(&content)
        .unwrap_or_else(|e| panic!("SpecLoader::parse failed for claroty.sensor.toml: {e:?}"));

    use prism_spec_engine::spec_parser::PaginationConfig;

    for table in &spec.tables {
        for step in &table.steps {
            match &step.pagination {
                Some(PaginationConfig::OffsetLimit { page_size }) => {
                    assert_eq!(
                        *page_size, 1000,
                        "table '{}' step '{}' must use page_size = 1000 (live-API bandwidth \
                         budget; 10x headroom vs 100/req baseline at 2000/min ceiling); got {}",
                        table.table_name, step.name, page_size
                    );
                }
                Some(_) => {} // non-offset-limit pagination — not Claroty pattern; skip
                None => {}    // no pagination declared; skip
            }
        }
    }
}
