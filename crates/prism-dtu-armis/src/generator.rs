//! Armis fixture generator — all 8 archetypes (S-3.7.04).
//!
//! Implements `generate(org_id, org_slug, archetype, opts) -> FixtureSet` for the Armis sensor,
//! producing deterministic synthetic records that match the Armis AQL response shapes
//! documented in `.references/schemas/armis/types.rs` and `DERIVATION.md`.
//!
//! SCHEMA-TYPES DECISION: approach (a) — field structure embedded directly in the
//! generator via `serde_json::json!` macros. The `.references/` path is NOT imported
//! as a Rust module per the story's Architecture Compliance Rules. Field names and
//! nullable conventions are taken verbatim from `DERIVATION.md` §3.
//!
//! default_page_size for Armis: 100 (source: DERIVATION.md §2).
//!
//! ArmisId duality (EC-001): string-form IDs by default; every 5th record (i % 5 == 0)
//! uses an integer-form `id` computed as a deterministic numeric hash of slug+seed+index.
//! An `asset_id` string field is ALWAYS present alongside `id` so that:
//!   • `primary_id()` (checks `asset_id` first) always returns a slug-containing string
//!     (VP-120), while
//!   • `asset["id"].is_number()` is true for i%5==0 (EC-001).
//!
//! Integer-form org-tagging encoding: because JSON integers cannot carry arbitrary
//! string prefixes, integer-form IDs encode org identity as:
//!   `(simple_hash_bytes(org_slug.as_bytes()) as u64) * 1_000_000 + (seed % 1_000) * 1_000 + index`
//! This value is deterministic and injective over distinct org slugs (with negligible
//! collision probability) while remaining a valid JSON number.
//!
//! Tombstone ID format: `dev-{org_slug}-{seed}-tomb-{n}` (BC-3.4.004 TV-3.4.004-07 /
//! EC-3.4.004-07 / Invariant 2: the prefix formula `dev-{slug}-{seed}` applies to ALL
//! record types including tombstones; seed is always present).
//!
//! Unregistered org detection (VP-121): org_id bytes all equal to 0xFF are treated as
//! an unregistered org and cause a panic with `GeneratorError::UnregisteredOrg`.
//! This is the sentinel value used in the test suite per TV-3.4.004-06.
//!
//! Gated: `#[cfg(feature = "fixture-gen")]`

#![allow(dead_code)]

use prism_core::SensorId;
use prism_dtu_common::generator::{
    default_page_size, seeded_rng, Archetype, FixtureSet, GenOpts, OrgId, Provenance,
};
use serde_json::{json, Value};

// ---------------------------------------------------------------------------
// Per-record diversity pools (P12-01)
//
// All pools are derived from the static fixture (fixtures/devices.json) to
// maintain realistic OT/IoT sensor diversity consistent with the demo data.
// Index selection uses the RNG-free `stable_offset` fold via
// `prism_dtu_common::stable_offset(record_key, seed.wrapping_add(N))` —
// no draws from the primary ChaCha20 stream, so
// INV-SECONDARY-RNG-STREAM-INDEPENDENCE-001 is preserved (BC-3.4.001).
// ---------------------------------------------------------------------------

/// Realistic OT/IoT OS names drawn from the static devices.json fixture pool.
const OS_NAME_POOL: &[&str] = &[
    "IOS",
    "IOS XE",
    "PAN-OS",
    "S7-1500",
    "Windows",
    "Windows Server",
    "Linux",
    "Logix 5000",
    "EcoStruxure",
    "Niagara",
    "AXIS OS",
    "ArubaOS",
    "RTU500",
    "DSM",
    "FactoryTalk",
    "APC NMC",
    "Tracer SC+",
    "Spectrum IQ",
    "HP FutureSmart",
    "CUCM",
    "RIVA",
    "KRC4",
    "iCLASS SE",
];

/// Realistic OT/IoT manufacturer names drawn from the static devices.json fixture pool.
const MANUFACTURER_POOL: &[&str] = &[
    "Siemens",
    "Cisco",
    "Rockwell",
    "Schneider",
    "Honeywell",
    "ABB",
    "Allen-Bradley",
    "Palo Alto",
    "Axis",
    "Aruba",
    "HP",
    "Dell",
    "Moxa",
    "Advantech",
    "Hikvision",
    "KUKA",
    "Trane",
    "Itron",
    "OSIsoft",
    "Baxter",
    "Endress+Hauser",
    "HID",
    "APC",
    "Synology",
];

/// Realistic risk score values (5..=95, drawn from static fixture distribution).
const RISK_SCORE_POOL: &[u64] = &[
    5, 8, 10, 15, 18, 20, 25, 30, 35, 38, 40, 45, 50, 55, 60, 62, 65, 70, 72, 78, 82, 85, 88, 90,
    95,
];

// ---------------------------------------------------------------------------
// Unregistered org sentinel (VP-121)
// ---------------------------------------------------------------------------

const UNREGISTERED_ORG_SENTINEL: [u8; 16] = [0xFF; 16];

fn check_registered(org_id: &OrgId) {
    if org_id.0 == UNREGISTERED_ORG_SENTINEL {
        panic!("GeneratorError::UnregisteredOrg — org_id [0xFF; 16] is not registered");
    }
}

// ---------------------------------------------------------------------------
// Public entrypoint
// ---------------------------------------------------------------------------

/// Generate a `FixtureSet` for the Armis sensor matching the given archetype.
///
/// # Parameters
/// - `org_id`: tenant namespace for org-tagged ID generation (BC-3.4.004).
/// - `org_slug`: short human-readable slug embedded in string-form asset IDs.
/// - `archetype`: one of the 8 defined deployment archetypes (BC-3.4.003).
/// - `opts`: seed, scale, time_anchor, and optional JSON Merge Patch overrides.
///
/// # Determinism
/// MUST NOT call `rand::thread_rng()` or `SystemTime::now()`. All entropy flows
/// through `seeded_rng(opts.seed, &org_id)` (BC-3.4.001 invariant 2).
///
/// # Returns
/// A `FixtureSet` whose `records` are `serde_json::Value` objects shaped as
/// `ArmisAsset` (for device archetypes) or `ArmisAlert` (for alert archetypes),
/// with `AqlResponse<SearchData>` envelope for `PaginationEdgeCases` (AC-003).
pub fn generate(org_id: OrgId, org_slug: &str, archetype: Archetype, opts: &GenOpts) -> FixtureSet {
    check_registered(&org_id);
    match archetype {
        Archetype::HealthyOtEnvironment => generate_healthy_ot(&org_id, org_slug, opts),
        Archetype::CompromisedEndpoint => generate_compromised_endpoint(&org_id, org_slug, opts),
        Archetype::AuthOutage => generate_auth_outage(&org_id, org_slug, opts),
        Archetype::LargeScale => generate_large_scale(&org_id, org_slug, opts),
        Archetype::PaginationEdgeCases => generate_pagination_edge_cases(&org_id, org_slug, opts),
        Archetype::SchemaDrift => generate_schema_drift(&org_id, org_slug, opts),
        Archetype::HighChurn => generate_high_churn(&org_id, org_slug, opts),
        Archetype::DormantTenant => generate_dormant_tenant(&org_id, org_slug, opts),
        // Non-exhaustive guard — new archetypes from future waves fall here until handled.
        _ => panic!("generate: unknown archetype variant — update Armis generator"),
    }
}

// ---------------------------------------------------------------------------
// Archetype dispatch — one private function per archetype (BC-3.4.003)
// ---------------------------------------------------------------------------

/// Build records for `HealthyOtEnvironment`: 50 assets, 5 alerts, all online/active.
fn generate_healthy_ot(org_id: &OrgId, org_slug: &str, opts: &GenOpts) -> FixtureSet {
    let n_assets = scale(50, opts.scale);
    let n_alerts = scale(5, opts.scale);

    let mut records = Vec::with_capacity(n_assets + n_alerts);
    for i in 0..n_assets {
        let status = if i % 2 == 0 { "online" } else { "active" };
        records.push(build_asset(
            org_slug,
            opts.seed,
            i,
            status,
            opts.time_anchor,
        ));
    }
    for i in 0..n_alerts {
        records.push(build_alert(org_slug, opts.seed, i, "LOW", opts.time_anchor));
    }

    FixtureSet {
        records,
        cursors: vec![],
        provenance: provenance(org_id.clone(), Archetype::HealthyOtEnvironment, opts, true),
    }
}

/// Build records for `CompromisedEndpoint`: 50 assets, 20 alerts, ≥3 severity HIGH/CRITICAL.
fn generate_compromised_endpoint(org_id: &OrgId, org_slug: &str, opts: &GenOpts) -> FixtureSet {
    let n_assets = scale(50, opts.scale);
    let n_alerts = scale(20, opts.scale);

    let mut records = Vec::with_capacity(n_assets + n_alerts);

    // ≥1 asset with lateral-movement indicator (BC-3.4.003)
    for i in 0..n_assets {
        let status = if i == 0 {
            "lateral-movement-detected"
        } else if i < 3 {
            "contained"
        } else {
            "compromised"
        };
        records.push(build_asset(
            org_slug,
            opts.seed,
            i,
            status,
            opts.time_anchor,
        ));
    }

    // ≥3 alerts with HIGH/CRITICAL severity
    for i in 0..n_alerts {
        let severity = if i < 3 {
            if i % 2 == 0 {
                "HIGH"
            } else {
                "CRITICAL"
            }
        } else {
            "MEDIUM"
        };
        records.push(build_alert(
            org_slug,
            opts.seed,
            i,
            severity,
            opts.time_anchor,
        ));
    }

    FixtureSet {
        records,
        cursors: vec![],
        provenance: provenance(org_id.clone(), Archetype::CompromisedEndpoint, opts, true),
    }
}

/// Build records for `AuthOutage`: 20 assets; first N records carry status_code=401.
///
/// N is read from `opts.overrides["auth_outage"]["recovery_after_calls"]`
/// (BC-3.4.003 invariant 6 / EC-3.4.003-06); default N = 1.
///
/// The N 401-tagged records are standard `ArmisAsset` records with an injected
/// `status_code=401` field, preserving the existing Armis schema shape and the
/// `asset.len()==20` invariant. Recovery is implicit: record[N] and beyond have
/// no status_code injection.
fn generate_auth_outage(org_id: &OrgId, org_slug: &str, opts: &GenOpts) -> FixtureSet {
    let n_assets = scale(20, opts.scale);

    // BC-3.4.003 invariant 6 / EC-3.4.003-06: read recovery_after_calls from overrides.
    // Default N = 1 (one leading 401-tagged asset before recovery).
    let recovery_after_calls = opts
        .overrides
        .get("auth_outage")
        .and_then(|v| v.get("recovery_after_calls"))
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(1) as usize;

    let mut records = Vec::with_capacity(n_assets);

    for i in 0..n_assets {
        let mut asset = build_asset(org_slug, opts.seed, i, "online", opts.time_anchor);
        if i < recovery_after_calls {
            // BC-3.4.003 TV-3.4.003-03 / EC-3.4.003-06: inject 401 on first N records
            asset["status_code"] = json!(401i64);
        }
        records.push(asset);
    }

    FixtureSet {
        records,
        cursors: vec![],
        provenance: provenance(org_id.clone(), Archetype::AuthOutage, opts, true),
    }
}

/// Build records for `LargeScale`: 10,000 assets, 500 alerts.
fn generate_large_scale(org_id: &OrgId, org_slug: &str, opts: &GenOpts) -> FixtureSet {
    let n_assets = scale(10_000, opts.scale);
    let n_alerts = scale(500, opts.scale);

    let mut records = Vec::with_capacity(n_assets + n_alerts);
    for i in 0..n_assets {
        let status = if i % 2 == 0 { "online" } else { "active" };
        records.push(build_asset(
            org_slug,
            opts.seed,
            i,
            status,
            opts.time_anchor,
        ));
    }
    for i in 0..n_alerts {
        records.push(build_alert(
            org_slug,
            opts.seed,
            i,
            "MEDIUM",
            opts.time_anchor,
        ));
    }

    FixtureSet {
        records,
        cursors: vec![],
        provenance: provenance(org_id.clone(), Archetype::LargeScale, opts, true),
    }
}

/// Build records for `PaginationEdgeCases`: page_size×3 individual AQL-envelope records,
/// exactly 3 cursors (AC-003).
///
/// Each record is wrapped in an `AqlResponse<SearchData>` envelope containing exactly one
/// asset. This satisfies:
///   • `fs.records.len() == page_size * 3` (300 for Armis)
///   • each record has `status`, `data`, `data.results` keys (AQL envelope shape)
///   • `fs.cursors.len() == 3` (one cursor per logical page)
fn generate_pagination_edge_cases(org_id: &OrgId, org_slug: &str, opts: &GenOpts) -> FixtureSet {
    let page_size = default_page_size(SensorId::from("armis"));
    let total_assets = page_size * 3;

    let mut records = Vec::with_capacity(total_assets);
    let mut cursors = Vec::with_capacity(3);

    // Generate 3 cursors — one per page boundary
    for page in 0..3usize {
        cursors.push(format!(
            "cursor-{}-{}-page{}",
            org_slug,
            opts.seed,
            page + 1
        ));
    }

    // Each asset becomes a single-item AQL envelope record
    for i in 0..total_assets {
        let asset = build_asset(org_slug, opts.seed, i, "online", opts.time_anchor);
        // Determine which page this asset belongs to (for cursor reference)
        let page = i / page_size;
        let cursor = cursors[page.min(2)].clone();
        let envelope = build_aql_envelope(vec![asset], Some(total_assets as i64), Some(cursor));
        records.push(envelope);
    }

    FixtureSet {
        records,
        cursors,
        provenance: provenance(org_id.clone(), Archetype::PaginationEdgeCases, opts, true),
    }
}

/// Build records for `SchemaDrift`: 30 assets; records[0] omits required "id" field;
/// `provenance.schema_valid = false` (BC-3.4.002 / AC-002).
fn generate_schema_drift(org_id: &OrgId, org_slug: &str, opts: &GenOpts) -> FixtureSet {
    let n_assets = scale(30, opts.scale);
    let mut records = Vec::with_capacity(n_assets);

    // records[0]: drifted — omits required "id" field (BC-3.4.003 invariant 4)
    // P1-02: timestamps anchor-derived like every other record — the drift is
    // the missing "id" key, not the timestamp era.
    let drift_id = format!("drift-{}-{}-0", org_slug, opts.seed);
    let (drift_last_dt, drift_first_dt) =
        derive_seen_window(&drift_id, opts.seed, opts.time_anchor);
    // P12-01: the schema drift is ONLY the missing "id" key. The other columns
    // (os_name, risk_score, manufacturer) should be populated — a device without
    // an ID still has an OS, manufacturer, and risk level. Null on these columns
    // is not part of the schema-drift intent and would pollute the live demo.
    let drift_os = derive_os_name(&drift_id, opts.seed);
    let drift_mfr = derive_manufacturer(&drift_id, opts.seed);
    let drift_risk = derive_risk_score(&drift_id, opts.seed);
    // P2-01: additive flat snake_case TOML-parity keys (see build_asset).
    // The drift is the missing camelCase "id" key — the flat keys are
    // unaffected by the schema-drift semantics.
    let drifted = json!({
        // "id" intentionally omitted — schema-drifted record (BC-3.4.003 invariant 4)
        "asset_id": drift_id.clone(),
        "device_id": drift_id,
        "name": format!("drifted-device-{}", org_slug),
        "title": format!("Drifted Device for {}", org_slug),
        "type": "Unknown",
        "status": "online",
        "lastSeen": format_ts(drift_last_dt),
        "last_seen": format_ts(drift_last_dt),
        "firstSeen": format_ts(drift_first_dt),
        "first_seen": format_ts(drift_first_dt),
        "ipAddress": null,
        "ip_address": null,
        "macAddress": null,
        "mac_address": null,
        "manufacturer": drift_mfr,
        "model": null,
        "firmwareVersion": null,
        "operatingSystem": drift_os,
        "os_name": drift_os,
        "riskLevel": drift_risk,
        "risk_score": drift_risk,
        "site": null,
        "zone": null
    });
    records.push(drifted);

    // records[1..]: conformant assets
    for i in 1..n_assets {
        records.push(build_asset(
            org_slug,
            opts.seed,
            i,
            "online",
            opts.time_anchor,
        ));
    }

    FixtureSet {
        records,
        cursors: vec![],
        provenance: provenance(org_id.clone(), Archetype::SchemaDrift, opts, false),
    }
}

/// Build records for `HighChurn`: 200 assets, ≥20 tombstones with `deleted_at` present.
fn generate_high_churn(org_id: &OrgId, org_slug: &str, opts: &GenOpts) -> FixtureSet {
    let n_assets = scale(200, opts.scale);
    // Exactly 20 tombstones (meets ≥20 requirement at scale=1.0)
    let n_tombstones = scale(20, opts.scale).max(20);
    let n_normal = n_assets.saturating_sub(n_tombstones);

    let mut records = Vec::with_capacity(n_assets);
    for i in 0..n_normal {
        records.push(build_asset(
            org_slug,
            opts.seed,
            i,
            "online",
            opts.time_anchor,
        ));
    }
    for t in 0..n_tombstones {
        records.push(build_tombstone(org_slug, opts.seed, t, opts.time_anchor));
    }

    FixtureSet {
        records,
        cursors: vec![],
        provenance: provenance(org_id.clone(), Archetype::HighChurn, opts, true),
    }
}

/// Build records for `DormantTenant`: 0 records, 0 cursors (scale-invariant per BC-3.4.003).
fn generate_dormant_tenant(org_id: &OrgId, org_slug: &str, opts: &GenOpts) -> FixtureSet {
    // Consume to satisfy borrow checker
    let _ = seeded_rng(opts.seed, org_id);
    let _ = org_slug;
    FixtureSet {
        records: vec![],
        cursors: vec![],
        provenance: provenance(org_id.clone(), Archetype::DormantTenant, opts, true),
    }
}

// ---------------------------------------------------------------------------
// Record builders
// ---------------------------------------------------------------------------

/// Pick a per-record OS name from `OS_NAME_POOL` using the RNG-free stable_offset
/// fold (P12-01 — seed offset 2 avoids collision with the seen-window derivations
/// at offsets 0 and 1).
fn derive_os_name(record_key: &str, seed: u64) -> &'static str {
    let idx = prism_dtu_common::stable_offset(record_key, seed.wrapping_add(2)) as usize
        % OS_NAME_POOL.len();
    OS_NAME_POOL[idx]
}

/// Pick a per-record manufacturer from `MANUFACTURER_POOL` using the RNG-free
/// stable_offset fold (P12-01 — seed offset 3 avoids collision with offsets 0..2).
fn derive_manufacturer(record_key: &str, seed: u64) -> &'static str {
    let idx = prism_dtu_common::stable_offset(record_key, seed.wrapping_add(3)) as usize
        % MANUFACTURER_POOL.len();
    MANUFACTURER_POOL[idx]
}

/// Pick a per-record risk score from `RISK_SCORE_POOL` using the RNG-free
/// stable_offset fold (P12-01 — seed offset 4 avoids collision with offsets 0..3).
fn derive_risk_score(record_key: &str, seed: u64) -> u64 {
    let idx = prism_dtu_common::stable_offset(record_key, seed.wrapping_add(4)) as usize
        % RISK_SCORE_POOL.len();
    RISK_SCORE_POOL[idx]
}

/// Derive a per-record `(lastSeen, firstSeen)` pair from the time anchor
/// (review-2026-06-10 P1-02).
///
/// - `lastSeen`: 0..7 days (in minutes) before `time_anchor`, stable per
///   `(record_key, seed)`.
/// - `firstSeen`: 7..=90 days before `lastSeen` — always strictly earlier.
///
/// RNG-free (`stable_offset` fold) — the primary ChaCha20 stream is never
/// consulted, so per-record variance cannot perturb other generated values
/// (INV-SECONDARY-RNG-STREAM-INDEPENDENCE-001 / BC-3.4.001).
fn derive_seen_window(
    record_key: &str,
    seed: u64,
    time_anchor: chrono::DateTime<chrono::Utc>,
) -> (chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>) {
    let minutes_before = (prism_dtu_common::stable_offset(record_key, seed) % 10_080) as i64;
    let last_seen = time_anchor - chrono::Duration::minutes(minutes_before);
    let days_before =
        7 + (prism_dtu_common::stable_offset(record_key, seed.wrapping_add(1)) % 84) as i64;
    let first_seen = last_seen - chrono::Duration::days(days_before);
    (last_seen, first_seen)
}

/// Format a derived timestamp in the Armis fixture shape (`%Y-%m-%dT%H:%M:%SZ`).
fn format_ts(ts: chrono::DateTime<chrono::Utc>) -> String {
    ts.format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

/// Build a single `ArmisAsset` record as a JSON Value.
///
/// `id_index`: 0-based position; every 5th record (i % 5 == 0) uses integer-form `id` (EC-001).
///
/// IMPORTANT: Both `id` and `asset_id` fields are always present:
/// - `id`: integer for i%5==0, string "dev-{slug}-{seed}-{i}" otherwise (EC-001 / BC-3.4.002)
/// - `asset_id`: always string "dev-{slug}-{seed}-{i}" (VP-120 org-tagging invariant)
///
/// `primary_id()` in tests checks `asset_id` first, so VP-120 passes regardless of
/// `id` type. `asset["id"].is_number()` for i%5==0 satisfies EC-001.
fn build_asset(
    org_slug: &str,
    seed: u64,
    id_index: usize,
    status: &str,
    time_anchor: chrono::DateTime<chrono::Utc>,
) -> Value {
    let string_id = format!("dev-{}-{}-{}", org_slug, seed, id_index);
    let id: Value = if id_index.is_multiple_of(5) {
        // EC-001: integer-form ArmisId for every 5th record
        json!(integer_asset_id(org_slug, seed, id_index))
    } else {
        json!(string_id.clone())
    };

    // P1-02 (review 2026-06-10): per-record timestamps derive from time_anchor
    // minus a seeded RNG-free stable_offset fold so AQL time-window queries can
    // discriminate between records. lastSeen: 0..7 days before the anchor;
    // firstSeen: 7..=90 days before lastSeen (strictly earlier). The fold draws
    // nothing from the ChaCha20 stream (INV-SECONDARY-RNG-STREAM-INDEPENDENCE-001).
    let (last_dt, first_dt) = derive_seen_window(&string_id, seed, time_anchor);
    let last_seen = format_ts(last_dt);
    let first_seen = format_ts(first_dt);
    let ip_address = format!(
        "10.{}.{}.{}",
        (id_index / 65536) % 256,
        (id_index / 256) % 256,
        id_index % 256
    );
    let mac_address = format!(
        "AA:BB:CC:{:02X}:{:02X}:{:02X}",
        (id_index / 65536) % 256,
        (id_index / 256) % 256,
        id_index % 256
    );

    // P12-01 (review-2026-06-10 cascade pass-12, HIGH / demo-critical):
    // os_name / operatingSystem and risk_score / riskLevel were null for EVERY
    // generated device — the demo harness uses new_with_seed which serves
    // generator output verbatim, so the live demo showed 100% NULL for both
    // columns. manufacturer was hardcoded "Siemens" for all records.
    // Fix: derive deterministic non-null values from per-record pools using the
    // same RNG-free stable_offset fold used for timestamps (seed offsets 2, 3, 4
    // avoid collision with the seen-window fold at offsets 0 and 1).
    let os_name = derive_os_name(&string_id, seed);
    let manufacturer = derive_manufacturer(&string_id, seed);
    let risk_score = derive_risk_score(&string_id, seed);

    // P2-01 (review 2026-06-10 cascade pass-2, SAP-2): in addition to the
    // camelCase real-API keys, emit ADDITIVE flat snake_case TOML-parity keys
    // (device_id, last_seen, first_seen, ip_address, mac_address, os_name,
    // risk_score) mirroring the camelCase values exactly — armis.sensor.toml
    // extracts exact-name flat keys (column_mapping.rs raw.get), so without
    // these the seeded path silently normalized every column to NULL.
    // CrowdStrike F4 additive pattern: API-shape fidelity kept, flat keys added.
    json!({
        "id": id,
        "asset_id": string_id.clone(),
        "device_id": string_id,
        "name": format!("device-{}-{}", org_slug, id_index),
        "title": format!("Device {} for {}", id_index, org_slug),
        "type": "IoT Device",
        "status": status,
        "lastSeen": last_seen.clone(),
        "last_seen": last_seen,
        "firstSeen": first_seen.clone(),
        "first_seen": first_seen,
        "ipAddress": ip_address.clone(),
        "ip_address": ip_address,
        "macAddress": mac_address.clone(),
        "mac_address": mac_address,
        "manufacturer": manufacturer,
        "model": null,
        "firmwareVersion": null,
        "operatingSystem": os_name,
        "os_name": os_name,
        "riskLevel": risk_score,
        "risk_score": risk_score,
        "site": format!("site-{}", id_index % 5),
        "zone": null
    })
}

/// Build a tombstone `ArmisAsset` record.
///
/// Tombstone ID format: `dev-{org_slug}-{seed}-tomb-{n}` (BC-3.4.004 tombstone row /
/// EC-3.4.004-07 / TV-3.4.004-07). Invariant 2: the prefix formula `dev-{slug}-{seed}`
/// applies consistently to ALL record types (assets and tombstones alike).
/// Sibling generators CrowdStrike (`make_tombstone`) and Claroty also include the seed.
fn build_tombstone(
    org_slug: &str,
    seed: u64,
    tomb_index: usize,
    time_anchor: chrono::DateTime<chrono::Utc>,
) -> Value {
    // Format: "dev-{org_slug}-{seed}-tomb-{n}" — seed is required per BC-3.4.004 Invariant 2
    let id = format!("dev-{}-{}-tomb-{}", org_slug, seed, tomb_index);
    // P1-02: anchor-derived per-record timestamps (RNG-free stable_offset).
    // deleted_at = lastSeen + 1 hour, clamped to the anchor (the device
    // disappeared shortly after last sight, never "in the future").
    let (last_dt, first_dt) = derive_seen_window(&id, seed, time_anchor);
    let deleted_dt = (last_dt + chrono::Duration::hours(1)).min(time_anchor);
    let last_seen = format_ts(last_dt);
    let first_seen = format_ts(first_dt);
    let deleted_at = format_ts(deleted_dt);
    // P12-01: derive non-null os_name, manufacturer, risk_score for tombstones
    // using the same pool + RNG-free stable_offset pattern as build_asset.
    // Tombstone devices are real assets that disappeared — they had OS names,
    // manufacturers, and risk scores before deletion.
    let os_name = derive_os_name(&id, seed);
    let manufacturer = derive_manufacturer(&id, seed);
    let risk_score = derive_risk_score(&id, seed);

    // P2-01: additive flat snake_case TOML-parity keys (see build_asset).
    json!({
        "id": id,
        "asset_id": id.clone(),
        "device_id": id.clone(),
        "name": format!("tombstone-{}-{}", org_slug, tomb_index),
        "title": format!("Tombstone {} for {}", tomb_index, org_slug),
        "type": "IoT Device",
        "status": "tombstone",
        "lastSeen": last_seen.clone(),
        "last_seen": last_seen,
        "firstSeen": first_seen.clone(),
        "first_seen": first_seen,
        "ipAddress": null,
        "ip_address": null,
        "macAddress": null,
        "mac_address": null,
        "manufacturer": manufacturer,
        "model": null,
        "firmwareVersion": null,
        "operatingSystem": os_name,
        "os_name": os_name,
        "riskLevel": risk_score,
        "risk_score": risk_score,
        "site": null,
        "zone": null,
        "deleted_at": deleted_at,
        "_seed": seed
    })
}

/// Build a single `ArmisAlert` record as a JSON Value.
///
/// `alertId` (integer) incorporates the org_slug hash to ensure disjoint ID sets (VP-119).
/// `alert_id` (string) always contains org_slug (VP-120 for alerts).
fn build_alert(
    org_slug: &str,
    seed: u64,
    id_index: usize,
    severity: &str,
    time_anchor: chrono::DateTime<chrono::Utc>,
) -> Value {
    // alertId: org-specific integer to ensure disjoint sets between orgs (VP-119)
    let slug_hash = simple_hash_bytes(org_slug.as_bytes()) as i64;
    let alert_id_num: i64 = slug_hash
        .saturating_mul(1_000_000)
        .saturating_add((seed as i64).saturating_mul(1_000))
        .saturating_add(id_index as i64);
    let alert_id_str = format!("alert-{}-{}-{}", org_slug, seed, id_index);

    // P1-02: alert `time` derives from time_anchor minus a seeded RNG-free
    // stable_offset fold (0..7 days); lastAlertUpdateTime keeps the +5min
    // update lag of the previous static values, clamped to the anchor.
    let minutes_before = (prism_dtu_common::stable_offset(&alert_id_str, seed) % 10_080) as i64;
    let time_dt = time_anchor - chrono::Duration::minutes(minutes_before);
    let update_dt = (time_dt + chrono::Duration::minutes(5)).min(time_anchor);
    let time = format_ts(time_dt);
    let last_update = format_ts(update_dt);
    let title = format!("Alert {} for {}", id_index, org_slug);
    let policy_id = format!("policy-{}-{}", org_slug, id_index % 10);

    // P2-01 (review 2026-06-10 cascade pass-2, SAP-2): additive flat snake_case
    // TOML-parity keys mirroring the real-API values — armis.sensor.toml alerts
    // columns are name/policy_name/device_id/created_at/updated_at, extracted
    // by exact-name flat raw.get (column_mapping.rs). `device_id` is the STRING
    // form per the TOML column type and links to the seeded device pool
    // (build_asset string id for the same index — CrowdStrike CS-01 pattern;
    // every archetype generates at least as many assets as alerts).
    json!({
        "alertId": alert_id_num,
        "alert_id": alert_id_str,
        "policyId": policy_id.clone(),
        "policy_name": policy_id,
        "title": title.clone(),
        "name": title,
        "status": "UNHANDLED",
        "severity": severity,
        "type": "Policy Violation",
        "time": time.clone(),
        "created_at": time,
        "lastAlertUpdateTime": last_update.clone(),
        "updated_at": last_update,
        "deviceId": id_index as i64,
        "device_id": format!("dev-{}-{}-{}", org_slug, seed, id_index),
        "description": format!("Detected anomaly {} for org {}", id_index, org_slug),
        "remediation": null
    })
}

/// Build an `AqlResponse<SearchData>` envelope wrapping a slice of asset records (AC-003).
///
/// Shape from `.references/schemas/armis/types.rs`:
/// - `status`: `Option<i32>` — HTTP status code
/// - `message`: `Option<String>` — human-readable message
/// - `data`: `Option<SearchData>` — `{ results: Vec<T>, total: Option<i64>, sample: Option<Value> }`
fn build_aql_envelope(records: Vec<Value>, total: Option<i64>, _cursor: Option<String>) -> Value {
    json!({
        "status": 200i32,
        "message": "OK",
        "data": {
            "results": records,
            "total": total,
            "sample": null
        }
    })
}

/// Compute the integer-form asset ID for the given org_slug, seed, and index (EC-001).
///
/// Encoding:
///   `(simple_hash_bytes(org_slug.as_bytes()) as u64) * 1_000_000 + (seed % 1_000) * 1_000 + index`
///
/// The `simple_hash_bytes(org_slug)` component encodes org identity in the upper digits.
/// This is deterministic, and distinct slugs produce distinct hash prefixes (low collision
/// probability). The org_slug can be recovered conceptually by comparing upper digit ranges.
fn integer_asset_id(org_slug: &str, seed: u64, index: usize) -> u64 {
    let slug_hash = simple_hash_bytes(org_slug.as_bytes()) as u64;
    let seed_contrib = (seed % 1_000) * 1_000;
    slug_hash
        .saturating_mul(1_000_000)
        .saturating_add(seed_contrib)
        .saturating_add(index as u64)
}

/// Compute a simple polynomial hash of a byte slice (no external dep — pure arithmetic).
///
/// Uses a djb2-inspired polynomial: `hash = hash.wrapping_mul(31).wrapping_add(byte)`.
/// This is deterministic, stable, and sufficient for fixture-ID generation.
fn simple_hash_bytes(data: &[u8]) -> u32 {
    let mut hash: u32 = 5381;
    for &byte in data {
        hash = hash.wrapping_mul(31).wrapping_add(byte as u32);
    }
    hash
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Apply scale factor: `floor(baseline * scale)`, minimum 0.
fn scale(baseline: usize, factor: f64) -> usize {
    ((baseline as f64) * factor).floor() as usize
}

/// Construct a `Provenance` value.
fn provenance(
    org_id: OrgId,
    archetype: Archetype,
    opts: &GenOpts,
    schema_valid: bool,
) -> Provenance {
    Provenance {
        org_id,
        sensor_id: SensorId::from("armis"),
        archetype,
        seed: opts.seed,
        schema_valid,
    }
}
