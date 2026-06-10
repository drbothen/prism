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
