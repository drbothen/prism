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

// Red Gate stub file: proptest bodies are todo!() — imports, unused variables,
// unreachable code, and diverging sub-expressions are expected in stub phase.
#[allow(
    unused_imports,
    unused_variables,
    unreachable_code,
    clippy::diverging_sub_expression
)]
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
        fn arbitrary_required_predicate()(_column_name in required_column_names()) -> Predicate {
            todo!("S-3.02 — arbitrary_required_predicate prop_compose")
        }
    }

    prop_compose! {
        /// Generate a `SensorSpec` that declares `column_name` as REQUIRED.
        fn sensor_spec_with_required()(_column_name in required_column_names()) -> (SensorSpec, String) {
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
            (_spec, _required_col) in sensor_spec_with_required()
        ) {
            todo!("S-3.02 — prop_required_columns_always_push_down")
        }
    }

    // -----------------------------------------------------------------------
    // VP-031 invariant: no predicate silently dropped
    // -----------------------------------------------------------------------

    prop_compose! {
        /// Generate a predicate that should end up in post_filter only.
        ///
        /// These represent OPTIMIZED or DEFAULT columns that must never be in
        /// push_down. (BC-2.11.007 classification invariant)
        fn arbitrary_post_filter_only_predicate()(_column_name in prop_oneof![
            Just("device.hostname".to_string()),
            Just("description".to_string()),
            Just("raw_payload".to_string()),
        ]) -> Predicate {
            todo!("S-3.02 — arbitrary_post_filter_only_predicate prop_compose")
        }
    }

    proptest! {
        /// VP-031 invariant: no predicate is silently dropped.
        ///
        /// For any predicate, it MUST appear in EXACTLY one of push_down or
        /// post_filter — never in both, never in neither.
        ///
        /// BC-2.11.007 invariant: "A predicate that cannot be pushed down is
        /// never silently dropped — it is always applied as a post-filter."
        ///
        /// This test is RED by design — `classify_predicates` is `todo!()`.
        #[test]
        fn prop_no_predicate_silently_dropped(
            _predicate in arbitrary_required_predicate()
        ) {
            todo!("S-3.02 — prop_no_predicate_silently_dropped")
        }
    }

    proptest! {
        /// VP-031 invariant: post-filter-only predicates never appear in push_down.
        ///
        /// OPTIMIZED and DEFAULT columns must NEVER be pushed down, regardless
        /// of any other predicate in the WHERE clause. (BC-2.11.007)
        ///
        /// This test is RED by design — `classify_predicates` is `todo!()`.
        #[test]
        fn prop_post_filter_only_predicates_never_in_push_down(
            _predicate in arbitrary_post_filter_only_predicate()
        ) {
            todo!("S-3.02 — prop_post_filter_only_predicates_never_in_push_down")
        }
    }

    proptest! {
        /// VP-031 materialization invariant: classify_predicates is deterministic.
        ///
        /// Given the same (predicate, SensorSpec) inputs, the function MUST
        /// produce the same PushDownPlan on every call. (BC-2.11.007 "canonical
        /// form for push-down filter translation before cache key input")
        ///
        /// This test is RED by design — `classify_predicates` is `todo!()`.
        #[test]
        fn prop_classify_predicates_is_deterministic(
            (_spec, _required_col) in sensor_spec_with_required()
        ) {
            todo!("S-3.02 — prop_classify_predicates_is_deterministic")
        }
    }
}
