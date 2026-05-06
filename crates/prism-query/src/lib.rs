//! `prism-query` — Prism query orchestration crate.
//!
//! Created in S-2.08 with the pure-data `_source_type` virtual field injection
//! function. S-3.01 adds the PrismQL parser (filter/SQL/pipe modes via Chumsky 0.12).
//! S-3.02 extends this crate with the DataFusion `TableProvider` integration.
//!
//! # Architecture Compliance (S-2.08)
//! This crate MUST NOT depend on DataFusion, Arrow, `arrow-schema`, or `arrow2`.
//! Those dependencies are added by S-3.02. See `Cargo.toml` for the enforcement
//! comment.
//!
//! # Architecture Compliance (S-3.01)
//! Parser modules MUST NOT import from `prism-sensors`, `prism-mcp`, or any I/O
//! crate. Parsing is a pure function: `&str -> Result<Ast, Vec<ParseError>>`.
//!
//! # Modules
//! - [`types`]                — `SensorQueryDescriptor` struct (table routing context, S-2.08)
//! - [`materialization`]      — `inject_source_type()` pure-data virtual field injection (S-2.08)
//! - [`org_scoped_session_id`] — org-scoped UUID v7 session ID generation for sensor pagination (S-3.2.08 / D-048)
//! - [`ast`]                  — PrismQL AST types: `FilterExpr`, `SqlQuery`, `PipeQuery`, `Expr`, etc. (S-3.01)
//! - [`error`]                — `ParseError` type and ariadne-based error formatting (S-3.01)
//! - [`filter_parser`]        — filter mode parser: `source | predicate` (S-3.01 / BC-2.11.002)
//! - [`sql_parser`]           — SQL mode parser: `SELECT … FROM … JOIN … WHERE …` (S-3.01 / BC-2.11.003)
//! - [`pipe_parser`]          — pipe mode parser: `source | stage | stage` (S-3.01 / BC-2.11.004)
//! - [`security`]             — query size, nesting depth, and stage count guards (S-3.01 / BC-2.11.006)
//! - [`error_recovery`]       — Chumsky recovery strategies shared across parsers (S-3.01)

// ── S-2.08 modules ────────────────────────────────────────────────────────────
pub mod materialization;
pub mod org_scoped_session_id;
pub mod types;

// ── S-3.01 modules ────────────────────────────────────────────────────────────
pub mod ast;
pub mod error;
pub mod error_recovery;
pub mod filter_parser;
pub mod pipe_parser;
pub mod security;
pub mod sql_parser;
pub mod visit;

// ── Kani proofs (cfg-gated; compile everywhere, run only under cargo kani) ────
pub mod proofs;

// ── Unit tests ────────────────────────────────────────────────────────────────
#[cfg(test)]
pub mod tests;

// ── S-3.01 re-exports ─────────────────────────────────────────────────────────
//
// # Security perimeter (B-3, BC-2.11.006, SEC-C-003, F-LOW-002)
//
// `PrismQlParser::parse` is the SOLE public security entry point. It applies:
//   1. `check_query_size` — rejects inputs > 64KB before any parsing
//   2. `check_paren_depth` — rejects inputs with > 64 lexical paren depth
//   3. Mode detection — dispatches to `parse_sql`, `parse_pipe`, or `parse_filter`
//
// The following symbols are `pub(crate)` and MUST NOT be exposed externally.
// Authoritative source: BC-2.11.006 frontmatter `restricted_symbols`.
//
// Sub-parsers:
//   `parse_filter`, `parse_filter_with_limits`
//   `parse_sql`, `parse_sql_with_limits`
//   `parse_pipe`, `parse_pipe_with_limits`
//
// Parser-builder factories:
//   `build_predicate_parser`, `build_source_ref_parser`,
//   `build_string_parser`, `build_literal_parser`,
//   `build_expr_parser`, `build_pipe_mode_parser`,
//   `build_pipe_parser`
//
// ParseLimits API:
//   `ParseLimits::install_thread_local`, `ParseLimits::clear_thread_local`,
//   `ParseLimits::current_regex_limit`, `ParseLimits::snapshot`,
//   `ParseLimits` struct fields
//
// Drop guard:
//   `ThreadLocalGuard` (filter_parser) — `pub(crate)` for unit-test
//   verification of Drop semantics; not part of the stable API.
//
// Tests that need direct sub-parser access (e.g., to obtain
// FilterExpr/PipeQuery/SqlQuery directly, or to bypass pre-parse guards to
// test post-parse depth checks in isolation) must live in src/tests/ (unit
// tests) where pub(crate) items are visible.
//
// External consumers MUST use `PrismQlParser::parse` exclusively.
pub use ast::Ast;
pub use error::ParseError;
pub use filter_parser::PrismQlParser;
