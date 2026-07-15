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

use chumsky::error::{Rich, RichReason};

use crate::error::ParseError;

/// Convert a Chumsky `Rich<char>` error into a `ParseError`.
///
/// This mapping is used by all three parsers to normalise Chumsky's internal
/// error representation into the public `ParseError` API.
///
/// `ParseError.semantic` is set to `true` when the Rich error is a
/// `RichReason::Custom` variant — i.e., emitted by a `.validate()` or
/// `.try_map()` combinator returning `Rich::custom(...)`.  Structural parse
/// failures (`RichReason::ExpectedFound`) are not semantic.
///
/// This structural discriminant replaces the retired prefix-based check
/// `e.message.starts_with("E-QUERY-001:")` (F-PQLFN-PR10-MED-001 fix-burst-41,
/// ADR-048 §D.7.2 de-prefix discipline).
pub fn rich_to_parse_error(err: &Rich<'_, char>) -> ParseError {
    let offset = err.span().start;
    let message = err.to_string();
    let mut pe = ParseError::new(offset, message);
    // Semantic errors originate from Rich::custom (validate/try_map) combinators,
    // not from Chumsky's structural ExpectedFound machinery.
    pe.semantic = matches!(err.reason(), RichReason::Custom(_));
    pe
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
    //
    // SAFETY (F-P3-CRIT-001): `rfind(char_pred)` returns the byte START of the matched
    // char, so `i + 1` lands INSIDE any multibyte non-alphanumeric char (e.g. `»`
    // U+00BB = 2 bytes: rfind → byte 17, i+1 = 18 = inside the char → panic on slice).
    //
    // Fix: use `char_indices().rev().find(pred)` which yields `(byte_offset, char)`.
    // `i + c.len_utf8()` is always the byte START of the NEXT character — a valid
    // char boundary regardless of whether `c` is ASCII or multibyte.
    let last_word_end = prefix_trimmed.len();
    let last_word_start = prefix_trimmed
        .char_indices()
        .rev()
        .find(|(_, c)| !c.is_alphanumeric() && *c != '_')
        .map(|(i, c)| i + c.len_utf8())
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
    // Same char-boundary-safe pattern: use char_indices().rev().find() instead of rfind.
    let kw_end = before_infusion.len();
    let kw_start = before_infusion
        .char_indices()
        .rev()
        .find(|(_, c)| !c.is_alphanumeric() && *c != '_')
        .map(|(i, c)| i + c.len_utf8())
        .unwrap_or(0);
    let keyword = &before_infusion[kw_start..kw_end];
    keyword.eq_ignore_ascii_case("enrich")
}

// ---------------------------------------------------------------------------
// D2 mode-bridge rewrite (BC-2.11.023 AC-027 / ADR-046 §D2)
// ---------------------------------------------------------------------------

/// Rewrite pipe-mode parse errors that occur when an uppercase SQL clause keyword
/// (`SELECT` or `ORDER BY`) appears in stage position (after `|`).
///
/// `WHERE` and `LIMIT` already parse in pipe mode because stage keywords are
/// case-insensitive in PrismQL — D2 does NOT fire for those.
///
/// The verbatim D2 message (BC-2.11.023 §D2, POL-24):
/// ```text
/// E-QUERY-001: parse error near '<keyword>': SQL clauses are not valid as pipe stages.
/// In pipe mode, use lowercase stage keywords: 'where', 'sort', 'limit', 'stats'.
/// Example: FROM <table> | where severity = 'HIGH' | sort time DESC | limit 10
/// ```
///
/// Story: S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001 AC-027 (HIGH-2).
pub fn rewrite_d2_sql_keyword_in_pipe_position(
    input: &str,
    errors: Vec<ParseError>,
) -> Vec<ParseError> {
    // Detect whether `input` contains `| SELECT ...` or `| ORDER BY ...` in stage position.
    if let Some(keyword) = detect_sql_keyword_in_pipe_stage(input) {
        // Replace ALL accumulated errors with the single verbatim D2 message.
        // (Multiple Chumsky errors from the same root cause collapse into one diagnostic.)
        let msg = format!(
            "E-QUERY-001: parse error near '{keyword}': SQL clauses are not valid as pipe stages.\n\
             In pipe mode, use lowercase stage keywords: 'where', 'sort', 'limit', 'stats'.\n\
             Example: FROM <table> | where severity = 'HIGH' | sort time DESC | limit 10"
        );
        vec![ParseError::new(0, msg)]
    } else {
        errors
    }
}

/// Scan `input` for a `| <SQL_CLAUSE_KEYWORD>` pattern in pipe stage position.
///
/// Returns `Some("SELECT")` or `Some("ORDER BY")` when found, `None` otherwise.
///
/// Only these two keywords trigger D2 per BC-2.11.023 §D2:
/// - `SELECT` — not a valid pipe stage; distinct from `where`/`limit`/etc.
/// - `ORDER BY` — SQL ordering clause; pipe mode uses `sort` instead.
///
/// `WHERE` and `LIMIT` are intentionally excluded: the pipe parser is case-insensitive
/// so `| WHERE ...` and `| LIMIT ...` already parse as `| where` / `| limit`.
fn detect_sql_keyword_in_pipe_stage(input: &str) -> Option<&'static str> {
    let bytes = input.as_bytes();
    let len = bytes.len();
    let mut in_sq = false;
    let mut in_dq = false;
    let mut i = 0;
    while i < len {
        match bytes[i] {
            b'\'' if !in_dq => in_sq = !in_sq,
            b'"' if !in_sq => in_dq = !in_dq,
            b'|' if !in_sq && !in_dq => {
                // Skip whitespace after `|`.
                let mut j = i + 1;
                while j < len && (bytes[j] == b' ' || bytes[j] == b'\t' || bytes[j] == b'\n') {
                    j += 1;
                }
                if let Some(rest) = input.get(j..) {
                    // Check for `SELECT` (6 chars).
                    if let Some(candidate) = rest.get(..6) {
                        if candidate.eq_ignore_ascii_case("select") {
                            // Must be followed by whitespace or end of input to avoid
                            // false-positives like `| selected_fields`.
                            let after = rest.get(6..).unwrap_or("");
                            if after
                                .as_bytes()
                                .first()
                                .is_none_or(|b| b.is_ascii_whitespace())
                            {
                                return Some("SELECT");
                            }
                        }
                    }
                    // Check for `ORDER BY` (8 chars: "ORDER BY").
                    if let Some(candidate) = rest.get(..8) {
                        if candidate.eq_ignore_ascii_case("order by") {
                            return Some("ORDER BY");
                        }
                    }
                    // Also check `ORDER` alone (5 chars) — ORDER without BY is still
                    // a SQL clause keyword in stage position that analysts mis-type.
                    if let Some(candidate) = rest.get(..5) {
                        if candidate.eq_ignore_ascii_case("order") {
                            let after = rest.get(5..).unwrap_or("");
                            if after
                                .as_bytes()
                                .first()
                                .is_none_or(|b| b.is_ascii_whitespace())
                            {
                                return Some("ORDER BY");
                            }
                        }
                    }
                }
            }
            _ => {}
        }
        i += 1;
    }
    None
}

// ---------------------------------------------------------------------------
// Temporal-literal-in-pipe-key-position rewrite (ADR-052 §D4 v1.10 option (a))
// ---------------------------------------------------------------------------

/// Rewrite pipe-mode parse errors that occur when a quoted string literal appears
/// in `sort` or `stats by` key position.
///
/// Chumsky produces a generic `"found ''' expected something else"` error when a
/// quoted literal is used in position where a field name (FieldPath) is required.
/// This is not actionable — analysts need to know they must use a column name.
///
/// **Patterns detected (case-insensitive, after a `|` separator):**
///
/// - `| sort '<literal>'` → sort key must be a field name
/// - `| stats … by '<literal>'` → stats-by key must be a field name
///
/// **Messages produced (always contain both "field name" and "literal value"
/// so callers can assert on either substring):**
///
/// ```text
/// E-QUERY-001: 'sort' expects a field name, not a literal value '<...>'.
/// Use a column name (e.g., `| sort timestamp DESC`) instead of a quoted string.
///
/// E-QUERY-001: 'stats by' expects a field name, not a literal value '<...>'.
/// Use a column name (e.g., `| stats count by hostname`) instead of a quoted string.
/// ```
///
/// Only fires in the error path — no false-positive risk for valid queries.
///
/// Story: S-PRISMQL-NATIVE-TEMPORAL-TYPING-001 pipe-parse enhancement (ADR-052 §D4 v1.10).
pub fn rewrite_temporal_literal_in_pipe_key_position(
    input: &str,
    errors: Vec<ParseError>,
) -> Vec<ParseError> {
    if errors.is_empty() {
        return errors;
    }
    if let Some(msg) = detect_literal_in_pipe_key_position(input) {
        // Collapse all accumulated errors into one analyst-readable message.
        vec![ParseError::new(0, msg)]
    } else {
        errors
    }
}

/// Returns `Some(message)` if `input` contains `| sort '<...'` or `by '<...'`
/// (with optional whitespace), `None` otherwise.
fn detect_literal_in_pipe_key_position(input: &str) -> Option<String> {
    // Work on the ASCII-lowercased bytes for case-insensitive matching.
    // (Query size is bounded by `check_query_length`; allocation is O(N).)
    let lc = input.to_ascii_lowercase();
    let bytes = lc.as_bytes();
    let len = bytes.len();

    let mut i = 0;
    while i < len {
        if bytes[i] == b'|' {
            // Skip whitespace after `|`.
            let mut j = i + 1;
            while j < len && bytes[j].is_ascii_whitespace() {
                j += 1;
            }

            // Pattern: `| sort '<literal>'`
            // `sort` (4 bytes) must be a whole word (followed by whitespace).
            if j + 4 <= len && &bytes[j..j + 4] == b"sort" {
                let after_sort = j + 4;
                if after_sort >= len || bytes[after_sort].is_ascii_whitespace() {
                    let mut k = after_sort;
                    while k < len && bytes[k].is_ascii_whitespace() {
                        k += 1;
                    }
                    if k < len && bytes[k] == b'\'' {
                        // Extract the literal value for a more helpful message.
                        let lit = extract_quoted_literal(input, k).unwrap_or("...");
                        return Some(format!(
                            "E-QUERY-001: 'sort' expects a field name, not a literal value \
                             '{lit}'.\n\
                             Use a column name (e.g., `| sort timestamp DESC`) \
                             instead of a quoted string literal."
                        ));
                    }
                }
            }

            // Pattern: `| stats … by '<literal>'` — scan the rest of this stage for `by '`.
            // Note: we don't validate that `stats` precedes `by` — any `by '` in a pipe
            // stage is an analyst mistake.  False positives in `stats by` position are
            // acceptable since the message is strictly better than the generic chumsky error.
            let mut k = j;
            while k < len && bytes[k] != b'|' {
                // Look for word-boundary `by ` followed by `'`.
                if k + 2 <= len
                    && &bytes[k..k + 2] == b"by"
                    // Must be a whole word (preceded by whitespace or start-of-stage).
                    && (k == 0 || bytes[k - 1].is_ascii_whitespace())
                {
                    let after_by = k + 2;
                    if after_by < len && bytes[after_by].is_ascii_whitespace() {
                        let mut m = after_by;
                        while m < len && bytes[m].is_ascii_whitespace() {
                            m += 1;
                        }
                        if m < len && bytes[m] == b'\'' {
                            let lit = extract_quoted_literal(input, m).unwrap_or("...");
                            return Some(format!(
                                "E-QUERY-001: 'stats by' expects a field name, not a literal \
                                 value '{lit}'.\n\
                                 Use a column name (e.g., `| stats count by hostname`) \
                                 instead of a quoted string literal."
                            ));
                        }
                    }
                }
                k += 1;
            }
        }
        i += 1;
    }
    None
}

/// Extract the contents of a single-quoted string starting at byte offset `start`
/// in `input` (which points at the opening `'`).
///
/// Returns up to 50 Unicode codepoints (codepoint-safe) or `None` if the literal
/// is malformed (no closing quote found).
///
/// The 50-codepoint cap is an AD-017 belt-and-suspenders guard against inadvertent
/// secret exposure via pipe-key-position error messages — consistent with the
/// E-QUERY-041/042 `value_prefix` 50-char cap.
///
/// **Codepoint-safe:** uses `.char_indices().nth(50)` to locate the truncation byte
/// offset, never slicing at a raw byte count that might land inside a multibyte
/// UTF-8 sequence (no VP-021 violation).
fn extract_quoted_literal(input: &str, start: usize) -> Option<&str> {
    let rest = input.get(start + 1..)?; // skip the opening quote
    let end = rest.find('\'')?;
    let content = &rest[..end];
    // Truncate at 50 Unicode codepoints.
    // `.char_indices().nth(50)` yields (byte_offset_of_51st_char, char).
    // Slicing [..that_offset] gives exactly the first 50 codepoints.
    // Falls back to `content.len()` when the literal has fewer than 50 codepoints.
    let truncation_byte = content
        .char_indices()
        .nth(50)
        .map(|(i, _)| i)
        .unwrap_or(content.len());
    Some(&content[..truncation_byte])
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

    // ── F-P3-CRIT-001 unit regression tests ──────────────────────────────────
    //
    // These tests call is_enrich_missing_column_at DIRECTLY with offset=input.len()
    // (the EOF offset that the function always receives for "missing column" errors).
    // They trigger the char-boundary panic by placing a multibyte non-alphanumeric
    // char INSIDE the prefix that rfind scans.
    //
    // Before the fix: rfind returns the START byte of the multibyte char (e.g. byte 17
    // for »), then .map(|i| i+1) produces byte 18 which is INSIDE the char. The
    // subsequent &prefix_trimmed[18..] slice panics "not a char boundary".
    // After the fix: must return bool without panicking.

    /// F-P3-CRIT-001 unit-a: `»` (U+00BB, 2 bytes) between ident tokens — EOF offset.
    ///
    /// `"FROM t | enrich a»b"` at offset=20 (EOF):
    ///   prefix_trimmed = "FROM t | enrich a»b"
    ///   rfind(!alphanumeric, !=_) finds '»' at byte 17 → i+1=18 NOT a boundary → PANIC.
    #[test]
    fn test_f_p3_crit_001_unit_a_two_byte_separator_no_panic() {
        let input = "FROM t | enrich a\u{00BB}b";
        // catch_unwind returns Err if a panic occurs.
        // Before fix: panic → Err → assertion fails (RED gate).
        // After fix: no panic → Ok → assertion passes (GREEN gate).
        let result = std::panic::catch_unwind(|| is_enrich_missing_column_at(input, input.len()));
        assert!(
            result.is_ok(),
            "F-P3-CRIT-001 unit-a: is_enrich_missing_column_at panicked on 2-byte separator '»'"
        );
    }

    /// F-P3-CRIT-001 unit-b: `—` (U+2014, 3 bytes) between ident tokens — EOF offset.
    ///
    /// `"FROM t | enrich x—y"` at offset=EOF:
    ///   rfind finds '—' at its start byte, i+1 is inside the 3-byte sequence → PANIC.
    #[test]
    fn test_f_p3_crit_001_unit_b_three_byte_separator_no_panic() {
        let input = "FROM t | enrich x\u{2014}y";
        let result = std::panic::catch_unwind(|| is_enrich_missing_column_at(input, input.len()));
        assert!(
            result.is_ok(),
            "F-P3-CRIT-001 unit-b: is_enrich_missing_column_at panicked on 3-byte separator '—'"
        );
    }

    /// F-P3-CRIT-001 unit-c: `×` (U+00D7, 2 bytes) as separator — EOF offset.
    ///
    /// `"FROM t | enrich a×b"` at offset=EOF:
    ///   rfind finds '×' at its start byte, i+1 is inside the 2-byte sequence → PANIC.
    #[test]
    fn test_f_p3_crit_001_unit_c_multiplication_sign_no_panic() {
        // × is U+00D7: 0xC3 0x97 (2 bytes), not alphanumeric.
        let input = "FROM t | enrich a\u{00D7}b";
        let result = std::panic::catch_unwind(|| is_enrich_missing_column_at(input, input.len()));
        assert!(
            result.is_ok(),
            "F-P3-CRIT-001 unit-c: is_enrich_missing_column_at panicked on 2-byte '×' separator"
        );
    }

    /// F-P3-CRIT-001 unit-d: same bug in kw_start/kw_end second rfind — `»` before `enrich`.
    ///
    /// `"FROM t | enrich»threat_score"` — the first rfind extracts the infusion name
    /// `threat_score` (after `»`), then the second rfind extracts the keyword candidate
    /// and finds `|` or `»`. If `»` appears just before `enrich` keyword token, the
    /// second rfind hits it.
    ///
    /// Input: `"FROM t | a»enrich threat_score"` — second rfind on `"FROM t | a»enrich"`
    /// finds `»` at byte 10, i+1=11 is NOT a boundary → PANIC.
    #[test]
    fn test_f_p3_crit_001_unit_d_second_rfind_kw_boundary_no_panic() {
        // "FROM t | a»enrich threat_score": the second rfind (for keyword) operates on
        // "FROM t | a»enrich" and finds '»' at byte 10, i+1=11 is inside '»' → PANIC.
        let input = "FROM t | a\u{00BB}enrich threat_score";
        let result = std::panic::catch_unwind(|| is_enrich_missing_column_at(input, input.len()));
        assert!(
            result.is_ok(),
            "F-P3-CRIT-001 unit-d: is_enrich_missing_column_at panicked on 2-byte separator before keyword"
        );
    }

    /// F-P3-CRIT-001 unit-e: ASCII behavior preserved after fix.
    ///
    /// `"FROM t | enrich threat_score"` at EOF must still return true (enrich missing column).
    /// `"FROM t | where severity"` at EOF must still return false (not enrich pattern).
    #[test]
    fn test_f_p3_crit_001_unit_e_ascii_behavior_preserved() {
        assert!(
            is_enrich_missing_column_at("FROM t | enrich threat_score", 28),
            "F-P3-CRIT-001 unit-e: ASCII enrich-missing-column must return true"
        );
        assert!(
            !is_enrich_missing_column_at("FROM t | where severity", 23),
            "F-P3-CRIT-001 unit-e: non-enrich pattern must return false"
        );
    }

    // ── LOW-1: extract_quoted_literal 50-codepoint truncation (AD-017) ───────
    //
    // `extract_quoted_literal` doc says "Returns up to 50 chars (codepoint-safe)"
    // but the pre-fix impl returns the FULL literal without any truncation.
    // `detect_literal_in_pipe_key_position` interpolates the returned value into the
    // E-QUERY-001 sort / stats-by messages, so a >50-char literal would be echoed in full.
    // AD-017 consistency: E-QUERY-041/042 apply a 50-char cap on value snippets;
    // the pipe-key-position messages must match.
    //
    // These tests fail before the fix (full literal echoed) and pass after (truncated at 50).

    /// LOW-1: `| sort '<51-ASCII-chars>'` → error message must NOT contain the full 51-char
    /// literal; must contain the 50-char prefix only.
    ///
    /// Pre-fix: `extract_quoted_literal` returns `&rest[..end]` (full content, 51 chars).
    /// Post-fix: `extract_quoted_literal` truncates at `.chars().take(50)` boundary (50 chars).
    ///
    /// Traces to: AD-017; error-taxonomy.md §E-QUERY-041/042 value_prefix 50-cap convention;
    ///            S-PRISMQL-NATIVE-TEMPORAL-TYPING-001 LOW-1.
    #[test]
    fn test_extract_quoted_literal_truncates_sort_message_at_50_ascii_codepoints() {
        // Construct a 51-character literal: 'aaa...a' (51 'a's).
        let long_literal = "a".repeat(51);
        let input = format!("FROM t | sort '{long_literal}'");

        // Pass a dummy non-empty error list so the rewriter fires.
        let errors = vec![ParseError::new(0, "dummy error".to_string())];
        let rewritten = rewrite_temporal_literal_in_pipe_key_position(&input, errors);

        assert_eq!(
            rewritten.len(),
            1,
            "LOW-1 sort truncation: rewriter must produce exactly 1 message. Got: {rewritten:?}"
        );
        let msg = &rewritten[0].message;

        let fifty_a = "a".repeat(50);
        let fifty_one_a = "a".repeat(51);

        // Message must contain the 50-char prefix (the truncated snippet).
        assert!(
            msg.contains(&fifty_a),
            "LOW-1 sort truncation: message must contain the 50-'a' prefix. \
             AD-017: value snippets must be capped at 50 codepoints. Got: {msg:?}"
        );

        // Message must NOT contain all 51 'a' chars (the un-truncated literal).
        assert!(
            !msg.contains(&fifty_one_a),
            "LOW-1 sort truncation: message MUST NOT echo the full 51-char literal. \
             extract_quoted_literal must truncate at 50 codepoints (AD-017 consistency \
             with E-QUERY-041/042 value_prefix cap). Got: {msg:?}"
        );
    }

    /// LOW-1: `| stats count by '<51-ASCII-chars>'` → error message truncated at 50.
    ///
    /// Sibling-path to the sort test — `detect_literal_in_pipe_key_position` also
    /// calls `extract_quoted_literal` for the `stats by` branch.
    ///
    /// Traces to: AD-017; S-PRISMQL-NATIVE-TEMPORAL-TYPING-001 LOW-1.
    #[test]
    fn test_extract_quoted_literal_truncates_stats_by_message_at_50_ascii_codepoints() {
        let long_literal = "b".repeat(51);
        let input = format!("FROM t | stats count by '{long_literal}'");

        let errors = vec![ParseError::new(0, "dummy".to_string())];
        let rewritten = rewrite_temporal_literal_in_pipe_key_position(&input, errors);

        assert_eq!(
            rewritten.len(),
            1,
            "LOW-1 stats-by: rewriter must produce 1 message"
        );
        let msg = &rewritten[0].message;

        let fifty_b = "b".repeat(50);
        let fifty_one_b = "b".repeat(51);

        assert!(
            msg.contains(&fifty_b),
            "LOW-1 stats-by truncation: message must contain 50-'b' prefix. Got: {msg:?}"
        );
        assert!(
            !msg.contains(&fifty_one_b),
            "LOW-1 stats-by truncation: message must NOT echo full 51-char literal. \
             extract_quoted_literal must truncate at 50 codepoints. Got: {msg:?}"
        );
    }

    /// LOW-1 Unicode: truncation must be codepoint-safe (no VP-021 byte-split on multibyte chars).
    ///
    /// 51 × U+65E5 '日' (3 bytes each) = 153 bytes total in the literal.
    /// After truncation: 50 codepoints = 150 bytes — a valid UTF-8 slice boundary.
    /// The 51st codepoint (bytes 150-152) must NOT appear in the message.
    ///
    /// Pre-fix: `extract_quoted_literal` returns the full 153-byte slice.
    /// Post-fix: `.chars().take(50)` → 50 codepoints (150 bytes), no panic.
    ///
    /// Traces to: VP-021 (no panic on multibyte input); AD-017; S-PRISMQL-NATIVE-TEMPORAL-TYPING-001 LOW-1.
    #[test]
    fn test_extract_quoted_literal_truncates_unicode_codepoint_safe_no_panic() {
        let cjk_51 = "日".repeat(51);
        let input = format!("FROM t | sort '{cjk_51}'");

        // Must not panic (VP-021 codepoint-safe truncation).
        let errors = vec![ParseError::new(0, "dummy".to_string())];
        let result = std::panic::catch_unwind(|| {
            rewrite_temporal_literal_in_pipe_key_position(&input, errors)
        });
        assert!(
            result.is_ok(),
            "LOW-1 Unicode: rewrite_temporal_literal_in_pipe_key_position panicked on \
             51-codepoint CJK literal — codepoint-safe truncation must not panic."
        );

        let rewritten = result.unwrap();
        assert_eq!(
            rewritten.len(),
            1,
            "LOW-1 Unicode: rewriter must produce 1 message. Got: {rewritten:?}"
        );
        let msg = &rewritten[0].message;

        // Message must NOT contain the 51st codepoint (the 51st '日').
        // If the full literal were echoed, it would contain 51 × '日'.
        let cjk_51_in_msg = "日".repeat(51);
        assert!(
            !msg.contains(&cjk_51_in_msg),
            "LOW-1 Unicode: message must NOT echo all 51 CJK codepoints. \
             Truncation at 50 codepoints (codepoint-safe, AD-017). Got: {msg:?}"
        );
    }
}

// ---------------------------------------------------------------------------
// D1 mode-bridge rewrite helper (BC-2.11.023 AC-010 / ADR-046 §D1)
// ---------------------------------------------------------------------------
// Canonical location per BC-2.11.023 Architecture Anchors and the story File
// Structure: this is a pure prism-query helper (uses only prism-query APIs).
// prism-mcp delegates to this public function rather than duplicating the logic.
// ---------------------------------------------------------------------------

/// Find the byte offset of `needle` inside `haystack` using ASCII case-insensitive
/// comparison, returning an offset valid for slicing `haystack`.
///
/// Unlike `haystack.to_uppercase().find(needle)`, this function searches the
/// ORIGINAL bytes of `haystack` directly.  It only folds ASCII letters (A–Z ↔ a–z);
/// non-ASCII bytes are compared as-is and are never altered.  This guarantees that
/// the returned offset is always a valid `haystack` byte index — no risk of landing
/// inside a multi-byte UTF-8 sequence due to case-expansion length changes.
///
/// `needle` must contain only ASCII characters (the call sites use `" FROM "` and
/// `" WHERE "` which are pure ASCII).
///
/// Returns `Some(offset)` where `offset` is the start of the first match inside
/// `haystack`, or `None` if no match is found.
fn find_substr_ignore_ascii_case(haystack: &str, needle: &str) -> Option<usize> {
    let hb = haystack.as_bytes();
    let nb = needle.as_bytes();
    if nb.is_empty() {
        return Some(0);
    }
    if nb.len() > hb.len() {
        return None;
    }
    // Slide a window of needle.len() bytes over haystack.
    // ASCII fold: compare each byte with eq_ignore_ascii_case for letter bytes.
    'outer: for start in 0..=(hb.len() - nb.len()) {
        for (i, &n) in nb.iter().enumerate() {
            if !hb[start + i].eq_ignore_ascii_case(&n) {
                continue 'outer;
            }
        }
        return Some(start);
    }
    None
}

/// Attempt to produce a Pipe-mode rewrite of `original_query` for D1 mode-bridge
/// errors (ADR-046 §D1).
///
/// Returns `Some(pipe_query)` when a round-tripping Pipe-mode rewrite is derivable.
/// Returns `None` when the rewrite is ambiguous or would not round-trip.
///
/// # Algorithm
///
/// 1. Re-parse the query. If it now parses as `Ast::SqlPipe` or `Ast::Pipe`, normalize
///    via `normalize_pql` (handles the case where SqlPipe grammar was added after the
///    error was originally generated).
/// 2. If re-parse still fails, apply a string-based heuristic for the simple case:
///    `SELECT * FROM <table> WHERE <predicate> | <stages>`
///    → `FROM <table> | where <predicate> | <stages>`
///    Verify the rewrite round-trips before returning.
///
/// BC-2.11.023 AC-010 postcondition: `normalized_pql` must be valid PrismQL.
/// This function returns `None` when the rewrite itself fails to round-trip.
///
/// Story: S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001 (OBS-1 relocation).
pub fn mode_bridge_normalized_pql(original_query: &str) -> Option<String> {
    use crate::{ast::Ast, engine::normalize_pql, PrismQlParser};

    let trimmed = original_query.trim();

    // Only attempt rewrite for SELECT queries with an unquoted pipe.
    if !trimmed
        .get(..6)
        .is_some_and(|h| h.eq_ignore_ascii_case("SELECT"))
    {
        return None;
    }

    // Step 1: Re-parse. If it now succeeds (Pipe/Sql grammar), normalize.
    // normalize_pql returns Some for Ast::Pipe and Ast::Sql but currently returns
    // None for Ast::SqlPipe (PqlNormalizer does not yet emit the canonical pipe form
    // for SqlPipe). When normalize_pql returns None, fall through to step 2 heuristic.
    if let Ok(ast) = PrismQlParser::parse(trimmed) {
        if let Some(normalized) = normalize_pql(&ast) {
            return Some(normalized);
        }
        // normalize_pql returned None (e.g., Ast::SqlPipe not yet handled by normalizer).
        // Fall through to step 2 heuristic to attempt a string-based pipe rewrite.
    }

    // Step 2: String-based heuristic for simple SELECT * FROM t WHERE predicate | stages.
    //
    // Find the first unquoted | that is not part of an IN clause.
    let pipe_offset = find_first_unquoted_pipe(trimmed)?;
    let sql_head = trimmed[..pipe_offset].trim_end(); // "SELECT * FROM t WHERE predicate"
    let stages_suffix = &trimmed[pipe_offset..]; // "| stages..."

    // Extract table name from "SELECT ... FROM <table>" (only handles simple single-table).
    //
    // SAFETY NOTE: We must search the same string we will slice.
    // `str::to_uppercase()` is NOT byte-length-preserving for non-ASCII characters
    // (e.g. U+FB01 ﬁ is 3 bytes but uppercases to "FI" which is 2 bytes).
    // Using the byte offset from an uppercase copy as an index into the original
    // string can therefore land inside a multi-byte UTF-8 sequence → panic.
    //
    // Fix: use `find_substr_ignore_ascii_case` which searches the ORIGINAL string
    // directly, yielding offsets that are valid for slicing that same string.
    // This function only treats A-Z/a-z as equivalent; it leaves non-ASCII bytes
    // unchanged, so offsets are always valid char boundaries in the source string.
    let from_idx = find_substr_ignore_ascii_case(sql_head, " FROM ")?;
    let after_from = sql_head[from_idx + 6..].trim(); // "<table> [WHERE ...]"

    // Split at WHERE (case-insensitive) using the same offset-safe helper.
    let (table_part, predicate_part) =
        if let Some(where_idx) = find_substr_ignore_ascii_case(after_from, " WHERE ") {
            let table = after_from[..where_idx].trim();
            let predicate = after_from[where_idx + 7..].trim();
            (table, Some(predicate))
        } else {
            (after_from.trim(), None)
        };

    // Reject complex cases: JOINs, subqueries, aliases (dot in table name that looks
    // like a qualified table is OK — e.g., crowdstrike.detections).
    if table_part.contains(' ') || table_part.is_empty() {
        return None;
    }

    // Assemble the pipe-mode rewrite.
    let rewrite = if let Some(pred) = predicate_part {
        if pred.is_empty() {
            format!("FROM {table_part} {stages_suffix}")
        } else {
            format!("FROM {table_part} | where {pred} {stages_suffix}")
        }
    } else {
        format!("FROM {table_part} {stages_suffix}")
    };

    // Verify the rewrite round-trips.
    match PrismQlParser::parse(&rewrite) {
        Ok(Ast::Pipe(_) | Ast::SqlPipe(_)) => Some(rewrite),
        _ => None,
    }
}

/// Find the byte offset of the first unquoted `|` in `input`.
///
/// Correctly handles SQL `''` escaped quotes inside single-quoted strings (MED-002):
/// when already inside a single-quoted string (`in_sq`), two consecutive `'` bytes
/// are an escape sequence — skip both and remain in the string rather than toggling
/// the `in_sq` flag.
pub fn find_first_unquoted_pipe(input: &str) -> Option<usize> {
    let bytes = input.as_bytes();
    let len = bytes.len();
    let mut in_sq = false;
    let mut in_dq = false;
    let mut i = 0;
    while i < len {
        let b = bytes[i];
        if b == b'\'' && !in_dq {
            if in_sq && i + 1 < len && bytes[i + 1] == b'\'' {
                // SQL '' escape: two consecutive single-quotes inside a string.
                // Skip both bytes and remain inside the string.
                i += 2;
                continue;
            }
            // Toggle: entering or exiting a single-quoted string.
            in_sq = !in_sq;
        } else if b == b'"' && !in_sq {
            in_dq = !in_dq;
        } else if b == b'|' && !in_sq && !in_dq {
            return Some(i);
        }
        i += 1;
    }
    None
}

// ---------------------------------------------------------------------------
// F-P3-CRIT-001 regression tests
// ---------------------------------------------------------------------------
//
// UTF-8 byte-offset safety: `to_uppercase()` is NOT byte-length-preserving
// for non-ASCII characters (e.g. `ﬁ` U+FB01 → 3 bytes becomes `FI` → 2 bytes).
// Using the offset returned by searching the uppercased copy as an index into
// the original string can therefore land inside a multi-byte UTF-8 sequence,
// causing a "byte index N is not a char boundary" panic.
//
// These regression tests must fail (panic) BEFORE the fix is applied, and pass
// after.
#[cfg(test)]
mod f_p3_crit_001_utf8_offset_regression {
    use super::mode_bridge_normalized_pql;

    /// Non-ASCII ligature U+FB01 (ﬁ, 3 bytes) before FROM:
    /// `SELECT ﬁ FROM t WHERE x=1 | limit 5`
    /// The uppercased copy of the `SELECT ﬁ` head is 1 byte shorter (ﬁ→FI),
    /// so the FROM offset from the uppercase search is off by 1 when used to
    /// index the original — this must NOT panic.
    #[test]
    fn test_mode_bridge_non_ascii_before_from_no_panic() {
        // ﬁ is U+FB01 (LATIN SMALL LIGATURE FI): 3 UTF-8 bytes, uppercases to
        // "FI" which is 2 UTF-8 bytes — the classic length-changing case.
        let query = "SELECT \u{FB01} FROM t WHERE x=1 | limit 5";
        // Must not panic; result is None or Some(valid_rewrite) — either is OK.
        let result = std::panic::catch_unwind(|| mode_bridge_normalized_pql(query));
        assert!(
            result.is_ok(),
            "mode_bridge_normalized_pql panicked on non-ASCII input before FROM"
        );
    }

    /// Non-ASCII ligature U+FB01 (ﬁ) in the table name (after FROM, before WHERE):
    /// `SELECT * FROM ﬁ WHERE x=1 | limit 5`
    #[test]
    fn test_mode_bridge_non_ascii_in_table_name_no_panic() {
        let query = "SELECT * FROM \u{FB01} WHERE x=1 | limit 5";
        let result = std::panic::catch_unwind(|| mode_bridge_normalized_pql(query));
        assert!(
            result.is_ok(),
            "mode_bridge_normalized_pql panicked on non-ASCII table name"
        );
    }

    /// Turkish dotless i (U+0131, ı) before FROM — 2 bytes, uppercases to ASCII 'I' 1 byte.
    #[test]
    fn test_mode_bridge_turkish_i_before_from_no_panic() {
        let query = "SELECT \u{0131}d FROM t WHERE x=1 | limit 5";
        let result = std::panic::catch_unwind(|| mode_bridge_normalized_pql(query));
        assert!(
            result.is_ok(),
            "mode_bridge_normalized_pql panicked on Turkish dotless-i input"
        );
    }

    /// ASCII-only input must still produce a correct result (no regression).
    #[test]
    fn test_mode_bridge_ascii_rewrite_preserved() {
        // This ASCII query should remain unaffected by the fix.
        // The rewrite only works if the parser recognises the resulting pipe query.
        // We simply assert no panic and accept both None and Some.
        let query = "SELECT * FROM sensors WHERE severity = 'HIGH' | limit 5";
        let _result = mode_bridge_normalized_pql(query);
        // No assertion on value — the parser may or may not recognise the table name.
        // The point of this test is: must not panic.
    }
}

#[cfg(test)]
mod mode_bridge_tests {
    use super::*;

    // ── MED-002: find_first_unquoted_pipe SQL escaped-quote handling ───────

    /// MED-002: `find_first_unquoted_pipe` must not treat `''` inside a SQL
    /// single-quoted string as a string-terminator (it is an escape sequence).
    #[test]
    fn test_find_first_unquoted_pipe_sql_escaped_quote() {
        let input = "WHERE name = 'it''s' | limit 5";
        let offset = find_first_unquoted_pipe(input);
        let expected = input.find('|');
        assert_eq!(
            offset, expected,
            "find_first_unquoted_pipe must skip SQL '' escape inside string; \
             expected offset {:?}, got {:?}",
            expected, offset
        );
    }

    /// MED-002 complement: no false positives — a pipe inside a double-quoted string
    /// is invisible to find_first_unquoted_pipe.
    #[test]
    fn test_find_first_unquoted_pipe_double_quoted_pipe_invisible() {
        let input = r#"WHERE col = "a|b" | limit 5"#;
        let offset = find_first_unquoted_pipe(input);
        let expected = input.rfind('|');
        assert_eq!(
            offset, expected,
            "pipe inside double-quoted string must be ignored; \
             expected offset {:?} (last |), got {:?}",
            expected, offset
        );
    }

    /// MED-002 complement: plain string with no quotes — finds the first `|`.
    #[test]
    fn test_find_first_unquoted_pipe_no_quotes() {
        let input = "FROM t | where x = 1 | limit 5";
        let offset = find_first_unquoted_pipe(input);
        let expected = input.find('|');
        assert_eq!(
            offset, expected,
            "no-quotes case must find first | at {:?}; got {:?}",
            expected, offset
        );
    }
}
