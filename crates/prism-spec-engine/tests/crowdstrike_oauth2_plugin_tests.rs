#![allow(non_snake_case, dead_code, unused_imports)]
//! PLUGIN-MIGRATION-001-E Red Gate Tests (AC-001 through AC-010).
//!
//! All tests in this module MUST FAIL before implementation begins (Red Gate).
//! Each test body calls `todo!()` or drives a function that does — the tests
//! compile but panic at runtime, satisfying BC-5.38.001 Red Gate discipline.
//!
//! BCs covered: BC-2.01.016, BC-2.16.013, BC-2.17.001, BC-2.17.006, BC-2.17.007, BC-2.22.001
//! VPs covered: VP-148, VP-150
//!
//! Test naming per project convention: `test_BC_S_SS_NNN_xxx` or
//! `test_PLUGIN_MIGRATION_001_E_NNN_description` for story-scoped tests.

// ---------------------------------------------------------------------------
// Imports (needed once implementation is wired; declared now for structural completeness)
// ---------------------------------------------------------------------------

use std::collections::HashMap;
use std::sync::Arc;

// prism-spec-engine types needed for AC tests
use prism_spec_engine::plugin::{CURRENT_SUPPORTED_VERSION, PluginRuntime};
use prism_spec_engine::spec_parser::{SensorSpec, SpecLoader};

// ---------------------------------------------------------------------------
// Test utilities
// ---------------------------------------------------------------------------

/// Build a test `PluginRuntime` (no audit sink = NoOpPluginAuditSink).
fn build_test_runtime() -> PluginRuntime {
    PluginRuntime::new(reqwest::Client::new()).expect("PluginRuntime::new must succeed")
}

// ---------------------------------------------------------------------------
// AC-001: Plugin compiles and manifest passes WIT + schema validation
// Traces to: BC-2.17.007 §Postcondition, BC-2.17.006 §Postcondition
// ---------------------------------------------------------------------------

/// Red Gate Test 1: plugin loads successfully (manifest valid + WIT exports present).
///
/// After implementation: `PluginRuntime::load_plugin(crowdstrike_oauth2_path)` returns
/// `Ok(plugin)` with `plugin.metadata.plugin_id == "crowdstrike-oauth2"`.
///
/// Pre-implementation: panics at `todo!()` inside `acquire_token()` or similar stub.
#[test]
fn test_PLUGIN_MIGRATION_001_E_001_plugin_compiles_and_manifest_validates() {
    // Red Gate: this test will panic when the plugin binary is a todo!() stub.
    // Implementation path: build plugin to WASM, load via PluginRuntime::load_plugin,
    // assert Ok(plugin) with plugin_id = "crowdstrike-oauth2".
    todo!(
        "PLUGIN-MIGRATION-001-E AC-001: build plugin wasm binary and verify \
         PluginRuntime::load_plugin returns Ok with correct plugin_id (Task 4)"
    )
}

// ---------------------------------------------------------------------------
// AC-002: auth_type_name() returns canonical value
// Traces to: BC-2.01.016 §Postcondition — auth_type_name(); INV-AUTH-OPEN-003 Rule A
// ---------------------------------------------------------------------------

/// Red Gate Test 2: plugin WIT export `auth-type-name` returns "oauth2_client_credentials".
///
/// After implementation: load plugin, call via PluginRuntime WIT dispatch,
/// assert returned string == "oauth2_client_credentials".
///
/// Pre-implementation: panics at `todo!()` in `auth_type_name()`.
#[test]
fn test_PLUGIN_MIGRATION_001_E_002_auth_type_name_returns_oauth2_client_credentials() {
    todo!(
        "PLUGIN-MIGRATION-001-E AC-002: call plugin auth-type-name WIT export and \
         assert returned string == \"oauth2_client_credentials\" (Task 4)"
    )
}

// ---------------------------------------------------------------------------
// AC-003: Token acquisition via POST /oauth2/token against DTU clone
// Traces to: BC-2.01.016 §Preconditions; BC-2.16.013 §Postcondition
// ---------------------------------------------------------------------------

/// Red Gate Test 3: acquire_token calls DTU /oauth2/token and returns access_token.
///
/// After implementation: start CrowdStrike DTU clone, call plugin's acquire_token,
/// assert DTU received POST /oauth2/token and returned "dtu-fake-cs-token".
///
/// Pre-implementation: panics at `todo!()` in `acquire_token()`.
#[test]
fn test_PLUGIN_MIGRATION_001_E_003_acquire_token_calls_oauth2_token_endpoint() {
    todo!(
        "PLUGIN-MIGRATION-001-E AC-003: start DTU clone, call acquire_token, assert \
         DTU /oauth2/token received POST and returned access_token (Task 4)"
    )
}

// ---------------------------------------------------------------------------
// AC-004: Token cached within TTL; second call reuses cache (no second request)
// Traces to: BC-2.01.016 §Invariant — stateless trait; BC-2.17.001 KV state
// ---------------------------------------------------------------------------

/// Red Gate Test 4: get_token() within TTL reuses KV cache (no second HTTP request).
///
/// After implementation: acquire token, then call get_token() within TTL,
/// assert DTU /oauth2/token endpoint was called exactly ONCE.
///
/// Pre-implementation: panics at `todo!()` in `get_token()`.
#[test]
fn test_PLUGIN_MIGRATION_001_E_004_token_cached_within_ttl_no_second_request() {
    todo!(
        "PLUGIN-MIGRATION-001-E AC-004: acquire token, call get_token within TTL, \
         assert DTU /oauth2/token called exactly once (Task 5)"
    )
}

// ---------------------------------------------------------------------------
// AC-005: Expired token triggers re-acquisition
// Traces to: BC-2.01.016 §acquire_token() forced-refresh path
// ---------------------------------------------------------------------------

/// Red Gate Test 5: get_token() with expired cache triggers a second /oauth2/token call.
///
/// After implementation: acquire token, mutate KV store expires_at_secs to past,
/// call get_token(), assert DTU /oauth2/token called twice.
///
/// Pre-implementation: panics at `todo!()` in `get_token()`.
#[test]
fn test_PLUGIN_MIGRATION_001_E_005_expired_token_triggers_reacquisition() {
    todo!(
        "PLUGIN-MIGRATION-001-E AC-005: acquire token, expire cache, call get_token, \
         assert DTU /oauth2/token called twice (Task 5)"
    )
}

// ---------------------------------------------------------------------------
// AC-006: 401 response triggers token refresh + single retry via PipelineExecutor
// Traces to: BC-2.01.016 §acquire_token() forced-refresh; VP-150
// ---------------------------------------------------------------------------

/// Red Gate Test 6: PipelineExecutor 401-retry path calls plugin acquire_token for refresh.
///
/// After implementation: configure DTU single-401 injection, run PipelineExecutor::execute()
/// for CrowdStrike detections via plugin auth path, assert:
///   (a) final result is Ok(records) with OCSF output
///   (b) /oauth2/token called twice (initial + refresh)
///   (c) detection query endpoint called twice (401 + retry 200)
///
/// Pre-implementation: panics at `todo!()` in `acquire_token()`.
#[test]
fn test_PLUGIN_MIGRATION_001_E_006_401_triggers_plugin_token_refresh_and_retry() {
    todo!(
        "PLUGIN-MIGRATION-001-E AC-006: DTU 401 injection, PipelineExecutor execute(), \
         assert token refreshed and retry succeeded (Task 7)"
    )
}

// ---------------------------------------------------------------------------
// AC-007: crowdstrike.sensor.toml declares auth_plugin = "crowdstrike-oauth2"
// Traces to: BC-2.16.013 §Postcondition; ADR-028 §D2
// ---------------------------------------------------------------------------

/// Red Gate Test 7: crowdstrike.sensor.toml parsed by SpecLoader has auth_plugin set.
///
/// This test verifies the TOML amendment (Task 6) is correctly parsed. Since the TOML
/// has been amended in the stub phase, this test should be driven GREEN by the TOML
/// change + the SensorSpec::auth_plugin field addition (Task 1).
///
/// After TOML amendment: SpecLoader::load_all() parses crowdstrike.sensor.toml,
/// resulting SensorSpec.auth_plugin == Some("crowdstrike-oauth2").
/// SensorSpec.auth_type == AuthType::Oauth2ClientCredentials (unchanged).
///
/// NOTE: This test does NOT call todo!() — it tests the TOML field addition
/// which is in-scope for the stub architect phase (AC-007 / Task 1+6).
/// It will PASS after the SensorSpec::auth_plugin field and TOML are in place.
#[test]
fn test_PLUGIN_MIGRATION_001_E_007_crowdstrike_toml_declares_auth_plugin() {
    // This test exercises the TOML parse path directly — no plugin binary needed.
    // The crowdstrike.sensor.toml has been amended with auth_plugin = "crowdstrike-oauth2".
    // SensorSpec::auth_plugin field added with #[serde(default)].
    let toml_content = include_str!("../../prism-sensors/specs/crowdstrike.sensor.toml");
    let spec = prism_spec_engine::spec_parser::SpecLoader::parse(toml_content)
        .expect("crowdstrike.sensor.toml must parse without error");
    assert_eq!(
        spec.auth_plugin,
        Some("crowdstrike-oauth2".to_string()),
        "crowdstrike.sensor.toml must declare auth_plugin = \"crowdstrike-oauth2\" \
         (PLUGIN-MIGRATION-001-E AC-007 / BC-2.16.013)"
    );
    assert_eq!(
        spec.auth_type,
        prism_spec_engine::spec_parser::AuthType::Oauth2ClientCredentials,
        "auth_type must remain oauth2_client_credentials after amendment (ADR-028 §D2 LOCKED)"
    );
}

// ---------------------------------------------------------------------------
// AC-008: VP-148 parity test remains GREEN after TOML amendment
// Traces to: BC-2.16.013 §INV-PARITY-001; VP-148
// ---------------------------------------------------------------------------

/// Red Gate Test 8: existing CrowdStrike DTU parity test passes with plugin auth path.
///
/// After implementation: the parity test in tests/parity/crowdstrike.rs is updated
/// to use plugin auth provider instead of NullAuthProvider; OCSF output matches fixture.
///
/// Pre-implementation: panics at `todo!()` (plugin not yet wired to PipelineExecutor).
#[test]
fn test_PLUGIN_MIGRATION_001_E_008_vp148_parity_green_after_toml_amendment() {
    todo!(
        "PLUGIN-MIGRATION-001-E AC-008: run CrowdStrike DTU parity test with plugin \
         auth path, assert OCSF output matches reference fixture (Task 7)"
    )
}

// ---------------------------------------------------------------------------
// AC-009: Plugin loaded at boot step 7.5 emits plugin_load_unsigned WARN
// Traces to: BC-2.22.001 §Sequencing Invariant; PREREQ-D AC-4
// ---------------------------------------------------------------------------

/// Red Gate Test 9: PluginRuntime::load_all_plugins discovers crowdstrike-oauth2 plugin
/// and emits the expected `plugin_load_unsigned` WARN event at boot step 7.5.
///
/// After implementation: uses the PREREQ-D boot integration test harness pattern
/// (plugin_integration_tests.rs) to assert the plugin is discovered and the WARN
/// event includes plugin_id = "crowdstrike-oauth2".
///
/// Pre-implementation: panics at `todo!()` — plugin binary does not exist as a .prx file.
#[test]
fn test_PLUGIN_MIGRATION_001_E_009_plugin_loaded_at_boot_step_7_5_emits_warn() {
    todo!(
        "PLUGIN-MIGRATION-001-E AC-009: use PREREQ-D boot harness pattern to assert \
         crowdstrike-oauth2 plugin loaded at boot step 7.5 with plugin_load_unsigned WARN \
         (Task 8)"
    )
}

// ---------------------------------------------------------------------------
// AC-010: Credential opaqueness — token value not in tracing output
// Traces to: BC-2.01.016 §Postcondition — Debug safety; AD-017
// ---------------------------------------------------------------------------

/// Red Gate Test 10 (security): acquire_token() does not log the access_token value.
///
/// After implementation: captures tracing subscriber output during acquire_token()
/// against DTU clone; asserts the returned access_token string does NOT appear
/// verbatim in any captured log line (AD-017 AI-opaque credential model).
///
/// Pre-implementation: panics at `todo!()` in `acquire_token()`.
#[test]
fn test_PLUGIN_MIGRATION_001_E_010_token_not_in_tracing_output() {
    todo!(
        "PLUGIN-MIGRATION-001-E AC-010: capture tracing output during acquire_token, \
         assert access_token value does not appear in any log line (Task 8)"
    )
}

// ---------------------------------------------------------------------------
// Unit test: SensorSpec with no auth_plugin field parses to None (Task 1 backward compat)
// Traces to: BC-2.16.013 §Postcondition; backward-compat for existing TOML files
// ---------------------------------------------------------------------------

/// Task 1 unit test: existing TOML without auth_plugin parses SensorSpec.auth_plugin = None.
///
/// This test is NOT a todo!() — it exercises the serde(default) parsing path
/// that is already complete via the SensorSpec struct amendment.
/// It drives GREEN immediately after the auth_plugin field is added to SensorSpec.
#[test]
fn test_PLUGIN_MIGRATION_001_E_task1_sensor_spec_without_auth_plugin_parses_to_none() {
    // Minimal valid TOML without auth_plugin field — must parse with auth_plugin = None.
    // Includes a minimal tables entry to satisfy the required `tables` field.
    let toml_without_auth_plugin = r#"
sensor_id = "test-sensor"
name = "Test Sensor"
auth_type = "api_key"
base_url = "https://api.example.com"
version = "1.0.0"

[[tables]]
table_name = "items"
ocsf_class = "security_finding"

  [[tables.columns]]
  name = "id"
  column_type = "string"

  [[tables.steps]]
  name = "fetch_items"
  method = "GET"
  path_template = "/items"
  response_path = "$.resources"
  variables_produced = []
"#;
    let spec = prism_spec_engine::spec_parser::SpecLoader::parse(toml_without_auth_plugin)
        .expect("TOML without auth_plugin must parse successfully (backward compat)");
    assert_eq!(
        spec.auth_plugin, None,
        "SensorSpec.auth_plugin must be None when not declared in TOML \
         (PLUGIN-MIGRATION-001-E Task 1 / #[serde(default)])"
    );
}
