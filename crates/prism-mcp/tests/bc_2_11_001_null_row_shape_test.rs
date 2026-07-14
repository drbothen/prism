//! RED gate tests for DEFECT-MCP-ROWSHAPE-NULLS-001 — DEFECT 1: NULL columns omitted
//! from row JSON objects ([C3]/[H20]).
//!
//! Root cause: `arrow_json::writer::WriterBuilder::new()` at `server.rs` lines 1950-1951
//! uses `explicit_nulls=false` (default), causing NULL-valued cells to be OMITTED
//! from row JSON objects instead of appearing as JSON `null`.
//!
//! Spec authority:
//! - BC-2.11.001 EC-11-079 §Postconditions "Row-shape null-not-absent" bullet
//!   (null-not-absent postcondition codified at v1.16 as EC-11-068; renumbered EC-11-079
//!   at v1.20 per SR-006)
//! - EC-11-079: every row must contain all schema keys; NULL cells → `{"sensor_ip":null}`
//!   not absent
//! - Canonical test vector (BC-2.11.001 §Test Vectors):
//!   `SELECT severity, sensor_ip FROM crowdstrike_alerts` where some rows have
//!   `sensor_ip=NULL` → every row must have both keys; NULL rows serialize as
//!   `{"severity":"...","sensor_ip":null}`.
//!
//! # Red Gate test catalogue
//!
//! | Test | BC clause | Originally failed because (Red Gate history) |
//! |---|---|---|
//! | `test_BC_2_11_001_EC_11_079_null_column_value_serialized_as_json_null_not_absent` | EC-11-079 | `WriterBuilder::new()` omitted NULL key from row JSON |
//! | `test_BC_2_11_001_EC_11_079_every_row_contains_all_schema_column_keys` | EC-11-079 invariant | NULL row was missing `sensor_ip` key |
//! | `test_BC_2_11_001_canonical_test_vector_select_severity_sensor_ip_null_rows` | BC-2.11.001 EC-11-079 canonical test vector | Row 1 (`severity=medium, sensor_ip=NULL`) was missing `sensor_ip` key |
//!
//! Red Gate: these tests originally failed against `WriterBuilder::new()` default (nulls omitted).
//! They pass with `.with_explicit_nulls(true)` added to `WriterBuilder` in `server.rs` (this branch) — now locking the invariant.

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, sync::Arc};

    use arrow::{
        array::{Float64Array, StringArray},
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
        InfusionField, InfusionRegistry, InfusionSpec, InfusionType,
    };
    use rmcp::handler::server::wrapper::Parameters;

    // =========================================================================
    // Stub types
    // =========================================================================

    /// Stub auth token — ignored by `ReturnsNullRowsAdapter::fetch`.
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
    /// Required so `fan_out()` reaches the adapter boundary rather than short-circuiting
    /// with a `CredentialNotFound` error. The stub auth is ignored by
    /// `ReturnsNullRowsAdapter::fetch`. Pattern matches `AlwaysSucceedsCreds` from
    /// `normalized_pql.rs` (SID-1 compliance).
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

    /// Sensor adapter that returns 3 RecordBatch rows with a nullable `sensor_ip` column.
    ///
    /// Canonical test vector from BC-2.11.001 EC-11-079:
    /// - Row 0: `severity="high",   sensor_ip=Some("10.0.0.1")`
    /// - Row 1: `severity="medium", sensor_ip=None`  ← NULL — triggers the defect
    /// - Row 2: `severity="low",    sensor_ip=Some("10.0.0.3")`
    ///
    /// The RecordBatch schema declares `sensor_ip` with `nullable=true` so Arrow propagates
    /// the NULL into the serialized output. The defect causes `WriterBuilder::new()` to OMIT
    /// the `sensor_ip` key for Row 1 rather than serializing it as JSON `null`.
    struct ReturnsNullRowsAdapter {
        sensor_id: SensorId,
    }

    #[async_trait]
    impl SensorAdapter for ReturnsNullRowsAdapter {
        fn sensor_type(&self) -> SensorId {
            self.sensor_id.clone()
        }

        fn sensor_name(&self) -> &'static str {
            "returns-null-rows-stub"
        }

        async fn fetch(
            &self,
            _spec: &SensorAdapterSpec,
            _params: &SensorQueryParams,
            _auth: &dyn SensorAuth,
        ) -> Result<Vec<RecordBatch>, SensorError> {
            // Schema: severity (non-nullable String), sensor_ip (nullable String).
            // `nullable=true` on `sensor_ip` is the key: Arrow will produce a NULL value
            // in the serialized JSON, which `WriterBuilder::new()` omits (the defect).
            let schema = Arc::new(Schema::new(vec![
                Field::new("severity", DataType::Utf8, false),
                Field::new("sensor_ip", DataType::Utf8, true),
            ]));
            let batch = RecordBatch::try_new(
                schema,
                vec![
                    Arc::new(StringArray::from(vec!["high", "medium", "low"])),
                    // Row 1 has None — the canonical NULL test value from BC-2.11.001.
                    Arc::new(StringArray::from(vec![
                        Some("10.0.0.1"),
                        None,
                        Some("10.0.0.3"),
                    ])),
                ],
            )
            .expect("RecordBatch construction must not fail in stub");
            Ok(vec![batch])
        }
    }

    // =========================================================================
    // Fixture helpers
    // =========================================================================

    /// Build a minimal `ResolvedSensorSpec` for a given sensor/table/org combination.
    ///
    /// Mirrors `make_resolved` from `normalized_pql.rs` so the fixture wiring is identical.
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

    /// Build a `PrismServer` wired with `ReturnsNullRowsAdapter` for `crowdstrike_alerts`.
    ///
    /// Structurally identical to `make_server_with_failing_adapter` in `normalized_pql.rs`
    /// except it uses `ReturnsNullRowsAdapter` (succeeds + returns NULL rows) instead of
    /// `AlwaysFailsAdapter`. The `AlwaysSucceedsCreds` resolver is wired so `fan_out()`
    /// reaches the adapter boundary (SID-1: unit test without live DTU).
    ///
    /// `resolve_org_id` Path 2 fallback: without an OrgRegistry, the query engine uses
    /// the first registered adapter's `OrgId` for the sensor — so the `org_id` here must
    /// match the registration key in `AdapterRegistry`.
    fn make_server_with_returning_null_adapter() -> PrismServer {
        use prism_core::column::ColumnType;

        // sensor_id="crowdstrike" + table_name="alerts" → full DataFusion table name
        // = "crowdstrike_alerts" (formed by TableRegistry as "{sensor_id}_{table_name}").
        // This is the canonical test vector table from BC-2.11.001 EC-11-079.
        let sensor_id_str = "crowdstrike";
        let table_name = "alerts";
        let org = "acme";

        let columns = vec![
            ColumnSpec::new("severity", ColumnType::String, None, vec![]),
            ColumnSpec::new("sensor_ip", ColumnType::String, None, vec![]),
        ];

        // Deterministic OrgId (EC-11-079 sentinel byte: 0x68 = 'h' for "null row sHape").
        let org_id = OrgId::from_uuid(uuid::Uuid::from_bytes([
            0x01, 0x9f, 0x3a, 0x71, 0x5c, 0x6d, 0x7a, 0x8b, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x68,
        ]));

        let (key, resolved) = make_resolved(sensor_id_str, table_name, columns, org);
        let mut resolved_map = HashMap::new();
        resolved_map.insert(key, resolved.clone());

        let registry = Arc::new(TableRegistry::new());
        registry
            .register_sensor(&resolved.spec)
            .expect("register_sensor must not fail in fixture");

        let sensor_id_typed = SensorId::new(sensor_id_str);
        let returning_null_adapter: Arc<dyn SensorAdapter> = Arc::new(ReturnsNullRowsAdapter {
            sensor_id: sensor_id_typed,
        });
        let mut adapter_registry = AdapterRegistry::new();
        adapter_registry.register(org_id, returning_null_adapter);

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

    /// Build `QueryToolParams` from a JSON string.
    ///
    /// `QueryToolParams` is `#[non_exhaustive]` — struct-literal construction is forbidden
    /// outside the defining crate (integration tests are separate crates). JSON
    /// deserialization is the correct pattern; mirrors usage in `normalized_pql.rs`.
    fn query_params(sql: &str) -> QueryToolParams {
        serde_json::from_str(&serde_json::json!({"query": sql}).to_string())
            .expect("QueryToolParams JSON must deserialize")
    }

    /// Extract the `structured_content` JSON value from a `CallToolResult`.
    fn envelope_json(result: rmcp::model::CallToolResult) -> serde_json::Value {
        result
            .structured_content
            .expect("query must return structured_content (not an error path)")
    }

    // =========================================================================
    // DEFECT 1 tests — Red Gate: originally failed against pre-fix code; now pass
    // =========================================================================

    /// BC-2.11.001 EC-11-079: a row with a NULL-valued column must serialize
    /// the column key with JSON `null`, NOT omit the key entirely.
    ///
    /// Red Gate: originally failed — `WriterBuilder::new()` used `explicit_nulls=false`
    /// (default). The NULL row had only `"severity"` key; `"sensor_ip"` key was absent
    /// from the JSON object.
    ///
    /// Passes now: `.with_explicit_nulls(true)` added to `WriterBuilder` in `server.rs`
    /// (this branch) — locking EC-11-079 against regression.
    #[tokio::test]
    async fn test_BC_2_11_001_EC_11_079_null_column_value_serialized_as_json_null_not_absent() {
        let server = make_server_with_returning_null_adapter();
        let result = server
            .query(Parameters(query_params(
                "SELECT severity, sensor_ip FROM crowdstrike_alerts",
            )))
            .await
            .expect(
                "query must return Ok — NULL-valued rows are valid data, not a query-level error",
            );

        let v = envelope_json(result);
        let rows = v["results"]["rows"]
            .as_array()
            .expect("results.rows must be a JSON array");

        assert!(
            !rows.is_empty(),
            "expected at least one row from the stub adapter; got empty results"
        );

        // Find the NULL-sensor_ip row (Row 1: severity="medium", sensor_ip=NULL).
        // Under the bug: this row has NO "sensor_ip" key at all.
        // Under the fix: this row has "sensor_ip": null.
        let null_row = rows
            .iter()
            .find(|row| row.get("sensor_ip").is_none_or(|v| v.is_null()))
            .expect(
                "expected to find a row with NULL sensor_ip \
                 (Row 1: severity=medium, sensor_ip=NULL from ReturnsNullRowsAdapter)",
            );

        // EC-11-079: the key MUST be present (not absent) with JSON null as its value.
        assert!(
            null_row.get("sensor_ip").is_some(),
            "EC-11-079 VIOLATION: NULL-valued 'sensor_ip' column must appear as \
             `\"sensor_ip\": null` in the row JSON — the key must NOT be absent. \
             Current defect: WriterBuilder::new() uses explicit_nulls=false, which \
             omits the key. Fix: .with_explicit_nulls(true). Got row: {null_row}"
        );
        assert!(
            null_row["sensor_ip"].is_null(),
            "EC-11-079: 'sensor_ip' key is present but its value must be JSON null; \
             got: {}",
            null_row["sensor_ip"]
        );
    }

    /// BC-2.11.001 EC-11-079 invariant: EVERY row must contain ALL projected
    /// column keys, regardless of whether their values are NULL.
    ///
    /// Key-completeness invariant: the set of keys in each row object must equal the
    /// set of projected columns (`{severity, sensor_ip}`).
    ///
    /// Red Gate: originally failed — the NULL row (Row 1) had only `"severity"` key;
    /// `"sensor_ip"` was absent, violating the invariant.
    #[tokio::test]
    async fn test_BC_2_11_001_EC_11_079_every_row_contains_all_schema_column_keys() {
        let server = make_server_with_returning_null_adapter();
        let result = server
            .query(Parameters(query_params(
                "SELECT severity, sensor_ip FROM crowdstrike_alerts",
            )))
            .await
            .expect("query must return Ok");

        let v = envelope_json(result);
        let rows = v["results"]["rows"]
            .as_array()
            .expect("results.rows must be a JSON array");

        assert_eq!(
            rows.len(),
            3,
            "expected exactly 3 rows from the stub adapter"
        );

        // EC-11-079 invariant: every row must have BOTH projected keys.
        let required_keys = ["severity", "sensor_ip"];
        for (i, row) in rows.iter().enumerate() {
            for key in &required_keys {
                assert!(
                    row.get(*key).is_some(),
                    "EC-11-079 VIOLATION at row {i}: projected column '{key}' is ABSENT \
                     from row object. Every row must contain all schema column keys even \
                     when the value is NULL. Got row {i}: {row}"
                );
            }
        }
    }

    /// BC-2.11.001 EC-11-079 canonical test vector: `SELECT severity, sensor_ip FROM
    /// crowdstrike_alerts` where some rows have `sensor_ip=NULL`.
    ///
    /// Per the BC canonical test vector (§Test Vectors):
    /// - Non-null rows: `{"severity":"high","sensor_ip":"10.0.0.1"}`
    /// - NULL row: `{"severity":"medium","sensor_ip":null}` — both keys MUST be present
    ///
    /// Red Gate: originally failed — Row 1 (`severity=medium, sensor_ip=NULL`) serialized as
    /// `{"severity":"medium"}` — `"sensor_ip"` key was absent.
    #[tokio::test]
    async fn test_BC_2_11_001_canonical_test_vector_select_severity_sensor_ip_null_rows() {
        let server = make_server_with_returning_null_adapter();
        let result = server
            .query(Parameters(query_params(
                "SELECT severity, sensor_ip FROM crowdstrike_alerts",
            )))
            .await
            .expect("query must return Ok — canonical test vector is a valid query");

        let v = envelope_json(result);
        let rows = v["results"]["rows"]
            .as_array()
            .expect("results.rows must be a JSON array");

        assert_eq!(
            rows.len(),
            3,
            "canonical test vector: expected 3 rows; got {}",
            rows.len()
        );

        // Row 0: severity="high", sensor_ip="10.0.0.1" (non-null — should always work)
        assert_eq!(rows[0]["severity"], "high", "row 0 severity mismatch");
        assert_eq!(rows[0]["sensor_ip"], "10.0.0.1", "row 0 sensor_ip mismatch");

        // Row 1: severity="medium", sensor_ip=NULL — the canonical null test row.
        // BC-2.11.001 canonical test vector: MUST serialize as
        // `{"severity":"medium","sensor_ip":null}` — NOT `{"severity":"medium"}`.
        assert_eq!(rows[1]["severity"], "medium", "row 1 severity mismatch");
        assert!(
            rows[1].get("sensor_ip").is_some(),
            "BC-2.11.001 canonical test vector VIOLATION: NULL sensor_ip row (Row 1) \
             must have 'sensor_ip' key present with JSON null. Got row 1: {}",
            rows[1]
        );
        assert!(
            rows[1]["sensor_ip"].is_null(),
            "BC-2.11.001: sensor_ip in null row must be JSON null; got: {}",
            rows[1]["sensor_ip"]
        );

        // Row 2: severity="low", sensor_ip="10.0.0.3" (non-null — should always work)
        assert_eq!(rows[2]["severity"], "low", "row 2 severity mismatch");
        assert_eq!(rows[2]["sensor_ip"], "10.0.0.3", "row 2 sensor_ip mismatch");
    }

    // =========================================================================
    // AC (b) enrich-stage NULL lock — BC-2.11.001 EC-11-079 probe [H20]
    // =========================================================================

    /// Build an `InfusionRegistry` containing a single `NullSource`-backed UDF
    /// named `null_enrich_udf`.
    ///
    /// Uses `InfusionType::LocalLookup` with no source file config (`spec.source = None`) —
    /// `load_spec` takes the LocalLookup `else` branch in `infusion/mod.rs` and wires
    /// `Arc::new(NullSource)` directly. No file I/O occurs; no error-fallback path is
    /// involved. `NullSource::enrich_single` always returns `None`, so every enrichment
    /// call produces a NULL output column value.
    ///
    /// Stub seam (SID-1): `NullSource` is internal to `prism-spec-engine`, wired at
    /// `load_spec` time. No DTU clone, no filesystem source file, no external service.
    /// The `source_path` field is diagnostic metadata only — `load_spec` never reads it
    /// for file I/O. Passing a descriptive non-path value makes this cross-platform.
    /// The null-input guard in `InfusionAsyncUdf::invoke_async_with_args`
    /// is also exercised for rows where the input column itself is NULL (ADR-051 §D2
    /// null-input guard — input column NULL → output NULL, no source call, no E-INFUSE-014;
    /// invoke_async_with_args short-circuits before project_value()).
    fn make_null_infusion_registry() -> Arc<InfusionRegistry> {
        let registry = InfusionRegistry::new();
        // LocalLookup + source:None → load_spec's else-branch wires NullSource (always returns None).
        // No file I/O, no error-fallback seam. source_path is diagnostic metadata only.
        let spec = InfusionSpec::new(
            "null_enrich_spec",
            "Null enrichment stub — EC-11-079 AC-b regression lock",
            InfusionType::LocalLookup,
            vec![InfusionField::new(
                "null_enrich_udf", // per-field UDF name registered in DataFusion
                "sensor_ip",       // input_field (for describe hints)
                "string",          // input_type
                "string",          // output_type → DataType::Utf8 (nullable StringArray)
            )],
            // source_path is metadata only — load_spec never opens it. Descriptive
            // placeholder avoids platform-specific path assumptions (F-MCPRS-PRL10-OBS-002).
            "null-enrich-spec.infusion.toml",
        );
        registry.load_spec(spec).expect(
            "null_enrich_udf spec must load — LocalLookup+source:None takes direct NullSource path",
        );
        Arc::new(registry)
    }

    /// Build a `PrismServer` wired with `ReturnsNullRowsAdapter` for `crowdstrike_alerts`
    /// AND a `null_enrich_udf` infusion UDF backed by `NullSource`.
    ///
    /// Used by the AC-b test to exercise the end-to-end enrich-stage null serialization
    /// path: sensor fan-out → DataFusion CTE with `| enrich` → RecordBatch →
    /// `WriterBuilder::with_explicit_nulls(true)` → MCP `structured_content` envelope.
    ///
    /// `null_enrich_udf` uses `NullSource` (SID-1: no external service). Every call to
    /// `enrich_one_scalar` returns `None` (ADR-051 §D2 — NULL as partial-failure signal).
    /// For row 1 (sensor_ip=NULL), the `invoke_async_with_args` null-input guard
    /// (ADR-051 §D2 null-input guard — input NULL → output NULL, no source call, no E-INFUSE-014)
    /// fires before project_value(). Both paths produce NULL output →
    /// the enriched column is an all-NULL `StringArray`. EC-11-079 invariant: this column
    /// serializes with the key present and value `null` (not absent) under
    /// `WriterBuilder::with_explicit_nulls(true)`.
    fn make_server_with_enrich_null_udf() -> PrismServer {
        use prism_core::column::ColumnType;

        let sensor_id_str = "crowdstrike";
        let table_name = "alerts";
        let org = "acme";

        let columns = vec![
            ColumnSpec::new("severity", ColumnType::String, None, vec![]),
            ColumnSpec::new("sensor_ip", ColumnType::String, None, vec![]),
        ];

        // Same deterministic OrgId as `make_server_with_returning_null_adapter` —
        // EC-11-079 sentinel byte 0x68 = 'h' for "null row sHape".
        let org_id = OrgId::from_uuid(uuid::Uuid::from_bytes([
            0x01, 0x9f, 0x3a, 0x71, 0x5c, 0x6d, 0x7a, 0x8b, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x68,
        ]));

        let (key, resolved) = make_resolved(sensor_id_str, table_name, columns, org);
        let mut resolved_map = HashMap::new();
        resolved_map.insert(key, resolved.clone());

        let registry = Arc::new(TableRegistry::new());
        registry
            .register_sensor(&resolved.spec)
            .expect("register_sensor must not fail in fixture");

        let sensor_id_typed = SensorId::new(sensor_id_str);
        let returning_null_adapter: Arc<dyn SensorAdapter> = Arc::new(ReturnsNullRowsAdapter {
            sensor_id: sensor_id_typed,
        });
        let mut adapter_registry = AdapterRegistry::new();
        adapter_registry.register(org_id, returning_null_adapter);

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
        engine = engine.with_infusion_registry(make_null_infusion_registry());

        PrismServer::new().with_query_engine(Arc::new(engine))
    }

    // =========================================================================
    // Filter-mode EC-11-079 regression lock
    // =========================================================================

    /// BC-2.11.001 EC-11-079 filter-mode lock: in filter-mode queries
    /// (`crowdstrike_alerts | predicate`), NULL-valued column keys must appear as
    /// JSON `null` in every row — the invariant applies to all three query modes
    /// (SQL, pipe, filter).
    ///
    /// BC-2.11.001 §Postconditions states the invariant holds across ALL query
    /// modes. SQL + pipe are locked by the preceding tests (via the single
    /// `WriterBuilder::with_explicit_nulls(true)` chokepoint in server.rs). This test
    /// locks filter mode against a future regression where a second row-emit path for
    /// filter queries is introduced without the fix.
    ///
    /// PASSES on arrival (GREEN): the single `WriterBuilder::with_explicit_nulls(true)`
    /// serializer chokepoint in server.rs covers filter-mode results just as it covers
    /// SQL and pipe results. If it FAILS, a second row-emit path has been introduced for
    /// filter queries without `with_explicit_nulls(true)` — fix that path and do NOT
    /// weaken the assertion.
    ///
    /// # Query syntax
    ///
    /// Filter mode source reference format: `sensor_table_name | predicate` (underscore form,
    /// `SourceRefKind::Custom`). The dot-notation form (`sensor.table`) produces
    /// `SourceRefKind::External` which is rejected by the E-QUERY-037 availability gate
    /// (EC-11-067 / BC-2.11.001). Use underscore form so the gate sees a registered
    /// Custom source ref.
    /// Predicate `severity != "notexist_severity"` matches all 3 rows (none has
    /// severity="notexist_severity"). DataFusion materializes this as:
    ///   `SELECT * FROM crowdstrike_alerts WHERE severity != 'notexist_severity'`
    /// Returning all columns (severity, sensor_ip) for all 3 rows.
    #[tokio::test]
    async fn test_BC_2_11_001_EC_11_079_filter_mode_null_column_serialized_as_json_null_not_absent()
    {
        let server = make_server_with_returning_null_adapter();
        let result = server
            .query(Parameters(query_params(
                // Filter-mode query: "crowdstrike_alerts | predicate" (underscore form).
                // Source ref "crowdstrike_alerts" is SourceRefKind::Custom — matches the
                // registered table name directly. The dot-notation form "crowdstrike.alerts"
                // produces SourceRefKind::External which the E-QUERY-037 availability gate
                // rejects (EC-11-067). Use underscore form to pass the gate.
                // Predicate matches all 3 rows — none has severity="notexist_severity".
                "crowdstrike_alerts | severity != \"notexist_severity\"",
            )))
            .await
            .expect(
                "filter-mode query must return Ok — NULL-valued rows are valid data, \
                 not a query-level error; severity != 'notexist_severity' matches all 3 rows",
            );

        let v = envelope_json(result);
        let rows = v["results"]["rows"]
            .as_array()
            .expect("results.rows must be a JSON array");

        assert_eq!(
            rows.len(),
            3,
            "filter-mode: expected 3 rows from ReturnsNullRowsAdapter; got {}",
            rows.len()
        );

        // EC-11-079 invariant: every row must contain ALL projected column keys.
        // In filter mode, `SELECT * FROM crowdstrike_alerts WHERE ...` projects all columns.
        let required_keys = ["severity", "sensor_ip"];
        for (i, row) in rows.iter().enumerate() {
            for key in &required_keys {
                assert!(
                    row.get(*key).is_some(),
                    "EC-11-079 VIOLATION (filter mode) at row {i}: projected column '{key}' \
                     is ABSENT from row object. Filter-mode NULL-valued column keys must \
                     serialize as JSON null (key present), not be omitted. \
                     Got row {i}: {row}"
                );
            }
        }

        // Specifically assert the NULL sensor_ip row (Row 1: severity="medium") is correct.
        let null_row = rows
            .iter()
            .find(|row| row.get("severity").and_then(|v| v.as_str()) == Some("medium"))
            .expect("expected to find severity='medium' row (Row 1, sensor_ip=NULL)");
        assert!(
            null_row.get("sensor_ip").is_some(),
            "EC-11-079 filter-mode: NULL sensor_ip key must be PRESENT (not absent) for Row 1 \
             (severity=medium). Got row: {null_row}"
        );
        assert!(
            null_row["sensor_ip"].is_null(),
            "EC-11-079 filter-mode: sensor_ip key must be JSON null for Row 1; \
             got: {}",
            null_row["sensor_ip"]
        );
    }

    /// BC-2.11.001 EC-11-079 AC (b) — probe [H20]: in pipe-mode `| enrich` queries, enrichment
    /// column values that are NULL must serialize as JSON `null` (key present), NOT be omitted.
    ///
    /// This is the enrich-stage companion to the projected-nullable-column tests above (AC (a)).
    /// The [H20] probe targets the enrich-specific code path where the UDF produces NULL output
    /// for a row — either because the source returns `None` (ADR-051 §D2 — NULL is the
    /// correct partial-failure signal for a single enrichment row), or because the input
    /// column is itself NULL (ADR-051 §D2 null-input guard — input column NULL → output
    /// NULL, no source call, no E-INFUSE-014; invoke_async_with_args short-circuits
    /// before project_value()).
    ///
    /// PASSES: `WriterBuilder::with_explicit_nulls(true)` already routes all RecordBatch columns
    /// (including enrich-stage columns) through the fixed serializer. This test is a REGRESSION
    /// LOCK: if a future change introduces a second row-emit path with an unpatched WriterBuilder,
    /// this test will catch it.
    ///
    /// # Mechanism
    ///
    /// - `ReturnsNullRowsAdapter` produces 3 rows: severity=high/medium/low with
    ///   sensor_ip="10.0.0.1" / NULL / "10.0.0.3".
    /// - `null_enrich_udf` is backed by `NullSource` (always returns `None`) via
    ///   `InfusionRegistry::load_spec(LocalLookup, no source config)`.
    /// - Query: `FROM crowdstrike_alerts | enrich null_enrich_udf(sensor_ip)`
    ///   DataFusion CTE: `SELECT *, null_enrich_udf(sensor_ip) AS null_enrich_udf FROM crowdstrike_alerts`
    /// - Rows 0 and 2 (sensor_ip non-null): `enrich_one_scalar` calls NullSource → `None` → NULL.
    /// - Row 1 (sensor_ip=NULL): `invoke_async_with_args` null-input guard fires → NULL
    ///   (ADR-051 §D2 null-input guard — input NULL → output NULL, no source call, no E-INFUSE-014;
    ///   invoke_async_with_args short-circuits before project_value()).
    /// - All 3 rows: `null_enrich_udf` column is an all-NULL `StringArray` in the RecordBatch.
    /// - EC-11-079 invariant: every row must contain all projected keys (severity, sensor_ip,
    ///   null_enrich_udf) regardless of nullability.
    #[tokio::test]
    async fn test_BC_2_11_001_AC_b_enrich_stage_null_udf_result_serialized_as_json_null_not_absent()
    {
        let server = make_server_with_enrich_null_udf();
        let result = server
            .query(Parameters(query_params(
                "FROM crowdstrike_alerts | enrich null_enrich_udf(sensor_ip)",
            )))
            .await
            .expect(
                "query must return Ok — pipe | enrich with NullSource is valid; \
                 NullSource returning None produces NULL column values, not a query error",
            );

        let v = envelope_json(result);
        let rows = v["results"]["rows"]
            .as_array()
            .expect("results.rows must be a JSON array");

        assert_eq!(
            rows.len(),
            3,
            "expected exactly 3 rows from ReturnsNullRowsAdapter; got {}",
            rows.len()
        );

        // EC-11-079 invariant: every row must contain ALL projected column keys.
        // For a pipe | enrich query, the columns are: severity, sensor_ip, null_enrich_udf.
        // `null_enrich_udf` is an all-NULL StringArray — the bug omits its key; the fix
        // serializes it as `"null_enrich_udf": null`.
        let required_keys = ["severity", "sensor_ip", "null_enrich_udf"];
        for (i, row) in rows.iter().enumerate() {
            for key in &required_keys {
                assert!(
                    row.get(*key).is_some(),
                    "EC-11-079 / BC-2.11.001 AC-b VIOLATION at row {i}: \
                     projected column '{key}' is ABSENT from row object. \
                     Enrich-stage NULL column values must serialize as JSON null (key present), \
                     not be omitted. \
                     Got row {i}: {row}"
                );
            }
        }

        // AC-b specific: `null_enrich_udf` must be JSON null in every row.
        // - NullSource returns None for all source calls (rows 0 and 2).
        // - Row 1 (sensor_ip=NULL): null-input guard fires → NULL (ADR-051 §D2 null-input guard —
        //   input NULL → output NULL, no source call, no E-INFUSE-014;
        //   invoke_async_with_args short-circuits before project_value()).
        // Before the WriterBuilder fix: key ABSENT. After fix: key present, value JSON null.
        for (i, row) in rows.iter().enumerate() {
            assert!(
                row["null_enrich_udf"].is_null(),
                "BC-2.11.001 AC-b: 'null_enrich_udf' must be JSON null for row {i} \
                 (NullSource returns None per ADR-051 §D2 — NULL as partial-failure signal; \
                  row 1 null-input guard per ADR-051 §D2 null-input guard — \
                  input NULL → output NULL, no source call). \
                 Got: {}",
                row["null_enrich_udf"]
            );
        }

        // Regression guard for row 1: the null-input guard (ADR-051 §D2 null-input guard —
        // sensor_ip=NULL → output NULL, no source call; invoke_async_with_args) fires
        // when sensor_ip=NULL. Verify AC (a) sensor_ip null-key-present invariant is still intact
        // — the enrich-stage regression lock must not accidentally break the projected-nullable
        // column invariant for the same row.
        assert!(
            rows[1].get("sensor_ip").is_some(),
            "EC-11-079 AC (a) regression: row 1 'sensor_ip' key must still be present \
             (AC (b) enrich-stage fix must not disturb AC (a) projected-column fix)"
        );
        assert!(
            rows[1]["sensor_ip"].is_null(),
            "EC-11-079 AC (a) regression: row 1 'sensor_ip' must still be JSON null; \
             got: {}",
            rows[1]["sensor_ip"]
        );
    }

    // =========================================================================
    // EC-11-081 — arrow-json v58 non-finite Float64 → JSON null boundary lock
    // =========================================================================

    /// Sensor adapter returning 5 rows with a Float64 column covering
    /// the full non-finite Float64 boundary: [1.5, NaN, +Inf, -Inf, Arrow-null].
    ///
    /// Used by `test_BC_2_11_001_EC_11_081_nonfinite_float_serializes_as_json_null`
    /// to exercise arrow-json's hardcoded non-finite-to-null encoder through
    /// the production MCP serialization path (F-MCPRS-PRL16-LOW-001).
    struct ReturnsNonfiniteFloatRowsAdapter {
        sensor_id: SensorId,
    }

    #[async_trait]
    impl SensorAdapter for ReturnsNonfiniteFloatRowsAdapter {
        fn sensor_type(&self) -> SensorId {
            self.sensor_id.clone()
        }

        fn sensor_name(&self) -> &'static str {
            "returns-nonfinite-float-rows-stub"
        }

        async fn fetch(
            &self,
            _spec: &SensorAdapterSpec,
            _params: &SensorQueryParams,
            _auth: &dyn SensorAuth,
        ) -> Result<Vec<RecordBatch>, SensorError> {
            // Schema: float_col (nullable Float64).
            // `nullable=true` allows Arrow-null (row 4) to appear in the validity bitmap.
            let schema = Arc::new(Schema::new(vec![Field::new(
                "float_col",
                DataType::Float64,
                true,
            )]));

            // 5 rows covering the full non-finite boundary per BC-2.11.001 EC-11-081:
            // Row 0: 1.5        — finite, expected JSON Number 1.5
            // Row 1: NaN        — non-finite, expected JSON null (arrow-json v58 hardcoded encoder)
            // Row 2: +Inf       — non-finite, expected JSON null
            // Row 3: -Inf       — non-finite, expected JSON null
            // Row 4: Arrow-null — validity bit unset; JSON null via with_explicit_nulls(true)
            let float_data = Float64Array::from(vec![
                Some(1.5_f64),
                Some(f64::NAN),
                Some(f64::INFINITY),
                Some(f64::NEG_INFINITY),
                None,
            ]);

            let batch = RecordBatch::try_new(schema, vec![Arc::new(float_data)])
                .expect("RecordBatch construction must not fail in stub");
            Ok(vec![batch])
        }
    }

    /// Build a `PrismServer` wired with `ReturnsNonfiniteFloatRowsAdapter`
    /// for the `floattest_floats` DataFusion table.
    ///
    /// Uses sensor_id=`floattest` / table_name=`floats` → DataFusion table name
    /// `floattest_floats`, distinct from `crowdstrike_alerts` to avoid schema
    /// collision with the EC-11-079 test fixtures. `floattest` (no underscores) is
    /// required because `sensor_id_from_table_name` splits on the first underscore;
    /// a sensor_id like `float_test` would yield prefix `float` and produce E-QUERY-036.
    /// Follows the same wiring pattern as `make_server_with_returning_null_adapter`
    /// (AlwaysSucceedsCreds + fresh TableRegistry + fresh QueryEngine).
    fn make_server_with_nonfinite_float_rows_adapter() -> PrismServer {
        use prism_core::column::ColumnType;

        // sensor_id="floattest" + table_name="floats" → DataFusion table "floattest_floats".
        // (TableRegistry forms "{sensor_id}_{table_name}".)
        // NOTE: sensor_id MUST NOT contain underscores — `sensor_id_from_table_name` splits
        // the source-ref string on the first underscore to extract the sensor prefix.
        // "float_test_floats" would yield prefix "float" (no adapter registered) → E-QUERY-036.
        // "floattest_floats" yields prefix "floattest" → correct adapter lookup.
        // Distinct from crowdstrike_alerts to prevent schema cross-contamination with EC-11-079.
        let sensor_id_str = "floattest";
        let table_name = "floats";
        let org = "acme";

        let columns = vec![ColumnSpec::new(
            "float_col",
            ColumnType::Float,
            None,
            vec![],
        )];

        // EC-11-081 sentinel byte: 0x81 — matches EC number for audit cross-reference.
        let org_id = OrgId::from_uuid(uuid::Uuid::from_bytes([
            0x01, 0x9f, 0x3a, 0x71, 0x5c, 0x6d, 0x7a, 0x8b, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x81,
        ]));

        let (key, resolved) = make_resolved(sensor_id_str, table_name, columns, org);
        let mut resolved_map = HashMap::new();
        resolved_map.insert(key, resolved.clone());

        let registry = Arc::new(TableRegistry::new());
        registry
            .register_sensor(&resolved.spec)
            .expect("register_sensor must not fail in fixture");

        let sensor_id_typed = SensorId::new(sensor_id_str);
        let float_adapter: Arc<dyn SensorAdapter> = Arc::new(ReturnsNonfiniteFloatRowsAdapter {
            sensor_id: sensor_id_typed,
        });
        let mut adapter_registry = AdapterRegistry::new();
        adapter_registry.register(org_id, float_adapter);

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

    /// BC-2.11.001 v1.22 EC-11-081: non-finite Float64 values (NaN, +Inf, -Inf) and
    /// Arrow-null Float64 values ALL serialize as JSON `null` at the MCP boundary.
    ///
    /// # Spec authority
    ///
    /// - BC-2.11.001 v1.22 EC-11-081 — "NaN/±Inf → JSON null at the MCP boundary,
    ///   key present, documented artifact (Option A ratified)"
    /// - F-MCPRS-PRL16-LOW-001 — finding source: non-finite Float64 probe confirmed
    ///   arrow-json v58 hardcodes NaN/±Inf as null; BC must document this behavior
    ///
    /// # Boundary lock
    ///
    /// Arrow's JSON encoder (`arrow_json::writer::JsonWriter`) v58 hardcodes non-finite
    /// Float64 values (NaN, +Inf, -Inf) as JSON `null` because JSON (RFC 8259) does not
    /// support non-finite numbers. This is NOT configurable — there is no flag to emit
    /// `"NaN"` or `"Infinity"` strings instead.
    ///
    /// | Input value | JSON output              |
    /// |-------------|--------------------------|
    /// | 1.5         | Number `1.5` (unchanged) |
    /// | NaN         | `null` (hardcoded)       |
    /// | +Infinity   | `null` (hardcoded)       |
    /// | -Infinity   | `null` (hardcoded)       |
    /// | Arrow-null  | `null` (explicit_nulls)  |
    ///
    /// **Regression signal:** if a future arrow-json bump changes this behavior (e.g.,
    /// starts emitting `"NaN"` or `"Infinity"` strings), this test FAILS. That failure
    /// is the correct signal — BC-2.11.001 EC-11-081 Option A must be re-adjudicated
    /// with the new arrow-json version before the upgrade can be accepted.
    ///
    /// # EC-11-079 invariant compliance
    ///
    /// All 5 rows must contain the `float_col` key regardless of whether the null
    /// originated from non-finite encoding or from an Arrow validity-bitmap null.
    #[tokio::test]
    async fn test_BC_2_11_001_EC_11_081_nonfinite_float_serializes_as_json_null() {
        let server = make_server_with_nonfinite_float_rows_adapter();
        let result = server
            .query(Parameters(query_params(
                "SELECT float_col FROM floattest_floats",
            )))
            .await
            .expect(
                "query must return Ok — non-finite Float64 values (NaN/Inf) are valid Arrow \
                 array values; DataFusion does not error on them in a simple SELECT",
            );

        let v = envelope_json(result);
        let rows = v["results"]["rows"]
            .as_array()
            .expect("results.rows must be a JSON array");

        assert_eq!(
            rows.len(),
            5,
            "expected 5 rows from ReturnsNonfiniteFloatRowsAdapter \
             (rows: 1.5, NaN, +Inf, -Inf, Arrow-null); got {}",
            rows.len()
        );

        // (a) EC-11-079 invariant: EVERY row must contain the `float_col` key,
        // regardless of whether the value is null (Arrow-null or non-finite-as-null).
        for (i, row) in rows.iter().enumerate() {
            assert!(
                row.get("float_col").is_some(),
                "EC-11-079 / EC-11-081 VIOLATION at row {i}: 'float_col' key is ABSENT \
                 from row object. Every row must contain all projected column keys even \
                 when the serialized value is JSON null. \
                 Got row {i}: {row}"
            );
        }

        // (b) Row 0: finite 1.5 → JSON Number 1.5 (NOT null, NOT a string).
        assert!(
            rows[0]["float_col"].is_number(),
            "EC-11-081: row 0 (finite 1.5) must serialize as a JSON Number, not null or string; \
             got: {}",
            rows[0]["float_col"]
        );
        assert_eq!(
            rows[0]["float_col"]
                .as_f64()
                .expect("row 0 float_col must be a JSON number"),
            1.5_f64,
            "EC-11-081: row 0 float_col must be exactly 1.5"
        );

        // (c) Rows 1-3: non-finite values (NaN, +Inf, -Inf) → JSON null.
        // arrow-json v58 hardcoded encoder: non-finite Float64 → JSON null (RFC 8259 compliance).
        // Regression signal: if this assertion fails after an arrow-json bump, the encoder
        // behavior changed — re-adjudicate BC-2.11.001 EC-11-081 before accepting the upgrade.
        let nonfinite_labels = ["NaN", "+Inf", "-Inf"];
        for (i, label) in nonfinite_labels.iter().enumerate() {
            let row_idx = i + 1;
            assert!(
                rows[row_idx]["float_col"].is_null(),
                "EC-11-081 VIOLATION (arrow-json non-finite encoder): row {row_idx} ({label}) \
                 must serialize as JSON null. arrow-json v58 hardcodes NaN/±Inf as null per \
                 RFC 8259 (JSON has no non-finite literal). If this fails after an arrow-json \
                 version bump, BC-2.11.001 EC-11-081 Option A must be re-adjudicated. \
                 Got: {}",
                rows[row_idx]["float_col"]
            );
        }

        // (c) Row 4: Arrow-null → JSON null (via with_explicit_nulls(true), EC-11-079 invariant).
        assert!(
            rows[4]["float_col"].is_null(),
            "EC-11-081 / EC-11-079: row 4 (Arrow-null) must serialize as JSON null \
             via with_explicit_nulls(true); got: {}",
            rows[4]["float_col"]
        );
    }
}
