//! Pipe-to-SQL emitter — lowers a `PipeQuery` AST to executable DataFusion SQL.
//!
//! # Design rationale
//!
//! `execute_against_session` routes `Ast::Pipe` through this emitter to produce a
//! SQL string, then passes it to `session_ctx.sql()` — the same proven path used by
//! `Ast::Sql(Select)`. This ensures all DataFusion invariants are respected:
//! - GreedyMemoryPool memory budget (BC-2.11.006, E-WATCHDOG-001) via `map_datafusion_memory_error`
//! - Async UDF execution via `execute_stream()` calling `invoke_async_with_args`
//! - Streaming collection via `collect_record_batch_stream`
//! - 10K materialized record cap (enforced in fan-out loop before reaching this module)
//! - 30s timeout (enforced by `tokio::time::timeout` in `execute()`)
//!
//! # Per-stage SQL lowering
//!
//! See `.factory/specs/architecture/scoping/pipe-execution-engine-design.md` §3 for the
//! authoritative mapping. Summary:
//!
//! | PipeStage variant        | SQL equivalent                        |
//! |--------------------------|---------------------------------------|
//! | `Enrich { infusion, field }` | CTE: `SELECT *, udf(field) AS udf` |
//! | `Where(Predicate)`       | `WHERE predicate_sql`                 |
//! | `Stats(StatsStage)`      | `SELECT aggs [GROUP BY by_fields]`    |
//! | `Sort(Vec<SortExpr>)`    | `ORDER BY field [ASC\|DESC]`          |
//! | `Limit(n)` / `head N`   | `LIMIT n`                             |
//! | `Tail(n)`                | `LIMIT n` (semantic gap — see §3.2)   |
//! | `Fields(FieldsStage)`    | `SELECT col_list` (include/exclude)   |
//! | `Dedup(Vec<FieldPath>)`  | `SELECT DISTINCT *`                   |
//! | `Join(JoinStage)`        | `Err` (deferred to ENRICH-4-C Join)   |
//!
//! # CTE pattern for chained enrichments
//!
//! Each `Enrich` stage wraps the current query in a CTE alias so subsequent stages
//! (including other `Enrich` stages and `Where` predicates) can reference the
//! enriched column by name:
//!
//! ```sql
//! WITH _pipe_0 AS (SELECT * FROM crowdstrike_detections),
//!      _pipe_1 AS (SELECT *, ioc_match(iocs_value) AS ioc_match FROM _pipe_0)
//! SELECT * FROM _pipe_1
//! WHERE ioc_match IS NOT NULL
//! ```
//!
//! # Predicate-to-SQL translation
//!
//! PQL predicate syntax diverges from DataFusion SQL for `CONTAINS`, `HAS`,
//! `MISSING`, `CIDR`, and regex operators. The `predicate_to_datafusion_sql`
//! helper handles these conversions. See §8 of the design doc for the full table.
//!
//! Traces to: BC-2.11.004 (Pipe Mode), BC-2.19.001 (Infusion UDFs), BC-2.11.006 (Security Limits)
//! Design doc: `.factory/specs/architecture/scoping/pipe-execution-engine-design.md`

// `#[non_exhaustive]` arms in match blocks are intentionally unreachable within this crate
// (same-crate matches are exhaustive) but required for external-crate forward compat.
// The allow suppresses the "unreachable pattern" warning without removing the guard.
#![allow(unreachable_patterns)]

use std::collections::HashMap;
use std::sync::Arc;

use arrow::datatypes::Schema;
use arrow::record_batch::RecordBatch;
use prism_core::PrismError;

use crate::ast::{
    AggFunc, CompareOp, Expr, FieldPath, FieldsStage, Literal, LogicalOp, PipeQuery, PipeStage,
    Predicate, SortDirection, StatsStage, StringOp,
};

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Lower a `PipeQuery` AST to an executable DataFusion SQL string.
///
/// The SQL is intended for immediate execution via `SessionContext::sql()` against
/// the MemTables already registered by `run_materialization_pipeline`. The caller
/// is responsible for ensuring the source MemTable (derived from `pipe.source.raw`)
/// has been registered before calling `execute_against_session`.
///
/// # Arguments
/// - `pipe` — the parsed `PipeQuery` AST.
/// - `table_batches` — the fan-out result map keyed by registered MemTable name;
///   used ONLY for schema inspection in the `Fields(exclude)` case.
///
/// # Errors
/// Returns `PrismError::QueryExecutionFailed` for unsupported stages (e.g., `Join`).
pub(crate) fn pipe_to_executable_sql(
    pipe: &PipeQuery,
    table_batches: &HashMap<String, Vec<RecordBatch>>,
) -> Result<String, PrismError> {
    let mut builder = PipeQueryBuilder::new(source_table_name(&pipe.source.raw), table_batches);
    builder.build(pipe)
}

/// Lower a `SqlPipeQuery` to an executable DataFusion SQL string.
///
/// The head SQL SELECT is wrapped in a CTE (`_sqlpipe_head`), and pipe stages
/// are applied on top of that CTE using the same `PipeQueryBuilder` pipeline
/// as `pipe_to_executable_sql`.  The caller must have already run the
/// FORBID-BOTH check (`plan_sqlpipe_query`) before calling this function.
///
/// # Arguments
/// - `query_str` — the original PrismQL query string (used to extract the head
///   SQL substring; the split point is re-derived using `find_sqlpipe_split`).
/// - `spq` — the parsed `SqlPipeQuery` AST (stages list).
/// - `table_batches` — fan-out result map; used ONLY for schema inspection in
///   the `Fields(exclude)` case.
///
/// # Errors
/// Returns `PrismError::QueryExecutionFailed` if the head SQL cannot be
/// extracted from `query_str` (should not occur in practice — the parser
/// already validated the split during `parse_sqlpipe_internal`).
pub(crate) fn sqlpipe_to_executable_sql(
    head_sql: &str,
    spq: &crate::ast::SqlPipeQuery,
    table_batches: &HashMap<String, Vec<RecordBatch>>,
) -> Result<String, PrismError> {
    // BC-2.11.021 / ADR-044 D4 / D-1333 Option A:
    // `head_sql` is the plan-pinned head SQL (already computed by the caller
    // from the inject_now-ed AST via PqlNormalizer::normalize). It must NOT
    // be re-derived from the raw query_str (which would contain runtime NOW()
    // or INTERVAL). The caller in execute_against_session passes the normalized
    // plan-pinned form directly.

    // Wrap head SQL in a CTE so pipe stages can reference it by alias.
    // CTE alias `_sqlpipe_head` is an internal name that cannot collide with
    // user-defined table names (which must match sensor table naming conventions).
    let cte_alias = "_sqlpipe_head";
    let mut builder =
        PipeQueryBuilder::new_with_cte(cte_alias.to_string(), head_sql.trim_end(), table_batches);

    // Apply pipe stages from the SqlPipeQuery.
    for stage in &spq.stages {
        builder.apply_stage(stage)?;
    }
    Ok(builder.assemble())
}

// ---------------------------------------------------------------------------
// Source table name resolution
// ---------------------------------------------------------------------------

/// Resolve the source table name from a `SourceRef.raw` string.
///
/// The MemTable is registered under the underscore form
/// (e.g., `crowdstrike_detections`). Dot notation (`crowdstrike.detections`)
/// is converted to underscore form by replacing the first `.` with `_`.
fn source_table_name(raw: &str) -> String {
    // Replace first dot with underscore: "crowdstrike.detections" → "crowdstrike_detections"
    match raw.find('.') {
        Some(pos) => {
            let mut s = raw.to_string();
            s.replace_range(pos..=pos, "_");
            s
        }
        None => raw.to_string(),
    }
}

// ---------------------------------------------------------------------------
// PipeQueryBuilder — accumulates SQL clauses from stage list
// ---------------------------------------------------------------------------

/// Multi-stage SQL query builder.
///
/// Walks `pipe.stages` in order, accumulating SQL clauses, then assembles the
/// final SQL string. The CTE list grows for each `Enrich` stage.
struct PipeQueryBuilder {
    /// SELECT items. Default: `["*"]`. Replaced by Stats or Fields stages.
    select_items: Vec<String>,
    /// WHERE predicates — ANDed together.
    where_clauses: Vec<String>,
    /// GROUP BY fields — set by Stats stage.
    group_by: Vec<String>,
    /// ORDER BY clauses — set by Sort stage.
    order_by: Vec<String>,
    /// LIMIT value — minimum across all Limit/Head/Tail stages.
    limit: Option<u64>,
    /// `SELECT DISTINCT` — set by Dedup stage.
    distinct: bool,
    /// CTE list: (alias, inner_sql). Built up by Enrich stages.
    /// Each entry wraps the previous stage's output.
    ctes: Vec<(String, String)>,
    /// The current "innermost" FROM target (either the source table or the latest CTE alias).
    current_from: String,
    /// Schema of the source MemTable — used by Fields-exclude for schema-aware projection.
    schema: Option<Arc<Schema>>,
}

impl PipeQueryBuilder {
    fn new(source_table: String, table_batches: &HashMap<String, Vec<RecordBatch>>) -> Self {
        // Extract schema from the source table's batches for Fields-exclude projection.
        let schema = table_batches
            .get(&source_table)
            .and_then(|bs| bs.first())
            .map(|b| b.schema());

        let current_from = source_table;
        Self {
            select_items: vec!["*".to_string()],
            where_clauses: Vec::new(),
            group_by: Vec::new(),
            order_by: Vec::new(),
            limit: None,
            distinct: false,
            ctes: Vec::new(),
            current_from,
            schema,
        }
    }

    /// Build a `PipeQueryBuilder` that wraps `head_sql` as a CTE under `cte_alias`.
    ///
    /// Used by `sqlpipe_to_executable_sql` for SQL→Pipe composition mode: the SQL
    /// head is wrapped as `cte_alias AS (head_sql)` so subsequent pipe stages can
    /// reference it as a named relation.
    fn new_with_cte(
        cte_alias: String,
        head_sql: &str,
        table_batches: &HashMap<String, Vec<RecordBatch>>,
    ) -> Self {
        // Schema cannot be statically inferred from the head SQL (it would require
        // DataFusion planning to resolve the output schema). Fields-exclude after a
        // SqlPipe head falls back to SELECT * (same as Fields-exclude after an Enrich CTE).
        let schema = table_batches
            .get(&cte_alias)
            .and_then(|bs| bs.first())
            .map(|b| b.schema());

        Self {
            select_items: vec!["*".to_string()],
            where_clauses: Vec::new(),
            group_by: Vec::new(),
            order_by: Vec::new(),
            limit: None,
            distinct: false,
            ctes: vec![(cte_alias.clone(), head_sql.to_string())],
            current_from: cte_alias,
            schema,
        }
    }

    fn build(&mut self, pipe: &PipeQuery) -> Result<String, PrismError> {
        for stage in &pipe.stages {
            self.apply_stage(stage)?;
        }
        Ok(self.assemble())
    }

    // -----------------------------------------------------------------------
    // Stage application
    // -----------------------------------------------------------------------

    fn apply_stage(&mut self, stage: &PipeStage) -> Result<(), PrismError> {
        match stage {
            PipeStage::Enrich(e) => self.apply_enrich(e),
            PipeStage::Where(pred) => self.apply_where(pred)?,
            PipeStage::Stats(stats) => self.apply_stats(stats)?,
            PipeStage::Sort(sort_exprs) => self.apply_sort(sort_exprs),
            PipeStage::Limit(n) => self.apply_limit(*n),
            PipeStage::Tail(n) => {
                // §3.2: tail N is lowered to LIMIT N with documented semantic gap.
                // Without a preceding sort stage, "last N" is not well-defined.
                // Treating it as LIMIT N matches analyst expectations for streaming contexts.
                self.apply_limit(*n);
            }
            PipeStage::Fields(fs) => self.apply_fields(fs)?,
            PipeStage::Dedup(dedup_fields) => self.apply_dedup(dedup_fields),
            PipeStage::Join(_) => {
                return Err(PrismError::QueryExecutionFailed {
                    detail: "JOIN in pipe mode is not yet supported (ENRICH-4-C). \
                             Remove the JOIN stage or rewrite as SQL."
                        .to_string(),
                });
            }
            // Non-exhaustive: unknown future stages are a no-op (forward compat).
            _ => {}
        }
        Ok(())
    }

    /// `enrich infusion(field)` → CTE: `SELECT *, infusion(field) AS infusion FROM prev`
    fn apply_enrich(&mut self, enrich: &crate::ast::EnrichStage) {
        let field_sql = field_path_to_sql(&enrich.field);
        let udf_name = escape_identifier(&enrich.infusion);
        let alias_name = escape_identifier(&enrich.infusion);

        // Inner SQL for this CTE: project everything from the previous source plus
        // the enriched column as `infusion_name`.
        let inner_sql = format!(
            "SELECT *, {udf_name}({field_sql}) AS {alias_name} FROM {}",
            self.current_from
        );

        // Allocate next CTE alias: _pipe_0, _pipe_1, ...
        let cte_alias = format!("_pipe_{}", self.ctes.len());
        self.ctes.push((cte_alias.clone(), inner_sql));
        self.current_from = cte_alias;
        // Update schema: after enrich the schema now includes the new column.
        // We can't know it statically, so set schema to None (no further Fields-exclude
        // projection can be schema-aware after an enrich — it will use the CTE columns).
        // In practice, Fields after Enrich is uncommon; DataFusion will catch invalid names.
        self.schema = None;
    }

    /// `where predicate` → append to WHERE clause (ANDed)
    fn apply_where(&mut self, pred: &Predicate) -> Result<(), PrismError> {
        let sql = predicate_to_datafusion_sql(pred)?;
        self.where_clauses.push(sql);
        Ok(())
    }

    /// `stats agg [, …] [by field, …]` → replaces SELECT clause entirely
    fn apply_stats(&mut self, stats: &StatsStage) -> Result<(), PrismError> {
        let mut items: Vec<String> = Vec::new();
        for sf in &stats.aggregates {
            let agg_sql = agg_func_to_sql(&sf.func)?;
            let item = match &sf.alias {
                Some(alias) => format!("{agg_sql} AS {}", escape_identifier(alias)),
                None => agg_sql,
            };
            items.push(item);
        }
        // Add GROUP BY columns to SELECT list (DataFusion requires this).
        for fp in &stats.by_fields {
            items.push(field_path_to_sql(fp));
        }
        self.select_items = items;
        self.group_by = stats.by_fields.iter().map(field_path_to_sql).collect();
        Ok(())
    }

    /// `sort field [asc|desc] [, …]` → ORDER BY clause
    fn apply_sort(&mut self, sort_exprs: &[crate::ast::SortExpr]) {
        for se in sort_exprs {
            let dir = match se.direction {
                SortDirection::Asc => "ASC",
                SortDirection::Desc => "DESC",
                _ => "ASC", // non_exhaustive arm
            };
            self.order_by
                .push(format!("{} {dir}", field_path_to_sql(&se.field)));
        }
    }

    /// `head N` / `limit N` → LIMIT clause (minimum across multiple stages)
    fn apply_limit(&mut self, n: u64) {
        self.limit = Some(match self.limit {
            None => n,
            Some(existing) => existing.min(n),
        });
    }

    /// `fields [+|-] col1, col2` → SELECT projection
    fn apply_fields(&mut self, fs: &FieldsStage) -> Result<(), PrismError> {
        if fs.include {
            // Include: `SELECT col1, col2, ... FROM ...`
            let cols: Vec<String> = fs.fields.iter().map(field_path_to_sql).collect();
            self.select_items = cols;
        } else {
            // Exclude: `SELECT retained_cols FROM ...` (DataFusion lacks EXCEPT syntax).
            // Requires schema knowledge; fall back gracefully if schema unavailable.
            //
            // IMPORTANT: compare bare (unescaped) identifiers on both sides.
            // `field_path_to_sql` quotes reserved words (e.g. `order` → `"order"`),
            // but `schema.fields()[i].name()` always returns the bare name.  Comparing
            // escaped against bare would silently fail to exclude reserved-word columns.
            let excluded_bare: Vec<String> =
                fs.fields.iter().map(|fp| fp.segments.join(".")).collect();
            match &self.schema {
                Some(schema) => {
                    let retained: Vec<String> = schema
                        .fields()
                        .iter()
                        .map(|f| f.name().clone())
                        .filter(|name| !excluded_bare.contains(name))
                        .map(|name| escape_identifier(&name))
                        .collect();
                    if retained.is_empty() {
                        // All columns excluded — return empty projection.
                        // DataFusion requires at least one SELECT item; use literal 1.
                        self.select_items = vec!["1 AS _empty".to_string()];
                    } else {
                        self.select_items = retained;
                    }
                }
                None => {
                    // Schema unavailable (e.g., after an Enrich CTE).
                    // Cannot build the exclude projection; fall back to SELECT *.
                    // This is a known limitation documented in §3.3.
                    tracing::warn!(
                        excluded = ?excluded_bare,
                        "pipe_sql_emitter: fields-exclude cannot be lowered \
                         without a schema (after an Enrich stage or when fan-out \
                         returned no batches); falling back to SELECT *"
                    );
                    // Keep select_items as-is (SELECT *).
                }
            }
        }
        Ok(())
    }

    /// `dedup field [, …]` → DISTINCT projection
    fn apply_dedup(&mut self, dedup_fields: &[FieldPath]) {
        self.distinct = true;
        if !dedup_fields.is_empty() {
            // If explicit dedup fields provided, project only those fields distinctly.
            self.select_items = dedup_fields.iter().map(field_path_to_sql).collect();
        }
        // If dedup_fields is empty, DISTINCT applies to the current SELECT list (SELECT DISTINCT *).
    }

    // -----------------------------------------------------------------------
    // Final SQL assembly
    // -----------------------------------------------------------------------

    fn assemble(&self) -> String {
        let distinct_kw = if self.distinct { "DISTINCT " } else { "" };
        let select_clause = format!("SELECT {distinct_kw}{}", self.select_items.join(", "));
        let from_clause = format!("FROM {}", self.current_from);

        let where_clause = if self.where_clauses.is_empty() {
            String::new()
        } else {
            // Wrap multi-predicate AND conditions in parens for clarity.
            let parts: Vec<String> = self
                .where_clauses
                .iter()
                .map(|p| {
                    if p.contains(" OR ") {
                        format!("({p})")
                    } else {
                        p.clone()
                    }
                })
                .collect();
            format!("WHERE {}", parts.join(" AND "))
        };

        let group_by_clause = if self.group_by.is_empty() {
            String::new()
        } else {
            format!("GROUP BY {}", self.group_by.join(", "))
        };

        let order_by_clause = if self.order_by.is_empty() {
            String::new()
        } else {
            format!("ORDER BY {}", self.order_by.join(", "))
        };

        let limit_clause = self.limit.map(|n| format!("LIMIT {n}")).unwrap_or_default();

        // Assemble the body (FROM + WHERE + GROUP BY + ORDER BY + LIMIT).
        let body = assemble_clauses(&[
            select_clause.as_str(),
            from_clause.as_str(),
            where_clause.as_str(),
            group_by_clause.as_str(),
            order_by_clause.as_str(),
            limit_clause.as_str(),
        ]);

        if self.ctes.is_empty() {
            body
        } else {
            // Wrap in WITH clause.
            let cte_defs: Vec<String> = self
                .ctes
                .iter()
                .map(|(alias, sql)| format!("{alias} AS ({sql})"))
                .collect();
            format!("WITH {} {}", cte_defs.join(", "), body)
        }
    }
}

// ---------------------------------------------------------------------------
// Helper: assemble non-empty clauses into a space-separated SQL fragment
// ---------------------------------------------------------------------------

fn assemble_clauses(clauses: &[&str]) -> String {
    clauses
        .iter()
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect::<Vec<_>>()
        .join(" ")
}

// ---------------------------------------------------------------------------
// Predicate-to-DataFusion-SQL translation
// ---------------------------------------------------------------------------

/// Translate a PQL `Predicate` to a DataFusion SQL fragment.
///
/// PQL predicate syntax diverges from SQL for `CONTAINS`, `HAS`, `MISSING`,
/// `CIDR`, and regex operators. See design doc §8 for the full mapping table.
///
/// # Errors
/// Returns `PrismError::QueryExecutionFailed` for predicates that cannot be
/// lowered to SQL (e.g., `InSubquery` — subqueries require additional MemTable
/// registration and are not supported in pipe-mode for MVP).
pub(crate) fn predicate_to_datafusion_sql(pred: &Predicate) -> Result<String, PrismError> {
    match pred {
        Predicate::Compare {
            lhs,
            op,
            rhs,
            case_insensitive,
        } => {
            // S-PRISMQL-CASE-INSENSITIVE-001: case-insensitive IEQ/INE operators lower
            // via `lower(field) OP lower('val')` DataFusion SQL pattern (BC-2.11.024).
            if *case_insensitive {
                // IEQ/INE RHS must be a string literal — lower() is string-only in DataFusion.
                // Non-string RHS → QueryPlanFailed (BC-2.11.024 error case, RG-016).
                let rhs_sql = match rhs.as_ref() {
                    crate::ast::Expr::Literal(Literal::String(s)) => {
                        format!("lower('{}')", escape_sql_string(s))
                    }
                    _ => {
                        return Err(PrismError::QueryPlanFailed {
                            detail: "IEQ/INE operators require a string literal \
                                     on the right-hand side; lower() is not applicable to \
                                     non-string values"
                                .to_string(),
                        });
                    }
                };
                let lhs_sql = expr_to_sql(lhs)?;
                // OBS-1 (S-PRISMQL-CASE-INSENSITIVE-001): IEQ maps to Eq, INE maps to Ne.
                // Any other CompareOp in a case_insensitive branch is a contract violation —
                // the parser only produces case_insensitive=true for IEQ/INE operators (BC-2.11.024).
                // Emit QueryPlanFailed rather than silently defaulting to "=" and producing
                // a wrong query.
                let op_str = match op {
                    CompareOp::Eq => "=",
                    CompareOp::Ne => "!=",
                    other => {
                        return Err(PrismError::QueryPlanFailed {
                            detail: format!(
                                "case_insensitive=true is only valid for IEQ (Eq) and INE (Ne) \
                                 operators; got {other:?} — this is an AST invariant violation"
                            ),
                        });
                    }
                };
                return Ok(format!("lower({lhs_sql}) {op_str} {rhs_sql}"));
            }
            let lhs_sql = expr_to_sql(lhs)?;
            let rhs_sql = expr_to_sql(rhs)?;
            let op_str = match op {
                CompareOp::Eq => "=",
                CompareOp::Ne => "!=",
                CompareOp::Gt => ">",
                CompareOp::Lt => "<",
                CompareOp::Ge => ">=",
                CompareOp::Le => "<=",
                CompareOp::Like => "LIKE",
                // CIDR in Compare position: use subnet_contains UDF.
                CompareOp::Cidr => {
                    // lhs is the field, rhs is the CIDR literal.
                    return Ok(format!("subnet_contains({rhs_sql}, {lhs_sql})"));
                }
                CompareOp::NotCidr => {
                    return Ok(format!("NOT subnet_contains({rhs_sql}, {lhs_sql})"));
                }
                _ => "=", // non_exhaustive arm
            };
            Ok(format!("{lhs_sql} {op_str} {rhs_sql}"))
        }

        Predicate::StringOp {
            field,
            op,
            pattern,
            case_insensitive,
        } => {
            let field_sql = field_path_to_sql(field);
            let escaped_pattern = escape_sql_string(pattern);
            // Map PQL string ops → DataFusion SQL LIKE patterns (design doc §8).
            let sql = match (op, case_insensitive) {
                (StringOp::Contains, false) => {
                    format!("{field_sql} LIKE '%{escaped_pattern}%'")
                }
                (StringOp::Contains, true) => {
                    format!("lower({field_sql}) LIKE lower('%{escaped_pattern}%')")
                }
                (StringOp::StartsWith, false) => {
                    format!("{field_sql} LIKE '{escaped_pattern}%'")
                }
                (StringOp::StartsWith, true) => {
                    format!("lower({field_sql}) LIKE lower('{escaped_pattern}%')")
                }
                (StringOp::EndsWith, false) => {
                    format!("{field_sql} LIKE '%{escaped_pattern}'")
                }
                (StringOp::EndsWith, true) => {
                    format!("lower({field_sql}) LIKE lower('%{escaped_pattern}')")
                }
                _ => {
                    // non_exhaustive arm: default to CONTAINS
                    format!("{field_sql} LIKE '%{escaped_pattern}%'")
                }
            };
            Ok(sql)
        }

        Predicate::Regex { field, pattern } => {
            // DataFusion SQL: `regexp_match(field, 'pattern') IS NOT NULL`
            let field_sql = field_path_to_sql(field);
            let pat = escape_sql_string(&pattern.pattern);
            Ok(format!("regexp_match({field_sql}, '{pat}') IS NOT NULL"))
        }

        Predicate::In {
            field,
            values,
            negated,
            case_insensitive,
        } => {
            // S-PRISMQL-CASE-INSENSITIVE-001: IIN operators lower via
            // `lower(field) IN (lower('v1'), ...)` DataFusion SQL pattern (BC-2.11.024).
            if *case_insensitive {
                // IIN is a parser-producible positive-only operator (grammar has no NIIN form).
                // A negated+case_insensitive In predicate cannot be produced by the parser
                // (BC-2.11.024 §AC-023); guard against direct AST construction.
                if *negated {
                    return Err(PrismError::QueryPlanFailed {
                        detail: "negated + case_insensitive is not a parser-producible IN \
                                 combination; IIN grammar is positive-only"
                            .to_string(),
                    });
                }
                if values.is_empty() {
                    return Err(PrismError::QueryPlanFailed {
                        detail: "IIN requires at least one value in the membership list"
                            .to_string(),
                    });
                }
                let field_sql = field_path_to_sql(field);
                let vals: Vec<String> = values
                    .iter()
                    .map(|lit| match lit {
                        Literal::String(s) => Ok(format!("lower('{}')", escape_sql_string(s))),
                        _ => Err(PrismError::QueryPlanFailed {
                            detail: "IIN requires string literals in the membership list; \
                                     lower() is not applicable to non-string values"
                                .to_string(),
                        }),
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                return Ok(format!("lower({field_sql}) IN ({})", vals.join(", ")));
            }
            let field_sql = field_path_to_sql(field);
            let vals: Vec<String> = values
                .iter()
                .map(literal_to_sql)
                .collect::<Result<Vec<_>, _>>()?;
            let not_kw = if *negated { "NOT IN" } else { "IN" };
            Ok(format!("{field_sql} {not_kw} ({})", vals.join(", ")))
        }

        Predicate::Between {
            field,
            low,
            high,
            negated,
        } => {
            let field_sql = field_path_to_sql(field);
            let not_kw = if *negated { "NOT BETWEEN" } else { "BETWEEN" };
            Ok(format!(
                "{field_sql} {not_kw} {} AND {}",
                literal_to_sql(low)?,
                literal_to_sql(high)?
            ))
        }

        Predicate::Cidr {
            field,
            cidr,
            negated,
        } => {
            // `field IN CIDR '10.0.0.0/8'` → `subnet_contains('10.0.0.0/8', field)` UDF
            let field_sql = field_path_to_sql(field);
            let cidr_lit = escape_sql_string(&cidr.cidr);
            if *negated {
                Ok(format!("NOT subnet_contains('{cidr_lit}', {field_sql})"))
            } else {
                Ok(format!("subnet_contains('{cidr_lit}', {field_sql})"))
            }
        }

        // `HAS field` → `field IS NOT NULL` (design doc §8)
        Predicate::Has(fp) => Ok(format!("{} IS NOT NULL", field_path_to_sql(fp))),

        // `MISSING field` → `field IS NULL` (design doc §8)
        Predicate::Missing(fp) => Ok(format!("{} IS NULL", field_path_to_sql(fp))),

        Predicate::IsNull { field, negated } => {
            let field_sql = field_path_to_sql(field);
            if *negated {
                Ok(format!("{field_sql} IS NOT NULL"))
            } else {
                Ok(format!("{field_sql} IS NULL"))
            }
        }

        Predicate::Wildcard {
            field,
            pattern,
            negated,
        } => {
            // Wildcard: `*` → SQL `%`, `?` → SQL `_`.
            let field_sql = field_path_to_sql(field);
            let sql_pattern = pattern.replace('*', "%").replace('?', "_");
            let escaped = escape_sql_string(&sql_pattern);
            if *negated {
                Ok(format!("{field_sql} NOT LIKE '{escaped}'"))
            } else {
                Ok(format!("{field_sql} LIKE '{escaped}'"))
            }
        }

        Predicate::Logical { op, predicates } => {
            let op_str = match op {
                LogicalOp::And => "AND",
                LogicalOp::Or => "OR",
                _ => "AND", // non_exhaustive arm
            };
            let parts: Vec<String> = predicates
                .iter()
                .map(|p| {
                    let inner = predicate_to_datafusion_sql(p)?;
                    // Wrap OR predicates in parens when inside an AND context.
                    Ok(
                        if matches!(p, Predicate::Logical { op: inner_op, .. }
                            if matches!(inner_op, LogicalOp::Or)
                                && matches!(op, LogicalOp::And))
                        {
                            format!("({inner})")
                        } else {
                            inner
                        },
                    )
                })
                .collect::<Result<Vec<_>, PrismError>>()?;
            Ok(parts.join(&format!(" {op_str} ")))
        }

        Predicate::Not(inner) => {
            let inner_sql = predicate_to_datafusion_sql(inner)?;
            Ok(format!("NOT ({inner_sql})"))
        }

        Predicate::InSubquery { .. } => Err(PrismError::QueryExecutionFailed {
            detail: "IN (SELECT …) subquery predicates in pipe-mode WHERE are not yet \
                     supported. Rewrite as SQL for subquery capability."
                .to_string(),
        }),

        // RecoveryError: emit a sentinel that always evaluates to false.
        Predicate::RecoveryError => Ok("(1 = 0)".to_string()),

        // Non-exhaustive: unknown future predicates → return false (fail-safe).
        _ => Ok("(1 = 0)".to_string()),
    }
}

// ---------------------------------------------------------------------------
// Aggregate function to SQL
// ---------------------------------------------------------------------------

fn agg_func_to_sql(f: &AggFunc) -> Result<String, PrismError> {
    let sql = match f {
        AggFunc::Count => "count(*)".to_string(),
        AggFunc::CountField(fp) => format!("count({})", field_path_to_sql(fp)),
        AggFunc::Sum(fp) => format!("sum({})", field_path_to_sql(fp)),
        AggFunc::Avg(fp) => format!("avg({})", field_path_to_sql(fp)),
        AggFunc::Min(fp) => format!("min({})", field_path_to_sql(fp)),
        AggFunc::Max(fp) => format!("max({})", field_path_to_sql(fp)),
        AggFunc::DistinctCount(fp) => format!("count(DISTINCT {})", field_path_to_sql(fp)),
        AggFunc::Percentile { field, p } => {
            // DataFusion 53.x: approx_percentile_cont(field, p/100.0)
            let pct = p.0 / 100.0;
            format!(
                "approx_percentile_cont({}, {})",
                field_path_to_sql(field),
                pct
            )
        }
        // Non-exhaustive: unknown future aggregate functions → count(*) as safe fallback.
        _ => "count(*)".to_string(),
    };
    Ok(sql)
}

// ---------------------------------------------------------------------------
// Expression to SQL
// ---------------------------------------------------------------------------

fn expr_to_sql(expr: &Expr) -> Result<String, PrismError> {
    match expr {
        Expr::Literal(lit) => literal_to_sql(lit),
        Expr::Field(fp) => Ok(field_path_to_sql(fp)),
        Expr::VirtualField(vf) => {
            use crate::ast::VirtualField;
            Ok(match vf {
                VirtualField::Sensor => "_sensor".to_string(),
                VirtualField::Client => "_client".to_string(),
                VirtualField::SourceTable => "_source_table".to_string(),
                VirtualField::SourceType => "_source_type".to_string(),
                VirtualField::SafetyFlags => "_safety_flags".to_string(),
                _ => "_unknown_virtual_field".to_string(), // non_exhaustive arm
            })
        }
        Expr::Compare { lhs, op, rhs } => {
            let lhs_sql = expr_to_sql(lhs)?;
            let rhs_sql = expr_to_sql(rhs)?;
            let op_str = match op {
                CompareOp::Eq => "=",
                CompareOp::Ne => "!=",
                CompareOp::Gt => ">",
                CompareOp::Lt => "<",
                CompareOp::Ge => ">=",
                CompareOp::Le => "<=",
                CompareOp::Like => "LIKE",
                _ => "=",
            };
            Ok(format!("{lhs_sql} {op_str} {rhs_sql}"))
        }
        Expr::Star => Ok("*".to_string()),
        // Temporal arithmetic: `'<iso>' ± INTERVAL '<n> seconds'`.
        //
        // After `inject_now` is called (BC-2.11.021), `Expr::Now` nodes are
        // replaced with `Expr::Literal(Literal::Timestamp(now))` and the outer
        // `TimestampArithmetic` is constant-folded. This arm fires only if folding
        // did not complete (defensive code path; should not occur in production).
        //
        // The base expression emits via expr_to_sql (which for Literal::Timestamp
        // emits the arrow_cast form per ADR-052 D3). Arithmetic on a Timestamp
        // via INTERVAL is supported by DataFusion. This arm is a defensive fallback
        // for unfold-able expressions; inject_now must fold these before emission.
        // If DataFusion rejects the emitted SQL, the query surfaces a
        // QueryExecutionFailed error (acceptable — the spec requires inject_now
        // to fold these before emission).
        // Using seconds as the canonical unit avoids ambiguity between calendar
        // days and SI days. `chrono::Duration::num_seconds()` is exact for all
        // sub-day durations; for whole-day durations it is `n * 86400`.
        Expr::TimestampArithmetic { base, op, offset } => {
            use crate::ast::BinaryOp;
            let base_sql = expr_to_sql(base)?;
            let op_str = match op {
                BinaryOp::Sub => "-",
                BinaryOp::Add => "+",
                _ => "-", // non_exhaustive arm — Sub is the only grammar-producible op
            };
            let secs = offset.num_seconds();
            Ok(format!("{base_sql} {op_str} INTERVAL '{secs} seconds'"))
        }
        // Non-exhaustive: FuncCall, Logical, Not, In, InSubquery → simplified fallback.
        _ => Err(PrismError::QueryExecutionFailed {
            detail: "Complex expression in pipe WHERE stage is not yet supported. \
                     Rewrite as SQL."
                .to_string(),
        }),
    }
}

// ---------------------------------------------------------------------------
// Literal to SQL
// ---------------------------------------------------------------------------

/// Convert a `Literal` to its SQL string representation.
///
/// Returns `Err` only for `Literal::RawTemporalLiteral` — that intermediate AST node
/// must be consumed by `check_temporal_literals` at plan time before the emitter
/// is called (belt-and-suspenders guard; ADR-052 §D4 Step 5; BC-2.11.021;
/// S-PRISMQL-NATIVE-TEMPORAL-TYPING-001 Task 11B).
fn literal_to_sql(lit: &Literal) -> Result<String, PrismError> {
    Ok(match lit {
        Literal::String(s) => format!("'{}'", escape_sql_string(s)),
        Literal::Integer(n) => n.to_string(),
        Literal::Float(f) => {
            if f.fract() == 0.0 {
                format!("{:.1}", f.0)
            } else {
                f.to_string()
            }
        }
        Literal::Bool(b) => b.to_string(),
        Literal::Null => "NULL".to_string(),
        Literal::Duration(d) => {
            // Emit as seconds for DataFusion; analysts using duration literals in WHERE
            // are comparing against integer epoch seconds.
            d.to_secs().to_string()
        }
        Literal::Cidr(c) => format!("'{}'", escape_sql_string(&c.cidr)),
        Literal::Regex(r) => format!("'{}'", escape_sql_string(&r.pattern)),
        Literal::IpAddr(ip) => format!("'{}'", ip.0 .0),
        // ADR-052 D3 / BC-2.11.004: The materialized Arrow column type for OCSF
        // Datetime fields is now DataType::Timestamp(Microsecond, UTC) per ADR-052 D1/D2.
        // DataFusion 53.1.0's `TIMESTAMP '<iso>'` syntax produces Timestamp(Nanosecond, None),
        // which mismatches the UTC-tagged Microsecond column type. Use arrow_cast() to produce
        // the correct Timestamp(Microsecond, Some("UTC")) typed literal.
        // (Supersedes ADR-044 D4 F-HIGH-002 comment; ADR-044 superseded by ADR-052.)
        //
        // SEC-001 (MED-1 sibling sweep): escape single-quotes in iso8601 before interpolating
        // into the arrow_cast first argument. RFC-3339 validation in TimestampLiteral::new()
        // prevents embedded `'` via the parser path, but direct in-crate AST construction
        // is a reachable injection vector. Matches the escape applied in
        // ast.rs::normalize_literal_for_datafusion (TD-VSDD-060 sibling sweep).
        // For valid RFC-3339 (no quotes): escape_sql_string is a no-op — byte-identical
        // to the pre-fix form (RG-003/RG-010 byte-identity preserved).
        Literal::Timestamp(ts) => format!(
            "arrow_cast('{}', 'Timestamp(Microsecond, Some(\"UTC\"))')",
            escape_sql_string(&ts.iso8601)
        ),
        // ADR-052 §D4 Step 5 guard (BC-2.11.021; S-PRISMQL-NATIVE-TEMPORAL-TYPING-001):
        // Belt-and-suspenders secondary defense: RawTemporalLiteral must NEVER reach SQL
        // emission. It is an intermediate AST node that check_temporal_literals must
        // consume at plan time via seven-arm dispatch (ADR-052 §D4 v1.10):
        //   (1) Datetime col → E-QUERY-041; (2) String col → COERCE; (3) Integer/Float/Bool → E-QUERY-002;
        //   (4) non-Field LHS → E-QUERY-042 NonColumnLhsComparison; (5) SELECT projection → COERCE;
        //   (6) GROUP BY → E-QUERY-042 GroupBy; (7) ORDER BY → E-QUERY-042 OrderBy.
        // When the column type is unresolvable (fail-open in the walker), the walker leaves
        // the RawTemporalLiteral in the AST; this guard catches it as the secondary gate,
        // returning E-QUERY-002 (QueryPlanFailed).
        //
        // Asymmetry between Pipe/Filter and SQL mode (intentional, ADR-sanctioned):
        //   Pipe/Filter (this guard): FAIL-CLOSED — returns Err(QueryPlanFailed) E-QUERY-002.
        //   SQL mode (ast.rs::normalize_literal): FAIL-OPEN — emits a plain quoted string
        //     (`Self::emit_quoted_string(s)`) so DataFusion acts as the tertiary correctness gate.
        //   The SQL-mode fail-open is ADR-sanctioned: E-QUERY-041 is a message upgrade;
        //   DataFusion rejects or correctly handles the quoted string at execution time.
        //
        // HIGH-3 fix: use QueryPlanFailed (plan-time invariant violation) not QueryParseFailed
        // (parse-time failure). This literal was parsed successfully; the gate runs post-parse.
        Literal::RawTemporalLiteral(s) => {
            return Err(PrismError::QueryPlanFailed {
                detail: format!(
                    "internal error — unvalidated RawTemporalLiteral '{s}' reached SQL \
                     emission; check_temporal_literals must run before emission"
                ),
            });
        }
        _ => "NULL".to_string(), // non_exhaustive arm
    })
}

// ---------------------------------------------------------------------------
// Field path helpers
// ---------------------------------------------------------------------------

/// Convert a `FieldPath` to a SQL column reference.
///
/// Single-segment paths are emitted as plain identifiers (e.g., `severity`).
/// Multi-segment paths are emitted dot-joined (e.g., `device.hostname`) which
/// DataFusion resolves against Arrow struct columns.
fn field_path_to_sql(fp: &FieldPath) -> String {
    fp.segments
        .iter()
        .map(|s| escape_identifier(s))
        .collect::<Vec<_>>()
        .join(".")
}

// ---------------------------------------------------------------------------
// SQL escaping helpers
// ---------------------------------------------------------------------------

/// Escape a SQL string literal value (single-quote escape: `'` → `''`).
fn escape_sql_string(s: &str) -> String {
    s.replace('\'', "''")
}

/// Escape a SQL identifier if it contains special characters or is a reserved word.
///
/// For simplicity: identifiers containing non-alphanumeric/underscore characters,
/// or starting with a digit, are double-quoted. Most PrismQL column names are
/// safe ([a-zA-Z0-9_]), so the common case emits plain identifiers.
fn escape_identifier(name: &str) -> String {
    let needs_quoting = name.is_empty()
        || name.starts_with(|c: char| c.is_ascii_digit())
        || name.contains(|c: char| !c.is_ascii_alphanumeric() && c != '_')
        || is_reserved_word(name);
    if needs_quoting {
        format!("\"{}\"", name.replace('"', "\"\""))
    } else {
        name.to_string()
    }
}

/// Returns `true` if `name` is a DataFusion SQL reserved word that must be quoted.
///
/// This list covers the most common cases for prism sensor column names. It is
/// intentionally conservative — quoting a non-reserved identifier is harmless.
fn is_reserved_word(name: &str) -> bool {
    matches!(
        name.to_uppercase().as_str(),
        "FROM"
            | "SELECT"
            | "WHERE"
            | "GROUP"
            | "ORDER"
            | "LIMIT"
            | "BY"
            | "AS"
            | "JOIN"
            | "ON"
            | "INNER"
            | "LEFT"
            | "RIGHT"
            | "OUTER"
            | "FULL"
            | "CROSS"
            | "WITH"
            | "TABLE"
            | "INDEX"
            | "VALUE"
            | "VALUES"
            | "DISTINCT"
            | "HAVING"
            | "UNION"
            | "INTERSECT"
            | "EXCEPT"
            | "ALL"
            | "AND"
            | "OR"
            | "NOT"
            | "IN"
            | "IS"
            | "NULL"
            | "TRUE"
            | "FALSE"
            | "LIKE"
            | "BETWEEN"
            | "CASE"
            | "WHEN"
            | "THEN"
            | "ELSE"
            | "END"
            | "EXISTS"
            | "TIMESTAMP"
    )
}

// ---------------------------------------------------------------------------
// Unit tests — pure SQL emission (no SessionContext required)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{EnrichStage, FieldPath, PipeQuery, PipeStage, Predicate, SourceRef};

    fn build_sql(pipe: &PipeQuery) -> String {
        pipe_to_executable_sql(pipe, &Default::default()).expect("must succeed")
    }

    #[test]
    fn test_source_table_name_underscore_passthrough() {
        assert_eq!(
            source_table_name("crowdstrike_detections"),
            "crowdstrike_detections"
        );
    }

    #[test]
    fn test_source_table_name_dot_to_underscore() {
        assert_eq!(
            source_table_name("crowdstrike.detections"),
            "crowdstrike_detections"
        );
    }

    #[test]
    fn test_simple_pipe_no_stages() {
        let pipe = PipeQuery::new(SourceRef::from_raw("cs_events"), vec![]);
        let sql = build_sql(&pipe);
        assert_eq!(sql, "SELECT * FROM cs_events");
    }

    #[test]
    fn test_pipe_where_eq() {
        let pipe = PipeQuery::new(
            SourceRef::from_raw("cs_events"),
            vec![PipeStage::Where(Predicate::Compare {
                lhs: Box::new(Expr::Field(FieldPath::new(["status"]))),
                op: CompareOp::Eq,
                rhs: Box::new(Expr::Literal(Literal::String("active".to_string()))),
                case_insensitive: false,
            })],
        );
        let sql = build_sql(&pipe);
        assert!(sql.contains("WHERE status = 'active'"), "got: {sql}");
    }

    #[test]
    fn test_pipe_head_limit() {
        let pipe = PipeQuery::new(SourceRef::from_raw("tbl"), vec![PipeStage::Limit(10)]);
        let sql = build_sql(&pipe);
        assert!(sql.contains("LIMIT 10"), "got: {sql}");
    }

    #[test]
    fn test_pipe_stats_count_star() {
        use crate::ast::{StatFunction, StatsStage};
        let pipe = PipeQuery::new(
            SourceRef::from_raw("tbl"),
            vec![PipeStage::Stats(StatsStage {
                aggregates: vec![StatFunction {
                    func: AggFunc::Count,
                    alias: None,
                }],
                by_fields: vec![],
            })],
        );
        let sql = build_sql(&pipe);
        assert!(sql.contains("count(*)"), "got: {sql}");
    }

    #[test]
    fn test_pipe_sort_desc() {
        use crate::ast::{SortDirection, SortExpr};
        let pipe = PipeQuery::new(
            SourceRef::from_raw("tbl"),
            vec![PipeStage::Sort(vec![SortExpr {
                field: FieldPath::new(["severity"]),
                direction: SortDirection::Desc,
            }])],
        );
        let sql = build_sql(&pipe);
        assert!(sql.contains("ORDER BY severity DESC"), "got: {sql}");
    }

    #[test]
    fn test_pipe_fields_include() {
        use crate::ast::FieldsStage;
        let pipe = PipeQuery::new(
            SourceRef::from_raw("tbl"),
            vec![PipeStage::Fields(FieldsStage {
                include: true,
                fields: vec![FieldPath::new(["col1"]), FieldPath::new(["col2"])],
            })],
        );
        let sql = build_sql(&pipe);
        assert!(sql.contains("SELECT col1, col2"), "got: {sql}");
        assert!(!sql.contains("SELECT *"), "got: {sql}");
    }

    #[test]
    fn test_pipe_enrich_produces_cte() {
        let pipe = PipeQuery::new(
            SourceRef::from_raw("cs_tbl"),
            vec![PipeStage::Enrich(EnrichStage::new(
                "my_udf",
                FieldPath::new(["ip"]),
            ))],
        );
        let sql = build_sql(&pipe);
        assert!(sql.starts_with("WITH "), "got: {sql}");
        assert!(sql.contains("my_udf(ip) AS my_udf"), "got: {sql}");
    }

    #[test]
    fn test_pipe_has_field_is_not_null() {
        let pipe = PipeQuery::new(
            SourceRef::from_raw("tbl"),
            vec![PipeStage::Where(Predicate::Has(FieldPath::new([
                "field_x",
            ])))],
        );
        let sql = build_sql(&pipe);
        assert!(sql.contains("field_x IS NOT NULL"), "got: {sql}");
    }

    #[test]
    fn test_pipe_missing_field_is_null() {
        let pipe = PipeQuery::new(
            SourceRef::from_raw("tbl"),
            vec![PipeStage::Where(Predicate::Missing(FieldPath::new([
                "field_y",
            ])))],
        );
        let sql = build_sql(&pipe);
        assert!(sql.contains("field_y IS NULL"), "got: {sql}");
    }

    #[test]
    fn test_pipe_contains_to_like() {
        let pipe = PipeQuery::new(
            SourceRef::from_raw("tbl"),
            vec![PipeStage::Where(Predicate::StringOp {
                field: FieldPath::new(["msg"]),
                op: StringOp::Contains,
                pattern: "critical".to_string(),
                case_insensitive: false,
            })],
        );
        let sql = build_sql(&pipe);
        assert!(sql.contains("msg LIKE '%critical%'"), "got: {sql}");
    }

    /// OBS-1 load-bearing test: `fields -` exclude of a reserved-word column name.
    ///
    /// Before the fix, `excluded` was built via `field_path_to_sql` which quotes reserved
    /// words (`order` → `"order"`), while `schema.fields().name()` returns the bare name
    /// `order`.  The `contains` check compared quoted against bare → mismatch → `order`
    /// was silently retained in the projection instead of being excluded.
    ///
    /// After the fix, `excluded_bare` collects raw segments (no escaping), so the
    /// comparison is bare-vs-bare and the exclude works correctly.
    #[test]
    fn test_pipe_fields_exclude_reserved_word_column() {
        use arrow::datatypes::{DataType, Field, Schema};
        use arrow::record_batch::RecordBatch;
        use std::collections::HashMap;
        use std::sync::Arc;

        // Build a schema with columns: "value", "order" (reserved word), "status"
        let schema = Arc::new(Schema::new(vec![
            Field::new("value", DataType::Utf8, true),
            Field::new("order", DataType::Utf8, true), // reserved word — would be quoted by escape_identifier
            Field::new("status", DataType::Utf8, true),
        ]));
        let batch = RecordBatch::new_empty(schema);
        let mut table_batches: HashMap<String, Vec<RecordBatch>> = HashMap::new();
        table_batches.insert("tbl".to_string(), vec![batch]);

        // `fields - order` — exclude the reserved-word column
        let pipe = PipeQuery::new(
            SourceRef::from_raw("tbl"),
            vec![PipeStage::Fields(FieldsStage {
                include: false,
                fields: vec![FieldPath::new(["order"])],
            })],
        );

        let sql = pipe_to_executable_sql(&pipe, &table_batches).expect("must succeed");

        // "order" must NOT appear in the projection (it was excluded).
        // The SELECT clause should contain "value" and "status" but NOT "order".
        assert!(
            !sql.contains("\"order\""),
            "OBS-1: reserved-word column 'order' must be excluded from SELECT; got: {sql}"
        );
        assert!(
            sql.contains("value"),
            "OBS-1: 'value' must be retained; got: {sql}"
        );
        assert!(
            sql.contains("status"),
            "OBS-1: 'status' must be retained; got: {sql}"
        );
    }

    #[test]
    fn test_pipe_join_returns_err() {
        use crate::ast::{JoinCondition, JoinKind, JoinStage};
        let pipe = PipeQuery::new(
            SourceRef::from_raw("tbl"),
            vec![PipeStage::Join(JoinStage {
                kind: JoinKind::Inner,
                source: SourceRef::from_raw("other_tbl"),
                on: JoinCondition::SameField(FieldPath::new(["id"])),
            })],
        );
        let result = pipe_to_executable_sql(&pipe, &Default::default());
        assert!(result.is_err(), "JOIN should return Err");
    }

    // -----------------------------------------------------------------------
    // SEC-001 MED-1 sibling sweep — injection guard for literal_to_sql Timestamp arm
    // -----------------------------------------------------------------------

    /// SEC-001 (MED-1 sibling sweep): `literal_to_sql(Literal::Timestamp(...))` must
    /// SQL-double any single-quote embedded in `TimestampLiteral.iso8601` so the
    /// `arrow_cast` first argument is injection-safe.
    ///
    /// This mirrors `test_sec_001_normalize_literal_for_datafusion_escapes_single_quote`
    /// in `ast.rs` but targets `literal_to_sql` in `pipe_sql_emitter.rs` — the sibling
    /// DataFusion-executed emission site identified by TD-VSDD-060 (sibling-site sweep).
    ///
    /// RFC-3339 validation in `TimestampLiteral::new()` prevents embedded `'` via the
    /// parser path; this test covers the direct-AST-construction vector (defense-in-depth).
    ///
    /// Verifies: single-quote is SQL-doubled (`'` → `''`), NOT passed through raw.
    #[test]
    fn test_sec_001_pipe_sql_emitter_timestamp_escapes_single_quote() {
        use crate::ast::TimestampLiteral;
        use chrono::Utc;
        // Adversarially-constructed TimestampLiteral: parser-unreachable but reachable
        // via direct struct construction inside the crate.
        let ts_lit = Literal::Timestamp(TimestampLiteral {
            iso8601: "2026-07-03T00:00:00Z' OR '1'='1".to_string(),
            instant: Utc::now(),
        });
        let emitted = literal_to_sql(&ts_lit)
            .expect("Literal::Timestamp must not return Err from literal_to_sql");
        // Must NOT contain the raw injection sequence.
        assert!(
            !emitted.contains("Z' OR '1'='1"),
            "SEC-001 MED-1: raw single-quote injection must be neutralized in literal_to_sql. \
             Got: {emitted:?}"
        );
        // Must contain the SQL-doubled form inside the arrow_cast first argument.
        assert!(
            emitted.contains("Z'' OR ''1''=''1"),
            "SEC-001 MED-1: single-quotes in iso8601 must be SQL-doubled (`'` → `''`) in \
             literal_to_sql. Got: {emitted:?}"
        );
    }

    /// SEC-001 MED-1 byte-identity: for valid RFC-3339 (no single-quotes),
    /// `escape_sql_string` is a no-op — output is byte-identical to the pre-fix form.
    ///
    /// This guards RG-003/RG-010 — those tests assert the EXACT
    /// `arrow_cast('2026-07-03T00:00:00Z', 'Timestamp(…)')` string. If the escape
    /// were not a no-op for clean input, it would break those tests.
    #[test]
    #[allow(clippy::expect_used)]
    fn test_sec_001_pipe_sql_emitter_timestamp_noop_for_valid_rfc3339() {
        use crate::ast::TimestampLiteral;
        use chrono::Utc;
        let instant = chrono::DateTime::parse_from_rfc3339("2026-07-03T00:00:00Z")
            .expect("known-good RFC-3339 must parse")
            .with_timezone(&Utc);
        let ts_lit = Literal::Timestamp(TimestampLiteral {
            iso8601: "2026-07-03T00:00:00Z".to_string(),
            instant,
        });
        let emitted = literal_to_sql(&ts_lit)
            .expect("Literal::Timestamp must not return Err from literal_to_sql");
        let expected =
            r#"arrow_cast('2026-07-03T00:00:00Z', 'Timestamp(Microsecond, Some("UTC"))')"#;
        assert_eq!(
            emitted, expected,
            "SEC-001 MED-1 byte-identity: for valid RFC-3339, literal_to_sql \
             must produce the exact arrow_cast form (guards RG-003/RG-010). \
             Got: {emitted:?}"
        );
    }

    // -----------------------------------------------------------------------
    // RG-003 / RG-010 — S-PRISMQL-NATIVE-TEMPORAL-TYPING-001 Red Gate tests
    // -----------------------------------------------------------------------

    /// RG-003: `literal_to_sql(Literal::Timestamp(...))` must emit the `arrow_cast(...)` form
    /// per ADR-052 D3 v1.1, NOT a bare single-quoted ISO string.
    ///
    /// # Red Gate pre-implementation failure
    /// The `Literal::Timestamp` arm currently emits `"'2026-07-03T00:00:00Z'"` (bare string).
    /// The assertion FAILS with:
    ///   left:  `"'2026-07-03T00:00:00Z'"`
    ///   right: `"arrow_cast('2026-07-03T00:00:00Z', 'Timestamp(Microsecond, Some(\"UTC\"))')"`
    ///
    /// # Why load-bearing (ADR-052 D3)
    /// The bare `'...'` form causes DataFusion to see a `Utf8` literal vs a
    /// `Timestamp(Microsecond, UTC)` column — a type mismatch that produces wrong comparisons
    /// or a plan error. The `arrow_cast(...)` form produces `Timestamp(Microsecond, UTC)`,
    /// matching the column type exactly.
    ///
    /// # Emitted form (post-implementation)
    /// `arrow_cast('2026-07-03T00:00:00Z', 'Timestamp(Microsecond, Some("UTC"))')`
    /// Note: inner double-quote escaping in the type string is REQUIRED to match
    /// DataFusion's arrow_cast signature expectation.
    ///
    /// Traces to: ADR-052 v1.1 D3; BC-2.11.021 §Postconditions ("DataFusion sees
    /// a concrete `WHERE timestamp > arrow_cast(...)` comparison").
    #[test]
    #[allow(clippy::expect_used)]
    fn test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_pipe_sql_emitter_yields_arrow_cast_literal() {
        use crate::ast::TimestampLiteral;
        use chrono::Utc;

        // Use a fixed, known RFC-3339 timestamp for deterministic output (TD-VSDD-091).
        let instant = chrono::DateTime::parse_from_rfc3339("2026-07-03T00:00:00Z")
            .expect("known-good RFC-3339 must parse")
            .with_timezone(&Utc);

        let ts_lit = Literal::Timestamp(TimestampLiteral {
            iso8601: "2026-07-03T00:00:00Z".to_string(),
            instant,
        });

        let emitted = literal_to_sql(&ts_lit)
            .expect("Literal::Timestamp must not return Err from literal_to_sql");

        // The expected form includes escaped double-quotes inside the type string
        // (DataFusion's arrow_cast type argument uses `Some("UTC")` with quotes).
        let expected =
            r#"arrow_cast('2026-07-03T00:00:00Z', 'Timestamp(Microsecond, Some("UTC"))')"#;

        assert_eq!(
            emitted, expected,
            "RG-003: literal_to_sql(Literal::Timestamp(...)) must emit the arrow_cast form \
             per ADR-052 D3. Currently emits bare single-quoted string '{{}}'.  \
             Expected: {expected:?}. Got: {emitted:?}. \
             Fix: update the Literal::Timestamp arm in pipe_sql_emitter.rs \
             (Task 11 of S-PRISMQL-NATIVE-TEMPORAL-TYPING-001)."
        );
    }

    /// RG-010: The ACTUAL emitter output must be the `arrow_cast(...)` form AND must
    /// successfully plan against a `Timestamp(Microsecond, Some("UTC"))` DataFusion column.
    ///
    /// # Red Gate pre-implementation failure (TWO-STAGE)
    ///
    /// **Stage 1 (string assertion, immediately fails):**
    /// The emitter currently produces `'2026-07-03T00:00:00Z'` (bare Utf8 literal).
    /// The assertion `emitted_fragment == expected_arrow_cast_form` FAILS with:
    ///   left:  `"'2026-07-03T00:00:00Z'"`
    ///   right: `"arrow_cast('2026-07-03T00:00:00Z', 'Timestamp(Microsecond, Some(\"UTC\"))')"`
    ///
    /// **Stage 2 (DataFusion plan, only reached post-implementation):**
    /// Once the emitter produces the `arrow_cast(...)` form, this test verifies it actually
    /// plans without error. This is the transitive coverage step — if the format string has
    /// a quoting or escaping mistake invisible to string comparison, DataFusion will reject
    /// the malformed type string in `arrow_cast`.
    ///
    /// # Why load-bearing (transitive gap between RG-002 and RG-003)
    /// - RG-002: hand-writes the `arrow_cast(...)` query string (probe, confirming DF supports it).
    /// - RG-003: tests the emitter's string output in isolation (unit test, confirming form).
    /// - RG-010: closes the gap — takes ACTUAL emitter output and proves it PLANS correctly
    ///   against a `Timestamp(Microsecond, UTC)` column. Catches quoting/escaping mistakes
    ///   that string comparison alone cannot detect.
    ///
    /// # Note on DataFusion implicit coercion
    /// DataFusion 53.1.0 with arrow-cast 58.2.0 WILL implicitly coerce bare string literals
    /// to Timestamp when compared against a Timestamp column. The `arrow_cast(...)` form
    /// is still required because: (a) explicit typing is a contract (ADR-052 D3), (b) the
    /// coercion behavior may change in future DataFusion versions, (c) the `arrow_cast`
    /// form ensures the EXACT Timestamp type (`Microsecond, UTC`) is used in the comparison,
    /// not whatever DataFusion's implicit coercion produces.
    ///
    /// Traces to: ADR-052 v1.1 D3; BC-2.11.021 §Postconditions.
    #[tokio::test]
    #[allow(clippy::expect_used, clippy::unwrap_used)]
    async fn test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_emitter_output_plans_against_timestamp_column(
    ) {
        use crate::ast::TimestampLiteral;
        use crate::materialization::register_mem_table;
        use crate::memory::build_session_context;
        use arrow::datatypes::{DataType, Field, Schema, TimeUnit};
        use chrono::Utc;
        use std::sync::Arc;

        // Step 1: Get the ACTUAL emitter output for a fixed timestamp literal.
        let instant = chrono::DateTime::parse_from_rfc3339("2026-07-03T00:00:00Z")
            .expect("known-good RFC-3339 must parse")
            .with_timezone(&Utc);
        let ts_lit = Literal::Timestamp(TimestampLiteral {
            iso8601: "2026-07-03T00:00:00Z".to_string(),
            instant,
        });
        let emitted_fragment = literal_to_sql(&ts_lit)
            .expect("Literal::Timestamp must not return Err from literal_to_sql");

        // Step 2: RED GATE ASSERTION — emitter must produce the arrow_cast form.
        // FAILS before implementation: emitter still produces bare '...' string.
        // PASSES after Task 11: emitter produces arrow_cast('...', 'Timestamp(...)').
        let expected_fragment =
            r#"arrow_cast('2026-07-03T00:00:00Z', 'Timestamp(Microsecond, Some("UTC"))')"#;
        assert_eq!(
            emitted_fragment, expected_fragment,
            "RG-010 stage-1: literal_to_sql(Literal::Timestamp(...)) must emit the arrow_cast \
             form per ADR-052 D3 before the DataFusion plan step can proceed. \
             Currently emits bare string. Expected: {expected_fragment:?}. \
             Got: {emitted_fragment:?}. Fix: Task 11 (update Timestamp arm in literal_to_sql)."
        );

        // Step 3: DataFusion plan step — verifies the actual emitter output plans correctly.
        // Only reached post-implementation (step 2 is the Red Gate).
        // Catches quoting/escaping mistakes invisible to string comparison.
        let schema = Arc::new(Schema::new(vec![Field::new(
            "ts_col",
            DataType::Timestamp(TimeUnit::Microsecond, Some(Arc::from("UTC"))),
            true,
        )]));
        let empty_batch = arrow::record_batch::RecordBatch::new_empty(schema.clone());
        let ctx = build_session_context(50 * 1024 * 1024)
            .expect("SessionContext construction must succeed");
        register_mem_table(&ctx, "t", vec![empty_batch]).expect("table registration must succeed");

        let sql = format!("SELECT * FROM t WHERE ts_col > {emitted_fragment}");
        let plan_result = ctx.sql(&sql).await;
        assert!(
            plan_result.is_ok(),
            "RG-010 stage-2: arrow_cast emitter output must plan successfully against \
             Timestamp(Microsecond, UTC) column. Query: {sql:?}. Error: {:?}. \
             Root cause: quoting or escaping mistake in the arrow_cast type string — \
             DataFusion rejected the malformed type argument. Fix the format string in \
             literal_to_sql.",
            plan_result.err()
        );
    }

    // ── RG-024 (stub y): emitter guard reachability test ─────────────────────

    /// RG-024 (stub y): `literal_to_sql(Literal::RawTemporalLiteral(_))` MUST return
    /// `Err(QueryPlanFailed)` — the emitter guard is a belt-and-suspenders defense ensuring
    /// that no `RawTemporalLiteral` ever reaches SQL emission without being resolved by
    /// `check_temporal_literals` first.
    ///
    /// HIGH-3 fix: changed from `QueryParseFailed` to `QueryPlanFailed` because the literal
    /// was parsed successfully — this is a plan-time invariant violation, not a parse failure.
    ///
    /// Traces to: BC-2.11.021 guard arm (belt-and-suspenders);
    /// ADR-052 §D4 Step 5 (emitter guard); ADR-052 §D4 Task 11B (return type change).
    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_S_PRISMQL_NATIVE_TEMPORAL_TYPING_001_emitter_guard_raw_temporal_literal() {
        use prism_core::error::PrismError;
        // Belt-and-suspenders: if a RawTemporalLiteral reaches the emitter without being
        // consumed by check_temporal_literals, literal_to_sql MUST return
        // Err(QueryPlanFailed) — never panic and never silently emit a bare string.
        // HIGH-3 fix: QueryPlanFailed (plan-time invariant), not QueryParseFailed (parse failure).
        // Traces to: ADR-052 §D4 Step 5 (emitter guard); BC-2.11.021 guard arm.
        let raw = Literal::RawTemporalLiteral("2026-06-24".to_string());
        let result = literal_to_sql(&raw);
        assert!(
            result.is_err(),
            "RG-024: literal_to_sql(RawTemporalLiteral) must return Err, got Ok({:?})",
            result.ok()
        );
        let err = result.unwrap_err();
        assert!(
            matches!(&err, PrismError::QueryPlanFailed { .. }),
            "RG-024: literal_to_sql(RawTemporalLiteral) must return Err(QueryPlanFailed), got {err:?}"
        );
        // OBS-4 fix: assert message content identifies the anomaly (not just the variant).
        if let PrismError::QueryPlanFailed { detail } = &err {
            assert!(
                detail.contains("RawTemporalLiteral"),
                "RG-024: QueryPlanFailed detail must mention 'RawTemporalLiteral', got: {detail}"
            );
            assert!(
                detail.contains("internal error"),
                "RG-024: QueryPlanFailed detail must mention 'internal error', got: {detail}"
            );
        }
    }
}
