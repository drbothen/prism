//! Filter mode parser: `[source |] predicate` (BC-2.11.002).
//!
//! Grammar (prismql-grammar.md §4):
//!   filter_query := [source_ref '|'] predicate
//!   predicate    := or_expr
//!   or_expr      := and_expr ('OR' | '||' and_expr)*
//!   and_expr     := not_expr ('AND' | '&&' not_expr)*
//!   not_expr     := ('NOT' | '!') not_expr | atom
//!   atom         := '(' predicate ')' | comparison
//!   comparison   := has_check | missing_check | regex_match | cidr_match
//!                 | not_in_list | iin_list | in_list | between | is_null
//!                 | string_op_match | cidr_bare | like_match
//!                 | ieq_compare | ine_compare | field_comparison
//!
//! All keywords are case-insensitive.
//!
//! Story: S-3.01 | BC-2.11.002

use chumsky::prelude::*;
use ordered_float::OrderedFloat;

use crate::{
    ast::{
        field_path_to_expr, Ast, CidrLiteral, CompareOp, DurationLiteral, DurationUnit, FieldPath,
        FilterExpr, Literal, LogicalOp, PipeQuery, Predicate, RegexLiteral, SourceRef, Span,
        StringOp, TimestampLiteral,
    },
    error::ParseError,
    error_recovery::{
        rewrite_d2_sql_keyword_in_pipe_position, rewrite_enrich_parse_errors, rich_to_parse_error,
    },
    pipe_parser::build_pipe_parser,
    security,
    write_verb_registry::WriteVerbRegistry,
};

/// RAII guard that clears the thread-local `ParseLimits` snapshot when dropped.
///
/// `PrismQlParser::parse` installs this guard immediately after calling
/// `ParseLimits::install_thread_local` so that the thread-local is cleared
/// even if `parse_with_limits` panics (Chumsky stack overflow, OOM,
/// `unreachable!`).  Without the guard the thread-local would survive the
/// panic unwind and leak limit values into subsequent parse calls on the same
/// thread.  (F-MEDIUM-002, BC-2.11.006)
///
/// `pub(crate)` so that unit tests can import the production type and verify
/// its Drop semantics directly, without defining a local copy that wouldn't
/// catch regressions in the real guard.  (F-MEDIUM-001)
pub(crate) struct ThreadLocalGuard;

impl Drop for ThreadLocalGuard {
    fn drop(&mut self) {
        security::ParseLimits::clear_thread_local();
    }
}

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
    /// - `check_paren_depth` is called before parsing to cap structural depth.
    ///
    /// # Errors
    /// Returns `Err(Vec<ParseError>)` if the input is syntactically invalid or
    /// exceeds security limits.
    pub fn parse(input: &str) -> Result<Ast, Vec<ParseError>> {
        // F-LOW-002 / F-HIGH-001: Snapshot all effective limits ONCE before any guard runs.
        // All security guards within this parse call use the same snapshotted values,
        // preventing concurrent env-var mutations from causing limit inconsistencies.
        let limits = security::ParseLimits::snapshot();

        // F-HIGH-001: Install the snapshot as the thread-local limit so that
        // AST-construction-time checks (e.g. RegexLiteral::new) also see the
        // snapshotted value rather than re-reading the env var.
        limits.install_thread_local();

        // F-MEDIUM-002: Use a Drop guard so the thread-local is cleared even
        // if `parse_with_limits` panics (Chumsky stack overflow, OOM, unreachable!).
        // Without the guard the thread-local would survive the panic unwind and leak
        // limit values into subsequent parse calls on the same thread.
        // The guard type is `pub(crate)` (ThreadLocalGuard at module level) so that
        // unit tests can exercise the production type directly.  (F-MEDIUM-001)
        let _guard = ThreadLocalGuard;

        Self::parse_with_limits(input, &limits)
        // _guard drops here (or on panic unwind), clearing the thread-local.
    }

    /// Parse a PrismQL query string with write-mode awareness.
    ///
    /// Applies the same pre-parse security guards as `parse`, then routes
    /// pipe-mode queries through `parse_pipe_with_write` so that a terminal
    /// write verb produces a `PipeQuery { write: Some(WriteNode) }` instead of
    /// a plain read-only `PipeQuery { write: None }`.
    ///
    /// Filter-mode queries with a write verb after `|` are rejected with
    /// `E-QUERY-010` via `reject_write_verbs_in_filter`.
    ///
    /// DML mode (INSERT/UPDATE/DELETE) and SQL SELECT mode are unaffected by
    /// the registry — write verbs do not apply there.
    ///
    /// # Design note
    /// `parse_with_registry` is a stateless pure function (no `&mut self`,
    /// no persistent state) consistent with the purity boundary defined in
    /// BC-2.11.004 and the `PrismQlParser` design.  The registry is passed per
    /// call, not stored in the parser.
    ///
    /// # Implements BC-2.11.004 — Write Parser Extension (F-PR130-CR-001)
    pub fn parse_with_registry(
        input: &str,
        registry: &WriteVerbRegistry,
    ) -> Result<Ast, Vec<ParseError>> {
        let limits = security::ParseLimits::snapshot();
        limits.install_thread_local();
        let _guard = ThreadLocalGuard;

        // Pre-parse security guards (identical to `parse`).
        limits
            .check_query_size(input)
            .map_err(|e| vec![ParseError::new(0, e.to_string())])?;
        limits
            .check_paren_depth(input)
            .map_err(|e| vec![ParseError::new(0, e.to_string())])?;

        let trimmed = input.trim();
        if trimmed.is_empty() {
            return Err(vec![ParseError::new(0, "E-QUERY-001: empty query string")]);
        }

        let first_token = trimmed.split_ascii_whitespace().next().unwrap_or("");
        let first_token_upper = first_token.to_uppercase();

        // DML mode: delegate as-is (write verbs not applicable here).
        if matches!(first_token_upper.as_str(), "INSERT" | "UPDATE" | "DELETE") {
            return parse_dml_internal(input, &limits);
        }

        // SQL SELECT mode: route through the shared mode-detection + D1/D2/enrich helper.
        // BC-2.11.023 MED-1: parse_with_registry is a co-equal public entry point and MUST
        // apply the same D1 mode-bridge diagnostic (and SqlPipe D2/enrich rewrites) as
        // parse_with_limits.  Both entry points now delegate to parse_select_mode so there
        // is only ONE implementation of the mode-bridge logic. (TD-VSDD-060 sibling-sweep)
        if first_token_upper == "SELECT" {
            return parse_select_mode(input, trimmed, &limits);
        }

        // F-PR130-P1-HIGH-001: Apply the same SQL denylist enforced by parse_with_limits.
        // This is mandatory: parse_with_registry is a co-equal public entry point
        // (see lib.rs §"External consumers MUST use PrismQlParser::parse or
        // PrismQlParser::parse_with_registry"). Both must reject denied SQL keywords
        // with E-QUERY-002. Without this check, inputs like "MERGE INTO foo" would
        // fall through to filter mode, producing E-QUERY-001 instead of E-QUERY-002.
        // (BC-2.11.003, Invariant DI-019)
        check_denied_keywords(&first_token_upper)?;

        // Pipe mode: route through parse_pipe_with_write.
        if first_token_upper == "FROM" || trimmed.starts_with('|') || is_pipe_mode(trimmed) {
            let pq = crate::pipe_parser::parse_pipe_with_write(input, registry, &limits)?;
            return Ok(Ast::Pipe(pq));
        }

        // Filter mode: reject write verbs, then parse normally.
        reject_write_verbs_in_filter(input, registry)?;
        parse_filter_internal(input, &limits)
    }

    /// Inner parse implementation that receives the already-snapshotted limits.
    ///
    /// Separated from `parse` so that the thread-local cleanup in `parse` is
    /// guaranteed to run even when this function returns early (both Ok and Err).
    fn parse_with_limits(
        input: &str,
        limits: &security::ParseLimits,
    ) -> Result<Ast, Vec<ParseError>> {
        // Security check: reject oversized queries before any parsing.
        limits
            .check_query_size(input)
            .map_err(|e| vec![ParseError::new(0, e.to_string())])?;

        // Security check: parenthesis nesting depth (EC-002, BC-2.11.006, VP-015).
        limits
            .check_paren_depth(input)
            .map_err(|e| vec![ParseError::new(0, e.to_string())])?;

        // Reject empty / whitespace-only queries.
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return Err(vec![ParseError::new(0, "E-QUERY-001: empty query string")]);
        }

        // Extract the first token once, used for mode detection and denylist.
        let first_token = trimmed.split_ascii_whitespace().next().unwrap_or("");
        let first_token_upper = first_token.to_uppercase();

        // S-3.06: Route DML statements (INSERT/UPDATE/DELETE) to the DML parser
        // BEFORE the general denylist check. These were previously denied outright
        // (BC-2.11.003 "read-only engine") but S-3.06 extends the engine with
        // write support. The DML parser enforces its own guards (E-QUERY-010, E-QUERY-022).
        if matches!(first_token_upper.as_str(), "INSERT" | "UPDATE" | "DELETE") {
            return parse_dml_internal(input, limits);
        }

        // Reject denied SQL statements before any parsing (BC-2.11.003, Invariant DI-019).
        // Shared helper — same list enforced by `parse_with_registry` (F-PR130-P1-HIGH-001).
        check_denied_keywords(&first_token_upper)?;

        // Mode detection:
        // 1. Starts with SELECT (case-insensitive) → SQL mode.
        // 2. Starts with FROM (case-insensitive) → Pipe mode.
        // 3. Starts with `|` → Pipe mode (no source prefix).
        // 4. Contains pipe stage keywords after `|` → Pipe mode.
        // 5. Otherwise → Filter mode.
        //
        // `first_token_upper` is the uppercase of the first whitespace-separated
        // token; it is the same as `trimmed.to_uppercase().split_whitespace().next()`.
        if first_token_upper == "SELECT" {
            // Delegate to the shared SELECT mode-detection + D1/D2/enrich helper so that
            // parse_with_limits and parse_with_registry share ONE implementation of the
            // mode-bridge diagnostics (BC-2.11.023 MED-1 / TD-VSDD-060 sibling-sweep).
            return parse_select_mode(input, trimmed, limits);
        }
        if first_token_upper == "FROM" || trimmed.starts_with('|') {
            return parse_pipe_internal(input, limits);
        }

        // Detect pipe-vs-filter: if there's a `|` and the token after it is a
        // pipe stage keyword, route to pipe mode.
        if is_pipe_mode(trimmed) {
            return parse_pipe_internal(input, limits);
        }

        // Filter mode.
        parse_filter_internal(input, limits)
    }
}

/// Check the first token of a query against the SQL denylist (BC-2.11.003, DI-019).
///
/// Used by both `parse_with_limits` (internal path) and `parse_with_registry`
/// (public write-aware entry point) so both public APIs enforce identical denylist
/// semantics. (F-PR130-P1-HIGH-001)
///
/// # Match semantics
/// - Case-insensitive (caller provides the uppercased first token).
/// - Full-token match (NOT substring). `INSERTED_AT` is NOT rejected;
///   `INSERT` (a full token) IS rejected via the DML routing that happens first.
///
/// The denylist covers ~33 keywords across 7 categories:
///   DML mutations (MERGE/REPLACE/UPSERT/COPY — INSERT/UPDATE/DELETE are routed
///   to the DML parser before this check fires),
///   DDL, TCL, DCL, Procedural, Diagnostic/utility, Vendor.
///
/// Returns `Err(vec![E-QUERY-002])` for any denied keyword, `Ok(())` otherwise.
fn check_denied_keywords(first_token_upper: &str) -> Result<(), Vec<ParseError>> {
    const DENIED_KEYWORDS: &[&str] = &[
        // DML mutations (INSERT/UPDATE/DELETE routed to DML parser before this call)
        "MERGE",
        "REPLACE",
        "UPSERT",
        "COPY",
        // DDL
        "CREATE",
        "DROP",
        "ALTER",
        "RENAME",
        "TRUNCATE",
        "COMMENT",
        // TCL (Transaction Control)
        "COMMIT",
        "ROLLBACK",
        "SAVEPOINT",
        "RELEASE",
        "BEGIN",
        "START",
        // DCL (Data Control)
        "GRANT",
        "REVOKE",
        "DENY",
        // Procedural
        "EXECUTE",
        "CALL",
        "DO",
        "PERFORM",
        // Diagnostic / utility
        "EXPLAIN",
        "ANALYZE",
        "VACUUM",
        "LOCK",
        "REINDEX",
        "SET",
        "SHOW",
        "USE",
        // Vendor extensions
        "PRAGMA",
        "ATTACH",
        "DETACH",
    ];
    for keyword in DENIED_KEYWORDS {
        if first_token_upper == *keyword {
            return Err(vec![ParseError::new(
                0,
                format!(
                    "E-QUERY-002: Only SELECT queries are supported. \
                     Prism is a read-only query engine. Denied keyword: `{keyword}`."
                ),
            )]);
        }
    }
    Ok(())
}

/// Pipe-stage keywords used by `is_pipe_mode`.
///
/// All entries are ASCII lowercase. Comparison uses `eq_ignore_ascii_case`
/// so "WHERE", "Where", "wHERE" all match, but non-ASCII lookalikes do not.
/// This is intentional: PrismQL is ASCII-only; full Unicode case-fold
/// (to_lowercase) would introduce inconsistency with the rest of the codebase
/// and risks false matches on Unicode homoglyphs. (F-LOW-001)
const PIPE_STAGE_KEYWORDS: &[&str] = &[
    "where", "sort", "head", "tail", "stats", "dedup", "fields", "join", "enrich", "limit",
];

/// Detect whether the input is pipe mode by looking for a `|` followed by
/// a pipe stage keyword.
///
/// # Performance (F-HIGH-001)
/// This function is a single-pass byte iterator with **zero heap allocation**.
/// The previous implementation allocated a `Vec<char>` for the full input
/// and, on every unquoted `|`, allocated a `String` of the remaining bytes
/// plus called `to_lowercase()`. With ~32K pipes in a 64KB input that was
/// ~32K heap allocations totalling ~2GB of transient memory.
///
/// The new implementation:
/// - Walks `input.as_bytes()` once.
/// - Tracks `in_sq` / `in_dq` quote state via byte equality.
/// - On an unquoted `|` at byte offset `i`, checks the next ≤ 10 bytes
///   via `input.get(i+1..)` and `eq_ignore_ascii_case` against the keyword
///   list.  No `Vec`, no `String`, no `to_lowercase()` per match.
///
/// # Case sensitivity (F-LOW-001)
/// Uses `eq_ignore_ascii_case` (ASCII-only). Non-ASCII Unicode variants of
/// keywords are NOT recognised — matching the codebase convention.
fn is_pipe_mode(input: &str) -> bool {
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
                // Skip whitespace after `|` (ASCII only — PrismQL is ASCII).
                let mut j = i + 1;
                while j < len && (bytes[j] == b' ' || bytes[j] == b'\t' || bytes[j] == b'\n') {
                    j += 1;
                }
                // Check each keyword against the bytes starting at j.
                // `eq_ignore_ascii_case` on str slices is safe and allocation-free.
                if let Some(rest) = input.get(j..) {
                    for kw in PIPE_STAGE_KEYWORDS {
                        let kw_len = kw.len();
                        if let Some(candidate) = rest.get(..kw_len) {
                            if candidate.eq_ignore_ascii_case(kw) {
                                // Must be followed by whitespace, end-of-input, or `|`.
                                let after_kw = rest.get(kw_len..).unwrap_or("");
                                if after_kw.is_empty()
                                    || matches!(
                                        after_kw.as_bytes().first(),
                                        Some(b' ' | b'\t' | b'\n' | b'|')
                                    )
                                {
                                    return true;
                                }
                            }
                        }
                    }
                }
            }
            _ => {}
        }
        i += 1;
    }
    false
}

/// Detect whether a `SELECT …` input should be parsed as SqlPipe mode
/// (BC-2.11.020, ADR-043) rather than pure SQL mode.
///
/// Returns `true` when ALL of:
/// 1. The input starts with `SELECT` (case-insensitive).
/// 2. There is at least one unquoted `|` in the input.
/// 3. The token immediately following that unquoted `|` (after optional
///    whitespace) is a recognised pipe stage keyword.
///
/// Condition 3 is the key discriminant: it prevents `WHERE x | y` (bitwise OR
/// inside a SQL expression) from being mis-classified as SqlPipe.  Only `|`
/// followed by a stage keyword triggers SqlPipe routing.
///
/// # Invariant (BC-2.11.020 additive invariant)
/// Pure SQL without pipe-stage `|` MUST still return `false` and be routed to
/// the SQL parser.  The check is conservative: if uncertain, fall through to
/// SQL mode.
fn is_sqlpipe_mode(input: &str) -> bool {
    let trimmed = input.trim();
    // Must start with SELECT (case-insensitive).
    if !trimmed
        .get(..6)
        .is_some_and(|h| h.eq_ignore_ascii_case("SELECT"))
    {
        return false;
    }
    // Reuse the same unquoted-`|`-scan logic as `is_pipe_mode`.
    // We must also ensure the character after `SELECT` is whitespace (not
    // `SELECTOR` etc.), but `is_pipe_mode` only checks for pipe-keyword after
    // `|`, so we can delegate directly.
    is_pipe_mode(trimmed)
}

/// Parse a SQL→Pipe composition query: `SELECT … | stage | stage …`.
///
/// Algorithm:
/// 1. Find the first unquoted `|` that is followed by a pipe stage keyword.
/// 2. Split the input at that position: `sql_head` | `stages_suffix`.
/// 3. Parse `sql_head` as SQL (`parse_sql_internal`).
/// 4. Parse `stages_suffix` as pipe stages (`build_pipe_stages_parser`).
/// 5. Combine into `Ast::SqlPipe(SqlPipeQuery { head, stages })`.
///
/// # Errors
/// Returns SQL parse errors or pipe stage parse errors if either half fails.
///
/// # Clippy exemption (OBS-002)
/// Same rationale as `parse_filter_internal`.
#[allow(clippy::disallowed_methods)]
fn parse_sqlpipe_internal(
    input: &str,
    limits: &security::ParseLimits,
) -> Result<Ast, Vec<ParseError>> {
    use crate::ast::{SqlPipeQuery, SqlStatement};

    // Step 1: Find the split offset — first unquoted `|` followed by a pipe
    // stage keyword.
    let split_offset = find_sqlpipe_split(input).ok_or_else(|| {
        vec![ParseError::new(
            0,
            "E-QUERY-001: SqlPipe mode detected but could not locate pipe stage boundary",
        )]
    })?;

    let sql_head_str = input[..split_offset].trim_end();
    let stages_str = &input[split_offset..]; // starts with `|`

    // Step 2: Parse SQL head.
    let sql_ast = crate::sql_parser::parse_sql_with_limits(sql_head_str, limits)?;
    let head = match sql_ast {
        Ast::Sql(SqlStatement::Select(sq)) => sq,
        _ => {
            return Err(vec![ParseError::new(
                0,
                "E-QUERY-001: SqlPipe head must be a SELECT statement",
            )]);
        }
    };

    // Step 3: Parse pipe stages suffix.
    let stage_parser = crate::pipe_parser::build_pipe_stages_parser();
    let (stage_result, stage_errs) = stage_parser.parse(stages_str).into_output_errors();
    if !stage_errs.is_empty() {
        let errs: Vec<ParseError> = stage_errs.iter().map(rich_to_parse_error).collect();
        // AC-025: apply the same guided-error rewrites that parse_pipe_with_limits uses,
        // so SqlPipe-routed pipe-stage errors receive the same actionable messages as
        // pure-pipe errors (BC-2.11.023 §D2 and AC-022/AC-025 "in all pipeline positions").
        // D2 rewrite takes precedence (applied first); enrich rewrite follows.
        let errs = rewrite_d2_sql_keyword_in_pipe_position(stages_str, errs);
        let errs = rewrite_enrich_parse_errors(stages_str, errs);
        return Err(errs);
    }
    let stages = stage_result.ok_or_else(|| {
        vec![ParseError::new(
            split_offset,
            "E-QUERY-001: SqlPipe stage list failed to parse",
        )]
    })?;

    // Security: check stage count.
    limits
        .check_pipe_stage_count_with(&stages)
        .map_err(|e| vec![ParseError::new(0, e.to_string())])?;

    Ok(Ast::SqlPipe(SqlPipeQuery { head, stages }))
}

/// Find the byte offset of the first unquoted `|` in `input` that is followed
/// (after optional ASCII whitespace) by a recognised pipe stage keyword.
///
/// Returns `None` if no such `|` exists.
pub(crate) fn find_sqlpipe_split(input: &str) -> Option<usize> {
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
                    for kw in PIPE_STAGE_KEYWORDS {
                        let kw_len = kw.len();
                        if let Some(candidate) = rest.get(..kw_len) {
                            if candidate.eq_ignore_ascii_case(kw) {
                                let after_kw = rest.get(kw_len..).unwrap_or("");
                                if after_kw.is_empty()
                                    || matches!(
                                        after_kw.as_bytes().first(),
                                        Some(b' ' | b'\t' | b'\n' | b'|' | b'(')
                                    )
                                {
                                    return Some(i);
                                }
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

/// Return `true` if `input` contains at least one unquoted `|` character.
///
/// Used by the BC-2.11.023 mode-bridge heuristic to detect `SELECT … | token`
/// patterns that should emit a D1 diagnostic instead of a raw Chumsky dump.
fn contains_unquoted_pipe(input: &str) -> bool {
    find_unquoted_pipe_offset(input).is_some()
}

/// Return the byte offset of the first unquoted `|` in `input`, or `None`.
fn find_unquoted_pipe_offset(input: &str) -> Option<usize> {
    let bytes = input.as_bytes();
    let len = bytes.len();
    let mut in_sq = false;
    let mut in_dq = false;
    for (i, &b) in bytes.iter().enumerate().take(len) {
        match b {
            b'\'' if !in_dq => in_sq = !in_sq,
            b'"' if !in_sq => in_dq = !in_dq,
            b'|' if !in_sq && !in_dq => return Some(i),
            _ => {}
        }
    }
    None
}

/// Parse filter mode internally, wrapping result as `Ast::Filter`.
///
/// # Clippy exemption (OBS-002)
/// `disallowed_methods` fires on `parse_filter` because it bypasses the
/// pre-parse guards. This call IS the sanctioned internal routing path inside
/// `PrismQlParser::parse`, which has already applied the guards. The exemption
/// is intentional and scoped to this helper.
#[allow(clippy::disallowed_methods)]
fn parse_filter_internal(
    input: &str,
    limits: &security::ParseLimits,
) -> Result<Ast, Vec<ParseError>> {
    parse_filter_with_limits(input, limits).map(Ast::Filter)
}

/// Parse SQL mode internally — delegates to `parse_sql` which returns `Ast::Sql(...)` directly.
///
/// # Clippy exemption (OBS-002)
/// Same rationale as `parse_filter_internal`. Guards are applied by the caller
/// (`PrismQlParser::parse`) before dispatching here.
#[allow(clippy::disallowed_methods)]
fn parse_sql_internal(input: &str, limits: &security::ParseLimits) -> Result<Ast, Vec<ParseError>> {
    crate::sql_parser::parse_sql_with_limits(input, limits)
}

/// Parse pipe mode internally, wrapping result as `Ast::Pipe`.
///
/// # Clippy exemption (OBS-002)
/// Same rationale as `parse_filter_internal`. Guards are applied by the caller
/// (`PrismQlParser::parse`) before dispatching here.
#[allow(clippy::disallowed_methods)]
fn parse_pipe_internal(
    input: &str,
    limits: &security::ParseLimits,
) -> Result<Ast, Vec<ParseError>> {
    crate::pipe_parser::parse_pipe_with_limits(input, limits).map(Ast::Pipe)
}

/// Parse DML mode internally — delegates to `parse_sql_dml_with_limits` (S-3.06).
///
/// Routes INSERT/UPDATE/DELETE statements to the DML parser added in S-3.06.
/// Pre-parse guards (size, paren depth) have already been applied by the caller;
/// this function forwards the snapshotted `limits` so post-parse depth and
/// list-size guards run on any embedded `SqlQuery` (F-PR130-CR-004, SEC-002).
///
/// # Clippy exemption (OBS-002)
/// Same rationale as `parse_filter_internal`.
#[allow(clippy::disallowed_methods)]
fn parse_dml_internal(input: &str, limits: &security::ParseLimits) -> Result<Ast, Vec<ParseError>> {
    crate::sql_parser::parse_sql_dml_with_limits(input, limits)
}

/// Shared SELECT-mode routing for both `parse_with_limits` and `parse_with_registry`.
///
/// Both public entry points (`PrismQlParser::parse` and
/// `PrismQlParser::parse_with_registry`) detect `SELECT` as the first token and
/// must apply identical mode-detection and error-rewrite logic (BC-2.11.023 MED-1,
/// TD-VSDD-060 sibling-sweep).  Centralising the logic here eliminates the
/// divergence that caused MED-1.
///
/// Algorithm:
/// 1. If the query is SqlPipe (SELECT … | stage …), route to `parse_sqlpipe_internal`
///    which already applies the D2 and enrich rewrites on its stage-suffix errors.
/// 2. Otherwise attempt pure-SQL parse.
///    - On success: return the `Ast::Sql` result unchanged.
///    - On failure + unquoted `|` present: replace raw Chumsky dump with the verbatim
///      BC-2.11.023 §D1 mode-bridge message (POL-24).
///    - On failure without `|`: return the raw SQL parse errors.
///
/// # Pre-conditions
/// - `input` is the original (untrimmed) query string.
/// - `trimmed` is `input.trim()` (caller must pre-compute to avoid duplicate allocations).
/// - Pre-parse guards (size, depth) have already been applied by the caller.
/// - `first_token_upper == "SELECT"` — caller is responsible for this check.
///
/// # Clippy exemption (OBS-002)
/// Same rationale as `parse_filter_internal`.
#[allow(clippy::disallowed_methods)]
fn parse_select_mode(
    input: &str,
    trimmed: &str,
    limits: &security::ParseLimits,
) -> Result<Ast, Vec<ParseError>> {
    // BC-2.11.020: If the input contains pipe stage keywords after an unquoted `|`,
    // route to SqlPipe mode (SELECT … | stage | stage …).  Pure SELECT (no pipe
    // stage `|`) falls through to the SQL parser unchanged.
    if is_sqlpipe_mode(trimmed) {
        return parse_sqlpipe_internal(input, limits);
    }

    // BC-2.11.023 / ADR-046 §D1 — mode-bridge diagnostic:
    // If SQL parse fails AND the input contains an unquoted `|`, replace the raw
    // Chumsky token-dump with an actionable message explaining that pipe stages are
    // not valid after a SQL SELECT query in SQL mode.
    let sql_result = parse_sql_internal(input, limits);
    match sql_result {
        ok @ Ok(_) => ok,
        Err(_errs) if contains_unquoted_pipe(trimmed) => {
            // BC-2.11.023 §D1 — verbatim mode-bridge message (POL-24).
            Err(vec![ParseError::new(
                find_unquoted_pipe_offset(trimmed).unwrap_or(0),
                "E-QUERY-001: parse error near '|': pipe stages are not valid after a SQL SELECT query in SQL mode.\n\
                 To use pipe stages (enrich, where, limit, sort, stats, dedup, fields), use one of:\n\
                   1. SQL+pipe composition:  SELECT <cols> FROM <table> | <pipe_stage> \u{2026}\n\
                   2. Pipe mode only:        FROM <table> | where <predicate> | <stage> \u{2026}\n\
                 See prismql://reference for the complete grammar.",
            )])
        }
        err => err,
    }
}

// ── Security re-export for convenient use in tests ────────────────────────────
pub use security::{PRISM_MAX_NESTING_DEPTH, PRISM_MAX_QUERY_SIZE};

/// Parse a filter-mode query: `[source |] predicate` or just `predicate`.
///
/// Called by `PrismQlParser::parse` after mode detection confirms filter mode.
///
/// # Security perimeter (SEC-C-003, F-LOW-002)
/// This function is `pub(crate)` to enforce that callers outside `prism-query`
/// use `PrismQlParser::parse` exclusively. Direct callers bypass the mandatory
/// pre-parse security guards (`check_query_size`, `check_paren_depth`).
///
/// # Errors
/// Returns accumulated `ParseError`s on failure.
// Used by src/tests/ — dead_code fires in non-test compilation but not in tests.
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn parse_filter(input: &str) -> Result<FilterExpr, Vec<ParseError>> {
    // When called directly (bypassing PrismQlParser::parse), use env-var limits.
    let limits = security::ParseLimits::snapshot();
    parse_filter_with_limits(input, &limits)
}

/// Parse a filter-mode query using the provided snapshotted limits (F-HIGH-001).
///
/// This is the race-free variant used by `PrismQlParser::parse`. All post-parse
/// security guards use the caller-provided `limits` snapshot instead of re-reading
/// env vars.
///
/// # Thread-local protocol (OBS-002)
/// When called via `PrismQlParser::parse`, the thread-local `ParseLimits` snapshot
/// is pre-installed by the caller (via `install_thread_local`) and cleared by the
/// `ThreadLocalGuard` Drop guard. `RegexLiteral::new` therefore uses the snapshotted
/// regex limit during AST construction.
///
/// When called directly from tests (bypassing `PrismQlParser::parse`), the
/// thread-local is NOT installed; `RegexLiteral::new` falls back to the env-var path
/// via `effective_regex_pattern_length_limit()`. Test code that depends on snapshot
/// semantics must call `ParseLimits::install_thread_local()` and the matching
/// `ParseLimits::clear_thread_local()` itself before/after the call.
pub(crate) fn parse_filter_with_limits(
    input: &str,
    limits: &security::ParseLimits,
) -> Result<FilterExpr, Vec<ParseError>> {
    let parser = build_filter_parser();
    let (result, errs) = parser.parse(input).into_output_errors();
    if errs.is_empty() {
        if let Some(fe) = result {
            // Security: check nesting depth on parsed predicate (race-free via snapshot).
            limits
                .check_predicate_nesting_depth_with(&fe.predicate, 0)
                .map_err(|e| vec![ParseError::new(0, e.to_string())])?;
            // Security: check IN list sizes (race-free via snapshot).
            limits
                .check_filter_list_sizes_with(&fe)
                .map_err(|e| vec![ParseError::new(0, e.to_string())])?;
            return Ok(fe);
        }
    }
    // Convert Chumsky errors to ParseErrors.
    let parse_errors: Vec<ParseError> = errs.iter().map(rich_to_parse_error).collect();
    if parse_errors.is_empty() {
        Err(vec![ParseError::new(0, "E-QUERY-001: parse failed")])
    } else {
        Err(parse_errors)
    }
}

/// Build the Chumsky filter-mode parser.
///
/// Returns a parser that accepts `[source_ref '|'] predicate`.
fn build_filter_parser<'a>(
) -> impl Parser<'a, &'a str, FilterExpr, extra::Err<Rich<'a, char>>> + Clone {
    let predicate = build_predicate_parser();

    // source_ref: dotted identifier, e.g. `crowdstrike.detections`
    let source_ref = build_source_ref_parser();

    // Optional `source_ref '|'` prefix.
    let with_source = source_ref
        .then_ignore(just('|').padded())
        .then(predicate.clone())
        .map(|(src, pred)| FilterExpr {
            source: src,
            predicate: pred,
        });

    // Filter without source prefix: just a predicate.
    let without_source = predicate.map(|pred| FilterExpr {
        source: SourceRef::from_raw(""),
        predicate: pred,
    });

    with_source.or(without_source)
}

/// Build the source reference parser (dotted-ident, rejects path traversal).
pub(crate) fn build_source_ref_parser<'a>(
) -> impl Parser<'a, &'a str, SourceRef, extra::Err<Rich<'a, char>>> + Clone {
    let segment = any::<&str, extra::Err<Rich<char>>>()
        .filter(|c: &char| c.is_ascii_alphanumeric() || *c == '_')
        .repeated()
        .at_least(1)
        .to_slice();

    segment
        .separated_by(just('.'))
        .at_least(1)
        .collect::<Vec<_>>()
        .to_slice()
        .try_map(|raw: &str, span| {
            // Reject path traversal: `..`, `/`, `\`.
            if raw.contains("..") || raw.contains('/') || raw.contains('\\') {
                return Err(Rich::custom(
                    span,
                    "EC-004: SourceRef contains path traversal characters ('..', '/', '\\')",
                ));
            }
            Ok(SourceRef::from_raw(raw))
        })
}

/// Build the predicate parser (boolean tree over field conditions).
///
/// This is the parser used for filter mode, pipe `where` stages, and
/// SQL WHERE / HAVING clauses.
#[allow(clippy::clone_on_copy)]
pub(crate) fn build_predicate_parser<'a>(
) -> impl Parser<'a, &'a str, Predicate, extra::Err<Rich<'a, char>>> + Clone {
    recursive(|predicate| {
        let literal = build_literal_parser();

        // Field path: dotted identifier supporting underscores and leading underscores.
        let ident_char = any::<&str, extra::Err<Rich<char>>>()
            .filter(|c: &char| c.is_ascii_alphanumeric() || *c == '_');

        let field_segment = ident_char.repeated().at_least(1).to_slice();

        let field_path = field_segment
            .separated_by(just('.'))
            .at_least(1)
            .collect::<Vec<&str>>()
            .map_with(|segs: Vec<&str>, e| {
                // Capture the actual byte-offset span from Chumsky (CR F-CR-007).
                let s = e.span();
                FieldPath {
                    segments: segs.into_iter().map(|seg| seg.to_string()).collect(),
                    span: Span {
                        start: s.start,
                        end: s.end,
                    },
                }
            });

        // Case-insensitive keyword helper.
        let kw = |k: &'static str| {
            any::<&str, extra::Err<Rich<char>>>()
                .filter(move |c: &char| c.is_ascii_alphabetic() || *c == '_')
                .repeated()
                .at_least(1)
                .to_slice()
                .try_map(move |s: &str, span| {
                    if s.eq_ignore_ascii_case(k) {
                        Ok(())
                    } else {
                        Err(Rich::custom(span, format!("expected keyword '{k}'")))
                    }
                })
        };

        // Compare operator (prismql-grammar.md §4.1).
        let compare_op = choice((
            just(">=").to(CompareOp::Ge),
            just("<=").to(CompareOp::Le),
            just("!=").to(CompareOp::Ne),
            just("==").to(CompareOp::Eq),
            just('>').to(CompareOp::Gt),
            just('<').to(CompareOp::Lt),
            just('=').to(CompareOp::Eq),
        ))
        .padded();

        // Quoted string literal for operator arguments (CIDR, regex, CONTAINS, etc.)
        let string_val = build_string_parser();

        // --- HAS field ---
        let has_check = kw("HAS")
            .padded()
            .ignore_then(field_path.clone().padded())
            .map(Predicate::Has);

        // --- MISSING field ---
        let missing_check = kw("MISSING")
            .padded()
            .ignore_then(field_path.clone().padded())
            .map(Predicate::Missing);

        // --- field =~ "regex" | field MATCHES "regex" ---
        let regex_match = field_path
            .clone()
            .padded()
            .then(choice((
                just("=~").padded().to(()),
                kw("MATCHES").padded().to(()),
            )))
            .then(string_val.clone().padded())
            .try_map(|((fp, ()), pat), span| {
                RegexLiteral::new(&pat)
                    .map(|rl| Predicate::Regex {
                        field: fp,
                        pattern: rl,
                    })
                    .map_err(|e| Rich::custom(span, e))
            });

        // --- field IN CIDR "10.0.0.0/8" ---
        let cidr_match = field_path
            .clone()
            .padded()
            .then_ignore(kw("IN").padded())
            .then_ignore(kw("CIDR").padded())
            .then(string_val.clone().padded())
            .try_map(|(fp, cidr_str), span| {
                CidrLiteral::new(&cidr_str)
                    .map(|cl| Predicate::Cidr {
                        field: fp,
                        cidr: cl,
                        negated: false,
                    })
                    .map_err(|e| Rich::custom(span, e))
            });

        // --- field NOT IN (val, …) ---
        let not_in_list = field_path
            .clone()
            .padded()
            .then_ignore(kw("NOT").padded())
            .then_ignore(kw("IN").padded())
            .then(
                literal
                    .clone()
                    .padded()
                    .separated_by(just(',').padded())
                    .at_least(1)
                    .collect::<Vec<_>>()
                    .delimited_by(just('(').padded(), just(')').padded()),
            )
            .map(|(fp, values)| Predicate::In {
                field: fp,
                values,
                negated: true,
                case_insensitive: false,
            });

        // --- field IIN (val, …) — case-insensitive set membership (S-PRISMQL-CASE-INSENSITIVE-001) ---
        // IIN must appear before IN in the atom choice so that 'iIN' / 'IIN' tokens are
        // consumed by this combinator. The kw() helper uses full-run eq_ignore_ascii_case,
        // so 'IIN' will not match kw("IN") (different length), but ordering is retained
        // as defensive practice per BC-2.11.002 IIN-before-IN discipline.
        //
        // Empty list (BC-2.11.024 error case "E-QUERY-001: IIN with empty membership list";
        // AC-021): use `.collect::<Vec<_>>()` (no `.at_least(1)`) so the delimited parser
        // accepts "()" without a Chumsky-level failure, then apply `try_map` at the
        // `delimited_by` level so the error span begins at '(' (e.g., position 13).
        // This ensures the E-QUERY-001 error wins choice() error-merging over the generic
        // "expected keyword 'MATCHES'" error at position 9 (Chumsky 0.12 selects by span.start).
        let iin_values = literal
            .clone()
            .padded()
            .separated_by(just(',').padded())
            .collect::<Vec<_>>()
            .delimited_by(just('(').padded(), just(')').padded())
            .try_map(|values, span| {
                if values.is_empty() {
                    return Err(Rich::custom(
                        span,
                        "E-QUERY-001: IIN operator requires at least one value in the \
                         membership list; empty () is not valid. \
                         Use at least one quoted string: e.g. `status IIN ('new', 'open')`.",
                    ));
                }
                // F-LOW-001: `classify_string_literal` (ADR-052 §D4) may reclassify
                // quoted date-like values (e.g. '2026-06-01') to `RawTemporalLiteral`
                // or `Timestamp`.  IIN values are always string/label comparisons; the
                // user's intent is a quoted string in a set membership test.  Normalise
                // both forms to `Literal::String` so the emitter can call `lower()` on
                // each element.  Temporal semantics are intentionally NOT applied here —
                // the `ci` (case-insensitive) emit path emits `lower('2026-06-01')`, not
                // `arrow_cast(...)`.
                let values: Vec<Literal> = values
                    .into_iter()
                    .map(|lit| match lit {
                        Literal::RawTemporalLiteral(s) => Literal::String(s),
                        Literal::Timestamp(ts) => Literal::String(ts.iso8601),
                        other => other,
                    })
                    .collect();
                // BC-2.11.024 F-MED-001: reject non-string values at parse time.
                // After temporal normalisation, only `Literal::String` is valid in an IIN
                // membership list.  Integer, Float, and Boolean literals are rejected here
                // so the analyst receives a clear, actionable error before SQL generation.
                if values.iter().any(|lit| !matches!(lit, Literal::String(_))) {
                    return Err(Rich::custom(
                        span,
                        "E-QUERY-001: IIN operator requires quoted string literals in the \
                         membership list; got a non-string value (integer, float, or boolean). \
                         Example: status IIN ('new', 'open'). If you need date-like strings, \
                         wrap them in quotes: created_at IIN ('2026-06-01', '2026-06-02').",
                    ));
                }
                Ok(values)
            });
        let iin_list = field_path
            .clone()
            .padded()
            .then_ignore(kw("IIN").padded())
            .then(iin_values)
            .map(|(fp, values)| Predicate::In {
                field: fp,
                values,
                negated: false,
                case_insensitive: true,
            });

        // --- field IN (val, …) ---
        let in_list = field_path
            .clone()
            .padded()
            .then_ignore(kw("IN").padded())
            .then(
                literal
                    .clone()
                    .padded()
                    .separated_by(just(',').padded())
                    .at_least(1)
                    .collect::<Vec<_>>()
                    .delimited_by(just('(').padded(), just(')').padded()),
            )
            .map(|(fp, values)| Predicate::In {
                field: fp,
                values,
                negated: false,
                case_insensitive: false,
            });

        // --- field BETWEEN low AND high ---
        let between = field_path
            .clone()
            .padded()
            .then_ignore(kw("BETWEEN").padded())
            .then(literal.clone().padded())
            .then_ignore(kw("AND").padded())
            .then(literal.clone().padded())
            .map(|((fp, low), high)| Predicate::Between {
                field: fp,
                low,
                high,
                negated: false,
            });

        // --- field IS NULL / field IS NOT NULL ---
        let is_null = field_path
            .clone()
            .padded()
            .then_ignore(kw("IS").padded())
            .then(kw("NOT").padded().to(true).or(empty().to(false)))
            .then_ignore(kw("NULL").padded())
            .map(|(fp, not)| Predicate::IsNull {
                field: fp,
                negated: not,
            });

        // --- string op: CONTAINS / STARTSWITH / ENDSWITH / ICONTAINS etc. ---
        let string_op = choice((
            kw("ICONTAINS").padded().to((StringOp::Contains, true)),
            kw("ISTARTSWITH").padded().to((StringOp::StartsWith, true)),
            kw("IENDSWITH").padded().to((StringOp::EndsWith, true)),
            kw("CONTAINS").padded().to((StringOp::Contains, false)),
            kw("STARTSWITH").padded().to((StringOp::StartsWith, false)),
            kw("ENDSWITH").padded().to((StringOp::EndsWith, false)),
        ));

        let string_op_match = field_path
            .clone()
            .padded()
            .then(string_op)
            .then(string_val.clone().padded())
            .map(|((fp, (op, ci)), pat)| Predicate::StringOp {
                field: fp,
                op,
                pattern: pat,
                case_insensitive: ci,
            });

        // --- LIKE operator (kept for BC compat) ---
        let like_match = field_path
            .clone()
            .padded()
            .then_ignore(choice((
                text::keyword("LIKE").padded(),
                text::keyword("like").padded(),
            )))
            .then(literal.clone().padded())
            .map(|(fp, lit)| {
                // LIKE is kept as Predicate::Compare for backward compat.
                // Wildcard promotion applies only to = and != (see field_comparison below).
                // Virtual-field promotion: _sensor/_client/etc. become Expr::VirtualField.
                Predicate::Compare {
                    lhs: Box::new(field_path_to_expr(fp)),
                    op: CompareOp::Like,
                    rhs: Box::new(crate::ast::Expr::Literal(lit)),
                    case_insensitive: false,
                }
            });

        // --- field IEQ 'value' — case-insensitive equality (S-PRISMQL-CASE-INSENSITIVE-001) ---
        // IEQ must appear before field_comparison in the atom choice.
        // RHS must be a string literal (AC-010, BC-2.11.024 error case "E-QUERY-001: IEQ/INE
        // with non-string RHS").
        //
        // The try_map is placed at the LITERAL level (not the full-sequence level). This ensures
        // the emitted error span begins at the position of the literal token itself (e.g., "42"
        // at position 13), not the start of the whole pattern (position 0). Chumsky 0.12 choice()
        // error merging uses span.start to select the "furthest" error; a span starting at 13
        // outcompetes the "expected keyword 'MATCHES'" error that starts at position 9.
        let ieq_rhs_string = literal.clone().padded().try_map(|lit, span| match lit {
            Literal::String(s) => Ok(s),
            // F-LOW-001: `classify_string_literal` (ADR-052 §D4 lenient parse) may
            // reclassify a quoted date-like value (e.g. '2026-06-01') to
            // `RawTemporalLiteral` or `Timestamp`.  The user explicitly quoted the
            // value for a string/label comparison — unwrap it as a plain string so
            // the emitter can call `lower()` on it.  No temporal semantics apply
            // on the `ci` (case-insensitive) emit path; the value is emitted as
            // `lower('2026-06-01')`, not as `arrow_cast(...)`.
            Literal::RawTemporalLiteral(s) => Ok(s),
            Literal::Timestamp(ts) => Ok(ts.iso8601),
            _ => Err(Rich::custom(
                span,
                "E-QUERY-001: IEQ operator requires a quoted string literal on the \
                 right-hand side; got a non-string value. \
                 Use a quoted string: e.g. `severity IEQ 'high'`.",
            )),
        });
        let ieq_compare = field_path
            .clone()
            .padded()
            .then_ignore(kw("IEQ").padded())
            .then(ieq_rhs_string)
            .map(|(fp, val)| Predicate::Compare {
                lhs: Box::new(field_path_to_expr(fp)),
                op: CompareOp::Eq,
                rhs: Box::new(crate::ast::Expr::Literal(Literal::String(val))),
                case_insensitive: true,
            });

        // --- field INE 'value' — case-insensitive inequality (S-PRISMQL-CASE-INSENSITIVE-001) ---
        // Same try_map-at-literal-level pattern as ieq_compare.
        let ine_rhs_string = literal.clone().padded().try_map(|lit, span| match lit {
            Literal::String(s) => Ok(s),
            // F-LOW-001: same RawTemporalLiteral/Timestamp acceptance as ieq_rhs_string.
            // Quoted date-like values must be treated as plain strings for INE comparison.
            Literal::RawTemporalLiteral(s) => Ok(s),
            Literal::Timestamp(ts) => Ok(ts.iso8601),
            _ => Err(Rich::custom(
                span,
                "E-QUERY-001: INE operator requires a quoted string literal on the \
                 right-hand side; got a non-string value. \
                 Use a quoted string: e.g. `severity INE 'high'`.",
            )),
        });
        let ine_compare = field_path
            .clone()
            .padded()
            .then_ignore(kw("INE").padded())
            .then(ine_rhs_string)
            .map(|(fp, val)| Predicate::Compare {
                lhs: Box::new(field_path_to_expr(fp)),
                op: CompareOp::Ne,
                rhs: Box::new(crate::ast::Expr::Literal(Literal::String(val))),
                case_insensitive: true,
            });

        // RHS value expression: temporal (NOW/INTERVAL) or literal.
        //
        // Temporal parser is tried first (BC-2.11.021 / ADR-044 D7 invariant):
        // `NOW()`, `NOW() - INTERVAL '...'`, `INTERVAL '...'` are recognized here
        // before falling back to a plain literal. This makes temporal expressions
        // available in all three modes (filter / SQL WHERE / pipe where) via the
        // shared build_predicate_parser.
        let temporal_rhs = build_temporal_rhs_parser();
        let rhs_expr = temporal_rhs
            .clone()
            .or(literal.clone().padded().map(crate::ast::Expr::Literal));

        // --- fn-call comparison: scalar_fn_name(args) compare_op rhs_expr ---
        //
        // (DEFECT-PQL-FNCALL-LHS-001, ADR-048 v1.2 §D.7.2) Grammar extension for scalar
        // fn-call LHS comparisons in pipe `| where`, filter mode, SQL WHERE, SqlPipe
        // `| where`, and SqlPipe-head WHERE. Positioned BEFORE `field_comparison` in the
        // atom choice so that `lower(col) = 'val'` is dispatched here rather than causing
        // a backtrack at `(`.
        //
        // PARSER-LEVEL AGGREGATE BLOCKLIST REMOVED (ADR-048 v1.2 OD-4): the prior
        // `AGGREGATE_FUNC_NAMES` `try_map` guard (count/sum/avg/min/max/distinct_count/
        // percentile) has been removed. Chumsky backtrack was swallowing the rejection
        // error, producing "found '('" instead of the canonical ADR-048 D.3 message.
        // Aggregate names now parse successfully as `FuncCall::Scalar(Unknown(name))` and
        // are intercepted by the plan-time `DATAFUSION_BUILTIN_AGGREGATE_NAMES` gate in
        // `check_enrich_udf_availability`, which fires the canonical E-QUERY-001 message
        // for all five predicate positions (ADR-048 D.7.1).
        //
        // Downstream handling requires NO logic changes — pre-existing code handles FuncCall LHS:
        //   - check_enrich_udf_availability: DATAFUSION_BUILTIN_AGGREGATE_NAMES gate
        //     fires E-QUERY-001 for aggregate names in WHERE predicates (ADR-048 D.3)
        //   - check_temporal_literals arm (4): non-Field LHS + RawTemporalLiteral → E-QUERY-042
        //   - collect_predicate_columns FuncCall arm: recurses args for E-QUERY-038 column gate
        //   - collect_predicate_columns_with_bareness FuncCall arm: same with bareness tracking
        let fn_call_arg = literal
            .clone()
            .padded()
            .map(crate::ast::Expr::Literal)
            .or(field_path.clone().padded().map(field_path_to_expr));
        let fn_call_comparison = any::<&str, extra::Err<Rich<char>>>()
            .filter(|c: &char| c.is_ascii_alphanumeric() || *c == '_')
            .repeated()
            .at_least(1)
            .to_slice()
            .padded()
            .map(|name: &str| name.to_string())
            .then(
                fn_call_arg
                    .separated_by(just(',').padded())
                    .collect::<Vec<_>>()
                    .delimited_by(just('(').padded(), just(')').padded()),
            )
            .then(compare_op.clone())
            .then(rhs_expr.clone().padded())
            .map(|(((func_name, args), op), rhs)| {
                use crate::ast::{FuncCall, ScalarFunc};
                Predicate::Compare {
                    lhs: Box::new(crate::ast::Expr::FuncCall(FuncCall::Scalar {
                        func: ScalarFunc::Unknown(func_name),
                        args,
                    })),
                    op,
                    rhs: Box::new(rhs),
                    case_insensitive: false,
                }
            });

        // --- Basic comparison: field op (temporal_expr | literal) ---
        // Auto-promotes = or != with wildcard string patterns to Predicate::Wildcard.
        let field_comparison = field_path
            .clone()
            .padded()
            .then(compare_op.clone())
            .then(rhs_expr.padded())
            .try_map(|((fp, op), rhs), span| {
                // Wildcard promotion only applies when RHS is a string literal.
                if let crate::ast::Expr::Literal(Literal::String(ref s)) = rhs {
                    if s.contains('*') || s.contains('?') {
                        match op {
                            CompareOp::Eq => {
                                return Ok(Predicate::Wildcard {
                                    field: fp,
                                    pattern: s.clone(),
                                    negated: false,
                                });
                            }
                            CompareOp::Ne => {
                                return Ok(Predicate::Wildcard {
                                    field: fp,
                                    pattern: s.clone(),
                                    negated: true,
                                });
                            }
                            _ => {
                                return Err(Rich::custom(
                                    span,
                                    "E-QUERY-001: ordering operators (>, <, >=, <=) are meaningless on wildcard patterns",
                                ));
                            }
                        }
                    }
                }
                // Virtual-field promotion: _sensor/_client/etc. become Expr::VirtualField.
                Ok(Predicate::Compare {
                    lhs: Box::new(field_path_to_expr(fp)),
                    op,
                    rhs: Box::new(rhs),
                    case_insensitive: false,
                })
            });

        // --- cidr operator (legacy bare `cidr` keyword syntax for backward compat) ---
        let cidr_bare = field_path
            .clone()
            .padded()
            .then_ignore(choice((
                text::keyword("cidr").padded(),
                text::keyword("CIDR").padded(),
            )))
            .then(string_val.clone().padded())
            .try_map(|(fp, cidr_str), span| {
                CidrLiteral::new(&cidr_str)
                    .map(|cl| Predicate::Cidr {
                        field: fp,
                        cidr: cl,
                        negated: false,
                    })
                    .map_err(|e| Rich::custom(span, e))
            });

        // Atom: `(predicate)` | one of the above
        // Ordering discipline: IIN before IN, IEQ/INE before field_comparison,
        // fn_call_comparison BEFORE field_comparison (DEFECT-PQL-FNCALL-LHS-001).
        let atom = choice((
            predicate
                .clone()
                .padded()
                .delimited_by(just('(').padded(), just(')').padded()),
            has_check,
            missing_check,
            regex_match,
            cidr_match,
            not_in_list,
            iin_list,
            in_list,
            between,
            is_null,
            string_op_match,
            cidr_bare,
            like_match,
            ieq_compare,
            ine_compare,
            fn_call_comparison,
            field_comparison,
        ));

        // NOT / ! predicate
        let not_pred = recursive(
            |not: Recursive<dyn Parser<'_, &str, Predicate, extra::Err<Rich<'_, char>>>>| {
                choice((
                    kw("NOT")
                        .padded()
                        .ignore_then(not.clone())
                        .map(|p| Predicate::Not(Box::new(p))),
                    just('!')
                        .padded()
                        .ignore_then(not.clone())
                        .map(|p| Predicate::Not(Box::new(p))),
                    atom,
                ))
            },
        );

        // AND / && combinator (left-associative, foldl into Vec).
        let and_pred = not_pred.clone().foldl(
            choice((kw("AND").padded().to(()), just("&&").padded().to(())))
                .ignore_then(not_pred)
                .repeated(),
            |lhs, rhs| {
                // Flatten nested ANDs into a single Logical::And.
                match lhs {
                    Predicate::Logical {
                        op: LogicalOp::And,
                        mut predicates,
                    } => {
                        predicates.push(rhs);
                        Predicate::Logical {
                            op: LogicalOp::And,
                            predicates,
                        }
                    }
                    other => Predicate::Logical {
                        op: LogicalOp::And,
                        predicates: vec![other, rhs],
                    },
                }
            },
        );

        // OR / || combinator (left-associative, foldl into Vec).
        and_pred.clone().foldl(
            choice((kw("OR").padded().to(()), just("||").padded().to(())))
                .ignore_then(and_pred)
                .repeated(),
            |lhs, rhs| match lhs {
                Predicate::Logical {
                    op: LogicalOp::Or,
                    mut predicates,
                } => {
                    predicates.push(rhs);
                    Predicate::Logical {
                        op: LogicalOp::Or,
                        predicates,
                    }
                }
                other => Predicate::Logical {
                    op: LogicalOp::Or,
                    predicates: vec![other, rhs],
                },
            },
        )
    })
}

/// Build a parser for quoted string values (single or double quoted).
pub(crate) fn build_string_parser<'a>(
) -> impl Parser<'a, &'a str, String, extra::Err<Rich<'a, char>>> + Clone {
    let single_quoted = none_of('\'')
        .repeated()
        .to_slice()
        .map(|s: &str| s.to_string())
        .delimited_by(just('\''), just('\''));

    let double_quoted = none_of('"')
        .repeated()
        .to_slice()
        .map(|s: &str| s.to_string())
        .delimited_by(just('"'), just('"'));

    single_quoted.or(double_quoted)
}

/// Heuristic: does `s` match any of the ADR-052 §D4 v1.4 seven canonical date-like forms?
///
/// Returns `true` when `s` looks like a date or offset-less datetime but is NOT a valid
/// RFC-3339 string (i.e., `chrono::DateTime::parse_from_rfc3339` would reject it).
/// Used by the lenient parser fallback to produce `Literal::RawTemporalLiteral` instead
/// of a hard parse error, so that `check_temporal_literals` can perform schema-aware
/// seven-arm dispatch (ADR-052 §D4 v1.10): Datetime col → E-QUERY-041; String col → COERCE;
/// Integer/Float/Bool col → E-QUERY-002; non-Field LHS → E-QUERY-042 NonColumnLhsComparison;
/// SELECT projection → COERCE; GROUP BY → E-QUERY-042 GroupBy; ORDER BY → E-QUERY-042 OrderBy.
///
/// The seven canonical forms (exhaustive — must match exactly, per BC-2.11.021):
///   1. `%Y-%m-%d`              — date-only
///   2. `%Y-%m-%dT%H:%M:%S`    — T-sep full seconds
///   3. `%Y-%m-%dT%H:%M:%S%.f` — T-sep fractional seconds
///   4. `%Y-%m-%dT%H:%M`       — T-sep no seconds
///   5. `%Y-%m-%d %H:%M:%S`    — space-sep full seconds
///   6. `%Y-%m-%d %H:%M:%S%.f` — space-sep fractional seconds
///   7. `%Y-%m-%d %H:%M`       — space-sep no seconds
///
/// Over-matched forms (unpadded month/day `'2026-6-24'`, big/signed years) are ACCEPTED
/// BENIGN per ADR-052 §D4 v1.4 — chrono `%m`/`%d`/`%Y` accept them; no regex guard applied.
/// (BC-2.11.021 EC-11-021-010..014)
///
/// Operates on already-parsed UTF-8 `&str` — no raw byte-offset operations;
/// Unicode safety guaranteed by construction (VP-021 invariant).
///
/// (ADR-052 §D4 Step 2; BC-2.11.021; S-PRISMQL-NATIVE-TEMPORAL-TYPING-001 Task 13)
/// Seven-form acceptance set per ADR-052 §D4 v1.4.
///
/// Over-matched forms (unpadded digits, big years) are ACCEPTED BENIGN —
/// chrono `%m`/`%d`/`%Y` accept them; no regex guard applied.
pub(crate) fn is_date_like(s: &str) -> bool {
    use chrono::{NaiveDate, NaiveDateTime};
    // Form 1: date-only
    NaiveDate::parse_from_str(s, "%Y-%m-%d").is_ok()
    // Form 2: T-sep full seconds (no offset)
    || NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S").is_ok()
    // Form 3: T-sep fractional seconds (no offset)
    || NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S%.f").is_ok()
    // Form 4: T-sep no seconds (no offset)
    || NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M").is_ok()
    // Form 5: space-sep full seconds (no offset)
    || NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S").is_ok()
    // Form 6: space-sep fractional seconds (no offset)
    || NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S%.f").is_ok()
    // Form 7: space-sep no seconds (no offset)
    || NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M").is_ok()
}

/// Classify a raw quoted string into the appropriate `Literal` variant.
///
/// Three-case lenient dispatch (ADR-052 §D4 Step 2; Option-A):
///
///   - RFC-3339 parse succeeds → `Literal::Timestamp` (full ISO-8601 with UTC offset)
///   - `is_date_like(s)` is true → `Literal::RawTemporalLiteral(s)` (date/offset-less form;
///     validated at plan time by `check_temporal_literals`)
///   - Otherwise → `Literal::String(s)` (plain string, no temporal semantics)
///
/// Infallible: parse ALWAYS succeeds. Validation of the temporal forms moves to plan time.
/// (ADR-052 §D4 Step 2; BC-2.11.021; S-PRISMQL-NATIVE-TEMPORAL-TYPING-001 Task 13)
fn classify_string_literal(s: &str) -> Literal {
    // Fast path: try RFC-3339 first (full timestamp with UTC offset).
    if let Ok(ts) = TimestampLiteral::new(s) {
        return Literal::Timestamp(ts);
    }
    // Date-like but not full RFC-3339 → intermediate node for plan-time validation.
    if is_date_like(s) {
        return Literal::RawTemporalLiteral(s.to_string());
    }
    // Non-date plain string.
    Literal::String(s.to_string())
}

/// Build the literal value parser.
pub(crate) fn build_literal_parser<'a>(
) -> impl Parser<'a, &'a str, Literal, extra::Err<Rich<'a, char>>> + Clone {
    // Single-quoted string literal (infallible: RFC-3339→Timestamp; date-like→RawTemporalLiteral; else→String).
    let single_quoted = none_of('\'')
        .repeated()
        .to_slice()
        .map(|s: &str| classify_string_literal(s))
        .delimited_by(just('\''), just('\''));

    // Double-quoted string literal (same three-case lenient dispatch as single-quoted).
    let double_quoted = none_of('"')
        .repeated()
        .to_slice()
        .map(|s: &str| classify_string_literal(s))
        .delimited_by(just('"'), just('"'));

    // NULL literal.
    let null_lit = any::<&str, extra::Err<Rich<char>>>()
        .filter(|c: &char| c.is_ascii_alphabetic())
        .repeated()
        .at_least(1)
        .to_slice()
        .try_map(|s: &str, span| {
            if s.eq_ignore_ascii_case("NULL") {
                Ok(Literal::Null)
            } else {
                Err(Rich::custom(span, "expected NULL"))
            }
        });

    // Boolean literals.
    let bool_lit = any::<&str, extra::Err<Rich<char>>>()
        .filter(|c: &char| c.is_ascii_alphabetic())
        .repeated()
        .at_least(1)
        .to_slice()
        .try_map(|s: &str, span| {
            if s.eq_ignore_ascii_case("TRUE") {
                Ok(Literal::Bool(true))
            } else if s.eq_ignore_ascii_case("FALSE") {
                Ok(Literal::Bool(false))
            } else {
                Err(Rich::custom(span, "expected TRUE or FALSE"))
            }
        });

    // Duration literal: digits followed by unit char (s, m, h, d).
    // Must be parsed BEFORE float/int to avoid consuming `30` from `30s`.
    //
    // SEC-S-001: The `unit_char` is produced by a combinator that filters to
    // exactly the four valid chars ('s', 'm', 'h', 'd'). The match below uses
    // a `try_map` returning `Err` for the wildcard arm instead of `unreachable!()`
    // — this ensures the parser never panics on attacker-influenced input even if
    // the combinator contract changes in the future.
    let duration_lit = text::int(10)
        .to_slice()
        .then(
            any::<&str, extra::Err<Rich<char>>>()
                .filter(|c: &char| matches!(c, 's' | 'm' | 'h' | 'd')),
        )
        .try_map(|(digits, unit_char): (&str, char), span| {
            let value: u64 = digits
                .parse()
                .map_err(|e| Rich::custom(span, format!("invalid duration value: {e}")))?;
            let unit = match unit_char {
                's' => DurationUnit::Seconds,
                'm' => DurationUnit::Minutes,
                'h' => DurationUnit::Hours,
                'd' => DurationUnit::Days,
                other => {
                    return Err(Rich::custom(
                        span,
                        format!("E-QUERY-001: unexpected duration unit char '{other}' (expected s/m/h/d)"),
                    ));
                }
            };
            DurationLiteral::new(value, unit)
                .map(Literal::Duration)
                .map_err(|e| Rich::custom(span, e))
        });

    // Float literal: optional minus, digits, dot, digits.
    let float_lit = just('-')
        .or_not()
        .then(text::int(10))
        .then(just('.').then(text::digits(10)))
        .to_slice()
        .try_map(|s: &str, span| {
            s.parse::<f64>()
                .map(|f| Literal::Float(OrderedFloat(f)))
                .map_err(|e| Rich::custom(span, format!("invalid float literal: {e}")))
        });

    // Integer literal: optional minus, digits.
    let int_lit = just('-')
        .or_not()
        .then(text::int(10))
        .to_slice()
        .try_map(|s: &str, span| {
            s.parse::<i64>()
                .map(Literal::Integer)
                .map_err(|e| Rich::custom(span, format!("invalid integer literal: {e}")))
        });

    choice((
        null_lit,
        bool_lit,
        single_quoted,
        double_quoted,
        duration_lit,
        float_lit,
        int_lit,
    ))
}

/// Parse a quoted INTERVAL duration string: `'<int><unit>'` where unit ∈ {s,m,h,d}.
///
/// Called from `build_temporal_rhs_parser`; separated because `chrono` 0.4.44 has
/// no duration-string parser and the custom `<int><unit>` format is ADR-044-specific.
///
/// Returns `Err` on any of:
/// - Non-integer digit prefix
/// - Unknown unit character (not s/m/h/d)
/// - Value overflow when converting to `chrono::Duration`
///
/// # Implements BC-2.11.021 / ADR-044
fn parse_interval_duration_str(s: &str) -> Result<chrono::Duration, String> {
    // Must be non-empty.
    if s.is_empty() {
        return Err("E-QUERY-001: INTERVAL duration string must not be empty".to_string());
    }

    // Extract the last *character* (not the last byte) as the unit.
    //
    // F-P3-CRIT-NEW-001 fix: the previous code did `s.split_at(s.len() - 1)`,
    // which is a byte-index split.  When the last character is a multi-byte
    // UTF-8 sequence (e.g. `é` = 2 bytes, `€` = 3 bytes), `s.len() - 1` lands
    // inside the sequence — a non-char-boundary index — causing an unconditional
    // panic.  The fix uses `s.chars().next_back()` (char-aware) and computes the
    // digits slice via `s.len() - last_char.len_utf8()`, which is always a valid
    // char boundary regardless of the unit char's encoding.
    //
    // Correctness: valid unit chars (s/m/h/d) are ASCII (1 byte each), so for
    // valid INTERVAL strings the behaviour is identical to the previous code.
    // For invalid input containing non-ASCII trailing chars the new code returns
    // a structured `E-QUERY-001` error instead of panicking.
    let last_char = match s.chars().next_back() {
        Some(c) => c,
        // Already guarded above, but make the type system happy.
        None => return Err("E-QUERY-001: INTERVAL duration string must not be empty".to_string()),
    };

    let unit_split = s.len() - last_char.len_utf8();
    let digits_part = &s[..unit_split];

    if digits_part.is_empty() {
        return Err(format!(
            "E-QUERY-001: INTERVAL duration '{s}' has no numeric part"
        ));
    }

    let value: u64 = digits_part.parse().map_err(|_| {
        format!(
            "E-QUERY-001: INTERVAL duration '{s}' has invalid numeric part '{digits_part}' (must be a non-negative integer)"
        )
    })?;

    // F-P3-FRESH-CRIT-001 fix: guard against TWO distinct overflow classes BEFORE
    // calling any chrono constructor:
    //
    // Class A — i64 cast-wrap: `value as i64` silently wraps for value > i64::MAX,
    //   producing a large NEGATIVE i64 that violates Duration's non-negative
    //   invariant and causes a panic inside Duration::{seconds,minutes,hours,days}.
    //   Fix: use `i64::try_from(value)` which returns Err for value > i64::MAX.
    //
    // Class B — TimeDelta bounds: chrono stores Duration internally as milliseconds
    //   in an i64, so even values ≤ i64::MAX can overflow when multiplied by the
    //   unit's ms factor (e.g. 106_751_991_168 days * 86_400_000 ms/day > i64::MAX).
    //   `Duration::{seconds,minutes,hours,days}` calls the checked `try_*` variant
    //   internally and panics on None. Fix: call `chrono::Duration::try_{unit}`
    //   directly and map None → structured Err.
    let value_i64 = i64::try_from(value).map_err(|_| {
        format!(
            "E-QUERY-001: INTERVAL duration '{s}' is too large \
             (value {value} exceeds representable range)"
        )
    })?;

    let duration = match last_char {
        's' => chrono::Duration::try_seconds(value_i64),
        'm' => chrono::Duration::try_minutes(value_i64),
        'h' => chrono::Duration::try_hours(value_i64),
        'd' => chrono::Duration::try_days(value_i64),
        other => {
            return Err(format!(
                "E-QUERY-001: INTERVAL duration '{s}' has unknown unit '{other}' \
                 (expected s=seconds, m=minutes, h=hours, d=days)"
            ));
        }
    };
    duration.ok_or_else(|| {
        format!(
            "E-QUERY-001: INTERVAL duration '{s}' is too large \
             (exceeds representable range)"
        )
    })
}

/// Build a parser for temporal RHS expressions used in predicate comparisons.
///
/// Recognises:
/// - `NOW()` → `Expr::Now`
/// - `NOW(args)` → `Err("E-QUERY-001: NOW() takes no arguments")`
/// - `NOW() - INTERVAL '...'` → `Expr::TimestampArithmetic { base: Now, op: Sub, offset }`
/// - `NOW() + INTERVAL '...'` → `Err("E-QUERY-001: subtraction-only constraint")`
/// - `INTERVAL '...'` (standalone) → `Expr::Interval(duration)`
///
/// # D7 invariant
/// This parser is used by `build_predicate_parser` which is shared across all
/// three query modes (filter / SQL WHERE / pipe where). Adding it here makes
/// temporal expressions available in all modes without duplication.
///
/// # Implements BC-2.11.021 / ADR-044
pub(crate) fn build_temporal_rhs_parser<'a>(
) -> impl Parser<'a, &'a str, crate::ast::Expr, extra::Err<Rich<'a, char>>> + Clone {
    use crate::ast::{BinaryOp, Expr};

    // Quoted string inside INTERVAL — any chars until closing quote.
    let interval_content = none_of('\'')
        .repeated()
        .to_slice()
        .delimited_by(just('\''), just('\''))
        .try_map(|s: &str, span| parse_interval_duration_str(s).map_err(|e| Rich::custom(span, e)));

    // `INTERVAL '<int><unit>'` — standalone interval literal (Expr::Interval).
    let interval_expr = any::<&str, extra::Err<Rich<char>>>()
        .filter(|c: &char| c.is_ascii_alphabetic())
        .repeated()
        .at_least(1)
        .to_slice()
        .try_map(|s: &str, span| {
            if s.eq_ignore_ascii_case("INTERVAL") {
                Ok(())
            } else {
                Err(Rich::custom(span, "expected INTERVAL keyword"))
            }
        })
        .padded()
        .ignore_then(interval_content.padded())
        .map(Expr::Interval);

    // `NOW` keyword (case-insensitive).
    let now_kw = any::<&str, extra::Err<Rich<char>>>()
        .filter(|c: &char| c.is_ascii_alphabetic())
        .repeated()
        .at_least(1)
        .to_slice()
        .try_map(|s: &str, span| {
            if s.eq_ignore_ascii_case("NOW") {
                Ok(())
            } else {
                Err(Rich::custom(span, "expected NOW keyword"))
            }
        });

    // `NOW(...)` — parse parens and error if any arguments present.
    let now_call = now_kw
        .padded()
        .then(
            just('(')
                .padded()
                .ignore_then(
                    // Capture everything inside the parens to detect arguments.
                    none_of(')').repeated().to_slice().padded(),
                )
                .then_ignore(just(')').padded()),
        )
        .try_map(|((), args): ((), &str), span| {
            let args_trimmed = args.trim();
            if !args_trimmed.is_empty() {
                return Err(Rich::custom(
                    span,
                    format!("E-QUERY-001: NOW() takes no arguments; got '{args_trimmed}'"),
                ));
            }
            Ok(Expr::Now)
        });

    // Design: NOW() with optional arithmetic.
    //
    // Using `choice((now_arithmetic, now_call))` would cause backtracking:
    // if now_arithmetic's try_map rejects '+' (subtraction-only), Chumsky
    // backtracks to now_call which succeeds — the error is silently swallowed.
    //
    // Instead: parse NOW() then OPTIONALLY accept arithmetic. If arithmetic is
    // present, validate the operator in try_map. If absent, return Expr::Now.
    // This is non-backtracking: once NOW( is committed, any following arithmetic
    // operator is consumed and validated.
    let now_then_optional_arithmetic = now_call
        .then(
            choice((
                just('-').padded().to(BinaryOp::Sub),
                just('+').padded().to(BinaryOp::Add),
            ))
            .padded()
            .then(
                any::<&str, extra::Err<Rich<char>>>()
                    .filter(|c: &char| c.is_ascii_alphabetic())
                    .repeated()
                    .at_least(1)
                    .to_slice()
                    .try_map(|s: &str, span| {
                        if s.eq_ignore_ascii_case("INTERVAL") {
                            Ok(())
                        } else {
                            Err(Rich::custom(span, "expected INTERVAL after NOW() operator"))
                        }
                    })
                    .padded()
                    .ignore_then(interval_content.padded()),
            )
            .or_not(),
        )
        .try_map(
            |(base_expr, arith_opt): (Expr, Option<(BinaryOp, chrono::Duration)>), span| {
                match arith_opt {
                    None => Ok(base_expr), // bare NOW() → Expr::Now
                    Some((BinaryOp::Add, _)) => Err(Rich::custom(
                        span,
                        "E-QUERY-001: timestamp arithmetic subtraction-only constraint in v1; \
                     NOW() + INTERVAL is not supported. Use NOW() - INTERVAL instead.",
                    )),
                    Some((BinaryOp::Sub, offset)) => Ok(Expr::TimestampArithmetic {
                        base: Box::new(base_expr),
                        op: BinaryOp::Sub,
                        offset,
                    }),
                }
            },
        );

    // Precedence: NOW() (with optional arithmetic) first, then standalone INTERVAL.
    choice((now_then_optional_arithmetic, interval_expr))
}

/// Build the shared Expr parser for value expressions (SELECT projections,
/// ORDER BY, GROUP BY, JOIN ON conditions).
///
/// This is distinct from `build_predicate_parser` — it produces `Expr` (value)
/// not `Predicate` (boolean). Reserved for S-3.02 (DataFusion TableProvider)
/// which will need standalone Expr parsing for projection pushdown.
#[allow(dead_code)] // Reserved for S-3.02 — not yet called by any story-in-scope parser.
#[allow(clippy::clone_on_copy)]
pub(crate) fn build_expr_parser<'a>(
) -> impl Parser<'a, &'a str, crate::ast::Expr, extra::Err<Rich<'a, char>>> + Clone {
    use crate::ast::{CompareOp as CO, Expr, LogicalOp as LO};

    recursive(|expr| {
        let literal = build_literal_parser();

        let ident_char = any::<&str, extra::Err<Rich<char>>>()
            .filter(|c: &char| c.is_ascii_alphanumeric() || *c == '_');

        let field_segment = ident_char.repeated().at_least(1).to_slice();

        let field_path = field_segment
            .separated_by(just('.'))
            .at_least(1)
            .collect::<Vec<&str>>()
            .map_with(|segs: Vec<&str>, e| {
                // Capture the actual byte-offset span from Chumsky (CR F-CR-007).
                let s = e.span();
                FieldPath {
                    segments: segs.into_iter().map(|seg| seg.to_string()).collect(),
                    span: Span {
                        start: s.start,
                        end: s.end,
                    },
                }
            });

        let compare_op = choice((
            just(">=").to(CO::Ge),
            just("<=").to(CO::Le),
            just("!=").to(CO::Ne),
            just("==").to(CO::Eq),
            just('>').to(CO::Gt),
            just('<').to(CO::Lt),
            just('=').to(CO::Eq),
            text::keyword("LIKE").to(CO::Like),
            text::keyword("like").to(CO::Like),
            text::keyword("cidr").to(CO::Cidr),
            text::keyword("CIDR").to(CO::Cidr),
        ))
        .padded();

        let in_list = field_path
            .clone()
            .padded()
            .then_ignore(choice((text::keyword("IN"), text::keyword("in"))).padded())
            .then(
                literal
                    .clone()
                    .padded()
                    .separated_by(just(',').padded())
                    .at_least(1)
                    .collect::<Vec<_>>()
                    .delimited_by(just('(').padded(), just(')').padded()),
            )
            .map(|(fp, values)| Expr::In { field: fp, values });

        let comparison = field_path
            .clone()
            .padded()
            .then(compare_op)
            .then(literal.clone().padded().map(Expr::Literal))
            .map(|((fp, op), rhs)| Expr::Compare {
                // Virtual-field promotion: _sensor/_client/etc. become Expr::VirtualField.
                lhs: Box::new(crate::ast::field_path_to_expr(fp)),
                op,
                rhs: Box::new(rhs),
            });

        let atom = choice((
            expr.clone()
                .padded()
                .delimited_by(just('(').padded(), just(')').padded()),
            in_list,
            comparison,
        ));

        let not_expr = recursive(
            |not: Recursive<dyn Parser<'_, &str, Expr, extra::Err<Rich<'_, char>>>>| {
                choice((
                    text::keyword("NOT")
                        .padded()
                        .ignore_then(not.clone())
                        .map(|e| Expr::Not(Box::new(e))),
                    text::keyword("not")
                        .padded()
                        .ignore_then(not)
                        .map(|e| Expr::Not(Box::new(e))),
                    atom,
                ))
            },
        );

        let and_expr = not_expr.clone().foldl(
            choice((text::keyword("AND").padded(), text::keyword("and").padded()))
                .ignore_then(not_expr)
                .repeated(),
            |lhs, rhs| Expr::Logical {
                lhs: Box::new(lhs),
                op: LO::And,
                rhs: Box::new(rhs),
            },
        );

        and_expr.clone().foldl(
            choice((text::keyword("OR").padded(), text::keyword("or").padded()))
                .ignore_then(and_expr)
                .repeated(),
            |lhs, rhs| Expr::Logical {
                lhs: Box::new(lhs),
                op: LO::Or,
                rhs: Box::new(rhs),
            },
        )
    })
}

/// Build the pipe stage parser (forwarded from pipe_parser module).
/// Used in mode detection contexts (reserved for future multi-mode dispatch refactors).
#[allow(dead_code)] // Reserved for future multi-mode dispatch — not called in current parse path.
pub(crate) fn build_pipe_mode_parser<'a>(
) -> impl Parser<'a, &'a str, PipeQuery, extra::Err<Rich<'a, char>>> {
    build_pipe_parser()
}

// ─────────────────────────────────────────────────────────────────────────────
// S-3.06 filter-mode write rejection (BC-2.11.004)
// ─────────────────────────────────────────────────────────────────────────────

/// Validate that a filter-mode query does not contain write verb tokens.
///
/// Filter mode is permanently read-only. This check is a grammar-level hard
/// rejection (BC-2.11.004, EC-11-064), not a semantic check — it fires
/// regardless of whether the verb would resolve to a valid write endpoint.
///
/// Implementation: scans for unquoted `|` followed by a registered write verb.
/// A write verb appearing as a field name in a predicate (e.g. `contain = 1`)
/// is NOT rejected — only `| verb` sequences are rejected.
///
/// When the registry is empty, always returns `Ok(())` per
/// BC-2.11.004 §INV-FILTER-EMPTY-REGISTRY.
///
/// Returns `Err(Vec<ParseError>)` if any write verb token is detected in the
/// filter input; `Ok(())` if the input is clean.
///
/// # Security perimeter (BC-2.11.006 INV-SEC-PERIMETER-001)
/// `pub(crate)` — never `pub`.
///
/// # Implements BC-2.11.004 — Write Parser Extension
pub(crate) fn reject_write_verbs_in_filter(
    input: &str,
    registry: &WriteVerbRegistry,
) -> Result<(), Vec<ParseError>> {
    // BC-2.11.004 §INV-FILTER-EMPTY-REGISTRY: empty registry → always Ok(()).
    if registry.is_empty() {
        return Ok(());
    }

    // Scan for `| verb` sequences outside string literals.
    // A write verb is only rejected when preceded by `|` (pipe operator).
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
                // Skip whitespace after `|`
                let mut j = i + 1;
                while j < len && (bytes[j] == b' ' || bytes[j] == b'\t' || bytes[j] == b'\n') {
                    j += 1;
                }
                // Extract the next identifier token.
                let mut k = j;
                while k < len && (bytes[k].is_ascii_alphanumeric() || bytes[k] == b'_') {
                    k += 1;
                }
                if k > j {
                    let token = &input[j..k];
                    if registry.is_write_verb(token) {
                        return Err(vec![ParseError::new(
                            j,
                            format!(
                                "E-QUERY-010: Write verbs are not permitted in filter mode; \
                                 filter mode is permanently read-only. \
                                 Write verb '{token}' was found after '|'"
                            ),
                        )]);
                    }
                }
            }
            _ => {}
        }
        i += 1;
    }
    Ok(())
}
