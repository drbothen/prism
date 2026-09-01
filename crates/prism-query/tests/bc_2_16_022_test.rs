// Test code — allow expect/unwrap per the project pattern for prism-query test files.
// (prism-query Cargo.toml sets expect_used = "deny" / unwrap_used = "deny" for
// production code only; tests use #![allow] to opt out.)
#![allow(clippy::expect_used, clippy::unwrap_used, non_snake_case)]
//! Red Gate tests for BC-2.16.022 §Query-gate — E-QUERY-038 column-gate for
//! `claroty_organization_acl_policies` (story S-CLAROTY-ACLPOLICY-001).
//!
//! Tests: RG-005, RG-006
//!
//! # Red Gate mechanism
//!
//! Both tests call `.find(...).expect(...)` on the loaded claroty.sensor.toml.
//! Pre-implementation the `claroty_organization_acl_policies` table is absent →
//! `.expect()` panics → test FAILS. Post-implementation the table is registered
//! and E-QUERY-038 fires for Tier-2 column names used as SELECT columns.
//!
//! # Table registered name (MED-1 correction)
//!
//! TOML: table_name = "organization_acl_policies" (bare, consistent with sibling tables).
//! sensor_id = "claroty" + table_name = "organization_acl_policies"
//! → TableRegistry-derived name = "claroty_organization_acl_policies"
//! SQL queries must reference this derived form.
//!
//! CONTAMINATION CONTROL: this file MUST NOT read holdout scenario files.

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, fs, path::Path, sync::Arc};

    use prism_core::{OrgSlug, PrismError, SensorId};
    use prism_query::{
        cache::CacheConfig,
        engine::{QueryEngine, QueryEngineConfig, QueryOptions},
        scoping::ClientRegistry,
        table_registry::TableRegistry,
    };
    use prism_spec_engine::{
        overlay::{OverlayLoader, ResolvedSensorSpec, ResolvedSpecKey, SensorInstanceOverlay},
        spec_parser::{SensorSpec, SpecLoader},
    };

    // =========================================================================
    // Fixture helpers
    // =========================================================================

    /// Minimal no-op credential store — prevents CredentialStore trait obj errors in tests.
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

    /// Absolute path to `claroty.sensor.toml` relative to this crate.
    fn claroty_spec_path() -> std::path::PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../prism-sensors/specs/claroty.sensor.toml")
    }

    /// Load and parse `claroty.sensor.toml` via pure TOML deserialization.
    fn load_claroty_spec() -> SensorSpec {
        let content = fs::read_to_string(claroty_spec_path())
            .expect("claroty.sensor.toml must be readable from prism-sensors/specs/");
        SpecLoader::parse(&content).expect("claroty.sensor.toml must be a valid SensorSpec TOML")
    }

    /// Build a `QueryEngine` wired with the actual claroty SensorSpec.
    ///
    /// Builds:
    /// 1. TableRegistry populated from the full sensor spec (all existing tables)
    /// 2. ResolvedSensorSpec overlay for "test-org"
    /// 3. QueryEngine with the resolved spec map and table registry
    ///
    /// sensor_id = "claroty" + table_name = "organization_acl_policies" (bare TOML)
    /// → registered DataFusion name = "claroty_organization_acl_policies"
    fn build_claroty_engine(sensor_spec: &SensorSpec) -> QueryEngine {
        let registry = Arc::new(TableRegistry::new());
        registry
            .register_sensor(sensor_spec)
            .expect("register_sensor must not fail for the claroty spec in test fixture");

        // Build a minimal overlay for the "test-org" tenant.
        let overlay_toml = "extends = \"claroty\"\ninstance_id = \"claroty@test-org\"";
        let overlay: SensorInstanceOverlay =
            toml::from_str(overlay_toml).expect("fixture overlay must parse");
        let org_slug = OrgSlug::new("test-org");
        let resolved: ResolvedSensorSpec =
            OverlayLoader::merge_overlay_onto_type_spec(sensor_spec, &overlay, org_slug.clone());
        let key: ResolvedSpecKey = (org_slug, SensorId::new("claroty"));
        let mut map: HashMap<ResolvedSpecKey, ResolvedSensorSpec> = HashMap::new();
        map.insert(key, resolved);

        QueryEngine::new_with_cache_config(
            Arc::new(prism_sensors::AdapterRegistry::new()),
            Arc::new(NoopCs),
            Arc::new(prism_ocsf::OcsfNormalizer::new()),
            Arc::new(ClientRegistry::new(vec![])),
            QueryEngineConfig::default(),
            CacheConfig::default(),
        )
        .with_resolved_spec_map(Arc::new(map))
        .with_table_registry(registry)
    }

    // =========================================================================
    // RG-005 / AC-005 — policy_source (Tier-2) triggers E-QUERY-038
    // =========================================================================

    /// BC-2.16.022 §Query-gate — `SELECT policy_source FROM claroty_organization_acl_policies`
    /// must return E-QUERY-038 (column not found at plan time).
    ///
    /// `policy_source` is a Tier-2 column (ocsf_field absent) — it is aggregated
    /// into `raw_extensions` and is NOT a first-class projected Arrow column.
    /// The E-QUERY-038 gate fires because "policy_source" is not in the registered
    /// ocsf_projected_column_names for this table.
    ///
    /// Assertions:
    /// - error is PrismError::ColumnNotFound (E-QUERY-038)
    /// - d.column == "policy_source"
    /// - d.table == "claroty_organization_acl_policies" (fully-qualified form)
    /// - d.available_columns contains the FULL Tier-1/synthesized set:
    ///   raw_extensions, metadata_uid, name, actor_user_name, comment, class_uid, _sensor
    ///   (BC-2.16.022 §Invariants — strengthened from raw_extensions-only per LOW finding)
    /// - d.available_columns does NOT contain "policy_source"
    ///
    /// Red Gate pre-implementation: `.find().expect()` panics (table absent) → FAILS.
    /// Red Gate post-TOML-add: E-QUERY-038 fires for Tier-2 column → PASSES.
    #[tokio::test]
    async fn test_BC_2_16_022_claroty_org_acl_policies_policy_source_tier2_e_query_038() {
        let sensor_spec = load_claroty_spec();

        // Red Gate: panics if `claroty_organization_acl_policies` absent from TOML.
        let _table = sensor_spec
            .tables
            .iter()
            .find(|t| t.table_name == "organization_acl_policies")
            .expect(
                "BC-2.16.022 AC-005 RED GATE: claroty_organization_acl_policies must exist in \
                 claroty.sensor.toml. After the table is present, this test verifies that \
                 querying a Tier-2 column by its raw API name returns E-QUERY-038 with \
                 'raw_extensions' in the available_columns hint.",
            );

        let engine = build_claroty_engine(&sensor_spec);

        // E-QUERY-038 is a plan-time gate — fires before any HTTP fan-out.
        // sensor_id="claroty" + table_name="organization_acl_policies" (bare TOML)
        // → DataFusion name = "claroty_organization_acl_policies"
        let result = engine
            .execute(
                "SELECT policy_source FROM claroty_organization_acl_policies LIMIT 1",
                QueryOptions::default(),
            )
            .await;

        match result {
            Err(PrismError::ColumnNotFound(ref d)) => {
                assert_eq!(
                    d.column, "policy_source",
                    "BC-2.16.022 AC-005: E-QUERY-038 MUST report column = 'policy_source'. \
                     Got: '{}'",
                    d.column
                );
                assert_eq!(
                    d.table, "claroty_organization_acl_policies",
                    "BC-2.16.022 AC-005: E-QUERY-038 MUST report table = \
                     'claroty_organization_acl_policies' (fully-qualified). Got: '{}'",
                    d.table
                );
                // Structured-error field assertion (L-3 relabel): ALL Tier-1/synthesized
                // columns must be in available_columns. These assertions check Rust error
                // struct fields (d.available_columns), not serialized MCP response bytes.
                // BC-2.16.022 §Invariants: `available_columns` MUST contain the full set —
                // raw_extensions, metadata_uid, name, actor_user_name, comment, class_uid,
                // _sensor. A regression silently dropping any member of this set would pass
                // if only `raw_extensions` were checked (the pre-fix gap this assertion closes).
                let required_available: &[&str] = &[
                    "raw_extensions",
                    "metadata_uid",
                    "name",
                    "actor_user_name",
                    "comment",
                    "class_uid",
                    "_sensor",
                ];
                for col in required_available {
                    assert!(
                        d.available_columns.contains(&col.to_string()),
                        "BC-2.16.022 AC-005: E-QUERY-038 available_columns MUST contain \
                         '{}' (BC-2.16.022 §Invariants — full Tier-1/synthesized set: \
                         raw_extensions, metadata_uid, name, actor_user_name, comment, \
                         class_uid, _sensor). available_columns: {:?}",
                        col,
                        d.available_columns
                    );
                }
                // Structured-error field assertion: "policy_source" must NOT be in
                // available_columns. Raw Tier-2 TOML column names are NOT projected as
                // standalone Arrow columns.
                assert!(
                    !d.available_columns.contains(&"policy_source".to_string()),
                    "BC-2.16.022 AC-005: E-QUERY-038 available_columns MUST NOT contain \
                     'policy_source' (raw API field names are not projected). \
                     available_columns: {:?}",
                    d.available_columns
                );
            }
            Err(other) => panic!(
                "BC-2.16.022 AC-005: expected E-QUERY-038 (ColumnNotFound) for Tier-2 column \
                 'policy_source', got a different error: {:?}",
                other
            ),
            Ok(_) => panic!(
                "BC-2.16.022 AC-005: expected E-QUERY-038 (ColumnNotFound) for Tier-2 column \
                 'policy_source' — column must NOT be a first-class projected name \
                 when ocsf_column_naming = true. Got Ok instead of Err."
            ),
        }
    }

    // =========================================================================
    // RG-006 / AC-006 — policy_id (raw API name) NOT projected; metadata_uid is
    // =========================================================================

    /// BC-2.16.022 §Query-gate — `SELECT policy_id FROM claroty_organization_acl_policies`
    /// must return E-QUERY-038 because the projected Arrow column name is "metadata_uid"
    /// (via ocsf_field = "metadata.uid" → dot→underscore flattening), NOT "policy_id".
    ///
    /// Assertions:
    /// - error is PrismError::ColumnNotFound (E-QUERY-038)
    /// - d.column == "policy_id" (the raw API name used in the query)
    /// - d.table == "claroty_organization_acl_policies" (fully-qualified)
    /// - d.available_columns contains "metadata_uid" (the correct OCSF-projected name)
    /// - d.available_columns does NOT contain "policy_id" (raw name NOT projected)
    ///
    /// Wire-shape assertion (SID-2): assert on the full E-QUERY-038 error string
    /// to verify that "metadata_uid" appears in the human-readable suggestion.
    ///
    /// Red Gate pre-implementation: `.find().expect()` panics (table absent) → FAILS.
    /// Red Gate post-TOML-add: E-QUERY-038 fires; "metadata_uid" in available_columns.
    #[tokio::test]
    async fn test_BC_2_16_022_claroty_org_acl_policies_policy_id_raw_name_not_projected_metadata_uid_is(
    ) {
        let sensor_spec = load_claroty_spec();

        // Red Gate: panics if `claroty_organization_acl_policies` absent from TOML.
        let _table = sensor_spec
            .tables
            .iter()
            .find(|t| t.table_name == "organization_acl_policies")
            .expect(
                "BC-2.16.022 AC-006 RED GATE: claroty_organization_acl_policies must exist in \
                 claroty.sensor.toml. After the table is present, this test verifies that \
                 querying the raw API name 'policy_id' returns E-QUERY-038 with \
                 'metadata_uid' (the OCSF-projected name) in available_columns.",
            );

        let engine = build_claroty_engine(&sensor_spec);

        let result = engine
            .execute(
                "SELECT policy_id FROM claroty_organization_acl_policies LIMIT 1",
                QueryOptions::default(),
            )
            .await;

        match result {
            Err(PrismError::ColumnNotFound(ref d)) => {
                assert_eq!(
                    d.column, "policy_id",
                    "BC-2.16.022 AC-006: E-QUERY-038 MUST report column = 'policy_id' \
                     (the raw API name used in the query). Got: '{}'",
                    d.column
                );
                assert_eq!(
                    d.table, "claroty_organization_acl_policies",
                    "BC-2.16.022 AC-006: E-QUERY-038 MUST report table = \
                     'claroty_organization_acl_policies' (fully-qualified). Got: '{}'",
                    d.table
                );
                // Structured-error field assertion: "metadata_uid" must be in
                // available_columns. This tells the LLM agent to use "metadata_uid"
                // instead of "policy_id".
                // ocsf_field_to_arrow_name("metadata.uid") = "metadata_uid".
                assert!(
                    d.available_columns.contains(&"metadata_uid".to_string()),
                    "BC-2.16.022 AC-006: E-QUERY-038 available_columns MUST contain \
                     'metadata_uid' (policy_id → ocsf_field 'metadata.uid' → \
                     Arrow name 'metadata_uid' via dot→underscore). \
                     available_columns: {:?}",
                    d.available_columns
                );
                // Structured-error field assertion: "policy_id" must NOT be in
                // available_columns. Raw API field names are NOT projected under
                // ocsf_column_naming = true.
                assert!(
                    !d.available_columns.contains(&"policy_id".to_string()),
                    "BC-2.16.022 AC-006: E-QUERY-038 available_columns MUST NOT contain \
                     'policy_id' (raw API name not projected under ocsf_column_naming = true). \
                     available_columns: {:?}",
                    d.available_columns
                );
                // SID-2: assert on the full composed E-QUERY-038 error string.
                // The error message must not duplicate field names or produce garbled output.
                let error_string = format!("{}", d);
                assert!(
                    error_string.contains("policy_id"),
                    "BC-2.16.022 AC-006 (SID-2): E-QUERY-038 Display MUST mention 'policy_id'. \
                     Got: {}",
                    error_string
                );
                assert!(
                    error_string.contains("claroty_organization_acl_policies"),
                    "BC-2.16.022 AC-006 (SID-2): E-QUERY-038 Display MUST mention the table name. \
                     Got: {}",
                    error_string
                );
            }
            Err(other) => panic!(
                "BC-2.16.022 AC-006: expected E-QUERY-038 (ColumnNotFound) for raw API name \
                 'policy_id' (projected as 'metadata_uid' under ocsf_column_naming=true). \
                 Got a different error: {:?}",
                other
            ),
            Ok(_) => panic!(
                "BC-2.16.022 AC-006: expected E-QUERY-038 (ColumnNotFound) for 'policy_id' \
                 (raw API field name is not a projected column under ocsf_column_naming=true). \
                 Got Ok instead of Err."
            ),
        }
    }
}
