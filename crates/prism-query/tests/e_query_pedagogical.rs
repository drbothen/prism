//! Load-bearing Red Gate tests for E-QUERY-038 plan-time column gate and
//! E-QUERY-001/002/003/037 pedagogical enrichments (BC-2.11.016, BC-2.11.017).
// Test code — allow expect/unwrap per the project pattern for prism-query test files.
// (prism-query Cargo.toml sets expect_used = "deny" / unwrap_used = "deny" for production code.)
#![allow(clippy::expect_used, clippy::unwrap_used)]
//!
//! Story: S-DEMO-PRISMQL-ONBOARDING-001-B
//!
//! All BEHAVIOUR tests drive real production code paths — they call
//! `QueryEngine::execute()` with a wired `resolved_spec_map` and
//! `TableRegistry`.  Deleting `check_column_availability` from engine.rs,
//! or removing the helpers from the error-construction sites, will cause
//! these tests to fail.
//!
//! TD-VSDD-059 (paper-fix detection) compliance: every assertion in a
//! Red Gate test exercises production code output, never a value the
//! test itself supplied to a constructor or helper.
//!
//! # Red Gate test catalogue (matches story Red Gate table)
//! | Test | AC | Production path exercised | Fails NOW because |
//! |---|---|---|---|
//! | `test_BC_2_11_016_e_query_038_did_you_mean` | AC-001 | `QueryEngine::execute` → `check_column_availability` | gate never called from execute_inner |
//! | `test_BC_2_11_016_e_query_038_gate_ordering_table_not_found_returns_037` | AC-001 | E-QUERY-037 fires BEFORE E-QUERY-038 | structural check — should stay green |
//! | `test_BC_2_11_016_e_query_038_org_scoped_available_columns` | AC-002 | execute with org-scoped resolved_spec_map | gate never called |
//! | `test_BC_2_11_016_ec_039_empty_table_columns_available_columns_empty` | AC-001 | execute against zero-column table | gate never called |
//! | `test_BC_2_11_017_e_query_037_suggestion_prism_describe` | AC-004 | execute → E-QUERY-037 Display contains "prism_describe" | e_query_037_suggestion not called from table gate |
//! | `test_BC_2_11_017_enrichment_helpers_valid_operators_for_type` | AC-003 | valid_operators_for_type returns correct slices | load-bearing for helper code |
//! | `test_BC_2_11_017_enrichment_helper_extract_near_text` | AC-003 | extract_near_text produces correct near-text | load-bearing for helper code |
//! | `test_BC_2_11_017_enrichment_helper_how_to_fix_for_security_limit` | AC-003 | how_to_fix_for_security_limit returns actionable strings | load-bearing for helper code |
//!
//! # BC references
//! - BC-2.11.016 — E-QUERY-038 Column-Not-Found Plan-Time Gate
//! - BC-2.11.017 — E-QUERY Pedagogical Enrichments

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, sync::Arc};

    use prism_core::{OrgSlug, PrismError, SensorId};
    use prism_query::{
        engine::{QueryEngine, QueryEngineConfig, QueryOptions},
        scoping::ClientRegistry,
        table_registry::TableRegistry,
    };
    use prism_spec_engine::{
        overlay::{OverlayLoader, ResolvedSensorSpec, ResolvedSpecKey, SensorInstanceOverlay},
        spec_parser::{AuthType, ColumnSpec, SensorSpec, TableSpec},
    };

    // =========================================================================
    // Test fixture helpers
    // =========================================================================

    /// Minimal no-op credential store — prevents CredentialStore trait obj errors.
    struct NoopCs;

    #[async_trait::async_trait]
    impl prism_credentials::CredentialStore for NoopCs {
        async fn get(
            &self,
            _t: &prism_core::OrgSlug,
            _s: &str,
            _n: &prism_credentials::namespace::CredentialName,
        ) -> Result<Option<secrecy::SecretString>, PrismError> {
            Ok(None)
        }
        async fn set(
            &self,
            _t: &prism_core::OrgSlug,
            _s: &str,
            _n: &prism_credentials::namespace::CredentialName,
            _v: secrecy::SecretString,
        ) -> Result<(), PrismError> {
            Ok(())
        }
        async fn delete(
            &self,
            _t: &prism_core::OrgSlug,
            _s: &str,
            _n: &prism_credentials::namespace::CredentialName,
        ) -> Result<bool, PrismError> {
            Ok(false)
        }
        async fn list(
            &self,
            _t: &prism_core::OrgSlug,
        ) -> Result<Vec<(String, prism_credentials::namespace::CredentialName)>, PrismError>
        {
            Ok(vec![])
        }
        async fn exists(
            &self,
            _t: &prism_core::OrgSlug,
            _s: &str,
            _n: &prism_credentials::namespace::CredentialName,
        ) -> Result<bool, PrismError> {
            Ok(false)
        }
    }

    /// Build a `ResolvedSensorSpec` via the real `OverlayLoader::merge_overlay_onto_type_spec`.
    ///
    /// This is the SAME pattern used by the production boot path.
    ///
    /// `table_suffix` is the SHORT table name (e.g. `"alerts"`), NOT the fully-qualified form.
    /// `register_sensor` constructs the fully-qualified `{sensor_id}_{table_suffix}` name —
    /// e.g. sensor_id=`"crowdstrike"` + suffix=`"alerts"` → registered as `"crowdstrike_alerts"`.
    /// SQL queries must reference the fully-qualified name: `FROM crowdstrike_alerts`.
    ///
    /// `check_column_availability` (when wired by the implementer) must match the query's
    /// fully-qualified table name to the spec entry by reconstructing the fully-qualified form
    /// as `format!("{sensor_id}_{table_name}")` — the BC test vectors use this naming contract.
    fn make_resolved(
        sensor_id: &str,
        table_suffix: &str,
        columns: Vec<ColumnSpec>,
        org: &str,
    ) -> (ResolvedSpecKey, ResolvedSensorSpec) {
        let spec = SensorSpec::new(
            sensor_id,
            format!("{sensor_id} sensor"),
            AuthType::ApiKey,
            "https://example.com",
            vec![TableSpec::new_point_in_time(
                table_suffix,
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
            toml::from_str(&overlay_toml).expect("fixture: SensorInstanceOverlay TOML must parse");
        let org_slug = OrgSlug::new(org);
        let resolved =
            OverlayLoader::merge_overlay_onto_type_spec(&spec, &overlay, org_slug.clone());
        let sensor_id_typed = SensorId::new(sensor_id);
        let key: ResolvedSpecKey = (org_slug, sensor_id_typed);
        (key, resolved)
    }

    /// Build a `QueryEngine` with a wired `resolved_spec_map` and `TableRegistry`.
    ///
    /// The `TableRegistry` is populated from the same sensor specs as the map so that
    /// `check_table_availability` passes for registered tables.
    ///
    /// `ClientRegistry` is empty — callers that pass `QueryOptions::clients = None`
    /// go through the `None` branch of `resolve_clients` (returns all clients = [])
    /// and execute with zero fan-out, which is correct for tests that want "no error
    /// from the client gate".
    ///
    /// Use `make_engine_with_clients` for tests that pass explicit clients in
    /// `QueryOptions::clients = Some(...)` and need them to be present in the registry.
    fn make_engine(
        resolved_map: HashMap<ResolvedSpecKey, ResolvedSensorSpec>,
        specs_for_registry: Vec<SensorSpec>,
    ) -> QueryEngine {
        make_engine_with_clients(resolved_map, specs_for_registry, vec![])
    }

    /// Build a `QueryEngine` with registered client IDs for tests that pass explicit
    /// `QueryOptions::clients = Some(...)` — those are validated by `resolve_clients`
    /// against the `ClientRegistry`.
    fn make_engine_with_clients(
        resolved_map: HashMap<ResolvedSpecKey, ResolvedSensorSpec>,
        specs_for_registry: Vec<SensorSpec>,
        clients: Vec<OrgSlug>,
    ) -> QueryEngine {
        let registry = Arc::new(TableRegistry::new());
        for spec in &specs_for_registry {
            registry
                .register_sensor(spec)
                .expect("register_sensor must not fail in test fixture");
        }

        let engine = QueryEngine::new_with_cache_config(
            Arc::new(prism_sensors::AdapterRegistry::new()),
            Arc::new(NoopCs),
            Arc::new(prism_ocsf::OcsfNormalizer::new()),
            Arc::new(ClientRegistry::new(clients)),
            QueryEngineConfig::default(),
            prism_query::cache::CacheConfig::default(),
        )
        .with_resolved_spec_map(Arc::new(resolved_map));
        engine.with_table_registry(registry)
    }

    // =========================================================================
    // AC-001 — E-QUERY-038 gate payload shape (Red Gate tests)
    // =========================================================================

    /// BC-2.11.016 / AC-001 / AC-002 — `QueryEngine::execute` returns `ColumnNotFound` for
    /// a misspelled column in the **WHERE clause** (BC-2.11.016 Precondition 2 covers WHERE,
    /// GROUP BY, ORDER BY — not just SELECT).
    ///
    /// LOAD-BEARING: this test uses the LITERAL AC-002 canonical query from BC-2.11.016:
    ///   `SELECT * FROM crowdstrike_alerts WHERE sevrity = 'high'`
    /// The `SELECT *` means the SELECT-clause check is skipped entirely. The WHERE column
    /// `sevrity` MUST be caught by the gate. Without WHERE-clause scanning in
    /// `check_query_column_availability`, `execute` returns `Ok(QueryResult)` — this test
    /// is RED until the WHERE-position check is wired.
    ///
    /// F-PRL-CRIT-001 (LOCAL adversary pass-2): the prior test used
    /// `SELECT sevrity FROM crowdstrike_alerts LIMIT 5` (SELECT position), which passed
    /// because the SELECT-position check already existed. That test was reshaped to mask
    /// the WHERE gap. This test reinstates the canonical AC-002 query.
    ///
    /// BC-2.11.016 Precondition 2: "The query references a column name in a position where
    /// column resolution is possible (e.g., `SELECT <column>`, `WHERE <column> = ...`,
    /// `GROUP BY <column>`, `ORDER BY <column>`)."
    #[tokio::test]
    async fn test_BC_2_11_016_e_query_038_did_you_mean() {
        use prism_core::column::ColumnType;

        let columns = vec![
            ColumnSpec::new("severity", ColumnType::String, None, vec![]),
            ColumnSpec::new("host_name", ColumnType::String, None, vec![]),
            ColumnSpec::new("detection_id", ColumnType::String, None, vec![]),
        ];
        // sensor_id="crowdstrike" + table_suffix="alerts" → registered as "crowdstrike_alerts"
        let (key, resolved) = make_resolved("crowdstrike", "alerts", columns, "acme");

        let mut map = HashMap::new();
        map.insert(key, resolved.clone());
        let sensor_spec = resolved.spec.clone();
        let engine = make_engine(map, vec![sensor_spec]);

        // ---- Case 1 (LITERAL AC-002 CANONICAL QUERY): WHERE position typo ----
        //
        // BC-2.11.016 canonical test vector: `SELECT * FROM crowdstrike_alerts WHERE sevrity = 'high'`
        // SELECT * means the SELECT check is skipped — only the WHERE check catches "sevrity".
        //
        // LOAD-BEARING: if check_query_column_availability only inspects SELECT clause columns,
        // this returns Ok(...) instead of Err(ColumnNotFound). The gate MUST also scan WHERE.
        //
        // FIX: extend check_query_column_availability to extract field references from
        // `sql_query.where_` (Predicate::Compare, Predicate::StringOp, etc.) and call
        // check_column_availability for each WHERE column reference.
        let result = engine
            .execute(
                "SELECT * FROM crowdstrike_alerts WHERE sevrity = 'high'",
                QueryOptions::default(),
            )
            .await;

        match result {
            Err(PrismError::ColumnNotFound(ref d)) => {
                assert_eq!(
                    d.column, "sevrity",
                    "BC-2.11.016 AC-002: column field must be 'sevrity'; got: '{}'",
                    d.column
                );
                assert_eq!(
                    d.table, "crowdstrike_alerts",
                    "BC-2.11.016 AC-002: table field must be 'crowdstrike_alerts'; got: '{}'",
                    d.table
                );
                assert!(
                    d.available_columns.contains(&"severity".to_string()),
                    "BC-2.11.016 AC-002: available_columns must contain 'severity'; \
                     got: {:?}",
                    d.available_columns
                );
                assert!(
                    !d.available_columns.is_empty(),
                    "BC-2.11.016 AC-002: available_columns must be non-empty"
                );
                assert_eq!(
                    d.did_you_mean,
                    Some("severity".to_string()),
                    "BC-2.11.016 AC-002: did_you_mean must be Some('severity') for Lev-1 typo; \
                     got: {:?}",
                    d.did_you_mean
                );
                let display = format!("{d}");
                assert!(
                    display.starts_with("E-QUERY-038:"),
                    "BC-2.11.016 AC-002: Display must start with 'E-QUERY-038:'; got: '{display}'"
                );
            }
            Ok(_) => panic!(
                "BC-2.11.016 AC-002 (F-PRL-CRIT-001): QueryEngine::execute MUST return \
                 Err(ColumnNotFound) for column 'sevrity' in WHERE clause of \
                 'SELECT * FROM crowdstrike_alerts WHERE sevrity = ...' when resolved_spec_map \
                 is wired. Got Ok — check_query_column_availability does NOT scan WHERE clause. \
                 FIX: extend check_query_column_availability to extract column references from \
                 sql_query.where_ (Predicate::Compare lhs, Predicate::StringOp field, \
                 Predicate::In field, etc.) and call check_column_availability for each."
            ),
            Err(other) => {
                panic!(
                    "BC-2.11.016 AC-002: expected ColumnNotFound for 'sevrity' in WHERE, \
                     got: {other:?}"
                )
            }
        }

        // ---- Case 2 (SELECT-position regression): still works in SELECT ----
        // This verifies the original SELECT-position check is not broken.
        let select_result = engine
            .execute(
                "SELECT completely_bogus_col FROM crowdstrike_alerts LIMIT 5",
                QueryOptions::default(),
            )
            .await;

        match select_result {
            Err(PrismError::ColumnNotFound(ref d)) => {
                assert!(
                    d.did_you_mean.is_none(),
                    "BC-2.11.016 SELECT-regression: did_you_mean must be None for \
                     'completely_bogus_col'; got: {:?}",
                    d.did_you_mean
                );
                assert!(
                    !d.available_columns.is_empty(),
                    "BC-2.11.016 SELECT-regression: available_columns must be non-empty"
                );
            }
            Ok(_) => panic!(
                "BC-2.11.016 SELECT-regression: must return ColumnNotFound for \
                 'completely_bogus_col' in SELECT — gate must still work in SELECT position."
            ),
            Err(other) => panic!(
                "BC-2.11.016 SELECT-regression: expected ColumnNotFound for \
                 'completely_bogus_col', got: {other:?}"
            ),
        }
    }

    /// BC-2.11.016 / F-PRL-CRIT-001 — E-QUERY-038 gate covers GROUP BY and ORDER BY positions.
    ///
    /// BC-2.11.016 Precondition 2 explicitly lists GROUP BY and ORDER BY alongside WHERE as
    /// positions where column resolution is required. The gate MUST catch typos in all four
    /// positions: SELECT, WHERE, GROUP BY, ORDER BY.
    ///
    /// LOAD-BEARING: both assertions FAIL on current HEAD because `check_query_column_availability`
    /// only scans SELECT clause columns. GROUP BY and ORDER BY fields are not extracted.
    ///
    /// FIX: extend check_query_column_availability to also collect column references from
    /// `sql_query.group_by` (Vec<Expr>) and `sql_query.order_by` (Vec<OrderExpr>).
    #[tokio::test]
    async fn test_BC_2_11_016_e_query_038_where_group_by_order_by_positions() {
        use prism_core::column::ColumnType;

        let columns = vec![
            ColumnSpec::new("severity", ColumnType::String, None, vec![]),
            ColumnSpec::new("host_name", ColumnType::String, None, vec![]),
            ColumnSpec::new("detection_id", ColumnType::String, None, vec![]),
        ];
        // sensor_id="crowdstrike" + table_suffix="alerts" → registered as "crowdstrike_alerts"
        let (key, resolved) = make_resolved("crowdstrike", "alerts", columns, "acme");
        let mut map = HashMap::new();
        map.insert(key, resolved.clone());
        let engine = make_engine(map, vec![resolved.spec.clone()]);

        // ---- GROUP BY position: typo in GROUP BY column ----
        // SELECT * skips SELECT check. "sevrity" only in GROUP BY → must be caught there.
        // LOAD-BEARING: returns Ok if GROUP BY columns are not extracted by the gate.
        let group_by_result = engine
            .execute(
                "SELECT COUNT(*) FROM crowdstrike_alerts GROUP BY sevrity",
                QueryOptions::default(),
            )
            .await;

        match group_by_result {
            Err(PrismError::ColumnNotFound(ref d)) => {
                assert_eq!(
                    d.column, "sevrity",
                    "BC-2.11.016 GROUP-BY: column must be 'sevrity'; got: '{}'",
                    d.column
                );
                assert!(
                    d.available_columns.contains(&"severity".to_string()),
                    "BC-2.11.016 GROUP-BY: available_columns must contain 'severity'; \
                     got: {:?}",
                    d.available_columns
                );
            }
            Ok(_) => panic!(
                "BC-2.11.016 (F-PRL-CRIT-001): GROUP BY position — \
                 'sevrity' in GROUP BY must produce ColumnNotFound. Got Ok. \
                 FIX: extract Expr::Field references from sql_query.group_by and pass \
                 each to check_column_availability."
            ),
            Err(other) => panic!(
                "BC-2.11.016 GROUP-BY: expected ColumnNotFound for 'sevrity' in GROUP BY, \
                 got: {other:?}"
            ),
        }

        // ---- ORDER BY position: typo in ORDER BY column ----
        // "sevrity" only in ORDER BY → must be caught there.
        // LOAD-BEARING: returns Ok if ORDER BY columns are not extracted by the gate.
        let order_by_result = engine
            .execute(
                "SELECT severity FROM crowdstrike_alerts ORDER BY sevrity",
                QueryOptions::default(),
            )
            .await;

        match order_by_result {
            Err(PrismError::ColumnNotFound(ref d)) => {
                // Note: "severity" IS in SELECT (valid), only "sevrity" in ORDER BY is invalid.
                assert_eq!(
                    d.column, "sevrity",
                    "BC-2.11.016 ORDER-BY: column must be 'sevrity'; got: '{}'",
                    d.column
                );
                assert!(
                    d.available_columns.contains(&"severity".to_string()),
                    "BC-2.11.016 ORDER-BY: available_columns must contain 'severity'; \
                     got: {:?}",
                    d.available_columns
                );
            }
            Ok(_) => panic!(
                "BC-2.11.016 (F-PRL-CRIT-001): ORDER BY position — \
                 'sevrity' in ORDER BY must produce ColumnNotFound. Got Ok. \
                 FIX: extract the column name from each OrderExpr in sql_query.order_by and \
                 pass each to check_column_availability."
            ),
            Err(other) => panic!(
                "BC-2.11.016 ORDER-BY: expected ColumnNotFound for 'sevrity' in ORDER BY, \
                 got: {other:?}"
            ),
        }
    }

    /// BC-2.11.016 / AC-001 — Gate ordering: nonexistent table → E-QUERY-037 (NOT E-QUERY-038).
    ///
    /// Verifies that table availability is checked BEFORE column availability.
    /// If gate ordering is reversed, a nonexistent table would produce ColumnNotFound
    /// instead of TableNotAvailable — this test catches that regression.
    ///
    /// This test should stay GREEN on current HEAD (the table gate fires already).
    /// It becomes CRITICAL if the implementer introduces an ordering bug when wiring
    /// check_column_availability.
    #[tokio::test]
    async fn test_BC_2_11_016_e_query_038_gate_ordering_table_not_found_returns_037() {
        use prism_core::column::ColumnType;

        let columns = vec![ColumnSpec::new(
            "severity",
            ColumnType::String,
            None,
            vec![],
        )];
        // sensor_id="crowdstrike" + suffix="alerts" → registered as "crowdstrike_alerts"
        let (key, resolved) = make_resolved("crowdstrike", "alerts", columns, "acme");

        let mut map = HashMap::new();
        map.insert(key, resolved.clone());
        let engine = make_engine(map, vec![resolved.spec.clone()]);

        let result = engine
            .execute(
                "SELECT * FROM nonexistent_table WHERE bogus_col = 1",
                QueryOptions::default(),
            )
            .await;

        match result {
            Err(PrismError::TableNotAvailable(ref d)) => {
                assert_eq!(
                    d.table, "nonexistent_table",
                    "BC-2.11.016 gate-ordering: E-QUERY-037 must report the queried table name"
                );
                let display = d.to_string();
                assert!(
                    display.starts_with("E-QUERY-037:"),
                    "gate-ordering: Display must start with 'E-QUERY-037:'; got: '{display}'"
                );
            }
            Err(PrismError::ColumnNotFound(_)) => panic!(
                "BC-2.11.016 gate ordering VIOLATION: ColumnNotFound fired for 'nonexistent_table'. \
                 E-QUERY-037 must fire BEFORE E-QUERY-038. \
                 FIX: check_column_availability must only run AFTER check_table_availability passes."
            ),
            Ok(_) => {
                panic!("gate-ordering: must return TableNotAvailable for 'nonexistent_table'.")
            }
            Err(other) => panic!(
                "gate-ordering: expected TableNotAvailable for 'nonexistent_table', got: {other:?}"
            ),
        }
    }

    // =========================================================================
    // AC-002 — E-QUERY-038 org-scoped available_columns (Red Gate test)
    // =========================================================================

    /// BC-2.11.016 / AC-002 / DI-008 — Org-scoped `available_columns` in E-QUERY-038 error.
    ///
    /// LOAD-BEARING: fails if `check_column_availability` is not wired, OR if the
    /// org-scope filter in `check_column_availability` is absent/incorrect.
    ///
    /// Multi-tenant fixture:
    ///   acme  → crowdstrike → crowdstrike_alerts: [severity, host_name, acme_only_field]
    ///   globex → crowdstrike → crowdstrike_alerts: [severity, globex_alert_type, globex_region]
    ///
    /// When acme executes a bad-column query, available_columns must ONLY contain acme's columns.
    /// No globex columns and no credential/URL strings must appear.
    #[tokio::test]
    async fn test_BC_2_11_016_e_query_038_org_scoped_available_columns() {
        use prism_core::column::ColumnType;

        let acme_columns = vec![
            ColumnSpec::new("severity", ColumnType::String, None, vec![]),
            ColumnSpec::new("host_name", ColumnType::String, None, vec![]),
            ColumnSpec::new("acme_only_field", ColumnType::String, None, vec![]),
        ];
        let globex_columns = vec![
            ColumnSpec::new("severity", ColumnType::String, None, vec![]),
            ColumnSpec::new("globex_alert_type", ColumnType::String, None, vec![]),
            ColumnSpec::new("globex_region", ColumnType::String, None, vec![]),
        ];

        // sensor_id="crowdstrike" + suffix="alerts" → registered as "crowdstrike_alerts"
        let (acme_key, acme_resolved) =
            make_resolved("crowdstrike", "alerts", acme_columns, "acme");
        let (globex_key, globex_resolved) =
            make_resolved("crowdstrike", "alerts", globex_columns, "globex");

        let mut map = HashMap::new();
        map.insert(acme_key, acme_resolved.clone());
        map.insert(globex_key, globex_resolved);

        let sensor_spec = acme_resolved.spec.clone();
        // Register both "acme" and "globex" so resolve_clients() accepts clients=Some([acme]).
        // Without client registration, `resolve_clients` returns InvalidClientId before
        // check_column_availability can fire — masking the production wiring gap.
        let engine = make_engine_with_clients(
            map,
            vec![sensor_spec],
            vec![OrgSlug::new("acme"), OrgSlug::new("globex")],
        );

        let mut opts = QueryOptions::default();
        opts.clients = Some(vec![OrgSlug::new("acme")]);

        let result = engine
            .execute("SELECT sevrity FROM crowdstrike_alerts LIMIT 5", opts)
            .await;

        match result {
            Err(PrismError::ColumnNotFound(ref d)) => {
                assert!(
                    d.available_columns.contains(&"acme_only_field".to_string()),
                    "DI-008 / AC-002: available_columns must contain acme's 'acme_only_field'; \
                     got: {:?}",
                    d.available_columns
                );
                assert!(
                    !d.available_columns
                        .contains(&"globex_alert_type".to_string()),
                    "DI-008 VIOLATION: 'globex_alert_type' must NOT appear in acme's \
                     available_columns; got: {:?}",
                    d.available_columns
                );
                assert!(
                    !d.available_columns.contains(&"globex_region".to_string()),
                    "DI-008 VIOLATION: 'globex_region' must NOT appear in acme's \
                     available_columns; got: {:?}",
                    d.available_columns
                );
                assert_eq!(
                    d.client_id, "acme",
                    "AC-002: client_id must match querying org 'acme'; got: '{}'",
                    d.client_id
                );
                // Injection safety: no credential-pattern strings in available_columns
                for col in &d.available_columns {
                    assert!(
                        !col.starts_with("https://"),
                        "AC-002: available_columns must not contain URL strings; col: '{col}'"
                    );
                    assert!(
                        !col.starts_with("Bearer "),
                        "AC-002: available_columns must not contain bearer token strings; \
                         col: '{col}'"
                    );
                }
            }
            Ok(_) => panic!(
                "AC-002: QueryEngine::execute must return Err(ColumnNotFound) for 'sevrity' in \
                 'crowdstrike_alerts' when resolved_spec_map is wired. Got Ok — \
                 check_column_availability is NOT called."
            ),
            Err(other) => {
                panic!("AC-002: expected ColumnNotFound for multi-tenant test, got: {other:?}")
            }
        }
    }

    // =========================================================================
    // AC-001 edge case: zero-column table
    // =========================================================================

    /// BC-2.11.016 EC-11-039 — table with zero columns → available_columns: [].
    ///
    /// LOAD-BEARING via execute path: without gate wiring, returns Ok instead of Err.
    #[tokio::test]
    async fn test_BC_2_11_016_ec_039_empty_table_columns_available_columns_empty() {
        // sensor_id="armis" + suffix="devices" → registered as "armis_devices"
        let (key, resolved) = make_resolved("armis", "devices", vec![], "acme");
        let mut map = HashMap::new();
        map.insert(key, resolved.clone());
        let engine = make_engine(map, vec![resolved.spec.clone()]);

        // Query uses the fully-qualified name: sensor_id + _ + suffix = armis_devices
        let result = engine
            .execute(
                "SELECT some_col FROM armis_devices LIMIT 5",
                QueryOptions::default(),
            )
            .await;

        match result {
            Err(PrismError::ColumnNotFound(ref d)) => {
                assert!(
                    d.available_columns.is_empty(),
                    "EC-11-039: available_columns must be [] when table has zero columns; \
                     got: {:?}",
                    d.available_columns
                );
                assert!(
                    d.did_you_mean.is_none(),
                    "EC-11-039: did_you_mean must be None when available_columns is empty; \
                     got: {:?}",
                    d.did_you_mean
                );
            }
            Ok(_) => panic!(
                "EC-11-039: must return Err(ColumnNotFound) for 'some_col' in 'armis_devices' \
                 (zero-column table) when gate is wired. Got Ok."
            ),
            Err(other) => panic!("EC-11-039: expected ColumnNotFound, got: {other:?}"),
        }
    }

    // =========================================================================
    // AC-004 — E-QUERY-037 suggestion with prism_describe reference (Red Gate test)
    // =========================================================================

    /// BC-2.11.017 / AC-004 — E-QUERY-037 Display contains "prism_describe".
    ///
    /// LOAD-BEARING: `TableNotAvailableDetails::Display` currently reads:
    ///   "E-QUERY-037: table '...' is not available — sensor '...' is not configured.
    ///    Available sensors: [...]. Available tables: [...]. Did you mean: '...'?"
    /// The word "prism_describe" does NOT appear — this test FAILS until wired.
    ///
    /// FIX: call `e_query_037_suggestion(client_id, did_you_mean_option)` when constructing
    /// `TableNotAvailableDetails` in `check_availability_gate` in table_registry.rs, and
    /// include the result in the Display output (via a new `suggestion` field, or by
    /// appending to `did_you_mean`).
    #[tokio::test]
    async fn test_BC_2_11_017_e_query_037_suggestion_prism_describe() {
        use prism_core::column::ColumnType;

        let columns = vec![ColumnSpec::new(
            "severity",
            ColumnType::String,
            None,
            vec![],
        )];
        // sensor_id="crowdstrike" + suffix="alerts" → registered as "crowdstrike_alerts"
        let (key, resolved) = make_resolved("crowdstrike", "alerts", columns, "acme");
        let mut map = HashMap::new();
        map.insert(key, resolved.clone());
        let engine = make_engine(map, vec![resolved.spec.clone()]);

        // ---- Case 1: "crowdstrike_alert" (Levenshtein-1 typo for "crowdstrike_alerts") ----
        let result = engine
            .execute(
                "SELECT severity FROM crowdstrike_alert LIMIT 5",
                QueryOptions::default(),
            )
            .await;

        match result {
            Err(PrismError::TableNotAvailable(ref d)) => {
                // The Display output must contain "prism_describe".
                // Currently the Display does NOT contain it — this assertion FAILS.
                let error_text = d.to_string();
                assert!(
                    error_text.contains("prism_describe"),
                    "BC-2.11.017 AC-004: E-QUERY-037 output must contain 'prism_describe' so \
                     users know how to discover available tables. \
                     Current output: '{error_text}'. \
                     FIX: call e_query_037_suggestion() when constructing TableNotAvailableDetails \
                     and include it in the Display output."
                );
                // When a close match exists, the suggestion must name it
                assert!(
                    error_text.contains("crowdstrike_alerts"),
                    "BC-2.11.017 AC-004: E-QUERY-037 with did_you_mean must name the corrected \
                     table 'crowdstrike_alerts'; got: '{error_text}'"
                );
            }
            Ok(_) => {
                panic!("BC-2.11.017 AC-004: must return TableNotAvailable for 'crowdstrike_alert'.")
            }
            Err(other) => panic!(
                "BC-2.11.017 AC-004: expected TableNotAvailable for 'crowdstrike_alert', \
                 got: {other:?}"
            ),
        }

        // ---- Case 2: "completely_made_up_table" (no close match) ----
        let no_match_result = engine
            .execute(
                "SELECT severity FROM completely_made_up_table LIMIT 5",
                QueryOptions::default(),
            )
            .await;

        match no_match_result {
            Err(PrismError::TableNotAvailable(ref d)) => {
                let error_text = d.to_string();
                assert!(
                    error_text.contains("prism_describe"),
                    "BC-2.11.017 AC-004: must contain 'prism_describe' even when no close \
                     match exists; got: '{error_text}'"
                );
                assert!(
                    !error_text.contains("If you meant"),
                    "BC-2.11.017 AC-004: no-match E-QUERY-037 must not contain 'If you meant'; \
                     got: '{error_text}'"
                );
            }
            Ok(_) => panic!(
                "BC-2.11.017 AC-004: must return TableNotAvailable for 'completely_made_up_table'."
            ),
            Err(other) => panic!("BC-2.11.017 AC-004: expected TableNotAvailable, got: {other:?}"),
        }
    }

    // =========================================================================
    // AC-004 (hardened) — E-QUERY-037 suggestion names the CLIENT_ID, not the sensor
    // =========================================================================

    /// BC-2.11.017 / AC-004 (F-001B-FRESH-P1-MED-001) — E-QUERY-037 suggestion uses
    /// the requesting CLIENT_ID in `prism_describe('<client_id>')`, NOT the sensor name.
    ///
    /// LOAD-BEARING (RED → GREEN): when `QueryOptions::clients = Some(["acme"])` the
    /// `org_scope` passed to `check_availability_gate` is `Some([OrgSlug("acme")])`.
    /// The suggestion must produce `prism_describe('acme')`, NOT `prism_describe('crowdstrike')`.
    ///
    /// BC-2.11.017 §E-QUERY-037 + error-taxonomy literally specify:
    ///   `prism_describe('<client_id>')`  — validated + resolved via OrgRegistry.
    /// Passing a sensor name (`crowdstrike`) breaks the LLM self-correction loop because
    /// `prism_describe('crowdstrike')` fails with EC-10-023 "Client not registered".
    ///
    /// Current behaviour (pre-fix): `check_availability_gate` calls
    ///   `e_query_037_suggestion(&sensor, ...)` → `prism_describe('crowdstrike')`.
    /// Fixed behaviour: uses `org_scope.and_then(|s| s.first()).map(|o| o.as_str())`
    ///   → `prism_describe('acme')`.
    ///
    /// Deleting the fix in table_registry.rs makes this test fail (load-bearing).
    #[tokio::test]
    async fn test_BC_2_11_017_e_query_037_suggestion_uses_client_id_not_sensor() {
        use prism_core::column::ColumnType;

        let columns = vec![ColumnSpec::new(
            "severity",
            ColumnType::String,
            None,
            vec![],
        )];
        // sensor_id="crowdstrike" + suffix="alerts" → registered as "crowdstrike_alerts"
        let (key, resolved) = make_resolved("crowdstrike", "alerts", columns, "acme");
        let mut map = HashMap::new();
        map.insert(key, resolved.clone());
        // Register "acme" in ClientRegistry so resolve_clients doesn't reject it
        // if the gate somehow passes (defensive; E-QUERY-037 fires before resolve_clients).
        let engine =
            make_engine_with_clients(map, vec![resolved.spec.clone()], vec![OrgSlug::new("acme")]);

        // Query an unregistered table while explicitly scoping to client "acme".
        // org_scope = Some([OrgSlug("acme")]) → suggestion must name 'acme'.
        let result = engine
            .execute(
                "SELECT severity FROM crowdstrike_alert LIMIT 5",
                QueryOptions {
                    clients: Some(vec![OrgSlug::new("acme")]),
                    ..QueryOptions::default()
                },
            )
            .await;

        match result {
            Err(PrismError::TableNotAvailable(ref d)) => {
                let error_text = d.to_string();

                // Must contain the client_id, not the sensor, in the prism_describe call.
                assert!(
                    error_text.contains("prism_describe('acme')"),
                    "BC-2.11.017 AC-004 (F-001B-FRESH-P1-MED-001): E-QUERY-037 suggestion must \
                     contain `prism_describe('acme')` (the client_id), not `prism_describe('crowdstrike')` \
                     (the sensor). \
                     Current output: '{error_text}'. \
                     FIX: in check_availability_gate, derive client_id from org_scope.first() \
                     instead of passing &sensor to e_query_037_suggestion."
                );

                // Must NOT name the sensor in the prism_describe call — that sends the LLM
                // to `prism_describe('crowdstrike')` which fails with EC-10-023.
                assert!(
                    !error_text.contains("prism_describe('crowdstrike')"),
                    "BC-2.11.017 AC-004 (F-001B-FRESH-P1-MED-001): E-QUERY-037 suggestion must NOT \
                     contain `prism_describe('crowdstrike')` (sensor name is not a valid client_id). \
                     Current output: '{error_text}'."
                );

                // Sanity: the did_you_mean table name must still appear (Levenshtein match).
                assert!(
                    error_text.contains("crowdstrike_alerts"),
                    "BC-2.11.017 AC-004: E-QUERY-037 with did_you_mean must still name the \
                     corrected table 'crowdstrike_alerts'; got: '{error_text}'"
                );
            }
            Ok(_) => panic!(
                "BC-2.11.017 AC-004: must return TableNotAvailable for 'crowdstrike_alert' with \
                 client scope [acme]."
            ),
            Err(other) => panic!(
                "BC-2.11.017 AC-004: expected TableNotAvailable for 'crowdstrike_alert', \
                 got: {other:?}"
            ),
        }
    }

    // =========================================================================
    // AC-003 — E-QUERY enrichment helpers (load-bearing for helper code)
    // =========================================================================

    /// BC-2.11.017 / AC-003 — `valid_operators_for_type` returns canonical operator sets.
    ///
    /// LOAD-BEARING: exercises the production helper. If the function is deleted or
    /// returns wrong values, these assertions fail.
    ///
    /// Note: the MCP layer wiring (helpers invoked from error_mapping.rs into the JSON
    /// error response) is tested separately in crates/prism-mcp/tests/normalized_pql.rs.
    ///
    /// Canonical values from BC-2.11.017 §E-QUERY-002 operator table.
    #[test]
    fn test_BC_2_11_017_enrichment_helpers_valid_operators_for_type() {
        use prism_core::column::ColumnType;
        use prism_query::engine::valid_operators_for_type;

        // String: exactly 8 operators (5 case-sensitive + 3 case-insensitive IEQ/IIN/INE).
        // BC-2.11.024: IEQ/IIN/INE added by F-P24-MED-001 (S-PRISMQL-CASE-INSENSITIVE-001).
        // "NOT IIN" absent — negated IIN is not representable in the PrismQL AST.
        let string_ops = valid_operators_for_type(ColumnType::String);
        let required_string = ["=", "!=", "LIKE", "IN", "NOT IN", "IEQ", "IIN", "INE"];
        for op in &required_string {
            assert!(
                string_ops.contains(op),
                "AC-003: valid_operators_for_type(String) must include '{op}'; got: {string_ops:?}"
            );
        }
        assert_eq!(
            string_ops.len(),
            8,
            "AC-003: String must have exactly 8 operators (= != LIKE IN NOT_IN IEQ IIN INE); got: {string_ops:?}"
        );

        // Integer: exactly 9 operators (includes BETWEEN)
        let int_ops = valid_operators_for_type(ColumnType::Integer);
        let required_int = ["=", "!=", "<", ">", "<=", ">=", "BETWEEN", "IN", "NOT IN"];
        for op in &required_int {
            assert!(
                int_ops.contains(op),
                "AC-003: valid_operators_for_type(Integer) must include '{op}'; got: {int_ops:?}"
            );
        }
        assert_eq!(
            int_ops.len(),
            9,
            "AC-003: Integer must have exactly 9 operators; got: {int_ops:?}"
        );

        // Float: exactly 7 operators
        let float_ops = valid_operators_for_type(ColumnType::Float);
        let required_float = ["=", "!=", "<", ">", "<=", ">=", "BETWEEN"];
        for op in &required_float {
            assert!(
                float_ops.contains(op),
                "AC-003: valid_operators_for_type(Float) must include '{op}'; got: {float_ops:?}"
            );
        }
        assert_eq!(
            float_ops.len(),
            7,
            "AC-003: Float must have exactly 7 operators; got: {float_ops:?}"
        );

        // Boolean: exactly 2 operators
        let bool_ops = valid_operators_for_type(ColumnType::Boolean);
        assert_eq!(
            bool_ops,
            &["=", "!="],
            "AC-003: Boolean must have exactly [=, !=]; got: {bool_ops:?}"
        );

        // Datetime: exactly 7 operators
        let dt_ops = valid_operators_for_type(ColumnType::Datetime);
        let required_dt = ["=", "!=", "<", ">", "<=", ">=", "BETWEEN"];
        for op in &required_dt {
            assert!(
                dt_ops.contains(op),
                "AC-003: valid_operators_for_type(Datetime) must include '{op}'; got: {dt_ops:?}"
            );
        }
        assert_eq!(
            dt_ops.len(),
            7,
            "AC-003: Datetime must have exactly 7 operators; got: {dt_ops:?}"
        );

        // Json: exactly 2 operators
        let json_ops = valid_operators_for_type(ColumnType::Json);
        assert_eq!(
            json_ops,
            &["=", "!="],
            "AC-003: Json must have exactly [=, !=]; got: {json_ops:?}"
        );
    }

    /// BC-2.11.017 / AC-003 — `extract_near_text` extracts the offending token.
    ///
    /// LOAD-BEARING: exercises the production helper. DI-006 truncation at 50 chars
    /// and EC-003 (end-of-input returns "") are verified.
    #[test]
    fn test_BC_2_11_017_enrichment_helper_extract_near_text() {
        use prism_query::engine::extract_near_text;

        // Normal: offset 0 extracts first word "SELCT"
        let near = extract_near_text("SELCT * FROM crowdstrike_alerts", 0);
        assert_eq!(
            near, "SELCT",
            "AC-003: extract_near_text at offset 0 of 'SELCT ...' must return 'SELCT'; \
             got: '{near}'"
        );

        // Mid-string: offset 7 → "sevrity"
        let near2 = extract_near_text("SELECT sevrity FROM", 7);
        assert_eq!(
            near2, "sevrity",
            "AC-003: extract_near_text at offset 7 of 'SELECT sevrity FROM' must return \
             'sevrity'; got: '{near2}'"
        );

        // DI-006: token > 50 chars truncated to exactly 50
        let long_token = "a".repeat(60);
        let long_input = format!("{long_token} rest");
        let near3 = extract_near_text(&long_input, 0);
        assert_eq!(
            near3.len(),
            50,
            "AC-003 DI-006: extract_near_text must truncate to 50 chars for >50-char tokens; \
             got {} chars",
            near3.len()
        );

        // EC-003: offset ≥ input length → empty string
        let near4 = extract_near_text("SELECT *", 100);
        assert_eq!(
            near4, "",
            "AC-003 EC-003: extract_near_text at end-of-input must return ''; got: '{near4}'"
        );
    }

    /// BC-2.11.017 / AC-003 — `how_to_fix_for_security_limit` returns actionable strings.
    ///
    /// LOAD-BEARING: exercises the production helper for all recognized detail categories.
    #[test]
    fn test_BC_2_11_017_enrichment_helper_how_to_fix_for_security_limit() {
        use prism_query::engine::how_to_fix_for_security_limit;

        // Size violation
        let fix_size = how_to_fix_for_security_limit("query size exceeds 64KB limit");
        assert!(
            fix_size.contains("Shorten"),
            "AC-003: how_to_fix for size violation must contain 'Shorten'; got: '{fix_size}'"
        );
        assert!(
            !fix_size.is_empty(),
            "AC-003: how_to_fix must be non-empty for size violation"
        );

        // Depth/nesting violation
        let fix_depth = how_to_fix_for_security_limit("nesting depth limit exceeded");
        assert!(
            fix_depth.contains("Flatten") || fix_depth.contains("nested"),
            "AC-003: how_to_fix for depth violation must reference flattening; got: '{fix_depth}'"
        );

        // Regex violation
        let fix_regex = how_to_fix_for_security_limit("regex complexity limit exceeded");
        assert!(
            fix_regex.contains("regex") || fix_regex.contains("LIKE"),
            "AC-003: how_to_fix for regex violation must reference regex or LIKE; \
             got: '{fix_regex}'"
        );

        // Catch-all: unknown violation → non-empty string
        let fix_other = how_to_fix_for_security_limit("unknown limit violation");
        assert!(
            !fix_other.is_empty(),
            "AC-003: how_to_fix catch-all must return a non-empty string"
        );
    }

    /// BC-2.11.017 / AC-003 — `how_to_fix_for_security_limit` returns the
    /// alias-expansion-specific message for the REAL detail strings emitted by
    /// `AliasResolver::expand` (alias_resolver.rs) and by the explain.rs
    /// expanded-query size check.
    ///
    /// LOAD-BEARING (F-PRL-FRESH-002): the function has a branch-ordering bug where
    /// the alias-expansion detail string (e.g. "expanded query exceeds 64KB limit
    /// (N bytes)") matches the GENERIC size branch (`contains("size")||contains("64kb")`)
    /// BEFORE the dedicated `contains("expanded")||contains("alias")` branch — returning
    /// the wrong message.  These tests MUST FAIL on the buggy code and PASS after the
    /// branch order is corrected.
    ///
    /// Real detail strings under test:
    ///   • `"expanded query exceeds 64KB limit (N bytes)"` — alias_resolver.rs lines 156, 199
    ///   • `"expanded query size N bytes exceeds maximum allowed M bytes"` — explain.rs ~L931
    #[test]
    fn test_BC_2_11_017_how_to_fix_alias_expansion_real_detail_strings() {
        use prism_query::engine::how_to_fix_for_security_limit;

        // ── Case 1: exact alias_resolver.rs emission format (both sites emit this) ──
        // e.g. "expanded query exceeds 64KB limit (65537 bytes)"
        let alias_resolver_detail = "expanded query exceeds 64KB limit (65537 bytes)";
        let fix_alias = how_to_fix_for_security_limit(alias_resolver_detail);
        assert_eq!(
            fix_alias,
            "The alias expansion produced a query over 64KB. Simplify the aliased query or use a narrower alias.",
            "BC-2.11.017 (F-PRL-FRESH-002): real alias_resolver.rs detail string \
             '{alias_resolver_detail}' must map to the alias-expansion-specific \
             how_to_fix message, NOT the generic size message. \
             Branch ordering bug: 'contains(\"64kb\")' fires before 'contains(\"expanded\")'."
        );

        // ── Case 2: explain.rs expanded-query size check format ───────────────────
        // e.g. "expanded query size 65537 bytes exceeds maximum allowed 65536 bytes"
        let explain_detail = "expanded query size 65537 bytes exceeds maximum allowed 65536 bytes";
        let fix_explain = how_to_fix_for_security_limit(explain_detail);
        assert_eq!(
            fix_explain,
            "The alias expansion produced a query over 64KB. Simplify the aliased query or use a narrower alias.",
            "BC-2.11.017 (F-PRL-FRESH-002): explain.rs expanded-query size detail \
             '{explain_detail}' must map to the alias-expansion-specific how_to_fix \
             message. Contains both 'expanded' and 'size' — expanded branch must win."
        );

        // ── Regression: synthetic generic size string (no 'expanded' token) ────────
        // This must STILL return the generic "Shorten" message (not regress).
        let plain_size_detail = "query size exceeds 64KB limit";
        let fix_plain = how_to_fix_for_security_limit(plain_size_detail);
        assert_eq!(
            fix_plain,
            "Shorten the query. Remove large IN (...) lists or break into multiple queries.",
            "BC-2.11.017 regression: plain size detail '{plain_size_detail}' must still \
             return the generic size message after branch reorder."
        );

        // ── Regression: depth violation ───────────────────────────────────────────
        let fix_depth = how_to_fix_for_security_limit("nesting depth limit exceeded");
        assert!(
            fix_depth.contains("Flatten") || fix_depth.contains("nested"),
            "BC-2.11.017 regression: depth detail must still return flatten message; \
             got: '{fix_depth}'"
        );

        // ── Regression: regex violation ───────────────────────────────────────────
        let fix_regex = how_to_fix_for_security_limit("regex complexity limit exceeded");
        assert!(
            fix_regex.contains("regex") || fix_regex.contains("LIKE"),
            "BC-2.11.017 regression: regex detail must still return regex/LIKE message; \
             got: '{fix_regex}'"
        );

        // ── Regression: unknown catch-all ─────────────────────────────────────────
        let fix_other = how_to_fix_for_security_limit("unknown limit violation");
        assert!(
            !fix_other.is_empty(),
            "BC-2.11.017 regression: catch-all must return a non-empty string"
        );
    }

    /// BC-2.11.017 / DI-006 — `extract_near_text` is char-boundary-safe for multibyte UTF-8.
    ///
    /// LOAD-BEARING (F-001-B-FRESH-001 fix): verifies that a token ≥50 BYTES containing
    /// multibyte UTF-8 where byte index 50 falls mid-char does NOT panic and returns a
    /// valid ≤50-CHARACTER result.
    ///
    /// Input construction: "—".repeat(20) (em-dash, U+2014, 3 bytes per char)
    ///   - 20 chars × 3 bytes = 60 bytes total in the token.
    ///   - Valid char boundaries are at multiples of 3 (0, 3, 6, … 48, 51 …).
    ///   - Byte index 50 is NOT a multiple of 3 (50 mod 3 = 2) → NOT a char boundary.
    ///   - The old `&token[..50]` byte-slice panics with "byte index 50 is not a char
    ///     boundary" on this input.
    ///   - The fixed implementation uses `token.chars().take(50).collect::<String>()`,
    ///     which takes up to 50 chars by character iteration (only 20 exist here, so the
    ///     full 20-char token is returned unchanged — all ≤50 chars).
    ///
    /// Additional sub-case: "é".repeat(60) (2-byte chars, 120 bytes)
    ///   - Old byte-slice `&token[..50]` returns exactly 25 chars (25 × 2 = 50 bytes),
    ///     truncating to the wrong count of 25 instead of 50 characters.
    ///   - Fixed implementation returns exactly 50 chars as the BC requires.
    ///
    /// Assertions:
    ///   (a) No panic for em-dash input (structural: test would not reach asserts if it did).
    ///   (b) No panic for é-accent input (same structural guarantee).
    ///   (c) é-accent result is ≤50 characters (char count, not byte count).
    ///   (d) é-accent result equals `"é".repeat(50)` — the expected char-truncated prefix.
    #[test]
    fn test_BC_2_11_017_extract_near_text_multibyte_utf8_no_panic() {
        use prism_query::engine::extract_near_text;

        // ── Sub-case 1: em-dash (3-byte chars) — old code panics here ────────────
        //
        // '—' = U+2014 = 0xE2 0x80 0x94 (3 bytes). 20 × 3 = 60 bytes total.
        // Valid char boundaries: 0, 3, 6, …, 48, 51. Byte 50 (mod 3 = 2) is NOT a
        // char boundary → old `&token[..50]` panics: "byte index 50 is not a char boundary".
        let token_em_dashes = "—".repeat(20);
        let input_em = format!("{token_em_dashes} rest");

        // (a) No panic: reaching this assert proves the call did not panic.
        let result_em = extract_near_text(&input_em, 0);
        assert!(
            result_em.chars().count() <= 50,
            "DI-006 (em-dash): extract_near_text must return ≤50 chars; \
             got {} chars",
            result_em.chars().count()
        );
        // 20 em-dashes < 50 chars, so the whole token is returned untruncated.
        assert_eq!(
            result_em, token_em_dashes,
            "DI-006 (em-dash): token of 20 chars < 50 must be returned whole; \
             got: '{result_em}'"
        );

        // ── Sub-case 2: é-accent (2-byte chars) — old code truncates to wrong count ─
        //
        // 'é' = U+00E9 = 0xC3 0xA9 (2 bytes). 60 × 2 = 120 bytes total.
        // Old `&token[..50]` succeeds (byte 50 = 25×2 = char boundary) but returns
        // 25 chars instead of the 50 chars the spec requires.
        let token_60_e_accents = "é".repeat(60);
        let input_e = format!("{token_60_e_accents} rest");

        // (b) No panic for é-accent input.
        let result_e = extract_near_text(&input_e, 0);

        // (c) ≤50 characters (char count, not byte count).
        let char_count = result_e.chars().count();
        assert!(
            char_count <= 50,
            "DI-006 (é-accent): extract_near_text must return ≤50 CHARACTERS; \
             got {char_count} chars (spec says ≤50 chars, not ≤50 bytes)"
        );
        assert_eq!(
            char_count, 50,
            "DI-006 (é-accent): extract_near_text must truncate a 60-char token to \
             exactly 50 chars; got {char_count} chars (old byte-slice returns only 25)"
        );

        // (d) Content equals the char-truncated prefix.
        let expected_e = "é".repeat(50);
        assert_eq!(
            result_e,
            expected_e,
            "DI-006 (é-accent): extract_near_text must return first 50 'é' chars; \
             got (byte len {}): '{result_e}'",
            result_e.len()
        );
    }

    // =========================================================================
    // F-001B-DC-HIGH-001 — table-qualified column references must NOT false-reject
    // =========================================================================
    //
    // Defect (LOCAL adversary deep-pass finding F-001B-DC-HIGH-001, HIGH):
    //   All four extraction positions in `check_query_column_availability` use
    //   `fp.segments.first()` to extract the column name. For a table-qualified
    //   reference like `crowdstrike_alerts.severity` the parser produces:
    //     FieldPath { segments: ["crowdstrike_alerts", "severity"] }
    //   so `.first()` returns "crowdstrike_alerts" — which is never in
    //   `available_columns` (those hold bare names like "severity") — and the
    //   gate SPURIOUSLY emits E-QUERY-038 with column="crowdstrike_alerts".
    //
    // Fix: a shared helper `extract_column_name_from_field_path` resolves the
    // correct column name at ALL FOUR positions (SELECT, WHERE, GROUP BY, ORDER BY).
    //
    // These tests are RED on HEAD and GREEN after the fix.

    /// F-001B-DC-HIGH-001 (WHERE position) — a table-qualified valid column in the WHERE
    /// clause must NOT produce E-QUERY-038.
    ///
    /// Canonical query: `SELECT * FROM crowdstrike_alerts WHERE crowdstrike_alerts.severity = 'high'`
    /// Before fix: gate extracts "crowdstrike_alerts" as the column name → not in
    /// available_columns → SPURIOUS E-QUERY-038 with column="crowdstrike_alerts".
    /// After fix: extracts "severity" (last segment) → in available_columns → Ok(()).
    #[tokio::test]
    async fn test_BC_2_11_016_qualified_where_valid_column_no_error() {
        use prism_core::column::ColumnType;

        let columns = vec![
            ColumnSpec::new("severity", ColumnType::String, None, vec![]),
            ColumnSpec::new("host_name", ColumnType::String, None, vec![]),
        ];
        let (key, resolved) = make_resolved("crowdstrike", "alerts", columns, "acme");
        let mut map = HashMap::new();
        map.insert(key, resolved.clone());
        let engine = make_engine(map, vec![resolved.spec.clone()]);

        // table-qualified column in WHERE — a VALID column with the table prefix.
        // Before fix: false-rejects with column="crowdstrike_alerts".
        // After fix: passes cleanly (severity IS in available_columns).
        let result = engine
            .execute(
                "SELECT * FROM crowdstrike_alerts WHERE crowdstrike_alerts.severity = 'high'",
                QueryOptions::default(),
            )
            .await;

        match result {
            Err(PrismError::ColumnNotFound(ref d)) => {
                panic!(
                    "F-001B-DC-HIGH-001 (WHERE): qualified valid column \
                     'crowdstrike_alerts.severity' MUST NOT produce E-QUERY-038. \
                     Got column='{}', table='{}', available={:?}. \
                     Root cause: fp.segments.first() returns the table qualifier; \
                     fix: use last segment when qualifier matches FROM table.",
                    d.column, d.table, d.available_columns
                );
            }
            Err(other) => {
                // Any non-ColumnNotFound error is acceptable (e.g. empty results from no adapters).
                // We only care that E-QUERY-038 does NOT fire for a valid qualified ref.
                let msg = format!("{other:?}");
                assert!(
                    !matches!(other, PrismError::ColumnNotFound(_)),
                    "F-001B-DC-HIGH-001: should not reach here; got: {msg}"
                );
            }
            Ok(_) => {
                // PASS — no false rejection; the query ran (with no results because no adapters).
            }
        }
    }

    /// F-001B-DC-HIGH-001 (SELECT position) — a table-qualified valid column in the SELECT
    /// clause must NOT produce E-QUERY-038.
    ///
    /// Canonical query: `SELECT crowdstrike_alerts.severity FROM crowdstrike_alerts`
    /// Before fix: `fp.segments.first()` → "crowdstrike_alerts" → spurious E-QUERY-038.
    /// After fix: extracts "severity" → passes.
    #[tokio::test]
    async fn test_BC_2_11_016_qualified_select_valid_column_no_error() {
        use prism_core::column::ColumnType;

        let columns = vec![
            ColumnSpec::new("severity", ColumnType::String, None, vec![]),
            ColumnSpec::new("host_name", ColumnType::String, None, vec![]),
        ];
        let (key, resolved) = make_resolved("crowdstrike", "alerts", columns, "acme");
        let mut map = HashMap::new();
        map.insert(key, resolved.clone());
        let engine = make_engine(map, vec![resolved.spec.clone()]);

        // table-qualified column in SELECT — valid column.
        let result = engine
            .execute(
                "SELECT crowdstrike_alerts.severity FROM crowdstrike_alerts",
                QueryOptions::default(),
            )
            .await;

        match result {
            Err(PrismError::ColumnNotFound(ref d)) => {
                panic!(
                    "F-001B-DC-HIGH-001 (SELECT): qualified valid column \
                     'crowdstrike_alerts.severity' in SELECT MUST NOT produce E-QUERY-038. \
                     Got column='{}', table='{}'. \
                     Root cause: fp.segments.first() returns table qualifier not column name.",
                    d.column, d.table
                );
            }
            Ok(_) | Err(_) => {
                // PASS — any non-ColumnNotFound outcome is correct here.
                // (Engine has no adapters so Ok with empty results is expected.)
            }
        }
    }

    /// F-001B-DC-HIGH-001 (qualified typo) — a table-qualified MISSPELLED column must
    /// still produce E-QUERY-038, but with column = LAST segment (the typo), NOT
    /// column = the table qualifier.
    ///
    /// Canonical query: `WHERE crowdstrike_alerts.sevrity = 'high'` (sevrity is the typo).
    /// Before fix: column="crowdstrike_alerts", did_you_mean=None (table name → no match).
    /// After fix: column="sevrity", did_you_mean=Some("severity") (Lev-1).
    #[tokio::test]
    async fn test_BC_2_11_016_qualified_where_typo_reports_last_segment_as_column() {
        use prism_core::column::ColumnType;

        let columns = vec![
            ColumnSpec::new("severity", ColumnType::String, None, vec![]),
            ColumnSpec::new("host_name", ColumnType::String, None, vec![]),
        ];
        let (key, resolved) = make_resolved("crowdstrike", "alerts", columns, "acme");
        let mut map = HashMap::new();
        map.insert(key, resolved.clone());
        let engine = make_engine(map, vec![resolved.spec.clone()]);

        // Qualified typo: "crowdstrike_alerts.sevrity" — the typo is in the LAST segment.
        let result = engine
            .execute(
                "SELECT * FROM crowdstrike_alerts WHERE crowdstrike_alerts.sevrity = 'high'",
                QueryOptions::default(),
            )
            .await;

        match result {
            Err(PrismError::ColumnNotFound(ref d)) => {
                assert_eq!(
                    d.column, "sevrity",
                    "F-001B-DC-HIGH-001 (qualified typo): column field MUST be the LAST \
                     segment 'sevrity', not the table qualifier 'crowdstrike_alerts'. \
                     Before fix: column='{}'. Root cause: fp.segments.first().",
                    d.column
                );
                assert_eq!(
                    d.did_you_mean,
                    Some("severity".to_string()),
                    "F-001B-DC-HIGH-001 (qualified typo): did_you_mean MUST be \
                     Some('severity') for Lev-1 typo 'sevrity'; got: {:?}. \
                     Before fix: did_you_mean=None because column='crowdstrike_alerts' \
                     has no close match in available_columns.",
                    d.did_you_mean
                );
                let display = format!("{d}");
                assert!(
                    display.starts_with("E-QUERY-038:"),
                    "F-001B-DC-HIGH-001: Display must start with 'E-QUERY-038:'; got: '{display}'"
                );
            }
            Ok(_) => panic!(
                "F-001B-DC-HIGH-001 (qualified typo): qualified misspelled column \
                 'crowdstrike_alerts.sevrity' MUST produce E-QUERY-038. Got Ok."
            ),
            Err(other) => panic!(
                "F-001B-DC-HIGH-001 (qualified typo): expected ColumnNotFound, got: {other:?}"
            ),
        }
    }

    /// F-001B-DC-HIGH-001 (GROUP BY + ORDER BY) — table-qualified valid columns in GROUP BY
    /// and ORDER BY positions must NOT produce E-QUERY-038.
    ///
    /// Before fix: `fp.segments.first()` → table qualifier → spurious E-QUERY-038.
    /// After fix: extracts the actual column name (last segment) → passes.
    #[tokio::test]
    async fn test_BC_2_11_016_qualified_group_by_order_by_valid_column_no_error() {
        use prism_core::column::ColumnType;

        let columns = vec![
            ColumnSpec::new("severity", ColumnType::String, None, vec![]),
            ColumnSpec::new("host_name", ColumnType::String, None, vec![]),
        ];
        let (key, resolved) = make_resolved("crowdstrike", "alerts", columns, "acme");
        let mut map = HashMap::new();
        map.insert(key, resolved.clone());
        let engine = make_engine(map, vec![resolved.spec.clone()]);

        // GROUP BY with table-qualified column — valid.
        let group_by_result = engine
            .execute(
                "SELECT COUNT(*) FROM crowdstrike_alerts GROUP BY crowdstrike_alerts.severity",
                QueryOptions::default(),
            )
            .await;

        match group_by_result {
            Err(PrismError::ColumnNotFound(ref d)) => {
                panic!(
                    "F-001B-DC-HIGH-001 (GROUP BY): qualified valid column \
                     'crowdstrike_alerts.severity' in GROUP BY MUST NOT produce E-QUERY-038. \
                     Got column='{}'. Root cause: fp.segments.first() extracts table qualifier.",
                    d.column
                );
            }
            Ok(_) | Err(_) => {
                // PASS — any non-ColumnNotFound outcome is correct.
            }
        }

        // ORDER BY with table-qualified column — valid.
        let order_by_result = engine
            .execute(
                "SELECT severity FROM crowdstrike_alerts ORDER BY crowdstrike_alerts.severity",
                QueryOptions::default(),
            )
            .await;

        match order_by_result {
            Err(PrismError::ColumnNotFound(ref d)) => {
                panic!(
                    "F-001B-DC-HIGH-001 (ORDER BY): qualified valid column \
                     'crowdstrike_alerts.severity' in ORDER BY MUST NOT produce E-QUERY-038. \
                     Got column='{}'. Root cause: fp.segments.first() extracts table qualifier.",
                    d.column
                );
            }
            Ok(_) | Err(_) => {
                // PASS — any non-ColumnNotFound outcome is correct.
            }
        }
    }

    // =========================================================================
    // F-FRESH-DC-LOW-001 — ALIAS-qualifier branch coverage
    // =========================================================================
    //
    // Finding: F-FRESH-DC-LOW-001 (LOCAL adversary, LOW, SID-1/TD-VSDD-059).
    //
    // The `extract_column_name_from_field_path` helper has an ALIAS-qualifier branch:
    //
    //   qualifier == table_name || table_alias.is_some_and(|alias| qualifier == alias)
    //
    // Prior tests only exercise the `qualifier == table_name` side (e.g.
    // `crowdstrike_alerts.severity`).  The `table_alias.is_some_and(…)` side — which
    // fires when a query uses `FROM crowdstrike_alerts t WHERE t.severity` — has zero
    // test coverage.
    //
    // These three tests exercise ONLY the alias branch:
    //   • The qualifier is a single-letter alias (`t`), NOT the table name.
    //   • The FROM clause uses the bare-alias form (`FROM crowdstrike_alerts t`) — the
    //     parser accepts both bare and AS-prefixed aliases; bare form exercises the same
    //     `from.alias` field, tested here for conciseness.
    //
    // LOAD-BEARING check (TD-VSDD-059):
    //   If the `table_alias.is_some_and(…)` branch were deleted from
    //   `extract_column_name_from_field_path`, the qualifier `"t"` would NOT match
    //   `table_name = "crowdstrike_alerts"` — the function would return `None` (fail-open)
    //   and the gate would skip that column reference.
    //   Consequence:
    //     • Test 1 (alias valid, no error) — would CONTINUE to pass (fail-open = no gate
    //       = no error), so removing the branch does NOT break test 1.  This is intentional:
    //       test 1 is an EXISTENCE GATE — it proves the branch does not false-reject.
    //     • Test 2 (alias typo → E-QUERY-038) — would FAIL: the deleted branch means
    //       `t.sevrity` is skipped (fail-open), engine returns Ok, test panics.
    //     • Test 3 (alias type gate → E-QUERY-002) — would FAIL: `t.severity > 5` is
    //       skipped by both the existence gate AND the type gate (both rely on the same
    //       helper), engine returns Ok, test panics.
    //
    // Therefore tests 2 and 3 are the true LOAD-BEARING tests that pin the alias branch.
    // Test 1 is the complementary EXISTENCE GATE that verifies the branch does not
    // false-reject valid alias-qualified references.

    /// F-FRESH-DC-LOW-001 — Test 1 (EXISTENCE GATE): valid alias-qualified column in the
    /// WHERE clause must NOT produce E-QUERY-038.
    ///
    /// Query: `SELECT * FROM crowdstrike_alerts t WHERE t.severity = 'high'`
    ///
    /// The qualifier `"t"` matches the FROM-clause alias.
    /// `extract_column_name_from_field_path` must return `"severity"` via the alias branch
    /// (`table_alias.is_some_and(|alias| qualifier == alias)`) so that the existence gate
    /// finds it in `available_columns` and passes without error.
    ///
    /// If the alias branch were ABSENT: `"t"` does not match `table_name =
    /// "crowdstrike_alerts"`, function returns `None`, gate skips the column reference,
    /// engine returns Ok — test still PASSES (fail-open).  This test is therefore the
    /// NON-BREAKING half of the alias coverage; tests 2 and 3 are the BREAKING half.
    #[tokio::test]
    async fn test_BC_2_11_016_alias_qualified_valid_column_no_error() {
        use prism_core::column::ColumnType;

        let columns = vec![
            ColumnSpec::new("severity", ColumnType::String, None, vec![]),
            ColumnSpec::new("host_name", ColumnType::String, None, vec![]),
        ];
        // sensor_id="crowdstrike" + table_suffix="alerts" → registered as "crowdstrike_alerts"
        let (key, resolved) = make_resolved("crowdstrike", "alerts", columns, "acme");
        let mut map = HashMap::new();
        map.insert(key, resolved.clone());
        let engine = make_engine(map, vec![resolved.spec.clone()]);

        // ── Sub-case A: bare alias syntax `FROM crowdstrike_alerts t` ──────────────
        //
        // Parser populates `from.alias = Some("t")`.
        // Column reference `t.severity` → FieldPath{segments:["t","severity"]}.
        // Alias branch: qualifier "t" == alias "t" → returns "severity" → gate passes.
        let result_bare = engine
            .execute(
                "SELECT * FROM crowdstrike_alerts t WHERE t.severity = 'high'",
                QueryOptions::default(),
            )
            .await;

        match result_bare {
            Err(PrismError::ColumnNotFound(ref d)) => {
                panic!(
                    "F-FRESH-DC-LOW-001 (alias valid, bare): alias-qualified valid column \
                     't.severity' MUST NOT produce E-QUERY-038. \
                     Got column='{}', table='{}', available={:?}. \
                     Root cause: alias branch in extract_column_name_from_field_path is \
                     absent or broken.",
                    d.column, d.table, d.available_columns
                );
            }
            Ok(_) | Err(_) => {
                // PASS — no false rejection from E-QUERY-038.
                // (Other errors e.g. no fan-out adapters are acceptable.)
            }
        }

        // ── Sub-case B: AS-alias syntax `FROM crowdstrike_alerts AS t` ──────────────
        //
        // Both AS-prefixed and bare forms populate `from.alias`; sub-case B confirms the
        // parser's AS path also reaches the alias branch.
        let result_as = engine
            .execute(
                "SELECT * FROM crowdstrike_alerts AS t WHERE t.severity = 'high'",
                QueryOptions::default(),
            )
            .await;

        match result_as {
            Err(PrismError::ColumnNotFound(ref d)) => {
                panic!(
                    "F-FRESH-DC-LOW-001 (alias valid, AS): AS-alias-qualified valid column \
                     't.severity' MUST NOT produce E-QUERY-038. \
                     Got column='{}', table='{}'. \
                     Root cause: alias branch in extract_column_name_from_field_path.",
                    d.column, d.table
                );
            }
            Ok(_) | Err(_) => {
                // PASS.
            }
        }

        // ── Sub-case C: alias-qualified in SELECT (`SELECT t.severity FROM … t`) ──
        //
        // Confirms the alias branch is exercised in the SELECT position as well.
        let result_select = engine
            .execute(
                "SELECT t.severity FROM crowdstrike_alerts t",
                QueryOptions::default(),
            )
            .await;

        match result_select {
            Err(PrismError::ColumnNotFound(ref d)) => {
                panic!(
                    "F-FRESH-DC-LOW-001 (alias SELECT): alias-qualified 't.severity' in \
                     SELECT MUST NOT produce E-QUERY-038. \
                     Got column='{}'. \
                     Root cause: alias branch missing in SELECT position of \
                     check_query_column_availability.",
                    d.column
                );
            }
            Ok(_) | Err(_) => {
                // PASS.
            }
        }
    }

    /// F-FRESH-DC-LOW-001 — Test 2 (LOAD-BEARING): alias-qualified MISSPELLED column
    /// must produce E-QUERY-038 with column = the TYPO (last segment), NOT the alias.
    ///
    /// Query: `SELECT * FROM crowdstrike_alerts t WHERE t.sevrity = 'high'`
    ///
    /// The alias branch resolves `["t", "sevrity"]` → `"sevrity"` (last segment).
    /// `"sevrity"` is NOT in `available_columns` → E-QUERY-038 fires with:
    ///   `column = "sevrity"`, `did_you_mean = Some("severity")`.
    ///
    /// LOAD-BEARING: if the alias branch (`table_alias.is_some_and(…)`) were removed,
    /// `extract_column_name_from_field_path` returns `None` (unknown qualifier `"t"`
    /// does not match table_name `"crowdstrike_alerts"`), the existence gate skips the
    /// column reference (fail-open), and the engine returns `Ok(…)` — this test FAILS.
    #[tokio::test]
    async fn test_BC_2_11_016_alias_qualified_typo_reports_last_segment_as_column() {
        use prism_core::column::ColumnType;

        let columns = vec![
            ColumnSpec::new("severity", ColumnType::String, None, vec![]),
            ColumnSpec::new("host_name", ColumnType::String, None, vec![]),
        ];
        // sensor_id="crowdstrike" + table_suffix="alerts" → registered as "crowdstrike_alerts"
        let (key, resolved) = make_resolved("crowdstrike", "alerts", columns, "acme");
        let mut map = HashMap::new();
        map.insert(key, resolved.clone());
        let engine = make_engine(map, vec![resolved.spec.clone()]);

        // "t.sevrity" — the typo is in the LAST segment (the column name).
        // Alias branch: qualifier "t" == alias "t" → column = "sevrity" → not in
        // available_columns → E-QUERY-038 with did_you_mean = "severity".
        let result = engine
            .execute(
                "SELECT * FROM crowdstrike_alerts t WHERE t.sevrity = 'high'",
                QueryOptions::default(),
            )
            .await;

        match result {
            Err(PrismError::ColumnNotFound(ref d)) => {
                // column field MUST be the last segment of the alias-qualified path — the
                // actual typo ("sevrity"), NOT the alias ("t") or the table ("crowdstrike_alerts").
                assert_eq!(
                    d.column, "sevrity",
                    "F-FRESH-DC-LOW-001 (alias typo): column MUST be the last segment \
                     'sevrity', not the alias 't' or the table name. \
                     Without alias branch: extract_column_name_from_field_path returns None \
                     → gate skips → Ok(...) → this test panics at the Ok arm instead. \
                     Got column='{}'",
                    d.column
                );
                assert_eq!(
                    d.did_you_mean,
                    Some("severity".to_string()),
                    "F-FRESH-DC-LOW-001 (alias typo): did_you_mean MUST be Some('severity') \
                     for Lev-1 typo 'sevrity' resolved via alias branch. \
                     Got: {:?}",
                    d.did_you_mean
                );
                let display = format!("{d}");
                assert!(
                    display.starts_with("E-QUERY-038:"),
                    "F-FRESH-DC-LOW-001 (alias typo): Display must start with 'E-QUERY-038:'; \
                     got: '{display}'"
                );
                assert_eq!(
                    d.table, "crowdstrike_alerts",
                    "F-FRESH-DC-LOW-001 (alias typo): table field must be 'crowdstrike_alerts'; \
                     got: '{}'",
                    d.table
                );
            }
            Ok(_) => panic!(
                "F-FRESH-DC-LOW-001 (alias typo) LOAD-BEARING FAIL: alias-qualified misspelled \
                 column 't.sevrity' MUST produce E-QUERY-038. Got Ok instead. \
                 Root cause: alias branch `table_alias.is_some_and(|alias| qualifier == alias)` \
                 is absent or broken in extract_column_name_from_field_path — the qualifier 't' \
                 falls into the fail-open path, gate skips the column reference, no error emitted."
            ),
            Err(other) => panic!(
                "F-FRESH-DC-LOW-001 (alias typo): expected ColumnNotFound for 't.sevrity', \
                 got: {other:?}"
            ),
        }
    }

    /// F-FRESH-DC-LOW-001 — Test 3 (LOAD-BEARING): alias-qualified column with a
    /// type-incompatible operator must produce E-QUERY-002 with correct type metadata.
    ///
    /// Query: `SELECT * FROM crowdstrike_alerts t WHERE t.severity > 5`
    ///
    /// The alias branch resolves `["t", "severity"]` → `"severity"` in BOTH the
    /// existence gate (`collect_predicate_columns`) AND the type-compatibility gate
    /// (`collect_predicate_type_pairs_inner`).  Because `severity` is `ColumnType::String`,
    /// the operator `>` (Gt) is not in `valid_operators_for_type(String)` → E-QUERY-002.
    ///
    /// LOAD-BEARING: if the alias branch were removed from
    /// `extract_column_name_from_field_path`, `collect_predicate_type_pairs_inner`
    /// returns `None` for `["t", "severity"]` → the `(col, op)` pair is NOT pushed
    /// → `check_operator_type_compatibility` is never called → engine returns `Ok(…)`
    /// instead of E-QUERY-002 — this test FAILS.
    ///
    /// This test additionally proves the alias branch wiring reaches
    /// `collect_predicate_type_pairs_inner`, not only `collect_predicate_columns`.
    #[tokio::test]
    async fn test_BC_2_11_017_alias_qualified_type_gate_e_query_002() {
        use prism_core::column::ColumnType;

        let columns = vec![
            // severity is String — the operator `>` is invalid for String (E-QUERY-002).
            ColumnSpec::new("severity", ColumnType::String, None, vec![]),
            ColumnSpec::new("host_name", ColumnType::String, None, vec![]),
        ];
        // sensor_id="crowdstrike" + table_suffix="alerts" → registered as "crowdstrike_alerts"
        let (key, resolved) = make_resolved("crowdstrike", "alerts", columns, "acme");
        let mut map = HashMap::new();
        map.insert(key, resolved.clone());
        let engine = make_engine(map, vec![resolved.spec.clone()]);

        // `t.severity > 5`: severity is String, `>` is not in valid_operators_for_type(String).
        // Alias branch resolves "t.severity" → "severity" → type lookup → Gt not valid → E-QUERY-002.
        let result = engine
            .execute(
                "SELECT * FROM crowdstrike_alerts t WHERE t.severity > 5",
                QueryOptions::default(),
            )
            .await;

        match result {
            // QueryTypeMismatch is an inline variant (not boxed): match by ref fields.
            Err(
                ref e @ PrismError::QueryTypeMismatch {
                    ref column,
                    ref table,
                    ref operator,
                    ..
                },
            ) => {
                // E-QUERY-002 fired — the alias branch reached the type gate.
                assert_eq!(
                    column.as_str(),
                    "severity",
                    "F-FRESH-DC-LOW-001/BC-2.11.017 (alias type gate): E-QUERY-002 column \
                     must be 'severity' (last segment of alias-qualified 't.severity'). \
                     Got: '{column}'"
                );
                assert_eq!(
                    table.as_str(),
                    "crowdstrike_alerts",
                    "F-FRESH-DC-LOW-001/BC-2.11.017 (alias type gate): table must be \
                     'crowdstrike_alerts'; got: '{table}'"
                );
                assert_eq!(
                    operator.as_str(),
                    ">",
                    "F-FRESH-DC-LOW-001/BC-2.11.017 (alias type gate): operator must be '>'; \
                     got: '{operator}'"
                );
                // The Display output carries the E-QUERY-002 prefix — inline variant uses
                // the #[error(...)] template from prism-core/src/error.rs.
                let display = format!("{e}");
                assert!(
                    display.starts_with("E-QUERY-002:"),
                    "F-FRESH-DC-LOW-001/BC-2.11.017 (alias type gate): Display must start \
                     with 'E-QUERY-002:'; got: '{display}'"
                );
            }
            Ok(_) => panic!(
                "F-FRESH-DC-LOW-001/BC-2.11.017 (alias type gate) LOAD-BEARING FAIL: \
                 'SELECT * FROM crowdstrike_alerts t WHERE t.severity > 5' MUST produce \
                 E-QUERY-002 (String column with '>' operator). Got Ok instead. \
                 Root cause: alias branch in extract_column_name_from_field_path is absent — \
                 collect_predicate_type_pairs_inner returns None for 't.severity', \
                 the (col,op) pair is not pushed, check_operator_type_compatibility \
                 is never called, engine silently accepts the type-incompatible query."
            ),
            Err(PrismError::ColumnNotFound(_)) => {
                // The existence gate fired BEFORE the type gate — this would indicate a
                // regression where "severity" is no longer in available_columns.
                panic!(
                    "F-FRESH-DC-LOW-001/BC-2.11.017 (alias type gate): got ColumnNotFound \
                     for 'severity' which IS in the spec — this is a fixture bug or an \
                     alias-branch regression in the existence gate, not the type gate."
                );
            }
            Err(other) => panic!(
                "F-FRESH-DC-LOW-001/BC-2.11.017 (alias type gate): expected \
                 QueryTypeMismatch for 't.severity > 5', got: {other:?}"
            ),
        }
    }

    // =========================================================================
    // GREEN-BY-DESIGN: type shape assertion
    // =========================================================================

    /// GREEN-BY-DESIGN: `ColumnNotFoundDetails` struct exists with correct fields + Display.
    ///
    /// Verifies compile-time type shape only — does NOT assert on production behavior.
    ///
    /// GREEN-BY-DESIGN criteria (BC-5.38.002):
    ///   1. Zero branching in assertion path
    ///   2. No I/O
    ///   3. Tests only type construction + Display shape
    #[test]
    fn test_column_not_found_details_type_check() {
        use prism_core::error::ColumnNotFoundDetails;

        let err = PrismError::ColumnNotFound(Box::new(ColumnNotFoundDetails::new(
            "sevrity",
            "crowdstrike_alerts",
            "acme",
            vec!["severity".to_string()],
            Some("severity".to_string()),
        )));
        let display = format!("{err}");
        assert!(
            display.starts_with("E-QUERY-038:"),
            "ColumnNotFound Display must start with 'E-QUERY-038:'; got: '{display}'"
        );
    }
}
