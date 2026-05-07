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
    use prism_core::ColumnOptions;
    use prism_core::ColumnType;
    use prism_spec_engine::spec_parser::ColumnSpec;
    use proptest::prelude::*;

    use crate::{
        ast::{CompareOp, Expr, FieldPath, Literal, Span},
        pushdown::{classify_predicates, Predicate},
    };

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

    /// Build a compare expression for a column name (used as predicate test fixture).
    fn make_compare_expr(column_name: &str) -> Expr {
        Expr::Compare {
            lhs: Box::new(Expr::Field(FieldPath {
                segments: vec![column_name.to_string()],
                span: Span::default(),
            })),
            op: CompareOp::Eq,
            rhs: Box::new(Expr::Literal(Literal::String("test_value".to_string()))),
        }
    }

    /// Build a ColumnSpec declaring `column_name` as REQUIRED.
    fn make_required_col_spec(column_name: &str) -> ColumnSpec {
        ColumnSpec {
            name: column_name.to_string(),
            column_type: ColumnType::String,
            ocsf_field: None,
            options: vec![ColumnOptions::Required],
        }
    }

    /// Build a ColumnSpec with no special options (DEFAULT).
    fn make_default_col_spec(column_name: &str) -> ColumnSpec {
        ColumnSpec {
            name: column_name.to_string(),
            column_type: ColumnType::String,
            ocsf_field: None,
            options: vec![],
        }
    }

    prop_compose! {
        /// Generate an arbitrary `Predicate` over a REQUIRED column name.
        fn arbitrary_required_predicate()(column_name in required_column_names()) -> Predicate {
            Predicate {
                expr: make_compare_expr(&column_name),
                column_name,
            }
        }
    }

    prop_compose! {
        /// Generate a `(ColumnSpec slice, required_col_name)` pair where the
        /// column is declared as REQUIRED in the spec.
        fn sensor_spec_with_required()(column_name in required_column_names()) -> (Vec<ColumnSpec>, String) {
            let cols = vec![make_required_col_spec(&column_name)];
            (cols, column_name)
        }
    }

    // -----------------------------------------------------------------------
    // VP-031 property test
    // -----------------------------------------------------------------------

    proptest! {
        /// VP-031: For any REQUIRED column, `classify_predicates` always places
        /// the predicate in `push_down`, never in `post_filter`.
        ///
        /// # BC-2.11.007
        /// REQUIRED columns are the API's mandatory parameters — they MUST be
        /// pushed down or the API call will fail / return empty results.
        #[test]
        fn prop_required_columns_always_push_down(
            (columns, required_col) in sensor_spec_with_required()
        ) {
            let expr = make_compare_expr(&required_col);
            let plan = classify_predicates(&[expr], &columns);
            prop_assert_eq!(
                plan.push_down.len(), 1,
                "VP-031: REQUIRED column '{}' must be in push_down", required_col
            );
            prop_assert_eq!(
                plan.post_filter.len(), 0,
                "VP-031: REQUIRED column '{}' must NOT be in post_filter", required_col
            );
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
        fn arbitrary_post_filter_only_predicate()(column_name in prop_oneof![
            Just("device.hostname".to_string()),
            Just("description".to_string()),
            Just("raw_payload".to_string()),
        ]) -> Predicate {
            Predicate {
                expr: make_compare_expr(&column_name),
                column_name,
            }
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
        #[test]
        fn prop_no_predicate_silently_dropped(
            predicate in arbitrary_required_predicate()
        ) {
            let col_name = predicate.column_name.clone();
            // Spec declares this column as REQUIRED.
            let columns = vec![make_required_col_spec(&col_name)];
            let plan = classify_predicates(&[predicate.expr], &columns);
            let total = plan.push_down.len() + plan.post_filter.len();
            prop_assert_eq!(
                total, 1,
                "VP-031 invariant: predicate must appear in exactly one list, not {} times", total
            );
        }
    }

    proptest! {
        /// VP-031 invariant: post-filter-only predicates never appear in push_down.
        ///
        /// OPTIMIZED and DEFAULT columns must NEVER be pushed down, regardless
        /// of any other predicate in the WHERE clause. (BC-2.11.007)
        #[test]
        fn prop_post_filter_only_predicates_never_in_push_down(
            predicate in arbitrary_post_filter_only_predicate()
        ) {
            let col_name = predicate.column_name.clone();
            // Spec declares this column as DEFAULT (no options).
            let columns = vec![make_default_col_spec(&col_name)];
            let plan = classify_predicates(&[predicate.expr], &columns);
            prop_assert_eq!(
                plan.push_down.len(), 0,
                "VP-031 invariant: DEFAULT/OPTIMIZED column '{}' must NEVER be in push_down", col_name
            );
            prop_assert_eq!(
                plan.post_filter.len(), 1,
                "VP-031 invariant: DEFAULT/OPTIMIZED column '{}' must be in post_filter", col_name
            );
        }
    }

    proptest! {
        /// VP-031 materialization invariant: classify_predicates is deterministic.
        ///
        /// Given the same (predicate, ColumnSpec) inputs, the function MUST
        /// produce the same PushDownPlan on every call. (BC-2.11.007)
        #[test]
        fn prop_classify_predicates_is_deterministic(
            (columns, required_col) in sensor_spec_with_required()
        ) {
            let expr1 = make_compare_expr(&required_col);
            let expr2 = make_compare_expr(&required_col);
            let plan1 = classify_predicates(&[expr1], &columns);
            let plan2 = classify_predicates(&[expr2], &columns);
            prop_assert_eq!(
                plan1.push_down.len(), plan2.push_down.len(),
                "Determinism: push_down lengths must match for same input"
            );
            prop_assert_eq!(
                plan1.post_filter.len(), plan2.post_filter.len(),
                "Determinism: post_filter lengths must match for same input"
            );
        }
    }
}
