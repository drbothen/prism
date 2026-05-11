use prism_core::sensor_id::SensorId;
use prism_spec_engine::validation::validate_sensor_id;
/// Cross-crate proptest: prism-core and prism-spec-engine validator parity.
///
/// F-LP4-MED-001 (pass-4 finding): the two validators must agree on every
/// input — `SensorId::try_from_str(s).is_ok()` iff
/// `prism_spec_engine::validation::validate_sensor_id(s, None).is_none()`.
///
/// Both implement the canonical rule `^[a-z][a-z0-9_-]*$` (first char [a-z],
/// subsequent chars [a-z0-9_-], 1..=64 total length). This test locks the
/// invariant so no future divergence can occur silently.
use proptest::prelude::*;

/// Generate arbitrary ASCII strings up to 70 characters to cover the
/// 1..=64 valid range and common invalid inputs (empty, too long, bad chars).
fn arb_candidate() -> impl Strategy<Value = String> {
    // Mix of targeted and arbitrary ASCII to stress all rule boundaries.
    prop_oneof![
        // Fully arbitrary printable ASCII strings (length 0..=70).
        proptest::string::string_regex("[[:print:]]{0,70}").unwrap(),
        // Strings from the valid charset, varying length.
        proptest::string::string_regex("[a-z0-9_-]{0,70}").unwrap(),
        // Strings starting with a digit.
        proptest::string::string_regex("[0-9][a-z0-9_-]{0,63}").unwrap(),
        // Strings starting with a lowercase letter (most likely to be valid).
        proptest::string::string_regex("[a-z][a-z0-9_-]{0,63}").unwrap(),
    ]
}

proptest! {
    /// Core parity invariant: both validators produce the same ok/err decision
    /// for every generated candidate string.
    ///
    /// `SensorId::try_from_str` returns `Ok` iff `validate_sensor_id` returns `None`
    /// (no validation error). Any divergence means the two rules have drifted apart.
    #[test]
    fn proptest_validator_parity(s in arb_candidate()) {
        let core_ok = SensorId::try_from_str(&s).is_ok();
        let spec_ok = validate_sensor_id(&s, None).is_none();
        prop_assert_eq!(
            core_ok,
            spec_ok,
            "validator parity violated for {:?}: prism-core says {}, prism-spec-engine says {}",
            s,
            if core_ok { "Ok" } else { "Err" },
            if spec_ok { "Ok" } else { "Err" },
        );
    }
}
