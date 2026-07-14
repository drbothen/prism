//! BC-2.06.020 Cyberint CVE correlation tests (PC-8, PC-9)
//!
//! Traces to:
//!   BC-2.06.020 §PC-8 (scenario mode: all cve_id values drawn from catalog.device_cves)
//!   BC-2.06.020 §PC-9 (baseline mode: CVE-9999- namespace; no real-year CVE IDs)
//!   BC-2.06.020 INV-CYBERINT-ALERT-CVE-CORRELATION-001
//!   TV-020-011 through TV-020-014
//!
//! D-1117 2026-06-12 human-directed

#![cfg(feature = "fixture-gen")]
#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::sync::Arc;

use prism_dtu_common::{
    build_default_incident_timeline, build_scenario_entity_catalog, Archetype, OrgId,
};
use prism_dtu_cyberint::CyberintClone;

/// Org ID with well-known first 4 bytes → org_slug = "deadbeef".
fn deadbeef_org() -> OrgId {
    OrgId([
        0xde, 0xad, 0xbe, 0xef, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00,
    ])
}

// ---------------------------------------------------------------------------
// TV-020-011 / VP-020-I — Baseline mode: CVE-9999- namespace, no real-year IDs
// ---------------------------------------------------------------------------

/// TV-020-011: Baseline-mode Cyberint CVE records use CVE-9999- namespace.
///
/// BC-2.06.020 §PC-9 / INV-CYBERINT-ALERT-CVE-CORRELATION-001 baseline clause.
///
/// Every generated CVE record's cve_id MUST match `^CVE-9999-\d{4}$`.
/// No cve_id may match `^CVE-202\d-` or any real-year pattern.
///
/// Regression against pre-fix `CVE-2024-*` behavior (EC-020-015).
///
/// LOAD-BEARING: uses `new_with_seed` (baseline, no scenario catalog).
#[test]
fn test_BC_2_06_020_cyberint_baseline_cve_uses_cve_9999_namespace() {
    let org = deadbeef_org();
    let seed: u64 = 100;

    // Use CompromisedEndpoint: 10 CVE records at scale=1.0 (more records = more coverage).
    let clone = CyberintClone::new_with_seed(seed, Archetype::CompromisedEndpoint, org.clone())
        .expect("new_with_seed must succeed");

    let cve_records: Vec<&serde_json::Value> = clone
        .state
        .generated_records
        .iter()
        .filter(|r| r.get("_surface").and_then(|v| v.as_str()) == Some("cve"))
        .collect();

    assert!(
        !cve_records.is_empty(),
        "CompromisedEndpoint must generate at least 1 CVE record; got 0 \
         (seed={seed}, org=deadbeef)"
    );

    let re_9999 = regex_cve_9999();
    let re_real_year = regex_real_year();

    for (i, record) in cve_records.iter().enumerate() {
        let cve_id = record
            .get("cve_id")
            .and_then(|v| v.as_str())
            .unwrap_or_else(|| panic!("CVE record[{i}] missing cve_id field"));

        assert!(
            re_9999.is_match(cve_id),
            "CVE record[{i}] cve_id '{cve_id}' does not match ^CVE-9999-\\d{{4}}$ \
             — baseline path must use CVE-9999- namespace (PC-9 / INV-CYBERINT-ALERT-CVE-CORRELATION-001); \
             pre-fix value would be 'CVE-2024-XXXX'. \
             Regression guard: TV-020-011 / EC-020-015."
        );

        assert!(
            !re_real_year.is_match(cve_id),
            "CVE record[{i}] cve_id '{cve_id}' matches a real calendar-year pattern \
             (^CVE-(199\\d|200\\d|201\\d|202\\d)-); this would collide with real NVD advisories. \
             PC-9 sentinel: only CVE-9999- is permitted in baseline mode. \
             BC-2.06.020 INV-CYBERINT-ALERT-CVE-CORRELATION-001 universal collision-safety."
        );
    }
}

// ---------------------------------------------------------------------------
// TV-020-012 / VP-020-J — Scenario mode: CVE IDs drawn from catalog.device_cves
// ---------------------------------------------------------------------------

/// TV-020-012: Scenario-mode Cyberint CVE records use catalog CVE IDs only.
///
/// BC-2.06.020 §PC-8 / INV-CYBERINT-ALERT-CVE-CORRELATION-001 scenario clause.
///
/// Given seed=100, CompromisedEndpoint (10 CVE records), catalog.device_cves = 3 entries:
/// - Every cve_id on every CVE-surface record is a member of catalog.device_cves.
/// - The set of distinct cve_id values == catalog.device_cves (no out-of-catalog IDs).
/// - Assignment is cyclic (EC-020-012).
///
/// LOAD-BEARING: uses `new_with_scenario` with catalog injected.
#[test]
fn test_BC_2_06_020_cyberint_scenario_cve_ids_from_catalog() {
    let org = deadbeef_org();
    let seed: u64 = 100;

    let catalog = build_scenario_entity_catalog(seed, &org);
    assert!(
        !catalog.device_cves.is_empty(),
        "catalog.device_cves must be non-empty for this test to be meaningful"
    );

    let expected_catalog_set: std::collections::BTreeSet<String> =
        catalog.device_cves.iter().cloned().collect();

    // Build a minimal timeline (scenario_start = far in the past → stage 3 = fully active).
    let scenario_start: i64 = chrono::Utc::now().timestamp() - 1_000;
    let timeline = Arc::new(build_default_incident_timeline(
        catalog.clone(),
        scenario_start,
        &[],
    ));
    let time_anchor = chrono::DateTime::from_timestamp(scenario_start, 0)
        .expect("valid timestamp")
        .with_timezone(&chrono::Utc);

    let clone = CyberintClone::new_with_scenario(
        seed,
        Archetype::CompromisedEndpoint,
        org.clone(),
        Arc::clone(&timeline),
        time_anchor,
        &catalog,
    )
    .expect("new_with_scenario must succeed");

    let cve_records: Vec<&serde_json::Value> = clone
        .state
        .generated_records
        .iter()
        .filter(|r| r.get("_surface").and_then(|v| v.as_str()) == Some("cve"))
        .collect();

    assert!(
        !cve_records.is_empty(),
        "CompromisedEndpoint scenario clone must generate at least 1 CVE record"
    );

    let re_real_year = regex_real_year();
    let mut seen_ids = std::collections::BTreeSet::new();

    for (i, record) in cve_records.iter().enumerate() {
        let cve_id = record
            .get("cve_id")
            .and_then(|v| v.as_str())
            .unwrap_or_else(|| panic!("CVE record[{i}] missing cve_id field"));

        assert!(
            expected_catalog_set.contains(cve_id),
            "CVE record[{i}] cve_id '{cve_id}' is NOT in catalog.device_cves {:?}; \
             scenario mode must use only catalog CVE IDs (cyclic assignment). \
             PC-8 / INV-CYBERINT-ALERT-CVE-CORRELATION-001.",
            catalog.device_cves
        );

        assert!(
            !re_real_year.is_match(cve_id),
            "CVE record[{i}] cve_id '{cve_id}' matches a real calendar-year pattern; \
             catalog CVEs must use CVE-9999- namespace (SEC-001 sentinel). \
             INV-CYBERINT-ALERT-CVE-CORRELATION-001 universal collision-safety."
        );

        seen_ids.insert(cve_id.to_owned());
    }

    // Verify cyclic coverage: set of seen IDs equals catalog set (all entries used).
    assert_eq!(
        seen_ids, expected_catalog_set,
        "Scenario CVE records should cycle through ALL catalog.device_cves entries; \
         saw {:?}, expected {:?}. \
         EC-020-012 cyclic assignment: for i records and N catalog entries, \
         each catalog entry must appear at least once when record_count >= N.",
        seen_ids, expected_catalog_set
    );
}

// ---------------------------------------------------------------------------
// TV-020-014 / VP-020-L — Cyclic assignment: record_count > catalog size
// ---------------------------------------------------------------------------

/// TV-020-014: Cyclic catalog assignment when record count exceeds catalog size.
///
/// BC-2.06.020 EC-020-012 / INV-CYBERINT-ALERT-CVE-CORRELATION-001.
///
/// CompromisedEndpoint produces 10 CVE records; catalog has 3 entries (from gen_device_cves).
/// The 10 records should cycle: indices 0,3,6,9 → cves[0]; 1,4,7 → cves[1]; 2,5,8 → cves[2].
/// Count of records must be 10; distinct cve_id set must equal catalog.device_cves exactly.
///
/// LOAD-BEARING: checks record count, distinct set, and per-index cyclic pattern.
#[test]
fn test_BC_2_06_020_cyberint_scenario_cyclic_catalog_assignment() {
    let org = deadbeef_org();
    let seed: u64 = 100;

    let catalog = build_scenario_entity_catalog(seed, &org);
    let catalog_cves = &catalog.device_cves;
    let n = catalog_cves.len();
    assert!(n > 0, "catalog.device_cves must be non-empty");

    let scenario_start: i64 = chrono::Utc::now().timestamp() - 1_000;
    let timeline = Arc::new(build_default_incident_timeline(
        catalog.clone(),
        scenario_start,
        &[],
    ));
    let time_anchor = chrono::DateTime::from_timestamp(scenario_start, 0)
        .expect("valid timestamp")
        .with_timezone(&chrono::Utc);

    let clone = CyberintClone::new_with_scenario(
        seed,
        Archetype::CompromisedEndpoint,
        org.clone(),
        Arc::clone(&timeline),
        time_anchor,
        &catalog,
    )
    .expect("new_with_scenario must succeed");

    let cve_records: Vec<&serde_json::Value> = clone
        .state
        .generated_records
        .iter()
        .filter(|r| r.get("_surface").and_then(|v| v.as_str()) == Some("cve"))
        .collect();

    // CompromisedEndpoint CVE baseline = 10.
    assert_eq!(
        cve_records.len(),
        10,
        "CompromisedEndpoint must produce exactly 10 CVE records; got {}. \
         Record count must be independent of catalog size.",
        cve_records.len()
    );

    // Verify cyclic assignment per index.
    for (i, record) in cve_records.iter().enumerate() {
        let cve_id = record
            .get("cve_id")
            .and_then(|v| v.as_str())
            .unwrap_or_else(|| panic!("CVE record[{i}] missing cve_id"));

        let expected = &catalog_cves[i % n];
        assert_eq!(
            cve_id, expected.as_str(),
            "CVE record[{i}] cve_id '{cve_id}' should be catalog_cves[{i}%{n}={idx}]='{expected}'; \
             cyclic assignment mismatch. EC-020-012.",
            idx = i % n
        );
    }

    // Distinct set must equal catalog exactly.
    let distinct: std::collections::BTreeSet<String> = cve_records
        .iter()
        .filter_map(|r| r.get("cve_id")?.as_str().map(|s| s.to_owned()))
        .collect();
    let catalog_set: std::collections::BTreeSet<String> = catalog_cves.iter().cloned().collect();
    assert_eq!(
        distinct, catalog_set,
        "Distinct CVE IDs in scenario records must equal catalog.device_cves exactly; \
         got {:?}, expected {:?}. TV-020-014 / EC-020-012.",
        distinct, catalog_set
    );
}

// ---------------------------------------------------------------------------
// Helper: compile simple regex patterns inline (no external crate needed)
// ---------------------------------------------------------------------------

/// Minimal manual CVE-9999-NNNN pattern matcher (4-digit suffix).
///
/// Returns true if `s` matches `^CVE-9999-\d{4}$` exactly.
fn matches_cve_9999(s: &str) -> bool {
    if let Some(rest) = s.strip_prefix("CVE-9999-") {
        rest.len() == 4 && rest.chars().all(|c| c.is_ascii_digit())
    } else {
        false
    }
}

/// Returns true if `s` starts with a real calendar year in CVE format.
///
/// Checks: `CVE-199X-`, `CVE-200X-`, `CVE-201X-`, `CVE-202X-`.
fn matches_real_year(s: &str) -> bool {
    let prefixes = ["CVE-199", "CVE-200", "CVE-201", "CVE-202"];
    prefixes.iter().any(|p| s.starts_with(p))
}

/// Wrapper struct to provide `.is_match()` for the 9999 pattern.
struct Cve9999Regex;
impl Cve9999Regex {
    fn is_match(&self, s: &str) -> bool {
        matches_cve_9999(s)
    }
}

/// Wrapper struct to provide `.is_match()` for the real-year pattern.
struct RealYearRegex;
impl RealYearRegex {
    fn is_match(&self, s: &str) -> bool {
        matches_real_year(s)
    }
}

fn regex_cve_9999() -> Cve9999Regex {
    Cve9999Regex
}

fn regex_real_year() -> RealYearRegex {
    RealYearRegex
}
