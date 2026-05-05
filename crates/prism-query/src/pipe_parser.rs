//! Pipe mode parser: `source | stage | stage …` (BC-2.11.004).
//!
//! Grammar:
//!   pipe_query  := 'FROM' source_ref ('|' pipe_stage)*
//!   pipe_stage  := where_stage | sort_stage | head_stage | tail_stage
//!                | stats_stage | dedup_stage | fields_stage
//!                | join_stage | enrich_stage | limit_stage
//!   where_stage := 'where' expr
//!   sort_stage  := 'sort' sort_expr (',' sort_expr)*
//!   head_stage  := 'head' integer
//!   tail_stage  := 'tail' integer
//!   limit_stage := 'limit' integer   (alias for head)
//!   stats_stage := 'stats' agg_func ['by' field_path]
//!   dedup_stage := 'dedup' field_path (',' field_path)*
//!   fields_stage:= 'fields' ['+' | '-'] field_path (',' field_path)*
//!   join_stage  := 'join' source_ref 'on' field_path
//!   enrich_stage:= 'enrich' ident '(' field_path ')'
//!
//! Mode detection: pipe mode is detected when the input starts with the
//! keyword `FROM` (case-insensitive).
//!
//! All stage keywords are case-insensitive.
//!
//! Story: S-3.01 | BC-2.11.004

use crate::ast::PipeQuery;
use crate::error::ParseError;
use crate::security;

/// Parse a pipe-mode query: `FROM source | stage | stage …`.
///
/// Called by `PrismQlParser::parse` after mode detection confirms the input
/// starts with `FROM`.
///
/// # Errors
/// Returns accumulated `ParseError`s on failure. `skip_then_retry_until`
/// recovery is used to recover past unknown tokens in pipe stages.
pub fn parse_pipe(input: &str) -> Result<PipeQuery, Vec<ParseError>> {
    todo!(
        "S-3.01: build Chumsky parser for FROM source ('|' pipe_stage)* grammar; \
         attach skip_then_retry_until recovery per stage; call \
         security::check_pipe_stage_count on resulting stages; input_len={}",
        input.len()
    )
}

// ── Security re-export for convenient use in tests ────────────────────────────
pub use security::PRISM_MAX_PIPE_STAGES;
