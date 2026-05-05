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

// ── Kani proofs (cfg-gated; compile everywhere, run only under cargo kani) ────
pub mod proofs;

// ── Unit tests ────────────────────────────────────────────────────────────────
#[cfg(test)]
pub mod tests;

// ── S-3.01 re-exports ─────────────────────────────────────────────────────────
pub use ast::Ast;
pub use error::ParseError;
pub use filter_parser::PrismQlParser;
