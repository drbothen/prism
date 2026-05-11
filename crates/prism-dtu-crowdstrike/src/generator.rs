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
//! # Org-tagging
//!
//! All IDs are prefixed with the org slug derived from the first 8 hex chars of
//! the org UUID (BC-3.4.004). Formats:
//! - Device:    `"dev-{org_slug}-{seed}-{n}"`
//! - Detection: `"alert-{org_slug}-{seed}-{n}"`
//! - Tombstone: `"dev-{org_slug}-{seed}-tomb-{n}"`
//! - Token:     `"tok-{org_slug}-{seed}-{call_n}"`

use serde_json::{json, Value};

use prism_core::SensorId;
use prism_dtu_common::generator::{
    default_page_size, Archetype, FixtureSet, GenOpts, OrgId, Provenance,
};

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

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

    let det_records: Vec<Value> = (0..det_count)
        .map(|n| {
            let det_id = format!("alert-{slug}-{}-{n}", opts.seed);
            make_detection(&det_id, 1, opts)
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

    // Ensure at least 1 contained device
    let mut records: Vec<Value> = (0..dev_count)
        .map(|n| {
            let id = format!("dev-{slug}-{}-{n}", opts.seed);
            let mut dev = make_device(&id, opts);
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

    // Generate detections: first 5 are high-severity (severity_id >= 4)
    let det_records: Vec<Value> = (0..det_count)
        .map(|n| {
            let det_id = format!("alert-{slug}-{}-{n}", opts.seed);
            // First 5 get severity_id=4+, rest get severity_id=2
            let severity_id = if n < 5 { 4_u8 } else { 2_u8 };
            make_detection(&det_id, severity_id, opts)
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

    // Build detection records
    let det_records: Vec<Value> = (0..det_count)
        .map(|n| {
            let det_id = format!("alert-{slug}-{}-{n}", opts.seed);
            make_detection(&det_id, 2, opts)
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

/// Build a `FalconDevice` JSON record (Step-2 detail).
///
/// Tagged with `"_record_type": "device"`.
/// `device_id` field aligns with `containment_store` key in state.rs (AC-004).
fn make_device(device_id: &str, opts: &GenOpts) -> Value {
    let ts = opts.time_anchor.format("%Y-%m-%dT%H:%M:%SZ").to_string();
    json!({
        "_record_type": "device",
        "device_id": device_id,
        "hostname": format!("host-{device_id}"),
        "platform_name": "Linux",
        "os_version": "Ubuntu 22.04",
        "status": "normal",
        "containment_status": "normal",
        "last_seen": ts,
        "external_ip": "203.0.113.1",
        "local_ip": "10.0.0.1",
        "agent_version": "7.10.0.0",
        "cid": "fixture-cid",
        "agent_id": device_id
    })
}

/// Build a `FalconDetection` JSON record (Step-2 detail).
///
/// Tagged with `"_record_type": "detection"`.
/// `detection_id` field aligns with `detection_status_store` key in state.rs (AC-004).
fn make_detection(detection_id: &str, severity_id: u8, opts: &GenOpts) -> Value {
    let ts = opts.time_anchor.format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let severity = match severity_id {
        1 => "Low",
        2 => "Medium",
        3 => "High",
        _ => "Critical",
    };
    json!({
        "_record_type": "detection",
        "detection_id": detection_id,
        "status": "new",
        "severity": severity,
        "severity_id": severity_id,
        "created_timestamp": ts,
        "updated_timestamp": ts,
        "confidence": 80,
        "display_name": format!("Detection {detection_id}"),
        "description": "Fixture detection record",
        "product": "epp",
        "platform": "Linux",
        "tactic": "Execution",
        "tactic_id": "TA0002",
        "technique": "Command and Scripting Interpreter",
        "technique_id": "T1059",
        "objective": "Falcon Detection Method"
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
