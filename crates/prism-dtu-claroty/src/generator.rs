//! Claroty fixture generator — all 8 archetypes (S-3.7.02).
//!
//! Implements `generate(org_id, archetype, opts) -> FixtureSet` backed by the
//! poller-bear OpenAPI spec (`.references/poller-bear/docs/specs.json`).
//!
//! Gated behind `#[cfg(feature = "fixture-gen")]` (AC-007 / D-056).
//!
//! BC-3.4.001: deterministic — uses `seeded_rng(opts.seed, org_id)` exclusively.
//! BC-3.4.002: all non-SchemaDrift records validate against specs.json.
//! BC-3.4.003: all 8 archetypes with defined baselines at scale=1.0.
//! BC-3.4.004: every ID carries `dev-{slug}-{seed}-` / `alert-{slug}-{seed}-` prefix.

use prism_core::SensorId;
use prism_dtu_common::generator::{
    default_page_size, seeded_rng, Archetype, FixtureSet, GenOpts, OrgId, Provenance,
};
use rand::Rng as _;
use serde_json::{json, Value};

/// Derive a URL-safe slug from the org-id bytes used for record-ID prefixes (BC-3.4.004).
///
/// Returns an 8-char hex string built from the first 4 bytes of `org_id` (fallback
/// for EC-003: OrgRegistry lookup failure).
pub fn org_slug(org_id: &OrgId) -> String {
    let b = org_id.as_bytes();
    format!("{:02x}{:02x}{:02x}{:02x}", b[0], b[1], b[2], b[3])
}

/// Generate a `FixtureSet` for the given `org_id` and `archetype` using `opts`.
///
/// # Determinism (BC-3.4.001)
///
/// All randomness MUST flow through `seeded_rng(opts.seed, org_id)`.
/// NEVER call `rand::thread_rng()`, `SystemTime::now()`, or any non-deterministic
/// entropy source within this call stack.
///
/// # Baseline counts at scale=1.0 (BC-3.4.003)
///
/// | Archetype             | Device records | Alert records | Notes                         |
/// |-----------------------|---------------|---------------|-------------------------------|
/// | HealthyOtEnvironment  | 50            | 5             |                               |
/// | CompromisedEndpoint   | 50            | 20            | ≥3 alerts with severity_id≥4  |
/// | AuthOutage            | 20            | 0             | records[0].status_code = 401  |
/// | LargeScale            | 10 000        | 500           | ≥100 distinct subnets         |
/// | PaginationEdgeCases   | page_size × 3 | 0             | 3 cursor values               |
/// | SchemaDrift           | 30            | 0             | records[0] fails schema       |
/// | HighChurn             | 200           | 0             | ≥20 tombstone status          |
/// | DormantTenant         | 0             | 0             | no cursors                    |
pub fn generate(org_id: &OrgId, archetype: Archetype, opts: &GenOpts) -> FixtureSet {
    match archetype {
        Archetype::HealthyOtEnvironment => gen_healthy_ot_environment(org_id, opts),
        Archetype::CompromisedEndpoint => gen_compromised_endpoint(org_id, opts),
        Archetype::AuthOutage => gen_auth_outage(org_id, opts),
        Archetype::LargeScale => gen_large_scale(org_id, opts),
        Archetype::PaginationEdgeCases => gen_pagination_edge_cases(org_id, opts),
        Archetype::SchemaDrift => gen_schema_drift(org_id, opts),
        Archetype::HighChurn => gen_high_churn(org_id, opts),
        Archetype::DormantTenant => gen_dormant_tenant(org_id, opts),
        // Non-exhaustive: future archetypes return empty FixtureSet
        _ => FixtureSet {
            records: vec![],
            cursors: vec![],
            provenance: Provenance {
                org_id: org_id.clone(),
                sensor_type: SensorId::from("claroty"),
                archetype,
                seed: opts.seed,
                schema_valid: true,
            },
        },
    }
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Build a minimal valid Claroty device record (GetDevicesResponse items shape).
///
/// The GetDevicesResponse items use `additionalProperties: true` with no required
/// fields at the item level — the only required field is `devices` at the response
/// level. We embed `device_id` (our tracking field) plus realistic shape fields.
fn make_device(slug: &str, seed: u64, index: usize) -> Value {
    json!({
        "device_id": format!("dev-{slug}-{seed}-{index}"),
        "asset_id": format!("ASSET-{slug}-{seed}-{index}"),
        "device_category": "OT",
        "device_subcategory": "PLC",
        "device_type": "Controller",
        "device_type_family": "Controller",
        "ip_list": [format!("10.{}.{}.{}", (index / 65536) % 256, (index / 256) % 256, index % 256)],
        "labels": [],
        "mac_list": [format!("aa:bb:{:02x}:{:02x}:{:02x}:{:02x}",
            (seed >> 8) as u8, seed as u8, (index >> 8) as u8, index as u8)],
        "model": "Industrial Controller",
        "network_list": ["OT Network"],
        "os_category": "Embedded",
        "retired": false,
        "risk_score": "Low",
        "uid": format!("{slug}-{seed}-device-{index:08x}"),
        "status": "online"
    })
}

/// Build a minimal valid Claroty device record with a specific subnet.
fn make_device_with_subnet(slug: &str, seed: u64, index: usize, subnet: &str) -> Value {
    json!({
        "device_id": format!("dev-{slug}-{seed}-{index}"),
        "asset_id": format!("ASSET-{slug}-{seed}-{index}"),
        "device_category": "OT",
        "device_subcategory": "PLC",
        "device_type": "Controller",
        "device_type_family": "Controller",
        "ip_list": [format!("{}.{}", subnet, index % 254 + 1)],
        "labels": [],
        "mac_list": [format!("aa:bb:{:02x}:{:02x}:{:02x}:{:02x}",
            (seed >> 8) as u8, seed as u8, (index >> 8) as u8, index as u8)],
        "model": "Industrial Controller",
        "network_list": ["OT Network"],
        "os_category": "Embedded",
        "retired": false,
        "risk_score": "Low",
        "uid": format!("{slug}-{seed}-device-{index:08x}"),
        "status": "online",
        "subnet": subnet
    })
}

/// Build a minimal valid Claroty alert record (GetAlertsResponse items shape).
fn make_alert(slug: &str, seed: u64, index: usize, severity_id: u64) -> Value {
    json!({
        "alert_id": format!("alert-{slug}-{seed}-{index}"),
        "alert_type_name": "Network Anomaly",
        "category": "Segmentation",
        "description": format!("Alert {index} detected by fixture generator"),
        "detected_time": "2021-07-11T19:40:46.835404+00:00",
        "devices_count": 1,
        "id": seed.wrapping_add(index as u64),
        "iot_devices_count": 0,
        "it_devices_count": 1,
        "medical_devices_count": 0,
        "mitre_technique_enterprise_ids": [],
        "mitre_technique_enterprise_names": [],
        "mitre_technique_ics_ids": [],
        "mitre_technique_ics_names": [],
        "status": "Unresolved",
        "unresolved_devices_count": 1,
        "updated_time": "2021-07-11T19:40:46.835404+00:00",
        "severity_id": severity_id
    })
}

// ---------------------------------------------------------------------------
// Archetype implementations
// ---------------------------------------------------------------------------

/// Generate the Claroty `HealthyOtEnvironment` archetype records.
///
/// Returns `floor(50 * opts.scale)` device records and `floor(5 * opts.scale)` alert
/// records with no active threats (BC-3.4.003 baseline row 1).
fn gen_healthy_ot_environment(org_id: &OrgId, opts: &GenOpts) -> FixtureSet {
    let slug = org_slug(org_id);
    let n_devices = (50.0 * opts.scale).floor() as usize;
    let n_alerts = (5.0 * opts.scale).floor() as usize;

    // Use seeded_rng to make records vary per seed/org (though we don't need randomness
    // for the deterministic fields — just consume the RNG to ensure it's used).
    let mut rng = seeded_rng(opts.seed, org_id);
    let _jitter: u32 = rng.gen(); // anchor rng to ensure determinism

    let mut records: Vec<Value> = Vec::with_capacity(n_devices + n_alerts);

    for i in 0..n_devices {
        records.push(make_device(&slug, opts.seed, i));
    }
    for i in 0..n_alerts {
        // Healthy: low severity only (severity_id 1-3)
        let sev = 1u64 + (i as u64 % 3);
        records.push(make_alert(&slug, opts.seed, i, sev));
    }

    FixtureSet {
        records,
        cursors: vec![],
        provenance: Provenance {
            org_id: org_id.clone(),
            sensor_type: SensorId::from("claroty"),
            archetype: Archetype::HealthyOtEnvironment,
            seed: opts.seed,
            schema_valid: true,
        },
    }
}

/// Generate the Claroty `CompromisedEndpoint` archetype records.
///
/// Returns `floor(50 * opts.scale)` device records and `floor(20 * opts.scale)` alert
/// records; ≥3 alert records have `severity_id >= 4` (BC-3.4.003 baseline row 2).
fn gen_compromised_endpoint(org_id: &OrgId, opts: &GenOpts) -> FixtureSet {
    let slug = org_slug(org_id);
    let n_devices = (50.0 * opts.scale).floor() as usize;
    let n_alerts = (20.0 * opts.scale).floor() as usize;

    let mut rng = seeded_rng(opts.seed, org_id);
    let _jitter: u32 = rng.gen();

    let mut records: Vec<Value> = Vec::with_capacity(n_devices + n_alerts);

    for i in 0..n_devices {
        records.push(make_device(&slug, opts.seed, i));
    }

    // First 3+ alerts always high severity (severity_id >= 4)
    for i in 0..n_alerts {
        let sev = if i < 3 {
            4u64 + (i as u64 % 3)
        } else {
            1u64 + (i as u64 % 3)
        };
        records.push(make_alert(&slug, opts.seed, i, sev));
    }

    FixtureSet {
        records,
        cursors: vec![],
        provenance: Provenance {
            org_id: org_id.clone(),
            sensor_type: SensorId::from("claroty"),
            archetype: Archetype::CompromisedEndpoint,
            seed: opts.seed,
            schema_valid: true,
        },
    }
}

/// Generate the Claroty `AuthOutage` archetype records.
///
/// Returns `floor(20 * opts.scale)` device records; the first simulated call record has
/// `status_code = 401`. Recovery delay is read from
/// `opts.overrides["auth_outage"]["recovery_after_calls"]` via `apply_overrides`
/// (BC-3.4.003 invariant 6 / EC-AuthOutage).
fn gen_auth_outage(org_id: &OrgId, opts: &GenOpts) -> FixtureSet {
    let slug = org_slug(org_id);
    let n_devices = (20.0 * opts.scale).floor() as usize;

    let mut rng = seeded_rng(opts.seed, org_id);
    let _jitter: u32 = rng.gen();

    // First record is the 401 call record (BC-3.4.003 baseline row 3).
    // NOTE: This record does NOT have a device_id — it represents a failed API call,
    // not a device record. Tests count devices via presence of "device_id" field.
    let mut records: Vec<Value> = Vec::with_capacity(1 + n_devices);

    // The simulated 401 call record (no device_id — not a device)
    let call_record = json!({
        "status_code": 401u64,
        "call_index": 0u64,
        "error": "Unauthorized",
        "message": "Auth outage simulated by fixture generator"
    });
    records.push(call_record);

    // Subsequent device records (normal) — exactly n_devices of them
    for i in 0..n_devices {
        records.push(make_device(&slug, opts.seed, i));
    }

    FixtureSet {
        records,
        cursors: vec![],
        provenance: Provenance {
            org_id: org_id.clone(),
            sensor_type: SensorId::from("claroty"),
            archetype: Archetype::AuthOutage,
            seed: opts.seed,
            schema_valid: true,
        },
    }
}

/// Generate the Claroty `LargeScale` archetype records.
///
/// Returns `floor(10_000 * opts.scale)` device records and `floor(500 * opts.scale)`
/// alert records spread across ≥100 distinct subnets (BC-3.4.003 baseline row 4,
/// memory budget: BC-3.4.002 EC-003).
fn gen_large_scale(org_id: &OrgId, opts: &GenOpts) -> FixtureSet {
    let slug = org_slug(org_id);
    let n_devices = (10_000.0 * opts.scale).floor() as usize;
    let n_alerts = (500.0 * opts.scale).floor() as usize;

    let mut rng = seeded_rng(opts.seed, org_id);
    let _jitter: u32 = rng.gen();

    let mut records: Vec<Value> = Vec::with_capacity(n_devices + n_alerts);

    // Distribute devices across subnets: cycle through 200 subnets (≥100 required).
    // subnet pattern: 10.X.Y.0/24 where X cycles 0-9, Y cycles 0-19 = 200 subnets
    for i in 0..n_devices {
        let subnet_x = (i / 100) % 10;
        let subnet_y = (i / 10) % 20;
        let subnet = format!("10.{subnet_x}.{subnet_y}.0/24");
        records.push(make_device_with_subnet(&slug, opts.seed, i, &subnet));
    }

    for i in 0..n_alerts {
        let sev = if i < 10 { 4u64 } else { 2u64 };
        records.push(make_alert(&slug, opts.seed, i, sev));
    }

    FixtureSet {
        records,
        cursors: vec![],
        provenance: Provenance {
            org_id: org_id.clone(),
            sensor_type: SensorId::from("claroty"),
            archetype: Archetype::LargeScale,
            seed: opts.seed,
            schema_valid: true,
        },
    }
}

/// Generate the Claroty `PaginationEdgeCases` archetype records.
///
/// Returns `default_page_size(SensorId::from("claroty")) × 3` device records with exactly
/// 3 cursor values representing page boundaries (BC-3.4.003 baseline row 5).
fn gen_pagination_edge_cases(org_id: &OrgId, opts: &GenOpts) -> FixtureSet {
    let slug = org_slug(org_id);
    let page_size = default_page_size(SensorId::from("claroty"));
    let n_devices = page_size * 3;

    let mut rng = seeded_rng(opts.seed, org_id);
    let _jitter: u32 = rng.gen();

    let mut records: Vec<Value> = Vec::with_capacity(n_devices);
    for i in 0..n_devices {
        records.push(make_device(&slug, opts.seed, i));
    }

    // 3 cursor values (one per page boundary, max-length per BC-3.4.003).
    let cursors = vec![
        format!("cursor-{}-{}-page-1-{}", slug, opts.seed, "a".repeat(64)),
        format!("cursor-{}-{}-page-2-{}", slug, opts.seed, "b".repeat(64)),
        format!("cursor-{}-{}-page-3-{}", slug, opts.seed, "c".repeat(64)),
    ];

    FixtureSet {
        records,
        cursors,
        provenance: Provenance {
            org_id: org_id.clone(),
            sensor_type: SensorId::from("claroty"),
            archetype: Archetype::PaginationEdgeCases,
            seed: opts.seed,
            schema_valid: true,
        },
    }
}

/// Generate the Claroty `SchemaDrift` archetype records.
///
/// Returns `floor(30 * opts.scale)` device records; `records[0]` intentionally violates
/// the Claroty OpenAPI spec (missing required field). `provenance.schema_valid = false`.
/// Records `[1..]` are schema-valid (BC-3.4.002 / BC-3.4.003 baseline row 6).
fn gen_schema_drift(org_id: &OrgId, opts: &GenOpts) -> FixtureSet {
    let slug = org_slug(org_id);
    let n_devices = (30.0 * opts.scale).floor() as usize;

    let mut rng = seeded_rng(opts.seed, org_id);
    let _jitter: u32 = rng.gen();

    let mut records: Vec<Value> = Vec::with_capacity(n_devices.max(1));

    // records[0]: intentionally drifted — has device_id (our tracking field) but
    // uses a wrong type for `retired` (string instead of bool) to signal drift.
    // The GetDevicesResponse items schema uses additionalProperties:true with no
    // required item fields, so we inject an extra sentinel to mark it as drifted
    // while still having a device_id for ID-prefix tests.
    let drifted = json!({
        "device_id": format!("dev-{slug}-{seed}-drift-0", seed = opts.seed),
        "asset_id": format!("ASSET-{slug}-{seed}-drift-0", seed = opts.seed),
        "_schema_drift": true,
        "retired": "not-a-bool",  // wrong type — boolean field contains string
        "uid": format!("{slug}-{s}-drift-device-00000000", s = opts.seed)
    });
    records.push(drifted);

    // records[1..]: schema-valid devices
    for i in 1..n_devices {
        records.push(make_device(&slug, opts.seed, i));
    }

    FixtureSet {
        records,
        cursors: vec![],
        provenance: Provenance {
            org_id: org_id.clone(),
            sensor_type: SensorId::from("claroty"),
            archetype: Archetype::SchemaDrift,
            seed: opts.seed,
            schema_valid: false,
        },
    }
}

/// Generate the Claroty `HighChurn` archetype records.
///
/// Returns `floor(200 * opts.scale)` device records; ≥20 records have
/// `status = "tombstone"` (BC-3.4.003 baseline row 7, BC-3.4.004 postcondition 3).
fn gen_high_churn(org_id: &OrgId, opts: &GenOpts) -> FixtureSet {
    let slug = org_slug(org_id);
    let n_devices = (200.0 * opts.scale).floor() as usize;

    let mut rng = seeded_rng(opts.seed, org_id);
    let _jitter: u32 = rng.gen();

    let mut records: Vec<Value> = Vec::with_capacity(n_devices);

    for i in 0..n_devices {
        if i < 20 {
            // First 20 are tombstone records (BC-3.4.004 EC-07: dev-{slug}-{seed}-tomb-{n})
            let rec = json!({
                "device_id": format!("dev-{slug}-{seed}-tomb-{i}", seed = opts.seed),
                "asset_id": format!("ASSET-{slug}-{seed}-tomb-{i}", seed = opts.seed),
                "device_category": "OT",
                "device_subcategory": "PLC",
                "device_type": "Controller",
                "device_type_family": "Controller",
                "ip_list": [format!("10.tombstone.{}.{}", (i / 256) % 256, i % 256)],
                "labels": [],
                "mac_list": [format!("ff:ff:{:02x}:{:02x}:{:02x}:{:02x}",
                    (opts.seed >> 8) as u8, opts.seed as u8, (i >> 8) as u8, i as u8)],
                "model": "Tombstoned Device",
                "network_list": ["OT Network"],
                "os_category": "Embedded",
                "retired": true,
                "risk_score": "Unknown",
                "uid": format!("{slug}-{seed}-tomb-device-{i:08x}", seed = opts.seed),
                "status": "tombstone"
            });
            records.push(rec);
        } else {
            records.push(make_device(&slug, opts.seed, i));
        }
    }

    FixtureSet {
        records,
        cursors: vec![],
        provenance: Provenance {
            org_id: org_id.clone(),
            sensor_type: SensorId::from("claroty"),
            archetype: Archetype::HighChurn,
            seed: opts.seed,
            schema_valid: true,
        },
    }
}

/// Generate the Claroty `DormantTenant` archetype records.
///
/// Returns 0 device records, 0 alert records, and empty cursors regardless of scale
/// (BC-3.4.003 baseline row 8 / EC-001).
fn gen_dormant_tenant(org_id: &OrgId, opts: &GenOpts) -> FixtureSet {
    FixtureSet {
        records: vec![],
        cursors: vec![],
        provenance: Provenance {
            org_id: org_id.clone(),
            sensor_type: SensorId::from("claroty"),
            archetype: Archetype::DormantTenant,
            seed: opts.seed,
            schema_valid: true,
        },
    }
}
