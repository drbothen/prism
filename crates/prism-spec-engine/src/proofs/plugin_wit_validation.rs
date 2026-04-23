//! VP-043: WIT Validation Rejects Plugin Missing Required Exports.
//!
//! # Property
//! For any WASM Component with a strict subset of the required Prism WIT exports,
//! `validate_wit_interface(component_exports, path)` returns
//! `Err(PluginError::InvalidInterface)` with an error message that names the missing
//! export. For a component that provides all required exports, returns `Ok(PluginType)`.
//!
//! The function is deterministic: same inputs → same output.
//!
//! # Method: proptest (all subsets of each required export set).
//!
//! # Source BC: BC-2.17.006 — WIT Interface Validation Before Plugin Registration.
//!
//! # Status: Red Gate stub — tests fail, proof not yet written.

#[cfg(test)]
mod tests {
    use prism_core::PluginError;
    use proptest::prelude::*;

    // Import the target under test — will not compile until S-1.15 is implemented.
    use crate::plugin::discovery::{
        validate_wit_interface, ACTION_REQUIRED_EXPORTS, INFUSION_REQUIRED_EXPORTS,
        SENSOR_REQUIRED_EXPORTS,
    };
    use crate::plugin::PluginType;

    /// Helper: generate all non-empty strict subsets of `exports` as proptest strategies.
    fn arb_strict_subset(
        exports: &'static [&'static str],
    ) -> impl Strategy<Value = Vec<&'static str>> {
        // Generate a bitmask over exports where at least one bit is 0 (strict subset).
        // We use a u64 bitmask for up to 64 exports (all our sets are << 64).
        let n = exports.len();
        let max_mask = (1u64 << n) - 1; // all bits set = full set
                                        // Exclude the full set mask (that would be a complete set, not strict subset).
        (0u64..max_mask).prop_map(move |mask| {
            exports
                .iter()
                .enumerate()
                .filter_map(|(i, &e)| if mask & (1 << i) != 0 { Some(e) } else { None })
                .collect()
        })
    }

    proptest! {
        /// VP-043: Any strict subset of infusion required exports must be rejected
        /// with Err(InvalidInterface) naming the missing export.
        ///
        /// Traces to: BC-2.17.006 postcondition "Invalid plugin: component missing
        /// required exports → E-PLUGIN-001 with missing export named"
        #[test]
        fn test_BC_2_17_006_vp043_infusion_strict_subset_rejected(
            present in arb_strict_subset(INFUSION_REQUIRED_EXPORTS)
        ) {
            let result = validate_wit_interface(&present, "test-plugin.prx");
            prop_assert!(
                result.is_err(),
                "VP-043: strict subset of infusion exports must be rejected, present={:?}",
                present
            );
            let err = result.unwrap_err();
            prop_assert!(
                matches!(err, PluginError::InvalidInterface { .. }),
                "VP-043: error must be InvalidInterface, got: {:?}",
                err
            );
            // The error message must name at least one missing export.
            let err_str = format!("{:?}", err);
            let missing: Vec<_> = INFUSION_REQUIRED_EXPORTS
                .iter()
                .filter(|&&e| !present.contains(&e))
                .collect();
            prop_assert!(
                !missing.is_empty(),
                "at least one export must be missing in a strict subset"
            );
            // At minimum the first missing export must appear in the error.
            let first_missing = missing[0];
            prop_assert!(
                err_str.contains(first_missing),
                "VP-043: error message must name missing export '{}', got: {}",
                first_missing,
                err_str
            );
        }

        /// VP-043: Any strict subset of sensor required exports must be rejected.
        ///
        /// Traces to: BC-2.17.006 postcondition (sensor variant)
        #[test]
        fn test_BC_2_17_006_vp043_sensor_strict_subset_rejected(
            present in arb_strict_subset(SENSOR_REQUIRED_EXPORTS)
        ) {
            let result = validate_wit_interface(&present, "test-sensor.prx");
            prop_assert!(
                result.is_err(),
                "VP-043: strict subset of sensor exports must be rejected"
            );
            let err = result.unwrap_err();
            prop_assert!(
                matches!(err, PluginError::InvalidInterface { .. }),
                "VP-043: error must be InvalidInterface"
            );
        }

        /// VP-043: Any strict subset of action required exports must be rejected.
        ///
        /// Traces to: BC-2.17.006 postcondition (action variant)
        #[test]
        fn test_BC_2_17_006_vp043_action_strict_subset_rejected(
            present in arb_strict_subset(ACTION_REQUIRED_EXPORTS)
        ) {
            let result = validate_wit_interface(&present, "test-action.prx");
            prop_assert!(
                result.is_err(),
                "VP-043: strict subset of action exports must be rejected"
            );
            let err = result.unwrap_err();
            prop_assert!(
                matches!(err, PluginError::InvalidInterface { .. }),
                "VP-043: error must be InvalidInterface"
            );
        }
    }

    /// VP-043 positive: all required infusion exports present → Ok(PluginType::Infusion).
    ///
    /// Traces to: BC-2.17.006 postcondition "Valid plugin: exported functions present →
    /// LoadedPlugin created and registered"
    #[test]
    fn test_BC_2_17_006_vp043_complete_infusion_exports_accepted() {
        let result = validate_wit_interface(INFUSION_REQUIRED_EXPORTS, "valid-infusion.prx");
        assert!(
            result.is_ok(),
            "VP-043: complete infusion export set must be accepted, got: {:?}",
            result
        );
        assert_eq!(
            result.unwrap(),
            PluginType::Infusion,
            "VP-043: accepted infusion plugin must have type Infusion"
        );
    }

    /// VP-043 positive: all required sensor exports present → Ok(PluginType::Sensor).
    #[test]
    fn test_BC_2_17_006_vp043_complete_sensor_exports_accepted() {
        let result = validate_wit_interface(SENSOR_REQUIRED_EXPORTS, "valid-sensor.prx");
        assert!(
            result.is_ok(),
            "VP-043: complete sensor export set must be accepted, got: {:?}",
            result
        );
        assert_eq!(result.unwrap(), PluginType::Sensor);
    }

    /// VP-043 positive: all required action exports present → Ok(PluginType::Action).
    #[test]
    fn test_BC_2_17_006_vp043_complete_action_exports_accepted() {
        let result = validate_wit_interface(ACTION_REQUIRED_EXPORTS, "valid-action.prx");
        assert!(
            result.is_ok(),
            "VP-043: complete action export set must be accepted, got: {:?}",
            result
        );
        assert_eq!(result.unwrap(), PluginType::Action);
    }

    /// VP-043 empty set: no exports at all → Err(InvalidInterface).
    #[test]
    fn test_BC_2_17_006_vp043_empty_exports_rejected() {
        let result = validate_wit_interface(&[], "empty-plugin.prx");
        assert!(result.is_err(), "VP-043: empty export set must be rejected");
        assert!(
            matches!(result.unwrap_err(), PluginError::InvalidInterface { .. }),
            "VP-043: error must be InvalidInterface"
        );
    }

    /// VP-043 determinism: same input always produces same output.
    ///
    /// Traces to: BC-2.17.006 VP-043 property statement "deterministic"
    #[test]
    fn test_BC_2_17_006_vp043_deterministic() {
        let exports = &["name", "version"][..]; // missing dispatch function
        let result_a = validate_wit_interface(exports, "plugin.prx");
        let result_b = validate_wit_interface(exports, "plugin.prx");
        // Both must agree on err/ok.
        assert_eq!(
            result_a.is_err(),
            result_b.is_err(),
            "VP-043: validate_wit_interface must be deterministic"
        );
    }
}
