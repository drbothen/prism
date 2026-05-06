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
use crate::error_recovery::{pipe_boundary_chars, rich_to_parse_error};
use crate::filter_parser::{build_predicate_parser, build_source_ref_parser};
use crate::security;
use crate::write_ast::{WriteArg, WriteNode};
use crate::write_verb_registry::WriteVerbRegistry;

// ── Security re-export for convenient use in tests ────────────────────────────
pub use security::PRISM_MAX_PIPE_STAGES;

/// Parse a pipe-mode query: `[FROM source | source] (| stage)*`.
///
/// Called by `PrismQlParser::parse` after mode detection confirms pipe mode.
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
pub(crate) fn parse_pipe(input: &str) -> Result<PipeQuery, Vec<ParseError>> {
    // When called directly (bypassing PrismQlParser::parse), use env-var limits.
    let limits = security::ParseLimits::snapshot();
    parse_pipe_with_limits(input, &limits)
}

/// Parse a pipe-mode query using the provided snapshotted limits (F-HIGH-001).
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
pub(crate) fn parse_pipe_with_limits(
    input: &str,
    limits: &security::ParseLimits,
) -> Result<PipeQuery, Vec<ParseError>> {
    let parser = build_pipe_parser();
    let (result, errs) = parser.parse(input).into_output_errors();
    if errs.is_empty() {
        if let Some(pq) = result {
            // Security: check pipe stage count (race-free via snapshot).
            limits
                .check_pipe_stage_count_with(&pq.stages)
                .map_err(|e| vec![ParseError::new(0, e.to_string())])?;

            // Security: check AST nesting depth in every pipe stage containing
            // a predicate or expression (race-free via snapshot).
            for stage in &pq.stages {
                check_pipe_stage_depth_with(stage, limits)
                    .map_err(|e| vec![ParseError::new(0, e.to_string())])?;
            }

            // Security: check list item counts in all pipe stages (race-free via snapshot).
            limits
                .check_pipe_list_sizes_with(&pq)
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
/// or expressions using the snapshotted limits.
///
/// (B-2, BC-2.11.006, DI-019, F-HIGH-001)
fn check_pipe_stage_depth_with(
    stage: &crate::ast::PipeStage,
    limits: &security::ParseLimits,
) -> Result<(), prism_core::error::PrismError> {
    use crate::ast::PipeStage;
    match stage {
        PipeStage::Where(pred) => limits.check_predicate_nesting_depth_with(pred, 0),
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
pub(crate) fn build_pipe_parser<'a>(
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
    //
    // SEC-S-001: Produce enum values directly so the downstream match is
    // compile-time exhaustive — no `unreachable!()` needed.
    let generic_agg = choice((
        kw_ci("sum")
            .padded()
            .to(AggFunc::Sum as fn(FieldPath) -> AggFunc),
        kw_ci("avg")
            .padded()
            .to(AggFunc::Avg as fn(FieldPath) -> AggFunc),
        kw_ci("min")
            .padded()
            .to(AggFunc::Min as fn(FieldPath) -> AggFunc),
        kw_ci("max")
            .padded()
            .to(AggFunc::Max as fn(FieldPath) -> AggFunc),
    ))
    .then(
        field_path
            .clone()
            .padded()
            .delimited_by(just('(').padded(), just(')').padded()),
    )
    .map(|(ctor, fp): (fn(FieldPath) -> AggFunc, FieldPath)| ctor(fp));

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

    // Wire up error recovery for malformed pipe stages (CR F-CR-009, AC-9).
    //
    // When a stage fails to parse, `skip_then_retry_until` skips tokens one at a
    // time and retries at the next `|` boundary. This produces a partial AST
    // (stages before the error) plus accumulated errors — callers can inspect
    // both. `pipe_boundary_chars()` returns `&['|']` — the canonical boundary set.
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
    ))
    .recover_with(skip_then_retry_until(
        any().ignored(),
        one_of(pipe_boundary_chars()).ignored(),
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

// ─────────────────────────────────────────────────────────────────────────────
// S-3.06 write-stage extensions (BC-2.11.004)
// ─────────────────────────────────────────────────────────────────────────────

/// Parse a pipe-mode query extended with an optional terminal write stage.
///
/// Grammar extension (BC-2.11.004, S-3.06):
/// ```text
/// pipe_pipeline = source_stage ("|" pipe_stage)* ("|" write_stage)?
/// write_stage   = write_verb (write_arg)*
/// write_verb    = <any verb registered in WriteVerbRegistry>
/// write_arg     = identifier "=" literal
/// ```
///
/// Rules enforced at parse time:
/// - Write stage MUST be the final stage (EC-11-060): returns `E-QUERY-024`.
/// - Unknown terminal identifier: returns `E-QUERY-023` with suggestion list.
/// - Filter mode is a separate parser; write verbs are hard-rejected there.
///
/// # Security perimeter (BC-2.11.006 INV-SEC-PERIMETER-001)
/// This function is `pub(crate)` — callers outside `prism-query` must use
/// `PrismQlParser::parse` exclusively.
///
/// # Implements BC-2.11.004 — Write Parser Extension
pub(crate) fn parse_pipe_with_write(
    input: &str,
    registry: &WriteVerbRegistry,
    limits: &crate::security::ParseLimits,
) -> Result<PipeQuery, Vec<ParseError>> {
    use crate::error::ParseError;

    // Strategy:
    // 1. Parse the query as a normal pipe query to get the base AST + stages.
    // 2. Check if the LAST "|" segment could be a write verb.
    // 3. If it is a write verb: re-parse stripping the write stage, attach WriteNode.
    // 4. If it looks like an identifier but NOT a write verb: E-QUERY-023.
    // 5. If a write verb appears in non-terminal position: E-QUERY-024.
    //
    // The implementation uses a two-pass approach:
    // Pass 1: Try parsing with the write-stage grammar (including write stage).
    // Pass 2: Fall back to standard pipe parsing if no write stage present.

    // Find all pipe segments to analyze positions.
    // Split on `|` outside of string literals to check for write verb placement.
    let segments = split_pipe_segments(input);

    if segments.is_empty() {
        return Err(vec![ParseError::new(0, "E-QUERY-001: empty pipe query")]);
    }

    // Check if any segment (not the last) contains a write verb → E-QUERY-024.
    // We check all segments except the last one.
    if segments.len() > 1 {
        for (pos, seg) in segments[..segments.len() - 1].iter().enumerate() {
            let token = seg
                .split_ascii_whitespace()
                .next()
                .unwrap_or("")
                .to_ascii_lowercase();
            if registry.is_write_verb(&token) {
                return Err(vec![ParseError::write_stage_not_terminal(0, &token, pos)]);
            }
        }
    }

    // Check the last segment — is it a write verb?
    // Safety: segments is non-empty — guarded above by the `segments.is_empty()` check.
    let last_seg = match segments.last() {
        Some(s) => s,
        None => return Err(vec![ParseError::new(0, "E-QUERY-001: empty pipe query")]),
    };
    let last_token = last_seg
        .split_ascii_whitespace()
        .next()
        .unwrap_or("")
        .to_ascii_lowercase();

    let has_write_stage = if !last_token.is_empty() {
        registry.is_write_verb(&last_token)
    } else {
        false
    };

    // Also check: is the last token an identifier (non-keyword) that is NOT a
    // registered pipe stage keyword and NOT a write verb → E-QUERY-023.
    let is_pipe_stage_kw = matches!(
        last_token.as_str(),
        "where"
            | "sort"
            | "head"
            | "tail"
            | "stats"
            | "dedup"
            | "fields"
            | "join"
            | "enrich"
            | "limit"
    );

    // Determine source sensor from first segment for E-QUERY-023 suggestions.
    let source_sensor_for_suggestion = {
        let first_seg = &segments[0];
        // The first segment is either "FROM source" or just "source".
        let tokens: Vec<&str> = first_seg.split_ascii_whitespace().collect();
        let raw_source = if tokens.len() >= 2
            && (tokens[0].eq_ignore_ascii_case("FROM")
                || (segments.len() == 1 && tokens[0].is_empty()))
        {
            tokens.get(1).copied().unwrap_or("")
        } else if !tokens.is_empty() {
            tokens[0]
        } else {
            ""
        };
        extract_sensor_prefix(raw_source)
    };

    // If last token is identifier-shaped, not a known pipe stage keyword, and
    // not empty, and not a write verb → E-QUERY-023.
    // But only if there ARE pipe-style separators (i.e., segments.len() > 1
    // or the input starts with '|'). A single-segment input that is just an
    // identifier might be a valid query of another mode; don't hijack it.
    let is_identifier_shaped = !last_token.is_empty()
        && last_token
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_');

    if !has_write_stage
        && segments.len() > 1
        && is_identifier_shaped
        && !is_pipe_stage_kw
        && !last_token.is_empty()
    {
        // Unknown terminal identifier in pipe position → E-QUERY-023.
        let available: Vec<String> = match &source_sensor_for_suggestion {
            Some(sensor) => registry
                .verbs_for_sensor(sensor)
                .into_iter()
                .map(|s| s.to_string())
                .collect(),
            None => registry.all_verbs().map(|s| s.to_string()).collect(),
        };
        let available_refs: Vec<&str> = available.iter().map(|s| s.as_str()).collect();
        return Err(vec![ParseError::unknown_write_verb(
            0,
            &last_token,
            &available_refs,
        )]);
    }

    if has_write_stage {
        // Strip the last write stage from the input and re-parse the base pipeline.
        // Find the position of the last `|` in the input.
        let last_pipe = find_last_pipe(input);

        let (base_input, write_stage_str) = if let Some(pos) = last_pipe {
            (&input[..pos], input[pos + 1..].trim())
        } else {
            // No pipe at all — the entire input is the write verb.
            // F-PR130-P1-LOW-003: Reachable only via direct `parse_pipe_with_write`
            // calls (test-only). `PrismQlParser::parse_with_registry` does not route
            // bare-identifier inputs to pipe mode (filter mode handles them), so this
            // branch is unreachable from the public API.
            ("", input.trim())
        };

        // Parse the write stage.
        let write_stage_parser = build_write_stage_parser(registry);
        let (write_result, write_errs) = write_stage_parser
            .parse(write_stage_str)
            .into_output_errors();
        if !write_errs.is_empty() {
            let errs: Vec<ParseError> = write_errs
                .iter()
                .map(|e| ParseError::new(0, format!("E-QUERY-001: {e}")))
                .collect();
            return Err(errs);
        }
        let mut write_node = write_result.ok_or_else(|| {
            vec![ParseError::new(
                0,
                "E-QUERY-001: failed to parse write stage",
            )]
        })?;

        // Populate source_sensor from the base pipeline's source.
        let base_query = if base_input.trim().is_empty() {
            // No source prefix — produce a PipeQuery with empty source.
            PipeQuery {
                source: crate::ast::SourceRef::from_raw(""),
                stages: vec![],
                write: None,
            }
        } else {
            // Parse the base pipeline (without write stage).
            // Use caller-provided limits (F-PR130-P1-MED-001 / BC-2.11.006 F-HIGH-001):
            // all guards within a single parse call MUST use the same snapshot.
            parse_pipe_with_limits(base_input, limits)?
        };

        let source_sensor = extract_sensor_prefix(&base_query.source.raw);
        write_node.source_sensor = source_sensor;

        // F-PR130-P1-LOW-002: The terminal write stage counts toward the
        // pipe-stage-count limit (BC-2.11.006 §DI-019). `parse_pipe_with_limits`
        // already checked that read stages <= limit, but it did not know about
        // the write stage. Enforce total count = read_stages + 1 <= limit here.
        let total_stages = base_query.stages.len() + 1;
        let limit = limits.pipe_stages;
        if total_stages > limit {
            return Err(vec![ParseError::new(
                0,
                format!(
                    "E-QUERY-003: pipe stage count {total_stages} (including terminal write stage) \
                     exceeds maximum allowed {limit}"
                ),
            )]);
        }

        Ok(PipeQuery {
            source: base_query.source,
            stages: base_query.stages,
            write: Some(write_node),
        })
    } else {
        // No write stage — parse as standard pipe query.
        // Use caller-provided limits (F-PR130-P1-MED-001 / BC-2.11.006 F-HIGH-001).
        parse_pipe_with_limits(input, limits)
    }
}

/// Split a pipe query string into segments on unquoted `|`.
/// Returns the segments without the `|` separators.
/// The first segment is the source (possibly with FROM prefix).
fn split_pipe_segments(input: &str) -> Vec<String> {
    let mut segments: Vec<String> = Vec::new();
    let mut current = String::new();
    let bytes = input.as_bytes();
    let len = bytes.len();
    let mut in_sq = false;
    let mut in_dq = false;
    let mut i = 0;
    while i < len {
        match bytes[i] {
            b'\'' if !in_dq => {
                in_sq = !in_sq;
                current.push('\'');
            }
            b'"' if !in_sq => {
                in_dq = !in_dq;
                current.push('"');
            }
            b'|' if !in_sq && !in_dq => {
                segments.push(current.trim().to_string());
                current = String::new();
            }
            c => {
                current.push(c as char);
            }
        }
        i += 1;
    }
    let last = current.trim().to_string();
    if !last.is_empty() || !segments.is_empty() {
        segments.push(last);
    }
    segments
}

/// Find the byte offset of the last unquoted `|` in the input.
fn find_last_pipe(input: &str) -> Option<usize> {
    let bytes = input.as_bytes();
    let len = bytes.len();
    let mut in_sq = false;
    let mut in_dq = false;
    let mut last_pipe: Option<usize> = None;
    let mut i = 0;
    while i < len {
        match bytes[i] {
            b'\'' if !in_dq => in_sq = !in_sq,
            b'"' if !in_sq => in_dq = !in_dq,
            b'|' if !in_sq && !in_dq => last_pipe = Some(i),
            _ => {}
        }
        i += 1;
    }
    last_pipe
}

/// Build a Chumsky write-stage parser for a single pipe write stage.
///
/// Returns a parser for:
/// `write_stage = write_verb (write_arg)*`
///
/// Uses `choice()` over a collected iterator of verb parsers (dynamic verb
/// set from `WriteVerbRegistry::all_verbs()`). The verb set is a runtime
/// value, not a compile-time literal — this is intentional (Story dev notes).
///
/// Verb matching is case-insensitive (BC-2.11.004 §INV-WRITE-VERB-CASE-INSENSITIVE).
///
/// # Security perimeter (BC-2.11.006 INV-SEC-PERIMETER-001)
/// `pub(crate)` — never `pub`.
///
/// # Implements BC-2.11.004 — Write Parser Extension
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn build_write_stage_parser<'a>(
    registry: &'a WriteVerbRegistry,
) -> impl Parser<'a, &'a str, WriteNode, extra::Err<Rich<'a, char>>> + Clone + 'a {
    let write_arg = build_write_arg_parser();

    // Collect all registered verbs into a sorted Vec so the parser is deterministic.
    let verbs: Vec<String> = {
        let mut v: Vec<String> = registry.all_verbs().map(|s| s.to_string()).collect();
        v.sort();
        v
    };

    // Build one sub-parser per registered verb (case-insensitive match).
    // Each sub-parser matches the verb string case-insensitively and returns
    // the canonical lowercase verb string.
    //
    // We clone `verbs` into the closure so the parser owns it.
    let ident_char = any::<&str, extra::Err<Rich<char>>>()
        .filter(|c: &char| c.is_ascii_alphanumeric() || *c == '_');

    let verb_parser = ident_char
        .repeated()
        .at_least(1)
        .to_slice()
        .try_map(move |s: &str, span| {
            let lower = s.to_ascii_lowercase();
            if verbs.contains(&lower) {
                Ok(lower)
            } else {
                Err(Rich::custom(
                    span,
                    format!("not a registered write verb: '{s}'"),
                ))
            }
        });

    verb_parser
        .padded()
        .then(write_arg.padded().repeated().collect::<Vec<_>>())
        .map(|(verb, args)| WriteNode {
            verb,
            args,
            source_sensor: None, // populated by parse_pipe_with_write
        })
}

/// Build a Chumsky write-argument parser: `identifier "=" literal`.
///
/// # Security perimeter (BC-2.11.006 INV-SEC-PERIMETER-001)
/// `pub(crate)` — never `pub`.
///
/// # Implements BC-2.11.004 — Write Parser Extension
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn build_write_arg_parser<'a>(
) -> impl Parser<'a, &'a str, WriteArg, extra::Err<Rich<'a, char>>> + Clone {
    use crate::filter_parser::build_literal_parser;

    let ident_char = any::<&str, extra::Err<Rich<char>>>()
        .filter(|c: &char| c.is_ascii_alphanumeric() || *c == '_');
    let key = ident_char
        .repeated()
        .at_least(1)
        .to_slice()
        .map(|s: &str| s.to_string());

    let literal = build_literal_parser();

    key.padded()
        .then_ignore(just('=').padded())
        .then(literal.padded())
        .map(|(key, value)| WriteArg { key, value })
}

#[cfg_attr(not(test), allow(dead_code))]
/// Extract the sensor prefix from a `SourceRef.raw` string.
///
/// Splits on `_` or `.` and returns the first segment, e.g.:
/// - `"crowdstrike_hosts"` → `Some("crowdstrike")`
/// - `"crowdstrike.hosts"` → `Some("crowdstrike")`
/// - `""` or `"hosts"` (no separator) → `None`
/// - `"_internal"` (leading `_`) → `Some("")` (empty string prefix, not None)
///
/// Used to populate `WriteNode.source_sensor` at parse time.
///
/// # Security perimeter (BC-2.11.006 INV-SEC-PERIMETER-001)
/// `pub(crate)` — never `pub`.
///
/// # Implements BC-2.11.004 — Write Parser Extension
pub(crate) fn extract_sensor_prefix(source_raw: &str) -> Option<String> {
    // Split on the first `_` or `.` separator.
    // Find the first occurrence of either separator.
    let underscore = source_raw.find('_');
    let dot = source_raw.find('.');
    let sep_pos = match (underscore, dot) {
        (Some(u), Some(d)) => Some(u.min(d)),
        (Some(u), None) => Some(u),
        (None, Some(d)) => Some(d),
        (None, None) => None,
    };
    sep_pos.map(|pos| source_raw[..pos].to_string())
}
