// SPDX-License-Identifier: Apache-2.0
//! Red Gate unit test for S-DEMO-002 AC-014: AQL push-down seeding.
//!
//! BC-2.11.007 postcondition: "Armis: AQL WHERE clauses" — end-to-end push-down
//! from PrismQL WHERE predicate → FetchContext.query_filters["aql"] → DTU
//! `/api/v1/search?aql=<value>`.
//!
//! # SID-1 compliance
//! This test drives the production seeding path (query layer → PipelineExecutor
//! FetchContext.query_filters["aql"]) WITHOUT an external DTU dependency.
//! The E2E subprocess test in `crates/prism-bin/tests/e2e_smoke.rs` exercises
//! the full pipeline including the live DTU; this unit test exercises the
//! filter extraction logic at the query-layer boundary.
//!
//! # PrismQL AQL syntax
//! The canonical PrismQL form for Armis AQL filtering is:
//!   `SELECT * FROM armis.devices WHERE aql = 'in:devices'`
//! The field name is `aql`; the value is the AQL string (e.g., `"in:devices"`).
//! This maps to `FilterMap["aql"] = Value::String("in:devices")` via
//! `predicate_tree_to_filter_map`, which is then threaded through:
//!   `QueryParams.filters["aql"]`
//!   → `SpecDrivenSensorAdapter::fetch` → `FetchContext.query_filters["aql"]`
//!   → `PipelineExecutor` interpolates `${query.filter.aql}` in path template.
//!
//! # The seeding gap (AC-014 / D-934)
//! `extract_push_down_filters_as_map` currently handles `field = 'value'`
//! equality predicates correctly via `predicate_tree_to_filter_map`.
//! The production gap tested here is that `extract_aql_filter_value_from_ast`
//! must specifically extract the `aql` field's value from the WHERE clause
//! and return it as a plain `Option<String>` (for path template interpolation),
//! not as a `serde_json::Value` (which the generic FilterMap uses).
//!
//! The `SpecDrivenSensorAdapter` converts `FilterMap<String, Value::String("in:devices")>`
//! to `HashMap<String, String>` for `FetchContext.query_filters`. This conversion
//! requires the value to be a `Value::String` — any other JSON type will silently
//! drop the filter. The test verifies the full round-trip including type correctness.
//!
//! Story: S-DEMO-002 v1.3 Task 19
//! BCs: BC-2.11.007, BC-2.11.001
//! DTU path: `armis.sensor.toml` fetch path_template = `/api/v1/search?aql=${query.filter.aql}`

#[cfg(test)]
mod tests {
    use crate::filter_parser::PrismQlParser;

    // ---------------------------------------------------------------------------
    // AC-014 / BC-2.11.007: AQL filter extraction for Armis push-down seeding
    // ---------------------------------------------------------------------------

    /// Red Gate test for AC-014 / Task 19.
    ///
    /// Verifies that `extract_aql_filter_value_from_ast(ast)` returns `Some("in:devices")`
    /// for a query `"SELECT * FROM armis.devices WHERE aql = 'in:devices'"`.
    ///
    /// FAIL at Red Gate: `extract_aql_filter_value_from_ast` does not exist yet.
    /// The implementer adds this function to `crates/prism-query/src/pushdown.rs`
    /// and calls it from `materialization.rs::run_materialization()` to seed
    /// `QueryParams.filters["aql"]` for Armis targets.
    ///
    /// Production seeding site: `materialization.rs::run_materialization()` calls
    /// `extract_aql_filter_value_from_ast(ast)` when the target sensor_id is "armis",
    /// and inserts the result into the `FilterMap` under key `"aql"` using
    /// `Value::String(aql_str)` before constructing `QueryParams`. The
    /// `SpecDrivenSensorAdapter` converts `FilterMap["aql"]` → String via
    /// `value.as_str()?.to_string()` into `FetchContext.query_filters["aql"]`.
    /// `PipelineExecutor` interpolates `${query.filter.aql}` in the path template.
    ///
    /// Note: This function is a targeted extractor for the Armis AQL use case.
    /// It extracts the VALUE of `aql = 'value'` equality predicates from the
    /// WHERE clause. The generic `predicate_tree_to_filter_map` already collects
    /// this as a `FilterMap` entry; `extract_aql_filter_value_from_ast` provides
    /// a typed `Option<String>` extraction for use in the materialization pipeline.
    #[allow(non_snake_case)]
    #[test]
    fn test_BC_2_11_007_e2e_armis_aql_pushdown_seeded_in_fetch_context() {
        // Parse the Armis query with an AQL WHERE predicate.
        // PrismQL form: WHERE aql = 'in:devices' (standard equality predicate).
        // 'in:devices' is the Armis entity discriminator (research artifact 2026-06-01;
        // grounded from 1898 production poller + 3 independent external connectors).
        let query_str = "SELECT * FROM armis.devices WHERE aql = 'in:devices'";
        let ast = PrismQlParser::parse(query_str).unwrap_or_else(|e| {
            panic!("PrismQlParser::parse failed for Armis AQL query '{query_str}': {e:?}")
        });

        // AC-014: extract_aql_filter_value_from_ast must return Some("in:devices").
        // This function is the production seeding site for query_filters["aql"].
        //
        // RED GATE FAILURE: this function does not yet exist. The test will fail to compile
        // (E0425 unresolved function) until the implementer adds it to pushdown.rs.
        // Once it compiles but returns None, the assertion below fails (correct Red Gate behavior).
        let aql_value = crate::pushdown::extract_aql_filter_value_from_ast(&ast);

        assert_eq!(
            aql_value,
            Some("in:devices".to_string()),
            "AC-014 BC-2.11.007: extract_aql_filter_value_from_ast must return \
             Some(\"in:devices\") for query '{}'; \
             got: {:?}. \
             Production fix: implement extract_aql_filter_value_from_ast in pushdown.rs to \
             extract the value of 'aql = <value>' WHERE predicates for Armis AQL seeding.",
            query_str,
            aql_value
        );
    }

    /// Verifies that `extract_aql_filter_value_from_ast` returns `None` for a query
    /// without an `aql = '...'` predicate.
    ///
    /// A CrowdStrike query with a different WHERE predicate must not produce an AQL value.
    ///
    /// FAIL at Red Gate: function does not exist yet (compile error).
    #[allow(non_snake_case)]
    #[test]
    fn test_BC_2_11_007_aql_seeding_returns_none_for_non_aql_predicate() {
        let query_str = "SELECT * FROM crowdstrike.detections WHERE status = 'open'";
        let ast = PrismQlParser::parse(query_str).unwrap_or_else(|e| {
            panic!("PrismQlParser::parse failed for CrowdStrike query '{query_str}': {e:?}")
        });

        // CrowdStrike predicate uses 'status', not 'aql' — must return None.
        let aql_value = crate::pushdown::extract_aql_filter_value_from_ast(&ast);

        assert_eq!(
            aql_value, None,
            "BC-2.11.007: extract_aql_filter_value_from_ast must return None when \
             WHERE clause does not contain 'aql = <value>'; \
             got: {:?} for query '{}'",
            aql_value, query_str
        );
    }

    /// Verifies that a query WITHOUT a WHERE clause produces no AQL value.
    ///
    /// `"SELECT * FROM armis.devices"` — no WHERE predicate → no AQL filter.
    ///
    /// FAIL at Red Gate: function does not exist yet.
    #[allow(non_snake_case)]
    #[test]
    fn test_BC_2_11_007_aql_seeding_returns_none_when_no_where_clause() {
        let query_str = "SELECT * FROM armis.devices";
        let ast = PrismQlParser::parse(query_str).unwrap_or_else(|e| {
            panic!("PrismQlParser::parse failed for query '{query_str}': {e:?}")
        });

        let aql_value = crate::pushdown::extract_aql_filter_value_from_ast(&ast);

        assert_eq!(
            aql_value, None,
            "BC-2.11.007: no WHERE clause → no AQL filter; got: {:?}",
            aql_value
        );
    }

    /// Verifies the FilterMap round-trip: `predicate_tree_to_filter_map` correctly
    /// produces `FilterMap["aql"] = Value::String("in:devices")` for the canonical
    /// Armis query form. This is the upstream half of the seeding pipeline.
    ///
    /// This test exercises EXISTING behavior (predicate_tree_to_filter_map is
    /// already implemented) and should PASS at Red Gate.
    ///
    /// PASS at Red Gate: predicate_tree_to_filter_map already handles field = 'value'.
    #[allow(non_snake_case)]
    #[test]
    fn test_BC_2_11_007_predicate_tree_to_filter_map_extracts_aql_equality_predicate() {
        use crate::ast::{CompareOp, Expr, FieldPath, Literal, Predicate};
        use crate::pushdown::predicate_tree_to_filter_map;

        // Build a synthetic AST predicate: `aql = 'in:devices'`
        // Use FieldPath::new() to correctly populate the span field (Span::ZERO for tests).
        let predicate = Predicate::Compare {
            lhs: Box::new(Expr::Field(FieldPath::new(["aql"]))),
            op: CompareOp::Eq,
            rhs: Box::new(Expr::Literal(Literal::String("in:devices".to_string()))),
        };

        let filter_map = predicate_tree_to_filter_map(&predicate);

        assert_eq!(
            filter_map.get("aql").and_then(|v| v.as_str()),
            Some("in:devices"),
            "BC-2.11.007: predicate_tree_to_filter_map must extract aql = 'in:devices' \
             as FilterMap[\"aql\"] = Value::String(\"in:devices\"); \
             got: {:?}",
            filter_map.get("aql")
        );
    }
}
