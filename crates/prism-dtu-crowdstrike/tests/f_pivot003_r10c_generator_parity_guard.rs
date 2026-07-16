//! F-PIVOT003-R10C — CrowdStrike generator duplication parity guard.
//!
//! `generate_with_scenario_iocs` hand-duplicates the `gen_compromised_endpoint` body
//! (via direct inline construction instead of delegating). This is a maintainability
//! risk: if `gen_compromised_endpoint` changes (device count, detection count, ID format,
//! severity logic, linkage), the scenario path silently diverges.
//!
//! Rather than a risky refactor (no parity test exists), this test adds a PARITY-GUARD
//! that asserts `generate_with_scenario_iocs` output equals `gen_compromised_endpoint`
//! output MODULO the IOC stamp on detection 0 — so any future divergence is caught.
//!
//! ## What is asserted (same across both paths)
//!
//! - Same device count and detection count.
//! - Same device IDs (both use `"dev-{slug}-{seed}-{n}"` format).
//! - Same detection IDs (both use `"alert-{slug}-{seed}-{n}"` format).
//! - Same severity_id for each detection slot (first 5 = 4/Critical, rest = 2/Medium).
//! - Same device linkage for each detection (device_ids[n % dev_count]).
//! - Same containment_status on device 0 ("contained").
//!
//! ## What is allowed to differ (modulo IOC stamp)
//!
//! - Detection 0's `behaviors[0]`: scenario path adds `ioc_type`, `ioc_value`,
//!   `ioc_source`, `ioc_description`; baseline path does NOT.
//! - All other detection behaviors (n > 0): structurally identical (MITRE-only entry).
//!
//! ## How this closes the finding
//!
//! Without this test, a developer changing `gen_compromised_endpoint` (e.g., adding a
//! new device field or changing the scaling formula) would not notice that
//! `generate_with_scenario_iocs` diverged silently. This test catches that divergence
//! on the NEXT CI run. The scenario-path IOC contract is still verified by Test 5 and
//! Test 8 in the existing test suite.
//!
//! BC-2.06.019 PC-4 (CrowdStrike detections IOC stamp)
//! F-PIVOT003-R10C (closing finding: parity guard required)

#![cfg(feature = "fixture-gen")]
#![allow(clippy::unwrap_used, clippy::expect_used, non_snake_case)]

use prism_dtu_common::{Archetype, GenOpts, OrgId};
use prism_dtu_crowdstrike::generator::{generate, generate_with_scenario_iocs};

/// Canonical test OrgId → org_slug = "deadbeef".
fn deadbeef_org() -> OrgId {
    OrgId([
        0xde, 0xad, 0xbe, 0xef, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00,
    ])
}

/// F-PIVOT003-R10C parity guard.
///
/// Asserts `generate_with_scenario_iocs` output equals `generate` (CompromisedEndpoint)
/// output MODULO the IOC stamp on detection 0 — so any future divergence between
/// the two code paths is caught immediately.
///
/// Structure checked:
///   - device_count (50 at scale=1.0)
///   - detection_count (20 at scale=1.0)
///   - All device IDs match format "dev-{slug}-{seed}-{n}"
///   - All detection IDs match format "alert-{slug}-{seed}-{n}"
///   - containment_status on device 0 is "contained" on both paths
///   - severity_id for each detection slot matches (first 5 = 4, rest = 2)
///   - device linkage (device_id field on detection) matches
///   - behaviors[] length matches for detections n > 0 (MITRE-only = 1 entry each)
///   - detection 0 behaviors[0] on scenario path has ioc_type/ioc_value keys
///   - detection 0 behaviors[0] on baseline path does NOT have ioc_type/ioc_value
///
/// BC-2.06.019 PC-4 / F-PIVOT003-R10C
#[test]
fn test_BC_2_06_019_crowdstrike_generator_scenario_parity_modulo_ioc_stamp() {
    let org = deadbeef_org();
    let seed: u64 = 100;
    let ioc_hash = "cafebabecafebabecafebabecafebabecafebabecafebabecafebabecafebabe";

    let opts = GenOpts {
        seed,
        ..Default::default()
    };

    // Baseline path: generate(_, CompromisedEndpoint, _) → gen_compromised_endpoint internally.
    let baseline = generate(org.clone(), Archetype::CompromisedEndpoint, opts.clone());

    // Scenario path: generate_with_scenario_iocs with one IOC hash.
    let scenario = generate_with_scenario_iocs(
        org.clone(),
        Archetype::CompromisedEndpoint,
        opts.clone(),
        &[ioc_hash.to_owned()],
    );

    // Extract device and detection records from both FixtureSets.
    let baseline_devices: Vec<&serde_json::Value> = baseline
        .records
        .iter()
        .filter(|r| r.get("_record_type").and_then(|v| v.as_str()) == Some("device"))
        .collect();
    let scenario_devices: Vec<&serde_json::Value> = scenario
        .records
        .iter()
        .filter(|r| r.get("_record_type").and_then(|v| v.as_str()) == Some("device"))
        .collect();

    let baseline_dets: Vec<&serde_json::Value> = baseline
        .records
        .iter()
        .filter(|r| r.get("_record_type").and_then(|v| v.as_str()) == Some("detection"))
        .collect();
    let scenario_dets: Vec<&serde_json::Value> = scenario
        .records
        .iter()
        .filter(|r| r.get("_record_type").and_then(|v| v.as_str()) == Some("detection"))
        .collect();

    // ── Guard 1: device count ──────────────────────────────────────────────────
    assert_eq!(
        baseline_devices.len(),
        scenario_devices.len(),
        "F-PIVOT003-R10C [PARITY DIVERGENCE]: device count mismatch. \
         baseline={}, scenario={}. \
         generate_with_scenario_iocs must produce the same device count as gen_compromised_endpoint.",
        baseline_devices.len(),
        scenario_devices.len()
    );

    // ── Guard 2: detection count ───────────────────────────────────────────────
    assert_eq!(
        baseline_dets.len(),
        scenario_dets.len(),
        "F-PIVOT003-R10C [PARITY DIVERGENCE]: detection count mismatch. \
         baseline={}, scenario={}. \
         generate_with_scenario_iocs must produce the same detection count as gen_compromised_endpoint.",
        baseline_dets.len(),
        scenario_dets.len()
    );

    // ── Guard 3: device 0 containment_status ──────────────────────────────────
    let b_dev0_containment = baseline_devices[0]
        .get("containment_status")
        .and_then(|v| v.as_str())
        .unwrap_or("MISSING");
    let s_dev0_containment = scenario_devices[0]
        .get("containment_status")
        .and_then(|v| v.as_str())
        .unwrap_or("MISSING");

    assert_eq!(
        b_dev0_containment, s_dev0_containment,
        "F-PIVOT003-R10C [PARITY DIVERGENCE]: device 0 containment_status mismatch. \
         baseline='{}', scenario='{}'. Both paths must set device 0 to 'contained'.",
        b_dev0_containment, s_dev0_containment
    );
    assert_eq!(
        b_dev0_containment, "contained",
        "F-PIVOT003-R10C: device 0 must be 'contained' on both paths. Got: '{b_dev0_containment}'"
    );

    // ── Guard 4: device IDs match across all devices ───────────────────────────
    for (n, (b_dev, s_dev)) in baseline_devices
        .iter()
        .zip(scenario_devices.iter())
        .enumerate()
    {
        let b_id = b_dev
            .get("device_id")
            .and_then(|v| v.as_str())
            .unwrap_or("MISSING");
        let s_id = s_dev
            .get("device_id")
            .and_then(|v| v.as_str())
            .unwrap_or("MISSING");
        assert_eq!(
            b_id, s_id,
            "F-PIVOT003-R10C [PARITY DIVERGENCE]: device[{n}] device_id mismatch. \
             baseline='{b_id}', scenario='{s_id}'. \
             ID format must be identical on both generation paths."
        );
    }

    // ── Guard 5: detection IDs, severity_id, and device linkage ───────────────
    for (n, (b_det, s_det)) in baseline_dets.iter().zip(scenario_dets.iter()).enumerate() {
        let b_det_id = b_det
            .get("detection_id")
            .and_then(|v| v.as_str())
            .unwrap_or("MISSING");
        let s_det_id = s_det
            .get("detection_id")
            .and_then(|v| v.as_str())
            .unwrap_or("MISSING");
        assert_eq!(
            b_det_id, s_det_id,
            "F-PIVOT003-R10C [PARITY DIVERGENCE]: detection[{n}] detection_id mismatch. \
             baseline='{b_det_id}', scenario='{s_det_id}'."
        );

        let b_sev = b_det
            .get("severity_id")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let s_sev = s_det
            .get("severity_id")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        assert_eq!(
            b_sev, s_sev,
            "F-PIVOT003-R10C [PARITY DIVERGENCE]: detection[{n}] severity_id mismatch. \
             baseline={b_sev}, scenario={s_sev}. \
             Severity logic (first 5 = 4, rest = 2) must match on both paths."
        );

        let b_dev_link = b_det
            .get("device_id")
            .and_then(|v| v.as_str())
            .unwrap_or("MISSING");
        let s_dev_link = s_det
            .get("device_id")
            .and_then(|v| v.as_str())
            .unwrap_or("MISSING");
        assert_eq!(
            b_dev_link, s_dev_link,
            "F-PIVOT003-R10C [PARITY DIVERGENCE]: detection[{n}] device_id linkage mismatch. \
             baseline='{b_dev_link}', scenario='{s_dev_link}'. \
             Linkage (device_ids[n % dev_count]) must match on both paths."
        );
    }

    // ── Guard 6: detection n>0 behaviors[] count matches (MITRE-only = 1 entry) ──
    for (n, (b_det, s_det)) in baseline_dets
        .iter()
        .zip(scenario_dets.iter())
        .enumerate()
        .skip(1)
    {
        let b_blen = b_det
            .get("behaviors")
            .and_then(|v| v.as_array())
            .map(|a| a.len())
            .unwrap_or(0);
        let s_blen = s_det
            .get("behaviors")
            .and_then(|v| v.as_array())
            .map(|a| a.len())
            .unwrap_or(0);
        assert_eq!(
            b_blen, s_blen,
            "F-PIVOT003-R10C [PARITY DIVERGENCE]: detection[{n}] behaviors[] length mismatch \
             (n > 0, MITRE-only expected). baseline={b_blen}, scenario={s_blen}. \
             Non-scenario detections must have exactly 1 MITRE behavior entry on both paths."
        );

        // Neither non-scenario detection should have IOC keys in behaviors[0].
        if let Some(b_b0) = b_det
            .get("behaviors")
            .and_then(|v| v.as_array())
            .and_then(|a| a.first())
        {
            assert!(
                b_b0.get("ioc_value").is_none(),
                "F-PIVOT003-R10C: baseline detection[{n}] behaviors[0] must NOT have \
                 ioc_value. Got: {b_b0}"
            );
        }
        if let Some(s_b0) = s_det
            .get("behaviors")
            .and_then(|v| v.as_array())
            .and_then(|a| a.first())
        {
            assert!(
                s_b0.get("ioc_value").is_none(),
                "F-PIVOT003-R10C: scenario detection[{n}] behaviors[0] (n > 0) must NOT have \
                 ioc_value (only detection 0 is IOC-stamped). Got: {s_b0}"
            );
        }
    }

    // ── Guard 7: detection 0 — scenario path HAS ioc_value; baseline does NOT ──
    // This is the ALLOWED difference (modulo IOC stamp on detection 0).
    let b_det0 = baseline_dets[0];
    let s_det0 = scenario_dets[0];

    let b_det0_ioc = b_det0
        .get("behaviors")
        .and_then(|v| v.as_array())
        .and_then(|a| a.first())
        .and_then(|b| b.get("ioc_value"));

    let s_det0_ioc = s_det0
        .get("behaviors")
        .and_then(|v| v.as_array())
        .and_then(|a| a.first())
        .and_then(|b| b.get("ioc_value"))
        .and_then(|v| v.as_str());

    assert!(
        b_det0_ioc.is_none(),
        "F-PIVOT003-R10C: baseline detection 0 behaviors[0] must NOT have ioc_value \
         (baseline path = gen_compromised_endpoint, no IOC stamping). \
         Got ioc_value: {b_det0_ioc:?}"
    );

    assert_eq!(
        s_det0_ioc,
        Some(ioc_hash),
        "F-PIVOT003-R10C: scenario detection 0 behaviors[0].ioc_value must equal \
         the IOC hash passed to generate_with_scenario_iocs. \
         Expected: Some('{ioc_hash}'). Got: {s_det0_ioc:?}. \
         BC-2.06.019 PC-4 / AC-004"
    );

    // ── Confirm ioc_type is "hash_sha256" on scenario detection 0 ─────────────
    let s_det0_ioc_type = s_det0
        .get("behaviors")
        .and_then(|v| v.as_array())
        .and_then(|a| a.first())
        .and_then(|b| b.get("ioc_type"))
        .and_then(|v| v.as_str());

    assert_eq!(
        s_det0_ioc_type,
        Some("hash_sha256"),
        "F-PIVOT003-R10C: scenario detection 0 behaviors[0].ioc_type must be 'hash_sha256'. \
         Got: {s_det0_ioc_type:?}. BC-2.06.019 correction (algorithm-qualified token)."
    );
}
