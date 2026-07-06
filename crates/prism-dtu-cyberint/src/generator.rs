//! Cyberint fixture generator — all 8 archetypes across 4 API surfaces.
//!
//! Implements `generate(org_id, archetype, opts) -> FixtureSet` using a single
//! deterministic `ChaCha20Rng` stream that advances sequentially through the
//! alert, ASM asset, CVE, and IOC surfaces (EC-003 / BC-3.4.001 invariant 2).
//!
//! Gated behind `#[cfg(feature = "fixture-gen")]` — never compiled into production
//! (AC-007 / D-056).
//!
//! Per-surface baselines at `scale = 1.0` (AC-001):
//! - `HealthyOtEnvironment` : alert=5, asm_asset=10, cve=5, ioc=5
//! - `CompromisedEndpoint`  : alert=20 (≥3 high-severity), asm_asset=10, cve=10, ioc=10
//! - `AuthOutage`           : alert=5, asm_asset=10, cve=5, ioc=5  (same as Healthy)
//! - `LargeScale`           : alert=500, asm_asset=2000, cve=1000, ioc=1000
//! - `PaginationEdgeCases`  : alert=10 (paginated), asm_asset=10, cve=5, ioc=5
//! - `SchemaDrift`          : alert=5 (index 0 invalid), asm_asset=10, cve=5, ioc=5
//! - `HighChurn`            : alert=20, asm_asset=30, cve=10, ioc=15 (+ tombstones)
//! - `DormantTenant`        : all surfaces empty (EC-001)
//!
//! Source specs (read-only, test-only validation):
//! - `.references/poller-express/docs/specs/alert_api_specs.json`
//! - `.references/poller-express/docs/specs/asm_assets_api_specs.json`
//! - `.references/poller-express/docs/specs/cve_api_specs.json`
//! - `.references/poller-express/docs/specs/ioc_api_specs.json`

use prism_core::SensorId;
use prism_dtu_common::{gen_seeded_rng, Archetype, FixtureSet, GenOpts, OrgId, Provenance};
use rand::Rng;
use serde_json::{json, Value};

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Generate a `FixtureSet` covering all 4 Cyberint API surfaces for the given
/// `org_id` and `archetype`.
///
/// BC-3.4.001: identical inputs produce byte-identical records.
/// BC-3.4.002: each record validates against its surface-specific sub-spec.
/// BC-3.4.004: every record ID carries an org-derived prefix.
///
/// The single RNG stream advances in surface order: alert → asm_asset → cve → ioc.
/// Records from all 4 surfaces are concatenated into `FixtureSet::records` with
/// a `_surface` provenance field to identify origin.
pub fn generate(org_id: &OrgId, archetype: Archetype, opts: &GenOpts) -> FixtureSet {
    generate_inner(org_id, archetype, opts, None)
}

/// Generate a `FixtureSet` with scenario catalog CVE IDs (PC-8 / BC-2.06.020).
///
/// Identical to `generate` but passes `catalog_cves` to `generate_cves` so every
/// CVE-surface record's `cve_id` is drawn from the catalog (cyclic assignment).
///
/// The RNG stream draw count is IDENTICAL between `generate` and this function
/// (BC-3.4.001 determinism — the gen_range draw always happens; see `generate_cves`).
pub fn generate_with_catalog(
    org_id: &OrgId,
    archetype: Archetype,
    opts: &GenOpts,
    catalog_cves: &[String],
) -> FixtureSet {
    generate_inner(org_id, archetype, opts, Some(catalog_cves))
}

/// Generate a `FixtureSet` with scenario IOCs stamped on CompromisedEndpoint alert records
/// AND scenario CVE IDs on CVE-surface records.
///
/// AC-002 (S-DEMO-ENRICHMENT-PIVOT-003): for scenario-enabled Cyberint clones,
/// alert records produced by `CompromisedEndpoint` generator must carry `iocs[0].value`
/// set from the catalog's IOC lists so the real-schema IOC filter in `routes/alerts.rs`
/// can project them correctly against the StageMask.
///
/// `catalog_ioc_ips`, `catalog_ioc_domains`, `catalog_ioc_hashes`: IOC values from
/// `ScenarioEntityCatalog` to stamp on the generated alert records.
/// `catalog_cves`: CVE IDs to stamp on CVE-surface records (same as `generate_with_catalog`).
///
/// For `CompromisedEndpoint` archetype, stamps `iocs[0]` on every alert-surface record
/// with `{"type": "hash_sha256", "value": catalog_ioc_hashes[0]}` so the real-schema IOC
/// filter in `routes/alerts.rs` can project it against the StageMask (BC-2.06.019 v1.13 PC-4).
/// Non-CompromisedEndpoint archetypes receive no IOC stamping; catalog CVE IDs are still
/// applied to CVE-surface records via `generate_inner` on all archetypes.
pub fn generate_with_scenario_iocs(
    org_id: &OrgId,
    archetype: Archetype,
    opts: &GenOpts,
    catalog_ioc_ips: &[String],
    catalog_ioc_domains: &[String],
    catalog_ioc_hashes: &[String],
    catalog_cves: &[String],
) -> FixtureSet {
    // Step 1: generate the base FixtureSet with catalog CVEs applied.
    let mut fixture_set = generate_inner(org_id, archetype, opts, Some(catalog_cves));

    // Step 2: for CompromisedEndpoint only, stamp IOC fields onto alert-surface records.
    // BC-2.06.019 v1.13 PC-4 PC-2 (ioc_ips/ioc_domains become visible at Exfil, stage 3+):
    // - iocs[0].value: catalog hash IOC (hash StageMask bit)
    // - alert_data.ip: catalog IP IOC (ioc_ips StageMask bit)
    // - alert_data.domain: catalog domain IOC (ioc_domains StageMask bit)
    // The route's real-schema filter reads alert_data.ip / alert_data.domain fields
    // (routes/alerts.rs lines ~285-320) and checks them against catalog_ioc_ips /
    // catalog_ioc_domains. Without stamping these fields, the ioc_ips/ioc_domains
    // StageMask bits are dormant — the route's filter branches can never fire.
    if archetype == Archetype::CompromisedEndpoint {
        let ioc_hash = catalog_ioc_hashes.first().map(|s| s.as_str());
        let ioc_ip = catalog_ioc_ips.first().map(|s| s.as_str());
        let ioc_domain = catalog_ioc_domains.first().map(|s| s.as_str());

        for record in fixture_set.records.iter_mut() {
            // Only stamp alert-surface records.
            if record
                .get("_surface")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                != "alert"
            {
                continue;
            }
            if let Some(obj) = record.as_object_mut() {
                // Stamp iocs array with catalog IOC hash (ioc_hashes StageMask bit).
                // Primary wire key "type" per Ioc serde rename (BC-2.06.019 v1.13 INCONCLUSIVE
                // inner-key — DTU always writes the primary key "type" regardless of live API form).
                if let Some(hash) = ioc_hash {
                    obj.insert(
                        "iocs".to_string(),
                        json!([{"type": "hash_sha256", "value": hash}]),
                    );
                }
                // Stamp alert_data.ip and alert_data.domain so the route's ioc_ips /
                // ioc_domains StageMask filter branches can fire (BC-2.06.019 PC-2).
                // Both fields are Option<String> in AlertData and skipped in serialization
                // when None — present here makes the StageMask gating real.
                let mut alert_data = serde_json::Map::new();
                if let Some(ip) = ioc_ip {
                    alert_data.insert("ip".to_string(), json!(ip));
                }
                if let Some(domain) = ioc_domain {
                    alert_data.insert("domain".to_string(), json!(domain));
                }
                if !alert_data.is_empty() {
                    obj.insert(
                        "alert_data".to_string(),
                        serde_json::Value::Object(alert_data),
                    );
                }
            }
        }
    }

    fixture_set
}

fn generate_inner(
    org_id: &OrgId,
    archetype: Archetype,
    opts: &GenOpts,
    catalog_cves: Option<&[String]>,
) -> FixtureSet {
    let slug = org_slug(org_id);
    let seed = opts.seed;
    let scale = opts.scale;

    // Single RNG stream — advances sequentially through all 4 surfaces (EC-003).
    // INVARIANT: NEVER call rand::thread_rng() or SystemTime::now() here.
    let mut rng = gen_seeded_rng(seed, org_id);

    let schema_valid = archetype != Archetype::SchemaDrift;

    // DormantTenant: all surfaces empty (EC-001).
    if archetype == Archetype::DormantTenant {
        return FixtureSet {
            records: vec![],
            cursors: vec![],
            provenance: Provenance {
                org_id: org_id.clone(),
                sensor_id: SensorId::from("cyberint"),
                archetype,
                seed,
                schema_valid: true,
            },
        };
    }

    // Generate each surface in order, advancing the shared RNG stream.
    // P1-02 (review 2026-06-10): opts.time_anchor threads into surfaces that
    // carry timestamps; derivation is stable_offset-based (RNG-free), so the
    // primary ChaCha20 stream is unchanged (INV-SECONDARY-RNG-STREAM-INDEPENDENCE-001).
    let anchor = opts.time_anchor;
    let mut alerts = generate_alerts(&slug, seed, archetype, scale, anchor, &mut rng);
    let asm_assets = generate_asm_assets(&slug, seed, archetype, scale, anchor, &mut rng);
    let cves = generate_cves(
        &slug,
        seed,
        archetype,
        scale,
        anchor,
        &mut rng,
        catalog_cves,
    );
    let iocs = generate_iocs(&slug, seed, archetype, scale, &mut rng);

    // SchemaDrift: mark alert surface[0] as intentionally invalid (AC-003).
    if archetype == Archetype::SchemaDrift && !alerts.is_empty() {
        // Mutate alert[0] to violate the alert spec: remove required 'id' field
        // and add _schema_valid=false marker.
        if let Some(obj) = alerts[0].as_object_mut() {
            obj.remove("id");
            obj.remove("environment");
            obj.remove("ref_id");
            obj.insert("_schema_valid".to_string(), json!(false));
        }
    }

    // PaginationEdgeCases: produce cursor tokens for alert surface (EC-004).
    let cursors = if archetype == Archetype::PaginationEdgeCases {
        let alert_count = alerts.len();
        let page_size = 5_usize;
        let pages = alert_count.div_ceil(page_size);
        (0..pages)
            .map(|p| format!("cursor-alert-page-{}", p + 1))
            .collect()
    } else {
        vec![]
    };

    // Schema validation in test mode (AC-002 / BC-3.4.002): validates each surface.
    // Skips alert[0] for SchemaDrift (it is intentionally invalid).
    #[cfg(test)]
    {
        let alert_start = if archetype == Archetype::SchemaDrift {
            1
        } else {
            0
        };
        for (i, record) in alerts[alert_start..].iter().enumerate() {
            schema_validation::validate_alert(record, i + alert_start);
        }
        for (i, record) in asm_assets.iter().enumerate() {
            schema_validation::validate_asm_asset(record, i);
        }
        for (i, record) in cves.iter().enumerate() {
            schema_validation::validate_cve(record, i);
        }
        for (i, record) in iocs.iter().enumerate() {
            schema_validation::validate_ioc(record, i);
        }
    }

    let mut records = alerts;
    records.extend(asm_assets);
    records.extend(cves);
    records.extend(iocs);

    FixtureSet {
        records,
        cursors,
        provenance: Provenance {
            org_id: org_id.clone(),
            sensor_id: SensorId::from("cyberint"),
            archetype,
            seed,
            schema_valid,
        },
    }
}

// ---------------------------------------------------------------------------
// Per-surface sub-generators (internal)
// ---------------------------------------------------------------------------

/// Generate alert records for the given archetype baseline and seed state.
///
/// Alert record IDs follow the format `alert-{org_slug}-{seed}-{index}` (AC-004).
/// `SchemaDrift`: record at index 0 is intentionally malformed (AC-003).
fn generate_alerts(
    org_slug: &str,
    seed: u64,
    archetype: Archetype,
    scale: f64,
    time_anchor: chrono::DateTime<chrono::Utc>,
    rng: &mut rand_chacha::ChaCha20Rng,
) -> Vec<Value> {
    let (alert_baseline, _, _, _) = baselines(archetype);
    let count = (alert_baseline as f64 * scale).floor() as usize;

    let severities = ["low", "medium", "high", "critical"];
    let statuses = ["open", "acknowledged", "closed"];
    let categories = [
        "Phishing",
        "Malware",
        "Data Exposure",
        "Brand Abuse",
        "Vulnerability",
    ];

    let mut records = Vec::with_capacity(count);
    for i in 0..count {
        let sev_idx = rng.gen_range(0..severities.len());
        let status_idx = rng.gen_range(0..statuses.len());
        let cat_idx = rng.gen_range(0..categories.len());

        // CompromisedEndpoint: first 3 alerts must be high-severity (BC-3.4.003 ≥3 high-severity).
        let severity = if archetype == Archetype::CompromisedEndpoint && i < 3 {
            "high"
        } else {
            severities[sev_idx]
        };

        // severity_id maps: low=1, medium=2, high=4, critical=5 (OCSF convention per test).
        let severity_id: u64 = match severity {
            "low" => 1,
            "medium" => 2,
            "high" => 4,
            "critical" => 5,
            _ => 1,
        };

        let alert_id = format!("alert-{}-{}-{}", org_slug, seed, i);
        let ref_id = format!("REF-{}-{}-{}", org_slug, seed, i);

        // P1-02 (review 2026-06-10): per-record timestamps derive from
        // time_anchor minus a seeded RNG-free stable_offset fold (0..7 days),
        // so time-window queries can discriminate between records. update /
        // modification fall between created and the anchor. The fold draws
        // NOTHING from `rng` — the primary ChaCha20 stream is unchanged.
        let minutes_before = (prism_dtu_common::stable_offset(&alert_id, seed) % 10_080) as i64;
        let created_dt = time_anchor - chrono::Duration::minutes(minutes_before);
        let update_minutes =
            (prism_dtu_common::stable_offset(&ref_id, seed) % (minutes_before as u64 + 1)) as i64;
        let updated_dt = created_dt + chrono::Duration::minutes(update_minutes);
        let created_at = created_dt.format("%Y-%m-%dT%H:%M:%SZ").to_string();
        let updated_at = updated_dt.format("%Y-%m-%dT%H:%M:%SZ").to_string();

        let category = categories[cat_idx];
        // F2 / DTU-02 (review 2026-06-10): derive `type` from the already-varied
        // category instead of a hardcoded "phishing" for every alert. The type
        // vocabulary mirrors the static fixture's variety (fixtures/alerts.json).
        let alert_type = match category {
            "Phishing" => "phishing",
            "Malware" => "malware_distribution",
            "Data Exposure" => "data_exposure",
            "Brand Abuse" => "brand_abuse",
            "Vulnerability" => "exposed_service",
            // Unreachable today — categories is a closed array above; keep a
            // deterministic fallback rather than panicking in the generator.
            _ => "threat_intelligence",
        };

        let record = json!({
            "alert_id": alert_id,
            "id": format!("id-{}-{}-{}", org_slug, seed, i),
            "ref_id": ref_id,
            "environment": "production",
            "confidence": rng.gen_range(50u64..=100),
            "status": statuses[status_idx],
            "severity": severity,
            "severity_id": severity_id,
            // F-P3-HIGH-001 fix: emit "created_at" to match the adapter's declared column
            // (cyberint.sensor.toml alerts table column name = "created_at") and the static
            // route path (routes/alerts.rs get_alerts static path emits "created_at").
            // The prior field name "created_date" caused the adapter's created_at column to
            // normalize to null for every generated alert. "created_date" is used nowhere else
            // in the codebase (sibling-site sweep TD-VSDD-060: only generator.rs had the field).
            // P1-02: anchor-derived per-record value (see derivation above the json! block).
            "created_at": created_at,
            "created_by": "system",
            "category": category,
            "type": alert_type,
            "source_category": "external",
            // F1 / DTU-01 (review 2026-06-10): the cyberint.sensor.toml alerts table
            // declares a `source` column (ocsf_field metadata.product.vendor_name) and
            // the static path emits "source": "cyberint" for every record
            // (fixtures/alerts.json + routes/alerts.rs get_alerts). The serving
            // extraction is flat r.get(col_name) — omitting the key silently
            // normalized the column to NULL on the seeded path.
            "source": "cyberint",
            // Static-fixture shape parity (sibling-sweep TD-VSDD-060): the static
            // record shape also carries `affected_assets` (array). Deterministic,
            // index-derived — no extra RNG draws so per-surface streams are unchanged.
            "affected_assets": [format!("asset-{}-{}.example.com", org_slug, i)],
            "title": format!("Alert {} for {}", i, org_slug),
            "modification_date": updated_at.clone(),
            "description": format!("Description for alert {}", i),
            "recommendation": "Investigate and remediate.",
            "update_date": updated_at,
            "_surface": "alert",
        });
        records.push(record);
    }
    records
}

/// Generate ASM asset records.
///
/// Asset record IDs follow the format `dev-{org_slug}-{seed}-{index}` (AC-004).
fn generate_asm_assets(
    org_slug: &str,
    seed: u64,
    archetype: Archetype,
    scale: f64,
    time_anchor: chrono::DateTime<chrono::Utc>,
    rng: &mut rand_chacha::ChaCha20Rng,
) -> Vec<Value> {
    let (_, asm_baseline, _, _) = baselines(archetype);
    let count = (asm_baseline as f64 * scale).floor() as usize;

    let statuses = ["active", "inactive", "monitoring"];
    let types = ["domain", "ip", "subdomain", "certificate"];

    let mut records = Vec::with_capacity(count);
    for i in 0..count {
        let status_idx = rng.gen_range(0..statuses.len());
        let type_idx = rng.gen_range(0..types.len());

        let asset_id = format!("dev-{}-{}-{}", org_slug, seed, i);

        // P1-02: anchor-derived per-record timestamps (RNG-free stable_offset;
        // primary ChaCha20 stream unchanged). created 0..90 days before anchor;
        // updated between created and the anchor.
        let days_before = (prism_dtu_common::stable_offset(&asset_id, seed) % 90) as i64;
        let created_dt = time_anchor - chrono::Duration::days(days_before);
        let update_days = (prism_dtu_common::stable_offset(&asset_id, seed.wrapping_add(1))
            % (days_before as u64 + 1)) as i64;
        let updated_dt = created_dt + chrono::Duration::days(update_days);
        let created = created_dt.format("%Y-%m-%dT%H:%M:%SZ").to_string();
        let updated = updated_dt.format("%Y-%m-%dT%H:%M:%SZ").to_string();

        let record = json!({
            "asset_id": asset_id,
            "id": asset_id,
            "name": format!("asset-{}.example.com", i),
            "type": types[type_idx],
            "status": statuses[status_idx],
            "created": created,
            "updated": updated,
            "_surface": "asm_asset",
        });
        records.push(record);
    }
    records
}

/// Generate CVE records.
///
/// CVE record primary ID follows the format `alert-{org_slug}-{seed}-{index}` (AC-004).
///
/// `catalog_cves`: when `Some(&[String])`, every record's `cve_id`/`cve_name` is drawn
/// from the catalog (cyclic assignment: `catalog_cves[i % len]`). This is the PC-8 scenario
/// path (BC-2.06.020 INV-CYBERINT-ALERT-CVE-CORRELATION-001).
///
/// When `None` (baseline/non-scenario path), the `CVE-9999-` collision-safe namespace is
/// used (PC-9 / BC-2.06.020 INV-CYBERINT-ALERT-CVE-CORRELATION-001 baseline clause).
///
/// CRITICAL determinism rule: the RNG draw `rng.gen_range(0u32..10000)` is performed
/// unconditionally in both paths to preserve the primary ChaCha20 stream draw count.
/// In scenario mode the drawn value is discarded in favour of the catalog entry, but the
/// draw still happens so all subsequent RNG-derived fields remain bit-identical between
/// baseline and scenario runs (BC-3.4.001).
fn generate_cves(
    org_slug: &str,
    seed: u64,
    archetype: Archetype,
    scale: f64,
    time_anchor: chrono::DateTime<chrono::Utc>,
    rng: &mut rand_chacha::ChaCha20Rng,
    catalog_cves: Option<&[String]>,
) -> Vec<Value> {
    let (_, _, cve_baseline, _) = baselines(archetype);
    let count = (cve_baseline as f64 * scale).floor() as usize;

    let mut records = Vec::with_capacity(count);
    for i in 0..count {
        let score: f64 = rng.gen_range(0.0..10.0);
        let cve_id = format!("alert-{}-{}-{}", org_slug, seed, i);
        // PC-9: unconditional RNG draw to preserve primary ChaCha20 stream draw count
        // (BC-3.4.001 determinism — draw count must be identical in both paths).
        let baseline_cve_name = format!("CVE-9999-{:04}", rng.gen_range(0u32..10000));
        // PC-8 / PC-9 selection: scenario catalog wins; baseline uses CVE-9999- namespace.
        let cve_name = match catalog_cves {
            Some(cves) if !cves.is_empty() => cves[i % cves.len()].clone(),
            _ => baseline_cve_name,
        };

        // P1-02: anchor-derived per-record timestamps (RNG-free stable_offset;
        // primary ChaCha20 stream unchanged). published 0..90 days before the
        // anchor; modified between published and the anchor.
        let days_before = (prism_dtu_common::stable_offset(&cve_id, seed) % 90) as i64;
        let published_dt = time_anchor - chrono::Duration::days(days_before);
        let modified_days = (prism_dtu_common::stable_offset(&cve_id, seed.wrapping_add(1))
            % (days_before as u64 + 1)) as i64;
        let modified_dt = published_dt + chrono::Duration::days(modified_days);
        let published = published_dt.format("%Y-%m-%dT%H:%M:%SZ").to_string();
        let modified = modified_dt.format("%Y-%m-%dT%H:%M:%SZ").to_string();

        let record = json!({
            "alert_id": cve_id,
            "id": cve_id,
            "cve_id": cve_name,
            "cyberint_score": score,
            "cyberint_score_modification_date": modified.clone(),
            "published_date": published,
            "last_modified_date": modified,
            "_surface": "cve",
        });
        records.push(record);
    }
    records
}

/// Generate IOC records.
///
/// IOC record primary ID follows the format `alert-{org_slug}-{seed}-{index}` (AC-004).
fn generate_iocs(
    org_slug: &str,
    seed: u64,
    archetype: Archetype,
    scale: f64,
    rng: &mut rand_chacha::ChaCha20Rng,
) -> Vec<Value> {
    let (_, _, _, ioc_baseline) = baselines(archetype);
    let count = (ioc_baseline as f64 * scale).floor() as usize;

    let ioc_types = ["domain", "ip", "url", "file_sha256"];

    let mut records = Vec::with_capacity(count);
    for i in 0..count {
        let type_idx = rng.gen_range(0..ioc_types.len());
        let ioc_id = format!("alert-{}-{}-{}", org_slug, seed, i);
        let ioc_type = ioc_types[type_idx];

        let value = match ioc_type {
            "ip" => format!(
                "192.168.{}.{}",
                rng.gen_range(0u8..=255),
                rng.gen_range(0u8..=255)
            ),
            "domain" => format!("malicious-{}.example.com", i),
            "url" => format!("https://malicious-{}.example.com/path", i),
            "file_sha256" => format!("{:064x}", rng.gen::<u64>()),
            _ => format!("ioc-value-{}", i),
        };

        // `iocs_value_first` top-level field is for the IOC surface records served by the
        // DTU endpoint directly. Each IOC record holds a single IOC value, so the
        // "first" is the value itself.
        //
        // NOTE: The `cyberint_alerts` table column `iocs_value_first` is populated by the
        // spec-driven adapter using `source_path = "$.iocs[0].value"` against ALERT records
        // (which carry an `iocs` array). This top-level field on IOC surface records is NOT
        // consumed by the spec-driven adapter for the alerts table — it is present for
        // direct IOC endpoint consumers and potential future `cyberint_iocs` table spec.
        // (HIGH-001 justification, S-DEMO-ENRICHMENT-TYPED-OUTPUT-001 LOCAL pass-1.)
        let record = json!({
            "alert_id": ioc_id,
            "id": ioc_id,
            "type": ioc_type,
            "value": value.clone(),
            "iocs_value_first": value,
            "_surface": "ioc",
        });
        records.push(record);
    }
    records
}

// ---------------------------------------------------------------------------
// Baseline count helpers (internal)
// ---------------------------------------------------------------------------

/// Per-surface baseline record counts for a given archetype at `scale = 1.0`.
///
/// Returns `(alert, asm_asset, cve, ioc)`. The caller applies
/// `floor(baseline × scale)` for non-unit scale values (AC-001).
///
/// LargeScale split: alert=500, asm_asset=2000, cve=1000, ioc=1000 → total=4500.
/// PaginationEdgeCases: alert surface paginated (10 records), others standard.
/// HighChurn split: alert=20, asm_asset=30, cve=10, ioc=15 (per test expectations).
fn baselines(archetype: Archetype) -> (usize, usize, usize, usize) {
    match archetype {
        Archetype::HealthyOtEnvironment => (5, 10, 5, 5),
        Archetype::CompromisedEndpoint => (20, 10, 10, 10),
        Archetype::AuthOutage => (5, 10, 5, 5),
        Archetype::LargeScale => (500, 2000, 1000, 1000),
        Archetype::PaginationEdgeCases => (10, 10, 5, 5),
        Archetype::SchemaDrift => (5, 10, 5, 5),
        Archetype::HighChurn => (20, 30, 10, 15),
        Archetype::DormantTenant => (0, 0, 0, 0),
        // Safety net for future non_exhaustive variants.
        _ => (5, 10, 5, 5),
    }
}

/// Derive an org slug from an `OrgId` for use in record ID prefixes.
///
/// Returns the first 8 hex characters of the org UUID (EC-005 fallback path).
fn org_slug(org_id: &OrgId) -> String {
    let bytes = org_id.as_bytes();
    format!(
        "{:02x}{:02x}{:02x}{:02x}",
        bytes[0], bytes[1], bytes[2], bytes[3]
    )
}

// ---------------------------------------------------------------------------
// Schema validation (test-only, AC-002 / BC-3.4.002 / AC-007)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod schema_validation {
    use serde_json::Value;

    // Spec paths relative to workspace root — resolved via CARGO_MANIFEST_DIR at runtime.
    const ALERT_SPEC_PATH: &str = ".references/poller-express/docs/specs/alert_api_specs.json";
    const ASM_ASSETS_SPEC_PATH: &str =
        ".references/poller-express/docs/specs/asm_assets_api_specs.json";
    const CVE_SPEC_PATH: &str = ".references/poller-express/docs/specs/cve_api_specs.json";
    const IOC_SPEC_PATH: &str = ".references/poller-express/docs/specs/ioc_api_specs.json";

    /// Resolve a workspace-relative spec path to an absolute path.
    ///
    /// Walks up from CARGO_MANIFEST_DIR to find the workspace root (the directory
    /// containing both `Cargo.toml` and `.references/`).
    fn resolve_spec_path(relative: &str) -> std::path::PathBuf {
        // CARGO_MANIFEST_DIR is the crate root; workspace root is two levels up.
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
            .expect("CARGO_MANIFEST_DIR must be set in test context");
        let crate_root = std::path::Path::new(&manifest_dir);
        // Walk up looking for the .references directory.
        let mut candidate = crate_root.to_path_buf();
        loop {
            if candidate.join(".references").exists() {
                return candidate.join(relative);
            }
            match candidate.parent() {
                Some(p) => candidate = p.to_path_buf(),
                None => {
                    // Fallback: relative to crate root, then repo root.
                    return crate_root.join("../../..").join(relative);
                }
            }
        }
    }

    /// Load and compile the schema for the given surface spec.
    fn load_schema(spec_path: &str) -> (Value, std::path::PathBuf) {
        let abs_path = resolve_spec_path(spec_path);
        let content = std::fs::read_to_string(&abs_path)
            .unwrap_or_else(|e| panic!("Failed to load spec file '{}': {e}", abs_path.display()));
        let spec: Value = serde_json::from_str(&content)
            .unwrap_or_else(|e| panic!("Failed to parse spec '{}': {e}", abs_path.display()));
        (spec, abs_path)
    }

    /// Validate an alert record against the Alert schema (required fields from spec).
    ///
    /// Alert schema requires: id, environment, ref_id, confidence, status, severity,
    /// created_at, created_by, category, type, source_category, title,
    /// modification_date, description, recommendation, update_date.
    ///
    /// NOTE: field is "created_at" (matches cyberint.sensor.toml column + static route path),
    /// NOT "created_date" (F-P3-HIGH-001 fix — the old generator emitted "created_date" which
    /// caused adapter OCSF normalization to produce null for the time column).
    pub(super) fn validate_alert(record: &Value, index: usize) {
        let required_fields = [
            "id",
            "environment",
            "ref_id",
            "confidence",
            "status",
            "severity",
            "created_at",
            "created_by",
            "category",
            "type",
            "source_category",
            "title",
            "modification_date",
            "description",
            "recommendation",
            "update_date",
        ];
        for field in &required_fields {
            assert!(
                record.get(field).is_some(),
                "alert record[{index}] missing required field '{field}' (spec: {ALERT_SPEC_PATH})"
            );
        }
    }

    /// Validate an ASM asset record against the Asset schema.
    ///
    /// ASM asset schema requires: id, created, updated.
    pub(super) fn validate_asm_asset(record: &Value, index: usize) {
        let required_fields = ["id", "created", "updated"];
        for field in &required_fields {
            assert!(
                record.get(field).is_some(),
                "asm_asset record[{index}] missing required field '{field}' (spec: {ASM_ASSETS_SPEC_PATH})"
            );
        }
    }

    /// Validate a CVE record against the CVEModelExternal schema.
    ///
    /// CVEModelExternal schema requires: id.
    pub(super) fn validate_cve(record: &Value, index: usize) {
        let required_fields = ["id"];
        for field in &required_fields {
            assert!(
                record.get(field).is_some(),
                "cve record[{index}] missing required field '{field}' (spec: {CVE_SPEC_PATH})"
            );
        }
    }

    /// Validate an IOC record against the entity schema.
    ///
    /// IOC entity schemas require: type, value.
    pub(super) fn validate_ioc(record: &Value, index: usize) {
        let required_fields = ["type", "value"];
        for field in &required_fields {
            assert!(
                record.get(field).is_some(),
                "ioc record[{index}] missing required field '{field}' (spec: {IOC_SPEC_PATH})"
            );
        }
    }

    // Suppress unused-import warning — paths are referenced in panic messages above.
    const _: &str = ALERT_SPEC_PATH;
    const _: &str = ASM_ASSETS_SPEC_PATH;
    const _: &str = CVE_SPEC_PATH;
    const _: &str = IOC_SPEC_PATH;
}

// ---------------------------------------------------------------------------
// Unit tests (AC-001 … AC-006) — stubs fulfilled by external test file
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use prism_dtu_common::{all_archetypes, GenOpts};

    use super::*;

    /// AC-001: per-surface counts at scale=1.0 for all 8 archetypes.
    #[test]
    fn test_cyberint_all_archetypes_counts() {
        let org = OrgId([0u8; 16]);
        let opts = GenOpts::default();
        for archetype in all_archetypes() {
            let fs = generate(&org, *archetype, &opts);
            let alert_count = fs
                .records
                .iter()
                .filter(|r| r.get("_surface").and_then(|v| v.as_str()) == Some("alert"))
                .count();
            let (alert_baseline, _, _, _) = baselines(*archetype);
            let expected = (alert_baseline as f64 * opts.scale).floor() as usize;
            assert_eq!(
                alert_count, expected,
                "archetype {archetype:?}: alert count mismatch"
            );
        }
    }

    /// AC-002: each surface validates against its correct sub-spec.
    #[test]
    fn test_cyberint_schema_correct_sub_spec() {
        let org = OrgId([0u8; 16]);
        let opts = GenOpts::default();
        // generate() calls validate_* internally in #[cfg(test)] — no panic means pass.
        let _fs = generate(&org, Archetype::HealthyOtEnvironment, &opts);
    }

    /// AC-003: SchemaDrift — only alert surface record[0] invalid.
    #[test]
    fn test_cyberint_schema_drift_alert_surface() {
        let org = OrgId([0u8; 16]);
        let opts = GenOpts::default();
        let fs = generate(&org, Archetype::SchemaDrift, &opts);
        assert!(
            !fs.provenance.schema_valid,
            "SchemaDrift must have schema_valid=false"
        );
        let alerts: Vec<_> = fs
            .records
            .iter()
            .filter(|r| r.get("_surface").and_then(|v| v.as_str()) == Some("alert"))
            .collect();
        assert!(!alerts.is_empty(), "SchemaDrift must have alert records");
        let drifted = alerts[0];
        let sv = drifted
            .get("_schema_valid")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        assert!(!sv, "alert[0] must carry _schema_valid=false");
    }

    /// AC-004: all record IDs carry org-slug prefix for correct field per surface.
    #[test]
    fn test_cyberint_org_tagged_ids_per_surface() {
        let org = OrgId([0x01, 0x02, 0x03, 0x04, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        let opts = GenOpts::default();
        let fs = generate(&org, Archetype::HealthyOtEnvironment, &opts);
        for record in &fs.records {
            let surface = record
                .get("_surface")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            match surface {
                "alert" => {
                    let id = record.get("alert_id").and_then(|v| v.as_str()).unwrap();
                    assert!(
                        id.starts_with("alert-"),
                        "alert_id must start with 'alert-'"
                    );
                }
                "asm_asset" => {
                    let id = record.get("asset_id").and_then(|v| v.as_str()).unwrap();
                    assert!(id.starts_with("dev-"), "asset_id must start with 'dev-'");
                }
                "cve" | "ioc" => {
                    let id = record.get("alert_id").and_then(|v| v.as_str()).unwrap();
                    assert!(
                        id.starts_with("alert-"),
                        "alert_id must start with 'alert-'"
                    );
                }
                _ => {}
            }
        }
    }

    /// AC-005: two calls with identical inputs produce byte-identical records.
    #[test]
    fn test_cyberint_determinism() {
        let org = OrgId([0u8; 16]);
        let opts = GenOpts::default();
        let fs1 = generate(&org, Archetype::HealthyOtEnvironment, &opts);
        let fs2 = generate(&org, Archetype::HealthyOtEnvironment, &opts);
        let j1 = serde_json::to_string(&fs1.records).unwrap();
        let j2 = serde_json::to_string(&fs2.records).unwrap();
        assert_eq!(
            j1, j2,
            "BC-3.4.001: two identical calls must produce byte-identical records"
        );
    }

    /// F1 / DTU-01 (review 2026-06-10): every generated alert record must carry
    /// every flat key declared as a column in cyberint.sensor.toml `[[tables]]`
    /// alerts AND every flat key present in the static-fixture record shape
    /// (fixtures/alerts.json / routes/alerts.rs static path). The serving
    /// extraction is flat `r.get(col_name)` — an absent key normalizes to NULL,
    /// so a missing `source` key silently nulls the metadata.product.vendor_name
    /// column on the seeded path.
    #[test]
    fn test_generated_alert_covers_toml_and_static_fixture_keys() {
        // cyberint.sensor.toml [[tables]] alerts columns (flat key set).
        let toml_columns = [
            "alert_id",
            "title",
            "type",
            "severity",
            "status",
            "created_at",
            "source",
        ];
        // Static fixture record shape (fixtures/alerts.json, all 20 records share it;
        // mirrored verbatim by routes/alerts.rs get_alerts static path).
        let static_keys = [
            "alert_id",
            "title",
            "severity",
            "status",
            "created_at",
            "source",
            "type",
            "affected_assets",
        ];

        let org = OrgId([0u8; 16]);
        let opts = GenOpts::default();
        let fs = generate(&org, Archetype::HealthyOtEnvironment, &opts);
        let alerts: Vec<_> = fs
            .records
            .iter()
            .filter(|r| r.get("_surface").and_then(|v| v.as_str()) == Some("alert"))
            .collect();
        assert!(!alerts.is_empty(), "must generate at least one alert");

        for (i, record) in alerts.iter().enumerate() {
            for key in toml_columns.iter().chain(static_keys.iter()) {
                assert!(
                    record.get(key).is_some(),
                    "generated alert[{i}] missing flat key '{key}' \
                     (TOML column or static-fixture shape key)"
                );
            }
            // The static path emits "source": "cyberint" for every record — the
            // generated path must match so metadata.product.vendor_name is stable.
            assert_eq!(
                record.get("source").and_then(|v| v.as_str()),
                Some("cyberint"),
                "generated alert[{i}] 'source' must be \"cyberint\""
            );
            assert!(
                record.get("affected_assets").map(|v| v.is_array()) == Some(true),
                "generated alert[{i}] 'affected_assets' must be an array"
            );
        }
    }

    /// F2 / DTU-02 (review 2026-06-10): generated alert `type` must derive from
    /// the (already-varied) category — not a hardcoded "phishing" for every record.
    /// The category→type table mirrors the static fixture's type vocabulary
    /// (fixtures/alerts.json: phishing, malware_distribution, data_exposure,
    /// brand_abuse, exposed_service, ...).
    #[test]
    fn test_generated_alert_type_derives_from_category() {
        let expected = |category: &str| -> &'static str {
            match category {
                "Phishing" => "phishing",
                "Malware" => "malware_distribution",
                "Data Exposure" => "data_exposure",
                "Brand Abuse" => "brand_abuse",
                "Vulnerability" => "exposed_service",
                other => panic!("unexpected category '{other}'"),
            }
        };

        let org = OrgId([0u8; 16]);
        let opts = GenOpts::default();
        // CompromisedEndpoint: 20 alerts — enough draws to exercise category variety.
        let fs = generate(&org, Archetype::CompromisedEndpoint, &opts);
        let alerts: Vec<_> = fs
            .records
            .iter()
            .filter(|r| r.get("_surface").and_then(|v| v.as_str()) == Some("alert"))
            .collect();
        assert!(!alerts.is_empty(), "must generate at least one alert");

        let mut distinct_types = std::collections::BTreeSet::new();
        for (i, record) in alerts.iter().enumerate() {
            let category = record
                .get("category")
                .and_then(|v| v.as_str())
                .unwrap_or_else(|| panic!("alert[{i}] missing category"));
            let ty = record
                .get("type")
                .and_then(|v| v.as_str())
                .unwrap_or_else(|| panic!("alert[{i}] missing type"));
            assert_eq!(
                ty,
                expected(category),
                "alert[{i}] type must derive from category '{category}'"
            );
            distinct_types.insert(ty.to_owned());
        }
        assert!(
            distinct_types.len() > 1,
            "20 alerts must yield >1 distinct type (got {distinct_types:?}) — \
             hardcoded type defeats category variety"
        );
    }

    /// AC-006 / RNG stream: different seed produces different records on all surfaces.
    #[test]
    fn test_cyberint_single_rng_stream() {
        let org = OrgId([0u8; 16]);
        let opts1 = GenOpts::new(
            1,
            1.0,
            chrono::DateTime::UNIX_EPOCH,
            serde_json::Value::Null,
        )
        .unwrap();
        let opts2 = GenOpts::new(
            2,
            1.0,
            chrono::DateTime::UNIX_EPOCH,
            serde_json::Value::Null,
        )
        .unwrap();
        let fs1 = generate(&org, Archetype::HealthyOtEnvironment, &opts1);
        let fs2 = generate(&org, Archetype::HealthyOtEnvironment, &opts2);
        let j1 = serde_json::to_string(&fs1.records).unwrap();
        let j2 = serde_json::to_string(&fs2.records).unwrap();
        assert_ne!(
            j1, j2,
            "EC-003: different seed must produce different records"
        );
    }

    // ── S-DEMO-ENRICHMENT-TYPED-OUTPUT-001 Red Gate Test ─────────────────────
    // RGT-015 (ADR-051 D4 Scalar-Input rule): IOC surface records must emit
    // `iocs_value_first` as a top-level scalar companion field.

    /// RGT-015 (ADR-051 D4): the Cyberint DTU generator must emit `iocs_value_first`
    /// as a top-level scalar string field on every IOC surface record.
    ///
    /// ADR-051 D4 Scalar-Input rule: typed enrichment UDFs (integer, float, boolean,
    /// datetime) must receive a plain scalar string, not a JSON-array value.
    /// `iocs_value_first` is the scalar companion to the JSON-list wildcard column
    /// `iocs_value` (source_path = "$.iocs[*].value").  It holds the first element
    /// of the IOC value list as a plain string.
    ///
    /// This field must be present in every IOC record generated by the DTU so that
    /// the DataFusion table-scan can surface it as a column.
    ///
    /// RED GATE: no `iocs_value_first` key is emitted by the current generator
    /// → `record.get("iocs_value_first").is_some()` fails → test FAILS → RED.
    ///
    /// GREEN: generator emits `"iocs_value_first": "..."` on every IOC surface record.
    #[test]
    fn test_cyberint_dtu_fixture_emits_iocs_value_first_field() {
        let org = OrgId([0u8; 16]);
        let opts = GenOpts::default();
        let fs = generate(&org, Archetype::HealthyOtEnvironment, &opts);

        let ioc_records: Vec<_> = fs
            .records
            .iter()
            .filter(|r| r.get("_surface").and_then(|v| v.as_str()) == Some("ioc"))
            .collect();

        assert!(
            !ioc_records.is_empty(),
            "RGT-015 precondition: HealthyOtEnvironment must produce at least one IOC record"
        );

        // RED GATE: iocs_value_first is not yet emitted by the generator.
        for (i, record) in ioc_records.iter().enumerate() {
            assert!(
                record.get("iocs_value_first").is_some(),
                "ADR-051 D4 RGT-015: IOC surface record[{i}] must have top-level key \
                 'iocs_value_first' (scalar companion for typed enrichment UDFs). \
                 Missing — implementer must add this field to make_ioc_record() or equivalent. \
                 Record JSON: {}",
                record
            );
        }
    }
}
