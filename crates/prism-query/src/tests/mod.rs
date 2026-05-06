//! Unit test modules for `prism-query`.
//!
//! Story: S-2.08 | AC-9, AC-10 | S-3.01 (parser tests moved here for pub(crate) access)
//! Story: S-3.02 (integration_tests — query materialization pipeline, Red Gate)
//!
//! # Test migration (F-LOW-002)
//! `parser_tests` and `regression_tests` were moved from `tests/` (integration tests)
//! to `src/tests/` (unit tests) so that they can access `pub(crate)` functions
//! (`parse_filter`, `parse_pipe`, `parse_sql`) directly. Integration tests in
//! `tests/` compile against the public API only, which no longer includes the
//! mode-specific sub-parsers.

pub mod bc_gap_fill_tests;
pub mod integration_tests;
pub mod materialization_tests;
pub mod parser_tests;
pub mod regression_tests;
pub(crate) mod util;
pub mod write_parser_unit_tests;
