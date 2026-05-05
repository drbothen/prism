//! Pipe mode parser: `source | stage | stage …` (BC-2.11.004).
//!
//! Grammar (prismql-grammar.md §6):
//!   pipe_query  := ['FROM' source_ref | source_ref] ('|' pipe_stage)*
//!               | '|' pipe_stage ('|' pipe_stage)*   -- no source prefix (EC-11-009)
//!   pipe_stage  := where_stage | sort_stage | head_stage | tail_stage
//!                | stats_stage | dedup_stage | fields_stage
//!                | join_stage | enrich_stage | limit_stage
//!   where_stage := 'where' predicate
//!   stats_stage := 'stats' stat_fn (',' stat_fn)* ['BY' field (',' field)*]
//!   stat_fn     := agg_func ['AS' ident]
//!   join_stage  := 'join' [join_kind] source 'ON' field ['==' field]
//!
//! All stage keywords are case-insensitive.
//!
//! Story: S-3.01 | BC-2.11.004

use ordered_float::OrderedFloat;

use chumsky::prelude::*;

use crate::ast::{
    AggFunc, EnrichStage, FieldPath, FieldsStage, JoinCondition, JoinKind, JoinStage, PipeQuery,
    PipeStage, SortDirection, SortExpr, SourceRef, Span, StatFunction, StatsStage,
};
use crate::error::ParseError;
use crate::error_recovery::rich_to_parse_error;
use crate::filter_parser::{build_predicate_parser, build_source_ref_parser};
use crate::security;

// ── Security re-export for convenient use in tests ────────────────────────────
pub use security::PRISM_MAX_PIPE_STAGES;

/// Parse a pipe-mode query: `[FROM source | source] (| stage)*`.
///
/// Called by `PrismQlParser::parse` after mode detection confirms pipe mode.
///
/// # Errors
/// Returns accumulated `ParseError`s on failure.
pub fn parse_pipe(input: &str) -> Result<PipeQuery, Vec<ParseError>> {
    let parser = build_pipe_parser();
    let (result, errs) = parser.parse(input).into_output_errors();
    if errs.is_empty() {
        if let Some(pq) = result {
            // Security: check pipe stage count.
            security::check_pipe_stage_count(&pq.stages)
                .map_err(|e| vec![ParseError::new(0, e.to_string())])?;

            // Security: check AST nesting depth in every pipe stage containing
            // a predicate or expression (B-2, BC-2.11.006, DI-019, EC-002).
            for stage in &pq.stages {
                check_pipe_stage_depth(stage)
                    .map_err(|e| vec![ParseError::new(0, e.to_string())])?;
            }

            // Security: check list item counts in all pipe stages (B-8, BC-2.11.006).
            security::check_pipe_list_sizes(&pq)
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

/// Walk a pipe stage and check nesting depth for any embedded predicates
/// or expressions.
///
/// (B-2, BC-2.11.006, DI-019)
fn check_pipe_stage_depth(
    stage: &crate::ast::PipeStage,
) -> Result<(), prism_core::error::PrismError> {
    use crate::ast::PipeStage;
    match stage {
        PipeStage::Where(pred) => security::check_predicate_nesting_depth(pred, 0),
        // Sort, Dedup, Fields, Stats, Join, Enrich contain only field paths
        // and simple agg functions — no unbounded predicate nesting.
        // Limit / Tail contain only a u64 scalar.
        _ => Ok(()),
    }
}

/// Build the Chumsky pipe-mode parser.
///
/// Returns a parser that accepts `[FROM source | source] ('|' pipe_stage)*`
/// or `'|' pipe_stage ('|' pipe_stage)*` (no-source prefix, EC-11-009).
#[allow(clippy::clone_on_copy)]
pub fn build_pipe_parser<'a>(
) -> impl Parser<'a, &'a str, PipeQuery, extra::Err<Rich<'a, char>>> + Clone {
    let predicate = build_predicate_parser();
    let source_ref = build_source_ref_parser();

    // Field path parser.
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
        .clone()
        .padded()
        .then(sort_direction)
        .map(|(field, direction)| SortExpr { field, direction });

    // Case-insensitive keyword helper for pipe parsers.
    let kw_ci = |k: &'static str| {
        ident_char
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

    // Aggregation function parser for pipe `stats` stage.
    // Follows prismql-grammar.md §6: stat_fn := agg_func ['AS' ident]
    //
    // Supported: count | count() | count(*) | sum(f) | avg(f) | min(f) | max(f)
    //          | distinct_count(f) | percentile(f, p)

    // percentile(field, p)
    let percentile_agg = kw_ci("percentile").padded().ignore_then(
        field_path
            .clone()
            .padded()
            .then_ignore(just(',').padded())
            .then(
                just('-')
                    .or_not()
                    .then(text::int(10))
                    .then(just('.').then(text::digits(10)).or_not())
                    .to_slice()
                    .try_map(|s: &str, span| {
                        s.parse::<f64>().map_err(|e| {
                            Rich::custom(span, format!("invalid percentile value: {e}"))
                        })
                    }),
            )
            .try_map(|(fp, p), span| {
                if !(0.0..=100.0).contains(&p) {
                    return Err(Rich::custom(
                        span,
                        format!("E-QUERY-001: percentile p={p} out of range [0, 100]"),
                    ));
                }
                Ok(AggFunc::Percentile {
                    field: fp,
                    p: OrderedFloat(p),
                })
            })
            .delimited_by(just('(').padded(), just(')').padded()),
    );

    // distinct_count(field)
    let distinct_count_agg = kw_ci("distinct_count").padded().ignore_then(
        field_path
            .clone()
            .padded()
            .map(AggFunc::DistinctCount)
            .delimited_by(just('(').padded(), just(')').padded()),
    );

    // count(*) | count() | bare count
    let count_agg = kw_ci("count").padded().ignore_then(
        choice((
            just('*')
                .padded()
                .delimited_by(just('(').padded(), just(')').padded())
                .to(AggFunc::Count),
            field_path
                .clone()
                .padded()
                .map(AggFunc::CountField)
                .delimited_by(just('(').padded(), just(')').padded()),
            just('(')
                .padded()
                .then_ignore(just(')').padded())
                .to(AggFunc::Count),
        ))
        .or_not()
        .map(|o| o.unwrap_or(AggFunc::Count)),
    );

    // sum(f) | avg(f) | min(f) | max(f)
    let generic_agg = choice((
        kw_ci("sum").padded().to("sum"),
        kw_ci("avg").padded().to("avg"),
        kw_ci("min").padded().to("min"),
        kw_ci("max").padded().to("max"),
    ))
    .then(
        field_path
            .clone()
            .padded()
            .delimited_by(just('(').padded(), just(')').padded()),
    )
    .map(|(fname, fp)| match fname {
        "sum" => AggFunc::Sum(fp),
        "avg" => AggFunc::Avg(fp),
        "min" => AggFunc::Min(fp),
        "max" => AggFunc::Max(fp),
        _ => unreachable!(),
    });

    // Single agg function
    let agg_func = choice((percentile_agg, distinct_count_agg, count_agg, generic_agg));

    // stat_fn := agg_func ['AS' ident]
    let stat_fn = agg_func
        .then(
            kw_ci("AS")
                .padded()
                .ignore_then(ident.clone().padded())
                .or_not(),
        )
        .map(|(func, alias)| StatFunction { func, alias });

    // Individual pipe stages.
    let where_stage = kw_ci("where")
        .padded()
        .ignore_then(predicate.clone().padded())
        .map(PipeStage::Where);

    let sort_stage = kw_ci("sort")
        .padded()
        .ignore_then(
            sort_expr
                .padded()
                .separated_by(just(',').padded())
                .at_least(1)
                .collect::<Vec<_>>(),
        )
        .map(PipeStage::Sort);

    let head_stage = kw_ci("head")
        .padded()
        .ignore_then(uint.padded())
        .map(PipeStage::Limit);

    let tail_stage = kw_ci("tail")
        .padded()
        .ignore_then(uint.padded())
        .map(PipeStage::Tail);

    let limit_stage = kw_ci("limit")
        .padded()
        .ignore_then(uint.padded())
        .map(PipeStage::Limit);

    // stats stage: multi-aggregate + multi-by-field
    // `stats agg [AS alias] [, agg [AS alias] …] [BY field [, field …]]`
    let stats_stage = kw_ci("stats")
        .padded()
        .ignore_then(
            stat_fn
                .clone()
                .padded()
                .separated_by(just(',').padded())
                .at_least(1)
                .collect::<Vec<_>>(),
        )
        .then(
            kw_ci("by")
                .padded()
                .ignore_then(
                    field_path
                        .clone()
                        .padded()
                        .separated_by(just(',').padded())
                        .at_least(1)
                        .collect::<Vec<_>>(),
                )
                .or_not()
                .map(|o| o.unwrap_or_default()),
        )
        .map(|(aggregates, by_fields)| {
            PipeStage::Stats(StatsStage {
                aggregates,
                by_fields,
            })
        });

    let dedup_stage = kw_ci("dedup")
        .padded()
        .ignore_then(
            field_path
                .padded()
                .separated_by(just(',').padded())
                .at_least(1)
                .collect::<Vec<_>>(),
        )
        .map(PipeStage::Dedup);

    let fields_stage = kw_ci("fields")
        .padded()
        .ignore_then(
            choice((just('+').padded().to(true), just('-').padded().to(false)))
                .or_not()
                .map(|sign| sign.unwrap_or(true)),
        )
        .then(
            field_path
                .clone()
                .padded()
                .separated_by(just(',').padded())
                .at_least(1)
                .collect::<Vec<_>>(),
        )
        .map(|(include, fields)| PipeStage::Fields(FieldsStage { include, fields }));

    // join stage: `join [kind] source ON field [== field]`
    let pipe_join_kind = choice((
        kw_ci("inner").padded().to(JoinKind::Inner),
        kw_ci("left").padded().to(JoinKind::Left),
        kw_ci("right").padded().to(JoinKind::Right),
        kw_ci("full").padded().to(JoinKind::FullOuter),
        kw_ci("cross").padded().to(JoinKind::Cross),
        empty().to(JoinKind::Inner),
    ));

    let join_stage = kw_ci("join")
        .padded()
        .ignore_then(pipe_join_kind)
        .then(source_ref.clone().padded())
        .then_ignore(kw_ci("on").padded())
        .then(field_path.clone().padded())
        .then(
            just("==")
                .padded()
                .ignore_then(field_path.clone().padded())
                .or_not(),
        )
        .map(|(((kind, source), left_field), right_field)| {
            let on = match right_field {
                Some(rf) => JoinCondition::Pair(left_field, rf),
                None => JoinCondition::SameField(left_field),
            };
            PipeStage::Join(JoinStage { kind, source, on })
        });

    let enrich_stage = kw_ci("enrich")
        .padded()
        .ignore_then(ident.padded())
        .then(
            field_path
                .clone()
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
        .map(|(source, stages)| PipeQuery {
            source,
            stages,
            write: None,
        });

    // Variant 2: `source ('|' pipe_stage)+` — bare source with pipe stages
    let bare_source_query = source_ref
        .clone()
        .padded()
        .then(stages_with_pipe.clone())
        .map(|(source, stages)| PipeQuery {
            source,
            stages,
            write: None,
        });

    // Variant 3: `'|' pipe_stage ('|' pipe_stage)*` — no source prefix (EC-11-009)
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
            source: SourceRef::from_raw(""),
            stages,
            write: None,
        });

    choice((from_source_query, no_source_query, bare_source_query))
}
