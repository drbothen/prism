//! VP-031: REQUIRED columns always produce `PushDown`, never `PostFilter`.
//!
//! Property under test: For any predicate referencing a REQUIRED column from
//! any sensor spec, `classify_predicates()` MUST place the predicate in
//! `PushDownPlan.push_down` — it MUST NOT appear in `PushDownPlan.post_filter`.
//!
//! This is the core correctness invariant of BC-2.11.007 §Column Push-Down
//! Capability Taxonomy. It prevents REQUIRED columns from being silently
//! dropped to post-filter, which would cause `fan_out()` to omit mandatory
//! API parameters.
//!
//! # Proof Method
//! Proptest: `prop_compose!` generates arbitrary `Predicate` trees over the
//! set of REQUIRED column names from an arbitrary `SensorSpec`. The property
//! is checked for all generated inputs.
//!
//! # BC References
//! - BC-2.11.007 — Sensor Filter Push-Down (REQUIRED column contract)
//! - VP-031 — Required column enforcement (proptest)
//!
//! Story: S-3.02

#[cfg(test)]
mod kani_proofs {
    use proptest::prelude::*;

    use crate::{
        ast::Expr,
        pushdown::{classify_predicates, ColumnPushDownOption, Predicate},
    };
    use prism_sensors::SensorSpec;

    // -----------------------------------------------------------------------
    // Arbitrary generators
    // -----------------------------------------------------------------------

    /// Generate an arbitrary REQUIRED column name from a fixed set.
    ///
    /// Uses a static list of representative REQUIRED columns across the four
    /// built-in sensors (CrowdStrike, Cyberint, Claroty, Armis).
    fn required_column_names() -> impl Strategy<Value = String> {
        prop_oneof![
            // CrowdStrike REQUIRED columns
            Just("customer_id".to_string()),
            Just("device_id".to_string()),
            // Cyberint REQUIRED columns
            Just("org_id".to_string()),
            // Claroty REQUIRED columns
            Just("site_id".to_string()),
            // Armis REQUIRED columns (AQL supports most columns natively)
            Just("organizationId".to_string()),
        ]
    }

    prop_compose! {
        /// Generate an arbitrary `Predicate` over a REQUIRED column name.
        fn arbitrary_required_predicate()(column_name in required_column_names()) -> Predicate {
            todo!("S-3.02 — arbitrary_required_predicate prop_compose")
        }
    }

    prop_compose! {
        /// Generate a `SensorSpec` that declares `column_name` as REQUIRED.
        fn sensor_spec_with_required()(column_name in required_column_names()) -> (SensorSpec, String) {
            todo!("S-3.02 — sensor_spec_with_required prop_compose")
        }
    }

    // -----------------------------------------------------------------------
    // VP-031 property test
    // -----------------------------------------------------------------------

    proptest! {
        /// VP-031: For any REQUIRED column, `classify_predicates` always places
        /// the predicate in `push_down`, never in `post_filter`.
        ///
        /// This test is RED by design — `classify_predicates` is `todo!()`.
        /// When implemented, this property MUST pass for all generated inputs.
        ///
        /// # BC-2.11.007
        /// REQUIRED columns are the API's mandatory parameters — they MUST be
        /// pushed down or the API call will fail / return empty results.
        #[test]
        fn prop_required_columns_always_push_down(
            (spec, required_col) in sensor_spec_with_required()
        ) {
            todo!("S-3.02 — prop_required_columns_always_push_down")
        }
    }
}
