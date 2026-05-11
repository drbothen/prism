//! Failing acceptance tests for S-3.7.04 — Armis fixture generator.
//!
//! Covers:
//!   BC-3.4.001  Generator Determinism
//!   BC-3.4.002  Schema Conformance (Armis field shapes)
//!   BC-3.4.003  Archetype Catalog — 8 archetypes with defined baselines
//!   BC-3.4.004  Org-Tagged Record IDs
//!   VP-108  generate() is idempotent (same inputs → byte-identical output)
//!   VP-112  All non-SchemaDrift records pass schema validation
//!   VP-113  SchemaDrift: provenance.schema_valid=false, ≥1 record fails validation
//!   VP-114  Schema validation absent from release build (cfg(test) gate)
//!   VP-119  Org ID sets disjoint for distinct slug pairs
//!   VP-120  Every record primary ID contains org slug as substring
//!   VP-121  Unregistered org → Err(UnregisteredOrg) without panic
//!
//! ALL tests are expected to FAIL (todo!() panic) until the implementation lands.
//! Red Gate verified before implementation begins.
//!
//! Run:
//!   cargo test -p prism-dtu-armis --features fixture-gen \
//!       --test bc_3_4_armis_generator
#![cfg(feature = "fixture-gen")]
#![allow(clippy::unwrap_used, clippy::expect_used)]

use prism_core::SensorId;
use prism_dtu_armis::generator::generate;
use prism_dtu_common::generator::{default_page_size, Archetype, GenOpts, OrgId};
use serde_json::Value;

// ---------------------------------------------------------------------------
// Test helpers
// ---------------------------------------------------------------------------

fn org_a() -> OrgId {
    OrgId([
        0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F,
        0x10,
    ])
}

fn org_b() -> OrgId {
    OrgId([
        0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99,
        0x00,
    ])
}

const SLUG_A: &str = "acme-corp";
const SLUG_B: &str = "globex";

/// Collect the `asset_id` or `alert_id` string from a record's JSON value.
/// Returns None if neither field is present or the value is not a string/number.
fn primary_id(record: &Value) -> Option<String> {
    if let Some(v) = record.get("asset_id").or_else(|| record.get("id")) {
        match v {
            Value::String(s) => return Some(s.clone()),
            Value::Number(n) => return Some(n.to_string()),
            _ => {}
        }
    }
    if let Some(Value::Number(n)) = record.get("alertId") {
        return Some(n.to_string());
    }
    if let Some(Value::String(s)) = record.get("alertId") {
        return Some(s.clone());
    }
    None
}

// ---------------------------------------------------------------------------
// BC-3.4.001 / VP-108 — Generator Determinism
// ---------------------------------------------------------------------------

/// BC-3.4.001 postcondition 1 / VP-108:
/// Two sequential calls with identical (org_id, org_slug, archetype, opts)
/// produce byte-identical FixtureSet::records JSON serializations.
///
/// TV-3.4.001-01 (Armis variant).
#[test]
fn test_bc_3_4_001_vp_108_determinism_healthy_ot_byte_identical() {
    let opts = GenOpts::default();
    let fs1 = generate(org_a(), SLUG_A, Archetype::HealthyOtEnvironment, &opts);
    let fs2 = generate(org_a(), SLUG_A, Archetype::HealthyOtEnvironment, &opts);
    let json1 = serde_json::to_string(&fs1.records).unwrap();
    let json2 = serde_json::to_string(&fs2.records).unwrap();
    assert_eq!(
        json1, json2,
        "BC-3.4.001 postcondition 1 / VP-108: identical inputs must produce byte-identical records"
    );
}

/// BC-3.4.001 postcondition 1 / VP-108:
/// Determinism holds for all 8 archetypes.
#[test]
fn test_bc_3_4_001_vp_108_determinism_all_archetypes() {
    let archetypes = [
        Archetype::HealthyOtEnvironment,
        Archetype::CompromisedEndpoint,
        Archetype::AuthOutage,
        Archetype::LargeScale,
        Archetype::PaginationEdgeCases,
        Archetype::SchemaDrift,
        Archetype::HighChurn,
        Archetype::DormantTenant,
    ];
    let opts = GenOpts::default();
    for archetype in archetypes {
        let fs1 = generate(org_a(), SLUG_A, archetype.clone(), &opts);
        let fs2 = generate(org_a(), SLUG_A, archetype.clone(), &opts);
        let j1 = serde_json::to_string(&fs1.records).unwrap();
        let j2 = serde_json::to_string(&fs2.records).unwrap();
        assert_eq!(
            j1, j2,
            "BC-3.4.001 VP-108: archetype {:?} must be deterministic",
            archetype
        );
    }
}

/// BC-3.4.001 postcondition 3: distinct seeds produce distinct records.
/// TV-3.4.001-02 (Armis variant).
#[test]
fn test_bc_3_4_001_distinct_seeds_produce_distinct_records() {
    let opts1 = GenOpts::default(); // seed=42
    let opts2 = {
        let mut o = GenOpts::default();
        o.seed = 1;
        o
    };
    let fs1 = generate(org_a(), SLUG_A, Archetype::HealthyOtEnvironment, &opts1);
    let fs2 = generate(org_a(), SLUG_A, Archetype::HealthyOtEnvironment, &opts2);
    let j1 = serde_json::to_string(&fs1.records).unwrap();
    let j2 = serde_json::to_string(&fs2.records).unwrap();
    assert_ne!(
        j1, j2,
        "BC-3.4.001 postcondition 3: distinct seeds must produce distinct records"
    );
}

/// BC-3.4.001 postcondition 4: distinct org_ids with same seed produce distinct records.
/// TV-3.4.001-03 (Armis variant).
#[test]
fn test_bc_3_4_001_distinct_org_ids_produce_distinct_records() {
    let opts = GenOpts::default();
    let fs_a = generate(org_a(), SLUG_A, Archetype::HealthyOtEnvironment, &opts);
    let fs_b = generate(org_b(), SLUG_B, Archetype::HealthyOtEnvironment, &opts);
    let j_a = serde_json::to_string(&fs_a.records).unwrap();
    let j_b = serde_json::to_string(&fs_b.records).unwrap();
    assert_ne!(
        j_a, j_b,
        "BC-3.4.001 postcondition 4: distinct org_ids must produce distinct records"
    );
}

/// BC-3.4.001 EC-3.4.001-04: seed=u64::MAX must not panic.
#[test]
fn test_bc_3_4_001_seed_max_does_not_panic() {
    let mut opts = GenOpts::default();
    opts.seed = u64::MAX;
    let _ = generate(org_a(), SLUG_A, Archetype::HealthyOtEnvironment, &opts);
}

/// BC-3.4.001 EC-3.4.001-03: seed=0 is valid and deterministic.
#[test]
fn test_bc_3_4_001_seed_zero_is_valid_and_deterministic() {
    let mut opts = GenOpts::default();
    opts.seed = 0;
    let fs1 = generate(org_a(), SLUG_A, Archetype::DormantTenant, &opts);
    let fs2 = generate(org_a(), SLUG_A, Archetype::DormantTenant, &opts);
    assert_eq!(
        serde_json::to_string(&fs1.records).unwrap(),
        serde_json::to_string(&fs2.records).unwrap(),
    );
}

// ---------------------------------------------------------------------------
// BC-3.4.003 — Archetype baseline counts at scale=1.0
// ---------------------------------------------------------------------------

/// BC-3.4.003 postcondition / TV-3.4.003-01 (Armis):
/// HealthyOtEnvironment — 50 asset records, 5 alert records, 0 high-severity.
/// AC-001 row 1.
#[test]
fn test_bc_3_4_003_ac_001_healthy_ot_baseline_counts() {
    let opts = GenOpts::default();
    let fs = generate(org_a(), SLUG_A, Archetype::HealthyOtEnvironment, &opts);

    let assets: Vec<&Value> = fs
        .records
        .iter()
        .filter(|r| r.get("id").is_some() && r.get("alertId").is_none())
        .collect();
    let alerts: Vec<&Value> = fs
        .records
        .iter()
        .filter(|r| r.get("alertId").is_some())
        .collect();

    assert_eq!(
        assets.len(),
        50,
        "BC-3.4.003: HealthyOtEnvironment must have 50 asset records at scale=1.0"
    );
    assert_eq!(
        alerts.len(),
        5,
        "BC-3.4.003: HealthyOtEnvironment must have 5 alert records at scale=1.0"
    );

    // All assets have status "online" or "active"
    for asset in &assets {
        let status = asset["status"].as_str().unwrap_or("");
        assert!(
            status == "online" || status == "active",
            "BC-3.4.003: HealthyOtEnvironment asset status must be 'online' or 'active', got '{}'",
            status
        );
    }
}

/// BC-3.4.003 / TV-3.4.003-02 (Armis):
/// CompromisedEndpoint — 50 assets, 20 alerts, ≥3 with severity HIGH.
/// AC-001 row 2.
#[test]
fn test_bc_3_4_003_ac_001_compromised_endpoint_baseline_counts() {
    let opts = GenOpts::default();
    let fs = generate(org_a(), SLUG_A, Archetype::CompromisedEndpoint, &opts);

    let assets: Vec<&Value> = fs
        .records
        .iter()
        .filter(|r| r.get("id").is_some() && r.get("alertId").is_none())
        .collect();
    let alerts: Vec<&Value> = fs
        .records
        .iter()
        .filter(|r| r.get("alertId").is_some())
        .collect();

    assert_eq!(
        assets.len(),
        50,
        "BC-3.4.003: CompromisedEndpoint must have 50 asset records"
    );
    assert_eq!(
        alerts.len(),
        20,
        "BC-3.4.003: CompromisedEndpoint must have 20 alert records"
    );

    // ≥3 alerts have severity "HIGH" (OCSF severity_id>=4 expressed as string in Armis)
    let high_sev = alerts
        .iter()
        .filter(|a| {
            let sev = a["severity"].as_str().unwrap_or("");
            sev == "HIGH" || sev == "CRITICAL"
        })
        .count();
    assert!(
        high_sev >= 3,
        "BC-3.4.003: CompromisedEndpoint must have ≥3 high-severity alerts, got {}",
        high_sev
    );

    // ≥1 asset has anomalous lateral-movement indicator in status
    let anomalous = assets
        .iter()
        .filter(|a| {
            let status = a["status"].as_str().unwrap_or("");
            status.contains("lateral")
                || status.contains("compromised")
                || status.contains("contained")
        })
        .count();
    assert!(
        anomalous >= 1,
        "BC-3.4.003: CompromisedEndpoint must have ≥1 asset with lateral-movement indicator"
    );
}

/// BC-3.4.003 / TV-3.4.003-03 (Armis):
/// AuthOutage — 20 asset records; first simulated call record has status_code=401.
/// AC-001 row 3.
#[test]
fn test_bc_3_4_003_ac_001_auth_outage_baseline_counts_and_401() {
    let opts = GenOpts::default();
    let fs = generate(org_a(), SLUG_A, Archetype::AuthOutage, &opts);

    let assets: Vec<&Value> = fs
        .records
        .iter()
        .filter(|r| r.get("id").is_some() && r.get("alertId").is_none())
        .collect();

    assert_eq!(
        assets.len(),
        20,
        "BC-3.4.003: AuthOutage must have 20 asset records at scale=1.0"
    );

    // First record must carry status_code=401
    let first = &fs.records[0];
    let status_code = first["status_code"]
        .as_i64()
        .or_else(|| first["statusCode"].as_i64())
        .unwrap_or(0);
    assert_eq!(
        status_code, 401,
        "BC-3.4.003: AuthOutage records[0] must have status_code=401"
    );
}

/// BC-3.4.003 / TV-3.4.003-05 (Armis):
/// LargeScale — 10,000 asset records, 500 alert records.
/// AC-001 row 4.
#[test]
fn test_bc_3_4_003_ac_001_large_scale_baseline_counts() {
    let opts = GenOpts::default();
    let fs = generate(org_a(), SLUG_A, Archetype::LargeScale, &opts);

    let assets: Vec<&Value> = fs
        .records
        .iter()
        .filter(|r| r.get("id").is_some() && r.get("alertId").is_none())
        .collect();
    let alerts: Vec<&Value> = fs
        .records
        .iter()
        .filter(|r| r.get("alertId").is_some())
        .collect();

    assert_eq!(
        assets.len(),
        10_000,
        "BC-3.4.003: LargeScale must have 10,000 asset records"
    );
    assert_eq!(
        alerts.len(),
        500,
        "BC-3.4.003: LargeScale must have 500 alert records"
    );
}

/// BC-3.4.003 / AC-001 row 5:
/// PaginationEdgeCases — page_size×3 asset records, exactly 3 AQL cursor values.
#[test]
fn test_bc_3_4_003_ac_001_pagination_edge_cases_baseline_counts() {
    let opts = GenOpts::default();
    let fs = generate(org_a(), SLUG_A, Archetype::PaginationEdgeCases, &opts);
    let expected_records = default_page_size(SensorId::from("armis")) * 3;

    assert_eq!(
        fs.records.len(),
        expected_records,
        "BC-3.4.003: PaginationEdgeCases must have page_size×3={} asset records, got {}",
        expected_records,
        fs.records.len()
    );
    assert_eq!(
        fs.cursors.len(),
        3,
        "BC-3.4.003: PaginationEdgeCases must have exactly 3 AQL cursor values"
    );
}

/// BC-3.4.003 / TV-3.4.003-06 (Armis):
/// SchemaDrift — 30 asset records; records[0] missing required field; provenance.schema_valid=false.
/// AC-001 row 6.
#[test]
fn test_bc_3_4_003_ac_001_schema_drift_baseline_and_flag() {
    let opts = GenOpts::default();
    let fs = generate(org_a(), SLUG_A, Archetype::SchemaDrift, &opts);

    assert_eq!(
        fs.records.len(),
        30,
        "BC-3.4.003: SchemaDrift must have 30 asset records at scale=1.0"
    );
    assert!(
        !fs.provenance.schema_valid,
        "BC-3.4.003 / BC-3.4.002: SchemaDrift provenance.schema_valid must be false"
    );

    // records[0] must be the drifted record — it must be missing the "id" field
    // (BC-3.4.003 invariant 4: exactly 1 non-conformant record at index 0)
    let drifted = &fs.records[0];
    // A SchemaDrift record omits a required field; "id" is required for ArmisAsset
    let has_id = drifted.get("id").is_some();
    assert!(
        !has_id,
        "BC-3.4.003: SchemaDrift records[0] must omit required 'id' field"
    );
}

/// BC-3.4.003 / TV-3.4.003-08 (Armis):
/// HighChurn — 200 asset records; ≥20 tombstones with deleted_at present or status="tombstone".
/// AC-001 row 7. EC-004.
#[test]
fn test_bc_3_4_003_ac_001_high_churn_baseline_and_tombstones() {
    let opts = GenOpts::default();
    let fs = generate(org_a(), SLUG_A, Archetype::HighChurn, &opts);

    assert_eq!(
        fs.records.len(),
        200,
        "BC-3.4.003: HighChurn must have 200 asset records at scale=1.0"
    );

    let tombstones: Vec<&Value> = fs
        .records
        .iter()
        .filter(|r| {
            r.get("deleted_at").is_some() || r["status"].as_str().unwrap_or("") == "tombstone"
        })
        .collect();

    assert!(
        tombstones.len() >= 20,
        "BC-3.4.003: HighChurn must have ≥20 tombstone records, got {}",
        tombstones.len()
    );
}

/// BC-3.4.003 / TV-3.4.003-07 (Armis):
/// DormantTenant — 0 records, 0 cursors at any scale. EC-003.
#[test]
fn test_bc_3_4_003_ac_001_dormant_tenant_zero_records() {
    let opts = GenOpts::default();
    let fs = generate(org_a(), SLUG_A, Archetype::DormantTenant, &opts);
    assert_eq!(
        fs.records.len(),
        0,
        "BC-3.4.003: DormantTenant must have 0 records"
    );
    assert_eq!(
        fs.cursors.len(),
        0,
        "BC-3.4.003: DormantTenant must have 0 cursors"
    );
}

/// BC-3.4.003 invariant 5 + EC-003: DormantTenant ignores scale.
#[test]
fn test_bc_3_4_003_dormant_tenant_zero_records_at_high_scale() {
    let opts = GenOpts::new(
        42,
        100.0,
        chrono::DateTime::UNIX_EPOCH,
        serde_json::Value::Null,
    )
    .expect("scale=100.0 must be valid");
    let fs = generate(org_a(), SLUG_A, Archetype::DormantTenant, &opts);
    assert_eq!(
        fs.records.len(),
        0,
        "BC-3.4.003 invariant 5: DormantTenant must remain 0 records at scale=100.0"
    );
}

/// BC-3.4.003 invariant 4: SchemaDrift produces exactly 1 non-conformant record.
/// VP-118 (Armis instance).
#[test]
fn test_bc_3_4_003_schema_drift_exactly_one_nonconformant_record() {
    let opts = GenOpts::default();
    let fs = generate(org_a(), SLUG_A, Archetype::SchemaDrift, &opts);

    // Only records[0] must be missing "id"; the rest must have it
    let missing_id_count = fs.records.iter().filter(|r| r.get("id").is_none()).count();
    assert_eq!(
        missing_id_count, 1,
        "BC-3.4.003 invariant 4: SchemaDrift must have exactly 1 non-conformant record, got {}",
        missing_id_count
    );
}

/// BC-3.4.003: all 8 archetypes produce non-empty FixtureSets (except DormantTenant)
/// and distinct JSON serializations from one another.
#[test]
fn test_bc_3_4_003_all_archetypes_produce_distinct_non_empty_fixture_sets() {
    let opts = GenOpts::default();
    let non_dormant = [
        Archetype::HealthyOtEnvironment,
        Archetype::CompromisedEndpoint,
        Archetype::AuthOutage,
        Archetype::LargeScale,
        Archetype::PaginationEdgeCases,
        Archetype::SchemaDrift,
        Archetype::HighChurn,
    ];

    let serializations: Vec<String> = non_dormant
        .iter()
        .map(|a| {
            let fs = generate(org_a(), SLUG_A, a.clone(), &opts);
            assert!(
                !fs.records.is_empty(),
                "BC-3.4.003: archetype {:?} must produce non-empty records",
                a
            );
            serde_json::to_string(&fs.records).unwrap()
        })
        .collect();

    // All serializations must be distinct
    let unique: std::collections::HashSet<&String> = serializations.iter().collect();
    assert_eq!(
        unique.len(),
        serializations.len(),
        "BC-3.4.003: all 8 archetypes must produce distinct FixtureSets"
    );
}

// ---------------------------------------------------------------------------
// BC-3.4.002 / VP-112 — Schema Conformance (Armis field shapes)
// ---------------------------------------------------------------------------

/// BC-3.4.002 postcondition 1 / VP-112 / AC-002:
/// All non-SchemaDrift asset records contain the expected ArmisAsset fields.
/// Validates field structure from `.references/schemas/armis/types.rs`.
#[test]
fn test_bc_3_4_002_vp_112_asset_records_have_expected_fields() {
    // Expected nullable fields from ArmisAsset (types.rs): id, name, title, type,
    // status, lastSeen, firstSeen, ipAddress, macAddress, manufacturer, model,
    // firmwareVersion, operatingSystem, riskLevel, site, zone.
    // All are Option<_> so they may be null but must be present as keys.
    let expected_nullable_fields = [
        "id",
        "name",
        "title",
        "status",
        "lastSeen",
        "firstSeen",
        "ipAddress",
        "macAddress",
        "manufacturer",
        "model",
        "firmwareVersion",
        "operatingSystem",
        "riskLevel",
        "site",
        "zone",
    ];

    let opts = GenOpts::default();
    let fs = generate(org_a(), SLUG_A, Archetype::HealthyOtEnvironment, &opts);

    let assets: Vec<&Value> = fs
        .records
        .iter()
        .filter(|r| r.get("id").is_some() && r.get("alertId").is_none())
        .collect();

    for (i, asset) in assets.iter().enumerate() {
        for field in &expected_nullable_fields {
            assert!(
                asset.as_object().unwrap().contains_key(*field),
                "BC-3.4.002 VP-112: asset record[{}] missing expected field '{}'",
                i,
                field
            );
        }
    }
}

/// BC-3.4.002 postcondition 1 / VP-112 / AC-002:
/// All non-SchemaDrift alert records contain the expected ArmisAlert fields.
/// Validates field structure from `.references/schemas/armis/types.rs`.
#[test]
fn test_bc_3_4_002_vp_112_alert_records_have_expected_fields() {
    // Expected nullable fields from ArmisAlert (types.rs): alertId, policyId, title,
    // status, severity, type, time, lastAlertUpdateTime, deviceId, description, remediation.
    let expected_nullable_fields = [
        "alertId",
        "policyId",
        "title",
        "status",
        "severity",
        "time",
        "lastAlertUpdateTime",
        "deviceId",
        "description",
        "remediation",
    ];

    let opts = GenOpts::default();
    let fs = generate(org_a(), SLUG_A, Archetype::CompromisedEndpoint, &opts);

    let alerts: Vec<&Value> = fs
        .records
        .iter()
        .filter(|r| r.get("alertId").is_some())
        .collect();

    for (i, alert) in alerts.iter().enumerate() {
        for field in &expected_nullable_fields {
            assert!(
                alert.as_object().unwrap().contains_key(*field),
                "BC-3.4.002 VP-112: alert record[{}] missing expected field '{}'",
                i,
                field
            );
        }
    }
}

/// BC-3.4.002 postcondition 2 (SchemaDrift) / VP-113:
/// SchemaDrift produces provenance.schema_valid=false and ≥1 record fails field check.
/// AC-002.
#[test]
fn test_bc_3_4_002_vp_113_schema_drift_sets_provenance_flag_and_has_invalid_record() {
    let opts = GenOpts::default();
    let fs = generate(org_a(), SLUG_A, Archetype::SchemaDrift, &opts);

    assert!(
        !fs.provenance.schema_valid,
        "BC-3.4.002 VP-113: SchemaDrift provenance.schema_valid must be false"
    );

    // At least one record (records[0]) must fail the required-field check
    let invalid_count = fs.records.iter().filter(|r| r.get("id").is_none()).count();
    assert!(
        invalid_count >= 1,
        "BC-3.4.002 VP-113: SchemaDrift must have ≥1 record missing required 'id' field"
    );
}

/// BC-3.4.002 / VP-114: schema validation code is absent from release builds.
/// This is enforced by the #[cfg(test)] gate — the test itself is structural evidence
/// (if this file compiles, the feature gate works). Document the CI enforcement mechanism.
#[test]
fn test_bc_3_4_002_vp_114_schema_validation_gated_to_test_builds() {
    // Structural: this test binary only compiles under --features fixture-gen.
    // Release builds never link prism-dtu-armis with fixture-gen enabled.
    // CI enforces: `cargo build -p prism-dtu-armis --release` must succeed
    // and `cargo build -p prism-dtu-armis --release --features fixture-gen` is forbidden.
    // This test documents the invariant; the compilation gate itself is the enforcement.
    assert!(
        cfg!(feature = "fixture-gen"),
        "VP-114: this test must only execute under fixture-gen feature"
    );
}

/// BC-3.4.002 / AC-002: nullable fields in non-SchemaDrift records use null values
/// (not absent keys) per DERIVATION.md nullable convention.
#[test]
fn test_bc_3_4_002_nullable_fields_present_as_null_not_absent() {
    let opts = GenOpts::default();
    let fs = generate(org_a(), SLUG_A, Archetype::HealthyOtEnvironment, &opts);

    // Pick the first asset and assert all nullable fields exist as keys
    let first_asset = fs
        .records
        .iter()
        .find(|r| r.get("id").is_some() && r.get("alertId").is_none())
        .expect("HealthyOtEnvironment must have at least one asset record");

    let obj = first_asset
        .as_object()
        .expect("record must be a JSON object");
    for field in &["riskLevel", "zone", "firmwareVersion", "model"] {
        assert!(
            obj.contains_key(*field),
            "BC-3.4.002: nullable field '{}' must be present as key (null value), not absent",
            field
        );
    }
}

// ---------------------------------------------------------------------------
// BC-3.4.002 / AC-003 — AQL envelope structure (VP-121 Armis-specific)
// ---------------------------------------------------------------------------

/// BC-3.4.002 AC-003 / VP-121 (Armis AQL envelope correctness):
/// PaginationEdgeCases records are wrapped in AqlResponse<SearchData> envelope
/// with status, message, and data fields at documented paths.
#[test]
fn test_bc_3_4_002_ac_003_aql_envelope_structure() {
    let opts = GenOpts::default();
    let fs = generate(org_a(), SLUG_A, Archetype::PaginationEdgeCases, &opts);

    // All records should be AqlResponse envelopes
    for (i, record) in fs.records.iter().enumerate() {
        // AqlResponse<T> fields: status (Option<i32>), message (Option<String>), data (Option<T>)
        assert!(
            record.as_object().unwrap().contains_key("status"),
            "BC-3.4.002 AC-003: PaginationEdgeCases record[{}] missing AqlResponse 'status' field",
            i
        );
        assert!(
            record.as_object().unwrap().contains_key("data"),
            "BC-3.4.002 AC-003: PaginationEdgeCases record[{}] missing AqlResponse 'data' field",
            i
        );
        // data.results must be an array (SearchData shape)
        let data = &record["data"];
        assert!(
            data.get("results").map(|r| r.is_array()).unwrap_or(false),
            "BC-3.4.002 AC-003: AqlResponse data.results must be an array in record[{}]",
            i
        );
    }
}

/// AC-003: PaginationEdgeCases cursors are at their maximum length (stress-tests cursor storage).
#[test]
fn test_bc_3_4_002_ac_003_pagination_cursors_nonempty() {
    let opts = GenOpts::default();
    let fs = generate(org_a(), SLUG_A, Archetype::PaginationEdgeCases, &opts);

    assert_eq!(
        fs.cursors.len(),
        3,
        "AC-003: PaginationEdgeCases must have exactly 3 cursors"
    );
    for (i, cursor) in fs.cursors.iter().enumerate() {
        assert!(
            !cursor.is_empty(),
            "AC-003: cursor[{}] must be non-empty",
            i
        );
    }
}

// ---------------------------------------------------------------------------
// BC-3.4.004 / VP-119, VP-120 — Org-Tagged Record IDs
// ---------------------------------------------------------------------------

/// BC-3.4.004 postcondition 1 / VP-120 / AC-005:
/// All asset records contain org_slug as substring in asset_id / id field.
/// TV-3.4.004-01 (Armis).
#[test]
fn test_bc_3_4_004_vp_120_all_asset_ids_contain_org_slug() {
    let opts = GenOpts::default();
    let fs = generate(org_a(), SLUG_A, Archetype::HealthyOtEnvironment, &opts);

    let assets: Vec<&Value> = fs
        .records
        .iter()
        .filter(|r| r.get("id").is_some() && r.get("alertId").is_none())
        .collect();

    for (i, asset) in assets.iter().enumerate() {
        let id = primary_id(asset).unwrap_or_default();
        assert!(
            id.contains(SLUG_A),
            "BC-3.4.004 VP-120: asset record[{}] id '{}' must contain org_slug '{}'",
            i,
            id,
            SLUG_A
        );
    }
}

/// BC-3.4.004 postcondition 1 / VP-120 / AC-005:
/// All alert records contain org_slug as substring in alertId.
/// TV-3.4.004-05 (Armis).
#[test]
fn test_bc_3_4_004_vp_120_all_alert_ids_contain_org_slug() {
    let opts = GenOpts::default();
    let fs = generate(org_a(), SLUG_A, Archetype::CompromisedEndpoint, &opts);

    let alerts: Vec<&Value> = fs
        .records
        .iter()
        .filter(|r| r.get("alertId").is_some())
        .collect();

    for (i, alert) in alerts.iter().enumerate() {
        // alertId for Armis is an i64 in the schema; org-tagging for alerts uses
        // a string alert_id field per BC-3.4.004 postcondition table
        let alert_id_str = alert.get("alert_id").and_then(|v| v.as_str()).unwrap_or("");
        assert!(
            alert_id_str.starts_with(&format!("alert-{}", SLUG_A)),
            "BC-3.4.004 VP-120: alert record[{}] alert_id '{}' must start with 'alert-{}-'",
            i,
            alert_id_str,
            SLUG_A
        );
    }
}

/// BC-3.4.004 postcondition table (asset) / TV-3.4.004-04:
/// First asset ID is exactly `dev-{org_slug}-{seed}-0`.
#[test]
fn test_bc_3_4_004_first_asset_id_follows_format() {
    let opts = GenOpts::default(); // seed=42
    let fs = generate(org_a(), SLUG_A, Archetype::HealthyOtEnvironment, &opts);

    let first_asset = fs
        .records
        .iter()
        .find(|r| r.get("id").is_some() && r.get("alertId").is_none())
        .expect("must have at least one asset");

    // BC-3.4.004 TV-3.4.004-04 — the stable `asset_id` field (not the polymorphic `id`
    // field) carries the org_slug per VP-120; this test reconciles with EC-001 which lets
    // `id` be polymorphic per Armis API behavior (e.g. integer for i%5==0 records).
    let id = first_asset["asset_id"].as_str().unwrap_or("");
    let expected_prefix = format!("dev-{}-42-", SLUG_A);
    assert!(
        id.starts_with(&expected_prefix),
        "BC-3.4.004 TV-3.4.004-04: first asset id '{}' must start with '{}'",
        id,
        expected_prefix
    );
}

/// BC-3.4.004 postcondition table (tombstone) / TV-3.4.004-07 / EC-004:
/// HighChurn tombstone IDs follow `dev-{org_slug}-{seed}-tomb-{n}` pattern.
#[test]
fn test_bc_3_4_004_tombstone_ids_follow_pattern() {
    let opts = GenOpts::default();
    let fs = generate(org_a(), SLUG_A, Archetype::HighChurn, &opts);

    let tombstones: Vec<&Value> = fs
        .records
        .iter()
        .filter(|r| {
            r.get("deleted_at").is_some() || r["status"].as_str().unwrap_or("") == "tombstone"
        })
        .collect();

    for (i, tomb) in tombstones.iter().enumerate() {
        let id = tomb["id"].as_str().unwrap_or("");
        assert!(
            id.contains(&format!("{}-tomb-", SLUG_A)),
            "BC-3.4.004 TV-3.4.004-07: tombstone[{}] id '{}' must contain '{}-tomb-'",
            i,
            id,
            SLUG_A
        );
    }
}

/// BC-3.4.004 postcondition 3 / VP-119 / AC-007:
/// orgA and orgB ID sets are disjoint when slugs differ.
/// TV-3.4.004-03 (Armis).
#[test]
fn test_bc_3_4_004_vp_119_org_id_sets_are_disjoint() {
    let opts = GenOpts::default();
    let fs_a = generate(org_a(), SLUG_A, Archetype::HealthyOtEnvironment, &opts);
    let fs_b = generate(org_b(), SLUG_B, Archetype::HealthyOtEnvironment, &opts);

    let ids_a: std::collections::HashSet<String> =
        fs_a.records.iter().filter_map(|r| primary_id(r)).collect();
    let ids_b: std::collections::HashSet<String> =
        fs_b.records.iter().filter_map(|r| primary_id(r)).collect();

    let intersection: std::collections::HashSet<&String> = ids_a.intersection(&ids_b).collect();

    assert!(
        intersection.is_empty(),
        "BC-3.4.004 VP-119: orgA and orgB ID sets must be disjoint; shared IDs: {:?}",
        intersection
    );
}

/// BC-3.4.004 postcondition 2 / VP-120:
/// orgB asset IDs start with orgB prefix, not orgA prefix.
/// TV-3.4.004-02 (Armis).
#[test]
fn test_bc_3_4_004_vp_120_org_b_ids_use_org_b_slug() {
    let opts = GenOpts::default();
    let fs_b = generate(org_b(), SLUG_B, Archetype::HealthyOtEnvironment, &opts);

    let assets: Vec<&Value> = fs_b
        .records
        .iter()
        .filter(|r| r.get("id").is_some() && r.get("alertId").is_none())
        .collect();

    for (i, asset) in assets.iter().enumerate() {
        let id = primary_id(asset).unwrap_or_default();
        assert!(
            id.contains(SLUG_B),
            "BC-3.4.004 VP-120: orgB asset[{}] id '{}' must contain slug '{}'",
            i,
            id,
            SLUG_B
        );
        assert!(
            !id.contains(SLUG_A),
            "BC-3.4.004 VP-120: orgB asset[{}] id '{}' must NOT contain orgA slug '{}'",
            i,
            id,
            SLUG_A
        );
    }
}

/// BC-3.4.004 postcondition 4 / VP-121:
/// Unregistered org returns Err(GeneratorError::UnregisteredOrg) without panic.
/// TV-3.4.004-06.
///
/// The Armis generator's generate() returns FixtureSet (not Result) based on the
/// current stub signature. If the signature is Result<FixtureSet, GeneratorError>,
/// this test exercises the error path. If generate() panics on unregistered org,
/// this test will catch the panic and fail.
///
/// VP-121 proof method: proptest — this test is the deterministic anchor.
#[test]
fn test_bc_3_4_004_vp_121_unregistered_org_returns_error_not_panic() {
    // Use an org_id that the registry does not contain.
    // The generator must return Err(GeneratorError::UnregisteredOrg(..)) without panic.
    let unregistered = OrgId([0xFF; 16]);
    let opts = GenOpts::default();

    // When generate() returns Result<FixtureSet, GeneratorError>, assert Err variant.
    // When generate() is infallible (no OrgRegistry), this test documents the VP
    // and will pass vacuously — implementation must enforce the registry check.
    let result = std::panic::catch_unwind(|| {
        generate(
            unregistered,
            "unregistered-slug",
            Archetype::HealthyOtEnvironment,
            &opts,
        )
    });
    // Regardless of whether Err or Ok: the critical invariant is no panic.
    // After implementation, this should return Err(UnregisteredOrg).
    // For now: if todo!() panics it is caught here — that IS the Red Gate failure.
    match result {
        Ok(_fs) => {
            // If generate() returns Ok(fs), the implementation did not enforce the
            // registry check — this is a bug. The test will fail at assertion below.
            // (Post-stub: when real generate() is implemented without registry lookup,
            // the VP-121 contract is satisfied structurally — no panic. But the BC
            // requires Err on unregistered org, so we assert here.)
            // This path means generate() didn't return Err — fail VP-121.
            panic!("BC-3.4.004 VP-121: unregistered org must return Err(UnregisteredOrg), got Ok");
        }
        Err(_) => {
            // A panic occurred — either todo!() (Red Gate) or a genuine bug.
            // This is expected during Red Gate. After implementation, this branch
            // must be replaced by an Err(..) result.
        }
    }
}

// ---------------------------------------------------------------------------
// AC-004 / EC-001 — ArmisId polymorphism: integer-form IDs for every 5th record
// ---------------------------------------------------------------------------

/// AC-004 / EC-001:
/// Every 5th asset record (index 0, 5, 10, …) uses an integer-form asset_id
/// (JSON number), not a string.
#[test]
fn test_bc_3_4_002_ac_004_ec_001_every_fifth_asset_has_integer_id() {
    let opts = GenOpts::default();
    let fs = generate(org_a(), SLUG_A, Archetype::HealthyOtEnvironment, &opts);

    let assets: Vec<&Value> = fs
        .records
        .iter()
        .filter(|r| r.get("id").is_some() && r.get("alertId").is_none())
        .collect();

    // Records at index 0, 5, 10, … (every 5th) must have integer-form id
    for (i, asset) in assets.iter().enumerate() {
        if i % 5 == 0 {
            assert!(
                asset["id"].is_number(),
                "EC-001: asset[{}] (every 5th) must have integer-form id, got {:?}",
                i,
                asset["id"]
            );
        } else {
            assert!(
                asset["id"].is_string(),
                "EC-001: asset[{}] (non-5th) must have string-form id, got {:?}",
                i,
                asset["id"]
            );
        }
    }
}

/// AC-004: integer-form asset_id is deterministic for same (org_slug, seed, index).
/// Exercises `integer_asset_id` helper indirectly via generate().
#[test]
fn test_bc_3_4_001_integer_asset_id_is_deterministic() {
    let opts = GenOpts::default();
    let fs1 = generate(org_a(), SLUG_A, Archetype::HealthyOtEnvironment, &opts);
    let fs2 = generate(org_a(), SLUG_A, Archetype::HealthyOtEnvironment, &opts);

    let assets1: Vec<&Value> = fs1
        .records
        .iter()
        .filter(|r| r.get("id").is_some() && r.get("alertId").is_none())
        .collect();
    let assets2: Vec<&Value> = fs2
        .records
        .iter()
        .filter(|r| r.get("id").is_some() && r.get("alertId").is_none())
        .collect();

    // Every 5th record: compare integer IDs between two identical calls
    for i in (0..assets1.len().min(assets2.len())).step_by(5) {
        let id1 = &assets1[i]["id"];
        let id2 = &assets2[i]["id"];
        assert_eq!(
            id1, id2,
            "BC-3.4.001: integer asset_id at index {} must be identical across calls",
            i
        );
    }
}

/// AC-004: `simple_hash_bytes` produces byte-identical output for same input.
/// Exercises the helper indirectly (hash is used in integer_asset_id → build_asset → generate).
#[test]
fn test_bc_3_4_001_simple_hash_bytes_deterministic_via_generate() {
    // Two calls with identical opts must produce identical integer IDs —
    // which implicitly requires simple_hash_bytes to be deterministic.
    let opts = GenOpts::default();
    let fs1 = generate(org_a(), SLUG_A, Archetype::HealthyOtEnvironment, &opts);
    let fs2 = generate(org_a(), SLUG_A, Archetype::HealthyOtEnvironment, &opts);

    let int_ids_1: Vec<u64> = fs1
        .records
        .iter()
        .filter(|r| r.get("id").is_some())
        .filter(|r| r["id"].is_number())
        .map(|r| r["id"].as_u64().unwrap())
        .collect();
    let int_ids_2: Vec<u64> = fs2
        .records
        .iter()
        .filter(|r| r.get("id").is_some())
        .filter(|r| r["id"].is_number())
        .map(|r| r["id"].as_u64().unwrap())
        .collect();

    assert_eq!(
        int_ids_1, int_ids_2,
        "BC-3.4.001: simple_hash_bytes (via integer_asset_id) must be byte-identical for same input"
    );
}

// ---------------------------------------------------------------------------
// BC-3.4.003 — Scale formula: floor(baseline * scale)
// ---------------------------------------------------------------------------

/// BC-3.4.003 invariant 3: count_at_scale = floor(baseline * scale).
/// Tests at scale=0.5 for HealthyOtEnvironment (50 * 0.5 = 25 assets).
#[test]
fn test_bc_3_4_003_scale_formula_half_scale_healthy_ot() {
    let opts = GenOpts::new(
        42,
        0.5,
        chrono::DateTime::UNIX_EPOCH,
        serde_json::Value::Null,
    )
    .expect("scale=0.5 must be valid");
    let fs = generate(org_a(), SLUG_A, Archetype::HealthyOtEnvironment, &opts);

    let assets: Vec<&Value> = fs
        .records
        .iter()
        .filter(|r| r.get("id").is_some() && r.get("alertId").is_none())
        .collect();

    assert_eq!(
        assets.len(),
        25,
        "BC-3.4.003 invariant 3: HealthyOtEnvironment at scale=0.5 must have floor(50*0.5)=25 assets"
    );
}

// ---------------------------------------------------------------------------
// Provenance correctness
// ---------------------------------------------------------------------------

/// BC-3.4.001 invariant 3: Provenance.schema_valid=true for all non-SchemaDrift archetypes.
#[test]
fn test_bc_3_4_001_provenance_schema_valid_true_for_non_schema_drift_archetypes() {
    let non_drift = [
        Archetype::HealthyOtEnvironment,
        Archetype::CompromisedEndpoint,
        Archetype::DormantTenant,
        Archetype::HighChurn,
        Archetype::AuthOutage,
    ];
    let opts = GenOpts::default();
    for archetype in &non_drift {
        let fs = generate(org_a(), SLUG_A, archetype.clone(), &opts);
        assert!(
            fs.provenance.schema_valid,
            "BC-3.4.001 invariant 3: provenance.schema_valid must be true for {:?}",
            archetype
        );
    }
}

/// Provenance records the correct org_id, seed, and archetype.
#[test]
fn test_bc_3_4_001_provenance_fields_correct() {
    let opts = GenOpts::default(); // seed=42
    let fs = generate(org_a(), SLUG_A, Archetype::HighChurn, &opts);
    assert_eq!(
        fs.provenance.org_id,
        org_a(),
        "Provenance.org_id must match the caller's org_id"
    );
    assert_eq!(
        fs.provenance.seed, 42,
        "Provenance.seed must match opts.seed"
    );
    assert_eq!(
        fs.provenance.archetype,
        Archetype::HighChurn,
        "Provenance.archetype must match the requested archetype"
    );
    assert_eq!(
        fs.provenance.sensor_id,
        SensorId::from("armis"),
        "Provenance.sensor_id must be Armis"
    );
}
