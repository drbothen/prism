//! SQL mode parser: `SELECT … FROM … JOIN … WHERE …` (BC-2.11.003).
//!
//! Grammar (abbreviated):
//!   sql_query   := 'SELECT' select_list 'FROM' source_ref [alias]
//!                  join_clause*
//!                  ['WHERE' expr]
//!                  ['GROUP BY' expr_list]
//!                  ['HAVING' expr]
//!                  ['ORDER BY' order_expr_list]
//!                  ['LIMIT' integer]
//!   select_list := '*' | 'DISTINCT' select_item (',' select_item)*
//!   select_item := '*' | 'table.*' | expr ['AS' ident]
//!   join_clause := join_kind 'JOIN' source_ref [alias] 'ON' expr
//!   join_kind   := 'INNER' | 'LEFT' | 'RIGHT' | 'FULL OUTER'
//!
//! Mode detection: SQL mode is detected when the input starts with the
//! keyword `SELECT` (case-insensitive).
//!
//! Subqueries: `WHERE field IN (SELECT …)` is supported via recursive
//! parser construction. Chumsky `recursive()` is used to handle this.
//!
//! Story: S-3.01 | BC-2.11.003

use crate::ast::SqlQuery;
use crate::error::ParseError;

/// Parse a SQL-mode query.
///
/// Called by `PrismQlParser::parse` after mode detection confirms the input
/// starts with `SELECT`.
///
/// # Errors
/// Returns accumulated `ParseError`s on failure. `nested_delimiters`
/// recovery is used inside parenthesized subexpressions.
pub fn parse_sql(input: &str) -> Result<SqlQuery, Vec<ParseError>> {
    todo!(
        "S-3.01: build Chumsky parser for full SQL grammar including JOIN, \
         subqueries, GROUP BY, HAVING, ORDER BY, LIMIT; attach nested_delimiters \
         recovery; input_len={}",
        input.len()
    )
}
