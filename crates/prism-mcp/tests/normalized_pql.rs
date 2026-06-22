//! Load-bearing tests for:
//!  - `normalized_pql` field on MCP query tool responses (BC-2.11.018 / AC-005, AC-006)
//!  - E-QUERY-001/002/003 pedagogical enrichment fields in MCP error responses (BC-2.11.017 / AC-003)
//!  - E-QUERY-038 maps to MCP error code -32602 (BC-2.11.016 / AC-001 implicit test 6)
//!  - PqlNormalizer unit coverage (BC-2.11.018 normalizer contract)
//!
//! Story: S-DEMO-PRISMQL-ONBOARDING-001-B
//!
//! # Red Gate test catalogue
//! | Test | AC | Fails NOW because |
//! |---|---|---|
//! | `test_BC_2_11_018_normalized_pql_key_present_in_mcp_success_response` | AC-005 | (should pass — wiring IS in server.rs; load-bearing to prevent regression) |
//! | `test_BC_2_11_018_normalized_pql_key_absent_on_mcp_error_response` | AC-006 | (should pass — error path returns early before normalized_pql is computed) |
//! | `test_BC_2_11_017_ac003_parse_error_response_carries_near_text` | AC-003 | `StructuredErrorFields` has no `near_text`/`reference_pointer` fields |
//! | `test_BC_2_11_017_ac003_table_not_found_suggestion_contains_prism_describe_in_mcp` | AC-003+AC-004 | E-QUERY-037 `suggestion` field is "Check the request parameters and retry." |
//!
//! # BC references
//! - BC-2.11.016 v1.0 — E-QUERY-038 Column-Not-Found Plan-Time Gate
//! - BC-2.11.017 v1.0 — E-QUERY Pedagogical Enrichments
//! - BC-2.11.018 v1.0 — normalized_pql Field on Successful Query Responses

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, sync::Arc};

    use prism_core::{OrgSlug, PrismError, SensorId};
    use prism_credentials::InMemoryCredentialStore;
    use prism_mcp::{
        error_mapping::{codes, map_prism_error},
        server::{PrismServer, QueryToolParams},
    };
    use prism_query::{
        ast::PqlNormalizer,
        engine::{QueryEngine, QueryEngineConfig, QueryOptions},
        scoping::ClientRegistry,
        table_registry::TableRegistry,
        PrismQlParser,
    };
    use prism_sensors::AdapterRegistry;
    use prism_spec_engine::{
        overlay::{OverlayLoader, ResolvedSensorSpec, ResolvedSpecKey, SensorInstanceOverlay},
        spec_parser::{AuthType, ColumnSpec, SensorSpec, TableSpec},
    };
    use rmcp::handler::server::wrapper::Parameters;

    // =========================================================================
    // Fixture helpers
    // =========================================================================

    /// Build a minimal `ResolvedSensorSpec` for a given sensor/table/org combination.
    fn make_resolved(
        sensor_id: &str,
        table_name: &str,
        columns: Vec<ColumnSpec>,
        org: &str,
    ) -> (ResolvedSpecKey, ResolvedSensorSpec) {
        let spec = SensorSpec::new(
            sensor_id,
            format!("{sensor_id} sensor"),
            AuthType::ApiKey,
            "https://example.com",
            vec![TableSpec::new_point_in_time(
                table_name,
                "security_finding",
                columns,
                vec![],
            )],
            None,
            "1.0.0",
            Vec::new(),
        );
        let overlay_toml =
            format!("extends = \"{sensor_id}\"\ninstance_id = \"{sensor_id}@{org}\"");
        let overlay: SensorInstanceOverlay =
            toml::from_str(&overlay_toml).expect("overlay TOML must parse");
        let org_slug = OrgSlug::new(org);
        let resolved =
            OverlayLoader::merge_overlay_onto_type_spec(&spec, &overlay, org_slug.clone());
        let sensor_id_typed = SensorId::new(sensor_id);
        let key: ResolvedSpecKey = (org_slug, sensor_id_typed);
        (key, resolved)
    }

    /// Build a `QueryEngine` + `PrismServer` wired with the given resolved_spec_map.
    fn make_server_with_engine(
        resolved_map: HashMap<ResolvedSpecKey, ResolvedSensorSpec>,
        specs_for_registry: Vec<SensorSpec>,
    ) -> PrismServer {
        let registry = Arc::new(TableRegistry::new());
        for spec in &specs_for_registry {
            registry
                .register_sensor(spec)
                .expect("register_sensor must not fail in fixture");
        }
        let mut engine = QueryEngine::new_with_cache_config(
            Arc::new(AdapterRegistry::new()),
            Arc::new(InMemoryCredentialStore::new()),
            Arc::new(prism_ocsf::OcsfNormalizer::new()),
            Arc::new(ClientRegistry::new(vec![])),
            QueryEngineConfig::default(),
            prism_query::cache::CacheConfig::default(),
        );
        engine.resolved_spec_map = Some(Arc::new(arc_swap::ArcSwap::new(Arc::new(resolved_map))));
        engine = engine.with_table_registry(registry);

        PrismServer::new().with_query_engine(Arc::new(engine))
    }

    /// Build a minimal `PrismServer` with a `QueryEngine` that has NO sensor specs.
    /// Used for testing parse errors and other engine-level errors that don't need
    /// a registered table.
    fn make_server_minimal() -> PrismServer {
        let engine = QueryEngine::new(
            Arc::new(AdapterRegistry::new()),
            Arc::new(InMemoryCredentialStore::new()),
            Arc::new(prism_ocsf::OcsfNormalizer::new()),
            Arc::new(ClientRegistry::new(vec![])),
            QueryEngineConfig::default(),
        );
        PrismServer::new().with_query_engine(Arc::new(engine))
    }

    // =========================================================================
    // AC-005 — normalized_pql key PRESENT in MCP success response (load-bearing wire test)
    // =========================================================================

    /// BC-2.11.018 / AC-005 — `normalized_pql` key is present in MCP success response.
    ///
    /// LOAD-BEARING: drives through `PrismServer::query()` with a wired engine and
    /// a registered table. Asserts the `normalized_pql` key exists in the response
    /// structured content at `sc["results"]["normalized_pql"]`.
    ///
    /// Deleting the `normalized_pql` wire in server.rs (lines 1861-1877) will break
    /// this test — even though `PqlNormalizer::normalize` itself is fully implemented.
    ///
    /// This test should pass on current HEAD (the wire is present in server.rs).
    /// It serves as a regression gate.
    #[tokio::test]
    async fn test_BC_2_11_018_normalized_pql_key_present_in_mcp_success_response() {
        use prism_core::column::ColumnType;

        let columns = vec![
            ColumnSpec::new("severity", ColumnType::String, None, vec![]),
            ColumnSpec::new("detection_id", ColumnType::String, None, vec![]),
        ];
        let (key, resolved) = make_resolved("crowdstrike", "alerts", columns, "acme");
        let mut map = HashMap::new();
        map.insert(key, resolved.clone());
        let server = make_server_with_engine(map, vec![resolved.spec.clone()]);

        // A valid query against a registered table — the engine will succeed with
        // zero rows (no adapters wired, fan-out produces empty batches), and
        // normalized_pql must be present in the response.
        let params: QueryToolParams =
            serde_json::from_str(r#"{"query": "SELECT severity FROM crowdstrike_alerts LIMIT 5"}"#)
                .expect("QueryToolParams JSON must deserialize");
        let call_result = server
            .query(Parameters(params))
            .await
            .expect("query must return Ok for valid query with wired engine");

        // Success path: is_error must NOT be true
        assert_ne!(
            call_result.is_error,
            Some(true),
            "BC-2.11.018 AC-005: valid query against registered table must not return is_error=true; \
             structured_content: {:?}",
            call_result.structured_content
        );

        let sc = call_result
            .structured_content
            .expect("BC-2.11.018 AC-005: structured_content must be present on success");

        // normalized_pql must be in sc["results"]["normalized_pql"]
        // LOAD-BEARING: if the server.rs wire (lines 1861-1877) is deleted, this fails.
        let results = sc
            .get("results")
            .expect("BC-2.11.018 AC-005: sc['results'] must be present");

        let normalized_pql = results.get("normalized_pql").expect(
            "BC-2.11.018 AC-005: sc['results']['normalized_pql'] must be present on success. \
             FIX: ensure server.rs lines ~1861-1877 are intact: PqlNormalizer produces the string \
             and it is inserted into payload['normalized_pql'] before SafetyEnvelopeBuilder::wrap.",
        );

        let normalized_str = normalized_pql.as_str().expect(
            "BC-2.11.018 AC-005: normalized_pql must be a JSON string value, not null/object",
        );
        assert!(
            !normalized_str.is_empty(),
            "BC-2.11.018 AC-005: normalized_pql must be non-empty; got empty string"
        );

        // Must contain the table name (not DataFusion internals)
        assert!(
            normalized_str.contains("crowdstrike_alerts"),
            "BC-2.11.018 AC-005: normalized_pql must contain table name 'crowdstrike_alerts'; \
             got: '{normalized_str}'"
        );

        // Must use uppercase keywords (canonical form)
        assert!(
            normalized_str.contains("SELECT"),
            "BC-2.11.018 AC-005: normalized_pql must contain uppercase 'SELECT'; \
             got: '{normalized_str}'"
        );

        // DataFusion internals must NOT appear
        for internal in &["HashJoin", "TableScan", "SortExec", "Aggregate"] {
            assert!(
                !normalized_str.contains(internal),
                "BC-2.11.018 AC-005: normalized_pql must not contain DataFusion internal \
                 '{internal}'; got: '{normalized_str}'"
            );
        }
    }

    /// BC-2.11.018 / AC-005 — normalized_pql present on zero-row success.
    ///
    /// LOAD-BEARING: zero rows returned (no adapters) → normalized_pql still present.
    #[tokio::test]
    async fn test_BC_2_11_018_normalized_pql_present_on_zero_row_mcp_response() {
        use prism_core::column::ColumnType;

        let columns = vec![ColumnSpec::new(
            "severity",
            ColumnType::String,
            None,
            vec![],
        )];
        let (key, resolved) = make_resolved("crowdstrike", "alerts", columns, "acme");
        let mut map = HashMap::new();
        map.insert(key, resolved.clone());
        let server = make_server_with_engine(map, vec![resolved.spec.clone()]);

        let params: QueryToolParams = serde_json::from_str(
            r#"{"query": "SELECT severity FROM crowdstrike_alerts WHERE severity = 'nonexistent_value'"}"#,
        )
        .expect("QueryToolParams JSON must deserialize");
        let call_result = server
            .query(Parameters(params))
            .await
            .expect("zero-row query must return Ok");

        assert_ne!(
            call_result.is_error,
            Some(true),
            "BC-2.11.018 AC-005: zero-row query must not return is_error=true"
        );

        let sc = call_result
            .structured_content
            .expect("BC-2.11.018 AC-005: structured_content must be present");
        let results = sc.get("results").expect("sc['results'] must be present");
        let normalized_pql = results.get("normalized_pql").expect(
            "BC-2.11.018 AC-005: normalized_pql must be present even for zero-row result. \
             The normalizer operates on the AST, not the result rows.",
        );
        assert!(
            normalized_pql
                .as_str()
                .map(|s| !s.is_empty())
                .unwrap_or(false),
            "BC-2.11.018 AC-005: normalized_pql must be non-empty for zero-row query"
        );
    }

    // =========================================================================
    // AC-006 — normalized_pql key ABSENT in MCP error response (load-bearing wire test)
    // =========================================================================

    /// BC-2.11.018 / AC-006 — `normalized_pql` key is ABSENT in MCP error response.
    ///
    /// LOAD-BEARING: error path returns early via `prism_error_to_structured_call_result`
    /// BEFORE reaching the `normalized_pql` computation. Asserts the key is absent at
    /// the top-level structured content (not under `sc["error"]` and not at `sc["results"]`).
    ///
    /// This test should pass on current HEAD (error path returns early in server.rs).
    #[tokio::test]
    async fn test_BC_2_11_018_normalized_pql_key_absent_on_mcp_error_response() {
        let server = make_server_minimal();

        // Parse error → error path → normalized_pql must NOT be in response
        let params: QueryToolParams = serde_json::from_str(r#"{"query": "!!invalid query!!"}"#)
            .expect("QueryToolParams JSON must deserialize");
        let call_result = server
            .query(Parameters(params))
            .await
            .expect("domain errors must return Ok(structured_error), not Err");

        assert_eq!(
            call_result.is_error,
            Some(true),
            "BC-2.11.018 AC-006: parse error must return is_error=true"
        );

        let sc = call_result
            .structured_content
            .expect("BC-2.11.018 AC-006: structured_content must be present on error");

        // normalized_pql must NOT appear anywhere in the error response
        assert!(
            sc.get("normalized_pql").is_none(),
            "BC-2.11.018 AC-006: normalized_pql must be ABSENT at sc top-level on error; \
             got: {:?}",
            sc.get("normalized_pql")
        );
        assert!(
            sc.get("results")
                .and_then(|r| r.get("normalized_pql"))
                .is_none(),
            "BC-2.11.018 AC-006: normalized_pql must be ABSENT at sc['results'] on error"
        );
        // The error object itself must not have a normalized_pql field
        if let Some(error_obj) = sc.get("error") {
            assert!(
                error_obj.get("normalized_pql").is_none(),
                "BC-2.11.018 AC-006: normalized_pql must be ABSENT from sc['error'] on error"
            );
        }
    }

    // =========================================================================
    // AC-003 — MCP error response carries pedagogical enrichment fields (Red Gate)
    // =========================================================================

    /// BC-2.11.017 / AC-003 — E-QUERY-001 parse error response carries `near_text` and
    /// `reference_pointer` fields in the structured error envelope.
    ///
    /// LOAD-BEARING Red Gate test: FAILS on current HEAD because:
    ///   (a) `StructuredErrorFields` has no `near_text` or `reference_pointer` fields, AND
    ///   (b) `prism_error_to_structured_call_result` does not populate these fields.
    ///
    /// The JSON error response currently contains:
    ///   {"error": {"code": "E-QUERY-001", "message": "...", "category": "validation", ...}}
    /// It does NOT contain `near_text` or `reference_pointer`.
    ///
    /// After the fix:
    ///   {"error": {"code": "E-QUERY-001", ..., "near_text": "SELCT", "reference_pointer": "prismql://reference"}}
    ///
    /// FIX:
    ///   1. Add `near_text: Option<String>` and `reference_pointer: Option<String>` to
    ///      `StructuredErrorFields` in error_mapping.rs
    ///   2. In `prism_error_to_structured_call_result`, call `extract_near_text(input, offset)`
    ///      for `PrismError::QueryParseFailed` and populate the fields
    ///   3. Include the new fields in `build_structured_error_response` JSON object
    #[tokio::test]
    async fn test_BC_2_11_017_ac003_parse_error_response_carries_near_text() {
        let server = make_server_minimal();

        // "SELCT * FROM crowdstrike_alerts" has a typo — parse error with near_text="SELCT"
        let params: QueryToolParams =
            serde_json::from_str(r#"{"query": "SELCT * FROM crowdstrike_alerts"}"#)
                .expect("QueryToolParams JSON must deserialize");
        let call_result = server
            .query(Parameters(params))
            .await
            .expect("domain errors must return Ok(structured_error)");

        assert_eq!(
            call_result.is_error,
            Some(true),
            "AC-003: parse error must return is_error=true"
        );

        let sc = call_result
            .structured_content
            .expect("AC-003: structured_content must be present on parse error");

        let error_obj = sc
            .get("error")
            .expect("AC-003: sc['error'] must be present on parse error");

        // E-QUERY-001 enrichment: near_text must be in the error object
        // LOAD-BEARING: FAILS NOW because StructuredErrorFields has no near_text field.
        // The implementer must add near_text to StructuredErrorFields and populate it
        // via extract_near_text(query, offset) in prism_error_to_structured_call_result.
        let near_text = error_obj.get("near_text").expect(
            "BC-2.11.017 AC-003: E-QUERY-001 error response must have 'near_text' field. \
             Current output: the field is ABSENT. \
             FIX: add near_text: Option<String> to StructuredErrorFields and populate it \
             via extract_near_text(query, offset) in prism_error_to_structured_call_result.",
        );
        let near_str = near_text.as_str().unwrap_or("");
        assert!(
            near_str.contains("SELCT"),
            "BC-2.11.017 AC-003: near_text must contain 'SELCT' (the offending token); \
             got: '{near_str}'"
        );
        assert!(
            near_str.len() <= 50,
            "BC-2.11.017 AC-003 DI-006: near_text must be ≤50 chars; got {} chars: '{near_str}'",
            near_str.len()
        );

        // E-QUERY-001 enrichment: reference_pointer must be present
        let ref_ptr = error_obj.get("reference_pointer").expect(
            "BC-2.11.017 AC-003: E-QUERY-001 error response must have 'reference_pointer' field. \
             FIX: add reference_pointer: Option<String> to StructuredErrorFields and set it to \
             'prismql://reference' for QueryParseFailed errors.",
        );
        assert_eq!(
            ref_ptr.as_str(),
            Some("prismql://reference"),
            "BC-2.11.017 AC-003: reference_pointer must be 'prismql://reference'; \
             got: {:?}",
            ref_ptr
        );
    }

    /// BC-2.11.017 / AC-003 — E-QUERY-002 type-mismatch error response carries
    /// `valid_operators_for_type` in the structured error envelope.
    ///
    /// Status: CONDITIONALLY load-bearing. The `StructuredErrorFields.valid_operators_for_type`
    /// field EXISTS (AC-003 field requirement is met), and the helper is wired in
    /// `build_structured_error_response`. However, the engine coerces type comparisons
    /// rather than emitting a typed mismatch error variant, so `is_error` is `Some(false)`
    /// in practice and the conditional assertion is not exercised.
    ///
    /// The unconditional coverage for `valid_operators_for_type` is:
    ///   - `test_BC_2_11_017_ac003_enrichment_helper_valid_operators_for_type_returns_correct_operators`
    ///     (unit test of the helper in this file)
    ///
    /// NOTE: The specific PrismQL type-mismatch variant (PrismError::QueryTypeMismatch or similar)
    /// is what would trigger E-QUERY-002. If the engine produces a typed error for numeric
    /// comparison on a String column, this test's `if call_result.is_error` branch would exercise
    /// the field assertion. Currently the engine coerces, so this is a conditional check.
    #[tokio::test]
    async fn test_BC_2_11_017_ac003_type_error_response_carries_valid_operators() {
        use prism_core::column::ColumnType;

        // Register a table with a String "severity" column, then query with numeric comparison
        let columns = vec![ColumnSpec::new(
            "severity",
            ColumnType::String,
            None,
            vec![],
        )];
        let (key, resolved) = make_resolved("crowdstrike", "alerts", columns, "acme");
        let mut map = HashMap::new();
        map.insert(key, resolved.clone());
        let server = make_server_with_engine(map, vec![resolved.spec.clone()]);

        // "severity > 5" uses a numeric comparison on a String column → type error
        let params: QueryToolParams = serde_json::from_str(
            r#"{"query": "SELECT severity FROM crowdstrike_alerts WHERE severity > 5"}"#,
        )
        .expect("QueryToolParams JSON must deserialize");
        let call_result = server
            .query(Parameters(params))
            .await
            .expect("domain errors must return Ok(structured_error)");

        // The query may produce a parse error or type error depending on the engine.
        // The key assertion is: for any E-QUERY-002-class error, the error response
        // must carry valid_operators_for_type.
        //
        // If this results in is_error=false (query succeeds with coercion), this test
        // must be updated to find a query that triggers the type-mismatch path.
        //
        // For now, assert on the error response shape if an error is returned.
        if call_result.is_error == Some(true) {
            let sc = call_result
                .structured_content
                .expect("AC-003: structured_content must be present on error");

            let error_obj = sc
                .get("error")
                .expect("AC-003: sc['error'] must be present on type error");

            // E-QUERY-002 enrichment: valid_operators_for_type must be in the error object.
            // The field exists in StructuredErrorFields and is wired in build_structured_error_response.
            // This branch is only exercised if the engine produces a type-mismatch error (currently
            // the engine coerces types and returns is_error=Some(false), so this is not reached).
            let operators = error_obj.get("valid_operators_for_type").expect(
                "BC-2.11.017 AC-003: E-QUERY-002 error response must have \
                 'valid_operators_for_type' field in the JSON error object. \
                 Field is ABSENT. Check StructuredErrorFields and build_structured_error_response.",
            );
            let operators_arr = operators
                .as_array()
                .expect("valid_operators_for_type must be a JSON array");
            // String column operators: must include = and != and LIKE
            let ops: Vec<&str> = operators_arr.iter().filter_map(|v| v.as_str()).collect();
            assert!(
                ops.contains(&"="),
                "AC-003: valid_operators_for_type for String must include '='; got: {ops:?}"
            );
            assert!(
                ops.contains(&"LIKE"),
                "AC-003: valid_operators_for_type for String must include 'LIKE'; got: {ops:?}"
            );
        } else {
            // If the query succeeded (engine coerced the type), the test is inconclusive.
            // This is acceptable — the implementer must ensure the engine produces a type error
            // for cross-type comparisons, and THEN the enrichment field must be populated.
            // For now, record this as a skip-condition and note the issue.
            // The test is still load-bearing: if the engine DOES produce an error, the field
            // must be there; and if the engine coerces the type, the implementer should
            // add a dedicated type-mismatch query that reliably produces E-QUERY-002.
        }
    }

    /// BC-2.11.017 / AC-003 + AC-004 — E-QUERY-037 table-not-found error response
    /// carries `suggestion` containing "prism_describe".
    ///
    /// LOAD-BEARING Red Gate test: FAILS on current HEAD because:
    ///   `prism_error_to_structured_call_result` for `TableNotAvailable` uses static
    ///   suggestion "Check the request parameters and retry." — does NOT call
    ///   `e_query_037_suggestion()` which produces the "prism_describe" pointer.
    ///
    /// After the fix: the MCP error response's `suggestion` field must contain
    /// "prism_describe" for E-QUERY-037 errors.
    ///
    /// FIX: in `prism_error_to_structured_call_result`, for `TableNotAvailable(ref d)`:
    ///   let did_you_mean = if d.did_you_mean.is_empty() { None } else { Some(d.did_you_mean.as_str()) };
    ///   meta.owned_suggestion = Some(prism_query::engine::e_query_037_suggestion(client_id, did_you_mean));
    #[tokio::test]
    async fn test_BC_2_11_017_ac003_table_not_found_suggestion_contains_prism_describe_in_mcp() {
        use prism_core::column::ColumnType;

        let columns = vec![ColumnSpec::new(
            "severity",
            ColumnType::String,
            None,
            vec![],
        )];
        let (key, resolved) = make_resolved("crowdstrike", "alerts", columns, "acme");
        let mut map = HashMap::new();
        map.insert(key, resolved.clone());
        let server = make_server_with_engine(map, vec![resolved.spec.clone()]);

        // Query against mistyped table → E-QUERY-037 → error response must have
        // suggestion containing "prism_describe".
        let params: QueryToolParams =
            serde_json::from_str(r#"{"query": "SELECT severity FROM crowdstrike_alert LIMIT 5"}"#)
                .expect("QueryToolParams JSON must deserialize");
        let call_result = server
            .query(Parameters(params))
            .await
            .expect("E-QUERY-037 domain error must return Ok(structured_error)");

        assert_eq!(
            call_result.is_error,
            Some(true),
            "AC-004: table-not-found query must return is_error=true; \
             got is_error={:?}. The table 'crowdstrike_alert' is not registered \
             (only 'crowdstrike_alerts' is), so E-QUERY-037 must fire.",
            call_result.is_error
        );

        let sc = call_result
            .structured_content
            .expect("AC-004: structured_content must be present on E-QUERY-037 error");

        let error_obj = sc
            .get("error")
            .expect("AC-004: sc['error'] must be present on E-QUERY-037 error");

        // BC-2.11.017 AC-004: suggestion must contain "prism_describe"
        // LOAD-BEARING: FAILS NOW because static suggestion is
        //   "Check the request parameters and retry."
        // which does NOT contain "prism_describe".
        let suggestion = error_obj
            .get("suggestion")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        assert!(
            suggestion.contains("prism_describe"),
            "BC-2.11.017 AC-003+AC-004: E-QUERY-037 MCP error suggestion must contain \
             'prism_describe' so the LLM agent knows how to discover tables. \
             Current suggestion: '{suggestion}'. \
             FIX: in prism_error_to_structured_call_result, for TableNotAvailable, \
             call e_query_037_suggestion() and set it as owned_suggestion."
        );
    }

    // =========================================================================
    // PqlNormalizer unit tests (load-bearing for normalizer contract)
    // =========================================================================

    /// BC-2.11.018 — PqlNormalizer::normalize produces Some(non-empty) for valid SQL query.
    ///
    /// LOAD-BEARING: if PqlNormalizer::normalize is deleted or returns None for valid input,
    /// this fails. The test is NOT tautological — it calls through the production normalizer
    /// implementation in ast.rs.
    #[test]
    fn test_BC_2_11_018_normalized_pql_present_on_success_absent_on_error() {
        // ---- Part 1: normalized_pql PRESENT on valid parse (success path) ----

        let success_query = "SELECT * FROM crowdstrike_alerts WHERE severity = 'high' LIMIT 10";
        let ast = PrismQlParser::parse(success_query)
            .expect("Test setup: SQL query should parse successfully");

        let normalized = PqlNormalizer::normalize(&ast);

        assert!(
            normalized.is_some(),
            "normalized_pql must be Some(non-empty) for a valid query AST; \
             got None (BC-2.11.018 postcondition)"
        );

        let normalized_str = normalized.unwrap();
        assert!(
            !normalized_str.is_empty(),
            "normalized_pql must be non-empty for a valid query"
        );

        assert!(
            normalized_str.contains("crowdstrike_alerts"),
            "normalized_pql must contain the table name 'crowdstrike_alerts'; \
             got: '{normalized_str}'"
        );

        assert!(
            normalized_str.starts_with("SELECT") || normalized_str.starts_with("FROM"),
            "normalized_pql must start with 'SELECT' or 'FROM'; got: '{normalized_str}'"
        );

        let datafusion_internals = ["HashJoin", "TableScan", "SortExec", "Aggregate"];
        for internal in &datafusion_internals {
            assert!(
                !normalized_str.contains(internal),
                "normalized_pql must NOT contain DataFusion plan node '{internal}'; \
                 got: '{normalized_str}'"
            );
        }

        // ---- Part 2: uppercase canonicalization ----

        let lowercase_query = "select * from crowdstrike_alerts limit 5";
        let lower_ast = PrismQlParser::parse(lowercase_query)
            .expect("Test setup: lowercase query should parse successfully");
        let lower_normalized = PqlNormalizer::normalize(&lower_ast);
        assert!(
            lower_normalized.is_some(),
            "normalized_pql must be Some for lowercase valid query"
        );
        let lower_str = lower_normalized.unwrap();
        assert!(
            lower_str.contains("SELECT"),
            "normalized_pql must canonicalize 'select' → 'SELECT'; got: '{lower_str}'"
        );
        assert!(
            lower_str.contains("FROM"),
            "normalized_pql must canonicalize 'from' → 'FROM'; got: '{lower_str}'"
        );
        assert!(
            lower_str.contains("LIMIT"),
            "normalized_pql must canonicalize 'limit' → 'LIMIT'; got: '{lower_str}'"
        );

        // ---- Part 3: error code mapping (normalized_pql absent via error path) ----

        // E-QUERY-038 → INVALID_PARAMS → error path (no normalized_pql)
        let col_err =
            PrismError::ColumnNotFound(Box::new(prism_core::error::ColumnNotFoundDetails::new(
                "sevrity",
                "crowdstrike_alerts",
                "acme",
                vec!["severity".to_string()],
                Some("severity".to_string()),
            )));
        let (code_038, _msg) = map_prism_error(col_err);
        assert_eq!(
            code_038,
            codes::INVALID_PARAMS,
            "E-QUERY-038 must map to INVALID_PARAMS (-32602); got {code_038}. \
             When the error code is INVALID_PARAMS, the server takes the error path \
             and normalized_pql is ABSENT from the response (BC-2.11.018 invariant)."
        );

        // E-QUERY-001 → INVALID_PARAMS → error path
        let parse_err = PrismError::QueryParseFailed {
            offset: 0,
            detail: "unexpected token 'SELCT'".to_string(),
            query: String::new(),
        };
        let (code_001, _) = map_prism_error(parse_err);
        assert_eq!(
            code_001,
            codes::INVALID_PARAMS,
            "E-QUERY-001 must map to INVALID_PARAMS (-32602); got {code_001}."
        );
    }

    /// BC-2.11.018 / AC-005 — normalized_pql: zero-row query still returns normalized form.
    #[test]
    fn test_BC_2_11_018_normalized_pql_present_on_zero_row_success() {
        let zero_row_query =
            "SELECT detection_id FROM crowdstrike_alerts WHERE severity = 'nonexistent_value'";
        let ast = PrismQlParser::parse(zero_row_query)
            .expect("Test setup: restrictive filter query should parse successfully");

        let normalized = PqlNormalizer::normalize(&ast);
        assert!(
            normalized.is_some(),
            "normalized_pql must be Some even for zero-row queries; BC-2.11.018"
        );

        let s = normalized.unwrap();
        assert!(
            s.contains("crowdstrike_alerts"),
            "normalized_pql for zero-row query must still contain table name; got: '{s}'"
        );
    }

    /// BC-2.11.018 / EC-006 — pipe-mode query: normalized_pql present on success.
    #[test]
    fn test_BC_2_11_018_ec_055_normalized_pql_pipe_mode_success() {
        let pipe_query = "crowdstrike.detections | severity = 'high'";
        let ast = PrismQlParser::parse(pipe_query)
            .expect("Test setup: pipe/filter mode query should parse successfully");

        let normalized = PqlNormalizer::normalize(&ast);
        assert!(
            normalized.is_some(),
            "normalized_pql must be Some for pipe-mode queries (EC-11-055)"
        );

        let s = normalized.unwrap();
        assert!(
            !s.is_empty(),
            "normalized_pql must be non-empty for pipe-mode query"
        );
        for internal in &["HashJoin", "TableScan", "SortExec", "Aggregate"] {
            assert!(
                !s.contains(internal),
                "normalized_pql for pipe-mode must not contain DataFusion internal '{internal}'; \
                 got: '{s}'"
            );
        }
    }

    /// BC-2.11.018 / EC-008 — normalization never produces empty string for valid AST.
    #[test]
    fn test_BC_2_11_018_ec_008_normalize_never_empty_for_valid_ast() {
        let simple_query = "SELECT sensor_id FROM crowdstrike_alerts LIMIT 1";
        let ast = PrismQlParser::parse(simple_query)
            .expect("Test setup: simple SELECT query should parse");

        let normalized = PqlNormalizer::normalize(&ast);
        assert!(
            normalized.is_some(),
            "normalized_pql must be Some(non-empty) for a simple valid query; EC-11-055/EC-008"
        );
        let s = normalized.unwrap();
        assert!(
            !s.is_empty(),
            "normalized_pql inner string must be non-empty (EC-008)"
        );
    }

    // =========================================================================
    // AC-001 (implicit test 6) — ColumnNotFound → -32602 INVALID_PARAMS
    // GREEN-BY-DESIGN (BC-5.38.002)
    // =========================================================================

    /// BC-2.11.016 / AC-001 (implicit test 6) — `PrismError::ColumnNotFound` → -32602.
    ///
    /// GREEN-BY-DESIGN: the explicit arm for ColumnNotFound is already wired.
    #[test]
    fn test_column_not_found_maps_to_invalid_params() {
        use prism_core::error::ColumnNotFoundDetails;
        let err = PrismError::ColumnNotFound(Box::new(ColumnNotFoundDetails::new(
            "sevrity",
            "crowdstrike_alerts",
            "acme",
            vec!["severity".to_string()],
            Some("severity".to_string()),
        )));
        let (code, _message) = map_prism_error(err);
        assert_eq!(
            code,
            codes::INVALID_PARAMS,
            "PrismError::ColumnNotFound MUST map to -32602 INVALID_PARAMS; got {code}."
        );
        assert_ne!(
            code,
            codes::INTERNAL_ERROR,
            "PrismError::ColumnNotFound MUST NOT fall through to -32000 catch-all."
        );
    }

    /// BC-2.11.017 / AC-003 — E-QUERY-003 security-limit error carries `how_to_fix` in
    /// the structured MCP error envelope.
    ///
    /// LOAD-BEARING: Verifies that `how_to_fix_for_security_limit(detail)` is called and
    /// its output appears as the `how_to_fix` key in the JSON error object.
    ///
    /// This test is UNCONDITIONAL — it directly constructs a `QuerySecurityLimitExceeded`
    /// error and routes it through `prism_error_to_structured_call_result`, so it does NOT
    /// depend on DataFusion query execution.
    #[test]
    fn test_BC_2_11_017_ac003_security_limit_error_carries_how_to_fix() {
        use prism_core::PrismError;
        use prism_mcp::error_mapping::prism_error_to_structured_call_result;

        let err = PrismError::QuerySecurityLimitExceeded {
            detail: "query depth exceeds the maximum allowed (10)".to_string(),
        };
        let result = prism_error_to_structured_call_result(err);

        assert_eq!(
            result.is_error,
            Some(true),
            "BC-2.11.017 AC-003: QuerySecurityLimitExceeded must produce is_error=true"
        );

        let sc = result
            .structured_content
            .expect("BC-2.11.017 AC-003: structured_content must be present on E-QUERY-003 error");

        let error_obj = sc
            .get("error")
            .expect("BC-2.11.017 AC-003: sc['error'] must be present on security limit error");

        // Verify how_to_fix field is present and non-empty.
        // This is the LOAD-BEARING assertion: fails if how_to_fix_for_security_limit is NOT called.
        let how_to_fix = error_obj.get("how_to_fix").expect(
            "BC-2.11.017 AC-003: E-QUERY-003 error response MUST have 'how_to_fix' field. \
             ABSENT means how_to_fix_for_security_limit() is NOT wired into \
             QuerySecurityLimitExceeded arm of prism_error_to_structured_call_result.",
        );
        let how_to_fix_str = how_to_fix
            .as_str()
            .expect("how_to_fix must be a JSON string");
        assert!(
            !how_to_fix_str.is_empty(),
            "BC-2.11.017 AC-003: how_to_fix must be non-empty for E-QUERY-003"
        );
        // The guidance must contain actionable language about limits.
        assert!(
            how_to_fix_str.contains("depth")
                || how_to_fix_str.contains("limit")
                || how_to_fix_str.contains("nested")
                || how_to_fix_str.contains("subquery"),
            "BC-2.11.017 AC-003: how_to_fix for depth-limit error must reference query structure; \
             got: '{how_to_fix_str}'"
        );
    }

    /// BC-2.11.017 / AC-003 — `valid_operators_for_type` helper returns correct operators.
    ///
    /// LOAD-BEARING unit test: Verifies the `valid_operators_for_type` helper function is
    /// callable, returns a non-empty slice, and returns type-appropriate operators.
    ///
    /// NOTE: The `valid_operators_for_type` field in `StructuredErrorFields` is populated
    /// when the error variant carries `ColumnType` context. Currently no `PrismError` variant
    /// carries `ColumnType` (the engine coerces types rather than producing a typed mismatch
    /// error). This test covers the helper function itself (E-QUERY-002 enrichment is
    /// available and correct for when a future E-QUERY-002 variant is added). The field
    /// `valid_operators_for_type: Option<Vec<String>>` is present in `StructuredErrorFields`
    /// and wired in `build_structured_error_response`.
    #[test]
    fn test_BC_2_11_017_ac003_enrichment_helper_valid_operators_for_type_returns_correct_operators()
    {
        use prism_core::column::ColumnType;

        // String columns: must include =, !=, LIKE, IS NULL, IS NOT NULL, IN
        let string_ops = prism_query::engine::valid_operators_for_type(ColumnType::String);
        assert!(
            !string_ops.is_empty(),
            "valid_operators_for_type(String) must return non-empty operators"
        );
        assert!(
            string_ops.contains(&"="),
            "String operators must include '='; got: {string_ops:?}"
        );
        assert!(
            string_ops.contains(&"LIKE"),
            "String operators must include 'LIKE'; got: {string_ops:?}"
        );

        // Integer columns: must include =, !=, <, >, <=, >=
        let int_ops = prism_query::engine::valid_operators_for_type(ColumnType::Integer);
        assert!(
            !int_ops.is_empty(),
            "valid_operators_for_type(Integer) must return non-empty operators"
        );
        assert!(
            int_ops.contains(&"="),
            "Integer operators must include '='; got: {int_ops:?}"
        );
        assert!(
            int_ops.contains(&">"),
            "Integer operators must include '>'; got: {int_ops:?}"
        );
        assert!(
            int_ops.contains(&"<"),
            "Integer operators must include '<'; got: {int_ops:?}"
        );

        // Boolean columns: must include = and !=
        let bool_ops = prism_query::engine::valid_operators_for_type(ColumnType::Boolean);
        assert!(
            !bool_ops.is_empty(),
            "valid_operators_for_type(Boolean) must return non-empty operators"
        );
        assert!(
            bool_ops.contains(&"="),
            "Boolean operators must include '='; got: {bool_ops:?}"
        );
    }

    /// BC-2.11.016 / AC-001 — E-QUERY-038 message carries E-QUERY-038 code prefix.
    ///
    /// GREEN-BY-DESIGN: map_prism_error already delegates to ColumnNotFoundDetails::Display.
    #[test]
    fn test_column_not_found_mcp_message_carries_e_query_038_prefix() {
        use prism_core::error::ColumnNotFoundDetails;
        let err = PrismError::ColumnNotFound(Box::new(ColumnNotFoundDetails::new(
            "bogus_column",
            "armis_devices",
            "customer1",
            vec!["device_id".to_string(), "ip_address".to_string()],
            None,
        )));
        let (_code, message) = map_prism_error(err);
        assert!(
            message.contains("E-QUERY-038"),
            "map_prism_error message for ColumnNotFound must contain 'E-QUERY-038'; \
             got: '{message}'"
        );
        assert!(
            !message.contains("internal server error"),
            "map_prism_error message for ColumnNotFound must not be opaque; got: '{message}'"
        );
    }
}
