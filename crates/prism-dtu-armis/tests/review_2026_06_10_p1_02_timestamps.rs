// Legitimately sensor-named: this IS the Armis DTU generator test. Exempt from
// tests/external/no-hardcoded-sensors/ compile-fail gate per ADR-023 §DTU-EXEMPT.
//! Fix-burst regression tests — review-2026-06-10 cascade pass-1, finding P1-02.
//!
//! The Armis generator emitted hardcoded timestamps (`"2024-01-01T00:00:00Z"`
//! `lastSeen`, `"2023-01-01T00:00:00Z"` `firstSeen`, `"2024-01-01T12:00:00Z"`
//! alert `time`) for every record, ignoring `GenOpts::time_anchor`. Timestamps
//! are now derived per record from `opts.time_anchor` minus an RNG-free
//! `stable_offset` fold (the primary ChaCha20 stream is untouched —
//! INV-SECONDARY-RNG-STREAM-INDEPENDENCE-001).
#![cfg(feature = "fixture-gen")]
#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::collections::BTreeSet;

use chrono::{DateTime, Utc};
use prism_dtu_armis::generator::generate;
use prism_dtu_common::{demo_time_anchor, Archetype, GenOpts, OrgId};

const SLUG_A: &str = "orga";

fn org_a() -> OrgId {
    OrgId([
        0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F,
        0x10,
    ])
}

fn parse_ts(s: &str) -> DateTime<Utc> {
    s.parse::<DateTime<Utc>>()
        .unwrap_or_else(|e| panic!("'{s}' is not ISO-8601: {e}"))
}

/// Timestamps of the given field from records that carry it
/// (CompromisedEndpoint: 50 assets + 20 alerts, default opts).
fn field_timestamps(field: &str) -> Vec<DateTime<Utc>> {
    let fs = generate(
        org_a(),
        SLUG_A,
        Archetype::CompromisedEndpoint,
        &GenOpts::default(),
    );
    fs.records
        .iter()
        .filter_map(|r| r.get(field).and_then(|v| v.as_str()).map(parse_ts))
        .collect()
}

/// P1-02: asset `lastSeen` varies per record.
#[test]
fn test_p1_02_armis_last_seen_distinct_across_records() {
    let stamps = field_timestamps("lastSeen");
    assert!(stamps.len() > 1, "need >1 asset to assert variety");
    let distinct: BTreeSet<_> = stamps.iter().collect();
    assert!(
        distinct.len() > 1,
        "all {} assets share one lastSeen — window filtering cannot discriminate (P1-02)",
        stamps.len()
    );
}

/// P1-02: alert `time` varies per record.
#[test]
fn test_p1_02_armis_alert_time_distinct_across_records() {
    let stamps = field_timestamps("time");
    assert!(stamps.len() > 1, "need >1 alert to assert variety");
    let distinct: BTreeSet<_> = stamps.iter().collect();
    assert!(
        distinct.len() > 1,
        "all {} alerts share one time — P1-02",
        stamps.len()
    );
}

/// P1-02: timestamps live in the anchor era; `firstSeen` strictly precedes
/// `lastSeen` per asset.
#[test]
fn test_p1_02_armis_timestamps_anchor_era_and_ordering() {
    let anchor = demo_time_anchor();
    let floor = anchor - chrono::Duration::days(200); // firstSeen up to ~97+7d before lastSeen
    let fs = generate(
        org_a(),
        SLUG_A,
        Archetype::CompromisedEndpoint,
        &GenOpts::default(),
    );
    for (i, r) in fs.records.iter().enumerate() {
        if let Some(last) = r.get("lastSeen").and_then(|v| v.as_str()) {
            let last = parse_ts(last);
            assert!(
                last > floor && last <= anchor,
                "record[{i}] lastSeen {last} outside anchor era ({floor}..={anchor})"
            );
            let first = r
                .get("firstSeen")
                .and_then(|v| v.as_str())
                .unwrap_or_else(|| panic!("record[{i}] has lastSeen but no firstSeen"));
            let first = parse_ts(first);
            assert!(
                first < last,
                "record[{i}] firstSeen {first} must precede lastSeen {last}"
            );
        }
        if let Some(t) = r.get("time").and_then(|v| v.as_str()) {
            let t = parse_ts(t);
            assert!(
                t > floor && t <= anchor,
                "record[{i}] alert time {t} outside anchor era"
            );
        }
    }
}

/// P1-02: timestamps are deterministic — two same-seed generations agree
/// byte-for-byte (BC-3.4.001; stable_offset is RNG-free).
#[test]
fn test_p1_02_armis_timestamps_deterministic() {
    let opts = GenOpts::default();
    let fs1 = generate(org_a(), SLUG_A, Archetype::CompromisedEndpoint, &opts);
    let fs2 = generate(org_a(), SLUG_A, Archetype::CompromisedEndpoint, &opts);
    assert_eq!(
        serde_json::to_string(&fs1.records).unwrap(),
        serde_json::to_string(&fs2.records).unwrap(),
        "same-seed generations must be byte-identical (incl. derived timestamps)"
    );
}

/// P1-02: a window pinned to the earliest lastSeen matches a strict, non-empty
/// subset — bounded windows discriminate between records.
#[test]
fn test_p1_02_armis_window_discriminates() {
    let stamps = field_timestamps("lastSeen");
    let earliest = *stamps.iter().min().expect("at least one asset");
    let in_window = stamps.iter().filter(|t| **t <= earliest).count();
    assert!(in_window >= 1, "window must match the earliest record");
    assert!(
        in_window < stamps.len(),
        "window matched ALL {} assets — timestamps do not discriminate (P1-02)",
        stamps.len()
    );
}
