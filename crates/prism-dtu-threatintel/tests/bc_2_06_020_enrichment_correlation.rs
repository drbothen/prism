//! Red Gate tests 13 and 16: BC-2.06.020 enrichment correlation
//!
//! Tests:
//!   Test 13: test_BC_2_06_020_threatintel_ioc_correlation_all_types
//!            test_BC_2_06_020_ac013_lookup_response_fields (companion — HTTP route verification)
//!   Test 16: test_BC_2_06_020_non_scenario_passthrough_and_perimeter_gate
//!
//! Story: S-DEMO-DTU-LIVE-SCENARIO-001-B
//! Traces to: BC-2.06.020 INV-THREATINTEL-IOC-CORRELATION-001 / PC-1, PC-2
//!            BC-2.06.020 INV-NON-SCENARIO-LOOKUP-PASSTHROUGH-001 + INV-PERIMETER-COMPLIANCE-001 / PC-6
//!            AC-013/PC-2: lookup response must carry threat_is_known_malicious=true, threat_score >= 75

#![cfg(feature = "fixture-gen")]
#![allow(clippy::unwrap_used, clippy::expect_used)]

use prism_dtu_common::{build_scenario_entity_catalog, build_test_client, BehavioralClone, OrgId};
use prism_dtu_threatintel::ThreatIntelClone;

/// Org ID with well-known first 4 bytes [0xde, 0xad, 0xbe, 0xef] → org_slug = "deadbeef".
fn deadbeef_org() -> OrgId {
    OrgId([
        0xde, 0xad, 0xbe, 0xef, 0x00, 0x00, 0x70, 0x00, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00,
    ])
}

// ---------------------------------------------------------------------------
// RED GATE TEST 13 — test_BC_2_06_020_threatintel_ioc_correlation_all_types
//
// BC-2.06.020 INV-THREATINTEL-IOC-CORRELATION-001 / PC-1, PC-2
// Verifies: ThreatIntelClone::new_with_scenario(entities) pre-populates the
// fixture_registry with all IOC IPs, domains, and hashes as FixtureKey::Malicious.
//
// FAIL mode: new_with_scenario stub delegates to new() without inserting IOCs.
// The fixture_registry won't contain any scenario IOC keys.
// Assertions that ioc_ips[0] resolves to FixtureKey::Malicious will FAIL.
// ---------------------------------------------------------------------------

/// RED GATE TEST 13 — test_BC_2_06_020_threatintel_ioc_correlation_all_types
///
/// BC-2.06.020 INV-THREATINTEL-IOC-CORRELATION-001 / PC-1, PC-2
///
/// Given ThreatIntelClone::new_with_scenario(&entities), all IOCs in
/// entities.ioc_ips, entities.ioc_domains, and entities.ioc_hashes must be
/// pre-populated in fixture_registry as FixtureKey::Malicious at construction time.
///
/// FAIL mode: stub calls new() without inserting IOCs into fixture_registry.
/// Lookups for scenario IOC IPs/domains/hashes return None (not Malicious).
#[test]
fn test_BC_2_06_020_threatintel_ioc_correlation_all_types() {
    use prism_dtu_threatintel::types::FixtureKey;

    let org = deadbeef_org();
    let seed: u64 = 77;

    // Build scenario entity catalog — contains the IOCs that must be injected.
    let catalog = build_scenario_entity_catalog(seed, &org);

    // Verify catalog is non-empty for the relevant fields.
    assert!(
        !catalog.ioc_ips.is_empty(),
        "ScenarioEntityCatalog must have non-empty ioc_ips; got empty \
         (secondary RNG derivation issue)"
    );
    assert!(
        !catalog.ioc_domains.is_empty(),
        "ScenarioEntityCatalog must have non-empty ioc_domains"
    );
    assert!(
        !catalog.ioc_hashes.is_empty(),
        "ScenarioEntityCatalog must have non-empty ioc_hashes"
    );

    // Construct ThreatIntelClone via new_with_scenario (infallible).
    // The stub delegates to new() without injecting scenario IOCs.
    let clone = ThreatIntelClone::new_with_scenario(&catalog);

    // Inspect fixture_registry directly (pub field on state) for IOC presence.
    // SAFETY: mutex poison only in pathological cases; tests control this state.
    #[allow(clippy::expect_used)]
    let registry = clone
        .state
        .fixture_registry
        .lock()
        .expect("fixture_registry lock must succeed");

    // Test IOC IP [0]: must be FixtureKey::Malicious.
    // FAIL: stub leaves fixture_registry as default_registry() — scenario IOC not present.
    let ioc_ip = &catalog.ioc_ips[0];
    let ip_fixture = registry.get(ioc_ip);
    assert!(
        ip_fixture.is_some(),
        "fixture_registry must contain scenario IOC IP '{ioc_ip}' after new_with_scenario; \
         got None — stub did not inject scenario IOCs \
         [RED GATE: INV-THREATINTEL-IOC-CORRELATION-001 / BC-2.06.020 PC-1]"
    );
    assert!(
        matches!(ip_fixture, Some(FixtureKey::Malicious)),
        "fixture_registry['{ioc_ip}'] must be FixtureKey::Malicious; got {ip_fixture:?} \
         — BC-2.06.020 PC-2 / AC-013"
    );

    // Test IOC domain [0]: must be FixtureKey::Malicious.
    let ioc_domain = &catalog.ioc_domains[0];
    let domain_fixture = registry.get(ioc_domain);
    assert!(
        domain_fixture.is_some(),
        "fixture_registry must contain scenario IOC domain '{ioc_domain}' after new_with_scenario; \
         got None — stub did not inject scenario IOCs \
         [RED GATE: INV-THREATINTEL-IOC-CORRELATION-001 / BC-2.06.020 PC-1]"
    );
    assert!(
        matches!(domain_fixture, Some(FixtureKey::Malicious)),
        "fixture_registry['{ioc_domain}'] must be FixtureKey::Malicious; got {domain_fixture:?}"
    );

    // Test IOC hash [0]: must be FixtureKey::Malicious.
    let ioc_hash = &catalog.ioc_hashes[0];
    let hash_fixture = registry.get(ioc_hash);
    assert!(
        hash_fixture.is_some(),
        "fixture_registry must contain scenario IOC hash '{ioc_hash}' after new_with_scenario; \
         got None — stub did not inject scenario IOCs \
         [RED GATE: INV-THREATINTEL-IOC-CORRELATION-001 / BC-2.06.020 PC-2]"
    );
    assert!(
        matches!(hash_fixture, Some(FixtureKey::Malicious)),
        "fixture_registry['{ioc_hash}'] must be FixtureKey::Malicious; got {hash_fixture:?}"
    );

    // All IOC IPs must be injected.
    for (i, ip) in catalog.ioc_ips.iter().enumerate() {
        let entry = registry.get(ip);
        assert!(
            matches!(entry, Some(FixtureKey::Malicious)),
            "fixture_registry[ioc_ips[{i}]='{ip}'] must be Malicious; got {entry:?}. \
             BC-2.06.020 INV-THREATINTEL-IOC-CORRELATION-001"
        );
    }

    // All IOC domains must be injected.
    for (i, domain) in catalog.ioc_domains.iter().enumerate() {
        let entry = registry.get(domain);
        assert!(
            matches!(entry, Some(FixtureKey::Malicious)),
            "fixture_registry[ioc_domains[{i}]='{domain}'] must be Malicious; got {entry:?}"
        );
    }

    // All IOC hashes must be injected.
    for (i, hash) in catalog.ioc_hashes.iter().enumerate() {
        let entry = registry.get(hash);
        assert!(
            matches!(entry, Some(FixtureKey::Malicious)),
            "fixture_registry[ioc_hashes[{i}]='{hash}'] must be Malicious; got {entry:?}"
        );
    }
}

// ---------------------------------------------------------------------------
// RED GATE TEST 16 — test_BC_2_06_020_non_scenario_passthrough_and_perimeter_gate
//
// BC-2.06.020 INV-NON-SCENARIO-LOOKUP-PASSTHROUGH-001 + INV-PERIMETER-COMPLIANCE-001 / PC-6
// Verifies: a non-scenario IP "192.0.2.1" (not in ioc_ips) resolves identically
// whether looked up via new_with_scenario or new() — scenario injection is additive.
//
// FAIL mode: This test should mostly PASS (the passthrough is preserved by new_with_scenario
// since it delegates to new() which sets up the default registry). HOWEVER, the test
// also verifies that after new_with_scenario, the scenario IOC IS present (asserting
// both the passthrough AND the injection), relying on test 13 assertions.
//
// If test 13 fails (IOC not injected), this test documents that passthrough works but
// injection doesn't — not strictly a Red Gate failure for the passthrough assertion.
//
// To make this a clear Red Gate test, we assert that the scenario IOC IS present
// (as Malicious) AND the non-scenario IP is also present (as default lookup),
// confirming both assertions hold simultaneously.
//
// Perimeter gate: prism-dtu-threatintel must NOT import from prism-spec-engine,
// prism-sensors, or prism-query (INV-PERIMETER-COMPLIANCE-001). Compliance is
// enforced STRUCTURALLY: prism-dtu-threatintel's Cargo.toml declares no dependency
// on those crates, so any forbidden `use` is an ordinary E0432 compile error caught
// by the standard build. (The `tests/external/perimeter-violation/` compile-fail
// crate covers the prism-query pub-API perimeter only — BC-2.11.006 — and does
// not reference the DTU crates.)
// ---------------------------------------------------------------------------

/// RED GATE TEST 16 — test_BC_2_06_020_non_scenario_passthrough_and_perimeter_gate
///
/// BC-2.06.020 INV-NON-SCENARIO-LOOKUP-PASSTHROUGH-001 / PC-6
///
/// Verifies that after new_with_scenario, non-scenario lookups (IP "192.0.2.1"
/// not in ioc_ips) behave identically to new() — scenario injection is strictly additive.
///
/// FAIL mode: The passthrough assertion PASSES (both new() and new_with_scenario use
/// the default registry for non-scenario IPs). The RED GATE failure comes from the
/// scenario IOC assertion which fails because the stub doesn't inject IOCs.
///
/// Perimeter gate: prism-dtu-threatintel must NOT import from prism-spec-engine,
/// prism-sensors, or prism-query (INV-PERIMETER-COMPLIANCE-001). Compliance is
/// enforced STRUCTURALLY: prism-dtu-threatintel's Cargo.toml declares no dependency
/// on those crates, so any forbidden `use` is an ordinary E0432 compile error caught
/// by the standard build. (The `tests/external/perimeter-violation/` compile-fail
/// crate covers the prism-query pub-API perimeter only — BC-2.11.006 — and does
/// not reference the DTU crates.)
/// prism-core is on the allow-list (explicit note in BC-2.06.020 INV-PERIMETER-COMPLIANCE-001).
#[test]
fn test_BC_2_06_020_non_scenario_passthrough_and_perimeter_gate() {
    use prism_dtu_threatintel::types::FixtureKey;

    let org = deadbeef_org();
    let seed: u64 = 88;

    let catalog = build_scenario_entity_catalog(seed, &org);

    // A non-scenario IP: "192.0.2.1" — this IP is NOT in catalog.ioc_ips.
    // It should NOT be in the fixture_registry either (not a known fixture IP).
    let non_scenario_ip = "192.0.2.1";
    assert!(
        !catalog.ioc_ips.contains(&non_scenario_ip.to_string()),
        "Test invariant: '192.0.2.1' must not be in catalog.ioc_ips for this test to be valid; \
         got {:?}",
        catalog.ioc_ips
    );

    // Build two clones: one via new() and one via new_with_scenario.
    let baseline_clone = ThreatIntelClone::new();
    let scenario_clone = ThreatIntelClone::new_with_scenario(&catalog);

    // Lock both registries for inspection.
    #[allow(clippy::expect_used)]
    let baseline_registry = baseline_clone
        .state
        .fixture_registry
        .lock()
        .expect("baseline registry lock must succeed");
    #[allow(clippy::expect_used)]
    let scenario_registry = scenario_clone
        .state
        .fixture_registry
        .lock()
        .expect("scenario registry lock must succeed");

    // Non-scenario IP "192.0.2.1" must behave identically in both registries.
    // "192.0.2.1" is not in the default_registry, so both should return None.
    let baseline_192 = baseline_registry.get(non_scenario_ip);
    let scenario_192 = scenario_registry.get(non_scenario_ip);

    assert_eq!(
        baseline_192.is_some(),
        scenario_192.is_some(),
        "INV-NON-SCENARIO-LOOKUP-PASSTHROUGH-001: non-scenario IP '192.0.2.1' must have \
         identical presence in baseline registry ({:?}) and scenario registry ({:?}). \
         BC-2.06.020 PC-6",
        baseline_192,
        scenario_192
    );

    // The known-malicious IP "45.55.100.1" (from default_registry in ThreatIntelState)
    // must still be Malicious in the scenario clone (passthrough preserved).
    let known_malicious_ip = "45.55.100.1";
    let scenario_known = scenario_registry.get(known_malicious_ip);
    assert!(
        matches!(scenario_known, Some(FixtureKey::Malicious)),
        "INV-NON-SCENARIO-LOOKUP-PASSTHROUGH-001: known-malicious IP '45.55.100.1' must \
         remain Malicious in scenario clone registry (scenario injection is additive, \
         not replacement); got {:?}. BC-2.06.020 PC-6",
        scenario_known
    );

    // Scenario IOC IP must be Malicious in scenario registry (but not in baseline).
    // FAIL: stub doesn't inject IOCs → scenario IOC not in registry.
    let ioc_ip = &catalog.ioc_ips[0];
    let scenario_ioc = scenario_registry.get(ioc_ip.as_str());
    let baseline_ioc = baseline_registry.get(ioc_ip.as_str());

    assert!(
        baseline_ioc.is_none(),
        "INV-NON-SCENARIO-LOOKUP-PASSTHROUGH-001: scenario IOC '{ioc_ip}' must NOT \
         be in baseline registry (proves it is not a default fixture entry); \
         got {:?}",
        baseline_ioc
    );

    // FAIL: stub doesn't inject IOC into scenario registry.
    assert!(
        matches!(scenario_ioc, Some(FixtureKey::Malicious)),
        "INV-NON-SCENARIO-LOOKUP-PASSTHROUGH-001: scenario IOC '{ioc_ip}' must be \
         Malicious in scenario clone registry; got {:?}. \
         BC-2.06.020 INV-NON-SCENARIO-LOOKUP-PASSTHROUGH-001 / PC-6 \
         [RED GATE: stub does not inject scenario IOCs]",
        scenario_ioc
    );

    // Perimeter compliance note:
    // prism-dtu-threatintel must NOT import from prism-spec-engine, prism-sensors,
    // or prism-query (INV-PERIMETER-COMPLIANCE-001). new_with_scenario constructor
    // in clone.rs uses only prism-dtu-common types (ScenarioEntityCatalog, FixtureKey).
    // Perimeter compliance (INV-PERIMETER-COMPLIANCE-001) is enforced STRUCTURALLY:
    // prism-dtu-threatintel's Cargo.toml declares no dependency on prism-spec-engine,
    // prism-sensors, or prism-query, so any forbidden `use` of those crates is an
    // ordinary E0432 compile error caught by the standard build.
    // (The `tests/external/perimeter-violation/` compile-fail crate covers the
    // prism-query pub-API perimeter only — BC-2.11.006 — and does not reference
    // the DTU crates.)
}

// ---------------------------------------------------------------------------
// COMPANION TEST for AC-013/PC-2 — HTTP route-level response field verification
//
// BC-2.06.020 AC-013/PC-2: a lookup response for catalog.ioc_ips[0] issued via
// the HTTP route (GET /v3/ip/:ip) must carry threat_is_known_malicious=true AND
// threat_score >= 75.
//
// This companion test starts an actual ThreatIntelClone server (new_with_scenario),
// issues an HTTP GET for ioc_ips[0], and asserts the AC-013/PC-2 response fields.
// It keeps the registry-presence assertions from test 13 separate (construction-time
// INV evidence) and adds direct end-to-end HTTP evidence here.
//
// Deterministic: ioc_ips are RNG-derived from (seed=77, deadbeef_org) and are always
// the same set; none will collide with benign or unknown fixture entries.
// ---------------------------------------------------------------------------

/// Companion to Test 13 — AC-013/PC-2 HTTP route lookup response fields.
///
/// BC-2.06.020 AC-013/PC-2: GET /v3/ip/{ioc_ips[0]} via new_with_scenario clone
/// must return HTTP 200 with threat_is_known_malicious=true and threat_score >= 75.
#[tokio::test]
async fn test_BC_2_06_020_ac013_lookup_response_fields() {
    let org = deadbeef_org();
    let seed: u64 = 77;

    let catalog = build_scenario_entity_catalog(seed, &org);

    assert!(
        !catalog.ioc_ips.is_empty(),
        "catalog.ioc_ips must be non-empty for this test to be valid"
    );
    let ioc_ip = &catalog.ioc_ips[0];

    // Start a real server via new_with_scenario.
    let mut clone = ThreatIntelClone::new_with_scenario(&catalog);
    clone
        .start()
        .await
        .expect("AC-013: ThreatIntelClone::start() must succeed");

    let base = clone.base_url();
    let client = build_test_client();

    // Issue HTTP GET /v3/ip/{ioc_ip}?key=test-key
    let resp = client
        .get(format!("{base}/v3/ip/{ioc_ip}"))
        .query(&[("key", "test-key-valid")])
        .send()
        .await
        .expect("AC-013: HTTP request to ThreatIntelClone must succeed");

    assert_eq!(
        resp.status().as_u16(),
        200,
        "AC-013: scenario IOC IP lookup must return HTTP 200; got {}",
        resp.status()
    );

    let body: serde_json::Value = resp
        .json()
        .await
        .expect("AC-013: response must be valid JSON");

    // AC-013/PC-2: threat_is_known_malicious must be true.
    let is_malicious = body
        .get("threat_is_known_malicious")
        .and_then(|v| v.as_bool())
        .expect("AC-013: response must contain 'threat_is_known_malicious' field");
    assert!(
        is_malicious,
        "AC-013/PC-2: threat_is_known_malicious must be true for scenario IOC IP '{ioc_ip}'; \
         BC-2.06.020 PC-2"
    );

    // AC-013/PC-2: threat_score must be >= 75.
    let threat_score = body
        .get("threat_score")
        .and_then(|v| v.as_u64())
        .expect("AC-013: response must contain 'threat_score' field");
    assert!(
        threat_score >= 75,
        "AC-013/PC-2: threat_score must be >= 75 for scenario IOC IP '{ioc_ip}'; \
         got {threat_score} — BC-2.06.020 PC-2"
    );

    clone
        .stop()
        .await
        .expect("AC-013: clone.stop() must succeed");
}
