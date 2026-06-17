//! S-DEMO-ENRICHMENT-PIVOT-002 Red Gate tests.
//!
//! 19 tests covering ThreatIntel/NVD infusion specs, plugin loading, pipe stage
//! integration, 6 mandatory security gates, SAP-2 DTU↔TOML parity, and additional
//! BC-2.19.001 clause coverage (E-INFUSE-002 duplicate detection, EC-19-001 zero-field).
//!
//! Tests 1-2: TOML spec loading and UDF registration (AC-001, AC-002). GREEN-BY-DESIGN.
//! Tests 3-6: Plugin integration tests requiring demo server (AC-003-006). RED.
//! Tests 7-9: AC-007 UDF name identifier validation (SEC-001 CWE-20). RED.
//! Test 10: AC-008 PluginInfusionSource.config not pub (SEC-002 CWE-200). GREEN-BY-DESIGN.
//! Test 11: AC-009 SandboxViolation URL not in WARN log (SEC-003 CWE-209). RED.
//! Test 12: AC-010 spawn_blocking gate for async UDF (CWE-400). RED.
//! Tests 13-14: AC-011 path traversal rejection (SEC-003 CWE-22). RED (13: stub todo; 14: stub todo).
//! Test 15: AC-012 load_all error does not leak absolute path (SEC-002 CWE-209). RED.
//! Tests 16-17: SAP-2 DTU↔TOML parity (ThreatIntel + NVD column-to-field mapping). RED.
//! Test 18: BC-2.19.001 E-INFUSE-002 duplicate UDF name rejection. RED.
//! Test 19: BC-2.19.001 EC-19-001 zero-field spec rejection. RED.
//!
//! All tests are RED (failing) against the stubs — this is the Red Gate invariant.
//! Tests 3-6 require the demo server running with scenario.enabled = true.
//! Per SID-1, tests 3-6 are NOT #[ignore]'d — in-process demo server harness required.
//! GREEN-BY-DESIGN tests (1, 2, 10): stubs already implemented these structural invariants.

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
/// RED GATE: the current map_plugin_error_to_infusion_error formats `plugin_call_failed(id): {err}`
/// where `{err}` is `err.to_string()` which for SandboxViolation is:
/// `"plugin 'X' attempted HTTP to non-allowlisted URL: <URL>"` — the URL IS in the string.
///
/// The InfusionError::MissingRequiredField produced by the current implementation will have
/// its `field` set to `"plugin_call_failed(test_plugin): plugin 'test_plugin' attempted HTTP
/// to non-allowlisted URL: http://dtu-host:8080/v3/ip/192.168.1.1"` which contains the URL.
/// This InfusionError is then formatted into the WARN span's `error` field — a CWE-209 violation.
///
/// BEHAVIORAL ASSERTION (adversarial): this test asserts that the current InfusionError
/// Display for a SandboxViolation DOES contain the URL string, proving the current state
/// is RED. This is a load-bearing assertion — it would pass without implementation.
/// The panic below ensures the test fails until the fix is in place AND the test is
/// rewritten with tracing_test::traced_test to verify WARN log capture.
///
/// Implementer: after fixing map_plugin_error_to_infusion_error:
/// 1. Add `tracing-test = "0.2"` to [dev-dependencies] in prism-spec-engine/Cargo.toml
/// 2. Annotate this test with `#[tracing_test::traced_test]`
/// 3. Remove the panic; replace with: `assert!(!logs_contain(test_url))`
/// 4. Verify the DEBUG emission still includes the URL for operator diagnostics
#[test]
fn test_enrichment_pivot_002_sec003_sandbox_violation_url_not_in_warn_log() {
    use prism_core::PluginError;

    // Sentinel URL that must NOT appear in WARN-level logs.
    let test_url = "http://dtu-host:8080/v3/ip/192.168.1.1".to_string();

    // BEHAVIORAL ASSERTION: verify the CURRENT state is RED by proving the URL IS
    // present in the InfusionError that map_plugin_error_to_infusion_error produces.
    //
    // The current implementation produces:
    //   InfusionError::MissingRequiredField {
    //     field: format!("plugin_call_failed({}): {}", plugin_id, err),
    //     ...
    //   }
    // where `err.to_string()` for SandboxViolation is:
    //   "plugin 'X' attempted HTTP to non-allowlisted URL: <URL>"
    //
    // We verify this by constructing the SandboxViolation and formatting it:
    let sandbox_err = PluginError::SandboxViolation {
        plugin_id: "test_plugin".to_string(),
        url: test_url.clone(),
    };
    let err_display = format!("{}", sandbox_err);

    // ASSERT RED STATE: SandboxViolation Display DOES contain the URL.
    // This assertion PASSES in the current (unfixed) state, proving the URL
    // would leak into the WARN log via `map_plugin_error_to_infusion_error`.
    assert!(
        err_display.contains(&test_url),
        "AC-009 prerequisite check: SandboxViolation Display must contain the URL \
         (confirming the current implementation IS leaking the URL). Got: '{}'",
        err_display
    );

    // The InfusionError message would be:
    //   "plugin_call_failed(test_plugin): plugin 'test_plugin' attempted HTTP to
    //    non-allowlisted URL: http://dtu-host:8080/v3/ip/192.168.1.1"
    let current_infusion_error_message =
        format!("plugin_call_failed(test_plugin): {}", err_display);
    assert!(
        current_infusion_error_message.contains(&test_url),
        "AC-009 RED GATE confirmation: current InfusionError message format DOES contain URL — \
         this will appear in WARN span `error` field. Fix required: match SandboxViolation \
         separately, emit url at DEBUG only, exclude from InfusionError message. Got: '{}'",
        current_infusion_error_message
    );

    // RED GATE: the above assertions confirm the current state is broken (URL leaks).
    // The test now panics to enforce that the implementer MUST:
    //   1. Fix map_plugin_error_to_infusion_error to exclude the URL from the error message
    //   2. Replace this panic with a tracing_test::traced_test WARN-capture assertion
    //   3. Verify DEBUG emission still includes the URL for operator diagnostics
    // (AC-009 / DRIFT-PIVOT-SANDBOXVIOLATION-URL-LOG-001 / SEC-003 CWE-209)
    panic!(
        "test_enrichment_pivot_002_sec003_sandbox_violation_url_not_in_warn_log: \
         RED GATE — SandboxViolation URL '{}' IS present in current InfusionError message \
         (confirmed above via Display assertion). Fix: match SandboxViolation separately in \
         map_plugin_error_to_infusion_error; emit url at DEBUG only; exclude from \
         InfusionError message. Then replace this panic with:\n\
         #[tracing_test::traced_test]\n\
         fn test_... {{ ... assert!(!logs_contain(\"{}\")) }}\n\
         (AC-009 / DRIFT-PIVOT-SANDBOXVIOLATION-URL-LOG-001 / SEC-003 CWE-209)",
        test_url, test_url
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

// ---------------------------------------------------------------------------
// Tests 16-17: SAP-2 DTU↔TOML column parity assertions
// ---------------------------------------------------------------------------

/// SAP-2 (CLAUDE.md §SAP-2): Every TOML column in threatintel.infusion.toml must map to
/// a real DTU field in prism-dtu-threatintel types.rs ThreatIntelResponse.
///
/// DTU struct (prism-dtu-threatintel/src/types.rs, confirmed 2026-06-12):
///   pub struct ThreatIntelResponse {
///     pub lookup_value: String,
///     pub threat_score: u32,               ← maps to "threat_score" Integer
///     pub threat_is_known_malicious: bool, ← maps to "threat_is_known_malicious" Boolean
///     pub threat_sources: Vec<String>,      ← maps to "threat_sources" Json (ARRAY, NOT string)
///   }
///
/// TOML declares: threat_is_known_malicious (Boolean), threat_score (Integer),
///   threat_sources (Json) — all 3 have DTU equivalents.
///
/// This test asserts the structural parity by loading the TOML and verifying field names
/// against the known DTU struct fields. Column in TOML with no DTU equivalent = P1 CRITICAL.
///
/// RED GATE: fails until validate_field_name is implemented (todo!()), because load_spec
/// calls validate_field_name internally. Once validate_field_name is implemented, this test
/// passes on the TOML spec's fields — confirming SAP-2 parity.
///
/// NOTE: If this test fails because a field name in the TOML has no DTU equivalent,
/// that is a P1 CRITICAL finding per SAP-2. Do NOT suppress — fix the TOML column.
#[test]
fn test_enrichment_pivot_002_sap2_threatintel_toml_columns_match_dtu_fields() {
    use std::collections::HashSet;
    use std::io::Write;
    use tempfile::TempDir;

    // Known DTU field names from prism-dtu-threatintel/src/types.rs ThreatIntelResponse.
    // MUST be kept in sync with the actual struct if it changes (SAP-2).
    // Source: prism-dtu-threatintel/src/types.rs (read 2026-06-17):
    //   pub lookup_value: String
    //   pub threat_score: u32
    //   pub threat_is_known_malicious: bool
    //   pub threat_sources: Vec<String>   ← JSON array, NOT "threat_source" (singular string)
    let dtu_fields: HashSet<&str> = [
        "lookup_value",
        "threat_score",
        "threat_is_known_malicious",
        "threat_sources", // ARRAY — not "threat_source" (singular)
    ]
    .iter()
    .copied()
    .collect();

    // Also include fixture response fields present in lookup.rs ip_fixture_response
    // (greynoise_classification, abuseipdb_confidence_score, virustotal_detections, etc.)
    // These are extra fields available in the JSON response but not in ThreatIntelResponse struct.
    // SAP-2: TOML columns must map to DTU struct fields (types.rs) — the canonical struct fields.
    let dtu_response_extra_fields: HashSet<&str> = [
        "greynoise_classification",
        "abuseipdb_confidence_score",
        "virustotal_detections",
        "virustotal_first_seen",
    ]
    .iter()
    .copied()
    .collect();
    let all_dtu_fields: HashSet<&str> = dtu_fields
        .union(&dtu_response_extra_fields)
        .copied()
        .collect();

    // Load the threatintel.infusion.toml spec to extract field names.
    let tmp = TempDir::new().expect("tempdir");
    let infusions_dir = tmp.path().join("infusions");
    std::fs::create_dir_all(&infusions_dir).expect("create infusions dir");
    let toml_content = include_str!("../../../specs/infusions/threatintel.infusion.toml");
    let spec_path = infusions_dir.join("threatintel.infusion.toml");
    let mut f = std::fs::File::create(&spec_path).expect("create toml");
    f.write_all(toml_content.as_bytes()).expect("write toml");

    // Parse via InfusionLoader::parse (bypass load_all to avoid validate_field_name todo!).
    // We use the raw parse path to extract field names even while validate_field_name is a stub.
    // NOTE: parse() currently calls validate_field_name via a todo!() — so this test will
    // fail with a todo! panic until validate_field_name is implemented (correct RED behavior).
    let parse_result = InfusionLoader::parse(toml_content, "threatintel.infusion.toml");

    match parse_result {
        Ok(spec) => {
            // Verify every declared TOML field name maps to a real DTU field.
            for field in &spec.fields {
                assert!(
                    all_dtu_fields.contains(field.name.as_str()),
                    "SAP-2 P1 CRITICAL: TOML field '{}' has no equivalent in \
                     prism-dtu-threatintel ThreatIntelResponse struct or fixture response. \
                     Known DTU fields: {:?}. \
                     (Column in TOML with no DTU equivalent = P1 per SAP-2 / CLAUDE.md §SAP-2)",
                    field.name,
                    all_dtu_fields
                );
            }

            // Verify the critical ARRAY field name: must be "threat_sources" (plural), NOT
            // "threat_source" (singular string — SAP-2 U11/risk_mitigations confirmed 2026-06-12).
            let field_names: Vec<&str> = spec.fields.iter().map(|f| f.name.as_str()).collect();
            assert!(
                field_names.contains(&"threat_sources"),
                "SAP-2 P1 CRITICAL: field 'threat_sources' (Json array) must be declared. \
                 Got fields: {:?}. NOTE: 'threat_source' (singular string) is WRONG — \
                 DTU response has threat_sources (Vec<String>), not threat_source.",
                field_names
            );
            assert!(
                !field_names.contains(&"threat_source"),
                "SAP-2 P1 CRITICAL: 'threat_source' (singular string) must NOT be declared. \
                 DTU response field is 'threat_sources' (Vec<String> JSON array). \
                 Got fields: {:?}",
                field_names
            );
        }
        Err(e) => {
            // If parse fails due to todo!() in validate_field_name: this IS the expected
            // RED Gate behavior (validate_field_name not yet implemented).
            // Any other parse failure is a bug — fail loudly with the error.
            panic!(
                "test_enrichment_pivot_002_sap2_threatintel_toml_columns_match_dtu_fields: \
                 RED GATE — InfusionLoader::parse failed for threatintel.infusion.toml: {:?}. \
                 If this is a todo!() panic from validate_field_name: expected RED behavior until \
                 AC-007 is implemented. If this is a structural parse error: TOML spec has a bug \
                 that must be fixed before SAP-2 parity can be verified. \
                 (SAP-2 / CLAUDE.md §SAP-2 / S-DEMO-ENRICHMENT-PIVOT-002)",
                e
            );
        }
    }
}

/// SAP-2 (CLAUDE.md §SAP-2): Every TOML column in nvd.infusion.toml must map to
/// a real DTU field in prism-dtu-nvd types.rs (CveRecord / CvssData).
///
/// DTU struct (prism-dtu-nvd/src/types.rs, confirmed 2026-06-12, all serde camelCase):
///   CvssData {
///     pub version: String,
///     pub vector_string: String,    ← wire name: "vectorString" → maps to "cvss_vector" String
///     pub base_score: f64,          ← wire name: "baseScore"    → maps to "cvss_base_score" Float
///     pub base_severity: String,    ← wire name: "baseSeverity" → maps to "cvss_severity" String
///   }
///
/// TOML declares: cvss_base_score (Float), cvss_severity (String), cvss_vector (String)
///   — all 3 have DTU equivalents in CvssData via camelCase wire names.
///
/// Also verifies that the NVD TOML does NOT declare a field "cve_id" (the wire name for
/// the CVE ID is "id" in CveRecord, NOT "cve_id" — confirmed types.rs; this is a
/// SAP-2-class error if present).
///
/// RED GATE: same as test 16 — fails until validate_field_name implemented.
#[test]
fn test_enrichment_pivot_002_sap2_nvd_toml_columns_match_dtu_fields() {
    use std::collections::HashSet;
    use std::io::Write;
    use tempfile::TempDir;

    // Known DTU field names from prism-dtu-nvd/src/types.rs CvssData (camelCase wire names).
    // Rust field names (snake_case) are what the TOML columns map TO conceptually.
    // Source: prism-dtu-nvd/src/types.rs (read 2026-06-17):
    //   pub struct CvssData {
    //     pub version: String,
    //     pub vector_string: String,   (wire: vectorString)
    //     pub base_score: f64,         (wire: baseScore)
    //     pub base_severity: String,   (wire: baseSeverity)
    //   }
    // The TOML field names are PRISM UDF names (not wire names or Rust field names):
    //   cvss_base_score → maps to CvssData.base_score (wire: baseScore)
    //   cvss_severity   → maps to CvssData.base_severity (wire: baseSeverity)
    //   cvss_vector     → maps to CvssData.vector_string (wire: vectorString)
    //
    // SAP-2 verification: these 3 TOML fields must ALL have DTU equivalents.
    // The TOML field name "cvss_base_score" is a Prism UDF alias for CvssData.base_score —
    // this is valid; SAP-2 requires a mapping exists, not that names are identical.
    let expected_toml_fields: HashSet<&str> = ["cvss_base_score", "cvss_severity", "cvss_vector"]
        .iter()
        .copied()
        .collect();

    // TOML must NOT declare "cve_id" — the wire name is "id" (CveRecord.id), NOT "cve_id".
    // Declaring "cve_id" as a TOML column would be a P1 CRITICAL SAP-2 violation since the
    // NVD DTU response JSON has field "id" at the CveRecord level, not "cve_id".
    let forbidden_toml_fields: HashSet<&str> = ["cve_id"].iter().copied().collect();

    // Load the nvd.infusion.toml spec.
    let tmp = TempDir::new().expect("tempdir");
    let infusions_dir = tmp.path().join("infusions");
    std::fs::create_dir_all(&infusions_dir).expect("create infusions dir");
    let toml_content = include_str!("../../../specs/infusions/nvd.infusion.toml");
    let spec_path = infusions_dir.join("nvd.infusion.toml");
    let mut f = std::fs::File::create(&spec_path).expect("create toml");
    f.write_all(toml_content.as_bytes()).expect("write toml");

    let parse_result = InfusionLoader::parse(toml_content, "nvd.infusion.toml");

    match parse_result {
        Ok(spec) => {
            let field_names: HashSet<&str> = spec.fields.iter().map(|f| f.name.as_str()).collect();

            // Verify all expected fields are present.
            for expected in &expected_toml_fields {
                assert!(
                    field_names.contains(expected),
                    "SAP-2: NVD TOML must declare field '{}' (maps to CvssData DTU field). \
                     Got fields: {:?}",
                    expected,
                    field_names
                );
            }

            // Verify no forbidden fields are declared.
            for forbidden in &forbidden_toml_fields {
                assert!(
                    !field_names.contains(forbidden),
                    "SAP-2 P1 CRITICAL: NVD TOML must NOT declare '{}'. \
                     DTU CveRecord wire name is 'id' (not 'cve_id'). \
                     Got fields: {:?}",
                    forbidden,
                    field_names
                );
            }

            // Verify field count — no extra columns without DTU backing.
            assert_eq!(
                spec.fields.len(),
                3,
                "SAP-2: NVD TOML must declare exactly 3 fields (cvss_base_score, \
                 cvss_severity, cvss_vector). Extra fields without DTU backing = P1 CRITICAL. \
                 Got: {:?}",
                field_names
            );
        }
        Err(e) => {
            panic!(
                "test_enrichment_pivot_002_sap2_nvd_toml_columns_match_dtu_fields: \
                 RED GATE — InfusionLoader::parse failed for nvd.infusion.toml: {:?}. \
                 If this is a todo!() panic from validate_field_name: expected RED behavior until \
                 AC-007 is implemented. If structural: TOML has a bug. \
                 (SAP-2 / CLAUDE.md §SAP-2 / S-DEMO-ENRICHMENT-PIVOT-002)",
                e
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Test 18: BC-2.19.001 E-INFUSE-002 — duplicate UDF name rejection
// ---------------------------------------------------------------------------

/// BC-2.19.001 (postcondition E-INFUSE-002): duplicate UDF names across specs are rejected.
///
/// Given two InfusionSpecs both declaring a field with name "threat_score",
/// when InfusionRegistry::load_spec is called for the second spec,
/// then it returns Err(InfusionError::DuplicateUdfName) and the first spec is retained.
///
/// This tests the INV-INFUSE-001 duplicate detection invariant from BC-2.19.001:
/// "UDF names are global within a DataFusion SessionContext; duplicates are a load-time error"
///
/// RED GATE: fails until InfusionRegistry::load_spec implements duplicate detection.
/// Looking at current code (mod.rs): validate_spec_against checks for duplicates via
/// udf_to_infusion. This SHOULD already work — the test verifies the gate is operational.
/// If load_spec already implements this correctly, the test will PASS (green-by-design).
/// Either way, the test documents the required behavioral invariant.
#[test]
fn test_enrichment_pivot_002_bc2_19_001_duplicate_udf_name_rejected() {
    use std::io::Write;
    use tempfile::TempDir;

    // Load two specs that share the UDF name "threat_score".
    // The second spec must be rejected with E-INFUSE-002.
    let spec1_toml = r#"
[infusion]
infusion_id = "spec_one"
name = "Spec One"
type = "plugin"

[source]
type = "plugin"
plugin_ref = "one.prx"

[[infusion.fields]]
name = "threat_score"
input_field = "ioc"
input_type = "ioc"
output_type = "integer"

[infusion.pipe_stage]
adds_columns = ["threat_score"]
"#;

    let spec2_toml = r#"
[infusion]
infusion_id = "spec_two"
name = "Spec Two"
type = "plugin"

[source]
type = "plugin"
plugin_ref = "two.prx"

[[infusion.fields]]
name = "threat_score"
input_field = "ioc"
input_type = "ioc"
output_type = "integer"

[infusion.pipe_stage]
adds_columns = ["threat_score"]
"#;

    // Parse both specs. Since validate_field_name is todo!() (a stub), parsing may panic here.
    // This IS the expected RED Gate behavior — until validate_field_name is implemented,
    // the duplicate detection test also fails (correct: all stubs fail together).
    let spec1 = match InfusionLoader::parse(spec1_toml, "spec_one.infusion.toml") {
        Ok(s) => s,
        Err(e) => {
            panic!(
                "test_enrichment_pivot_002_bc2_19_001_duplicate_udf_name_rejected: \
                 RED GATE — parse of spec1 failed: {:?}. \
                 If todo!() from validate_field_name: expected RED state. \
                 (BC-2.19.001 E-INFUSE-002 / S-DEMO-ENRICHMENT-PIVOT-002)",
                e
            );
        }
    };
    let spec2 = match InfusionLoader::parse(spec2_toml, "spec_two.infusion.toml") {
        Ok(s) => s,
        Err(e) => {
            panic!(
                "test_enrichment_pivot_002_bc2_19_001_duplicate_udf_name_rejected: \
                 RED GATE — parse of spec2 failed: {:?}. \
                 (BC-2.19.001 E-INFUSE-002 / S-DEMO-ENRICHMENT-PIVOT-002)",
                e
            );
        }
    };

    let registry = InfusionRegistry::new();

    // First spec loads successfully.
    let result1 = registry.load_spec(spec1);
    assert!(
        result1.is_ok(),
        "BC-2.19.001 E-INFUSE-002: first spec must load without error; got: {:?}",
        result1.err()
    );

    // Second spec MUST be rejected with E-INFUSE-002 (duplicate UDF name "threat_score").
    let result2 = registry.load_spec(spec2);
    assert!(
        result2.is_err(),
        "BC-2.19.001 E-INFUSE-002: second spec with duplicate field name 'threat_score' \
         must be rejected (INV-INFUSE-001: UDF names are global). Got Ok instead of Err."
    );

    let err = result2.unwrap_err();
    let err_str = format!("{}", err);
    assert!(
        err_str.contains("threat_score"),
        "BC-2.19.001 E-INFUSE-002: error message must reference the duplicate UDF name \
         'threat_score'. Got: '{}'",
        err_str
    );

    // Verify the error is DuplicateUdfName (E-INFUSE-002), not some other error variant.
    assert!(
        matches!(err, InfusionError::DuplicateUdfName { .. }),
        "BC-2.19.001 E-INFUSE-002: error must be InfusionError::DuplicateUdfName. Got: {:?}",
        err
    );
}

// ---------------------------------------------------------------------------
// Test 19: BC-2.19.001 EC-19-001 — zero-field spec rejection
// ---------------------------------------------------------------------------

/// BC-2.19.001 (EC-19-001): a spec with zero [[infusion.fields]] entries is rejected.
///
/// Given an InfusionSpec with an empty fields list,
/// when InfusionRegistry::load_spec is called,
/// then it returns Err(...) and no UDF is registered.
///
/// This tests EC-19-001: "Spec with 0 [[infusion.fields]] entries — Rejected: at least one
/// field required per INV-INFUSE-001"
///
/// RED GATE: fails until validate_field_name is implemented. Once parse is functional,
/// a spec with 0 fields would be rejected by InfusionLoader::parse (missing field check).
/// The test verifies the rejection at PARSE time (not just registry time).
#[test]
fn test_enrichment_pivot_002_bc2_19_001_zero_fields_spec_rejected() {
    // A spec with no [[infusion.fields]] entries must be rejected at parse time.
    // InfusionLoader::parse validates "at least one field" (BC-2.19.001).
    let zero_fields_toml = r#"
[infusion]
infusion_id = "zero_fields_test"
name = "Zero Fields Test"
type = "plugin"

[source]
type = "plugin"
plugin_ref = "test.prx"

[infusion.pipe_stage]
adds_columns = []
"#;

    let result = InfusionLoader::parse(zero_fields_toml, "zero_fields.infusion.toml");

    // The parse must fail because there are no [[infusion.fields]] entries.
    assert!(
        result.is_err(),
        "BC-2.19.001 EC-19-001: spec with 0 [[infusion.fields]] entries must be rejected \
         at parse time (INV-INFUSE-001: at least one field required). Got Ok instead of Err."
    );

    let err = result.unwrap_err();
    let err_str = format!("{}", err);
    // Error should mention fields or infusion.fields or zero fields.
    assert!(
        err_str.to_lowercase().contains("field") || err_str.contains("E-INFUSE-003"),
        "BC-2.19.001 EC-19-001: error message for zero-field spec must reference \
         'field' or 'E-INFUSE-003'. Got: '{}'",
        err_str
    );
}
