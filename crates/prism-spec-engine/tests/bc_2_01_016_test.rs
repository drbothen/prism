#![allow(non_snake_case)]
//! Red Gate tests for BC-2.01.016 — SensorAuth Open Trait Contract.
//!
//! Tests 2, 4, 5 of the S-PLUGIN-PREREQ-E Red Gate set (prism-spec-engine crate).
//!
//! | Test | Name                                                          | AC   | Red Gate failure mode              |
//! |------|---------------------------------------------------------------|------|------------------------------------|
//! | 2    | test_BC_2_01_016_002_auth_composition_runtime_rejection       | AC-3 | todo!() in validate_cross_composition → panic |
//! | 4    | test_BC_2_01_016_e_spec_013_multiple_credential_refs_rejected | AC-3b| todo!() in validate_cross_composition → panic |
//! | 5    | test_BC_2_01_016_e_spec_014_credential_type_mismatch_rejected | AC-3c| todo!() in validate_cross_composition → panic |
//!
//! Story: S-PLUGIN-PREREQ-E | BC: BC-2.01.016 | ADR-026 §D3 | ADR-023 Rule 2

use prism_spec_engine::spec_parser::SpecLoader;

// ---------------------------------------------------------------------------
// Test 2 — test_BC_2_01_016_002_auth_composition_runtime_rejection
// AC-3: auth_type multi-value (array) → E-SPEC-012 (Rule A).
//
// Pre-implementation failure mode: todo!() panic in validate_cross_composition.
// Post-implementation: validate_cross_composition returns Err(AuthTypeCrossComposition).
// ---------------------------------------------------------------------------

/// BC-2.01.016 AC-3 / INV-AUTH-OPEN-003: A `SensorSpec` with a multi-valued
/// `auth_type` (array) must be rejected at spec-load with E-SPEC-012.
///
/// Exercises BC-2.01.016 §Error Cases E-SPEC-012 (Rule A — multi-valued or
/// out-of-set auth_type). Post-unsealing, the sealed-trait compile-time guard
/// is replaced by this runtime rejection rule (ADR-023 Rule 2, Rule A).
///
/// Red Gate failure mode: `validate_cross_composition` is `todo!()` — panics
/// rather than returning the expected error. Test fails on panic.
///
/// Story: S-PLUGIN-PREREQ-E AC-3 | BC: BC-2.01.016 | ADR-026 §D3 | ADR-023 Rule 2 Rule A
#[test]
fn test_BC_2_01_016_002_auth_composition_runtime_rejection() {
    // Sensor spec with out-of-set auth_type value (simulates cross-composition attempt).
    // The auth_type "invalid_composite_type" is outside the closed enumeration
    // {oauth2_client_credentials, bearer_static, cookie_roundtrip, api_key, custom_via_plugin}.
    let result = SpecLoader::validate_cross_composition(
        "test_sensor",
        "invalid_composite_type", // out-of-set value — triggers Rule A (E-SPEC-012)
        1,                        // credential_refs_count: exactly 1 (Rule B not triggered)
        "oauth2_client_credentials", // expected_shape
        "oauth2_client_credentials", // actual_shape matches (Rule C not triggered)
    );
    assert!(
        result.is_err(),
        "AC-3: auth_type outside closed enumeration must be rejected with E-SPEC-012; \
         validate_cross_composition returned Ok(()) instead"
    );
    let err = result.unwrap_err();
    let err_str = format!("{err}");
    assert!(
        err_str.contains("E-SPEC-012"),
        "AC-3: error must cite E-SPEC-012; got: {err_str}"
    );
}

// ---------------------------------------------------------------------------
// Test 4 — test_BC_2_01_016_e_spec_013_multiple_credential_refs_rejected
// AC-3b: multiple credential_refs → E-SPEC-013 (Rule B).
//
// Pre-implementation failure mode: todo!() panic in validate_cross_composition.
// Post-implementation: validate_cross_composition returns Err(MultipleCredentialRefs).
// ---------------------------------------------------------------------------

/// BC-2.01.016 AC-3b: A `SensorSpec` with multiple `credential_refs` for a
/// single auth method must be rejected at spec-load with E-SPEC-013.
///
/// Exercises BC-2.01.016 §Error Cases E-SPEC-013 (Rule B — multiple credential_refs
/// per auth method; cardinality must be exactly 1). ADR-023 Rule 2, Rule B.
///
/// Red Gate failure mode: `validate_cross_composition` is `todo!()` — panics.
///
/// Story: S-PLUGIN-PREREQ-E AC-3b | BC: BC-2.01.016 | ADR-023 Rule 2 Rule B
#[test]
fn test_BC_2_01_016_e_spec_013_multiple_credential_refs_rejected() {
    // Valid auth_type (Rule A passes) but credential_refs_count > 1 (Rule B fires).
    let result = SpecLoader::validate_cross_composition(
        "test_sensor",
        "oauth2_client_credentials", // valid auth_type — Rule A passes
        2,                           // 2 credential_refs — triggers Rule B (E-SPEC-013)
        "oauth2_client_credentials",
        "oauth2_client_credentials",
    );
    assert!(
        result.is_err(),
        "AC-3b: multiple credential_refs must be rejected with E-SPEC-013; \
         validate_cross_composition returned Ok(()) instead"
    );
    let err = result.unwrap_err();
    let err_str = format!("{err}");
    assert!(
        err_str.contains("E-SPEC-013"),
        "AC-3b: error must cite E-SPEC-013; got: {err_str}"
    );
}

// ---------------------------------------------------------------------------
// Test 5 — test_BC_2_01_016_e_spec_014_credential_type_mismatch_rejected
// AC-3c: auth_type/credential structural mismatch → E-SPEC-014 (Rule C).
//
// Pre-implementation failure mode: todo!() panic in validate_cross_composition.
// Post-implementation: validate_cross_composition returns Err(AuthTypeCredentialMismatch).
// ---------------------------------------------------------------------------

/// BC-2.01.016 AC-3c: A `SensorSpec` where `auth_type` and the resolved
/// credential's structural shape disagree must be rejected with E-SPEC-014.
///
/// Example: `auth_type = "oauth2_client_credentials"` paired with an
/// API-key-shaped credential (expected `client_id+client_secret`, got `api_key`).
///
/// Exercises BC-2.01.016 §Error Cases E-SPEC-014 (Rule C — structural mismatch
/// between auth_type and credential shape). ADR-023 Rule 2, Rule C.
///
/// Red Gate failure mode: `validate_cross_composition` is `todo!()` — panics.
///
/// Story: S-PLUGIN-PREREQ-E AC-3c | BC: BC-2.01.016 | ADR-023 Rule 2 Rule C
#[test]
fn test_BC_2_01_016_e_spec_014_credential_type_mismatch_rejected() {
    // Valid auth_type (Rule A passes), single credential_ref (Rule B passes),
    // but credential shape "api_key" doesn't match expected "client_id+client_secret".
    let result = SpecLoader::validate_cross_composition(
        "test_sensor",
        "oauth2_client_credentials", // declared auth_type
        1,                           // single credential_ref — Rule B passes
        "client_id+client_secret",   // expected structural shape for oauth2_client_credentials
        "api_key",                   // actual resolved credential shape — MISMATCH (E-SPEC-014)
    );
    assert!(
        result.is_err(),
        "AC-3c: auth_type/credential structural mismatch must be rejected with E-SPEC-014; \
         validate_cross_composition returned Ok(()) instead"
    );
    let err = result.unwrap_err();
    let err_str = format!("{err}");
    assert!(
        err_str.contains("E-SPEC-014"),
        "AC-3c: error must cite E-SPEC-014; got: {err_str}"
    );
}
