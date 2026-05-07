//! `prism-query` — Prism query orchestration crate.
//!
//! Created in S-2.08 with the pure-data `_source_type` virtual field injection
//! function. S-3.01 adds the PrismQL parser (filter/SQL/pipe modes via Chumsky 0.12).
//! S-3.02 extends this crate with the DataFusion `TableProvider` integration,
//! `QueryEngine`, and the full ephemeral materialization pipeline.
//! S-3.06 extends the parser with write-mode productions (pipe terminal write stages,
//! SQL DML statements, filter-mode write rejection).
//!
//! # Architecture Compliance (S-3.01)
//! Parser modules MUST NOT import from `prism-sensors`, `prism-mcp`, or any I/O
//! crate. Parsing is a pure function: `&str -> Result<Ast, Vec<ParseError>>`.
//!
//! # Architecture Compliance (S-3.02 / BC-2.11.006 / INV-SEC-PERIMETER-001)
//! Materialization code consumes the parser ONLY via `PrismQlParser::parse`.
//! Restricted sub-parser symbols MUST NOT appear in any S-3.02 module.
//!
//! # Architecture Compliance (S-3.06)
//! Write parser extensions are pure: `WriteVerbRegistry` is initialized once before
//! parse calls and is immutable during parsing — no `WriteEndpointRegistry` I/O
//! during a parse call (BC-2.11.004 purity rule).
//!
//! # Modules
//! - [`types`]                — `SensorQueryDescriptor` struct (table routing context, S-2.08)
//! - [`materialization`]      — ephemeral materialization pipeline + `inject_source_type()` (S-2.08/S-3.02)
//! - [`org_scoped_session_id`] — org-scoped UUID v7 session ID generation for sensor pagination (S-3.2.08 / D-048)
//! - [`ast`]                  — PrismQL AST types: `FilterExpr`, `SqlQuery`, `PipeQuery`, `Expr`, etc. (S-3.01)
//! - [`write_ast`]            — Write mode AST types: `WriteNode`, `DmlNode`, `WriteArg`, `Assignment` (S-3.06)
//! - [`write_verb_registry`]  — `WriteVerbRegistry` wrapping `WriteEndpointRegistry` or test `HashSet` (S-3.06)
//! - [`error`]                — `ParseError` type and ariadne-based error formatting (S-3.01)
//! - [`filter_parser`]        — filter mode parser: `source | predicate` (S-3.01 / BC-2.11.002)
//! - [`sql_parser`]           — SQL mode parser: `SELECT … FROM … JOIN … WHERE …` (S-3.01 / BC-2.11.003)
//! - [`pipe_parser`]          — pipe mode parser: `source | stage | stage` (S-3.01 / BC-2.11.004)
//! - [`security`]             — query size, nesting depth, and stage count guards (S-3.01 / BC-2.11.006)
//! - [`error_recovery`]       — Chumsky recovery strategies shared across parsers (S-3.01)
//! - [`engine`]               — `QueryEngine` struct, `execute()`, `execute_scheduled()` (S-3.02 / BC-2.11.001)
//! - [`pushdown`]             — predicate push-down classification (S-3.02 / BC-2.11.007)
//! - [`scoping`]              — cross-client scope resolution (S-3.02 / BC-2.11.011)
//! - [`virtual_fields`]       — `_sensor`, `_client`, `_source_table` injection (S-3.02 / BC-2.11.012)
//! - [`memory`]               — GreedyMemoryPool + error mapping (S-3.02 / BC-2.11.006)
//! - [`session`]              — `SessionScope` RAII wrapper (S-3.02 / BC-2.11.005)
//! - [`internal_tables`]      — `RocksDbTableProvider` DataFusion integration (S-3.02 / BC-2.15.011)
//! - [`cursor`]               — ephemeral internal pagination cursor for sensor fetch loops (S-3.05 / BC-2.07.001/002)
//! - [`cache_key`]            — SHA-256 cache key derivation, 4-tuple `(client_id, sensor_id, source_id, push_down_hash)` (S-3.05 / BC-2.07.005)
//! - [`cache`]                — sensor-fetch response cache with TTL and LRU eviction (S-3.05 / BC-2.07.003/006)
//! - [`invalidation`]         — synchronous cache invalidation on write operations (S-3.05 / BC-2.07.004)
//! - [`write_pipeline`]       — `WriteExecutor`, `WritePlan`, `WriteOutcome`, `QueryContext` (S-3.07)
//! - [`write_result`]         — `WriteResult`, `WritePreview`, `ConfirmationTokenPreview` (S-3.07)
//! - [`safety_check`]         — Phase 2 pure safety pre-check: feature gates, risk tier (S-3.07)
//! - [`dry_run`]              — Phase 4 dry-run gate, confirmation token gating (S-3.07)
//! - [`write_dispatch`]       — Phase 5 audit intent, semaphore, fan-out, outcome (S-3.07)
//! - [`write_table_registration`] — DataFusion write-capable TableProvider registration (S-3.07)

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

// ── S-3.02 modules ────────────────────────────────────────────────────────────
pub mod engine;
pub mod internal_tables;
pub mod memory;
pub mod pushdown;
pub mod scoping;
pub mod session;
pub mod virtual_fields;

// ── S-3.03 modules ────────────────────────────────────────────────────────────
pub mod explain;

// ── S-3.06 modules ────────────────────────────────────────────────────────────
pub mod write_ast;
pub mod write_verb_registry;

// ── S-3.04 modules — alias system ─────────────────────────────────────────────
pub mod alias_capability;
pub mod alias_resolver;
pub mod alias_store;
pub mod alias_tools;
pub mod alias_types;

// ── S-3.05 modules ────────────────────────────────────────────────────────────
pub mod cache;
pub mod cache_key;
pub mod cursor;
pub mod invalidation;

// ── S-3.07 modules ────────────────────────────────────────────────────────────
pub mod dry_run;
pub mod safety_check;
pub mod write_dispatch;
pub mod write_pipeline;
pub mod write_result;
pub mod write_table_registration;

// ── Kani proofs (cfg-gated; compile everywhere, run only under cargo kani) ────
pub mod proofs;

// ── Unit tests ────────────────────────────────────────────────────────────────
#[cfg(test)]
pub mod tests;

// ── S-3.01 re-exports ─────────────────────────────────────────────────────────
//
// # Security perimeter (B-3, BC-2.11.006, SEC-C-003, F-LOW-002)
//
// `PrismQlParser::parse` and `PrismQlParser::parse_with_registry` are the public
// security entry points. Both apply:
//   1. `check_query_size` — rejects inputs > 64KB before any parsing
//   2. `check_paren_depth` — rejects inputs with > 64 lexical paren depth
//   3. Mode detection — dispatches to `parse_sql`, `parse_pipe`, or `parse_filter`
// `parse_with_registry` additionally routes pipe mode through `parse_pipe_with_write`
// and filter mode through `reject_write_verbs_in_filter` (BC-2.11.004, F-PR130-CR-001).
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
// Write-parser internals (S-3.06, BC-2.11.004 + BC-2.11.006 v1.16 DI-034 layer 4):
//   `parse_pipe_with_write`, `build_write_stage_parser`,
//   `build_write_arg_parser`, `extract_sensor_prefix` (pipe_parser)
//   `parse_sql_dml`, `parse_sql_dml_with_limits`,
//   `is_internal_prism_table`, `check_unbounded_write` (sql_parser)
//   `reject_write_verbs_in_filter` (filter_parser)
//
// Alias-system internals (S-3.04, BC-2.11.008 + BC-2.11.006 v1.17 DI-034 layer 5):
//   `create_alias` (alias_tools)                          — ungated create; MCP MUST use *_gated (SEC-011)
//   `create_alias_with_clients` (alias_tools)             — ungated create+clients; MCP MUST use *_gated
//   `create_alias_with_clients_gated_inner` (alias_tools) — internal token-store split (F-LOCAL-P2-HIGH-005)
//   `delete_alias` (alias_tools)                          — ungated delete; MCP MUST use *_gated (SEC-011)
//   `AliasStore` (alias_store) — `create_or_update` method pub(crate): direct store mutation; bypasses guards (CR-018)
//
// Tests that need direct sub-parser access (e.g., to obtain
// FilterExpr/PipeQuery/SqlQuery directly, or to bypass pre-parse guards to
// test post-parse depth checks in isolation) must live in src/tests/ (unit
// tests) where pub(crate) items are visible.
//
// External consumers MUST use `PrismQlParser::parse` or `PrismQlParser::parse_with_registry`.
pub use ast::Ast;
pub use error::ParseError;
pub use filter_parser::PrismQlParser;
pub use write_verb_registry::WriteVerbRegistry;

// ── S-3.07 re-exports ─────────────────────────────────────────────────────────
pub use write_pipeline::{QueryContext, WriteExecutor, WriteOutcome, WritePlan};
pub use write_result::{ConfirmationTokenPreview, SensorWriteError, WritePreview, WriteResult};
