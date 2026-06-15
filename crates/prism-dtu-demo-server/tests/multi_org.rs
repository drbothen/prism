//! Red Gate tests for S-DEMO-LAUNCHER-CONSOLIDATION-001 — `demo-server start-multi`.
//!
//! # Purpose
//!
//! These 5 tests define the behavioral contract for the `start-multi` subcommand and its
//! supporting types. All 5 tests MUST FAIL before implementation begins (Red Gate).
//! The implementer's task is to make each test pass, one at a time, with minimum code.
//!
//! # Red Gate tests
//!
//! | Test | BC traces | Red reason |
//! |------|-----------|------------|
//! | RG-001 `test_multi_org_config_parses_valid_three_org_toml` | BC-2.06.001 PC-1 | `from_str` is `todo!()` |
//! | RG-002 `test_multi_org_config_rejects_unknown_fields` | BC-2.06.001 INV | `from_str` is `todo!()` |
//! | RG-003 `test_nested_sidecar_format_has_correct_structure` | BC-2.06.017 PC-1 | pure serde shape test; passes at Red Gate (see note) |
//! | RG-004 `test_clone_factory_dispatch_returns_clone_for_each_sensor` | BC-2.06.017 PC-1 | `build_multi_clone_factory` is `todo!()` |
//! | RG-005 `test_start_multi_stands_up_per_org_distinct_sockets` | BC-2.06.017 PC-3 | `start_multi_for_config` is `todo!()` |
//!
//! # Note on RG-003
//!
//! RG-003 is a pure serde round-trip test on stdlib types (`HashMap<String, HashMap<String, String>>`).
//! It validates the SHAPE of the nested sidecar format, not the `write_multi_url_sidecar` function.
//! It passes at Red Gate because it exercises no stub code — it verifies the data contract
//! that the implementer must satisfy when writing the real `write_multi_url_sidecar`.
//! This is intentional and documents the expected JSON shape for demo-run.sh.
//!
//! # Feature gate
//!
//! All tests require `dtu` + `fixture-gen` features (Cargo.toml `required-features`).
//! RG-004 and RG-005 call `build_multi_clone_factory` / `start_multi_for_config`
//! which are `#[cfg(feature = "fixture-gen")]`-gated. Without `fixture-gen`, the
//! `#[cfg(not(feature = "fixture-gen"))]` stub panics (hard error, not silent
//! fallback to unseeded `new()`) — GAP-1 enforcement.
//!
//! # Story anchor
//!
//! S-DEMO-LAUNCHER-CONSOLIDATION-001 v2.1 (BC-2.06.001, BC-2.06.017)

#![cfg(all(feature = "dtu", feature = "fixture-gen"))]
#![allow(clippy::unwrap_used, clippy::expect_used, non_snake_case)]

use prism_dtu_demo_server::{multi_instance::InstanceEntry, MultiOrgDemoConfig};

// Pull in the testable stub functions from main.rs via the crate's test-binary path.
// These are `pub(crate)` functions that integration tests access through the binary.
//
// Note: `build_multi_clone_factory` and `start_multi_for_config` live in `main.rs` as
// `pub(crate)`. Integration tests in `tests/` are compiled as separate crates and
// cannot access `pub(crate)` items from `main.rs`. They ARE accessible when the test
// links against the library; however these functions are in the binary target, not the
// library. We work around this by re-declaring test-visible wrappers below that call
// through to the functions under the same feature gate.
//
// ARCHITECTURE NOTE: The story spec requires `build_multi_clone_factory` and
// `start_multi_for_config` to be "extracted testable functions". For integration tests
// to call them directly, they must be re-exported through the library crate
// (`prism_dtu_demo_server`) rather than being `pub(crate)` in `main.rs`. The
// implementer must move them to `lib.rs` re-exports or a `pub mod multi_org` module.
// The tests below assume they are accessible as:
//   - `prism_dtu_demo_server::build_multi_clone_factory`
//   - `prism_dtu_demo_server::start_multi_for_config`
//
// Until the implementer adds those re-exports, RG-004 and RG-005 will fail to COMPILE
// (not just fail at runtime). This is an acceptable Red Gate failure mode — the test
// cannot compile because the surface it tests doesn't exist yet.

// ---------------------------------------------------------------------------
// RG-001: MultiOrgDemoConfig parses a valid 3-org TOML
//
// Traces to: BC-2.06.001 Postcondition 1 (config must parse and deserialize correctly)
// Red reason: `MultiOrgDemoConfig::from_str` is `todo!()` — panics with "not yet implemented"
// ---------------------------------------------------------------------------

/// BC-2.06.001 PC-1: A valid 3-org multi-org config TOML must parse without error.
///
/// Tests the canonical 3-org demo config matching the S-DEMO-004 seed assignments:
/// - org-a: seed=100 (CrowdStrike + Armis)
/// - org-b: seed=150 (Claroty + Cyberint, with initial_access_token)
/// - org-c: seed=200 (all 4 sensors)
#[test]
fn test_multi_org_config_parses_valid_three_org_toml() {
    let toml = r#"
        [harness]
        bind = "127.0.0.1"

        [orgs.org-a]
        org_id = "0196f4b2-3c8d-7e1a-b5f0-2d4c6e8a0000"
        sensors = ["crowdstrike", "armis"]
        seed = 100

        [orgs.org-b]
        org_id = "0196f4b2-3c8d-7e1a-b5f0-2d4c6e8a0001"
        sensors = ["claroty", "cyberint"]
        seed = 150
        initial_access_token = "demo-cyberint-token"

        [orgs.org-c]
        org_id = "0196f4b2-3c8d-7e1a-b5f0-2d4c6e8a0002"
        sensors = ["crowdstrike", "armis", "claroty", "cyberint"]
        seed = 200
    "#;
    let cfg = MultiOrgDemoConfig::from_str(toml).expect("valid 3-org config must parse");
    assert_eq!(cfg.orgs.len(), 3);
    assert_eq!(cfg.orgs["org-a"].sensors, ["crowdstrike", "armis"]);
    assert_eq!(cfg.orgs["org-a"].seed, 100);
    assert_eq!(
        cfg.orgs["org-b"].initial_access_token.as_deref(),
        Some("demo-cyberint-token")
    );
    assert_eq!(cfg.orgs["org-c"].sensors.len(), 4);
}

// ---------------------------------------------------------------------------
// RG-002: MultiOrgDemoConfig rejects unknown fields at every level
//
// Traces to: BC-2.06.001 Invariant (config schema must be strict; unknown fields → error)
// Red reason: `MultiOrgDemoConfig::from_str` is `todo!()` — panics with "not yet implemented"
// ---------------------------------------------------------------------------

/// BC-2.06.001 INV: Unknown fields at any level must be rejected by `deny_unknown_fields`.
///
/// Covers: top-level unknown key, `[orgs.X]` typo (`seed` → `seeds`), `[harness]` typo.
/// A typo'd key silently ignored means the demo runs with defaults the operator
/// believes they overrode — this is a runtime defect that deny_unknown_fields prevents.
#[test]
fn test_multi_org_config_rejects_unknown_fields() {
    let cases: &[(&str, &str)] = &[
        ("unknown top-level key", "unknown_field = true\n"),
        (
            "[orgs.org-a] unknown key (typo 'seed' → 'seeds')",
            "[orgs.org-a]\norg_id = \"00000000-0000-0000-0000-000000000000\"\nseeds = 99\nsensors = []\nseed = 0\n",
        ),
        (
            "[orgs.org-a] only the typo (should fail)",
            "[orgs.org-a]\norg_id = \"00000000-0000-0000-0000-000000000000\"\nseeds = 99\nsensors = []\n",
        ),
        (
            "[harness] typo (typo 'bind' → 'bnd')",
            "[harness]\nbnd = \"127.0.0.1\"\n",
        ),
    ];
    for (label, toml) in cases {
        assert!(
            MultiOrgDemoConfig::from_str(toml).is_err(),
            "unknown field at {label} must be rejected by deny_unknown_fields, \
             but parsed without error: {toml:?}"
        );
    }
}

// ---------------------------------------------------------------------------
// RG-003: Nested sidecar serialization shape
//
// Traces to: BC-2.06.017 Postcondition 1 (multi-instance socket map must be accessible
//            to downstream tools; the nested format encodes per-org URL routing)
// Red status: PASSES at Red Gate (pure serde shape test — no stub code exercised).
//             Intentional: documents the expected JSON contract for demo-run.sh.
//             The real `write_multi_url_sidecar` must produce this exact shape.
// ---------------------------------------------------------------------------

/// BC-2.06.017 PC-1: The nested sidecar must encode `{org_slug: {sensor_id: url}}`.
///
/// Verifies the JSON data contract that `write_multi_url_sidecar` must satisfy.
/// This test passes at Red Gate because it exercises no stub code — it proves the
/// shape is correct before the implementer writes `write_multi_url_sidecar`.
///
/// The sidecar is distinct from the flat `{name: url}` format written by `start`:
/// the inner values are JSON objects (not strings), enabling per-org URL routing.
#[test]
fn test_nested_sidecar_format_has_correct_structure() {
    use std::collections::HashMap;

    let mut sensor_map_a: HashMap<String, String> = HashMap::new();
    sensor_map_a.insert(
        "crowdstrike".to_string(),
        "http://127.0.0.1:54321".to_string(),
    );
    sensor_map_a.insert("armis".to_string(), "http://127.0.0.1:54322".to_string());
    let mut nested: HashMap<String, HashMap<String, String>> = HashMap::new();
    nested.insert("org-a".to_string(), sensor_map_a);

    let json = serde_json::to_string(&nested).expect("must serialize");
    let parsed: HashMap<String, HashMap<String, String>> =
        serde_json::from_str(&json).expect("must round-trip");
    assert_eq!(parsed["org-a"]["crowdstrike"], "http://127.0.0.1:54321");
    assert_eq!(parsed["org-a"]["armis"], "http://127.0.0.1:54322");

    // Verify that the inner values are JSON objects (nested format), NOT strings (flat format).
    let raw: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert!(
        raw["org-a"].is_object(),
        "inner value must be a JSON object (nested format) — got: {:?}",
        raw["org-a"]
    );
    assert!(
        !raw["org-a"].is_string(),
        "inner value must NOT be a plain string (that would be the flat sidecar format)"
    );
}

// ---------------------------------------------------------------------------
// RG-004: clone_factory dispatches (org_slug, sensor_id) → Box<dyn BehavioralClone>
//
// Traces to: BC-2.06.017 Postcondition 1 (each InstanceEntry must produce a running clone)
// Red reason: `build_multi_clone_factory` is `todo!()` — panics with "not yet implemented"
//
// NOTE: This test references `prism_dtu_demo_server::build_multi_clone_factory` which
// does not yet exist in the library's public API. Until the implementer adds this
// re-export, this test will FAIL TO COMPILE — which counts as a Red Gate failure.
// The implementer must add `pub use` (or move the fn) to make the function accessible
// from the library crate.
// ---------------------------------------------------------------------------

/// BC-2.06.017 PC-1: The clone_factory closure must dispatch `(org_slug, sensor_id)`
/// to the correct seeded clone constructor for each of the 4 supported sensors.
///
/// RG-004 validates that `build_multi_clone_factory` produces valid clones for
/// CrowdStrike and Armis entries from org-a (seed=42, fixture-gen required).
///
/// The `#[tokio::test]` annotation is used because the story spec uses it, but
/// note that `build_multi_clone_factory` itself is synchronous — the async test
/// executor matches the signature used by RG-005 for consistency.
#[tokio::test]
async fn test_clone_factory_dispatch_returns_clone_for_each_sensor() {
    let toml = r#"
        [orgs.org-a]
        org_id = "0196f4b2-3c8d-7e1a-b5f0-2d4c6e8a0000"
        sensors = ["crowdstrike", "armis"]
        seed = 42
    "#;
    let cfg = MultiOrgDemoConfig::from_str(toml).expect("must parse");

    // build_multi_clone_factory must be pub-accessible from the library crate.
    // Implementer: add `pub use` to lib.rs or move the function to a pub module.
    let factory = prism_dtu_demo_server::build_multi_clone_factory(&cfg);

    // "org-a-crowdstrike" entry → must produce a valid clone (not panic, not None)
    let entry_cs = InstanceEntry::new("org-a-crowdstrike", "127.0.0.1:0".parse().unwrap());
    let _clone_cs: Box<dyn prism_dtu_common::BehavioralClone> = factory(&entry_cs);

    // "org-a-armis" entry → must produce a valid clone (not panic)
    let entry_armis = InstanceEntry::new("org-a-armis", "127.0.0.1:0".parse().unwrap());
    let _clone_armis: Box<dyn prism_dtu_common::BehavioralClone> = factory(&entry_armis);
}

// ---------------------------------------------------------------------------
// RG-005: start_multi binds per-org clones to distinct socket addresses
//
// Traces to: BC-2.06.017 Postcondition 3 (distinct per-org socket addresses for same sensor)
//            AC-004 / INV-DISTINCT-DATA-001
// Red reason: `start_multi_for_config` is `todo!()` — panics with "not yet implemented"
//
// NOTE: Same re-export requirement as RG-004 — `prism_dtu_demo_server::start_multi_for_config`
// must be pub-accessible from the library. This test will FAIL TO COMPILE until the
// implementer adds the re-export. That compile failure is the Red Gate.
// ---------------------------------------------------------------------------

/// BC-2.06.017 PC-3: org-a CrowdStrike and org-c CrowdStrike must bind to DIFFERENT ports.
///
/// Integration-level socket isolation test. Uses port=0 (OS-assigned ephemeral) for both
/// org-a and org-c CrowdStrike instances. Asserts:
/// 1. Both ports are non-zero (actually bound to a real OS port).
/// 2. The two ports are distinct (per-org socket isolation per BC-2.06.017).
///
/// The `fixture-gen` feature is required (Cargo.toml `required-features`). Without it,
/// `build_multi_clone_factory` (called internally by `start_multi_for_config`) would
/// panic with the GAP-1 guard message rather than silently serving identical data.
#[tokio::test]
async fn test_start_multi_stands_up_per_org_distinct_sockets() {
    let toml = r#"
        [harness]
        bind = "127.0.0.1"

        [orgs.org-a]
        org_id = "0196f4b2-3c8d-7e1a-b5f0-2d4c6e8a0000"
        sensors = ["crowdstrike"]
        seed = 100

        [orgs.org-c]
        org_id = "0196f4b2-3c8d-7e1a-b5f0-2d4c6e8a0002"
        sensors = ["crowdstrike"]
        seed = 200
    "#;
    let cfg = MultiOrgDemoConfig::from_str(toml).expect("must parse");

    // start_multi_for_config must be pub-accessible from the library crate.
    // Implementer: add `pub use` to lib.rs or move the function to a pub module.
    let servers = prism_dtu_demo_server::start_multi_for_config(&cfg)
        .await
        .expect("must bind both org-a and org-c CrowdStrike clones");

    let socket_map = servers.socket_map();
    let org_a_port = socket_map["org-a-crowdstrike"].port();
    let org_c_port = socket_map["org-c-crowdstrike"].port();

    assert_ne!(
        org_a_port, org_c_port,
        "org-a and org-c CrowdStrike clones must bind to distinct ports (BC-2.06.017 PC-3); \
         found both at port {org_a_port}"
    );
    assert_ne!(
        org_a_port, 0,
        "org-a CrowdStrike must be bound to a real OS port (not port 0)"
    );
    assert_ne!(
        org_c_port, 0,
        "org-c CrowdStrike must be bound to a real OS port (not port 0)"
    );

    // Graceful shutdown (no zombie instances).
    servers.shutdown();
}

// ---------------------------------------------------------------------------
// CRIT-1 load-bearing: scripts/demo.toml parses as MultiOrgDemoConfig
//
// This test FAILED before the CRIT-1 fix (scripts/demo.toml contained [clones.*]
// sections rejected by deny_unknown_fields on MultiOrgDemoConfig).
// It PASSES after: scripts/demo.toml is orgs-only ([harness] + [orgs.*] only).
//
// Traces to: CRIT-1 (adversary pass-1) + BC-2.06.001 PC-1
// ---------------------------------------------------------------------------

/// CRIT-1: The actual scripts/demo.toml must parse as MultiOrgDemoConfig.
///
/// Before the fix, MultiOrgDemoConfig::from_file("scripts/demo.toml") failed
/// with a deny_unknown_fields error because the file contained [clones.*] sections
/// that are not recognised by MultiOrgDemoConfig (which only accepts [harness] and
/// [orgs.*]). The fix removes [clones.*] from scripts/demo.toml.
///
/// Asserts:
/// 1. The file exists and parses without error.
/// 2. Exactly 3 orgs are configured (org-a, org-b, org-c).
/// 3. org-a has sensors = ["crowdstrike", "armis"] and seed = 100.
/// 4. org-b has initial_access_token set (Cyberint token).
/// 5. org-c has all 4 sensors and initial_access_token set.
#[test]
fn test_scripts_demo_toml_parses_as_multi_org_config() {
    // Resolve scripts/demo.toml from CARGO_MANIFEST_DIR (crates/prism-dtu-demo-server/)
    // up to the workspace root, then into scripts/.
    let demo_toml = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..") // → crates/
        .join("..") // → workspace root
        .join("scripts")
        .join("demo.toml");

    let demo_toml = demo_toml.canonicalize().unwrap_or_else(|e| {
        panic!(
            "CRIT-1: scripts/demo.toml not found at expected path {:?}: {}. \
             Ensure scripts/demo.toml exists at the workspace root.",
            demo_toml, e
        )
    });

    let cfg = MultiOrgDemoConfig::from_file(&demo_toml).unwrap_or_else(|e| {
        panic!(
            "CRIT-1: MultiOrgDemoConfig::from_file(\"scripts/demo.toml\") failed: {}. \
             This was caused by [clones.*] sections in the file being rejected by \
             deny_unknown_fields. Fix: scripts/demo.toml must be orgs-only.",
            e
        )
    });

    // Assert 3 orgs are present.
    assert_eq!(
        cfg.orgs.len(),
        3,
        "CRIT-1: scripts/demo.toml must configure exactly 3 orgs (org-a, org-b, org-c)"
    );

    // Assert org-a has expected sensors and seed.
    let org_a = cfg.orgs.get("org-a").expect("org-a must be present");
    let mut org_a_sensors = org_a.sensors.clone();
    org_a_sensors.sort();
    assert_eq!(
        org_a_sensors,
        ["armis", "crowdstrike"],
        "org-a must have sensors [crowdstrike, armis]"
    );
    assert_eq!(
        org_a.seed, 100,
        "org-a seed must be 100 (INV-DISTINCT-DATA-001)"
    );
    assert!(
        org_a.initial_access_token.is_none(),
        "org-a must not have initial_access_token (no Cyberint sensor)"
    );

    // Assert org-b has Cyberint token.
    let org_b = cfg.orgs.get("org-b").expect("org-b must be present");
    assert!(
        org_b.initial_access_token.is_some(),
        "org-b must have initial_access_token (Cyberint sensor)"
    );
    assert_eq!(
        org_b.seed, 150,
        "org-b seed must be 150 (INV-DISTINCT-DATA-001)"
    );

    // Assert org-c has all 4 sensors and Cyberint token.
    let org_c = cfg.orgs.get("org-c").expect("org-c must be present");
    assert_eq!(
        org_c.sensors.len(),
        4,
        "org-c must have all 4 sensors (crowdstrike, armis, claroty, cyberint)"
    );
    assert!(
        org_c.initial_access_token.is_some(),
        "org-c must have initial_access_token (Cyberint sensor)"
    );
    assert_eq!(
        org_c.seed, 200,
        "org-c seed must be 200 (INV-DISTINCT-DATA-001)"
    );
}

// ---------------------------------------------------------------------------
// CRIT-2 load-bearing: start_multi with Cyberint token does NOT panic
//
// Before the fix, the factory closure called
// `tokio::runtime::Handle::current().block_on(clone.configure(...))` inside a
// synchronous closure invoked on a tokio worker thread → panic: "Cannot start a
// runtime from within a runtime". The fix calls clone.state.apply_config(...) directly
// (synchronous path) before boxing the clone.
//
// Traces to: CRIT-2 (adversary pass-1) + BC-2.06.017 PC-1 (GAP-2)
// ---------------------------------------------------------------------------

/// CRIT-2: start_multi_for_config with Cyberint + initial_access_token must not panic.
///
/// Before the fix, block_on inside the factory closure panicked when invoked from a
/// tokio worker thread. The fix uses the synchronous CyberintState::apply_config path.
///
/// Asserts:
/// 1. start_multi_for_config returns Ok (no panic during clone construction).
/// 2. The Cyberint clone accepts HTTP requests authenticated with the seeded token
///    (token is in the clone's allowlist → GET /api/v1/alerts returns 200, not 401).
/// 3. An invalid token returns 401 (allowlist not poisoned).
#[tokio::test]
async fn test_start_multi_cyberint_token_seeded_no_panic() {
    const TEST_TOKEN: &str = "crit-2-test-access-token-abc123";

    let toml = r#"
        [harness]
        bind = "127.0.0.1"

        [orgs.org-b]
        org_id = "0196f4b2-3c8d-7e1a-b5f0-2d4c6e8a0001"
        sensors = ["cyberint"]
        seed = 150
        initial_access_token = "crit-2-test-access-token-abc123"
    "#;
    let cfg = MultiOrgDemoConfig::from_str(toml).expect("must parse");

    // This must not panic. Before CRIT-2 fix it panicked with:
    // "Cannot start a runtime from within a runtime"
    let servers = prism_dtu_demo_server::start_multi_for_config(&cfg)
        .await
        .expect(
            "CRIT-2: start_multi_for_config with Cyberint initial_access_token must not panic \
                 or return Err. Before the fix, block_on inside the factory closure panicked \
                 on the tokio worker thread.",
        );

    let socket_map = servers.socket_map();
    let cyberint_addr = socket_map
        .get("org-b-cyberint")
        .expect("org-b-cyberint must be in socket_map");

    // Verify the token is in the Cyberint clone's allowlist by making an authenticated
    // HTTP request. If the token was NOT seeded, the response would be 401.
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .expect("reqwest client must build");

    let alerts_url = format!("http://{cyberint_addr}/api/v1/alerts");

    // Case 1: seeded token → expect 200 (token is in allowlist)
    let resp = client
        .get(&alerts_url)
        .header("Cookie", format!("access_token={TEST_TOKEN}"))
        .send()
        .await
        .expect("GET /api/v1/alerts with seeded token must not network-error");

    assert_eq!(
        resp.status().as_u16(),
        200,
        "CRIT-2: Cyberint clone must accept the seeded initial_access_token. \
         Got HTTP {} instead of 200. The token was not registered in the allowlist — \
         the GAP-2 composite path (apply_config after new_with_seed) is broken.",
        resp.status().as_u16()
    );

    // Case 2: wrong token → expect 401 (allowlist is not poisoned / not disabled)
    let resp_bad = client
        .get(&alerts_url)
        .header("Cookie", "access_token=wrong-token-should-fail")
        .send()
        .await
        .expect("GET /api/v1/alerts with wrong token must not network-error");

    assert_eq!(
        resp_bad.status().as_u16(),
        401,
        "CRIT-2 sanity: Cyberint clone must reject an invalid token (401). \
         Got HTTP {} — the allowlist validation is broken.",
        resp_bad.status().as_u16()
    );

    // Graceful shutdown.
    servers.shutdown();
}

// ---------------------------------------------------------------------------
// MED-1 load-bearing: org-a vs org-c CrowdStrike serve DISTINCT response bodies
//
// INV-DISTINCT-DATA-001 — per-org seeds (100 vs 200) must produce different JSON bodies.
// Before this test existed, the z24 hollow-feature class applied: per-org sockets were
// distinct (RG-005) but nothing verified the BODIES served actually differed.
//
// Traces to: INV-DISTINCT-DATA-001 (seed-based data distinctness per org)
//            BC-2.06.017 PC-1 (each org's clone serves its seeded fixture)
// ---------------------------------------------------------------------------

/// INV-DISTINCT-DATA-001: org-a CrowdStrike (seed=100) and org-c CrowdStrike (seed=200)
/// must serve DISTINCT response bodies.
///
/// This test closes the z24 hollow-feature gap: RG-005 proved per-org sockets are distinct
/// but never verified the CONTENT served by those sockets also differs.
///
/// Asserts:
/// 1. Both clones return HTTP 200 on the CrowdStrike detections endpoint.
/// 2. The response body from org-a differs from the response body from org-c.
///    (Different seeds → different generated fixture data.)
#[tokio::test]
async fn test_org_a_vs_org_c_crowdstrike_serve_distinct_data() {
    let toml = r#"
        [harness]
        bind = "127.0.0.1"

        [orgs.org-a]
        org_id = "0196f4b2-3c8d-7e1a-b5f0-2d4c6e8a0000"
        sensors = ["crowdstrike"]
        seed = 100

        [orgs.org-c]
        org_id = "0196f4b2-3c8d-7e1a-b5f0-2d4c6e8a0002"
        sensors = ["crowdstrike"]
        seed = 200
    "#;
    let cfg = MultiOrgDemoConfig::from_str(toml).expect("must parse");
    let servers = prism_dtu_demo_server::start_multi_for_config(&cfg)
        .await
        .expect("must start org-a and org-c CrowdStrike clones");

    let socket_map = servers.socket_map();
    let org_a_addr = socket_map["org-a-crowdstrike"];
    let org_c_addr = socket_map["org-c-crowdstrike"];

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .expect("reqwest client must build");

    // Fetch detections from org-a CrowdStrike.
    // CrowdStrike DTU requires Authorization: Bearer <token> — any non-empty token accepted.
    let resp_a = client
        .get(format!("http://{org_a_addr}/detects/queries/detects/v1"))
        .header("Authorization", "Bearer dtu-test-token-org-a")
        .send()
        .await
        .expect("GET detections from org-a must not network-error");
    assert_eq!(
        resp_a.status().as_u16(),
        200,
        "org-a CrowdStrike must return HTTP 200 on detections endpoint"
    );
    let body_a = resp_a
        .text()
        .await
        .expect("org-a detections body must be readable");

    // Fetch detections from org-c CrowdStrike.
    let resp_c = client
        .get(format!("http://{org_c_addr}/detects/queries/detects/v1"))
        .header("Authorization", "Bearer dtu-test-token-org-c")
        .send()
        .await
        .expect("GET detections from org-c must not network-error");
    assert_eq!(
        resp_c.status().as_u16(),
        200,
        "org-c CrowdStrike must return HTTP 200 on detections endpoint"
    );
    let body_c = resp_c
        .text()
        .await
        .expect("org-c detections body must be readable");

    // INV-DISTINCT-DATA-001: bodies must differ (seed=100 vs seed=200).
    assert_ne!(
        body_a, body_c,
        "INV-DISTINCT-DATA-001: org-a CrowdStrike (seed=100) and org-c CrowdStrike (seed=200) \
         must serve DISTINCT response bodies. If they are identical, the fixture-gen seeded path \
         is not working correctly — check that build_multi_clone_factory calls new_with_seed \
         with the per-org seed rather than a hardcoded value."
    );

    servers.shutdown();
}

// ---------------------------------------------------------------------------
// MED-2 load-bearing: write_multi_url_sidecar calls the real function and asserts
//   (a) all N×M keys are present in the output, and
//   (b) a missing socket entry fails LOUDLY (not silently dropped via filter_map).
//
// Before this test existed, RG-003 only checked the serialization shape on a
// hand-built HashMap — never calling write_multi_url_sidecar itself.
//
// Traces to: BC-2.06.017 PC-1 (sidecar must contain all N×M entries)
//            Production-grade principle §Standing Rule 3 §2 (no silent drops)
// ---------------------------------------------------------------------------

/// MED-2a: write_multi_url_sidecar against a real MultiInstanceServers produces all N×M keys.
///
/// Verifies that after start_multi_for_config, write_multi_url_sidecar writes a JSON file
/// containing exactly the org_slug×sensor_id keys declared in the config — no keys silently
/// dropped.
#[tokio::test]
async fn test_write_multi_url_sidecar_produces_all_keys() {
    let toml = r#"
        [harness]
        bind = "127.0.0.1"

        [orgs.org-a]
        org_id = "0196f4b2-3c8d-7e1a-b5f0-2d4c6e8a0000"
        sensors = ["crowdstrike", "armis"]
        seed = 100

        [orgs.org-b]
        org_id = "0196f4b2-3c8d-7e1a-b5f0-2d4c6e8a0001"
        sensors = ["claroty"]
        seed = 150
    "#;
    let cfg = MultiOrgDemoConfig::from_str(toml).expect("must parse");
    let servers = prism_dtu_demo_server::start_multi_for_config(&cfg)
        .await
        .expect("must start org-a and org-b clones");

    // Write the sidecar to a tempdir so we don't pollute the workspace cwd.
    let tmp_dir = tempfile::tempdir().expect("tempdir must create");
    let sidecar_path = tmp_dir.path().join("urls-multi.json");

    // write_multi_url_sidecar must be accessible from the library for test use.
    prism_dtu_demo_server::write_multi_url_sidecar_to_path(&servers, &cfg, &sidecar_path)
        .expect("write_multi_url_sidecar_to_path must succeed");

    // Read and parse the written sidecar.
    let contents =
        std::fs::read_to_string(&sidecar_path).expect("sidecar file must exist after write");
    let parsed: std::collections::HashMap<String, std::collections::HashMap<String, String>> =
        serde_json::from_str(&contents).expect("sidecar must be valid JSON");

    // Assert all org×sensor combinations are present.
    assert_eq!(
        parsed.len(),
        2,
        "sidecar must contain exactly 2 org entries (org-a, org-b); got: {:?}",
        parsed.keys().collect::<Vec<_>>()
    );
    assert!(parsed.contains_key("org-a"), "sidecar must contain org-a");
    assert!(parsed.contains_key("org-b"), "sidecar must contain org-b");
    assert_eq!(
        parsed["org-a"].len(),
        2,
        "org-a must have 2 sensor entries (crowdstrike, armis); got: {:?}",
        parsed["org-a"].keys().collect::<Vec<_>>()
    );
    assert!(
        parsed["org-a"].contains_key("crowdstrike"),
        "org-a must have crowdstrike entry"
    );
    assert!(
        parsed["org-a"].contains_key("armis"),
        "org-a must have armis entry"
    );
    assert_eq!(
        parsed["org-b"].len(),
        1,
        "org-b must have 1 sensor entry (claroty)"
    );
    assert!(
        parsed["org-b"].contains_key("claroty"),
        "org-b must have claroty entry"
    );

    // Verify URLs are non-empty and start with http://127.0.0.1
    for (org_slug, sensor_map) in &parsed {
        for (sensor_id, url) in sensor_map {
            assert!(
                url.starts_with("http://127.0.0.1:"),
                "MED-2: {org_slug}:{sensor_id} URL must start with http://127.0.0.1:, got: {url}"
            );
        }
    }

    servers.shutdown();
}

/// MED-2b: write_multi_url_sidecar_to_path with a socket_map missing an expected sensor
/// must FAIL LOUDLY (return Err), not silently drop the entry via filter_map.
///
/// This exercises the production-grade "no silent drops" requirement. The test
/// builds a config that declares org-a sensors = ["crowdstrike", "armis"], but starts
/// only a single crowdstrike clone via start_instances directly (simulating a partial
/// start where armis is missing from socket_map). The write call must return an error.
#[tokio::test]
async fn test_write_multi_url_sidecar_fails_loudly_on_missing_socket() {
    use prism_dtu_demo_server::multi_instance::{InstanceEntry, MultiInstanceConfig};

    // Config declares 2 sensors for org-a: crowdstrike + armis
    let toml = r#"
        [harness]
        bind = "127.0.0.1"

        [orgs.org-a]
        org_id = "0196f4b2-3c8d-7e1a-b5f0-2d4c6e8a0000"
        sensors = ["crowdstrike", "armis"]
        seed = 100
    "#;
    let cfg = MultiOrgDemoConfig::from_str(toml).expect("must parse");

    // Start ONLY crowdstrike — armis is intentionally missing from socket_map.
    // We build a partial MultiInstanceConfig with only org-a-crowdstrike.
    let partial_cfg = MultiInstanceConfig::new(vec![InstanceEntry::new(
        "org-a-crowdstrike",
        "127.0.0.1:0".parse().unwrap(),
    )]);

    // Use build_multi_clone_factory to get a real factory for the crowdstrike clone.
    let factory = prism_dtu_demo_server::build_multi_clone_factory(&cfg);
    let servers = prism_dtu_demo_server::start_instances(partial_cfg, factory)
        .await
        .expect("must start org-a-crowdstrike only");

    // write_multi_url_sidecar_to_path must FAIL LOUDLY when armis is missing from socket_map.
    // Before the MED-2 fix, filter_map silently dropped the missing armis entry and
    // the sidecar only contained crowdstrike — a production bug: demo-run.sh would
    // write no overlay for armis, causing prism boot failure for org-a armis queries.
    let tmp_dir = tempfile::tempdir().expect("tempdir must create");
    let sidecar_path = tmp_dir.path().join("urls-multi-partial.json");

    let result =
        prism_dtu_demo_server::write_multi_url_sidecar_to_path(&servers, &cfg, &sidecar_path);

    assert!(
        result.is_err(),
        "MED-2: write_multi_url_sidecar_to_path must return Err when socket_map is missing \
         an expected sensor entry (org-a-armis). Before the fix, filter_map silently dropped \
         the entry. Got Ok instead of Err — the fix is not in place."
    );

    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("org-a-armis") || err_msg.contains("armis"),
        "MED-2: error message must mention the missing entry ('org-a-armis' or 'armis'), \
         got: {err_msg}"
    );

    servers.shutdown();
}

// ---------------------------------------------------------------------------
// HIGH-1 load-bearing: cmd_configure resolution under nested (multi-org) sidecar
//
// After start-multi, URL_MULTI_FILE (nested) exists but URL_FILE (flat) does not.
// EC-007 specifies `configure cyberint <json>` as the recovery path, but that only
// works if configure can resolve "cyberint" to a URL. Under start-multi, the sidecar
// key is "{org_slug}-{sensor_id}" (e.g. "org-b-cyberint"). The fix must also accept
// bare sensor name lookup across orgs, OR accept the full {org_slug}-{sensor_id} key.
//
// Traces to: EC-007 (configure recovery under start-multi)
//            BC-2.06.017 PC-1 (configure subcommand must be usable after start-multi)
// ---------------------------------------------------------------------------

/// HIGH-1: resolve_configure_url reads the nested sidecar when the flat sidecar is absent.
///
/// Tests the core logic of `cmd_configure` under `start-multi` layout:
/// - Given: only URL_MULTI_FILE (nested) exists; URL_FILE (flat) is absent.
/// - When: `resolve_configure_url("org-b-cyberint", ...)` is called.
/// - Then: returns the correct URL from the nested sidecar.
///
/// Also tests that bare sensor name lookup ("cyberint") resolves when only one org
/// has that sensor (the EC-007 documented recovery form).
#[tokio::test]
async fn test_configure_resolves_url_from_nested_sidecar() {
    // Config with org-b having cyberint only.
    let toml = r#"
        [harness]
        bind = "127.0.0.1"

        [orgs.org-b]
        org_id = "0196f4b2-3c8d-7e1a-b5f0-2d4c6e8a0001"
        sensors = ["cyberint"]
        seed = 150
        initial_access_token = "test-token-high1"
    "#;
    let cfg = MultiOrgDemoConfig::from_str(toml).expect("must parse");
    let servers = prism_dtu_demo_server::start_multi_for_config(&cfg)
        .await
        .expect("must start org-b cyberint");

    let tmp_dir = tempfile::tempdir().expect("tempdir must create");
    let nested_sidecar_path = tmp_dir.path().join("urls-multi.json");

    // Write the nested sidecar.
    prism_dtu_demo_server::write_multi_url_sidecar_to_path(&servers, &cfg, &nested_sidecar_path)
        .expect("write must succeed");

    // HIGH-1: resolve_configure_url with the full {org_slug}-{sensor_id} key must work.
    let url_full = prism_dtu_demo_server::resolve_configure_url(
        "org-b-cyberint",
        None, // no flat sidecar
        Some(&nested_sidecar_path),
    )
    .expect("resolve_configure_url('org-b-cyberint', nested) must find the URL");

    assert!(
        url_full.starts_with("http://127.0.0.1:"),
        "HIGH-1: resolved URL must be http://127.0.0.1:<port>, got: {url_full}"
    );

    // HIGH-1: resolve_configure_url with bare sensor name "cyberint" must also work
    // (EC-007 documented recovery form — one match in the nested sidecar).
    let url_bare =
        prism_dtu_demo_server::resolve_configure_url("cyberint", None, Some(&nested_sidecar_path))
            .expect(
                "resolve_configure_url('cyberint', nested) must find the URL via bare sensor name",
            );

    assert!(
        url_bare.starts_with("http://127.0.0.1:"),
        "HIGH-1: bare sensor name 'cyberint' must resolve to http://127.0.0.1:<port>, got: {url_bare}"
    );

    // The two forms must resolve to the same URL.
    assert_eq!(
        url_full, url_bare,
        "HIGH-1: full key ('org-b-cyberint') and bare sensor name ('cyberint') must resolve \
         to the same URL when only one org has that sensor"
    );

    servers.shutdown();
}
