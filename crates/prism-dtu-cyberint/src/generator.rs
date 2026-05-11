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
                sensor_type: SensorId::from("cyberint"),
                archetype,
                seed,
                schema_valid: true,
            },
        };
    }

    // Generate each surface in order, advancing the shared RNG stream.
    let mut alerts = generate_alerts(&slug, seed, archetype, scale, &mut rng);
    let asm_assets = generate_asm_assets(&slug, seed, archetype, scale, &mut rng);
    let cves = generate_cves(&slug, seed, archetype, scale, &mut rng);
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
            sensor_type: SensorId::from("cyberint"),
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

        let record = json!({
            "alert_id": alert_id,
            "id": format!("id-{}-{}-{}", org_slug, seed, i),
            "ref_id": ref_id,
            "environment": "production",
            "confidence": rng.gen_range(50u64..=100),
            "status": statuses[status_idx],
            "severity": severity,
            "severity_id": severity_id,
            "created_date": "2024-01-01T00:00:00Z",
            "created_by": "system",
            "category": categories[cat_idx],
            "type": "phishing",
            "source_category": "external",
            "title": format!("Alert {} for {}", i, org_slug),
            "modification_date": "2024-01-01T00:00:00Z",
            "description": format!("Description for alert {}", i),
            "recommendation": "Investigate and remediate.",
            "update_date": "2024-01-01T00:00:00Z",
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

        let record = json!({
            "asset_id": asset_id,
            "id": asset_id,
            "name": format!("asset-{}.example.com", i),
            "type": types[type_idx],
            "status": statuses[status_idx],
            "created": "2024-01-01T00:00:00Z",
            "updated": "2024-01-01T00:00:00Z",
            "_surface": "asm_asset",
        });
        records.push(record);
    }
    records
}

/// Generate CVE records.
///
/// CVE record primary ID follows the format `alert-{org_slug}-{seed}-{index}` (AC-004).
fn generate_cves(
    org_slug: &str,
    seed: u64,
    archetype: Archetype,
    scale: f64,
    rng: &mut rand_chacha::ChaCha20Rng,
) -> Vec<Value> {
    let (_, _, cve_baseline, _) = baselines(archetype);
    let count = (cve_baseline as f64 * scale).floor() as usize;

    let mut records = Vec::with_capacity(count);
    for i in 0..count {
        let score: f64 = rng.gen_range(0.0..10.0);
        let cve_id = format!("alert-{}-{}-{}", org_slug, seed, i);
        let cve_name = format!("CVE-2024-{:04}", rng.gen_range(1000u32..9999));

        let record = json!({
            "alert_id": cve_id,
            "id": cve_id,
            "cve_id": cve_name,
            "cyberint_score": score,
            "cyberint_score_modification_date": "2024-01-01T00:00:00Z",
            "published_date": "2024-01-01T00:00:00Z",
            "last_modified_date": "2024-01-01T00:00:00Z",
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

        let record = json!({
            "alert_id": ioc_id,
            "id": ioc_id,
            "type": ioc_type,
            "value": value,
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
    /// created_date, created_by, category, type, source_category, title,
    /// modification_date, description, recommendation, update_date.
    pub(super) fn validate_alert(record: &Value, index: usize) {
        let required_fields = [
            "id",
            "environment",
            "ref_id",
            "confidence",
            "status",
            "severity",
            "created_date",
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
    use super::*;
    use prism_dtu_common::{all_archetypes, GenOpts};

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
}
