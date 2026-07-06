//! CrowdStrike fixture generator — all 8 archetypes, 2-step pagination, OAuth2.
//!
//! # 2-step pagination record convention
//!
//! CrowdStrike uses an IDs-first then details pattern:
//!   Step 1: `GET /queries/<resource>/v1` returns an `IdPage` JSON object tagged
//!           with `"_record_type": "id_page"`.
//!   Step 2: `POST /entities/<resource>/v1` returns `FalconDevice` / `FalconDetection`
//!           JSON objects tagged with `"_record_type": "device"` or `"detection"`.
//!
//! Within a `FixtureSet::records` slice, `IdPage` records always precede their
//! corresponding detail records. `FixtureSet::cursors` contains the FQL offset
//! cursors for the ID-list step only (one cursor per id-page boundary).
//!
//! This convention must be consistent with how the CrowdStrike DTU handler reads
//! fixture data — any change here requires a matching change in `routes/`.
//!
//! # Canonical record shape (F8 / CS-06, review 2026-06-10)
//!
//! The flat scalar key set of generated `detection` / `device` records MUST
//! equal the flat scalar key set of the static fixtures
//! (`fixtures/detections-detail.json` / `fixtures/hosts-detail.json`).
//! Serving extraction is flat `r.get(col_name)` — a key present on only one
//! path silently NULLs that column on the other path the moment a TOML column
//! references it (the CS-01/02/03 failure class). Enforced by
//! `tests/review_2026_06_10_cs_parity.rs::test_f8_cs06_*_shape_parity`:
//! adding/removing a flat field here requires the matching static-fixture
//! change in the same commit (and vice versa).
//!
//! # Org-tagging
//!
//! All IDs are prefixed with the org slug derived from the first 8 hex chars of
//! the org UUID (BC-3.4.004). Formats:
//! - Device:    `"dev-{org_slug}-{seed}-{n}"`
//! - Detection: `"alert-{org_slug}-{seed}-{n}"`
//! - Tombstone: `"dev-{org_slug}-{seed}-tomb-{n}"`
//! - Token:     `"tok-{org_slug}-{seed}-{call_n}"`

use prism_core::SensorId;
use prism_dtu_common::generator::{
    default_page_size, stable_offset, Archetype, FixtureSet, GenOpts, OrgId, Provenance,
};
use serde_json::{json, Value};

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Generate a `FixtureSet` for the CrowdStrike sensor with scenario IOC hashes stamped.
///
/// AC-004 / F-PIVOT003-R2-001: parallel to Armis `generate_with_scenario_cves`.
/// For `CompromisedEndpoint`, detection 0 (the detection linked to the primary contained
/// device) receives `behaviors[0].ioc_value = ioc_hashes[0]` so the ThreatIntel pivot
/// `enrich threat_intel(iocs[].value)` resolves against the catalog at stage ≥ 3.
///
/// For all other archetypes, delegates to `generate()` unchanged.
///
/// If `ioc_hashes` is empty, no IOC stamping occurs (has_ioc_value filter → 0 results;
/// that is the expected behavior for an empty catalog).
pub fn generate_with_scenario_iocs(
    org_id: OrgId,
    archetype: Archetype,
    opts: GenOpts,
    ioc_hashes: &[String],
) -> FixtureSet {
    if archetype != Archetype::CompromisedEndpoint || ioc_hashes.is_empty() {
        return generate(org_id, archetype, opts);
    }

    let slug = org_slug(&org_id);
    let dev_count = scaled(50, opts.scale, 1);
    let det_count = scaled(20, opts.scale, 1);

    let device_ids: Vec<String> = (0..dev_count)
        .map(|n| format!("dev-{slug}-{}-{n}", opts.seed))
        .collect();

    // Ensure at least 1 contained device (mirrors gen_compromised_endpoint).
    let mut records: Vec<Value> = device_ids
        .iter()
        .enumerate()
        .map(|(n, id)| {
            let mut dev = make_device(id, &opts);
            if n == 0 {
                dev["containment_status"] = json!("contained");
                dev["status"] = json!("contained");
            } else {
                dev["containment_status"] = json!("normal");
            }
            dev
        })
        .collect();

    // Stamp ioc_hashes[0] on detection 0 — the detection linked to the primary device
    // (device_ids[0 % dev_count] = device_ids[0]).
    // Detection 0 is the anchor for the ThreatIntel pivot: it carries the IOC that
    // identifies the compromised endpoint in the scenario.
    let ioc_hash = ioc_hashes[0].as_str();
    let det_records: Vec<Value> = (0..det_count)
        .map(|n| {
            let det_id = format!("alert-{slug}-{}-{n}", opts.seed);
            let severity_id = if n < 5 { 4_u8 } else { 2_u8 };
            let scenario_hash = if n == 0 { Some(ioc_hash) } else { None };
            make_detection_with_ioc(
                &det_id,
                &device_ids[n % device_ids.len()],
                severity_id,
                n,
                &opts,
                scenario_hash,
            )
        })
        .collect();

    records.extend(det_records);

    FixtureSet {
        records,
        cursors: Vec::new(),
        provenance: prism_dtu_common::generator::Provenance {
            org_id,
            sensor_id: SensorId::from("crowdstrike"),
            archetype,
            seed: opts.seed,
            schema_valid: true,
        },
    }
}

/// Generate a `FixtureSet` for the CrowdStrike sensor.
///
/// Implements BC-3.4.001 (determinism), BC-3.4.002 (schema validity),
/// BC-3.4.003 (8 archetypes), and BC-3.4.004 (org-tagged IDs).
///
/// Never panics in production use.
pub fn generate(org_id: OrgId, archetype: Archetype, opts: GenOpts) -> FixtureSet {
    let (records, cursors, schema_valid) = match archetype {
        Archetype::HealthyOtEnvironment => {
            let (recs, curs) = gen_healthy_ot(&org_id, &opts);
            (recs, curs, true)
        }
        Archetype::CompromisedEndpoint => {
            let (recs, curs) = gen_compromised_endpoint(&org_id, &opts);
            (recs, curs, true)
        }
        Archetype::AuthOutage => {
            let (recs, curs) = gen_auth_outage(&org_id, &opts);
            (recs, curs, true)
        }
        Archetype::LargeScale => {
            let (recs, curs) = gen_large_scale(&org_id, &opts);
            (recs, curs, true)
        }
        Archetype::PaginationEdgeCases => {
            let (recs, curs) = gen_pagination_edge_cases(&org_id, &opts);
            (recs, curs, true)
        }
        Archetype::SchemaDrift => {
            let (recs, curs) = gen_schema_drift(&org_id, &opts);
            (recs, curs, false)
        }
        Archetype::HighChurn => {
            let (recs, curs) = gen_high_churn(&org_id, &opts);
            (recs, curs, true)
        }
        Archetype::DormantTenant => {
            let (recs, curs) = gen_dormant_tenant(&org_id, &opts);
            (recs, curs, true)
        }
        // Non-exhaustive: future archetypes produce empty fixture sets.
        _ => (Vec::new(), Vec::new(), true),
    };

    FixtureSet {
        records,
        cursors,
        provenance: Provenance {
            org_id,
            sensor_id: SensorId::from("crowdstrike"),
            archetype,
            seed: opts.seed,
            schema_valid,
        },
    }
}

// ---------------------------------------------------------------------------
// Archetype dispatch helpers
// ---------------------------------------------------------------------------

/// Generate `HealthyOtEnvironment` archetype records.
///
/// Baseline (scale=1.0): 50 device records, 5 detection records; no containment.
fn gen_healthy_ot(org_id: &OrgId, opts: &GenOpts) -> (Vec<Value>, Vec<String>) {
    let slug = org_slug(org_id);
    let dev_count = scaled(50, opts.scale, 1);
    let det_count = scaled(5, opts.scale, 1);

    let device_ids: Vec<String> = (0..dev_count)
        .map(|n| format!("dev-{slug}-{}-{n}", opts.seed))
        .collect();

    let mut records: Vec<Value> = device_ids
        .iter()
        .map(|id| {
            let mut dev = make_device(id, opts);
            // Healthy: no containment
            dev["containment_status"] = json!("normal");
            dev
        })
        .collect();

    // F4 / CS-01: each detection links to a device from the seeded pool (n % dev_count).
    let det_records: Vec<Value> = (0..det_count)
        .map(|n| {
            let det_id = format!("alert-{slug}-{}-{n}", opts.seed);
            make_detection(&det_id, &device_ids[n % device_ids.len()], 1, n, opts)
        })
        .collect();

    records.extend(det_records);
    (records, Vec::new())
}

/// Generate `CompromisedEndpoint` archetype records.
///
/// Baseline: 50 device records, 20 detection records; >=3 severity_id >= 4;
/// >=1 device with containment_status = "contained".
fn gen_compromised_endpoint(org_id: &OrgId, opts: &GenOpts) -> (Vec<Value>, Vec<String>) {
    let slug = org_slug(org_id);
    let dev_count = scaled(50, opts.scale, 1);
    let det_count = scaled(20, opts.scale, 1);

    let device_ids: Vec<String> = (0..dev_count)
        .map(|n| format!("dev-{slug}-{}-{n}", opts.seed))
        .collect();

    // Ensure at least 1 contained device
    let mut records: Vec<Value> = device_ids
        .iter()
        .enumerate()
        .map(|(n, id)| {
            let mut dev = make_device(id, opts);
            // First device is always contained (EC-003)
            if n == 0 {
                dev["containment_status"] = json!("contained");
                dev["status"] = json!("contained");
            } else {
                dev["containment_status"] = json!("normal");
            }
            dev
        })
        .collect();

    // Generate detections: first 5 are high-severity (severity_id >= 4).
    // F4 / CS-01: each detection links to a device from the seeded pool
    // (n % dev_count) — detection 0 maps to device 0, the contained endpoint.
    let det_records: Vec<Value> = (0..det_count)
        .map(|n| {
            let det_id = format!("alert-{slug}-{}-{n}", opts.seed);
            // First 5 get severity_id=4+, rest get severity_id=2
            let severity_id = if n < 5 { 4_u8 } else { 2_u8 };
            make_detection(
                &det_id,
                &device_ids[n % device_ids.len()],
                severity_id,
                n,
                opts,
            )
        })
        .collect();

    records.extend(det_records);
    (records, Vec::new())
}

/// Generate `AuthOutage` archetype records.
///
/// Baseline: 20 device records; first OAuth2 record has status_code=401;
/// recovery after `overrides.auth_outage.recovery_after_calls` calls (default 1).
fn gen_auth_outage(org_id: &OrgId, opts: &GenOpts) -> (Vec<Value>, Vec<String>) {
    let slug = org_slug(org_id);
    let dev_count = scaled(20, opts.scale, 1);

    // Read recovery_after_calls from overrides (EC-002)
    let recovery_after_calls = opts
        .overrides
        .get("auth_outage")
        .and_then(|v| v.get("recovery_after_calls"))
        .and_then(Value::as_u64)
        .unwrap_or(1) as usize;

    // Generate OAuth2 token records: first N are 401, then one 200
    let mut records: Vec<Value> = Vec::new();

    for call_n in 0..recovery_after_calls {
        records.push(make_oauth2_record(&slug, opts.seed, call_n, 401));
    }
    // Recovery token (200)
    records.push(make_oauth2_record(
        &slug,
        opts.seed,
        recovery_after_calls,
        200,
    ));

    // Device records
    let device_records: Vec<Value> = (0..dev_count)
        .map(|n| {
            let id = format!("dev-{slug}-{}-{n}", opts.seed);
            let mut dev = make_device(&id, opts);
            dev["containment_status"] = json!("normal");
            dev
        })
        .collect();

    records.extend(device_records);
    (records, Vec::new())
}

/// Generate `LargeScale` archetype records.
///
/// Baseline: 10,000 device records, 500 detection records.
/// Produces 2-step pagination: IdPage records followed by detail records.
fn gen_large_scale(org_id: &OrgId, opts: &GenOpts) -> (Vec<Value>, Vec<String>) {
    let slug = org_slug(org_id);
    let dev_count = scaled(10_000, opts.scale, 1);
    let det_count = scaled(500, opts.scale, 1);
    let page_size = default_page_size(SensorId::from("crowdstrike"));

    let device_ids: Vec<String> = (0..dev_count)
        .map(|n| format!("dev-{slug}-{}-{n}", opts.seed))
        .collect();

    // Build id_pages for device IDs
    let (id_page_records, cursors) = build_id_pages(&device_ids, page_size, &slug, opts.seed);

    // Build device detail records
    let device_records: Vec<Value> = device_ids
        .iter()
        .map(|id| {
            let mut dev = make_device(id, opts);
            dev["containment_status"] = json!("normal");
            dev
        })
        .collect();

    // Build detection records.
    // F4 / CS-01: each detection links to a device from the seeded pool (n % dev_count).
    let det_records: Vec<Value> = (0..det_count)
        .map(|n| {
            let det_id = format!("alert-{slug}-{}-{n}", opts.seed);
            make_detection(&det_id, &device_ids[n % device_ids.len()], 2, n, opts)
        })
        .collect();

    let mut records = id_page_records;
    records.extend(device_records);
    records.extend(det_records);

    (records, cursors)
}

/// Generate `PaginationEdgeCases` archetype records.
///
/// Baseline: `default_page_size(CrowdStrike) x 3` device records.
/// Produces 3 IdPage records + 3 detail pages.
fn gen_pagination_edge_cases(org_id: &OrgId, opts: &GenOpts) -> (Vec<Value>, Vec<String>) {
    let slug = org_slug(org_id);
    let page_size = default_page_size(SensorId::from("crowdstrike"));
    let dev_count = scaled(page_size * 3, opts.scale, page_size * 3);

    let device_ids: Vec<String> = (0..dev_count)
        .map(|n| format!("dev-{slug}-{}-{n}", opts.seed))
        .collect();

    // Build exactly 3 id_pages (one per page of page_size devices)
    let (id_page_records, cursors) = build_id_pages(&device_ids, page_size, &slug, opts.seed);

    // Build device detail records
    let device_records: Vec<Value> = device_ids
        .iter()
        .map(|id| {
            let mut dev = make_device(id, opts);
            dev["containment_status"] = json!("normal");
            dev
        })
        .collect();

    let mut records = id_page_records;
    records.extend(device_records);

    (records, cursors)
}

/// Generate `SchemaDrift` archetype records.
///
/// Baseline: 30 device records; `records[0]` violates CrowdStrike device schema;
/// `provenance.schema_valid = false`.
fn gen_schema_drift(org_id: &OrgId, opts: &GenOpts) -> (Vec<Value>, Vec<String>) {
    let slug = org_slug(org_id);
    let dev_count = scaled(30, opts.scale, 1);

    let mut records: Vec<Value> = Vec::with_capacity(dev_count);

    for n in 0..dev_count {
        let id = format!("dev-{slug}-{}-{n}", opts.seed);
        if n == 0 {
            // First record is drifted: device_id is null (required field missing)
            let mut dev = make_device(&id, opts);
            dev["device_id"] = Value::Null;
            dev["containment_status"] = json!("normal");
            records.push(dev);
        } else {
            let mut dev = make_device(&id, opts);
            dev["containment_status"] = json!("normal");
            records.push(dev);
        }
    }

    (records, Vec::new())
}

/// Generate `HighChurn` archetype records.
///
/// Baseline: 200 device records; >=20 tombstones.
fn gen_high_churn(org_id: &OrgId, opts: &GenOpts) -> (Vec<Value>, Vec<String>) {
    let slug = org_slug(org_id);
    let dev_count = scaled(200, opts.scale, 1);
    let tomb_count = scaled(20, opts.scale, 20);

    let mut records: Vec<Value> = (0..dev_count)
        .map(|n| {
            let id = format!("dev-{slug}-{}-{n}", opts.seed);
            let mut dev = make_device(&id, opts);
            dev["containment_status"] = json!("normal");
            dev
        })
        .collect();

    // Add tombstone records
    let tombstones: Vec<Value> = (0..tomb_count)
        .map(|n| make_tombstone(&slug, opts.seed, n))
        .collect();

    records.extend(tombstones);
    (records, Vec::new())
}

/// Generate `DormantTenant` archetype records.
///
/// Baseline: 0 records; 0 cursors. Both IdPage and detail records are empty.
fn gen_dormant_tenant(_org_id: &OrgId, _opts: &GenOpts) -> (Vec<Value>, Vec<String>) {
    (Vec::new(), Vec::new())
}

// ---------------------------------------------------------------------------
// 2-step pagination helpers
// ---------------------------------------------------------------------------

/// Build a set of `IdPage` records and FQL offset cursors from a list of device IDs.
///
/// Partitions `ids` into pages of `page_size` each. Returns:
/// - A `Vec<Value>` of id_page records (one per page), each tagged `_record_type: "id_page"`.
/// - A `Vec<String>` of FQL offset cursors (one per page boundary).
fn build_id_pages(
    ids: &[String],
    page_size: usize,
    slug: &str,
    seed: u64,
) -> (Vec<Value>, Vec<String>) {
    let mut id_page_records = Vec::new();
    let mut cursors = Vec::new();

    let pages: Vec<&[String]> = ids.chunks(page_size).collect();
    let total_pages = pages.len();

    for (page_idx, chunk) in pages.iter().enumerate() {
        // FQL cursor: format "fql-{slug}-{seed}-page-{page_idx}"
        let cursor = format!("fql-{slug}-{seed}-page-{page_idx}");
        cursors.push(cursor.clone());

        // Offset cursor for next page (None if last page)
        let next_cursor = if page_idx + 1 < total_pages {
            Some(
                format!("fql-{slug}-{seed}-page-{}", page_idx + 1)
                    .as_str()
                    .to_owned(),
            )
        } else {
            None
        };

        let ids_vec: Vec<String> = chunk.to_vec();
        let page_record = make_id_page(&ids_vec, next_cursor.as_deref());
        id_page_records.push(page_record);
    }

    (id_page_records, cursors)
}

/// Build an `IdPage` JSON record (Step-1 of the 2-step pattern).
///
/// The returned value is tagged with `"_record_type": "id_page"` for
/// disambiguation within `FixtureSet::records`.
///
/// Shape mirrors `.references/schemas/crowdstrike/types.rs:IdPage`.
fn make_id_page(ids: &[String], offset_cursor: Option<&str>) -> Value {
    let resources: Vec<Value> = ids.iter().map(|id| json!(id)).collect();
    let mut page = json!({
        "_record_type": "id_page",
        "resources": resources,
        "errors": [],
        "meta": {
            "query_time": 0.01,
            "pagination": {
                "total": ids.len(),
                "count": ids.len(),
                "limit": ids.len()
            },
            "trace_id": "fixture-trace"
        }
    });

    if let Some(cursor) = offset_cursor {
        page["meta"]["pagination"]["offset"] = json!(cursor);
    }

    page
}

// `stable_offset` lifted to `prism_dtu_common::generator::offset::stable_offset`
// (review-2026-06-10 P1-02) so the Cyberint/Claroty/Armis generators share one
// RNG-free fold instead of triplicating it. Identical FNV-1a algorithm —
// derived timestamps are byte-identical to the pre-lift values.

/// Build a `FalconDevice` JSON record (Step-2 detail).
///
/// Tagged with `"_record_type": "device"`.
/// `device_id` field aligns with `containment_store` key in state.rs (AC-004).
///
/// F6 / CS-03 (review 2026-06-10): `first_seen` is required by the
/// crowdstrike.sensor.toml `devices` table (flat datetime column). It is
/// seeded-deterministic (stable fold of device_id × seed → 7..=90 days before
/// `last_seen`) and always strictly earlier than `last_seen`.
///
/// P4-01 (cascade pass-4): `last_seen` varies per record — `time_anchor`
/// minus a seeded `stable_offset` fold (0..7 days in minutes), mirroring
/// Armis `derive_seen_window`. RNG-free: the fold never consults the ChaCha20
/// stream, so per-record variance cannot perturb other generated values
/// (BC-3.4.001 determinism preserved). Strict ordering holds by construction:
/// `last_seen >= anchor - 10_079 min` while
/// `first_seen <= last_seen - 7 days (10_080 min)`.
fn make_device(device_id: &str, opts: &GenOpts) -> Value {
    // last_seen: 0..7 days (in minutes) before the time anchor, stable per
    // (device_id, seed) — P4-01.
    let minutes_before = (stable_offset(device_id, opts.seed) % 10_080) as i64;
    let last_seen_dt = opts.time_anchor - chrono::Duration::minutes(minutes_before);
    let ts = last_seen_dt.format("%Y-%m-%dT%H:%M:%SZ").to_string();
    // first_seen: 7..=90 days before last_seen, stable per (device_id, seed+1)
    // — strictly earlier than last_seen (F6 / CS-03).
    let days_before = 7 + (stable_offset(device_id, opts.seed.wrapping_add(1)) % 84) as i64;
    let first_seen = (last_seen_dt - chrono::Duration::days(days_before))
        .format("%Y-%m-%dT%H:%M:%SZ")
        .to_string();
    json!({
        "_record_type": "device",
        "device_id": device_id,
        "hostname": format!("host-{device_id}"),
        "platform_name": "Linux",
        "os_version": "Ubuntu 22.04",
        "status": "normal",
        "containment_status": "normal",
        "first_seen": first_seen,
        "last_seen": ts,
        "external_ip": "203.0.113.1",
        "local_ip": "10.0.0.1",
        "agent_version": "7.10.0.0",
        "cid": "fixture-cid",
        "agent_id": device_id
    })
}

/// Canonical MITRE ATT&CK technique ↔ tactic table (review-2026-06-10 P1-03 + P2-06).
///
/// Tuple layout: `(technique_id, technique_name, tactic_id, tactic_name)`.
///
/// Single mapping source for the generator AND the static fixtures
/// (`fixtures/detections-detail.json`): the flat `technique` column carries the
/// DISPLAY NAME and the flat `technique_id` column carries the MITRE ID — the
/// same value classes on both serving paths, so the crowdstrike.sensor.toml
/// `attack.technique.name` mapping normalizes identically regardless of path.
/// Covers every technique ID cycled by the static fixture set.
///
/// P2-06 (cascade pass-2): each technique additionally carries its canonical
/// tactic pairing, valid per the MITRE ATT&CK Enterprise matrix. For
/// multi-tactic techniques (e.g. T1078 Valid Accounts, T1053 Scheduled
/// Task/Job) ONE valid tactic is pinned so fixtures and generator agree on a
/// single, verifiable pairing. The static fixtures previously cross-paired
/// rotated tactic/technique lists (e.g. "Initial Access"/TA0001 with T1059,
/// which is Execution/TA0002) — invalid MITRE data that an LLM agent consumer
/// would reproduce.
///
/// NOTE: real-API value semantics (name vs ID in the flat `technique` field of
/// CrowdStrike detect responses) carry a MEDIUM confidence flag from the
/// adversary — to be confirmed by dtu-validator against the live API.
pub const MITRE_TECHNIQUES: &[(&str, &str, &str, &str)] = &[
    (
        "T1059",
        "Command and Scripting Interpreter",
        "TA0002",
        "Execution",
    ),
    ("T1078", "Valid Accounts", "TA0001", "Initial Access"),
    ("T1053", "Scheduled Task/Job", "TA0003", "Persistence"),
    (
        "T1055",
        "Process Injection",
        "TA0004",
        "Privilege Escalation",
    ),
    (
        "T1003",
        "OS Credential Dumping",
        "TA0006",
        "Credential Access",
    ),
    ("T1021", "Remote Services", "TA0008", "Lateral Movement"),
    ("T1018", "Remote System Discovery", "TA0007", "Discovery"),
    (
        "T1082",
        "System Information Discovery",
        "TA0007",
        "Discovery",
    ),
    ("T1098", "Account Manipulation", "TA0003", "Persistence"),
    (
        "T1071",
        "Application Layer Protocol",
        "TA0011",
        "Command and Control",
    ),
];

/// Look up the canonical MITRE technique display name for an ID (P1-03).
pub fn technique_name(technique_id: &str) -> Option<&'static str> {
    MITRE_TECHNIQUES
        .iter()
        .find(|(id, _, _, _)| *id == technique_id)
        .map(|(_, name, _, _)| *name)
}

/// Look up the canonical `(tactic_id, tactic_name)` pairing for a technique
/// ID (P2-06). Returns the single pinned tactic from [`MITRE_TECHNIQUES`].
pub fn tactic_pair_for_technique(technique_id: &str) -> Option<(&'static str, &'static str)> {
    MITRE_TECHNIQUES
        .iter()
        .find(|(id, _, _, _)| *id == technique_id)
        .map(|(_, _, tactic_id, tactic_name)| (*tactic_id, *tactic_name))
}

/// Build a `FalconDetection` JSON record (Step-2 detail).
///
/// Tagged with `"_record_type": "detection"`.
/// `detection_id` field aligns with `detection_status_store` key in state.rs (AC-004).
///
/// F4 / CS-01 (review 2026-06-10): `device_id` links the detection to a record
/// from the seeded device pool — crowdstrike.sensor.toml declares a flat
/// `detections.device_id` column, and the serving extraction is flat
/// `r.get(col_name)`; an absent key silently normalized the column to NULL.
///
/// F7 / CS-04 (review 2026-06-10): `created_timestamp` varies per record —
/// time_anchor minus a seeded offset (stable fold of detection_id × seed,
/// 0..7 days) — so FQL time-window filtering can discriminate between records.
/// A single shared timestamp made every bounded window all-or-nothing.
///
/// P4-02 (cascade pass-4): the MITRE tuple cycles the canonical table per
/// record (`det_index % MITRE_TECHNIQUES.len()`) — deterministic and RNG-free
/// — instead of pinning `MITRE_TECHNIQUES[0]` on every detection. Pairing
/// validity for every cycle slot is guaranteed by the P2-06 table.
///
/// AC-004 (S-DEMO-ENRICHMENT-PIVOT-003): `scenario_ioc_hash` is the IOC hash
/// value to stamp on `behaviors[0].ioc_value` when present. The stamped
/// `behaviors[0].ioc_type` MUST be `"hash_sha256"` — algorithm-qualified per
/// BC-2.06.019 v1.13 (bare `"hash"` is incorrect; `"cmdline"` is a SEPARATE
/// sibling field, never an ioc_type value).
///
/// NOTE (U19): no typed `Detection` or `Behavior` struct exists in this crate.
/// IOC fields are added as JSON keys in the `serde_json::Value` detection record.
/// Shape parity: `behaviors[]` added here MUST also appear in
/// `fixtures/detections-detail.json` in the same commit (review_2026_06_10_cs_parity.rs).
fn make_detection(
    detection_id: &str,
    device_id: &str,
    severity_id: u8,
    det_index: usize,
    opts: &GenOpts,
) -> Value {
    make_detection_with_ioc(detection_id, device_id, severity_id, det_index, opts, None)
}

/// Build a `FalconDetection` JSON record with optional scenario IOC hash stamping.
///
/// AC-004 (S-DEMO-ENRICHMENT-PIVOT-003): when `scenario_ioc_hash` is `Some(hash)`,
/// the returned detection record includes a `"behaviors"` array with at least one
/// entry carrying `"ioc_type": "hash_sha256"` and `"ioc_value": hash`.
///
/// When `scenario_ioc_hash` is `None`, the `"behaviors"` array contains the existing
/// MITRE-only behavior entry (matching the static fixture shape — shape parity).
pub(crate) fn make_detection_with_ioc(
    detection_id: &str,
    device_id: &str,
    severity_id: u8,
    det_index: usize,
    opts: &GenOpts,
    scenario_ioc_hash: Option<&str>,
) -> Value {
    // created_timestamp: 0..10080 minutes (7 days) before the anchor, stable
    // per (detection_id, seed) — deterministic per BC-3.4.001.
    let minutes_before = (stable_offset(detection_id, opts.seed) % 10_080) as i64;
    let created = (opts.time_anchor - chrono::Duration::minutes(minutes_before))
        .format("%Y-%m-%dT%H:%M:%SZ")
        .to_string();
    let severity = match severity_id {
        1 => "Low",
        2 => "Medium",
        3 => "High",
        _ => "Critical",
    };
    // P4-02: cycle the canonical MITRE table per record — full tuple from one
    // slot, so the (tactic, technique) pairing is always table-valid.
    let (technique_id, technique, tactic_id, tactic) =
        MITRE_TECHNIQUES[det_index % MITRE_TECHNIQUES.len()];

    // AC-004 (S-DEMO-ENRICHMENT-PIVOT-003): build behaviors array.
    // Shape parity rule: this array MUST match the `behaviors` key in
    // `fixtures/detections-detail.json` (review_2026_06_10_cs_parity.rs).
    //
    // Base MITRE behavior entry (present in both scenario and non-scenario records
    // to maintain static fixture shape parity).
    let base_behavior = json!({
        "tactic": tactic,
        "technique": technique,
        "technique_id": technique_id
    });

    let behaviors = if let Some(ioc_hash) = scenario_ioc_hash {
        // AC-004: ioc_type MUST be "hash_sha256" (algorithm-qualified per BC-2.06.019 v1.13).
        // Tolerant-unknown-type policy applies to READING; we always WRITE "hash_sha256".
        let mut ioc_behavior = base_behavior.clone();
        if let Some(obj) = ioc_behavior.as_object_mut() {
            obj.insert("ioc_type".to_string(), json!("hash_sha256"));
            obj.insert("ioc_value".to_string(), json!(ioc_hash));
            obj.insert("ioc_source".to_string(), json!("catalog"));
            obj.insert("ioc_description".to_string(), json!("scenario IOC"));
        }
        json!([ioc_behavior])
    } else {
        json!([base_behavior])
    };

    json!({
        "_record_type": "detection",
        "detection_id": detection_id,
        "device_id": device_id,
        "status": "new",
        "severity": severity,
        "severity_id": severity_id,
        "created_timestamp": created,
        "updated_timestamp": created,
        "confidence": 80,
        "display_name": format!("Detection {detection_id}"),
        "description": "Fixture detection record",
        "product": "epp",
        "platform": "Linux",
        // P1-03 + P2-06 + P4-02: tactic and technique sourced from the
        // canonical table — name in `technique`/`tactic`, MITRE IDs in
        // `technique_id`/`tactic_id` (same value classes as the static
        // fixtures), cycled per record by det_index (P4-02).
        "tactic": tactic,
        "tactic_id": tactic_id,
        "technique": technique,
        "technique_id": technique_id,
        "objective": "Falcon Detection Method",
        // AC-004 (S-DEMO-ENRICHMENT-PIVOT-003): behaviors array with MITRE entry
        // (+ IOC keys in scenario mode). Shape parity: must match detections-detail.json.
        "behaviors": behaviors,
    })
}

/// Build a tombstone device record.
///
/// ID format: `"dev-{org_slug}-{seed}-tomb-{n}"` (BC-3.4.004, AC-005).
fn make_tombstone(org_slug: &str, seed: u64, n: usize) -> Value {
    let device_id = format!("dev-{org_slug}-{seed}-tomb-{n}");
    json!({
        "_record_type": "tombstone",
        "device_id": device_id,
        "status": "deleted",
        "containment_status": "normal"
    })
}

// ---------------------------------------------------------------------------
// OAuth2 helpers
// ---------------------------------------------------------------------------

/// Build an `OAuth2TokenResponse` fixture record.
///
/// Tagged with `"_record_type": "oauth2_token"`.
/// Shape mirrors `.references/schemas/crowdstrike/types.rs:OAuth2TokenResponse`.
///
/// `status_code=401` for the outage record; `200` with a deterministic
/// `access_token = "tok-{org_slug}-{seed}-{call_n}"` for subsequent records.
fn make_oauth2_record(org_slug: &str, seed: u64, call_n: usize, status_code: u16) -> Value {
    if status_code == 401 {
        json!({
            "_record_type": "oauth2_token",
            "status_code": 401_u64,
            "error": "invalid_client",
            "error_description": "Simulated auth outage — fixture record"
        })
    } else {
        let access_token = format!("tok-{org_slug}-{seed}-{call_n}");
        json!({
            "_record_type": "oauth2_token",
            "status_code": 200_u64,
            "access_token": access_token,
            "token_type": "bearer",
            "expires_in": 1799_i64
        })
    }
}

// ---------------------------------------------------------------------------
// Org-slug helper
// ---------------------------------------------------------------------------

/// Derive an org slug from the first 8 hex chars of the org UUID bytes.
///
/// `org_slug = hex(org_id.as_bytes()[0..4])` -- deterministic, 8 characters.
fn org_slug(org_id: &OrgId) -> String {
    let bytes = org_id.as_bytes();
    format!(
        "{:02x}{:02x}{:02x}{:02x}",
        bytes[0], bytes[1], bytes[2], bytes[3]
    )
}

// ---------------------------------------------------------------------------
// Scale helper
// ---------------------------------------------------------------------------

/// Scale a baseline count by `opts.scale`, flooring to nearest integer.
///
/// Minimum result is `min_count` (never returns 0 for non-DormantTenant archetypes).
fn scaled(baseline: usize, scale: f64, min_count: usize) -> usize {
    let count = (baseline as f64 * scale).floor() as usize;
    count.max(min_count)
}

// ---------------------------------------------------------------------------
// In-crate unit tests (fixture-gen gated)
// ---------------------------------------------------------------------------
//
// Test 5 is placed here because `make_detection_with_ioc` is `pub(crate)` and
// cannot be called from an external integration test file.

#[cfg(all(test, feature = "fixture-gen"))]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    /// Test 5 — BC-2.06.019 v1.13 PC-4 CrowdStrike IOC stamp correction:
    /// `make_detection_with_ioc()` must stamp `behaviors[0]["ioc_type"] == "hash_sha256"`
    /// (algorithm-qualified token), NOT bare `"hash"` (BC-2.06.019 v1.13 correction).
    ///
    /// Also asserts:
    /// - `behaviors[0]["ioc_value"]` == the scenario_ioc_hash passed in
    /// - `behaviors[0]["ioc_source"]` == `"catalog"`
    /// - `behaviors[0]["ioc_description"]` == `"scenario IOC"`
    ///
    /// Canonical test vector (BC-2.06.019 v1.13):
    ///   Input: scenario_ioc_hash = Some("aabbccdd" * 8)
    ///   Expected: behaviors[0]["ioc_type"] = "hash_sha256" (NOT "hash")
    ///             behaviors[0]["ioc_value"] = "aabbccdd" * 8
    ///             behaviors[0]["ioc_source"] = "catalog"
    ///             behaviors[0]["ioc_description"] = "scenario IOC"
    ///
    /// BC-2.06.019 v1.13 PC-4 (CrowdStrike detections IOC stamp).
    /// Red Gate test plan #5 (S-DEMO-ENRICHMENT-PIVOT-003).
    #[test]
    #[allow(non_snake_case)]
    fn test_BC_2_06_019_crowdstrike_detection_behaviors_ioc_hash_stamped() {
        let org = OrgId([
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E,
            0x0F, 0x10,
        ]);
        let seed: u64 = 42;
        let opts = GenOpts {
            seed,
            ..Default::default()
        };

        // Canonical test vector (BC-2.06.019 v1.13): 64-char SHA256-like hex string.
        let ioc_hash = "aabbccddaabbccddaabbccddaabbccddaabbccddaabbccddaabbccddaabbccdd";

        let slug = org_slug(&org);
        let detection_id = format!("det-{slug}-{seed}-0");
        let device_id = format!("dev-{slug}-{seed}-0");

        let record = make_detection_with_ioc(
            &detection_id,
            &device_id,
            2, // severity_id: Medium
            0, // det_index: 0
            &opts,
            Some(ioc_hash),
        );

        // Verify behaviors[] array is present and non-empty.
        let behaviors = record
            .get("behaviors")
            .and_then(|v| v.as_array())
            .expect("detection record must have a 'behaviors' array after IOC stamping");

        assert!(
            !behaviors.is_empty(),
            "BC-2.06.019 v1.13 PC-4: behaviors[] array must be non-empty after IOC stamping"
        );

        let b0 = &behaviors[0];

        // CRITICAL assertion: ioc_type MUST be "hash_sha256" (NOT bare "hash").
        // BC-2.06.019 v1.13 correction: algorithm-qualified tokens only.
        let ioc_type = b0
            .get("ioc_type")
            .and_then(|v| v.as_str())
            .expect("behaviors[0] must have 'ioc_type' key");

        assert_eq!(
            ioc_type, "hash_sha256",
            "BC-2.06.019 v1.13 correction: behaviors[0]['ioc_type'] must be 'hash_sha256' \
             (algorithm-qualified), NOT bare 'hash'. Got: '{ioc_type}'. \
             CrowdStrike enum: {{hash_sha256, hash_md5, domain, filename, registry_key}}."
        );

        // Assert ioc_value matches the input hash exactly.
        let ioc_value = b0
            .get("ioc_value")
            .and_then(|v| v.as_str())
            .expect("behaviors[0] must have 'ioc_value' key");

        assert_eq!(
            ioc_value, ioc_hash,
            "behaviors[0]['ioc_value'] must equal the scenario_ioc_hash passed in; \
             expected '{}', got '{}'",
            ioc_hash, ioc_value
        );

        // Assert ioc_source is "catalog".
        let ioc_source = b0
            .get("ioc_source")
            .and_then(|v| v.as_str())
            .expect("behaviors[0] must have 'ioc_source' key");

        assert_eq!(
            ioc_source, "catalog",
            "behaviors[0]['ioc_source'] must be 'catalog'; got '{ioc_source}'"
        );

        // Assert ioc_description is "scenario IOC".
        let ioc_description = b0
            .get("ioc_description")
            .and_then(|v| v.as_str())
            .expect("behaviors[0] must have 'ioc_description' key");

        assert_eq!(
            ioc_description, "scenario IOC",
            "behaviors[0]['ioc_description'] must be 'scenario IOC'; got '{ioc_description}'"
        );
    }

    /// Complementary test: `make_detection_with_ioc(None)` must NOT include IOC fields —
    /// only the base MITRE behavior entry should be present.
    ///
    /// This test must be GREEN even before implementation (the None branch is already
    /// implemented). It serves as a regression guard for shape parity.
    ///
    /// BC-2.06.019 v1.13: tolerant-unknown-type policy applies to READING; writing "hash_sha256"
    /// only applies when scenario_ioc_hash is Some. None path = MITRE-only.
    #[test]
    #[allow(non_snake_case)]
    fn test_BC_2_06_019_crowdstrike_detection_behaviors_no_ioc_when_none() {
        let org = OrgId([
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E,
            0x0F, 0x10,
        ]);
        let seed: u64 = 42;
        let opts = GenOpts {
            seed,
            ..Default::default()
        };

        let slug = org_slug(&org);
        let detection_id = format!("det-{slug}-{seed}-0");
        let device_id = format!("dev-{slug}-{seed}-0");

        // None path: no IOC stamping — behaviors[] contains only the base MITRE entry.
        let record = make_detection_with_ioc(&detection_id, &device_id, 2, 0, &opts, None);

        let behaviors = record
            .get("behaviors")
            .and_then(|v| v.as_array())
            .expect("detection record must have a 'behaviors' array even without IOC stamping");

        assert_eq!(
            behaviors.len(),
            1,
            "Without IOC stamping (None), behaviors[] must have exactly 1 MITRE-only entry; \
             got {}",
            behaviors.len()
        );

        // The MITRE-only entry must NOT have ioc_type or ioc_value keys.
        let b0 = &behaviors[0];
        assert!(
            b0.get("ioc_type").is_none(),
            "MITRE-only behavior entry must NOT have 'ioc_type' key; got: {b0}"
        );
        assert!(
            b0.get("ioc_value").is_none(),
            "MITRE-only behavior entry must NOT have 'ioc_value' key; got: {b0}"
        );
    }

    // ── S-DEMO-ENRICHMENT-TYPED-OUTPUT-001 Red Gate Test ─────────────────────
    // RGT-016 (AC-011): `crowdstrike_detections.behaviors_ioc_value_first` is populated via
    // JSONPath `source_path = "$.behaviors[0].ioc_value"` (nested, not top-level scalar).
    // OBS-001 (LOCAL adversary pass-2): removed dead top-level `behaviors_ioc_value_first`
    // scalar — only the nested path matters for spec-driven adapter column population.

    /// RGT-016 (AC-011): the spec-driven adapter populates `behaviors_ioc_value_first` in
    /// `crowdstrike_detections` via `source_path = "$.behaviors[0].ioc_value"` (JSONPath
    /// extraction from the nested `behaviors` array, NOT from a dead top-level scalar).
    ///
    /// This test binds the full AC-011 chain:
    ///   1. TOML spec declares `source_path = "$.behaviors[0].ioc_value"` for the column.
    ///   2. DTU generator (`make_detection_with_ioc`) emits `behaviors[0].ioc_value = <hash>`
    ///      (via the real production generator path, not a hand-crafted record).
    ///   3. The JSONPath `$.behaviors[0].ioc_value` resolves to the expected hash.
    ///
    /// OBS-001: the dead top-level `behaviors_ioc_value_first` scalar has been removed from
    /// `make_detection_with_ioc` — this assertion also verifies it is gone.
    #[test]
    fn test_ac011_crowdstrike_detections_behaviors_ioc_value_first_column_via_jsonpath() {
        // Step 1: Read source_path from the TOML spec (not hardcoded).
        let toml_str = include_str!("../../prism-sensors/specs/crowdstrike.sensor.toml");
        let parsed: toml::Value = toml_str.parse().expect("valid crowdstrike.sensor.toml");
        let tables = parsed
            .get("tables")
            .and_then(|v| v.as_array())
            .expect("crowdstrike.sensor.toml must have [[tables]] section");
        let det_table = tables
            .iter()
            .find(|t| t.get("table_name").and_then(|v| v.as_str()) == Some("detections"))
            .expect("crowdstrike.sensor.toml must have a 'detections' table");
        let columns = det_table
            .get("columns")
            .and_then(|v| v.as_array())
            .expect("detections table must have columns");
        let biv_col = columns
            .iter()
            .find(|c| c.get("name").and_then(|v| v.as_str()) == Some("behaviors_ioc_value_first"))
            .expect("detections table must have behaviors_ioc_value_first column");
        let source_path = biv_col
            .get("source_path")
            .and_then(|v| v.as_str())
            .expect("behaviors_ioc_value_first must declare source_path");
        assert_eq!(
            source_path, "$.behaviors[0].ioc_value",
            "AC-011 RGT-016: TOML must declare source_path='$.behaviors[0].ioc_value' \
             for behaviors_ioc_value_first; got: '{source_path}'"
        );

        // Step 2: Generate a detection record via the real production generator path.
        let org = OrgId([
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E,
            0x0F, 0x10,
        ]);
        let seed: u64 = 42;
        let opts = GenOpts {
            seed,
            ..Default::default()
        };
        // Canonical test vector: known SHA256-like IOC hash.
        let ioc_hash = "aabbccddaabbccddaabbccddaabbccddaabbccddaabbccddaabbccddaabbccdd";

        let slug = org_slug(&org);
        let detection_id = format!("det-{slug}-{seed}-0");
        let device_id = format!("dev-{slug}-{seed}-0");

        let record = make_detection_with_ioc(
            &detection_id,
            &device_id,
            2, // severity_id: Medium
            0, // det_index: 0
            &opts,
            Some(ioc_hash),
        );

        // Step 3: Apply source_path extraction — $.behaviors[0].ioc_value.
        // The spec-driven adapter calls extract_at_path(record, source_path) to populate
        // the behaviors_ioc_value_first column.
        let extracted = &record["behaviors"][0]["ioc_value"];
        assert_eq!(
            extracted.as_str(),
            Some(ioc_hash),
            "AC-011 RGT-016: source_path='$.behaviors[0].ioc_value' must resolve to \
             '{ioc_hash}' from the DTU-generated detection record. \
             Got: {extracted}"
        );

        // OBS-001: the dead top-level scalar must NOT be emitted (column is populated via
        // source_path, not a pre-computed top-level key).
        assert!(
            record.get("behaviors_ioc_value_first").is_none(),
            "OBS-001 RGT-016: dead top-level 'behaviors_ioc_value_first' must NOT be emitted. \
             The column is populated via source_path='$.behaviors[0].ioc_value', not a \
             pre-computed scalar. Found: {:?}",
            record.get("behaviors_ioc_value_first")
        );
    }
}
