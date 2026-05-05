//! Query security limit enforcement (BC-2.11.006, DI-019).
//!
//! All checks in this module run before any AST is returned to callers —
//! they are mandatory pre-AST guards, not optional post-processing steps.
//!
//! # Limits (canonical values from BC-2.11.006)
//! - `PRISM_MAX_QUERY_SIZE`: 65,536 bytes (64KB) — EC-001
//! - `PRISM_MAX_NESTING_DEPTH`: 64 — EC-002 (VP-015; canonical limit is 64,
//!   not 32; see S-3.01 v1.6 changelog and BC-2.11.006 DI-019 EC-002)
//! - `PRISM_MAX_PIPE_STAGES`: 32 — EC-003
//! - `PRISM_MAX_REGEX_PATTERN_LEN`: 1,024 bytes — BC-2.11.006 regex limit
//!
//! Story: S-3.01 | BC-2.11.006 | DI-019 | VP-014 | VP-015

use prism_core::error::PrismError;

use crate::ast::{Expr, PipeStage};

/// Maximum PrismQL query string size: 64KB (BC-2.11.006, EC-001).
///
/// Overridable at runtime via the `PRISM_MAX_QUERY_SIZE` environment variable.
/// The constant is the compile-time default.
pub const PRISM_MAX_QUERY_SIZE: usize = 65_536;

/// Maximum AST expression nesting depth: 64 (BC-2.11.006, DI-019, EC-002).
///
/// VP-015 proves that `check_nesting_depth` never returns `Ok` when depth
/// exceeds this value. The canonical limit is **64** — not 32. See S-3.01
/// v1.6 changelog.
///
/// Overridable at runtime via the `PRISM_MAX_NESTING_DEPTH` environment variable.
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
/// This check MUST run before any parsing attempt. An oversized query is
/// rejected immediately; no AST is produced. (BC-2.11.006 postcondition 1,
/// EC-001, VP-014)
///
/// # Errors
/// Returns `PrismError::QueryExecutionFailed` with code `E-QUERY-003` if
/// `raw.len() > PRISM_MAX_QUERY_SIZE` (or the value of the
/// `PRISM_MAX_QUERY_SIZE` env var if set).
pub fn check_query_size(raw: &str) -> Result<(), PrismError> {
    todo!(
        "S-3.01: read PRISM_MAX_QUERY_SIZE env var (default {default}); \
         return Err(PrismError::QueryExecutionFailed {{ detail: … }}) if \
         raw.len() ({len}) > limit",
        default = PRISM_MAX_QUERY_SIZE,
        len = raw.len()
    )
}

/// Recursively check that an expression AST does not exceed the maximum
/// allowed nesting depth.
///
/// # Security
/// This check MUST run on the fully-parsed AST before it is returned to the
/// caller. Deeply nested expressions can cause stack overflows during
/// evaluation; this guard enforces the limit at parse time.
/// (BC-2.11.006, DI-019, EC-002, VP-015)
///
/// The canonical maximum depth is `PRISM_MAX_NESTING_DEPTH` = **64**.
///
/// # Errors
/// Returns `PrismError::QueryExecutionFailed` with code `E-QUERY-003` if
/// the depth of `ast` exceeds `PRISM_MAX_NESTING_DEPTH` (or the value of
/// the `PRISM_MAX_NESTING_DEPTH` env var if set).
pub fn check_nesting_depth(_ast: &Expr, depth: u32) -> Result<(), PrismError> {
    todo!(
        "S-3.01: recursively descend into Expr variants; count depth; \
         return Err(PrismError::QueryExecutionFailed {{ detail: … }}) \
         if depth ({depth}) > PRISM_MAX_NESTING_DEPTH ({max})",
        depth = depth,
        max = PRISM_MAX_NESTING_DEPTH
    )
}

/// Check that a pipe query does not contain more than the maximum allowed
/// number of stages.
///
/// # Security
/// Enforces BC-2.11.006 pipe stage count limit (EC-003). Called by
/// `pipe_parser::parse_pipe` after the stage list is parsed.
///
/// # Errors
/// Returns `PrismError::QueryExecutionFailed` with code `E-QUERY-003` if
/// `stages.len() > PRISM_MAX_PIPE_STAGES`.
pub fn check_pipe_stage_count(stages: &[PipeStage]) -> Result<(), PrismError> {
    todo!(
        "S-3.01: return Err(PrismError::QueryExecutionFailed {{ detail: … }}) \
         if stages.len() ({len}) > PRISM_MAX_PIPE_STAGES ({max})",
        len = stages.len(),
        max = PRISM_MAX_PIPE_STAGES
    )
}

/// Check that a regex pattern string does not exceed the maximum allowed
/// length.
///
/// # Security
/// Enforces BC-2.11.006 regex pattern length limit. Called for every
/// `matches` predicate found during parsing.
///
/// # Errors
/// Returns `PrismError::QueryExecutionFailed` with code `E-QUERY-003` if
/// `pattern.len() > PRISM_MAX_REGEX_PATTERN_LEN`.
pub fn check_regex_pattern_length(pattern: &str) -> Result<(), PrismError> {
    todo!(
        "S-3.01: return Err(PrismError::QueryExecutionFailed {{ detail: … }}) \
         if pattern.len() ({len}) > PRISM_MAX_REGEX_PATTERN_LEN ({max})",
        len = pattern.len(),
        max = PRISM_MAX_REGEX_PATTERN_LEN
    )
}

/// Return the effective query size limit, reading `PRISM_MAX_QUERY_SIZE`
/// from the environment if set, otherwise falling back to the compile-time
/// default.
///
/// Called by `check_query_size`. Separated out to enable deterministic
/// testing without process-level env var mutation.
pub fn effective_query_size_limit() -> usize {
    todo!("S-3.01: read PRISM_MAX_QUERY_SIZE env var; parse as usize; fall back to PRISM_MAX_QUERY_SIZE")
}

/// Return the effective nesting depth limit, reading `PRISM_MAX_NESTING_DEPTH`
/// from the environment if set, otherwise falling back to the compile-time
/// default of 64.
pub fn effective_nesting_depth_limit() -> u32 {
    todo!("S-3.01: read PRISM_MAX_NESTING_DEPTH env var; parse as u32; fall back to PRISM_MAX_NESTING_DEPTH")
}
