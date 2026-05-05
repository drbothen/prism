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

use crate::ast::{Expr, PipeStage, Predicate, SqlQuery};

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

/// Error code string for security-limit violations.
pub const E_QUERY_003: &str = "E-QUERY-003";

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
    Ok(())
}

/// Check that a pipe query does not contain more than the maximum allowed
/// number of stages. (BC-2.11.006, EC-003)
pub fn check_pipe_stage_count(stages: &[PipeStage]) -> Result<(), PrismError> {
    if stages.len() > PRISM_MAX_PIPE_STAGES {
        return Err(PrismError::QueryExecutionFailed {
            detail: format!(
                "{E_QUERY_003}: pipe stage count {} exceeds maximum allowed {}",
                stages.len(),
                PRISM_MAX_PIPE_STAGES
            ),
        });
    }
    Ok(())
}

/// Check that a regex pattern string does not exceed the maximum allowed length.
/// (BC-2.11.006, CWE-1333)
pub fn check_regex_pattern_length(pattern: &str) -> Result<(), PrismError> {
    if pattern.len() > PRISM_MAX_REGEX_PATTERN_LEN {
        return Err(PrismError::QueryExecutionFailed {
            detail: format!(
                "{E_QUERY_003}: regex pattern length {} bytes exceeds maximum allowed {} bytes",
                pattern.len(),
                PRISM_MAX_REGEX_PATTERN_LEN
            ),
        });
    }
    Ok(())
}

/// Return the effective query size limit.
pub fn effective_query_size_limit() -> usize {
    std::env::var("PRISM_MAX_QUERY_SIZE")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(PRISM_MAX_QUERY_SIZE)
}

/// Return the effective nesting depth limit.
pub fn effective_nesting_depth_limit() -> u32 {
    std::env::var("PRISM_MAX_NESTING_DEPTH")
        .ok()
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(PRISM_MAX_NESTING_DEPTH)
}
