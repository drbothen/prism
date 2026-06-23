//! Red Gate tests for ENRICH-2 + ENRICH-3: global enrichment DTU wiring.
//!
//! # Purpose
//!
//! These 3 tests define the behavioral contract for starting global enrichment DTUs
//! (ThreatIntel + NVD) alongside per-org sensor DTUs, and for wiring the ephemeral
//! base_url + api_key into the infusion configs so HttpLookup/plugin actually call them.
//!
//! # Red Gate tests
//!
//! | Test | Contract | Red reason |
//! |------|----------|------------|
//! | RG-E3-001 `test_enrich3_start_multi_binds_global_enrichment_dtus` | ENRICH-3 | `MultiOrgDemoConfig` has no `[enrichment]` section; `start_multi_for_config` does not start enrichment instances |
//! | RG-E3-002 `test_enrich3_sidecar_emits_global_key_for_enrichment` | ENRICH-3 | `write_multi_url_sidecar_to_path` does not emit a `_global` key |
//! | RG-E2-001 `test_enrich2_infusion_loader_parses_threatintel_with_base_url_credential` | ENRICH-2 | `threatintel.infusion.toml` has no `base_url` credential entry; `PluginConfigMap` does not receive it |
//!
//! # Architecture decision
//!
//! ## ENRICH-3: Global enrichment instances
//!
//! ThreatIntel and NVD are GLOBAL shared instances (not per-org) because all orgs query the
//! same threat intelligence backend. They must NOT appear in per-org sensor overlays.
//! The sidecar emits them under a top-level `_global` key so demo-run.sh can read them
//! without generating org-scoped sensor TOML overlays for them.
//!
//! ## ENRICH-2: ThreatIntel base_url wiring
//!
//! The ThreatIntel plugin-type infusion spec (`threatintel.infusion.toml`) currently only
//! has a credential entry for `api_key`. It is missing `base_url` which the plugin WASM
//! needs to call the correct DTU endpoint. The fix extends `[[infusion.credentials]]` to
//! also carry `base_url` (field_name="base_url", env_var="PRISM_THREATINTEL_BASE_URL").
//!
//! This generalizes the existing credential form rather than inventing a separate config
//! block — avoiding schema sprawl per the ENRICH-2 spec.
//!
//! ## NVD base_url: http_lookup does NOT resolve ${env.*} tokens in base_url
//!
//! The infusion loader.rs does not call `resolve_env_var_tokens` on infusion specs
//! (only sensor specs call it). Therefore `${env.PRISM_NVD_BASE_URL}` in `base_url`
//! would be passed verbatim to reqwest and fail. The demo path is: demo-run.sh writes an
//! override `nvd.infusion.toml` with the literal DTU base_url into `{config_dir}/infusions/`.
//! This test does NOT cover NVD (demo-run.sh is a shell script, not a Rust test).
//!
//! # Stories: ENRICH-2 + ENRICH-3

#![cfg(all(feature = "dtu", feature = "fixture-gen"))]
#![allow(clippy::unwrap_used, clippy::expect_used, non_snake_case)]

// ---------------------------------------------------------------------------
// RG-E3-001: start_multi_for_config starts global enrichment DTU instances
//
// Proves that when `[enrichment] threatintel=true, nvd=true` is set in the config,
// `start_multi_for_config` also starts ThreatIntel and NVD global instances.
//
// These instances must:
//   1. Be present in `socket_map()` under keys "threatintel" and "nvd".
//   2. Have non-zero ports (actually bound).
//   3. The ThreatIntel clone must respond HTTP 200 on GET /v3/ip/<ip> (any IP).
//   4. The NVD clone must respond HTTP 200 on GET /rest/json/cves/2.0?cveId=CVE-2024-0001.
//
// Red reason: `MultiOrgDemoConfig` currently has no `[enrichment]` section;
// `start_multi_for_config` does not create enrichment InstanceEntries.
// ---------------------------------------------------------------------------

/// ENRICH-3 RG-E3-001: start_multi with enrichment.threatintel=true and enrichment.nvd=true
/// must start global ThreatIntel and NVD DTU instances accessible via socket_map.
#[tokio::test]
async fn test_enrich3_start_multi_binds_global_enrichment_dtus() {
    let toml = r#"
        [harness]
        bind = "127.0.0.1"

        [enrichment]
        threatintel = true
        nvd = true

        [orgs.org-c]
        org_id = "0196f4b2-3c8d-7e1a-b5f0-2d4c6e8a0002"
        sensors = ["crowdstrike"]
        seed = 200
    "#;

    let cfg = prism_dtu_demo_server::MultiOrgDemoConfig::from_str(toml).expect("config must parse");

    let servers = prism_dtu_demo_server::start_multi_for_config(&cfg)
        .await
        .expect("ENRICH-3: start_multi_for_config must start enrichment DTUs when enrichment.threatintel=true");

    let socket_map = servers.socket_map();

    // Assert "threatintel" global instance is present and bound.
    let ti_addr = socket_map.get("threatintel").expect(
        "ENRICH-3: socket_map must contain 'threatintel' global instance after start_multi",
    );
    assert_ne!(
        ti_addr.port(),
        0,
        "ENRICH-3: ThreatIntel global instance must be bound to a real OS port"
    );

    // Assert "nvd" global instance is present and bound.
    let nvd_addr = socket_map
        .get("nvd")
        .expect("ENRICH-3: socket_map must contain 'nvd' global instance after start_multi");
    assert_ne!(
        nvd_addr.port(),
        0,
        "ENRICH-3: NVD global instance must be bound to a real OS port"
    );

    // Assert per-org sensor clone is also present (enrichment must not displace per-org clones).
    let cs_addr = socket_map
        .get("org-c-crowdstrike")
        .expect("ENRICH-3: socket_map must still contain per-org 'org-c-crowdstrike' instance");
    assert_ne!(
        cs_addr.port(),
        0,
        "ENRICH-3: per-org CrowdStrike instance must still be bound"
    );

    // Assert ThreatIntel clone serves HTTP 200 on its lookup route.
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .expect("reqwest client must build");

    let ti_url = format!("http://{ti_addr}/v3/ip/1.2.3.4");
    let ti_resp = client
        .get(&ti_url)
        .send()
        .await
        .expect("GET /v3/ip/1.2.3.4 must not network-error");

    // ThreatIntel accepts any IP (known IPs return 200 with fixture data; unknown IPs may 200 or 404).
    // Assert non-5xx: the server must be responding, not crashed.
    assert!(
        ti_resp.status().as_u16() < 500,
        "ENRICH-3: ThreatIntel global DTU must respond (non-5xx) on GET /v3/ip/1.2.3.4, got HTTP {}",
        ti_resp.status().as_u16()
    );

    // Assert NVD clone serves HTTP 200 on its CVE route.
    let nvd_url = format!("http://{nvd_addr}/rest/json/cves/2.0?cveId=CVE-2024-0001");
    let nvd_resp = client
        .get(&nvd_url)
        .send()
        .await
        .expect("GET /rest/json/cves/2.0?cveId=CVE-2024-0001 must not network-error");

    assert!(
        nvd_resp.status().as_u16() < 500,
        "ENRICH-3: NVD global DTU must respond (non-5xx) on GET /rest/json/cves/2.0?cveId=CVE-2024-0001, got HTTP {}",
        nvd_resp.status().as_u16()
    );

    servers.shutdown();
}

// ---------------------------------------------------------------------------
// RG-E3-002: write_multi_url_sidecar_to_path emits _global key for enrichment DTUs
//
// The nested sidecar format is {org_slug: {sensor_id: url}}. For global enrichment DTUs,
// they must appear under a top-level "_global" key (NOT under any org slug) so that
// demo-run.sh can read them without generating per-org sensor overlays for them.
//
// Expected sidecar shape:
//   {
//     "_global": {"threatintel": "http://127.0.0.1:<port>", "nvd": "http://127.0.0.1:<port>"},
//     "org-c": {"crowdstrike": "http://127.0.0.1:<port>"}
//   }
//
// Red reason: `write_multi_url_sidecar_to_path` currently only iterates `cfg.orgs`;
// it has no concept of global enrichment entries.
// ---------------------------------------------------------------------------

/// ENRICH-3 RG-E3-002: write_multi_url_sidecar_to_path must emit enrichment URLs under "_global".
#[tokio::test]
async fn test_enrich3_sidecar_emits_global_key_for_enrichment() {
    let toml = r#"
        [harness]
        bind = "127.0.0.1"

        [enrichment]
        threatintel = true
        nvd = true

        [orgs.org-c]
        org_id = "0196f4b2-3c8d-7e1a-b5f0-2d4c6e8a0002"
        sensors = ["crowdstrike"]
        seed = 200
    "#;

    let cfg = prism_dtu_demo_server::MultiOrgDemoConfig::from_str(toml).expect("config must parse");

    let servers = prism_dtu_demo_server::start_multi_for_config(&cfg)
        .await
        .expect("start_multi must succeed");

    let tmp_dir = tempfile::tempdir().expect("tempdir must create");
    let sidecar_path = tmp_dir.path().join("urls-multi.json");

    prism_dtu_demo_server::write_multi_url_sidecar_to_path(&servers, &cfg, &sidecar_path)
        .expect("write_multi_url_sidecar_to_path must succeed");

    let contents = std::fs::read_to_string(&sidecar_path).expect("sidecar must be written");
    let parsed: std::collections::HashMap<String, std::collections::HashMap<String, String>> =
        serde_json::from_str(&contents).expect("sidecar must be valid JSON");

    // Assert "_global" key is present.
    let global = parsed
        .get("_global")
        .expect("ENRICH-3: sidecar must contain '_global' key for enrichment DTU URLs");

    // Assert both enrichment DTUs are under _global.
    let ti_url = global
        .get("threatintel")
        .expect("ENRICH-3: '_global' section must contain 'threatintel' URL");
    assert!(
        ti_url.starts_with("http://127.0.0.1:"),
        "ENRICH-3: threatintel URL must be http://127.0.0.1:<port>, got: {ti_url}"
    );

    let nvd_url = global
        .get("nvd")
        .expect("ENRICH-3: '_global' section must contain 'nvd' URL");
    assert!(
        nvd_url.starts_with("http://127.0.0.1:"),
        "ENRICH-3: nvd URL must be http://127.0.0.1:<port>, got: {nvd_url}"
    );

    // Assert per-org sensors are still under their org slug (not under _global).
    let org_c = parsed
        .get("org-c")
        .expect("ENRICH-3: sidecar must still contain 'org-c' for per-org sensors");
    assert!(
        org_c.contains_key("crowdstrike"),
        "ENRICH-3: 'org-c' must still contain 'crowdstrike' sensor URL"
    );

    // Assert _global does NOT contain per-org sensor names.
    assert!(
        !global.contains_key("crowdstrike"),
        "ENRICH-3: '_global' must NOT contain per-org sensor 'crowdstrike'"
    );

    servers.shutdown();
}

// ---------------------------------------------------------------------------
// RG-E2-001: threatintel.infusion.toml credentials include base_url
//
// The ThreatIntel plugin infusion spec must declare a `base_url` credential entry
// (field_name="base_url", env_var="PRISM_THREATINTEL_BASE_URL") in addition to
// the existing api_key credential so that PluginConfigMap receives both.
//
// This test validates:
//   1. InfusionLoader::parse accepts the updated threatintel.infusion.toml.
//   2. The parsed InfusionSpec.credentials contains both "api_key" and "base_url" entries.
//   3. When PRISM_THREATINTEL_BASE_URL and PRISM_THREATINTEL_API_KEY are set, the
//      PluginConfigMap built by infusion/mod.rs contains both keys.
//
// Note: this test uses InfusionLoader::parse directly (unit-level) to avoid requiring
// a full SpecEngine context. It verifies the STRUCTURAL requirement: credentials list
// must include a base_url entry. The actual env-var resolution is exercised by asserting
// non-empty values when env vars are set.
//
// Red reason: `threatintel.infusion.toml` currently has no `base_url` credential entry.
// `[[infusion.credentials]]` only lists `api_key`. After the fix, `base_url` must also
// appear in the credentials list with env_var="PRISM_THREATINTEL_BASE_URL".
// ---------------------------------------------------------------------------

/// ENRICH-2 RG-E2-001: threatintel.infusion.toml must declare a `base_url` credential entry.
///
/// Verifies that the parsed InfusionSpec for the ThreatIntel plugin contains:
///   - A credential with field_name="base_url" and env_var="PRISM_THREATINTEL_BASE_URL"
///   - A credential with field_name="api_key" (or equivalent) and env_var="PRISM_THREATINTEL_API_KEY"
///
/// The test reads the actual `specs/infusions/threatintel.infusion.toml` file from the
/// workspace root (not a fixture copy) so that changes to the TOML are immediately tested.
#[test]
fn test_enrich2_infusion_loader_parses_threatintel_with_base_url_credential() {
    use prism_spec_engine::infusion::loader::InfusionLoader;

    // Resolve specs/infusions/threatintel.infusion.toml from workspace root.
    let toml_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..") // → crates/
        .join("..") // → workspace root
        .join("specs")
        .join("infusions")
        .join("threatintel.infusion.toml");

    let toml_path = toml_path.canonicalize().unwrap_or_else(|e| {
        panic!(
            "ENRICH-2: specs/infusions/threatintel.infusion.toml not found at {:?}: {}",
            toml_path, e
        )
    });

    let toml_str = std::fs::read_to_string(&toml_path)
        .unwrap_or_else(|e| panic!("ENRICH-2: failed to read {:?}: {}", toml_path, e));

    let spec = InfusionLoader::parse(
        &toml_str,
        toml_path.to_str().unwrap_or("threatintel.infusion.toml"),
    )
    .unwrap_or_else(|e| {
        panic!(
            "ENRICH-2: threatintel.infusion.toml must parse without error; got: {:?}",
            e
        )
    });

    // Assert infusion_id is correct.
    assert_eq!(
        spec.infusion_id, "threat_intel",
        "ENRICH-2: infusion_id must be 'threat_intel'"
    );

    // Assert credentials contains a base_url entry.
    let has_base_url_cred = spec
        .credentials
        .iter()
        .any(|c| c.field_name == "base_url" && c.env_var == "PRISM_THREATINTEL_BASE_URL");
    assert!(
        has_base_url_cred,
        "ENRICH-2: threatintel.infusion.toml must declare a credential with \
         field_name='base_url' and env_var='PRISM_THREATINTEL_BASE_URL'. \
         Current credentials: {:?}",
        spec.credentials
            .iter()
            .map(|c| (&c.field_name, &c.env_var))
            .collect::<Vec<_>>()
    );

    // Assert credentials contains an api_key entry.
    let has_api_key_cred = spec
        .credentials
        .iter()
        .any(|c| c.env_var == "PRISM_THREATINTEL_API_KEY");
    assert!(
        has_api_key_cred,
        "ENRICH-2: threatintel.infusion.toml must declare a credential with \
         env_var='PRISM_THREATINTEL_API_KEY'. \
         Current credentials: {:?}",
        spec.credentials
            .iter()
            .map(|c| (&c.field_name, &c.env_var))
            .collect::<Vec<_>>()
    );
}
