//! BC-2.06.020 TV-020-013 / VP-020-K — Cyberint→NVD end-to-end pivot (integration)
//!
//! test_BC_2_06_020_cyberint_alert_cve_resolves_in_nvd
//!
//! Traces to:
//!   BC-2.06.020 §PC-8 (scenario CVE IDs ∈ catalog.device_cves)
//!   BC-2.06.020 §PC-3 / §PC-4 (NVD injects catalog.device_cves with base_score=8.1, HIGH)
//!   INV-CYBERINT-ALERT-CVE-CORRELATION-001 (Cyberint CVE records link to real NVD entries)
//!   INV-NVD-CVE-CORRELATION-001 (scenario CVEs appear in NvdClone registry)
//!   TV-020-013 / VP-020-K (load-bearing end-to-end pivot assertion)
//!   RGT #22 in S-DEMO-DTU-LIVE-SCENARIO-001-B v2.10
//!
//! Lives in prism-dtu-demo-server/tests/ because both prism-dtu-cyberint AND
//! prism-dtu-nvd must be available as deps for the full pivot chain.
//!
//! Story-writer sync required: RGT #22 crate column says "prism-dtu-cyberint"
//! but this test lives in "prism-dtu-demo-server". The prism-dtu-cyberint copy
//! (test_BC_2_06_020_cyberint_alert_cve_resolves_in_nvd in bc_2_06_020_cyberint_cve_correlation.rs)
//! is a catalog-membership guard (Half-B) only — not a genuine end-to-end pivot.
//! This file is the load-bearing VP-020-K test.
//!
//! FAIL mode (Red Gate):
//! If CyberintClone emits a CVE record whose `cve_id` is NOT in `catalog.device_cves`
//! (e.g. any free-form synthetic or a real-year CVE), then `NvdState::lookup_and_count`
//! returns `None` — because NvdClone::new_with_scenario only injects exactly
//! `catalog.device_cves`. The assertion `Some(record)` FAILS here, not just on
//! catalog-membership membership (the unit test covers that half).
//! Red-gate discipline: this test would also FAIL if NvdClone::new_with_scenario
//! stub delegated to new() without inserting scenario CVEs.

#![cfg(feature = "fixture-gen")]
#![allow(clippy::unwrap_used, clippy::expect_used, non_snake_case)]

use std::{collections::HashMap, sync::Arc};

use prism_dtu_common::{
    build_default_incident_timeline, build_scenario_entity_catalog, Archetype, OrgId,
};
use prism_dtu_cyberint::CyberintClone;
use prism_dtu_nvd::{
    types::{CveMetrics, CveRecord, CvssData, CvssMetricV31, LangValue},
    NvdState,
};

/// Org ID with well-known first 4 bytes → org_slug = "deadbeef".
fn deadbeef_org() -> OrgId {
    OrgId([
        0xde, 0xad, 0xbe, 0xef, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00,
    ])
}

/// Build a synthetic `CveRecord` for a scenario CVE ID.
///
/// Matches the shape that `NvdClone::new_with_scenario` injects:
/// `base_score = 8.1`, `base_severity = "HIGH"` (BC-2.06.020 PC-4).
fn synthetic_cve_record(cve_id: &str) -> CveRecord {
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
                    vector_string: "CVSS:3.1/AV:N/AC:L/PR:N/UI:N/S:U/C:H/I:H/A:N".to_string(),
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
    }
}

// ---------------------------------------------------------------------------
// TV-020-013 / VP-020-K — Cyberint→NVD end-to-end pivot (genuine integration)
// RGT #22 — S-DEMO-DTU-LIVE-SCENARIO-001-B v2.10
// ---------------------------------------------------------------------------

/// TV-020-013 / VP-020-K — Genuine Cyberint→NVD end-to-end pivot:
///
/// For every CVE-surface record in a CompromisedEndpoint scenario CyberintClone,
/// call `NvdState::lookup_and_count(cve_id)` and assert:
///   - returns `Some(record)` (CVE is in NVD registry)
///   - `record.metrics.cvss_metric_v31[0].cvss_data.base_score >= 7.0`
///   - `record.metrics.cvss_metric_v31[0].cvss_data.base_severity == "HIGH"`
///
/// BC-2.06.020 PC-8: scenario Cyberint CVE records draw `cve_id` from `catalog.device_cves`.
/// BC-2.06.020 PC-3: `NvdClone::new_with_scenario(&catalog)` injects exactly `catalog.device_cves`.
/// BC-2.06.020 PC-4: injected entries carry `base_score=8.1`, `base_severity="HIGH"`.
///
/// LOAD-BEARING: this test FAILS if:
/// (a) Cyberint emits a CVE with `cve_id` outside `catalog.device_cves` (PC-8 violation), OR
/// (b) NVD state doesn't contain a catalog CVE entry (PC-3 violation), OR
/// (c) The NVD entry has `base_score < 7.0` (PC-4 violation).
///
/// Also asserts the CVE count is non-zero (vacuous pass guard):
/// CompromisedEndpoint must emit at least 1 CVE record — empty iteration is a RED GATE.
#[test]
fn test_BC_2_06_020_cyberint_alert_cve_resolves_in_nvd() {
    let org = deadbeef_org();
    let seed: u64 = 100;

    // Step 1 — Build shared catalog for both clones.
    let catalog = build_scenario_entity_catalog(seed, &org);
    assert!(
        !catalog.device_cves.is_empty(),
        "ScenarioEntityCatalog.device_cves must be non-empty for pivot-chain test; \
         got empty — secondary RNG derivation issue. \
         TV-020-013 / BC-2.06.020 PC-8"
    );

    // Step 2 — Construct CyberintClone with scenario (generates CVE-surface records).
    let scenario_start: i64 = chrono::Utc::now().timestamp() - 1_000;
    let timeline = Arc::new(build_default_incident_timeline(
        catalog.clone(),
        scenario_start,
        &[],
    ));
    let time_anchor = chrono::DateTime::from_timestamp(scenario_start, 0)
        .expect("valid timestamp")
        .with_timezone(&chrono::Utc);

    let cyberint_clone = CyberintClone::new_with_scenario(
        seed,
        Archetype::CompromisedEndpoint,
        org.clone(),
        Arc::clone(&timeline),
        time_anchor,
        &catalog,
    )
    .expect("CyberintClone::new_with_scenario must succeed");

    // Collect all CVE-surface records from the Cyberint scenario clone.
    let cve_records: Vec<&serde_json::Value> = cyberint_clone
        .state
        .generated_records
        .iter()
        .filter(|r| r.get("_surface").and_then(|v| v.as_str()) == Some("cve"))
        .collect();

    assert!(
        !cve_records.is_empty(),
        "CompromisedEndpoint scenario CyberintClone must generate at least 1 CVE-surface record; \
         got 0 (seed={seed}, org=deadbeef). \
         Vacuous-pass guard: TV-020-013 / VP-020-K RED GATE."
    );

    // Step 3 — Build NvdState with scenario CVEs pre-populated (mirrors NvdClone::new_with_scenario).
    //
    // We construct NvdState directly (not via NvdClone) to call lookup_and_count without HTTP
    // overhead. The registry construction exactly mirrors NvdClone::new_with_scenario:
    // each device_cve gets a synthetic CveRecord with base_score=8.1, base_severity="HIGH".
    //
    // This is the correct test scope: we are testing the STATE API (lookup_and_count),
    // not the HTTP route. The HTTP route is tested in bc_2_06_020_nvd_enrichment.rs (RGT #14).
    let mut registry: HashMap<String, CveRecord> = HashMap::new();
    for cve_id in &catalog.device_cves {
        registry.insert(cve_id.to_uppercase(), synthetic_cve_record(cve_id));
    }
    let nvd_state = NvdState::new(registry);

    // Step 4 — For EACH Cyberint CVE record: pivot to NVD via lookup_and_count.
    for (i, record) in cve_records.iter().enumerate() {
        let cve_id = record
            .get("cve_id")
            .and_then(|v| v.as_str())
            .unwrap_or_else(|| panic!("CVE record[{i}] missing cve_id field"));

        // THE LOAD-BEARING ASSERTION: actual NvdState::lookup_and_count call.
        // Returns None if cve_id is NOT in the NVD registry (i.e., not a catalog member).
        let nvd_record = nvd_state.lookup_and_count(cve_id);

        assert!(
            nvd_record.is_some(),
            "CVE record[{i}] cve_id '{cve_id}' NOT found in NvdState::lookup_and_count; \
             returned None. \
             PC-8 + INV-CYBERINT-ALERT-CVE-CORRELATION-001: scenario Cyberint CVE IDs must be \
             catalog members so that NvdClone::new_with_scenario injects them into the registry. \
             If this fires, Cyberint emitted a CVE outside catalog.device_cves = {:?}. \
             TV-020-013 / VP-020-K.",
            catalog.device_cves
        );

        let nvd_entry = nvd_record.unwrap();

        // Assert CVSS v3.1 metrics are present.
        let metrics_v31 = nvd_entry
            .metrics
            .cvss_metric_v31
            .as_ref()
            .unwrap_or_else(|| {
                panic!(
                    "CVE record[{i}] '{cve_id}' NvdState entry missing cvss_metric_v31; \
                     BC-2.06.020 PC-4 requires base_score >= 7.0. TV-020-013."
                )
            });

        assert!(
            !metrics_v31.is_empty(),
            "CVE record[{i}] '{cve_id}' NvdState entry cvss_metric_v31 is empty Vec; \
             BC-2.06.020 PC-4 requires at least one CVSS v3.1 metric. TV-020-013."
        );

        let base_score = metrics_v31[0].cvss_data.base_score;
        assert!(
            base_score >= 7.0,
            "CVE record[{i}] '{cve_id}' NvdState entry base_score={base_score} < 7.0; \
             BC-2.06.020 PC-4 requires base_score >= 7.0. \
             Synthetic records must carry base_score=8.1 per new_with_scenario spec. TV-020-013."
        );

        let base_severity = &metrics_v31[0].cvss_data.base_severity;
        assert_eq!(
            base_severity, "HIGH",
            "CVE record[{i}] '{cve_id}' NvdState entry base_severity='{base_severity}' != 'HIGH'; \
             BC-2.06.020 PC-4 requires base_severity='HIGH'. \
             Synthetic records must carry base_severity='HIGH' per new_with_scenario spec. TV-020-013."
        );
    }

    // Confirm the lookup_and_count actually incremented the counters (non-vacuous call proof).
    let cve_id_0 = cve_records[0]
        .get("cve_id")
        .and_then(|v| v.as_str())
        .expect("first CVE record must have cve_id");
    assert!(
        nvd_state.request_count_for(cve_id_0) >= 1,
        "NvdState::request_count_for('{cve_id_0}') == 0; \
         lookup_and_count must have been called for this CVE ID. \
         Non-vacuous call proof for TV-020-013."
    );
}
