//! Fix-burst regression tests — 2026-06-10 review, CrowdStrike DTU parity findings.
//!
//! Findings covered (review package "Fix PR 1: DTU fleet"):
//! - F4 / CS-01: TOML column `detections.device_id` must exist as a FLAT key on
//!   both the generated path (make_detection) and the static fixture path
//!   (fixtures/detections-detail.json). Serving extraction is flat
//!   `r.get(col_name)` — an absent key normalizes to NULL.
//!
//! Further findings (F5/CS-02, F6/CS-03, F7/CS-04, F8/CS-06) accrete to this
//! file as their fixes land in the same fix-burst.

#![cfg(all(feature = "fixture-gen", feature = "dtu"))]
#![allow(clippy::unwrap_used, clippy::expect_used, non_snake_case)]

use std::collections::BTreeSet;

use prism_dtu_common::{Archetype, GenOpts, OrgId};
use prism_dtu_crowdstrike::generate;

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

fn static_detections() -> Vec<serde_json::Value> {
    let raw = include_str!("../fixtures/detections-detail.json");
    serde_json::from_str(raw).expect("detections-detail.json must be a JSON array")
}

/// Generated records of a given `_record_type` for CompromisedEndpoint at seed 42.
fn generated_records(record_type: &str) -> Vec<serde_json::Value> {
    let fs = generate(
        deadbeef_org(),
        Archetype::CompromisedEndpoint,
        GenOpts::default(),
    );
    fs.records
        .into_iter()
        .filter(|r| r.get("_record_type").and_then(|v| v.as_str()) == Some(record_type))
        .collect()
}

// ---------------------------------------------------------------------------
// F4 / CS-01 — detections.device_id flat on both paths
// ---------------------------------------------------------------------------

/// F4 / CS-01: every generated detection carries a flat `device_id` drawn from
/// the seeded device pool (detections correlate with generated devices).
#[test]
fn test_f4_cs01_generated_detection_has_flat_device_id_from_device_pool() {
    let devices = generated_records("device");
    let detections = generated_records("detection");
    assert!(!devices.is_empty(), "must generate devices");
    assert!(!detections.is_empty(), "must generate detections");

    let device_pool: BTreeSet<&str> = devices
        .iter()
        .filter_map(|d| d.get("device_id").and_then(|v| v.as_str()))
        .collect();

    for (i, det) in detections.iter().enumerate() {
        let dev_id = det
            .get("device_id")
            .and_then(|v| v.as_str())
            .unwrap_or_else(|| panic!("generated detection[{i}] missing flat device_id (CS-01)"));
        assert!(
            device_pool.contains(dev_id),
            "generated detection[{i}] device_id '{dev_id}' not in seeded device pool"
        );
    }
}

/// F4 / CS-01: every static fixture detection carries a flat `device_id` equal
/// to the nested `device.device_id` (nested object kept for API-shape fidelity).
#[test]
fn test_f4_cs01_static_detection_has_flat_device_id() {
    for (i, det) in static_detections().iter().enumerate() {
        let nested = det
            .get("device")
            .and_then(|d| d.get("device_id"))
            .and_then(|v| v.as_str())
            .unwrap_or_else(|| panic!("static detection[{i}] missing nested device.device_id"));
        let flat = det
            .get("device_id")
            .and_then(|v| v.as_str())
            .unwrap_or_else(|| panic!("static detection[{i}] missing flat device_id (CS-01)"));
        assert_eq!(
            flat, nested,
            "static detection[{i}] flat device_id must equal nested device.device_id"
        );
    }
}

// ---------------------------------------------------------------------------
// F5 / CS-02 — detections.tactic / technique flat on the static path
// ---------------------------------------------------------------------------

/// F5 / CS-02: every static fixture detection carries flat `tactic` / `technique`
/// equal to `behaviors[0].tactic` / `behaviors[0].technique` (behaviors[] kept
/// intact for API-shape fidelity). The generator already emits these flat.
#[test]
fn test_f5_cs02_static_detection_has_flat_tactic_technique() {
    for (i, det) in static_detections().iter().enumerate() {
        let b0 = det
            .get("behaviors")
            .and_then(|b| b.get(0))
            .unwrap_or_else(|| panic!("static detection[{i}] missing behaviors[0]"));
        for key in ["tactic", "technique"] {
            let nested = b0
                .get(key)
                .and_then(|v| v.as_str())
                .unwrap_or_else(|| panic!("static detection[{i}] behaviors[0] missing {key}"));
            let flat = det
                .get(key)
                .and_then(|v| v.as_str())
                .unwrap_or_else(|| panic!("static detection[{i}] missing flat {key} (CS-02)"));
            assert_eq!(
                flat, nested,
                "static detection[{i}] flat {key} must equal behaviors[0].{key}"
            );
        }
    }
}

/// F5 / CS-02 (non-regression): the generated path already emits flat
/// `tactic` / `technique` — keep it that way.
#[test]
fn test_f5_cs02_generated_detection_has_flat_tactic_technique() {
    for (i, det) in generated_records("detection").iter().enumerate() {
        for key in ["tactic", "technique"] {
            assert!(
                det.get(key).and_then(|v| v.as_str()).is_some(),
                "generated detection[{i}] missing flat {key}"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// F6 / CS-03 — devices.first_seen on both paths
// ---------------------------------------------------------------------------

/// Parse an ISO-8601 Z timestamp, panicking with context on failure.
fn parse_ts(s: &str, ctx: &str) -> chrono::DateTime<chrono::Utc> {
    s.parse::<chrono::DateTime<chrono::Utc>>()
        .unwrap_or_else(|e| panic!("{ctx}: '{s}' is not ISO-8601 Z: {e}"))
}

fn static_hosts() -> Vec<serde_json::Value> {
    let raw = include_str!("../fixtures/hosts-detail.json");
    serde_json::from_str(raw).expect("hosts-detail.json must be a JSON array")
}

/// F6 / CS-03: every generated device carries `first_seen` (ISO-8601 Z) strictly
/// earlier than `last_seen`, and the value is seeded-deterministic (two identical
/// generate calls produce byte-identical device records).
#[test]
fn test_f6_cs03_generated_device_first_seen() {
    let devices = generated_records("device");
    assert!(!devices.is_empty(), "must generate devices");
    for (i, dev) in devices.iter().enumerate() {
        let first = dev
            .get("first_seen")
            .and_then(|v| v.as_str())
            .unwrap_or_else(|| panic!("generated device[{i}] missing first_seen (CS-03)"));
        let last = dev
            .get("last_seen")
            .and_then(|v| v.as_str())
            .unwrap_or_else(|| panic!("generated device[{i}] missing last_seen"));
        assert!(
            parse_ts(first, "first_seen") < parse_ts(last, "last_seen"),
            "generated device[{i}] first_seen '{first}' must be earlier than last_seen '{last}'"
        );
    }

    // Determinism: identical inputs → byte-identical records (BC-3.4.001).
    let again = generated_records("device");
    assert_eq!(
        serde_json::to_string(&devices).unwrap(),
        serde_json::to_string(&again).unwrap(),
        "generated devices (incl. first_seen) must be deterministic for identical inputs"
    );
}

/// F6 / CS-03: every static fixture host carries `first_seen` (ISO-8601 Z)
/// strictly earlier than `last_seen`.
#[test]
fn test_f6_cs03_static_host_first_seen() {
    for (i, host) in static_hosts().iter().enumerate() {
        let first = host
            .get("first_seen")
            .and_then(|v| v.as_str())
            .unwrap_or_else(|| panic!("static host[{i}] missing first_seen (CS-03)"));
        let last = host
            .get("last_seen")
            .and_then(|v| v.as_str())
            .unwrap_or_else(|| panic!("static host[{i}] missing last_seen"));
        assert!(
            parse_ts(first, "first_seen") < parse_ts(last, "last_seen"),
            "static host[{i}] first_seen '{first}' must be earlier than last_seen '{last}'"
        );
    }
}

// ---------------------------------------------------------------------------
// F7 / CS-04 — seeded-path FQL time-window filtering (demo-critical)
// ---------------------------------------------------------------------------

/// F7 / CS-04 (part 1): generated detections must NOT all share one
/// created_timestamp — timestamps vary deterministically per record
/// (time_anchor minus seeded offsets) so FQL time windows can discriminate.
#[test]
fn test_f7_cs04_generated_detection_timestamps_vary() {
    let detections = generated_records("detection");
    assert!(detections.len() > 1, "need >1 detection to assert variety");
    let distinct: BTreeSet<&str> = detections
        .iter()
        .filter_map(|d| d.get("created_timestamp").and_then(|v| v.as_str()))
        .collect();
    assert!(
        distinct.len() > 1,
        "all {} generated detections share one created_timestamp ({distinct:?}) — \
         FQL time-window filtering cannot discriminate",
        detections.len()
    );
}

/// F7 / CS-04 (part 2): on a SEEDED clone, FQL created_timestamp window
/// filtering must consult the generated detections' own timestamps — not the
/// static load_detection_details() map (whose IDs never match generated IDs,
/// yielding ZERO rows for ANY bounded query).
///
/// - A window covering the generation time_anchor (`demo_time_anchor()` =
///   2026-01-01T00:00:00Z for GenOpts::default() per review-2026-06-10 P1-01;
///   detection offsets ≤ 7 days earlier) returns > 0 rows.
/// - A window entirely before all generated timestamps returns 0 rows.
#[tokio::test]
async fn test_f7_cs04_seeded_fql_window_filters_generated_timestamps() {
    use prism_dtu_common::BehavioralClone;

    let mut clone = prism_dtu_crowdstrike::CrowdstrikeClone::new_with_seed(
        42,
        Archetype::CompromisedEndpoint,
        deadbeef_org(),
    );
    clone.start().await.expect("clone start must succeed");
    let base_url = clone.base_url();
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("client build");

    // Window covering the demo-era anchor (2026-01-01T00:00:00Z) and the seeded
    // offsets up to 7 days before it (P1-01: realistic demo windows match data).
    let covering =
        "created_timestamp:>'2025-12-01T00:00:00Z'+created_timestamp:<'2026-01-02T00:00:00Z'";
    let resp = client
        .get(format!("{base_url}/detects/queries/detects/v1"))
        .query(&[("filter", covering)])
        .header("Authorization", "Bearer test-token-f7")
        .send()
        .await
        .expect("request must succeed");
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.expect("json body");
    let resources = body["resources"].as_array().expect("resources array");
    assert!(
        !resources.is_empty(),
        "seeded clone returned ZERO rows for an FQL window covering the anchor \
         (CS-04: static detail map consulted instead of generated detections)"
    );

    // Window entirely before all generated timestamps (pre-anchor era; the
    // earliest generated detection is anchor minus 7 days = 2025-12-25).
    let before_all =
        "created_timestamp:>'2020-01-01T00:00:00Z'+created_timestamp:<'2021-01-01T00:00:00Z'";
    let resp = client
        .get(format!("{base_url}/detects/queries/detects/v1"))
        .query(&[("filter", before_all)])
        .header("Authorization", "Bearer test-token-f7")
        .send()
        .await
        .expect("request must succeed");
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.expect("json body");
    let resources = body["resources"].as_array().expect("resources array");
    assert!(
        resources.is_empty(),
        "window entirely before all generated timestamps must return 0 rows, got {}",
        resources.len()
    );

    clone.stop().await.expect("clone stop must succeed");
}

// ---------------------------------------------------------------------------
// P1-03 (cascade pass-1) — technique VALUE-class parity (name vs MITRE ID)
// ---------------------------------------------------------------------------

/// `^T\d{4}$`-shaped MITRE technique ID (e.g. "T1059") without a regex dep.
fn is_technique_id_shaped(s: &str) -> bool {
    s.len() == 5 && s.starts_with('T') && s[1..].chars().all(|c| c.is_ascii_digit())
}

/// P1-03: VALUE-class agreement between the generator and the static fixtures —
/// not just key presence (the F8 shape-parity gap). On BOTH paths:
/// - flat `technique` is the MITRE display NAME (never ID-shaped),
/// - flat `technique_id` is the MITRE ID,
/// - the (id, name) pair matches the canonical `MITRE_TECHNIQUES` table,
/// so the TOML `attack.technique.name` column normalizes identically.
///
/// Real-API value semantics to be confirmed by dtu-validator (MEDIUM
/// confidence flag from the adversary).
#[test]
fn test_p1_03_technique_value_class_agreement_both_paths() {
    use prism_dtu_crowdstrike::generator::technique_name;

    let check = |records: &[serde_json::Value], path: &str| {
        for (i, det) in records.iter().enumerate() {
            let tid = det
                .get("technique_id")
                .and_then(|v| v.as_str())
                .unwrap_or_else(|| panic!("{path}[{i}] missing technique_id"));
            assert!(
                is_technique_id_shaped(tid),
                "{path}[{i}] technique_id '{tid}' is not MITRE-ID-shaped (P1-03)"
            );
            let tech = det
                .get("technique")
                .and_then(|v| v.as_str())
                .unwrap_or_else(|| panic!("{path}[{i}] missing technique"));
            assert!(
                !is_technique_id_shaped(tech),
                "{path}[{i}] technique '{tech}' is ID-shaped — must be the display \
                 name (value-class divergence, P1-03)"
            );
            let expected = technique_name(tid).unwrap_or_else(|| {
                panic!("{path}[{i}] technique_id '{tid}' not in MITRE_TECHNIQUES table")
            });
            assert_eq!(
                tech, expected,
                "{path}[{i}] technique must be the canonical name for {tid}"
            );
        }
    };

    check(&generated_records("detection"), "generated detection");
    check(&static_detections(), "static detection");
}

/// P1-03: nested `behaviors[0]` agrees with the flat pair on the static path —
/// `behaviors[0].technique` is the display name and `behaviors[0].technique_id`
/// is the MITRE ID (the nested object is the API-shape fidelity carrier).
#[test]
fn test_p1_03_static_behaviors_nested_flat_agreement() {
    for (i, det) in static_detections().iter().enumerate() {
        let b0 = det
            .get("behaviors")
            .and_then(|b| b.get(0))
            .unwrap_or_else(|| panic!("static detection[{i}] missing behaviors[0]"));
        for key in ["technique", "technique_id"] {
            let nested = b0
                .get(key)
                .and_then(|v| v.as_str())
                .unwrap_or_else(|| panic!("static detection[{i}] behaviors[0] missing {key}"));
            let flat = det
                .get(key)
                .and_then(|v| v.as_str())
                .unwrap_or_else(|| panic!("static detection[{i}] missing flat {key}"));
            assert_eq!(
                flat, nested,
                "static detection[{i}] flat {key} must equal behaviors[0].{key} (P1-03)"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// P1-01 (cascade pass-1) — demo-era default anchor + anchored constructor
// ---------------------------------------------------------------------------

/// P1-01: with `GenOpts::default()` the generated timestamps live in the
/// demo era (anchored at 2026-01-01T00:00:00Z, offsets ≤ 97 days earlier) —
/// NOT at UNIX_EPOCH. Realistic demo time-window queries must match data.
#[test]
fn test_p1_01_default_generated_timestamps_in_demo_era() {
    let anchor = prism_dtu_common::demo_time_anchor();
    let floor = anchor - chrono::Duration::days(98); // devices: first_seen ≤ 97d before
    for (i, det) in generated_records("detection").iter().enumerate() {
        let ts = det.get("created_timestamp").and_then(|v| v.as_str());
        let ts = parse_ts(ts.unwrap_or_default(), "created_timestamp");
        assert!(
            ts > floor && ts <= anchor,
            "generated detection[{i}] created_timestamp {ts} outside demo era \
             ({floor}..={anchor}) — P1-01 anchor regression"
        );
    }
    for (i, dev) in generated_records("device").iter().enumerate() {
        let ts = dev.get("first_seen").and_then(|v| v.as_str());
        let ts = parse_ts(ts.unwrap_or_default(), "first_seen");
        assert!(
            ts > floor && ts <= anchor,
            "generated device[{i}] first_seen {ts} outside demo era — P1-01"
        );
    }
}

/// P1-01: `new_with_seed_anchored` threads a caller-chosen anchor through to
/// the generated records (Story B `scenario_start_secs` → `time_anchor` hook).
#[tokio::test]
async fn test_p1_01_anchored_constructor_threads_time_anchor() {
    use prism_dtu_common::BehavioralClone;

    let custom: chrono::DateTime<chrono::Utc> =
        "2030-06-15T00:00:00Z".parse().expect("literal must parse");
    let mut clone = prism_dtu_crowdstrike::CrowdstrikeClone::new_with_seed_anchored(
        42,
        Archetype::CompromisedEndpoint,
        deadbeef_org(),
        custom,
    );
    clone.start().await.expect("clone start must succeed");
    let base_url = clone.base_url();
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("client build");

    // Window covering the custom anchor era returns rows.
    let covering =
        "created_timestamp:>'2030-06-01T00:00:00Z'+created_timestamp:<'2030-06-16T00:00:00Z'";
    let resp = client
        .get(format!("{base_url}/detects/queries/detects/v1"))
        .query(&[("filter", covering)])
        .header("Authorization", "Bearer test-token-p101")
        .send()
        .await
        .expect("request must succeed");
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.expect("json body");
    assert!(
        !body["resources"].as_array().expect("resources").is_empty(),
        "custom-anchored clone must serve rows in the custom anchor window"
    );

    clone.stop().await.expect("clone stop must succeed");
}

// ---------------------------------------------------------------------------
// F8 / CS-06 — canonical record shape parity (generator ↔ static fixture)
// ---------------------------------------------------------------------------

/// Flat scalar key set of a record: keys whose values are scalars
/// (string/number/bool/null), excluding `_`-prefixed internal tags
/// (`_record_type`). Nested objects/arrays (static `device{}` / `behaviors[]`)
/// are API-shape fidelity carriers invisible to the flat `r.get(col)` serving
/// extraction, so they are excluded from the parity contract.
fn flat_scalar_keys(record: &serde_json::Value) -> BTreeSet<String> {
    record
        .as_object()
        .expect("record must be a JSON object")
        .iter()
        .filter(|(k, v)| !k.starts_with('_') && !v.is_object() && !v.is_array())
        .map(|(k, _)| k.clone())
        .collect()
}

/// Union of flat scalar keys across a record slice (also asserts every record
/// carries the identical key set — no per-record drift within a path).
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

/// F8 / CS-06: the flat scalar key set of generated detection records must
/// equal the flat scalar key set of static fixture detection records — the
/// module doc (generator.rs) mandates route/generator consistency, and this is
/// the test class that would have caught CS-01/CS-02/CS-03 and Cyberint DTU-01:
/// a key present on only one path silently NULLs a column on the other path
/// the moment a TOML column references it.
#[test]
fn test_f8_cs06_detection_shape_parity() {
    let generated = uniform_flat_keys(&generated_records("detection"), "generated detections");
    let stat = uniform_flat_keys(&static_detections(), "static detections");
    assert_eq!(
        generated,
        stat,
        "detection flat scalar key sets diverge between generator and static fixture \
         (generated-only: {:?}; static-only: {:?})",
        generated.difference(&stat).collect::<Vec<_>>(),
        stat.difference(&generated).collect::<Vec<_>>()
    );
}

/// F8 / CS-06: same parity contract for the devices table.
#[test]
fn test_f8_cs06_device_shape_parity() {
    let generated = uniform_flat_keys(&generated_records("device"), "generated devices");
    let stat = uniform_flat_keys(&static_hosts(), "static hosts");
    assert_eq!(
        generated,
        stat,
        "device flat scalar key sets diverge between generator and static fixture \
         (generated-only: {:?}; static-only: {:?})",
        generated.difference(&stat).collect::<Vec<_>>(),
        stat.difference(&generated).collect::<Vec<_>>()
    );
}

/// F8 / CS-06: both canonical shapes must cover every column declared in
/// crowdstrike.sensor.toml for their table — the direct TOML↔record contract
/// (SAP-2 parity probe, code-level anchor).
#[test]
fn test_f8_cs06_toml_columns_covered() {
    // crowdstrike.sensor.toml [[tables]] detections / devices column names.
    let detection_columns = [
        "detection_id",
        "created_timestamp",
        "status",
        "severity",
        "device_id",
        "tactic",
        "technique",
    ];
    let device_columns = [
        "device_id",
        "hostname",
        "platform_name",
        "status",
        "first_seen",
        "last_seen",
    ];

    let gen_det = uniform_flat_keys(&generated_records("detection"), "generated detections");
    let stat_det = uniform_flat_keys(&static_detections(), "static detections");
    for col in detection_columns {
        assert!(
            gen_det.contains(col),
            "generated detections missing TOML column '{col}'"
        );
        assert!(
            stat_det.contains(col),
            "static detections missing TOML column '{col}'"
        );
    }

    let gen_dev = uniform_flat_keys(&generated_records("device"), "generated devices");
    let stat_dev = uniform_flat_keys(&static_hosts(), "static hosts");
    for col in device_columns {
        assert!(
            gen_dev.contains(col),
            "generated devices missing TOML column '{col}'"
        );
        assert!(
            stat_dev.contains(col),
            "static hosts missing TOML column '{col}'"
        );
    }
}
