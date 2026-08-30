//! Red Gate tests for S-DEMO-CROWDSTRIKE-MULTIREGION-001 —
//! CrowdStrike multi-region base_url fidelity via `${env.CROWDSTRIKE_BASE_URL}`.
//!
//! # Story
//! S-DEMO-CROWDSTRIKE-MULTIREGION-001 v1.4 — wave 5, P2.
//!
//! # Red Gate status
//! ALL tests in this file MUST FAIL before the implementer:
//!   (a) changes `crowdstrike.sensor.toml` base_url from
//!       `"https://api.crowdstrike.com"` to `"${env.CROWDSTRIKE_BASE_URL}"`, AND
//!   (b) verifies that the env-var resolver (already implemented in S-SPEC-ENV-VAR-001)
//!       handles the missing-env case with E-SPEC-024 on the CrowdStrike spec.
//!
//! # Why tests fail before implementation
//! The current `crowdstrike.sensor.toml` contains:
//!   `base_url = "https://api.crowdstrike.com"`  (hardcoded us-1)
//!
//! - `test_BC_2_16_013_crowdstrike_eu1_base_url_env_var_resolves_correctly`:
//!   FAILS because the loaded spec.base_url == "https://api.crowdstrike.com"
//!   (hardcoded), NOT "https://api.eu-1.crowdstrike.com" (the env var value).
//!   The assertion `spec.base_url == eu-1 URL` therefore fails.
//!
//! - `test_BC_2_16_013_crowdstrike_base_url_env_unset_returns_spec_error_not_panic`:
//!   FAILS because the current TOML contains no `${env.CROWDSTRIKE_BASE_URL}` token,
//!   so `parse_and_validate_spec_toml` SUCCEEDS (no env token to resolve → no E-SPEC-024).
//!   The assertion `result.is_err()` therefore fails.
//!
//! - `test_BC_2_16_013_crowdstrike_base_url_env_points_to_local_dtu_demo_works`:
//!   FAILS because the loaded spec.base_url == "https://api.crowdstrike.com"
//!   (hardcoded), NOT the DTU address set via CROWDSTRIKE_BASE_URL.
//!   The assertion `spec.base_url == dtu_addr` therefore fails.
//!
//! # AC mapping
//!
//! | Test | AC | BC | Description |
//! |------|----|----|-------------|
//! | `test_BC_2_16_013_crowdstrike_eu1_base_url_env_var_resolves_correctly` | AC-002 | BC-2.16.013 §Postconditions §1 | Set CROWDSTRIKE_BASE_URL=eu-1 URL; spec.base_url resolves to eu-1 |
//! | `test_BC_2_16_013_crowdstrike_base_url_env_unset_returns_spec_error_not_panic` | AC-003 | BC-2.16.009 §VR6 E-SPEC-024 | Unset CROWDSTRIKE_BASE_URL; load returns Err(EnvVarNotSet), NOT panic |
//! | `test_BC_2_16_013_crowdstrike_base_url_env_points_to_local_dtu_demo_works` | AC-004 | BC-2.16.013 §Postconditions §2 | Set CROWDSTRIKE_BASE_URL to local DTU addr; spec.base_url resolves to DTU addr |
//!
//! # Env var isolation
//! - Each test uses `CROWDSTRIKE_BASE_URL` (the production env var per story AC-001).
//! - Tests that `set_var` always `remove_var` in cleanup at the end.
//! - Tests that require the var to be absent call `remove_var` at both start and end.
//! - nextest runs each integration test binary in a forked process for process-level
//!   isolation. The `unsafe set_var/remove_var` calls are safe within these tests:
//!   each test binary forks before running, and `CROWDSTRIKE_BASE_URL` is only
//!   written in this test file.
//!
//! BC-2.16.013 §Postconditions §1 (CrowdStrike spec authoring fidelity);
//! BC-2.16.009 §Validation Rules 6 (env-var resolver) + E-SPEC-024;
//! S-DEMO-CROWDSTRIKE-MULTIREGION-001; ADR-031 §D8-c.

use prism_spec_engine::add_sensor_spec::parse_and_validate_spec_toml;

// ---------------------------------------------------------------------------
// Test helpers
// ---------------------------------------------------------------------------

/// Absolute path to the production `crowdstrike.sensor.toml` file.
///
/// Uses `CARGO_MANIFEST_DIR` (the `prism-spec-engine` crate root) to navigate
/// to the sibling `prism-sensors/specs/` directory.
fn crowdstrike_sensor_toml_path() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../prism-sensors/specs/crowdstrike.sensor.toml")
}

/// Read the production `crowdstrike.sensor.toml` content.
///
/// Panics (test setup failure) if the file cannot be read.
fn read_crowdstrike_toml() -> String {
    let path = crowdstrike_sensor_toml_path();
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("crowdstrike.sensor.toml must be readable at {path:?}: {e}"))
}

// ---------------------------------------------------------------------------
// Red Gate test 1 / AC-002:
// Set CROWDSTRIKE_BASE_URL=https://api.eu-1.crowdstrike.com → spec.base_url
// must equal the eu-1 URL after parse_and_validate_spec_toml.
//
// Fails before implementation because the current crowdstrike.sensor.toml has:
//   base_url = "https://api.crowdstrike.com"   (hardcoded us-1)
// There is no ${env.CROWDSTRIKE_BASE_URL} token, so the env resolver is a no-op,
// and spec.base_url stays "https://api.crowdstrike.com" — NOT the eu-1 URL.
// The assertion `spec.base_url == EU1_URL` therefore fails.
//
// After implementation (crowdstrike.sensor.toml has base_url = "${env.CROWDSTRIKE_BASE_URL}"):
// The resolver replaces the token with the env var value and the assertion passes.
//
// Traces to: BC-2.16.013 §Postconditions §1 (CrowdStrike spec authoring fidelity);
//            BC-2.16.009 EC-009-008 (eu-1 URL happy path);
//            S-DEMO-CROWDSTRIKE-MULTIREGION-001 AC-002.
// ---------------------------------------------------------------------------

/// test_BC_2_16_013_crowdstrike_eu1_base_url_env_var_resolves_correctly (AC-002)
///
/// Precondition: CROWDSTRIKE_BASE_URL is set to the eu-1 base URL.
/// Expected:     parse_and_validate_spec_toml succeeds;
///               spec.base_url == "https://api.eu-1.crowdstrike.com".
///
/// RED GATE FAILURE REASON: crowdstrike.sensor.toml still has
/// `base_url = "https://api.crowdstrike.com"` (hardcoded us-1).
/// No ${env.CROWDSTRIKE_BASE_URL} token exists → resolver is a no-op →
/// spec.base_url remains "https://api.crowdstrike.com" ≠ eu-1 URL.
#[test]
fn test_BC_2_16_013_crowdstrike_eu1_base_url_env_var_resolves_correctly() {
    // AC-002: BC-2.16.013 §Postconditions §1 — eu-1 URL resolves from env var.
    // EC-009-008: eu-1 URL happy path.
    const VAR: &str = "CROWDSTRIKE_BASE_URL";
    const EU1_URL: &str = "https://api.eu-1.crowdstrike.com";

    // SAFETY: std::env::set_var is unsafe in Rust 2024 in multithreaded contexts.
    // nextest runs each integration test binary in a forked process, providing per-test
    // process isolation. CROWDSTRIKE_BASE_URL is only written in this test file and
    // the tests run in separate binaries (cargo nextest fork model).
    unsafe {
        std::env::set_var(VAR, EU1_URL);
    }

    let toml_content = read_crowdstrike_toml();
    let source_path = "crates/prism-sensors/specs/crowdstrike.sensor.toml";
    let result = parse_and_validate_spec_toml(&toml_content, source_path);

    // Cleanup before asserting (ensures cleanup even if assertions panic).
    unsafe {
        std::env::remove_var(VAR);
    }

    // Postcondition 1: spec must load without errors when env var is set.
    let spec = result.unwrap_or_else(|errs| {
        panic!(
            "AC-002: parse_and_validate_spec_toml must succeed when CROWDSTRIKE_BASE_URL \
             is set to '{}'. Got {} error(s): {:#?}",
            EU1_URL,
            errs.len(),
            errs
        )
    });

    // Postcondition 2: spec.base_url must equal the eu-1 URL (resolved from env var).
    // RED GATE: This assertion FAILS before implementation because spec.base_url ==
    // "https://api.crowdstrike.com" (hardcoded), not EU1_URL.
    assert_eq!(
        spec.base_url, EU1_URL,
        "AC-002 (BC-2.16.013): spec.base_url must equal the eu-1 URL resolved from \
         CROWDSTRIKE_BASE_URL. \
         Expected: '{}'. Actual: '{}'. \
         If actual is 'https://api.crowdstrike.com', the TOML still has the hardcoded \
         us-1 URL — change base_url to \"${{env.CROWDSTRIKE_BASE_URL}}\" (story AC-001).",
        EU1_URL, spec.base_url,
    );

    // Postcondition 3: auth_type and auth_plugin must be unchanged (D-747 LOCKED).
    use prism_spec_engine::spec_parser::AuthType;
    assert_eq!(
        spec.auth_type,
        AuthType::Oauth2ClientCredentials,
        "AC-005 / D-747 LOCKED: auth_type must remain oauth2_client_credentials. \
         Got: {:?}",
        spec.auth_type,
    );
    assert_eq!(
        spec.auth_plugin.as_deref(),
        Some("crowdstrike-oauth2"),
        "AC-005 / D-747 LOCKED: auth_plugin must remain 'crowdstrike-oauth2'. \
         Got: {:?}",
        spec.auth_plugin,
    );
}

// ---------------------------------------------------------------------------
// Red Gate test 2 / AC-003:
// Unset CROWDSTRIKE_BASE_URL → parse_and_validate_spec_toml must return
// Err(ValidationError) containing an E-SPEC-024 / EnvVarNotSet error for
// CROWDSTRIKE_BASE_URL. Must NOT panic.
//
// Fails before implementation because the current crowdstrike.sensor.toml has:
//   base_url = "https://api.crowdstrike.com"   (hardcoded — no ${env.VAR} token)
// There is no env token to resolve, so the load SUCCEEDS even when
// CROWDSTRIKE_BASE_URL is unset. The assertion `result.is_err()` therefore fails.
//
// After implementation (crowdstrike.sensor.toml has base_url = "${env.CROWDSTRIKE_BASE_URL}"):
// The resolver encounters the token, CROWDSTRIKE_BASE_URL is unset, emits E-SPEC-024,
// and returns Err. The assertion passes.
//
// Traces to: BC-2.16.009 §Validation Rules 6 (env-var resolver) + E-SPEC-024;
//            EC-009-009 (CROWDSTRIKE_BASE_URL unset → E-SPEC-024);
//            S-DEMO-CROWDSTRIKE-MULTIREGION-001 AC-003.
// ---------------------------------------------------------------------------

/// test_BC_2_16_013_crowdstrike_base_url_env_unset_returns_spec_error_not_panic (AC-003)
///
/// Precondition: CROWDSTRIKE_BASE_URL is NOT set.
/// Expected:     parse_and_validate_spec_toml returns Err containing an error
///               message that references "CROWDSTRIKE_BASE_URL" (E-SPEC-024).
///               It does NOT panic.
///
/// RED GATE FAILURE REASON: crowdstrike.sensor.toml has no ${env.CROWDSTRIKE_BASE_URL}
/// token. The env resolver is a no-op → the load succeeds → result.is_err() is false.
#[test]
fn test_BC_2_16_013_crowdstrike_base_url_env_unset_returns_spec_error_not_panic() {
    // AC-003: BC-2.16.009 §VR6 E-SPEC-024 — missing CROWDSTRIKE_BASE_URL → error, not panic.
    // EC-009-009: CROWDSTRIKE_BASE_URL unset at spec-load time.
    const VAR: &str = "CROWDSTRIKE_BASE_URL";

    // Ensure the var is absent (defensive: remove in case of prior test pollution).
    unsafe {
        std::env::remove_var(VAR);
    }

    let toml_content = read_crowdstrike_toml();
    let source_path = "crates/prism-sensors/specs/crowdstrike.sensor.toml";

    // The load must not panic — if it does, the test itself panics and reports as FAIL.
    // This test catches panics implicitly because Rust tests report panics as failures.
    let result = parse_and_validate_spec_toml(&toml_content, source_path);

    // Cleanup (no-op when already absent, but symmetric with set-based tests).
    unsafe {
        std::env::remove_var(VAR);
    }

    // Postcondition 1: the load must return Err (spec is rejected when env var is absent).
    // RED GATE: This assertion FAILS before implementation because there is no
    // ${env.CROWDSTRIKE_BASE_URL} token → the load succeeds → result.is_ok().
    let errs = result.unwrap_err_or_else_fail(
        "AC-003 (BC-2.16.009 E-SPEC-024): parse_and_validate_spec_toml must return Err \
         when CROWDSTRIKE_BASE_URL is unset. \
         If the load succeeded, crowdstrike.sensor.toml still has the hardcoded URL \
         instead of \"${env.CROWDSTRIKE_BASE_URL}\" — story AC-001 fix not yet applied.",
    );

    // Postcondition 2: the combined error text must reference CROWDSTRIKE_BASE_URL.
    // This confirms the error is E-SPEC-024 (env var not set), not some other error.
    let combined_error_text: String = errs
        .iter()
        .flat_map(|ve| ve.errors.iter().map(|s| s.as_str()))
        .collect::<Vec<_>>()
        .join("; ");

    assert!(
        combined_error_text.contains(VAR),
        "AC-003 (BC-2.16.009 E-SPEC-024): the error message must reference \
         '{}' so the operator knows which env var to configure. \
         Got error text: '{}'",
        VAR,
        combined_error_text,
    );

    // NOTE — why there is no Postcondition 3 ordering assertion here:
    //
    // The `parse_and_validate_spec_toml` SUT (add_sensor_spec.rs) does NOT call
    // `validate_sensor_spec`. The "must start with http" / E-SPEC-001 URL-format
    // error is produced exclusively by `validate_sensor_spec`. Therefore, for a
    // fully-tokenized `base_url = "${env.CROWDSTRIKE_BASE_URL}"` with the var unset,
    // `parse_and_validate_spec_toml` can only ever yield E-SPEC-024 (EnvVarNotSet)
    // on this code path — "must start with http" is structurally unreachable here,
    // making any `!contains("must start with http")` assertion inert (it cannot
    // distinguish correct from incorrect ordering because the wrong branch is
    // unreachable regardless of ordering).
    //
    // The BC-2.16.009 §VR6 env-resolver-before-URL-validation ORDERING invariant IS
    // load-bearing tested, but in the correct SUT scope (S-SPEC-ENV-VAR-001):
    //   - `test_env_var_resolution_runs_before_url_format_validation` in
    //     tests/env_var_resolution_tests.rs — calls `validate_sensor_spec` directly
    //     and confirms the error is E-SPEC-024, not E-SPEC-001.
    //   - `test_env_var_ordering_production_path_absent_var_produces_e_spec_024_not_url_format_error`
    //     in tests/env_var_resolution_tests.rs — exercises the production path and
    //     verifies the same ordering guarantee.
    //
    // Postconditions 1 (is_err) and 2 (error references CROWDSTRIKE_BASE_URL) above
    // ARE load-bearing for this test's scope and must not be removed.
}

// ---------------------------------------------------------------------------
// Red Gate test 3 / AC-004:
// Set CROWDSTRIKE_BASE_URL=http://127.0.0.1:<port> (DTU address) → spec.base_url
// must equal the DTU address after parse_and_validate_spec_toml.
//
// This proves the env-var pattern does not break the DTU demo path:
// when CROWDSTRIKE_BASE_URL points to the local DTU clone, the spec loads
// correctly with the DTU address as base_url.
//
// ADR-031 §D8-c EC-005: DTU operates over plain HTTP (loopback) — http:// is valid
// for the loopback fixture even though production URLs use https://.
//
// Fails before implementation because the current crowdstrike.sensor.toml has:
//   base_url = "https://api.crowdstrike.com"   (hardcoded — no ${env.VAR} token)
// There is no env token to resolve, so spec.base_url stays the hardcoded URL
// regardless of CROWDSTRIKE_BASE_URL. The assertion `spec.base_url == dtu_addr` fails.
//
// After implementation (crowdstrike.sensor.toml has base_url = "${env.CROWDSTRIKE_BASE_URL}"):
// The resolver replaces the token with the DTU address and the assertion passes.
//
// Traces to: BC-2.16.013 §Postconditions §2 (DTU parity; DTU region-agnostic per ADR-031 §D8-c);
//            S-DEMO-CROWDSTRIKE-MULTIREGION-001 AC-004; EC-005 (http DTU loopback).
// ---------------------------------------------------------------------------

/// test_BC_2_16_013_crowdstrike_base_url_env_points_to_local_dtu_demo_works (AC-004)
///
/// Precondition: CROWDSTRIKE_BASE_URL is set to a local DTU address
///               (http://127.0.0.1:9999 used as a representative port — no actual
///               DTU connection is made; this test verifies spec LOADING only).
/// Expected:     parse_and_validate_spec_toml succeeds;
///               spec.base_url == "http://127.0.0.1:9999".
///
/// RED GATE FAILURE REASON: crowdstrike.sensor.toml has no ${env.CROWDSTRIKE_BASE_URL}
/// token. The env resolver is a no-op → spec.base_url stays the hardcoded URL →
/// the assertion `spec.base_url == dtu_addr` fails.
///
/// Note: AC-004 (story) requires the pipeline to CONNECT to the DTU. That connection
/// test depends on the DTU clone running and is gated behind #[ignore] per SID-1.
/// This load-path test (non-#[ignore]) verifies the spec LOADING behavior without
/// requiring a live DTU — it proves env var substitution works for the DTU address.
/// A separate #[ignore]'d test (below) covers the full pipeline connection.
#[test]
fn test_BC_2_16_013_crowdstrike_base_url_env_points_to_local_dtu_demo_works() {
    // AC-004: BC-2.16.013 §Postconditions §2 — DTU address resolves from env var.
    // EC-005: http DTU loopback address (plain HTTP, not https, is valid for loopback).
    const VAR: &str = "CROWDSTRIKE_BASE_URL";
    // Use a representative loopback DTU address.
    // No actual DTU needs to be running — this test validates spec LOADING only.
    // The URL uses http:// (not https://) per ADR-031 §D8-c EC-005 (DTU loopback uses plain HTTP).
    const DTU_ADDR: &str = "http://127.0.0.1:9999";

    unsafe {
        std::env::set_var(VAR, DTU_ADDR);
    }

    let toml_content = read_crowdstrike_toml();
    let source_path = "crates/prism-sensors/specs/crowdstrike.sensor.toml";
    let result = parse_and_validate_spec_toml(&toml_content, source_path);

    // Cleanup before asserting.
    unsafe {
        std::env::remove_var(VAR);
    }

    // Postcondition 1: spec must load without errors when env var is set.
    let spec = result.unwrap_or_else(|errs| {
        panic!(
            "AC-004: parse_and_validate_spec_toml must succeed when CROWDSTRIKE_BASE_URL \
             is set to '{}' (DTU loopback address). Got {} error(s): {:#?}",
            DTU_ADDR,
            errs.len(),
            errs
        )
    });

    // Postcondition 2: spec.base_url must equal the DTU address (resolved from env var).
    // RED GATE: This assertion FAILS before implementation because spec.base_url ==
    // "https://api.crowdstrike.com" (hardcoded), not DTU_ADDR.
    assert_eq!(
        spec.base_url, DTU_ADDR,
        "AC-004 (BC-2.16.013 §Postconditions §2): spec.base_url must equal the DTU \
         loopback address resolved from CROWDSTRIKE_BASE_URL. \
         Expected: '{}'. Actual: '{}'. \
         If actual is 'https://api.crowdstrike.com', the TOML still has the hardcoded \
         us-1 URL — change base_url to \"${{env.CROWDSTRIKE_BASE_URL}}\" (story AC-001).",
        DTU_ADDR, spec.base_url,
    );

    // Postcondition 3: auth_type and auth_plugin must be unchanged (D-747 LOCKED).
    use prism_spec_engine::spec_parser::AuthType;
    assert_eq!(
        spec.auth_type,
        AuthType::Oauth2ClientCredentials,
        "AC-005 / D-747 LOCKED: auth_type must remain oauth2_client_credentials. \
         Got: {:?}",
        spec.auth_type,
    );
    assert_eq!(
        spec.auth_plugin.as_deref(),
        Some("crowdstrike-oauth2"),
        "AC-005 / D-747 LOCKED: auth_plugin must remain 'crowdstrike-oauth2'. \
         Got: {:?}",
        spec.auth_plugin,
    );
}

// ---------------------------------------------------------------------------
// DTU pipeline connection test (AC-004 full pipeline path) — #[ignore]'d
//
// This companion test exercises the full pipeline connection to the DTU clone
// when CROWDSTRIKE_BASE_URL points to the running DTU address. It is #[ignore]'d
// per SID-1: the DTU clone (prism-dtu-crowdstrike) must be running for this test
// to exercise its full behavior. The non-#[ignore]'d test above verifies the
// spec-loading aspect of AC-004 without requiring the DTU.
//
// Remove the #[ignore] tag after confirming the DTU clone is available in CI.
// Per SID-1: This #[ignore] tag is justified because it depends on the running DTU
// clone. The non-ignored test_BC_2_16_013_crowdstrike_base_url_env_points_to_local_dtu_demo_works
// above provides the unit-level coverage of the spec-load behavior.
// DTU-EXT-001: requires prism-dtu-crowdstrike running; ungated in CI after S-6.07 merges.
// ---------------------------------------------------------------------------

/// AC-004 DTU pipeline connection test.
///
/// When CROWDSTRIKE_BASE_URL is set to the running DTU address, the spec loads
/// and the pipeline can complete an OAuth2 token exchange + detection fetch
/// without error. This proves the env-var pattern does not break the DTU demo path.
///
/// #[ignore]'d per SID-1: requires prism-dtu-crowdstrike DTU clone running.
/// The non-#[ignore]'d test above covers the spec-load aspect of AC-004.
#[ignore = "DTU-EXT-001: requires prism-dtu-crowdstrike DTU clone running; \
             ungated after S-6.07 merges. Unit coverage of spec-load behavior \
             provided by test_BC_2_16_013_crowdstrike_base_url_env_points_to_local_dtu_demo_works."]
#[tokio::test]
async fn test_BC_2_16_013_crowdstrike_base_url_dtu_pipeline_connection_succeeds() {
    use std::collections::HashMap;

    use prism_core::OrgSlug;
    use prism_dtu_common::BehavioralClone;
    use prism_dtu_crowdstrike::CrowdstrikeClone;
    use prism_spec_engine::NullAuthProvider;
    use prism_spec_engine::pipeline::{FetchContext, PipelineExecutor};

    // AC-004 (DTU path): BC-2.16.013 §Postconditions §2 — pipeline connects to DTU
    // when CROWDSTRIKE_BASE_URL points to the local DTU clone address.
    const VAR: &str = "CROWDSTRIKE_BASE_URL";

    // Step 1: Start the CrowdStrike DTU clone on an ephemeral port.
    let mut clone = CrowdstrikeClone::new();
    let bound_addr = clone
        .start_on("127.0.0.1:0".parse().unwrap(), None, None)
        .await
        .expect("CrowdStrike DTU clone must start for AC-004 DTU pipeline test");
    let dtu_base_url = format!("http://{bound_addr}");

    // Step 2: Set CROWDSTRIKE_BASE_URL to the DTU address.
    unsafe {
        std::env::set_var(VAR, &dtu_base_url);
    }

    // Step 3: Load the spec via parse_and_validate_spec_toml (the production load path).
    // This verifies that CROWDSTRIKE_BASE_URL is resolved from the env var, not hardcoded.
    let toml_content = read_crowdstrike_toml();
    let source_path = "crates/prism-sensors/specs/crowdstrike.sensor.toml";
    let spec = parse_and_validate_spec_toml(&toml_content, source_path).unwrap_or_else(|errs| {
        unsafe {
            std::env::remove_var(VAR);
        }
        panic!(
            "AC-004 DTU: parse_and_validate_spec_toml must succeed when \
                 CROWDSTRIKE_BASE_URL='{}'. Got: {:#?}",
            dtu_base_url, errs
        )
    });

    // Cleanup env var before asserting.
    unsafe {
        std::env::remove_var(VAR);
    }

    // Step 4: Verify spec.base_url resolved to the DTU address.
    assert_eq!(
        spec.base_url, dtu_base_url,
        "AC-004 DTU: spec.base_url must equal the DTU address resolved from CROWDSTRIKE_BASE_URL. \
         Expected: '{}'. Actual: '{}'.",
        dtu_base_url, spec.base_url,
    );

    // Step 5: Execute the detections pipeline against the DTU.
    let table = spec
        .tables
        .iter()
        .find(|t| t.table_name == "detections")
        .expect("crowdstrike spec must declare a 'detections' table (AC-004 DTU)");

    let context = FetchContext::new(OrgSlug::new("test-org"), HashMap::new(), None);
    let http_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("reqwest client with 30s timeout (CLAUDE.md §Conventions) must build");

    let auth_provider = NullAuthProvider;
    let result = PipelineExecutor::execute(&spec, table, &context, &http_client, &auth_provider)
        .await
        .expect(
            "AC-004 DTU: PipelineExecutor::execute must succeed when CROWDSTRIKE_BASE_URL \
             points to the local DTU clone (ADR-031 §D8-c — DTU is region-agnostic)",
        );

    // Step 6: At least 2 requests (QueryV2 + PostEntities — the CrowdStrike 2-step pipeline).
    assert!(
        result.request_count >= 2,
        "AC-004 DTU: CrowdStrike two-step pipeline must issue >= 2 requests to the DTU \
         (QueryV2 GET + PostEntities POST). Got request_count = {}.",
        result.request_count,
    );
}

// ---------------------------------------------------------------------------
// Helper trait to produce test-friendly error messages
// ---------------------------------------------------------------------------

/// Extension trait that produces a test-friendly error message when `is_ok()`
/// but `is_err()` was expected (AC-003 case).
trait UnwrapErrOrElseFail<T, E> {
    fn unwrap_err_or_else_fail(self, msg: &str) -> E;
}

impl<T: std::fmt::Debug, E> UnwrapErrOrElseFail<T, E> for Result<T, E> {
    fn unwrap_err_or_else_fail(self, msg: &str) -> E {
        match self {
            Err(e) => e,
            Ok(v) => panic!(
                "{msg}\n\n(The load SUCCEEDED with value: {v:?} — but an error was expected.)"
            ),
        }
    }
}
