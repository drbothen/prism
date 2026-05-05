//! SQL mode parser: `SELECT … FROM … JOIN … WHERE …` (BC-2.11.003).
//!
//! Grammar (abbreviated):
//!   sql_query   := 'SELECT' select_list 'FROM' source_ref [alias]
//!                  join_clause*
//!                  ['WHERE' expr]
//!                  ['GROUP BY' expr_list]
//!                  ['HAVING' expr]
//!                  ['ORDER BY' order_expr_list]
//!                  ['LIMIT' integer]
//!   select_list := '*' | 'DISTINCT' select_item (',' select_item)*
//!   select_item := '*' | 'table.*' | expr ['AS' ident]
//!   join_clause := join_kind 'JOIN' source_ref [alias] 'ON' expr
//!   join_kind   := 'INNER' | 'LEFT' | 'RIGHT' | 'FULL OUTER'
//!
//! Mode detection: SQL mode is detected when the input starts with the
//! keyword `SELECT` (case-insensitive).
//!
//! Subqueries: `WHERE field IN (SELECT …)` is supported via recursive
//! parser construction. Chumsky `recursive()` is used to handle this.
//!
//! Story: S-3.01 | BC-2.11.003

use chumsky::prelude::*;

use crate::ast::{
    CompareOp, Expr, FieldPath, FromClause, Join, JoinKind, Literal, LogicalOp, OrderExpr,
    SelectClause, SelectItem, SortDirection, SqlQuery,
};
use crate::error::ParseError;
use crate::error_recovery::rich_to_parse_error;
use crate::filter_parser::{build_literal_parser, build_source_ref_parser};

/// SQL keywords that must not be consumed as aliases.
const SQL_KEYWORDS: &[&str] = &[
    "SELECT", "select", "FROM", "from", "WHERE", "where", "JOIN", "join", "INNER", "inner", "LEFT",
    "left", "RIGHT", "right", "FULL", "full", "OUTER", "outer", "ON", "on", "AS", "as", "AND",
    "and", "OR", "or", "NOT", "not", "IN", "in", "LIKE", "like", "NULL", "null", "TRUE", "true",
    "FALSE", "false", "GROUP", "group", "BY", "by", "HAVING", "having", "ORDER", "order", "LIMIT",
    "limit", "DISTINCT", "distinct",
];

/// Parse a SQL-mode query.
///
/// Called by `PrismQlParser::parse` after mode detection confirms the input
/// starts with `SELECT`.
///
/// # Errors
/// Returns accumulated `ParseError`s on failure. `nested_delimiters`
/// recovery is used inside parenthesized subexpressions.
pub fn parse_sql(input: &str) -> Result<SqlQuery, Vec<ParseError>> {
    let parser = build_sql_parser();
    let (result, errs) = parser.parse(input).into_output_errors();
    if errs.is_empty() {
        if let Some(sq) = result {
            return Ok(sq);
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
fn build_sql_parser<'a>() -> impl Parser<'a, &'a str, SqlQuery, extra::Err<Rich<'a, char>>> {
    // We use recursive to handle subqueries in WHERE IN (SELECT ...).
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
        let alias_ident = ident_char
            .repeated()
            .at_least(1)
            .to_slice()
            .try_map(|s: &str, span| {
                if SQL_KEYWORDS.contains(&s) {
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
            .map(|segs: Vec<&str>| FieldPath {
                segments: segs.into_iter().map(|s| s.to_string()).collect(),
            });

        // Build the expression parser.
        let expr = build_sql_expr_parser(sql_query.clone(), field_path, literal.clone());

        // Alias: `AS ident` OR bare non-keyword ident (for table aliases like `a`, `b`).
        let explicit_alias = text::keyword("AS")
            .or(text::keyword("as"))
            .padded()
            .ignore_then(ident.padded())
            .map(Some);
        let bare_alias = alias_ident.padded().map(Some);
        let alias = explicit_alias.or(bare_alias).or(empty().to(None));

        // SelectItem: `table.*` | `*` | `func(*)` | `expr [AS alias]`
        // `table.*` — e.g. `a.*`
        let table_star = ident
            .then_ignore(just(".*").padded())
            .map(SelectItem::TableStar);

        // `*` (Star)
        let star = just('*').padded().to(SelectItem::Star);

        // `func(*)` and `func(field)` — aggregate function calls in SELECT.
        // Handled by the expression parser which emits Expr::FuncCall { name, args }.
        // Here they fall through to expr_item (SelectItem::Expr).

        // `expr [AS alias]`
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

        // JOIN kind
        let join_kind = choice((
            text::keyword("FULL")
                .or(text::keyword("full"))
                .padded()
                .ignore_then(text::keyword("OUTER").or(text::keyword("outer")).padded())
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
            .ignore_then(expr.clone().padded())
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
            .ignore_then(expr.clone().padded())
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

/// Build an expression parser extended with SQL-specific `IN (SELECT ...)` subquery
/// and function call syntax like `count(*)`.
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

        // Function call syntax: `func(*)` and `func(field)`.
        //
        // These emit the proper `Expr::FuncCall` variant so downstream stories can
        // distinguish "the column named 'count'" from "the count() aggregate":
        //   - S-3.04: aliased function calls (`count(*) AS total`)
        //   - S-3.07: aggregate evaluation (GROUP BY / HAVING)
        //   - S-3.10: aggregate cost models
        //   - S-3.12: push-down decisions (aggregates cannot be pushed to sensors)
        //
        // `count(*)` → Expr::FuncCall { name: "count", args: [Expr::Star] }
        // `sum(field)` → Expr::FuncCall { name: "sum", args: [Expr::Field(field)] }
        let func_call_star = ident
            .padded()
            .then_ignore(just('(').padded())
            .then_ignore(just('*').padded())
            .then_ignore(just(')').padded())
            .map(|name: String| Expr::FuncCall {
                name,
                args: vec![Expr::Star],
            });

        let func_call_field = ident
            .padded()
            .then(
                field_path
                    .clone()
                    .padded()
                    .delimited_by(just('(').padded(), just(')').padded()),
            )
            .map(|(name, fp): (String, FieldPath)| Expr::FuncCall {
                name,
                args: vec![Expr::Field(fp)],
            });

        // Basic comparison (field vs literal).
        let comparison = field_path
            .clone()
            .padded()
            .then(compare_op.clone())
            .then(literal.clone().padded().map(Expr::Literal))
            .map(|((fp, op), rhs)| Expr::Compare {
                lhs: Box::new(Expr::Field(fp)),
                op,
                rhs: Box::new(rhs),
            });

        // field = field comparisons (for JOIN ON conditions and cross-field comparisons).
        let field_comparison = field_path
            .clone()
            .padded()
            .then(compare_op)
            .then(field_path.clone().padded().map(Expr::Field))
            .map(|((fp, op), rhs)| Expr::Compare {
                lhs: Box::new(Expr::Field(fp)),
                op,
                rhs: Box::new(rhs),
            });

        // Atom — order matters: try subquery before in_list (both start with field IN).
        let atom = choice((
            expr.clone()
                .padded()
                .delimited_by(just('(').padded(), just(')').padded()),
            in_subquery,
            in_list,
            func_call_star,
            func_call_field,
            field_comparison,
            comparison,
            literal.clone().padded().map(Expr::Literal),
            field_path.clone().padded().map(Expr::Field),
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
