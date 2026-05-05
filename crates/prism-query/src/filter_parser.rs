//! Filter mode parser: `source | predicate` (BC-2.11.002).
//!
//! Grammar:
//!   filter_query := source_ref '|' predicate
//!   source_ref   := dotted_ident  (e.g. `crowdstrike.detections`)
//!   predicate    := expr
//!   expr         := or_expr
//!   or_expr      := and_expr ('OR' and_expr)*
//!   and_expr     := not_expr ('AND' not_expr)*
//!   not_expr     := 'NOT' not_expr | compare_expr
//!   compare_expr := field_path op literal
//!                 | field_path 'IN' '(' literal_list ')'
//!   op           := '=' | '!=' | '>' | '<' | '>=' | '<=' | 'LIKE'
//!
//! Mode detection: filter mode is detected when a `|` immediately follows
//! the source ref with no `SELECT` or `FROM` keyword.
//!
//! Story: S-3.01 | BC-2.11.002

use crate::ast::{Ast, FilterExpr};
use crate::error::ParseError;
use crate::security;

/// Entry point for the PrismQL parser.
///
/// Detects the query mode (filter / SQL / pipe) and dispatches to the
/// appropriate sub-parser. Security checks (size, nesting depth, stage
/// count) run before any AST is returned.
///
/// Returns `Ok(Ast)` on full parse success, or `Err(Vec<ParseError>)` with
/// all accumulated errors (including partial recovery errors) on failure.
pub struct PrismQlParser;

impl PrismQlParser {
    /// Parse a PrismQL query string and return the AST.
    ///
    /// # Security
    /// - `check_query_size` is called first; oversized inputs return `E-QUERY-003`.
    /// - `check_nesting_depth` is called on the resulting AST before returning.
    ///
    /// # Errors
    /// Returns `Err(Vec<ParseError>)` if the input is syntactically invalid or
    /// exceeds security limits. The vector may contain multiple errors when
    /// Chumsky error recovery is active.
    pub fn parse(input: &str) -> Result<Ast, Vec<ParseError>> {
        todo!(
            "S-3.01: detect query mode (filter/SQL/pipe), run security::check_query_size, \
             dispatch to filter_parser / sql_parser / pipe_parser, run \
             security::check_nesting_depth on resulting AST; input_len={}",
            input.len()
        )
    }
}

/// Parse a filter-mode query: `source | predicate`.
///
/// Called by `PrismQlParser::parse` after mode detection confirms the input
/// is filter mode (a `|` follows the source ref with no SELECT/FROM keyword).
///
/// # Errors
/// Returns accumulated `ParseError`s on failure. On recoverable errors,
/// both a partial AST and errors may be available via the recovery protocol
/// in `error_recovery.rs`.
pub fn parse_filter(input: &str) -> Result<FilterExpr, Vec<ParseError>> {
    todo!(
        "S-3.01: build Chumsky parser for source_ref '|' predicate grammar; \
         attach skip_then_retry_until recovery; call security::check_query_size \
         and security::check_nesting_depth; input_len={}",
        input.len()
    )
}

// ── Security re-export for convenient use in tests ────────────────────────────
pub use security::{PRISM_MAX_NESTING_DEPTH, PRISM_MAX_QUERY_SIZE};
