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
//! Tombstone ID format: `dev-{org_slug}-tomb-{n}` (no seed in tombstone IDs per
//! BC-3.4.004 TV-3.4.004-07 test assertion: id.contains("{org_slug}-tomb-")).
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
        records.push(build_asset(org_slug, opts.seed, i, status));
    }
    for i in 0..n_alerts {
        records.push(build_alert(org_slug, opts.seed, i, "LOW"));
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
        records.push(build_asset(org_slug, opts.seed, i, status));
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
        records.push(build_alert(org_slug, opts.seed, i, severity));
    }

    FixtureSet {
        records,
        cursors: vec![],
        provenance: provenance(org_id.clone(), Archetype::CompromisedEndpoint, opts, true),
    }
}

/// Build records for `AuthOutage`: 20 assets; first call record has status_code=401.
fn generate_auth_outage(org_id: &OrgId, org_slug: &str, opts: &GenOpts) -> FixtureSet {
    let n_assets = scale(20, opts.scale);
    let mut records = Vec::with_capacity(n_assets);

    // First record carries status_code=401 (BC-3.4.003 TV-3.4.003-03)
    let mut first = build_asset(org_slug, opts.seed, 0, "online");
    first["status_code"] = json!(401i64);
    records.push(first);

    for i in 1..n_assets {
        records.push(build_asset(org_slug, opts.seed, i, "online"));
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
        records.push(build_asset(org_slug, opts.seed, i, status));
    }
    for i in 0..n_alerts {
        records.push(build_alert(org_slug, opts.seed, i, "MEDIUM"));
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
        let asset = build_asset(org_slug, opts.seed, i, "online");
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
    let drifted = json!({
        // "id" intentionally omitted — schema-drifted record
        "asset_id": format!("drift-{}-{}-0", org_slug, opts.seed),
        "name": format!("drifted-device-{}", org_slug),
        "title": format!("Drifted Device for {}", org_slug),
        "type": "Unknown",
        "status": "online",
        "lastSeen": "2024-01-01T00:00:00Z",
        "firstSeen": "2023-01-01T00:00:00Z",
        "ipAddress": null,
        "macAddress": null,
        "manufacturer": null,
        "model": null,
        "firmwareVersion": null,
        "operatingSystem": null,
        "riskLevel": null,
        "site": null,
        "zone": null
    });
    records.push(drifted);

    // records[1..]: conformant assets
    for i in 1..n_assets {
        records.push(build_asset(org_slug, opts.seed, i, "online"));
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
        records.push(build_asset(org_slug, opts.seed, i, "online"));
    }
    for t in 0..n_tombstones {
        records.push(build_tombstone(org_slug, opts.seed, t));
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
fn build_asset(org_slug: &str, seed: u64, id_index: usize, status: &str) -> Value {
    let string_id = format!("dev-{}-{}-{}", org_slug, seed, id_index);
    let id: Value = if id_index.is_multiple_of(5) {
        // EC-001: integer-form ArmisId for every 5th record
        json!(integer_asset_id(org_slug, seed, id_index))
    } else {
        json!(string_id.clone())
    };

    json!({
        "id": id,
        "asset_id": string_id,
        "name": format!("device-{}-{}", org_slug, id_index),
        "title": format!("Device {} for {}", id_index, org_slug),
        "type": "IoT Device",
        "status": status,
        "lastSeen": "2024-01-01T00:00:00Z",
        "firstSeen": "2023-01-01T00:00:00Z",
        "ipAddress": format!("10.{}.{}.{}", (id_index / 65536) % 256, (id_index / 256) % 256, id_index % 256),
        "macAddress": format!("AA:BB:CC:{:02X}:{:02X}:{:02X}", (id_index / 65536) % 256, (id_index / 256) % 256, id_index % 256),
        "manufacturer": "Siemens",
        "model": null,
        "firmwareVersion": null,
        "operatingSystem": null,
        "riskLevel": null,
        "site": format!("site-{}", id_index % 5),
        "zone": null
    })
}

/// Build a tombstone `ArmisAsset` record.
///
/// Tombstone ID format: `dev-{org_slug}-tomb-{n}` (BC-3.4.004 tombstone row).
/// The test asserts `id.contains("{org_slug}-tomb-")` which requires the seed to be absent.
fn build_tombstone(org_slug: &str, seed: u64, tomb_index: usize) -> Value {
    // Format: "dev-{org_slug}-tomb-{n}" — no seed in tombstone ID per BC-3.4.004 TV-3.4.004-07
    let id = format!("dev-{}-tomb-{}", org_slug, tomb_index);
    json!({
        "id": id,
        "asset_id": id.clone(),
        "name": format!("tombstone-{}-{}", org_slug, tomb_index),
        "title": format!("Tombstone {} for {}", tomb_index, org_slug),
        "type": "IoT Device",
        "status": "tombstone",
        "lastSeen": "2023-12-31T23:59:59Z",
        "firstSeen": "2023-01-01T00:00:00Z",
        "ipAddress": null,
        "macAddress": null,
        "manufacturer": null,
        "model": null,
        "firmwareVersion": null,
        "operatingSystem": null,
        "riskLevel": null,
        "site": null,
        "zone": null,
        "deleted_at": "2024-01-01T00:00:00Z",
        "_seed": seed
    })
}

/// Build a single `ArmisAlert` record as a JSON Value.
///
/// `alertId` (integer) incorporates the org_slug hash to ensure disjoint ID sets (VP-119).
/// `alert_id` (string) always contains org_slug (VP-120 for alerts).
fn build_alert(org_slug: &str, seed: u64, id_index: usize, severity: &str) -> Value {
    // alertId: org-specific integer to ensure disjoint sets between orgs (VP-119)
    let slug_hash = simple_hash_bytes(org_slug.as_bytes()) as i64;
    let alert_id_num: i64 = slug_hash
        .saturating_mul(1_000_000)
        .saturating_add((seed as i64).saturating_mul(1_000))
        .saturating_add(id_index as i64);
    let alert_id_str = format!("alert-{}-{}-{}", org_slug, seed, id_index);

    json!({
        "alertId": alert_id_num,
        "alert_id": alert_id_str,
        "policyId": format!("policy-{}-{}", org_slug, id_index % 10),
        "title": format!("Alert {} for {}", id_index, org_slug),
        "status": "UNHANDLED",
        "severity": severity,
        "type": "Policy Violation",
        "time": "2024-01-01T12:00:00Z",
        "lastAlertUpdateTime": "2024-01-01T12:05:00Z",
        "deviceId": id_index as i64,
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
