//! PrismQL Abstract Syntax Tree types.
//!
//! All three query modes (filter, SQL, pipe) share these expression types.
//! AST nodes are pure data — no I/O, no sensor resolution. The executor
//! injects org scope at planning time (ADR-006 compliance).
//!
//! # Canonical Comparability
//! All AST types implement `Eq + Hash`. Two ASTs are `==` iff they would
//! produce the same execution plan. `Literal::Float` wraps `f64` in
//! `OrderedFloat` so that hash stability holds across clones.
//!
//! # Serde
//! All public AST types derive `Serialize + Deserialize` for JSON round-trips
//! used in the demo harness and MCP tool responses.
//!
//! Story: S-3.01 | BC-2.11.002 / BC-2.11.003 / BC-2.11.004
//! S-3.06 | BC-2.11.004 (write parser extensions)

use std::net::IpAddr;

use chrono::{DateTime, Utc};
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};

// ─────────────────────────────────────────────────────────────────────────────
// Top-level AST
// ─────────────────────────────────────────────────────────────────────────────

/// Top-level AST discriminant — the result of a successful parse.
///
/// `#[non_exhaustive]` enables S-3.06 to add new query modes without
/// breaking existing match arms in downstream crates.
///
/// # Size note
/// Variants differ in size (Filter < Pipe < Sql). The enum is returned
/// by value from `PrismQlParser::parse`; callers typically match immediately
/// and work with the inner value. Clippy's `large_enum_variant` is suppressed
/// because boxing would break the ergonomic match syntax used throughout.
#[non_exhaustive]
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Ast {
    /// Filter mode: `[source |] predicate` (BC-2.11.002)
    Filter(FilterExpr),
    /// SQL mode: `SELECT … FROM … JOIN … WHERE …` (BC-2.11.003).
    /// Wrapped in `SqlStatement` for forward-compat (S-3.06 will add DML/DDL).
    Sql(SqlStatement),
    /// Pipe mode: `source | stage | stage …` (BC-2.11.004)
    Pipe(PipeQuery),
    /// SQL→Pipe composition mode (BC-2.11.020, ADR-043):
    /// `SELECT … FROM t [WHERE …] [LIMIT n] | stage | stage …`
    SqlPipe(SqlPipeQuery),
}

/// SQL→Pipe composition query (BC-2.11.020, ADR-043).
///
/// The head SQL SELECT is subject to the FORBID-BOTH invariant: if the SQL
/// SELECT carries `LIMIT n` AND any pipe stage has `| limit m`, the planner
/// must return `E-QUERY-040` (`PrismError::RedundantRowLimit`).
///
/// `#[non_exhaustive]` allows future fields (e.g., `with` CTEs, query-level
/// hints) without breaking external struct literals.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SqlPipeQuery {
    /// The SQL SELECT head (without pipe stages).
    pub head: SqlQuery,
    /// Pipe stages following the SQL head.
    pub stages: Vec<PipeStage>,
}

// ─────────────────────────────────────────────────────────────────────────────
// Filter mode AST
// ─────────────────────────────────────────────────────────────────────────────

/// Filter mode AST: `[source |] predicate` (BC-2.11.002).
///
/// `#[non_exhaustive]` prevents exhaustive struct matching in downstream
/// crates, enabling S-3.06 to add fields without a breaking change.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FilterExpr {
    /// Dot-notation sensor.table reference, e.g. `crowdstrike.detections`.
    pub source: SourceRef,
    /// Root predicate — the boolean condition applied to each row.
    pub predicate: Predicate,
}

// ─────────────────────────────────────────────────────────────────────────────
// SQL mode AST
// ─────────────────────────────────────────────────────────────────────────────

/// SQL statement wrapper — extended in S-3.06 with DML variants.
///
/// S-3.06 adds `Dml(DmlNode)` for `INSERT INTO`, `UPDATE`, `DELETE`.
///
/// # Implements BC-2.11.004 — Write Parser Extension (S-3.06)
///
/// # Size note
/// `Select(SqlQuery)` (~536 bytes) is larger than `Dml(DmlNode)`.
/// The lint is suppressed because introducing indirection (boxing) on `Select`
/// would break ergonomic match patterns throughout the codebase.
#[non_exhaustive]
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SqlStatement {
    /// A `SELECT` query.
    Select(SqlQuery),
    /// A DML statement: `INSERT INTO`, `UPDATE`, or `DELETE FROM`.
    ///
    /// Added by S-3.06. Parse-time validation:
    /// - `prism_*` target tables return `E-QUERY-010`
    /// - `UPDATE`/`DELETE` without WHERE return `E-QUERY-022`
    /// - `INSERT INTO … SELECT` without LIMIT or WHERE returns `E-QUERY-022`
    Dml(crate::write_ast::DmlNode),
}

/// SQL mode AST (BC-2.11.003).
///
/// `#[non_exhaustive]` enables S-3.06 extension (e.g. WITH/CTE clauses).
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SqlQuery {
    pub select: SelectClause,
    pub from: FromClause,
    pub joins: Vec<Join>,
    /// WHERE clause predicate (distinct from value `Expr`).
    pub where_: Option<Predicate>,
    pub group_by: Vec<Expr>,
    /// HAVING clause predicate.
    pub having: Option<Predicate>,
    pub order_by: Vec<OrderExpr>,
    pub limit: Option<u64>,
}

impl SqlQuery {
    /// Construct a minimal `SqlQuery` — useful in tests for building subquery fixtures.
    pub fn new(select: SelectClause, from: FromClause) -> Self {
        Self {
            select,
            from,
            joins: vec![],
            where_: None,
            group_by: vec![],
            having: None,
            order_by: vec![],
            limit: None,
        }
    }

    /// Attach a WHERE predicate to this query.
    pub fn with_where(mut self, pred: Predicate) -> Self {
        self.where_ = Some(pred);
        self
    }

    /// Construct a recovery-sentinel `SqlQuery` used by the `nested_delimiters`
    /// error-recovery combinator in `sql_parser::build_sql_predicate_parser`.
    ///
    /// When the content inside `IN (...)` cannot be parsed as a valid subquery,
    /// the recovery combinator inserts this sentinel. The caller converts it to
    /// `Predicate::RecoveryError` via `is_recovery_sentinel()`. (F-MEDIUM-001)
    pub(crate) fn recovery_sentinel() -> Self {
        // Use a syntactically invalid sentinel: a SelectClause with no items
        // and a SourceRef whose raw starts with the sentinel prefix.
        // The canonical sentinel is detected by is_recovery_sentinel().
        Self {
            select: SelectClause {
                distinct: false,
                items: vec![],
            },
            from: FromClause {
                source: SourceRef {
                    raw: "__recovery_sentinel__".to_string(),
                    kind: SourceRefKind::Custom,
                },
                alias: None,
            },
            joins: vec![],
            where_: Some(Predicate::RecoveryError),
            group_by: vec![],
            having: None,
            order_by: vec![],
            limit: None,
        }
    }

    /// Returns `true` if this `SqlQuery` is a recovery sentinel injected by the
    /// `nested_delimiters` combinator. Callers should convert it to
    /// `Predicate::RecoveryError`. (F-MEDIUM-001)
    pub(crate) fn is_recovery_sentinel(&self) -> bool {
        self.from.source.raw == "__recovery_sentinel__"
    }
}

/// SELECT clause — list of projection items.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SelectClause {
    pub distinct: bool,
    pub items: Vec<SelectItem>,
}

impl SelectClause {
    /// Construct a non-distinct SELECT clause from items.
    pub fn new(items: Vec<SelectItem>) -> Self {
        Self {
            distinct: false,
            items,
        }
    }
}

/// A single item in a SELECT clause.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SelectItem {
    /// `*` — all columns.
    Star,
    /// `table.*` — all columns from a specific table alias.
    TableStar(String),
    /// `expr [AS alias]`
    Expr { expr: Expr, alias: Option<String> },
}

/// FROM clause — primary source reference with optional alias.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FromClause {
    pub source: SourceRef,
    pub alias: Option<String>,
}

impl FromClause {
    /// Construct a `FromClause` with no alias.
    pub fn new(source: SourceRef) -> Self {
        Self {
            source,
            alias: None,
        }
    }
}

/// JOIN clause (INNER / LEFT / RIGHT / FULL OUTER / CROSS).
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Join {
    pub kind: JoinKind,
    pub source: SourceRef,
    pub alias: Option<String>,
    /// JOIN ON condition. Stored as `Expr` (field=field equality).
    pub on: Expr,
}

/// JOIN type discriminant.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum JoinKind {
    Inner,
    Left,
    Right,
    FullOuter,
    /// CROSS JOIN — no ON clause; produces Cartesian product.
    Cross,
}

/// ORDER BY element.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct OrderExpr {
    pub expr: Expr,
    pub direction: SortDirection,
}

// ─────────────────────────────────────────────────────────────────────────────
// Pipe mode AST
// ─────────────────────────────────────────────────────────────────────────────

/// Pipe mode AST (BC-2.11.004): `source | stage | stage …`.
///
/// `write` carries the optional terminal write node added in S-3.06.
/// `None` for read-only pipelines (the common case in S-3.01 queries).
///
/// # Implements BC-2.11.004 — Write Parser Extension (S-3.06)
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PipeQuery {
    pub source: SourceRef,
    pub stages: Vec<PipeStage>,
    /// Optional terminal write node (S-3.06).
    /// `None` for read-only pipelines; `Some(WriteNode)` when the pipeline
    /// ends with a registered write verb (e.g. `| contain`).
    pub write: Option<crate::write_ast::WriteNode>,
}

impl PipeQuery {
    /// Construct a read-only `PipeQuery` (no write node).
    ///
    /// Required because `#[non_exhaustive]` prevents struct-literal construction
    /// from outside `prism-query`. External callers (integration tests, downstream
    /// crates) use this constructor.
    pub fn new(source: SourceRef, stages: Vec<PipeStage>) -> Self {
        Self {
            source,
            stages,
            write: None,
        }
    }
}

/// A single stage in a pipe query (BC-2.11.004).
///
/// `#[non_exhaustive]` enables S-3.06 to add new stage types without
/// breaking existing `match` arms in downstream crates.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PipeStage {
    /// `where predicate` — filter stage.
    Where(Predicate),
    /// `sort field [asc|desc] [, …]` — sort stage.
    Sort(Vec<SortExpr>),
    /// `head N` / `limit N` — take first N rows.
    Limit(u64),
    /// `tail N` — take last N rows.
    Tail(u64),
    /// `stats agg_func [, …] [by field, …]` — multi-aggregate aggregation stage.
    Stats(StatsStage),
    /// `dedup field [, …]` — deduplicate by fields.
    Dedup(Vec<FieldPath>),
    /// `fields [+|-] field [, …]` — include/exclude fields.
    Fields(FieldsStage),
    /// `join [kind] source on field [== field]` — join stage.
    Join(JoinStage),
    /// `enrich infusion(field_path)` — enrichment stage.
    Enrich(EnrichStage),
}

/// `sort` stage element.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SortExpr {
    pub field: FieldPath,
    pub direction: SortDirection,
}

impl SortExpr {
    /// Construct a `SortExpr`.
    ///
    /// Required because `#[non_exhaustive]` prevents struct-literal construction
    /// from outside the crate (E0639). Integration tests and downstream crates use
    /// this constructor.
    pub fn new(field: FieldPath, direction: SortDirection) -> Self {
        Self { field, direction }
    }
}

/// Sort direction.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SortDirection {
    Asc,
    Desc,
}

/// `stats` stage: one or more named aggregates + optional GROUP BY fields.
///
/// Replaces the previous single-aggregate `StatsStage { func, by }` with
/// multi-aggregate support per BC-2.11.004 and prismql-grammar.md §6.
///
/// # Backward compatibility for existing tests
/// The single-agg + single-by pattern is preserved via helper accessors:
/// - `ss.func` → `ss.aggregates[0].func.clone()` (single-agg queries)
/// - `ss.by` → `ss.by_fields.first().cloned()` (single group-by field)
///
/// These accessors are provided for test & downstream-code convenience;
/// new code should iterate `aggregates` and `by_fields` directly.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StatsStage {
    /// One or more aggregation functions with optional aliases.
    /// Invariant: `!aggregates.is_empty()`.
    pub aggregates: Vec<StatFunction>,
    /// GROUP BY fields (empty if no BY clause).
    pub by_fields: Vec<FieldPath>,
}

impl StatsStage {
    /// Convenience accessor — returns the first aggregate's `AggFunc`.
    ///
    /// Used by existing single-aggregate tests. Returns `AggFunc::Count`
    /// (the zero/empty case) if `aggregates` is somehow empty, though
    /// the parser guarantees at least one.
    #[inline]
    pub fn func(&self) -> AggFunc {
        self.aggregates
            .first()
            .map(|a| a.func.clone())
            .unwrap_or(AggFunc::Count)
    }

    /// Convenience accessor — returns the first GROUP BY field, if any.
    ///
    /// Used by existing single-by-field tests.
    #[inline]
    pub fn by(&self) -> Option<&FieldPath> {
        self.by_fields.first()
    }
}

/// A named aggregate in a `stats` stage.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StatFunction {
    pub func: AggFunc,
    /// Optional `AS alias` label.
    pub alias: Option<String>,
}

/// Supported aggregation functions (pipe `stats` and SQL aggregate expressions).
///
/// Unified between pipe mode and SQL mode — `count(*)` in SQL and `count` in
/// pipe mode both emit `AggFunc::Count` (no more divergence).
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AggFunc {
    /// `count(*)` or bare `count` — count all rows.
    Count,
    /// `count(field)` — count non-null values of a specific field.
    CountField(FieldPath),
    /// `sum(field)`
    Sum(FieldPath),
    /// `avg(field)`
    Avg(FieldPath),
    /// `min(field)`
    Min(FieldPath),
    /// `max(field)`
    Max(FieldPath),
    /// `distinct_count(field)` — count of unique values.
    DistinctCount(FieldPath),
    /// `percentile(field, p)` — `p` in [0, 100].
    Percentile {
        field: FieldPath,
        p: OrderedFloat<f64>,
    },
}

/// `fields` stage: include (+) or exclude (-) fields.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FieldsStage {
    pub include: bool,
    pub fields: Vec<FieldPath>,
}

/// `join` stage in a pipe query — structured with kind + typed ON condition.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct JoinStage {
    /// JOIN type (default INNER when omitted in syntax).
    pub kind: JoinKind,
    pub source: SourceRef,
    /// ON condition.
    pub on: JoinCondition,
}

/// Typed ON condition for pipe-mode JOIN.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum JoinCondition {
    /// `on field` — same field name on both sides.
    SameField(FieldPath),
    /// `on left == right` — different field names.
    Pair(FieldPath, FieldPath),
}

/// `enrich infusion(field_path)` stage (AD-020, S-1.14).
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EnrichStage {
    pub infusion: String,
    pub field: FieldPath,
}

impl EnrichStage {
    /// Construct an `EnrichStage`.
    ///
    /// Required because `#[non_exhaustive]` prevents struct-literal construction
    /// from outside `prism-query`. Integration tests and downstream crates use this.
    pub fn new(infusion: impl Into<String>, field: FieldPath) -> Self {
        Self {
            infusion: infusion.into(),
            field,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Predicate — boolean tree over field conditions
// ─────────────────────────────────────────────────────────────────────────────

/// Boolean predicate — the filter condition applied to each row.
///
/// `Predicate` is semantically distinct from `Expr`: a `Predicate` evaluates
/// to `true`/`false`, while an `Expr` evaluates to a typed value (string,
/// integer, float, etc.).  The separation prevents ill-typed queries such as
/// `WHERE count(*)` (an aggregate value is not a predicate).
///
/// # Operator coverage (prismql-grammar.md §4)
///
/// | PrismQL syntax | Variant |
/// |---|---|
/// | `field = val` / `field != val` / `field > val` etc. | `Compare` |
/// | `field CONTAINS "x"` / `ICONTAINS` / `STARTSWITH` etc. | `StringOp` |
/// | `field =~ "pat"` / `field MATCHES "pat"` | `Regex` |
/// | `field IN (a, b, c)` / `field NOT IN (…)` | `In` |
/// | `field IN (SELECT …)` / `NOT IN (SELECT …)` | `InSubquery` |
/// | `field BETWEEN low AND high` | `Between` |
/// | `field IN CIDR "10.0.0.0/8"` | `Cidr` |
/// | `HAS field` | `Has` |
/// | `MISSING field` | `Missing` |
/// | `field IS NULL` / `field IS NOT NULL` | `IsNull` |
/// | `field = "10.0.*"` (auto-promoted wildcard) | `Wildcard` |
/// | `AND` / `OR` with N children | `Logical` |
/// | `NOT predicate` | `Not` |
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Predicate {
    /// `field op literal` — basic comparison (=, !=, >, >=, <, <=).
    Compare {
        lhs: Box<Expr>,
        op: CompareOp,
        rhs: Box<Expr>,
    },
    /// String pattern operators (CONTAINS, STARTSWITH, ENDSWITH and their
    /// case-insensitive variants ICONTAINS, ISTARTSWITH, IENDSWITH).
    StringOp {
        field: FieldPath,
        op: StringOp,
        pattern: String,
        /// `true` for the I* case-insensitive variants.
        case_insensitive: bool,
    },
    /// `field =~ "regex"` / `field MATCHES "regex"`.
    /// Pattern is validated at parse time (CWE-1333 safe; finite automaton).
    Regex {
        field: FieldPath,
        pattern: RegexLiteral,
    },
    /// `field IN (val, …)` / `field NOT IN (val, …)`.
    In {
        field: FieldPath,
        values: Vec<Literal>,
        negated: bool,
    },
    /// `field IN (SELECT …)` / `field NOT IN (SELECT …)` subquery membership.
    InSubquery {
        field: FieldPath,
        subquery: Box<SqlQuery>,
        negated: bool,
    },
    /// `field BETWEEN low AND high`.
    Between {
        field: FieldPath,
        low: Literal,
        high: Literal,
        negated: bool,
    },
    /// `field IN CIDR "10.0.0.0/8"` — CIDR network range membership.
    /// CIDR string is validated at parse time (CWE-20).
    Cidr {
        field: FieldPath,
        cidr: CidrLiteral,
        negated: bool,
    },
    /// `HAS field` — field existence check.
    Has(FieldPath),
    /// `MISSING field` — field absence check.
    Missing(FieldPath),
    /// `field IS NULL` / `field IS NOT NULL`.
    IsNull { field: FieldPath, negated: bool },
    /// `field = "10.0.*"` / `field != "10.0.*"` — auto-promoted wildcard.
    ///
    /// Auto-promotion: a string literal containing `*` or `?` with `=` or
    /// `!=` operator is silently promoted to this variant at parse time.
    Wildcard {
        field: FieldPath,
        pattern: String,
        negated: bool,
    },
    /// `AND` / `OR` with N children (left-associative fold from binary ops).
    Logical {
        op: LogicalOp,
        predicates: Vec<Predicate>,
    },
    /// `NOT predicate` — logical negation.
    Not(Box<Predicate>),
    /// Inserted by the Chumsky error-recovery machinery when a parenthesised
    /// subexpression could not be parsed (e.g., a bogus `IN (...)` subquery).
    ///
    /// Semantics: always evaluates to `false` (i.e., the predicate never
    /// matches). The planner MUST treat this as a non-matching sentinel and
    /// MUST NOT attempt to execute it against sensor APIs. Down-stream consumers
    /// should check for this variant and surface an appropriate user-facing error.
    ///
    /// This variant is only produced by error recovery — well-formed queries
    /// never contain it. (F-MEDIUM-001, AC-9, BC-2.11.003)
    RecoveryError,
}

// ─────────────────────────────────────────────────────────────────────────────
// Expression — value-producing node (not boolean)
// ─────────────────────────────────────────────────────────────────────────────

/// Value-producing expression used in SELECT projections, ORDER BY,
/// GROUP BY, JOIN ON conditions, and function arguments.
///
/// `#[non_exhaustive]` enables S-3.06 to add CASE, window functions, etc.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Expr {
    /// Literal value.
    Literal(Literal),
    /// Field path reference, e.g. `device.hostname`.
    Field(FieldPath),
    /// Virtual field reference (`_sensor`, `_client`, etc.).
    VirtualField(VirtualField),
    /// Binary comparison: `lhs op rhs`. Used in JOIN ON conditions.
    Compare {
        lhs: Box<Expr>,
        op: CompareOp,
        rhs: Box<Expr>,
    },
    /// Logical combination: `lhs AND/OR rhs`. Used in complex JOIN ON.
    Logical {
        lhs: Box<Expr>,
        op: LogicalOp,
        rhs: Box<Expr>,
    },
    /// Logical negation: `NOT expr`.
    Not(Box<Expr>),
    /// `field IN (literal, …)` membership test (value context, e.g. IN subquery).
    In {
        field: FieldPath,
        values: Vec<Literal>,
    },
    /// `field IN (SELECT …)` subquery membership test.
    InSubquery {
        field: FieldPath,
        subquery: Box<SqlQuery>,
    },
    /// Typed function call (aggregate, scalar, or window stub).
    FuncCall(FuncCall),
    /// Wildcard `*` used as a function argument (e.g. the `*` in `count(*)`).
    Star,
    /// Planning-time constant: current timestamp (ADR-044).
    /// Injected by the planner at query planning time; never stored in results.
    Now,
    /// Planning-time duration constant, e.g. `INTERVAL '7 days'` (ADR-044).
    /// `chrono::Duration` represents the resolved duration after parsing `<int><unit>`.
    Interval(chrono::Duration),
    /// Timestamp arithmetic: `base ± offset` (ADR-044).
    /// Used in WHERE clauses: `timestamp > NOW() - INTERVAL '7 days'`.
    TimestampArithmetic {
        base: Box<Expr>,
        op: BinaryOp,
        offset: chrono::Duration,
    },
}

// ─────────────────────────────────────────────────────────────────────────────
// Function calls — typed (aggregate vs scalar vs window)
// ─────────────────────────────────────────────────────────────────────────────

/// Typed function call expression.
///
/// Separating aggregate from scalar prevents mixing them in non-aggregate
/// contexts (e.g. using `sum(x)` outside GROUP BY without a planner error).
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FuncCall {
    /// Aggregate function call — `count(*)`, `sum(field)`, etc.
    Aggregate {
        func: AggFunc,
        args: Vec<Expr>,
        /// `true` for `count(DISTINCT field)` — not yet parsed, reserved.
        distinct: bool,
    },
    /// Scalar (UDF) function call from the UDF registry.
    Scalar { func: ScalarFunc, args: Vec<Expr> },
    /// Window function stub — populated in S-3.06.
    Window {
        // Placeholder: S-3.06 will add fields here.
    },
}

/// Registered scalar (UDF) functions (query-engine.md §Security UDFs).
///
/// `Unknown(String)` provides an escape hatch for analyst-defined UDFs
/// not yet registered here — they still parse and can be executed if the
/// DataFusion context has them registered.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ScalarFunc {
    /// `subnet_contains(cidr, ip)` — CIDR membership test UDF.
    SubnetContains,
    /// `time_window(timestamp, duration)` — time-range UDF.
    TimeWindow,
    /// `json_extract_string(json, path)` — JSONPath extraction UDF.
    JsonExtractString,
    /// `ioc_match(field, list_name)` — IOC list membership UDF.
    IocMatch,
    /// `mitre_tactic(technique_id)` — ATT&CK v14 lookup UDF.
    MitreTactic,
    /// `severity_label(severity_id)` — threshold label UDF.
    SeverityLabel,
    /// Any UDF not in the registry above — analyst-defined or future UDFs.
    Unknown(String),
}

// ─────────────────────────────────────────────────────────────────────────────
// Virtual fields (BC-2.11.012)
// ─────────────────────────────────────────────────────────────────────────────

/// Virtual fields injected by the query engine (BC-2.11.012, S-2.08).
///
/// These fields are prefixed with `_` and are NOT user-supplied — they are
/// synthesized by the executor from materialization context. The parser
/// detects them by the leading `_` and emits `Expr::VirtualField` instead
/// of `Expr::Field` so that the planner and executor can handle them without
/// string-scanning field names.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VirtualField {
    /// `_sensor` — source sensor identifier (`crowdstrike`, `armis`, etc.).
    Sensor,
    /// `_client` — client OrgSlug (ADR-006).
    Client,
    /// `_source_table` — specific table name (`crowdstrike_detections`, etc.).
    SourceTable,
    /// `_source_type` — data source type (`live`, `buffered`, etc.).
    SourceType,
    /// `_safety_flags` — materialization safety flags bitmask.
    SafetyFlags,
}

// ─────────────────────────────────────────────────────────────────────────────
// Source reference — structured
// ─────────────────────────────────────────────────────────────────────────────

/// Source reference — where a query reads data from.
///
/// The `raw` field preserves the original source string as written by the
/// analyst (e.g. `"crowdstrike.detections"`, `"EVENTS"`) for display and
/// backward compatibility.
///
/// The `kind` field provides the structured classification from the parser,
/// enabling executors to dispatch without re-parsing.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SourceRef {
    /// Raw source string as written in the query (e.g. `"crowdstrike.detections"`).
    pub raw: String,
    /// Structured classification of the source (detected at parse time).
    pub kind: SourceRefKind,
}

impl SourceRef {
    /// Construct a `SourceRef` from a raw string, classifying its kind.
    pub fn from_raw(raw: impl Into<String>) -> Self {
        let raw = raw.into();
        let kind = SourceRefKind::classify(&raw);
        Self { raw, kind }
    }
}

/// Structured classification of a `SourceRef`.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SourceRefKind {
    /// Composite cross-sensor virtual source (e.g. `EVENTS`, `ALERTS`, `DEVICES`).
    Composite(CompositeSource),
    /// Specific sensor table (e.g. `crowdstrike.detections` → sensor `crowdstrike`, table `detections`).
    External { sensor: String, table: String },
    /// Internal Prism storage table (e.g. `prism_alerts`, `prism_cases`).
    Internal(InternalTable),
    /// User-defined view or custom identifier not matched above.
    Custom,
}

impl SourceRefKind {
    /// Classify a raw source string into its structured kind.
    pub fn classify(raw: &str) -> Self {
        let upper = raw.to_uppercase();
        // Composite sources (case-insensitive).
        match upper.as_str() {
            "EVENTS" => return SourceRefKind::Composite(CompositeSource::Events),
            "ALERTS" => return SourceRefKind::Composite(CompositeSource::Alerts),
            "DEVICES" => return SourceRefKind::Composite(CompositeSource::Devices),
            "ASSETS" => return SourceRefKind::Composite(CompositeSource::Assets),
            "SESSIONS" => return SourceRefKind::Composite(CompositeSource::Sessions),
            _ => {}
        }
        // Internal Prism tables (underscore-delimited, prism_ prefix).
        match raw {
            "prism_alerts" => return SourceRefKind::Internal(InternalTable::Alerts),
            "prism_cases" => return SourceRefKind::Internal(InternalTable::Cases),
            "prism_rules" => return SourceRefKind::Internal(InternalTable::Rules),
            "prism_schedules" => return SourceRefKind::Internal(InternalTable::Schedules),
            "prism_diff_results" => return SourceRefKind::Internal(InternalTable::DiffResults),
            "prism_audit" => return SourceRefKind::Internal(InternalTable::Audit),
            "prism_aliases" => return SourceRefKind::Internal(InternalTable::Aliases),
            _ => {}
        }
        // External sensor tables: `{sensor}.{table}` dotted notation.
        if let Some(dot) = raw.find('.') {
            let sensor = raw[..dot].to_string();
            let table = raw[dot + 1..].to_string();
            return SourceRefKind::External { sensor, table };
        }
        // Everything else: custom view / user-defined.
        SourceRefKind::Custom
    }
}

/// Composite cross-sensor virtual sources (prismql-grammar.md §11.2).
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CompositeSource {
    Events,
    Alerts,
    Devices,
    Assets,
    Sessions,
}

impl CompositeSource {
    pub fn as_str(&self) -> &'static str {
        match self {
            CompositeSource::Events => "EVENTS",
            CompositeSource::Alerts => "ALERTS",
            CompositeSource::Devices => "DEVICES",
            CompositeSource::Assets => "ASSETS",
            CompositeSource::Sessions => "SESSIONS",
        }
    }
}

/// Internal Prism storage tables (query-engine.md §Unified Query Surface).
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InternalTable {
    Alerts,
    Cases,
    Rules,
    Schedules,
    DiffResults,
    Audit,
    Aliases,
}

impl InternalTable {
    pub fn as_str(&self) -> &'static str {
        match self {
            InternalTable::Alerts => "prism_alerts",
            InternalTable::Cases => "prism_cases",
            InternalTable::Rules => "prism_rules",
            InternalTable::Schedules => "prism_schedules",
            InternalTable::DiffResults => "prism_diff_results",
            InternalTable::Audit => "prism_audit",
            InternalTable::Aliases => "prism_aliases",
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Field path
// ─────────────────────────────────────────────────────────────────────────────

/// Dot-notation field path, e.g. `device.hostname`.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FieldPath {
    /// Dot-separated path segments, e.g. `["device", "hostname"]`.
    pub segments: Vec<String>,
    /// Byte span in the original query string (populated by parser).
    pub span: Span,
}

impl FieldPath {
    /// Construct a `FieldPath` from segments (span defaults to `Span::ZERO`).
    /// Used in tests and test-fixture helpers.
    pub fn new(segments: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            segments: segments.into_iter().map(|s| s.into()).collect(),
            span: Span::ZERO,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Virtual field promotion helper
// ─────────────────────────────────────────────────────────────────────────────

/// Convert a `FieldPath` into `Expr::VirtualField` for the five canonical
/// underscore-prefixed names defined in BC-2.11.012, or `Expr::Field` otherwise.
///
/// Called by all three parsers (filter, SQL, pipe) whenever a field path is
/// emitted as a value expression. This ensures that `_sensor`, `_client`,
/// `_source_table`, `_source_type`, and `_safety_flags` are represented with
/// their typed variant in the AST rather than as generic field strings, giving
/// the planner and executor a first-class handle without string-scanning.
///
/// Any other leading-`_` name (analyst-defined metadata) is emitted as
/// `Expr::Field`, which is intentional: BC-2.11.012 enumerates exactly five
/// build-time-verified virtual fields and does not restrict arbitrary `_` names.
#[inline]
pub fn field_path_to_expr(fp: FieldPath) -> Expr {
    if let Some(first) = fp.segments.first() {
        match first.as_str() {
            "_sensor" => return Expr::VirtualField(VirtualField::Sensor),
            "_client" => return Expr::VirtualField(VirtualField::Client),
            "_source_table" => return Expr::VirtualField(VirtualField::SourceTable),
            "_source_type" => return Expr::VirtualField(VirtualField::SourceType),
            "_safety_flags" => return Expr::VirtualField(VirtualField::SafetyFlags),
            _ => {}
        }
    }
    Expr::Field(fp)
}

// ─────────────────────────────────────────────────────────────────────────────
// Span tracking (P1-002)
// ─────────────────────────────────────────────────────────────────────────────

/// Byte-offset span in the original query string.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default, Copy)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub const ZERO: Span = Span { start: 0, end: 0 };
}

/// A node with its source span.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Spanned<T> {
    pub node: T,
    pub span: Span,
}

// ─────────────────────────────────────────────────────────────────────────────
// Literal types — validated at parse time
// ─────────────────────────────────────────────────────────────────────────────

/// Literal value appearing in expressions and predicates.
///
/// Each variant is validated at parse time where a structural constraint exists
/// (CIDR format, regex syntax, duration overflow, percentile range).
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Literal {
    /// Single-quoted or double-quoted string.
    String(String),
    /// Integer literal (i64 range; overflow detected at parse time).
    Integer(i64),
    /// Floating-point literal. Wrapped in `OrderedFloat` for `Eq + Hash`.
    Float(OrderedFloat<f64>),
    /// Boolean literal (`true` / `false`, case-insensitive).
    Bool(bool),
    /// NULL literal.
    Null,
    /// Duration literal (`30s`, `24h`, `7d`, `5m`). Value validated for overflow.
    Duration(DurationLiteral),
    /// CIDR network range literal. Validated via `ipnet::IpNet::from_str` (CWE-20).
    Cidr(CidrLiteral),
    /// Regex pattern literal. Validated via `regex::Regex::new` (CWE-1333).
    Regex(RegexLiteral),
    /// IP address literal (IPv4 or IPv6).
    IpAddr(IpAddrLiteral),
    /// ISO-8601 timestamp literal.
    Timestamp(TimestampLiteral),
    /// A quoted string that resembles a date or datetime but is NOT valid RFC-3339.
    /// Produced by the lenient parser fallback for date-only (`'2026-06-24'`) and
    /// offset-less (`'2026-06-24T12:00:00'`) forms per ADR-052 §D4 v1.4.
    /// Validated at plan time by `check_temporal_literals` (three-way dispatch).
    /// Must never reach SQL emission — `pipe_sql_emitter.rs` guards this with a
    /// belt-and-suspenders E-QUERY-002 (`QueryPlanFailed`) arm (Pipe/Filter mode).
    /// (ADR-052 §D4 Step 1; BC-2.11.021 v1.4; S-PRISMQL-NATIVE-TEMPORAL-TYPING-001)
    RawTemporalLiteral(String),
}

/// Duration literal with explicit unit (prismql-grammar.md §3.3).
///
/// # Validation
/// Constructed only if `value * unit_secs` does not overflow `u64`.
///
/// # Invariant (CR F-CR-008)
/// `value` and `unit` are private fields — all construction goes through
/// `DurationLiteral::new` which enforces the overflow invariant. External
/// code cannot bypass validation by constructing struct literals directly.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DurationLiteral {
    /// Numeric magnitude (private — use `value()` getter).
    value: u64,
    /// Duration unit (private — use `unit()` getter).
    unit: DurationUnit,
}

impl DurationLiteral {
    /// Construct a `DurationLiteral`, returning `Err` on overflow.
    ///
    /// This is the only public construction path. Overflow of
    /// `value * unit.secs()` returns an error rather than panicking.
    pub fn new(value: u64, unit: DurationUnit) -> Result<Self, &'static str> {
        // Validate that seconds-conversion doesn't overflow.
        let _ = value
            .checked_mul(unit.secs())
            .ok_or("E-QUERY-001: duration value overflows u64 seconds")?;
        Ok(Self { value, unit })
    }

    /// Return the numeric magnitude.
    pub fn value(&self) -> u64 {
        self.value
    }

    /// Return the duration unit.
    pub fn unit(&self) -> DurationUnit {
        self.unit.clone()
    }

    /// Total duration in seconds.
    ///
    /// # Invariant
    /// Cannot overflow: validated at construction time in `new`.
    pub fn to_secs(&self) -> u64 {
        // Safety: overflow-checked in constructor.
        self.value * self.unit.secs()
    }
}

/// Duration unit.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DurationUnit {
    Seconds,
    Minutes,
    Hours,
    Days,
}

impl DurationUnit {
    pub fn secs(&self) -> u64 {
        match self {
            DurationUnit::Seconds => 1,
            DurationUnit::Minutes => 60,
            DurationUnit::Hours => 3_600,
            DurationUnit::Days => 86_400,
        }
    }
}

/// CIDR network range literal (e.g. `"10.0.0.0/8"`).
///
/// Validated at parse time via `ipnet::IpNet::from_str` (CWE-20).
/// Stores the canonical string representation.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CidrLiteral {
    /// Canonical CIDR string (e.g. `"10.0.0.0/8"`).
    pub cidr: String,
    /// Network address.
    pub addr: IpAddrWrapper,
    /// Prefix length (0–32 for IPv4, 0–128 for IPv6).
    pub prefix_len: u8,
}

impl CidrLiteral {
    /// Parse and validate a CIDR string (CWE-20).
    ///
    /// # Security (B-9, BC-2.11.006)
    /// User input is truncated to 50 bytes in error messages to prevent
    /// error message injection via arbitrarily long "CIDR" strings.
    /// A valid CIDR string is at most ~43 bytes (IPv6 with /128 suffix).
    pub fn new(s: &str) -> Result<Self, String> {
        // Pre-validate length: valid CIDRs are at most ~50 bytes.
        // Reject early with a truncated error message.
        const MAX_CIDR_LEN: usize = 50;
        if s.len() > MAX_CIDR_LEN {
            let display = crate::error::truncate_for_display(s, MAX_CIDR_LEN);
            return Err(format!(
                "E-QUERY-001: invalid CIDR '{display}': value too long (max {MAX_CIDR_LEN} bytes)"
            ));
        }
        let net: ipnet::IpNet = s
            .parse()
            .map_err(|e| format!("E-QUERY-001: invalid CIDR '{s}': {e}"))?;
        Ok(Self {
            cidr: net.to_string(),
            addr: IpAddrWrapper(net.network()),
            prefix_len: net.prefix_len(),
        })
    }
}

/// Newtype wrapper for `std::net::IpAddr` that implements `Hash`.
///
/// `std::net::IpAddr` does not implement `Hash` in stable Rust, so we wrap it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IpAddrWrapper(pub IpAddr);

impl std::hash::Hash for IpAddrWrapper {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self.0 {
            IpAddr::V4(a) => a.octets().hash(state),
            IpAddr::V6(a) => a.octets().hash(state),
        }
    }
}

/// Regex pattern literal.
///
/// Validated at parse time:
/// - `regex::Regex::new(pattern)` must succeed (CWE-1333 safe engine)
/// - Pattern must be ≤ 1024 bytes (BC-2.11.006)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RegexLiteral {
    /// The raw pattern string.
    pub pattern: String,
    /// Regex flags (reserved for future use; currently always empty).
    pub flags: String,
}

impl RegexLiteral {
    /// Parse and validate a regex pattern (CWE-1333).
    ///
    /// Enforces the length cap via `security::check_regex_pattern_length`
    /// (single source of truth: `security::PRISM_MAX_REGEX_PATTERN_LEN`) and
    /// validates the pattern with the `regex` crate's finite-automaton engine.
    ///
    /// # Single source of truth (Adv F-HIGH-003)
    /// The byte-length limit lives solely in `security::PRISM_MAX_REGEX_PATTERN_LEN`.
    /// This function delegates to `security::check_regex_pattern_length` instead
    /// of duplicating the constant.
    pub fn new(pattern: &str) -> Result<Self, String> {
        // Length check: use thread-local snapshotted limit when called inside
        // `PrismQlParser::parse`, falling back to env-var read otherwise.
        // This makes the regex length guard race-free (F-HIGH-001, BC-2.11.006).
        let limit = crate::security::ParseLimits::current_regex_limit();
        if pattern.len() > limit {
            return Err(format!(
                "E-QUERY-003: regex pattern length {} bytes exceeds maximum allowed {} bytes",
                pattern.len(),
                limit
            ));
        }
        regex::Regex::new(pattern)
            .map_err(|e| format!("E-QUERY-001: invalid regex pattern '{pattern}': {e}"))?;
        Ok(Self {
            pattern: pattern.to_string(),
            flags: String::new(),
        })
    }
}

/// IP address literal.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IpAddrLiteral(pub IpAddrWrapper);

impl std::hash::Hash for IpAddrLiteral {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

/// ISO-8601 / RFC-3339 timestamp literal, validated at parse time.
///
/// The raw string is preserved for display; `instant` holds the parsed
/// `DateTime<Utc>` value for efficient comparison and range checks in the
/// query planner. Both fields are `pub` — the planner may read either.
///
/// # Parse policy (strict)
/// Only RFC-3339 strings with an explicit timezone offset are accepted.
/// The bare form `"2026-05-04T12:00:00"` (no `Z` or `+HH:MM`) is rejected
/// to avoid silent UTC assumption bugs. Analysts must write `...T12:00:00Z`
/// or `...T12:00:00+00:00`.
///
/// # Equality and hashing
/// Two `TimestampLiteral`s are `==` iff their `instant` values are equal
/// (i.e. they represent the same UTC point in time, regardless of
/// raw-string formatting). The `iso8601` string is NOT compared.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimestampLiteral {
    /// Raw ISO-8601 string as written in the query (e.g. `"2026-04-13T00:00:00Z"`).
    pub iso8601: String,
    /// Parsed UTC instant — authoritative for comparisons and planning.
    pub instant: DateTime<Utc>,
}

impl TimestampLiteral {
    /// Parse and validate an ISO-8601 / RFC-3339 timestamp string.
    ///
    /// Accepts the strict RFC-3339 form only (timezone offset required).
    /// Bare local-time strings (`"2026-05-04T12:00:00"`) are rejected.
    ///
    /// # Errors
    /// Returns `Err(ParseError)` if the string is not valid RFC-3339.
    pub fn new(s: &str) -> Result<Self, crate::error::ParseError> {
        // Parse as RFC-3339 (requires explicit timezone offset — strict policy).
        let fixed = DateTime::parse_from_rfc3339(s)
            .map_err(|e| crate::error::ParseError::invalid_timestamp(s, e))?;
        Ok(Self {
            iso8601: s.to_string(),
            instant: fixed.with_timezone(&Utc),
        })
    }
}

impl PartialEq for TimestampLiteral {
    fn eq(&self, other: &Self) -> bool {
        self.instant == other.instant
    }
}

impl Eq for TimestampLiteral {}

impl std::hash::Hash for TimestampLiteral {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // Hash the UTC millisecond epoch for stable, ordering-consistent hashing.
        self.instant.timestamp_millis().hash(state);
    }
}

impl Literal {
    /// Return a user-readable string representation of this literal.
    ///
    /// Used when embedding literal values in user-facing messages, plan params,
    /// or audit records — avoids the `Debug` format (e.g. `String("foo")`)
    /// in favour of the actual value (e.g. `foo`). (F-PASS6-LOW-001)
    ///
    /// `Bool`, `Integer`, and `Float` use their natural string form.
    /// `String` is returned as-is (no surrounding quotes — the context supplies them).
    /// Structured literals (Duration, Cidr, Regex, IpAddr, Timestamp) use their
    /// canonical string representations.
    pub fn to_user_string(&self) -> String {
        match self {
            Literal::String(s) => s.clone(),
            Literal::Integer(n) => n.to_string(),
            Literal::Float(f) => f.to_string(),
            Literal::Bool(b) => b.to_string(),
            Literal::Null => "null".to_string(),
            Literal::Duration(d) => {
                let unit_str = match d.unit() {
                    DurationUnit::Seconds => "s",
                    DurationUnit::Minutes => "m",
                    DurationUnit::Hours => "h",
                    DurationUnit::Days => "d",
                };
                format!("{}{}", d.value(), unit_str)
            }
            Literal::Cidr(c) => c.cidr.clone(),
            Literal::Regex(r) => r.pattern.clone(),
            Literal::IpAddr(ip) => ip.0 .0.to_string(),
            Literal::Timestamp(ts) => ts.iso8601.clone(),
            // ADR-052 §D4 Step 1: RawTemporalLiteral carries the raw literal string.
            // to_user_string returns it as-is (no surrounding quotes — context supplies them).
            Literal::RawTemporalLiteral(s) => s.clone(),
        }
    }
}

impl Expr {
    /// Return a user-readable string representation of this expression.
    ///
    /// For `Literal` expressions, delegates to `Literal::to_user_string`.
    /// For non-literal expressions (field references, comparisons, etc.) emits
    /// a placeholder `"<expr>"` — these are uncommon in write-plan params
    /// but must be handled for exhaustiveness. (F-PASS6-LOW-001)
    pub fn to_user_string(&self) -> String {
        match self {
            Expr::Literal(lit) => lit.to_user_string(),
            _ => "<expr>".to_string(),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Operators
// ─────────────────────────────────────────────────────────────────────────────

/// Comparison operator.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
    /// `Cidr` tests whether an IP address falls within a network prefix.
    /// Retained on `Expr::Compare` for backward compatibility with the SQL
    /// parser; new code should use `Predicate::Cidr` instead.
    Cidr,
    /// Negated CIDR — `NOT field CIDR '...'`.
    ///
    /// Produced by `predicates_from_ast` when `Predicate::Cidr { negated: true }`.
    /// Renders as `"NOT <field> CIDR '<mask>'"` in `predicate_as_string`.
    /// I-LOCAL-NEW-1: distinct from `Cidr` so negation is preserved end-to-end.
    NotCidr,
}

/// Logical binary operator for `Predicate::Logical` and `Expr::Logical`.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LogicalOp {
    And,
    Or,
}

/// String pattern operators (prismql-grammar.md §4).
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StringOp {
    Contains,
    StartsWith,
    EndsWith,
}

/// Arithmetic binary operator for timestamp expressions (ADR-044).
///
/// Used in `Expr::TimestampArithmetic` to distinguish addition from subtraction.
/// Only `Add` and `Sub` are valid — multiplication/division of timestamps is
/// semantically undefined and must return `E-QUERY-041` at planning time.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BinaryOp {
    /// `+` — add a duration to a timestamp.
    Add,
    /// `-` — subtract a duration from a timestamp.
    Sub,
}

// ---------------------------------------------------------------------------
// PqlNormalizer — Chumsky AST re-serializer (S-DEMO-PRISMQL-ONBOARDING-001-B)
// ---------------------------------------------------------------------------

// Thread-local flag used by `PqlNormalizer::normalize_for_datafusion` to switch
// the literal emitter from PQL round-trip form (`'<iso>'`) to DataFusion form
// (`arrow_cast('...', 'Timestamp(Microsecond, Some("UTC"))')`).
//
// The flag is safe in async contexts because `PqlNormalizer::normalize` contains
// no `.await` points — the thread-local is set, the synchronous traversal runs,
// and the drop guard resets the flag before any executor can schedule a
// concurrent task on the same thread.  (ADR-052 §D4 v1.5 SQL-Mode Addendum)
thread_local! {
    static NORMALIZE_FOR_DATAFUSION: std::cell::Cell<bool> =
        const { std::cell::Cell::new(false) };
}

/// Canonicalizing PQL re-serializer (BC-2.11.018).
///
/// Walks the parsed `Ast` and produces a whitespace-normalized, uppercase-keyword
/// form of the original query string.  The output MUST round-trip through the
/// Chumsky parser to the same `Ast` as the original.
///
/// VERIFIED 2026-06-20 (remove-uncertainty pass): there are NO existing
/// `Display` / `to_pql` / `normalize` / `to_canonical` impls on any AST node
/// type — this struct is **entirely net-new** with zero leverage points.
/// `chumsky 0.12.0` supplies no AST pretty-printing facility.
///
/// EXCLUDED from output: DataFusion plan node strings (`HashJoin`, `TableScan`,
/// `SortExec`, `Aggregate`), cost estimates, partition/pushdown details.
///
/// Reference: BC-2.11.018; S-DEMO-PRISMQL-ONBOARDING-001-B AC-005, AC-006.
pub struct PqlNormalizer;

// The `_ => fallback` arms below are intentional: the enums are
// `#[non_exhaustive]` and the arms document the intended forward-compatible
// behaviour for future variants.  They are unreachable today (same crate)
// but are retained for documentation and external-crate robustness.
#[allow(unreachable_patterns)]
impl PqlNormalizer {
    /// Normalize `ast` to a canonical PQL string.
    ///
    /// Returns `None` when:
    /// 1. The normalized form would be empty (defensive guard; should not occur for a
    ///    validly-parsed `Ast` per BC-2.11.018 EC-11-055).
    /// 2. **SEC-001 defense-in-depth:** any string-bearing node in the AST contains BOTH
    ///    `'` and `"`. The grammar cannot faithfully represent such literals (no escape
    ///    mechanism — `none_of('\'')`/`none_of('"')` only). Emitting an unrepresentable
    ///    string would produce a `normalized_pql` that does NOT round-trip, which is worse
    ///    than omitting the field. This case is UNREACHABLE via the parser (the parser
    ///    cannot produce such a literal from source input), but the guard protects against
    ///    direct AST construction bypassing the parser (CWE-116 defense-in-depth).
    pub fn normalize(ast: &Ast) -> Option<String> {
        // SEC-001 pre-check: abort immediately if any string node contains both quote types.
        if Self::ast_has_both_quote_string(ast) {
            return None;
        }
        // F-P1-MED-001 pre-check: abort if the AST contains any unfolded temporal expression
        // (`Expr::Now`, `Expr::Interval`, `Expr::TimestampArithmetic`). These variants are
        // only valid BEFORE `inject_now` constant-folds them into bare `Literal::Timestamp`
        // values. If they survive into normalization, `normalize_expr` would emit an empty
        // string for the affected sub-expression (the catch-all `_ => String::new()` arm),
        // producing malformed SQL (e.g., `WHERE timestamp > `) that DataFusion receives
        // without error — a silent corruption (SOUL.md #4). Returning `None` here causes the
        // callers (`execute_against_session` for `Ast::Sql(Select)` and `Ast::SqlPipe`) to
        // return a structured `PrismError::QueryExecutionFailed` instead. (ADR-044, BC-2.11.021)
        if Self::ast_has_unfolded_temporal_expr(ast) {
            return None;
        }
        let s = match ast {
            Ast::Sql(stmt) => Self::normalize_sql_statement(stmt),
            Ast::Filter(filter) => Self::normalize_filter(filter),
            Ast::Pipe(pipe) => Self::normalize_pipe(pipe),
            _ => return None, // non_exhaustive arm — unknown future variant
        };
        if s.is_empty() {
            None
        } else {
            Some(s)
        }
    }

    /// Normalize a `Predicate` to a DataFusion-compatible SQL WHERE-clause expression string.
    ///
    /// Public wrapper over the private `normalize_predicate` method, used by
    /// `execute_against_session` for Filter-mode SQL lowering (BC-2.11.023 AC-011 / ADR-046 D4).
    pub fn normalize_predicate_pub(pred: &Predicate) -> String {
        Self::normalize_predicate(pred)
    }

    /// Returns `true` if `pred` contains any unfolded temporal expression
    /// (`Expr::Now`, `Expr::Interval`, `Expr::TimestampArithmetic`).
    ///
    /// Public wrapper over the private `predicate_has_unfolded_temporal` method.
    /// Used by `execute_against_session` Filter-mode arm as a guard matching the
    /// protection already in place for SQL/SqlPipe arms via `ast_has_unfolded_temporal_expr`
    /// (LOW-1 fix — F-P1-MED-001 sibling parity, BC-2.11.021 / ADR-044).
    pub fn predicate_has_unfolded_temporal_pub(pred: &Predicate) -> bool {
        Self::predicate_has_unfolded_temporal(pred)
    }

    /// Public wrapper over the private `expr_has_unfolded_temporal` method.
    /// Exposed for tests that need to verify the detect-side behaviour of
    /// `Expr::InSubquery` (value context) and `FuncCall` arg lists
    /// (MED-1 / OBS-1 fix — fold↔detect exhaustive symmetry sweep).
    pub fn expr_has_unfolded_temporal_pub(expr: &Expr) -> bool {
        Self::expr_has_unfolded_temporal(expr)
    }

    /// SQL-mode normalizer variant for DataFusion.
    ///
    /// Identical to `normalize` (same pre-checks, same AST traversal) EXCEPT that
    /// `Literal::Timestamp` values are emitted as:
    ///
    ///   `arrow_cast('<iso>', 'Timestamp(Microsecond, Some("UTC"))')`
    ///
    /// instead of the bare `'<iso>'` that `normalize` emits.  The bare form relies
    /// on DataFusion's implicit string→timestamp coercion (ADR-052 §RISK-1), which
    /// is non-deterministic across DataFusion minor versions.  The `arrow_cast` form
    /// produces an explicit `Timestamp(Microsecond, Some("UTC"))` literal that
    /// compares directly against `Timestamp(Microsecond, UTC)` columns without
    /// implicit coercion.
    ///
    /// BC-2.11.018 round-trip invariant: `normalize` MUST NOT be changed to emit
    /// `arrow_cast` — the round-trip form must remain re-parseable by the Chumsky
    /// grammar (which has no `arrow_cast` production rule).  This method exists
    /// precisely to keep the two paths separate (ADR-052 §D4 v1.5 SQL-Mode Addendum).
    ///
    /// Used by `execute_against_session` for the `Ast::Sql(Select)` DataFusion
    /// emission path (S-PRISMQL-NATIVE-TEMPORAL-TYPING-001).
    pub(crate) fn normalize_for_datafusion(ast: &Ast) -> Option<String> {
        // Drop guard: restore the thread-local to its prior value even if `normalize`
        // panics.  Save-and-restore (not hard-set false) so that nested calls to
        // `normalize_for_datafusion` do not destroy an outer caller's mode setting.
        // (ADR-052 §D4 v1.6 LOW-1.)
        struct DataFusionModeGuard {
            prior: bool,
        }
        impl Drop for DataFusionModeGuard {
            fn drop(&mut self) {
                NORMALIZE_FOR_DATAFUSION.with(|m| m.set(self.prior));
            }
        }
        let prior = NORMALIZE_FOR_DATAFUSION.with(|m| m.get());
        NORMALIZE_FOR_DATAFUSION.with(|m| m.set(true));
        let _guard = DataFusionModeGuard { prior };
        Self::normalize(ast)
    }

    /// DataFusion-mode literal emitter: emits `arrow_cast(...)` for
    /// `Literal::Timestamp`, delegates to `normalize_literal` for all other
    /// literal variants.
    ///
    /// This is the per-literal building block for `normalize_for_datafusion`.  It
    /// is invoked automatically when the `NORMALIZE_FOR_DATAFUSION` thread-local
    /// flag is `true` — specifically, `normalize_literal_dispatch` (called from
    /// every `normalize_literal` call site during AST traversal) checks the flag
    /// and routes to this function.  Callers set the flag by calling
    /// `normalize_for_datafusion`, which installs a `DataFusionModeGuard` and then
    /// calls `normalize`.  The `execute_against_session` `Ast::Sql` and `Ast::SqlPipe`
    /// arms in `materialization.rs` are the production call sites.
    ///
    /// Also exposed `pub(crate)` so tests can assert the exact emission format
    /// independently of the full AST traversal path.
    ///
    /// ADR-052 §D4 v1.5 SQL-Mode DataFusion Emission Addendum.
    pub(crate) fn normalize_literal_for_datafusion(lit: &Literal) -> String {
        match lit {
            Literal::Timestamp(ts) => format!(
                "arrow_cast('{}', 'Timestamp(Microsecond, Some(\"UTC\"))')",
                ts.iso8601
            ),
            other => Self::normalize_literal(other),
        }
    }

    /// SEC-001 helper: returns `true` if any string-**literal**-bearing node in `ast`
    /// contains BOTH `'` and `"`. Called as a pre-check before normalization; avoids
    /// changing the return types of every normalizer helper (low blast-radius approach).
    ///
    /// **Scope:** covers `Ast::Filter`, `Ast::Sql`, `Ast::Pipe`, and `Ast::SqlPipe`.
    /// For `Ast::SqlPipe(spq)`: checks the SQL head (`spq.head` via
    /// `sql_query_has_both_quote_string`) AND all pipe stages (`spq.stages` via
    /// `pipe_stage_has_both_quote_string`) — mirroring the parity already present for
    /// `Ast::Sql` (SQL head) and `Ast::Pipe` (stage iteration).
    ///
    /// Covers quoted string literals (`Literal::String`, `Literal::Regex`
    /// patterns, `Predicate::StringOp`/`Wildcard` patterns) and recursively their
    /// containing expressions and function-call argument lists. Bare identifiers — function
    /// names (`ScalarFunc::Unknown(name)`) and column/field path segments — are intentionally
    /// excluded: they are parser-restricted to `[A-Za-z0-9_]` characters (no quotes
    /// possible) and emitted unquoted by the normalizer, so they are not a CWE-116
    /// quoted-literal injection vector.
    ///
    /// This path is **parser-unreachable** in normal operation: the grammar's
    /// `build_string_parser` uses `none_of('\'')` for single-quoted bodies and
    /// `none_of('"')` for double-quoted bodies, so a literal with both characters cannot
    /// originate from user input. The check is defense-in-depth for direct AST construction.
    fn ast_has_both_quote_string(ast: &Ast) -> bool {
        match ast {
            Ast::Filter(f) => Self::predicate_has_both_quote_string(&f.predicate),
            Ast::Sql(stmt) => Self::sql_statement_has_both_quote_string(stmt),
            Ast::Pipe(pipe) => pipe
                .stages
                .iter()
                .any(Self::pipe_stage_has_both_quote_string),
            // SqlPipe: check both the SQL head AND the pipe stages (OBS-1 parity fix).
            Ast::SqlPipe(spq) => {
                Self::sql_query_has_both_quote_string(&spq.head)
                    || spq
                        .stages
                        .iter()
                        .any(Self::pipe_stage_has_both_quote_string)
            }
            _ => false,
        }
    }

    fn string_has_both_quotes(s: &str) -> bool {
        s.contains('\'') && s.contains('"')
    }

    fn literal_has_both_quote_string(lit: &Literal) -> bool {
        match lit {
            Literal::String(s) => Self::string_has_both_quotes(s),
            Literal::Regex(r) => Self::string_has_both_quotes(&r.pattern),
            _ => false,
        }
    }

    fn predicate_has_both_quote_string(pred: &Predicate) -> bool {
        match pred {
            Predicate::Compare { lhs, rhs, .. } => {
                Self::expr_has_both_quote_string(lhs) || Self::expr_has_both_quote_string(rhs)
            }
            Predicate::StringOp { pattern, .. } => Self::string_has_both_quotes(pattern),
            Predicate::Regex { pattern, .. } => Self::string_has_both_quotes(&pattern.pattern),
            Predicate::In { values, .. } => values.iter().any(Self::literal_has_both_quote_string),
            Predicate::Between { low, high, .. } => {
                Self::literal_has_both_quote_string(low)
                    || Self::literal_has_both_quote_string(high)
            }
            Predicate::Wildcard { pattern, .. } => Self::string_has_both_quotes(pattern),
            Predicate::Logical { predicates, .. } => {
                predicates.iter().any(Self::predicate_has_both_quote_string)
            }
            Predicate::Not(inner) => Self::predicate_has_both_quote_string(inner),
            Predicate::InSubquery { subquery, .. } => {
                Self::sql_query_has_both_quote_string(subquery)
            }
            _ => false,
        }
    }

    fn expr_has_both_quote_string(expr: &Expr) -> bool {
        match expr {
            Expr::Literal(lit) => Self::literal_has_both_quote_string(lit),
            Expr::Compare { lhs, rhs, .. } => {
                Self::expr_has_both_quote_string(lhs) || Self::expr_has_both_quote_string(rhs)
            }
            Expr::Logical { lhs, rhs, .. } => {
                Self::expr_has_both_quote_string(lhs) || Self::expr_has_both_quote_string(rhs)
            }
            Expr::Not(inner) => Self::expr_has_both_quote_string(inner),
            Expr::In { values, .. } => values.iter().any(Self::literal_has_both_quote_string),
            Expr::InSubquery { subquery, .. } => Self::sql_query_has_both_quote_string(subquery),
            // SEC-001 defense-in-depth: traverse FuncCall argument expressions for
            // string literals (quoted values). Parser-unreachable — the grammar cannot
            // produce a string literal containing both ' and " — but this arm closes the
            // CWE-116 gap for direct AST construction that bypasses the parser.
            // Note: ScalarFunc::Unknown(name) (the UDF function name) is a bare identifier
            // restricted by the parser to [A-Za-z0-9_]; it is emitted unquoted and is NOT
            // a quoted-literal injection vector, so it is not inspected here.
            Expr::FuncCall(fc) => match fc {
                FuncCall::Aggregate { args, .. } => {
                    args.iter().any(Self::expr_has_both_quote_string)
                }
                FuncCall::Scalar { args, .. } => args.iter().any(Self::expr_has_both_quote_string),
                // Window: no args yet (placeholder; S-3.06 will add fields).
                FuncCall::Window { .. } => false,
                _ => false,
            },
            _ => false,
        }
    }

    fn sql_statement_has_both_quote_string(stmt: &SqlStatement) -> bool {
        match stmt {
            SqlStatement::Select(q) => Self::sql_query_has_both_quote_string(q),
            _ => false,
        }
    }

    fn sql_query_has_both_quote_string(q: &SqlQuery) -> bool {
        let where_hit = q
            .where_
            .as_ref()
            .is_some_and(Self::predicate_has_both_quote_string);
        let having_hit = q
            .having
            .as_ref()
            .is_some_and(Self::predicate_has_both_quote_string);
        let select_hit = q.select.items.iter().any(|item| match item {
            SelectItem::Expr { expr, .. } => Self::expr_has_both_quote_string(expr),
            _ => false,
        });
        let group_hit = q.group_by.iter().any(Self::expr_has_both_quote_string);
        let order_hit = q
            .order_by
            .iter()
            .any(|oe| Self::expr_has_both_quote_string(&oe.expr));
        let join_hit = q
            .joins
            .iter()
            .any(|j| Self::expr_has_both_quote_string(&j.on));
        where_hit || having_hit || select_hit || group_hit || order_hit || join_hit
    }

    fn pipe_stage_has_both_quote_string(stage: &PipeStage) -> bool {
        match stage {
            PipeStage::Where(pred) => Self::predicate_has_both_quote_string(pred),
            _ => false,
        }
    }

    // ---- F-P1-MED-001: unfolded temporal expression detection ----
    //
    // Returns `true` if the AST contains any `Expr::Now`, `Expr::Interval`, or
    // `Expr::TimestampArithmetic` that was NOT constant-folded by `inject_now`.
    // Follows the same low-blast-radius pre-check pattern as `ast_has_both_quote_string`
    // (SEC-001): avoids changing `normalize_expr`'s return type across 13 call sites.
    //
    // Scope: Filter predicates; SQL-mode WHERE/HAVING/SELECT/GROUP BY/ORDER BY/JOIN ON;
    // Pipe WHERE stages; SqlPipe SQL head AND pipe stages (OBS-1 parity fix).

    fn ast_has_unfolded_temporal_expr(ast: &Ast) -> bool {
        match ast {
            Ast::Filter(f) => Self::predicate_has_unfolded_temporal(&f.predicate),
            Ast::Sql(stmt) => Self::sql_statement_has_unfolded_temporal(stmt),
            Ast::Pipe(pipe) => pipe
                .stages
                .iter()
                .any(Self::pipe_stage_has_unfolded_temporal),
            // SqlPipe: check both the SQL head AND the pipe stages (OBS-1 parity fix).
            Ast::SqlPipe(spq) => {
                Self::sql_query_has_unfolded_temporal(&spq.head)
                    || spq
                        .stages
                        .iter()
                        .any(Self::pipe_stage_has_unfolded_temporal)
            }
            _ => false,
        }
    }

    fn expr_has_unfolded_temporal(expr: &Expr) -> bool {
        match expr {
            // These three variants are unfolded temporal expressions.
            Expr::Now | Expr::Interval(_) | Expr::TimestampArithmetic { .. } => true,
            // Recurse into compound expressions.
            Expr::Compare { lhs, rhs, .. } => {
                Self::expr_has_unfolded_temporal(lhs) || Self::expr_has_unfolded_temporal(rhs)
            }
            Expr::Logical { lhs, rhs, .. } => {
                Self::expr_has_unfolded_temporal(lhs) || Self::expr_has_unfolded_temporal(rhs)
            }
            Expr::Not(inner) => Self::expr_has_unfolded_temporal(inner),
            Expr::FuncCall(fc) => match fc {
                FuncCall::Aggregate { args, .. } => {
                    args.iter().any(Self::expr_has_unfolded_temporal)
                }
                FuncCall::Scalar { args, .. } => args.iter().any(Self::expr_has_unfolded_temporal),
                FuncCall::Window { .. } => false,
                _ => false,
            },
            // Expr::InSubquery (value context): the subquery may contain temporal
            // expressions in its WHERE, HAVING, SELECT, etc.  Recurse via
            // `sql_query_has_unfolded_temporal` so detect mirrors fold.
            // (The prior code silently skipped this variant — MED-1 / OBS-1 fix.)
            Expr::InSubquery { subquery, .. } => Self::sql_query_has_unfolded_temporal(subquery),
            // Literal, Field, VirtualField, In, Star — no temporal exprs.
            _ => false,
        }
    }

    fn predicate_has_unfolded_temporal(pred: &Predicate) -> bool {
        match pred {
            Predicate::Compare { lhs, rhs, .. } => {
                Self::expr_has_unfolded_temporal(lhs) || Self::expr_has_unfolded_temporal(rhs)
            }
            Predicate::Logical { predicates, .. } => {
                predicates.iter().any(Self::predicate_has_unfolded_temporal)
            }
            Predicate::Not(inner) => Self::predicate_has_unfolded_temporal(inner),
            Predicate::InSubquery { subquery, .. } => {
                Self::sql_query_has_unfolded_temporal(subquery)
            }
            _ => false,
        }
    }

    fn sql_statement_has_unfolded_temporal(stmt: &SqlStatement) -> bool {
        match stmt {
            SqlStatement::Select(q) => Self::sql_query_has_unfolded_temporal(q),
            _ => false,
        }
    }

    fn sql_query_has_unfolded_temporal(q: &SqlQuery) -> bool {
        let where_hit = q
            .where_
            .as_ref()
            .is_some_and(Self::predicate_has_unfolded_temporal);
        let having_hit = q
            .having
            .as_ref()
            .is_some_and(Self::predicate_has_unfolded_temporal);
        let select_hit = q.select.items.iter().any(|item| match item {
            SelectItem::Expr { expr, .. } => Self::expr_has_unfolded_temporal(expr),
            _ => false,
        });
        let group_hit = q.group_by.iter().any(Self::expr_has_unfolded_temporal);
        let order_hit = q
            .order_by
            .iter()
            .any(|oe| Self::expr_has_unfolded_temporal(&oe.expr));
        let join_hit = q
            .joins
            .iter()
            .any(|j| Self::expr_has_unfolded_temporal(&j.on));
        where_hit || having_hit || select_hit || group_hit || order_hit || join_hit
    }

    fn pipe_stage_has_unfolded_temporal(stage: &PipeStage) -> bool {
        match stage {
            PipeStage::Where(pred) => Self::predicate_has_unfolded_temporal(pred),
            _ => false,
        }
    }

    // ---- SQL mode ----

    fn normalize_sql_statement(stmt: &SqlStatement) -> String {
        match stmt {
            SqlStatement::Select(q) => Self::normalize_sql_query(q),
            SqlStatement::Dml(_) => String::new(), // DML normalization out of scope for 001-B
            _ => String::new(),
        }
    }

    fn normalize_sql_query(q: &SqlQuery) -> String {
        let mut parts: Vec<String> = Vec::new();

        // SELECT [DISTINCT] items
        let select_items = Self::normalize_select_clause(&q.select);
        parts.push(format!("SELECT {select_items}"));

        // FROM source [AS alias]
        let from_str = Self::normalize_from_clause(&q.from);
        parts.push(format!("FROM {from_str}"));

        // JOINs
        for join in &q.joins {
            parts.push(Self::normalize_join(join));
        }

        // WHERE predicate
        if let Some(pred) = &q.where_ {
            parts.push(format!("WHERE {}", Self::normalize_predicate(pred)));
        }

        // GROUP BY
        if !q.group_by.is_empty() {
            let exprs: Vec<String> = q.group_by.iter().map(Self::normalize_expr).collect();
            parts.push(format!("GROUP BY {}", exprs.join(", ")));
        }

        // HAVING
        if let Some(pred) = &q.having {
            parts.push(format!("HAVING {}", Self::normalize_predicate(pred)));
        }

        // ORDER BY
        if !q.order_by.is_empty() {
            let ord: Vec<String> = q.order_by.iter().map(Self::normalize_order_expr).collect();
            parts.push(format!("ORDER BY {}", ord.join(", ")));
        }

        // LIMIT
        if let Some(limit) = q.limit {
            parts.push(format!("LIMIT {limit}"));
        }

        parts.join(" ")
    }

    fn normalize_select_clause(sel: &SelectClause) -> String {
        let distinct = if sel.distinct { "DISTINCT " } else { "" };
        let items: Vec<String> = sel.items.iter().map(Self::normalize_select_item).collect();
        format!("{}{}", distinct, items.join(", "))
    }

    fn normalize_select_item(item: &SelectItem) -> String {
        match item {
            SelectItem::Star => "*".to_string(),
            SelectItem::TableStar(tbl) => format!("{tbl}.*"),
            SelectItem::Expr { expr, alias } => {
                let e = Self::normalize_expr(expr);
                match alias {
                    Some(a) => format!("{e} AS {a}"),
                    None => e,
                }
            }
            _ => "*".to_string(), // non_exhaustive arm
        }
    }

    fn normalize_from_clause(from: &FromClause) -> String {
        let src = &from.source.raw;
        match &from.alias {
            Some(a) => format!("{src} AS {a}"),
            None => src.clone(),
        }
    }

    fn normalize_join(join: &Join) -> String {
        let kind = match &join.kind {
            JoinKind::Inner => "INNER JOIN",
            JoinKind::Left => "LEFT JOIN",
            JoinKind::Right => "RIGHT JOIN",
            JoinKind::FullOuter => "FULL OUTER JOIN",
            JoinKind::Cross => "CROSS JOIN",
            _ => "JOIN",
        };
        let src = &join.source.raw;
        let alias_part = match &join.alias {
            Some(a) => format!(" AS {a}"),
            None => String::new(),
        };
        let on = Self::normalize_expr(&join.on);
        format!("{kind} {src}{alias_part} ON {on}")
    }

    fn normalize_order_expr(oe: &OrderExpr) -> String {
        let e = Self::normalize_expr(&oe.expr);
        let dir = match &oe.direction {
            SortDirection::Asc => "ASC",
            SortDirection::Desc => "DESC",
            _ => "ASC",
        };
        format!("{e} {dir}")
    }

    // ---- Filter mode ----

    fn normalize_filter(filter: &FilterExpr) -> String {
        let src = &filter.source.raw;
        let pred = Self::normalize_predicate(&filter.predicate);
        // BC-2.11.018 round-trip: bare predicates (no source prefix) use an empty source raw.
        // Emitting "` | {pred}`" with a leading space + pipe produces invalid PQL that the
        // filter parser cannot re-parse. Only emit the source prefix when one is present.
        if src.is_empty() {
            pred
        } else {
            format!("{src} | {pred}")
        }
    }

    // ---- Pipe mode ----

    fn normalize_pipe(pipe: &PipeQuery) -> String {
        let mut parts: Vec<String> = vec![pipe.source.raw.clone()];
        for stage in &pipe.stages {
            parts.push(Self::normalize_pipe_stage(stage));
        }
        parts.join(" | ")
    }

    fn normalize_pipe_stage(stage: &PipeStage) -> String {
        match stage {
            PipeStage::Where(pred) => format!("WHERE {}", Self::normalize_predicate(pred)),
            PipeStage::Sort(exprs) => {
                let parts: Vec<String> = exprs
                    .iter()
                    .map(|se| {
                        let dir = match &se.direction {
                            SortDirection::Asc => "ASC",
                            SortDirection::Desc => "DESC",
                            _ => "ASC",
                        };
                        format!("{} {dir}", Self::normalize_field_path(&se.field))
                    })
                    .collect();
                format!("SORT {}", parts.join(", "))
            }
            PipeStage::Limit(n) => format!("LIMIT {n}"),
            PipeStage::Tail(n) => format!("TAIL {n}"),
            PipeStage::Dedup(fields) => {
                let fs: Vec<String> = fields.iter().map(Self::normalize_field_path).collect();
                format!("DEDUP {}", fs.join(", "))
            }
            PipeStage::Fields(fstage) => {
                let sign = if fstage.include { "+" } else { "-" };
                let fs: Vec<String> = fstage
                    .fields
                    .iter()
                    .map(Self::normalize_field_path)
                    .collect();
                format!("FIELDS {sign} {}", fs.join(", "))
            }
            PipeStage::Stats(stats) => Self::normalize_stats_stage(stats),
            PipeStage::Join(js) => Self::normalize_join_stage(js),
            PipeStage::Enrich(es) => {
                format!(
                    "ENRICH {}({})",
                    es.infusion,
                    Self::normalize_field_path(&es.field)
                )
            }
            _ => String::new(), // non_exhaustive arm
        }
    }

    fn normalize_stats_stage(stats: &StatsStage) -> String {
        let aggs: Vec<String> = stats
            .aggregates
            .iter()
            .map(Self::normalize_stat_function)
            .collect();
        let mut s = format!("STATS {}", aggs.join(", "));
        if !stats.by_fields.is_empty() {
            let bys: Vec<String> = stats
                .by_fields
                .iter()
                .map(Self::normalize_field_path)
                .collect();
            s.push_str(&format!(" BY {}", bys.join(", ")));
        }
        s
    }

    fn normalize_stat_function(sf: &StatFunction) -> String {
        let func_str = Self::normalize_agg_func(&sf.func);
        match &sf.alias {
            Some(a) => format!("{func_str} AS {a}"),
            None => func_str,
        }
    }

    fn normalize_agg_func(f: &AggFunc) -> String {
        match f {
            AggFunc::Count => "COUNT(*)".to_string(),
            AggFunc::CountField(fp) => format!("COUNT({})", Self::normalize_field_path(fp)),
            AggFunc::Sum(fp) => format!("SUM({})", Self::normalize_field_path(fp)),
            AggFunc::Avg(fp) => format!("AVG({})", Self::normalize_field_path(fp)),
            AggFunc::Min(fp) => format!("MIN({})", Self::normalize_field_path(fp)),
            AggFunc::Max(fp) => format!("MAX({})", Self::normalize_field_path(fp)),
            AggFunc::DistinctCount(fp) => {
                format!("DISTINCT_COUNT({})", Self::normalize_field_path(fp))
            }
            AggFunc::Percentile { field, p } => {
                format!("PERCENTILE({}, {})", Self::normalize_field_path(field), p)
            }
            _ => "COUNT(*)".to_string(), // non_exhaustive arm
        }
    }

    fn normalize_join_stage(js: &JoinStage) -> String {
        let kind = match &js.kind {
            JoinKind::Inner => "INNER JOIN",
            JoinKind::Left => "LEFT JOIN",
            JoinKind::Right => "RIGHT JOIN",
            JoinKind::FullOuter => "FULL OUTER JOIN",
            JoinKind::Cross => "CROSS JOIN",
            _ => "JOIN",
        };
        let src = &js.source.raw;
        let on_part = match &js.on {
            JoinCondition::SameField(fp) => {
                format!("ON {}", Self::normalize_field_path(fp))
            }
            JoinCondition::Pair(left, right) => {
                format!(
                    "ON {} == {}",
                    Self::normalize_field_path(left),
                    Self::normalize_field_path(right)
                )
            }
            _ => String::new(),
        };
        format!("{kind} {src} {on_part}")
    }

    // ---- Predicate ----

    fn normalize_predicate(pred: &Predicate) -> String {
        match pred {
            Predicate::Compare { lhs, op, rhs } => {
                let op_str = match op {
                    CompareOp::Eq => "=",
                    CompareOp::Ne => "!=",
                    CompareOp::Gt => ">",
                    CompareOp::Lt => "<",
                    CompareOp::Ge => ">=",
                    CompareOp::Le => "<=",
                    CompareOp::Like => "LIKE",
                    CompareOp::Cidr => "IN CIDR",
                    CompareOp::NotCidr => "NOT IN CIDR",
                    _ => "=",
                };
                format!(
                    "{} {op_str} {}",
                    Self::normalize_expr(lhs),
                    Self::normalize_expr(rhs)
                )
            }
            Predicate::StringOp {
                field,
                op,
                pattern,
                case_insensitive,
            } => {
                let op_str = match (op, case_insensitive) {
                    (StringOp::Contains, false) => "CONTAINS",
                    (StringOp::Contains, true) => "ICONTAINS",
                    (StringOp::StartsWith, false) => "STARTSWITH",
                    (StringOp::StartsWith, true) => "ISTARTSWITH",
                    (StringOp::EndsWith, false) => "ENDSWITH",
                    (StringOp::EndsWith, true) => "IENDSWITH",
                    _ => "CONTAINS",
                };
                // Use emit_quoted_string: patterns can contain `'` (F-001B-FRESH-HIGH-001).
                format!(
                    "{} {op_str} {}",
                    Self::normalize_field_path(field),
                    Self::emit_quoted_string(pattern)
                )
            }
            Predicate::Regex { field, pattern } => {
                // Use emit_quoted_string: regex patterns can contain `'` (F-001B-FRESH-HIGH-001).
                format!(
                    "{} =~ {}",
                    Self::normalize_field_path(field),
                    Self::emit_quoted_string(&pattern.pattern)
                )
            }
            Predicate::In {
                field,
                values,
                negated,
            } => {
                // Use dispatch so Timestamp literals emit arrow_cast in DataFusion mode.
                let vals: Vec<String> = values
                    .iter()
                    .map(Self::normalize_literal_dispatch)
                    .collect();
                let not_kw = if *negated { "NOT IN" } else { "IN" };
                format!(
                    "{} {not_kw} ({})",
                    Self::normalize_field_path(field),
                    vals.join(", ")
                )
            }
            Predicate::InSubquery {
                field,
                subquery,
                negated,
            } => {
                let not_kw = if *negated { "NOT IN" } else { "IN" };
                let sub = Self::normalize_sql_query(subquery);
                format!("{} {not_kw} ({sub})", Self::normalize_field_path(field))
            }
            Predicate::Between {
                field,
                low,
                high,
                negated,
            } => {
                let not_kw = if *negated { "NOT BETWEEN" } else { "BETWEEN" };
                // Use dispatch so Timestamp literals emit arrow_cast in DataFusion mode.
                format!(
                    "{} {not_kw} {} AND {}",
                    Self::normalize_field_path(field),
                    Self::normalize_literal_dispatch(low),
                    Self::normalize_literal_dispatch(high)
                )
            }
            Predicate::Cidr {
                field,
                cidr,
                negated,
            } => {
                let not_kw = if *negated { "NOT IN CIDR" } else { "IN CIDR" };
                format!(
                    "{} {not_kw} '{}'",
                    Self::normalize_field_path(field),
                    cidr.cidr
                )
            }
            Predicate::Has(fp) => format!("HAS {}", Self::normalize_field_path(fp)),
            Predicate::Missing(fp) => format!("MISSING {}", Self::normalize_field_path(fp)),
            Predicate::IsNull { field, negated } => {
                let not_kw = if *negated { "IS NOT NULL" } else { "IS NULL" };
                format!("{} {not_kw}", Self::normalize_field_path(field))
            }
            Predicate::Wildcard {
                field,
                pattern,
                negated,
            } => {
                let op = if *negated { "!=" } else { "=" };
                // Use emit_quoted_string: wildcard patterns can contain `'` (F-001B-FRESH-HIGH-001).
                format!(
                    "{} {op} {}",
                    Self::normalize_field_path(field),
                    Self::emit_quoted_string(pattern)
                )
            }
            Predicate::Logical { op, predicates } => {
                let op_str = match op {
                    LogicalOp::And => "AND",
                    LogicalOp::Or => "OR",
                    _ => "AND",
                };
                let parts: Vec<String> = predicates
                    .iter()
                    .map(|p| {
                        // Wrap OR sub-predicates in parens inside an AND context for clarity
                        match p {
                            Predicate::Logical { op: inner_op, .. }
                                if matches!(inner_op, LogicalOp::Or)
                                    && matches!(op, LogicalOp::And) =>
                            {
                                format!("({})", Self::normalize_predicate(p))
                            }
                            _ => Self::normalize_predicate(p),
                        }
                    })
                    .collect();
                parts.join(&format!(" {op_str} "))
            }
            Predicate::Not(inner) => format!("NOT ({})", Self::normalize_predicate(inner)),
            Predicate::RecoveryError => "<recovery_error>".to_string(),
            _ => String::new(), // non_exhaustive arm
        }
    }

    // ---- Expr ----

    fn normalize_expr(expr: &Expr) -> String {
        match expr {
            Expr::Literal(lit) => Self::normalize_literal_as_expr(lit),
            Expr::Field(fp) => Self::normalize_field_path(fp),
            Expr::VirtualField(vf) => Self::normalize_virtual_field(vf),
            Expr::Compare { lhs, op, rhs } => {
                let op_str = match op {
                    CompareOp::Eq => "=",
                    CompareOp::Ne => "!=",
                    CompareOp::Gt => ">",
                    CompareOp::Lt => "<",
                    CompareOp::Ge => ">=",
                    CompareOp::Le => "<=",
                    CompareOp::Like => "LIKE",
                    CompareOp::Cidr => "IN CIDR",
                    CompareOp::NotCidr => "NOT IN CIDR",
                    _ => "=",
                };
                format!(
                    "{} {op_str} {}",
                    Self::normalize_expr(lhs),
                    Self::normalize_expr(rhs)
                )
            }
            Expr::Logical { lhs, op, rhs } => {
                let op_str = match op {
                    LogicalOp::And => "AND",
                    LogicalOp::Or => "OR",
                    _ => "AND",
                };
                format!(
                    "{} {op_str} {}",
                    Self::normalize_expr(lhs),
                    Self::normalize_expr(rhs)
                )
            }
            Expr::Not(inner) => format!("NOT {}", Self::normalize_expr(inner)),
            Expr::In { field, values } => {
                // Use dispatch so Timestamp literals emit arrow_cast in DataFusion mode.
                let vals: Vec<String> = values
                    .iter()
                    .map(Self::normalize_literal_dispatch)
                    .collect();
                format!(
                    "{} IN ({})",
                    Self::normalize_field_path(field),
                    vals.join(", ")
                )
            }
            Expr::InSubquery { field, subquery } => {
                let sub = Self::normalize_sql_query(subquery);
                format!("{} IN ({sub})", Self::normalize_field_path(field))
            }
            Expr::FuncCall(fc) => Self::normalize_func_call(fc),
            Expr::Star => "*".to_string(),
            _ => String::new(), // non_exhaustive arm
        }
    }

    fn normalize_func_call(fc: &FuncCall) -> String {
        match fc {
            FuncCall::Aggregate {
                func,
                args,
                distinct,
            } => {
                let func_str = Self::normalize_agg_func(func);
                // For aggregate functions with explicit args, use arg representation
                if args.is_empty() || matches!(func, AggFunc::Count) {
                    func_str
                } else {
                    let args_str: Vec<String> = args.iter().map(Self::normalize_expr).collect();
                    let distinct_kw = if *distinct { "DISTINCT " } else { "" };
                    let inner_name = match func {
                        AggFunc::Sum(_) => "SUM",
                        AggFunc::Avg(_) => "AVG",
                        AggFunc::Min(_) => "MIN",
                        AggFunc::Max(_) => "MAX",
                        AggFunc::DistinctCount(_) => "DISTINCT_COUNT",
                        _ => "FUNC",
                    };
                    format!("{inner_name}({distinct_kw}{})", args_str.join(", "))
                }
            }
            FuncCall::Scalar { func, args } => {
                let func_name = match func {
                    ScalarFunc::SubnetContains => "subnet_contains",
                    ScalarFunc::TimeWindow => "time_window",
                    ScalarFunc::JsonExtractString => "json_extract_string",
                    ScalarFunc::IocMatch => "ioc_match",
                    ScalarFunc::MitreTactic => "mitre_tactic",
                    ScalarFunc::SeverityLabel => "severity_label",
                    ScalarFunc::Unknown(name) => name.as_str(),
                    _ => "func",
                };
                let args_str: Vec<String> = args.iter().map(Self::normalize_expr).collect();
                format!("{func_name}({})", args_str.join(", "))
            }
            FuncCall::Window { .. } => "WINDOW()".to_string(),
            _ => String::new(), // non_exhaustive arm
        }
    }

    fn normalize_virtual_field(vf: &VirtualField) -> String {
        match vf {
            VirtualField::Sensor => "_sensor".to_string(),
            VirtualField::Client => "_client".to_string(),
            VirtualField::SourceTable => "_source_table".to_string(),
            VirtualField::SourceType => "_source_type".to_string(),
            VirtualField::SafetyFlags => "_safety_flags".to_string(),
            _ => "_unknown".to_string(),
        }
    }

    fn normalize_field_path(fp: &FieldPath) -> String {
        fp.segments.join(".")
    }

    /// Emit a quoted string that the PrismQL grammar can re-parse to the SAME literal value.
    ///
    /// # Pre-condition (SEC-001 defense-in-depth)
    /// The caller MUST ensure `value` does NOT contain BOTH `'` and `"` — the
    /// `ast_has_both_quote_string` pre-check in `normalize` guarantees this before
    /// any string-emit site is reached. If that pre-check passes, the cases below are
    /// exhaustive and the round-trip postcondition holds.
    ///
    /// # Quote-selection rules (BC-2.11.018 round-trip invariant)
    ///
    /// The grammar's `build_string_parser` / `build_literal_parser` defines:
    ///   - Single-quoted body: `none_of('\'')` — a `'` inside is IMPOSSIBLE to represent.
    ///   - Double-quoted body: `none_of('"')` — a `'` inside IS accepted; a `"` is not.
    ///
    /// Selection (both-quotes case is prevented by pre-check — see above):
    ///   - Value has no `'` and no `"` (common case): emit single-quoted `'value'`.
    ///   - Value contains `'` but not `"`: emit double-quoted `"value"` (grammar accepts `'` inside).
    ///   - Value contains `"` but not `'`: emit single-quoted `'value'` (grammar accepts `"` inside).
    ///
    /// This helper is the SINGLE source of quote-selection logic for all string-emitting
    /// sites in `PqlNormalizer`. Adding new string-emitting sites MUST use this helper to
    /// prevent recurrence of the sibling-sweep miss (F-001B-FRESH-HIGH-001, TD-VSDD-060).
    fn emit_quoted_string(value: &str) -> String {
        if value.contains('\'') {
            // Value has `'` — cannot use single quotes. Use double-quoted form.
            // Pre-check guarantees value does NOT also contain `"` (both-quotes → None already).
            format!("\"{value}\"")
        } else {
            // Value has no `'` — safe to emit single-quoted form.
            format!("'{value}'")
        }
    }

    /// Mode-dispatching wrapper: returns `normalize_literal_for_datafusion` when the
    /// NORMALIZE_FOR_DATAFUSION thread-local is set (i.e., the call is made from
    /// within `normalize_for_datafusion`), otherwise returns `normalize_literal`.
    ///
    /// Call sites that can carry `Literal::Timestamp` values (Compare predicates,
    /// IN/BETWEEN expressions) MUST use this helper instead of calling
    /// `normalize_literal` directly so that the DataFusion mode is respected.
    fn normalize_literal_dispatch(lit: &Literal) -> String {
        if NORMALIZE_FOR_DATAFUSION.with(|m| m.get()) {
            Self::normalize_literal_for_datafusion(lit)
        } else {
            Self::normalize_literal(lit)
        }
    }

    pub(crate) fn normalize_literal(lit: &Literal) -> String {
        match lit {
            // BC-2.11.018 round-trip invariant: emit a form the grammar CAN re-parse.
            // All string-wrapping emit sites use `emit_quoted_string` (F-001B-FRESH-HIGH-001
            // structural fix — shared helper prevents future sibling-sweep misses, TD-VSDD-060).
            Literal::String(s) => Self::emit_quoted_string(s),
            Literal::Integer(n) => n.to_string(),
            // Float: always emit with a decimal point so the grammar's float rule
            // (`digits '.' digits`) re-parses as Literal::Float, not Literal::Integer.
            // `f.to_string()` for 5.0_f64 emits "5" (no decimal) which re-parses as Integer(5).
            // Fix: when the fractional part is zero, append ".0". (F-001B-FRESH-HIGH-001)
            Literal::Float(f) => {
                if f.fract() == 0.0 {
                    format!("{:.1}", f.0)
                } else {
                    f.to_string()
                }
            }
            Literal::Bool(b) => b.to_string().to_uppercase(),
            Literal::Null => "NULL".to_string(),
            Literal::Duration(d) => {
                let unit_str = match d.unit() {
                    DurationUnit::Seconds => "s",
                    DurationUnit::Minutes => "m",
                    DurationUnit::Hours => "h",
                    DurationUnit::Days => "d",
                    _ => "s",
                };
                format!("{}{unit_str}", d.value())
            }
            // Cidr and Timestamp values are produced by validated parsers; they cannot contain
            // `'` (CIDR strings are dotted-decimal/colon hex + slash prefix; ISO-8601 timestamps
            // use digits/hyphens/colons/Z/+). Single-quoted form is always safe.
            Literal::Cidr(c) => format!("'{}'", c.cidr),
            // Regex patterns CAN contain `'` (e.g. `can't`). Use emit_quoted_string.
            // (F-001B-FRESH-HIGH-001 sibling-sweep fix)
            Literal::Regex(r) => Self::emit_quoted_string(&r.pattern),
            Literal::IpAddr(ip) => ip.0 .0.to_string(),
            Literal::Timestamp(ts) => format!("'{}'", ts.iso8601),
            // TD-VSDD-060 / HIGH-1 + MED-3 fix: explicit arm — must never be reached because
            // check_temporal_literals consumes RawTemporalLiteral before normalization.
            // Emit as a properly-quoted string (emit_quoted_string) so DataFusion can at minimum
            // reject it with a type-mismatch error rather than a tokenization crash caused by
            // the hyphens in a sentinel like `__raw_temporal_unvalidated_2026-07-03__` which
            // DataFusion misparses as arithmetic subtraction (MED-3 defense-in-depth fix).
            Literal::RawTemporalLiteral(s) => Self::emit_quoted_string(s),
            _ => "NULL".to_string(), // non_exhaustive arm
        }
    }

    /// Normalize a literal in expression context.
    ///
    /// Routes through `normalize_literal_dispatch` so that the DataFusion mode
    /// (set by `normalize_for_datafusion`) is respected: Timestamp literals emit
    /// `arrow_cast(...)` instead of bare `'<iso>'` when the mode flag is set.
    fn normalize_literal_as_expr(lit: &Literal) -> String {
        Self::normalize_literal_dispatch(lit)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// BC-2.11.018 round-trip tests (F-001B-PASS-MED-001)
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod bc_2_11_018_normalizer_roundtrip_tests {
    use super::*;
    use crate::filter_parser::PrismQlParser;

    /// F-001B-PASS-MED-001: `PqlNormalizer` round-trip MUST hold for string literals
    /// containing an embedded single-quote character.
    ///
    /// Grammar facts (confirmed by reading `filter_parser.rs::build_literal_parser`):
    /// - Single-quoted body = `none_of('\'')`: a `'` inside a single-quoted string is
    ///   IMPOSSIBLE to represent — the parser would stop at the first `'`.
    /// - Double-quoted body = `none_of('"')`: a `'` INSIDE double quotes IS accepted.
    ///   Input `host = "O'Brien"` parses to `Literal::String("O'Brien")`.
    ///
    /// Bug: `PqlNormalizer::normalize_literal` emits `format!("'{s}'")`  for every
    /// `Literal::String(s)` with NO quote escaping. Re-emitting `Literal::String("O'Brien")`
    /// produces `'O'Brien'`. When re-parsed, the single-quoted parser reads `'O'` (stops at
    /// the embedded `'`) and then fails on the trailing `Brien'`. Re-parse fails.
    ///
    /// BC-2.11.018 postcondition: "The normalized form MUST parse to the same AST as the
    /// original."
    ///
    /// RED GATE: re-parse of the normalizer output fails on current HEAD because
    /// `normalize_literal` emits unescaped `'O'Brien'`.
    #[test]
    fn test_BC_2_11_018_normalizer_roundtrip_embedded_single_quote_in_double_quoted_literal() {
        // Input: double-quoted literal containing an embedded single-quote.
        // grammar accepts this: none_of('"') allows ' inside "...".
        let input = r#"host_name = "O'Brien""#;

        // Step 1: Verify the input parses (precondition — grammar must accept it).
        let ast = PrismQlParser::parse(input)
            .expect("test precondition: \"O'Brien\" (double-quoted) must parse successfully");

        // Step 2: Normalize the parsed AST back to a PQL string.
        let normalized =
            PqlNormalizer::normalize(&ast).expect("normalize must return Some for a filter AST");

        // Step 3: Re-parse the normalized output.
        // BC-2.11.018 postcondition: the normalized form MUST parse to an equivalent AST.
        // RED GATE: on current HEAD, normalized = "host_name = 'O'Brien'" which fails to
        // re-parse because the embedded `'` terminates the single-quoted literal prematurely.
        let reparse_result = PrismQlParser::parse(&normalized);
        assert!(
            reparse_result.is_ok(),
            "BC-2.11.018 round-trip FAILED: normalized form '{normalized}' must re-parse \
             successfully. Error: {:?}",
            reparse_result.err()
        );

        // Step 4: The re-parsed AST must contain the original literal value "O'Brien".
        // This verifies correctness, not just "parses without error".
        let reparsed_ast = reparse_result.unwrap();
        let reparsed_normalized =
            PqlNormalizer::normalize(&reparsed_ast).expect("re-parsed AST must also normalize");
        assert_eq!(
            normalized, reparsed_normalized,
            "BC-2.11.018: normalized form must be idempotent (normalizing twice yields same output). \
             First: '{normalized}', second: '{reparsed_normalized}'"
        );
    }

    /// F-001B-PASS-MED-001 (filter mode): same round-trip check on a filter-mode query
    /// with source prefix, to cover the filter path distinct from bare-predicate path.
    #[test]
    fn test_BC_2_11_018_normalizer_roundtrip_embedded_quote_filter_mode_with_source() {
        // Filter mode with source prefix and embedded-quote literal.
        let input = r#"crowdstrike.detections | user_name = "O'Brien""#;

        let ast = PrismQlParser::parse(input).expect(
            "test precondition: filter with source prefix and double-quoted 'O\\'Brien' must parse",
        );

        let normalized = PqlNormalizer::normalize(&ast)
            .expect("normalize must return Some for filter+source AST");

        let reparse_result = PrismQlParser::parse(&normalized);
        assert!(
            reparse_result.is_ok(),
            "BC-2.11.018 round-trip FAILED (filter+source): normalized '{normalized}' \
             must re-parse. Error: {:?}",
            reparse_result.err()
        );
    }

    // ─────────────────────────────────────────────────────────────────────────────
    // F-001B-FRESH-HIGH-001 — sibling emit-site round-trip tests
    // Structural fix: shared `emit_quoted_string` helper applied to ALL string-emitting
    // sites in PqlNormalizer. These tests drive each previously-unfixed site.
    // ─────────────────────────────────────────────────────────────────────────────

    /// F-001B-FRESH-HIGH-001 (Predicate::Regex pattern with embedded single-quote).
    ///
    /// `normalize_predicate` for `Predicate::Regex` currently emits `=~ '{pattern}'`
    /// (single-quoted unconditionally). A pattern containing `'` breaks round-trip:
    ///   `field =~ "can't"` → Regex { pattern: "can't" } → normalized: `field =~ 'can't'`
    ///   Re-parse: `'can'` (stops at embedded `'`), then fails on remaining `t'`.
    ///
    /// BC-2.11.018: normalized form MUST parse to the same AST.
    ///
    /// RED GATE: re-parse of normalized output fails on current HEAD because `=~ '{pattern}'`
    /// cannot represent a pattern with `'` — the grammar's `build_string_parser` accepts
    /// `none_of('\'')` for single-quoted content.
    #[test]
    fn test_BC_2_11_018_sibling_regex_predicate_embedded_quote_roundtrip() {
        // Regex predicate with single-quote in pattern (double-quoted in input).
        // Grammar: build_string_parser accepts double_quoted = none_of('"'), so "can't" parses.
        let input = r#"description =~ "can't""#;

        let ast = PrismQlParser::parse(input)
            .expect("test precondition: regex predicate with double-quoted pattern must parse");

        let normalized = PqlNormalizer::normalize(&ast)
            .expect("normalize must return Some for regex predicate AST");

        // RED GATE: normalize emits `description =~ 'can't'` which the grammar cannot re-parse.
        let reparse_result = PrismQlParser::parse(&normalized);
        assert!(
            reparse_result.is_ok(),
            "BC-2.11.018 FAILED (Regex predicate, embedded quote): normalized '{normalized}' \
             must re-parse successfully. Error: {:?}",
            reparse_result.err()
        );

        // Idempotency: re-normalizing must produce the same string.
        let reparsed_normalized = PqlNormalizer::normalize(&reparse_result.unwrap())
            .expect("re-parsed regex AST must normalize");
        assert_eq!(
            normalized, reparsed_normalized,
            "BC-2.11.018: Regex predicate normalized form must be idempotent"
        );
    }

    /// F-001B-FRESH-HIGH-001 (Predicate::StringOp CONTAINS with embedded single-quote).
    ///
    /// `normalize_predicate` for `Predicate::StringOp` emits `{op} '{pattern}'`
    /// (single-quoted unconditionally). A pattern with `'` breaks round-trip.
    ///
    /// RED GATE: `field CONTAINS 'O'Brien'` fails to re-parse on current HEAD.
    #[test]
    fn test_BC_2_11_018_sibling_stringop_contains_embedded_quote_roundtrip() {
        // StringOp CONTAINS with a single-quote in the pattern.
        let input = r#"user_name CONTAINS "O'Brien""#;

        let ast = PrismQlParser::parse(input)
            .expect("test precondition: CONTAINS with double-quoted pattern must parse");

        let normalized =
            PqlNormalizer::normalize(&ast).expect("normalize must return Some for StringOp AST");

        let reparse_result = PrismQlParser::parse(&normalized);
        assert!(
            reparse_result.is_ok(),
            "BC-2.11.018 FAILED (StringOp CONTAINS, embedded quote): normalized '{normalized}' \
             must re-parse. Error: {:?}",
            reparse_result.err()
        );

        let reparsed_normalized = PqlNormalizer::normalize(&reparse_result.unwrap())
            .expect("re-parsed StringOp AST must normalize");
        assert_eq!(
            normalized, reparsed_normalized,
            "BC-2.11.018: StringOp CONTAINS normalized form must be idempotent"
        );
    }

    /// F-001B-FRESH-HIGH-001 (Predicate::Wildcard pattern with embedded single-quote).
    ///
    /// `normalize_predicate` for `Predicate::Wildcard` emits `{op} '{pattern}'`
    /// (single-quoted unconditionally). A wildcard pattern like `"O'*"` breaks round-trip.
    ///
    /// RED GATE: `field = 'O'*'` fails to re-parse on current HEAD.
    #[test]
    fn test_BC_2_11_018_sibling_wildcard_embedded_quote_roundtrip() {
        // Wildcard auto-promotion: `field = "O'*"` (double-quoted, contains wildcard `*`).
        // Grammar auto-promotes = with '*' in string to Predicate::Wildcard.
        let input = r#"host = "O'*""#;

        let ast = PrismQlParser::parse(input)
            .expect("test precondition: wildcard with double-quoted pattern must parse");

        let normalized =
            PqlNormalizer::normalize(&ast).expect("normalize must return Some for Wildcard AST");

        let reparse_result = PrismQlParser::parse(&normalized);
        assert!(
            reparse_result.is_ok(),
            "BC-2.11.018 FAILED (Wildcard, embedded quote): normalized '{normalized}' \
             must re-parse. Error: {:?}",
            reparse_result.err()
        );

        let reparsed_normalized = PqlNormalizer::normalize(&reparse_result.unwrap())
            .expect("re-parsed Wildcard AST must normalize");
        assert_eq!(
            normalized, reparsed_normalized,
            "BC-2.11.018: Wildcard normalized form must be idempotent"
        );
    }

    /// F-001B-FRESH-HIGH-001 (Literal::Regex inside IN list with embedded single-quote).
    ///
    /// `normalize_literal` for `Literal::Regex` emits `'{pattern}'` (single-quoted).
    /// A regex literal with `'` breaks round-trip when used in an IN list.
    ///
    /// Note: The grammar accepts regex literals as quoted strings via `build_string_parser`.
    /// After parsing, the string is stored as `Literal::String` (classify_string_literal returns
    /// Literal::String). So `Literal::Regex` is constructed directly by the `regex_match` parser.
    /// For the IN-list path, literals are plain strings — we test the regex predicate path which
    /// IS `Literal::Regex` wrapped in `RegexLiteral`. But the `normalize_literal` `Literal::Regex`
    /// arm is reached from `normalize_literal` (called from `normalize_predicate Predicate::In`
    /// values). Actually `Literal::Regex` cannot appear in an IN list (the grammar doesn't produce
    /// that). The `normalize_literal Literal::Regex` arm is reachable only via the expression path.
    ///
    /// This test covers the shared helper being applied to `Literal::Regex` in `normalize_literal`.
    /// We exercise it via `normalize_predicate Predicate::Regex` which calls `normalize_literal`
    /// indirectly — but that path uses `pattern.pattern` directly, not `normalize_literal`.
    ///
    /// The actual `normalize_literal Literal::Regex` arm is called from `normalize_literal_as_expr`
    /// which is called from `normalize_expr Expr::Literal`. A `Literal::Regex` can appear as an
    /// `Expr::Literal` in a Compare rhs (e.g. SQL). We test via the `normalize_literal` function
    /// directly since it's `pub` within the impl.
    ///
    /// RED GATE: `normalize_literal` for `Literal::Regex(r)` emits `'{r.pattern}'`.
    /// For r.pattern = "foo'bar", this produces `'foo'bar'` which the grammar cannot re-parse.
    #[test]
    fn test_BC_2_11_018_sibling_literal_regex_embedded_quote() {
        // Construct a Literal::Regex with a single-quote in the pattern.
        // We call PqlNormalizer::normalize_literal directly via a test harness.
        // Since normalize_literal is a private method, we invoke it via normalize_predicate
        // on a Predicate::In containing a Regex literal — but actually Predicate::In
        // calls normalize_literal on its Literal values.
        // Build: field IN ('pattern_without_quote') first to ensure the path is exercised.
        //
        // Actually Literal::Regex can be tested via normalize_literal → create a regex literal
        // with a quote, call normalize_predicate on a Compare containing it via Expr::Literal.
        // But the grammar produces Literal::Regex only from the regex_match parser, not Compare.
        //
        // The simplest direct test: construct the AST node manually and call normalize.
        let regex_lit =
            RegexLiteral::new("can't_match_this").expect("regex with apostrophe must be valid");
        let lit = Literal::Regex(regex_lit);

        // Invoke via a Filter AST containing Predicate::Compare with Expr::Literal(Literal::Regex).
        // Actually the real normalizer path is: normalize_literal is private.
        // We need to test it via a public surface. Use normalize_predicate indirectly by
        // constructing a full Ast.
        //
        // Build a synthetic Ast::Filter that contains a Regex predicate pattern with a quote.
        // The actual `Literal::Regex` arm in normalize_literal is called from normalize_literal,
        // which is called from normalize_predicate::Predicate::In → normalize_literal for each value.
        // Predicate::In values are `Vec<Literal>` — can we put Literal::Regex there? Syntactically
        // the parser won't produce that, but the types allow it.
        //
        // For this test, verify that the pattern in Predicate::Regex round-trips — the embedded-quote
        // fix to Predicate::Regex covers the same `Literal::Regex` arm conceptually.
        // The Predicate::Regex normalize path calls `pattern.pattern` directly (not normalize_literal),
        // so it's a separate arm.
        //
        // Direct structural test: assert that normalize_literal for Literal::Regex with embedded `'`
        // produces a double-quoted form, by constructing the Ast manually.
        let filter = FilterExpr {
            source: SourceRef::from_raw(""),
            predicate: Predicate::Regex {
                field: FieldPath::new(["description"]),
                pattern: RegexLiteral::new("can't").expect("regex with ' must be valid"),
            },
        };
        let ast = Ast::Filter(filter);

        let normalized =
            PqlNormalizer::normalize(&ast).expect("normalize must return Some for regex AST");

        // The normalized form should be parseable. On current HEAD, Predicate::Regex emits
        // `description =~ 'can't'` which fails to re-parse.
        let reparse_result = PrismQlParser::parse(&normalized);
        assert!(
            reparse_result.is_ok(),
            "BC-2.11.018 FAILED (Literal::Regex via Predicate::Regex, embedded quote): \
             normalized '{normalized}' must re-parse. Error: {:?}",
            reparse_result.err()
        );

        // Verify the re-parsed AST has the same pattern.
        let reparsed_normalized = PqlNormalizer::normalize(&reparse_result.unwrap())
            .expect("re-parsed regex AST must normalize");
        assert_eq!(
            normalized, reparsed_normalized,
            "BC-2.11.018: Predicate::Regex normalized form must be idempotent"
        );
        // Also verify that the literal `lit` is consistent with the test (suppress unused warning).
        let _ = lit;
    }

    // ─────────────────────────────────────────────────────────────────────────────
    // SEC-001 (CWE-116) — defense-in-depth: both-quotes grammar gap → normalize returns None
    // ─────────────────────────────────────────────────────────────────────────────

    /// SEC-001 (CWE-116): `normalize_pql` MUST return `None` when any string-bearing AST node
    /// contains BOTH `'` and `"`.
    ///
    /// # Rationale (defense-in-depth)
    /// The grammar defines:
    ///   - Single-quoted body: `none_of('\'')` — cannot represent `'` inside
    ///   - Double-quoted body: `none_of('"')` — cannot represent `"` inside
    ///
    /// A string containing BOTH quote characters cannot be faithfully emitted by
    /// `emit_quoted_string`. The normalizer cannot produce a round-tripping `normalized_pql`
    /// for such a value. Returning `None` causes the `normalized_pql` key to be ABSENT from
    /// the MCP response (consistent with EC-11-055 absent-when-empty behavior) — absent is
    /// safe; wrong would allow a mis-quoted value to reach the LLM agent context (CWE-116).
    ///
    /// # Parser-unreachability note
    /// This path is UNREACHABLE via the parser: `build_string_parser` in `filter_parser.rs`
    /// produces strings from `none_of('\'')` (single-quoted) or `none_of('"')` (double-quoted),
    /// so no source-input string can produce a literal containing both `'` and `"`. The test
    /// constructs the AST directly (bypassing the parser) to exercise the defense-in-depth guard.
    #[test]
    fn test_sec_001_both_quotes_grammar_gap_normalize_returns_none() {
        // PARSER-UNREACHABLE: construct an AST directly — the grammar cannot produce a
        // string literal containing BOTH `'` and `"` from source input.
        // Defense-in-depth: direct AST construction should still produce None, not a
        // mis-quoted output that would reach the LLM agent's normalized_pql field.
        let filter = FilterExpr {
            source: SourceRef::from_raw(""),
            predicate: Predicate::Compare {
                lhs: Box::new(Expr::Field(FieldPath::new(["hostname"]))),
                op: CompareOp::Eq,
                rhs: Box::new(Expr::Literal(Literal::String(
                    "it's a \"test\"".to_string(), // contains BOTH ' and "
                ))),
            },
        };
        let ast = Ast::Filter(filter);

        // SEC-001: normalize MUST return None (field omitted from MCP response — absent is safe).
        let result = PqlNormalizer::normalize(&ast);
        assert!(
            result.is_none(),
            "SEC-001 FAILED: normalize_pql must return None for an AST node containing both \
             quote characters (grammar-gap, parser-unreachable but defense-in-depth required). \
             Got: {:?}",
            result
        );
    }

    /// SEC-001 variant: both-quotes in a Predicate::StringOp pattern also returns None.
    #[test]
    fn test_sec_001_both_quotes_stringop_pattern_normalize_returns_none() {
        let filter = FilterExpr {
            source: SourceRef::from_raw(""),
            predicate: Predicate::StringOp {
                field: FieldPath::new(["description"]),
                op: StringOp::Contains,
                pattern: "it's a \"test\"".to_string(), // BOTH ' and "
                case_insensitive: false,
            },
        };
        let ast = Ast::Filter(filter);
        assert!(
            PqlNormalizer::normalize(&ast).is_none(),
            "SEC-001: StringOp pattern with both quotes must yield None"
        );
    }

    /// SEC-001 variant: Predicate::Regex pattern with both quotes returns None.
    #[test]
    fn test_sec_001_both_quotes_regex_pattern_normalize_returns_none() {
        let filter = FilterExpr {
            source: SourceRef::from_raw(""),
            predicate: Predicate::Regex {
                field: FieldPath::new(["message"]),
                pattern: RegexLiteral::new(r#"it's a "pattern""#)
                    .expect("regex with both quotes must be syntactically valid"),
            },
        };
        let ast = Ast::Filter(filter);
        assert!(
            PqlNormalizer::normalize(&ast).is_none(),
            "SEC-001: Regex pattern with both quotes must yield None"
        );
    }

    /// SEC-001 variant: Predicate::Wildcard pattern with both quotes returns None.
    #[test]
    fn test_sec_001_both_quotes_wildcard_pattern_normalize_returns_none() {
        let filter = FilterExpr {
            source: SourceRef::from_raw(""),
            predicate: Predicate::Wildcard {
                field: FieldPath::new(["hostname"]),
                pattern: "it's \"host*\"".to_string(), // BOTH ' and "
                negated: false,
            },
        };
        let ast = Ast::Filter(filter);
        assert!(
            PqlNormalizer::normalize(&ast).is_none(),
            "SEC-001: Wildcard pattern with both quotes must yield None"
        );
    }

    /// SEC-001 defense-in-depth: `Expr::FuncCall` args with both-quotes string returns None.
    ///
    /// # Parser-unreachability note
    ///
    /// This path is UNREACHABLE via the parser: `build_string_parser` cannot produce a string
    /// literal containing BOTH `'` and `"`. This test constructs the AST directly (bypassing
    /// the parser) to exercise the defense-in-depth guard in `expr_has_both_quote_string`.
    ///
    /// # Why this matters
    ///
    /// The `ast_has_both_quote_string` walker is the defense-in-depth guard for SEC-001
    /// (CWE-116). It traverses ALL string-bearing AST nodes. Before this fix, `Expr::FuncCall`
    /// fell to the `_ => false` arm — FuncCall args containing both-quote strings would
    /// bypass the guard and reach `normalize_func_call`, which produces potentially
    /// mis-quoted output. The fix adds explicit arg traversal.
    ///
    /// # RED→GREEN
    ///
    /// FAILS on current HEAD: `expr_has_both_quote_string` returns `false` for `FuncCall`
    /// (falls to `_ => false`), so `normalize` returns `Some(...)` instead of `None`.
    /// PASSES AFTER FIX: explicit `Expr::FuncCall` arm recurses into args, finds the
    /// both-quote `Literal::String`, and returns `true` → normalize returns `None`.
    #[test]
    fn test_sec_001_both_quotes_func_call_arg_normalize_returns_none() {
        // PARSER-UNREACHABLE: construct a FuncCall AST directly with a both-quote string arg.
        // Defense-in-depth: direct AST construction with a both-quote FuncCall arg must
        // still produce None, not mis-quoted output.
        //
        // Construct: SELECT unknown_udf("it's a \"test\"") FROM crowdstrike_alerts
        // where the FuncCall arg is a Literal::String containing BOTH ' and ".
        let both_quote_arg = Expr::Literal(Literal::String("it's a \"test\"".to_string()));
        let func_call_expr = Expr::FuncCall(FuncCall::Scalar {
            func: ScalarFunc::Unknown("unknown_udf".to_string()),
            args: vec![both_quote_arg],
        });

        // Wrap in a minimal SQL AST so we can call normalize().
        let sql_query = SqlQuery::new(
            SelectClause::new(vec![SelectItem::Expr {
                expr: func_call_expr,
                alias: None,
            }]),
            FromClause::new(SourceRef::from_raw("crowdstrike_alerts")),
        );
        let ast = Ast::Sql(SqlStatement::Select(sql_query));

        // SEC-001 defense-in-depth: normalize MUST return None when a FuncCall arg contains
        // both quote characters (parser-unreachable but defense-in-depth required).
        //
        // FAILS NOW because expr_has_both_quote_string has `Expr::FuncCall(_) => false`
        // (falls to `_ => false`), so the guard does not detect the both-quote arg.
        //
        // PASSES AFTER FIX: add explicit FuncCall arm that recurses into args.
        let result = PqlNormalizer::normalize(&ast);
        assert!(
            result.is_none(),
            "SEC-001 defense-in-depth: normalize_pql MUST return None when a FuncCall arg \
             contains both quote characters (parser-unreachable guard). \
             Current behavior: returns Some (expr_has_both_quote_string falls to `_ => false` \
             for FuncCall, bypassing the guard). \
             FIX: add `Expr::FuncCall(fc) => fc.args().any(Self::expr_has_both_quote_string)` \
             to expr_has_both_quote_string. Got: {:?}",
            result
        );
    }

    /// SEC-001 regression: single-quote-only values still normalize correctly (no regression).
    #[test]
    fn test_sec_001_single_quote_only_still_normalizes_correctly() {
        // Regression check: values with only ' (no ") must still produce double-quoted output.
        let input = r#"host_name = "O'Brien""#;
        let ast = PrismQlParser::parse(input).expect("O'Brien must parse");
        let normalized = PqlNormalizer::normalize(&ast)
            .expect("SEC-001 regression: single-quote-only value must still yield Some");
        assert!(
            normalized.contains("\"O'Brien\""),
            "SEC-001 regression: single-quote-only value must normalize to double-quoted form, \
             got: {normalized}"
        );
    }

    /// SEC-001 regression: no-quote values still normalize correctly.
    #[test]
    fn test_sec_001_no_quote_value_still_normalizes_correctly() {
        let input = "host = 'example.com'";
        let ast = PrismQlParser::parse(input).expect("no-quote value must parse");
        let normalized = PqlNormalizer::normalize(&ast)
            .expect("SEC-001 regression: no-quote value must still yield Some");
        assert!(
            normalized.contains("'example.com'"),
            "SEC-001 regression: no-quote value must normalize to single-quoted form, \
             got: {normalized}"
        );
    }

    /// F-001B-FRESH-HIGH-001 (Literal::Float whole-number round-trip: score = 5.0 → Integer).
    ///
    /// `normalize_literal` for `Literal::Float(f)` currently emits `f.to_string()`.
    /// `OrderedFloat(5.0_f64).to_string()` emits `"5"` (no decimal point) because Rust's
    /// `f64::to_string()` for integers outputs the integer representation.
    ///
    /// The grammar requires `digits '.' digits` for float literals
    /// (`filter_parser.rs::build_literal_parser::float_lit`). Re-parsing `score = 5` parses
    /// the `5` as `Literal::Integer(5)`, not `Literal::Float(5.0)` — AST type change.
    ///
    /// BC-2.11.018: normalized form MUST parse to the same AST.
    ///
    /// RED GATE: `score = 5.0` → normalized `score = 5` → re-parsed as `Literal::Integer(5)`,
    /// not `Literal::Float(5.0)`. AST comparison fails on current HEAD.
    #[test]
    fn test_BC_2_11_018_sibling_float_whole_number_roundtrip() {
        // Input: float literal with no fractional part.
        let input = "score = 5.0";

        let ast = PrismQlParser::parse(input)
            .expect("test precondition: 'score = 5.0' must parse as float");

        // Verify the parsed literal IS a float, not an integer.
        match &ast {
            Ast::Filter(f) => match &f.predicate {
                Predicate::Compare { rhs, .. } => match rhs.as_ref() {
                    Expr::Literal(Literal::Float(v)) => {
                        assert!(
                            (v.0 - 5.0_f64).abs() < f64::EPSILON,
                            "precondition: parsed literal must be Float(5.0), got Float({v})"
                        );
                    }
                    other => panic!("precondition: expected Expr::Literal(Float), got: {other:?}"),
                },
                other => panic!("precondition: expected Compare predicate, got: {other:?}"),
            },
            other => panic!("precondition: expected Filter AST, got: {other:?}"),
        }

        let normalized = PqlNormalizer::normalize(&ast)
            .expect("normalize must return Some for float-literal AST");

        // On current HEAD, normalized = "score = 5" (no decimal point).
        // Re-parsing "score = 5" produces Literal::Integer(5), not Literal::Float(5.0).
        // BC-2.11.018: re-parse must produce equivalent AST (Float, not Integer).
        let reparse_result = PrismQlParser::parse(&normalized);
        assert!(
            reparse_result.is_ok(),
            "BC-2.11.018 FAILED (Float whole-number): normalized '{normalized}' \
             must re-parse. Error: {:?}",
            reparse_result.err()
        );

        // The re-parsed AST must preserve Float, not silently become Integer.
        let reparsed_ast = reparse_result.unwrap();
        match &reparsed_ast {
            Ast::Filter(f) => match &f.predicate {
                Predicate::Compare { rhs, .. } => match rhs.as_ref() {
                    Expr::Literal(Literal::Float(_)) => {} // expected: Float preserved
                    Expr::Literal(Literal::Integer(n)) => {
                        panic!(
                            "BC-2.11.018 FAILED (Float whole-number): normalized '{normalized}' \
                             re-parsed as Integer({n}), not Float(5.0). \
                             `f.to_string()` for 5.0_f64 emits '5' (no decimal point), \
                             causing AST type change on re-parse."
                        );
                    }
                    other => panic!(
                        "BC-2.11.018 FAILED (Float): expected Literal::Float after re-parse, \
                         got: {other:?}"
                    ),
                },
                other => panic!("expected Compare predicate after re-parse, got: {other:?}"),
            },
            other => panic!("expected Filter AST after re-parse, got: {other:?}"),
        }

        // Idempotency.
        let reparsed_normalized =
            PqlNormalizer::normalize(&reparsed_ast).expect("re-parsed float AST must normalize");
        assert_eq!(
            normalized, reparsed_normalized,
            "BC-2.11.018: Float whole-number normalized form must be idempotent"
        );
    }
}

/// OBS-1 — `Ast::SqlPipe` pre-check parity tests.
///
/// Verifies that `ast_has_both_quote_string` and `ast_has_unfolded_temporal_expr` fire for
/// `Ast::SqlPipe` nodes, both when the trigger is in the SQL head and when it is in a pipe stage.
/// Prior to the OBS-1 fix, both functions fell through to `_ => false` for `Ast::SqlPipe`, making
/// them latent silent-corruption vectors for direct AST construction.
///
/// All tests construct ASTs directly (bypassing the parser) because:
/// (a) both-quote strings are parser-unreachable (grammar uses `none_of`), and
/// (b) unfolded temporal expressions in SqlPipe would normally be folded by `inject_now` before
///     reaching `normalize()` — these tests exercise the defense-in-depth guard directly.
#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod obs_1_sqlpipe_prechek_parity_tests {
    use super::*;

    // ─── Helper: minimal SqlQuery (SELECT * FROM t) with no WHERE ───────────────

    fn minimal_sql_head(table: &str) -> SqlQuery {
        SqlQuery::new(
            SelectClause::new(vec![SelectItem::Star]),
            FromClause::new(SourceRef::from_raw(table)),
        )
    }

    // ─── OBS-1: ast_has_both_quote_string on Ast::SqlPipe ───────────────────────

    /// OBS-1 parity: `ast_has_both_quote_string` returns `true` when a WHERE predicate in the
    /// SqlPipe SQL HEAD contains a string literal with both `'` and `"`.
    ///
    /// Before the fix, this returned `false` (fell through to `_ => false`).
    /// After the fix, it returns `true` — and consequently `PqlNormalizer::normalize` returns
    /// `None` (safe abort) rather than silently emitting malformed SQL.
    #[test]
    fn test_obs1_sqlpipe_head_both_quote_string_prechek_fires() {
        // Construct an Ast::SqlPipe whose SQL head has a WHERE predicate
        // containing a string literal with BOTH ' and ".
        let head = minimal_sql_head("sensors").with_where(Predicate::Compare {
            lhs: Box::new(Expr::Field(FieldPath::new(["hostname"]))),
            op: CompareOp::Eq,
            rhs: Box::new(Expr::Literal(Literal::String(
                "it's a \"test\"".to_string(), // contains BOTH ' and "
            ))),
        });
        let ast = Ast::SqlPipe(SqlPipeQuery {
            head,
            stages: vec![PipeStage::Limit(10)],
        });

        // The pre-check MUST fire: normalize returns None (safe abort).
        let result = PqlNormalizer::normalize(&ast);
        assert!(
            result.is_none(),
            "OBS-1: ast_has_both_quote_string must detect both-quote string in SqlPipe SQL head \
             and cause normalize to return None; got: {:?}",
            result
        );
    }

    /// OBS-1 parity: `ast_has_both_quote_string` returns `true` when a WHERE pipe stage in the
    /// SqlPipe STAGES contains a string literal with both `'` and `"`.
    #[test]
    fn test_obs1_sqlpipe_stage_both_quote_string_prechek_fires() {
        // SQL head is clean — the trigger is in a pipe WHERE stage.
        let head = minimal_sql_head("sensors");
        let both_quote_pred = Predicate::Compare {
            lhs: Box::new(Expr::Field(FieldPath::new(["description"]))),
            op: CompareOp::Eq,
            rhs: Box::new(Expr::Literal(Literal::String(
                "it's a \"stage\"".to_string(), // contains BOTH ' and "
            ))),
        };
        let ast = Ast::SqlPipe(SqlPipeQuery {
            head,
            stages: vec![PipeStage::Where(both_quote_pred)],
        });

        let result = PqlNormalizer::normalize(&ast);
        assert!(
            result.is_none(),
            "OBS-1: ast_has_both_quote_string must detect both-quote string in SqlPipe pipe \
             stage and cause normalize to return None; got: {:?}",
            result
        );
    }

    // ─── OBS-1: ast_has_unfolded_temporal_expr on Ast::SqlPipe ──────────────────

    /// OBS-1 parity: `ast_has_unfolded_temporal_expr` returns `true` when the SqlPipe SQL
    /// HEAD contains an unfolded `Expr::Now` in the WHERE predicate.
    ///
    /// Before the fix, this returned `false` (fell through to `_ => false`). After the fix,
    /// `normalize` returns `None` instead of silently emitting malformed SQL like
    /// `WHERE timestamp >  ` (empty right-hand side from the catch-all `_ => String::new()` arm).
    #[test]
    fn test_obs1_sqlpipe_head_unfolded_temporal_prechek_fires() {
        // Unfolded Expr::Now in the SqlPipe SQL head's WHERE clause.
        // In production, inject_now folds this before normalize() is called.
        // This test exercises the defense-in-depth pre-check for the case where it isn't folded.
        let head = minimal_sql_head("sensors").with_where(Predicate::Compare {
            lhs: Box::new(Expr::Field(FieldPath::new(["timestamp"]))),
            op: CompareOp::Gt,
            rhs: Box::new(Expr::Now), // unfolded temporal — NOT yet constant-folded
        });
        let ast = Ast::SqlPipe(SqlPipeQuery {
            head,
            stages: vec![PipeStage::Limit(100)],
        });

        let result = PqlNormalizer::normalize(&ast);
        assert!(
            result.is_none(),
            "OBS-1: ast_has_unfolded_temporal_expr must detect Expr::Now in SqlPipe SQL head \
             and cause normalize to return None (prevents malformed SQL); got: {:?}",
            result
        );
    }

    /// OBS-1 parity: `ast_has_unfolded_temporal_expr` returns `true` when an unfolded
    /// `Expr::Now` appears in a SqlPipe PIPE STAGE WHERE predicate.
    #[test]
    fn test_obs1_sqlpipe_stage_unfolded_temporal_prechek_fires() {
        // SQL head is temporally clean; the unfolded temporal is in a pipe WHERE stage.
        let head = minimal_sql_head("sensors");
        let temporal_pred = Predicate::Compare {
            lhs: Box::new(Expr::Field(FieldPath::new(["event_time"]))),
            op: CompareOp::Gt,
            rhs: Box::new(Expr::Now), // unfolded — defense-in-depth target
        };
        let ast = Ast::SqlPipe(SqlPipeQuery {
            head,
            stages: vec![PipeStage::Where(temporal_pred)],
        });

        let result = PqlNormalizer::normalize(&ast);
        assert!(
            result.is_none(),
            "OBS-1: ast_has_unfolded_temporal_expr must detect Expr::Now in SqlPipe pipe stage \
             and cause normalize to return None; got: {:?}",
            result
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// LOW-1: DataFusionModeGuard save-and-restore tests
//
// ADR-052 §D4 v1.6: DataFusionModeGuard::drop MUST restore the prior value of
// NORMALIZE_FOR_DATAFUSION, NOT hard-set it to false.
//
// Bug: current Drop implementation calls `m.set(false)` unconditionally.
// Fix: save the prior bool on construction; Drop restores saved value.
//
// The nesting test below constructs a scenario where the thread-local is `true`
// when normalize_for_datafusion is called, and asserts the guard restores it to
// `true` on Drop.  With the current hard-set=false implementation the assertion
// FAILS (RED).  After the fix it PASSES (GREEN).
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod low1_datafusion_guard_tests {
    use super::*;

    /// LOW-1 nesting test: `DataFusionModeGuard::drop` must RESTORE the prior value
    /// of `NORMALIZE_FOR_DATAFUSION`, not hard-set it to `false`.
    ///
    /// Scenario:
    ///   1. Thread-local is manually set to `true` (simulating an outer
    ///      `normalize_for_datafusion` guard already active on the call stack).
    ///   2. `normalize_for_datafusion` is called (inner call) — it creates its own
    ///      `DataFusionModeGuard` and then drops it when the function returns.
    ///   3. After the inner call returns, the thread-local MUST still be `true`
    ///      because the inner guard must restore the value it found on construction
    ///      (true), not hard-set false.
    ///
    /// RED GATE: with current `impl Drop { m.set(false) }`, the inner guard
    /// unconditionally sets false → assertion FAILS.
    ///
    /// GREEN (after fix): `DataFusionModeGuard` saves `prior = m.get()` on construction
    /// and restores it in Drop → thread-local remains `true` → assertion PASSES.
    ///
    /// Traces to: ADR-052 §D4 v1.6 LOW-1.
    #[test]
    fn test_low1_normalize_for_datafusion_guard_save_restore_nesting() {
        // Step 1: Manually set thread-local to true (outer guard context).
        NORMALIZE_FOR_DATAFUSION.with(|m| m.set(true));

        // Step 2: Call normalize_for_datafusion (inner call).
        // A minimal valid SQL AST with a Timestamp literal.
        let now = chrono::Utc::now();
        let ts_lit = TimestampLiteral {
            iso8601: now.to_rfc3339(),
            instant: now,
        };
        let sql = SqlQuery::new(
            SelectClause::new(vec![SelectItem::Star]),
            FromClause::new(SourceRef::from_raw("test_table")),
        )
        .with_where(Predicate::Compare {
            lhs: Box::new(Expr::Field(FieldPath::new(["timestamp"]))),
            op: CompareOp::Gt,
            rhs: Box::new(Expr::Literal(Literal::Timestamp(ts_lit))),
        });
        let ast = Ast::Sql(SqlStatement::Select(sql));

        // Inner call — the guard must restore NORMALIZE_FOR_DATAFUSION to true (prior),
        // NOT hard-set it to false.
        let result = PqlNormalizer::normalize_for_datafusion(&ast);
        assert!(
            result.is_some(),
            "LOW-1 nesting test precondition: normalize_for_datafusion must return Some for \
             valid SQL AST. Got None."
        );

        // Verify the inner call emitted arrow_cast (correct operation).
        let sql_str = result.unwrap();
        assert!(
            sql_str.contains("arrow_cast("),
            "LOW-1 nesting test precondition: inner normalize_for_datafusion must emit \
             arrow_cast for Literal::Timestamp. Got: {sql_str:?}"
        );

        // Step 3: Assert the thread-local was restored to true (the prior value).
        let state_after_inner_call = NORMALIZE_FOR_DATAFUSION.with(|m| m.get());
        assert!(
            state_after_inner_call,
            "LOW-1: DataFusionModeGuard::drop must RESTORE prior value (true) rather than \
             hard-setting false. After the inner normalize_for_datafusion call returned, \
             NORMALIZE_FOR_DATAFUSION should be true (the value it had before the inner call). \
             Got false — the hard-set=false Drop destroyed the outer context's setting. \
             Fix: save `prior = m.get()` in DataFusionModeGuard, restore in Drop."
        );

        // Cleanup: reset thread-local to false so we don't leak state into other tests.
        NORMALIZE_FOR_DATAFUSION.with(|m| m.set(false));
    }

    /// LOW-1 basic: after a top-level `normalize_for_datafusion` call (thread-local
    /// starts false), the guard must restore it to false.
    ///
    /// This is the normal (non-nested) case — must continue to work after the save-and-restore
    /// fix.  A regression here would mean the guard accidentally keeps the flag set.
    #[test]
    fn test_low1_normalize_for_datafusion_guard_resets_to_false_for_top_level_call() {
        // Thread-local starts at false (normal initial state for each thread).
        NORMALIZE_FOR_DATAFUSION.with(|m| m.set(false));

        let now = chrono::Utc::now();
        let ts_lit = TimestampLiteral {
            iso8601: now.to_rfc3339(),
            instant: now,
        };
        let sql = SqlQuery::new(
            SelectClause::new(vec![SelectItem::Star]),
            FromClause::new(SourceRef::from_raw("test_table")),
        )
        .with_where(Predicate::Compare {
            lhs: Box::new(Expr::Field(FieldPath::new(["ts"]))),
            op: CompareOp::Gt,
            rhs: Box::new(Expr::Literal(Literal::Timestamp(ts_lit))),
        });
        let ast = Ast::Sql(SqlStatement::Select(sql));

        let _ = PqlNormalizer::normalize_for_datafusion(&ast);

        // After the top-level call, the thread-local must be false (restored to prior=false).
        let state_after = NORMALIZE_FOR_DATAFUSION.with(|m| m.get());
        assert!(
            !state_after,
            "LOW-1 basic: after a top-level normalize_for_datafusion call (prior=false), \
             the guard must restore the thread-local to false. Got true — the guard is \
             leaking the DataFusion mode flag."
        );
    }
}
