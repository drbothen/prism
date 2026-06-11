// Legitimately sensor-named: this IS the Armis DTU parity test. Exempt from
// tests/external/no-hardcoded-sensors/ compile-fail gate per ADR-023 §DTU-EXEMPT.
//! Fix-burst regression tests — review-2026-06-10 cascade pass-2, finding P2-01.
//!
//! P2-01 (CRIT, SAP-2): the Armis SEEDED path served generator records whose
//! keys were camelCase real-API shapes (`lastSeen`, `ipAddress`, `riskLevel`,
//! alert `title` / `policyId` / `time`), while `armis.sensor.toml` extracts
//! FLAT snake_case column names (`last_seen`, `ip_address`, `risk_score`,
//! alert `name` / `policy_name` / `created_at`). The serving extraction is
//! exact-name flat `raw.get(col)` (column_mapping.rs) — every TOML column
//! silently normalized to NULL on the seeded path. The static path
//! (types.rs snake_case structs) matched; only the generator drifted.
//!
//! Fix (CrowdStrike F4 additive pattern): the generator now emits ADDITIVE
//! flat snake_case TOML-parity keys ALONGSIDE the camelCase real-API keys —
//! API-shape fidelity is preserved, and the flat keys mirror the camelCase
//! values exactly.
//!
//! This file is the CS-06-style shape-parity test class for Armis — the
//! missing test that let P2-01 survive: generator flat-key set vs TOML
//! columns vs static-fixture keys.

#![cfg(all(feature = "dtu", feature = "fixture-gen"))]
#![allow(clippy::unwrap_used, clippy::expect_used, non_snake_case)]

use std::collections::BTreeSet;

use prism_dtu_armis::generator::generate;
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

const SLUG: &str = "deadbeef";

/// armis.sensor.toml `devices` table data columns (the `aql` push-down
/// pseudo-column is excluded — it has no `ocsf_field` and is never extracted
/// from response records; BC-2.11.007 §Mechanism B).
const DEVICE_COLUMNS: &[&str] = &[
    "device_id",
    "name",
    "type",
    "manufacturer",
    "last_seen",
    "first_seen",
    "ip_address",
    "mac_address",
    "os_name",
    "risk_score",
];

/// armis.sensor.toml `alerts` table data columns (same `aql` exclusion).
const ALERT_COLUMNS: &[&str] = &[
    "alert_id",
    "name",
    "severity",
    "status",
    "policy_name",
    "device_id",
    "created_at",
    "updated_at",
];

/// Generate CompromisedEndpoint records (50 assets + 20 alerts, default opts).
fn generated_fixture() -> Vec<serde_json::Value> {
    generate(
        deadbeef_org(),
        SLUG,
        Archetype::CompromisedEndpoint,
        &GenOpts::default(),
    )
    .records
}

/// Generated device records — same partition predicate as routes/search.rs.
fn generated_devices() -> Vec<serde_json::Value> {
    generated_fixture()
        .into_iter()
        .filter(|r| r.get("asset_id").is_some() && r.get("alert_id").is_none())
        .collect()
}

/// Generated alert records — same partition predicate as routes/search.rs.
fn generated_alerts() -> Vec<serde_json::Value> {
    generated_fixture()
        .into_iter()
        .filter(|r| r.get("alert_id").is_some())
        .collect()
}

/// Static fixture devices SERVED shape: deserialize fixtures/devices.json
/// through `DeviceRecord` and re-serialize — exactly what the static route
/// path does, so Option fields absent in the file appear as null keys.
fn static_devices() -> Vec<serde_json::Value> {
    let raw = include_str!("../fixtures/devices.json");
    let records: Vec<prism_dtu_armis::types::DeviceRecord> =
        serde_json::from_str(raw).expect("devices.json must deserialize as Vec<DeviceRecord>");
    records
        .iter()
        .map(|r| serde_json::to_value(r).expect("DeviceRecord serializes"))
        .collect()
}

/// Static fixture alerts SERVED shape (via `AlertRecord`).
fn static_alerts() -> Vec<serde_json::Value> {
    let raw = include_str!("../fixtures/alerts.json");
    let records: Vec<prism_dtu_armis::types::AlertRecord> =
        serde_json::from_str(raw).expect("alerts.json must deserialize as Vec<AlertRecord>");
    records
        .iter()
        .map(|r| serde_json::to_value(r).expect("AlertRecord serializes"))
        .collect()
}

/// Flat scalar key set of a record: keys whose values are scalars
/// (string/number/bool/null), excluding `_`-prefixed internal tags (`_seed`).
/// Nested objects/arrays are API-shape carriers invisible to the flat
/// `r.get(col)` serving extraction (CS-06 precedent).
fn flat_scalar_keys(record: &serde_json::Value) -> BTreeSet<String> {
    record
        .as_object()
        .expect("record must be a JSON object")
        .iter()
        .filter(|(k, v)| !k.starts_with('_') && !v.is_object() && !v.is_array())
        .map(|(k, _)| k.clone())
        .collect()
}

/// Flat scalar keys of a record slice, asserting every record carries the
/// identical key set — no per-record drift within a path (CS-06 precedent).
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

fn parse_ts(s: &str, ctx: &str) -> chrono::DateTime<chrono::Utc> {
    s.parse::<chrono::DateTime<chrono::Utc>>()
        .unwrap_or_else(|e| panic!("{ctx}: '{s}' is not ISO-8601: {e}"))
}

// ---------------------------------------------------------------------------
// P2-01 — TOML column coverage on BOTH serving paths
// ---------------------------------------------------------------------------

/// P2-01: every generated device record carries every armis.sensor.toml
/// `devices` column as a flat key — the seeded path must not NULL columns.
#[test]
fn test_p2_01_generated_devices_cover_toml_columns() {
    let keys = uniform_flat_keys(&generated_devices(), "generated devices");
    for col in DEVICE_COLUMNS {
        assert!(
            keys.contains(*col),
            "generated devices missing TOML column '{col}' (P2-01 SAP-2: \
             flat raw.get extraction silently normalizes it to NULL)"
        );
    }
}

/// P2-01: every generated alert record carries every armis.sensor.toml
/// `alerts` column as a flat key.
#[test]
fn test_p2_01_generated_alerts_cover_toml_columns() {
    let keys = uniform_flat_keys(&generated_alerts(), "generated alerts");
    for col in ALERT_COLUMNS {
        assert!(
            keys.contains(*col),
            "generated alerts missing TOML column '{col}' (P2-01 SAP-2)"
        );
    }
}

/// P2-01: the static fixture path (types.rs snake_case structs) covers the
/// TOML devices columns — non-regression pin for the path that was correct.
#[test]
fn test_p2_01_static_devices_cover_toml_columns() {
    let keys = uniform_flat_keys(&static_devices(), "static devices");
    for col in DEVICE_COLUMNS {
        assert!(
            keys.contains(*col),
            "static devices missing TOML column '{col}'"
        );
    }
}

/// P2-01: static alerts path covers the TOML alerts columns.
#[test]
fn test_p2_01_static_alerts_cover_toml_columns() {
    let keys = uniform_flat_keys(&static_alerts(), "static alerts");
    for col in ALERT_COLUMNS {
        assert!(
            keys.contains(*col),
            "static alerts missing TOML column '{col}'"
        );
    }
}

// ---------------------------------------------------------------------------
// P2-01 — flat snake_case keys MIRROR the camelCase real-API values
// (CrowdStrike F4 additive pattern: flat == nested/camel agreement)
// ---------------------------------------------------------------------------

/// P2-01: on generated devices the additive flat keys mirror their camelCase
/// real-API counterparts exactly — the two shapes cannot drift apart.
#[test]
fn test_p2_01_generated_device_flat_keys_mirror_camelcase() {
    let mirrors = [
        ("last_seen", "lastSeen"),
        ("first_seen", "firstSeen"),
        ("ip_address", "ipAddress"),
        ("mac_address", "macAddress"),
        ("os_name", "operatingSystem"),
        ("risk_score", "riskLevel"),
        ("device_id", "asset_id"),
    ];
    for (i, dev) in generated_devices().iter().enumerate() {
        for (flat, camel) in mirrors {
            assert_eq!(
                dev.get(flat),
                dev.get(camel),
                "generated device[{i}] flat '{flat}' must mirror '{camel}' (P2-01)"
            );
        }
    }
}

/// P2-01: on generated alerts the additive flat keys mirror their real-API
/// counterparts; `device_id` (string, TOML type) links to the seeded device
/// pool exactly like CrowdStrike CS-01.
#[test]
fn test_p2_01_generated_alert_flat_keys_mirror_api_shape() {
    let device_pool: BTreeSet<String> = generated_devices()
        .iter()
        .filter_map(|d| {
            d.get("device_id")
                .and_then(|v| v.as_str())
                .map(String::from)
        })
        .collect();
    assert!(!device_pool.is_empty(), "must generate devices");

    let mirrors = [
        ("name", "title"),
        ("policy_name", "policyId"),
        ("created_at", "time"),
        ("updated_at", "lastAlertUpdateTime"),
    ];
    for (i, alert) in generated_alerts().iter().enumerate() {
        for (flat, api) in mirrors {
            assert_eq!(
                alert.get(flat),
                alert.get(api),
                "generated alert[{i}] flat '{flat}' must mirror '{api}' (P2-01)"
            );
        }
        let dev_id = alert
            .get("device_id")
            .and_then(|v| v.as_str())
            .unwrap_or_else(|| panic!("generated alert[{i}] missing flat string device_id"));
        assert!(
            device_pool.contains(dev_id),
            "generated alert[{i}] device_id '{dev_id}' not in seeded device pool (P2-01)"
        );
        let dev_idx = alert
            .get("deviceId")
            .and_then(serde_json::Value::as_i64)
            .unwrap_or_else(|| panic!("generated alert[{i}] missing integer deviceId"));
        assert!(
            dev_id.ends_with(&format!("-{dev_idx}")),
            "generated alert[{i}] device_id '{dev_id}' must reference deviceId {dev_idx}"
        );
    }
}

/// P2-01: TOML column TYPES hold on the generated path — datetime columns are
/// ISO-8601 datetime strings (anchor-derived per the P1-02 pattern), string
/// columns are strings, and `risk_score` is integer-or-null.
#[test]
fn test_p2_01_generated_flat_key_types_match_toml() {
    for (i, dev) in generated_devices().iter().enumerate() {
        for col in ["last_seen", "first_seen"] {
            let ts = dev
                .get(col)
                .and_then(|v| v.as_str())
                .unwrap_or_else(|| panic!("generated device[{i}] missing datetime '{col}'"));
            parse_ts(ts, col);
        }
        for col in [
            "device_id",
            "name",
            "type",
            "manufacturer",
            "ip_address",
            "mac_address",
        ] {
            assert!(
                dev.get(col).is_some_and(serde_json::Value::is_string),
                "generated device[{i}] '{col}' must be a string (TOML column_type)"
            );
        }
        let risk = dev.get("risk_score").expect("risk_score key present");
        assert!(
            risk.is_null() || risk.is_u64(),
            "generated device[{i}] risk_score must be integer-or-null (TOML integer), got {risk}"
        );
    }
    for (i, alert) in generated_alerts().iter().enumerate() {
        for col in ["created_at", "updated_at"] {
            let ts = alert
                .get(col)
                .and_then(|v| v.as_str())
                .unwrap_or_else(|| panic!("generated alert[{i}] missing datetime '{col}'"));
            parse_ts(ts, col);
        }
        for col in [
            "alert_id",
            "name",
            "severity",
            "status",
            "policy_name",
            "device_id",
        ] {
            assert!(
                alert.get(col).is_some_and(serde_json::Value::is_string),
                "generated alert[{i}] '{col}' must be a string (TOML column_type)"
            );
        }
    }
}
