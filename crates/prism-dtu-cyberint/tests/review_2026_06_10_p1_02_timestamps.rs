// Legitimately sensor-named: this IS the Cyberint DTU generator test. Exempt from
// tests/external/no-hardcoded-sensors/ compile-fail gate per ADR-023 §DTU-EXEMPT.
//! Fix-burst regression tests — review-2026-06-10 cascade pass-1, finding P1-02.
//!
//! The Cyberint generator emitted a single hardcoded timestamp
//! (`"2024-01-01T00:00:00Z"`) for every record, ignoring `GenOpts::time_anchor`.
//! Time-window queries could not discriminate between records and missed the
//! demo era entirely. Timestamps are now derived per record from
//! `opts.time_anchor` minus an RNG-free `stable_offset` fold (the primary
//! ChaCha20 stream is untouched — INV-SECONDARY-RNG-STREAM-INDEPENDENCE-001).
#![cfg(feature = "fixture-gen")]
#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::collections::BTreeSet;

use chrono::{DateTime, Utc};
use prism_dtu_common::{demo_time_anchor, Archetype, GenOpts, OrgId};
use prism_dtu_cyberint::generator::generate;

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

/// Records of one `_surface`, with the given timestamp field extracted.
fn surface_timestamps(surface: &str, field: &str) -> Vec<DateTime<Utc>> {
    let fs = generate(
        &org_a(),
        Archetype::CompromisedEndpoint,
        &GenOpts::default(),
    );
    fs.records
        .iter()
        .filter(|r| r.get("_surface").and_then(|v| v.as_str()) == Some(surface))
        .map(|r| {
            let ts = r
                .get(field)
                .and_then(|v| v.as_str())
                .unwrap_or_else(|| panic!("{surface} record missing {field}"));
            parse_ts(ts)
        })
        .collect()
}

/// P1-02: alert `created_at` varies per record (windows can discriminate).
#[test]
fn test_p1_02_cyberint_alert_created_at_distinct_across_records() {
    let stamps = surface_timestamps("alert", "created_at");
    assert!(stamps.len() > 1, "need >1 alert to assert variety");
    let distinct: BTreeSet<_> = stamps.iter().collect();
    assert!(
        distinct.len() > 1,
        "all {} alerts share one created_at — window filtering cannot discriminate (P1-02)",
        stamps.len()
    );
}

/// P1-02: asm_asset `created` varies per record.
#[test]
fn test_p1_02_cyberint_asm_created_distinct_across_records() {
    let stamps = surface_timestamps("asm_asset", "created");
    assert!(stamps.len() > 1, "need >1 asm asset to assert variety");
    let distinct: BTreeSet<_> = stamps.iter().collect();
    assert!(
        distinct.len() > 1,
        "all {} asm assets share one created — P1-02",
        stamps.len()
    );
}

/// P1-02: timestamps live in the anchor era — within 98 days before `time_anchor`.
#[test]
fn test_p1_02_cyberint_timestamps_anchor_era() {
    let anchor = demo_time_anchor();
    let floor = anchor - chrono::Duration::days(98);
    for (surface, field) in [
        ("alert", "created_at"),
        ("alert", "update_date"),
        ("asm_asset", "created"),
        ("cve", "published_date"),
    ] {
        for ts in surface_timestamps(surface, field) {
            assert!(
                ts > floor && ts <= anchor,
                "{surface}.{field} {ts} outside anchor era ({floor}..={anchor})"
            );
        }
    }
}

/// P1-02: timestamps are deterministic — two same-seed generations agree
/// byte-for-byte (BC-3.4.001; stable_offset is RNG-free).
#[test]
fn test_p1_02_cyberint_timestamps_deterministic() {
    let fs1 = generate(
        &org_a(),
        Archetype::CompromisedEndpoint,
        &GenOpts::default(),
    );
    let fs2 = generate(
        &org_a(),
        Archetype::CompromisedEndpoint,
        &GenOpts::default(),
    );
    assert_eq!(
        serde_json::to_string(&fs1.records).unwrap(),
        serde_json::to_string(&fs2.records).unwrap(),
        "same-seed generations must be byte-identical (incl. derived timestamps)"
    );
}

/// P1-02: a window pinned to the earliest alert timestamp matches a strict,
/// non-empty subset — bounded windows discriminate between records.
#[test]
fn test_p1_02_cyberint_window_discriminates() {
    let stamps = surface_timestamps("alert", "created_at");
    let earliest = *stamps.iter().min().expect("at least one alert");
    let in_window = stamps.iter().filter(|t| **t <= earliest).count();
    assert!(in_window >= 1, "window must match the earliest record");
    assert!(
        in_window < stamps.len(),
        "window matched ALL {} alerts — timestamps do not discriminate (P1-02)",
        stamps.len()
    );
}
