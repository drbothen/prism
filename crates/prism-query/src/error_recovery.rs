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
    let offset = err.span().start;
    let message = err.to_string();
    ParseError::new(offset, message)
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

/// Rewrite parse errors that occur after a bare `enrich <ident>` (missing `(<column>)`).
///
/// Chumsky produces a raw "found … expected '('" error when `enrich <ident>` appears
/// without the required `(<column>)` argument.  This is not actionable — analysts
/// need to know what syntax is required.
///
/// This function inspects each error offset against `input` to detect the pattern
/// `enrich <ident> <EOF|space|pipe>` (no `(` following the infusion name) and
/// replaces the raw message with an actionable guidance string.
///
/// The guided message (AC-022 / GRAMMAR-005 verbatim):
/// ```text
/// enrich requires a column argument: | enrich <infusion>(<column>).
/// Example: | enrich threat_score(iocs_value)
/// ```
///
/// This runs in O(N·E) where N = input length and E = error count — both are
/// small (query size limit enforced before parsing).
///
/// Story: S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001 AC-022 (GRAMMAR-005), AC-025 (GRAMMAR-015).
pub fn rewrite_enrich_parse_errors(input: &str, errors: Vec<ParseError>) -> Vec<ParseError> {
    errors
        .into_iter()
        .map(|err| {
            if is_enrich_missing_column_at(input, err.offset) {
                ParseError::new(
                    err.offset,
                    "enrich requires a column argument: \
                     | enrich <infusion>(<column>). \
                     Example: | enrich threat_score(iocs_value)"
                        .to_string(),
                )
            } else {
                err
            }
        })
        .collect()
}

/// Returns `true` if the parse error at `offset` into `input` was caused by a
/// bare `enrich <ident>` without a following `(<column>)` argument.
///
/// Detection heuristic:
/// 1. Scan backwards from `offset` (clamped to input length) through whitespace.
/// 2. Find the previous word boundary — extract the word before the error position.
/// 3. Continue scanning backwards through whitespace to find the word before that.
/// 4. If the word two positions back matches `enrich` (case-insensitive), this is
///    an enrich-missing-column error.
///
/// This does NOT use regex to keep the implementation zero-allocation and compatible
/// with the security perimeter's `no_std`-compatible subset.
fn is_enrich_missing_column_at(input: &str, offset: usize) -> bool {
    // Clamp offset to input byte length (error may point past end for EOF).
    let pos = offset.min(input.len());
    let prefix = &input[..pos];

    // Skip trailing whitespace.
    let prefix_trimmed = prefix.trim_end();
    if prefix_trimmed.is_empty() {
        return false;
    }

    // Extract the last word (infusion name) before the error offset.
    let last_word_end = prefix_trimmed.len();
    let last_word_start = prefix_trimmed
        .rfind(|c: char| !c.is_alphanumeric() && c != '_')
        .map(|i| i + 1)
        .unwrap_or(0);
    if last_word_start >= last_word_end {
        return false;
    }
    let _infusion_name = &prefix_trimmed[last_word_start..last_word_end];

    // Now look at what precedes the infusion name.
    let before_infusion = prefix_trimmed[..last_word_start].trim_end();
    if before_infusion.is_empty() {
        return false;
    }

    // The word before the infusion name must be `enrich` (case-insensitive).
    let kw_end = before_infusion.len();
    let kw_start = before_infusion
        .rfind(|c: char| !c.is_alphanumeric() && c != '_')
        .map(|i| i + 1)
        .unwrap_or(0);
    let keyword = &before_infusion[kw_start..kw_end];
    keyword.eq_ignore_ascii_case("enrich")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// is_enrich_missing_column_at returns true for `FROM t | enrich threat_score` at EOF offset.
    #[test]
    fn test_enrich_missing_column_detection_simple() {
        let input = "FROM t | enrich threat_score";
        // EOF offset is input.len()
        assert!(
            is_enrich_missing_column_at(input, input.len()),
            "should detect enrich missing column at EOF"
        );
    }

    /// is_enrich_missing_column_at returns true in multi-stage pipeline.
    #[test]
    fn test_enrich_missing_column_detection_multi_stage() {
        let input = "FROM t | where severity = 'HIGH' | enrich threat_score";
        assert!(
            is_enrich_missing_column_at(input, input.len()),
            "should detect enrich missing column in multi-stage pipeline at EOF"
        );
    }

    /// is_enrich_missing_column_at returns false when column arg IS present.
    #[test]
    fn test_enrich_missing_column_detection_false_when_arg_present() {
        let input = "FROM t | enrich threat_score(iocs_value)";
        // Error offset would be past the closing paren — not matching enrich pattern.
        assert!(
            !is_enrich_missing_column_at(input, input.len()),
            "should NOT detect enrich missing column when arg is present"
        );
    }

    /// is_enrich_missing_column_at returns false for unrelated errors.
    #[test]
    fn test_enrich_missing_column_detection_false_for_other_keyword() {
        let input = "FROM t | where severity";
        assert!(
            !is_enrich_missing_column_at(input, input.len()),
            "should NOT detect enrich missing column for non-enrich keyword"
        );
    }
}
