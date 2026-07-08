//! `prism-query` — Prism query orchestration crate.
//!
//! Created in S-2.08 with the pure-data `_source_type` virtual field injection
//! function. S-3.01 adds the PrismQL parser (filter/SQL/pipe modes via Chumsky 0.12).
//! S-3.02 extends this crate with the DataFusion `TableProvider` integration,
//! `QueryEngine`, and the full ephemeral materialization pipeline.
//! S-3.06 extends the parser with write-mode productions (pipe terminal write stages,
//! SQL DML statements, filter-mode write rejection).
//!
//! # Architecture Compliance (S-3.01)
//! Parser modules MUST NOT import from `prism-sensors`, `prism-mcp`, or any I/O
//! crate. Parsing is a pure function: `&str -> Result<Ast, Vec<ParseError>>`.
//!
//! # Architecture Compliance (S-3.02 / BC-2.11.006 / INV-SEC-PERIMETER-001)
//! Materialization code consumes the parser ONLY via `PrismQlParser::parse`.
//! Restricted sub-parser symbols MUST NOT appear in any S-3.02 module.
//!
//! # Architecture Compliance (S-3.06)
//! Write parser extensions are pure: `WriteVerbRegistry` is initialized once before
//! parse calls and is immutable during parsing — no `WriteEndpointRegistry` I/O
//! during a parse call (BC-2.11.004 purity rule).
//!
//! # Modules
//! - [`types`]                — `SensorQueryDescriptor` struct (table routing context, S-2.08)
//! - [`materialization`]      — ephemeral materialization pipeline + `inject_source_type()` (S-2.08/S-3.02)
//! - [`org_scoped_session_id`] — org-scoped UUID v7 session ID generation for sensor pagination (S-3.2.08 / D-048)
//! - [`ast`]                  — PrismQL AST types: `FilterExpr`, `SqlQuery`, `PipeQuery`, `Expr`, etc. (S-3.01)
//! - [`write_ast`]            — Write mode AST types: `WriteNode`, `DmlNode`, `WriteArg`, `Assignment` (S-3.06)
//! - [`write_verb_registry`]  — `WriteVerbRegistry` wrapping `WriteEndpointRegistry` or test `HashSet` (S-3.06)
//! - [`error`]                — `ParseError` type and ariadne-based error formatting (S-3.01)
//! - [`filter_parser`]        — filter mode parser: `source | predicate` (S-3.01 / BC-2.11.002)
//! - [`sql_parser`]           — SQL mode parser: `SELECT … FROM … JOIN … WHERE …` (S-3.01 / BC-2.11.003)
//! - [`pipe_parser`]          — pipe mode parser: `source | stage | stage` (S-3.01 / BC-2.11.004)
//! - [`security`]             — query size, nesting depth, and stage count guards (S-3.01 / BC-2.11.006)
//! - [`error_recovery`]       — Chumsky recovery strategies shared across parsers (S-3.01)
//! - [`engine`]               — `QueryEngine` struct, `execute()`, `execute_scheduled()` (S-3.02 / BC-2.11.001)
//! - [`pushdown`]             — predicate push-down classification (S-3.02 / BC-2.11.007)
//! - [`scoping`]              — cross-client scope resolution (S-3.02 / BC-2.11.011)
//! - [`virtual_fields`]       — `_sensor`, `_client`, `_source_table` injection (S-3.02 / BC-2.11.012)
//! - [`memory`]               — GreedyMemoryPool + error mapping (S-3.02 / BC-2.11.006)
//! - [`session`]              — `SessionScope` RAII wrapper (S-3.02 / BC-2.11.005)
//! - [`internal_tables`]      — `RocksDbTableProvider` DataFusion integration (S-3.02 / BC-2.15.011)
//! - [`cursor`]               — ephemeral internal pagination cursor for sensor fetch loops (S-3.05 / BC-2.07.001/002)
//! - [`cache_key`]            — SHA-256 cache key derivation, 4-tuple `(client_id, sensor_id, source_id, push_down_hash)` (S-3.05 / BC-2.07.005)
//! - [`cache`]                — sensor-fetch response cache with TTL and LRU eviction (S-3.05 / BC-2.07.003/006)
//! - [`invalidation`]         — synchronous cache invalidation on write operations (S-3.05 / BC-2.07.004)
//! - [`write_pipeline`]       — `WriteExecutor`, `WritePlan`, `WriteOutcome`, `QueryContext` (S-3.07)
//! - [`write_result`]         — `WriteResult`, `WritePreview`, `ConfirmationTokenPreview` (S-3.07)
//! - [`safety_check`]         — Phase 2 pure safety pre-check: feature gates, risk tier (S-3.07)
//! - [`dry_run`]              — Phase 4 dry-run gate, confirmation token gating (S-3.07)
//! - [`write_dispatch`]       — Phase 5 audit intent, semaphore, fan-out, outcome (S-3.07)
//! - [`write_table_registration`] — DataFusion write-capable TableProvider registration (S-3.07)

// ── S-2.08 modules ────────────────────────────────────────────────────────────
pub mod materialization;
pub mod org_scoped_session_id;
pub mod types;

// ── S-3.01 modules ────────────────────────────────────────────────────────────
pub mod ast;
pub mod error;
pub mod error_recovery;
pub mod filter_parser;
pub mod pipe_parser;
pub mod security;
pub mod sql_parser;
pub mod visit;

// ── S-3.02 modules ────────────────────────────────────────────────────────────
pub mod engine;
pub mod internal_tables;
pub mod memory;
pub mod pushdown;
pub mod scoping;
pub mod session;
pub mod virtual_fields;

// ── S-3.03 modules ────────────────────────────────────────────────────────────
pub mod explain;

// ── S-3.06 modules ────────────────────────────────────────────────────────────
pub mod write_ast;
pub mod write_verb_registry;

// ── S-3.04 modules — alias system ─────────────────────────────────────────────
pub mod alias_capability;
pub mod alias_resolver;
pub mod alias_store;
pub mod alias_tools;
pub mod alias_types;

// ── S-3.05 modules ────────────────────────────────────────────────────────────
pub mod cache;
pub mod cache_key;
pub mod cursor;
pub mod invalidation;

// ── S-DEMO-ENRICHMENT-PIVOT-001 modules ───────────────────────────────────────
// Infusion enrichment UDF registration for DataFusion SessionContext (BC-2.19.001).
pub mod infusion_udf;
// No-op CacheBackend placeholder for InfusionTier3Cache before real RocksDB is wired.
pub(crate) mod null_cache;

// ── ENRICH-4-B modules ────────────────────────────────────────────────────────
// Pipe-to-SQL emitter — lowers PipeQuery AST to DataFusion executable SQL (BC-2.11.004).
pub(crate) mod pipe_sql_emitter;

// ── S-3.13 modules ────────────────────────────────────────────────────────────
/// Dynamic table registry — tracks which sensor tables are currently available.
/// Populated from `ConfigSnapshot.sensor_specs` at startup; updated on hot-reload.
/// Used by the plan-time availability gate in `engine.rs` (E-QUERY-037 / BC-2.11.001).
pub mod table_registry;

// ── S-3.07 modules ────────────────────────────────────────────────────────────
pub mod dry_run;
pub mod safety_check;
pub mod write_dispatch;
pub mod write_pipeline;
pub mod write_result;
// MED-2: was pub(crate) pending CRIT-2 implementation; now pub since
// WriteCapableTableProvider::new is implemented and externally testable.
pub mod write_table_registration;

// ── Kani proofs (cfg-gated; compile everywhere, run only under cargo kani) ────
pub mod proofs;

// ── Unit tests ────────────────────────────────────────────────────────────────
#[cfg(test)]
pub mod tests;

// ── S-3.01 re-exports ─────────────────────────────────────────────────────────
//
// # Security perimeter (B-3, BC-2.11.006, SEC-C-003, F-LOW-002)
//
// `PrismQlParser::parse` and `PrismQlParser::parse_with_registry` are the public
// security entry points. Both apply:
//   1. `check_query_size` — rejects inputs > 64KB before any parsing
//   2. `check_paren_depth` — rejects inputs with > 64 lexical paren depth
//   3. Mode detection — dispatches to `parse_sql`, `parse_pipe`, or `parse_filter`
// `parse_with_registry` additionally routes pipe mode through `parse_pipe_with_write`
// and filter mode through `reject_write_verbs_in_filter` (BC-2.11.004, F-PR130-CR-001).
//
// The following symbols are `pub(crate)` and MUST NOT be exposed externally.
// Authoritative source: BC-2.11.006 frontmatter `restricted_symbols`.
//
// Sub-parsers:
//   `parse_filter`, `parse_filter_with_limits`
//   `parse_sql`, `parse_sql_with_limits`
//   `parse_pipe`, `parse_pipe_with_limits`
//
// Parser-builder factories:
//   `build_predicate_parser`, `build_source_ref_parser`,
//   `build_string_parser`, `build_literal_parser`,
//   `build_expr_parser`, `build_pipe_mode_parser`,
//   `build_pipe_parser`
//
// ParseLimits API:
//   `ParseLimits::install_thread_local`, `ParseLimits::clear_thread_local`,
//   `ParseLimits::current_regex_limit`, `ParseLimits::snapshot`,
//   `ParseLimits` struct fields
//
// Drop guard:
//   `ThreadLocalGuard` (filter_parser) — `pub(crate)` for unit-test
//   verification of Drop semantics; not part of the stable API.
//
// Write-parser internals (S-3.06, BC-2.11.004 + BC-2.11.006 DI-034 layer 4):
//   `parse_pipe_with_write`, `build_write_stage_parser`,
//   `build_write_arg_parser`, `extract_sensor_prefix` (pipe_parser)
//   `parse_sql_dml`, `parse_sql_dml_with_limits`,
//   `is_internal_prism_table`, `check_unbounded_write` (sql_parser)
//   `reject_write_verbs_in_filter` (filter_parser)
//
// Alias-system internals (S-3.04, BC-2.11.008 + BC-2.11.006 DI-034 layer 5):
//   `create_alias` (alias_tools)                          — ungated create; MCP MUST use *_gated (SEC-011)
//   `create_alias_with_clients` (alias_tools)             — ungated create+clients; MCP MUST use *_gated
//   `create_alias_with_clients_gated_inner` (alias_tools) — internal token-store split (F-LOCAL-P2-HIGH-005)
//   `delete_alias` (alias_tools)                          — ungated delete; MCP MUST use *_gated (SEC-011)
//   `AliasStore` (alias_store) — `create_or_update` method pub(crate): direct store mutation; bypasses guards (CR-018)
//
// Tests that need direct sub-parser access (e.g., to obtain
// FilterExpr/PipeQuery/SqlQuery directly, or to bypass pre-parse guards to
// test post-parse depth checks in isolation) must live in src/tests/ (unit
// tests) where pub(crate) items are visible.
//
// External consumers MUST use `PrismQlParser::parse` or `PrismQlParser::parse_with_registry`.
pub use ast::Ast;
pub use error::ParseError;
pub use filter_parser::PrismQlParser;
// ── S-3.07 re-exports ─────────────────────────────────────────────────────────
pub use write_pipeline::{QueryContext, WriteExecutor, WriteOutcome, WritePlan};
pub use write_result::{ConfirmationTokenPreview, SensorWriteError, WritePreview, WriteResult};
pub use write_verb_registry::WriteVerbRegistry;

// ── S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001 plan-time gates ───────────────────
// These public functions implement the plan-time gates for Areas A (SqlPipe) and
// B (NOW() temporal injection) per the story acceptance criteria.

/// Plan a parsed `SqlPipeQuery` AST: applies the FORBID-BOTH E-QUERY-040 check
/// (BC-2.11.020 postcondition 5, ADR-043 §C §D4).
///
/// Returns `Ok(())` on a valid plan, or `Err(PrismError::RedundantRowLimit)`
/// when both the SQL head LIMIT and a pipe `| limit` stage are present.
///
/// # FORBID-BOTH invariant (ADR-043 §C §D4 — permanent ruling)
/// It is a hard error to have both `SELECT … LIMIT n` and `| limit m` in the
/// same SqlPipe query. The caller must remove one.  This is a plan-time check
/// (not a parse-time check) so that the AST is available for inspection.
pub fn plan_sqlpipe_query(spq: &ast::SqlPipeQuery) -> Result<(), prism_core::error::PrismError> {
    use ast::PipeStage;

    // Find the SQL head LIMIT, if any.
    if let Some(sql_limit) = spq.head.limit {
        // Find the first pipe stage that imposes a row cap (lowers to SQL LIMIT).
        //
        // Row-capping PipeStage variants (determined by pipe_sql_emitter.rs apply_stage):
        //   - `Limit(n)` → `LIMIT n` (direct limit stage)
        //   - `Tail(n)`  → `LIMIT n` (lowered via apply_limit; semantic gap §3.2)
        //
        // Non-capping variants (verified in pipe_sql_emitter.rs apply_stage):
        //   - Where, Sort, Stats, Fields, Dedup, Enrich — none lower to LIMIT/OFFSET.
        //   - Join — returns Err (unsupported); no row cap emitted.
        //
        // HIGH-1 fix: include Tail in addition to Limit so `SELECT … LIMIT n | tail m`
        // is rejected with FORBID-BOTH (ADR-043 §D4 / INV-FORBID-BOTH-PERMANENT).
        for stage in &spq.stages {
            let pipe_limit = match stage {
                PipeStage::Limit(n) => Some(*n),
                PipeStage::Tail(n) => Some(*n),
                // All other variants do not lower to a SQL LIMIT — no row cap.
                _ => None,
            };
            if let Some(pipe_limit) = pipe_limit {
                return Err(prism_core::error::PrismError::RedundantRowLimit {
                    sql_limit,
                    pipe_limit,
                });
            }
        }
    }
    Ok(())
}

/// Parse a PrismQL string and apply plan-time constant injection (BC-2.11.021, ADR-044).
///
/// Returns `Ok(Ast)` with all `Expr::Now` nodes replaced by
/// `Expr::Literal(Literal::Timestamp(now))` where `now` is captured ONCE at the
/// start of planning.  All occurrences of `NOW()` in the query are substituted
/// with the same instant, so multi-predicate queries behave consistently.
///
/// # Implements BC-2.11.021 postcondition — planning-time constant injection
pub fn parse_and_plan(input: &str) -> Result<Ast, Vec<ParseError>> {
    use ast::{Expr, Literal, TimestampLiteral};
    use chrono::Utc;

    let parsed = PrismQlParser::parse(input)?;

    // Capture NOW() once for the entire planning session.
    let now: chrono::DateTime<Utc> = Utc::now();
    let now_iso = now.to_rfc3339();
    let now_ts = TimestampLiteral {
        iso8601: now_iso,
        instant: now,
    };
    let now_literal_expr = Expr::Literal(Literal::Timestamp(now_ts));

    inject_now(parsed, &now_literal_expr)
}

/// Recursively replace all `Expr::Now` nodes in `ast` with `now_literal`.
///
/// Returns `Ok(Ast)` with all `Expr::Now` nodes replaced on success.
/// Returns `Err(Vec<ParseError>)` when the constant-fold of a
/// `TimestampArithmetic` node overflows the `DateTime<Utc>` representable range
/// (F-P3-FRESH-CRIT-001 — VP-021 compliance, BC-2.11.021).
///
/// `pub(crate)` so `materialization.rs` can call it directly (BC-2.11.021 wiring).
pub(crate) fn inject_now(ast: Ast, now_literal: &ast::Expr) -> Result<Ast, Vec<error::ParseError>> {
    use ast::{Ast as A, FilterExpr, PipeQuery, SqlPipeQuery, SqlStatement};

    // Internal helpers return `Result<T, error::ParseError>` (single error).
    // The public API returns `Result<Ast, Vec<error::ParseError>>`.
    // Convert single → vec with `.map_err(|e| vec![e])?`.
    let result = match ast {
        A::Filter(fe) => A::Filter(FilterExpr {
            source: fe.source,
            predicate: inject_now_predicate(fe.predicate, now_literal).map_err(|e| vec![e])?,
        }),
        A::Sql(SqlStatement::Select(sq)) => A::Sql(SqlStatement::Select(
            inject_now_sql_query(sq, now_literal).map_err(|e| vec![e])?,
        )),
        A::Sql(other) => A::Sql(other), // DML — no NOW() injection needed
        A::Pipe(pq) => {
            let mut folded_stages = Vec::with_capacity(pq.stages.len());
            for s in pq.stages {
                folded_stages.push(inject_now_pipe_stage(s, now_literal).map_err(|e| vec![e])?);
            }
            A::Pipe(PipeQuery {
                source: pq.source,
                stages: folded_stages,
                write: pq.write,
            })
        }
        A::SqlPipe(spq) => {
            let mut folded_stages = Vec::with_capacity(spq.stages.len());
            for s in spq.stages {
                folded_stages.push(inject_now_pipe_stage(s, now_literal).map_err(|e| vec![e])?);
            }
            A::SqlPipe(SqlPipeQuery {
                head: inject_now_sql_query(spq.head, now_literal).map_err(|e| vec![e])?,
                stages: folded_stages,
            })
        }
    };
    Ok(result)
}

fn inject_now_sql_query(
    mut sq: ast::SqlQuery,
    now_literal: &ast::Expr,
) -> Result<ast::SqlQuery, error::ParseError> {
    // Fold NOW() in WHERE and HAVING predicates.
    sq.where_ = sq
        .where_
        .map(|p| inject_now_predicate(p, now_literal))
        .transpose()?;
    sq.having = sq
        .having
        .map(|p| inject_now_predicate(p, now_literal))
        .transpose()?;

    // Fold NOW() in SELECT projection expressions.
    // `sql_query_has_unfolded_temporal` detects these — fold must mirror detect.
    let mut new_items = Vec::with_capacity(sq.select.items.len());
    for item in sq.select.items {
        new_items.push(match item {
            ast::SelectItem::Expr { expr, alias } => ast::SelectItem::Expr {
                expr: inject_now_expr(expr, now_literal)?,
                alias,
            },
            other => other,
        });
    }
    sq.select.items = new_items;

    // Fold NOW() in GROUP BY expressions.
    // `sql_query_has_unfolded_temporal` detects these — fold must mirror detect.
    let mut new_group_by = Vec::with_capacity(sq.group_by.len());
    for e in sq.group_by {
        new_group_by.push(inject_now_expr(e, now_literal)?);
    }
    sq.group_by = new_group_by;

    // Fold NOW() in ORDER BY expressions.
    // `sql_query_has_unfolded_temporal` detects these — fold must mirror detect.
    let mut new_order_by = Vec::with_capacity(sq.order_by.len());
    for oe in sq.order_by {
        new_order_by.push(ast::OrderExpr {
            expr: inject_now_expr(oe.expr, now_literal)?,
            direction: oe.direction,
        });
    }
    sq.order_by = new_order_by;

    // Fold NOW() in JOIN ON expressions.
    // `sql_query_has_unfolded_temporal` detects these — fold must mirror detect.
    let mut new_joins = Vec::with_capacity(sq.joins.len());
    for j in sq.joins {
        new_joins.push(ast::Join {
            kind: j.kind,
            source: j.source,
            alias: j.alias,
            on: inject_now_expr(j.on, now_literal)?,
        });
    }
    sq.joins = new_joins;

    Ok(sq)
}

fn inject_now_pipe_stage(
    stage: ast::PipeStage,
    now_literal: &ast::Expr,
) -> Result<ast::PipeStage, error::ParseError> {
    use ast::PipeStage;
    match stage {
        PipeStage::Where(pred) => Ok(PipeStage::Where(inject_now_predicate(pred, now_literal)?)),
        other => Ok(other),
    }
}

fn inject_now_predicate(
    pred: ast::Predicate,
    now_literal: &ast::Expr,
) -> Result<ast::Predicate, error::ParseError> {
    use ast::Predicate;
    match pred {
        Predicate::Compare {
            lhs,
            op,
            rhs,
            case_insensitive,
        } => Ok(Predicate::Compare {
            lhs: Box::new(inject_now_expr(*lhs, now_literal)?),
            op,
            rhs: Box::new(inject_now_expr(*rhs, now_literal)?),
            case_insensitive,
        }),
        Predicate::Logical { op, predicates } => {
            let mut folded = Vec::with_capacity(predicates.len());
            for p in predicates {
                folded.push(inject_now_predicate(p, now_literal)?);
            }
            Ok(Predicate::Logical {
                op,
                predicates: folded,
            })
        }
        Predicate::Not(inner) => Ok(Predicate::Not(Box::new(inject_now_predicate(
            *inner,
            now_literal,
        )?))),
        // F-P2-MED-001: InSubquery — fold NOW() inside the nested subquery's
        // WHERE and HAVING clauses.  The detection side (`predicate_has_unfolded_temporal`)
        // recurses into `InSubquery { subquery }` via `sql_query_has_unfolded_temporal`.
        // The fold side must mirror that recursion or `normalize` returns None and the
        // query is wrongly rejected with a generic E-QUERY-034 internal error.
        Predicate::InSubquery {
            field,
            subquery,
            negated,
        } => Ok(Predicate::InSubquery {
            field,
            subquery: Box::new(inject_now_sql_query(*subquery, now_literal)?),
            negated,
        }),
        // All other predicate variants do not contain Expr::Now.
        other => Ok(other),
    }
}

fn inject_now_expr(
    expr: ast::Expr,
    now_literal: &ast::Expr,
) -> Result<ast::Expr, error::ParseError> {
    use ast::Expr;
    match expr {
        Expr::Now => Ok(now_literal.clone()),
        Expr::TimestampArithmetic { base, op, offset } => {
            let folded_base = inject_now_expr(*base, now_literal)?;
            // BC-2.11.021 constant-fold: if the base resolved to a bare Timestamp literal,
            // compute `t ± offset` immediately so `extract_time_bounds_from_predicate`
            // can match the RHS as `Expr::Literal(Literal::Timestamp(_))` for push-down
            // (ADR-033 T1). Without folding, the outer `TimestampArithmetic` wrapper
            // blocks push-down extraction (pushdown.rs requires a bare Literal::Timestamp).
            if let Expr::Literal(ast::Literal::Timestamp(ref ts)) = folded_base {
                // F-P3-FRESH-CRIT-001 fix (Site 2): replace the panicking `ts.instant - offset`
                // / `ts.instant + offset` operators with `checked_sub_signed` /
                // `checked_add_signed`. These return `None` when the resulting DateTime
                // falls outside the representable range (e.g. subtracting a max-magnitude
                // Duration from a near-epoch timestamp underflows to before year -262,000).
                // Map `None` → structured `E-QUERY-001` parse error (VP-021 compliance).
                let computed_opt = match op {
                    ast::BinaryOp::Sub => ts.instant.checked_sub_signed(offset),
                    ast::BinaryOp::Add => ts.instant.checked_add_signed(offset),
                };
                let computed = computed_opt.ok_or_else(|| {
                    error::ParseError::new(
                        0,
                        format!(
                            "E-QUERY-001: timestamp arithmetic overflow: \
                             NOW() {} INTERVAL results in a DateTime outside \
                             the representable range",
                            match op {
                                ast::BinaryOp::Sub => "-",
                                ast::BinaryOp::Add => "+",
                            }
                        ),
                    )
                })?;
                let iso = computed.to_rfc3339();
                Ok(Expr::Literal(ast::Literal::Timestamp(
                    ast::TimestampLiteral {
                        iso8601: iso,
                        instant: computed,
                    },
                )))
            } else {
                Ok(Expr::TimestampArithmetic {
                    base: Box::new(folded_base),
                    op,
                    offset,
                })
            }
        }
        Expr::Compare { lhs, op, rhs } => Ok(Expr::Compare {
            lhs: Box::new(inject_now_expr(*lhs, now_literal)?),
            op,
            rhs: Box::new(inject_now_expr(*rhs, now_literal)?),
        }),
        Expr::Logical { lhs, op, rhs } => Ok(Expr::Logical {
            lhs: Box::new(inject_now_expr(*lhs, now_literal)?),
            op,
            rhs: Box::new(inject_now_expr(*rhs, now_literal)?),
        }),
        Expr::Not(inner) => Ok(Expr::Not(Box::new(inject_now_expr(*inner, now_literal)?))),
        // FuncCall (aggregate / scalar): fold NOW() inside argument expressions.
        // `expr_has_unfolded_temporal` recurses into FuncCall args — fold must mirror detect.
        Expr::FuncCall(fc) => {
            use ast::FuncCall;
            let folded_fc = match fc {
                FuncCall::Aggregate {
                    func,
                    args,
                    distinct,
                } => {
                    let mut folded_args = Vec::with_capacity(args.len());
                    for a in args {
                        folded_args.push(inject_now_expr(a, now_literal)?);
                    }
                    FuncCall::Aggregate {
                        func,
                        args: folded_args,
                        distinct,
                    }
                }
                FuncCall::Scalar { func, args } => {
                    let mut folded_args = Vec::with_capacity(args.len());
                    for a in args {
                        folded_args.push(inject_now_expr(a, now_literal)?);
                    }
                    FuncCall::Scalar {
                        func,
                        args: folded_args,
                    }
                }
                // Window functions carry no expression args today; pass through.
                other => other,
            };
            Ok(Expr::FuncCall(folded_fc))
        }
        // Expr::InSubquery (value context): fold NOW() inside the subquery's clauses.
        // `expr_has_unfolded_temporal` now recurses into Expr::InSubquery via
        // `sql_query_has_unfolded_temporal` — fold must mirror detect.
        // (The prior comment claiming "symmetric mutual-omission" was incorrect;
        // a subquery in value context can have temporal WHERE/HAVING/SELECT/etc.)
        Expr::InSubquery { field, subquery } => Ok(Expr::InSubquery {
            field,
            subquery: Box::new(inject_now_sql_query(*subquery, now_literal)?),
        }),
        // Literal, Field, VirtualField, In, Star, Interval — no NOW() to inject.
        other => Ok(other),
    }
}
