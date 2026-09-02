// SPDX-License-Identifier: Apache-2.0
//! Red Gate wire-shape tests for DEFECT-LIVE-ENVELOPE-OBS-001.
//!
//! Two envelope-contract defects from the v1 live validation:
//!
//! **OBS-1** (`_meta.data_source` must reflect the sensor even when all targets fail):
//! Root cause: `sensors_queried.insert(target.sensor_id.to_string())` in
//! `materialize_single_external_target` fires only in the `Ok(fan_result)` arm.
//! Empty/all-failed responses hit the Err arm → `sensors_queried` empty →
//! `_meta.data_source: ["unknown"]`.
//!
//! **OBS-2** (`_meta.has_more` MUST always be `false`; no cursor pagination):
//! Root cause: `server.rs` passes `result.is_truncated` directly as `has_more` to
//! `SafetyEnvelopeBuilder::wrap()`. ADR-060 §D8.7 requires `has_more = false` always;
//! truncation is signaled only via `results.is_truncated` + `results.total_available`.
//!
//! # BC authority
//!
//! - BC-2.09.008: `_meta.data_source`, `_meta.has_more`, `_meta.next_cursor` envelope contracts
//! - ADR-060 §D8.7: has_more MUST always be false (OFFSET not in grammar)
//!
//! # Test catalogue
//!
//! | Test | Defect | Red Gate? | Pre-fix failure |
//! |---|---|---|---|
//! | `test_BC_2_09_008_OBS_1_wire_data_source_reflects_sensor_on_all_targets_failed` | OBS-1 | YES — fails today | data_source = ["unknown"] instead of ["claroty"] |
//! | `test_BC_2_09_008_OBS_2_wire_has_more_always_false_when_truncated` | OBS-2 | YES — fails today | has_more = true (= is_truncated) violates ADR-060 §D8.7 |
//! | `test_BC_2_09_008_OBS_2_wire_has_more_false_when_not_truncated` | OBS-2 | Regression lock — may already pass | has_more already false when not truncated |
//!
//! # Wire-shape discipline (CLAUDE.md 2026-07-13)
//!
//! All envelope assertions operate on the SERIALIZED JSON output — the exact bytes the
//! LLM agent consumes — not on pre-serialization Rust structures (SID-2).
//!
//! # Mock backend note (SID-1)
//!
//! No HTTP calls are made. Stub `SensorAdapter` implementations return errors or rows
//! at the adapter boundary without loopback sockets or live services. This follows the
//! established prism-mcp test pattern from `query_tool_sensor_errors_test.rs`.

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
        adapter::FetchOutput, AdapterRegistry, CredentialResolver,
        QueryParams as SensorQueryParams, SensorAdapter, SensorAuth, SensorError,
        SensorSpec as SensorAdapterSpec,
    };
    use prism_spec_engine::{
        overlay::{OverlayLoader, ResolvedSensorSpec, ResolvedSpecKey, SensorInstanceOverlay},
        spec_parser::{AuthType, ColumnSpec, SensorSpec, TableSpec},
    };
    use rmcp::handler::server::wrapper::Parameters;

    // =========================================================================
    // Stub types
    // =========================================================================

    /// Stub sensor adapter that always returns `SensorError::HttpError`.
    ///
    /// Causes `fan_out()` to collect a `FanOutError` and — since all targets fail —
    /// return `Err(AllTargetsFailed)`. This drives the Err arm in
    /// `materialize_single_external_target`, where OBS-1 manifests:
    /// `sensors_queried.insert()` is absent from that arm.
    ///
    /// No HTTP calls are made (SID-1: unit test at the adapter boundary).
    struct AlwaysFailsAdapter {
        sensor_id: SensorId,
    }

    #[async_trait]
    impl SensorAdapter for AlwaysFailsAdapter {
        fn sensor_type(&self) -> SensorId {
            self.sensor_id.clone()
        }

        fn sensor_name(&self) -> &'static str {
            "always-fails-obs1-stub"
        }

        async fn fetch(
            &self,
            _spec: &SensorAdapterSpec,
            _params: &SensorQueryParams,
            _auth: &dyn SensorAuth,
        ) -> Result<FetchOutput, SensorError> {
            Err(SensorError::HttpError {
                sensor: self.sensor_id.to_string(),
                status: 503,
                body: String::new(),
            })
        }
    }

    /// Stub sensor adapter that returns `row_count` rows with a single `item_id` column.
    ///
    /// Used for OBS-2: when the adapter returns more rows than the tool-level limit
    /// (e.g., 10 rows with limit=3), the engine sets `is_truncated = true`. The pre-fix
    /// code passes `is_truncated` directly as `has_more` → `has_more = true`, violating
    /// ADR-060 §D8.7.
    struct ReturnsNRowsAdapter {
        sensor_id: SensorId,
        row_count: usize,
    }

    #[async_trait]
    impl SensorAdapter for ReturnsNRowsAdapter {
        fn sensor_type(&self) -> SensorId {
            self.sensor_id.clone()
        }

        fn sensor_name(&self) -> &'static str {
            "returns-n-rows-obs2-stub"
        }

        async fn fetch(
            &self,
            _spec: &SensorAdapterSpec,
            _params: &SensorQueryParams,
            _auth: &dyn SensorAuth,
        ) -> Result<FetchOutput, SensorError> {
            let schema = Arc::new(Schema::new(vec![Field::new(
                "item_id",
                DataType::Utf8,
                false,
            )]));
            let values: Vec<String> = (0..self.row_count)
                .map(|i| format!("item-{i:04}"))
                .collect();
            let values_ref: Vec<&str> = values.iter().map(|s| s.as_str()).collect();
            let batch = RecordBatch::try_new(schema, vec![Arc::new(StringArray::from(values_ref))])
                .expect("RecordBatch construction must not fail in stub");
            Ok(FetchOutput::new(vec![batch], false, false))
        }
    }

    /// Stub auth — ignored by all stub adapters above.
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
    /// short-circuiting with a `CredentialNotFound` error (SID-1 compliance).
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
    /// Mirrors `make_resolved` from `query_tool_sensor_errors_test.rs` and
    /// `bc_2_11_001_null_row_shape_test.rs`.
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

    /// Build `QueryToolParams` from a SQL query string (default limit = 25).
    fn query_params(sql: &str) -> QueryToolParams {
        serde_json::from_str(&serde_json::json!({"query": sql}).to_string())
            .expect("QueryToolParams JSON must deserialize")
    }

    /// Build `QueryToolParams` with an explicit tool-level row limit.
    ///
    /// The `limit` field is the TOOL-LEVEL limit (not a SQL LIMIT clause). The engine
    /// uses this to compute `is_truncated = total_rows > limit` after DataFusion execution.
    fn query_params_with_limit(sql: &str, limit: u32) -> QueryToolParams {
        serde_json::from_str(&serde_json::json!({"query": sql, "limit": limit}).to_string())
            .expect("QueryToolParams JSON must deserialize")
    }

    /// Extract `structured_content` from a `CallToolResult` as a `serde_json::Value`.
    fn envelope_json(result: rmcp::model::CallToolResult) -> serde_json::Value {
        result
            .structured_content
            .expect("query must return structured_content (not an error path)")
    }

    /// Build a `PrismServer` with an `AlwaysFailsAdapter` for `claroty_organization_acl_policies`.
    ///
    /// Used for OBS-1 wire-shape tests. The adapter returns `SensorError::HttpError` on every
    /// `fetch()` call, driving the `AllTargetsFailed` Err arm in `materialize_single_external_target`.
    ///
    /// `sensor_id = "claroty"` (no underscore), `table_name = "organization_acl_policies"` →
    /// full DataFusion table name = "claroty_organization_acl_policies".
    fn make_server_with_failing_claroty_adapter() -> PrismServer {
        use prism_core::column::ColumnType;

        let sensor_id = "claroty";
        let table_name = "organization_acl_policies";
        let org = "acme";

        let columns = vec![ColumnSpec::new("item_id", ColumnType::String, None, vec![])];

        // Deterministic OrgId (sentinel byte 0xb1 encodes OBS-1 for traceability).
        let org_id = OrgId::from_uuid(uuid::Uuid::from_bytes([
            0x01, 0xb1, 0xb1, 0xb1, 0x0e, 0x1a, 0x7a, 0x8b, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0xb1,
        ]));

        let (key, resolved) = make_resolved(sensor_id, table_name, columns, org);
        let mut resolved_map = HashMap::new();
        resolved_map.insert(key, resolved.clone());

        let registry = Arc::new(TableRegistry::new());
        registry
            .register_sensor(&resolved.spec)
            .expect("register_sensor must not fail in fixture");

        let adapter: Arc<dyn SensorAdapter> = Arc::new(AlwaysFailsAdapter {
            sensor_id: SensorId::new(sensor_id),
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

        PrismServer::new().with_query_engine(Arc::new(engine))
    }

    /// Build a `PrismServer` with a `ReturnsNRowsAdapter` for a minimal stub table.
    ///
    /// Used for OBS-2 wire-shape tests (has_more always false). The adapter returns
    /// `row_count` rows so the engine can compute `is_truncated = total_rows > options.limit`.
    ///
    /// `sensor_id = "stub"` (no underscore), `table_name = "items"` →
    /// full DataFusion table name = "stub_items".
    fn make_server_with_n_rows_adapter(row_count: usize) -> PrismServer {
        use prism_core::column::ColumnType;

        let sensor_id = "stub";
        let table_name = "items";
        let org = "acme";

        let columns = vec![ColumnSpec::new("item_id", ColumnType::String, None, vec![])];

        // Deterministic OrgId (sentinel byte 0xb2 encodes OBS-2 for traceability).
        let org_id = OrgId::from_uuid(uuid::Uuid::from_bytes([
            0x01, 0xb2, 0xb2, 0xb2, 0x0e, 0x1a, 0x7a, 0x8b, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0xb2,
        ]));

        let (key, resolved) = make_resolved(sensor_id, table_name, columns, org);
        let mut resolved_map = HashMap::new();
        resolved_map.insert(key, resolved.clone());

        let registry = Arc::new(TableRegistry::new());
        registry
            .register_sensor(&resolved.spec)
            .expect("register_sensor must not fail in fixture");

        let adapter: Arc<dyn SensorAdapter> = Arc::new(ReturnsNRowsAdapter {
            sensor_id: SensorId::new(sensor_id),
            row_count,
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

        PrismServer::new().with_query_engine(Arc::new(engine))
    }

    // =========================================================================
    // TEST B: OBS-1 wire-shape — _meta.data_source must not be ["unknown"]
    // =========================================================================

    /// BC-2.09.008 OBS-1 (wire-shape): `_meta.data_source` must reflect the sensor even
    /// when all fan-out targets fail.
    ///
    /// # What this test covers
    ///
    /// Issues a `query` for `claroty_organization_acl_policies` against an
    /// `AlwaysFailsAdapter` (HTTP 503, empty body). `fan_out()` returns
    /// `Err(AllTargetsFailed)`, driving the Err arm in `materialize_single_external_target`.
    /// Serializes the `CallToolResult` to JSON and asserts `_meta.data_source == ["claroty"]`.
    ///
    /// # Pre-fix failure (OBS-1)
    ///
    /// `sensors_queried.insert()` fires only in the `Ok(fan_result)` arm.
    /// On the Err path, `sensors_queried` stays empty → `server.rs` falls back to
    /// `sensor_ids = ["unknown"]` → wire JSON has `"data_source": ["unknown"]`.
    ///
    /// # Red Gate
    ///
    /// FAILS today: `_meta.data_source` is `["unknown"]`, not `["claroty"]`.
    /// PASSES after fix adds `sensors_queried.insert(target.sensor_id.to_string())`
    /// to the `Err(AllTargetsFailed)` arm in `materialize_single_external_target`.
    ///
    /// BC-2.09.008 | DEFECT-LIVE-ENVELOPE-OBS-001 OBS-1
    #[tokio::test]
    async fn test_BC_2_09_008_OBS_1_wire_data_source_reflects_sensor_on_all_targets_failed() {
        let server = make_server_with_failing_claroty_adapter();

        let result = server
            .query(Parameters(query_params(
                "SELECT * FROM claroty_organization_acl_policies",
            )))
            .await
            .expect(
                "query MUST return Ok even when all targets fail; sensor failures propagate \
                 to sensor_errors, not to is_error=true (BC-2.01.010 partial-failure)",
            );

        // Serialize to wire bytes — SID-2 / CLAUDE.md wire-shape discipline.
        let sc = envelope_json(result);
        let wire = serde_json::to_string(&sc).expect("structured_content must serialize to JSON");

        // ─── POSITIVE ASSERTION: data_source must contain "claroty" ───────────
        let data_source = &sc["_meta"]["data_source"];
        let contains_claroty = data_source
            .as_array()
            .is_some_and(|arr| arr.iter().any(|s| s.as_str() == Some("claroty")));
        assert!(
            contains_claroty,
            "OBS-1 (BC-2.09.008) FAIL: _meta.data_source must contain \"claroty\" even when \
             all claroty fan-out targets fail. \
             Root cause: sensors_queried.insert() fires only in the Ok(fan_result) arm of \
             materialize_single_external_target; the Err(AllTargetsFailed) arm leaves \
             sensors_queried empty → server.rs falls back to [\"unknown\"]. \
             Fix: add sensors_queried.insert(target.sensor_id.to_string()) to the Err arm. \
             Got data_source: {data_source}. Full wire: {wire}"
        );

        // ─── NEGATIVE ASSERTION: data_source MUST NOT be ["unknown"] ──────────
        let contains_unknown = data_source
            .as_array()
            .is_some_and(|arr| arr.iter().any(|s| s.as_str() == Some("unknown")));
        assert!(
            !contains_unknown,
            "OBS-1 (BC-2.09.008) FAIL (negative gate): _meta.data_source MUST NOT contain \
             \"unknown\" when the queried sensor is \"claroty\". \
             [\"unknown\"] is the pre-fix fallback for an empty sensors_queried set. \
             Got data_source: {data_source}. Full wire: {wire}"
        );
    }

    // =========================================================================
    // TEST C: OBS-2 wire-shape (truncated) — has_more must always be false
    // =========================================================================

    /// BC-2.09.008 OBS-2 (wire-shape, truncated path): `_meta.has_more` MUST always be `false`.
    ///
    /// # What this test covers
    ///
    /// The adapter returns 10 rows. The tool-level limit is 3 (`QueryToolParams.limit = 3`).
    /// The engine computes `total_rows(10) > limit(3) → is_truncated = true`. Truncation
    /// is signaled via `results.is_truncated = true` + `results.total_available = 10`.
    /// However, `_meta.has_more` MUST remain `false` (ADR-060 §D8.7: OFFSET not in grammar,
    /// cursor pagination unsupported).
    ///
    /// # Pre-fix failure (OBS-2)
    ///
    /// `server.rs` passes `result.is_truncated` directly as the `has_more` argument to
    /// `SafetyEnvelopeBuilder::wrap()`. When `is_truncated = true`, `has_more = true` —
    /// a direct violation of ADR-060 §D8.7.
    ///
    /// # Red Gate
    ///
    /// FAILS today: `_meta.has_more = true` (= `is_truncated`).
    /// PASSES after fix hard-wires `has_more = false` in the `server.rs` query handler.
    ///
    /// BC-2.09.008 | ADR-060 §D8.7 | DEFECT-LIVE-ENVELOPE-OBS-001 OBS-2
    #[tokio::test]
    async fn test_BC_2_09_008_OBS_2_wire_has_more_always_false_when_truncated() {
        // 10 rows from adapter, tool limit = 3 → total_rows(10) > limit(3) → is_truncated = true.
        // Pre-fix: has_more = is_truncated = true (BUG).
        // Post-fix: has_more = false always.
        let server = make_server_with_n_rows_adapter(10);

        let result = server
            .query(Parameters(query_params_with_limit(
                // No SQL LIMIT clause — the tool-level limit (3) is applied by the engine
                // AFTER DataFusion returns all 10 rows from the MemTable. This ensures
                // total_rows(10) > options.limit(3) → is_truncated = true.
                "SELECT * FROM stub_items",
                3,
            )))
            .await
            .expect("query must return Ok for a successful adapter");

        let sc = envelope_json(result);
        let wire = serde_json::to_string(&sc).expect("structured_content must serialize to JSON");

        // ─── PRIMARY ASSERTION: has_more MUST be false ────────────────────────
        let has_more = sc["_meta"]["has_more"].as_bool();
        assert_eq!(
            has_more,
            Some(false),
            "OBS-2 (BC-2.09.008 + ADR-060 §D8.7) FAIL: _meta.has_more MUST always be false; \
             truncation is signaled via results.is_truncated only (OFFSET not in grammar). \
             Root cause: server.rs passes result.is_truncated directly as has_more to \
             SafetyEnvelopeBuilder::wrap(). With limit=3 and 10 rows, is_truncated=true → \
             has_more=true pre-fix. Fix: always pass false as has_more in server.rs. \
             Got has_more: {has_more:?}. Full wire: {wire}"
        );

        // ─── SECONDARY ASSERTION: next_cursor MUST be present AND null ──────────
        // L-4 null-vs-absent guard: serde_json Value::Index returns Value::Null for BOTH
        // a missing key AND an explicit null. We must assert the key is PRESENT (not absent)
        // before asserting its value is null — absence would be a contract violation because
        // output_schema lists next_cursor in `required` (BC-2.09.008 v1.5).
        assert!(
            sc["_meta"].get("next_cursor").is_some(),
            "BC-2.09.008 v1.5: _meta.next_cursor key MUST be PRESENT and null, not absent \
             (output_schema lists it in `required`). A regression to key-omission would pass \
             the is_null() check silently. Full wire: {wire}"
        );
        let next_cursor = &sc["_meta"]["next_cursor"];
        assert!(
            next_cursor.is_null(),
            "OBS-2 (BC-2.09.008): _meta.next_cursor MUST be null (no cursor pagination; \
             ADR-060 §D8.7: OFFSET not in PrismQL grammar). Got: {next_cursor}"
        );

        // ─── TERTIARY ASSERTION: results.is_truncated MUST be true ────────────
        // Truncation IS correctly signaled via results.is_truncated (not has_more).
        let is_truncated = sc["results"]["is_truncated"].as_bool();
        assert_eq!(
            is_truncated,
            Some(true),
            "BC-2.09.008: results.is_truncated MUST be true when 10 rows exceed limit=3. \
             This is the canonical truncation signal (not has_more). Got: {is_truncated:?}. \
             Full wire: {wire}"
        );
    }

    // =========================================================================
    // TEST D: OBS-2 regression lock (non-truncated) — has_more already false
    // =========================================================================

    /// BC-2.09.008 OBS-2 (wire-shape, non-truncated path): `_meta.has_more` must be `false`
    /// when the result is NOT truncated (regression lock).
    ///
    /// # What this test covers
    ///
    /// The adapter returns 3 rows. Default tool limit (no explicit `limit` in
    /// `QueryToolParams`) = 25 via `map_or(25, ...)` in `server.rs`. The engine computes
    /// `total_rows(3) ≤ limit(25) → is_truncated = false`. Since `is_truncated = false`,
    /// `has_more = is_truncated = false` — this is already correct today.
    ///
    /// Included as a regression lock: must still pass after the OBS-2 fix.
    ///
    /// # Red Gate status
    ///
    /// MAY already pass today (has_more is false when not truncated). The Red Gate for
    /// OBS-2 is TEST C (truncated path). This test is a regression assertion only.
    ///
    /// BC-2.09.008 | ADR-060 §D8.7 | DEFECT-LIVE-ENVELOPE-OBS-001 OBS-2
    #[tokio::test]
    async fn test_BC_2_09_008_OBS_2_wire_has_more_false_when_not_truncated() {
        // 3 rows from adapter, default limit = 25 → total_rows(3) ≤ 25 → is_truncated = false.
        let server = make_server_with_n_rows_adapter(3);

        let result = server
            .query(Parameters(query_params(
                "SELECT * FROM stub_items", // default limit 25 > 3 rows → not truncated
            )))
            .await
            .expect("query must return Ok for a successful adapter");

        let sc = envelope_json(result);
        let wire = serde_json::to_string(&sc).expect("structured_content must serialize to JSON");

        // has_more MUST be false (regression lock — this was correct pre-fix too).
        let has_more = sc["_meta"]["has_more"].as_bool();
        assert_eq!(
            has_more,
            Some(false),
            "BC-2.09.008 OBS-2 REGRESSION: _meta.has_more must be false when result is not \
             truncated (3 rows ≤ 25 limit). Got: {has_more:?}. Full wire: {wire}"
        );

        // next_cursor MUST be present AND null (regression lock).
        // L-4 null-vs-absent guard: key presence asserted separately before value check.
        assert!(
            sc["_meta"].get("next_cursor").is_some(),
            "BC-2.09.008 v1.5 REGRESSION: _meta.next_cursor key MUST be PRESENT and null, \
             not absent (output_schema lists it in `required`). Full wire: {wire}"
        );
        let next_cursor = &sc["_meta"]["next_cursor"];
        assert!(
            next_cursor.is_null(),
            "BC-2.09.008 OBS-2 REGRESSION: _meta.next_cursor must be null. \
             Got: {next_cursor}"
        );

        // results.is_truncated MUST be false (regression lock).
        let is_truncated = sc["results"]["is_truncated"].as_bool();
        assert_eq!(
            is_truncated,
            Some(false),
            "BC-2.09.008 OBS-2 REGRESSION: results.is_truncated must be false \
             (3 rows ≤ 25 limit). Got: {is_truncated:?}. Full wire: {wire}"
        );
    }
}
