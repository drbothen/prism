// Unit test modules for S-1.02 entity types.
pub mod test_alert_severity;
pub mod test_case_status;
pub mod test_credential_name;
pub mod test_cursor_registry;
pub mod test_ids;

// S-1.03: Unit test modules.
pub mod capability_tests;

// S-2.08: TableType canonical enum tests.
pub mod table_type_tests;

// S-DEMO-FIDELITY-REMEDIATION-001: HIGH-002/004 Display regression tests for
// EnrichUdfNotFoundDetails — byte-exact match against PO canonical E-QUERY-039 template.
pub mod test_enrich_udf_not_found_display;

// F-MCPRS-PRL10-OBS-003: Exhaustive compile-time sentinel — every PrismError variant
// must appear here. Because PrismError is #[non_exhaustive], exhaustive matching without
// a wildcard arm only compiles from within the defining crate (this file's location).
// Maintainers: add new variants here AND in prism_error_to_structured_call_result.
pub mod error_category_coverage;
