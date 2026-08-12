// Legitimately sensor-named: this IS the Claroty DTU parity test. Exempt from
// tests/external/no-hardcoded-sensors/ compile-fail gate per ADR-023 §DTU-EXEMPT.
//! Fix-burst regression tests — review-2026-06-10 cascade pass-2, finding P2-01.
//!
//! CS-06-style shape-parity test class for Claroty: generator flat-key set vs
//! claroty.sensor.toml columns vs static-fixture keys. The Armis seeded path
//! failed SAP-2 because no test pinned this contract per sensor — this file is
//! the Claroty instance of that missing test class. The serving extraction is
//! exact-name flat `raw.get(col)` (column_mapping.rs): a TOML column with no
//! matching record key silently normalizes to NULL.
//!
//! Coverage:
//! - `devices` / `alerts` tables: BOTH paths (generator + static fixtures) —
//!   seeded clones serve generated records for these surfaces.
//! - `audit_logs` table: static fixture only — the generator emits no
//!   audit-log surface and routes/audit_log.rs has no seeded branch; seeded
//!   clones serve the static fixture for this table.

#![cfg(feature = "fixture-gen")]
#![allow(clippy::unwrap_used, clippy::expect_used, non_snake_case)]

use std::collections::BTreeSet;

use prism_dtu_claroty::generate;
use prism_dtu_common::{Archetype, GenOpts, OrgId};

// ---------------------------------------------------------------------------
// Spec-parse-driven column derivation (TD-VSDD-097 dimension-1 structural fix)
//
// `claroty.sensor.toml` is the single source of truth for column lists. All
// column sets are derived at compile time via `include_str!` + `toml::from_str`
// rather than hand-maintained parallel arrays.
//
// Root cause of HIGH-2 (TD-VSDD-097 dimension-1 sibling-pair failure): PR #236
// added 10 new columns across devices/alerts/device_alert_relations tables.
// `AUDIT_LOG_COLUMNS` was updated (5->8) but its two same-file same-purpose twins
// -- `DEVICE_COLUMNS` (stayed at 6 of 20) and `ALERT_COLUMNS` (stayed at 8 of 11)
// -- were not. The spec-parse-driven approach makes every future TOML column
// addition automatically visible as a guard failure if the generator or fixture
// is not updated, eliminating this recurrence class for Claroty.
// TD-VSDD-097 dimension-1 (sibling-pair sweep): `review_2026_06_10_armis_parity.rs`
// is the same-pattern twin for Armis. It uses hand-maintained column arrays rather
// than spec-parse-driven derivation — an independent sibling story should apply the
// same spec-parse-driven approach to that file to close the same drift risk there.
//
// Array-column handling: columns with `source_path = "$.name[*]"` (ENRICH-1
// wildcard extraction) produce JSON arrays rather than scalars.
// `flat_scalar_keys` (which filters `!v.is_array()`) will not include them, so
// they are extracted as root keys (e.g. `"$.ip_list[*]"` -> `"ip_list"`) and
// verified separately via `assert_array_keys_present`.
// ---------------------------------------------------------------------------

#[derive(serde::Deserialize)]
struct SensorSpec {
    tables: Vec<TableSpec>,
}

#[derive(serde::Deserialize)]
struct TableSpec {
    table_name: String,
    columns: Vec<ColumnSpec>,
}

#[derive(serde::Deserialize)]
struct ColumnSpec {
    name: String,
    /// Present on ENRICH-1 wildcard columns: `"$.field_name[*]"`.
    #[serde(default)]
    source_path: Option<String>,
}

/// Parse `claroty.sensor.toml` embedded at compile time.
///
/// `include_str!` resolves `../../prism-sensors/specs/claroty.sensor.toml`
/// relative to this file's location (`crates/prism-dtu-claroty/tests/`).
/// Any change to the TOML invalidates and recompiles this test binary, so the
/// guard is always current without manual const updates.
fn parse_claroty_spec() -> SensorSpec {
    toml::from_str(include_str!(
        "../../prism-sensors/specs/claroty.sensor.toml"
    ))
    .expect("claroty.sensor.toml must be valid TOML with [[tables]] + [[tables.columns]]")
}

/// Extract the JSON root key from a JSONPath `source_path` expression.
///
/// `"$.ip_list[*]"` -> `"ip_list"`, `"$.vlan_list[*]"` -> `"vlan_list"`.
fn source_path_root_key(source_path: &str) -> String {
    let after_prefix = source_path.strip_prefix("$.").unwrap_or(source_path);
    match after_prefix.find('[') {
        Some(pos) => after_prefix[..pos].to_string(),
        None => after_prefix.to_string(),
    }
}

/// Returns `(scalar_columns, array_root_keys)` for the named table.
///
/// - `scalar_columns`: column names that map to flat scalar JSON values.
///   Checked via `flat_scalar_keys` + `assert_columns_covered`.
/// - `array_root_keys`: the JSON top-level key for each `source_path`-bearing
///   column (e.g. `ip_list` for `source_path = "$.ip_list[*]"`).
///   Checked via `assert_array_keys_present`.
fn columns_for_table(spec: &SensorSpec, table_name: &str) -> (Vec<String>, Vec<String>) {
    let table = spec
        .tables
        .iter()
        .find(|t| t.table_name == table_name)
        .unwrap_or_else(|| panic!("table '{table_name}' not found in claroty.sensor.toml"));
    let mut scalar = Vec::new();
    let mut array = Vec::new();
    for col in &table.columns {
        match &col.source_path {
            None => scalar.push(col.name.clone()),
            Some(sp) => array.push(source_path_root_key(sp)),
        }
    }
    (scalar, array)
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Canonical test org: bytes [0xde, 0xad, 0xbe, 0xef, ...] -> org_slug = "deadbeef".
fn deadbeef_org() -> OrgId {
    OrgId([
        0xde, 0xad, 0xbe, 0xef, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00,
    ])
}

/// Generated records of a given `_surface` for the specified archetype, default opts.
///
/// F-CLARO-P2-MED-001 fix: parameterised so each archetype can be independently
/// guarded. Before this fix, only `CompromisedEndpoint` was ever tested.
fn generated_surface_for(archetype: Archetype, surface: &str) -> Vec<serde_json::Value> {
    let fs = generate(&deadbeef_org(), archetype, &GenOpts::default());
    fs.records
        .into_iter()
        .filter(|r| r.get("_surface").and_then(|v| v.as_str()) == Some(surface))
        .collect()
}

/// Generated records of a given `_surface` for CompromisedEndpoint, default opts.
///
/// Retained for tests that do not require multi-archetype iteration (e.g. the
/// type-shape test which only needs one representative archetype). New coverage
/// tests should use `generated_surface_for` directly.
fn generated_surface(surface: &str) -> Vec<serde_json::Value> {
    generated_surface_for(Archetype::CompromisedEndpoint, surface)
}

fn static_fixture(raw: &str, name: &str) -> Vec<serde_json::Value> {
    serde_json::from_str(raw).unwrap_or_else(|e| panic!("{name} must be a JSON array: {e}"))
}

/// Flat scalar key set: scalar-valued keys, excluding `_`-prefixed internal
/// tags (`_surface`). Nested arrays/objects (ip_list, labels, ...) are invisible
/// to the flat `r.get(col)` extraction (CS-06 precedent).
fn flat_scalar_keys(record: &serde_json::Value) -> BTreeSet<String> {
    record
        .as_object()
        .expect("record must be a JSON object")
        .iter()
        .filter(|(k, v)| !k.starts_with('_') && !v.is_object() && !v.is_array())
        .map(|(k, _)| k.clone())
        .collect()
}

/// Flat scalar keys of a record slice, asserting intra-path uniformity.
fn uniform_flat_keys(records: &[serde_json::Value], path_name: &str) -> BTreeSet<String> {
    assert!(!records.is_empty(), "{path_name}: no records");
    let first = flat_scalar_keys(&records[0]);
    for (i, rec) in records.iter().enumerate() {
        assert_eq!(
            flat_scalar_keys(rec),
            first,
            "{path_name}: record[{i}] flat key set drifts from record[0]"
        );
    }
    first
}

fn assert_columns_covered(keys: &BTreeSet<String>, columns: &[&str], path_name: &str) {
    for col in columns {
        assert!(
            keys.contains(*col),
            "{path_name} missing claroty.sensor.toml column '{col}' (P2-01 SAP-2: \
             flat raw.get extraction silently normalizes it to NULL)"
        );
    }
}

/// Assert every named root key is present in every record's top-level JSON
/// object, regardless of value type.
///
/// Used for `source_path`-bearing TOML columns (ENRICH-1 wildcard) that produce
/// JSON array values. `flat_scalar_keys` will not include them since it filters
/// `!v.is_array()`.
fn assert_array_keys_present(records: &[serde_json::Value], keys: &[&str], path_name: &str) {
    assert!(
        !records.is_empty(),
        "{path_name}: no records to check array keys"
    );
    for (i, record) in records.iter().enumerate() {
        let obj = record.as_object().expect("record must be a JSON object");
        for key in keys {
            assert!(
                obj.contains_key(*key),
                "{path_name} record[{i}] missing array column '{key}' \
                 (HIGH-2: source_path array column absent from generator/fixture)"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// P2-01 -- TOML column coverage per serving path
// ---------------------------------------------------------------------------

/// Generated device records cover every TOML `devices` column — for ALL
/// archetypes that emit device-surface records.
///
/// F-CLARO-P2-MED-001 fix: prior guard was archetype-blind (CompromisedEndpoint
/// only). This test now iterates every device-producing archetype so a new
/// construction site cannot silently miss a TOML column for a single archetype.
///
/// EXCLUDED archetypes (SAP-3 defense-in-depth):
/// - `SchemaDrift`: records[0] is deliberately sparse — a sentinel record carrying
///   `"_schema_drift": true` with minimal fields to exercise schema-drift detection.
///   It is intentionally non-representative of the production-data path and is
///   governed by BC-3.4.002 / BC-3.4.003 row 6. Including it would require the
///   guard to special-case a record that is designed to violate column coverage.
/// - `DormantTenant`: emits zero device records by BC-3.4.003 row 8 / EC-001.
///   No records to check.
#[test]
fn test_p2_01_claroty_generated_devices_cover_toml_columns() {
    let spec = parse_claroty_spec();
    let (scalar_cols, array_root_keys) = columns_for_table(&spec, "devices");
    let scalar_refs: Vec<&str> = scalar_cols.iter().map(String::as_str).collect();
    let array_refs: Vec<&str> = array_root_keys.iter().map(String::as_str).collect();

    // All archetypes that produce device-surface records.
    // HighChurn: includes tombstone records (i < 20) — these MUST carry all TOML
    // columns just like normal device records; the tombstone status is conveyed via
    // field values (`"status": "tombstone"`, `"retired": true`), not absent keys.
    for archetype in [
        Archetype::HealthyOtEnvironment,
        Archetype::CompromisedEndpoint,
        Archetype::AuthOutage,
        Archetype::LargeScale,
        Archetype::PaginationEdgeCases,
        Archetype::HighChurn,
    ] {
        let path_name = format!("generated devices ({archetype:?})");
        let records = generated_surface_for(archetype, "device");
        assert!(
            !records.is_empty(),
            "{path_name}: expected device records but got none"
        );
        let keys = uniform_flat_keys(&records, &path_name);
        assert_columns_covered(&keys, &scalar_refs, &path_name);
        assert_array_keys_present(&records, &array_refs, &path_name);
    }
}

/// Generated alert records cover every TOML `alerts` column.
#[test]
fn test_p2_01_claroty_generated_alerts_cover_toml_columns() {
    let spec = parse_claroty_spec();
    let (scalar_cols, array_root_keys) = columns_for_table(&spec, "alerts");
    let records = generated_surface("alert");
    let keys = uniform_flat_keys(&records, "generated alerts");
    let scalar_refs: Vec<&str> = scalar_cols.iter().map(String::as_str).collect();
    assert_columns_covered(&keys, &scalar_refs, "generated alerts");
    // alerts currently has no source_path columns; this call is a no-op that
    // keeps the pattern uniform and future-safe if array columns are added.
    let array_refs: Vec<&str> = array_root_keys.iter().map(String::as_str).collect();
    if !array_refs.is_empty() {
        assert_array_keys_present(&records, &array_refs, "generated alerts");
    }
}

/// Static fixture devices cover every TOML `devices` column.
#[test]
fn test_p2_01_claroty_static_devices_cover_toml_columns() {
    let spec = parse_claroty_spec();
    let (scalar_cols, array_root_keys) = columns_for_table(&spec, "devices");
    let records = static_fixture(include_str!("../fixtures/devices.json"), "devices.json");
    let keys = uniform_flat_keys(&records, "static devices");
    let scalar_refs: Vec<&str> = scalar_cols.iter().map(String::as_str).collect();
    assert_columns_covered(&keys, &scalar_refs, "static devices");
    let array_refs: Vec<&str> = array_root_keys.iter().map(String::as_str).collect();
    assert_array_keys_present(&records, &array_refs, "static devices");
}

/// Static fixture alerts cover every TOML `alerts` column.
#[test]
fn test_p2_01_claroty_static_alerts_cover_toml_columns() {
    let spec = parse_claroty_spec();
    let (scalar_cols, _array_root_keys) = columns_for_table(&spec, "alerts");
    let records = static_fixture(include_str!("../fixtures/alerts.json"), "alerts.json");
    let keys = uniform_flat_keys(&records, "static alerts");
    let scalar_refs: Vec<&str> = scalar_cols.iter().map(String::as_str).collect();
    assert_columns_covered(&keys, &scalar_refs, "static alerts");
}

/// Static fixture audit logs cover every TOML `audit_logs` column (the only
/// serving path for this table -- no generated audit-log surface exists).
#[test]
fn test_p2_01_claroty_static_audit_logs_cover_toml_columns() {
    let spec = parse_claroty_spec();
    let (scalar_cols, _array_root_keys) = columns_for_table(&spec, "audit_logs");
    let records = static_fixture(include_str!("../fixtures/audit-log.json"), "audit-log.json");
    let keys = uniform_flat_keys(&records, "static audit logs");
    let scalar_refs: Vec<&str> = scalar_cols.iter().map(String::as_str).collect();
    assert_columns_covered(&keys, &scalar_refs, "static audit logs");
}

/// Static fixture device-alert-relations cover every TOML `device_alert_relations`
/// column.
///
/// HIGH-2 fix: new test for the Tier 3 table added in PR #236.
/// This table has no generated path (no StageMask/seeded branch in the route
/// handler), so only static fixture coverage exists.
///
/// Note: nullable columns (network_signature_severity, malicious_ip_severity,
/// external_ip) appear as `null` in some records -- flat_scalar_keys includes null
/// (it passes !is_object() && !is_array()), so they are present in the key set.
#[test]
fn test_p2_01_claroty_static_device_alert_relations_cover_toml_columns() {
    let spec = parse_claroty_spec();
    let (scalar_cols, _array_root_keys) = columns_for_table(&spec, "device_alert_relations");
    let records = static_fixture(
        include_str!("../fixtures/device-alert-relations.json"),
        "device-alert-relations.json",
    );
    let keys = uniform_flat_keys(&records, "static device_alert_relations");
    let scalar_refs: Vec<&str> = scalar_cols.iter().map(String::as_str).collect();
    assert_columns_covered(&keys, &scalar_refs, "static device_alert_relations");
}

// ---------------------------------------------------------------------------
// P2-01 -- TOML column TYPES hold on the generated path
// ---------------------------------------------------------------------------

/// Generated records carry TOML-compatible value types for all columns.
///
/// Extended in PR #236 fix-burst to cover the 10 newly-added device and alert
/// fields (CRITICAL-1 generator fix + HIGH-2 type-guard extension):
/// - devices: `purdue_level`, `site_name`, `criticality`, `device_name`,
///   `manufacturer` (column_type = "string"), `is_online` (column_type = "boolean")
/// - alerts: `alert_class`, `alert_name` (column_type = "string"),
///   `ot_devices_count` (column_type = "integer")
#[test]
fn test_p2_01_claroty_generated_flat_key_types_match_toml() {
    for (i, dev) in generated_surface("device").iter().enumerate() {
        // TOML column_type = "string" columns
        for col in [
            "uid",
            "asset_id",
            "device_category",
            "device_type",
            "risk_score",
            // Tier 2 string columns added in PR #236 (CRITICAL-1 fix):
            "purdue_level",
            "site_name",
            "criticality",
            "device_name",
            "manufacturer",
        ] {
            assert!(
                dev.get(col).is_some_and(serde_json::Value::is_string),
                "generated device[{i}] '{col}' must be a string \
                 (TOML column_type = \"string\")"
            );
        }
        // TOML column_type = "boolean" columns
        for col in [
            "retired",
            // Tier 2 boolean column added in PR #236 (CRITICAL-1 fix):
            "is_online",
        ] {
            assert!(
                dev.get(col).is_some_and(serde_json::Value::is_boolean),
                "generated device[{i}] '{col}' must be a boolean \
                 (TOML column_type = \"boolean\")"
            );
        }
    }
    for (i, alert) in generated_surface("alert").iter().enumerate() {
        // TOML column_type = "datetime" columns: verify RFC 3339 parse
        for col in ["detected_time", "updated_time"] {
            let ts = alert
                .get(col)
                .and_then(|v| v.as_str())
                .unwrap_or_else(|| panic!("generated alert[{i}] missing datetime '{col}'"));
            ts.parse::<chrono::DateTime<chrono::Utc>>()
                .unwrap_or_else(|e| panic!("generated alert[{i}] '{col}'='{ts}' not RFC3339: {e}"));
        }
        // TOML column_type = "integer" columns
        for col in [
            "devices_count",
            // Tier 2 integer column added in PR #236 (CRITICAL-1 fix):
            "ot_devices_count",
        ] {
            assert!(
                alert.get(col).is_some_and(serde_json::Value::is_u64),
                "generated alert[{i}] '{col}' must be an integer \
                 (TOML column_type = \"integer\")"
            );
        }
        // TOML column_type = "string" columns added in PR #236 (CRITICAL-1 fix)
        for col in ["alert_class", "alert_name"] {
            assert!(
                alert.get(col).is_some_and(serde_json::Value::is_string),
                "generated alert[{i}] '{col}' must be a string \
                 (TOML column_type = \"string\")"
            );
        }
        // TOML alerts.id is polymorphic (handles int and UUID string upstream
        // IDs): the generator emits the integer form -- assert it is one of the
        // two valid wire classes.
        let id = alert.get("id").expect("id key present");
        assert!(
            id.is_u64() || id.is_string(),
            "generated alert[{i}] 'id' must be integer or string (polymorphic), got {id}"
        );
    }
}
