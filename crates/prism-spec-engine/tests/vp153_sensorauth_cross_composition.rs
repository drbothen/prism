#![allow(non_snake_case)]
//! VP-153: SensorAuth Runtime Cross-Composition Prevention — Rules A and B
//!
//! BC-2.01.016 §Postconditions Rules A+B via `SpecLoader::validate_cross_composition`.
//!
//! ADR-026 §D3 — Runtime cross-sensor auth-composition prevention (3 rules):
//!   Rule A (E-SPEC-012): auth_type must be a scalar from the closed enumeration
//!     {oauth2_client_credentials, bearer_static, cookie_roundtrip, api_key, custom_via_plugin}.
//!     Multi-valued or out-of-set values are rejected at spec-load time.
//!   Rule B (E-SPEC-013): credential_refs must have exactly one entry per auth method.
//!     Multiple bindings are rejected at spec-load time.
//!   Rule C (E-SPEC-014): auth_type / credential structural shape mismatch.
//!     Per ADR-026 §D3 Rule C Backend Scope (D-706 amendment), Rule C fires only when
//!     the credential backend exposes shape metadata via CredentialRefProbe::probe()
//!     returning Some(shape). The keyring backend returns Ok(None). Rule C proptest
//!     via ShapedProbe injection lives in crates/prism-bin/tests/vp153_rule_c_shaped_probe.rs
//!     (prism-bin is a direct dependency of prism-spec-engine in the REVERSE direction;
//!     adding prism-bin as a dev-dep here would create a workspace cycle).
//!
//! ## Test inventory
//!
//! | Test | Name | Rule | BC Clause |
//! |------|------|------|-----------|
//! | 1 | prop_rule_a_invalid_auth_type_rejected_with_e_spec_012 | A | E-SPEC-012 |
//! | 2 | prop_rule_a_valid_auth_type_accepted | A | postcondition |
//! | 3 | prop_rule_b_multi_credential_refs_rejected_with_e_spec_013 | B | E-SPEC-013 |
//! | 4 | prop_rule_b_single_or_zero_credential_refs_accepted | B | postcondition |
//! | 5 | prop_rule_a_boundary_whitespace_and_empty_rejected | A | E-SPEC-012 edge |
//! | 6 | prop_rule_b_credential_count_boundary | B | E-SPEC-013 boundary |
//!
//! PROPTEST_CASES: respects env var (32 for `just iter`, 256 for `just check`).
//!
//! Story: S-PLUGIN-PREREQ-E | BC: BC-2.01.016 | ADR-026 §D3 | ADR-023 Rule 2
//! VP: VP-153 (SensorAuth Runtime Cross-Composition Prevention)
//! Fixes: F-LP-IMPL-P8-IMP-001 (pass-8 blind spot — VP-153 P0 artifact never authored)

use prism_spec_engine::spec_parser::SpecLoader;
use proptest::prelude::*;

// ---------------------------------------------------------------------------
// Constants — canonical valid auth_type set (BC-2.01.016 / ADR-026 §D3 Rule 1)
// ---------------------------------------------------------------------------

const VALID_AUTH_TYPES: &[&str] = &[
    "oauth2_client_credentials",
    "bearer_static",
    "cookie_roundtrip",
    "api_key",
    "custom_via_plugin",
];

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

/// Strategy: one of the 5 valid auth_type strings (per ADR-026 §D3 enumerated set).
fn arb_valid_auth_type() -> impl Strategy<Value = &'static str> {
    prop_oneof![
        Just("oauth2_client_credentials"),
        Just("bearer_static"),
        Just("cookie_roundtrip"),
        Just("api_key"),
        Just("custom_via_plugin"),
    ]
}

/// Strategy: a string that is explicitly NOT in the valid auth_type set.
///
/// Generates from several classes of invalid values:
/// - Arbitrary ASCII-alpha lowercase strings of 1..=24 chars (most won't match)
/// - Empty string (boundary)
/// - Whitespace-only strings
/// - Known near-misses (capitalised variants, hyphenated variants, etc.)
///
/// Filters out exact matches to any VALID_AUTH_TYPES member so every generated
/// value is guaranteed to trigger Rule A (E-SPEC-012).
fn arb_invalid_auth_type() -> impl Strategy<Value = String> {
    prop_oneof![
        // Arbitrary lowercase+underscore strings — rarely match any valid type
        "[a-z_]{1,32}".prop_map(String::from),
        // Empty string
        Just(String::new()),
        // Whitespace-only
        prop_oneof![
            Just(" ".to_string()),
            Just("  ".to_string()),
            Just("\t".to_string()),
        ],
        // Mixed-case / capitalised variants that look like auth types but differ
        prop_oneof![
            Just("OAuth2_client_credentials".to_string()),
            Just("BEARER_STATIC".to_string()),
            Just("Cookie_Roundtrip".to_string()),
            Just("ApiKey".to_string()),
            Just("Custom_Via_Plugin".to_string()),
        ],
        // Dot-prefixed or slash-prefixed
        prop_oneof![
            Just(".oauth2_client_credentials".to_string()),
            Just("/bearer_static".to_string()),
        ],
        // Arbitrary Unicode strings outside ASCII
        "\\PC{1,16}".prop_map(String::from),
        // Known composite/multi-value representation strings that look like arrays
        prop_oneof![
            Just("[oauth2_client_credentials,bearer_static]".to_string()),
            Just("oauth2_client_credentials+bearer_static".to_string()),
        ],
    ]
    .prop_filter("must not accidentally be a valid auth_type", |s| {
        !VALID_AUTH_TYPES.contains(&s.as_str())
    })
}

/// Strategy: credential_refs_count drawn from [2..=8].
/// Used for Rule B violation tests (exactly 1 is required; >1 fires E-SPEC-013).
fn arb_multi_credential_count() -> impl Strategy<Value = usize> {
    2usize..=8usize
}

// ---------------------------------------------------------------------------
// Rule A (E-SPEC-012) proptests
// ---------------------------------------------------------------------------

proptest! {
    /// VP-153 prop 1 of 6: Rule A — invalid/out-of-set auth_type is rejected with E-SPEC-012.
    ///
    /// Exercises BC-2.01.016 §Error Cases E-SPEC-012: "auth_type for sensor '{sensor_id}'
    /// must be a single value from the canonical enumerated set; got: '{value}'."
    ///
    /// Preconditions: credential_refs_count = 1 (Rule B not triggered); expected_shape ==
    ///   actual_shape (Rule C not triggered). Only Rule A fires.
    ///
    /// Postcondition: validate_cross_composition returns Err containing "E-SPEC-012".
    ///
    /// Story: S-PLUGIN-PREREQ-E AC-3 | BC-2.01.016 Rule A | ADR-026 §D3 Rule 1
    /// VP-153 prop 1/6: F-LP-IMPL-P8-IMP-001
    #[test]
    fn prop_rule_a_invalid_auth_type_rejected_with_e_spec_012(
        invalid_type in arb_invalid_auth_type(),
        // Use "placeholder_shape" for both expected/actual so Rule C never fires
        // (Rule A violation is primary; Rule C equality check is trivially satisfied)
    ) {
        let result = SpecLoader::validate_cross_composition(
            "vp153-test-sensor",
            &invalid_type,
            1,               // single credential_ref — Rule B passes
            "placeholder",   // expected_shape: same as actual — Rule C passes
            "placeholder",   // actual_shape: same as expected — Rule C passes
        );
        prop_assert!(
            result.is_err(),
            "Rule A (E-SPEC-012): invalid auth_type {:?} was accepted; expected Err",
            invalid_type
        );
        let err = result.unwrap_err();
        let err_str = format!("{err}");
        prop_assert!(
            err_str.contains("E-SPEC-012"),
            "Rule A: error must cite E-SPEC-012; auth_type={:?}; got: {}",
            invalid_type,
            err_str
        );
        // AD-017: error must not contain credential VALUES — only logical names
        // (invalid_type is the auth_type string itself, not a credential value)
    }
}

proptest! {
    /// VP-153 prop 2 of 6: Rule A — valid auth_type is accepted.
    ///
    /// Regression guard: none of the 5 valid auth_type values should trigger Rule A.
    ///
    /// Preconditions: credential_refs_count = allowed count for auth_type; expected_shape
    ///   == actual_shape. Postcondition: validate_cross_composition returns Ok(()).
    ///
    /// Per BC-2.06.003 / ADR-032: `oauth2_client_credentials` requires 2 refs;
    /// all other auth types require 1.
    ///
    /// Story: S-PLUGIN-PREREQ-E AC-3 | BC-2.01.016 postcondition | ADR-026 §D3 Rule 1
    /// VP-153 prop 2/6: F-LP-IMPL-P8-IMP-001
    #[test]
    fn prop_rule_a_valid_auth_type_accepted(
        valid_type in arb_valid_auth_type(),
    ) {
        // BC-2.06.003 / ADR-032: use the correct credential count for each auth type.
        let allowed_count = if valid_type == "oauth2_client_credentials" { 2 } else { 1 };
        let result = SpecLoader::validate_cross_composition(
            "vp153-test-sensor",
            valid_type,
            allowed_count,   // correct count for this auth_type — Rule B passes
            valid_type,      // expected_shape matches auth_type — Rule C passes
            valid_type,      // actual_shape matches — Rule C passes
        );
        prop_assert!(
            result.is_ok(),
            "Rule A regression guard: valid auth_type {:?} (count={}) was rejected; expected Ok(())",
            valid_type,
            allowed_count
        );
    }
}

// ---------------------------------------------------------------------------
// Rule B (E-SPEC-013) proptests
// ---------------------------------------------------------------------------

proptest! {
    /// VP-153 prop 3 of 6: Rule B — multiple credential_refs rejected with E-SPEC-013.
    ///
    /// Exercises BC-2.01.016 §Error Cases E-SPEC-013: "auth method for sensor
    /// '{sensor_id}' declares {count} credential_refs; exactly one is required."
    ///
    /// Preconditions: auth_type is valid (Rule A passes); credential_refs_count > 1;
    ///   expected_shape == actual_shape (Rule C passes). Only Rule B fires.
    ///
    /// Postcondition: validate_cross_composition returns Err containing "E-SPEC-013".
    ///
    /// Per BC-2.06.003 / ADR-032: `oauth2_client_credentials` now allows exactly 2
    /// credential_refs. We filter the (auth_type=oauth2, count=2) valid combination.
    ///
    /// Story: S-PLUGIN-PREREQ-E AC-3b | BC-2.01.016 Rule B | ADR-026 §D3 Rule 2
    /// VP-153 prop 3/6: F-LP-IMPL-P8-IMP-001
    #[test]
    fn prop_rule_b_multi_credential_refs_rejected_with_e_spec_013(
        valid_type in arb_valid_auth_type(),
        count in arb_multi_credential_count(),
    ) {
        // BC-2.06.003 / ADR-032: oauth2_client_credentials allows exactly 2 refs.
        // Skip the valid (oauth2_client_credentials, 2) combination — it passes Rule B.
        let allowed_count = if valid_type == "oauth2_client_credentials" { 2 } else { 1 };
        prop_assume!(count != allowed_count);

        let result = SpecLoader::validate_cross_composition(
            "vp153-test-sensor",
            valid_type,
            count,           // count != allowed_count for this auth_type — Rule B fires (E-SPEC-013)
            valid_type,      // expected_shape — Rule C passes (same as actual)
            valid_type,      // actual_shape — Rule C passes
        );
        prop_assert!(
            result.is_err(),
            "Rule B (E-SPEC-013): credential_refs_count={} was accepted; expected Err for auth_type={}",
            count,
            valid_type
        );
        let err = result.unwrap_err();
        let err_str = format!("{err}");
        prop_assert!(
            err_str.contains("E-SPEC-013"),
            "Rule B: error must cite E-SPEC-013; count={}; got: {}",
            count,
            err_str
        );
    }
}

proptest! {
    /// VP-153 prop 4 of 6: Rule B — exactly one credential_ref accepted.
    ///
    /// Regression guard: credential_refs_count = 1 must not trigger Rule B.
    ///
    /// Note: In production `validate_cross_composition` is only called when the
    /// sensor spec has at least one credential_ref (guarded by `!credential_refs.is_empty()`
    /// in `add_sensor_spec.rs` line 188). The function's Rule B check is
    /// `credential_refs_count != 1`, meaning count=0 would also fire Rule B if the
    /// function were called with 0 — but it never is in production. This test
    /// exercises count=1 (the only non-violating count).
    ///
    /// Preconditions: auth_type is valid (Rule A passes); count = 1 for non-oauth2 types,
    /// count = 2 for oauth2_client_credentials.
    /// Postcondition: validate_cross_composition returns Ok(()).
    ///
    /// Per BC-2.06.003 / ADR-032: `oauth2_client_credentials` requires exactly 2
    /// credential_refs (client_id + client_secret). All other auth types require exactly 1.
    ///
    /// Story: S-PLUGIN-PREREQ-E AC-3b | BC-2.01.016 postcondition | ADR-026 §D3 Rule 2
    /// VP-153 prop 4/6: F-LP-IMPL-P8-IMP-001
    #[test]
    fn prop_rule_b_single_credential_ref_accepted(
        valid_type in arb_valid_auth_type(),
    ) {
        // BC-2.06.003 / ADR-032: oauth2_client_credentials requires 2; others require 1.
        let allowed_count = if valid_type == "oauth2_client_credentials" { 2 } else { 1 };
        let result = SpecLoader::validate_cross_composition(
            "vp153-test-sensor",
            valid_type,
            allowed_count,   // correct count for this auth_type — Rule B must NOT fire
            valid_type,      // expected_shape matches auth_type — Rule C passes
            valid_type,      // actual_shape matches — Rule C passes
        );
        prop_assert!(
            result.is_ok(),
            "Rule B regression guard: credential_refs_count={} was rejected; \
             valid auth_type={:?}; expected Ok(())",
            allowed_count,
            valid_type
        );
    }
}

// ---------------------------------------------------------------------------
// Rule A boundary edge cases
// ---------------------------------------------------------------------------

proptest! {
    /// VP-153 prop 5 of 6: Rule A — whitespace and empty strings are rejected.
    ///
    /// Edge case: whitespace-only and empty auth_type values are clearly outside
    /// the closed enumeration and must trigger E-SPEC-012. This documents the
    /// boundary behavior explicitly in addition to the general arb_invalid_auth_type
    /// coverage above.
    ///
    /// Story: S-PLUGIN-PREREQ-E AC-3 | BC-2.01.016 E-SPEC-012 edge | ADR-026 §D3 Rule 1
    /// VP-153 prop 5/6: F-LP-IMPL-P8-IMP-001
    #[test]
    fn prop_rule_a_boundary_whitespace_and_empty_rejected(
        auth_type in prop_oneof![
            Just(String::new()),
            Just(" ".to_string()),
            Just("  ".to_string()),
            Just("\t".to_string()),
            Just("\n".to_string()),
        ],
    ) {
        let result = SpecLoader::validate_cross_composition(
            "vp153-boundary-sensor",
            &auth_type,
            1,
            "placeholder",
            "placeholder",
        );
        prop_assert!(
            result.is_err(),
            "Rule A boundary: empty/whitespace auth_type {:?} was accepted; expected Err",
            auth_type
        );
        let err = result.unwrap_err();
        let err_str = format!("{err}");
        prop_assert!(
            err_str.contains("E-SPEC-012"),
            "Rule A boundary: error must cite E-SPEC-012; auth_type={:?}; got: {}",
            auth_type,
            err_str
        );
    }
}

// ---------------------------------------------------------------------------
// Rule B credential count boundary sweep
// ---------------------------------------------------------------------------

proptest! {
    /// VP-153 prop 6 of 6: Rule B — credential count boundary: all counts ≥ 2 rejected.
    ///
    /// Exercises BC-2.01.016 §Error Cases E-SPEC-013 across a wider count range [2..=32].
    /// Counts that exceed the auth_type's allowed cardinality fire E-SPEC-013.
    ///
    /// Per BC-2.06.003 / ADR-032: `oauth2_client_credentials` allows exactly 2;
    /// other auth types allow exactly 1. Counts above the allowed value trigger E-SPEC-013.
    ///
    /// Story: S-PLUGIN-PREREQ-E AC-3b | BC-2.01.016 E-SPEC-013 boundary | ADR-026 §D3
    /// VP-153 prop 6/6: F-LP-IMPL-P8-IMP-001
    #[test]
    fn prop_rule_b_credential_count_boundary(
        valid_type in arb_valid_auth_type(),
        count in 2usize..=32,
    ) {
        // BC-2.06.003 / ADR-032: skip the exactly-valid (oauth2, 2) combination.
        let allowed_count = if valid_type == "oauth2_client_credentials" { 2 } else { 1 };
        prop_assume!(count != allowed_count);

        let result = SpecLoader::validate_cross_composition(
            "vp153-count-boundary-sensor",
            valid_type,
            count,
            valid_type,
            valid_type,
        );
        prop_assert!(
            result.is_err(),
            "Rule B boundary: count={} (allowed={}) was accepted; expected Err(E-SPEC-013) for auth_type={}",
            count,
            allowed_count,
            valid_type
        );
        let err = result.unwrap_err();
        let err_str = format!("{err}");
        prop_assert!(
            err_str.contains("E-SPEC-013"),
            "Rule B boundary: error must cite E-SPEC-013; count={}; got: {}",
            count,
            err_str
        );
    }
}
