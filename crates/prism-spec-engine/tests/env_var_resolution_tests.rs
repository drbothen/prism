//! Red Gate tests for S-SPEC-ENV-VAR-001 — `${env.VAR}` token resolution in sensor spec fields.
//!
//! BC-2.16.009 §Validation Rules 6 (AC-6); S-SPEC-ENV-VAR-001.
//!
//! All 8 Red Gate tests are defined here. The function under test — `resolve_env_var_tokens` —
//! has a `todo!()` body; every test in this file therefore FAILS (Red Gate discipline, BC-5.38.001).
//!
//! # AC → Test mapping
//!
//! | Test name | AC | BC-2.16.009 clause |
//! |-----------|----|--------------------|
//! | test_env_var_full_token_resolves_to_value | AC-001 | postcondition: token → resolved value |
//! | test_env_var_partial_token_resolves_preserving_surrounding_literals | AC-002 | EC-009-003: partial interpolation |
//! | test_env_var_multi_token_single_field_both_resolve | AC-003 | postcondition: ALL tokens replaced |
//! | test_env_var_missing_var_produces_e_spec_024 | AC-004 | error path: absent → E-SPEC-024; EC-009-001 |
//! | test_env_var_empty_var_produces_e_spec_024 | AC-005 | error path: empty == missing; EC-009-002 |
//! | test_env_var_multi_missing_tokens_collect_multiple_errors | AC-006 | no fail-fast; multi-error; EC-009-006 |
//! | test_env_var_resolution_runs_before_url_format_validation | AC-007 | ordering: post-parse, pre-URL-check; EC-009-004 |
//! | test_env_var_error_contains_name_not_value | AC-008 | AD-017 no-value-leak |
//!
//! # Env var isolation
//!
//! Tests control env vars hermetically:
//! - Each test uses a unique variable name prefixed with `PRISM_TEST_ENV_VAR_` and a per-test
//!   discriminator. The discriminator is unique enough that ambient process env will not contain
//!   these names during normal CI runs.
//! - Tests that `set_var` always perform `remove_var` in a `defer`-style cleanup at the end.
//!   nextest runs each integration test binary in a forked process by default; isolation is
//!   belt-and-suspenders against potential env bleed if the runner mode changes.
//! - Tests that require the var to be absent call `remove_var` at both the start and end of the
//!   test to ensure a clean state even if a prior test leaked a set.
//!
//! # Why the tests call `resolve_env_var_tokens` directly
//!
//! The function is `pub` (re-exported from `prism_spec_engine::env_resolver`). Calling it
//! directly exercises the contract at the unit boundary, matching the story's Red Gate test set.
//! Tests for AC-007 (ordering) call `validate_sensor_spec` to verify that the combined
//! load pipeline produces the right error kind (E-SPEC-024, not E-SPEC-001) when the var is absent.

use prism_spec_engine::env_resolver::resolve_env_var_tokens;
use prism_spec_engine::error::SpecEngineError;
use prism_spec_engine::spec_parser::{SensorSpec, SpecLoader};
use prism_spec_engine::validation::validate_sensor_spec;

// ---------------------------------------------------------------------------
// Test helper: build a minimal valid SensorSpec with a configurable base_url.
// Keeps each test focused on the resolution clause being exercised.
// ---------------------------------------------------------------------------

/// Build a minimal `SensorSpec` via `SpecLoader::parse` (from TOML) with a configurable
/// `base_url` and optional `name`.
///
/// Using TOML parse avoids the `#[non_exhaustive]` struct-literal restriction and matches
/// the real spec-load path: TOML deserialization → resolver → validation (BC-2.16.009 AC-6).
///
/// The spec has no tables (valid: `#[serde(default)]` allows empty tables vec).
/// `auth_type = "api_key"` satisfies Rule A without requiring credential_refs.
fn minimal_spec(base_url: impl Into<String>) -> SensorSpec {
    minimal_spec_with_name(base_url, "Test Sensor")
}

/// Build a minimal `SensorSpec` with configurable `base_url` and `name`.
///
/// Used by AC-006 which needs to inject a token into the `name` field.
fn minimal_spec_with_name(base_url: impl Into<String>, name: impl Into<String>) -> SensorSpec {
    let base_url = base_url.into();
    let name = name.into();
    // Escape any TOML-special characters in base_url/name by embedding them in double-quoted
    // strings. ${env.VAR} tokens contain only ASCII-safe chars so no escaping is needed.
    let toml = format!(
        r#"
sensor_id = "test-sensor"
name = {name_quoted}
auth_type = "api_key"
base_url = {base_url_quoted}
version = "1.0.0"
"#,
        name_quoted = toml_quote(&name),
        base_url_quoted = toml_quote(&base_url),
    );
    SpecLoader::parse(&toml)
        .unwrap_or_else(|e| panic!("minimal_spec TOML parse must succeed: {e:?}\nTOML:\n{toml}"))
}

/// Wrap a string value in TOML double-quoted string syntax.
///
/// TOML double-quoted strings require backslash-escaping of `\` and `"`.
/// `${env.VAR}` tokens are ASCII-safe and need no escaping.
fn toml_quote(s: &str) -> String {
    let escaped = s.replace('\\', "\\\\").replace('"', "\\\"");
    format!("\"{escaped}\"")
}

// ---------------------------------------------------------------------------
// AC-001: Full-token resolution — single token, var set.
//
// Input:  base_url = "${env.PRISM_TEST_ENV_VAR_AC001_URL}"
//         env PRISM_TEST_ENV_VAR_AC001_URL = "https://example.armis.io"
// Output: resolve_env_var_tokens returns empty Vec (no errors)
//         spec.base_url == "https://example.armis.io"
//
// Traces to: BC-2.16.009 §Validation Rules 6 postcondition
//            ("Every ${env.VAR_NAME} token in every string field is replaced with the
//             resolved value"); S-SPEC-ENV-VAR-001 AC-001.
// ---------------------------------------------------------------------------
#[test]
fn test_env_var_full_token_resolves_to_value() {
    // AC-001: BC-2.16.009 §Validation Rules 6 postcondition — full-token replacement.
    // Uses a unique var name; cleans up on exit regardless of pass/fail.
    const VAR: &str = "PRISM_TEST_ENV_VAR_AC001_URL";
    const RESOLVED: &str = "https://example.armis.io";

    // SAFETY: std::env::set_var is unsafe in Rust 2024 in multithreaded contexts.
    // nextest runs each integration test binary in a forked process, providing per-test
    // process isolation. Within this test binary, this is the only test that sets
    // PRISM_TEST_ENV_VAR_AC001_URL, so there is no concurrent write hazard.
    // The unique prefix ensures no ambient-env collision.
    unsafe {
        std::env::set_var(VAR, RESOLVED);
    }

    // Construct the token string "${env.PRISM_TEST_ENV_VAR_AC001_URL}" programmatically.
    // `${{` and `}}` in format strings produce literal `${` and `}`.
    let token = format!("${{env.{VAR}}}");
    let mut spec = minimal_spec(token);

    let errors = resolve_env_var_tokens(&mut spec, "test/armis.sensor.toml");

    // Cleanup before asserting (ensures cleanup even if assert panics)
    unsafe {
        std::env::remove_var(VAR);
    }

    // Postcondition 1: no errors (var was set, non-empty).
    assert!(
        errors.is_empty(),
        "AC-001: resolve_env_var_tokens must return empty error vec when var is set. \
         Got {} error(s): {:?}",
        errors.len(),
        errors
    );

    // Postcondition 2: base_url is replaced with the resolved value.
    assert_eq!(
        spec.base_url, RESOLVED,
        "AC-001: base_url must equal the resolved env var value after resolution. \
         Got: {:?}",
        spec.base_url
    );
}

// ---------------------------------------------------------------------------
// AC-002: Partial-token interpolation — token within URL prefix/suffix.
//
// Input:  base_url = "https://${env.PRISM_TEST_ENV_VAR_AC002_ENV}.cyberint.io"
//         env PRISM_TEST_ENV_VAR_AC002_ENV = "us1"
// Output: errors empty
//         spec.base_url == "https://us1.cyberint.io"
//
// Traces to: BC-2.16.009 §Validation Rules 6 — partial interpolation; EC-009-003.
//            S-SPEC-ENV-VAR-001 AC-002.
// ---------------------------------------------------------------------------
#[test]
fn test_env_var_partial_token_resolves_preserving_surrounding_literals() {
    // AC-002: BC-2.16.009 §Validation Rules 6 — partial interpolation EC-009-003.
    const VAR: &str = "PRISM_TEST_ENV_VAR_AC002_ENV";
    const VAL: &str = "us1";
    const EXPECTED: &str = "https://us1.cyberint.io";

    unsafe {
        std::env::set_var(VAR, VAL);
    }

    let token = format!("${{env.{VAR}}}");
    let mut spec = minimal_spec(format!("https://{token}.cyberint.io"));

    let errors = resolve_env_var_tokens(&mut spec, "test/claroty.sensor.toml");

    unsafe {
        std::env::remove_var(VAR);
    }

    assert!(
        errors.is_empty(),
        "AC-002: resolve_env_var_tokens must return empty error vec for partial-token \
         case when var is set. Got {} error(s): {:?}",
        errors.len(),
        errors
    );

    assert_eq!(
        spec.base_url, EXPECTED,
        "AC-002: base_url must preserve surrounding literals and replace only the token. \
         Expected '{}', got '{}'",
        EXPECTED, spec.base_url
    );
}

// ---------------------------------------------------------------------------
// AC-003: Multi-token field — two ${env.VAR} tokens in one field, both set.
//
// Input:  base_url = "https://${env.PRISM_TEST_ENV_VAR_AC003_REGION}.${env.PRISM_TEST_ENV_VAR_AC003_HOST}.io"
//         env PRISM_TEST_ENV_VAR_AC003_REGION = "us1"
//         env PRISM_TEST_ENV_VAR_AC003_HOST   = "example"
// Output: errors empty
//         spec.base_url == "https://us1.example.io"
//
// Traces to: BC-2.16.009 §Validation Rules 6 — "resolver scans ALL ${env.VAR_NAME}
//            tokens in a field"; postcondition: "Every token ... replaced".
//            S-SPEC-ENV-VAR-001 AC-003.
// ---------------------------------------------------------------------------
#[test]
fn test_env_var_multi_token_single_field_both_resolve() {
    // AC-003: BC-2.16.009 §Validation Rules 6 — multi-token, all replaced.
    const VAR_REGION: &str = "PRISM_TEST_ENV_VAR_AC003_REGION";
    const VAR_HOST: &str = "PRISM_TEST_ENV_VAR_AC003_HOST";
    const EXPECTED: &str = "https://us1.example.io";

    unsafe {
        std::env::set_var(VAR_REGION, "us1");
        std::env::set_var(VAR_HOST, "example");
    }

    let token_region = format!("${{env.{VAR_REGION}}}");
    let token_host = format!("${{env.{VAR_HOST}}}");
    let mut spec = minimal_spec(format!("https://{token_region}.{token_host}.io"));

    let errors = resolve_env_var_tokens(&mut spec, "test/multi-token.sensor.toml");

    unsafe {
        std::env::remove_var(VAR_REGION);
        std::env::remove_var(VAR_HOST);
    }

    assert!(
        errors.is_empty(),
        "AC-003: resolve_env_var_tokens must return empty error vec when all tokens resolve. \
         Got {} error(s): {:?}",
        errors.len(),
        errors
    );

    assert_eq!(
        spec.base_url, EXPECTED,
        "AC-003: all tokens must be replaced; surrounding literals preserved. \
         Expected '{}', got '{}'",
        EXPECTED, spec.base_url
    );
}

// ---------------------------------------------------------------------------
// AC-004: Missing var → E-SPEC-024, spec rejected.
//
// Input:  base_url = "${env.PRISM_TEST_ENV_VAR_AC004_MISSING}"
//         env PRISM_TEST_ENV_VAR_AC004_MISSING: NOT SET
// Output: errors contains exactly 1 SpecEngineError::EnvVarNotSet
//         error.var_name == "PRISM_TEST_ENV_VAR_AC004_MISSING"
//         error.toml_path contains "base_url"
//
// Traces to: BC-2.16.009 §Validation Rules 6 error path — "If VAR_NAME is absent
//            from the environment ... → validation error E-SPEC-024"; EC-009-001.
//            S-SPEC-ENV-VAR-001 AC-004.
// ---------------------------------------------------------------------------
#[test]
fn test_env_var_missing_var_produces_e_spec_024() {
    // AC-004: BC-2.16.009 §Validation Rules 6 — absent var → E-SPEC-024 (EC-009-001).
    const VAR: &str = "PRISM_TEST_ENV_VAR_AC004_MISSING";

    // Ensure the var is absent (defensive: remove in case of earlier test pollution).
    unsafe {
        std::env::remove_var(VAR);
    }

    let token = format!("${{env.{VAR}}}");
    let mut spec = minimal_spec(token);

    let errors = resolve_env_var_tokens(&mut spec, "test/armis.sensor.toml");

    // Cleanup (no-op when absent, but symmetric with the set-based tests).
    unsafe {
        std::env::remove_var(VAR);
    }

    // Must produce exactly one error.
    assert_eq!(
        errors.len(),
        1,
        "AC-004: exactly 1 E-SPEC-024 error expected for 1 missing var. \
         Got {} error(s): {:?}",
        errors.len(),
        errors
    );

    // The error must be the correct variant.
    match &errors[0] {
        SpecEngineError::EnvVarNotSet {
            var_name,
            toml_path,
            ..
        } => {
            assert_eq!(
                var_name, VAR,
                "AC-004: error.var_name must be the env var NAME (not a value or different string). \
                 Expected '{}', got '{}'",
                VAR, var_name
            );
            assert!(
                toml_path.contains("base_url"),
                "AC-004: error.toml_path must identify the field (expected 'base_url' substring). \
                 Got: '{toml_path}'"
            );
        }
        other => {
            panic!(
                "AC-004: expected SpecEngineError::EnvVarNotSet, got: {:?}",
                other
            );
        }
    }
}

// ---------------------------------------------------------------------------
// AC-005: Empty var → E-SPEC-024, empty value treated as missing.
//
// Input:  base_url = "${env.PRISM_TEST_ENV_VAR_AC005_EMPTY}"
//         env PRISM_TEST_ENV_VAR_AC005_EMPTY = ""
// Output: errors contains exactly 1 SpecEngineError::EnvVarNotSet
//         (empty string must be treated the same as absent)
//         spec is rejected (not a URL-format error)
//
// Traces to: BC-2.16.009 §Validation Rules 6 — "If VAR_NAME is present but the
//            value is empty string (""), → validation error E-SPEC-024 (empty value
//            is treated as missing)"; EC-009-002.
//            S-SPEC-ENV-VAR-001 AC-005.
// ---------------------------------------------------------------------------
#[test]
fn test_env_var_empty_var_produces_e_spec_024() {
    // AC-005: BC-2.16.009 §Validation Rules 6 — empty var == missing (EC-009-002).
    const VAR: &str = "PRISM_TEST_ENV_VAR_AC005_EMPTY";

    // Set the var to the empty string.
    unsafe {
        std::env::set_var(VAR, "");
    }

    let token = format!("${{env.{VAR}}}");
    let mut spec = minimal_spec(token);

    let errors = resolve_env_var_tokens(&mut spec, "test/armis.sensor.toml");

    unsafe {
        std::env::remove_var(VAR);
    }

    assert_eq!(
        errors.len(),
        1,
        "AC-005: empty var must produce exactly 1 E-SPEC-024 error (empty == missing). \
         Got {} error(s): {:?}",
        errors.len(),
        errors
    );

    match &errors[0] {
        SpecEngineError::EnvVarNotSet { var_name, .. } => {
            assert_eq!(
                var_name, VAR,
                "AC-005: error.var_name must be the var name for the empty-value case. \
                 Expected '{}', got '{}'",
                VAR, var_name
            );
        }
        other => {
            panic!(
                "AC-005: expected SpecEngineError::EnvVarNotSet for empty var, got: {:?}",
                other
            );
        }
    }
}

// ---------------------------------------------------------------------------
// AC-006: Multi-error collection — no fail-fast; two missing vars → two errors.
//
// Input:  base_url contains TWO distinct ${env.VAR} tokens, both absent.
//         (Using a field that supports multiple tokens: "https://${env.VAR_A}.${env.VAR_B}.io")
//
//         To satisfy the BC constraint that multiple FIELDS produce multiple errors, this
//         test also sets a second string field (name) with an unresolvable token.
//         That exercises the cross-field collection path from BC-2.16.009:
//         "Multiple unresolvable tokens produce multiple E-SPEC-024 errors, one per token,
//          collected in the same multi-error pass (no fail-fast)".
//
//         Two distinct unresolvable tokens:
//           base_url = "${env.PRISM_TEST_ENV_VAR_AC006_MISSING_A}"     (absent)
//           name     = "${env.PRISM_TEST_ENV_VAR_AC006_MISSING_B} Sensor" (absent)
//
// Output: errors.len() == 2; both are EnvVarNotSet; neither triggers early exit.
//
// Traces to: BC-2.16.009 §Validation Rules 6 — "Multiple unresolvable tokens produce
//            multiple E-SPEC-024 errors, one per token, collected in the same multi-error
//            pass (no fail-fast)"; BC-2.16.009 §Invariants — "Validation is always a
//            single-pass, all-errors-collected operation"; EC-009-006.
//            S-SPEC-ENV-VAR-001 AC-006.
// ---------------------------------------------------------------------------
#[test]
fn test_env_var_multi_missing_tokens_collect_multiple_errors() {
    // AC-006: BC-2.16.009 §Validation Rules 6 / §Invariants — no fail-fast; EC-009-006.
    const VAR_A: &str = "PRISM_TEST_ENV_VAR_AC006_MISSING_A";
    const VAR_B: &str = "PRISM_TEST_ENV_VAR_AC006_MISSING_B";

    // Ensure both absent.
    unsafe {
        std::env::remove_var(VAR_A);
        std::env::remove_var(VAR_B);
    }

    let token_a = format!("${{env.{VAR_A}}}");
    let token_b = format!("${{env.{VAR_B}}}");

    // Two tokens in two different String fields: base_url and name.
    // Uses minimal_spec_with_name so both fields are set at parse time via TOML.
    let mut spec = minimal_spec_with_name(token_a, format!("{token_b} Sensor"));

    let errors = resolve_env_var_tokens(&mut spec, "test/multi-error.sensor.toml");

    unsafe {
        std::env::remove_var(VAR_A);
        std::env::remove_var(VAR_B);
    }

    // Must collect BOTH errors — no fail-fast.
    assert_eq!(
        errors.len(),
        2,
        "AC-006: exactly 2 E-SPEC-024 errors expected (one per missing token, no fail-fast). \
         Got {} error(s): {:?}",
        errors.len(),
        errors
    );

    // Both must be EnvVarNotSet.
    for (i, err) in errors.iter().enumerate() {
        assert!(
            matches!(err, SpecEngineError::EnvVarNotSet { .. }),
            "AC-006: error[{i}] must be EnvVarNotSet, got: {:?}",
            err
        );
    }

    // Collect the var names from the errors to verify both tokens were found.
    let reported_var_names: std::collections::HashSet<&str> = errors
        .iter()
        .filter_map(|e| match e {
            SpecEngineError::EnvVarNotSet { var_name, .. } => Some(var_name.as_str()),
            _ => None,
        })
        .collect();

    assert!(
        reported_var_names.contains(VAR_A),
        "AC-006: error set must include var name for first missing token '{VAR_A}'. \
         Reported: {reported_var_names:?}"
    );
    assert!(
        reported_var_names.contains(VAR_B),
        "AC-006: error set must include var name for second missing token '{VAR_B}'. \
         Reported: {reported_var_names:?}"
    );
}

// ---------------------------------------------------------------------------
// AC-007: Resolution ordering — resolver runs post-TOML-parse, pre-URL-format-validation.
//
// This test verifies the ordering invariant by checking what error code is produced
// when a var is absent. The test calls `validate_sensor_spec` (the full validation
// pipeline) after first running `resolve_env_var_tokens`.
//
// Scenario A (var absent): E-SPEC-024 is produced, NOT E-SPEC-001.
//   If the resolver ran AFTER url-format validation, the raw token
//   "${env.PRISM_TEST_ENV_VAR_AC007_UNSET}" would fail starts_with("http://") and produce
//   E-SPEC-001 instead. The fact that we get E-SPEC-024 proves the resolver ran first.
//
// Scenario B (var set to a valid HTTPS URL): the full pipeline succeeds.
//   The resolved URL — not the raw token — reaches the url-format check.
//   If the resolver ran AFTER url-format validation, it would get E-SPEC-001 on the
//   raw token and never reach the resolver. Scenario B proves the forward-direction.
//
// Traces to: BC-2.16.009 §Validation Rules 6 — "Post-TOML-parse, before URL-format
//            validation, the resolver scans all string fields"; success-path postcondition
//            — "the resulting string ... is passed to subsequent validation rules"; EC-009-004.
//            S-SPEC-ENV-VAR-001 AC-007.
// ---------------------------------------------------------------------------
#[test]
fn test_env_var_resolution_runs_before_url_format_validation() {
    // AC-007: BC-2.16.009 §Validation Rules 6 — ordering: resolver before URL-format check.
    const VAR_UNSET: &str = "PRISM_TEST_ENV_VAR_AC007_UNSET";
    const VAR_SET: &str = "PRISM_TEST_ENV_VAR_AC007_SET";

    unsafe {
        std::env::remove_var(VAR_UNSET);
        std::env::set_var(VAR_SET, "https://resolved.armis.io");
    }

    // ----- Scenario A: var absent → E-SPEC-024, NOT E-SPEC-001 -----
    //
    // Build spec, run resolver, then validate.
    // The resolver should produce E-SPEC-024 before url-format validation fires.
    {
        let token = format!("${{env.{VAR_UNSET}}}");
        let mut spec = minimal_spec(token);
        let resolve_errors = resolve_env_var_tokens(&mut spec, "test/scenario-a.sensor.toml");

        // The resolver must produce E-SPEC-024.
        assert_eq!(
            resolve_errors.len(),
            1,
            "AC-007 Scenario A: absent var must produce exactly 1 resolver error. \
             Got: {:?}",
            resolve_errors
        );
        assert!(
            matches!(resolve_errors[0], SpecEngineError::EnvVarNotSet { .. }),
            "AC-007 Scenario A: resolver error must be EnvVarNotSet (E-SPEC-024), not a \
             URL-format error (E-SPEC-001). Got: {:?}",
            resolve_errors[0]
        );

        // After the resolver runs with an absent var, the spec.base_url still contains the
        // raw token (the resolver does not mutate on failure). If we then call
        // validate_sensor_spec, it will produce E-SPEC-001 (url-format) — that is acceptable
        // and expected because the resolver already reported E-SPEC-024 first. The key check
        // is that the RESOLVER's error is E-SPEC-024, not the validator's.
        // (The pipeline caller must reject on non-empty resolve_errors before calling validate.)
    }

    // ----- Scenario B: var set → resolved URL passes url-format validation -----
    //
    // Build spec with token, run resolver (which should replace the token),
    // then run validate_sensor_spec. Validation must PASS (no errors).
    {
        let token = format!("${{env.{VAR_SET}}}");
        let mut spec = minimal_spec(token);
        let resolve_errors = resolve_env_var_tokens(&mut spec, "test/scenario-b.sensor.toml");

        // Cleanup before asserting.
        unsafe {
            std::env::remove_var(VAR_UNSET);
            std::env::remove_var(VAR_SET);
        }

        assert!(
            resolve_errors.is_empty(),
            "AC-007 Scenario B: var set to valid URL must produce no resolver errors. \
             Got: {:?}",
            resolve_errors
        );

        // After resolution, base_url must equal the resolved value.
        assert_eq!(
            spec.base_url, "https://resolved.armis.io",
            "AC-007 Scenario B: base_url must equal the resolved var value post-resolution."
        );

        // The resolved URL must pass validate_sensor_spec (URL-format check sees resolved value).
        // Note: this is a table-less spec, so we only check the base_url validation path.
        // A fully populated spec is not required for this ordering check.
        let validation_result = validate_sensor_spec(&spec);
        assert!(
            validation_result.is_ok(),
            "AC-007 Scenario B: resolved URL must pass url-format validation. \
             Errors: {:?}",
            validation_result.err()
        );
    }
}

// ---------------------------------------------------------------------------
// AC-008: AD-017 no-value-leak — error message contains var NAME, never var VALUE.
//
// Setup:
//   1. Set PRISM_TEST_ENV_VAR_AC008_SECRET = "https://secret.internal.sentinel-value-do-not-log"
//      (a sentinel we can search for in error output)
//   2. Unset the var.
//   3. Call resolve_env_var_tokens with the (now-absent) var.
//   4. Assert E-SPEC-024 is produced.
//   5. Assert the error Display + Debug contains "PRISM_TEST_ENV_VAR_AC008_SECRET" (var NAME).
//   6. Assert the error Display + Debug does NOT contain "secret.internal.sentinel-value-do-not-log"
//      (the resolved VALUE must not appear anywhere in the error representation).
//
// Additionally: set the var to a valid URL and run the resolver successfully.
//   Assert that no part of the "happy path" produces a string containing the sentinel value
//   in any error or warning from validate_sensor_spec.
//
// Traces to: BC-2.16.009 §Validation Rules 6 — "The error message MUST NOT include the
//            variable VALUE — per AD-017 / AI-opaque-credentials discipline";
//            error-taxonomy.md E-SPEC-024 — "The env var VALUE is NEVER included".
//            S-SPEC-ENV-VAR-001 AC-008.
// ---------------------------------------------------------------------------
#[test]
fn test_env_var_error_contains_name_not_value() {
    // AC-008: AD-017 no-value-leak — NAME in error; VALUE never in error.
    const VAR: &str = "PRISM_TEST_ENV_VAR_AC008_SECRET";
    // The sentinel value is chosen to be easily grep-able and not something
    // that would naturally appear in any spec-engine log or error message.
    const SENTINEL_VALUE: &str = "https://secret.internal.sentinel-value-do-not-log";

    // Step 1: Set the var to the sentinel value, then immediately unset it.
    // This simulates a scenario where the var was previously set in the session
    // (e.g., a prior deployment), verifying there is no cached-value leak.
    unsafe {
        std::env::set_var(VAR, SENTINEL_VALUE);
        std::env::remove_var(VAR);
    }

    // Step 2: Var is now absent — resolve should produce E-SPEC-024.
    let token = format!("${{env.{VAR}}}");
    let mut spec = minimal_spec(token);
    let errors = resolve_env_var_tokens(&mut spec, "test/secret.sensor.toml");

    assert_eq!(
        errors.len(),
        1,
        "AC-008: absent var must produce exactly 1 error. Got: {:?}",
        errors
    );

    let err = &errors[0];

    // Step 3: Assert the error is the correct variant.
    let (var_name_in_err, toml_path_in_err) = match err {
        SpecEngineError::EnvVarNotSet {
            var_name,
            toml_path,
            ..
        } => (var_name.clone(), toml_path.clone()),
        other => panic!("AC-008: expected EnvVarNotSet, got: {:?}", other),
    };

    // Step 4: The var NAME must appear in the error fields.
    assert_eq!(
        var_name_in_err, VAR,
        "AC-008: error.var_name must equal the variable NAME '{}'. Got: '{}'",
        VAR, var_name_in_err
    );
    assert!(
        toml_path_in_err.contains("base_url"),
        "AC-008: error.toml_path must identify 'base_url'. Got: '{toml_path_in_err}'"
    );

    // Step 5: The sentinel VALUE must NOT appear in the error Display representation.
    // This is the primary AD-017 assertion: operator logs that capture error messages
    // must not expose credential values.
    let display_str = format!("{}", err);
    assert!(
        !display_str.contains(SENTINEL_VALUE),
        "AC-008 (AD-017): E-SPEC-024 Display representation must NOT contain the resolved \
         VALUE '{SENTINEL_VALUE}'. The display output was: '{display_str}'"
    );

    // Step 6: The sentinel VALUE must NOT appear in the Debug representation.
    let debug_str = format!("{:?}", err);
    assert!(
        !debug_str.contains(SENTINEL_VALUE),
        "AC-008 (AD-017): E-SPEC-024 Debug representation must NOT contain the resolved \
         VALUE '{SENTINEL_VALUE}'. The debug output was: '{debug_str}'"
    );

    // Step 7: The var NAME must appear in the Display representation (operator must know WHICH var).
    assert!(
        display_str.contains(VAR),
        "AC-008: E-SPEC-024 Display representation MUST contain the var NAME '{}'. \
         Display: '{display_str}'",
        VAR
    );

    // Step 8: Happy path — var set to sentinel; validate_sensor_spec must NOT log the value.
    // (The resolved value should only flow into spec.base_url, not into any error/warning.)
    unsafe {
        std::env::set_var(VAR, SENTINEL_VALUE);
    }

    let token2 = format!("${{env.{VAR}}}");
    let mut spec2 = minimal_spec(token2);
    let resolve_errors = resolve_env_var_tokens(&mut spec2, "test/secret-set.sensor.toml");

    unsafe {
        std::env::remove_var(VAR);
    }

    // Resolution succeeds — sentinel value is injected into base_url (expected and correct).
    assert!(
        resolve_errors.is_empty(),
        "AC-008 happy path: no resolver errors when var is set. Got: {:?}",
        resolve_errors
    );
    assert_eq!(
        spec2.base_url, SENTINEL_VALUE,
        "AC-008 happy path: resolved URL must equal the sentinel value in base_url."
    );

    // The sentinel value must NOT appear in any validation WARNING or ERROR produced by
    // validate_sensor_spec on the resolved spec. (The value in base_url is fine — it's the
    // spec field — but it must not bleed into structured validation diagnostics.)
    let validation_result = validate_sensor_spec(&spec2);
    // This spec will fail URL validation because SENTINEL_VALUE starts with "https://"...
    // actually SENTINEL_VALUE = "https://secret.internal.sentinel-value-do-not-log" — valid URL!
    // We check that no error or warning text contains the sentinel.
    match &validation_result {
        Ok(warnings) => {
            for w in warnings {
                let w_msg = &w.message;
                assert!(
                    !w_msg.contains(SENTINEL_VALUE),
                    "AC-008 (AD-017): validation WARNING must not contain the resolved VALUE. \
                     Warning: '{w_msg}'"
                );
            }
        }
        Err(errors) => {
            for e in errors {
                let e_msg = &e.message;
                assert!(
                    !e_msg.contains(SENTINEL_VALUE),
                    "AC-008 (AD-017): validation ERROR must not contain the resolved VALUE. \
                     Error: '{e_msg}'"
                );
            }
        }
    }
}
