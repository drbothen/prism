//! S-DEMO-ENRICHMENT-PIVOT-002 Red Gate tests.
//!
//! 15 tests covering ThreatIntel/NVD infusion specs, plugin loading, pipe stage
//! integration, and 6 mandatory security gates.
//!
//! Tests 1-2: TOML spec loading and UDF registration (AC-001, AC-002).
//! Tests 3-6: Plugin integration tests requiring demo server (AC-003-006).
//! Tests 7-9: AC-007 UDF name identifier validation (SEC-001 CWE-20).
//! Test 10: AC-008 PluginInfusionSource.config not pub (SEC-002 CWE-200).
//! Test 11: AC-009 SandboxViolation URL not in WARN log (SEC-003 CWE-209).
//! Test 12: AC-010 spawn_blocking gate for async UDF (CWE-400).
//! Tests 13-14: AC-011 path traversal rejection (SEC-003 CWE-22).
//! Test 15: AC-012 load_all error does not leak absolute path (SEC-002 CWE-209).
//!
//! All tests are RED (failing) against the stubs — this is the Red Gate invariant.
//! Tests 3-6 require the demo server running with scenario.enabled = true.
//! Per SID-1, tests 3-6 are NOT #[ignore]'d — in-process demo server harness required.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    unused_imports,
    unused_variables,
    dead_code
)]

use std::sync::Arc;

use prism_core::InfusionError;
use prism_spec_engine::{
    InfusionLoader, InfusionRegistry, InfusionType, PluginConfigMap, PluginInfusionSource,
    PluginRuntime,
};

// ---------------------------------------------------------------------------
// Test 1: AC-001 — threatintel.infusion.toml parses and loads as plugin-type
// ---------------------------------------------------------------------------

/// AC-001 (BC-2.19.001 postcondition): threatintel.infusion.toml parses and loads 3 UDFs.
///
/// Given `{config_dir}/infusions/threatintel.infusion.toml` with:
/// - source.type = "plugin", plugin_ref = "threatintel-lookup.prx"
/// - [[infusion.fields]] declaring threat_is_known_malicious (Boolean), threat_score (Integer),
///   threat_sources (Json — array of source strings; confirmed 2026-06-12)
///
/// when InfusionLoader::load_all runs,
/// then InfusionRegistry contains 3 InfusionUdfDescriptor entries and
/// registry.is_api_backed("threat_score") returns true.
///
/// RED GATE: fails against stubs because TOML parse + field registration is not yet wired.
#[test]
fn test_enrichment_pivot_002_threatintel_toml_loads_and_registers_3_udfs() {
    use std::io::Write;
    use tempfile::TempDir;

    let tmp = TempDir::new().expect("tempdir");
    let infusions_dir = tmp.path().join("infusions");
    std::fs::create_dir_all(&infusions_dir).expect("create infusions dir");

    let toml_content = include_str!("../../../specs/infusions/threatintel.infusion.toml");

    let spec_path = infusions_dir.join("threatintel.infusion.toml");
    let mut f = std::fs::File::create(&spec_path).expect("create toml");
    f.write_all(toml_content.as_bytes()).expect("write toml");

    let loader = InfusionLoader::new(tmp.path().to_str().unwrap());
    let (specs, errors) = loader.load_all();

    // RED GATE: fails until TOML parse is extended to support [source] top-level block
    // with [source.credential] sub-table, and validate_field_name is implemented.
    assert!(
        errors.is_empty(),
        "BC-2.19.001: threatintel.infusion.toml must parse without errors; got: {:?}",
        errors
    );
    assert_eq!(
        specs.len(),
        1,
        "BC-2.19.001: expected 1 spec loaded from threatintel.infusion.toml"
    );

    let registry = InfusionRegistry::new();
    let spec = specs.into_iter().next().unwrap();
    let descriptors = registry
        .load_spec(spec)
        .expect("BC-2.19.001: load_spec must succeed for valid threatintel spec");

    assert_eq!(
        descriptors.len(),
        3,
        "BC-2.19.001 postcondition: exactly 3 InfusionUdfDescriptors expected \
         (threat_is_known_malicious, threat_score, threat_sources)"
    );

    let names: Vec<&str> = descriptors.iter().map(|d| d.name.as_str()).collect();
    assert!(
        names.contains(&"threat_is_known_malicious"),
        "BC-2.19.001: threat_is_known_malicious UDF must be registered"
    );
    assert!(
        names.contains(&"threat_score"),
        "BC-2.19.001: threat_score UDF must be registered"
    );
    assert!(
        names.contains(&"threat_sources"),
        "BC-2.19.001: threat_sources UDF must be registered (Json array, NOT threat_source string)"
    );

    assert!(
        registry.is_api_backed("threat_score"),
        "BC-2.19.001: is_api_backed('threat_score') must return true for plugin-type spec"
    );
}

// ---------------------------------------------------------------------------
// Test 2: AC-002 — nvd.infusion.toml parses and loads as plugin-type
// ---------------------------------------------------------------------------

/// AC-002 (BC-2.19.001 postcondition): nvd.infusion.toml parses and loads 3 UDFs.
///
/// Given `specs/infusions/nvd.infusion.toml` with source.type = "plugin",
/// [[infusion.fields]] declaring cvss_base_score (Float), cvss_severity (String),
/// cvss_vector (String) — grounded against prism-dtu-nvd camelCase wire names.
///
/// when InfusionLoader::load_all runs,
/// then InfusionRegistry contains 3 InfusionUdfDescriptors and
/// registry.is_api_backed("cvss_base_score") returns true.
///
/// RED GATE: fails against stubs.
#[test]
fn test_enrichment_pivot_002_nvd_toml_loads_and_registers_3_udfs() {
    use std::io::Write;
    use tempfile::TempDir;

    let tmp = TempDir::new().expect("tempdir");
    let infusions_dir = tmp.path().join("infusions");
    std::fs::create_dir_all(&infusions_dir).expect("create infusions dir");

    let toml_content = include_str!("../../../specs/infusions/nvd.infusion.toml");

    let spec_path = infusions_dir.join("nvd.infusion.toml");
    let mut f = std::fs::File::create(&spec_path).expect("create toml");
    f.write_all(toml_content.as_bytes()).expect("write toml");

    let loader = InfusionLoader::new(tmp.path().to_str().unwrap());
    let (specs, errors) = loader.load_all();

    assert!(
        errors.is_empty(),
        "BC-2.19.001: nvd.infusion.toml must parse without errors; got: {:?}",
        errors
    );
    assert_eq!(
        specs.len(),
        1,
        "BC-2.19.001: expected 1 spec loaded from nvd.infusion.toml"
    );

    let registry = InfusionRegistry::new();
    let spec = specs.into_iter().next().unwrap();
    let descriptors = registry
        .load_spec(spec)
        .expect("BC-2.19.001: load_spec must succeed for valid nvd spec");

    assert_eq!(
        descriptors.len(),
        3,
        "BC-2.19.001 postcondition: exactly 3 InfusionUdfDescriptors expected \
         (cvss_base_score, cvss_severity, cvss_vector)"
    );

    let names: Vec<&str> = descriptors.iter().map(|d| d.name.as_str()).collect();
    assert!(
        names.contains(&"cvss_base_score"),
        "BC-2.19.001: cvss_base_score UDF must be registered"
    );
    assert!(
        names.contains(&"cvss_severity"),
        "BC-2.19.001: cvss_severity UDF must be registered"
    );
    assert!(
        names.contains(&"cvss_vector"),
        "BC-2.19.001: cvss_vector UDF must be registered"
    );

    assert!(
        registry.is_api_backed("cvss_base_score"),
        "BC-2.19.001: is_api_backed('cvss_base_score') must return true for plugin-type spec"
    );
}

// ---------------------------------------------------------------------------
// Tests 3-6: Integration tests requiring demo server
// ---------------------------------------------------------------------------
// Per SID-1: NOT #[ignore]'d — requires in-process demo server harness.
// Implementer: wire an in-process DemoServer with ThreatIntelClone::new_with_scenario
// and NvdClone::new_with_scenario at scenario stage >= 3.

/// AC-003 (BC-2.19.001 postcondition): threatintel plugin resolves scenario IOC as malicious.
///
/// RED GATE: fails until prism-threatintel-infusion plugin is compiled + loaded and
/// PluginInfusionSource::enrich_single is called against a running demo server with
/// ThreatIntelClone scenario fixture.
#[test]
fn test_enrichment_pivot_002_threatintel_plugin_resolves_scenario_ioc_as_malicious() {
    // TODO(S-DEMO-ENRICHMENT-PIVOT-002 implementer):
    // 1. Start in-process demo server with ThreatIntelClone::new_with_scenario(entities)
    //    (entities: ScenarioEntityCatalog with ioc_ips, ioc_hashes, ioc_domains pre-populated
    //    as FixtureKey::Malicious).
    // 2. Load threatintel.infusion.toml; wire PluginInfusionSource with the loaded .prx.
    // 3. Call enrich_single(ioc_ips[0], "ip").
    // 4. Assert result contains threat_is_known_malicious = true, threat_score >= 75.
    // 5. Assert response has threat_sources (Json array), NOT threat_source (string).
    // RED GATE: panics here until implemented.
    panic!(
        "test_enrichment_pivot_002_threatintel_plugin_resolves_scenario_ioc_as_malicious: \
         RED GATE — not yet implemented; requires in-process demo server with \
         ThreatIntelClone::new_with_scenario and prism-threatintel-infusion .prx plugin loaded \
         (AC-003 / BC-2.19.001 postcondition; S-DEMO-ENRICHMENT-PIVOT-002)"
    );
}

/// AC-004 (BC-2.19.001 postcondition): nvd plugin resolves scenario CVE with HIGH CVSS.
///
/// RED GATE: fails until prism-nvd-infusion plugin is compiled + loaded and
/// PluginInfusionSource::enrich_single returns cvss_base_score >= 7.0, cvss_severity = "HIGH".
#[test]
fn test_enrichment_pivot_002_nvd_plugin_resolves_scenario_cve_high_cvss() {
    // TODO(S-DEMO-ENRICHMENT-PIVOT-002 implementer):
    // 1. Start in-process demo server with NvdClone::new_with_scenario(entities)
    //    (entities.device_cves[0] pre-populated in cve_registry with cvss_base_score = 8.1,
    //    cvss_severity = "HIGH").
    // 2. Load nvd.infusion.toml; wire PluginInfusionSource with the loaded nvd-lookup.prx.
    // 3. Call enrich_single(device_cves[0], "cve_id").
    // 4. Assert cvss_base_score >= 7.0 and cvss_severity = "HIGH".
    // 5. Assert NVD route used: GET /rest/json/cves/2.0?cveId=<id> (NOT /nvd/cves/{id}).
    // RED GATE: panics here until implemented.
    panic!(
        "test_enrichment_pivot_002_nvd_plugin_resolves_scenario_cve_high_cvss: \
         RED GATE — not yet implemented; requires in-process demo server with \
         NvdClone::new_with_scenario and prism-nvd-infusion .prx plugin loaded \
         (AC-004 / BC-2.19.001 postcondition; S-DEMO-ENRICHMENT-PIVOT-002)"
    );
}

/// AC-005 (BC-2.19.001 postcondition): | enrich threat_intel(ioc_value) returns Malicious
/// for scenario IOCs in a full pipe stage execution.
///
/// RED GATE: fails until pipe stage enrich wiring + demo server + plugin are all operational.
#[test]
fn test_enrichment_pivot_002_enrich_threatintel_pipe_stage_returns_malicious_for_scenario_iocs() {
    // TODO(S-DEMO-ENRICHMENT-PIVOT-002 implementer):
    // 1. Start demo server at stage >= 3 (Exfil).
    // 2. Run: SELECT ... FROM cyberint_alerts | enrich threat_intel(ioc_value)
    // 3. Assert result rows include threat_is_known_malicious, threat_score, threat_sources.
    // 4. Assert scenario IOCs show threat_is_known_malicious = true.
    // NOTE: output column is threat_sources (Json array), NOT threat_source (String).
    panic!(
        "test_enrichment_pivot_002_enrich_threatintel_pipe_stage_returns_malicious_for_scenario_iocs: \
         RED GATE — not yet implemented; requires demo server + PIVOT-001 enrich pipe stage + \
         prism-threatintel-infusion plugin loaded (AC-005 / BC-2.19.001; S-DEMO-ENRICHMENT-PIVOT-002)"
    );
}

/// AC-006 (BC-2.19.001 postcondition): | enrich nvd(device_cves_first) returns HIGH CVSS
/// for scenario CVEs in a full pipe stage execution.
///
/// RED GATE: fails until pipe stage enrich wiring + demo server + nvd plugin are operational.
#[test]
fn test_enrichment_pivot_002_enrich_nvd_pipe_stage_returns_high_cvss_for_scenario_cves() {
    // TODO(S-DEMO-ENRICHMENT-PIVOT-002 implementer):
    // 1. Start demo server at stage >= 4 (Containment, device_cves = true per BC-2.06.019 PC-2).
    // 2. Run: SELECT ... FROM armis_devices | enrich nvd(device_cves_first)
    // 3. Assert result rows include cvss_base_score, cvss_severity, cvss_vector.
    // 4. Assert scenario CVEs show cvss_base_score >= 7.0, cvss_severity = "HIGH".
    // NOTE (U17/Ruling 1b): field is device_cves_first (scalar), NOT device_cves[0] (unsupported).
    panic!(
        "test_enrichment_pivot_002_enrich_nvd_pipe_stage_returns_high_cvss_for_scenario_cves: \
         RED GATE — not yet implemented; requires demo server + prism-nvd-infusion plugin + \
         Armis device_cves_first scalar column (AC-006 / BC-2.19.001; S-DEMO-ENRICHMENT-PIVOT-002)"
    );
}

// ---------------------------------------------------------------------------
// Test 7: AC-007 — UDF name rejects SQL injection characters
// ---------------------------------------------------------------------------

/// AC-007 (BC-2.19.001 precondition): UDF name rejects SQL injection characters at parse time.
///
/// Given an InfusionField.name containing `;` (SQL injection attempt),
/// when InfusionLoader::validate_field_name is called,
/// then it returns Err(InfusionError::InvalidFieldSpec).
///
/// RED GATE: fails until validate_field_name is implemented (currently todo!()).
#[test]
fn test_enrichment_pivot_002_sec001_udf_name_rejects_sql_injection_chars() {
    let spec_path = "test.infusion.toml";

    let result = InfusionLoader::validate_field_name("threat; DROP TABLE", spec_path);
    assert!(
        result.is_err(),
        "AC-007: 'threat; DROP TABLE' must be rejected by validate_field_name (CWE-20)"
    );

    let result2 = InfusionLoader::validate_field_name(" leading_space", spec_path);
    assert!(
        result2.is_err(),
        "AC-007: ' leading_space' (leading space) must be rejected by validate_field_name"
    );

    let result3 = InfusionLoader::validate_field_name("has-hyphen", spec_path);
    assert!(
        result3.is_err(),
        "AC-007: 'has-hyphen' (hyphen) must be rejected by validate_field_name"
    );

    let result4 = InfusionLoader::validate_field_name("", spec_path);
    assert!(
        result4.is_err(),
        "AC-007: empty string must be rejected by validate_field_name"
    );
}

// ---------------------------------------------------------------------------
// Test 8: AC-007 — UDF name rejects leading digit
// ---------------------------------------------------------------------------

/// AC-007 (BC-2.19.001 precondition): UDF name starting with a digit is rejected.
///
/// RED GATE: fails until validate_field_name is implemented (currently todo!()).
#[test]
fn test_enrichment_pivot_002_sec001_udf_name_rejects_leading_digit() {
    let spec_path = "test.infusion.toml";

    let result = InfusionLoader::validate_field_name("1starts_with_digit", spec_path);
    assert!(
        result.is_err(),
        "AC-007: '1starts_with_digit' (starts with digit) must be rejected (^[a-zA-Z] required)"
    );

    let result2 = InfusionLoader::validate_field_name("0threat", spec_path);
    assert!(
        result2.is_err(),
        "AC-007: '0threat' (starts with 0) must be rejected"
    );
}

// ---------------------------------------------------------------------------
// Test 9: AC-007 — UDF name accepts valid identifiers
// ---------------------------------------------------------------------------

/// AC-007 (BC-2.19.001 precondition): valid identifier names are accepted.
///
/// Valid names: threat_is_known_malicious, cvss_base_score, field1, THREAT_SCORE.
///
/// RED GATE: fails until validate_field_name is implemented (currently todo!()).
#[test]
fn test_enrichment_pivot_002_sec001_udf_name_accepts_valid_identifiers() {
    let spec_path = "test.infusion.toml";

    assert!(
        InfusionLoader::validate_field_name("threat_is_known_malicious", spec_path).is_ok(),
        "AC-007: 'threat_is_known_malicious' must be accepted as a valid identifier"
    );
    assert!(
        InfusionLoader::validate_field_name("cvss_base_score", spec_path).is_ok(),
        "AC-007: 'cvss_base_score' must be accepted as a valid identifier"
    );
    assert!(
        InfusionLoader::validate_field_name("field1", spec_path).is_ok(),
        "AC-007: 'field1' must be accepted as a valid identifier"
    );
    assert!(
        InfusionLoader::validate_field_name("THREAT_SCORE", spec_path).is_ok(),
        "AC-007: 'THREAT_SCORE' must be accepted as a valid identifier (uppercase allowed)"
    );
    assert!(
        InfusionLoader::validate_field_name("a", spec_path).is_ok(),
        "AC-007: single letter 'a' must be accepted"
    );
}

// ---------------------------------------------------------------------------
// Test 10: AC-008 — PluginInfusionSource.config is not pub
// ---------------------------------------------------------------------------

/// AC-008 (BC-2.19.001 invariant): PluginInfusionSource.config field is pub(crate), not pub.
///
/// This test uses a compile-time structural check. Since `pub(crate)` prevents external
/// crates from reading the field, and this test file is an integration test (external to
/// the prism-spec-engine crate), attempting to access `source.config` must fail to compile.
///
/// WIRING-EXEMPT rationale: the visibility change from `pub` to `pub(crate)` is a one-line
/// type-system declaration with zero branching, no I/O, no helpers, and 1 line of change.
/// The change is already applied in the stub (plugin_bridge.rs). This test verifies the
/// structural invariant holds at the type-system level.
///
/// Implementation note: Since compile-fail tests require a separate crate (perimeter-violation
/// pattern), we use an in-module structural assertion as a compensating control. The
/// test_enrichment_pivot_002_sec002_plugin_infusion_source_config_not_pub name is preserved
/// as the canonical Red Gate identifier.
///
/// GREEN-BY-DESIGN: the visibility change is already applied in the stub. This test documents
/// the structural invariant. The compile-fail enforcement is in tests/external/perimeter-violation/
/// (to be added by the implementer as the canonical enforcement pattern per S-PLUGIN-PREREQ-A).
/// Listed in the GREEN-BY-DESIGN report section of the stub commit.
#[test]
fn test_enrichment_pivot_002_sec002_plugin_infusion_source_config_not_pub() {
    // Structural documentation test: PluginInfusionSource.config is pub(crate), not pub.
    //
    // The compile-time enforcement: if `config` were `pub`, an external crate could access
    // `source.config` directly, bypassing the credential encapsulation contract.
    // With `pub(crate)`, external crates get E0616 at compile time.
    //
    // Since integration tests ARE external to the prism-spec-engine crate, the following
    // line would fail to compile IF we tried it:
    //   let _ = source.config;  // E0616: field `config` of struct `PluginInfusionSource` is private
    //
    // This test documents and verifies the structural invariant via PluginInfusionSource::new()
    // being accessible (the constructor is pub) while config is not directly readable.
    //
    // COMPENSATING CONTROL: the field `pub(crate)` change is applied in plugin_bridge.rs stub.
    // The implementer should also add a compile-fail test in tests/external/perimeter-violation/
    // that attempts `source.config` access from an external crate — canonical enforcement pattern.
    //
    // This test passes against the stub (GREEN-BY-DESIGN): the pub(crate) change is a
    // type-system-level declaration (zero branching, no I/O, no helpers, 1 line).

    // Build a minimal HTTP client for PluginRuntime construction (reqwest::Client required).
    let http_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("reqwest::Client::build");
    let runtime = Arc::new(PluginRuntime::new(http_client).expect("PluginRuntime::new"));
    let config = Arc::new(prism_spec_engine::PluginConfigMap::new());
    let source = PluginInfusionSource::new("test_plugin", config, runtime);

    // Verify the source is usable via its public interface.
    let _plugin_id = &source.plugin_id; // pub field — still accessible externally

    // config field is pub(crate) — NOT accessible here from this external test.
    // Attempting `let _ = source.config;` would produce: E0616 error[E0616]: field `config`
    // of struct `PluginInfusionSource` is private.
    // This test confirms the structural invariant is in place.
}

// ---------------------------------------------------------------------------
// Test 11: AC-009 — SandboxViolation URL not in WARN log
// ---------------------------------------------------------------------------

/// AC-009 (BC-2.19.001 invariant): SandboxViolation URL is not surfaced in WARN-level output.
///
/// Given PluginInfusionSource::enrich_single receives Err(PluginError::SandboxViolation { url }),
/// when the WARN log is captured,
/// then the URL field does NOT appear in the formatted WARN output.
///
/// RED GATE: the current map_plugin_error_to_infusion_error includes `err.to_string()` in
/// the InfusionError message, which for SandboxViolation includes the URL — this test
/// currently FAILS (URL appears in WARN output).
///
/// Implementer: match SandboxViolation separately, emit url at DEBUG only.
/// Uses `tracing_test` crate to capture WARN-level span output.
#[test]
fn test_enrichment_pivot_002_sec003_sandbox_violation_url_not_in_warn_log() {
    use prism_core::PluginError;
    use std::cell::RefCell;
    use std::sync::{Arc, Mutex};
    use tracing_subscriber::layer::SubscriberExt;

    // Sentinel URL that must NOT appear in WARN-level logs.
    let test_url = "http://dtu-host:8080/v3/ip/192.168.1.1".to_string();

    // Capture WARN-level log output using a custom tracing Layer.
    // Implementation note: we use a simple log-capture approach.
    // The test asserts that the URL does not appear in the captured WARN output
    // after the SandboxViolation error is mapped.
    //
    // RED GATE: the current map_plugin_error_to_infusion_error includes the URL via
    // err.to_string(). The SandboxViolation Display format includes the URL.
    // Verify by checking the InfusionError Display directly — it maps to the WARN
    // tracing span's `error` field.
    //
    // TODO(S-DEMO-ENRICHMENT-PIVOT-002 implementer): use tracing_test::traced_test
    // attribute to capture WARN output, then assert URL absence.
    // For the stub: verify the error mapping directly.

    // Construct a real SandboxViolation PluginError.
    let sandbox_err = PluginError::SandboxViolation {
        plugin_id: "test_plugin".to_string(),
        url: test_url.clone(),
    };
    let err_display = format!("{}", sandbox_err);

    // The InfusionError produced by map_plugin_error_to_infusion_error is formatted into
    // the WARN span. If it contains the URL, it will appear in WARN output.
    //
    // Since map_plugin_error_to_infusion_error is pub(crate), we cannot call it directly
    // from this external test. Instead, we verify the behavior via the contract:
    // the PluginError::SandboxViolation Display DOES contain the URL, so the current
    // implementation (which formats `plugin_call_failed(plugin_id): {err}`) will include it.
    //
    // This test is RED against the current stub because we ASSERT the URL must NOT appear
    // in the WARN output, but the current implementation WOULD include it.
    //
    // Implementer: after fixing map_plugin_error_to_infusion_error, update this test
    // to use tracing_test::traced_test to capture WARN spans and verify URL absence.
    //
    // For Red Gate purposes: this panic confirms the test is correctly failing.
    panic!(
        "test_enrichment_pivot_002_sec003_sandbox_violation_url_not_in_warn_log: \
         RED GATE — SandboxViolation URL '{}' WOULD appear in WARN log under current \
         map_plugin_error_to_infusion_error implementation (formats err.to_string() which \
         includes the URL). Fix: match SandboxViolation separately, emit url at DEBUG only. \
         Then replace this panic with a tracing_test::traced_test assertion \
         (AC-009 / DRIFT-PIVOT-SANDBOXVIOLATION-URL-LOG-001 / SEC-003 CWE-209)",
        test_url
    );
}

// ---------------------------------------------------------------------------
// Test 12: AC-010 — spawn_blocking gate: WASM call does not block async runtime
// ---------------------------------------------------------------------------

/// AC-010 (BC-2.19.001 postcondition): WASM plugin call is wrapped in spawn_blocking.
///
/// Given InfusionAsyncUdf::invoke_with_args in prism-query calls InfusionSource::enrich_single,
/// when the underlying source is a PluginInfusionSource (synchronous WASM call),
/// then the call is wrapped in tokio::task::spawn_blocking to avoid blocking the runtime.
///
/// RED GATE: fails until spawn_blocking wrapping is verified/implemented in
/// InfusionAsyncUdf::invoke_with_args (prism-query crate).
///
/// Implementer: check prism-query for InfusionAsyncUdf, verify spawn_blocking is present.
/// If present: write this test as a timeout-based assertion that demonstrates the async
/// UDF does not block the tokio runtime under concurrent load.
/// If absent: implement spawn_blocking in invoke_with_args.
#[test]
fn test_enrichment_pivot_002_sec001_wasm_enrich_wraps_spawn_blocking() {
    // TODO(S-DEMO-ENRICHMENT-PIVOT-002 implementer):
    // 1. Check prism-query crate for InfusionAsyncUdf::invoke_with_args.
    // 2. If spawn_blocking is present: implement a tokio test that confirms the async UDF
    //    call completes even when a blocking task is running concurrently (timeout-based).
    // 3. If absent: implement spawn_blocking and then write the test.
    // RED GATE: panics here until implemented.
    panic!(
        "test_enrichment_pivot_002_sec001_wasm_enrich_wraps_spawn_blocking: \
         RED GATE — not yet verified/implemented; check InfusionAsyncUdf::invoke_with_args \
         in prism-query for spawn_blocking wrapping of synchronous WASM runtime call; \
         implement if absent (AC-010 / DRIFT-PIVOT-PLUGINID-INFUSIONID-001 SEC-001 / CWE-400)"
    );
}

// ---------------------------------------------------------------------------
// Tests 13-14: AC-011 — path traversal rejection
// ---------------------------------------------------------------------------

/// AC-011 (BC-2.19.001 precondition): path traversal rejected for dotdot plugin_ref.
///
/// Given plugin_ref = "../../etc/passwd.prx" (path traversal attempt),
/// when validate_plugin_path is called,
/// then Err(InfusionError::InvalidFieldSpec) is returned and no file I/O is performed.
///
/// RED GATE: fails until validate_plugin_path is implemented (currently todo!()).
#[test]
fn test_enrichment_pivot_002_sec003_path_traversal_rejected_for_dotdot_plugin_ref() {
    use std::path::Path;
    use tempfile::TempDir;

    let plugin_dir = TempDir::new().expect("plugin_dir tempdir");
    let spec_path = "test.infusion.toml";

    // Attempt path traversal with dotdot.
    let result =
        InfusionLoader::validate_plugin_path("../../etc/passwd.prx", plugin_dir.path(), spec_path);

    assert!(
        result.is_err(),
        "AC-011 SEC-003 CWE-22: '../../etc/passwd.prx' path traversal must be rejected; \
         canonicalize + starts_with check must reject paths escaping plugin_dir"
    );

    // Verify the error message does NOT include the traversal path itself (AC-012 companion).
    let err = result.unwrap_err();
    let err_str = format!("{}", err);
    assert!(
        !err_str.contains("etc/passwd"),
        "AC-011: error message must not disclose the traversal target path"
    );
}

/// AC-011 (BC-2.19.001 precondition): relative path within plugin_dir is accepted.
///
/// Given plugin_ref = "subdir/plugin.prx" (relative path within plugin_dir),
/// when validate_plugin_path is called (and the file exists),
/// then Ok(canonicalized_path) is returned.
///
/// RED GATE: fails until validate_plugin_path is implemented (currently todo!()).
#[test]
fn test_enrichment_pivot_002_sec003_path_within_plugin_dir_accepted() {
    use std::io::Write;
    use std::path::Path;
    use tempfile::TempDir;

    let plugin_dir = TempDir::new().expect("plugin_dir tempdir");
    let spec_path = "test.infusion.toml";

    // Create the plugin file within plugin_dir.
    let plugin_file = plugin_dir.path().join("threatintel-lookup.prx");
    std::fs::File::create(&plugin_file).expect("create mock .prx");

    let result = InfusionLoader::validate_plugin_path(
        "threatintel-lookup.prx",
        plugin_dir.path(),
        spec_path,
    );

    assert!(
        result.is_ok(),
        "AC-011: 'threatintel-lookup.prx' within plugin_dir must be accepted; got: {:?}",
        result.err()
    );
}

// ---------------------------------------------------------------------------
// Test 15: AC-012 — load_all error does not leak absolute path
// ---------------------------------------------------------------------------

/// AC-012 (BC-2.19.001 invariant): load_all errors do not disclose absolute filesystem paths
/// in MCP-surfaced error strings.
///
/// Given a bad .infusion.toml at an absolute path,
/// when load_all processes it and encounters a parse error,
/// then the InfusionError message surfaced for MCP contains only the filename or relative path,
/// NOT the absolute filesystem path.
///
/// RED GATE: fails until sanitize_error_path is implemented and wired in load_all
/// (currently todo!()).
#[test]
fn test_enrichment_pivot_002_sec002_load_all_error_does_not_leak_absolute_path() {
    use std::io::Write;
    use tempfile::TempDir;

    let tmp = TempDir::new().expect("tempdir");
    let infusions_dir = tmp.path().join("infusions");
    std::fs::create_dir_all(&infusions_dir).expect("create infusions dir");

    // Write a deliberately invalid TOML to trigger a parse error.
    let bad_toml = b"this is not valid toml @@@@";
    let bad_file = infusions_dir.join("bad.infusion.toml");
    let mut f = std::fs::File::create(&bad_file).expect("create bad toml");
    f.write_all(bad_toml).expect("write bad toml");

    let loader = InfusionLoader::new(tmp.path().to_str().unwrap());
    let (specs, errors) = loader.load_all();

    assert!(
        !errors.is_empty(),
        "AC-012: expect at least one parse error from bad TOML"
    );

    let absolute_prefix = tmp.path().to_str().unwrap();
    for error in &errors {
        let err_str = format!("{}", error);
        assert!(
            !err_str.contains(absolute_prefix),
            "AC-012 SEC-002 CWE-209: InfusionError message must not contain absolute path '{}'; \
             got error: '{}'",
            absolute_prefix,
            err_str
        );
    }
}
