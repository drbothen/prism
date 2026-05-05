//! PrismQL Abstract Syntax Tree types.
//!
//! All three query modes (filter, SQL, pipe) share these expression types.
//! AST nodes are pure data — no I/O, no sensor resolution. The executor
//! injects org scope at planning time (ADR-006 compliance).
//!
//! Story: S-3.01 | BC-2.11.002 / BC-2.11.003 / BC-2.11.004

/// Top-level AST discriminant — the result of a successful parse.
///
/// `#[non_exhaustive]` enables S-3.06 to add new query modes without
/// breaking existing match arms in downstream crates.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq)]
pub enum Ast {
    /// Filter mode: `source | predicate` (BC-2.11.002)
    Filter(FilterExpr),
    /// SQL mode: `SELECT … FROM … JOIN … WHERE …` (BC-2.11.003)
    Sql(SqlQuery),
    /// Pipe mode: `source | stage | stage …` (BC-2.11.004)
    Pipe(PipeQuery),
}

// ─────────────────────────────────────────────────────────────────────────────
// Filter mode AST
// ─────────────────────────────────────────────────────────────────────────────

/// Filter mode AST: `source | predicate` (BC-2.11.002).
///
/// `#[non_exhaustive]` prevents exhaustive struct matching in downstream
/// crates, enabling S-3.06 to add fields without a breaking change.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq)]
pub struct FilterExpr {
    /// Dot-notation sensor.table reference, e.g. `crowdstrike.detections`.
    pub source: SourceRef,
    /// Root predicate expression.
    pub predicate: Predicate,
}

// ─────────────────────────────────────────────────────────────────────────────
// SQL mode AST
// ─────────────────────────────────────────────────────────────────────────────

/// SQL mode AST (BC-2.11.003).
///
/// `#[non_exhaustive]` enables S-3.06 extension (e.g. WITH/CTE clauses).
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq)]
pub struct SqlQuery {
    pub select: SelectClause,
    pub from: FromClause,
    pub joins: Vec<Join>,
    pub where_: Option<Expr>,
    pub group_by: Vec<Expr>,
    pub having: Option<Expr>,
    pub order_by: Vec<OrderExpr>,
    pub limit: Option<u64>,
}

/// SELECT clause — list of projection items.
#[derive(Debug, Clone, PartialEq)]
pub struct SelectClause {
    pub distinct: bool,
    pub items: Vec<SelectItem>,
}

/// A single item in a SELECT clause.
#[derive(Debug, Clone, PartialEq)]
pub enum SelectItem {
    /// `*` — all columns.
    Star,
    /// `table.*` — all columns from a specific table alias.
    TableStar(String),
    /// `expr [AS alias]`
    Expr { expr: Expr, alias: Option<String> },
}

/// FROM clause — primary source reference with optional alias.
#[derive(Debug, Clone, PartialEq)]
pub struct FromClause {
    pub source: SourceRef,
    pub alias: Option<String>,
}

/// JOIN clause (INNER / LEFT / RIGHT / FULL OUTER).
#[derive(Debug, Clone, PartialEq)]
pub struct Join {
    pub kind: JoinKind,
    pub source: SourceRef,
    pub alias: Option<String>,
    pub on: Expr,
}

/// JOIN type discriminant.
#[derive(Debug, Clone, PartialEq)]
pub enum JoinKind {
    Inner,
    Left,
    Right,
    FullOuter,
}

/// ORDER BY element.
#[derive(Debug, Clone, PartialEq)]
pub struct OrderExpr {
    pub expr: Expr,
    pub direction: SortDirection,
}

// ─────────────────────────────────────────────────────────────────────────────
// Pipe mode AST
// ─────────────────────────────────────────────────────────────────────────────

/// Pipe mode AST (BC-2.11.004): `source | stage | stage …`.
#[derive(Debug, Clone, PartialEq)]
pub struct PipeQuery {
    pub source: SourceRef,
    pub stages: Vec<PipeStage>,
}

/// A single stage in a pipe query (BC-2.11.004).
///
/// `#[non_exhaustive]` enables S-3.06 to add new stage types without
/// breaking existing `match` arms in downstream crates.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq)]
pub enum PipeStage {
    /// `where expr` — filter stage.
    Where(Expr),
    /// `sort field [asc|desc] [, …]` — sort stage.
    Sort(Vec<SortExpr>),
    /// `head N` / `limit N` — take first N rows.
    Limit(u64),
    /// `tail N` — take last N rows.
    Tail(u64),
    /// `stats agg_func [by field]` — aggregation stage.
    Stats(StatsStage),
    /// `dedup field [, …]` — deduplicate by fields.
    Dedup(Vec<FieldPath>),
    /// `fields [+|-] field [, …]` — include/exclude fields.
    Fields(FieldsStage),
    /// `join source_ref on field_path` — join stage.
    Join(JoinStage),
    /// `enrich infusion(field_path)` — enrichment stage.
    Enrich(EnrichStage),
}

/// `sort` stage element.
#[derive(Debug, Clone, PartialEq)]
pub struct SortExpr {
    pub field: FieldPath,
    pub direction: SortDirection,
}

/// Sort direction.
#[derive(Debug, Clone, PartialEq)]
pub enum SortDirection {
    Asc,
    Desc,
}

/// `stats` stage: aggregation function + optional GROUP BY field.
#[derive(Debug, Clone, PartialEq)]
pub struct StatsStage {
    pub func: AggFunc,
    pub by: Option<FieldPath>,
}

/// Supported aggregation functions in the `stats` stage.
#[derive(Debug, Clone, PartialEq)]
pub enum AggFunc {
    Count,
    Sum(FieldPath),
    Avg(FieldPath),
    Min(FieldPath),
    Max(FieldPath),
}

/// `fields` stage: include (+) or exclude (-) fields.
#[derive(Debug, Clone, PartialEq)]
pub struct FieldsStage {
    pub include: bool,
    pub fields: Vec<FieldPath>,
}

/// `join` stage in a pipe query.
#[derive(Debug, Clone, PartialEq)]
pub struct JoinStage {
    pub source: SourceRef,
    pub on: FieldPath,
}

/// `enrich infusion(field_path)` stage (AD-020, S-1.14).
#[derive(Debug, Clone, PartialEq)]
pub struct EnrichStage {
    pub infusion: String,
    pub field: FieldPath,
}

// ─────────────────────────────────────────────────────────────────────────────
// Shared expression types (used by all three modes)
// ─────────────────────────────────────────────────────────────────────────────

/// Shared expression type used across all three query modes.
///
/// `#[non_exhaustive]` enables S-3.06 to add new expression forms (e.g.
/// window functions, CASE expressions) without breaking downstream matches.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// Literal value.
    Literal(Literal),
    /// Field path reference, e.g. `device.hostname`.
    Field(FieldPath),
    /// Binary comparison: `lhs op rhs`.
    Compare {
        lhs: Box<Expr>,
        op: CompareOp,
        rhs: Box<Expr>,
    },
    /// Logical combination: `lhs AND/OR rhs`.
    Logical {
        lhs: Box<Expr>,
        op: LogicalOp,
        rhs: Box<Expr>,
    },
    /// Logical negation: `NOT expr`.
    Not(Box<Expr>),
    /// `field IN (literal, …)` membership test.
    In {
        field: FieldPath,
        values: Vec<Literal>,
    },
    /// `field IN (SELECT …)` subquery membership test.
    InSubquery {
        field: FieldPath,
        subquery: Box<SqlQuery>,
    },
    /// SQL aggregate / scalar function call, e.g. `count(*)`, `sum(bytes)`.
    ///
    /// Added in S-3.01 to preserve function-call semantics that would otherwise
    /// be silently collapsed into `Expr::Field`. Downstream stories that need
    /// this:
    /// - S-3.04: aliased function calls (`count(*) AS total`)
    /// - S-3.07: aggregate evaluation (`GROUP BY` / `HAVING`)
    /// - S-3.10: aggregate cost models
    /// - S-3.12: push-down decisions (aggregates cannot be pushed down to sensors)
    FuncCall {
        /// Function name, e.g. `"count"`, `"sum"`, `"avg"`.
        name: String,
        /// Argument expressions. For `count(*)` this is `[Expr::Star]`.
        args: Vec<Expr>,
    },
    /// Wildcard `*` used as a function argument (e.g. the `*` in `count(*)`).
    ///
    /// Distinct from `SelectItem::Star` (which expands all columns in a SELECT
    /// clause). `Expr::Star` is only valid as an argument inside a `FuncCall`.
    Star,
}

/// Predicate is a type alias for `Expr` — used in filter mode for clarity.
pub type Predicate = Expr;

/// Dot-notation sensor.table reference, e.g. `crowdstrike.detections`.
///
/// Path traversal characters (`/`, `\`, `..`) are rejected at parse time
/// (S-3.01 §Architecture Compliance Rules, EC-004).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SourceRef {
    /// Raw dot-notation string, e.g. `"crowdstrike.detections"`.
    pub raw: String,
}

/// Dot-notation field path, e.g. `device.hostname`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FieldPath {
    /// Dot-separated path segments, e.g. `["device", "hostname"]`.
    pub segments: Vec<String>,
}

/// Literal value appearing in expressions.
#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    /// Single-quoted or double-quoted string.
    String(String),
    /// Integer literal.
    Integer(i64),
    /// Floating-point literal.
    Float(f64),
    /// Boolean literal (`true` / `false`, case-insensitive).
    Bool(bool),
    /// NULL literal.
    Null,
}

/// Comparison operator.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompareOp {
    Eq,
    Ne,
    Gt,
    Lt,
    Ge,
    Le,
    /// Glob/pattern matching operator (`LIKE`).
    Like,
    /// CIDR network range membership operator (`cidr`).
    ///
    /// Semantically distinct from `Like` — `Like` is glob/regex matching while
    /// `Cidr` tests whether an IP address falls within a network prefix (e.g.
    /// `dst_endpoint.ip cidr "10.0.0.0/8"`). Downstream executors (S-3.07) and
    /// optimizers (S-3.10/S-3.12) use this variant to select the correct
    /// implementation strategy.
    Cidr,
}

/// Logical binary operator.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LogicalOp {
    And,
    Or,
}
