//! Filter mode parser: `source | predicate` (BC-2.11.002).
//!
//! Grammar:
//!   filter_query := source_ref '|' predicate
//!   source_ref   := dotted_ident  (e.g. `crowdstrike.detections`)
//!   predicate    := expr
//!   expr         := or_expr
//!   or_expr      := and_expr ('OR' and_expr)*
//!   and_expr     := not_expr ('AND' not_expr)*
//!   not_expr     := 'NOT' not_expr | atom_expr
//!   atom_expr    := '(' expr ')' | compare_expr
//!   compare_expr := field_path op literal
//!                 | field_path 'IN' '(' literal_list ')'
//!   op           := '=' | '!=' | '>' | '<' | '>=' | '<=' | 'LIKE' | 'cidr'
//!
//! Mode detection: filter mode is detected when a `|` immediately follows
//! the source ref with no `SELECT` or `FROM` keyword.
//!
//! Story: S-3.01 | BC-2.11.002

use chumsky::prelude::*;

use crate::ast::{
    Ast, CompareOp, Expr, FieldPath, FilterExpr, Literal, LogicalOp, PipeQuery, SourceRef,
};
use crate::error::ParseError;
use crate::error_recovery::rich_to_parse_error;
use crate::pipe_parser::build_pipe_parser;
use crate::security;

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
    /// - `check_nesting_depth` is called on the resulting AST before returning.
    ///
    /// # Errors
    /// Returns `Err(Vec<ParseError>)` if the input is syntactically invalid or
    /// exceeds security limits. The vector may contain multiple errors when
    /// Chumsky error recovery is active.
    pub fn parse(input: &str) -> Result<Ast, Vec<ParseError>> {
        // Security check: reject oversized queries before any parsing.
        security::check_query_size(input).map_err(|e| vec![ParseError::new(0, e.to_string())])?;

        // Security check: parenthesis nesting depth (EC-002, BC-2.11.006, VP-015).
        // Uses the canonical `check_paren_depth` guard which correctly skips string
        // literals and counts maximum simultaneous open-paren depth.
        // This check runs before the Chumsky parser to cap structural recursion depth
        // before the parser even starts descending.
        security::check_paren_depth(input).map_err(|e| vec![ParseError::new(0, e.to_string())])?;

        // Reject empty / whitespace-only queries.
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return Err(vec![ParseError::new(0, "E-QUERY-001: empty query string")]);
        }

        // Reject mutation statements immediately.
        let upper = trimmed.to_uppercase();
        if upper.starts_with("INSERT ")
            || upper.starts_with("UPDATE ")
            || upper.starts_with("DELETE ")
            || upper.starts_with("DROP ")
            || upper.starts_with("CREATE ")
            || upper.starts_with("ALTER ")
            || upper.starts_with("TRUNCATE ")
        {
            return Err(vec![ParseError::new(
                0,
                "E-QUERY-001: mutation statements (INSERT/UPDATE/DELETE) are not permitted",
            )]);
        }

        // Mode detection:
        // 1. Starts with SELECT (case-insensitive) → SQL mode.
        // 2. Starts with FROM (case-insensitive) → Pipe mode.
        // 3. Starts with `|` → Pipe mode (no source prefix, EC-11-009).
        // 4. Contains pipe stage keywords after `|` → Pipe mode.
        // 5. Otherwise → Filter mode.
        if upper.starts_with("SELECT") {
            return parse_sql_internal(input);
        }
        if upper.starts_with("FROM") || trimmed.starts_with('|') {
            return parse_pipe_internal(input);
        }

        // Detect pipe-vs-filter: if there's a `|` and the token after it is a
        // pipe stage keyword (where/sort/head/tail/stats/dedup/fields/join/enrich/limit),
        // route to pipe mode.
        if is_pipe_mode(trimmed) {
            return parse_pipe_internal(input);
        }

        // Filter mode.
        parse_filter_internal(input)
    }
}

/// Detect whether the input is pipe mode by looking for a `|` followed by
/// a pipe stage keyword.
fn is_pipe_mode(input: &str) -> bool {
    // Find the first `|` that is not inside a string literal.
    // For simplicity (and correctness for non-edge-case inputs), scan for `|`
    // and check what comes after it (ignoring whitespace).
    let pipe_stage_keywords = [
        "where", "sort", "head", "tail", "stats", "dedup", "fields", "join", "enrich", "limit",
    ];

    let mut in_single_quote = false;
    let mut in_double_quote = false;
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        match chars[i] {
            '\'' if !in_double_quote => in_single_quote = !in_single_quote,
            '"' if !in_single_quote => in_double_quote = !in_double_quote,
            '|' if !in_single_quote && !in_double_quote => {
                // Look at what follows the `|`
                let rest: String = chars[i + 1..].iter().collect();
                let rest_trimmed = rest.trim_start();
                let lower_rest = rest_trimmed.to_lowercase();
                for kw in &pipe_stage_keywords {
                    if let Some(after) = lower_rest.strip_prefix(kw) {
                        // Check it's a keyword boundary (not part of identifier)
                        if after.is_empty()
                            || after.starts_with(' ')
                            || after.starts_with('\t')
                            || after.starts_with('\n')
                        {
                            return true;
                        }
                    }
                }
                // A `|` followed by non-keyword is still filter mode if it's the first `|`
            }
            _ => {}
        }
        i += 1;
    }
    false
}

/// Parse filter mode internally, wrapping result as `Ast::Filter`.
fn parse_filter_internal(input: &str) -> Result<Ast, Vec<ParseError>> {
    parse_filter(input).map(Ast::Filter)
}

/// Parse SQL mode internally, wrapping result as `Ast::Sql`.
fn parse_sql_internal(input: &str) -> Result<Ast, Vec<ParseError>> {
    crate::sql_parser::parse_sql(input).map(Ast::Sql)
}

/// Parse pipe mode internally, wrapping result as `Ast::Pipe`.
fn parse_pipe_internal(input: &str) -> Result<Ast, Vec<ParseError>> {
    crate::pipe_parser::parse_pipe(input).map(Ast::Pipe)
}

// ── Security re-export for convenient use in tests ────────────────────────────
pub use security::{PRISM_MAX_NESTING_DEPTH, PRISM_MAX_QUERY_SIZE};

/// Parse a filter-mode query: `source | predicate` or just `predicate`.
///
/// Called by `PrismQlParser::parse` after mode detection confirms the input
/// is filter mode (a `|` follows the source ref with no SELECT/FROM keyword).
///
/// # Errors
/// Returns accumulated `ParseError`s on failure. On recoverable errors,
/// both a partial AST and errors may be available via the recovery protocol
/// in `error_recovery.rs`.
pub fn parse_filter(input: &str) -> Result<FilterExpr, Vec<ParseError>> {
    let parser = build_filter_parser();
    let (result, errs) = parser.parse(input).into_output_errors();
    if errs.is_empty() {
        if let Some(fe) = result {
            // Security: check nesting depth on parsed predicate.
            security::check_nesting_depth(&fe.predicate, 0)
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
    let expr = build_expr_parser();

    // source_ref: dotted identifier, e.g. `crowdstrike.detections`
    // Reject path traversal characters (`/`, `\`, `..`) per EC-004.
    let source_ref = build_source_ref_parser();

    // Optional `source_ref '|'` prefix.
    // If present: parse source, skip `|`, then parse predicate.
    // If absent: use empty source and parse only the predicate.
    let with_source = source_ref
        .then_ignore(just('|').padded())
        .then(expr.clone())
        .map(|(src, pred)| FilterExpr {
            source: src,
            predicate: pred,
        });

    // Filter without source prefix: just an expression.
    let without_source = expr.map(|pred| FilterExpr {
        source: SourceRef { raw: String::new() },
        predicate: pred,
    });

    with_source.or(without_source)
}

/// Build the source reference parser (dotted-ident, rejects path traversal).
pub fn build_source_ref_parser<'a>(
) -> impl Parser<'a, &'a str, SourceRef, extra::Err<Rich<'a, char>>> + Clone {
    // A source ref segment: starts with alpha/underscore, continues with alphanumeric/underscore.
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
            Ok(SourceRef {
                raw: raw.to_string(),
            })
        })
}

/// Build the shared expression parser used across all three query modes.
pub fn build_expr_parser<'a>() -> impl Parser<'a, &'a str, Expr, extra::Err<Rich<'a, char>>> + Clone
{
    recursive(|expr| {
        // Literal values.
        let literal = build_literal_parser();

        // Field path: dotted identifier supporting underscores and leading underscores.
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
            // CIDR network-range operator — semantically distinct from LIKE.
            text::keyword("cidr").to(CompareOp::Cidr),
            text::keyword("CIDR").to(CompareOp::Cidr),
        ))
        .padded();

        // IN list: `field_path IN (literal, ...)`
        let in_list = field_path
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

        // Basic comparison: `field_path op literal`.
        let comparison = field_path
            .padded()
            .then(compare_op)
            .then(literal.clone().padded().map(Expr::Literal))
            .map(|((fp, op), rhs)| Expr::Compare {
                lhs: Box::new(Expr::Field(fp)),
                op,
                rhs: Box::new(rhs),
            });

        // Atom: `(expr)` | in_list | comparison
        let atom = choice((
            expr.clone()
                .padded()
                .delimited_by(just('(').padded(), just(')').padded()),
            in_list,
            comparison,
        ));

        // NOT expr
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

        // AND combinator (left-associative).
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

        // OR combinator (left-associative).
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

/// Build the literal value parser.
pub fn build_literal_parser<'a>(
) -> impl Parser<'a, &'a str, Literal, extra::Err<Rich<'a, char>>> + Clone {
    // Single-quoted string literal (standard SQL style).
    let single_quoted = none_of('\'')
        .repeated()
        .to_slice()
        .map(|s: &str| Literal::String(s.to_string()))
        .delimited_by(just('\''), just('\''));

    // Double-quoted string literal.
    let double_quoted = none_of('"')
        .repeated()
        .to_slice()
        .map(|s: &str| Literal::String(s.to_string()))
        .delimited_by(just('"'), just('"'));

    // NULL literal.
    let null_lit = choice((
        text::keyword("NULL").to(Literal::Null),
        text::keyword("null").to(Literal::Null),
    ));

    // Boolean literals.
    let bool_lit = choice((
        text::keyword("true").to(Literal::Bool(true)),
        text::keyword("false").to(Literal::Bool(false)),
        text::keyword("TRUE").to(Literal::Bool(true)),
        text::keyword("FALSE").to(Literal::Bool(false)),
    ));

    // Numeric literals: try float first (has decimal point), then integer.
    let float_lit = just('-')
        .or_not()
        .then(text::int(10))
        .then(just('.').then(text::digits(10)))
        .to_slice()
        .try_map(|s: &str, span| {
            s.parse::<f64>()
                .map(Literal::Float)
                .map_err(|e| Rich::custom(span, format!("invalid float literal: {e}")))
        });

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
        float_lit,
        int_lit,
    ))
}

/// Build the pipe stage parser (forwarded from pipe_parser module).
/// Used in mode detection contexts.
pub fn build_pipe_mode_parser<'a>(
) -> impl Parser<'a, &'a str, PipeQuery, extra::Err<Rich<'a, char>>> {
    build_pipe_parser()
}
