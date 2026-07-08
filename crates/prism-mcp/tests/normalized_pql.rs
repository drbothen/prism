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
//! | `test_BC_2_11_017_ac003_type_error_response_carries_valid_operators` | AC-003 | Engine has no plan-time type-mismatch gate → String+`>` query SUCCEEDS instead of E-QUERY-002; even if it errored, operators would be the type-agnostic superset, not the String-specific set |
//! | `test_BC_2_11_017_ec11046_near_text_present_as_empty_string_at_end_of_input` | AC-003 | F-PRL-MED-001: empty `near_text` mapped to `None` → key ABSENT; must be `Some("")` |
//! | `test_BC_2_11_018_ec11054_normalized_pql_present_on_partial_failure` | AC-005 | OBS-2: regression gate for partial-failure path (EC-11-054) |
//!
//! # BC references
//! - BC-2.11.016 — E-QUERY-038 Column-Not-Found Plan-Time Gate
//! - BC-2.11.017 — E-QUERY Pedagogical Enrichments
//! - BC-2.11.018 — normalized_pql Field on Successful Query Responses

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, sync::Arc};

    use arrow::record_batch::RecordBatch;
    use async_trait::async_trait;
    use prism_core::{OrgId, OrgSlug, PrismError, SensorId};
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
    use prism_sensors::{
        AdapterRegistry, CredentialResolver, QueryParams as SensorQueryParams, SensorAdapter,
        SensorAuth, SensorError, SensorSpec as SensorAdapterSpec,
    };
    use prism_spec_engine::{
        overlay::{OverlayLoader, ResolvedSensorSpec, ResolvedSpecKey, SensorInstanceOverlay},
        spec_parser::{AuthType, ColumnSpec, SensorSpec, TableSpec},
    };
    use rmcp::handler::server::wrapper::Parameters;

    // =========================================================================
    // Stub types for partial-failure injection (EC-11-054 test seam)
    // =========================================================================

    /// Sensor adapter that always fails with HTTP 503.
    ///
    /// Injected into the AdapterRegistry for the partial-failure test so that
    /// fan_out() can reach a real adapter boundary and return sensor_errors.
    /// (SID-1: unit test at the dependency boundary; no live DTU required.)
    struct AlwaysFailsAdapter {
        sensor_id: SensorId,
    }

    #[async_trait]
    impl SensorAdapter for AlwaysFailsAdapter {
        fn sensor_type(&self) -> SensorId {
            self.sensor_id.clone()
        }

        fn sensor_name(&self) -> &'static str {
            "always-fails-stub"
        }

        async fn fetch(
            &self,
            _spec: &SensorAdapterSpec,
            _params: &SensorQueryParams,
            _auth: &dyn SensorAuth,
        ) -> Result<Vec<RecordBatch>, SensorError> {
            Err(SensorError::HttpError {
                sensor: self.sensor_id.to_string(),
                status: 503,
                body: "stub: simulated partial failure for EC-11-054 test".into(),
            })
        }
    }

    /// Stub auth token for the partial-failure test.
    struct StubAuth;

    impl SensorAuth for StubAuth {
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }

        fn auth_type_name(&self) -> &'static str {
            "custom_via_plugin"
        }
    }

    /// Credential resolver that always succeeds (returns StubAuth).
    ///
    /// Required so fan_out() reaches the adapter boundary rather than
    /// short-circuiting with a CredentialNotFound error. The stub auth
    /// is ignored by AlwaysFailsAdapter::fetch.
    struct AlwaysSucceedsCreds;

    impl CredentialResolver for AlwaysSucceedsCreds {
        fn resolve(
            &self,
            _client_id: &str,
            _sensor_id: SensorId,
        ) -> Result<Box<dyn SensorAuth>, SensorError> {
            Ok(Box::new(StubAuth))
        }
    }

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

    /// Build a `PrismServer` wired with a FAILING adapter for the given sensor/table/org.
    ///
    /// The `AdapterRegistry` is NON-EMPTY (contains an `AlwaysFailsAdapter` for
    /// `sensor_id`), so `resolve_source_refs` creates fan-out targets that reach
    /// the adapter boundary. When `fan_out()` calls `AlwaysFailsAdapter::fetch`,
    /// it returns HTTP 503, which the materialization pipeline converts to a
    /// `sensor_errors` entry. The query still returns `Ok(QueryResult)` — this is
    /// the "all-targets-failed" partial-failure path (materialization.rs Err(e) branch,
    /// line ~763) that pushes to `sensor_errors` and continues rather than erroring.
    ///
    /// `AlwaysSucceedsCreds` is wired so credentials don't short-circuit before the
    /// adapter boundary (SID-1: unit test without live DTU).
    ///
    /// Used ONLY by `test_BC_2_11_018_ec11054_normalized_pql_present_on_partial_failure`.
    fn make_server_with_failing_adapter(
        sensor_id_str: &str,
        table_name: &str,
        columns: Vec<ColumnSpec>,
        org: &str,
    ) -> (PrismServer, OrgId) {
        // Fixed deterministic OrgId (UUID v7 prefix bytes, same pattern as bc_2_01_010.rs).
        let org_id = OrgId::from_uuid(uuid::Uuid::from_bytes([
            0x01, 0x9f, 0x3a, 0x71, 0x5c, 0x6d, 0x7a, 0x8b, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0xEC, // 0xEC = "EC" for EC-11-054 sentinel
        ]));

        // Build the TableRegistry + ResolvedSensorSpec so the plan-time availability
        // gate (E-QUERY-037) passes and the query reaches the fan-out phase.
        let (key, resolved) = make_resolved(sensor_id_str, table_name, columns, org);
        let mut resolved_map = HashMap::new();
        resolved_map.insert(key, resolved.clone());

        let registry = Arc::new(TableRegistry::new());
        registry
            .register_sensor(&resolved.spec)
            .expect("register_sensor must not fail in fixture");

        // Non-empty AdapterRegistry: registers the failing adapter.
        // resolve_source_refs checks `!adapter_registry.is_empty()` before
        // is_sensor_registered — so a non-empty registry with the sensor registered
        // produces fan-out targets that reach AlwaysFailsAdapter::fetch.
        let sensor_id_typed = SensorId::new(sensor_id_str);
        let failing_adapter: Arc<dyn SensorAdapter> = Arc::new(AlwaysFailsAdapter {
            sensor_id: sensor_id_typed,
        });
        let mut adapter_registry = AdapterRegistry::new();
        adapter_registry.register(org_id, failing_adapter);

        let mut engine = QueryEngine::new_with_cache_config(
            Arc::new(adapter_registry),
            Arc::new(InMemoryCredentialStore::new()),
            Arc::new(prism_ocsf::OcsfNormalizer::new()),
            Arc::new(ClientRegistry::new(vec![])),
            QueryEngineConfig::default(),
            prism_query::cache::CacheConfig::default(),
        );
        // Wire AlwaysSucceedsCreds so fan_out() reaches the adapter (not blocked by creds).
        engine = engine.with_credential_resolver(Arc::new(AlwaysSucceedsCreds));
        // Wire resolved_spec_map and table_registry.
        engine.resolved_spec_map = Some(Arc::new(arc_swap::ArcSwap::new(Arc::new(resolved_map))));
        engine = engine.with_table_registry(registry);

        let server = PrismServer::new().with_query_engine(Arc::new(engine));
        (server, org_id)
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

    /// BC-2.11.017 / AC-003 + F-198-FRESH-MED-001 — E-QUERY-001 parse error structured response
    /// carries `code: "E-QUERY-001"` (NOT `"E-MCP-002"`).
    ///
    /// # LOAD-BEARING Red Gate
    ///
    /// FAILS on current HEAD because the `QueryParseFailed` arm in
    /// `prism_error_to_structured_call_result` has `ec_code_override: None`.
    ///
    /// Code derivation (current HEAD):
    ///   - `map_prism_error(QueryParseFailed)` → message `"PrismQL parse error: {detail}"` (no E- prefix)
    ///   - `message.starts_with("E-")` → false
    ///   - Falls to `match code_i32 { INVALID_PARAMS => "E-MCP-002", ... }`
    ///   - Result: `code: "E-MCP-002"` ← WRONG (semantically "tool not available" / permission error)
    ///
    /// BC-2.11.017 §E-QUERY-001 + AC-003 mandate `code: "E-QUERY-001"` for parse errors.
    /// The LLM self-correction loop classifies failures by `code` — wrong code breaks it.
    ///
    /// FIX: set `ec_code_override: Some("E-QUERY-001")` on the `QueryParseFailed` arm
    /// in `prism_error_to_structured_call_result` (same mechanism as `E-AUTH-010`, `E-SENSOR-020`).
    ///
    /// RED→GREEN proof: this test FAILS (code=="E-MCP-002") before the fix,
    /// and PASSES (code=="E-QUERY-001") after setting ec_code_override.
    #[tokio::test]
    async fn test_BC_2_11_017_ac003_parse_error_structured_code_is_e_query_001() {
        let server = make_server_minimal();

        // "SELCT * FROM crowdstrike_alerts" — parse error → must produce code "E-QUERY-001"
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
            "F-198-FRESH-MED-001: parse error must return is_error=true"
        );

        let sc = call_result
            .structured_content
            .expect("F-198-FRESH-MED-001: structured_content must be present");

        let error_obj = sc
            .get("error")
            .expect("F-198-FRESH-MED-001: sc['error'] must be present");

        // LOAD-BEARING: code MUST be "E-QUERY-001", NOT "E-MCP-002".
        //
        // FAILS NOW because ec_code_override is None on the QueryParseFailed arm —
        // the code derivation falls to `match INVALID_PARAMS => "E-MCP-002"`.
        //
        // PASSES AFTER FIX: setting ec_code_override: Some("E-QUERY-001") pins the code
        // directly, bypassing the message-string-based fallback.
        //
        // "E-MCP-002" is semantically "tool not available" — completely wrong for a parse error.
        // The LLM self-correction loop classifies failures by `code`; wrong code breaks it.
        let code = error_obj
            .get("code")
            .expect("F-198-FRESH-MED-001: 'code' field must be present in error object")
            .as_str()
            .expect("F-198-FRESH-MED-001: 'code' must be a string");

        assert_eq!(
            code, "E-QUERY-001",
            "F-198-FRESH-MED-001 (BC-2.11.017 AC-003): parse error structured response \
             MUST carry code='E-QUERY-001'. \
             Got: '{}'. \
             Current bug: ec_code_override is None on QueryParseFailed arm — falls to \
             INVALID_PARAMS => 'E-MCP-002' (semantically wrong: 'tool not available'). \
             FIX: set ec_code_override: Some(\"E-QUERY-001\") on the QueryParseFailed arm \
             in prism_error_to_structured_call_result.",
            code
        );

        // REGRESSION GUARD: near_text and reference_pointer must still be present after the fix.
        assert!(
            error_obj.get("near_text").is_some(),
            "F-198-FRESH-MED-001: near_text must still be present after ec_code_override fix"
        );
        assert_eq!(
            error_obj.get("reference_pointer").and_then(|v| v.as_str()),
            Some("prismql://reference"),
            "F-198-FRESH-MED-001: reference_pointer must still be 'prismql://reference' after fix"
        );
    }

    /// BC-2.11.017 / AC-003 — E-QUERY-002 type-mismatch error for a String column carries
    /// `valid_operators_for_type` equal to the STRING-SPECIFIC operator set.
    ///
    /// BC-2.11.017 canonical test vector:
    ///   Query: `SELECT * FROM <table> WHERE <string_col> > 5`
    ///   Expected: E-QUERY-002 with `valid_operators_for_type: ["=","!=","LIKE","IN","NOT IN","IEQ","IIN","INE"]`
    ///   (STRING-SPECIFIC set — must NOT contain "<", ">", "<=", ">=", "BETWEEN")
    ///   BC-2.11.024 v1.3: IEQ/IIN/INE added as valid string operators (F-P24-MED-001).
    ///
    /// LOAD-BEARING RED GATE test (TD-VSDD-059 paper-fix detection):
    ///
    ///   The prior implementation satisfied the old test by hardcoding the GENERIC operator
    ///   superset `["=","!=","<",">","<=",">=","LIKE","IN","NOT IN","BETWEEN"]` on the
    ///   `QueryPlanFailed` arm regardless of actual column type. The old test only asserted
    ///   "non-empty array of strings" — which the superset satisfies. This is the
    ///   paper-fix: the assertion didn't depend on production behavior at all.
    ///
    ///   This test is ENGINE-DRIVEN and TYPE-SPECIFIC:
    ///   1. Constructs a `QueryEngine` with a `resolved_spec_map` containing a table whose
    ///      column is `ColumnType::String` — this is the ACTUAL column type in the schema.
    ///   2. Drives `SELECT * FROM crowdstrike_alerts WHERE severity > 5` through the REAL
    ///      plan-time path (`PrismServer::query`) — NOT a synthetic QueryPlanFailed.
    ///   3. Asserts `is_error == Some(true)` — the engine MUST reject this query.
    ///   4. Asserts `valid_operators_for_type` is EXACTLY the String-specific set
    ///      `["=","!=","LIKE","IN","NOT IN","IEQ","IIN","INE"]` as returned by
    ///      `prism_query::engine::valid_operators_for_type(ColumnType::String)`.
    ///      (BC-2.11.024 v1.3: IEQ/IIN/INE added as valid string operators — F-P24-MED-001.)
    ///      The assertion FAILS if the array contains ">" or "<" or "BETWEEN" — i.e., the
    ///      generic superset. Only the type-specific subset is acceptable.
    ///   5. A second case (Boolean column + `>`) asserts the Boolean-specific set `["=","!="]`
    ///      to prove operators are DERIVED FROM THE COLUMN TYPE, not hardcoded.
    ///
    /// FAILS ON CURRENT HEAD because:
    ///   (a) The engine has no plan-time type-mismatch detection — `severity > 5` on a String
    ///       column either succeeds (DataFusion coerces) or produces a generic execution error,
    ///       NOT E-QUERY-002. The `is_error == Some(true)` assertion will fail (query succeeds).
    ///   (b) Even if an error fires, the `QueryPlanFailed` arm in error_mapping.rs hardcodes
    ///       the type-agnostic superset, not the String-specific set — the EXACT-SET assertion
    ///       would fail because ">" is in the array.
    ///
    /// The implementer must:
    ///   1. Add plan-time type-mismatch detection in `check_query_column_availability` or a new
    ///      `check_operator_type_compatibility` gate: after verifying a column EXISTS, check that
    ///      the operator used against it is valid for its `ColumnType`. Return a new error
    ///      variant (e.g., `PrismError::QueryTypeMismatch { column, actual_type, operator }`)
    ///      that carries the `ColumnType` so the error-mapping arm can call
    ///      `valid_operators_for_type(actual_type)` to get the TYPE-SPECIFIC operator set.
    ///   2. Add the error-mapping arm in `prism_error_to_structured_call_result` that calls
    ///      `valid_operators_for_type(actual_type)` from the variant, not from a hardcoded list.
    ///
    /// This test also replaces the prior test
    /// `test_BC_2_11_017_ac003_type_error_response_carries_valid_operators` which was
    /// a paper-fix (synthetic error, no engine path, no type-specific assertion).
    #[tokio::test]
    async fn test_BC_2_11_017_ac003_type_error_response_carries_valid_operators() {
        use prism_core::column::ColumnType;
        use prism_query::engine::valid_operators_for_type;

        // ---- Case 1: String column + ordering operator ----
        //
        // BC-2.11.017 canonical test vector:
        //   severity is a STRING column; the query uses `>` (an ordering operator valid only
        //   for Integer/Float/Datetime). The engine MUST reject this at plan time with E-QUERY-002
        //   and populate `valid_operators_for_type` with the STRING-specific set.
        //
        // LOAD-BEARING: this drives the real plan-time path, NOT a synthetic QueryPlanFailed.
        let columns_string = vec![
            ColumnSpec::new("severity", ColumnType::String, None, vec![]),
            ColumnSpec::new("detection_id", ColumnType::String, None, vec![]),
        ];
        let (key_s, resolved_s) = make_resolved("crowdstrike", "alerts", columns_string, "acme");
        let mut map_s = HashMap::new();
        map_s.insert(key_s, resolved_s.clone());
        let server_string = make_server_with_engine(map_s, vec![resolved_s.spec.clone()]);

        // String column `severity` with ordering operator `>` — BC-2.11.017 canonical vector.
        // On current HEAD: the engine has no type-mismatch gate so DataFusion coerces or the
        // query succeeds → is_error will be None/false → this assertion FAILS (RED gate).
        let params_string: QueryToolParams = serde_json::from_str(
            r#"{"query": "SELECT * FROM crowdstrike_alerts WHERE severity > 5"}"#,
        )
        .expect("QueryToolParams JSON must deserialize");
        let result_string = server_string
            .query(Parameters(params_string))
            .await
            .expect("domain errors must return Ok(structured_error), not Err");

        assert_eq!(
            result_string.is_error,
            Some(true),
            "BC-2.11.017 AC-003 (String+>): ordering operator '>' on String column 'severity' \
             MUST produce is_error=true (E-QUERY-002 type-mismatch). \
             CURRENT BEHAVIOR: query succeeds (engine has no plan-time type-mismatch gate). \
             FIX: add check_operator_type_compatibility gate — after the column-availability check \
             passes (column exists), verify the operator is valid for the column's ColumnType. \
             Return PrismError::QueryTypeMismatch {{ column, actual_type, operator }} when \
             the operator is not in valid_operators_for_type(actual_type)."
        );

        let sc_string = result_string.structured_content.expect(
            "BC-2.11.017 AC-003 (String+>): structured_content must be present on E-QUERY-002",
        );

        let error_obj_string = sc_string
            .get("error")
            .expect("BC-2.11.017 AC-003 (String+>): sc['error'] must be present");

        // LOAD-BEARING ASSERTION: the operators array must be EXACTLY the String-specific set.
        // valid_operators_for_type(ColumnType::String) returns
        // ["=", "!=", "LIKE", "IN", "NOT IN", "IEQ", "IIN", "INE"].
        // BC-2.11.024 v1.3: IEQ/IIN/INE added by F-P24-MED-001 (S-PRISMQL-CASE-INSENSITIVE-001).
        // The test derives expected from valid_operators_for_type so it tracks changes automatically.
        // This assertion FAILS if:
        //   - the field is absent (no type-mismatch detection)
        //   - the field is the generic superset (">", "<", "BETWEEN" present)
        let operators_val = error_obj_string.get("valid_operators_for_type").expect(
            "BC-2.11.017 AC-003 (String+>): E-QUERY-002 error response MUST have \
             'valid_operators_for_type' field. ABSENT on current HEAD. \
             FIX: add a PrismError::QueryTypeMismatch arm to prism_error_to_structured_call_result \
             that calls valid_operators_for_type(actual_type) to populate this field.",
        );
        let operators_arr = operators_val
            .as_array()
            .expect("BC-2.11.017 AC-003 (String+>): valid_operators_for_type must be a JSON array");

        // Get the canonical String operator set from the production helper.
        // The test derives the expected value from valid_operators_for_type — not from literals —
        // so the assertion tracks the helper automatically if it changes.
        let expected_string_ops: Vec<serde_json::Value> =
            valid_operators_for_type(ColumnType::String)
                .iter()
                .map(|s| serde_json::Value::String(s.to_string()))
                .collect();

        // EXACT SET ASSERTION: the operators array must equal the String-specific set,
        // not the generic superset. This fails if ">" is present.
        assert_eq!(
            operators_arr, &expected_string_ops,
            "BC-2.11.017 AC-003 (String+>): valid_operators_for_type MUST equal the \
             String-specific set {:?}. \
             Current value: {:?}. \
             If '>' or '<' or 'BETWEEN' appear in the array, the implementation is \
             returning the type-agnostic superset instead of the ColumnType::String set. \
             FIX: the error-mapping arm must call \
             valid_operators_for_type(actual_type) from the error variant's ColumnType context, \
             NOT hardcode a superset.",
            expected_string_ops, operators_arr
        );

        // NEGATIVE ASSERTION: ordering operators must NOT appear for String columns.
        let ordering_ops = [">", "<", "<=", ">=", "BETWEEN"];
        for op in &ordering_ops {
            let op_value = serde_json::Value::String(op.to_string());
            assert!(
                !operators_arr.contains(&op_value),
                "BC-2.11.017 AC-003 (String+>): ordering operator '{}' MUST NOT appear \
                 in valid_operators_for_type for ColumnType::String (it's a numeric/datetime \
                 operator). Got: {:?}",
                op,
                operators_arr
            );
        }

        // ---- Case 2: Boolean column + ordering operator ----
        //
        // Proves operators are DERIVED FROM THE COLUMN TYPE, not hardcoded.
        // Boolean columns only support ["=", "!="] — no ordering operators at all.
        let columns_bool = vec![
            ColumnSpec::new("is_active", ColumnType::Boolean, None, vec![]),
            ColumnSpec::new("sensor_name", ColumnType::String, None, vec![]),
        ];
        let (key_b, resolved_b) = make_resolved("armis", "devices", columns_bool, "acme");
        let mut map_b = HashMap::new();
        map_b.insert(key_b, resolved_b.clone());
        let server_bool = make_server_with_engine(map_b, vec![resolved_b.spec.clone()]);

        let params_bool: QueryToolParams =
            serde_json::from_str(r#"{"query": "SELECT * FROM armis_devices WHERE is_active > 1"}"#)
                .expect("QueryToolParams JSON must deserialize");
        let result_bool = server_bool
            .query(Parameters(params_bool))
            .await
            .expect("domain errors must return Ok(structured_error), not Err");

        assert_eq!(
            result_bool.is_error,
            Some(true),
            "BC-2.11.017 AC-003 (Boolean+>): ordering operator '>' on Boolean column 'is_active' \
             MUST produce is_error=true (E-QUERY-002 type-mismatch). \
             CURRENT BEHAVIOR: query succeeds (no plan-time type-mismatch gate). \
             FIX: same gate as Case 1 — check operator against valid_operators_for_type(actual_type)."
        );

        let sc_bool = result_bool.structured_content.expect(
            "BC-2.11.017 AC-003 (Boolean+>): structured_content must be present on E-QUERY-002",
        );

        let error_obj_bool = sc_bool
            .get("error")
            .expect("BC-2.11.017 AC-003 (Boolean+>): sc['error'] must be present");

        let bool_ops_val = error_obj_bool.get("valid_operators_for_type").expect(
            "BC-2.11.017 AC-003 (Boolean+>): valid_operators_for_type must be present for Boolean E-QUERY-002",
        );
        let bool_ops_arr = bool_ops_val.as_array().expect(
            "BC-2.11.017 AC-003 (Boolean+>): valid_operators_for_type must be a JSON array",
        );

        // Canonical Boolean set: only ["=", "!="].
        let expected_bool_ops: Vec<serde_json::Value> =
            valid_operators_for_type(ColumnType::Boolean)
                .iter()
                .map(|s| serde_json::Value::String(s.to_string()))
                .collect();

        assert_eq!(
            bool_ops_arr, &expected_bool_ops,
            "BC-2.11.017 AC-003 (Boolean+>): valid_operators_for_type MUST equal the \
             Boolean-specific set {:?}. Got: {:?}. \
             This case proves operators are DERIVED FROM ColumnType::Boolean (not hardcoded): \
             Boolean only allows '=' and '!='. If String-set or full-superset appears, the \
             implementation is not calling valid_operators_for_type(actual_type) from \
             the variant's ColumnType context.",
            expected_bool_ops, bool_ops_arr
        );
    }

    /// BC-2.11.017 / EC-11-046 — E-QUERY-001 parse error at END-OF-INPUT must carry
    /// `near_text: ""` (present, empty string) — NOT absent.
    ///
    /// BC-2.11.017 EC-11-046: "E-QUERY-001 parse error at end-of-input →
    ///   `near_text: ""` (empty string); `reference_pointer` still present"
    ///
    /// LOAD-BEARING RED GATE test (F-PRL-MED-001, LOCAL adversary pass-2):
    ///   FAILS on current HEAD because error_mapping.rs lines 1022-1026:
    ///     ```rust
    ///     near_text: if near_text.is_empty() {
    ///         None  // BUG: should be Some("") per EC-11-046
    ///     } else {
    ///         Some(near_text)
    ///     },
    ///     ```
    ///   When `extract_near_text` returns `""` (offset ≥ input.len()), the empty string
    ///   is converted to `None`. `build_structured_error_response` then omits the key
    ///   entirely from JSON (guarded by `if let Some(nt)`). So the `near_text` key is
    ///   ABSENT for end-of-input errors.
    ///
    /// BC-2.11.017 EC-11-046 requires `near_text` to be PRESENT with value `""`.
    /// The key must exist in the JSON error object; its value must be the empty string.
    ///
    /// FIX in error_mapping.rs: change the guard from:
    ///   `near_text: if near_text.is_empty() { None } else { Some(near_text) }`
    /// to:
    ///   `near_text: Some(near_text)`
    /// This preserves `near_text: ""` for end-of-input and `near_text: "token"` for mid-input.
    #[test]
    fn test_BC_2_11_017_ec11046_near_text_present_as_empty_string_at_end_of_input() {
        use prism_mcp::error_mapping::prism_error_to_structured_call_result;

        // End-of-input path: the effective_offset computation in error_mapping.rs has an
        // `if *offset == 0` branch that directly calls extract_near_text(query, 0).
        // When query is empty "", extract_near_text("", 0) returns "" because 0 >= 0.
        // The bug: `if near_text.is_empty() { None }` then sets near_text to None.
        // EC-11-046 requires near_text: "" (present, empty string) at end-of-input.
        //
        // Using offset=0, query="" triggers: effective_offset=0 → extract_near_text("", 0) = ""
        // → near_text.is_empty() is true → BUG: set to None (should be Some("")).
        let query = String::new(); // empty query → end-of-input at offset 0
        let err = prism_core::PrismError::QueryParseFailed {
            offset: 0, // at start/end of empty query → extract_near_text returns ""
            detail: "unexpected end of input; expected query".to_string(),
            query: query.clone(),
        };
        let result = prism_error_to_structured_call_result(err);

        assert_eq!(
            result.is_error,
            Some(true),
            "BC-2.11.017 EC-11-046: QueryParseFailed at end-of-input must produce is_error=true"
        );

        let sc = result
            .structured_content
            .expect("BC-2.11.017 EC-11-046: structured_content must be present on parse error");

        let error_obj = sc
            .get("error")
            .expect("BC-2.11.017 EC-11-046: sc['error'] must be present on parse error");

        // LOAD-BEARING: near_text must be PRESENT (key exists) with value "" (empty string).
        // FAILS NOW because error_mapping.rs converts empty string to None → key ABSENT.
        //
        // FIX: change `if near_text.is_empty() { None } else { Some(near_text) }`
        //      to `Some(near_text)` in the QueryParseFailed arm of
        //      prism_error_to_structured_call_result.
        let near_text_value = error_obj.get("near_text").expect(
            "BC-2.11.017 EC-11-046: E-QUERY-001 parse error at end-of-input MUST have \
             'near_text' key present in the error object. \
             Current behavior: key is ABSENT when extract_near_text returns empty string. \
             FIX: change the `near_text: if near_text.is_empty() { None } else { Some(near_text) }` \
             guard in error_mapping.rs QueryParseFailed arm to `near_text: Some(near_text)`. \
             EC-11-046 requires the key to be present with value \"\" at end-of-input."
        );

        // The value must be the empty string (not null, not missing).
        assert_eq!(
            near_text_value.as_str(),
            Some(""),
            "BC-2.11.017 EC-11-046: near_text at end-of-input must be empty string \"\"; \
             got: {:?}",
            near_text_value
        );

        // reference_pointer must still be present at end-of-input (EC-11-046: "still present").
        let ref_ptr = error_obj.get("reference_pointer").expect(
            "BC-2.11.017 EC-11-046: reference_pointer must be present even at end-of-input \
             ('still present' per EC-11-046)",
        );
        assert_eq!(
            ref_ptr.as_str(),
            Some("prismql://reference"),
            "BC-2.11.017 EC-11-046: reference_pointer must be 'prismql://reference'; \
             got: {:?}",
            ref_ptr
        );
    }

    /// BC-2.11.018 / EC-11-054 — `normalized_pql` is PRESENT on partial-failure response
    /// (query-level success, some sensors errored, non-empty `sensor_errors` list).
    ///
    /// BC-2.11.018 EC-11-054: "Query produces partial results (some sensors errored,
    ///   some succeeded) → `normalized_pql` is PRESENT"
    ///
    /// # LOAD-BEARING (TD-VSDD-059)
    ///
    /// This test drives a REAL partial-failure path:
    ///   1. `AlwaysFailsAdapter` is registered for the `crowdstrike` sensor (non-empty
    ///      `AdapterRegistry`). `resolve_source_refs` creates a fan-out target because
    ///      the registry is non-empty AND `is_sensor_registered` returns true.
    ///   2. `AlwaysSucceedsCreds` resolves credentials so fan_out() reaches
    ///      `AlwaysFailsAdapter::fetch`, which returns HTTP 503.
    ///   3. `fan_out()` returns `Err(AllTargetsFailed)` (1 target, 1 failure).
    ///   4. The materialization pipeline's `Err(e)` branch pushes to `sensor_errors`
    ///      and continues — the overall pipeline returns `Ok(MaterializationOutput)`
    ///      with `sensor_errors` non-empty.
    ///   5. `QueryEngine::execute()` returns `Ok(QueryResult)` with
    ///      `sensor_errors` non-empty and `batches` empty (no successful fetches).
    ///   6. `PrismServer::query` computes `normalized_pql` UNCONDITIONALLY on the
    ///      success path — it is NOT gated on `sensor_errors.is_empty()`.
    ///
    /// DELETION PROOF: removing the `normalized_pql` insert in server.rs WILL break
    /// this test because the assertion checks `results.get("normalized_pql")` directly.
    ///
    /// ZERO-ROW PATH PROOF: the test also asserts `sensor_errors` is non-empty in the
    /// response, which is IMPOSSIBLE via the empty-registry zero-row path (empty
    /// AdapterRegistry → no fan-out → sensor_errors = []). The assertion on
    /// `sensor_errors` length distinguishes the partial-failure path from the
    /// zero-row proxy.
    ///
    /// NOTE: because all targets fail (1 failing adapter), the query returns zero rows —
    /// but the `sensor_errors` array in the response is non-empty, proving the
    /// partial-failure branch was exercised rather than the no-adapter zero-row path.
    #[tokio::test]
    async fn test_BC_2_11_018_ec11054_normalized_pql_present_on_partial_failure() {
        use prism_core::column::ColumnType;

        // Wire a table spec and a FAILING adapter so the query reaches fan_out()
        // and produces a real sensor error. The TableRegistry registration ensures
        // the plan-time availability gate (E-QUERY-037) passes so the query reaches
        // the materialization phase.
        let columns = vec![
            ColumnSpec::new("severity", ColumnType::String, None, vec![]),
            ColumnSpec::new("detection_id", ColumnType::String, None, vec![]),
        ];
        let (server, _org_id) =
            make_server_with_failing_adapter("crowdstrike", "alerts", columns, "acme");

        // Valid query against the registered table. The AdapterRegistry is non-empty
        // (AlwaysFailsAdapter registered), so resolve_source_refs creates a fan-out target.
        // AlwaysFailsAdapter::fetch returns HTTP 503 → AllTargetsFailed → sensor_errors
        // non-empty. The query still returns Ok(QueryResult) with sensor_errors non-empty.
        let params: QueryToolParams =
            serde_json::from_str(r#"{"query": "SELECT severity FROM crowdstrike_alerts LIMIT 5"}"#)
                .expect("QueryToolParams JSON must deserialize");
        let call_result = server
            .query(Parameters(params))
            .await
            .expect("query must return Ok(QueryResult) even when sensors fail (partial-failure)");

        // ASSERTION 1: NOT a query-level error. Sensor failures produce sensor_errors,
        // not is_error=true. The query succeeds at the query-engine level.
        assert_ne!(
            call_result.is_error,
            Some(true),
            "BC-2.11.018 EC-11-054: partial-failure query MUST NOT return is_error=true. \
             Sensor errors go to sensor_errors, not query-level error. \
             structured_content: {:?}",
            call_result.structured_content
        );

        let sc = call_result.structured_content.expect(
            "BC-2.11.018 EC-11-054: structured_content must be present on partial-failure success",
        );

        let results = sc.get("results").expect(
            "BC-2.11.018 EC-11-054: sc['results'] must be present on partial-failure success",
        );

        // ASSERTION 2 (LOAD-BEARING — normalized_pql wire):
        // normalized_pql MUST be present even when sensor_errors is non-empty.
        // Removing the normalized_pql insert from server.rs breaks this assertion.
        let normalized_pql = results.get("normalized_pql").expect(
            "BC-2.11.018 EC-11-054 (OBS-2 fix): normalized_pql MUST be present on \
             partial-failure success (query-level OK, AlwaysFailsAdapter returned 503). \
             FIX: ensure the normalized_pql wire in server.rs is NOT gated behind \
             `sensor_errors.is_empty()` — it must execute on ALL non-error query paths, \
             including partial-failure (sensor_errors non-empty).",
        );
        let normalized_str = normalized_pql
            .as_str()
            .expect("BC-2.11.018 EC-11-054: normalized_pql must be a JSON string, not null");
        assert!(
            !normalized_str.is_empty(),
            "BC-2.11.018 EC-11-054: normalized_pql must be non-empty on partial-failure path; \
             got empty string"
        );

        // ASSERTION 3 (LOAD-BEARING — partial-failure branch proof):
        // sensor_errors MUST be non-empty. This assertion is IMPOSSIBLE via the
        // empty-registry zero-row path (no adapters → no fan-out → sensor_errors=[]).
        // If this assertion passes, the test is driving the REAL partial-failure branch
        // (AlwaysFailsAdapter::fetch returned HTTP 503 → AllTargetsFailed → sensor_errors).
        //
        // The sensor_errors field in the query response is populated from
        // QueryResult.sensor_errors in PrismServer::query (OBS-1 fix in server.rs:
        // `"sensor_errors": result.sensor_errors` added to the payload).
        let sensor_errors = results
            .get("sensor_errors")
            .expect(
                "BC-2.11.018 EC-11-054: sc['results']['sensor_errors'] must be present in \
                 the query response. FIX: add `\"sensor_errors\": result.sensor_errors` to \
                 the payload json! in PrismServer::query in server.rs.",
            )
            .as_array()
            .expect("BC-2.11.018 EC-11-054: sensor_errors must be a JSON array");
        assert!(
            !sensor_errors.is_empty(),
            "BC-2.11.018 EC-11-054 (partial-failure branch proof): sensor_errors MUST be \
             non-empty. If this fails, the test is NOT exercising the partial-failure branch — \
             it may have fallen back to the zero-row proxy path (empty registry). \
             Check that AlwaysFailsAdapter is registered in make_server_with_failing_adapter \
             and that resolve_source_refs finds a non-empty registry. \
             Got sensor_errors: {:?}",
            sensor_errors
        );
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

    // =========================================================================
    // BC-2.11.016 / AC-001 — E-QUERY-038 MCP error envelope carries
    // available_columns + did_you_mean (F-PRL-CRIT-001 Red Gate)
    // =========================================================================

    /// BC-2.11.016 / AC-001 — The MCP `query`-tool error envelope for E-QUERY-038
    /// MUST contain `available_columns` (non-empty array including "severity") and
    /// `did_you_mean` == "severity" when the typo is within Levenshtein distance ≤ 3.
    ///
    /// # What is being proved
    ///
    /// The E-QUERY-038 gate correctly COMPUTES `available_columns` and `did_you_mean`
    /// and stores them in `ColumnNotFoundDetails`. This test proves they are also
    /// THREADED THROUGH to the MCP error response that the LLM agent actually receives
    /// — i.e., they appear in `structured_content.error` of the `CallToolResult`.
    ///
    /// # LOAD-BEARING (TD-VSDD-059, F-PRL-CRIT-001)
    ///
    /// - Asserts on `call_result.structured_content["error"]["available_columns"]` —
    ///   the actual MCP envelope the LLM agent sees, NOT on `ColumnNotFoundDetails` fields.
    /// - Deleting `available_columns` from `StructuredErrorFields` or failing to populate
    ///   it in the `ColumnNotFound` arm of `prism_error_to_structured_call_result` WILL
    ///   break assertion #2 — the field will be absent from the JSON.
    /// - Deleting `did_you_mean` from `StructuredErrorFields` or failing to populate it
    ///   WILL break assertion #3.
    /// - No skip guard; no `#[ignore]`; drives in-process (plan-time gate, no live sensor).
    ///
    /// # Why FAILS on HEAD b1fe61b6
    ///
    /// `StructuredErrorFields` has no `available_columns` or `did_you_mean` fields.
    /// The `ColumnNotFound` arm in `prism_error_to_structured_call_result` uses
    /// `PrismError::ColumnNotFound(..)` (wildcard — discards the inner `ColumnNotFoundDetails`).
    /// Neither field is inserted into `build_structured_error_response`'s `error_obj`.
    /// Therefore `sc["error"]["available_columns"]` is ABSENT → the `.expect(...)` panics
    /// with "BC-2.11.016 AC-001: E-QUERY-038 MCP error MUST have 'available_columns'…".
    ///
    /// # FIX (for the implementer)
    ///
    /// 1. Add `available_columns: Option<Vec<String>>` and `did_you_mean: Option<String>`
    ///    to `StructuredErrorFields` with `#[serde(skip_serializing_if = "Option::is_none")]`.
    /// 2. Change the `PrismError::ColumnNotFound(..)` wildcard arm in
    ///    `prism_error_to_structured_call_result` to bind the inner struct:
    ///    `PrismError::ColumnNotFound(ref d) => VariantMeta { ...,
    ///       available_columns: Some(d.available_columns.clone()),
    ///       did_you_mean: d.did_you_mean.clone(), ... }`.
    /// 3. Add `VariantMeta::available_columns` and `VariantMeta::did_you_mean` fields.
    /// 4. In `build_structured_error_response`, emit:
    ///    ```
    ///    if let Some(cols) = fields.available_columns { error_obj["available_columns"] = ... }
    ///    if let Some(dym) = fields.did_you_mean { error_obj["did_you_mean"] = ... }
    ///    ```
    ///
    /// # BC reference
    ///
    /// BC-2.11.016 v1.1 §"E-QUERY-038 error payload shape": `available_columns` ALWAYS present,
    /// `did_you_mean` present when Levenshtein distance ≤ 3.
    /// BC-2.11.016 §"Canonical Test Vectors" EC-11-039:
    ///   `sevrity` → `did_you_mean: "severity"`, `available_columns` includes "severity".
    ///
    /// # SID-1
    ///
    /// Drives in-process via the plan-time gate (no live sensor needed).
    /// The `make_server_with_engine` fixture uses `AdapterRegistry::new()` (empty registry),
    /// which means fan-out never occurs — the query is rejected at plan time before fan-out.
    #[tokio::test]
    async fn test_BC_2_11_016_ac001_column_not_found_mcp_error_carries_available_columns_and_did_you_mean(
    ) {
        use prism_core::column::ColumnType;

        // Register `crowdstrike_alerts` with `severity` (and other columns) so the
        // E-QUERY-037 table-availability gate passes. The column `sevrity` (typo) is NOT
        // registered — the E-QUERY-038 column-not-found gate fires.
        let columns = vec![
            ColumnSpec::new("severity", ColumnType::String, None, vec![]),
            ColumnSpec::new("detection_id", ColumnType::String, None, vec![]),
            ColumnSpec::new("host_name", ColumnType::String, None, vec![]),
        ];
        let (key, resolved) = make_resolved("crowdstrike", "alerts", columns, "acme");
        let mut map = HashMap::new();
        map.insert(key, resolved.clone());
        let server = make_server_with_engine(map, vec![resolved.spec.clone()]);

        // BC-2.11.016 §Canonical Test Vectors EC-11-039:
        //   "sevrity" is a 1-edit-distance typo of "severity" (Levenshtein distance 1).
        //   The WHERE clause ensures the column is REFERENCED so the gate fires on it.
        //   AC-001 query: `SELECT * FROM crowdstrike_alerts WHERE sevrity = 'high'`
        let params: QueryToolParams = serde_json::from_str(
            r#"{"query": "SELECT * FROM crowdstrike_alerts WHERE sevrity = 'high'"}"#,
        )
        .expect("QueryToolParams JSON must deserialize");
        let call_result = server
            .query(Parameters(params))
            .await
            .expect("domain errors must return Ok(structured_error), not Err");

        // ASSERTION 1: The MCP tool returned an error (is_error == Some(true)).
        // MCP code must be -32602 INVALID_PARAMS per BC-2.11.016 §"Structured error response".
        assert_eq!(
            call_result.is_error,
            Some(true),
            "BC-2.11.016 AC-001: column typo 'sevrity' MUST produce is_error=true. \
             Got is_error={:?}. \
             If the query SUCCEEDS (is_error != Some(true)), the E-QUERY-038 gate is not firing. \
             Check that check_query_column_availability is wired into the plan-time path in engine.rs.",
            call_result.is_error
        );

        let sc = call_result
            .structured_content
            .expect("BC-2.11.016 AC-001: structured_content must be present on E-QUERY-038 error");

        let error_obj = sc
            .get("error")
            .expect("BC-2.11.016 AC-001: sc['error'] must be present on E-QUERY-038 error");

        // Verify E-QUERY-038 code for debugging context.
        let code_val = error_obj
            .get("code")
            .and_then(|v| v.as_str())
            .unwrap_or("<absent>");
        assert_eq!(
            code_val, "E-QUERY-038",
            "BC-2.11.016 AC-001: error code must be 'E-QUERY-038'; got '{code_val}'. \
             If 'E-INT-001' appears, the ColumnNotFound arm is falling through to the catch-all."
        );

        // ASSERTION 2 (LOAD-BEARING — available_columns in MCP envelope):
        // `available_columns` must be present in sc["error"] as a non-empty JSON array
        // containing "severity". This is the field the LLM agent receives to self-correct.
        //
        // FAILS ON HEAD b1fe61b6 because StructuredErrorFields has no available_columns field
        // and ColumnNotFound arm uses (..) wildcard — the field is ABSENT from the JSON.
        //
        // FIX: add available_columns: Option<Vec<String>> to StructuredErrorFields and
        //      populate it in the ColumnNotFound arm of prism_error_to_structured_call_result.
        let available_columns_val = error_obj.get("available_columns").expect(
            "BC-2.11.016 AC-001: E-QUERY-038 MCP error MUST have 'available_columns' field. \
             ABSENT on current HEAD because StructuredErrorFields has no available_columns field \
             and the ColumnNotFound arm uses (..) wildcard (discards ColumnNotFoundDetails). \
             FIX: (1) add `available_columns: Option<Vec<String>>` to StructuredErrorFields; \
             (2) change PrismError::ColumnNotFound(..) to PrismError::ColumnNotFound(ref d) and \
             set available_columns: Some(d.available_columns.clone()) in VariantMeta; \
             (3) emit it in build_structured_error_response as a JSON array.",
        );
        let available_columns_arr = available_columns_val.as_array().expect(
            "BC-2.11.016 AC-001: available_columns must be a JSON array, not a string/null",
        );
        assert!(
            !available_columns_arr.is_empty(),
            "BC-2.11.016 AC-001: available_columns must be non-empty (severity, detection_id, \
             host_name were registered); got empty array. \
             BC-2.11.016 §payload: 'ALWAYS present (never null, never omitted)'. \
             Check that ColumnNotFoundDetails.available_columns is populated in engine.rs."
        );
        let severity_val = serde_json::Value::String("severity".to_string());
        assert!(
            available_columns_arr.contains(&severity_val),
            "BC-2.11.016 AC-001: available_columns MUST contain 'severity' — it is a registered \
             column in crowdstrike_alerts for org 'acme'. Got: {:?}",
            available_columns_arr
        );

        // ASSERTION 3 (LOAD-BEARING — did_you_mean in MCP envelope):
        // `did_you_mean` must be present and equal "severity".
        // Levenshtein("sevrity", "severity") = 1 (one transposition) — within threshold ≤ 3.
        // BC-2.11.016 §payload: "present when Levenshtein distance ≤ 3".
        // BC-2.11.016 §Canonical Test Vectors EC-11-039: `did_you_mean: "severity"`.
        //
        // FAILS ON HEAD b1fe61b6 because StructuredErrorFields has no did_you_mean field.
        //
        // FIX: add `did_you_mean: Option<String>` to StructuredErrorFields and populate it
        //      from d.did_you_mean.clone() in the ColumnNotFound arm.
        let did_you_mean_val = error_obj.get("did_you_mean").expect(
            "BC-2.11.016 AC-001: E-QUERY-038 MCP error MUST have 'did_you_mean' field when \
             Levenshtein distance ≤ 3. ABSENT on current HEAD. \
             The typo 'sevrity' is distance 1 from 'severity' — well within the ≤3 threshold. \
             FIX: add `did_you_mean: Option<String>` to StructuredErrorFields; \
             populate from d.did_you_mean.clone() in ColumnNotFound arm of \
             prism_error_to_structured_call_result; emit in build_structured_error_response.",
        );
        assert_eq!(
            did_you_mean_val.as_str(),
            Some("severity"),
            "BC-2.11.016 AC-001: did_you_mean must be 'severity' (distance-1 match); \
             got: {:?}. \
             The ColumnNotFoundDetails.did_you_mean is computed by strsim::levenshtein in \
             engine.rs — verify the computation runs and the result is threaded through to \
             StructuredErrorFields.did_you_mean.",
            did_you_mean_val
        );
    }

    /// BC-2.11.016 / AC-001 (negative) — When no column is within Levenshtein distance ≤ 3,
    /// the MCP error MUST carry `available_columns` (still present, non-empty) but `did_you_mean`
    /// MUST be ABSENT from the error JSON (not null — the key must not exist at all).
    ///
    /// # BC reference
    ///
    /// BC-2.11.016 v1.1 §"Payload fields": `did_you_mean` "is omitted (not null, not empty
    /// string — absent)" when no column is within threshold.
    /// BC-2.11.016 §Canonical Test Vectors EC-11-040: `completely_bogus_field` →
    ///   `available_columns` includes real column names, `did_you_mean` absent.
    ///
    /// # LOAD-BEARING (TD-VSDD-059)
    ///
    /// - Deleting `available_columns` from the MCP response breaks assertion #2.
    /// - Incorrectly setting `did_you_mean` to a value (or null) when no near-match exists
    ///   breaks assertion #3 (the key must be ABSENT, not null).
    /// - No skip guard; drives in-process (plan-time gate, no live sensor).
    ///
    /// # Why FAILS on HEAD b1fe61b6
    ///
    /// Same root cause as the positive test above — `available_columns` is absent from the
    /// MCP envelope. The `.expect(...)` on `available_columns` panics before the `did_you_mean`
    /// assertion is reached.
    #[tokio::test]
    async fn test_BC_2_11_016_ac001_column_not_found_no_near_match_has_available_columns_but_no_did_you_mean(
    ) {
        use prism_core::column::ColumnType;

        let columns = vec![
            ColumnSpec::new("severity", ColumnType::String, None, vec![]),
            ColumnSpec::new("detection_id", ColumnType::String, None, vec![]),
            ColumnSpec::new("host_name", ColumnType::String, None, vec![]),
        ];
        let (key, resolved) = make_resolved("crowdstrike", "alerts", columns, "acme");
        let mut map = HashMap::new();
        map.insert(key, resolved.clone());
        let server = make_server_with_engine(map, vec![resolved.spec.clone()]);

        // BC-2.11.016 §Canonical Test Vectors EC-11-040:
        //   "zzz_bogus_col" has no near match in {severity, detection_id, host_name}
        //   — Levenshtein distance is >> 3 for all registered columns.
        let params: QueryToolParams = serde_json::from_str(
            r#"{"query": "SELECT * FROM crowdstrike_alerts WHERE zzz_bogus_col = 'x'"}"#,
        )
        .expect("QueryToolParams JSON must deserialize");
        let call_result = server
            .query(Parameters(params))
            .await
            .expect("domain errors must return Ok(structured_error), not Err");

        // ASSERTION 1: is_error must be Some(true).
        assert_eq!(
            call_result.is_error,
            Some(true),
            "BC-2.11.016 AC-001 (no-near-match): 'zzz_bogus_col' must produce is_error=true. \
             Got is_error={:?}.",
            call_result.is_error
        );

        let sc = call_result
            .structured_content
            .expect("BC-2.11.016 AC-001 (no-near-match): structured_content must be present");

        let error_obj = sc
            .get("error")
            .expect("BC-2.11.016 AC-001 (no-near-match): sc['error'] must be present");

        // ASSERTION 2 (LOAD-BEARING — available_columns ALWAYS present):
        // Even when did_you_mean is absent, available_columns must still be in the envelope.
        // BC-2.11.016 §payload: "ALWAYS present (never null, never omitted)".
        //
        // FAILS ON HEAD: same root cause — available_columns absent from StructuredErrorFields.
        let available_columns_val = error_obj.get("available_columns").expect(
            "BC-2.11.016 AC-001 (no-near-match): 'available_columns' MUST be present in the \
             MCP error envelope even when no Levenshtein match exists. \
             BC-2.11.016 §payload: 'ALWAYS present (never null, never omitted)'. \
             FIX: same as the positive test — add available_columns to StructuredErrorFields \
             and populate it unconditionally from d.available_columns in the ColumnNotFound arm.",
        );
        let available_columns_arr = available_columns_val
            .as_array()
            .expect("BC-2.11.016 AC-001 (no-near-match): available_columns must be a JSON array");
        assert!(
            !available_columns_arr.is_empty(),
            "BC-2.11.016 AC-001 (no-near-match): available_columns must be non-empty; \
             severity, detection_id, host_name were registered. Got empty array."
        );

        // ASSERTION 3 (LOAD-BEARING — did_you_mean ABSENT when no near match):
        // BC-2.11.016 §payload: "If absent (no match within threshold), the field is omitted
        //   (not null, not empty string — absent)."
        // BC-2.11.016 §Canonical Test Vectors EC-11-040: `did_you_mean` absent.
        //
        // The key must NOT exist in the JSON error object — not even as null.
        // If the implementation sets did_you_mean: null for no-match (instead of omitting
        // the key), this assertion will FAIL, correctly flagging the deviation.
        assert!(
            error_obj.get("did_you_mean").is_none(),
            "BC-2.11.016 AC-001 (no-near-match): 'did_you_mean' MUST be ABSENT (not null, \
             not empty string) when no column is within Levenshtein distance ≤ 3. \
             BC-2.11.016 §payload: 'omitted (not null, not empty string — absent)'. \
             Got: {:?}. \
             FIX: use #[serde(skip_serializing_if = \"Option::is_none\")] on the \
             did_you_mean field in StructuredErrorFields so None → key absent in JSON.",
            error_obj.get("did_you_mean")
        );
    }

    // =========================================================================
    // BC-2.11.018 EC-11-052 / F-PRL-MED-001 — normalized_pql must carry the
    // ALIAS-EXPANDED form, not the raw alias token from params.query
    // =========================================================================

    /// BC-2.11.018 EC-11-052 — `normalized_pql` on a successful alias query contains
    /// the ALIAS-EXPANDED canonical form, not the raw `@alias_name` token.
    ///
    /// # Background
    ///
    /// BC-2.11.018 §Field content states:
    ///   "alias-expanded to the canonical form the planner used"
    ///
    /// EC-11-052 states:
    ///   "Model submits a query that uses an alias defined via `create_alias` →
    ///    `normalized_pql` contains the alias-expanded form (the PQL the planner
    ///    executed, with alias replaced by its definition)"
    ///
    /// BC-2.11.009 §Postconditions states alias expansion runs BEFORE Chumsky parse:
    ///   "The fully expanded query is then ... passed to the parser."
    ///   "`result.context.expanded_query`" holds the post-expansion form.
    ///
    /// # What fails on HEAD 84052e8e (F-PRL-MED-001)
    ///
    /// `PrismServer::query` at server.rs lines ~1886-1889 computes `normalized_pql` by
    /// re-parsing `params.query` (raw input):
    ///
    /// ```rust
    /// let normalized_pql_str: Option<String> =
    ///     prism_query::filter_parser::PrismQlParser::parse(&params.query)
    ///         .ok()
    ///         .and_then(|ast| prism_query::engine::normalize_pql(&ast));
    /// ```
    ///
    /// When `params.query = "SELECT severity FROM crowdstrike_alerts WHERE @high_sev"`:
    /// - `PrismQlParser::parse` fails (`@` is not valid PQL syntax)
    /// - `.ok()` → `None`
    /// - `normalized_pql_str = None`
    /// - `normalized_pql` key is **absent** from the response
    ///
    /// The correct fix: use `result.context.expanded_query` (which the engine already
    /// populated with the alias-expanded form) as input to the normalizer.
    ///
    /// # Load-bearing assertion (TD-VSDD-059)
    ///
    /// The test drives through `PrismServer::query` → `QueryEngine::execute` with:
    ///   - A real `AliasStore` containing `@high_sev → severity = 'high'`
    ///   - A registered `crowdstrike_alerts` table with a `severity` column
    ///   - The alias store wired into BOTH the `QueryEngine` (for expansion) AND the
    ///     `PrismServer` (for alias tool consistency)
    ///
    /// Assertions:
    ///   1. Query succeeds (is_error != Some(true)) — alias expansion produces valid PQL
    ///   2. `normalized_pql` key is PRESENT — not absent (current HEAD failure mode)
    ///   3. `normalized_pql` value DOES NOT contain `@high_sev` (raw alias token)
    ///   4. `normalized_pql` value DOES contain `severity` (the expanded form)
    ///
    /// Assertion 2 FAILS on current HEAD with the message:
    ///   "BC-2.11.018 EC-11-052: normalized_pql must be PRESENT ... currently ABSENT"
    ///
    /// After the fix (change `params.query` → `result.context.expanded_query` at
    /// server.rs lines ~1886-1889), all 4 assertions pass.
    ///
    /// # SID-1 compliance
    ///
    /// In-process test — no live sensor, no DTU required.
    /// `AlwaysSucceedsCreds` is NOT needed because the engine has no adapters wired;
    /// the alias expansion (Step 0) completes before any fan-out attempt.
    #[tokio::test]
    async fn test_BC_2_11_018_ec11052_normalized_pql_contains_alias_expanded_form_not_raw_token() {
        use prism_core::column::ColumnType;
        use prism_query::alias_store::AliasStore;
        use prism_query::alias_types::{AliasEntry, AliasScope};
        use std::sync::{Arc, Mutex};

        // ---- Step 1: build a populated AliasStore via AliasStore::load() ----
        //
        // `create_or_update` is pub(crate) and cannot be called from prism-mcp/tests/.
        // Alternative: write an aliases.toml to a temp dir and call AliasStore::load().
        //
        // TOML format (AliasesFile { aliases: Vec<AliasEntry> }):
        //   [[aliases]]
        //   name = "high_sev"
        //   scope = "global"          ← AliasScope::Global serializes to "global"
        //   query = "severity = 'high'"
        //
        // The alias expands @high_sev → severity = 'high' (a simple filter predicate).
        // We use it in the WHERE clause: "SELECT severity FROM crowdstrike_alerts WHERE @high_sev"
        // After expansion the engine parses: "SELECT severity FROM crowdstrike_alerts WHERE severity = 'high'"
        let tmpdir = tempfile::tempdir().expect("create tmpdir for alias store");
        let alias_toml_path = tmpdir.path().join("aliases.toml");
        std::fs::write(
            &alias_toml_path,
            r#"
[[aliases]]
name = "high_sev"
scope = "global"
query = "severity = 'high'"
"#,
        )
        .expect("write aliases.toml must succeed");

        let alias_store = AliasStore::load(&alias_toml_path)
            .expect("AliasStore::load must succeed for valid aliases.toml");
        let alias_arc = Arc::new(Mutex::new(alias_store));

        // ---- Step 2: build a QueryEngine + TableRegistry with @high_sev expansion ----
        //
        // The QueryEngine MUST have the alias_store wired so that execute_inner Step 0
        // expands @high_sev → severity = 'high' before Chumsky parse. Without the alias
        // store, execute("... WHERE @high_sev") fails with E-ALIAS-001 (alias not found)
        // or a parse error on `@`.
        //
        // `QueryEngine::with_alias_store` is the test seam added by F-PRL-MED-001
        // fix-burst (parallel to `with_credential_resolver`; 4-line builder in engine.rs).
        let columns = vec![
            ColumnSpec::new("severity", ColumnType::String, None, vec![]),
            ColumnSpec::new("detection_id", ColumnType::String, None, vec![]),
        ];
        let (key, resolved) = make_resolved("crowdstrike", "alerts", columns, "acme");
        let mut spec_map = HashMap::new();
        spec_map.insert(key, resolved.clone());

        let registry = Arc::new(TableRegistry::new());
        registry
            .register_sensor(&resolved.spec)
            .expect("register_sensor must not fail");

        let engine = {
            use prism_credentials::InMemoryCredentialStore;
            use prism_query::engine::{QueryEngine, QueryEngineConfig};
            use prism_query::scoping::ClientRegistry;
            QueryEngine::new_with_cache_config(
                Arc::new(prism_sensors::AdapterRegistry::new()),
                Arc::new(InMemoryCredentialStore::new()),
                Arc::new(prism_ocsf::OcsfNormalizer::new()),
                Arc::new(ClientRegistry::new(vec![])),
                QueryEngineConfig::default(),
                prism_query::cache::CacheConfig::default(),
            )
            .with_alias_store(Arc::clone(&alias_arc))
            .with_table_registry(registry)
        };
        // Wire resolved_spec_map for the plan-time table availability gate (E-QUERY-037).
        // Without this, the query would be rejected before reaching alias expansion.
        let mut engine = engine;
        engine.resolved_spec_map = Some(Arc::new(arc_swap::ArcSwap::new(Arc::new(spec_map))));

        // ---- Step 3: build PrismServer wired with both engine + alias_store ----
        let server = PrismServer::new()
            .with_query_engine(Arc::new(engine))
            .with_alias_store_for_test(Arc::clone(&alias_arc));

        // ---- Step 4: submit a query using the @high_sev alias ----
        //
        // params.query = "SELECT severity FROM crowdstrike_alerts WHERE @high_sev"
        // Engine Step 0 expands: @high_sev → severity = 'high'
        // Engine then parses+executes: "SELECT severity FROM crowdstrike_alerts WHERE severity = 'high'"
        // result.context.expanded_query = "SELECT severity FROM crowdstrike_alerts WHERE severity = 'high'"
        //
        // CURRENT HEAD (84052e8e) failure path:
        //   server.rs lines ~1886-1889 re-parse params.query = "...WHERE @high_sev"
        //   PrismQlParser::parse("...WHERE @high_sev") → Err (@ not valid PQL syntax)
        //   .ok().and_then(…) → None
        //   normalized_pql key is ABSENT from response
        //   → ASSERTION 2 fails: "normalized_pql must be PRESENT … currently ABSENT"
        //
        // AFTER FIX:
        //   server.rs uses result.context.expanded_query = "...WHERE severity = 'high'"
        //   PrismQlParser::parse succeeds → normalize_pql produces canonical form
        //   normalized_pql = "SELECT severity FROM crowdstrike_alerts WHERE severity = 'high'"
        //   → All 4 assertions pass
        let params: QueryToolParams = serde_json::from_str(
            r#"{"query": "SELECT severity FROM crowdstrike_alerts WHERE @high_sev"}"#,
        )
        .expect("QueryToolParams JSON must deserialize");

        let call_result = server
            .query(Parameters(params))
            .await
            .expect("query must return Ok (domain errors → Ok(structured_error))");

        // ASSERTION 1: query must SUCCEED (alias expansion produces valid PQL)
        // If the engine has no alias_store, execute fails with E-ALIAS-001, not success.
        // If the alias_store is wired correctly, expansion succeeds and the engine returns Ok.
        assert_ne!(
            call_result.is_error,
            Some(true),
            "BC-2.11.018 EC-11-052: query with @high_sev alias MUST succeed (is_error != true). \
             If this fails, the alias_store is not wired into the QueryEngine — \
             the with_alias_store() builder must be called on the engine. \
             structured_content: {:?}",
            call_result.structured_content
        );

        let sc = call_result
            .structured_content
            .expect("BC-2.11.018 EC-11-052: structured_content must be present on success");
        let results = sc
            .get("results")
            .expect("BC-2.11.018 EC-11-052: sc['results'] must be present on success");

        // ASSERTION 2 (LOAD-BEARING — F-PRL-MED-001 RED GATE):
        // normalized_pql MUST be PRESENT.
        //
        // FAILS ON HEAD 84052e8e because server.rs re-parses params.query (which
        // contains @high_sev — not valid PQL), so parse returns Err, .ok() → None,
        // and the key is absent from the JSON payload.
        //
        // FIX: change server.rs lines ~1886-1889 from:
        //   PrismQlParser::parse(&params.query)
        // to:
        //   PrismQlParser::parse(&result.context.expanded_query)
        // so the normalizer receives the EXPANDED form (valid PQL) rather than the
        // raw alias token form (invalid PQL).
        let normalized_pql = results.get("normalized_pql").expect(
            "BC-2.11.018 EC-11-052 (F-PRL-MED-001 RED GATE): normalized_pql MUST be PRESENT \
             in the MCP response for a successful alias query. \
             CURRENT BEHAVIOR: key is ABSENT because server.rs re-parses params.query \
             ('...WHERE @high_sev'), which fails Chumsky parse (@ is not valid PQL), \
             so .ok().and_then(…) returns None and the key is omitted. \
             FIX: in PrismServer::query at server.rs lines ~1886-1889, change \
             PrismQlParser::parse(&params.query) to \
             PrismQlParser::parse(&result.context.expanded_query) so the normalizer \
             receives the alias-expanded form.",
        );

        let normalized_str = normalized_pql.as_str().expect(
            "BC-2.11.018 EC-11-052: normalized_pql must be a JSON string (not null/object)",
        );

        assert!(
            !normalized_str.is_empty(),
            "BC-2.11.018 EC-11-052: normalized_pql must be non-empty; got empty string"
        );

        // ASSERTION 3 (LOAD-BEARING — alias token must NOT appear):
        // The raw alias token @high_sev must not appear in normalized_pql.
        // If the fix accidentally uses params.query (verbatim echo), @high_sev would appear.
        assert!(
            !normalized_str.contains("@high_sev"),
            "BC-2.11.018 EC-11-052: normalized_pql MUST NOT contain raw alias token '@high_sev'. \
             normalized_pql must reflect the canonical expanded form. \
             Got: '{normalized_str}'"
        );

        // ASSERTION 4 (LOAD-BEARING — expanded form must appear):
        // The expanded predicate 'severity' must appear in normalized_pql.
        // This proves the normalizer ran on the EXPANDED form, not the raw alias form.
        assert!(
            normalized_str.contains("severity"),
            "BC-2.11.018 EC-11-052: normalized_pql MUST contain 'severity' (the expanded \
             predicate from @high_sev → severity = 'high'). \
             Got: '{normalized_str}'"
        );

        // ASSERTION 5 (canonical form — table name must survive normalization):
        assert!(
            normalized_str.contains("crowdstrike_alerts"),
            "BC-2.11.018 EC-11-052: normalized_pql must contain table name 'crowdstrike_alerts'; \
             Got: '{normalized_str}'"
        );

        // ASSERTION 6 (canonical form — uppercase SELECT):
        assert!(
            normalized_str.contains("SELECT"),
            "BC-2.11.018 EC-11-052: normalized_pql must use uppercase SELECT (canonical form); \
             Got: '{normalized_str}'"
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
