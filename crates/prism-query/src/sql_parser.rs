//! SQL mode parser: `SELECT … FROM … JOIN … WHERE …` (BC-2.11.003).
//!
//! Grammar (abbreviated):
//!   sql_query   := 'SELECT' select_list 'FROM' source_ref [alias]
//!                  join_clause*
//!                  ['WHERE' predicate]
//!                  ['GROUP BY' expr_list]
//!                  ['HAVING' predicate]
//!                  ['ORDER BY' order_expr_list]
//!                  ['LIMIT' integer]
//!   select_list := '*' | 'DISTINCT' select_item (',' select_item)*
//!   select_item := '*' | 'table.*' | expr ['AS' ident]
//!   join_clause := join_kind 'JOIN' source_ref [alias] 'ON' expr
//!   join_kind   := 'INNER' | 'LEFT' | 'RIGHT' | 'FULL OUTER' | 'CROSS'
//!
//! Mode detection: SQL mode is detected when the input starts with the
//! keyword `SELECT` (case-insensitive).
//!
//! Story: S-3.01 | BC-2.11.003

use chumsky::prelude::*;

use crate::ast::{
    field_path_to_expr, AggFunc, Ast, CompareOp, Expr, FieldPath, FromClause, FuncCall, Join,
    JoinKind, Literal, LogicalOp, OrderExpr, Predicate, ScalarFunc, SelectClause, SelectItem,
    SortDirection, Span, SqlQuery, SqlStatement,
};
use crate::write_ast::{DmlNode, DmlOperation};
// S-3.06: Assignment is used by build_dml_parser (UPDATE SET col=val production)
use crate::error::ParseError;
use crate::error_recovery::{rich_to_parse_error, sql_paren_delimiters};
use crate::filter_parser::{build_literal_parser, build_predicate_parser, build_source_ref_parser};
use crate::security;
#[allow(unused_imports)]
use crate::write_ast::Assignment;

/// SQL keywords that must not be consumed as aliases (canonical uppercase form).
///
/// # Security (B-7, BC-2.11.003)
/// Keyword detection is CASE-INSENSITIVE — `alias_ident` must call
/// `SQL_KEYWORDS.iter().any(|kw| kw.eq_ignore_ascii_case(s))` rather than
/// `SQL_KEYWORDS.contains(&s)`. Storing uppercase-only canonical forms and
/// doing a case-insensitive comparison prevents bypass via titlecase variants
/// like "Where", "Select", "sElEcT", etc.
const SQL_KEYWORDS: &[&str] = &[
    "SELECT",
    "FROM",
    "WHERE",
    "JOIN",
    "INNER",
    "LEFT",
    "RIGHT",
    "FULL",
    "OUTER",
    "CROSS",
    "ON",
    "AS",
    "AND",
    "OR",
    "NOT",
    "IN",
    "LIKE",
    "NULL",
    "TRUE",
    "FALSE",
    "GROUP",
    "BY",
    "HAVING",
    "ORDER",
    "LIMIT",
    "DISTINCT",
    "COUNT",
    "SUM",
    "AVG",
    "MIN",
    "MAX",
    "DISTINCT_COUNT",
    "PERCENTILE",
];

/// Parse a SQL-mode query and return `Ast::Sql(SqlStatement::Select(SqlQuery))`.
///
/// This is the canonical entry point — symmetric with `parse_filter()` (returns
/// `Result<FilterExpr, _>` unwrapped by `Ast::Filter`) and `parse_pipe()` (returns
/// `Result<PipeQuery, _>` unwrapped by `Ast::Pipe`).  Callers that need the inner
/// `SqlQuery` pattern-match: `let Ast::Sql(SqlStatement::Select(sq)) = parse_sql(…)?`.
///
/// `parse_sql_ast` is removed — this function supersedes it.
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
pub(crate) fn parse_sql(input: &str) -> Result<Ast, Vec<ParseError>> {
    // When called directly (bypassing PrismQlParser::parse), use env-var limits.
    let limits = security::ParseLimits::snapshot();
    parse_sql_with_limits(input, &limits)
}

/// Parse a SQL-mode query using the provided snapshotted limits (F-HIGH-001).
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
pub(crate) fn parse_sql_with_limits(
    input: &str,
    limits: &security::ParseLimits,
) -> Result<Ast, Vec<ParseError>> {
    let parser = build_sql_parser();
    let (result, errs) = parser.parse(input).into_output_errors();

    // Happy path: no errors, full AST produced.
    if errs.is_empty() {
        if let Some(sq) = result {
            // Security: check AST nesting depth across WHERE, HAVING, JOIN ON,
            // and ORDER BY expressions (race-free via snapshot).
            limits
                .check_sql_query_nesting_depth_with(&sq, 0)
                .map_err(|e| vec![ParseError::new(0, e.to_string())])?;
            // Security: check list item counts (race-free via snapshot).
            limits
                .check_sql_list_sizes_with(&sq)
                .map_err(|e| vec![ParseError::new(0, e.to_string())])?;
            return Ok(Ast::Sql(SqlStatement::Select(sq)));
        }
    }

    // Recovery path (F-MEDIUM-001, AC-9): Chumsky recovered from a parse error
    // via nested_delimiters and produced a partial AST alongside errors.
    // Return the partial AST so callers can still inspect valid sub-expressions
    // (e.g., outer AND predicates beyond a broken IN subquery).
    // Security checks still apply to the partial AST.
    if let Some(sq) = result {
        let parse_errors: Vec<ParseError> = errs.iter().map(rich_to_parse_error).collect();
        if !parse_errors.is_empty() {
            // Partial AST with recovery errors: validate depth and list sizes
            // before returning. The AST may contain Predicate::RecoveryError
            // sentinels where recovery occurred.
            if limits.check_sql_query_nesting_depth_with(&sq, 0).is_ok()
                && limits.check_sql_list_sizes_with(&sq).is_ok()
            {
                return Ok(Ast::Sql(SqlStatement::Select(sq)));
            }
        }
    }

    let parse_errors: Vec<ParseError> = errs.iter().map(rich_to_parse_error).collect();
    if parse_errors.is_empty() {
        Err(vec![ParseError::new(0, "E-QUERY-001: SQL parse failed")])
    } else {
        Err(parse_errors)
    }
}

/// Build the full SQL-mode parser.
#[allow(clippy::clone_on_copy)]
fn build_sql_parser<'a>() -> impl Parser<'a, &'a str, SqlQuery, extra::Err<Rich<'a, char>>> {
    recursive(|sql_query| {
        let source_ref = build_source_ref_parser();
        let literal = build_literal_parser();

        // Identifier (for aliases, column names, etc.).
        let ident_char = any::<&str, extra::Err<Rich<char>>>()
            .filter(|c: &char| c.is_ascii_alphanumeric() || *c == '_');
        let ident = ident_char
            .repeated()
            .at_least(1)
            .to_slice()
            .map(|s: &str| s.to_string());

        // Non-keyword identifier — for aliases that appear without AS.
        //
        // Keyword rejection is CASE-INSENSITIVE (B-7, BC-2.11.003): "Where",
        // "sElEcT", "WHERE" are all rejected. The SQL_KEYWORDS list stores
        // canonical uppercase forms; we use eq_ignore_ascii_case for matching.
        let alias_ident = ident_char
            .repeated()
            .at_least(1)
            .to_slice()
            .try_map(|s: &str, span| {
                if SQL_KEYWORDS.iter().any(|kw| kw.eq_ignore_ascii_case(s)) {
                    Err(Rich::custom(
                        span,
                        format!("'{s}' is a reserved keyword, not a valid alias"),
                    ))
                } else {
                    Ok(s.to_string())
                }
            });

        // Field path (dotted identifier).
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

        // Build the expression parser for SELECT projections / ORDER BY / GROUP BY / JOIN ON.
        let expr = build_sql_expr_parser(sql_query.clone(), field_path.clone(), literal.clone());

        // Build the predicate parser for WHERE / HAVING (same as filter mode).
        let predicate =
            build_sql_predicate_parser(sql_query.clone(), field_path.clone(), literal.clone());

        // Alias: `AS ident` OR bare non-keyword ident.
        //
        // Both AS-prefixed and bare aliases use `alias_ident` (case-insensitive
        // keyword rejection) so that `SELECT a AS Select FROM t` is rejected
        // in the same way as `SELECT a FROM t Select` (B-7, BC-2.11.003).
        let explicit_alias = text::keyword("AS")
            .or(text::keyword("as"))
            .padded()
            .ignore_then(alias_ident.clone().padded())
            .map(Some);
        let bare_alias = alias_ident.padded().map(Some);
        let alias = explicit_alias.or(bare_alias).or(empty().to(None));

        // SelectItem: `table.*` | `*` | `expr [AS alias]`
        let table_star = ident
            .then_ignore(just(".*").padded())
            .map(SelectItem::TableStar);

        let star = just('*').padded().to(SelectItem::Star);

        let expr_item = expr
            .clone()
            .padded()
            .then(alias.clone())
            .map(|(e, a)| SelectItem::Expr { expr: e, alias: a });

        let select_item = choice((table_star, star, expr_item));

        // SELECT [DISTINCT] item [, item ...]
        let select_clause = text::keyword("SELECT")
            .or(text::keyword("select"))
            .padded()
            .ignore_then(
                text::keyword("DISTINCT")
                    .or(text::keyword("distinct"))
                    .padded()
                    .to(true)
                    .or_not()
                    .map(|d| d.unwrap_or(false)),
            )
            .then(
                select_item
                    .separated_by(just(',').padded())
                    .at_least(1)
                    .collect::<Vec<_>>(),
            )
            .map(|(distinct, items)| SelectClause { distinct, items });

        // FROM source_ref [alias]
        let from_clause = text::keyword("FROM")
            .or(text::keyword("from"))
            .padded()
            .ignore_then(source_ref.clone().padded())
            .then(alias.clone())
            .map(|(source, alias)| FromClause { source, alias });

        // JOIN kind — includes CROSS
        let join_kind = choice((
            text::keyword("FULL")
                .or(text::keyword("full"))
                .padded()
                .ignore_then(
                    text::keyword("OUTER")
                        .or(text::keyword("outer"))
                        .padded()
                        .or_not(),
                )
                .to(JoinKind::FullOuter),
            text::keyword("INNER")
                .or(text::keyword("inner"))
                .padded()
                .to(JoinKind::Inner),
            text::keyword("LEFT")
                .or(text::keyword("left"))
                .padded()
                .to(JoinKind::Left),
            text::keyword("RIGHT")
                .or(text::keyword("right"))
                .padded()
                .to(JoinKind::Right),
            text::keyword("CROSS")
                .or(text::keyword("cross"))
                .padded()
                .to(JoinKind::Cross),
            // Plain `JOIN` with no modifier = INNER
            empty().to(JoinKind::Inner),
        ));

        // JOIN clause: `[kind] JOIN source_ref [alias] ON expr`
        let join_clause = join_kind
            .then_ignore(text::keyword("JOIN").or(text::keyword("join")).padded())
            .then(source_ref.clone().padded())
            .then(alias.clone())
            .then_ignore(text::keyword("ON").or(text::keyword("on")).padded())
            .then(expr.clone().padded())
            .map(|(((kind, source), alias), on)| Join {
                kind,
                source,
                alias,
                on,
            });

        // WHERE clause
        let where_clause = text::keyword("WHERE")
            .or(text::keyword("where"))
            .padded()
            .ignore_then(predicate.clone().padded())
            .or_not();

        // GROUP BY clause
        let group_by_clause = text::keyword("GROUP")
            .or(text::keyword("group"))
            .padded()
            .ignore_then(text::keyword("BY").or(text::keyword("by")).padded())
            .ignore_then(
                expr.clone()
                    .padded()
                    .separated_by(just(',').padded())
                    .at_least(1)
                    .collect::<Vec<_>>(),
            )
            .or_not()
            .map(|g| g.unwrap_or_default());

        // HAVING clause
        let having_clause = text::keyword("HAVING")
            .or(text::keyword("having"))
            .padded()
            .ignore_then(predicate.clone().padded())
            .or_not();

        // ORDER BY clause
        let order_direction = choice((
            text::keyword("DESC")
                .or(text::keyword("desc"))
                .padded()
                .to(SortDirection::Desc),
            text::keyword("ASC")
                .or(text::keyword("asc"))
                .padded()
                .to(SortDirection::Asc),
        ))
        .or_not()
        .map(|d| d.unwrap_or(SortDirection::Asc));

        let order_expr = expr
            .clone()
            .padded()
            .then(order_direction)
            .map(|(expr, direction)| OrderExpr { expr, direction });

        let order_by_clause = text::keyword("ORDER")
            .or(text::keyword("order"))
            .padded()
            .ignore_then(text::keyword("BY").or(text::keyword("by")).padded())
            .ignore_then(
                order_expr
                    .separated_by(just(',').padded())
                    .at_least(1)
                    .collect::<Vec<_>>(),
            )
            .or_not()
            .map(|o| o.unwrap_or_default());

        // LIMIT clause
        let limit_clause = text::keyword("LIMIT")
            .or(text::keyword("limit"))
            .padded()
            .ignore_then(text::int(10).to_slice().try_map(|s: &str, span| {
                s.parse::<u64>()
                    .map_err(|e| Rich::custom(span, format!("invalid LIMIT value: {e}")))
            }))
            .or_not();

        // Full SQL query.
        select_clause
            .then(from_clause)
            .then(join_clause.repeated().collect::<Vec<_>>())
            .then(where_clause)
            .then(group_by_clause)
            .then(having_clause)
            .then(order_by_clause)
            .then(limit_clause)
            .map(
                |(((((((select, from), joins), where_), group_by), having), order_by), limit)| {
                    SqlQuery {
                        select,
                        from,
                        joins,
                        where_,
                        group_by,
                        having,
                        order_by,
                        limit,
                    }
                },
            )
    })
}

/// Build a SQL predicate parser for WHERE / HAVING clauses.
///
/// Extends the base predicate parser with `IN (SELECT ...)` subquery support.
fn build_sql_predicate_parser<'a>(
    sql_query: impl Parser<'a, &'a str, SqlQuery, extra::Err<Rich<'a, char>>> + Clone + 'a,
    field_path: impl Parser<'a, &'a str, FieldPath, extra::Err<Rich<'a, char>>> + Clone + 'a,
    _literal: impl Parser<'a, &'a str, Literal, extra::Err<Rich<'a, char>>> + Clone + 'a,
) -> impl Parser<'a, &'a str, Predicate, extra::Err<Rich<'a, char>>> + Clone {
    // For WHERE / HAVING, we delegate to the base predicate parser.
    // Subquery in WHERE (field IN (SELECT ...)) is rare; for now we use
    // the filter-mode predicate parser and handle IN subquery at the
    // predicate level via a separate arm.
    //
    // The build_predicate_parser already handles all filter operators.
    // SQL-specific extensions (IN subquery) are added here.
    let base = build_predicate_parser();

    // sql_paren_delimiters() returns ('(', ')') — the canonical delimiter pair
    // for SQL subquery recovery (CR F-CR-009). Used here to document the pairing
    // between the recovery helper and the actual delimited_by call below.
    let (open_paren, close_paren) = sql_paren_delimiters();

    // IN subquery: `field IN (SELECT ...)` / `field NOT IN (SELECT ...)`
    //
    // The subquery arm is extended with `nested_delimiters` recovery (F-MEDIUM-001,
    // AC-9, BC-2.11.003): when the content inside `IN (...)` cannot be parsed as a
    // valid SQL subquery, the recovery combinator skips the entire parenthesised
    // region and inserts `Predicate::RecoveryError` as a sentinel. This allows the
    // parser to continue past the broken subquery and still parse the outer AND/OR
    // predicates, producing a partial AST.
    let in_subquery = field_path
        .clone()
        .padded()
        .then(
            text::keyword("NOT")
                .or(text::keyword("not"))
                .padded()
                .to(true)
                .or_not()
                .map(|n| n.unwrap_or(false)),
        )
        .then_ignore(choice((text::keyword("IN"), text::keyword("in"))).padded())
        .then(
            sql_query
                .clone()
                .padded()
                .delimited_by(just(open_paren).padded(), just(close_paren).padded())
                // F-MEDIUM-001: recovery for malformed IN subquery bodies.
                // nested_delimiters matches `(... any content ...)` and returns the
                // fallback when the inner content fails to parse as a SqlQuery.
                .recover_with(via_parser(nested_delimiters(
                    open_paren,
                    close_paren,
                    [],
                    |_span| SqlQuery::recovery_sentinel(),
                ))),
        )
        .map(|((fp, negated), sq)| {
            // If recovery produced the sentinel, emit RecoveryError for this arm.
            if sq.is_recovery_sentinel() {
                Predicate::RecoveryError
            } else {
                Predicate::InSubquery {
                    field: fp,
                    subquery: Box::new(sq),
                    negated,
                }
            }
        });

    // Prefer IN subquery over base (which handles IN list).
    in_subquery.or(base)
}

/// Build an expression parser extended with SQL aggregate functions,
/// `IN (SELECT ...)` subquery, and `func(*)` syntax.
#[allow(clippy::clone_on_copy)]
fn build_sql_expr_parser<'a>(
    sql_query: impl Parser<'a, &'a str, SqlQuery, extra::Err<Rich<'a, char>>> + Clone + 'a,
    field_path: impl Parser<'a, &'a str, FieldPath, extra::Err<Rich<'a, char>>> + Clone + 'a,
    literal: impl Parser<'a, &'a str, Literal, extra::Err<Rich<'a, char>>> + Clone + 'a,
) -> impl Parser<'a, &'a str, Expr, extra::Err<Rich<'a, char>>> + Clone {
    let ident_char = any::<&str, extra::Err<Rich<char>>>()
        .filter(|c: &char| c.is_ascii_alphanumeric() || *c == '_');
    let ident = ident_char
        .repeated()
        .at_least(1)
        .to_slice()
        .map(|s: &str| s.to_string());

    recursive(move |expr| {
        // Compare operator.
        let compare_op = choice((
            just(">=").to(CompareOp::Ge),
            just("<=").to(CompareOp::Le),
            just("!=").to(CompareOp::Ne),
            just("==").to(CompareOp::Eq),
            just('>').to(CompareOp::Gt),
            just('<').to(CompareOp::Lt),
            just('=').to(CompareOp::Eq),
            text::keyword("LIKE").to(CompareOp::Like),
            text::keyword("like").to(CompareOp::Like),
        ))
        .padded();

        // IN subquery: `field IN (SELECT ...)`
        let in_subquery = field_path
            .clone()
            .padded()
            .then_ignore(choice((text::keyword("IN"), text::keyword("in"))).padded())
            .then(
                sql_query
                    .clone()
                    .padded()
                    .delimited_by(just('(').padded(), just(')').padded()),
            )
            .map(|(fp, sq)| Expr::InSubquery {
                field: fp,
                subquery: Box::new(sq),
            });

        // IN list: `field IN (literal, ...)`
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

        // --- Aggregate function calls (emit FuncCall::Aggregate) ---
        // PERCENTILE(field, p)
        let percentile_agg = text::keyword("PERCENTILE")
            .or(text::keyword("percentile"))
            .padded()
            .ignore_then(
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
                        use ordered_float::OrderedFloat;
                        Ok(Expr::FuncCall(FuncCall::Aggregate {
                            func: AggFunc::Percentile {
                                field: fp,
                                p: OrderedFloat(p),
                            },
                            args: vec![],
                            distinct: false,
                        }))
                    })
                    .delimited_by(just('(').padded(), just(')').padded()),
            );

        // DISTINCT_COUNT(field)
        let distinct_count_agg = text::keyword("DISTINCT_COUNT")
            .or(text::keyword("distinct_count"))
            .padded()
            .ignore_then(
                field_path
                    .clone()
                    .padded()
                    .map(|fp| {
                        Expr::FuncCall(FuncCall::Aggregate {
                            func: AggFunc::DistinctCount(fp.clone()),
                            args: vec![field_path_to_expr(fp)],
                            distinct: false,
                        })
                    })
                    .delimited_by(just('(').padded(), just(')').padded()),
            );

        // count(*) → AggFunc::Count, count(field) → AggFunc::CountField
        let count_agg = text::keyword("COUNT")
            .or(text::keyword("count"))
            .padded()
            .ignore_then(
                choice((
                    just('*').padded().to(Expr::FuncCall(FuncCall::Aggregate {
                        func: AggFunc::Count,
                        args: vec![Expr::Star],
                        distinct: false,
                    })),
                    field_path.clone().padded().map(|fp| {
                        Expr::FuncCall(FuncCall::Aggregate {
                            func: AggFunc::CountField(fp.clone()),
                            args: vec![field_path_to_expr(fp)],
                            distinct: false,
                        })
                    }),
                    empty().to(Expr::FuncCall(FuncCall::Aggregate {
                        func: AggFunc::Count,
                        args: vec![],
                        distinct: false,
                    })),
                ))
                .delimited_by(just('(').padded(), just(')').padded()),
            );

        // Generic aggregate: SUM / AVG / MIN / MAX
        //
        // SEC-S-001: Produce enum constructors directly so the downstream match
        // is compile-time exhaustive — no `unreachable!()` needed.
        let generic_agg = choice((
            text::keyword("SUM")
                .or(text::keyword("sum"))
                .padded()
                .to(AggFunc::Sum as fn(FieldPath) -> AggFunc),
            text::keyword("AVG")
                .or(text::keyword("avg"))
                .padded()
                .to(AggFunc::Avg as fn(FieldPath) -> AggFunc),
            text::keyword("MIN")
                .or(text::keyword("min"))
                .padded()
                .to(AggFunc::Min as fn(FieldPath) -> AggFunc),
            text::keyword("MAX")
                .or(text::keyword("max"))
                .padded()
                .to(AggFunc::Max as fn(FieldPath) -> AggFunc),
        ))
        .then(
            field_path
                .clone()
                .padded()
                .delimited_by(just('(').padded(), just(')').padded()),
        )
        .map(|(ctor, fp): (fn(FieldPath) -> AggFunc, FieldPath)| {
            let func = ctor(fp.clone());
            Expr::FuncCall(FuncCall::Aggregate {
                func,
                args: vec![field_path_to_expr(fp)],
                distinct: false,
            })
        });

        // --- Scalar function calls (registered UDFs) ---
        let known_scalar = ident.clone().padded().try_map(|name: String, _span| {
            let func = match name.to_lowercase().as_str() {
                "subnet_contains" => ScalarFunc::SubnetContains,
                "time_window" => ScalarFunc::TimeWindow,
                "json_extract_string" => ScalarFunc::JsonExtractString,
                "ioc_match" => ScalarFunc::IocMatch,
                "mitre_tactic" => ScalarFunc::MitreTactic,
                "severity_label" => ScalarFunc::SeverityLabel,
                _ => ScalarFunc::Unknown(name),
            };
            Ok(func)
        });

        let scalar_call = known_scalar
            .then(
                expr.clone()
                    .padded()
                    .separated_by(just(',').padded())
                    .collect::<Vec<_>>()
                    .delimited_by(just('(').padded(), just(')').padded()),
            )
            .map(|(func, args)| Expr::FuncCall(FuncCall::Scalar { func, args }));

        // Basic comparison (field vs literal).
        // Virtual-field promotion: _sensor/_client/etc. become Expr::VirtualField.
        let comparison = field_path
            .clone()
            .padded()
            .then(compare_op.clone())
            .then(literal.clone().padded().map(Expr::Literal))
            .map(|((fp, op), rhs)| Expr::Compare {
                lhs: Box::new(field_path_to_expr(fp)),
                op,
                rhs: Box::new(rhs),
            });

        // field = field comparisons (JOIN ON conditions).
        // Virtual-field promotion applies to both sides.
        let field_comparison = field_path
            .clone()
            .padded()
            .then(compare_op)
            .then(field_path.clone().padded().map(field_path_to_expr))
            .map(|((fp, op), rhs)| Expr::Compare {
                lhs: Box::new(field_path_to_expr(fp)),
                op,
                rhs: Box::new(rhs),
            });

        // Atom — order matters.
        let atom = choice((
            expr.clone()
                .padded()
                .delimited_by(just('(').padded(), just(')').padded()),
            in_subquery,
            in_list,
            percentile_agg,
            distinct_count_agg,
            count_agg,
            generic_agg,
            scalar_call,
            field_comparison,
            comparison,
            literal.clone().padded().map(Expr::Literal),
            field_path.clone().padded().map(field_path_to_expr),
        ));

        // NOT.
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

        // AND.
        let and_expr = not_expr.clone().foldl(
            choice((text::keyword("AND").padded(), text::keyword("and").padded()))
                .ignore_then(not_expr)
                .repeated(),
            |lhs, rhs| Expr::Logical {
                lhs: Box::new(lhs),
                op: LogicalOp::And,
                rhs: Box::new(rhs),
            },
        );

        // OR.
        and_expr.clone().foldl(
            choice((text::keyword("OR").padded(), text::keyword("or").padded()))
                .ignore_then(and_expr)
                .repeated(),
            |lhs, rhs| Expr::Logical {
                lhs: Box::new(lhs),
                op: LogicalOp::Or,
                rhs: Box::new(rhs),
            },
        )
    })
}

// ─────────────────────────────────────────────────────────────────────────────
// S-3.06 DML parser extensions (BC-2.11.004)
// ─────────────────────────────────────────────────────────────────────────────

/// Parse a SQL-mode DML statement, returning `Ast::Sql(SqlStatement::Dml(DmlNode))`.
///
/// Accepts:
/// - `INSERT INTO table_name (col_list) SELECT …`
/// - `UPDATE table_name SET col = val [, col = val]* WHERE expr`
/// - `DELETE FROM table_name WHERE expr`
///
/// Parse-time validation (BC-2.11.004):
/// - `prism_*` target tables → `E-QUERY-010` ("Internal Prism table is write-protected")
/// - `UPDATE`/`DELETE` without WHERE → `E-QUERY-022` (unbounded write)
/// - `INSERT INTO … SELECT` without LIMIT or WHERE → `E-QUERY-022`
///
/// # Security perimeter (BC-2.11.006 INV-SEC-PERIMETER-001)
/// This function is `pub(crate)` — never `pub`.
///
/// # Implements BC-2.11.004 — Write Parser Extension
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn parse_sql_dml(input: &str) -> Result<Ast, Vec<ParseError>> {
    // Dispatch to the correct sub-parser based on the first token.
    // This avoids Chumsky choice() error priority issues when try_map
    // fires after consuming the entire input but choice() still picks
    // the first alternative's error (BC-2.11.004, S-3.06 fix).
    let first_token = input
        .trim()
        .split_ascii_whitespace()
        .next()
        .unwrap_or("")
        .to_ascii_uppercase();
    let node_result: Result<DmlNode, Vec<ParseError>> = match first_token.as_str() {
        "DELETE" => run_dml_parser(build_delete_parser(), input, "DELETE"),
        "UPDATE" => run_dml_parser(build_update_parser(), input, "UPDATE"),
        "INSERT" => run_dml_parser(build_insert_parser(), input, "INSERT"),
        _ => Err(vec![ParseError::new(
            0,
            format!("E-QUERY-001: unrecognized DML keyword '{first_token}'"),
        )]),
    };
    match node_result {
        Ok(node) => Ok(Ast::Sql(SqlStatement::Dml(node))),
        Err(errs) => Err(errs),
    }
}

/// Run a DML sub-parser on `input` and convert the result to `Result<DmlNode, Vec<ParseError>>`.
///
/// This helper exists to avoid a complex `Box<dyn Fn>` type in `parse_sql_dml`
/// (clippy::type_complexity). Each DML operation has a dedicated builder called here.
fn run_dml_parser<'a, P>(
    parser: P,
    input: &'a str,
    op: &'static str,
) -> Result<DmlNode, Vec<ParseError>>
where
    P: Parser<'a, &'a str, DmlNode, extra::Err<Rich<'a, char>>>,
{
    let (result, errs) = parser.parse(input).into_output_errors();
    if errs.is_empty() {
        if let Some(node) = result {
            return Ok(node);
        }
    }
    let parse_errors: Vec<ParseError> = errs.iter().map(rich_to_parse_error).collect();
    Err(if parse_errors.is_empty() {
        vec![ParseError::new(
            0,
            format!("E-QUERY-001: {op} parse failed"),
        )]
    } else {
        parse_errors
    })
}

/// Build the Chumsky DML parser (composite — tries DELETE, UPDATE, INSERT in order).
///
/// NOTE: In practice, `parse_sql_dml` dispatches directly to the per-operation
/// builders (`build_delete_parser`, `build_update_parser`, `build_insert_parser`)
/// based on the first token, avoiding Chumsky `choice()` error-priority issues
/// where `try_map` errors after full-input consumption lose to the first
/// alternative's position-0 failure.
///
/// This function is kept for compatibility with external test callers that
/// may call `build_dml_parser()` directly; internally `parse_sql_dml` does
/// not use it.
///
/// # Security perimeter (BC-2.11.006 INV-SEC-PERIMETER-001)
/// `pub(crate)` — never `pub`.
///
/// # Implements BC-2.11.004 — Write Parser Extension
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn build_dml_parser<'a>() -> impl Parser<'a, &'a str, DmlNode, extra::Err<Rich<'a, char>>>
{
    // Keep choice for the public build_dml_parser API; parse_sql_dml dispatches
    // per-token to avoid choice() error-priority pathology.
    choice((
        build_delete_parser(),
        build_update_parser(),
        build_insert_parser(),
    ))
}

/// Build a Chumsky parser for `DELETE FROM table [WHERE pred]`.
///
/// Security checks (prism_* table guard, unbounded-write guard) run inside
/// `.try_map()`. Called directly by `parse_sql_dml` to avoid `choice()` error
/// priority issues (BC-2.11.004 S-3.06 fix).
///
/// # Security perimeter (BC-2.11.006 INV-SEC-PERIMETER-001)
/// `pub(crate)` — never `pub`.
#[cfg_attr(not(test), allow(dead_code))]
#[allow(clippy::type_complexity)]
fn build_delete_parser<'a>() -> impl Parser<'a, &'a str, DmlNode, extra::Err<Rich<'a, char>>> {
    let predicate = build_predicate_parser();
    let ident_char = any::<&str, extra::Err<Rich<char>>>()
        .filter(|c: &char| c.is_ascii_alphanumeric() || *c == '_');
    let ident = ident_char
        .repeated()
        .at_least(1)
        .to_slice()
        .map(|s: &str| s.to_string());
    let kw_ci = move |k: &'static str| {
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
    let where_present = kw_ci("WHERE")
        .padded()
        .ignore_then(predicate.padded())
        .or_not()
        .map(|p| p.map(|_| crate::ast::Expr::Literal(crate::ast::Literal::Bool(true))));

    kw_ci("DELETE")
        .padded()
        .ignore_then(kw_ci("FROM").padded())
        .ignore_then(ident.padded())
        .then(where_present)
        .try_map(|(table, filter), span| {
            if is_internal_prism_table(&table) {
                return Err(Rich::custom(
                    span,
                    format!(
                        "E-QUERY-010: Internal Prism table '{table}' is write-protected; \
                         use the dedicated MCP tool for this operation"
                    ),
                ));
            }
            let node = DmlNode {
                operation: DmlOperation::Delete,
                target_table: table,
                assignments: vec![],
                filter,
                source_select: None,
            };
            if let Some(e) = check_unbounded_write(&node, 0) {
                return Err(Rich::custom(span, e.message));
            }
            Ok(node)
        })
}

/// Build a Chumsky parser for `UPDATE table SET col=val [, col=val]* [WHERE pred]`.
///
/// Security checks (prism_* table guard, unbounded-write guard) run inside
/// `.try_map()`. Called directly by `parse_sql_dml` to avoid `choice()` error
/// priority issues.
///
/// # Security perimeter (BC-2.11.006 INV-SEC-PERIMETER-001)
/// `pub(crate)` — never `pub`.
#[cfg_attr(not(test), allow(dead_code))]
#[allow(clippy::type_complexity, clippy::clone_on_copy)]
fn build_update_parser<'a>() -> impl Parser<'a, &'a str, DmlNode, extra::Err<Rich<'a, char>>> {
    let literal = build_literal_parser();
    let predicate = build_predicate_parser();
    let ident_char = any::<&str, extra::Err<Rich<char>>>()
        .filter(|c: &char| c.is_ascii_alphanumeric() || *c == '_');
    let ident = ident_char
        .repeated()
        .at_least(1)
        .to_slice()
        .map(|s: &str| s.to_string());
    let kw_ci = move |k: &'static str| {
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
    let where_present = kw_ci("WHERE")
        .padded()
        .ignore_then(predicate.padded())
        .or_not()
        .map(|p| p.map(|_| crate::ast::Expr::Literal(crate::ast::Literal::Bool(true))));

    let assign_value = choice((
        literal.map(crate::ast::Expr::Literal),
        ident.clone().padded().map(|s| {
            use crate::ast::{FieldPath, Span};
            crate::ast::Expr::Field(FieldPath {
                segments: vec![s],
                span: Span::ZERO,
            })
        }),
    ));
    let assignment = ident
        .clone()
        .padded()
        .then_ignore(just('=').padded())
        .then(assign_value.padded())
        .map(|(column, value)| crate::write_ast::Assignment { column, value });

    kw_ci("UPDATE")
        .padded()
        .ignore_then(ident.padded())
        .then_ignore(kw_ci("SET").padded())
        .then(
            assignment
                .padded()
                .separated_by(just(',').padded())
                .at_least(1)
                .collect::<Vec<_>>(),
        )
        .then(where_present)
        .try_map(|((table, assignments), filter), span| {
            if is_internal_prism_table(&table) {
                return Err(Rich::custom(
                    span,
                    format!(
                        "E-QUERY-010: Internal Prism table '{table}' is write-protected; \
                         use the dedicated MCP tool for this operation"
                    ),
                ));
            }
            let node = DmlNode {
                operation: DmlOperation::Update,
                target_table: table,
                assignments,
                filter,
                source_select: None,
            };
            if let Some(e) = check_unbounded_write(&node, 0) {
                return Err(Rich::custom(span, e.message));
            }
            Ok(node)
        })
}

/// Build a Chumsky parser for `INSERT INTO table [(col_list)] SELECT ...`.
///
/// Security checks (prism_* table guard, unbounded-write guard) run inside
/// `.try_map()`. Called directly by `parse_sql_dml` to avoid `choice()` error
/// priority issues.
///
/// # Security perimeter (BC-2.11.006 INV-SEC-PERIMETER-001)
/// `pub(crate)` — never `pub`.
#[cfg_attr(not(test), allow(dead_code))]
#[allow(clippy::type_complexity, clippy::clone_on_copy)]
fn build_insert_parser<'a>() -> impl Parser<'a, &'a str, DmlNode, extra::Err<Rich<'a, char>>> {
    let ident_char = any::<&str, extra::Err<Rich<char>>>()
        .filter(|c: &char| c.is_ascii_alphanumeric() || *c == '_');
    let ident = ident_char
        .repeated()
        .at_least(1)
        .to_slice()
        .map(|s: &str| s.to_string());
    let kw_ci = move |k: &'static str| {
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

    kw_ci("INSERT")
        .padded()
        .ignore_then(kw_ci("INTO").padded())
        .ignore_then(ident.clone().padded())
        .then(
            ident
                .clone()
                .padded()
                .separated_by(just(',').padded())
                .at_least(1)
                .collect::<Vec<_>>()
                .delimited_by(just('(').padded(), just(')').padded())
                .or_not(),
        )
        .then(build_sql_parser())
        .try_map(|((table, _cols), sq), span| {
            if is_internal_prism_table(&table) {
                return Err(Rich::custom(
                    span,
                    format!(
                        "E-QUERY-010: Internal Prism table '{table}' is write-protected; \
                         use the dedicated MCP tool for this operation"
                    ),
                ));
            }
            let node = DmlNode {
                operation: DmlOperation::InsertInto,
                target_table: table,
                assignments: vec![],
                filter: None,
                source_select: Some(sq),
            };
            if let Some(e) = check_unbounded_write(&node, 0) {
                return Err(Rich::custom(span, e.message));
            }
            Ok(node)
        })
}

#[cfg_attr(not(test), allow(dead_code))]
/// Check whether a target table name begins with the `prism_` prefix.
///
/// Returns `true` if the table is an internal Prism table (write-protected).
/// Used by `build_dml_parser` to emit `E-QUERY-010` at parse time.
///
/// # Security perimeter (BC-2.11.006 INV-SEC-PERIMETER-001)
/// `pub(crate)` — never `pub`.
///
/// # Implements BC-2.11.004 — Write Parser Extension
pub(crate) fn is_internal_prism_table(table_name: &str) -> bool {
    table_name.to_ascii_lowercase().starts_with("prism_")
}

#[cfg_attr(not(test), allow(dead_code))]
/// Check whether a `DmlNode` would perform an unbounded write.
///
/// A write is unbounded when:
/// - `UPDATE` or `DELETE FROM` has no WHERE clause.
/// - `INSERT INTO … SELECT` has no LIMIT and no WHERE on the source SELECT.
///
/// Returns `Some(ParseError::unbounded_write(...))` if unbounded; `None` if safe.
///
/// Used inside `build_dml_parser` combinators to emit `E-QUERY-022`.
///
/// # Security perimeter (BC-2.11.006 INV-SEC-PERIMETER-001)
/// `pub(crate)` — never `pub`.
///
/// # Implements BC-2.11.004 — Write Parser Extension
pub(crate) fn check_unbounded_write(node: &DmlNode, offset: usize) -> Option<ParseError> {
    use crate::write_ast::DmlOperation;
    match node.operation {
        DmlOperation::Delete | DmlOperation::Update => {
            if node.filter.is_none() {
                let op = match node.operation {
                    DmlOperation::Delete => "DELETE",
                    DmlOperation::Update => "UPDATE",
                    _ => "DML",
                };
                Some(ParseError::unbounded_write(offset, op))
            } else {
                None
            }
        }
        DmlOperation::InsertInto => {
            // INSERT INTO ... SELECT is unbounded if source SELECT has no WHERE and no LIMIT.
            if let Some(ref sq) = node.source_select {
                if sq.where_.is_none() && sq.limit.is_none() {
                    Some(ParseError::unbounded_write(offset, "INSERT INTO...SELECT"))
                } else {
                    None
                }
            } else {
                // INSERT without a SELECT sub-query: no unbounded check needed
                // (would be a malformed INSERT caught earlier).
                None
            }
        }
    }
}
