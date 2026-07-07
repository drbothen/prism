//! Unit test modules for `prism-query`.
//!
//! Story: S-2.08 | AC-9, AC-10 | S-3.01 (parser tests moved here for pub(crate) access)
//! Story: S-3.02 (integration_tests — query materialization pipeline, Red Gate)
//! Story: S-DEMO-002 (aql_pushdown_tests — AC-014 AQL seeding Red Gate)
//!
//! # Test migration (F-LOW-002)
//! `parser_tests` and `regression_tests` were moved from `tests/` (integration tests)
//! to `src/tests/` (unit tests) so that they can access `pub(crate)` functions
//! (`parse_filter`, `parse_pipe`, `parse_sql`) directly. Integration tests in
//! `tests/` compile against the public API only, which no longer includes the
//! mode-specific sub-parsers.

pub mod alias_tests;
// S-DEMO-FIDELITY-REMEDIATION-001: Red Gate tests for AC-N1B — BC-2.11.019 E-QUERY-039
// plan-time enrichment gate (net-new: EnrichUdfNotFound variant + engine.rs AST visitor).
pub mod bc_2_11_019_n1b_test;
// S-PRISMQL-NATIVE-TEMPORAL-TYPING-001: Red Gate tests RG-004, RG-005, RG-007 —
// E-QUERY-041 temporal literal pre-validator (check_temporal_literals defined in
// materialization.rs; invoked as an early gate in engine.rs before check_table_availability).
// Tests verify that date-only string literals trigger E-QUERY-041, and valid RFC-3339
// strings pass the gate (BC-2.11.021 v1.2, BC-2.11.003 v1.6, BC-2.11.004 v1.7; ADR-052 D4).
pub mod temporal_typing_tests;
// S-DEMO-FIDELITY-REMEDIATION-001: Red Gate tests for AC-N2 — BC-2.11.001 v1.15 EC-11-067
// dot-notation FROM target must return E-QUERY-037 (TableNotAvailable) with did_you_mean,
// not route to fan-out silently. Includes BC-2.11.023/ADR-046 filter-mode regression guard.
pub mod bc_2_11_001_n2_test;
// S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001: HIGH-002 plan-time pinning unit tests.
// Drives inject_now + PqlNormalizer::normalize to assert plan-pinned form (no NOW(), no INTERVAL).
pub mod high002_plan_pinning_tests;
// S-DEMO-002: Red Gate unit tests for AC-014 AQL push-down seeding.
// Drives production code path (pushdown::predicate_tree_to_filter_map) without external DTU.
pub mod aql_pushdown_tests;
// S-3.13: Red Gate tests for dynamic table availability + E-QUERY-037 plan-time gate.
// Tests covering BC-2.11.001 / BC-2.16.001 / BC-2.16.007 (table availability registry + gate).
pub mod bc_gap_fill_tests;
pub mod cache_tests;
pub mod explain_tests;
pub mod integration_tests;
pub mod materialization_tests;
pub mod pagination_tests;
pub mod parser_tests;
pub mod regression_tests;
pub mod table_registry_tests;
pub(crate) mod util;
pub mod write_parser_unit_tests;
// S-PRISMQL-CASE-INSENSITIVE-001: Red Gate tests RG-001 through RG-018, RG-022, RG-023, RG-024 —
// IEQ/IIN/INE case-insensitive operator parsing, normalization, and DataFusion SQL lowering.
pub mod test_case_insensitive_operators;
