//! Pipe mode parser: `source | stage | stage …` (BC-2.11.004).
//!
//! Grammar:
//!   pipe_query  := ['FROM' source_ref | source_ref] ('|' pipe_stage)*
//!               | '|' pipe_stage ('|' pipe_stage)*   -- no source prefix (EC-11-009)
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
//! keyword `FROM` (case-insensitive) or starts with `|`.
//!
//! All stage keywords are case-insensitive.
//!
//! Story: S-3.01 | BC-2.11.004

use chumsky::prelude::*;

use crate::ast::{
    AggFunc, EnrichStage, FieldPath, FieldsStage, JoinStage, PipeQuery, PipeStage, SortDirection,
    SortExpr, SourceRef, StatsStage,
};
use crate::error::ParseError;
use crate::error_recovery::rich_to_parse_error;
use crate::filter_parser::{build_expr_parser, build_source_ref_parser};
use crate::security;

// ── Security re-export for convenient use in tests ────────────────────────────
pub use security::PRISM_MAX_PIPE_STAGES;

/// Parse a pipe-mode query: `[FROM source | source] (| stage)*`.
///
/// Called by `PrismQlParser::parse` after mode detection confirms the input
/// starts with `FROM` or `|`.
///
/// # Errors
/// Returns accumulated `ParseError`s on failure. `skip_then_retry_until`
/// recovery is used to recover past unknown tokens in pipe stages.
pub fn parse_pipe(input: &str) -> Result<PipeQuery, Vec<ParseError>> {
    let parser = build_pipe_parser();
    let (result, errs) = parser.parse(input).into_output_errors();
    if errs.is_empty() {
        if let Some(pq) = result {
            // Security: check pipe stage count.
            security::check_pipe_stage_count(&pq.stages)
                .map_err(|e| vec![ParseError::new(0, e.to_string())])?;
            return Ok(pq);
        }
    }
    let parse_errors: Vec<ParseError> = errs.iter().map(rich_to_parse_error).collect();
    if parse_errors.is_empty() {
        Err(vec![ParseError::new(0, "E-QUERY-001: pipe parse failed")])
    } else {
        Err(parse_errors)
    }
}

/// Build the Chumsky pipe-mode parser.
///
/// Returns a parser that accepts `[FROM source | source] ('|' pipe_stage)*`
/// or `'|' pipe_stage ('|' pipe_stage)*` (no-source prefix, EC-11-009).
pub fn build_pipe_parser<'a>(
) -> impl Parser<'a, &'a str, PipeQuery, extra::Err<Rich<'a, char>>> + Clone {
    let expr = build_expr_parser();
    let source_ref = build_source_ref_parser();

    // Field path parser.
    let ident_char = any::<&str, extra::Err<Rich<char>>>()
        .filter(|c: &char| c.is_ascii_alphanumeric() || *c == '_');
    let field_segment = ident_char.repeated().at_least(1).to_slice();
    let field_path = field_segment
        .separated_by(just('.'))
        .at_least(1)
        .collect::<Vec<&str>>()
        .map(|segs: Vec<&str>| FieldPath {
            segments: segs.into_iter().map(|s| s.to_string()).collect(),
        });

    // Identifier (for keywords, infusion names, etc.)
    let ident = ident_char
        .repeated()
        .at_least(1)
        .to_slice()
        .map(|s: &str| s.to_string());

    // Integer literal (non-negative for stage args).
    let uint = text::int(10).to_slice().try_map(|s: &str, span| {
        s.parse::<u64>()
            .map_err(|e| Rich::custom(span, format!("invalid integer: {e}")))
    });

    // Sort expression: `field [asc|desc]`
    let sort_direction = choice((
        text::keyword("desc")
            .or(text::keyword("DESC"))
            .to(SortDirection::Desc),
        text::keyword("asc")
            .or(text::keyword("ASC"))
            .to(SortDirection::Asc),
    ))
    .padded()
    .or_not()
    .map(|dir| dir.unwrap_or(SortDirection::Asc));

    let sort_expr = field_path
        .padded()
        .then(sort_direction)
        .map(|(field, direction)| SortExpr { field, direction });

    // Aggregation functions: count | sum(field) | avg(field) | min(field) | max(field)
    let agg_func = choice((
        text::keyword("count")
            .or(text::keyword("COUNT"))
            .padded()
            .to(AggFunc::Count),
        text::keyword("sum")
            .or(text::keyword("SUM"))
            .padded()
            .ignore_then(
                field_path
                    .padded()
                    .delimited_by(just('(').padded(), just(')').padded()),
            )
            .map(AggFunc::Sum),
        text::keyword("avg")
            .or(text::keyword("AVG"))
            .padded()
            .ignore_then(
                field_path
                    .padded()
                    .delimited_by(just('(').padded(), just(')').padded()),
            )
            .map(AggFunc::Avg),
        text::keyword("min")
            .or(text::keyword("MIN"))
            .padded()
            .ignore_then(
                field_path
                    .padded()
                    .delimited_by(just('(').padded(), just(')').padded()),
            )
            .map(AggFunc::Min),
        text::keyword("max")
            .or(text::keyword("MAX"))
            .padded()
            .ignore_then(
                field_path
                    .padded()
                    .delimited_by(just('(').padded(), just(')').padded()),
            )
            .map(AggFunc::Max),
    ));

    // Individual pipe stages.
    let where_stage = text::keyword("where")
        .or(text::keyword("WHERE"))
        .padded()
        .ignore_then(expr.clone().padded())
        .map(PipeStage::Where);

    let sort_stage = text::keyword("sort")
        .or(text::keyword("SORT"))
        .padded()
        .ignore_then(
            sort_expr
                .padded()
                .separated_by(just(',').padded())
                .at_least(1)
                .collect::<Vec<_>>(),
        )
        .map(PipeStage::Sort);

    let head_stage = text::keyword("head")
        .or(text::keyword("HEAD"))
        .padded()
        .ignore_then(uint.padded())
        .map(PipeStage::Limit);

    let tail_stage = text::keyword("tail")
        .or(text::keyword("TAIL"))
        .padded()
        .ignore_then(uint.padded())
        .map(PipeStage::Tail);

    let limit_stage = text::keyword("limit")
        .or(text::keyword("LIMIT"))
        .padded()
        .ignore_then(uint.padded())
        .map(PipeStage::Limit);

    let stats_stage = text::keyword("stats")
        .or(text::keyword("STATS"))
        .padded()
        .ignore_then(agg_func.padded())
        .then(
            text::keyword("by")
                .or(text::keyword("BY"))
                .padded()
                .ignore_then(field_path.padded())
                .or_not(),
        )
        .map(|(func, by)| PipeStage::Stats(StatsStage { func, by }));

    let dedup_stage = text::keyword("dedup")
        .or(text::keyword("DEDUP"))
        .padded()
        .ignore_then(
            field_path
                .padded()
                .separated_by(just(',').padded())
                .at_least(1)
                .collect::<Vec<_>>(),
        )
        .map(PipeStage::Dedup);

    let fields_stage = text::keyword("fields")
        .or(text::keyword("FIELDS"))
        .padded()
        .ignore_then(
            choice((just('+').padded().to(true), just('-').padded().to(false)))
                .or_not()
                .map(|sign| sign.unwrap_or(true)),
        )
        .then(
            field_path
                .padded()
                .separated_by(just(',').padded())
                .at_least(1)
                .collect::<Vec<_>>(),
        )
        .map(|(include, fields)| PipeStage::Fields(FieldsStage { include, fields }));

    let join_stage = text::keyword("join")
        .or(text::keyword("JOIN"))
        .padded()
        .ignore_then(source_ref.clone().padded())
        .then_ignore(text::keyword("on").or(text::keyword("ON")).padded())
        .then(field_path.padded())
        .map(|(src, on_field)| {
            PipeStage::Join(JoinStage {
                source: src,
                on: on_field,
            })
        });

    let enrich_stage = text::keyword("enrich")
        .or(text::keyword("ENRICH"))
        .padded()
        .ignore_then(ident.padded())
        .then(
            field_path
                .padded()
                .delimited_by(just('(').padded(), just(')').padded()),
        )
        .map(|(infusion, field)| PipeStage::Enrich(EnrichStage { infusion, field }));

    let pipe_stage = choice((
        where_stage,
        sort_stage,
        head_stage,
        tail_stage,
        limit_stage,
        stats_stage,
        dedup_stage,
        fields_stage,
        join_stage,
        enrich_stage,
    ));

    // Stages separated by `|`
    let stages_with_pipe = just('|')
        .padded()
        .ignore_then(pipe_stage.clone().padded())
        .repeated()
        .collect::<Vec<_>>();

    // Variant 1: `FROM source ('|' pipe_stage)*`
    let from_source_query = text::keyword("FROM")
        .or(text::keyword("from"))
        .padded()
        .ignore_then(source_ref.clone().padded())
        .then(stages_with_pipe.clone())
        .map(|(source, stages)| PipeQuery { source, stages });

    // Variant 2: `source ('|' pipe_stage)+` — bare source with pipe stages
    // (called only when is_pipe_mode confirmed a pipe keyword after `|`)
    let bare_source_query = source_ref
        .clone()
        .padded()
        .then(stages_with_pipe.clone())
        .map(|(source, stages)| PipeQuery { source, stages });

    // Variant 3: `'|' pipe_stage ('|' pipe_stage)*` — no source prefix (EC-11-009)
    // The leading `|` indicates start-of-pipeline; first stage follows immediately.
    let no_source_query = just('|')
        .padded()
        .ignore_then(
            pipe_stage
                .clone()
                .padded()
                .then(stages_with_pipe.clone())
                .map(|(first, mut rest)| {
                    let mut stages = vec![first];
                    stages.append(&mut rest);
                    stages
                }),
        )
        .map(|stages| PipeQuery {
            source: SourceRef { raw: String::new() },
            stages,
        });

    choice((from_source_query, no_source_query, bare_source_query))
}
