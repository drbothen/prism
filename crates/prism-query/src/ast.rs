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

use std::net::IpAddr;

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

/// SQL statement wrapper — forward-compat shim for S-3.06.
///
/// S-3.06 will add `Dml(DmlNode)` and `Ddl(DdlNode)` variants without
/// breaking the `Ast::Sql(SqlStatement)` shape.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SqlStatement {
    /// A `SELECT` query.
    Select(SqlQuery),
    // S-3.06: Dml(DmlNode), Ddl(DdlNode)
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
/// `write` is a forward-compat placeholder for S-3.06 (output write node).
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PipeQuery {
    pub source: SourceRef,
    pub stages: Vec<PipeStage>,
    /// S-3.06 placeholder: write target. Always `None` in S-3.01.
    // TODO(S-3.06): replace `Option<()>` with `Option<WriteNode>`.
    pub write: Option<()>,
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
}

/// Duration literal with explicit unit (prismql-grammar.md §3.3).
///
/// # Validation
/// Constructed only if `value * unit_secs` does not overflow `u64`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DurationLiteral {
    /// Numeric magnitude.
    pub value: u64,
    /// Duration unit.
    pub unit: DurationUnit,
}

impl DurationLiteral {
    /// Construct a `DurationLiteral`, returning `Err` on overflow.
    pub fn new(value: u64, unit: DurationUnit) -> Result<Self, &'static str> {
        // Validate that seconds-conversion doesn't overflow.
        let _ = value
            .checked_mul(unit.secs())
            .ok_or("E-QUERY-001: duration value overflows u64 seconds")?;
        Ok(Self { value, unit })
    }

    /// Total duration in seconds.
    pub fn to_secs(&self) -> u64 {
        // Safety: validated in constructor.
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
    pub fn new(s: &str) -> Result<Self, String> {
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
    /// Enforces the 1024-byte cap (BC-2.11.006) and validates the pattern
    /// with the `regex` crate's finite-automaton engine.
    pub fn new(pattern: &str) -> Result<Self, String> {
        const MAX_LEN: usize = 1_024;
        if pattern.len() > MAX_LEN {
            return Err(format!(
                "E-QUERY-003: regex pattern length {} bytes exceeds maximum allowed {} bytes",
                pattern.len(),
                MAX_LEN
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

/// ISO-8601 timestamp literal.
///
/// The raw string is preserved for display; `epoch_ms` stores the parsed
/// millisecond epoch value. If parsing fails, `epoch_ms` is `None` and
/// downstream evaluation uses the raw string.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TimestampLiteral {
    /// Raw ISO-8601 string (e.g. `"2026-04-13T00:00:00Z"`).
    pub iso8601: String,
    // TODO(S-3.xx): parse via `chrono` or `time` crate and populate epoch_ms.
    // For now: None. The query planner falls back to string comparison.
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
