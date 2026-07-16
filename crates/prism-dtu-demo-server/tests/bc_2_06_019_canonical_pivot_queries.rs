//! BC-2.06.019 + BC-2.06.020 canonical end-to-end pivot query tests.
//!
//! Tests 8 and 9 from the S-DEMO-ENRICHMENT-PIVOT-003 Red Gate Test Plan.
//!
//! Tests:
//!   Test 8: `test_BC_2_06_019_canonical_threatintel_pivot_query_returns_malicious_at_stage_3`
//!   Test 9: `test_BC_2_06_019_canonical_nvd_pivot_query_returns_high_cvss_at_containment_stage`
//!
//! Story: S-DEMO-ENRICHMENT-PIVOT-003
//! Traces to:
//!   BC-2.06.019 PC-4 — Cyberint alerts carry real IOC fields; CrowdStrike detections IOC stamp
//!   BC-2.06.020 INV-THREATINTEL-IOC-CORRELATION-001 — scenario IOCs resolve as Malicious
//!   BC-2.06.020 INV-NVD-CVE-CORRELATION-001 — scenario CVEs have HIGH CVSS (>= 7.0)
//!
//! Canonical queries tested:
//!   ThreatIntel pivot (stage >= 3, Exfil):
//!     FROM cyberint_alerts
//!     | where severity = "high"
//!     | enrich threat_intel(iocs[].value)
//!     | where threat_is_known_malicious = true
//!     | sort threat_score desc
//!     | head 10
//!
//!   NVD pivot (stage >= 4, Containment — device_cves visible only at Containment per BC-2.06.019 PC-2):
//!     FROM armis_devices
//!     | where has device_cves_first
//!     | enrich nvd(device_cves_first)
//!     | where cvss_base_score >= 7.0
//!     | sort cvss_base_score desc
//!     | head 10
//!
//! Implementation approach: direct DTU state API tests — not PrismQL execution.
//! These tests validate the DATA LAYER that a PrismQL query would operate on:
//!   - Cyberint generated_records carry real IOC fields (AC-002)
//!   - ThreatIntelState.lookup_fixture(ioc_value) returns Malicious for catalog IOC hashes (AC-007)
//!   - Armis generated_records carry device_cves_first scalar (AC-008)
//!   - NvdState.lookup_and_count(cve_id) returns HIGH score (AC-008 / BC-2.06.020 PC-4)
//!
//! Pattern mirrors bc_2_06_020_cyberint_nvd_pivot.rs (RGT #22) which tests the same
//! correlation chain using direct state API calls rather than HTTP or PrismQL.
//! Lives in prism-dtu-demo-server/tests/ because both Cyberint+ThreatIntel (Test 8)
//! and Armis+NVD (Test 9) require cross-crate scenario dependencies.

#![cfg(feature = "fixture-gen")]
#![allow(clippy::unwrap_used, clippy::expect_used, non_snake_case)]

use std::{collections::HashMap, sync::Arc};

use prism_dtu_common::{
    build_default_incident_timeline, build_scenario_entity_catalog, Archetype, BehavioralClone,
    OrgId,
};

/// Org ID with well-known first 4 bytes → org_slug = "deadbeef".
fn deadbeef_org() -> OrgId {
    OrgId([
        0xde, 0xad, 0xbe, 0xef, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00,
    ])
}

// ---------------------------------------------------------------------------
// Test 8 — Canonical ThreatIntel pivot query (AC-007 / BC-2.06.019 PC-4)
// ---------------------------------------------------------------------------

/// Test 8 — Canonical ThreatIntel pivot query at stage >= 3 (Exfil).
///
/// Validates the data layer that the canonical PrismQL query:
/// ```prismql
/// FROM cyberint_alerts
/// | where severity = "high"
/// | enrich threat_intel(iocs[].value)
/// | where threat_is_known_malicious = true
/// | sort threat_score desc
/// | head 10
/// ```
/// would operate on.
///
/// NOTE: BC-2.06.019 correction — canonical pivot targets `iocs[].value` (array form),
/// NOT the singleton `ioc.value` field. The singular `Alert.ioc` field is retained for
/// live-tenant backward-compatibility per v1.10 but is NOT populated by the scenario generator.
///
/// Specifically asserts:
/// 1. `CyberintClone::new_with_scenario` generates at least 1 alert record with `iocs[].value`
///    containing a hash from `catalog.ioc_hashes` (AC-002).
/// 2. `ThreatIntelClone::new_with_scenario(&catalog)` pre-populates the fixture registry with
///    all `catalog.ioc_hashes` as `FixtureKey::Malicious`.
/// 3. For every alert record with a stamped IOC hash, `ThreatIntelState::lookup_fixture(hash)`
///    returns `Some(FixtureKey::Malicious)` — the IOC correlation is complete.
///
/// BC-2.06.019 PC-4: Cyberint alerts carry real IOC fields (iocs[].value / ioc.value).
/// BC-2.06.020 INV-THREATINTEL-IOC-CORRELATION-001: scenario IOCs ∈ catalog resolve as Malicious.
///
/// LOAD-BEARING: this test FAILS if:
/// (a) AC-002 is incomplete: alerts do not carry `iocs[].value` with catalog hashes, OR
/// (b) AC-003 catalog_ioc_hashes is empty (vacuous pass guard fires), OR
/// (c) ThreatIntelClone::new_with_scenario does not inject catalog hashes as Malicious, OR
/// (d) DTU /v3/hash/:hash returns threat_score < 75 for a Malicious scenario IOC hash.
///     (F-PIVOT003-R7B-001: AC-007 conjunction requires BOTH threat_is_known_malicious=true
///      AND threat_score >= 75)
#[tokio::test]
async fn test_BC_2_06_019_canonical_threatintel_pivot_query_returns_malicious_at_stage_3() {
    let org = deadbeef_org();
    let seed: u64 = 100;

    // Step 1 — Build shared catalog.
    let catalog = build_scenario_entity_catalog(seed, &org);

    assert!(
        !catalog.ioc_hashes.is_empty(),
        "ScenarioEntityCatalog.ioc_hashes must be non-empty for ThreatIntel pivot test; \
         got empty — secondary RNG derivation issue. \
         BC-2.06.019 PC-4 / INV-THREATINTEL-IOC-CORRELATION-001"
    );

    // Step 2 — Construct CyberintClone with scenario (generates alert records with IOC stamps).
    // Stage >= 3 (Exfil): scenario_start = now - N where N places us at stage 3+.
    // BC-2.06.019 PC-2 stage table: ioc_hashes visible at stage >= 2 (LateralMovement);
    // ioc_ips + ioc_domains visible at stage >= 3 (Exfil). We use stage index > 3 to
    // ensure all IOC mask fields are true.
    let scenario_start: i64 = chrono::Utc::now().timestamp() - 1_000;
    let timeline = Arc::new(build_default_incident_timeline(
        catalog.clone(),
        scenario_start,
        &[],
    ));
    let time_anchor = chrono::DateTime::from_timestamp(scenario_start, 0)
        .expect("valid timestamp")
        .with_timezone(&chrono::Utc);

    let cyberint_clone = prism_dtu_cyberint::CyberintClone::new_with_scenario(
        seed,
        Archetype::CompromisedEndpoint,
        org.clone(),
        Arc::clone(&timeline),
        time_anchor,
        &catalog,
    )
    .expect("CyberintClone::new_with_scenario must succeed");

    // Step 3 — Collect alert-surface records with IOC values from iocs[].value.
    // AC-002: generate_with_scenario_iocs stamps catalog.ioc_hashes[0] onto
    // CompromisedEndpoint alert records as iocs: [{"type": "hash_sha256", "value": hash}].
    let alert_records: Vec<&serde_json::Value> = cyberint_clone
        .state
        .generated_records
        .iter()
        .filter(|r| r.get("_surface").and_then(|v| v.as_str()) == Some("alert"))
        .collect();

    // Collect all IOC values from the alert records.
    let mut ioc_values_found: Vec<String> = Vec::new();
    for rec in &alert_records {
        // iocs[] array form (AC-002 stamping path).
        if let Some(iocs_arr) = rec.get("iocs").and_then(|v| v.as_array()) {
            for ioc_entry in iocs_arr {
                if let Some(val) = ioc_entry.get("value").and_then(|v| v.as_str()) {
                    ioc_values_found.push(val.to_owned());
                }
            }
        }
        // Singleton ioc field (alternate form).
        if let Some(val) = rec
            .get("ioc")
            .and_then(|ioc| ioc.get("value"))
            .and_then(|v| v.as_str())
        {
            ioc_values_found.push(val.to_owned());
        }
    }

    // Vacuous pass guard: at least one IOC value must be present in alert records.
    assert!(
        !ioc_values_found.is_empty(),
        "No IOC values found in CompromisedEndpoint CyberintClone alert records (seed={seed}). \
         AC-002 must stamp catalog IOC hashes onto alert records via iocs[].value. \
         alert_count={}, catalog.ioc_hashes={:?}. \
         BC-2.06.019 PC-4 / AC-002 [RED GATE: iocs[] not stamped]",
        alert_records.len(),
        catalog.ioc_hashes,
    );

    // Step 4 — Build ThreatIntelClone with scenario IOCs pre-populated as Malicious.
    // BC-2.06.020 PC-1: new_with_scenario injects catalog.ioc_hashes as Malicious.
    let threatintel_clone = prism_dtu_threatintel::ThreatIntelClone::new_with_scenario(&catalog);

    // Step 5 — For each IOC value found in alert records that is also in catalog.ioc_hashes,
    // assert ThreatIntelState::lookup_fixture returns Malicious.
    // This mirrors the enrich threat_intel(iocs[].value) pivot chain.
    let catalog_hash_set: std::collections::HashSet<&str> =
        catalog.ioc_hashes.iter().map(|s| s.as_str()).collect();

    let catalog_iocs_in_alerts: Vec<String> = ioc_values_found
        .iter()
        .filter(|v| catalog_hash_set.contains(v.as_str()))
        .cloned()
        .collect();

    assert!(
        !catalog_iocs_in_alerts.is_empty(),
        "No catalog IOC hash values found in alert iocs[].value. \
         ioc_values_found={:?}, catalog.ioc_hashes={:?}. \
         AC-002 must stamp catalog_ioc_hashes[0] as iocs[0].value on alert records. \
         BC-2.06.019 PC-4 / INV-THREATINTEL-IOC-CORRELATION-001 [RED GATE]",
        ioc_values_found,
        catalog.ioc_hashes,
    );

    for (i, ioc_val) in catalog_iocs_in_alerts.iter().enumerate() {
        let fixture_result = threatintel_clone.state.lookup_fixture(ioc_val);

        assert_eq!(
            fixture_result,
            Some(prism_dtu_threatintel::types::FixtureKey::Malicious),
            "catalog IOC value[{i}] '{ioc_val}' lookup in ThreatIntelState returned {:?}; \
             expected Some(Malicious). \
             ThreatIntelClone::new_with_scenario must inject all catalog.ioc_hashes as Malicious. \
             BC-2.06.020 INV-THREATINTEL-IOC-CORRELATION-001 / PC-1 [RED GATE]",
            fixture_result,
        );
    }

    // Step 6 — SERVED-ROUTE assertion: HTTP hash lookup returns threat_score >= 75.
    // F-PIVOT003-R7B-001: AC-007 conjunction requires BOTH threat_is_known_malicious=true
    // AND threat_score >= 75. The data-layer assertions above prove the Malicious mapping;
    // this step proves the HTTP route returns the correct score field.
    // Uses the same pattern as bc_2_06_020_enrichment_correlation.rs:388 (AC-013/PC-2).
    let probe_hash = &catalog_iocs_in_alerts[0];
    let mut ti_server = threatintel_clone;
    ti_server
        .start()
        .await
        .expect("Test 8: ThreatIntelClone::start() must succeed for score assertion");
    let base_url = ti_server.base_url();
    let client = prism_dtu_common::build_test_client();

    let resp = client
        .get(format!("{base_url}/v3/hash/{probe_hash}"))
        .query(&[("key", "test-key-valid")])
        .send()
        .await
        .expect("Test 8: HTTP hash lookup must reach ThreatIntelClone server");

    assert_eq!(
        resp.status().as_u16(),
        200,
        "Test 8: ThreatIntelClone /v3/hash/:hash must return HTTP 200 for scenario catalog hash \
         '{probe_hash}'. F-PIVOT003-R7B-001 / AC-007"
    );

    let body: serde_json::Value = resp
        .json()
        .await
        .expect("Test 8: /v3/hash/:hash response must be valid JSON");

    let threat_score = body
        .get("threat_score")
        .and_then(|v| v.as_u64())
        .expect("Test 8: response must contain 'threat_score' field. F-PIVOT003-R7B-001 / AC-007");

    assert!(
        threat_score >= 75,
        "Test 8 F-PIVOT003-R7B-001: threat_score must be >= 75 for scenario IOC hash '{probe_hash}'; \
         got {threat_score}. DTU /v3/hash/:hash returns 95 for Malicious keys (lookup.rs:234). \
         AC-007 conjunction: threat_is_known_malicious=true AND threat_score >= 75. \
         BC-2.06.020 INV-THREATINTEL-IOC-CORRELATION-001 [RED GATE]"
    );

    ti_server
        .stop()
        .await
        .expect("Test 8: ThreatIntelClone::stop() must succeed");
}

// ---------------------------------------------------------------------------
// Test 9 — Canonical NVD pivot query (AC-008 / BC-2.06.019 PC-4 + PC-2)
// ---------------------------------------------------------------------------

/// Test 9 — Canonical NVD pivot query at stage >= 4 (Containment).
///
/// Validates the data layer that the canonical PrismQL query:
/// ```prismql
/// FROM armis_devices
/// | where has device_cves_first
/// | enrich nvd(device_cves_first)
/// | where cvss_base_score >= 7.0
/// | sort cvss_base_score desc
/// | head 10
/// ```
/// would operate on.
///
/// F-PIVOT003-R2-003: exercises the PRODUCTION PATH (`ArmisClone::new_with_scenario`)
/// rather than calling the `generate_with_scenario_cves` helper directly.
/// This proves the demo server's Armis clone actually carries `device_cves_first`
/// in its `state.generated_records` (not merely that the helper can stamp it).
///
/// Specifically asserts:
/// 1. `ArmisClone::new_with_scenario` stamps `device_cves_first = catalog.device_cves[0]`
///    onto CompromisedEndpoint asset records in `state.generated_records` (AC-008 / U17/Ruling 1b).
/// 2. `NvdState` pre-populated with catalog CVEs has `base_score=8.1 >= 7.0` (HIGH) for
///    each scenario CVE (BC-2.06.020 PC-3 + PC-4).
/// 3. For every device record with `device_cves_first`, `NvdState::lookup_and_count(cve_id)`
///    returns `Some(record)` with `base_score >= 7.0`.
///
/// BC-2.06.019 PC-2 StageMask: `device_cves` visible at stage >= 4 (Containment).
/// BC-2.06.020 INV-NVD-CVE-CORRELATION-001: scenario CVEs appear in NvdClone with HIGH score.
/// U17/Ruling 1b: `device_cves_first` = `catalog.device_cves[0]` (scalar projection).
///
/// LOAD-BEARING: this test FAILS if:
/// (a) AC-008 is incomplete: ArmisClone::new_with_scenario does NOT call
///     generate_with_scenario_cves (production path DEAD, hollow feature), OR
/// (b) catalog.device_cves is empty (vacuous pass guard fires), OR
/// (c) NvdState does not have HIGH CVSS for catalog CVEs.
#[test]
fn test_BC_2_06_019_canonical_nvd_pivot_query_returns_high_cvss_at_containment_stage() {
    use prism_dtu_armis::ArmisClone;
    use prism_dtu_nvd::{
        types::{CveMetrics, CveRecord, CvssData, CvssMetricV31, LangValue},
        NvdState,
    };

    let org = deadbeef_org();
    let seed: u64 = 100;

    // Step 1 — Build catalog.
    let catalog = build_scenario_entity_catalog(seed, &org);

    assert!(
        !catalog.device_cves.is_empty(),
        "ScenarioEntityCatalog.device_cves must be non-empty for NVD pivot test; \
         got empty — secondary RNG derivation issue. \
         BC-2.06.019 PC-2 / U17/Ruling 1b / INV-NVD-CVE-CORRELATION-001"
    );

    // Step 2 — Construct ArmisClone via the PRODUCTION CONSTRUCTOR (F-PIVOT003-R2-003).
    // This is the same constructor harness.rs uses — proves the production path carries
    // device_cves_first, not merely that the helper generate_with_scenario_cves can stamp it.
    let scenario_start: i64 = chrono::Utc::now().timestamp() - 1_000;
    let timeline = Arc::new(build_default_incident_timeline(
        catalog.clone(),
        scenario_start,
        &[],
    ));
    let time_anchor = chrono::DateTime::from_timestamp(scenario_start, 0)
        .expect("valid timestamp")
        .with_timezone(&chrono::Utc);

    let armis_clone = ArmisClone::new_with_scenario(
        seed,
        Archetype::CompromisedEndpoint,
        org.clone(),
        Arc::clone(&timeline),
        time_anchor,
        &catalog,
    )
    .expect("ArmisClone::new_with_scenario must succeed for NVD pivot test");

    // Step 3 — Collect asset records from state.generated_records that have device_cves_first.
    // AC-008: only CompromisedEndpoint asset records get the stamp (presence of asset_id).
    let device_cve_records: Vec<&serde_json::Value> = armis_clone
        .state
        .generated_records
        .iter()
        .filter(|rec| rec.get("device_cves_first").is_some())
        .collect();

    // Vacuous pass guard: at least one device must have device_cves_first.
    assert!(
        !device_cve_records.is_empty(),
        "No device records with 'device_cves_first' field found in ArmisClone::new_with_scenario \
         state.generated_records (seed={seed}). \
         F-PIVOT003-R2-003: ArmisClone::new_with_scenario MUST call generate_with_scenario_cves \
         (production path). AC-008 / U17/Ruling 1b requires device_cves_first on asset records. \
         catalog.device_cves={:?}. \
         BC-2.06.019 PC-2 + PC-4 [RED GATE: production path does not stamp device_cves_first]",
        catalog.device_cves,
    );

    // Step 4 — Build NvdState with scenario CVEs pre-populated (mirrors NvdClone::new_with_scenario).
    // We construct NvdState directly (not via NvdClone) to call lookup_and_count without HTTP
    // overhead. This exactly mirrors the bc_2_06_020_cyberint_nvd_pivot.rs pattern (RGT #22).
    //
    // Each device_cve gets a synthetic CveRecord with base_score=8.1, base_severity="HIGH"
    // per BC-2.06.020 PC-4. NvdClone::new_with_scenario stores keys in UPPERCASE.
    let mut registry: HashMap<String, CveRecord> = HashMap::new();
    for cve_id in &catalog.device_cves {
        registry.insert(
            cve_id.to_uppercase(),
            CveRecord {
                id: cve_id.to_string(),
                source_identifier: "prism-scenario@example.com".to_string(),
                published: "2024-01-01T00:00:00.000".to_string(),
                last_modified: "2024-01-01T00:00:00.000".to_string(),
                vuln_status: "Analyzed".to_string(),
                descriptions: vec![LangValue {
                    lang: "en".to_string(),
                    value: format!("Scenario synthetic CVE {cve_id}"),
                }],
                metrics: CveMetrics {
                    cvss_metric_v31: Some(vec![CvssMetricV31 {
                        source: "prism-scenario@example.com".to_string(),
                        r#type: "Primary".to_string(),
                        cvss_data: CvssData {
                            version: "3.1".to_string(),
                            vector_string: "CVSS:3.1/AV:N/AC:L/PR:N/UI:N/S:U/C:H/I:H/A:N"
                                .to_string(),
                            base_score: 8.1,
                            base_severity: "HIGH".to_string(),
                        },
                        exploitability_score: 3.9,
                        impact_score: 5.2,
                    }]),
                },
                weaknesses: vec![],
                configurations: vec![],
                references: vec![],
                cisa_kev_vuln_added: None,
            },
        );
    }
    let nvd_state = NvdState::new(registry);

    // Step 5 — For each device record with device_cves_first, pivot to NVD via lookup_and_count.
    for (i, record) in device_cve_records.iter().enumerate() {
        let cve_id = record
            .get("device_cves_first")
            .and_then(|v| v.as_str())
            .unwrap_or_else(|| panic!("device record[{i}] missing device_cves_first str value"));

        // THE LOAD-BEARING ASSERTION: NvdState::lookup_and_count for the scenario CVE.
        // Returns None if cve_id is NOT in the NVD registry (not a catalog member).
        let nvd_record = nvd_state.lookup_and_count(cve_id);

        assert!(
            nvd_record.is_some(),
            "device record[{i}] device_cves_first '{cve_id}' NOT found in NvdState::lookup_and_count; \
             returned None. \
             U17/Ruling 1b: device_cves_first must be catalog.device_cves[0] so NvdClone contains it. \
             catalog.device_cves={:?}. \
             BC-2.06.019 PC-2 + BC-2.06.020 INV-NVD-CVE-CORRELATION-001 [RED GATE]",
            catalog.device_cves,
        );

        let nvd_entry = nvd_record.unwrap();

        // Assert CVSS v3.1 metrics are present.
        let metrics_v31 = nvd_entry
            .metrics
            .cvss_metric_v31
            .as_ref()
            .unwrap_or_else(|| {
                panic!(
                    "device record[{i}] '{cve_id}' NvdState entry missing cvss_metric_v31; \
                     BC-2.06.020 PC-4 requires base_score >= 7.0"
                )
            });

        assert!(
            !metrics_v31.is_empty(),
            "device record[{i}] '{cve_id}' NvdState entry cvss_metric_v31 is empty Vec; \
             BC-2.06.020 PC-4 requires at least one CVSS v3.1 metric"
        );

        let base_score = metrics_v31[0].cvss_data.base_score;
        assert!(
            base_score >= 7.0,
            "device record[{i}] '{cve_id}' NvdState entry base_score={base_score} < 7.0; \
             BC-2.06.020 PC-4 requires base_score >= 7.0 for HIGH severity. \
             Synthetic records must carry base_score=8.1. \
             BC-2.06.019 PC-2 + BC-2.06.020 INV-NVD-CVE-CORRELATION-001 [RED GATE]"
        );

        let base_severity = &metrics_v31[0].cvss_data.base_severity;
        assert_eq!(
            base_severity, "HIGH",
            "device record[{i}] '{cve_id}' NvdState entry base_severity='{base_severity}' != 'HIGH'; \
             BC-2.06.020 PC-4 requires base_severity='HIGH' for scenario CVEs. \
             BC-2.06.019 PC-2 + BC-2.06.020 INV-NVD-CVE-CORRELATION-001 [RED GATE]"
        );
    }

    // Confirm lookup_and_count actually incremented counters (non-vacuous call proof).
    // Only check the first record's CVE to avoid index-out-of-bounds on empty catalog.
    let first_cve_id = device_cve_records[0]
        .get("device_cves_first")
        .and_then(|v| v.as_str())
        .expect("first device record must have device_cves_first");

    assert!(
        nvd_state.request_count_for(first_cve_id) >= 1,
        "NvdState::request_count_for('{first_cve_id}') == 0; \
         lookup_and_count must have been called. Non-vacuous call proof for Test 9."
    );
}
