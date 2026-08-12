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
// Helpers
// ---------------------------------------------------------------------------

/// Canonical test org: bytes [0xde, 0xad, 0xbe, 0xef, ...] → org_slug = "deadbeef".
fn deadbeef_org() -> OrgId {
    OrgId([
        0xde, 0xad, 0xbe, 0xef, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00,
    ])
}

/// claroty.sensor.toml `devices` table columns.
const DEVICE_COLUMNS: &[&str] = &[
    "uid",
    "asset_id",
    "device_category",
    "device_type",
    "risk_score",
    "retired",
];

/// claroty.sensor.toml `alerts` table columns.
const ALERT_COLUMNS: &[&str] = &[
    "id",
    "alert_type_name",
    "category",
    "status",
    "detected_time",
    "updated_time",
    "devices_count",
    "description",
];

/// claroty.sensor.toml `audit_logs` table columns (Tier 0+1 real xDome fields).
const AUDIT_LOG_COLUMNS: &[&str] = &[
    "id",
    "action",
    "user_display_name",
    "category",
    "timestamp",
    "details",
    "username",
    "note",
];

/// Generated records of a given `_surface` for CompromisedEndpoint, default opts.
fn generated_surface(surface: &str) -> Vec<serde_json::Value> {
    let fs = generate(
        &deadbeef_org(),
        Archetype::CompromisedEndpoint,
        &GenOpts::default(),
    );
    fs.records
        .into_iter()
        .filter(|r| r.get("_surface").and_then(|v| v.as_str()) == Some(surface))
        .collect()
}

fn static_fixture(raw: &str, name: &str) -> Vec<serde_json::Value> {
    serde_json::from_str(raw).unwrap_or_else(|e| panic!("{name} must be a JSON array: {e}"))
}

/// Flat scalar key set: scalar-valued keys, excluding `_`-prefixed internal
/// tags (`_surface`). Nested arrays/objects (ip_list, labels, …) are invisible
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

// ---------------------------------------------------------------------------
// P2-01 — TOML column coverage per serving path
// ---------------------------------------------------------------------------

/// Generated device records cover every TOML `devices` column.
#[test]
fn test_p2_01_claroty_generated_devices_cover_toml_columns() {
    let keys = uniform_flat_keys(&generated_surface("device"), "generated devices");
    assert_columns_covered(&keys, DEVICE_COLUMNS, "generated devices");
}

/// Generated alert records cover every TOML `alerts` column.
#[test]
fn test_p2_01_claroty_generated_alerts_cover_toml_columns() {
    let keys = uniform_flat_keys(&generated_surface("alert"), "generated alerts");
    assert_columns_covered(&keys, ALERT_COLUMNS, "generated alerts");
}

/// Static fixture devices cover every TOML `devices` column.
#[test]
fn test_p2_01_claroty_static_devices_cover_toml_columns() {
    let records = static_fixture(include_str!("../fixtures/devices.json"), "devices.json");
    let keys = uniform_flat_keys(&records, "static devices");
    assert_columns_covered(&keys, DEVICE_COLUMNS, "static devices");
}

/// Static fixture alerts cover every TOML `alerts` column.
#[test]
fn test_p2_01_claroty_static_alerts_cover_toml_columns() {
    let records = static_fixture(include_str!("../fixtures/alerts.json"), "alerts.json");
    let keys = uniform_flat_keys(&records, "static alerts");
    assert_columns_covered(&keys, ALERT_COLUMNS, "static alerts");
}

/// Static fixture audit logs cover every TOML `audit_logs` column (the only
/// serving path for this table — no generated audit-log surface exists).
#[test]
fn test_p2_01_claroty_static_audit_logs_cover_toml_columns() {
    let records = static_fixture(include_str!("../fixtures/audit-log.json"), "audit-log.json");
    let keys = uniform_flat_keys(&records, "static audit logs");
    assert_columns_covered(&keys, AUDIT_LOG_COLUMNS, "static audit logs");
}

// ---------------------------------------------------------------------------
// P2-01 — TOML column TYPES hold on the generated path
// ---------------------------------------------------------------------------

/// Generated records carry TOML-compatible value types: datetime columns parse
/// as RFC 3339, `devices_count` is an integer, `retired` is a boolean,
/// `risk_score` is a string (claroty.sensor.toml declares it string-typed).
#[test]
fn test_p2_01_claroty_generated_flat_key_types_match_toml() {
    for (i, dev) in generated_surface("device").iter().enumerate() {
        for col in [
            "uid",
            "asset_id",
            "device_category",
            "device_type",
            "risk_score",
        ] {
            assert!(
                dev.get(col).is_some_and(serde_json::Value::is_string),
                "generated device[{i}] '{col}' must be a string (TOML column_type)"
            );
        }
        assert!(
            dev.get("retired")
                .is_some_and(serde_json::Value::is_boolean),
            "generated device[{i}] 'retired' must be a boolean (TOML column_type)"
        );
    }
    for (i, alert) in generated_surface("alert").iter().enumerate() {
        for col in ["detected_time", "updated_time"] {
            let ts = alert
                .get(col)
                .and_then(|v| v.as_str())
                .unwrap_or_else(|| panic!("generated alert[{i}] missing datetime '{col}'"));
            ts.parse::<chrono::DateTime<chrono::Utc>>()
                .unwrap_or_else(|e| panic!("generated alert[{i}] '{col}'='{ts}' not RFC3339: {e}"));
        }
        assert!(
            alert
                .get("devices_count")
                .is_some_and(serde_json::Value::is_u64),
            "generated alert[{i}] 'devices_count' must be an integer (TOML column_type)"
        );
        // TOML alerts.id is string-typed POLYMORPHICALLY (handles int and UUID
        // upstream IDs); the generator emits the integer form — assert it is
        // one of the two real-API value classes.
        let id = alert.get("id").expect("id key present");
        assert!(
            id.is_u64() || id.is_string(),
            "generated alert[{i}] 'id' must be integer or string (polymorphic), got {id}"
        );
    }
}
