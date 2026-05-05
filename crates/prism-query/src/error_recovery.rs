//! Chumsky 0.12 error recovery strategies shared across all parser modules.
//!
//! Recovery strategies:
//! - `skip_then_retry_until` — used in filter/pipe parsers to skip past
//!   unknown tokens and retry at the next recognizable token
//! - `nested_delimiters` — used in SQL parser to recover inside
//!   parenthesized subexpressions
//!
//! Errors are accumulated; the parser returns both a partial AST (for
//! valid prefixes) and all accumulated `ParseError`s.
//!
//! Story: S-3.01

use chumsky::error::Rich;

use crate::error::ParseError;

/// Convert a Chumsky `Rich<char>` error into a `ParseError`.
///
/// This mapping is used by all three parsers to normalise Chumsky's internal
/// error representation into the public `ParseError` API.
pub fn rich_to_parse_error(err: &Rich<'_, char>) -> ParseError {
    todo!(
        "S-3.01: extract offset and reason from Rich<char>; construct ParseError; \
         err_span={:?}",
        err.span()
    )
}

/// Return the set of characters that signal a pipe-stage boundary.
///
/// The filter and pipe parsers use these as the `retry_until` token set
/// when constructing `skip_then_retry_until` recovery combinators.
///
/// Implementer: pass these characters to `skip_then_retry_until` inside
/// `filter_parser::build_filter_parser` and `pipe_parser::build_pipe_parser`.
pub fn pipe_boundary_chars() -> &'static [char] {
    &['|']
}

/// Return the delimiter pair used by the SQL parser's `nested_delimiters`
/// recovery combinator: `('(', ')')`.
///
/// Implementer: pass this pair to `nested_delimiters` inside
/// `sql_parser::build_sql_parser` when constructing subquery recovery.
pub fn sql_paren_delimiters() -> (char, char) {
    ('(', ')')
}
