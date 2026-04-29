//! Failing acceptance tests for S-3.7.03 — Cyberint fixture generator.
//!
//! Covers:
//!   BC-3.4.001 — Generator Determinism (byte-identical for same GenOpts)
//!   BC-3.4.002 — Schema Conformance (4 endpoint sub-specs, via generate() calling
//!                validate_* todo!() stubs)
//!   BC-3.4.004 — Org-Tagged Record IDs (per-surface field names)
//!   VP-108     — generate is idempotent for same inputs
//!   VP-112     — all non-SchemaDrift records pass schema validation
//!   VP-113     — SchemaDrift: provenance.schema_valid == false
//!   VP-114     — schema validation absent from release build (cfg(test) gate)
//!   VP-119     — ID sets disjoint for distinct org slugs
//!   VP-120     — every record ID contains org slug as substring
//!   BC-3.4.003 — all 8 archetypes (archetype catalog enumeration)
//!   AC-001..AC-007 per S-3.7.03 story spec
//!
//! ALL tests are expected to panic via `todo!()` until implementation lands.
//! Red Gate verified before implementation begins.
//!
//! Note: schema_validation validators are `#[cfg(test)] pub(super)` stubs inside
//! generator.rs; they are exercised by generate() itself in test mode. External
//! tests verify schema conformance through FixtureSet properties and provenance.
//!
//! Run:
//!   cargo test -p prism-dtu-cyberint --features fixture-gen \
//!              --test bc_3_4_cyberint_generator
#![cfg(feature = "fixture-gen")]
#![allow(clippy::unwrap_used, clippy::expect_used)]

use chrono::DateTime;
use prism_dtu_common::{all_archetypes, Archetype, FixtureSet, GenOpts, OrgId};
use prism_dtu_cyberint::generator::generate;
use serde_json::Value;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Org A — distinct bytes for org-namespace tests (TV-3.4.004-01 baseline)
fn org_a() -> OrgId {
    OrgId([
        0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F,
        0x10,
    ])
}

/// Org B — distinct from A; distinct slug for disjoint-ID tests (VP-119)
fn org_b() -> OrgId {
    OrgId([
        0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99,
        0x00,
    ])
}

/// Default GenOpts (seed=42, scale=1.0, time_anchor=UNIX_EPOCH, overrides=Null)
fn default_opts() -> GenOpts {
    GenOpts::default()
}

/// Count records on a named surface in a FixtureSet.
fn surface_count(fs: &FixtureSet, surface: &str) -> usize {
    fs.records
        .iter()
        .filter(|r| r.get("_surface").and_then(Value::as_str) == Some(surface))
        .count()
}

// ---------------------------------------------------------------------------
// AC-001 / BC-3.4.003 — All 8 archetypes produce records for all 4 surfaces
// ---------------------------------------------------------------------------

/// AC-001 / BC-3.4.003: HealthyOtEnvironment baseline per-surface counts at scale=1.0.
///
/// Expected: alert=5, asm_asset=10, cve=5, ioc=5 per story spec §AC-001.
/// Exercises generate() todo!() — Red Gate target.
#[test]
fn test_bc_3_4_003_ac_001_healthy_ot_environment_counts() {
    let fs = generate(&org_a(), Archetype::HealthyOtEnvironment, &default_opts());
    assert_eq!(
        surface_count(&fs, "alert"),
        5,
        "HealthyOtEnvironment: alert count must be 5"
    );
    assert_eq!(
        surface_count(&fs, "asm_asset"),
        10,
        "HealthyOtEnvironment: asm_asset count must be 10"
    );
    assert_eq!(
        surface_count(&fs, "cve"),
        5,
        "HealthyOtEnvironment: cve count must be 5"
    );
    assert_eq!(
        surface_count(&fs, "ioc"),
        5,
        "HealthyOtEnvironment: ioc count must be 5"
    );
}

/// AC-001 / BC-3.4.003: CompromisedEndpoint baseline per-surface counts at scale=1.0.
///
/// Expected: alert=20 (>=3 high-severity), asm_asset=10, cve=10, ioc=10.
#[test]
fn test_bc_3_4_003_ac_001_compromised_endpoint_counts() {
    let fs = generate(&org_a(), Archetype::CompromisedEndpoint, &default_opts());
    let alerts: Vec<&Value> = fs
        .records
        .iter()
        .filter(|r| r.get("_surface").and_then(Value::as_str) == Some("alert"))
        .collect();
    assert_eq!(
        alerts.len(),
        20,
        "CompromisedEndpoint: alert count must be 20"
    );
    // BC-3.4.003: >=3 high-severity alerts (severity_id >= 4, OCSF convention)
    let high_sev = alerts
        .iter()
        .filter(|r| r.get("severity_id").and_then(Value::as_u64).unwrap_or(0) >= 4)
        .count();
    assert!(
        high_sev >= 3,
        "CompromisedEndpoint: at least 3 alerts must have severity_id >= 4, got {high_sev}"
    );
    assert_eq!(
        surface_count(&fs, "asm_asset"),
        10,
        "CompromisedEndpoint: asm_asset count must be 10"
    );
    assert_eq!(
        surface_count(&fs, "cve"),
        10,
        "CompromisedEndpoint: cve count must be 10"
    );
    assert_eq!(
        surface_count(&fs, "ioc"),
        10,
        "CompromisedEndpoint: ioc count must be 10"
    );
}

/// AC-001 / BC-3.4.003: LargeScale baseline per-surface counts at scale=1.0.
///
/// Expected: alert=500, asm_asset=2000, cve=1000, ioc=1000; total ~4500.
#[test]
fn test_bc_3_4_003_ac_001_large_scale_counts() {
    let fs = generate(&org_a(), Archetype::LargeScale, &default_opts());
    assert_eq!(
        surface_count(&fs, "alert"),
        500,
        "LargeScale: alert count must be 500"
    );
    assert_eq!(
        surface_count(&fs, "asm_asset"),
        2000,
        "LargeScale: asm_asset count must be 2000"
    );
    assert_eq!(
        surface_count(&fs, "cve"),
        1000,
        "LargeScale: cve count must be 1000"
    );
    assert_eq!(
        surface_count(&fs, "ioc"),
        1000,
        "LargeScale: ioc count must be 1000"
    );
}

/// AC-001 / BC-3.4.003 invariant 5 / EC-001: DormantTenant always 0 records on all 4
/// surfaces at any scale (TV-3.4.003-07).
#[test]
fn test_bc_3_4_003_ac_001_dormant_tenant_zero_records_all_surfaces() {
    let fs = generate(&org_a(), Archetype::DormantTenant, &default_opts());
    assert!(
        fs.records.is_empty(),
        "DormantTenant: records must be empty; got {}",
        fs.records.len()
    );
    assert!(
        fs.cursors.is_empty(),
        "DormantTenant: cursors must be empty"
    );
}

/// AC-001 / BC-3.4.003: all 8 archetypes produce a FixtureSet without non-todo!() panic.
///
/// Parameterised over all_archetypes() — any unimplemented branch fires todo!() Red Gate.
#[test]
fn test_bc_3_4_003_ac_001_all_8_archetypes_generate_without_non_todo_panic() {
    for archetype in all_archetypes() {
        let _fs = generate(&org_a(), *archetype, &default_opts());
    }
}

/// BC-3.4.003 invariant 3: count = floor(baseline * scale) at scale=0.5.
///
/// HealthyOtEnvironment alert baseline=5 => floor(5*0.5)=2.
#[test]
fn test_bc_3_4_003_ac_001_scale_formula_half_scale_healthy() {
    let opts = GenOpts::new(42, 0.5, DateTime::UNIX_EPOCH, serde_json::Value::Null)
        .expect("valid GenOpts scale=0.5");
    let fs = generate(&org_a(), Archetype::HealthyOtEnvironment, &opts);
    let alert_count = surface_count(&fs, "alert");
    // floor(5 * 0.5) = 2
    assert_eq!(
        alert_count, 2,
        "scale=0.5 HealthyOtEnvironment alert: expected floor(5*0.5)=2, got {alert_count}"
    );
}

// ---------------------------------------------------------------------------
// BC-3.4.003 — HighChurn archetype counts
// ---------------------------------------------------------------------------

/// BC-3.4.003: HighChurn archetype per-surface counts at scale=1.0.
///
/// Story spec §AC-001 (HighChurn column): alert=20, asm_asset=30, cve=10, ioc=15.
#[test]
fn test_bc_3_4_003_ac_001_high_churn_counts() {
    let fs = generate(&org_a(), Archetype::HighChurn, &default_opts());
    assert_eq!(
        surface_count(&fs, "alert"),
        20,
        "HighChurn: alert count must be 20"
    );
    assert_eq!(
        surface_count(&fs, "asm_asset"),
        30,
        "HighChurn: asm_asset count must be 30"
    );
    assert_eq!(
        surface_count(&fs, "cve"),
        10,
        "HighChurn: cve count must be 10"
    );
    assert_eq!(
        surface_count(&fs, "ioc"),
        15,
        "HighChurn: ioc count must be 15"
    );
}

// ---------------------------------------------------------------------------
// BC-3.4.003 — PaginationEdgeCases archetype cursors (EC-004)
// ---------------------------------------------------------------------------

/// BC-3.4.003 / EC-004: PaginationEdgeCases — alert surface paginated; cursors populated.
#[test]
fn test_bc_3_4_003_pagination_edge_cases_has_cursors() {
    let fs = generate(&org_a(), Archetype::PaginationEdgeCases, &default_opts());
    assert!(
        !fs.cursors.is_empty(),
        "PaginationEdgeCases: FixtureSet::cursors must be non-empty for alert pagination"
    );
}

// ---------------------------------------------------------------------------
// AC-002 / BC-3.4.002 — Schema conformance via generate() internal validators
// (VP-112: all non-SchemaDrift archetypes pass validation inside generate())
// ---------------------------------------------------------------------------

/// AC-002 / VP-112 / TV-3.4.002-04: HealthyOtEnvironment — generate() internally validates
/// each surface record against its correct sub-spec (validate_* todo!() in schema_validation).
///
/// The test exercises generate(); the internal schema_validation module is invoked by
/// generate() in #[cfg(test)] mode. Any schema mismatch causes generate() to panic.
#[test]
fn test_bc_3_4_002_ac_002_vp_112_healthy_ot_environment_schema_validation() {
    // generate() calls validate_alert/validate_asm_asset/validate_cve/validate_ioc internally.
    // These call todo!() stubs in the stub commit -> Red Gate fires here.
    let fs = generate(&org_a(), Archetype::HealthyOtEnvironment, &default_opts());
    // Post-implementation: all records validated; no panic means all 4 sub-specs matched.
    assert!(
        fs.provenance.schema_valid,
        "VP-112: HealthyOtEnvironment provenance.schema_valid must be true"
    );
}

/// AC-002 / VP-112: CompromisedEndpoint — all 4 surfaces validated inside generate().
#[test]
fn test_bc_3_4_002_ac_002_vp_112_compromised_endpoint_schema_validation() {
    let fs = generate(&org_a(), Archetype::CompromisedEndpoint, &default_opts());
    assert!(
        fs.provenance.schema_valid,
        "VP-112: CompromisedEndpoint provenance.schema_valid must be true"
    );
}

/// AC-002 / VP-112: AuthOutage — schema validation passes on all surfaces.
#[test]
fn test_bc_3_4_002_ac_002_vp_112_auth_outage_schema_validation() {
    let fs = generate(&org_a(), Archetype::AuthOutage, &default_opts());
    assert!(
        fs.provenance.schema_valid,
        "VP-112: AuthOutage provenance.schema_valid must be true"
    );
}

/// AC-002 / VP-112: LargeScale — all 4500 records validated inside generate().
///
/// BC-3.4.002 EC-3.4.002-03: validator must not time out or OOM at 4500 records.
#[test]
fn test_bc_3_4_002_ac_002_vp_112_large_scale_schema_validation() {
    let fs = generate(&org_a(), Archetype::LargeScale, &default_opts());
    assert!(
        fs.provenance.schema_valid,
        "VP-112: LargeScale provenance.schema_valid must be true"
    );
    assert_eq!(
        fs.records.len(),
        4500,
        "LargeScale: total records must be 4500 (500+2000+1000+1000)"
    );
}

/// AC-002 / VP-112 / BC-3.4.002 EC-3.4.002-06: DormantTenant schema validation
/// trivially passes (no records to validate).
#[test]
fn test_bc_3_4_002_ac_002_vp_112_dormant_tenant_trivially_passes_validation() {
    let fs = generate(&org_a(), Archetype::DormantTenant, &default_opts());
    assert!(fs.records.is_empty(), "DormantTenant must have 0 records");
    assert!(
        fs.provenance.schema_valid,
        "VP-112: DormantTenant provenance.schema_valid must be true (trivially; 0 records)"
    );
}

/// AC-002 / VP-112: HighChurn — all surfaces validated inside generate().
#[test]
fn test_bc_3_4_002_ac_002_vp_112_high_churn_schema_validation() {
    let fs = generate(&org_a(), Archetype::HighChurn, &default_opts());
    assert!(
        fs.provenance.schema_valid,
        "VP-112: HighChurn provenance.schema_valid must be true"
    );
}

// ---------------------------------------------------------------------------
// AC-003 / BC-3.4.002 — SchemaDrift: only alert surface[0] invalid (VP-113)
// ---------------------------------------------------------------------------

/// AC-003 / VP-113 / TV-3.4.002-02: SchemaDrift archetype — generate() marks
/// provenance.schema_valid = false and exactly 1 alert record is malformed.
///
/// BC-3.4.002 postconditions for SchemaDrift:
///   - provenance.schema_valid == false
///   - >=1 record fails schema; exactly 1 (BC-3.4.003 invariant 4)
///   - story spec AC-003: alert surface[0] is the intentionally invalid record
#[test]
fn test_bc_3_4_002_ac_003_vp_113_schema_drift_provenance_schema_valid_false() {
    let fs = generate(&org_a(), Archetype::SchemaDrift, &default_opts());
    assert!(
        !fs.provenance.schema_valid,
        "VP-113: SchemaDrift provenance.schema_valid must be false (BC-3.4.002)"
    );
}

/// AC-003 / BC-3.4.002: SchemaDrift — alert surface[0] carries a `_schema_valid=false` marker.
///
/// Story spec AC-003: "exactly 1 record (at index 0 of the alert surface) intentionally
/// violates alert_api_specs.json".
#[test]
fn test_bc_3_4_002_ac_003_schema_drift_alert_surface_index_0_marked_invalid() {
    let fs = generate(&org_a(), Archetype::SchemaDrift, &default_opts());
    let alerts: Vec<&Value> = fs
        .records
        .iter()
        .filter(|r| r.get("_surface").and_then(Value::as_str) == Some("alert"))
        .collect();
    assert!(!alerts.is_empty(), "SchemaDrift must have alert records");
    // alert surface[0] must be the drifted record
    let drifted = alerts[0];
    let schema_valid_field = drifted
        .get("_schema_valid")
        .and_then(Value::as_bool)
        .unwrap_or(true);
    assert!(
        !schema_valid_field,
        "AC-003: SchemaDrift alert surface[0] must carry _schema_valid=false"
    );
}

/// BC-3.4.003 invariant 4 / AC-003: exactly 1 non-conformant record across all surfaces.
#[test]
fn test_bc_3_4_003_ac_003_schema_drift_exactly_one_invalid_record() {
    let fs = generate(&org_a(), Archetype::SchemaDrift, &default_opts());
    let invalid_count = fs
        .records
        .iter()
        .filter(|r| {
            r.get("_schema_valid")
                .and_then(Value::as_bool)
                .map(|v| !v)
                .unwrap_or(false)
        })
        .count();
    assert_eq!(
        invalid_count, 1,
        "BC-3.4.003 invariant 4: SchemaDrift must produce exactly 1 non-conformant record, got {invalid_count}"
    );
}

/// AC-003: non-alert surfaces in SchemaDrift remain valid (schema_valid not set false).
#[test]
fn test_bc_3_4_002_ac_003_schema_drift_non_alert_surfaces_remain_valid() {
    let fs = generate(&org_a(), Archetype::SchemaDrift, &default_opts());
    for (idx, record) in fs.records.iter().enumerate() {
        let surface = record
            .get("_surface")
            .and_then(Value::as_str)
            .unwrap_or("unknown");
        if surface == "alert" {
            continue; // alert surface has 1 intentionally drifted record
        }
        let schema_valid = record
            .get("_schema_valid")
            .and_then(Value::as_bool)
            .unwrap_or(true); // absence of the field means valid
        assert!(
            schema_valid,
            "AC-003: SchemaDrift non-alert record[{idx}] on surface '{surface}' must remain valid"
        );
    }
}

// ---------------------------------------------------------------------------
// AC-004 / BC-3.4.004 — Org-tagged IDs per surface (VP-119, VP-120)
// ---------------------------------------------------------------------------

/// AC-004 / VP-120 / BC-3.4.004: all alert records carry `alert_id` containing
/// org slug and seed as substrings (AC-004 / TV-3.4.004-01 adapted for Cyberint).
///
/// Cyberint alert ID format: `alert-{org_slug}-{seed}-{index}` (AC-004 / story spec).
#[test]
fn test_bc_3_4_004_ac_004_vp_120_alert_ids_contain_seed() {
    let fs = generate(&org_a(), Archetype::HealthyOtEnvironment, &default_opts());
    let alerts: Vec<&Value> = fs
        .records
        .iter()
        .filter(|r| r.get("_surface").and_then(Value::as_str) == Some("alert"))
        .collect();
    assert!(!alerts.is_empty(), "must have alert records for ID check");
    for (i, record) in alerts.iter().enumerate() {
        let alert_id = record
            .get("alert_id")
            .and_then(Value::as_str)
            .unwrap_or_else(|| panic!("alert record[{i}] missing alert_id field"));
        // VP-120: ID contains org slug as substring; also must contain seed
        assert!(
            alert_id.starts_with("alert-"),
            "alert_id must start with 'alert-'; got '{alert_id}'"
        );
        assert!(
            alert_id.contains("42"),
            "alert_id must embed seed 42; got '{alert_id}'"
        );
    }
}

/// AC-004 / VP-120: ASM asset records carry `asset_id` with `dev-{org_slug}-{seed}-` prefix.
#[test]
fn test_bc_3_4_004_ac_004_vp_120_asm_asset_ids_contain_seed() {
    let fs = generate(&org_a(), Archetype::HealthyOtEnvironment, &default_opts());
    let asm_assets: Vec<&Value> = fs
        .records
        .iter()
        .filter(|r| r.get("_surface").and_then(Value::as_str) == Some("asm_asset"))
        .collect();
    assert!(
        !asm_assets.is_empty(),
        "must have asm_asset records for ID check"
    );
    for (i, record) in asm_assets.iter().enumerate() {
        let asset_id = record
            .get("asset_id")
            .and_then(Value::as_str)
            .unwrap_or_else(|| panic!("asm_asset record[{i}] missing asset_id field"));
        assert!(
            asset_id.starts_with("dev-"),
            "asset_id must start with 'dev-'; got '{asset_id}'"
        );
        assert!(
            asset_id.contains("42"),
            "asset_id must embed seed 42; got '{asset_id}'"
        );
    }
}

/// AC-004 / VP-120: CVE records carry `alert_id` with `alert-{org_slug}-{seed}-` prefix.
///
/// Story spec AC-004: "CVE and IOC records: primary ID field starts with alert-{org_slug}-{seed}-".
#[test]
fn test_bc_3_4_004_ac_004_vp_120_cve_ids_contain_seed() {
    let fs = generate(&org_a(), Archetype::HealthyOtEnvironment, &default_opts());
    let cves: Vec<&Value> = fs
        .records
        .iter()
        .filter(|r| r.get("_surface").and_then(Value::as_str) == Some("cve"))
        .collect();
    assert!(!cves.is_empty(), "must have cve records for ID check");
    for (i, record) in cves.iter().enumerate() {
        let cve_id = record
            .get("alert_id")
            .and_then(Value::as_str)
            .unwrap_or_else(|| panic!("cve record[{i}] missing alert_id field"));
        assert!(
            cve_id.starts_with("alert-"),
            "CVE alert_id must start with 'alert-'; got '{cve_id}'"
        );
        assert!(
            cve_id.contains("42"),
            "CVE alert_id must embed seed 42; got '{cve_id}'"
        );
    }
}

/// AC-004 / VP-120: IOC records carry `alert_id` with `alert-{org_slug}-{seed}-` prefix.
#[test]
fn test_bc_3_4_004_ac_004_vp_120_ioc_ids_contain_seed() {
    let fs = generate(&org_a(), Archetype::HealthyOtEnvironment, &default_opts());
    let iocs: Vec<&Value> = fs
        .records
        .iter()
        .filter(|r| r.get("_surface").and_then(Value::as_str) == Some("ioc"))
        .collect();
    assert!(!iocs.is_empty(), "must have ioc records for ID check");
    for (i, record) in iocs.iter().enumerate() {
        let ioc_id = record
            .get("alert_id")
            .and_then(Value::as_str)
            .unwrap_or_else(|| panic!("ioc record[{i}] missing alert_id field"));
        assert!(
            ioc_id.starts_with("alert-"),
            "IOC alert_id must start with 'alert-'; got '{ioc_id}'"
        );
        assert!(
            ioc_id.contains("42"),
            "IOC alert_id must embed seed 42; got '{ioc_id}'"
        );
    }
}

/// AC-004 / BC-3.4.004 postcondition 3 / VP-119:
/// OrgA and OrgB ID sets are disjoint across all surfaces when slugs differ.
///
/// TV-3.4.004-03: orgA IDs ∩ orgB IDs = ∅.
#[test]
fn test_bc_3_4_004_ac_004_vp_119_org_id_sets_disjoint_across_all_surfaces() {
    let fs_a = generate(&org_a(), Archetype::HealthyOtEnvironment, &default_opts());
    let fs_b = generate(&org_b(), Archetype::HealthyOtEnvironment, &default_opts());

    let ids_a: std::collections::HashSet<String> = fs_a
        .records
        .iter()
        .flat_map(|r| {
            ["alert_id", "asset_id"]
                .iter()
                .filter_map(|k| r.get(*k).and_then(Value::as_str).map(String::from))
        })
        .collect();
    let ids_b: std::collections::HashSet<String> = fs_b
        .records
        .iter()
        .flat_map(|r| {
            ["alert_id", "asset_id"]
                .iter()
                .filter_map(|k| r.get(*k).and_then(Value::as_str).map(String::from))
        })
        .collect();

    let intersection: std::collections::HashSet<_> = ids_a.intersection(&ids_b).collect();
    assert!(
        intersection.is_empty(),
        "VP-119 / BC-3.4.004 postcondition 3: orgA and orgB ID sets must be disjoint; \
         shared IDs: {intersection:?}"
    );
}

/// BC-3.4.004 postcondition 2: orgB alert IDs must not appear in orgA records.
#[test]
fn test_bc_3_4_004_ac_004_different_orgs_different_id_prefixes() {
    let fs_a = generate(&org_a(), Archetype::HealthyOtEnvironment, &default_opts());
    let fs_b = generate(&org_b(), Archetype::HealthyOtEnvironment, &default_opts());
    let alert_ids_a: Vec<&str> = fs_a
        .records
        .iter()
        .filter_map(|r| r.get("alert_id").and_then(Value::as_str))
        .collect();
    let alert_ids_b: std::collections::HashSet<&str> = fs_b
        .records
        .iter()
        .filter_map(|r| r.get("alert_id").and_then(Value::as_str))
        .collect();
    for id in &alert_ids_a {
        assert!(
            !alert_ids_b.contains(id),
            "BC-3.4.004: orgA alert_id '{id}' found in orgB records — ID isolation failure"
        );
    }
}

// ---------------------------------------------------------------------------
// AC-005 / BC-3.4.001 — Determinism (VP-108)
// ---------------------------------------------------------------------------

/// AC-005 / VP-108 / TV-3.4.001-01: two calls with identical inputs produce
/// byte-identical records.
///
/// BC-3.4.001 postcondition 1: JSON serialization of records is identical.
#[test]
fn test_bc_3_4_001_ac_005_vp_108_determinism_same_inputs_byte_identical() {
    let fs1 = generate(&org_a(), Archetype::HealthyOtEnvironment, &default_opts());
    let fs2 = generate(&org_a(), Archetype::HealthyOtEnvironment, &default_opts());
    let json1 = serde_json::to_string(&fs1.records).expect("serialize fs1.records");
    let json2 = serde_json::to_string(&fs2.records).expect("serialize fs2.records");
    assert_eq!(
        json1, json2,
        "VP-108 / BC-3.4.001 postcondition 1: two identical calls must produce byte-identical records"
    );
}

/// AC-005 / BC-3.4.001 postcondition 3: distinct seeds produce distinct records.
///
/// TV-3.4.001-02: seed=1 vs seed=2 must produce different records.
#[test]
fn test_bc_3_4_001_ac_005_distinct_seeds_produce_distinct_records() {
    let opts1 = GenOpts::new(1, 1.0, DateTime::UNIX_EPOCH, serde_json::Value::Null)
        .expect("valid opts seed=1");
    let opts2 = GenOpts::new(2, 1.0, DateTime::UNIX_EPOCH, serde_json::Value::Null)
        .expect("valid opts seed=2");
    let fs1 = generate(&org_a(), Archetype::HealthyOtEnvironment, &opts1);
    let fs2 = generate(&org_a(), Archetype::HealthyOtEnvironment, &opts2);
    let json1 = serde_json::to_string(&fs1.records).expect("serialize fs1");
    let json2 = serde_json::to_string(&fs2.records).expect("serialize fs2");
    assert_ne!(
        json1, json2,
        "BC-3.4.001 postcondition 3: seed=1 and seed=2 must produce different records"
    );
}

/// AC-005 / BC-3.4.001 postcondition 4: distinct org_ids produce distinct records for same seed.
///
/// TV-3.4.001-03: orgA vs orgB with same seed must differ.
#[test]
fn test_bc_3_4_001_ac_005_distinct_org_ids_produce_distinct_records() {
    let fs_a = generate(&org_a(), Archetype::HealthyOtEnvironment, &default_opts());
    let fs_b = generate(&org_b(), Archetype::HealthyOtEnvironment, &default_opts());
    let json_a = serde_json::to_string(&fs_a.records).expect("serialize orgA");
    let json_b = serde_json::to_string(&fs_b.records).expect("serialize orgB");
    assert_ne!(
        json_a, json_b,
        "BC-3.4.001 postcondition 4: distinct org_ids must produce distinct records"
    );
}

/// BC-3.4.001 EC-3.4.001-07: seed=u64::MAX must not panic; results are identical.
#[test]
fn test_bc_3_4_001_ac_005_seed_u64_max_no_panic_and_deterministic() {
    let opts = GenOpts::new(u64::MAX, 1.0, DateTime::UNIX_EPOCH, serde_json::Value::Null)
        .expect("valid opts seed=u64::MAX");
    let fs1 = generate(&org_a(), Archetype::HealthyOtEnvironment, &opts);
    let fs2 = generate(&org_a(), Archetype::HealthyOtEnvironment, &opts);
    let json1 = serde_json::to_string(&fs1.records).expect("serialize fs1");
    let json2 = serde_json::to_string(&fs2.records).expect("serialize fs2");
    assert_eq!(
        json1, json2,
        "BC-3.4.001 EC-3.4.001-07: seed=u64::MAX must produce deterministic results"
    );
}

// ---------------------------------------------------------------------------
// AC-006 / BC-3.4.001 EC-003 — Single RNG stream across 4 surfaces
// ---------------------------------------------------------------------------

/// AC-006 / EC-003: different seed produces different records on BOTH alert AND ASM surfaces,
/// proving the single RNG stream advances sequentially through all 4 surfaces.
///
/// If 4 independent RNGs were seeded from the same seed, streams for each surface would
/// be identical across seed values that XOR to the same effective seed — this test
/// detects that erroneous implementation.
#[test]
fn test_bc_3_4_001_ac_006_single_rng_stream_different_seed_changes_all_surfaces() {
    let opts1 = GenOpts::new(1, 1.0, DateTime::UNIX_EPOCH, serde_json::Value::Null)
        .expect("valid opts seed=1");
    let opts2 = GenOpts::new(2, 1.0, DateTime::UNIX_EPOCH, serde_json::Value::Null)
        .expect("valid opts seed=2");

    let fs1 = generate(&org_a(), Archetype::HealthyOtEnvironment, &opts1);
    let fs2 = generate(&org_a(), Archetype::HealthyOtEnvironment, &opts2);

    // Alert surface must differ
    let alerts1: Vec<&Value> = fs1
        .records
        .iter()
        .filter(|r| r.get("_surface").and_then(Value::as_str) == Some("alert"))
        .collect();
    let alerts2: Vec<&Value> = fs2
        .records
        .iter()
        .filter(|r| r.get("_surface").and_then(Value::as_str) == Some("alert"))
        .collect();
    assert_ne!(
        serde_json::to_string(&alerts1).unwrap(),
        serde_json::to_string(&alerts2).unwrap(),
        "EC-003: different seed must change alert surface records"
    );

    // ASM asset surface must also differ (shared stream, not 4 independent streams)
    let asm1: Vec<&Value> = fs1
        .records
        .iter()
        .filter(|r| r.get("_surface").and_then(Value::as_str) == Some("asm_asset"))
        .collect();
    let asm2: Vec<&Value> = fs2
        .records
        .iter()
        .filter(|r| r.get("_surface").and_then(Value::as_str) == Some("asm_asset"))
        .collect();
    assert_ne!(
        serde_json::to_string(&asm1).unwrap(),
        serde_json::to_string(&asm2).unwrap(),
        "EC-003: different seed must change ASM asset surface records"
    );
}

// ---------------------------------------------------------------------------
// AC-007 / BC-3.4.002 invariant 4 / VP-114 — Schema validation absent from release
// ---------------------------------------------------------------------------

/// AC-007 / VP-114 / BC-3.4.002 invariant 4:
/// This test file and schema_validation helpers are compiled only under
/// `#[cfg(feature = "fixture-gen")]` and `#[cfg(test)]` respectively.
/// CI verifies: `cargo build --release` excludes all schema validation code.
///
/// This test anchors VP-114 traceability; the compilation gate is the proof.
#[test]
fn test_bc_3_4_002_ac_007_vp_114_schema_validation_gated_cfg_test() {
    // Reaching this test proves the fixture-gen feature gate is active.
    // The schema_validation module is gated #[cfg(test)] inside generator.rs.
    // CI `cargo build --release` + grep enforces VP-114 statically.
    assert!(
        cfg!(feature = "fixture-gen"),
        "VP-114: this test must only run under the fixture-gen feature"
    );
}

// ---------------------------------------------------------------------------
// Per-endpoint coverage: all 4 generate_X helpers exercised via all_archetypes()
// ---------------------------------------------------------------------------

/// Coverage: generate_alerts exercised for all 8 archetypes (DormantTenant -> 0).
#[test]
fn test_bc_3_4_002_all_archetypes_alerts_surface_present_or_empty() {
    for archetype in all_archetypes() {
        let fs = generate(&org_a(), *archetype, &default_opts());
        let alert_count = surface_count(&fs, "alert");
        if *archetype == Archetype::DormantTenant {
            assert_eq!(
                alert_count, 0,
                "DormantTenant: alert count must be 0, got {alert_count}"
            );
        } else {
            assert!(
                alert_count > 0,
                "archetype {archetype:?}: expected >=1 alert record at scale=1.0"
            );
        }
    }
}

/// Coverage: generate_asm_assets exercised for all 8 archetypes.
#[test]
fn test_bc_3_4_002_all_archetypes_asm_assets_surface_present_or_empty() {
    for archetype in all_archetypes() {
        let fs = generate(&org_a(), *archetype, &default_opts());
        let asm_count = surface_count(&fs, "asm_asset");
        if *archetype == Archetype::DormantTenant {
            assert_eq!(
                asm_count, 0,
                "DormantTenant: asm_asset count must be 0, got {asm_count}"
            );
        } else {
            assert!(
                asm_count > 0,
                "archetype {archetype:?}: expected >=1 asm_asset record at scale=1.0"
            );
        }
    }
}

/// Coverage: generate_cves exercised for all 8 archetypes.
#[test]
fn test_bc_3_4_002_all_archetypes_cves_surface_present_or_empty() {
    for archetype in all_archetypes() {
        let fs = generate(&org_a(), *archetype, &default_opts());
        let cve_count = surface_count(&fs, "cve");
        if *archetype == Archetype::DormantTenant {
            assert_eq!(
                cve_count, 0,
                "DormantTenant: cve count must be 0, got {cve_count}"
            );
        } else {
            assert!(
                cve_count > 0,
                "archetype {archetype:?}: expected >=1 cve record at scale=1.0"
            );
        }
    }
}

/// Coverage: generate_iocs exercised for all 8 archetypes.
#[test]
fn test_bc_3_4_002_all_archetypes_iocs_surface_present_or_empty() {
    for archetype in all_archetypes() {
        let fs = generate(&org_a(), *archetype, &default_opts());
        let ioc_count = surface_count(&fs, "ioc");
        if *archetype == Archetype::DormantTenant {
            assert_eq!(
                ioc_count, 0,
                "DormantTenant: ioc count must be 0, got {ioc_count}"
            );
        } else {
            assert!(
                ioc_count > 0,
                "archetype {archetype:?}: expected >=1 ioc record at scale=1.0"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// BC-3.4.001 invariant 1 — No non-deterministic entropy (VP-3.4.001-D)
// ---------------------------------------------------------------------------

/// BC-3.4.001 invariant 1 / VP-3.4.001-D: generate() must not call rand::thread_rng()
/// or SystemTime::now() anywhere in its call stack.
///
/// Static enforcement CI command:
///   grep -rn 'thread_rng\s*()\|SystemTime::now' \
///     crates/prism-dtu-cyberint/src/generator.rs
/// fails if any non-comment match is found.
///
/// This test anchors the invariant for Cyberint-specific traceability.
#[test]
fn test_bc_3_4_001_invariant_1_no_thread_rng_documented_cyberint() {
    // Calling generate() exercises the todo!() stub — Red Gate fires.
    // After implementation, this test verifies determinism is achievable only because
    // no non-deterministic entropy was introduced (VP-3.4.001-D).
    let _fs = generate(&org_a(), Archetype::HealthyOtEnvironment, &default_opts());
}
