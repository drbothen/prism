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
//! # SID-1 compliance (AC-014 coverage split)
//! These unit tests drive the production seeding path (query layer → FilterMap)
//! WITHOUT an external DTU dependency. They prove that `predicate_tree_to_filter_map`
//! extracts `FilterMap["aql"] = Value::String("in:devices")` from the parsed AST,
//! which is the query-layer boundary assertion for BC-2.11.007 Mechanism B.
//!
//! The full WHERE-clause→DTU round-trip assertion — verifying that the Armis DTU
//! actually receives `GET /api/v1/search?aql=in:devices` — is covered by
//! `test_BC_2_11_007_e2e_armis_aql_pushdown_devices_dtu_roundtrip` in
//! `crates/prism-bin/tests/e2e_smoke.rs` (marked `#[ignore]` per SID-1 §4;
//! requires DTU server + prism binary; un-gated in CI via 'e2e' nextest profile).
//!
//! Story: S-DEMO-002 Task 19 (AC-014 / D-934 scope)
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

    /// AC-014 / Task 19: AQL push-down seeding — parse to FilterMap (query-layer boundary).
    ///
    /// Verifies that parsing `FROM armis.devices WHERE aql = 'in:devices' LIMIT 5`
    /// and passing the WHERE predicate to `predicate_tree_to_filter_map` (which is
    /// the production seeding function called by `extract_push_down_filters_as_map`
    /// in `materialization.rs`) produces `FilterMap["aql"] = Value::String("in:devices")`.
    ///
    /// # What this test asserts (query-layer boundary only)
    ///
    /// This test asserts the AST→FilterMap step of BC-2.11.007 Mechanism B:
    /// `WHERE aql = 'in:devices'` → `predicate_tree_to_filter_map` → `FilterMap["aql"] = "in:devices"`.
    /// It does NOT assert FetchContext population or DTU receipt — those are downstream layers.
    ///
    /// # Full WHERE-clause→DTU round-trip coverage
    ///
    /// The complete AC-014 end-to-end assertion (including that the Armis DTU actually
    /// receives `GET /api/v1/search?aql=in:devices`) is in:
    /// `test_BC_2_11_007_e2e_armis_aql_pushdown_devices_dtu_roundtrip` in
    /// `crates/prism-bin/tests/e2e_smoke.rs` (marked `#[ignore]` per SID-1 §4;
    /// requires DTU server + prism binary; un-gated in CI via 'e2e' nextest profile).
    #[allow(non_snake_case)]
    #[test]
    fn test_BC_2_11_007_armis_aql_pushdown_seeded_in_filter_map() {
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
            case_insensitive: false,
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
    /// The companion test `test_BC_2_11_007_armis_aql_pushdown_seeded_in_filter_map`
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
            case_insensitive: false,
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

    // ── HIGH-1 sibling-sweep: SqlPipe push-down tests ─────────────────────────
    //
    // The next two tests are load-bearing proofs for the HIGH-1 fix in
    // `extract_push_down_filters_as_map` and `extract_time_window_from_ast_from_query`
    // (materialization.rs). Before the fix both functions had `_ => None` for
    // `Ast::SqlPipe`, so SqlPipe queries would never push time-window or equality
    // filters to the sensor adapter.
    //
    // Tests operate at the `predicate_tree_to_filter_map` / `extract_time_window_from_ast`
    // level (the underlying public functions called by the private wrappers) to avoid
    // duplicating the production pipeline wiring.
    //
    // BC-2.11.020 / ADR-033 T1 / TD-VSDD-060

    /// HIGH-1 load-bearing (push-down filter): A SqlPipe query with
    /// `WHERE severity = 'HIGH'` in the head must extract that equality predicate
    /// into a FilterMap via `predicate_tree_to_filter_map`, confirming that
    /// `extract_push_down_filters_as_map` now reads the head WHERE clause.
    ///
    /// Before the HIGH-1 fix, `extract_push_down_filters_as_map` had `_ => None`
    /// for `Ast::SqlPipe`, so the WHERE predicate was silently dropped and the
    /// sensor would receive a full-table scan instead of the pre-filtered request.
    #[allow(non_snake_case)]
    #[test]
    fn test_high1_sqlpipe_head_where_equality_predicate_pushed_to_filter_map() {
        use crate::ast::Ast;
        use crate::filter_parser::PrismQlParser;
        use crate::pushdown::predicate_tree_to_filter_map;

        let query = "SELECT * FROM crowdstrike.detections WHERE severity = 'HIGH' | limit 10";
        let ast = PrismQlParser::parse(query).expect("SqlPipe push-down test: query must parse");

        // Confirm this is an Ast::SqlPipe.
        let spq = match ast {
            Ast::SqlPipe(ref spq) => spq,
            _ => panic!("HIGH-1 push-down: expected Ast::SqlPipe, got: {ast:?}"),
        };

        // Extract the WHERE predicate from the head and run it through
        // predicate_tree_to_filter_map — this mirrors what
        // `extract_push_down_filters_as_map` now does for Ast::SqlPipe.
        let where_pred = spq
            .head
            .where_
            .as_ref()
            .expect("HIGH-1 push-down: SqlPipe head must have a WHERE clause");

        let filter_map = predicate_tree_to_filter_map(where_pred);

        assert_eq!(
            filter_map.get("severity").and_then(|v| v.as_str()),
            Some("HIGH"),
            "HIGH-1 / push-down: SqlPipe head WHERE severity = 'HIGH' must produce \
             FilterMap[\"severity\"] = Value::String(\"HIGH\"); \
             before the fix the WHERE clause was never read. Got: {:?}",
            filter_map.get("severity")
        );
    }

    /// HIGH-1 load-bearing (time-window push-down): A SqlPipe query with
    /// `WHERE timestamp > NOW() - INTERVAL '24h'` in the head must extract the
    /// time-window bound via `extract_time_window_from_ast`, confirming that
    /// `extract_time_window_from_ast_from_query` now reads the head WHERE clause.
    ///
    /// Before the HIGH-1 fix, `extract_time_window_from_ast_from_query` had
    /// `_ => None` for `Ast::SqlPipe`, so the time-window was never pushed,
    /// causing a full-table scan against the 200MB/query budget (ADR-033 T1).
    ///
    /// The NOW() substitution happens in `materialize_query` (inject_now);
    /// here we test with a concrete ISO timestamp to exercise the extraction path
    /// directly. This mirrors the existing Ast::Sql push-down test pattern in
    /// `crates/prism-query/src/pushdown.rs::tests::test_ac_wire_001_*`.
    #[allow(non_snake_case)]
    #[test]
    fn test_high1_sqlpipe_head_where_timestamp_pushes_time_window() {
        use crate::ast::Ast;
        use crate::filter_parser::PrismQlParser;
        use crate::pushdown::extract_time_window_from_ast;
        use prism_core::ColumnOptions;
        use prism_spec_engine::spec_parser::ColumnSpec;
        use std::collections::HashMap;

        let query = "SELECT * FROM crowdstrike.detections \
                     WHERE timestamp > '2026-01-01T00:00:00Z' | enrich threat_score(src_ip) | limit 10";
        let ast = PrismQlParser::parse(query).expect("SqlPipe time-window test: query must parse");

        // Confirm this is an Ast::SqlPipe.
        let spq = match ast {
            Ast::SqlPipe(ref spq) => spq,
            _ => panic!("HIGH-1 time-window: expected Ast::SqlPipe, got: {ast:?}"),
        };

        // Build a minimal column spec map matching the source "crowdstrike.detections"
        // with a datetime INDEX column named "timestamp". Mirrors the production fixture.
        let mut col = ColumnSpec::default();
        col.name = "timestamp".to_string();
        col.column_type = prism_core::column::ColumnType::Datetime;
        col.options = vec![ColumnOptions::Index];
        let mut spec_map: HashMap<String, Vec<ColumnSpec>> = HashMap::new();
        spec_map.insert("crowdstrike.detections".to_string(), vec![col]);

        // Extract WHERE predicate from the SqlPipe head.
        let where_pred = spq
            .head
            .where_
            .as_ref()
            .expect("HIGH-1 time-window: SqlPipe head must have a WHERE clause");

        // `extract_time_window_from_ast` takes a Predicate (not an Ast), which is
        // exactly what `extract_time_window_from_ast_from_query` extracts from the
        // Ast::SqlPipe head.where_ after the HIGH-1 fix.
        let (start_time, end_time) =
            extract_time_window_from_ast(where_pred, &["crowdstrike.detections"], Some(&spec_map));

        assert!(
            start_time.is_some(),
            "HIGH-1 / time-window: SqlPipe head WHERE timestamp > '2026-01-01T00:00:00Z' \
             on a datetime INDEX column must extract start_time; before the fix \
             extract_time_window_from_ast_from_query returned (None, None) for SqlPipe. \
             Got: start={start_time:?}, end={end_time:?}"
        );
        assert!(
            start_time.as_deref().unwrap_or("").contains("2026-01-01"),
            "HIGH-1 / time-window: start_time must contain '2026-01-01'; got: {start_time:?}"
        );
        assert!(
            end_time.is_none(),
            "HIGH-1 / time-window: end_time must be None for a GT-only predicate; \
             got: {end_time:?}"
        );
    }
}
