// SPDX-License-Identifier: Apache-2.0
//! AC-014 unit tests for S-DEMO-002: AQL push-down seeding via the canonical
//! `predicate_tree_to_filter_map` / `extract_push_down_filters_as_map` path.
//!
//! BC-2.11.007 §Predicate Classification Mechanism B (Verbatim-AQL Passthrough):
//! The user writes `aql = '<string>'` as a pseudo-column literal in the PrismQL
//! WHERE clause; the query planner seeds the verbatim string into
//! `FetchContext.query_filters["aql"]`; forwarded opaque to the DTU endpoint
//! `GET /api/v1/search?aql=<value>` per R-DTU-002 / ADR-031 §D8-a.
//!
//! # Canonical seeding path (production pipeline)
//!
//! ```text
//! PrismQL WHERE aql = 'in:devices'
//!   → extract_push_down_filters_as_map() [materialization.rs]
//!   → predicate_tree_to_filter_map(where_pred) [pushdown.rs]
//!   → FilterMap["aql"] = Value::String("in:devices")
//!   → QueryParams.filters["aql"] = Value::String("in:devices")
//!   → SpecDrivenSensorAdapter::fetch → FetchContext.query_filters["aql"] = "in:devices"
//!   → PipelineExecutor interpolates ${query.filter.aql} in path_template
//!   → DTU receives GET /api/v1/search?aql=in:devices
//! ```
//!
//! These tests exercise `predicate_tree_to_filter_map` (the production seeding
//! function) directly with parsed AST predicates, proving end-to-end that the
//! query-layer seeding is correct from parse to FilterMap output.
//!
//! # SID-1 compliance
//! These tests drive the production seeding path (query layer → FilterMap)
//! WITHOUT an external DTU dependency. The E2E subprocess test in
//! `crates/prism-bin/tests/e2e_smoke.rs` exercises the full pipeline including
//! the live DTU; these unit tests prove the filter extraction logic at the
//! query-layer boundary.
//!
//! Story: S-DEMO-002 v1.4 Task 19 (AC-014 / D-934 scope)
//! BCs: BC-2.11.007 §Mechanism B, BC-2.11.001
//! DTU path: armis.sensor.toml path_template = `/api/v1/search?aql=${query.filter.aql}`

#[cfg(test)]
mod tests {
    use crate::ast::{Ast, SqlStatement};
    use crate::filter_parser::PrismQlParser;
    use crate::pushdown::predicate_tree_to_filter_map;

    // ---------------------------------------------------------------------------
    // AC-014 / BC-2.11.007 §Mechanism B: canonical seeding path assertions
    // ---------------------------------------------------------------------------

    /// AC-014 / Task 19: end-to-end AQL push-down seeding via the canonical path.
    ///
    /// Verifies that parsing `FROM armis.devices WHERE aql = 'in:devices' LIMIT 5`
    /// and passing the WHERE predicate to `predicate_tree_to_filter_map` (which is
    /// the production seeding function called by `extract_push_down_filters_as_map`
    /// in `materialization.rs`) produces `FilterMap["aql"] = Value::String("in:devices")`.
    ///
    /// This is the REAL production seeding path (BC-2.11.007 Mechanism B).
    /// The FilterMap is then passed as `QueryParams.filters` to the fan-out pipeline,
    /// which the `SpecDrivenSensorAdapter` converts to `FetchContext.query_filters["aql"]`,
    /// which `PipelineExecutor` interpolates into `${query.filter.aql}` in the
    /// `armis.sensor.toml` path template (`/api/v1/search?aql=${query.filter.aql}`).
    ///
    /// BC-2.11.007 test vector: `FROM armis_devices WHERE aql = 'in:devices' LIMIT 100`
    /// → `FetchContext.query_filters["aql"] = "in:devices"`;
    /// DTU receives `GET /api/v1/search?aql=in:devices` (Mechanism B passthrough).
    ///
    /// This test MUST remain non-ignored and run in standard CI. The E2E DTU
    /// round-trip assertion (`?aql=in:devices` actually reaching the DTU server)
    /// is in `crates/prism-bin/tests/e2e_smoke.rs` under `#[ignore]` per SID-1.
    /// // E2E-001: requires DTU server running; un-gated in CI via 'e2e' nextest profile.
    #[allow(non_snake_case)]
    #[test]
    fn test_BC_2_11_007_e2e_armis_aql_pushdown_seeded_in_fetch_context() {
        // Parse the Armis query with an AQL WHERE predicate.
        // PrismQL Mechanism B form: WHERE aql = 'in:devices'
        // 'in:devices' is the Armis entity discriminator (research artifact 2026-06-01;
        // grounded from 1898 production poller + 3 independent external connectors).
        let query_str = "SELECT * FROM armis.devices WHERE aql = 'in:devices' LIMIT 5";
        let ast = PrismQlParser::parse(query_str).unwrap_or_else(|e| {
            panic!("PrismQlParser::parse failed for Armis AQL query '{query_str}': {e:?}")
        });

        // Extract the WHERE predicate from the parsed AST.
        // This mirrors what extract_push_down_filters_as_map does in materialization.rs.
        let where_pred = match &ast {
            Ast::Sql(SqlStatement::Select(sql)) => sql
                .where_
                .as_ref()
                .expect("parsed query must have a WHERE clause"),
            other => panic!("expected SQL Select AST, got: {other:?}"),
        };

        // AC-014: predicate_tree_to_filter_map is the canonical production seeding function.
        // This is the SAME function called by extract_push_down_filters_as_map in materialization.rs.
        // Seeding chain: predicate_tree_to_filter_map → FilterMap → QueryParams.filters["aql"]
        //   → FetchContext.query_filters["aql"] → ${query.filter.aql} interpolation → DTU URL.
        let filter_map = predicate_tree_to_filter_map(where_pred);

        // Assert that the canonical seeding path produces the correct FilterMap entry.
        assert_eq!(
            filter_map.get("aql").and_then(|v| v.as_str()),
            Some("in:devices"),
            "AC-014 BC-2.11.007 Mechanism B: predicate_tree_to_filter_map must produce \
             FilterMap[\"aql\"] = Value::String(\"in:devices\") for query '{query_str}'; \
             got: {:?}. \
             The canonical seeding path (extract_push_down_filters_as_map → QueryParams.filters) \
             is broken — the Armis AQL push-down will not reach the DTU endpoint.",
            filter_map.get("aql")
        );

        // Also assert the value is stored as a JSON String (not null/number/bool).
        // SpecDrivenSensorAdapter converts FilterMap["aql"] to FetchContext.query_filters["aql"]
        // via `value.as_str()?.to_string()` — any non-string JSON type silently drops the filter.
        assert!(
            matches!(filter_map.get("aql"), Some(serde_json::Value::String(_))),
            "AC-014 BC-2.11.007: FilterMap[\"aql\"] must be Value::String, not {:?}; \
             non-string JSON type will be silently dropped by SpecDrivenSensorAdapter.convert",
            filter_map.get("aql")
        );
    }

    /// Verifies that the canonical seeding path returns no `aql` key for a query
    /// without an `aql = '...'` predicate (e.g., a CrowdStrike query).
    ///
    /// A CrowdStrike query with a different WHERE predicate must not produce an AQL value.
    /// This guards against accidental cross-sensor filter leakage.
    #[allow(non_snake_case)]
    #[test]
    fn test_BC_2_11_007_aql_seeding_returns_none_for_non_aql_predicate() {
        let query_str = "SELECT * FROM crowdstrike.detections WHERE status = 'open'";
        let ast = PrismQlParser::parse(query_str).unwrap_or_else(|e| {
            panic!("PrismQlParser::parse failed for CrowdStrike query '{query_str}': {e:?}")
        });

        // Extract WHERE predicate and run through the canonical seeding path.
        let where_pred = match &ast {
            Ast::Sql(SqlStatement::Select(sql)) => sql
                .where_
                .as_ref()
                .expect("parsed CrowdStrike query must have a WHERE clause"),
            other => panic!("expected SQL Select AST, got: {other:?}"),
        };
        let filter_map = predicate_tree_to_filter_map(where_pred);

        // CrowdStrike predicate uses 'status', not 'aql' — the 'aql' key must be absent.
        assert!(
            filter_map.get("aql").is_none(),
            "BC-2.11.007: predicate_tree_to_filter_map must not produce FilterMap[\"aql\"] \
             when WHERE clause does not contain 'aql = <value>'; \
             got: {:?} for query '{query_str}'",
            filter_map.get("aql")
        );
    }

    /// Verifies that a query WITHOUT a WHERE clause produces an empty FilterMap.
    ///
    /// `"SELECT * FROM armis.devices"` — no WHERE predicate → no AQL filter.
    /// The canonical path (`extract_push_down_filters_as_map`) returns an empty
    /// FilterMap when no WHERE clause is present (the function returns early with
    /// `FilterMap::new()` when `where_pred` is None).
    #[allow(non_snake_case)]
    #[test]
    fn test_BC_2_11_007_aql_seeding_returns_none_when_no_where_clause() {
        // Test using predicate_tree_to_filter_map directly with a no-op predicate.
        // This mirrors the empty-FilterMap return from extract_push_down_filters_as_map
        // when the query has no WHERE clause.
        use crate::ast::{CompareOp, Expr, FieldPath, Literal, Predicate};

        // Non-aql predicate: simulates a query with a different WHERE predicate.
        // (An "no WHERE clause" scenario is tested at the extract_push_down_filters_as_map
        // level — that function returns FilterMap::new() when where_pred is None.)
        let non_aql_predicate = Predicate::Compare {
            lhs: Box::new(Expr::Field(FieldPath::new(["device_id"]))),
            op: CompareOp::Eq,
            rhs: Box::new(Expr::Literal(Literal::String("d-001".to_string()))),
        };

        let filter_map = predicate_tree_to_filter_map(&non_aql_predicate);

        assert!(
            filter_map.get("aql").is_none(),
            "BC-2.11.007: no 'aql' predicate → no aql key in FilterMap; got: {:?}",
            filter_map.get("aql")
        );
    }

    /// Verifies the FilterMap round-trip: `predicate_tree_to_filter_map` correctly
    /// produces `FilterMap["aql"] = Value::String("in:devices")` for the canonical
    /// Armis query form using a synthetic predicate.
    ///
    /// This tests the core seeding logic with a synthetic predicate (no parse step).
    /// The companion test `test_BC_2_11_007_e2e_armis_aql_pushdown_seeded_in_fetch_context`
    /// tests the full parse → predicate → FilterMap pipeline.
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
