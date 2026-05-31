//! Red Gate tests for S-SPEC-ENV-VAR-001 — `${env.VAR}` token resolution in sensor spec fields.
//!
//! BC-2.16.009 §Validation Rules 6 (AC-6); S-SPEC-ENV-VAR-001.
//!
//! All 8 Red Gate tests are defined here. At stub time, the test bodies are `todo!()` stubs
//! so every test in this file FAILS (Red Gate discipline, BC-5.38.001).
//!
//! Test names map 1:1 to ACs in the story spec:
//!
//! | Test name | AC |
//! |-----------|-----|
//! | test_env_var_full_token_resolves_to_value | AC-001 |
//! | test_env_var_partial_token_resolves_preserving_surrounding_literals | AC-002 |
//! | test_env_var_multi_token_single_field_both_resolve | AC-003 |
//! | test_env_var_missing_var_produces_e_spec_024 | AC-004 |
//! | test_env_var_empty_var_produces_e_spec_024 | AC-005 |
//! | test_env_var_multi_missing_tokens_collect_multiple_errors | AC-006 |
//! | test_env_var_resolution_runs_before_url_format_validation | AC-007 |
//! | test_env_var_error_contains_name_not_value | AC-008 |
//!
//! Written by: test-writer (stub file authored by stub-architect; test bodies authored by test-writer).
