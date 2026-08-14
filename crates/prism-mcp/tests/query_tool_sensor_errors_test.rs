// SPDX-License-Identifier: Apache-2.0
//! RG-016: `query` MCP tool `sensor_errors` wire field MUST carry per-target HTTP detail.
//!
//! # BC authority
//!
//! - BC-2.11.001 v1.23 §Postconditions — `sensor_errors` per-target HTTP detail contract
//! - AC-QERR-001 — per-target HTTP status+body in `sensor_errors`
//! - EC-11-088 — HTTP 4xx response with non-empty body: `"<table>: HTTP <status>: <body>"`
//! - EC-11-089 — HTTP 5xx response with empty body: `"<table>: HTTP <status>"` (no trailing `: `)
//! - EC-11-090 — body exceeding 256 bytes truncated to first 256 UTF-8 bytes via
//!   `sanitize_body_snippet_bytes`; control bytes replaced with space
//!
//! # Red Gate reason (pre-fix: materialization Err arm re-prefixed an already-prefixed HttpError.body)
//!
//! On the real spec-driven query path, `pipeline.rs` `issue_request_with_retry` builds
//! `detail = format!("HTTP {status}: {body_snippet}")` where `{status}` is a reqwest
//! `StatusCode` Display (e.g. "403 Forbidden" — includes reason phrase). Then
//! `spec_driven_adapter.rs` `map_spec_engine_error_to_sensor_error` Arm 1 sets
//! `SensorError::HttpError { body: detail.clone() }` — so `HttpError.body` carries
//! the ALREADY-PREFIXED string `"HTTP 403 Forbidden: <snippet>"`. Finally
//! `materialization.rs` formats `"{table}: HTTP {status}: {body}"` — prepending
//! ANOTHER `"HTTP {status}: "` prefix, producing the doubled output
//! `"claroty_devices: HTTP 403: HTTP 403 Forbidden: <snippet>"` (F-P37-HIGH-001).
//!
//! The test stubs now inject `HttpError.body` in the PRODUCTION SHAPE (the pipeline
//! detail string) so that the assertion catches the doubling at the wire level.
//! The fix must strip the `"HTTP {status_reason}: "` prefix from `detail` before
//! populating `HttpError.body`, so consumers see the raw sanitized snippet.
//!
//! # Test seam
//!
//! Tests wire a stub `SensorAdapter` (`StubHttpErrorAdapter`) that returns
//! `Err(SensorError::HttpError { status, body, sensor })` directly — no HTTP calls,
//! no external service, no loopback socket required (SID-1: unit test at the adapter
//! boundary). Stub bodies are set to the PRODUCTION SHAPE (pipeline detail format:
//! `"HTTP {status_reason}: {body_snippet}"`) to exercise the doubled-prefix defect path:
//!   stub adapter (production-shaped body) → fan_out() → AllTargetsFailed
//!   → materialization.rs Err arm → sensor_errors Vec<String>
//!   → PrismServer::query payload → structured_content wire
//!
//! # Test catalogue
//!
//! | RG   | EC          | Scenario                              | Pre-fix failure (F-P37-HIGH-001)                    |
//! |------|-------------|---------------------------------------|-----------------------------------------------------|
//! | RG-016 | EC-11-088 | 403 non-empty body, production-shaped | doubled prefix `"HTTP 403: HTTP 403 Forbidden: ..."` |
//! | RG-016 | EC-11-089 | 503 empty body (already correct shape)| `"rg016b_devices: HTTP 503"` unchanged               |
//! | RG-016 | EC-11-090 | 300-byte production-shaped body       | doubled prefix + wrong truncation point              |
//! | RG-016 | absent     | success query, no errors              | `"sensor_errors":[]` present (must be absent)        |

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, sync::Arc};

    use arrow::{
        array::StringArray,
        datatypes::{DataType, Field, Schema},
        record_batch::RecordBatch,
    };
    use async_trait::async_trait;
    use prism_core::{OrgId, OrgSlug, SensorId};
    use prism_credentials::InMemoryCredentialStore;
    use prism_mcp::server::{PrismServer, QueryToolParams};
    use prism_query::{
        engine::{QueryEngine, QueryEngineConfig},
        scoping::ClientRegistry,
        table_registry::TableRegistry,
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
    // Stub types
    // =========================================================================

    /// Stub sensor adapter that returns `SensorError::HttpError` with a
    /// configurable status code and body.
    ///
    /// Used as the test seam for per-target HTTP error surfacing in `sensor_errors`.
    /// No HTTP calls are made — errors are returned directly at the adapter boundary
    /// (SID-1: unit test without live DTU or loopback socket).
    ///
    /// Call chain exercised:
    ///   `StubHttpErrorAdapter::fetch` returns `Err(SensorError::HttpError { status, body })`
    ///   → `fan_out()` collects as `FanOutError` → all targets fail → `Err(AllTargetsFailed)`
    ///   → `materialization.rs` Err arm pushes to `sensor_errors`
    ///   → `PrismServer::query` serialises `sensor_errors` into `structured_content` wire
    struct StubHttpErrorAdapter {
        sensor_id: SensorId,
        http_status: u16,
        http_body: String,
    }

    #[async_trait]
    impl SensorAdapter for StubHttpErrorAdapter {
        fn sensor_type(&self) -> SensorId {
            self.sensor_id.clone()
        }

        fn sensor_name(&self) -> &'static str {
            "stub-http-error-adapter"
        }

        async fn fetch(
            &self,
            _spec: &SensorAdapterSpec,
            _params: &SensorQueryParams,
            _auth: &dyn SensorAuth,
        ) -> Result<Vec<RecordBatch>, SensorError> {
            Err(SensorError::HttpError {
                sensor: self.sensor_id.to_string(),
                status: self.http_status,
                body: self.http_body.clone(),
            })
        }
    }

    /// Stub auth token — ignored by `StubHttpErrorAdapter::fetch`.
    struct StubAuth;

    impl SensorAuth for StubAuth {
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }

        fn auth_type_name(&self) -> &'static str {
            "custom_via_plugin"
        }
    }

    /// Credential resolver that always succeeds (returns `StubAuth`).
    ///
    /// Required so `fan_out()` reaches the adapter boundary rather than
    /// short-circuiting with a `CredentialNotFound` error.
    /// Pattern matches `AlwaysSucceedsCreds` in `normalized_pql.rs` (SID-1).
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
    ///
    /// Mirrors `make_resolved` from `normalized_pql.rs` / `bc_2_11_001_null_row_shape_test.rs`.
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

    /// Build a `PrismServer` wired with a `StubHttpErrorAdapter` for the given
    /// sensor/table/org combination.
    ///
    /// The adapter returns `SensorError::HttpError { status, body }` on every
    /// `fetch()` call, driving the `AllTargetsFailed` path in the materialization
    /// pipeline. `AlwaysSucceedsCreds` ensures `fan_out()` reaches the adapter
    /// boundary (SID-1: no live DTU, no external service, no loopback socket).
    ///
    /// `sensor_id` MUST NOT contain underscores — `sensor_id_from_table_name` in the
    /// query engine splits the DataFusion table name "{sensor_id}_{table_name}" on the
    /// first underscore to derive the sensor prefix for adapter lookup.
    fn make_server_with_http_error_adapter(
        sensor_id: &str,
        table_name: &str,
        org: &str,
        http_status: u16,
        http_body: &str,
    ) -> (PrismServer, OrgId) {
        use prism_core::column::ColumnType;

        // Single column spec so the TableRegistry and DataFusion schema are valid.
        let columns = vec![ColumnSpec::new("item_id", ColumnType::String, None, vec![])];

        // Deterministic OrgId keyed on the http_status for audit cross-reference.
        // Sentinel byte encodes http_status low byte for traceability.
        let status_byte = (http_status & 0xff) as u8;
        let org_id = OrgId::from_uuid(uuid::Uuid::from_bytes([
            0x01,
            0xac,
            0xce,
            0x77,
            0x0e,
            0x1a,
            0x7a,
            0x8b,
            0x80,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            status_byte,
        ]));

        let (key, resolved) = make_resolved(sensor_id, table_name, columns, org);
        let mut resolved_map = HashMap::new();
        resolved_map.insert(key, resolved.clone());

        let registry = Arc::new(TableRegistry::new());
        registry
            .register_sensor(&resolved.spec)
            .expect("register_sensor must not fail in fixture");

        let sensor_id_typed = SensorId::new(sensor_id);
        let adapter: Arc<dyn SensorAdapter> = Arc::new(StubHttpErrorAdapter {
            sensor_id: sensor_id_typed,
            http_status,
            http_body: http_body.to_string(),
        });
        let mut adapter_registry = AdapterRegistry::new();
        adapter_registry.register(org_id, adapter);

        let mut engine = QueryEngine::new_with_cache_config(
            Arc::new(adapter_registry),
            Arc::new(InMemoryCredentialStore::new()),
            Arc::new(prism_ocsf::OcsfNormalizer::new()),
            Arc::new(ClientRegistry::new(vec![])),
            QueryEngineConfig::default(),
            prism_query::cache::CacheConfig::default(),
        );
        engine = engine.with_credential_resolver(Arc::new(AlwaysSucceedsCreds));
        engine = engine.with_resolved_spec_map(Arc::new(resolved_map));
        engine = engine.with_table_registry(registry);

        let server = PrismServer::new().with_query_engine(Arc::new(engine));
        (server, org_id)
    }

    /// Build a `PrismServer` wired with an EMPTY `AdapterRegistry` but with the
    /// given sensor/table/org registered in `TableRegistry` and `resolved_spec_map`.
    ///
    /// Used for the absent-on-success assertion: no fan-out targets → no sensor_errors →
    /// the fix must omit the `"sensor_errors"` key from the response entirely.
    fn make_server_with_no_adapters(sensor_id: &str, table_name: &str, org: &str) -> PrismServer {
        use prism_core::column::ColumnType;

        let columns = vec![ColumnSpec::new("item_id", ColumnType::String, None, vec![])];
        let (key, resolved) = make_resolved(sensor_id, table_name, columns, org);
        let mut resolved_map = HashMap::new();
        resolved_map.insert(key, resolved.clone());

        let registry = Arc::new(TableRegistry::new());
        registry
            .register_sensor(&resolved.spec)
            .expect("register_sensor must not fail in fixture");

        let mut engine = QueryEngine::new_with_cache_config(
            Arc::new(AdapterRegistry::new()), // EMPTY — no fan-out, no sensor_errors
            Arc::new(InMemoryCredentialStore::new()),
            Arc::new(prism_ocsf::OcsfNormalizer::new()),
            Arc::new(ClientRegistry::new(vec![])),
            QueryEngineConfig::default(),
            prism_query::cache::CacheConfig::default(),
        );
        engine = engine.with_resolved_spec_map(Arc::new(resolved_map));
        engine = engine.with_table_registry(registry);

        PrismServer::new().with_query_engine(Arc::new(engine))
    }

    /// Build `QueryToolParams` from a SQL string.
    fn query_params(sql: &str) -> QueryToolParams {
        serde_json::from_str(&serde_json::json!({"query": sql}).to_string())
            .expect("QueryToolParams JSON must deserialize")
    }

    // =========================================================================
    // RG-016: per-target HTTP detail in `sensor_errors` wire field
    // =========================================================================

    /// RG-016: `query` MCP tool MUST surface per-target HTTP detail in `sensor_errors`.
    ///
    /// # What this test covers
    ///
    /// - **EC-11-088:** target HTTP 403 with non-empty body → `sensor_errors` entry is
    ///   `"{table}: HTTP 403: {body}"` (sanitized body, first 256 UTF-8 bytes)
    /// - **EC-11-089:** target HTTP 503 with EMPTY body → entry is `"{table}: HTTP 503"`
    ///   (status-only; NO trailing `: `)
    /// - **EC-11-090:** target HTTP 403 with 300-byte body → entry body portion is
    ///   truncated to first 256 bytes (optional strengthening, included)
    /// - **absent-on-success:** success query (no failing targets) → `sensor_errors` key
    ///   is ABSENT from serialised JSON (not `null`, not `[]`)
    ///
    /// # Red Gate failure (pre-fix: materialization Err arm re-prefixed an already-prefixed HttpError.body)
    ///
    /// **EC-11-088 FAILS (F-P37-HIGH-001):**
    ///   The stub body is set to the PRODUCTION SHAPE: `"HTTP 403 Forbidden: <snippet>"`.
    ///   `materialization.rs` formats `"{table}: HTTP {status}: {body}"`, producing the
    ///   doubled output `"rg016a_devices: HTTP 403: HTTP 403 Forbidden: access_denied..."`.
    ///   The assertion expects the single-prefix target format, so it FAILS pre-fix.
    ///
    /// **EC-11-090 FAILS (F-P37-HIGH-001):**
    ///   Same doubling — production-shaped body `"HTTP 403 Forbidden: " + 300×'x'` is
    ///   re-prefixed to `"HTTP 403: HTTP 403 Forbidden: " + 236×'x'` (256-byte cap on
    ///   the already-prefixed body), not the target `"HTTP 403: " + 256×'x'`.
    ///
    /// **EC-11-089 and absent-on-success:** these pass with or without the fix because
    ///   the 503 stub uses empty body (production-shaped) and the absent key was fixed
    ///   in the previous commit.
    ///
    /// # Mock backend note (SID-1)
    ///
    /// No HTTP calls are made. `StubHttpErrorAdapter::fetch` returns
    /// `Err(SensorError::HttpError { status, body })` directly at the adapter boundary —
    /// equivalent to a loopback mock backend but without socket overhead.
    /// This is the established prism-mcp test pattern (mirrors `AlwaysFailsAdapter` in
    /// `normalized_pql.rs`). No connection to api.claroty.com or any live service.
    ///
    /// BC-2.11.001 §Postconditions EC-11-088/089/090 | AC-QERR-001 |
    /// DEFECT-ADAPTER-TLS-XDOME-LIVE-001 RG-016
    #[tokio::test]
    async fn test_BC_2_11_001_query_sensor_errors_surfaces_per_target_http_detail() {
        // =====================================================================
        // Sub-test 1: EC-11-088 — HTTP 403 with non-empty body (production-shaped)
        //
        // Sensor "rg016a", table "devices" → DataFusion table "rg016a_devices".
        //
        // Stub body models the PRODUCTION HttpError.body shape: pipeline.rs
        // `issue_request_with_retry` formats detail as "HTTP {status_reason}: {snippet}"
        // and spec_driven_adapter.rs Arm 1 puts detail into HttpError.body verbatim.
        // So on the real spec-driven path HttpError.body = "HTTP 403 Forbidden: access_denied..."
        // (F-P37-HIGH-001).
        //
        // Adapter returns: SensorError::HttpError { status: 403,
        //   body: "HTTP 403 Forbidden: access_denied_by_security_policy" }
        // Expected sensor_errors[0] (TARGET):
        //   "rg016a_devices: HTTP 403: access_denied_by_security_policy"
        // Current (pre-fix) output (DOUBLED PREFIX):
        //   "rg016a_devices: HTTP 403: HTTP 403 Forbidden: access_denied_by_security_policy"
        // =====================================================================

        let (server_403, _) = make_server_with_http_error_adapter(
            "rg016a",
            "devices",
            "acme",
            403,
            "HTTP 403 Forbidden: access_denied_by_security_policy",
        );

        let result_403 = server_403
            .query(Parameters(query_params(
                "SELECT item_id FROM rg016a_devices LIMIT 1",
            )))
            .await
            .expect(
                "RG-016 EC-11-088: query MUST return Ok(QueryResult) on adapter error; \
                 sensor failures propagate to sensor_errors, not to is_error=true",
            );

        // Must NOT be a query-level error: sensor errors populate sensor_errors, not is_error.
        assert_ne!(
            result_403.is_error,
            Some(true),
            "RG-016 EC-11-088: partial-failure query MUST NOT return is_error=true; \
             sensor errors go to sensor_errors. structured_content: {:?}",
            result_403.structured_content
        );

        let sc_403 = result_403
            .structured_content
            .expect("RG-016 EC-11-088: structured_content must be present");

        // SID-2 wire-level assertion: serialize the full structured_content to bytes
        // and assert the EXACT per-target HTTP detail string appears at the wire level.
        let wire_403 =
            serde_json::to_string(&sc_403).expect("RG-016 EC-11-088: serialize must succeed");

        // ASSERTION 1 (SID-2 wire-level, EC-11-088):
        // sensor_errors wire MUST contain the per-target HTTP detail string.
        // FAILS pre-fix: wire contains "all targets failed (E-SENSOR-030)" not "HTTP 403: ..."
        assert!(
            wire_403.contains("rg016a_devices: HTTP 403: access_denied_by_security_policy"),
            "RG-016 EC-11-088 FAIL (SID-2 wire-level): sensor_errors wire MUST contain \
             per-target HTTP detail '\"rg016a_devices: HTTP 403: access_denied_by_security_policy\"'. \
             \nCURRENT (pre-fix) wire contains: 'all targets failed (E-SENSOR-030)' — \
             the HttpError status and body inside AllTargetsFailed.errors[0].error are discarded. \
             \nFIX: in materialization.rs Err arm, iterate AllTargetsFailed.errors and format \
             each as '<table>: HTTP <status>: <sanitized_body>' (non-empty body) or \
             '<table>: HTTP <status>' (empty body). \
             \nFull wire_403: {wire_403}"
        );

        // ASSERTION 2 (negative gate): aggregate 'all targets failed' format MUST be absent.
        // FAILS pre-fix: wire DOES contain "all targets failed (E-SENSOR-030)".
        assert!(
            !wire_403.contains("all targets failed (E-SENSOR-030)"),
            "RG-016 EC-11-088 FAIL (negative gate): sensor_errors MUST NOT contain \
             aggregate 'all targets failed (E-SENSOR-030)' format — per-target HTTP detail \
             replaces the aggregate. \
             \nFull wire_403: {wire_403}"
        );

        // ASSERTION 3 (structured content): navigate to sensor_errors[0] and check exact value.
        // FAILS pre-fix: sensor_errors[0] is "rg016a_devices: all targets failed (E-SENSOR-030)".
        let results_403 = sc_403
            .get("results")
            .expect("RG-016 EC-11-088: sc['results'] must be present");
        let errors_403 = results_403
            .get("sensor_errors")
            .and_then(|v| v.as_array())
            .expect("RG-016 EC-11-088: sensor_errors must be a non-null array on partial failure");
        assert_eq!(
            errors_403.len(),
            1,
            "RG-016 EC-11-088: sensor_errors must have exactly 1 entry (one failing target). \
             Got: {:?}",
            errors_403
        );
        assert_eq!(
            errors_403[0].as_str().unwrap_or(""),
            "rg016a_devices: HTTP 403: access_denied_by_security_policy",
            "RG-016 EC-11-088 FAIL (structured content): sensor_errors[0] MUST be \
             'rg016a_devices: HTTP 403: access_denied_by_security_policy'. \
             \nCURRENT (pre-fix): 'rg016a_devices: all targets failed (E-SENSOR-030)' — \
             per-target HTTP detail not surfaced."
        );

        // =====================================================================
        // Sub-test 2: EC-11-089 — HTTP 503 with EMPTY body
        //
        // Empty body → status-only format: "{table}: HTTP {status}"
        // NO trailing ": " after the status code.
        // Sensor "rg016b", table "devices" → DataFusion table "rg016b_devices".
        // =====================================================================

        let (server_503, _) = make_server_with_http_error_adapter(
            "rg016b", "devices", "acme", 503, "", // empty body
        );

        let result_503 = server_503
            .query(Parameters(query_params(
                "SELECT item_id FROM rg016b_devices LIMIT 1",
            )))
            .await
            .expect("RG-016 EC-11-089: query MUST return Ok(QueryResult) on adapter error");

        let sc_503 = result_503
            .structured_content
            .expect("RG-016 EC-11-089: structured_content must be present");

        let wire_503 =
            serde_json::to_string(&sc_503).expect("RG-016 EC-11-089: serialize must succeed");

        // ASSERTION 1 (SID-2 wire-level, EC-11-089): status-only format — no body, no ": " suffix.
        // FAILS pre-fix: wire has "all targets failed (E-SENSOR-030)" not "HTTP 503".
        assert!(
            wire_503.contains("rg016b_devices: HTTP 503"),
            "RG-016 EC-11-089 FAIL (SID-2 wire-level): empty-body sensor_errors MUST contain \
             'rg016b_devices: HTTP 503' (status-only, no trailing ': '). \
             \nCURRENT (pre-fix) wire: {wire_503}"
        );

        // ASSERTION 2 (EC-11-089 no-trailing-colon): empty body must NOT produce ": " after status.
        // "rg016b_devices: HTTP 503" is correct; "rg016b_devices: HTTP 503: " would be wrong.
        // FAILS pre-fix: pre-fix wire has "all targets failed", which does not contain
        // "HTTP 503: " either — but the positive assertion above would have already failed.
        // This assertion independently verifies the empty-body format invariant.
        assert!(
            !wire_503.contains("rg016b_devices: HTTP 503: "),
            "RG-016 EC-11-089 FAIL (no-trailing-colon): empty body MUST produce \
             'rg016b_devices: HTTP 503' with NO trailing ': '. \
             An empty body must not produce a colon-space suffix. \
             \nFull wire_503: {wire_503}"
        );

        // ASSERTION 3 (structured content): sensor_errors[0] exact value.
        // FAILS pre-fix: sensor_errors[0] is "rg016b_devices: all targets failed (E-SENSOR-030)".
        let results_503 = sc_503
            .get("results")
            .expect("RG-016 EC-11-089: sc['results'] must be present");
        let errors_503 = results_503
            .get("sensor_errors")
            .and_then(|v| v.as_array())
            .expect("RG-016 EC-11-089: sensor_errors must be a non-null array on partial failure");
        assert_eq!(
            errors_503[0].as_str().unwrap_or(""),
            "rg016b_devices: HTTP 503",
            "RG-016 EC-11-089 FAIL (structured content): empty-body sensor_errors[0] MUST be \
             'rg016b_devices: HTTP 503' (no trailing ': '). \
             \nCURRENT (pre-fix): 'rg016b_devices: all targets failed (E-SENSOR-030)'."
        );

        // =====================================================================
        // Sub-test 3: absent-on-success — no sensor errors → `sensor_errors` key ABSENT
        //
        // Success query (no failing adapters) → sensor_errors = Vec::new()
        // Spec: key MUST be ABSENT (not null, not []).
        // Current: PrismServer::query hardcodes "sensor_errors": result.sensor_errors,
        //          which serialises as "sensor_errors":[] even when Vec is empty.
        // =====================================================================

        let server_ok = make_server_with_no_adapters("rg016ok", "data", "acme");

        let result_ok = server_ok
            .query(Parameters(query_params(
                "SELECT item_id FROM rg016ok_data LIMIT 1",
            )))
            .await
            .expect(
                "RG-016 absent-on-success: success query (no adapters, empty result) \
                 MUST return Ok(QueryResult)",
            );

        let sc_ok = result_ok
            .structured_content
            .expect("RG-016 absent-on-success: structured_content must be present");

        let wire_ok = serde_json::to_string(&sc_ok)
            .expect("RG-016 absent-on-success: serialize must succeed");

        // ASSERTION 1 (SID-2 wire-level): the key "sensor_errors" MUST NOT appear in wire JSON.
        // FAILS pre-fix: wire contains '"sensor_errors":[]' because the payload json! literal
        // always includes "sensor_errors": result.sensor_errors regardless of emptiness.
        assert!(
            !wire_ok.contains("\"sensor_errors\""),
            "RG-016 absent-on-success FAIL (SID-2 wire-level): success query wire MUST NOT \
             contain '\"sensor_errors\"' key (BC-2.11.001 AC-QERR-001: key ABSENT when no errors). \
             \nCURRENT (pre-fix): PrismServer::query always emits 'sensor_errors': result.sensor_errors, \
             which serialises as '\"sensor_errors\":[]' on success — the key is always present. \
             \nFIX: conditionally omit the sensor_errors key when result.sensor_errors.is_empty(), \
             using the same pattern as 'normalized_pql' (conditional insert after payload construction). \
             \nFull wire_ok: {wire_ok}"
        );

        // ASSERTION 2 (structured content): sc["results"].get("sensor_errors") must be None.
        // FAILS pre-fix: returns Some(Value::Array([])).
        let results_ok = sc_ok
            .get("results")
            .expect("RG-016 absent-on-success: sc['results'] must be present");
        assert!(
            results_ok.get("sensor_errors").is_none(),
            "RG-016 absent-on-success FAIL (structured content): sc['results']['sensor_errors'] \
             MUST be absent (None) on success — not null, not []. \
             \nCURRENT (pre-fix): Some(Array([])) — key is always present in payload. \
             Got: {:?}",
            results_ok.get("sensor_errors")
        );

        // =====================================================================
        // Sub-test 4: EC-11-090 — body truncation at 256 UTF-8 bytes (production-shaped)
        //
        // Stub body models the PRODUCTION HttpError.body shape for a 403 with 300-byte
        // body: "HTTP 403 Forbidden: " + 300×'x'.  After the fix, spec_driven_adapter.rs
        // strips the "HTTP 403 Forbidden: " prefix → 300×'x'; materialization.rs then
        // applies sanitize_body_snippet_bytes(snippet, 256) → 256×'x'.
        //
        // Sensor "rg016d", table "devices" → DataFusion table "rg016d_devices".
        // Expected sensor_errors[0] (TARGET):
        //   "rg016d_devices: HTTP 403: " + "x".repeat(256)
        // Current (pre-fix) output: sanitize_body_snippet_bytes truncates the already-
        // prefixed body at 256 bytes, then materialization re-prefixes with "HTTP 403: ",
        // producing "rg016d_devices: HTTP 403: HTTP 403 Forbidden: " + 236×'x'.
        // =====================================================================

        let long_body = format!("HTTP 403 Forbidden: {}", "x".repeat(300));
        let (server_long, _) =
            make_server_with_http_error_adapter("rg016d", "devices", "acme", 403, &long_body);

        let result_long = server_long
            .query(Parameters(query_params(
                "SELECT item_id FROM rg016d_devices LIMIT 1",
            )))
            .await
            .expect("RG-016 EC-11-090: query MUST return Ok(QueryResult) on adapter error");

        let sc_long = result_long
            .structured_content
            .expect("RG-016 EC-11-090: structured_content must be present");

        let results_long = sc_long
            .get("results")
            .expect("RG-016 EC-11-090: sc['results'] must be present");
        let errors_long = results_long
            .get("sensor_errors")
            .and_then(|v| v.as_array())
            .expect("RG-016 EC-11-090: sensor_errors must be present on partial failure");

        // ASSERTION (EC-11-090 body truncation): body portion is exactly 256 bytes.
        // FAILS pre-fix: sensor_errors[0] is "rg016d_devices: all targets failed (E-SENSOR-030)".
        let expected_truncated = format!("rg016d_devices: HTTP 403: {}", "x".repeat(256));
        assert_eq!(
            errors_long[0].as_str().unwrap_or(""),
            expected_truncated,
            "RG-016 EC-11-090 FAIL (body truncation): 300-byte body MUST be truncated to \
             256 UTF-8 bytes in sensor_errors entry via sanitize_body_snippet_bytes(body, 256). \
             \nExpected: '{expected_truncated}' \
             \nCURRENT (pre-fix): 'rg016d_devices: all targets failed (E-SENSOR-030)' — \
             body is never surfaced, let alone truncated."
        );

        // Verify the body was genuinely truncated (not just accidentally 256 x's).
        // The original body was 300 bytes; the entry must NOT end with all 300 x's.
        assert_ne!(
            errors_long[0].as_str().unwrap_or(""),
            format!("rg016d_devices: HTTP 403: {}", "x".repeat(300)),
            "RG-016 EC-11-090: sensor_errors[0] must NOT contain the full 300-byte body — \
             truncation at 256 bytes is required."
        );
    }
}
