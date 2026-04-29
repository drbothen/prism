//! Failing acceptance tests for S-3.7.05 — CrowdStrike fixture generator.
//!
//! Covers:
//!   BC-3.4.001 (Generator Determinism)
//!   BC-3.4.002 (Schema Conformance — CrowdStrike)
//!   BC-3.4.003 (8 Archetypes with defined baselines)
//!   BC-3.4.004 (Org-tagged IDs)
//!
//! VP coverage: VP-108, VP-112, VP-113, VP-114, VP-119, VP-120, VP-121
//!
//! ALL tests below are expected to fail (todo!() panic) until the implementation
//! lands. Red Gate verified before implementation begins.
//!
//! Run:
//!   cargo test -p prism-dtu-crowdstrike --features fixture-gen \
//!       --test bc_3_4_crowdstrike_generator
#![cfg(feature = "fixture-gen")]
#![allow(clippy::unwrap_used, clippy::expect_used)]

use prism_dtu_common::generator::{default_page_size, Archetype, FixtureSet, GenOpts, OrgId};
use prism_dtu_crowdstrike::generate;
use serde_json::Value;

// ---------------------------------------------------------------------------
// Test org fixtures
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

/// Collect device-type records (tagged `"_record_type": "device"`) from FixtureSet.
fn devices(fs: &FixtureSet) -> Vec<&Value> {
    fs.records
        .iter()
        .filter(|r| r.get("_record_type").and_then(Value::as_str) == Some("device"))
        .collect()
}

/// Collect tombstone-type records from FixtureSet.
fn tombstones(fs: &FixtureSet) -> Vec<&Value> {
    fs.records
        .iter()
        .filter(|r| r.get("_record_type").and_then(Value::as_str) == Some("tombstone"))
        .collect()
}

/// Collect detection-type records from FixtureSet.
fn detections(fs: &FixtureSet) -> Vec<&Value> {
    fs.records
        .iter()
        .filter(|r| r.get("_record_type").and_then(Value::as_str) == Some("detection"))
        .collect()
}

/// Collect id_page records from FixtureSet.
fn id_pages(fs: &FixtureSet) -> Vec<&Value> {
    fs.records
        .iter()
        .filter(|r| r.get("_record_type").and_then(Value::as_str) == Some("id_page"))
        .collect()
}

/// Collect oauth2_token records from FixtureSet.
fn oauth2_tokens(fs: &FixtureSet) -> Vec<&Value> {
    fs.records
        .iter()
        .filter(|r| r.get("_record_type").and_then(Value::as_str) == Some("oauth2_token"))
        .collect()
}

// ---------------------------------------------------------------------------
// AC-001 / BC-3.4.003 — Archetype baseline counts at scale=1.0
// ---------------------------------------------------------------------------

/// BC-3.4.003 postcondition: HealthyOtEnvironment → 50 devices, 5 detections,
/// 0 contained. VP-112 (archetype baseline compliance).
#[test]
fn test_bc_3_4_003_ac_001_healthy_ot_baseline_counts() {
    let fs = generate(org_a(), Archetype::HealthyOtEnvironment, GenOpts::default());
    let devs = devices(&fs);
    let dets = detections(&fs);
    assert_eq!(
        devs.len(),
        50,
        "BC-3.4.003: HealthyOtEnvironment must produce 50 device records at scale=1.0"
    );
    assert_eq!(
        dets.len(),
        5,
        "BC-3.4.003: HealthyOtEnvironment must produce 5 detection records at scale=1.0"
    );
    let contained = devs.iter().filter(|d| {
        d.get("containment_status")
            .and_then(Value::as_str)
            .map(|s| s == "contained")
            .unwrap_or(false)
    });
    assert_eq!(
        contained.count(),
        0,
        "BC-3.4.003: HealthyOtEnvironment must have 0 contained devices"
    );
}

/// BC-3.4.003 postcondition: CompromisedEndpoint → 50 devices, 20 detections,
/// >=3 with severity_id>=4, >=1 contained device.
#[test]
fn test_bc_3_4_003_ac_001_compromised_endpoint_baseline_counts() {
    let fs = generate(org_a(), Archetype::CompromisedEndpoint, GenOpts::default());
    let devs = devices(&fs);
    let dets = detections(&fs);
    assert_eq!(
        devs.len(),
        50,
        "BC-3.4.003: CompromisedEndpoint must produce 50 device records at scale=1.0"
    );
    assert_eq!(
        dets.len(),
        20,
        "BC-3.4.003: CompromisedEndpoint must produce 20 detection records at scale=1.0"
    );

    // At least 3 high-severity detections (severity_id >= 4)
    let high_sev = dets
        .iter()
        .filter(|d| {
            d.get("severity_id")
                .and_then(Value::as_u64)
                .map(|s| s >= 4)
                .unwrap_or(false)
        })
        .count();
    assert!(
        high_sev >= 3,
        "BC-3.4.003: CompromisedEndpoint must have >=3 detections with severity_id>=4, got {high_sev}"
    );

    // At least 1 contained device
    let contained = devs
        .iter()
        .filter(|d| {
            d.get("containment_status")
                .and_then(Value::as_str)
                .map(|s| s == "contained")
                .unwrap_or(false)
        })
        .count();
    assert!(
        contained >= 1,
        "BC-3.4.003 / AC-003 (EC-003): CompromisedEndpoint must have >=1 device with containment_status='contained', got {contained}"
    );
}

/// BC-3.4.003 postcondition: AuthOutage → 20 devices; first OAuth2 record is 401.
/// AC-003: first record is OAuth2 with status_code=401.
#[test]
fn test_bc_3_4_003_ac_001_auth_outage_baseline_counts() {
    let fs = generate(org_a(), Archetype::AuthOutage, GenOpts::default());
    let devs = devices(&fs);
    assert_eq!(
        devs.len(),
        20,
        "BC-3.4.003: AuthOutage must produce 20 device records at scale=1.0"
    );
}

/// BC-3.4.003 postcondition: LargeScale → 10,000 devices, 500 detections.
/// VP-113 (large-scale pagination).
#[test]
fn test_bc_3_4_003_ac_001_large_scale_baseline_counts() {
    let fs = generate(org_a(), Archetype::LargeScale, GenOpts::default());
    let devs = devices(&fs);
    let dets = detections(&fs);
    assert_eq!(
        devs.len(),
        10_000,
        "BC-3.4.003: LargeScale must produce 10,000 device records at scale=1.0"
    );
    assert_eq!(
        dets.len(),
        500,
        "BC-3.4.003: LargeScale must produce 500 detection records at scale=1.0"
    );
}

/// BC-3.4.003 postcondition: PaginationEdgeCases → page_size×3 devices;
/// 3 IdPage records + 3 detail pages.
/// AC-002: two-step pagination structure preserved.
#[test]
fn test_bc_3_4_003_ac_001_pagination_edge_cases_baseline_counts() {
    let page_size = default_page_size(prism_core::types::SensorType::CrowdStrike);
    let fs = generate(org_a(), Archetype::PaginationEdgeCases, GenOpts::default());
    let devs = devices(&fs);
    let pages = id_pages(&fs);
    assert_eq!(
        devs.len(),
        page_size * 3,
        "BC-3.4.003: PaginationEdgeCases must produce page_size*3={} device records, got {}",
        page_size * 3,
        devs.len()
    );
    assert_eq!(
        pages.len(),
        3,
        "BC-3.4.003 / AC-002: PaginationEdgeCases must produce 3 IdPage records, got {}",
        pages.len()
    );
}

/// BC-3.4.003 postcondition: SchemaDrift → 30 devices; records[0] has
/// provenance.schema_valid=false.
#[test]
fn test_bc_3_4_003_ac_001_schema_drift_baseline_counts() {
    let fs = generate(org_a(), Archetype::SchemaDrift, GenOpts::default());
    let devs = devices(&fs);
    assert_eq!(
        devs.len(),
        30,
        "BC-3.4.003: SchemaDrift must produce 30 device records at scale=1.0"
    );
}

/// BC-3.4.003 postcondition: HighChurn → 200 devices; >=20 tombstones.
#[test]
fn test_bc_3_4_003_ac_001_high_churn_baseline_counts() {
    let fs = generate(org_a(), Archetype::HighChurn, GenOpts::default());
    let devs = devices(&fs);
    let tombs = tombstones(&fs);
    assert_eq!(
        devs.len(),
        200,
        "BC-3.4.003: HighChurn must produce 200 device records at scale=1.0"
    );
    assert!(
        tombs.len() >= 20,
        "BC-3.4.003: HighChurn must produce >=20 tombstone records, got {}",
        tombs.len()
    );
}

/// BC-3.4.003 postcondition: DormantTenant → 0 records, 0 cursors.
/// EC-004 (DormantTenant 2-step IdPage is also empty).
#[test]
fn test_bc_3_4_003_ac_001_dormant_tenant_zero_records() {
    let fs = generate(org_a(), Archetype::DormantTenant, GenOpts::default());
    assert_eq!(
        fs.records.len(),
        0,
        "BC-3.4.003: DormantTenant must produce 0 records at any scale"
    );
    assert_eq!(
        fs.cursors.len(),
        0,
        "BC-3.4.003: DormantTenant must produce 0 cursors"
    );
}

/// BC-3.4.003 invariant 5: DormantTenant always 0 regardless of scale.
#[test]
fn test_bc_3_4_003_dormant_tenant_zero_at_large_scale() {
    let opts = GenOpts::new(
        42,
        100.0_f64,
        chrono::DateTime::UNIX_EPOCH,
        serde_json::Value::Null,
    )
    .expect("valid opts");
    let fs = generate(org_a(), Archetype::DormantTenant, opts);
    assert_eq!(
        fs.records.len(),
        0,
        "BC-3.4.003 invariant 5: DormantTenant must produce 0 records at scale=100.0"
    );
}

// ---------------------------------------------------------------------------
// AC-002 / BC-3.4.002 — 2-step IDs→detail pagination structure
// VP-113 (pagination structure correctness)
// VP-114 (cursor integrity)
// ---------------------------------------------------------------------------

/// AC-002 / BC-3.4.002 postcondition 1: PaginationEdgeCases produces IdPage records
/// BEFORE detail records in FixtureSet::records. IDs in each IdPage correspond to
/// the device detail records that follow.
#[test]
fn test_bc_3_4_002_ac_002_two_step_pagination_id_pages_precede_detail() {
    let fs = generate(org_a(), Archetype::PaginationEdgeCases, GenOpts::default());

    // Find the index of the last id_page record and the first device detail record
    let last_id_page_idx = fs
        .records
        .iter()
        .enumerate()
        .filter_map(|(i, r)| {
            if r.get("_record_type").and_then(Value::as_str) == Some("id_page") {
                Some(i)
            } else {
                None
            }
        })
        .last()
        .expect("BC-3.4.002: PaginationEdgeCases must have at least one IdPage record");

    let first_device_idx = fs
        .records
        .iter()
        .enumerate()
        .find_map(|(i, r)| {
            if r.get("_record_type").and_then(Value::as_str) == Some("device") {
                Some(i)
            } else {
                None
            }
        })
        .expect("BC-3.4.002: PaginationEdgeCases must have at least one device detail record");

    assert!(
        last_id_page_idx < first_device_idx,
        "BC-3.4.002 / AC-002: IdPage records (last at idx {last_id_page_idx}) must precede device detail records (first at idx {first_device_idx})"
    );
}

/// AC-002: each IdPage has a `resources` array containing only string IDs.
/// IdPage shape must match `.references/schemas/crowdstrike/types.rs:IdPage`.
#[test]
fn test_bc_3_4_002_ac_002_id_page_has_resources_array_of_strings() {
    let fs = generate(org_a(), Archetype::PaginationEdgeCases, GenOpts::default());
    let pages = id_pages(&fs);
    assert!(
        !pages.is_empty(),
        "BC-3.4.002: expected at least one id_page record"
    );
    for (i, page) in pages.iter().enumerate() {
        let resources = page
            .get("resources")
            .and_then(Value::as_array)
            .unwrap_or_else(|| panic!("BC-3.4.002: id_page[{i}] must have a 'resources' array"));
        for (j, id) in resources.iter().enumerate() {
            assert!(
                id.is_string(),
                "BC-3.4.002: id_page[{i}].resources[{j}] must be a string, got {id:?}"
            );
        }
    }
}

/// AC-002: The IDs in each IdPage correspond to device_id fields in the
/// subsequent detail records. Every ID listed in step-1 has a matching
/// step-2 detail record.
#[test]
fn test_bc_3_4_002_ac_002_id_page_ids_match_detail_device_ids() {
    let page_size = default_page_size(prism_core::types::SensorType::CrowdStrike);
    let fs = generate(org_a(), Archetype::PaginationEdgeCases, GenOpts::default());

    let all_ids_in_pages: Vec<String> = fs
        .records
        .iter()
        .filter(|r| r.get("_record_type").and_then(Value::as_str) == Some("id_page"))
        .flat_map(|page| {
            page.get("resources")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default()
        })
        .filter_map(|v| v.as_str().map(str::to_owned))
        .collect();

    let all_device_ids: std::collections::HashSet<&str> = fs
        .records
        .iter()
        .filter(|r| r.get("_record_type").and_then(Value::as_str) == Some("device"))
        .filter_map(|d| d.get("device_id").and_then(Value::as_str))
        .collect();

    assert_eq!(
        all_ids_in_pages.len(),
        page_size * 3,
        "BC-3.4.002: total IDs across all IdPages must equal page_size*3={}",
        page_size * 3
    );

    for id in &all_ids_in_pages {
        assert!(
            all_device_ids.contains(id.as_str()),
            "BC-3.4.002 / AC-002: id_page ID '{id}' has no corresponding device detail record"
        );
    }
}

/// AC-002 / VP-114: FixtureSet::cursors contains FQL offset cursors for the ID-list step.
/// PaginationEdgeCases must have 3 cursors — one per page boundary.
#[test]
fn test_bc_3_4_002_ac_002_vp_114_pagination_edge_cases_cursor_count() {
    let fs = generate(org_a(), Archetype::PaginationEdgeCases, GenOpts::default());
    assert_eq!(
        fs.cursors.len(),
        3,
        "BC-3.4.002 / VP-114: PaginationEdgeCases must produce 3 cursors (one per page boundary), got {}",
        fs.cursors.len()
    );
}

/// BC-3.4.002 / LargeScale also uses 2-step pagination structure.
#[test]
fn test_bc_3_4_002_large_scale_has_id_pages_and_detail_records() {
    let fs = generate(org_a(), Archetype::LargeScale, GenOpts::default());
    let pages = id_pages(&fs);
    let devs = devices(&fs);
    assert!(
        !pages.is_empty(),
        "BC-3.4.002: LargeScale must produce IdPage records"
    );
    assert!(
        !devs.is_empty(),
        "BC-3.4.002: LargeScale must produce device detail records"
    );
    assert!(
        !fs.cursors.is_empty(),
        "BC-3.4.002 / VP-114: LargeScale must produce at least one cursor"
    );
}

// ---------------------------------------------------------------------------
// AC-003 / BC-3.4.002 — OAuth2 token fixture (AuthOutage)
// VP-112 (auth outage fixture shape)
// ---------------------------------------------------------------------------

/// AC-003 / BC-3.4.002: AuthOutage first OAuth2 record has status_code=401.
/// VP-112 (auth outage first record is 401).
#[test]
fn test_bc_3_4_002_ac_003_vp_112_auth_outage_first_token_is_401() {
    let fs = generate(org_a(), Archetype::AuthOutage, GenOpts::default());
    let tokens = oauth2_tokens(&fs);
    assert!(
        !tokens.is_empty(),
        "BC-3.4.002 / AC-003: AuthOutage must produce at least one OAuth2 token record"
    );
    let status = tokens[0]
        .get("status_code")
        .and_then(Value::as_u64)
        .unwrap_or_else(|| panic!("BC-3.4.002: first OAuth2 record must have 'status_code' field"));
    assert_eq!(
        status, 401,
        "BC-3.4.002 / AC-003 / VP-112: first OAuth2 record must have status_code=401, got {status}"
    );
}

/// AC-003: second OAuth2 record (recovery) has status_code=200 and a deterministic
/// access_token matching `"tok-{org_slug}-{seed}-{call_n}"`.
#[test]
fn test_bc_3_4_002_ac_003_auth_outage_second_token_is_200_with_access_token() {
    let fs = generate(org_a(), Archetype::AuthOutage, GenOpts::default());
    let tokens = oauth2_tokens(&fs);
    assert!(
        tokens.len() >= 2,
        "BC-3.4.002 / AC-003: AuthOutage must produce at least 2 OAuth2 token records (401 + 200), got {}",
        tokens.len()
    );
    let status = tokens[1]
        .get("status_code")
        .and_then(Value::as_u64)
        .unwrap_or_else(|| {
            panic!("BC-3.4.002: second OAuth2 record must have 'status_code' field")
        });
    assert_eq!(
        status, 200,
        "BC-3.4.002 / AC-003: second OAuth2 record must have status_code=200, got {status}"
    );
    let access_token = tokens[1]
        .get("access_token")
        .and_then(Value::as_str)
        .unwrap_or_else(|| {
            panic!("BC-3.4.002: valid OAuth2 record must have 'access_token' field")
        });
    assert!(
        access_token.starts_with("tok-"),
        "BC-3.4.002 / AC-003: access_token must follow 'tok-{{org_slug}}-{{seed}}-{{call_n}}' format, got '{access_token}'"
    );
}

/// AC-003: OAuth2 record shape includes required fields: access_token, token_type,
/// expires_in (on 200 response); mirrors OAuth2TokenResponse in types.rs.
#[test]
fn test_bc_3_4_002_ac_003_oauth2_record_shape_matches_types_rs() {
    let fs = generate(org_a(), Archetype::AuthOutage, GenOpts::default());
    let tokens = oauth2_tokens(&fs);
    assert!(tokens.len() >= 2, "AC-003: need at least 2 OAuth2 records");
    // Second record (200) must have all fields
    let ok_token = tokens[1];
    assert!(
        ok_token.get("access_token").is_some(),
        "BC-3.4.002: OAuth2 200 record must have 'access_token'"
    );
    assert!(
        ok_token.get("token_type").is_some(),
        "BC-3.4.002: OAuth2 200 record must have 'token_type'"
    );
    assert!(
        ok_token.get("expires_in").is_some(),
        "BC-3.4.002: OAuth2 200 record must have 'expires_in'"
    );
}

/// EC-002: AuthOutage with overrides recovery_after_calls=3 → first 3 tokens are 401,
/// 4th is 200. BC-3.4.003 invariant 6.
#[test]
fn test_bc_3_4_003_ec_002_auth_outage_configurable_recovery_after_3_calls() {
    use serde_json::json;
    let opts = GenOpts::new(
        42,
        1.0_f64,
        chrono::DateTime::UNIX_EPOCH,
        json!({"auth_outage": {"recovery_after_calls": 3}}),
    )
    .expect("valid opts");
    let fs = generate(org_a(), Archetype::AuthOutage, opts);
    let tokens = oauth2_tokens(&fs);
    assert!(
        tokens.len() >= 4,
        "EC-002: AuthOutage with recovery_after_calls=3 must produce >=4 OAuth2 records, got {}",
        tokens.len()
    );
    for i in 0..3 {
        let status = tokens[i]
            .get("status_code")
            .and_then(Value::as_u64)
            .unwrap_or(0);
        assert_eq!(
            status, 401,
            "EC-002: OAuth2 record[{i}] must have status_code=401 (recovery_after_calls=3)"
        );
    }
    let status = tokens[3]
        .get("status_code")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    assert_eq!(
        status, 200,
        "EC-002: OAuth2 record[3] must have status_code=200 (recovery after 3 calls)"
    );
}

// ---------------------------------------------------------------------------
// AC-004 / BC-3.4.002 — State field alignment (device_id ↔ containment_store)
// VP-112 (field name conformance)
// ---------------------------------------------------------------------------

/// AC-004: generated FalconDevice records have a `device_id` field (matches
/// containment_store key in state.rs). No field name drift.
#[test]
fn test_bc_3_4_002_ac_004_device_records_have_device_id_field() {
    let fs = generate(org_a(), Archetype::HealthyOtEnvironment, GenOpts::default());
    let devs = devices(&fs);
    for (i, dev) in devs.iter().enumerate() {
        assert!(
            dev.get("device_id").is_some(),
            "BC-3.4.002 / AC-004: device record[{i}] must have 'device_id' field (containment_store key in state.rs)"
        );
    }
}

/// AC-004: generated FalconDetection records have a `detection_id` field (matches
/// detection_status_store key in state.rs). No field name drift.
#[test]
fn test_bc_3_4_002_ac_004_detection_records_have_detection_id_field() {
    let fs = generate(org_a(), Archetype::CompromisedEndpoint, GenOpts::default());
    let dets = detections(&fs);
    for (i, det) in dets.iter().enumerate() {
        assert!(
            det.get("detection_id").is_some(),
            "BC-3.4.002 / AC-004: detection record[{i}] must have 'detection_id' field (detection_status_store key in state.rs)"
        );
    }
}

// ---------------------------------------------------------------------------
// AC-005 / BC-3.4.004 — Org-tagged IDs
// VP-120 (every record ID contains org slug)
// ---------------------------------------------------------------------------

/// BC-3.4.004 postcondition: device_id starts with `"dev-{org_slug}-{seed}-"`.
/// VP-120 (every record ID contains org slug as substring).
#[test]
fn test_bc_3_4_004_ac_005_vp_120_device_id_starts_with_dev_org_slug_seed() {
    let fs = generate(org_a(), Archetype::HealthyOtEnvironment, GenOpts::default());
    let devs = devices(&fs);
    assert!(!devs.is_empty(), "need at least one device record");
    for (i, dev) in devs.iter().enumerate() {
        let device_id = dev
            .get("device_id")
            .and_then(Value::as_str)
            .unwrap_or_else(|| panic!("device[{i}] missing device_id"));
        assert!(
            device_id.starts_with("dev-"),
            "BC-3.4.004 / AC-005: device_id[{i}] must start with 'dev-', got '{device_id}'"
        );
        // Seed 42 must appear in the ID
        assert!(
            device_id.contains("-42-"),
            "BC-3.4.004 / AC-005: device_id[{i}] must contain seed '-42-', got '{device_id}'"
        );
    }
}

/// BC-3.4.004 postcondition: detection_id starts with `"alert-{org_slug}-{seed}-"`.
#[test]
fn test_bc_3_4_004_ac_005_detection_id_starts_with_alert_org_slug_seed() {
    let fs = generate(org_a(), Archetype::CompromisedEndpoint, GenOpts::default());
    let dets = detections(&fs);
    assert!(!dets.is_empty(), "need at least one detection record");
    for (i, det) in dets.iter().enumerate() {
        let detection_id = det
            .get("detection_id")
            .and_then(Value::as_str)
            .unwrap_or_else(|| panic!("detection[{i}] missing detection_id"));
        assert!(
            detection_id.starts_with("alert-"),
            "BC-3.4.004 / AC-005: detection_id[{i}] must start with 'alert-', got '{detection_id}'"
        );
        assert!(
            detection_id.contains("-42-"),
            "BC-3.4.004 / AC-005: detection_id[{i}] must contain seed '-42-', got '{detection_id}'"
        );
    }
}

/// BC-3.4.004 postcondition: tombstone device_id format is
/// `"dev-{org_slug}-{seed}-tomb-{n}"`.
#[test]
fn test_bc_3_4_004_ac_005_tombstone_id_format() {
    let fs = generate(org_a(), Archetype::HighChurn, GenOpts::default());
    let tombs = tombstones(&fs);
    assert!(
        !tombs.is_empty(),
        "BC-3.4.004: HighChurn must produce tombstone records"
    );
    for (i, tomb) in tombs.iter().enumerate() {
        let device_id = tomb
            .get("device_id")
            .and_then(Value::as_str)
            .unwrap_or_else(|| panic!("tombstone[{i}] missing device_id"));
        assert!(
            device_id.contains("tomb"),
            "BC-3.4.004 / AC-005: tombstone device_id[{i}] must contain 'tomb', got '{device_id}'"
        );
        assert!(
            device_id.starts_with("dev-"),
            "BC-3.4.004 / AC-005: tombstone device_id[{i}] must start with 'dev-', got '{device_id}'"
        );
    }
}

// ---------------------------------------------------------------------------
// VP-119 / BC-3.4.004 — OrgA and OrgB ID sets are disjoint
// ---------------------------------------------------------------------------

/// VP-119 / BC-3.4.004 postcondition 3: generate(orgA).ids ∩ generate(orgB).ids = ∅
/// when org slugs differ.
#[test]
fn test_bc_3_4_004_vp_119_org_a_and_org_b_id_sets_are_disjoint() {
    let fs_a = generate(org_a(), Archetype::HealthyOtEnvironment, GenOpts::default());
    let fs_b = generate(org_b(), Archetype::HealthyOtEnvironment, GenOpts::default());

    let ids_a: std::collections::HashSet<&str> = fs_a
        .records
        .iter()
        .filter_map(|r| {
            r.get("device_id")
                .or_else(|| r.get("detection_id"))
                .and_then(Value::as_str)
        })
        .collect();

    let ids_b: std::collections::HashSet<&str> = fs_b
        .records
        .iter()
        .filter_map(|r| {
            r.get("device_id")
                .or_else(|| r.get("detection_id"))
                .and_then(Value::as_str)
        })
        .collect();

    let intersection: Vec<&&str> = ids_a.intersection(&ids_b).collect();
    assert!(
        intersection.is_empty(),
        "BC-3.4.004 / VP-119: orgA and orgB ID sets must be disjoint; shared IDs: {intersection:?}"
    );
}

/// VP-120 / BC-3.4.004: every device_id and detection_id contains the org slug
/// as a substring (cross-tenant leakage detectable by inspection).
#[test]
fn test_bc_3_4_004_vp_120_every_id_contains_org_slug() {
    let fs = generate(org_a(), Archetype::CompromisedEndpoint, GenOpts::default());
    // Determine the expected org slug prefix from the first device_id
    // The slug is the hex of org_id bytes[0..4] (8 hex chars).
    let first_device_id = devices(&fs)
        .into_iter()
        .next()
        .and_then(|d| {
            d.get("device_id")
                .and_then(Value::as_str)
                .map(str::to_owned)
        })
        .expect("VP-120: need at least one device record");

    // Extract slug: everything between "dev-" and the next "-{seed}-" segment
    let slug_part = first_device_id
        .strip_prefix("dev-")
        .and_then(|s| s.rfind("-42-").map(|i| &s[..i]))
        .unwrap_or_else(|| panic!("VP-120: could not extract org slug from '{first_device_id}'"));

    // All device and detection IDs must contain the slug
    for r in &fs.records {
        if let Some(id) = r
            .get("device_id")
            .or_else(|| r.get("detection_id"))
            .and_then(Value::as_str)
        {
            assert!(
                id.contains(slug_part),
                "BC-3.4.004 / VP-120: record ID '{id}' does not contain org slug '{slug_part}'"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// AC-006 / BC-3.4.001 — Determinism (VP-108)
// ---------------------------------------------------------------------------

/// AC-006 / BC-3.4.001 postcondition 1 / VP-108: two identical calls produce
/// byte-identical FixtureSet::records.
#[test]
fn test_bc_3_4_001_ac_006_vp_108_generate_is_deterministic_healthy_ot() {
    let fs1 = generate(org_a(), Archetype::HealthyOtEnvironment, GenOpts::default());
    let fs2 = generate(org_a(), Archetype::HealthyOtEnvironment, GenOpts::default());
    let json1 = serde_json::to_string(&fs1.records).expect("serialize records1");
    let json2 = serde_json::to_string(&fs2.records).expect("serialize records2");
    assert_eq!(
        json1, json2,
        "BC-3.4.001 / AC-006 / VP-108: identical inputs must produce byte-identical records (HealthyOtEnvironment)"
    );
}

/// BC-3.4.001 / TV-3.4.001-06: determinism holds for CompromisedEndpoint (different
/// sensor + archetype combination).
#[test]
fn test_bc_3_4_001_vp_108_generate_deterministic_compromised_endpoint() {
    let fs1 = generate(org_a(), Archetype::CompromisedEndpoint, GenOpts::default());
    let fs2 = generate(org_a(), Archetype::CompromisedEndpoint, GenOpts::default());
    let json1 = serde_json::to_string(&fs1.records).expect("serialize");
    let json2 = serde_json::to_string(&fs2.records).expect("serialize");
    assert_eq!(
        json1, json2,
        "BC-3.4.001 / VP-108: CompromisedEndpoint must be deterministic"
    );
}

/// BC-3.4.001 postcondition 3: different seeds produce different records.
#[test]
fn test_bc_3_4_001_different_seeds_produce_different_records() {
    let opts1 = GenOpts::new(
        1,
        1.0,
        chrono::DateTime::UNIX_EPOCH,
        serde_json::Value::Null,
    )
    .expect("opts1");
    let opts2 = GenOpts::new(
        2,
        1.0,
        chrono::DateTime::UNIX_EPOCH,
        serde_json::Value::Null,
    )
    .expect("opts2");
    let fs1 = generate(org_a(), Archetype::HealthyOtEnvironment, opts1);
    let fs2 = generate(org_a(), Archetype::HealthyOtEnvironment, opts2);
    let json1 = serde_json::to_string(&fs1.records).expect("serialize");
    let json2 = serde_json::to_string(&fs2.records).expect("serialize");
    assert_ne!(
        json1, json2,
        "BC-3.4.001 postcondition 3: seed=1 and seed=2 must produce different records"
    );
}

/// BC-3.4.001 postcondition 4: different org_ids (same seed) produce different records.
#[test]
fn test_bc_3_4_001_different_org_ids_produce_different_records() {
    let fs_a = generate(org_a(), Archetype::HealthyOtEnvironment, GenOpts::default());
    let fs_b = generate(org_b(), Archetype::HealthyOtEnvironment, GenOpts::default());
    let json_a = serde_json::to_string(&fs_a.records).expect("serialize");
    let json_b = serde_json::to_string(&fs_b.records).expect("serialize");
    assert_ne!(
        json_a, json_b,
        "BC-3.4.001 postcondition 4: different org_ids must produce different records"
    );
}

/// BC-3.4.001 TV-3.4.001-07: seed=u64::MAX does not panic and is deterministic.
#[test]
fn test_bc_3_4_001_seed_max_does_not_panic_and_is_deterministic() {
    let opts1 = GenOpts::new(
        u64::MAX,
        1.0,
        chrono::DateTime::UNIX_EPOCH,
        serde_json::Value::Null,
    )
    .expect("opts max seed");
    let opts2 = GenOpts::new(
        u64::MAX,
        1.0,
        chrono::DateTime::UNIX_EPOCH,
        serde_json::Value::Null,
    )
    .expect("opts max seed 2");
    let fs1 = generate(org_a(), Archetype::HealthyOtEnvironment, opts1);
    let fs2 = generate(org_a(), Archetype::HealthyOtEnvironment, opts2);
    let json1 = serde_json::to_string(&fs1.records).expect("serialize");
    let json2 = serde_json::to_string(&fs2.records).expect("serialize");
    assert_eq!(
        json1, json2,
        "BC-3.4.001 TV-3.4.001-07: seed=u64::MAX must be deterministic and not panic"
    );
}

// ---------------------------------------------------------------------------
// BC-3.4.002 — Schema conformance (SchemaDrift)
// VP-112 (schema drift flag)
// ---------------------------------------------------------------------------

/// BC-3.4.002 postcondition (SchemaDrift): provenance.schema_valid = false.
/// VP-112 (schema drift detection).
#[test]
fn test_bc_3_4_002_vp_112_schema_drift_provenance_schema_valid_is_false() {
    let fs = generate(org_a(), Archetype::SchemaDrift, GenOpts::default());
    assert!(
        !fs.provenance.schema_valid,
        "BC-3.4.002 / VP-112: SchemaDrift must set provenance.schema_valid=false"
    );
}

/// BC-3.4.003 invariant 4: SchemaDrift produces exactly 1 non-conformant record.
/// The drifted record is always at index 0 (device list).
#[test]
fn test_bc_3_4_003_schema_drift_first_device_missing_required_field() {
    let fs = generate(org_a(), Archetype::SchemaDrift, GenOpts::default());
    let devs = devices(&fs);
    assert!(
        !devs.is_empty(),
        "BC-3.4.003: SchemaDrift must produce at least 1 device record"
    );
    // The drift record at index 0 must be missing `device_id` (the required field)
    // OR have it set to null, to simulate schema drift.
    let drifted = devs[0];
    let has_valid_device_id = drifted
        .get("device_id")
        .map(|v| v.is_string() && !v.as_str().unwrap_or("").is_empty())
        .unwrap_or(false);
    assert!(
        !has_valid_device_id,
        "BC-3.4.003 invariant 4: SchemaDrift records[0] must have a non-conformant (absent or null) 'device_id' field to simulate drift"
    );
}

/// BC-3.4.002 postcondition: non-SchemaDrift records must have schema_valid=true.
#[test]
fn test_bc_3_4_002_non_schema_drift_provenance_schema_valid_is_true() {
    for archetype in [
        Archetype::HealthyOtEnvironment,
        Archetype::CompromisedEndpoint,
        Archetype::HighChurn,
        Archetype::DormantTenant,
    ] {
        let fs = generate(org_a(), archetype.clone(), GenOpts::default());
        assert!(
            fs.provenance.schema_valid,
            "BC-3.4.002: non-SchemaDrift archetype {archetype:?} must set provenance.schema_valid=true"
        );
    }
}

// ---------------------------------------------------------------------------
// BC-3.4.004 postcondition 4 / VP-121 — Unregistered org returns Err
// ---------------------------------------------------------------------------

/// VP-121 / BC-3.4.004 postcondition 4: when OrgRegistry does not contain the
/// org_id, generator returns Err(UnregisteredOrg) — no records, no panic.
///
/// Note: The stub generate() signature returns FixtureSet directly. This test
/// will be updated if the signature changes to Result<FixtureSet, GeneratorError>.
/// For now it exercises the todo!() → Red Gate path and documents the VP-121 contract.
/// The test is written to fail (todo!) until implementation lands.
#[test]
fn test_bc_3_4_004_vp_121_generate_signature_supports_unregistered_org_error() {
    // This test exercises BC-3.4.004 postcondition 4 contract documentation.
    // The generate() stub panics with todo!() — that is the Red Gate failure.
    // Post-implementation: if the signature is Result<_, GeneratorError>, this
    // test asserts Err(GeneratorError::UnregisteredOrg(_)) for an unknown org.
    // If the signature is FixtureSet (no error), the implementation must panic
    // with a clear message — this test verifies the call path executes at all.
    let _ = generate(org_a(), Archetype::DormantTenant, GenOpts::default());
}

// ---------------------------------------------------------------------------
// BC-3.4.003 — Scale formula: floor(baseline * scale)
// ---------------------------------------------------------------------------

/// BC-3.4.003 invariant 3: scale formula — actual_count = floor(baseline * scale).
/// At scale=0.1, HealthyOtEnvironment → floor(50 * 0.1) = 5 devices.
#[test]
fn test_bc_3_4_003_scale_formula_healthy_ot_at_0_1() {
    let opts = GenOpts::new(
        42,
        0.1_f64,
        chrono::DateTime::UNIX_EPOCH,
        serde_json::Value::Null,
    )
    .expect("valid opts");
    let fs = generate(org_a(), Archetype::HealthyOtEnvironment, opts);
    let devs = devices(&fs);
    assert_eq!(
        devs.len(),
        5,
        "BC-3.4.003: HealthyOtEnvironment at scale=0.1 must produce floor(50*0.1)=5 devices, got {}",
        devs.len()
    );
}

// ---------------------------------------------------------------------------
// Determinism of low-level helpers via generate() round-trip
// ---------------------------------------------------------------------------

/// BC-3.4.001: make_id_page is deterministic (tested via PaginationEdgeCases
/// generate round-trip: same seed → same IdPage JSON in records).
#[test]
fn test_bc_3_4_001_make_id_page_deterministic_via_generate() {
    let fs1 = generate(org_a(), Archetype::PaginationEdgeCases, GenOpts::default());
    let fs2 = generate(org_a(), Archetype::PaginationEdgeCases, GenOpts::default());
    let pages1: Vec<&Value> = fs1
        .records
        .iter()
        .filter(|r| r.get("_record_type").and_then(Value::as_str) == Some("id_page"))
        .collect();
    let pages2: Vec<&Value> = fs2
        .records
        .iter()
        .filter(|r| r.get("_record_type").and_then(Value::as_str) == Some("id_page"))
        .collect();
    let json1 = serde_json::to_string(&pages1).expect("serialize");
    let json2 = serde_json::to_string(&pages2).expect("serialize");
    assert_eq!(
        json1, json2,
        "BC-3.4.001: make_id_page must produce deterministic output for same seed"
    );
}

/// BC-3.4.001: make_device output is deterministic (tested via generate round-trip).
#[test]
fn test_bc_3_4_001_make_device_deterministic_via_generate() {
    let fs1 = generate(org_a(), Archetype::HealthyOtEnvironment, GenOpts::default());
    let fs2 = generate(org_a(), Archetype::HealthyOtEnvironment, GenOpts::default());
    let devs1 = devices(&fs1);
    let devs2 = devices(&fs2);
    assert_eq!(
        serde_json::to_string(&devs1).unwrap(),
        serde_json::to_string(&devs2).unwrap(),
        "BC-3.4.001: make_device must produce deterministic output for same seed"
    );
}
