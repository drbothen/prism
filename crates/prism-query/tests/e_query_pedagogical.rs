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
//! - BC-2.11.016 v1.0 — E-QUERY-038 Column-Not-Found Plan-Time Gate
//! - BC-2.11.017 v1.0 — E-QUERY Pedagogical Enrichments

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

        let mut engine = QueryEngine::new_with_cache_config(
            Arc::new(prism_sensors::AdapterRegistry::new()),
            Arc::new(NoopCs),
            Arc::new(prism_ocsf::OcsfNormalizer::new()),
            Arc::new(ClientRegistry::new(clients)),
            QueryEngineConfig::default(),
            prism_query::cache::CacheConfig::default(),
        );
        engine.resolved_spec_map = Some(Arc::new(arc_swap::ArcSwap::new(Arc::new(resolved_map))));
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
            Ok(_) => panic!(
                "gate-ordering: must return TableNotAvailable for 'nonexistent_table'."
            ),
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

        // String: exactly 5 operators
        let string_ops = valid_operators_for_type(ColumnType::String);
        let required_string = ["=", "!=", "LIKE", "IN", "NOT IN"];
        for op in &required_string {
            assert!(
                string_ops.contains(op),
                "AC-003: valid_operators_for_type(String) must include '{op}'; got: {string_ops:?}"
            );
        }
        assert_eq!(
            string_ops.len(),
            5,
            "AC-003: String must have exactly 5 operators; got: {string_ops:?}"
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
