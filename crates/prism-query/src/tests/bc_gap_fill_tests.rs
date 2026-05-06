//! BC Gap-fill tests for S-3.02 — Query Materialization.
//!
//! This module adds missing coverage identified in the test-writer gap-fill
//! audit. The stub-architect's 14 tests in `integration_tests.rs` scaffold
//! AC-1..AC-9. This file covers:
//!
//! # Categories
//!
//! ## VP-031 Extended Proptest Coverage
//! - INDEX columns always produce push-down (BC-2.11.007)
//! - ADDITIONAL columns always produce push-down (BC-2.11.007)
//! - OPTIMIZED columns NEVER produce push-down (BC-2.11.007)
//! - DEFAULT columns NEVER produce push-down (BC-2.11.007)
//! - Mixed predicate list: REQUIRED + DEFAULT → correct split (BC-2.11.007)
//! - Empty predicate list: both push_down and post_filter are empty (BC-2.11.007)
//!
//! ## BC-2.11.007 Pushdown Classification — Concrete Unit Tests
//! - `test_BC_2_11_007_required_column_in_push_down` (REQUIRED → always push)
//! - `test_BC_2_11_007_index_column_in_push_down` (INDEX → always push)
//! - `test_BC_2_11_007_additional_column_in_push_down` (ADDITIONAL → always push)
//! - `test_BC_2_11_007_optimized_column_in_post_filter` (OPTIMIZED → post-filter)
//! - `test_BC_2_11_007_default_column_in_post_filter` (DEFAULT → post-filter)
//! - `test_BC_2_11_007_unknown_column_defaults_to_post_filter`
//! - `test_BC_2_11_007_translation_failure_falls_back_to_post_filter`
//! - `test_BC_2_11_007_rejects_query_missing_required_column_e_query_009`
//! - `test_BC_2_11_007_empty_predicates_returns_empty_plan`
//! - `test_BC_2_11_007_mixed_predicates_split_correctly`
//!
//! ## BC-2.11.011 Cross-Client Scoping
//! - `test_BC_2_11_011_resolve_clients_none_returns_all`
//! - `test_BC_2_11_011_resolve_clients_some_valid_list`
//! - `test_BC_2_11_011_resolve_clients_invalid_id_returns_error`
//! - `test_BC_2_11_011_intersect_predicates_narrows_scope`
//! - `test_BC_2_11_011_intersect_predicates_empty_result_not_error` (EC-11-001)
//! - `test_BC_2_11_011_intersect_predicates_cannot_widen_scope` (EC-11-028)
//! - `test_BC_2_11_011_single_client_registry_resolve`
//! - `test_BC_2_11_011_empty_registry_none_returns_empty`
//!
//! ## BC-2.11.012 Virtual Field Injection
//! - `test_BC_2_11_012_inject_virtual_fields_adds_all_three_columns`
//! - `test_BC_2_11_012_inject_virtual_fields_overwrites_sensor_spoofed_column` (EC-005)
//! - `test_BC_2_11_012_inject_virtual_fields_idempotent_second_call`
//! - `test_BC_2_11_012_remove_spoofed_columns_strips_all_reserved_names`
//! - `test_BC_2_11_012_sensor_type_to_string_crowdstrike`
//! - `test_BC_2_11_012_sensor_type_to_string_armis`
//! - `test_BC_2_11_012_virtual_field_names_are_correct_constants`
//!
//! ## BC-2.11.006 Security Limits — Additional Coverage
//! - `test_BC_2_11_006_memory_pool_constant_is_200mb`
//! - `test_BC_2_11_006_record_cap_constant_is_10k`
//! - `test_BC_2_11_006_timeout_constant_is_30s`
//! - `test_BC_2_11_006_ec001_memory_pool_limit_no_partial_results` (EC-001)
//! - `test_BC_2_11_006_ec004_panic_path_session_scope_drops_context` (EC-004)
//! - `test_BC_2_11_006_ec002_timeout_returns_query_timeout_not_execution_error`
//! - `test_BC_2_11_006_ec003_record_cap_message_includes_count_and_sources`
//!
//! ## BC-2.11.005 Ephemeral Materialization
//! - `test_BC_2_11_005_session_scope_drops_context_on_normal_return`
//! - `test_BC_2_11_005_in_query_cache_avoids_redundant_api_calls`
//! - `test_BC_2_11_005_partial_failure_returns_sensor_errors_not_full_abort`
//! - `test_BC_2_11_005_empty_all_sensors_returns_empty_result_not_error`
//!
//! ## BC-2.11.001 Tool Interface
//! - `test_BC_2_11_001_result_truncation_is_truncated_true_when_over_limit`
//! - `test_BC_2_11_001_result_not_truncated_when_under_limit`
//! - `test_BC_2_11_001_query_context_contains_original_query_string`
//!
//! All tests in this module are RED by design unless explicitly marked
//! GREEN-BY-DESIGN — the stubs they exercise are `todo!()`.
//!
//! # BC References
//! - BC-2.11.001 — `query` MCP Tool
//! - BC-2.11.005 — Ephemeral Materialization
//! - BC-2.11.006 — Query Security Limits
//! - BC-2.11.007 — Sensor Filter Push-Down
//! - BC-2.11.011 — Cross-Client Query Scoping
//! - BC-2.11.012 — Virtual Fields
//! - VP-031 — Required column enforcement (proptest)
//!
//! Story: S-3.02

// Red Gate test file: all non-trivial bodies are todo!() per BC-5.38.001.
// These suppressions are intentional — imports are referenced in todo!() bodies
// that diverge before they are used, and the unreachable / diverge warnings are
// a consequence of the todo!() macro expanding to `panic!()`.
#[allow(
    unused_imports,
    unreachable_code,
    clippy::diverging_sub_expression,
    dead_code
)]
#[cfg(test)]
mod bc_gap_fill {

    // =========================================================================
    // VP-031 EXTENDED PROPTEST COVERAGE
    // =========================================================================

    mod vp031_extended {
        use proptest::prelude::*;

        proptest! {
            /// VP-031 extension: INDEX columns are pushed down, never post-filter.
            ///
            /// BC-2.11.007: INDEX columns represent native sensor API filter
            /// parameters and MUST always be pushed down.
            ///
            /// RED by design — `classify_predicates` is `todo!()`.
            #[test]
            fn prop_BC_2_11_007_index_columns_always_push_down(
                _column_name in prop_oneof![
                    Just("severity"),
                    Just("created_at"),
                    Just("status"),
                    Just("alert_type"),
                ]
            ) {
                todo!("S-3.02 — prop_BC_2_11_007_index_columns_always_push_down")
            }
        }

        proptest! {
            /// VP-031 extension: ADDITIONAL columns are pushed down.
            ///
            /// BC-2.11.007: ADDITIONAL columns use secondary API filtering
            /// and MUST be pushed down when present.
            ///
            /// RED by design — `classify_predicates` is `todo!()`.
            #[test]
            fn prop_BC_2_11_007_additional_columns_always_push_down(
                _column_name in prop_oneof![
                    Just("resolved"),
                    Just("include_resolved"),
                    Just("with_details"),
                ]
            ) {
                todo!("S-3.02 — prop_BC_2_11_007_additional_columns_always_push_down")
            }
        }

        proptest! {
            /// VP-031 extension: OPTIMIZED columns NEVER appear in push_down.
            ///
            /// BC-2.11.007: OPTIMIZED columns are locally optimized; sensor APIs
            /// do not support them. They MUST always be post-filter.
            ///
            /// RED by design — `classify_predicates` is `todo!()`.
            #[test]
            fn prop_BC_2_11_007_optimized_columns_never_push_down(
                _column_name in prop_oneof![
                    Just("device.hostname"),
                    Just("device.ip"),
                    Just("ocsf_class_uid"),
                ]
            ) {
                todo!("S-3.02 — prop_BC_2_11_007_optimized_columns_never_push_down")
            }
        }

        proptest! {
            /// VP-031 extension: DEFAULT columns NEVER appear in push_down.
            ///
            /// BC-2.11.007: DEFAULT columns have no push-down support.
            /// They MUST always end up in post_filter.
            ///
            /// RED by design — `classify_predicates` is `todo!()`.
            #[test]
            fn prop_BC_2_11_007_default_columns_never_push_down(
                _column_name in prop_oneof![
                    Just("description"),
                    Just("event_data"),
                    Just("raw_payload"),
                ]
            ) {
                todo!("S-3.02 — prop_BC_2_11_007_default_columns_never_push_down")
            }
        }

        proptest! {
            /// VP-031 extension: empty predicate list produces empty plan.
            ///
            /// BC-2.11.007: When no predicates are present, both push_down and
            /// post_filter must be empty. No false positives.
            ///
            /// RED by design — `classify_predicates` is `todo!()`.
            #[test]
            fn prop_BC_2_11_007_empty_predicate_list_both_empty(
                _seed in 0u32..1000u32
            ) {
                todo!("S-3.02 — prop_BC_2_11_007_empty_predicate_list_both_empty")
            }
        }

        proptest! {
            /// VP-031 extension: Mixed predicates split correctly.
            ///
            /// BC-2.11.007: When a WHERE clause has both REQUIRED and DEFAULT
            /// columns, REQUIRED ends up in push_down, DEFAULT in post_filter.
            ///
            /// RED by design — `classify_predicates` is `todo!()`.
            #[test]
            fn prop_BC_2_11_007_mixed_predicates_split_correctly(
                _required_col in prop_oneof![
                    Just("customer_id"),
                    Just("org_id"),
                ],
                _default_col in prop_oneof![
                    Just("description"),
                    Just("raw_payload"),
                ]
            ) {
                todo!("S-3.02 — prop_BC_2_11_007_mixed_predicates_split_correctly")
            }
        }
    }

    // =========================================================================
    // BC-2.11.007 PUSHDOWN CLASSIFICATION — CONCRETE UNIT TESTS
    // =========================================================================

    mod pushdown_classification {

        /// BC-2.11.007: REQUIRED column predicate MUST appear in push_down.
        ///
        /// AC-4: `severity_id >= 3` (REQUIRED on CrowdStrike) must be pushed.
        ///
        /// RED by design — `classify_predicates` is `todo!()`.
        #[test]
        fn test_BC_2_11_007_required_column_in_push_down() {
            todo!("S-3.02 — test_BC_2_11_007_required_column_in_push_down")
        }

        /// BC-2.11.007: INDEX column predicate MUST appear in push_down.
        ///
        /// INDEX columns have native API filter support; they MUST be pushed.
        ///
        /// RED by design — `classify_predicates` is `todo!()`.
        #[test]
        fn test_BC_2_11_007_index_column_in_push_down() {
            todo!("S-3.02 — test_BC_2_11_007_index_column_in_push_down")
        }

        /// BC-2.11.007: ADDITIONAL column predicate MUST appear in push_down.
        ///
        /// ADDITIONAL columns use secondary API filtering; they MUST be pushed.
        ///
        /// RED by design — `classify_predicates` is `todo!()`.
        #[test]
        fn test_BC_2_11_007_additional_column_in_push_down() {
            todo!("S-3.02 — test_BC_2_11_007_additional_column_in_push_down")
        }

        /// BC-2.11.007: OPTIMIZED column predicate MUST appear in post_filter.
        ///
        /// OPTIMIZED columns are locally optimized; the sensor does not support
        /// them as API filters. They MUST stay in post_filter.
        ///
        /// RED by design — `classify_predicates` is `todo!()`.
        #[test]
        fn test_BC_2_11_007_optimized_column_in_post_filter() {
            todo!("S-3.02 — test_BC_2_11_007_optimized_column_in_post_filter")
        }

        /// BC-2.11.007: DEFAULT column predicate MUST appear in post_filter.
        ///
        /// DEFAULT columns have no push-down support; DataFusion applies them
        /// after materialization.
        ///
        /// RED by design — `classify_predicates` is `todo!()`.
        #[test]
        fn test_BC_2_11_007_default_column_in_post_filter() {
            todo!("S-3.02 — test_BC_2_11_007_default_column_in_post_filter")
        }

        /// BC-2.11.007: Unknown column name MUST default to post_filter.
        ///
        /// "When in doubt, classify as PostFilter." (BC-2.11.007 conservative
        /// fallback rule)
        ///
        /// RED by design — `column_push_down_option` is `todo!()`.
        #[test]
        fn test_BC_2_11_007_unknown_column_defaults_to_post_filter() {
            todo!("S-3.02 — test_BC_2_11_007_unknown_column_defaults_to_post_filter")
        }

        /// BC-2.11.007: Translation failure falls back to post_filter with WARN.
        ///
        /// "If translation fails, log warning and fall back to post-filter."
        /// (BC-2.11.007 error cases table)
        ///
        /// RED by design — `translate_push_down_filter` is `todo!()`.
        #[test]
        fn test_BC_2_11_007_translation_failure_falls_back_to_post_filter() {
            todo!("S-3.02 — test_BC_2_11_007_translation_failure_falls_back_to_post_filter")
        }

        /// BC-2.11.007 / VP-031: Empty predicates → empty push_down and post_filter.
        ///
        /// No predicates means no filtering; both lists must be empty.
        ///
        /// RED by design — `classify_predicates` is `todo!()`.
        #[test]
        fn test_BC_2_11_007_empty_predicates_returns_empty_plan() {
            todo!("S-3.02 — test_BC_2_11_007_empty_predicates_returns_empty_plan")
        }

        /// BC-2.11.007: Mixed predicate list splits REQUIRED and DEFAULT correctly.
        ///
        /// Given [REQUIRED, DEFAULT], push_down has 1 entry, post_filter has 1 entry.
        ///
        /// RED by design — `classify_predicates` is `todo!()`.
        #[test]
        fn test_BC_2_11_007_mixed_predicates_split_correctly() {
            todo!("S-3.02 — test_BC_2_11_007_mixed_predicates_split_correctly")
        }

        /// BC-2.11.007 E-QUERY-009: Query missing REQUIRED column is rejected
        /// before any API calls are made.
        ///
        /// "Query rejected with E-QUERY-009 if a REQUIRED column is not constrained."
        ///
        /// RED by design — enforcement logic is `todo!()`.
        #[test]
        fn test_BC_2_11_007_rejects_query_missing_required_column_e_query_009() {
            todo!("S-3.02 — test_BC_2_11_007_rejects_query_missing_required_column_e_query_009")
        }
    }

    // =========================================================================
    // BC-2.11.011 CROSS-CLIENT SCOPING
    // =========================================================================

    mod cross_client_scoping {
        use prism_core::OrgSlug;

        use crate::scoping::{intersect_query_client_predicates, resolve_clients, ClientRegistry};

        fn make_slug(s: &str) -> OrgSlug {
            OrgSlug::new(s)
        }

        fn make_registry(slugs: &[&str]) -> ClientRegistry {
            ClientRegistry::new(slugs.iter().map(|s| make_slug(s)).collect())
        }

        /// BC-2.11.011: `clients: None` returns all configured clients.
        ///
        /// AC-5 unit path: when the tool passes `None`, the engine fans out to
        /// every client in the registry.
        ///
        /// RED by design — `resolve_clients` is `todo!()`.
        #[test]
        fn test_BC_2_11_011_resolve_clients_none_returns_all() {
            todo!("S-3.02 — test_BC_2_11_011_resolve_clients_none_returns_all")
        }

        /// BC-2.11.011: `clients: Some(["acme"])` returns only `["acme"]`.
        ///
        /// When an explicit list is provided, only those clients are resolved.
        ///
        /// RED by design — `resolve_clients` is `todo!()`.
        #[test]
        fn test_BC_2_11_011_resolve_clients_some_valid_list() {
            todo!("S-3.02 — test_BC_2_11_011_resolve_clients_some_valid_list")
        }

        /// BC-2.11.011 / BC-2.11.001 E-MCP-004: invalid client ID returns error.
        ///
        /// "InvalidClientId / E-AUTH-003 if any client ID not found in registry."
        ///
        /// RED by design — `resolve_clients` is `todo!()`.
        #[test]
        fn test_BC_2_11_011_resolve_clients_invalid_id_returns_error() {
            todo!("S-3.02 — test_BC_2_11_011_resolve_clients_invalid_id_returns_error")
        }

        /// BC-2.11.011: intersect_query_client_predicates narrows scope correctly.
        ///
        /// Tool scope ["acme", "contoso"] ∩ query predicate ["acme"] = ["acme"].
        ///
        /// RED by design — `intersect_query_client_predicates` is `todo!()`.
        #[test]
        fn test_BC_2_11_011_intersect_predicates_narrows_scope() {
            todo!("S-3.02 — test_BC_2_11_011_intersect_predicates_narrows_scope")
        }

        /// BC-2.11.011 EC-11-001: Empty intersection produces empty Vec, not error.
        ///
        /// "`clients: ["acme"]` but query has `client_id = "globex"` → intersection
        /// is empty; return empty result set." (BC-2.11.011 edge case)
        ///
        /// RED by design — `intersect_query_client_predicates` is `todo!()`.
        #[test]
        fn test_BC_2_11_011_intersect_predicates_empty_result_not_error() {
            todo!("S-3.02 — test_BC_2_11_011_intersect_predicates_empty_result_not_error")
        }

        /// BC-2.11.011 EC-11-028: Query predicates cannot WIDEN scope.
        ///
        /// Tool scope ["acme"] + query `client_id = "acme" OR client_id = "globex"`
        /// → intersection is ["acme"] only; "globex" is silently excluded.
        ///
        /// RED by design — `intersect_query_client_predicates` is `todo!()`.
        #[test]
        fn test_BC_2_11_011_intersect_predicates_cannot_widen_scope() {
            todo!("S-3.02 — test_BC_2_11_011_intersect_predicates_cannot_widen_scope")
        }

        /// BC-2.11.011: Single-client registry resolves correctly.
        ///
        /// Edge case: a single configured client resolved when clients=None.
        ///
        /// RED by design — `resolve_clients` is `todo!()`.
        #[test]
        fn test_BC_2_11_011_single_client_registry_resolve() {
            todo!("S-3.02 — test_BC_2_11_011_single_client_registry_resolve")
        }

        /// BC-2.11.011: Empty registry + clients=None returns empty Vec.
        ///
        /// No configured clients → zero fan-out targets; empty result set.
        ///
        /// RED by design — `resolve_clients` is `todo!()`.
        #[test]
        fn test_BC_2_11_011_empty_registry_none_returns_empty() {
            todo!("S-3.02 — test_BC_2_11_011_empty_registry_none_returns_empty")
        }
    }

    // =========================================================================
    // BC-2.11.012 VIRTUAL FIELD INJECTION
    // =========================================================================

    mod virtual_field_injection {
        use crate::virtual_fields::{
            inject_virtual_fields, remove_spoofed_virtual_columns, sensor_type_to_string,
            VIRTUAL_FIELD_CLIENT, VIRTUAL_FIELD_SENSOR, VIRTUAL_FIELD_SOURCE_TABLE,
        };

        /// BC-2.11.012: inject_virtual_fields adds all three columns to the batch.
        ///
        /// Given a RecordBatch with no virtual fields, after injection it must
        /// have `_sensor`, `_client`, and `_source_table` columns present.
        ///
        /// RED by design — `inject_virtual_fields` is `todo!()`.
        #[test]
        fn test_BC_2_11_012_inject_virtual_fields_adds_all_three_columns() {
            todo!("S-3.02 — test_BC_2_11_012_inject_virtual_fields_adds_all_three_columns")
        }

        /// BC-2.11.012 EC-005: Engine overwrites sensor-emitted `_sensor` column.
        ///
        /// "If sensor emits a field named `_sensor`, engine overwrites it with
        /// canonical virtual field value; sensor-emitted value discarded."
        ///
        /// RED by design — `inject_virtual_fields` is `todo!()`.
        #[test]
        fn test_BC_2_11_012_inject_virtual_fields_overwrites_sensor_spoofed_column() {
            todo!(
                "S-3.02 — test_BC_2_11_012_inject_virtual_fields_overwrites_sensor_spoofed_column"
            )
        }

        /// BC-2.11.012: Calling inject_virtual_fields twice is idempotent.
        ///
        /// The second call must overwrite with the same values. The batch must
        /// still have exactly one column per virtual field name (no duplicates).
        ///
        /// RED by design — `inject_virtual_fields` is `todo!()`.
        #[test]
        fn test_BC_2_11_012_inject_virtual_fields_idempotent_second_call() {
            todo!("S-3.02 — test_BC_2_11_012_inject_virtual_fields_idempotent_second_call")
        }

        /// BC-2.11.012: remove_spoofed_virtual_columns strips all three reserved names.
        ///
        /// A batch with columns named `_sensor`, `_client`, `_source_table` must
        /// have all three removed, leaving other columns intact.
        ///
        /// RED by design — `remove_spoofed_virtual_columns` is `todo!()`.
        #[test]
        fn test_BC_2_11_012_remove_spoofed_columns_strips_all_reserved_names() {
            todo!("S-3.02 — test_BC_2_11_012_remove_spoofed_columns_strips_all_reserved_names")
        }

        /// BC-2.11.012: sensor_type_to_string(CrowdStrike) returns "crowdstrike".
        ///
        /// Virtual field `_sensor` value must be the lowercase sensor type string.
        ///
        /// RED by design — `sensor_type_to_string` is `todo!()`.
        #[test]
        fn test_BC_2_11_012_sensor_type_to_string_crowdstrike() {
            todo!("S-3.02 — test_BC_2_11_012_sensor_type_to_string_crowdstrike")
        }

        /// BC-2.11.012: sensor_type_to_string(Armis) returns "armis".
        ///
        /// Verifies the Armis sensor type produces its canonical string.
        ///
        /// RED by design — `sensor_type_to_string` is `todo!()`.
        #[test]
        fn test_BC_2_11_012_sensor_type_to_string_armis() {
            todo!("S-3.02 — test_BC_2_11_012_sensor_type_to_string_armis")
        }

        /// BC-2.11.012: Virtual field name constants have correct values.
        ///
        /// GREEN-BY-DESIGN: structural test documenting BC-2.11.012's field name
        /// contract. Constants are already defined (not `todo!()`), so this passes
        /// immediately. Included to catch accidental renames.
        #[test]
        fn test_BC_2_11_012_virtual_field_names_are_correct_constants() {
            assert_eq!(
                VIRTUAL_FIELD_SENSOR, "_sensor",
                "BC-2.11.012: _sensor constant must have value \"_sensor\""
            );
            assert_eq!(
                VIRTUAL_FIELD_CLIENT, "_client",
                "BC-2.11.012: _client constant must have value \"_client\""
            );
            assert_eq!(
                VIRTUAL_FIELD_SOURCE_TABLE, "_source_table",
                "BC-2.11.012: _source_table constant must have value \"_source_table\""
            );
        }
    }

    // =========================================================================
    // BC-2.11.006 SECURITY LIMITS — ADDITIONAL COVERAGE
    // =========================================================================

    mod security_limits {
        use crate::memory::{
            MAX_MATERIALIZED_RECORDS, QUERY_MEMORY_POOL_BYTES, QUERY_TIMEOUT_SECS,
        };

        /// BC-2.11.006: Memory pool constant is exactly 200 MB.
        ///
        /// GREEN-BY-DESIGN: structural test documenting the BC-2.11.006 contract
        /// value. The constant is already defined, so this passes immediately.
        /// Catches accidental constant changes.
        #[test]
        fn test_BC_2_11_006_memory_pool_constant_is_200mb() {
            assert_eq!(
                QUERY_MEMORY_POOL_BYTES,
                200 * 1024 * 1024,
                "BC-2.11.006: per-query memory pool MUST be exactly 200MB (209715200 bytes)"
            );
        }

        /// BC-2.11.006: Record cap constant is exactly 10,000 records.
        ///
        /// GREEN-BY-DESIGN: structural test documenting the BC-2.11.006 contract
        /// value. Catches accidental constant changes.
        #[test]
        fn test_BC_2_11_006_record_cap_constant_is_10k() {
            assert_eq!(
                MAX_MATERIALIZED_RECORDS, 10_000,
                "BC-2.11.006: materialization record cap MUST be exactly 10,000"
            );
        }

        /// BC-2.11.006: Timeout constant is exactly 30 seconds.
        ///
        /// GREEN-BY-DESIGN: structural test documenting the BC-2.11.006 contract
        /// value. Catches accidental constant changes.
        #[test]
        fn test_BC_2_11_006_timeout_constant_is_30s() {
            assert_eq!(
                QUERY_TIMEOUT_SECS, 30,
                "BC-2.11.006: query execution timeout MUST be exactly 30 seconds"
            );
        }

        /// BC-2.11.006 EC-001: Memory pool limit exceeded → E-QUERY-004,
        /// no partial results emitted.
        ///
        /// "DataFusion returns ResourcesExhausted → mapped to
        /// PrismError::QueryMemoryBudgetExceeded." (BC-2.11.006 3-tier fallback)
        ///
        /// RED by design — `map_datafusion_memory_error` is `todo!()`.
        #[tokio::test]
        async fn test_BC_2_11_006_ec001_memory_pool_limit_no_partial_results() {
            todo!("S-3.02 — test_BC_2_11_006_ec001_memory_pool_limit_no_partial_results")
        }

        /// BC-2.11.006 EC-004: Panic inside materialization → SessionScope drops.
        ///
        /// "If execute() panics inside materialization pipeline, SessionScope RAII
        /// drop fires; SessionContext released even on panic." (EC-004)
        ///
        /// RED by design — `SessionScope::context` is `todo!()`.
        #[test]
        fn test_BC_2_11_006_ec004_panic_path_session_scope_drops_context() {
            todo!("S-3.02 — test_BC_2_11_006_ec004_panic_path_session_scope_drops_context")
        }

        /// BC-2.11.006 EC-002: Timeout returns QueryTimeout, NOT QueryExecutionFailed.
        ///
        /// Story §EC-002: "Assert PrismError::QueryTimeout is returned.
        /// NOT E-QUERY-003 (which is for syntactic validation failures)."
        ///
        /// RED by design — timeout enforcement in `execute()` is `todo!()`.
        #[tokio::test]
        async fn test_BC_2_11_006_ec002_timeout_returns_query_timeout_not_execution_error() {
            todo!(
                "S-3.02 — test_BC_2_11_006_ec002_timeout_returns_query_timeout_not_execution_error"
            )
        }

        /// BC-2.11.006 EC-003: Record cap error message includes count and sources.
        ///
        /// "Abort with E-QUERY-005; message includes count and source list."
        ///
        /// RED by design — record cap enforcement in `run_materialization_pipeline`
        /// is `todo!()`.
        #[tokio::test]
        async fn test_BC_2_11_006_ec003_record_cap_message_includes_count_and_sources() {
            todo!("S-3.02 — test_BC_2_11_006_ec003_record_cap_message_includes_count_and_sources")
        }
    }

    // =========================================================================
    // BC-2.11.005 EPHEMERAL MATERIALIZATION — SESSION LIFECYCLE
    // =========================================================================

    mod ephemeral_materialization {

        /// BC-2.11.005 AC-7: SessionScope drops context on normal return.
        ///
        /// "When execute() returns (including on error), SessionContext is dropped."
        ///
        /// RED by design — `SessionScope::context` is `todo!()`.
        #[test]
        fn test_BC_2_11_005_session_scope_drops_context_on_normal_return() {
            todo!("S-3.02 — test_BC_2_11_005_session_scope_drops_context_on_normal_return")
        }

        /// BC-2.11.005: In-query cache avoids redundant API calls for self-joins.
        ///
        /// "Per-query in-memory cache prevents redundant API calls when DataFusion's
        /// plan accesses the same source multiple times." (BC-2.11.005 In-Query Cache)
        ///
        /// The cache is keyed on (client_id, sensor_id, source_id, push_down_params).
        ///
        /// RED by design — `MaterializationContext` and cache logic are `todo!()`.
        #[tokio::test]
        async fn test_BC_2_11_005_in_query_cache_avoids_redundant_api_calls() {
            todo!("S-3.02 — test_BC_2_11_005_in_query_cache_avoids_redundant_api_calls")
        }

        /// BC-2.11.005 / BC-2.11.001: Partial failure returns sensor_errors,
        /// not a full abort.
        ///
        /// "One of 3 sensors returns HTTP 503 → partial results from 2 sensors;
        /// failed sensor in sensor_errors." (BC-2.11.005 Test Vectors)
        ///
        /// RED by design — `run_materialization_pipeline` is `todo!()`.
        #[tokio::test]
        async fn test_BC_2_11_005_partial_failure_returns_sensor_errors_not_full_abort() {
            todo!("S-3.02 — test_BC_2_11_005_partial_failure_returns_sensor_errors_not_full_abort")
        }

        /// BC-2.11.005 DEC-022: All sensors return empty → empty result set, not error.
        ///
        /// "All sensor API calls return empty: Empty result set with total_results: 0,
        /// not an error." (BC-2.11.001 DEC-022)
        ///
        /// RED by design — `run_materialization_pipeline` is `todo!()`.
        #[tokio::test]
        async fn test_BC_2_11_005_empty_all_sensors_returns_empty_result_not_error() {
            todo!("S-3.02 — test_BC_2_11_005_empty_all_sensors_returns_empty_result_not_error")
        }
    }

    // =========================================================================
    // BC-2.11.001 TOOL INTERFACE — RESULT TRUNCATION
    // =========================================================================

    mod tool_interface {

        /// BC-2.11.001: When total_available > limit, is_truncated is true.
        ///
        /// "If more results exist than limit, the response includes is_truncated: true."
        /// (BC-2.11.001 Postconditions §No cross-call pagination)
        ///
        /// RED by design — `QueryEngine::execute` is `todo!()`.
        #[tokio::test]
        async fn test_BC_2_11_001_result_truncation_is_truncated_true_when_over_limit() {
            todo!("S-3.02 — test_BC_2_11_001_result_truncation_is_truncated_true_when_over_limit")
        }

        /// BC-2.11.001: When total_available <= limit, is_truncated is false.
        ///
        /// Results within the limit must not set is_truncated.
        ///
        /// RED by design — `QueryEngine::execute` is `todo!()`.
        #[tokio::test]
        async fn test_BC_2_11_001_result_not_truncated_when_under_limit() {
            todo!("S-3.02 — test_BC_2_11_001_result_not_truncated_when_under_limit")
        }

        /// BC-2.11.001: query_context.original_query carries the input string.
        ///
        /// "Response includes query_context with: original_query."
        /// (BC-2.11.001 Postconditions §query_context)
        ///
        /// RED by design — `QueryEngine::execute` is `todo!()`.
        #[tokio::test]
        async fn test_BC_2_11_001_query_context_contains_original_query_string() {
            todo!("S-3.02 — test_BC_2_11_001_query_context_contains_original_query_string")
        }
    }

    // =========================================================================
    // BC-2.11.007 PUSHDOWN — E-QUERY-009 REQUIRED COLUMN ENFORCEMENT
    // =========================================================================

    mod required_column_enforcement {

        /// BC-2.11.007 VP-031: E-QUERY-009 returned before any API calls.
        ///
        /// "Query rejected with E-QUERY-009 if column is not constrained in
        /// WHERE clause. Rejection occurs before any API calls. Error message
        /// lists the required columns and example usage."
        ///
        /// RED by design — enforcement logic is `todo!()`.
        #[tokio::test]
        async fn test_BC_2_11_007_e_query_009_before_any_api_calls() {
            todo!("S-3.02 — test_BC_2_11_007_e_query_009_before_any_api_calls")
        }

        /// BC-2.11.007: REQUIRED column rejection error includes the sensor name
        /// and example WHERE clause.
        ///
        /// "Structured error includes: the sensor name, the list of REQUIRED columns,
        /// and example WHERE clause syntax." (BC-2.11.007 E-QUERY-009)
        ///
        /// RED by design — enforcement logic is `todo!()`.
        #[tokio::test]
        async fn test_BC_2_11_007_e_query_009_error_includes_sensor_name_and_example() {
            todo!("S-3.02 — test_BC_2_11_007_e_query_009_error_includes_sensor_name_and_example")
        }

        /// BC-2.11.007 / VP-031: Multiple REQUIRED columns all present → query proceeds.
        ///
        /// When all REQUIRED columns are constrained, the query must NOT be rejected.
        /// All REQUIRED predicates end up in push_down.
        ///
        /// RED by design — enforcement logic is `todo!()`.
        #[tokio::test]
        async fn test_BC_2_11_007_all_required_columns_present_query_proceeds() {
            todo!("S-3.02 — test_BC_2_11_007_all_required_columns_present_query_proceeds")
        }
    }

    // =========================================================================
    // BC-2.11.006 MEMORY SUBSYSTEM
    // =========================================================================

    mod memory_subsystem {
        use crate::memory::{build_session_context, map_datafusion_memory_error};

        /// BC-2.11.006: build_session_context creates a per-query SessionContext.
        ///
        /// "Each call produces a fresh, independent SessionContext — the pool is
        /// never shared across queries." (BC-2.11.006 architecture compliance)
        ///
        /// RED by design — `build_session_context` is `todo!()`.
        #[test]
        fn test_BC_2_11_006_build_session_context_creates_fresh_context() {
            todo!("S-3.02 — test_BC_2_11_006_build_session_context_creates_fresh_context")
        }

        /// BC-2.11.006: Two separate build_session_context calls produce
        /// independent pool limits (not shared).
        ///
        /// "Per-query memory pool, not global — do not use a shared global pool."
        ///
        /// RED by design — `build_session_context` is `todo!()`.
        #[test]
        fn test_BC_2_11_006_two_contexts_have_independent_pools() {
            todo!("S-3.02 — test_BC_2_11_006_two_contexts_have_independent_pools")
        }

        /// BC-2.11.006: map_datafusion_memory_error maps ResourcesExhausted
        /// to PrismError::QueryMemoryBudgetExceeded.
        ///
        /// "Pool trips → DataFusion returns ResourcesExhausted → mapped to
        /// PrismError::QueryMemoryBudgetExceeded." (BC-2.11.006 3-tier fallback)
        ///
        /// RED by design — `map_datafusion_memory_error` is `todo!()`.
        #[test]
        fn test_BC_2_11_006_map_datafusion_memory_error_resources_exhausted() {
            todo!("S-3.02 — test_BC_2_11_006_map_datafusion_memory_error_resources_exhausted")
        }

        /// BC-2.11.006: map_datafusion_memory_error wraps non-memory errors in
        /// PrismError::QueryExecutionFailed.
        ///
        /// "If err is not a ResourcesExhausted error, wraps it in
        /// PrismError::QueryExecutionFailed." (memory.rs doc)
        ///
        /// RED by design — `map_datafusion_memory_error` is `todo!()`.
        #[test]
        fn test_BC_2_11_006_map_datafusion_memory_error_other_error_wraps_execution_failed() {
            todo!("S-3.02 — test_BC_2_11_006_map_datafusion_memory_error_other_error_wraps_execution_failed")
        }
    }

    // =========================================================================
    // BC-2.11.005 MATERIALIZATION PIPELINE — SPECIFIC STEPS
    // =========================================================================

    mod pipeline_steps {

        /// BC-2.11.005 Step 2: resolve_source_refs maps query source names to SourceRef.
        ///
        /// "Resolve source refs to (SensorType, client_id, SensorSpec) tuples."
        ///
        /// RED by design — `resolve_source_refs` is `todo!()`.
        #[tokio::test]
        async fn test_BC_2_11_005_resolve_source_refs_maps_names_to_sourceref() {
            todo!("S-3.02 — test_BC_2_11_005_resolve_source_refs_maps_names_to_sourceref")
        }

        /// BC-2.11.005 Step 6: register_mem_table registers batches in DataFusion.
        ///
        /// "Register each source as a DataFusion MemTable named by source ref."
        ///
        /// RED by design — `register_mem_table` is `todo!()`.
        #[test]
        fn test_BC_2_11_005_register_mem_table_creates_accessible_table() {
            todo!("S-3.02 — test_BC_2_11_005_register_mem_table_creates_accessible_table")
        }

        /// BC-2.11.005 Step 8: collect_record_batch_stream drains the stream fully.
        ///
        /// "Collect SendableRecordBatchStream → Vec<RecordBatch>."
        ///
        /// RED by design — `collect_record_batch_stream` is `todo!()`.
        #[tokio::test]
        async fn test_BC_2_11_005_collect_record_batch_stream_drains_fully() {
            todo!("S-3.02 — test_BC_2_11_005_collect_record_batch_stream_drains_fully")
        }
    }

    // =========================================================================
    // SESSION SCOPE (BC-2.11.005 AC-7)
    // =========================================================================

    mod session_scope {

        /// BC-2.11.005: SessionScope::context() returns the inner SessionContext.
        ///
        /// "Obtain a reference to the inner SessionContext for query execution."
        ///
        /// RED by design — `SessionScope::context` is `todo!()`.
        #[test]
        fn test_BC_2_11_005_session_scope_context_returns_inner() {
            todo!("S-3.02 — test_BC_2_11_005_session_scope_context_returns_inner")
        }

        /// BC-2.11.005: SessionScope::into_arc() converts to Arc<SessionContext>.
        ///
        /// "Consume the scope and return the inner SessionContext as an Arc."
        /// Only called from execute_scheduled.
        ///
        /// RED by design — `SessionScope::into_arc` is `todo!()`.
        #[test]
        fn test_BC_2_11_005_session_scope_into_arc_returns_shared_context() {
            todo!("S-3.02 — test_BC_2_11_005_session_scope_into_arc_returns_shared_context")
        }

        /// BC-2.11.005 AC-7: SessionScope Drop releases the inner context.
        ///
        /// The Drop impl calls `drop(self.inner.take())`. The full test requires
        /// `context()` and `into_arc()` to be implemented (both are `todo!()`),
        /// so this test is RED by design even though the Drop skeleton exists.
        ///
        /// RED by design (full coverage) — relies on `context()` being `todo!()`.
        #[test]
        fn test_BC_2_11_005_session_scope_drop_releases_context() {
            todo!("S-3.02 — test_BC_2_11_005_session_scope_drop_releases_context")
        }
    }

    // =========================================================================
    // PERIMETER VERIFICATION (BC-2.11.006 INV-SEC-PERIMETER-001)
    // =========================================================================

    mod perimeter_verification {
        /// S-3.02 perimeter audit: no new restricted symbols added.
        ///
        /// Stub-architect confirmed: S-3.02 adds NO new restricted symbols beyond
        /// those already enforced by BC-2.11.006 v1.10 and covered in
        /// `tests/external/perimeter-violation/src/main.rs`.
        ///
        /// The new S-3.02 modules (engine.rs, materialization.rs, pushdown.rs,
        /// scoping.rs, virtual_fields.rs, memory.rs, session.rs, internal_tables.rs)
        /// use `PrismQlParser::parse` as the sole public parser entry point —
        /// they do NOT re-export any restricted sub-parser symbols.
        ///
        /// GREEN-BY-DESIGN: records the audit finding; always passes.
        /// Actual enforcement is in `tests/external/perimeter-violation/` (CI gate).
        #[test]
        fn test_BC_2_11_006_s302_adds_no_new_restricted_symbols_perimeter_audit_clean() {
            // GREEN-BY-DESIGN: Documents the audit finding that S-3.02's new modules
            // (engine, materialization, pushdown, scoping, virtual_fields, memory,
            // session, internal_tables) add NO new restricted symbols beyond those
            // already enforced in tests/external/perimeter-violation/src/main.rs.
            //
            // Actual enforcement is the CI gate `perimeter-compile-fail`.
            // This test records the audit conclusion for traceability.
        }
    }
}
