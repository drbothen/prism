//! Query security limit enforcement (BC-2.11.006, DI-019).
//!
//! All checks in this module run before any AST is returned to callers —
//! they are mandatory pre-AST guards, not optional post-processing steps.
//!
//! # Limits (canonical values from BC-2.11.006)
//! - `PRISM_MAX_QUERY_SIZE`: 65,536 bytes (64KB) — EC-001
//! - `PRISM_MAX_NESTING_DEPTH`: 64 — EC-002 (VP-015)
//! - `PRISM_MAX_PIPE_STAGES`: 32 — EC-003
//! - `PRISM_MAX_REGEX_PATTERN_LEN`: 1,024 bytes — BC-2.11.006
//!
//! Story: S-3.01 | BC-2.11.006 | DI-019 | VP-014 | VP-015

use prism_core::error::PrismError;

use crate::ast::{Expr, FilterExpr, PipeQuery, PipeStage, Predicate, SqlQuery};

/// Maximum PrismQL query string size: 64KB (BC-2.11.006, EC-001).
pub const PRISM_MAX_QUERY_SIZE: usize = 65_536;

/// Maximum AST expression nesting depth: 64 (BC-2.11.006, DI-019, EC-002).
///
/// VP-015 proves that `check_nesting_depth` never returns `Ok` when depth
/// exceeds this value. The canonical limit is **64** — not 32.
pub const PRISM_MAX_NESTING_DEPTH: u32 = 64;

/// Maximum number of pipe stages in a single pipe query (BC-2.11.006, EC-003).
pub const PRISM_MAX_PIPE_STAGES: usize = 32;

/// Maximum regex pattern length in `matches` predicates (BC-2.11.006).
pub const PRISM_MAX_REGEX_PATTERN_LEN: usize = 1_024;

/// Maximum number of items in IN lists, ORDER BY, GROUP BY, sort, dedup,
/// fields, and stats aggregate lists (B-8, BC-2.11.006).
///
/// Prevents O(N) denial-of-service attacks via large list literals.
pub const PRISM_MAX_LIST_ITEMS: usize = 1_024;

/// Error code string for security-limit violations.
pub const E_QUERY_003: &str = "E-QUERY-003";

/// Snapshot of all effective parse limits, captured once at the start of
/// `PrismQlParser::parse`.
///
/// # Motivation (F-LOW-002, BC-2.11.006)
/// Each `effective_*_limit()` function reads an env var independently. If another
/// thread calls `std::env::set_var(...)` between guard invocations within a single
/// `parse()` call, different guards would see different limit values — allowing
/// an attacker to exploit the window to bypass a guard. Snapshotting all limits
/// once before any guard runs eliminates this race.
///
/// # Usage
/// ```rust,ignore
/// let limits = ParseLimits::snapshot();
/// // Pass &limits to check_query_size_with, check_paren_depth_with, etc.
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParseLimits {
    /// Effective query size limit in bytes (`PRISM_MAX_QUERY_SIZE`).
    pub query_size: usize,
    /// Effective nesting depth limit (`PRISM_MAX_NESTING_DEPTH`).
    pub nesting_depth: u32,
    /// Effective paren depth limit (same as `nesting_depth`).
    pub paren_depth: u32,
    /// Effective list items limit (`PRISM_MAX_LIST_ITEMS`).
    pub list_items: usize,
    /// Effective pipe stages limit (`PRISM_MAX_PIPE_STAGES`).
    pub pipe_stages: usize,
    /// Effective regex pattern length limit (`PRISM_MAX_REGEX_PATTERN_LEN`).
    pub regex_pattern: usize,
}

impl ParseLimits {
    /// Snapshot all effective limits from environment variables (with clamping).
    ///
    /// This must be called ONCE at the start of `PrismQlParser::parse` so that
    /// all security guards within a single parse call use consistent values.
    pub fn snapshot() -> Self {
        let nesting_depth = effective_nesting_depth_limit();
        Self {
            query_size: effective_query_size_limit(),
            nesting_depth,
            paren_depth: nesting_depth,
            list_items: effective_list_items_limit(),
            pipe_stages: effective_pipe_stage_limit(),
            regex_pattern: effective_regex_pattern_length_limit(),
        }
    }

    /// Check query size using the snapshotted limit (no env-var re-read).
    pub fn check_query_size(&self, raw: &str) -> Result<(), PrismError> {
        if raw.len() > self.query_size {
            return Err(PrismError::QueryExecutionFailed {
                detail: format!(
                    "{E_QUERY_003}: query size {} bytes exceeds maximum allowed {} bytes (64KB limit)",
                    raw.len(),
                    self.query_size
                ),
            });
        }
        Ok(())
    }

    /// Check paren depth using the snapshotted limit (no env-var re-read).
    pub fn check_paren_depth(&self, raw: &str) -> Result<(), PrismError> {
        let limit = self.paren_depth;
        let mut depth: u32 = 0;
        let mut in_sq = false;
        let mut in_dq = false;
        for ch in raw.chars() {
            match ch {
                '\'' if !in_dq => in_sq = !in_sq,
                '"' if !in_sq => in_dq = !in_dq,
                '(' if !in_sq && !in_dq => {
                    depth += 1;
                    if depth > limit {
                        return Err(PrismError::QueryExecutionFailed {
                            detail: format!(
                                "{E_QUERY_003}: expression nesting depth exceeds maximum allowed {limit}"
                            ),
                        });
                    }
                }
                ')' if !in_sq && !in_dq => {
                    depth = depth.saturating_sub(1);
                }
                _ => {}
            }
        }
        if in_sq || in_dq {
            return Err(PrismError::QueryExecutionFailed {
                detail: format!("{E_QUERY_003}: unclosed string literal (quote) at end of input"),
            });
        }
        Ok(())
    }
}

/// Check that a raw query string does not exceed the maximum allowed size.
///
/// # Security
/// MUST run before any parsing attempt. (BC-2.11.006 postcondition 1, EC-001, VP-014)
///
/// # Errors
/// Returns `PrismError::QueryExecutionFailed` with code `E-QUERY-003` if
/// `raw.len() > PRISM_MAX_QUERY_SIZE`.
pub fn check_query_size(raw: &str) -> Result<(), PrismError> {
    let limit = effective_query_size_limit();
    if raw.len() > limit {
        return Err(PrismError::QueryExecutionFailed {
            detail: format!(
                "{E_QUERY_003}: query size {} bytes exceeds maximum allowed {} bytes (64KB limit)",
                raw.len(),
                limit
            ),
        });
    }
    Ok(())
}

/// Recursively check that a `Predicate` AST does not exceed the maximum
/// allowed nesting depth.
///
/// # Security
/// This check covers ALL recursive paths in a `Predicate` tree, including
/// `Predicate::Logical`, `Predicate::Not`, `Predicate::InSubquery`
/// (which embeds a `SqlQuery`). The previous `Expr`-based check had a
/// security gap where subquery nesting was not traversed.
///
/// (BC-2.11.006, DI-019, EC-002, VP-015)
///
/// # Errors
/// Returns `PrismError::QueryExecutionFailed` with code `E-QUERY-003` if
/// the depth of `pred` exceeds `PRISM_MAX_NESTING_DEPTH`.
pub fn check_predicate_nesting_depth(pred: &Predicate, depth: u32) -> Result<(), PrismError> {
    let limit = effective_nesting_depth_limit();
    if depth > limit {
        return Err(PrismError::QueryExecutionFailed {
            detail: format!(
                "{E_QUERY_003}: expression nesting depth {depth} exceeds maximum allowed {limit}"
            ),
        });
    }
    let next = depth + 1;
    match pred {
        Predicate::Compare { lhs, rhs, .. } => {
            check_expr_nesting_depth(lhs, next)?;
            check_expr_nesting_depth(rhs, next)
        }
        Predicate::StringOp { .. } => Ok(()),
        Predicate::Regex { .. } => Ok(()),
        Predicate::In { .. } => Ok(()),
        Predicate::InSubquery { subquery, .. } => check_sql_query_nesting_depth(subquery, next),
        Predicate::Between { .. } => Ok(()),
        Predicate::Cidr { .. } => Ok(()),
        Predicate::Has(_) => Ok(()),
        Predicate::Missing(_) => Ok(()),
        Predicate::IsNull { .. } => Ok(()),
        Predicate::Wildcard { .. } => Ok(()),
        Predicate::Logical { predicates, .. } => {
            for p in predicates {
                check_predicate_nesting_depth(p, next)?;
            }
            Ok(())
        }
        Predicate::Not(inner) => check_predicate_nesting_depth(inner, next),
        // Recovery sentinel — no sub-expressions to check.
        Predicate::RecoveryError => Ok(()),
    }
}

/// Check nesting depth in a `SqlQuery` — recurses into WHERE, HAVING,
/// JOIN ON conditions, and ORDER BY expressions.
pub fn check_sql_query_nesting_depth(sq: &SqlQuery, depth: u32) -> Result<(), PrismError> {
    let limit = effective_nesting_depth_limit();
    if depth > limit {
        return Err(PrismError::QueryExecutionFailed {
            detail: format!(
                "{E_QUERY_003}: expression nesting depth {depth} exceeds maximum allowed {limit}"
            ),
        });
    }
    let next = depth + 1;
    if let Some(w) = &sq.where_ {
        check_predicate_nesting_depth(w, next)?;
    }
    if let Some(h) = &sq.having {
        check_predicate_nesting_depth(h, next)?;
    }
    for join in &sq.joins {
        check_expr_nesting_depth(&join.on, next)?;
    }
    for oe in &sq.order_by {
        check_expr_nesting_depth(&oe.expr, next)?;
    }
    Ok(())
}

/// Check nesting depth of an `Expr` (value expression).
///
/// Recurses into Compare / Logical / Not / InSubquery.
pub fn check_expr_nesting_depth(expr: &Expr, depth: u32) -> Result<(), PrismError> {
    let limit = effective_nesting_depth_limit();
    if depth > limit {
        return Err(PrismError::QueryExecutionFailed {
            detail: format!(
                "{E_QUERY_003}: expression nesting depth {depth} exceeds maximum allowed {limit}"
            ),
        });
    }
    let next = depth + 1;
    match expr {
        Expr::Literal(_) | Expr::Field(_) | Expr::VirtualField(_) | Expr::Star => Ok(()),
        Expr::Compare { lhs, rhs, .. } => {
            check_expr_nesting_depth(lhs, next)?;
            check_expr_nesting_depth(rhs, next)
        }
        Expr::Logical { lhs, rhs, .. } => {
            check_expr_nesting_depth(lhs, next)?;
            check_expr_nesting_depth(rhs, next)
        }
        Expr::Not(inner) => check_expr_nesting_depth(inner, next),
        Expr::In { .. } => Ok(()),
        Expr::InSubquery { subquery, .. } => check_sql_query_nesting_depth(subquery, next),
        Expr::FuncCall(fc) => {
            use crate::ast::FuncCall;
            match fc {
                FuncCall::Aggregate { args, .. } | FuncCall::Scalar { args, .. } => {
                    for arg in args {
                        check_expr_nesting_depth(arg, next)?;
                    }
                    Ok(())
                }
                FuncCall::Window { .. } => Ok(()),
            }
        }
    }
}

/// Backward-compatible entry point: check nesting depth of an `Expr`.
///
/// Called by tests that use `check_nesting_depth(&expr, depth)`.
/// Delegates to `check_expr_nesting_depth`.
pub fn check_nesting_depth(ast: &Expr, depth: u32) -> Result<(), PrismError> {
    check_expr_nesting_depth(ast, depth)
}

/// Check that a query string does not contain parenthesised expressions nested
/// more deeply than `PRISM_MAX_NESTING_DEPTH` (EC-002, BC-2.11.006).
///
/// This pre-parse lexical scan catches structural depth bombs before the
/// Chumsky parser descends into recursion.
///
/// # Semantics (Adv F-MEDIUM-003)
/// This function tracks **max-instantaneous open-paren depth**, NOT the total
/// count of parenthesis events. A sequence of balanced pairs `()()()...`
/// reaches a maximum depth of 1 (at any instant, at most one `(` is open)
/// and will never trigger a rejection, regardless of how many pairs appear.
/// Only genuinely nested structures like `(((...)))` accumulate depth.
///
/// Concretely, `depth` is incremented on `(` and decremented on `)` using
/// `saturating_sub`. The rejection fires only when depth exceeds the limit at
/// the moment of the `(`, not as an aggregate.
///
/// # Unclosed quotes (F-MEDIUM-002)
/// An unclosed `'` or `"` at EOF is explicitly rejected with `E-QUERY-003`.
/// An unclosed quote would otherwise keep the in-quote flag `true` for the
/// remainder of input, silently masking all subsequent `(` characters from
/// the depth counter and allowing an attacker to hide unbounded paren depth
/// from this guard. Rejection at EOF is defence-in-depth: the downstream
/// parser would also reject the unclosed quote, but the pre-parse guard
/// must not be bypassable via unmatched-quote-at-EOF.
pub fn check_paren_depth(raw: &str) -> Result<(), PrismError> {
    let limit = effective_nesting_depth_limit();
    let mut depth: u32 = 0;
    let mut in_sq = false;
    let mut in_dq = false;
    for ch in raw.chars() {
        match ch {
            '\'' if !in_dq => in_sq = !in_sq,
            '"' if !in_sq => in_dq = !in_dq,
            '(' if !in_sq && !in_dq => {
                depth += 1;
                if depth > limit {
                    return Err(PrismError::QueryExecutionFailed {
                        detail: format!(
                            "{E_QUERY_003}: expression nesting depth exceeds maximum allowed {limit}"
                        ),
                    });
                }
            }
            ')' if !in_sq && !in_dq => {
                depth = depth.saturating_sub(1);
            }
            _ => {}
        }
    }
    // F-MEDIUM-002: unclosed quote at EOF is invalid input — reject explicitly.
    // An unclosed quote silently masks all subsequent parens from the depth counter,
    // allowing an attacker to bypass this guard. Rejecting here is defence-in-depth.
    if in_sq || in_dq {
        return Err(PrismError::QueryExecutionFailed {
            detail: format!("{E_QUERY_003}: unclosed string literal (quote) at end of input"),
        });
    }
    Ok(())
}

/// Minimum safe list items limit.
///
/// Prevents `PRISM_MAX_LIST_ITEMS=0` from silently disabling the list-length guard.
pub const MIN_SAFE_LIST_ITEMS: usize = 16;

/// Maximum safe list items limit.
///
/// Prevents `PRISM_MAX_LIST_ITEMS=<huge>` from effectively disabling the guard.
pub const MAX_SAFE_LIST_ITEMS: usize = 16_384;

/// Minimum safe pipe stage count limit.
///
/// Prevents `PRISM_MAX_PIPE_STAGES=0` from disabling the guard entirely.
pub const MIN_SAFE_PIPE_STAGES: usize = 1;

/// Maximum safe pipe stage count limit.
///
/// Prevents `PRISM_MAX_PIPE_STAGES=<huge>` from effectively disabling the guard.
pub const MAX_SAFE_PIPE_STAGES: usize = 256;

/// Minimum safe regex pattern length limit.
///
/// Prevents `PRISM_MAX_REGEX_PATTERN_LEN=0` from silently bypassing the length guard.
pub const MIN_SAFE_REGEX_PATTERN_LEN: usize = 64;

/// Maximum safe regex pattern length limit.
///
/// Prevents `PRISM_MAX_REGEX_PATTERN_LEN=<huge>` from effectively disabling the guard.
pub const MAX_SAFE_REGEX_PATTERN_LEN: usize = 65_536;

/// Return the effective pipe stage limit, clamped to [MIN_SAFE_PIPE_STAGES, MAX_SAFE_PIPE_STAGES].
///
/// If the env var is out of range, a warning is emitted and the value is clamped.
///
/// (Adv F-LOW-002, BC-2.11.006)
pub fn effective_pipe_stage_limit() -> usize {
    match std::env::var("PRISM_MAX_PIPE_STAGES")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
    {
        None => PRISM_MAX_PIPE_STAGES,
        Some(v) if v < MIN_SAFE_PIPE_STAGES => {
            eprintln!(
                "prism-query: PRISM_MAX_PIPE_STAGES={v} is below minimum safe value \
                 ({MIN_SAFE_PIPE_STAGES}); clamping to {MIN_SAFE_PIPE_STAGES}"
            );
            MIN_SAFE_PIPE_STAGES
        }
        Some(v) if v > MAX_SAFE_PIPE_STAGES => {
            eprintln!(
                "prism-query: PRISM_MAX_PIPE_STAGES={v} is above maximum safe value \
                 ({MAX_SAFE_PIPE_STAGES}); clamping to {MAX_SAFE_PIPE_STAGES}"
            );
            MAX_SAFE_PIPE_STAGES
        }
        Some(v) => v,
    }
}

/// Return the effective regex pattern length limit, clamped to
/// [MIN_SAFE_REGEX_PATTERN_LEN, MAX_SAFE_REGEX_PATTERN_LEN].
///
/// If the env var is out of range, a warning is emitted and the value is clamped.
///
/// (Adv F-LOW-002, BC-2.11.006)
pub fn effective_regex_pattern_length_limit() -> usize {
    match std::env::var("PRISM_MAX_REGEX_PATTERN_LEN")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
    {
        None => PRISM_MAX_REGEX_PATTERN_LEN,
        Some(v) if v < MIN_SAFE_REGEX_PATTERN_LEN => {
            eprintln!(
                "prism-query: PRISM_MAX_REGEX_PATTERN_LEN={v} is below minimum safe value \
                 ({MIN_SAFE_REGEX_PATTERN_LEN}); clamping to {MIN_SAFE_REGEX_PATTERN_LEN}"
            );
            MIN_SAFE_REGEX_PATTERN_LEN
        }
        Some(v) if v > MAX_SAFE_REGEX_PATTERN_LEN => {
            eprintln!(
                "prism-query: PRISM_MAX_REGEX_PATTERN_LEN={v} is above maximum safe value \
                 ({MAX_SAFE_REGEX_PATTERN_LEN}); clamping to {MAX_SAFE_REGEX_PATTERN_LEN}"
            );
            MAX_SAFE_REGEX_PATTERN_LEN
        }
        Some(v) => v,
    }
}

/// Check that a pipe query does not contain more than the maximum allowed
/// number of stages. (BC-2.11.006, EC-003)
pub fn check_pipe_stage_count(stages: &[PipeStage]) -> Result<(), PrismError> {
    let limit = effective_pipe_stage_limit();
    if stages.len() > limit {
        return Err(PrismError::QueryExecutionFailed {
            detail: format!(
                "{E_QUERY_003}: pipe stage count {} exceeds maximum allowed {}",
                stages.len(),
                limit
            ),
        });
    }
    Ok(())
}

/// Return the effective list items limit, clamped to
/// [MIN_SAFE_LIST_ITEMS, MAX_SAFE_LIST_ITEMS].
///
/// If the env var is out of range, a warning is emitted and the value is clamped.
///
/// (F-LOW-003, BC-2.11.006)
pub fn effective_list_items_limit() -> usize {
    match std::env::var("PRISM_MAX_LIST_ITEMS")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
    {
        None => PRISM_MAX_LIST_ITEMS,
        Some(v) if v < MIN_SAFE_LIST_ITEMS => {
            eprintln!(
                "prism-query: PRISM_MAX_LIST_ITEMS={v} is below minimum safe value \
                 ({MIN_SAFE_LIST_ITEMS}); clamping to {MIN_SAFE_LIST_ITEMS}"
            );
            MIN_SAFE_LIST_ITEMS
        }
        Some(v) if v > MAX_SAFE_LIST_ITEMS => {
            eprintln!(
                "prism-query: PRISM_MAX_LIST_ITEMS={v} is above maximum safe value \
                 ({MAX_SAFE_LIST_ITEMS}); clamping to {MAX_SAFE_LIST_ITEMS}"
            );
            MAX_SAFE_LIST_ITEMS
        }
        Some(v) => v,
    }
}

/// Check that a list (IN items, ORDER BY items, GROUP BY items, etc.) does not
/// exceed the maximum allowed item count (B-8, BC-2.11.006).
///
/// # Arguments
/// - `count`: the number of items in the list.
/// - `context`: a human-readable label for error messages (e.g., `"IN list"`,
///   `"ORDER BY"`, `"GROUP BY"`, `"sort"`, `"dedup"`, `"fields"`, `"stats"`).
///
/// # Errors
/// Returns `PrismError::QueryExecutionFailed` with code `E-QUERY-003` if
/// `count > effective_list_items_limit()`.
pub fn check_list_length(count: usize, context: &str) -> Result<(), PrismError> {
    let limit = effective_list_items_limit();
    if count > limit {
        return Err(PrismError::QueryExecutionFailed {
            detail: format!(
                "{E_QUERY_003}: {context} item count {count} exceeds maximum allowed \
                 {limit}"
            ),
        });
    }
    Ok(())
}

/// Check that a regex pattern string does not exceed the maximum allowed length.
/// (BC-2.11.006, CWE-1333, Adv F-HIGH-003)
///
/// Uses `effective_regex_pattern_length_limit()` so that the env-var override
/// `PRISM_MAX_REGEX_PATTERN_LEN` is respected (clamped to safe range).
/// This function is the single source of truth for the regex length limit —
/// `RegexLiteral::new` delegates here rather than duplicating the constant.
pub fn check_regex_pattern_length(pattern: &str) -> Result<(), PrismError> {
    let limit = effective_regex_pattern_length_limit();
    if pattern.len() > limit {
        return Err(PrismError::QueryExecutionFailed {
            detail: format!(
                "{E_QUERY_003}: regex pattern length {} bytes exceeds maximum allowed {} bytes",
                pattern.len(),
                limit
            ),
        });
    }
    Ok(())
}

/// Check list lengths in a `FilterExpr` (IN lists in predicates).
///
/// (B-8, BC-2.11.006)
pub fn check_filter_list_sizes(fe: &FilterExpr) -> Result<(), PrismError> {
    check_predicate_list_sizes(&fe.predicate)
}

/// Check list lengths in a `SqlQuery` (IN lists, ORDER BY, GROUP BY).
///
/// (B-8, BC-2.11.006)
pub fn check_sql_list_sizes(sq: &SqlQuery) -> Result<(), PrismError> {
    check_list_length(sq.order_by.len(), "ORDER BY")?;
    check_list_length(sq.group_by.len(), "GROUP BY")?;
    if let Some(w) = &sq.where_ {
        check_predicate_list_sizes(w)?;
    }
    if let Some(h) = &sq.having {
        check_predicate_list_sizes(h)?;
    }
    for join in &sq.joins {
        check_expr_list_sizes(&join.on)?;
    }
    Ok(())
}

/// Check list lengths in a `PipeQuery` (sort, dedup, fields, stats, IN lists).
///
/// (B-8, BC-2.11.006)
pub fn check_pipe_list_sizes(pq: &PipeQuery) -> Result<(), PrismError> {
    for stage in &pq.stages {
        check_pipe_stage_list_sizes(stage)?;
    }
    Ok(())
}

/// Check list lengths in a single pipe stage.
fn check_pipe_stage_list_sizes(stage: &PipeStage) -> Result<(), PrismError> {
    use crate::ast::PipeStage;
    match stage {
        PipeStage::Where(pred) => check_predicate_list_sizes(pred),
        PipeStage::Sort(exprs) => check_list_length(exprs.len(), "sort"),
        PipeStage::Dedup(fields) => check_list_length(fields.len(), "dedup"),
        PipeStage::Fields(fs) => check_list_length(fs.fields.len(), "fields"),
        PipeStage::Stats(ss) => {
            check_list_length(ss.aggregates.len(), "stats aggregate")?;
            check_list_length(ss.by_fields.len(), "stats BY")
        }
        _ => Ok(()),
    }
}

/// Check list lengths in a `Predicate` (IN lists).
fn check_predicate_list_sizes(pred: &Predicate) -> Result<(), PrismError> {
    match pred {
        Predicate::In { values, .. } => check_list_length(values.len(), "IN list"),
        Predicate::Logical { predicates, .. } => {
            for p in predicates {
                check_predicate_list_sizes(p)?;
            }
            Ok(())
        }
        Predicate::Not(inner) => check_predicate_list_sizes(inner),
        Predicate::Compare { lhs, rhs, .. } => {
            check_expr_list_sizes(lhs)?;
            check_expr_list_sizes(rhs)
        }
        Predicate::InSubquery { subquery, .. } => check_sql_list_sizes(subquery),
        _ => Ok(()),
    }
}

/// Check list lengths in an `Expr` (IN lists).
fn check_expr_list_sizes(expr: &Expr) -> Result<(), PrismError> {
    match expr {
        Expr::In { values, .. } => check_list_length(values.len(), "IN list"),
        Expr::Logical { lhs, rhs, .. } => {
            check_expr_list_sizes(lhs)?;
            check_expr_list_sizes(rhs)
        }
        Expr::Not(inner) => check_expr_list_sizes(inner),
        Expr::Compare { lhs, rhs, .. } => {
            check_expr_list_sizes(lhs)?;
            check_expr_list_sizes(rhs)
        }
        Expr::InSubquery { subquery, .. } => check_sql_list_sizes(subquery),
        _ => Ok(()),
    }
}

/// Minimum safe query size limit (1KB). Env-var values below this are clamped up.
///
/// Prevents `PRISM_MAX_QUERY_SIZE=0` from silently bypassing the size guard.
pub const MIN_SAFE_QUERY_SIZE: usize = 1_024;

/// Maximum safe query size limit (1MB). Env-var values above this are clamped down.
///
/// Prevents `PRISM_MAX_QUERY_SIZE=<huge>` from effectively disabling the guard.
pub const MAX_SAFE_QUERY_SIZE: usize = 1_048_576;

/// Minimum safe nesting depth limit. Env-var values below this are clamped up.
///
/// Prevents `PRISM_MAX_NESTING_DEPTH=0` from silently bypassing the nesting guard.
pub const MIN_SAFE_NESTING_DEPTH: u32 = 8;

/// Maximum safe nesting depth limit. Env-var values above this are clamped down.
///
/// Prevents `PRISM_MAX_NESTING_DEPTH=<huge>` from effectively disabling the guard.
pub const MAX_SAFE_NESTING_DEPTH: u32 = 256;

/// Return the effective query size limit, clamped to [MIN_SAFE_QUERY_SIZE, MAX_SAFE_QUERY_SIZE].
///
/// If the env var is out of range, a warning is emitted and the value is clamped.
pub fn effective_query_size_limit() -> usize {
    match std::env::var("PRISM_MAX_QUERY_SIZE")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
    {
        None => PRISM_MAX_QUERY_SIZE,
        Some(v) if v < MIN_SAFE_QUERY_SIZE => {
            eprintln!(
                "prism-query: PRISM_MAX_QUERY_SIZE={v} is below minimum safe value \
                 ({MIN_SAFE_QUERY_SIZE}); clamping to {MIN_SAFE_QUERY_SIZE}"
            );
            MIN_SAFE_QUERY_SIZE
        }
        Some(v) if v > MAX_SAFE_QUERY_SIZE => {
            eprintln!(
                "prism-query: PRISM_MAX_QUERY_SIZE={v} is above maximum safe value \
                 ({MAX_SAFE_QUERY_SIZE}); clamping to {MAX_SAFE_QUERY_SIZE}"
            );
            MAX_SAFE_QUERY_SIZE
        }
        Some(v) => v,
    }
}

/// Return the effective nesting depth limit, clamped to [MIN_SAFE_NESTING_DEPTH, MAX_SAFE_NESTING_DEPTH].
///
/// If the env var is out of range, a warning is emitted and the value is clamped.
pub fn effective_nesting_depth_limit() -> u32 {
    match std::env::var("PRISM_MAX_NESTING_DEPTH")
        .ok()
        .and_then(|v| v.parse::<u32>().ok())
    {
        None => PRISM_MAX_NESTING_DEPTH,
        Some(v) if v < MIN_SAFE_NESTING_DEPTH => {
            eprintln!(
                "prism-query: PRISM_MAX_NESTING_DEPTH={v} is below minimum safe value \
                 ({MIN_SAFE_NESTING_DEPTH}); clamping to {MIN_SAFE_NESTING_DEPTH}"
            );
            MIN_SAFE_NESTING_DEPTH
        }
        Some(v) if v > MAX_SAFE_NESTING_DEPTH => {
            eprintln!(
                "prism-query: PRISM_MAX_NESTING_DEPTH={v} is above maximum safe value \
                 ({MAX_SAFE_NESTING_DEPTH}); clamping to {MAX_SAFE_NESTING_DEPTH}"
            );
            MAX_SAFE_NESTING_DEPTH
        }
        Some(v) => v,
    }
}
